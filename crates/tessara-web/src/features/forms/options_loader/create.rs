//! Option loader for the form create page.

use crate::features::forms::FormSummary;
#[cfg(feature = "hydrate")]
use crate::features::forms::api::{FormsApiError, fetch_form_create_options};
use crate::features::organization::NodeTypeCatalogEntry;
#[cfg(feature = "hydrate")]
use crate::http::redirect_to_login;
use leptos::prelude::*;

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

            match fetch_form_create_options().await {
                Ok(options) => {
                    node_types.set(options.node_types);
                    existing_forms.set(options.existing_forms);
                    is_loading.set(false);
                }
                Err(FormsApiError::Unauthorized) => {
                    node_types.set(Vec::new());
                    existing_forms.set(Vec::new());
                    is_loading.set(false);
                    redirect_to_login();
                }
                Err(FormsApiError::Message(error)) => {
                    node_types.set(Vec::new());
                    existing_forms.set(Vec::new());
                    message.set(Some(error));
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
