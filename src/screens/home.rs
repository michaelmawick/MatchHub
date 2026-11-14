use eframe::egui::{self, RichText};

use crate::anim;
use crate::session::Session;
use crate::theme;
use crate::widgets;

pub fn show(ui: &mut egui::Ui, session: &Session) -> bool {
    let mut logout_clicked = false;

    anim::page(ui, "home", 0.0, |ui| {
        ui.vertical_centered(|ui| {
            widgets::card(ui, 420.0, |ui| {
                ui.vertical_centered(|ui| {
                    ui.label(
                        RichText::new(format!("Hallo, {}", session.name)).font(theme::bold(26.0)),
                    );
                    ui.label(
                        RichText::new(format!("Angemeldet als {}", session.email))
                            .color(theme::TEXT_MUTED),
                    );
                    ui.add_space(16.0);
                    let width = ui.available_width();
                    if widgets::secondary_button(ui, "Abmelden", width, None).clicked() {
                        logout_clicked = true;
                    }
                });
            });
        });
    });

    logout_clicked
}
