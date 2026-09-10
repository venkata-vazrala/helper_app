use eframe::egui::{self, Layout, RichText, Sense};

use crate::app::HelperApp;
use crate::clipboard;
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
                ui.with_layout(Layout::right_to_left(egui::Align::Center), |ui| {
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
                        .auto_shrink([false, true])
                        .scroll_bar_visibility(
                            egui::containers::scroll_area::ScrollBarVisibility::AlwaysVisible,
                        )
                        .show(ui, |ui| {
                            if self.store.snippets.is_empty() {
                                ui.label(theme::muted(&p, "No snippets yet."));
                                return;
                            }
                            let mut clicked = None;
                            let mut copied: Option<(String, bool)> = None;
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
                                ui.push_id(&snippet.id, |ui| {
                                    egui::containers::Sides::new()
                                        .height(52.0)
                                        .shrink_left()
                                        .show(
                                            ui,
                                            |ui| {
                                                let inner = egui::Frame::new()
                                                    .fill(if selected {
                                                        p.accent_soft
                                                    } else {
                                                        p.surface_2
                                                    })
                                                    .stroke(egui::Stroke::new(
                                                        1.0,
                                                        if selected { p.accent } else { p.border },
                                                    ))
                                                    .corner_radius(8)
                                                    .inner_margin(egui::Margin::symmetric(10, 6))
                                                    .show(ui, |ui| {
                                                        ui.set_width(ui.available_width());
                                                        ui.add(
                                                            egui::Label::new(
                                                                RichText::new(&snippet.title)
                                                                    .strong()
                                                                    .color(p.text),
                                                            )
                                                            .truncate(),
                                                        );
                                                        ui.add(
                                                            egui::Label::new(
                                                                RichText::new(&preview)
                                                                    .small()
                                                                    .color(p.muted)
                                                                    .monospace(),
                                                            )
                                                            .truncate(),
                                                        );
                                                    });
                                                if inner.response.interact(Sense::click()).clicked()
                                                {
                                                    clicked = Some(snippet.id.clone());
                                                }
                                            },
                                            |ui| {
                                                if theme::ghost_button(ui, &p, "Copy").clicked() {
                                                    let ok = clipboard::copy_text(
                                                        ui.ctx(),
                                                        snippet.body.clone(),
                                                    );
                                                    copied = Some((snippet.title.clone(), ok));
                                                }
                                            },
                                        );
                                });
                                ui.add_space(6.0);
                            }
                            if let Some(id) = clicked {
                                self.selected_snippet = Some(id);
                            }
                            if let Some((title, ok)) = copied {
                                self.status = if ok {
                                    format!("Copied “{title}”.")
                                } else {
                                    "Could not copy to the system clipboard.".into()
                                };
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
                // Compact header: one line of title, copy on the right. Body gets the rest.
                ui.horizontal(|ui| {
                    ui.label(theme::muted(&p, "Title"));
                    ui.with_layout(Layout::right_to_left(egui::Align::Center), |ui| {
                        if theme::primary_button(ui, &p, "Copy")
                            .on_hover_text("Copy body to clipboard")
                            .clicked()
                        {
                            copy_now = true;
                        }
                        ui.add_space(8.0);
                        ui.label(
                            RichText::new(format!("{lines} lines · {chars} chars"))
                                .small()
                                .color(p.muted),
                        );
                    });
                });
                ui.add_space(4.0);
                let title_h = ui.spacing().interact_size.y;
                if ui
                    .add_sized(
                        [ui.available_width(), title_h],
                        egui::TextEdit::singleline(&mut self.store.snippets[index].title)
                            .hint_text("Snippet name"),
                    )
                    .changed()
                {
                    dirty = true;
                }
                ui.add_space(8.0);
                ui.label(theme::muted(&p, "Text"));
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
                let ok = clipboard::copy_text(ui.ctx(), body);
                self.status = if ok {
                    format!("Copied “{title}”.")
                } else {
                    "Could not copy to the system clipboard.".into()
                };
            }
            if dirty {
                self.mark_store_dirty();
            }
        });
    }
}
