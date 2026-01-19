extern crate simterm;
extern crate simweb;
extern crate simcolor;
use std::{
    collections::HashMap,
    path::PathBuf, env::{self,consts}, process::Command,
};

use simterm::{Terminal, VERSION as TERM_VERSION};
use simcolor::Colorized;

const VERSION: &str = env!("VERSION");

struct Commander;

impl Terminal for Commander {
    fn init(&self) -> (PathBuf, PathBuf, HashMap<String,Vec<String>>,&str) {
        let web = simweb::WebData::new();
        let os_drive =
            if "windows" == consts::OS {
                unsafe { env::set_var("TERM", "xterm-256color") }
                env::var("SystemDrive").unwrap_or_default()
            } else {
                 String::new()
            };
        let cwd = match web.param("cwd") {
            Some(cwd) => PathBuf::from(cwd),
            _ => PathBuf::from(format!("{os_drive}{}", std::path::MAIN_SEPARATOR_STR))
        };
        let mut aliases = HashMap::new();
        if let Some(mut home_dir) = env::home_dir () {
            home_dir.push(".beerc.7b");
            if home_dir.is_file() {
                let output = Command::new("rb")
                    .arg("-f")
                     .arg(home_dir.display().to_string())
                     .current_dir(&cwd)
                     .output();
                if let Ok(output) = output {
                    for line in String::from_utf8_lossy(&output.stdout).lines() {
                        if let Some((key,val)) = line.split_once('=') {
                            if let Some(alias) = key.strip_prefix("alias ") {
                                if val.len() > 2 && let Some(val) = val.strip_prefix('\'') && let Some(val) = val.strip_suffix('\'') {
                                    aliases.insert(alias.to_string(), val.split_whitespace().map(str::to_string).collect());
                                }
                            } else {
                                unsafe { env::set_var(key,val); }
                            }
                        }
                    }
                }
            }
        }
        (cwd.clone(),cwd,aliases,VERSION)
    }
    fn greeting(&self, version: &str) -> String {
        let ver = version.color_num(196).to_string();
        format!("OS terminal {ver}/{TERM_VERSION}")
    }
}

fn main() {
    let _ = Commander.main_loop();
}