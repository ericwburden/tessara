//! Public boundary for the Organization feature.
//!
//! Re-export only route content components; keep Organization-specific
//! implementation details in child modules.

mod detail;
mod http;
mod metadata;
mod node_editor;
mod node_metadata;
mod node_options;
mod pages;
mod pagination;
mod related_work;
mod related_work_controls;
mod related_work_tables;
mod text;
mod tree;
pub(crate) mod types;
mod url;

pub use pages::{
    OrganizationDetailContent, OrganizationIndexContent, OrganizationNodeCreateContent,
    OrganizationNodeEditContent,
};
