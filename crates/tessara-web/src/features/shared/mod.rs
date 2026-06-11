//! Public boundary for the Shared feature.
//!
//! Re-export only the pages, types, and helpers other modules need; keep Shared-specific implementation details in child modules.

use crate::ui::empty_view;
use crate::utils::metadata::metadata_label as filter_metadata_label;
use crate::utils::text::text_matches as filter_text_matches;
use icons::{ListFilter, Search};
use leptos::prelude::*;

#[cfg(feature = "hydrate")]
/// Handles the navigate to href behavior.
pub(crate) fn navigate_to_href(href: &str) {
    if let Some(window) = web_sys::window() {
        let _ = window.location().set_href(href);
    }
}

mod display;
mod helpers;
mod placeholder;
mod types;
mod ui;
pub(crate) use display::status_badge_class;
pub(crate) use placeholder::NativePlaceholderRoute;
pub(crate) use types::{
    FormAttachmentLink, FormsAttachedNodesSheetData, WorkflowAssignedUsersSheetData,
    WorkflowAvailableNodesSheetData,
};
pub(crate) use ui::{node_count_label, node_display_path, user_count_label};

mod filtering;
pub(crate) use filtering::{
    FormNodeFilterOption, form_matches_node_filter, form_node_filter_options, indented_node_label,
    slug_from_label, unique_filter_options, visible_form_node_filter_options,
    workflow_form_version_options, workflow_step_form_label,
};
#[cfg(feature = "hydrate")]
pub(crate) use filtering::{
    existing_form_slugs, existing_form_slugs_for_update, existing_workflow_slugs,
    unique_slug_from_label,
};

#[component]
/// Renders the filter header view.
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
            format!("Filter {label}: {}", filter_metadata_label(&current))
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
                filter_text_matches(
                    &query,
                    &[option.as_str(), filter_metadata_label(option).as_str()],
                )
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
                                let option_label = filter_metadata_label(&option);
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
