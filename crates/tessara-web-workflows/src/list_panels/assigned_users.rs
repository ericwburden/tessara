//! Assigned-user panels for the workflow list.

use crate::shared::{FormAttachmentLink, WorkflowAssignedUsersSheetData, user_count_label};
use icons::{ExternalLink, PanelRight};
use leptos::prelude::*;
use tessara_web_ui::{SideSheet, TableSearch, empty_view};

const ASSIGNED_USERS_SHEET_ID: &str = "workflow-assigned-users-sheet";

#[component]
pub(crate) fn WorkflowAssignedUsersList(
    users: Vec<FormAttachmentLink>,
    workflow_name: String,
    workflow_href: String,
    sheet: RwSignal<Option<WorkflowAssignedUsersSheetData>>,
) -> impl IntoView {
    let total_users = users.len();
    let users_for_sheet = users.clone();
    let workflow_name_for_sheet = workflow_name.clone();
    let workflow_href_for_sheet = workflow_href.clone();
    let workflow_href_for_expanded = workflow_href;

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
                        aria-haspopup="dialog"
                        aria-controls=ASSIGNED_USERS_SHEET_ID
                        aria-expanded=move || sheet
                            .get()
                            .as_ref()
                            .is_some_and(|detail| detail.workflow_href == workflow_href_for_expanded)
                            .to_string()
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
pub(crate) fn WorkflowAssignedUsersSheet(
    detail: RwSignal<Option<WorkflowAssignedUsersSheetData>>,
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
        <SideSheet
            id=ASSIGNED_USERS_SHEET_ID
            title="Assigned users"
            open
            on_close=close
            close_label="Close assigned users"
            class="forms-attached-sheet"
        >
            {move || {
                detail
                    .get()
                    .map(|data| {
                        let total = data.users.len();
                        view! {
                            <a class="icon-button sheet-panel__open forms-attached-sheet__open" href=data.workflow_href aria-label="Open workflow detail" title="Open workflow detail">
                                <ExternalLink class="icon-button__icon"/>
                            </a>
                            <header class="sheet-panel__header">
                                <p>"Assigned Users"</p>
                                <h2>{data.workflow_name}</h2>
                                <span class="forms-attached-sheet__count">{user_count_label(total)}</span>
                            </header>
                            <section class="sheet-panel__section">
                                <TableSearch
                                    value=Signal::derive(move || search.get())
                                    on_input=Callback::new(move |value| search.set(value))
                                    label="Search assigned users"
                                    placeholder="Search assigned users"
                                    class="forms-attached-sheet__search"
                                />
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
        </SideSheet>
    }
}

#[cfg(all(test, feature = "ssr"))]
mod tests {
    use super::*;

    #[test]
    fn assigned_users_trigger_exposes_its_dialog_relationship() {
        let html = Owner::new().with(|| {
            let href = "/workflows/workflow-1".to_string();
            let user = FormAttachmentLink {
                href: "/users/user-1".to_string(),
                label: "Casey".to_string(),
                title: "Assigned user".to_string(),
            };
            let sheet = RwSignal::new(Some(WorkflowAssignedUsersSheetData {
                workflow_name: "Inspection".to_string(),
                workflow_href: href.clone(),
                users: vec![user.clone()],
            }));

            view! {
                <WorkflowAssignedUsersList
                    users=vec![user]
                    workflow_name="Inspection".to_string()
                    workflow_href=href
                    sheet
                />
            }
            .to_html()
        });

        assert!(html.contains("aria-haspopup=\"dialog\""));
        assert!(html.contains(&format!("aria-controls=\"{ASSIGNED_USERS_SHEET_ID}\"")));
        assert!(html.contains("aria-expanded=\"true\""));
    }
}
