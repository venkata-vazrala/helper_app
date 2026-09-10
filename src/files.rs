use std::path::PathBuf;

use eframe::egui::{self, Color32, RichText};

use crate::app::{HelperApp, open_with_row};

impl HelperApp {
    pub fn ui_files(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            if ui.button("Add file").clicked()
                && let Some(path) = rfd::FileDialog::new().pick_file()
            {
                let id = self.store.add_pin(path);
                self.select_pin(id);
                self.persist_store();
                self.status = "Pinned file.".into();
            }
            if ui.button("Add folder").clicked()
                && let Some(path) = rfd::FileDialog::new().pick_folder()
            {
                let id = self.store.add_pin(path);
                self.select_pin(id);
                self.persist_store();
                self.status = "Pinned folder.".into();
            }
            if ui.button("Remove pin").clicked()
                && let Some(id) = self.selected_pin.clone()
            {
                self.store.pins.retain(|p| p.id != id);
                self.selected_pin = self.store.pins.first().map(|p| p.id.clone());
                self.active_path = self
                    .selected_pin
                    .as_ref()
                    .and_then(|sid| self.store.pins.iter().find(|p| p.id == *sid))
                    .map(|p| p.path.clone());
                self.refresh_listing();
                self.persist_store();
                self.status = "Removed pin.".into();
            }
            ui.separator();
            let mut show_hidden = self.store.show_hidden;
            if ui.checkbox(&mut show_hidden, "Show hidden").changed() {
                self.store.show_hidden = show_hidden;
                self.refresh_listing();
                self.persist_store();
            }
        });

        ui.add_space(6.0);
        ui.horizontal(|ui| {
            ui.label("Filter");
            ui.add(
                egui::TextEdit::singleline(&mut self.filter)
                    .desired_width(240.0)
                    .hint_text("name or path"),
            );
        });
        ui.add_space(8.0);

        let filter = self.filter.to_lowercase();
        egui::Panel::left("pins")
            .resizable(true)
            .default_size(280.0)
            .show(ui, |ui| {
                ui.label(RichText::new("Workspace").strong());
                ui.add_space(4.0);
                egui::ScrollArea::vertical().show(ui, |ui| {
                    if self.store.pins.is_empty() {
                        ui.label(
                            RichText::new("Pin files and folders you use often.")
                                .weak()
                                .italics(),
                        );
                        return;
                    }

                    let mut clicked: Option<String> = None;
                    let mut open_default = false;
                    for pin in &self.store.pins {
                        let hay = format!("{} {}", pin.label, pin.path.display()).to_lowercase();
                        if !filter.is_empty() && !hay.contains(&filter) {
                            continue;
                        }
                        let selected = self.selected_pin.as_deref() == Some(pin.id.as_str());
                        let icon = if pin.path.is_dir() { "📁" } else { "📄" };
                        let response =
                            ui.selectable_label(selected, format!("{icon}  {}", pin.label));
                        if response.clicked() {
                            clicked = Some(pin.id.clone());
                        }
                        if response.double_clicked() {
                            clicked = Some(pin.id.clone());
                            open_default = true;
                        }
                        ui.label(
                            RichText::new(pin.path.display().to_string())
                                .small()
                                .color(Color32::from_rgb(130, 140, 148)),
                        );
                        ui.add_space(4.0);
                    }
                    if let Some(id) = clicked {
                        self.select_pin(id);
                    }
                    if open_default {
                        self.open_active_default();
                    }
                });
            });

        egui::CentralPanel::default().show(ui, |ui| {
            let Some(path) = self.active_path.clone() else {
                ui.label(RichText::new("Select a pinned item, or add one.").weak());
                return;
            };

            ui.label(RichText::new(path.display().to_string()).monospace());
            if !path.exists() {
                ui.colored_label(Color32::from_rgb(220, 140, 90), "This path is missing.");
            }
            ui.add_space(6.0);
            open_with_row(ui, self);
            ui.add_space(8.0);
            ui.separator();
            ui.add_space(6.0);

            if let Some(err) = &self.listing_error {
                ui.colored_label(Color32::from_rgb(220, 120, 120), err);
                return;
            }

            ui.horizontal(|ui| {
                ui.label(RichText::new("Contents").strong());
                if ui.small_button("Up").clicked()
                    && let Some(path) = self.active_path.clone()
                {
                    let listing_dir = if path.is_dir() {
                        path
                    } else {
                        path.parent().map(|p| p.to_path_buf()).unwrap_or(path)
                    };
                    if let Some(parent) = listing_dir.parent() {
                        self.active_path = Some(parent.to_path_buf());
                        self.refresh_listing();
                    }
                }
                if ui.small_button("Pin selected").clicked()
                    && let Some(active) = self.active_path.clone()
                {
                    let already = self.store.pins.iter().any(|p| p.path == active);
                    if already {
                        self.status = "Already pinned.".into();
                    } else {
                        let id = self.store.add_pin(active);
                        self.select_pin(id);
                        self.persist_store();
                        self.status = "Pinned.".into();
                    }
                }
            });

            ui.add_space(4.0);
            egui::ScrollArea::both()
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    egui::Grid::new("listing")
                        .num_columns(2)
                        .striped(true)
                        .spacing([16.0, 4.0])
                        .show(ui, |ui| {
                            ui.label(RichText::new("Name").strong());
                            ui.label(RichText::new("Kind").strong());
                            ui.end_row();

                            let mut pick: Option<PathBuf> = None;
                            let mut open_default = false;
                            for row in &self.listing {
                                if !filter.is_empty() && !row.name.to_lowercase().contains(&filter)
                                {
                                    continue;
                                }
                                let selected = self.active_path.as_ref() == Some(&row.path);
                                let icon = if row.is_dir { "📁" } else { "📄" };
                                let response =
                                    ui.selectable_label(selected, format!("{icon}  {}", row.name));
                                ui.label(if row.is_dir { "folder" } else { "file" });
                                if response.clicked() {
                                    pick = Some(row.path.clone());
                                }
                                if response.double_clicked() {
                                    pick = Some(row.path.clone());
                                    open_default = true;
                                }
                                ui.end_row();
                            }
                            if let Some(path) = pick {
                                self.active_path = Some(path);
                                self.refresh_listing();
                            }
                            if open_default {
                                self.open_active_default();
                            }
                        });
                });
        });
    }
}
