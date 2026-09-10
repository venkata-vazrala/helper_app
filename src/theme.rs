use eframe::egui::{
    self, Color32, CornerRadius, FontId, Frame, Margin, RichText, Stroke, TextStyle, Theme, Vec2,
};

use crate::config::{Accent, Appearance, Density};

#[derive(Clone, Copy)]
pub struct Palette {
    pub bg: Color32,
    pub surface: Color32,
    pub surface_2: Color32,
    pub inset: Color32,
    pub border: Color32,
    pub accent: Color32,
    pub accent_dim: Color32,
    pub accent_soft: Color32,
    pub text: Color32,
    pub muted: Color32,
    pub danger: Color32,
    pub warn: Color32,
    pub on_accent: Color32,
}

impl Accent {
    pub fn color(self) -> Color32 {
        match self {
            Accent::Teal => Color32::from_rgb(62, 176, 162),
            Accent::Blue => Color32::from_rgb(64, 140, 220),
            Accent::Amber => Color32::from_rgb(214, 164, 88),
            Accent::Rose => Color32::from_rgb(196, 90, 120),
        }
    }
}

impl Palette {
    pub fn new(theme: Theme, accent: Accent) -> Self {
        let accent_c = accent.color();
        match theme {
            Theme::Dark => Self {
                bg: Color32::from_rgb(16, 18, 21),
                surface: Color32::from_rgb(28, 32, 37),
                surface_2: Color32::from_rgb(36, 41, 47),
                inset: Color32::from_rgb(20, 23, 26),
                border: Color32::from_rgb(58, 66, 74),
                accent: accent_c,
                accent_dim: dim(accent_c, 0.35),
                accent_soft: dim(accent_c, 0.22),
                text: Color32::from_rgb(236, 240, 243),
                muted: Color32::from_rgb(148, 160, 170),
                danger: Color32::from_rgb(196, 84, 84),
                warn: Color32::from_rgb(214, 164, 88),
                on_accent: Color32::from_rgb(12, 24, 22),
            },
            Theme::Light => Self {
                bg: Color32::from_rgb(236, 238, 241),
                surface: Color32::from_rgb(255, 255, 255),
                surface_2: Color32::from_rgb(244, 246, 248),
                inset: Color32::from_rgb(248, 249, 251),
                border: Color32::from_rgb(200, 206, 212),
                accent: accent_c,
                accent_dim: mix(accent_c, Color32::from_rgb(255, 255, 255), 0.72),
                accent_soft: mix(accent_c, Color32::from_rgb(255, 255, 255), 0.85),
                text: Color32::from_rgb(22, 26, 30),
                muted: Color32::from_rgb(90, 100, 110),
                danger: Color32::from_rgb(176, 56, 56),
                warn: Color32::from_rgb(170, 120, 32),
                on_accent: Color32::from_rgb(12, 24, 22),
            },
        }
    }
}

pub fn luminance(c: Color32) -> f32 {
    let r = c.r() as f32 / 255.0;
    let g = c.g() as f32 / 255.0;
    let b = c.b() as f32 / 255.0;
    0.2126 * r + 0.7152 * g + 0.0722 * b
}

fn dim(c: Color32, amount: f32) -> Color32 {
    Color32::from_rgb(
        (c.r() as f32 * amount) as u8,
        (c.g() as f32 * amount) as u8,
        (c.b() as f32 * amount) as u8,
    )
}

fn mix(a: Color32, b: Color32, t: f32) -> Color32 {
    let t = t.clamp(0.0, 1.0);
    Color32::from_rgb(
        (a.r() as f32 + (b.r() as f32 - a.r() as f32) * t) as u8,
        (a.g() as f32 + (b.g() as f32 - a.g() as f32) * t) as u8,
        (a.b() as f32 + (b.b() as f32 - a.b() as f32) * t) as u8,
    )
}

pub fn apply(ctx: &egui::Context, theme: Theme, appearance: &Appearance, palette: &Palette) {
    let mut visuals = match theme {
        Theme::Dark => egui::Visuals::dark(),
        Theme::Light => egui::Visuals::light(),
    };
    visuals.panel_fill = palette.bg;
    visuals.window_fill = palette.surface;
    visuals.extreme_bg_color = palette.inset;
    visuals.faint_bg_color = palette.surface_2;
    visuals.window_stroke = Stroke::new(1.0, palette.border);
    visuals.window_corner_radius = CornerRadius::same(10);
    visuals.widgets.noninteractive.bg_fill = palette.surface;
    visuals.widgets.noninteractive.fg_stroke = Stroke::new(1.0, palette.muted);
    visuals.widgets.noninteractive.corner_radius = CornerRadius::same(7);
    visuals.widgets.inactive.bg_fill = palette.surface_2;
    visuals.widgets.inactive.weak_bg_fill = palette.surface_2;
    visuals.widgets.inactive.fg_stroke = Stroke::new(1.0, palette.text);
    visuals.widgets.inactive.corner_radius = CornerRadius::same(7);
    visuals.widgets.hovered.bg_fill = palette.accent_soft;
    visuals.widgets.hovered.weak_bg_fill = palette.accent_soft;
    visuals.widgets.hovered.corner_radius = CornerRadius::same(7);
    visuals.widgets.active.bg_fill = palette.accent_dim;
    visuals.widgets.active.corner_radius = CornerRadius::same(7);
    visuals.selection.bg_fill = palette.accent_dim;
    visuals.selection.stroke = Stroke::new(1.0, palette.accent);
    visuals.hyperlink_color = palette.accent;
    visuals.override_text_color = Some(palette.text);

    let (item, pad, interact) = match appearance.density {
        Density::Comfortable => (Vec2::new(10.0, 8.0), Vec2::new(12.0, 7.0), 30.0),
        Density::Compact => (Vec2::new(6.0, 4.0), Vec2::new(8.0, 5.0), 24.0),
    };
    let scale = appearance.font_scale;

    ctx.set_theme(theme);
    ctx.style_mut_of(theme, |style| {
        style.visuals = visuals;
        style.spacing.item_spacing = item;
        style.spacing.button_padding = pad;
        style.spacing.interact_size.y = interact;
        style.spacing.window_margin = Margin::same(12);
        style
            .text_styles
            .insert(TextStyle::Heading, FontId::proportional(22.0 * scale));
        style
            .text_styles
            .insert(TextStyle::Body, FontId::proportional(15.0 * scale));
        style
            .text_styles
            .insert(TextStyle::Button, FontId::proportional(14.0 * scale));
        style
            .text_styles
            .insert(TextStyle::Monospace, FontId::monospace(13.5 * scale));
        style
            .text_styles
            .insert(TextStyle::Small, FontId::proportional(12.0 * scale));
    });
}

pub fn card(p: &Palette) -> Frame {
    Frame::new()
        .fill(p.surface)
        .stroke(Stroke::new(1.0, p.border))
        .corner_radius(10)
        .inner_margin(Margin::symmetric(14, 12))
}

pub fn inset(p: &Palette) -> Frame {
    Frame::new()
        .fill(p.inset)
        .stroke(Stroke::new(1.0, p.border))
        .corner_radius(8)
        .inner_margin(Margin::symmetric(10, 8))
}

pub fn toolbar(p: &Palette) -> Frame {
    Frame::new()
        .fill(p.surface)
        .stroke(Stroke::new(1.0, p.border))
        .corner_radius(10)
        .inner_margin(Margin::symmetric(10, 8))
}

pub fn primary_button(ui: &mut egui::Ui, p: &Palette, label: &str) -> egui::Response {
    ui.add(
        egui::Button::new(RichText::new(label).color(p.on_accent).strong())
            .fill(p.accent)
            .corner_radius(7)
            .min_size(Vec2::new(0.0, 30.0)),
    )
}

pub fn danger_button(ui: &mut egui::Ui, p: &Palette, label: &str) -> egui::Response {
    ui.add(
        egui::Button::new(RichText::new(label).color(Color32::WHITE).strong())
            .fill(p.danger)
            .corner_radius(7)
            .min_size(Vec2::new(0.0, 30.0)),
    )
}

pub fn ghost_button(ui: &mut egui::Ui, p: &Palette, label: &str) -> egui::Response {
    ui.add(
        egui::Button::new(RichText::new(label).color(p.text))
            .fill(p.surface_2)
            .stroke(Stroke::new(1.0, p.border))
            .corner_radius(7)
            .min_size(Vec2::new(0.0, 30.0)),
    )
}

pub fn search_field(
    ui: &mut egui::Ui,
    p: &Palette,
    value: &mut String,
    hint: &str,
    width: f32,
) -> egui::Response {
    ui.add(
        egui::TextEdit::singleline(value)
            .desired_width(width)
            .hint_text(RichText::new(hint).color(p.muted))
            .background_color(p.inset),
    )
}

pub fn section_label(ui: &mut egui::Ui, p: &Palette, title: &str, hint: &str) {
    ui.horizontal(|ui| {
        ui.label(RichText::new(title).strong().color(p.text).size(15.0));
        if !hint.is_empty() {
            ui.label(RichText::new(hint).color(p.muted).size(12.0));
        }
    });
}

pub fn empty_state(ui: &mut egui::Ui, p: &Palette, title: &str, body: &str) {
    ui.vertical_centered(|ui| {
        ui.add_space(48.0);
        ui.label(RichText::new(title).size(18.0).strong().color(p.text));
        ui.add_space(6.0);
        ui.label(RichText::new(body).color(p.muted));
    });
}

pub fn muted(p: &Palette, text: impl Into<String>) -> RichText {
    RichText::new(text.into()).color(p.muted)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dark_text_is_lighter_than_background() {
        let p = Palette::new(Theme::Dark, Accent::Teal);
        assert!(luminance(p.text) > luminance(p.bg));
    }

    #[test]
    fn light_text_is_darker_than_background() {
        let p = Palette::new(Theme::Light, Accent::Teal);
        assert!(luminance(p.text) < luminance(p.bg));
    }
}
