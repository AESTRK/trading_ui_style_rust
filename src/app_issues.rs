//! Modèle unifié erreurs / avertissements — journal TTL + bandeau egui + logs tracing.
//!
//! Pattern de référence : `orderbook_rust` (`IssueJournal` + bandeau rouge / orange).

use crate::issues::{IssueJournal, IssueSeverity, StackIssueBoard};
use std::time::{Duration, Instant};

pub const DEFAULT_ISSUE_TTL: Duration = Duration::from_secs(45);
pub const DEFAULT_MAX_ISSUE_RECORDS: usize = 16;
pub const DEFAULT_MAX_BANNER_ISSUES: usize = 6;
pub const DEFAULT_WARNING_BANNER_TITLE: &str = "AVERTISSEMENT";
pub const DEFAULT_RESOLVED_BANNER_SECS: f64 = 4.0;
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
    egui::TopBottomPanel::top(egui::Id::new("stack_issue_resolved_banner")).show(ctx, |ui| {
        crate::banner::draw_resolved_banner(ui, ctx, &title, detail);
    });
    ctx.request_repaint_after(Duration::from_millis(33));
}

/// Mémorise les erreurs actives et affiche un flash vert quand elles disparaissent.
#[derive(Debug, Default)]
pub struct IssueBannerController {
    had_errors: bool,
    resolved_until: Option<Instant>,
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
        if self.showing_resolved() {
            show_resolved_banner(ctx, app_display_name, resolved_detail);
            return;
        }
        show_app_issues(ctx, app_display_name, board);
    }

    fn on_board(&mut self, board: &StackIssueBoard) {
        let errors_active = board.has_errors();
        if self.had_errors && !errors_active {
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
        &issue_error_title(app_display_name),
        DEFAULT_WARNING_BANNER_TITLE,
    );
    ctx.request_repaint_after(Duration::from_millis(33));
}

/// Fragments connectivité (plusieurs sources) → bandeau classifié.
pub fn connectivity_issue_board(
    fragments: impl IntoIterator<Item = impl Into<String>>,
) -> StackIssueBoard {
    let mut board = StackIssueBoard::new();
    for fragment in fragments {
        board.push_fragment(fragment.into());
    }
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

/// Connectivité + fragments ` · ` classifiés (apps sans journal local).
pub fn show_classified_connectivity_banner(
    ctx: &egui::Context,
    app_display_name: &str,
    combined_text: &str,
) {
    show_app_issues(
        ctx,
        app_display_name,
        &StackIssueBoard::from_combined_text(combined_text),
    );
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
    }
}
