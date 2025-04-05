use eframe::egui::{self, TopBottomPanel};

pub fn show_logger(ctx: &egui::Context) {
    TopBottomPanel::bottom("Log").show(ctx, |ui| {
        // draws the logger ui.
        egui_logger::logger_ui().show(ui);
    });
}