//! Payload preparation helpers for dataset mutations.

use super::editor::canonical_field_key;
use super::expressions::{build_expression_ast, expression_uses_join, root_expression_operation};
use super::types::*;

#[cfg(feature = "hydrate")]
pub(super) fn dataset_payload_from_drafts(
    name: String,
    slug: String,
    visibility_node_ids: Vec<String>,
    mut sources: Vec<DatasetSourceDraft>,
    expression: DatasetExpressionDraft,
    fields: Vec<DatasetFieldDraft>,
    join_left_key: String,
    join_right_key: String,
) -> Result<DatasetPayload, String> {
    let root_composition_mode =
        root_expression_operation(&expression).unwrap_or_else(|| "union".into());
    if expression_uses_join(&expression) {
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
            key: canonical_field_key(&field.source_alias, &field.source_field_key),
            label: field.label,
            source_alias: field.source_alias,
            source_field_key: field.source_field_key,
            position: index as i32,
        })
        .collect::<Vec<_>>();
    let Some(definition_ast) =
        build_expression_ast(&sources, &expression, &join_left_key, &join_right_key)
    else {
        return Err("Choose at least one complete dataset input before saving.".into());
    };
    Ok(DatasetPayload {
        name,
        slug,
        grain: "submission".into(),
        composition_mode: root_composition_mode,
        visibility_node_ids,
        definition_ast,
        fields: field_payloads,
    })
}
