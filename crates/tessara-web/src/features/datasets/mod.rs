//! Public boundary for the Datasets feature.
//!
//! Re-export only the pages, types, and helpers other modules need; keep Datasets-specific implementation details in child modules.

mod actions;
mod api;
mod components;
mod display;
mod editor;
mod expressions;
mod loaders;
mod pages;
mod permissions;
mod types;
mod validation;
pub(crate) use pages::{
    DatasetsDetailPage, DatasetsEditPage, DatasetsNewPage, DatasetsPage, DatasetsPreviewPage,
};
