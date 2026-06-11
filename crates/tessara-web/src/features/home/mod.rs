//! Public boundary for the Home feature.
//!
//! Re-export only the pages, types, and helpers other modules need; keep Home-specific implementation details in child modules.

mod pages;

pub(crate) use pages::HomePage;
