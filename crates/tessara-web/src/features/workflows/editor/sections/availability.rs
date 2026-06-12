//! Workflow editor availability section.

use crate::features::organization::OrganizationNode;
use leptos::prelude::*;
use std::collections::HashSet;

use super::super::WorkflowAvailableNodesPicker;

#[component]
pub(in crate::features::workflows) fn WorkflowAvailabilitySection(
    organization_nodes: RwSignal<Vec<OrganizationNode>>,
    available_node_ids: RwSignal<HashSet<String>>,
) -> impl IntoView {
    view! {
        <section class="form-section">
            <h3>"Available At"</h3>
            <WorkflowAvailableNodesPicker
                nodes=organization_nodes.get()
                selected_node_ids=available_node_ids
            />
        </section>
    }
}
