//! Fixture-based legacy migration rehearsal support.
//!
//! This module intentionally imports a small JSON fixture instead of connecting
//! directly to the legacy Django database. The goal for Slice 10 is a repeatable
//! end-to-end migration rehearsal while the final import model is still being
//! defined.

use std::{
    collections::{HashMap, HashSet},
    path::Path,
    str::FromStr,
};

use anyhow::Context;
use axum::{Json, extract::State, http::HeaderMap};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::{PgPool, Row};
use tessara_core::FieldType;
use tessara_reporting::MissingDataPolicy;
use uuid::Uuid;

use crate::{
    analytics, auth,
    db::AppState,
    error::{ApiError, ApiResult},
    workflows,
};

/// Summary returned after importing a legacy rehearsal fixture.
#[derive(Serialize)]
pub struct LegacyImportSummary {
    /// Fixture identifier used to make imported submission audit events stable.
    pub fixture_name: String,
    /// Imported Partner node.
    pub partner_node_id: Uuid,
    /// Imported Program node.
    pub program_node_id: Uuid,
    /// Imported Activity node.
    pub activity_node_id: Uuid,
    /// Imported Session node.
    pub session_node_id: Uuid,
    /// Imported form family identifier.
    pub form_id: Uuid,
    /// Published imported form version identifier.
    pub form_version_id: Uuid,
    /// Imported submitted response identifier.
    pub submission_id: Uuid,
    /// Report created for rehearsal validation.
    pub report_id: Uuid,
    /// Dashboard created for rehearsal validation.
    pub dashboard_id: Uuid,
    /// Count of projected analytics values after import refresh.
    pub analytics_values: i64,
}

/// Validation report produced before importing a legacy rehearsal fixture.
///
/// The report is intentionally structured so migration rehearsals can surface
/// actionable row/path-level feedback instead of failing at the first database
/// constraint or type conversion error.
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct LegacyImportValidationReport {
    /// Number of validation issues found in the fixture.
    pub issue_count: usize,
    /// Specific validation issues ordered by fixture traversal.
    pub issues: Vec<LegacyImportValidationIssue>,
}

/// Dry-run report for a legacy fixture import.
///
/// Dry runs parse and validate the fixture but intentionally do not require a
/// database connection or write any Tessara records.
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct LegacyImportDryRunReport {
    /// Fixture identifier from the import payload.
    pub fixture_name: String,
    /// Whether the fixture would be handed to the importer.
    pub would_import: bool,
    /// Validation details for the fixture.
    pub validation: LegacyImportValidationReport,
}

/// Fixture example exposed by the local migration workbench.
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct LegacyFixtureExample {
    /// Human-readable fixture identifier.
    pub name: String,
    /// Complete fixture JSON ready for validation or dry-run.
    pub fixture_json: String,
}

impl LegacyImportValidationReport {
    /// Returns `true` when the fixture is safe to hand to the database importer.
    pub fn is_clean(&self) -> bool {
        self.issues.is_empty()
    }

    fn into_error_message(self) -> String {
        let preview = self
            .issues
            .iter()
            .take(5)
            .map(|issue| format!("{} at {}: {}", issue.code, issue.path, issue.message))
            .collect::<Vec<_>>()
            .join("; ");

        if self.issue_count > 5 {
            format!(
                "legacy fixture validation failed with {} issues: {}; ...",
                self.issue_count, preview
            )
        } else {
            format!(
                "legacy fixture validation failed with {} issues: {}",
                self.issue_count, preview
            )
        }
    }
}

/// Single actionable legacy fixture validation issue.
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct LegacyImportValidationIssue {
    /// Stable issue code suitable for filtering migration reports.
    pub code: String,
    /// JSON-like fixture path for the affected value.
    pub path: String,
    /// Human-readable explanation of the mapping or data problem.
    pub message: String,
}

#[derive(Deserialize)]
struct LegacyFixture {
    name: String,
    partner: LegacyNode,
    program: LegacyNode,
    activity: LegacyNode,
    session: LegacySession,
    #[serde(default)]
    choice_lists: Vec<LegacyChoiceList>,
    form: LegacyForm,
    submission: LegacySubmission,
    report: LegacyReport,
    dashboard: LegacyDashboard,
}

#[derive(Deserialize)]
struct LegacyNode {
    legacy_id: String,
    name: String,
    #[serde(default = "default_true")]
    is_active: bool,
    #[serde(default)]
    locked: bool,
}

#[derive(Deserialize)]
struct LegacySession {
    legacy_id: String,
    name: String,
    date: String,
    #[serde(default = "default_true")]
    is_active: bool,
    #[serde(default)]
    locked: bool,
}

#[derive(Deserialize)]
struct LegacyChoiceList {
    legacy_id: String,
    name: String,
    #[serde(default = "default_true")]
    is_active: bool,
    choices: Vec<LegacyChoice>,
}

#[derive(Deserialize)]
struct LegacyChoice {
    legacy_id: String,
    label: String,
    #[serde(default)]
    description: Option<String>,
    ordering: i32,
    #[serde(default = "default_true")]
    is_active: bool,
}

#[derive(Deserialize)]
struct LegacyForm {
    legacy_id: String,
    name: String,
    slug: String,
    scope_node_type: String,
    compatibility_group: String,
    version_label: String,
    sections: Vec<LegacySection>,
}

#[derive(Deserialize)]
struct LegacySection {
    legacy_id: String,
    label: String,
    position: i32,
    fields: Vec<LegacyField>,
}

#[derive(Deserialize)]
struct LegacyField {
    legacy_id: String,
    key: String,
    label: String,
    field_type: String,
    required: bool,
    position: i32,
}

#[derive(Deserialize)]
struct LegacySubmission {
    legacy_id: String,
    node_type: String,
    values: HashMap<String, Value>,
}

#[derive(Deserialize)]
struct LegacyReport {
    name: String,
    logical_key: String,
    source_field_key: String,
    #[serde(default = "default_missing_policy")]
    missing_policy: String,
}

#[derive(Deserialize)]
struct LegacyDashboard {
    name: String,
    chart_name: String,
}

#[derive(Deserialize)]
pub struct ValidateLegacyFixtureRequest {
    fixture_json: String,
}

/// Validates a legacy fixture submitted through the local migration workbench.
pub async fn validate_legacy_fixture_endpoint(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<ValidateLegacyFixtureRequest>,
) -> ApiResult<Json<LegacyImportValidationReport>> {
    auth::require_capability(&state.pool, &headers, "admin:all").await?;
    Ok(Json(validate_legacy_fixture_str(&payload.fixture_json)?))
}

/// Dry-runs a legacy fixture submitted through the local migration workbench.
pub async fn dry_run_legacy_fixture_endpoint(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<ValidateLegacyFixtureRequest>,
) -> ApiResult<Json<LegacyImportDryRunReport>> {
    auth::require_capability(&state.pool, &headers, "admin:all").await?;
    Ok(Json(dry_run_legacy_fixture_str(&payload.fixture_json)?))
}

/// Imports a validated legacy fixture submitted through the local workbench.
pub async fn import_legacy_fixture_endpoint(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<ValidateLegacyFixtureRequest>,
) -> ApiResult<Json<LegacyImportSummary>> {
    auth::require_capability(&state.pool, &headers, "admin:all").await?;
    Ok(Json(
        import_legacy_fixture_str(&state.pool, &payload.fixture_json).await?,
    ))
}

/// Lists bundled legacy fixture examples for local migration workbench testing.
pub async fn list_legacy_fixture_examples(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> ApiResult<Json<Vec<LegacyFixtureExample>>> {
    auth::require_capability(&state.pool, &headers, "admin:all").await?;
    Ok(Json(vec![
        LegacyFixtureExample {
            name: "legacy-rehearsal".to_string(),
            fixture_json: include_str!("../../../fixtures/legacy-rehearsal.json").to_string(),
        },
        LegacyFixtureExample {
            name: "legacy-inactive-locked".to_string(),
            fixture_json: include_str!("../../../fixtures/legacy-inactive-locked.json").to_string(),
        },
    ]))
}

/// Imports a legacy rehearsal fixture from a JSON file path.
pub async fn import_legacy_fixture_file(
    pool: &PgPool,
    path: impl AsRef<Path>,
) -> anyhow::Result<LegacyImportSummary> {
    let path = path.as_ref();
    let fixture = std::fs::read_to_string(path)
        .with_context(|| format!("failed to read legacy fixture {}", path.display()))?;
    import_legacy_fixture_str(pool, &fixture)
        .await
        .map_err(anyhow::Error::from)
}

/// Imports a legacy rehearsal fixture from a JSON string.
pub async fn import_legacy_fixture_str(
    pool: &PgPool,
    fixture_json: &str,
) -> ApiResult<LegacyImportSummary> {
    let fixture = parse_legacy_fixture(fixture_json)?;
    let validation = validate_legacy_fixture(&fixture);
    if !validation.is_clean() {
        return Err(ApiError::BadRequest(validation.into_error_message()));
    }

    import_legacy_fixture(pool, fixture).await
}

/// Validates a legacy rehearsal fixture without writing to the database.
///
/// This is the preflight surface used by tests and migration scripts to find
/// ambiguous mappings, missing field references, and type mismatches before an
/// import attempts to mutate Tessara state.
pub fn validate_legacy_fixture_str(fixture_json: &str) -> ApiResult<LegacyImportValidationReport> {
    let fixture = parse_legacy_fixture(fixture_json)?;
    Ok(validate_legacy_fixture(&fixture))
}

/// Parses and validates a legacy fixture without touching the database.
pub fn dry_run_legacy_fixture_str(fixture_json: &str) -> ApiResult<LegacyImportDryRunReport> {
    let fixture = parse_legacy_fixture(fixture_json)?;
    let fixture_name = fixture.name.clone();
    let validation = validate_legacy_fixture(&fixture);
    let would_import = validation.is_clean();

    Ok(LegacyImportDryRunReport {
        fixture_name,
        would_import,
        validation,
    })
}

fn parse_legacy_fixture(fixture_json: &str) -> ApiResult<LegacyFixture> {
    serde_json::from_str(fixture_json).map_err(|err| ApiError::BadRequest(err.to_string()))
}

fn validate_legacy_fixture(fixture: &LegacyFixture) -> LegacyImportValidationReport {
    let mut validator = LegacyFixtureValidator::default();

    validator.require_non_empty("name", &fixture.name, "fixture name");
    validator.validate_node("partner", &fixture.partner);
    validator.validate_node("program", &fixture.program);
    validator.validate_node("activity", &fixture.activity);
    validator.validate_session("session", &fixture.session);
    validator.validate_distinct_node_legacy_ids(fixture);
    validator.validate_choice_lists(&fixture.choice_lists);

    let mut field_keys = HashMap::new();
    validator.validate_form(&fixture.form, &mut field_keys);
    validator.validate_submission(&fixture.submission, &field_keys);
    validator.validate_report(&fixture.report, &field_keys);
    validator.require_non_empty("dashboard.name", &fixture.dashboard.name, "dashboard name");
    validator.require_non_empty(
        "dashboard.chart_name",
        &fixture.dashboard.chart_name,
        "dashboard chart name",
    );

    validator.into_report()
}

#[derive(Default)]
struct LegacyFixtureValidator {
    issues: Vec<LegacyImportValidationIssue>,
}

impl LegacyFixtureValidator {
    fn into_report(self) -> LegacyImportValidationReport {
        LegacyImportValidationReport {
            issue_count: self.issues.len(),
            issues: self.issues,
        }
    }

    fn validate_node(&mut self, path: &str, node: &LegacyNode) {
        self.require_non_empty(
            &format!("{path}.legacy_id"),
            &node.legacy_id,
            "node legacy ID",
        );
        self.require_non_empty(&format!("{path}.name"), &node.name, "node name");
    }

    fn validate_session(&mut self, path: &str, session: &LegacySession) {
        self.require_non_empty(
            &format!("{path}.legacy_id"),
            &session.legacy_id,
            "session legacy ID",
        );
        self.require_non_empty(&format!("{path}.name"), &session.name, "session name");
        self.require_non_empty(&format!("{path}.date"), &session.date, "session date");
    }

    fn validate_distinct_node_legacy_ids(&mut self, fixture: &LegacyFixture) {
        let mut seen = HashMap::new();
        for (path, legacy_id) in [
            ("partner.legacy_id", &fixture.partner.legacy_id),
            ("program.legacy_id", &fixture.program.legacy_id),
            ("activity.legacy_id", &fixture.activity.legacy_id),
            ("session.legacy_id", &fixture.session.legacy_id),
        ] {
            if legacy_id.trim().is_empty() {
                continue;
            }
            if let Some(previous_path) = seen.insert(legacy_id.as_str(), path) {
                self.push_issue(
                    "duplicate_node_legacy_id",
                    path,
                    format!(
                        "node legacy ID '{legacy_id}' is also used at {previous_path}; import mappings must be unambiguous"
                    ),
                );
            }
        }
    }

    fn validate_choice_lists(&mut self, choice_lists: &[LegacyChoiceList]) {
        let mut list_names = HashSet::new();
        for (list_index, choice_list) in choice_lists.iter().enumerate() {
            let list_path = format!("choice_lists[{list_index}]");
            self.require_non_empty(
                &format!("{list_path}.legacy_id"),
                &choice_list.legacy_id,
                "choice list legacy ID",
            );
            self.require_non_empty(
                &format!("{list_path}.name"),
                &choice_list.name,
                "choice list name",
            );
            if !list_names.insert(choice_list.name.to_ascii_lowercase()) {
                self.push_issue(
                    "duplicate_choice_list_name",
                    format!("{list_path}.name"),
                    format!(
                        "choice list name '{}' appears more than once in this fixture",
                        choice_list.name
                    ),
                );
            }

            let mut choice_ids = HashSet::new();
            for (choice_index, choice) in choice_list.choices.iter().enumerate() {
                let choice_path = format!("{list_path}.choices[{choice_index}]");
                self.require_non_empty(
                    &format!("{choice_path}.legacy_id"),
                    &choice.legacy_id,
                    "choice legacy ID",
                );
                self.require_non_empty(
                    &format!("{choice_path}.label"),
                    &choice.label,
                    "choice label",
                );
                if !choice.legacy_id.trim().is_empty()
                    && !choice_ids.insert(choice.legacy_id.as_str())
                {
                    self.push_issue(
                        "duplicate_choice_legacy_id",
                        format!("{choice_path}.legacy_id"),
                        format!(
                            "choice legacy ID '{}' appears more than once in choice list '{}'",
                            choice.legacy_id, choice_list.name
                        ),
                    );
                }
            }
        }
    }

    fn validate_form<'a>(
        &mut self,
        form: &'a LegacyForm,
        field_keys: &mut HashMap<&'a str, &'a LegacyField>,
    ) {
        self.require_non_empty("form.legacy_id", &form.legacy_id, "form legacy ID");
        self.require_non_empty("form.name", &form.name, "form name");
        self.require_non_empty("form.slug", &form.slug, "form slug");
        self.require_non_empty(
            "form.compatibility_group",
            &form.compatibility_group,
            "form compatibility group",
        );
        self.require_non_empty(
            "form.version_label",
            &form.version_label,
            "form version label",
        );
        self.validate_scope("form.scope_node_type", &form.scope_node_type);

        let mut section_labels = HashSet::new();
        for (section_index, section) in form.sections.iter().enumerate() {
            let section_path = format!("form.sections[{section_index}]");
            self.require_non_empty(
                &format!("{section_path}.legacy_id"),
                &section.legacy_id,
                "section legacy ID",
            );
            self.require_non_empty(
                &format!("{section_path}.label"),
                &section.label,
                "section label",
            );
            if !section_labels.insert(section.label.to_ascii_lowercase()) {
                self.push_issue(
                    "duplicate_section_label",
                    format!("{section_path}.label"),
                    format!("section label '{}' appears more than once", section.label),
                );
            }

            for (field_index, field) in section.fields.iter().enumerate() {
                let field_path = format!("{section_path}.fields[{field_index}]");
                self.require_non_empty(
                    &format!("{field_path}.legacy_id"),
                    &field.legacy_id,
                    "field legacy ID",
                );
                self.require_non_empty(&format!("{field_path}.key"), &field.key, "field key");
                self.require_non_empty(&format!("{field_path}.label"), &field.label, "field label");
                if FieldType::from_str(&field.field_type).is_err() {
                    self.push_issue(
                        "unsupported_field_type",
                        format!("{field_path}.field_type"),
                        format!(
                            "field '{}' uses unsupported type '{}'",
                            field.key, field.field_type
                        ),
                    );
                }
                if !field.key.trim().is_empty()
                    && field_keys.insert(field.key.as_str(), field).is_some()
                {
                    self.push_issue(
                        "duplicate_form_field_key",
                        format!("{field_path}.key"),
                        format!("field key '{}' appears more than once", field.key),
                    );
                }
            }
        }
    }

    fn validate_submission(
        &mut self,
        submission: &LegacySubmission,
        field_keys: &HashMap<&str, &LegacyField>,
    ) {
        self.require_non_empty(
            "submission.legacy_id",
            &submission.legacy_id,
            "submission legacy ID",
        );
        self.validate_scope("submission.node_type", &submission.node_type);

        for field in field_keys.values().filter(|field| field.required) {
            if !submission.values.contains_key(&field.key) {
                self.push_issue(
                    "missing_required_submission_value",
                    format!("submission.values.{}", field.key),
                    format!(
                        "required field '{}' is missing from the submission",
                        field.key
                    ),
                );
            }
        }

        for (field_key, value) in &submission.values {
            let Some(field) = field_keys.get(field_key.as_str()) else {
                self.push_issue(
                    "unknown_submission_field",
                    format!("submission.values.{field_key}"),
                    format!("submission value references unknown form field '{field_key}'"),
                );
                continue;
            };

            let Ok(field_type) = FieldType::from_str(&field.field_type) else {
                continue;
            };
            if let Err(error) = field_type.validate_json_value(value) {
                self.push_issue(
                    "invalid_submission_value_type",
                    format!("submission.values.{field_key}"),
                    format!(
                        "submission value for field '{field_key}' is not compatible with {}: {error}",
                        field_type.as_str()
                    ),
                );
            }
        }
    }

    fn validate_report(&mut self, report: &LegacyReport, field_keys: &HashMap<&str, &LegacyField>) {
        self.require_non_empty("report.name", &report.name, "report name");
        self.require_non_empty(
            "report.logical_key",
            &report.logical_key,
            "report logical key",
        );
        self.require_non_empty(
            "report.source_field_key",
            &report.source_field_key,
            "report source field key",
        );
        if MissingDataPolicy::from_str(&report.missing_policy).is_err() {
            self.push_issue(
                "unsupported_missing_policy",
                "report.missing_policy",
                format!(
                    "report '{}' uses unsupported missing-data policy '{}'",
                    report.name, report.missing_policy
                ),
            );
        }
        if !report.source_field_key.trim().is_empty()
            && !field_keys.contains_key(report.source_field_key.as_str())
        {
            self.push_issue(
                "unknown_report_source_field",
                "report.source_field_key",
                format!(
                    "report '{}' references unknown source field '{}'",
                    report.name, report.source_field_key
                ),
            );
        }
    }

    fn validate_scope(&mut self, path: &str, scope: &str) {
        self.require_non_empty(path, scope, "node type scope");
        if !["partner", "program", "activity", "session"]
            .iter()
            .any(|allowed| allowed.eq_ignore_ascii_case(scope))
        {
            self.push_issue(
                "unknown_node_type_scope",
                path,
                format!(
                    "node type scope '{scope}' is not one of partner, program, activity, session"
                ),
            );
        }
    }

    fn require_non_empty(&mut self, path: &str, value: &str, label: &str) {
        if value.trim().is_empty() {
            self.push_issue(
                "missing_required_value",
                path,
                format!("{label} is required"),
            );
        }
    }

    fn push_issue(
        &mut self,
        code: impl Into<String>,
        path: impl Into<String>,
        message: impl Into<String>,
    ) {
        self.issues.push(LegacyImportValidationIssue {
            code: code.into(),
            path: path.into(),
            message: message.into(),
        });
    }
}

async fn import_legacy_fixture(
    pool: &PgPool,
    fixture: LegacyFixture,
) -> ApiResult<LegacyImportSummary> {
    let partner_type_id = ensure_node_type(pool, "Partner", "partner").await?;
    let program_type_id = ensure_node_type(pool, "Program", "program").await?;
    let activity_type_id = ensure_node_type(pool, "Activity", "activity").await?;
    let session_type_id = ensure_node_type(pool, "Session", "session").await?;

    ensure_node_type_relationship(pool, partner_type_id, program_type_id).await?;
    ensure_node_type_relationship(pool, program_type_id, activity_type_id).await?;
    ensure_node_type_relationship(pool, activity_type_id, session_type_id).await?;

    ensure_standard_node_metadata(pool, partner_type_id, false).await?;
    ensure_standard_node_metadata(pool, program_type_id, false).await?;
    ensure_standard_node_metadata(pool, activity_type_id, false).await?;
    ensure_standard_node_metadata(pool, session_type_id, true).await?;

    let partner_node_id = ensure_node(pool, partner_type_id, None, &fixture.partner, None).await?;
    let program_node_id = ensure_node(
        pool,
        program_type_id,
        Some(partner_node_id),
        &fixture.program,
        None,
    )
    .await?;
    let activity_node_id = ensure_node(
        pool,
        activity_type_id,
        Some(program_node_id),
        &fixture.activity,
        None,
    )
    .await?;
    let session_node_id = ensure_node(
        pool,
        session_type_id,
        Some(activity_node_id),
        &LegacyNode {
            legacy_id: fixture.session.legacy_id.clone(),
            name: fixture.session.name.clone(),
            is_active: fixture.session.is_active,
            locked: fixture.session.locked,
        },
        Some(&fixture.session.date),
    )
    .await?;

    let scope_node_type_id = node_type_id_for_scope(
        &fixture.form.scope_node_type,
        [
            ("partner", partner_type_id),
            ("program", program_type_id),
            ("activity", activity_type_id),
            ("session", session_type_id),
        ],
    )?;
    let form_id = ensure_form(pool, &fixture.form, scope_node_type_id).await?;
    let compatibility_group_id =
        ensure_compatibility_group(pool, form_id, &fixture.form.compatibility_group).await?;
    let form_version_id = ensure_form_version(
        pool,
        form_id,
        compatibility_group_id,
        &fixture.form.version_label,
    )
    .await?;
    ensure_form_sections_and_fields(pool, form_version_id, &fixture.form).await?;
    for choice_list in &fixture.choice_lists {
        ensure_choice_list(pool, form_version_id, choice_list).await?;
    }
    publish_form_version(pool, form_id, compatibility_group_id, form_version_id).await?;

    let submission_node_id = node_id_for_submission_node_type(
        &fixture.submission.node_type,
        [
            ("partner", partner_node_id),
            ("program", program_node_id),
            ("activity", activity_node_id),
            ("session", session_node_id),
        ],
    )?;
    let submission_id = ensure_submission(
        pool,
        &fixture.name,
        form_version_id,
        submission_node_id,
        &fixture.submission,
    )
    .await?;
    let analytics_status = analytics::refresh_projection(pool).await?;

    let report_id = ensure_report(pool, form_id, &fixture.report).await?;
    let chart_id = ensure_chart(pool, &fixture.dashboard.chart_name, report_id).await?;
    let dashboard_id = ensure_dashboard(pool, &fixture.dashboard.name).await?;
    ensure_dashboard_component(pool, dashboard_id, chart_id).await?;

    Ok(LegacyImportSummary {
        fixture_name: fixture.name,
        partner_node_id,
        program_node_id,
        activity_node_id,
        session_node_id,
        form_id,
        form_version_id,
        submission_id,
        report_id,
        dashboard_id,
        analytics_values: analytics_status.value_count,
    })
}

async fn ensure_node_type(pool: &PgPool, name: &str, slug: &str) -> ApiResult<Uuid> {
    Ok(sqlx::query_scalar(
        r#"
        INSERT INTO node_types (name, slug)
        VALUES ($1, $2)
        ON CONFLICT (slug) DO UPDATE SET name = EXCLUDED.name
        RETURNING id
        "#,
    )
    .bind(name)
    .bind(slug)
    .fetch_one(pool)
    .await?)
}

async fn ensure_node_type_relationship(
    pool: &PgPool,
    parent_node_type_id: Uuid,
    child_node_type_id: Uuid,
) -> ApiResult<()> {
    sqlx::query(
        r#"
        INSERT INTO node_type_relationships (parent_node_type_id, child_node_type_id)
        VALUES ($1, $2)
        ON CONFLICT DO NOTHING
        "#,
    )
    .bind(parent_node_type_id)
    .bind(child_node_type_id)
    .execute(pool)
    .await?;

    Ok(())
}

async fn ensure_standard_node_metadata(
    pool: &PgPool,
    node_type_id: Uuid,
    include_session_date: bool,
) -> ApiResult<()> {
    for (key, label, field_type, required) in [
        ("legacy_id", "Legacy ID", "text", true),
        ("is_active", "Legacy Active", "boolean", true),
        ("locked", "Legacy Locked", "boolean", true),
    ] {
        ensure_node_metadata_field(pool, node_type_id, key, label, field_type, required).await?;
    }

    if include_session_date {
        ensure_node_metadata_field(
            pool,
            node_type_id,
            "session_date",
            "Session Date",
            "date",
            true,
        )
        .await?;
    }

    Ok(())
}

async fn ensure_node_metadata_field(
    pool: &PgPool,
    node_type_id: Uuid,
    key: &str,
    label: &str,
    field_type: &str,
    required: bool,
) -> ApiResult<Uuid> {
    Ok(sqlx::query_scalar(
        r#"
        INSERT INTO node_metadata_field_definitions
            (node_type_id, key, label, field_type, required)
        VALUES ($1, $2, $3, $4::field_type, $5)
        ON CONFLICT (node_type_id, key)
        DO UPDATE SET label = EXCLUDED.label, required = EXCLUDED.required
        RETURNING id
        "#,
    )
    .bind(node_type_id)
    .bind(key)
    .bind(label)
    .bind(field_type)
    .bind(required)
    .fetch_one(pool)
    .await?)
}

async fn ensure_node(
    pool: &PgPool,
    node_type_id: Uuid,
    parent_node_id: Option<Uuid>,
    node: &LegacyNode,
    session_date: Option<&str>,
) -> ApiResult<Uuid> {
    let node_id = if let Some(id) = sqlx::query_scalar(
        r#"
        SELECT nodes.id
        FROM nodes
        JOIN node_metadata_values ON node_metadata_values.node_id = nodes.id
        JOIN node_metadata_field_definitions
            ON node_metadata_field_definitions.id = node_metadata_values.field_definition_id
        WHERE nodes.node_type_id = $1
          AND nodes.parent_node_id IS NOT DISTINCT FROM $2
          AND node_metadata_field_definitions.key = 'legacy_id'
          AND node_metadata_values.value = to_jsonb($3::text)
        "#,
    )
    .bind(node_type_id)
    .bind(parent_node_id)
    .bind(&node.legacy_id)
    .fetch_optional(pool)
    .await?
    {
        sqlx::query("UPDATE nodes SET name = $1 WHERE id = $2")
            .bind(&node.name)
            .bind(id)
            .execute(pool)
            .await?;
        id
    } else {
        sqlx::query_scalar(
            "INSERT INTO nodes (node_type_id, parent_node_id, name) VALUES ($1, $2, $3) RETURNING id",
        )
        .bind(node_type_id)
        .bind(parent_node_id)
        .bind(&node.name)
        .fetch_one(pool)
        .await?
    };

    set_node_metadata(
        pool,
        node_id,
        node_type_id,
        "legacy_id",
        Value::String(node.legacy_id.clone()),
    )
    .await?;
    set_node_metadata(
        pool,
        node_id,
        node_type_id,
        "is_active",
        Value::Bool(node.is_active),
    )
    .await?;
    set_node_metadata(
        pool,
        node_id,
        node_type_id,
        "locked",
        Value::Bool(node.locked),
    )
    .await?;
    if let Some(session_date) = session_date {
        set_node_metadata(
            pool,
            node_id,
            node_type_id,
            "session_date",
            Value::String(session_date.to_string()),
        )
        .await?;
    }

    Ok(node_id)
}

async fn set_node_metadata(
    pool: &PgPool,
    node_id: Uuid,
    node_type_id: Uuid,
    key: &str,
    value: Value,
) -> ApiResult<()> {
    let field_definition_id: Uuid = sqlx::query_scalar(
        "SELECT id FROM node_metadata_field_definitions WHERE node_type_id = $1 AND key = $2",
    )
    .bind(node_type_id)
    .bind(key)
    .fetch_one(pool)
    .await?;

    sqlx::query(
        r#"
        INSERT INTO node_metadata_values (node_id, field_definition_id, value)
        VALUES ($1, $2, $3)
        ON CONFLICT (node_id, field_definition_id)
        DO UPDATE SET value = EXCLUDED.value
        "#,
    )
    .bind(node_id)
    .bind(field_definition_id)
    .bind(value)
    .execute(pool)
    .await?;

    Ok(())
}

async fn ensure_choice_list(
    pool: &PgPool,
    form_version_id: Uuid,
    choice_list: &LegacyChoiceList,
) -> ApiResult<Uuid> {
    let choice_list_id: Uuid = sqlx::query_scalar(
        r#"
        INSERT INTO choice_lists (form_version_id, name)
        VALUES ($1, $2)
        ON CONFLICT (form_version_id, name) DO UPDATE SET name = EXCLUDED.name
        RETURNING id
        "#,
    )
    .bind(form_version_id)
    .bind(&choice_list.name)
    .fetch_one(pool)
    .await?;

    let active_item_value = format!("legacy-active:{}", choice_list.legacy_id);
    sqlx::query(
        r#"
        INSERT INTO choice_list_items (choice_list_id, value, label, position)
        VALUES ($1, $2, $3, -1)
        ON CONFLICT (choice_list_id, value)
        DO UPDATE SET label = EXCLUDED.label, position = EXCLUDED.position
        "#,
    )
    .bind(choice_list_id)
    .bind(active_item_value)
    .bind(format!("legacy is_active={}", choice_list.is_active))
    .execute(pool)
    .await?;

    for choice in &choice_list.choices {
        sqlx::query(
            r#"
            INSERT INTO choice_list_items (choice_list_id, value, label, position)
            VALUES ($1, $2, $3, $4)
            ON CONFLICT (choice_list_id, value)
            DO UPDATE SET label = EXCLUDED.label, position = EXCLUDED.position
            "#,
        )
        .bind(choice_list_id)
        .bind(&choice.legacy_id)
        .bind(choice.description.as_deref().unwrap_or(&choice.label))
        .bind(choice.ordering)
        .execute(pool)
        .await?;

        if !choice.is_active {
            let inactive_item_value = format!("legacy-inactive:{}", choice.legacy_id);
            sqlx::query(
                r#"
                INSERT INTO choice_list_items (choice_list_id, value, label, position)
                VALUES ($1, $2, $3, $4)
                ON CONFLICT (choice_list_id, value)
                DO UPDATE SET label = EXCLUDED.label, position = EXCLUDED.position
                "#,
            )
            .bind(choice_list_id)
            .bind(inactive_item_value)
            .bind(format!("inactive: {}", choice.label))
            .bind(choice.ordering)
            .execute(pool)
            .await?;
        }
    }

    Ok(choice_list_id)
}

async fn ensure_form(
    pool: &PgPool,
    form: &LegacyForm,
    scope_node_type_id: Uuid,
) -> ApiResult<Uuid> {
    let _form_legacy_id = &form.legacy_id;
    Ok(sqlx::query_scalar(
        r#"
        INSERT INTO forms (name, slug, scope_node_type_id)
        VALUES ($1, $2, $3)
        ON CONFLICT (slug)
        DO UPDATE SET name = EXCLUDED.name, scope_node_type_id = EXCLUDED.scope_node_type_id
        RETURNING id
        "#,
    )
    .bind(&form.name)
    .bind(&form.slug)
    .bind(scope_node_type_id)
    .fetch_one(pool)
    .await?)
}

async fn ensure_compatibility_group(pool: &PgPool, form_id: Uuid, name: &str) -> ApiResult<Uuid> {
    Ok(sqlx::query_scalar(
        r#"
        INSERT INTO compatibility_groups (form_id, name)
        VALUES ($1, $2)
        ON CONFLICT (form_id, name)
        DO UPDATE SET name = EXCLUDED.name
        RETURNING id
        "#,
    )
    .bind(form_id)
    .bind(name)
    .fetch_one(pool)
    .await?)
}

async fn ensure_form_version(
    pool: &PgPool,
    form_id: Uuid,
    compatibility_group_id: Uuid,
    version_label: &str,
) -> ApiResult<Uuid> {
    let (version_major, version_minor, version_patch) =
        parse_semantic_version_label(version_label)?;
    Ok(sqlx::query_scalar(
        r#"
        INSERT INTO form_versions (
            form_id,
            compatibility_group_id,
            version_label,
            version_major,
            version_minor,
            version_patch,
            started_new_major_line
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7)
        ON CONFLICT (form_id, version_label)
        DO UPDATE SET
            compatibility_group_id = EXCLUDED.compatibility_group_id,
            version_major = EXCLUDED.version_major,
            version_minor = EXCLUDED.version_minor,
            version_patch = EXCLUDED.version_patch,
            started_new_major_line = EXCLUDED.started_new_major_line
        RETURNING id
        "#,
    )
    .bind(form_id)
    .bind(compatibility_group_id)
    .bind(version_label)
    .bind(version_major)
    .bind(version_minor)
    .bind(version_patch)
    .bind(version_minor == 0 && version_patch == 0)
    .fetch_one(pool)
    .await?)
}

async fn ensure_form_sections_and_fields(
    pool: &PgPool,
    form_version_id: Uuid,
    form: &LegacyForm,
) -> ApiResult<()> {
    for section in &form.sections {
        let section_id: Uuid = if let Some(id) = sqlx::query_scalar(
            "SELECT id FROM form_sections WHERE form_version_id = $1 AND title = $2",
        )
        .bind(form_version_id)
        .bind(&section.label)
        .fetch_optional(pool)
        .await?
        {
            sqlx::query("UPDATE form_sections SET position = $1 WHERE id = $2")
                .bind(section.position)
                .bind(id)
                .execute(pool)
                .await?;
            id
        } else {
            sqlx::query_scalar(
                "INSERT INTO form_sections (form_version_id, title, position) VALUES ($1, $2, $3) RETURNING id",
            )
            .bind(form_version_id)
            .bind(&section.label)
            .bind(section.position)
            .fetch_one(pool)
            .await?
        };

        let _section_legacy_id = &section.legacy_id;
        for field in &section.fields {
            let field_type = FieldType::from_str(&field.field_type)
                .map_err(|error| ApiError::BadRequest(error.to_string()))?;
            let _field_legacy_id = &field.legacy_id;
            sqlx::query(
                r#"
                INSERT INTO form_fields
                    (form_version_id, section_id, key, label, field_type, required, position)
                VALUES ($1, $2, $3, $4, $5::field_type, $6, $7)
                ON CONFLICT (form_version_id, key)
                DO UPDATE SET
                    section_id = EXCLUDED.section_id,
                    label = EXCLUDED.label,
                    field_type = EXCLUDED.field_type,
                    required = EXCLUDED.required,
                    position = EXCLUDED.position
                "#,
            )
            .bind(form_version_id)
            .bind(section_id)
            .bind(&field.key)
            .bind(&field.label)
            .bind(field_type.as_str())
            .bind(field.required)
            .bind(field.position)
            .execute(pool)
            .await?;
        }
    }

    Ok(())
}

async fn publish_form_version(
    pool: &PgPool,
    form_id: Uuid,
    _compatibility_group_id: Uuid,
    form_version_id: Uuid,
) -> ApiResult<()> {
    sqlx::query(
        r#"
        UPDATE form_versions
        SET status = 'superseded'::form_version_status
        WHERE form_id = $1
          AND version_major = (SELECT version_major FROM form_versions WHERE id = $2)
          AND id != $2
          AND status = 'published'::form_version_status
        "#,
    )
    .bind(form_id)
    .bind(form_version_id)
    .execute(pool)
    .await?;

    sqlx::query(
        r#"
        UPDATE form_versions
        SET status = 'published'::form_version_status,
            published_at = COALESCE(published_at, now())
        WHERE id = $1
        "#,
    )
    .bind(form_version_id)
    .execute(pool)
    .await?;

    Ok(())
}

async fn ensure_submission(
    pool: &PgPool,
    fixture_name: &str,
    form_version_id: Uuid,
    node_id: Uuid,
    submission: &LegacySubmission,
) -> ApiResult<Uuid> {
    let event_type = format!("legacy_import:{fixture_name}:{}", submission.legacy_id);
    if let Some(id) = sqlx::query_scalar(
        r#"
        SELECT submission_id
        FROM submission_audit_events
        WHERE event_type = $1
        LIMIT 1
        "#,
    )
    .bind(&event_type)
    .fetch_optional(pool)
    .await?
    {
        return Ok(id);
    }

    let account_id: Uuid =
        sqlx::query_scalar("SELECT id FROM accounts WHERE email = 'admin@tessara.local' LIMIT 1")
            .fetch_one(pool)
            .await?;
    let assignment_id: Uuid = sqlx::query_scalar(
        r#"
        INSERT INTO form_assignments (form_version_id, node_id, account_id)
        VALUES ($1, $2, $3)
        RETURNING id
        "#,
    )
    .bind(form_version_id)
    .bind(node_id)
    .bind(account_id)
    .fetch_one(pool)
    .await?;
    let _ = workflows::ensure_workflow_assignment_for_form_assignment(pool, assignment_id).await?;
    let submission_id: Uuid = sqlx::query_scalar(
        r#"
        INSERT INTO submissions (assignment_id, form_version_id, node_id, status, submitted_at)
        VALUES ($1, $2, $3, 'submitted'::submission_status, now())
        RETURNING id
        "#,
    )
    .bind(assignment_id)
    .bind(form_version_id)
    .bind(node_id)
    .fetch_one(pool)
    .await?;

    for (field_key, value) in &submission.values {
        let row = sqlx::query(
            "SELECT id, field_type::text AS field_type FROM form_fields WHERE form_version_id = $1 AND key = $2",
        )
        .bind(form_version_id)
        .bind(field_key)
        .fetch_one(pool)
        .await?;
        let field_id: Uuid = row.try_get("id")?;
        let field_type = FieldType::from_str(&row.try_get::<String, _>("field_type")?)
            .map_err(|error| ApiError::BadRequest(error.to_string()))?;
        field_type
            .validate_json_value(value)
            .map_err(|error| ApiError::BadRequest(error.to_string()))?;

        sqlx::query(
            "INSERT INTO submission_values (submission_id, field_id, value) VALUES ($1, $2, $3)",
        )
        .bind(submission_id)
        .bind(field_id)
        .bind(value)
        .execute(pool)
        .await?;
    }

    sqlx::query(
        "INSERT INTO submission_audit_events (submission_id, event_type, account_id) VALUES ($1, $2, $3)",
    )
    .bind(submission_id)
    .bind(event_type)
    .bind(account_id)
    .execute(pool)
    .await?;

    Ok(submission_id)
}

async fn ensure_report(pool: &PgPool, form_id: Uuid, report: &LegacyReport) -> ApiResult<Uuid> {
    let form_version_major = current_form_major(pool, form_id).await?;
    let report_id: Uuid = if let Some(id) =
        sqlx::query_scalar("SELECT id FROM reports WHERE form_id = $1 AND name = $2")
            .bind(form_id)
            .bind(&report.name)
            .fetch_optional(pool)
            .await?
    {
        sqlx::query("UPDATE reports SET form_version_major = $1 WHERE id = $2")
            .bind(form_version_major)
            .bind(id)
            .execute(pool)
            .await?;
        id
    } else {
        sqlx::query_scalar(
            "INSERT INTO reports (name, form_id, form_version_major) VALUES ($1, $2, $3) RETURNING id",
        )
            .bind(&report.name)
            .bind(form_id)
            .bind(form_version_major)
            .fetch_one(pool)
            .await?
    };

    sqlx::query("DELETE FROM report_field_bindings WHERE report_id = $1")
        .bind(report_id)
        .execute(pool)
        .await?;
    sqlx::query(
        r#"
        INSERT INTO report_field_bindings
            (report_id, logical_key, source_field_key, missing_policy, position)
        VALUES ($1, $2, $3, $4::missing_data_policy, 0)
        "#,
    )
    .bind(report_id)
    .bind(&report.logical_key)
    .bind(&report.source_field_key)
    .bind(&report.missing_policy)
    .execute(pool)
    .await?;

    Ok(report_id)
}

fn parse_semantic_version_label(version_label: &str) -> ApiResult<(i32, i32, i32)> {
    if let Some((major, minor, patch)) = parse_strict_semantic_version(version_label) {
        return Ok((major, minor, patch));
    }
    if let Some(major) = parse_legacy_major_version(version_label) {
        return Ok((major, 0, 0));
    }
    Err(ApiError::BadRequest(format!(
        "invalid semantic version '{version_label}'"
    )))
}

fn parse_strict_semantic_version(version_label: &str) -> Option<(i32, i32, i32)> {
    let mut parts = version_label.split('.');
    let major = parts.next()?.parse().ok()?;
    let minor = parts.next()?.parse().ok()?;
    let patch = parts.next()?.parse().ok()?;
    if parts.next().is_some() {
        return None;
    }
    Some((major, minor, patch))
}

fn parse_legacy_major_version(version_label: &str) -> Option<i32> {
    let digits = version_label
        .trim()
        .rsplit(|character: char| !character.is_ascii_digit())
        .next()?;
    if digits.is_empty() || !digits.chars().all(|character| character.is_ascii_digit()) {
        return None;
    }
    digits.parse().ok()
}

async fn current_form_major(pool: &PgPool, form_id: Uuid) -> ApiResult<Option<i32>> {
    sqlx::query_scalar(
        r#"
        SELECT version_major
        FROM form_versions
        WHERE form_id = $1
          AND status = 'published'::form_version_status
          AND version_major IS NOT NULL
        ORDER BY
            version_major DESC,
            version_minor DESC NULLS LAST,
            version_patch DESC NULLS LAST,
            published_at DESC NULLS LAST,
            created_at DESC
        LIMIT 1
        "#,
    )
    .bind(form_id)
    .fetch_optional(pool)
    .await
    .map_err(Into::into)
}

async fn ensure_chart(pool: &PgPool, name: &str, report_id: Uuid) -> ApiResult<Uuid> {
    if let Some(id) = sqlx::query_scalar("SELECT id FROM charts WHERE name = $1")
        .bind(name)
        .fetch_optional(pool)
        .await?
    {
        sqlx::query("UPDATE charts SET report_id = $1, chart_type = 'table' WHERE id = $2")
            .bind(report_id)
            .bind(id)
            .execute(pool)
            .await?;
        return Ok(id);
    }

    Ok(sqlx::query_scalar(
        "INSERT INTO charts (name, report_id, chart_type) VALUES ($1, $2, 'table') RETURNING id",
    )
    .bind(name)
    .bind(report_id)
    .fetch_one(pool)
    .await?)
}

async fn ensure_dashboard(pool: &PgPool, name: &str) -> ApiResult<Uuid> {
    if let Some(id) = sqlx::query_scalar("SELECT id FROM dashboards WHERE name = $1")
        .bind(name)
        .fetch_optional(pool)
        .await?
    {
        return Ok(id);
    }

    Ok(
        sqlx::query_scalar("INSERT INTO dashboards (name) VALUES ($1) RETURNING id")
            .bind(name)
            .fetch_one(pool)
            .await?,
    )
}

async fn ensure_dashboard_component(
    pool: &PgPool,
    dashboard_id: Uuid,
    chart_id: Uuid,
) -> ApiResult<Uuid> {
    sqlx::query("DELETE FROM dashboard_components WHERE dashboard_id = $1")
        .bind(dashboard_id)
        .execute(pool)
        .await?;

    Ok(sqlx::query_scalar(
        r#"
        INSERT INTO dashboard_components (dashboard_id, chart_id, position, config)
        VALUES ($1, $2, 0, '{"title":"Legacy rehearsal report"}'::jsonb)
        RETURNING id
        "#,
    )
    .bind(dashboard_id)
    .bind(chart_id)
    .fetch_one(pool)
    .await?)
}

fn node_type_id_for_scope<'a>(
    scope: &str,
    node_types: impl IntoIterator<Item = (&'a str, Uuid)>,
) -> ApiResult<Uuid> {
    node_types
        .into_iter()
        .find_map(|(key, id)| key.eq_ignore_ascii_case(scope).then_some(id))
        .ok_or_else(|| ApiError::BadRequest(format!("unknown form scope node type '{scope}'")))
}

fn node_id_for_submission_node_type<'a>(
    scope: &str,
    nodes: impl IntoIterator<Item = (&'a str, Uuid)>,
) -> ApiResult<Uuid> {
    nodes
        .into_iter()
        .find_map(|(key, id)| key.eq_ignore_ascii_case(scope).then_some(id))
        .ok_or_else(|| ApiError::BadRequest(format!("unknown submission node type '{scope}'")))
}

fn default_true() -> bool {
    true
}

fn default_missing_policy() -> String {
    "null".to_string()
}

#[cfg(test)]
mod tests {
    use serde_json::Value;

    use super::{dry_run_legacy_fixture_str, validate_legacy_fixture_str};

    const LEGACY_FIXTURE: &str = include_str!("../../../fixtures/legacy-rehearsal.json");

    #[test]
    fn validation_accepts_representative_legacy_fixture() {
        let report =
            validate_legacy_fixture_str(LEGACY_FIXTURE).expect("fixture should deserialize");

        assert!(report.is_clean());
        assert_eq!(report.issue_count, 0);
    }

    #[test]
    fn dry_run_reports_whether_fixture_would_import() {
        let report =
            dry_run_legacy_fixture_str(LEGACY_FIXTURE).expect("fixture should deserialize");

        assert_eq!(report.fixture_name, "legacy-rehearsal");
        assert!(report.would_import);
        assert_eq!(report.validation.issue_count, 0);
    }

    #[test]
    fn validation_reports_actionable_mapping_and_value_issues() {
        let mut fixture: Value =
            serde_json::from_str(LEGACY_FIXTURE).expect("fixture should parse as json");
        fixture["program"]["legacy_id"] = fixture["partner"]["legacy_id"].clone();
        fixture["form"]["scope_node_type"] = Value::String("county".to_string());
        fixture["submission"]["node_type"] = Value::String("county".to_string());
        fixture["form"]["sections"][0]["fields"][1]["key"] =
            fixture["form"]["sections"][0]["fields"][0]["key"].clone();
        fixture["form"]["sections"][0]["fields"][2]["field_type"] =
            Value::String("file_upload".to_string());
        fixture["submission"]["values"]["participants"] = Value::String("forty-two".to_string());
        fixture["submission"]["values"]["unknown_field"] = Value::String("ignored".to_string());
        fixture["report"]["source_field_key"] = Value::String("missing_field".to_string());
        fixture["report"]["missing_policy"] = Value::String("drop_column".to_string());

        let report = validate_legacy_fixture_str(&fixture.to_string())
            .expect("invalid fixture should still deserialize");
        let issue_codes = report
            .issues
            .iter()
            .map(|issue| issue.code.as_str())
            .collect::<Vec<_>>();

        assert!(issue_codes.contains(&"duplicate_node_legacy_id"));
        assert!(issue_codes.contains(&"unknown_node_type_scope"));
        assert!(issue_codes.contains(&"duplicate_form_field_key"));
        assert!(issue_codes.contains(&"unsupported_field_type"));
        assert!(issue_codes.contains(&"unknown_submission_field"));
        assert!(issue_codes.contains(&"unknown_report_source_field"));
        assert!(issue_codes.contains(&"unsupported_missing_policy"));
    }
}
