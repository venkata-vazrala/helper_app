mod app;
mod config;
mod files;
mod launch;
mod notes;
mod settings;
mod snippets;
mod store;
mod theme;

fn main() -> eframe::Result<()> {
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
