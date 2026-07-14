//! Deterministic demo data seeding for local development and smoke tests.

use axum::{Json, Router, extract::State, routing::post};
use serde::Serialize;
use serde_json::{Value, json};
use sqlx::PgPool;
use tessara_dashboards::GridRect;
use uuid::Uuid;

use crate::{
    auth::AuthenticatedRequest,
    db::AppState,
    error::{ApiError, ApiResult},
};

mod accounts;
mod analytics;
mod forms;
mod hierarchy;
mod responses;
mod workflows;

use accounts::{
    ensure_account_delegation, ensure_account_scope_assignment, ensure_demo_account,
    require_dev_admin_account,
};
use analytics::{
    DashboardComponentPlacementSeed, DatasetFieldBinding, ensure_component,
    ensure_component_with_config, ensure_dashboard, ensure_dataset, replace_dashboard_components,
};
use forms::{DemoFormSpec, FormFieldDef, ensure_demo_form, replace_form_scope_nodes};
use hierarchy::{
    DemoNodeSpec, MetadataFieldDef, ensure_demo_node, ensure_metadata_fields, ensure_node_type,
    ensure_node_type_relationship,
};
use responses::{SeedSubmissionSpec, ensure_seed_submission};
use workflows::{
    WorkflowStepSeed, ensure_program_checkpoint_workflow, ensure_single_form_workflow_assignment,
};

const DEMO_SEED_VERSION: &str = "uat-demo-v2";

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
    pub dataset_count: i64,
    pub dataset_revision_count: i64,
    pub component_count: i64,
    pub dashboard_count: i64,
    pub organization_node_id: Uuid,
    pub form_id: Uuid,
    pub form_version_id: Uuid,
    pub submission_id: Uuid,
    pub dataset_id: Uuid,
    pub dataset_revision_id: Uuid,
    pub component_id: Uuid,
    pub component_version_id: Uuid,
    pub dashboard_id: Uuid,
    pub partner_node_id: Uuid,
    pub program_node_id: Uuid,
    pub activity_node_id: Uuid,
    pub session_node_id: Uuid,
    pub partner_form_id: Uuid,
    pub program_form_id: Uuid,
    pub activity_form_id: Uuid,
    pub intake_activity_form_id: Uuid,
    pub workshop_activity_form_id: Uuid,
    pub session_form_id: Uuid,
    pub partner_form_version_id: Uuid,
    pub program_form_version_id: Uuid,
    pub activity_form_version_id: Uuid,
    pub intake_activity_form_version_id: Uuid,
    pub workshop_activity_form_version_id: Uuid,
    pub session_form_version_id: Uuid,
    pub program_workflow_id: Uuid,
    pub program_workflow_version_id: Uuid,
    pub program_workflow_assignment_id: Uuid,
    pub analytics_values: i64,
}

pub(crate) fn routes() -> Router<AppState> {
    Router::new().route("/api/demo/seed", post(seed_demo_endpoint))
}

pub(crate) async fn seed_demo_endpoint(
    State(state): State<AppState>,
    request: AuthenticatedRequest,
) -> ApiResult<Json<DemoSeedSummary>> {
    request.require_capability("admin:all")?;
    Ok(Json(seed_demo(&state.pool).await?))
}

/// Seeds the end-to-end Tessara UAT demo dataset into an otherwise empty app database.
pub async fn seed_demo(pool: &PgPool) -> ApiResult<DemoSeedSummary> {
    let account_id = require_dev_admin_account(pool).await?;
    require_demo_seed_target_empty(pool, account_id).await?;
    let operator_account_id = ensure_demo_account(
        pool,
        "operator@tessara.local",
        "Demo Operator",
        "operator",
        "tessara-dev-operator",
    )
    .await?;
    let delegator_account_id = ensure_demo_account(
        pool,
        "delegator@tessara.local",
        "Demo Delegator",
        "respondent",
        "tessara-dev-delegator",
    )
    .await?;
    let respondent_account_id = ensure_demo_account(
        pool,
        "respondent@tessara.local",
        "Demo Respondent",
        "respondent",
        "tessara-dev-respondent",
    )
    .await?;
    let delegate_account_id = ensure_demo_account(
        pool,
        "delegate@tessara.local",
        "Demo Delegate",
        "respondent",
        "tessara-dev-delegate",
    )
    .await?;

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
                key: "source_code",
                label: "Source Code",
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
                key: "source_code",
                label: "Source Code",
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
                key: "source_code",
                label: "Source Code",
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
                key: "source_code",
                label: "Source Code",
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
                ("source_code", json!("partner-1001")),
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
                ("source_code", json!("partner-1002")),
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
                ("source_code", json!("program-2001")),
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
                ("source_code", json!("program-2002")),
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
                ("source_code", json!("program-2003")),
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
                ("source_code", json!("program-2004")),
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
                ("source_code", json!("activity-3001")),
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
                ("source_code", json!("activity-3002")),
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
                ("source_code", json!("activity-3003")),
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
                ("source_code", json!("activity-3004")),
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
                ("source_code", json!("activity-3005")),
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
                ("source_code", json!("activity-3006")),
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
                ("source_code", json!("session-4001")),
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
                ("source_code", json!("session-4002")),
                ("session_date", json!("2026-05-06")),
                ("capacity", json!(20)),
                ("cancelled", json!(false)),
                ("topics", json!(["intake", "follow_up"])),
                ("room_label", json!("Room B")),
            ],
        },
    )
    .await?;
    let session_c = ensure_demo_node(
        pool,
        session_type_id,
        Some(activity_b),
        &session_fields,
        DemoNodeSpec {
            name: "Demo Session Spring Family Workshop",
            metadata: vec![
                ("source_code", json!("session-4003")),
                ("session_date", json!("2026-04-20")),
                ("capacity", json!(18)),
                ("cancelled", json!(false)),
                ("topics", json!(["wellness", "family_support"])),
                ("room_label", json!("Workshop Hall")),
            ],
        },
    )
    .await?;
    let session_d = ensure_demo_node(
        pool,
        session_type_id,
        Some(activity_b),
        &session_fields,
        DemoNodeSpec {
            name: "Demo Session Summer Family Workshop",
            metadata: vec![
                ("source_code", json!("session-4004")),
                ("session_date", json!("2026-06-18")),
                ("capacity", json!(16)),
                ("cancelled", json!(false)),
                ("topics", json!(["nutrition", "family_support"])),
                ("room_label", json!("Workshop Hall")),
            ],
        },
    )
    .await?;
    let session_e = ensure_demo_node(
        pool,
        session_type_id,
        Some(activity_c),
        &session_fields,
        DemoNodeSpec {
            name: "Demo Session Mentor Kickoff",
            metadata: vec![
                ("source_code", json!("session-4005")),
                ("session_date", json!("2026-05-12")),
                ("capacity", json!(30)),
                ("cancelled", json!(false)),
                ("topics", json!(["mentoring", "onboarding"])),
                ("room_label", json!("Studio 2")),
            ],
        },
    )
    .await?;
    let session_f = ensure_demo_node(
        pool,
        session_type_id,
        Some(activity_d),
        &session_fields,
        DemoNodeSpec {
            name: "Demo Session Mentor Follow-up",
            metadata: vec![
                ("source_code", json!("session-4006")),
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
                ("source_code", json!("session-4007")),
                ("session_date", json!("2026-06-10")),
                ("capacity", json!(15)),
                ("cancelled", json!(false)),
                ("topics", json!(["resume", "job_search"])),
                ("room_label", json!("Career Center")),
            ],
        },
    )
    .await?;
    let session_h = ensure_demo_node(
        pool,
        session_type_id,
        Some(activity_f),
        &session_fields,
        DemoNodeSpec {
            name: "Demo Session Benefits Intake",
            metadata: vec![
                ("source_code", json!("session-4008")),
                ("session_date", json!("2026-06-22")),
                ("capacity", json!(10)),
                ("cancelled", json!(false)),
                ("topics", Value::Null),
                ("room_label", Value::Null),
            ],
        },
    )
    .await?;

    ensure_account_scope_assignment(pool, operator_account_id, program_a).await?;
    ensure_account_scope_assignment(pool, operator_account_id, activity_e).await?;
    ensure_account_delegation(pool, delegator_account_id, delegate_account_id).await?;

    let partner_form = ensure_demo_form(
        pool,
        DemoFormSpec {
            name: "Demo Partner Profile",
            slug: "demo-partner-profile",
            scope_node_type_id: partner_type_id,
            compatibility_group_name: "Demo Partner Profile Compatible",
            version_label: "1.0.0",
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
            version_label: "1.0.0",
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
            version_label: "1.0.0",
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
    let intake_activity_form = ensure_demo_form(
        pool,
        DemoFormSpec {
            name: "Demo Intake Activity Checkpoint",
            slug: "demo-intake-activity-checkpoint",
            scope_node_type_id: activity_type_id,
            compatibility_group_name: "Demo Intake Activity Checkpoint Compatible",
            version_label: "1.0.0",
            section_title: "Intake Activity Checkpoint",
            fields: vec![
                FormFieldDef {
                    key: "checkpoint_notes",
                    label: "Checkpoint Notes",
                    field_type: "text",
                    required: true,
                    position: 1,
                },
                FormFieldDef {
                    key: "orientation_complete",
                    label: "Orientation Complete",
                    field_type: "boolean",
                    required: true,
                    position: 2,
                },
                FormFieldDef {
                    key: "families_ready",
                    label: "Families Ready",
                    field_type: "number",
                    required: true,
                    position: 3,
                },
            ],
        },
    )
    .await?;
    let workshop_activity_form = ensure_demo_form(
        pool,
        DemoFormSpec {
            name: "Demo Workshop Activity Checkpoint",
            slug: "demo-workshop-activity-checkpoint",
            scope_node_type_id: activity_type_id,
            compatibility_group_name: "Demo Workshop Activity Checkpoint Compatible",
            version_label: "1.0.0",
            section_title: "Workshop Activity Checkpoint",
            fields: vec![
                FormFieldDef {
                    key: "workshop_notes",
                    label: "Workshop Notes",
                    field_type: "text",
                    required: true,
                    position: 1,
                },
                FormFieldDef {
                    key: "materials_ready",
                    label: "Materials Ready",
                    field_type: "boolean",
                    required: true,
                    position: 2,
                },
                FormFieldDef {
                    key: "expected_families",
                    label: "Expected Families",
                    field_type: "number",
                    required: true,
                    position: 3,
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
            version_label: "1.0.0",
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

    replace_form_scope_nodes(pool, partner_form.form_id, &[partner_a, partner_b]).await?;
    replace_form_scope_nodes(
        pool,
        program_form.form_id,
        &[program_a, program_b, program_c, program_d],
    )
    .await?;
    replace_form_scope_nodes(
        pool,
        activity_form.form_id,
        &[
            activity_a, activity_b, activity_c, activity_d, activity_e, activity_f,
        ],
    )
    .await?;
    replace_form_scope_nodes(
        pool,
        intake_activity_form.form_id,
        &[activity_a, activity_e],
    )
    .await?;
    replace_form_scope_nodes(
        pool,
        workshop_activity_form.form_id,
        &[activity_b, activity_f],
    )
    .await?;
    replace_form_scope_nodes(
        pool,
        session_form.form_id,
        &[
            session_a, session_b, session_c, session_d, session_e, session_f, session_g, session_h,
        ],
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
        delegator_account_id,
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
        operator_account_id,
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
        delegator_account_id,
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
        operator_account_id,
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
        operator_account_id,
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
        delegate_account_id,
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
        respondent_account_id,
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
        delegate_account_id,
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
        respondent_account_id,
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
    ensure_component_testing_session_responses(
        pool,
        session_form.form_version_id,
        &[
            (session_a, respondent_account_id),
            (session_b, delegate_account_id),
            (session_g, operator_account_id),
        ],
    )
    .await?;

    ensure_single_form_workflow_assignment(
        pool,
        program_form.form_version_id,
        program_d,
        respondent_account_id,
    )
    .await?;
    ensure_single_form_workflow_assignment(
        pool,
        activity_form.form_version_id,
        activity_d,
        delegate_account_id,
    )
    .await?;
    ensure_single_form_workflow_assignment(
        pool,
        activity_form.form_version_id,
        activity_b,
        respondent_account_id,
    )
    .await?;
    ensure_single_form_workflow_assignment(
        pool,
        activity_form.form_version_id,
        activity_f,
        respondent_account_id,
    )
    .await?;
    ensure_single_form_workflow_assignment(
        pool,
        intake_activity_form.form_version_id,
        activity_a,
        respondent_account_id,
    )
    .await?;
    ensure_single_form_workflow_assignment(
        pool,
        workshop_activity_form.form_version_id,
        activity_b,
        respondent_account_id,
    )
    .await?;

    let (program_workflow_id, program_workflow_version_id, program_workflow_assignment_id) =
        ensure_program_checkpoint_workflow(
            pool,
            program_type_id,
            program_a,
            respondent_account_id,
            &[
                WorkflowStepSeed {
                    form_version_id: program_form.form_version_id,
                    title: "Program Snapshot",
                    position: 0,
                },
                WorkflowStepSeed {
                    form_version_id: intake_activity_form.form_version_id,
                    title: "Intake Activity Checkpoint",
                    position: 1,
                },
                WorkflowStepSeed {
                    form_version_id: workshop_activity_form.form_version_id,
                    title: "Workshop Activity Checkpoint",
                    position: 2,
                },
            ],
        )
        .await?;

    let analytics_status = crate::analytics::refresh_projection(pool).await?;

    let (_partner_dataset_id, partner_dataset_revision_id) = ensure_dataset(
        pool,
        partner_form.form_id,
        "Demo Partner Profile Dataset",
        "demo-partner-profile",
        "partner",
        &[partner_a, partner_b],
        &[DatasetFieldBinding {
            label: "Contact Name",
            source_field_key: "contact_name",
            field_type: "text",
        }],
    )
    .await?;
    let (_program_dataset_id, program_dataset_revision_id) = ensure_dataset(
        pool,
        program_form.form_id,
        "Demo Program Snapshot Dataset",
        "demo-program-snapshot",
        "program",
        &[program_a, program_b, program_c, program_d],
        &[DatasetFieldBinding {
            label: "Participant Target",
            source_field_key: "participant_target",
            field_type: "number",
        }],
    )
    .await?;
    let (_activity_dataset_id, activity_dataset_revision_id) = ensure_dataset(
        pool,
        activity_form.form_id,
        "Demo Activity Plan Dataset",
        "demo-activity-plan",
        "activity",
        &[
            activity_a, activity_b, activity_c, activity_d, activity_e, activity_f,
        ],
        &[DatasetFieldBinding {
            label: "Expected Attendees",
            source_field_key: "expected_attendees",
            field_type: "number",
        }],
    )
    .await?;
    let (session_dataset_id, session_dataset_revision_id) = ensure_dataset(
        pool,
        session_form.form_id,
        "Demo Session Log Dataset",
        "demo-session-log",
        "session",
        &[
            session_a, session_b, session_c, session_d, session_e, session_f, session_g, session_h,
        ],
        &[
            DatasetFieldBinding {
                label: "Session Date",
                source_field_key: "session_date",
                field_type: "date",
            },
            DatasetFieldBinding {
                label: "Participants",
                source_field_key: "participants",
                field_type: "number",
            },
            DatasetFieldBinding {
                label: "Completed As Planned",
                source_field_key: "completed_as_planned",
                field_type: "boolean",
            },
            DatasetFieldBinding {
                label: "Facilitator Notes",
                source_field_key: "facilitator_notes",
                field_type: "text",
            },
            DatasetFieldBinding {
                label: "Topics Covered",
                source_field_key: "topics_covered",
                field_type: "multi_choice",
            },
        ],
    )
    .await?;

    let (_partner_component_id, partner_component_version_id) = ensure_component(
        pool,
        "Demo Partner Profile Table",
        "demo-partner-profile-table",
        partner_dataset_revision_id,
    )
    .await?;
    let (_program_component_id, program_component_version_id) = ensure_component(
        pool,
        "Demo Program Snapshot Table",
        "demo-program-snapshot-table",
        program_dataset_revision_id,
    )
    .await?;
    let (_activity_component_id, activity_component_version_id) = ensure_component(
        pool,
        "Demo Activity Plan Table",
        "demo-activity-plan-table",
        activity_dataset_revision_id,
    )
    .await?;
    let (session_component_id, session_component_version_id) = ensure_component(
        pool,
        "Demo Session Log Table",
        "demo-session-log-table",
        session_dataset_revision_id,
    )
    .await?;
    let (_session_bar_component_id, session_bar_component_version_id) =
        ensure_component_with_config(
            pool,
            "Demo Session Participants Bar",
            "demo-session-log-bar",
            session_dataset_revision_id,
            "bar",
            json!({
                "mode": "comparison",
                "summary_field": "session__participants",
                "summary_type": "sum",
                "category_field": "session__session_date",
                "comparison_field": "session__completed_as_planned",
                "comparison_layout": "stacked",
                "orientation": "horizontal",
                "sort_field": "summary_value",
                "sort_direction": "desc",
                "number_of_points": 20,
                "value_format": "integer",
                "x_axis_label": "Participants",
                "y_axis_label": "Session Date",
                "legend_title": "Completion Status",
                "category_labels": {
                    "true": "Completed as planned",
                    "false": "Did not complete as planned"
                },
                "category_colors": {
                    "true": "var(--semantic-primary)",
                    "false": "var(--semantic-warning)"
                }
            }),
        )
        .await?;
    let (_session_line_component_id, session_line_component_version_id) =
        ensure_component_with_config(
            pool,
            "Demo Session Participants Line",
            "demo-session-log-line",
            session_dataset_revision_id,
            "line",
            json!({
                "summary_field": "session__participants",
                "summary_type": "sum",
                "x_field": "session__session_date",
                "sort_field": "x",
                "sort_direction": "asc",
                "number_of_points": 20,
                "value_format": "integer"
            }),
        )
        .await?;
    let (_session_pie_component_id, session_pie_component_version_id) =
        ensure_component_with_config(
            pool,
            "Demo Session Completion Pie",
            "demo-session-completion-pie",
            session_dataset_revision_id,
            "pie",
            json!({
                "summary_field": "session__participants",
                "summary_type": "sum",
                "category_field": "session__completed_as_planned",
                "sort_field": "summary_value",
                "sort_direction": "desc",
                "max_slices": 10,
                "value_format": "integer",
                "legend_title": "Completion Status",
                "category_labels": {
                    "true": "Completed as planned",
                    "false": "Did not complete as planned"
                },
                "category_colors": {
                    "true": "var(--semantic-primary)",
                    "false": "var(--semantic-warning)"
                }
            }),
        )
        .await?;
    let (_session_donut_component_id, session_donut_component_version_id) =
        ensure_component_with_config(
            pool,
            "Demo Session Completion Donut",
            "demo-session-completion-donut",
            session_dataset_revision_id,
            "donut",
            json!({
                "summary_field": "session__participants",
                "summary_type": "sum",
                "category_field": "session__completed_as_planned",
                "sort_field": "summary_value",
                "sort_direction": "desc",
                "max_slices": 10,
                "value_format": "integer",
                "legend_title": "Completion Status",
                "category_labels": {
                    "true": "Completed as planned",
                    "false": "Did not complete as planned"
                },
                "category_colors": {
                    "true": "var(--semantic-primary)",
                    "false": "var(--semantic-warning)"
                }
            }),
        )
        .await?;
    let (_session_stat_component_id, session_stat_component_version_id) =
        ensure_component_with_config(
            pool,
            "Demo Session Total Participants StatCard",
            "demo-session-total-participants-stat-card",
            session_dataset_revision_id,
            "stat_card",
            json!({
                "summary_field": "session__participants",
                "summary_type": "sum",
                "label": "Total participants",
                "supporting_text": "Submitted Demo Session Log entries",
                "panel_style": "accent",
                "value_format": "integer"
            }),
        )
        .await?;

    let dashboard_id = ensure_dashboard(
        pool,
        "Demo Operations Dashboard",
        Some("Operational view of partner, program, activity, and session data."),
        &[
            partner_a, partner_b, program_a, program_b, program_c, program_d, activity_a,
            activity_b, activity_c, activity_d, activity_e, activity_f, session_a, session_b,
            session_c, session_d, session_e, session_f, session_g, session_h,
        ],
    )
    .await?;
    replace_dashboard_components(
        pool,
        dashboard_id,
        &[
            DashboardComponentPlacementSeed::new(
                partner_component_version_id,
                Some("Partner Profile"),
                GridRect::new(1, 1, 6, 4),
            ),
            DashboardComponentPlacementSeed::new(
                program_component_version_id,
                Some("Program Snapshot"),
                GridRect::new(1, 7, 6, 4),
            ),
            DashboardComponentPlacementSeed::new(
                activity_component_version_id,
                Some("Activity Plan"),
                GridRect::new(5, 1, 6, 4),
            ),
            DashboardComponentPlacementSeed::new(
                session_component_version_id,
                Some("Session Log Table"),
                GridRect::new(9, 1, 12, 6),
            ),
            DashboardComponentPlacementSeed::new(
                session_bar_component_version_id,
                Some("Participants by Completion"),
                GridRect::new(15, 1, 6, 3),
            ),
            DashboardComponentPlacementSeed::new(
                session_line_component_version_id,
                Some("Participants Over Time"),
                GridRect::new(15, 7, 6, 2),
            ),
            DashboardComponentPlacementSeed::new(
                session_pie_component_version_id,
                Some("Completion Share"),
                GridRect::new(18, 1, 3, 3),
            ),
            DashboardComponentPlacementSeed::new(
                session_donut_component_version_id,
                Some("Completion Donut"),
                GridRect::new(18, 4, 3, 3),
            ),
            DashboardComponentPlacementSeed::new(
                session_stat_component_version_id,
                Some("Total Participants"),
                GridRect::new(18, 7, 3, 2),
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
        form_count: 6,
        draft_submission_count: 4,
        submitted_submission_count: 58,
        dataset_count: 4,
        dataset_revision_count: 4,
        component_count: 9,
        dashboard_count: 1,
        organization_node_id: session_a,
        form_id: session_form.form_id,
        form_version_id: session_form.form_version_id,
        submission_id: session_submitted_a,
        dataset_id: session_dataset_id,
        dataset_revision_id: session_dataset_revision_id,
        component_id: session_component_id,
        component_version_id: session_component_version_id,
        dashboard_id,
        partner_node_id: partner_a,
        program_node_id: program_a,
        activity_node_id: activity_a,
        session_node_id: session_a,
        partner_form_id: partner_form.form_id,
        program_form_id: program_form.form_id,
        activity_form_id: activity_form.form_id,
        intake_activity_form_id: intake_activity_form.form_id,
        workshop_activity_form_id: workshop_activity_form.form_id,
        session_form_id: session_form.form_id,
        partner_form_version_id: partner_form.form_version_id,
        program_form_version_id: program_form.form_version_id,
        activity_form_version_id: activity_form.form_version_id,
        intake_activity_form_version_id: intake_activity_form.form_version_id,
        workshop_activity_form_version_id: workshop_activity_form.form_version_id,
        session_form_version_id: session_form.form_version_id,
        program_workflow_id,
        program_workflow_version_id,
        program_workflow_assignment_id,
        analytics_values: analytics_status.value_count,
    })
}

async fn require_demo_seed_target_empty(
    pool: &PgPool,
    dev_admin_account_id: Uuid,
) -> ApiResult<()> {
    let existing_rows: i64 = sqlx::query_scalar(
        r#"
        SELECT
            (SELECT COUNT(*) FROM accounts WHERE id <> $1)
          + (SELECT COUNT(*) FROM node_types)
          + (SELECT COUNT(*) FROM nodes)
          + (SELECT COUNT(*) FROM forms)
          + (SELECT COUNT(*) FROM form_versions)
          + (SELECT COUNT(*) FROM submissions)
          + (SELECT COUNT(*) FROM workflows)
          + (SELECT COUNT(*) FROM workflow_versions)
          + (SELECT COUNT(*) FROM datasets)
          + (SELECT COUNT(*) FROM dataset_revisions)
          + (SELECT COUNT(*) FROM components)
          + (SELECT COUNT(*) FROM component_versions)
          + (SELECT COUNT(*) FROM dashboards)
        "#,
    )
    .bind(dev_admin_account_id)
    .fetch_one(pool)
    .await?;

    if existing_rows == 0 {
        return Ok(());
    }

    Err(ApiError::BadRequest(
        "Demo seed requires an empty database. Recreate the local database or run local launch with -FreshData before seeding.".into(),
    ))
}

async fn ensure_component_testing_session_responses(
    pool: &PgPool,
    form_version_id: Uuid,
    session_accounts: &[(Uuid, Uuid)],
) -> ApiResult<()> {
    const TOPIC_SETS: &[&[&str]] = &[
        &["intake", "welcome"],
        &["attendance", "check_in"],
        &["family_support", "wellness"],
        &["resume", "job_search"],
        &["mentoring", "onboarding"],
        &["nutrition", "follow_up"],
    ];
    const NOTE_PATTERNS: &[&str] = &[
        "Session stayed on pace with steady participation.",
        "Facilitator adjusted pacing after participant questions.",
        "Follow-up materials were requested by several attendees.",
        "Attendance was lower than planned but discussion quality was high.",
        "Participants completed the core activities before wrap-up.",
    ];

    for index in 0..50 {
        let (node_id, account_id) = session_accounts[index % session_accounts.len()];
        let topic_set = TOPIC_SETS[index % TOPIC_SETS.len()];
        let participant_count = 8 + ((index * 7) % 31) as i64;
        let completed_as_planned = index % 7 != 3;
        let month = 4 + (index / 17);
        let day = 1 + ((index * 3) % 27);
        let session_date = format!("2026-{month:02}-{day:02}");
        let seed_key = format!("seed_demo:session-component-test-{index:02}");
        let facilitator_notes = format!(
            "{} Batch response {}.",
            NOTE_PATTERNS[index % NOTE_PATTERNS.len()],
            index + 1
        );
        let topics = Value::Array(
            topic_set
                .iter()
                .map(|topic| Value::String((*topic).to_string()))
                .collect(),
        );

        ensure_seed_submission(
            pool,
            account_id,
            form_version_id,
            node_id,
            SeedSubmissionSpec {
                seed_key: &seed_key,
                status: "submitted",
                values: vec![
                    ("session_date", json!(session_date)),
                    ("participants", json!(participant_count)),
                    ("completed_as_planned", json!(completed_as_planned)),
                    ("facilitator_notes", json!(facilitator_notes)),
                    ("topics_covered", topics),
                ],
            },
        )
        .await?;
    }

    Ok(())
}
