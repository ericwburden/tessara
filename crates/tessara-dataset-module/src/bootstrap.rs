//! Dataset-owned composition bootstrap.

use std::collections::{BTreeMap, BTreeSet};

use axum::{
    Json,
    extract::{Request, State},
};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tessara_composition::{BootstrapReceiptV1, OwnerBootstrapRequestV1, OwnerBootstrapResponseV1};
use tessara_datasets_contract::{
    DatasetAuthoringRequestV1, DatasetMajorLineReference, DatasetProductSourceV1, DatasetReference,
    DatasetRevisionReference,
};
use tessara_module_contract::{AuthorizationAudienceV1, ModuleDefinitionId, ResourceOwner};

use crate::{DatasetModuleError, DatasetModuleState, MODULE_DEFINITION_ID, load_security_state};

pub const DATASET_BOOTSTRAP_SCHEMA_VERSION: &str = "tessara.io/dataset-bootstrap/v1";

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct DatasetBootstrapV1 {
    pub schema_version: String,
    pub datasets: Vec<DatasetBootstrapDefinitionV1>,
    #[serde(default)]
    pub expected_rejections: Vec<DatasetBootstrapRejectionV1>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct DatasetBootstrapDefinitionV1 {
    pub resource_key: String,
    pub definition: Value,
    #[serde(default)]
    pub reference_bindings: Vec<DatasetBootstrapReferenceBindingV1>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct DatasetBootstrapReferenceBindingV1 {
    pub target_pointer: String,
    pub source_resource_key: String,
    pub source_alias: String,
    pub selector: DatasetBootstrapReferenceSelectorV1,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum DatasetBootstrapReferenceSelectorV1 {
    Revision,
    MajorLine { version_major: i32 },
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct DatasetBootstrapRejectionV1 {
    pub resource_key: String,
    pub source_resource_keys: Vec<String>,
    pub expected_code: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DatasetBootstrapReadBackV2 {
    pub schema_version: u16,
    pub dataset: DatasetReference,
    pub revision: DatasetRevisionReference,
    pub major_line: DatasetMajorLineReference,
    pub materialized_row_count: i64,
}

pub(crate) async fn apply_bootstrap(
    State(state): State<DatasetModuleState>,
    request: Request,
) -> Result<Json<OwnerBootstrapResponseV1>, DatasetModuleError> {
    crate::require_control_key(request.headers())?;
    crate::require_json_content_type(request.headers())?;
    let request: OwnerBootstrapRequestV1<DatasetBootstrapV1> =
        crate::decode_bounded_json(request, "Dataset bootstrap").await?;
    if request.input.schema_version != DATASET_BOOTSTRAP_SCHEMA_VERSION
        || request.apply_sequence == 0
        || request.dependency_validation.is_some()
        || request.idempotency_key.trim().is_empty()
        || !request
            .validate_input_digest()
            .map_err(|error| DatasetModuleError::BadRequest(error.to_string()))?
    {
        return Err(DatasetModuleError::BadRequest(
            "Dataset bootstrap contract or digest is invalid".into(),
        ));
    }
    let security = load_security_state(&state.pool).await?.ok_or_else(|| {
        DatasetModuleError::Unavailable("Dataset security state is unavailable".into())
    })?;
    let owner = AuthorizationAudienceV1::ModuleInstance {
        module_instance_id: security.module_instance_id,
        module_definition_id: ModuleDefinitionId::new(MODULE_DEFINITION_ID)
            .map_err(|error| DatasetModuleError::Internal(error.to_string()))?,
    };
    request
        .validate_authorization_for(
            &state.core_owner_bootstrap_verifier,
            &owner,
            MODULE_DEFINITION_ID,
            Utc::now(),
        )
        .map_err(|_| DatasetModuleError::Forbidden)?;
    validate_input(&request.input)?;

    if let Some((locked_input_digest, input_digest, desired_revision, apply_sequence, stored)) =
        sqlx::query_as::<_, (String, String, i64, i64, Value)>(
            "SELECT locked_input_digest,input_digest,desired_revision,apply_sequence,receipt
             FROM dataset_bootstrap_receipts WHERE idempotency_key=$1",
        )
        .bind(&request.idempotency_key)
        .fetch_optional(&state.pool)
        .await?
    {
        if locked_input_digest != request.locked_input_digest.to_string()
            || input_digest != request.input_digest.to_string()
            || desired_revision != request.desired_revision as i64
            || apply_sequence != request.apply_sequence as i64
        {
            return Err(DatasetModuleError::Conflict(
                "Dataset bootstrap idempotency key was reused with different locked input or apply identity"
                    .into(),
            ));
        }
        let mut response: OwnerBootstrapResponseV1 = serde_json::from_value(stored)
            .map_err(|error| DatasetModuleError::Internal(error.to_string()))?;
        response.receipt.changed = false;
        response.signed_receipt = state
            .bootstrap_receipt_signer
            .sign(response.receipt.clone())
            .map_err(|error| DatasetModuleError::Internal(error.to_string()))?;
        return Ok(Json(response));
    }

    let bootstrap_grant = crate::provider_client::BootstrapProviderAuthorization {
        authorization: request.authorization.clone(),
    };
    let mut transaction = state.pool.begin().await?;
    crate::dependency_dag::lock_dependency_graph_in_transaction(&mut transaction).await?;
    let mut read_backs = BTreeMap::new();
    let mut resources = BTreeMap::new();
    for dataset in &request.input.datasets {
        let definition = resolve_definition(
            dataset,
            &read_backs,
            security.installation_id,
            security.module_instance_id,
        )?;
        let created_dataset = crate::product::create_bootstrap_dataset_in_transaction(
            &state,
            &bootstrap_grant,
            &mut transaction,
            Some(&dataset.resource_key),
            &definition,
        )
        .await?;
        let row_count: i64 =
            sqlx::query_scalar("SELECT materialized_row_count FROM dataset_revisions WHERE id=$1")
                .bind(created_dataset.revision_id)
                .fetch_one(&mut *transaction)
                .await?;
        let read_back = DatasetBootstrapReadBackV2 {
            schema_version: 2,
            dataset: DatasetReference::from_parts(
                security.installation_id,
                security.module_instance_id,
                created_dataset.dataset_id,
            )
            .map_err(|error| DatasetModuleError::Internal(error.to_string()))?,
            revision: DatasetRevisionReference::from_parts(
                security.installation_id,
                security.module_instance_id,
                created_dataset.revision_id,
            )
            .map_err(|error| DatasetModuleError::Internal(error.to_string()))?,
            major_line: DatasetMajorLineReference::from_parts(
                security.installation_id,
                security.module_instance_id,
                created_dataset.dataset_id,
                1,
            )
            .map_err(|error| DatasetModuleError::Internal(error.to_string()))?,
            materialized_row_count: row_count,
        };
        resources.insert(
            dataset.resource_key.clone(),
            serde_json::to_string(&read_back.major_line)
                .map_err(|error| DatasetModuleError::Internal(error.to_string()))?,
        );
        resources.insert(
            format!("{}.read_back", dataset.resource_key),
            serde_json::to_string(&read_back)
                .map_err(|error| DatasetModuleError::Internal(error.to_string()))?,
        );
        read_backs.insert(dataset.resource_key.clone(), read_back);
    }
    let result_digest = tessara_composition::canonical_digest(&resources)
        .map_err(|error| DatasetModuleError::Internal(error.to_string()))?;
    let response = OwnerBootstrapResponseV1::signed(
        BootstrapReceiptV1 {
            owner: MODULE_DEFINITION_ID.into(),
            schema_version: DATASET_BOOTSTRAP_SCHEMA_VERSION.into(),
            input_digest: request.input_digest.clone(),
            result_digest,
            changed: true,
            resource_ids: resources,
        },
        &state.bootstrap_receipt_signer,
    )
    .map_err(|error| DatasetModuleError::Internal(error.to_string()))?;
    sqlx::query(
        "INSERT INTO dataset_bootstrap_receipts
         (idempotency_key,locked_input_digest,input_digest,desired_revision,apply_sequence,authority_jti,receipt)
         VALUES($1,$2,$3,$4,$5,$6,$7)",
    )
    .bind(&request.idempotency_key)
    .bind(request.locked_input_digest.to_string())
    .bind(request.input_digest.to_string())
    .bind(request.desired_revision as i64)
    .bind(request.apply_sequence as i64)
    .bind(request.authorization.payload.jti)
    .bind(
        serde_json::to_value(&response)
            .map_err(|error| DatasetModuleError::Internal(error.to_string()))?,
    )
    .execute(&mut *transaction)
    .await?;
    transaction.commit().await?;
    Ok(Json(response))
}

fn validate_input(input: &DatasetBootstrapV1) -> Result<(), DatasetModuleError> {
    if input.datasets.is_empty() {
        return Err(DatasetModuleError::ValidationFailed(
            "Dataset bootstrap must contain at least one Dataset".into(),
        ));
    }
    let mut keys = BTreeSet::new();
    if input.datasets.iter().any(|dataset| {
        dataset.resource_key.trim().is_empty()
            || dataset.resource_key.ends_with(".read_back")
            || !keys.insert(dataset.resource_key.as_str())
    }) {
        return Err(DatasetModuleError::ValidationFailed(
            "Dataset bootstrap resource keys must be non-empty, unique, and not use the reserved '.read_back' suffix".into(),
        ));
    }
    for dataset in &input.datasets {
        let mut targets = BTreeSet::new();
        for binding in &dataset.reference_bindings {
            if !is_canonical_json_pointer(&binding.target_pointer)
                || binding.source_resource_key.trim().is_empty()
                || binding.source_alias.trim().is_empty()
                || !targets.insert(binding.target_pointer.as_str())
                || matches!(
                    binding.selector,
                    DatasetBootstrapReferenceSelectorV1::MajorLine { version_major }
                        if version_major <= 0
                )
            {
                return Err(DatasetModuleError::ValidationFailed(
                    "Dataset bootstrap reference bindings must have unique canonical targets, a prior logical source key, a source alias, and a valid selector".into(),
                ));
            }
            let target = dataset
                .definition
                .pointer(&binding.target_pointer)
                .ok_or_else(|| {
                    DatasetModuleError::ValidationFailed(
                        "Dataset bootstrap reference binding target does not exist".into(),
                    )
                })?;
            if !target.is_null() {
                return Err(DatasetModuleError::ValidationFailed(
                    "Dataset bootstrap reference binding targets must be null before resolution"
                        .into(),
                ));
            }
        }
    }
    validate_expected_rejections(input)?;
    Ok(())
}

fn resolve_definition(
    dataset: &DatasetBootstrapDefinitionV1,
    read_backs: &BTreeMap<String, DatasetBootstrapReadBackV2>,
    installation_id: uuid::Uuid,
    module_instance_id: uuid::Uuid,
) -> Result<DatasetAuthoringRequestV1, DatasetModuleError> {
    let mut definition = dataset.definition.clone();
    for binding in &dataset.reference_bindings {
        let source = read_backs
            .get(&binding.source_resource_key)
            .ok_or_else(|| {
                DatasetModuleError::ValidationFailed(
                    "Dataset bootstrap references must resolve to an earlier logical resource key"
                        .into(),
                )
            })?;
        require_local_read_back(source, installation_id, module_instance_id)?;
        let resolved = match binding.selector {
            DatasetBootstrapReferenceSelectorV1::Revision => DatasetProductSourceV1::Dataset {
                alias: binding.source_alias.clone(),
                dataset_id: source.dataset.dataset_id().to_string(),
                dataset_revision_id: source.revision.revision_id().to_string(),
            },
            DatasetBootstrapReferenceSelectorV1::MajorLine { version_major } => {
                if source.major_line.major() != version_major {
                    return Err(DatasetModuleError::ValidationFailed(
                        "Dataset bootstrap major-line selector does not match the owner read-back"
                            .into(),
                    ));
                }
                DatasetProductSourceV1::DatasetMajor {
                    alias: binding.source_alias.clone(),
                    dataset_id: source.major_line.dataset_id().to_string(),
                    version_major,
                }
            }
        };
        let target = definition
            .pointer_mut(&binding.target_pointer)
            .ok_or_else(|| {
                DatasetModuleError::ValidationFailed(
                    "Dataset bootstrap reference binding target does not exist".into(),
                )
            })?;
        if !target.is_null() {
            return Err(DatasetModuleError::ValidationFailed(
                "Dataset bootstrap reference binding target was already occupied".into(),
            ));
        }
        *target = serde_json::to_value(resolved)
            .map_err(|error| DatasetModuleError::Internal(error.to_string()))?;
    }
    let definition: DatasetAuthoringRequestV1 = serde_json::from_value(definition).map_err(|error| {
        DatasetModuleError::ValidationFailed(format!(
            "Dataset bootstrap authoring definition is invalid after reference resolution: {error}"
        ))
    })?;
    let resolved_dataset_source_count = dataset_sources(&definition)
        .filter(|source| {
            matches!(
                source,
                DatasetProductSourceV1::Dataset { .. }
                    | DatasetProductSourceV1::DatasetMajor { .. }
            )
        })
        .count();
    if resolved_dataset_source_count != dataset.reference_bindings.len() {
        return Err(DatasetModuleError::ValidationFailed(
            "Every Dataset-backed bootstrap source must be resolved from one prior logical resource binding"
                .into(),
        ));
    }
    Ok(definition)
}

fn dataset_sources(
    definition: &DatasetAuthoringRequestV1,
) -> impl Iterator<Item = &DatasetProductSourceV1> {
    std::iter::once(&definition.initial_source).chain(definition.operations.iter().filter_map(
        |operation| match operation {
            tessara_datasets_contract::DatasetProductOperationV1::AddSource { source, .. } => {
                Some(source)
            }
            _ => None,
        },
    ))
}

fn require_local_read_back(
    read_back: &DatasetBootstrapReadBackV2,
    installation_id: uuid::Uuid,
    module_instance_id: uuid::Uuid,
) -> Result<(), DatasetModuleError> {
    let references = [
        read_back.dataset.reference(),
        read_back.revision.reference(),
        read_back.major_line.reference(),
    ];
    if references.iter().any(|reference| {
        reference.installation_id() != installation_id
            || reference.owner()
                != &(ResourceOwner::ModuleInstance {
                    installation_id,
                    module_instance_id,
                })
    }) || read_back.dataset.dataset_id() != read_back.major_line.dataset_id()
    {
        return Err(DatasetModuleError::ValidationFailed(
            "Dataset bootstrap source read-back has a substituted owner or identity".into(),
        ));
    }
    Ok(())
}

fn is_canonical_json_pointer(pointer: &str) -> bool {
    if pointer.is_empty() || !pointer.starts_with('/') {
        return false;
    }
    let mut characters = pointer.chars();
    while let Some(character) = characters.next() {
        if character == '~' && !matches!(characters.next(), Some('0' | '1')) {
            return false;
        }
    }
    true
}

fn validate_expected_rejections(input: &DatasetBootstrapV1) -> Result<(), DatasetModuleError> {
    let mut graph = BTreeMap::<&str, BTreeSet<&str>>::new();
    for dataset in &input.datasets {
        graph.insert(
            dataset.resource_key.as_str(),
            dataset
                .reference_bindings
                .iter()
                .map(|binding| binding.source_resource_key.as_str())
                .collect(),
        );
    }
    let successful_keys = graph.keys().copied().collect::<BTreeSet<_>>();
    let mut rejection_keys = BTreeSet::new();
    for rejection in &input.expected_rejections {
        if rejection.expected_code != "dataset.dependencies.cycle"
            || rejection.resource_key.trim().is_empty()
            || rejection.source_resource_keys.is_empty()
            || successful_keys.contains(rejection.resource_key.as_str())
            || !rejection_keys.insert(rejection.resource_key.as_str())
        {
            return Err(DatasetModuleError::ValidationFailed(
                "Dataset bootstrap expected rejection is invalid".into(),
            ));
        }
        graph.insert(
            rejection.resource_key.as_str(),
            rejection
                .source_resource_keys
                .iter()
                .map(String::as_str)
                .collect(),
        );
    }
    let all_keys = graph.keys().copied().collect::<BTreeSet<_>>();
    if graph
        .values()
        .flatten()
        .any(|source| !all_keys.contains(source))
    {
        return Err(DatasetModuleError::ValidationFailed(
            "Dataset bootstrap expected rejection references an unknown logical resource key"
                .into(),
        ));
    }
    for rejection in &input.expected_rejections {
        let participates_in_cycle = rejection.source_resource_keys.iter().any(|source| {
            logical_path_exists(
                &graph,
                source,
                &rejection.resource_key,
                &mut BTreeSet::new(),
            )
        });
        if !participates_in_cycle {
            return Err(DatasetModuleError::ValidationFailed(
                "Dataset bootstrap expected cycle rejection is not cyclic".into(),
            ));
        }
    }
    Ok(())
}

fn logical_path_exists<'a>(
    graph: &BTreeMap<&'a str, BTreeSet<&'a str>>,
    current: &str,
    target: &str,
    visited: &mut BTreeSet<&'a str>,
) -> bool {
    if current == target {
        return true;
    }
    let Some((canonical_current, sources)) = graph.get_key_value(current) else {
        return false;
    };
    if !visited.insert(*canonical_current) {
        return false;
    }
    sources
        .iter()
        .any(|source| logical_path_exists(graph, source, target, visited))
}

#[cfg(test)]
mod tests {
    use serde_json::json;
    use uuid::Uuid;

    use super::*;

    const INSTALLATION_ID: Uuid = Uuid::from_u128(1);
    const MODULE_INSTANCE_ID: Uuid = Uuid::from_u128(2);
    const DATASET_ID: Uuid = Uuid::from_u128(3);
    const REVISION_ID: Uuid = Uuid::from_u128(4);

    fn read_back() -> DatasetBootstrapReadBackV2 {
        DatasetBootstrapReadBackV2 {
            schema_version: 2,
            dataset: DatasetReference::from_parts(INSTALLATION_ID, MODULE_INSTANCE_ID, DATASET_ID)
                .unwrap(),
            revision: DatasetRevisionReference::from_parts(
                INSTALLATION_ID,
                MODULE_INSTANCE_ID,
                REVISION_ID,
            )
            .unwrap(),
            major_line: DatasetMajorLineReference::from_parts(
                INSTALLATION_ID,
                MODULE_INSTANCE_ID,
                DATASET_ID,
                1,
            )
            .unwrap(),
            materialized_row_count: 7,
        }
    }

    fn derived_definition() -> DatasetBootstrapDefinitionV1 {
        DatasetBootstrapDefinitionV1 {
            resource_key: "dataset.derived".into(),
            definition: json!({
                "name": "Derived",
                "slug": "derived",
                "grain": "submission",
                "visibility_node_ids": [Uuid::from_u128(5).to_string()],
                "initial_source": null,
                "operations": []
            }),
            reference_bindings: vec![DatasetBootstrapReferenceBindingV1 {
                target_pointer: "/initial_source".into(),
                source_resource_key: "dataset.base".into(),
                source_alias: "base".into(),
                selector: DatasetBootstrapReferenceSelectorV1::MajorLine { version_major: 1 },
            }],
        }
    }

    #[test]
    fn prior_logical_read_back_resolves_to_canonical_dataset_major_source() {
        let definition = derived_definition();
        let resolved = resolve_definition(
            &definition,
            &BTreeMap::from([("dataset.base".into(), read_back())]),
            INSTALLATION_ID,
            MODULE_INSTANCE_ID,
        )
        .unwrap();
        assert_eq!(
            resolved.initial_source,
            DatasetProductSourceV1::DatasetMajor {
                alias: "base".into(),
                dataset_id: DATASET_ID.to_string(),
                version_major: 1,
            }
        );
    }

    #[test]
    fn unresolved_or_substituted_logical_read_back_is_rejected() {
        let definition = derived_definition();
        assert!(
            resolve_definition(
                &definition,
                &BTreeMap::new(),
                INSTALLATION_ID,
                MODULE_INSTANCE_ID,
            )
            .is_err()
        );
        assert!(
            resolve_definition(
                &definition,
                &BTreeMap::from([("dataset.base".into(), read_back())]),
                INSTALLATION_ID,
                Uuid::from_u128(99),
            )
            .is_err()
        );
    }

    #[test]
    fn binding_targets_are_unique_canonical_json_pointers_to_null_values() {
        let mut definition = derived_definition();
        definition
            .reference_bindings
            .push(DatasetBootstrapReferenceBindingV1 {
                target_pointer: "/initial_source".into(),
                source_resource_key: "dataset.base".into(),
                source_alias: "base-copy".into(),
                selector: DatasetBootstrapReferenceSelectorV1::Revision,
            });
        assert!(
            validate_input(&DatasetBootstrapV1 {
                schema_version: DATASET_BOOTSTRAP_SCHEMA_VERSION.into(),
                datasets: vec![definition],
                expected_rejections: Vec::new(),
            })
            .is_err()
        );
    }

    #[test]
    fn expected_cycle_rejections_must_form_a_real_logical_cycle() {
        let valid = DatasetBootstrapV1 {
            schema_version: DATASET_BOOTSTRAP_SCHEMA_VERSION.into(),
            datasets: vec![DatasetBootstrapDefinitionV1 {
                resource_key: "dataset.base".into(),
                definition: json!({"initial_source": null}),
                reference_bindings: Vec::new(),
            }],
            expected_rejections: vec![
                DatasetBootstrapRejectionV1 {
                    resource_key: "dataset.cycle-a".into(),
                    source_resource_keys: vec!["dataset.cycle-b".into()],
                    expected_code: "dataset.dependencies.cycle".into(),
                },
                DatasetBootstrapRejectionV1 {
                    resource_key: "dataset.cycle-b".into(),
                    source_resource_keys: vec!["dataset.cycle-a".into()],
                    expected_code: "dataset.dependencies.cycle".into(),
                },
            ],
        };
        validate_input(&valid).unwrap();

        let mut acyclic = valid;
        acyclic.expected_rejections[1].source_resource_keys = vec!["dataset.base".into()];
        assert!(validate_input(&acyclic).is_err());
    }
}
