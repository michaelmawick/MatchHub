use eframe::egui::{self, RichText};

use crate::session::Session;
use crate::theme;

pub fn show(ui: &mut egui::Ui, session: &Session) -> bool {
    let mut logout_clicked = false;

    theme::centered(ui, 260.0, |ui| {
        theme::card(ui, |ui| {
            ui.vertical_centered(|ui| {
                ui.label(
                    RichText::new(format!("Hallo, {}", session.name))
                        .heading()
                        .strong(),
                );
                ui.label(
                    RichText::new(format!("Angemeldet als {}", session.email))
                        .color(theme::TEXT_MUTED),
                );
                ui.add_space(24.0);

                if theme::secondary_button(ui, "Abmelden").clicked() {
                    logout_clicked = true;
                }
            });
        });
    });

    logout_clicked
}
