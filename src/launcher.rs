//! Intégration launcher AlphaLagoon (masquage Dock macOS).
//!
//! Le launcher Xcode définit `ALPHA_LAGOON_HIDE_DOCK=1` pour toute la stack.
//! Équivalent Rust de `~/CommonProjects/config/launcher/macos_dock.py` (Python/Tk).

/// `true` si le launcher a demandé de masquer l'icône Dock.
pub fn hide_dock_requested() -> bool {
    std::env::var("ALPHA_LAGOON_HIDE_DOCK")
        .map(|v| v.trim() == "1")
        .unwrap_or(false)
}

/// Masque l'icône Dock via `winit::ActivationPolicy::Accessory` sur `eframe::NativeOptions`.
///
/// Macro (pas de dépendance `eframe` dans ce crate — compatible 0.31 / 0.34).
///
/// ```ignore
/// let mut native_options = eframe::NativeOptions { /* … */ };
/// trading_ui_style_rust::apply_launcher_hide_dock!(native_options);
/// eframe::run_native(/* … */);
/// ```
#[macro_export]
macro_rules! apply_launcher_hide_dock {
    ($native_options:expr) => {
        if $crate::hide_dock_requested() {
            $native_options.event_loop_builder = Some(::std::boxed::Box::new(|builder| {
                #[cfg(target_os = "macos")]
                {
                    use ::winit::platform::macos::{ActivationPolicy, EventLoopBuilderExtMacOS};
                    builder.with_activation_policy(ActivationPolicy::Accessory);
                }
                #[cfg(not(target_os = "macos"))]
                {
                    let _ = builder;
                }
            }));
        }
    };
}
