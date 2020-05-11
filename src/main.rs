#![deny(clippy::all)]
#![deny(clippy::pedantic)]
#![warn(rust_2018_idioms)]

use std::env;
use std::option::Option;
use std::path::{Path, PathBuf};
use std::process::Command;
use win32job::Job;

#[derive(Debug, PartialEq)]
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

fn is_unix_path(string: &str) -> UnixPathType {
    match get_drive_letter(string) {
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
    }
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
    let is_unix_path = is_unix_path(string);

    let current_dir_path = env::current_dir().unwrap_or_else(|_| -> PathBuf { PathBuf::from(".") });
    let current_dir = current_dir_path.to_str().unwrap_or("");

    match is_unix_path {
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
            let home_env = home::home_dir().unwrap();
            let home_path = home_env.to_str().unwrap();

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

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let job = Job::create()?;
    let mut info = job.query_extended_limit_info()?;

    info.limit_kill_on_job_close();

    job.set_extended_limit_info(&mut info)?;

    job.assign_current_process()?;
    let args: Vec<String> = env::args().collect();
    let status = Command::new("wsl.exe")
        .args(escape(&args))
        .status()
        .expect("failed to execute WSL");

    if status.success() {
        Ok(())
    } else {
        Err(Box::from(format!(
            "{} returned a status code of {:?}.",
            args[0],
            status.code()
        )))
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn remove_first() {
        use super::remove_first;
        assert_eq!(remove_first("Test").unwrap(), r#"est"#);
    }

    #[test]
    fn backslash_to_slash() {
        use super::backslash_to_slash;
        assert_eq!(backslash_to_slash(r#"\/\/\/\"#), r#"///////"#);
    }
    #[test]
    fn get_drive_letter() {
        use super::get_drive_letter;
        assert_eq!(get_drive_letter("C:\\"), Some('C'));
        assert_eq!(get_drive_letter("C://"), Some('C'));
    }
    #[test]
    fn is_unix_path() {
        use super::is_unix_path;
        use super::UnixPathType;
        assert_eq!(is_unix_path(r#"/mnt/bsd/Downloads"#), UnixPathType::Root);
        assert_eq!(is_unix_path(r#"~/.config/"#), UnixPathType::Home);
        assert_eq!(is_unix_path(r#"e:/Docs"#), UnixPathType::None);
        assert_eq!(is_unix_path(r#"F:/Documents"#), UnixPathType::None);
        assert_eq!(is_unix_path(r#"r:\\Home"#), UnixPathType::None);
        assert_eq!(is_unix_path(r#"a:\\shared"#), UnixPathType::None);
        assert_eq!(is_unix_path(r#".\test/stuff"#), UnixPathType::None);
        assert_eq!(is_unix_path(r#"./"#), UnixPathType::Other);
    }
}
