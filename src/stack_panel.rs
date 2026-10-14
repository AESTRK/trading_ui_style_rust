//! Panneaux top empilés (toolbar → bandeaux → contenu) — hauteur maîtrisée, pas de bande viewport.

use eframe::egui;

use crate::banner::{feed_status_panel_exact_height, issue_panel_exact_height, prepare_issue_top_panel};
use crate::egui_theme::fill_top_stack_remainder;
use crate::egui_theme::{paint_top_stack_panel_bg, stack_top_panel_frame};

/// Bandeau feed / status (1–2 lignes) sous la toolbar — `exact_height` + fond stack.
pub fn show_stack_strip_panel<R>(
    ctx: &egui::Context,
    panel_id: egui::Id,
    visible_strip_rows: u8,
    draw: impl FnOnce(&mut egui::Ui) -> R,
) -> R {
    let rows = visible_strip_rows.max(1);
    let mut out = None;
    egui::TopBottomPanel::top(panel_id)
        .resizable(false)
        .show_separator_line(false)
        .frame(stack_top_panel_frame(ctx))
        .exact_height(issue_panel_exact_height(rows))
        .show(ctx, |ui| {
            paint_top_stack_panel_bg(ui);
            prepare_issue_top_panel(ui);
            out = Some(draw(ui));
            fill_top_stack_remainder(ui);
        });
    out.expect("stack strip panel")
}

/// Bandeau réseau feed (1 bloc bleu, 2 lignes) — hauteur alignée sur `draw_feed_banner`, pas 2 strips issue.
pub fn show_feed_status_strip_panel<R>(
    ctx: &egui::Context,
    panel_id: egui::Id,
    draw: impl FnOnce(&mut egui::Ui) -> R,
) -> R {
    let mut out = None;
    egui::TopBottomPanel::top(panel_id)
        .resizable(false)
        .show_separator_line(false)
        .frame(stack_top_panel_frame(ctx))
        .exact_height(feed_status_panel_exact_height())
        .show(ctx, |ui| {
            paint_top_stack_panel_bg(ui);
            prepare_issue_top_panel(ui);
            out = Some(draw(ui));
            fill_top_stack_remainder(ui);
        });
    out.expect("feed status strip panel")
}

/// Sous-toolbar compacte (filtres dates, onglets secondaires) — hauteur = contenu.
pub fn show_stack_aux_top_panel<R>(
    ctx: &egui::Context,
    panel_id: egui::Id,
    draw: impl FnOnce(&mut egui::Ui) -> R,
) -> R {
    let mut out = None;
    egui::TopBottomPanel::top(panel_id)
        .resizable(false)
        .show_separator_line(false)
        .frame(stack_top_panel_frame(ctx))
        .show(ctx, |ui| {
            paint_top_stack_panel_bg(ui);
            ui.spacing_mut().item_spacing.y = 4.0;
            out = Some(draw(ui));
        });
    out.expect("stack aux panel")
}
