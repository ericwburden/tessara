//! Editor-local dataset draft types.

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct DatasetSourceDraft {
    pub(crate) input_kind: String,
    pub(crate) source_alias: String,
    pub(crate) form_id: String,
    pub(crate) form_version_id: String,
    pub(crate) dataset_id: String,
    pub(crate) dataset_revision_id: String,
    pub(crate) dataset_version_major: Option<i32>,
}

pub use tessara_web_data_ops::{
    DatasetAggregationDraft, DatasetAggregationMetricDraft, DatasetFieldDraft,
    DatasetRowFilterDraft, DatasetRowPickerDraft, DatasetRowPickerSortDraft,
};

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct DatasetCalculatedFieldDraft {
    pub(crate) id: u64,
    pub(crate) key: String,
    pub(crate) label: String,
    pub(crate) base_field_key: String,
    pub(crate) functions: Vec<DatasetCalculationFunctionDraft>,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct DatasetCalculationFunctionDraft {
    pub(crate) id: u64,
    pub(crate) function: String,
    pub(crate) argument: String,
    pub(crate) argument_mode: String,
    pub(crate) argument_field_key: String,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum DatasetOperationDraftKind {
    AddSource,
    Projection,
    Aggregation,
    CalculatedFields,
    Filter,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct DatasetOperationDraft {
    pub(crate) id: u64,
    pub(crate) kind: DatasetOperationDraftKind,
    pub(crate) source: Option<DatasetSourceDraft>,
    pub(crate) add_type: String,
    pub(crate) left_field_key: String,
    pub(crate) right_field_key: String,
    pub(crate) projection_fields: Vec<DatasetFieldDraft>,
    pub(crate) aggregation: DatasetAggregationDraft,
    pub(crate) calculated_fields: Vec<DatasetCalculatedFieldDraft>,
    pub(crate) row_filters: Vec<DatasetRowFilterDraft>,
}

impl Default for DatasetSourceDraft {
    fn default() -> Self {
        Self {
            input_kind: "form".into(),
            source_alias: "source_1".into(),
            form_id: String::new(),
            form_version_id: String::new(),
            dataset_id: String::new(),
            dataset_revision_id: String::new(),
            dataset_version_major: None,
        }
    }
}

impl DatasetOperationDraftKind {
    pub(crate) fn label(self) -> &'static str {
        match self {
            Self::AddSource => "Add Source",
            Self::Projection => "Projection",
            Self::Aggregation => "Aggregation",
            Self::CalculatedFields => "Calculated Fields",
            Self::Filter => "Filter",
        }
    }
}

impl DatasetOperationDraft {
    pub(crate) fn new(id: u64, kind: DatasetOperationDraftKind) -> Self {
        Self {
            id,
            kind,
            source: None,
            add_type: String::new(),
            left_field_key: String::new(),
            right_field_key: String::new(),
            projection_fields: Vec::new(),
            aggregation: DatasetAggregationDraft::default(),
            calculated_fields: Vec::new(),
            row_filters: Vec::new(),
        }
    }
}
