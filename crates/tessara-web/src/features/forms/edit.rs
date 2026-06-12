//! Form edit route page.

use crate::features::forms::builder::{
    FormBuilderCanvas, FormBuilderEditorState, new_form_builder_editor_state,
};
use crate::features::forms::options_loader::load_form_edit_options;
use crate::features::forms::{
    FormDefinition, FormEditableVersionSummary, FormIdentityFields, FormSummary, RenderedForm,
    submit_update_form,
};
use crate::features::organization::NodeTypeCatalogEntry;
use crate::types::route_params::{FormRouteParams, require_route_params};
use crate::ui::{
    AppShell, Breadcrumb, BreadcrumbItem, BreadcrumbLink, BreadcrumbPage, BreadcrumbSeparator,
    PageHeader, empty_view,
};
use leptos::prelude::*;

/// Renders the forms edit page view.
#[component]
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
                                    <FormIdentityFields
                                        name=name
                                        workflow_node_type_id=workflow_node_type_id
                                        node_types=node_types
                                    />

                                    <FormEditableVersionSummary
                                        edit_version_status=edit_version_status
                                        builder_field_count=builder_field_count
                                    />

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
