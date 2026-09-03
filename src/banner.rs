//! Bandeaux de statut empilés (réseau / alertes) — layout 2 lignes partagé entre apps egui.

use crate::banner_dismiss::{draw_banner_close_button, BannerDismissRegistry};
use crate::{Rgb, TEXT_SIZES};
use egui::{self, RichText};

pub const BANNER_NETWORK_OK: Rgb = Rgb::new(23, 92, 211);
pub const BANNER_NETWORK_ALERT: Rgb = Rgb::new(217, 45, 32);
pub const BANNER_RESOLVED: Rgb = Rgb::new(28, 140, 72);
pub const BANNER_CARNETS_WARN: Rgb = Rgb::new(220, 145, 0);
pub const BANNER_NEUTRAL: Rgb = Rgb::new(52, 64, 84);
pub const BANNER_TEXT: egui::Color32 = egui::Color32::WHITE;

const BLINK_PERIOD_SEC: f64 = 0.55;

pub fn rgb_color(rgb: Rgb) -> egui::Color32 {
    egui::Color32::from_rgb(rgb.r, rgb.g, rgb.b)
}

#[derive(Debug, Clone)]
pub struct WsDowntimeStats {
    pub disconnect_count: u64,
    pub cumulative_downtime_sec: f64,
    pub current_downtime_sec: f64,
    pub last_disconnect_ts: String,
}

#[derive(Debug, Clone)]
pub struct FeedBanner {
    pub bg: Rgb,
    pub title: String,
    pub detail: String,
}

#[derive(Debug, Clone)]
pub struct BannerMetric {
    pub label: String,
    pub value: String,
}

#[derive(Debug, Clone)]
pub struct BannerButtonStyle {
    pub label: String,
    pub text_color: egui::Color32,
    pub fill: egui::Color32,
}

pub fn alert_bg(ctx: &egui::Context, base: Rgb) -> egui::Color32 {
    let base_c = rgb_color(base);
    let t = ctx.input(|i| i.time);
    if (t / BLINK_PERIOD_SEC) as i64 % 2 == 0 {
        base_c
    } else {
        base_c.gamma_multiply(0.38)
    }
}

pub fn warning_bg(ctx: &egui::Context, base: Rgb) -> egui::Color32 {
    let base_c = rgb_color(base);
    let t = ctx.input(|i| i.time);
    if (t / BLINK_PERIOD_SEC) as i64 % 2 == 0 {
        base_c
    } else {
        base_c.gamma_multiply(0.55)
    }
}

pub fn draw_ws_downtime_stats(ui: &mut egui::Ui, st: &WsDowntimeStats) {
    let text = BANNER_TEXT.gamma_multiply(0.92);
    let size = TEXT_SIZES.status;
    let last = if st.last_disconnect_ts.is_empty() {
        "—".to_string()
    } else {
        st.last_disconnect_ts.clone()
    };
    ui.label(
        RichText::new(format!("Dernière coupure: {last}"))
            .color(text)
            .size(size),
    );
    ui.label(
        RichText::new(format!(
            "Cumul coupures: {:.1}s",
            st.cumulative_downtime_sec
        ))
        .color(text)
        .size(size),
    );
    ui.label(
        RichText::new(format!(
            "Coupure en cours: {:.1}s",
            st.current_downtime_sec
        ))
        .color(text)
        .size(size),
    );
    ui.label(
        RichText::new(format!("Déconnexions: {}", st.disconnect_count))
            .color(text)
            .size(size),
    );
}

/// Bandeau principal sur 2 lignes. Retourne `true` si le bouton action a été cliqué.
pub fn draw_feed_banner(
    ui: &mut egui::Ui,
    ctx: &egui::Context,
    banner: &FeedBanner,
    metrics: &[BannerMetric],
    ws_stats: Option<&WsDowntimeStats>,
    action: Option<BannerButtonStyle>,
    blink_alert: bool,
    dismiss: Option<(&mut BannerDismissRegistry, &str, &str)>,
) -> bool {
    let is_dismissed = dismiss
        .as_ref()
        .map(|(reg, _, kind)| reg.is_dismissed_content(kind, &banner.detail))
        .unwrap_or(false);
    if is_dismissed {
        return false;
    }
    let mut clicked = false;
    let fill = if blink_alert {
        alert_bg(ctx, banner.bg)
    } else {
        rgb_color(banner.bg)
    };
    egui::Frame::new()
        .fill(fill)
        .inner_margin(egui::Margin::symmetric(10, 5))
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.vertical(|ui| {
                ui.horizontal(|ui| {
                    ui.label(
                        RichText::new(format!("■ {}", banner.title))
                            .color(BANNER_TEXT)
                            .size(TEXT_SIZES.toolbar)
                            .strong(),
                    );
                    ui.add(
                        egui::Label::new(
                            RichText::new(&banner.detail)
                                .color(BANNER_TEXT.gamma_multiply(0.92))
                                .size(TEXT_SIZES.status),
                        )
                        .wrap(),
                    );
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if let Some((reg, app_id, kind)) = dismiss {
                            draw_banner_close_button(
                                ui,
                                reg,
                                app_id,
                                kind,
                                &banner.detail,
                                false,
                            );
                        }
                        if let Some(btn) = action {
                            if ui
                                .add(
                                    egui::Button::new(
                                        RichText::new(&btn.label)
                                            .color(btn.text_color)
                                            .size(TEXT_SIZES.toolbar),
                                    )
                                    .fill(btn.fill),
                                )
                                .clicked()
                            {
                                clicked = true;
                            }
                        }
                    });
                });
                if !metrics.is_empty() || ws_stats.is_some() {
                    ui.horizontal(|ui| {
                        for m in metrics {
                            ui.label(
                                RichText::new(format!("{} {}", m.label, m.value))
                                    .color(BANNER_TEXT)
                                    .size(TEXT_SIZES.status)
                                    .monospace(),
                            );
                        }
                        if let Some(st) = ws_stats {
                            if !metrics.is_empty() {
                                ui.separator();
                            }
                            draw_ws_downtime_stats(ui, st);
                        }
                    });
                }
            });
        });
    clicked
}

/// Bandeau vert fixe (problème résolu, sans clignotement).
pub fn draw_resolved_banner(
    ui: &mut egui::Ui,
    ctx: &egui::Context,
    title: &str,
    detail: &str,
    dismiss: Option<(&mut BannerDismissRegistry, &str)>,
) {
    draw_feed_banner(
        ui,
        ctx,
        &FeedBanner {
            bg: BANNER_RESOLVED,
            title: title.to_string(),
            detail: detail.to_string(),
        },
        &[],
        None,
        None,
        false,
        dismiss.map(|(reg, app_id)| (reg, app_id, "resolved")),
    );
}

/// Bandeaux empilés erreur (rouge clignotant) + avertissement (ambre).
pub fn draw_stacked_issue_banners(
    ui: &mut egui::Ui,
    ctx: &egui::Context,
    app_id: &str,
    error_title: &str,
    errors: &[String],
    warning_title: &str,
    warnings: &[String],
    reg: &mut BannerDismissRegistry,
) {
    if !errors.is_empty() {
        let detail = errors.join(" · ");
        if !reg.is_dismissed_content("error", &detail) {
            draw_feed_banner(
                ui,
                ctx,
                &FeedBanner {
                    bg: BANNER_NETWORK_ALERT,
                    title: error_title.to_string(),
                    detail,
                },
                &[],
                None,
                None,
                true,
                Some((reg, app_id, "error")),
            );
        }
    }
    if !warnings.is_empty() {
        let detail = warnings.join(" · ");
        if reg.is_dismissed_content("warning", &detail) {
            return;
        }
        let fill = warning_bg(ctx, BANNER_CARNETS_WARN);
        egui::Frame::new()
            .fill(fill)
            .inner_margin(egui::Margin::symmetric(10, 5))
            .show(ui, |ui| {
                ui.set_width(ui.available_width());
                ui.horizontal(|ui| {
                    ui.label(
                        RichText::new(format!("■ {}", warning_title))
                            .color(BANNER_TEXT)
                            .size(TEXT_SIZES.toolbar)
                            .strong(),
                    );
                    ui.add(
                        egui::Label::new(
                            RichText::new(&detail)
                                .color(BANNER_TEXT.gamma_multiply(0.92))
                                .size(TEXT_SIZES.status),
                        )
                        .wrap(),
                    );
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        draw_banner_close_button(ui, reg, app_id, "warning", &detail, false);
                    });
                });
            });
    }
}

/// Bandeau secondaire (ex. carnets silencieux, API absente) sous le bandeau réseau.
pub fn draw_secondary_banner(ui: &mut egui::Ui, banner: &FeedBanner, subline: Option<&str>) {
    ui.add_space(2.0);
    egui::Frame::new()
        .fill(rgb_color(banner.bg))
        .inner_margin(egui::Margin::symmetric(10, 4))
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.vertical(|ui| {
                ui.horizontal(|ui| {
                    ui.label(
                        RichText::new(format!("■ {}", banner.title))
                            .color(BANNER_TEXT)
                            .size(TEXT_SIZES.toolbar)
                            .strong(),
                    );
                    ui.add(
                        egui::Label::new(
                            RichText::new(&banner.detail)
                                .color(BANNER_TEXT.gamma_multiply(0.92))
                                .size(TEXT_SIZES.status),
                        )
                        .wrap(),
                    );
                });
                if let Some(sub) = subline {
                    ui.label(
                        RichText::new(sub)
                            .color(BANNER_TEXT.gamma_multiply(0.88))
                            .size(TEXT_SIZES.status),
                    );
                }
            });
        });
}
