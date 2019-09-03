#![deny(clippy::all)]
#![deny(clippy::pedantic)]
#![warn(rust_2018_idioms)]

use std::env;
use std::option::Option;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitStatus};

enum UnixPathType {
    None,
    Other,
    Root,
    Home,
}

fn remove_first(s: &str) -> Option<&str> {
    s.chars().next().map(|c| &s[c.len_utf8()..])
}

fn get_drive_letter(drive: &str) -> Option<char> {
    let mut it = drive.chars();
    match (it.next(), it.next()) {
        (Some(c), Some(':')) if c >= 'a' && c <= 'z' || c >= 'A' && c <= 'Z' => Some(c),
        _ => None,
    }
}

#[allow(clippy::needless_return)]
fn is_unix_path(drive_letter: Option<char>, string: &str) -> UnixPathType {
    return match drive_letter {
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

fn backslash_to_slash(string: &str) -> String {
    string.replace('\\', "/")
}

fn format_path(string: &str, drive_letter: char, new_path: &str) -> String {
    string
        .replace(&format!("{}:\\", drive_letter), new_path)
        .replace(&format!("{}:/", drive_letter), new_path)
}

fn convert_path(string: &str) -> String {
    let current_drive = get_drive_letter(string);

    let current_dir_path = env::current_dir().unwrap_or_else(|_| -> PathBuf { PathBuf::from(".") });
    let current_dir = current_dir_path.to_str().unwrap_or("");

    match is_unix_path(current_drive, string) {
        UnixPathType::Root => {
            if let Some(drive_letter) = get_drive_letter(current_dir) {
                let root_path = format!("/mnt/{}", drive_letter.to_lowercase());
                let path = format!(
                    "{}{}",
                    root_path,
                    format_path(string, drive_letter, &root_path)
                );
                backslash_to_slash(&path)
            } else {
                let path = &format!("{}{}", "/mnt", string);
                backslash_to_slash(path)
            }
        }
        UnixPathType::Home => {
            let home_path = env::var("USERPROFILE").unwrap_or_else(|_| env::var("HOME").unwrap());
            if let Some(drive_letter) = get_drive_letter(&home_path) {
                let root_path = format!("/mnt/{}/", drive_letter.to_lowercase());
                let old_path = format!("{}{}", home_path, remove_first(string).unwrap_or(""));
                let path = format_path(&old_path, drive_letter, &root_path);
                backslash_to_slash(&path)
            } else {
                backslash_to_slash(string)
            }
        }
        UnixPathType::Other => backslash_to_slash(string),
        UnixPathType::None => {
            if let Some(drive_letter) = current_drive {
                let root_path = format!("/mnt/{}/", drive_letter.to_lowercase());
                let path = format_path(string, drive_letter, &root_path);
                backslash_to_slash(&path)
            } else if let Some(drive_letter) = get_drive_letter(current_dir) {
                if string.starts_with('\\') {
                    let root_path = format!("/mnt/{}", drive_letter.to_lowercase());
                    let path = format!(
                        "{}{}",
                        root_path,
                        format_path(string, drive_letter, &root_path)
                    );
                    backslash_to_slash(&path)
                } else {
                    let root_path = format!("/mnt/{}/", drive_letter.to_lowercase());
                    let path = format!(
                        "{}/{}",
                        format_path(current_dir, drive_letter, &root_path),
                        string
                    );
                    backslash_to_slash(&path)
                }
            } else {
                backslash_to_slash(string)
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
    let args: Vec<String> = env::args().collect();
    let status = Command::new("wsl.exe")
        .args(escape(&args))
        .status()
        .expect("failed to execute WSL");

    std::process::exit(status.code().unwrap_or(0))
}
