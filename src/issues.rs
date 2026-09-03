//! Classification erreur / avertissement et journal TTL — partagé entre apps egui.

use crate::banner_dismiss::with_dismiss_registry;
use std::collections::VecDeque;
use std::time::{Duration, Instant};

/// Panneau top egui (id fixe ou viewport secondaire).
pub fn show_issue_panel(
    ctx: &egui::Context,
    panel_id: egui::Id,
    app_id: &str,
    error_title: &str,
    errors: &[String],
    warning_title: &str,
    warnings: &[String],
) {
    if errors.is_empty() && warnings.is_empty() {
        return;
    }
    with_dismiss_registry(ctx, app_id, |reg| {
        let error_detail = errors.join(" · ");
        let warning_detail = warnings.join(" · ");
        let show_errors = !errors.is_empty() && !reg.is_dismissed_content("error", &error_detail);
        let show_warnings =
            !warnings.is_empty() && !reg.is_dismissed_content("warning", &warning_detail);
        if !show_errors && !show_warnings {
            return;
        }
        egui::TopBottomPanel::top(panel_id).show(ctx, |ui| {
            crate::banner::draw_stacked_issue_banners(
                ui,
                ctx,
                app_id,
                error_title,
                errors,
                warning_title,
                warnings,
                reg,
            );
        });
    });
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IssueSeverity {
    Error,
    Warning,
}

#[derive(Debug, Clone)]
struct IssueRecord {
    at: Instant,
    severity: IssueSeverity,
    text: String,
    dedupe_key: String,
}

/// Journal circulaire de messages opérationnels (erreurs / avertissements).
#[derive(Debug, Default)]
pub struct IssueJournal {
    records: VecDeque<IssueRecord>,
    ttl: Duration,
    max_records: usize,
    dedupe_window: Duration,
}

impl IssueJournal {
    pub fn new(ttl: Duration, max_records: usize) -> Self {
        Self {
            records: VecDeque::new(),
            ttl,
            max_records,
            dedupe_window: Duration::from_secs(20),
        }
    }

    pub fn push(&mut self, severity: IssueSeverity, text: String) {
        self.push_keyed(severity, text, None);
    }

    /// `dedupe_key` regroupe les variantes (ex. age_sec changeant) en une seule entrée.
    pub fn push_keyed(&mut self, severity: IssueSeverity, text: String, dedupe_key: Option<&str>) {
        let text = text.trim();
        if text.is_empty() {
            return;
        }
        let key = dedupe_key.unwrap_or(text).trim();
        if key.is_empty() {
            return;
        }
        let now = Instant::now();
        if let Some(existing) = self.records.iter_mut().find(|r| {
            r.dedupe_key == key && now.duration_since(r.at) < self.dedupe_window
        }) {
            existing.text = text.to_string();
            existing.at = now;
            if severity == IssueSeverity::Error {
                existing.severity = IssueSeverity::Error;
            }
            return;
        }
        self.records.push_back(IssueRecord {
            at: now,
            severity,
            text: text.to_string(),
            dedupe_key: key.to_string(),
        });
        while self.records.len() > self.max_records {
            self.records.pop_front();
        }
    }

    pub fn clear_if(&mut self, pred: impl Fn(&str) -> bool) {
        self.records.retain(|r| !pred(&r.text));
    }

    pub fn prune(&mut self) {
        let now = Instant::now();
        self.records
            .retain(|r| now.duration_since(r.at) <= self.ttl);
    }

    pub fn active_records(&self) -> impl Iterator<Item = (IssueSeverity, &str)> {
        self.records.iter().filter_map(|r| {
            if r.at.elapsed() <= self.ttl {
                Some((r.severity, r.text.as_str()))
            } else {
                None
            }
        })
    }
}

/// Bandeau empilé erreur / avertissement (sans journal TTL).
#[derive(Debug, Default)]
pub struct StackIssueBoard {
    errors: Vec<String>,
    warnings: Vec<String>,
}

impl StackIssueBoard {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn from_fragments(fragments: impl IntoIterator<Item = String>) -> Self {
        let mut board = Self::new();
        board.extend_fragments(fragments);
        board
    }

    pub fn from_combined_text(text: &str) -> Self {
        let mut board = Self::new();
        let trimmed = text.trim();
        if trimmed.is_empty() {
            return board;
        }
        if trimmed.contains('·') {
            board.extend_fragments(split_banner_fragments(trimmed));
        } else {
            board.push_fragment(trimmed);
        }
        board
    }

    pub fn extend_fragments(&mut self, fragments: impl IntoIterator<Item = String>) {
        for fragment in fragments {
            self.push_fragment(fragment);
        }
    }

    pub fn push_fragment(&mut self, text: impl Into<String>) {
        append_classified_fragment(&mut self.errors, &mut self.warnings, text.into());
    }

    pub fn push_error(&mut self, text: impl Into<String>) {
        push_unique_line(&mut self.errors, text.into());
    }

    pub fn push_warning(&mut self, text: impl Into<String>) {
        push_unique_line(&mut self.warnings, text.into());
    }

    pub fn push_optional(&mut self, text: Option<String>) {
        if let Some(text) = text {
            self.push_fragment(text);
        }
    }

    pub fn is_empty(&self) -> bool {
        self.errors.is_empty() && self.warnings.is_empty()
    }

    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }

    pub fn errors(&self) -> &[String] {
        &self.errors
    }

    pub fn warnings(&self) -> &[String] {
        &self.warnings
    }

    pub fn from_journal_and_text(
        combined_text: &str,
        journal: &IssueJournal,
        max_per_severity: usize,
    ) -> Self {
        let mut board = Self::from_combined_text(combined_text);
        append_journal_to_board(journal, &mut board, max_per_severity);
        board
    }

    pub fn merge_journal(&mut self, journal: &IssueJournal, max_per_severity: usize) {
        append_journal_to_board(journal, self, max_per_severity);
    }

    pub fn show_top(&self, ctx: &egui::Context, app_id: &str, error_title: &str, warning_title: &str) {
        self.show_in_panel(ctx, egui::Id::new("stack_issue_banner"), app_id, error_title, warning_title);
    }

    /// Bandeau issues pour fenêtres egui secondaires (viewport dupliqué, etc.).
    pub fn show_in_panel(
        &self,
        ctx: &egui::Context,
        panel_id: impl Into<egui::Id>,
        app_id: &str,
        error_title: &str,
        warning_title: &str,
    ) {
        if self.is_empty() {
            return;
        }
        show_issue_panel(
            ctx,
            panel_id.into(),
            app_id,
            error_title,
            &self.errors,
            warning_title,
            &self.warnings,
        );
        ctx.request_repaint_after(Duration::from_millis(33));
    }
}

pub fn classify_issue_severity(text: &str) -> IssueSeverity {
    let u = text.to_uppercase();
    const WARN_ONLY: &[&str] = &[
        "ORDERBOOK_RECEPTION_LATE",
        "MARKET_DATA_FLAT_BUT_WS_ALIVE",
        "WS SANS DONNÉES",
        "WS SANS DONNEES",
        "ATTENTE CARNET",
        "ORDER_BLOCKED",
        "WS_TRADING_OFF",
        "_SNAPSHOT_EMPTY",
        "REFERENCE_SNAPSHOT_EMPTY",
        "OKX_SNAPSHOT_EMPTY",
        "KRAKEN_SNAPSHOT_EMPTY",
        "COINBASE_SNAPSHOT_EMPTY",
        "SYMBOL_RULES_EMPTY",
    ];
    if WARN_ONLY.iter().any(|k| u.contains(k)) {
        return IssueSeverity::Warning;
    }
    const ERROR_MARKERS: &[&str] = &[
        "CRITIQUE",
        "CRITICAL",
        "FAILED",
        "FAIL",
        "ERROR",
        "UNAVAILABLE",
        "BLOQU",
        "INDISPONIBLE",
        "429",
        "RATE LIMIT",
        "HARD_STALE",
        "ORDERBOOK_CROSSED",
        "ORDER_FAILED",
        "WS_UNAVAILABLE",
        "WS_TRADING",
        "ADD_ORDER",
        "REST_ERROR",
        "WS_ERROR",
        "WS_CONNECT_FAILED",
        "SUBSCRIBE_FAILED",
        "TOKIO_RUNTIME",
        "STATICS",
        "CONNECTIVITY",
        "IGNORÉ",
        "IGNORED",
        "DOWN",
        "OFFLINE",
        "MANQUANT",
    ];
    if ERROR_MARKERS.iter().any(|k| u.contains(k)) {
        IssueSeverity::Error
    } else {
        IssueSeverity::Warning
    }
}

pub fn push_unique_line(lines: &mut Vec<String>, text: String) {
    let text = text.trim();
    if text.is_empty() || lines.iter().any(|l| l == text) {
        return;
    }
    lines.push(text.to_string());
}

pub fn append_classified_fragment(
    errors: &mut Vec<String>,
    warnings: &mut Vec<String>,
    text: String,
) {
    let text = text.trim();
    if text.is_empty() {
        return;
    }
    match classify_issue_severity(text) {
        IssueSeverity::Error => push_unique_line(errors, text.to_string()),
        IssueSeverity::Warning => push_unique_line(warnings, text.to_string()),
    }
}

pub fn split_banner_fragments(combined: &str) -> Vec<String> {
    combined
        .split('·')
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_string)
        .collect()
}

pub fn append_journal_to_board(
    journal: &IssueJournal,
    board: &mut StackIssueBoard,
    max_per_severity: usize,
) {
    for (severity, text) in journal.active_records() {
        match severity {
            IssueSeverity::Error if board.errors().len() < max_per_severity => {
                board.push_error(text.to_string());
            }
            IssueSeverity::Warning if board.warnings().len() < max_per_severity => {
                board.push_warning(text.to_string());
            }
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classify_transient_ws_wait_as_warning() {
        assert_eq!(
            classify_issue_severity("WS sans données — kraken ADA/EUR (attente carnet)"),
            IssueSeverity::Warning
        );
    }

    #[test]
    fn classify_connect_failed_as_error() {
        assert_eq!(
            classify_issue_severity("WS_CONNECT_FAILED symbol=ADA/EUR err=429"),
            IssueSeverity::Error
        );
    }

    #[test]
    fn journal_dedupes_and_upgrades_severity() {
        let mut j = IssueJournal::new(Duration::from_secs(60), 8);
        j.push(IssueSeverity::Warning, "foo".into());
        j.push(IssueSeverity::Error, "foo".into());
        assert_eq!(j.records.len(), 1);
        assert_eq!(
            j.active_records().next().unwrap().0,
            IssueSeverity::Error
        );
    }

    #[test]
    fn classify_statics_and_connect_as_error() {
        assert_eq!(
            classify_issue_severity("STATICS ARRÊTÉ — publication expirée"),
            IssueSeverity::Error
        );
        assert_eq!(
            classify_issue_severity("WS_CONNECT_FAILED symbol=ADA/EUR err=429"),
            IssueSeverity::Error
        );
    }

    #[test]
    fn classify_order_blocked_as_warning() {
        assert_eq!(
            classify_issue_severity(
                "ORDER_BLOCKED | stage=preflight | venue=kraken | symbol=ETHEUR | reason=min"
            ),
            IssueSeverity::Warning
        );
    }

    #[test]
    fn classify_order_failed_as_error() {
        assert_eq!(
            classify_issue_severity("ORDER_FAILED | venue=kraken | error=disconnected"),
            IssueSeverity::Error
        );
    }
}
