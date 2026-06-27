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
mod http;
mod list;
mod loaders;
mod options_loader;
mod pages;
mod save;
mod support;
mod tables;
pub(crate) mod types;
mod versions;
mod versions_table;

pub(in crate::features::forms) use attached_nodes::{
    FormsAttachedNodesList, FormsAttachedNodesSheet,
};
pub(crate) use create::FormNewContent;
pub(crate) use detail::FormDetailContent;
pub(in crate::features::forms) use detail_content::FormDetailBody;
pub(crate) use display::{
    FormWorkflowSourceMarker, form_attached_nodes, form_attached_to_label,
    form_definition_scope_label, form_field_count_label, form_status_label,
    form_workflow_revision_label_from_option, node_count_label, rendered_field_layout_label,
    rendered_field_type_label, status_badge_class,
};
pub(crate) use edit::FormEditContent;
pub(in crate::features::forms) use edit_form::FormEditForm;
pub(in crate::features::forms) use editor_sections::{
    FormEditableVersionSummary, FormIdentityFields, FormInitialVersionSummary,
};
pub(crate) use filtering::{
    FormNodeFilterOption, form_matches_node_filter, form_node_filter_options, indented_node_label,
    visible_form_node_filter_options,
};
pub(crate) use pages::FormsIndexContent;
pub(crate) use types::{
    FormAttachmentLink, FormDatasetSourceLink, FormDefinition, FormNodeTypeOption, FormSummary,
    FormVersionSummary, FormWorkflowLink, FormsAttachedNodesSheetData, RenderedForm,
};
pub(crate) use versions::{
    active_form_definition_version, active_form_version, form_version_label,
};
pub(crate) use versions_table::FormVersionsTable;
