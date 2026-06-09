//! Theme-state primitives and constants for Tessara web.
//!
//! This module owns browser theme metadata currently consumed by shell bootstrap
//! and runtime toggles.

pub(crate) const STORAGE_KEY: &str = "tessara.themePreference";
pub(crate) const LIGHT_THEME_COLOR: &str = "#F8FAFC";
pub(crate) const DARK_THEME_COLOR: &str = "#0F172A";

/// Placeholder theme state type maintained for compatibility with existing imports.
#[allow(clippy::exhaustive_structs)]
pub(crate) struct ThemeState;
