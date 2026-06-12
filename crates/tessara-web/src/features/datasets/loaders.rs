//! Data-loading helpers for the Datasets feature.
//!
//! Keep reusable load routines here when multiple Datasets pages need the same fetch-and-signal update pattern.

#[cfg(feature = "hydrate")]
use super::api;
#[cfg(feature = "hydrate")]
use super::expressions::expression_to_editor_drafts;
use super::types::*;
use leptos::prelude::*;
use std::collections::{BTreeMap, BTreeSet};

#[cfg(feature = "hydrate")]
/// Loads the load account data.
pub(super) fn load_account(account: RwSignal<Option<SessionAccount>>) {
    leptos::task::spawn_local(async move {
        if let Ok(Some(payload)) = api::fetch_account().await {
            account.set(Some(payload));
        }
    });
}

#[cfg(not(feature = "hydrate"))]
/// Loads the load account data.
pub(super) fn load_account(_: RwSignal<Option<SessionAccount>>) {}

#[cfg(feature = "hydrate")]
/// Loads the load datasets data.
pub(super) fn load_datasets(
    datasets: RwSignal<Vec<DatasetSummary>>,
    is_loading: RwSignal<bool>,
    load_error: RwSignal<Option<String>>,
) {
    leptos::task::spawn_local(async move {
        is_loading.set(true);
        match api::fetch_datasets().await {
            Ok(Some(payload)) => datasets.set(payload),
            Ok(None) => {}
            Err(message) => load_error.set(Some(message)),
        }
        is_loading.set(false);
    });
}

#[cfg(not(feature = "hydrate"))]
/// Loads the load datasets data.
pub(super) fn load_datasets(
    _: RwSignal<Vec<DatasetSummary>>,
    _: RwSignal<bool>,
    _: RwSignal<Option<String>>,
) {
}

#[cfg(feature = "hydrate")]
/// Loads the load dataset detail data.
pub(super) fn load_dataset_detail(
    dataset_id: String,
    dataset: RwSignal<Option<DatasetDefinition>>,
    is_loading: RwSignal<bool>,
    load_error: RwSignal<Option<String>>,
) {
    leptos::task::spawn_local(async move {
        is_loading.set(true);
        match api::fetch_dataset_detail(&dataset_id).await {
            Ok(Some(payload)) => dataset.set(Some(payload)),
            Ok(None) => {}
            Err(message) => load_error.set(Some(message)),
        }
        is_loading.set(false);
    });
}

#[cfg(not(feature = "hydrate"))]
/// Loads the load dataset detail data.
pub(super) fn load_dataset_detail(
    _: String,
    _: RwSignal<Option<DatasetDefinition>>,
    _: RwSignal<bool>,
    _: RwSignal<Option<String>>,
) {
}

#[cfg(feature = "hydrate")]
/// Loads the load dataset table data.
pub(super) fn load_dataset_table(
    dataset_id: String,
    table: RwSignal<Option<DatasetTable>>,
    table_error: RwSignal<Option<String>>,
) {
    leptos::task::spawn_local(async move {
        match api::fetch_dataset_table(&dataset_id).await {
            Ok(Some(payload)) => table.set(Some(payload)),
            Ok(None) => {}
            Err(message) => table_error.set(Some(message)),
        }
    });
}

#[cfg(not(feature = "hydrate"))]
/// Loads the load dataset table data.
pub(super) fn load_dataset_table(
    _: String,
    _: RwSignal<Option<DatasetTable>>,
    _: RwSignal<Option<String>>,
) {
}

#[cfg(feature = "hydrate")]
/// Loads the load forms data.
pub(super) fn load_forms(
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
/// Loads the load forms data.
pub(super) fn load_forms(_: RwSignal<Vec<DatasetFormOption>>, _: RwSignal<Option<String>>) {}

#[cfg(feature = "hydrate")]
/// Loads the load nodes data.
pub(super) fn load_nodes(nodes: RwSignal<Vec<NodeResponse>>, load_error: RwSignal<Option<String>>) {
    leptos::task::spawn_local(async move {
        match api::fetch_nodes().await {
            Ok(Some(payload)) => nodes.set(payload),
            Ok(None) => {}
            Err(message) => load_error.set(Some(message)),
        }
    });
}

#[cfg(not(feature = "hydrate"))]
/// Loads the load nodes data.
pub(super) fn load_nodes(_: RwSignal<Vec<NodeResponse>>, _: RwSignal<Option<String>>) {}

#[cfg(feature = "hydrate")]
/// Loads the load rendered form data.
pub(super) fn load_rendered_form(
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
/// Loads the load rendered form data.
pub(super) fn load_rendered_form(_: String, _: RwSignal<BTreeMap<String, DatasetRenderedForm>>) {}

#[cfg(feature = "hydrate")]
/// Loads the load dataset for edit data.
pub(super) fn load_dataset_for_edit(
    dataset_id: String,
    name: RwSignal<String>,
    slug: RwSignal<String>,
    composition_mode: RwSignal<String>,
    visibility_node_ids: RwSignal<BTreeSet<String>>,
    sources: RwSignal<Vec<DatasetSourceDraft>>,
    fields: RwSignal<Vec<DatasetFieldDraft>>,
    join_left_key: RwSignal<String>,
    join_right_key: RwSignal<String>,
    sql_preview: RwSignal<Option<String>>,
    load_error: RwSignal<Option<String>>,
) {
    leptos::task::spawn_local(async move {
        match api::fetch_dataset_detail(&dataset_id).await {
            Ok(Some(payload)) => {
                name.set(payload.name);
                slug.set(payload.slug);
                composition_mode.set(payload.composition_mode);
                sql_preview.set(payload.generated_sql.clone());
                visibility_node_ids.set(
                    payload
                        .visibility_nodes
                        .into_iter()
                        .map(|node| node.node_id)
                        .collect(),
                );
                let Some(ast) = payload.definition_ast.as_ref() else {
                    load_error.set(Some(
                            "This dataset was not created with the query designer and cannot be edited here."
                                .into(),
                        ));
                    return;
                };
                let (source_drafts, root_operation, join_keys) = expression_to_editor_drafts(ast);
                if !root_operation.is_empty() {
                    composition_mode.set(root_operation);
                }
                if let Some(join_key) = join_keys.first() {
                    join_left_key.set(join_key.left_field.clone());
                    join_right_key.set(join_key.right_field.clone());
                }
                sources.set(if source_drafts.is_empty() {
                    vec![DatasetSourceDraft::default()]
                } else {
                    source_drafts
                });
                fields.set(
                    payload
                        .fields
                        .into_iter()
                        .map(|field| DatasetFieldDraft {
                            key: field.key,
                            label: field.label,
                            source_alias: field.source_alias,
                            source_field_key: field.source_field_key,
                        })
                        .collect(),
                );
            }
            Ok(None) => {}
            Err(message) => load_error.set(Some(message)),
        }
    });
}

#[cfg(not(feature = "hydrate"))]
#[allow(clippy::too_many_arguments)]
/// Loads the load dataset for edit data.
pub(super) fn load_dataset_for_edit(
    _: String,
    _: RwSignal<String>,
    _: RwSignal<String>,
    _: RwSignal<String>,
    _: RwSignal<BTreeSet<String>>,
    _: RwSignal<Vec<DatasetSourceDraft>>,
    _: RwSignal<Vec<DatasetFieldDraft>>,
    _: RwSignal<String>,
    _: RwSignal<String>,
    _: RwSignal<Option<String>>,
    _: RwSignal<Option<String>>,
) {
}
