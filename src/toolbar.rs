//! Barre d'outils standard — bouton Config Manager (deep link app).

use eframe::egui::{self, FontId, RichText, Stroke};

use crate::config_window::{config_manager_toolbar_button, TOOLBAR_CONTROL_HEIGHT};
use crate::TEXT_SIZES;

/// Hauteur max du panneau toolbar (wrap 2–3 lignes). Sans ça, les `Separator` verticaux
/// prennent toute la hauteur dispo et le panneau « mange » la fenêtre (régression wrap layout).
const TOOLBAR_PANEL_MAX_HEIGHT: f32 = 96.0;

/// Séparateur vertical compact — ne pas utiliser `ui.separator()` dans la toolbar horizontale.
pub fn toolbar_separator(ui: &mut egui::Ui) {
    let h = TOOLBAR_CONTROL_HEIGHT;
    let (rect, _) = ui.allocate_exact_size(egui::vec2(6.0, h), egui::Sense::hover());
    let stroke = ui.visuals().widgets.noninteractive.bg_stroke;
    ui.painter().vline(
        rect.center().x,
        rect.top()..=rect.bottom(),
        stroke,
    );
}

/// Panneau top compact : ouvre Config Manager sur cette app, puis contenu app.
pub fn show_app_toolbar<R>(
    ctx: &egui::Context,
    crate_app_id: &str,
    add_contents: impl FnOnce(&mut egui::Ui) -> R,
) -> R {
    let mut out = None;
    let panel_id = egui::Id::new(("app_toolbar_v2", crate_app_id));
    egui::TopBottomPanel::top(panel_id)
        .resizable(false)
        .max_height(TOOLBAR_PANEL_MAX_HEIGHT)
        .show(ctx, |ui| {
            ui.set_max_height(TOOLBAR_PANEL_MAX_HEIGHT);
            out = Some(
                ui.horizontal_wrapped(|ui| {
                    ui.spacing_mut().item_spacing = egui::vec2(8.0, 4.0);
                    config_manager_toolbar_button(ui, crate_app_id);
                    toolbar_separator(ui);
                    add_contents(ui)
                })
                .inner,
            );
        });
    out.expect("toolbar panel")
}

/// Libellé toolbar (ex. « Venue IPC ») — même hauteur que le bouton Config Manager.
pub fn toolbar_row_caption(ui: &mut egui::Ui, text: impl Into<RichText>) {
    let text = text.into();
    ui.allocate_ui_with_layout(
        egui::vec2(0.0, TOOLBAR_CONTROL_HEIGHT),
        egui::Layout::left_to_right(egui::Align::Center),
        |ui| {
            ui.label(text);
        },
    );
}

/// Champ lecture seule — même widget / hauteur que `config_manager_toolbar_button`.
pub fn toolbar_readonly_chip(
    ui: &mut egui::Ui,
    width: f32,
    label: RichText,
    fill: egui::Color32,
    stroke: egui::Color32,
) -> egui::Response {
    ui.add_enabled(
        false,
        egui::Button::new(label)
            .fill(fill)
            .stroke(Stroke::new(1.0, stroke))
            .corner_radius(4.0)
            .min_size(egui::vec2(width, TOOLBAR_CONTROL_HEIGHT)),
    )
}

/// Statut toolbar (point + texte) dans la hauteur standard.
pub fn toolbar_row_status(
    ui: &mut egui::Ui,
    dot_color: egui::Color32,
    text: RichText,
    draw_dot: impl FnOnce(&mut egui::Ui, egui::Rect, egui::Color32),
) {
    ui.allocate_ui_with_layout(
        egui::vec2(0.0, TOOLBAR_CONTROL_HEIGHT),
        egui::Layout::left_to_right(egui::Align::Center),
        |ui| {
            ui.spacing_mut().item_spacing.x = 4.0;
            let (rect, _) = ui.allocate_exact_size(egui::vec2(8.0, 8.0), egui::Sense::hover());
            draw_dot(ui, rect, dot_color);
            ui.label(text);
        },
    );
}

/// Police toolbar standard (12 px proportional).
pub fn toolbar_font() -> FontId {
    FontId::proportional(TEXT_SIZES.toolbar)
}
