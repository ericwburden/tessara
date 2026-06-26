//! Mobile card rendering for the forms list.

use crate::features::forms::FormSummary;
use crate::features::forms::{
    FormsAttachedNodesList, active_form_version, form_attached_nodes, form_field_count_label,
    form_status_label, form_version_label,
};
use crate::features::shared::{FormsAttachedNodesSheetData, status_badge_class};
use crate::utils::pagination::pagination_page_start;
use leptos::prelude::*;

#[component]
pub(super) fn FormsMobileCards(
    forms: Vec<FormSummary>,
    total_count: Memo<usize>,
    page_size: RwSignal<usize>,
    page_index: RwSignal<usize>,
    attached_nodes_sheet: RwSignal<Option<FormsAttachedNodesSheetData>>,
) -> impl IntoView {
    view! {
        <div class="forms-list-mobile-cards">
            {move || if forms.is_empty() {
                view! { <p class="forms-list-mobile-empty">"No Forms to Display"</p> }.into_any()
            } else {
                forms
                    .iter()
                    .skip(pagination_page_start(total_count.get(), page_size.get(), page_index.get()))
                    .take(page_size.get())
                    .map(|form| {
                        let href = format!("/forms/{}", form.id);
                        let active_version = active_form_version(form);
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
    }
}
