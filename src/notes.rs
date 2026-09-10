use eframe::egui::{self, RichText};

use crate::app::HelperApp;
use crate::store::now_unix;

impl HelperApp {
    pub fn ui_notes(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            if ui.button("New note").clicked() {
                let id = self.store.add_note();
                self.selected_note = Some(id);
                self.persist_store();
                self.status = "Note created.".into();
            }
            if ui.button("Delete note").clicked()
                && let Some(id) = self.selected_note.clone()
            {
                self.store.notes.retain(|n| n.id != id);
                self.selected_note = self.store.notes.first().map(|n| n.id.clone());
                self.persist_store();
                self.status = "Note deleted.".into();
            }
            ui.separator();
            ui.label("Filter");
            ui.add(
                egui::TextEdit::singleline(&mut self.note_filter)
                    .desired_width(220.0)
                    .hint_text("title or text"),
            );
        });
        ui.add_space(8.0);

        let filter = self.note_filter.to_lowercase();
        egui::Panel::left("notes_list")
            .resizable(true)
            .default_size(260.0)
            .show(ui, |ui| {
                ui.label(RichText::new("Collection").strong());
                egui::ScrollArea::vertical().show(ui, |ui| {
                    if self.store.notes.is_empty() {
                        ui.label(RichText::new("No notes yet.").weak().italics());
                        return;
                    }
                    let mut clicked = None;
                    for note in &self.store.notes {
                        let hay = format!("{} {}", note.title, note.body).to_lowercase();
                        if !filter.is_empty() && !hay.contains(&filter) {
                            continue;
                        }
                        let selected = self.selected_note.as_deref() == Some(note.id.as_str());
                        if ui.selectable_label(selected, &note.title).clicked() {
                            clicked = Some(note.id.clone());
                        }
                    }
                    if let Some(id) = clicked {
                        self.selected_note = Some(id);
                    }
                });
            });

        egui::CentralPanel::default().show(ui, |ui| {
            let Some(id) = self.selected_note.clone() else {
                ui.label(RichText::new("Create a note to get started.").weak());
                return;
            };
            let Some(index) = self.store.notes.iter().position(|n| n.id == id) else {
                return;
            };

            let mut dirty = false;
            {
                let note = &mut self.store.notes[index];
                ui.label("Title");
                if ui
                    .add(egui::TextEdit::singleline(&mut note.title).desired_width(f32::INFINITY))
                    .changed()
                {
                    dirty = true;
                }
                ui.add_space(8.0);
                ui.label("Body");
                let editor = egui::TextEdit::multiline(&mut note.body)
                    .desired_width(f32::INFINITY)
                    .desired_rows(24)
                    .font(egui::TextStyle::Monospace);
                if ui.add(editor).changed() {
                    dirty = true;
                }
            }
            if dirty {
                self.store.notes[index].updated_unix = now_unix();
                self.persist_store();
            }
        });
    }
}
