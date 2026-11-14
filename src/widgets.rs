use std::f32::consts::{PI, TAU};

use eframe::egui::{
    self, Color32, CornerRadius, CursorIcon, Pos2, Rect, RichText, Sense, Shadow, Stroke,
    StrokeKind, Vec2, epaint::Mesh, epaint::PathShape, pos2, vec2,
};

use crate::theme;

pub enum Icon {
    Google,
    Apple,
}

pub fn mix(a: Color32, b: Color32, t: f32) -> Color32 {
    let t = t.clamp(0.0, 1.0);
    let channel = |x: u8, y: u8| (x as f32 + (y as f32 - x as f32) * t).round() as u8;
    Color32::from_rgba_premultiplied(
        channel(a.r(), b.r()),
        channel(a.g(), b.g()),
        channel(a.b(), b.b()),
        channel(a.a(), b.a()),
    )
}

pub fn card<R>(ui: &mut egui::Ui, width: f32, add_contents: impl FnOnce(&mut egui::Ui) -> R) -> R {
    egui::Frame::new()
        .fill(theme::SURFACE)
        .stroke(Stroke::new(1.0, theme::BORDER))
        .corner_radius(CornerRadius::same(22))
        .inner_margin(36)
        .shadow(Shadow {
            offset: [0, 24],
            blur: 64,
            spread: 0,
            color: Color32::from_black_alpha(130),
        })
        .show(ui, |ui| {
            ui.set_width(width - 72.0);
            ui.with_layout(egui::Layout::top_down(egui::Align::Min), add_contents)
                .inner
        })
        .inner
}

pub fn primary_button(ui: &mut egui::Ui, text: &str, width: f32) -> egui::Response {
    let (rect, response) =
        ui.allocate_exact_size(vec2(width, theme::CONTROL_HEIGHT), Sense::click());
    let response = response.on_hover_cursor(CursorIcon::PointingHand);

    if ui.is_rect_visible(rect) {
        let ctx = ui.ctx();
        let hover = ctx.animate_bool_with_time(response.id.with("hover"), response.hovered(), 0.2);
        let pressed = ctx.animate_bool_with_time(
            response.id.with("press"),
            response.is_pointer_button_down_on(),
            0.08,
        );
        let rect = rect.shrink(pressed * 1.5);
        let painter = ui.painter();

        painter.add(
            Shadow {
                offset: [0, 10],
                blur: (18.0 + 16.0 * hover) as u8,
                spread: 0,
                color: theme::PRIMARY.gamma_multiply(0.35 + 0.25 * hover),
            }
            .as_shape(rect, CornerRadius::same(theme::RADIUS)),
        );
        paint_gradient_rect(
            painter,
            rect,
            theme::RADIUS as f32,
            mix(theme::PRIMARY, theme::PRIMARY_LIGHT, hover * 0.6),
            mix(theme::PRIMARY_LIGHT, theme::PRIMARY_GLOW, hover * 0.6),
        );
        painter.rect_stroke(
            rect,
            CornerRadius::same(theme::RADIUS),
            Stroke::new(1.0, Color32::from_white_alpha((24.0 + 24.0 * hover) as u8)),
            StrokeKind::Inside,
        );

        let galley = painter.layout_no_wrap(text.to_owned(), theme::semibold(15.0), Color32::WHITE);
        painter.galley(rect.center() - galley.size() / 2.0, galley, Color32::WHITE);
    }

    response
}

pub fn secondary_button(
    ui: &mut egui::Ui,
    text: &str,
    width: f32,
    icon: Option<Icon>,
) -> egui::Response {
    let (rect, response) =
        ui.allocate_exact_size(vec2(width, theme::CONTROL_HEIGHT), Sense::click());
    let response = response.on_hover_cursor(CursorIcon::PointingHand);

    if ui.is_rect_visible(rect) {
        let hover =
            ui.ctx()
                .animate_bool_with_time(response.id.with("hover"), response.hovered(), 0.2);
        let painter = ui.painter();

        painter.rect(
            rect,
            CornerRadius::same(theme::RADIUS),
            mix(theme::FIELD, theme::FIELD_HOVER, hover),
            Stroke::new(1.0, mix(theme::BORDER, theme::PRIMARY_LIGHT, hover * 0.8)),
            StrokeKind::Inside,
        );

        let galley = painter.layout_no_wrap(text.to_owned(), theme::medium(15.0), theme::TEXT);
        let icon_size = 18.0;
        let gap = if icon.is_some() { 10.0 } else { 0.0 };
        let icon_width = if icon.is_some() { icon_size } else { 0.0 };
        let total = icon_width + gap + galley.size().x;
        let left = rect.center().x - total / 2.0;

        if let Some(icon) = icon {
            let center = pos2(left + icon_size / 2.0, rect.center().y);
            match icon {
                Icon::Google => paint_google(painter, center, icon_size),
                Icon::Apple => paint_apple(painter, center, icon_size, theme::TEXT),
            }
        }

        painter.galley(
            pos2(
                left + icon_width + gap,
                rect.center().y - galley.size().y / 2.0,
            ),
            galley,
            theme::TEXT,
        );
    }

    response
}

pub fn text_field(ui: &mut egui::Ui, value: &mut String, hint: &str, password: bool) -> bool {
    let response = ui.add(
        egui::TextEdit::singleline(value)
            .hint_text(RichText::new(hint).color(theme::TEXT_SUBTLE))
            .password(password)
            .desired_width(f32::INFINITY)
            .margin(vec2(16.0, 14.0)),
    );

    let focus =
        ui.ctx()
            .animate_bool_with_time(response.id.with("focus"), response.has_focus(), 0.2);
    if focus > 0.0 {
        ui.painter().rect_stroke(
            response.rect.expand(4.0),
            CornerRadius::same(theme::RADIUS + 4),
            Stroke::new(4.0, theme::PRIMARY_LIGHT.gamma_multiply(0.28 * focus)),
            StrokeKind::Inside,
        );
    }

    response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter))
}

pub fn segmented(
    ui: &mut egui::Ui,
    id: egui::Id,
    labels: &[&str],
    selected: usize,
) -> Option<usize> {
    let height = 44.0;
    let (rect, _) = ui.allocate_exact_size(vec2(ui.available_width(), height), Sense::hover());
    let painter = ui.painter().clone();

    painter.rect(
        rect,
        CornerRadius::same(theme::RADIUS),
        theme::FIELD,
        Stroke::new(1.0, theme::BORDER),
        StrokeKind::Inside,
    );

    let inset = 4.0;
    let segment_width = (rect.width() - inset * 2.0) / labels.len() as f32;
    let position = ui.ctx().animate_value_with_time(id, selected as f32, 0.28);

    let pill = Rect::from_min_size(
        pos2(
            rect.left() + inset + position * segment_width,
            rect.top() + inset,
        ),
        vec2(segment_width, height - inset * 2.0),
    );
    painter.add(
        Shadow {
            offset: [0, 4],
            blur: 14,
            spread: 0,
            color: theme::PRIMARY.gamma_multiply(0.45),
        }
        .as_shape(pill, CornerRadius::same(theme::RADIUS - 3)),
    );
    paint_gradient_rect(
        &painter,
        pill,
        (theme::RADIUS - 3) as f32,
        theme::PRIMARY,
        theme::PRIMARY_LIGHT,
    );

    let mut clicked = None;
    for (index, label) in labels.iter().enumerate() {
        let segment = Rect::from_min_size(
            pos2(
                rect.left() + inset + index as f32 * segment_width,
                rect.top() + inset,
            ),
            vec2(segment_width, height - inset * 2.0),
        );
        let response = ui
            .interact(segment, id.with(index), Sense::click())
            .on_hover_cursor(CursorIcon::PointingHand);
        if response.clicked() && index != selected {
            clicked = Some(index);
        }

        let closeness = 1.0 - (position - index as f32).abs().min(1.0);
        let color = mix(theme::TEXT_MUTED, Color32::WHITE, closeness);
        let galley = painter.layout_no_wrap((*label).to_owned(), theme::medium(14.0), color);
        painter.galley(segment.center() - galley.size() / 2.0, galley, color);
    }

    clicked
}

pub fn divider(ui: &mut egui::Ui, text: &str) {
    let galley =
        ui.painter()
            .layout_no_wrap(text.to_owned(), theme::medium(12.5), theme::TEXT_SUBTLE);
    let (rect, _) =
        ui.allocate_exact_size(vec2(ui.available_width(), galley.size().y), Sense::hover());
    let line = ((rect.width() - galley.size().x) / 2.0 - 14.0).max(0.0);
    let stroke = Stroke::new(1.0, theme::BORDER_STRONG);
    let painter = ui.painter();
    painter.hline(rect.left()..=rect.left() + line, rect.center().y, stroke);
    painter.hline(rect.right() - line..=rect.right(), rect.center().y, stroke);
    painter.galley(
        pos2(rect.center().x - galley.size().x / 2.0, rect.top()),
        galley,
        theme::TEXT_SUBTLE,
    );
}

pub fn notice(ui: &mut egui::Ui, text: &str, color: Color32) {
    egui::Frame::new()
        .fill(color.gamma_multiply(0.1))
        .stroke(Stroke::new(1.0, color.gamma_multiply(0.35)))
        .corner_radius(CornerRadius::same(10))
        .inner_margin(vec2(14.0, 10.0))
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.label(RichText::new(text).color(color).size(14.0));
        });
}

pub fn logo(ui: &mut egui::Ui, size: f32) {
    let (rect, _) = ui.allocate_exact_size(Vec2::splat(size), Sense::hover());
    let painter = ui.painter();
    let time = ui.input(|i| i.time) as f32;
    let pulse = 0.5 + 0.5 * (time * 1.6).sin();

    theme::paint_glow(
        painter,
        rect.center(),
        size * (1.15 + 0.12 * pulse),
        theme::PRIMARY_LIGHT.gamma_multiply(0.22 + 0.12 * pulse),
    );

    let radius = size * 0.28;
    paint_gradient_rect(painter, rect, radius, theme::PRIMARY, theme::CYAN);
    painter.rect_stroke(
        rect,
        CornerRadius::same(radius as u8),
        Stroke::new(1.0, Color32::from_white_alpha(40)),
        StrokeKind::Inside,
    );

    let stroke = Stroke::new(size * 0.085, Color32::WHITE);
    let inner = rect.shrink(size * 0.27);
    let points = vec![
        inner.left_bottom(),
        inner.left_top(),
        pos2(inner.center().x, inner.top() + inner.height() * 0.55),
        inner.right_top(),
        inner.right_bottom(),
    ];
    painter.add(PathShape::line(points, stroke));
}

pub fn paint_gradient_rect(
    painter: &egui::Painter,
    rect: Rect,
    radius: f32,
    left: Color32,
    right: Color32,
) {
    let points = rounded_rect_points(rect, radius);
    let color_at = |x: f32| mix(left, right, (x - rect.left()) / rect.width().max(1.0));

    let mut mesh = Mesh::default();
    mesh.colored_vertex(rect.center(), color_at(rect.center().x));
    for point in &points {
        mesh.colored_vertex(*point, color_at(point.x));
    }
    let count = points.len() as u32;
    for index in 0..count {
        mesh.add_triangle(0, 1 + index, 1 + (index + 1) % count);
    }
    painter.add(mesh);
}

fn rounded_rect_points(rect: Rect, radius: f32) -> Vec<Pos2> {
    let radius = radius.min(rect.width() / 2.0).min(rect.height() / 2.0);
    let steps = 10;
    let corners = [
        (pos2(rect.left() + radius, rect.top() + radius), PI),
        (pos2(rect.right() - radius, rect.top() + radius), 1.5 * PI),
        (pos2(rect.right() - radius, rect.bottom() - radius), 0.0),
        (pos2(rect.left() + radius, rect.bottom() - radius), 0.5 * PI),
    ];

    let mut points = Vec::with_capacity(corners.len() * (steps + 1));
    for (center, start) in corners {
        for step in 0..=steps {
            let angle = start + step as f32 / steps as f32 * PI / 2.0;
            points.push(center + vec2(angle.cos(), angle.sin()) * radius);
        }
    }
    points
}

fn arc(center: Pos2, radius: f32, from_degrees: f32, to_degrees: f32) -> Vec<Pos2> {
    let steps = 16;
    (0..=steps)
        .map(|step| {
            let degrees = from_degrees + (to_degrees - from_degrees) * step as f32 / steps as f32;
            let angle = degrees.to_radians();
            center + vec2(angle.cos(), angle.sin()) * radius
        })
        .collect()
}

fn paint_google(painter: &egui::Painter, center: Pos2, size: f32) {
    let radius = size * 0.38;
    let width = size * 0.2;
    let segments = [
        (215.0, 320.0, Color32::from_rgb(234, 67, 53)),
        (145.0, 215.0, Color32::from_rgb(251, 188, 5)),
        (40.0, 145.0, Color32::from_rgb(52, 168, 83)),
        (0.0, 40.0, Color32::from_rgb(66, 133, 244)),
    ];
    for (from, to, color) in segments {
        painter.add(PathShape::line(
            arc(center, radius, from, to),
            Stroke::new(width, color),
        ));
    }
    painter.line_segment(
        [
            center + vec2(radius * 0.05, 0.0),
            center + vec2(radius + width / 2.0, 0.0),
        ],
        Stroke::new(width, Color32::from_rgb(66, 133, 244)),
    );
}

fn paint_apple(painter: &egui::Painter, center: Pos2, size: f32, color: Color32) {
    let body_center = center + vec2(0.0, size * 0.08);
    let radius = size * 0.42;
    let steps = 96;
    let bump = |angle: f32, at: f32, width: f32| {
        let mut delta = (angle - at).rem_euclid(TAU);
        if delta > PI {
            delta -= TAU;
        }
        (-(delta / width).powi(2)).exp()
    };

    let points: Vec<Pos2> = (0..steps)
        .map(|step| {
            let angle = step as f32 / steps as f32 * TAU;
            let r = radius
                * (1.0
                    - 0.22 * bump(angle, 1.5 * PI, 0.32)
                    - 0.30 * bump(angle, -0.2, 0.36)
                    - 0.10 * bump(angle, 0.5 * PI, 0.28)
                    + 0.04 * bump(angle, 0.75 * PI, 0.5));
            body_center + vec2(angle.cos() * r * 0.92, angle.sin() * r)
        })
        .collect();

    let mut mesh = Mesh::default();
    mesh.colored_vertex(body_center, color);
    for point in &points {
        mesh.colored_vertex(*point, color);
    }
    for index in 0..steps as u32 {
        mesh.add_triangle(0, 1 + index, 1 + (index + 1) % steps as u32);
    }
    painter.add(mesh);
    painter.add(PathShape::closed_line(points, Stroke::new(0.8, color)));

    let leaf_center = center + vec2(size * 0.08, -size * 0.42);
    let leaf: Vec<Pos2> = (0..24)
        .map(|step| {
            let angle = step as f32 / 24.0 * TAU;
            let local = vec2(angle.cos() * size * 0.07, angle.sin() * size * 0.15);
            let rotation = 0.7_f32;
            leaf_center
                + vec2(
                    local.x * rotation.cos() - local.y * rotation.sin(),
                    local.x * rotation.sin() + local.y * rotation.cos(),
                )
        })
        .collect();
    painter.add(PathShape::convex_polygon(leaf, color, Stroke::NONE));
}
