//! Editor-local dataset draft types.

#[derive(Clone, Debug, PartialEq)]
pub(in crate::features::datasets) struct DatasetSourceDraft {
    pub(in crate::features::datasets) input_kind: String,
    pub(in crate::features::datasets) source_alias: String,
    pub(in crate::features::datasets) form_id: String,
    pub(in crate::features::datasets) form_version_id: String,
    pub(in crate::features::datasets) form_version_major: Option<i32>,
    pub(in crate::features::datasets) dataset_id: String,
    pub(in crate::features::datasets) dataset_revision_id: String,
    pub(in crate::features::datasets) selection_rule: String,
}

#[derive(Clone, Debug, PartialEq)]
pub(in crate::features::datasets) struct DatasetFieldDraft {
    pub(in crate::features::datasets) key: String,
    pub(in crate::features::datasets) label: String,
    pub(in crate::features::datasets) source_alias: String,
    pub(in crate::features::datasets) source_field_key: String,
}

#[derive(Clone, Debug, PartialEq)]
pub(in crate::features::datasets) enum DatasetDesignerSelection {
    Operation,
    Source(usize),
    Field(usize),
}

impl Default for DatasetSourceDraft {
    fn default() -> Self {
        Self {
            input_kind: "form".into(),
            source_alias: "source_1".into(),
            form_id: String::new(),
            form_version_id: String::new(),
            form_version_major: None,
            dataset_id: String::new(),
            dataset_revision_id: String::new(),
            selection_rule: "latest".into(),
        }
    }
}
