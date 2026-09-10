use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub launchers: Vec<Launcher>,
    #[serde(default)]
    pub appearance: Appearance,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Launcher {
    pub name: String,
    pub command: String,
    pub args: Vec<String>,
    #[serde(default)]
    pub is_default: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum ThemeChoice {
    #[default]
    Dark,
    Light,
    System,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum Accent {
    #[default]
    Teal,
    Blue,
    Amber,
    Rose,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum Density {
    #[default]
    Comfortable,
    Compact,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Appearance {
    #[serde(default)]
    pub theme: ThemeChoice,
    #[serde(default)]
    pub accent: Accent,
    #[serde(default)]
    pub density: Density,
    #[serde(default = "default_font_scale")]
    pub font_scale: f32,
}

impl Default for Appearance {
    fn default() -> Self {
        Self {
            theme: ThemeChoice::Dark,
            accent: Accent::Teal,
            density: Density::Comfortable,
            font_scale: 1.0,
        }
    }
}

impl Appearance {
    pub fn clamp(&mut self) {
        if !self.font_scale.is_finite() {
            self.font_scale = 1.0;
        }
        self.font_scale = self.font_scale.clamp(0.9, 1.2);
    }
}

fn default_font_scale() -> f32 {
    1.0
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            launchers: default_launchers(),
            appearance: Appearance::default(),
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
            Ok(text) => Self::parse_toml(&text).unwrap_or_default(),
            Err(_) => Self::default(),
        }
    }

    pub fn parse_toml(text: &str) -> Result<Self, toml::de::Error> {
        let mut cfg: Self = toml::from_str(text)?;
        cfg.appearance.clamp();
        Ok(cfg)
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

/// Parse first; only then replace the file. Invalid TOML leaves `dest` untouched.
pub fn import_config(dest: &Path, text: &str) -> Result<AppConfig, String> {
    let cfg = AppConfig::parse_toml(text).map_err(|e| e.to_string())?;
    cfg.save(dest).map_err(|e| e.to_string())?;
    Ok(cfg)
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
        assert_eq!(parsed.appearance.theme, ThemeChoice::Dark);
    }

    #[test]
    fn appearance_clamps_font_scale() {
        let mut a = Appearance {
            font_scale: 9.0,
            ..Appearance::default()
        };
        a.clamp();
        assert_eq!(a.font_scale, 1.2);
        a.font_scale = f32::NAN;
        a.clamp();
        assert_eq!(a.font_scale, 1.0);
    }

    #[test]
    fn missing_appearance_section_defaults() {
        let text = r#"
[[launchers]]
name = "Finder"
command = "open"
args = ["{path}"]
"#;
        let cfg = AppConfig::parse_toml(text).unwrap();
        assert_eq!(cfg.appearance.accent, Accent::Teal);
        assert_eq!(cfg.launchers.len(), 1);
    }

    #[test]
    fn invalid_import_does_not_clobber() {
        let dir = tempfile::tempdir().unwrap();
        let dest = dir.path().join("config.toml");
        let good = AppConfig::default();
        good.save(&dest).unwrap();
        let before = fs::read_to_string(&dest).unwrap();
        let err = import_config(&dest, "not = toml [[[").unwrap_err();
        assert!(!err.is_empty());
        assert_eq!(fs::read_to_string(&dest).unwrap(), before);
    }
}
