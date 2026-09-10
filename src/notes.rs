use eframe::egui::{self, RichText, Sense};

use crate::app::HelperApp;
use crate::store::now_unix;
use crate::theme;

impl HelperApp {
    pub fn ui_notes(&mut self, ui: &mut egui::Ui) {
        let p = self.palette;
        theme::toolbar(&p).show(ui, |ui| {
            ui.horizontal(|ui| {
                if theme::primary_button(ui, &p, "New note").clicked() {
                    let id = self.store.add_note();
                    self.selected_note = Some(id);
                    self.persist_store();
                    self.status = "Note created.".into();
                }
                if theme::danger_button(ui, &p, "Delete note").clicked()
                    && let Some(id) = self.selected_note.clone()
                {
                    self.store.notes.retain(|n| n.id != id);
                    self.selected_note = self.store.notes.first().map(|n| n.id.clone());
                    self.persist_store();
                    self.status = "Note deleted.".into();
                }
                if theme::ghost_button(ui, &p, "▴").clicked() {
                    self.reorder_selected(false);
                }
                if theme::ghost_button(ui, &p, "▾").clicked() {
                    self.reorder_selected(true);
                }
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    theme::search_field(ui, &p, &mut self.note_filter, "Search notes", 240.0);
                });
            });
        });
        ui.add_space(10.0);

        let filter = self.note_filter.to_lowercase();
        egui::Panel::left("notes_list")
            .resizable(true)
            .default_size(280.0)
            .show_separator_line(false)
            .show(ui, |ui| {
                theme::card(&p).show(ui, |ui| {
                    theme::section_label(
                        ui,
                        &p,
                        "Collection",
                        &format!("{} notes", self.store.notes.len()),
                    );
                    ui.add_space(8.0);
                    egui::ScrollArea::vertical().show(ui, |ui| {
                        if self.store.notes.is_empty() {
                            ui.label(theme::muted(&p, "No notes yet."));
                            return;
                        }
                        let mut clicked = None;
                        for note in &self.store.notes {
                            let hay = format!("{} {}", note.title, note.body).to_lowercase();
                            if !filter.is_empty() && !hay.contains(&filter) {
                                continue;
                            }
                            let selected = self.selected_note.as_deref() == Some(note.id.as_str());
                            let preview: String = note
                                .body
                                .lines()
                                .find(|line| !line.trim().is_empty())
                                .unwrap_or("Empty note")
                                .chars()
                                .take(48)
                                .collect();
                            let fill = if selected { p.accent_soft } else { p.surface_2 };
                            let inner = egui::Frame::new()
                                .fill(fill)
                                .stroke(egui::Stroke::new(
                                    1.0,
                                    if selected { p.accent } else { p.border },
                                ))
                                .corner_radius(8)
                                .inner_margin(egui::Margin::symmetric(10, 8))
                                .show(ui, |ui| {
                                    ui.set_width(ui.available_width());
                                    ui.label(RichText::new(&note.title).strong().color(p.text));
                                    ui.label(RichText::new(preview).small().color(p.muted));
                                });
                            if inner.response.interact(Sense::click()).clicked() {
                                clicked = Some(note.id.clone());
                            }
                            ui.add_space(6.0);
                        }
                        if let Some(id) = clicked {
                            self.selected_note = Some(id);
                        }
                    });
                });
            });

        egui::CentralPanel::default().show(ui, |ui| {
            let Some(id) = self.selected_note.clone() else {
                theme::empty_state(
                    ui,
                    &p,
                    "No note open",
                    "Create a note and it stays on this machine.",
                );
                return;
            };
            let Some(index) = self.store.notes.iter().position(|n| n.id == id) else {
                return;
            };

            let mut dirty = false;
            theme::card(&p).show(ui, |ui| {
                let note = &mut self.store.notes[index];
                ui.label(theme::muted(&p, "Title"));
                if ui
                    .add(
                        egui::TextEdit::singleline(&mut note.title)
                            .desired_width(f32::INFINITY)
                            .font(egui::TextStyle::Heading),
                    )
                    .changed()
                {
                    dirty = true;
                }
                ui.add_space(10.0);
                ui.label(theme::muted(&p, "Body"));
                theme::inset(&p).show(ui, |ui| {
                    let editor = egui::TextEdit::multiline(&mut note.body)
                        .desired_width(f32::INFINITY)
                        .desired_rows(22)
                        .font(egui::TextStyle::Monospace);
                    if ui.add(editor).changed() {
                        dirty = true;
                    }
                });
            });
            if dirty {
                self.store.notes[index].updated_unix = now_unix();
                self.mark_store_dirty();
            }
        });
    }
}
