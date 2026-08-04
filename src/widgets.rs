//! Widgets graphiques peints — pas de glyphes Unicode fragiles (coches, flèches combo).

use egui::{Color32, CursorIcon, Pos2, Rect, Response, Sense, Shape, Stroke, Ui, Vec2};

use crate::egui_theme::color;
use crate::{palette, ThemeMode, BUY_GREEN};

/// Trait de contour pour widgets interactifs (case à cocher, flèche combo).
pub fn widget_stroke(ui: &Ui, width: f32) -> Stroke {
    let mode = if ui.visuals().dark_mode {
        ThemeMode::Dark
    } else {
        ThemeMode::Light
    };
    Stroke::new(width, color(palette(mode).text))
}

/// Petit triangle (tri des colonnes, flèche menu déroulant).
pub fn draw_triangle(ui: &Ui, area: Rect, downward: bool, fill: Color32) {
    let h = 5.0;
    let w = 7.0;
    let cx = area.center().x;
    let cy = area.center().y;
    let points = if downward {
        vec![
            Pos2::new(cx - w * 0.5, cy - h * 0.35),
            Pos2::new(cx, cy + h * 0.45),
            Pos2::new(cx + w * 0.5, cy - h * 0.35),
        ]
    } else {
        vec![
            Pos2::new(cx - w * 0.5, cy + h * 0.35),
            Pos2::new(cx, cy - h * 0.45),
            Pos2::new(cx + w * 0.5, cy + h * 0.35),
        ]
    };
    ui.painter()
        .add(Shape::convex_polygon(points, fill, Stroke::NONE));
}

/// Coche dessinée (ne dépend pas des polices Unicode).
pub fn draw_check_mark(ui: &Ui, rect: Rect, mark_color: Color32) {
    let w = rect.width();
    let h = rect.height();
    let a = Pos2::new(rect.left() + w * 0.18, rect.center().y + h * 0.08);
    let b = Pos2::new(rect.left() + w * 0.42, rect.bottom() - h * 0.22);
    let c = Pos2::new(rect.right() - w * 0.16, rect.top() + h * 0.28);
    ui.painter()
        .add(Shape::line(vec![a, b, c], Stroke::new(1.8_f32, mark_color)));
}

/// Petite croix (état erreur) — pas de glyphe Unicode.
pub fn draw_x_mark(ui: &Ui, rect: Rect, mark_color: Color32) {
    let pad = rect.width().max(rect.height()) * 0.22;
    let stroke = Stroke::new(1.8_f32, mark_color);
    ui.painter().add(Shape::line(
        vec![
            Pos2::new(rect.left() + pad, rect.top() + pad),
            Pos2::new(rect.right() - pad, rect.bottom() - pad),
        ],
        stroke,
    ));
    ui.painter().add(Shape::line(
        vec![
            Pos2::new(rect.right() - pad, rect.top() + pad),
            Pos2::new(rect.left() + pad, rect.bottom() - pad),
        ],
        stroke,
    ));
}

/// Petite flèche droite (état en cours).
pub fn draw_arrow_right(ui: &Ui, area: Rect, fill: Color32) {
    let cx = area.center().x;
    let cy = area.center().y;
    let w = 5.0;
    let h = 7.0;
    let points = vec![
        Pos2::new(cx - w * 0.35, cy - h * 0.5),
        Pos2::new(cx + w * 0.45, cy),
        Pos2::new(cx - w * 0.35, cy + h * 0.5),
    ];
    ui.painter()
        .add(Shape::convex_polygon(points, fill, Stroke::NONE));
}

fn paint_checkbox(ui: &Ui, rect: Rect, checked: bool, enabled: bool) {
    let box_rect = rect.shrink(3.0);
    let stroke_color = if enabled {
        widget_stroke(ui, 1.0).color
    } else {
        ui.visuals().weak_text_color()
    };
    ui.painter().rect(
        box_rect,
        2.0,
        Color32::TRANSPARENT,
        Stroke::new(1.0_f32, stroke_color),
        egui::StrokeKind::Inside,
    );
    if checked {
        let mark_color = if enabled {
            color(BUY_GREEN)
        } else {
            stroke_color
        };
        draw_check_mark(ui, box_rect.shrink(2.0), mark_color);
    }
}

/// Case à cocher peinte (indépendante du thème egui / fg_stroke).
pub fn checkbox(ui: &mut Ui, checked: &mut bool) -> Response {
    let desired = Vec2::new(18.0, 18.0);
    let (rect, mut response) = ui.allocate_exact_size(desired, Sense::click());
    paint_checkbox(ui, rect, *checked, ui.is_enabled());
    if response.clicked() && ui.is_enabled() {
        *checked = !*checked;
        response.mark_changed();
    }
    if ui.is_enabled() {
        response = response.on_hover_cursor(CursorIcon::PointingHand);
    }
    response
}

/// Affichage seul (colonne obligatoire, état figé).
pub fn checkbox_readonly(ui: &mut Ui, checked: bool) -> Response {
    let desired = Vec2::new(18.0, 18.0);
    let (rect, response) = ui.allocate_exact_size(desired, Sense::hover());
    paint_checkbox(ui, rect, checked, false);
    response
}

/// Case à cocher + libellé séparés (labels longs, endpoints TCP, etc.).
pub fn checkbox_row(ui: &mut Ui, checked: &mut bool, label: &str) -> Response {
    ui.horizontal(|ui| {
        let cb = checkbox(ui, checked);
        ui.label(label);
        cb
    })
    .inner
}

/// Bouton de sélection — distinct visuellement d'un état « publié » (vert).
pub fn selection_button(ui: &mut Ui, selected: bool, choose_label: &str) -> Response {
    if selected {
        let mode = if ui.visuals().dark_mode {
            ThemeMode::Dark
        } else {
            ThemeMode::Light
        };
        let mark = color(palette(mode).text);
        ui.horizontal(|ui| {
            let (rect, _) = ui.allocate_exact_size(Vec2::new(14.0, 14.0), Sense::hover());
            draw_arrow_right(ui, rect, mark);
            ui.label(egui::RichText::new("Choisie").small().weak());
        })
        .response
    } else {
        ui.button(choose_label)
    }
}
