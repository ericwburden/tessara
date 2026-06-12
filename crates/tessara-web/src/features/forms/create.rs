//! Form creation route page.

use crate::features::forms::builder::{
    FormBuilderCanvas, FormBuilderEditorState, new_form_builder_editor_state,
};
use crate::features::forms::options_loader::load_form_create_options;
use crate::features::forms::{FormIdentityFields, FormInitialVersionSummary, FormSummary};
use crate::features::organization::NodeTypeCatalogEntry;
use crate::ui::{
    AppShell, Breadcrumb, BreadcrumbItem, BreadcrumbLink, BreadcrumbPage, BreadcrumbSeparator,
    Button, PageHeader,
};
use leptos::prelude::*;

use super::save::submit_create_form;

#[component]
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
                                <FormIdentityFields
                                    name=name
                                    workflow_node_type_id=workflow_node_type_id
                                    node_types=node_types
                                />

                                <FormInitialVersionSummary builder_field_count=builder_field_count/>

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
