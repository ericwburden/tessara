//! CSS assets for the local Tessara shell.

/// Shared structural styles and vendor CSS applied to all themes.
pub const BASE_STYLE: &str = include_str!("../assets/base.css");

/// Light theme token definitions and light-specific overrides.
pub const LIGHT_STYLE: &str = include_str!("../assets/light.css");

/// Dark theme token definitions and dark-specific overrides.
pub const DARK_STYLE: &str = include_str!("../assets/dark.css");
