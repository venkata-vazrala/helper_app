use std::fs;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use eframe::egui::{self, Align2, Color32, FontId, Pos2, Rect, RichText, Sense, Stroke, Vec2};

use crate::bundle::{HelperBundle, bundle_path, import_bundle};
use crate::config::{AppConfig, Appearance, Density, Launcher, ThemeChoice};
use crate::launch;
use crate::reorder::{move_down, move_up};
use crate::store::{Store, app_dir};
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
    ImportBundle(HelperBundle),
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
        let mut bundle = HelperBundle::load(&app_dir);
        bundle.settings.appearance.clamp();
        let config = bundle.settings;
        let store = bundle.workspace;
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
        self.persist_bundle();
    }

    pub fn persist_config(&mut self) {
        self.persist_bundle();
    }

    fn persist_bundle(&mut self) {
        let bundle = HelperBundle::from_parts(self.store.clone(), self.config.clone());
        if let Err(err) = bundle.save(&bundle_path(&self.app_dir)) {
            self.status = format!("Could not save: {err}");
        } else {
            self.store_dirty = false;
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
        HelperBundle::from_parts(self.store.clone(), self.config.clone())
            .save(&bundle_path(&self.app_dir))?;
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
        let bundle = HelperBundle::load(&self.app_dir);
        self.config = bundle.settings;
        self.config.appearance.clamp();
        self.store = bundle.workspace;
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

    pub fn apply_imported_bundle(&mut self, bundle: HelperBundle) {
        self.store = bundle.workspace;
        self.config = bundle.settings;
        self.config.appearance.clamp();
        self.persist_bundle();
        self.selected_pin = self.store.pins.first().map(|p| p.id.clone());
        self.active_path = self.store.pins.first().map(|p| p.path.clone());
        self.selected_note = self.store.notes.first().map(|n| n.id.clone());
        self.selected_snippet = self.store.snippets.first().map(|s| s.id.clone());
        self.refresh_listing();
        self.invalidate_style();
        self.status = "Imported Helper data.".into();
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
            Density::Compact => 48.0,
            Density::Comfortable => 52.0,
        };
        egui::Panel::top("chrome")
            .exact_size(chrome_h)
            .frame(
                egui::Frame::new()
                    .fill(p.bg)
                    .inner_margin(egui::Margin::symmetric(16, 0)),
            )
            .show_separator_line(false)
            .show(ui, |ui| {
                let mut selected_rect: Option<Rect> = None;
                ui.horizontal_centered(|ui| {
                    let (mark, _) = ui.allocate_exact_size(Vec2::splat(26.0), egui::Sense::hover());
                    ui.painter().rect_filled(mark, 6.0, p.accent);
                    ui.painter().text(
                        mark.center(),
                        Align2::CENTER_CENTER,
                        "H",
                        FontId::proportional(14.0),
                        p.on_accent,
                    );
                    ui.add_space(8.0);
                    ui.label(RichText::new("Helper").size(16.0).strong().color(p.text));
                    ui.add_space(22.0);
                    ui.spacing_mut().item_spacing.x = 2.0;
                    let mut clicked_tab = None;
                    for tab in [Tab::Files, Tab::Notes, Tab::Snippets, Tab::Settings] {
                        let selected = self.tab == tab;
                        let (rect, clicked) = tab_chip(ui, &p, selected, tab);
                        if selected {
                            selected_rect = Some(rect);
                        }
                        if clicked {
                            clicked_tab = Some(tab);
                        }
                    }
                    if let Some(tab) = clicked_tab {
                        self.switch_tab(tab);
                    }
                });
                let bar = ui.max_rect();
                let y = bar.bottom() - 1.0;
                let line = Stroke::new(1.0, p.border);
                if let Some(sel) = selected_rect {
                    if sel.left() > bar.left() {
                        ui.painter().hline(bar.left()..=sel.left() + 1.0, y, line);
                    }
                    if sel.right() < bar.right() {
                        ui.painter().hline(sel.right() - 1.0..=bar.right(), y, line);
                    }
                } else {
                    ui.painter().hline(bar.x_range(), y, line);
                }
            });

        egui::Panel::bottom("status")
            .exact_size(32.0)
            .frame(
                egui::Frame::new()
                    .fill(p.bg)
                    .inner_margin(egui::Margin::symmetric(16, 6)),
            )
            .show_separator_line(false)
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    let (dot, _) = ui.allocate_exact_size(Vec2::splat(8.0), egui::Sense::hover());
                    ui.painter().circle_filled(dot.center(), 3.5, p.accent);
                    ui.label(RichText::new(&self.status).color(p.muted).size(13.0));
                });
            });

        egui::CentralPanel::default()
            .frame(
                egui::Frame::new()
                    .fill(p.surface)
                    .inner_margin(egui::Margin::symmetric(14, 12)),
            )
            .show(ui, |ui| match self.tab {
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

fn tab_chip(ui: &mut egui::Ui, p: &Palette, selected: bool, tab: Tab) -> (Rect, bool) {
    let size = Vec2::new(108.0, 36.0);
    let (id_rect, response) = ui.allocate_exact_size(size, Sense::click());
    // Hang 1px over the chrome/content join so the selected tab is one plane with the window.
    let rect = if selected {
        Rect::from_min_max(id_rect.min, Pos2::new(id_rect.max.x, id_rect.max.y + 1.0))
    } else {
        id_rect
    };
    let hovered = response.hovered();
    let fill = if selected {
        p.surface
    } else if hovered {
        p.surface_2
    } else {
        Color32::TRANSPARENT
    };
    let rounding = egui::CornerRadius {
        nw: 8,
        ne: 8,
        sw: 0,
        se: 0,
    };
    ui.painter()
        .rect(rect, rounding, fill, Stroke::NONE, egui::StrokeKind::Inside);
    if selected {
        let cap = Rect::from_min_max(
            Pos2::new(rect.left() + 12.0, rect.top() + 4.0),
            Pos2::new(rect.right() - 12.0, rect.top() + 6.0),
        );
        ui.painter().rect_filled(cap, 1.0, p.accent);
    }
    ui.painter().text(
        Pos2::new(rect.left() + 12.0, rect.center().y + 1.0),
        Align2::LEFT_CENTER,
        tab.label(),
        FontId::proportional(14.0),
        if selected { p.text } else { p.muted },
    );
    ui.painter().text(
        Pos2::new(rect.right() - 7.0, rect.top() + 6.0),
        Align2::RIGHT_TOP,
        tab.shortcut(),
        FontId::proportional(9.5),
        p.muted,
    );
    (id_rect, response.clicked())
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

pub fn maybe_import_bundle_text(app: &mut HelperApp, text: String) {
    match HelperBundle::parse_json(&text) {
        Ok(bundle) => {
            app.pending = Some(PendingAction::ImportBundle(bundle));
            app.status = "Import ready — confirm replace in Settings.".into();
        }
        Err(err) => app.status = format!("Import rejected: {err}"),
    }
}

pub fn confirm_pending(app: &mut HelperApp) {
    match app.pending.take() {
        Some(PendingAction::ImportBundle(bundle)) => match serde_json::to_string(&bundle) {
            Ok(text) => match import_bundle(&bundle_path(&app.app_dir), &text) {
                Ok(saved) => app.apply_imported_bundle(saved),
                Err(err) => app.status = format!("Import failed: {err}"),
            },
            Err(err) => app.status = format!("Import failed: {err}"),
        },
        Some(PendingAction::Reload) => app.reload_from_disk(),
        None => {}
    }
}
