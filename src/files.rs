use std::path::PathBuf;

use eframe::egui::{self, RichText, Sense};

use crate::app::{HelperApp, open_with_row};
use crate::theme;

impl HelperApp {
    pub fn ui_files(&mut self, ui: &mut egui::Ui) {
        let p = self.palette;
        theme::toolbar(&p).show(ui, |ui| {
            ui.horizontal(|ui| {
                if theme::primary_button(ui, &p, "Add file").clicked()
                    && let Some(path) = rfd::FileDialog::new().pick_file()
                {
                    let id = self.store.add_pin(path);
                    self.select_pin(id);
                    self.persist_store();
                    self.status = "Pinned file.".into();
                }
                if theme::ghost_button(ui, &p, "Add folder").clicked()
                    && let Some(path) = rfd::FileDialog::new().pick_folder()
                {
                    let id = self.store.add_pin(path);
                    self.select_pin(id);
                    self.persist_store();
                    self.status = "Pinned folder.".into();
                }
                if theme::danger_button(ui, &p, "Unpin").clicked()
                    && let Some(id) = self.selected_pin.clone()
                    && self.store.remove_pin(&id)
                {
                    self.selected_pin = self.store.pins.first().map(|p| p.id.clone());
                    self.active_path = self
                        .selected_pin
                        .as_ref()
                        .and_then(|sid| self.store.pins.iter().find(|p| p.id == *sid))
                        .map(|pin| pin.path.clone());
                    self.refresh_listing();
                    self.persist_store();
                    self.status = "Moved to Recents.".into();
                }
                if theme::chevron_button(ui, &p, false).clicked() {
                    self.reorder_selected(false);
                }
                if theme::chevron_button(ui, &p, true).clicked() {
                    self.reorder_selected(true);
                }
                ui.separator();
                let mut show_hidden = self.store.show_hidden;
                if ui.checkbox(&mut show_hidden, "Show hidden").changed() {
                    self.store.show_hidden = show_hidden;
                    self.refresh_listing();
                    self.persist_store();
                }
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    theme::search_field(ui, &p, &mut self.filter, "Filter name or path", 260.0);
                });
            });
        });
        ui.add_space(10.0);

        let filter = self.filter.to_lowercase();
        egui::Panel::left("pins")
            .resizable(true)
            .default_size(300.0)
            .show_separator_line(false)
            .show(ui, |ui| {
                theme::card(&p).show(ui, |ui| {
                    theme::section_label(
                        ui,
                        &p,
                        "Workspace",
                        &format!("{} pinned", self.store.pins.len()),
                    );
                    ui.add_space(8.0);
                    egui::ScrollArea::vertical()
                        .id_salt("pins_scroll")
                        .max_height(ui.available_height() * 0.58)
                        .show(ui, |ui| {
                            if self.store.pins.is_empty() {
                                ui.label(theme::muted(&p, "Pin files and folders you use often."));
                            }

                            let mut clicked: Option<String> = None;
                            for pin in &self.store.pins {
                                let hay =
                                    format!("{} {}", pin.label, pin.path.display()).to_lowercase();
                                if !filter.is_empty() && !hay.contains(&filter) {
                                    continue;
                                }
                                let selected =
                                    self.selected_pin.as_deref() == Some(pin.id.as_str());
                                if pin_row(
                                    ui,
                                    &p,
                                    selected,
                                    &pin.label,
                                    &pin.path.display().to_string(),
                                ) {
                                    clicked = Some(pin.id.clone());
                                }
                                // double-click via response is inside pin_row — use extra for open
                                ui.add_space(6.0);
                            }
                            if let Some(id) = clicked {
                                self.select_pin(id);
                            }
                        });

                    ui.add_space(12.0);
                    ui.horizontal(|ui| {
                        theme::section_label(
                            ui,
                            &p,
                            "Recents",
                            &format!("{} unpinned", self.store.recents.len()),
                        );
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if theme::ghost_button(ui, &p, "Clear").clicked() {
                                self.store.clear_recents();
                                self.persist_store();
                                self.status = "Recents cleared.".into();
                            }
                        });
                    });
                    ui.add_space(6.0);
                    egui::ScrollArea::vertical()
                        .id_salt("recents_scroll")
                        .show(ui, |ui| {
                            if self.store.recents.is_empty() {
                                ui.label(theme::muted(
                                    &p,
                                    "Unpinned items land here so they are not lost.",
                                ));
                            }
                            let mut restore = None;
                            for recent in &self.store.recents {
                                let missing = !recent.path.exists();
                                ui.horizontal(|ui| {
                                    ui.vertical(|ui| {
                                        ui.label(
                                            RichText::new(&recent.label).strong().color(p.text),
                                        );
                                        let path_color = if missing { p.warn } else { p.muted };
                                        ui.label(
                                            RichText::new(recent.path.display().to_string())
                                                .small()
                                                .color(path_color)
                                                .monospace(),
                                        );
                                    });
                                    ui.with_layout(
                                        egui::Layout::right_to_left(egui::Align::Center),
                                        |ui| {
                                            if theme::ghost_button(ui, &p, "Pin again").clicked() {
                                                restore = Some(recent.id.clone());
                                            }
                                        },
                                    );
                                });
                                ui.add_space(4.0);
                            }
                            if let Some(id) = restore
                                && let Some(pin_id) = self.store.restore_recent(&id)
                            {
                                self.select_pin(pin_id);
                                self.persist_store();
                                self.status = "Pinned from Recents.".into();
                            }
                        });
                });
            });

        egui::CentralPanel::default().show(ui, |ui| {
            let Some(path) = self.active_path.clone() else {
                theme::empty_state(
                    ui,
                    &p,
                    "Nothing selected",
                    "Pin a file or folder to start this workspace.",
                );
                return;
            };

            theme::card(&p).show(ui, |ui| {
                ui.label(
                    RichText::new(path.display().to_string())
                        .monospace()
                        .size(13.0),
                );
                if !path.exists() {
                    ui.colored_label(p.warn, "This path is missing on disk.");
                }
                ui.add_space(8.0);
                open_with_row(ui, self);
            });
            ui.add_space(10.0);

            theme::card(&p).show(ui, |ui| {
                if let Some(err) = &self.listing_error {
                    ui.colored_label(p.danger, err);
                    return;
                }

                ui.horizontal(|ui| {
                    theme::section_label(ui, &p, "Contents", "");
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if theme::ghost_button(ui, &p, "Pin selected").clicked()
                            && let Some(active) = self.active_path.clone()
                        {
                            let id = self.store.add_pin(active);
                            self.select_pin(id);
                            self.persist_store();
                            self.status = "Pinned.".into();
                        }
                        if theme::ghost_button(ui, &p, "Up").clicked()
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
                    });
                });
                ui.add_space(6.0);

                egui::ScrollArea::both()
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        egui::Grid::new("listing")
                            .num_columns(2)
                            .striped(true)
                            .spacing([18.0, 6.0])
                            .show(ui, |ui| {
                                ui.label(RichText::new("Name").strong().color(p.muted));
                                ui.label(RichText::new("Kind").strong().color(p.muted));
                                ui.end_row();

                                let mut pick: Option<PathBuf> = None;
                                let mut open_default = false;
                                for row in &self.listing {
                                    if !filter.is_empty()
                                        && !row.name.to_lowercase().contains(&filter)
                                    {
                                        continue;
                                    }
                                    let selected = self.active_path.as_ref() == Some(&row.path);
                                    let icon = if row.is_dir { "▸" } else { "·" };
                                    let response = ui.selectable_label(
                                        selected,
                                        format!("{icon}  {}", row.name),
                                    );
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
        });
    }
}

fn pin_row(ui: &mut egui::Ui, p: &theme::Palette, selected: bool, label: &str, path: &str) -> bool {
    let fill = if selected { p.accent_soft } else { p.surface_2 };
    let stroke = if selected {
        egui::Stroke::new(1.0, p.accent)
    } else {
        egui::Stroke::new(1.0, p.border)
    };
    let inner = egui::Frame::new()
        .fill(fill)
        .stroke(stroke)
        .corner_radius(8)
        .inner_margin(egui::Margin::symmetric(10, 8))
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.label(RichText::new(label).strong().color(p.text));
            ui.label(RichText::new(path).small().color(p.muted).monospace());
        });
    let response = inner.response.interact(Sense::click());
    if response.double_clicked() {
        // caller treats click as select; double-click select is still useful
    }
    response.clicked() || response.double_clicked()
}
