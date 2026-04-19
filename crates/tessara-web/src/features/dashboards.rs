use leptos::prelude::*;

use crate::features::native_shell::{BreadcrumbItem, MetadataStrip, NativePage, PageHeader, Panel};
use crate::infra::routing::{DashboardRouteParams, require_route_params};

#[cfg(feature = "hydrate")]
use crate::features::native_runtime::{get_json, post_json, put_json, redirect};
use serde::Deserialize;
#[cfg(feature = "hydrate")]
use serde_json::json;
#[cfg(feature = "hydrate")]
use wasm_bindgen_futures::spawn_local;

#[derive(Clone, Deserialize)]
struct DashboardSummary {
    id: String,
    name: String,
    component_count: i64,
}

#[derive(Clone, Deserialize)]
struct ChartResponse {
    id: String,
    name: String,
    chart_type: String,
    report_name: Option<String>,
    aggregation_name: Option<String>,
}

#[derive(Clone, Deserialize)]
struct DashboardComponentResponse {
    id: String,
    position: i32,
    chart: ChartResponse,
}

#[derive(Clone, Deserialize)]
struct DashboardResponse {
    id: String,
    name: String,
    components: Vec<DashboardComponentResponse>,
}

#[derive(Clone, Deserialize)]
struct IdResponse {
    id: String,
}

#[component]
pub fn DashboardsPage() -> impl IntoView {
    let dashboards = RwSignal::new(Vec::<DashboardSummary>::new());
    let loading = RwSignal::new(true);
    let error = RwSignal::new(None::<String>);

    #[cfg(feature = "hydrate")]
    Effect::new(move |_| {
        spawn_local(async move {
            loading.set(true);
            match get_json::<Vec<DashboardSummary>>("/api/dashboards").await {
                Ok(items) => {
                    dashboards.set(items);
                    error.set(None);
                }
                Err(message) => error.set(Some(message)),
            }
            loading.set(false);
        });
    });

    view! {
        <NativePage
            title="Tessara Dashboards"
            description="Tessara dashboards list screen."
            page_key="dashboard-list"
            active_route="dashboards"
            workspace_label="Product Area"
            required_capability="reports:read"
            breadcrumbs=vec![BreadcrumbItem::current("Home"), BreadcrumbItem::current("Dashboards")]
        >
            <PageHeader
                eyebrow="Product Area"
                title="Dashboards"
                description="Browse dashboards and inspect the current component footprint without leaving the native application shell."
            />
            <MetadataStrip items=vec![
                ("Mode", "Directory".into()),
                ("Surface", "Dashboard runtime overview".into()),
                ("State", "Native SSR shell".into()),
            ]/>
            <Panel title="Dashboard Directory" description="Current dashboard records and component counts appear here.">
                <div id="dashboard-list" class="record-list">
                    <Show
                        when=move || !loading.get()
                        fallback=|| view! { <p class="muted">"Loading dashboard records..."</p> }
                    >
                        {move || {
                            if let Some(message) = error.get() {
                                return view! { <p class="muted">{message}</p> }.into_any();
                            }
                            let items = dashboards.get();
                            if items.is_empty() {
                                return view! { <p class="muted">"No dashboard records found."</p> }.into_any();
                            }
                            view! {
                                {items
                                    .into_iter()
                                    .map(|dashboard| {
                                        let detail_href = format!("/app/dashboards/{}", dashboard.id);
                                        let edit_href = format!("{detail_href}/edit");
                                        view! {
                                            <article class="record-card">
                                                <h4>{dashboard.name}</h4>
                                                <p class="muted">{format!("{} components", dashboard.component_count)}</p>
                                                <div class="actions">
                                                    <a class="button-link" href=detail_href.clone()>"View"</a>
                                                    <a class="button-link button is-light" href=edit_href>"Edit"</a>
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
        </NativePage>
    }
}

#[component]
pub fn DashboardCreatePage() -> impl IntoView {
    dashboard_form_page(None)
}

#[component]
pub fn DashboardDetailPage() -> impl IntoView {
    let DashboardRouteParams { dashboard_id } = require_route_params();
    let dashboard = RwSignal::new(None::<DashboardResponse>);
    let loading = RwSignal::new(true);
    let error = RwSignal::new(None::<String>);
    let record_id = dashboard_id.clone();

    #[cfg(feature = "hydrate")]
    Effect::new(move |_| {
        let dashboard_id = dashboard_id.clone();
        spawn_local(async move {
            loading.set(true);
            match get_json::<DashboardResponse>(&format!("/api/dashboards/{dashboard_id}")).await {
                Ok(item) => {
                    dashboard.set(Some(item));
                    error.set(None);
                }
                Err(message) => error.set(Some(message)),
            }
            loading.set(false);
        });
    });

    view! {
        <NativePage
            title="Dashboard Detail"
            description="Inspect a Tessara dashboard."
            page_key="dashboard-detail"
            active_route="dashboards"
            workspace_label="Product Area"
            record_id=record_id.clone()
            required_capability="reports:read"
            breadcrumbs=vec![
                BreadcrumbItem::link("Home", "/app"),
                BreadcrumbItem::link("Dashboards", "/app/dashboards"),
                BreadcrumbItem::current("Dashboard Detail"),
            ]
        >
            <PageHeader
                eyebrow="Product Area"
                title="Dashboard Detail"
                description="Review the selected dashboard and its current component summary."
            />
            <MetadataStrip items=vec![
                ("Mode", "Detail".into()),
                ("Surface", "Dashboard inspection".into()),
                ("State", "Native SSR shell".into()),
            ]/>
            <Panel title="Dashboard Summary" description="Dashboard identity and component coverage appear here.">
                <div id="dashboard-detail" class="record-list">
                    <Show
                        when=move || !loading.get()
                        fallback=|| view! { <p class="muted">"Loading dashboard detail..."</p> }
                    >
                        {move || {
                            if let Some(message) = error.get() {
                                return view! { <p class="muted">{message}</p> }.into_any();
                            }
                            match dashboard.get() {
                                Some(dashboard) => view! {
                                    <article class="record-card">
                                        <h4>{dashboard.name.clone()}</h4>
                                        <p class="muted">{format!("{} components", dashboard.components.len())}</p>
                                    </article>
                                }.into_any(),
                                None => view! { <p class="muted">"Dashboard detail is unavailable."</p> }.into_any(),
                            }
                        }}
                    </Show>
                </div>
            </Panel>
            <Panel title="Component Summary" description="Charts currently assigned to this dashboard appear here.">
                <div id="dashboard-component-summary" class="record-list">
                    {move || {
                        if loading.get() {
                            return view! { <p class="muted">"Loading components..."</p> }.into_any();
                        }
                        if error.get().is_some() {
                            return view! { <p class="muted">"Component detail is unavailable while the dashboard fails to load."</p> }.into_any();
                        }
                        match dashboard.get() {
                            Some(dashboard) if dashboard.components.is_empty() => {
                                view! { <p class="muted">"No dashboard components found."</p> }.into_any()
                            }
                            Some(dashboard) => view! {
                                {dashboard
                                    .components
                                    .into_iter()
                                    .map(|component| {
                                        let source = component
                                            .chart
                                            .report_name
                                            .clone()
                                            .or(component.chart.aggregation_name.clone())
                                            .unwrap_or_else(|| "Unknown source".into());
                                        view! {
                                            <article class="record-card compact-record-card">
                                                <h4>{component.chart.name}</h4>
                                                <p>{format!("{} chart", component.chart.chart_type)}</p>
                                                <p class="muted">{format!("Source: {source}")}</p>
                                                <p class="muted">{format!("Position {}", component.position)}</p>
                                            </article>
                                        }
                                    })
                                    .collect_view()}
                            }.into_any(),
                            None => view! { <p class="muted">"Dashboard detail is unavailable."</p> }.into_any(),
                        }
                    }}
                </div>
            </Panel>
        </NativePage>
    }
}

#[component]
pub fn DashboardEditPage() -> impl IntoView {
    let DashboardRouteParams { dashboard_id } = require_route_params();
    dashboard_form_page(Some(dashboard_id))
}

fn dashboard_form_page(dashboard_id: Option<String>) -> impl IntoView {
    let name = RwSignal::new(String::new());
    let loading = RwSignal::new(dashboard_id.is_some());
    let busy = RwSignal::new(false);
    let error = RwSignal::new(None::<String>);
    let form_status = RwSignal::new(None::<String>);
    let is_edit = dashboard_id.is_some();
    let record_id = dashboard_id.clone();

    let _dashboard_id_for_load = dashboard_id.clone();
    #[cfg(feature = "hydrate")]
    if let Some(dashboard_id) = _dashboard_id_for_load {
        Effect::new(move |_| {
            let dashboard_id = dashboard_id.clone();
            spawn_local(async move {
                loading.set(true);
                match get_json::<DashboardResponse>(&format!("/api/dashboards/{dashboard_id}")).await {
                    Ok(dashboard) => {
                        name.set(dashboard.name);
                        error.set(None);
                    }
                    Err(message) => error.set(Some(message)),
                }
                loading.set(false);
            });
        });
    }

    let cancel_href = StoredValue::new("/app/dashboards".to_string());
    let _dashboard_id_for_submit = dashboard_id.clone();
    let submit = StoredValue::new(std::sync::Arc::new(move |event: leptos::ev::SubmitEvent| {
        event.prevent_default();
        if busy.get_untracked() {
            return;
        }

        busy.set(true);
        error.set(None);
        form_status.set(Some(if is_edit {
            "Saving dashboard changes...".into()
        } else {
            "Creating dashboard...".into()
        }));

        #[cfg(feature = "hydrate")]
        {
            let dashboard_id = _dashboard_id_for_submit.clone();
            let payload_name = name.get_untracked();
            spawn_local(async move {
                let payload = json!({ "name": payload_name.trim() });
                let result = match dashboard_id {
                    Some(dashboard_id) => {
                        put_json::<IdResponse>(&format!("/api/admin/dashboards/{dashboard_id}"), &payload).await
                    }
                    None => post_json::<IdResponse>("/api/admin/dashboards", &payload).await,
                };
                match result {
                    Ok(response) => redirect(&format!("/app/dashboards/{}", response.id)),
                    Err(message) => {
                        error.set(Some(message));
                        form_status.set(Some("Unable to save the dashboard.".into()));
                    }
                }
                busy.set(false);
            });
        }
    }));

    let title = if is_edit { "Edit Dashboard" } else { "Create Dashboard" };
    let description = if is_edit {
        "Edit the selected dashboard from a dedicated native form screen."
    } else {
        "Create a top-level dashboard from a dedicated native form screen."
    };

    view! {
        <NativePage
            title=title
            description=description
            page_key=if is_edit { "dashboard-edit" } else { "dashboard-create" }
            active_route="dashboards"
            workspace_label="Product Area"
            record_id=record_id.unwrap_or_default()
            required_capability="reports:write"
            breadcrumbs={
                let mut items = vec![
                    BreadcrumbItem::link("Home", "/app"),
                    BreadcrumbItem::link("Dashboards", "/app/dashboards"),
                ];
                if is_edit {
                    if let Some(dashboard_id) = dashboard_id.clone() {
                        items.push(BreadcrumbItem::link("Dashboard Detail", format!("/app/dashboards/{dashboard_id}")));
                    }
                }
                items.push(BreadcrumbItem::current(if is_edit { "Edit Dashboard" } else { "New Dashboard" }));
                items
            }
        >
            <PageHeader
                eyebrow="Product Area"
                title=title
                description=description
            />
            <MetadataStrip items=vec![
                ("Mode", if is_edit { "Edit".into() } else { "Create".into() }),
                ("Surface", "Dashboard authoring".into()),
                ("State", if loading.get() { "Loading dashboard".into() } else { "Native SSR shell".into() }),
            ]/>
            <Panel title="Dashboard Form" description="Dashboard identity changes save here.">
                <form id="dashboard-form" class="entity-form" on:submit=move |event| submit.with_value(|submit| submit(event))>
                    <div class="form-grid">
                        <div class="form-field wide-field">
                            <label for="dashboard-name">"Name"</label>
                            <input
                                id="dashboard-name"
                                class="input"
                                type="text"
                                prop:value=move || name.get()
                                on:input=move |event| name.set(event_target_value(&event))
                                placeholder="Dashboard name"
                            />
                        </div>
                    </div>
                    <p id="dashboard-form-status" class="muted">
                        {move || {
                            if let Some(message) = error.get() {
                                message
                            } else {
                                form_status.get().unwrap_or_else(|| {
                                    if loading.get() {
                                        "Loading dashboard authoring surface...".into()
                                    } else if is_edit {
                                        "Update the dashboard name and save changes here.".into()
                                    } else {
                                        "Provide a name and create a new dashboard.".into()
                                    }
                                })
                            }
                        }}
                    </p>
                    <div class="actions">
                        <button class="button-link" type="submit" disabled=move || busy.get() || loading.get()>
                            {move || if busy.get() {
                                if is_edit { "Saving..." } else { "Creating..." }
                            } else if is_edit {
                                "Save Dashboard"
                            } else {
                                "Create Dashboard"
                            }}
                        </button>
                        <a class="button-link button is-light" href=move || cancel_href.get_value()>"Cancel"</a>
                    </div>
                </form>
            </Panel>
        </NativePage>
    }
}
