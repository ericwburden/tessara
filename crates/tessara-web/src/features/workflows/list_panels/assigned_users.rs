//! Assigned-user panels for the workflow list.

use crate::features::shared::{
    FormAttachmentLink, WorkflowAssignedUsersSheetData, user_count_label,
};
use crate::ui::empty_view;
use icons::{ExternalLink, PanelRight, Search, X};
use leptos::portal::Portal;
use leptos::prelude::*;

#[component]
pub(in crate::features::workflows) fn WorkflowAssignedUsersList(
    users: Vec<FormAttachmentLink>,
    workflow_name: String,
    workflow_href: String,
    sheet: RwSignal<Option<WorkflowAssignedUsersSheetData>>,
) -> impl IntoView {
    let total_users = users.len();
    let users_for_sheet = users.clone();
    let workflow_name_for_sheet = workflow_name.clone();
    let workflow_href_for_sheet = workflow_href.clone();

    view! {
        <div class="forms-attached-list">
            {if total_users == 0 {
                view! { <p>"No active assignments"</p> }.into_any()
            } else {
                view! {
                    <button
                        class="forms-attached-list__more"
                        type="button"
                        aria-label=format!("View assigned users for {workflow_name_for_sheet}")
                        title="Opens detail panel"
                        on:click=move |_| {
                            sheet.set(Some(WorkflowAssignedUsersSheetData {
                                workflow_name: workflow_name_for_sheet.clone(),
                                workflow_href: workflow_href_for_sheet.clone(),
                                users: users_for_sheet.clone(),
                            }));
                        }
                    >
                        <span>{user_count_label(total_users)}</span>
                        <PanelRight class="forms-attached-list__icon"/>
                    </button>
                }
                .into_any()
            }}
        </div>
    }
}

#[component]
pub(in crate::features::workflows) fn WorkflowAssignedUsersSheet(
    detail: RwSignal<Option<WorkflowAssignedUsersSheetData>>,
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
                data.users
                    .into_iter()
                    .filter(|user| {
                        query.is_empty()
                            || user.label.to_lowercase().contains(&query)
                            || user.title.to_lowercase().contains(&query)
                    })
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default()
    };

    view! {
        <Portal>
            <Show when=move || detail.get().is_some()>
                <section class="sheet-overlay forms-attached-overlay" aria-label="Assigned users">
                    <button class="sheet-overlay__scrim" type="button" aria-label="Close assigned users" on:click=close></button>
                    <aside class="sheet-panel blurred-surface forms-attached-sheet" role="dialog" aria-modal="true" aria-label="Assigned users">
                        <div class="sheet-panel__actions">
                            {move || {
                                detail
                                    .get()
                                    .map(|data| {
                                        view! {
                                            <a class="icon-button sheet-panel__open" href=data.workflow_href aria-label="Open workflow detail" title="Open workflow detail">
                                                <ExternalLink class="icon-button__icon"/>
                                            </a>
                                        }
                                        .into_any()
                                    })
                                    .unwrap_or_else(empty_view)
                            }}
                            <button class="icon-button sheet-panel__close" type="button" aria-label="Close assigned users" title="Close assigned users" on:click=close>
                                <X class="icon-button__icon"/>
                            </button>
                        </div>
                        {move || {
                            detail
                                .get()
                                .map(|data| {
                                    let total = data.users.len();
                                    view! {
                                        <header class="sheet-panel__header">
                                            <p>"Assigned Users"</p>
                                            <h2>{data.workflow_name}</h2>
                                            <span class="forms-attached-sheet__count">{user_count_label(total)}</span>
                                        </header>
                                        <section class="sheet-panel__section">
                                            <label class="searchable-data-table__search searchable-data-table__control forms-attached-sheet__search">
                                                <Search class="searchable-data-table__control-icon"/>
                                                <span class="sr-only">"Search assigned users"</span>
                                                <input
                                                    type="search"
                                                    placeholder="Search assigned users"
                                                    prop:value=move || search.get()
                                                    on:input=move |event| search.set(event_target_value(&event))
                                                />
                                            </label>
                                            <div class="forms-attached-sheet__list">
                                                {move || {
                                                    let users = filtered_nodes();
                                                    if users.is_empty() {
                                                        view! { <p class="forms-attached-sheet__empty">"No Assigned Users to Display"</p> }.into_any()
                                                    } else {
                                                        users
                                                            .into_iter()
                                                            .map(|user| {
                                                                let user_title = user.title.clone();
                                                                view! {
                                                                    <a class="forms-attached-sheet__item" href=user.href title=user_title>
                                                                        <span>{user.label}</span>
                                                                        <small>{user.title}</small>
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
