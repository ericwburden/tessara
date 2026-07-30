//! Attached-node sheet for the forms list.

use crate::FormsAttachedNodesSheetData;
use icons::ExternalLink;
use leptos::prelude::*;
use tessara_module_ui::{SideSheet, TableSearch, empty_view};

use super::ATTACHED_NODES_SHEET_ID;

#[component]
pub(crate) fn FormsAttachedNodesSheet(
    detail: RwSignal<Option<FormsAttachedNodesSheetData>>,
) -> impl IntoView {
    let search = RwSignal::new(String::new());
    let close = Callback::new(move |_| {
        detail.set(None);
        search.set(String::new());
    });
    let open = Signal::derive(move || detail.get().is_some());
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
        <SideSheet
            id=ATTACHED_NODES_SHEET_ID
            title="Attached organization nodes"
            open
            on_close=close
            close_label="Close attached nodes"
            class="forms-attached-sheet"
        >
            {move || {
                detail
                    .get()
                    .map(|data| {
                        let total = data.nodes.len();
                        view! {
                            <a class="icon-button sheet-panel__open forms-attached-sheet__open" href=data.form_href aria-label="Open form detail" title="Open form detail">
                                <ExternalLink class="icon-button__icon"/>
                            </a>
                            <header class="sheet-panel__header">
                                <p>"Attached Nodes"</p>
                                <h2>{data.form_name}</h2>
                                <span class="forms-attached-sheet__count">{format!("{total} nodes")}</span>
                            </header>
                            <section class="sheet-panel__section">
                                <TableSearch
                                    value=Signal::derive(move || search.get())
                                    on_input=Callback::new(move |value| search.set(value))
                                    label="Search attached nodes"
                                    placeholder="Search attached nodes"
                                    class="forms-attached-sheet__search"
                                />
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
        </SideSheet>
    }
}
