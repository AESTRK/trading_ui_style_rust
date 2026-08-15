//! Barre d'outils standard — « Menu config » en haut à gauche (référence `open_orders_rust`).

use eframe::egui;

use crate::config_window::menu_config_button;

/// Panneau top compact : bouton config, séparateur, puis contenu app (statut, actions, onglets).
pub fn show_app_toolbar<R>(
    ctx: &egui::Context,
    open_config: &mut bool,
    add_contents: impl FnOnce(&mut egui::Ui) -> R,
) -> R {
    let mut out = None;
    egui::TopBottomPanel::top("toolbar").show(ctx, |ui| {
        out = Some(
            ui.horizontal_wrapped(|ui| {
                menu_config_button(ui, open_config);
                ui.separator();
                add_contents(ui)
            })
            .inner,
        );
    });
    out.expect("toolbar panel")
}
