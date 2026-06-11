//! Public boundary for the Forms feature.
//!
//! Re-export only the pages, types, and helpers other modules need; keep Forms-specific implementation details in child modules.

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

pub(crate) use detail::FormsDetailPage;
pub(crate) use display::{
    form_attached_nodes, form_attached_to_label, form_definition_scope_label,
    form_field_count_label, form_status_label, form_version_desc_sort_key,
    rendered_field_layout_label, rendered_field_type_label,
};
pub(crate) use list::FormsList;
pub(crate) use pages::{FormsEditPage, FormsNewPage, FormsPage};
pub(crate) use types::{
    FormDatasetSourceLink, FormDefinition, FormSummary, FormVersionSummary, FormWorkflowLink,
    RenderedField, RenderedForm,
};
pub(crate) use versions::FormVersionsTable;
