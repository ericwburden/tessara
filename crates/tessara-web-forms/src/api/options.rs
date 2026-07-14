//! Form editor option transport.

#[cfg(feature = "hydrate")]
use super::FormsApiError;
#[cfg(feature = "hydrate")]
use crate::FormNodeTypeOption;
#[cfg(feature = "hydrate")]
use crate::versions::editable_form_definition_version;
#[cfg(feature = "hydrate")]
use crate::{FormDefinition, FormSummary, RenderedForm};

#[cfg(feature = "hydrate")]
pub(crate) struct FormCreateOptions {
    pub(crate) node_types: Vec<FormNodeTypeOption>,
    pub(crate) existing_forms: Vec<FormSummary>,
}

#[cfg(feature = "hydrate")]
pub(crate) struct FormEditOptions {
    pub(crate) node_types: Vec<FormNodeTypeOption>,
    pub(crate) existing_forms: Vec<FormSummary>,
    pub(crate) detail: FormDefinition,
    pub(crate) rendered_form: Option<RenderedForm>,
    pub(crate) edit_version_id: Option<String>,
    pub(crate) edit_version_status: Option<String>,
}

#[cfg(feature = "hydrate")]
pub(crate) async fn fetch_form_create_options() -> Result<FormCreateOptions, FormsApiError> {
    let node_types = tessara_web_http::fetch_json("/api/node-types", "Form node type options")
        .await
        .map_err(FormsApiError::from_request)?;
    let existing_forms = tessara_web_http::fetch_json("/api/forms", "Form options")
        .await
        .map_err(FormsApiError::from_request)?;

    Ok(FormCreateOptions {
        node_types,
        existing_forms,
    })
}

#[cfg(feature = "hydrate")]
pub(crate) async fn fetch_form_edit_options(
    form_id: &str,
) -> Result<FormEditOptions, FormsApiError> {
    let node_types = tessara_web_http::fetch_json("/api/node-types", "Form node type options")
        .await
        .map_err(FormsApiError::from_request)?;
    let existing_forms = tessara_web_http::fetch_json("/api/forms", "Form options")
        .await
        .map_err(FormsApiError::from_request)?;
    let detail: FormDefinition =
        tessara_web_http::fetch_json(&format!("/api/admin/forms/{form_id}"), "Form edit detail")
            .await
            .map_err(FormsApiError::from_request)?;

    let selected_version = editable_form_definition_version(&detail);
    let rendered_form = if let Some(version) = selected_version {
        match tessara_web_http::fetch_json::<RenderedForm>(
            &format!("/api/form-versions/{}/render", version.id),
            "Rendered form",
        )
        .await
        {
            Ok(form) => Some(form),
            Err(error) if error.is_authentication() => {
                return Err(FormsApiError::Unauthorized);
            }
            Err(_) => None,
        }
    } else {
        None
    };

    Ok(FormEditOptions {
        node_types,
        existing_forms,
        edit_version_id: selected_version.map(|version| version.id.clone()),
        edit_version_status: selected_version.map(|version| version.status.clone()),
        detail,
        rendered_form,
    })
}
