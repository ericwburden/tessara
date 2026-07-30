use std::collections::BTreeSet;
#[cfg(all(feature = "hydrate", target_arch = "wasm32"))]
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};

use icons::{
    ArrowDown, ArrowLeft, ArrowRight, ArrowUp, ChartBar, ChartLine, ChartPie, Donut, GripVertical,
    SquareActivity, Table2, TriangleAlert,
};
use leptos::{ev, prelude::*};
use tessara_core::{resolve_move_request, resolve_resize_request};
use tessara_dashboards::{
    DASHBOARD_GRID_CONSTRAINTS, DashboardPlacementSizePolicy, GridPlacement, GridRect, GridSize,
    reflow_dashboard_movement, validate_dashboard_layout, validate_dashboard_resize,
};
use tessara_web_component_viewer::{
    ComponentVersionExecutionContent, ComponentVersionKind, ComponentVersionTarget,
    ComponentViewerMode,
};
use tessara_web_ui::placement_editor::{
    GridMoveDirection, GridMoveRequest, GridResizeRequest, PlacementDragPreview,
    PlacementGridCanvas, PlacementGridCell, PlacementGridGuides, PlacementGridTarget,
    PlacementResizeHandles, PlacementResizeStart, PlacementTileShell, clear_placement_drag_intent,
    install_placement_drag_cleanup, placement_move_request_from_direct,
    placement_move_request_from_keyboard, placement_resize_request_from_direct,
    schedule_placement_drag_preview, set_placement_drag_preview,
};
#[cfg(all(feature = "hydrate", target_arch = "wasm32"))]
use tessara_web_ui::placement_editor::{
    PlacementResizeSession, placement_grid_metrics_from_element, placement_tile_style,
};
use tessara_web_ui::{EmptyState, ModalDialog, SideSheet, SideSheetSide, TableSearch};
#[cfg(all(feature = "hydrate", target_arch = "wasm32"))]
use wasm_bindgen::{JsCast, closure::Closure};

use crate::types::{
    DashboardComponentVersion, DashboardComponentVersionOption, DashboardComposition,
    DashboardCompositionCommand, DashboardMetadataRequest, DashboardPlacement,
    DashboardPlacementAvailability, DashboardPlacementConfigState, DashboardPlacementGeometry,
    DashboardPlacementOperation, DashboardPlacementResolutionState, EditorPlacement,
    ReconcileDashboardCompositionRequest, SessionAccount, VisibilityNodeOption,
};
use crate::{DashboardRouteBootstrap, dashboard_route_bootstrap};

use super::{detail::kind_label, version_label};

mod navigation_guard;
mod operation;
mod requests;

use navigation_guard::{
    install_dirty_navigation_guard, navigate, navigate_with_dirty_confirmation,
};
use operation::EditorOperation;
use requests::{
    CompositionLoadContext, LayoutSaveContext, SettingsSaveContext, load_composition,
    load_settings_nodes, save_layout, save_settings,
};

#[cfg(all(feature = "hydrate", target_arch = "wasm32"))]
thread_local! {
    static ACTIVE_DASHBOARD_RESIZE_CLEANUP: RefCell<Option<DashboardResizeCleanup>> =
        const { RefCell::new(None) };
}

#[component]
pub fn DashboardEditorContent(dashboard_id: String) -> impl IntoView {
    let (initial_account, initial_composition, initial_visibility_nodes, bootstrapped) =
        match dashboard_route_bootstrap() {
            Some(DashboardRouteBootstrap::Editor {
                account,
                composition,
                visibility_nodes,
            }) if composition.dashboard.id == dashboard_id => {
                (Some(account), Some(composition), visibility_nodes, true)
            }
            _ => (None, None, Vec::new(), false),
        };
    let initial_placements = initial_composition
        .as_ref()
        .map(|composition| {
            composition
                .dashboard
                .placements
                .iter()
                .cloned()
                .map(EditorPlacement::existing)
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    let composition = RwSignal::new(initial_composition);
    let account = RwSignal::new(initial_account);
    let placements = RwSignal::new(initial_placements);
    let settings_nodes = RwSignal::new(initial_visibility_nodes);
    let loading = RwSignal::new(cfg!(feature = "hydrate") && !bootstrapped);
    let load_error = RwSignal::new(None::<String>);
    let selected = RwSignal::new(None::<String>);
    let dirty = RwSignal::new(false);
    let operation = RwSignal::new(EditorOperation::Idle);
    let save_error = RwSignal::new(None::<String>);
    let announcement = RwSignal::new(String::new());
    let preview_open = RwSignal::new(false);
    let components_open = RwSignal::new(false);
    let inspector_open = RwSignal::new(false);
    let issue_placement = RwSignal::new(None::<DashboardPlacement>);
    let confirmed_navigation = RwSignal::new(false);
    let next_client_key = RwSignal::new(1_u64);
    let save_dashboard_id = StoredValue::new(dashboard_id.clone());

    Effect::new({
        let dashboard_id = dashboard_id.clone();
        move |_| {
            if !bootstrapped {
                load_composition(
                    dashboard_id.clone(),
                    CompositionLoadContext {
                        composition,
                        account,
                        placements,
                        loading,
                        error: load_error,
                    },
                );
            }
        }
    });

    install_dirty_navigation_guard(dirty, confirmed_navigation);

    let local_placement_count = Signal::derive(move || {
        placements
            .get()
            .iter()
            .filter(|placement| !placement.removed)
            .count()
    });

    view! {
        <section class="route-panel dashboards-page dashboard-editor">
            {move || if loading.get() {
                view! { <EmptyState title="Loading Dashboard editor" message="Fetching composition metadata without executing Components."/> }.into_any()
            } else if let Some(message) = load_error.get() {
                view! { <EmptyState title="Dashboard editor unavailable" message=message/> }.into_any()
            } else if let Some(loaded) = composition.get() {
                let dashboard_name = loaded.dashboard.name.clone();
                let detail_href = format!("/dashboards/{}", loaded.dashboard.id);
                let viewer_href = format!("/dashboards/{}/view", loaded.dashboard.id);
                let metadata_dashboard_id = loaded.dashboard.id.clone();
                let issue_retry_href =
                    StoredValue::new(format!("/dashboards/{}/edit", loaded.dashboard.id));
                let issue_title = Signal::derive(move || {
                    issue_placement
                        .get()
                        .map(|placement| placement.effective_resolution_state().title().to_string())
                        .unwrap_or_else(|| "Placement issue".to_string())
                });
                let current_account = account.get();
                let can_read_dashboard = editor_reader_actions_visible(current_account.as_ref());
                view! {
                        <div
                            class="dashboard-editor__background"
                            inert=move || {
                                preview_open.get()
                                    || components_open.get()
                                    || inspector_open.get()
                                    || issue_placement.get().is_some()
                                    || operation.get().is_busy()
                            }
                            aria-busy=move || operation.get().is_busy().to_string()
                        >
                        <header class="dashboard-editor__header">
                            <div>
                                <span class="dashboard-editor__eyebrow">"Dashboard builder"</span>
                                <h1>{dashboard_name}</h1>
                                <p>{move || format!("{} local placements · reading order is derived", local_placement_count.get())}</p>
                                <span
                                    id="dashboard-editor-unsaved-status"
                                    class="dashboard-editor__unsaved-status"
                                    class:is-unsaved=move || dirty.get()
                                    role="status"
                                    aria-live="polite"
                                >
                                    {move || if dirty.get() { "Unsaved layout changes" } else { "Layout changes saved" }}
                                </span>
                            </div>
                            <div class="dashboard-editor__header-actions">
                                {can_read_dashboard.then(|| {
                                    let detail_href_for_click = detail_href.clone();
                                    view! {
                                        <a
                                            class="button button--secondary"
                                            href=detail_href
                                            on:click=move |event| {
                                                event.prevent_default();
                                                navigate_with_dirty_confirmation(
                                                    &detail_href_for_click,
                                                    dirty,
                                                    confirmed_navigation,
                                                );
                                            }
                                        >"Details"</a>
                                    }
                                })}
                                {can_read_dashboard.then(|| view! {
                                    <button
                                        class="button button--secondary"
                                        type="button"
                                        disabled=move || dirty.get()
                                        aria-describedby=move || dirty.get().then_some("dashboard-preview-requirement")
                                        title=move || dirty.get().then_some("Save the layout before previewing the Dashboard.")
                                        on:click={
                                            let viewer_href = viewer_href.clone();
                                            move |_| navigate(&viewer_href)
                                        }
                                    >"Preview Dashboard"</button>
                                })}
                                <span id="dashboard-preview-requirement" class="sr-only">
                                    "Save the layout before previewing the Dashboard."
                                </span>
                                <button
                                    class="button"
                                    type="button"
                                    disabled=move || operation.get().is_busy() || !dirty.get()
                                    on:click=move |_| {
                                        if operation.get_untracked().is_busy()
                                            || !dirty.get_untracked()
                                        {
                                            return;
                                        }
                                        match build_reconcile_request(&placements.get_untracked()) {
                                            Ok(payload) => save_layout(
                                                save_dashboard_id.get_value(),
                                                payload,
                                                LayoutSaveContext {
                                                    composition,
                                                    placements,
                                                    selected,
                                                    dirty,
                                                    operation,
                                                    error: save_error,
                                                    announcement,
                                                },
                                            ),
                                            Err(message) => save_error.set(Some(message)),
                                        }
                                    }
                                >
                                    {move || if operation.get() == EditorOperation::SavingLayout {
                                        "Saving…"
                                    } else {
                                        "Save layout"
                                    }}
                                </button>
                            </div>
                        </header>

                        <DashboardMetadataSettings
                            dashboard_id=metadata_dashboard_id
                            composition
                            announcement
                            nodes=settings_nodes
                            nodes_bootstrapped=bootstrapped
                            placements
                            layout_dirty=dirty
                            operation
                        />

                        <div class="dashboard-editor__tools" aria-label="Dashboard editor tools">
                            <div class="dashboard-editor__tool-actions">
                                <button
                                    id="dashboard-components-trigger"
                                    class="button button--secondary"
                                    type="button"
                                    aria-haspopup="dialog"
                                    aria-controls="dashboard-components-sheet"
                                    aria-expanded=move || components_open.get().to_string()
                                    on:click=move |_| {
                                        inspector_open.set(false);
                                        components_open.set(true);
                                    }
                                >"Components"</button>
                                <button
                                    id="dashboard-placement-details-trigger"
                                    class="button button--secondary"
                                    type="button"
                                    disabled=move || selected.get().is_none()
                                    aria-haspopup="dialog"
                                    aria-controls="dashboard-placement-inspector"
                                    aria-expanded=move || inspector_open.get().to_string()
                                    on:click=move |_| {
                                        if selected.get_untracked().is_some() {
                                            components_open.set(false);
                                            inspector_open.set(true);
                                        }
                                    }
                                >"Placement details"</button>
                            </div>
                            <p aria-live="polite">
                                {move || if selected.get().is_some() {
                                    "A placement is selected. Open Placement details to resize, reposition, replace, or preview it."
                                } else {
                                    "Select a placement on the canvas to enable Placement details."
                                }}
                            </p>
                        </div>

                        <div
                            class="dashboard-editor__workspace"
                            class:is-saving=move || operation.get().is_busy()
                            inert=move || operation.get().is_busy()
                            aria-busy=move || operation.get().is_busy().to_string()
                        >
                            <CompositionCanvas
                                placements
                                selected
                                issue_placement
                                dirty
                                save_error
                                announcement
                            />
                        </div>

                        <SideSheet
                            id="dashboard-components-sheet"
                            title="Components"
                            description="Search placeable Component versions and add them to the Dashboard canvas."
                            eyebrow="Dashboard editor"
                            side=SideSheetSide::Start
                            open=Signal::derive(move || components_open.get() && !preview_open.get())
                            on_close=Callback::new(move |_| components_open.set(false))
                            close_label="Close Components"
                            class="dashboard-components-sheet"
                        >
                            <ComponentPalette
                                options=loaded.available_component_versions.clone()
                                placements
                                selected
                                dirty
                                save_error
                                announcement
                                next_client_key
                            />
                        </SideSheet>

                        <SideSheet
                            id="dashboard-placement-issue"
                            title=issue_title
                            description="Authorized diagnostic detail for the selected placement."
                            eyebrow="Placement issue"
                            side=SideSheetSide::End
                            open=Signal::derive(move || issue_placement.get().is_some())
                            on_close=Callback::new(move |_| issue_placement.set(None))
                            close_label="Close placement issue"
                            class="dashboard-placement-issue-sheet"
                        >
                            {move || issue_placement.get().map(|placement| {
                                let state = placement.effective_resolution_state();
                                let retry_href = issue_retry_href.get_value();
                                view! {
                                    <div class="dashboard-placement-issue-sheet__body">
                                        <div class="dashboard-placement-issue-sheet__icon" aria-hidden="true">
                                            <TriangleAlert/>
                                        </div>
                                        <span class="status-badge status-badge--warning">{state.label()}</span>
                                        <p>{state.message()}</p>
                                        <p class="dashboard-placement-issue-sheet__containment">
                                            "The saved footprint and exact ComponentVersion reference are unchanged."
                                        </p>
                                        {state.retryable().then(|| view! {
                                            <button
                                                type="button"
                                                class="button dashboard-placement-issue-sheet__retry"
                                                on:click=move |_| retry_resolution(&retry_href)
                                            >
                                                "Retry resolution"
                                            </button>
                                        })}
                                    </div>
                                }
                            })}
                        </SideSheet>

                        <SideSheet
                            id="dashboard-placement-inspector"
                            title="Placement details"
                            description="Edit the selected placement without reducing the canvas workspace."
                            eyebrow="Dashboard editor"
                            side=SideSheetSide::End
                            open=Signal::derive(move || inspector_open.get())
                            on_close=Callback::new(move |_| inspector_open.set(false))
                            close_label="Close Placement details"
                            class="dashboard-placement-sheet"
                        >
                            <PlacementInspector
                                placements
                                selected
                                options=composition
                                dirty
                                save_error
                                announcement
                                preview_open
                            />
                        </SideSheet>

                        <div class="dashboard-editor__status" aria-live="polite" aria-atomic="true">
                            {move || save_error.get().map(|message| view! { <p class="form-error">{message}</p> })}
                            <span>{move || announcement.get()}</span>
                        </div>
                    </div>

                    <SelectedPlacementPreview placements selected preview_open/>
                }.into_any()
            } else {
                view! { <EmptyState title="Dashboard editor unavailable" message="No composition payload was returned."/> }.into_any()
            }}
        </section>
    }
}

#[cfg(feature = "hydrate")]
fn retry_resolution(href: &str) {
    if let Some(window) = web_sys::window() {
        let _ = window.location().set_href(href);
    }
}

#[cfg(not(feature = "hydrate"))]
fn retry_resolution(_: &str) {}

#[component]
fn ComponentPalette(
    options: Vec<DashboardComponentVersionOption>,
    placements: RwSignal<Vec<EditorPlacement>>,
    selected: RwSignal<Option<String>>,
    dirty: RwSignal<bool>,
    save_error: RwSignal<Option<String>>,
    announcement: RwSignal<String>,
    next_client_key: RwSignal<u64>,
) -> impl IntoView {
    let search = RwSignal::new(String::new());
    let kind_filter = RwSignal::new("all".to_string());
    let options = StoredValue::new(options);
    let filtered = Memo::new(move |_| {
        let query = search.get().trim().to_lowercase();
        let kind = kind_filter.get();
        options
            .get_value()
            .into_iter()
            .filter(|option| {
                (kind == "all" || option.component_type == kind)
                    && (query.is_empty()
                        || option.component_name.to_lowercase().contains(&query)
                        || option.component_type.to_lowercase().contains(&query)
                        || option.version_label.to_lowercase().contains(&query))
            })
            .collect::<Vec<_>>()
    });

    view! {
        <section class="dashboard-editor__palette" aria-label="Available Components">
            <TableSearch
                value=Signal::derive(move || search.get())
                on_input=Callback::new(move |value| search.set(value))
                label="Search Components"
                placeholder="Search Components"
                class="dashboard-component-search"
            />
            <label class="form-field dashboard-component-kind-picker">
                <span>"Kind"</span>
                <select
                    aria-label="Filter components by kind"
                    prop:value=move || kind_filter.get()
                    on:change=move |event| kind_filter.set(event_target_value(&event))
                >
                    <option value="all">"All kinds"</option>
                    <option value="table">"Table"</option>
                    <option value="bar">"Bar"</option>
                    <option value="line">"Line"</option>
                    <option value="pie">"Pie"</option>
                    <option value="donut">"Donut"</option>
                    <option value="stat_card">"Stat Card"</option>
                </select>
            </label>
            <div class="dashboard-component-list">
                {move || if filtered.get().is_empty() {
                    view! { <p class="field-help">"No matching placeable Component versions."</p> }.into_any()
                } else {
                    filtered.get().into_iter().map(|option| {
                        let add_option = option.clone();
                        let add_label = format!(
                            "Add {} exact version {} to Dashboard",
                            option.component_name, option.component_version_id
                        );
                        view! {
                            <article class="dashboard-component-option" data-component-kind=option.component_type>
                                <span class="dashboard-component-option__icon" aria-hidden="true">
                                    {kind_icon(&option.component_type)}
                                </span>
                                <div>
                                    <strong>{option.component_name}</strong>
                                    <span>{format!("{} · {}", kind_label(&option.component_type), version_label(option.version_number, &option.version_label))}</span>
                                </div>
                                <button
                                    class="button button--secondary button--compact"
                                    type="button"
                                    aria-label=add_label
                                    on:click=move |_| {
                                        add_component(
                                            add_option.clone(),
                                            placements,
                                            selected,
                                            dirty,
                                            save_error,
                                            announcement,
                                            next_client_key,
                                        );
                                    }
                                >"Add"</button>
                            </article>
                        }
                    }).collect_view().into_any()
                }}
            </div>
        </section>
    }
}

#[component]
fn CompositionCanvas(
    placements: RwSignal<Vec<EditorPlacement>>,
    selected: RwSignal<Option<String>>,
    issue_placement: RwSignal<Option<DashboardPlacement>>,
    dirty: RwSignal<bool>,
    save_error: RwSignal<Option<String>>,
    announcement: RwSignal<String>,
) -> impl IntoView {
    let dragged = RwSignal::new(None::<String>);
    let drag_preview = RwSignal::new(None::<PlacementDragPreview<String, String>>);
    let pending_drag_preview = RwSignal::new(None::<PlacementDragPreview<String, String>>);
    let drag_preview_timeout = RwSignal::new(None::<i32>);
    install_placement_drag_cleanup(drag_preview, pending_drag_preview, drag_preview_timeout);
    #[cfg(all(feature = "hydrate", target_arch = "wasm32"))]
    on_cleanup(clear_active_dashboard_resize);
    let visible = Memo::new(move |_| {
        let mut active = placements
            .get()
            .into_iter()
            .filter(|placement| !placement.removed)
            .collect::<Vec<_>>();
        active.sort_by_key(|placement| {
            (
                placement.placement.grid_row,
                placement.placement.grid_column,
                placement.key().to_string(),
            )
        });
        active
    });
    let row_count = Signal::derive(move || {
        visible
            .get()
            .iter()
            .map(|placement| {
                placement
                    .placement
                    .grid_row
                    .saturating_add(placement.placement.grid_height)
                    .saturating_sub(1)
            })
            .max()
            .unwrap_or(1)
            .clamp(12, 240)
    });
    let grid_cells = Signal::derive(move || {
        (1..=row_count.get())
            .flat_map(|row| (1..=12).map(move |column| PlacementGridCell { row, column }))
            .collect::<Vec<_>>()
    });
    let dragging = Signal::derive(move || dragged.get().is_some());
    let on_drag_target = Callback::new(move |target: PlacementGridTarget| {
        let Some(placement_id) = dragged.get_untracked() else {
            return;
        };
        schedule_placement_drag_preview(
            drag_preview,
            pending_drag_preview,
            drag_preview_timeout,
            PlacementDragPreview {
                placement_id,
                canvas_id: "dashboard-composition".to_string(),
                row: target.cell.row,
                column: target.cell.column,
            },
            target.target_id,
        );
    });
    let on_drop_target = Callback::new(move |target: PlacementGridTarget| {
        let Some(placement_id) = dragged.get_untracked() else {
            return;
        };
        set_placement_drag_preview(
            drag_preview,
            PlacementDragPreview {
                placement_id: placement_id.clone(),
                canvas_id: "dashboard-composition".to_string(),
                row: target.cell.row,
                column: target.cell.column,
            },
        );
        update_position_request(
            placement_id,
            placement_move_request_from_direct(target.cell.row, target.cell.column),
            placements,
            dirty,
            save_error,
            announcement,
        );
        clear_placement_drag_intent(drag_preview, pending_drag_preview, drag_preview_timeout);
        dragged.set(None);
    });
    let on_cancel_drag = Callback::new(move |_| {
        clear_placement_drag_intent(drag_preview, pending_drag_preview, drag_preview_timeout);
    });

    view! {
        <section class="dashboard-editor__canvas-panel" aria-label="Dashboard composition canvas">
            <div class="dashboard-canvas__header">
                <div>
                    <span class="dashboard-editor__eyebrow">"Canvas"</span>
                    <h2>"12-column layout"</h2>
                </div>
                <span aria-live="polite">{move || format!("{} placements", visible.get().len())}</span>
            </div>
            <PlacementGridCanvas
                canvas_id="dashboard-composition".to_string()
                row_count=row_count
                dragging=dragging
                on_drag_target=on_drag_target
                on_drop_target=on_drop_target
                on_cancel_drag=on_cancel_drag
                on_click=Callback::new(move |_| {})
                class="dashboard-composition-canvas"
                stack_on_narrow=true
            >
                <PlacementGridGuides
                    canvas_id="dashboard-composition".to_string()
                    cells=grid_cells
                    cell_label=Callback::new(|cell: PlacementGridCell| {
                        format!("Dashboard row {}, column {}", cell.row, cell.column)
                    })
                />
                {move || if visible.get().is_empty() {
                    view! {
                        <div class="dashboard-canvas-empty">
                            <strong>"Add a Component to begin"</strong>
                            <p>"Open Components above the canvas. Components execute only in an explicit preview or saved Dashboard viewer."</p>
                        </div>
                    }.into_any()
                } else {
                    visible.get().into_iter().map(|editor| {
                        let key = editor.key().to_string();
                        let selected_key = key.clone();
                        let select_key = key.clone();
                        let focus_key = key.clone();
                        let select_dom_id = placement_select_dom_id(&key);
                        let drag_state_key = key.clone();
                        let drag_start_key = key.clone();
                        let may_move = editor_can_move(&editor);
                        let may_resize = editor_can_resize(&editor);
                        let placement = editor.placement;
                        let issue_target = placement.clone();
                        let title = placement.display_title();
                        let subtitle = placement.component.as_ref().map(|component| {
                            format!("{} · {}", kind_label(&component.component_type), version_label(component.version_number, &component.version_label))
                        });
                        let resolution_state = placement.effective_resolution_state();
                        let has_issue = resolution_state != DashboardPlacementResolutionState::Available;
                        let rect = GridRect::new(
                            placement.grid_row,
                            placement.grid_column,
                            placement.grid_width,
                            placement.grid_height,
                        );
                        let tile_rect = Signal::derive(move || Some(rect));
                        let tile_class = Signal::derive(move || {
                            if has_issue {
                                format!(
                                    "dashboard-composition-tile is-unavailable placement-state--{}",
                                    resolution_state.css_class()
                                )
                            } else {
                                "dashboard-composition-tile".to_string()
                            }
                        });
                        let selected_signal = Signal::derive(move || {
                            selected.get().as_deref() == Some(selected_key.as_str())
                        });
                        let dragging_signal = Signal::derive(move || {
                            dragged.get().as_deref() == Some(drag_state_key.as_str())
                        });
                        let resize_key = key.clone();
                        let resize_selected_key = key.clone();
                        view! {
                            <PlacementTileShell
                                rect=tile_rect
                                class=tile_class
                                selected=selected_signal
                                dragging=dragging_signal
                                draggable=may_move
                                on_select=Callback::new(move |_| selected.set(Some(select_key.clone())))
                                on_drag_start=Callback::new(move |event: ev::DragEvent| {
                                    if !may_move {
                                        event.prevent_default();
                                        return;
                                    }
                                    dragged.set(Some(drag_start_key.clone()));
                                    if let Some(transfer) = event.data_transfer() {
                                        let _ = transfer.set_data("text/plain", &drag_start_key);
                                    }
                                })
                                on_drag_enter=Callback::new(move |_| {})
                                on_drag_end=Callback::new(move |_| {
                                    clear_placement_drag_intent(
                                        drag_preview,
                                        pending_drag_preview,
                                        drag_preview_timeout,
                                    );
                                    dragged.set(None);
                                })
                            >
                                <header>
                                    <span class="dashboard-placement-card__order">{placement.position + 1}</span>
                                    <div>
                                        <strong>{title}</strong>
                                        {subtitle.map(|subtitle| view! { <span>{subtitle}</span> })}
                                    </div>
                                    <span class="dashboard-composition-tile__grip" aria-hidden="true">
                                        {may_move.then(|| view! { <GripVertical/> })}
                                    </span>
                                </header>
                                <button
                                    id=select_dom_id
                                    class="dashboard-composition-tile__select"
                                    type="button"
                                    aria-label=if has_issue {
                                        format!("Open issue details for placement {}", placement.position + 1)
                                    } else {
                                        format!("Select placement {}", placement.position + 1)
                                    }
                                    on:focus=move |_| selected.set(Some(focus_key.clone()))
                                    on:click=move |_| {
                                        selected.set(Some(key.clone()));
                                        if has_issue {
                                            issue_placement.set(Some(issue_target.clone()));
                                        }
                                    }
                                >
                                <span class="dashboard-composition-tile__symbol" aria-hidden="true">
                                    {if has_issue {
                                        view! { <TriangleAlert/> }.into_any()
                                    } else {
                                        kind_icon(placement.kind_label()).into_any()
                                    }}
                                </span>
                                </button>
                                <footer>{format!("{} × {}", placement.grid_width, placement.grid_height)}</footer>
                                {move || (may_resize
                                    && selected.get().as_deref() == Some(resize_selected_key.as_str()))
                                    .then(|| view! {
                                        <PlacementResizeHandles
                                            on_resize_start=Callback::new({
                                                let resize_key = resize_key.clone();
                                                move |start: PlacementResizeStart| {
                                                    #[cfg(all(feature = "hydrate", target_arch = "wasm32"))]
                                                    {
                                                        clear_active_dashboard_resize();
                                                        let cleanup = start_dashboard_placement_resize(
                                                            start,
                                                            resize_key.clone(),
                                                            placements,
                                                            dirty,
                                                            save_error,
                                                            announcement,
                                                        );
                                                        ACTIVE_DASHBOARD_RESIZE_CLEANUP.with(|active| {
                                                            *active.borrow_mut() = cleanup;
                                                        });
                                                    }
                                                    #[cfg(not(all(feature = "hydrate", target_arch = "wasm32")))]
                                                    start_dashboard_placement_resize(
                                                        start,
                                                        resize_key.clone(),
                                                        placements,
                                                        dirty,
                                                        save_error,
                                                        announcement,
                                                    );
                                                }
                                            })
                                            width_title="Resize placement width"
                                            height_title="Resize placement height"
                                            handle_class="dashboard-composition-resize-handle"
                                        />
                                    })}
                            </PlacementTileShell>
                        }
                    }).collect_view().into_any()
                }}
            </PlacementGridCanvas>
            <p class="dashboard-canvas__tip">"Tip: select a placement, then open Placement details above the canvas for precise keyboard-accessible movement and sizing."</p>
        </section>
    }
}

#[component]
fn PlacementInspector(
    placements: RwSignal<Vec<EditorPlacement>>,
    selected: RwSignal<Option<String>>,
    options: RwSignal<Option<DashboardComposition>>,
    dirty: RwSignal<bool>,
    save_error: RwSignal<Option<String>>,
    announcement: RwSignal<String>,
    preview_open: RwSignal<bool>,
) -> impl IntoView {
    view! {
        <section
            class="dashboard-editor__inspector"
            aria-label="Placement details"
        >
            {move || selected.get().and_then(|key| {
                placements.get().into_iter().find(|placement| !placement.removed && placement.key() == key)
            }).map(|editor| {
                let key = editor.key().to_string();
                let placement = editor.placement;
                let needs_repair = placement.config_state == Some(DashboardPlacementConfigState::NeedsRepair);
                let available_options = options.get().map(|composition| composition.available_component_versions).unwrap_or_default();
                let repair_enabled = needs_repair && editor.repair;
                let replaceable_redacted = matches!(
                    placement.config_state,
                    Some(DashboardPlacementConfigState::Valid | DashboardPlacementConfigState::Legacy)
                ) && !available_options.is_empty();
                let may_move = placement.allows(DashboardPlacementOperation::Move) || editor.client_key.is_some() || repair_enabled;
                let may_resize = placement.allows(DashboardPlacementOperation::Resize) || editor.client_key.is_some() || repair_enabled;
                let may_retitle = placement.allows(DashboardPlacementOperation::Retitle) || editor.client_key.is_some() || editor.replace_with.is_some();
                let may_replace = placement.allows(DashboardPlacementOperation::Replace) || editor.client_key.is_some() || replaceable_redacted;
                let may_preview = placement.allows(DashboardPlacementOperation::Preview) || editor.client_key.is_some() || editor.replace_with.is_some();
                let replacement_options = available_options.clone();
                let title_value = editor.requested_title.clone().flatten().or_else(|| placement.title.clone()).unwrap_or_default();
                let component_value = editor.replace_with.clone().or_else(|| placement.component.as_ref().map(|component| component.component_version_id.clone())).unwrap_or_default();
                let minimum = DashboardPlacementSizePolicy::new().minimum_for(
                    placement
                        .component
                        .as_ref()
                        .map(|component| component.component_type.as_str())
                        .unwrap_or_default(),
                );
                let maximum_height = dashboard_placement_height_limit(placement.grid_row);
                let exact_version_link = placement.component.as_ref().map(|component| {
                    let version_id = editor
                        .replace_with
                        .clone()
                        .unwrap_or_else(|| component.component_version_id.clone());
                    (
                        component_version_href(&component.component_slug, &version_id),
                        version_id,
                    )
                });
                let update_key = key.clone();
                let row_key = key.clone();
                let column_key = key.clone();
                let width_key = key.clone();
                let height_key = key.clone();
                let up_key = key.clone();
                let left_key = key.clone();
                let down_key = key.clone();
                let right_key = key.clone();
                let remove_key = key.clone();
                let replace_key = key.clone();
                let repair_key = key.clone();
                view! {
                    <div class="dashboard-inspector__content">
                        <section>
                            <span class="dashboard-editor__eyebrow">"Placement"</span>
                            <h3>{placement.display_title()}</h3>
                            <p>{format!("Reading order {}", placement.position + 1)}</p>
                        </section>

                        <label class="field-label" for="placement-title">"Title override"</label>
                        <input
                            id="placement-title"
                            class="text-input"
                            disabled=!may_retitle
                            prop:value=title_value
                            on:change=move |event| {
                                let value = event_target_value(&event);
                                mutate_editor(&update_key, placements, |editor| {
                                    editor.requested_title = Some((!value.trim().is_empty()).then_some(value.trim().to_string()));
                                });
                                dirty.set(true);
                                announcement.set("Placement title changed.".into());
                            }
                        />

                        <label class="field-label" for="placement-component">"Pinned Component version"</label>
                        <select
                            id="placement-component"
                            class="text-input"
                            disabled=!may_replace
                            prop:value=component_value
                            on:change=move |event| {
                                let value = event_target_value(&event);
                                let replacement = replacement_options
                                    .iter()
                                    .find(|option| option.component_version_id == value)
                                    .cloned();
                                mutate_editor(&replace_key, placements, |editor| {
                                    editor.replace_with = Some(value.clone());
                                    if let Some(replacement) = replacement.as_ref() {
                                        editor.placement.component = Some(option_to_component(replacement));
                                        editor.placement.availability = DashboardPlacementAvailability::Available;
                                    }
                                });
                                dirty.set(true);
                                announcement.set("Pinned Component version changed.".into());
                            }
                        >
                            <option value="" disabled>"Choose a Component version"</option>
                            {available_options.into_iter().map(|option| view! {
                                <option value=option.component_version_id>
                                    {format!("{} · {} · {}", option.component_name, kind_label(&option.component_type), version_label(option.version_number, &option.version_label))}
                                </option>
                            }).collect_view()}
                        </select>

                        {exact_version_link.map(|(href, version_id)| {
                            let accessible_label = format!("Open exact Component version {version_id}");
                            view! {
                                <a
                                    class="dashboard-inspector__version-link"
                                    href=href
                                    aria-label=accessible_label.clone()
                                    title=accessible_label
                                >
                                    <span>"Open exact Component version"</span>
                                    <code>{version_id}</code>
                                </a>
                            }
                        })}

                        <fieldset class="dashboard-inspector__geometry">
                            <legend>"Position and size"</legend>
                            <label>"Row" <input type="number" min="1" max="240" disabled=!may_move prop:value=placement.grid_row on:change=move |event| {
                                let input = event_target::<web_sys::HtmlInputElement>(&event);
                                if let Ok(row) = event_target_value(&event).parse() {
                                    update_position(row_key.clone(), row, current_geometry(&row_key, placements).map(|geometry| geometry.grid_column).unwrap_or(1), placements, dirty, save_error, announcement);
                                }
                                let canonical = current_geometry(&row_key, placements).map(|geometry| geometry.grid_row).unwrap_or(1);
                                input.set_value(&canonical.to_string());
                            }/></label>
                            <label>"Column" <input type="number" min="1" max="12" disabled=!may_move prop:value=placement.grid_column on:change=move |event| {
                                let input = event_target::<web_sys::HtmlInputElement>(&event);
                                if let Ok(column) = event_target_value(&event).parse() {
                                    update_position(column_key.clone(), current_geometry(&column_key, placements).map(|geometry| geometry.grid_row).unwrap_or(1), column, placements, dirty, save_error, announcement);
                                }
                                let canonical = current_geometry(&column_key, placements).map(|geometry| geometry.grid_column).unwrap_or(1);
                                input.set_value(&canonical.to_string());
                            }/></label>
                            <label>"Width" <input type="number" min=minimum.width max="12" disabled=!may_resize prop:value=placement.grid_width on:change=move |event| {
                                let input = event_target::<web_sys::HtmlInputElement>(&event);
                                if let Ok(width) = event_target_value(&event).parse() {
                                    update_size(width_key.clone(), width, current_geometry(&width_key, placements).map(|geometry| geometry.grid_height).unwrap_or(1), placements, dirty, save_error, announcement);
                                }
                                let canonical = current_geometry(&width_key, placements).map(|geometry| geometry.grid_width).unwrap_or(1);
                                input.set_value(&canonical.to_string());
                            }/></label>
                            <label>"Height" <input type="number" min=minimum.height max=maximum_height disabled=!may_resize prop:value=placement.grid_height on:change=move |event| {
                                let input = event_target::<web_sys::HtmlInputElement>(&event);
                                if let Ok(height) = event_target_value(&event).parse() {
                                    update_size(height_key.clone(), current_geometry(&height_key, placements).map(|geometry| geometry.grid_width).unwrap_or(1), height, placements, dirty, save_error, announcement);
                                }
                                let canonical = current_geometry(&height_key, placements).map(|geometry| geometry.grid_height).unwrap_or(1);
                                input.set_value(&canonical.to_string());
                            }/></label>
                        </fieldset>

                        <fieldset class="dashboard-inspector__nudge" disabled=!may_move>
                            <legend>"Move with arrow keys"</legend>
                            <button type="button" aria-label="Move placement up" on:click=move |_| nudge(&up_key, -1, 0, placements, dirty, save_error, announcement)><span aria-hidden="true"><ArrowUp/></span></button>
                            <button type="button" aria-label="Move placement left" on:click=move |_| nudge(&left_key, 0, -1, placements, dirty, save_error, announcement)><span aria-hidden="true"><ArrowLeft/></span></button>
                            <button type="button" aria-label="Move placement down" on:click=move |_| nudge(&down_key, 1, 0, placements, dirty, save_error, announcement)><span aria-hidden="true"><ArrowDown/></span></button>
                            <button type="button" aria-label="Move placement right" on:click=move |_| nudge(&right_key, 0, 1, placements, dirty, save_error, announcement)><span aria-hidden="true"><ArrowRight/></span></button>
                        </fieldset>

                        {needs_repair.then(|| view! {
                            <label class="dashboard-repair-control">
                                <input type="checkbox" prop:checked=editor.repair on:change=move |event| {
                                    mutate_editor(&repair_key, placements, |editor| editor.repair = event_target_checked(&event));
                                    dirty.set(true);
                                }/>
                                <span><strong>"Repair malformed placement config"</strong> "Write canonical V1 geometry on save."</span>
                            </label>
                        })}

                        <div class="dashboard-inspector__actions">
                            <button id="placement-preview-trigger" class="button button--secondary" type="button" disabled=!may_preview on:click=move |_| preview_open.set(true)>"Preview selected"</button>
                            <button class="button button--danger" type="button" on:click=move |_| remove_placement(&remove_key, placements, selected, dirty, announcement)>"Remove placement"</button>
                        </div>
                    </div>
                }.into_any()
            }).unwrap_or_else(|| view! {
                <div class="dashboard-inspector-empty">
                    <p>"Select a placement to resize, reposition, replace, repair, or preview it."</p>
                </div>
            }.into_any())}
        </section>
    }
}

#[component]
fn SelectedPlacementPreview(
    placements: RwSignal<Vec<EditorPlacement>>,
    selected: RwSignal<Option<String>>,
    preview_open: RwSignal<bool>,
) -> impl IntoView {
    let selected_placement = move || {
        let key = selected.get()?;
        placements
            .get()
            .into_iter()
            .find(|placement| !placement.removed && placement.key() == key)
    };
    view! {
        <ModalDialog
            id="selected-preview"
            title="Selected Component"
            description="On-demand preview"
            open=Signal::derive(move || preview_open.get())
            on_close=Callback::new(move |_| preview_open.set(false))
            close_label="Close"
            class="dashboard-preview-dialog"
        >
            {move || preview_open.get().then(|| selected_placement().and_then(|editor| {
                    let component = editor.placement.component?;
                    let version_id = editor.replace_with.unwrap_or(component.component_version_id);
                    let kind = ComponentVersionKind::from_api_kind(&component.component_type)?;
                    let target = ComponentVersionTarget::new(component.component_slug, version_id, kind);
                    Some(view! { <ComponentVersionExecutionContent target mode=ComponentViewerMode::Full/> })
                }).map(IntoAny::into_any).unwrap_or_else(|| view! {
                    <EmptyState title="Preview unavailable" message="This placement is redacted or has no supported executable Component."/>
                }.into_any()))}
        </ModalDialog>
    }
}

#[component]
fn DashboardMetadataSettings(
    dashboard_id: String,
    composition: RwSignal<Option<DashboardComposition>>,
    announcement: RwSignal<String>,
    nodes: RwSignal<Vec<VisibilityNodeOption>>,
    nodes_bootstrapped: bool,
    placements: RwSignal<Vec<EditorPlacement>>,
    layout_dirty: RwSignal<bool>,
    operation: RwSignal<EditorOperation>,
) -> impl IntoView {
    let open = RwSignal::new(false);
    let name = RwSignal::new(String::new());
    let description = RwSignal::new(String::new());
    let selected_nodes = RwSignal::new(Vec::<String>::new());
    let initialized = RwSignal::new(false);
    let error = RwSignal::new(None::<String>);

    Effect::new(move |_| {
        if !initialized.get()
            && let Some(loaded) = composition.get()
        {
            name.set(loaded.dashboard.name);
            description.set(loaded.dashboard.description.unwrap_or_default());
            selected_nodes.set(
                loaded
                    .dashboard
                    .visibility_nodes
                    .into_iter()
                    .map(|node| node.node_id)
                    .collect(),
            );
            initialized.set(true);
        }
    });
    Effect::new(move |_| {
        if open.get() && !nodes_bootstrapped && nodes.get_untracked().is_empty() {
            load_settings_nodes(nodes, error);
        }
    });

    view! {
        <details class="dashboard-settings" open=move || open.get() on:toggle=move |event| {
            open.set(event_target::<web_sys::HtmlDetailsElement>(&event).open());
        }>
            <summary>"Dashboard settings"</summary>
            <div class="dashboard-settings__panel">
                <label>"Name" <input class="text-input" prop:value=move || name.get() on:input=move |event| name.set(event_target_value(&event))/></label>
                <label>"Description" <textarea class="text-input" prop:value=move || description.get() on:input=move |event| description.set(event_target_value(&event))></textarea></label>
                <fieldset>
                    <legend>"Visibility"</legend>
                    {move || nodes.get().into_iter().map(|node| {
                        let node_id = node.id.clone();
                        let checked_id = node.id.clone();
                        view! {
                            <label class="dashboard-scope-option">
                                <input type="checkbox" prop:checked=move || selected_nodes.get().contains(&checked_id) on:change=move |event| {
                                    let mut selected = selected_nodes.get_untracked();
                                    if event_target_checked(&event) {
                                        if !selected.contains(&node_id) { selected.push(node_id.clone()); }
                                    } else {
                                        selected.retain(|id| id != &node_id);
                                    }
                                    selected_nodes.set(selected);
                                }/>
                                {node.label()}
                            </label>
                        }
                    }).collect_view()}
                </fieldset>
                {move || error.get().map(|message| view! { <p class="form-error">{message}</p> })}
                {move || layout_dirty.get().then(|| view! {
                    <p class="field-help">"Save or discard layout changes before changing Dashboard scope."</p>
                })}
                <button class="button" type="button" disabled=move || operation.get().is_busy() || layout_dirty.get() on:click={
                    let dashboard_id = dashboard_id.clone();
                    move |_| {
                        if operation.get_untracked().is_busy() || layout_dirty.get_untracked() {
                            return;
                        }
                        let payload = DashboardMetadataRequest {
                            name: name.get_untracked().trim().to_string(),
                            description: {
                                let description = description.get_untracked().trim().to_string();
                                (!description.is_empty()).then_some(description)
                            },
                            visibility_node_ids: selected_nodes.get_untracked(),
                        };
                        save_settings(
                            dashboard_id.clone(),
                            payload,
                            SettingsSaveContext {
                                composition,
                                placements,
                                operation,
                                error,
                                announcement,
                                open,
                            },
                        );
                    }
                }>{move || if operation.get() == EditorOperation::SavingSettings {
                    "Saving…"
                } else {
                    "Save settings"
                }}</button>
            </div>
        </details>
    }
}

fn add_component(
    option: DashboardComponentVersionOption,
    placements: RwSignal<Vec<EditorPlacement>>,
    selected: RwSignal<Option<String>>,
    dirty: RwSignal<bool>,
    error: RwSignal<Option<String>>,
    announcement: RwSignal<String>,
    next_client_key: RwSignal<u64>,
) {
    let mut current = placements.get_untracked();
    let active_count = current
        .iter()
        .filter(|placement| !placement.removed)
        .count();
    if active_count >= 240 {
        error.set(Some("A Dashboard can store at most 240 placements.".into()));
        return;
    }
    let requested = GridSize::new(option.default_grid_width, option.default_grid_height);
    let Some(rect) = first_available_rect(&current, requested) else {
        error.set(Some(
            "No valid grid space remains for this Component size.".into(),
        ));
        return;
    };
    let sequence = next_client_key.get_untracked();
    next_client_key.set(sequence + 1);
    let client_key = format!("new-{sequence}");
    let placement = DashboardPlacement {
        placement_id: format!("client:{client_key}"),
        position: i32::try_from(active_count).unwrap_or(i32::MAX),
        grid_row: rect.row,
        grid_column: rect.column,
        grid_width: rect.width,
        grid_height: rect.height,
        availability: DashboardPlacementAvailability::Available,
        resolution_state: Some(DashboardPlacementResolutionState::Available),
        config_state: Some(DashboardPlacementConfigState::Valid),
        title: None,
        component: Some(option_to_component(&option)),
        allowed_operations: Some(vec![
            DashboardPlacementOperation::Retain,
            DashboardPlacementOperation::Move,
            DashboardPlacementOperation::Resize,
            DashboardPlacementOperation::Retitle,
            DashboardPlacementOperation::Replace,
            DashboardPlacementOperation::Preview,
            DashboardPlacementOperation::Remove,
        ]),
    };
    current.push(EditorPlacement {
        placement,
        client_key: Some(client_key.clone()),
        removed: false,
        requested_title: None,
        replace_with: None,
        repair: false,
    });
    canonicalize_positions(&mut current);
    placements.set(current);
    selected.set(Some(client_key));
    dirty.set(true);
    error.set(None);
    announcement.set(format!("{} added to the layout.", option.component_name));
}

fn editor_can_move(editor: &EditorPlacement) -> bool {
    editor.client_key.is_some()
        || editor.placement.allows(DashboardPlacementOperation::Move)
        || (editor.placement.config_state == Some(DashboardPlacementConfigState::NeedsRepair)
            && editor.repair)
}

fn editor_can_resize(editor: &EditorPlacement) -> bool {
    editor.client_key.is_some()
        || editor.placement.allows(DashboardPlacementOperation::Resize)
        || (editor.placement.config_state == Some(DashboardPlacementConfigState::NeedsRepair)
            && editor.repair)
}

fn update_position(
    key: String,
    row: i32,
    column: i32,
    placements: RwSignal<Vec<EditorPlacement>>,
    dirty: RwSignal<bool>,
    error: RwSignal<Option<String>>,
    announcement: RwSignal<String>,
) {
    update_position_request(
        key,
        placement_move_request_from_direct(row, column),
        placements,
        dirty,
        error,
        announcement,
    );
}

fn update_position_request(
    key: String,
    request: GridMoveRequest,
    placements: RwSignal<Vec<EditorPlacement>>,
    dirty: RwSignal<bool>,
    error: RwSignal<Option<String>>,
    announcement: RwSignal<String>,
) {
    let mut current = placements.get_untracked();
    if !current
        .iter()
        .find(|placement| placement.key() == key)
        .is_some_and(editor_can_move)
    {
        error.set(Some(
            "This placement cannot be moved in its current state.".into(),
        ));
        return;
    }
    let layout = active_layout(&current);
    let Some(current_rect) = layout
        .iter()
        .find(|placement| placement.id == key)
        .map(|placement| placement.rect)
    else {
        error.set(Some(
            "Placement no longer exists in the local layout.".into(),
        ));
        return;
    };
    let requested = match resolve_move_request(DASHBOARD_GRID_CONSTRAINTS, current_rect, request) {
        Ok(requested) => requested,
        Err(cause) => {
            error.set(Some(format!("Placement could not move: {cause}")));
            return;
        }
    };
    match reflow_dashboard_movement(&layout, &key, requested.row, requested.column) {
        Ok(reflowed) => {
            let moves_protected_placement = reflowed.iter().any(|moved| {
                current
                    .iter()
                    .find(|placement| placement.key() == moved.id.as_str())
                    .is_some_and(|editor| {
                        let geometry_changed = editor.placement.grid_row != moved.rect.row
                            || editor.placement.grid_column != moved.rect.column;
                        geometry_changed && !editor_can_move(editor)
                    })
            });
            if moves_protected_placement {
                error.set(Some(
                    "That move would displace a placement which must be retained unchanged.".into(),
                ));
                return;
            }
            for moved in reflowed {
                mutate_editor_in(&mut current, &moved.id, |editor| {
                    editor.placement.grid_row = moved.rect.row;
                    editor.placement.grid_column = moved.rect.column;
                });
            }
            canonicalize_positions(&mut current);
            placements.set(current);
            dirty.set(true);
            error.set(None);
            announcement.set(format!(
                "Placement moved to row {}, column {}.",
                requested.row, requested.column
            ));
        }
        Err(cause) => error.set(Some(format!("Placement could not move: {cause}"))),
    }
}

fn update_size(
    key: String,
    width: i32,
    height: i32,
    placements: RwSignal<Vec<EditorPlacement>>,
    dirty: RwSignal<bool>,
    error: RwSignal<Option<String>>,
    announcement: RwSignal<String>,
) {
    update_size_request(
        key,
        placement_resize_request_from_direct(width, height),
        placements,
        dirty,
        error,
        announcement,
    );
}

fn dashboard_placement_height_limit(grid_row: i32) -> i32 {
    DASHBOARD_GRID_CONSTRAINTS
        .max_rows()
        .saturating_sub(grid_row)
        .saturating_add(1)
        .clamp(1, DASHBOARD_GRID_CONSTRAINTS.max_rows())
}

fn update_size_request(
    key: String,
    request: GridResizeRequest,
    placements: RwSignal<Vec<EditorPlacement>>,
    dirty: RwSignal<bool>,
    error: RwSignal<Option<String>>,
    announcement: RwSignal<String>,
) {
    let mut current = placements.get_untracked();
    if !current
        .iter()
        .find(|placement| placement.key() == key)
        .is_some_and(editor_can_resize)
    {
        error.set(Some(
            "This placement cannot be resized in its current state.".into(),
        ));
        return;
    }
    let layout = active_layout(&current);
    let kind = current
        .iter()
        .find(|placement| placement.key() == key)
        .map(|placement| placement.placement.kind_label().to_string())
        .unwrap_or_default();
    let Some(current_rect) = layout
        .iter()
        .find(|placement| placement.id == key)
        .map(|placement| placement.rect)
    else {
        error.set(Some(
            "Placement no longer exists in the local layout.".into(),
        ));
        return;
    };
    let policy = DashboardPlacementSizePolicy::new();
    let requested = match resolve_resize_request(
        DASHBOARD_GRID_CONSTRAINTS,
        current_rect,
        request,
        policy.minimum_for(&kind),
    ) {
        Ok(size) => size,
        Err(cause) => {
            error.set(Some(format!("Placement size was not changed: {cause}")));
            return;
        }
    };
    if requested.width == current_rect.width && requested.height == current_rect.height {
        error.set(None);
        return;
    }
    match validate_dashboard_resize(&layout, &key, requested, &kind, &policy) {
        Ok(resized) => {
            mutate_editor_in(&mut current, &key, |editor| {
                editor.placement.grid_width = resized.rect.width;
                editor.placement.grid_height = resized.rect.height;
            });
            canonicalize_positions(&mut current);
            placements.set(current);
            dirty.set(true);
            error.set(None);
            announcement.set(format!(
                "Placement resized to {} by {}.",
                requested.width, requested.height
            ));
        }
        Err(cause) => error.set(Some(format!("Placement size was not changed: {cause}"))),
    }
}

fn nudge(
    key: &str,
    row_delta: i32,
    column_delta: i32,
    placements: RwSignal<Vec<EditorPlacement>>,
    dirty: RwSignal<bool>,
    error: RwSignal<Option<String>>,
    announcement: RwSignal<String>,
) {
    let direction = match (row_delta, column_delta) {
        (-1, 0) => GridMoveDirection::Up,
        (1, 0) => GridMoveDirection::Down,
        (0, -1) => GridMoveDirection::Left,
        (0, 1) => GridMoveDirection::Right,
        _ => return,
    };
    update_position_request(
        key.to_string(),
        placement_move_request_from_keyboard(direction),
        placements,
        dirty,
        error,
        announcement,
    );
}

#[cfg(all(feature = "hydrate", target_arch = "wasm32"))]
type MouseEventCallback = Rc<RefCell<Option<Closure<dyn FnMut(web_sys::MouseEvent)>>>>;
#[cfg(all(feature = "hydrate", target_arch = "wasm32"))]
type EventCallback = Rc<RefCell<Option<Closure<dyn FnMut(web_sys::Event)>>>>;
#[cfg(all(feature = "hydrate", target_arch = "wasm32"))]
type DashboardResizeCleanup = Box<dyn FnOnce()>;

#[cfg(all(feature = "hydrate", target_arch = "wasm32"))]
fn clear_active_dashboard_resize() {
    let cleanup = ACTIVE_DASHBOARD_RESIZE_CLEANUP.with(|active| active.borrow_mut().take());
    if let Some(cleanup) = cleanup {
        cleanup();
    }
}

#[cfg(all(feature = "hydrate", target_arch = "wasm32"))]
fn remove_dashboard_resize_listeners(
    move_callback: &MouseEventCallback,
    up_callback: &MouseEventCallback,
    blur_callback: &EventCallback,
) {
    if let Some(window) = web_sys::window() {
        if let Some(callback) = move_callback.borrow().as_ref() {
            let _ = window.remove_event_listener_with_callback(
                "mousemove",
                callback.as_ref().unchecked_ref(),
            );
        }
        if let Some(callback) = up_callback.borrow().as_ref() {
            let _ = window
                .remove_event_listener_with_callback("mouseup", callback.as_ref().unchecked_ref());
        }
        if let Some(callback) = blur_callback.borrow().as_ref() {
            let _ = window
                .remove_event_listener_with_callback("blur", callback.as_ref().unchecked_ref());
        }
    }
}

#[cfg(all(feature = "hydrate", target_arch = "wasm32"))]
fn start_dashboard_placement_resize(
    start: PlacementResizeStart,
    placement_id: String,
    placements: RwSignal<Vec<EditorPlacement>>,
    dirty: RwSignal<bool>,
    error: RwSignal<Option<String>>,
    announcement: RwSignal<String>,
) -> Option<DashboardResizeCleanup> {
    let event = start.event;
    event.prevent_default();
    event.stop_propagation();
    let Some(window) = web_sys::window() else {
        return None;
    };
    if window
        .match_media("(max-width: 767px)")
        .ok()
        .flatten()
        .is_some_and(|query| query.matches())
    {
        return None;
    }
    let Some(target) = event
        .target()
        .and_then(|target| target.dyn_into::<web_sys::Element>().ok())
    else {
        return None;
    };
    let Some(tile) = target.closest(".placement-editor-tile").ok().flatten() else {
        return None;
    };
    let Some(grid) = target
        .closest("[data-placement-grid-canvas]")
        .ok()
        .flatten()
    else {
        return None;
    };
    let Some(metrics) = placement_grid_metrics_from_element(&grid) else {
        return None;
    };
    let current = placements.get_untracked();
    let Some(editor) = current
        .iter()
        .find(|placement| !placement.removed && placement.key() == placement_id)
    else {
        return None;
    };
    let start_rect = GridRect::new(
        editor.placement.grid_row,
        editor.placement.grid_column,
        editor.placement.grid_width,
        editor.placement.grid_height,
    );
    let kind = editor.placement.kind_label().to_string();
    let session = PlacementResizeSession::new(
        start_rect,
        start.axis,
        f64::from(event.client_x()),
        f64::from(event.client_y()),
    );
    let active = Rc::new(Cell::new(true));
    let last_width = Rc::new(Cell::new(start_rect.width));
    let last_height = Rc::new(Cell::new(start_rect.height));
    let has_valid_change = Rc::new(Cell::new(false));
    let _ = tile.class_list().add_1("is-resizing");

    let move_callback: MouseEventCallback = Rc::new(RefCell::new(None));
    let up_callback: MouseEventCallback = Rc::new(RefCell::new(None));
    let blur_callback: EventCallback = Rc::new(RefCell::new(None));

    let active_for_move = active.clone();
    let tile_for_move = tile.clone();
    let width_for_move = last_width.clone();
    let height_for_move = last_height.clone();
    let changed_for_move = has_valid_change.clone();
    let placement_id_for_move = placement_id.clone();
    *move_callback.borrow_mut() = Some(Closure::wrap(Box::new(move |event: web_sys::MouseEvent| {
        if !active_for_move.get() {
            return;
        }
        event.prevent_default();
        let policy = DashboardPlacementSizePolicy::new();
        let request = session.request_at(
            metrics,
            f64::from(event.client_x()),
            f64::from(event.client_y()),
        );
        let Ok(size) = resolve_resize_request(
            DASHBOARD_GRID_CONSTRAINTS,
            start_rect,
            request,
            policy.minimum_for(&kind),
        ) else {
            return;
        };
        let layout = active_layout(&placements.get_untracked());
        let Ok(resized) =
            validate_dashboard_resize(&layout, &placement_id_for_move, size, &kind, &policy)
        else {
            return;
        };
        width_for_move.set(resized.rect.width);
        height_for_move.set(resized.rect.height);
        changed_for_move.set(resized.rect != start_rect);
        let _ = tile_for_move.set_attribute("style", &placement_tile_style(resized.rect));
    }) as Box<dyn FnMut(_)>));

    let active_for_up = active.clone();
    let tile_for_up = tile.clone();
    let width_for_up = last_width.clone();
    let height_for_up = last_height.clone();
    let changed_for_up = has_valid_change.clone();
    let move_callback_for_up = move_callback.clone();
    let up_callback_for_up = up_callback.clone();
    let blur_callback_for_up = blur_callback.clone();
    *up_callback.borrow_mut() = Some(Closure::wrap(Box::new(move |event: web_sys::MouseEvent| {
        if !active_for_up.replace(false) {
            return;
        }
        event.prevent_default();
        let _ = tile_for_up.class_list().remove_1("is-resizing");
        if changed_for_up.get() {
            update_size(
                placement_id.clone(),
                width_for_up.get(),
                height_for_up.get(),
                placements,
                dirty,
                error,
                announcement,
            );
        } else {
            let _ = tile_for_up.set_attribute("style", &placement_tile_style(start_rect));
        }
        remove_dashboard_resize_listeners(
            &move_callback_for_up,
            &up_callback_for_up,
            &blur_callback_for_up,
        );
        move_callback_for_up.borrow_mut().take();
        up_callback_for_up.borrow_mut().take();
        blur_callback_for_up.borrow_mut().take();
    }) as Box<dyn FnMut(_)>));

    let active_for_blur = active.clone();
    let tile_for_blur = tile.clone();
    let move_callback_for_blur = move_callback.clone();
    let up_callback_for_blur = up_callback.clone();
    let blur_callback_for_blur = blur_callback.clone();
    *blur_callback.borrow_mut() = Some(Closure::wrap(Box::new(move |_: web_sys::Event| {
        if !active_for_blur.replace(false) {
            return;
        }
        let _ = tile_for_blur.class_list().remove_1("is-resizing");
        let _ = tile_for_blur.set_attribute("style", &placement_tile_style(start_rect));
        remove_dashboard_resize_listeners(
            &move_callback_for_blur,
            &up_callback_for_blur,
            &blur_callback_for_blur,
        );
        move_callback_for_blur.borrow_mut().take();
        up_callback_for_blur.borrow_mut().take();
        blur_callback_for_blur.borrow_mut().take();
    }) as Box<dyn FnMut(_)>));

    if let Some(callback) = move_callback.borrow().as_ref() {
        let _ =
            window.add_event_listener_with_callback("mousemove", callback.as_ref().unchecked_ref());
    }
    if let Some(callback) = up_callback.borrow().as_ref() {
        let _ =
            window.add_event_listener_with_callback("mouseup", callback.as_ref().unchecked_ref());
    }
    if let Some(callback) = blur_callback.borrow().as_ref() {
        let _ = window.add_event_listener_with_callback("blur", callback.as_ref().unchecked_ref());
    }

    Some(Box::new(move || {
        if active.replace(false) {
            let _ = tile.class_list().remove_1("is-resizing");
            let _ = tile.set_attribute("style", &placement_tile_style(start_rect));
        }
        remove_dashboard_resize_listeners(&move_callback, &up_callback, &blur_callback);
        move_callback.borrow_mut().take();
        up_callback.borrow_mut().take();
        blur_callback.borrow_mut().take();
    }))
}

#[cfg(not(all(feature = "hydrate", target_arch = "wasm32")))]
fn start_dashboard_placement_resize(
    _: PlacementResizeStart,
    _: String,
    _: RwSignal<Vec<EditorPlacement>>,
    _: RwSignal<bool>,
    _: RwSignal<Option<String>>,
    _: RwSignal<String>,
) {
}

fn remove_placement(
    key: &str,
    placements: RwSignal<Vec<EditorPlacement>>,
    selected: RwSignal<Option<String>>,
    dirty: RwSignal<bool>,
    announcement: RwSignal<String>,
) {
    let mut current = placements.get_untracked();
    if let Some(index) = current.iter().position(|placement| placement.key() == key) {
        let removed_active_index = current
            .iter()
            .filter(|placement| !placement.removed)
            .position(|placement| placement.key() == key)
            .unwrap_or_default();
        if current[index].client_key.is_some() {
            current.remove(index);
        } else {
            current[index].removed = true;
        }
        canonicalize_positions(&mut current);
        let next = selection_after_removal(&current, removed_active_index);
        placements.set(current);
        selected.set(next.clone());
        dirty.set(true);
        if next.is_some() {
            announcement.set("Placement removed. Focus moved to the next placement.".into());
        } else {
            announcement.set("Placement removed. The layout is now empty.".into());
        }
        focus_after_removal(next);
    }
}

fn selection_after_removal(
    placements: &[EditorPlacement],
    removed_active_index: usize,
) -> Option<String> {
    let active = placements
        .iter()
        .filter(|placement| !placement.removed)
        .collect::<Vec<_>>();
    active
        .get(removed_active_index)
        .or_else(|| active.last())
        .map(|placement| placement.key().to_string())
}

fn placement_select_dom_id(key: &str) -> String {
    let suffix = key
        .chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() || matches!(character, '-' | '_') {
                character
            } else {
                '-'
            }
        })
        .collect::<String>();
    format!("dashboard-placement-select-{suffix}")
}

#[cfg(all(feature = "hydrate", target_arch = "wasm32"))]
fn focus_after_removal(next: Option<String>) {
    let callback = Closure::once(move || {
        let Some(document) = web_sys::window().and_then(|window| window.document()) else {
            return;
        };
        let target = next
            .as_deref()
            .and_then(|key| document.get_element_by_id(&placement_select_dom_id(key)))
            .or_else(|| document.get_element_by_id("dashboard-placement-details-trigger"))
            .and_then(|element| element.dyn_into::<web_sys::HtmlElement>().ok());
        if let Some(target) = target {
            let _ = target.focus();
        }
    });
    if let Some(window) = web_sys::window() {
        let _ = window.set_timeout_with_callback_and_timeout_and_arguments_0(
            callback.as_ref().unchecked_ref(),
            0,
        );
        callback.forget();
    }
}

#[cfg(not(all(feature = "hydrate", target_arch = "wasm32")))]
fn focus_after_removal(_: Option<String>) {}

fn first_available_rect(placements: &[EditorPlacement], size: GridSize) -> Option<GridRect> {
    let layout = active_layout(placements);
    for row in 1..=240 {
        for column in 1..=12 {
            let rect = GridRect::new(row, column, size.width, size.height);
            let candidate = GridPlacement::new("__candidate__".to_string(), rect);
            let mut with_candidate = layout.clone();
            with_candidate.push(candidate);
            if validate_dashboard_layout(&with_candidate).is_ok() {
                return Some(rect);
            }
        }
    }
    None
}

fn active_layout(placements: &[EditorPlacement]) -> Vec<GridPlacement<String>> {
    placements
        .iter()
        .filter(|placement| !placement.removed)
        .map(|placement| {
            GridPlacement::new(
                placement.key().to_string(),
                GridRect::new(
                    placement.placement.grid_row,
                    placement.placement.grid_column,
                    placement.placement.grid_width,
                    placement.placement.grid_height,
                ),
            )
        })
        .collect()
}

fn canonicalize_positions(placements: &mut [EditorPlacement]) {
    let mut ordered = placements
        .iter()
        .enumerate()
        .filter(|(_, placement)| !placement.removed)
        .map(|(index, placement)| {
            (
                placement.placement.grid_row,
                placement.placement.grid_column,
                placement.key().to_string(),
                index,
            )
        })
        .collect::<Vec<_>>();
    ordered.sort();
    for (position, (_, _, _, index)) in ordered.into_iter().enumerate() {
        placements[index].placement.position = i32::try_from(position).unwrap_or(i32::MAX);
    }
}

fn current_geometry(
    key: &str,
    placements: RwSignal<Vec<EditorPlacement>>,
) -> Option<DashboardPlacementGeometry> {
    placements
        .get_untracked()
        .iter()
        .find(|placement| placement.key() == key)
        .map(EditorPlacement::geometry)
}

fn mutate_editor(
    key: &str,
    placements: RwSignal<Vec<EditorPlacement>>,
    mutate: impl FnOnce(&mut EditorPlacement),
) {
    let mut current = placements.get_untracked();
    mutate_editor_in(&mut current, key, mutate);
    placements.set(current);
}

fn mutate_editor_in(
    placements: &mut [EditorPlacement],
    key: &str,
    mutate: impl FnOnce(&mut EditorPlacement),
) {
    if let Some(placement) = placements
        .iter_mut()
        .find(|placement| placement.key() == key)
    {
        mutate(placement);
    }
}

fn option_to_component(option: &DashboardComponentVersionOption) -> DashboardComponentVersion {
    DashboardComponentVersion {
        component_version_id: option.component_version_id.clone(),
        component_id: option.component_id.clone(),
        component_name: option.component_name.clone(),
        component_slug: option.component_slug.clone(),
        component_type: option.component_type.clone(),
        version_number: option.version_number,
        version_label: option.version_label.clone(),
        version_status: option.version_status.clone(),
    }
}

fn editor_reader_actions_visible(account: Option<&SessionAccount>) -> bool {
    account.is_some_and(SessionAccount::can_read_dashboards)
}

fn component_version_href(component_slug: &str, component_version_id: &str) -> String {
    format!("/components/{component_slug}/versions?version={component_version_id}")
}

fn build_reconcile_request(
    placements: &[EditorPlacement],
) -> Result<ReconcileDashboardCompositionRequest, String> {
    let mut seen_existing = BTreeSet::new();
    let mut seen_clients = BTreeSet::new();
    let mut commands = Vec::new();
    for editor in placements {
        if let Some(client_key) = &editor.client_key {
            if editor.removed {
                continue;
            }
            if !seen_clients.insert(client_key.clone()) {
                return Err("New placement correlation keys must be unique.".into());
            }
            let component_version_id = editor
                .replace_with
                .clone()
                .or_else(|| {
                    editor
                        .placement
                        .component
                        .as_ref()
                        .map(|component| component.component_version_id.clone())
                })
                .ok_or_else(|| {
                    "New placements require a readable Component version.".to_string()
                })?;
            commands.push(DashboardCompositionCommand::Bind {
                placement_id: None,
                client_key: Some(client_key.clone()),
                component_version_id,
                geometry: editor.geometry(),
                title: requested_title(&editor.requested_title),
            });
            continue;
        }

        let placement_id = editor.placement.placement_id.clone();
        if !seen_existing.insert(placement_id.clone()) {
            return Err("Existing placement ids must be unique.".into());
        }
        if editor.removed {
            commands.push(DashboardCompositionCommand::Remove { placement_id });
        } else if let Some(component_version_id) = &editor.replace_with {
            commands.push(DashboardCompositionCommand::Bind {
                placement_id: Some(placement_id),
                client_key: None,
                component_version_id: component_version_id.clone(),
                geometry: editor.geometry(),
                title: requested_title(&editor.requested_title),
            });
        } else {
            commands.push(DashboardCompositionCommand::Retain {
                placement_id,
                geometry: editor.geometry(),
                title: requested_title(&editor.requested_title),
                repair: editor.repair,
            });
        }
    }
    Ok(ReconcileDashboardCompositionRequest { commands })
}

fn requested_title(request: &Option<Option<String>>) -> Option<String> {
    match request {
        None => None,
        Some(Some(title)) => Some(title.clone()),
        Some(None) => Some(String::new()),
    }
}

fn kind_icon(kind: &str) -> AnyView {
    match kind {
        "table" => view! { <Table2/> }.into_any(),
        "bar" => view! { <ChartBar/> }.into_any(),
        "line" => view! { <ChartLine/> }.into_any(),
        "pie" => view! { <ChartPie/> }.into_any(),
        "donut" => view! { <Donut/> }.into_any(),
        "stat_card" => view! { <SquareActivity/> }.into_any(),
        _ => view! { <Table2/> }.into_any(),
    }
}

#[cfg(test)]
mod tests {
    #[cfg(all(feature = "ssr", not(feature = "hydrate")))]
    use super::CompositionCanvas;
    use super::{
        build_reconcile_request, component_version_href, dashboard_placement_height_limit,
        editor_can_resize, editor_reader_actions_visible, first_available_rect,
        placement_select_dom_id, selection_after_removal,
    };
    use crate::types::{
        DashboardComponentVersion, DashboardPlacement, DashboardPlacementAvailability,
        DashboardPlacementConfigState, DashboardPlacementOperation,
        DashboardPlacementResolutionState, EditorPlacement, SessionAccount,
    };
    #[cfg(all(feature = "ssr", not(feature = "hydrate")))]
    use leptos::prelude::*;
    use tessara_dashboards::GridSize;

    fn placement(id: &str, row: i32, column: i32, width: i32, height: i32) -> EditorPlacement {
        EditorPlacement::existing(DashboardPlacement {
            placement_id: id.into(),
            position: 0,
            grid_row: row,
            grid_column: column,
            grid_width: width,
            grid_height: height,
            availability: DashboardPlacementAvailability::Available,
            resolution_state: Some(DashboardPlacementResolutionState::Available),
            config_state: Some(DashboardPlacementConfigState::Valid),
            title: None,
            component: Some(DashboardComponentVersion {
                component_version_id: "version".into(),
                component_id: "component".into(),
                component_name: "Component".into(),
                component_slug: "component".into(),
                component_type: "table".into(),
                version_number: 1,
                version_label: "Published".into(),
                version_status: "published".into(),
            }),
            allowed_operations: Some(vec![
                DashboardPlacementOperation::Retain,
                DashboardPlacementOperation::Move,
                DashboardPlacementOperation::Resize,
            ]),
        })
    }

    #[test]
    fn add_finds_next_non_overlapping_default_footprint() {
        let placements = vec![placement("one", 1, 1, 6, 4)];
        assert_eq!(
            first_available_rect(&placements, GridSize::new(6, 4)),
            Some(tessara_dashboards::GridRect::new(1, 7, 6, 4))
        );
    }

    #[test]
    fn height_limit_tracks_the_rows_remaining_below_the_placement() {
        assert_eq!(dashboard_placement_height_limit(1), 240);
        assert_eq!(dashboard_placement_height_limit(2), 239);
        assert_eq!(dashboard_placement_height_limit(240), 1);
    }

    #[test]
    fn needs_repair_placement_only_enables_resize_after_explicit_opt_in() {
        let mut placement = placement("table", 1, 1, 12, 1);
        placement.placement.config_state = Some(DashboardPlacementConfigState::NeedsRepair);
        placement.placement.allowed_operations = Some(vec![DashboardPlacementOperation::Retain]);

        assert!(!editor_can_resize(&placement));
        placement.repair = true;
        assert!(editor_can_resize(&placement));
    }

    #[test]
    fn reconcile_keeps_every_existing_identity_once() {
        let placements = vec![placement("one", 1, 1, 6, 4), placement("two", 1, 7, 6, 4)];
        let request = build_reconcile_request(&placements).expect("request");
        assert_eq!(request.commands.len(), 2);
        let json = serde_json::to_value(request).expect("serialize");
        assert_eq!(json["commands"][0]["placement_id"], "one");
        assert_eq!(json["commands"][1]["placement_id"], "two");
    }

    #[test]
    fn removal_selects_the_following_tile_then_falls_back_to_the_previous_tile() {
        let mut placements = vec![
            placement("one", 1, 1, 4, 2),
            placement("two", 1, 5, 4, 2),
            placement("three", 1, 9, 4, 2),
        ];
        placements[1].removed = true;
        assert_eq!(
            selection_after_removal(&placements, 1).as_deref(),
            Some("three")
        );

        placements[1].removed = false;
        placements[2].removed = true;
        assert_eq!(
            selection_after_removal(&placements, 2).as_deref(),
            Some("two")
        );
    }

    #[test]
    fn editor_reader_actions_require_read_even_when_manage_is_present() {
        let manager = SessionAccount {
            capabilities: vec!["dashboards:manage".into()],
        };
        let reader = SessionAccount {
            capabilities: vec!["dashboards:read".into()],
        };

        assert!(!editor_reader_actions_visible(Some(&manager)));
        assert!(editor_reader_actions_visible(Some(&reader)));
        assert!(!editor_reader_actions_visible(None));
    }

    #[test]
    fn exact_version_link_and_focus_id_preserve_stable_identity() {
        assert_eq!(
            component_version_href("delivery-health", "version-42"),
            "/components/delivery-health/versions?version=version-42"
        );
        assert_eq!(
            placement_select_dom_id("placement:42"),
            "dashboard-placement-select-placement-42"
        );
    }

    #[cfg(all(feature = "ssr", not(feature = "hydrate")))]
    #[test]
    fn every_degraded_editor_tile_uses_warning_icon_without_inline_diagnostic_copy() {
        let _ = any_spawner::Executor::init_futures_executor();
        for state in [
            DashboardPlacementResolutionState::Restricted,
            DashboardPlacementResolutionState::ProviderUnavailable,
            DashboardPlacementResolutionState::Inactive,
            DashboardPlacementResolutionState::Superseded,
            DashboardPlacementResolutionState::Tombstoned,
            DashboardPlacementResolutionState::OwnerTombstoned,
            DashboardPlacementResolutionState::OwnerDataDestroyed,
            DashboardPlacementResolutionState::Missing,
            DashboardPlacementResolutionState::Incompatible,
            DashboardPlacementResolutionState::NotEvaluated,
        ] {
            let mut degraded = placement("one", 1, 1, 6, 4);
            degraded.placement.availability = DashboardPlacementAvailability::Unavailable;
            degraded.placement.resolution_state = Some(state);
            degraded.placement.title = Some("Partner Profile".into());
            degraded.placement.component = None;
            let html = Owner::new().with(|| {
                view! {
                    <CompositionCanvas
                        placements=RwSignal::new(vec![degraded])
                        selected=RwSignal::new(None)
                        issue_placement=RwSignal::new(None)
                        dirty=RwSignal::new(false)
                        save_error=RwSignal::new(None)
                        announcement=RwSignal::new(String::new())
                    />
                }
                .to_html()
            });

            assert!(
                html.contains(&format!("placement-state--{}", state.css_class())),
                "{state:?}: {html}"
            );
            assert!(html.contains("Open issue details for placement 1"));
            assert!(html.contains("Partner Profile"));
            assert!(!html.contains("Redacted placement"));
            assert!(!html.contains(state.message()));
            assert!(!html.contains("Retry resolution"));
        }
    }
}
