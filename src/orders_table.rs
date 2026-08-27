//! Grille ordres (Orders / Open Orders) — tri par clic sur les en-têtes, UPDATED en premier.

use egui::{Color32, FontId, Id, Ui};
use trading_models_rust::VenueId;

use crate::data_table::{self, DEFAULT_ROW_HEIGHT};
use crate::TEXT_SIZES;

pub const ORDER_TABLE_COLUMNS: &[&str] = &[
    "updated",
    "source",
    "flag",
    "symbol",
    "order_id",
    "side",
    "type",
    "status",
    "limit_px",
    "stop_px",
    "cur_px",
    "avg_px",
    "min_sell_px",
    "exec_qty",
    "orig_qty",
    "rem_qty",
    "tif",
    "client_id",
];

const ROW_HEIGHT: f32 = DEFAULT_ROW_HEIGHT + 2.0;

#[derive(Clone, Debug)]
pub struct OrderTableSort {
    pub column: String,
    pub reverse: bool,
}

impl Default for OrderTableSort {
    fn default() -> Self {
        Self {
            column: "updated".to_string(),
            reverse: true,
        }
    }
}

#[derive(Clone, Debug)]
pub struct OrderGridCells {
    pub updated: String,
    pub source: String,
    pub flag: String,
    pub symbol: String,
    pub order_id: String,
    pub side: String,
    pub order_type: String,
    pub status: String,
    pub limit_px: String,
    pub stop_px: String,
    pub cur_px: String,
    pub avg_px: String,
    pub min_sell_px: String,
    pub exec_qty: String,
    pub orig_qty: String,
    pub rem_qty: String,
    pub tif: String,
    pub client_id: String,
}

impl OrderGridCells {
    pub fn empty() -> Self {
        Self {
            updated: String::new(),
            source: String::new(),
            flag: String::new(),
            symbol: String::new(),
            order_id: String::new(),
            side: String::new(),
            order_type: String::new(),
            status: String::new(),
            limit_px: String::new(),
            stop_px: String::new(),
            cur_px: String::new(),
            avg_px: String::new(),
            min_sell_px: String::new(),
            exec_qty: String::new(),
            orig_qty: String::new(),
            rem_qty: String::new(),
            tif: String::new(),
            client_id: String::new(),
        }
    }

    pub fn get(&self, key: &str) -> &str {
        match key {
            "updated" => &self.updated,
            "source" => &self.source,
            "flag" => &self.flag,
            "symbol" => &self.symbol,
            "order_id" => &self.order_id,
            "side" => &self.side,
            "type" => &self.order_type,
            "status" => &self.status,
            "limit_px" => &self.limit_px,
            "stop_px" => &self.stop_px,
            "cur_px" => &self.cur_px,
            "avg_px" => &self.avg_px,
            "min_sell_px" => &self.min_sell_px,
            "exec_qty" => &self.exec_qty,
            "orig_qty" => &self.orig_qty,
            "rem_qty" => &self.rem_qty,
            "tif" => &self.tif,
            "client_id" => &self.client_id,
            _ => "",
        }
    }
}

#[derive(Clone, Debug)]
pub struct OrderGridRow {
    pub venue: VenueId,
    pub order_id: String,
    pub updated_ms: i64,
    pub limit_px: f64,
    pub avg_px: f64,
    pub exec_qty: f64,
    pub orig_qty: f64,
    pub rem_qty: f64,
    pub stop_px: f64,
    pub cur_px: f64,
    pub cells: OrderGridCells,
    pub color: Color32,
}

pub fn apply_sort_click(sort: &mut OrderTableSort, key: Option<String>) {
    apply_sort_click_for_columns(sort, key, ORDER_TABLE_COLUMNS);
}

pub fn apply_sort_click_for_columns(
    sort: &mut OrderTableSort,
    key: Option<String>,
    visible_columns: &[&str],
) {
    let Some(key) = key else {
        return;
    };
    if !visible_columns.contains(&key.as_str()) {
        return;
    }
    if sort.column == key {
        sort.reverse = !sort.reverse;
    } else {
        sort.column = key.clone();
        sort.reverse = key == "updated";
    }
}

pub fn sort_rows(rows: &mut [OrderGridRow], sort: &OrderTableSort) {
    rows.sort_by(|a, b| {
        let ord = compare_rows(a, b, &sort.column);
        if sort.reverse {
            ord.reverse()
        } else {
            ord
        }
    });
}

fn compare_rows(a: &OrderGridRow, b: &OrderGridRow, col: &str) -> std::cmp::Ordering {
    use std::cmp::Ordering;
    match col {
        "updated" => a.updated_ms.cmp(&b.updated_ms),
        "limit_px" => f64_cmp(a.limit_px, b.limit_px),
        "stop_px" => f64_cmp(a.stop_px, b.stop_px),
        "cur_px" => f64_cmp(a.cur_px, b.cur_px),
        "avg_px" => f64_cmp(a.avg_px, b.avg_px),
        "exec_qty" => f64_cmp(a.exec_qty, b.exec_qty),
        "orig_qty" => f64_cmp(a.orig_qty, b.orig_qty),
        "rem_qty" => f64_cmp(a.rem_qty, b.rem_qty),
        _ => a.cells.get(col).cmp(b.cells.get(col)),
    }
}

fn f64_cmp(a: f64, b: f64) -> std::cmp::Ordering {
    a.partial_cmp(&b).unwrap_or(std::cmp::Ordering::Equal)
}

pub fn show_sortable_table(
    ui: &mut Ui,
    table_id: Id,
    rows: &[OrderGridRow],
    sort: &mut OrderTableSort,
) {
    show_sortable_table_columns(ui, table_id, rows, sort, ORDER_TABLE_COLUMNS);
}

pub fn show_sortable_table_columns(
    ui: &mut Ui,
    table_id: Id,
    rows: &[OrderGridRow],
    sort: &mut OrderTableSort,
    visible_columns: &[&str],
) {
    show_sortable_table_columns_with_menu(ui, table_id, rows, sort, visible_columns, None::<fn(usize, &OrderGridRow)>);
}

pub fn show_sortable_table_columns_with_menu(
    ui: &mut Ui,
    table_id: Id,
    rows: &[OrderGridRow],
    sort: &mut OrderTableSort,
    visible_columns: &[&str],
    mut on_row_context_menu: Option<impl FnMut(usize, &OrderGridRow)>,
) {
    if visible_columns.is_empty() {
        return;
    }
    if !visible_columns.contains(&sort.column.as_str()) {
        sort.column = visible_columns[0].to_string();
        sort.reverse = sort.column == "updated";
    }
    let columns: Vec<String> = visible_columns.iter().map(|s| s.to_string()).collect();
    let sort_column = sort.column.clone();
    let sort_reverse = sort.reverse;

    let (header_event, ()) = data_table::show_sticky_header_table(
        ui,
        table_id,
        |ui| {
            data_table::draw_header_row(
                ui,
                table_id.with("header"),
                &columns,
                column_width,
                column_label,
                column_hint,
                &sort_column,
                sort_reverse,
                ROW_HEIGHT,
                table_header_font(),
            )
        },
        |ui| {
            for (index, row) in rows.iter().enumerate() {
                let response = data_table::draw_selectable_row(
                    ui,
                    table_id.with(("row", index)),
                    &columns,
                    column_width,
                    ROW_HEIGHT,
                    false,
                    index % 2 == 1,
                    index,
                    |ui, key, cell, _default| {
                        data_table::draw_cell_text(
                            ui,
                            cell,
                            row.cells.get(key),
                            table_font(),
                            row.color,
                        );
                    },
                );
                if let Some(handler) = on_row_context_menu.as_mut() {
                    let row = &rows[index];
                    response.context_menu(|ui| {
                        if ui.button("Détail fees / MIN_SELL…").clicked() {
                            handler(index, row);
                            ui.close_menu();
                        }
                    });
                }
            }
        },
    );
    apply_sort_click_for_columns(sort, header_event.sort_key, visible_columns);
}

fn column_width(key: &str) -> f32 {
    match key {
        "updated" => 132.0,
        "source" => 80.0,
        "flag" => 56.0,
        "symbol" => 88.0,
        "order_id" => 108.0,
        "side" => 44.0,
        "type" => 64.0,
        "status" => 72.0,
        "limit_px" | "stop_px" | "cur_px" | "avg_px" => 88.0,
        "exec_qty" | "orig_qty" | "rem_qty" => 92.0,
        "tif" => 48.0,
        "client_id" => 120.0,
        _ => 72.0,
    }
}

fn column_label(key: &str) -> String {
    match key {
        "updated" => "UPDATED".to_string(),
        "source" => "SOURCE".to_string(),
        "flag" => "FLAG".to_string(),
        "symbol" => "SYMBOL".to_string(),
        "order_id" => "ORDER_ID".to_string(),
        "side" => "SIDE".to_string(),
        "type" => "TYPE".to_string(),
        "status" => "STATUS".to_string(),
        "limit_px" => "LIMIT_PX".to_string(),
        "stop_px" => "STOP_PX".to_string(),
        "cur_px" => "CUR_PX".to_string(),
        "avg_px" => "AVG_PX".to_string(),
        "exec_qty" => "EXEC_QTY".to_string(),
        "orig_qty" => "ORIG_QTY".to_string(),
        "rem_qty" => "REM_QTY".to_string(),
        "tif" => "TIF".to_string(),
        "client_id" => "CLIENT_ID".to_string(),
        other => other.to_uppercase(),
    }
}

fn column_hint(key: &str) -> String {
    format!("Trier par {}", column_label(key))
}

fn table_font() -> FontId {
    FontId::proportional(TEXT_SIZES.table)
}

fn table_header_font() -> FontId {
    FontId::proportional(TEXT_SIZES.table_header)
}
