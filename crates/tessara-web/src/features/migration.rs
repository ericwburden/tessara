use leptos::prelude::*;

use crate::features::native_shell::{BreadcrumbItem, MetadataStrip, NativePage, PageHeader, Panel};

#[cfg(feature = "hydrate")]
use crate::features::native_runtime::{get_json, post_json};
use serde::Deserialize;
#[cfg(feature = "hydrate")]
use serde_json::json;
#[cfg(feature = "hydrate")]
use wasm_bindgen_futures::spawn_local;

#[derive(Clone, Deserialize)]
struct LegacyFixtureExample {
    name: String,
    fixture_json: String,
}

#[derive(Clone, Deserialize)]
struct LegacyImportValidationIssue {
    code: String,
    path: String,
    message: String,
}

#[derive(Clone, Deserialize)]
struct LegacyImportValidationReport {
    issue_count: usize,
    issues: Vec<LegacyImportValidationIssue>,
}

#[derive(Clone, Deserialize)]
struct LegacyImportDryRunReport {
    fixture_name: String,
    would_import: bool,
    validation: LegacyImportValidationReport,
}

#[derive(Clone, Deserialize)]
struct LegacyImportSummary {
    fixture_name: String,
    form_version_id: String,
    submission_id: String,
    dashboard_id: String,
    analytics_values: i64,
}

#[derive(Clone)]
enum MigrationResult {
    Validation(LegacyImportValidationReport),
    DryRun(LegacyImportDryRunReport),
    Import(LegacyImportSummary),
    Error(String),
}

#[component]
pub fn MigrationPage() -> impl IntoView {
    let examples = RwSignal::new(Vec::<LegacyFixtureExample>::new());
    let fixture_json = RwSignal::new(String::new());
    let result = RwSignal::new(None::<MigrationResult>);
    let loading = RwSignal::new(true);
    let busy = RwSignal::new(false);

    #[cfg(feature = "hydrate")]
    Effect::new(move |_| {
        spawn_local(async move {
            loading.set(true);
            match get_json::<Vec<LegacyFixtureExample>>("/api/admin/legacy-fixtures/examples").await
            {
                Ok(items) => examples.set(items),
                Err(message) => result.set(Some(MigrationResult::Error(message))),
            }
            loading.set(false);
        });
    });

    let run_action = move |_path: &'static str| {
        if busy.get_untracked() {
            return;
        }
        busy.set(true);
        result.set(None);

        #[cfg(feature = "hydrate")]
        {
            let payload = fixture_json.get_untracked();
            spawn_local(async move {
                let body = json!({ "fixture_json": payload });
                let outcome = match _path {
                    "/api/admin/legacy-fixtures/validate" => {
                        post_json::<LegacyImportValidationReport>(_path, &body)
                            .await
                            .map(MigrationResult::Validation)
                    }
                    "/api/admin/legacy-fixtures/dry-run" => {
                        post_json::<LegacyImportDryRunReport>(_path, &body)
                            .await
                            .map(MigrationResult::DryRun)
                    }
                    "/api/admin/legacy-fixtures/import" => {
                        post_json::<LegacyImportSummary>(_path, &body)
                            .await
                            .map(MigrationResult::Import)
                    }
                    _ => Err("unsupported migration action".into()),
                };
                match outcome {
                    Ok(payload) => result.set(Some(payload)),
                    Err(message) => result.set(Some(MigrationResult::Error(message))),
                }
                busy.set(false);
            });
        }
    };

    view! {
        <NativePage
            title="Migration"
            description="Tessara migration workbench."
            page_key="migration"
            active_route="migration"
            workspace_label="Internal Area"
            required_capability="admin:all"
            breadcrumbs=vec![
                BreadcrumbItem::link("Home", "/app"),
                BreadcrumbItem::current("Migration"),
            ]
        >
            <PageHeader
                eyebrow="Internal Area"
                title="Migration Workbench"
                description="Validate, dry-run, and import representative legacy fixtures from this native operator surface."
            />
            <MetadataStrip items=vec![
                ("Mode", "Workspace".into()),
                ("Surface", "Operator import flow".into()),
                ("State", "Native SSR shell".into()),
            ]/>
            <Panel title="Fixture Examples" description="Bundled rehearsal fixtures can be loaded into the editor below.">
                <div id="migration-list" class="record-list">
                    <Show
                        when=move || !loading.get()
                        fallback=|| view! { <p class="muted">"Loading fixture examples..."</p> }
                    >
                        {move || {
                            let items = examples.get();
                            if items.is_empty() {
                                return view! { <p class="muted">"No fixture examples available."</p> }.into_any();
                            }
                            view! {
                                {items
                                    .into_iter()
                                    .map(|fixture| {
                                        let fixture_name = fixture.name.clone();
                                        let fixture_payload = fixture.fixture_json.clone();
                                        view! {
                                            <article class="record-card">
                                                <h4>{fixture_name.clone()}</h4>
                                                <p class="muted">{format!("{} bytes", fixture_payload.len())}</p>
                                                <div class="actions">
                                                    <button
                                                        class="button-link button is-light"
                                                        type="button"
                                                        on:click=move |_| fixture_json.set(fixture_payload.clone())
                                                    >
                                                        "Use Fixture"
                                                    </button>
                                                </div>
                                            </article>
                                        }
                                    })
                                    .collect_view()}
                            }.into_any()
                        }}
                    </Show>
                </div>
            </Panel>
            <Panel title="Fixture Editor" description="Paste or refine a legacy rehearsal fixture before validating or importing it.">
                <div class="form-grid">
                    <div class="form-field wide-field">
                        <label for="migration-fixture-json">"Fixture JSON"</label>
                        <textarea
                            id="migration-fixture-json"
                            class="textarea"
                            rows="18"
                            prop:value=move || fixture_json.get()
                            on:input=move |event| fixture_json.set(event_target_value(&event))
                        ></textarea>
                    </div>
                </div>
                <div class="actions">
                    <button class="button-link button is-light" type="button" disabled=move || busy.get() on:click=move |_| run_action("/api/admin/legacy-fixtures/validate")>
                        "Validate Fixture"
                    </button>
                    <button class="button-link button is-light" type="button" disabled=move || busy.get() on:click=move |_| run_action("/api/admin/legacy-fixtures/dry-run")>
                        "Dry Run"
                    </button>
                    <button class="button-link" type="button" disabled=move || busy.get() on:click=move |_| run_action("/api/admin/legacy-fixtures/import")>
                        "Import Fixture"
                    </button>
                </div>
            </Panel>
            <Panel title="Migration Results" description="Validation findings, dry-run outcomes, and import summaries appear here.">
                <div id="migration-results" class="record-list">
                    {move || match result.get() {
                        Some(MigrationResult::Validation(report)) => {
                            if report.issue_count == 0 {
                                view! {
                                    <article class="record-card">
                                        <h4>"Validation Passed"</h4>
                                        <p>"No migration issues were reported for the current fixture."</p>
                                    </article>
                                }.into_any()
                            } else {
                                view! {
                                    <article class="record-card">
                                        <h4>"Validation Issues"</h4>
                                        <p class="muted">{format!("{} issues found", report.issue_count)}</p>
                                        <ul class="app-list">
                                            {report
                                                .issues
                                                .into_iter()
                                                .map(|issue| view! {
                                                    <li>{format!("{} at {}: {}", issue.code, issue.path, issue.message)}</li>
                                                })
                                                .collect_view()}
                                        </ul>
                                    </article>
                                }.into_any()
                            }
                        }
                        Some(MigrationResult::DryRun(report)) => view! {
                            <article class="record-card">
                                <h4>{report.fixture_name}</h4>
                                <p>{format!("Would import: {}", if report.would_import { "yes" } else { "no" })}</p>
                                <p class="muted">{format!("Validation issues: {}", report.validation.issue_count)}</p>
                            </article>
                        }.into_any(),
                        Some(MigrationResult::Import(summary)) => view! {
                            <article class="record-card">
                                <h4>"Import Complete"</h4>
                                <p>{format!("Fixture: {}", summary.fixture_name)}</p>
                                <p class="muted">{format!("Form version: {}", summary.form_version_id)}</p>
                                <p class="muted">{format!("Submission: {}", summary.submission_id)}</p>
                                <p class="muted">{format!("Dashboard: {}", summary.dashboard_id)}</p>
                                <p class="muted">{format!("Analytics values: {}", summary.analytics_values)}</p>
                            </article>
                        }.into_any(),
                        Some(MigrationResult::Error(message)) => view! {
                            <p class="muted">{message}</p>
                        }.into_any(),
                        None if busy.get() => view! { <p class="muted">"Running migration action..."</p> }.into_any(),
                        None => view! { <p class="muted">"Run validation, dry-run, or import to see results here."</p> }.into_any(),
                    }}
                </div>
            </Panel>
        </NativePage>
    }
}
