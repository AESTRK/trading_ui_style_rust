//! Fermeture manuelle des bandeaux (✕) avec journalisation tracing.

use std::collections::HashSet;

use egui::{self, RichText};

use crate::TEXT_SIZES;

#[derive(Clone, Default)]
pub struct BannerDismissRegistry {
    /// Clés stables (ex. `stale_data`) — masquées jusqu'à redémarrage de l'app.
    kinds: HashSet<String>,
    /// Clé = `kind` + contenu — réapparaît si le message change.
    content: HashSet<String>,
}

impl BannerDismissRegistry {
    pub fn is_dismissed_kind(&self, kind: &str) -> bool {
        self.kinds.contains(kind)
    }

    pub fn is_dismissed_content(&self, kind: &str, content: &str) -> bool {
        self.content.contains(&Self::content_key(kind, content))
    }

    pub fn dismiss_kind(&mut self, app_id: &str, kind: &str, summary: &str) {
        if self.kinds.insert(kind.to_string()) {
            Self::log_dismiss(app_id, kind, summary);
        }
    }

    pub fn dismiss_content(&mut self, app_id: &str, kind: &str, content: &str) {
        let key = Self::content_key(kind, content);
        if self.content.insert(key) {
            Self::log_dismiss(app_id, kind, content);
        }
    }

    fn content_key(kind: &str, content: &str) -> String {
        format!("{kind}:{}", content.trim())
    }

    fn log_dismiss(app_id: &str, kind: &str, summary: &str) {
        let preview: String = summary.chars().take(200).collect();
        tracing::info!(
            "GUI_BANNER_DISMISSED | app={} | kind={} | summary={}",
            app_id.trim(),
            kind,
            preview
        );
    }
}

pub fn registry_id(app_id: &str) -> egui::Id {
    egui::Id::new((
        "banner_dismiss_registry",
        app_id.trim().to_ascii_uppercase(),
    ))
}

pub fn with_dismiss_registry<R>(
    ctx: &egui::Context,
    app_id: &str,
    f: impl FnOnce(&mut BannerDismissRegistry) -> R,
) -> R {
    let id = registry_id(app_id);
    let mut reg = ctx
        .data_mut(|d| d.get_temp_mut_or_insert_with(id, BannerDismissRegistry::default).clone());
    let out = f(&mut reg);
    ctx.data_mut(|d| *d.get_temp_mut_or_insert_with(id, BannerDismissRegistry::default) = reg);
    out
}

/// Bouton ✕ aligné à droite dans une rangée de bandeau.
pub fn draw_banner_close_button(
    ui: &mut egui::Ui,
    reg: &mut BannerDismissRegistry,
    app_id: &str,
    kind: &str,
    summary: &str,
    stable_kind_only: bool,
) {
    let clicked = ui
        .add(
            egui::Button::new(RichText::new("✕").size(TEXT_SIZES.toolbar).color(egui::Color32::WHITE))
                .min_size(egui::vec2(22.0, 20.0)),
        )
        .on_hover_text("Fermer ce bandeau")
        .clicked();
    if clicked {
        if stable_kind_only {
            reg.dismiss_kind(app_id, kind, summary);
        } else {
            reg.dismiss_content(app_id, kind, summary);
        }
    }
}

/// Ligne d'avertissement inline (hors bandeau top) avec ✕.
pub fn draw_dismissible_warning_row(
    ui: &mut egui::Ui,
    ctx: &egui::Context,
    app_id: &str,
    dismiss_kind: &str,
    text: &str,
    color: egui::Color32,
) {
    with_dismiss_registry(ctx, app_id, |reg| {
        if reg.is_dismissed_kind(dismiss_kind) {
            return;
        }
        ui.horizontal(|ui| {
            ui.label(
                RichText::new(text)
                    .color(color)
                    .size(TEXT_SIZES.status),
            );
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                draw_banner_close_button(ui, reg, app_id, dismiss_kind, text, true);
            });
        });
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dismiss_content_logs_once_per_key() {
        let mut reg = BannerDismissRegistry::default();
        assert!(!reg.is_dismissed_content("error", "foo"));
        reg.dismiss_content("TEST", "error", "foo");
        assert!(reg.is_dismissed_content("error", "foo"));
        reg.dismiss_content("TEST", "error", "foo");
        assert!(reg.is_dismissed_content("error", "foo"));
    }

    #[test]
    fn dismiss_kind_is_stable() {
        let mut reg = BannerDismissRegistry::default();
        reg.dismiss_kind("TEST", "stale_data", "age=10s");
        assert!(reg.is_dismissed_kind("stale_data"));
        assert!(!reg.is_dismissed_kind("market_date_mismatch"));
    }
}
