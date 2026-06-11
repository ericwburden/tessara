//! Owns the features::forms module behavior.

pub(crate) mod api;
mod attached_nodes;
pub(crate) mod builder;
mod detail;
mod display;
mod list;
mod pages;
mod tables;
pub(crate) mod types;
mod versions;

pub(crate) use detail::{FormsDetailPage, FormsEditPage, FormsNewPage};
pub(crate) use display::{
    form_attached_nodes, form_attached_to_label, form_definition_scope_label,
    form_field_count_label, form_status_label, form_version_desc_sort_key,
    rendered_field_layout_label, rendered_field_type_label,
};
pub(crate) use list::FormsList;
pub(crate) use pages::FormsPage;
pub(crate) use types::{
    FormDatasetSourceLink, FormDefinition, FormSummary, FormVersionSummary, FormWorkflowLink,
    RenderedField, RenderedForm,
};
pub(crate) use versions::FormVersionsTable;
