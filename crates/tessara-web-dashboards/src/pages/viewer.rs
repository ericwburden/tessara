#[cfg(any(test, all(feature = "hydrate", target_arch = "wasm32")))]
use std::collections::VecDeque;

#[cfg(all(feature = "hydrate", target_arch = "wasm32"))]
use std::{
    cell::{Cell, RefCell},
    collections::HashMap,
};

use leptos::html;
use leptos::prelude::*;
use tessara_web_component_viewer::{
    ComponentRequestActivity, ComponentRequestActivityCallback, ComponentTablePresentation,
    ComponentVersionExecutionContent, ComponentVersionKind, ComponentVersionTarget,
    ComponentViewerMode,
};
use tessara_web_ui::{
    Breadcrumb, BreadcrumbItem, BreadcrumbLink, BreadcrumbPage, BreadcrumbSeparator, Button,
    EmptyState, PageHeader,
};

use crate::types::{
    Dashboard, DashboardPlacement, DashboardPlacementAvailability,
    DashboardPlacementResolutionState,
};
use crate::{DashboardRouteBootstrap, dashboard_route_bootstrap};

use super::detail::placement_style;

/// Hard execution ceiling for the focused viewer. This remains lower than the
/// 240-placement storage contract and is intentionally not a Dashboard setting.
pub const DASHBOARD_VIEWER_MAX_CONCURRENT_EXECUTIONS: usize = 6;

/// Bounds retained renderer DOM independently from the 240 saved footprints.
/// Offscreen renderers are evicted only after settling; Table query/page state
/// is restored by the exact-version viewer's opaque persistence key.
#[cfg(any(test, all(feature = "hydrate", target_arch = "wasm32")))]
pub const DASHBOARD_VIEWER_MAX_MOUNTED_RENDERERS: usize = 48;

#[cfg(any(test, all(feature = "hydrate", target_arch = "wasm32")))]
#[derive(Clone, Debug, PartialEq, Eq)]
enum ExecutionSchedulerAction {
    Activate(String),
    Deactivate(String),
    SetBusy(String, bool),
    Unmount(String),
}

#[cfg(all(feature = "hydrate", target_arch = "wasm32"))]
impl ExecutionSchedulerAction {
    fn placement_id(&self) -> &str {
        match self {
            Self::Activate(placement_id)
            | Self::Deactivate(placement_id)
            | Self::SetBusy(placement_id, _)
            | Self::Unmount(placement_id) => placement_id,
        }
    }
}

#[cfg(any(test, all(feature = "hydrate", target_arch = "wasm32")))]
#[derive(Clone, Debug, PartialEq, Eq)]
struct ExecutionEntry {
    placement_id: String,
    eligible: bool,
    requested: bool,
    permit: bool,
    busy_requests: usize,
    mounted: bool,
    removed: bool,
}

#[cfg(any(test, all(feature = "hydrate", target_arch = "wasm32")))]
#[derive(Debug)]
struct ExecutionSchedulerState {
    capacity: usize,
    mounted_capacity: usize,
    entries: Vec<ExecutionEntry>,
    waiting: VecDeque<String>,
}

#[cfg(any(test, all(feature = "hydrate", target_arch = "wasm32")))]
impl ExecutionSchedulerState {
    fn new(capacity: usize, mounted_capacity: usize) -> Self {
        assert!(
            capacity > 0,
            "execution scheduler capacity must be positive"
        );
        assert!(
            mounted_capacity >= capacity,
            "mounted renderer capacity must cover every execution permit"
        );
        Self {
            capacity,
            mounted_capacity,
            entries: Vec::new(),
            waiting: VecDeque::new(),
        }
    }

    fn enter_viewport(&mut self, placement_id: String) -> Vec<ExecutionSchedulerAction> {
        let index = self.ensure_entry(placement_id.clone());
        let entry = &mut self.entries[index];
        entry.eligible = true;
        entry.removed = false;
        if !entry.mounted {
            entry.requested = true;
        }
        self.enqueue_if_ready(&placement_id);
        self.promote_waiting()
    }

    fn leave_viewport(&mut self, placement_id: &str) -> Vec<ExecutionSchedulerAction> {
        self.waiting.retain(|waiting| waiting != placement_id);
        let Some(index) = self
            .entries
            .iter()
            .position(|entry| entry.placement_id == placement_id)
        else {
            return Vec::new();
        };

        let entry = &mut self.entries[index];
        entry.eligible = false;
        let mut actions = Vec::new();
        if entry.permit {
            actions.push(ExecutionSchedulerAction::Deactivate(
                placement_id.to_string(),
            ));
            if entry.busy_requests == 0 {
                entry.permit = false;
                entry.requested = true;
            }
        }
        actions.extend(self.promote_waiting());
        actions
    }

    fn remove(&mut self, placement_id: &str) -> Vec<ExecutionSchedulerAction> {
        self.waiting.retain(|waiting| waiting != placement_id);
        let Some(index) = self
            .entries
            .iter()
            .position(|entry| entry.placement_id == placement_id)
        else {
            return Vec::new();
        };
        let entry = &mut self.entries[index];
        entry.eligible = false;
        entry.requested = false;
        entry.removed = true;
        let mut actions = Vec::new();
        if entry.permit {
            actions.push(ExecutionSchedulerAction::Deactivate(
                placement_id.to_string(),
            ));
        }
        if entry.busy_requests == 0 {
            if entry.mounted {
                actions.push(ExecutionSchedulerAction::Unmount(placement_id.to_string()));
            }
            self.entries.remove(index);
            actions.extend(self.promote_waiting());
        }
        actions
    }

    fn record_activity(
        &mut self,
        placement_id: &str,
        activity: ComponentRequestActivity,
    ) -> Vec<ExecutionSchedulerAction> {
        let Some(index) = self
            .entries
            .iter()
            .position(|entry| entry.placement_id == placement_id)
        else {
            return Vec::new();
        };

        match activity {
            ComponentRequestActivity::Requested => {
                let entry = &mut self.entries[index];
                if entry.removed || entry.permit {
                    return Vec::new();
                }
                entry.requested = true;
                self.enqueue_if_ready(placement_id);
                self.promote_waiting()
            }
            ComponentRequestActivity::Started => {
                let entry = &mut self.entries[index];
                if !entry.permit {
                    return Vec::new();
                }
                entry.busy_requests = entry.busy_requests.saturating_add(1);
                (entry.busy_requests == 1)
                    .then(|| ExecutionSchedulerAction::SetBusy(placement_id.to_string(), true))
                    .into_iter()
                    .collect()
            }
            ComponentRequestActivity::Settled => {
                let entry = &mut self.entries[index];
                if entry.busy_requests == 0 {
                    return Vec::new();
                }
                entry.busy_requests -= 1;
                if entry.busy_requests > 0 {
                    return Vec::new();
                }
                let mut actions = vec![ExecutionSchedulerAction::SetBusy(
                    placement_id.to_string(),
                    false,
                )];
                if entry.permit {
                    entry.permit = false;
                    actions.push(ExecutionSchedulerAction::Deactivate(
                        placement_id.to_string(),
                    ));
                }
                if entry.removed {
                    if entry.mounted {
                        actions.push(ExecutionSchedulerAction::Unmount(placement_id.to_string()));
                    }
                    self.entries.remove(index);
                }
                actions.extend(self.promote_waiting());
                actions
            }
        }
    }

    fn promote_waiting(&mut self) -> Vec<ExecutionSchedulerAction> {
        let mut actions = Vec::new();
        while self.active_count_internal() < self.capacity {
            let Some(placement_id) = self.waiting.pop_front() else {
                break;
            };
            let Some(index) = self
                .entries
                .iter()
                .position(|entry| entry.placement_id == placement_id)
            else {
                continue;
            };
            if self.entries[index].removed
                || !self.entries[index].eligible
                || !self.entries[index].requested
                || self.entries[index].permit
            {
                continue;
            }
            if !self.entries[index].mounted
                && self.mounted_count_internal() >= self.mounted_capacity
            {
                let Some(eviction_index) = self.entries.iter().position(|entry| {
                    entry.mounted
                        && !entry.eligible
                        && !entry.permit
                        && entry.busy_requests == 0
                        && !entry.removed
                }) else {
                    self.waiting.push_front(placement_id);
                    break;
                };
                let evicted_id = self.entries[eviction_index].placement_id.clone();
                self.entries[eviction_index].mounted = false;
                self.entries[eviction_index].requested = true;
                actions.push(ExecutionSchedulerAction::Unmount(evicted_id));
            }
            let entry = &mut self.entries[index];
            entry.permit = true;
            entry.requested = false;
            entry.mounted = true;
            actions.push(ExecutionSchedulerAction::Activate(placement_id));
        }
        actions
    }

    fn ensure_entry(&mut self, placement_id: String) -> usize {
        if let Some(index) = self
            .entries
            .iter()
            .position(|entry| entry.placement_id == placement_id)
        {
            return index;
        }
        self.entries.push(ExecutionEntry {
            placement_id,
            eligible: false,
            requested: false,
            permit: false,
            busy_requests: 0,
            mounted: false,
            removed: false,
        });
        self.entries.len() - 1
    }

    fn enqueue_if_ready(&mut self, placement_id: &str) {
        let ready = self.entries.iter().any(|entry| {
            entry.placement_id == placement_id
                && entry.eligible
                && entry.requested
                && !entry.permit
                && !entry.removed
        });
        if ready && !self.waiting.iter().any(|waiting| waiting == placement_id) {
            self.waiting.push_back(placement_id.to_string());
        }
    }

    fn contains(&self, placement_id: &str) -> bool {
        self.entries
            .iter()
            .any(|entry| entry.placement_id == placement_id)
    }

    fn active_count_internal(&self) -> usize {
        self.entries.iter().filter(|entry| entry.permit).count()
    }

    fn mounted_count_internal(&self) -> usize {
        self.entries.iter().filter(|entry| entry.mounted).count()
    }

    #[cfg(test)]
    fn active_count(&self) -> usize {
        self.active_count_internal()
    }

    #[cfg(test)]
    fn busy_request_count(&self) -> usize {
        self.entries.iter().map(|entry| entry.busy_requests).sum()
    }

    #[cfg(test)]
    fn mounted_count(&self) -> usize {
        self.mounted_count_internal()
    }
}

#[cfg(all(feature = "hydrate", target_arch = "wasm32"))]
#[derive(Clone)]
struct ExecutionSignals {
    mounted: ArcRwSignal<bool>,
    active: ArcRwSignal<bool>,
    busy: ArcRwSignal<bool>,
}

#[cfg(all(feature = "hydrate", target_arch = "wasm32"))]
enum RuntimeSignalUpdate {
    Activate(ExecutionSignals),
    Deactivate(ExecutionSignals),
    SetBusy(ExecutionSignals, bool),
    Unmount(ExecutionSignals),
}

#[cfg(all(feature = "hydrate", target_arch = "wasm32"))]
struct ExecutionScheduler {
    state: ExecutionSchedulerState,
    signals: HashMap<String, ExecutionSignals>,
}

#[cfg(all(feature = "hydrate", target_arch = "wasm32"))]
impl Default for ExecutionScheduler {
    fn default() -> Self {
        Self {
            state: ExecutionSchedulerState::new(
                DASHBOARD_VIEWER_MAX_CONCURRENT_EXECUTIONS,
                DASHBOARD_VIEWER_MAX_MOUNTED_RENDERERS,
            ),
            signals: HashMap::new(),
        }
    }
}

#[cfg(all(feature = "hydrate", target_arch = "wasm32"))]
impl ExecutionScheduler {
    fn resolve_actions(
        &mut self,
        actions: Vec<ExecutionSchedulerAction>,
    ) -> Vec<RuntimeSignalUpdate> {
        let updates = actions
            .into_iter()
            .filter_map(|action| {
                let signals = self.signals.get(action.placement_id()).cloned()?;
                Some(match action {
                    ExecutionSchedulerAction::Activate(_) => RuntimeSignalUpdate::Activate(signals),
                    ExecutionSchedulerAction::Deactivate(_) => {
                        RuntimeSignalUpdate::Deactivate(signals)
                    }
                    ExecutionSchedulerAction::SetBusy(_, busy) => {
                        RuntimeSignalUpdate::SetBusy(signals, busy)
                    }
                    ExecutionSchedulerAction::Unmount(_) => RuntimeSignalUpdate::Unmount(signals),
                })
            })
            .collect();
        self.signals
            .retain(|placement_id, _| self.state.contains(placement_id));
        updates
    }
}

#[cfg(all(feature = "hydrate", target_arch = "wasm32"))]
thread_local! {
    static EXECUTION_SCHEDULER: RefCell<ExecutionScheduler> = RefCell::new(ExecutionScheduler::default());
}

#[cfg(all(feature = "hydrate", target_arch = "wasm32"))]
struct ViewportObserverRegistration {
    observer: web_sys::IntersectionObserver,
    _callback:
        wasm_bindgen::closure::Closure<dyn FnMut(js_sys::Array, web_sys::IntersectionObserver)>,
}

#[cfg(all(feature = "hydrate", target_arch = "wasm32"))]
thread_local! {
    static NEXT_VIEWPORT_OBSERVER_ID: Cell<u64> = const { Cell::new(1) };
    static VIEWPORT_OBSERVERS: RefCell<HashMap<u64, ViewportObserverRegistration>> =
        RefCell::new(HashMap::new());
}

#[component]
pub fn DashboardViewerContent(dashboard_id: String) -> impl IntoView {
    let (initial_dashboard, bootstrapped) = match dashboard_route_bootstrap() {
        Some(DashboardRouteBootstrap::Viewer { dashboard, .. }) if dashboard.id == dashboard_id => {
            (Some(dashboard), true)
        }
        _ => (None, false),
    };
    let dashboard = RwSignal::new(initial_dashboard);
    let loading = RwSignal::new(cfg!(feature = "hydrate") && !bootstrapped);
    let error = RwSignal::new(None::<String>);

    Effect::new({
        let dashboard_id = dashboard_id.clone();
        move |_| {
            if !bootstrapped {
                load_dashboard(dashboard_id.clone(), dashboard, loading, error);
            }
        }
    });

    view! {
        <section class="route-panel dashboards-page dashboard-viewer">
            {move || if loading.get() {
                view! { <EmptyState title="Loading Dashboard" message="Fetching the saved Dashboard layout."/> }.into_any()
            } else if let Some(message) = error.get() {
                view! { <EmptyState title="Dashboard unavailable" message=message/> }.into_any()
            } else if let Some(loaded) = dashboard.get() {
                let detail_href = format!("/dashboards/{}", loaded.id);
                let edit_href = format!("/dashboards/{}/edit", loaded.id);
                let title = loaded.name.clone();
                let description = loaded.description.clone().unwrap_or_else(|| "Saved Dashboard view".into());
                let can_manage = loaded.can_manage;
                view! {
                    <Breadcrumb>
                        <BreadcrumbItem><BreadcrumbLink href="/dashboards">"Dashboards"</BreadcrumbLink></BreadcrumbItem>
                        <BreadcrumbSeparator/>
                        <BreadcrumbItem><BreadcrumbLink href=detail_href>"Detail"</BreadcrumbLink></BreadcrumbItem>
                        <BreadcrumbSeparator/>
                        <BreadcrumbItem><BreadcrumbPage>"Viewer"</BreadcrumbPage></BreadcrumbItem>
                    </Breadcrumb>
                    <PageHeader title description>
                        {can_manage.then(|| view! {
                            <Button label="Edit Dashboard" href=edit_href/>
                        })}
                    </PageHeader>
                    {if loaded.placements.is_empty() {
                        view! { <EmptyState title="Empty Dashboard" message="This Dashboard has no saved placements."/> }.into_any()
                    } else {
                        let degraded_count = loaded.placements.iter().filter(|placement| {
                            placement.effective_resolution_state()
                                != DashboardPlacementResolutionState::Available
                        }).count();
                        view! {
                            {if degraded_count > 0 {
                                view! {
                                    <aside class="dashboard-viewer__containment-note" role="status">
                                        <div>
                                            <strong>{format!("{degraded_count} placement issue{}", if degraded_count == 1 { "" } else { "s" })}</strong>
                                            <p>"Healthy placements remain available. Saved references and layout are unchanged."</p>
                                        </div>
                                        <a href="/administration/modules/tessara.dashboards#diagnostics">
                                            "Open Dashboard diagnostics"
                                        </a>
                                    </aside>
                                }.into_any()
                            } else {
                                ().into_any()
                            }}
                            <div
                                class="dashboard-saved-grid dashboard-viewer__grid"
                                style="--dashboard-track-size: clamp(48px, 5.8vw, 80px)"
                            >
                                {loaded.placements.into_iter().map(|placement| {
                                    view! { <DashboardViewerPlacement placement/> }
                                }).collect_view()}
                            </div>
                        }.into_any()
                    }}
                }.into_any()
            } else {
                view! { <EmptyState title="Dashboard unavailable" message="No Dashboard payload was returned."/> }.into_any()
            }}
        </section>
    }
}

#[component]
fn DashboardViewerPlacement(placement: DashboardPlacement) -> impl IntoView {
    let style = placement_style(
        placement.grid_row,
        placement.grid_column,
        placement.grid_width,
        placement.grid_height,
    );
    let available = placement.availability == DashboardPlacementAvailability::Available;
    let resolution_state = placement.effective_resolution_state();
    let has_issue = resolution_state != DashboardPlacementResolutionState::Available;
    let order = placement.position + 1;
    let title = if available {
        placement.display_title()
    } else {
        "Unavailable placement".into()
    };
    let component = placement.component.clone();
    let component_kind = component
        .as_ref()
        .and_then(|component| ComponentVersionKind::from_api_kind(&component.component_type));
    let is_table = available && component_kind == Some(ComponentVersionKind::Table);
    let is_chart = available
        && matches!(
            component_kind,
            Some(
                ComponentVersionKind::Bar
                    | ComponentVersionKind::Line
                    | ComponentVersionKind::Pie
                    | ComponentVersionKind::Donut
            )
        );
    let is_stat_card = available && component_kind == Some(ComponentVersionKind::StatCard);
    let has_panel_header = !is_table && !is_chart && !is_stat_card;
    let presentation = if !available {
        "unavailable"
    } else if is_table {
        "table"
    } else if is_chart {
        "chart"
    } else if is_stat_card {
        "stat-card"
    } else {
        "fallback"
    };
    let placement_id = placement.placement_id.clone();
    let title_id = format!("dashboard-placement-title-{placement_id}");
    let fullscreen_dialog_id = format!("dashboard-table-fullscreen-{placement_id}");
    let fullscreen_title = format!("{title} — fullscreen Table");
    let table_presentation = is_table.then(|| {
        ComponentTablePresentation::new()
            .with_title(title_id.clone(), title.clone())
            .with_fullscreen(fullscreen_dialog_id, fullscreen_title)
    });
    let persistence_key = placement_id.clone();
    let scheduler_id = placement_id.clone();
    let viewport_ref = NodeRef::<html::Div>::new();
    let in_view = ArcRwSignal::new(false);
    let mounted = ArcRwSignal::new(false);
    let active = ArcRwSignal::new(false);
    let busy = ArcRwSignal::new(false);
    let target = component.as_ref().and_then(|component| {
        ComponentVersionKind::from_api_kind(&component.component_type).map(|kind| {
            ComponentVersionTarget::new(
                component.component_slug.clone(),
                component.component_version_id.clone(),
                kind,
            )
        })
    });
    let target = StoredValue::new(target);
    let execution_capable = available && target.get_value().is_some();

    Effect::new({
        let in_view = in_view.clone();
        move |_| {
            if execution_capable && let Some(element) = viewport_ref.get() {
                observe_viewport(element.into(), in_view.clone());
            }
        }
    });
    Effect::new({
        let scheduler_id = scheduler_id.clone();
        let in_view = in_view.clone();
        let mounted = mounted.clone();
        let active = active.clone();
        let busy = busy.clone();
        move |_| {
            if !execution_capable {
                return;
            }
            if in_view.get() {
                request_execution(
                    scheduler_id.clone(),
                    mounted.clone(),
                    active.clone(),
                    busy.clone(),
                );
            } else {
                release_execution(&scheduler_id, active.clone());
            }
        }
    });
    on_cleanup({
        let scheduler_id = scheduler_id.clone();
        let active = active.clone();
        move || remove_execution(&scheduler_id, active.clone())
    });
    let request_activity = ComponentRequestActivityCallback::new({
        let scheduler_id = scheduler_id.clone();
        move |activity| record_execution_activity(&scheduler_id, activity)
    });
    let in_view_for_class = in_view.clone();
    let mounted_for_class = mounted.clone();
    let active_for_data = active.clone();
    let busy_for_data = busy.clone();
    let mounted_for_content = mounted.clone();
    let active_for_content = active.clone();
    let article_aria_label = (is_stat_card || is_table).then(|| title.clone());
    let article_aria_labelledby = (!is_stat_card && !is_table).then(|| title_id.clone());
    let header_title = title.clone();
    let header_title_id = title_id.clone();
    let chart_title = title;
    let chart_title_id = title_id;

    view! {
        <article
            class:dashboard-viewer-placement=true
            class:dashboard-viewer-placement--chart=is_chart
            class:dashboard-viewer-placement--stat-card=is_stat_card
            class:is-unavailable=move || !available
            class:is-degraded=has_issue
            style=style
            data-placement-id=placement_id
            data-placement-presentation=presentation
            aria-label=article_aria_label
            aria-labelledby=article_aria_labelledby
        >
            {if has_panel_header {
                view! {
                    <header class="dashboard-viewer-placement__header">
                        <div>
                            <span class="dashboard-placement-card__order">{order}</span>
                            <h2 id=header_title_id>{header_title}</h2>
                        </div>
                    </header>
                }.into_any()
            } else {
                ().into_any()
            }}
            <div
                node_ref=viewport_ref
                class:dashboard-viewer-placement__content=true
                class:is-suspended=move || mounted_for_class.get() && !in_view_for_class.get()
                data-execution-active=move || active_for_data.get().to_string()
                data-execution-busy=move || busy_for_data.get().to_string()
            >
                {if has_issue && available {
                    view! {
                        <div class="dashboard-placement-state-banner" role="status">
                            <span class="status-badge status-badge--warning">
                                {resolution_state.label()}
                            </span>
                            <span>{resolution_state.message()}</span>
                        </div>
                    }.into_any()
                } else {
                    ().into_any()
                }}
                {if is_chart {
                    view! {
                        <h2 id=chart_title_id class="dashboard-viewer-placement__chart-title">
                            {chart_title}
                        </h2>
                    }.into_any()
                } else {
                    ().into_any()
                }}
                {move || {
                    let target = target.get_value();
                    if !available {
                        view! {
                            <div class="dashboard-redacted-placeholder" role="status">
                                <span aria-hidden="true">"⚠"</span>
                                <span class="status-badge status-badge--warning">
                                    {resolution_state.label()}
                                </span>
                                <strong>{resolution_state.label()}</strong>
                                <p>{resolution_state.message()}</p>
                                <small>"The saved footprint and exact reference are preserved."</small>
                                {resolution_state.retryable().then(|| view! {
                                    <a class="button button--secondary" href="">"Retry resolution"</a>
                                })}
                            </div>
                        }.into_any()
                    } else if let Some(target) = target {
                        if mounted_for_content.get() {
                            view! {
                                <ComponentVersionExecutionContent
                                    target
                                    mode=ComponentViewerMode::Embedded
                                    execution_active=Signal::from(active_for_content.clone())
                                    on_request_activity=request_activity.clone()
                                    persistence_key=persistence_key.clone()
                                    table_presentation=table_presentation.clone().unwrap_or_default()
                                />
                            }.into_any()
                        } else {
                            view! {
                                <div class="dashboard-lazy-placeholder" role="status">
                                    <span aria-hidden="true">"◌"</span>
                                    <strong>"Component ready"</strong>
                                    <p>"Execution starts when this placement approaches the viewport."</p>
                                </div>
                            }.into_any()
                        }
                    } else {
                        view! {
                            <PlacementFailure message="This Component kind is not supported by the Dashboard viewer."/>
                        }.into_any()
                    }
                }}
            </div>
        </article>
    }
}

#[cfg(all(feature = "hydrate", target_arch = "wasm32"))]
fn observe_viewport(element: web_sys::HtmlElement, in_view: ArcRwSignal<bool>) {
    use wasm_bindgen::{JsCast, closure::Closure};
    use web_sys::{IntersectionObserver, IntersectionObserverEntry, IntersectionObserverInit};

    let in_view_for_callback = in_view.clone();
    let callback = Closure::<dyn FnMut(js_sys::Array, IntersectionObserver)>::new(
        move |entries: js_sys::Array, _observer: IntersectionObserver| {
            let Some(entry) = entries.get(0).dyn_into::<IntersectionObserverEntry>().ok() else {
                return;
            };
            in_view_for_callback.set(entry.is_intersecting());
        },
    );
    let options = IntersectionObserverInit::new();
    options.set_root_margin("240px 0px");
    let Ok(observer) =
        IntersectionObserver::new_with_options(callback.as_ref().unchecked_ref(), &options)
    else {
        in_view.set(true);
        return;
    };
    observer.observe(element.as_ref());
    let registration_id = NEXT_VIEWPORT_OBSERVER_ID.with(|next_id| {
        let registration_id = next_id.get();
        next_id.set(registration_id.wrapping_add(1).max(1));
        registration_id
    });
    VIEWPORT_OBSERVERS.with(|observers| {
        observers.borrow_mut().insert(
            registration_id,
            ViewportObserverRegistration {
                observer,
                _callback: callback,
            },
        );
    });
    // The cleanup captures only a numeric id, satisfying Leptos' Send + Sync
    // bound while the browser observer and callback remain thread-local.
    on_cleanup(move || remove_viewport_observer(registration_id));
}

#[cfg(not(all(feature = "hydrate", target_arch = "wasm32")))]
fn observe_viewport(_: web_sys::HtmlElement, _: ArcRwSignal<bool>) {}

#[cfg(all(feature = "hydrate", target_arch = "wasm32"))]
fn remove_viewport_observer(registration_id: u64) {
    VIEWPORT_OBSERVERS.with(|observers| {
        if let Some(registration) = observers.borrow_mut().remove(&registration_id) {
            registration.observer.disconnect();
        }
    });
}

#[cfg(all(feature = "hydrate", target_arch = "wasm32"))]
fn request_execution(
    placement_id: String,
    mounted: ArcRwSignal<bool>,
    active: ArcRwSignal<bool>,
    busy: ArcRwSignal<bool>,
) {
    let updates = EXECUTION_SCHEDULER.with(|scheduler| {
        let mut scheduler = scheduler.borrow_mut();
        scheduler.signals.insert(
            placement_id.clone(),
            ExecutionSignals {
                mounted,
                active,
                busy,
            },
        );
        let actions = scheduler.state.enter_viewport(placement_id);
        scheduler.resolve_actions(actions)
    });
    apply_runtime_updates(updates);
}

#[cfg(not(all(feature = "hydrate", target_arch = "wasm32")))]
fn request_execution(_: String, _: ArcRwSignal<bool>, _: ArcRwSignal<bool>, _: ArcRwSignal<bool>) {}

#[cfg(all(feature = "hydrate", target_arch = "wasm32"))]
fn release_execution(placement_id: &str, active: ArcRwSignal<bool>) {
    let updates = EXECUTION_SCHEDULER.with(|scheduler| {
        let mut scheduler = scheduler.borrow_mut();
        let actions = scheduler.state.leave_viewport(placement_id);
        scheduler.resolve_actions(actions)
    });
    // Keep this local signal defensive when cleanup races registration.
    active.set(false);
    apply_runtime_updates(updates);
}

#[cfg(not(all(feature = "hydrate", target_arch = "wasm32")))]
fn release_execution(_: &str, _: ArcRwSignal<bool>) {}

#[cfg(all(feature = "hydrate", target_arch = "wasm32"))]
fn remove_execution(placement_id: &str, active: ArcRwSignal<bool>) {
    let updates = EXECUTION_SCHEDULER.with(|scheduler| {
        let mut scheduler = scheduler.borrow_mut();
        let actions = scheduler.state.remove(placement_id);
        scheduler.resolve_actions(actions)
    });
    active.set(false);
    apply_runtime_updates(updates);
}

#[cfg(not(all(feature = "hydrate", target_arch = "wasm32")))]
fn remove_execution(_: &str, _: ArcRwSignal<bool>) {}

#[cfg(all(feature = "hydrate", target_arch = "wasm32"))]
fn record_execution_activity(placement_id: &str, activity: ComponentRequestActivity) {
    let updates = EXECUTION_SCHEDULER.with(|scheduler| {
        let mut scheduler = scheduler.borrow_mut();
        let actions = scheduler.state.record_activity(placement_id, activity);
        scheduler.resolve_actions(actions)
    });
    apply_runtime_updates(updates);
}

#[cfg(not(all(feature = "hydrate", target_arch = "wasm32")))]
fn record_execution_activity(_: &str, _: ComponentRequestActivity) {}

#[cfg(all(feature = "hydrate", target_arch = "wasm32"))]
fn apply_runtime_updates(updates: Vec<RuntimeSignalUpdate>) {
    // Apply signals after dropping the scheduler's RefCell borrow. Activating
    // a renderer can synchronously report `Started` from its request effect.
    for update in updates {
        match update {
            RuntimeSignalUpdate::Activate(signals) => {
                signals.active.set(true);
                // A first mount must observe its permit immediately; mounting
                // while inactive would emit a redundant `Requested` intent.
                // Do not notify the placement's conditional view when the
                // renderer is already mounted. Recreating it would discard
                // owner-local UI state such as an open Table fullscreen dialog.
                if !signals.mounted.get_untracked() {
                    signals.mounted.set(true);
                }
            }
            RuntimeSignalUpdate::Deactivate(signals) => signals.active.set(false),
            RuntimeSignalUpdate::SetBusy(signals, busy) => signals.busy.set(busy),
            RuntimeSignalUpdate::Unmount(signals) => {
                signals.active.set(false);
                signals.busy.set(false);
                signals.mounted.set(false);
            }
        }
    }
}

#[component]
fn PlacementFailure(message: &'static str) -> impl IntoView {
    view! {
        <div class="dashboard-placement-failure" role="alert">
            <strong>"Component could not render"</strong>
            <p>{message}</p>
        </div>
    }
}

#[cfg(feature = "hydrate")]
fn load_dashboard(
    dashboard_id: String,
    dashboard: RwSignal<Option<Dashboard>>,
    loading: RwSignal<bool>,
    error: RwSignal<Option<String>>,
) {
    leptos::task::spawn_local(async move {
        loading.set(true);
        error.set(None);
        match crate::api::fetch_dashboard(&dashboard_id).await {
            Ok(payload) => dashboard.set(Some(payload)),
            Err(message) => error.set(Some(message)),
        }
        loading.set(false);
    });
}

#[cfg(not(feature = "hydrate"))]
fn load_dashboard(
    _: String,
    _: RwSignal<Option<Dashboard>>,
    _: RwSignal<bool>,
    _: RwSignal<Option<String>>,
) {
}

#[cfg(test)]
mod tests {
    use super::{
        ComponentRequestActivity, DASHBOARD_VIEWER_MAX_CONCURRENT_EXECUTIONS,
        DASHBOARD_VIEWER_MAX_MOUNTED_RENDERERS, ExecutionSchedulerAction, ExecutionSchedulerState,
    };
    #[cfg(feature = "ssr")]
    use super::{DashboardViewerContent, DashboardViewerPlacement};
    #[cfg(feature = "ssr")]
    use crate::DashboardRouteBootstrap;
    #[cfg(feature = "ssr")]
    use crate::types::{
        Dashboard, DashboardComponentVersion, DashboardPlacement, DashboardPlacementAvailability,
        DashboardPlacementResolutionState, SessionAccount,
    };
    #[cfg(feature = "ssr")]
    use leptos::prelude::*;

    #[cfg(feature = "ssr")]
    fn dashboard(can_manage: bool, placements: Vec<DashboardPlacement>) -> Dashboard {
        Dashboard {
            id: "dashboard-1".into(),
            name: "Delivery health".into(),
            description: Some("Saved operational view".into()),
            visibility_nodes: Vec::new(),
            placement_count: i64::try_from(placements.len()).expect("fixture length"),
            can_manage,
            placements,
        }
    }

    #[cfg(feature = "ssr")]
    fn table_placement() -> DashboardPlacement {
        component_placement("table", "Program table")
    }

    #[cfg(feature = "ssr")]
    fn component_placement(kind: &str, title: &str) -> DashboardPlacement {
        DashboardPlacement {
            placement_id: "placement-1".into(),
            position: 0,
            grid_row: 1,
            grid_column: 1,
            grid_width: 6,
            grid_height: 4,
            availability: DashboardPlacementAvailability::Available,
            resolution_state: Some(DashboardPlacementResolutionState::Available),
            config_state: None,
            title: Some(title.into()),
            component: Some(DashboardComponentVersion {
                component_version_id: "version-1".into(),
                component_id: "component-1".into(),
                component_name: title.into(),
                component_slug: format!("demo-{kind}"),
                component_type: kind.into(),
                version_number: 1,
                version_label: "Published".into(),
                version_status: "published".into(),
            }),
            allowed_operations: None,
        }
    }

    #[test]
    fn execution_ceiling_is_named_and_below_storage_capacity() {
        const {
            assert!(DASHBOARD_VIEWER_MAX_CONCURRENT_EXECUTIONS > 0);
            assert!(DASHBOARD_VIEWER_MAX_CONCURRENT_EXECUTIONS < 240);
            assert!(
                DASHBOARD_VIEWER_MAX_MOUNTED_RENDERERS
                    >= DASHBOARD_VIEWER_MAX_CONCURRENT_EXECUTIONS
            );
            assert!(DASHBOARD_VIEWER_MAX_MOUNTED_RENDERERS < 240);
        }
    }

    #[cfg(feature = "ssr")]
    #[test]
    fn viewer_header_replaces_the_count_with_a_capability_gated_edit_action() {
        let _ = any_spawner::Executor::init_futures_executor();
        for (can_manage, should_show_edit) in [(false, false), (true, true)] {
            let html = Owner::new().with(|| {
                provide_context(DashboardRouteBootstrap::viewer(
                    SessionAccount {
                        capabilities: if can_manage {
                            vec!["dashboards:manage".into()]
                        } else {
                            Vec::new()
                        },
                    },
                    dashboard(can_manage, Vec::new()),
                ));
                view! { <DashboardViewerContent dashboard_id="dashboard-1".into()/> }.to_html()
            });

            assert_eq!(html.contains("Edit Dashboard"), should_show_edit);
            assert!(!html.contains("0 placements"));
        }
    }

    #[cfg(feature = "ssr")]
    #[test]
    fn table_placements_delegate_title_and_fullscreen_chrome_to_the_standard_renderer() {
        let _ = any_spawner::Executor::init_futures_executor();
        let html = Owner::new()
            .with(|| view! { <DashboardViewerPlacement placement=table_placement()/> }.to_html());

        assert!(html.contains("data-placement-presentation=\"table\""));
        assert!(html.contains("aria-label=\"Program table\""));
        assert!(!html.contains("dashboard-viewer-placement__header"));
        assert!(!html.contains("dashboard-placement-card__order"));
        assert!(!html.contains("dashboard-viewer-placement__fullscreen"));
        assert!(!html.contains("6 × 4"));
        assert!(!html.contains("dashboard-placement-card__size"));
    }

    #[cfg(feature = "ssr")]
    #[test]
    fn chart_placements_put_accessible_titles_inside_borderless_content() {
        let _ = any_spawner::Executor::init_futures_executor();
        for kind in ["bar", "line", "pie", "donut"] {
            let title = format!("{kind} delivery summary");
            let html = Owner::new().with(|| {
                view! {
                    <DashboardViewerPlacement placement=component_placement(kind, &title)/>
                }
                .to_html()
            });

            assert!(html.contains("data-placement-presentation=\"chart\""));
            assert!(html.contains("dashboard-viewer-placement--chart"));
            assert!(html.contains("aria-labelledby=\"dashboard-placement-title-placement-1\""));
            assert!(!html.contains("dashboard-viewer-placement__header"));
            assert!(!html.contains("dashboard-placement-card__order"));
            let content_index = html
                .find("dashboard-viewer-placement__content")
                .expect("chart content");
            let title_index = html
                .find("dashboard-viewer-placement__chart-title")
                .expect("chart title");
            assert!(
                title_index > content_index,
                "title must live inside chart content: {html}"
            );
            assert!(html.contains(&title));
        }
    }

    #[cfg(feature = "ssr")]
    #[test]
    fn stat_card_placements_expose_intrinsic_content_without_outer_chrome() {
        let _ = any_spawner::Executor::init_futures_executor();
        let html = Owner::new().with(|| {
            view! {
                <DashboardViewerPlacement
                    placement=component_placement("stat_card", "Total participants")
                />
            }
            .to_html()
        });

        assert!(html.contains("data-placement-presentation=\"stat-card\""));
        assert!(html.contains("dashboard-viewer-placement--stat-card"));
        assert!(html.contains("aria-label=\"Total participants\""));
        assert!(!html.contains("dashboard-viewer-placement__header"));
        assert!(!html.contains("dashboard-placement-card__order"));
        assert!(!html.contains("dashboard-viewer-placement__chart-title"));
    }

    #[cfg(feature = "ssr")]
    #[test]
    fn unavailable_placements_keep_redacted_panel_chrome_without_metadata_leakage() {
        let _ = any_spawner::Executor::init_futures_executor();
        let mut placement = component_placement("bar", "Confidential performance chart");
        placement.availability = DashboardPlacementAvailability::Unavailable;
        placement.resolution_state = Some(DashboardPlacementResolutionState::Restricted);
        placement
            .component
            .as_mut()
            .expect("fixture component")
            .component_slug = "confidential-performance-chart".into();
        let html = Owner::new().with(|| view! { <DashboardViewerPlacement placement/> }.to_html());

        assert!(html.contains("data-placement-presentation=\"unavailable\""));
        assert!(html.contains("dashboard-viewer-placement__header"));
        assert!(html.contains("dashboard-redacted-placeholder"));
        assert!(html.contains("Unavailable placement"));
        assert!(!html.contains("dashboard-viewer-placement--chart"));
        assert!(!html.contains("dashboard-viewer-placement__chart-title"));
        assert!(!html.contains("Confidential performance chart"));
        assert!(!html.contains("confidential-performance-chart"));
    }

    #[cfg(feature = "ssr")]
    #[test]
    fn provider_outage_is_contained_with_authorized_copy_and_retry() {
        let _ = any_spawner::Executor::init_futures_executor();
        let mut placement = component_placement("table", "Program table");
        placement.availability = DashboardPlacementAvailability::Unavailable;
        placement.resolution_state = Some(DashboardPlacementResolutionState::ProviderUnavailable);
        let html = Owner::new().with(|| view! { <DashboardViewerPlacement placement/> }.to_html());

        assert!(html.contains("Provider unavailable"));
        assert!(html.contains("Dashboard remains available"));
        assert!(html.contains("Retry resolution"));
        assert!(html.contains("exact reference are preserved"));
    }

    #[test]
    fn twelve_visible_placements_rotate_six_request_permits_without_starvation() {
        let mut scheduler = ExecutionSchedulerState::new(6, 48);
        for index in 1..=12 {
            let actions = scheduler.enter_viewport(format!("placement-{index}"));
            assert_eq!(actions.len(), usize::from(index <= 6));
            assert!(scheduler.active_count() <= 6);
        }
        for index in 1..=6 {
            scheduler.record_activity(
                &format!("placement-{index}"),
                ComponentRequestActivity::Started,
            );
        }
        assert_eq!(scheduler.busy_request_count(), 6);

        for index in 1..=6 {
            let actions = scheduler.record_activity(
                &format!("placement-{index}"),
                ComponentRequestActivity::Settled,
            );
            assert!(actions.iter().any(|action| matches!(
                action,
                ExecutionSchedulerAction::Activate(placement_id)
                    if placement_id == &format!("placement-{}", index + 6)
            )));
            assert!(scheduler.active_count() <= 6);
            assert!(scheduler.busy_request_count() <= 6);
        }
        for index in 7..=12 {
            scheduler.record_activity(
                &format!("placement-{index}"),
                ComponentRequestActivity::Started,
            );
            scheduler.record_activity(
                &format!("placement-{index}"),
                ComponentRequestActivity::Settled,
            );
        }
        assert_eq!(scheduler.active_count(), 0);
        assert_eq!(scheduler.busy_request_count(), 0);
        assert_eq!(scheduler.mounted_count(), 12);
    }

    #[test]
    fn offscreen_busy_renderer_deactivates_but_holds_slot_until_settled() {
        let mut scheduler = ExecutionSchedulerState::new(1, 4);
        scheduler.enter_viewport("first".into());
        assert_eq!(
            scheduler.record_activity("first", ComponentRequestActivity::Started),
            vec![ExecutionSchedulerAction::SetBusy("first".into(), true)]
        );
        scheduler.enter_viewport("waiting".into());

        assert_eq!(
            scheduler.leave_viewport("first"),
            vec![ExecutionSchedulerAction::Deactivate("first".into())]
        );
        assert_eq!(scheduler.active_count(), 1);
        assert!(scheduler.contains("waiting"));

        assert_eq!(
            scheduler.record_activity("first", ComponentRequestActivity::Settled),
            vec![
                ExecutionSchedulerAction::SetBusy("first".into(), false),
                ExecutionSchedulerAction::Deactivate("first".into()),
                ExecutionSchedulerAction::Activate("waiting".into()),
            ]
        );
        assert_eq!(scheduler.active_count(), 1);
        assert!(scheduler.contains("first"));
        assert!(scheduler.contains("waiting"));
    }

    #[test]
    fn settled_table_control_request_reacquires_a_permit() {
        let mut scheduler = ExecutionSchedulerState::new(1, 4);
        scheduler.enter_viewport("table".into());
        scheduler.record_activity("table", ComponentRequestActivity::Started);
        scheduler.record_activity("table", ComponentRequestActivity::Settled);
        assert_eq!(scheduler.active_count(), 0);
        assert_eq!(scheduler.mounted_count(), 1);

        assert_eq!(
            scheduler.record_activity("table", ComponentRequestActivity::Requested),
            vec![ExecutionSchedulerAction::Activate("table".into())]
        );
        assert_eq!(scheduler.active_count(), 1);
    }

    #[test]
    fn busy_cleanup_waits_for_settlement_before_unmounting() {
        let mut scheduler = ExecutionSchedulerState::new(1, 4);
        scheduler.enter_viewport("first".into());
        scheduler.record_activity("first", ComponentRequestActivity::Started);
        assert_eq!(
            scheduler.remove("first"),
            vec![ExecutionSchedulerAction::Deactivate("first".into())]
        );
        assert!(scheduler.contains("first"));
        assert_eq!(scheduler.active_count(), 1);
        assert_eq!(
            scheduler.record_activity("first", ComponentRequestActivity::Settled),
            vec![
                ExecutionSchedulerAction::SetBusy("first".into(), false),
                ExecutionSchedulerAction::Deactivate("first".into()),
                ExecutionSchedulerAction::Unmount("first".into()),
            ]
        );
        assert!(!scheduler.contains("first"));
        assert_eq!(scheduler.active_count(), 0);
    }

    #[test]
    fn offscreen_settled_renderer_is_evicted_before_mounted_cap_is_exceeded() {
        let mut scheduler = ExecutionSchedulerState::new(1, 2);
        for placement_id in ["first", "second"] {
            scheduler.enter_viewport(placement_id.into());
            scheduler.record_activity(placement_id, ComponentRequestActivity::Started);
            scheduler.record_activity(placement_id, ComponentRequestActivity::Settled);
            scheduler.leave_viewport(placement_id);
        }
        assert_eq!(scheduler.mounted_count(), 2);

        assert_eq!(
            scheduler.enter_viewport("third".into()),
            vec![
                ExecutionSchedulerAction::Unmount("first".into()),
                ExecutionSchedulerAction::Activate("third".into()),
            ]
        );
        assert_eq!(scheduler.mounted_count(), 2);
    }
}
