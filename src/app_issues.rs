//! Modèle unifié erreurs / avertissements — journal TTL + bandeau egui + logs tracing.
//!
//! Pattern de référence : `orderbook_rust` (`IssueJournal` + bandeau rouge / orange).

use crate::banner_dismiss::with_dismiss_registry;
use crate::issues::{IssueJournal, IssueSeverity, StackIssueBoard};
use std::time::{Duration, Instant};

pub const DEFAULT_ISSUE_TTL: Duration = Duration::from_secs(45);
pub const DEFAULT_MAX_ISSUE_RECORDS: usize = 16;
pub const DEFAULT_MAX_BANNER_ISSUES: usize = 6;
pub const DEFAULT_WARNING_BANNER_TITLE: &str = "AVERTISSEMENT";
/// Durée du bandeau vert **RÉSOLU** après disparition des erreurs (secondes).
pub const DEFAULT_RESOLVED_BANNER_SECS: f64 = 10.0;
pub const DEFAULT_RESOLVED_BANNER_DETAIL: &str = "Le problème signalé est corrigé.";

/// Journal opérationnel par application (`ORDERBOOK`, `MANUAL_ORDERS`, …).
pub struct AppIssueReporter {
    journal: IssueJournal,
    app_id: String,
}

impl AppIssueReporter {
    pub fn new(app_id: &str, ttl: Duration, max_records: usize) -> Self {
        Self {
            journal: IssueJournal::new(ttl, max_records),
            app_id: app_id.trim().to_ascii_uppercase(),
        }
    }

    pub fn with_defaults(app_id: &str) -> Self {
        Self::new(app_id, DEFAULT_ISSUE_TTL, DEFAULT_MAX_ISSUE_RECORDS)
    }

    pub fn app_id(&self) -> &str {
        &self.app_id
    }

    pub fn journal(&self) -> &IssueJournal {
        &self.journal
    }

    pub fn journal_mut(&mut self) -> &mut IssueJournal {
        &mut self.journal
    }

    pub fn prune(&mut self) {
        self.journal.prune();
    }

    pub fn clear_if(&mut self, pred: impl Fn(&str) -> bool) {
        self.journal.clear_if(pred);
    }

    pub fn report(&mut self, severity: IssueSeverity, text: impl Into<String>) {
        Self::log_and_push(&mut self.journal, &self.app_id, severity, text, None);
    }

    pub fn report_keyed(
        &mut self,
        severity: IssueSeverity,
        text: impl Into<String>,
        dedupe_key: &str,
    ) {
        Self::log_and_push(
            &mut self.journal,
            &self.app_id,
            severity,
            text,
            Some(dedupe_key),
        );
    }

    fn log_and_push(
        journal: &mut IssueJournal,
        app_id: &str,
        severity: IssueSeverity,
        text: impl Into<String>,
        dedupe_key: Option<&str>,
    ) {
        let text = text.into();
        let trimmed = text.trim();
        if trimmed.is_empty() {
            return;
        }
        if let Some(key) = dedupe_key.filter(|k| !k.trim().is_empty()) {
            journal.push_keyed(severity, trimmed.to_string(), Some(key));
        } else {
            journal.push(severity, trimmed.to_string());
        }
        match severity {
            IssueSeverity::Error => tracing::error!("{app_id} | {trimmed}"),
            IssueSeverity::Warning => tracing::warn!("{app_id} | {trimmed}"),
        }
    }
}

pub fn issue_error_title(app_display_name: &str) -> String {
    format!("ERREUR {}", app_display_name.trim())
}

pub fn issue_resolved_title(app_display_name: &str) -> String {
    format!("RÉSOLU {}", app_display_name.trim())
}

/// Bandeau vert top après disparition des erreurs (flash puis auto-dismiss).
pub fn show_resolved_banner(
    ctx: &egui::Context,
    app_display_name: &str,
    detail: &str,
) {
    let title = issue_resolved_title(app_display_name);
    with_dismiss_registry(ctx, app_display_name, |reg| {
        if reg.is_dismissed_content("resolved", detail) {
            return;
        }
        egui::TopBottomPanel::top(egui::Id::new("stack_issue_resolved_banner"))
            .resizable(false)
            .show_separator_line(false)
            .frame(crate::egui_theme::stack_top_panel_frame(ctx))
            .exact_height(crate::banner::issue_panel_exact_height(1))
            .show(ctx, |ui| {
            crate::egui_theme::paint_top_stack_panel_bg(ui);
            crate::banner::prepare_issue_top_panel(ui);
            crate::banner::draw_resolved_banner(
                ui,
                ctx,
                &title,
                detail,
                Some((reg, app_display_name)),
            );
            crate::egui_theme::fill_top_panel_remainder(ui);
        });
    });
}

fn format_board_errors(board: &StackIssueBoard) -> String {
    board
        .errors()
        .iter()
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join(" · ")
}

/// Mémorise les erreurs actives et affiche un flash vert quand elles disparaissent.
#[derive(Debug, Default)]
pub struct IssueBannerController {
    had_errors: bool,
    resolved_until: Option<Instant>,
    /// Dernières lignes d'erreur affichées (reprises telles quelles dans le bandeau vert).
    last_error_banner_text: String,
}

impl IssueBannerController {
    pub fn show(
        &mut self,
        ctx: &egui::Context,
        app_display_name: &str,
        board: &StackIssueBoard,
    ) {
        self.show_with_detail(ctx, app_display_name, board, DEFAULT_RESOLVED_BANNER_DETAIL);
    }

    pub fn show_with_detail(
        &mut self,
        ctx: &egui::Context,
        app_display_name: &str,
        board: &StackIssueBoard,
        resolved_detail: &str,
    ) {
        self.on_board(board);
        if board.has_errors() {
            show_app_issues(ctx, app_display_name, board);
            return;
        }
        if self.showing_resolved() {
            let detail = self.resolved_banner_detail(resolved_detail);
            show_resolved_banner(ctx, app_display_name, &detail);
            if let Some(until) = self.resolved_until {
                let rem = until.saturating_duration_since(Instant::now());
                if rem > Duration::ZERO {
                    ctx.request_repaint_after(rem);
                }
            }
            return;
        }
        if !board.is_empty() {
            show_app_issues(ctx, app_display_name, board);
        }
    }

    pub(crate) fn resolved_banner_detail(&self, fallback: &str) -> String {
        if self.last_error_banner_text.is_empty() {
            fallback.trim().to_string()
        } else {
            self.last_error_banner_text.clone()
        }
    }

    fn on_board(&mut self, board: &StackIssueBoard) {
        let errors_active = board.has_errors();
        if errors_active {
            self.resolved_until = None;
            self.last_error_banner_text = format_board_errors(board);
        } else if self.had_errors {
            self.resolved_until =
                Some(Instant::now() + Duration::from_secs_f64(DEFAULT_RESOLVED_BANNER_SECS));
        }
        self.had_errors = errors_active;
        if let Some(until) = self.resolved_until {
            if Instant::now() >= until {
                self.resolved_until = None;
            }
        }
    }

    fn showing_resolved(&self) -> bool {
        self.resolved_until
            .is_some_and(|until| Instant::now() < until)
    }
}

/// Bandeau top rouge / orange + repaint pour clignotement.
pub fn show_app_issues(ctx: &egui::Context, app_display_name: &str, board: &StackIssueBoard) {
    if board.is_empty() {
        return;
    }
    board.show_top(
        ctx,
        app_display_name,
        &issue_error_title(app_display_name),
        DEFAULT_WARNING_BANNER_TITLE,
    );
}

/// Fragments connectivité (plusieurs sources) → bandeau classifié.
pub fn connectivity_issue_board(
    fragments: impl IntoIterator<Item = impl Into<String>>,
) -> StackIssueBoard {
    let mut board = StackIssueBoard::new();
    for fragment in fragments {
        let text = fragment.into();
        let trimmed = text.trim();
        if trimmed.is_empty() {
            continue;
        }
        if trimmed.contains('·') {
            board.extend_fragments(crate::issues::split_banner_fragments(trimmed));
        } else {
            board.push_fragment(trimmed);
        }
    }
    board
}

/// Seuil par défaut : alerte si PUB ZMQ absente ou trop vieille alors que des symboles sont actifs.
pub const DEFAULT_FEED_ZMQ_STALL_MS: i64 = 45_000;

/// Publishers marché : hub KO mais lignes live → warning orange, pas erreur rouge bloquante.
pub fn feed_publisher_issue_board(
    hub_ok: bool,
    hub_banner_text: &str,
    live_row_count: usize,
) -> StackIssueBoard {
    let text = hub_banner_text.trim();
    if hub_ok || text.is_empty() {
        return StackIssueBoard::new();
    }
    if live_row_count > 0 {
        let mut board = StackIssueBoard::new();
        board.push_warning(format!(
            "CONNECTIVITÉ HUB — {text} (données feed locales actives)"
        ));
        return board;
    }
    connectivity_issue_board([text.to_string()])
}

/// Hub OK mais feed local muet — distingue connectivité exchange vs publisher métier.
pub fn append_feed_local_freshness_warnings(
    board: &mut StackIssueBoard,
    hub_ok: bool,
    tracked_symbols: usize,
    live_rows: usize,
    zmq_publish_idle_ms: Option<i64>,
    stall_ms: i64,
) {
    if tracked_symbols == 0 || live_rows > 0 {
        return;
    }
    let stall_ms = stall_ms.max(5_000);
    let message = match zmq_publish_idle_ms {
        None if hub_ok => Some(format!(
            "FEED LOCAL — hub OK mais 0/{tracked_symbols} symboles live (aucune publication ZMQ encore)"
        )),
        None => None,
        Some(idle) if idle >= stall_ms => Some(format!(
            "FEED LOCAL — hub {} — dernière PUB ZMQ il y a {idle} ms ({tracked_symbols} symboles, 0 live)",
            if hub_ok { "OK" } else { "KO" }
        )),
        _ => None,
    };
    if let Some(msg) = message {
        board.push_warning(msg);
    }
}

/// Hub + freshness ZMQ pour publishers snapshot / microstructure.
pub fn feed_publisher_issue_board_with_freshness(
    hub_ok: bool,
    hub_banner_text: &str,
    tracked_symbols: usize,
    live_row_count: usize,
    zmq_publish_idle_ms: Option<i64>,
    stall_ms: i64,
) -> StackIssueBoard {
    let mut board = feed_publisher_issue_board(hub_ok, hub_banner_text, live_row_count);
    append_feed_local_freshness_warnings(
        &mut board,
        hub_ok,
        tracked_symbols,
        live_row_count,
        zmq_publish_idle_ms,
        stall_ms,
    );
    board
}

/// Connectivité hub + erreur opérationnelle locale (sync REST, persistance, …).
pub fn operational_issue_board(connectivity_text: &str, operational_error: &str) -> StackIssueBoard {
    let mut board = connectivity_issue_board([connectivity_text]);
    let err = operational_error.trim();
    if !err.is_empty() {
        board.push_error(err);
    }
    board
}

/// Texte vert affiché après disparition des erreurs (harmonisé stack).
pub fn default_resolved_detail(app_display_name: &str) -> String {
    format!(
        "{} — plus d'erreur bloquante.",
        app_display_name.trim()
    )
}

/// Bandeau rouge / orange / vert **RÉSOLU** — pattern standard toutes apps egui.
pub fn show_harmonized_issue_banner(
    controller: &mut IssueBannerController,
    ctx: &egui::Context,
    app_display_name: &str,
    board: &StackIssueBoard,
) {
    let detail = default_resolved_detail(app_display_name);
    controller.show_with_detail(ctx, app_display_name, board, &detail);
}

/// Connectivité hub (texte ` · `) + cycle vert résolu.
pub fn harmonized_connectivity_banner(
    controller: &mut IssueBannerController,
    ctx: &egui::Context,
    app_display_name: &str,
    combined_text: &str,
) {
    let board = StackIssueBoard::from_combined_text(combined_text);
    show_harmonized_issue_banner(controller, ctx, app_display_name, &board);
}

/// Fragments connectivité + entrées du journal TTL.
pub fn build_issue_board(
    connectivity_text: &str,
    journal: &IssueJournal,
    max_banner_issues: usize,
) -> StackIssueBoard {
    StackIssueBoard::from_journal_and_text(connectivity_text, journal, max_banner_issues)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn issue_error_title_formats_app_name() {
        assert_eq!(issue_error_title("MANUAL ORDERS"), "ERREUR MANUAL ORDERS");
    }

    #[test]
    fn build_issue_board_merges_journal() {
        let mut reporter = AppIssueReporter::with_defaults("TEST");
        reporter.report(IssueSeverity::Warning, "ORDER_BLOCKED | reason=x");
        let board = build_issue_board("", reporter.journal(), 4);
        assert_eq!(board.warnings().len(), 1);
    }

    #[test]
    fn issue_resolved_title_formats_app_name() {
        assert_eq!(issue_resolved_title("FEES"), "RÉSOLU FEES");
    }

    #[test]
    fn issue_banner_controller_enters_resolved_after_errors_clear() {
        let mut ctrl = IssueBannerController::default();
        let mut board = StackIssueBoard::new();
        board.push_error("Binance indisponible");
        ctrl.on_board(&board);
        assert!(!ctrl.showing_resolved());

        board = StackIssueBoard::new();
        ctrl.on_board(&board);
        assert!(ctrl.showing_resolved());

        board.push_error("retour en erreur");
        ctrl.on_board(&board);
        assert!(!ctrl.showing_resolved());
    }

    #[test]
    fn issue_banner_controller_keeps_last_error_for_resolved_detail() {
        let mut ctrl = IssueBannerController::default();
        let mut board = StackIssueBoard::new();
        board.push_error("capital_flows_rust OFF — ZMQ requis");
        ctrl.on_board(&board);
        board = StackIssueBoard::new();
        ctrl.on_board(&board);
        assert!(ctrl.showing_resolved());
        assert_eq!(
            ctrl.resolved_banner_detail("fallback"),
            "capital_flows_rust OFF — ZMQ requis"
        );
    }
}
