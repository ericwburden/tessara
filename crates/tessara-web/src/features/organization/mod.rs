//! Public boundary for the Organization feature.
//!
//! Re-export only the pages, types, and helpers other modules need; keep Organization-specific implementation details in child modules.

pub(crate) mod api;
mod detail;
mod node_editor;
mod node_metadata;
mod node_options;
pub(crate) mod pages;
mod related_work;
mod related_work_controls;
mod related_work_tables;
mod tree;
pub(crate) mod types;
pub(crate) use api::IntoNonemptyString;
#[cfg(feature = "hydrate")]
pub(crate) use api::current_search_param;
pub(crate) use pages::{
    OrganizationDetailPage, OrganizationEditPage, OrganizationNewPage, OrganizationPage,
};
pub(crate) use related_work::RelatedWorkPaginationFooter;
pub(crate) use types::{
    AdminRoleSummary, NodeMetadataFieldSummary, NodeTypeCatalogEntry, NodeTypeDefinition,
    NodeTypeFormLink, NodeTypeUpsertRequest, OrganizationNode,
};
#[cfg(feature = "hydrate")]
pub(crate) use types::{
    CreateNodeMetadataFieldRequest, IdResponse, UpdateNodeMetadataFieldRequest,
};
