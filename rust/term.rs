extern crate simterm;
extern crate simweb;
extern crate simcolor;
use std::{
    collections::HashMap,
    path::PathBuf, env::{self,consts},
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
        (cwd.clone(),cwd,HashMap::new(),VERSION)
    }
    fn greeting(&self, version: &str) -> String {
        let ver = version.color_num(196).to_string();
        format!("OS terminal {ver}/{TERM_VERSION}")
    }
}

fn main() {
    let _ = Commander.main_loop();
}