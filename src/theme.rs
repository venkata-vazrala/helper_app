use eframe::egui::{
    self, Color32, CornerRadius, FontId, Frame, Margin, RichText, Stroke, TextStyle, Vec2,
};

pub const BG: Color32 = Color32::from_rgb(16, 18, 21);
pub const SURFACE: Color32 = Color32::from_rgb(28, 32, 37);
pub const SURFACE_2: Color32 = Color32::from_rgb(36, 41, 47);
pub const BORDER: Color32 = Color32::from_rgb(58, 66, 74);
pub const ACCENT: Color32 = Color32::from_rgb(62, 176, 162);
pub const ACCENT_DIM: Color32 = Color32::from_rgb(28, 72, 68);
pub const ACCENT_SOFT: Color32 = Color32::from_rgb(34, 58, 56);
pub const TEXT: Color32 = Color32::from_rgb(236, 240, 243);
pub const MUTED: Color32 = Color32::from_rgb(148, 160, 170);
pub const DANGER: Color32 = Color32::from_rgb(196, 84, 84);
pub const WARN: Color32 = Color32::from_rgb(214, 164, 88);

pub fn setup(ctx: &egui::Context) {
    let mut visuals = egui::Visuals::dark();
    visuals.panel_fill = BG;
    visuals.window_fill = SURFACE;
    visuals.extreme_bg_color = Color32::from_rgb(12, 14, 16);
    visuals.faint_bg_color = SURFACE_2;
    visuals.window_stroke = Stroke::new(1.0, BORDER);
    visuals.window_corner_radius = CornerRadius::same(10);
    visuals.widgets.noninteractive.bg_fill = SURFACE;
    visuals.widgets.noninteractive.fg_stroke = Stroke::new(1.0, MUTED);
    visuals.widgets.noninteractive.corner_radius = CornerRadius::same(7);
    visuals.widgets.inactive.bg_fill = SURFACE_2;
    visuals.widgets.inactive.weak_bg_fill = SURFACE_2;
    visuals.widgets.inactive.fg_stroke = Stroke::new(1.0, TEXT);
    visuals.widgets.inactive.corner_radius = CornerRadius::same(7);
    visuals.widgets.hovered.bg_fill = Color32::from_rgb(48, 70, 68);
    visuals.widgets.hovered.weak_bg_fill = Color32::from_rgb(48, 70, 68);
    visuals.widgets.hovered.corner_radius = CornerRadius::same(7);
    visuals.widgets.active.bg_fill = ACCENT_DIM;
    visuals.widgets.active.corner_radius = CornerRadius::same(7);
    visuals.selection.bg_fill = ACCENT_DIM;
    visuals.selection.stroke = Stroke::new(1.0, ACCENT);
    visuals.hyperlink_color = ACCENT;
    visuals.override_text_color = Some(TEXT);

    ctx.set_theme(egui::Theme::Dark);
    ctx.style_mut_of(egui::Theme::Dark, |style| {
        style.visuals = visuals;
        style.spacing.item_spacing = Vec2::new(10.0, 8.0);
        style.spacing.button_padding = Vec2::new(12.0, 7.0);
        style.spacing.interact_size.y = 30.0;
        style.spacing.window_margin = Margin::same(12);
        style
            .text_styles
            .insert(TextStyle::Heading, FontId::proportional(22.0));
        style
            .text_styles
            .insert(TextStyle::Body, FontId::proportional(15.0));
        style
            .text_styles
            .insert(TextStyle::Button, FontId::proportional(14.0));
        style
            .text_styles
            .insert(TextStyle::Monospace, FontId::monospace(13.5));
        style
            .text_styles
            .insert(TextStyle::Small, FontId::proportional(12.0));
    });
}

pub fn card() -> Frame {
    Frame::new()
        .fill(SURFACE)
        .stroke(Stroke::new(1.0, BORDER))
        .corner_radius(10)
        .inner_margin(Margin::symmetric(14, 12))
}

pub fn inset() -> Frame {
    Frame::new()
        .fill(Color32::from_rgb(20, 23, 26))
        .stroke(Stroke::new(1.0, BORDER))
        .corner_radius(8)
        .inner_margin(Margin::symmetric(10, 8))
}

pub fn toolbar() -> Frame {
    Frame::new()
        .fill(SURFACE)
        .stroke(Stroke::new(1.0, BORDER))
        .corner_radius(10)
        .inner_margin(Margin::symmetric(10, 8))
}

pub fn primary_button(ui: &mut egui::Ui, label: &str) -> egui::Response {
    ui.add(
        egui::Button::new(
            RichText::new(label)
                .color(Color32::from_rgb(12, 24, 22))
                .strong(),
        )
        .fill(ACCENT)
        .corner_radius(7)
        .min_size(Vec2::new(0.0, 30.0)),
    )
}

pub fn danger_button(ui: &mut egui::Ui, label: &str) -> egui::Response {
    ui.add(
        egui::Button::new(RichText::new(label).color(Color32::WHITE).strong())
            .fill(DANGER)
            .corner_radius(7)
            .min_size(Vec2::new(0.0, 30.0)),
    )
}

pub fn ghost_button(ui: &mut egui::Ui, label: &str) -> egui::Response {
    ui.add(
        egui::Button::new(RichText::new(label).color(TEXT))
            .fill(SURFACE_2)
            .stroke(Stroke::new(1.0, BORDER))
            .corner_radius(7)
            .min_size(Vec2::new(0.0, 30.0)),
    )
}

pub fn search_field(
    ui: &mut egui::Ui,
    value: &mut String,
    hint: &str,
    width: f32,
) -> egui::Response {
    ui.add(
        egui::TextEdit::singleline(value)
            .desired_width(width)
            .hint_text(RichText::new(hint).color(MUTED))
            .background_color(Color32::from_rgb(20, 23, 26)),
    )
}

pub fn section_label(ui: &mut egui::Ui, title: &str, hint: &str) {
    ui.horizontal(|ui| {
        ui.label(RichText::new(title).strong().color(TEXT).size(15.0));
        if !hint.is_empty() {
            ui.label(RichText::new(hint).color(MUTED).size(12.0));
        }
    });
}

pub fn empty_state(ui: &mut egui::Ui, title: &str, body: &str) {
    ui.vertical_centered(|ui| {
        ui.add_space(48.0);
        ui.label(RichText::new(title).size(18.0).strong().color(TEXT));
        ui.add_space(6.0);
        ui.label(RichText::new(body).color(MUTED));
    });
}

pub fn muted(text: impl Into<String>) -> RichText {
    RichText::new(text.into()).color(MUTED)
}
