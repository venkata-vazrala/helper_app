use eframe::egui::{self, RichText, Sense};

use crate::app::HelperApp;
use crate::theme;

impl HelperApp {
    pub fn ui_snippets(&mut self, ui: &mut egui::Ui) {
        let p = self.palette;
        theme::toolbar(&p).show(ui, |ui| {
            ui.horizontal(|ui| {
                if theme::primary_button(ui, &p, "New snippet").clicked() {
                    let id = self.store.add_snippet();
                    self.selected_snippet = Some(id);
                    self.persist_store();
                    self.status = "Snippet created.".into();
                }
                if theme::danger_button(ui, &p, "Delete snippet").clicked()
                    && let Some(id) = self.selected_snippet.clone()
                {
                    self.store.snippets.retain(|s| s.id != id);
                    self.selected_snippet = self.store.snippets.first().map(|s| s.id.clone());
                    self.persist_store();
                    self.status = "Snippet deleted.".into();
                }
                if theme::chevron_button(ui, &p, false).clicked() {
                    self.reorder_selected(false);
                }
                if theme::chevron_button(ui, &p, true).clicked() {
                    self.reorder_selected(true);
                }
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    theme::search_field(ui, &p, &mut self.snippet_filter, "Search snippets", 240.0);
                });
            });
        });
        ui.add_space(10.0);

        let filter = self.snippet_filter.to_lowercase();
        egui::Panel::left("snippets_list")
            .resizable(true)
            .default_size(300.0)
            .show_separator_line(false)
            .show(ui, |ui| {
                theme::card(&p).show(ui, |ui| {
                    theme::section_label(
                        ui,
                        &p,
                        "Copy-ready",
                        &format!("{} snippets", self.store.snippets.len()),
                    );
                    ui.add_space(8.0);
                    egui::ScrollArea::vertical()
                        .id_salt("snippets_list_scroll")
                        .auto_shrink([false, false])
                        .scroll_bar_visibility(
                            egui::containers::scroll_area::ScrollBarVisibility::AlwaysVisible,
                        )
                        .show(ui, |ui| {
                            if self.store.snippets.is_empty() {
                                ui.label(theme::muted(&p, "No snippets yet."));
                                return;
                            }
                            let mut clicked = None;
                            let mut copy_id = None;
                            for snippet in &self.store.snippets {
                                let hay =
                                    format!("{} {}", snippet.title, snippet.body).to_lowercase();
                                if !filter.is_empty() && !hay.contains(&filter) {
                                    continue;
                                }
                                let selected =
                                    self.selected_snippet.as_deref() == Some(snippet.id.as_str());
                                let preview: String = snippet
                                    .body
                                    .lines()
                                    .find(|line| !line.trim().is_empty())
                                    .unwrap_or("Empty snippet")
                                    .chars()
                                    .take(52)
                                    .collect();
                                let inner = egui::Frame::new()
                                    .fill(if selected { p.accent_soft } else { p.surface_2 })
                                    .stroke(egui::Stroke::new(
                                        1.0,
                                        if selected { p.accent } else { p.border },
                                    ))
                                    .corner_radius(8)
                                    .inner_margin(egui::Margin::symmetric(10, 8))
                                    .show(ui, |ui| {
                                        ui.set_width(ui.available_width());
                                        ui.horizontal(|ui| {
                                            ui.label(
                                                RichText::new(&snippet.title)
                                                    .strong()
                                                    .color(p.text),
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
                                        ui.label(
                                            RichText::new(preview)
                                                .small()
                                                .color(p.muted)
                                                .monospace(),
                                        );
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
                                && let Some(snippet) =
                                    self.store.snippets.iter().find(|s| s.id == id)
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
                    &p,
                    "No snippet selected",
                    "Keep paste-ready text here and copy it in one click.",
                );
                return;
            };
            let Some(index) = self.store.snippets.iter().position(|s| s.id == id) else {
                return;
            };

            let (lines, chars) = {
                let body = &self.store.snippets[index].body;
                let lines = if body.is_empty() {
                    0
                } else {
                    body.lines().count()
                };
                (lines, body.chars().count())
            };

            let mut dirty = false;
            let mut copy_now = false;
            theme::card(&p).show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.vertical(|ui| {
                        ui.label(theme::muted(&p, "Title"));
                        if ui
                            .add(
                                egui::TextEdit::singleline(&mut self.store.snippets[index].title)
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
                        if theme::primary_button(ui, &p, "Copy to clipboard").clicked() {
                            copy_now = true;
                        }
                    });
                });
                ui.add_space(6.0);
                ui.horizontal(|ui| {
                    ui.label(theme::muted(&p, "Text"));
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.label(
                            RichText::new(format!("{lines} lines · {chars} chars"))
                                .small()
                                .color(p.muted),
                        );
                    });
                });
                ui.add_space(4.0);
                if theme::scrollable_multiline(
                    ui,
                    &p,
                    format!("snippet-body-{id}"),
                    &mut self.store.snippets[index].body,
                )
                .changed()
                {
                    dirty = true;
                }
            });
            if copy_now {
                let body = self.store.snippets[index].body.clone();
                let title = self.store.snippets[index].title.clone();
                ui.ctx().copy_text(body);
                self.status = format!("Copied “{title}”.");
            }
            if dirty {
                self.mark_store_dirty();
            }
        });
    }
}
