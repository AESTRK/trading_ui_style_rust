//! Construction de lignes grille partagée pour `orders_rust`.

use egui::Color32;
use trading_models_rust::{display_time_in_force, fmt_num, ExecutedOrder, OpenOrder};

use crate::orders_table::{OrderGridCells, OrderGridRow, ORDER_TABLE_COLUMNS};

/// Colonne à formater : visible à l'écran ou colonne de tri active.
pub fn column_needed(visible: &[&str], sort_column: &str, key: &str) -> bool {
    visible.iter().any(|c| *c == key) || sort_column == key
}

pub fn open_order_grid_row(
    order: &OpenOrder,
    row_color: Color32,
    cur_px: Option<f64>,
    min_sell_px: Option<f64>,
) -> OrderGridRow {
    open_order_grid_row_for_columns(
        order,
        row_color,
        ORDER_TABLE_COLUMNS,
        "updated",
        cur_px,
        min_sell_px,
    )
}

pub fn open_order_grid_row_for_columns(
    order: &OpenOrder,
    row_color: Color32,
    visible: &[&str],
    sort_column: &str,
    cur_px: Option<f64>,
    min_sell_px: Option<f64>,
) -> OrderGridRow {
    let rem = order.remaining_qty();
    let mut cells = OrderGridCells::empty();
    if column_needed(visible, sort_column, "updated") {
        cells.updated = order.updated_local();
    }
    if column_needed(visible, sort_column, "source") {
        cells.source = order.venue.display_label().to_string();
    }
    if column_needed(visible, sort_column, "flag") {
        cells.flag = order.flag();
    }
    if column_needed(visible, sort_column, "symbol") {
        cells.symbol = order.symbol.clone();
    }
    if column_needed(visible, sort_column, "order_id") {
        cells.order_id = order.order_id.clone();
    }
    if column_needed(visible, sort_column, "side") {
        cells.side = order.side.as_str().to_string();
    }
    if column_needed(visible, sort_column, "type") {
        cells.order_type = order.display_order_type();
    }
    if column_needed(visible, sort_column, "status") {
        cells.status = order.status.as_str().to_string();
    }
    if column_needed(visible, sort_column, "limit_px") {
        cells.limit_px = fmt_num(order.limit_price);
    }
    if column_needed(visible, sort_column, "stop_px") {
        cells.stop_px = fmt_num_or_dash(order.stop_price);
    }
    if column_needed(visible, sort_column, "cur_px") {
        cells.cur_px = cur_px.map(fmt_num).unwrap_or_else(|| "-".into());
    }
    if column_needed(visible, sort_column, "avg_px") {
        cells.avg_px = fmt_num(order.display_price());
    }
    if column_needed(visible, sort_column, "min_sell_px") {
        cells.min_sell_px = min_sell_px.map(fmt_num).unwrap_or_else(|| "-".into());
    }
    if column_needed(visible, sort_column, "exec_qty") {
        cells.exec_qty = fmt_num(order.executed_qty);
    }
    if column_needed(visible, sort_column, "orig_qty") {
        cells.orig_qty = fmt_num(order.orig_qty);
    }
    if column_needed(visible, sort_column, "rem_qty") {
        cells.rem_qty = fmt_num(rem);
    }
    if column_needed(visible, sort_column, "tif") {
        cells.tif = display_time_in_force(&order.time_in_force);
    }
    if column_needed(visible, sort_column, "client_id") {
        cells.client_id = order.display_client_id();
    }

    OrderGridRow {
        updated_ms: order.update_time_ms,
        limit_px: order.limit_price,
        avg_px: order.display_price(),
        exec_qty: order.executed_qty,
        orig_qty: order.orig_qty,
        rem_qty: rem,
        stop_px: order.stop_price,
        cur_px: cur_px.unwrap_or(0.0),
        cells,
        color: row_color,
    }
}

pub fn executed_order_grid_row(
    order: &ExecutedOrder,
    row_color: Color32,
    cur_px: Option<f64>,
    min_sell_px: Option<f64>,
) -> OrderGridRow {
    executed_order_grid_row_for_columns(
        order,
        row_color,
        ORDER_TABLE_COLUMNS,
        "updated",
        cur_px,
        min_sell_px,
    )
}

pub fn executed_order_grid_row_for_columns(
    order: &ExecutedOrder,
    row_color: Color32,
    visible: &[&str],
    sort_column: &str,
    cur_px: Option<f64>,
    min_sell_px: Option<f64>,
) -> OrderGridRow {
    let rem = (order.orig_qty - order.executed_qty).max(0.0);
    let min_sell = min_sell_px.or((order.min_sell_px > 0.0).then_some(order.min_sell_px));
    let mut cells = OrderGridCells::empty();
    if column_needed(visible, sort_column, "updated") {
        cells.updated = order.updated_local();
    }
    if column_needed(visible, sort_column, "source") {
        cells.source = order.venue.display_label().to_string();
    }
    if column_needed(visible, sort_column, "flag") {
        cells.flag = order.flag();
    }
    if column_needed(visible, sort_column, "symbol") {
        cells.symbol = order.symbol.clone();
    }
    if column_needed(visible, sort_column, "order_id") {
        cells.order_id = order.order_id.clone();
    }
    if column_needed(visible, sort_column, "side") {
        cells.side = order.side.as_str().to_string();
    }
    if column_needed(visible, sort_column, "type") {
        cells.order_type = order.display_order_type();
    }
    if column_needed(visible, sort_column, "status") {
        cells.status = order.status.as_str().to_string();
    }
    if column_needed(visible, sort_column, "limit_px") {
        cells.limit_px = fmt_num(order.limit_price);
    }
    if column_needed(visible, sort_column, "stop_px") {
        cells.stop_px = fmt_num_or_dash(order.stop_price);
    }
    if column_needed(visible, sort_column, "cur_px") {
        cells.cur_px = cur_px.map(fmt_num).unwrap_or_else(|| "-".into());
    }
    if column_needed(visible, sort_column, "avg_px") {
        cells.avg_px = fmt_num(order.display_price());
    }
    if column_needed(visible, sort_column, "min_sell_px") {
        cells.min_sell_px = min_sell
            .map(fmt_num)
            .unwrap_or_else(|| "-".into());
    }
    if column_needed(visible, sort_column, "exec_qty") {
        cells.exec_qty = fmt_num(order.executed_qty);
    }
    if column_needed(visible, sort_column, "orig_qty") {
        cells.orig_qty = fmt_num(order.orig_qty);
    }
    if column_needed(visible, sort_column, "rem_qty") {
        cells.rem_qty = "-".into();
    }
    if column_needed(visible, sort_column, "tif") {
        cells.tif = display_time_in_force(&order.time_in_force);
    }
    if column_needed(visible, sort_column, "client_id") {
        cells.client_id = order.display_client_id();
    }

    OrderGridRow {
        updated_ms: order.update_time_ms,
        limit_px: order.limit_price,
        avg_px: order.display_price(),
        exec_qty: order.executed_qty,
        orig_qty: order.orig_qty,
        rem_qty: rem,
        stop_px: order.stop_price,
        cur_px: cur_px.unwrap_or(0.0),
        cells,
        color: row_color,
    }
}

fn fmt_num_or_dash(v: f64) -> String {
    if v > 0.0 {
        fmt_num(v)
    } else {
        "-".into()
    }
}
