use std::io;
use std::path::Path;
use std::process::{Command, Stdio};

use crate::config::Launcher;

/// Replace `{path}`, `{dir}`, and `{name}` in launcher args.
pub fn substitute(args: &[String], path: &Path) -> Vec<String> {
    let path_str = path.to_string_lossy();
    let name = path
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_default();
    let dir = dir_of(path).to_string_lossy().into_owned();

    args.iter()
        .map(|arg| {
            arg.replace("{path}", &path_str)
                .replace("{dir}", &dir)
                .replace("{name}", &name)
        })
        .collect()
}

pub fn dir_of(path: &Path) -> &Path {
    if path.is_dir() {
        path
    } else {
        path.parent().unwrap_or(path)
    }
}

pub fn open_with(launcher: &Launcher, path: &Path) -> io::Result<()> {
    let args = substitute(&launcher.args, path);
    Command::new(&launcher.command)
        .args(&args)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .map(|_| ())
}

/// Split a launcher-args field, honoring double quotes.
pub fn split_args(input: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut cur = String::new();
    let mut in_quotes = false;
    for c in input.chars() {
        match c {
            '"' => in_quotes = !in_quotes,
            c if c.is_whitespace() && !in_quotes => {
                if !cur.is_empty() {
                    out.push(std::mem::take(&mut cur));
                }
            }
            c => cur.push(c),
        }
    }
    if !cur.is_empty() {
        out.push(cur);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn substitutes_placeholders_for_a_file() {
        let path = PathBuf::from("/tmp/workspace/readme.md");
        let args = vec![
            "{path}".into(),
            "{dir}".into(),
            "{name}".into(),
            "--cwd".into(),
            "{dir}".into(),
        ];
        let out = substitute(&args, &path);
        assert_eq!(out[0], "/tmp/workspace/readme.md");
        assert_eq!(out[1], "/tmp/workspace");
        assert_eq!(out[2], "readme.md");
        assert_eq!(out[3], "--cwd");
        assert_eq!(out[4], "/tmp/workspace");
    }

    #[test]
    fn dir_of_file_is_parent_even_if_missing() {
        let path = Path::new("/does/not/exist/file.rs");
        assert_eq!(dir_of(path), Path::new("/does/not/exist"));
    }

    #[test]
    fn split_args_respects_quotes() {
        let args = split_args(r#"-a "Visual Studio Code" {path}"#);
        assert_eq!(args, vec!["-a", "Visual Studio Code", "{path}"]);
    }
}
