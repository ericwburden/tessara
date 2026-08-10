//! Private wire DTOs for exact-version Component execution responses.
#![cfg_attr(not(feature = "hydrate"), allow(dead_code))]

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

#[derive(Clone, Deserialize, PartialEq)]
pub(crate) struct ComponentTable {
    pub(crate) component_id: String,
    pub(crate) component_version_id: String,
    pub(crate) dataset_id: String,
    pub(crate) dataset_version_major: i32,
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
    pub component_id: String,
    pub component_version_id: String,
    pub dataset_id: String,
    pub dataset_version_major: i32,
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
