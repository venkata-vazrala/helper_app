use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::config::{AppConfig, config_path};
use crate::store::{Store, store_path};

/// Reject huge imports so a malicious file cannot balloon memory.
pub const MAX_JSON_BYTES: usize = 8 * 1024 * 1024;

/// One JSON document for workspace data and settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HelperBundle {
    #[serde(default = "default_version")]
    pub version: u32,
    #[serde(default)]
    pub workspace: Store,
    #[serde(default)]
    pub settings: AppConfig,
}

fn default_version() -> u32 {
    1
}

impl Default for HelperBundle {
    fn default() -> Self {
        Self {
            version: 1,
            workspace: Store::default(),
            settings: AppConfig::default(),
        }
    }
}

pub fn bundle_path(app_dir: &Path) -> PathBuf {
    app_dir.join("helper.json")
}

impl HelperBundle {
    pub fn from_parts(workspace: Store, settings: AppConfig) -> Self {
        Self {
            version: 1,
            workspace,
            settings,
        }
    }

    pub fn parse_json(text: &str) -> Result<Self, String> {
        if text.len() > MAX_JSON_BYTES {
            return Err(format!("JSON is larger than {} bytes", MAX_JSON_BYTES));
        }
        let value: serde_json::Value = serde_json::from_str(text).map_err(|e| e.to_string())?;
        if value.get("workspace").is_some() && value.get("settings").is_some() {
            let mut bundle: Self = serde_json::from_value(value).map_err(|e| e.to_string())?;
            if bundle.version == 0 {
                bundle.version = 1;
            }
            bundle.settings.appearance.clamp();
            return Ok(bundle);
        }
        if value.get("pins").is_some() || value.get("notes").is_some() {
            let store: Store = serde_json::from_value(value).map_err(|e| e.to_string())?;
            return Ok(Self::from_parts(store, AppConfig::default()));
        }
        Err("not a Helper JSON file".into())
    }

    /// Load from disk. Never overwrites an existing `helper.json` that failed to parse.
    pub fn load(app_dir: &Path) -> (Self, Option<String>) {
        let path = bundle_path(app_dir);
        if path.exists() {
            if let Ok(meta) = fs::metadata(&path)
                && meta.len() > MAX_JSON_BYTES as u64
            {
                return (
                    Self::default(),
                    Some("helper.json is larger than 8 MiB; left on disk unchanged.".into()),
                );
            }
            return match fs::read_to_string(&path) {
                Ok(text) => match Self::parse_json(&text) {
                    Ok(mut bundle) => {
                        bundle.settings.appearance.clamp();
                        if bundle.settings.ensure_known_launchers() {
                            let _ = bundle.save(&path);
                        }
                        (bundle, None)
                    }
                    Err(err) => (
                        Self::default(),
                        Some(format!(
                            "Could not read helper.json (left on disk unchanged): {err}"
                        )),
                    ),
                },
                Err(err) => (
                    Self::default(),
                    Some(format!("Could not read helper.json: {err}")),
                ),
            };
        }

        let mut migrated = Self {
            version: 1,
            workspace: Store::load_or_default(&store_path(app_dir)),
            settings: AppConfig::load_or_default(&config_path(app_dir)),
        };
        let _ = migrated.settings.ensure_known_launchers();
        let _ = migrated.save(&path);
        (migrated, None)
    }

    pub fn save(&self, path: &Path) -> io::Result<()> {
        save_parts(path, &self.workspace, &self.settings)
    }
}

pub fn save_parts(path: &Path, workspace: &Store, settings: &AppConfig) -> io::Result<()> {
    #[derive(Serialize)]
    struct Parts<'a> {
        version: u32,
        workspace: &'a Store,
        settings: &'a AppConfig,
    }
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let text = serde_json::to_string_pretty(&Parts {
        version: 1,
        workspace,
        settings,
    })
    .map_err(io::Error::other)?;
    crate::atomic::write_atomic(path, text.as_bytes())
}

/// Parse first; only then replace the file.
pub fn import_bundle(dest: &Path, text: &str) -> Result<HelperBundle, String> {
    let bundle = HelperBundle::parse_json(text)?;
    bundle.save(dest).map_err(|e| e.to_string())?;
    Ok(bundle)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::store::Pin;
    use std::path::PathBuf;

    fn empty_store() -> Store {
        Store {
            pins: Vec::new(),
            notes: Vec::new(),
            snippets: Vec::new(),
            recents: Vec::new(),
            show_hidden: false,
        }
    }

    #[test]
    fn roundtrip_includes_workspace_and_settings() {
        let mut workspace = empty_store();
        workspace.pins.push(Pin {
            id: "1".into(),
            path: PathBuf::from("/tmp/x"),
            label: "x".into(),
        });
        let bundle = HelperBundle::from_parts(workspace, AppConfig::default());
        let text = serde_json::to_string(&bundle).unwrap();
        let parsed = HelperBundle::parse_json(&text).unwrap();
        assert_eq!(parsed.workspace.pins[0].label, "x");
        assert!(!parsed.settings.launchers.is_empty());
    }

    #[test]
    fn invalid_import_does_not_clobber() {
        let dir = tempfile::tempdir().unwrap();
        let dest = dir.path().join("helper.json");
        let good = HelperBundle::default();
        good.save(&dest).unwrap();
        let before = fs::read_to_string(&dest).unwrap();
        let err = import_bundle(&dest, "{not json").unwrap_err();
        assert!(!err.is_empty());
        assert_eq!(fs::read_to_string(&dest).unwrap(), before);
    }

    #[test]
    fn migrates_split_store_and_config() {
        let dir = tempfile::tempdir().unwrap();
        let mut store = empty_store();
        store.pins.push(Pin {
            id: "p".into(),
            path: PathBuf::from("/tmp/proj"),
            label: "proj".into(),
        });
        store.save(&store_path(dir.path())).unwrap();
        AppConfig::default().save(&config_path(dir.path())).unwrap();

        let (loaded, warn) = HelperBundle::load(dir.path());
        assert!(warn.is_none());
        assert_eq!(loaded.workspace.pins[0].label, "proj");
        assert!(
            loaded
                .settings
                .launchers
                .iter()
                .any(|l| l.name == "VS Code")
        );
        assert!(bundle_path(dir.path()).exists());
    }

    #[test]
    fn rejects_random_json_object() {
        assert!(HelperBundle::parse_json("{}").is_err());
        assert!(HelperBundle::parse_json(r#"{"oops":true}"#).is_err());
    }

    #[test]
    fn rejects_oversized_json() {
        let huge = "x".repeat(MAX_JSON_BYTES + 1);
        assert!(HelperBundle::parse_json(&huge).is_err());
    }

    #[test]
    fn accepts_legacy_workspace_only_json() {
        let text = r#"{"pins":[{"id":"1","path":"/tmp/a","label":"a"}],"notes":[],"snippets":[]}"#;
        let bundle = HelperBundle::parse_json(text).unwrap();
        assert_eq!(bundle.workspace.pins[0].label, "a");
    }
}
