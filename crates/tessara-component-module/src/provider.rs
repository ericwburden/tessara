use std::collections::BTreeMap;

use axum::{Json, Router, body::Bytes, extract::State, http::HeaderMap, routing::post};
use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
use chrono::Utc;
use semver::Version;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use sqlx::Row;
use tessara_components_contract::{
    COMPONENT_CONTRACT_ID, COMPONENT_CONTRACT_SCHEMA_VERSION, COMPONENT_CONTRACT_VERSION,
    ComponentAction, ComponentCatalogResponse, ComponentChange, ComponentChangeCategory,
    ComponentLifecycleState, ComponentMetadata, ComponentPublicationState, ComponentRenderRequest,
    ComponentResolutionRequest, ComponentResolutionResponse, ComponentSuccessor,
    ComponentVersionReference,
};
use tessara_datasets_contract::{
    DATASET_CONTRACT_SCHEMA_VERSION, DatasetAction, DatasetAggregate, DatasetAggregateFunction,
    DatasetExecutionRequest, DatasetExecutionResponse, DatasetFilter, DatasetFilterOperator,
    DatasetMajorLineReference, DatasetMissingPolicy, DatasetSearch, DatasetSort,
    DatasetSortDirection,
};
use tessara_module_contract::{
    AuthorizationAudienceV1, AuthorizationGrantOperationV1, AuthorizationGrantV3,
    AuthorizationValidationContextV3, ContractCompatibilityState, FunctionalContractId,
    ModuleDefinitionId, ModuleInstanceOwnerState, ModuleServicePrincipalV1, ModuleServiceRequestV1,
    ModuleServiceRequestValidationContextV1, OwnerDataState, ProviderAvailabilityState,
    ProviderContractIdentity, ResourceAccessState, ResourceIdentityState, ResourceLifecycleState,
    ResourceObservationStrategy, ResourceObservationV1, ResourceOwner, ResourceOwnerState,
    ResourceResolutionV1, ResourceRevision, ResourceTypeId, SecurityCapabilityId, SignedEnvelopeV1,
};
use uuid::Uuid;

use crate::{
    ComponentModuleError, ComponentModuleState, READ_CAPABILITY, dataset_client,
    load_security_state,
};

pub(super) fn routes() -> Router<ComponentModuleState> {
    Router::new()
        .route("/api/private/components/resolve", post(resolve))
        .route("/api/private/components/catalog", post(catalog))
        .route("/api/private/components/render", post(render))
}

async fn resolve(
    State(state): State<ComponentModuleState>,
    headers: HeaderMap,
    Json(request): Json<ComponentResolutionRequest>,
) -> Result<Json<ComponentResolutionResponse>, ComponentModuleError> {
    let body = serde_json::to_vec(&request).map_err(internal)?;
    let component_grant = validate_provider_request(
        &state,
        &headers,
        "/api/private/components/resolve",
        &body,
        "components.resolve",
    )
    .await?;
    let reference = request.reference.reference();
    let security = load_security_state(&state.pool)
        .await?
        .ok_or_else(unavailable_security)?;
    if reference.installation_id() != security.installation_id
        || reference.owner()
            != &(ResourceOwner::ModuleInstance {
                installation_id: security.installation_id,
                module_instance_id: security.module_instance_id,
            })
    {
        return restricted(ResourceAccessState::NotEvaluated).map(Json);
    }
    let Ok(version_id) = Uuid::parse_str(reference.resource_id()) else {
        return restricted(ResourceAccessState::NotEvaluated).map(Json);
    };
    let Some(row) = load_version(&state, version_id).await? else {
        return undisclosed_reference().map(Json);
    };
    let scope: Vec<Uuid> = row.try_get("dataset_scope_node_ids")?;
    if !scope_authorized(&component_grant.payload, &scope) {
        return undisclosed_reference().map(Json);
    }
    resolved_response(
        &state,
        request,
        row,
        security.installation_id,
        security.module_instance_id,
    )
    .await
    .map(Json)
}

async fn catalog(
    State(state): State<ComponentModuleState>,
    headers: HeaderMap,
    body: Bytes,
) -> Result<Json<ComponentCatalogResponse>, ComponentModuleError> {
    if !body.is_empty() {
        return Err(ComponentModuleError::BadRequest(
            "Component catalog body must be empty".into(),
        ));
    }
    let component_grant = validate_provider_request(
        &state,
        &headers,
        "/api/private/components/catalog",
        &body,
        "components.catalog",
    )
    .await?;
    let security = load_security_state(&state.pool)
        .await?
        .ok_or_else(unavailable_security)?;
    let rows = sqlx::query(
        "SELECT v.id AS version_id,v.component_id,c.name AS component_name,c.slug AS component_slug,
                v.component_type::text AS component_type,v.status::text AS status,
                v.lifecycle_state::text AS lifecycle_state,v.resource_revision,v.authority_revision,
                v.version_number,v.version_label,v.successor_version_id,v.dataset_scope_node_ids
         FROM component_versions v JOIN components c ON c.id=v.component_id
         WHERE v.status IN ('published','superseded') AND v.lifecycle_state <> 'tombstoned'
         ORDER BY c.name,v.version_number DESC",
    )
    .fetch_all(&state.pool)
    .await?;
    let components = rows
        .into_iter()
        .filter_map(|row| {
            let scope = row.try_get::<Vec<Uuid>, _>("dataset_scope_node_ids").ok()?;
            scope_authorized(&component_grant.payload, &scope).then(|| {
                metadata(
                    &row,
                    security.installation_id,
                    security.module_instance_id,
                    scope,
                )
            })
        })
        .collect::<Result<Vec<_>, _>>()?;
    Ok(Json(ComponentCatalogResponse {
        schema_version: COMPONENT_CONTRACT_SCHEMA_VERSION,
        components,
    }))
}

async fn render(
    State(state): State<ComponentModuleState>,
    headers: HeaderMap,
    Json(request): Json<ComponentRenderRequest>,
) -> Result<Json<Value>, ComponentModuleError> {
    if request.action != ComponentAction::Render {
        return Err(ComponentModuleError::Forbidden);
    }
    let body = serde_json::to_vec(&request).map_err(internal)?;
    let component_grant = validate_provider_request(
        &state,
        &headers,
        "/api/private/components/render",
        &body,
        "components.render",
    )
    .await?;
    let security = load_security_state(&state.pool)
        .await?
        .ok_or_else(unavailable_security)?;
    let reference = request.reference.reference();
    if reference.installation_id() != security.installation_id
        || reference.owner()
            != &(ResourceOwner::ModuleInstance {
                installation_id: security.installation_id,
                module_instance_id: security.module_instance_id,
            })
    {
        return Err(ComponentModuleError::Forbidden);
    }
    let version_id =
        Uuid::parse_str(reference.resource_id()).map_err(|_| ComponentModuleError::Forbidden)?;
    let row = load_version(&state, version_id)
        .await?
        .ok_or(ComponentModuleError::Forbidden)?;
    let scope: Vec<Uuid> = row.try_get("dataset_scope_node_ids")?;
    if !scope_authorized(&component_grant.payload, &scope)
        || row.try_get::<i64, _>("authority_revision")? as u64
            != request.resource_authority_revision
        || row.try_get::<String, _>("lifecycle_state")? != "active"
    {
        return Err(ComponentModuleError::Forbidden);
    }
    let dataset_reference: DatasetMajorLineReference =
        serde_json::from_value(row.try_get("dataset_reference")?).map_err(internal)?;
    let authorization =
        URL_SAFE_NO_PAD.encode(serde_json::to_vec(&component_grant).map_err(internal)?);
    let component_type: String = row.try_get("component_type")?;
    let config: Value = row.try_get("config")?;
    let execution_request = execution_request(
        dataset_reference.clone(),
        &component_type,
        &config,
        &request.query,
    )?;
    let execution: DatasetExecutionResponse = dataset_client::post(
        &state,
        &authorization,
        "/api/private/datasets/execute",
        &execution_request,
    )
    .await?;
    Ok(Json(render_execution(
        execution,
        version_id,
        row.try_get("component_id")?,
        dataset_reference,
        &component_type,
        &config,
        execution_request.limit,
    )))
}

pub(super) fn execution_request(
    reference: DatasetMajorLineReference,
    component_type: &str,
    config: &Value,
    query: &str,
) -> Result<DatasetExecutionRequest, ComponentModuleError> {
    let parameters = query_parameters(query)?;
    let mut filters = config
        .get("filters")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .map(|filter| -> Result<DatasetFilter, ComponentModuleError> {
            let field_key = filter
                .get("field")
                .or_else(|| filter.get("field_key"))
                .and_then(Value::as_str)
                .ok_or_else(|| {
                    ComponentModuleError::BadRequest("Component filter field is invalid".into())
                })?;
            let operator = filter_operator(
                filter
                    .get("operator")
                    .and_then(Value::as_str)
                    .unwrap_or("equals"),
            )?;
            Ok(DatasetFilter {
                field_key: field_key.into(),
                operator,
                value: filter.get("value").cloned(),
            })
        })
        .collect::<Result<Vec<_>, _>>()?;
    filters.extend(query_filters(&parameters)?);
    let mut projection = Vec::new();
    let mut group_by = Vec::new();
    let mut group_missing_policies = BTreeMap::new();
    let mut aggregates = Vec::new();
    let mut order_by = Vec::new();
    let mut limit = query_limit(&parameters, config);
    let mut search = None;
    if component_type == "table" {
        let configured_projection: Vec<String> = config
            .get("visible_columns")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
            .filter_map(|item| {
                item.as_str()
                    .or_else(|| item.get("key").and_then(Value::as_str))
                    .or_else(|| item.get("field").and_then(Value::as_str))
                    .or_else(|| item.get("field_key").and_then(Value::as_str))
            })
            .map(str::to_string)
            .collect();
        projection = parameters
            .get("visible_columns")
            .map(|value| csv_keys(value))
            .filter(|values| !values.is_empty())
            .unwrap_or_else(|| configured_projection.clone());
        if projection
            .iter()
            .any(|field| !configured_projection.contains(field))
        {
            return Err(ComponentModuleError::BadRequest(
                "Runtime visible columns cannot expand the Component projection".into(),
            ));
        }
        if let Some(query) = parameters
            .get("search")
            .filter(|value| !value.trim().is_empty())
        {
            let configured_search = config
                .get("search_fields")
                .and_then(Value::as_array)
                .map(|values| {
                    values
                        .iter()
                        .filter_map(Value::as_str)
                        .map(str::to_string)
                        .collect::<Vec<_>>()
                })
                .filter(|values| !values.is_empty())
                .unwrap_or_else(|| projection.clone());
            search = Some(DatasetSearch {
                field_keys: configured_search,
                query: query.clone(),
            });
        }
        if let Some(sort) = parameters
            .get("sort")
            .map(|value| parse_sort(value))
            .transpose()?
            .or_else(|| config.get("default_sort").and_then(parse_stored_sort))
        {
            if !projection.contains(&sort.field_key) {
                return Err(ComponentModuleError::BadRequest(
                    "Component table sort field is outside the projection".into(),
                ));
            }
            order_by.push(sort);
        }
    } else {
        let summary_type = config
            .get("summary_type")
            .and_then(Value::as_str)
            .unwrap_or("count");
        let summary_field = config
            .get("summary_field")
            .and_then(Value::as_str)
            .filter(|value| !value.is_empty())
            .map(str::to_string);
        let function = match summary_type {
            "row_count" | "count" => DatasetAggregateFunction::Count,
            "unique_count" => DatasetAggregateFunction::UniqueCount,
            "sum" => DatasetAggregateFunction::Sum,
            "average" | "avg" => DatasetAggregateFunction::Average,
            "median" => DatasetAggregateFunction::Median,
            "none" => DatasetAggregateFunction::SingleValue,
            "minimum" | "min" => DatasetAggregateFunction::Minimum,
            "maximum" | "max" => DatasetAggregateFunction::Maximum,
            _ => {
                return Err(ComponentModuleError::BadRequest(
                    "Component summary type is invalid".into(),
                ));
            }
        };
        aggregates.push(DatasetAggregate {
            field_key: if summary_type == "row_count" {
                None
            } else {
                summary_field
            },
            function,
            output_key: "summary_value".into(),
            missing_policy: missing_policy(
                config
                    .get("value_missing_policy")
                    .or_else(|| config.get("missing_policy"))
                    .and_then(Value::as_str)
                    .unwrap_or("omit"),
            )?,
        });
        if component_type != "stat_card" {
            let category = if component_type == "line" {
                config.get("x_field")
            } else {
                config.get("category_field")
            }
            .and_then(Value::as_str)
            .filter(|v| !v.is_empty())
            .ok_or_else(|| {
                ComponentModuleError::BadRequest("Component category field is required".into())
            })?;
            group_by.push(category.into());
            group_missing_policies.insert(
                category.into(),
                missing_policy(
                    config
                        .get(if component_type == "line" {
                            "x_missing_policy"
                        } else {
                            "category_missing_policy"
                        })
                        .or_else(|| config.get("missing_policy"))
                        .and_then(Value::as_str)
                        .unwrap_or("omit"),
                )?,
            );
            if let Some(comparison) = config
                .get("comparison_field")
                .and_then(Value::as_str)
                .filter(|v| !v.is_empty())
            {
                group_by.push(comparison.into());
                group_missing_policies.insert(
                    comparison.into(),
                    missing_policy(
                        config
                            .get("comparison_missing_policy")
                            .or_else(|| config.get("missing_policy"))
                            .and_then(Value::as_str)
                            .unwrap_or("omit"),
                    )?,
                );
            }
        }
        limit = config
            .get("number_of_points")
            .or_else(|| config.get("max_slices"))
            .and_then(Value::as_u64)
            .and_then(|v| u32::try_from(v).ok())
            .unwrap_or(limit)
            .clamp(1, 1000);
        let sort_key = match config.get("sort_field").and_then(Value::as_str) {
            Some("summary_value") => "summary_value".into(),
            Some("x") | Some("category") => group_by
                .first()
                .cloned()
                .unwrap_or_else(|| "summary_value".into()),
            Some(value) if group_by.iter().any(|key| key == value) => value.into(),
            _ => group_by
                .first()
                .cloned()
                .unwrap_or_else(|| "summary_value".into()),
        };
        let direction = if config.get("sort_direction").and_then(Value::as_str) == Some("desc") {
            DatasetSortDirection::Desc
        } else {
            DatasetSortDirection::Asc
        };
        order_by.push(DatasetSort {
            field_key: sort_key,
            direction,
        });
    }
    Ok(DatasetExecutionRequest {
        schema_version: DATASET_CONTRACT_SCHEMA_VERSION,
        action: DatasetAction::Execute,
        reference,
        projection,
        filters,
        group_by,
        group_missing_policies,
        aggregates,
        order_by,
        limit,
        search,
        cursor: query_cursor(&parameters)?,
    })
}

fn missing_policy(value: &str) -> Result<DatasetMissingPolicy, ComponentModuleError> {
    match value {
        "omit" => Ok(DatasetMissingPolicy::Omit),
        "zero" => Ok(DatasetMissingPolicy::Zero),
        "explicit_missing" => Ok(DatasetMissingPolicy::ExplicitMissing),
        _ => Err(ComponentModuleError::BadRequest(
            "Component missing-value policy is invalid".into(),
        )),
    }
}

pub(super) fn render_execution(
    execution: DatasetExecutionResponse,
    version_id: Uuid,
    component_id: Uuid,
    dataset_reference: DatasetMajorLineReference,
    component_type: &str,
    config: &Value,
    limit: u32,
) -> Value {
    if component_type == "table" {
        let values=execution.rows.into_iter().map(|row|json!({"row_id":row.row_id,"values":row.values.into_iter().map(|(key,value)|(key,value.map(|value|text_value(&value)))).collect::<std::collections::BTreeMap<_,_>>() })).collect::<Vec<_>>();
        return json!({
        "schema_version": 1,
        "component_version_id": version_id,
        "component_id": component_id,
        "dataset_reference": dataset_reference,
        "component_type": component_type,
        "materialization_state": execution.materialization_state,
        "columns": execution.fields.into_iter().map(|field| {
            let label = config.get("display_labels").and_then(Value::as_object).and_then(|labels| labels.get(&field.key)).and_then(Value::as_str).unwrap_or(&field.label);
            json!({"key":field.key,"label":label,"field_type":field.field_type})
        }).collect::<Vec<_>>(),
        "rows": values,
        "pagination": {"page_size":limit,"next_cursor":execution.next_cursor,"has_more":execution.next_cursor.is_some()}
        });
    }
    let value_format = config
        .get("value_format")
        .and_then(Value::as_str)
        .unwrap_or("number");
    let category_labels = config.get("category_labels").and_then(Value::as_object);
    let category_colors = config.get("category_colors").and_then(Value::as_object);
    let rows = execution.rows;
    let numeric = |row: &tessara_datasets_contract::DatasetExecutionRow| {
        row.values
            .get("summary_value")
            .and_then(Option::as_ref)
            .and_then(number_value)
            .unwrap_or(0.0)
    };
    let category_key = if component_type == "line" {
        config.get("x_field")
    } else {
        config.get("category_field")
    }
    .and_then(Value::as_str);
    let comparison_key = config
        .get("comparison_field")
        .and_then(Value::as_str)
        .filter(|key| !key.is_empty());
    let has_bar_comparison = component_type == "bar" && comparison_key.is_some();
    let dimension = |row: &tessara_datasets_contract::DatasetExecutionRow, key: Option<&str>| {
        key.and_then(|key| row.values.get(key))
            .and_then(Option::as_ref)
            .map(text_value)
            .unwrap_or_default()
    };
    let label = |raw: String| {
        category_labels
            .and_then(|labels| labels.get(&raw))
            .and_then(Value::as_str)
            .unwrap_or(&raw)
            .to_string()
    };
    let color = |raw: &str| {
        category_colors
            .and_then(|colors| colors.get(raw))
            .and_then(Value::as_str)
            .map(str::to_string)
    };
    let stat=(component_type=="stat_card").then(||{let value=rows.first().map(&numeric);json!({"label":config.get("label").and_then(Value::as_str).unwrap_or("Value"),"value":value,"display_value":value.map(|v|format_value(v,value_format)),"supporting_text":config.get("supporting_text"),"panel_style":config.get("panel_style").and_then(Value::as_str).unwrap_or("default")})});
    let points = if matches!(component_type, "bar" | "line") {
        rows.iter().map(|row|{let raw=dimension(row,category_key);let comparison=dimension(row,comparison_key);let value=numeric(row);let x=if has_bar_comparison {raw.clone()} else {label(raw.clone())};let point_color=color(if has_bar_comparison {&comparison} else {&raw});let comparison_label=(!comparison.is_empty()).then(||if has_bar_comparison {label(comparison.clone())} else {comparison.clone()});json!({"x":x,"value":value,"display_value":format_value(value,value_format),"color":point_color,"comparison":comparison_label})}).collect::<Vec<_>>()
    } else {
        Vec::new()
    };
    let slices = if matches!(component_type, "pie" | "donut") {
        rows.iter().map(|row|{let raw=dimension(row,category_key);let value=numeric(row);json!({"category":label(raw.clone()),"value":value,"display_value":format_value(value,value_format),"color":color(&raw)})}).collect::<Vec<_>>()
    } else {
        Vec::new()
    };
    json!({"schema_version":1,"component_version_id":version_id,"component_id":component_id,"dataset_reference":dataset_reference,"component_type":component_type,"materialization_state":execution.materialization_state,"value_format":value_format,"legend_title":config.get("legend_title"),"bar_orientation":config.get("orientation"),"bar_comparison_layout":config.get("comparison_layout"),"x_axis_label":config.get("x_axis_label"),"y_axis_label":config.get("y_axis_label"),"line_smoothing":config.get("smoothing").and_then(Value::as_bool).unwrap_or(true),"stat":stat,"points":points,"slices":slices})
}

fn text_value(value: &Value) -> String {
    match value {
        Value::String(value) => value.clone(),
        other => other.to_string(),
    }
}
fn number_value(value: &Value) -> Option<f64> {
    value.as_f64().or_else(|| value.as_str()?.parse().ok())
}
fn format_value(value: f64, format: &str) -> String {
    match format {
        "integer" => format!("{value:.0}"),
        "percent" => format!("{:.1}%", value * 100.0),
        _ => {
            let value = format!("{value:.2}");
            value
                .trim_end_matches('0')
                .trim_end_matches('.')
                .to_string()
        }
    }
}

async fn validate_provider_request(
    state: &ComponentModuleState,
    headers: &HeaderMap,
    path: &str,
    body: &[u8],
    expected_action: &str,
) -> Result<SignedEnvelopeV1<AuthorizationGrantV3>, ComponentModuleError> {
    let authorization = authorization_header(headers)?;
    let grant: SignedEnvelopeV1<AuthorizationGrantV3> = decode(authorization)?;
    state
        .core_authorization_verifier
        .verify(&grant)
        .map_err(|_| ComponentModuleError::Forbidden)?;
    let security = load_security_state(&state.pool)
        .await?
        .ok_or_else(unavailable_security)?;
    let correlation_id = headers
        .get("x-tessara-correlation-id")
        .and_then(|value| value.to_str().ok())
        .and_then(|value| Uuid::parse_str(value).ok())
        .ok_or(ComponentModuleError::Forbidden)?;
    let presenting_service = match &grant.payload.presenting_service {
        ModuleServicePrincipalV1::ModuleInstance {
            module_instance_id,
            module_definition_id,
        } => ModuleServicePrincipalV1::ModuleInstance {
            module_instance_id: *module_instance_id,
            module_definition_id: module_definition_id.clone(),
        },
        ModuleServicePrincipalV1::CoreGateway => return Err(ComponentModuleError::Forbidden),
    };
    let audience = AuthorizationAudienceV1::ModuleInstance {
        module_instance_id: security.module_instance_id,
        module_definition_id: ModuleDefinitionId::new(crate::MODULE_DEFINITION_ID)
            .map_err(internal)?,
    };
    grant
        .payload
        .validate_for(&AuthorizationValidationContextV3 {
            installation_id: security.installation_id,
            correlation_id,
            presenting_service: presenting_service.clone(),
            audience,
            dependency_binding: grant.payload.dependency_binding.clone(),
            functional_contract: FunctionalContractId::new(COMPONENT_CONTRACT_ID)
                .map_err(internal)?,
            action: expected_action.into(),
            operation: AuthorizationGrantOperationV1::Read,
            resource_assertion: None,
            authorization_revision: security.authorization_revision as u64,
            organization_revision: security.organization_revision as u64,
            now: Utc::now(),
        })
        .map_err(|_| ComponentModuleError::Forbidden)?;
    let service_encoded = headers
        .get("x-tessara-module-service-request")
        .and_then(|value| value.to_str().ok())
        .ok_or(ComponentModuleError::Forbidden)?;
    let service: SignedEnvelopeV1<ModuleServiceRequestV1> = decode(service_encoded)?;
    let (caller_module_instance_id, caller_module_definition_id) = match presenting_service {
        ModuleServicePrincipalV1::ModuleInstance {
            module_instance_id,
            module_definition_id,
        } => (module_instance_id, module_definition_id),
        ModuleServicePrincipalV1::CoreGateway => unreachable!("checked above"),
    };
    state
        .service_identity_registry
        .module_service_verifier(&caller_module_definition_id)
        .map_err(|_| ComponentModuleError::Forbidden)?
        .verify(&service)
        .map_err(|_| ComponentModuleError::Forbidden)?;
    service
        .payload
        .validate_for(&ModuleServiceRequestValidationContextV1 {
            installation_id: security.installation_id,
            module_instance_id: caller_module_instance_id,
            module_definition_id: caller_module_definition_id,
            method: "POST".into(),
            path: path.into(),
            canonical_body_digest: sha256_hex(body),
            inbound_grant_digest: sha256_hex(authorization.as_bytes()),
            correlation_id: grant.payload.correlation_id.to_string(),
            now: Utc::now(),
        })
        .map_err(|_| ComponentModuleError::Forbidden)?;
    let consumed = sqlx::query("INSERT INTO component_consumed_service_nonces(module_instance_id,nonce,authorization_jti,correlation_id,issued_at) VALUES($1,$2,$3,$4,$5) ON CONFLICT DO NOTHING")
        .bind(service.payload.module_instance_id).bind(service.payload.nonce).bind(grant.payload.jti).bind(&service.payload.correlation_id).bind(service.payload.issued_at).execute(&state.pool).await?;
    if consumed.rows_affected() != 1 {
        return Err(ComponentModuleError::Forbidden);
    }
    Ok(grant)
}

async fn load_version(
    state: &ComponentModuleState,
    version_id: Uuid,
) -> Result<Option<sqlx::postgres::PgRow>, ComponentModuleError> {
    sqlx::query("SELECT v.id AS version_id,v.component_id,c.name AS component_name,c.slug AS component_slug,v.dataset_reference,v.dataset_scope_node_ids,v.component_type::text AS component_type,v.status::text AS status,v.lifecycle_state::text AS lifecycle_state,v.resource_revision,v.authority_revision,v.version_number,v.version_label,v.successor_version_id,v.config FROM component_versions v JOIN components c ON c.id=v.component_id WHERE v.id=$1 AND v.status IN ('published','superseded')")
        .bind(version_id).fetch_optional(&state.pool).await.map_err(Into::into)
}

async fn resolved_response(
    state: &ComponentModuleState,
    request: ComponentResolutionRequest,
    row: sqlx::postgres::PgRow,
    installation_id: Uuid,
    module_instance_id: Uuid,
) -> Result<ComponentResolutionResponse, ComponentModuleError> {
    let scope: Vec<Uuid> = row.try_get("dataset_scope_node_ids")?;
    let metadata = metadata(&row, installation_id, module_instance_id, scope)?;
    let revision = ResourceRevision::new(row.try_get::<i64, _>("resource_revision")? as u64)
        .map_err(internal)?;
    let observation = ResourceObservationV1::new(
        metadata.reference.reference().clone(),
        ProviderContractIdentity::new(
            FunctionalContractId::new(COMPONENT_CONTRACT_ID).map_err(internal)?,
            Version::parse(COMPONENT_CONTRACT_VERSION).map_err(internal)?,
        ),
        ResourceObservationStrategy::LiveResolutionWithRevision,
        revision,
    );
    let lifecycle = metadata.lifecycle_state;
    let changes = changes_since(
        &state.pool,
        metadata.component_version_id,
        request.changes_since_revision,
        revision,
    )
    .await?;
    let successor = row
        .try_get::<Option<Uuid>, _>("successor_version_id")?
        .map(|id| {
            component_reference(installation_id, module_instance_id, id)
                .map(|reference| ComponentSuccessor { reference })
        })
        .transpose()?;
    let tombstoned = lifecycle == ComponentLifecycleState::Tombstoned;
    ComponentResolutionResponse::new(
        ResourceResolutionV1::authorized(
            ResourceOwnerState::ModuleInstance {
                instance_state: ModuleInstanceOwnerState::Live,
                data_state: OwnerDataState::Retained,
            },
            ResourceIdentityState::Resolved,
            ResourceLifecycleState::ProviderDefined {
                state: lifecycle_name(lifecycle).into(),
            },
            ContractCompatibilityState::Compatible,
            ProviderAvailabilityState::Available,
        )
        .map_err(internal)?,
        Some(observation),
        (!tombstoned).then_some(metadata),
        changes,
        (!tombstoned).then_some(successor).flatten(),
    )
    .map_err(internal)
}

fn metadata(
    row: &sqlx::postgres::PgRow,
    installation_id: Uuid,
    module_instance_id: Uuid,
    mut scope_node_ids: Vec<Uuid>,
) -> Result<ComponentMetadata, ComponentModuleError> {
    scope_node_ids.sort_unstable();
    scope_node_ids.dedup();
    let version_id = row.try_get("version_id")?;
    Ok(ComponentMetadata {
        reference: component_reference(installation_id, module_instance_id, version_id)?,
        component_version_id: version_id,
        component_id: row.try_get("component_id")?,
        component_name: row.try_get("component_name")?,
        component_slug: row.try_get("component_slug")?,
        component_type: row.try_get("component_type")?,
        version_number: row.try_get("version_number")?,
        version_label: row.try_get("version_label")?,
        publication_state: publication(&row.try_get::<String, _>("status")?)?,
        lifecycle_state: lifecycle(&row.try_get::<String, _>("lifecycle_state")?)?,
        authority_revision: row.try_get::<i64, _>("authority_revision")? as u64,
        scope_node_ids,
    })
}

fn component_reference(
    installation_id: Uuid,
    module_instance_id: Uuid,
    version_id: Uuid,
) -> Result<ComponentVersionReference, ComponentModuleError> {
    ComponentVersionReference::new(
        tessara_module_contract::TypedResourceReference::new(
            installation_id,
            ResourceOwner::ModuleInstance {
                installation_id,
                module_instance_id,
            },
            ResourceTypeId::new(tessara_components_contract::COMPONENT_RESOURCE_TYPE)
                .map_err(internal)?,
            version_id.to_string(),
        )
        .map_err(internal)?,
    )
    .map_err(internal)
}
fn publication(value: &str) -> Result<ComponentPublicationState, ComponentModuleError> {
    match value {
        "draft" => Ok(ComponentPublicationState::Draft),
        "published" => Ok(ComponentPublicationState::Published),
        "superseded" => Ok(ComponentPublicationState::Superseded),
        _ => Err(ComponentModuleError::Internal(
            "Stored Component publication state is invalid".into(),
        )),
    }
}
fn lifecycle(value: &str) -> Result<ComponentLifecycleState, ComponentModuleError> {
    match value {
        "active" => Ok(ComponentLifecycleState::Active),
        "inactive" => Ok(ComponentLifecycleState::Inactive),
        "archived" => Ok(ComponentLifecycleState::Archived),
        "tombstoned" => Ok(ComponentLifecycleState::Tombstoned),
        _ => Err(ComponentModuleError::Internal(
            "Stored Component lifecycle state is invalid".into(),
        )),
    }
}
fn lifecycle_name(value: ComponentLifecycleState) -> &'static str {
    match value {
        ComponentLifecycleState::Active => "active",
        ComponentLifecycleState::Inactive => "inactive",
        ComponentLifecycleState::Archived => "archived",
        ComponentLifecycleState::Tombstoned => "tombstoned",
    }
}

async fn changes_since(
    pool: &sqlx::PgPool,
    version_id: Uuid,
    prior: Option<ResourceRevision>,
    current: ResourceRevision,
) -> Result<Vec<ComponentChange>, ComponentModuleError> {
    let Some(prior) = prior else {
        return Ok(Vec::new());
    };
    if prior > current {
        return Err(ComponentModuleError::BadRequest(
            "changes_since_revision exceeds the current revision".into(),
        ));
    }
    let rows=sqlx::query("SELECT resource_revision,array_agg(category::text ORDER BY category::text) categories FROM component_version_change_events WHERE component_version_id=$1 AND resource_revision>$2 AND resource_revision<=$3 GROUP BY resource_revision ORDER BY resource_revision").bind(version_id).bind(prior.get() as i64).bind(current.get() as i64).fetch_all(pool).await?;
    rows.into_iter()
        .map(|row| {
            let resource_revision =
                ResourceRevision::new(row.try_get::<i64, _>("resource_revision")? as u64)
                    .map_err(internal)?;
            let categories = row
                .try_get::<Vec<String>, _>("categories")?
                .into_iter()
                .map(|value| match value.as_str() {
                    "publication" => Ok(ComponentChangeCategory::Publication),
                    "lifecycle" => Ok(ComponentChangeCategory::Lifecycle),
                    "payload" => Ok(ComponentChangeCategory::Payload),
                    "successor" => Ok(ComponentChangeCategory::Successor),
                    _ => Err(ComponentModuleError::Internal(
                        "Stored Component change category is invalid".into(),
                    )),
                })
                .collect::<Result<Vec<_>, _>>()?;
            Ok(ComponentChange {
                resource_revision,
                categories,
            })
        })
        .collect()
}

fn restricted(
    access: ResourceAccessState,
) -> Result<ComponentResolutionResponse, ComponentModuleError> {
    ComponentResolutionResponse::new(
        tessara_module_contract::ResourceResolutionV1::restricted(access).map_err(internal)?,
        None,
        None,
        Vec::new(),
        None,
    )
    .map_err(internal)
}

fn undisclosed_reference() -> Result<ComponentResolutionResponse, ComponentModuleError> {
    restricted(ResourceAccessState::Unauthorized)
}
fn scope_authorized(grant: &AuthorizationGrantV3, scope: &[Uuid]) -> bool {
    SecurityCapabilityId::new(READ_CAPABILITY)
        .ok()
        .is_some_and(|capability| scope.iter().any(|id| grant.authorizes(&capability, *id)))
}
fn authorization_header(headers: &HeaderMap) -> Result<&str, ComponentModuleError> {
    headers
        .get("x-tessara-authorization")
        .and_then(|value| value.to_str().ok())
        .ok_or(ComponentModuleError::Forbidden)
}
fn decode<T: serde::de::DeserializeOwned>(value: &str) -> Result<T, ComponentModuleError> {
    let bytes = URL_SAFE_NO_PAD
        .decode(value)
        .map_err(|_| ComponentModuleError::Forbidden)?;
    serde_json::from_slice(&bytes).map_err(|_| ComponentModuleError::Forbidden)
}
fn sha256_hex(value: &[u8]) -> String {
    Sha256::digest(value)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}
fn query_limit(parameters: &BTreeMap<String, String>, config: &Value) -> u32 {
    parameters
        .get("page_size")
        .and_then(|value| value.parse::<u32>().ok())
        .or_else(|| {
            config
                .get("page_size")
                .and_then(Value::as_u64)
                .and_then(|value| u32::try_from(value).ok())
        })
        .unwrap_or(25)
        .clamp(1, 200)
}

fn query_cursor(
    parameters: &BTreeMap<String, String>,
) -> Result<Option<String>, ComponentModuleError> {
    parameters
        .get("cursor")
        .map(|value| {
            let offset = value
                .strip_prefix("offset:")
                .and_then(|value| value.parse::<u32>().ok())
                .filter(|value| *value > 0)
                .ok_or_else(|| {
                    ComponentModuleError::BadRequest("Component cursor is invalid".into())
                })?;
            Ok(format!("offset:{offset}"))
        })
        .transpose()
}

fn query_parameters(query: &str) -> Result<BTreeMap<String, String>, ComponentModuleError> {
    query
        .split('&')
        .filter(|part| !part.is_empty())
        .map(|part| {
            let (key, value) = part.split_once('=').unwrap_or((part, ""));
            Ok((percent_decode(key)?, percent_decode(value)?))
        })
        .collect()
}

fn percent_decode(value: &str) -> Result<String, ComponentModuleError> {
    let bytes = value.as_bytes();
    let mut decoded = Vec::with_capacity(bytes.len());
    let mut index = 0;
    while index < bytes.len() {
        match bytes[index] {
            b'+' => decoded.push(b' '),
            b'%' if index + 2 < bytes.len() => {
                let pair = std::str::from_utf8(&bytes[index + 1..index + 3]).map_err(|_| {
                    ComponentModuleError::BadRequest("Component query encoding is invalid".into())
                })?;
                decoded.push(u8::from_str_radix(pair, 16).map_err(|_| {
                    ComponentModuleError::BadRequest("Component query encoding is invalid".into())
                })?);
                index += 2;
            }
            b'%' => {
                return Err(ComponentModuleError::BadRequest(
                    "Component query encoding is invalid".into(),
                ));
            }
            byte => decoded.push(byte),
        }
        index += 1;
    }
    String::from_utf8(decoded)
        .map_err(|_| ComponentModuleError::BadRequest("Component query encoding is invalid".into()))
}

fn query_filters(
    parameters: &BTreeMap<String, String>,
) -> Result<Vec<DatasetFilter>, ComponentModuleError> {
    let mut filters = BTreeMap::<String, (Option<String>, Option<String>)>::new();
    for (key, value) in parameters {
        let Some(remainder) = key.strip_prefix("filter[") else {
            continue;
        };
        let Some((field, suffix)) = remainder.split_once("][") else {
            continue;
        };
        let Some(kind) = suffix.strip_suffix(']') else {
            continue;
        };
        let entry = filters.entry(field.into()).or_default();
        match kind {
            "operator" => entry.0 = Some(value.clone()),
            "value" => entry.1 = Some(value.clone()),
            _ => {}
        }
    }
    filters
        .into_iter()
        .map(|(field_key, (operator, value))| {
            let operator = operator.ok_or_else(|| {
                ComponentModuleError::BadRequest(format!(
                    "Component table filter for '{field_key}' is missing an operator"
                ))
            })?;
            Ok(DatasetFilter {
                field_key,
                operator: filter_operator(&operator)?,
                value: value.map(Value::String),
            })
        })
        .collect()
}

fn filter_operator(value: &str) -> Result<DatasetFilterOperator, ComponentModuleError> {
    match value {
        "eq" | "equals" => Ok(DatasetFilterOperator::Eq),
        "not_eq" | "not_equals" => Ok(DatasetFilterOperator::NotEq),
        "contains" => Ok(DatasetFilterOperator::Contains),
        "not_contains" => Ok(DatasetFilterOperator::NotContains),
        "starts_with" => Ok(DatasetFilterOperator::StartsWith),
        "ends_with" => Ok(DatasetFilterOperator::EndsWith),
        "gt" | "greater_than" => Ok(DatasetFilterOperator::GreaterThan),
        "gte" | "greater_than_or_equal" => Ok(DatasetFilterOperator::GreaterThanOrEqual),
        "lt" | "less_than" => Ok(DatasetFilterOperator::LessThan),
        "lte" | "less_than_or_equal" => Ok(DatasetFilterOperator::LessThanOrEqual),
        "between" => Ok(DatasetFilterOperator::Between),
        "not_between" => Ok(DatasetFilterOperator::NotBetween),
        "is_empty" => Ok(DatasetFilterOperator::IsEmpty),
        "is_not_empty" => Ok(DatasetFilterOperator::IsNotEmpty),
        "is_null" => Ok(DatasetFilterOperator::IsNull),
        "is_not_null" => Ok(DatasetFilterOperator::IsNotNull),
        _ => Err(ComponentModuleError::BadRequest(
            "Component filter operator is invalid".into(),
        )),
    }
}

fn parse_sort(value: &str) -> Result<DatasetSort, ComponentModuleError> {
    let (field_key, direction) = value.split_once(':').unwrap_or((value, "asc"));
    if field_key.trim().is_empty() {
        return Err(ComponentModuleError::BadRequest(
            "Component table sort field is required".into(),
        ));
    }
    let direction = match direction.trim().to_ascii_lowercase().as_str() {
        "" | "asc" => DatasetSortDirection::Asc,
        "desc" => DatasetSortDirection::Desc,
        _ => {
            return Err(ComponentModuleError::BadRequest(
                "Component table sort direction must be asc or desc".into(),
            ));
        }
    };
    Ok(DatasetSort {
        field_key: field_key.trim().into(),
        direction,
    })
}

fn parse_stored_sort(value: &Value) -> Option<DatasetSort> {
    let field_key = value.get("field_key")?.as_str()?.to_string();
    let direction = if value.get("direction").and_then(Value::as_str) == Some("desc") {
        DatasetSortDirection::Desc
    } else {
        DatasetSortDirection::Asc
    };
    Some(DatasetSort {
        field_key,
        direction,
    })
}

fn csv_keys(value: &str) -> Vec<String> {
    value
        .split(',')
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
        .collect()
}
fn unavailable_security() -> ComponentModuleError {
    ComponentModuleError::Unavailable("Component security state is unavailable".into())
}
fn internal(error: impl std::fmt::Display) -> ComponentModuleError {
    ComponentModuleError::Internal(error.to_string())
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use serde_json::json;
    use tessara_datasets_contract::{
        DatasetAggregateFunction, DatasetExecutionResponse, DatasetExecutionRow,
        DatasetFilterOperator, DatasetMajorLineReference, DatasetMissingPolicy,
        DatasetSortDirection,
    };
    use uuid::Uuid;

    use axum::{Json, body::to_bytes, response::IntoResponse};
    use tessara_module_contract::{
        ContractCompatibilityState, ProviderAvailabilityState, ResourceAccessState,
        ResourceIdentityState, ResourceOwnerState,
    };

    use super::{execution_request, render_execution, undisclosed_reference};

    fn dataset_reference() -> DatasetMajorLineReference {
        DatasetMajorLineReference::from_parts(Uuid::from_u128(1), Uuid::from_u128(2), 1)
            .expect("canonical Dataset reference")
    }

    #[tokio::test]
    async fn missing_and_unauthorized_references_share_the_exact_restricted_response() {
        let missing =
            Json(undisclosed_reference().expect("missing reference projection")).into_response();
        let unauthorized =
            Json(undisclosed_reference().expect("unauthorized reference projection"))
                .into_response();

        assert_eq!(missing.status(), unauthorized.status());
        assert_eq!(missing.status(), axum::http::StatusCode::OK);
        assert_eq!(
            missing.headers().get(axum::http::header::CONTENT_TYPE),
            unauthorized.headers().get(axum::http::header::CONTENT_TYPE)
        );

        let missing_body = to_bytes(missing.into_body(), usize::MAX)
            .await
            .expect("missing response body");
        let unauthorized_body = to_bytes(unauthorized.into_body(), usize::MAX)
            .await
            .expect("unauthorized response body");
        assert_eq!(missing_body, unauthorized_body);

        let body: serde_json::Value =
            serde_json::from_slice(&missing_body).expect("restricted response JSON");
        assert_eq!(body["resolution"]["access_state"], "unauthorized");
        assert_eq!(body["resolution"]["owner_state"]["kind"], "undisclosed");
        assert_eq!(body["resolution"]["resource_identity_state"], "undisclosed");
        assert_eq!(
            body["resolution"]["resource_lifecycle_state"]["kind"],
            "undisclosed"
        );
        assert_eq!(body["resolution"]["compatibility_state"], "undisclosed");
        assert_eq!(body["resolution"]["availability_state"], "undisclosed");
        assert_eq!(
            undisclosed_reference()
                .expect("restricted response")
                .resolution()
                .access_state(),
            ResourceAccessState::Unauthorized
        );
        assert_eq!(
            undisclosed_reference()
                .expect("restricted response")
                .resolution()
                .owner_state(),
            ResourceOwnerState::Undisclosed
        );
        assert_eq!(
            undisclosed_reference()
                .expect("restricted response")
                .resolution()
                .resource_identity_state(),
            ResourceIdentityState::Undisclosed
        );
        assert_eq!(
            undisclosed_reference()
                .expect("restricted response")
                .resolution()
                .compatibility_state(),
            ContractCompatibilityState::Undisclosed
        );
        assert_eq!(
            undisclosed_reference()
                .expect("restricted response")
                .resolution()
                .availability_state(),
            ProviderAvailabilityState::Undisclosed
        );
    }

    #[test]
    fn visual_execution_preserves_declared_category_and_comparison_dimensions() {
        let config = json!({
            "summary_type": "count",
            "summary_field": "participant_id",
            "category_field": "status",
            "comparison_field": "program",
            "category_labels": {
                "Completed": "Completed work",
                "North": "North region"
            },
            "category_colors": {
                "Completed": "#111111",
                "North": "#3568d4"
            },
            "number_of_points": 20,
            "value_format": "integer"
        });
        let request =
            execution_request(dataset_reference(), "bar", &config, "").expect("execution request");
        assert_eq!(request.group_by, ["status", "program"]);

        let response = DatasetExecutionResponse {
            schema_version: 1,
            materialization_state: "ready".into(),
            fields: Vec::new(),
            rows: vec![DatasetExecutionRow {
                row_id: "row-1".into(),
                values: BTreeMap::from([
                    ("program".into(), Some(json!("North"))),
                    ("status".into(), Some(json!("Completed"))),
                    ("summary_value".into(), Some(json!(7))),
                ]),
            }],
            next_cursor: None,
        };
        let rendered = render_execution(
            response,
            Uuid::from_u128(3),
            Uuid::from_u128(4),
            dataset_reference(),
            "bar",
            &config,
            request.limit,
        );
        assert_eq!(rendered["points"][0]["x"], "Completed");
        assert_eq!(rendered["points"][0]["comparison"], "North region");
        assert_eq!(rendered["points"][0]["color"], "#3568d4");
        assert_eq!(rendered["points"][0]["display_value"], "7");
    }

    #[test]
    fn visual_execution_preserves_category_labels_and_colors_without_comparison() {
        let config = json!({
            "summary_type": "count",
            "summary_field": "participant_id",
            "category_field": "status",
            "category_labels": {"Completed": "Completed work"},
            "category_colors": {"Completed": "#3568d4"},
            "number_of_points": 20,
            "value_format": "integer"
        });
        let request =
            execution_request(dataset_reference(), "bar", &config, "").expect("execution request");
        assert_eq!(request.group_by, ["status"]);

        let response = DatasetExecutionResponse {
            schema_version: 1,
            materialization_state: "ready".into(),
            fields: Vec::new(),
            rows: vec![DatasetExecutionRow {
                row_id: "row-1".into(),
                values: BTreeMap::from([
                    ("status".into(), Some(json!("Completed"))),
                    ("summary_value".into(), Some(json!(7))),
                ]),
            }],
            next_cursor: None,
        };
        let rendered = render_execution(
            response,
            Uuid::from_u128(3),
            Uuid::from_u128(4),
            dataset_reference(),
            "bar",
            &config,
            request.limit,
        );
        assert_eq!(rendered["points"][0]["x"], "Completed work");
        assert!(rendered["points"][0]["comparison"].is_null());
        assert_eq!(rendered["points"][0]["color"], "#3568d4");
        assert_eq!(rendered["points"][0]["display_value"], "7");
    }

    #[test]
    fn line_execution_emits_the_canonical_smoothing_default() {
        let config = json!({
            "summary_type": "count",
            "summary_field": "participant_id",
            "x_field": "status",
            "number_of_points": 20
        });
        let response = DatasetExecutionResponse {
            schema_version: 1,
            materialization_state: "ready".into(),
            fields: Vec::new(),
            rows: Vec::new(),
            next_cursor: None,
        };
        let rendered = render_execution(
            response,
            Uuid::from_u128(3),
            Uuid::from_u128(4),
            dataset_reference(),
            "line",
            &config,
            20,
        );
        assert_eq!(rendered["line_smoothing"], true);
    }

    #[test]
    fn table_execution_projects_only_configured_columns() {
        let config = json!({"visible_columns":["participant", {"key":"status"}]});
        let request = execution_request(dataset_reference(), "table", &config, "page_size=250")
            .expect("table request");
        assert_eq!(request.projection, ["participant", "status"]);
        assert_eq!(request.limit, 200);
        assert!(request.group_by.is_empty());
        assert!(request.aggregates.is_empty());
    }

    #[test]
    fn table_execution_preserves_runtime_search_sort_filter_and_narrowing_semantics() {
        let config = json!({
            "visible_columns":["participant", "status", "score"],
            "search_fields":["participant", "status"],
            "default_sort":{"field_key":"participant", "direction":"asc"},
            "page_size":50
        });
        let request = execution_request(
            dataset_reference(),
            "table",
            &config,
            "visible_columns=participant%2Cscore&search=North+Team&sort=score%3Adesc&filter%5Bscore%5D%5Boperator%5D=between&filter%5Bscore%5D%5Bvalue%5D=10..20",
        )
        .expect("table controls");
        assert_eq!(request.projection, ["participant", "score"]);
        let search = request.search.expect("search contract");
        assert_eq!(search.field_keys, ["participant", "status"]);
        assert_eq!(search.query, "North Team");
        assert_eq!(request.order_by.len(), 1);
        assert_eq!(request.order_by[0].field_key, "score");
        assert_eq!(request.order_by[0].direction, DatasetSortDirection::Desc);
        assert_eq!(request.filters.len(), 1);
        assert_eq!(request.filters[0].field_key, "score");
        assert_eq!(request.filters[0].operator, DatasetFilterOperator::Between);
        assert_eq!(request.filters[0].value, Some(json!("10..20")));
        assert_eq!(request.limit, 50);

        assert!(
            execution_request(
                dataset_reference(),
                "table",
                &config,
                "visible_columns=undeclared"
            )
            .is_err()
        );
    }

    #[test]
    fn visual_execution_uses_each_canonical_aggregate_identity() {
        for (summary_type, expected) in [
            ("unique_count", DatasetAggregateFunction::UniqueCount),
            ("median", DatasetAggregateFunction::Median),
            ("none", DatasetAggregateFunction::SingleValue),
        ] {
            let config = json!({
                "summary_type":summary_type,
                "summary_field":"score",
                "label":"Score"
            });
            let request = execution_request(dataset_reference(), "stat_card", &config, "")
                .expect("aggregate request");
            assert_eq!(request.aggregates.len(), 1);
            assert_eq!(request.aggregates[0].function, expected);
            assert_eq!(request.aggregates[0].field_key.as_deref(), Some("score"));
        }
    }

    #[test]
    fn visual_execution_preserves_value_and_dimension_missing_policies() {
        let config = json!({
            "mode":"comparison",
            "summary_type":"sum",
            "summary_field":"score",
            "missing_policy":"omit",
            "value_missing_policy":"zero",
            "category_field":"status",
            "category_missing_policy":"explicit_missing",
            "comparison_field":"program",
            "comparison_missing_policy":"omit",
            "comparison_layout":"stacked"
        });
        let request =
            execution_request(dataset_reference(), "bar", &config, "").expect("missing policies");
        assert_eq!(
            request.aggregates[0].missing_policy,
            DatasetMissingPolicy::Zero
        );
        assert_eq!(
            request.group_missing_policies["status"],
            DatasetMissingPolicy::ExplicitMissing
        );
        assert_eq!(
            request.group_missing_policies["program"],
            DatasetMissingPolicy::Omit
        );
    }

    #[test]
    fn table_execution_preserves_the_exact_server_cursor() {
        let config = json!({"visible_columns":["participant"]});
        let request = execution_request(
            dataset_reference(),
            "table",
            &config,
            "page_size=25&cursor=offset%3A25",
        )
        .expect("table cursor request");
        assert_eq!(request.cursor.as_deref(), Some("offset:25"));
        assert!(
            execution_request(dataset_reference(), "table", &config, "cursor=unexpected",).is_err()
        );
    }
}
