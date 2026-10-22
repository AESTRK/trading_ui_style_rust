//! Format qty/prix grille ordres — Statics (`None` → « — », pas d’heuristique legacy).

use reference_client_rust::{
    format_price_for_pair, format_qty_for_pair, StaticsFormatCache,
};
use trading_models_rust::VenueId;

const MISSING: &str = "—";

pub fn grid_qty(
    statics: Option<&StaticsFormatCache>,
    venue: VenueId,
    symbol: &str,
    value: f64,
) -> String {
    statics
        .map(|cache| format_qty_for_pair(cache, venue, symbol, value))
        .unwrap_or_else(|| MISSING.into())
}

pub fn grid_price(
    statics: Option<&StaticsFormatCache>,
    venue: VenueId,
    symbol: &str,
    value: f64,
) -> String {
    statics
        .map(|cache| format_price_for_pair(cache, venue, symbol, value))
        .unwrap_or_else(|| MISSING.into())
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
    statics
        .map(|cache| format_price_for_pair(cache, venue, symbol, value))
        .unwrap_or_else(|| MISSING.into())
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
