//! Barre d'outils standard — bouton Config Manager (deep link app).

use eframe::egui;

use crate::config_window::config_manager_toolbar_button;

/// Panneau top compact : ouvre Config Manager sur cette app, puis contenu app.
pub fn show_app_toolbar<R>(
    ctx: &egui::Context,
    crate_app_id: &str,
    add_contents: impl FnOnce(&mut egui::Ui) -> R,
) -> R {
    let mut out = None;
    egui::TopBottomPanel::top("toolbar").show(ctx, |ui| {
        out = Some(
            ui.horizontal_wrapped(|ui| {
                config_manager_toolbar_button(ui, crate_app_id);
                ui.separator();
                add_contents(ui)
            })
            .inner,
        );
    });
    out.expect("toolbar panel")
}
