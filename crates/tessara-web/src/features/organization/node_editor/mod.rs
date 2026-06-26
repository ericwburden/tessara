//! Organization node creation and editing workflows.
//!
//! Keep route pages, load orchestration, and submit actions in focused child modules.

mod actions;
#[cfg(feature = "hydrate")]
mod api;
mod create;
mod create_form;
mod create_surface;
mod edit;
mod edit_form;
mod edit_surface;
mod loaders;
mod metadata_section;
#[cfg(feature = "hydrate")]
mod options;
mod state;

pub(crate) use super::node_metadata::MetadataFieldInput;
pub(crate) use super::node_options::{
    available_node_types_for_parent, parent_node_options, parent_node_options_for_edit,
};
pub(crate) use actions::{
    SubmitCreateNodeInput, SubmitUpdateNodeInput, submit_create_node, submit_update_node,
};
pub(crate) use create::OrganizationNewPage;
use create_form::OrganizationNodeCreateForm;
use create_surface::OrganizationNodeCreateSurface;
pub(crate) use edit::OrganizationEditPage;
use edit_form::OrganizationNodeEditForm;
use edit_surface::OrganizationNodeEditSurface;
pub(crate) use loaders::{
    load_node_type_metadata, load_organization_create_options, load_organization_edit_options,
};
use metadata_section::OrganizationNodeMetadataSection;
use state::{OrganizationNodeCreateState, OrganizationNodeEditState};
