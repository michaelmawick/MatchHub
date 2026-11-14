use eframe::egui::{self, RichText};

use crate::theme;

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

#[derive(Default)]
pub struct AuthScreen {
    mode: Mode,
    name: String,
    email: String,
    password: String,
    password_confirm: String,
    notice: Option<Notice>,
}

impl AuthScreen {
    pub fn show(&mut self, ui: &mut egui::Ui) {
        let content_height = match self.mode {
            Mode::Login => 560.0,
            Mode::Register => 680.0,
        };

        theme::centered(ui, content_height, |ui| {
            theme::card(ui, |ui| {
                ui.vertical_centered(|ui| {
                    let (title, subtitle) = match self.mode {
                        Mode::Login => ("Anmelden", "Schön, dass du wieder da bist."),
                        Mode::Register => ("Konto erstellen", "Erstelle dein MatchHub-Konto."),
                    };
                    ui.label(RichText::new(title).heading().strong());
                    ui.label(RichText::new(subtitle).color(theme::TEXT_MUTED));
                });

                ui.add_space(16.0);
                self.mode_switch(ui);
                ui.add_space(16.0);

                if theme::secondary_button(ui, "  Mit Google fortfahren").clicked() {
                    self.notice = Some(Notice::Info("Die Anmeldung mit Google ist noch nicht verfügbar.".to_owned()));
                }
                if theme::secondary_button(ui, "  Mit Apple fortfahren").clicked() {
                    self.notice = Some(Notice::Info("Die Anmeldung mit Apple ist noch nicht verfügbar.".to_owned()));
                }

                ui.add_space(8.0);
                divider(ui, "oder mit E-Mail");
                ui.add_space(8.0);

                let mut submitted = false;

                if self.mode == Mode::Register {
                    submitted |= text_field(ui, &mut self.name, "Name", false);
                }
                submitted |= text_field(ui, &mut self.email, "E-Mail", false);
                submitted |= text_field(ui, &mut self.password, "Passwort", true);
                if self.mode == Mode::Register {
                    submitted |= text_field(ui, &mut self.password_confirm, "Passwort bestätigen", true);
                }

                if let Some(notice) = &self.notice {
                    let (text, color) = match notice {
                        Notice::Error(text) => (text, theme::ERROR),
                        Notice::Info(text) => (text, theme::SUCCESS),
                    };
                    ui.label(RichText::new(text).color(color));
                }

                ui.add_space(4.0);
                let submit_label = match self.mode {
                    Mode::Login => "Anmelden",
                    Mode::Register => "Registrieren",
                };
                if theme::primary_button(ui, submit_label).clicked() || submitted {
                    self.submit();
                }

                ui.add_space(8.0);
                ui.vertical_centered(|ui| {
                    let (question, action, target) = match self.mode {
                        Mode::Login => ("Noch kein Konto?", "Jetzt registrieren", Mode::Register),
                        Mode::Register => ("Schon ein Konto?", "Jetzt anmelden", Mode::Login),
                    };
                    ui.horizontal(|ui| {
                        let width = ui.fonts_mut(|fonts| {
                            let font = egui::TextStyle::Body.resolve(ui.style());
                            fonts.layout_no_wrap(format!("{question} {action}"), font, theme::TEXT).size().x
                        });
                        ui.add_space(((ui.available_width() - width) / 2.0 - 4.0).max(0.0));
                        ui.label(RichText::new(question).color(theme::TEXT_MUTED));
                        if ui.link(RichText::new(action).color(theme::ACCENT_HOVER)).clicked() {
                            self.set_mode(target);
                        }
                    });
                });
            });
        });
    }

    fn mode_switch(&mut self, ui: &mut egui::Ui) {
        let width = (ui.available_width() - ui.spacing().item_spacing.x) / 2.0;
        ui.horizontal(|ui| {
            for (mode, label) in [(Mode::Login, "Anmelden"), (Mode::Register, "Registrieren")] {
                let selected = self.mode == mode;
                let text = RichText::new(label).color(if selected { theme::TEXT } else { theme::TEXT_MUTED });
                let mut button = egui::Button::new(text).corner_radius(egui::CornerRadius::same(theme::RADIUS));
                if selected {
                    button = button.fill(theme::SURFACE_HOVER).stroke(egui::Stroke::new(1.0, theme::ACCENT));
                }
                if ui.add_sized([width, 36.0], button).clicked() {
                    self.set_mode(mode);
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

    fn submit(&mut self) {
        self.notice = Some(match self.validate() {
            Err(error) => Notice::Error(error),
            Ok(()) => Notice::Info(match self.mode {
                Mode::Login => "Eingaben gültig. Die Anmeldung ist noch nicht an einen Server angebunden.".to_owned(),
                Mode::Register => "Eingaben gültig. Die Registrierung ist noch nicht an einen Server angebunden.".to_owned(),
            }),
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

fn text_field(ui: &mut egui::Ui, value: &mut String, hint: &str, password: bool) -> bool {
    let response = ui.add(
        egui::TextEdit::singleline(value)
            .hint_text(RichText::new(hint).color(theme::TEXT_MUTED))
            .password(password)
            .desired_width(f32::INFINITY)
            .margin(egui::vec2(12.0, 12.0)),
    );
    response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter))
}

fn divider(ui: &mut egui::Ui, text: &str) {
    ui.horizontal(|ui| {
        let galley = ui.painter().layout_no_wrap(
            text.to_owned(),
            egui::TextStyle::Small.resolve(ui.style()),
            theme::TEXT_MUTED,
        );
        let line_width = ((ui.available_width() - galley.size().x) / 2.0 - 12.0).max(0.0);
        let (rect, _) = ui.allocate_exact_size(egui::vec2(ui.available_width(), galley.size().y), egui::Sense::hover());
        let y = rect.center().y;
        let stroke = egui::Stroke::new(1.0, theme::BORDER);
        ui.painter().hline(rect.left()..=rect.left() + line_width, y, stroke);
        ui.painter().hline(rect.right() - line_width..=rect.right(), y, stroke);
        ui.painter().galley(
            egui::pos2(rect.center().x - galley.size().x / 2.0, rect.top()),
            galley,
            theme::TEXT_MUTED,
        );
    });
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
