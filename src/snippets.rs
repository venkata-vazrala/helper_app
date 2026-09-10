use eframe::egui::{self, Color32, RichText};

use crate::app::HelperApp;

impl HelperApp {
    pub fn ui_snippets(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            if ui.button("New snippet").clicked() {
                let id = self.store.add_snippet();
                self.selected_snippet = Some(id);
                self.persist_store();
                self.status = "Snippet created.".into();
            }
            if ui.button("Delete snippet").clicked()
                && let Some(id) = self.selected_snippet.clone()
            {
                self.store.snippets.retain(|s| s.id != id);
                self.selected_snippet = self.store.snippets.first().map(|s| s.id.clone());
                self.persist_store();
                self.status = "Snippet deleted.".into();
            }
            ui.separator();
            ui.label("Filter");
            ui.add(
                egui::TextEdit::singleline(&mut self.snippet_filter)
                    .desired_width(220.0)
                    .hint_text("title or text"),
            );
        });
        ui.add_space(8.0);

        let filter = self.snippet_filter.to_lowercase();
        egui::Panel::left("snippets_list")
            .resizable(true)
            .default_size(260.0)
            .show(ui, |ui| {
                ui.label(RichText::new("Copy-ready").strong());
                egui::ScrollArea::vertical().show(ui, |ui| {
                    if self.store.snippets.is_empty() {
                        ui.label(RichText::new("No snippets yet.").weak().italics());
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
                        ui.horizontal(|ui| {
                            if ui.selectable_label(selected, &snippet.title).clicked() {
                                clicked = Some(snippet.id.clone());
                            }
                            if ui.small_button("Copy").clicked() {
                                copy_id = Some(snippet.id.clone());
                            }
                        });
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

        egui::CentralPanel::default().show(ui, |ui| {
            let Some(id) = self.selected_snippet.clone() else {
                ui.label(RichText::new("Create a snippet, then copy it in one click.").weak());
                return;
            };
            let Some(index) = self.store.snippets.iter().position(|s| s.id == id) else {
                return;
            };

            let mut dirty = false;
            let mut copy_now = false;
            {
                let snippet = &mut self.store.snippets[index];
                ui.horizontal(|ui| {
                    ui.label("Title");
                    if ui
                        .add(
                            egui::TextEdit::singleline(&mut snippet.title)
                                .desired_width(ui.available_width() - 80.0),
                        )
                        .changed()
                    {
                        dirty = true;
                    }
                    if ui
                        .add(
                            egui::Button::new(RichText::new("Copy").color(Color32::WHITE))
                                .fill(Color32::from_rgb(42, 92, 88)),
                        )
                        .clicked()
                    {
                        copy_now = true;
                    }
                });
                ui.add_space(8.0);
                ui.label("Text");
                let editor = egui::TextEdit::multiline(&mut snippet.body)
                    .desired_width(f32::INFINITY)
                    .desired_rows(22)
                    .font(egui::TextStyle::Monospace);
                if ui.add(editor).changed() {
                    dirty = true;
                }
            }
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
