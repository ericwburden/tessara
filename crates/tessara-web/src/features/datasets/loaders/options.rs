//! Editor option loaders for the Datasets feature.

#[cfg(feature = "hydrate")]
use super::super::api;
use super::super::types::{
    DatasetFormOption, DatasetRenderedForm, DatasetUserOption, NodeResponse,
};
use leptos::prelude::*;
use std::collections::BTreeMap;

#[cfg(feature = "hydrate")]
pub(in crate::features::datasets) fn load_forms(
    forms: RwSignal<Vec<DatasetFormOption>>,
    load_error: RwSignal<Option<String>>,
) {
    leptos::task::spawn_local(async move {
        match api::fetch_forms().await {
            Ok(Some(payload)) => forms.set(payload),
            Ok(None) => {}
            Err(message) => load_error.set(Some(message)),
        }
    });
}

#[cfg(not(feature = "hydrate"))]
pub(in crate::features::datasets) fn load_forms(
    _: RwSignal<Vec<DatasetFormOption>>,
    _: RwSignal<Option<String>>,
) {
}

#[cfg(feature = "hydrate")]
pub(in crate::features::datasets) fn load_nodes(
    nodes: RwSignal<Vec<NodeResponse>>,
    load_error: RwSignal<Option<String>>,
) {
    leptos::task::spawn_local(async move {
        match api::fetch_nodes().await {
            Ok(Some(payload)) => nodes.set(payload),
            Ok(None) => {}
            Err(message) => load_error.set(Some(message)),
        }
    });
}

#[cfg(not(feature = "hydrate"))]
pub(in crate::features::datasets) fn load_nodes(
    _: RwSignal<Vec<NodeResponse>>,
    _: RwSignal<Option<String>>,
) {
}

#[cfg(feature = "hydrate")]
pub(in crate::features::datasets) fn load_users(
    users: RwSignal<Vec<DatasetUserOption>>,
    load_error: RwSignal<Option<String>>,
) {
    leptos::task::spawn_local(async move {
        match api::fetch_users().await {
            Ok(Some(payload)) => users.set(payload),
            Ok(None) => {}
            Err(message) => load_error.set(Some(message)),
        }
    });
}

#[cfg(not(feature = "hydrate"))]
pub(in crate::features::datasets) fn load_users(
    _: RwSignal<Vec<DatasetUserOption>>,
    _: RwSignal<Option<String>>,
) {
}

#[cfg(feature = "hydrate")]
pub(in crate::features::datasets) fn load_rendered_form(
    form_version_id: String,
    rendered_forms: RwSignal<BTreeMap<String, DatasetRenderedForm>>,
) {
    if form_version_id.is_empty()
        || rendered_forms
            .get_untracked()
            .contains_key(&form_version_id)
    {
        return;
    }
    leptos::task::spawn_local(async move {
        if let Ok(Some(payload)) = api::fetch_rendered_form(&form_version_id).await {
            rendered_forms.update(|forms| {
                forms.insert(form_version_id, payload);
            });
        }
    });
}

#[cfg(not(feature = "hydrate"))]
pub(in crate::features::datasets) fn load_rendered_form(
    _: String,
    _: RwSignal<BTreeMap<String, DatasetRenderedForm>>,
) {
}
