//! Owns the features::responses module behavior.

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
