#![recursion_limit = "512"]

//! Public boundary for the Datasets feature.
//!
//! Re-export only the pages, types, and helpers other modules need; keep Datasets-specific implementation details in child modules.

mod actions;
mod api;
mod components;
mod display;
mod editor;
mod expressions;
mod http;
mod loaders;
mod pages;
mod pagination;
#[cfg(feature = "hydrate")]
mod payloads;
mod permissions;
mod text;
mod types;
mod validation;
pub use pages::{
    DatasetDetailContent, DatasetEditorContent, DatasetPreviewContent, DatasetsIndexContent,
};
