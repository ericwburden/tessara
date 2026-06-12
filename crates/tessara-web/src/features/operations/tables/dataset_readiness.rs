//! Dataset readiness table for Operations.

use crate::features::operations::types::DatasetStatus;
use crate::features::shared::unique_filter_options;
use crate::ui::{DataTable, EmptyState, StatusBadge, TableFilterHeader, TablePaginationFooter};
use crate::utils::pagination::pagination_page_start;
use icons::Search;
use leptos::prelude::*;

use super::dataset_readiness_filtering::filtered_dataset_readiness;

#[component]
pub(crate) fn DatasetReadinessTable(datasets: Vec<DatasetStatus>) -> impl IntoView {
    let all_datasets = datasets.clone();
    let search = RwSignal::new(String::new());
    let status_filter = RwSignal::new("all".to_string());
    let page_size = RwSignal::new(10usize);
    let page_index = RwSignal::new(0usize);
    let status_options =
        unique_filter_options(datasets.iter().map(|dataset| dataset.readiness.clone()));
    let filtered_datasets = Memo::new(move |_| {
        filtered_dataset_readiness(&all_datasets, &search.get(), &status_filter.get())
    });
    let total_count = Memo::new(move |_| filtered_datasets.get().len());

    view! {
        <section class="route-panel__section operations-table-section" aria-label="Dataset readiness">
            <h3>"Dataset Readiness"</h3>
            {if datasets.is_empty() {
                view! {
                    <EmptyState
                        title="No visible datasets"
                        message="No dataset readiness information is visible for the current account."
                    />
                }
                .into_any()
            } else {
                view! {
                    <div class="searchable-data-table operations-status-table operations-responsive-table">
                        <div class="searchable-data-table__toolbar forms-list__toolbar">
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
                        </div>
                        <DataTable>
                            <thead>
                                <tr>
                                    <th scope="col">"Dataset"</th>
                                    <th class="data-table__cell--center" scope="col">
                                        <TableFilterHeader
                                            label="Status"
                                            all_label="All Statuses"
                                            filter=status_filter
                                            options=status_options.clone()
                                        />
                                    </th>
                                    <th scope="col">"Published version"</th>
                                    <th class="data-table__cell--center" scope="col">"Linked forms"</th>
                                    <th class="data-table__cell--center" scope="col">"Columns"</th>
                                    <th class="data-table__cell--center" scope="col">"Ready responses"</th>
                                </tr>
                            </thead>
                            <tbody>
                                {move || filtered_datasets.get()
                                    .into_iter()
                                    .skip(pagination_page_start(total_count.get(), page_size.get(), page_index.get()))
                                    .take(page_size.get())
                                    .map(|dataset| {
                                        view! { <DatasetReadinessRow dataset/> }
                                    })
                                    .collect_view()}
                            </tbody>
                        </DataTable>
                        <TablePaginationFooter
                            aria_label="Dataset readiness table pagination"
                            item_label="datasets"
                            total_count=total_count
                            page_size=page_size
                            page_index=page_index
                        />
                        <div class="operations-mobile-cards">
                            {move || {
                                let visible_datasets = filtered_datasets.get();
                                if visible_datasets.is_empty() {
                                    view! { <p class="related-work-mobile-empty">"No Datasets to Display"</p> }.into_any()
                                } else {
                                    visible_datasets
                                        .into_iter()
                                        .skip(pagination_page_start(total_count.get(), page_size.get(), page_index.get()))
                                        .take(page_size.get())
                                        .map(|dataset| view! { <DatasetReadinessMobileCard dataset/> })
                                        .collect_view()
                                        .into_any()
                                }
                            }}
                        </div>
                    </div>
                }
                .into_any()
            }}
        </section>
    }
}

#[component]
fn DatasetReadinessRow(dataset: DatasetStatus) -> impl IntoView {
    let dataset_href = format!("/datasets/{}", dataset.dataset_id);
    view! {
        <tr>
            <th scope="row">
                <a class="data-table__primary-link" href=dataset_href>{dataset.dataset_name}</a>
            </th>
            <td class="data-table__cell--center"><StatusBadge label=dataset.readiness.clone()/></td>
            <td>{dataset.revision_status.clone()}</td>
            <td class="data-table__cell--center">{dataset.source_count}</td>
            <td class="data-table__cell--center">{dataset.field_count}</td>
            <td class="data-table__cell--center">{dataset.ready_response_count}</td>
        </tr>
    }
}

#[component]
fn DatasetReadinessMobileCard(dataset: DatasetStatus) -> impl IntoView {
    let dataset_href = format!("/datasets/{}", dataset.dataset_id);
    view! {
        <article class="related-work-mobile-card operations-mobile-card">
            <header class="related-work-mobile-card__header">
                <a href=dataset_href>{dataset.dataset_name.clone()}</a>
            </header>
            <dl>
                <div>
                    <dt>"Status"</dt>
                    <dd><StatusBadge label=dataset.readiness.clone()/></dd>
                </div>
                <div>
                    <dt>"Published version"</dt>
                    <dd>{dataset.revision_status.clone()}</dd>
                </div>
                <div>
                    <dt>"Linked forms"</dt>
                    <dd>{dataset.source_count}</dd>
                </div>
                <div>
                    <dt>"Columns"</dt>
                    <dd>{dataset.field_count}</dd>
                </div>
                <div>
                    <dt>"Ready responses"</dt>
                    <dd>{dataset.ready_response_count}</dd>
                </div>
            </dl>
        </article>
    }
}
