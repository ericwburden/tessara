//! Form-to-organization attachment views.
//!
//! Keep components that summarize and navigate nodes attached to forms here; generic node labels belong in shared helpers.

use crate::features::shared::{FormAttachmentLink, FormsAttachedNodesSheetData, node_count_label};
use crate::ui::{SearchableDataTable, TablePaginationFooter, empty_view};
use crate::utils::pagination::pagination_page_start;
use crate::utils::text::text_matches;
use icons::{ExternalLink, PanelRight, Search, X};
use leptos::portal::Portal;
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
            <TablePaginationFooter
                aria_label="Attached nodes table pagination"
                item_label="attached nodes"
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

#[component]
/// Renders the forms attached nodes list view.
pub(in crate::features::forms) fn FormsAttachedNodesList(
    nodes: Vec<FormAttachmentLink>,
    form_name: String,
    form_href: String,
    sheet: RwSignal<Option<FormsAttachedNodesSheetData>>,
) -> impl IntoView {
    let total_nodes = nodes.len();
    let nodes_for_sheet = nodes.clone();
    let form_name_for_sheet = form_name.clone();
    let form_href_for_sheet = form_href.clone();

    view! {
        <div class="forms-attached-list">
            {if total_nodes == 0 {
                view! { <p>"Not attached"</p> }.into_any()
            } else if total_nodes > 0 {
                view! {
                    <button
                        class="forms-attached-list__more"
                        type="button"
                        aria-label=format!("View attached organization nodes for {form_name_for_sheet}")
                        title="Opens detail panel"
                        on:click=move |_| {
                            sheet.set(Some(FormsAttachedNodesSheetData {
                                form_name: form_name_for_sheet.clone(),
                                form_href: form_href_for_sheet.clone(),
                                nodes: nodes_for_sheet.clone(),
                            }));
                        }
                    >
                        <span>{node_count_label(total_nodes)}</span>
                        <PanelRight class="forms-attached-list__icon"/>
                    </button>
                }
                .into_any()
            } else {
                empty_view()
            }}
        </div>
    }
}

#[component]
/// Renders the forms attached nodes sheet view.
pub(in crate::features::forms) fn FormsAttachedNodesSheet(
    detail: RwSignal<Option<FormsAttachedNodesSheetData>>,
) -> impl IntoView {
    let search = RwSignal::new(String::new());
    let close = move |_| {
        detail.set(None);
        search.set(String::new());
    };
    let filtered_nodes = move || {
        let query = search.get().trim().to_lowercase();
        detail
            .get()
            .map(|data| {
                data.nodes
                    .into_iter()
                    .filter(|node| {
                        query.is_empty()
                            || node.label.to_lowercase().contains(&query)
                            || node.title.to_lowercase().contains(&query)
                    })
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default()
    };

    view! {
        <Portal>
            <Show when=move || detail.get().is_some()>
                <section class="sheet-overlay forms-attached-overlay" aria-label="Attached organization nodes">
                    <button class="sheet-overlay__scrim" type="button" aria-label="Close attached nodes" on:click=close></button>
                    <aside class="sheet-panel blurred-surface forms-attached-sheet" role="dialog" aria-modal="true" aria-label="Attached organization nodes">
                        <div class="sheet-panel__actions">
                            {move || {
                                detail
                                    .get()
                                    .map(|data| {
                                        view! {
                                            <a class="icon-button sheet-panel__open" href=data.form_href aria-label="Open form detail" title="Open form detail">
                                                <ExternalLink class="icon-button__icon"/>
                                            </a>
                                        }
                                        .into_any()
                                    })
                                    .unwrap_or_else(empty_view)
                            }}
                            <button class="icon-button sheet-panel__close" type="button" aria-label="Close attached nodes" title="Close attached nodes" on:click=close>
                                <X class="icon-button__icon"/>
                            </button>
                        </div>
                        {move || {
                            detail
                                .get()
                                .map(|data| {
                                    let total = data.nodes.len();
                                    view! {
                                        <header class="sheet-panel__header">
                                            <p>"Attached Nodes"</p>
                                            <h2>{data.form_name}</h2>
                                            <span class="forms-attached-sheet__count">{format!("{total} nodes")}</span>
                                        </header>
                                        <section class="sheet-panel__section">
                                            <label class="searchable-data-table__search searchable-data-table__control forms-attached-sheet__search">
                                                <Search/>
                                                <span class="sr-only">"Search attached nodes"</span>
                                                <input
                                                    type="search"
                                                    placeholder="Search attached nodes"
                                                    prop:value=move || search.get()
                                                    on:input=move |event| search.set(event_target_value(&event))
                                                />
                                            </label>
                                            <div class="forms-attached-sheet__list">
                                                {move || {
                                                    let nodes = filtered_nodes();
                                                    if nodes.is_empty() {
                                                        view! { <p class="forms-attached-sheet__empty">"No Attached Nodes to Display"</p> }.into_any()
                                                    } else {
                                                        nodes
                                                            .into_iter()
                                                            .map(|node| {
                                                                let node_title = node.title.clone();
                                                                view! {
                                                                    <a class="forms-attached-sheet__item" href=node.href title=node_title>
                                                                        <span>{node.label}</span>
                                                                        <small>{node.title}</small>
                                                                    </a>
                                                                }
                                                            })
                                                            .collect_view()
                                                            .into_any()
                                                    }
                                                }}
                                            </div>
                                        </section>
                                    }
                                    .into_any()
                                })
                                .unwrap_or_else(empty_view)
                        }}
                    </aside>
                </section>
            </Show>
        </Portal>
    }
}
