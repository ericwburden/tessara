//! Summary components for the Operations feature.

use crate::features::operations::types::{OperationsSummary, ReportingDataStatus};
use leptos::prelude::*;

#[component]
pub(crate) fn OperationsSummaryPanel(
    summary: OperationsSummary,
    reporting_data: ReportingDataStatus,
) -> impl IntoView {
    view! {
        <section class="route-panel__section operations-summary" aria-label="Operations overview">
            <div class="metric-grid operations-action-metrics">
                <OperationsMetric label="Open workflow assignments" value=summary.open_workflow_assignment_count.to_string()/>
                <OperationsMetric label="Draft form responses" value=summary.draft_response_count.to_string()/>
                <OperationsMetric label="Datasets needing attention" value=summary.dataset_attention_count.to_string()/>
                <OperationsMetric label="Reporting data status" value=reporting_data.status.clone()/>
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
