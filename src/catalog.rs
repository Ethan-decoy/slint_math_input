//! Shared, inspectable catalog of mathematical input actions.
//!
//! Entries describe notation and editing actions; they do not evaluate expressions.
use crate::{Action, Template};
use serde::Deserialize;
use std::sync::OnceLock;

/// One function, symbol or structural insertion in the mouse catalog.
#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CatalogEntry {
    /// Stable identifier accepted by [`crate::Editor::insert_catalog`].
    pub id: String,
    /// Human-readable group heading.
    pub category: String,
    /// Short button label.
    pub label: String,
    /// Accessible description of the insertion.
    pub description: String,
    /// Atomic editing action, shared by GUI and non-GUI hosts.
    pub action: Action,
}

/// All built-in functions, symbols and templates, in display order.
pub fn entries() -> &'static [CatalogEntry] {
    static ENTRIES: OnceLock<Vec<CatalogEntry>> = OnceLock::new();
    ENTRIES.get_or_init(|| {
        serde_json::from_str(include_str!("../assets/math-catalog.json"))
            .expect("bundled mathematical catalog is validated by the coverage tests")
    })
}

/// Look up a stable catalog identifier.
pub fn entry(id: &str) -> Option<&'static CatalogEntry> {
    entries().iter().find(|entry| entry.id == id)
}

pub(crate) fn function_names() -> impl Iterator<Item = &'static str> {
    entries().iter().filter_map(|entry| match &entry.action {
        Action::Function(name) | Action::Template(Template::Limit(name)) => Some(name.as_str()),
        Action::Template(Template::LogBase) => Some("log"),
        _ => None,
    })
}

// This is a LaTeX/amsopn vocabulary, separate from the application's function catalog.
pub(crate) const STANDARD_FUNCTIONS: &[&str] = &[
    "arccos", "arcsin", "arctan", "arg", "cos", "cosh", "cot", "coth", "csc", "deg", "det", "dim",
    "exp", "gcd", "hom", "inf", "ker", "lg", "lim", "liminf", "limsup", "ln", "log", "max", "min",
    "Pr", "sec", "sin", "sinh", "sup", "tan", "tanh",
];
