use eframe::egui;

use crate::screens::auth::AuthScreen;
use crate::screens::{home, welcome};
use crate::session::Session;
use crate::theme;

const WELCOME_SEEN_KEY: &str = "welcome_seen";

enum Screen {
    Welcome,
    Auth,
    Home(Session),
}

pub struct MatchHubApp {
    screen: Screen,
    auth: AuthScreen,
}

impl MatchHubApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        theme::apply(&cc.egui_ctx);

        let welcome_seen = cc
            .storage
            .and_then(|storage| storage.get_string(WELCOME_SEEN_KEY))
            .is_some_and(|value| value == "true");

        Self {
            screen: if welcome_seen {
                Screen::Auth
            } else {
                Screen::Welcome
            },
            auth: AuthScreen::default(),
        }
    }

    fn finish_welcome(&mut self, frame: &mut eframe::Frame) {
        if let Some(storage) = frame.storage_mut() {
            storage.set_string(WELCOME_SEEN_KEY, "true".to_owned());
            storage.flush();
        }
        self.screen = Screen::Auth;
    }
}

impl eframe::App for MatchHubApp {
    fn ui(&mut self, ui: &mut egui::Ui, frame: &mut eframe::Frame) {
        egui::CentralPanel::default()
            .frame(egui::Frame::new().fill(theme::BACKGROUND))
            .show(ui, |ui| match &self.screen {
                Screen::Welcome => {
                    if welcome::show(ui) {
                        self.finish_welcome(frame);
                    }
                }
                Screen::Auth => {
                    if let Some(session) = self.auth.show(ui) {
                        self.screen = Screen::Home(session);
                    }
                }
                Screen::Home(session) => {
                    if home::show(ui, session) {
                        self.screen = Screen::Auth;
                    }
                }
            });

        if matches!(self.screen, Screen::Home(_)) {
            show_controller(ui.ctx());
        }
    }
}

fn show_controller(ctx: &egui::Context) {
    ctx.show_viewport_deferred(
        egui::ViewportId::from_hash_of("matchhub_controller"),
        egui::ViewportBuilder::default()
            .with_title("MatchHub Controller")
            .with_inner_size([480.0, 640.0]),
        |ui, _class| {
            egui::CentralPanel::default()
                .frame(egui::Frame::new().fill(theme::BACKGROUND).inner_margin(24))
                .show(ui, |ui| {
                    ui.heading("MatchHub Controller");
                });

            if ui.ctx().input(|i| i.viewport().close_requested()) {
                ui.ctx()
                    .send_viewport_cmd_to(egui::ViewportId::ROOT, egui::ViewportCommand::Close);
            }
        },
    );
}
