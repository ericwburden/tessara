//! List view components for the Forms feature.
//!
//! Keep collection tables, list filters, and list-page presentation here; detail/editor flows should stay in their dedicated modules.

use super::components::FormsNodeLineageFilter;
use crate::features::forms::{
    FormNodeFilterOption, FormSummary, active_form_version, form_version_label,
};
use crate::features::forms::{FormsAttachedNodesList, FormsAttachedNodesSheet};
use crate::features::forms::{form_attached_nodes, form_field_count_label, form_status_label};
use crate::features::shared::{FormsAttachedNodesSheetData, status_badge_class};
use crate::ui::{DataTable, TableFilterHeader, TablePaginationFooter};
use crate::utils::pagination::pagination_page_start;
use icons::Search;
use leptos::prelude::*;

#[component]
/// Renders the forms list view.
pub(crate) fn FormsList(
    forms: Vec<FormSummary>,
    search: RwSignal<String>,
    status_filter: RwSignal<String>,
    node_filter_query: RwSignal<String>,
    selected_node_id: RwSignal<Option<String>>,
    status_options: Vec<String>,
    node_filter_options: Vec<FormNodeFilterOption>,
) -> impl IntoView {
    let mut table_forms = forms.clone();
    table_forms.sort_by(|left, right| {
        left.name
            .to_lowercase()
            .cmp(&right.name.to_lowercase())
            .then(left.id.cmp(&right.id))
    });
    let card_forms = table_forms.clone();
    let page_size = RwSignal::new(10usize);
    let page_index = RwSignal::new(0usize);
    let total_count = table_forms.len();
    let total_count = Memo::new(move |_| total_count);
    let attached_nodes_sheet = RwSignal::new(None::<FormsAttachedNodesSheetData>);

    view! {
        <div class="forms-list forms-list-responsive-table">
            <div class="searchable-data-table">
                <div class="searchable-data-table__toolbar forms-list__toolbar">
                    <label class="searchable-data-table__search searchable-data-table__control">
                        <Search class="searchable-data-table__control-icon"/>
                        <span class="sr-only">"Search forms"</span>
                        <input
                            type="search"
                            placeholder="Search forms"
                            prop:value=move || search.get()
                            on:input=move |event| search.set(event_target_value(&event))
                        />
                    </label>
                </div>
                <DataTable>
                <thead>
                    <tr>
                        <th scope="col">"Form name"</th>
                        <th scope="col">
                            <div class="data-table-filter">
                                <span>"Attached To"</span>
                                <FormsNodeLineageFilter
                                    options=node_filter_options
                                    selected_node_id
                                    query=node_filter_query
                                />
                            </div>
                        </th>
                        <th class="data-table__cell--center" scope="col">"Active version"</th>
                        <th class="data-table__cell--center" scope="col">
                            <TableFilterHeader
                                label="Status"
                                all_label="All statuses"
                                filter=status_filter
                                options=status_options
                            />
                        </th>
                        <th class="data-table__cell--center" scope="col">"Fields"</th>
                    </tr>
                </thead>
                <tbody>
                    {move || if table_forms.is_empty() {
                        view! {
                            <tr>
                                <td class="data-table__empty" colspan="5">"No Forms to Display"</td>
                            </tr>
                        }
                        .into_any()
                    } else {
                        table_forms
                            .iter()
                            .skip(pagination_page_start(total_count.get(), page_size.get(), page_index.get()))
                            .take(page_size.get())
                            .cloned()
                            .map(|form| {
                                let href = format!("/forms/{}", form.id);
                                let active_version = active_form_version(&form);
                                let status = active_version
                                    .map(|version| version.status.as_str())
                                    .unwrap_or("none");
                                let name = form.name.clone();
                                let attached_nodes = form_attached_nodes(active_version);
                                let attached_nodes_form_name = name.clone();
                                let version_label = form_version_label(active_version);
                                let status_label = form_status_label(active_version);
                                let field_count = form_field_count_label(active_version);
                                view! {
                                    <tr>
                                        <th scope="row">
                                            <a class="data-table__primary-link" href=href.clone()>{name}</a>
                                        </th>
                                        <td>
                                            <FormsAttachedNodesList
                                                nodes=attached_nodes
                                                form_name=attached_nodes_form_name
                                                form_href=href
                                                sheet=attached_nodes_sheet
                                            />
                                        </td>
                                        <td class="data-table__cell--center">{version_label}</td>
                                        <td class="data-table__cell--center"><span class=status_badge_class(status)>{status_label}</span></td>
                                        <td class="data-table__cell--center">{field_count}</td>
                                    </tr>
                                }
                            })
                            .collect_view()
                            .into_any()
                    }}
                </tbody>
                </DataTable>
                <TablePaginationFooter
                    aria_label="Forms table pagination"
                    item_label="forms"
                    total_count=total_count
                    page_size=page_size
                    page_index=page_index
                />
            </div>
            <div class="forms-list-mobile-cards">
                {move || if card_forms.is_empty() {
                    view! { <p class="forms-list-mobile-empty">"No Forms to Display"</p> }.into_any()
                } else {
                    card_forms
                        .iter()
                        .skip(pagination_page_start(total_count.get(), page_size.get(), page_index.get()))
                        .take(page_size.get())
                        .cloned()
                        .map(|form| {
                            let href = format!("/forms/{}", form.id);
                            let active_version = active_form_version(&form);
                            let status = active_version
                                .map(|version| version.status.as_str())
                                .unwrap_or("none");
                            let name = form.name.clone();
                            let attached_nodes = form_attached_nodes(active_version);
                            let attached_nodes_form_name = name.clone();
                            let version_label = form_version_label(active_version);
                            let status_label = form_status_label(active_version);
                            let field_count = form_field_count_label(active_version);
                            view! {
                                <article class="forms-list-mobile-card">
                                    <div class="forms-list-mobile-card__header">
                                        <div>
                                            <h3><a href=href.clone()>{name}</a></h3>
                                        </div>
                                    </div>
                                    <dl>
                                        <div>
                                            <dt>"Attached To"</dt>
                                            <dd>
                                                <FormsAttachedNodesList
                                                    nodes=attached_nodes
                                                    form_name=attached_nodes_form_name
                                                    form_href=href
                                                    sheet=attached_nodes_sheet
                                                />
                                            </dd>
                                        </div>
                                        <div>
                                            <dt>"Active version"</dt>
                                            <dd>{version_label}</dd>
                                        </div>
                                        <div>
                                            <dt>"Status"</dt>
                                            <dd><span class=status_badge_class(status)>{status_label}</span></dd>
                                        </div>
                                        <div>
                                            <dt>"Fields"</dt>
                                            <dd>{field_count}</dd>
                                        </div>
                                    </dl>
                                </article>
                            }
                        })
                        .collect_view()
                        .into_any()
                }}
            </div>
            <FormsAttachedNodesSheet detail=attached_nodes_sheet/>
        </div>
    }
}
