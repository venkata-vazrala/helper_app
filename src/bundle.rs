use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::config::{AppConfig, config_path};
use crate::store::{Store, store_path};

/// One JSON document for workspace data and settings.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct HelperBundle {
    #[serde(default)]
    pub workspace: Store,
    #[serde(default)]
    pub settings: AppConfig,
}

pub fn bundle_path(app_dir: &Path) -> PathBuf {
    app_dir.join("helper.json")
}

impl HelperBundle {
    pub fn from_parts(workspace: Store, settings: AppConfig) -> Self {
        Self {
            workspace,
            settings,
        }
    }

    pub fn parse_json(text: &str) -> Result<Self, String> {
        if let Ok(mut bundle) = serde_json::from_str::<Self>(text)
            && bundle.looks_like_bundle(text)
        {
            bundle.settings.appearance.clamp();
            return Ok(bundle);
        }
        if let Ok(store) = Store::parse_json(text) {
            return Ok(Self {
                workspace: store,
                settings: AppConfig::default(),
            });
        }
        Err("not a Helper JSON file".into())
    }

    fn looks_like_bundle(&self, text: &str) -> bool {
        text.contains("\"workspace\"") || text.contains("\"settings\"")
    }

    pub fn load(app_dir: &Path) -> Self {
        let path = bundle_path(app_dir);
        if let Ok(text) = fs::read_to_string(&path)
            && let Ok(mut bundle) = Self::parse_json(&text)
        {
            bundle.settings.appearance.clamp();
            return bundle;
        }

        let migrated = Self {
            workspace: Store::load_or_default(&store_path(app_dir)),
            settings: AppConfig::load_or_default(&config_path(app_dir)),
        };
        let _ = migrated.save(&path);
        migrated
    }

    pub fn save(&self, path: &Path) -> io::Result<()> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let text = serde_json::to_string_pretty(self).map_err(io::Error::other)?;
        let tmp = path.with_extension("json.tmp");
        fs::write(&tmp, text)?;
        fs::rename(tmp, path)
    }
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

        let loaded = HelperBundle::load(dir.path());
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
    fn accepts_legacy_workspace_only_json() {
        let text = r#"{"pins":[{"id":"1","path":"/tmp/a","label":"a"}],"notes":[],"snippets":[]}"#;
        let bundle = HelperBundle::parse_json(text).unwrap();
        assert_eq!(bundle.workspace.pins[0].label, "a");
    }
}
