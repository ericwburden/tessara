//! Form edit route page.

use crate::features::forms::builder::{FormBuilderEditorState, new_form_builder_editor_state};
use crate::features::forms::options_loader::load_form_edit_options;
use crate::features::forms::{FormDefinition, FormEditForm, FormSummary, RenderedForm};
use crate::features::organization::NodeTypeCatalogEntry;
use crate::types::route_params::{FormRouteParams, require_route_params};
use crate::ui::{
    AppShell, Breadcrumb, BreadcrumbItem, BreadcrumbLink, BreadcrumbPage, BreadcrumbSeparator,
    PageHeader, empty_view,
};
use leptos::prelude::*;

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
    let builder_state = new_form_builder_editor_state();
    let FormBuilderEditorState {
        builder_sections,
        active_builder_section,
        next_builder_section_id,
        builder_fields,
        next_builder_field_id,
        ..
    } = builder_state;
    let is_loading = RwSignal::new(true);
    let is_saving = RwSignal::new(false);
    let message = RwSignal::new(None::<String>);

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
                        view! {
                            <FormEditForm
                                form_id=form_id_for_submit
                                cancel_href=cancel_href.clone()
                                name=name
                                workflow_node_type_id=workflow_node_type_id
                                node_types=node_types
                                existing_forms=existing_forms
                                rendered_form=rendered_form
                                edit_version_id=edit_version_id
                                edit_version_status=edit_version_status
                                builder_state=builder_state
                                is_loading=is_loading
                                is_saving=is_saving
                                message=message
                            />
                        }
                        .into_any()
                    }
                }}
            </section>
        </AppShell>
    }
}
