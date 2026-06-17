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
}

#[derive(Clone, Debug, PartialEq)]
pub(in crate::features::datasets) struct DatasetFieldDraft {
    pub(in crate::features::datasets) key: String,
    pub(in crate::features::datasets) label: String,
    pub(in crate::features::datasets) source_alias: String,
    pub(in crate::features::datasets) source_field_key: String,
    pub(in crate::features::datasets) field_type: String,
}

#[derive(Clone, Debug, PartialEq)]
pub(in crate::features::datasets) struct DatasetAggregationDraft {
    pub(in crate::features::datasets) enabled: bool,
    pub(in crate::features::datasets) group_fields: Vec<String>,
    pub(in crate::features::datasets) metrics: Vec<DatasetAggregationMetricDraft>,
    pub(in crate::features::datasets) row_picker: Option<DatasetRowPickerDraft>,
}

#[derive(Clone, Debug, PartialEq)]
pub(in crate::features::datasets) struct DatasetAggregationMetricDraft {
    pub(in crate::features::datasets) id: u64,
    pub(in crate::features::datasets) key: String,
    pub(in crate::features::datasets) label: String,
    pub(in crate::features::datasets) function: String,
    pub(in crate::features::datasets) source_field_key: String,
}

#[derive(Clone, Debug, PartialEq)]
pub(in crate::features::datasets) struct DatasetRowPickerDraft {
    pub(in crate::features::datasets) sort_fields: Vec<DatasetRowPickerSortDraft>,
    pub(in crate::features::datasets) direction: String,
}

#[derive(Clone, Debug, PartialEq)]
pub(in crate::features::datasets) struct DatasetRowPickerSortDraft {
    pub(in crate::features::datasets) field_key: String,
}

#[derive(Clone, Debug, PartialEq)]
pub(in crate::features::datasets) enum DatasetExpressionDraft {
    Source(usize),
    Operation {
        operation: String,
        left: Box<DatasetExpressionDraft>,
        right: Box<DatasetExpressionDraft>,
    },
}

#[derive(Clone, Debug, PartialEq)]
pub(in crate::features::datasets) enum DatasetDesignerSelection {
    Operation(Vec<bool>),
    Source(usize),
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
        }
    }
}

impl Default for DatasetExpressionDraft {
    fn default() -> Self {
        Self::Source(0)
    }
}

impl Default for DatasetAggregationDraft {
    fn default() -> Self {
        Self {
            enabled: false,
            group_fields: Vec::new(),
            metrics: Vec::new(),
            row_picker: None,
        }
    }
}
