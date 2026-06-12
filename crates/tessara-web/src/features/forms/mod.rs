//! Public boundary for the Forms feature.
//!
//! Re-export only the pages, types, and helpers other modules need; keep Forms-specific implementation details in child modules.

mod api;
mod attached_nodes;
pub(crate) mod builder;
mod components;
mod create;
mod detail;
mod detail_content;
mod display;
mod edit;
mod edit_form;
mod editor_sections;
mod filtering;
mod list;
mod loaders;
mod options_loader;
mod pages;
mod save;
mod tables;
pub(crate) mod types;
mod versions;
mod versions_table;

pub(in crate::features::forms) use attached_nodes::{
    FormsAttachedNodesList, FormsAttachedNodesSheet,
};
pub(crate) use create::FormsNewPage;
pub(crate) use detail::FormsDetailPage;
pub(in crate::features::forms) use detail_content::FormDetailContent;
pub(crate) use display::{
    form_attached_nodes, form_attached_to_label, form_definition_scope_label,
    form_field_count_label, form_status_label, rendered_field_layout_label,
    rendered_field_type_label,
};
pub(crate) use edit::FormsEditPage;
pub(in crate::features::forms) use edit_form::FormEditForm;
pub(in crate::features::forms) use editor_sections::{
    FormEditableVersionSummary, FormIdentityFields, FormInitialVersionSummary,
};
pub(crate) use filtering::{
    FormNodeFilterOption, form_matches_node_filter, form_node_filter_options, indented_node_label,
    visible_form_node_filter_options,
};
pub(crate) use pages::FormsPage;
pub(crate) use types::{
    FormDatasetSourceLink, FormDefinition, FormSummary, FormVersionSummary, FormWorkflowLink,
    RenderedField, RenderedForm,
};
pub(crate) use versions::{
    active_form_definition_version, active_form_version, form_version_label,
    form_version_sort_label,
};
pub(crate) use versions_table::FormVersionsTable;
