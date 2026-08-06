//! Dashboard-private wire DTOs for exact Component execution responses.
#![cfg_attr(not(feature = "hydrate"), allow(dead_code))]

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use tessara_datasets_contract::DatasetMajorLineReference;

#[derive(Clone, Deserialize, PartialEq)]
pub(crate) struct ComponentTable {
    pub(crate) schema_version: u16,
    pub(crate) component_id: String,
    pub(crate) component_version_id: String,
    pub(crate) dataset_reference: DatasetMajorLineReference,
    pub(crate) component_type: String,
    pub(crate) materialization_state: String,
    pub(crate) columns: Vec<ComponentTableColumn>,
    pub(crate) rows: Vec<ComponentTableRow>,
    pub(crate) pagination: ComponentTablePagination,
}

#[derive(Clone, Deserialize, PartialEq, Eq)]
pub(crate) struct ComponentTablePagination {
    pub(crate) page_size: usize,
    pub(crate) next_cursor: Option<String>,
    pub(crate) has_more: bool,
}

#[derive(Clone, Deserialize, PartialEq, Eq)]
pub(crate) struct ComponentTableColumn {
    pub(crate) key: String,
    pub(crate) label: String,
    pub(crate) field_type: String,
}

#[derive(Clone, Deserialize, PartialEq, Eq)]
pub(crate) struct ComponentTableRow {
    pub(crate) row_id: String,
    pub(crate) values: BTreeMap<String, Option<String>>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct ComponentVisual {
    pub schema_version: u16,
    pub component_id: String,
    pub component_version_id: String,
    pub dataset_reference: DatasetMajorLineReference,
    pub component_type: String,
    pub materialization_state: String,
    pub value_format: String,
    pub legend_title: Option<String>,
    pub bar_orientation: Option<String>,
    pub bar_comparison_layout: Option<String>,
    pub x_axis_label: Option<String>,
    pub y_axis_label: Option<String>,
    #[serde(default)]
    pub line_smoothing: Option<bool>,
    pub stat: Option<ComponentStatValue>,
    pub points: Vec<ComponentVisualPoint>,
    pub slices: Vec<ComponentVisualSlice>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct ComponentStatValue {
    pub label: String,
    pub value: Option<f64>,
    pub display_value: Option<String>,
    pub supporting_text: Option<String>,
    pub panel_style: String,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct ComponentVisualPoint {
    pub x: String,
    pub value: f64,
    pub display_value: String,
    pub color: Option<String>,
    pub comparison: Option<String>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct ComponentVisualSlice {
    pub category: String,
    pub value: f64,
    pub display_value: String,
    pub color: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::ComponentVisual;

    #[test]
    fn canonical_component_visual_response_deserializes() {
        let response = r#"{
            "bar_comparison_layout":null,
            "bar_orientation":null,
            "component_id":"01980000-0002-7000-8000-000000000011",
            "component_type":"bar",
            "component_version_id":"01980000-0001-7000-8000-000000000011",
            "dataset_reference":{"reference":{"installation_id":"01980000-0000-7000-8000-00000000008a","owner":{"installation_id":"01980000-0000-7000-8000-00000000008a","kind":"core_installation"},"resource_id":"01980000-0002-7000-8000-000000000003@1","resource_type":"tessara.transition.dataset_major_line"}},
            "legend_title":null,
            "line_smoothing":null,
            "materialization_state":"ready",
            "points":[{"color":null,"comparison":null,"display_value":"1","value":1.0,"x":"PUBLIC"}],
            "schema_version":1,
            "slices":[],
            "stat":null,
            "value_format":"integer",
            "x_axis_label":null,
            "y_axis_label":null
        }"#;

        let visual: ComponentVisual =
            serde_json::from_str(response).expect("canonical Component visual response");
        assert_eq!(visual.schema_version, 1);
        assert_eq!(visual.component_type, "bar");
        assert_eq!(visual.dataset_reference.major(), 1);
        assert_eq!(visual.points.len(), 1);
    }
}
