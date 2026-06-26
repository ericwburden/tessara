//! Source operation helpers for the dataset editor pipeline.

#[cfg(feature = "hydrate")]
use super::types::{DatasetSourceDraft, DatasetSourcePayload};

/// Converts a source draft into a dataset source payload.
#[cfg(feature = "hydrate")]
pub(crate) fn source_payload(source: &DatasetSourceDraft) -> Option<DatasetSourcePayload> {
    if source.source_alias.trim().is_empty() {
        return None;
    }
    if source.input_kind == "dataset" {
        if source.dataset_id.is_empty() || source.dataset_revision_id.is_empty() {
            return None;
        }
        Some(DatasetSourcePayload::Dataset {
            alias: source.source_alias.clone(),
            dataset_id: source.dataset_id.clone(),
            dataset_revision_id: source.dataset_revision_id.clone(),
        })
    } else {
        if source.form_id.is_empty() || source.form_version_id.is_empty() {
            return None;
        }
        Some(DatasetSourcePayload::Form {
            alias: source.source_alias.clone(),
            form_id: source.form_id.clone(),
            form_version_id: source.form_version_id.clone(),
        })
    }
}

#[cfg(feature = "hydrate")]
pub(crate) fn source_payload_to_draft(source: &DatasetSourcePayload) -> DatasetSourceDraft {
    match source {
        DatasetSourcePayload::Form {
            alias,
            form_id,
            form_version_id,
        } => DatasetSourceDraft {
            input_kind: "form".into(),
            source_alias: alias.clone(),
            form_id: form_id.clone(),
            form_version_id: form_version_id.clone(),
            dataset_id: String::new(),
            dataset_revision_id: String::new(),
        },
        DatasetSourcePayload::Dataset {
            alias,
            dataset_id,
            dataset_revision_id,
        } => DatasetSourceDraft {
            input_kind: "dataset".into(),
            source_alias: alias.clone(),
            form_id: String::new(),
            form_version_id: String::new(),
            dataset_id: dataset_id.clone(),
            dataset_revision_id: dataset_revision_id.clone(),
        },
    }
}
