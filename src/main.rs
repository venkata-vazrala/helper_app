mod app;
mod config;
mod files;
mod launch;
mod notes;
mod settings;
mod snippets;
mod store;

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1120.0, 740.0])
            .with_min_inner_size([820.0, 540.0])
            .with_title("Helper"),
        ..Default::default()
    };

    eframe::run_native(
        "Helper",
        options,
        Box::new(|cc| Ok(Box::new(app::HelperApp::new(cc)))),
    )
}
