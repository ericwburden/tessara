//! Signal-aware loaders for the Forms feature.
//!
//! Keep list and detail page loading state here; endpoint transport belongs in `api`.

#[cfg(feature = "hydrate")]
use crate::features::forms::active_form_definition_version;
use crate::features::forms::{FormDefinition, FormSummary, RenderedForm};
#[cfg(feature = "hydrate")]
use crate::http::redirect_to_login;
use leptos::prelude::*;

#[cfg(feature = "hydrate")]
use super::api::{FormsApiError, fetch_form_detail, fetch_forms, fetch_rendered_form_version};

/// Loads the forms list into page state.
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

/// Loads a form detail and its active rendered version into page state.
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
/// Loads a rendered form version into page state.
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
