//! Related work table components for organization nodes.

use super::related_work_controls::{StatusFilterHeader, related_work_page_summary};
use super::types::{NodeDashboardLink, NodeFormLink, NodeSubmissionLink};
use crate::ui::{DataTable, SearchableDataTable, Timestamp};
use crate::utils::pagination::{
    pagination_current_page, pagination_page_count, pagination_page_start,
};
use crate::utils::text::{nonempty_text, sentence_label, text_matches};
use icons::Search;
use leptos::prelude::*;

#[component]
/// Renders the related responses table view.
pub(crate) fn RelatedResponsesTable(responses: Vec<NodeSubmissionLink>) -> impl IntoView {
    let search = RwSignal::new(String::new());
    let status_filter = RwSignal::new("all".to_string());
    let page_size = RwSignal::new(10usize);
    let page_index = RwSignal::new(0usize);
    let responses_for_filter = responses;
    let filtered_responses = Memo::new(move |_| {
        let query = search.get();
        let status = status_filter.get();
        responses_for_filter
            .iter()
            .filter(|response| status == "all" || response.status == status)
            .filter(|response| {
                text_matches(
                    &query,
                    &[
                        &response.form_name,
                        &response.version_label,
                        &response.status,
                        response.submitted_by.as_deref().unwrap_or("Unknown"),
                    ],
                )
            })
            .cloned()
            .collect::<Vec<_>>()
    });

    view! {
        <div class="searchable-data-table related-work-responsive-table">
            <div class="searchable-data-table__toolbar related-work-mobile-toolbar">
                <label class="searchable-data-table__search searchable-data-table__control">
                    <Search class="searchable-data-table__control-icon"/>
                    <span class="sr-only">"Search responses"</span>
                    <input
                        type="search"
                        placeholder="Search related responses"
                        prop:value=move || search.get()
                        on:input=move |event| search.set(event_target_value(&event))
                    />
                </label>
                <div class="related-work-mobile-filter">
                    <StatusFilterHeader status_filter compact_control=true/>
                </div>
            </div>
            <DataTable>
                <thead>
                    <tr>
                        <th scope="col">"Form name"</th>
                        <th scope="col">"Version"</th>
                        <th scope="col">
                            <StatusFilterHeader status_filter/>
                        </th>
                        <th scope="col">"Submitted Date"</th>
                        <th scope="col">"Submitted By"</th>
                    </tr>
                </thead>
                <tbody>
                    {move || {
                        let rows = filtered_responses.get();
                        if rows.is_empty() {
                            view! {
                                <tr>
                                    <td class="data-table__empty" colspan="5">"No Related Responses to Display"</td>
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
                                .map(|response| {
                                    let href = format!("/responses/{}", response.submission_id);
                                    let submitted_date = response
                                        .submitted_at
                                        .clone()
                                        .unwrap_or_else(|| response.created_at.clone());
                                    view! {
                                        <tr>
                                            <th scope="row">
                                                <a class="data-table__primary-link" href=href>{response.form_name}</a>
                                            </th>
                                            <td>{response.version_label}</td>
                                            <td>{sentence_label(&response.status)}</td>
                                            <td><Timestamp value=submitted_date/></td>
                                            <td>{response.submitted_by.unwrap_or_else(|| "Unknown".to_string())}</td>
                                        </tr>
                                    }
                                })
                                .collect_view()
                                .into_any()
                        }
                    }}
                </tbody>
            </DataTable>
            <div class="directory-table-pagination" aria-label="Related responses table pagination">
                <p>{move || related_work_page_summary(filtered_responses.get().len(), page_size.get(), page_index.get(), "related responses")}</p>
                <div class="directory-table-pagination__actions">
                    <label class="directory-table-pagination__page-size searchable-data-table__filter searchable-data-table__control">
                        <span>"Rows"</span>
                        <select
                            prop:value=move || page_size.get().to_string()
                            on:change=move |event| {
                                if let Ok(size) = event_target_value(&event).parse::<usize>() {
                                    page_size.set(size);
                                    page_index.set(0);
                                }
                            }
                        >
                            <option value="10">"10"</option>
                            <option value="25">"25"</option>
                            <option value="50">"50"</option>
                        </select>
                    </label>
                    <button
                        class="button button--compact button--secondary"
                        type="button"
                        disabled=move || pagination_current_page(filtered_responses.get().len(), page_size.get(), page_index.get()) == 0
                        on:click=move |_| {
                            page_index.update(|page| *page = page.saturating_sub(1));
                        }
                    >
                        "Previous"
                    </button>
                    <span>{move || {
                        let total_count = filtered_responses.get().len();
                        format!(
                            "Page {} of {}",
                            pagination_current_page(total_count, page_size.get(), page_index.get()) + 1,
                            pagination_page_count(total_count, page_size.get())
                        )
                    }}</span>
                    <button
                        class="button button--compact button--secondary"
                        type="button"
                        disabled=move || {
                            let total_count = filtered_responses.get().len();
                            pagination_current_page(total_count, page_size.get(), page_index.get()) + 1
                                >= pagination_page_count(total_count, page_size.get())
                        }
                        on:click=move |_| {
                            let last_page = pagination_page_count(filtered_responses.get().len(), page_size.get()).saturating_sub(1);
                            page_index.update(|page| *page = (*page + 1).min(last_page));
                        }
                    >
                        "Next"
                    </button>
                </div>
            </div>
            <div class="related-work-mobile-cards">
                {move || {
                    let rows = filtered_responses.get();
                    if rows.is_empty() {
                        view! { <p class="related-work-mobile-empty">"No Related Responses to Display"</p> }.into_any()
                    } else {
                        let total_count = rows.len();
                        let start = pagination_page_start(total_count, page_size.get(), page_index.get());
                        rows
                            .iter()
                            .skip(start)
                            .take(page_size.get())
                            .cloned()
                            .map(|response| {
                                let href = format!("/responses/{}", response.submission_id);
                                let submitted_date = response
                                    .submitted_at
                                    .clone()
                                    .unwrap_or_else(|| response.created_at.clone());
                                view! {
                                    <article class="related-work-mobile-card">
                                        <div class="related-work-mobile-card__header">
                                            <h4><a href=href>{response.form_name}</a></h4>
                                        </div>
                                        <dl>
                                            <div>
                                                <dt>"Version"</dt>
                                                <dd>{response.version_label}</dd>
                                            </div>
                                            <div>
                                                <dt>"Status"</dt>
                                                <dd>{sentence_label(&response.status)}</dd>
                                            </div>
                                            <div>
                                                <dt>"Submitted Date"</dt>
                                                <dd><Timestamp value=submitted_date/></dd>
                                            </div>
                                            <div>
                                                <dt>"Submitted By"</dt>
                                                <dd>{response.submitted_by.unwrap_or_else(|| "Unknown".to_string())}</dd>
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
            <div class="directory-table-pagination" aria-label="Related forms table pagination">
                <p>{move || related_work_page_summary(filtered_forms.get().len(), page_size.get(), page_index.get(), "related forms")}</p>
                <div class="directory-table-pagination__actions">
                    <label class="directory-table-pagination__page-size searchable-data-table__filter searchable-data-table__control">
                        <span>"Rows"</span>
                        <select
                            prop:value=move || page_size.get().to_string()
                            on:change=move |event| {
                                if let Ok(size) = event_target_value(&event).parse::<usize>() {
                                    page_size.set(size);
                                    page_index.set(0);
                                }
                            }
                        >
                            <option value="10">"10"</option>
                            <option value="25">"25"</option>
                            <option value="50">"50"</option>
                        </select>
                    </label>
                    <button
                        class="button button--compact button--secondary"
                        type="button"
                        disabled=move || pagination_current_page(filtered_forms.get().len(), page_size.get(), page_index.get()) == 0
                        on:click=move |_| {
                            page_index.update(|page| *page = page.saturating_sub(1));
                        }
                    >
                        "Previous"
                    </button>
                    <span>{move || {
                        let total_count = filtered_forms.get().len();
                        format!(
                            "Page {} of {}",
                            pagination_current_page(total_count, page_size.get(), page_index.get()) + 1,
                            pagination_page_count(total_count, page_size.get())
                        )
                    }}</span>
                    <button
                        class="button button--compact button--secondary"
                        type="button"
                        disabled=move || {
                            let total_count = filtered_forms.get().len();
                            pagination_current_page(total_count, page_size.get(), page_index.get()) + 1
                                >= pagination_page_count(total_count, page_size.get())
                        }
                        on:click=move |_| {
                            let last_page = pagination_page_count(filtered_forms.get().len(), page_size.get()).saturating_sub(1);
                            page_index.update(|page| *page = (*page + 1).min(last_page));
                        }
                    >
                        "Next"
                    </button>
                </div>
            </div>
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

#[component]
#[allow(unused_variables)]
/// Renders the related dashboards table view.
pub(crate) fn RelatedDashboardsTable(dashboards: Vec<NodeDashboardLink>) -> impl IntoView {
    let search = RwSignal::new(String::new());
    let page_size = RwSignal::new(10usize);
    let page_index = RwSignal::new(0usize);
    let dashboards_for_filter = dashboards;
    let filtered_dashboards = Memo::new(move |_| {
        let query = search.get();
        dashboards_for_filter
            .iter()
            .filter(|dashboard| {
                text_matches(
                    &query,
                    &[
                        &dashboard.dashboard_name,
                        &dashboard.component_count.to_string(),
                        dashboard.description.as_deref().unwrap_or("No description"),
                    ],
                )
            })
            .cloned()
            .collect::<Vec<_>>()
    });

    view! {
        <div class="related-work-responsive-table">
            <SearchableDataTable search_label="Search dashboards" placeholder="Search related dashboards" search>
                <thead>
                    <tr>
                        <th scope="col">"Dashboard name"</th>
                        <th scope="col">"Component Count"</th>
                        <th scope="col">"Description"</th>
                    </tr>
                </thead>
                <tbody>
                    {move || {
                        let rows = filtered_dashboards.get();
                        if rows.is_empty() {
                            view! {
                                <tr>
                                    <td class="data-table__empty" colspan="3">"No Related Dashboards to Display"</td>
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
                                .map(|dashboard| {
                                    let href = format!("/dashboards/{}", dashboard.dashboard_id);
                                    view! {
                                        <tr>
                                            <th scope="row">
                                                <a class="data-table__primary-link" href=href>{dashboard.dashboard_name}</a>
                                            </th>
                                            <td>{dashboard.component_count}</td>
                                            <td>{nonempty_text(dashboard.description.as_deref(), "No description")}</td>
                                        </tr>
                                    }
                                })
                                .collect_view()
                                .into_any()
                        }
                    }}
                </tbody>
            </SearchableDataTable>
            <div class="directory-table-pagination" aria-label="Related dashboards table pagination">
                <p>{move || related_work_page_summary(filtered_dashboards.get().len(), page_size.get(), page_index.get(), "related dashboards")}</p>
                <div class="directory-table-pagination__actions">
                    <label class="directory-table-pagination__page-size searchable-data-table__filter searchable-data-table__control">
                        <span>"Rows"</span>
                        <select
                            prop:value=move || page_size.get().to_string()
                            on:change=move |event| {
                                if let Ok(size) = event_target_value(&event).parse::<usize>() {
                                    page_size.set(size);
                                    page_index.set(0);
                                }
                            }
                        >
                            <option value="10">"10"</option>
                            <option value="25">"25"</option>
                            <option value="50">"50"</option>
                        </select>
                    </label>
                    <button
                        class="button button--compact button--secondary"
                        type="button"
                        disabled=move || pagination_current_page(filtered_dashboards.get().len(), page_size.get(), page_index.get()) == 0
                        on:click=move |_| {
                            page_index.update(|page| *page = page.saturating_sub(1));
                        }
                    >
                        "Previous"
                    </button>
                    <span>{move || {
                        let total_count = filtered_dashboards.get().len();
                        format!(
                            "Page {} of {}",
                            pagination_current_page(total_count, page_size.get(), page_index.get()) + 1,
                            pagination_page_count(total_count, page_size.get())
                        )
                    }}</span>
                    <button
                        class="button button--compact button--secondary"
                        type="button"
                        disabled=move || {
                            let total_count = filtered_dashboards.get().len();
                            pagination_current_page(total_count, page_size.get(), page_index.get()) + 1
                                >= pagination_page_count(total_count, page_size.get())
                        }
                        on:click=move |_| {
                            let last_page = pagination_page_count(filtered_dashboards.get().len(), page_size.get()).saturating_sub(1);
                            page_index.update(|page| *page = (*page + 1).min(last_page));
                        }
                    >
                        "Next"
                    </button>
                </div>
            </div>
            <div class="related-work-mobile-cards">
                {move || {
                    let rows = filtered_dashboards.get();
                    if rows.is_empty() {
                        view! { <p class="related-work-mobile-empty">"No Related Dashboards to Display"</p> }.into_any()
                    } else {
                        let total_count = rows.len();
                        let start = pagination_page_start(total_count, page_size.get(), page_index.get());
                        rows
                            .iter()
                            .skip(start)
                            .take(page_size.get())
                            .cloned()
                            .map(|dashboard| {
                                let href = format!("/dashboards/{}", dashboard.dashboard_id);
                                view! {
                                    <article class="related-work-mobile-card">
                                        <div class="related-work-mobile-card__header">
                                            <h4><a href=href>{dashboard.dashboard_name}</a></h4>
                                        </div>
                                        <dl>
                                            <div>
                                                <dt>"Component Count"</dt>
                                                <dd>{dashboard.component_count}</dd>
                                            </div>
                                            <div>
                                                <dt>"Description"</dt>
                                                <dd>{nonempty_text(dashboard.description.as_deref(), "No description")}</dd>
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
