//! Public boundary for the Shared feature.
//!
//! Re-export only the pages, types, and helpers other modules need; keep Shared-specific implementation details in child modules.

mod display;
mod placeholder;
pub(crate) use display::status_badge_class;
pub(crate) use placeholder::NativePlaceholderRoute;
