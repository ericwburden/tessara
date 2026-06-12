//! Option loader for the form create page.

use crate::features::forms::FormSummary;
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
