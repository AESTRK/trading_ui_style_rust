//! Thème egui partagé — une seule apparence fenêtre (sans contour parasite).

use crate::{palette, Rgb, ThemeMode, TradingPalette};

/// Espacement standard stack (publishers + executor).
pub const ITEM_SPACING: egui::Vec2 = egui::vec2(6.0, 4.0);
pub const BUTTON_PADDING: egui::Vec2 = egui::vec2(8.0, 4.0);
pub const CONTENT_INNER_MARGIN: egui::Margin = egui::Margin::same(4);

pub fn color(rgb: Rgb) -> egui::Color32 {
    egui::Color32::from_rgb(rgb.r, rgb.g, rgb.b)
}

pub fn theme_mode(ctx: &egui::Context) -> ThemeMode {
    match ctx.system_theme().unwrap_or(egui::Theme::Light) {
        egui::Theme::Dark => ThemeMode::Dark,
        egui::Theme::Light => ThemeMode::Light,
    }
}

pub fn theme_mode_ui(ui: &egui::Ui) -> ThemeMode {
    if ui.visuals().dark_mode {
        ThemeMode::Dark
    } else {
        ThemeMode::Light
    }
}

pub fn visuals_from_palette(mode: ThemeMode, trading: TradingPalette) -> egui::Visuals {
    let mut visuals = match mode {
        ThemeMode::Dark => egui::Visuals::dark(),
        ThemeMode::Light => egui::Visuals::light(),
    };
    visuals.override_text_color = Some(color(trading.text));
    visuals.window_fill = color(trading.window_fill);
    visuals.panel_fill = color(trading.panel_fill);
    visuals.faint_bg_color = color(trading.faint_bg);
    visuals.extreme_bg_color = color(trading.plot_bg);
    visuals.code_bg_color = color(trading.code_bg);
    visuals.widgets.noninteractive.bg_fill = color(trading.panel_fill);
    visuals.widgets.inactive.bg_fill = color(trading.widget_inactive);
    visuals.widgets.hovered.bg_fill = color(trading.widget_hovered);
    visuals.widgets.active.bg_fill = color(trading.widget_active);
    visuals.widgets.open.bg_fill = color(trading.widget_open);
    visuals.selection.bg_fill = color(trading.selection_bg);
    visuals.selection.stroke.color = color(trading.selection_stroke);

    let no_bg_stroke = egui::Stroke::NONE;
    let widget_fg = egui::Stroke::new(1.0_f32, color(trading.text));
    visuals.window_stroke = no_bg_stroke;
    visuals.widgets.noninteractive.bg_stroke = no_bg_stroke;
    visuals.widgets.inactive.bg_stroke = no_bg_stroke;
    visuals.widgets.hovered.bg_stroke = no_bg_stroke;
    visuals.widgets.active.bg_stroke = no_bg_stroke;
    visuals.widgets.open.bg_stroke = no_bg_stroke;
    // fg_stroke requis pour coches, flèches ComboBox, etc. — ne pas mettre à NONE.
    visuals.widgets.noninteractive.fg_stroke = widget_fg;
    visuals.widgets.inactive.fg_stroke = widget_fg;
    visuals.widgets.hovered.fg_stroke = widget_fg;
    visuals.widgets.active.fg_stroke = widget_fg;
    visuals.widgets.open.fg_stroke = widget_fg;
    visuals
}

pub fn apply_system_visuals(ctx: &egui::Context) {
    let mode = theme_mode(ctx);
    ctx.set_visuals(visuals_from_palette(mode, palette(mode)));
    let mut style = (*ctx.style()).clone();
    style.spacing.item_spacing = ITEM_SPACING;
    style.spacing.button_padding = BUTTON_PADDING;
    ctx.set_style(style);
}

pub fn central_panel_frame(ctx: &egui::Context) -> egui::Frame {
    let fill = color(palette(theme_mode(ctx)).panel_fill);
    egui::Frame::new()
        .fill(fill)
        .stroke(egui::Stroke::NONE)
        .inner_margin(CONTENT_INNER_MARGIN)
}

/// Panneau principal — pas de contour, fond unifié.
pub fn show_central_panel<R>(
    ctx: &egui::Context,
    add_contents: impl FnOnce(&mut egui::Ui) -> R,
) -> egui::InnerResponse<R> {
    egui::CentralPanel::default()
        .frame(central_panel_frame(ctx))
        .show(ctx, add_contents)
}

/// Encadré interne (table, config) — fond seul, sans bord.
pub fn inset_frame(fill: egui::Color32) -> egui::Frame {
    egui::Frame::new()
        .fill(fill)
        .stroke(egui::Stroke::NONE)
}

pub fn inset_frame_margin(fill: egui::Color32, margin: egui::Margin) -> egui::Frame {
    inset_frame(fill).inner_margin(margin)
}
