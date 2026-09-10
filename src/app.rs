use std::fs;
use std::path::{Path, PathBuf};

use eframe::egui::{self, Color32, FontId, RichText, Stroke, Vec2};

use crate::config::{AppConfig, Launcher, config_path};
use crate::launch;
use crate::store::{Store, app_dir, store_path};
use crate::theme;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Tab {
    Files,
    Notes,
    Snippets,
    Settings,
}

impl Tab {
    fn label(self) -> &'static str {
        match self {
            Tab::Files => "Files",
            Tab::Notes => "Notes",
            Tab::Snippets => "Snippets",
            Tab::Settings => "Settings",
        }
    }

    fn shortcut(self) -> &'static str {
        match self {
            Tab::Files => "⌘1",
            Tab::Notes => "⌘2",
            Tab::Snippets => "⌘3",
            Tab::Settings => "⌘4",
        }
    }
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
        theme::setup(&cc.egui_ctx);

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

        egui::Panel::top("chrome")
            .exact_size(64.0)
            .show_separator_line(false)
            .show(ui, |ui| {
                theme::card().show(ui, |ui| {
                    ui.horizontal(|ui| {
                        let (mark, _) =
                            ui.allocate_exact_size(Vec2::splat(28.0), egui::Sense::hover());
                        ui.painter().rect_filled(mark, 7.0, theme::ACCENT);
                        ui.painter().text(
                            mark.center(),
                            egui::Align2::CENTER_CENTER,
                            "H",
                            FontId::proportional(15.0),
                            Color32::from_rgb(12, 24, 22),
                        );
                        ui.add_space(6.0);
                        ui.vertical(|ui| {
                            ui.add_space(1.0);
                            ui.label(
                                RichText::new("Helper")
                                    .size(18.0)
                                    .strong()
                                    .color(theme::TEXT),
                            );
                            ui.label(theme::muted("Workspace"));
                        });
                        ui.add_space(18.0);
                        for tab in [Tab::Files, Tab::Notes, Tab::Snippets, Tab::Settings] {
                            tab_button(ui, &mut self.tab, tab);
                        }
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            ui.label(theme::muted("⌘1–4"));
                        });
                    });
                });
            });

        egui::Panel::bottom("status")
            .exact_size(36.0)
            .show_separator_line(false)
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.add_space(6.0);
                    let (dot, _) = ui.allocate_exact_size(Vec2::splat(8.0), egui::Sense::hover());
                    ui.painter().circle_filled(dot.center(), 3.5, theme::ACCENT);
                    ui.label(RichText::new(&self.status).color(theme::MUTED).size(13.0));
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

fn tab_button(ui: &mut egui::Ui, current: &mut Tab, tab: Tab) {
    let selected = *current == tab;
    let fill = if selected {
        theme::ACCENT_DIM
    } else {
        Color32::TRANSPARENT
    };
    let text = if selected { theme::TEXT } else { theme::MUTED };
    let button = egui::Button::new(
        RichText::new(format!("{}  {}", tab.label(), tab.shortcut()))
            .color(text)
            .strong(),
    )
    .fill(fill)
    .stroke(Stroke::new(
        1.0,
        if selected {
            theme::ACCENT
        } else {
            Color32::TRANSPARENT
        },
    ))
    .corner_radius(8)
    .min_size(Vec2::new(108.0, 32.0));
    if ui.add(button).clicked() {
        *current = tab;
    }
}

pub fn open_with_row(ui: &mut egui::Ui, app: &mut HelperApp) {
    ui.label(RichText::new("Open with").small().color(theme::MUTED));
    ui.add_space(4.0);
    ui.horizontal_wrapped(|ui| {
        let names: Vec<(usize, String, bool)> = app
            .config
            .launchers
            .iter()
            .enumerate()
            .map(|(i, l)| (i, l.name.clone(), l.is_default))
            .collect();
        for (i, name, is_default) in names {
            if is_default {
                if theme::primary_button(ui, &format!("{name}  default")).clicked() {
                    app.open_active_with_index(i);
                }
            } else if theme::ghost_button(ui, &name).clicked() {
                app.open_active_with_index(i);
            }
        }
    });
}
