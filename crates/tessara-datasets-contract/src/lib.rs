//! Exact Core compatibility contract used by Components while Datasets remains
//! in process. The contract exposes Dataset-owned catalog, schema,
//! compatibility, distinct-value, and execution behavior without exposing
//! Dataset storage or implementation types.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;
use tessara_module_contract::{ResourceOwner, TypedResourceReference};
use uuid::Uuid;

pub const DATASET_CONTRACT_SCHEMA_VERSION: u16 = 1;
pub const DATASET_CONTRACT_VERSION: &str = "1.0.0";
pub const DATASET_CONTRACT_ID: &str = "tessara.datasets.dataset-major-line";
pub const DATASET_BINDING_KEY: &str = "tessara.components.dataset-major-line";
pub const DATASET_RESOURCE_TYPE: &str = "tessara.transition.dataset_major_line";

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct DatasetMajorLineReference {
    reference: TypedResourceReference,
}

impl DatasetMajorLineReference {
    pub fn new(
        reference: TypedResourceReference,
    ) -> Result<Self, DatasetMajorLineReferenceValidationError> {
        reference.validate()?;
        if !matches!(reference.owner(), ResourceOwner::CoreInstallation { .. }) {
            return Err(DatasetMajorLineReferenceValidationError::ExpectedCoreInstallationOwner);
        }
        if reference.resource_type().as_str() != DATASET_RESOURCE_TYPE {
            return Err(
                DatasetMajorLineReferenceValidationError::UnexpectedResourceType {
                    actual: reference.resource_type().as_str().to_string(),
                },
            );
        }
        parse_resource_id(reference.resource_id())?;
        Ok(Self { reference })
    }

    pub fn from_parts(
        installation_id: Uuid,
        dataset_id: Uuid,
        major: u32,
    ) -> Result<Self, DatasetMajorLineReferenceValidationError> {
        if major == 0 {
            return Err(DatasetMajorLineReferenceValidationError::InvalidMajor);
        }
        Self::new(TypedResourceReference::new(
            installation_id,
            ResourceOwner::CoreInstallation { installation_id },
            DATASET_RESOURCE_TYPE
                .parse()
                .expect("Dataset resource type constant is valid"),
            format!("{dataset_id}@{major}"),
        )?)
    }

    pub const fn reference(&self) -> &TypedResourceReference {
        &self.reference
    }

    pub fn dataset_id(&self) -> Uuid {
        parse_resource_id(self.reference.resource_id())
            .expect("validated Dataset major-line reference")
            .0
    }

    pub fn major(&self) -> u32 {
        parse_resource_id(self.reference.resource_id())
            .expect("validated Dataset major-line reference")
            .1
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct DatasetMajorLineReferenceWire {
    reference: TypedResourceReference,
}

impl<'de> Deserialize<'de> for DatasetMajorLineReference {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let wire = DatasetMajorLineReferenceWire::deserialize(deserializer)?;
        Self::new(wire.reference).map_err(serde::de::Error::custom)
    }
}

#[derive(Clone, Debug, Eq, PartialEq, thiserror::Error)]
pub enum DatasetMajorLineReferenceValidationError {
    #[error(transparent)]
    InvalidReference(#[from] tessara_module_contract::ReferenceValidationError),
    #[error("Dataset major-line references must be Core installation owned")]
    ExpectedCoreInstallationOwner,
    #[error(
        "Dataset major-line reference has resource type '{actual}', expected \
         'tessara.transition.dataset_major_line'"
    )]
    UnexpectedResourceType { actual: String },
    #[error("Dataset major-line resource identity must be '<canonical-uuid>@<positive-major>'")]
    InvalidResourceId,
    #[error("Dataset major-line major version must be positive")]
    InvalidMajor,
}

fn parse_resource_id(value: &str) -> Result<(Uuid, u32), DatasetMajorLineReferenceValidationError> {
    let Some((dataset, major_text)) = value.split_once('@') else {
        return Err(DatasetMajorLineReferenceValidationError::InvalidResourceId);
    };
    if major_text.contains('@') {
        return Err(DatasetMajorLineReferenceValidationError::InvalidResourceId);
    }
    let dataset_id = Uuid::parse_str(dataset)
        .map_err(|_| DatasetMajorLineReferenceValidationError::InvalidResourceId)?;
    if dataset_id.to_string() != dataset {
        return Err(DatasetMajorLineReferenceValidationError::InvalidResourceId);
    }
    let major = major_text
        .parse::<u32>()
        .map_err(|_| DatasetMajorLineReferenceValidationError::InvalidResourceId)?;
    if major == 0 || major.to_string() != major_text {
        return Err(DatasetMajorLineReferenceValidationError::InvalidMajor);
    }
    Ok((dataset_id, major))
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DatasetAction {
    Catalog,
    ResolveSchema,
    DistinctValues,
    CheckCompatibility,
    Execute,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DatasetFieldContract {
    pub key: String,
    pub label: String,
    pub field_type: String,
    pub restriction_tier: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DatasetMajorLineMetadata {
    pub reference: DatasetMajorLineReference,
    pub dataset_name: String,
    pub dataset_slug: String,
    pub materialization_state: String,
    pub fields: Vec<DatasetFieldContract>,
    pub scope_node_ids: Vec<Uuid>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DatasetCatalogRequest {
    #[serde(deserialize_with = "deserialize_schema_version")]
    pub schema_version: u16,
    pub action: DatasetAction,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DatasetCatalogResponse {
    #[serde(deserialize_with = "deserialize_schema_version")]
    pub schema_version: u16,
    pub datasets: Vec<DatasetMajorLineMetadata>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DatasetSchemaRequest {
    #[serde(deserialize_with = "deserialize_schema_version")]
    pub schema_version: u16,
    pub action: DatasetAction,
    pub reference: DatasetMajorLineReference,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DatasetDistinctValuesRequest {
    #[serde(deserialize_with = "deserialize_schema_version")]
    pub schema_version: u16,
    pub action: DatasetAction,
    pub reference: DatasetMajorLineReference,
    pub field_key: String,
    pub limit: u16,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DatasetDistinctValuesResponse {
    #[serde(deserialize_with = "deserialize_schema_version")]
    pub schema_version: u16,
    pub values: Vec<Value>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DatasetFieldRequirement {
    pub field_key: String,
    pub accepted_types: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DatasetCompatibilityRequest {
    #[serde(deserialize_with = "deserialize_schema_version")]
    pub schema_version: u16,
    pub action: DatasetAction,
    pub reference: DatasetMajorLineReference,
    pub required_fields: Vec<DatasetFieldRequirement>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DatasetCompatibilityFinding {
    pub code: String,
    pub field_key: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DatasetCompatibilityResponse {
    #[serde(deserialize_with = "deserialize_schema_version")]
    pub schema_version: u16,
    pub compatible: bool,
    pub findings: Vec<DatasetCompatibilityFinding>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DatasetFilterOperator {
    Eq,
    NotEq,
    Contains,
    StartsWith,
    EndsWith,
    GreaterThan,
    GreaterThanOrEqual,
    LessThan,
    LessThanOrEqual,
    IsNull,
    IsNotNull,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DatasetFilter {
    pub field_key: String,
    pub operator: DatasetFilterOperator,
    #[serde(default)]
    pub value: Option<Value>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DatasetAggregateFunction {
    Count,
    Sum,
    Average,
    Minimum,
    Maximum,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DatasetAggregate {
    pub field_key: Option<String>,
    pub function: DatasetAggregateFunction,
    pub output_key: String,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DatasetSortDirection {
    Asc,
    Desc,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DatasetSort {
    pub field_key: String,
    pub direction: DatasetSortDirection,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DatasetExecutionRequest {
    #[serde(deserialize_with = "deserialize_schema_version")]
    pub schema_version: u16,
    pub action: DatasetAction,
    pub reference: DatasetMajorLineReference,
    #[serde(default)]
    pub projection: Vec<String>,
    #[serde(default)]
    pub filters: Vec<DatasetFilter>,
    #[serde(default)]
    pub group_by: Vec<String>,
    #[serde(default)]
    pub aggregates: Vec<DatasetAggregate>,
    #[serde(default)]
    pub order_by: Vec<DatasetSort>,
    pub limit: u32,
    #[serde(default)]
    pub cursor: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DatasetExecutionRow {
    pub row_id: String,
    pub values: BTreeMap<String, Option<Value>>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DatasetExecutionResponse {
    #[serde(deserialize_with = "deserialize_schema_version")]
    pub schema_version: u16,
    pub materialization_state: String,
    pub fields: Vec<DatasetFieldContract>,
    pub rows: Vec<DatasetExecutionRow>,
    pub next_cursor: Option<String>,
}

fn deserialize_schema_version<'de, D>(deserializer: D) -> Result<u16, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let version = u16::deserialize(deserializer)?;
    if version != DATASET_CONTRACT_SCHEMA_VERSION {
        return Err(serde::de::Error::custom(format!(
            "Dataset contract schema version {version} is unsupported; expected {DATASET_CONTRACT_SCHEMA_VERSION}"
        )));
    }
    Ok(version)
}

#[cfg(test)]
mod tests {
    use serde_json::json;
    use tessara_module_contract::{ResourceOwner, TypedResourceReference};

    use super::*;

    const INSTALLATION_ID: Uuid = Uuid::from_u128(1);
    const DATASET_ID: Uuid = Uuid::from_u128(2);

    #[test]
    fn canonical_dataset_major_line_round_trips() {
        let reference =
            DatasetMajorLineReference::from_parts(INSTALLATION_ID, DATASET_ID, 3).unwrap();
        assert_eq!(reference.dataset_id(), DATASET_ID);
        assert_eq!(reference.major(), 3);
        assert_eq!(
            reference.reference().resource_id(),
            format!("{DATASET_ID}@3")
        );
        let wire = serde_json::to_value(&reference).unwrap();
        assert_eq!(
            serde_json::from_value::<DatasetMajorLineReference>(wire).unwrap(),
            reference
        );
    }

    #[test]
    fn wrong_owner_type_and_noncanonical_identity_fail() {
        let module_owned = TypedResourceReference::new(
            INSTALLATION_ID,
            ResourceOwner::ModuleInstance {
                installation_id: INSTALLATION_ID,
                module_instance_id: Uuid::from_u128(3),
            },
            DATASET_RESOURCE_TYPE.parse().unwrap(),
            format!("{DATASET_ID}@1"),
        )
        .unwrap();
        assert!(DatasetMajorLineReference::new(module_owned).is_err());

        let wrong_type = TypedResourceReference::new(
            INSTALLATION_ID,
            ResourceOwner::CoreInstallation {
                installation_id: INSTALLATION_ID,
            },
            "tessara.transition.dataset_revision".parse().unwrap(),
            format!("{DATASET_ID}@1"),
        )
        .unwrap();
        assert!(DatasetMajorLineReference::new(wrong_type).is_err());

        let invalid = json!({
            "reference": {
                "installation_id": INSTALLATION_ID,
                "owner": {"kind": "core_installation", "installation_id": INSTALLATION_ID},
                "resource_type": DATASET_RESOURCE_TYPE,
                "resource_id": format!("{}@0", DATASET_ID.to_string().to_uppercase())
            }
        });
        assert!(serde_json::from_value::<DatasetMajorLineReference>(invalid).is_err());
    }

    #[test]
    fn old_schema_and_unknown_fields_fail_closed() {
        let reference =
            DatasetMajorLineReference::from_parts(INSTALLATION_ID, DATASET_ID, 1).unwrap();
        let request = DatasetSchemaRequest {
            schema_version: 1,
            action: DatasetAction::ResolveSchema,
            reference,
        };
        let mut wire = serde_json::to_value(request).unwrap();
        wire["schema_version"] = json!(0);
        assert!(serde_json::from_value::<DatasetSchemaRequest>(wire.clone()).is_err());
        wire["schema_version"] = json!(1);
        wire.as_object_mut()
            .unwrap()
            .insert("fallback".into(), json!(true));
        assert!(serde_json::from_value::<DatasetSchemaRequest>(wire).is_err());
    }
}
