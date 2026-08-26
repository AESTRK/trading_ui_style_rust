//! Fenêtre de configuration détachable (viewport OS) — pattern emergency_panel / orderbook stack.

use std::collections::HashSet;
use std::sync::{Mutex, OnceLock};

use eframe::egui;
use serde_json::Value;

use crate::egui_theme;

/// Libellé du bouton toolbar → deep link Config Manager.
pub const MENU_CONFIG_LABEL: &str = "Config Manager";

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

/// Lance Config Manager pour le crate courant (`APP_PERSIST_ID` si défini).
pub fn open_config_manager_for_crate(crate_app_id: &str) {
    open_config_manager(&resolve_persist_app_id(crate_app_id));
}

/// Bouton toolbar : ouvre Config Manager sur la section persist de cette app.
pub fn config_manager_toolbar_button(ui: &mut egui::Ui, crate_app_id: &str) -> bool {
    if ui.button(MENU_CONFIG_LABEL).clicked() {
        open_config_manager_for_crate(crate_app_id);
        true
    } else {
        false
    }
}

/// Ancien bouton — ouvrait une fenêtre config in-app (préférer `config_manager_toolbar_button`).
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
                ui.heading("Configuration");
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

/// Schéma URL deep link : `io.aestrk.configmanager://app/<app_id>`.
pub const CONFIG_MANAGER_URL_SCHEME: &str = "io.aestrk.configmanager";

/// Lance Config Manager (macOS `open`), optionnellement sur la section d'une app.
/// Une seule fenêtre côté Config Manager (`Window` + deep link sur l'existant).
pub fn open_config_manager(app_id: &str) {
    #[cfg(target_os = "macos")]
    {
        use std::process::Command;
        let url = format!("{CONFIG_MANAGER_URL_SCHEME}://app/{app_id}");
        let _ = Command::new("open").arg(url).status();
    }
    #[cfg(not(target_os = "macos"))]
    {
        let _ = app_id;
    }
}

/// Corps standard du menu config : Config Manager + chemins fichiers.
pub fn minimal_config_panel(ui: &mut egui::Ui, app_id: &str, reference: &str, runtime: &str) {
    config_manager_hint(ui, app_id);
    ui.add_space(6.0);
    ui.label(
        egui::RichText::new(
            "Les modifications enregistrées dans Config Manager sont poussées en temps réel (bus ZMQ).",
        )
        .weak()
        .small(),
    );
    config_paths_footer(ui, reference, runtime);
}

/// ID stack (`APP_PERSIST_ID`) ou nom crate Rust.
fn resolve_persist_app_id(crate_app_id: &str) -> String {
    app_runtime_rust::persist_app_id(crate_app_id)
}

/// Installe l'écouteur ZMQ + relay (une fois par app).
fn ensure_config_persist_listener(ctx: &egui::Context, app_id: &str, env_keys: &[&str]) {
    static INSTALLED: OnceLock<Mutex<HashSet<String>>> = OnceLock::new();
    let persist_id = resolve_persist_app_id(app_id);
    let key = if env_keys.is_empty() {
        persist_id.clone()
    } else {
        format!("{persist_id}\0{}", env_keys.join("\0"))
    };
    let mut installed = INSTALLED
        .get_or_init(|| Mutex::new(HashSet::new()))
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    if !installed.insert(key) {
        return;
    }
    let ctx = ctx.clone();
    let persist_id = persist_id.clone();
    app_runtime_rust::install_config_persist_listener(&persist_id, move || {
        ctx.request_repaint();
    });
    if std::env::var("CONFIG_PERSIST_RELAY_EXTERNAL")
        .map(|v| {
            let v = v.trim().to_lowercase();
            !(v == "1" || v == "true" || v == "yes" || v == "on")
        })
        .unwrap_or(true)
    {
        app_runtime_rust::ensure_config_persist_relay();
    }
}

/// À appeler en tête de `update()` — `Some(persist)` si Config Manager a poussé une mise à jour.
pub fn poll_config_persist(
    ctx: &egui::Context,
    app_id: &str,
    env_keys: &[&str],
) -> Option<Value> {
    poll_config_persist_for(ctx, app_id, env_keys)
}

/// Mise à jour bus config persist (réglages + commandes éphémères).
pub struct ConfigPersistSync {
    pub persist: Option<serde_json::Value>,
    pub commands: Vec<app_runtime_rust::ConfigPersistCommand>,
}

/// Variante avec les mêmes `env_keys` que `AppPaths::runtime_settings_file`.
pub fn poll_config_persist_sync_for(
    ctx: &egui::Context,
    app_id: &str,
    env_keys: &[&str],
) -> Option<ConfigPersistSync> {
    let persist_id = resolve_persist_app_id(app_id);
    ensure_config_persist_listener(ctx, app_id, env_keys);
    let message = app_runtime_rust::take_config_persist_update(&persist_id)?;
    let commands = message.commands.clone();
    let filtered = app_runtime_rust::filter_persist_for_app(&persist_id, &message.persist);
    let mut msg = message;
    msg.persist = filtered.clone();
    let persist = if persist_payload_nonempty(&filtered) {
        app_runtime_rust::apply_config_persist_message(&persist_id, env_keys, &msg)
            .ok()
            .filter(|v| persist_payload_nonempty(v))
    } else {
        let _ = app_runtime_rust::apply_config_persist_message(&persist_id, env_keys, &msg);
        None
    };
    Some(ConfigPersistSync { persist, commands })
}

fn persist_payload_nonempty(persist: &serde_json::Value) -> bool {
    persist
        .as_object()
        .is_some_and(|obj| obj.iter().any(|(k, v)| !k.starts_with('_') && !v.is_null()))
}

/// Variante avec les mêmes `env_keys` que `AppPaths::runtime_settings_file`.
pub fn poll_config_persist_for(
    ctx: &egui::Context,
    app_id: &str,
    env_keys: &[&str],
) -> Option<serde_json::Value> {
    poll_config_persist_sync_for(ctx, app_id, env_keys).and_then(|sync| sync.persist)
}

/// Applique le reload si Config Manager a poussé une mise à jour persist.
pub fn sync_config_persist(
    ctx: &egui::Context,
    app_id: &str,
    reload: impl FnOnce(Option<Value>),
) {
    sync_config_persist_for(ctx, app_id, &[], reload);
}

/// Persist + commandes éphémères (tests audio, etc.).
pub fn sync_config_persist_with(
    ctx: &egui::Context,
    app_id: &str,
    reload: impl FnOnce(&ConfigPersistSync),
) {
    sync_config_persist_with_for(ctx, app_id, &[], reload);
}

pub fn sync_config_persist_with_for(
    ctx: &egui::Context,
    app_id: &str,
    env_keys: &[&str],
    reload: impl FnOnce(&ConfigPersistSync),
) {
    if let Some(sync) = poll_config_persist_sync_for(ctx, app_id, env_keys) {
        if let Some(ref persist) = sync.persist {
            app_runtime_rust::apply_persist_env(persist);
        }
        reload(&sync);
    }
}

/// Variante avec les mêmes `env_keys` que `AppPaths::runtime_settings_file`.
pub fn sync_config_persist_for(
    ctx: &egui::Context,
    app_id: &str,
    env_keys: &[&str],
    reload: impl FnOnce(Option<Value>),
) {
    if let Some(persist) = poll_config_persist_for(ctx, app_id, env_keys) {
        app_runtime_rust::apply_persist_env(&persist);
        reload(Some(persist));
    }
}

/// Bloc standard : IPC + préférences → Config Manager.
pub fn config_manager_hint(ui: &mut egui::Ui, app_id: &str) {
    ui.separator();
    ui.label(egui::RichText::new("IPC et préférences").strong());
    ui.label(
        "Ports, endpoints et options persistées — éditez dans Config Manager (Registre IPC + onglet app). Les changements enregistrés sont poussés sur le bus config en temps réel.",
    );
    if ui.button("Ouvrir Config Manager").clicked() {
        open_config_manager(app_id);
    }
}
