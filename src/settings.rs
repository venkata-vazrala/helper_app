use eframe::egui::{self, RichText};

use crate::app::HelperApp;
use crate::config::{Launcher, default_launchers};
use crate::launch::split_args;

impl HelperApp {
    pub fn ui_settings(&mut self, ui: &mut egui::Ui) {
        ui.label(RichText::new("Launchers").heading());
        ui.label(
            RichText::new(
                "These apps appear as Open-with targets on the Files tab. \
                 Use {path}, {dir}, and {name} in the args field.",
            )
            .weak(),
        );
        ui.add_space(8.0);

        let mut remove_at = None;
        let mut set_default = None;
        let mut changed = false;

        egui::ScrollArea::vertical()
            .max_height(ui.available_height() - 140.0)
            .show(ui, |ui| {
                egui::Grid::new("launchers")
                    .num_columns(5)
                    .spacing([8.0, 6.0])
                    .striped(true)
                    .show(ui, |ui| {
                        ui.label(RichText::new("Name").strong());
                        ui.label(RichText::new("Command").strong());
                        ui.label(RichText::new("Args").strong());
                        ui.label(RichText::new("Default").strong());
                        ui.label("");
                        ui.end_row();

                        for (i, launcher) in self.config.launchers.iter_mut().enumerate() {
                            if ui
                                .add(
                                    egui::TextEdit::singleline(&mut launcher.name)
                                        .desired_width(120.0),
                                )
                                .changed()
                            {
                                changed = true;
                            }
                            if ui
                                .add(
                                    egui::TextEdit::singleline(&mut launcher.command)
                                        .desired_width(120.0),
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
                            if ui.button("Remove").clicked() {
                                remove_at = Some(i);
                            }
                            ui.end_row();
                        }
                    });
            });

        ui.add_space(8.0);
        ui.horizontal(|ui| {
            if ui.button("Add launcher").clicked() {
                self.config.launchers.push(Launcher {
                    name: "New app".into(),
                    command: String::new(),
                    args: vec!["{path}".into()],
                    is_default: false,
                });
                changed = true;
            }
            if ui.button("Reset to defaults").clicked() {
                self.config.launchers = default_launchers();
                changed = true;
                self.status = "Launchers reset.".into();
            }
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

        ui.add_space(16.0);
        ui.separator();
        ui.add_space(8.0);
        ui.label(RichText::new("Data").heading());
        ui.label(format!("Folder: {}", self.app_dir.display()));
        ui.horizontal(|ui| {
            if ui.button("Open data folder").clicked() {
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
