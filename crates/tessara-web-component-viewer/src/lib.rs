#![recursion_limit = "512"]

//! Reusable, route-free execution of exact Component versions.
//!
//! This audited leaf crate owns reader-side execution state and presentation
//! for every published Component kind without depending on a feature crate.

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
