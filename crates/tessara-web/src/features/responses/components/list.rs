//! Response list table and mobile card components.

use super::{ResponseDesktopTable, ResponseMobileCards};
use crate::features::responses::types::SubmissionSummary;
use crate::ui::TablePaginationFooter;

use icons::Search;
use leptos::prelude::*;

#[component]
pub(crate) fn ResponsesList(
    submissions: Vec<SubmissionSummary>,
    search: RwSignal<String>,
    assignee_filter: RwSignal<String>,
    status_filter: RwSignal<String>,
    assignee_options: Vec<String>,
    status_options: Vec<String>,
) -> impl IntoView {
    let mut table_submissions = submissions.clone();
    table_submissions.sort_by(|left, right| {
        right
            .last_modified_at
            .cmp(&left.last_modified_at)
            .then(
                left.form_name
                    .to_lowercase()
                    .cmp(&right.form_name.to_lowercase()),
            )
            .then(left.id.cmp(&right.id))
    });
    let card_submissions = table_submissions.clone();
    let page_size = RwSignal::new(10usize);
    let page_index = RwSignal::new(0usize);
    let total_count = table_submissions.len();
    let total_count_memo = Memo::new(move |_| total_count);

    view! {
        <div class="forms-list forms-list-responsive-table responses-list">
            <div class="searchable-data-table">
                <div class="searchable-data-table__toolbar forms-list__toolbar">
                    <label class="searchable-data-table__search searchable-data-table__control">
                        <Search class="searchable-data-table__control-icon"/>
                        <span class="sr-only">"Search responses"</span>
                        <input
                            type="search"
                            placeholder="Search responses"
                            prop:value=move || search.get()
                            on:input=move |event| search.set(event_target_value(&event))
                        />
                    </label>
                </div>
                <ResponseDesktopTable
                    submissions=table_submissions
                    total_count
                    page_size
                    page_index
                    assignee_filter
                    status_filter
                    assignee_options
                    status_options
                />
                <TablePaginationFooter
                    aria_label="Responses table pagination"
                    item_label="responses"
                    total_count=total_count_memo
                    page_size=page_size
                    page_index=page_index
                />
            </div>
            <ResponseMobileCards
                submissions=card_submissions
                total_count
                page_size
                page_index
            />
        </div>
    }
}
