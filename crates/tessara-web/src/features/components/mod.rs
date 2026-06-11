//! Public boundary for the Components feature.
//!
//! Re-export only the pages, types, and helpers other modules need; keep Components-specific implementation details in child modules.

mod pages;

pub(crate) use pages::{ComponentsDetailPage, ComponentsPage};
