use eframe::egui::{self, RichText};

use crate::anim::{self, Reveal};
use crate::screens::auth::{self, AuthScreen};
use crate::screens::{home, welcome};
use crate::session::Session;
use crate::supabase::Client;
use crate::task::{Poll, Task};
use crate::theme;

const WELCOME_SEEN_KEY: &str = "welcome_seen";
const REFRESH_TOKEN_KEY: &str = "refresh_token";

enum Screen {
    Welcome,
    Restoring(Task<Result<Session, String>>),
    Auth,
    Home(Session),
}

pub struct MatchHubApp {
    screen: Screen,
    auth: AuthScreen,
    client: Option<Client>,
}

impl MatchHubApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        theme::apply(&cc.egui_ctx);

        let client = Client::from_env();
        let stored = |key: &str| {
            cc.storage
                .and_then(|storage| storage.get_string(key))
                .filter(|value| !value.is_empty())
        };

        let screen = match (stored(WELCOME_SEEN_KEY), stored(REFRESH_TOKEN_KEY), &client) {
            (None, _, _) => Screen::Welcome,
            (Some(_), Some(refresh_token), Some(client)) => {
                let client = client.clone();
                Screen::Restoring(Task::spawn(&cc.egui_ctx, move || {
                    client.refresh(&refresh_token)
                }))
            }
            _ => Screen::Auth,
        };

        Self {
            screen,
            auth: AuthScreen::default(),
            client,
        }
    }

    fn finish_welcome(&mut self, frame: &mut eframe::Frame) {
        store(frame, WELCOME_SEEN_KEY, "true");
        self.screen = Screen::Auth;
    }

    fn sign_in(&mut self, frame: &mut eframe::Frame, session: Session) {
        store(frame, REFRESH_TOKEN_KEY, &session.refresh_token);
        self.screen = Screen::Home(session);
    }

    fn sign_out(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        store(frame, REFRESH_TOKEN_KEY, "");
        if let (Screen::Home(session), Some(client)) = (&self.screen, &self.client) {
            let client = client.clone();
            let access_token = session.access_token.clone();
            let _ = Task::spawn(ctx, move || client.sign_out(&access_token));
        }
        Reveal::restart(ctx, auth::REVEAL_KEY);
        self.screen = Screen::Auth;
    }
}

fn store(frame: &mut eframe::Frame, key: &str, value: &str) {
    if let Some(storage) = frame.storage_mut() {
        storage.set_string(key, value.to_owned());
        storage.flush();
    }
}

impl eframe::App for MatchHubApp {
    fn ui(&mut self, ui: &mut egui::Ui, frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();
        theme::paint_background(&ctx);

        egui::CentralPanel::default()
            .frame(egui::Frame::new().inner_margin(24))
            .show(ui, |ui| match &self.screen {
                Screen::Welcome => {
                    if welcome::show(ui) {
                        self.finish_welcome(frame);
                    }
                }
                Screen::Restoring(task) => match task.poll() {
                    Poll::Pending => restoring(ui),
                    Poll::Ready(Ok(session)) => self.sign_in(frame, session),
                    Poll::Ready(Err(_)) | Poll::Lost => {
                        store(frame, REFRESH_TOKEN_KEY, "");
                        self.screen = Screen::Auth;
                    }
                },
                Screen::Auth => {
                    if let Some(session) = self.auth.show(ui, self.client.as_ref()) {
                        self.sign_in(frame, session);
                    }
                }
                Screen::Home(session) => {
                    if home::show(ui, session) {
                        self.sign_out(&ctx, frame);
                    }
                }
            });

        if matches!(self.screen, Screen::Home(_)) {
            show_controller(&ctx);
        }
    }
}

fn restoring(ui: &mut egui::Ui) {
    anim::page(ui, "restoring", 0.0, |ui| {
        ui.vertical_centered(|ui| {
            ui.add(egui::Spinner::new().size(28.0).color(theme::PRIMARY_GLOW));
            ui.label(RichText::new("Anmeldung wird wiederhergestellt …").color(theme::TEXT_MUTED));
        });
    });
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
