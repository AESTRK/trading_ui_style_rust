# trading_ui_style_rust

> Crate Rust partagé — palettes, couleurs et formatters UI pour les applications de trading egui.

## Rôle dans l'écosystème

Bibliothèque de style commune consommée par les applications Rust (`orderbook_rust`, `chartboard_rust`). Centralise les couleurs achat/vente, les palettes clair/sombre et les helpers de formatage pour garantir une cohérence visuelle entre les outils.

## Entrées et sorties

### Fonctionnel

| Entrées | Sorties |
|---------|---------|
| Mode thème appelant (clair / sombre) | Palette cohérente buy/sell/mid/widgets |
| Valeurs numériques (pct, EUR, volume) | Chaînes formatées pour affichage UI |
| Variation % (positive / négative) | Couleur conditionnelle (`pct_rgb`) |

### Technique

| Entrées | Sorties |
|---------|---------|
| `ThemeMode` / `palette(mode)` | `TradingPalette`, `LIGHT_PALETTE`, `DARK_PALETTE` |
| `f64` (montants, volumes) | `String` via `format_pct`, `format_eur_compact`, `format_volume_compact` |
| — | `Rgb`, `TEXT_SIZES`, constantes `BUY_GREEN`, `SELL_RED`, `MID_BLUE` |

## Stack technique

| Composant | Détail |
|-----------|--------|
| Langage | Rust 2024 |
| Dépendances runtime | `egui` 0.31 (module `banner` uniquement) |
| Type | Bibliothèque (`lib` uniquement, pas de binaire) |

## API principale

| Export | Description |
|--------|-------------|
| `TradingPalette`, `LIGHT_PALETTE`, `DARK_PALETTE` | Couleurs thème (buy, sell, mid, widgets…) |
| `ThemeMode`, `palette()` | Sélection clair / sombre |
| `Rgb`, `BUY_GREEN`, `SELL_RED`, `MID_BLUE` | Couleurs de base |
| `TEXT_SIZES` | Tailles de police standardisées |
| `format_pct`, `format_eur_compact`, `format_volume_compact` | Formatters |
| `pct_rgb` | Couleur conditionnelle selon variation |
| `hide_dock_requested` | Lit la variable d'environnement du launcher |
| `prepare_native_options` | Applique le masquage Dock sur `eframe::NativeOptions` |
| `run_native` | `eframe::run_native` + masquage Dock automatique (point d'entrée GUI standard) |
| `apply_launcher_hide_dock!` | Compatibilité — préférer `run_native` |
| `banner::draw_feed_banner` | Bandeau réseau 2 lignes (titre, métriques, stats WS, bouton) |
| `banner::draw_secondary_banner` | Bandeau secondaire (carnets, API absente, connectivité) |
| `banner::WsDowntimeStats` | Dernière coupure / cumul / déconnexions |

## Launcher / Dock macOS

Le launcher Xcode définit `ALPHA_LAGOON_HIDE_DOCK=1` pour toute la stack (équivalent de `macos_dock.py` côté Python/Tk).

Chaque app GUI Rust doit lancer sa fenêtre via `trading_ui_style_rust::run_native` — le masquage Dock est automatique :

```rust
trading_ui_style_rust::run_native(
    APP_TITLE,
    eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([width, height])
            .with_title(APP_TITLE),
        ..Default::default()
    },
    Box::new(|cc| {
        egui_theme::apply_system_visuals(&cc.egui_ctx);
        Ok(Box::new(MyApp::new()))
    }),
)?;
```

Aucune dépendance `winit` supplémentaire dans l'app : elle est portée par `trading_ui_style_rust` (feature `launcher`, activée par défaut).

## Installation (consommateur)

Dépendance Git dans `Cargo.toml` :

```toml
trading_ui_style_rust = { git = "https://github.com/AESTRK/trading_ui_style_rust.git", branch = "main" }
```

## Build et tests

```bash
cargo build
cargo test
```

## Dépôt

https://github.com/AESTRK/trading_ui_style_rust
