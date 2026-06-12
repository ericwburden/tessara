//! Organization node creation and editing workflows.
//!
//! Keep route pages, load orchestration, and submit actions in focused child modules.

mod actions;
mod loaders;
mod pages;

pub(crate) use super::node_metadata::MetadataFieldInput;
pub(crate) use super::node_options::{
    available_node_types_for_parent, parent_node_options, parent_node_options_for_edit,
};
pub(crate) use actions::{submit_create_node, submit_update_node};
pub(crate) use loaders::{
    load_node_type_metadata, load_organization_create_options, load_organization_edit_options,
};
pub(crate) use pages::{OrganizationEditPage, OrganizationNewPage};
