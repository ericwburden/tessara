//! Owns the features::datasets module behavior.

mod api;
mod loaders;
mod pages;
mod types;
pub(crate) use pages::{
    DatasetsDetailPage, DatasetsEditPage, DatasetsNewPage, DatasetsPage, DatasetsPreviewPage,
};
