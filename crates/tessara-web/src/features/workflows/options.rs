//! Option list helpers for workflow editor loaders.

use crate::features::forms::FormSummary;
use crate::features::organization::{NodeTypeCatalogEntry, OrganizationNode};
use crate::features::shared::node_display_path;
use crate::features::workflows::types::WorkflowSummary;

/// Selectable workflow editor options after applying display ordering.
pub(super) struct WorkflowEditorOptions {
    pub(super) node_types: Vec<NodeTypeCatalogEntry>,
    pub(super) organization_nodes: Vec<OrganizationNode>,
    pub(super) forms: Vec<FormSummary>,
    pub(super) workflows: Vec<WorkflowSummary>,
}

/// Applies the stable display ordering used by workflow editor pickers.
pub(super) fn ordered_workflow_editor_options(
    mut node_types: Vec<NodeTypeCatalogEntry>,
    mut organization_nodes: Vec<OrganizationNode>,
    mut forms: Vec<FormSummary>,
    mut workflows: Vec<WorkflowSummary>,
) -> WorkflowEditorOptions {
    node_types.sort_by(|left, right| {
        left.singular_label
            .cmp(&right.singular_label)
            .then(left.name.cmp(&right.name))
    });
    forms.sort_by(|left, right| left.name.cmp(&right.name).then(left.slug.cmp(&right.slug)));
    organization_nodes.sort_by(|left, right| {
        node_display_path(left)
            .cmp(&node_display_path(right))
            .then(left.name.cmp(&right.name))
    });
    workflows.sort_by(|left, right| left.name.cmp(&right.name).then(left.slug.cmp(&right.slug)));

    WorkflowEditorOptions {
        node_types,
        organization_nodes,
        forms,
        workflows,
    }
}
