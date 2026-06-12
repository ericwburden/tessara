//! Related dataset source table for form detail pages.

use crate::features::forms::FormDatasetSourceLink;
use crate::ui::{SearchableDataTable, TablePaginationFooter};
use crate::utils::pagination::pagination_page_start;
use crate::utils::text::{sentence_label, text_matches};
use leptos::prelude::*;

#[component]
pub(crate) fn FormRelatedDatasetSourcesTable(
    dataset_sources: Vec<FormDatasetSourceLink>,
) -> impl IntoView {
    let search = RwSignal::new(String::new());
    let page_size = RwSignal::new(10usize);
    let page_index = RwSignal::new(0usize);
    let sources_for_filter = dataset_sources;
    let filtered_sources = Memo::new(move |_| {
        let query = search.get();
        sources_for_filter
            .iter()
            .filter(|source| {
                text_matches(
                    &query,
                    &[
                        &source.dataset_name,
                        &source.source_alias,
                        &source.selection_rule,
                    ],
                )
            })
            .cloned()
            .collect::<Vec<_>>()
    });
    let total_count = Memo::new(move |_| filtered_sources.get().len());

    view! {
        <div class="related-work-responsive-table">
            <SearchableDataTable search_label="Search dataset sources" placeholder="Search related dataset sources" search>
                <thead>
                    <tr>
                        <th scope="col">"Dataset"</th>
                        <th scope="col">"Alias"</th>
                        <th scope="col">"Selection rule"</th>
                    </tr>
                </thead>
                <tbody>
                    {move || {
                        let rows = filtered_sources.get();
                        if rows.is_empty() {
                            view! {
                                <tr>
                                    <td class="data-table__empty" colspan="3">"No Related Dataset Sources to Display"</td>
                                </tr>
                            }
                            .into_any()
                        } else {
                            let total_count = rows.len();
                            let start = pagination_page_start(total_count, page_size.get(), page_index.get());
                            rows
                                .iter()
                                .skip(start)
                                .take(page_size.get())
                                .cloned()
                                .map(|source| {
                                    view! {
                                        <tr>
                                            <th scope="row">
                                                <a class="data-table__primary-link" href=format!("/datasets/{}", source.dataset_id)>{source.dataset_name}</a>
                                            </th>
                                            <td>{source.source_alias}</td>
                                            <td>{sentence_label(&source.selection_rule)}</td>
                                        </tr>
                                    }
                                })
                                .collect_view()
                                .into_any()
                        }
                    }}
                </tbody>
            </SearchableDataTable>
            <TablePaginationFooter
                aria_label="Related dataset sources table pagination"
                item_label="related dataset sources"
                total_count=total_count
                page_size=page_size
                page_index=page_index
            />
            <div class="related-work-mobile-cards">
                {move || {
                    let rows = filtered_sources.get();
                    if rows.is_empty() {
                        view! { <p class="related-work-mobile-empty">"No Related Dataset Sources to Display"</p> }.into_any()
                    } else {
                        let total_count = rows.len();
                        let start = pagination_page_start(total_count, page_size.get(), page_index.get());
                        rows
                            .iter()
                            .skip(start)
                            .take(page_size.get())
                            .cloned()
                            .map(|source| {
                                view! {
                                    <article class="related-work-mobile-card">
                                        <div class="related-work-mobile-card__header">
                                            <h4><a href=format!("/datasets/{}", source.dataset_id)>{source.dataset_name}</a></h4>
                                        </div>
                                        <dl>
                                            <div>
                                                <dt>"Alias"</dt>
                                                <dd>{source.source_alias}</dd>
                                            </div>
                                            <div>
                                                <dt>"Selection rule"</dt>
                                                <dd>{sentence_label(&source.selection_rule)}</dd>
                                            </div>
                                        </dl>
                                    </article>
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
