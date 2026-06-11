//! Dataset directory table and mobile-card components.

use super::super::display::{table_summary, visibility_label};
use super::super::types::DatasetSummary;
use crate::ui::DataTable;
use crate::utils::{
    pagination::{pagination_current_page, pagination_page_count, pagination_page_start},
    text::sentence_label,
};
use icons::Search;
use leptos::prelude::*;

#[component]
/// Renders the dataset directory table view.
pub(crate) fn DatasetDirectoryTable(
    datasets: Vec<DatasetSummary>,
    search: RwSignal<String>,
    page_index: RwSignal<usize>,
    page_size: RwSignal<usize>,
) -> impl IntoView {
    let total_count = datasets.len();
    let page_count = pagination_page_count(total_count, page_size.get());
    let current_page = pagination_current_page(total_count, page_size.get(), page_index.get());
    let summary = table_summary(total_count, page_size.get(), page_index.get(), "datasets");
    let page_start = pagination_page_start(total_count, page_size.get(), page_index.get());
    let paged_datasets = datasets
        .iter()
        .skip(page_start)
        .take(page_size.get())
        .cloned()
        .collect::<Vec<_>>();

    view! {
        <section class="route-panel__section dataset-table-section">
            <label class="searchable-data-table__search searchable-data-table__control">
                <Search class="searchable-data-table__control-icon"/>
                <span class="sr-only">"Search datasets"</span>
                <input
                    type="search"
                    placeholder="Search datasets"
                    prop:value=move || search.get()
                    on:input=move |event| {
                        search.set(event_target_value(&event));
                        page_index.set(0);
                    }
                />
            </label>
            <DataTable>
                <thead>
                    <tr>
                        <th scope="col">"Dataset"</th>
                        <th scope="col">"Grain"</th>
                        <th scope="col">"Composition"</th>
                        <th scope="col">"Visibility"</th>
                        <th scope="col" class="data-table__cell--center">"Sources"</th>
                        <th scope="col" class="data-table__cell--center">"Fields"</th>
                    </tr>
                </thead>
                <tbody>
                    {paged_datasets
                        .into_iter()
                        .map(|dataset| view! { <DatasetSummaryRow dataset/> })
                        .collect_view()}
                </tbody>
            </DataTable>
            <DatasetMobileCards datasets=datasets.clone() page_index page_size/>
            <TablePagination
                summary=summary
                page_count=page_count
                current_page=current_page
                page_index
                page_size
            />
        </section>
    }
}

#[component]
/// Renders the dataset summary row view.
fn DatasetSummaryRow(dataset: DatasetSummary) -> impl IntoView {
    let href = format!("/datasets/{}", dataset.id);
    view! {
        <tr>
            <th scope="row">
                <a class="data-table__primary-link" href=href>{dataset.name}</a>
                <span class="data-table__secondary-text">{dataset.slug}</span>
            </th>
            <td>{sentence_label(&dataset.grain)}</td>
            <td>{sentence_label(&dataset.composition_mode)}</td>
            <td>{visibility_label(&dataset.visibility_nodes)}</td>
            <td class="data-table__cell--center">{dataset.source_count}</td>
            <td class="data-table__cell--center">{dataset.field_count}</td>
        </tr>
    }
}

#[component]
/// Renders the dataset mobile cards view.
fn DatasetMobileCards(
    datasets: Vec<DatasetSummary>,
    page_index: RwSignal<usize>,
    page_size: RwSignal<usize>,
) -> impl IntoView {
    let total_count = datasets.len();
    let page_start = pagination_page_start(total_count, page_size.get(), page_index.get());
    let paged_datasets = datasets
        .into_iter()
        .skip(page_start)
        .take(page_size.get())
        .collect::<Vec<_>>();
    view! {
        <div class="related-work-mobile-cards">
            {paged_datasets
                .into_iter()
                .map(|dataset| {
                    let href = format!("/datasets/{}", dataset.id);
                    view! {
                        <article class="related-work-mobile-card">
                            <h4><a href=href>{dataset.name}</a></h4>
                            <dl>
                                <dt>"Grain"</dt><dd>{sentence_label(&dataset.grain)}</dd>
                                <dt>"Composition"</dt><dd>{sentence_label(&dataset.composition_mode)}</dd>
                                <dt>"Visibility"</dt><dd>{visibility_label(&dataset.visibility_nodes)}</dd>
                                <dt>"Sources"</dt><dd>{dataset.source_count}</dd>
                                <dt>"Fields"</dt><dd>{dataset.field_count}</dd>
                            </dl>
                        </article>
                    }
                })
                .collect_view()}
        </div>
    }
}

#[component]
/// Renders the table pagination view.
fn TablePagination(
    summary: String,
    page_count: usize,
    current_page: usize,
    page_index: RwSignal<usize>,
    page_size: RwSignal<usize>,
) -> impl IntoView {
    view! {
        <div class="directory-table-pagination">
            <span>{summary}</span>
            <div class="directory-table-pagination__actions">
                <label>"Rows"
                    <select prop:value=move || page_size.get().to_string() on:change=move |event| {
                        if let Ok(value) = event_target_value(&event).parse::<usize>() {
                            page_size.set(value);
                            page_index.set(0);
                        }
                    }>
                        <option value="10">"10"</option>
                        <option value="25">"25"</option>
                        <option value="50">"50"</option>
                    </select>
                </label>
                <button class="button button--compact" type="button" disabled=move || page_index.get() == 0 on:click=move |_| page_index.update(|value| *value = value.saturating_sub(1))>"Previous"</button>
                <strong>{format!("Page {current_page} of {page_count}")}</strong>
                <button class="button button--compact" type="button" disabled=move || page_index.get() + 1 >= page_count on:click=move |_| page_index.update(|value| *value += 1)>"Next"</button>
            </div>
        </div>
    }
}
