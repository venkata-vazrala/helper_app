use std::fs;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use eframe::egui::{self, Align2, Color32, FontId, Pos2, Rect, RichText, Sense, Stroke, Vec2};

use crate::config::{
    AppConfig, Appearance, Density, Launcher, ThemeChoice, config_path, import_config,
};
use crate::launch;
use crate::reorder::{move_down, move_up};
use crate::store::{Store, app_dir, import_store, store_path};
use crate::theme::{self, Palette};

pub const SAVE_DEBOUNCE: Duration = Duration::from_millis(400);

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

pub enum PendingAction {
    ImportStore(Store),
    ImportConfig(AppConfig),
    Reload,
}

pub struct HelperApp {
    pub app_dir: PathBuf,
    pub tab: Tab,
    pub config: AppConfig,
    pub store: Store,
    pub palette: Palette,
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
    pub store_dirty: bool,
    pub last_edit: Option<Instant>,
    pub pending: Option<PendingAction>,
    last_style: Option<(egui::Theme, Appearance)>,
}

impl HelperApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let app_dir = app_dir();
        let mut config = AppConfig::load_or_default(&config_path(&app_dir));
        config.appearance.clamp();
        let store = Store::load_or_default(&store_path(&app_dir));
        let theme = resolve_theme(&cc.egui_ctx, config.appearance.theme);
        let palette = Palette::new(theme, config.appearance.accent);
        theme::apply(&cc.egui_ctx, theme, &config.appearance, &palette);

        let selected_pin = store.pins.first().map(|p| p.id.clone());
        let active_path = store.pins.first().map(|p| p.path.clone());
        let selected_note = store.notes.first().map(|n| n.id.clone());
        let selected_snippet = store.snippets.first().map(|s| s.id.clone());

        let mut app = Self {
            app_dir,
            tab: Tab::Files,
            config,
            store,
            palette,
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
            store_dirty: false,
            last_edit: None,
            pending: None,
            last_style: None,
        };
        app.refresh_listing();
        let _ = app.persist_all();
        app
    }

    pub fn persist_store(&mut self) {
        if let Err(err) = self.store.save(&store_path(&self.app_dir)) {
            self.status = format!("Could not save workspace: {err}");
        } else {
            self.store_dirty = false;
        }
    }

    pub fn persist_config(&mut self) {
        if let Err(err) = self.config.save(&config_path(&self.app_dir)) {
            self.status = format!("Could not save settings: {err}");
        }
    }

    pub fn mark_store_dirty(&mut self) {
        self.store_dirty = true;
        self.last_edit = Some(Instant::now());
    }

    pub fn flush_store(&mut self) {
        if self.store_dirty {
            self.persist_store();
        }
    }

    fn persist_all(&mut self) -> std::io::Result<()> {
        self.config.save(&config_path(&self.app_dir))?;
        self.store.save(&store_path(&self.app_dir))?;
        self.store_dirty = false;
        Ok(())
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

    pub fn reorder_selected(&mut self, down: bool) {
        let changed = match self.tab {
            Tab::Files => {
                let Some(id) = self.selected_pin.clone() else {
                    return;
                };
                let Some(i) = self.store.pins.iter().position(|p| p.id == id) else {
                    return;
                };
                if down {
                    move_down(&mut self.store.pins, i)
                } else {
                    move_up(&mut self.store.pins, i)
                }
            }
            Tab::Notes => {
                let Some(id) = self.selected_note.clone() else {
                    return;
                };
                let Some(i) = self.store.notes.iter().position(|n| n.id == id) else {
                    return;
                };
                if down {
                    move_down(&mut self.store.notes, i)
                } else {
                    move_up(&mut self.store.notes, i)
                }
            }
            Tab::Snippets => {
                let Some(id) = self.selected_snippet.clone() else {
                    return;
                };
                let Some(i) = self.store.snippets.iter().position(|s| s.id == id) else {
                    return;
                };
                if down {
                    move_down(&mut self.store.snippets, i)
                } else {
                    move_up(&mut self.store.snippets, i)
                }
            }
            Tab::Settings => {
                if down {
                    // launchers: no single selected index stored; skip here
                    false
                } else {
                    false
                }
            }
        };
        if changed {
            self.persist_store();
            self.status = "Reordered.".into();
        }
    }

    pub fn reorder_launcher(&mut self, index: usize, down: bool) {
        let changed = if down {
            move_down(&mut self.config.launchers, index)
        } else {
            move_up(&mut self.config.launchers, index)
        };
        if changed {
            self.persist_config();
        }
    }

    pub fn switch_tab(&mut self, tab: Tab) {
        if self.tab != tab {
            self.flush_store();
            self.tab = tab;
        }
    }

    pub fn reload_from_disk(&mut self) {
        self.flush_store();
        self.config = AppConfig::load_or_default(&config_path(&self.app_dir));
        self.config.appearance.clamp();
        self.store = Store::load_or_default(&store_path(&self.app_dir));
        self.selected_pin = self.store.pins.first().map(|p| p.id.clone());
        self.active_path = self.store.pins.first().map(|p| p.path.clone());
        self.selected_note = self.store.notes.first().map(|n| n.id.clone());
        self.selected_snippet = self.store.snippets.first().map(|s| s.id.clone());
        self.refresh_listing();
        self.status = "Reloaded from disk.".into();
        self.invalidate_style();
    }

    pub fn invalidate_style(&mut self) {
        self.last_style = None;
    }

    pub fn apply_imported_store(&mut self, store: Store) {
        self.store = store;
        self.persist_store();
        self.selected_pin = self.store.pins.first().map(|p| p.id.clone());
        self.active_path = self.store.pins.first().map(|p| p.path.clone());
        self.selected_note = self.store.notes.first().map(|n| n.id.clone());
        self.selected_snippet = self.store.snippets.first().map(|s| s.id.clone());
        self.refresh_listing();
        self.status = "Imported workspace.".into();
    }

    pub fn apply_imported_config(&mut self, config: AppConfig) {
        self.config = config;
        self.config.appearance.clamp();
        self.persist_config();
        self.invalidate_style();
        self.status = "Imported settings.".into();
    }

    fn sync_style(&mut self, ctx: &egui::Context) {
        let theme = resolve_theme(ctx, self.config.appearance.theme);
        let appearance = self.config.appearance.clone();
        let key = (theme, appearance.clone());
        if self.last_style.as_ref() != Some(&key) {
            self.palette = Palette::new(theme, appearance.accent);
            theme::apply(ctx, theme, &appearance, &self.palette);
            self.last_style = Some(key);
        }
    }

    fn handle_shortcuts(&mut self, ctx: &egui::Context) {
        let input = ctx.input(|i| {
            (
                i.modifiers.command,
                i.modifiers.alt,
                i.key_pressed(egui::Key::Num1),
                i.key_pressed(egui::Key::Num2),
                i.key_pressed(egui::Key::Num3),
                i.key_pressed(egui::Key::Num4),
                i.key_pressed(egui::Key::ArrowUp),
                i.key_pressed(egui::Key::ArrowDown),
            )
        });
        if input.0 {
            if input.2 {
                self.switch_tab(Tab::Files);
            } else if input.3 {
                self.switch_tab(Tab::Notes);
            } else if input.4 {
                self.switch_tab(Tab::Snippets);
            } else if input.5 {
                self.switch_tab(Tab::Settings);
            }
        }
        if input.1 && !ctx.egui_wants_keyboard_input() {
            if input.6 {
                self.reorder_selected(false);
            } else if input.7 {
                self.reorder_selected(true);
            }
        }
    }
}

impl eframe::App for HelperApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        if ui.ctx().input(|i| i.viewport().close_requested()) {
            self.flush_store();
        }
        if self.store_dirty && self.last_edit.is_some_and(|t| t.elapsed() >= SAVE_DEBOUNCE) {
            self.persist_store();
        }

        self.sync_style(ui.ctx());
        self.handle_shortcuts(ui.ctx());
        let p = self.palette;

        let chrome_h = match self.config.appearance.density {
            Density::Compact => 58.0,
            Density::Comfortable => 68.0,
        };
        egui::Panel::top("chrome")
            .exact_size(chrome_h)
            .show_separator_line(false)
            .show(ui, |ui| {
                theme::card(&p).show(ui, |ui| {
                    ui.horizontal(|ui| {
                        let (mark, _) =
                            ui.allocate_exact_size(Vec2::splat(28.0), egui::Sense::hover());
                        ui.painter().rect_filled(mark, 7.0, p.accent);
                        ui.painter().text(
                            mark.center(),
                            Align2::CENTER_CENTER,
                            "H",
                            FontId::proportional(15.0),
                            p.on_accent,
                        );
                        ui.add_space(6.0);
                        ui.vertical(|ui| {
                            ui.add_space(1.0);
                            ui.label(RichText::new("Helper").size(18.0).strong().color(p.text));
                            ui.label(theme::muted(&p, "Workspace"));
                        });
                        ui.add_space(18.0);
                        for tab in [Tab::Files, Tab::Notes, Tab::Snippets, Tab::Settings] {
                            if tab_chip(ui, &p, self.tab == tab, tab) {
                                self.switch_tab(tab);
                            }
                        }
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
                    ui.painter().circle_filled(dot.center(), 3.5, p.accent);
                    ui.label(RichText::new(&self.status).color(p.muted).size(13.0));
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

fn resolve_theme(ctx: &egui::Context, choice: ThemeChoice) -> egui::Theme {
    match choice {
        ThemeChoice::Dark => egui::Theme::Dark,
        ThemeChoice::Light => egui::Theme::Light,
        ThemeChoice::System => ctx.system_theme().unwrap_or(egui::Theme::Dark),
    }
}

fn tab_chip(ui: &mut egui::Ui, p: &Palette, selected: bool, tab: Tab) -> bool {
    let size = Vec2::new(112.0, 36.0);
    let (rect, response) = ui.allocate_exact_size(size, Sense::click());
    let hovered = response.hovered();
    let fill = if selected {
        p.accent_dim
    } else if hovered {
        p.surface_2
    } else {
        Color32::TRANSPARENT
    };
    let stroke = if selected {
        Stroke::new(1.0, p.accent)
    } else {
        Stroke::new(1.0, Color32::TRANSPARENT)
    };
    ui.painter()
        .rect(rect, 8.0, fill, stroke, egui::StrokeKind::Inside);
    if selected {
        let bar = Rect::from_min_max(
            Pos2::new(rect.left() + 10.0, rect.bottom() - 3.0),
            Pos2::new(rect.right() - 10.0, rect.bottom() - 1.0),
        );
        ui.painter().rect_filled(bar, 1.0, p.accent);
    }
    ui.painter().text(
        Pos2::new(rect.left() + 12.0, rect.center().y),
        Align2::LEFT_CENTER,
        tab.label(),
        FontId::proportional(14.5),
        if selected { p.text } else { p.muted },
    );
    ui.painter().text(
        Pos2::new(rect.right() - 6.0, rect.top() + 5.0),
        Align2::RIGHT_TOP,
        tab.shortcut(),
        FontId::proportional(10.0),
        p.muted,
    );
    response.clicked()
}

pub fn open_with_row(ui: &mut egui::Ui, app: &mut HelperApp) {
    let p = app.palette;
    ui.label(RichText::new("Open with").small().color(p.muted));
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
                if theme::primary_button(ui, &p, &format!("{name}  default")).clicked() {
                    app.open_active_with_index(i);
                }
            } else if theme::ghost_button(ui, &p, &name).clicked() {
                app.open_active_with_index(i);
            }
        }
    });
}

pub fn maybe_import_store_text(app: &mut HelperApp, text: String) {
    match Store::parse_json(&text) {
        Ok(store) => {
            app.pending = Some(PendingAction::ImportStore(store));
            app.status = "Import ready — confirm replace in Settings.".into();
        }
        Err(err) => app.status = format!("Import rejected: {err}"),
    }
}

pub fn maybe_import_config_text(app: &mut HelperApp, text: String) {
    match AppConfig::parse_toml(&text) {
        Ok(cfg) => {
            app.pending = Some(PendingAction::ImportConfig(cfg));
            app.status = "Settings import ready — confirm replace.".into();
        }
        Err(err) => app.status = format!("Import rejected: {err}"),
    }
}

pub fn confirm_pending(app: &mut HelperApp) {
    match app.pending.take() {
        Some(PendingAction::ImportStore(store)) => match serde_json::to_string(&store) {
            Ok(text) => match import_store(&store_path(&app.app_dir), &text) {
                Ok(saved) => app.apply_imported_store(saved),
                Err(err) => app.status = format!("Import failed: {err}"),
            },
            Err(err) => app.status = format!("Import failed: {err}"),
        },
        Some(PendingAction::ImportConfig(cfg)) => match toml::to_string(&cfg) {
            Ok(text) => match import_config(&config_path(&app.app_dir), &text) {
                Ok(saved) => app.apply_imported_config(saved),
                Err(err) => app.status = format!("Import failed: {err}"),
            },
            Err(err) => app.status = format!("Import failed: {err}"),
        },
        Some(PendingAction::Reload) => app.reload_from_disk(),
        None => {}
    }
}
