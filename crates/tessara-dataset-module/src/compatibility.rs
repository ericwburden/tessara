//! Dataset-owned authoring compatibility classification.

use std::collections::{BTreeMap, BTreeSet};

use tessara_datasets_contract::{
    DatasetProductAggregationMetricV1, DatasetProductCalculatedFieldV1,
    DatasetProductCompatibilityFindingV1, DatasetProductCompatibilityStateV1,
    DatasetProductCompatibilitySummaryV1, DatasetProductFieldV1, DatasetProductOperationV1,
    DatasetProductRestrictionPolicyV1, DatasetProductRevisionMetadataV1, DatasetProductRowPickerV1,
    DatasetProductSemanticBumpV1, DatasetProductSourceV1, DatasetProductVersionImpactV1,
};

use crate::DatasetModuleError;

/// The canonical authoring state needed to classify a revision changelog.
///
/// This intentionally contains only Dataset-owned contract values. Persistence
/// rows are decoded into this shape before comparison so storage details cannot
/// become a second compatibility model.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct DatasetCompatibilitySnapshot {
    pub(crate) metadata: DatasetProductRevisionMetadataV1,
    pub(crate) initial_source: DatasetProductSourceV1,
    pub(crate) operations: Vec<DatasetProductOperationV1>,
    pub(crate) restriction_policy: Option<DatasetProductRestrictionPolicyV1>,
    pub(crate) output_fields: Vec<DatasetProductFieldV1>,
}

pub(crate) fn compatibility_findings(
    published: &DatasetCompatibilitySnapshot,
    candidate: &DatasetCompatibilitySnapshot,
) -> Vec<DatasetProductCompatibilityFindingV1> {
    let mut findings = Vec::new();
    let published_fields = fields_by_key(&published.output_fields);
    let candidate_fields = fields_by_key(&candidate.output_fields);

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
                DatasetProductVersionImpactV1::Major,
                "removed_output_field",
                format!("Output field '{}' is removed.", field.label),
                Some((*key).to_owned()),
            )),
            Some(candidate_field) if candidate_field.field_type != field.field_type => {
                findings.push(compatibility_finding(
                    DatasetProductVersionImpactV1::Major,
                    "changed_output_field_type",
                    format!(
                        "Output field '{}' changes type from '{}' to '{}'.",
                        field.label, field.field_type, candidate_field.field_type
                    ),
                    Some((*key).to_owned()),
                ));
            }
            Some(candidate_field) if candidate_field.label != field.label => {
                findings.push(compatibility_finding(
                    DatasetProductVersionImpactV1::Patch,
                    "changed_output_field_label",
                    format!(
                        "Output field key '{}' changes label from '{}' to '{}'.",
                        key, field.label, candidate_field.label
                    ),
                    Some((*key).to_owned()),
                ));
            }
            _ => {}
        }
    }
    for (key, field) in &candidate_fields {
        if !published_fields.contains_key(key) {
            findings.push(compatibility_finding(
                DatasetProductVersionImpactV1::Minor,
                "added_output_field",
                format!("Output field '{}' is added.", field.label),
                Some((*key).to_owned()),
            ));
        }
    }

    if published.restriction_policy != candidate.restriction_policy {
        findings.push(compatibility_finding(
            DatasetProductVersionImpactV1::Minor,
            "changed_restriction_policy",
            "Restriction policy changes and should be reviewed before carry-forward.".into(),
            None,
        ));
    }
    if published.metadata.name != candidate.metadata.name {
        findings.push(compatibility_finding(
            DatasetProductVersionImpactV1::Patch,
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
            DatasetProductVersionImpactV1::Patch,
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
            DatasetProductVersionImpactV1::Patch,
            "changed_dataset_visibility",
            "Dataset visibility scope changes.".into(),
            None,
        ));
    }

    findings
}

fn fields_by_key(fields: &[DatasetProductFieldV1]) -> BTreeMap<&str, &DatasetProductFieldV1> {
    fields
        .iter()
        .map(|field| (field.key.as_str(), field))
        .collect()
}

fn source_changelog_findings(
    published: &DatasetCompatibilitySnapshot,
    candidate: &DatasetCompatibilitySnapshot,
) -> Vec<DatasetProductCompatibilityFindingV1> {
    let published_sources = revision_sources_by_alias(published);
    let candidate_sources = revision_sources_by_alias(candidate);
    let mut findings = Vec::new();

    for (alias, source) in &published_sources {
        match candidate_sources.get(alias) {
            None => findings.push(compatibility_finding(
                DatasetProductVersionImpactV1::Major,
                "removed_dataset_source",
                format!("Source '{alias}' is removed."),
                Some(alias.clone()),
            )),
            Some(candidate_source) if candidate_source != source => {
                findings.push(compatibility_finding(
                    DatasetProductVersionImpactV1::Minor,
                    "changed_dataset_source",
                    format!("Source '{alias}' changes binding."),
                    Some(alias.clone()),
                ));
            }
            _ => {}
        }
    }
    for alias in candidate_sources.keys() {
        if !published_sources.contains_key(alias) {
            findings.push(compatibility_finding(
                DatasetProductVersionImpactV1::Minor,
                "added_dataset_source",
                format!("Source '{alias}' is added."),
                Some(alias.clone()),
            ));
        }
    }

    findings
}

fn revision_sources_by_alias(
    snapshot: &DatasetCompatibilitySnapshot,
) -> BTreeMap<String, DatasetProductSourceV1> {
    let mut sources = BTreeMap::new();
    sources.insert(
        source_alias(&snapshot.initial_source).to_owned(),
        snapshot.initial_source.clone(),
    );
    for operation in &snapshot.operations {
        if let DatasetProductOperationV1::AddSource { source, .. } = operation {
            sources.insert(source_alias(source).to_owned(), source.clone());
        }
    }
    sources
}

fn source_alias(source: &DatasetProductSourceV1) -> &str {
    match source {
        DatasetProductSourceV1::Form { alias, .. }
        | DatasetProductSourceV1::Dataset { alias, .. }
        | DatasetProductSourceV1::DatasetMajor { alias, .. } => alias,
    }
}

fn operation_changelog_findings(
    published: &DatasetCompatibilitySnapshot,
    candidate: &DatasetCompatibilitySnapshot,
    published_fields: &BTreeMap<&str, &DatasetProductFieldV1>,
    candidate_fields: &BTreeMap<&str, &DatasetProductFieldV1>,
) -> Vec<DatasetProductCompatibilityFindingV1> {
    let mut findings = Vec::new();
    let operation_count = published.operations.len().max(candidate.operations.len());

    for index in 0..operation_count {
        match (
            published.operations.get(index),
            candidate.operations.get(index),
        ) {
            (None, Some(DatasetProductOperationV1::AddSource { .. })) => {}
            (None, Some(operation)) => findings.push(compatibility_finding(
                operation_version_impact(operation),
                &format!("added_{}_operation", operation_code(operation)),
                format!("{} operation is added.", operation_label(operation)),
                None,
            )),
            (Some(DatasetProductOperationV1::AddSource { .. }), None) => {}
            (Some(operation), None) => findings.push(compatibility_finding(
                DatasetProductVersionImpactV1::Major,
                &format!("removed_{}_operation", operation_code(operation)),
                format!("{} operation is removed.", operation_label(operation)),
                None,
            )),
            (Some(published_operation), Some(candidate_operation))
                if operation_code(published_operation) != operation_code(candidate_operation) =>
            {
                findings.push(compatibility_finding(
                    DatasetProductVersionImpactV1::Minor,
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
                if published_operation != candidate_operation =>
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
    published: &DatasetProductOperationV1,
    candidate: &DatasetProductOperationV1,
    published_output_fields: &BTreeMap<&str, &DatasetProductFieldV1>,
    candidate_output_fields: &BTreeMap<&str, &DatasetProductFieldV1>,
) -> Vec<DatasetProductCompatibilityFindingV1> {
    match (published, candidate) {
        (
            DatasetProductOperationV1::Aggregation {
                group_fields: published_group_fields,
                metrics: published_metrics,
                row_picker: published_row_picker,
                ..
            },
            DatasetProductOperationV1::Aggregation {
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
            DatasetProductOperationV1::CalculatedFields {
                fields: published_fields,
                ..
            },
            DatasetProductOperationV1::CalculatedFields {
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
    published_metrics: &[DatasetProductAggregationMetricV1],
    published_row_picker: &Option<DatasetProductRowPickerV1>,
    candidate_group_fields: &[String],
    candidate_metrics: &[DatasetProductAggregationMetricV1],
    candidate_row_picker: &Option<DatasetProductRowPickerV1>,
) -> Vec<DatasetProductCompatibilityFindingV1> {
    let mut findings = Vec::new();
    if published_group_fields != candidate_group_fields {
        findings.push(compatibility_finding(
            DatasetProductVersionImpactV1::Minor,
            "changed_aggregation_grouping",
            "Aggregation grouping changes.".into(),
            None,
        ));
    }
    if published_row_picker != candidate_row_picker {
        findings.push(compatibility_finding(
            DatasetProductVersionImpactV1::Minor,
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
                DatasetProductVersionImpactV1::Major,
                "removed_aggregation_metric",
                format!("Aggregation metric '{}' is removed.", metric.label),
                Some((*key).to_owned()),
            )),
            Some(candidate_metric) => {
                if metric.function != candidate_metric.function {
                    findings.push(compatibility_finding(
                        DatasetProductVersionImpactV1::Minor,
                        "changed_aggregation_metric_function",
                        format!(
                            "Aggregation metric '{}' changes function from '{}' to '{}'.",
                            metric.label, metric.function, candidate_metric.function
                        ),
                        Some((*key).to_owned()),
                    ));
                }
                if metric.source_field_key != candidate_metric.source_field_key {
                    findings.push(compatibility_finding(
                        DatasetProductVersionImpactV1::Minor,
                        "changed_aggregation_metric_source",
                        format!(
                            "Aggregation metric '{}' changes source field.",
                            metric.label
                        ),
                        Some((*key).to_owned()),
                    ));
                }
            }
        }
    }
    for (key, metric) in &candidate_by_key {
        if !published_by_key.contains_key(key) {
            findings.push(compatibility_finding(
                DatasetProductVersionImpactV1::Minor,
                "added_aggregation_metric",
                format!("Aggregation metric '{}' is added.", metric.label),
                Some((*key).to_owned()),
            ));
        }
    }

    findings
}

fn calculated_fields_changelog_findings(
    published_fields: &[DatasetProductCalculatedFieldV1],
    candidate_fields: &[DatasetProductCalculatedFieldV1],
    published_output_fields: &BTreeMap<&str, &DatasetProductFieldV1>,
    candidate_output_fields: &BTreeMap<&str, &DatasetProductFieldV1>,
) -> Vec<DatasetProductCompatibilityFindingV1> {
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
                DatasetProductVersionImpactV1::Major,
                "removed_calculated_field",
                format!("Calculated field '{}' is removed.", field.label),
                Some((*key).to_owned()),
            )),
            Some(candidate_field) => {
                if field.base_field_key != candidate_field.base_field_key {
                    findings.push(compatibility_finding(
                        DatasetProductVersionImpactV1::Patch,
                        "changed_calculated_field_base",
                        format!("Calculated field '{}' changes base field.", field.label),
                        Some((*key).to_owned()),
                    ));
                }
                if let (Some(published_output), Some(candidate_output)) = (
                    published_output_fields.get(key),
                    candidate_output_fields.get(key),
                ) && published_output.field_type != candidate_output.field_type
                {
                    findings.push(compatibility_finding(
                        DatasetProductVersionImpactV1::Major,
                        "changed_calculated_field_type",
                        format!(
                            "Calculated field '{}' changes type from '{}' to '{}'.",
                            field.label, published_output.field_type, candidate_output.field_type
                        ),
                        Some((*key).to_owned()),
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
                DatasetProductVersionImpactV1::Minor,
                "added_calculated_field",
                format!("Calculated field '{}' is added.", field.label),
                Some((*key).to_owned()),
            ));
        }
    }

    findings
}

fn calculation_function_changelog_findings(
    published: &DatasetProductCalculatedFieldV1,
    candidate: &DatasetProductCalculatedFieldV1,
) -> Vec<DatasetProductCompatibilityFindingV1> {
    let mut findings = Vec::new();
    let function_count = published.functions.len().max(candidate.functions.len());

    for index in 0..function_count {
        match (
            published.functions.get(index),
            candidate.functions.get(index),
        ) {
            (Some(function), None) => findings.push(compatibility_finding(
                DatasetProductVersionImpactV1::Patch,
                "removed_calculation_function",
                format!(
                    "Calculated field '{}' removes function '{}'.",
                    published.label, function.function
                ),
                Some(published.key.clone()),
            )),
            (None, Some(function)) => findings.push(compatibility_finding(
                DatasetProductVersionImpactV1::Patch,
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
                        DatasetProductVersionImpactV1::Patch,
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
                        DatasetProductVersionImpactV1::Patch,
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

fn operation_code(operation: &DatasetProductOperationV1) -> &'static str {
    match operation {
        DatasetProductOperationV1::AddSource { .. } => "add_source",
        DatasetProductOperationV1::Projection { .. } => "projection",
        DatasetProductOperationV1::Aggregation { .. } => "aggregation",
        DatasetProductOperationV1::CalculatedFields { .. } => "calculated_fields",
        DatasetProductOperationV1::Filter { .. } => "filter",
    }
}

fn operation_label(operation: &DatasetProductOperationV1) -> &'static str {
    match operation {
        DatasetProductOperationV1::AddSource { .. } => "Add source",
        DatasetProductOperationV1::Projection { .. } => "Projection",
        DatasetProductOperationV1::Aggregation { .. } => "Aggregation",
        DatasetProductOperationV1::CalculatedFields { .. } => "Calculated fields",
        DatasetProductOperationV1::Filter { .. } => "Filter",
    }
}

fn operation_version_impact(
    operation: &DatasetProductOperationV1,
) -> DatasetProductVersionImpactV1 {
    match operation {
        DatasetProductOperationV1::AddSource { .. }
        | DatasetProductOperationV1::Aggregation { .. } => DatasetProductVersionImpactV1::Minor,
        DatasetProductOperationV1::Projection { .. }
        | DatasetProductOperationV1::CalculatedFields { .. }
        | DatasetProductOperationV1::Filter { .. } => DatasetProductVersionImpactV1::Patch,
    }
}

pub(crate) fn compatibility_finding(
    version_impact: DatasetProductVersionImpactV1,
    code: &str,
    message: String,
    field_key: Option<String>,
) -> DatasetProductCompatibilityFindingV1 {
    let state = match version_impact {
        DatasetProductVersionImpactV1::Major => DatasetProductCompatibilityStateV1::Breaking,
        DatasetProductVersionImpactV1::Minor => DatasetProductCompatibilityStateV1::Review,
        DatasetProductVersionImpactV1::Patch => DatasetProductCompatibilityStateV1::Compatible,
    };
    DatasetProductCompatibilityFindingV1 {
        version_impact,
        state,
        code: code.into(),
        message,
        field_key,
    }
}

pub(crate) fn compatibility_summary(
    findings: &[DatasetProductCompatibilityFindingV1],
) -> DatasetProductCompatibilitySummaryV1 {
    let major_count = findings
        .iter()
        .filter(|finding| finding.version_impact == DatasetProductVersionImpactV1::Major)
        .count();
    let minor_count = findings
        .iter()
        .filter(|finding| finding.version_impact == DatasetProductVersionImpactV1::Minor)
        .count();
    let patch_count = findings
        .iter()
        .filter(|finding| finding.version_impact == DatasetProductVersionImpactV1::Patch)
        .count();
    let state = if major_count > 0 {
        DatasetProductCompatibilityStateV1::Breaking
    } else if minor_count > 0 {
        DatasetProductCompatibilityStateV1::Review
    } else {
        DatasetProductCompatibilityStateV1::Compatible
    };
    DatasetProductCompatibilitySummaryV1 {
        state,
        major_count,
        minor_count,
        patch_count,
    }
}

pub(crate) fn semantic_bump_for_publish(
    summary: &DatasetProductCompatibilitySummaryV1,
    force_new_major_version: bool,
) -> DatasetProductSemanticBumpV1 {
    if force_new_major_version || summary.major_count > 0 {
        DatasetProductSemanticBumpV1::Major
    } else if summary.minor_count > 0 {
        DatasetProductSemanticBumpV1::Minor
    } else {
        DatasetProductSemanticBumpV1::Patch
    }
}

pub(crate) fn require_publishable_changelog(
    findings: &[DatasetProductCompatibilityFindingV1],
) -> Result<(), DatasetModuleError> {
    if findings.is_empty() {
        return Err(DatasetModuleError::BadRequest(
            "Dataset revision has no changelog entries to publish".into(),
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use tessara_datasets_contract::{DatasetProductCalculationFunctionV1, DatasetProductJoinKeyV1};

    use super::*;

    #[test]
    fn changelog_marks_added_fields_as_minor() {
        let published = snapshot(vec![field("participant_id", "Participant ID", "text")]);
        let candidate = snapshot(vec![
            field("participant_id", "Participant ID", "text"),
            field("score", "Score", "number"),
        ]);

        let findings = compatibility_findings(&published, &candidate);
        let summary = compatibility_summary(&findings);

        assert_eq!(summary.minor_count, 1);
        assert_eq!(summary.state, DatasetProductCompatibilityStateV1::Review);
        assert!(findings.iter().any(|finding| {
            finding.code == "added_output_field"
                && finding.field_key.as_deref() == Some("score")
                && finding.version_impact == DatasetProductVersionImpactV1::Minor
        }));
    }

    #[test]
    fn changelog_marks_removed_and_type_changed_fields_as_major() {
        let published = snapshot(vec![
            field("participant_id", "Participant ID", "text"),
            field("score", "Score", "number"),
        ]);
        let candidate = snapshot(vec![field("participant_id", "Participant ID", "number")]);

        let findings = compatibility_findings(&published, &candidate);
        let summary = compatibility_summary(&findings);

        assert_eq!(summary.major_count, 2);
        assert_eq!(summary.state, DatasetProductCompatibilityStateV1::Breaking);
        assert!(findings.iter().any(|finding| {
            finding.code == "changed_output_field_type"
                && finding.field_key.as_deref() == Some("participant_id")
        }));
        assert!(findings.iter().any(|finding| {
            finding.code == "removed_output_field" && finding.field_key.as_deref() == Some("score")
        }));
    }

    #[test]
    fn changelog_marks_restriction_changes_as_minor() {
        let published = snapshot(vec![field("participant_id", "Participant ID", "text")]);
        let mut candidate = published.clone();
        candidate.restriction_policy = Some(DatasetProductRestrictionPolicyV1 {
            internal_field_key: Some("internal_flag".into()),
            restricted_field_key: None,
            confidential_field_key: None,
        });

        let findings = compatibility_findings(&published, &candidate);

        assert!(findings.iter().any(|finding| {
            finding.code == "changed_restriction_policy"
                && finding.version_impact == DatasetProductVersionImpactV1::Minor
        }));
    }

    #[test]
    fn changelog_marks_label_changes_as_patch() {
        let published = snapshot(vec![field("participant_id", "Participant ID", "text")]);
        let candidate = snapshot(vec![field(
            "participant_id",
            "Participant Identifier",
            "text",
        )]);

        let findings = compatibility_findings(&published, &candidate);

        assert!(findings.iter().any(|finding| {
            finding.code == "changed_output_field_label"
                && finding.version_impact == DatasetProductVersionImpactV1::Patch
        }));
    }

    #[test]
    fn changelog_marks_added_sources_as_minor() {
        let published = snapshot(Vec::new());
        let mut candidate = published.clone();
        candidate.operations = vec![DatasetProductOperationV1::AddSource {
            source: DatasetProductSourceV1::Form {
                alias: "source_2".into(),
                form_id: "20000000-0000-0000-0000-000000000001".into(),
                form_version_id: "20000000-0000-0000-0000-000000000002".into(),
            },
            add_type: "left_join".into(),
            join_keys: vec![DatasetProductJoinKeyV1 {
                left_field: "source_1__participant_id".into(),
                right_field: "source_2__participant_id".into(),
            }],
            position: 0,
        }];

        let findings = compatibility_findings(&published, &candidate);

        assert!(findings.iter().any(|finding| {
            finding.code == "added_dataset_source"
                && finding.version_impact == DatasetProductVersionImpactV1::Minor
        }));
    }

    #[test]
    fn changelog_marks_aggregation_changes_as_minor() {
        let mut published = snapshot(Vec::new());
        published.operations = vec![aggregation_operation("count", None)];
        let mut candidate = published.clone();
        candidate.operations = vec![aggregation_operation("sum", Some("source_1__score".into()))];

        let findings = compatibility_findings(&published, &candidate);

        assert!(findings.iter().any(|finding| {
            finding.code == "changed_aggregation_metric_function"
                && finding.version_impact == DatasetProductVersionImpactV1::Minor
        }));
        assert!(findings.iter().any(|finding| {
            finding.code == "changed_aggregation_metric_source"
                && finding.version_impact == DatasetProductVersionImpactV1::Minor
        }));
    }

    #[test]
    fn changelog_marks_calculation_constant_changes_as_patch() {
        let mut published = snapshot(Vec::new());
        published.operations = vec![calculation_operation("add", "10")];
        let mut candidate = published.clone();
        candidate.operations = vec![calculation_operation("add", "20")];

        let findings = compatibility_findings(&published, &candidate);

        assert!(findings.iter().any(|finding| {
            finding.code == "changed_calculation_function_argument"
                && finding.version_impact == DatasetProductVersionImpactV1::Patch
        }));
    }

    #[test]
    fn changelog_marks_calculated_field_type_changes_as_major() {
        let mut published = snapshot(vec![field("calculated_1", "Calculated 1", "number")]);
        published.operations = vec![calculation_operation("add", "10")];
        let mut candidate = snapshot(vec![field("calculated_1", "Calculated 1", "boolean")]);
        candidate.operations = vec![calculation_operation("is", "true")];

        let findings = compatibility_findings(&published, &candidate);

        assert!(findings.iter().any(|finding| {
            finding.code == "changed_calculated_field_type"
                && finding.version_impact == DatasetProductVersionImpactV1::Major
        }));
    }

    #[test]
    fn changelog_marks_visibility_changes_as_patch() {
        let published = snapshot(Vec::new());
        let mut candidate = published.clone();
        candidate.metadata.visibility_node_ids = vec!["organization:child".into()];

        let findings = compatibility_findings(&published, &candidate);

        assert!(findings.iter().any(|finding| {
            finding.code == "changed_dataset_visibility"
                && finding.version_impact == DatasetProductVersionImpactV1::Patch
        }));
    }

    #[test]
    fn semantic_bump_follows_dataset_compatibility_findings() {
        let patch = compatibility_summary(&[compatibility_finding(
            DatasetProductVersionImpactV1::Patch,
            "changed_output_field_label",
            "Output field label changed.".into(),
            Some("field".into()),
        )]);
        assert_eq!(
            semantic_bump_for_publish(&patch, false),
            DatasetProductSemanticBumpV1::Patch
        );

        let minor = compatibility_summary(&[compatibility_finding(
            DatasetProductVersionImpactV1::Minor,
            "added_output_field",
            "Output field is added.".into(),
            Some("new_field".into()),
        )]);
        assert_eq!(
            semantic_bump_for_publish(&minor, false),
            DatasetProductSemanticBumpV1::Minor
        );

        let major = compatibility_summary(&[compatibility_finding(
            DatasetProductVersionImpactV1::Major,
            "removed_output_field",
            "Output field is removed.".into(),
            Some("old_field".into()),
        )]);
        assert_eq!(
            semantic_bump_for_publish(&major, false),
            DatasetProductSemanticBumpV1::Major
        );
    }

    #[test]
    fn forced_major_overrides_compatible_dataset_publish() {
        let compatible = compatibility_summary(&[compatibility_finding(
            DatasetProductVersionImpactV1::Patch,
            "changed_output_field_label",
            "Output field label changed.".into(),
            Some("field".into()),
        )]);

        assert_eq!(
            semantic_bump_for_publish(&compatible, true),
            DatasetProductSemanticBumpV1::Major
        );
    }

    #[test]
    fn empty_changelog_is_not_publishable() {
        assert!(require_publishable_changelog(&[]).is_err());
        assert!(
            require_publishable_changelog(&[compatibility_finding(
                DatasetProductVersionImpactV1::Patch,
                "changed_output_field_label",
                "Output field label changed.".into(),
                Some("field".into()),
            )])
            .is_ok()
        );
    }

    #[test]
    fn unchanged_snapshot_has_no_synthetic_patch_finding() {
        let published = snapshot(vec![field("participant_id", "Participant ID", "text")]);

        assert!(compatibility_findings(&published, &published).is_empty());
    }

    #[test]
    fn source_operation_and_metadata_changes_keep_distinct_impacts() {
        let mut published = snapshot(vec![field("participant_id", "Participant ID", "text")]);
        published.operations = vec![DatasetProductOperationV1::Filter {
            filters: vec![],
            position: 0,
        }];
        let mut candidate = published.clone();
        candidate.initial_source = DatasetProductSourceV1::Form {
            alias: "source_1".into(),
            form_id: "10000000-0000-0000-0000-000000000001".into(),
            form_version_id: "10000000-0000-0000-0000-000000000099".into(),
        };
        candidate.operations = vec![DatasetProductOperationV1::Projection {
            fields: vec![],
            position: 0,
        }];
        candidate.metadata.name = "Renamed Dataset".into();
        candidate.metadata.slug = "renamed-dataset".into();

        let findings = compatibility_findings(&published, &candidate);

        for expected_code in ["changed_dataset_source", "changed_operation_sequence"] {
            assert!(findings.iter().any(|finding| {
                finding.code == expected_code
                    && finding.version_impact == DatasetProductVersionImpactV1::Minor
            }));
        }
        for expected_code in ["changed_dataset_name", "changed_dataset_slug"] {
            assert!(findings.iter().any(|finding| {
                finding.code == expected_code
                    && finding.version_impact == DatasetProductVersionImpactV1::Patch
            }));
        }
    }

    #[test]
    fn removing_source_metric_and_calculated_field_remains_breaking() {
        let mut published = snapshot(Vec::new());
        published.operations = vec![
            DatasetProductOperationV1::AddSource {
                source: DatasetProductSourceV1::Form {
                    alias: "source_2".into(),
                    form_id: "20000000-0000-0000-0000-000000000001".into(),
                    form_version_id: "20000000-0000-0000-0000-000000000002".into(),
                },
                add_type: "union_all".into(),
                join_keys: Vec::new(),
                position: 0,
            },
            aggregation_operation("count", None),
            calculation_operation("add", "1"),
        ];
        let candidate = snapshot(Vec::new());

        let findings = compatibility_findings(&published, &candidate);

        assert!(findings.iter().any(|finding| {
            finding.code == "removed_dataset_source"
                && finding.version_impact == DatasetProductVersionImpactV1::Major
        }));
        assert!(findings.iter().any(|finding| {
            finding.code == "removed_aggregation_operation"
                && finding.version_impact == DatasetProductVersionImpactV1::Major
        }));
        assert!(findings.iter().any(|finding| {
            finding.code == "removed_calculated_fields_operation"
                && finding.version_impact == DatasetProductVersionImpactV1::Major
        }));
    }

    fn snapshot(output_fields: Vec<DatasetProductFieldV1>) -> DatasetCompatibilitySnapshot {
        DatasetCompatibilitySnapshot {
            metadata: DatasetProductRevisionMetadataV1 {
                name: "Test Dataset".into(),
                slug: "test-dataset".into(),
                grain: "submission".into(),
                visibility_node_ids: Vec::new(),
            },
            initial_source: DatasetProductSourceV1::Form {
                alias: "source_1".into(),
                form_id: "10000000-0000-0000-0000-000000000001".into(),
                form_version_id: "10000000-0000-0000-0000-000000000002".into(),
            },
            operations: Vec::new(),
            restriction_policy: None,
            output_fields,
        }
    }

    fn field(key: &str, label: &str, field_type: &str) -> DatasetProductFieldV1 {
        DatasetProductFieldV1 {
            key: key.into(),
            label: label.into(),
            source_alias: "source_1".into(),
            source_field_key: key.into(),
            field_type: field_type.into(),
            position: 0,
        }
    }

    fn aggregation_operation(
        function: &str,
        source_field_key: Option<String>,
    ) -> DatasetProductOperationV1 {
        DatasetProductOperationV1::Aggregation {
            group_fields: vec!["source_1__participant_id".into()],
            metrics: vec![DatasetProductAggregationMetricV1 {
                key: "metric_1".into(),
                label: "Metric 1".into(),
                function: function.into(),
                source_field_key,
                position: 0,
            }],
            row_picker: None,
            position: 0,
        }
    }

    fn calculation_operation(function: &str, argument: &str) -> DatasetProductOperationV1 {
        DatasetProductOperationV1::CalculatedFields {
            fields: vec![DatasetProductCalculatedFieldV1 {
                key: "calculated_1".into(),
                label: "Calculated 1".into(),
                base_field_key: "source_1__score".into(),
                functions: vec![DatasetProductCalculationFunctionV1 {
                    function: function.into(),
                    argument: Some(argument.into()),
                    argument_mode: "value".into(),
                    argument_field_key: None,
                    position: 0,
                }],
                position: 0,
            }],
            position: 0,
        }
    }
}
