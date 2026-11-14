use eframe::egui;

fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("MatchHub")
            .with_inner_size([1280.0, 720.0]),
        ..Default::default()
    };

    eframe::run_native(
        "MatchHub",
        options,
        Box::new(|_cc| Ok(Box::new(MatchHubApp::default()))),
    )
}

#[derive(Default)]
struct MatchHubApp;

impl eframe::App for MatchHubApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ui, |ui| {
            ui.heading("MatchHub");
        });

        ui.ctx().show_viewport_deferred(
            egui::ViewportId::from_hash_of("matchhub_controller"),
            egui::ViewportBuilder::default()
                .with_title("MatchHub Controller")
                .with_inner_size([480.0, 640.0]),
            |ui, _class| {
                egui::CentralPanel::default().show(ui, |ui| {
                    ui.heading("MatchHub Controller");
                });

                if ui.ctx().input(|i| i.viewport().close_requested()) {
                    ui.ctx()
                        .send_viewport_cmd_to(egui::ViewportId::ROOT, egui::ViewportCommand::Close);
                }
            },
        );
    }
}
