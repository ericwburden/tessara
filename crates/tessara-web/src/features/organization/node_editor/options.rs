//! Option selection helpers for organization node editor loaders.

use crate::features::organization::node_options::available_node_types_for_parent;
use crate::features::organization::types::{NodeTypeCatalogEntry, OrganizationNode};

/// Selected create-page node type and parent derived from URL hints and available options.
pub(super) struct OrganizationCreateSelection {
    pub(super) node_type_id: String,
    pub(super) parent_node_id: String,
}

/// Chooses valid initial create-page selections from requested URL values.
pub(super) fn organization_create_selection(
    requested_node_type_id: Option<String>,
    requested_parent_id: Option<String>,
    node_types: &[NodeTypeCatalogEntry],
    nodes: &[OrganizationNode],
) -> OrganizationCreateSelection {
    let selected_parent = requested_parent_id
        .filter(|requested| nodes.iter().any(|node| node.id == *requested))
        .unwrap_or_default();
    let available_types = available_node_types_for_parent(&selected_parent, node_types, nodes);
    let selected_type = requested_node_type_id
        .filter(|requested| {
            available_types
                .iter()
                .any(|node_type| node_type.id == *requested)
        })
        .or_else(|| {
            available_types
                .first()
                .map(|node_type| node_type.id.clone())
        })
        .unwrap_or_default();

    OrganizationCreateSelection {
        node_type_id: selected_type,
        parent_node_id: selected_parent,
    }
}
