//! Related forms table for organization node detail.

use super::super::types::NodeFormLink;
use crate::ui::{SearchableDataTable, TablePaginationFooter};
use crate::utils::pagination::pagination_page_start;
use crate::utils::text::text_matches;
use leptos::prelude::*;

#[component]
/// Renders the related forms table view.
pub(crate) fn RelatedFormsTable(forms: Vec<NodeFormLink>) -> impl IntoView {
    let search = RwSignal::new(String::new());
    let page_size = RwSignal::new(10usize);
    let page_index = RwSignal::new(0usize);
    let forms_for_filter = forms;
    let filtered_forms = Memo::new(move |_| {
        let query = search.get();
        forms_for_filter
            .iter()
            .filter(|form| {
                text_matches(
                    &query,
                    &[
                        &form.form_name,
                        &form.form_slug,
                        form.active_version_label.as_deref().unwrap_or(""),
                    ],
                )
            })
            .cloned()
            .collect::<Vec<_>>()
    });
    let total_count = Memo::new(move |_| filtered_forms.get().len());

    view! {
        <div class="related-work-responsive-table">
            <SearchableDataTable search_label="Search forms" placeholder="Search related forms" search>
                <thead>
                    <tr>
                        <th scope="col">"Form name"</th>
                        <th scope="col">"Slug"</th>
                        <th scope="col">"Active version"</th>
                    </tr>
                </thead>
                <tbody>
                    {move || {
                        let rows = filtered_forms.get();
                        if rows.is_empty() {
                            view! {
                                <tr>
                                    <td class="data-table__empty" colspan="3">"No Related Forms to Display"</td>
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
                                .map(|form| {
                                    let href = format!("/forms/{}", form.form_id);
                                    view! {
                                        <tr>
                                            <th scope="row">
                                                <a class="data-table__primary-link" href=href>{form.form_name}</a>
                                            </th>
                                            <td>{form.form_slug}</td>
                                            <td>{form.active_version_label.unwrap_or_else(|| "-".to_string())}</td>
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
                aria_label="Related forms table pagination"
                item_label="related forms"
                total_count=total_count
                page_size=page_size
                page_index=page_index
            />
            <div class="related-work-mobile-cards">
                {move || {
                    let rows = filtered_forms.get();
                    if rows.is_empty() {
                        view! { <p class="related-work-mobile-empty">"No Related Forms to Display"</p> }.into_any()
                    } else {
                        let total_count = rows.len();
                        let start = pagination_page_start(total_count, page_size.get(), page_index.get());
                        rows
                            .iter()
                            .skip(start)
                            .take(page_size.get())
                            .cloned()
                            .map(|form| {
                                let href = format!("/forms/{}", form.form_id);
                                view! {
                                    <article class="related-work-mobile-card">
                                        <div class="related-work-mobile-card__header">
                                            <h4><a href=href>{form.form_name}</a></h4>
                                        </div>
                                        <dl>
                                            <div>
                                                <dt>"Slug"</dt>
                                                <dd>{form.form_slug}</dd>
                                            </div>
                                            <div>
                                                <dt>"Active version"</dt>
                                                <dd>{form.active_version_label.unwrap_or_else(|| "-".to_string())}</dd>
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
