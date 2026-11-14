mod app;
mod screens;
mod theme;

use eframe::egui;

fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("MatchHub")
            .with_inner_size([1280.0, 720.0])
            .with_min_inner_size([480.0, 600.0]),
        ..Default::default()
    };

    eframe::run_native(
        "MatchHub",
        options,
        Box::new(|cc| Ok(Box::new(app::MatchHubApp::new(cc)))),
    )
}
