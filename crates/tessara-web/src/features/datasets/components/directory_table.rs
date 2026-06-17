//! Dataset directory table and mobile-card components.

use super::super::display::visibility_label;
use super::super::types::DatasetSummary;
use crate::ui::{DataTable, TablePaginationFooter};
use crate::utils::{pagination::pagination_page_start, text::sentence_label};
use icons::Search;
use leptos::prelude::*;

#[component]
pub(crate) fn DatasetDirectoryTable(
    datasets: Vec<DatasetSummary>,
    search: RwSignal<String>,
    page_index: RwSignal<usize>,
    page_size: RwSignal<usize>,
) -> impl IntoView {
    let total_count_value = datasets.len();
    let total_count = Memo::new(move |_| total_count_value);
    let page_start = pagination_page_start(total_count_value, page_size.get(), page_index.get());
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
            <TablePaginationFooter
                aria_label="Datasets table pagination"
                item_label="datasets"
                total_count=total_count
                page_size=page_size
                page_index=page_index
            />
        </section>
    }
}

#[component]
fn DatasetSummaryRow(dataset: DatasetSummary) -> impl IntoView {
    let href = format!("/datasets/{}", dataset.id);
    view! {
        <tr>
            <th scope="row" class="data-table__stacked-label">
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
