use std::fs;

use eframe::egui::{self, RichText};

use crate::app::{HelperApp, PendingAction, confirm_pending, maybe_import_bundle_text};
use crate::bundle::{HelperBundle, bundle_path};
use crate::config::{AccentColor, Density, Launcher, ThemeChoice, default_launchers};
use crate::launch::split_args;
use crate::theme;

impl HelperApp {
    pub fn ui_settings(&mut self, ui: &mut egui::Ui) {
        self.ui_pending_bar(ui);

        egui::ScrollArea::vertical().show(ui, |ui| {
            self.ui_appearance_card(ui);
            ui.add_space(12.0);
            self.ui_data_card(ui);
            ui.add_space(12.0);
            self.ui_launchers_card(ui);
        });
    }

    fn ui_pending_bar(&mut self, ui: &mut egui::Ui) {
        let p = self.palette;
        let Some(pending) = &self.pending else {
            return;
        };
        let message = match pending {
            PendingAction::ImportBundle(_) => {
                "Import will replace workspace and settings from this JSON file."
            }
            PendingAction::Reload => "Reload from disk and discard in-memory edits?",
        };
        theme::card(&p).show(ui, |ui| {
            ui.colored_label(p.warn, message);
            ui.add_space(6.0);
            ui.horizontal(|ui| {
                if theme::primary_button(ui, &p, "Confirm").clicked() {
                    confirm_pending(self);
                }
                if theme::ghost_button(ui, &p, "Cancel").clicked() {
                    self.pending = None;
                    self.status = "Cancelled.".into();
                }
            });
        });
        ui.add_space(12.0);
    }

    fn ui_appearance_card(&mut self, ui: &mut egui::Ui) {
        let p = self.palette;
        let mut changed = false;
        theme::card(&p).show(ui, |ui| {
            ui.label(RichText::new("Appearance").heading().color(p.text));
            ui.label(theme::muted(
                &p,
                "Theme, accent, density, and type size. Applied when you change them — not every frame.",
            ));
            ui.add_space(10.0);

            ui.label(theme::muted(&p, "Theme"));
            ui.horizontal(|ui| {
                for (choice, label) in [
                    (ThemeChoice::Dark, "Dark"),
                    (ThemeChoice::Light, "Light"),
                    (ThemeChoice::System, "System"),
                ] {
                    if ui
                        .selectable_label(self.config.appearance.theme == choice, label)
                        .clicked()
                    {
                        self.config.appearance.theme = choice;
                        changed = true;
                    }
                }
            });

            ui.add_space(8.0);
            ui.horizontal(|ui| {
                ui.label(theme::muted(&p, "Accent"));
                ui.label(
                    RichText::new(self.config.appearance.accent.to_string())
                        .small()
                        .monospace()
                        .color(p.muted),
                );
            });
            ui.add_space(4.0);
            ui.horizontal(|ui| {
                for swatch in AccentColor::SWATCHES {
                    let selected = self.config.appearance.accent == swatch;
                    if theme::color_chip(ui, &p, swatch.to_color32(), selected).clicked() {
                        self.config.appearance.accent = swatch;
                        changed = true;
                    }
                }
            });

            ui.add_space(8.0);
            ui.label(theme::muted(&p, "Density"));
            ui.horizontal(|ui| {
                for (density, label) in [
                    (Density::Comfortable, "Comfortable"),
                    (Density::Compact, "Compact"),
                ] {
                    if ui
                        .selectable_label(self.config.appearance.density == density, label)
                        .clicked()
                    {
                        self.config.appearance.density = density;
                        changed = true;
                    }
                }
            });

            ui.add_space(8.0);
            ui.label(theme::muted(&p, "Font scale"));
            let mut scale = self.config.appearance.font_scale;
            if ui
                .add(egui::Slider::new(&mut scale, 0.9..=1.2).suffix("×"))
                .changed()
            {
                self.config.appearance.font_scale = scale;
                self.config.appearance.clamp();
                changed = true;
            }
        });
        if changed {
            self.invalidate_style();
            self.persist_config();
        }
    }

    fn ui_data_card(&mut self, ui: &mut egui::Ui) {
        let p = self.palette;
        theme::card(&p).show(ui, |ui| {
            ui.label(RichText::new("Data").heading().color(p.text));
            ui.label(theme::muted(
                &p,
                "Workspace and settings share one JSON file. Import parses first, then replaces.",
            ));
            ui.add_space(6.0);
            ui.label(RichText::new(bundle_path(&self.app_dir).display().to_string()).monospace());
            ui.add_space(8.0);
            ui.horizontal_wrapped(|ui| {
                if theme::ghost_button(ui, &p, "Reload from disk").clicked() {
                    self.pending = Some(PendingAction::Reload);
                }
                if theme::ghost_button(ui, &p, "Export JSON").clicked()
                    && let Some(path) = rfd::FileDialog::new()
                        .set_file_name("helper.json")
                        .save_file()
                {
                    let bundle = HelperBundle::from_parts(self.store.clone(), self.config.clone());
                    match bundle.save(&path) {
                        Ok(()) => self.status = format!("Exported {}", path.display()),
                        Err(err) => self.status = format!("Export failed: {err}"),
                    }
                }
                if theme::ghost_button(ui, &p, "Import JSON").clicked()
                    && let Some(path) = rfd::FileDialog::new()
                        .add_filter("JSON", &["json"])
                        .pick_file()
                {
                    match fs::read_to_string(&path) {
                        Ok(text) => maybe_import_bundle_text(self, text),
                        Err(err) => self.status = format!("Could not read file: {err}"),
                    }
                }
                if theme::ghost_button(ui, &p, "Open data folder").clicked() {
                    let dir = self.app_dir.clone();
                    if let Some(finder) = self
                        .config
                        .launchers
                        .iter()
                        .find(|l| l.name.eq_ignore_ascii_case("Finder"))
                        .cloned()
                    {
                        self.open_path_with(&finder, &dir);
                    } else {
                        let _ = std::process::Command::new("open").arg(&dir).spawn();
                        self.status = "Opened data folder.".into();
                    }
                }
            });
        });
    }

    fn ui_launchers_card(&mut self, ui: &mut egui::Ui) {
        let p = self.palette;
        let mut remove_at = None;
        let mut set_default = None;
        let mut changed = false;
        let mut reorder = None;

        theme::card(&p).show(ui, |ui| {
            ui.label(RichText::new("Launchers").heading().color(p.text));
            ui.label(theme::muted(
                &p,
                "Open-with targets on the Files tab. Use {path}, {dir}, and {name} in args.",
            ));
            ui.add_space(10.0);

            egui::Grid::new("launchers")
                .num_columns(7)
                .spacing([8.0, 8.0])
                .striped(true)
                .show(ui, |ui| {
                    ui.label(RichText::new("Name").strong().color(p.muted));
                    ui.label(RichText::new("Command").strong().color(p.muted));
                    ui.label(RichText::new("Args").strong().color(p.muted));
                    ui.label(RichText::new("Default").strong().color(p.muted));
                    ui.label("");
                    ui.label("");
                    ui.label("");
                    ui.end_row();

                    for (i, launcher) in self.config.launchers.iter_mut().enumerate() {
                        if ui
                            .add(
                                egui::TextEdit::singleline(&mut launcher.name).desired_width(110.0),
                            )
                            .changed()
                        {
                            changed = true;
                        }
                        if ui
                            .add(
                                egui::TextEdit::singleline(&mut launcher.command)
                                    .desired_width(110.0),
                            )
                            .changed()
                        {
                            changed = true;
                        }
                        let mut args = launcher.args.join(" ");
                        if ui
                            .add(egui::TextEdit::singleline(&mut args).desired_width(220.0))
                            .changed()
                        {
                            launcher.args = split_args(&args);
                            changed = true;
                        }
                        if ui.radio(launcher.is_default, "").clicked() {
                            set_default = Some(i);
                        }
                        if theme::chevron_button(ui, &p, false).clicked() {
                            reorder = Some((i, false));
                        }
                        if theme::chevron_button(ui, &p, true).clicked() {
                            reorder = Some((i, true));
                        }
                        if theme::danger_button(ui, &p, "Remove").clicked() {
                            remove_at = Some(i);
                        }
                        ui.end_row();
                    }
                });

            ui.add_space(10.0);
            ui.horizontal(|ui| {
                if theme::primary_button(ui, &p, "Add launcher").clicked() {
                    self.config.launchers.push(Launcher {
                        name: "New app".into(),
                        command: String::new(),
                        args: vec!["{path}".into()],
                        is_default: false,
                    });
                    changed = true;
                }
                if theme::ghost_button(ui, &p, "Reset to defaults").clicked() {
                    self.config.launchers = default_launchers();
                    changed = true;
                    self.status = "Launchers reset.".into();
                }
            });
        });

        if let Some(i) = remove_at {
            self.config.launchers.remove(i);
            changed = true;
        }
        if let Some(i) = set_default {
            for (idx, launcher) in self.config.launchers.iter_mut().enumerate() {
                launcher.is_default = idx == i;
            }
            changed = true;
        }
        if let Some((i, down)) = reorder {
            self.reorder_launcher(i, down);
        }
        if changed {
            self.persist_config();
        }
    }
}
