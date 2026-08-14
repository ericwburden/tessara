//! Core-owned module discovery, enrolled release/instance inventory, and the
//! frozen transition-catalog boundary.

mod catalog;
mod destination;
mod dto;
mod error;
mod native;
mod navigation_catalog;
mod reference;
mod repository;
mod routes;
mod service;
mod shell_navigation;

pub(crate) use native::{detail as native_detail, directory as native_directory};
pub(crate) use service::{
    CompositionProjectionDocuments, project_bootstrap_module_security, project_composition_modules,
    synchronize_catalog,
};
pub(crate) use shell_navigation::load_context_navigation;

pub(crate) fn routes() -> axum::Router<crate::db::AppState> {
    routes::routes().merge(shell_navigation::routes())
}
