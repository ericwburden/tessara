use axum::{
    Json, Router,
    extract::{Path, State},
    http::{HeaderMap, HeaderValue, header},
    response::{IntoResponse, Response},
    routing::get,
};
use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
use chrono::Utc;
use semver::Version;
use tessara_dataset_ui::{
    DATASET_JS_SHA256, DATASET_LIFECYCLE_CSS_SHA256, DatasetEditorBootstrap, DatasetRouteBootstrap,
    render_dataset_document,
};
use tessara_datasets_contract::{DatasetProductOperationV1, DatasetProductSourceV1};
use tessara_module_contract::{
    ArtifactDigest, AuthorizationAudienceV1, AuthorizationGrantOperationV1, AuthorizationGrantV3,
    AuthorizationValidationContextV3, BrowserLifecycleAssetV1, BrowserLifecycleBootstrapV1,
    DependencyBindingKey, FunctionalContractId, ModuleDefinitionId, ModuleServicePrincipalV1,
    SecurityCapabilityId, SemanticRouteName, SignedEnvelopeV1,
};
use uuid::Uuid;

use crate::{
    DatasetModuleError, DatasetModuleState, MODULE_DEFINITION_ID, MODULE_RELEASE_VERSION,
    load_security_state, product, verified_shell_context,
};

const CORE_DATASET_BINDING: &str = "tessara.core.datasets";
const DATASET_RESOURCE_CONTRACT: &str = "tessara.datasets.dataset-major-line";
const DATASET_AUTHORING_CONTRACT: &str = "tessara.datasets.authoring";

pub(super) fn routes() -> Router<DatasetModuleState> {
    Router::new()
        .route("/datasets", get(directory))
        .route("/datasets/new", get(create))
        .route("/datasets/{dataset_id}/edit", get(edit))
        .route("/datasets/{dataset_id}/preview", get(preview))
        .route("/datasets/{dataset_id}/revisions", get(revisions))
        .route(
            "/datasets/{dataset_id}/revisions/{revision_id}/edit",
            get(revision_edit),
        )
        .route(
            "/datasets/{dataset_id}/revisions/{revision_id}",
            get(revision_detail),
        )
        .route("/datasets/{dataset_id}", get(detail))
}

async fn directory(
    State(state): State<DatasetModuleState>,
    headers: HeaderMap,
) -> Result<Response, DatasetModuleError> {
    let grant =
        authorize_browser(&state, &headers, "datasets.list", DATASET_RESOURCE_CONTRACT).await?;
    let datasets = product::list_dataset_summaries(&state, &grant.payload).await?;
    document(
        &state,
        &headers,
        "/datasets",
        "Datasets",
        DatasetRouteBootstrap::Directory {
            datasets,
            can_manage: has_capability(&grant.payload, "datasets:manage"),
        },
    )
    .await
}

async fn create(
    State(state): State<DatasetModuleState>,
    headers: HeaderMap,
) -> Result<Response, DatasetModuleError> {
    let grant = authorize_browser(
        &state,
        &headers,
        "datasets.create",
        DATASET_AUTHORING_CONTRACT,
    )
    .await?;
    let editor = editor_bootstrap(&state, &grant, None, None).await?;
    document(
        &state,
        &headers,
        "/datasets/new",
        "Create Dataset",
        DatasetRouteBootstrap::Create {
            editor,
            can_manage: has_capability(&grant.payload, "datasets:manage"),
        },
    )
    .await
}

async fn detail(
    State(state): State<DatasetModuleState>,
    headers: HeaderMap,
    Path(dataset_id): Path<String>,
) -> Result<Response, DatasetModuleError> {
    dataset_document(state, headers, dataset_id, None, "detail").await
}

async fn preview(
    State(state): State<DatasetModuleState>,
    headers: HeaderMap,
    Path(dataset_id): Path<String>,
) -> Result<Response, DatasetModuleError> {
    dataset_document(state, headers, dataset_id, None, "preview").await
}

async fn edit(
    State(state): State<DatasetModuleState>,
    headers: HeaderMap,
    Path(dataset_id): Path<String>,
) -> Result<Response, DatasetModuleError> {
    dataset_document(state, headers, dataset_id, None, "edit").await
}

async fn revisions(
    State(state): State<DatasetModuleState>,
    headers: HeaderMap,
    Path(dataset_id): Path<String>,
) -> Result<Response, DatasetModuleError> {
    dataset_document(state, headers, dataset_id, None, "revisions").await
}

async fn revision_detail(
    State(state): State<DatasetModuleState>,
    headers: HeaderMap,
    Path((dataset_id, revision_id)): Path<(String, String)>,
) -> Result<Response, DatasetModuleError> {
    dataset_document(
        state,
        headers,
        dataset_id,
        Some(revision_id),
        "revision_detail",
    )
    .await
}

async fn revision_edit(
    State(state): State<DatasetModuleState>,
    headers: HeaderMap,
    Path((dataset_id, revision_id)): Path<(String, String)>,
) -> Result<Response, DatasetModuleError> {
    dataset_document(
        state,
        headers,
        dataset_id,
        Some(revision_id),
        "revision_edit",
    )
    .await
}

async fn dataset_document(
    state: DatasetModuleState,
    headers: HeaderMap,
    dataset_id: String,
    revision_id: Option<String>,
    route: &str,
) -> Result<Response, DatasetModuleError> {
    let (action, contract) = match route {
        "edit" | "revision_edit" => ("datasets.get_revision", DATASET_AUTHORING_CONTRACT),
        "preview" => ("datasets.preview_table", DATASET_RESOURCE_CONTRACT),
        "revisions" => ("datasets.list_revisions", DATASET_RESOURCE_CONTRACT),
        "revision_detail" => ("datasets.get_revision", DATASET_RESOURCE_CONTRACT),
        _ => ("datasets.get", DATASET_RESOURCE_CONTRACT),
    };
    let grant = authorize_browser(&state, &headers, action, contract).await?;
    let can_manage = has_capability(&grant.payload, "datasets:manage");
    let dataset_uuid = parse_product_id(&dataset_id)?;
    let (path, title, bootstrap) = match (route, revision_id) {
        ("preview", _) => {
            let dataset = product::dataset_definition(&state, &grant.payload, dataset_uuid).await?;
            let (table, table_error) = table_projection(&state, &grant.payload, dataset_uuid).await;
            (
                format!("/datasets/{dataset_id}/preview"),
                "Dataset Preview",
                DatasetRouteBootstrap::Preview {
                    dataset_id,
                    dataset,
                    table,
                    table_error,
                    can_manage,
                },
            )
        }
        ("edit", _) => {
            let dataset = product::dataset_definition(&state, &grant.payload, dataset_uuid).await?;
            let editor = editor_bootstrap(&state, &grant, Some(dataset), None).await?;
            (
                format!("/datasets/{dataset_id}/edit"),
                "Edit Dataset",
                DatasetRouteBootstrap::Edit {
                    dataset_id,
                    editor,
                    can_manage,
                },
            )
        }
        ("revisions", _) => {
            let dataset = product::dataset_definition(&state, &grant.payload, dataset_uuid).await?;
            let revisions =
                product::dataset_revision_summaries(&state, &grant.payload, dataset_uuid).await?;
            (
                format!("/datasets/{dataset_id}/revisions"),
                "Dataset Revisions",
                DatasetRouteBootstrap::Revisions {
                    dataset_id,
                    dataset,
                    revisions,
                    can_manage,
                },
            )
        }
        ("revision_detail", Some(revision_id)) => {
            let revision_uuid = parse_product_id(&revision_id)?;
            let revision = match product::dataset_revision_detail(
                &state,
                &grant.payload,
                dataset_uuid,
                revision_uuid,
            )
            .await
            {
                Ok(revision) => revision,
                Err(DatasetModuleError::NotFound(_)) => {
                    return document(
                        &state,
                        &headers,
                        &format!("/datasets/{dataset_id}/revisions/{revision_id}"),
                        "Dataset Revision",
                        DatasetRouteBootstrap::RevisionUnavailable {
                            dataset_id,
                            revision_id,
                            message: "Dataset revision was not found.".into(),
                            can_manage,
                        },
                    )
                    .await;
                }
                Err(error) => return Err(error),
            };
            (
                format!("/datasets/{dataset_id}/revisions/{revision_id}"),
                "Dataset Revision",
                DatasetRouteBootstrap::RevisionDetail {
                    dataset_id,
                    revision_id,
                    revision,
                    can_manage,
                },
            )
        }
        ("revision_edit", Some(revision_id)) => {
            let revision_uuid = parse_product_id(&revision_id)?;
            let dataset = product::dataset_definition(&state, &grant.payload, dataset_uuid).await?;
            let revision = product::dataset_revision_detail(
                &state,
                &grant.payload,
                dataset_uuid,
                revision_uuid,
            )
            .await?;
            let editor = editor_bootstrap(&state, &grant, Some(dataset), Some(revision)).await?;
            (
                format!("/datasets/{dataset_id}/revisions/{revision_id}/edit"),
                "Edit Revision",
                DatasetRouteBootstrap::RevisionEdit {
                    dataset_id,
                    revision_id,
                    editor,
                    can_manage,
                },
            )
        }
        _ => {
            let dataset = product::dataset_definition(&state, &grant.payload, dataset_uuid).await?;
            let (table, table_error) = table_projection(&state, &grant.payload, dataset_uuid).await;
            (
                format!("/datasets/{dataset_id}"),
                "Dataset Detail",
                DatasetRouteBootstrap::Detail {
                    dataset_id,
                    dataset,
                    table,
                    table_error,
                    can_manage,
                },
            )
        }
    };
    document(&state, &headers, &path, title, bootstrap).await
}

async fn table_projection(
    state: &DatasetModuleState,
    grant: &AuthorizationGrantV3,
    dataset_id: Uuid,
) -> (
    Option<tessara_datasets_contract::DatasetProductTableV1>,
    Option<String>,
) {
    match product::dataset_table(state, grant, dataset_id).await {
        Ok(table) => (Some(table), None),
        Err(_) => (
            None,
            Some("Dataset preview is temporarily unavailable.".into()),
        ),
    }
}

async fn editor_bootstrap(
    state: &DatasetModuleState,
    grant: &SignedEnvelopeV1<AuthorizationGrantV3>,
    dataset: Option<tessara_datasets_contract::DatasetProductDefinitionV1>,
    revision: Option<tessara_datasets_contract::DatasetProductRevisionDetailV1>,
) -> Result<DatasetEditorBootstrap, DatasetModuleError> {
    let form_version_ids = referenced_form_version_ids(dataset.as_ref(), revision.as_ref());
    let (datasets, forms, nodes, principals) = tokio::join!(
        product::list_dataset_summaries(state, &grant.payload),
        product::editor_form_option_projection(state, grant),
        product::editor_scope_option_projection(state, grant),
        product::editor_principal_option_projection(state, grant),
    );
    let datasets = datasets?;
    let mut provider_unavailable = false;
    let forms = forms.map(|(value, _)| value).unwrap_or_else(|_| {
        provider_unavailable = true;
        Vec::new()
    });
    let nodes = nodes.map(|(value, _)| value).unwrap_or_else(|_| {
        provider_unavailable = true;
        Vec::new()
    });
    let principals = principals.map(|(value, _)| value).unwrap_or_else(|_| {
        provider_unavailable = true;
        Vec::new()
    });
    let mut rendered_forms = std::collections::BTreeMap::new();
    for form_version_id in form_version_ids {
        let Ok(form_version_id) = Uuid::parse_str(&form_version_id) else {
            provider_unavailable = true;
            continue;
        };
        match product::editor_form_schema_projection(state, grant, form_version_id).await {
            Ok((rendered, _)) => {
                rendered_forms.insert(form_version_id.to_string(), rendered);
            }
            Err(_) => provider_unavailable = true,
        }
    }
    Ok(DatasetEditorBootstrap {
        dataset,
        revision,
        datasets,
        forms,
        nodes,
        principals,
        rendered_forms,
        provider_error: provider_unavailable
            .then(|| "Some Dataset editor options are temporarily unavailable.".into()),
    })
}

fn referenced_form_version_ids(
    dataset: Option<&tessara_datasets_contract::DatasetProductDefinitionV1>,
    revision: Option<&tessara_datasets_contract::DatasetProductRevisionDetailV1>,
) -> std::collections::BTreeSet<String> {
    let mut ids = std::collections::BTreeSet::new();
    if let Some(dataset) = dataset {
        if let Some(source) = dataset.initial_source.as_ref() {
            collect_form_version_id(source, &mut ids);
        }
        for operation in &dataset.operations {
            collect_operation_form_version_id(operation, &mut ids);
        }
    }
    if let Some(revision) = revision {
        collect_form_version_id(&revision.initial_source, &mut ids);
        for operation in &revision.operations {
            collect_operation_form_version_id(operation, &mut ids);
        }
    }
    ids
}

fn collect_operation_form_version_id(
    operation: &DatasetProductOperationV1,
    ids: &mut std::collections::BTreeSet<String>,
) {
    if let DatasetProductOperationV1::AddSource { source, .. } = operation {
        collect_form_version_id(source, ids);
    }
}

fn collect_form_version_id(
    source: &DatasetProductSourceV1,
    ids: &mut std::collections::BTreeSet<String>,
) {
    if let DatasetProductSourceV1::Form {
        form_version_id, ..
    } = source
    {
        ids.insert(form_version_id.clone());
    }
}

fn parse_product_id(value: &str) -> Result<Uuid, DatasetModuleError> {
    Uuid::parse_str(value).map_err(|_| DatasetModuleError::NotFound("Dataset was not found".into()))
}

async fn document(
    state: &DatasetModuleState,
    headers: &HeaderMap,
    path: &str,
    title: &str,
    bootstrap: DatasetRouteBootstrap,
) -> Result<Response, DatasetModuleError> {
    let context = verified_shell_context(state, headers).await?;
    if accepts_lifecycle_bootstrap(headers) {
        let projection = BrowserLifecycleBootstrapV1 {
            schema_version: BrowserLifecycleBootstrapV1::SCHEMA_VERSION,
            definition_id: ModuleDefinitionId::new(MODULE_DEFINITION_ID).map_err(internal)?,
            release_version: Version::parse(MODULE_RELEASE_VERSION).map_err(internal)?,
            lifecycle_abi: Version::new(1, 0, 0),
            destination: SemanticRouteName::new(destination(&bootstrap)).map_err(internal)?,
            path: path.to_string(),
            title: title.to_string(),
            document_state: context.document_state,
            entry_asset: lifecycle_asset(
                DATASET_JS_SHA256,
                "dataset.js",
                "text/javascript; charset=utf-8",
            ),
            stylesheet_assets: vec![lifecycle_asset(
                DATASET_LIFECYCLE_CSS_SHA256,
                "dataset-lifecycle.css",
                "text/css; charset=utf-8",
            )],
            payload: serde_json::to_value(&bootstrap).map_err(internal)?,
        };
        let mut response = Json(projection).into_response();
        response.headers_mut().insert(
            header::CONTENT_TYPE,
            HeaderValue::from_static("application/vnd.tessara.module-view+json; version=1"),
        );
        no_store(&mut response);
        return Ok(response);
    }
    let html = render_dataset_document(&context, path, title, &bootstrap, MODULE_RELEASE_VERSION);
    let mut response = axum::response::Html(html).into_response();
    no_store(&mut response);
    Ok(response)
}

fn destination(value: &DatasetRouteBootstrap) -> &'static str {
    match value {
        DatasetRouteBootstrap::Directory { .. } => "datasets.directory",
        DatasetRouteBootstrap::Create { .. } => "datasets.create",
        DatasetRouteBootstrap::Detail { .. } => "datasets.detail",
        DatasetRouteBootstrap::Preview { .. } => "datasets.preview",
        DatasetRouteBootstrap::Edit { .. } => "datasets.edit",
        DatasetRouteBootstrap::Revisions { .. } => "datasets.revisions",
        DatasetRouteBootstrap::RevisionDetail { .. } => "datasets.revision_detail",
        DatasetRouteBootstrap::RevisionUnavailable { .. } => "datasets.revision_detail",
        DatasetRouteBootstrap::RevisionEdit { .. } => "datasets.revision_edit",
    }
}

fn accepts_lifecycle_bootstrap(headers: &HeaderMap) -> bool {
    headers
        .get(header::ACCEPT)
        .and_then(|value| value.to_str().ok())
        .is_some_and(|value| {
            value.split(',').any(|media| {
                media
                    .trim()
                    .starts_with("application/vnd.tessara.module-view+json")
            })
        })
}

fn lifecycle_asset(digest: &str, name: &str, content_type: &str) -> BrowserLifecycleAssetV1 {
    BrowserLifecycleAssetV1 {
        url: format!(
            "/_tessara/modules/{MODULE_DEFINITION_ID}/{MODULE_RELEASE_VERSION}/sha256:{digest}/{name}"
        ),
        digest: ArtifactDigest::new(format!("sha256:{digest}"))
            .expect("compiled Dataset asset digest"),
        content_type: content_type.into(),
    }
}

fn no_store(response: &mut Response) {
    response.headers_mut().insert(
        header::CACHE_CONTROL,
        HeaderValue::from_static("private, no-store"),
    );
    response.headers_mut().insert(
        header::VARY,
        HeaderValue::from_static("Accept, Authorization"),
    );
}

fn internal(error: impl std::fmt::Display) -> DatasetModuleError {
    DatasetModuleError::Internal(error.to_string())
}

async fn authorize_browser(
    state: &DatasetModuleState,
    headers: &HeaderMap,
    action: &str,
    contract: &str,
) -> Result<SignedEnvelopeV1<AuthorizationGrantV3>, DatasetModuleError> {
    let encoded = headers
        .get("x-tessara-authorization")
        .and_then(|value| value.to_str().ok())
        .ok_or(DatasetModuleError::Forbidden)?;
    let bytes = URL_SAFE_NO_PAD
        .decode(encoded)
        .map_err(|_| DatasetModuleError::Forbidden)?;
    let envelope: SignedEnvelopeV1<AuthorizationGrantV3> =
        serde_json::from_slice(&bytes).map_err(|_| DatasetModuleError::Forbidden)?;
    state
        .core_authorization_verifier
        .verify(&envelope)
        .map_err(|_| DatasetModuleError::Forbidden)?;
    let security = load_security_state(&state.pool)
        .await?
        .ok_or_else(|| DatasetModuleError::Unavailable("security state unavailable".into()))?;
    if !security.enabled || security.document_state != "enabled" {
        return Err(DatasetModuleError::Unavailable(
            "Dataset module is not enabled".into(),
        ));
    }
    let correlation_id = headers
        .get("x-tessara-correlation-id")
        .and_then(|value| value.to_str().ok())
        .and_then(|value| uuid::Uuid::parse_str(value).ok())
        .ok_or(DatasetModuleError::Forbidden)?;
    envelope
        .payload
        .validate_for(&AuthorizationValidationContextV3 {
            installation_id: security.installation_id,
            correlation_id,
            presenting_service: ModuleServicePrincipalV1::CoreGateway,
            audience: AuthorizationAudienceV1::ModuleInstance {
                module_instance_id: security.module_instance_id,
                module_definition_id: ModuleDefinitionId::new(MODULE_DEFINITION_ID)
                    .map_err(|error| DatasetModuleError::Internal(error.to_string()))?,
            },
            dependency_binding: DependencyBindingKey::new(CORE_DATASET_BINDING)
                .map_err(|error| DatasetModuleError::Internal(error.to_string()))?,
            functional_contract: FunctionalContractId::new(contract)
                .map_err(|error| DatasetModuleError::Internal(error.to_string()))?,
            action: action.into(),
            operation: AuthorizationGrantOperationV1::Read,
            resource_assertion: None,
            authorization_revision: security.authorization_revision as u64,
            organization_revision: security.organization_revision as u64,
            now: Utc::now(),
        })
        .map_err(|_| DatasetModuleError::Forbidden)?;
    Ok(envelope)
}

fn has_capability(grant: &AuthorizationGrantV3, capability: &str) -> bool {
    let expected = SecurityCapabilityId::new(capability).expect("static Dataset capability");
    grant
        .capability_scope_bindings
        .iter()
        .any(|binding| binding.capability == expected)
}
