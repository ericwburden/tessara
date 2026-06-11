//! Public boundary for the Responses feature.
//!
//! Re-export only the pages, types, and helpers other modules need; keep Responses-specific implementation details in child modules.

mod api;
mod detail;
pub(crate) mod display;
mod edit;
mod list;
mod pages;
mod start;
pub(crate) mod types;
pub(crate) mod value_collection;
pub(crate) use pages::{ResponsesDetailPage, ResponsesEditPage, ResponsesNewPage, ResponsesPage};
