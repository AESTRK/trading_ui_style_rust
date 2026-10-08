//! Bandeaux de statut empilés (réseau / alertes) — layout 2 lignes partagé entre apps egui.

use crate::banner_dismiss::{draw_banner_close_button, BannerDismissRegistry};
use crate::config_window::TOOLBAR_CONTROL_HEIGHT;
use crate::widgets::{draw_check_mark, draw_filled_dot, draw_pending_ring, draw_x_mark};
use crate::{Rgb, TEXT_SIZES};
use egui::{self, RichText, Sense, Vec2};
use std::time::Duration;

/// Icône de titre de bandeau — dessinée (pas de glyphe Unicode ■/✓).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BannerTitleMark {
    Error,
    Resolved,
    Warning,
    Neutral,
}

fn infer_banner_title_mark(banner: &FeedBanner, blink_alert: bool) -> BannerTitleMark {
    if banner.bg == BANNER_RESOLVED {
        BannerTitleMark::Resolved
    } else if blink_alert {
        BannerTitleMark::Error
    } else {
        BannerTitleMark::Neutral
    }
}

fn draw_banner_title(ui: &mut egui::Ui, mark: BannerTitleMark, title: &str) {
    let icon = Vec2::splat(14.0);
    let (rect, _) = ui.allocate_exact_size(icon, Sense::hover());
    match mark {
        BannerTitleMark::Error => draw_x_mark(ui, rect, BANNER_TEXT),
        BannerTitleMark::Resolved => draw_check_mark(ui, rect, BANNER_TEXT),
        BannerTitleMark::Warning => draw_filled_dot(ui, rect, BANNER_TEXT),
        BannerTitleMark::Neutral => draw_pending_ring(ui, rect, BANNER_TEXT),
    }
    ui.add_space(4.0);
    ui.label(
        RichText::new(title)
            .color(BANNER_TEXT)
            .size(TEXT_SIZES.toolbar)
            .strong(),
    );
}

/// Titre + détail + actions sur une ligne (layout d’origine — hauteur = contenu).
fn draw_banner_title_detail_row(
    ui: &mut egui::Ui,
    title_mark: BannerTitleMark,
    title: &str,
    detail: &str,
    detail_hover_text: bool,
    trailing: impl FnOnce(&mut egui::Ui),
) {
    ui.horizontal(|ui| {
        draw_banner_title(ui, title_mark, title);
        let detail_label = ui.add(
            egui::Label::new(
                RichText::new(detail)
                    .color(BANNER_TEXT.gamma_multiply(0.92))
                    .size(TEXT_SIZES.status),
            )
            .truncate(),
        );
        if detail_hover_text {
            detail_label.on_hover_text(detail);
        }
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            trailing(ui);
        });
    });
}

pub const BANNER_NETWORK_OK: Rgb = Rgb::new(23, 92, 211);
pub const BANNER_NETWORK_ALERT: Rgb = Rgb::new(217, 45, 32);
pub const BANNER_RESOLVED: Rgb = Rgb::new(28, 140, 72);
pub const BANNER_CARNETS_WARN: Rgb = Rgb::new(220, 145, 0);
pub const BANNER_NEUTRAL: Rgb = Rgb::new(52, 64, 84);
pub const BANNER_TEXT: egui::Color32 = egui::Color32::WHITE;

/// Hauteur ligne bandeau issue — alignée toolbar (`Config Manager`, …).
pub const ISSUE_BANNER_ROW_HEIGHT: f32 = TOOLBAR_CONTROL_HEIGHT;

/// Espace sous la bande colorée (peint en `panel_fill`, pas transparent).
pub const ISSUE_BANNER_TO_TOOLBAR_GAP: f32 = 2.0;

/// Marge interne des bandeaux issue une ligne (erreur / warning / résolu).
pub fn issue_strip_inner_margin() -> egui::Margin {
    egui::Margin::symmetric(10, 3)
}

/// Hauteur peinte d’une bande (contenu + padding).
pub fn issue_banner_strip_outer_height() -> f32 {
    ISSUE_BANNER_ROW_HEIGHT + issue_strip_inner_margin().sum().y
}

/// Hauteur panneau top issue (bandes + interligne + marge toolbar).
pub fn issue_panel_exact_height(visible_strip_count: u8) -> f32 {
    let n = visible_strip_count.max(1) as f32;
    n * issue_banner_strip_outer_height() + (n - 1.0).max(0.0) * 2.0 + ISSUE_BANNER_TO_TOOLBAR_GAP
}

/// Respiration sous les bandes — rectangle peint (un `add_space` seul laisse le viewport noir en mode clair).
pub fn finish_issue_top_panel(ui: &mut egui::Ui) {
    let h = ISSUE_BANNER_TO_TOOLBAR_GAP;
    if h < 0.5 {
        return;
    }
    let w = ui.available_width().max(1.0);
    let (rect, _) = ui.allocate_exact_size(egui::vec2(w, h), egui::Sense::hover());
    let fill = crate::egui_theme::top_stack_fill(ui.ctx());
    ui.painter().rect_filled(rect, 0.0, fill);
}

fn clamp_issue_strip_ui(ui: &mut egui::Ui) {
    ui.set_min_height(ISSUE_BANNER_ROW_HEIGHT);
    ui.set_max_height(ISSUE_BANNER_ROW_HEIGHT);
}

/// Période clignotement erreur / warning (alternance clair / assombri).
pub const BLINK_PERIOD_SEC: f64 = 0.55;

/// Délai jusqu'à la prochaine transition de phase du clignotement.
pub fn duration_until_next_blink_at(time_sec: f64) -> Duration {
    let phase = time_sec / BLINK_PERIOD_SEC;
    let next_edge = (phase.floor() + 1.0) * BLINK_PERIOD_SEC;
    let dt = (next_edge - time_sec).clamp(0.01, BLINK_PERIOD_SEC);
    Duration::from_secs_f64(dt)
}

/// Planifie un repaint au prochain changement visuel du clignotement (~2 Hz, pas 30 Hz).
pub fn request_repaint_for_issue_blink(ctx: &egui::Context) {
    let t = ctx.input(|i| i.time);
    ctx.request_repaint_after(duration_until_next_blink_at(t));
}

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
    title_mark: Option<BannerTitleMark>,
    detail_hover_text: bool,
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
    let compact_strip = metrics.is_empty() && ws_stats.is_none();
    let inner_margin = if compact_strip {
        issue_strip_inner_margin()
    } else {
        egui::Margin::symmetric(10, 5)
    };
    egui::Frame::new()
        .fill(fill)
        .inner_margin(inner_margin)
        .show(ui, |ui| {
            ui.set_max_width(ui.available_width());
            if compact_strip {
                clamp_issue_strip_ui(ui);
            }
            let mark = title_mark.unwrap_or_else(|| infer_banner_title_mark(banner, blink_alert));
            draw_banner_title_detail_row(
                ui,
                mark,
                &banner.title,
                &banner.detail,
                detail_hover_text,
                |ui| {
                if let Some((reg, app_id, kind)) = dismiss {
                    draw_banner_close_button(
                        ui,
                        reg,
                        app_id,
                        kind,
                        &banner.detail,
                        false,
                    );
                    ui.add_space(4.0);
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
            },
            );
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
        Some(BannerTitleMark::Resolved),
        false,
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
                Some(BannerTitleMark::Error),
                false,
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
            .inner_margin(issue_strip_inner_margin())
            .show(ui, |ui| {
                ui.set_max_width(ui.available_width());
                clamp_issue_strip_ui(ui);
                draw_banner_title_detail_row(
                    ui,
                    BannerTitleMark::Warning,
                    warning_title,
                    &detail,
                    false,
                    |ui| {
                        draw_banner_close_button(ui, reg, app_id, "warning", &detail, false);
                    },
                );
            });
    }
}

/// Bandeau secondaire (ex. carnets silencieux, API absente) sous le bandeau réseau.
pub fn draw_secondary_banner(ui: &mut egui::Ui, banner: &FeedBanner, subline: Option<&str>) {
    egui::Frame::new()
        .fill(rgb_color(banner.bg))
        .inner_margin(egui::Margin::symmetric(10, 4))
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.vertical(|ui| {
                ui.horizontal(|ui| {
                    draw_banner_title(ui, BannerTitleMark::Neutral, &banner.title);
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn blink_repaint_targets_phase_edges() {
        let d0 = duration_until_next_blink_at(0.0);
        assert!(d0.as_secs_f64() > 0.0 && d0.as_secs_f64() <= BLINK_PERIOD_SEC);
        let d_mid = duration_until_next_blink_at(0.2);
        assert!((d_mid.as_secs_f64() - 0.35).abs() < 0.02);
        let d_edge = duration_until_next_blink_at(0.55);
        assert!((d_edge.as_secs_f64() - BLINK_PERIOD_SEC).abs() < 0.02);
    }

    #[test]
    fn issue_panel_height_scales_with_strip_count() {
        let one = issue_banner_strip_outer_height() + ISSUE_BANNER_TO_TOOLBAR_GAP;
        assert!((issue_panel_exact_height(1) - one).abs() < 0.01);
        assert!(issue_panel_exact_height(2) > issue_panel_exact_height(1));
    }
}
