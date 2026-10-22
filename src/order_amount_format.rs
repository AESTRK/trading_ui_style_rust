//! Format qty/prix grille ordres — Statics ou fallback legacy.

#![allow(deprecated)]

use reference_client_rust::{
    format_price_for_pair, format_qty_for_pair, StaticsFormatCache,
};
use trading_models_rust::{fmt_num, fmt_quote_notional, VenueId};

pub fn grid_qty(
    statics: Option<&StaticsFormatCache>,
    venue: VenueId,
    symbol: &str,
    value: f64,
) -> String {
    if let Some(cache) = statics {
        return format_qty_for_pair(cache, venue, symbol, value);
    }
    fmt_num(value)
}

pub fn grid_price(
    statics: Option<&StaticsFormatCache>,
    venue: VenueId,
    symbol: &str,
    value: f64,
) -> String {
    if let Some(cache) = statics {
        return format_price_for_pair(cache, venue, symbol, value);
    }
    fmt_num(value)
}

pub fn grid_price_or_dash(
    statics: Option<&StaticsFormatCache>,
    venue: VenueId,
    symbol: &str,
    value: f64,
) -> String {
    if value > 0.0 {
        grid_price(statics, venue, symbol, value)
    } else {
        "-".into()
    }
}

pub fn grid_quote_notional(
    statics: Option<&StaticsFormatCache>,
    venue: VenueId,
    symbol: &str,
    value: f64,
) -> String {
    if value <= 0.0 {
        return "-".into();
    }
    if let Some(cache) = statics {
        return format_price_for_pair(cache, venue, symbol, value);
    }
    fmt_quote_notional(symbol, value)
}

pub fn grid_cur_px(
    statics: Option<&StaticsFormatCache>,
    venue: VenueId,
    symbol: &str,
    value: Option<f64>,
) -> String {
    match value {
        Some(v) => grid_price(statics, venue, symbol, v),
        None => "-".into(),
    }
}
