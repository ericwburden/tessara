//! Core-owned module discovery persistence boundary.
//!
//! Sprint 6A synchronizes transition descriptors only. It deliberately exposes
//! no Module Release/Instance repository or mutation path.

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
pub(crate) use service::{project_composition_modules, synchronize_catalog};

pub(crate) fn routes() -> axum::Router<crate::db::AppState> {
    routes::routes().merge(shell_navigation::routes())
}
