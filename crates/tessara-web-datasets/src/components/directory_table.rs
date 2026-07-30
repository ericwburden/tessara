//! Dataset directory table and mobile-card components.

use super::super::display::visibility_label;
use super::super::types::DatasetSummary;
use crate::{pagination::pagination_page_start, text::sentence_label};
use icons::Search;
use leptos::prelude::*;
use tessara_module_ui::{DataTable, TablePaginationFooter};

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
                        <th scope="col">"Catalog"</th>
                        <th scope="col">"Grain"</th>
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
    let tag_text = tag_label(&dataset.tags);
    let provenance_text = provenance_label(&dataset);
    view! {
        <tr>
            <th scope="row" class="data-table__stacked-label">
                <a class="data-table__primary-link" href=href>{dataset.name}</a>
                <span class="data-table__secondary-text">{dataset.slug}</span>
            </th>
            <td class="data-table__stacked-label">
                <span>{tag_text}</span>
                <span class="data-table__secondary-text">{provenance_text}</span>
            </td>
            <td>{sentence_label(&dataset.grain)}</td>
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
                    let tag_text = tag_label(&dataset.tags);
                    let provenance_text = provenance_label(&dataset);
                    view! {
                        <article class="related-work-mobile-card">
                            <h4><a href=href>{dataset.name}</a></h4>
                            <dl>
                                <dt>"Grain"</dt><dd>{sentence_label(&dataset.grain)}</dd>
                                <dt>"Tags"</dt><dd>{tag_text}</dd>
                                <dt>"Provenance"</dt><dd>{provenance_text}</dd>
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

fn tag_label(tags: &[String]) -> String {
    if tags.is_empty() {
        "No tags".into()
    } else {
        tags.join(", ")
    }
}

fn provenance_label(dataset: &DatasetSummary) -> String {
    let form_count = dataset.provenance.forms.len();
    let dataset_count = dataset.provenance.datasets.len();
    match (form_count, dataset_count) {
        (0, 0) => "No source provenance".into(),
        (forms, 0) => format!("{forms} form source{}", plural_suffix(forms)),
        (0, datasets) => format!("{datasets} dataset source{}", plural_suffix(datasets)),
        (forms, datasets) => format!(
            "{forms} form source{}, {datasets} dataset source{}",
            plural_suffix(forms),
            plural_suffix(datasets)
        ),
    }
}

fn plural_suffix(count: usize) -> &'static str {
    if count == 1 { "" } else { "s" }
}
