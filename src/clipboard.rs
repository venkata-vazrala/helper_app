/// Copy to both egui and the OS clipboard. `ctx.copy_text` alone can stay inside the app.
pub fn copy_text(ctx: &eframe::egui::Context, text: impl Into<String>) -> bool {
    let text = text.into();
    ctx.copy_text(text.clone());
    match arboard::Clipboard::new().and_then(|mut clip| clip.set_text(text)) {
        Ok(()) => true,
        Err(_) => false,
    }
}
