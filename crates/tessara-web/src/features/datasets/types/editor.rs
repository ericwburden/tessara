//! Editor-local dataset draft types.

#[derive(Clone, Debug, PartialEq)]
pub(in crate::features::datasets) struct DatasetSourceDraft {
    pub(in crate::features::datasets) input_kind: String,
    pub(in crate::features::datasets) source_alias: String,
    pub(in crate::features::datasets) form_id: String,
    pub(in crate::features::datasets) form_version_id: String,
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

#[derive(Clone, Debug, Default, PartialEq)]
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
pub(in crate::features::datasets) struct DatasetRowFilterDraft {
    pub(in crate::features::datasets) id: u64,
    pub(in crate::features::datasets) field_key: String,
    pub(in crate::features::datasets) operator: String,
    pub(in crate::features::datasets) value: String,
    pub(in crate::features::datasets) value_mode: String,
    pub(in crate::features::datasets) value_field_key: String,
}

#[derive(Clone, Debug, PartialEq)]
pub(in crate::features::datasets) struct DatasetCalculatedFieldDraft {
    pub(in crate::features::datasets) id: u64,
    pub(in crate::features::datasets) key: String,
    pub(in crate::features::datasets) label: String,
    pub(in crate::features::datasets) base_field_key: String,
    pub(in crate::features::datasets) functions: Vec<DatasetCalculationFunctionDraft>,
}

#[derive(Clone, Debug, PartialEq)]
pub(in crate::features::datasets) struct DatasetCalculationFunctionDraft {
    pub(in crate::features::datasets) id: u64,
    pub(in crate::features::datasets) function: String,
    pub(in crate::features::datasets) argument: String,
    pub(in crate::features::datasets) argument_mode: String,
    pub(in crate::features::datasets) argument_field_key: String,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::features::datasets) enum DatasetOperationDraftKind {
    AddSource,
    Projection,
    Aggregation,
    CalculatedFields,
    Filter,
}

#[derive(Clone, Debug, PartialEq)]
pub(in crate::features::datasets) struct DatasetOperationDraft {
    pub(in crate::features::datasets) id: u64,
    pub(in crate::features::datasets) kind: DatasetOperationDraftKind,
    pub(in crate::features::datasets) source: Option<DatasetSourceDraft>,
    pub(in crate::features::datasets) add_type: String,
    pub(in crate::features::datasets) left_field_key: String,
    pub(in crate::features::datasets) right_field_key: String,
    pub(in crate::features::datasets) projection_fields: Vec<DatasetFieldDraft>,
    pub(in crate::features::datasets) aggregation: DatasetAggregationDraft,
    pub(in crate::features::datasets) calculated_fields: Vec<DatasetCalculatedFieldDraft>,
    pub(in crate::features::datasets) row_filters: Vec<DatasetRowFilterDraft>,
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
        }
    }
}

impl DatasetOperationDraftKind {
    pub(in crate::features::datasets) fn label(self) -> &'static str {
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
    pub(in crate::features::datasets) fn new(id: u64, kind: DatasetOperationDraftKind) -> Self {
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
