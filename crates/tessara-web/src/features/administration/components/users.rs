//! User list Administration components.

use super::super::display::{
    admin_user_role_names, admin_user_status_key, admin_user_status_label,
};
use crate::features::administration::models::AdminUserSummary;
use crate::features::shared::status_badge_class;
#[cfg(feature = "hydrate")]
use crate::http::navigate_to_href;
use crate::ui::{DataTable, DropdownMenu, TableFilterHeader, TablePaginationFooter};
use crate::utils::pagination::pagination_page_start;
use icons::{PanelRight, Pencil, Search};
use leptos::prelude::*;

/// Renders the administration users list view.
#[component]
pub(crate) fn AdministrationUsersList(
    users: Vec<AdminUserSummary>,
    search: RwSignal<String>,
    status_filter: RwSignal<String>,
    role_filter: RwSignal<String>,
    role_options: Vec<String>,
) -> impl IntoView {
    let table_users = users.clone();
    let card_users = users.clone();
    let page_size = RwSignal::new(10usize);
    let page_index = RwSignal::new(0usize);
    let total_count = users.len();
    let total_count = Memo::new(move |_| total_count);

    view! {
        <div class="forms-list forms-list-responsive-table administration-users-list">
            <div class="searchable-data-table">
                <div class="searchable-data-table__toolbar forms-list__toolbar">
                    <label class="searchable-data-table__search searchable-data-table__control">
                        <Search class="searchable-data-table__control-icon"/>
                        <span class="sr-only">"Search users"</span>
                        <input
                            type="search"
                            placeholder="Search users"
                            prop:value=move || search.get()
                            on:input=move |event| {
                                search.set(event_target_value(&event));
                                page_index.set(0);
                            }
                        />
                    </label>
                </div>

                <DataTable>
                    <thead>
                        <tr>
                            <th scope="col">"User"</th>
                            <th scope="col">
                                <TableFilterHeader
                                    label="Role"
                                    all_label="All Roles"
                                    filter=role_filter
                                    options=role_options
                                />
                            </th>
                            <th class="data-table__cell--center" scope="col">
                                <TableFilterHeader
                                    label="Status"
                                    all_label="All Statuses"
                                    filter=status_filter
                                    options=vec!["active".to_string(), "inactive".to_string()]
                                />
                            </th>
                            <th class="data-table__cell--center" scope="col">"Roles"</th>
                            <th class="data-table__cell--center" scope="col">"Actions"</th>
                        </tr>
                    </thead>
                    <tbody>
                        {move || {
                            if table_users.is_empty() {
                                view! {
                                    <tr>
                                        <td class="data-table__empty" colspan="5">"No Users to Display"</td>
                                    </tr>
                                }
                                .into_any()
                            } else {
                                let total_count = table_users.len();
                                let start = pagination_page_start(total_count, page_size.get(), page_index.get());
                                table_users
                                    .iter()
                                    .skip(start)
                                    .take(page_size.get())
                                    .cloned()
                                    .map(|user| {
                                        let status_key = admin_user_status_key(&user);
                                        let status_label = admin_user_status_label(&user);
                                        let role_names = admin_user_role_names(&user);
                                        let detail_href = format!("/administration/users/{}", user.id);
                                        let edit_href = format!("/administration/users/{}/edit", user.id);
                                        let display_name = user.display_name.clone();
                                        let detail_href_for_click = detail_href.clone();
                                        let edit_href_for_click = edit_href.clone();
                                        view! {
                                            <tr>
                                                <th scope="row">
                                                    <a class="data-table__primary-link" href=detail_href>{user.display_name}</a>
                                                    <small class="workflow-assignment-step-meta">{user.email}</small>
                                                </th>
                                                <td>{role_names}</td>
                                                <td class="data-table__cell--center">
                                                    <span class=status_badge_class(status_key)>{status_label}</span>
                                                </td>
                                                <td class="data-table__cell--center">{user.roles.len()}</td>
                                                <td class="data-table__cell--center">
                                                    <DropdownMenu label=format!("Open actions for {display_name}")>
                                                        <button
                                                            class="dropdown-menu__item"
                                                            type="button"
                                                            role="menuitem"
                                                            on:click=move |_| {
                                                                #[cfg(feature = "hydrate")]
                                                                navigate_to_href(&detail_href_for_click);
                                                                #[cfg(not(feature = "hydrate"))]
                                                                let _ = &detail_href_for_click;
                                                            }
                                                        >
                                                            <PanelRight class="dropdown-menu__item-icon"/>
                                                            <span>"View Details"</span>
                                                        </button>
                                                        <button
                                                            class="dropdown-menu__item"
                                                            type="button"
                                                            role="menuitem"
                                                            on:click=move |_| {
                                                                #[cfg(feature = "hydrate")]
                                                                navigate_to_href(&edit_href_for_click);
                                                                #[cfg(not(feature = "hydrate"))]
                                                                let _ = &edit_href_for_click;
                                                            }
                                                        >
                                                            <Pencil class="dropdown-menu__item-icon"/>
                                                            <span>"Edit Account"</span>
                                                        </button>
                                                    </DropdownMenu>
                                                </td>
                                            </tr>
                                        }
                                    })
                                    .collect_view()
                                    .into_any()
                            }
                        }}
                    </tbody>
                </DataTable>
                <TablePaginationFooter
                    aria_label="Administration users table pagination"
                    item_label="users"
                    total_count=total_count
                    page_size=page_size
                    page_index=page_index
                />
            </div>

            <div class="forms-list-mobile-cards administration-users-mobile-cards">
                {move || {
                    if card_users.is_empty() {
                        view! { <p class="forms-list-mobile-empty">"No Users to Display"</p> }.into_any()
                    } else {
                        let total_count = card_users.len();
                        let start = pagination_page_start(total_count, page_size.get(), page_index.get());
                        card_users
                            .iter()
                            .skip(start)
                            .take(page_size.get())
                            .cloned()
                            .map(|user| {
                                let status_key = admin_user_status_key(&user);
                                let status_label = admin_user_status_label(&user);
                                let role_names = admin_user_role_names(&user);
                                let detail_href = format!("/administration/users/{}", user.id);
                                let edit_href = format!("/administration/users/{}/edit", user.id);
                                view! {
                                    <article class="forms-list-mobile-card administration-user-mobile-card">
                                        <div class="forms-list-mobile-card__header">
                                            <div>
                                                <h3><a href=detail_href.clone()>{user.display_name}</a></h3>
                                                <span>{user.email}</span>
                                            </div>
                                            <span class=status_badge_class(status_key)>{status_label}</span>
                                        </div>
                                        <dl>
                                            <div>
                                                <dt>"Roles"</dt>
                                                <dd>{role_names}</dd>
                                            </div>
                                            <div>
                                                <dt>"Role Count"</dt>
                                                <dd>{user.roles.len()}</dd>
                                            </div>
                                        </dl>
                                        <div class="workflow-assignment-mobile-card__actions">
                                            <a class="button button--compact" href=detail_href>"View Details"</a>
                                            <a class="button button--compact button--secondary" href=edit_href>"Edit Account"</a>
                                        </div>
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
