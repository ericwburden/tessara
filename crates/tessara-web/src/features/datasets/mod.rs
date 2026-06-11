//! Public boundary for the Datasets feature.
//!
//! Re-export only the pages, types, and helpers other modules need; keep Datasets-specific implementation details in child modules.

mod api;
mod loaders;
mod pages;
mod types;
pub(crate) use pages::{
    DatasetsDetailPage, DatasetsEditPage, DatasetsNewPage, DatasetsPage, DatasetsPreviewPage,
};
