#[cfg(feature = "hydrate")]
pub(crate) use crate::api::client::{redirect_to_login, send_json_request};
#[cfg(feature = "hydrate")]
pub(crate) use std::{cell::Cell, cell::RefCell, rc::Rc};
#[cfg(feature = "hydrate")]
pub(crate) use wasm_bindgen::JsCast;

use crate::ui::components::{
    AppShell, Breadcrumb, BreadcrumbItem, BreadcrumbLink, BreadcrumbPage, BreadcrumbSeparator,
    Button, DataTable, DropdownMenu, EmptyState, InfoListTable, InfoRow, PageHeader,
    SearchableDataTable, StatusBadge, Tabs, TabsContent, TabsList, TabsTrigger, Timestamp,
};
use crate::features::core::{HomePage, LoginPage};
use crate::features::forms::*;
use crate::features::organization::*;
use crate::features::administration::*;
use crate::features::workflows::submission::*;
pub(crate) use crate::types::route_params::{
    AccountRouteParams, DashboardRouteParams, FormRouteParams, NodeRouteParams, NodeTypeRouteParams,
    ReportRouteParams, RoleRouteParams, SubmissionRouteParams, WorkflowRouteParams,
    WorkflowRouteParams as WorkflowRouteParamsForShared, require_route_params,
};
use crate::ui::empty_view;
use icons::{
    ArrowDown, ArrowUp, CalendarDays, ChevronDown, ChevronRight, CircleDot, ExternalLink, FileText,
    Hash, ListChecks, ListFilter, LockKeyhole, Mail, PanelRight, Pencil, Plus, Search,
    SquareCheckBig, TextCursorInput, TextQuote, Trash2, X,
};
use leptos::portal::Portal;
use leptos::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::{BTreeMap, HashMap, HashSet};

#[cfg(feature = "hydrate")]
pub(crate) fn navigate_to_href(href: &str) {
    if let Some(window) = web_sys::window() {
        let _ = window.location().set_href(href);
    }
}

mod shared {
    pub(crate) use crate::features::shared_data::*;
}
pub(crate) use shared::*;

mod ui;
mod helpers;
mod pagination;
pub(crate) use ui::{
    active_workflow_definition_version,
    assignment_count_label,
    blank_form_builder_field_at,
    collect_response_values,
    form_attached_nodes,
    form_builder_field_default_label,
    form_builder_field_has_collision,
    form_builder_field_type_icon,
    form_builder_fields_overlap,
    form_builder_linear_grid_index,
    form_builder_occupancy_map,
    form_builder_section_fields,
    form_builder_section_layout,
    form_definition_scope_label,
    form_field_count_label,
    form_version_desc_sort_key,
    FormBuilderGridCell,
    FormBuilderSectionLayout,
    node_count_label,
    node_display_path,
    prepared_form_builder_fields,
    prepared_form_builder_sections,
    rendered_field_layout_label,
    rendered_field_type_label,
    response_input_value,
    response_selected_assignment,
    response_start_can_submit,
    submission_assignee_label,
    submission_progress_label,
    submission_status_key,
    submission_status_label,
    submission_step_label,
    submission_value_maps,
    submission_workflow_label,
    text_matches,
    user_count_label,
    workflow_assigned_user_links,
    workflow_assignee_label,
    workflow_assignment_assignee_label,
    workflow_assignment_candidate_key,
    workflow_assignment_revision_label,
    workflow_assignment_state,
    workflow_assignment_state_label,
    workflow_assignment_status_key,
    workflow_assignment_status_label,
    workflow_available_node_links,
    workflow_available_nodes_label,
    workflow_definition_status_label,
    workflow_definition_version_label,
    workflow_description_label,
    workflow_revision_label_from_raw,
    workflow_source_label,
    workflow_status_key,
    workflow_status_label,
    workflow_version_label,
    WorkflowSourceMarker,
};

pub(crate) use pagination::{
    pagination_current_page,
    pagination_page_count,
    pagination_page_end,
    pagination_page_start,
};

mod filtering;
pub(crate) use filtering::*;

#[component]
pub(crate) fn FilterHeader(
    label: &'static str,
    all_label: &'static str,
    filter: RwSignal<String>,
    options: Vec<String>,
    #[prop(default = false)] always_searchable: bool,
) -> impl IntoView {
    const FILTER_SEARCH_THRESHOLD: usize = 8;

    let is_open = RwSignal::new(false);
    let filter_query = RwSignal::new(String::new());
    let is_searchable = always_searchable || options.len() > FILTER_SEARCH_THRESHOLD;
    let options_for_buttons = options.clone();
    let menu_class = move || {
        if is_open.get() {
            "data-table-filter is-open"
        } else {
            "data-table-filter"
        }
    };
    let button_label = move || {
        let current = filter.get();
        if current == "all" {
            format!("Filter {label}")
        } else {
            format!("Filter {label}: {}", metadata_label(&current))
        }
    };
    let trigger_class = move || {
        if filter.get() == "all" {
            "icon-button data-table-filter__trigger"
        } else {
            "icon-button data-table-filter__trigger is-filtered"
        }
    };
    let filter_option_class = |current: &str, value: &str| {
        if current == value {
            "data-table-filter__option is-active"
        } else {
            "data-table-filter__option"
        }
    };
    let filtered_options = move || {
        let query = filter_query.get();
        options_for_buttons
            .iter()
            .filter(|option| {
                text_matches(&query, &[option.as_str(), metadata_label(option).as_str()])
            })
            .cloned()
            .collect::<Vec<_>>()
    };

    view! {
        <div class=menu_class>
            <span>{label}</span>
            <button
                class=trigger_class
                type="button"
                aria-label=button_label
                title=button_label
                aria-haspopup="menu"
                aria-expanded=move || is_open.get().to_string()
                on:click=move |_| is_open.update(|open| *open = !*open)
            >
                <ListFilter/>
            </button>
            <button
                class="data-table-filter__scrim"
                type="button"
                aria-label=format!("Close {label} filter")
                on:click=move |_| {
                    is_open.set(false);
                    filter_query.set(String::new());
                }
            ></button>
            <div class="data-table-filter__menu blurred-surface" role="menu">
                {if is_searchable {
                    view! {
                        <label class="data-table-filter__search">
                            <Search/>
                            <span class="sr-only">{format!("Search {label} filters")}</span>
                            <input
                                type="search"
                                placeholder=format!("Search {label}")
                                prop:value=move || filter_query.get()
                                on:input=move |event| filter_query.set(event_target_value(&event))
                            />
                        </label>
                    }
                    .into_any()
                } else {
                    empty_view()
                }}
                <button
                    class=move || filter_option_class(&filter.get(), "all")
                    type="button"
                    role="menuitemradio"
                    aria-checked=move || (filter.get() == "all").to_string()
                    on:click=move |_| {
                        filter.set("all".to_string());
                        is_open.set(false);
                        filter_query.set(String::new());
                    }
                >
                    {all_label}
                </button>
                {move || {
                    let visible_options = filtered_options();
                    if visible_options.is_empty() {
                        view! {
                            <p class="data-table-filter__empty">"No matching filters"</p>
                        }
                        .into_any()
                    } else {
                        visible_options
                            .into_iter()
                            .map(|option| {
                                let option_for_class = option.clone();
                                let option_for_checked = option.clone();
                                let option_for_click = option.clone();
                                let option_label = metadata_label(&option);
                                view! {
                                    <button
                                        class=move || filter_option_class(&filter.get(), &option_for_class)
                                        type="button"
                                        role="menuitemradio"
                                        aria-checked=move || (filter.get() == option_for_checked).to_string()
                                        on:click=move |_| {
                                            filter.set(option_for_click.clone());
                                            is_open.set(false);
                                            filter_query.set(String::new());
                                        }
                                    >
                                        {option_label}
                                    </button>
                                }
                            })
                            .collect_view()
                            .into_any()
                    }
                }}
            </div>
        </div>
    }
}

