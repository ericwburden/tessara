//! Deterministic demo data seeding for local development and smoke tests.

use std::collections::HashMap;

use axum::{Json, extract::State, http::HeaderMap};
use serde::Serialize;
use serde_json::{Value, json};
use sqlx::{PgPool, Row};
use uuid::Uuid;

use crate::{
    analytics, auth,
    db::AppState,
    error::{ApiError, ApiResult},
};

const DEMO_SEED_VERSION: &str = "uat-demo-v1";

#[derive(Serialize)]
pub struct DemoNodeCounts {
    pub partners: i64,
    pub programs: i64,
    pub activities: i64,
    pub sessions: i64,
}

/// Entity identifiers and counts produced by the deterministic demo seed workflow.
#[derive(Serialize)]
pub struct DemoSeedSummary {
    pub seed_version: &'static str,
    pub node_counts: DemoNodeCounts,
    pub form_count: i64,
    pub draft_submission_count: i64,
    pub submitted_submission_count: i64,
    pub report_count: i64,
    pub dashboard_count: i64,
    pub organization_node_id: Uuid,
    pub form_id: Uuid,
    pub form_version_id: Uuid,
    pub submission_id: Uuid,
    pub report_id: Uuid,
    pub chart_id: Uuid,
    pub dashboard_id: Uuid,
    pub partner_node_id: Uuid,
    pub program_node_id: Uuid,
    pub activity_node_id: Uuid,
    pub session_node_id: Uuid,
    pub partner_form_id: Uuid,
    pub program_form_id: Uuid,
    pub activity_form_id: Uuid,
    pub session_form_id: Uuid,
    pub partner_form_version_id: Uuid,
    pub program_form_version_id: Uuid,
    pub activity_form_version_id: Uuid,
    pub session_form_version_id: Uuid,
    pub analytics_values: i64,
}

struct MetadataFieldDef<'a> {
    key: &'a str,
    label: &'a str,
    field_type: &'a str,
    required: bool,
}

struct FormFieldDef<'a> {
    key: &'a str,
    label: &'a str,
    field_type: &'a str,
    required: bool,
    position: i32,
}

struct DemoNodeSpec<'a> {
    name: &'a str,
    metadata: Vec<(&'a str, Value)>,
}

struct DemoFormSpec<'a> {
    name: &'a str,
    slug: &'a str,
    scope_node_type_id: Uuid,
    compatibility_group_name: &'a str,
    version_label: &'a str,
    section_title: &'a str,
    fields: Vec<FormFieldDef<'a>>,
}

struct ReportBinding<'a> {
    logical_key: &'a str,
    source_field_key: &'a str,
}

struct SeedSubmissionSpec<'a> {
    seed_key: &'a str,
    status: &'a str,
    values: Vec<(&'a str, Value)>,
}

struct EnsuredForm {
    form_id: Uuid,
    form_version_id: Uuid,
}

pub(crate) async fn seed_demo_endpoint(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> ApiResult<Json<DemoSeedSummary>> {
    auth::require_capability(&state.pool, &headers, "admin:all").await?;
    Ok(Json(seed_demo(&state.pool).await?))
}

/// Seeds an idempotent end-to-end Tessara UAT demo dataset.
pub async fn seed_demo(pool: &PgPool) -> ApiResult<DemoSeedSummary> {
    let account_id = require_dev_admin_account(pool).await?;

    let partner_type_id = ensure_node_type(pool, "Partner", "partner").await?;
    let program_type_id = ensure_node_type(pool, "Program", "program").await?;
    let activity_type_id = ensure_node_type(pool, "Activity", "activity").await?;
    let session_type_id = ensure_node_type(pool, "Session", "session").await?;

    ensure_node_type_relationship(pool, partner_type_id, program_type_id).await?;
    ensure_node_type_relationship(pool, program_type_id, activity_type_id).await?;
    ensure_node_type_relationship(pool, activity_type_id, session_type_id).await?;

    let partner_fields = ensure_metadata_fields(
        pool,
        partner_type_id,
        &[
            MetadataFieldDef {
                key: "legacy_id",
                label: "Legacy ID",
                field_type: "text",
                required: true,
            },
            MetadataFieldDef {
                key: "region",
                label: "Region",
                field_type: "single_choice",
                required: true,
            },
            MetadataFieldDef {
                key: "active_contract",
                label: "Active Contract",
                field_type: "boolean",
                required: true,
            },
            MetadataFieldDef {
                key: "partner_since",
                label: "Partner Since",
                field_type: "date",
                required: false,
            },
            MetadataFieldDef {
                key: "focus_areas",
                label: "Focus Areas",
                field_type: "multi_choice",
                required: false,
            },
        ],
    )
    .await?;
    let program_fields = ensure_metadata_fields(
        pool,
        program_type_id,
        &[
            MetadataFieldDef {
                key: "legacy_id",
                label: "Legacy ID",
                field_type: "text",
                required: true,
            },
            MetadataFieldDef {
                key: "program_code",
                label: "Program Code",
                field_type: "text",
                required: true,
            },
            MetadataFieldDef {
                key: "annual_target",
                label: "Annual Target",
                field_type: "number",
                required: false,
            },
            MetadataFieldDef {
                key: "funded",
                label: "Funded",
                field_type: "boolean",
                required: true,
            },
            MetadataFieldDef {
                key: "service_window_start",
                label: "Service Window Start",
                field_type: "date",
                required: false,
            },
        ],
    )
    .await?;
    let activity_fields = ensure_metadata_fields(
        pool,
        activity_type_id,
        &[
            MetadataFieldDef {
                key: "legacy_id",
                label: "Legacy ID",
                field_type: "text",
                required: true,
            },
            MetadataFieldDef {
                key: "delivery_mode",
                label: "Delivery Mode",
                field_type: "single_choice",
                required: true,
            },
            MetadataFieldDef {
                key: "planned_participants",
                label: "Planned Participants",
                field_type: "number",
                required: false,
            },
            MetadataFieldDef {
                key: "focus_tags",
                label: "Focus Tags",
                field_type: "multi_choice",
                required: false,
            },
            MetadataFieldDef {
                key: "launch_date",
                label: "Launch Date",
                field_type: "date",
                required: false,
            },
        ],
    )
    .await?;
    let session_fields = ensure_metadata_fields(
        pool,
        session_type_id,
        &[
            MetadataFieldDef {
                key: "legacy_id",
                label: "Legacy ID",
                field_type: "text",
                required: true,
            },
            MetadataFieldDef {
                key: "session_date",
                label: "Session Date",
                field_type: "date",
                required: true,
            },
            MetadataFieldDef {
                key: "capacity",
                label: "Capacity",
                field_type: "number",
                required: false,
            },
            MetadataFieldDef {
                key: "cancelled",
                label: "Cancelled",
                field_type: "boolean",
                required: true,
            },
            MetadataFieldDef {
                key: "topics",
                label: "Topics",
                field_type: "multi_choice",
                required: false,
            },
            MetadataFieldDef {
                key: "room_label",
                label: "Room Label",
                field_type: "text",
                required: false,
            },
        ],
    )
    .await?;

    let partner_a = ensure_demo_node(
        pool,
        partner_type_id,
        None,
        &partner_fields,
        DemoNodeSpec {
            name: "Demo Partner North Star Services",
            metadata: vec![
                ("legacy_id", json!("partner-1001")),
                ("region", json!("north")),
                ("active_contract", json!(true)),
                ("partner_since", json!("2022-01-15")),
                ("focus_areas", json!(["family_support", "youth_services"])),
            ],
        },
    )
    .await?;
    let partner_b = ensure_demo_node(
        pool,
        partner_type_id,
        None,
        &partner_fields,
        DemoNodeSpec {
            name: "Demo Partner Community Bridge",
            metadata: vec![
                ("legacy_id", json!("partner-1002")),
                ("region", json!("south")),
                ("active_contract", json!(false)),
                ("partner_since", Value::Null),
                ("focus_areas", Value::Null),
            ],
        },
    )
    .await?;

    let program_a = ensure_demo_node(
        pool,
        program_type_id,
        Some(partner_a),
        &program_fields,
        DemoNodeSpec {
            name: "Demo Program Family Outreach",
            metadata: vec![
                ("legacy_id", json!("program-2001")),
                ("program_code", json!("FO-01")),
                ("annual_target", json!(120)),
                ("funded", json!(true)),
                ("service_window_start", json!("2026-01-10")),
            ],
        },
    )
    .await?;
    let program_b = ensure_demo_node(
        pool,
        program_type_id,
        Some(partner_a),
        &program_fields,
        DemoNodeSpec {
            name: "Demo Program Youth Mentoring",
            metadata: vec![
                ("legacy_id", json!("program-2002")),
                ("program_code", json!("YM-02")),
                ("annual_target", json!(80)),
                ("funded", json!(true)),
                ("service_window_start", json!("2026-02-01")),
            ],
        },
    )
    .await?;
    let program_c = ensure_demo_node(
        pool,
        program_type_id,
        Some(partner_b),
        &program_fields,
        DemoNodeSpec {
            name: "Demo Program Workforce Readiness",
            metadata: vec![
                ("legacy_id", json!("program-2003")),
                ("program_code", json!("WR-03")),
                ("annual_target", json!(150)),
                ("funded", json!(true)),
                ("service_window_start", json!("2026-03-15")),
            ],
        },
    )
    .await?;
    let program_d = ensure_demo_node(
        pool,
        program_type_id,
        Some(partner_b),
        &program_fields,
        DemoNodeSpec {
            name: "Demo Program Health Navigation",
            metadata: vec![
                ("legacy_id", json!("program-2004")),
                ("program_code", json!("HN-04")),
                ("annual_target", Value::Null),
                ("funded", json!(false)),
                ("service_window_start", Value::Null),
            ],
        },
    )
    .await?;

    let activity_a = ensure_demo_node(
        pool,
        activity_type_id,
        Some(program_a),
        &activity_fields,
        DemoNodeSpec {
            name: "Demo Activity Intake and Orientation",
            metadata: vec![
                ("legacy_id", json!("activity-3001")),
                ("delivery_mode", json!("in_person")),
                ("planned_participants", json!(25)),
                ("focus_tags", json!(["orientation", "enrollment"])),
                ("launch_date", json!("2026-04-01")),
            ],
        },
    )
    .await?;
    let activity_b = ensure_demo_node(
        pool,
        activity_type_id,
        Some(program_a),
        &activity_fields,
        DemoNodeSpec {
            name: "Demo Activity Family Workshops",
            metadata: vec![
                ("legacy_id", json!("activity-3002")),
                ("delivery_mode", json!("hybrid")),
                ("planned_participants", json!(18)),
                ("focus_tags", json!(["family_support", "wellness"])),
                ("launch_date", json!("2026-04-12")),
            ],
        },
    )
    .await?;
    let activity_c = ensure_demo_node(
        pool,
        activity_type_id,
        Some(program_b),
        &activity_fields,
        DemoNodeSpec {
            name: "Demo Activity Mentor Match",
            metadata: vec![
                ("legacy_id", json!("activity-3003")),
                ("delivery_mode", json!("remote")),
                ("planned_participants", json!(30)),
                ("focus_tags", json!(["mentoring", "youth_services"])),
                ("launch_date", json!("2026-05-01")),
            ],
        },
    )
    .await?;
    let activity_d = ensure_demo_node(
        pool,
        activity_type_id,
        Some(program_b),
        &activity_fields,
        DemoNodeSpec {
            name: "Demo Activity After School Check-ins",
            metadata: vec![
                ("legacy_id", json!("activity-3004")),
                ("delivery_mode", json!("in_person")),
                ("planned_participants", json!(22)),
                ("focus_tags", json!(["after_school"])),
                ("launch_date", json!("2026-05-15")),
            ],
        },
    )
    .await?;
    let activity_e = ensure_demo_node(
        pool,
        activity_type_id,
        Some(program_c),
        &activity_fields,
        DemoNodeSpec {
            name: "Demo Activity Job Coaching",
            metadata: vec![
                ("legacy_id", json!("activity-3005")),
                ("delivery_mode", json!("hybrid")),
                ("planned_participants", json!(16)),
                ("focus_tags", Value::Null),
                ("launch_date", json!("2026-06-03")),
            ],
        },
    )
    .await?;
    let activity_f = ensure_demo_node(
        pool,
        activity_type_id,
        Some(program_d),
        &activity_fields,
        DemoNodeSpec {
            name: "Demo Activity Enrollment Navigation",
            metadata: vec![
                ("legacy_id", json!("activity-3006")),
                ("delivery_mode", json!("remote")),
                ("planned_participants", json!(12)),
                ("focus_tags", json!(["benefits", "intake"])),
                ("launch_date", Value::Null),
            ],
        },
    )
    .await?;

    let session_a = ensure_demo_node(
        pool,
        session_type_id,
        Some(activity_a),
        &session_fields,
        DemoNodeSpec {
            name: "Demo Session April Orientation",
            metadata: vec![
                ("legacy_id", json!("session-4001")),
                ("session_date", json!("2026-04-08")),
                ("capacity", json!(25)),
                ("cancelled", json!(false)),
                ("topics", json!(["intake", "welcome"])),
                ("room_label", json!("Room A")),
            ],
        },
    )
    .await?;
    let session_b = ensure_demo_node(
        pool,
        session_type_id,
        Some(activity_a),
        &session_fields,
        DemoNodeSpec {
            name: "Demo Session May Orientation",
            metadata: vec![
                ("legacy_id", json!("session-4002")),
                ("session_date", json!("2026-05-06")),
                ("capacity", json!(20)),
                ("cancelled", json!(false)),
                ("topics", json!(["intake", "follow_up"])),
                ("room_label", json!("Room B")),
            ],
        },
    )
    .await?;
    let _session_c = ensure_demo_node(
        pool,
        session_type_id,
        Some(activity_b),
        &session_fields,
        DemoNodeSpec {
            name: "Demo Session Spring Family Workshop",
            metadata: vec![
                ("legacy_id", json!("session-4003")),
                ("session_date", json!("2026-04-20")),
                ("capacity", json!(18)),
                ("cancelled", json!(false)),
                ("topics", json!(["wellness", "family_support"])),
                ("room_label", json!("Workshop Hall")),
            ],
        },
    )
    .await?;
    let _session_d = ensure_demo_node(
        pool,
        session_type_id,
        Some(activity_b),
        &session_fields,
        DemoNodeSpec {
            name: "Demo Session Summer Family Workshop",
            metadata: vec![
                ("legacy_id", json!("session-4004")),
                ("session_date", json!("2026-06-18")),
                ("capacity", json!(16)),
                ("cancelled", json!(false)),
                ("topics", json!(["nutrition", "family_support"])),
                ("room_label", json!("Workshop Hall")),
            ],
        },
    )
    .await?;
    let _session_e = ensure_demo_node(
        pool,
        session_type_id,
        Some(activity_c),
        &session_fields,
        DemoNodeSpec {
            name: "Demo Session Mentor Kickoff",
            metadata: vec![
                ("legacy_id", json!("session-4005")),
                ("session_date", json!("2026-05-12")),
                ("capacity", json!(30)),
                ("cancelled", json!(false)),
                ("topics", json!(["mentoring", "onboarding"])),
                ("room_label", json!("Studio 2")),
            ],
        },
    )
    .await?;
    let _session_f = ensure_demo_node(
        pool,
        session_type_id,
        Some(activity_d),
        &session_fields,
        DemoNodeSpec {
            name: "Demo Session Mentor Follow-up",
            metadata: vec![
                ("legacy_id", json!("session-4006")),
                ("session_date", json!("2026-05-26")),
                ("capacity", json!(14)),
                ("cancelled", json!(false)),
                ("topics", json!(["check_in", "attendance"])),
                ("room_label", json!("Studio 4")),
            ],
        },
    )
    .await?;
    let session_g = ensure_demo_node(
        pool,
        session_type_id,
        Some(activity_e),
        &session_fields,
        DemoNodeSpec {
            name: "Demo Session Resume Lab",
            metadata: vec![
                ("legacy_id", json!("session-4007")),
                ("session_date", json!("2026-06-10")),
                ("capacity", json!(15)),
                ("cancelled", json!(false)),
                ("topics", json!(["resume", "job_search"])),
                ("room_label", json!("Career Center")),
            ],
        },
    )
    .await?;
    let _session_h = ensure_demo_node(
        pool,
        session_type_id,
        Some(activity_f),
        &session_fields,
        DemoNodeSpec {
            name: "Demo Session Benefits Intake",
            metadata: vec![
                ("legacy_id", json!("session-4008")),
                ("session_date", json!("2026-06-22")),
                ("capacity", json!(10)),
                ("cancelled", json!(false)),
                ("topics", Value::Null),
                ("room_label", Value::Null),
            ],
        },
    )
    .await?;

    let partner_form = ensure_demo_form(
        pool,
        DemoFormSpec {
            name: "Demo Partner Profile",
            slug: "demo-partner-profile",
            scope_node_type_id: partner_type_id,
            compatibility_group_name: "Demo Partner Profile Compatible",
            version_label: "v1",
            section_title: "Partner Profile",
            fields: vec![
                FormFieldDef {
                    key: "contact_name",
                    label: "Contact Name",
                    field_type: "text",
                    required: true,
                    position: 1,
                },
                FormFieldDef {
                    key: "reporting_region",
                    label: "Reporting Region",
                    field_type: "single_choice",
                    required: true,
                    position: 2,
                },
                FormFieldDef {
                    key: "compliance_confirmed",
                    label: "Compliance Confirmed",
                    field_type: "boolean",
                    required: true,
                    position: 3,
                },
                FormFieldDef {
                    key: "review_date",
                    label: "Review Date",
                    field_type: "date",
                    required: true,
                    position: 4,
                },
                FormFieldDef {
                    key: "service_focus",
                    label: "Service Focus",
                    field_type: "multi_choice",
                    required: false,
                    position: 5,
                },
            ],
        },
    )
    .await?;
    let program_form = ensure_demo_form(
        pool,
        DemoFormSpec {
            name: "Demo Program Snapshot",
            slug: "demo-program-snapshot",
            scope_node_type_id: program_type_id,
            compatibility_group_name: "Demo Program Snapshot Compatible",
            version_label: "v1",
            section_title: "Program Snapshot",
            fields: vec![
                FormFieldDef {
                    key: "snapshot_notes",
                    label: "Snapshot Notes",
                    field_type: "text",
                    required: true,
                    position: 1,
                },
                FormFieldDef {
                    key: "participant_target",
                    label: "Participant Target",
                    field_type: "number",
                    required: true,
                    position: 2,
                },
                FormFieldDef {
                    key: "funding_confirmed",
                    label: "Funding Confirmed",
                    field_type: "boolean",
                    required: true,
                    position: 3,
                },
                FormFieldDef {
                    key: "review_window_start",
                    label: "Review Window Start",
                    field_type: "date",
                    required: true,
                    position: 4,
                },
            ],
        },
    )
    .await?;
    let activity_form = ensure_demo_form(
        pool,
        DemoFormSpec {
            name: "Demo Activity Plan",
            slug: "demo-activity-plan",
            scope_node_type_id: activity_type_id,
            compatibility_group_name: "Demo Activity Plan Compatible",
            version_label: "v1",
            section_title: "Activity Plan",
            fields: vec![
                FormFieldDef {
                    key: "activity_summary",
                    label: "Activity Summary",
                    field_type: "text",
                    required: true,
                    position: 1,
                },
                FormFieldDef {
                    key: "delivery_mode",
                    label: "Delivery Mode",
                    field_type: "single_choice",
                    required: true,
                    position: 2,
                },
                FormFieldDef {
                    key: "focus_tags",
                    label: "Focus Tags",
                    field_type: "multi_choice",
                    required: false,
                    position: 3,
                },
                FormFieldDef {
                    key: "expected_attendees",
                    label: "Expected Attendees",
                    field_type: "number",
                    required: true,
                    position: 4,
                },
            ],
        },
    )
    .await?;
    let session_form = ensure_demo_form(
        pool,
        DemoFormSpec {
            name: "Demo Session Log",
            slug: "demo-session-log",
            scope_node_type_id: session_type_id,
            compatibility_group_name: "Demo Session Log Compatible",
            version_label: "v1",
            section_title: "Session Log",
            fields: vec![
                FormFieldDef {
                    key: "session_date",
                    label: "Session Date",
                    field_type: "date",
                    required: true,
                    position: 1,
                },
                FormFieldDef {
                    key: "participants",
                    label: "Participants",
                    field_type: "number",
                    required: true,
                    position: 2,
                },
                FormFieldDef {
                    key: "completed_as_planned",
                    label: "Completed As Planned",
                    field_type: "boolean",
                    required: true,
                    position: 3,
                },
                FormFieldDef {
                    key: "facilitator_notes",
                    label: "Facilitator Notes",
                    field_type: "text",
                    required: false,
                    position: 4,
                },
                FormFieldDef {
                    key: "topics_covered",
                    label: "Topics Covered",
                    field_type: "multi_choice",
                    required: false,
                    position: 5,
                },
            ],
        },
    )
    .await?;

    let _partner_submitted_a = ensure_seed_submission(
        pool,
        account_id,
        partner_form.form_version_id,
        partner_a,
        SeedSubmissionSpec {
            seed_key: "seed_demo:partner-submitted-a",
            status: "submitted",
            values: vec![
                ("contact_name", json!("Avery Johnson")),
                ("reporting_region", json!("north")),
                ("compliance_confirmed", json!(true)),
                ("review_date", json!("2026-03-31")),
                ("service_focus", json!(["family_support", "youth_services"])),
            ],
        },
    )
    .await?;
    ensure_seed_submission(
        pool,
        account_id,
        partner_form.form_version_id,
        partner_b,
        SeedSubmissionSpec {
            seed_key: "seed_demo:partner-submitted-b",
            status: "submitted",
            values: vec![
                ("contact_name", json!("Morgan Lee")),
                ("reporting_region", json!("south")),
                ("compliance_confirmed", json!(false)),
                ("review_date", json!("2026-04-02")),
                ("service_focus", Value::Null),
            ],
        },
    )
    .await?;
    ensure_seed_submission(
        pool,
        account_id,
        partner_form.form_version_id,
        partner_a,
        SeedSubmissionSpec {
            seed_key: "seed_demo:partner-draft-a",
            status: "draft",
            values: vec![
                ("contact_name", json!("Avery Johnson")),
                ("reporting_region", json!("north")),
                ("compliance_confirmed", json!(true)),
                ("review_date", json!("2026-04-20")),
            ],
        },
    )
    .await?;

    ensure_seed_submission(
        pool,
        account_id,
        program_form.form_version_id,
        program_a,
        SeedSubmissionSpec {
            seed_key: "seed_demo:program-submitted-a",
            status: "submitted",
            values: vec![
                ("snapshot_notes", json!("Enrollment remains on track.")),
                ("participant_target", json!(120)),
                ("funding_confirmed", json!(true)),
                ("review_window_start", json!("2026-04-01")),
            ],
        },
    )
    .await?;
    ensure_seed_submission(
        pool,
        account_id,
        program_form.form_version_id,
        program_c,
        SeedSubmissionSpec {
            seed_key: "seed_demo:program-submitted-b",
            status: "submitted",
            values: vec![
                (
                    "snapshot_notes",
                    json!("Hiring delays require schedule changes."),
                ),
                ("participant_target", json!(150)),
                ("funding_confirmed", json!(true)),
                ("review_window_start", json!("2026-04-15")),
            ],
        },
    )
    .await?;
    ensure_seed_submission(
        pool,
        account_id,
        program_form.form_version_id,
        program_b,
        SeedSubmissionSpec {
            seed_key: "seed_demo:program-draft-a",
            status: "draft",
            values: vec![
                ("snapshot_notes", json!("Partner review is in progress.")),
                ("participant_target", json!(90)),
                ("funding_confirmed", json!(true)),
                ("review_window_start", json!("2026-05-01")),
            ],
        },
    )
    .await?;
    ensure_seed_submission(
        pool,
        account_id,
        activity_form.form_version_id,
        activity_a,
        SeedSubmissionSpec {
            seed_key: "seed_demo:activity-submitted-a",
            status: "submitted",
            values: vec![
                (
                    "activity_summary",
                    json!("Orientation sessions prepared for new intake."),
                ),
                ("delivery_mode", json!("in_person")),
                ("focus_tags", json!(["orientation", "intake"])),
                ("expected_attendees", json!(25)),
            ],
        },
    )
    .await?;
    ensure_seed_submission(
        pool,
        account_id,
        activity_form.form_version_id,
        activity_e,
        SeedSubmissionSpec {
            seed_key: "seed_demo:activity-submitted-b",
            status: "submitted",
            values: vec![
                (
                    "activity_summary",
                    json!("Resume lab demand exceeded expectations."),
                ),
                ("delivery_mode", json!("hybrid")),
                ("focus_tags", Value::Null),
                ("expected_attendees", json!(16)),
            ],
        },
    )
    .await?;
    ensure_seed_submission(
        pool,
        account_id,
        activity_form.form_version_id,
        activity_c,
        SeedSubmissionSpec {
            seed_key: "seed_demo:activity-draft-a",
            status: "draft",
            values: vec![
                (
                    "activity_summary",
                    json!("Mentor kickoff planning is underway."),
                ),
                ("delivery_mode", json!("remote")),
                ("expected_attendees", json!(30)),
            ],
        },
    )
    .await?;

    let session_submitted_a = ensure_seed_submission(
        pool,
        account_id,
        session_form.form_version_id,
        session_a,
        SeedSubmissionSpec {
            seed_key: "seed_demo:session-submitted-a",
            status: "submitted",
            values: vec![
                ("session_date", json!("2026-04-08")),
                ("participants", json!(42)),
                ("completed_as_planned", json!(true)),
                (
                    "facilitator_notes",
                    json!("Orientation completed with strong attendance."),
                ),
                ("topics_covered", json!(["intake", "welcome"])),
            ],
        },
    )
    .await?;
    ensure_seed_submission(
        pool,
        account_id,
        session_form.form_version_id,
        session_g,
        SeedSubmissionSpec {
            seed_key: "seed_demo:session-submitted-b",
            status: "submitted",
            values: vec![
                ("session_date", json!("2026-06-10")),
                ("participants", json!(18)),
                ("completed_as_planned", json!(true)),
                ("facilitator_notes", Value::Null),
                ("topics_covered", json!(["resume", "job_search"])),
            ],
        },
    )
    .await?;
    ensure_seed_submission(
        pool,
        account_id,
        session_form.form_version_id,
        session_b,
        SeedSubmissionSpec {
            seed_key: "seed_demo:session-draft-a",
            status: "draft",
            values: vec![
                ("session_date", json!("2026-05-06")),
                ("participants", json!(15)),
                ("completed_as_planned", json!(false)),
            ],
        },
    )
    .await?;

    let analytics_status = analytics::refresh_projection(pool).await?;

    let partner_report_id = ensure_report(
        pool,
        partner_form.form_id,
        "Demo Partner Profile Report",
        &[ReportBinding {
            logical_key: "contact_name",
            source_field_key: "contact_name",
        }],
    )
    .await?;
    let program_report_id = ensure_report(
        pool,
        program_form.form_id,
        "Demo Program Snapshot Report",
        &[ReportBinding {
            logical_key: "participant_target",
            source_field_key: "participant_target",
        }],
    )
    .await?;
    let activity_report_id = ensure_report(
        pool,
        activity_form.form_id,
        "Demo Activity Plan Report",
        &[ReportBinding {
            logical_key: "expected_attendees",
            source_field_key: "expected_attendees",
        }],
    )
    .await?;
    let session_report_id = ensure_report(
        pool,
        session_form.form_id,
        "Participants Report",
        &[ReportBinding {
            logical_key: "participants",
            source_field_key: "participants",
        }],
    )
    .await?;

    let partner_chart_id = ensure_chart(
        pool,
        "Demo Partner Profile Table",
        Some(partner_report_id),
        "table",
    )
    .await?;
    let program_chart_id = ensure_chart(
        pool,
        "Demo Program Snapshot Table",
        Some(program_report_id),
        "table",
    )
    .await?;
    let activity_chart_id = ensure_chart(
        pool,
        "Demo Activity Plan Table",
        Some(activity_report_id),
        "table",
    )
    .await?;
    let session_chart_id =
        ensure_chart(pool, "Participants Table", Some(session_report_id), "table").await?;

    let dashboard_id = ensure_dashboard(pool, "Demo Operations Dashboard").await?;
    replace_dashboard_components(
        pool,
        dashboard_id,
        &[
            (partner_chart_id, 0, json!({"title": "Partner Profile"})),
            (program_chart_id, 1, json!({"title": "Program Snapshot"})),
            (activity_chart_id, 2, json!({"title": "Activity Plan"})),
            (
                session_chart_id,
                3,
                json!({"title": "Session Participation"}),
            ),
        ],
    )
    .await?;

    Ok(DemoSeedSummary {
        seed_version: DEMO_SEED_VERSION,
        node_counts: DemoNodeCounts {
            partners: 2,
            programs: 4,
            activities: 6,
            sessions: 8,
        },
        form_count: 4,
        draft_submission_count: 4,
        submitted_submission_count: 8,
        report_count: 4,
        dashboard_count: 1,
        organization_node_id: session_a,
        form_id: session_form.form_id,
        form_version_id: session_form.form_version_id,
        submission_id: session_submitted_a,
        report_id: session_report_id,
        chart_id: session_chart_id,
        dashboard_id,
        partner_node_id: partner_a,
        program_node_id: program_a,
        activity_node_id: activity_a,
        session_node_id: session_a,
        partner_form_id: partner_form.form_id,
        program_form_id: program_form.form_id,
        activity_form_id: activity_form.form_id,
        session_form_id: session_form.form_id,
        partner_form_version_id: partner_form.form_version_id,
        program_form_version_id: program_form.form_version_id,
        activity_form_version_id: activity_form.form_version_id,
        session_form_version_id: session_form.form_version_id,
        analytics_values: analytics_status.value_count,
    })
}

async fn require_dev_admin_account(pool: &PgPool) -> ApiResult<Uuid> {
    sqlx::query_scalar("SELECT id FROM accounts WHERE email = 'admin@tessara.local' LIMIT 1")
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| ApiError::NotFound("dev admin account".into()))
}

async fn ensure_node_type(pool: &PgPool, name: &str, slug: &str) -> ApiResult<Uuid> {
    sqlx::query_scalar(
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
    .await
    .map_err(Into::into)
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

async fn ensure_metadata_fields(
    pool: &PgPool,
    node_type_id: Uuid,
    definitions: &[MetadataFieldDef<'_>],
) -> ApiResult<HashMap<String, Uuid>> {
    let mut field_ids = HashMap::new();
    for definition in definitions {
        let id = ensure_node_metadata_field(
            pool,
            node_type_id,
            definition.key,
            definition.label,
            definition.field_type,
            definition.required,
        )
        .await?;
        field_ids.insert(definition.key.to_string(), id);
    }
    Ok(field_ids)
}

async fn ensure_node_metadata_field(
    pool: &PgPool,
    node_type_id: Uuid,
    key: &str,
    label: &str,
    field_type: &str,
    required: bool,
) -> ApiResult<Uuid> {
    sqlx::query_scalar(
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
    .await
    .map_err(Into::into)
}

async fn ensure_demo_node(
    pool: &PgPool,
    node_type_id: Uuid,
    parent_node_id: Option<Uuid>,
    metadata_fields: &HashMap<String, Uuid>,
    spec: DemoNodeSpec<'_>,
) -> ApiResult<Uuid> {
    let node_id = ensure_node(pool, node_type_id, parent_node_id, spec.name).await?;
    for (key, value) in spec.metadata {
        if let Some(field_definition_id) = metadata_fields.get(key) {
            upsert_metadata_value(pool, node_id, *field_definition_id, value).await?;
        }
    }
    Ok(node_id)
}

async fn ensure_node(
    pool: &PgPool,
    node_type_id: Uuid,
    parent_node_id: Option<Uuid>,
    name: &str,
) -> ApiResult<Uuid> {
    if let Some(id) = sqlx::query_scalar(
        r#"
        SELECT id
        FROM nodes
        WHERE node_type_id = $1
          AND parent_node_id IS NOT DISTINCT FROM $2
          AND name = $3
        "#,
    )
    .bind(node_type_id)
    .bind(parent_node_id)
    .bind(name)
    .fetch_optional(pool)
    .await?
    {
        return Ok(id);
    }

    sqlx::query_scalar(
        "INSERT INTO nodes (node_type_id, parent_node_id, name) VALUES ($1, $2, $3) RETURNING id",
    )
    .bind(node_type_id)
    .bind(parent_node_id)
    .bind(name)
    .fetch_one(pool)
    .await
    .map_err(Into::into)
}

async fn upsert_metadata_value(
    pool: &PgPool,
    node_id: Uuid,
    field_definition_id: Uuid,
    value: Value,
) -> ApiResult<()> {
    if value.is_null() {
        sqlx::query(
            "DELETE FROM node_metadata_values WHERE node_id = $1 AND field_definition_id = $2",
        )
        .bind(node_id)
        .bind(field_definition_id)
        .execute(pool)
        .await?;
    } else {
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
    }
    Ok(())
}

async fn ensure_demo_form(pool: &PgPool, spec: DemoFormSpec<'_>) -> ApiResult<EnsuredForm> {
    let form_id = ensure_form(pool, spec.name, spec.slug, Some(spec.scope_node_type_id)).await?;
    let compatibility_group_id =
        ensure_compatibility_group(pool, form_id, spec.compatibility_group_name).await?;
    let form_version_id =
        ensure_form_version(pool, form_id, compatibility_group_id, spec.version_label).await?;
    let section_id = ensure_form_section(pool, form_version_id, spec.section_title, 1).await?;

    for field in spec.fields {
        ensure_form_field(
            pool,
            form_version_id,
            section_id,
            field.key,
            field.label,
            field.field_type,
            field.required,
            field.position,
        )
        .await?;
    }

    publish_form_version(pool, form_version_id).await?;

    Ok(EnsuredForm {
        form_id,
        form_version_id,
    })
}

async fn ensure_form(
    pool: &PgPool,
    name: &str,
    slug: &str,
    scope_node_type_id: Option<Uuid>,
) -> ApiResult<Uuid> {
    sqlx::query_scalar(
        r#"
        INSERT INTO forms (name, slug, scope_node_type_id)
        VALUES ($1, $2, $3)
        ON CONFLICT (slug)
        DO UPDATE SET name = EXCLUDED.name, scope_node_type_id = EXCLUDED.scope_node_type_id
        RETURNING id
        "#,
    )
    .bind(name)
    .bind(slug)
    .bind(scope_node_type_id)
    .fetch_one(pool)
    .await
    .map_err(Into::into)
}

async fn ensure_compatibility_group(pool: &PgPool, form_id: Uuid, name: &str) -> ApiResult<Uuid> {
    if let Some(id) =
        sqlx::query_scalar("SELECT id FROM compatibility_groups WHERE form_id = $1 AND name = $2")
            .bind(form_id)
            .bind(name)
            .fetch_optional(pool)
            .await?
    {
        return Ok(id);
    }

    sqlx::query_scalar(
        "INSERT INTO compatibility_groups (form_id, name) VALUES ($1, $2) RETURNING id",
    )
    .bind(form_id)
    .bind(name)
    .fetch_one(pool)
    .await
    .map_err(Into::into)
}

async fn ensure_form_version(
    pool: &PgPool,
    form_id: Uuid,
    compatibility_group_id: Uuid,
    version_label: &str,
) -> ApiResult<Uuid> {
    sqlx::query_scalar(
        r#"
        INSERT INTO form_versions (form_id, compatibility_group_id, version_label)
        VALUES ($1, $2, $3)
        ON CONFLICT (form_id, version_label)
        DO UPDATE SET compatibility_group_id = EXCLUDED.compatibility_group_id
        RETURNING id
        "#,
    )
    .bind(form_id)
    .bind(compatibility_group_id)
    .bind(version_label)
    .fetch_one(pool)
    .await
    .map_err(Into::into)
}

async fn ensure_form_section(
    pool: &PgPool,
    form_version_id: Uuid,
    title: &str,
    position: i32,
) -> ApiResult<Uuid> {
    if let Some(id) =
        sqlx::query_scalar("SELECT id FROM form_sections WHERE form_version_id = $1 AND title = $2")
            .bind(form_version_id)
            .bind(title)
            .fetch_optional(pool)
            .await?
    {
        return Ok(id);
    }

    sqlx::query_scalar(
        "INSERT INTO form_sections (form_version_id, title, position) VALUES ($1, $2, $3) RETURNING id",
    )
    .bind(form_version_id)
    .bind(title)
    .bind(position)
    .fetch_one(pool)
    .await
    .map_err(Into::into)
}

#[allow(clippy::too_many_arguments)]
async fn ensure_form_field(
    pool: &PgPool,
    form_version_id: Uuid,
    section_id: Uuid,
    key: &str,
    label: &str,
    field_type: &str,
    required: bool,
    position: i32,
) -> ApiResult<Uuid> {
    sqlx::query_scalar(
        r#"
        INSERT INTO form_fields
            (form_version_id, section_id, key, label, field_type, required, position)
        VALUES ($1, $2, $3, $4, $5::field_type, $6, $7)
        ON CONFLICT (form_version_id, key)
        DO UPDATE SET
            section_id = EXCLUDED.section_id,
            label = EXCLUDED.label,
            required = EXCLUDED.required,
            position = EXCLUDED.position
        RETURNING id
        "#,
    )
    .bind(form_version_id)
    .bind(section_id)
    .bind(key)
    .bind(label)
    .bind(field_type)
    .bind(required)
    .bind(position)
    .fetch_one(pool)
    .await
    .map_err(Into::into)
}

async fn publish_form_version(pool: &PgPool, form_version_id: Uuid) -> ApiResult<()> {
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

async fn ensure_seed_submission(
    pool: &PgPool,
    account_id: Uuid,
    form_version_id: Uuid,
    node_id: Uuid,
    spec: SeedSubmissionSpec<'_>,
) -> ApiResult<Uuid> {
    let submission_id = if let Some(id) = sqlx::query_scalar(
        r#"
        SELECT submission_id
        FROM submission_audit_events
        WHERE event_type = $1
        LIMIT 1
        "#,
    )
    .bind(spec.seed_key)
    .fetch_optional(pool)
    .await?
    {
        id
    } else {
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

        let submission_id: Uuid = sqlx::query_scalar(
            r#"
            INSERT INTO submissions (assignment_id, form_version_id, node_id, status, submitted_at)
            VALUES (
                $1,
                $2,
                $3,
                $4::submission_status,
                CASE WHEN $4 = 'submitted' THEN now() ELSE NULL END
            )
            RETURNING id
            "#,
        )
        .bind(assignment_id)
        .bind(form_version_id)
        .bind(node_id)
        .bind(spec.status)
        .fetch_one(pool)
        .await?;

        sqlx::query(
            r#"
            INSERT INTO submission_audit_events (submission_id, event_type, account_id)
            VALUES ($1, $2, $3)
            "#,
        )
        .bind(submission_id)
        .bind(spec.seed_key)
        .bind(account_id)
        .execute(pool)
        .await?;

        submission_id
    };

    let field_rows = sqlx::query(
        r#"
        SELECT id, key
        FROM form_fields
        WHERE form_version_id = $1
        "#,
    )
    .bind(form_version_id)
    .fetch_all(pool)
    .await?;

    let mut field_ids_by_key = HashMap::new();
    for row in field_rows {
        let field_id: Uuid = row.try_get("id")?;
        let key: String = row.try_get("key")?;
        field_ids_by_key.insert(key, field_id);
    }

    let mut retained_field_ids = Vec::new();
    for (key, value) in spec.values {
        let field_id = field_ids_by_key
            .get(key)
            .copied()
            .ok_or_else(|| ApiError::BadRequest(format!("unknown demo seed field '{key}'")))?;
        retained_field_ids.push(field_id);
        upsert_submission_value(pool, submission_id, field_id, value).await?;
    }

    sqlx::query(
        r#"
        DELETE FROM submission_values
        WHERE submission_id = $1
          AND NOT (field_id = ANY($2))
        "#,
    )
    .bind(submission_id)
    .bind(&retained_field_ids)
    .execute(pool)
    .await?;

    sqlx::query(
        "DELETE FROM submission_value_multi WHERE submission_id = $1 AND NOT (field_id = ANY($2))",
    )
    .bind(submission_id)
    .bind(&retained_field_ids)
    .execute(pool)
    .await?;

    if spec.status == "submitted" {
        sqlx::query(
            r#"
            UPDATE submissions
            SET status = 'submitted'::submission_status,
                submitted_at = COALESCE(submitted_at, now())
            WHERE id = $1
            "#,
        )
        .bind(submission_id)
        .execute(pool)
        .await?;
    } else {
        sqlx::query(
            r#"
            UPDATE submissions
            SET status = 'draft'::submission_status,
                submitted_at = NULL
            WHERE id = $1
            "#,
        )
        .bind(submission_id)
        .execute(pool)
        .await?;
    }

    Ok(submission_id)
}

async fn upsert_submission_value(
    pool: &PgPool,
    submission_id: Uuid,
    field_id: Uuid,
    value: Value,
) -> ApiResult<()> {
    if value.is_null() {
        sqlx::query("DELETE FROM submission_values WHERE submission_id = $1 AND field_id = $2")
            .bind(submission_id)
            .bind(field_id)
            .execute(pool)
            .await?;
        sqlx::query(
            "DELETE FROM submission_value_multi WHERE submission_id = $1 AND field_id = $2",
        )
        .bind(submission_id)
        .bind(field_id)
        .execute(pool)
        .await?;
        return Ok(());
    }

    sqlx::query(
        r#"
        INSERT INTO submission_values (submission_id, field_id, value)
        VALUES ($1, $2, $3)
        ON CONFLICT (submission_id, field_id)
        DO UPDATE SET value = EXCLUDED.value
        "#,
    )
    .bind(submission_id)
    .bind(field_id)
    .bind(&value)
    .execute(pool)
    .await?;

    sqlx::query("DELETE FROM submission_value_multi WHERE submission_id = $1 AND field_id = $2")
        .bind(submission_id)
        .bind(field_id)
        .execute(pool)
        .await?;

    if let Some(items) = value.as_array() {
        for item in items {
            let Some(item_value) = item.as_str() else {
                continue;
            };
            sqlx::query(
                r#"
                INSERT INTO submission_value_multi (submission_id, field_id, value)
                VALUES ($1, $2, $3)
                ON CONFLICT DO NOTHING
                "#,
            )
            .bind(submission_id)
            .bind(field_id)
            .bind(item_value)
            .execute(pool)
            .await?;
        }
    }

    Ok(())
}

async fn ensure_report(
    pool: &PgPool,
    form_id: Uuid,
    name: &str,
    bindings: &[ReportBinding<'_>],
) -> ApiResult<Uuid> {
    let report_id = if let Some(id) =
        sqlx::query_scalar("SELECT id FROM reports WHERE form_id = $1 AND name = $2")
            .bind(form_id)
            .bind(name)
            .fetch_optional(pool)
            .await?
    {
        id
    } else {
        sqlx::query_scalar("INSERT INTO reports (name, form_id) VALUES ($1, $2) RETURNING id")
            .bind(name)
            .bind(form_id)
            .fetch_one(pool)
            .await?
    };

    sqlx::query("DELETE FROM report_field_bindings WHERE report_id = $1")
        .bind(report_id)
        .execute(pool)
        .await?;

    for (position, binding) in bindings.iter().enumerate() {
        sqlx::query(
            r#"
            INSERT INTO report_field_bindings
                (report_id, logical_key, source_field_key, missing_policy, position)
            VALUES ($1, $2, $3, 'null'::missing_data_policy, $4)
            "#,
        )
        .bind(report_id)
        .bind(binding.logical_key)
        .bind(binding.source_field_key)
        .bind(position as i32)
        .execute(pool)
        .await?;
    }

    Ok(report_id)
}

async fn ensure_chart(
    pool: &PgPool,
    name: &str,
    report_id: Option<Uuid>,
    chart_type: &str,
) -> ApiResult<Uuid> {
    if let Some(id) = sqlx::query_scalar("SELECT id FROM charts WHERE name = $1")
        .bind(name)
        .fetch_optional(pool)
        .await?
    {
        sqlx::query("UPDATE charts SET report_id = $1, chart_type = $2 WHERE id = $3")
            .bind(report_id)
            .bind(chart_type)
            .bind(id)
            .execute(pool)
            .await?;
        return Ok(id);
    }

    sqlx::query_scalar(
        "INSERT INTO charts (name, report_id, chart_type) VALUES ($1, $2, $3) RETURNING id",
    )
    .bind(name)
    .bind(report_id)
    .bind(chart_type)
    .fetch_one(pool)
    .await
    .map_err(Into::into)
}

async fn ensure_dashboard(pool: &PgPool, name: &str) -> ApiResult<Uuid> {
    if let Some(id) = sqlx::query_scalar("SELECT id FROM dashboards WHERE name = $1")
        .bind(name)
        .fetch_optional(pool)
        .await?
    {
        return Ok(id);
    }

    sqlx::query_scalar("INSERT INTO dashboards (name) VALUES ($1) RETURNING id")
        .bind(name)
        .fetch_one(pool)
        .await
        .map_err(Into::into)
}

async fn replace_dashboard_components(
    pool: &PgPool,
    dashboard_id: Uuid,
    components: &[(Uuid, i32, Value)],
) -> ApiResult<()> {
    sqlx::query("DELETE FROM dashboard_components WHERE dashboard_id = $1")
        .bind(dashboard_id)
        .execute(pool)
        .await?;

    for (chart_id, position, config) in components {
        sqlx::query(
            r#"
            INSERT INTO dashboard_components (dashboard_id, chart_id, position, config)
            VALUES ($1, $2, $3, $4)
            "#,
        )
        .bind(dashboard_id)
        .bind(chart_id)
        .bind(position)
        .bind(config)
        .execute(pool)
        .await?;
    }

    Ok(())
}
