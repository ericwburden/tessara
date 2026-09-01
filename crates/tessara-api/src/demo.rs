//! Deterministic demo data seeding for local development and smoke tests.

use axum::{Json, Router, extract::State, routing::post};
use serde::Serialize;
use serde_json::{Value, json};
use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    auth::AuthenticatedRequest,
    db::AppState,
    error::{ApiError, ApiResult},
};

mod accounts;
mod forms;
mod hierarchy;
mod workflows;

use accounts::{
    ensure_account_delegation, ensure_account_scope_assignment, ensure_demo_account,
    require_dev_admin_account,
};
use forms::{DemoFormSpec, FormFieldDef, ensure_demo_form, replace_form_scope_nodes};
use hierarchy::{
    DemoNodeSpec, MetadataFieldDef, ensure_demo_node, ensure_metadata_fields, ensure_node_type,
    ensure_node_type_relationship,
};
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
    pub organization_node_id: Uuid,
    pub form_id: Uuid,
    pub form_version_id: Uuid,
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

    Ok(DemoSeedSummary {
        seed_version: DEMO_SEED_VERSION,
        node_counts: DemoNodeCounts {
            partners: 2,
            programs: 4,
            activities: 6,
            sessions: 8,
        },
        form_count: 6,
        organization_node_id: session_a,
        form_id: session_form.form_id,
        form_version_id: session_form.form_version_id,
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
          + (SELECT COUNT(*) FROM workflows)
          + (SELECT COUNT(*) FROM workflow_versions)
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
