use eframe::egui::{self, Align, CornerRadius, Layout, RichText, Stroke, vec2};

use crate::anim::{self, Reveal};
use crate::theme;
use crate::widgets;

pub const REVEAL_KEY: &str = "welcome";

const FEATURES: [(&str, &str, &str); 3] = [
    (
        "📅",
        "Matches planen",
        "Termine, Teams und Ergebnisse an einem Ort.",
    ),
    (
        "🎮",
        "Live steuern",
        "Der Controller gibt dir volle Kontrolle.",
    ),
    (
        "📊",
        "Alles im Blick",
        "Statistiken und Verläufe auf einen Blick.",
    ),
];

pub fn show(ui: &mut egui::Ui) -> bool {
    let reveal = Reveal::new(ui.ctx(), REVEAL_KEY);
    let slide = (1.0 - reveal.progress(0.0, 0.8)) * 28.0;
    let mut continue_clicked = false;

    anim::page(ui, REVEAL_KEY, slide, |ui| {
        ui.vertical_centered(|ui| {
            reveal.fade(ui, 0.0, |ui| widgets::logo(ui, 76.0));
            ui.add_space(20.0);

            reveal.fade(ui, 0.12, |ui| {
                ui.label(
                    RichText::new("Willkommen bei")
                        .font(theme::medium(16.0))
                        .color(theme::PRIMARY_GLOW),
                );
                ui.label(RichText::new("MatchHub").font(theme::bold(56.0)));
            });

            reveal.fade(ui, 0.24, |ui| {
                ui.set_max_width(520.0);
                ui.label(
                    RichText::new(
                        "Organisiere deine Matches, steuere alles über den Controller \
                         und behalte jederzeit den Überblick.",
                    )
                    .size(17.0)
                    .color(theme::TEXT_MUTED),
                );
            });

            ui.add_space(28.0);
            features(ui, &reveal);
            ui.add_space(32.0);

            reveal.fade(ui, 0.7, |ui| {
                if widgets::primary_button(ui, "Los geht's  →", 280.0).clicked() {
                    continue_clicked = true;
                }
            });

            ui.add_space(16.0);
            reveal.fade(ui, 0.85, |ui| {
                ui.label(
                    RichText::new(format!("Version {}", env!("CARGO_PKG_VERSION")))
                        .size(12.0)
                        .color(theme::TEXT_SUBTLE),
                );
            });
        });
    });

    continue_clicked
}

fn features(ui: &mut egui::Ui, reveal: &Reveal) {
    let gap = 14.0;
    let available = ui.available_width().min(720.0);
    let stacked = available < 600.0;
    let card_width = if stacked {
        available.min(360.0)
    } else {
        (available - gap * 2.0) / 3.0
    };

    let draw = |ui: &mut egui::Ui, index: usize| {
        let (icon, title, text) = FEATURES[index];
        reveal.fade(ui, 0.36 + index as f32 * 0.1, |ui| {
            egui::Frame::new()
                .fill(theme::SURFACE.gamma_multiply(0.7))
                .stroke(Stroke::new(1.0, theme::BORDER))
                .corner_radius(CornerRadius::same(16))
                .inner_margin(18)
                .show(ui, |ui| {
                    ui.set_width(card_width - 36.0);
                    ui.with_layout(Layout::top_down(Align::Min), |ui| {
                        ui.label(RichText::new(icon).size(22.0).color(theme::PRIMARY_GLOW));
                        ui.add_space(2.0);
                        ui.label(RichText::new(title).font(theme::semibold(16.0)));
                        ui.label(RichText::new(text).size(13.5).color(theme::TEXT_MUTED));
                    });
                });
        });
    };

    if stacked {
        for index in 0..FEATURES.len() {
            draw(ui, index);
        }
    } else {
        let total = card_width * 3.0 + gap * 2.0;
        ui.allocate_ui_with_layout(vec2(total, 0.0), Layout::left_to_right(Align::Min), |ui| {
            ui.spacing_mut().item_spacing.x = gap;
            for index in 0..FEATURES.len() {
                draw(ui, index);
            }
        });
    }
}
