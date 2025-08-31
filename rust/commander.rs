extern crate simjson;
extern crate simweb;
extern crate simtime;
extern crate simzip;
extern crate exif;
extern crate simcfg;
use std::{io::{self,Read,stdin,Write,ErrorKind}, fmt::Write as FmtWrite, 
    fs::{self,read_dir,}, time::{UNIX_EPOCH,SystemTime}, path::{PathBuf,Path},
    env::consts, env, convert::TryInto,
};

use simjson::{JsonData::{self,Data,Text,Arr,Num,Bool},parse_fragment};
use simweb::{json_encode,html_encode};
use simzip::{ZipEntry,ZipInfo};
use crate::simcfg::get_config_root;

const MAX_BLOCK_LEN : usize = 4*1024*1024;

const VERSION: &str = env!("VERSION");

struct State {
    left: String,
    right: String,
}

fn main() -> io::Result<()> {
    //let web = simweb::WebData::new();
    let mut buffer : Vec<u8> = vec![0u8; MAX_BLOCK_LEN].try_into().unwrap();
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
    match env::var("QUERY_STRING") {
        Ok(value) if value == "restart" => (),
        _ => {
            println!(r#"{{"panel":"control", "system":"{}", "root":"{}", "separator":"{}","left":"{}", "right":"{}"}}"#, consts::OS,
                 os_drive, json_encode(&std::path::MAIN_SEPARATOR_STR),json_encode(&state.left), json_encode(&state.right));
            io::stdout().flush()?;
        },
    }
    loop {
        let Ok(len) = stdin().read(&mut buffer[0..]) else {break};
        // loop until entire payload read
        if len == 0 { break }
        if len == 4 && buffer[0] == 255 && buffer[1] == 255 && buffer[2] == 255 && buffer[3] == 4 {
            break
        }
        let commands = String::from_utf8_lossy(&buffer[..len]);
        let mut chars = commands.chars();
        loop {
            let res = parse_fragment(&mut chars);
            let json = match res.0 {
                Data(json) => json,
                JsonData::None => break,
                _ => {eprintln!("invalid json {:?} of {commands} - {len}", res.0);break},
            };
       // eprintln!("parsed {json:?}");
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
                        if panel == "right" { state.right =  dir.clone() } else { state.left =  dir.clone()}
                        match get_dir(dir) {
                            Ok(dir_contents) => {
                                println!(r#"{{"panel":"{panel}", "dir":[{dir_contents}], "path":"{}"}}"#,
                                json_encode(dir));
                                io::stdout().flush()?;
                            }
                            Err(err) => report(&format!("an error {err:?} in reading {dir}"))?,
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
                                    if src_path.is_dir() {
                                        let _ = copy_directory_contents(&src_path,&dst_path);
                                    } else if src_path.is_file() {
                                        let _ = fs::copy(&src_path,&dst_path);
                                    }
                                    need_copy = false;
                                }
                            }
                        }
                        if need_copy {
                            for file in files {
                                let Text(file) = file else { continue };
                                src_path.push(file.clone());
                                dst_path.push(file);
                                let _ = 
                                if src_path.is_file() {
                                    fs::copy(&src_path,&dst_path)
                                } else if src_path.is_dir() {
                                    copy_directory_contents(&src_path,&dst_path)
                                } else { Ok(0)};
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
                        let mut was_move = false;
                        if let Some(Text(dst_file)) = json.get("file") {
                            if files.len() == 1 {
                            // only rename in the same directory
                                dst_path.push(dst_file);
                                if !dst_path.exists() {
                                    if let Text(file) = &files[0] {
                                        src_path.push(file);
                                        match fs::rename(&src_path,&dst_path) {
                                            Ok(()) => was_move = true,
                                            Err(err) => report(&format!("Can't move {src_path:?} to {dst_path:?}, because {err:?}"))?
                                        }
                                    }
                                }
                            }
                        } else if files.len() > 0 {
                            for file in files {
                                let Text(file) = file else { continue };
                                src_path.push(file.clone());
                                dst_path.push(file);
                                match fs::rename(&src_path,&dst_path) {
                                    Err(err) => {
                                        if err.kind() == ErrorKind:: CrossesDevices {
                                            if src_path.is_file() {
                                                match fs::copy(&src_path,&dst_path) {
                                                    Ok(_) => {let _ = fs::remove_file(&src_path);}
                                                    Err(_) => (),
                                                }
                                            } else if src_path.is_dir() {
                                                match copy_directory_contents(&src_path,&dst_path) {
                                                    Ok(_) => {
                                                        // TODO decide of cases when only some files were copied
                                                        let _ = fs::remove_dir_all(&src_path);
                                                    }
                                                    Err(err) => report(&format!("Can't copy {src_path:?} because {err:?}"))?
                                                }
                                            }
                                        }
                                    }
                                    _ => ()
                                }
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
                            let err;
                            if src_path.is_file() {
                                err =fs::remove_file(&src_path);
                            } else if src_path.is_dir() {
                                err = fs::remove_dir_all(&src_path);
                            } else {
                                err = Ok(())
                            }
                            if let Err(err) = err {
                                report(&format!("Can't delete {src_path:?} because {err:?}"))?
                            }
                            src_path.pop();
                        }
                        if json.get("same") == Some(&Bool(true)) {
                            let dir = get_dir(&src).unwrap(); // TODO add if Ok(dir)
                            println!(r#"{{"panel":"left", "dir":[{}]}}"#, &dir);
                            io::stdout().flush()?;
                            println!(r#"{{"panel":"right", "dir":[{}]}}"#, dir);
                        } else {
                            println!(r#"{{"panel":"{panel}", "dir":[{}]}}"#, get_dir(&src).unwrap());
                        }
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
                            Ok(()) => { 
                                if json.get("same") == Some(&Bool(true)) {
                                    let dir = get_dir(&src).unwrap();
                                    println!(r#"{{"panel":"left", "dir":[{}]}}"#, &dir);
                                    io::stdout().flush()?;
                                    println!(r#"{{"panel":"right", "dir":[{}]}}"#, dir);
                                } else {
                                    println!(r#"{{"panel":"{panel}", "dir":[{}]}}"#, get_dir(&src).unwrap());
                                }
                                io::stdout().flush()?;
                            },
                            Err(err) => report(&format!("Can't make directory {create_path:?} because {err:?}"))?,
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
                            match fs::read_to_string(&show_path) {
                                Ok(file_contents)  => {
                                    println!(r#"{{"panel":"center", "content":"{}"}}"#, json_encode(&html_encode(&file_contents)));
                                }
                                Err(err) => {
                                    println!(r#"{{"panel":"info", "message":"{}"}}"#, json_encode(&format!("The file {show_path:?} can't be shown, because {err}")));
                                }
                            }
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
                            match fs::read_to_string(&edit_path) {
                                Ok(file_contents) => {
                                    let (modified,_) = get_file_modified(&edit_path);
                                    println!(r#"{{"panel":"{panel}", "op":"edit", "file":"{}", "content":"{}", "modified":{modified}}}"#, 
                                        json_encode(&edit_path.display().to_string()), json_encode(&html_encode(&file_contents)));
                                }
                                Err(err) => println!(r#"{{"panel":"info", "message":"{}"}}"#, json_encode(&format!("The file {edit_path:?} can't be edited, because {err}"))),
                            }
                            io::stdout().flush()?;
                        } else if !edit_path.exists() {
                            println!(r#"{{"panel":"{panel}", "op":"edit", "file":"{}", "content":""}}"#, 
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
                            (false, get_file_modified(&save_path).0)
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
                            let (modified,size) = get_file_modified(&save_path);
                            println!(r#"{{"panel":"info", "modified":{modified}, "file":"{}", "size":{size}}}"#,
                                json_encode(&file));
                            io::stdout().flush()?;
                            if new_file {
                                save_path.pop();
                                if json.get("same") == Some(&Bool(true)) {
                                    let dir = get_dir(&save_path.display().to_string()).unwrap();
                                    println!(r#"{{"panel":"left", "dir":[{}]}}"#, &dir);
                                    io::stdout().flush()?;
                                    println!(r#"{{"panel":"right", "dir":[{}]}}"#, dir);
                                } else {
                                    println!(r#"{{"panel":"{panel}", "dir":[{}]}}"#, get_dir(&save_path.display().to_string()).unwrap());
                                }
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
                        let mut zip_file = ZipInfo::new_with_comment(&src_path, &format!("The zip's created using simcommander {VERSION}"));
                        for file in files {
                            let Text(file) = file else { continue };
                            src_path.pop();
                            src_path.push(file);
                            if src_path . is_file() {
                                zip_file.add(ZipEntry::from_file(&src_path, Some("")));
                            } else if src_path . is_dir() {
                                match zip_dir(&mut zip_file, &src_path, Some(file)) {
                                    Ok(()) => (),
                                    Err(err) => {println!(r#"{{"panel":"info", "message":"Can't zip dir {}"}}"#, json_encode(&format!("{err:?}")));
                                        io::stdout().flush()?;
                                        continue
                                    }
                                }
                            }
                        }
                        match zip_file.store() {
                            Ok(()) => {
                                src_path.pop();
                                let dir = get_dir(&src_path.display().to_string()).unwrap();
                                if json.get("same") == Some(&Bool(true)) {
                                    println!(r#"{{"panel":"left", "dir":[{}]}}"#, &dir);
                                    io::stdout().flush()?;
                                    println!(r#"{{"panel":"right", "dir":[{}]}}"#, dir);
                                } else {
                                    println!(r#"{{"panel":"{panel}", "dir":[{}]}}"#, dir);
                                }
                                io::stdout().flush()?;
                            },
                            Err(msg) => {println!(r#"{{"panel":"info", "message":"Can't zip because {}"}}"#, json_encode(&format!("{msg:?}")));
                                io::stdout().flush()?;
                            }
                        }
                    }
                    "search" => {
                    
                        
                    }
                    "info" => {
                        let Some(Text(file)) = json.get("file") else {
                            eprintln!("no file to get info");
                            continue
                        };
                        let Some(Text(dir)) = json.get("src") else {
                            eprintln!("no dir to get info");
                            continue
                        };
                        let obtain_info = || -> io::Result<String> {
                            let mut path = PathBuf::from(&dir);
                            path.push(file);
                            let file = std::fs::File::open(path)?;
                            let mut bufreader = std::io::BufReader::new(&file);
                            let exifreader = exif::Reader::new();
                            let exif = exifreader.read_from_container(&mut bufreader).map_err(|e| io::Error::new(ErrorKind::Other, format!("Exif parsing err: {e:?}")))?;
                            let mut info = String::from("[");
                            
                            for f in exif.fields() {
                                if info.len() > 1 {
                                    info.push(',');
                                }
                                let _ = write!(info, r#"{{"tag":"{}", "id":"{}", "value":"{}"}}"#,
                                              f.tag, f.ifd_num, json_encode(&first_n_chars(&f.display_value().with_unit(&exif).to_string(),120)));
                            }
                            info.push(']');
                            Ok(info)};
                        let Ok(info) = obtain_info() else {
                            continue
                        };
                        println!(r#"{{"panel":"info", "kind":"exif", "details":{}}}"#, info);
                        io::stdout().flush()?;
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
    let Ok(mut config) = get_config_root() else {
        return None
    };
    config.push(".sc");
    if config.exists() {
        let file_contents = fs::read_to_string(config).ok()?; 
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
    let Ok(mut config) = get_config_root() else {
        return Err(io::Error::new(io::ErrorKind::Other, "no config directory".to_string()))
    };
    config.push(".sc");
    let mut state_str = String::new();
    write!(state_str,"left={}\nright={}",state.left,state.right).unwrap();
    fs::write(config, state_str)?;
    Ok(())
}

fn get_dir(dir: &str) -> io::Result<String> {
    let mut init = String::new();
    let path = Path::new(&dir);
    if let Some(_parent_path) = path.parent() {
        //let timestamp = fs::metadata(parent_path)?.modified()?;
        write!(init,r#"{{"name":"..", "dir":true}}"#).unwrap()//,"..",true)//,timestamp.duration_since(UNIX_EPOCH).unwrap().as_millis()).unwrap()
    };
    Ok(read_dir(dir)?.fold(init,
       |mut res,cur| {if let Ok(cur) = cur {
            let md = cur.metadata().unwrap(); 
             write!(res,r#"{}{{"name":"{}", "dir":{}, "size":{}, "timestamp":{}}}"#, if res.is_empty() {""} else {","},
             json_encode(&cur.file_name().display().to_string()),
             md.is_dir(),
             md.len(),md.modified().unwrap().duration_since(UNIX_EPOCH).unwrap().as_millis()).unwrap();
            }
         res
        }
    ))
}

fn get_file_modified(path: &PathBuf) -> (u64,u64) { // in seconds, in bytes
    match fs::metadata(path) {
        Ok(metadata) => (if let Ok(time) = metadata.modified() {time.duration_since(SystemTime::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs()} else {0}, metadata.len()) ,
        _ => (0,0)
    }
}

fn zip_dir (zip: &mut simzip::ZipInfo, dir: &Path, path:Option<&str>) -> io::Result<()> {
    for entry in dir.read_dir()? {
        let entry = entry?; 
        if let Ok(file_type) = entry.file_type() { 
            let name = entry.file_name().to_str().unwrap().to_owned();
            if file_type.is_file() {
                zip.add(simzip::ZipEntry::from_file(&entry.path().as_os_str().to_str().unwrap().to_string(),
                    path.map(str::to_string).as_ref()));
            }  else if file_type.is_dir() {
                let zip_path = match path {
                    None => name,
                    Some(path) => path.to_owned() + "/" + &name
                };
                zip_dir(zip, &entry.path(), Some(&zip_path))?
            }   
        }
    }
    Ok(())
}

fn report(msg: &str) -> io::Result<()> {
    eprintln!("{msg}");
    println!(r#"{{"panel":"info", "message":"{}"}}"#, json_encode(&msg));
    io::stdout().flush()?;
    Ok(())
}

// from AI offerring
fn copy_directory_contents(
    source_dir: &Path,
    destination_dir: &Path,
) -> io::Result<u64> {
    fs::create_dir_all(destination_dir)?; // Create the destination directory if it doesn't exist
    let mut count = 0u64;
    for entry in fs::read_dir(source_dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_file() {
            let file_name = path
                .file_name()
                .ok_or_else(|| {
                    io::Error::new(io::ErrorKind::InvalidInput, "Invalid file name")
                })?;
            let dest_path = destination_dir.join(file_name);
            count += fs::copy(&path, &dest_path)?;
            eprintln!("Copied file: {:?} to {:?}", path, dest_path);
        } else if path.is_dir() {
            let file_name = path
                .file_name()
                .ok_or_else(|| {
                    io::Error::new(io::ErrorKind::InvalidInput, "Invalid file name")
                })?;
            let dest_path = destination_dir.join(file_name);
            match copy_directory_contents(&path, &dest_path) {
                Ok(files) => count += files,
                Err(err) => return Err(err)
            }
        }
    }
    Ok(count)
}

fn first_n_chars(s: &String, n: usize) -> &str {
    match s.char_indices().nth(n) {
        Some((x, _) ) => &s[..x],
        None => s
    }
}
