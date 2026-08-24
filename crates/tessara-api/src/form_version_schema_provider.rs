//! Forms-owned immutable FormVersion catalog and schema provider.

use std::collections::BTreeSet;

use axum::{
    Router,
    body::{Body, Bytes},
    extract::State,
    http::{HeaderMap, StatusCode, header},
    response::Response,
    routing::post,
};
use serde::Serialize;
use sha2::{Digest, Sha256};
use sqlx::Row;
use tessara_forms_contract::{
    FORM_VERSION_CATALOG_ACTION, FORM_VERSION_CATALOG_PATH, FORM_VERSION_SCHEMA_ACTION,
    FORM_VERSION_SCHEMA_CONTRACT_ID, FORM_VERSION_SCHEMA_MEDIA_TYPE, FORM_VERSION_SCHEMA_PATH,
    FORM_VERSION_SCHEMA_VERSION, FormVersionCatalogItem, FormVersionCatalogRequest,
    FormVersionCatalogResponse, FormVersionField, FormVersionSchemaAction,
    FormVersionSchemaRequest, FormVersionSchemaResponse, FormVersionSection,
    RESPONSE_FORM_VERSION_SCHEMA_ACTION, RESPONSE_FORM_VERSION_SCHEMA_PATH,
};
use uuid::Uuid;

use crate::{
    db::AppState,
    error::{ApiError, ApiResult},
};

const MAX_CATALOG_PAGE_SIZE: u16 = 250;

pub(crate) fn routes() -> Router<AppState> {
    Router::new()
        .route(FORM_VERSION_CATALOG_PATH, post(catalog))
        .route(FORM_VERSION_SCHEMA_PATH, post(schema))
        .route(RESPONSE_FORM_VERSION_SCHEMA_PATH, post(response_schema))
}

async fn catalog(
    State(state): State<AppState>,
    headers: HeaderMap,
    body: Bytes,
) -> ApiResult<Response> {
    require_media_type(&headers)?;
    let grant = authorize(
        &state,
        &headers,
        FORM_VERSION_CATALOG_ACTION,
        FORM_VERSION_CATALOG_PATH,
        &body,
    )
    .await?;
    let request: FormVersionCatalogRequest =
        serde_json::from_slice(&body).map_err(|_| restricted())?;
    if request.action != FormVersionSchemaAction::Catalog
        || request.limit == 0
        || request.limit > MAX_CATALOG_PAGE_SIZE
    {
        return Err(restricted());
    }
    let after = request
        .cursor
        .as_deref()
        .map(Uuid::parse_str)
        .transpose()
        .map_err(|_| restricted())?;
    let scopes = authorized_scope(&grant);
    if scopes.is_empty() {
        return Err(restricted());
    }
    let rows = sqlx::query(
        "SELECT fv.id AS form_version_id
         FROM form_versions fv
         WHERE fv.status='published'::form_version_status
           AND EXISTS(
             SELECT 1 FROM form_scope_nodes fs WHERE fs.form_id=fv.form_id
           )
           AND NOT EXISTS(
             SELECT 1 FROM form_scope_nodes fs
             WHERE fs.form_id=fv.form_id AND NOT (fs.node_id=ANY($1))
           )
           AND ($2::uuid IS NULL OR fv.id>$2)
         ORDER BY fv.id
         LIMIT $3",
    )
    .bind(scopes.iter().copied().collect::<Vec<_>>())
    .bind(after)
    .bind(i64::from(request.limit) + 1)
    .fetch_all(&state.pool)
    .await?;
    let has_more = rows.len() > usize::from(request.limit);
    let selected = rows.into_iter().take(usize::from(request.limit));
    let mut items = Vec::new();
    for row in selected {
        let form_version_id: Uuid = row.try_get("form_version_id")?;
        let schema = load_schema(&state, form_version_id, &scopes).await?;
        let field_count = i64::try_from(schema.fields.len()).map_err(|_| {
            ApiError::Internal(anyhow::anyhow!(
                "FormVersion field count exceeds the contract limit"
            ))
        })?;
        items.push(FormVersionCatalogItem {
            form_id: schema.form_id,
            form_version_id,
            form_name: schema.form_name,
            form_slug: schema.form_slug,
            version_label: schema.version_label,
            version_major: schema.version_major,
            published: true,
            field_count,
            content_revision: schema.content_revision,
            content_digest: schema.content_digest,
        });
    }
    let next_cursor = has_more
        .then(|| items.last().map(|item| item.form_version_id.to_string()))
        .flatten();
    let requested_set = sqlx::query_scalar::<_, Uuid>(
        "SELECT DISTINCT fv.id
         FROM form_versions fv
         WHERE fv.status='published'::form_version_status
           AND EXISTS(
             SELECT 1 FROM form_scope_nodes fs WHERE fs.form_id=fv.form_id
           )
           AND NOT EXISTS(
             SELECT 1 FROM form_scope_nodes fs
             WHERE fs.form_id=fv.form_id AND NOT (fs.node_id=ANY($1))
           )
         ORDER BY fv.id",
    )
    .bind(scopes.iter().copied().collect::<Vec<_>>())
    .fetch_all(&state.pool)
    .await?;
    let requested_set_digest = digest_json(&requested_set)?;
    let response = FormVersionCatalogResponse {
        schema_version: FORM_VERSION_SCHEMA_VERSION,
        items,
        next_cursor,
        requested_set_digest,
    };
    response.validate().map_err(|_| restricted())?;
    contract_response(&response)
}

async fn schema(
    State(state): State<AppState>,
    headers: HeaderMap,
    body: Bytes,
) -> ApiResult<Response> {
    require_media_type(&headers)?;
    let grant = authorize(
        &state,
        &headers,
        FORM_VERSION_SCHEMA_ACTION,
        FORM_VERSION_SCHEMA_PATH,
        &body,
    )
    .await?;
    let request: FormVersionSchemaRequest =
        serde_json::from_slice(&body).map_err(|_| restricted())?;
    if request.action != FormVersionSchemaAction::ResolveSchema {
        return Err(restricted());
    }
    let scopes = authorized_scope(&grant);
    if scopes.is_empty() {
        return Err(restricted());
    }
    contract_response(&load_schema(&state, request.form_version_id, &scopes).await?)
}

async fn response_schema(
    State(state): State<AppState>,
    headers: HeaderMap,
    body: Bytes,
) -> ApiResult<Response> {
    require_media_type(&headers)?;
    let grant = authorize(
        &state,
        &headers,
        RESPONSE_FORM_VERSION_SCHEMA_ACTION,
        RESPONSE_FORM_VERSION_SCHEMA_PATH,
        &body,
    )
    .await?;
    let request: FormVersionSchemaRequest =
        serde_json::from_slice(&body).map_err(|_| restricted())?;
    if request.action != FormVersionSchemaAction::ResolveSchema {
        return Err(restricted());
    }
    let mut scopes = authorized_response_scope(&grant);
    if has_global_response_scope(&grant) {
        scopes.extend(
            sqlx::query_scalar::<_, Uuid>("SELECT id FROM nodes ORDER BY id")
                .fetch_all(&state.pool)
                .await?,
        );
    }
    if scopes.is_empty() {
        return Err(restricted());
    }
    contract_response(&load_schema(&state, request.form_version_id, &scopes).await?)
}

async fn authorize(
    state: &AppState,
    headers: &HeaderMap,
    action: &'static str,
    path: &'static str,
    body: &[u8],
) -> ApiResult<crate::module_service_requests::CoreProviderAuthorizationV1> {
    crate::module_service_requests::authorize_core_provider(
        state,
        headers,
        FORM_VERSION_SCHEMA_CONTRACT_ID,
        action,
        path,
        body,
        "FormVersion schema is unavailable",
    )
    .await
}

fn authorized_scope(
    grant: &crate::module_service_requests::CoreProviderAuthorizationV1,
) -> BTreeSet<Uuid> {
    grant
        .payload
        .capability_scope_bindings
        .iter()
        .filter(|binding| binding.capability.as_str() == "datasets:manage")
        .flat_map(|binding| {
            std::iter::once(binding.organization_root_id)
                .chain(binding.authorized_organization_ids.iter().copied())
        })
        .collect()
}

fn authorized_response_scope(
    grant: &crate::module_service_requests::CoreProviderAuthorizationV1,
) -> BTreeSet<Uuid> {
    grant
        .payload
        .capability_scope_bindings
        .iter()
        .filter(|binding| {
            matches!(
                binding.capability.as_str(),
                "submissions:respond" | "submissions:manage"
            )
        })
        .flat_map(|binding| {
            std::iter::once(binding.organization_root_id)
                .chain(binding.authorized_organization_ids.iter().copied())
        })
        .collect()
}

fn has_global_response_scope(
    grant: &crate::module_service_requests::CoreProviderAuthorizationV1,
) -> bool {
    grant
        .payload
        .capability_scope_bindings
        .iter()
        .any(|binding| {
            matches!(
                binding.capability.as_str(),
                "submissions:respond" | "submissions:manage"
            ) && binding.organization_root_id == grant.payload.installation_id
        })
}

async fn load_schema(
    state: &AppState,
    form_version_id: Uuid,
    authorized_scopes: &BTreeSet<Uuid>,
) -> ApiResult<FormVersionSchemaResponse> {
    let version = sqlx::query(
        "SELECT fv.form_id,f.name AS form_name,f.slug AS form_slug,
                fv.version_label,fv.version_major
         FROM form_versions fv JOIN forms f ON f.id=fv.form_id
         WHERE fv.id=$1 AND fv.status='published'::form_version_status",
    )
    .bind(form_version_id)
    .fetch_optional(&state.pool)
    .await?
    .ok_or_else(restricted)?;
    let form_id: Uuid = version.try_get("form_id")?;
    let source_scope_node_ids = sqlx::query_scalar::<_, Uuid>(
        "SELECT node_id FROM form_scope_nodes WHERE form_id=$1 ORDER BY node_id",
    )
    .bind(form_id)
    .fetch_all(&state.pool)
    .await?;
    if !source_scope_fully_managed(&source_scope_node_ids, authorized_scopes) {
        return Err(restricted());
    }
    let section_rows = sqlx::query(
        "SELECT id,title,description,position FROM form_sections
         WHERE form_version_id=$1 ORDER BY position,id",
    )
    .bind(form_version_id)
    .fetch_all(&state.pool)
    .await?;
    let sections = section_rows
        .into_iter()
        .map(|row| {
            let section_id: Uuid = row.try_get("id")?;
            Ok(FormVersionSection {
                section_id,
                key: section_id.to_string(),
                label: row.try_get("title")?,
                description: row.try_get("description")?,
                position: row.try_get("position")?,
            })
        })
        .collect::<Result<Vec<_>, sqlx::Error>>()?;
    let field_rows = sqlx::query(
        "SELECT ff.field_id,ff.key,ff.label,ff.field_type::text AS field_type,
                ff.required,ff.section_id,ff.position,ff.grid_row,ff.grid_column,
                ff.grid_width,ff.grid_height
         FROM form_fields ff
         JOIN form_sections fs ON fs.id=ff.section_id
         WHERE ff.form_version_id=$1
         ORDER BY fs.position,fs.id,ff.position,ff.grid_row,ff.grid_column,
                  ff.label,ff.key,ff.field_id",
    )
    .bind(form_version_id)
    .fetch_all(&state.pool)
    .await?;
    let fields = field_rows
        .into_iter()
        .map(|row| {
            Ok(FormVersionField {
                field_id: row.try_get("field_id")?,
                key: row.try_get("key")?,
                label: row.try_get("label")?,
                field_type: row.try_get("field_type")?,
                required: row.try_get("required")?,
                options: Vec::new(),
                section_id: row.try_get("section_id")?,
                position: row.try_get("position")?,
                grid_row: row.try_get("grid_row")?,
                grid_column: row.try_get("grid_column")?,
                grid_width: row.try_get("grid_width")?,
                grid_height: row.try_get("grid_height")?,
            })
        })
        .collect::<Result<Vec<_>, sqlx::Error>>()?;
    let form_name = version.try_get::<String, _>("form_name")?;
    let form_slug = version.try_get::<String, _>("form_slug")?;
    let version_label = version.try_get::<Option<String>, _>("version_label")?;
    let version_major = version.try_get("version_major")?;
    FormVersionSchemaResponse {
        schema_version: FORM_VERSION_SCHEMA_VERSION,
        form_id,
        form_version_id,
        form_name,
        form_slug,
        version_label,
        version_major,
        source_scope_node_ids,
        source_scope_revision: String::new(),
        source_scope_digest: String::new(),
        content_revision: String::new(),
        content_digest: String::new(),
        sections,
        fields,
    }
    .with_recomputed_digests()
    .map_err(|_| restricted())
}

fn source_scope_fully_managed(
    source_scope_node_ids: &[Uuid],
    authorized_scopes: &BTreeSet<Uuid>,
) -> bool {
    !source_scope_node_ids.is_empty()
        && source_scope_node_ids
            .iter()
            .all(|node_id| authorized_scopes.contains(node_id))
}

fn require_media_type(headers: &HeaderMap) -> ApiResult<()> {
    if headers
        .get(header::CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        != Some(FORM_VERSION_SCHEMA_MEDIA_TYPE)
    {
        return Err(restricted());
    }
    Ok(())
}

fn contract_response<T: Serialize>(value: &T) -> ApiResult<Response> {
    let body = serde_json::to_vec(value)
        .map_err(|error| ApiError::Internal(anyhow::Error::from(error)))?;
    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, FORM_VERSION_SCHEMA_MEDIA_TYPE)
        .body(Body::from(body))
        .map_err(|error| ApiError::Internal(anyhow::Error::from(error)))
}

fn digest_json<T: Serialize>(value: &T) -> ApiResult<String> {
    let encoded = serde_json::to_vec(value)
        .map_err(|error| ApiError::Internal(anyhow::Error::from(error)))?;
    Ok(format!("sha256:{:x}", Sha256::digest(encoded)))
}

fn restricted() -> ApiError {
    ApiError::NotFound("FormVersion schema is unavailable".into())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn form_source_scope_requires_exact_whole_scope_containment() {
        let first = Uuid::from_u128(1);
        let second = Uuid::from_u128(2);
        let source_scope = vec![first, second];

        assert!(source_scope_fully_managed(
            &source_scope,
            &BTreeSet::from([first, second, Uuid::from_u128(3)])
        ));
        assert!(!source_scope_fully_managed(
            &source_scope,
            &BTreeSet::from([first])
        ));
        assert!(!source_scope_fully_managed(
            &[],
            &BTreeSet::from([first, second])
        ));
    }
}
