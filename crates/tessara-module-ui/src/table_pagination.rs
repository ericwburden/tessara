//! Reusable table pagination controls.
//!
//! This module owns footer markup and row-range summaries for paginated feature tables.

use icons::{ChevronLeft, ChevronRight};
use leptos::prelude::*;

/// Presentation-only pagination shell. Callers own page state and provide
/// controls suitable for local, offset, or cursor-backed pagination.
#[component]
pub fn TablePaginationBar(
    summary: Signal<String>,
    children: Children,
    #[prop(default = "Table pagination")] aria_label: &'static str,
    #[prop(optional, into)] class: String,
) -> impl IntoView {
    let class = if class.trim().is_empty() {
        "directory-table-pagination".to_string()
    } else {
        format!("directory-table-pagination {}", class.trim())
    };
    view! {
        <nav class=class aria-label=aria_label>
            <p>{move || summary.get()}</p>
            <div class="directory-table-pagination__actions">{children()}</div>
        </nav>
    }
}

#[component]
/// Renders shared pagination controls and a row-range summary for feature tables.
pub fn TablePaginationFooter(
    aria_label: &'static str,
    item_label: &'static str,
    #[prop(optional)] empty_item_label: Option<&'static str>,
    #[prop(optional, into)] class: String,
    total_count: Memo<usize>,
    page_size: RwSignal<usize>,
    page_index: RwSignal<usize>,
) -> impl IntoView {
    let summary = Signal::derive(move || {
        table_page_summary(
            total_count.get(),
            page_size.get(),
            page_index.get(),
            item_label,
            empty_item_label,
        )
    });
    view! {
        <TablePaginationBar summary aria_label class>
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
                    class="interactive-data-table__page-button"
                    type="button"
                    aria-label="Previous page"
                    title="Previous page"
                    disabled=move || pagination_current_page(total_count.get(), page_size.get(), page_index.get()) == 0
                    on:click=move |_| {
                        page_index.update(|page| *page = page.saturating_sub(1));
                    }
                >
                    <ChevronLeft/>
                </button>
                <span>{move || {
                    format!(
                        "Page {} of {}",
                        pagination_current_page(total_count.get(), page_size.get(), page_index.get()) + 1,
                        pagination_page_count(total_count.get(), page_size.get())
                    )
                }}</span>
                <button
                    class="interactive-data-table__page-button"
                    type="button"
                    aria-label="Next page"
                    title="Next page"
                    disabled=move || {
                        pagination_current_page(total_count.get(), page_size.get(), page_index.get()) + 1
                            >= pagination_page_count(total_count.get(), page_size.get())
                    }
                    on:click=move |_| {
                        let last_page = pagination_page_count(total_count.get(), page_size.get()).saturating_sub(1);
                        page_index.update(|page| *page = (*page + 1).min(last_page));
                    }
                >
                    <ChevronRight/>
                </button>
        </TablePaginationBar>
    }
}

#[cfg(all(test, feature = "ssr"))]
mod tests {
    use super::*;

    #[test]
    fn pagination_bar_accepts_caller_owned_controls() {
        let html = Owner::new().with(|| {
            view! {
                <TablePaginationBar
                    summary=Signal::derive(|| "Showing 1-10 of 25 rows".to_string())
                    aria_label="Orders pagination"
                >
                    <button type="button">"Next"</button>
                </TablePaginationBar>
            }
            .to_html()
        });

        assert!(html.contains("<nav"));
        assert!(html.contains("aria-label=\"Orders pagination\""));
        assert!(html.contains("Showing 1-10 of 25 rows"));
        assert!(html.contains("directory-table-pagination__actions"));
        assert!(html.contains(">Next</button>"));
    }
}

/// Formats the visible row-range summary while preserving the empty-table copy.
fn table_page_summary(
    total_count: usize,
    page_size: usize,
    page_index: usize,
    item_label: &'static str,
    empty_item_label: Option<&'static str>,
) -> String {
    if total_count == 0 {
        format!("No {} to display", empty_item_label.unwrap_or(item_label))
    } else {
        format!(
            "Showing {}-{} of {} {item_label}",
            pagination_page_start(total_count, page_size, page_index) + 1,
            pagination_page_end(total_count, page_size, page_index),
            total_count
        )
    }
}

fn pagination_page_count(total_count: usize, page_size: usize) -> usize {
    if total_count == 0 {
        1
    } else {
        total_count.div_ceil(page_size)
    }
}

fn pagination_current_page(total_count: usize, page_size: usize, page_index: usize) -> usize {
    page_index.min(pagination_page_count(total_count, page_size) - 1)
}

fn pagination_page_start(total_count: usize, page_size: usize, page_index: usize) -> usize {
    if total_count == 0 {
        0
    } else {
        pagination_current_page(total_count, page_size, page_index) * page_size
    }
}

fn pagination_page_end(total_count: usize, page_size: usize, page_index: usize) -> usize {
    (pagination_page_start(total_count, page_size, page_index) + page_size).min(total_count)
}
