//! Route-level page composition for the Operations feature.
//!
//! Keep Leptos page components that correspond directly to routes here; reusable widgets, API calls, and DTOs should live in sibling modules.

use crate::ui::{AppShell, EmptyState, PageHeader};
use leptos::prelude::*;

use super::loaders::load_operations_status;
use super::tables::{DatasetReadinessTable, OperationsSummaryPanel, WorkflowAssignmentsTable};
use super::types::*;

#[component]
pub fn OperationsPage() -> impl IntoView {
    let status = RwSignal::new(None::<OperationsStatus>);
    let is_loading = RwSignal::new(true);
    let load_error = RwSignal::new(None::<String>);

    Effect::new(move |_| {
        load_operations_status(status, is_loading, load_error);
    });

    view! {
        <AppShell active_route="operations" title="Operations">
            <section class="route-panel operations-page">
                <PageHeader
                    title="Operations"
                />

                {move || {
                    if is_loading.get() {
                        view! {
                            <section class="organization-state" aria-live="polite">
                                <h3>"Loading operations"</h3>
                                <p>"Fetching visible workflow assignments and dataset status."</p>
                            </section>
                        }
                        .into_any()
                    } else if let Some(message) = load_error.get() {
                        view! {
                            <section class="organization-state is-error" role="alert">
                                <h3>"Operations unavailable"</h3>
                                <p>{message}</p>
                            </section>
                        }
                        .into_any()
                    } else if let Some(loaded_status) = status.get() {
                        view! {
                            <OperationsSummaryPanel summary=loaded_status.summary.clone() reporting_data=loaded_status.reporting_data.clone()/>
                            <WorkflowAssignmentsTable assignments=loaded_status.workflow_assignments.clone()/>
                            <DatasetReadinessTable datasets=loaded_status.dataset_readiness.datasets.clone()/>
                        }
                        .into_any()
                    } else {
                        view! {
                            <EmptyState
                                title="Operations unavailable"
                                message="Workflow and dataset status could not be loaded."
                            />
                        }
                        .into_any()
                    }
                }}
            </section>
        </AppShell>
    }
}
