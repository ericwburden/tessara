//! Related work filtering and pagination controls for organization nodes.

use crate::ui::TablePaginationFooter;
use crate::utils::pagination::{pagination_page_end, pagination_page_start};
use crate::utils::text::sentence_label;
use icons::ListFilter;
use leptos::prelude::*;

#[component]
/// Renders the status filter header view.
pub(crate) fn StatusFilterHeader(
    status_filter: RwSignal<String>,
    #[prop(optional)] compact_control: bool,
) -> impl IntoView {
    let is_open = RwSignal::new(false);
    let menu_class = move || {
        if is_open.get() {
            "data-table-filter is-open"
        } else {
            "data-table-filter"
        }
    };
    let button_label = move || {
        let current = status_filter.get();
        if current == "all" {
            "Filter Status".to_string()
        } else {
            format!("Filter Status: {}", sentence_label(&current))
        }
    };
    let trigger_class = move || {
        let size_class = if compact_control {
            " icon-button--compact-control"
        } else {
            ""
        };
        let filtered_class = if status_filter.get() == "all" {
            ""
        } else {
            " is-filtered"
        };
        format!("icon-button{size_class} data-table-filter__trigger{filtered_class}")
    };

    view! {
        <div class=menu_class>
            <span>"Status"</span>
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
                aria-label="Close status filter"
                on:click=move |_| is_open.set(false)
            ></button>
            <div class="data-table-filter__menu blurred-surface" role="menu">
                <button
                    class=move || filter_option_class(&status_filter.get(), "all")
                    type="button"
                    role="menuitemradio"
                    aria-checked=move || (status_filter.get() == "all").to_string()
                    on:click=move |_| {
                        status_filter.set("all".to_string());
                        is_open.set(false);
                    }
                >
                    "All statuses"
                </button>
                <button
                    class=move || filter_option_class(&status_filter.get(), "draft")
                    type="button"
                    role="menuitemradio"
                    aria-checked=move || (status_filter.get() == "draft").to_string()
                    on:click=move |_| {
                        status_filter.set("draft".to_string());
                        is_open.set(false);
                    }
                >
                    "Draft"
                </button>
                <button
                    class=move || filter_option_class(&status_filter.get(), "submitted")
                    type="button"
                    role="menuitemradio"
                    aria-checked=move || (status_filter.get() == "submitted").to_string()
                    on:click=move |_| {
                        status_filter.set("submitted".to_string());
                        is_open.set(false);
                    }
                >
                    "Submitted"
                </button>
            </div>
        </div>
    }
}

/// Handles the filter option class behavior.
pub(crate) fn filter_option_class(current: &str, value: &str) -> &'static str {
    if current == value {
        "data-table-filter__option is-active"
    } else {
        "data-table-filter__option"
    }
}

#[component]
/// Renders the related work pagination footer view.
pub(crate) fn RelatedWorkPaginationFooter(
    aria_label: &'static str,
    label: &'static str,
    total_count: Memo<usize>,
    page_size: RwSignal<usize>,
    page_index: RwSignal<usize>,
) -> impl IntoView {
    view! {
        <TablePaginationFooter aria_label=aria_label item_label=label total_count=total_count page_size=page_size page_index=page_index/>
    }
}

/// Handles the related work page summary behavior.
pub(crate) fn related_work_page_summary(
    total_count: usize,
    page_size: usize,
    page_index: usize,
    label: &'static str,
) -> String {
    if total_count == 0 {
        format!("No {label} to display")
    } else {
        format!(
            "Showing {}-{} of {} {label}",
            pagination_page_start(total_count, page_size, page_index) + 1,
            pagination_page_end(total_count, page_size, page_index),
            total_count
        )
    }
}
