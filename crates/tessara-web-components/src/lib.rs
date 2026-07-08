#![recursion_limit = "512"]

//! Public boundary for the Components feature.

mod api;
mod http;
mod pages;
mod types;

pub use pages::{
    ComponentEditorContent, ComponentVersionsContent, ComponentViewerContent,
    ComponentsIndexContent,
};
