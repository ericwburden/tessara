//! Related dataset source table for form detail pages.

use crate::support::pagination::pagination_page_start;
use crate::support::text::text_matches;
use crate::{FormDatasetSourceLink, FormDatasetSourcesState};
use leptos::prelude::*;
use tessara_module_ui::{SearchableDataTable, TablePaginationFooter};

#[component]
pub(crate) fn FormRelatedDatasetSourcesTable(
    state: FormDatasetSourcesState,
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
            .filter(|source| text_matches(&query, &[&source.dataset_name, &source.source_alias]))
            .cloned()
            .collect::<Vec<_>>()
    });
    let total_count = Memo::new(move |_| filtered_sources.get().len());

    if let Some(message) = dataset_sources_state_message(state) {
        return view! {
            <div class="empty-state" role="status">
                <p>{message}</p>
            </div>
        }
        .into_any();
    }

    view! {
        <div class="related-work-responsive-table">
            <SearchableDataTable search_label="Search dataset sources" placeholder="Search related dataset sources" search>
                <thead>
                    <tr>
                        <th scope="col">"Dataset"</th>
                        <th scope="col">"Alias"</th>
                    </tr>
                </thead>
                <tbody>
                    {move || {
                        let rows = filtered_sources.get();
                        if rows.is_empty() {
                            view! {
                                <tr>
                                    <td class="data-table__empty" colspan="2">"No Related Dataset Sources to Display"</td>
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
                                    let href = dataset_source_href(&source);
                                    view! {
                                        <tr>
                                            <th scope="row">
                                                {match href {
                                                    Some(href) => view! {
                                                        <a class="data-table__primary-link" href=href>{source.dataset_name}</a>
                                                    }.into_any(),
                                                    None => view! { <span>{source.dataset_name}</span> }.into_any(),
                                                }}
                                            </th>
                                            <td>{source.source_alias}</td>
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
                                let href = dataset_source_href(&source);
                                view! {
                                    <article class="related-work-mobile-card">
                                        <div class="related-work-mobile-card__header">
                                            <h4>
                                                {match href {
                                                    Some(href) => view! { <a href=href>{source.dataset_name}</a> }.into_any(),
                                                    None => view! { <span>{source.dataset_name}</span> }.into_any(),
                                                }}
                                            </h4>
                                        </div>
                                        <dl>
                                            <div>
                                                <dt>"Alias"</dt>
                                                <dd>{source.source_alias}</dd>
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
    .into_any()
}

fn dataset_source_href(source: &FormDatasetSourceLink) -> Option<String> {
    let dataset_id = source
        .semantic_destination
        .strip_prefix("datasets.detail:")?;
    (dataset_id == source.dataset_id).then(|| format!("/datasets/{dataset_id}"))
}

fn dataset_sources_state_message(state: FormDatasetSourcesState) -> Option<&'static str> {
    match state {
        FormDatasetSourcesState::Available => None,
        FormDatasetSourcesState::Empty => Some("No Related Dataset Sources to Display"),
        FormDatasetSourcesState::Unavailable => Some(
            "Dataset source information is temporarily unavailable. Other Form details remain available.",
        ),
        FormDatasetSourcesState::Undisclosed => {
            Some("Dataset source information is unavailable for this Form.")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{dataset_source_href, dataset_sources_state_message};
    use crate::{FormDatasetSourceLink, FormDatasetSourcesState};

    #[test]
    fn empty_outage_and_undisclosed_have_distinct_nonleaking_copy() {
        assert_eq!(
            dataset_sources_state_message(FormDatasetSourcesState::Empty),
            Some("No Related Dataset Sources to Display")
        );
        assert!(
            dataset_sources_state_message(FormDatasetSourcesState::Unavailable)
                .expect("unavailable copy")
                .contains("temporarily unavailable")
        );
        assert_eq!(
            dataset_sources_state_message(FormDatasetSourcesState::Undisclosed),
            Some("Dataset source information is unavailable for this Form.")
        );
    }

    #[test]
    fn dataset_links_use_the_provider_semantic_destination_identity() {
        let dataset_id = "018f032a-1f76-7f15-9f31-f1cfec675bbe";
        let mut source = FormDatasetSourceLink {
            dataset_id: dataset_id.into(),
            dataset_name: "Cases".into(),
            source_alias: "case_form".into(),
            pinned_form_version_id: "018f032a-1f76-7f15-9f31-f1cfec675bbf".into(),
            lifecycle_state: "active".into(),
            semantic_destination: format!("datasets.detail:{dataset_id}"),
        };
        assert_eq!(
            dataset_source_href(&source).as_deref(),
            Some("/datasets/018f032a-1f76-7f15-9f31-f1cfec675bbe")
        );

        source.semantic_destination = "datasets.detail:018f032a-1f76-7f15-9f31-f1cfec675bc0".into();
        assert_eq!(dataset_source_href(&source), None);
    }
}
