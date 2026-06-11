//! Route-level page composition for the Forms feature.
//!
//! Keep Leptos page components that correspond directly to routes here; reusable widgets, API calls, and DTOs should live in sibling modules.

use crate::features::forms::api::{load_form_create_options, load_form_edit_options, load_forms};
use crate::features::forms::builder::{
    FormBuilderCanvas, FormBuilderEditorState, new_form_builder_editor_state,
};
use crate::features::forms::{FormDefinition, FormSummary, FormsList, RenderedForm};
use crate::features::forms::{form_attached_to_label, form_status_label};
use crate::features::organization::{
    NodeTypeCatalogEntry, active_form_version, form_version_label, submit_create_form,
    submit_update_form,
};
use crate::features::shared::{
    form_matches_node_filter, form_node_filter_options, status_badge_class, unique_filter_options,
};
use crate::types::route_params::{FormRouteParams, require_route_params};
use crate::ui::{
    AppShell, Breadcrumb, BreadcrumbItem, BreadcrumbLink, BreadcrumbPage, BreadcrumbSeparator,
    Button, InfoListTable, InfoRow, PageHeader, empty_view,
};
use crate::utils::text::{sentence_label, text_matches};
use leptos::prelude::*;

#[component]
/// Renders the forms page view.
pub fn FormsPage() -> impl IntoView {
    let forms = RwSignal::new(Vec::<FormSummary>::new());
    let search = RwSignal::new(String::new());
    let status_filter = RwSignal::new("all".to_string());
    let node_filter_query = RwSignal::new(String::new());
    let selected_node_id = RwSignal::new(None::<String>);
    let is_loading = RwSignal::new(true);
    let load_error = RwSignal::new(None::<String>);

    Effect::new(move |_| {
        load_forms(forms, is_loading, load_error);
    });

    let filtered_forms = move || {
        let query = search.get();
        let selected_status = status_filter.get();
        let selected_node = selected_node_id.get();
        let loaded_forms = forms.get();
        let node_options = form_node_filter_options(&loaded_forms);

        loaded_forms
            .into_iter()
            .filter(|form| {
                let active_version = active_form_version(form);
                let attached_to = form_attached_to_label(active_version);
                let status = form_status_label(active_version);
                let matches_status = selected_status == "all" || status == selected_status;
                let matches_node_filter =
                    form_matches_node_filter(form, selected_node.as_deref(), &node_options);
                if !matches_status || !matches_node_filter {
                    return false;
                }
                text_matches(
                    &query,
                    &[
                        &form.name,
                        &form.slug,
                        &attached_to,
                        &form_version_label(active_version),
                        &status,
                    ],
                )
            })
            .collect::<Vec<_>>()
    };

    let status_options = move || {
        unique_filter_options(
            forms
                .get()
                .iter()
                .map(|form| form_status_label(active_form_version(form)))
                .collect::<Vec<_>>(),
        )
    };
    let node_filter_options = move || form_node_filter_options(&forms.get());

    view! {
        <AppShell active_route="forms" title="Forms">
            <section class="route-panel forms-page">
                <PageHeader title="Forms">
                    <Button label="Create Form" href="/forms/new"/>
                </PageHeader>

                {move || {
                    if is_loading.get() {
                        view! {
                            <section class="organization-state" aria-live="polite">
                                <h3>"Loading forms"</h3>
                                <p>"Fetching available form definitions."</p>
                            </section>
                        }
                        .into_any()
                    } else if let Some(message) = load_error.get() {
                        view! {
                            <section class="organization-state is-error" role="alert">
                                <h3>"Forms unavailable"</h3>
                                <p>{message}</p>
                            </section>
                        }
                        .into_any()
                    } else {
                        view! {
                            <FormsList
                                forms=filtered_forms()
                                search
                                status_filter
                                node_filter_query
                                selected_node_id
                                status_options=status_options()
                                node_filter_options=node_filter_options()
                            />
                        }
                        .into_any()
                    }
                }}
            </section>
        </AppShell>
    }
}

#[component]
/// Renders the forms new page view.
pub fn FormsNewPage() -> impl IntoView {
    let node_types = RwSignal::new(Vec::<NodeTypeCatalogEntry>::new());
    let existing_forms = RwSignal::new(Vec::<FormSummary>::new());
    let name = RwSignal::new(String::new());
    let workflow_node_type_id = RwSignal::new(String::new());
    let FormBuilderEditorState {
        builder_sections,
        active_builder_section,
        next_builder_section_id,
        builder_fields,
        active_builder_field,
        dragged_builder_field,
        builder_drag_preview,
        pending_builder_drag_preview,
        builder_drag_preview_timeout,
        suppress_builder_field_click,
        next_builder_field_id,
    } = new_form_builder_editor_state();
    let is_loading = RwSignal::new(true);
    let is_saving = RwSignal::new(false);
    let message = RwSignal::new(None::<String>);
    let builder_field_count = Memo::new(move |_| builder_fields.get().len());

    Effect::new(move |_| {
        load_form_create_options(node_types, existing_forms, is_loading, message);
    });

    let can_submit = move || !is_loading.get() && !is_saving.get() && !name.get().trim().is_empty();

    view! {
        <AppShell active_route="forms" title="Forms">
            <Breadcrumb>
                <BreadcrumbItem>
                    <BreadcrumbLink href="/forms">"Forms"</BreadcrumbLink>
                </BreadcrumbItem>
                <BreadcrumbSeparator/>
                <BreadcrumbItem>
                    <BreadcrumbPage>"Create Form"</BreadcrumbPage>
                </BreadcrumbItem>
            </Breadcrumb>
            <section class="route-panel forms-page form-editor-panel">
                <PageHeader title="Create Form"/>

                {move || {
                    if is_loading.get() {
                        view! {
                            <section class="organization-state" aria-live="polite">
                                <h3>"Loading form options"</h3>
                                <p>"Fetching available organization scopes."</p>
                            </section>
                        }
                        .into_any()
                    } else {
                        view! {
                            <div class="form-create-workspace">
                            <form
                                class="native-form form-create-form"
                                on:submit=move |event| {
                                    event.prevent_default();
                                    submit_create_form(
                                        name,
                                        workflow_node_type_id,
                                        builder_sections,
                                        builder_fields,
                                        existing_forms,
                                        is_saving,
                                        message,
                                        false,
                                    );
                                }
                            >
                                <div class="form-grid">
                                    <label class="form-field form-field--wide" for="form-name">
                                        <span>"Form Name"</span>
                                        <input
                                            id="form-name"
                                            type="text"
                                            autocomplete="off"
                                            prop:value=move || name.get()
                                            on:input=move |event| name.set(event_target_value(&event))
                                            required
                                        />
                                    </label>

                                    <label class="form-field" for="form-scope-node-type">
                                        <span>"Scope"</span>
                                        <select
                                            id="form-scope-node-type"
                                            prop:value=move || workflow_node_type_id.get()
                                            on:change=move |event| workflow_node_type_id.set(event_target_value(&event))
                                        >
                                            <option value="">"No scope"</option>
                                            {move || {
                                                let mut options = node_types.get();
                                                options.sort_by(|left, right| {
                                                    left.singular_label
                                                        .cmp(&right.singular_label)
                                                        .then(left.name.cmp(&right.name))
                                                });
                                                options
                                                    .into_iter()
                                                    .map(|node_type| {
                                                        view! {
                                                            <option value=node_type.id>{node_type.singular_label}</option>
                                                        }
                                                    })
                                                    .collect_view()
                                            }}
                                        </select>
                                    </label>

                                </div>

                                <section class="form-section">
                                    <h3>"Initial Version"</h3>
                                    <InfoListTable>
                                        <InfoRow label="Status" value="Draft"/>
                                        <tr>
                                            <th scope="row">"Fields"</th>
                                            <td>
                                                {move || builder_field_count.get().to_string()}
                                            </td>
                                        </tr>
                                    </InfoListTable>
                                </section>

                                <FormBuilderCanvas state=FormBuilderEditorState {
                                    builder_sections,
                                    active_builder_section,
                                    next_builder_section_id,
                                    builder_fields,
                                    active_builder_field,
                                    dragged_builder_field,
                                    builder_drag_preview,
                                    pending_builder_drag_preview,
                                    builder_drag_preview_timeout,
                                    suppress_builder_field_click,
                                    next_builder_field_id,
                                }/>
                                {move || message.get().map(|message| view! {
                                    <p class="form-message" role="status">{message}</p>
                                })}

                                <div class="form-actions">
                                    <Button label="Cancel" href="/forms"/>
                                    <button class="button button--secondary" type="submit" disabled=move || !can_submit()>
                                        {move || if is_saving.get() { "Saving..." } else { "Save as Draft" }}
                                    </button>
                                    <button
                                        class="button"
                                        type="button"
                                        disabled=move || !can_submit()
                                        on:click=move |_| {
                                            submit_create_form(
                                                name,
                                                workflow_node_type_id,
                                                builder_sections,
                                                builder_fields,
                                                existing_forms,
                                                is_saving,
                                                message,
                                                true,
                                            );
                                        }
                                    >
                                        {move || if is_saving.get() { "Publishing..." } else { "Create and Publish" }}
                                    </button>
                                </div>
                            </form>
                            </div>
                        }
                        .into_any()
                    }
                }}
            </section>
        </AppShell>
    }
}

#[component]
/// Renders the forms edit page view.
pub fn FormsEditPage() -> impl IntoView {
    let params = require_route_params::<FormRouteParams>();
    let form_id = params.form_id;
    let form_id_for_load = form_id.clone();
    let form_id_for_submit = form_id.clone();
    let cancel_href = format!("/forms/{form_id}");
    let node_types = RwSignal::new(Vec::<NodeTypeCatalogEntry>::new());
    let existing_forms = RwSignal::new(Vec::<FormSummary>::new());
    let detail = RwSignal::new(None::<FormDefinition>);
    let rendered_form = RwSignal::new(None::<RenderedForm>);
    let edit_version_id = RwSignal::new(None::<String>);
    let edit_version_status = RwSignal::new(None::<String>);
    let name = RwSignal::new(String::new());
    let workflow_node_type_id = RwSignal::new(String::new());
    let FormBuilderEditorState {
        builder_sections,
        active_builder_section,
        next_builder_section_id,
        builder_fields,
        active_builder_field,
        dragged_builder_field,
        builder_drag_preview,
        pending_builder_drag_preview,
        builder_drag_preview_timeout,
        suppress_builder_field_click,
        next_builder_field_id,
    } = new_form_builder_editor_state();
    let is_loading = RwSignal::new(true);
    let is_saving = RwSignal::new(false);
    let message = RwSignal::new(None::<String>);
    let builder_field_count = Memo::new(move |_| builder_fields.get().len());

    Effect::new(move |_| {
        load_form_edit_options(
            form_id_for_load.clone(),
            node_types,
            existing_forms,
            detail,
            rendered_form,
            edit_version_id,
            edit_version_status,
            name,
            workflow_node_type_id,
            builder_sections,
            builder_fields,
            active_builder_section,
            next_builder_section_id,
            next_builder_field_id,
            is_loading,
            message,
        );
    });

    let can_submit = move || !is_loading.get() && !is_saving.get() && !name.get().trim().is_empty();

    view! {
        <AppShell active_route="forms" title="Forms">
            <Breadcrumb>
                <BreadcrumbItem>
                    <BreadcrumbLink href="/forms">"Forms"</BreadcrumbLink>
                </BreadcrumbItem>
                <BreadcrumbSeparator/>
                {move || {
                    detail
                        .get()
                        .map(|form| {
                            let href = format!("/forms/{}", form.id);
                            view! {
                                <BreadcrumbItem>
                                    <BreadcrumbLink href=href>{form.name}</BreadcrumbLink>
                                </BreadcrumbItem>
                                <BreadcrumbSeparator/>
                            }
                            .into_any()
                        })
                        .unwrap_or_else(empty_view)
                }}
                <BreadcrumbItem>
                    <BreadcrumbPage>"Edit Form"</BreadcrumbPage>
                </BreadcrumbItem>
            </Breadcrumb>

            <section class="route-panel forms-page form-editor-panel">
                <PageHeader title="Edit Form"/>

                {move || {
                    if is_loading.get() {
                        view! {
                            <section class="organization-state" aria-live="polite">
                                <h3>"Loading form"</h3>
                                <p>"Fetching form definition and editable version."</p>
                            </section>
                        }
                        .into_any()
                    } else {
                        let form_id_for_submit = form_id_for_submit.clone();
                        let form_id_for_draft_submit = form_id_for_submit.clone();
                        let form_id_for_publish_submit = form_id_for_submit.clone();
                        view! {
                            <div class="form-create-workspace">
                                <form
                                    class="native-form form-create-form"
                                    on:submit=move |event| {
                                        event.prevent_default();
                                        submit_update_form(
                                            form_id_for_draft_submit.clone(),
                                            name,
                                            workflow_node_type_id,
                                            builder_sections,
                                            builder_fields,
                                            existing_forms,
                                            edit_version_id,
                                            edit_version_status,
                                            rendered_form,
                                            is_saving,
                                            message,
                                            false,
                                        );
                                    }
                                >
                                    <div class="form-grid">
                                        <label class="form-field form-field--wide" for="form-name">
                                            <span>"Form Name"</span>
                                            <input
                                                id="form-name"
                                                type="text"
                                                autocomplete="off"
                                                prop:value=move || name.get()
                                                on:input=move |event| name.set(event_target_value(&event))
                                                required
                                            />
                                        </label>

                                        <label class="form-field" for="form-scope-node-type">
                                            <span>"Scope"</span>
                                            <select
                                                id="form-scope-node-type"
                                                prop:value=move || workflow_node_type_id.get()
                                                on:change=move |event| workflow_node_type_id.set(event_target_value(&event))
                                            >
                                                <option value="">"No scope"</option>
                                                {move || {
                                                    let mut options = node_types.get();
                                                    options.sort_by(|left, right| {
                                                        left.singular_label
                                                            .cmp(&right.singular_label)
                                                            .then(left.name.cmp(&right.name))
                                                    });
                                                    options
                                                        .into_iter()
                                                        .map(|node_type| {
                                                            view! {
                                                                <option value=node_type.id>{node_type.singular_label}</option>
                                                            }
                                                        })
                                                        .collect_view()
                                                }}
                                            </select>
                                        </label>
                                    </div>

                                    <section class="form-section">
                                        <h3>"Editable Version"</h3>
                                        <InfoListTable>
                                            <tr>
                                                <th scope="row">"Status"</th>
                                                <td>
                                                    {move || {
                                                        edit_version_status
                                                            .get()
                                                            .map(|status| {
                                                                view! {
                                                                    <span class=status_badge_class(&status)>
                                                                        {sentence_label(&status)}
                                                                    </span>
                                                                }
                                                                .into_any()
                                                            })
                                                            .unwrap_or_else(|| view! { <span>"Draft"</span> }.into_any())
                                                    }}
                                                </td>
                                            </tr>
                                            <tr>
                                                <th scope="row">"Fields"</th>
                                                <td>
                                                    {move || builder_field_count.get().to_string()}
                                                </td>
                                            </tr>
                                        </InfoListTable>
                                    </section>

                                    <FormBuilderCanvas state=FormBuilderEditorState {
                                        builder_sections,
                                        active_builder_section,
                                        next_builder_section_id,
                                        builder_fields,
                                        active_builder_field,
                                        dragged_builder_field,
                                        builder_drag_preview,
                                        pending_builder_drag_preview,
                                        builder_drag_preview_timeout,
                                        suppress_builder_field_click,
                                        next_builder_field_id,
                                    }/>
                                    {move || message.get().map(|message| view! {
                                        <p class="form-message" role="status">{message}</p>
                                    })}

                                    <div class="form-actions">
                                        <a class="button" href=cancel_href.clone()>"Cancel"</a>
                                        <button class="button button--secondary" type="submit" disabled=move || !can_submit()>
                                            {move || if is_saving.get() { "Saving..." } else { "Save as Draft" }}
                                        </button>
                                        <button
                                            class="button"
                                            type="button"
                                            disabled=move || !can_submit()
                                            on:click=move |_| {
                                                submit_update_form(
                                                    form_id_for_publish_submit.clone(),
                                                    name,
                                                    workflow_node_type_id,
                                                    builder_sections,
                                                    builder_fields,
                                                    existing_forms,
                                                    edit_version_id,
                                                    edit_version_status,
                                                    rendered_form,
                                                    is_saving,
                                                    message,
                                                    true,
                                                );
                                            }
                                        >
                                            {move || if is_saving.get() { "Publishing..." } else { "Save and Publish" }}
                                        </button>
                                    </div>
                                </form>
                            </div>
                        }
                        .into_any()
                    }
                }}
            </section>
        </AppShell>
    }
}
