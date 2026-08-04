//! Fenêtre de configuration détachable (viewport OS) — pattern emergency_panel / orderbook stack.

use eframe::egui;

use crate::egui_theme;

/// Libellé du bouton toolbar (identique à `orderbook_rust`).
pub const MENU_CONFIG_LABEL: &str = "Menu config";

/// Taille par défaut des fenêtres config.
pub const CONFIG_WINDOW_SIZE: egui::Vec2 = egui::vec2(460.0, 520.0);

/// Titre barre OS : « {app} — Configuration ».
pub fn config_viewport_title(app_title: &str) -> String {
    format!("{app_title} — Configuration")
}

/// Identifiant viewport config pour une app mono-fenêtre.
pub fn config_viewport_id(app_id: &str) -> egui::ViewportId {
    egui::ViewportId::from_hash_of(("alphalagoon_config_viewport", app_id))
}

/// Identifiant viewport config lié à une fenêtre parente (ex. orderbook dupliqué).
pub fn paired_config_viewport_id(app_id: &str, parent: egui::ViewportId) -> egui::ViewportId {
    egui::ViewportId::from_hash_of(("alphalagoon_config_viewport", app_id, parent))
}

/// Bouton toolbar standard. Ouvre la fenêtre de config.
pub fn menu_config_button(ui: &mut egui::Ui, open: &mut bool) -> bool {
    if ui.button(MENU_CONFIG_LABEL).clicked() {
        *open = true;
        true
    } else {
        false
    }
}

/// Affiche une fenêtre OS détachable si `open` est vrai. Met à jour `open` à la fermeture.
pub fn show_config_viewport(
    ctx: &egui::Context,
    app_id: &str,
    app_title: &str,
    open: &mut bool,
    add_contents: impl FnMut(&mut egui::Ui),
) {
    show_config_viewport_with_id(
        ctx,
        config_viewport_id(app_id),
        app_title,
        open,
        add_contents,
    );
}

/// Variante avec identifiant viewport explicite (fenêtres multiples).
pub fn show_config_viewport_with_id(
    ctx: &egui::Context,
    viewport_id: egui::ViewportId,
    app_title: &str,
    open: &mut bool,
    mut add_contents: impl FnMut(&mut egui::Ui),
) {
    if !*open {
        return;
    }
    let title = config_viewport_title(app_title);
    let mut still_open = *open;
    ctx.show_viewport_immediate(
        viewport_id,
        egui::ViewportBuilder::default()
            .with_title(title)
            .with_inner_size(CONFIG_WINDOW_SIZE)
            .with_resizable(true),
        |ctx, _class| {
            egui_theme::apply_system_visuals(ctx);
            egui::CentralPanel::default().show(ctx, |ui| {
                ui.heading("Menu config / diagnostic");
                ui.separator();
                egui::ScrollArea::vertical()
                    .auto_shrink([false, false])
                    .show(ui, |ui| add_contents(ui));
            });
            if ctx.input(|i| i.viewport().close_requested()) {
                still_open = false;
            }
        },
    );
    *open = still_open;
}

/// Grille lecture seule label / valeur (endpoints stack, inject).
pub fn readonly_grid(ui: &mut egui::Ui, id: &str, rows: &[(&str, &str)]) {
    egui::Grid::new(id)
        .num_columns(2)
        .spacing(egui::vec2(12.0, 4.0))
        .show(ui, |ui| {
            for (label, value) in rows {
                ui.label(egui::RichText::new(*label).weak());
                ui.label(*value);
                ui.end_row();
            }
        });
}

/// Pied de page chemins config (référence + runtime).
pub fn config_paths_footer(ui: &mut egui::Ui, reference: &str, runtime: &str) {
    ui.separator();
    ui.label(format!("Référence: {reference}"));
    ui.label(format!("Runtime: {runtime}"));
}
