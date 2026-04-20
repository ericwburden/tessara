use leptos::prelude::*;
#[cfg(feature = "hydrate")]
use serde_json::{Value, json};

use crate::app::transitional::{TransitionalPage, render_transitional_route};
use crate::features::native_runtime::set_page_context;
#[cfg(feature = "hydrate")]
use crate::features::native_runtime::{get_json, post_json, redirect};
use crate::features::native_shell::{
    AccountContext, BreadcrumbItem, NativePage, has_capability, use_account_session,
};

#[cfg_attr(feature = "hydrate", derive(serde::Deserialize))]
#[derive(Clone)]
struct LoginSessionStateResponse {
    authenticated: bool,
}

#[cfg_attr(feature = "hydrate", derive(serde::Deserialize))]
#[derive(Clone)]
struct HomeSummaryResponse {
    published_form_versions: i64,
    draft_submissions: i64,
    submitted_submissions: i64,
    dashboards: i64,
}

#[cfg_attr(feature = "hydrate", derive(serde::Deserialize))]
#[derive(Clone)]
struct HomePendingAssignmentSummary {
    workflow_assignment_id: String,
    workflow_name: String,
    form_name: String,
    form_version_label: Option<String>,
    node_name: String,
    account_display_name: String,
}

fn quantify(count: i64, singular: &'static str, plural: &'static str) -> String {
    if count == 1 {
        singular.to_string()
    } else {
        plural.to_string()
    }
}

fn scope_root_labels(account: &AccountContext) -> Vec<(String, String)> {
    let mut roots = account
        .scope_nodes
        .iter()
        .filter(|node| {
            !account
                .scope_nodes
                .iter()
                .any(|candidate| Some(candidate.node_id.as_str()) == node.parent_node_id.as_deref())
        })
        .map(|node| (node.node_type_name.clone(), node.node_name.clone()))
        .collect::<Vec<_>>();
    roots.sort_by(|left, right| left.0.cmp(&right.0).then_with(|| left.1.cmp(&right.1)));
    roots.dedup();
    roots
}

#[component]
pub fn AdminWorkbenchPage() -> impl IntoView {
    render_transitional_route(TransitionalPage {
        title: "Tessara",
        description: "Tessara local admin workbench for migration setup and workflow testing.",
        body_html: crate::admin_shell_html(),
        page_key: "admin-shell",
        active_route: "administration",
        record_id: None,
    })
}

#[component]
pub fn HomePage() -> impl IntoView {
    let session = use_account_session();
    let home_summary = RwSignal::new(None::<HomeSummaryResponse>);
    let pending_assignments = RwSignal::new(None::<Vec<HomePendingAssignmentSummary>>);
    let home_summary_error = RwSignal::new(None::<String>);
    let home_queue_error = RwSignal::new(None::<String>);

    #[cfg(feature = "hydrate")]
    Effect::new(move |_| {
        if !session.loaded.get() || session.account.get().is_none() {
            return;
        }

        leptos::task::spawn_local(async move {
            match get_json::<HomeSummaryResponse>("/api/app/summary").await {
                Ok(summary) => {
                    home_summary.set(Some(summary));
                    home_summary_error.set(None);
                }
                Err(error) => {
                    home_summary.set(None);
                    home_summary_error.set(Some(error));
                }
            }

            match get_json::<Vec<HomePendingAssignmentSummary>>("/api/workflow-assignments/pending")
                .await
            {
                Ok(items) => {
                    pending_assignments.set(Some(items));
                    home_queue_error.set(None);
                }
                Err(error) => {
                    pending_assignments.set(None);
                    home_queue_error.set(Some(error));
                }
            }
        });
    });

    view! {
        <NativePage
            title="Tessara Home"
            description="Tessara application home for local replacement workflow testing."
            page_key="home"
            active_route="home"
            workspace_label="Shared Home"
            breadcrumbs=vec![BreadcrumbItem::current("Home")]
        >
            <section id="home-summary" class="home-summary">
                <div class="home-summary__copy">
                    <p class="eyebrow">"Shared Home"</p>
                    <h1>"Current Work"</h1>
                    <p class="muted home-summary__description">
                        "Start from pending response work, compact operational totals, and current hierarchy context."
                    </p>
                </div>
            </section>
            {move || {
                let summary = home_summary.get();
                if let Some(summary) = summary {
                    let pending_count = pending_assignments.get().map(|items| items.len()).unwrap_or(0);
                    let pending_label = if pending_count == 1 {
                        "assignment".to_string()
                    } else {
                        "assignments".to_string()
                    };
                    let dashboard_count = if summary.dashboards > 0 {
                        summary.dashboards
                    } else {
                        summary.published_form_versions
                    };
                    let dashboard_label = if summary.dashboards > 0 {
                        quantify(summary.dashboards, "dashboard", "dashboards")
                    } else {
                        quantify(
                            summary.published_form_versions,
                            "published form",
                            "published forms",
                        )
                    };

                    return view! {
                        <section id="home-metric-strip" class="home-metric-strip">
                            <article class="home-metric">
                                <strong>{pending_count}</strong>
                                <span>{pending_label}</span>
                            </article>
                            <article class="home-metric">
                                <strong>{summary.draft_submissions}</strong>
                                <span>{quantify(summary.draft_submissions, "draft response", "draft responses")}</span>
                            </article>
                            <article class="home-metric">
                                <strong>{summary.submitted_submissions}</strong>
                                <span>{quantify(summary.submitted_submissions, "submitted response", "submitted responses")}</span>
                            </article>
                            <article class="home-metric">
                                <strong>{dashboard_count}</strong>
                                <span>{dashboard_label}</span>
                            </article>
                        </section>
                    }
                    .into_any();
                }

                if let Some(error) = home_summary_error.get() {
                    return view! { <p id="home-metric-feedback" class="muted">{error}</p> }.into_any();
                }

                view! { <p id="home-metric-feedback" class="muted">"Loading home metrics..."</p> }
                    .into_any()
            }}
            <div class="home-workspace-grid">
                <section class="home-primary-panel">
                    <div class="home-panel-header">
                        <div class="home-panel-header__copy">
                            <p class="eyebrow">"Assignment queue"</p>
                            <h2>"What needs attention now"</h2>
                            <p class="muted">
                                "Pending response assignments stay primary on Home so the route starts with work rather than destination launchers."
                            </p>
                        </div>
                        <div class="actions">
                            <a class="button-link button is-light" href="/app/responses">"Open full queue"</a>
                        </div>
                    </div>
                    <div id="home-current-work" class="home-queue-panel">
                        {move || {
                            if let Some(error) = home_queue_error.get() {
                                return view! {
                                    <p id="home-queue-feedback" class="muted">{error}</p>
                                }
                                .into_any();
                            }

                            if let Some(items) = pending_assignments.get() {
                                if items.is_empty() {
                                    let draft_count = home_summary
                                        .get()
                                        .map(|summary| summary.draft_submissions)
                                        .unwrap_or_default();
                                    let submitted_count = home_summary
                                        .get()
                                        .map(|summary| summary.submitted_submissions)
                                        .unwrap_or_default();
                                    return view! {
                                        <div class="home-empty-state" id="home-queue-feedback">
                                            <strong>"No assignments are waiting right now."</strong>
                                            <p class="muted">
                                                {if draft_count > 0 {
                                                    format!("You still have {draft_count} draft response{} ready to continue in Responses.", if draft_count == 1 { "" } else { "s" })
                                                } else if submitted_count > 0 {
                                                    "The active queue is clear. Responses still holds your submitted history.".into()
                                                } else {
                                                    "Responses becomes the next stop when new assignments arrive.".into()
                                                }}
                                            </p>
                                        </div>
                                    }
                                    .into_any();
                                }

                                return view! {
                                    <div class="home-queue-list" id="home-queue-list">
                                        {items
                                            .into_iter()
                                            .take(4)
                                            .map(|item| {
                                                let assignee_label = item.account_display_name.clone();
                                                let show_assignee = !assignee_label.trim().is_empty();
                                                view! {
                                                    <article class="home-queue-row">
                                                        <div class="home-queue-row__copy">
                                                            <strong>{item.workflow_name}</strong>
                                                            <p>{item.node_name}</p>
                                                        </div>
                                                        <div class="home-queue-row__meta">
                                                            <p class="muted">
                                                                {format!(
                                                                    "Form: {} {}",
                                                                    item.form_name,
                                                                    item.form_version_label.unwrap_or_default()
                                                                )}
                                                            </p>
                                                            <Show when=move || show_assignee>
                                                                <p class="muted">
                                                                    {format!("Assigned to {}", assignee_label)}
                                                                </p>
                                                            </Show>
                                                        </div>
                                                        <div class="actions">
                                                            <a
                                                                class="button-link button is-light"
                                                                href=format!(
                                                                    "/app/responses?workflowAssignmentId={}",
                                                                    item.workflow_assignment_id
                                                                )
                                                            >
                                                                "Start"
                                                            </a>
                                                        </div>
                                                    </article>
                                                }
                                            })
                                            .collect_view()}
                                    </div>
                                }
                                .into_any();
                            }

                            view! {
                                <p id="home-queue-feedback" class="muted">"Loading current work..."</p>
                            }
                            .into_any()
                        }}
                        <div class="home-next-step">
                            <a class="button-link button is-primary" href="/app/responses">"Open Responses"</a>
                            {move || {
                                let account = session.account.get();
                                if account.as_ref().is_some_and(|account| !account.delegations.is_empty()) {
                                    return view! {
                                        <p class="muted">
                                            "Delegated response context remains available from Responses when you switch acting account."
                                        </p>
                                    }
                                    .into_any();
                                }

                                view! { <></> }.into_any()
                            }}
                        </div>
                    </div>
                </section>
                <aside class="home-secondary-panel">
                    <section id="home-hierarchy-context" class="home-secondary-section">
                        <div class="home-panel-header__copy">
                            <p class="eyebrow">"Hierarchy context"</p>
                            <h2>"Organization scope"</h2>
                            <p class="muted">
                                "Home keeps hierarchy awareness visible without turning the body back into a second navigation launcher."
                            </p>
                        </div>
                        <div class="home-hierarchy-panel">
                            {move || {
                                let account = session.account.get();
                                let Some(account) = account else {
                                    return view! { <p class="muted">"Loading hierarchy context..."</p> }.into_any();
                                };

                                if !has_capability(Some(&account), "hierarchy:read") {
                                    return view! {
                                        <p class="muted">
                                            "This account does not currently have Organization access."
                                        </p>
                                    }
                                    .into_any();
                                }

                                let roots = scope_root_labels(&account);
                                if roots.is_empty() {
                                    return view! {
                                        <>
                                            <p class="muted">"Full application access across the organization tree."</p>
                                            <div class="actions">
                                                <a class="button-link button is-light" href="/app/organization">"Open Organization"</a>
                                            </div>
                                        </>
                                    }
                                    .into_any();
                                }

                                view! {
                                    <>
                                        <ul id="home-hierarchy-list" class="home-hierarchy-list">
                                            {roots
                                                .into_iter()
                                                .map(|(node_type, node_name)| view! {
                                                    <li class="home-hierarchy-item">
                                                        <span class="home-hierarchy-item__type">{node_type}</span>
                                                        <span class="home-hierarchy-item__name">{node_name}</span>
                                                    </li>
                                                })
                                                .collect_view()}
                                        </ul>
                                        <div class="actions">
                                            <a class="button-link button is-light" href="/app/organization">"Open Organization"</a>
                                        </div>
                                    </>
                                }
                                .into_any()
                            }}
                        </div>
                    </section>
                    <section id="home-operational-snapshot" class="home-secondary-section">
                        <div class="home-panel-header__copy">
                            <p class="eyebrow">"Operational snapshot"</p>
                            <h2>"Current totals"</h2>
                            <p class="muted">
                                "Compact totals stay glanceable here instead of expanding into a second dashboard row."
                            </p>
                        </div>
                        <div class="home-snapshot-list">
                            {move || {
                                if let Some(summary) = home_summary.get() {
                                    return view! {
                                        <div class="home-snapshot-grid">
                                            <div class="home-snapshot-item">
                                                <span>"Draft responses"</span>
                                                <strong>{summary.draft_submissions}</strong>
                                            </div>
                                            <div class="home-snapshot-item">
                                                <span>"Submitted responses"</span>
                                                <strong>{summary.submitted_submissions}</strong>
                                            </div>
                                            <div class="home-snapshot-item">
                                                <span>"Published forms"</span>
                                                <strong>{summary.published_form_versions}</strong>
                                            </div>
                                            <div class="home-snapshot-item">
                                                <span>"Dashboards"</span>
                                                <strong>{summary.dashboards}</strong>
                                            </div>
                                        </div>
                                    }
                                    .into_any();
                                }

                                if let Some(error) = home_summary_error.get() {
                                    return view! { <p class="muted">{error}</p> }.into_any();
                                }

                                view! { <p class="muted">"Loading operational totals..."</p> }.into_any()
                            }}
                        </div>
                    </section>
                </aside>
            </div>
        </NativePage>
    }
}

#[component]
pub fn LoginPage() -> impl IntoView {
    let session_checked = RwSignal::new(false);
    let email = RwSignal::new(String::new());
    let password = RwSignal::new(String::new());
    let feedback = RwSignal::new(None::<String>);
    let busy = RwSignal::new(false);

    set_page_context("login", "login", None);

    #[cfg(feature = "hydrate")]
    Effect::new(move |_| {
        leptos::task::spawn_local(async move {
            match get_json::<LoginSessionStateResponse>("/api/auth/session").await {
                Ok(response) if response.authenticated => redirect("/app"),
                Ok(_) | Err(_) => session_checked.set(true),
            }
        });
    });

    let submit = move |_| {
        if busy.get_untracked() {
            return;
        }

        let email_value = email.get_untracked().trim().to_string();
        let password_value = password.get_untracked();
        if email_value.is_empty() || password_value.is_empty() {
            feedback.set(Some("Enter an email address and password.".into()));
            return;
        }

        busy.set(true);
        feedback.set(None);

        #[cfg(feature = "hydrate")]
        leptos::task::spawn_local(async move {
            let body = json!({
                "email": email_value,
                "password": password_value,
            });

            match post_json::<Value>("/api/auth/login", &body).await {
                Ok(_) => redirect("/app"),
                Err(error) => feedback.set(Some(error)),
            }

            busy.set(false);
        });
    };

    view! {
        <main class="auth-shell" data-auth-surface>
            <section class="auth-shell__panel">
                <div class="auth-shell__brand">
                    <img class="auth-shell__mark" src="/assets/tessara-icon-256.svg" alt="" />
                    <div class="auth-shell__copy">
                        <p class="eyebrow">"Access"</p>
                        <h1>"Sign In"</h1>
                        <p class="muted">
                            "Use your Tessara account to open the shared workspace."
                        </p>
                    </div>
                </div>
                <form
                    id="login-form"
                    class="stacked-form auth-shell__form"
                    on:submit=move |event| {
                        event.prevent_default();
                        submit(());
                    }
                >
                    <div
                        id="login-feedback"
                        class="notification is-danger is-light"
                        class:is-hidden=move || feedback.get().is_none()
                        role="alert"
                    >
                        {move || feedback.get().unwrap_or_default()}
                    </div>
                    <div class="form-grid">
                        <label class="field">
                            <span>"Email"</span>
                            <input
                                id="login-email"
                                class="input"
                                type="email"
                                autocomplete="username"
                                prop:value=email
                                on:input=move |event| email.set(event_target_value(&event))
                            />
                        </label>
                        <label class="field">
                            <span>"Password"</span>
                            <input
                                id="login-password"
                                class="input"
                                type="password"
                                autocomplete="current-password"
                                prop:value=password
                                on:input=move |event| password.set(event_target_value(&event))
                            />
                        </label>
                    </div>
                    <div class="actions">
                        <button class="button-link button is-primary" type="submit" disabled=move || busy.get()>
                            {move || if busy.get() { "Signing In..." } else { "Sign In" }}
                        </button>
                    </div>
                </form>
                <Show when=move || !session_checked.get()>
                    <p class="auth-shell__status muted">"Checking your current session..."</p>
                </Show>
            </section>
        </main>
    }
}
