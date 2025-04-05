use eframe::egui;

fn ui(ctx: &egui::Context) {
    egui::Window::new("Log").show(ctx, |ui| {
        // draws the logger ui.
        egui_logger::logger_ui().show(ui);
    });
}