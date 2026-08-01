//! Native Core application-composition administration surface.

use leptos::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json::Value;
#[cfg(feature = "hydrate")]
use serde_json::json;

use crate::ui::{AppShell, PageHeader};

#[derive(Clone, Debug, Deserialize, Serialize)]
struct CompositionSummaryV1 {
    schema_version: u16,
    installation_id: String,
    latest_blueprint: Option<Value>,
    latest_lockfile: Option<Value>,
    latest_approval: Option<Value>,
    active_operation: Option<Value>,
    latest_receipt: Option<Value>,
    drift_findings: Vec<Value>,
    emergency_overrides: Vec<Value>,
}

#[component]
pub fn ApplicationCompositionPage() -> impl IntoView {
    let summary = RwSignal::new(None::<CompositionSummaryV1>);
    let error = RwSignal::new(None::<String>);
    let busy = RwSignal::new(false);
    let blueprint_json = RwSignal::new(String::new());
    let catalog_json = RwSignal::new(String::new());
    let emergency_module = RwSignal::new(String::new());
    let emergency_reason = RwSignal::new(String::new());

    #[cfg(feature = "hydrate")]
    Effect::new(move |_| {
        leptos::task::spawn_local(async move {
            reload(summary, error).await;
        });
    });

    let create_draft = move |_| {
        #[cfg(feature = "hydrate")]
        leptos::task::spawn_local(async move {
            busy.set(true);
            error.set(None);
            let result = async {
                let document: Value = serde_json::from_str(&blueprint_json.get_untracked())
                    .map_err(|error| format!("Blueprint JSON is invalid: {error}"))?;
                tessara_web_http::send_json::<Value, _>(
                    gloo_net::http::Request::post("/api/admin/composition/blueprints"),
                    &document,
                    "Create Blueprint draft",
                )
                .await
                .map_err(tessara_web_http::RequestError::into_message)?;
                Ok::<(), String>(())
            }
            .await;
            if let Err(message) = result {
                error.set(Some(message));
            } else {
                reload(summary, error).await;
            }
            busy.set(false);
        });
    };

    let resolve_latest = move |_| {
        #[cfg(feature = "hydrate")]
        leptos::task::spawn_local(async move {
            busy.set(true);
            error.set(None);
            let result = async {
                let revision = latest_revision(summary.get_untracked().as_ref())?;
                let catalog: Value = serde_json::from_str(&catalog_json.get_untracked())
                    .map_err(|error| format!("Catalog JSON is invalid: {error}"))?;
                tessara_web_http::send_json::<Value, _>(
                    gloo_net::http::Request::post(&format!(
                        "/api/admin/composition/blueprints/{revision}/resolve"
                    )),
                    &json!({ "catalog": catalog }),
                    "Resolve Blueprint",
                )
                .await
                .map_err(tessara_web_http::RequestError::into_message)?;
                Ok::<(), String>(())
            }
            .await;
            if let Err(message) = result {
                error.set(Some(message));
            } else {
                reload(summary, error).await;
            }
            busy.set(false);
        });
    };

    let approve_latest = move |_| {
        #[cfg(feature = "hydrate")]
        leptos::task::spawn_local(async move {
            busy.set(true);
            error.set(None);
            let result = async {
                let revision = latest_revision(summary.get_untracked().as_ref())?;
                let approved_effects = latest_required_effects(summary.get_untracked().as_ref())?;
                tessara_web_http::send_json::<Value, _>(
                    gloo_net::http::Request::post(&format!(
                        "/api/admin/composition/blueprints/{revision}/approve"
                    )),
                    &json!({
                        "approved_effects": approved_effects,
                        "reason": "Approved through Application Composition"
                    }),
                    "Approve composition plan",
                )
                .await
                .map_err(tessara_web_http::RequestError::into_message)?;
                Ok::<(), String>(())
            }
            .await;
            if let Err(message) = result {
                error.set(Some(message));
            } else {
                reload(summary, error).await;
            }
            busy.set(false);
        });
    };

    let apply_latest = move |_| {
        #[cfg(feature = "hydrate")]
        leptos::task::spawn_local(async move {
            busy.set(true);
            error.set(None);
            let result = async {
                let revision = latest_revision(summary.get_untracked().as_ref())?;
                tessara_web_http::send_json::<Value, _>(
                    gloo_net::http::Request::post(&format!(
                        "/api/admin/composition/blueprints/{revision}/apply"
                    )),
                    &json!({}),
                    "Apply composition",
                )
                .await
                .map_err(tessara_web_http::RequestError::into_message)?;
                Ok::<(), String>(())
            }
            .await;
            if let Err(message) = result {
                error.set(Some(message));
            } else {
                reload(summary, error).await;
            }
            busy.set(false);
        });
    };

    let resolve_drift = move |finding_id: String, disposition: &'static str| {
        #[cfg(not(feature = "hydrate"))]
        let _ = (&finding_id, disposition);
        #[cfg(feature = "hydrate")]
        leptos::task::spawn_local(async move {
            busy.set(true);
            error.set(None);
            let result = tessara_web_http::send_json_without_response(
                gloo_net::http::Request::post(&format!(
                    "/api/admin/composition/drift/{finding_id}/{disposition}"
                )),
                &json!({}),
                if disposition == "adopt" {
                    "Adopt drift"
                } else {
                    "Reconcile drift"
                },
            )
            .await;
            if let Err(request_error) = result {
                error.set(Some(request_error.into_message()));
            } else {
                reload(summary, error).await;
            }
            busy.set(false);
        });
    };

    let emergency_disable = move |_| {
        #[cfg(feature = "hydrate")]
        leptos::task::spawn_local(async move {
            busy.set(true);
            error.set(None);
            let definition_id = emergency_module.get_untracked();
            let reason = emergency_reason.get_untracked();
            let result = tessara_web_http::send_json::<Value, _>(
                gloo_net::http::Request::post(&format!(
                    "/api/admin/composition/modules/{definition_id}/emergency-disable"
                )),
                &json!({ "reason": reason, "expires_in_minutes": 60 }),
                "Emergency disable",
            )
            .await;
            if let Err(request_error) = result {
                error.set(Some(request_error.into_message()));
            } else {
                emergency_reason.set(String::new());
                reload(summary, error).await;
            }
            busy.set(false);
        });
    };

    view! {
        <AppShell active_route="application_composition" title="Application Composition">
            <section class="route-panel composition-page">
                <PageHeader title="Application Composition" />
                <p class="route-panel__description">
                    "Declare desired Core and Module releases, resolve an immutable plan, approve its effects, and compare the Supervisor receipt with desired state."
                </p>
                {move || error.get().map(|message| view! {
                    <section class="organization-state is-error" role="alert"><h3>"Composition action failed"</h3><p>{message}</p></section>
                })}
                <div class="summary-grid">
                    <CompositionStatusCard title="Blueprint" value=move || projection_label(summary.get().and_then(|state| state.latest_blueprint), "No draft") />
                    <CompositionStatusCard title="Resolved plan" value=move || projection_label(summary.get().and_then(|state| state.latest_lockfile), "Not resolved") />
                    <CompositionStatusCard title="Approval" value=move || projection_label(summary.get().and_then(|state| state.latest_approval), "Awaiting approval") />
                    <CompositionStatusCard title="Observed receipt" value=move || projection_label(summary.get().and_then(|state| state.latest_receipt), "Not materialized") />
                </div>
                <section class="panel-card">
                    <h2>"1. Create a Blueprint draft"</h2>
                    <p>"Paste strict application-blueprint/v1 JSON. Each accepted edit creates the next immutable revision."</p>
                    <textarea rows="12" prop:value=move || blueprint_json.get() on:input=move |event| blueprint_json.set(event_target_value(&event)) />
                    <button class="button" disabled=move || busy.get() on:click=create_draft>"Create draft"</button>
                </section>
                <section class="panel-card">
                    <h2>"2. Resolve against a verified catalog"</h2>
                    <p>"Paste the payload from a Supervisor-verified release-catalog/v1 envelope."</p>
                    <textarea rows="10" prop:value=move || catalog_json.get() on:input=move |event| catalog_json.set(event_target_value(&event)) />
                    <button class="button" disabled=move || busy.get() on:click=resolve_latest>"Resolve latest draft"</button>
                </section>
                <section class="panel-card">
                    <h2>"3. Approve explicit effects"</h2>
                    <p>"Approval is separate from planning and binds the current lockfile and materialization plan. Destructive data removal is excluded."</p>
                    <button class="button" disabled=move || busy.get() on:click=approve_latest>"Approve current plan"</button>
                </section>
                <section class="panel-card">
                    <h2>"4. Apply through Supervisor"</h2>
                    <p>"A short-lived signed authorization is minted only from the exact persisted approval. Supervisor remains the sole materialization authority."</p>
                    <button class="button" disabled=move || busy.get() on:click=apply_latest>"Apply approved plan"</button>
                </section>
                <section class="panel-card">
                    <h2>"Drift"</h2>
                    <p>{move || summary.get().map_or("Loading composition state…".into(), |state| format!("{} open finding(s)", state.drift_findings.len()))}</p>
                    <div class="composition-drift-list">
                        {move || summary.get().map(|state| state.drift_findings.into_iter().map(|finding| {
                            let finding_id = finding.get("finding_id").and_then(Value::as_str).unwrap_or_default().to_string();
                            let path = finding.get("path").and_then(Value::as_str).unwrap_or("unknown path").to_string();
                            let desired = finding.get("desired").cloned().unwrap_or(Value::Null);
                            let observed = finding.get("observed").cloned().unwrap_or(Value::Null);
                            let adopt_id = finding_id.clone();
                            let reconcile_id = finding_id.clone();
                            view! {
                                <article class="summary-card">
                                    <strong>{path}</strong>
                                    <p>"Desired: "<code>{desired.to_string()}</code></p>
                                    <p>"Observed: "<code>{observed.to_string()}</code></p>
                                    <div class="button-row">
                                        <button class="button button--secondary" disabled=move || busy.get() on:click=move |_| resolve_drift(adopt_id.clone(), "adopt")>"Adopt as new draft"</button>
                                        <button class="button" disabled=move || busy.get() on:click=move |_| resolve_drift(reconcile_id.clone(), "reconcile")>"Restore desired"</button>
                                    </div>
                                </article>
                            }
                        }).collect_view())}
                    </div>
                </section>
                <section class="panel-card">
                    <h2>"Emergency module disable"</h2>
                    <p>"Apply a signed, single-module disable override with a required reason and a one-hour expiry. The approved Blueprint is unchanged."</p>
                    <label>"Module definition ID"<input prop:value=move || emergency_module.get() on:input=move |event| emergency_module.set(event_target_value(&event)) /></label>
                    <label>"Reason"<input prop:value=move || emergency_reason.get() on:input=move |event| emergency_reason.set(event_target_value(&event)) /></label>
                    <button class="button" disabled=move || busy.get() || emergency_module.get().trim().is_empty() || emergency_reason.get().trim().is_empty() on:click=emergency_disable>"Emergency disable for 1 hour"</button>
                    <div class="composition-drift-list">
                        {move || summary.get().map(|state| state.emergency_overrides.into_iter().map(|override_record| {
                            let definition = override_record.get("definition_id").and_then(Value::as_str).unwrap_or("unknown module").to_string();
                            let reason = override_record.get("reason").and_then(Value::as_str).unwrap_or("No reason recorded").to_string();
                            let expiry = override_record.get("expires_at").and_then(Value::as_str).unwrap_or("no expiry").to_string();
                            let status = if override_record.get("reconciled_at").is_some_and(|value| !value.is_null()) { "Reconciled" } else if override_record.get("expired").and_then(Value::as_bool) == Some(true) { "Expired; restore or adopt still required" } else { "Active" };
                            view! { <article class="summary-card"><strong>{definition}</strong><p>{reason}</p><small>{format!("{status} · Expires {expiry}")}</small></article> }
                        }).collect_view())}
                    </div>
                </section>
            </section>
        </AppShell>
    }
}

#[component]
fn CompositionStatusCard<F>(title: &'static str, value: F) -> impl IntoView
where
    F: Fn() -> String + Send + Sync + 'static,
{
    view! { <article class="summary-card"><span>{title}</span><strong>{value}</strong></article> }
}

fn projection_label(value: Option<Value>, empty: &str) -> String {
    value
        .and_then(|value| {
            value
                .get("revision")
                .or_else(|| value.get("blueprint_revision"))
                .map(|revision| format!("Revision {revision}"))
        })
        .unwrap_or_else(|| empty.into())
}

#[cfg(feature = "hydrate")]
fn latest_revision(summary: Option<&CompositionSummaryV1>) -> Result<u64, String> {
    summary
        .and_then(|summary| summary.latest_blueprint.as_ref())
        .and_then(|blueprint| blueprint.get("revision"))
        .and_then(Value::as_u64)
        .ok_or_else(|| "Create a Blueprint draft first.".into())
}

#[cfg(feature = "hydrate")]
fn latest_required_effects(summary: Option<&CompositionSummaryV1>) -> Result<Vec<String>, String> {
    let actions = summary
        .and_then(|summary| summary.latest_lockfile.as_ref())
        .and_then(|lockfile| lockfile.pointer("/materialization_plan/actions"))
        .and_then(Value::as_array)
        .ok_or_else(|| "Resolve the Blueprint before approval.".to_string())?;
    let mut effects = std::collections::BTreeSet::new();
    for action in actions {
        match action.get("action").and_then(Value::as_str) {
            Some("acquire_image" | "provision_database" | "migrate") => {
                effects.insert("install".to_string());
            }
            Some("configure") => {
                effects.insert("configure".to_string());
            }
            Some("bootstrap") => {
                effects.insert("bootstrap".to_string());
            }
            Some("switch_traffic") => {
                effects.insert("upgrade".to_string());
            }
            Some("set_enablement")
                if action.get("enabled").and_then(Value::as_bool) == Some(true) =>
            {
                effects.insert("enable".to_string());
            }
            Some("set_enablement") => {
                effects.insert("disable".to_string());
            }
            _ => {}
        }
    }
    Ok(effects.into_iter().collect())
}

#[cfg(feature = "hydrate")]
async fn reload(summary: RwSignal<Option<CompositionSummaryV1>>, error: RwSignal<Option<String>>) {
    match tessara_web_http::fetch_json("/api/admin/composition", "Application composition").await {
        Ok(value) => summary.set(Some(value)),
        Err(request_error) => error.set(Some(request_error.into_message())),
    }
}
