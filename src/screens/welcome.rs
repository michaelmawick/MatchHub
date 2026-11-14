use eframe::egui::{self, RichText};

use crate::theme;

pub fn show(ui: &mut egui::Ui) -> bool {
    let mut continue_clicked = false;

    theme::centered(ui, 360.0, |ui| {
        theme::card(ui, |ui| {
            ui.vertical_centered(|ui| {
                ui.label(RichText::new("⚡").size(48.0).color(theme::ACCENT));
                ui.add_space(8.0);
                ui.label(RichText::new("Willkommen bei MatchHub").heading().strong());
                ui.add_space(4.0);
                ui.label(
                    RichText::new("Organisiere deine Matches, steuere alles über den Controller und behalte jederzeit den Überblick.")
                        .color(theme::TEXT_MUTED),
                );
                ui.add_space(24.0);

                if theme::primary_button(ui, "Los geht's").clicked() {
                    continue_clicked = true;
                }
            });
        });
    });

    continue_clicked
}
