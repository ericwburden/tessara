#![recursion_limit = "512"]

//! Dashboard-owned, route-free execution of exact Component versions.
//!
//! This consumer leaf owns Dashboard placement execution and presentation for
//! every published Component kind without depending on a provider feature crate.

mod api;
mod http;
mod request;
mod types;
mod viewer;
mod visual;

pub use types::{ComponentStatValue, ComponentVisual, ComponentVisualPoint, ComponentVisualSlice};
pub use viewer::{
    ComponentRequestActivity, ComponentRequestActivityCallback, ComponentTablePresentation,
    ComponentVersionExecutionContent, ComponentVersionKind, ComponentVersionTarget,
    ComponentViewerMode,
};
pub use visual::ComponentVisualPresentation;
