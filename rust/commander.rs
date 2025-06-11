extern crate simjson;
extern crate simweb;
extern crate simtime;
use std::{io::{self,Read,stdin,Write}, fmt::Write as FmtWrite, 
    fs::read_dir, time::UNIX_EPOCH,
};

use simjson::JsonData::{Data,Text};

const MAX_BLOCK_LEN : usize = 40960;

fn main() -> io::Result<()> {
    let web = simweb::WebData::new();
    let mut buffer = [0_u8;MAX_BLOCK_LEN];
    loop {
        let Ok(len) = stdin().read(&mut buffer) else {break};
        if len == 0 { break }
        let Data(json) = simjson::parse(&String::from_utf8_lossy(&buffer[0..len])) else {
            continue
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
                    print!(r#"{{"panel":"{panel}", "dir":[{}]}}"#, get_dir(dir).unwrap());
                    io::stdout().flush()?;
                }
                _ => continue
            }
            _ => continue
        }
    }
    
    Ok(())
}

fn get_dir(dir: &str) -> Result<String,std::io::Error> {
    Ok(read_dir(dir)?.fold(String::new(),
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