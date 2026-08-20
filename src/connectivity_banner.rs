//! Bandeaux connectivité / issues empilés — panneau top egui partagé.

use crate::banner::draw_stacked_issue_banners;
use crate::issues::StackIssueBoard;
use crate::{Rgb, TEXT_SIZES};
use egui::{self, FontId, RichText};

const CONN_BANNER_ALERT: Rgb = Rgb::new(217, 45, 32);
const CONN_BLINK_PERIOD_SEC: f64 = 0.55;

fn rgb_color(rgb: Rgb) -> egui::Color32 {
    egui::Color32::from_rgb(rgb.r, rgb.g, rgb.b)
}

fn connectivity_blink_on(ctx: &egui::Context) -> bool {
    let t = ctx.input(|i| i.time);
    (t / CONN_BLINK_PERIOD_SEC) as i64 % 2 == 0
}

fn connectivity_alert_bg(ctx: &egui::Context) -> egui::Color32 {
    if connectivity_blink_on(ctx) {
        rgb_color(CONN_BANNER_ALERT)
    } else {
        egui::Color32::from_rgb(140, 20, 12)
    }
}

/// Bandeaux erreur + avertissement empilés en haut de fenêtre.
pub fn show_top_issue_panel(
    ctx: &egui::Context,
    error_title: &str,
    errors: &[String],
    warning_title: &str,
    warnings: &[String],
) {
    if errors.is_empty() && warnings.is_empty() {
        return;
    }
    egui::TopBottomPanel::top("stack_issue_banner").show(ctx, |ui| {
        draw_stacked_issue_banners(ui, ctx, error_title, errors, warning_title, warnings);
    });
}

/// Bandeau classifié avec titre d'app explicite.
pub fn show_top_classified_panel(ctx: &egui::Context, error_title: &str, text: &str) {
    StackIssueBoard::from_combined_text(text).show_top(ctx, error_title, "AVERTISSEMENT");
}

/// Affiche un bandeau rouge clignotant en haut de la fenêtre si `text` est non vide.
/// Découpe ` · ` et classifie erreur / avertissement (harmonisé stack AlphaLagoon).
pub fn show_top_alert_panel(ctx: &egui::Context, text: &str) {
    show_top_classified_panel(ctx, "ERREUR", text);
}

/// Variante legacy : texte plat sans classification (tout en erreur).
pub fn show_top_flat_error_panel(ctx: &egui::Context, text: &str) {
    if text.trim().is_empty() {
        return;
    }
    egui::TopBottomPanel::top("connectivity_banner").show(ctx, |ui| {
        egui::Frame::new()
            .fill(connectivity_alert_bg(ctx))
            .inner_margin(egui::Margin::symmetric(14, 10))
            .show(ui, |ui| {
                ui.add(
                    egui::Label::new(
                        RichText::new(text)
                            .color(egui::Color32::WHITE)
                            .font(FontId::proportional(TEXT_SIZES.toolbar)),
                    )
                    .wrap(),
                );
            });
    });
}
