use leptos::prelude::*;
use serde::Deserialize;
use std::sync::Arc;

#[cfg(feature = "hydrate")]
use crate::features::native_runtime::get_json;
use crate::features::native_shell::{BreadcrumbItem, MetadataStrip, NativePage, PageHeader, Panel};
use crate::infra::routing::{ComponentRouteParams, require_route_params};
#[cfg(feature = "hydrate")]
use wasm_bindgen_futures::spawn_local;

#[derive(Clone, Deserialize)]
struct ChartResponse {
    id: String,
    name: String,
    chart_type: String,
    report_id: Option<String>,
    report_name: Option<String>,
    report_form_name: Option<String>,
    aggregation_id: Option<String>,
    aggregation_name: Option<String>,
    aggregation_report_name: Option<String>,
}

#[derive(Clone, Deserialize)]
struct DashboardSummary {
    id: String,
    name: String,
    component_count: i64,
}

#[derive(Clone, Deserialize)]
struct ChartDefinition {
    chart: ChartResponse,
    dashboards: Vec<DashboardSummary>,
}

#[component]
pub fn ComponentsPage() -> impl IntoView {
    let charts = RwSignal::new(Vec::<ChartResponse>::new());
    let loading = RwSignal::new(true);
    let error = RwSignal::new(None::<String>);

    #[cfg(feature = "hydrate")]
    Effect::new(move |_| {
        spawn_local(async move {
            loading.set(true);
            match get_json::<Vec<ChartResponse>>("/api/charts").await {
                Ok(items) => {
                    charts.set(items);
                    error.set(None);
                }
                Err(message) => error.set(Some(message)),
            }
            loading.set(false);
        });
    });

    view! {
        <NativePage
            title="Tessara Components"
            description="Tessara components list screen."
            page_key="component-list"
            active_route="components"
            workspace_label="Product Area"
            required_capability="reports:read"
            breadcrumbs=vec![
                BreadcrumbItem::link("Home", "/app"),
                BreadcrumbItem::current("Components"),
            ]
        >
            <PageHeader
                eyebrow="Product Area"
                title="Components"
                description="Browse dashboard component definitions and inspect how each component maps to a dashboard, chart, and report surface."
                actions=Arc::new(|| {
                    view! {
                        <a class="button-link button is-light" href="/app/dashboards">"Open Dashboards"</a>
                    }
                    .into_any()
                })
            />
            <MetadataStrip items=vec![
                ("Mode", "Directory".into()),
                ("Surface", "Internal composition".into()),
                ("State", "Native SSR shell".into()),
            ]/>
            <Panel
                title="Component Directory"
                description="Components remain internal-facing, but they should be inspectable from the shared shell without relying on the legacy workbench."
            >
                <div id="component-list" class="record-list">
                    <Show
                        when=move || !loading.get()
                        fallback=|| view! { <p class="muted">"Loading dashboard components..."</p> }
                    >
                        {move || {
                            if let Some(message) = error.get() {
                                return view! { <p class="muted">{message}</p> }.into_any();
                            }

                            let items = charts.get();
                            if items.is_empty() {
                                return view! { <p class="muted">"No dashboard components are readable yet."</p> }.into_any();
                            }

                            view! {
                                {items
                                    .into_iter()
                                    .map(|chart| {
                                        let detail_href = format!("/app/components/{}", chart.id);
                                        let source_name = chart
                                            .report_name
                                            .clone()
                                            .or(chart.aggregation_name.clone())
                                            .unwrap_or_else(|| "No linked source".into());
                                        let source_kind = if chart.report_id.is_some() {
                                            "Report source"
                                        } else if chart.aggregation_id.is_some() {
                                            "Aggregation source"
                                        } else {
                                            "Unlinked source"
                                        };
                                        view! {
                                            <article class="record-card compact-record-card">
                                                <h4>{chart.name}</h4>
                                                <p>{format!("{} chart", chart.chart_type)}</p>
                                                <p class="muted">{format!("{source_kind}: {source_name}")}</p>
                                                <div class="actions">
                                                    <a class="button-link" href=detail_href>"View"</a>
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
pub fn ComponentDetailPage() -> impl IntoView {
    let ComponentRouteParams { component_ref } = require_route_params();
    let definition = RwSignal::new(None::<ChartDefinition>);
    let loading = RwSignal::new(true);
    let error = RwSignal::new(None::<String>);
    let record_id = component_ref.clone();

    #[cfg(feature = "hydrate")]
    Effect::new(move |_| {
        let component_ref = component_ref.clone();
        spawn_local(async move {
            loading.set(true);
            match get_json::<ChartDefinition>(&format!("/api/charts/{component_ref}")).await {
                Ok(item) => {
                    definition.set(Some(item));
                    error.set(None);
                }
                Err(message) => error.set(Some(message)),
            }
            loading.set(false);
        });
    });

    view! {
        <NativePage
            title="Component Detail"
            description="Inspect a Tessara component definition."
            page_key="component-detail"
            active_route="components"
            workspace_label="Product Area"
            record_id=record_id
            required_capability="reports:read"
            breadcrumbs=vec![
                BreadcrumbItem::link("Home", "/app"),
                BreadcrumbItem::link("Components", "/app/components"),
                BreadcrumbItem::current("Component Detail"),
            ]
        >
            <PageHeader
                eyebrow="Product Area"
                title="Component Detail"
                description="Review the selected dashboard component, its chart dependency, and the dashboard context that currently owns it."
                actions=Arc::new(|| {
                    view! {
                        <a class="button-link button is-light" href="/app/components">"Back to List"</a>
                        <a class="button-link button is-light" href="/app/dashboards">"Open Dashboards"</a>
                    }
                    .into_any()
                })
            />
            <MetadataStrip items=vec![
                ("Mode", "Detail".into()),
                ("Surface", "Component definition".into()),
                ("State", "Native SSR shell".into()),
            ]/>
            <Panel
                title="Component Definition"
                description="The linked chart source and runtime footprint remain visible here."
            >
                <div id="component-detail" class="record-list">
                    <Show
                        when=move || !loading.get()
                        fallback=|| view! { <p class="muted">"Loading component detail..."</p> }
                    >
                        {move || {
                            if let Some(message) = error.get() {
                                return view! { <p class="muted">{message}</p> }.into_any();
                            }

                            match definition.get() {
                                Some(definition) => {
                                    let chart = definition.chart;
                                    let source_name = chart
                                        .report_name
                                        .clone()
                                        .or(chart.aggregation_name.clone())
                                        .unwrap_or_else(|| "No linked source".into());
                                    let source_meta = chart
                                        .report_form_name
                                        .clone()
                                        .or(chart.aggregation_report_name.clone())
                                        .unwrap_or_else(|| "No runtime record attached".into());
                                    view! {
                                        <article class="record-card">
                                            <h4>{chart.name}</h4>
                                            <div class="detail-grid">
                                                <p><strong>"Chart Type:"</strong> {format!(" {}", chart.chart_type)}</p>
                                                <p><strong>"Source:"</strong> {format!(" {source_name}")}</p>
                                                <p><strong>"Runtime Context:"</strong> {format!(" {source_meta}")}</p>
                                                <p><strong>"Dashboards:"</strong> {format!(" {}", definition.dashboards.len())}</p>
                                            </div>
                                        </article>
                                    }.into_any()
                                }
                                None => view! { <p class="muted">"Component detail is unavailable."</p> }.into_any(),
                            }
                        }}
                    </Show>
                </div>
            </Panel>
            <Panel
                title="Dashboard Usage"
                description="Dashboards currently using this component stay visible from the detail route."
            >
                <div id="component-dashboard-summary" class="record-list">
                    {move || {
                        if loading.get() {
                            return view! { <p class="muted">"Loading dashboard usage..."</p> }.into_any();
                        }
                        if error.get().is_some() {
                            return view! { <p class="muted">"Dashboard usage is unavailable while the component fails to load."</p> }.into_any();
                        }

                        match definition.get() {
                            Some(definition) if definition.dashboards.is_empty() => {
                                view! { <p class="muted">"No dashboards currently use this component."</p> }.into_any()
                            }
                            Some(definition) => view! {
                                {definition.dashboards
                                    .into_iter()
                                    .map(|dashboard| {
                                        view! {
                                            <article class="record-card compact-record-card">
                                                <h4>{dashboard.name}</h4>
                                                <p class="muted">{format!("{} components in dashboard", dashboard.component_count)}</p>
                                                <div class="actions">
                                                    <a class="button-link" href=format!("/app/dashboards/{}", dashboard.id)>"Open Dashboard"</a>
                                                </div>
                                            </article>
                                        }
                                    })
                                    .collect_view()}
                            }.into_any(),
                            None => view! { <p class="muted">"Dashboard usage is unavailable."</p> }.into_any(),
                        }
                    }}
                </div>
            </Panel>
        </NativePage>
    }
}
