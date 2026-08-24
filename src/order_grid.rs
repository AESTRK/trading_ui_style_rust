//! Construction de lignes grille partagée entre `open_orders_rust` et `orders_rust`.

use egui::Color32;
use trading_models_rust::{display_time_in_force, fmt_num, ExecutedOrder, OpenOrder};

use crate::orders_table::{OrderGridCells, OrderGridRow};

pub fn open_order_grid_row(order: &OpenOrder, row_color: Color32, cur_px: Option<f64>) -> OrderGridRow {
    let rem = order.remaining_qty();
    OrderGridRow {
        updated_ms: order.update_time_ms,
        limit_px: order.limit_price,
        avg_px: order.display_price(),
        exec_qty: order.executed_qty,
        orig_qty: order.orig_qty,
        rem_qty: rem,
        stop_px: order.stop_price,
        cur_px: cur_px.unwrap_or(0.0),
        cells: OrderGridCells {
            updated: order.updated_local(),
            source: order.venue.label().to_string(),
            flag: order.flag(),
            symbol: order.symbol.clone(),
            order_id: order.order_id.clone(),
            side: order.side.as_str().to_string(),
            order_type: order.display_order_type(),
            status: order.status.as_str().to_string(),
            limit_px: fmt_num(order.limit_price),
            stop_px: fmt_num_or_dash(order.stop_price),
            cur_px: cur_px.map(fmt_num).unwrap_or_else(|| "-".into()),
            avg_px: fmt_num(order.display_price()),
            exec_qty: fmt_num(order.executed_qty),
            orig_qty: fmt_num(order.orig_qty),
            rem_qty: fmt_num(rem),
            tif: display_time_in_force(&order.time_in_force),
            client_id: order.display_client_id(),
        },
        color: row_color,
    }
}

pub fn executed_order_grid_row(order: &ExecutedOrder, row_color: Color32) -> OrderGridRow {
    OrderGridRow {
        updated_ms: order.update_time_ms,
        limit_px: order.limit_price,
        avg_px: order.display_price(),
        exec_qty: order.executed_qty,
        orig_qty: order.orig_qty,
        rem_qty: (order.orig_qty - order.executed_qty).max(0.0),
        stop_px: order.stop_price,
        cur_px: 0.0,
        cells: OrderGridCells {
            updated: order.updated_local(),
            source: order.venue.label().to_string(),
            flag: order.flag(),
            symbol: order.symbol.clone(),
            order_id: order.order_id.clone(),
            side: order.side.as_str().to_string(),
            order_type: order.display_order_type(),
            status: order.status.as_str().to_string(),
            limit_px: fmt_num(order.limit_price),
            stop_px: fmt_num_or_dash(order.stop_price),
            cur_px: "-".into(),
            avg_px: fmt_num(order.display_price()),
            exec_qty: fmt_num(order.executed_qty),
            orig_qty: fmt_num(order.orig_qty),
            rem_qty: "-".into(),
            tif: display_time_in_force(&order.time_in_force),
            client_id: order.display_client_id(),
        },
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
