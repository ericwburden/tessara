//! Dataset editor save and preview actions.

#[cfg(feature = "hydrate")]
use super::api;
#[cfg(feature = "hydrate")]
use super::expressions::{build_expression_ast, is_join_operation};
use super::types::*;
use leptos::prelude::*;

#[cfg(feature = "hydrate")]
#[allow(clippy::too_many_arguments)]
/// Handles the save dataset behavior.
pub(super) fn save_dataset(
    dataset_id: Option<String>,
    name: String,
    slug: String,
    composition_mode: String,
    visibility_node_ids: Vec<String>,
    sources: Vec<DatasetSourceDraft>,
    fields: Vec<DatasetFieldDraft>,
    join_left_key: String,
    join_right_key: String,
    save_error: RwSignal<Option<String>>,
    save_message: RwSignal<Option<String>>,
) {
    leptos::task::spawn_local(async move {
        save_error.set(None);
        save_message.set(None);
        let payload = match dataset_payload_from_drafts(
            name,
            slug,
            composition_mode,
            visibility_node_ids,
            sources,
            fields,
            join_left_key,
            join_right_key,
        ) {
            Ok(payload) => payload,
            Err(message) => {
                save_error.set(Some(message));
                return;
            }
        };
        match api::save_dataset_payload(dataset_id.as_deref(), &payload).await {
            Ok(value) => {
                let id = value
                    .get("id")
                    .and_then(|value| value.as_str())
                    .unwrap_or_default()
                    .to_string();
                save_message.set(Some("Dataset saved.".into()));
                if !id.is_empty() {
                    if let Some(window) = web_sys::window() {
                        let _ = window.location().set_href(&format!("/datasets/{id}"));
                    }
                }
            }
            Err(message) => save_error.set(Some(message)),
        }
    });
}

#[cfg(not(feature = "hydrate"))]
#[allow(clippy::too_many_arguments)]
/// Handles the save dataset behavior.
pub(super) fn save_dataset(
    _: Option<String>,
    _: String,
    _: String,
    _: String,
    _: Vec<String>,
    _: Vec<DatasetSourceDraft>,
    _: Vec<DatasetFieldDraft>,
    _: String,
    _: String,
    _: RwSignal<Option<String>>,
    _: RwSignal<Option<String>>,
) {
}

#[cfg(feature = "hydrate")]
/// Handles the dataset payload from drafts behavior.
pub(super) fn dataset_payload_from_drafts(
    name: String,
    slug: String,
    composition_mode: String,
    visibility_node_ids: Vec<String>,
    mut sources: Vec<DatasetSourceDraft>,
    fields: Vec<DatasetFieldDraft>,
    join_left_key: String,
    join_right_key: String,
) -> Result<DatasetPayload, String> {
    if is_join_operation(&composition_mode) {
        for source in &mut sources {
            if source.selection_rule == "all" {
                source.selection_rule = "latest".into();
            }
        }
    }
    let field_payloads = fields
        .into_iter()
        .enumerate()
        .filter(|(_, field)| {
            !field.key.trim().is_empty()
                && !field.label.trim().is_empty()
                && !field.source_alias.trim().is_empty()
                && !field.source_field_key.trim().is_empty()
        })
        .map(|(index, field)| DatasetFieldPayload {
            key: field.key,
            label: field.label,
            source_alias: field.source_alias,
            source_field_key: field.source_field_key,
            position: index as i32,
        })
        .collect::<Vec<_>>();
    let Some(definition_ast) =
        build_expression_ast(&sources, &composition_mode, &join_left_key, &join_right_key)
    else {
        return Err("Choose at least one complete dataset input before saving.".into());
    };
    Ok(DatasetPayload {
        name,
        slug,
        grain: "submission".into(),
        composition_mode,
        visibility_node_ids,
        definition_ast,
        fields: field_payloads,
    })
}

#[cfg(feature = "hydrate")]
#[allow(clippy::too_many_arguments)]
/// Handles the preview dataset sql behavior.
pub(super) fn preview_dataset_sql(
    dataset_id: Option<String>,
    name: String,
    slug: String,
    composition_mode: String,
    visibility_node_ids: Vec<String>,
    sources: Vec<DatasetSourceDraft>,
    fields: Vec<DatasetFieldDraft>,
    join_left_key: String,
    join_right_key: String,
    sql_preview: RwSignal<Option<String>>,
    sql_preview_error: RwSignal<Option<String>>,
) {
    leptos::task::spawn_local(async move {
        sql_preview_error.set(None);
        let payload = match dataset_payload_from_drafts(
            name,
            slug,
            composition_mode,
            visibility_node_ids,
            sources,
            fields,
            join_left_key,
            join_right_key,
        ) {
            Ok(payload) => payload,
            Err(message) => {
                sql_preview_error.set(Some(message));
                return;
            }
        };
        match api::preview_dataset_sql_payload(dataset_id.as_deref(), &payload).await {
            Ok(response) => sql_preview.set(Some(response.generated_sql)),
            Err(message) => sql_preview_error.set(Some(message)),
        }
    });
}

#[cfg(not(feature = "hydrate"))]
#[allow(clippy::too_many_arguments)]
/// Handles the preview dataset sql behavior.
pub(super) fn preview_dataset_sql(
    _: Option<String>,
    _: String,
    _: String,
    _: String,
    _: Vec<String>,
    _: Vec<DatasetSourceDraft>,
    _: Vec<DatasetFieldDraft>,
    _: String,
    _: String,
    _: RwSignal<Option<String>>,
    _: RwSignal<Option<String>>,
) {
}
