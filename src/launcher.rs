//! Intégration launcher AlphaLagoon (masquage Dock macOS).
//!
//! Le launcher Xcode définit `ALPHA_LAGOON_HIDE_DOCK=1` pour toute la stack.
//! Pendant « Lancer tout », `ALPHA_LAGOON_START_GUI_HIDDEN=1` évite le flash des fenêtres eframe
//! (équivalent Python/Tk `withdraw` + SIGUSR2 pour révéler).

/// `true` si le launcher a demandé de masquer l'icône Dock.
pub fn hide_dock_requested() -> bool {
    std::env::var("ALPHA_LAGOON_HIDE_DOCK")
        .map(|v| v.trim() == "1")
        .unwrap_or(false)
}

/// Fenêtre invisible au démarrage ; révélation via SIGUSR2 (console launcher) ou [`poll_start_gui_reveal`].
pub fn start_gui_hidden_requested() -> bool {
    std::env::var("ALPHA_LAGOON_START_GUI_HIDDEN")
        .map(|v| v.trim() == "1")
        .unwrap_or(false)
}

#[cfg(all(feature = "launcher", unix))]
mod start_hidden {
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::Once;

    static REVEAL_PENDING: AtomicBool = AtomicBool::new(false);
    static SIGUSR2_HOOK: Once = Once::new();

    extern "C" fn on_sigusr2(_: libc::c_int) {
        REVEAL_PENDING.store(true, Ordering::Release);
    }

    pub fn install_sigusr2_hook() {
        SIGUSR2_HOOK.call_once(|| unsafe {
            libc::signal(libc::SIGUSR2, on_sigusr2 as *const () as libc::sighandler_t);
        });
    }

    pub fn poll(ctx: &egui::Context) {
        if !REVEAL_PENDING.swap(false, Ordering::Acquire) {
            return;
        }
        ctx.send_viewport_cmd(egui::ViewportCommand::Visible(true));
        ctx.request_repaint();
    }
}

#[cfg(all(feature = "launcher", not(unix)))]
mod start_hidden {
    pub fn install_sigusr2_hook() {}
    pub fn poll(_ctx: &egui::Context) {}
}

/// À appeler en début de frame si l'app n'est pas passée par [`run_native`] (rare).
pub fn poll_start_gui_reveal(ctx: &egui::Context) {
    if start_gui_hidden_requested() {
        start_hidden::poll(ctx);
    }
}

struct LauncherStackApp<'a> {
    inner: Box<dyn 'a + eframe::App>,
}

impl<'a> eframe::App for LauncherStackApp<'a> {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        start_hidden::poll(ctx);
        self.inner.update(ctx, frame);
    }
}

/// Applique la politique Dock launcher sur des `eframe::NativeOptions`.
pub fn prepare_native_options(mut options: eframe::NativeOptions) -> eframe::NativeOptions {
    if start_gui_hidden_requested() {
        options.viewport = options.viewport.with_visible(false);
    }

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
    let wrap_reveal = start_gui_hidden_requested();
    let wrapped: eframe::AppCreator<'_> = Box::new(move |cc| {
        crate::egui_theme::apply_system_visuals(&cc.egui_ctx);
        if wrap_reveal {
            start_hidden::install_sigusr2_hook();
            cc.egui_ctx
                .send_viewport_cmd(egui::ViewportCommand::Visible(false));
        }
        let app = creator(cc)?;
        if wrap_reveal {
            Ok(Box::new(LauncherStackApp { inner: app }))
        } else {
            Ok(app)
        }
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
