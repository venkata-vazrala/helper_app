use eframe::egui::{self, Color32, RichText, Sense};

use crate::app::HelperApp;
use crate::theme;

impl HelperApp {
    pub fn ui_snippets(&mut self, ui: &mut egui::Ui) {
        theme::toolbar().show(ui, |ui| {
            ui.horizontal(|ui| {
                if theme::primary_button(ui, "New snippet").clicked() {
                    let id = self.store.add_snippet();
                    self.selected_snippet = Some(id);
                    self.persist_store();
                    self.status = "Snippet created.".into();
                }
                if theme::danger_button(ui, "Delete snippet").clicked()
                    && let Some(id) = self.selected_snippet.clone()
                {
                    self.store.snippets.retain(|s| s.id != id);
                    self.selected_snippet = self.store.snippets.first().map(|s| s.id.clone());
                    self.persist_store();
                    self.status = "Snippet deleted.".into();
                }
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    theme::search_field(ui, &mut self.snippet_filter, "Search snippets", 240.0);
                });
            });
        });
        ui.add_space(10.0);

        let filter = self.snippet_filter.to_lowercase();
        egui::Panel::left("snippets_list")
            .resizable(true)
            .default_size(280.0)
            .show_separator_line(false)
            .show(ui, |ui| {
                theme::card().show(ui, |ui| {
                    theme::section_label(
                        ui,
                        "Copy-ready",
                        &format!("{} snippets", self.store.snippets.len()),
                    );
                    ui.add_space(8.0);
                    egui::ScrollArea::vertical().show(ui, |ui| {
                        if self.store.snippets.is_empty() {
                            ui.label(theme::muted("No snippets yet."));
                            return;
                        }
                        let mut clicked = None;
                        let mut copy_id = None;
                        for snippet in &self.store.snippets {
                            let hay = format!("{} {}", snippet.title, snippet.body).to_lowercase();
                            if !filter.is_empty() && !hay.contains(&filter) {
                                continue;
                            }
                            let selected =
                                self.selected_snippet.as_deref() == Some(snippet.id.as_str());
                            let inner = egui::Frame::new()
                                .fill(if selected {
                                    theme::ACCENT_SOFT
                                } else {
                                    theme::SURFACE_2
                                })
                                .stroke(egui::Stroke::new(
                                    1.0,
                                    if selected {
                                        theme::ACCENT
                                    } else {
                                        theme::BORDER
                                    },
                                ))
                                .corner_radius(8)
                                .inner_margin(egui::Margin::symmetric(10, 8))
                                .show(ui, |ui| {
                                    ui.set_width(ui.available_width());
                                    ui.horizontal(|ui| {
                                        ui.label(
                                            RichText::new(&snippet.title)
                                                .strong()
                                                .color(theme::TEXT),
                                        );
                                        ui.with_layout(
                                            egui::Layout::right_to_left(egui::Align::Center),
                                            |ui| {
                                                if ui.small_button("Copy").clicked() {
                                                    copy_id = Some(snippet.id.clone());
                                                }
                                            },
                                        );
                                    });
                                });
                            if inner.response.interact(Sense::click()).clicked() {
                                clicked = Some(snippet.id.clone());
                            }
                            ui.add_space(6.0);
                        }
                        if let Some(id) = clicked {
                            self.selected_snippet = Some(id);
                        }
                        if let Some(id) = copy_id
                            && let Some(snippet) = self.store.snippets.iter().find(|s| s.id == id)
                        {
                            ui.ctx().copy_text(snippet.body.clone());
                            self.status = format!("Copied “{}”.", snippet.title);
                        }
                    });
                });
            });

        egui::CentralPanel::default().show(ui, |ui| {
            let Some(id) = self.selected_snippet.clone() else {
                theme::empty_state(
                    ui,
                    "No snippet selected",
                    "Keep paste-ready text here and copy it in one click.",
                );
                return;
            };
            let Some(index) = self.store.snippets.iter().position(|s| s.id == id) else {
                return;
            };

            let mut dirty = false;
            let mut copy_now = false;
            theme::card().show(ui, |ui| {
                let snippet = &mut self.store.snippets[index];
                ui.horizontal(|ui| {
                    ui.vertical(|ui| {
                        ui.label(theme::muted("Title"));
                        if ui
                            .add(
                                egui::TextEdit::singleline(&mut snippet.title)
                                    .desired_width(ui.available_width()),
                            )
                            .changed()
                        {
                            dirty = true;
                        }
                    });
                    ui.add_space(12.0);
                    ui.vertical(|ui| {
                        ui.add_space(18.0);
                        if ui
                            .add(
                                egui::Button::new(
                                    RichText::new("Copy to clipboard")
                                        .color(Color32::from_rgb(12, 24, 22))
                                        .strong(),
                                )
                                .fill(theme::ACCENT)
                                .min_size(egui::vec2(160.0, 34.0)),
                            )
                            .clicked()
                        {
                            copy_now = true;
                        }
                    });
                });
                ui.add_space(10.0);
                ui.label(theme::muted("Text"));
                theme::inset().show(ui, |ui| {
                    let editor = egui::TextEdit::multiline(&mut snippet.body)
                        .desired_width(f32::INFINITY)
                        .desired_rows(20)
                        .font(egui::TextStyle::Monospace);
                    if ui.add(editor).changed() {
                        dirty = true;
                    }
                });
            });
            if copy_now {
                let body = self.store.snippets[index].body.clone();
                let title = self.store.snippets[index].title.clone();
                ui.ctx().copy_text(body);
                self.status = format!("Copied “{title}”.");
            }
            if dirty {
                self.persist_store();
            }
        });
    }
}
