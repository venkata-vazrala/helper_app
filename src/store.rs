use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Store {
    #[serde(default)]
    pub pins: Vec<Pin>,
    #[serde(default)]
    pub notes: Vec<Note>,
    #[serde(default)]
    pub snippets: Vec<Snippet>,
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
            show_hidden: false,
        }
    }
}

impl Store {
    pub fn load_or_default(path: &Path) -> Self {
        match fs::read_to_string(path) {
            Ok(text) => serde_json::from_str(&text).unwrap_or_default(),
            Err(_) => Self::default(),
        }
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

    #[test]
    fn add_pin_uses_file_name_as_label() {
        let mut store = Store {
            pins: Vec::new(),
            notes: Vec::new(),
            snippets: Vec::new(),
            show_hidden: false,
        };
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
    }
}
