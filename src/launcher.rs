//! Intégration launcher AlphaLagoon (masquage Dock macOS).
//!
//! Le launcher Xcode définit `ALPHA_LAGOON_HIDE_DOCK=1` pour toute la stack.

use eframe::{App, Frame};

/// Fond viewport + thème avant le premier paint (toutes les apps `run_native`).
struct StackClearColorApp<'a>(Box<dyn App + 'a>);

impl<'a> App for StackClearColorApp<'a> {
    fn update(&mut self, ctx: &egui::Context, frame: &mut Frame) {
        crate::egui_theme::apply_system_visuals(ctx);
        self.0.update(ctx, frame);
    }

    fn clear_color(&self, visuals: &egui::Visuals) -> [f32; 4] {
        crate::egui_theme::stack_clear_color(visuals)
    }

    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        self.0.save(storage);
    }

    fn auto_save_interval(&self) -> std::time::Duration {
        self.0.auto_save_interval()
    }

    fn persist_egui_memory(&self) -> bool {
        self.0.persist_egui_memory()
    }

    fn on_exit(&mut self, gl: Option<&eframe::glow::Context>) {
        self.0.on_exit(gl);
    }
}

/// `true` si le launcher a demandé de masquer l'icône Dock.
pub fn hide_dock_requested() -> bool {
    std::env::var("ALPHA_LAGOON_HIDE_DOCK")
        .map(|v| v.trim() == "1")
        .unwrap_or(false)
}

/// Applique la politique Dock launcher sur des `eframe::NativeOptions`.
pub fn prepare_native_options(mut options: eframe::NativeOptions) -> eframe::NativeOptions {
    if !hide_dock_requested() {
        return options;
    }

    let previous = options.event_loop_builder.take();
    options.event_loop_builder = Some(Box::new(move |builder| {
        if let Some(prev) = previous {
            prev(builder);
        }
        #[cfg(target_os = "macos")]
        {
            use winit::platform::macos::{ActivationPolicy, EventLoopBuilderExtMacOS};
            builder.with_activation_policy(ActivationPolicy::Accessory);
        }
        #[cfg(not(target_os = "macos"))]
        {
            let _ = builder;
        }
    }));
    options
}

/// Point d'entrée GUI standard stack : masque le Dock si demandé par le launcher.
pub fn run_native(
    app_name: &str,
    options: eframe::NativeOptions,
    creator: eframe::AppCreator<'_>,
) -> eframe::Result {
    app_runtime_rust::ensure_config_persist_relay();
    let wrapped: eframe::AppCreator<'_> = Box::new(move |cc| {
        crate::egui_theme::apply_system_visuals(&cc.egui_ctx);
        let app = creator(cc)?;
        Ok(Box::new(StackClearColorApp(app)) as Box<dyn App>)
    });
    eframe::run_native(app_name, prepare_native_options(options), wrapped)
}

/// Compatibilité — préférer [`prepare_native_options`] ou [`run_native`].
#[macro_export]
macro_rules! apply_launcher_hide_dock {
    ($native_options:expr) => {
        $native_options = $crate::prepare_native_options($native_options);
    };
}
