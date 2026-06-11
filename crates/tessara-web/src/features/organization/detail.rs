//! Detail view components for the Organization feature.
//!
//! Keep read-focused panels and detail-page presentation here; mutation workflows should live in editor or API modules.

use super::related_work::RelatedWorkSummary;
use crate::features::organization::types::OrganizationNodeDetail;
use crate::ui::empty_view;
use crate::utils::metadata::metadata_rows;
use icons::{ExternalLink, X};
use leptos::portal::Portal;
use leptos::prelude::*;

#[component]
/// Renders the organization detail sheet view.
pub(crate) fn OrganizationDetailSheet(
    detail: RwSignal<Option<OrganizationNodeDetail>>,
    is_loading: RwSignal<bool>,
    error: RwSignal<Option<String>>,
) -> impl IntoView {
    let close = move |_| {
        detail.set(None);
        error.set(None);
        is_loading.set(false);
    };

    view! {
        <Portal>
            <Show when=move || detail.get().is_some() || is_loading.get() || error.get().is_some()>
                <section class="sheet-overlay organization-detail-overlay" aria-label="Organization detail overlay">
                    <button class="sheet-overlay__scrim" type="button" aria-label="Close details" on:click=close></button>
                    <aside class="sheet-panel blurred-surface organization-detail-sheet" role="dialog" aria-modal="true" aria-label="Organization details">
                        <div class="sheet-panel__actions">
                            {move || {
                                detail
                                    .get()
                                    .map(|node_detail| {
                                        let href = format!("/organization/{}", node_detail.id);
                                        view! {
                                            <a class="icon-button sheet-panel__open" href=href aria-label="Open detail page" title="Open detail page">
                                                <ExternalLink class="icon-button__icon"/>
                                            </a>
                                        }
                                        .into_any()
                                    })
                                    .unwrap_or_else(empty_view)
                            }}
                            <button class="icon-button sheet-panel__close" type="button" aria-label="Close details" title="Close details" on:click=close>
                                <X class="icon-button__icon"/>
                            </button>
                        </div>
                        {move || {
                            if is_loading.get() {
                                view! {
                                    <div class="sheet-panel__state" aria-live="polite">
                                        <h2>"Loading details"</h2>
                                        <p>"Fetching organization node details."</p>
                                    </div>
                                }
                                .into_any()
                            } else if let Some(message) = error.get() {
                                view! {
                                    <div class="sheet-panel__state is-error" role="alert">
                                        <h2>"Details unavailable"</h2>
                                        <p>{message}</p>
                                    </div>
                                }
                                .into_any()
                            } else if let Some(node_detail) = detail.get() {
                                view! { <OrganizationDetailContent detail=node_detail/> }.into_any()
                            } else {
                                empty_view()
                            }
                        }}
                    </aside>
                </section>
            </Show>
        </Portal>
    }
}

#[component]
/// Renders the organization detail content view.
pub(crate) fn OrganizationDetailContent(detail: OrganizationNodeDetail) -> impl IntoView {
    let metadata_rows = metadata_rows(&detail.metadata);
    let node_type = detail.node_type_singular_label.clone();

    view! {
        <header class="sheet-panel__header">
            <p>{format!("{} Detail", node_type)}</p>
            <h2>{detail.name.clone()}</h2>
        </header>
        <section class="sheet-panel__section">
            <h3>"Details"</h3>
            <DynamicInfoTable rows=vec![
                ("Parent".to_string(), detail.parent_node_name.clone().unwrap_or_else(|| "Top-level".to_string())),
                ("Type".to_string(), detail.node_type_name.clone()),
                ("Plural".to_string(), detail.node_type_plural_label.clone()),
            ]/>
        </section>
        <section class="sheet-panel__section">
            <h3>"Metadata"</h3>
            {if metadata_rows.is_empty() {
                view! { <p class="muted">"No metadata recorded."</p> }.into_any()
            } else {
                view! { <DynamicInfoTable rows=metadata_rows/> }.into_any()
            }}
        </section>
        <section class="sheet-panel__section">
            <h3>"Related Work"</h3>
            <RelatedWorkSummary detail cards_only=true/>
        </section>
    }
}

#[component]
/// Renders the organization detail full content view.
pub(crate) fn OrganizationDetailFullContent(detail: OrganizationNodeDetail) -> impl IntoView {
    let metadata_rows = metadata_rows(&detail.metadata);
    let node_type = detail.node_type_singular_label.clone();

    view! {
        <div class="organization-detail-content">
            <header class="organization-detail-content__header">
                <p>{format!("{} Detail", node_type)}</p>
                <h3>{detail.name.clone()}</h3>
            </header>
            <div class="organization-detail-content__grid">
                <section class="organization-detail-card">
                    <h3>"Details"</h3>
                    <DynamicInfoTable rows=vec![
                        ("Parent".to_string(), detail.parent_node_name.clone().unwrap_or_else(|| "Top-level".to_string())),
                        ("Type".to_string(), detail.node_type_name.clone()),
                        ("Plural".to_string(), detail.node_type_plural_label.clone()),
                    ]/>
                </section>
                <section class="organization-detail-card">
                    <h3>"Metadata"</h3>
                    {if metadata_rows.is_empty() {
                        view! { <p class="muted">"No metadata recorded."</p> }.into_any()
                    } else {
                        view! { <DynamicInfoTable rows=metadata_rows/> }.into_any()
                    }}
                </section>
                <section class="organization-detail-card organization-detail-card--wide">
                    <h3>"Related Work"</h3>
                    <RelatedWorkSummary detail/>
                </section>
            </div>
        </div>
    }
}

#[component]
/// Renders the dynamic info table view.
pub(crate) fn DynamicInfoTable(rows: Vec<(String, String)>) -> impl IntoView {
    view! {
        <table class="info-list-table">
            <tbody>
                {rows
                    .into_iter()
                    .map(|(label, value)| view! {
                        <tr>
                            <th scope="row">{label}</th>
                            <td>{value}</td>
                        </tr>
                    })
                    .collect_view()}
            </tbody>
        </table>
    }
}
