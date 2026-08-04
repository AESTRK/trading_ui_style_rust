//! Tableau aligné avec tri par clic sur les en-têtes (flèche asc/desc).
//!
//! Utilisable par toutes les apps GUI AlphaLagoon (statics, publishers, etc.).

use egui::{Color32, FontId, Id, Rect, Response, ScrollArea, Sense, Ui, Vec2};

/// Événements émis par une ligne d'en-tête.
#[derive(Clone, Debug, Default)]
pub struct TableHeaderEvent {
    pub sort_key: Option<String>,
}

pub const DEFAULT_ROW_HEIGHT: f32 = 17.0;
pub const HEADER_BODY_GAP: f32 = 6.0;
const CELL_PADDING_X: f32 = 6.0;
const SORT_INDICATOR_WIDTH: f32 = 10.0;

pub fn column_spacing(ui: &Ui) -> f32 {
    ui.spacing().item_spacing.x.max(12.0)
}

pub fn padded_cell(cell: Rect) -> Rect {
    cell.shrink2(Vec2::new(CELL_PADDING_X, 0.0))
}

pub fn table_content_width(
    columns: &[String],
    column_width: impl Fn(&str) -> f32,
    ui: &Ui,
) -> f32 {
    let spacing = column_spacing(ui);
    columns
        .iter()
        .map(|key| column_width(key))
        .sum::<f32>()
        + spacing * columns.len().saturating_sub(1) as f32
}

pub fn cell_rect(
    row_rect: Rect,
    col_index: usize,
    columns: &[String],
    spacing: f32,
    column_width: impl Fn(&str) -> f32,
) -> Rect {
    let mut x = row_rect.min.x;
    for key in columns.iter().take(col_index) {
        x += column_width(key) + spacing;
    }
    let w = column_width(&columns[col_index]);
    Rect::from_min_size(egui::pos2(x, row_rect.min.y), Vec2::new(w, row_rect.height()))
}

/// Désactive l'espacement vertical entre les lignes du tableau.
pub fn with_zero_row_spacing<R>(ui: &mut Ui, f: impl FnOnce(&mut Ui) -> R) -> R {
    let spacing = ui.spacing().item_spacing;
    ui.spacing_mut().item_spacing.y = 0.0;
    let out = f(ui);
    ui.spacing_mut().item_spacing = spacing;
    out
}

fn draw_sort_triangle(ui: &Ui, area: Rect, ascending: bool, color: Color32) {
    crate::widgets::draw_triangle(ui, area, !ascending, color);
}

fn draw_header_cell_text(
    ui: &Ui,
    cell: Rect,
    text: &str,
    font: FontId,
    color: Color32,
) {
    ui.painter().with_clip_rect(cell).text(
        cell.left_center(),
        egui::Align2::LEFT_CENTER,
        text,
        font,
        color,
    );
}

pub fn draw_cell_text(
    ui: &Ui,
    cell: Rect,
    text: &str,
    font: FontId,
    color: Color32,
) {
    let text_cell = padded_cell(cell);
    ui.painter().with_clip_rect(text_cell).text(
        text_cell.left_center(),
        egui::Align2::LEFT_CENTER,
        text,
        font,
        color,
    );
}

/// Tableau : en-tête fixe au scroll vertical, scroll horizontal partagé avec le corps.
pub fn show_sticky_header_table<R>(
    ui: &mut Ui,
    table_id: Id,
    draw_header: impl FnOnce(&mut Ui) -> TableHeaderEvent,
    draw_body: impl FnOnce(&mut Ui) -> R,
) -> (TableHeaderEvent, R) {
    let mut header_event = TableHeaderEvent::default();
    let mut body_out: Option<R> = None;
    ScrollArea::horizontal()
        .id_salt(table_id.with("h_scroll"))
        .auto_shrink([false, false])
        .drag_to_scroll(false)
        .show(ui, |ui| {
            ui.vertical(|ui| {
                header_event = with_zero_row_spacing(ui, draw_header);
                let body_height = ui.available_height();
                body_out = Some(
                    ScrollArea::vertical()
                        .id_salt(table_id.with("v_scroll"))
                        .auto_shrink([false, false])
                        .drag_to_scroll(false)
                        .max_height(body_height)
                        .show(ui, |ui| with_zero_row_spacing(ui, draw_body))
                        .inner,
                );
            });
        });
    (header_event, body_out.expect("sticky table body"))
}

/// En-tête triable : clic sur une colonne pour trier ; reclic inverse le sens (flèche).
pub fn draw_header_row(
    ui: &mut Ui,
    table_id: Id,
    columns: &[String],
    column_width: impl Fn(&str) -> f32 + Copy,
    column_label: impl Fn(&str) -> String + Copy,
    column_hint: impl Fn(&str) -> String + Copy,
    sort_column: &str,
    sort_reverse: bool,
    row_height: f32,
    header_font: FontId,
) -> TableHeaderEvent {
    let mut event = TableHeaderEvent::default();
    if columns.is_empty() {
        return event;
    }
    let spacing = column_spacing(ui);
    let row_w = table_content_width(columns, column_width, ui);
    let (rect, _) = ui.allocate_exact_size(Vec2::new(row_w, row_height), Sense::hover());

    for (col_index, key) in columns.iter().enumerate() {
        let cell = cell_rect(rect, col_index, columns, spacing, column_width);
        let sort_id = table_id.with(("col_sort", key.as_str()));
        let label = column_label(key);
        let sorted = sort_column == key;
        let text_color = ui.visuals().strong_text_color();

        let inner = cell.shrink2(Vec2::new(CELL_PADDING_X, 0.0));
        let label_rect = Rect::from_min_max(
            inner.min,
            egui::pos2(
                (inner.max.x - SORT_INDICATOR_WIDTH).max(inner.min.x),
                inner.max.y,
            ),
        );
        let sort_rect = Rect::from_min_max(
            egui::pos2(label_rect.max.x, inner.min.y),
            inner.max,
        );

        let sort_response = ui
            .interact(cell, sort_id, Sense::click())
            .on_hover_text(column_hint(key));
        if sort_response.clicked() {
            event.sort_key = Some(key.clone());
        }

        draw_header_cell_text(ui, label_rect, &label, header_font.clone(), text_color);
        if sorted {
            draw_sort_triangle(ui, sort_rect, !sort_reverse, text_color);
        }
    }

    ui.add_space(HEADER_BODY_GAP);
    event
}

/// Ligne sélectionnable ; le contenu des cellules est dessiné via le callback (non interactif).
pub fn draw_selectable_row(
    ui: &mut Ui,
    row_id: Id,
    columns: &[String],
    column_width: impl Fn(&str) -> f32 + Copy,
    row_height: f32,
    selected: bool,
    striped: bool,
    row_index: usize,
    mut draw_cell: impl FnMut(&Ui, &str, Rect, Color32),
) -> Response {
    let spacing = column_spacing(ui);
    let row_w = table_content_width(columns, column_width, ui).max(ui.available_width());
    let (rect, _) = ui.allocate_exact_size(Vec2::new(row_w, row_height), Sense::hover());
    if selected {
        ui.painter()
            .rect_filled(rect, 1.0, ui.visuals().selection.bg_fill);
    } else if striped && row_index % 2 == 1 {
        ui.painter()
            .rect_filled(rect, 0.0, ui.visuals().faint_bg_color);
    }
    let text_color = ui.visuals().text_color();
    for (col_index, key) in columns.iter().enumerate() {
        let cell = cell_rect(rect, col_index, columns, spacing, column_width);
        draw_cell(ui, key, cell, text_color);
    }
    ui.interact(rect, row_id, Sense::click())
}
