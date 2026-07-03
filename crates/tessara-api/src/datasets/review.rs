use super::*;

pub(super) fn compatibility_findings(
    published: &DatasetRevisionSnapshot,
    candidate: &DatasetRevisionSnapshot,
) -> Vec<DatasetCompatibilityFinding> {
    let mut findings = Vec::new();
    let published_fields = published
        .output_fields
        .iter()
        .map(|field| (field.key.as_str(), field))
        .collect::<BTreeMap<_, _>>();
    let candidate_fields = candidate
        .output_fields
        .iter()
        .map(|field| (field.key.as_str(), field))
        .collect::<BTreeMap<_, _>>();
    findings.extend(source_changelog_findings(published, candidate));
    findings.extend(operation_changelog_findings(
        published,
        candidate,
        &published_fields,
        &candidate_fields,
    ));

    for (key, field) in &published_fields {
        match candidate_fields.get(key) {
            None => findings.push(compatibility_finding(
                DatasetVersionImpact::Major,
                "removed_output_field",
                format!("Output field '{}' is removed.", field.label),
                Some((*key).to_string()),
            )),
            Some(candidate_field) if candidate_field.field_type != field.field_type => {
                findings.push(compatibility_finding(
                    DatasetVersionImpact::Major,
                    "changed_output_field_type",
                    format!(
                        "Output field '{}' changes type from '{}' to '{}'.",
                        field.label, field.field_type, candidate_field.field_type
                    ),
                    Some((*key).to_string()),
                ));
            }
            Some(candidate_field) if candidate_field.label != field.label => {
                findings.push(compatibility_finding(
                    DatasetVersionImpact::Patch,
                    "changed_output_field_label",
                    format!(
                        "Output field key '{}' changes label from '{}' to '{}'.",
                        key, field.label, candidate_field.label
                    ),
                    Some((*key).to_string()),
                ));
            }
            _ => {}
        }
    }
    for (key, field) in &candidate_fields {
        if !published_fields.contains_key(key) {
            findings.push(compatibility_finding(
                DatasetVersionImpact::Minor,
                "added_output_field",
                format!("Output field '{}' is added.", field.label),
                Some((*key).to_string()),
            ));
        }
    }
    if restriction_policy_json(&published.restriction_policy)
        != restriction_policy_json(&candidate.restriction_policy)
    {
        findings.push(compatibility_finding(
            DatasetVersionImpact::Minor,
            "changed_restriction_policy",
            "Restriction policy changes and should be reviewed before carry-forward.".into(),
            None,
        ));
    }
    if published.metadata.name != candidate.metadata.name {
        findings.push(compatibility_finding(
            DatasetVersionImpact::Patch,
            "changed_dataset_name",
            format!(
                "Dataset name changes from '{}' to '{}'.",
                published.metadata.name, candidate.metadata.name
            ),
            None,
        ));
    }
    if published.metadata.slug != candidate.metadata.slug {
        findings.push(compatibility_finding(
            DatasetVersionImpact::Patch,
            "changed_dataset_slug",
            format!(
                "Dataset slug changes from '{}' to '{}'.",
                published.metadata.slug, candidate.metadata.slug
            ),
            None,
        ));
    }
    if published
        .metadata
        .visibility_node_ids
        .iter()
        .collect::<BTreeSet<_>>()
        != candidate
            .metadata
            .visibility_node_ids
            .iter()
            .collect::<BTreeSet<_>>()
    {
        findings.push(compatibility_finding(
            DatasetVersionImpact::Patch,
            "changed_dataset_visibility",
            "Dataset visibility scope changes.".into(),
            None,
        ));
    }
    findings
}

fn source_changelog_findings(
    published: &DatasetRevisionSnapshot,
    candidate: &DatasetRevisionSnapshot,
) -> Vec<DatasetCompatibilityFinding> {
    let published_sources = revision_sources_by_alias(published);
    let candidate_sources = revision_sources_by_alias(candidate);
    let mut findings = Vec::new();

    for (alias, source) in &published_sources {
        match candidate_sources.get(alias) {
            None => findings.push(compatibility_finding(
                DatasetVersionImpact::Major,
                "removed_dataset_source",
                format!("Source '{}' is removed.", alias),
                Some(alias.clone()),
            )),
            Some(candidate_source) if candidate_source != source => {
                findings.push(compatibility_finding(
                    DatasetVersionImpact::Minor,
                    "changed_dataset_source",
                    format!("Source '{}' changes binding.", alias),
                    Some(alias.clone()),
                ));
            }
            _ => {}
        }
    }

    for alias in candidate_sources.keys() {
        if !published_sources.contains_key(alias) {
            findings.push(compatibility_finding(
                DatasetVersionImpact::Minor,
                "added_dataset_source",
                format!("Source '{}' is added.", alias),
                Some(alias.clone()),
            ));
        }
    }

    findings
}

fn revision_sources_by_alias(
    snapshot: &DatasetRevisionSnapshot,
) -> BTreeMap<String, serde_json::Value> {
    let mut sources = BTreeMap::new();
    sources.insert(
        source_alias(&snapshot.initial_source).to_string(),
        source_json(&snapshot.initial_source),
    );
    for operation in &snapshot.operations {
        if let DatasetOperationRequest::AddSource { source, .. } = operation {
            sources.insert(source_alias(source).to_string(), source_json(source));
        }
    }
    sources
}

fn source_alias(source: &DatasetSourceRequest) -> &str {
    match source {
        DatasetSourceRequest::Form { alias, .. }
        | DatasetSourceRequest::Dataset { alias, .. }
        | DatasetSourceRequest::DatasetMajor { alias, .. } => alias,
    }
}

fn source_json(source: &DatasetSourceRequest) -> serde_json::Value {
    serde_json::to_value(source).unwrap_or(serde_json::Value::Null)
}

fn operation_changelog_findings(
    published: &DatasetRevisionSnapshot,
    candidate: &DatasetRevisionSnapshot,
    published_fields: &BTreeMap<&str, &DatasetFieldDefinition>,
    candidate_fields: &BTreeMap<&str, &DatasetFieldDefinition>,
) -> Vec<DatasetCompatibilityFinding> {
    let mut findings = Vec::new();
    let operation_count = published.operations.len().max(candidate.operations.len());

    for index in 0..operation_count {
        match (
            published.operations.get(index),
            candidate.operations.get(index),
        ) {
            (None, Some(DatasetOperationRequest::AddSource { .. })) => {}
            (None, Some(operation)) => findings.push(compatibility_finding(
                operation_version_impact(operation),
                &format!("added_{}_operation", operation_code(operation)),
                format!("{} operation is added.", operation_label(operation)),
                None,
            )),
            (Some(DatasetOperationRequest::AddSource { .. }), None) => {}
            (Some(operation), None) => findings.push(compatibility_finding(
                DatasetVersionImpact::Major,
                &format!("removed_{}_operation", operation_code(operation)),
                format!("{} operation is removed.", operation_label(operation)),
                None,
            )),
            (Some(published_operation), Some(candidate_operation))
                if operation_code(published_operation) != operation_code(candidate_operation) =>
            {
                findings.push(compatibility_finding(
                    DatasetVersionImpact::Minor,
                    "changed_operation_sequence",
                    format!(
                        "Operation {} changes from {} to {}.",
                        index + 1,
                        operation_label(published_operation),
                        operation_label(candidate_operation)
                    ),
                    None,
                ));
            }
            (Some(published_operation), Some(candidate_operation))
                if operation_json(published_operation) != operation_json(candidate_operation) =>
            {
                let mut detailed_findings = detailed_operation_changelog_findings(
                    published_operation,
                    candidate_operation,
                    published_fields,
                    candidate_fields,
                );
                if detailed_findings.is_empty() {
                    detailed_findings.push(compatibility_finding(
                        operation_version_impact(candidate_operation),
                        &format!("changed_{}_operation", operation_code(candidate_operation)),
                        format!(
                            "{} operation settings change.",
                            operation_label(candidate_operation)
                        ),
                        None,
                    ));
                }
                findings.extend(detailed_findings);
            }
            _ => {}
        }
    }

    findings
}

fn detailed_operation_changelog_findings(
    published: &DatasetOperationRequest,
    candidate: &DatasetOperationRequest,
    published_output_fields: &BTreeMap<&str, &DatasetFieldDefinition>,
    candidate_output_fields: &BTreeMap<&str, &DatasetFieldDefinition>,
) -> Vec<DatasetCompatibilityFinding> {
    match (published, candidate) {
        (
            DatasetOperationRequest::Aggregation {
                group_fields: published_group_fields,
                metrics: published_metrics,
                row_picker: published_row_picker,
                ..
            },
            DatasetOperationRequest::Aggregation {
                group_fields: candidate_group_fields,
                metrics: candidate_metrics,
                row_picker: candidate_row_picker,
                ..
            },
        ) => aggregation_changelog_findings(
            published_group_fields,
            published_metrics,
            published_row_picker,
            candidate_group_fields,
            candidate_metrics,
            candidate_row_picker,
        ),
        (
            DatasetOperationRequest::CalculatedFields {
                fields: published_fields,
                ..
            },
            DatasetOperationRequest::CalculatedFields {
                fields: candidate_fields,
                ..
            },
        ) => calculated_fields_changelog_findings(
            published_fields,
            candidate_fields,
            published_output_fields,
            candidate_output_fields,
        ),
        _ => Vec::new(),
    }
}

fn aggregation_changelog_findings(
    published_group_fields: &[String],
    published_metrics: &[DatasetAggregationMetricRequest],
    published_row_picker: &Option<DatasetRowPickerRequest>,
    candidate_group_fields: &[String],
    candidate_metrics: &[DatasetAggregationMetricRequest],
    candidate_row_picker: &Option<DatasetRowPickerRequest>,
) -> Vec<DatasetCompatibilityFinding> {
    let mut findings = Vec::new();
    if published_group_fields != candidate_group_fields {
        findings.push(compatibility_finding(
            DatasetVersionImpact::Minor,
            "changed_aggregation_grouping",
            "Aggregation grouping changes.".into(),
            None,
        ));
    }
    if serde_json::to_value(published_row_picker).ok()
        != serde_json::to_value(candidate_row_picker).ok()
    {
        findings.push(compatibility_finding(
            DatasetVersionImpact::Minor,
            "changed_aggregation_row_picker",
            "Aggregation row picker changes.".into(),
            None,
        ));
    }

    let published_by_key = published_metrics
        .iter()
        .map(|metric| (metric.key.as_str(), metric))
        .collect::<BTreeMap<_, _>>();
    let candidate_by_key = candidate_metrics
        .iter()
        .map(|metric| (metric.key.as_str(), metric))
        .collect::<BTreeMap<_, _>>();

    for (key, metric) in &published_by_key {
        match candidate_by_key.get(key) {
            None => findings.push(compatibility_finding(
                DatasetVersionImpact::Major,
                "removed_aggregation_metric",
                format!("Aggregation metric '{}' is removed.", metric.label),
                Some((*key).into()),
            )),
            Some(candidate_metric) => {
                if metric.function != candidate_metric.function {
                    findings.push(compatibility_finding(
                        DatasetVersionImpact::Minor,
                        "changed_aggregation_metric_function",
                        format!(
                            "Aggregation metric '{}' changes function from '{}' to '{}'.",
                            metric.label, metric.function, candidate_metric.function
                        ),
                        Some((*key).into()),
                    ));
                }
                if metric.source_field_key != candidate_metric.source_field_key {
                    findings.push(compatibility_finding(
                        DatasetVersionImpact::Minor,
                        "changed_aggregation_metric_source",
                        format!(
                            "Aggregation metric '{}' changes source field.",
                            metric.label
                        ),
                        Some((*key).into()),
                    ));
                }
            }
        }
    }
    for (key, metric) in &candidate_by_key {
        if !published_by_key.contains_key(key) {
            findings.push(compatibility_finding(
                DatasetVersionImpact::Minor,
                "added_aggregation_metric",
                format!("Aggregation metric '{}' is added.", metric.label),
                Some((*key).into()),
            ));
        }
    }

    findings
}

fn calculated_fields_changelog_findings(
    published_fields: &[DatasetCalculatedFieldRequest],
    candidate_fields: &[DatasetCalculatedFieldRequest],
    published_output_fields: &BTreeMap<&str, &DatasetFieldDefinition>,
    candidate_output_fields: &BTreeMap<&str, &DatasetFieldDefinition>,
) -> Vec<DatasetCompatibilityFinding> {
    let mut findings = Vec::new();
    let published_by_key = published_fields
        .iter()
        .map(|field| (field.key.as_str(), field))
        .collect::<BTreeMap<_, _>>();
    let candidate_by_key = candidate_fields
        .iter()
        .map(|field| (field.key.as_str(), field))
        .collect::<BTreeMap<_, _>>();

    for (key, field) in &published_by_key {
        match candidate_by_key.get(key) {
            None => findings.push(compatibility_finding(
                DatasetVersionImpact::Major,
                "removed_calculated_field",
                format!("Calculated field '{}' is removed.", field.label),
                Some((*key).into()),
            )),
            Some(candidate_field) => {
                if field.base_field_key != candidate_field.base_field_key {
                    findings.push(compatibility_finding(
                        DatasetVersionImpact::Patch,
                        "changed_calculated_field_base",
                        format!("Calculated field '{}' changes base field.", field.label),
                        Some((*key).into()),
                    ));
                }
                if let (Some(published_output), Some(candidate_output)) = (
                    published_output_fields.get(key),
                    candidate_output_fields.get(key),
                ) && published_output.field_type != candidate_output.field_type
                {
                    findings.push(compatibility_finding(
                        DatasetVersionImpact::Major,
                        "changed_calculated_field_type",
                        format!(
                            "Calculated field '{}' changes type from '{}' to '{}'.",
                            field.label, published_output.field_type, candidate_output.field_type
                        ),
                        Some((*key).into()),
                    ));
                }
                findings.extend(calculation_function_changelog_findings(
                    field,
                    candidate_field,
                ));
            }
        }
    }
    for (key, field) in &candidate_by_key {
        if !published_by_key.contains_key(key) {
            findings.push(compatibility_finding(
                DatasetVersionImpact::Minor,
                "added_calculated_field",
                format!("Calculated field '{}' is added.", field.label),
                Some((*key).into()),
            ));
        }
    }

    findings
}

fn calculation_function_changelog_findings(
    published: &DatasetCalculatedFieldRequest,
    candidate: &DatasetCalculatedFieldRequest,
) -> Vec<DatasetCompatibilityFinding> {
    let mut findings = Vec::new();
    let function_count = published.functions.len().max(candidate.functions.len());

    for index in 0..function_count {
        match (
            published.functions.get(index),
            candidate.functions.get(index),
        ) {
            (Some(function), None) => findings.push(compatibility_finding(
                DatasetVersionImpact::Patch,
                "removed_calculation_function",
                format!(
                    "Calculated field '{}' removes function '{}'.",
                    published.label, function.function
                ),
                Some(published.key.clone()),
            )),
            (None, Some(function)) => findings.push(compatibility_finding(
                DatasetVersionImpact::Patch,
                "added_calculation_function",
                format!(
                    "Calculated field '{}' adds function '{}'.",
                    published.label, function.function
                ),
                Some(published.key.clone()),
            )),
            (Some(published_function), Some(candidate_function)) => {
                if published_function.function != candidate_function.function {
                    findings.push(compatibility_finding(
                        DatasetVersionImpact::Patch,
                        "changed_calculation_function",
                        format!(
                            "Calculated field '{}' changes function from '{}' to '{}'.",
                            published.label,
                            published_function.function,
                            candidate_function.function
                        ),
                        Some(published.key.clone()),
                    ));
                }
                if published_function.argument != candidate_function.argument
                    || published_function.argument_mode != candidate_function.argument_mode
                    || published_function.argument_field_key
                        != candidate_function.argument_field_key
                {
                    findings.push(compatibility_finding(
                        DatasetVersionImpact::Patch,
                        "changed_calculation_function_argument",
                        format!(
                            "Calculated field '{}' changes the '{}' function argument.",
                            published.label, candidate_function.function
                        ),
                        Some(published.key.clone()),
                    ));
                }
            }
            _ => {}
        }
    }

    findings
}

fn operation_json(operation: &DatasetOperationRequest) -> serde_json::Value {
    serde_json::to_value(operation).unwrap_or(serde_json::Value::Null)
}

fn operation_code(operation: &DatasetOperationRequest) -> &'static str {
    match operation {
        DatasetOperationRequest::AddSource { .. } => "add_source",
        DatasetOperationRequest::Projection { .. } => "projection",
        DatasetOperationRequest::Aggregation { .. } => "aggregation",
        DatasetOperationRequest::CalculatedFields { .. } => "calculated_fields",
        DatasetOperationRequest::Filter { .. } => "filter",
    }
}

fn operation_label(operation: &DatasetOperationRequest) -> &'static str {
    match operation {
        DatasetOperationRequest::AddSource { .. } => "Add source",
        DatasetOperationRequest::Projection { .. } => "Projection",
        DatasetOperationRequest::Aggregation { .. } => "Aggregation",
        DatasetOperationRequest::CalculatedFields { .. } => "Calculated fields",
        DatasetOperationRequest::Filter { .. } => "Filter",
    }
}

fn operation_version_impact(operation: &DatasetOperationRequest) -> DatasetVersionImpact {
    match operation {
        DatasetOperationRequest::AddSource { .. } | DatasetOperationRequest::Aggregation { .. } => {
            DatasetVersionImpact::Minor
        }
        DatasetOperationRequest::Projection { .. }
        | DatasetOperationRequest::CalculatedFields { .. }
        | DatasetOperationRequest::Filter { .. } => DatasetVersionImpact::Patch,
    }
}

pub(super) fn compatibility_finding(
    version_impact: DatasetVersionImpact,
    code: &str,
    message: String,
    field_key: Option<String>,
) -> DatasetCompatibilityFinding {
    let state = match version_impact {
        DatasetVersionImpact::Major => DatasetCompatibilityState::Breaking,
        DatasetVersionImpact::Minor => DatasetCompatibilityState::Review,
        DatasetVersionImpact::Patch => DatasetCompatibilityState::Compatible,
    };
    DatasetCompatibilityFinding {
        version_impact,
        state,
        code: code.into(),
        message,
        field_key,
    }
}

pub(super) fn compatibility_summary(
    findings: &[DatasetCompatibilityFinding],
) -> DatasetCompatibilitySummary {
    let major_count = findings
        .iter()
        .filter(|finding| finding.version_impact == DatasetVersionImpact::Major)
        .count();
    let minor_count = findings
        .iter()
        .filter(|finding| finding.version_impact == DatasetVersionImpact::Minor)
        .count();
    let patch_count = findings
        .iter()
        .filter(|finding| finding.version_impact == DatasetVersionImpact::Patch)
        .count();
    let state = if major_count > 0 {
        DatasetCompatibilityState::Breaking
    } else if minor_count > 0 {
        DatasetCompatibilityState::Review
    } else {
        DatasetCompatibilityState::Compatible
    };
    DatasetCompatibilitySummary {
        state,
        major_count,
        minor_count,
        patch_count,
    }
}

fn restriction_policy_json(
    policy: &Option<DatasetRestrictionPolicyRequest>,
) -> Option<serde_json::Value> {
    policy
        .as_ref()
        .and_then(|policy| serde_json::to_value(policy).ok())
}

pub(super) async fn load_dependency_impacts(
    pool: &sqlx::PgPool,
    scope: &DependencyImpactScope,
    pinned_revision_id: Uuid,
    source_dataset_id: Uuid,
    current_version_major: Option<i32>,
    candidate_version_major: Option<i32>,
    compatibility_state: DatasetCompatibilityState,
) -> ApiResult<Vec<DatasetDependencyImpact>> {
    let carry_forward_state = carry_forward_state_for(compatibility_state);
    let message = carry_forward_message(carry_forward_state).to_string();
    let mut impacts = Vec::new();

    let dataset_rows = match &scope.datasets {
        auth::CapabilityBoundary::Global => {
            sqlx::query(
                r#"
        SELECT datasets.id, datasets.name
        FROM dataset_sources
        JOIN datasets ON datasets.id = dataset_sources.dataset_id
        WHERE dataset_sources.dataset_revision_id = $1
        ORDER BY datasets.name
        "#,
            )
            .bind(pinned_revision_id)
            .fetch_all(pool)
            .await?
        }
        auth::CapabilityBoundary::Scoped(scope_ids) => {
            sqlx::query(
                r#"
        SELECT DISTINCT datasets.id, datasets.name
        FROM dataset_sources
        JOIN datasets ON datasets.id = dataset_sources.dataset_id
        JOIN dataset_scope_nodes ON dataset_scope_nodes.dataset_id = datasets.id
        WHERE dataset_sources.dataset_revision_id = $1
          AND dataset_scope_nodes.node_id = ANY($2)
        ORDER BY datasets.name
        "#,
            )
            .bind(pinned_revision_id)
            .bind(scope_ids)
            .fetch_all(pool)
            .await?
        }
        auth::CapabilityBoundary::None => Vec::new(),
    };
    for row in dataset_rows {
        impacts.push(DatasetDependencyImpact {
            kind: DatasetDependencyKind::Dataset,
            id: row.try_get("id")?,
            name: row.try_get("name")?,
            pinned_revision_id,
            pinned_version_major: None,
            binding_mode: DatasetDependencyBindingMode::ExactRevision,
            carry_forward_state,
            message: message.clone(),
        });
    }

    if let Some(current_major) = current_version_major {
        let major_line_message = if candidate_version_major == Some(current_major) {
            "Major-line dependency is bound to this Version and will receive compatible minor/patch rows after publish."
                .to_string()
        } else {
            "Major-line dependency remains on its current Version and needs review before moving to a new major version."
                .to_string()
        };
        let major_line_state = if candidate_version_major == Some(current_major) {
            DatasetCarryForwardState::Safe
        } else {
            DatasetCarryForwardState::ManualReview
        };
        let major_line_rows = match &scope.datasets {
            auth::CapabilityBoundary::Global => {
                sqlx::query(
                    r#"
            SELECT datasets.id, datasets.name
            FROM dataset_sources
            JOIN datasets ON datasets.id = dataset_sources.dataset_id
            WHERE dataset_sources.source_dataset_id = $1
              AND dataset_sources.dataset_version_major = $2
            ORDER BY datasets.name
            "#,
                )
                .bind(source_dataset_id)
                .bind(current_major)
                .fetch_all(pool)
                .await?
            }
            auth::CapabilityBoundary::Scoped(scope_ids) => {
                sqlx::query(
                    r#"
            SELECT DISTINCT datasets.id, datasets.name
            FROM dataset_sources
            JOIN datasets ON datasets.id = dataset_sources.dataset_id
            JOIN dataset_scope_nodes ON dataset_scope_nodes.dataset_id = datasets.id
            WHERE dataset_sources.source_dataset_id = $1
              AND dataset_sources.dataset_version_major = $2
              AND dataset_scope_nodes.node_id = ANY($3)
            ORDER BY datasets.name
            "#,
                )
                .bind(source_dataset_id)
                .bind(current_major)
                .bind(scope_ids)
                .fetch_all(pool)
                .await?
            }
            auth::CapabilityBoundary::None => Vec::new(),
        };
        for row in major_line_rows {
            impacts.push(DatasetDependencyImpact {
                kind: DatasetDependencyKind::Dataset,
                id: row.try_get("id")?,
                name: row.try_get("name")?,
                pinned_revision_id,
                pinned_version_major: Some(current_major),
                binding_mode: DatasetDependencyBindingMode::MajorLine,
                carry_forward_state: major_line_state,
                message: major_line_message.clone(),
            });
        }
    }

    let component_rows = match &scope.components {
        auth::CapabilityBoundary::Global => {
            sqlx::query(
                r#"
        SELECT component_versions.id,
               components.name || ' ' || component_versions.version_label AS name
        FROM component_versions
        JOIN components ON components.id = component_versions.component_id
        WHERE component_versions.dataset_revision_id = $1
        ORDER BY components.name, component_versions.version_number
        "#,
            )
            .bind(pinned_revision_id)
            .fetch_all(pool)
            .await?
        }
        auth::CapabilityBoundary::Scoped(scope_ids) => {
            sqlx::query(
                r#"
        SELECT DISTINCT component_versions.id,
               components.name || ' ' || component_versions.version_label AS name
        FROM component_versions
        JOIN components ON components.id = component_versions.component_id
        JOIN dataset_revisions ON dataset_revisions.id = component_versions.dataset_revision_id
        JOIN dataset_scope_nodes ON dataset_scope_nodes.dataset_id = dataset_revisions.dataset_id
        WHERE component_versions.dataset_revision_id = $1
          AND dataset_scope_nodes.node_id = ANY($2)
        ORDER BY name
        "#,
            )
            .bind(pinned_revision_id)
            .bind(scope_ids)
            .fetch_all(pool)
            .await?
        }
        auth::CapabilityBoundary::None => Vec::new(),
    };
    for row in component_rows {
        impacts.push(DatasetDependencyImpact {
            kind: DatasetDependencyKind::ComponentVersion,
            id: row.try_get("id")?,
            name: row.try_get("name")?,
            pinned_revision_id,
            pinned_version_major: None,
            binding_mode: DatasetDependencyBindingMode::ExactRevision,
            carry_forward_state,
            message: message.clone(),
        });
    }

    let dashboard_rows = match &scope.dashboards {
        auth::CapabilityBoundary::Global => {
            sqlx::query(
                r#"
        SELECT dashboards.id, dashboards.name
        FROM dashboard_components
        JOIN dashboards ON dashboards.id = dashboard_components.dashboard_id
        JOIN component_versions ON component_versions.id = dashboard_components.component_version_id
        WHERE component_versions.dataset_revision_id = $1
        ORDER BY dashboards.name
        "#,
            )
            .bind(pinned_revision_id)
            .fetch_all(pool)
            .await?
        }
        auth::CapabilityBoundary::Scoped(scope_ids) => {
            sqlx::query(
                r#"
        SELECT DISTINCT dashboards.id, dashboards.name
        FROM dashboard_components
        JOIN dashboards ON dashboards.id = dashboard_components.dashboard_id
        JOIN dashboard_scope_nodes ON dashboard_scope_nodes.dashboard_id = dashboards.id
        JOIN component_versions ON component_versions.id = dashboard_components.component_version_id
        WHERE component_versions.dataset_revision_id = $1
          AND dashboard_scope_nodes.node_id = ANY($2)
        ORDER BY dashboards.name
        "#,
            )
            .bind(pinned_revision_id)
            .bind(scope_ids)
            .fetch_all(pool)
            .await?
        }
        auth::CapabilityBoundary::None => Vec::new(),
    };
    for row in dashboard_rows {
        impacts.push(DatasetDependencyImpact {
            kind: DatasetDependencyKind::Dashboard,
            id: row.try_get("id")?,
            name: row.try_get("name")?,
            pinned_revision_id,
            pinned_version_major: None,
            binding_mode: DatasetDependencyBindingMode::ExactRevision,
            carry_forward_state,
            message: message.clone(),
        });
    }

    Ok(impacts)
}

pub(super) struct DependencyImpactScope {
    pub(super) datasets: auth::CapabilityBoundary,
    pub(super) components: auth::CapabilityBoundary,
    pub(super) dashboards: auth::CapabilityBoundary,
}

pub(super) fn dependency_summary(impacts: &[DatasetDependencyImpact]) -> DatasetDependencySummary {
    let dataset_count = impacts
        .iter()
        .filter(|impact| impact.kind == DatasetDependencyKind::Dataset)
        .count();
    let component_version_count = impacts
        .iter()
        .filter(|impact| impact.kind == DatasetDependencyKind::ComponentVersion)
        .count();
    let dashboard_count = impacts
        .iter()
        .filter(|impact| impact.kind == DatasetDependencyKind::Dashboard)
        .count();
    let carry_forward_state = if impacts
        .iter()
        .any(|impact| impact.carry_forward_state == DatasetCarryForwardState::Blocked)
    {
        DatasetCarryForwardState::Blocked
    } else if impacts
        .iter()
        .any(|impact| impact.carry_forward_state == DatasetCarryForwardState::ManualReview)
    {
        DatasetCarryForwardState::ManualReview
    } else {
        DatasetCarryForwardState::Safe
    };
    DatasetDependencySummary {
        dependency_count: impacts.len(),
        dataset_count,
        component_version_count,
        dashboard_count,
        carry_forward_state,
    }
}

pub(super) fn carry_forward_state_for(
    compatibility_state: DatasetCompatibilityState,
) -> DatasetCarryForwardState {
    match compatibility_state {
        DatasetCompatibilityState::Compatible => DatasetCarryForwardState::Safe,
        DatasetCompatibilityState::Review => DatasetCarryForwardState::ManualReview,
        DatasetCompatibilityState::Breaking => DatasetCarryForwardState::Blocked,
    }
}

fn carry_forward_message(state: DatasetCarryForwardState) -> &'static str {
    match state {
        DatasetCarryForwardState::Safe => {
            "Dependency remains pinned; carry-forward appears safe to review."
        }
        DatasetCarryForwardState::ManualReview => {
            "Dependency remains pinned; carry-forward needs manual review."
        }
        DatasetCarryForwardState::Blocked => {
            "Dependency remains pinned; carry-forward is blocked by breaking findings."
        }
    }
}
