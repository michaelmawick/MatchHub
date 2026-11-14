use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use eframe::egui::{self, Align, CursorIcon, Id, Layout, RichText, vec2};

use crate::anim::{self, Reveal};
use crate::session::Session;
use crate::supabase::oauth::{self, Provider};
use crate::supabase::{Client, SignUpOutcome};
use crate::task::{Poll, Task};
use crate::theme;
use crate::widgets::{self, Icon};

pub const REVEAL_KEY: &str = "auth";

const CARD_WIDTH: f32 = 440.0;
const HERO_WIDTH: f32 = 440.0;
const COLUMN_GAP: f32 = 72.0;

const NOT_CONFIGURED: &str =
    "Supabase ist nicht konfiguriert. Lege .cargo/config.toml nach dem Beispiel an.";

const HIGHLIGHTS: [&str; 3] = [
    "Matches planen und live verfolgen",
    "Steuerung über den MatchHub Controller",
    "Deine Daten sicher in der Cloud",
];

#[derive(Default, PartialEq, Eq, Clone, Copy)]
enum Mode {
    #[default]
    Login,
    Register,
}

enum Notice {
    Error(String),
    Info(String),
}

enum Pending {
    SignIn(Task<Result<Session, String>>),
    SignUp {
        email: String,
        task: Task<Result<SignUpOutcome, String>>,
    },
    OAuth {
        provider: Provider,
        task: Task<Result<Session, String>>,
        cancelled: Arc<AtomicBool>,
    },
}

#[derive(Default)]
pub struct AuthScreen {
    mode: Mode,
    name: String,
    email: String,
    password: String,
    password_confirm: String,
    notice: Option<Notice>,
    pending: Option<Pending>,
}

impl AuthScreen {
    pub fn show(&mut self, ui: &mut egui::Ui, client: Option<&Client>) -> Option<Session> {
        let session = self.poll_pending();
        let reveal = Reveal::new(ui.ctx(), REVEAL_KEY);
        let slide = (1.0 - reveal.progress(0.0, 0.8)) * 28.0;
        let wide = ui.available_width() >= CARD_WIDTH + HERO_WIDTH + COLUMN_GAP + 96.0;

        anim::page(ui, REVEAL_KEY, slide, |ui| {
            if wide {
                let total = HERO_WIDTH + COLUMN_GAP + CARD_WIDTH;
                let padding = ((ui.available_width() - total) / 2.0).max(0.0);
                let card_height = ui
                    .ctx()
                    .data(|data| data.get_temp::<f32>(Id::unique("auth_card_height")))
                    .unwrap_or(560.0);

                ui.allocate_ui_with_layout(
                    vec2(ui.available_width(), card_height),
                    Layout::left_to_right(Align::Center),
                    |ui| {
                        ui.add_space(padding);
                        ui.allocate_ui_with_layout(
                            vec2(HERO_WIDTH, card_height),
                            Layout::top_down(Align::Min),
                            |ui| {
                                ui.set_width(HERO_WIDTH);
                                hero(ui, &reveal);
                            },
                        );
                        ui.add_space(COLUMN_GAP);
                        let response = ui.scope(|ui| self.card(ui, client, &reveal)).response;
                        ui.ctx().data_mut(|data| {
                            data.insert_temp(Id::unique("auth_card_height"), response.rect.height())
                        });
                    },
                );
            } else {
                ui.vertical_centered(|ui| {
                    reveal.fade(ui, 0.0, |ui| widgets::logo(ui, 56.0));
                    ui.add_space(20.0);
                    self.card(ui, client, &reveal);
                });
            }
        });

        if session.is_some() {
            *self = Self::default();
        }
        session
    }

    fn card(&mut self, ui: &mut egui::Ui, client: Option<&Client>, reveal: &Reveal) {
        let busy = self.pending.is_some();

        reveal.fade(ui, 0.15, |ui| {
            widgets::card(ui, CARD_WIDTH, |ui| {
                let (title, subtitle) = match self.mode {
                    Mode::Login => ("Willkommen zurück", "Melde dich an, um fortzufahren."),
                    Mode::Register => (
                        "Konto erstellen",
                        "Starte in wenigen Sekunden mit MatchHub.",
                    ),
                };
                ui.label(RichText::new(title).font(theme::bold(26.0)));
                ui.add_space(-6.0);
                ui.label(RichText::new(subtitle).color(theme::TEXT_MUTED));
                ui.add_space(8.0);

                ui.add_enabled_ui(!busy, |ui| {
                    let selected = match self.mode {
                        Mode::Login => 0,
                        Mode::Register => 1,
                    };
                    if let Some(index) = widgets::segmented(
                        ui,
                        Id::unique("auth_mode"),
                        &["Anmelden", "Registrieren"],
                        selected,
                    ) {
                        self.set_mode(if index == 0 {
                            Mode::Login
                        } else {
                            Mode::Register
                        });
                    }

                    ui.add_space(6.0);
                    let half = (ui.available_width() - ui.spacing().item_spacing.x) / 2.0;
                    ui.horizontal(|ui| {
                        if widgets::secondary_button(ui, "Google", half, Some(Icon::Google))
                            .clicked()
                        {
                            self.start_oauth(ui.ctx(), client, Provider::Google);
                        }
                        if widgets::secondary_button(ui, "Apple", half, Some(Icon::Apple)).clicked()
                        {
                            self.start_oauth(ui.ctx(), client, Provider::Apple);
                        }
                    });

                    ui.add_space(4.0);
                    widgets::divider(ui, "ODER MIT E-MAIL");
                    ui.add_space(4.0);

                    let mut submitted = false;
                    if self.mode == Mode::Register {
                        submitted |= widgets::text_field(ui, &mut self.name, "Name", false);
                    }
                    submitted |= widgets::text_field(ui, &mut self.email, "E-Mail-Adresse", false);
                    submitted |= widgets::text_field(ui, &mut self.password, "Passwort", true);
                    if self.mode == Mode::Register {
                        submitted |= widgets::text_field(
                            ui,
                            &mut self.password_confirm,
                            "Passwort bestätigen",
                            true,
                        );
                    }

                    if let Some(notice) = &self.notice {
                        match notice {
                            Notice::Error(text) => widgets::notice(ui, text, theme::ERROR),
                            Notice::Info(text) => widgets::notice(ui, text, theme::SUCCESS),
                        }
                    }

                    ui.add_space(4.0);
                    let label = match self.mode {
                        Mode::Login => "Anmelden",
                        Mode::Register => "Konto erstellen",
                    };
                    let width = ui.available_width();
                    if widgets::primary_button(ui, label, width).clicked() || submitted {
                        self.submit(ui.ctx(), client);
                    }
                });

                if busy {
                    self.pending_status(ui);
                }

                ui.add_space(4.0);
                ui.add_enabled_ui(!busy, |ui| self.mode_footer(ui));
            });
        });
    }

    fn mode_footer(&mut self, ui: &mut egui::Ui) {
        let (question, action, target) = match self.mode {
            Mode::Login => ("Noch kein Konto?", "Jetzt registrieren", Mode::Register),
            Mode::Register => ("Schon ein Konto?", "Jetzt anmelden", Mode::Login),
        };

        let font = egui::TextStyle::Body.resolve(ui.style());
        let width = ui.fonts_mut(|fonts| {
            fonts
                .layout_no_wrap(format!("{question} {action}"), font, theme::TEXT)
                .size()
                .x
        });

        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 4.0;
            ui.add_space(((ui.available_width() - width) / 2.0 - 2.0).max(0.0));
            ui.label(RichText::new(question).color(theme::TEXT_MUTED));
            let link = ui
                .add(
                    egui::Label::new(
                        RichText::new(action)
                            .font(theme::semibold(15.0))
                            .color(theme::PRIMARY_GLOW),
                    )
                    .sense(egui::Sense::click()),
                )
                .on_hover_cursor(CursorIcon::PointingHand);
            if link.clicked() {
                self.set_mode(target);
            }
        });
    }

    fn pending_status(&mut self, ui: &mut egui::Ui) {
        let text = match &self.pending {
            Some(Pending::OAuth { provider, .. }) => {
                format!("Warte auf Anmeldung mit {} im Browser …", provider.label())
            }
            Some(Pending::SignUp { .. }) => "Konto wird erstellt …".to_owned(),
            _ => "Anmeldung läuft …".to_owned(),
        };

        ui.horizontal(|ui| {
            ui.add(egui::Spinner::new().size(16.0).color(theme::PRIMARY_GLOW));
            ui.label(RichText::new(text).color(theme::TEXT_MUTED));
            if let Some(Pending::OAuth { cancelled, .. }) = &self.pending {
                let cancel = ui
                    .add(
                        egui::Label::new(
                            RichText::new("Abbrechen")
                                .font(theme::semibold(15.0))
                                .color(theme::PRIMARY_GLOW),
                        )
                        .sense(egui::Sense::click()),
                    )
                    .on_hover_cursor(CursorIcon::PointingHand);
                if cancel.clicked() {
                    cancelled.store(true, Ordering::Relaxed);
                    self.pending = None;
                }
            }
        });
    }

    fn set_mode(&mut self, mode: Mode) {
        if self.mode != mode {
            self.mode = mode;
            self.password.clear();
            self.password_confirm.clear();
            self.notice = None;
        }
    }

    fn poll_pending(&mut self) -> Option<Session> {
        let result = match self.pending.as_ref()? {
            Pending::SignIn(task) | Pending::OAuth { task, .. } => match task.poll() {
                Poll::Pending => return None,
                Poll::Ready(result) => result.map(Some),
                Poll::Lost => Err("Unerwarteter Fehler bei der Anmeldung.".to_owned()),
            },
            Pending::SignUp { email, task } => match task.poll() {
                Poll::Pending => return None,
                Poll::Ready(Ok(SignUpOutcome::SignedIn(session))) => Ok(Some(session)),
                Poll::Ready(Ok(SignUpOutcome::ConfirmationRequired)) => {
                    let email = email.clone();
                    self.pending = None;
                    self.mode = Mode::Login;
                    self.password.clear();
                    self.password_confirm.clear();
                    self.notice = Some(Notice::Info(format!(
                        "Fast geschafft! Wir haben dir eine E-Mail an {email} geschickt. \
                         Bestätige deine Adresse und melde dich dann hier an."
                    )));
                    return None;
                }
                Poll::Ready(Err(error)) => Err(error),
                Poll::Lost => Err("Unerwarteter Fehler bei der Registrierung.".to_owned()),
            },
        };

        self.pending = None;
        match result {
            Ok(session) => session,
            Err(error) => {
                self.notice = Some(Notice::Error(error));
                None
            }
        }
    }

    fn start_oauth(&mut self, ctx: &egui::Context, client: Option<&Client>, provider: Provider) {
        let Some(client) = client.cloned() else {
            self.notice = Some(Notice::Error(NOT_CONFIGURED.to_owned()));
            return;
        };
        self.notice = None;
        let cancelled = Arc::new(AtomicBool::new(false));
        let flag = cancelled.clone();
        let task = Task::spawn(ctx, move || oauth::sign_in(&client, provider, flag));
        self.pending = Some(Pending::OAuth {
            provider,
            task,
            cancelled,
        });
    }

    fn submit(&mut self, ctx: &egui::Context, client: Option<&Client>) {
        if let Err(error) = self.validate() {
            self.notice = Some(Notice::Error(error));
            return;
        }
        let Some(client) = client.cloned() else {
            self.notice = Some(Notice::Error(NOT_CONFIGURED.to_owned()));
            return;
        };

        self.notice = None;
        let email = self.email.trim().to_owned();
        let password = self.password.clone();

        self.pending = Some(match self.mode {
            Mode::Login => {
                Pending::SignIn(Task::spawn(ctx, move || client.sign_in(&email, &password)))
            }
            Mode::Register => {
                let name = self.name.trim().to_owned();
                Pending::SignUp {
                    email: email.clone(),
                    task: Task::spawn(ctx, move || client.sign_up(&name, &email, &password)),
                }
            }
        });
    }

    fn validate(&self) -> Result<(), String> {
        if self.mode == Mode::Register && self.name.trim().is_empty() {
            return Err("Bitte gib deinen Namen ein.".to_owned());
        }
        if !is_valid_email(self.email.trim()) {
            return Err("Bitte gib eine gültige E-Mail-Adresse ein.".to_owned());
        }
        if self.password.is_empty() {
            return Err("Bitte gib dein Passwort ein.".to_owned());
        }
        if self.mode == Mode::Register {
            if self.password.chars().count() < 8 {
                return Err("Das Passwort muss mindestens 8 Zeichen lang sein.".to_owned());
            }
            if self.password != self.password_confirm {
                return Err("Die Passwörter stimmen nicht überein.".to_owned());
            }
        }
        Ok(())
    }
}

fn hero(ui: &mut egui::Ui, reveal: &Reveal) {
    reveal.fade(ui, 0.0, |ui| {
        ui.horizontal(|ui| {
            widgets::logo(ui, 40.0);
            ui.add_space(4.0);
            ui.label(RichText::new("MatchHub").font(theme::bold(22.0)));
        });
    });
    ui.add_space(28.0);

    reveal.fade(ui, 0.1, |ui| {
        ui.label(RichText::new("Dein Spieltag.").font(theme::bold(44.0)));
        ui.add_space(-12.0);
        ui.label(
            RichText::new("Perfekt organisiert.")
                .font(theme::bold(44.0))
                .color(theme::PRIMARY_GLOW),
        );
    });
    ui.add_space(12.0);

    reveal.fade(ui, 0.2, |ui| {
        ui.label(
            RichText::new(
                "Plane Matches, steuere sie live über den Controller und behalte \
                 alle Ergebnisse im Blick.",
            )
            .size(17.0)
            .color(theme::TEXT_MUTED),
        );
    });
    ui.add_space(24.0);

    for (index, text) in HIGHLIGHTS.iter().enumerate() {
        reveal.fade(ui, 0.3 + index as f32 * 0.1, |ui| {
            ui.horizontal(|ui| {
                let (rect, _) = ui.allocate_exact_size(vec2(22.0, 22.0), egui::Sense::hover());
                let painter = ui.painter();
                painter.circle_filled(rect.center(), 11.0, theme::PRIMARY.gamma_multiply(0.25));
                painter.add(egui::epaint::PathShape::line(
                    vec![
                        rect.center() + vec2(-4.5, 0.5),
                        rect.center() + vec2(-1.0, 4.0),
                        rect.center() + vec2(5.0, -3.5),
                    ],
                    egui::Stroke::new(2.0, theme::PRIMARY_GLOW),
                ));
                ui.add_space(4.0);
                ui.label(RichText::new(*text).size(15.5).color(theme::TEXT));
            });
        });
    }
}

fn is_valid_email(email: &str) -> bool {
    let Some((local, domain)) = email.split_once('@') else {
        return false;
    };
    !local.is_empty()
        && !domain.contains('@')
        && domain.split('.').count() >= 2
        && domain.split('.').all(|part| !part.is_empty())
        && !email.contains(char::is_whitespace)
}
