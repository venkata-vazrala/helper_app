use std::env;
use std::path::Path;

use serde::Deserialize;

use crate::config::Launcher;

#[derive(Debug, Clone, Deserialize)]
pub struct Catalog {
    pub apps: Vec<CatalogApp>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CatalogApp {
    pub id: String,
    pub name: String,
    pub platforms: Vec<String>,
    pub command: String,
    pub args: Vec<String>,
    #[serde(default)]
    pub detect: Vec<String>,
}

impl CatalogApp {
    pub fn on_this_platform(&self) -> bool {
        let os = current_os();
        self.platforms.iter().any(|p| p == os)
    }

    pub fn to_launcher(&self) -> Launcher {
        Launcher {
            name: self.name.clone(),
            command: self.command.clone(),
            args: self.args.clone(),
            is_default: false,
        }
    }

    pub fn is_installed(&self) -> bool {
        if self.detect.is_empty() {
            return path_has_command(&self.command);
        }
        self.detect.iter().any(|entry| detect_entry(entry))
    }
}

fn current_os() -> &'static str {
    if cfg!(target_os = "macos") {
        "macos"
    } else if cfg!(target_os = "linux") {
        "linux"
    } else if cfg!(target_os = "windows") {
        "windows"
    } else {
        "other"
    }
}

fn detect_entry(entry: &str) -> bool {
    if let Some(rest) = entry.strip_prefix("~/")
        && let Some(home) = dirs::home_dir()
    {
        return home.join(rest).exists();
    }
    if entry.contains('/') || entry.contains('\\') {
        return Path::new(entry).exists();
    }
    path_has_command(entry)
}

fn path_has_command(name: &str) -> bool {
    let sep = if cfg!(windows) { ';' } else { ':' };
    env::var_os("PATH")
        .and_then(|p| p.into_string().ok())
        .into_iter()
        .flat_map(|p| p.split(sep).map(str::to_string).collect::<Vec<_>>())
        .any(|dir| {
            let base = Path::new(&dir).join(name);
            base.is_file()
                || (cfg!(windows)
                    && (base.with_extension("exe").is_file()
                        || base.with_extension("cmd").is_file()
                        || base.with_extension("bat").is_file()))
        })
}

pub fn load() -> Catalog {
    toml::from_str(include_str!("catalog.toml")).expect("catalog.toml is valid")
}

pub fn for_this_platform() -> Vec<CatalogApp> {
    load()
        .apps
        .into_iter()
        .filter(|a| a.on_this_platform())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::launch::substitute;
    use std::collections::HashSet;
    use std::path::PathBuf;

    #[test]
    fn catalog_ids_are_unique() {
        let cat = load();
        let mut seen = HashSet::new();
        for app in &cat.apps {
            assert!(
                seen.insert(app.id.as_str()),
                "duplicate catalog id {}",
                app.id
            );
            assert!(!app.name.is_empty());
            assert!(!app.command.is_empty());
            assert!(!app.platforms.is_empty());
        }
        assert!(!cat.apps.is_empty());
    }

    #[test]
    fn vscode_uses_dash_dash_before_path() {
        let vs = load().apps.into_iter().find(|a| a.id == "vscode").unwrap();
        let out = substitute(&vs.args, &PathBuf::from("/tmp/project"));
        assert_eq!(out, vec!["--", "/tmp/project"]);
    }

    #[test]
    fn open_a_presets_keep_app_name() {
        let ag = load()
            .apps
            .into_iter()
            .find(|a| a.id == "antigravity")
            .unwrap();
        let out = substitute(&ag.args, &PathBuf::from("/tmp/project"));
        assert_eq!(out[0], "-a");
        assert_eq!(out[1], "Antigravity IDE");
    }
}
