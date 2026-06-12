//! Option loader for the form edit page.

#[cfg(feature = "hydrate")]
use crate::features::forms::builder::hydrate_form_builder_from_rendered;
use crate::features::forms::builder::{FormBuilderFieldDraft, FormBuilderSectionDraft};
#[cfg(feature = "hydrate")]
use crate::features::forms::editable_form_definition_version;
use crate::features::forms::{FormDefinition, FormSummary, RenderedForm};
use crate::features::organization::NodeTypeCatalogEntry;
#[cfg(feature = "hydrate")]
use crate::http::redirect_to_login;
use leptos::prelude::*;

#[allow(clippy::too_many_arguments)]
pub(crate) fn load_form_edit_options(
    form_id: String,
    node_types: RwSignal<Vec<NodeTypeCatalogEntry>>,
    existing_forms: RwSignal<Vec<FormSummary>>,
    detail: RwSignal<Option<FormDefinition>>,
    rendered_form: RwSignal<Option<RenderedForm>>,
    edit_version_id: RwSignal<Option<String>>,
    edit_version_status: RwSignal<Option<String>>,
    name: RwSignal<String>,
    workflow_node_type_id: RwSignal<String>,
    builder_sections: RwSignal<Vec<FormBuilderSectionDraft>>,
    builder_fields: RwSignal<Vec<FormBuilderFieldDraft>>,
    active_builder_section: RwSignal<String>,
    next_builder_section_id: RwSignal<usize>,
    next_builder_field_id: RwSignal<usize>,
    is_loading: RwSignal<bool>,
    message: RwSignal<Option<String>>,
) {
    #[cfg(feature = "hydrate")]
    {
        leptos::task::spawn_local(async move {
            is_loading.set(true);
            message.set(None);
            detail.set(None);
            rendered_form.set(None);
            edit_version_id.set(None);
            edit_version_status.set(None);

            let node_types_response = gloo_net::http::Request::get("/api/node-types").send().await;
            let forms_response = gloo_net::http::Request::get("/api/forms").send().await;
            let detail_response =
                gloo_net::http::Request::get(&format!("/api/admin/forms/{form_id}"))
                    .send()
                    .await;

            match (node_types_response, forms_response, detail_response) {
                (Ok(response), _, _) if response.status() == 401 => {
                    is_loading.set(false);
                    redirect_to_login();
                }
                (_, Ok(response), _) if response.status() == 401 => {
                    is_loading.set(false);
                    redirect_to_login();
                }
                (_, _, Ok(response)) if response.status() == 401 => {
                    is_loading.set(false);
                    redirect_to_login();
                }
                (Ok(node_types_response), Ok(forms_response), Ok(detail_response))
                    if node_types_response.ok() && forms_response.ok() && detail_response.ok() =>
                {
                    let loaded_node_types = node_types_response
                        .json::<Vec<NodeTypeCatalogEntry>>()
                        .await;
                    let loaded_forms = forms_response.json::<Vec<FormSummary>>().await;
                    let loaded_detail = detail_response.json::<FormDefinition>().await;

                    match (loaded_node_types, loaded_forms, loaded_detail) {
                        (Ok(loaded_node_types), Ok(loaded_forms), Ok(form)) => {
                            let selected_version = editable_form_definition_version(&form).cloned();
                            let mut loaded_rendered_form = None;

                            if let Some(version) = selected_version.as_ref() {
                                match gloo_net::http::Request::get(&format!(
                                    "/api/form-versions/{}/render",
                                    version.id
                                ))
                                .send()
                                .await
                                {
                                    Ok(response) if response.ok() => {
                                        loaded_rendered_form =
                                            response.json::<RenderedForm>().await.ok();
                                    }
                                    Ok(response) if response.status() == 401 => {
                                        is_loading.set(false);
                                        redirect_to_login();
                                        return;
                                    }
                                    _ => {
                                        loaded_rendered_form = None;
                                    }
                                }
                            }

                            let (sections, fields, next_section, next_field) =
                                hydrate_form_builder_from_rendered(loaded_rendered_form.as_ref());
                            let active_section = sections
                                .first()
                                .map(|section| section.id.to_string())
                                .unwrap_or_else(|| "1".to_string());

                            name.set(form.name.clone());
                            workflow_node_type_id
                                .set(form.scope_node_type_id.clone().unwrap_or_default());
                            edit_version_id
                                .set(selected_version.as_ref().map(|version| version.id.clone()));
                            edit_version_status.set(
                                selected_version
                                    .as_ref()
                                    .map(|version| version.status.clone()),
                            );
                            active_builder_section.set(active_section);
                            next_builder_section_id.set(next_section);
                            next_builder_field_id.set(next_field);
                            builder_sections.set(sections);
                            builder_fields.set(fields);
                            rendered_form.set(loaded_rendered_form);
                            detail.set(Some(form));
                            node_types.set(loaded_node_types);
                            existing_forms.set(loaded_forms);
                            is_loading.set(false);
                        }
                        _ => {
                            is_loading.set(false);
                            message.set(Some("Form edit options could not be read.".into()));
                        }
                    }
                }
                (Ok(node_types_response), Ok(forms_response), Ok(detail_response)) => {
                    is_loading.set(false);
                    message.set(Some(format!(
                        "Form edit options failed with status {} / {} / {}.",
                        node_types_response.status(),
                        forms_response.status(),
                        detail_response.status()
                    )));
                }
                _ => {
                    is_loading.set(false);
                    message.set(Some("Could not reach the form edit APIs.".into()));
                }
            }
        });
    }

    #[cfg(not(feature = "hydrate"))]
    {
        let _ = (
            form_id,
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
    }
}
