extern crate simjson;
extern crate simweb;
extern crate simtime;
use std::{io::{self,Read,stdin,Write}, fmt::Write as FmtWrite, 
    fs::{self,read_dir}, time::UNIX_EPOCH, path::{PathBuf,Path},
};

use simjson::{JsonData::{Data,Text,Arr},parse_fragment};

const MAX_BLOCK_LEN : usize = 40960;

fn main() -> io::Result<()> {
    let web = simweb::WebData::new();
    let mut buffer = [0_u8;MAX_BLOCK_LEN];
    loop {
        let Ok(len) = stdin().read(&mut buffer) else {break};
        if len == 0 { break }
        if len == 4 && buffer[0] == 255 && buffer[1] == 255 && buffer[2] == 255 && buffer[3] == 4 {
            // ws close
            break
        }
        //eprintln!("parsing {}", String::from_utf8_lossy(&buffer[0..len]));
        let commands = String::from_utf8_lossy(&buffer[0..len]);

        let mut chars = commands.chars();
        loop {
            let res = parse_fragment(&mut chars);
            let json = match res.0 {
                Data(json) => json,
                _ => break,
            };
        
            let Some(Text(panel)) = json.get("panel") else {
                    continue
            };
            
            match json.get("op") {
                Some(Text(op)) => match op.as_str() {
                    "dir" => {
                        //eprintln!("{json:?}");
                        let Some(Text(dir)) = json.get("dir") else {
                            continue
                        };
                        println!(r#"{{"panel":"{panel}", "dir":[{}]}}"#, get_dir(dir).unwrap());
                        io::stdout().flush()?;
                    }
                    "copy" => {
                        eprintln!("copy {:?} -> {:?} : {:?}",json.get("src"), json.get("dst"), json.get("files"))
                    }
                    "move" => {
                        let Some(Arr(files)) = json.get("files") else {
                            eprintln!("no files to move");
                            continue
                        };
                        let Some(Text(src)) = json.get("src") else {
                            eprintln!("no src to move");
                            continue
                        };
                        let mut src = PathBuf::from(&src);
                        if src.is_file() {
                            eprintln!("src should be dir");
                            continue
                        }
                        let Some(Text(dst)) = json.get("dst") else {
                            eprintln!("no dst to move");
                            continue
                        };
                        let mut dst = PathBuf::from(&dst);
                        if dst .is_file()  {
                            if files.len() == 1 {
                                if let Text(file) = &files[0] {
                                    src.push(file);
                                    fs::rename(src,dst).unwrap()
                                }
                            }
                        } else {
                            for file in files {
                                let Text(file) = file else { continue };
                                src.push(file.clone());
                                dst.push(file);
                                fs::rename(&src,&dst).unwrap();
                                src.pop();
                                dst.pop();
                            }
                        }
                    }
                    _ => continue
                }
                _ => continue
            }
        }
    }
    
    Ok(())
}

fn get_dir(dir: &str) -> Result<String,std::io::Error> {
    let mut init = String::new();
    let path = Path::new(&dir);
    if let Some(_path) = path.parent() {
        write!(init,r#"{{"name":"{}", "dir":{}}}"#,"..",true).unwrap()
    };
    Ok(read_dir(dir)?.fold(init,
       |mut res,cur| {if let Ok(cur) = cur {
            let md = cur.metadata().unwrap(); 
             write!(res,r#"{}{{"name":"{}", "dir":{}, "size":{}, "timestamp":{}}}"#, if res.is_empty() {""} else {","},
             simweb::json_encode(&cur.file_name().display().to_string()), // add json escape ""
             md.is_dir(),
             md.len(),md.modified().unwrap().duration_since(UNIX_EPOCH).unwrap().as_millis()).unwrap();
            }
         res
        }
    ))
}