//! Transport calls for the Forms feature.
//!
//! Keep endpoint requests and response parsing here; Leptos signal orchestration belongs in loaders and save actions.

#[cfg(feature = "hydrate")]
use super::versions::editable_form_definition_version;
#[cfg(feature = "hydrate")]
use crate::features::forms::{FormDefinition, FormSummary, RenderedForm};
#[cfg(feature = "hydrate")]
use crate::features::organization::NodeTypeCatalogEntry;

#[cfg(feature = "hydrate")]
pub(super) enum FormsApiError {
    Unauthorized,
    Message(String),
}

#[cfg(feature = "hydrate")]
pub(super) struct FormCreateOptions {
    pub(super) node_types: Vec<NodeTypeCatalogEntry>,
    pub(super) existing_forms: Vec<FormSummary>,
}

#[cfg(feature = "hydrate")]
pub(super) struct FormEditOptions {
    pub(super) node_types: Vec<NodeTypeCatalogEntry>,
    pub(super) existing_forms: Vec<FormSummary>,
    pub(super) detail: FormDefinition,
    pub(super) rendered_form: Option<RenderedForm>,
    pub(super) edit_version_id: Option<String>,
    pub(super) edit_version_status: Option<String>,
}

#[cfg(feature = "hydrate")]
impl FormsApiError {
    fn message(message: impl Into<String>) -> Self {
        Self::Message(message.into())
    }
}

#[cfg(feature = "hydrate")]
pub(super) async fn fetch_forms() -> Result<Vec<FormSummary>, FormsApiError> {
    match gloo_net::http::Request::get("/api/forms").send().await {
        Ok(response) if response.status() == 401 => Err(FormsApiError::Unauthorized),
        Ok(response) if response.ok() => response
            .json::<Vec<FormSummary>>()
            .await
            .map_err(|error| FormsApiError::message(format!("Unable to parse forms: {error}"))),
        Ok(response) => Err(FormsApiError::message(format!(
            "Unable to load forms. Server returned {}.",
            response.status()
        ))),
        Err(error) => Err(FormsApiError::message(format!(
            "Unable to load forms: {error}"
        ))),
    }
}

#[cfg(feature = "hydrate")]
pub(super) async fn fetch_form_detail(form_id: &str) -> Result<FormDefinition, FormsApiError> {
    match gloo_net::http::Request::get(&format!("/api/forms/{form_id}"))
        .send()
        .await
    {
        Ok(response) if response.status() == 401 => Err(FormsApiError::Unauthorized),
        Ok(response) if response.ok() => response.json::<FormDefinition>().await.map_err(|error| {
            FormsApiError::message(format!("Unable to parse form detail: {error}"))
        }),
        Ok(response) => Err(FormsApiError::message(format!(
            "Unable to load form detail. Server returned {}.",
            response.status()
        ))),
        Err(error) => Err(FormsApiError::message(format!(
            "Unable to load form detail: {error}"
        ))),
    }
}

#[cfg(feature = "hydrate")]
pub(super) async fn fetch_rendered_form_version(
    form_version_id: &str,
) -> Result<RenderedForm, FormsApiError> {
    match gloo_net::http::Request::get(&format!("/api/form-versions/{form_version_id}/render"))
        .send()
        .await
    {
        Ok(response) if response.status() == 401 => Err(FormsApiError::Unauthorized),
        Ok(response) if response.ok() => response.json::<RenderedForm>().await.map_err(|error| {
            FormsApiError::message(format!("Unable to parse rendered form: {error}"))
        }),
        Ok(response) => Err(FormsApiError::message(format!(
            "Unable to load rendered form. Server returned {}.",
            response.status()
        ))),
        Err(error) => Err(FormsApiError::message(format!(
            "Unable to load rendered form: {error}"
        ))),
    }
}

#[cfg(feature = "hydrate")]
pub(super) async fn fetch_form_create_options() -> Result<FormCreateOptions, FormsApiError> {
    let node_types_response = gloo_net::http::Request::get("/api/node-types").send().await;
    let forms_response = gloo_net::http::Request::get("/api/forms").send().await;

    match (node_types_response, forms_response) {
        (Ok(response), _) if response.status() == 401 => Err(FormsApiError::Unauthorized),
        (_, Ok(response)) if response.status() == 401 => Err(FormsApiError::Unauthorized),
        (Ok(node_types_response), Ok(forms_response))
            if node_types_response.ok() && forms_response.ok() =>
        {
            let node_types = node_types_response
                .json::<Vec<NodeTypeCatalogEntry>>()
                .await;
            let existing_forms = forms_response.json::<Vec<FormSummary>>().await;

            match (node_types, existing_forms) {
                (Ok(node_types), Ok(existing_forms)) => Ok(FormCreateOptions {
                    node_types,
                    existing_forms,
                }),
                _ => Err(FormsApiError::message("Form options could not be read.")),
            }
        }
        (Ok(node_types_response), Ok(forms_response)) => Err(FormsApiError::message(format!(
            "Form options failed with status {} / {}.",
            node_types_response.status(),
            forms_response.status()
        ))),
        _ => Err(FormsApiError::message(
            "Could not reach the form option APIs.",
        )),
    }
}

#[cfg(feature = "hydrate")]
pub(super) async fn fetch_form_edit_options(
    form_id: &str,
) -> Result<FormEditOptions, FormsApiError> {
    let node_types_response = gloo_net::http::Request::get("/api/node-types").send().await;
    let forms_response = gloo_net::http::Request::get("/api/forms").send().await;
    let detail_response = gloo_net::http::Request::get(&format!("/api/admin/forms/{form_id}"))
        .send()
        .await;

    match (node_types_response, forms_response, detail_response) {
        (Ok(response), _, _) if response.status() == 401 => Err(FormsApiError::Unauthorized),
        (_, Ok(response), _) if response.status() == 401 => Err(FormsApiError::Unauthorized),
        (_, _, Ok(response)) if response.status() == 401 => Err(FormsApiError::Unauthorized),
        (Ok(node_types_response), Ok(forms_response), Ok(detail_response))
            if node_types_response.ok() && forms_response.ok() && detail_response.ok() =>
        {
            let node_types = node_types_response
                .json::<Vec<NodeTypeCatalogEntry>>()
                .await;
            let existing_forms = forms_response.json::<Vec<FormSummary>>().await;
            let detail = detail_response.json::<FormDefinition>().await;

            match (node_types, existing_forms, detail) {
                (Ok(node_types), Ok(existing_forms), Ok(detail)) => {
                    let selected_version = editable_form_definition_version(&detail);
                    let mut rendered_form = None;

                    if let Some(version) = selected_version {
                        match gloo_net::http::Request::get(&format!(
                            "/api/form-versions/{}/render",
                            version.id
                        ))
                        .send()
                        .await
                        {
                            Ok(response) if response.ok() => {
                                rendered_form = response.json::<RenderedForm>().await.ok();
                            }
                            Ok(response) if response.status() == 401 => {
                                return Err(FormsApiError::Unauthorized);
                            }
                            _ => {
                                rendered_form = None;
                            }
                        }
                    }

                    Ok(FormEditOptions {
                        node_types,
                        existing_forms,
                        edit_version_id: selected_version.map(|version| version.id.clone()),
                        edit_version_status: selected_version.map(|version| version.status.clone()),
                        detail,
                        rendered_form,
                    })
                }
                _ => Err(FormsApiError::message(
                    "Form edit options could not be read.",
                )),
            }
        }
        (Ok(node_types_response), Ok(forms_response), Ok(detail_response)) => {
            Err(FormsApiError::message(format!(
                "Form edit options failed with status {} / {} / {}.",
                node_types_response.status(),
                forms_response.status(),
                detail_response.status()
            )))
        }
        _ => Err(FormsApiError::message(
            "Could not reach the form edit APIs.",
        )),
    }
}
