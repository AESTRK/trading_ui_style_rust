//! Bandeau connectivité clignotant (hub ZMQ) — panneau top egui partagé.

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

/// Affiche un bandeau rouge clignotant en haut de la fenêtre si `text` est non vide.
pub fn show_top_alert_panel(ctx: &egui::Context, text: &str) {
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
