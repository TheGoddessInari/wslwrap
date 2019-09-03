#![deny(clippy::all)]
#![deny(clippy::pedantic)]

#[macro_use]
extern crate lazy_static;
extern crate regex;

#[macro_use]
extern crate log;
extern crate env_logger;

use std::env;
use std::option::Option;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitStatus};

use regex::Regex;

lazy_static! {
    static ref DRIVE_LETTER: Regex = Regex::new(r#"([A-Za-z]):(.*)"#).unwrap();
    static ref PATH_SEP: Regex = Regex::new(r#"(\\)"#).unwrap();
    static ref USERDIR: String =
        env::var("USERPROFILE").unwrap_or_else(|_| env::var("HOME").unwrap());
}

enum UnixPathType {
    None,
    Other,
    Root,
    Home,
}

trait CaptureExt<'t> {
    fn map_to_str(&self, i: usize) -> &str;
}
impl<'t> CaptureExt<'t> for regex::Captures<'t> {
    fn map_to_str(&self, i: usize) -> &str {
        self.get(i).map_or("", |s| s.as_str())
    }
}

fn remove_first(s: &str) -> Option<&str> {
    s.chars().next().map(|c| &s[c.len_utf8()..])
}

#[allow(clippy::needless_return)]
fn is_unix_path(captures: &Option<regex::Captures<'_>>, string: &str) -> UnixPathType {
    debug!("{:?}\n{}", captures, string);
    return match captures {
        Some(_) => UnixPathType::None,
        None => {
            if string.starts_with('/') {
                UnixPathType::Root
            } else if string.starts_with("~/") {
                UnixPathType::Home
            } else if string.find('\\').is_some() {
                UnixPathType::None
            } else {
                UnixPathType::Other
            }
        }
    };
}

fn format_path(string: &str, drive_letter: &str, new_path: &str) -> String {
    string
        .replace(&format!("{}:\\", drive_letter), new_path)
        .replace(&format!("{}:/", drive_letter), new_path)
}

fn convert_path(string: &str) -> String {
    let drive_letter = DRIVE_LETTER.captures(string);
    debug!("{:?}", drive_letter);

    let current_dir_path = env::current_dir().unwrap_or_else(|_| -> PathBuf { PathBuf::from(".") });
    let current_dir = current_dir_path.to_str().unwrap_or("");

    match is_unix_path(&drive_letter, string) {
        UnixPathType::Root => {
            if let Some(cap) = DRIVE_LETTER.captures(current_dir) {
                let new_path = format!("/mnt/{}", cap.map_to_str(1).to_lowercase());
                let path = format!(
                    "{}{}",
                    new_path,
                    format_path(string, cap.map_to_str(1), &new_path)
                );
                debug!("{:?}\n{:?}", new_path, path);
                PATH_SEP.replace_all(&path, r#"/"#).to_string()
            } else {
                let new_path = &format!("{}{}", "/mnt", string);
                PATH_SEP.replace_all(new_path, r#"/"#).to_string()
            }
        }
        UnixPathType::Home => {
            let home_path = USERDIR.to_string();
            if let Some(cap) = DRIVE_LETTER.captures(&home_path) {
                let new_path = format!("/mnt/{}/", cap.map_to_str(1).to_lowercase());
                let old_path = format!("{}{}", home_path, remove_first(string).unwrap_or(""));
                let path = format_path(&old_path, cap.map_to_str(1), &new_path);
                debug!("{:?}\n{:?}\n{:?}", new_path, old_path, path);
                PATH_SEP.replace_all(&path, r#"/"#).to_string()
            } else {
                PATH_SEP.replace_all(string, r#"/"#).to_string()
            }
        }
        UnixPathType::Other => PATH_SEP.replace_all(string, r#"/"#).to_string(),
        UnixPathType::None => {
            if let Some(cap) = drive_letter {
                let new_path = format!("/mnt/{}/", cap.map_to_str(1).to_lowercase());
                let path = format_path(string, cap.map_to_str(1), &new_path);
                debug!("{:?}", cap);
                PATH_SEP.replace_all(&path, r#"/"#).to_string()
            } else if let Some(cap) = DRIVE_LETTER.captures(current_dir) {
                if string.starts_with('\\') {
                    let new_path = format!("/mnt/{}", cap.map_to_str(1).to_lowercase());
                    let path = format!(
                        "{}{}",
                        new_path,
                        format_path(string, cap.map_to_str(1), &new_path)
                    );
                    debug!("{:?}\n{:?}", new_path, path);
                    PATH_SEP.replace_all(&path, r#"/"#).to_string()
                } else {
                    let new_path = format!("/mnt/{}/", cap.map_to_str(1).to_lowercase());
                    let path = format!(
                        "{}/{}",
                        format_path(current_dir, cap.map_to_str(1), &new_path),
                        string
                    );
                    debug!("{:?}\n{:?}", new_path, path);
                    PATH_SEP.replace_all(&path, r#"/"#).to_string()
                }
            } else {
                PATH_SEP.replace_all(string, r#"/"#).to_string()
            }
        }
    }
}

fn escape(strings: &[String]) -> Vec<String> {
    let mut escaped_strings: Vec<String> = Vec::new();
    if let Some((first, rest)) = strings.split_first() {
        escaped_strings.push(String::from(
            Path::new(first).file_stem().unwrap().to_string_lossy(),
        ));
        for string in rest {
            if string.is_empty() {
                continue;
            }
            escaped_strings.push(convert_path(string));
        }
    }
    escaped_strings
}

fn main() -> Result<(), ExitStatus> {
    env_logger::init();
    let args: Vec<String> = env::args().collect();
    let status = Command::new("wsl.exe")
        .args(escape(&args))
        .status()
        .expect("failed to execute WSL");

    std::process::exit(status.code().unwrap_or(0))
}
