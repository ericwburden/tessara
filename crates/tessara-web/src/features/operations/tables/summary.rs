//! Summary components for the Operations feature.

use crate::features::operations::types::{
    OperationsSummary, ResponseOwnerStatus, ResponseProviderResultState,
};
use leptos::prelude::*;

#[component]
pub(crate) fn OperationsSummaryPanel(
    summary: OperationsSummary,
    response_owner: ResponseOwnerStatus,
) -> impl IntoView {
    let response_status = match response_owner.state {
        ResponseProviderResultState::Available => response_owner.status.map_or_else(
            || "Unavailable".to_string(),
            |status| if status.ready { "Ready" } else { "Attention" }.to_string(),
        ),
        ResponseProviderResultState::Empty | ResponseProviderResultState::Unavailable => {
            "Unavailable".to_string()
        }
        ResponseProviderResultState::Undisclosed => "Restricted".to_string(),
    };
    view! {
        <section class="route-panel__section operations-summary" aria-label="Operations overview">
            <div class="metric-grid operations-action-metrics">
                <OperationsMetric label="Open workflow assignments" value=summary.open_workflow_assignment_count.to_string()/>
                <OperationsMetric label="Draft form responses" value=summary.draft_response_count.to_string()/>
                <OperationsMetric
                    label="Datasets needing attention"
                    value=summary.dataset_attention_count.map_or_else(|| "Unavailable".into(), |count| count.to_string())
                />
                <OperationsMetric label="Response owner status" value=response_status/>
            </div>
        </section>
    }
}

#[component]
fn OperationsMetric(label: &'static str, value: String) -> impl IntoView {
    view! {
        <div class="metric-card">
            <span>{label}</span>
            <strong>{value}</strong>
        </div>
    }
}
