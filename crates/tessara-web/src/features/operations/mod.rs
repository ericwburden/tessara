//! Public boundary for the Operations feature.
//!
//! Re-export only the pages, types, and helpers other modules need; keep Operations-specific implementation details in child modules.

mod api;
mod pages;
mod types;
pub(crate) use pages::OperationsPage;
