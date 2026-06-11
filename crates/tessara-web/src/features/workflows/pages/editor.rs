//! Owns the features::workflows::pages::editor module behavior.

use crate::features::forms::FormSummary;
#[cfg(feature = "hydrate")]
use crate::features::organization::current_search_param;
use crate::features::organization::{
    NodeTypeCatalogEntry, OrganizationNode, submit_create_workflow, submit_update_workflow,
    workflow_step_form_version_id_by_id, workflow_step_signature, workflow_step_title_by_id,
};
use crate::features::shared::{
    node_display_path, status_badge_class, workflow_form_version_options, workflow_step_form_label,
};
use crate::features::workflows::active_workflow_definition_version;
use crate::features::workflows::api::workflow_revision_label_from_raw as workflow_submission_workflow_revision_label_from_raw;
use crate::features::workflows::api::{load_workflow_create_options, load_workflow_detail};
use crate::features::workflows::types::{
    WorkflowDefinition, WorkflowSaveIntent, WorkflowStepDraft, WorkflowSummary,
};
use crate::types::route_params::{WorkflowRouteParams, require_route_params};
use crate::ui::{
    AppShell, Breadcrumb, BreadcrumbItem, BreadcrumbLink, BreadcrumbPage, BreadcrumbSeparator,
    PageHeader,
};
use crate::utils::text::sentence_label;
use icons::{ArrowDown, ArrowUp, Search, Trash2};
use leptos::prelude::*;
use std::collections::HashSet;

#[component]
/// Renders the workflow available nodes picker view.
fn WorkflowAvailableNodesPicker(
    nodes: Vec<OrganizationNode>,
    selected_node_ids: RwSignal<HashSet<String>>,
) -> impl IntoView {
    let search = RwSignal::new(String::new());
    let filtered_nodes = move || {
        let query = search.get();
        nodes
            .clone()
            .into_iter()
            .filter(|node| {
                crate::utils::text::text_matches(
                    &query,
                    &[
                        node.name.as_str(),
                        node.node_type_singular_label.as_str(),
                        node.parent_node_name.as_deref().unwrap_or(""),
                    ],
                )
            })
            .collect::<Vec<_>>()
    };

    view! {
        <div class="permission-picker workflow-available-node-picker">
            <label class="searchable-data-table__search searchable-data-table__control">
                <Search class="searchable-data-table__control-icon"/>
                <span class="sr-only">"Search available nodes"</span>
                <input
                    type="search"
                    placeholder="Search available nodes"
                    prop:value=move || search.get()
                    on:input=move |event| search.set(event_target_value(&event))
                />
            </label>
            <div class="checkbox-list permission-picker__list permission-picker__list--compact">
                {move || {
                    let nodes = filtered_nodes();
                    if nodes.is_empty() {
                        return view! {
                            <section class="organization-state">
                                <h3>"No nodes found"</h3>
                                <p>"Adjust the search to choose where this workflow is available."</p>
                            </section>
                        }
                        .into_any();
                    }

                    nodes
                        .into_iter()
                        .map(|node| {
                            let node_id = node.id.clone();
                            let node_id_for_checked = node_id.clone();
                            let node_id_for_change = node_id.clone();
                            let node_name = node.name.clone();
                            let node_type = node.node_type_singular_label.clone();
                            let node_path = node_display_path(&node);
                            view! {
                                <label class="checkbox-list__item permission-picker__item">
                                    <input
                                        type="checkbox"
                                        prop:checked=move || selected_node_ids.get().contains(&node_id_for_checked)
                                        on:change=move |event| {
                                            let checked = event_target_checked(&event);
                                            selected_node_ids.update(|ids| {
                                                if checked {
                                                    ids.insert(node_id_for_change.clone());
                                                } else {
                                                    ids.remove(&node_id_for_change);
                                                }
                                            });
                                        }
                                    />
                                    <span>
                                        <strong>{node_name}</strong>
                                        <small>{format!("{node_type} - {node_path}")}</small>
                                    </span>
                                </label>
                            }
                        })
                        .collect_view()
                        .into_any()
                }}
            </div>
        </div>
    }
}

#[component]
/// Renders the workflows new page view.
pub fn WorkflowsNewPage() -> impl IntoView {
    let node_types = RwSignal::new(Vec::<NodeTypeCatalogEntry>::new());
    let organization_nodes = RwSignal::new(Vec::<OrganizationNode>::new());
    let forms = RwSignal::new(Vec::<FormSummary>::new());
    let existing_workflows = RwSignal::new(Vec::<WorkflowSummary>::new());
    let seeded_from_form = RwSignal::new(false);
    let name = RwSignal::new(String::new());
    let available_node_ids = RwSignal::new(HashSet::<String>::new());
    let steps = RwSignal::new(Vec::<WorkflowStepDraft>::new());
    let next_step_id = RwSignal::new(1_usize);
    let description = RwSignal::new(String::new());
    let is_loading = RwSignal::new(true);
    let is_saving = RwSignal::new(false);
    let message = RwSignal::new(None::<String>);

    Effect::new(move |_| {
        load_workflow_create_options(
            node_types,
            organization_nodes,
            forms,
            existing_workflows,
            is_loading,
            message,
        );
    });

    Effect::new(move |_| {
        if is_loading.get() || seeded_from_form.get_untracked() {
            return;
        }

        let form_id: Option<String> = {
            #[cfg(feature = "hydrate")]
            {
                current_search_param("form_id")
            }
            #[cfg(not(feature = "hydrate"))]
            {
                None
            }
        };
        let Some(form_id) = form_id else {
            seeded_from_form.set(true);
            return;
        };

        let available_forms = forms.get();
        let Some(form) = available_forms.iter().find(|form| form.id == form_id) else {
            seeded_from_form.set(true);
            return;
        };
        let Some(version) = form
            .versions
            .iter()
            .find(|version| version.status == "published")
        else {
            seeded_from_form.set(true);
            return;
        };

        name.set(format!("{} Workflow", form.name));
        description.set(format!("Workflow for {}.", form.name));
        steps.set(vec![WorkflowStepDraft {
            id: 1,
            title: format!("{} Response", form.name),
            form_version_id: version.id.clone(),
        }]);
        next_step_id.set(2);
        seeded_from_form.set(true);
    });

    Effect::new(move |_| {
        if is_loading.get() {
            return;
        }
        let available_options = workflow_form_version_options(&forms.get(), &node_types.get(), "");
        steps.update(|steps| {
            steps.retain(|step| {
                step.form_version_id.is_empty()
                    || available_options
                        .iter()
                        .any(|(id, _, _)| id == &step.form_version_id)
            });
        });
    });

    let add_step = move || {
        let id = next_step_id.get_untracked();
        next_step_id.set(id + 1);
        steps.update(|steps| {
            steps.push(WorkflowStepDraft {
                id,
                title: format!("Step {}", steps.len() + 1),
                form_version_id: String::new(),
            });
        });
    };

    let can_submit = move || {
        !is_saving.get()
            && !name.get().trim().is_empty()
            && !available_node_ids.get().is_empty()
            && {
                let current_steps = steps.get();
                !current_steps.is_empty()
                    && current_steps
                        .iter()
                        .all(|step| !step.form_version_id.trim().is_empty())
            }
    };

    view! {
        <AppShell active_route="workflows" title="Workflows">
            <div class="app-page">
                <Breadcrumb>
                    <BreadcrumbItem>
                        <BreadcrumbLink href="/workflows">"Workflows"</BreadcrumbLink>
                    </BreadcrumbItem>
                    <BreadcrumbSeparator/>
                    <BreadcrumbItem>
                        <BreadcrumbPage>"Create Workflow"</BreadcrumbPage>
                    </BreadcrumbItem>
                </Breadcrumb>

                <section class="route-panel workflows-page">
                    <PageHeader title="Create Workflow"/>

                    {move || {
                        if is_loading.get() {
                            view! {
                                <section class="organization-state" aria-live="polite">
                                    <h3>"Loading workflow options"</h3>
                                    <p>"Fetching forms and workflow names."</p>
                                </section>
                            }
                            .into_any()
                        } else {
                            view! {
                                <form
                                    class="native-form workflow-create-form"
                                    on:submit=move |event| {
                                        event.prevent_default();
                                        submit_create_workflow(
                                            name,
                                            available_node_ids,
                                            steps,
                                            description,
                                            existing_workflows,
                                            is_saving,
                                            message,
                                        );
                                    }
                                >
                                    <div class="form-grid">
                                        <label class="form-field">
                                            <span>"Workflow Name"</span>
                                            <input
                                                type="text"
                                                value=move || name.get()
                                                on:input=move |event| {
                                                    name.set(event_target_value(&event));
                                                }
                                            />
                                        </label>
                                        <label class="form-field">
                                            <span>"Description"</span>
                                            <textarea
                                                prop:value=move || description.get()
                                                on:input=move |event| {
                                                    description.set(event_target_value(&event));
                                                }
                                            ></textarea>
                                        </label>
                                    </div>

                                    <section class="form-section">
                                        <h3>"Available At"</h3>
                                        <WorkflowAvailableNodesPicker
                                            nodes=organization_nodes.get()
                                            selected_node_ids=available_node_ids
                                        />
                                    </section>

                                    <section class="form-section">
                                        <div class="form-builder-section-card__header">
                                            <h3>"Workflow Steps"</h3>
                                            <button
                                                class="button button--secondary"
                                                type="button"
                                                disabled=move || {
                                                    workflow_form_version_options(
                                                        &forms.get(),
                                                        &node_types.get(),
                                                        "",
                                                    ).is_empty()
                                                }
                                                on:click=move |_| add_step()
                                            >
                                                "+ Add Step"
                                            </button>
                                        </div>
                                        {move || {
                                            let options = workflow_form_version_options(
                                                &forms.get(),
                                                &node_types.get(),
                                                "",
                                            );
                                            if options.is_empty() {
                                                return view! {
                                                    <section class="organization-state">
                                                        <h3>"No published forms available"</h3>
                                                        <p>"Publish at least one form version before creating a workflow."</p>
                                                    </section>
                                                }
                                                .into_any();
                                            }

                                            if steps.get().is_empty() {
                                                return view! {
                                                    <section class="organization-state">
                                                        <h3>"No workflow steps yet"</h3>
                                                        <p>"Add one or more form steps to define the workflow."</p>
                                                    </section>
                                                }
                                                .into_any();
                                            }

                                            view! {
                                                <div class="workflow-step-list">
                                                    <For
                                                        each=move || {
                                                            steps.get().into_iter().enumerate().collect::<Vec<_>>()
                                                        }
                                                        key=|(_, step)| step.id
                                                        children=move |(index, step)| {
                                                            let step_id = step.id;
                                                            let step_position = move || {
                                                                steps
                                                                    .get()
                                                                    .iter()
                                                                    .position(|step| step.id == step_id)
                                                                    .map(|index| index + 1)
                                                                    .unwrap_or(index + 1)
                                                            };
                                                            view! {
                                                                <article class="workflow-step-card">
                                                                    <header class="workflow-step-card__header">
                                                                        <span class="workflow-step-card__position">{move || format!("Step {}", step_position())}</span>
                                                                        <div class="workflow-step-card__actions">
                                                                            <button
                                                                                class="icon-button icon-button--control"
                                                                                type="button"
                                                                                title="Move step up"
                                                                                disabled=move || step_position() <= 1
                                                                                on:click=move |_| {
                                                                                    steps.update(|steps| {
                                                                                        if let Some(index) = steps.iter().position(|step| step.id == step_id)
                                                                                            && index > 0 {
                                                                                                steps.swap(index, index - 1);
                                                                                            }
                                                                                    });
                                                                                }
                                                                            >
                                                                                <ArrowUp/>
                                                                            </button>
                                                                            <button
                                                                                class="icon-button icon-button--control"
                                                                                type="button"
                                                                                title="Move step down"
                                                                                disabled=move || {
                                                                                    let step_count = steps.get().len();
                                                                                    step_position() >= step_count
                                                                                }
                                                                                on:click=move |_| {
                                                                                    steps.update(|steps| {
                                                                                        if let Some(index) = steps.iter().position(|step| step.id == step_id)
                                                                                            && index + 1 < steps.len() {
                                                                                                steps.swap(index, index + 1);
                                                                                            }
                                                                                    });
                                                                                }
                                                                            >
                                                                                <ArrowDown/>
                                                                            </button>
                                                                            <button
                                                                                class="icon-button icon-button--danger"
                                                                                type="button"
                                                                                title="Remove step"
                                                                                on:click=move |_| {
                                                                                    steps.update(|steps| {
                                                                                        steps.retain(|step| step.id != step_id);
                                                                                    });
                                                                                }
                                                                            >
                                                                                <Trash2/>
                                                                            </button>
                                                                        </div>
                                                                    </header>
                                                                    <div class="form-grid">
                                                                        <label class="form-field">
                                                                            <span>"Step Title"</span>
                                                                            <input
                                                                                type="text"
                                                                                prop:value=move || {
                                                                                    workflow_step_title_by_id(&steps.get(), step_id)
                                                                                }
                                                                                on:input=move |event| {
                                                                                    let value = event_target_value(&event);
                                                                                    steps.update(|steps| {
                                                                                        if let Some(step) = steps.iter_mut().find(|step| step.id == step_id) {
                                                                                            step.title = value;
                                                                                        }
                                                                                    });
                                                                                }
                                                                            />
                                                                        </label>
                                                                        <label class="form-field">
                                                                            <span>"Form Version"</span>
                                                                            <select
                                                                                prop:value=move || {
                                                                                    workflow_step_form_version_id_by_id(&steps.get(), step_id)
                                                                                }
                                                                                on:change=move |event| {
                                                                                    let value = event_target_value(&event);
                                                                                    steps.update(|steps| {
                                                                                        if let Some(step) = steps.iter_mut().find(|step| step.id == step_id) {
                                                                                            step.form_version_id = value;
                                                                                        }
                                                                                    });
                                                                                }
                                                                            >
                                                                                <option value="">"Select form version"</option>
                                                                                {workflow_form_version_options(
                                                                                    &forms.get(),
                                                                                    &node_types.get(),
                                                                                    "",
                                                                                )
                                                                                    .into_iter()
                                                                                    .map(|(id, label, _)| {
                                                                                        view! {
                                                                                            <option value=id>{label}</option>
                                                                                        }
                                                                                    })
                                                                                    .collect_view()}
                                                                            </select>
                                                                        </label>
                                                                    </div>
                                                                    <div class="workflow-step-card__footer">
                                                                        <span>{move || {
                                                                            let selected_form_version_id = steps
                                                                                .get()
                                                                                .into_iter()
                                                                                .find(|step| step.id == step_id)
                                                                                .map(|step| step.form_version_id)
                                                                                .unwrap_or_default();
                                                                            workflow_step_form_label(&forms.get(), &selected_form_version_id)
                                                                        }}</span>
                                                                    </div>
                                                                </article>
                                                            }
                                                        }
                                                    />
                                                </div>
                                            }
                                            .into_any()
                                        }}
                                    </section>

                                    {move || message.get().map(|message| view! {
                                        <p class="form-message" role="status">{message}</p>
                                    })}

                                    <div class="form-actions">
                                        <a class="button" href="/workflows">"Cancel"</a>
                                        <button class="button button--secondary" type="submit" disabled=move || !can_submit()>
                                            {move || if is_saving.get() { "Creating..." } else { "Create Workflow" }}
                                        </button>
                                    </div>
                                </form>
                            }
                            .into_any()
                        }
                    }}
                </section>
            </div>
        </AppShell>
    }
}

#[component]
/// Renders the workflows edit page view.
pub fn WorkflowsEditPage() -> impl IntoView {
    let params = require_route_params::<WorkflowRouteParams>();
    let workflow_id = params.workflow_id;
    let detail = RwSignal::new(None::<WorkflowDefinition>);
    let node_types = RwSignal::new(Vec::<NodeTypeCatalogEntry>::new());
    let organization_nodes = RwSignal::new(Vec::<OrganizationNode>::new());
    let forms = RwSignal::new(Vec::<FormSummary>::new());
    let existing_workflows = RwSignal::new(Vec::<WorkflowSummary>::new());
    let name = RwSignal::new(String::new());
    let slug = RwSignal::new(String::new());
    let available_node_ids = RwSignal::new(HashSet::<String>::new());
    let description = RwSignal::new(String::new());
    let steps = RwSignal::new(Vec::<WorkflowStepDraft>::new());
    let original_steps = RwSignal::new(Vec::<WorkflowStepDraft>::new());
    let next_step_id = RwSignal::new(1_usize);
    let edit_version_id = RwSignal::new(None::<String>);
    let edit_version_label = RwSignal::new(String::new());
    let edit_version_status = RwSignal::new(String::new());
    let version_is_draft = RwSignal::new(false);
    let initialized = RwSignal::new(false);
    let detail_loading = RwSignal::new(true);
    let options_loading = RwSignal::new(true);
    let detail_error = RwSignal::new(None::<String>);
    let message = RwSignal::new(None::<String>);
    let is_saving = RwSignal::new(false);
    let save_intent = RwSignal::new(None::<WorkflowSaveIntent>);

    {
        let workflow_id = workflow_id.clone();
        Effect::new(move |_| {
            load_workflow_detail(workflow_id.clone(), detail, detail_loading, detail_error);
        });
    }

    Effect::new(move |_| {
        load_workflow_create_options(
            node_types,
            organization_nodes,
            forms,
            existing_workflows,
            options_loading,
            message,
        );
    });

    Effect::new(move |_| {
        if initialized.get_untracked() {
            return;
        }
        let Some(workflow) = detail.get() else {
            return;
        };

        name.set(workflow.name.clone());
        slug.set(workflow.slug.clone());
        available_node_ids.set(
            workflow
                .available_nodes
                .iter()
                .map(|node| node.id.clone())
                .collect(),
        );
        description.set(workflow.description.clone());

        let requested_version_id = {
            #[cfg(feature = "hydrate")]
            {
                current_search_param("version_id")
            }
            #[cfg(not(feature = "hydrate"))]
            {
                None::<String>
            }
        };
        let edit_version = requested_version_id
            .as_ref()
            .and_then(|version_id| {
                workflow
                    .versions
                    .iter()
                    .find(|version| version.id == *version_id)
                    .cloned()
            })
            .or_else(|| active_workflow_definition_version(&workflow).cloned());

        edit_version_id.set(edit_version.as_ref().map(|version| version.id.clone()));
        edit_version_label.set(
            edit_version
                .as_ref()
                .and_then(|version| version.workflow_revision_label.clone())
                .as_deref()
                .map(workflow_submission_workflow_revision_label_from_raw)
                .unwrap_or_else(|| "-".to_string()),
        );
        edit_version_status.set(
            edit_version
                .as_ref()
                .map(|version| sentence_label(&version.status))
                .unwrap_or_else(|| "No revisions".to_string()),
        );
        version_is_draft.set(
            edit_version
                .as_ref()
                .map(|version| version.status.eq_ignore_ascii_case("draft"))
                .unwrap_or(false),
        );

        let mut step_summaries = edit_version
            .as_ref()
            .map(|version| version.steps.clone())
            .unwrap_or_default();
        step_summaries.sort_by_key(|step| step.position);
        let draft_steps = step_summaries
            .into_iter()
            .enumerate()
            .map(|(index, step)| WorkflowStepDraft {
                id: index + 1,
                title: step.title,
                form_version_id: step.form_version_id,
            })
            .collect::<Vec<_>>();
        let next_id = draft_steps.len() + 1;
        original_steps.set(draft_steps.clone());
        steps.set(draft_steps);
        next_step_id.set(next_id);
        initialized.set(true);
    });

    Effect::new(move |_| {
        if !initialized.get() {
            return;
        }
        if options_loading.get() {
            return;
        }
        let available_options = workflow_form_version_options(&forms.get(), &node_types.get(), "");
        steps.update(|steps| {
            steps.retain(|step| {
                step.form_version_id.is_empty()
                    || available_options
                        .iter()
                        .any(|(id, _, _)| id == &step.form_version_id)
            });
        });
    });

    let add_step = move || {
        let id = next_step_id.get_untracked();
        next_step_id.set(id + 1);
        steps.update(|steps| {
            steps.push(WorkflowStepDraft {
                id,
                title: format!("Step {}", steps.len() + 1),
                form_version_id: String::new(),
            });
        });
    };

    let can_submit = move || {
        if is_saving.get() || name.get().trim().is_empty() {
            return false;
        }
        if available_node_ids.get().is_empty() {
            return false;
        }
        let current_steps = steps.get();
        !current_steps.is_empty()
            && current_steps
                .iter()
                .all(|step| !step.form_version_id.trim().is_empty())
    };
    let has_step_changes = move || {
        workflow_step_signature(&steps.get()) != workflow_step_signature(&original_steps.get())
    };

    view! {
        <AppShell active_route="workflows" title="Workflows">
            <div class="app-page">
                <Breadcrumb>
                    <BreadcrumbItem>
                        <BreadcrumbLink href="/workflows">"Workflows"</BreadcrumbLink>
                    </BreadcrumbItem>
                    {move || detail.get().map(|workflow| view! {
                        <>
                            <BreadcrumbSeparator/>
                            <BreadcrumbItem>
                                <BreadcrumbLink href=format!("/workflows/{}", workflow.id)>{workflow.name}</BreadcrumbLink>
                            </BreadcrumbItem>
                        </>
                    })}
                    <BreadcrumbSeparator/>
                    <BreadcrumbItem>
                        <BreadcrumbPage>"Edit Workflow"</BreadcrumbPage>
                    </BreadcrumbItem>
                </Breadcrumb>

                <section class="route-panel workflows-page workflow-edit-page">
                    <PageHeader title="Edit Workflow"/>

                    {move || {
                        if detail_loading.get() || options_loading.get() {
                            view! {
                                <section class="organization-state" aria-live="polite">
                                    <h3>"Loading workflow"</h3>
                                    <p>"Fetching workflow details and form versions."</p>
                                </section>
                            }
                            .into_any()
                        } else if let Some(error) = detail_error.get() {
                            view! {
                                <section class="organization-state is-error" role="alert">
                                    <h3>"Workflow unavailable"</h3>
                                    <p>{error}</p>
                                </section>
                            }
                            .into_any()
                        } else {
                            let workflow_id_for_href = workflow_id.clone();
                            let workflow_id_for_submit = workflow_id.clone();
                            let workflow_id_for_publish = workflow_id.clone();
                            let workflow_href = format!("/workflows/{}", workflow_id_for_href);
                            view! {
                                <form
                                    class="native-form workflow-create-form"
                                    on:submit=move |event| {
                                        event.prevent_default();
                                        submit_update_workflow(
                                            workflow_id_for_submit.clone(),
                                            edit_version_id.get_untracked(),
                                            version_is_draft.get_untracked(),
                                            name,
                                            slug,
                                            available_node_ids,
                                            steps,
                                            original_steps,
                                            description,
                                            is_saving,
                                            save_intent,
                                            message,
                                            WorkflowSaveIntent::Draft,
                                        );
                                    }
                                >
                                    <div class="form-grid">
                                        <label class="form-field">
                                            <span>"Workflow Name"</span>
                                            <input
                                                type="text"
                                                value=move || name.get()
                                                on:input=move |event| {
                                                    name.set(event_target_value(&event));
                                                }
                                            />
                                        </label>
                                        <label class="form-field">
                                            <span>"Description"</span>
                                            <textarea
                                                prop:value=move || description.get()
                                                on:input=move |event| {
                                                    description.set(event_target_value(&event));
                                                }
                                            ></textarea>
                                        </label>
                                    </div>

                                    <section class="form-section">
                                        <h3>"Available At"</h3>
                                        <WorkflowAvailableNodesPicker
                                            nodes=organization_nodes.get()
                                            selected_node_ids=available_node_ids
                                        />
                                    </section>

                                    <section class="form-section">
                                        <h3>"Active Revision"</h3>
                                        <table class="info-list-table">
                                            <tbody>
                                                <tr>
                                                    <th scope="row">"Revision"</th>
                                                    <td>{move || edit_version_label.get()}</td>
                                                </tr>
                                                <tr>
                                                    <th scope="row">"Status"</th>
                                                    <td>{move || {
                                                        let status = edit_version_status.get();
                                                        let key = status.to_lowercase().replace(' ', "-");
                                                        view! { <span class=status_badge_class(&key)>{status}</span> }
                                                    }}</td>
                                                </tr>
                                            </tbody>
                                        </table>
                                    </section>

                                    <section class="form-section">
                                        <div class="form-builder-section-card__header">
                                            <h3>"Workflow Steps"</h3>
                                            <button
                                                class="button button--secondary"
                                                type="button"
                                                disabled=move || {
                                                    workflow_form_version_options(
                                                        &forms.get(),
                                                        &node_types.get(),
                                                        "",
                                                    )
                                                    .is_empty()
                                                }
                                                on:click=move |_| add_step()
                                            >
                                                "+ Add Step"
                                            </button>
                                        </div>

                                        {move || {
                                            if workflow_form_version_options(
                                                &forms.get(),
                                                &node_types.get(),
                                                "",
                                            ).is_empty() {
                                                return view! {
                                                    <section class="organization-state">
                                                        <h3>"No published forms available"</h3>
                                                        <p>"Publish at least one form version before editing workflow steps."</p>
                                                    </section>
                                                }
                                                .into_any();
                                            }

                                            if !version_is_draft.get() {
                                                view! {
                                                    <p class="form-message" role="status">
                                                        "Step changes will create a new draft workflow revision."
                                                    </p>
                                                }
                                                .into_any()
                                            } else {
                                                let _: () = view! { <></> };
                                                ().into_any()
                                            }
                                        }}

                                        {move || {
                                            if steps.get().is_empty() {
                                                return view! {
                                                    <section class="organization-state">
                                                        <h3>"No workflow steps"</h3>
                                                        <p>"This workflow revision does not have steps yet."</p>
                                                    </section>
                                                }
                                                .into_any();
                                            }

                                            view! {
                                                <div class="workflow-step-list">
                                                    <For
                                                        each=move || {
                                                            steps.get().into_iter().enumerate().collect::<Vec<_>>()
                                                        }
                                                        key=|(_, step)| step.id
                                                        children=move |(index, step)| {
                                                            let step_id = step.id;
                                                            let step_position = move || {
                                                                steps
                                                                    .get()
                                                                    .iter()
                                                                    .position(|step| step.id == step_id)
                                                                    .map(|index| index + 1)
                                                                    .unwrap_or(index + 1)
                                                            };
                                                            view! {
                                                                <article class="workflow-step-card">
                                                                    <header class="workflow-step-card__header">
                                                                        <span class="workflow-step-card__position">{move || format!("Step {}", step_position())}</span>
                                                                        <div class="workflow-step-card__actions">
                                                                            <button
                                                                                class="icon-button icon-button--control"
                                                                                type="button"
                                                                                title="Move step up"
                                                                                disabled=move || step_position() <= 1
                                                                                on:click=move |_| {
                                                                                    steps.update(|steps| {
                                                                                        if let Some(index) = steps.iter().position(|step| step.id == step_id)
                                                                                            && index > 0 {
                                                                                                steps.swap(index, index - 1);
                                                                                            }
                                                                                    });
                                                                                }
                                                                            >
                                                                                <ArrowUp/>
                                                                            </button>
                                                                            <button
                                                                                class="icon-button icon-button--control"
                                                                                type="button"
                                                                                title="Move step down"
                                                                                disabled=move || {
                                                                                    let step_count = steps.get().len();
                                                                                    step_position() >= step_count
                                                                                }
                                                                                on:click=move |_| {
                                                                                    steps.update(|steps| {
                                                                                        if let Some(index) = steps.iter().position(|step| step.id == step_id)
                                                                                            && index + 1 < steps.len() {
                                                                                                steps.swap(index, index + 1);
                                                                                            }
                                                                                    });
                                                                                }
                                                                            >
                                                                                <ArrowDown/>
                                                                            </button>
                                                                            <button
                                                                                class="icon-button icon-button--danger"
                                                                                type="button"
                                                                                title="Remove step"
                                                                                on:click=move |_| {
                                                                                    steps.update(|steps| {
                                                                                        steps.retain(|step| step.id != step_id);
                                                                                    });
                                                                                }
                                                                            >
                                                                                <Trash2/>
                                                                            </button>
                                                                        </div>
                                                                    </header>
                                                                    <div class="form-grid">
                                                                        <label class="form-field">
                                                                            <span>"Step Title"</span>
                                                                            <input
                                                                                type="text"
                                                                                prop:value=move || {
                                                                                    workflow_step_title_by_id(&steps.get(), step_id)
                                                                                }
                                                                                on:input=move |event| {
                                                                                    let value = event_target_value(&event);
                                                                                    steps.update(|steps| {
                                                                                        if let Some(step) = steps.iter_mut().find(|step| step.id == step_id) {
                                                                                            step.title = value;
                                                                                        }
                                                                                    });
                                                                                }
                                                                            />
                                                                        </label>
                                                                        <label class="form-field">
                                                                            <span>"Form Version"</span>
                                                                            <select
                                                                                prop:value=move || {
                                                                                    workflow_step_form_version_id_by_id(&steps.get(), step_id)
                                                                                }
                                                                                on:change=move |event| {
                                                                                    let value = event_target_value(&event);
                                                                                    steps.update(|steps| {
                                                                                        if let Some(step) = steps.iter_mut().find(|step| step.id == step_id) {
                                                                                            step.form_version_id = value;
                                                                                        }
                                                                                    });
                                                                                }
                                                                            >
                                                                                <option value="">"Select form version"</option>
                                                                                {workflow_form_version_options(
                                                                                    &forms.get(),
                                                                                    &node_types.get(),
                                                                                    "",
                                                                                )
                                                                                    .into_iter()
                                                                                    .map(|(id, label, _)| {
                                                                                        view! {
                                                                                            <option value=id>{label}</option>
                                                                                        }
                                                                                    })
                                                                                    .collect_view()}
                                                                            </select>
                                                                        </label>
                                                                    </div>
                                                                    <div class="workflow-step-card__footer">
                                                                        <span>{move || {
                                                                            let selected_form_version_id = steps
                                                                                .get()
                                                                                .into_iter()
                                                                                .find(|step| step.id == step_id)
                                                                                .map(|step| step.form_version_id)
                                                                                .unwrap_or_default();
                                                                            workflow_step_form_label(&forms.get(), &selected_form_version_id)
                                                                        }}</span>
                                                                    </div>
                                                                </article>
                                                            }
                                                        }
                                                    />
                                                </div>
                                            }
                                            .into_any()
                                        }}
                                    </section>

                                    {move || message.get().map(|message| view! {
                                        <p class="form-message" role="status">{message}</p>
                                    })}

                                    <div class="form-actions">
                                        <a class="button" href=workflow_href>"Cancel"</a>
                                        <button class="button button--secondary" type="submit" disabled=move || !can_submit()>
                                            {move || {
                                                if save_intent.get() == Some(WorkflowSaveIntent::Draft) {
                                                    "Saving..."
                                                } else if has_step_changes() {
                                                    "Save as Draft"
                                                } else {
                                                    "Save Changes"
                                                }
                                            }}
                                        </button>
                                        <button
                                            class="button button--secondary"
                                            type="button"
                                            disabled=move || {
                                                !can_submit()
                                                    || (!version_is_draft.get() && !has_step_changes())
                                            }
                                            on:click=move |_| {
                                                submit_update_workflow(
                                                    workflow_id_for_publish.clone(),
                                                    edit_version_id.get_untracked(),
                                                    version_is_draft.get_untracked(),
                                                    name,
                                                    slug,
                                                    available_node_ids,
                                                    steps,
                                                    original_steps,
                                                    description,
                                                    is_saving,
                                                    save_intent,
                                                    message,
                                                    WorkflowSaveIntent::Publish,
                                                );
                                            }
                                        >
                                            {move || {
                                                if save_intent.get() == Some(WorkflowSaveIntent::Publish) {
                                                    "Publishing..."
                                                } else {
                                                    "Save and Publish"
                                                }
                                            }}
                                        </button>
                                    </div>
                                </form>
                            }
                            .into_any()
                        }
                    }}
                </section>
            </div>
        </AppShell>
    }
}
