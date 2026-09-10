use std::fs;
use std::path::{Path, PathBuf};

use eframe::egui::{self, Color32, FontId, RichText, Stroke};

use crate::config::{AppConfig, Launcher, config_path};
use crate::launch;
use crate::store::{Store, app_dir, store_path};

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Tab {
    Files,
    Notes,
    Snippets,
    Settings,
}

#[derive(Clone)]
pub struct DirEntryInfo {
    pub path: PathBuf,
    pub name: String,
    pub is_dir: bool,
}

pub struct HelperApp {
    pub app_dir: PathBuf,
    pub tab: Tab,
    pub config: AppConfig,
    pub store: Store,
    pub selected_pin: Option<String>,
    pub active_path: Option<PathBuf>,
    pub selected_note: Option<String>,
    pub selected_snippet: Option<String>,
    pub listing: Vec<DirEntryInfo>,
    pub listing_error: Option<String>,
    pub filter: String,
    pub status: String,
    pub note_filter: String,
    pub snippet_filter: String,
}

impl HelperApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        setup_style(&cc.egui_ctx);

        let app_dir = app_dir();
        let config = AppConfig::load_or_default(&config_path(&app_dir));
        let store = Store::load_or_default(&store_path(&app_dir));
        let selected_pin = store.pins.first().map(|p| p.id.clone());
        let active_path = store.pins.first().map(|p| p.path.clone());
        let selected_note = store.notes.first().map(|n| n.id.clone());
        let selected_snippet = store.snippets.first().map(|s| s.id.clone());

        let mut app = Self {
            app_dir,
            tab: Tab::Files,
            config,
            store,
            selected_pin,
            active_path,
            selected_note,
            selected_snippet,
            listing: Vec::new(),
            listing_error: None,
            filter: String::new(),
            status: "Ready.".into(),
            note_filter: String::new(),
            snippet_filter: String::new(),
        };
        app.refresh_listing();
        let _ = app.persist_all();
        app
    }

    pub fn persist_store(&mut self) {
        if let Err(err) = self.store.save(&store_path(&self.app_dir)) {
            self.status = format!("Could not save workspace: {err}");
        }
    }

    pub fn persist_config(&mut self) {
        if let Err(err) = self.config.save(&config_path(&self.app_dir)) {
            self.status = format!("Could not save settings: {err}");
        }
    }

    fn persist_all(&mut self) -> std::io::Result<()> {
        self.config.save(&config_path(&self.app_dir))?;
        self.store.save(&store_path(&self.app_dir))
    }

    pub fn refresh_listing(&mut self) {
        self.listing.clear();
        self.listing_error = None;
        let Some(path) = self.active_path.clone() else {
            return;
        };
        let dir = if path.is_dir() {
            path
        } else {
            match path.parent() {
                Some(parent) => parent.to_path_buf(),
                None => return,
            }
        };
        match fs::read_dir(&dir) {
            Ok(entries) => {
                let mut rows: Vec<DirEntryInfo> = entries
                    .flatten()
                    .map(|e| {
                        let path = e.path();
                        let is_dir = path.is_dir();
                        let name = e.file_name().to_string_lossy().into_owned();
                        DirEntryInfo { path, name, is_dir }
                    })
                    .filter(|row| self.store.show_hidden || !row.name.starts_with('.'))
                    .collect();
                rows.sort_by(|a, b| {
                    b.is_dir
                        .cmp(&a.is_dir)
                        .then_with(|| a.name.to_lowercase().cmp(&b.name.to_lowercase()))
                });
                self.listing = rows;
            }
            Err(err) => {
                self.listing_error = Some(err.to_string());
            }
        }
    }

    pub fn select_pin(&mut self, id: String) {
        self.selected_pin = Some(id.clone());
        if let Some(pin) = self.store.pins.iter().find(|p| p.id == id) {
            self.active_path = Some(pin.path.clone());
        }
        self.refresh_listing();
    }

    pub fn open_path_with(&mut self, launcher: &Launcher, path: &Path) {
        match launch::open_with(launcher, path) {
            Ok(()) => self.status = format!("Opened in {}", launcher.name),
            Err(err) => self.status = format!("Could not open in {}: {err}", launcher.name),
        }
    }

    pub fn open_active_with_index(&mut self, index: usize) {
        let Some(path) = self.active_path.clone() else {
            self.status = "Nothing selected to open.".into();
            return;
        };
        let Some(launcher) = self.config.launchers.get(index).cloned() else {
            return;
        };
        self.open_path_with(&launcher, &path);
    }

    pub fn open_active_default(&mut self) {
        let Some(path) = self.active_path.clone() else {
            return;
        };
        if let Some(launcher) = self.config.default_launcher().cloned() {
            self.open_path_with(&launcher, &path);
        }
    }

    fn handle_shortcuts(&mut self, ctx: &egui::Context) {
        let input = ctx.input(|i| {
            (
                i.modifiers.command,
                i.key_pressed(egui::Key::Num1),
                i.key_pressed(egui::Key::Num2),
                i.key_pressed(egui::Key::Num3),
                i.key_pressed(egui::Key::Num4),
            )
        });
        if !input.0 {
            return;
        }
        if input.1 {
            self.tab = Tab::Files;
        } else if input.2 {
            self.tab = Tab::Notes;
        } else if input.3 {
            self.tab = Tab::Snippets;
        } else if input.4 {
            self.tab = Tab::Settings;
        }
    }
}

impl eframe::App for HelperApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        self.handle_shortcuts(ui.ctx());

        egui::Panel::top("tabs").show(ui, |ui| {
            ui.add_space(6.0);
            ui.horizontal(|ui| {
                ui.add_space(8.0);
                ui.label(
                    RichText::new("Helper")
                        .font(FontId::proportional(18.0))
                        .color(Color32::from_rgb(232, 236, 239))
                        .strong(),
                );
                ui.add_space(16.0);
                tab_button(ui, &mut self.tab, Tab::Files, "Files");
                tab_button(ui, &mut self.tab, Tab::Notes, "Notes");
                tab_button(ui, &mut self.tab, Tab::Snippets, "Snippets");
                tab_button(ui, &mut self.tab, Tab::Settings, "Settings");
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.add_space(8.0);
                    ui.label(
                        RichText::new("⌘1–4 switch tabs")
                            .small()
                            .color(Color32::from_rgb(140, 150, 160)),
                    );
                });
            });
            ui.add_space(6.0);
        });

        egui::Panel::bottom("status").show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.add_space(8.0);
                ui.label(RichText::new(&self.status).color(Color32::from_rgb(180, 190, 200)));
            });
        });

        egui::CentralPanel::default().show(ui, |ui| match self.tab {
            Tab::Files => self.ui_files(ui),
            Tab::Notes => self.ui_notes(ui),
            Tab::Snippets => self.ui_snippets(ui),
            Tab::Settings => self.ui_settings(ui),
        });
    }
}

fn tab_button(ui: &mut egui::Ui, current: &mut Tab, tab: Tab, label: &str) {
    let selected = *current == tab;
    let fill = if selected {
        Color32::from_rgb(42, 92, 88)
    } else {
        Color32::from_rgb(32, 36, 40)
    };
    let text = if selected {
        Color32::from_rgb(220, 245, 238)
    } else {
        Color32::from_rgb(170, 180, 188)
    };
    let button = egui::Button::new(RichText::new(label).color(text).strong())
        .fill(fill)
        .stroke(Stroke::new(
            1.0,
            if selected {
                Color32::from_rgb(78, 168, 156)
            } else {
                Color32::from_rgb(50, 56, 62)
            },
        ))
        .min_size(egui::vec2(96.0, 28.0));
    if ui.add(button).clicked() {
        *current = tab;
    }
}

fn setup_style(ctx: &egui::Context) {
    let mut visuals = egui::Visuals::dark();
    visuals.panel_fill = Color32::from_rgb(22, 24, 27);
    visuals.window_fill = Color32::from_rgb(28, 31, 35);
    visuals.extreme_bg_color = Color32::from_rgb(18, 20, 22);
    visuals.widgets.inactive.bg_fill = Color32::from_rgb(40, 45, 50);
    visuals.widgets.hovered.bg_fill = Color32::from_rgb(52, 64, 62);
    visuals.widgets.active.bg_fill = Color32::from_rgb(42, 92, 88);
    visuals.selection.bg_fill = Color32::from_rgb(42, 92, 88);
    visuals.hyperlink_color = Color32::from_rgb(110, 198, 186);
    ctx.set_visuals(visuals);
}

pub fn open_with_row(ui: &mut egui::Ui, app: &mut HelperApp) {
    ui.label(RichText::new("Open with").small().weak());
    ui.horizontal_wrapped(|ui| {
        let names: Vec<(usize, String, bool)> = app
            .config
            .launchers
            .iter()
            .enumerate()
            .map(|(i, l)| (i, l.name.clone(), l.is_default))
            .collect();
        for (i, name, is_default) in names {
            let label = if is_default {
                format!("{name} · default")
            } else {
                name
            };
            if ui.button(label).clicked() {
                app.open_active_with_index(i);
            }
        }
    });
}
