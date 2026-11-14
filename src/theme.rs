use std::sync::Arc;

use eframe::egui::{
    self, Color32, CornerRadius, FontData, FontDefinitions, FontFamily, FontId, Pos2, Stroke,
    TextStyle, epaint::Mesh, vec2,
};

pub const BACKGROUND: Color32 = Color32::from_rgb(5, 10, 28);
pub const BACKGROUND_TOP: Color32 = Color32::from_rgb(4, 9, 26);
pub const BACKGROUND_BOTTOM: Color32 = Color32::from_rgb(9, 30, 82);

pub const PRIMARY: Color32 = Color32::from_rgb(37, 99, 235);
pub const PRIMARY_LIGHT: Color32 = Color32::from_rgb(59, 130, 246);
pub const PRIMARY_GLOW: Color32 = Color32::from_rgb(96, 165, 250);
pub const CYAN: Color32 = Color32::from_rgb(34, 211, 238);
pub const INDIGO: Color32 = Color32::from_rgb(79, 70, 229);

pub const SURFACE: Color32 = Color32::from_rgba_premultiplied(12, 20, 42, 214);
pub const FIELD: Color32 = Color32::from_rgba_premultiplied(4, 8, 22, 150);
pub const FIELD_HOVER: Color32 = Color32::from_rgba_premultiplied(10, 18, 40, 170);
pub const BORDER: Color32 = Color32::from_rgba_premultiplied(30, 41, 70, 160);
pub const BORDER_STRONG: Color32 = Color32::from_rgba_premultiplied(51, 65, 105, 200);

pub const TEXT: Color32 = Color32::from_rgb(241, 245, 249);
pub const TEXT_MUTED: Color32 = Color32::from_rgb(148, 163, 184);
pub const TEXT_SUBTLE: Color32 = Color32::from_rgb(100, 116, 139);
pub const ERROR: Color32 = Color32::from_rgb(248, 113, 113);
pub const SUCCESS: Color32 = Color32::from_rgb(74, 222, 128);

pub const RADIUS: u8 = 12;
pub const CONTROL_HEIGHT: f32 = 48.0;

const MEDIUM: &str = "inter-medium";
const SEMIBOLD: &str = "inter-semibold";
const BOLD: &str = "inter-bold";

pub fn apply(ctx: &egui::Context) {
    install_fonts(ctx);
    ctx.set_visuals(egui::Visuals::dark());

    ctx.global_style_mut(|style| {
        style.text_styles = [
            (
                TextStyle::Heading,
                FontId::new(30.0, FontFamily::Name(BOLD.into())),
            ),
            (TextStyle::Body, FontId::proportional(15.0)),
            (
                TextStyle::Button,
                FontId::new(15.0, FontFamily::Name(MEDIUM.into())),
            ),
            (TextStyle::Small, FontId::proportional(13.0)),
            (TextStyle::Monospace, FontId::monospace(14.0)),
        ]
        .into();

        style.animation_time = 0.18;
        style.spacing.item_spacing = vec2(8.0, 12.0);
        style.spacing.button_padding = vec2(16.0, 12.0);

        let visuals = &mut style.visuals;
        visuals.panel_fill = Color32::TRANSPARENT;
        visuals.window_fill = SURFACE;
        visuals.extreme_bg_color = FIELD;
        visuals.override_text_color = Some(TEXT);
        visuals.hyperlink_color = PRIMARY_GLOW;
        visuals.selection.bg_fill = PRIMARY.gamma_multiply(0.6);
        visuals.selection.stroke = Stroke::new(1.5, PRIMARY_LIGHT);
        visuals.text_cursor.stroke = Stroke::new(2.0, PRIMARY_GLOW);

        let radius = CornerRadius::same(RADIUS);
        for widget in [
            &mut visuals.widgets.inactive,
            &mut visuals.widgets.hovered,
            &mut visuals.widgets.active,
            &mut visuals.widgets.open,
            &mut visuals.widgets.noninteractive,
        ] {
            widget.corner_radius = radius;
            widget.expansion = 0.0;
        }

        visuals.widgets.inactive.bg_fill = FIELD;
        visuals.widgets.inactive.weak_bg_fill = FIELD;
        visuals.widgets.inactive.bg_stroke = Stroke::new(1.0, BORDER);
        visuals.widgets.hovered.bg_fill = FIELD_HOVER;
        visuals.widgets.hovered.weak_bg_fill = FIELD_HOVER;
        visuals.widgets.hovered.bg_stroke = Stroke::new(1.0, BORDER_STRONG);
        visuals.widgets.active.bg_fill = FIELD_HOVER;
        visuals.widgets.active.weak_bg_fill = FIELD_HOVER;
        visuals.widgets.active.bg_stroke = Stroke::new(1.5, PRIMARY_LIGHT);
        visuals.widgets.noninteractive.bg_stroke = Stroke::new(1.0, BORDER);
    });
}

fn install_fonts(ctx: &egui::Context) {
    let mut fonts = FontDefinitions::default();
    let faces: [(&str, &'static [u8]); 4] = [
        ("inter", include_bytes!("../assets/fonts/inter-400.ttf")),
        (MEDIUM, include_bytes!("../assets/fonts/inter-500.ttf")),
        (SEMIBOLD, include_bytes!("../assets/fonts/inter-600.ttf")),
        (BOLD, include_bytes!("../assets/fonts/inter-700.ttf")),
    ];

    let fallbacks = fonts
        .families
        .get(&FontFamily::Proportional)
        .cloned()
        .unwrap_or_default();

    for (name, bytes) in faces {
        fonts
            .font_data
            .insert(name.to_owned(), Arc::new(FontData::from_static(bytes)));
    }

    fonts
        .families
        .entry(FontFamily::Proportional)
        .or_default()
        .insert(0, "inter".to_owned());

    for name in [MEDIUM, SEMIBOLD, BOLD] {
        let mut family = vec![name.to_owned()];
        family.extend(fallbacks.iter().cloned());
        fonts.families.insert(FontFamily::Name(name.into()), family);
    }

    ctx.set_fonts(fonts);
}

pub fn medium(size: f32) -> FontId {
    FontId::new(size, FontFamily::Name(MEDIUM.into()))
}

pub fn semibold(size: f32) -> FontId {
    FontId::new(size, FontFamily::Name(SEMIBOLD.into()))
}

pub fn bold(size: f32) -> FontId {
    FontId::new(size, FontFamily::Name(BOLD.into()))
}

pub fn paint_background(ctx: &egui::Context) {
    let rect = ctx.content_rect();
    let painter = ctx.layer_painter(egui::LayerId::background());
    let time = ctx.input(|i| i.time) as f32;

    let mut mesh = Mesh::default();
    for (pos, color) in [
        (rect.left_top(), BACKGROUND_TOP),
        (rect.right_top(), Color32::from_rgb(6, 16, 44)),
        (rect.right_bottom(), BACKGROUND_BOTTOM),
        (rect.left_bottom(), Color32::from_rgb(5, 18, 52)),
    ] {
        mesh.colored_vertex(pos, color);
    }
    mesh.add_triangle(0, 1, 2);
    mesh.add_triangle(0, 2, 3);
    painter.add(mesh);

    let size = rect.size();
    let reach = size.x.max(size.y);
    let orbs = [
        (0.18, 0.22, 0.55, PRIMARY, 0.11, 0.0),
        (0.85, 0.30, 0.45, CYAN, 0.08, 2.1),
        (0.70, 0.90, 0.60, INDIGO, 0.07, 4.2),
        (0.10, 0.95, 0.40, PRIMARY_LIGHT, 0.09, 1.3),
    ];
    for (x, y, radius, color, speed, phase) in orbs {
        let t = time * speed + phase;
        let center = Pos2::new(
            rect.left() + size.x * (x + 0.08 * t.sin()),
            rect.top() + size.y * (y + 0.06 * (t * 1.3).cos()),
        );
        paint_glow(&painter, center, reach * radius, color.gamma_multiply(0.35));
    }

    ctx.request_repaint_after_secs(1.0 / 60.0);
}

pub fn paint_glow(painter: &egui::Painter, center: Pos2, radius: f32, color: Color32) {
    const SEGMENTS: usize = 64;
    const RINGS: usize = 6;

    let mut mesh = Mesh::default();
    mesh.colored_vertex(center, color);
    for ring in 1..=RINGS {
        let f = ring as f32 / RINGS as f32;
        let falloff = (1.0 - f).powf(2.2);
        let ring_color = color.gamma_multiply(falloff);
        for segment in 0..SEGMENTS {
            let angle = segment as f32 / SEGMENTS as f32 * std::f32::consts::TAU;
            mesh.colored_vertex(
                center + vec2(angle.cos(), angle.sin()) * radius * f,
                ring_color,
            );
        }
    }

    for segment in 0..SEGMENTS as u32 {
        let next = (segment + 1) % SEGMENTS as u32;
        mesh.add_triangle(0, 1 + segment, 1 + next);
    }
    for ring in 0..(RINGS - 1) as u32 {
        let inner = 1 + ring * SEGMENTS as u32;
        let outer = inner + SEGMENTS as u32;
        for segment in 0..SEGMENTS as u32 {
            let next = (segment + 1) % SEGMENTS as u32;
            mesh.add_triangle(inner + segment, outer + segment, outer + next);
            mesh.add_triangle(inner + segment, outer + next, inner + next);
        }
    }

    painter.add(mesh);
}
