//! Fenêtre de configuration détachable (viewport OS) — pattern emergency_panel / orderbook stack.

use eframe::egui;

use crate::egui_theme;

/// Libellé du bouton toolbar (identique à `orderbook_rust`).
pub const MENU_CONFIG_LABEL: &str = "Menu config";

/// Taille par défaut des fenêtres config.
pub const CONFIG_WINDOW_SIZE: egui::Vec2 = egui::vec2(460.0, 520.0);

/// Fenêtre config plus haute (sélection de paires, listes longues).
pub const CONFIG_WINDOW_SIZE_TALL: egui::Vec2 = egui::vec2(520.0, 720.0);

const MIN_VIEWPORT_SIZE: egui::Vec2 = egui::vec2(360.0, 280.0);

fn initial_size_key(viewport_id: egui::ViewportId) -> egui::Id {
    egui::Id::new((viewport_id, "initial_size_set"))
}

/// Viewport redimensionnable : `inner_size` uniquement à la première ouverture.
/// Sans ça, egui force la taille par défaut à chaque frame et le resize devient saccadé.
pub fn resizable_viewport_builder(
    ctx: &egui::Context,
    viewport_id: egui::ViewportId,
    title: impl Into<String>,
    default_size: egui::Vec2,
) -> egui::ViewportBuilder {
    let sized_key = initial_size_key(viewport_id);
    let mut builder = egui::ViewportBuilder::default()
        .with_title(title.into())
        .with_resizable(true)
        .with_min_inner_size(MIN_VIEWPORT_SIZE);
    if !ctx.data(|d| d.get_temp::<bool>(sized_key).unwrap_or(false)) {
        builder = builder.with_inner_size(default_size);
        ctx.data_mut(|d| d.insert_temp(sized_key, true));
    }
    builder
}

fn clear_initial_size_hint(ctx: &egui::Context, viewport_id: egui::ViewportId) {
    ctx.data_mut(|d| d.remove::<bool>(initial_size_key(viewport_id)));
}

pub fn clear_viewport_size_hint(ctx: &egui::Context, viewport_id: egui::ViewportId) {
    clear_initial_size_hint(ctx, viewport_id);
}

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
    show_config_viewport_sized(ctx, app_id, app_title, open, CONFIG_WINDOW_SIZE, add_contents);
}

/// Variante avec taille initiale personnalisée.
pub fn show_config_viewport_sized(
    ctx: &egui::Context,
    app_id: &str,
    app_title: &str,
    open: &mut bool,
    default_size: egui::Vec2,
    add_contents: impl FnMut(&mut egui::Ui),
) {
    show_config_viewport_with_id_sized(
        ctx,
        config_viewport_id(app_id),
        app_title,
        open,
        default_size,
        add_contents,
    );
}

/// Variante avec identifiant viewport explicite (fenêtres multiples).
pub fn show_config_viewport_with_id(
    ctx: &egui::Context,
    viewport_id: egui::ViewportId,
    app_title: &str,
    open: &mut bool,
    add_contents: impl FnMut(&mut egui::Ui),
) {
    show_config_viewport_with_id_sized(
        ctx,
        viewport_id,
        app_title,
        open,
        CONFIG_WINDOW_SIZE,
        add_contents,
    );
}

pub fn show_config_viewport_with_id_sized(
    ctx: &egui::Context,
    viewport_id: egui::ViewportId,
    app_title: &str,
    open: &mut bool,
    default_size: egui::Vec2,
    mut add_contents: impl FnMut(&mut egui::Ui),
) {
    if !*open {
        clear_initial_size_hint(ctx, viewport_id);
        return;
    }
    let title = config_viewport_title(app_title);
    let mut still_open = *open;
    ctx.show_viewport_immediate(
        viewport_id,
        resizable_viewport_builder(ctx, viewport_id, title, default_size),
        |ctx, _class| {
            egui_theme::apply_system_visuals(ctx);
            egui_theme::show_central_panel(ctx, |ui| {
                ui.heading("Menu config / diagnostic");
                ui.separator();
                egui::ScrollArea::vertical()
                    .id_salt(egui::Id::new((viewport_id, "config_scroll")))
                    .auto_shrink([false, false])
                    .show(ui, |ui| add_contents(ui));
            });
            if ctx.input(|i| i.viewport().close_requested()) {
                still_open = false;
            }
        },
    );
    if !still_open {
        clear_initial_size_hint(ctx, viewport_id);
    }
    *open = still_open;
}

/// Grille lecture seule label / valeur (endpoints stack, inject).
pub fn readonly_grid(ui: &mut egui::Ui, id: &str, rows: &[(&str, &str)]) {
    egui::Grid::new(id)
        .num_columns(2)
        .spacing(egui::vec2(12.0, 4.0))
        .striped(true)
        .show(ui, |ui| {
            for (label, value) in rows {
                ui.label(egui::RichText::new(*label).weak());
                ui.add(
                    egui::Label::new(egui::RichText::new(*value).monospace())
                        .wrap()
                        .selectable(true),
                );
                ui.end_row();
            }
        });
}

/// Pied de page chemins config (référence + runtime).
pub fn config_paths_footer(ui: &mut egui::Ui, reference: &str, runtime: &str) {
    ui.separator();
    ui.label("Référence:");
    ui.add(
        egui::Label::new(egui::RichText::new(reference).monospace().small())
            .wrap()
            .selectable(true),
    );
    ui.label("Overrides (config/local):");
    ui.add(
        egui::Label::new(egui::RichText::new(runtime).monospace().small())
            .wrap()
            .selectable(true),
    );
}

/// Bundle macOS Config Manager (`io.aestrk.configmanager`).
pub const CONFIG_MANAGER_BUNDLE_ID: &str = "io.aestrk.configmanager";

/// Lance Config Manager (macOS `open -b`).
pub fn open_config_manager() {
    #[cfg(target_os = "macos")]
    {
        use std::process::Command;
        let _ = Command::new("open")
            .args(["-b", CONFIG_MANAGER_BUNDLE_ID])
            .status();
    }
}

/// Bloc standard : préférences → Config Manager.
pub fn config_manager_hint(ui: &mut egui::Ui) {
    ui.separator();
    ui.label(egui::RichText::new("Préférences utilisateur").strong());
    ui.label(
        "Colonnes, watchlists, refresh, scope trading… — éditez dans Config Manager (onglet Préférences).",
    );
    if ui.button("Ouvrir Config Manager").clicked() {
        open_config_manager();
    }
}
