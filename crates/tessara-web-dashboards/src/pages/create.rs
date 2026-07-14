use leptos::prelude::*;
use tessara_web_ui::{
    Breadcrumb, BreadcrumbItem, BreadcrumbLink, BreadcrumbPage, BreadcrumbSeparator, EmptyState,
    PageHeader,
};

use crate::types::{DashboardMetadataRequest, VisibilityNodeOption};
use crate::{DashboardRouteBootstrap, dashboard_route_bootstrap};

#[component]
pub fn DashboardCreateContent() -> impl IntoView {
    let (initial_nodes, bootstrapped) = match dashboard_route_bootstrap() {
        Some(DashboardRouteBootstrap::Create {
            visibility_nodes, ..
        }) => (visibility_nodes, true),
        _ => (Vec::new(), false),
    };
    let nodes = RwSignal::new(initial_nodes);
    let nodes_loading = RwSignal::new(cfg!(feature = "hydrate") && !bootstrapped);
    let load_error = RwSignal::new(None::<String>);
    let name = RwSignal::new(String::new());
    let description = RwSignal::new(String::new());
    let selected_nodes = RwSignal::new(Vec::<String>::new());
    let save_error = RwSignal::new(None::<String>);
    let saving = RwSignal::new(false);

    Effect::new(move |_| {
        if !bootstrapped {
            load_nodes(nodes, nodes_loading, load_error);
        }
    });

    let submit = move |event: leptos::ev::SubmitEvent| {
        event.prevent_default();
        let trimmed_name = name.get().trim().to_string();
        if trimmed_name.is_empty() {
            save_error.set(Some("Dashboard name is required.".into()));
            return;
        }
        let visibility_node_ids = selected_nodes.get();
        if visibility_node_ids.is_empty() {
            save_error.set(Some("Select at least one visibility node.".into()));
            return;
        }
        let description = description.get().trim().to_string();
        let payload = DashboardMetadataRequest {
            name: trimmed_name,
            description: (!description.is_empty()).then_some(description),
            visibility_node_ids,
        };
        create_dashboard(payload, saving, save_error);
    };

    view! {
        <section class="route-panel dashboards-page dashboard-create">
            <Breadcrumb>
                <BreadcrumbItem><BreadcrumbLink href="/dashboards">"Dashboards"</BreadcrumbLink></BreadcrumbItem>
                <BreadcrumbSeparator/>
                <BreadcrumbItem><BreadcrumbPage>"Create"</BreadcrumbPage></BreadcrumbItem>
            </Breadcrumb>
            <PageHeader
                title="Create Dashboard"
                description="Set Dashboard metadata and visibility before arranging Components."
            />

            <form class="dashboard-metadata-form" on:submit=submit>
                <label class="field-label" for="dashboard-name">"Name"</label>
                <input
                    id="dashboard-name"
                    class="text-input"
                    required
                    maxlength="160"
                    prop:value=move || name.get()
                    on:input=move |event| name.set(event_target_value(&event))
                />

                <label class="field-label" for="dashboard-description">"Description"</label>
                <textarea
                    id="dashboard-description"
                    class="text-input"
                    rows="4"
                    prop:value=move || description.get()
                    on:input=move |event| description.set(event_target_value(&event))
                ></textarea>

                <fieldset class="dashboard-scope-picker">
                    <legend>"Visibility scope"</legend>
                    <p class="field-help">"A Dashboard can place only Components whose Dataset scope is fully contained here."</p>
                    {move || if nodes_loading.get() {
                        view! { <EmptyState title="Loading scope" message="Fetching visible organization nodes."/> }.into_any()
                    } else if let Some(message) = load_error.get() {
                        view! { <EmptyState title="Scope unavailable" message=message/> }.into_any()
                    } else {
                        view! {
                            <div class="dashboard-scope-picker__options">
                                {nodes.get().into_iter().map(|node| {
                                    let node_id = node.id.clone();
                                    let checked_id = node.id.clone();
                                    let label = node.label();
                                    view! {
                                        <label class="dashboard-scope-option">
                                            <input
                                                type="checkbox"
                                                prop:checked=move || selected_nodes.get().contains(&checked_id)
                                                on:change=move |event| {
                                                    let mut selected = selected_nodes.get_untracked();
                                                    if event_target_checked(&event) {
                                                        if !selected.contains(&node_id) {
                                                            selected.push(node_id.clone());
                                                        }
                                                    } else {
                                                        selected.retain(|selected_id| selected_id != &node_id);
                                                    }
                                                    selected_nodes.set(selected);
                                                }
                                            />
                                            <span>{label}</span>
                                        </label>
                                    }
                                }).collect_view()}
                            </div>
                        }.into_any()
                    }}
                </fieldset>

                <div class="dashboard-form__status" aria-live="polite">
                    {move || save_error.get().map(|message| view! { <p class="form-error">{message}</p> })}
                </div>
                <div class="dashboard-form__actions">
                    <a class="button button--secondary" href="/dashboards">"Cancel"</a>
                    <button class="button" type="submit" disabled=move || saving.get()>
                        {move || if saving.get() { "Creating…" } else { "Create and compose" }}
                    </button>
                </div>
            </form>
        </section>
    }
}

#[cfg(feature = "hydrate")]
fn load_nodes(
    nodes: RwSignal<Vec<VisibilityNodeOption>>,
    loading: RwSignal<bool>,
    error: RwSignal<Option<String>>,
) {
    leptos::task::spawn_local(async move {
        loading.set(true);
        match crate::api::fetch_visibility_nodes().await {
            Ok(payload) => nodes.set(payload),
            Err(message) => error.set(Some(message)),
        }
        loading.set(false);
    });
}

#[cfg(not(feature = "hydrate"))]
fn load_nodes(
    _: RwSignal<Vec<VisibilityNodeOption>>,
    _: RwSignal<bool>,
    _: RwSignal<Option<String>>,
) {
}

#[cfg(feature = "hydrate")]
fn create_dashboard(
    payload: DashboardMetadataRequest,
    saving: RwSignal<bool>,
    error: RwSignal<Option<String>>,
) {
    leptos::task::spawn_local(async move {
        saving.set(true);
        error.set(None);
        match crate::api::create_dashboard(&payload).await {
            Ok(created) => {
                if let Some(window) = web_sys::window() {
                    let _ = window
                        .location()
                        .set_href(&format!("/dashboards/{}/edit", created.id));
                }
            }
            Err(message) => error.set(Some(message)),
        }
        saving.set(false);
    });
}

#[cfg(not(feature = "hydrate"))]
fn create_dashboard(_: DashboardMetadataRequest, _: RwSignal<bool>, _: RwSignal<Option<String>>) {}
