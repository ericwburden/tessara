//! Controlled, server-backed execution of exact Component versions.
//!
//! The shared web UI crate remains presentation-only. Callers provide one
//! already-authorized public Component reference, exact version id, and kind;
//! this module owns request, render, and per-Table paging state.

use std::{collections::BTreeMap, sync::Arc};

#[cfg(all(feature = "hydrate", target_arch = "wasm32"))]
use std::cell::RefCell;
#[cfg(any(test, all(feature = "hydrate", target_arch = "wasm32")))]
use std::collections::VecDeque;

use icons::{ArrowDown, ArrowUp, ChevronLeft, ChevronRight, Fullscreen, ListFilter, RotateCcw};
use leptos::prelude::*;
use tessara_module_ui::{
    EmptyState, FullscreenDialog, TableColumnOption, TableColumnSelector, TablePaginationBar,
    TablePopoverController, TableSearch,
};

#[cfg(feature = "hydrate")]
use crate::api;
use crate::request::{
    BoundedRetry, RequestCompletion, RequestLifecycle, RequestLifecycleDecision,
    notify_request_activity,
};
#[cfg(feature = "hydrate")]
use crate::request::{RequestActivityGuard, schedule_bounded_retry};
use crate::types::{ComponentTable, ComponentTableColumn};
use crate::visual::ComponentVisualViewer;

/// The six executable Component kinds supported by reader surfaces.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ComponentVersionKind {
    Table,
    Bar,
    Line,
    Pie,
    Donut,
    StatCard,
}

impl ComponentVersionKind {
    /// Adapts the API's stable snake-case Component kind vocabulary.
    pub fn from_api_kind(value: &str) -> Option<Self> {
        match value {
            "table" => Some(Self::Table),
            "bar" => Some(Self::Bar),
            "line" => Some(Self::Line),
            "pie" => Some(Self::Pie),
            "donut" => Some(Self::Donut),
            "stat_card" => Some(Self::StatCard),
            _ => None,
        }
    }

    /// Returns the stable API vocabulary for this kind.
    pub const fn as_api_value(self) -> &'static str {
        match self {
            Self::Table => "table",
            Self::Bar => "bar",
            Self::Line => "line",
            Self::Pie => "pie",
            Self::Donut => "donut",
            Self::StatCard => "stat_card",
        }
    }

    /// Returns the canonical execution endpoint segment for this kind.
    pub const fn endpoint_segment(self) -> &'static str {
        match self {
            Self::StatCard => "stat-card",
            other => other.as_api_value(),
        }
    }

    pub(crate) const fn label(self) -> &'static str {
        match self {
            Self::Table => "Table",
            Self::Bar => "Bar",
            Self::Line => "Line",
            Self::Pie => "Pie",
            Self::Donut => "Donut",
            Self::StatCard => "Stat Card",
        }
    }
}

/// Identifies one exact published or superseded ComponentVersion endpoint.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ComponentVersionTarget {
    component_ref: String,
    component_version_id: String,
    kind: ComponentVersionKind,
    execution_route: ComponentExecutionRoute,
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum ComponentExecutionRoute {
    Direct,
    Mediated { endpoint_path: String },
}

impl ComponentVersionTarget {
    /// Creates a route-free execution target pinned to one stable version id.
    pub fn new(
        component_ref: impl Into<String>,
        component_version_id: impl Into<String>,
        kind: ComponentVersionKind,
    ) -> Self {
        Self {
            component_ref: component_ref.into(),
            component_version_id: component_version_id.into(),
            kind,
            execution_route: ComponentExecutionRoute::Direct,
        }
    }

    /// Creates a target whose functional owner supplies the exact execution
    /// endpoint. Component identifiers remain presentation metadata and need
    /// not be included in that endpoint.
    pub fn mediated(
        component_ref: impl Into<String>,
        component_version_id: impl Into<String>,
        kind: ComponentVersionKind,
        endpoint_path: impl Into<String>,
    ) -> Self {
        Self {
            component_ref: component_ref.into(),
            component_version_id: component_version_id.into(),
            kind,
            execution_route: ComponentExecutionRoute::Mediated {
                endpoint_path: endpoint_path.into(),
            },
        }
    }

    /// Returns the target kind without exposing mutable execution state.
    pub const fn kind(&self) -> ComponentVersionKind {
        self.kind
    }

    /// Returns the public Component slug or id used by the execution endpoint.
    pub fn component_ref(&self) -> &str {
        &self.component_ref
    }

    /// Returns the exact stable ComponentVersion id used by the endpoint.
    pub fn component_version_id(&self) -> &str {
        &self.component_version_id
    }

    #[cfg(any(feature = "hydrate", test))]
    pub(crate) fn endpoint_path(&self) -> String {
        match &self.execution_route {
            ComponentExecutionRoute::Direct => format!(
                "/api/components/{}/versions/{}/{}",
                self.component_ref,
                self.component_version_id,
                self.kind.endpoint_segment()
            ),
            ComponentExecutionRoute::Mediated { endpoint_path } => endpoint_path.clone(),
        }
    }
}

/// Controls presentation density without introducing route or feature concepts.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ComponentViewerMode {
    Full,
    #[default]
    Embedded,
}

/// Optional route-free presentation for a rendered Component Table.
///
/// The renderer owns fullscreen state so its inline and dialog presentations
/// always share one query, paging, filter, sort, and visible-column state
/// machine. Visual Component kinds intentionally ignore this configuration.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ComponentTablePresentation {
    title: Option<ComponentTableTitle>,
    fullscreen: Option<ComponentTableFullscreen>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct ComponentTableTitle {
    id: String,
    text: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct ComponentTableFullscreen {
    dialog_id: String,
    dialog_title: String,
}

impl ComponentTablePresentation {
    /// Creates a Table presentation with no title or fullscreen action.
    pub const fn new() -> Self {
        Self {
            title: None,
            fullscreen: None,
        }
    }

    /// Adds an optional compact-toolbar title with a caller-stable DOM id.
    #[must_use]
    pub fn with_title(mut self, id: impl Into<String>, text: impl Into<String>) -> Self {
        self.title = Some(ComponentTableTitle {
            id: id.into(),
            text: text.into(),
        });
        self
    }

    /// Adds an internally owned fullscreen action and accessible dialog.
    #[must_use]
    pub fn with_fullscreen(
        mut self,
        dialog_id: impl Into<String>,
        dialog_title: impl Into<String>,
    ) -> Self {
        self.fullscreen = Some(ComponentTableFullscreen {
            dialog_id: dialog_id.into(),
            dialog_title: dialog_title.into(),
        });
        self
    }
}

/// Reports the lifecycle of one exact-version HTTP request.
///
/// Dashboard callers use this route-free signal to keep a renderer's
/// concurrency slot until its in-flight work has actually settled. A single
/// renderer serializes requests, so every `Started` notification has exactly
/// one matching `Settled` notification before another request can start.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ComponentRequestActivity {
    Requested,
    Started,
    Settled,
}

/// Owner-independent request lifecycle callback used by embedding schedulers.
///
/// Unlike an arena-backed reactive callback, this value remains valid if the
/// renderer's Leptos owner is disposed while its fetch future is still
/// settling.
#[derive(Clone)]
pub struct ComponentRequestActivityCallback {
    callback: Arc<dyn Fn(ComponentRequestActivity) + Send + Sync>,
}

impl ComponentRequestActivityCallback {
    pub fn new(callback: impl Fn(ComponentRequestActivity) + Send + Sync + 'static) -> Self {
        Self {
            callback: Arc::new(callback),
        }
    }

    pub(crate) fn run(&self, activity: ComponentRequestActivity) {
        (self.callback)(activity);
    }
}

impl ComponentViewerMode {
    const fn class_name(self) -> &'static str {
        match self {
            Self::Full => "component-version-execution component-version-execution--full",
            Self::Embedded => "component-version-execution component-version-execution--embedded",
        }
    }

    const fn as_data_value(self) -> &'static str {
        match self {
            Self::Full => "full",
            Self::Embedded => "embedded",
        }
    }
}

/// Fetches and renders one exact Component version with isolated local state.
#[component]
pub fn ComponentVersionExecutionContent(
    target: ComponentVersionTarget,
    #[prop(default = ComponentViewerMode::Embedded)] mode: ComponentViewerMode,
    /// Enables request execution while preserving the mounted renderer and
    /// all of its local Table/view state when disabled.
    #[prop(optional)]
    execution_active: Option<Signal<bool>>,
    /// Notifies a parent scheduler when an actual HTTP request starts and
    /// settles. No notification is emitted while execution is inactive.
    #[prop(optional)]
    on_request_activity: Option<ComponentRequestActivityCallback>,
    /// Opaque caller-owned key used to restore private Table query/page state
    /// after a bounded parent unmounts and later remounts this renderer.
    #[prop(optional)]
    persistence_key: Option<String>,
    /// Optional title and internally owned fullscreen behavior for Tables.
    /// Visual Component kinds intentionally ignore this configuration.
    #[prop(optional)]
    table_presentation: Option<ComponentTablePresentation>,
) -> impl IntoView {
    let kind = target.kind;
    let kind_value = kind.as_api_value();
    let execution_active = execution_active.unwrap_or_else(|| Signal::derive(|| true));
    view! {
        <div
            class=mode.class_name()
            data-component-kind=kind_value
            data-viewer-mode=mode.as_data_value()
        >
            {match kind {
                ComponentVersionKind::Table => {
                    view! {
                        <ComponentTableViewer
                            target
                            mode
                            execution_active
                            on_request_activity
                            persistence_key
                            table_presentation
                        />
                    }
                    .into_any()
                }
                _ => {
                    view! {
                        <ComponentVisualViewer
                            target
                            execution_active
                            on_request_activity
                        />
                    }
                    .into_any()
                }
            }}
        </div>
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct ComponentTableSort {
    field_key: String,
    direction: &'static str,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct ComponentTableFilter {
    operator: &'static str,
    value: String,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
struct ComponentTableViewState {
    search: String,
    requested_page_size: Option<usize>,
    cursor: Option<String>,
    previous_cursors: Vec<Option<String>>,
    sort: Option<ComponentTableSort>,
    filters: BTreeMap<String, ComponentTableFilter>,
    visible_columns: Option<Vec<String>>,
}

impl ComponentTableViewState {
    fn for_mode(mode: ComponentViewerMode) -> Self {
        Self {
            requested_page_size: (mode == ComponentViewerMode::Embedded).then_some(10),
            ..Self::default()
        }
    }

    fn apply_mode_default(&mut self, mode: ComponentViewerMode) {
        if mode == ComponentViewerMode::Embedded && self.requested_page_size.is_none() {
            self.requested_page_size = Some(10);
        }
    }

    fn reset_paging(&mut self) {
        self.cursor = None;
        self.previous_cursors.clear();
    }

    fn move_next(&mut self, next_cursor: String) {
        self.previous_cursors.push(self.cursor.clone());
        self.cursor = Some(next_cursor);
    }

    fn move_previous(&mut self) {
        if let Some(previous) = self.previous_cursors.pop() {
            self.cursor = previous;
        }
    }

    fn page_number(&self) -> usize {
        self.previous_cursors.len() + 1
    }

    fn query(&self) -> String {
        let mut params = Vec::new();
        push_query_param(&mut params, "q", &self.search);
        if let Some(page_size) = self.requested_page_size {
            push_query_param(&mut params, "page_size", &page_size.to_string());
        }
        if let Some(cursor) = &self.cursor {
            push_query_param(&mut params, "cursor", cursor);
        }
        if let Some(sort) = &self.sort {
            push_query_param(
                &mut params,
                "sort",
                &format!("{}:{}", sort.field_key, sort.direction),
            );
        }
        for (field_key, filter) in &self.filters {
            push_query_param(
                &mut params,
                &format!("filter[{field_key}][operator]"),
                filter.operator,
            );
            push_query_param(
                &mut params,
                &format!("filter[{field_key}][value]"),
                &filter.value,
            );
        }
        if let Some(visible_columns) = &self.visible_columns {
            push_query_param(&mut params, "visible_columns", &visible_columns.join(","));
        }
        params.join("&")
    }
}

#[cfg(all(feature = "hydrate", target_arch = "wasm32"))]
const PERSISTED_TABLE_STATE_LIMIT: usize = 240;

#[cfg(any(test, all(feature = "hydrate", target_arch = "wasm32")))]
#[derive(Default)]
struct PersistedTableStateCache {
    states: BTreeMap<String, ComponentTableViewState>,
    order: VecDeque<String>,
}

#[cfg(any(test, all(feature = "hydrate", target_arch = "wasm32")))]
impl PersistedTableStateCache {
    fn load(&mut self, key: &str) -> ComponentTableViewState {
        let state = self.states.get(key).cloned().unwrap_or_default();
        self.order.retain(|cached_key| cached_key != key);
        self.order.push_back(key.to_string());
        state
    }

    fn persist(&mut self, key: &str, state: ComponentTableViewState, limit: usize) {
        self.states.insert(key.to_string(), state);
        self.order.retain(|cached_key| cached_key != key);
        self.order.push_back(key.to_string());
        while self.order.len() > limit {
            if let Some(expired) = self.order.pop_front() {
                self.states.remove(&expired);
            }
        }
    }
}

#[cfg(all(feature = "hydrate", target_arch = "wasm32"))]
thread_local! {
    static PERSISTED_TABLE_STATES: RefCell<PersistedTableStateCache> =
        RefCell::new(PersistedTableStateCache::default());
}

#[cfg(all(feature = "hydrate", target_arch = "wasm32"))]
fn load_persisted_table_state(key: Option<&str>) -> ComponentTableViewState {
    let Some(key) = key else {
        return ComponentTableViewState::default();
    };
    PERSISTED_TABLE_STATES.with(|cache| cache.borrow_mut().load(key))
}

#[cfg(not(all(feature = "hydrate", target_arch = "wasm32")))]
fn load_persisted_table_state(_: Option<&str>) -> ComponentTableViewState {
    ComponentTableViewState::default()
}

#[cfg(all(feature = "hydrate", target_arch = "wasm32"))]
fn persist_table_state(key: Option<&str>, state: ComponentTableViewState) {
    let Some(key) = key else { return };
    PERSISTED_TABLE_STATES.with(|cache| {
        cache
            .borrow_mut()
            .persist(key, state, PERSISTED_TABLE_STATE_LIMIT);
    });
}

#[cfg(not(all(feature = "hydrate", target_arch = "wasm32")))]
fn persist_table_state(_: Option<&str>, _: ComponentTableViewState) {}

/// A reusable Table renderer whose controls always re-query the Component API.
///
/// The renderer intentionally exposes no query-state props or callbacks. This
/// keeps cursor history, server filters, sort, search, page size, and visible
/// column selection consistent in standalone and future Dashboard placements.
#[component]
fn ComponentTableViewer(
    target: ComponentVersionTarget,
    mode: ComponentViewerMode,
    execution_active: Signal<bool>,
    on_request_activity: Option<ComponentRequestActivityCallback>,
    persistence_key: Option<String>,
    table_presentation: Option<ComponentTablePresentation>,
) -> impl IntoView {
    let persistence_key = persistence_key.map(|key| {
        format!(
            "{}:{}:{}",
            key,
            target.component_ref(),
            target.component_version_id()
        )
    });
    let mut initial_state = load_persisted_table_state(persistence_key.as_deref());
    initial_state.apply_mode_default(mode);
    let state = RwSignal::new(initial_state);
    on_cleanup({
        let persistence_key = persistence_key.clone();
        move || persist_table_state(persistence_key.as_deref(), state.get_untracked())
    });
    let table = ArcRwSignal::new(None::<ComponentTable>);
    let known_columns = ArcRwSignal::new(Vec::<ComponentTableColumn>::new());
    let loading = ArcRwSignal::new(true);
    let error = ArcRwSignal::new(None::<String>);
    let active_request_id = ArcRwSignal::new(0_u64);
    let request_lifecycle = ArcRwSignal::new(RequestLifecycle::<(String, u64)>::default());
    let retry = ArcRwSignal::new(BoundedRetry::<String>::default());
    let fullscreen_open = RwSignal::new(false);

    Effect::new({
        let target = target.clone();
        let table = table.clone();
        let known_columns = known_columns.clone();
        let loading = loading.clone();
        let error = error.clone();
        let active_request_id = active_request_id.clone();
        let request_lifecycle = request_lifecycle.clone();
        let retry = retry.clone();
        let on_request_activity = on_request_activity.clone();
        move |_| {
            let query = state.get().query();
            // Track settlement so a query changed while the prior request was
            // busy starts exactly once after that request completes.
            let _request_state = request_lifecycle.get();
            let retry_state = retry.get();
            let retry_attempt = retry_state.attempt_for(&query);
            let retry_epoch = retry_state.epoch();
            let decision = request_lifecycle
                .try_maybe_update(|lifecycle| {
                    let (changed, decision) =
                        lifecycle.prepare(execution_active.get(), (query.clone(), retry_epoch));
                    (changed, decision)
                })
                .unwrap_or(RequestLifecycleDecision::None);
            match decision {
                RequestLifecycleDecision::None => return,
                RequestLifecycleDecision::RequestActivation => {
                    notify_request_activity(
                        on_request_activity.as_ref(),
                        ComponentRequestActivity::Requested,
                    );
                    return;
                }
                RequestLifecycleDecision::Start => {}
            }
            let request_id = active_request_id.get_untracked().wrapping_add(1);
            active_request_id.set(request_id);
            if retry_attempt == 0 {
                loading.set(true);
                error.set(None);
            }
            load_component_table(ComponentTableRequest {
                target: target.clone(),
                query,
                request_id,
                active_request_id: active_request_id.clone(),
                table: table.clone(),
                known_columns: known_columns.clone(),
                loading: loading.clone(),
                error: error.clone(),
                request_lifecycle: request_lifecycle.clone(),
                retry: retry.clone(),
                retry_attempt,
                retry_epoch,
                on_request_activity: on_request_activity.clone(),
            });
        }
    });

    let loading_for_busy = loading.clone();
    let table_for_inline_error = table.clone();
    let error_for_inline_error = error.clone();
    let table_for_results = table.clone();
    let known_columns_for_results = known_columns.clone();
    let loading_for_results = loading.clone();
    let title = table_presentation
        .as_ref()
        .and_then(|presentation| presentation.title.clone());
    let fullscreen = table_presentation
        .and_then(|presentation| presentation.fullscreen)
        .map(|presentation| ComponentTableFullscreenAction {
            dialog_id: presentation.dialog_id,
            dialog_title: presentation.dialog_title,
            open: fullscreen_open,
        });
    let inline_column_menu_id = fullscreen
        .as_ref()
        .map(|fullscreen| format!("{}-inline-columns", fullscreen.dialog_id))
        .unwrap_or_else(|| format!("component-table-{}-columns", target.component_version_id()));
    let fullscreen_for_results = fullscreen.clone();

    view! {
        <div
            class="component-table-viewer"
            aria-busy=move || loading_for_busy.get().to_string()
        >
            {move || {
                (table_for_inline_error.get().is_some())
                    .then(|| error_for_inline_error.get())
                    .flatten()
                    .map(|message| {
                        view! {
                            <p class="form-status is-error" aria-live="polite">{message}</p>
                        }
                    })
            }}
            {move || {
                if loading_for_results.get()
                    && table_for_results.get().is_none()
                    && error.get().is_none()
                {
                    view! {
                        <EmptyState
                            title="Loading preview"
                            message="Fetching the published table preview."
                        />
                    }
                    .into_any()
                } else if let Some(table) = table_for_results.get() {
                    if table.materialization_state != "ready" {
                        let (title, message) =
                            materialization_empty_state(&table.materialization_state);
                        view! { <EmptyState title message/> }.into_any()
                    } else if table.columns.is_empty() {
                        view! {
                            <EmptyState
                                title="No visible columns"
                                message="This component does not currently expose any table columns."
                            />
                        }
                        .into_any()
                    } else {
                        view! {
                            <ComponentTableResults
                                table
                                state
                                reset_state=ComponentTableViewState::for_mode(mode)
                                known_columns=known_columns_for_results.clone()
                                loading=loading_for_results.clone()
                                compact=mode == ComponentViewerMode::Embedded
                                title=title.clone()
                                fullscreen=fullscreen_for_results.clone()
                                column_menu_id=inline_column_menu_id.clone()
                            />
                        }
                        .into_any()
                    }
                } else if let Some(message) = error.get() {
                    view! { <EmptyState title="Preview unavailable" message/> }.into_any()
                } else {
                    view! {
                        <EmptyState
                            title="Preview unavailable"
                            message="Component table data could not be loaded."
                        />
                    }
                    .into_any()
                }
            }}
            {fullscreen.map(|fullscreen| {
                let fullscreen_table = table.clone();
                let fullscreen_columns = known_columns.clone();
                let fullscreen_loading = loading.clone();
                let open = Signal::from(fullscreen.open);
                let column_menu_id = format!("{}-dialog-columns", fullscreen.dialog_id);
                view! {
                    <FullscreenDialog
                        id=fullscreen.dialog_id
                        title=fullscreen.dialog_title
                        open
                        on_close=Callback::new(move |_| fullscreen_open.set(false))
                        class="component-table-fullscreen"
                    >
                        <ComponentTableFullscreenResults
                            open
                            table=fullscreen_table.clone()
                            state
                            known_columns=fullscreen_columns.clone()
                            loading=fullscreen_loading.clone()
                            column_menu_id=column_menu_id.clone()
                        />
                    </FullscreenDialog>
                }
            })}
        </div>
    }
}

#[derive(Clone, Debug)]
struct ComponentTableFullscreenAction {
    dialog_id: String,
    dialog_title: String,
    open: RwSignal<bool>,
}

#[component]
fn ComponentTableFullscreenResults(
    open: Signal<bool>,
    table: ArcRwSignal<Option<ComponentTable>>,
    state: RwSignal<ComponentTableViewState>,
    known_columns: ArcRwSignal<Vec<ComponentTableColumn>>,
    loading: ArcRwSignal<bool>,
    column_menu_id: String,
) -> impl IntoView {
    move || {
        if !open.get() {
            return None;
        }
        let known_columns = known_columns.clone();
        let loading = loading.clone();
        let column_menu_id = column_menu_id.clone();
        table.get().map(|table| {
            view! {
                <ComponentTableResults
                    table
                    state
                    reset_state=ComponentTableViewState::for_mode(ComponentViewerMode::Full)
                    known_columns
                    loading
                    compact=false
                    title=None
                    fullscreen=None
                    column_menu_id=column_menu_id.clone()
                />
            }
        })
    }
}

#[component]
fn ComponentTableResults(
    table: ComponentTable,
    state: RwSignal<ComponentTableViewState>,
    reset_state: ComponentTableViewState,
    known_columns: ArcRwSignal<Vec<ComponentTableColumn>>,
    loading: ArcRwSignal<bool>,
    #[prop(default = false)] compact: bool,
    title: Option<ComponentTableTitle>,
    fullscreen: Option<ComponentTableFullscreenAction>,
    column_menu_id: String,
) -> impl IntoView {
    let columns = table.columns.clone();
    let rows = table.rows.clone();
    let row_count = rows.len();
    let returned_page_size = table.pagination.page_size.max(1);
    let next_cursor = table.pagination.next_cursor.clone();
    let has_more = table.pagination.has_more && next_cursor.is_some();
    let mut page_sizes = vec![10_usize, 25, 50, 100, 200];
    if !page_sizes.contains(&returned_page_size) {
        page_sizes.push(returned_page_size);
        page_sizes.sort_unstable();
    }
    let loading_for_previous = loading.clone();
    let loading_for_next = loading.clone();

    view! {
        <div
            class="interactive-data-table component-table-viewer__table"
            class:interactive-data-table--compact=compact
        >
            <div class="interactive-data-table__toolbar">
                {title.map(|title| {
                    view! {
                        <h2 id=title.id class="interactive-data-table__title">{title.text}</h2>
                    }
                })}
                <TableSearch
                    value=Signal::derive(move || state.get().search)
                    on_input=Callback::new(move |value| {
                        state.update(|state| {
                            state.search = value;
                            state.reset_paging();
                        });
                    })
                    label="Search component rows"
                    placeholder="Search table"
                    class="interactive-data-table__search"
                />
                <div class="interactive-data-table__toolbar-actions">
                    <button
                        class="icon-button icon-button--control interactive-data-table__reset"
                        type="button"
                        aria-label="Reset table controls"
                        title="Reset table controls"
                        on:click={
                            let reset_state = reset_state.clone();
                            move |_| state.set(reset_state.clone())
                        }
                    >
                        <RotateCcw/>
                    </button>
                    <ComponentTableColumnMenu id=column_menu_id known_columns state/>
                    {fullscreen.map(|fullscreen| {
                        let dialog_id = fullscreen.dialog_id;
                        let open = fullscreen.open;
                        view! {
                            <button
                                class="icon-button icon-button--control interactive-data-table__fullscreen"
                                type="button"
                                aria-label="View fullscreen"
                                title="View fullscreen"
                                aria-haspopup="dialog"
                                aria-controls=dialog_id
                                aria-expanded=move || open.get().to_string()
                                on:click=move |_| open.set(true)
                            >
                                <Fullscreen/>
                            </button>
                        }
                    })}
                </div>
            </div>
            <div class="table-wrap interactive-data-table__table">
                <table class="data-table">
                    <thead>
                        <tr>
                            {columns
                                .clone()
                                .into_iter()
                                .map(|column| {
                                    view! { <ComponentTableHeader column state/> }
                                })
                                .collect_view()}
                        </tr>
                    </thead>
                    <tbody>
                        {if columns.is_empty() {
                            view! {
                                <tr>
                                    <td class="data-table__empty" colspan="1">
                                        "Select at least one column to display."
                                    </td>
                                </tr>
                            }
                            .into_any()
                        } else if rows.is_empty() {
                            view! {
                                <tr>
                                    <td
                                        class="data-table__empty"
                                        colspan=columns.len().to_string()
                                    >
                                        "No table rows match the current controls."
                                    </td>
                                </tr>
                            }
                            .into_any()
                        } else {
                            rows.into_iter()
                                .map(|row| {
                                    let columns = columns.clone();
                                    view! {
                                        <tr data-row-id=row.row_id>
                                            {columns
                                                .into_iter()
                                                .map(|column| {
                                                    let value = row
                                                        .values
                                                        .get(&column.key)
                                                        .cloned()
                                                        .flatten()
                                                        .unwrap_or_default();
                                                    view! { <td>{value}</td> }
                                                })
                                                .collect_view()}
                                        </tr>
                                    }
                                })
                                .collect_view()
                                .into_any()
                        }}
                    </tbody>
                </table>
            </div>
            <TablePaginationBar
                summary=Signal::derive(move || format!(
                    "Page {} · {} rows",
                    state.get().page_number(),
                    row_count
                ))
                aria_label="Table pagination"
                class="interactive-data-table__pagination"
            >
                    <label class="directory-table-pagination__page-size searchable-data-table__filter searchable-data-table__control">
                        <span>"Rows"</span>
                        <select
                            prop:value=move || state
                                .get()
                                .requested_page_size
                                .unwrap_or(returned_page_size)
                                .to_string()
                            on:change=move |event| {
                                if let Ok(page_size) = event_target_value(&event).parse::<usize>() {
                                    state.update(|state| {
                                        state.requested_page_size = Some(page_size);
                                        state.reset_paging();
                                    });
                                }
                            }
                        >
                            {page_sizes
                                .into_iter()
                                .map(|page_size| {
                                    view! {
                                        <option value=page_size.to_string()>
                                            {page_size.to_string()}
                                        </option>
                                    }
                                })
                                .collect_view()}
                        </select>
                    </label>
                    <button
                        class="interactive-data-table__page-button"
                        type="button"
                        aria-label="Previous page"
                        title="Previous page"
                        disabled=move || {
                            loading_for_previous.get() || state.get().previous_cursors.is_empty()
                        }
                        on:click=move |_| state.update(ComponentTableViewState::move_previous)
                    >
                        <ChevronLeft/>
                    </button>
                    <span>{move || format!("Page {}", state.get().page_number())}</span>
                    <button
                        class="interactive-data-table__page-button"
                        type="button"
                        aria-label="Next page"
                        title="Next page"
                        disabled=move || loading_for_next.get() || !has_more
                        on:click=move |_| {
                            if let Some(next_cursor) = next_cursor.clone() {
                                state.update(|state| state.move_next(next_cursor));
                            }
                        }
                    >
                        <ChevronRight/>
                    </button>
            </TablePaginationBar>
        </div>
    }
}

#[component]
fn ComponentTableColumnMenu(
    id: String,
    known_columns: ArcRwSignal<Vec<ComponentTableColumn>>,
    state: RwSignal<ComponentTableViewState>,
) -> impl IntoView {
    let columns = Signal::derive({
        let known_columns = known_columns.clone();
        move || {
            known_columns
                .get()
                .into_iter()
                .map(|column| TableColumnOption::new(column.key, column.label))
                .collect::<Vec<_>>()
        }
    });
    let visible_column_keys = Signal::derive({
        let known_columns = known_columns.clone();
        move || {
            state.get().visible_columns.unwrap_or_else(|| {
                known_columns
                    .get()
                    .into_iter()
                    .map(|column| column.key)
                    .collect()
            })
        }
    });
    let on_change = Callback::new(move |visible: Vec<String>| {
        let all_keys = known_columns
            .get_untracked()
            .into_iter()
            .map(|column| column.key)
            .collect::<Vec<_>>();
        state.update(|state| {
            state.visible_columns = (visible != all_keys).then_some(visible);
            state.reset_paging();
        });
    });
    view! {
        <TableColumnSelector
            id
            columns
            visible_column_keys
            on_change
            minimum_visible_columns=1
        />
    }
}

#[component]
fn ComponentTableHeader(
    column: ComponentTableColumn,
    state: RwSignal<ComponentTableViewState>,
) -> impl IntoView {
    let popover = TablePopoverController::new();
    let key = column.key;
    let label = column.label;
    let filter_operator = default_filter_operator(&column.field_type);
    let filter_label = if filter_operator == "contains" {
        "Contains"
    } else {
        "Equals"
    };
    let class_key = key.clone();
    let icon_key = key.clone();
    let ascending_key = key.clone();
    let descending_key = key.clone();
    let clear_sort_key = key.clone();
    let filter_key = key.clone();
    let filter_value_key = key.clone();
    let clear_filter_key = key.clone();

    view! {
        <th scope="col">
            <div class="interactive-data-table__header-cell">
                <span>{label.clone()}</span>
                <div class=move || {
                    if popover.open.get() {
                        "interactive-data-table__header-menu is-open"
                    } else {
                        "interactive-data-table__header-menu"
                    }
                }>
                    <button
                        node_ref=popover.trigger
                        class=move || {
                            if column_is_sorted_or_filtered(&state.get(), &class_key) {
                                "icon-button data-table-filter__trigger is-filtered"
                            } else {
                                "icon-button data-table-filter__trigger"
                            }
                        }
                        type="button"
                        aria-label=format!("Sort and filter {label}")
                        title=format!("Sort and filter {label}")
                        aria-haspopup="dialog"
                        aria-expanded=move || popover.open.get().to_string()
                        on:click=move |_| popover.toggle()
                    >
                        {move || match state.get().sort {
                            Some(sort) if sort.field_key == icon_key && sort.direction == "asc" => {
                                view! { <ArrowUp/> }.into_any()
                            }
                            Some(sort) if sort.field_key == icon_key => {
                                view! { <ArrowDown/> }.into_any()
                            }
                            _ => view! { <ListFilter/> }.into_any(),
                        }}
                    </button>
                    <button
                        class="data-table-filter__scrim"
                        type="button"
                        aria-label=format!("Close {label} controls")
                        on:click=move |_| popover.close()
                    ></button>
                    <div
                        node_ref=popover.panel
                        class="data-table-filter__menu blurred-surface interactive-data-table__header-controls"
                        role="dialog"
                        aria-label=format!("Sort and filter {label}")
                        tabindex="-1"
                        on:keydown=move |event| popover.handle_keydown(event)
                    >
                        <button
                            class="data-table-filter__option"
                            type="button"
                            on:click=move |_| {
                                state.update(|state| {
                                    state.sort = Some(ComponentTableSort {
                                        field_key: ascending_key.clone(),
                                        direction: "asc",
                                    });
                                    state.reset_paging();
                                });
                                popover.close();
                            }
                        >
                            "Sort ascending"
                        </button>
                        <button
                            class="data-table-filter__option"
                            type="button"
                            on:click=move |_| {
                                state.update(|state| {
                                    state.sort = Some(ComponentTableSort {
                                        field_key: descending_key.clone(),
                                        direction: "desc",
                                    });
                                    state.reset_paging();
                                });
                                popover.close();
                            }
                        >
                            "Sort descending"
                        </button>
                        <button
                            class="data-table-filter__option"
                            type="button"
                            on:click=move |_| {
                                state.update(|state| {
                                    if state.sort.as_ref().is_some_and(|sort| {
                                        sort.field_key == clear_sort_key
                                    }) {
                                        state.sort = None;
                                    }
                                    state.reset_paging();
                                });
                                popover.close();
                            }
                        >
                            "Clear sort"
                        </button>
                        <hr class="interactive-data-table__menu-rule"/>
                        <label class="form-field">
                            <span>{filter_label}</span>
                            <input
                                type="search"
                                placeholder="Filter values"
                                prop:value=move || state
                                    .get()
                                    .filters
                                    .get(&filter_value_key)
                                    .map(|filter| filter.value.clone())
                                    .unwrap_or_default()
                                on:input=move |event| {
                                    let value = event_target_value(&event);
                                    state.update(|state| {
                                        if value.trim().is_empty() {
                                            state.filters.remove(&filter_key);
                                        } else {
                                            state.filters.insert(
                                                filter_key.clone(),
                                                ComponentTableFilter {
                                                    operator: filter_operator,
                                                    value,
                                                },
                                            );
                                        }
                                        state.reset_paging();
                                    });
                                }
                            />
                        </label>
                        <button
                            class="data-table-filter__option"
                            type="button"
                            on:click=move |_| {
                                state.update(|state| {
                                    state.filters.remove(&clear_filter_key);
                                    state.reset_paging();
                                });
                                popover.close();
                            }
                        >
                            "Clear filter"
                        </button>
                    </div>
                </div>
            </div>
        </th>
    }
}

fn default_filter_operator(field_type: &str) -> &'static str {
    match field_type {
        "text" | "static_text" => "contains",
        _ => "equals",
    }
}

fn column_is_sorted_or_filtered(state: &ComponentTableViewState, key: &str) -> bool {
    state
        .sort
        .as_ref()
        .is_some_and(|sort| sort.field_key == key)
        || state.filters.contains_key(key)
}

#[cfg(feature = "hydrate")]
fn merge_known_columns(
    known_columns: &mut Vec<ComponentTableColumn>,
    response_columns: &[ComponentTableColumn],
) {
    for column in response_columns {
        if !known_columns.iter().any(|known| known.key == column.key) {
            known_columns.push(column.clone());
        }
    }
}

fn push_query_param(params: &mut Vec<String>, key: &str, value: &str) {
    let value = value.trim();
    if !value.is_empty() {
        params.push(format!(
            "{}={}",
            percent_encode_query_component(key),
            percent_encode_query_component(value)
        ));
    }
}

fn percent_encode_query_component(value: &str) -> String {
    value
        .bytes()
        .flat_map(|byte| match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                vec![byte as char]
            }
            byte => format!("%{byte:02X}").chars().collect::<Vec<_>>(),
        })
        .collect()
}

pub(crate) fn materialization_empty_state(state: &str) -> (&'static str, String) {
    match state {
        "failed" | "error" => (
            "Table materialization failed",
            "The component configuration is valid, but the bound Dataset major-line table could not be materialized. Retry after the Dataset materialization is rebuilt.".into(),
        ),
        "pending" => (
            "Table materializing",
            "The component configuration is valid, but the bound Dataset major-line table is still being prepared.".into(),
        ),
        other => (
            "Table materializing",
            format!("The component configuration is valid, but the bound Dataset major-line table is not ready yet. Materialization state: {other}"),
        ),
    }
}

#[cfg(feature = "hydrate")]
fn materialization_is_retryable(state: &str) -> bool {
    !matches!(state, "ready" | "failed" | "error")
}

struct ComponentTableRequest {
    target: ComponentVersionTarget,
    query: String,
    request_id: u64,
    active_request_id: ArcRwSignal<u64>,
    table: ArcRwSignal<Option<ComponentTable>>,
    known_columns: ArcRwSignal<Vec<ComponentTableColumn>>,
    loading: ArcRwSignal<bool>,
    error: ArcRwSignal<Option<String>>,
    request_lifecycle: ArcRwSignal<RequestLifecycle<(String, u64)>>,
    retry: ArcRwSignal<BoundedRetry<String>>,
    retry_attempt: usize,
    retry_epoch: u64,
    on_request_activity: Option<ComponentRequestActivityCallback>,
}

fn load_component_table(request: ComponentTableRequest) {
    #[cfg(feature = "hydrate")]
    let ComponentTableRequest {
        target,
        query,
        request_id,
        active_request_id,
        table,
        known_columns,
        loading,
        error,
        request_lifecycle,
        retry,
        retry_attempt,
        retry_epoch,
        on_request_activity,
    } = request;
    #[cfg(feature = "hydrate")]
    {
        let request_guard = RequestActivityGuard::new(request_lifecycle, on_request_activity);
        leptos::task::spawn_local(async move {
            let mut request_guard = request_guard;
            let expected_version_id = target.component_version_id.clone();
            let endpoint = target.endpoint_path();
            let result = api::fetch_component_table_endpoint(&endpoint, &query).await;
            if active_request_id.get_untracked() != request_id {
                return;
            }
            loading.set(false);
            let completion = match result {
                Ok(Some(response))
                    if response.component_type == "table"
                        && response.component_version_id == expected_version_id =>
                {
                    let retryable = materialization_is_retryable(&response.materialization_state);
                    known_columns.update(|known| merge_known_columns(known, &response.columns));
                    table.set(Some(response));
                    error.set(None);
                    if retryable {
                        schedule_bounded_retry(
                            retry,
                            query,
                            retry_attempt,
                            retry_epoch,
                            request_id,
                            active_request_id,
                        );
                        RequestCompletion::Retryable
                    } else {
                        RequestCompletion::Successful
                    }
                }
                Ok(Some(_)) => {
                    table.set(None);
                    error.set(Some(
                        "Component table response did not match the requested exact version."
                            .into(),
                    ));
                    RequestCompletion::TerminalFailure
                }
                Ok(None) => {
                    table.set(None);
                    RequestCompletion::TerminalFailure
                }
                Err(request_error) => {
                    let retryable = request_error.is_retryable();
                    error.set(Some(request_error.into_message()));
                    if retryable {
                        schedule_bounded_retry(
                            retry,
                            query,
                            retry_attempt,
                            retry_epoch,
                            request_id,
                            active_request_id,
                        );
                        RequestCompletion::Retryable
                    } else {
                        RequestCompletion::TerminalFailure
                    }
                }
            };
            request_guard.complete(completion);
        });
    }
    #[cfg(not(feature = "hydrate"))]
    {
        let ComponentTableRequest {
            target,
            query,
            request_id,
            active_request_id,
            table,
            known_columns,
            loading,
            error,
            request_lifecycle,
            retry,
            retry_attempt,
            retry_epoch,
            on_request_activity,
        } = request;
        let _ = (
            target,
            query,
            request_id,
            active_request_id,
            table,
            known_columns,
            loading,
            error,
            retry,
            retry_attempt,
            retry_epoch,
            on_request_activity,
        );
        request_lifecycle.update(|lifecycle| lifecycle.settle(RequestCompletion::Successful));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[cfg(feature = "ssr")]
    use crate::types::{
        ComponentStatValue, ComponentTablePagination, ComponentTableRow, ComponentVisual,
        ComponentVisualPoint, ComponentVisualSlice,
    };
    #[cfg(feature = "ssr")]
    use std::collections::BTreeMap;

    #[test]
    fn persisted_table_state_restores_page_controls_and_stays_bounded() {
        let mut cache = PersistedTableStateCache::default();
        let mut page_two = ComponentTableViewState {
            requested_page_size: Some(50),
            cursor: Some("offset:50".into()),
            previous_cursors: vec![None],
            search: "outreach".into(),
            ..Default::default()
        };
        page_two.visible_columns = Some(vec!["program".into()]);
        cache.persist("placement-one", page_two.clone(), 2);

        assert_eq!(cache.load("placement-one"), page_two);
        cache.persist("placement-two", ComponentTableViewState::default(), 2);
        cache.persist("placement-three", ComponentTableViewState::default(), 2);
        assert!(!cache.states.contains_key("placement-one"));
        assert_eq!(cache.states.len(), 2);
    }

    #[test]
    fn exact_version_targets_build_all_supported_endpoint_paths() {
        for (kind, segment) in [
            (ComponentVersionKind::Table, "table"),
            (ComponentVersionKind::Bar, "bar"),
            (ComponentVersionKind::Line, "line"),
            (ComponentVersionKind::Pie, "pie"),
            (ComponentVersionKind::Donut, "donut"),
            (ComponentVersionKind::StatCard, "stat-card"),
        ] {
            assert_eq!(
                ComponentVersionTarget::new("attendance", "version-2", kind).endpoint_path(),
                format!("/api/components/attendance/versions/version-2/{segment}")
            );
        }
    }

    #[test]
    fn mediated_targets_use_only_the_owner_supplied_endpoint() {
        let target = ComponentVersionTarget::mediated(
            "attendance",
            "secret-version-id",
            ComponentVersionKind::Table,
            "/api/presentations/container-1/items/item-2/render/table",
        );
        let endpoint = target.endpoint_path();
        assert_eq!(
            endpoint,
            "/api/presentations/container-1/items/item-2/render/table"
        );
        assert!(!endpoint.contains("attendance"));
        assert!(!endpoint.contains("secret-version-id"));
    }

    #[test]
    fn api_kind_adapter_accepts_only_the_six_reader_kinds() {
        for kind in [
            ComponentVersionKind::Table,
            ComponentVersionKind::Bar,
            ComponentVersionKind::Line,
            ComponentVersionKind::Pie,
            ComponentVersionKind::Donut,
            ComponentVersionKind::StatCard,
        ] {
            assert_eq!(
                ComponentVersionKind::from_api_kind(kind.as_api_value()),
                Some(kind)
            );
        }
        assert_eq!(ComponentVersionKind::from_api_kind("report"), None);
    }

    #[test]
    fn query_serializes_complete_server_view_state() {
        let mut state = ComponentTableViewState {
            search: "family outreach".into(),
            requested_page_size: Some(25),
            cursor: Some("offset:25".into()),
            sort: Some(ComponentTableSort {
                field_key: "program".into(),
                direction: "desc",
            }),
            visible_columns: Some(vec!["program".into(), "row_count".into()]),
            ..Default::default()
        };
        state.filters.insert(
            "program".into(),
            ComponentTableFilter {
                operator: "contains",
                value: "demo".into(),
            },
        );
        state.filters.insert(
            "row_count".into(),
            ComponentTableFilter {
                operator: "between",
                value: "1,10".into(),
            },
        );

        assert_eq!(
            state.query(),
            "q=family%20outreach&page_size=25&cursor=offset%3A25&sort=program%3Adesc&filter%5Bprogram%5D%5Boperator%5D=contains&filter%5Bprogram%5D%5Bvalue%5D=demo&filter%5Brow_count%5D%5Boperator%5D=between&filter%5Brow_count%5D%5Bvalue%5D=1%2C10&visible_columns=program%2Crow_count"
        );
    }

    #[test]
    fn cursor_history_supports_server_backed_previous_and_next() {
        let mut state = ComponentTableViewState::default();
        state.move_next("offset:25".into());
        state.move_next("offset:50".into());
        assert_eq!(state.cursor.as_deref(), Some("offset:50"));
        assert_eq!(state.page_number(), 3);

        state.move_previous();
        assert_eq!(state.cursor.as_deref(), Some("offset:25"));
        assert_eq!(state.page_number(), 2);
        state.move_previous();
        assert_eq!(state.cursor, None);
        assert_eq!(state.page_number(), 1);
    }

    #[test]
    fn changing_query_controls_resets_cursor_history() {
        let mut state = ComponentTableViewState::default();
        state.move_next("offset:25".into());
        state.move_next("offset:50".into());
        state.search = "updated".into();
        state.reset_paging();

        assert_eq!(state.cursor, None);
        assert!(state.previous_cursors.is_empty());
        assert_eq!(state.query(), "q=updated");
    }

    #[test]
    fn filter_operator_matches_the_server_field_contract() {
        assert_eq!(default_filter_operator("text"), "contains");
        assert_eq!(default_filter_operator("static_text"), "contains");
        assert_eq!(default_filter_operator("number"), "equals");
        assert_eq!(default_filter_operator("date"), "equals");
        assert_eq!(default_filter_operator("boolean"), "equals");
    }

    #[test]
    fn materialization_empty_state_distinguishes_failed_from_pending() {
        let (pending_title, pending_message) = materialization_empty_state("pending");
        assert_eq!(pending_title, "Table materializing");
        assert!(pending_message.contains("still being prepared"));

        let (failed_title, failed_message) = materialization_empty_state("failed");
        assert_eq!(failed_title, "Table materialization failed");
        assert!(failed_message.contains("configuration is valid"));

        let (retry_title, retry_message) = materialization_empty_state("retry");
        assert_eq!(retry_title, "Table materializing");
        assert!(retry_message.contains("retry"));
    }

    #[cfg(feature = "ssr")]
    #[test]
    fn table_result_renders_a_bounded_server_page_with_shared_presentation_controls() {
        let mut values = BTreeMap::new();
        values.insert("program".into(), Some("Outreach".into()));
        let table = ComponentTable {
            component_id: "component-1".into(),
            component_version_id: "version-1".into(),
            dataset_id: "dataset-1".into(),
            dataset_version_major: 1,
            component_type: "table".into(),
            materialization_state: "ready".into(),
            columns: vec![ComponentTableColumn {
                key: "program".into(),
                label: "Program".into(),
                field_type: "text".into(),
            }],
            rows: vec![ComponentTableRow {
                row_id: "row-1".into(),
                values,
            }],
            pagination: ComponentTablePagination {
                page_size: 25,
                next_cursor: Some("offset:25".into()),
                has_more: true,
            },
        };
        let html = Owner::new().with(|| {
            let state = RwSignal::new(ComponentTableViewState::default());
            let known_columns = ArcRwSignal::new(table.columns.clone());
            let loading = ArcRwSignal::new(false);
            let fullscreen_open = RwSignal::new(false);
            view! {
                <ComponentTableResults
                    table
                    state
                    reset_state=ComponentTableViewState::default()
                    known_columns
                    loading
                    title=Some(ComponentTableTitle {
                        id: "component-table-title-version-1".into(),
                        text: "Program activity".into(),
                    })
                    fullscreen=Some(ComponentTableFullscreenAction {
                        dialog_id: "component-table-fullscreen-version-1".into(),
                        dialog_title: "Program activity — fullscreen Table".into(),
                        open: fullscreen_open,
                    })
                    column_menu_id="component-table-fullscreen-version-1-inline-columns".into()
                />
            }
            .to_html()
        });

        assert!(html.contains("Outreach"));
        assert!(html.contains("Search table"));
        assert!(html.contains("Next page"));
        assert!(html.contains("Page 1"));
        assert!(html.contains("class=\"interactive-data-table__title\""));
        assert!(html.contains("id=\"component-table-title-version-1\""));
        assert!(html.contains("Program activity"));
        assert!(html.contains("aria-label=\"View fullscreen\""));
        assert!(html.contains("title=\"View fullscreen\""));
        assert!(html.contains("aria-haspopup=\"dialog\""));
        assert!(html.contains("aria-controls=\"component-table-fullscreen-version-1\""));
        assert!(
            html.contains("aria-controls=\"component-table-fullscreen-version-1-inline-columns\"")
        );
        assert!(html.contains("id=\"component-table-fullscreen-version-1-inline-columns\""));
        assert!(html.contains("aria-expanded=\"false\""));
        let reset_index = html.find("Reset table controls").expect("reset action");
        let columns_index = html.find("Choose visible columns").expect("columns action");
        let fullscreen_index = html.find("View fullscreen").expect("fullscreen action");
        assert!(reset_index < columns_index && columns_index < fullscreen_index);
        assert!(!html.contains("component-table-preview__header"));
        assert!(!html.contains("Showing 1 rows across 1 visible columns."));
    }

    #[cfg(feature = "ssr")]
    #[test]
    fn visual_result_renders_bar_line_pie_donut_and_stat_card() {
        for kind in ["bar", "line", "pie", "donut"] {
            let html = view! {
                <crate::visual::ComponentVisualPresentation visual=visual_fixture(kind)/>
            }
            .to_html();
            assert!(
                html.contains("component-d3-chart"),
                "missing chart for {kind}"
            );
            assert!(html.contains("data-chart"), "missing payload for {kind}");
            assert!(
                html.contains(&format!("{} chart preview", kind_label(kind))),
                "missing accessible label for {kind}: {html}"
            );
        }

        let stat_html = view! {
            <crate::visual::ComponentVisualPresentation visual=visual_fixture("stat_card")/>
        }
        .to_html();
        assert!(stat_html.contains("component-stat-card--accent"));
        assert!(stat_html.contains("Active families"));
        assert!(stat_html.contains("42"));
    }

    #[test]
    fn public_modes_have_stable_feature_neutral_markers() {
        assert_eq!(
            ComponentViewerMode::Full.class_name(),
            "component-version-execution component-version-execution--full"
        );
        assert_eq!(ComponentViewerMode::Full.as_data_value(), "full");
        assert_eq!(
            ComponentViewerMode::Embedded.class_name(),
            "component-version-execution component-version-execution--embedded"
        );
        assert_eq!(ComponentViewerMode::Embedded.as_data_value(), "embedded");
    }

    #[test]
    fn embedded_tables_default_to_a_compact_page_without_constraining_full_mode() {
        assert_eq!(
            ComponentTableViewState::for_mode(ComponentViewerMode::Embedded).requested_page_size,
            Some(10)
        );
        assert_eq!(
            ComponentTableViewState::for_mode(ComponentViewerMode::Full).requested_page_size,
            None
        );
    }

    #[cfg(feature = "ssr")]
    fn visual_fixture(kind: &str) -> ComponentVisual {
        let is_stat = kind == "stat_card";
        let is_round = matches!(kind, "pie" | "donut");
        ComponentVisual {
            component_id: "component-1".into(),
            component_version_id: "version-1".into(),
            dataset_id: "dataset-1".into(),
            dataset_version_major: 1,
            component_type: kind.into(),
            materialization_state: "ready".into(),
            value_format: "number".into(),
            legend_title: Some("Program".into()),
            bar_orientation: (kind == "bar").then(|| "vertical".into()),
            bar_comparison_layout: None,
            x_axis_label: None,
            y_axis_label: None,
            line_smoothing: (kind == "line").then_some(true),
            stat: is_stat.then(|| ComponentStatValue {
                label: "Active families".into(),
                value: Some(42.0),
                display_value: Some("42".into()),
                supporting_text: Some("Current period".into()),
                panel_style: "accent".into(),
            }),
            points: if !is_stat && !is_round {
                vec![ComponentVisualPoint {
                    x: "Outreach".into(),
                    value: 42.0,
                    display_value: "42".into(),
                    color: None,
                    comparison: None,
                }]
            } else {
                Vec::new()
            },
            slices: if is_round {
                vec![ComponentVisualSlice {
                    category: "Outreach".into(),
                    value: 42.0,
                    display_value: "42".into(),
                    color: None,
                }]
            } else {
                Vec::new()
            },
        }
    }

    #[cfg(feature = "ssr")]
    fn kind_label(kind: &str) -> &'static str {
        match kind {
            "bar" => "Bar",
            "line" => "Line",
            "pie" => "Pie",
            "donut" => "Donut",
            _ => "Component",
        }
    }
}
