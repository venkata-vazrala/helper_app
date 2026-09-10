//! Helper workspace library. The binary is a thin native window around this crate.

pub mod app;
pub mod bundle;
pub mod config;
pub mod files;
pub mod launch;
pub mod notes;
pub mod reorder;
pub mod settings;
pub mod snippets;
pub mod store;
pub mod theme;

use eframe::egui;

pub fn run() -> eframe::Result<()> {
    let mut viewport = egui::ViewportBuilder::default()
        .with_inner_size([1180.0, 780.0])
        .with_min_inner_size([880.0, 580.0])
        .with_title("Helper");
    if let Ok(icon) = eframe::icon_data::from_png_bytes(include_bytes!("../assets/icon.png")) {
        viewport = viewport.with_icon(icon);
    }

    let options = eframe::NativeOptions {
        viewport,
        ..Default::default()
    };

    eframe::run_native(
        "Helper",
        options,
        Box::new(|cc| Ok(Box::new(app::HelperApp::new(cc)))),
    )
}
