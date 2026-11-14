mod anim;
mod app;
mod screens;
mod session;
mod supabase;
mod task;
mod theme;
mod widgets;

use eframe::egui;

fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("MatchHub")
            .with_inner_size([1280.0, 820.0])
            .with_min_inner_size([520.0, 640.0]),
        ..Default::default()
    };

    eframe::run_native(
        "MatchHub",
        options,
        Box::new(|cc| Ok(Box::new(app::MatchHubApp::new(cc)))),
    )
}
