//! Owns the routes module behavior.

pub mod administration;
pub mod components;
pub mod dashboards;
pub mod datasets;
pub mod forms;
pub mod home;
pub mod login;
pub mod not_found;
pub mod operations;
pub mod organization;
pub mod responses;
pub mod workflows;

use leptos_router::SsrMode;

pub use not_found::NotFoundPage;

pub const PRIMARY_SSR_MODE: SsrMode = SsrMode::InOrder;

/// Handles the routes behavior.
pub fn routes() -> impl leptos_router::MatchNestedRoutes + Clone {
    (
        home::home_routes(),
        login::login_routes(),
        organization::organization_routes(),
        forms::form_routes(),
        workflows::workflow_routes(),
        responses::response_routes(),
        operations::operation_routes(),
        components::component_routes(),
        dashboards::dashboard_routes(),
        datasets::dataset_routes(),
        administration::administration_routes(),
    )
}
