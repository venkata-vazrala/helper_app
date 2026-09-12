use std::fmt;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Deserializer, Serialize, Serializer};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    #[serde(default = "default_launchers")]
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

/// Stored as `#RRGGBB`. Old names (`teal`, `blue`, …) still parse.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AccentColor {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl AccentColor {
    pub const fn rgb(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }

    pub const TEAL: Self = Self::rgb(62, 176, 162);
    pub const BLUE: Self = Self::rgb(64, 140, 220);
    pub const SKY: Self = Self::rgb(56, 189, 248);
    pub const INDIGO: Self = Self::rgb(99, 102, 241);
    pub const VIOLET: Self = Self::rgb(167, 139, 250);
    pub const ROSE: Self = Self::rgb(244, 114, 182);
    pub const RED: Self = Self::rgb(239, 68, 68);
    pub const ORANGE: Self = Self::rgb(249, 115, 22);
    pub const AMBER: Self = Self::rgb(245, 158, 11);
    pub const LIME: Self = Self::rgb(132, 204, 22);
    pub const GREEN: Self = Self::rgb(34, 197, 94);
    pub const MINT: Self = Self::rgb(45, 212, 191);

    pub const SWATCHES: [Self; 12] = [
        Self::TEAL,
        Self::MINT,
        Self::GREEN,
        Self::LIME,
        Self::SKY,
        Self::BLUE,
        Self::INDIGO,
        Self::VIOLET,
        Self::ROSE,
        Self::RED,
        Self::ORANGE,
        Self::AMBER,
    ];

    pub fn to_color32(self) -> eframe::egui::Color32 {
        eframe::egui::Color32::from_rgb(self.r, self.g, self.b)
    }

    pub fn parse(s: &str) -> Option<Self> {
        let s = s.trim();
        match s.to_ascii_lowercase().as_str() {
            "teal" => Some(Self::TEAL),
            "blue" => Some(Self::BLUE),
            "amber" => Some(Self::AMBER),
            "rose" => Some(Self::ROSE),
            "green" => Some(Self::GREEN),
            "mint" => Some(Self::MINT),
            other if other.starts_with('#') && other.len() == 7 => {
                let r = u8::from_str_radix(&other[1..3], 16).ok()?;
                let g = u8::from_str_radix(&other[3..5], 16).ok()?;
                let b = u8::from_str_radix(&other[5..7], 16).ok()?;
                Some(Self::rgb(r, g, b))
            }
            _ => None,
        }
    }
}

impl Default for AccentColor {
    fn default() -> Self {
        Self::TEAL
    }
}

impl fmt::Display for AccentColor {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "#{:02X}{:02X}{:02X}", self.r, self.g, self.b)
    }
}

impl Serialize for AccentColor {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> Deserialize<'de> for AccentColor {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let s = String::deserialize(deserializer)?;
        Self::parse(&s).ok_or_else(|| serde::de::Error::custom(format!("invalid accent {s}")))
    }
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
    pub accent: AccentColor,
    #[serde(default)]
    pub density: Density,
    #[serde(default = "default_font_scale")]
    pub font_scale: f32,
}

impl Default for Appearance {
    fn default() -> Self {
        Self {
            theme: ThemeChoice::Dark,
            accent: AccentColor::TEAL,
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
    let mut launchers = vec![
        system_open_launcher(),
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
        antigravity_launcher(),
        Launcher {
            name: "Grok".into(),
            command: "grok".into(),
            args: vec!["--cwd".into(), "{dir}".into()],
            is_default: false,
        },
    ];
    launchers.extend(terminal_launchers());
    launchers
}

fn system_open_launcher() -> Launcher {
    #[cfg(target_os = "macos")]
    {
        Launcher {
            name: "Finder".into(),
            command: "open".into(),
            args: vec!["{path}".into()],
            is_default: false,
        }
    }
    #[cfg(target_os = "linux")]
    {
        Launcher {
            name: "Files".into(),
            command: "xdg-open".into(),
            args: vec!["{path}".into()],
            is_default: false,
        }
    }
    #[cfg(target_os = "windows")]
    {
        Launcher {
            name: "Explorer".into(),
            command: "explorer".into(),
            args: vec!["{path}".into()],
            is_default: false,
        }
    }
    #[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
    {
        Launcher {
            name: "Open".into(),
            command: "xdg-open".into(),
            args: vec!["{path}".into()],
            is_default: false,
        }
    }
}

fn antigravity_launcher() -> Launcher {
    #[cfg(target_os = "macos")]
    {
        Launcher {
            name: "Antigravity IDE".into(),
            command: "open".into(),
            args: vec!["-a".into(), "Antigravity IDE".into(), "{path}".into()],
            is_default: false,
        }
    }
    #[cfg(not(target_os = "macos"))]
    {
        Launcher {
            name: "Antigravity IDE".into(),
            command: "antigravity-ide".into(),
            args: vec!["{path}".into()],
            is_default: false,
        }
    }
}

fn terminal_launchers() -> Vec<Launcher> {
    #[cfg(target_os = "macos")]
    {
        vec![Launcher {
            name: "Terminal".into(),
            command: "open".into(),
            args: vec!["-a".into(), "Terminal".into(), "{dir}".into()],
            is_default: false,
        }]
    }
    #[cfg(target_os = "linux")]
    {
        vec![Launcher {
            name: "Terminal".into(),
            command: "x-terminal-emulator".into(),
            args: vec!["--working-directory".into(), "{dir}".into()],
            is_default: false,
        }]
    }
    #[cfg(target_os = "windows")]
    {
        Vec::new()
    }
    #[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
    {
        Vec::new()
    }
}

fn is_antigravity(launcher: &Launcher) -> bool {
    launcher.command == "antigravity-ide"
        || launcher.command == "agy-ide"
        || launcher.name.eq_ignore_ascii_case("Antigravity IDE")
        || launcher.name.eq_ignore_ascii_case("Antigravity")
        || launcher
            .args
            .iter()
            .any(|a| a.eq_ignore_ascii_case("Antigravity IDE"))
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
        crate::atomic::write_atomic(path, text.as_bytes())
    }

    pub fn default_launcher(&self) -> Option<&Launcher> {
        self.launchers
            .iter()
            .find(|l| l.is_default)
            .or_else(|| self.launchers.first())
    }

    /// Add built-in launchers that older configs may not have. VS Code stays the default.
    pub fn ensure_known_launchers(&mut self) -> bool {
        let mut changed = false;
        if let Some(existing) = self.launchers.iter_mut().find(|l| is_antigravity(l)) {
            let wanted = antigravity_launcher();
            if existing.command != wanted.command || existing.args != wanted.args {
                existing.command = wanted.command;
                existing.args = wanted.args;
                existing.name = wanted.name;
                existing.is_default = false;
                changed = true;
            } else if existing.is_default {
                existing.is_default = false;
                changed = true;
            }
        } else {
            let insert_at = self
                .launchers
                .iter()
                .position(|l| l.name.eq_ignore_ascii_case("Cursor"))
                .map(|i| i + 1)
                .unwrap_or(self.launchers.len());
            self.launchers.insert(insert_at, antigravity_launcher());
            changed = true;
        }

        if !self.launchers.iter().any(|l| l.is_default)
            && let Some(vs) = self
                .launchers
                .iter_mut()
                .find(|l| l.command == "code" || l.name.eq_ignore_ascii_case("VS Code"))
        {
            vs.is_default = true;
            changed = true;
        }
        changed
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_has_vscode_and_grok() {
        let cfg = AppConfig::default();
        let names: Vec<_> = cfg.launchers.iter().map(|l| l.name.as_str()).collect();
        assert!(names.contains(&"VS Code"));
        assert!(names.contains(&"Grok"));
        assert!(names.contains(&"Antigravity IDE"));
        assert!(
            names.contains(&"Finder") || names.contains(&"Files") || names.contains(&"Explorer")
        );
        let ag = cfg
            .launchers
            .iter()
            .find(|l| l.name == "Antigravity IDE")
            .unwrap();
        #[cfg(target_os = "macos")]
        assert_eq!(ag.command, "open");
        #[cfg(not(target_os = "macos"))]
        assert_eq!(ag.command, "antigravity-ide");
        assert!(!ag.is_default);
        let vs = cfg.launchers.iter().find(|l| l.name == "VS Code").unwrap();
        assert!(vs.is_default);
    }

    #[test]
    fn ensure_known_launchers_inserts_antigravity_once() {
        let mut cfg = AppConfig {
            launchers: vec![Launcher {
                name: "Finder".into(),
                command: "open".into(),
                args: vec!["{path}".into()],
                is_default: false,
            }],
            appearance: Appearance::default(),
        };
        assert!(cfg.ensure_known_launchers());
        assert_eq!(
            cfg.launchers
                .iter()
                .filter(|l| l.name == "Antigravity IDE")
                .count(),
            1
        );
        assert!(
            !cfg.launchers
                .iter()
                .any(|l| l.is_default && l.name == "Antigravity IDE")
        );
        assert!(!cfg.ensure_known_launchers());
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
        assert_eq!(cfg.appearance.accent, AccentColor::TEAL);
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

    #[test]
    fn accent_parses_hex_and_legacy_names() {
        assert_eq!(AccentColor::parse("#3EB0A2"), Some(AccentColor::TEAL));
        assert_eq!(AccentColor::parse("teal"), Some(AccentColor::TEAL));
        assert_eq!(AccentColor::parse("rose"), Some(AccentColor::ROSE));
        assert!(AccentColor::parse("nope").is_none());
        assert_eq!(AccentColor::TEAL.to_string(), "#3EB0A2");
    }
}
