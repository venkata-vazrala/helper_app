use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub launchers: Vec<Launcher>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Launcher {
    pub name: String,
    pub command: String,
    pub args: Vec<String>,
    #[serde(default)]
    pub is_default: bool,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            launchers: default_launchers(),
        }
    }
}

pub fn default_launchers() -> Vec<Launcher> {
    vec![
        Launcher {
            name: "Finder".into(),
            command: "open".into(),
            args: vec!["{path}".into()],
            is_default: false,
        },
        Launcher {
            name: "VS Code".into(),
            command: "code".into(),
            args: vec!["{path}".into()],
            is_default: true,
        },
        Launcher {
            name: "Cursor".into(),
            command: "cursor".into(),
            args: vec!["{path}".into()],
            is_default: false,
        },
        Launcher {
            name: "Grok".into(),
            command: "grok".into(),
            args: vec!["--cwd".into(), "{dir}".into()],
            is_default: false,
        },
        Launcher {
            name: "Terminal".into(),
            command: "open".into(),
            args: vec!["-a".into(), "Terminal".into(), "{dir}".into()],
            is_default: false,
        },
    ]
}

impl AppConfig {
    pub fn load_or_default(path: &Path) -> Self {
        match fs::read_to_string(path) {
            Ok(text) => toml::from_str(&text).unwrap_or_default(),
            Err(_) => Self::default(),
        }
    }

    pub fn save(&self, path: &Path) -> io::Result<()> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let text = toml::to_string_pretty(self).map_err(io::Error::other)?;
        atomic_write(path, text.as_bytes())
    }

    pub fn default_launcher(&self) -> Option<&Launcher> {
        self.launchers
            .iter()
            .find(|l| l.is_default)
            .or_else(|| self.launchers.first())
    }
}

pub fn config_path(app_dir: &Path) -> PathBuf {
    app_dir.join("config.toml")
}

fn atomic_write(path: &Path, bytes: &[u8]) -> io::Result<()> {
    let tmp = path.with_extension("toml.tmp");
    fs::write(&tmp, bytes)?;
    fs::rename(tmp, path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_has_vscode_and_grok() {
        let cfg = AppConfig::default();
        let names: Vec<_> = cfg.launchers.iter().map(|l| l.name.as_str()).collect();
        assert!(names.contains(&"VS Code"));
        assert!(names.contains(&"Grok"));
        assert!(cfg.launchers.iter().any(|l| l.is_default));
    }

    #[test]
    fn roundtrip_toml() {
        let cfg = AppConfig::default();
        let text = toml::to_string_pretty(&cfg).unwrap();
        let parsed: AppConfig = toml::from_str(&text).unwrap();
        assert_eq!(parsed.launchers.len(), cfg.launchers.len());
        assert_eq!(parsed.launchers[1].args, vec!["{path}"]);
    }
}
