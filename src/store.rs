use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub const RECENTS_CAP: usize = 50;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Store {
    #[serde(default)]
    pub pins: Vec<Pin>,
    #[serde(default)]
    pub notes: Vec<Note>,
    #[serde(default)]
    pub snippets: Vec<Snippet>,
    #[serde(default)]
    pub recents: Vec<RecentItem>,
    #[serde(default)]
    pub show_hidden: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pin {
    pub id: String,
    pub path: PathBuf,
    pub label: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecentItem {
    pub id: String,
    pub path: PathBuf,
    pub label: String,
    pub unpinned_unix: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Note {
    pub id: String,
    pub title: String,
    pub body: String,
    pub updated_unix: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Snippet {
    pub id: String,
    pub title: String,
    pub body: String,
}

impl Default for Store {
    fn default() -> Self {
        Self {
            pins: Vec::new(),
            notes: vec![Note {
                id: Uuid::new_v4().to_string(),
                title: "Welcome".into(),
                body: "This is your notes tab.\n\nAdd a note, write freely, and it stays on this machine.\n"
                    .into(),
                updated_unix: now_unix(),
            }],
            snippets: vec![
                Snippet {
                    id: Uuid::new_v4().to_string(),
                    title: "TODO".into(),
                    body: "- [ ] ".into(),
                },
                Snippet {
                    id: Uuid::new_v4().to_string(),
                    title: "Rust function".into(),
                    body: "fn name() {\n    \n}\n".into(),
                },
            ],
            recents: Vec::new(),
            show_hidden: false,
        }
    }
}

impl Store {
    pub fn load_or_default(path: &Path) -> Self {
        match fs::read_to_string(path) {
            Ok(text) => Self::parse_json(&text).unwrap_or_default(),
            Err(_) => Self::default(),
        }
    }

    pub fn parse_json(text: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(text)
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

    pub fn add_pin(&mut self, path: PathBuf) -> String {
        self.recents.retain(|r| r.path != path);
        if let Some(existing) = self.pins.iter().find(|p| p.path == path) {
            return existing.id.clone();
        }
        let label = path
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_else(|| path.to_string_lossy().into_owned());
        let id = Uuid::new_v4().to_string();
        self.pins.push(Pin {
            id: id.clone(),
            path,
            label,
        });
        id
    }

    pub fn remove_pin(&mut self, id: &str) -> bool {
        let Some(pos) = self.pins.iter().position(|p| p.id == id) else {
            return false;
        };
        let pin = self.pins.remove(pos);
        self.push_recent(pin);
        true
    }

    fn push_recent(&mut self, pin: Pin) {
        self.recents.retain(|r| r.path != pin.path);
        self.recents.insert(
            0,
            RecentItem {
                id: pin.id,
                path: pin.path,
                label: pin.label,
                unpinned_unix: now_unix(),
            },
        );
        self.recents.truncate(RECENTS_CAP);
    }

    pub fn restore_recent(&mut self, id: &str) -> Option<String> {
        let pos = self.recents.iter().position(|r| r.id == id)?;
        let recent = self.recents.remove(pos);
        Some(self.add_pin(recent.path))
    }

    pub fn clear_recents(&mut self) {
        self.recents.clear();
    }

    pub fn add_note(&mut self) -> String {
        let id = Uuid::new_v4().to_string();
        self.notes.insert(
            0,
            Note {
                id: id.clone(),
                title: "Untitled".into(),
                body: String::new(),
                updated_unix: now_unix(),
            },
        );
        id
    }

    pub fn add_snippet(&mut self) -> String {
        let id = Uuid::new_v4().to_string();
        self.snippets.insert(
            0,
            Snippet {
                id: id.clone(),
                title: "New snippet".into(),
                body: String::new(),
            },
        );
        id
    }
}

/// Parse first; only then replace the file. Invalid JSON leaves `dest` untouched.
pub fn import_store(dest: &Path, text: &str) -> Result<Store, String> {
    let store = Store::parse_json(text).map_err(|e| e.to_string())?;
    store.save(dest).map_err(|e| e.to_string())?;
    Ok(store)
}

pub fn app_dir() -> PathBuf {
    dirs::data_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("helper-app")
}

pub fn store_path(app_dir: &Path) -> PathBuf {
    app_dir.join("store.json")
}

pub fn now_unix() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

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
    fn add_pin_uses_file_name_as_label() {
        let mut store = empty_store();
        let id = store.add_pin(PathBuf::from("/tmp/demo/project"));
        assert_eq!(store.pins.len(), 1);
        assert_eq!(store.pins[0].id, id);
        assert_eq!(store.pins[0].label, "project");
    }

    #[test]
    fn json_roundtrip() {
        let store = Store::default();
        let text = serde_json::to_string(&store).unwrap();
        let parsed: Store = serde_json::from_str(&text).unwrap();
        assert!(!parsed.notes.is_empty());
        assert!(!parsed.snippets.is_empty());
        assert!(parsed.recents.is_empty());
    }

    #[test]
    fn unpin_moves_to_recents() {
        let mut store = empty_store();
        let id = store.add_pin(PathBuf::from("/tmp/demo/a"));
        assert!(store.remove_pin(&id));
        assert!(store.pins.is_empty());
        assert_eq!(store.recents.len(), 1);
        assert_eq!(store.recents[0].path, PathBuf::from("/tmp/demo/a"));
    }

    #[test]
    fn restore_repins_and_drops_recent() {
        let mut store = empty_store();
        let id = store.add_pin(PathBuf::from("/tmp/demo/a"));
        store.remove_pin(&id);
        let recent_id = store.recents[0].id.clone();
        let restored = store.restore_recent(&recent_id).unwrap();
        assert_eq!(store.pins.len(), 1);
        assert_eq!(store.pins[0].id, restored);
        assert!(store.recents.is_empty());
    }

    #[test]
    fn duplicate_path_does_not_double_recent() {
        let mut store = empty_store();
        let id = store.add_pin(PathBuf::from("/tmp/demo/a"));
        store.remove_pin(&id);
        let id = store.add_pin(PathBuf::from("/tmp/demo/a"));
        store.remove_pin(&id);
        assert_eq!(store.recents.len(), 1);
    }

    #[test]
    fn recents_cap_evicts_oldest() {
        let mut store = empty_store();
        for i in 0..(RECENTS_CAP + 5) {
            let id = store.add_pin(PathBuf::from(format!("/tmp/item-{i}")));
            store.remove_pin(&id);
        }
        assert_eq!(store.recents.len(), RECENTS_CAP);
        assert_eq!(
            store.recents[0].path,
            PathBuf::from(format!("/tmp/item-{}", RECENTS_CAP + 4))
        );
    }

    #[test]
    fn invalid_import_does_not_clobber() {
        let dir = tempfile::tempdir().unwrap();
        let dest = dir.path().join("store.json");
        let mut good = empty_store();
        good.add_pin(PathBuf::from("/tmp/keep-me"));
        good.save(&dest).unwrap();
        let before = fs::read_to_string(&dest).unwrap();
        let err = import_store(&dest, "{not json").unwrap_err();
        assert!(!err.is_empty());
        assert_eq!(fs::read_to_string(&dest).unwrap(), before);
    }

    #[test]
    fn valid_import_replaces() {
        let dir = tempfile::tempdir().unwrap();
        let dest = dir.path().join("store.json");
        empty_store().save(&dest).unwrap();
        let incoming =
            r#"{"pins":[{"id":"1","path":"/tmp/x","label":"x"}],"notes":[],"snippets":[]}"#;
        let store = import_store(&dest, incoming).unwrap();
        assert_eq!(store.pins.len(), 1);
        assert_eq!(store.pins[0].label, "x");
        let on_disk: Store = Store::parse_json(&fs::read_to_string(&dest).unwrap()).unwrap();
        assert_eq!(on_disk.pins[0].label, "x");
    }
}
