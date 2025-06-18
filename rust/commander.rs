extern crate simjson;
extern crate simweb;
extern crate simtime;
use std::{io::{self,Read,stdin,Write}, fmt::Write as FmtWrite, 
    fs::{self,read_dir}, time::UNIX_EPOCH, path::{PathBuf,Path},
};

use simjson::{JsonData::{Data,Text,Arr},parse_fragment};
use simweb::{json_encode,html_encode};

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
                        let Some(Arr(files)) = json.get("files") else {
                            eprintln!("no files to copy");
                            continue
                        };
                        let Some(Text(src)) = json.get("src") else {
                            eprintln!("no src of copy");
                            continue
                        };
                        let mut src_path = PathBuf::from(&src);
                        if src_path.is_file() {
                            eprintln!("src should be dir");
                            continue
                        }
                        let Some(Text(dst)) = json.get("dst") else {
                            eprintln!("no dst to copy");
                            continue
                        };
                        
                        let mut dst_path = PathBuf::from(&dst);
                        for file in files {
                            let Text(file) = file else { continue };
                            src_path.push(file.clone());
                            dst_path.push(file);
                            fs::copy(&src_path,&dst_path).unwrap();
                            src_path.pop();
                            dst_path.pop();
                        }
                        println!(r#"{{"panel":"{panel}", "dir":[{}]}}"#, get_dir(&dst).unwrap());
                        io::stdout().flush()?;
                        let other_panel = if panel == "left" { "right" } else { "left" };
                        println!(r#"{{"panel":"{other_panel}", "dir":[{}]}}"#, get_dir(&src).unwrap());
                        io::stdout().flush()?;
                        //eprintln!("copy {:?} -> {:?} : {:?}",json.get("src"), json.get("dst"), json.get("files"))
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
                        let mut src_path = PathBuf::from(&src);
                        if src_path.is_file() {
                            eprintln!("src should be dir");
                            continue
                        }
                        let Some(Text(dst)) = json.get("dst") else {
                            eprintln!("no dst to move");
                            continue
                        };
                        let mut dst_path = PathBuf::from(&dst);
                        let mut was_move = false;
                        if dst_path .is_file()  {
                            if files.len() == 1 {
                                if let Text(file) = &files[0] {
                                    src_path.push(file);
                                    fs::rename(src_path,dst_path).unwrap();
                                    was_move = true
                                }
                            }
                        } else if files.len() > 0 {
                            for file in files {
                                let Text(file) = file else { continue };
                                src_path.push(file.clone());
                                dst_path.push(file);
                                fs::rename(&src_path,&dst_path).unwrap();
                                src_path.pop();
                                dst_path.pop();
                            }
                            was_move = true
                        }
                        if was_move {
                             println!(r#"{{"panel":"{panel}", "dir":[{}]}}"#, get_dir(&dst).unwrap());
                            io::stdout().flush()?;
                            let other_panel = if panel == "left" { "right" } else { "left" };
                            println!(r#"{{"panel":"{other_panel}", "dir":[{}]}}"#, get_dir(&src).unwrap());
                            io::stdout().flush()?;
                        }
                    }
                    "del" => {
                        let Some(Arr(files)) = json.get("files") else {
                            eprintln!("no files to delete");
                            continue
                        };
                        let Some(Text(src)) = json.get("src") else {
                            eprintln!("no src to delete");
                            continue
                        };
                        let mut src_path = PathBuf::from(&src);
                        for file in files {
                            let Text(file) = file else { continue };
                            src_path.push(file);
                            if src_path.is_file() {
                                fs::remove_file(&src_path).unwrap();
                            } else if src_path.is_dir() {
                                fs::remove_dir_all(&src_path).unwrap();
                            }
                            src_path.pop();
                        }
                        println!(r#"{{"panel":"{panel}", "dir":[{}]}}"#, get_dir(&src).unwrap());
                        io::stdout().flush()?;
                    }
                    "mkdir" => {
                        let Some(Text(src)) = json.get("src") else {
                            eprintln!("no src where to create");
                            continue
                        };
                        let Some(Text(file)) = json.get("file") else {
                            eprintln!("no dir name to create");
                            continue
                        };
                        let mut create_path = PathBuf::from(&src);
                        create_path.push(file);
                        fs::create_dir(&create_path).unwrap();
                        println!(r#"{{"panel":"{panel}", "dir":[{}]}}"#, get_dir(&src).unwrap());
                        io::stdout().flush()?;
                    }
                    "show" => {
                        let Some(Text(src)) = json.get("src") else {
                            eprintln!("no src of what to show");
                            continue
                        };
                        let Some(Text(file)) = json.get("file") else {
                            eprintln!("no file name to show");
                            continue
                        };
                        let mut show_path = PathBuf::from(&src);
                        show_path.push(file);
                        if show_path.is_file() {
                            let file_contents = fs::read_to_string(show_path)?;
                             println!(r#"{{"panel":"center", "content":"{}"}}"#, json_encode(&html_encode(&file_contents)));
                            io::stdout().flush()?;
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
    if let Some(parent_path) = path.parent() {
        let timestamp = fs::metadata(parent_path)?.modified()?;
        write!(init,r#"{{"name":"{}", "dir":{}, "timestamp":{}}}"#,"..",true,timestamp.duration_since(UNIX_EPOCH).unwrap().as_millis()).unwrap()
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