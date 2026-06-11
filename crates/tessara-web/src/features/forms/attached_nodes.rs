//! Form-to-organization attachment views.
//!
//! Keep components that summarize and navigate nodes attached to forms here; generic node labels belong in shared helpers.

use crate::features::organization::RelatedWorkPaginationFooter;
use crate::features::shared::FormAttachmentLink;
use crate::ui::SearchableDataTable;
use crate::utils::pagination::pagination_page_start;
use crate::utils::text::text_matches;
use leptos::prelude::*;

#[component]
/// Renders the form attached nodes related table view.
pub(crate) fn FormAttachedNodesRelatedTable(nodes: Vec<FormAttachmentLink>) -> impl IntoView {
    let search = RwSignal::new(String::new());
    let page_size = RwSignal::new(10usize);
    let page_index = RwSignal::new(0usize);
    let nodes_for_filter = nodes;
    let filtered_nodes = Memo::new(move |_| {
        let query = search.get();
        nodes_for_filter
            .iter()
            .filter(|node| text_matches(&query, &[&node.label, &node.title]))
            .cloned()
            .collect::<Vec<_>>()
    });
    let total_count = Memo::new(move |_| filtered_nodes.get().len());

    view! {
        <div class="related-work-responsive-table">
            <SearchableDataTable search_label="Search attached nodes" placeholder="Search attached nodes" search>
                <thead>
                    <tr>
                        <th scope="col">"Node"</th>
                        <th scope="col">"Context"</th>
                    </tr>
                </thead>
                <tbody>
                    {move || {
                        let rows = filtered_nodes.get();
                        if rows.is_empty() {
                            view! {
                                <tr>
                                    <td class="data-table__empty" colspan="2">"No Attached Nodes to Display"</td>
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
                                .map(|node| {
                                    let title = node.title.clone();
                                    view! {
                                        <tr>
                                            <th scope="row">
                                                <a class="data-table__primary-link" href=node.href title=title>{node.label}</a>
                                            </th>
                                            <td>{node.title}</td>
                                        </tr>
                                    }
                                })
                                .collect_view()
                                .into_any()
                        }
                    }}
                </tbody>
            </SearchableDataTable>
            <RelatedWorkPaginationFooter
                aria_label="Attached nodes table pagination"
                label="attached nodes"
                total_count=total_count
                page_size=page_size
                page_index=page_index
            />
            <div class="related-work-mobile-cards">
                {move || {
                    let rows = filtered_nodes.get();
                    if rows.is_empty() {
                        view! { <p class="related-work-mobile-empty">"No Attached Nodes to Display"</p> }.into_any()
                    } else {
                        let total_count = rows.len();
                        let start = pagination_page_start(total_count, page_size.get(), page_index.get());
                        rows
                            .iter()
                            .skip(start)
                            .take(page_size.get())
                            .cloned()
                            .map(|node| {
                                let title = node.title.clone();
                                view! {
                                    <article class="related-work-mobile-card">
                                        <div class="related-work-mobile-card__header">
                                            <h4><a href=node.href title=title>{node.label}</a></h4>
                                        </div>
                                        <dl>
                                            <div>
                                                <dt>"Context"</dt>
                                                <dd>{node.title}</dd>
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
