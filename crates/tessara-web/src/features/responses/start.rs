//! Response start page components.
//!
//! Keep the workflow/form launch flow for new responses here; response editing and detail presentation belong in sibling modules.

use super::actions::start_assignment_response_and_navigate;
use super::components::ResponseAssignmentStartForm;
use super::loaders::load_response_start_options;
use crate::features::responses::types::AssignmentResponseStartOptions;
use crate::ui::{
    AppShell, Breadcrumb, BreadcrumbItem, BreadcrumbLink, BreadcrumbPage, BreadcrumbSeparator,
    PageHeader,
};
#[cfg(feature = "hydrate")]
use crate::utils::url::current_search_param;

use leptos::prelude::*;

#[component]
pub(super) fn ResponsesNewPageContent() -> impl IntoView {
    let options = RwSignal::new(None::<AssignmentResponseStartOptions>);
    let is_loading = RwSignal::new(true);
    let is_saving = RwSignal::new(false);
    let message = RwSignal::new(None::<String>);
    let selected_assignment_index = RwSignal::new(String::new());

    #[cfg(feature = "hydrate")]
    let requested_workflow_assignment_id = current_search_param("workflowAssignmentId")
        .or_else(|| current_search_param("workflow_assignment_id"));
    #[cfg(not(feature = "hydrate"))]
    let requested_workflow_assignment_id = None::<String>;

    let requested_workflow_assignment_id_for_effect = requested_workflow_assignment_id.clone();
    let requested_workflow_assignment_id_for_view = requested_workflow_assignment_id.clone();
    #[cfg(feature = "hydrate")]
    let delegate_account_id_for_effect = current_search_param("delegateAccountId")
        .or_else(|| current_search_param("delegate_account_id"));
    #[cfg(not(feature = "hydrate"))]
    let delegate_account_id_for_effect = None::<String>;

    Effect::new(move |_| {
        if let Some(workflow_assignment_id) = requested_workflow_assignment_id_for_effect.clone() {
            is_loading.set(false);
            start_assignment_response_and_navigate(workflow_assignment_id, is_saving, message);
        } else {
            load_response_start_options(
                options,
                is_loading,
                message,
                delegate_account_id_for_effect.clone(),
            );
        }
    });

    view! {
        <AppShell active_route="responses" title="Start Response">
            <div class="app-page">
                <Breadcrumb>
                    <BreadcrumbItem>
                        <BreadcrumbLink href="/responses">"Responses"</BreadcrumbLink>
                    </BreadcrumbItem>
                    <BreadcrumbSeparator/>
                    <BreadcrumbItem>
                        <BreadcrumbPage>"Start Response"</BreadcrumbPage>
                    </BreadcrumbItem>
                </Breadcrumb>
                <section class="route-panel responses-page">
                    <PageHeader title="Start Response"/>

                    {move || {
                        if requested_workflow_assignment_id_for_view.is_some() && is_saving.get() {
                            view! {
                                <section class="organization-state" aria-live="polite">
                                    <h3>"Starting assigned response"</h3>
                                    <p>"Creating a draft from the selected workflow assignment."</p>
                                </section>
                            }
                            .into_any()
                        } else if is_loading.get() {
                            view! {
                                <section class="organization-state" aria-live="polite">
                                    <h3>"Loading start options"</h3>
                                    <p>"Fetching available response contexts."</p>
                                </section>
                            }
                            .into_any()
                        } else if let Some(error) = message.get().filter(|_| options.get().is_none()) {
                            view! {
                                <section class="organization-state is-error" role="alert">
                                    <h3>"Response start unavailable"</h3>
                                    <p>{error}</p>
                                </section>
                            }
                            .into_any()
                        } else if let Some(loaded_options) = options.get() {
                            view! {
                                <ResponseAssignmentStartForm
                                    assignments=loaded_options.assignments
                                    options
                                    is_loading
                                    is_saving
                                    message
                                    selected_assignment_index
                                />
                            }
                            .into_any()
                        } else {
                            view! {
                                <section class="organization-state is-error" role="alert">
                                    <h3>"Response start unavailable"</h3>
                                    <p>"Response start options could not be loaded."</p>
                                </section>
                            }
                            .into_any()
                        }
                    }}
                </section>
            </div>
        </AppShell>
    }
}
