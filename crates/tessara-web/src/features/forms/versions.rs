use crate::features::organization::form_version_sort_label;
use crate::features::shared::{
    FormVersionSummary, form_version_desc_sort_key, sentence_label, status_badge_class,
};
use crate::ui::{DataTable, Timestamp, empty_view};
use crate::utils::text::nonempty_text;
use leptos::prelude::*;

#[component]
pub(crate) fn FormVersionsTable(versions: Vec<FormVersionSummary>) -> impl IntoView {
    const DEFAULT_VISIBLE_FORM_VERSIONS: usize = 5;

    let visible_count = RwSignal::new(DEFAULT_VISIBLE_FORM_VERSIONS);
    let mut sorted_versions = versions;
    sorted_versions.sort_by(|left, right| {
        form_version_desc_sort_key(right).cmp(&form_version_desc_sort_key(left))
    });
    let table_versions = sorted_versions.clone();
    let card_versions = sorted_versions.clone();
    let version_count = sorted_versions.len();

    view! {
        <div class="forms-list-responsive-table">
            <DataTable>
                <thead>
                    <tr>
                        <th scope="col">"Version"</th>
                        <th scope="col">"Status"</th>
                        <th scope="col">"Compatibility"</th>
                        <th scope="col">"Published"</th>
                        <th class="data-table__cell--center" scope="col">"Fields"</th>
                    </tr>
                </thead>
                <tbody>
                    {if table_versions.is_empty() {
                        view! {
                            <tr>
                                <td class="data-table__empty" colspan="5">"No Versions to Display"</td>
                            </tr>
                        }
                        .into_any()
                    } else {
                        table_versions
                            .iter()
                            .take(visible_count.get())
                            .cloned()
                            .map(|version| {
                                let status = version.status.clone();
                                let published_at = version.published_at.clone();
                                view! {
                                    <tr>
                                        <th scope="row">{form_version_sort_label(&version)}</th>
                                        <td><span class=status_badge_class(&status)>{sentence_label(&status)}</span></td>
                                        <td>{nonempty_text(version.compatibility_group_name.as_deref(), "-")}</td>
                                        <td>
                                            {published_at
                                                .map(|value| view! { <Timestamp value/> }.into_any())
                                                .unwrap_or_else(|| view! { <span>"-"</span> }.into_any())}
                                        </td>
                                        <td class="data-table__cell--center">{version.field_count.to_string()}</td>
                                    </tr>
                                }
                            })
                            .collect_view()
                            .into_any()
                    }}
                </tbody>
            </DataTable>
            <div class="forms-list-mobile-cards">
                {if card_versions.is_empty() {
                    view! { <p class="forms-list-mobile-empty">"No Versions to Display"</p> }.into_any()
                } else {
                    card_versions
                        .iter()
                        .take(visible_count.get())
                        .cloned()
                        .map(|version| {
                            let status = version.status.clone();
                            let published_at = version.published_at.clone();
                            view! {
                                <article class="forms-list-mobile-card">
                                    <div class="forms-list-mobile-card__header">
                                        <h3>{form_version_sort_label(&version)}</h3>
                                    </div>
                                    <dl>
                                        <div>
                                            <dt>"Status"</dt>
                                            <dd><span class=status_badge_class(&status)>{sentence_label(&status)}</span></dd>
                                        </div>
                                        <div>
                                            <dt>"Compatibility"</dt>
                                            <dd>{nonempty_text(version.compatibility_group_name.as_deref(), "-")}</dd>
                                        </div>
                                        <div>
                                            <dt>"Published"</dt>
                                            <dd>
                                                {published_at
                                                    .map(|value| view! { <Timestamp value/> }.into_any())
                                                    .unwrap_or_else(|| view! { <span>"-"</span> }.into_any())}
                                            </dd>
                                        </div>
                                        <div>
                                            <dt>"Fields"</dt>
                                            <dd>{version.field_count.to_string()}</dd>
                                        </div>
                                    </dl>
                                </article>
                            }
                        })
                        .collect_view()
                        .into_any()
                }}
            </div>
            {move || {
                if version_count > visible_count.get() {
                    let remaining = version_count.saturating_sub(visible_count.get());
                    view! {
                        <button
                            class="button button--compact button--secondary form-versions-load-more"
                            type="button"
                            on:click=move |_| {
                                visible_count.update(|count| {
                                    *count = (*count + DEFAULT_VISIBLE_FORM_VERSIONS).min(version_count);
                                });
                            }
                        >
                            {format!("Load More ({remaining} older)")}
                        </button>
                    }
                    .into_any()
                } else {
                    empty_view()
                }
            }}
        </div>
    }
}
