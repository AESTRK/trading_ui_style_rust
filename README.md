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
| Dépendances runtime | Aucune |
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

## Installation (consommateur)

Dépendance Git dans `Cargo.toml` :

```toml
trading_ui_style_rust = { git = "https://github.com/AESTRK/trading_ui_style_rust.git", branch = "master" }
```

## Build et tests

```bash
cargo build
cargo test
```

## Dépôt

https://github.com/AESTRK/trading_ui_style_rust
