//! Components feature DTOs.
#![cfg_attr(not(feature = "hydrate"), allow(dead_code))]

use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_json::Value;
use std::{cell::RefCell, collections::BTreeMap};

use crate::{ComponentDefinitionBootstrap, ComponentDirectoryItem, ComponentVersionBootstrap};

#[derive(Clone, PartialEq, Eq)]
pub(crate) struct ComponentSummary {
    pub(crate) id: String,
    pub(crate) name: String,
    pub(crate) slug: String,
    pub(crate) description: Option<String>,
    pub(crate) current_version_id: Option<String>,
    pub(crate) current_version_label: Option<String>,
    pub(crate) current_component_type: Option<String>,
    pub(crate) draft_version_id: Option<String>,
    pub(crate) draft_version_label: Option<String>,
}

impl From<ComponentDirectoryItem> for ComponentSummary {
    fn from(value: ComponentDirectoryItem) -> Self {
        Self {
            id: value.component_id,
            name: value.name,
            slug: value.slug,
            description: value.description,
            current_version_id: value.current_version_id,
            current_version_label: value.current_version_label,
            current_component_type: (!value.component_type.is_empty())
                .then_some(value.component_type),
            draft_version_id: value.draft_version_id,
            draft_version_label: value.draft_version_label,
        }
    }
}

impl<'de> Deserialize<'de> for ComponentSummary {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = Value::deserialize(deserializer)?;
        let versions = value.get("versions").and_then(Value::as_array);
        let current = value.get("current_version").or_else(|| {
            versions.and_then(|items| {
                items
                    .iter()
                    .find(|item| value_text(item, "publication_state") == "published")
                    .or_else(|| {
                        items
                            .iter()
                            .find(|item| value_text(item, "publication_state") == "draft")
                    })
                    .or_else(|| items.first())
            })
        });
        let draft = versions.and_then(|items| {
            items
                .iter()
                .find(|item| value_text(item, "publication_state") == "draft")
        });
        Ok(Self {
            id: value_text(&value, "component_id"),
            name: value_text(&value, "name"),
            slug: value_text(&value, "slug"),
            description: value
                .get("description")
                .and_then(Value::as_str)
                .map(str::to_string),
            current_version_id: current
                .map(|item| value_text(item, "component_version_id"))
                .filter(|item| !item.is_empty()),
            current_version_label: current
                .map(|item| value_text(item, "version_label"))
                .filter(|item| !item.is_empty()),
            current_component_type: current
                .map(|item| value_text(item, "component_type"))
                .filter(|item| !item.is_empty()),
            draft_version_id: draft
                .map(|item| value_text(item, "component_version_id"))
                .filter(|item| !item.is_empty()),
            draft_version_label: draft
                .map(|item| value_text(item, "version_label"))
                .filter(|item| !item.is_empty()),
        })
    }
}

#[derive(Clone, PartialEq)]
pub(crate) struct ComponentDefinition {
    pub(crate) id: String,
    pub(crate) name: String,
    pub(crate) slug: String,
    pub(crate) description: Option<String>,
    pub(crate) versions: Vec<ComponentVersionSummary>,
}

impl From<ComponentDefinitionBootstrap> for ComponentDefinition {
    fn from(value: ComponentDefinitionBootstrap) -> Self {
        let component_id = value.component_id;
        Self {
            id: component_id.clone(),
            name: value.name,
            slug: value.slug,
            description: value.description,
            versions: value
                .versions
                .into_iter()
                .map(|version| ComponentVersionSummary::from_bootstrap(&component_id, version))
                .collect(),
        }
    }
}

impl<'de> Deserialize<'de> for ComponentDefinition {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = Value::deserialize(deserializer)?;
        let component_id = value_text(&value, "component_id");
        let mut versions: Vec<ComponentVersionSummary> = serde_json::from_value(
            value
                .get("versions")
                .cloned()
                .unwrap_or_else(|| Value::Array(vec![])),
        )
        .map_err(serde::de::Error::custom)?;
        for version in &mut versions {
            if version.component_id.is_empty() {
                version.component_id.clone_from(&component_id);
            }
        }
        Ok(Self {
            id: component_id,
            name: value_text(&value, "name"),
            slug: value_text(&value, "slug"),
            description: value
                .get("description")
                .and_then(Value::as_str)
                .map(str::to_string),
            versions,
        })
    }
}

#[derive(Clone, PartialEq)]
pub(crate) struct ComponentVersionSummary {
    pub(crate) id: String,
    pub(crate) component_id: String,
    pub(crate) dataset_id: String,
    pub(crate) dataset_version_major: i32,
    pub(crate) binding_mode: String,
    pub(crate) component_type: String,
    pub(crate) status: String,
    pub(crate) lifecycle_state: Option<String>,
    pub(crate) resource_revision: i64,
    pub(crate) successor_version_id: Option<String>,
    pub(crate) version_label: String,
    pub(crate) version_note: String,
    pub(crate) config: Value,
}

impl<'de> Deserialize<'de> for ComponentVersionSummary {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = Value::deserialize(deserializer)?;
        let resource_id = value
            .pointer("/dataset_reference/reference/resource_id")
            .and_then(Value::as_str)
            .unwrap_or_default();
        let (dataset_id, major) = resource_id.rsplit_once('@').unwrap_or((resource_id, "1"));
        Ok(Self {
            id: value_text(&value, "component_version_id"),
            component_id: value_text(&value, "component_id"),
            dataset_id: dataset_id.to_string(),
            dataset_version_major: major.parse().unwrap_or(1),
            binding_mode: "fixed_major".into(),
            component_type: value_text(&value, "component_type"),
            status: value_text(&value, "publication_state"),
            lifecycle_state: value
                .get("lifecycle_state")
                .and_then(Value::as_str)
                .map(str::to_string),
            resource_revision: value
                .get("resource_revision")
                .and_then(Value::as_i64)
                .unwrap_or_default(),
            successor_version_id: None,
            version_label: value_text(&value, "version_label"),
            version_note: value_text(&value, "version_note"),
            config: value.get("config").cloned().unwrap_or(Value::Null),
        })
    }
}

impl ComponentVersionSummary {
    fn from_bootstrap(component_id: &str, value: ComponentVersionBootstrap) -> Self {
        let resource_id = value
            .dataset_reference
            .pointer("/reference/resource_id")
            .and_then(Value::as_str)
            .unwrap_or_default();
        let (dataset_id, major) = resource_id.rsplit_once('@').unwrap_or((resource_id, "1"));
        Self {
            id: value.component_version_id,
            component_id: component_id.to_string(),
            dataset_id: dataset_id.to_string(),
            dataset_version_major: major.parse().unwrap_or(1),
            binding_mode: "fixed_major".into(),
            component_type: value.component_type,
            status: value.publication_state,
            lifecycle_state: Some(value.lifecycle_state),
            resource_revision: i64::try_from(value.resource_revision).unwrap_or(i64::MAX),
            successor_version_id: None,
            version_label: value.version_label,
            version_note: value.version_note,
            config: value.config,
        }
    }
}

pub(crate) fn datasets_from_bootstrap(value: &Value) -> Vec<DatasetSummary> {
    let datasets = value
        .get("datasets")
        .cloned()
        .unwrap_or_else(|| Value::Array(Vec::new()));
    serde_json::from_value(datasets).unwrap_or_default()
}

#[derive(Clone)]
pub(crate) struct UpdateComponentRequest {
    pub(crate) name: String,
    pub(crate) slug: String,
    pub(crate) description: Option<String>,
}

impl Serialize for UpdateComponentRequest {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serde_json::json!({"schema_version":1,"name":self.name,"slug":self.slug,"description":self.description}).serialize(serializer)
    }
}

#[derive(Clone, Serialize)]
pub(crate) struct SaveComponentEditRequest {
    #[serde(rename = "schema_version")]
    pub(crate) schema_version: u16,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) component_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) draft_version_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) published_version_id: Option<String>,
    pub(crate) action: String,
    pub(crate) component: UpdateComponentRequest,
    pub(crate) version: CreateComponentVersionRequest,
}

#[derive(Clone, Deserialize, PartialEq, Eq)]
pub(crate) struct ComponentValidationResponse {
    pub(crate) valid: bool,
    pub(crate) findings: Vec<ComponentValidationFinding>,
}

#[derive(Clone, Deserialize, PartialEq, Eq)]
pub(crate) struct ComponentValidationFinding {
    pub(crate) code: String,
    #[serde(default = "default_error_severity")]
    pub(crate) severity: String,
    pub(crate) field_path: Option<String>,
    pub(crate) message: String,
}

#[derive(Clone)]
pub(crate) struct CreateComponentVersionRequest {
    pub(crate) dataset_id: Option<String>,
    pub(crate) dataset_version_major: Option<i32>,
    pub(crate) component_type: String,
    pub(crate) config: Value,
    pub(crate) version_note: Option<String>,
}

impl Serialize for CreateComponentVersionRequest {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let dataset_id = self
            .dataset_id
            .as_deref()
            .ok_or_else(|| serde::ser::Error::custom("dataset is required"))?;
        let major = self
            .dataset_version_major
            .ok_or_else(|| serde::ser::Error::custom("dataset major is required"))?;
        let reference = dataset_reference(dataset_id, major).ok_or_else(|| {
            serde::ser::Error::custom("selected Dataset reference is unavailable")
        })?;
        serde_json::json!({"dataset_reference":reference,"component_type":self.component_type,"config":self.config,"version_note":self.version_note.clone().unwrap_or_default()}).serialize(serializer)
    }
}

#[derive(Clone, PartialEq, Eq)]
pub(crate) struct IdResponse {
    pub(crate) id: String,
}

impl<'de> Deserialize<'de> for IdResponse {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = Value::deserialize(deserializer)?;
        let component_version_id = value_text(&value, "component_version_id");
        Ok(Self {
            id: if component_version_id.is_empty() {
                value_text(&value, "component_id")
            } else {
                component_version_id
            },
        })
    }
}

#[derive(Clone, Serialize)]
pub(crate) struct ComponentLifecycleRequest {
    pub(crate) schema_version: u16,
    pub(crate) action: String,
    pub(crate) expected_resource_revision: i64,
}

pub(crate) type ComponentLifecycleResponse = IdResponse;

#[derive(Clone, PartialEq, Eq)]
pub(crate) struct DatasetSummary {
    pub(crate) reference: Value,
    pub(crate) id: String,
    pub(crate) current_version_major: Option<i32>,
    pub(crate) major_versions: Vec<i32>,
    pub(crate) name: String,
    pub(crate) slug: String,
    pub(crate) grain: String,
    pub(crate) tags: Vec<String>,
    pub(crate) provenance: DatasetProvenanceSummary,
    pub(crate) output_fields: Vec<DatasetFieldDefinition>,
    pub(crate) revisions: Vec<DatasetRevisionFieldSummary>,
}

impl<'de> Deserialize<'de> for DatasetSummary {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = Value::deserialize(deserializer)?;
        let reference = value.get("reference").cloned().unwrap_or(Value::Null);
        let resource_id = reference
            .pointer("/reference/resource_id")
            .and_then(Value::as_str)
            .unwrap_or_default();
        let (id, major) = resource_id.rsplit_once('@').unwrap_or((resource_id, "1"));
        let id = id.to_string();
        let major = major.parse().unwrap_or(1);
        let output_fields = serde_json::from_value(
            value
                .get("fields")
                .cloned()
                .unwrap_or_else(|| Value::Array(vec![])),
        )
        .map_err(serde::de::Error::custom)?;
        Ok(Self {
            reference,
            id,
            current_version_major: Some(major),
            major_versions: vec![major],
            name: value_text(&value, "dataset_name"),
            slug: value_text(&value, "dataset_slug"),
            grain: value_text(&value, "grain"),
            tags: serde_json::from_value(
                value
                    .get("tags")
                    .cloned()
                    .unwrap_or_else(|| Value::Array(vec![])),
            )
            .unwrap_or_default(),
            provenance: DatasetProvenanceSummary::default(),
            output_fields,
            revisions: vec![],
        })
    }
}

#[derive(Clone, Deserialize, PartialEq, Eq)]
pub(crate) struct DatasetRevisionFieldSummary {
    pub(crate) version_number: i32,
    pub(crate) version_major: Option<i32>,
    pub(crate) output_fields: Vec<DatasetFieldDefinition>,
}

#[derive(Clone, Deserialize, PartialEq, Eq)]
pub(crate) struct DatasetDistinctValues {
    pub(crate) dataset_id: String,
    pub(crate) version_major: i32,
    pub(crate) field: String,
    pub(crate) values: Vec<String>,
}

#[derive(Clone, Default, Deserialize, PartialEq, Eq)]
pub(crate) struct DatasetProvenanceSummary {
    #[serde(default)]
    pub(crate) forms: Vec<DatasetProvenanceItem>,
    #[serde(default)]
    pub(crate) datasets: Vec<DatasetProvenanceItem>,
}

#[derive(Clone, Deserialize, PartialEq, Eq)]
pub(crate) struct DatasetProvenanceItem {
    pub(crate) id: String,
    pub(crate) name: String,
    pub(crate) slug: Option<String>,
}

#[derive(Clone, Deserialize, PartialEq, Eq)]
pub(crate) struct DatasetFieldDefinition {
    #[serde(alias = "field_key")]
    pub(crate) key: String,
    pub(crate) label: String,
    #[serde(alias = "data_type")]
    pub(crate) field_type: String,
    #[serde(default)]
    pub(crate) restriction_tier: Option<String>,
}

thread_local! {
    static DATASET_REFERENCES: RefCell<BTreeMap<(String, i32), Value>> = const { RefCell::new(BTreeMap::new()) };
}

pub(crate) fn remember_dataset_references(datasets: &[DatasetSummary]) {
    DATASET_REFERENCES.with(|references| {
        let mut references = references.borrow_mut();
        references.clear();
        for dataset in datasets {
            for major in &dataset.major_versions {
                references.insert((dataset.id.clone(), *major), dataset.reference.clone());
            }
        }
    });
}

pub(crate) fn dataset_reference(dataset_id: &str, major: i32) -> Option<Value> {
    DATASET_REFERENCES.with(|references| {
        references
            .borrow()
            .get(&(dataset_id.to_string(), major))
            .cloned()
    })
}

fn value_text(value: &Value, key: &str) -> String {
    value
        .get(key)
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_string()
}

fn default_error_severity() -> String {
    "error".into()
}
