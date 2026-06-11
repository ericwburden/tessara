//! Signal-aware loaders for the Forms feature.
//!
//! Keep page loading state and multi-endpoint option orchestration here; endpoint transport belongs in `api`.

#[cfg(feature = "hydrate")]
use crate::features::forms::builder::hydrate_form_builder_from_rendered;
use crate::features::forms::builder::{FormBuilderFieldDraft, FormBuilderSectionDraft};
use crate::features::forms::{FormDefinition, FormSummary, RenderedForm};
#[cfg(feature = "hydrate")]
use crate::features::forms::{active_form_definition_version, editable_form_definition_version};
use crate::features::organization::NodeTypeCatalogEntry;
#[cfg(feature = "hydrate")]
use crate::http::redirect_to_login;
use leptos::prelude::*;

#[cfg(feature = "hydrate")]
use super::api::{FormsApiError, fetch_form_detail, fetch_forms, fetch_rendered_form_version};

/// Loads the load forms data.
pub(crate) fn load_forms(
    forms: RwSignal<Vec<FormSummary>>,
    is_loading: RwSignal<bool>,
    load_error: RwSignal<Option<String>>,
) {
    #[cfg(feature = "hydrate")]
    {
        leptos::task::spawn_local(async move {
            is_loading.set(true);
            load_error.set(None);

            match fetch_forms().await {
                Ok(loaded_forms) => {
                    forms.set(loaded_forms);
                    is_loading.set(false);
                }
                Err(FormsApiError::Unauthorized) => {
                    forms.set(Vec::new());
                    is_loading.set(false);
                    redirect_to_login();
                }
                Err(FormsApiError::Message(error)) => {
                    forms.set(Vec::new());
                    load_error.set(Some(error));
                    is_loading.set(false);
                }
            }
        });
    }

    #[cfg(not(feature = "hydrate"))]
    {
        let _ = (forms, is_loading, load_error);
    }
}

/// Loads the load form detail data.
pub(crate) fn load_form_detail(
    form_id: String,
    detail: RwSignal<Option<FormDefinition>>,
    rendered_form: RwSignal<Option<RenderedForm>>,
    is_loading: RwSignal<bool>,
    load_error: RwSignal<Option<String>>,
) {
    #[cfg(feature = "hydrate")]
    {
        leptos::task::spawn_local(async move {
            is_loading.set(true);
            load_error.set(None);
            rendered_form.set(None);

            match fetch_form_detail(&form_id).await {
                Ok(form) => {
                    let active_version_id =
                        active_form_definition_version(&form).map(|version| version.id.clone());
                    detail.set(Some(form));
                    if let Some(version_id) = active_version_id {
                        load_rendered_form_version(version_id, rendered_form);
                    }
                    is_loading.set(false);
                }
                Err(FormsApiError::Unauthorized) => {
                    detail.set(None);
                    rendered_form.set(None);
                    is_loading.set(false);
                    redirect_to_login();
                }
                Err(FormsApiError::Message(error)) => {
                    detail.set(None);
                    load_error.set(Some(error));
                    is_loading.set(false);
                }
            }
        });
    }

    #[cfg(not(feature = "hydrate"))]
    {
        let _ = (form_id, detail, rendered_form, is_loading, load_error);
    }
}

#[cfg_attr(not(feature = "hydrate"), allow(dead_code))]
/// Loads the load rendered form version data.
pub(crate) fn load_rendered_form_version(
    form_version_id: String,
    rendered_form: RwSignal<Option<RenderedForm>>,
) {
    #[cfg(feature = "hydrate")]
    {
        leptos::task::spawn_local(async move {
            match fetch_rendered_form_version(&form_version_id).await {
                Ok(rendered) => rendered_form.set(Some(rendered)),
                Err(_) => rendered_form.set(None),
            }
        });
    }

    #[cfg(not(feature = "hydrate"))]
    {
        let _ = (form_version_id, rendered_form);
    }
}

/// Loads the load form create options data.
pub(crate) fn load_form_create_options(
    node_types: RwSignal<Vec<NodeTypeCatalogEntry>>,
    existing_forms: RwSignal<Vec<FormSummary>>,
    is_loading: RwSignal<bool>,
    message: RwSignal<Option<String>>,
) {
    #[cfg(feature = "hydrate")]
    {
        leptos::task::spawn_local(async move {
            is_loading.set(true);
            message.set(None);

            let node_types_response = gloo_net::http::Request::get("/api/node-types").send().await;
            let forms_response = gloo_net::http::Request::get("/api/forms").send().await;

            match (node_types_response, forms_response) {
                (Ok(response), _) if response.status() == 401 => {
                    node_types.set(Vec::new());
                    existing_forms.set(Vec::new());
                    is_loading.set(false);
                    redirect_to_login();
                }
                (_, Ok(response)) if response.status() == 401 => {
                    node_types.set(Vec::new());
                    existing_forms.set(Vec::new());
                    is_loading.set(false);
                    redirect_to_login();
                }
                (Ok(node_types_response), Ok(forms_response))
                    if node_types_response.ok() && forms_response.ok() =>
                {
                    let loaded_node_types = node_types_response
                        .json::<Vec<NodeTypeCatalogEntry>>()
                        .await;
                    let loaded_forms = forms_response.json::<Vec<FormSummary>>().await;

                    match (loaded_node_types, loaded_forms) {
                        (Ok(loaded_node_types), Ok(loaded_forms)) => {
                            node_types.set(loaded_node_types);
                            existing_forms.set(loaded_forms);
                            is_loading.set(false);
                        }
                        _ => {
                            node_types.set(Vec::new());
                            existing_forms.set(Vec::new());
                            message.set(Some("Form options could not be read.".into()));
                            is_loading.set(false);
                        }
                    }
                }
                (Ok(node_types_response), Ok(forms_response)) => {
                    node_types.set(Vec::new());
                    existing_forms.set(Vec::new());
                    message.set(Some(format!(
                        "Form options failed with status {} / {}.",
                        node_types_response.status(),
                        forms_response.status()
                    )));
                    is_loading.set(false);
                }
                _ => {
                    node_types.set(Vec::new());
                    existing_forms.set(Vec::new());
                    message.set(Some("Could not reach the form option APIs.".into()));
                    is_loading.set(false);
                }
            }
        });
    }

    #[cfg(not(feature = "hydrate"))]
    {
        let _ = (node_types, existing_forms, is_loading, message);
    }
}

#[allow(clippy::too_many_arguments)]
/// Loads the load form edit options data.
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
