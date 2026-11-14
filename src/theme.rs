use eframe::egui::{self, Color32, CornerRadius, FontId, Stroke, TextStyle};

pub const BACKGROUND: Color32 = Color32::from_rgb(14, 17, 24);
pub const SURFACE: Color32 = Color32::from_rgb(24, 29, 40);
pub const SURFACE_HOVER: Color32 = Color32::from_rgb(34, 40, 54);
pub const BORDER: Color32 = Color32::from_rgb(46, 53, 70);
pub const TEXT: Color32 = Color32::from_rgb(236, 239, 245);
pub const TEXT_MUTED: Color32 = Color32::from_rgb(140, 148, 166);
pub const ACCENT: Color32 = Color32::from_rgb(88, 101, 242);
pub const ACCENT_HOVER: Color32 = Color32::from_rgb(110, 122, 255);
pub const ERROR: Color32 = Color32::from_rgb(242, 98, 98);
pub const SUCCESS: Color32 = Color32::from_rgb(87, 200, 140);

pub const CARD_WIDTH: f32 = 380.0;
pub const CONTROL_HEIGHT: f32 = 44.0;
pub const RADIUS: u8 = 10;

pub fn apply(ctx: &egui::Context) {
    ctx.set_visuals(egui::Visuals::dark());

    ctx.global_style_mut(|style| {
        style.text_styles = [
            (TextStyle::Heading, FontId::proportional(28.0)),
            (TextStyle::Body, FontId::proportional(15.0)),
            (TextStyle::Button, FontId::proportional(15.0)),
            (TextStyle::Small, FontId::proportional(12.0)),
            (TextStyle::Monospace, FontId::monospace(14.0)),
        ]
        .into();

        style.spacing.item_spacing = egui::vec2(8.0, 10.0);
        style.spacing.button_padding = egui::vec2(14.0, 10.0);

        let visuals = &mut style.visuals;
        visuals.panel_fill = BACKGROUND;
        visuals.window_fill = SURFACE;
        visuals.extreme_bg_color = BACKGROUND;
        visuals.override_text_color = Some(TEXT);
        visuals.selection.bg_fill = ACCENT;
        visuals.selection.stroke = Stroke::new(1.0, TEXT);

        let radius = CornerRadius::same(RADIUS);
        for widget in [
            &mut visuals.widgets.inactive,
            &mut visuals.widgets.hovered,
            &mut visuals.widgets.active,
            &mut visuals.widgets.open,
            &mut visuals.widgets.noninteractive,
        ] {
            widget.corner_radius = radius;
        }

        visuals.widgets.inactive.bg_fill = SURFACE;
        visuals.widgets.inactive.weak_bg_fill = SURFACE;
        visuals.widgets.inactive.bg_stroke = Stroke::new(1.0, BORDER);
        visuals.widgets.hovered.bg_fill = SURFACE_HOVER;
        visuals.widgets.hovered.weak_bg_fill = SURFACE_HOVER;
        visuals.widgets.hovered.bg_stroke = Stroke::new(1.0, ACCENT);
        visuals.widgets.active.bg_fill = SURFACE_HOVER;
        visuals.widgets.active.weak_bg_fill = SURFACE_HOVER;
        visuals.widgets.active.bg_stroke = Stroke::new(1.0, ACCENT_HOVER);
        visuals.widgets.noninteractive.bg_stroke = Stroke::new(1.0, BORDER);
    });
}

pub fn card<R>(ui: &mut egui::Ui, add_contents: impl FnOnce(&mut egui::Ui) -> R) -> R {
    egui::Frame::new()
        .fill(SURFACE)
        .stroke(Stroke::new(1.0, BORDER))
        .corner_radius(CornerRadius::same(16))
        .inner_margin(32)
        .show(ui, |ui| {
            ui.set_width(CARD_WIDTH);
            add_contents(ui)
        })
        .inner
}

pub fn centered<R>(
    ui: &mut egui::Ui,
    content_height: f32,
    add_contents: impl FnOnce(&mut egui::Ui) -> R,
) -> R {
    egui::ScrollArea::vertical()
        .auto_shrink([false, false])
        .show(ui, |ui| {
            let top = ((ui.available_height() - content_height) / 2.0).max(24.0);
            ui.vertical_centered(|ui| {
                ui.add_space(top);
                add_contents(ui)
            })
            .inner
        })
        .inner
}

pub fn primary_button(ui: &mut egui::Ui, text: &str) -> egui::Response {
    ui.add_sized(
        [ui.available_width(), CONTROL_HEIGHT],
        egui::Button::new(egui::RichText::new(text).strong().color(Color32::WHITE))
            .fill(ACCENT)
            .stroke(Stroke::NONE)
            .corner_radius(CornerRadius::same(RADIUS)),
    )
}

pub fn secondary_button(ui: &mut egui::Ui, text: &str) -> egui::Response {
    ui.add_sized(
        [ui.available_width(), CONTROL_HEIGHT],
        egui::Button::new(egui::RichText::new(text).color(TEXT))
            .corner_radius(CornerRadius::same(RADIUS)),
    )
}
