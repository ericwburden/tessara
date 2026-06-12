//! Node type detail collection components.

use super::node_type_metadata_fields::NodeTypeMetadataList;
use crate::features::organization::{NodeTypeDefinition, NodeTypeFormLink};
use crate::ui::TablePaginationFooter;
use crate::utils::pagination::pagination_page_start;
use icons::Search;
use leptos::prelude::*;

/// Renders the node type scoped forms list view.
#[component]
pub(crate) fn NodeTypeScopedFormsList(forms: Vec<NodeTypeFormLink>) -> impl IntoView {
    let search = RwSignal::new(String::new());
    let page_size = RwSignal::new(10usize);
    let page_index = RwSignal::new(0usize);
    let has_forms = !forms.is_empty();
    let searchable_forms = forms;
    let filtered_forms = Memo::new(move |_| {
        let query = search.get().trim().to_lowercase();
        searchable_forms
            .iter()
            .filter(|form| {
                query.is_empty()
                    || form.form_name.to_lowercase().contains(&query)
                    || form.form_slug.to_lowercase().contains(&query)
            })
            .cloned()
            .collect::<Vec<_>>()
    });
    let total_count = Memo::new(move |_| filtered_forms.get().len());
    view! {
        <section class="organization-detail-card node-type-detail-list node-type-detail-list--wide">
            <div class="node-type-detail-list__header">
                <h3>"Scoped Forms"</h3>
                <label class="searchable-data-table__search searchable-data-table__control node-type-detail-list__search">
                    <Search class="searchable-data-table__control-icon"/>
                    <span class="sr-only">"Search scoped forms"</span>
                    <input
                        type="search"
                        placeholder="Search forms"
                        prop:value=move || search.get()
                        on:input=move |event| {
                            search.set(event_target_value(&event));
                            page_index.set(0);
                        }
                    />
                </label>
            </div>
            {if !has_forms {
                view! { <p class="muted">"No forms are scoped to this node type."</p> }.into_any()
            } else {
                view! {
                    <div class="capability-list node-type-scoped-forms-list">
                        {move || {
                            let visible_forms = filtered_forms.get();
                            if visible_forms.is_empty() {
                                view! { <div class="capability-list__item">"No scoped forms match this search."</div> }.into_any()
                            } else {
                                let total_count = visible_forms.len();
                                let start = pagination_page_start(total_count, page_size.get(), page_index.get());
                                visible_forms
                                    .iter()
                                    .skip(start)
                                    .take(page_size.get())
                                    .cloned()
                                    .map(|form| view! {
                                        <div class="capability-list__item">
                                            <strong>{form.form_name}</strong>
                                            <small>{form.form_slug}</small>
                                        </div>
                                    })
                                    .collect_view()
                                    .into_any()
                            }
                        }}
                    </div>
                    <TablePaginationFooter
                        aria_label="Scoped forms list pagination"
                        item_label="scoped forms"
                        total_count=total_count
                        page_size=page_size
                        page_index=page_index
                    />
                }
                .into_any()
            }}
        </section>
    }
}

/// Renders the node type detail collections view.
#[component]
pub(crate) fn NodeTypeDetailCollections(
    detail: Option<NodeTypeDefinition>,
    on_metadata_changed: impl Fn() + 'static + Copy + Send + Sync,
) -> impl IntoView {
    if let Some(detail) = detail {
        view! {
            <div class="administration-node-type-collections">
                <NodeTypeMetadataList
                    node_type_id=detail.id
                    fields=detail.metadata_fields
                    on_metadata_changed
                />
                <NodeTypeScopedFormsList forms=detail.scoped_forms/>
            </div>
        }
        .into_any()
    } else {
        view! { <div></div> }.into_any()
    }
}
