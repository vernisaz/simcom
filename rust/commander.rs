extern crate simjson;
extern crate simweb;
extern crate simtime;
extern crate simzip;
use std::{io::{self,Read,stdin,Write}, fmt::Write as FmtWrite, 
    fs::{self,read_dir,}, time::{UNIX_EPOCH,SystemTime}, path::{PathBuf,Path},
    env::consts, env,
};

use simjson::{JsonData::{Data,Text,Arr,Num},parse_fragment};
use simweb::{json_encode,html_encode};
use simzip::{ZipEntry,ZipInfo};

const MAX_BLOCK_LEN : usize = 40960;

struct State {
    left: String,
    right: String,
}

fn main() -> io::Result<()> {
    let web = simweb::WebData::new();
    let mut buffer = [0_u8;MAX_BLOCK_LEN];
    let os_drive =
    if "windows" == consts::OS {
        match env::var("SystemDrive") {
            Ok(value) => value,
            Err(_e) => String::new(),
        }
    } else {
         String::new()
    };
    
    let mut state = State{
        left: format!{"{os_drive}{}", std::path::MAIN_SEPARATOR_STR},
        right:format!{"{os_drive}{}", std::path::MAIN_SEPARATOR_STR},
    };
    if let Some(stored_state) = read_state() {
        state.left = stored_state.left;
        state.right = stored_state.right;
    }
    println!(r#"{{"panel":"control", "system":"{}", "root":"{}", "separator":"{}","left":"{}", "right":"{}"}}"#, consts::OS,
        os_drive, json_encode(&std::path::MAIN_SEPARATOR_STR),json_encode(&state.left), json_encode(&state.right));
    io::stdout().flush()?;
    loop {
        let Ok(len) = stdin().read(&mut buffer) else {break};
        if len == 0 { break }
        if len == 4 && buffer[0] == 255 && buffer[1] == 255 && buffer[2] == 255 && buffer[3] == 4 {
            
            break
        }
        let commands = String::from_utf8_lossy(&buffer[0..len]);
        //eprintln!("parsing {commands}");
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
                        match get_dir(dir) {
                            Ok(dir) => {
                                println!(r#"{{"panel":"{panel}", "dir":[{dir}]}}"#);
                                io::stdout().flush()?;
                                if panel == "right" { state.right = dir } else { state.left = dir }
                            }
                            _ => eprintln!("an error in reading {dir} "),
                        }
                        
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
                        let mut need_copy = true;
                        if files.len() == 1 {
                            if let Some(Text(dst_file)) = json.get("file") {
                                if let Text(file) = &files[0] {
                                    dst_path.push(dst_file);
                                    src_path.push(file);
                                    fs::copy(&src_path,&dst_path).unwrap();
                                    need_copy = false;
                                }
                            }
                        }
                        if need_copy {
                            for file in files {
                                let Text(file) = file else { continue };
                                src_path.push(file.clone());
                                dst_path.push(file);
                                fs::copy(&src_path,&dst_path).unwrap();
                                src_path.pop();
                                dst_path.pop();
                            }
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
                        if let Some(Text(dst_file)) = json.get("file") {
                            dst_path.push(dst_file)
                        }
                        let mut was_move = false;
                        if dst_path .is_file() || !dst_path.exists() { // potential renaming can happen in a new directory
                            if files.len() == 1 {
                                if let Text(file) = &files[0] {
                                    src_path.push(file);
                                    match fs::rename(&src_path,&dst_path) {
                                        Ok(()) => was_move = true,
                                        Err(err) => eprintln!("not renamed {src_path:?} to {dst_path:?} because {err}")
                                    }
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
                        match fs::create_dir(&create_path) {
                            Ok(()) => { println!(r#"{{"panel":"{panel}", "dir":[{}]}}"#, get_dir(&src).unwrap());
                                io::stdout().flush()?;},
                            Err(_) => eprintln!("can't create dir {create_path:?}"),
                        }
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
                    "edit" => {
                        let Some(Text(src)) = json.get("src") else {
                            eprintln!("no src of what to edit");
                            continue
                        };
                        let Some(Text(file)) = json.get("file") else {
                            eprintln!("no file name to edit");
                            continue
                        };
                        let mut edit_path = PathBuf::from(&src);
                        edit_path.push(file);
                        if edit_path.is_file() {
                            let modified = get_file_modified(&edit_path);
                            let file_contents = fs::read_to_string(&edit_path)?;
                             println!(r#"{{"panel":"center", "op":"edit", "file":"{}", "content":"{}", "modified":{modified}}}"#, 
                                json_encode(&edit_path.display().to_string()), json_encode(&html_encode(&file_contents)));
                            io::stdout().flush()?;
                        } else if !edit_path.exists() {
                            println!(r#"{{"panel":"center", "op":"edit", "file":"{}", "content":""}}"#, 
                                json_encode(&edit_path.display().to_string()));
                            io::stdout().flush()?;
                        }
                    }
                    "save" => {
                        let Some(Text(file)) = json.get("file") else {
                            eprintln!("no file to save");
                            continue
                        };
                        let saved_modified = if let Some(Num(modified)) = json.get("modified") {
                            *modified as u64
                        } else {
                            0
                        };
                        let mut save_path = PathBuf::from(&file);
                        let (new_file, modified) = if save_path.exists() {
                            (false, get_file_modified(&save_path))
                        } else {
                            (true, 0)
                        };
                        if saved_modified < modified {
                            println!(r#"{{"panel":"info", "message":"The file can't be saved, because it's been already modified"}}"#);
                            io::stdout().flush()?;
                            continue
                        }
                        if save_path.is_file() || new_file {
                            let Some(Text(content)) = json.get("content") else {
                                eprintln!("no content to save");
                                continue
                            };
                            match fs::write(&save_path, content) {
                                Err(err) => {
                                    println!(r#"{{"panel":"info", "message":"Can't save because {}"}}"#, json_encode(&err.to_string()));
                                    io::stdout().flush()?;
                                    continue
                                }
                                _ => (),
                            }
                            let modified = get_file_modified(&save_path);
                            println!(r#"{{"panel":"info", "modified":{modified}}}"#);
                            io::stdout().flush()?;
                            if new_file {
                                save_path.pop();
                                println!(r#"{{"panel":"{panel}", "dir":[{}]}}"#, get_dir(&save_path.display().to_string()).unwrap());
                                io::stdout().flush()?;
                            }
                        }
                    }
                    "zip" => {
                        let Some(Arr(files)) = json.get("files") else {
                            eprintln!("no files to zip");
                            continue
                        };
                        let Some(Text(src)) = json.get("src") else {
                            eprintln!("no src of filws");
                            continue
                        };
                        let Some(Text(zip)) = json.get("zip") else {
                            eprintln!("no zip name");
                            continue
                        };
                        let mut src_path = PathBuf::from(&src);
                        src_path.push(zip);
                        let mut zip_file = ZipInfo::new_with_comment(&src_path, "The zip created using simcommander 1.02");
                        for file in files {
                            let Text(file) = file else { continue };
                            src_path.pop();
                            src_path.push(file);
                            if src_path . is_file() {
                                zip_file.add(ZipEntry::from_file(&src_path, Some("")));
                            } else if src_path . is_dir() {
                            }
                        }
                        match zip_file.store() {
                            Ok(()) => {
                                src_path.pop();
                                println!(r#"{{"panel":"{panel}", "dir":[{}]}}"#, get_dir(&src_path.display().to_string()).unwrap());
                                io::stdout().flush()?;
                            },
                            Err(msg) => {println!(r#"{{"panel":"info", "message":"Can't zip because {}"}}"#, json_encode(&format!("{msg:?}")));
                                io::stdout().flush()?;
                            }
                        }
                    }
                    _ => continue
                }
                _ => continue
            }
        }
    }
    // ws close
    let _ = save_state(state);
    Ok(())
}

fn read_state() -> Option<State> {
    let home = if "windows" == consts::OS {
        env::var("USERPROFILE")
        } else {
            env::var("HOME")
        };
    let Ok(home ) = home else {
        return None
    };
    let mut home = PathBuf::from(home);
    home.push(".sc");
    if home.exists() {
        let file_contents = fs::read_to_string(home).ok()?; 
        if let Some(pair) = file_contents.split_once('\n') {
            if let Some((panel,dir)) = pair.0.split_once('=' ) {
                if let Some((other_panel,other_dir)) = pair.1.split_once('=' ) {
                    return Some(State {
                        right: if panel == "right" {dir} else {other_dir}.to_string(),
                        left: if other_panel == "right" {dir} else {other_dir}.to_string(),
                    })
                }
            }
        }
    }
    None
}

fn save_state(state:State) -> io::Result<()> {
    let home = if "windows" == consts::OS {
        env::var("USERPROFILE")
        } else {
            env::var("HOME")
        };
    let Ok(home ) = home else {
        return Ok(())
    };
    let mut home = PathBuf::from(home);
    home.push(".sc");
    let mut state_str = String::new();
    write!(state_str,"left={}\nright={}",state.left,state.right).unwrap();
    fs::write(home, state_str)?;
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
             simweb::json_encode(&cur.file_name().display().to_string()),
             md.is_dir(),
             md.len(),md.modified().unwrap().duration_since(UNIX_EPOCH).unwrap().as_millis()).unwrap();
            }
         res
        }
    ))
}

fn get_file_modified(path: &PathBuf) -> u64 { // in seconds
    match fs::metadata(path) {
        Ok(metadata) => if let Ok(time) = metadata.modified() {time.duration_since(SystemTime::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs()} else {0}
        _ => 0
    }
}
