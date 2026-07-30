use icons::{Copy, Download};
use leptos::prelude::*;
use tessara_module_contract::{DeploymentOperationV1, IdentityChangeV1, ReleaseChangeV1};

use super::{directory::compact_value, models::DeploymentReceiptV1};

#[component]
pub(super) fn DeploymentLedger(
    receipt: Option<DeploymentReceiptV1>,
    #[prop(default = Vec::new())] history: Vec<DeploymentReceiptV1>,
) -> impl IntoView {
    let Some(receipt) = receipt else {
        return view! {
            <section class="organization-detail-card module-deployment-empty" aria-live="polite">
                <h2>"Current deployment"</h2>
                <p>"No applied deployment receipt has been recorded for this installation."</p>
            </section>
        }
        .into_any();
    };

    let plan_digest = receipt.plan_digest.to_string();
    let all_healthy = receipt.components.iter().all(|component| component.healthy);
    let previous_receipt = receipt.previous_revision.and_then(|revision| {
        history
            .iter()
            .find(|candidate| candidate.revision == revision)
    });
    let change = receipt.classify_change(previous_receipt);
    let operation = operation_label(receipt.revision, change.operation);
    let compatibility_label = match change.operation {
        DeploymentOperationV1::RolledBack { .. } => "Compatible rollback",
        DeploymentOperationV1::Applied => "Compatible change",
        DeploymentOperationV1::Installed => "Compatible installation",
    };
    let rollback_target = receipt
        .rollback_target_revision
        .map(|revision| format!("Revision {revision}"))
        .or_else(|| {
            receipt
                .previous_revision
                .map(|revision| format!("Revision {revision} retained and compatible"))
        })
        .unwrap_or_else(|| "Initial deployment".into());
    let current_modules = receipt.modules.clone();
    let curated_modules = receipt.modules.clone();
    let change_rows = change.modules;

    view! {
        <div class="module-deployment-ledger">
            <header class="module-deployment-heading">
                <div>
                    <h2>"Current deployment"</h2>
                    <p>"Read-only state from the latest sanitized deployment receipt."</p>
                </div>
            </header>
            <section class="module-runtime-strip module-deployment-summary" aria-label="Current deployment receipt">
                <div class="module-runtime-strip__item"><div><span>"Revision"</span><strong>{receipt.revision}</strong></div></div>
                <div class="module-runtime-strip__item"><div><span>"Plan digest"</span><span class="module-digest-inline"><code>{compact_value(&plan_digest)}</code><CopyDigest value=plan_digest/></span></div></div>
                <div class="module-runtime-strip__item"><div><span>"Applied"</span><LocalTime value=receipt.applied_at.clone()/></div></div>
                <div class="module-runtime-strip__item"><div><span>"Operator"</span><strong>{receipt.operator.clone()}</strong></div></div>
            </section>
            <div class="module-deployment-grid">
                <div class="module-deployment-main">
                    <section class="organization-detail-card module-detail-overview-card">
                        <header class="module-detail__heading"><div><h2>"Resolved components"</h2><p>"Exact artifacts and observed runtime health from the current receipt."</p></div><a
                            class="button button--secondary"
                            href=format!("/api/admin/deployment-receipts/{}", receipt.revision)
                            download=format!("deployment-receipt-{}.json", receipt.revision)
                        ><Download class="button__icon"/>"Download receipt"</a></header>
                        <div class="table-wrap"><table class="data-table"><thead><tr><th>"Component"</th><th>"Artifact"</th><th>"Runtime"</th><th>"Health"</th></tr></thead><tbody>
                        {receipt.components.into_iter().map(|component| {
                            let artifact = component.artifact.to_string();
                            let badge_class = if component.healthy { "status-badge is-success" } else { "status-badge is-danger" };
                            let health = if component.healthy { "Healthy" } else { "Unhealthy" };
                            let tooltip = if component.healthy { "The component passed its current health check." } else { "The component failed its current health check." };
                            view! { <tr><th scope="row">{component.name}</th><td><span class="module-digest-inline"><code>{compact_value(&artifact)}</code><CopyDigest value=artifact/></span></td><td>{component.runtime}</td><td><span class=badge_class title=tooltip>{health}</span></td></tr> }
                        }).collect_view()}
                        </tbody></table></div>
                    </section>
                    <section class="organization-detail-card module-detail-overview-card">
                        <header class="module-detail__heading"><div><h2>"Applied change"</h2><p>{operation.clone()}</p></div><span class="status-badge is-info" title="The applied release satisfies this installation's compatibility requirements and retains a supported rollback target.">{compatibility_label}</span></header>
                        {change_rows.into_iter().map(|change| {
                            let definition = change.definition_id.to_string();
                            let release = release_change_label(&change.release);
                            let (instance_state, instance) = identity_change_label(&change.instance);
                            let (database_state, database) = change.database.as_ref().map_or_else(
                                || ("Not used", "Module-owned state".into()),
                                identity_change_label,
                            );
                            view! {
                            <dl class="module-detail-overview__list module-deployment-change">
                                <div><dt>{module_display_name(&definition)}</dt><dd>{format!("Release {release}")}</dd></div>
                                <div><dt>"Instance identity"</dt><dd><span class="module-digest-inline">{instance_state}" · "<code>{compact_value(&instance)}</code><CopyDigest value=instance/></span></dd></div>
                                <div><dt>"Database binding"</dt><dd>{database_state}" · "<code>{database}</code></dd></div>
                                <div><dt>"Migration"</dt><dd>"Not recorded in receipt"</dd></div>
                                <div><dt>"Rollback target"</dt><dd>{rollback_target.clone()}</dd></div>
                            </dl>
                        }}).collect_view()}
                    </section>
                </div>
                <aside class="module-deployment-sidebar">
                    <section class="organization-detail-card module-detail-overview-card">
                        <h2>"Receipt history"</h2>
                        <dl class="module-detail-overview__summary-list module-deployment-history">
                            {history.into_iter().map(|item| view! {
                                <div><dt>{operation_label(item.revision, item.classify_change(None).operation)}</dt><dd><LocalTime value=item.applied_at/></dd></div>
                            }).collect_view()}
                        </dl>
                    </section>
                    <section class="organization-detail-card module-detail-overview-card">
                        <h2>"Release provenance"</h2>
                        <dl class="module-detail-overview__summary-list">
                            <div><dt>"Source"</dt><dd>"Curated Tessara release"</dd></div>
                            <div><dt>"Component health"</dt><dd><span class=if all_healthy { "status-badge is-success" } else { "status-badge is-danger" } title=if all_healthy { "Every component in the current receipt is healthy." } else { "At least one component in the current receipt is unhealthy." }>{if all_healthy { "Healthy" } else { "Needs attention" }}</span></dd></div>
                            <div><dt>"Rollback target"</dt><dd>{rollback_target}</dd></div>
                        </dl>
                        <ul class="module-deployment-publishers">
                            {curated_modules.into_iter().map(|module| view! {
                                <li><strong>{module.publisher.to_string()}</strong><span class="module-digest-inline"><code>{compact_value(module.manifest_digest.as_str())}</code><CopyDigest value=module.manifest_digest.to_string()/></span></li>
                            }).collect_view()}
                        </ul>
                    </section>
                    <section class="organization-detail-card module-detail-overview-card">
                        <h2>"Module identities"</h2>
                        <dl class="module-detail-overview__summary-list">
                            {current_modules.into_iter().map(|module| view! {
                                <div><dt>{module.definition_id.to_string()}</dt><dd><span class="module-digest-inline"><code>{compact_value(&module.instance_id.to_string())}</code><CopyDigest value=module.instance_id.to_string()/></span></dd></div>
                            }).collect_view()}
                        </dl>
                    </section>
                </aside>
            </div>
        </div>
    }.into_any()
}

fn operation_label(revision: u64, operation: DeploymentOperationV1) -> String {
    match operation {
        DeploymentOperationV1::Installed => format!("Revision {revision} installed"),
        DeploymentOperationV1::Applied => format!("Revision {revision} applied"),
        DeploymentOperationV1::RolledBack { target_revision } => {
            format!("Revision {revision} rolled back to {target_revision}")
        }
    }
}

fn release_change_label(change: &ReleaseChangeV1) -> String {
    match change {
        ReleaseChangeV1::Installed { version } => format!("Installed {version}"),
        ReleaseChangeV1::Unchanged { version } => format!("{version} unchanged"),
        ReleaseChangeV1::Changed { from, to } => format!("{from} → {to}"),
    }
}

fn identity_change_label(change: &IdentityChangeV1) -> (&'static str, String) {
    match change {
        IdentityChangeV1::Created { value } => ("Created", value.clone()),
        IdentityChangeV1::Preserved { value } => ("Preserved", value.clone()),
        IdentityChangeV1::Replaced { from, to } => ("Replaced", format!("{from} → {to}")),
    }
}

fn module_display_name(definition_id: &str) -> String {
    definition_id
        .rsplit(['.', ':'])
        .next()
        .unwrap_or(definition_id)
        .split('-')
        .map(|word| {
            let mut chars = word.chars();
            chars.next().map_or_else(String::new, |first| {
                first.to_uppercase().collect::<String>() + chars.as_str()
            })
        })
        .collect::<Vec<_>>()
        .join(" ")
}

#[component]
fn LocalTime(value: String) -> impl IntoView {
    let display = RwSignal::new(value.clone());
    #[cfg(all(feature = "hydrate", target_arch = "wasm32"))]
    {
        let instant = value.clone();
        Effect::new(move |_| {
            use wasm_bindgen::JsValue;
            let date = js_sys::Date::new(&JsValue::from_str(&instant));
            let localized = date.to_locale_string("default", &JsValue::UNDEFINED);
            if let Some(localized) = localized.as_string() {
                display.set(localized);
            }
        });
    }
    view! { <time datetime=value title="Displayed in your local time zone">{move || display.get()}</time> }
}

#[component]
fn CopyDigest(value: String) -> impl IntoView {
    let value_for_copy = value.clone();
    view! { <button class="icon-button module-directory__copy" type="button" title="Copy complete value" aria-label="Copy complete value" on:click=move |_| copy(value_for_copy.clone())><Copy/></button> }
}

#[cfg(feature = "hydrate")]
fn copy(value: String) {
    if let Some(window) = web_sys::window() {
        let clipboard = window.navigator().clipboard();
        wasm_bindgen_futures::spawn_local(async move {
            let _ = wasm_bindgen_futures::JsFuture::from(clipboard.write_text(&value)).await;
        });
    }
}
#[cfg(not(feature = "hydrate"))]
fn copy(_value: String) {}
