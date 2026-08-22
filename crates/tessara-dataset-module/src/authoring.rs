//! Dataset-owned authoring validation and QuerySpec SQL compilation.
//!
//! This compiler consumes the canonical Dataset contract directly. Form facts
//! arrive through the FormVersion provider and all executable source SQL reads
//! only Dataset-owned imported or materialized tables.

use std::collections::{BTreeSet, HashMap};

use sha2::{Digest, Sha256};
use sqlx::{PgConnection, Postgres, Row, Transaction};
use tessara_data_ops::{
    AggregateFunction, AggregateMetric, AggregationPlan, DataField, FieldType, FilterOperator,
    validate_aggregation_plan,
};
use tessara_datasets_contract::{
    DatasetAuthoringRequestV1 as CreateDatasetRequest,
    DatasetProductAggregationMetricV1 as DatasetAggregationMetricRequest,
    DatasetProductCalculatedFieldV1 as DatasetCalculatedFieldRequest, DatasetProductFieldV1,
    DatasetProductJoinKeyV1 as DatasetJoinKeyRequest,
    DatasetProductOperationV1 as DatasetOperationRequest,
    DatasetProductProjectionFieldV1 as DatasetProjectionFieldRequest,
    DatasetProductRestrictionPolicyV1 as DatasetRestrictionPolicyRequest,
    DatasetProductRowFilterV1 as DatasetRowFilterRequest,
    DatasetProductRowPickerV1 as DatasetRowPickerRequest,
    DatasetProductSourceV1 as DatasetSourceRequest,
};
use tessara_forms_contract::{
    FORM_VERSION_SCHEMA_ACTION, FORM_VERSION_SCHEMA_BINDING_KEY, FORM_VERSION_SCHEMA_CONTRACT_ID,
    FORM_VERSION_SCHEMA_MEDIA_TYPE, FORM_VERSION_SCHEMA_PATH, FORM_VERSION_SCHEMA_VERSION,
    FormVersionSchemaAction, FormVersionSchemaRequest, FormVersionSchemaResponse,
};
use tessara_module_contract::{AuthorizationGrantV3, SignedEnvelopeV1};
use uuid::Uuid;

use crate::{
    DatasetModuleError, DatasetModuleState,
    provider_client::{self, ProviderAction},
};

const FORM_SCHEMA_PROVIDER: ProviderAction = ProviderAction {
    binding: FORM_VERSION_SCHEMA_BINDING_KEY,
    contract: FORM_VERSION_SCHEMA_CONTRACT_ID,
    action: FORM_VERSION_SCHEMA_ACTION,
    path: FORM_VERSION_SCHEMA_PATH,
    media_type: FORM_VERSION_SCHEMA_MEDIA_TYPE,
    retry_safe_observation: true,
};

type ApiResult<T> = Result<T, ApiError>;

#[derive(Debug, thiserror::Error)]
pub(crate) enum ApiError {
    #[error("{0}")]
    BadRequest(String),
    #[error(transparent)]
    Database(#[from] sqlx::Error),
    #[error(transparent)]
    Internal(#[from] anyhow::Error),
    #[error(transparent)]
    Module(#[from] DatasetModuleError),
}

impl From<ApiError> for DatasetModuleError {
    fn from(error: ApiError) -> Self {
        match error {
            ApiError::BadRequest(message) => Self::ValidationFailed(message),
            ApiError::Database(error) => Self::Database(error),
            ApiError::Internal(error) => Self::Internal(error.to_string()),
            ApiError::Module(error) => error,
        }
    }
}

#[derive(Clone)]
pub(crate) struct ValidatedDatasetSource {
    pub(crate) source_binding_id: Option<Uuid>,
    pub(crate) reference: DatasetSourceRequest,
    pub(crate) source_alias: String,
    pub(crate) form_version_id: Option<Uuid>,
    pub(crate) source_name: String,
    pub(crate) source_slug: Option<String>,
    pub(crate) source_version_label: Option<String>,
    pub(crate) source_scope_node_ids: Vec<Uuid>,
    pub(crate) source_scope_revision: String,
    pub(crate) source_scope_digest: String,
    pub(crate) source_content_revision: String,
    pub(crate) source_content_digest: String,
    pub(crate) position: i32,
}

#[derive(Clone)]
pub(crate) struct ValidatedDatasetField {
    pub(crate) id: Option<Uuid>,
    pub(crate) key: String,
    pub(crate) label: String,
    pub(crate) source_alias: String,
    pub(crate) source_field_key: String,
    pub(crate) source_field_id: Option<Uuid>,
    pub(crate) field_type: String,
    pub(crate) position: i32,
}

pub(crate) struct CompiledDataset {
    pub(crate) initial_source: DatasetSourceRequest,
    pub(crate) operations: Vec<DatasetOperationRequest>,
    pub(crate) restriction_policy: Option<DatasetRestrictionPolicyRequest>,
    pub(crate) generated_sql: String,
    pub(crate) sources: Vec<ValidatedDatasetSource>,
    pub(crate) fields: Vec<ValidatedDatasetField>,
}

struct QuerySpecBuilder<'state, 'connection> {
    state: &'state DatasetModuleState,
    grant: &'state dyn provider_client::ProviderAuthorization,
    dataset_id: Option<Uuid>,
    requested_scope_node_ids: BTreeSet<Uuid>,
    authorized_scope_node_ids: BTreeSet<Uuid>,
    ctes: Vec<String>,
    current_cte: String,
    fields: Vec<ValidatedDatasetField>,
    cte_index: usize,
    aliases: BTreeSet<String>,
    sources: Vec<ValidatedDatasetSource>,
    connection: Option<&'connection mut PgConnection>,
}

#[derive(Clone)]
struct CompiledSource {
    cte_name: String,
    fields: Vec<ValidatedDatasetField>,
}

struct UnionColumnMapping {
    output_key: String,
    left_key: Option<String>,
    right_key: Option<String>,
}

#[derive(Clone)]
struct ValidatedAggregation {
    group_fields: Vec<String>,
    metrics: Vec<ValidatedAggregationMetric>,
    row_picker: Option<ValidatedRowPicker>,
}

#[derive(Clone)]
struct ValidatedAggregationMetric {
    key: String,
    label: String,
    function: AggregateFunction,
    source_field_key: Option<String>,
    field_type: String,
    position: i32,
}

#[derive(Clone)]
struct ValidatedRowPicker {
    sort_fields: Vec<ValidatedRowPickerSort>,
    direction: String,
}

#[derive(Clone)]
struct ValidatedRowPickerSort {
    field_key: String,
    field_type: String,
}

#[derive(Clone)]
struct ValidatedRowFilter {
    field_key: String,
    field_type: String,
    operator: FilterOperator,
    value: Option<String>,
    value_field_key: Option<String>,
}

#[derive(Clone)]
struct ValidatedCalculatedField {
    key: String,
    label: String,
    base_field_key: String,
    functions: Vec<ValidatedCalculationFunction>,
    field_type: String,
}

#[derive(Clone)]
struct ValidatedCalculationFunction {
    function: CalculationFunction,
    argument: Option<String>,
    argument_field_key: Option<String>,
    input_type: String,
}

#[derive(Clone, Copy)]
enum CalculationFunction {
    Trim,
    Uppercase,
    Lowercase,
    Prefix,
    Suffix,
    Concat,
    Coalesce,
    Constant,
    MapValue,
    Add,
    Subtract,
    Multiply,
    Divide,
    Round,
    FormatDate,
    GreaterThan,
    GreaterThanOrEqual,
    LessThan,
    LessThanOrEqual,
    Equal,
    NotEqual,
    IsEmpty,
    IsNotEmpty,
    ToText,
    ToNumber,
    ToBoolean,
    ToDate,
}

#[derive(Clone, Default)]
struct ValidatedRestrictionPolicy {
    internal_field_key: Option<String>,
    restricted_field_key: Option<String>,
    confidential_field_key: Option<String>,
}

#[derive(Clone)]
struct SourceCompileField {
    key: String,
    source_field_key: String,
    source_field_id: Option<Uuid>,
}

#[derive(Clone)]
struct DatasetAggregationRequest {
    group_fields: Vec<String>,
    metrics: Vec<DatasetAggregationMetricRequest>,
    row_picker: Option<DatasetRowPickerRequest>,
}

pub(crate) async fn compile_dataset_definition(
    state: &DatasetModuleState,
    grant: &SignedEnvelopeV1<AuthorizationGrantV3>,
    dataset_id: Option<Uuid>,
    payload: &CreateDatasetRequest,
) -> ApiResult<CompiledDataset> {
    compile_dataset_definition_with_transaction(state, grant, None, dataset_id, payload).await
}

pub(crate) async fn compile_dataset_definition_in_transaction(
    state: &DatasetModuleState,
    grant: &dyn provider_client::ProviderAuthorization,
    transaction: &mut Transaction<'_, Postgres>,
    dataset_id: Option<Uuid>,
    payload: &CreateDatasetRequest,
) -> ApiResult<CompiledDataset> {
    compile_dataset_definition_with_transaction(
        state,
        grant,
        Some(&mut **transaction),
        dataset_id,
        payload,
    )
    .await
}

async fn compile_dataset_definition_with_transaction(
    state: &DatasetModuleState,
    grant: &dyn provider_client::ProviderAuthorization,
    connection: Option<&mut PgConnection>,
    dataset_id: Option<Uuid>,
    payload: &CreateDatasetRequest,
) -> ApiResult<CompiledDataset> {
    let mut spec = QuerySpecBuilder::init(
        state,
        grant,
        connection,
        dataset_id,
        &payload.visibility_node_ids,
        &payload.initial_source,
    )
    .await?;
    spec.apply_operations(&payload.operations).await?;
    let restriction_policy =
        validate_dataset_restriction_policy(payload.restriction_policy.clone(), spec.fields())?;
    require_dataset_output_fields(spec.fields())?;
    let generated_sql = spec.final_sql(&restriction_policy);
    Ok(CompiledDataset {
        initial_source: payload.initial_source.clone(),
        operations: payload.operations.clone(),
        restriction_policy: payload.restriction_policy.clone(),
        generated_sql,
        sources: spec.sources,
        fields: spec.fields,
    })
}

impl<'state, 'connection> QuerySpecBuilder<'state, 'connection> {
    async fn require_dataset_fully_authorized(&mut self, dataset_id: Uuid) -> ApiResult<()> {
        let query = sqlx::query_scalar::<_, Uuid>(
            "SELECT node_id FROM dataset_scope_nodes WHERE dataset_id=$1 ORDER BY node_id",
        )
        .bind(dataset_id);
        let scope = match self.connection.as_deref_mut() {
            Some(connection) => query.fetch_all(connection).await?,
            None => query.fetch_all(&self.state.pool).await?,
        };
        if scope.is_empty()
            || scope
                .iter()
                .any(|node_id| !self.authorized_scope_node_ids.contains(node_id))
        {
            return Err(ApiError::BadRequest("Dataset source was not found".into()));
        }
        Ok(())
    }

    async fn require_materialized_system_columns(
        &mut self,
        schema: &str,
        table: &str,
    ) -> ApiResult<(String, String)> {
        let query = sqlx::query_as::<_, (bool, bool)>(
            "SELECT
                EXISTS (
                    SELECT 1 FROM information_schema.columns
                    WHERE table_schema=$1 AND table_name=$2
                      AND column_name='__restriction_tier'
                ),
                EXISTS (
                    SELECT 1 FROM information_schema.columns
                    WHERE table_schema=$1 AND table_name=$2
                      AND column_name='__scope_node_ids' AND udt_name='_uuid'
                )",
        )
        .bind(schema)
        .bind(table);
        let (has_restriction_tier, has_scope_node_ids) = match self.connection.as_deref_mut() {
            Some(connection) => query.fetch_one(connection).await?,
            None => query.fetch_one(&self.state.pool).await?,
        };
        if !has_restriction_tier || !has_scope_node_ids {
            return Err(ApiError::BadRequest(
                "Dataset source materialization is incompatible".into(),
            ));
        }
        Ok((
            format!(
                "{} AS {}",
                quote_identifier("__restriction_tier"),
                quote_identifier("__restriction_tier")
            ),
            format!(
                "{} AS {}",
                quote_identifier("__scope_node_ids"),
                quote_identifier("__scope_node_ids")
            ),
        ))
    }

    async fn init(
        state: &'state DatasetModuleState,
        grant: &'state dyn provider_client::ProviderAuthorization,
        connection: Option<&'connection mut PgConnection>,
        dataset_id: Option<Uuid>,
        requested_scope_node_ids: &[String],
        initial_source: &DatasetSourceRequest,
    ) -> ApiResult<Self> {
        let mut builder = Self {
            state,
            grant,
            dataset_id,
            requested_scope_node_ids: requested_scope_node_ids
                .iter()
                .map(|node_id| parse_uuid("Dataset visibility node", node_id))
                .collect::<ApiResult<_>>()?,
            authorized_scope_node_ids: authorized_manage_scope(grant),
            ctes: Vec::new(),
            current_cte: String::new(),
            fields: Vec::new(),
            cte_index: 0,
            aliases: BTreeSet::new(),
            sources: Vec::new(),
            connection,
        };
        let source = builder.compile_source(initial_source).await?;
        builder.current_cte = source.cte_name;
        builder.fields = source.fields;
        Ok(builder)
    }

    fn fields(&self) -> &[ValidatedDatasetField] {
        &self.fields
    }

    async fn apply_operations(&mut self, operations: &[DatasetOperationRequest]) -> ApiResult<()> {
        for (index, operation) in operations.iter().enumerate() {
            validate_operation_position(operation, index)?;
            self.apply_operation(operation).await?;
        }
        Ok(())
    }

    async fn apply_operation(&mut self, operation: &DatasetOperationRequest) -> ApiResult<()> {
        match operation {
            DatasetOperationRequest::AddSource {
                source,
                add_type,
                join_keys,
                position,
            } => {
                let right = self.compile_source(source).await?;
                self.apply_add_source(right, add_type, join_keys, *position)
            }
            DatasetOperationRequest::Projection { fields, .. } => self.apply_projection(fields),
            DatasetOperationRequest::Aggregation {
                group_fields,
                metrics,
                row_picker,
                ..
            } => self.apply_aggregation(group_fields, metrics, row_picker),
            DatasetOperationRequest::CalculatedFields { fields, .. } => {
                self.apply_calculated_fields(fields)
            }
            DatasetOperationRequest::Filter { filters, .. } => self.apply_filter(filters),
        }
    }

    fn apply_add_source(
        &mut self,
        right: CompiledSource,
        add_type: &str,
        join_keys: &[DatasetJoinKeyRequest],
        operation_position: i32,
    ) -> ApiResult<()> {
        match add_type {
            "union" => self.apply_union_add_type(right, false, operation_position),
            "union_all" => self.apply_union_add_type(right, true, operation_position),
            "left_join" | "inner_join" | "outer_join" => {
                self.apply_join_add_type(right, add_type, join_keys)
            }
            _ => Err(ApiError::BadRequest(format!(
                "unsupported add source type '{add_type}'"
            ))),
        }
    }

    async fn compile_source(&mut self, source: &DatasetSourceRequest) -> ApiResult<CompiledSource> {
        match source {
            DatasetSourceRequest::Form {
                alias,
                form_id,
                form_version_id,
            } => {
                self.compile_form_source(
                    alias,
                    parse_uuid("dataset form", form_id)?,
                    parse_uuid("dataset form version", form_version_id)?,
                )
                .await
            }
            DatasetSourceRequest::Dataset {
                alias,
                dataset_id,
                dataset_revision_id,
            } => {
                self.compile_dataset_source(
                    alias,
                    parse_uuid("source Dataset", dataset_id)?,
                    parse_uuid("source Dataset revision", dataset_revision_id)?,
                )
                .await
            }
            DatasetSourceRequest::DatasetMajor {
                alias,
                dataset_id,
                version_major,
            } => {
                self.compile_dataset_major_source(
                    alias,
                    parse_uuid("source Dataset", dataset_id)?,
                    *version_major,
                )
                .await
            }
        }
    }

    async fn compile_form_source(
        &mut self,
        alias: &str,
        form_id: Uuid,
        form_version_id: Uuid,
    ) -> ApiResult<CompiledSource> {
        require_identifier("dataset source alias", alias)?;
        if form_id.is_nil() {
            return Err(ApiError::BadRequest(
                "dataset form source must reference a form".into(),
            ));
        }
        if form_version_id.is_nil() {
            return Err(ApiError::BadRequest(
                "dataset form source must reference a form version".into(),
            ));
        }
        if !self.aliases.insert(alias.to_string()) {
            return Err(ApiError::BadRequest(format!(
                "dataset expression alias '{alias}' is duplicated"
            )));
        }
        let form_version_wire: serde_json::Value = provider_client::post(
            self.state,
            self.grant,
            FORM_SCHEMA_PROVIDER,
            &FormVersionSchemaRequest {
                schema_version: FORM_VERSION_SCHEMA_VERSION,
                action: FormVersionSchemaAction::ResolveSchema,
                form_version_id,
            },
        )
        .await?;
        let form_version = validate_form_version_schema(
            form_version_wire,
            form_id,
            form_version_id,
            &self.requested_scope_node_ids,
            &self.authorized_scope_node_ids,
        )?;
        let source = validated_form_source(
            self.dataset_id,
            alias,
            form_id,
            form_version_id,
            self.sources.len() as i32,
            &form_version,
        );
        let source_binding_id = source.source_binding_id.ok_or_else(|| {
            ApiError::Internal(anyhow::anyhow!(
                "validated Form source has no binding identity"
            ))
        })?;
        let cte_name = self.next_cte_name(alias)?;
        let fields = load_form_source_catalog(&source, &form_version)?;
        let source_fields = fields
            .iter()
            .map(|field| SourceCompileField {
                key: field.key.clone(),
                source_field_key: field.source_field_key.clone(),
                source_field_id: field.source_field_id,
            })
            .collect::<Vec<_>>();
        let mut value_field_ids = BTreeSet::new();
        let mut group_by_expressions = BTreeSet::from([
            "imported.source_binding_id".to_string(),
            "imported.response_id".to_string(),
            "imported.form_version_id".to_string(),
            "imported.restriction_tier".to_string(),
            "imported.scope_node_ids".to_string(),
        ]);
        let select_columns = source_fields
            .iter()
            .map(|field| {
                let column = quote_identifier(&field.key);
                if let Some(expression) = system_source_field_expression(&field.source_field_key) {
                    group_by_expressions.insert(expression.to_string());
                    Ok(format!("{expression} AS {column}"))
                } else {
                    let Some(source_field_id) = field.source_field_id else {
                        return Err(ApiError::BadRequest(format!(
                            "dataset field '{}' cannot be projected without a stable source field id",
                            field.key
                        )));
                    };
                    value_field_ids.insert(source_field_id);
                    Ok(format!(
                        "MAX(imported_value.value_text) FILTER (WHERE imported_value.field_id = {}::uuid) AS {column}",
                        sql_literal(&source_field_id.to_string())
                    ))
                }
            })
            .collect::<ApiResult<Vec<_>>>()?
            .join(",\n                ");
        let extra_select_columns = if select_columns.is_empty() {
            String::new()
        } else {
            format!(",\n                {select_columns}")
        };
        let value_join = if value_field_ids.is_empty() {
            String::new()
        } else {
            let field_id_filter = value_field_ids
                .iter()
                .map(|field_id| format!("{}::uuid", sql_literal(&field_id.to_string())))
                .collect::<Vec<_>>()
                .join(", ");
            format!(
                r#"
            LEFT JOIN dataset_imported_response_values imported_value
              ON imported_value.source_binding_id = imported.source_binding_id
             AND imported_value.response_id = imported.response_id
             AND imported_value.form_version_id = imported.form_version_id
             AND imported_value.field_id IN ({field_id_filter})"#
            )
        };
        let group_by = if value_field_ids.is_empty() {
            String::new()
        } else {
            format!(
                r#"
            GROUP BY {}"#,
                group_by_expressions
                    .into_iter()
                    .collect::<Vec<_>>()
                    .join(", ")
            )
        };
        let sql = format!(
            r#"{cte_name} AS (
            SELECT
                md5(concat_ws('|', 'form', imported.form_version_id::text, imported.response_id::text)) AS __row_id,
                imported.restriction_tier AS __restriction_tier,
                imported.scope_node_ids AS __scope_node_ids{extra_select_columns}
            FROM dataset_imported_responses imported{value_join}
            WHERE imported.tombstoned = FALSE
              AND imported.status = 'submitted'
              AND imported.source_binding_id = {}::uuid
              AND imported.form_version_id = {}::uuid{group_by}
        )"#,
            sql_literal(&source_binding_id.to_string()),
            sql_literal(&form_version_id.to_string())
        );
        self.ctes.push(sql);
        self.sources.push(source);
        Ok(CompiledSource { cte_name, fields })
    }

    async fn compile_dataset_source(
        &mut self,
        alias: &str,
        dataset_id: Uuid,
        dataset_revision_id: Uuid,
    ) -> ApiResult<CompiledSource> {
        require_identifier("dataset source alias", alias)?;
        if !self.aliases.insert(alias.to_string()) {
            return Err(ApiError::BadRequest(format!(
                "dataset expression alias '{alias}' is duplicated"
            )));
        }
        if Some(dataset_id) == self.dataset_id {
            return Err(ApiError::BadRequest(
                "dataset definitions cannot reference themselves".into(),
            ));
        }
        self.require_dataset_fully_authorized(dataset_id).await?;
        let query = sqlx::query(
            r#"
            SELECT r.materialized_schema,r.materialized_table,r.version_label,r.output_fields,
                   r.resource_revision,d.name,d.slug,
                   ARRAY(SELECT node_id FROM dataset_scope_nodes WHERE dataset_id=d.id ORDER BY node_id) AS scope_node_ids,
                   (SELECT requested_set_revision FROM dataset_scope_nodes
                    WHERE dataset_id=d.id ORDER BY node_id LIMIT 1) AS scope_revision,
                   (SELECT requested_set_digest FROM dataset_scope_nodes
                    WHERE dataset_id=d.id ORDER BY node_id LIMIT 1) AS scope_digest
            FROM dataset_revisions r JOIN datasets d ON d.id=r.dataset_id
            WHERE r.id = $1
              AND r.dataset_id = $2
              AND r.materialized_table IS NOT NULL
            "#,
        )
        .bind(dataset_revision_id)
        .bind(dataset_id);
        let row = match self.connection.as_deref_mut() {
            Some(connection) => query.fetch_optional(connection).await?,
            None => query.fetch_optional(&self.state.pool).await?,
        }
        .ok_or_else(|| {
            ApiError::BadRequest(format!(
                "dataset revision {dataset_revision_id} is not materialized"
            ))
        })?;
        let schema: String = row.try_get("materialized_schema")?;
        let table: String = row.try_get("materialized_table")?;
        let output_fields: serde_json::Value = row.try_get("output_fields")?;
        let source_content_digest = digest_serializable(&output_fields)?;
        let source_content_revision =
            format!("resource:{}", row.try_get::<i64, _>("resource_revision")?);
        let (restriction_tier_select, scope_node_ids_select) = self
            .require_materialized_system_columns(&schema, &table)
            .await?;
        let cte_name = self.next_cte_name(alias)?;
        let source_binding_id = source_binding_uuid(
            self.dataset_id,
            alias,
            "dataset_revision",
            &dataset_revision_id.to_string(),
        );
        let source = ValidatedDatasetSource {
            source_binding_id: Some(source_binding_id),
            reference: DatasetSourceRequest::Dataset {
                alias: alias.to_owned(),
                dataset_id: dataset_id.to_string(),
                dataset_revision_id: dataset_revision_id.to_string(),
            },
            source_alias: alias.to_string(),
            form_version_id: None,
            source_name: row.try_get("name")?,
            source_slug: row.try_get("slug")?,
            source_version_label: row.try_get("version_label")?,
            source_scope_node_ids: row.try_get("scope_node_ids")?,
            source_scope_revision: row.try_get("scope_revision")?,
            source_scope_digest: row.try_get("scope_digest")?,
            source_content_revision,
            source_content_digest,
            position: self.sources.len() as i32,
        };
        let fields = dataset_source_catalog_from_output_fields(&source, output_fields)?;
        let source_fields = fields
            .iter()
            .map(|field| SourceCompileField {
                key: field.key.clone(),
                source_field_key: field.source_field_key.clone(),
                source_field_id: None,
            })
            .collect::<Vec<_>>();
        let select_columns = source_fields
            .iter()
            .map(|field| {
                format!(
                    "{}::text AS {}",
                    quote_identifier(&field.source_field_key),
                    quote_identifier(&field.key)
                )
            })
            .collect::<Vec<_>>()
            .join(",\n                ");
        let extra_select_columns = if select_columns.is_empty() {
            String::new()
        } else {
            format!(",\n                {select_columns}")
        };
        self.ctes.push(format!(
            r#"{cte_name} AS (
            SELECT
                __row_id,
                {restriction_tier_select},
                {scope_node_ids_select}{extra_select_columns}
            FROM {schema}.{table}
        )"#,
            schema = quote_identifier(&schema),
            table = quote_identifier(&table)
        ));
        self.sources.push(source);
        Ok(CompiledSource { cte_name, fields })
    }

    async fn compile_dataset_major_source(
        &mut self,
        alias: &str,
        dataset_id: Uuid,
        version_major: i32,
    ) -> ApiResult<CompiledSource> {
        require_identifier("dataset source alias", alias)?;
        if !self.aliases.insert(alias.to_string()) {
            return Err(ApiError::BadRequest(format!(
                "dataset expression alias '{alias}' is duplicated"
            )));
        }
        if Some(dataset_id) == self.dataset_id {
            return Err(ApiError::BadRequest(
                "dataset definitions cannot reference themselves".into(),
            ));
        }
        if version_major < 1 {
            return Err(ApiError::BadRequest(
                "dataset major version must be greater than zero".into(),
            ));
        }
        self.require_dataset_fully_authorized(dataset_id).await?;
        let query = sqlx::query(
            r#"
            SELECT m.materialized_schema,m.materialized_table,d.name,d.slug,d.resource_revision,
                   ARRAY(SELECT node_id FROM dataset_scope_nodes WHERE dataset_id=d.id ORDER BY node_id) AS scope_node_ids,
                   (SELECT requested_set_revision FROM dataset_scope_nodes
                    WHERE dataset_id=d.id ORDER BY node_id LIMIT 1) AS scope_revision,
                   (SELECT requested_set_digest FROM dataset_scope_nodes
                    WHERE dataset_id=d.id ORDER BY node_id LIMIT 1) AS scope_digest,
                   (SELECT output_fields FROM dataset_revisions r
                    WHERE r.dataset_id=d.id AND r.version_major=m.version_major
                      AND r.status IN ('published','superseded')
                    ORDER BY r.version_minor DESC,r.version_patch DESC,r.version_number DESC LIMIT 1) AS output_fields
            FROM dataset_major_materializations m JOIN datasets d ON d.id=m.dataset_id
            WHERE m.dataset_id = $1
              AND m.version_major = $2
              AND m.materialized_table IS NOT NULL
              AND m.rebuild_status = 'ready'
            "#,
        )
        .bind(dataset_id)
        .bind(version_major);
        let row = match self.connection.as_deref_mut() {
            Some(connection) => query.fetch_optional(connection).await?,
            None => query.fetch_optional(&self.state.pool).await?,
        }
        .ok_or_else(|| {
            ApiError::BadRequest(format!(
                "dataset version {version_major} is not materialized"
            ))
        })?;
        let schema: String = row.try_get("materialized_schema")?;
        let table: String = row.try_get("materialized_table")?;
        let output_fields: serde_json::Value = row.try_get("output_fields")?;
        let source_content_digest = digest_serializable(&output_fields)?;
        let source_content_revision =
            format!("resource:{}", row.try_get::<i64, _>("resource_revision")?);
        let (restriction_tier_select, scope_node_ids_select) = self
            .require_materialized_system_columns(&schema, &table)
            .await?;
        let cte_name = self.next_cte_name(alias)?;
        let source_binding_id = source_binding_uuid(
            self.dataset_id,
            alias,
            "dataset_major_line",
            &format!("{dataset_id}:{version_major}"),
        );
        let source = ValidatedDatasetSource {
            source_binding_id: Some(source_binding_id),
            reference: DatasetSourceRequest::DatasetMajor {
                alias: alias.to_owned(),
                dataset_id: dataset_id.to_string(),
                version_major,
            },
            source_alias: alias.to_string(),
            form_version_id: None,
            source_name: row.try_get("name")?,
            source_slug: row.try_get("slug")?,
            source_version_label: Some(format!("Version {version_major}")),
            source_scope_node_ids: row.try_get("scope_node_ids")?,
            source_scope_revision: row.try_get("scope_revision")?,
            source_scope_digest: row.try_get("scope_digest")?,
            source_content_revision,
            source_content_digest,
            position: self.sources.len() as i32,
        };
        let raw_fields: Vec<DatasetProductFieldV1> = serde_json::from_value(output_fields)
            .map_err(|error| {
                ApiError::Internal(anyhow::anyhow!(
                    "stored Dataset major fields are invalid: {error}"
                ))
            })?;
        let source_fields = raw_fields
            .iter()
            .map(|field| SourceCompileField {
                key: canonical_source_column_key(alias, &field.key),
                source_field_key: field.key.clone(),
                source_field_id: None,
            })
            .collect::<Vec<_>>();
        let fields = raw_fields
            .into_iter()
            .enumerate()
            .map(|(index, field)| ValidatedDatasetField {
                id: None,
                key: canonical_source_column_key(alias, &field.key),
                label: field.label,
                source_alias: alias.to_string(),
                source_field_key: field.key,
                source_field_id: None,
                field_type: field.field_type,
                position: index as i32,
            })
            .collect::<Vec<_>>();
        let select_columns = source_fields
            .iter()
            .map(|field| {
                format!(
                    "{}::text AS {}",
                    quote_identifier(&field.source_field_key),
                    quote_identifier(&field.key)
                )
            })
            .collect::<Vec<_>>()
            .join(",\n                ");
        let extra_select_columns = if select_columns.is_empty() {
            String::new()
        } else {
            format!(",\n                {select_columns}")
        };
        self.ctes.push(format!(
            r#"{cte_name} AS (
            SELECT
                __row_id,
                {restriction_tier_select},
                {scope_node_ids_select}{extra_select_columns}
            FROM {schema}.{table}
        )"#,
            schema = quote_identifier(&schema),
            table = quote_identifier(&table)
        ));
        self.sources.push(source);
        Ok(CompiledSource { cte_name, fields })
    }

    fn apply_join_add_type(
        &mut self,
        right: CompiledSource,
        operation: &str,
        join_keys: &[DatasetJoinKeyRequest],
    ) -> ApiResult<()> {
        let join = match operation.trim() {
            "left_join" => "LEFT JOIN",
            "inner_join" => "INNER JOIN",
            "outer_join" => "FULL OUTER JOIN",
            "union" | "union_all" => {
                return Err(ApiError::BadRequest(
                    "add_source join modes require a join type".into(),
                ));
            }
            other => {
                return Err(ApiError::BadRequest(format!(
                    "unsupported join type '{other}'"
                )));
            }
        };
        let cte_name = self.next_cte_name("op")?;
        let left_columns = catalog_columns(&self.fields);
        let right_columns = catalog_columns(&right.fields);
        let output_fields = merge_field_catalogs(&self.fields, &right.fields)?;
        let output_columns = catalog_columns(&output_fields);
        if join_keys.is_empty() {
            return Err(ApiError::BadRequest(
                "join operations require at least one explicit join key".into(),
            ));
        }
        let predicates = join_keys
            .iter()
            .map(|key| {
                if !left_columns.contains(&key.left_field) {
                    return Err(ApiError::BadRequest(format!(
                        "left join key '{}' is not available from the current input",
                        key.left_field
                    )));
                }
                if !right_columns.contains(&key.right_field) {
                    return Err(ApiError::BadRequest(format!(
                        "right join key '{}' is not available from the joined source",
                        key.right_field
                    )));
                }
                Ok(format!(
                    "l.{} = r.{}",
                    quote_identifier(&key.left_field),
                    quote_identifier(&key.right_field)
                ))
            })
            .collect::<ApiResult<Vec<_>>>()?
            .join(" AND ");
        let columns = ordered_columns(&output_columns)
            .iter()
            .map(|column| coalesced_join_expression(&left_columns, &right_columns, column))
            .collect::<Vec<_>>()
            .join(",\n                ");
        self.ctes.push(format!(
            r#"{cte_name} AS (
            SELECT
                {columns}
            FROM {} l
            {join} {} r ON {predicates}
        )"#,
            quote_identifier(&self.current_cte),
            quote_identifier(&right.cte_name)
        ));
        self.current_cte = cte_name;
        self.fields = output_fields;
        Ok(())
    }

    fn apply_union_add_type(
        &mut self,
        right: CompiledSource,
        union_all: bool,
        operation_position: i32,
    ) -> ApiResult<()> {
        let cte_name = self.next_cte_name("op")?;
        let left_columns = catalog_columns(&self.fields);
        let right_columns = catalog_columns(&right.fields);
        let union_alias = union_output_alias(operation_position);
        let (output_fields, output_columns) =
            union_field_catalog(&self.fields, &right.fields, &union_alias)?;
        let operation = if union_all { "UNION ALL" } else { "UNION" };
        let left_selects = output_columns
            .iter()
            .map(|column| {
                select_union_expression(
                    &left_columns,
                    column.left_key.as_deref(),
                    &column.output_key,
                )
            })
            .collect::<Vec<_>>()
            .join(",\n                ");
        let right_selects = output_columns
            .iter()
            .map(|column| {
                select_union_expression(
                    &right_columns,
                    column.right_key.as_deref(),
                    &column.output_key,
                )
            })
            .collect::<Vec<_>>()
            .join(",\n                ");
        self.ctes.push(format!(
            r#"{cte_name} AS (
            SELECT
                {left_selects}
            FROM {}
            {operation}
            SELECT
                {right_selects}
            FROM {}
        )"#,
            quote_identifier(&self.current_cte),
            quote_identifier(&right.cte_name)
        ));
        self.current_cte = cte_name;
        self.fields = output_fields;
        Ok(())
    }

    fn next_cte_name(&mut self, seed: &str) -> ApiResult<String> {
        require_identifier("dataset expression alias", seed)?;
        self.cte_index += 1;
        Ok(format!("{}_{}", sanitize_identifier(seed), self.cte_index))
    }
}

impl QuerySpecBuilder<'_, '_> {
    fn apply_projection(&mut self, requests: &[DatasetProjectionFieldRequest]) -> ApiResult<()> {
        let mut requests = requests
            .iter()
            .filter(|field| {
                !field.key.trim().is_empty()
                    || !field
                        .input_field_key
                        .as_deref()
                        .unwrap_or_default()
                        .trim()
                        .is_empty()
            })
            .cloned()
            .collect::<Vec<_>>();
        requests.sort_by_key(|field| (field.position, field.key.clone()));
        if requests.is_empty() {
            return Err(ApiError::BadRequest(
                "projection operation requires at least one field".into(),
            ));
        }
        let input_by_key = self
            .fields
            .iter()
            .map(|field| (field.key.as_str(), field.clone()))
            .collect::<HashMap<_, _>>();
        let mut seen_output_keys = BTreeSet::new();
        let mut seen_input_keys = BTreeSet::new();
        let mut output_fields = Vec::new();
        let mut select_fields = vec![
            quote_identifier("__row_id"),
            quote_identifier("__restriction_tier"),
            quote_identifier("__scope_node_ids"),
        ];

        for (index, request) in requests.into_iter().enumerate() {
            require_text("projection field key", &request.key)?;
            require_text("projection field label", &request.label)?;
            require_identifier("projection field key", &request.key)?;
            if internal_dataset_columns().contains(&request.key) {
                return Err(ApiError::BadRequest(format!(
                    "projection field key '{}' conflicts with an internal dataset column",
                    request.key
                )));
            }
            if !seen_output_keys.insert(request.key.clone()) {
                return Err(ApiError::BadRequest(format!(
                    "projection field key '{}' is duplicated",
                    request.key
                )));
            }
            let input_key = projection_input_field_key(&request)?;
            if !seen_input_keys.insert(input_key.clone()) {
                return Err(ApiError::BadRequest(format!(
                    "projection input field '{}' is duplicated",
                    input_key
                )));
            }
            let input = input_by_key.get(input_key.as_str()).ok_or_else(|| {
                ApiError::BadRequest(format!(
                    "projection field '{}' references unavailable field '{}'",
                    request.key, input_key
                ))
            })?;
            select_fields.push(format!(
                "{} AS {}",
                quote_identifier(&input.key),
                quote_identifier(&request.key)
            ));
            output_fields.push(ValidatedDatasetField {
                id: input.id,
                key: request.key,
                label: request.label,
                source_alias: input.source_alias.clone(),
                source_field_key: input.source_field_key.clone(),
                source_field_id: input.source_field_id,
                field_type: input.field_type.clone(),
                position: index as i32,
            });
        }

        let cte_name = self.next_cte_name("projection")?;
        let select_list = select_fields.join(",\n                ");
        self.ctes.push(format!(
            r#"{cte_name} AS (
            SELECT
                {select_list}
            FROM {}
        )"#,
            quote_identifier(&self.current_cte)
        ));
        self.current_cte = cte_name;
        self.fields = output_fields;
        Ok(())
    }

    fn apply_aggregation(
        &mut self,
        group_fields: &[String],
        metrics: &[DatasetAggregationMetricRequest],
        row_picker: &Option<DatasetRowPickerRequest>,
    ) -> ApiResult<()> {
        let request = DatasetAggregationRequest {
            group_fields: group_fields.to_vec(),
            metrics: metrics.to_vec(),
            row_picker: row_picker.clone(),
        };
        let aggregation = validate_dataset_aggregation(Some(request), &self.fields)?;
        let cte_name = self.next_cte_name("aggregation")?;
        let sql_body = aggregation
            .as_ref()
            .map(|aggregation| {
                aggregation_sql_from_source(&self.current_cte, &self.fields, aggregation)
            })
            .unwrap_or_else(|| format!("SELECT * FROM {}", quote_identifier(&self.current_cte)));
        self.ctes.push(format!(
            r#"{cte_name} AS (
            {sql_body}
        )"#
        ));
        if let Some(aggregation) = aggregation.as_ref() {
            self.fields = fields_after_aggregation(&self.fields, Some(aggregation));
        }
        self.current_cte = cte_name;
        Ok(())
    }

    fn apply_calculated_fields(
        &mut self,
        requests: &[DatasetCalculatedFieldRequest],
    ) -> ApiResult<()> {
        let calculated = validate_dataset_calculated_fields(requests.to_vec(), &self.fields)?;
        let cte_name = self.next_cte_name("calculated_fields")?;
        let base_selects = self
            .fields
            .iter()
            .map(|field| quote_identifier(&field.key))
            .collect::<Vec<_>>();
        let calculated_selects = calculated
            .iter()
            .map(calculated_field_sql)
            .collect::<Vec<_>>();
        let select_list = ["__row_id", "__restriction_tier", "__scope_node_ids"]
            .into_iter()
            .map(quote_identifier)
            .chain(base_selects)
            .chain(calculated_selects)
            .collect::<Vec<_>>()
            .join(",\n                ");
        self.ctes.push(format!(
            r#"{cte_name} AS (
            SELECT
                {select_list}
            FROM {}
        )"#,
            quote_identifier(&self.current_cte)
        ));
        let mut calculated_output_fields = calculated_fields_for_dataset(&calculated);
        let offset = self.fields.len() as i32;
        for (index, field) in calculated_output_fields.iter_mut().enumerate() {
            field.position = offset + index as i32;
        }
        self.fields.extend(calculated_output_fields);
        self.current_cte = cte_name;
        Ok(())
    }

    fn apply_filter(&mut self, requests: &[DatasetRowFilterRequest]) -> ApiResult<()> {
        let filters = validate_dataset_row_filters(requests.to_vec(), &self.fields)?;
        let cte_name = self.next_cte_name("filtered_fields")?;
        let where_clause = row_filters_sql(&filters);
        self.ctes.push(format!(
            r#"{cte_name} AS (
            SELECT
                *
            FROM {}{where_clause}
        )"#,
            quote_identifier(&self.current_cte)
        ));
        self.current_cte = cte_name;
        Ok(())
    }

    fn final_sql(&self, restriction_policy: &ValidatedRestrictionPolicy) -> String {
        let field_selects = self.fields.iter().map(|field| quote_identifier(&field.key));
        let final_selects = ["__row_id".to_string()]
            .into_iter()
            .chain(std::iter::once(format!(
                "{} AS {}",
                effective_restriction_tier_sql(restriction_policy),
                quote_identifier("__restriction_tier")
            )))
            .chain(std::iter::once(quote_identifier("__scope_node_ids")))
            .chain(field_selects)
            .collect::<Vec<_>>()
            .join(",\n            ");

        format!(
            r#"WITH
        {}
        SELECT
            {final_selects}
        FROM {}"#,
            self.ctes.join(",\n        "),
            quote_identifier(&self.current_cte)
        )
    }
}

fn validate_operation_position(operation: &DatasetOperationRequest, index: usize) -> ApiResult<()> {
    let expected = index as i32;
    let actual = match operation {
        DatasetOperationRequest::AddSource { position, .. }
        | DatasetOperationRequest::Projection { position, .. }
        | DatasetOperationRequest::Aggregation { position, .. }
        | DatasetOperationRequest::CalculatedFields { position, .. }
        | DatasetOperationRequest::Filter { position, .. } => *position,
    };
    if actual != expected {
        return Err(ApiError::BadRequest(format!(
            "dataset operation at array index {index} has position {actual}; expected {expected}"
        )));
    }
    Ok(())
}
fn projection_input_field_key(request: &DatasetProjectionFieldRequest) -> ApiResult<String> {
    if let Some(input_field_key) = request
        .input_field_key
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        return Ok(input_field_key.to_string());
    }
    require_text("projection input field", &request.key)?;
    Ok(request.key.clone())
}

fn canonical_source_column_key(source_alias: &str, source_field_key: &str) -> String {
    format!(
        "{source_alias}__{}",
        source_field_key.trim_start_matches('_')
    )
}

fn catalog_columns(fields: &[ValidatedDatasetField]) -> BTreeSet<String> {
    let mut columns = internal_dataset_columns();
    columns.extend(fields.iter().map(|field| field.key.clone()));
    columns
}

fn merge_field_catalogs(
    left: &[ValidatedDatasetField],
    right: &[ValidatedDatasetField],
) -> ApiResult<Vec<ValidatedDatasetField>> {
    let mut output = left.to_vec();
    let mut index_by_key = output
        .iter()
        .enumerate()
        .map(|(index, field)| (field.key.clone(), index))
        .collect::<HashMap<_, _>>();
    for field in right {
        if let Some(index) = index_by_key.get(&field.key).copied() {
            let existing = &output[index];
            if existing.field_type != field.field_type {
                return Err(ApiError::BadRequest(format!(
                    "field '{}' has incompatible types '{}' and '{}'",
                    field.key, existing.field_type, field.field_type
                )));
            }
            continue;
        }
        index_by_key.insert(field.key.clone(), output.len());
        output.push(field.clone());
    }
    for (index, field) in output.iter_mut().enumerate() {
        field.position = index as i32;
    }
    Ok(output)
}

fn union_field_catalog(
    left: &[ValidatedDatasetField],
    right: &[ValidatedDatasetField],
    union_alias: &str,
) -> ApiResult<(Vec<ValidatedDatasetField>, Vec<UnionColumnMapping>)> {
    let mut output = left.to_vec();
    let mut mappings = vec![
        UnionColumnMapping {
            output_key: "__row_id".into(),
            left_key: Some("__row_id".into()),
            right_key: Some("__row_id".into()),
        },
        UnionColumnMapping {
            output_key: "__restriction_tier".into(),
            left_key: Some("__restriction_tier".into()),
            right_key: Some("__restriction_tier".into()),
        },
        UnionColumnMapping {
            output_key: "__scope_node_ids".into(),
            left_key: Some("__scope_node_ids".into()),
            right_key: Some("__scope_node_ids".into()),
        },
    ];
    mappings.extend(left.iter().map(|field| UnionColumnMapping {
        output_key: field.key.clone(),
        left_key: Some(field.key.clone()),
        right_key: None,
    }));

    let mut index_by_key = output
        .iter()
        .enumerate()
        .map(|(index, field)| (field.key.clone(), index))
        .collect::<HashMap<_, _>>();
    let mut input_counts = HashMap::<String, usize>::new();
    for field in &output {
        *input_counts
            .entry(field.source_field_key.clone())
            .or_default() += 1;
    }
    let index_by_unique_input = output
        .iter()
        .enumerate()
        .filter(|(_, field)| input_counts.get(&field.source_field_key) == Some(&1))
        .map(|(index, field)| (field.source_field_key.clone(), index))
        .collect::<HashMap<_, _>>();

    for field in right {
        let matching_index = index_by_key
            .get(&field.key)
            .copied()
            .or_else(|| index_by_unique_input.get(&field.source_field_key).copied());
        if let Some(index) = matching_index {
            let existing_key = output[index].key.clone();
            if output[index].field_type != field.field_type {
                return Err(ApiError::BadRequest(format!(
                    "union field '{}' has incompatible types '{}' and '{}'",
                    field.source_field_key, output[index].field_type, field.field_type
                )));
            }
            let output_key =
                canonical_source_column_key(union_alias, &output[index].source_field_key);
            if let Some(mapping) = mappings
                .iter_mut()
                .find(|mapping| mapping.output_key == existing_key)
            {
                mapping.output_key = output_key.clone();
                mapping.right_key = Some(field.key.clone());
            }
            index_by_key.remove(&existing_key);
            output[index].key = output_key.clone();
            output[index].source_alias = union_alias.to_string();
            index_by_key.insert(output_key, index);
            continue;
        }

        index_by_key.insert(field.key.clone(), output.len());
        mappings.push(UnionColumnMapping {
            output_key: field.key.clone(),
            left_key: None,
            right_key: Some(field.key.clone()),
        });
        output.push(field.clone());
    }

    for (index, field) in output.iter_mut().enumerate() {
        field.position = index as i32;
    }
    Ok((output, mappings))
}

fn union_output_alias(operation_position: i32) -> String {
    format!("union_{}", operation_position.saturating_add(1))
}

fn load_form_source_catalog(
    source: &ValidatedDatasetSource,
    schema: &FormVersionSchemaResponse,
) -> ApiResult<Vec<ValidatedDatasetField>> {
    let mut fields = system_source_field_keys()
        .into_iter()
        .enumerate()
        .map(|(index, key)| ValidatedDatasetField {
            id: None,
            key: canonical_source_column_key(&source.source_alias, key),
            label: system_source_field_label(key).to_string(),
            source_alias: source.source_alias.clone(),
            source_field_key: key.to_string(),
            source_field_id: None,
            field_type: system_source_field_type(key).unwrap_or("text").to_string(),
            position: index as i32,
        })
        .collect::<Vec<_>>();

    let offset = fields.len() as i32;
    for (index, field) in schema.fields.iter().enumerate() {
        let source_field_key = field.key.clone();
        fields.push(ValidatedDatasetField {
            id: None,
            key: canonical_source_column_key(&source.source_alias, &source_field_key),
            label: field.label.clone(),
            source_alias: source.source_alias.clone(),
            source_field_key,
            source_field_id: Some(field.field_id),
            field_type: field.field_type.clone(),
            position: offset + index as i32,
        });
    }
    Ok(fields)
}

fn validate_form_version_schema(
    wire: serde_json::Value,
    expected_form_id: Uuid,
    expected_form_version_id: Uuid,
    requested_scope_node_ids: &BTreeSet<Uuid>,
    authorized_scope_node_ids: &BTreeSet<Uuid>,
) -> ApiResult<FormVersionSchemaResponse> {
    let schema: FormVersionSchemaResponse =
        serde_json::from_value(wire).map_err(|_| form_schema_dependency_incompatible())?;
    schema
        .validate_for(expected_form_version_id)
        .map_err(|_| form_schema_dependency_incompatible())?;
    if schema.form_id != expected_form_id
        || schema
            .source_scope_node_ids
            .iter()
            .any(|node_id| !authorized_scope_node_ids.contains(node_id))
    {
        return Err(form_schema_dependency_incompatible());
    }
    if schema
        .source_scope_node_ids
        .iter()
        .any(|node_id| !requested_scope_node_ids.contains(node_id))
    {
        return Err(ApiError::BadRequest(
            "Form source scope must be fully contained in the Dataset visibility scope".into(),
        ));
    }
    Ok(schema)
}

fn validated_form_source(
    dataset_id: Option<Uuid>,
    alias: &str,
    form_id: Uuid,
    form_version_id: Uuid,
    position: i32,
    schema: &FormVersionSchemaResponse,
) -> ValidatedDatasetSource {
    ValidatedDatasetSource {
        source_binding_id: Some(source_binding_uuid(
            dataset_id,
            alias,
            "response_export",
            &form_version_id.to_string(),
        )),
        reference: DatasetSourceRequest::Form {
            alias: alias.to_owned(),
            form_id: form_id.to_string(),
            form_version_id: form_version_id.to_string(),
        },
        source_alias: alias.to_owned(),
        form_version_id: Some(form_version_id),
        source_name: schema.form_name.clone(),
        source_slug: Some(schema.form_slug.clone()),
        source_version_label: schema.version_label.clone(),
        source_scope_node_ids: schema.source_scope_node_ids.clone(),
        source_scope_revision: schema.source_scope_revision.clone(),
        source_scope_digest: schema.source_scope_digest.clone(),
        source_content_revision: schema.content_revision.clone(),
        source_content_digest: schema.content_digest.clone(),
        position,
    }
}

fn form_schema_dependency_incompatible() -> ApiError {
    ApiError::Module(DatasetModuleError::DependencyIncompatible(
        "FormVersion schema dependency is incompatible".into(),
    ))
}

fn dataset_source_catalog_from_output_fields(
    source: &ValidatedDatasetSource,
    output_fields: serde_json::Value,
) -> ApiResult<Vec<ValidatedDatasetField>> {
    let fields: Vec<DatasetProductFieldV1> =
        serde_json::from_value(output_fields).map_err(|error| {
            ApiError::Internal(anyhow::anyhow!(
                "stored Dataset revision fields are invalid: {error}"
            ))
        })?;
    Ok(fields
        .into_iter()
        .enumerate()
        .map(|(index, field)| ValidatedDatasetField {
            id: None,
            key: canonical_source_column_key(&source.source_alias, &field.key),
            label: field.label,
            source_alias: source.source_alias.clone(),
            source_field_key: field.key,
            source_field_id: None,
            field_type: field.field_type,
            position: index as i32,
        })
        .collect())
}

fn aggregation_sql_from_source(
    source_cte: &str,
    fields: &[ValidatedDatasetField],
    aggregation: &ValidatedAggregation,
) -> String {
    let group_selects = aggregation
        .group_fields
        .iter()
        .map(|field| {
            let quoted = quote_identifier(field);
            format!("{quoted} AS {quoted}")
        })
        .collect::<Vec<_>>();
    let metric_selects = aggregation
        .metrics
        .iter()
        .map(aggregation_metric_sql)
        .collect::<Vec<_>>();
    let row_pick_selects = aggregation
        .row_picker
        .as_ref()
        .map(|_| {
            fields
                .iter()
                .filter(|field| !aggregation.group_fields.contains(&field.key))
                .map(|field| {
                    let quoted = quote_identifier(&field.key);
                    format!("MAX({quoted}) FILTER (WHERE __pick_rank = 1) AS {quoted}")
                })
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    let output_selects = group_selects
        .into_iter()
        .chain(row_pick_selects)
        .chain(metric_selects)
        .collect::<Vec<_>>();
    let field_select = if output_selects.is_empty() {
        "COUNT(*)::text AS row_count".to_string()
    } else {
        output_selects.join(",\n            ")
    };
    let restriction_tier_select = format!(
        "{} AS {}",
        max_restriction_tier_sql("__restriction_tier"),
        quote_identifier("__restriction_tier")
    );
    let scope_node_ids_select = format!(
        "dataset_scope_union({}) AS {}",
        quote_identifier("__scope_node_ids"),
        quote_identifier("__scope_node_ids")
    );
    let group_columns = aggregation
        .group_fields
        .iter()
        .map(|field| quote_identifier(field))
        .collect::<Vec<_>>();
    let all_group_columns = group_columns;
    let group_by = if all_group_columns.is_empty() {
        String::new()
    } else {
        format!("\n        GROUP BY {}", all_group_columns.join(", "))
    };
    let row_id = if all_group_columns.is_empty() {
        "'aggregate'::text".to_string()
    } else {
        format!(
            "md5(concat_ws('|', {}))",
            all_group_columns
                .iter()
                .map(|column| format!("{column}::text"))
                .collect::<Vec<_>>()
                .join(", ")
        )
    };
    let source = if let Some(row_picker) = &aggregation.row_picker {
        let partition = if all_group_columns.is_empty() {
            String::new()
        } else {
            format!("PARTITION BY {}", all_group_columns.join(", "))
        };
        let order_by = row_picker
            .sort_fields
            .iter()
            .map(|sort| {
                let direction = if row_picker.direction == "highest" {
                    "DESC"
                } else {
                    "ASC"
                };
                let expression =
                    typed_orderable_sql(&quote_identifier(&sort.field_key), &sort.field_type);
                format!("{expression} {direction} NULLS LAST")
            })
            .chain(std::iter::once("__row_id".to_string()))
            .collect::<Vec<_>>()
            .join(", ");
        format!(
            r#"(SELECT
                {}.*,
                ROW_NUMBER() OVER (
                    {partition}
                    ORDER BY {order_by}
                ) AS __pick_rank
            FROM {})"#,
            quote_identifier(source_cte),
            quote_identifier(source_cte)
        )
    } else {
        quote_identifier(source_cte)
    };
    format!(
        r#"SELECT
            {row_id} AS __row_id,
            {restriction_tier_select},
            {scope_node_ids_select},
            {field_select}
        FROM {source}{group_by}"#
    )
}

fn row_filters_sql(filters: &[ValidatedRowFilter]) -> String {
    if filters.is_empty() {
        return String::new();
    }
    let predicates = filters
        .iter()
        .map(row_filter_sql)
        .collect::<Vec<_>>()
        .join("\n              AND ");
    format!("\n            WHERE {predicates}")
}

fn validate_dataset_row_filters(
    mut requests: Vec<DatasetRowFilterRequest>,
    fields: &[ValidatedDatasetField],
) -> ApiResult<Vec<ValidatedRowFilter>> {
    requests
        .retain(|filter| !filter.field_key.trim().is_empty() || !filter.operator.trim().is_empty());
    requests.sort_by_key(|filter| (filter.position, filter.field_key.clone()));
    let field_by_key = fields
        .iter()
        .map(|field| (field.key.as_str(), field))
        .collect::<HashMap<_, _>>();
    let mut filters = Vec::new();
    for filter in requests {
        require_text("row filter field", &filter.field_key)?;
        require_text("row filter operator", &filter.operator)?;
        let field = field_by_key.get(filter.field_key.as_str()).ok_or_else(|| {
            ApiError::BadRequest(format!(
                "row filter field '{}' is not projected",
                filter.field_key
            ))
        })?;
        let operator = FilterOperator::parse(&filter.operator).map_err(data_op_error)?;
        operator
            .validate_for_field(&data_field_from_validated(field))
            .map_err(data_op_error)?;
        let value_mode = filter.value_mode.trim().to_string();
        if !matches!(value_mode.as_str(), "value" | "field") {
            return Err(ApiError::BadRequest(format!(
                "row filter on '{}' has unsupported value mode '{}'",
                filter.field_key, filter.value_mode
            )));
        }
        if matches!(
            operator,
            FilterOperator::Between | FilterOperator::NotBetween
        ) && value_mode == "field"
        {
            return Err(ApiError::BadRequest(format!(
                "row filter on '{}' does not support field value mode for operator '{}'",
                filter.field_key,
                operator.as_str()
            )));
        }
        let value_field_key = if value_mode == "field" {
            Some(
                filter
                    .value_field_key
                    .map(|key| key.trim().to_string())
                    .filter(|key| !key.is_empty())
                    .ok_or_else(|| {
                        ApiError::BadRequest(format!(
                            "row filter on '{}' requires a value field",
                            filter.field_key
                        ))
                    })?,
            )
        } else {
            None
        };
        if let Some(value_field_key) = &value_field_key {
            let value_field = field_by_key.get(value_field_key.as_str()).ok_or_else(|| {
                ApiError::BadRequest(format!(
                    "row filter value field '{value_field_key}' is not projected"
                ))
            })?;
            if value_field.field_type != field.field_type {
                return Err(ApiError::BadRequest(format!(
                    "row filter value field '{value_field_key}' has type '{}' but '{}' has type '{}'",
                    value_field.field_type, filter.field_key, field.field_type
                )));
            }
        }
        let value = if value_mode == "value" {
            filter.value.map(|value| value.trim().to_string())
        } else {
            None
        };
        if operator.requires_value()
            && value_mode == "value"
            && value.as_deref().unwrap_or_default().is_empty()
        {
            return Err(ApiError::BadRequest(format!(
                "row filter on '{}' requires a value",
                filter.field_key
            )));
        }
        if matches!(
            operator,
            FilterOperator::Between | FilterOperator::NotBetween
        ) {
            let (lower, upper) = split_between_value(value.as_deref().unwrap_or_default())
                .ok_or_else(|| {
                    ApiError::BadRequest(format!(
                        "row filter on '{}' requires two values for operator '{}'",
                        filter.field_key,
                        operator.as_str()
                    ))
                })?;
            validate_filter_literal(&field.field_type, &lower)?;
            validate_filter_literal(&field.field_type, &upper)?;
        } else if operator.requires_value() && value_mode == "value" {
            validate_filter_literal(&field.field_type, value.as_deref().unwrap_or_default())?;
        }
        filters.push(ValidatedRowFilter {
            field_key: filter.field_key,
            field_type: field.field_type.clone(),
            operator,
            value,
            value_field_key,
        });
    }
    Ok(filters)
}

fn split_between_value(value: &str) -> Option<(String, String)> {
    value
        .split_once("..")
        .or_else(|| value.split_once(','))
        .map(|(lower, upper)| (lower.trim().to_string(), upper.trim().to_string()))
        .filter(|(lower, upper)| !lower.is_empty() && !upper.is_empty())
}

fn validate_filter_literal(field_type: &str, value: &str) -> ApiResult<()> {
    validate_typed_literal("row filter", field_type, value)
}

fn validate_typed_literal(label: &str, field_type: &str, value: &str) -> ApiResult<()> {
    match field_type {
        "number" => {
            value.parse::<f64>().map_err(|_| {
                ApiError::BadRequest(format!(
                    "{label} requires a numeric value for field type '{field_type}'"
                ))
            })?;
        }
        "date" => validate_date_literal(label, value)?,
        "datetime" | "timestamp" => validate_datetime_literal(label, value)?,
        "boolean" => validate_boolean_literal(label, value)?,
        _ => {}
    }
    Ok(())
}

fn validate_boolean_literal(label: &str, value: &str) -> ApiResult<()> {
    if matches!(
        value.trim().to_ascii_lowercase().as_str(),
        "true" | "false" | "t" | "f" | "1" | "0" | "yes" | "no" | "y" | "n"
    ) {
        Ok(())
    } else {
        Err(ApiError::BadRequest(format!(
            "{label} requires a boolean value"
        )))
    }
}

fn validate_dataset_calculated_fields(
    mut requests: Vec<DatasetCalculatedFieldRequest>,
    fields: &[ValidatedDatasetField],
) -> ApiResult<Vec<ValidatedCalculatedField>> {
    requests
        .retain(|field| !field.key.trim().is_empty() || !field.base_field_key.trim().is_empty());
    requests.sort_by_key(|field| (field.position, field.key.clone()));
    let field_by_key = fields
        .iter()
        .map(|field| (field.key.as_str(), field.clone()))
        .collect::<HashMap<_, _>>();
    let mut seen_keys = field_by_key
        .keys()
        .map(|key| (*key).to_string())
        .collect::<BTreeSet<_>>();
    let mut calculated = Vec::new();
    for request in requests {
        require_text("calculated field key", &request.key)?;
        require_text("calculated field label", &request.label)?;
        require_text("calculated field base field", &request.base_field_key)?;
        require_identifier("calculated field key", &request.key)?;
        if !seen_keys.insert(request.key.clone()) {
            return Err(ApiError::BadRequest(format!(
                "calculated field key '{}' conflicts with an existing field",
                request.key
            )));
        }
        let base_field = field_by_key
            .get(request.base_field_key.as_str())
            .cloned()
            .ok_or_else(|| {
                ApiError::BadRequest(format!(
                    "calculated field '{}' references unknown base field '{}'",
                    request.key, request.base_field_key
                ))
            })?;
        let mut field_type = base_field.field_type.clone();
        let mut functions = request.functions.clone();
        functions.sort_by_key(|function| (function.position, function.function.clone()));
        let mut validated_functions = Vec::new();
        for function in functions {
            let calculation_function = CalculationFunction::parse(&function.function)?;
            let argument_field_key = match function.argument_mode.as_str() {
                "value" => None,
                "field" => {
                    let key = function
                        .argument_field_key
                        .clone()
                        .filter(|key| !key.trim().is_empty())
                        .ok_or_else(|| {
                            ApiError::BadRequest(format!(
                                "calculated field '{}' function requires an argument field",
                                request.key
                            ))
                        })?;
                    let argument_field = field_by_key.get(key.as_str()).ok_or_else(|| {
                        ApiError::BadRequest(format!(
                            "calculated field '{}' references unknown argument field '{}'",
                            request.key, key
                        ))
                    })?;
                    calculation_function.validate_argument_field(
                        &argument_field.field_type,
                        &field_type,
                        &request.key,
                    )?;
                    Some(key)
                }
                other => {
                    return Err(ApiError::BadRequest(format!(
                        "calculated field '{}' has unsupported argument mode '{}'",
                        request.key, other
                    )));
                }
            };
            if argument_field_key.is_none() {
                calculation_function.validate_argument(
                    &function.argument,
                    &field_type,
                    &request.key,
                )?;
            } else if !calculation_function.accepts_field_argument() {
                return Err(ApiError::BadRequest(format!(
                    "calculated field '{}' function does not accept a field argument",
                    request.key
                )));
            }
            let input_type = field_type.clone();
            field_type = calculation_function.output_field_type(&field_type, &request.key)?;
            validated_functions.push(ValidatedCalculationFunction {
                function: calculation_function,
                argument: function.argument,
                argument_field_key,
                input_type,
            });
        }
        let validated = ValidatedCalculatedField {
            key: request.key.clone(),
            label: request.label.clone(),
            base_field_key: request.base_field_key.clone(),
            functions: validated_functions,
            field_type: field_type.clone(),
        };
        calculated.push(validated);
    }
    Ok(calculated)
}

fn calculated_fields_for_dataset(
    calculated: &[ValidatedCalculatedField],
) -> Vec<ValidatedDatasetField> {
    calculated
        .iter()
        .enumerate()
        .map(|(index, field)| ValidatedDatasetField {
            id: None,
            key: field.key.clone(),
            label: field.label.clone(),
            source_alias: "calculated".into(),
            source_field_key: field.base_field_key.clone(),
            source_field_id: None,
            field_type: field.field_type.clone(),
            position: i32::MAX - 10_000 + index as i32,
        })
        .collect()
}

fn validate_dataset_restriction_policy(
    request: Option<DatasetRestrictionPolicyRequest>,
    fields: &[ValidatedDatasetField],
) -> ApiResult<ValidatedRestrictionPolicy> {
    let Some(request) = request else {
        return Ok(ValidatedRestrictionPolicy::default());
    };
    let internal_field_key =
        validate_restriction_boolean_field("internal", request.internal_field_key, fields)?;
    let restricted_field_key =
        validate_restriction_boolean_field("restricted", request.restricted_field_key, fields)?;
    let confidential_field_key =
        validate_restriction_boolean_field("confidential", request.confidential_field_key, fields)?;
    Ok(ValidatedRestrictionPolicy {
        internal_field_key,
        restricted_field_key,
        confidential_field_key,
    })
}

fn validate_restriction_boolean_field(
    tier: &str,
    field_key: Option<String>,
    fields: &[ValidatedDatasetField],
) -> ApiResult<Option<String>> {
    let field_key = field_key
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty());
    let Some(field_key) = field_key else {
        return Ok(None);
    };
    let field = fields
        .iter()
        .find(|field| field.key == field_key)
        .ok_or_else(|| {
            ApiError::BadRequest(format!(
                "{tier} restriction field '{field_key}' is not projected"
            ))
        })?;
    if field.field_type != "boolean" {
        return Err(ApiError::BadRequest(format!(
            "{tier} restriction field '{}' must be boolean, got '{}'",
            field.key, field.field_type
        )));
    }
    Ok(Some(field_key))
}

fn calculated_field_sql(field: &ValidatedCalculatedField) -> String {
    let mut expression = quote_identifier(&field.base_field_key);
    for function in &field.functions {
        expression = calculation_function_sql(function, &expression);
    }
    format!("{} AS {}", expression, quote_identifier(&field.key))
}

fn calculation_function_sql(function: &ValidatedCalculationFunction, input: &str) -> String {
    let argument = function.argument.as_deref().unwrap_or_default();
    let argument_sql = calculation_argument_sql(function);
    match function.function {
        CalculationFunction::Trim => format!("BTRIM({input})"),
        CalculationFunction::Uppercase => format!("UPPER({input})"),
        CalculationFunction::Lowercase => format!("LOWER({input})"),
        CalculationFunction::Prefix => {
            format!("CONCAT({argument_sql}, COALESCE({input}, ''))")
        }
        CalculationFunction::Suffix | CalculationFunction::Concat => {
            format!("CONCAT(COALESCE({input}, ''), {argument_sql})")
        }
        CalculationFunction::Coalesce => {
            format!("COALESCE(NULLIF({input}, ''), {argument_sql})")
        }
        CalculationFunction::Constant => argument_sql,
        CalculationFunction::MapValue => {
            let branches = split_map_arguments(argument)
                .into_iter()
                .map(|(from, to)| {
                    format!(
                        "WHEN COALESCE({input}, '') = {} THEN {}",
                        sql_literal(&from),
                        sql_literal(&to)
                    )
                })
                .collect::<Vec<_>>()
                .join(" ");
            format!("CASE {branches} ELSE {input} END")
        }
        CalculationFunction::Add => format!(
            "(NULLIF({input}, '')::numeric + {})::text",
            numeric_operand_sql(function, argument)
        ),
        CalculationFunction::Subtract => {
            format!(
                "(NULLIF({input}, '')::numeric - {})::text",
                numeric_operand_sql(function, argument)
            )
        }
        CalculationFunction::Multiply => {
            format!(
                "(NULLIF({input}, '')::numeric * {})::text",
                numeric_operand_sql(function, argument)
            )
        }
        CalculationFunction::Divide => {
            format!(
                "(NULLIF({input}, '')::numeric / NULLIF({}, 0))::text",
                numeric_operand_sql(function, argument)
            )
        }
        CalculationFunction::Round => format!(
            "ROUND(NULLIF({input}, '')::numeric, {})::text",
            integer_literal(argument)
        ),
        CalculationFunction::FormatDate => {
            let typed_input = date_format_input_sql(input, &function.input_type);
            format!("TO_CHAR({typed_input}, {})", sql_literal(argument))
        }
        CalculationFunction::GreaterThan => comparison_function_sql(input, ">", function),
        CalculationFunction::GreaterThanOrEqual => comparison_function_sql(input, ">=", function),
        CalculationFunction::LessThan => comparison_function_sql(input, "<", function),
        CalculationFunction::LessThanOrEqual => comparison_function_sql(input, "<=", function),
        CalculationFunction::Equal => {
            let equality = equality_sql(input, &argument_sql, &function.input_type);
            format!("CASE WHEN {equality} THEN 'true' ELSE 'false' END")
        }
        CalculationFunction::NotEqual => {
            let equality = equality_sql(input, &argument_sql, &function.input_type);
            format!("CASE WHEN NOT ({equality}) THEN 'true' ELSE 'false' END")
        }
        CalculationFunction::IsEmpty => {
            format!("CASE WHEN NULLIF({input}, '') IS NULL THEN 'true' ELSE 'false' END")
        }
        CalculationFunction::IsNotEmpty => {
            format!("CASE WHEN NULLIF({input}, '') IS NOT NULL THEN 'true' ELSE 'false' END")
        }
        CalculationFunction::ToText => input.to_string(),
        CalculationFunction::ToNumber => format!("NULLIF({input}, '')::numeric::text"),
        CalculationFunction::ToBoolean => {
            format!(
                "CASE WHEN {} THEN 'true' ELSE 'false' END",
                boolean_expression_sql(input)
            )
        }
        CalculationFunction::ToDate => format!("NULLIF({input}, '')::date::text"),
    }
}

fn calculation_argument_sql(function: &ValidatedCalculationFunction) -> String {
    function
        .argument_field_key
        .as_deref()
        .map(quote_identifier)
        .unwrap_or_else(|| sql_literal(function.argument.as_deref().unwrap_or_default()))
}

fn numeric_operand_sql(function: &ValidatedCalculationFunction, argument: &str) -> String {
    function
        .argument_field_key
        .as_deref()
        .map(|field_key| format!("NULLIF({}, '')::numeric", quote_identifier(field_key)))
        .unwrap_or_else(|| numeric_literal(argument))
}

fn comparison_function_sql(
    input: &str,
    operator: &str,
    function: &ValidatedCalculationFunction,
) -> String {
    let argument_sql = calculation_argument_sql(function);
    let predicate = comparison_sql(input, operator, &argument_sql, &function.input_type);
    format!("CASE WHEN {predicate} THEN 'true' ELSE 'false' END")
}

fn comparison_sql(left: &str, operator: &str, right: &str, field_type: &str) -> String {
    typed_comparable_sql(left, field_type)
        .zip(typed_comparable_sql(right, field_type))
        .map(|(left, right)| format!("{left} {operator} {right}"))
        .unwrap_or_else(|| "FALSE /* unsupported comparison */".to_string())
}

fn equality_sql(left: &str, right: &str, field_type: &str) -> String {
    typed_comparable_sql(left, field_type)
        .zip(typed_comparable_sql(right, field_type))
        .map(|(left, right)| {
            format!("COALESCE({left} = {right}, {left} IS NULL AND {right} IS NULL)")
        })
        .unwrap_or_else(|| format!("COALESCE({left}, '') = COALESCE({right}, '')"))
}

fn typed_comparable_sql(expression: &str, field_type: &str) -> Option<String> {
    match field_type {
        "number" => Some(format!("NULLIF({expression}, '')::numeric")),
        "date" => Some(format!("NULLIF({expression}, '')::date")),
        "datetime" | "timestamp" => Some(format!("NULLIF({expression}, '')::timestamptz")),
        "boolean" => Some(nullable_boolean_expression_sql(expression)),
        _ => None,
    }
}

fn boolean_expression_sql(expression: &str) -> String {
    format!("LOWER(COALESCE({expression}, '')) IN ('true', 't', '1', 'yes', 'y')")
}

fn nullable_boolean_expression_sql(expression: &str) -> String {
    format!(
        "CASE WHEN NULLIF({expression}, '') IS NULL THEN NULL ELSE {} END",
        boolean_expression_sql(expression)
    )
}

fn typed_orderable_sql(expression: &str, field_type: &str) -> String {
    typed_comparable_sql(expression, field_type)
        .unwrap_or_else(|| format!("NULLIF({expression}, '')"))
}

fn date_format_input_sql(expression: &str, field_type: &str) -> String {
    match field_type {
        "date" => format!("NULLIF({expression}, '')::date"),
        "datetime" | "timestamp" => format!("NULLIF({expression}, '')::timestamptz"),
        _ => format!("NULLIF({expression}, '')::timestamptz"),
    }
}

impl CalculationFunction {
    fn parse(value: &str) -> ApiResult<Self> {
        match value {
            "trim" => Ok(Self::Trim),
            "uppercase" => Ok(Self::Uppercase),
            "lowercase" => Ok(Self::Lowercase),
            "prefix" => Ok(Self::Prefix),
            "suffix" => Ok(Self::Suffix),
            "concat" => Ok(Self::Concat),
            "coalesce" => Ok(Self::Coalesce),
            "constant" => Ok(Self::Constant),
            "map_value" => Ok(Self::MapValue),
            "add" => Ok(Self::Add),
            "subtract" => Ok(Self::Subtract),
            "multiply" => Ok(Self::Multiply),
            "divide" => Ok(Self::Divide),
            "round" => Ok(Self::Round),
            "format_date" => Ok(Self::FormatDate),
            "greater_than" => Ok(Self::GreaterThan),
            "greater_than_or_equal" => Ok(Self::GreaterThanOrEqual),
            "less_than" => Ok(Self::LessThan),
            "less_than_or_equal" => Ok(Self::LessThanOrEqual),
            "equal" => Ok(Self::Equal),
            "not_equal" => Ok(Self::NotEqual),
            "is_empty" => Ok(Self::IsEmpty),
            "is_not_empty" => Ok(Self::IsNotEmpty),
            "to_text" => Ok(Self::ToText),
            "to_number" => Ok(Self::ToNumber),
            "to_boolean" => Ok(Self::ToBoolean),
            "to_date" => Ok(Self::ToDate),
            other => Err(ApiError::BadRequest(format!(
                "unsupported calculation function '{other}'"
            ))),
        }
    }

    fn validate_argument(
        self,
        argument: &Option<String>,
        input_type: &str,
        field_key: &str,
    ) -> ApiResult<()> {
        let value = argument.as_deref().unwrap_or_default().trim();
        if matches!(self, Self::MapValue) && split_map_arguments(value).is_empty() {
            return Err(ApiError::BadRequest(format!(
                "calculated field '{field_key}' map function requires a from=>to argument"
            )));
        }
        match self {
            Self::Trim
            | Self::Uppercase
            | Self::Lowercase
            | Self::ToText
            | Self::ToNumber
            | Self::ToBoolean
            | Self::ToDate
            | Self::IsEmpty
            | Self::IsNotEmpty => {
                if value.is_empty() {
                    Ok(())
                } else {
                    Err(ApiError::BadRequest(format!(
                        "calculated field '{field_key}' function does not accept an argument"
                    )))
                }
            }
            Self::Add | Self::Subtract | Self::Multiply | Self::Divide => {
                value.parse::<f64>().map_err(|_| {
                    ApiError::BadRequest(format!(
                        "calculated field '{field_key}' requires a numeric argument"
                    ))
                })?;
                Ok(())
            }
            Self::GreaterThan
            | Self::GreaterThanOrEqual
            | Self::LessThan
            | Self::LessThanOrEqual => {
                if input_type == "number" {
                    value.parse::<f64>().map_err(|_| {
                        ApiError::BadRequest(format!(
                            "calculated field '{field_key}' requires a numeric argument"
                        ))
                    })?;
                    Ok(())
                } else if matches!(input_type, "date" | "datetime" | "timestamp") {
                    validate_typed_comparison_argument(input_type, value, field_key)
                } else {
                    Err(ApiError::BadRequest(format!(
                        "calculated field '{field_key}' comparison function requires a number or date input"
                    )))
                }
            }
            Self::Equal | Self::NotEqual => {
                if input_type == "number" {
                    value.parse::<f64>().map_err(|_| {
                        ApiError::BadRequest(format!(
                            "calculated field '{field_key}' requires a numeric argument"
                        ))
                    })?;
                    Ok(())
                } else if matches!(input_type, "date" | "datetime" | "timestamp") {
                    validate_typed_comparison_argument(input_type, value, field_key)
                } else if input_type == "boolean" {
                    validate_boolean_literal("calculated field", value)
                } else {
                    validate_non_empty_comparison_argument(value, field_key)
                }
            }
            Self::Round => {
                value.parse::<i32>().map_err(|_| {
                    ApiError::BadRequest(format!(
                        "calculated field '{field_key}' requires an integer argument"
                    ))
                })?;
                Ok(())
            }
            Self::Prefix | Self::Suffix | Self::Concat | Self::MapValue | Self::FormatDate => {
                if value.is_empty() {
                    Err(ApiError::BadRequest(format!(
                        "calculated field '{field_key}' function requires an argument"
                    )))
                } else {
                    Ok(())
                }
            }
            Self::Coalesce | Self::Constant => {
                if value.is_empty() {
                    Err(ApiError::BadRequest(format!(
                        "calculated field '{field_key}' function requires an argument"
                    )))
                } else {
                    validate_typed_literal("calculated field", input_type, value)
                }
            }
        }
    }

    fn accepts_field_argument(self) -> bool {
        matches!(
            self,
            Self::Prefix
                | Self::Suffix
                | Self::Concat
                | Self::Coalesce
                | Self::Constant
                | Self::Add
                | Self::Subtract
                | Self::Multiply
                | Self::Divide
                | Self::GreaterThan
                | Self::GreaterThanOrEqual
                | Self::LessThan
                | Self::LessThanOrEqual
                | Self::Equal
                | Self::NotEqual
        )
    }

    fn validate_argument_field(
        self,
        argument_type: &str,
        input_type: &str,
        field_key: &str,
    ) -> ApiResult<()> {
        if !self.accepts_field_argument() {
            return Err(ApiError::BadRequest(format!(
                "calculated field '{field_key}' function does not accept a field argument"
            )));
        }
        let valid = match self {
            Self::Add | Self::Subtract | Self::Multiply | Self::Divide => argument_type == "number",
            Self::GreaterThan
            | Self::GreaterThanOrEqual
            | Self::LessThan
            | Self::LessThanOrEqual => {
                argument_type == input_type && input_type == "number"
                    || date_like_type(input_type) && date_like_type(argument_type)
            }
            Self::Prefix | Self::Suffix | Self::Concat => {
                matches!(argument_type, "text" | "static_text")
            }
            Self::Equal | Self::NotEqual | Self::Coalesce | Self::Constant => {
                argument_type == input_type
                    || date_like_type(input_type) && date_like_type(argument_type)
            }
            _ => false,
        };
        if valid {
            Ok(())
        } else {
            Err(ApiError::BadRequest(format!(
                "calculated field '{field_key}' argument field type '{argument_type}' is not compatible with input type '{input_type}'"
            )))
        }
    }

    fn output_field_type(self, input_type: &str, field_key: &str) -> ApiResult<String> {
        match self {
            Self::Trim
            | Self::Uppercase
            | Self::Lowercase
            | Self::Prefix
            | Self::Suffix
            | Self::Concat
            | Self::MapValue => Ok("text".into()),
            Self::Coalesce | Self::Constant => Ok(input_type.into()),
            Self::Add | Self::Subtract | Self::Multiply | Self::Divide | Self::Round => {
                if input_type == "number" {
                    Ok("number".into())
                } else {
                    Err(ApiError::BadRequest(format!(
                        "calculated field '{field_key}' numeric function requires a number input"
                    )))
                }
            }
            Self::GreaterThan
            | Self::GreaterThanOrEqual
            | Self::LessThan
            | Self::LessThanOrEqual => {
                if matches!(input_type, "number" | "date" | "datetime" | "timestamp") {
                    Ok("boolean".into())
                } else {
                    Err(ApiError::BadRequest(format!(
                        "calculated field '{field_key}' comparison function requires a number or date input"
                    )))
                }
            }
            Self::Equal | Self::NotEqual | Self::IsEmpty | Self::IsNotEmpty => Ok("boolean".into()),
            Self::ToText => Ok("text".into()),
            Self::ToNumber => Ok("number".into()),
            Self::ToBoolean => Ok("boolean".into()),
            Self::ToDate => Ok("date".into()),
            Self::FormatDate => {
                if matches!(input_type, "date" | "datetime" | "timestamp") {
                    Ok("text".into())
                } else {
                    Err(ApiError::BadRequest(format!(
                        "calculated field '{field_key}' date formatting requires a date input"
                    )))
                }
            }
        }
    }
}

fn date_like_type(field_type: &str) -> bool {
    matches!(field_type, "date" | "datetime" | "timestamp")
}

fn validate_non_empty_comparison_argument(value: &str, field_key: &str) -> ApiResult<()> {
    if value.is_empty() {
        Err(ApiError::BadRequest(format!(
            "calculated field '{field_key}' comparison function requires an argument"
        )))
    } else {
        Ok(())
    }
}

fn validate_typed_comparison_argument(
    input_type: &str,
    value: &str,
    field_key: &str,
) -> ApiResult<()> {
    validate_non_empty_comparison_argument(value, field_key)?;
    match input_type {
        "date" => validate_date_literal("calculated field", value),
        "datetime" | "timestamp" => validate_datetime_literal("calculated field", value),
        _ => Ok(()),
    }
}

fn validate_date_literal(label: &str, value: &str) -> ApiResult<()> {
    chrono::NaiveDate::parse_from_str(value, "%Y-%m-%d")
        .map(|_| ())
        .map_err(|_| ApiError::BadRequest(format!("{label} requires a date value as YYYY-MM-DD")))
}

fn validate_datetime_literal(label: &str, value: &str) -> ApiResult<()> {
    if chrono::DateTime::parse_from_rfc3339(value).is_ok()
        || chrono::NaiveDateTime::parse_from_str(value, "%Y-%m-%dT%H:%M:%S").is_ok()
        || chrono::NaiveDateTime::parse_from_str(value, "%Y-%m-%dT%H:%M").is_ok()
    {
        Ok(())
    } else {
        Err(ApiError::BadRequest(format!(
            "{label} requires a datetime value as RFC3339 or YYYY-MM-DDTHH:MM"
        )))
    }
}

fn numeric_literal(value: &str) -> String {
    value
        .parse::<f64>()
        .expect("numeric literal should be validated before SQL generation")
        .to_string()
}

fn integer_literal(value: &str) -> String {
    value
        .parse::<i32>()
        .expect("integer literal should be validated before SQL generation")
        .to_string()
}

fn split_map_arguments(value: &str) -> Vec<(String, String)> {
    value
        .split(',')
        .filter_map(|entry| {
            entry
                .split_once("=>")
                .or_else(|| entry.split_once('='))
                .map(|(from, to)| (from.trim().to_string(), to.trim().to_string()))
        })
        .filter(|(from, to)| !from.is_empty() && !to.is_empty())
        .collect()
}

fn row_filter_sql(filter: &ValidatedRowFilter) -> String {
    let field = quote_identifier(&filter.field_key);
    let operand = filter_operand_sql(filter);
    match filter.operator {
        FilterOperator::Equals => equality_sql(&field, &operand, &filter.field_type),
        FilterOperator::NotEquals => {
            let equality = equality_sql(&field, &operand, &filter.field_type);
            format!("NOT ({equality})")
        }
        FilterOperator::Contains => format!(
            "POSITION(LOWER({}) IN LOWER(COALESCE({field}, ''))) > 0",
            operand
        ),
        FilterOperator::NotContains => format!(
            "POSITION(LOWER({}) IN LOWER(COALESCE({field}, ''))) = 0",
            operand
        ),
        FilterOperator::Gt => comparison_filter_sql(&field, ">", filter),
        FilterOperator::Gte => comparison_filter_sql(&field, ">=", filter),
        FilterOperator::Lt => comparison_filter_sql(&field, "<", filter),
        FilterOperator::Lte => comparison_filter_sql(&field, "<=", filter),
        FilterOperator::Between => between_filter_sql(&field, filter, false),
        FilterOperator::NotBetween => between_filter_sql(&field, filter, true),
        FilterOperator::IsEmpty => format!("NULLIF({field}, '') IS NULL"),
        FilterOperator::IsNotEmpty => format!("NULLIF({field}, '') IS NOT NULL"),
        FilterOperator::IsNull => format!("{field} IS NULL"),
        FilterOperator::IsNotNull => format!("{field} IS NOT NULL"),
    }
}

fn filter_operand_sql(filter: &ValidatedRowFilter) -> String {
    filter
        .value_field_key
        .as_deref()
        .map(quote_identifier)
        .unwrap_or_else(|| sql_literal(filter.value.as_deref().unwrap_or_default()))
}

fn comparison_filter_sql(field: &str, operator: &str, filter: &ValidatedRowFilter) -> String {
    let operand = filter_operand_sql(filter);
    comparison_sql(field, operator, &operand, &filter.field_type)
}

fn between_filter_sql(field: &str, filter: &ValidatedRowFilter, negated: bool) -> String {
    let (lower, upper) = split_between_value(filter.value.as_deref().unwrap_or_default())
        .expect("between literal should be validated before SQL generation");
    let lower = sql_literal(&lower);
    let upper = sql_literal(&upper);
    let lower_sql = comparison_sql(field, ">=", &lower, &filter.field_type);
    let upper_sql = comparison_sql(field, "<=", &upper, &filter.field_type);
    let predicate = format!("({lower_sql} AND {upper_sql})");
    if negated {
        format!("NOT {predicate}")
    } else {
        predicate
    }
}

fn validate_dataset_aggregation(
    request: Option<DatasetAggregationRequest>,
    fields: &[ValidatedDatasetField],
) -> ApiResult<Option<ValidatedAggregation>> {
    let Some(request) = request else {
        return Ok(None);
    };
    let active = !request.group_fields.is_empty()
        || !request.metrics.is_empty()
        || request.row_picker.is_some();
    if !active {
        return Ok(None);
    }
    let field_by_key = fields
        .iter()
        .map(|field| (field.key.as_str(), field))
        .collect::<HashMap<_, _>>();
    let plan = AggregationPlan {
        group_fields: request.group_fields,
        metrics: request
            .metrics
            .into_iter()
            .map(|metric| {
                Ok(AggregateMetric {
                    key: metric.key,
                    label: metric.label,
                    function: AggregateFunction::parse(&metric.function).map_err(data_op_error)?,
                    source_field_key: metric.source_field_key,
                    position: metric.position,
                })
            })
            .collect::<ApiResult<Vec<_>>>()?,
    };
    let data_fields = fields
        .iter()
        .map(data_field_from_validated)
        .collect::<Vec<_>>();
    let validated_plan = validate_aggregation_plan(plan, &data_fields).map_err(data_op_error)?;
    let group_fields = validated_plan.group_fields;
    let metrics = validated_plan
        .metrics
        .into_iter()
        .map(|metric| ValidatedAggregationMetric {
            key: metric.key,
            label: metric.label,
            function: metric.function,
            source_field_key: metric.source_field_key,
            field_type: metric.output_field_type.as_str().to_string(),
            position: metric.position,
        })
        .collect::<Vec<_>>();
    let row_picker = request
        .row_picker
        .map(|mut row_picker| {
            if row_picker.sort_fields.is_empty() {
                return Err(ApiError::BadRequest(
                    "row picker requires at least one sort field".into(),
                ));
            }
            row_picker
                .sort_fields
                .sort_by_key(|sort| (sort.position, sort.field_key.clone()));
            if !matches!(row_picker.direction.as_str(), "lowest" | "highest") {
                return Err(ApiError::BadRequest(
                    "row picker direction must be 'lowest' or 'highest'".into(),
                ));
            }
            let mut seen_sort_fields = BTreeSet::new();
            let mut sort_fields = Vec::new();
            for sort in &row_picker.sort_fields {
                require_text("row picker sort field", &sort.field_key)?;
                let field = field_by_key.get(sort.field_key.as_str()).ok_or_else(|| {
                    ApiError::BadRequest(format!(
                        "row picker sort field '{}' is not projected",
                        sort.field_key
                    ))
                })?;
                if !seen_sort_fields.insert(sort.field_key.clone()) {
                    return Err(ApiError::BadRequest(format!(
                        "row picker sort field '{}' is duplicated",
                        sort.field_key
                    )));
                }
                sort_fields.push(ValidatedRowPickerSort {
                    field_key: sort.field_key.clone(),
                    field_type: field.field_type.clone(),
                });
            }
            Ok(ValidatedRowPicker {
                sort_fields,
                direction: row_picker.direction,
            })
        })
        .transpose()?;
    Ok(Some(ValidatedAggregation {
        group_fields,
        metrics,
        row_picker,
    }))
}

fn fields_after_aggregation(
    fields: &[ValidatedDatasetField],
    aggregation: Option<&ValidatedAggregation>,
) -> Vec<ValidatedDatasetField> {
    let Some(aggregation) = aggregation else {
        return fields.to_vec();
    };
    let field_by_key = fields
        .iter()
        .map(|field| (field.key.as_str(), field))
        .collect::<HashMap<_, _>>();
    let mut output = Vec::new();
    let mut seen = BTreeSet::new();
    for key in &aggregation.group_fields {
        if let Some(field) = field_by_key.get(key.as_str())
            && seen.insert(field.key.clone())
        {
            output.push((*field).clone());
        }
    }
    if aggregation.row_picker.is_some() {
        for field in fields {
            if seen.insert(field.key.clone()) {
                output.push(field.clone());
            }
        }
    }
    let metric_offset = output.len() as i32;
    let mut metrics = aggregation.metrics.clone();
    metrics.sort_by_key(|metric| (metric.position, metric.key.clone()));
    for (index, metric) in metrics.into_iter().enumerate() {
        output.push(ValidatedDatasetField {
            id: None,
            key: metric.key,
            label: metric.label,
            source_alias: "aggregation".into(),
            source_field_key: metric.source_field_key.unwrap_or_default(),
            source_field_id: None,
            field_type: metric.field_type,
            position: metric_offset + index as i32,
        });
    }
    for (index, field) in output.iter_mut().enumerate() {
        field.position = index as i32;
    }
    output
}

fn require_dataset_output_fields(fields: &[ValidatedDatasetField]) -> ApiResult<()> {
    if fields.is_empty() {
        Err(ApiError::BadRequest(
            "dataset output requires at least one field".into(),
        ))
    } else {
        Ok(())
    }
}

fn aggregation_metric_sql(metric: &ValidatedAggregationMetric) -> String {
    let key = quote_identifier(&metric.key);
    match metric.function {
        AggregateFunction::Count => format!("COUNT(*)::text AS {key}"),
        AggregateFunction::CountValues => format!(
            "COUNT(NULLIF({}, ''))::text AS {key}",
            quote_identifier(metric.source_field_key.as_deref().unwrap_or_default())
        ),
        AggregateFunction::CountDistinct => format!(
            "COUNT(DISTINCT NULLIF({}, ''))::text AS {key}",
            quote_identifier(metric.source_field_key.as_deref().unwrap_or_default())
        ),
        AggregateFunction::Sum => format!(
            "SUM(NULLIF({}, '')::numeric)::text AS {key}",
            quote_identifier(metric.source_field_key.as_deref().unwrap_or_default())
        ),
        AggregateFunction::Avg => format!(
            "AVG(NULLIF({}, '')::numeric)::text AS {key}",
            quote_identifier(metric.source_field_key.as_deref().unwrap_or_default())
        ),
        AggregateFunction::Min => format!(
            "MIN({})::text AS {key}",
            aggregation_source_value_sql(metric)
        ),
        AggregateFunction::Max => format!(
            "MAX({})::text AS {key}",
            aggregation_source_value_sql(metric)
        ),
    }
}

fn aggregation_source_value_sql(metric: &ValidatedAggregationMetric) -> String {
    let source = quote_identifier(metric.source_field_key.as_deref().unwrap_or_default());
    typed_orderable_sql(&source, &metric.field_type)
}

fn data_field_from_validated(field: &ValidatedDatasetField) -> DataField {
    DataField {
        key: field.key.clone(),
        label: field.label.clone(),
        field_type: FieldType::parse(&field.field_type),
        position: field.position,
    }
}

fn data_op_error(error: tessara_data_ops::DataOpError) -> ApiError {
    ApiError::BadRequest(error.message().to_string())
}

fn internal_dataset_columns() -> BTreeSet<String> {
    ["__row_id", "__restriction_tier", "__scope_node_ids"]
        .into_iter()
        .map(String::from)
        .collect()
}

fn ordered_columns(columns: &BTreeSet<String>) -> Vec<String> {
    let mut ordered = ["__row_id", "__restriction_tier", "__scope_node_ids"]
        .into_iter()
        .filter(|column| columns.contains(*column))
        .map(String::from)
        .collect::<Vec<_>>();
    let internal = ordered.iter().cloned().collect::<BTreeSet<_>>();
    ordered.extend(
        columns
            .iter()
            .filter(|column| !internal.contains(*column))
            .cloned(),
    );
    ordered
}

fn select_union_expression(
    source_columns: &BTreeSet<String>,
    source_column: Option<&str>,
    output_column: &str,
) -> String {
    let quoted_output = quote_identifier(output_column);
    if let Some(source_column) = source_column
        && source_columns.contains(source_column)
    {
        return format!("{} AS {quoted_output}", quote_identifier(source_column));
    }
    if output_column == "__restriction_tier" {
        return format!("'public'::text AS {quoted_output}");
    }
    if output_column == "__scope_node_ids" {
        return format!("'{{}}'::uuid[] AS {quoted_output}");
    }
    format!("NULL::text AS {quoted_output}")
}

fn coalesced_join_expression(
    left_columns: &BTreeSet<String>,
    right_columns: &BTreeSet<String>,
    column: &str,
) -> String {
    if column == "__row_id" {
        return "CASE WHEN l.__row_id IS NOT NULL AND r.__row_id IS NOT NULL THEN md5(concat_ws('|', l.__row_id, r.__row_id)) ELSE COALESCE(l.__row_id, r.__row_id) END AS __row_id".to_string();
    }
    if column == "__restriction_tier" {
        return format!(
            "{} AS {}",
            greatest_restriction_tier_sql(&[
                "l.__restriction_tier".to_string(),
                "r.__restriction_tier".to_string(),
            ]),
            quote_identifier("__restriction_tier")
        );
    }
    if column == "__scope_node_ids" {
        return format!(
            "dataset_scope_union_state(COALESCE(l.{scope}, '{{}}'::uuid[]), \
             COALESCE(r.{scope}, '{{}}'::uuid[])) AS {scope}",
            scope = quote_identifier("__scope_node_ids")
        );
    }
    let quoted = quote_identifier(column);
    match (
        left_columns.contains(column),
        right_columns.contains(column),
    ) {
        (true, true) => format!("COALESCE(l.{quoted}, r.{quoted}) AS {quoted}"),
        (true, false) => format!("l.{quoted} AS {quoted}"),
        (false, true) => format!("r.{quoted} AS {quoted}"),
        (false, false) => format!("NULL::text AS {quoted}"),
    }
}

fn restriction_policy_tier_sql(policy: &ValidatedRestrictionPolicy) -> String {
    let predicate = |key: Option<&str>| {
        key.map(|key| boolean_expression_sql(&quote_identifier(key)))
            .unwrap_or_else(|| "FALSE".into())
    };
    let internal = predicate(policy.internal_field_key.as_deref());
    let restricted = predicate(policy.restricted_field_key.as_deref());
    let confidential = predicate(policy.confidential_field_key.as_deref());
    format!(
        "CASE
                    WHEN {confidential} THEN 'confidential'
                    WHEN {restricted} THEN 'restricted'
                    WHEN {internal} THEN 'internal'
                    ELSE 'public'
                END"
    )
}

fn effective_restriction_tier_sql(policy: &ValidatedRestrictionPolicy) -> String {
    greatest_restriction_tier_sql(&[
        quote_identifier("__restriction_tier"),
        restriction_policy_tier_sql(policy),
    ])
}

fn greatest_restriction_tier_sql(expressions: &[String]) -> String {
    let rank_args = expressions
        .iter()
        .map(|expression| restriction_tier_rank_sql(expression))
        .collect::<Vec<_>>()
        .join(", ");
    format!(
        "CASE GREATEST({rank_args})
                    WHEN 3 THEN 'confidential'
                    WHEN 2 THEN 'restricted'
                    WHEN 1 THEN 'internal'
                    ELSE 'public'
                END"
    )
}

fn max_restriction_tier_sql(expression: &str) -> String {
    format!(
        "CASE COALESCE(MAX({}), 0)
                    WHEN 3 THEN 'confidential'
                    WHEN 2 THEN 'restricted'
                    WHEN 1 THEN 'internal'
                    ELSE 'public'
                END",
        restriction_tier_rank_sql(expression)
    )
}

fn restriction_tier_rank_sql(expression: &str) -> String {
    format!(
        "CASE {expression}
                    WHEN 'confidential' THEN 3
                    WHEN 'restricted' THEN 2
                    WHEN 'internal' THEN 1
                    ELSE 0
                END"
    )
}

fn require_text(label: &str, value: &str) -> ApiResult<()> {
    if value.trim().is_empty() {
        Err(ApiError::BadRequest(format!("{label} is required")))
    } else {
        Ok(())
    }
}

fn require_identifier(label: &str, value: &str) -> ApiResult<()> {
    let valid = !value.is_empty()
        && value.len() <= 63
        && value
            .chars()
            .all(|character| character.is_ascii_alphanumeric() || character == '_')
        && value
            .chars()
            .next()
            .is_some_and(|character| character.is_ascii_alphabetic() || character == '_');
    if valid {
        Ok(())
    } else {
        Err(ApiError::BadRequest(format!(
            "{label} must use only letters, numbers, and underscores, and must start with a letter or underscore"
        )))
    }
}

fn sanitize_identifier(value: &str) -> String {
    value
        .chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() || character == '_' {
                character
            } else {
                '_'
            }
        })
        .collect()
}

fn quote_identifier(value: &str) -> String {
    format!("\"{}\"", value.replace('"', "\"\""))
}

fn sql_literal(value: &str) -> String {
    format!("'{}'", value.replace('\'', "''"))
}

fn parse_uuid(label: &str, value: &str) -> ApiResult<Uuid> {
    let id = Uuid::parse_str(value)
        .map_err(|_| ApiError::BadRequest(format!("{label} must be a canonical UUID")))?;
    if id.is_nil() || id.to_string() != value {
        return Err(ApiError::BadRequest(format!(
            "{label} must be a canonical non-nil UUID"
        )));
    }
    Ok(id)
}

fn source_binding_uuid(
    dataset_id: Option<Uuid>,
    alias: &str,
    source_kind: &str,
    source_identity: &str,
) -> Uuid {
    let mut digest = Sha256::new();
    digest.update(dataset_id.unwrap_or_else(Uuid::nil).as_bytes());
    digest.update([0]);
    digest.update(alias.as_bytes());
    digest.update([0]);
    digest.update(source_kind.as_bytes());
    digest.update([0]);
    digest.update(source_identity.as_bytes());
    let mut bytes = [0_u8; 16];
    bytes.copy_from_slice(&digest.finalize()[..16]);
    bytes[6] = (bytes[6] & 0x0f) | 0x50;
    bytes[8] = (bytes[8] & 0x3f) | 0x80;
    Uuid::from_bytes(bytes)
}

fn digest_serializable(value: &impl serde::Serialize) -> ApiResult<String> {
    let bytes = serde_jcs::to_vec(value).map_err(|error| {
        ApiError::Internal(anyhow::anyhow!(
            "Dataset source facts could not be canonicalized: {error}"
        ))
    })?;
    Ok(format!("sha256:{:x}", Sha256::digest(bytes)))
}

fn authorized_manage_scope(grant: &dyn provider_client::ProviderAuthorization) -> BTreeSet<Uuid> {
    grant
        .capability_scope_bindings()
        .iter()
        .filter(|binding| binding.capability.as_str() == "datasets:manage")
        .flat_map(|binding| {
            std::iter::once(binding.organization_root_id)
                .chain(binding.authorized_organization_ids.iter().copied())
        })
        .collect()
}

fn system_source_field_expression(source_field_key: &str) -> Option<&'static str> {
    match source_field_key {
        "__submission_id" => Some("imported.response_id::text"),
        "__form_version_id" => Some("imported.form_version_id::text"),
        "__node_id" => Some("imported.node_id::text"),
        "__node_name" => Some("imported.node_name"),
        "__submission_status" => Some("imported.status"),
        "__submitted_at" => Some("imported.submitted_at::text"),
        "__submission_created_at" => Some("imported.created_at::text"),
        "__last_updated_at" => Some("imported.last_modified_at::text"),
        "__last_updated_by_user_name" => Some("imported.last_modified_by_user_name"),
        _ => None,
    }
}

fn system_source_field_keys() -> [&'static str; 9] {
    [
        "__submission_id",
        "__form_version_id",
        "__node_id",
        "__node_name",
        "__submission_status",
        "__submitted_at",
        "__submission_created_at",
        "__last_updated_at",
        "__last_updated_by_user_name",
    ]
}

fn system_source_field_label(source_field_key: &str) -> &'static str {
    match source_field_key {
        "__submission_id" => "Submission ID",
        "__form_version_id" => "Form Version ID",
        "__node_id" => "Attached Node ID",
        "__node_name" => "Attached Node Name",
        "__submission_status" => "Submission Status",
        "__submitted_at" => "Submitted Date",
        "__submission_created_at" => "Created Date",
        "__last_updated_at" => "Updated Date",
        "__last_updated_by_user_name" => "Updated By User Name",
        _ => "System Field",
    }
}

fn system_source_field_type(source_field_key: &str) -> Option<&'static str> {
    match source_field_key {
        "__submitted_at" | "__submission_created_at" | "__last_updated_at" => Some("date"),
        "__submission_status" => Some("single_choice"),
        "__submission_id"
        | "__form_version_id"
        | "__node_id"
        | "__node_name"
        | "__last_updated_by_user_name" => Some("text"),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{Duration, Utc};
    use serde_json::json;
    use sqlx::postgres::PgPoolOptions;
    use std::{
        collections::BTreeMap,
        sync::{Arc, OnceLock},
    };
    use tessara_datasets_contract::DatasetProductCalculationFunctionV1;
    use tessara_forms_contract::{FormVersionField, FormVersionSection};
    use tessara_module_contract::{
        AUTHORIZATION_GRANT_SCHEMA_VERSION_V3, AuthorizationAudienceV1,
        AuthorizationGrantOperationV1, DependencyBindingKey, FunctionalContractId,
        ModuleDefinitionId, ModuleServiceIdentityRegistryV1, ModuleServicePrincipalV1,
        ProtocolSignaturePurposeV1, PurposeBoundSigningKeyV1,
    };

    fn validated_field(key: &str, field_type: &str) -> ValidatedDatasetField {
        ValidatedDatasetField {
            id: None,
            key: key.into(),
            label: key.into(),
            source_alias: "program".into(),
            source_field_key: key.into(),
            source_field_id: None,
            field_type: field_type.into(),
            position: 0,
        }
    }

    fn source_field(
        alias: &str,
        source_field_key: &str,
        field_type: &str,
    ) -> ValidatedDatasetField {
        ValidatedDatasetField {
            id: None,
            key: canonical_source_column_key(alias, source_field_key),
            label: source_field_key.into(),
            source_alias: alias.into(),
            source_field_key: source_field_key.into(),
            source_field_id: None,
            field_type: field_type.into(),
            position: 0,
        }
    }

    fn projection_field(key: &str, input_field_key: &str) -> DatasetProjectionFieldRequest {
        DatasetProjectionFieldRequest {
            key: key.into(),
            label: key.into(),
            input_field_key: Some(input_field_key.into()),
            position: 0,
        }
    }

    fn projection_operation(fields: Vec<DatasetProjectionFieldRequest>) -> DatasetOperationRequest {
        DatasetOperationRequest::Projection {
            fields: fields
                .into_iter()
                .enumerate()
                .map(|(position, mut field)| {
                    field.position = position as i32;
                    field
                })
                .collect(),
            position: 0,
        }
    }

    fn calculation_function(
        function: &str,
        argument: Option<&str>,
    ) -> DatasetProductCalculationFunctionV1 {
        DatasetProductCalculationFunctionV1 {
            function: function.into(),
            argument: argument.map(str::to_string),
            argument_mode: "value".into(),
            argument_field_key: None,
            position: 0,
        }
    }

    fn calculated_field_operation(
        key: &str,
        base_field_key: &str,
        function: &str,
        argument: Option<&str>,
    ) -> DatasetOperationRequest {
        DatasetOperationRequest::CalculatedFields {
            fields: vec![DatasetCalculatedFieldRequest {
                key: key.into(),
                label: key.into(),
                base_field_key: base_field_key.into(),
                functions: vec![calculation_function(function, argument)],
                position: 0,
            }],
            position: 0,
        }
    }

    fn filter_operation(
        field_key: &str,
        operator: &str,
        value: Option<&str>,
    ) -> DatasetOperationRequest {
        DatasetOperationRequest::Filter {
            filters: vec![DatasetRowFilterRequest {
                field_key: field_key.into(),
                operator: operator.into(),
                value_mode: "value".into(),
                value: value.map(str::to_string),
                value_field_key: None,
                position: 0,
            }],
            position: 0,
        }
    }

    fn count_rows_aggregation_operation(key: &str) -> DatasetOperationRequest {
        DatasetOperationRequest::Aggregation {
            group_fields: Vec::new(),
            metrics: vec![DatasetAggregationMetricRequest {
                key: key.into(),
                label: key.into(),
                function: "count_rows".into(),
                source_field_key: None,
                position: 0,
            }],
            row_picker: None,
            position: 0,
        }
    }

    fn positioned_operations(
        operations: Vec<DatasetOperationRequest>,
    ) -> Vec<DatasetOperationRequest> {
        operations
            .into_iter()
            .enumerate()
            .map(|(index, operation)| operation_with_position(operation, index as i32))
            .collect()
    }

    fn operation_with_position(
        operation: DatasetOperationRequest,
        position: i32,
    ) -> DatasetOperationRequest {
        match operation {
            DatasetOperationRequest::AddSource {
                source,
                add_type,
                join_keys,
                ..
            } => DatasetOperationRequest::AddSource {
                source,
                add_type,
                join_keys,
                position,
            },
            DatasetOperationRequest::Projection { fields, .. } => {
                DatasetOperationRequest::Projection { fields, position }
            }
            DatasetOperationRequest::Aggregation {
                group_fields,
                metrics,
                row_picker,
                ..
            } => DatasetOperationRequest::Aggregation {
                group_fields,
                metrics,
                row_picker,
                position,
            },
            DatasetOperationRequest::CalculatedFields { fields, .. } => {
                DatasetOperationRequest::CalculatedFields { fields, position }
            }
            DatasetOperationRequest::Filter { filters, .. } => {
                DatasetOperationRequest::Filter { filters, position }
            }
        }
    }

    fn test_state() -> &'static DatasetModuleState {
        static STATE: OnceLock<DatasetModuleState> = OnceLock::new();
        STATE.get_or_init(|| {
            let authorization_signer = signing_key(
                "tessara.core",
                "authoring-test-core",
                ProtocolSignaturePurposeV1::AuthorizationGrant,
                [71; 32],
            );
            let service_request_signer = signing_key(
                "tessara.core",
                "authoring-test-core",
                ProtocolSignaturePurposeV1::ModuleServiceRequest,
                [72; 32],
            );
            let owner_bootstrap_signer = signing_key(
                "tessara.core",
                "authoring-test-core",
                ProtocolSignaturePurposeV1::OwnerBootstrapAuthorization,
                [76; 32],
            );
            let bootstrap_signer = signing_key(
                "tessara.core",
                "authoring-test-core",
                ProtocolSignaturePurposeV1::BootstrapValidationAuthorization,
                [73; 32],
            );
            let shell_signer = signing_key(
                "tessara.core",
                "authoring-test-core",
                ProtocolSignaturePurposeV1::ShellContext,
                [74; 32],
            );
            let dataset_signer = Arc::new(signing_key(
                "tessara.datasets",
                "authoring-test-dataset",
                ProtocolSignaturePurposeV1::ModuleServiceRequest,
                [75; 32],
            ));
            let bootstrap_receipt_signer = Arc::new(signing_key(
                "tessara.datasets",
                "authoring-test-dataset",
                ProtocolSignaturePurposeV1::OwnerBootstrapReceipt,
                [77; 32],
            ));
            let identity_registry = ModuleServiceIdentityRegistryV1::from_json(
                r#"{"schema_version":1,"identities":{"tessara.components":{"key_id":"authoring-test-component","public_key":"11qYAYKxCrfVS_7TyWQHOg7hcvPapiMlrwIaaPcHURo"}}}"#,
            )
            .expect("authoring test identity registry");
            let provider_urls = crate::REQUIRED_DATASET_PROVIDER_BINDINGS
                .into_iter()
                .map(|binding| (binding.to_owned(), "http://127.0.0.1:2".to_owned()))
                .collect::<BTreeMap<_, _>>();
            DatasetModuleState::new(
                PgPoolOptions::new()
                    .connect_lazy("postgres://dataset:dataset@127.0.0.1:1/dataset_test")
                    .expect("lazy authoring test pool"),
                crate::DatasetCoreVerifiers {
                    authorization: authorization_signer.verifier(),
                    owner_bootstrap: owner_bootstrap_signer.verifier(),
                    service_request: service_request_signer.verifier(),
                    bootstrap_validation: bootstrap_signer.verifier(),
                    shell: shell_signer.verifier(),
                },
                identity_registry,
                dataset_signer,
                bootstrap_receipt_signer,
                crate::DatasetServiceEndpoints::new("http://127.0.0.1:1", provider_urls)
                    .expect("authoring test service endpoints"),
                crate::DatasetValidationFaultControl::disabled(),
            )
            .expect("authoring test state")
        })
    }

    fn test_grant() -> &'static SignedEnvelopeV1<AuthorizationGrantV3> {
        static GRANT: OnceLock<SignedEnvelopeV1<AuthorizationGrantV3>> = OnceLock::new();
        GRANT.get_or_init(|| {
            let now = Utc::now();
            signing_key(
                "tessara.core",
                "authoring-test-core",
                ProtocolSignaturePurposeV1::AuthorizationGrant,
                [71; 32],
            )
            .sign(AuthorizationGrantV3 {
                schema_version: AUTHORIZATION_GRANT_SCHEMA_VERSION_V3,
                installation_id: Uuid::from_u128(900),
                original_actor_id: Uuid::from_u128(901),
                correlation_id: Uuid::from_u128(902),
                presenting_service: ModuleServicePrincipalV1::CoreGateway,
                audience: AuthorizationAudienceV1::ModuleInstance {
                    module_instance_id: Uuid::from_u128(903),
                    module_definition_id: ModuleDefinitionId::new("tessara.datasets")
                        .expect("Dataset definition ID"),
                },
                dependency_binding: DependencyBindingKey::new("tessara.core.datasets")
                    .expect("Dataset binding"),
                functional_contract: FunctionalContractId::new(
                    "tessara.datasets.dataset-major-line",
                )
                .expect("Dataset resource contract"),
                action: "datasets.authoring.test".into(),
                operation: AuthorizationGrantOperationV1::Mutation,
                capability_scope_bindings: Vec::new(),
                resource_assertion: None,
                delegation_basis: Vec::new(),
                authorization_revision: 1,
                organization_revision: 1,
                jti: Uuid::from_u128(904),
                issued_at: now,
                expires_at: now + Duration::minutes(5),
            })
            .expect("signed authoring test grant")
        })
    }

    fn signing_key(
        issuer: &str,
        key_id: &str,
        purpose: ProtocolSignaturePurposeV1,
        secret: [u8; 32],
    ) -> PurposeBoundSigningKeyV1 {
        PurposeBoundSigningKeyV1::from_secret_bytes(issuer, key_id, purpose, secret)
            .expect("authoring test signing key")
    }

    fn query_spec_builder(
        ctes: Vec<String>,
        current_cte: String,
        fields: Vec<ValidatedDatasetField>,
        cte_index: usize,
    ) -> QuerySpecBuilder<'static, 'static> {
        QuerySpecBuilder {
            state: test_state(),
            grant: test_grant(),
            dataset_id: None,
            requested_scope_node_ids: BTreeSet::new(),
            authorized_scope_node_ids: BTreeSet::new(),
            ctes,
            current_cte,
            fields,
            cte_index,
            aliases: BTreeSet::new(),
            sources: Vec::new(),
            connection: None,
        }
    }

    #[test]
    fn restriction_policy_uses_most_sensitive_matching_flag() {
        let policy = ValidatedRestrictionPolicy {
            internal_field_key: Some("is_internal".into()),
            restricted_field_key: Some("is_restricted".into()),
            confidential_field_key: Some("is_confidential".into()),
        };

        let sql = restriction_policy_tier_sql(&policy);
        let confidential = sql.find("confidential").expect("confidential branch");
        let restricted = sql.find("restricted").expect("restricted branch");
        let internal = sql.find("internal").expect("internal branch");

        assert!(confidential < restricted);
        assert!(restricted < internal);
    }

    #[test]
    fn effective_restriction_tier_preserves_upstream_dataset_tier() {
        let policy = ValidatedRestrictionPolicy::default();

        let sql = effective_restriction_tier_sql(&policy);

        assert!(sql.contains("GREATEST"));
        assert!(sql.contains("\"__restriction_tier\""));
        assert!(sql.contains("WHEN 'confidential' THEN 3"));
    }

    #[test]
    fn operation_positions_must_match_array_order() {
        let operation =
            operation_with_position(projection_operation(vec![projection_field("a", "a")]), 3);

        let error = validate_operation_position(&operation, 0)
            .expect_err("operation position mismatch should fail")
            .to_string();

        assert!(error.contains("array index 0 has position 3; expected 0"));
    }

    #[test]
    fn date_calculation_comparison_uses_field_argument_and_date_casts() {
        let function = ValidatedCalculationFunction {
            function: CalculationFunction::LessThanOrEqual,
            argument: None,
            argument_field_key: Some("program2__session_date".into()),
            input_type: "date".into(),
        };

        assert_eq!(
            calculation_function_sql(&function, "\"program__session_date\""),
            "CASE WHEN NULLIF(\"program__session_date\", '')::date <= NULLIF(\"program2__session_date\", '')::date THEN 'true' ELSE 'false' END"
        );
    }

    #[test]
    fn date_format_calculation_uses_date_cast_for_date_input() {
        let function = ValidatedCalculationFunction {
            function: CalculationFunction::FormatDate,
            argument: Some("%Y-%m-%d".into()),
            argument_field_key: None,
            input_type: "date".into(),
        };

        assert_eq!(
            calculation_function_sql(&function, "\"program__session_date\""),
            "TO_CHAR(NULLIF(\"program__session_date\", '')::date, '%Y-%m-%d')"
        );
    }

    #[test]
    fn invalid_date_filter_literals_are_rejected_before_sql_generation() {
        assert!(validate_filter_literal("date", "2026-06-03").is_ok());
        assert!(validate_filter_literal("date", "06/03/2026").is_err());
        assert!(validate_filter_literal("datetime", "2026-06-03T13:45").is_ok());
        assert!(validate_filter_literal("datetime", "not-a-date").is_err());
    }

    #[test]
    fn invalid_boolean_filter_literals_are_rejected_before_sql_generation() {
        assert!(validate_filter_literal("boolean", "yes").is_ok());
        assert!(validate_filter_literal("boolean", "0").is_ok());
        assert!(validate_filter_literal("boolean", "maybe").is_err());
    }

    #[test]
    fn add_source_union_catalog_merges_matching_source_fields_under_union_alias() {
        let left = vec![
            source_field("source_1", "activity_summary", "text"),
            source_field("source_1", "expected_attendees", "number"),
        ];
        let right = vec![
            source_field("source_3", "activity_summary", "text"),
            source_field("source_3", "focus_tags", "multi_choice"),
        ];

        let (fields, mappings) =
            union_field_catalog(&left, &right, "union_2").expect("compatible union catalog");

        assert_eq!(
            fields
                .iter()
                .map(|field| field.key.as_str())
                .collect::<Vec<_>>(),
            vec![
                "union_2__activity_summary",
                "source_1__expected_attendees",
                "source_3__focus_tags",
            ]
        );
        assert_eq!(fields[0].source_alias, "union_2");
        let activity_mapping = mappings
            .iter()
            .find(|mapping| mapping.output_key == "union_2__activity_summary")
            .expect("merged activity mapping");
        assert_eq!(
            activity_mapping.left_key.as_deref(),
            Some("source_1__activity_summary")
        );
        assert_eq!(
            activity_mapping.right_key.as_deref(),
            Some("source_3__activity_summary")
        );
    }

    #[test]
    fn golden_catalog_union_merge_matches_editor_contract() {
        let left = vec![
            source_field("source_1", "activity_summary", "text"),
            source_field("source_1", "expected_attendees", "number"),
        ];
        let right = vec![
            source_field("source_3", "activity_summary", "text"),
            source_field("source_3", "focus_tags", "multi_choice"),
        ];

        let (fields, _) =
            union_field_catalog(&left, &right, "union_2").expect("compatible union catalog");

        assert_eq!(
            fields
                .iter()
                .map(|field| (field.key.as_str(), field.field_type.as_str()))
                .collect::<Vec<_>>(),
            vec![
                ("union_2__activity_summary", "text"),
                ("source_1__expected_attendees", "number"),
                ("source_3__focus_tags", "multi_choice"),
            ]
        );
    }

    #[test]
    fn add_source_union_catalog_rejects_matching_source_fields_with_different_types() {
        let left = vec![source_field("source_1", "activity_summary", "text")];
        let right = vec![source_field("source_3", "activity_summary", "number")];

        let error = match union_field_catalog(&left, &right, "union_2") {
            Ok(_) => panic!("conflicting source field types should fail"),
            Err(error) => error.to_string(),
        };

        assert!(error.contains("union field 'activity_summary' has incompatible types"));
    }

    #[test]
    fn numeric_min_max_aggregation_uses_numeric_casts() {
        let min_metric = ValidatedAggregationMetric {
            key: "minimum_target".into(),
            label: "Minimum Target".into(),
            function: AggregateFunction::Min,
            source_field_key: Some("program__participant_target".into()),
            field_type: "number".into(),
            position: 0,
        };
        let max_metric = ValidatedAggregationMetric {
            function: AggregateFunction::Max,
            key: "maximum_target".into(),
            label: "Maximum Target".into(),
            source_field_key: Some("program__participant_target".into()),
            field_type: "number".into(),
            position: 1,
        };

        assert_eq!(
            aggregation_metric_sql(&min_metric),
            "MIN(NULLIF(\"program__participant_target\", '')::numeric)::text AS \"minimum_target\""
        );
        assert_eq!(
            aggregation_metric_sql(&max_metric),
            "MAX(NULLIF(\"program__participant_target\", '')::numeric)::text AS \"maximum_target\""
        );
    }

    #[test]
    fn count_distinct_aggregation_uses_distinct_non_empty_values() {
        let metric = ValidatedAggregationMetric {
            key: "distinct_programs".into(),
            label: "Distinct Programs".into(),
            function: AggregateFunction::CountDistinct,
            source_field_key: Some("program__name".into()),
            field_type: "number".into(),
            position: 0,
        };

        assert_eq!(
            aggregation_metric_sql(&metric),
            "COUNT(DISTINCT NULLIF(\"program__name\", ''))::text AS \"distinct_programs\""
        );
    }

    #[test]
    fn row_picker_ordering_uses_typed_expression() {
        assert_eq!(
            typed_orderable_sql("\"program__participant_target\"", "number"),
            "NULLIF(\"program__participant_target\", '')::numeric"
        );
        assert_eq!(
            typed_orderable_sql("\"program__review_started\"", "date"),
            "NULLIF(\"program__review_started\", '')::date"
        );
    }

    #[test]
    fn map_value_calculation_parses_multiple_pairs() {
        let function = ValidatedCalculationFunction {
            function: CalculationFunction::MapValue,
            argument: Some("draft=>booger, submitted=snot".into()),
            argument_field_key: None,
            input_type: "text".into(),
        };

        let sql = calculation_function_sql(&function, "\"program__submission_status\"");

        assert!(
            sql.contains(
                "WHEN COALESCE(\"program__submission_status\", '') = 'draft' THEN 'booger'"
            )
        );
        assert!(sql.contains(
            "WHEN COALESCE(\"program__submission_status\", '') = 'submitted' THEN 'snot'"
        ));
        assert!(sql.ends_with("ELSE \"program__submission_status\" END"));
    }

    #[test]
    fn numeric_row_filter_can_compare_against_another_field() {
        let filter = ValidatedRowFilter {
            field_key: "program__participants".into(),
            field_type: "number".into(),
            operator: FilterOperator::Gte,
            value: None,
            value_field_key: Some("program2__participants".into()),
        };

        assert_eq!(
            row_filter_sql(&filter),
            "NULLIF(\"program__participants\", '')::numeric >= NULLIF(\"program2__participants\", '')::numeric"
        );
    }

    #[test]
    fn row_filter_equality_uses_field_type_casts() {
        let numeric_filter = ValidatedRowFilter {
            field_key: "program__participants".into(),
            field_type: "number".into(),
            operator: FilterOperator::Equals,
            value: Some("135.0".into()),
            value_field_key: None,
        };
        let date_filter = ValidatedRowFilter {
            field_key: "program__review_window_start".into(),
            field_type: "date".into(),
            operator: FilterOperator::Equals,
            value: Some("2026-06-03".into()),
            value_field_key: None,
        };

        assert!(row_filter_sql(&numeric_filter).contains("::numeric"));
        let date_sql = row_filter_sql(&date_filter);
        assert!(date_sql.contains("::date"));
        assert!(!date_sql.contains("::timestamptz"));
    }

    #[test]
    fn shared_negative_and_range_filters_generate_sql() {
        let not_contains = ValidatedRowFilter {
            field_key: "program__name".into(),
            field_type: "text".into(),
            operator: FilterOperator::NotContains,
            value: Some("archived".into()),
            value_field_key: None,
        };
        let between = ValidatedRowFilter {
            field_key: "program__participants".into(),
            field_type: "number".into(),
            operator: FilterOperator::Between,
            value: Some("10, 20".into()),
            value_field_key: None,
        };

        assert_eq!(
            row_filter_sql(&not_contains),
            "POSITION(LOWER('archived') IN LOWER(COALESCE(\"program__name\", ''))) = 0"
        );
        assert_eq!(
            row_filter_sql(&between),
            "(NULLIF(\"program__participants\", '')::numeric >= NULLIF('10', '')::numeric AND NULLIF(\"program__participants\", '')::numeric <= NULLIF('20', '')::numeric)"
        );
    }

    #[test]
    fn numeric_calculation_equality_rejects_non_numeric_literals() {
        let result = CalculationFunction::Equal.validate_argument(
            &Some("george".into()),
            "number",
            "calculated_1",
        );

        assert!(result.is_err());
    }

    #[test]
    fn typed_default_literals_reject_invalid_values() {
        assert!(
            CalculationFunction::Constant
                .validate_argument(&Some("banana".into()), "number", "calculated_1")
                .is_err()
        );
        assert!(
            CalculationFunction::Coalesce
                .validate_argument(&Some("not-a-date".into()), "date", "calculated_1")
                .is_err()
        );
        assert!(
            CalculationFunction::Equal
                .validate_argument(&Some("maybe".into()), "boolean", "calculated_1")
                .is_err()
        );
    }

    #[test]
    fn coalesce_preserves_carry_forward_type_for_numeric_pipeline() {
        let fields = vec![validated_field("response_count", "number")];
        let calculated = validate_dataset_calculated_fields(
            vec![DatasetCalculatedFieldRequest {
                key: "defaulted_plus_one".into(),
                label: "Defaulted Plus One".into(),
                base_field_key: "response_count".into(),
                functions: vec![
                    DatasetProductCalculationFunctionV1 {
                        function: "coalesce".into(),
                        argument: Some("0".into()),
                        argument_mode: "value".into(),
                        argument_field_key: None,
                        position: 0,
                    },
                    DatasetProductCalculationFunctionV1 {
                        function: "add".into(),
                        argument: Some("1".into()),
                        argument_mode: "value".into(),
                        argument_field_key: None,
                        position: 1,
                    },
                ],
                position: 0,
            }],
            &fields,
        )
        .expect("calculated field should validate");

        assert_eq!(calculated[0].field_type, "number");
        let sql = calculated_field_sql(&calculated[0]);
        assert!(sql.contains("COALESCE(NULLIF(\"response_count\", ''), '0')"));
        assert!(sql.contains("::numeric + 1"));
    }

    #[test]
    fn boolean_equality_uses_nullable_boolean_normalization() {
        let filter = ValidatedRowFilter {
            field_key: "program__funding_confirmed".into(),
            field_type: "boolean".into(),
            operator: FilterOperator::Equals,
            value: Some("yes".into()),
            value_field_key: None,
        };

        let sql = row_filter_sql(&filter);

        assert!(sql.contains("CASE WHEN NULLIF(\"program__funding_confirmed\", '') IS NULL"));
        assert!(sql.contains("CASE WHEN NULLIF('yes', '') IS NULL"));
        assert!(sql.contains("LOWER(COALESCE(\"program__funding_confirmed\", '')) IN"));
        assert!(sql.contains("LOWER(COALESCE('yes', '')) IN"));
    }

    #[tokio::test]
    async fn query_spec_renders_ctes_in_saved_operation_order() {
        let field = validated_field("program__participant_target", "number");
        let mut spec = query_spec_builder(
            vec![r#"program_1 AS (SELECT "__row_id", "__restriction_tier", "__scope_node_ids", "program__participant_target")"#.into()],
            "program_1".into(),
            vec![field],
            1,
        );

        spec.apply_operations(&[
            DatasetOperationRequest::Projection {
                fields: vec![DatasetProjectionFieldRequest {
                    key: "program__participant_target".into(),
                    label: "Participant Target".into(),
                    input_field_key: Some("program__participant_target".into()),
                    position: 0,
                }],
                position: 0,
            },
            DatasetOperationRequest::Aggregation {
                group_fields: vec![],
                metrics: vec![DatasetAggregationMetricRequest {
                    key: "target_average".into(),
                    label: "Target Average".into(),
                    function: "average".into(),
                    source_field_key: Some("program__participant_target".into()),
                    position: 0,
                }],
                row_picker: None,
                position: 1,
            },
            DatasetOperationRequest::CalculatedFields {
                fields: vec![DatasetCalculatedFieldRequest {
                    key: "target_plus_one".into(),
                    label: "Target Plus One".into(),
                    base_field_key: "target_average".into(),
                    functions: vec![DatasetProductCalculationFunctionV1 {
                        function: "add".into(),
                        argument: Some("1".into()),
                        argument_mode: "value".into(),
                        argument_field_key: None,
                        position: 0,
                    }],
                    position: 0,
                }],
                position: 2,
            },
            DatasetOperationRequest::Filter {
                filters: vec![DatasetRowFilterRequest {
                    field_key: "target_plus_one".into(),
                    operator: "greater_than_or_equal".into(),
                    value_mode: "value".into(),
                    value: Some("100".into()),
                    value_field_key: None,
                    position: 0,
                }],
                position: 3,
            },
        ])
        .await
        .expect("operation pipeline should validate");
        let sql = spec.final_sql(&ValidatedRestrictionPolicy::default());

        let source = sql.find("program_1 AS").expect("source cte");
        let projection = sql.find("projection_2 AS").expect("projection cte");
        let aggregated = sql.find("aggregation_3 AS").expect("aggregation cte");
        let calculated = sql.find("calculated_fields_4 AS").expect("calculation cte");
        let filtered = sql.find("filtered_fields_5 AS").expect("filter cte");
        let final_select = sql.rfind("SELECT").expect("final select");
        assert!(source < projection);
        assert!(projection < aggregated);
        assert!(aggregated < calculated);
        assert!(calculated < filtered);
        assert!(filtered < final_select);
    }

    #[tokio::test]
    async fn projection_rejects_unavailable_input_keys_without_legacy_resolution() {
        let mut spec = query_spec_builder(
            vec![r#"program_1 AS (SELECT "__row_id", "__restriction_tier", "__scope_node_ids", "program__participant_target")"#.into()],
            "program_1".into(),
            vec![validated_field("program__participant_target", "number")],
            1,
        );

        let error = spec
            .apply_operations(&[projection_operation(vec![projection_field(
                "participant_target",
                "participant_target",
            )])])
            .await
            .expect_err("unprefixed input should not resolve to a canonical field")
            .to_string();

        assert!(error.contains(
            "projection field 'participant_target' references unavailable field 'participant_target'"
        ));
    }

    #[tokio::test]
    async fn query_spec_supports_repeated_operations_in_saved_order() {
        let mut spec = query_spec_builder(
            vec![
                r#"source_1 AS (SELECT "__row_id", "__restriction_tier", "__scope_node_ids", "a")"#
                    .into(),
            ],
            "source_1".into(),
            vec![validated_field("a", "number")],
            1,
        );

        spec.apply_operations(&positioned_operations(vec![
            projection_operation(vec![projection_field("a", "a")]),
            calculated_field_operation("a_plus_one", "a", "add", Some("1")),
            filter_operation("a_plus_one", "greater_than", Some("1")),
            projection_operation(vec![projection_field("renamed_a", "a_plus_one")]),
            calculated_field_operation("renamed_a_plus_one", "renamed_a", "add", Some("1")),
            filter_operation("renamed_a_plus_one", "less_than_or_equal", Some("10")),
            count_rows_aggregation_operation("row_count"),
            calculated_field_operation("row_count_plus_one", "row_count", "add", Some("1")),
        ]))
        .await
        .expect("repeated operations should validate");
        let sql = spec.final_sql(&ValidatedRestrictionPolicy::default());

        let expected_order = [
            "source_1 AS",
            "projection_2 AS",
            "calculated_fields_3 AS",
            "filtered_fields_4 AS",
            "projection_5 AS",
            "calculated_fields_6 AS",
            "filtered_fields_7 AS",
            "aggregation_8 AS",
            "calculated_fields_9 AS",
            "FROM \"calculated_fields_9\"",
        ];
        let mut previous_index = 0;
        for needle in expected_order {
            let index = sql
                .find(needle)
                .unwrap_or_else(|| panic!("expected SQL to contain {needle}"));
            assert!(
                index >= previous_index,
                "{needle} should appear after the prior operation"
            );
            previous_index = index;
        }
        assert_eq!(
            spec.fields()
                .iter()
                .map(|field| field.key.as_str())
                .collect::<Vec<_>>(),
            vec!["row_count", "row_count_plus_one"]
        );
    }

    #[tokio::test]
    async fn query_spec_rejects_references_removed_by_projection() {
        let mut spec = query_spec_builder(
            vec![r#"source_1 AS (SELECT "__row_id", "__restriction_tier", "__scope_node_ids", "kept", "removed")"#.into()],
            "source_1".into(),
            vec![
                validated_field("kept", "text"),
                validated_field("removed", "text"),
            ],
            1,
        );

        let error = spec
            .apply_operations(&positioned_operations(vec![
                projection_operation(vec![projection_field("kept", "kept")]),
                filter_operation("removed", "is_not_empty", None),
            ]))
            .await
            .expect_err("removed field should not be available after projection")
            .to_string();

        assert!(error.contains("row filter field 'removed' is not projected"));
    }

    #[tokio::test]
    async fn calculated_output_types_feed_later_filters_and_aggregations() {
        let mut spec = query_spec_builder(
            vec![r#"source_1 AS (SELECT "__row_id", "__restriction_tier", "__scope_node_ids", "score")"#.into()],
            "source_1".into(),
            vec![validated_field("score", "number")],
            1,
        );

        spec.apply_operations(&positioned_operations(vec![
            projection_operation(vec![projection_field("score", "score")]),
            calculated_field_operation("score_plus_one", "score", "add", Some("1")),
            calculated_field_operation(
                "score_is_large",
                "score_plus_one",
                "greater_than",
                Some("10"),
            ),
            filter_operation("score_is_large", "equals", Some("true")),
            DatasetOperationRequest::Aggregation {
                group_fields: Vec::new(),
                metrics: vec![DatasetAggregationMetricRequest {
                    key: "average_score".into(),
                    label: "Average Score".into(),
                    function: "average".into(),
                    source_field_key: Some("score_plus_one".into()),
                    position: 0,
                }],
                row_picker: None,
                position: 0,
            },
        ]))
        .await
        .expect("calculated field types should feed later validation");

        assert_eq!(spec.fields()[0].key, "average_score");
        assert_eq!(spec.fields()[0].field_type, "number");
    }

    #[tokio::test]
    async fn golden_catalog_projection_aggregation_calculation_matches_editor_contract() {
        let mut spec = query_spec_builder(
            vec![r#"source_1 AS (SELECT "__row_id", "__restriction_tier", "__scope_node_ids", "source_1__expected_attendees")"#.into()],
            "source_1".into(),
            vec![source_field("source_1", "expected_attendees", "number")],
            1,
        );

        spec.apply_operations(&positioned_operations(vec![
            projection_operation(vec![projection_field(
                "expected_attendees",
                "source_1__expected_attendees",
            )]),
            DatasetOperationRequest::Aggregation {
                group_fields: Vec::new(),
                metrics: vec![DatasetAggregationMetricRequest {
                    key: "attendee_total".into(),
                    label: "Attendee Total".into(),
                    function: "sum".into(),
                    source_field_key: Some("expected_attendees".into()),
                    position: 0,
                }],
                row_picker: None,
                position: 0,
            },
            calculated_field_operation(
                "attendee_total_plus_one",
                "attendee_total",
                "add",
                Some("1"),
            ),
        ]))
        .await
        .expect("golden catalog pipeline should validate");

        assert_eq!(
            spec.fields()
                .iter()
                .map(|field| (field.key.as_str(), field.field_type.as_str()))
                .collect::<Vec<_>>(),
            vec![
                ("attendee_total", "number"),
                ("attendee_total_plus_one", "number"),
            ]
        );
    }

    #[tokio::test]
    async fn query_spec_rejects_duplicate_projection_output_keys() {
        let mut spec = query_spec_builder(
            vec![r#"source_1 AS (SELECT "__row_id", "__restriction_tier", "__scope_node_ids", "a", "b")"#.into()],
            "source_1".into(),
            vec![validated_field("a", "text"), validated_field("b", "text")],
            1,
        );

        let error = spec
            .apply_operations(&[projection_operation(vec![
                projection_field("dup", "a"),
                projection_field("dup", "b"),
            ])])
            .await
            .expect_err("duplicate projection keys should fail")
            .to_string();

        assert!(error.contains("projection field key 'dup' is duplicated"));
    }

    #[tokio::test]
    async fn query_spec_rejects_duplicate_projection_input_keys() {
        let mut spec = query_spec_builder(
            vec![
                r#"source_1 AS (SELECT "__row_id", "__restriction_tier", "__scope_node_ids", "a")"#
                    .into(),
            ],
            "source_1".into(),
            vec![validated_field("a", "text")],
            1,
        );

        let error = spec
            .apply_operations(&[projection_operation(vec![
                projection_field("a", "a"),
                projection_field("a_copy", "a"),
            ])])
            .await
            .expect_err("duplicate projection inputs should fail")
            .to_string();

        assert!(error.contains("projection input field 'a' is duplicated"));
    }

    #[tokio::test]
    async fn restrictions_render_after_all_operations_and_keep_restriction_fields_available() {
        let restriction_flag = validated_field("internal_flag", "boolean");
        let mut spec = query_spec_builder(
            vec![r#"source_1 AS (SELECT "__row_id", "__restriction_tier", "__scope_node_ids", "title", "internal_flag")"#.into()],
            "source_1".into(),
            vec![validated_field("title", "text"), restriction_flag],
            1,
        );

        spec.apply_operations(&positioned_operations(vec![
            projection_operation(vec![
                projection_field("title", "title"),
                projection_field("internal_flag", "internal_flag"),
            ]),
            filter_operation("title", "contains", Some("Ready")),
        ]))
        .await
        .expect("restriction source field should remain available after final operation");
        let sql = spec.final_sql(&ValidatedRestrictionPolicy {
            internal_field_key: Some("internal_flag".into()),
            restricted_field_key: None,
            confidential_field_key: None,
        });

        assert!(
            sql.find("filtered_fields_3 AS").expect("filter cte")
                < sql
                    .rfind("FROM \"filtered_fields_3\"")
                    .expect("final filtered source")
        );
        assert!(sql.contains("WHEN LOWER(COALESCE(\"internal_flag\", '')) IN"));
        assert!(sql.contains("\"internal_flag\""));
        assert!(
            spec.fields()
                .iter()
                .any(|field| field.key == "internal_flag")
        );
    }

    fn canonical_form_schema() -> FormVersionSchemaResponse {
        let first_section = Uuid::from_u128(10);
        let second_section = Uuid::from_u128(11);
        FormVersionSchemaResponse {
            schema_version: FORM_VERSION_SCHEMA_VERSION,
            form_id: Uuid::from_u128(1),
            form_version_id: Uuid::from_u128(2),
            form_name: "Enrollment".into(),
            form_slug: "enrollment".into(),
            version_label: Some("Published".into()),
            version_major: Some(1),
            source_scope_node_ids: vec![Uuid::from_u128(100), Uuid::from_u128(101)],
            source_scope_revision: String::new(),
            source_scope_digest: String::new(),
            content_revision: String::new(),
            content_digest: String::new(),
            sections: vec![
                FormVersionSection {
                    section_id: first_section,
                    key: first_section.to_string(),
                    label: "Applicant".into(),
                    position: 0,
                },
                FormVersionSection {
                    section_id: second_section,
                    key: second_section.to_string(),
                    label: "Program".into(),
                    position: 1,
                },
            ],
            fields: vec![
                FormVersionField {
                    field_id: Uuid::from_u128(20),
                    key: "first".into(),
                    label: "Zeta".into(),
                    field_type: "text".into(),
                    required: true,
                    options: Vec::new(),
                    section_id: Some(first_section),
                    position: 0,
                    grid_row: 1,
                    grid_column: 1,
                },
                FormVersionField {
                    field_id: Uuid::from_u128(21),
                    key: "second".into(),
                    label: "Alpha".into(),
                    field_type: "text".into(),
                    required: false,
                    options: Vec::new(),
                    section_id: Some(first_section),
                    position: 0,
                    grid_row: 1,
                    grid_column: 7,
                },
                FormVersionField {
                    field_id: Uuid::from_u128(22),
                    key: "third".into(),
                    label: "Program".into(),
                    field_type: "single_choice".into(),
                    required: false,
                    options: vec![json!("A"), json!("B")],
                    section_id: Some(second_section),
                    position: 0,
                    grid_row: 1,
                    grid_column: 1,
                },
            ],
        }
        .with_recomputed_digests()
        .unwrap()
    }

    fn scope() -> BTreeSet<Uuid> {
        BTreeSet::from([Uuid::from_u128(100), Uuid::from_u128(101)])
    }

    fn is_dependency_incompatible(result: &ApiResult<FormVersionSchemaResponse>) -> bool {
        matches!(
            result,
            Err(ApiError::Module(
                DatasetModuleError::DependencyIncompatible(_)
            ))
        )
    }

    #[test]
    fn form_schema_adoption_preserves_bound_snapshot_and_grid_order() {
        let schema = canonical_form_schema();
        let adopted = validate_form_version_schema(
            serde_json::to_value(&schema).unwrap(),
            schema.form_id,
            schema.form_version_id,
            &scope(),
            &scope(),
        )
        .unwrap();
        let source = validated_form_source(
            Some(Uuid::from_u128(30)),
            "responses",
            adopted.form_id,
            adopted.form_version_id,
            0,
            &adopted,
        );

        assert_eq!(source.source_slug.as_deref(), Some("enrollment"));
        assert_eq!(source.source_scope_node_ids, adopted.source_scope_node_ids);
        assert_eq!(source.source_scope_revision, adopted.source_scope_revision);
        assert_eq!(source.source_scope_digest, adopted.source_scope_digest);
        assert_eq!(source.source_content_revision, adopted.content_revision);
        assert_eq!(source.source_content_digest, adopted.content_digest);
        let fields = load_form_source_catalog(&source, &adopted).unwrap();
        assert_eq!(
            fields[system_source_field_keys().len()..]
                .iter()
                .map(|field| field.key.as_str())
                .collect::<Vec<_>>(),
            ["responses__first", "responses__second", "responses__third"]
        );
    }

    #[test]
    fn malformed_substituted_and_digest_invalid_form_schemas_are_incompatible() {
        let schema = canonical_form_schema();
        let wire = serde_json::to_value(&schema).unwrap();

        let mut malformed = wire.clone();
        malformed["unexpected"] = json!(true);
        assert!(is_dependency_incompatible(&validate_form_version_schema(
            malformed,
            schema.form_id,
            schema.form_version_id,
            &scope(),
            &scope(),
        )));
        assert!(is_dependency_incompatible(&validate_form_version_schema(
            wire.clone(),
            Uuid::from_u128(999),
            schema.form_version_id,
            &scope(),
            &scope(),
        )));

        let mut tampered = wire;
        tampered["fields"][0]["label"] = json!("Tampered");
        assert!(is_dependency_incompatible(&validate_form_version_schema(
            tampered,
            schema.form_id,
            schema.form_version_id,
            &scope(),
            &scope(),
        )));
    }

    #[test]
    fn form_source_scope_requires_requested_and_granted_whole_scope_containment() {
        let schema = canonical_form_schema();
        let wire = serde_json::to_value(&schema).unwrap();
        let partial = BTreeSet::from([Uuid::from_u128(100)]);

        assert!(matches!(
            validate_form_version_schema(
                wire.clone(),
                schema.form_id,
                schema.form_version_id,
                &partial,
                &scope(),
            ),
            Err(ApiError::BadRequest(_))
        ));
        assert!(is_dependency_incompatible(&validate_form_version_schema(
            wire,
            schema.form_id,
            schema.form_version_id,
            &scope(),
            &partial,
        )));
    }

    #[test]
    fn hidden_row_scope_is_canonical_through_pipeline_operations() {
        assert_eq!(
            internal_dataset_columns(),
            BTreeSet::from([
                "__restriction_tier".to_string(),
                "__row_id".to_string(),
                "__scope_node_ids".to_string(),
            ])
        );
        assert_eq!(
            ordered_columns(&internal_dataset_columns()),
            ["__row_id", "__restriction_tier", "__scope_node_ids"]
        );
        assert!(
            coalesced_join_expression(
                &internal_dataset_columns(),
                &internal_dataset_columns(),
                "__scope_node_ids",
            )
            .contains("dataset_scope_union_state")
        );
        assert_eq!(
            select_union_expression(&BTreeSet::new(), None, "__scope_node_ids"),
            r#"'{}'::uuid[] AS "__scope_node_ids""#
        );
    }
}
