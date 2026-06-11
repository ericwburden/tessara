//! Application state module registry.
//!
//! Keep cross-route state boundaries such as navigation metadata, shell session context, and theme constants here instead of feature-local modules.

pub mod navigation;

pub mod session;
pub mod theme;
