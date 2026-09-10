use eframe::egui::{self, RichText};

use crate::app::HelperApp;
use crate::config::{Launcher, default_launchers};
use crate::launch::split_args;
use crate::theme;

impl HelperApp {
    pub fn ui_settings(&mut self, ui: &mut egui::Ui) {
        let mut remove_at = None;
        let mut set_default = None;
        let mut changed = false;

        theme::card().show(ui, |ui| {
            ui.label(RichText::new("Launchers").heading().color(theme::TEXT));
            ui.label(theme::muted(
                "These apps appear as Open-with targets on the Files tab. Use {path}, {dir}, and {name} in args.",
            ));
            ui.add_space(10.0);

            egui::ScrollArea::vertical()
                .max_height(ui.available_height() - 180.0)
                .show(ui, |ui| {
                    egui::Grid::new("launchers")
                        .num_columns(5)
                        .spacing([10.0, 8.0])
                        .striped(true)
                        .show(ui, |ui| {
                            ui.label(RichText::new("Name").strong().color(theme::MUTED));
                            ui.label(RichText::new("Command").strong().color(theme::MUTED));
                            ui.label(RichText::new("Args").strong().color(theme::MUTED));
                            ui.label(RichText::new("Default").strong().color(theme::MUTED));
                            ui.label("");
                            ui.end_row();

                            for (i, launcher) in self.config.launchers.iter_mut().enumerate() {
                                if ui
                                    .add(
                                        egui::TextEdit::singleline(&mut launcher.name)
                                            .desired_width(130.0),
                                    )
                                    .changed()
                                {
                                    changed = true;
                                }
                                if ui
                                    .add(
                                        egui::TextEdit::singleline(&mut launcher.command)
                                            .desired_width(130.0),
                                    )
                                    .changed()
                                {
                                    changed = true;
                                }
                                let mut args = launcher.args.join(" ");
                                if ui
                                    .add(egui::TextEdit::singleline(&mut args).desired_width(280.0))
                                    .changed()
                                {
                                    launcher.args = split_args(&args);
                                    changed = true;
                                }
                                if ui.radio(launcher.is_default, "").clicked() {
                                    set_default = Some(i);
                                }
                                if theme::danger_button(ui, "Remove").clicked() {
                                    remove_at = Some(i);
                                }
                                ui.end_row();
                            }
                        });
                });

            ui.add_space(10.0);
            ui.horizontal(|ui| {
                if theme::primary_button(ui, "Add launcher").clicked() {
                    self.config.launchers.push(Launcher {
                        name: "New app".into(),
                        command: String::new(),
                        args: vec!["{path}".into()],
                        is_default: false,
                    });
                    changed = true;
                }
                if theme::ghost_button(ui, "Reset to defaults").clicked() {
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
        if changed {
            self.persist_config();
        }

        ui.add_space(12.0);
        theme::card().show(ui, |ui| {
            ui.label(RichText::new("Data").heading().color(theme::TEXT));
            ui.label(theme::muted(
                "Pins, notes, snippets, and launchers live here — not in the git repo.",
            ));
            ui.add_space(6.0);
            ui.label(RichText::new(self.app_dir.display().to_string()).monospace());
            ui.add_space(8.0);
            if theme::ghost_button(ui, "Open data folder").clicked() {
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
    }
}
