pub mod launcher;

pub use launcher::hide_dock_requested;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Rgb {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Rgb {
    pub const fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ThemeMode {
    Light,
    Dark,
}

#[derive(Clone, Copy, Debug)]
pub struct TradingPalette {
    pub text: Rgb,
    pub muted: Rgb,
    pub window_fill: Rgb,
    pub panel_fill: Rgb,
    pub plot_bg: Rgb,
    pub faint_bg: Rgb,
    pub code_bg: Rgb,
    pub border: Rgb,
    pub grid: Rgb,
    pub widget_inactive: Rgb,
    pub widget_hovered: Rgb,
    pub widget_active: Rgb,
    pub widget_open: Rgb,
    pub selection_bg: Rgb,
    pub selection_stroke: Rgb,
    pub buy: Rgb,
    pub sell: Rgb,
    pub mid: Rgb,
    pub warning: Rgb,
}

#[derive(Clone, Copy, Debug)]
pub struct TradingTextSizes {
    pub toolbar: f32,
    pub table: f32,
    pub table_header: f32,
    pub heading: f32,
    pub chart_label: f32,
    pub small_chart_title: f32,
    pub status: f32,
}

pub const BUY_GREEN: Rgb = Rgb::new(0, 170, 80);
pub const SELL_RED: Rgb = Rgb::new(220, 45, 65);
pub const MID_BLUE: Rgb = Rgb::new(60, 130, 255);
pub const WARNING_YELLOW: Rgb = Rgb::new(220, 160, 20);

pub const TEXT_SIZES: TradingTextSizes = TradingTextSizes {
    toolbar: 12.0,
    table: 11.0,
    table_header: 11.5,
    heading: 18.0,
    chart_label: 10.0,
    small_chart_title: 13.5,
    status: 11.0,
};

pub const DARK_PALETTE: TradingPalette = TradingPalette {
    text: Rgb::new(215, 215, 215),
    muted: Rgb::new(145, 145, 145),
    window_fill: Rgb::new(0, 0, 0),
    panel_fill: Rgb::new(0, 0, 0),
    plot_bg: Rgb::new(18, 18, 18),
    faint_bg: Rgb::new(10, 10, 10),
    code_bg: Rgb::new(10, 10, 10),
    border: Rgb::new(105, 105, 105),
    grid: Rgb::new(45, 45, 45),
    widget_inactive: Rgb::new(42, 42, 42),
    widget_hovered: Rgb::new(58, 58, 58),
    widget_active: Rgb::new(72, 72, 72),
    widget_open: Rgb::new(48, 48, 48),
    selection_bg: Rgb::new(55, 75, 110),
    selection_stroke: Rgb::new(235, 235, 235),
    buy: BUY_GREEN,
    sell: SELL_RED,
    mid: MID_BLUE,
    warning: WARNING_YELLOW,
};

pub const LIGHT_PALETTE: TradingPalette = TradingPalette {
    text: Rgb::new(25, 25, 25),
    muted: Rgb::new(75, 75, 75),
    window_fill: Rgb::new(248, 248, 248),
    panel_fill: Rgb::new(248, 248, 248),
    plot_bg: Rgb::new(255, 255, 255),
    faint_bg: Rgb::new(238, 238, 238),
    code_bg: Rgb::new(242, 242, 242),
    border: Rgb::new(125, 125, 125),
    grid: Rgb::new(210, 210, 210),
    widget_inactive: Rgb::new(232, 232, 232),
    widget_hovered: Rgb::new(220, 228, 240),
    widget_active: Rgb::new(205, 218, 236),
    widget_open: Rgb::new(225, 225, 225),
    selection_bg: Rgb::new(190, 215, 245),
    selection_stroke: Rgb::new(25, 25, 25),
    buy: Rgb::new(0, 125, 55),
    sell: Rgb::new(190, 40, 60),
    mid: Rgb::new(35, 95, 190),
    warning: Rgb::new(180, 140, 0),
};

pub fn palette(mode: ThemeMode) -> TradingPalette {
    match mode {
        ThemeMode::Dark => DARK_PALETTE,
        ThemeMode::Light => LIGHT_PALETTE,
    }
}

pub fn pct_rgb(value: Option<f64>, mode: ThemeMode) -> Rgb {
    let palette = palette(mode);
    match value {
        Some(v) if v > 0.10 => palette.buy,
        Some(v) if v < -0.10 => palette.sell,
        Some(_) => palette.warning,
        None => palette.muted,
    }
}

pub fn format_pct(value: Option<f64>) -> String {
    value
        .map(|v| format!("{v:+.2}%"))
        .unwrap_or_else(|| "n/a".to_string())
}

pub fn format_eur_compact(amount: f64) -> String {
    if amount >= 1_000_000_000.0 {
        format!("{:.2}B€", amount / 1_000_000_000.0)
    } else if amount >= 1_000_000.0 {
        format!("{:.2}M€", amount / 1_000_000.0)
    } else if amount >= 1_000.0 {
        format!("{:.2}K€", amount / 1_000.0)
    } else {
        format!("{amount:.2}€")
    }
}

pub fn format_volume_compact(volume: f64) -> String {
    if volume >= 1_000_000.0 {
        format!("{:.2}M", volume / 1_000_000.0)
    } else if volume >= 1_000.0 {
        format!("{:.2}K", volume / 1_000.0)
    } else {
        format!("{volume:.3}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pct_formatting_keeps_sign() {
        assert_eq!(format_pct(Some(1.234)), "+1.23%");
        assert_eq!(format_pct(Some(-1.234)), "-1.23%");
        assert_eq!(format_pct(None), "n/a");
    }
}
