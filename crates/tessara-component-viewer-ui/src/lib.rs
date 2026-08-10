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

use leptos::prelude::*;

pub use types::{
    ComponentRenderResponse, ComponentStatValue, ComponentVisual, ComponentVisualPoint,
    ComponentVisualSlice,
};
pub use viewer::{
    ComponentRequestActivity, ComponentRequestActivityCallback, ComponentTablePresentation,
    ComponentVersionExecutionContent, ComponentVersionKind, ComponentVersionTarget,
    ComponentViewerMode,
};
pub use visual::ComponentVisualPresentation;

/// Renders one already-loaded canonical Component execution response.
///
/// Authoring previews and exact-version readers therefore share the same
/// Table and visual presentation implementations.
#[component]
pub fn ComponentRenderPresentation(response: ComponentRenderResponse) -> impl IntoView {
    match response {
        ComponentRenderResponse::Table(table) => {
            viewer::component_table_response_presentation(*table).into_any()
        }
        ComponentRenderResponse::Visual(visual) => {
            view! { <ComponentVisualPresentation visual=*visual/> }.into_any()
        }
    }
}
