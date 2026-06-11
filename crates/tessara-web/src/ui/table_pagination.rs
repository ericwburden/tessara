//! Reusable table pagination controls.
//!
//! This module owns footer markup and row-range summaries for paginated feature tables.

use leptos::prelude::*;

use crate::utils::pagination::{
    pagination_current_page, pagination_page_count, pagination_page_end, pagination_page_start,
};

#[component]
/// Renders the table pagination footer view.
pub(crate) fn TablePaginationFooter(
    aria_label: &'static str,
    item_label: &'static str,
    total_count: Memo<usize>,
    page_size: RwSignal<usize>,
    page_index: RwSignal<usize>,
) -> impl IntoView {
    view! {
        <div class="directory-table-pagination" aria-label=aria_label>
            <p>{move || table_page_summary(total_count.get(), page_size.get(), page_index.get(), item_label)}</p>
            <div class="directory-table-pagination__actions">
                <label class="directory-table-pagination__page-size searchable-data-table__filter searchable-data-table__control">
                    <span>"Rows"</span>
                    <select
                        prop:value=move || page_size.get().to_string()
                        on:change=move |event| {
                            if let Ok(size) = event_target_value(&event).parse::<usize>() {
                                page_size.set(size);
                                page_index.set(0);
                            }
                        }
                    >
                        <option value="10">"10"</option>
                        <option value="25">"25"</option>
                        <option value="50">"50"</option>
                    </select>
                </label>
                <button
                    class="button button--compact button--secondary"
                    type="button"
                    disabled=move || pagination_current_page(total_count.get(), page_size.get(), page_index.get()) == 0
                    on:click=move |_| {
                        page_index.update(|page| *page = page.saturating_sub(1));
                    }
                >
                    "Previous"
                </button>
                <span>{move || {
                    format!(
                        "Page {} of {}",
                        pagination_current_page(total_count.get(), page_size.get(), page_index.get()) + 1,
                        pagination_page_count(total_count.get(), page_size.get())
                    )
                }}</span>
                <button
                    class="button button--compact button--secondary"
                    type="button"
                    disabled=move || {
                        pagination_current_page(total_count.get(), page_size.get(), page_index.get()) + 1
                            >= pagination_page_count(total_count.get(), page_size.get())
                    }
                    on:click=move |_| {
                        let last_page = pagination_page_count(total_count.get(), page_size.get()).saturating_sub(1);
                        page_index.update(|page| *page = (*page + 1).min(last_page));
                    }
                >
                    "Next"
                </button>
            </div>
        </div>
    }
}

/// Handles the table page summary behavior.
fn table_page_summary(
    total_count: usize,
    page_size: usize,
    page_index: usize,
    item_label: &'static str,
) -> String {
    if total_count == 0 {
        format!("No {item_label} to display")
    } else {
        format!(
            "Showing {}-{} of {} {item_label}",
            pagination_page_start(total_count, page_size, page_index) + 1,
            pagination_page_end(total_count, page_size, page_index),
            total_count
        )
    }
}
