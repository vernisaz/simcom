extern crate simcolor;
extern crate simterm;
extern crate simweb;
use std::{
    collections::HashMap,
    env::{self, consts},
    error::Error,
    path::{PathBuf, MAIN_SEPARATOR_STR},
    process::Command,
};

use simcolor::Colorized;
use simterm::{Terminal, VERSION as TERM_VERSION};

const VERSION: &str = env!("VERSION");

struct Commander;

impl Terminal for Commander {
    fn init(&self) -> (PathBuf, PathBuf, HashMap<String, String>, &str) {
        unsafe { env::set_var("COLORTERM", "truecolor") } // since terminal can be invoked from a service (not a regular terminal session)
        let web = simweb::WebData::new();
        let os_drive = if "windows" == consts::OS {
            env::var("SystemDrive").unwrap_or_default()
        } else {
            String::new()
        };
        let cwd = match web.param("cwd") {
            Some(cwd) => PathBuf::from(cwd),
            _ => PathBuf::from(format!("{os_drive}{}", MAIN_SEPARATOR_STR)),
        };
        let mut aliases = HashMap::new();
        if let Some(mut home_dir) = env::home_dir() {
            home_dir.push(".beerc.7b");
            if home_dir.is_file() {
                let output = Command::new("rb")
                    .arg("-f")
                    .arg(home_dir.display().to_string())
                    .current_dir(&cwd)
                    .output();
                if let Ok(output) = output {
                    for (key, val) in String::from_utf8_lossy(&output.stdout)
                        .lines()
                        .filter_map(|line| line.split_once('='))
                    {
                        if let Some(alias) = key.strip_prefix("alias ") {
                            #[allow(unused_mut)]
                            let mut alias = alias.to_string();
                            #[cfg(target_os = "windows")]
                            alias.make_ascii_uppercase();
                            if val.len() > 2 {
                                aliases.insert(alias, val[1..val.len() - 1].to_string());
                            }
                        } else {
                            unsafe {
                                env::set_var(key, val);
                            }
                        }
                    }
                }
            }
        }
        (cwd.clone(), cwd, aliases, VERSION)
    }
    fn greeting(&self, version: &str) -> String {
        let ver = version.color_num(196).to_string();
        let (_, _, _, h, m, _, _) = simtime::get_datetime(1970, simtime::local_now_secs());
        let (h, p) = convert_24_to_12(h).unwrap();
        format!(
            "Web terminal [{h}:{m:<02} {}M] v{ver}/{TERM_VERSION}",
            if p { "P" } else { "A" }
        )
    }
}

/// Converts a time string in "HH:MM" 24-hour format to "hh:MM AM/PM" 12-hour format.
///
/// # Arguments
/// * `time_24` - A string slice containing the time in 24-hour format.
///
/// # Returns
/// * `Ok((h_12,P)` - The converted time in 12-hour format.
/// * `Err(String)` - An error message if the input is invalid.
///
/// # Examples
/// ```
/// assert_eq!(convert_24_to_12(0).unwrap().0, 12);
/// assert_eq!(convert_24_to_12(13).unwrap().0, 1);
/// ```
fn convert_24_to_12(hour_24: u32) -> Result<(u32, bool), Box<dyn Error>> {
    // Validate ranges
    if hour_24 > 23 {
        return Err("Hour must be 0-23".into());
    }

    // Convert to 12-hour format
    let hour_12 = match hour_24 % 12 {
        0 => 12,
        h => h,
    };

    Ok((hour_12, hour_24 > 11))
}

fn main() {
    let _ = Commander.main_loop();
}
