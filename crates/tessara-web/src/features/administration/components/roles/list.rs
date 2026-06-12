//! Role-management Administration list component.

use crate::features::organization::AdminRoleSummary;
use crate::ui::{DataTable, DropdownMenu, TablePaginationFooter};
use crate::utils::pagination::pagination_page_start;
use icons::{PanelRight, Search};
use leptos::prelude::*;

#[component]
pub(crate) fn AdministrationRolesList(
    roles: Vec<AdminRoleSummary>,
    search: RwSignal<String>,
    selected_role_id: RwSignal<Option<String>>,
) -> impl IntoView {
    let table_roles = roles.clone();
    let card_roles = roles.clone();
    let page_size = RwSignal::new(10usize);
    let page_index = RwSignal::new(0usize);
    let total_count_value = roles.len();
    let total_count = Memo::new(move |_| total_count_value);

    view! {
        <div class="forms-list forms-list-responsive-table administration-roles-list">
            <div class="searchable-data-table">
                <div class="searchable-data-table__toolbar forms-list__toolbar">
                    <label class="searchable-data-table__search searchable-data-table__control">
                        <Search class="searchable-data-table__control-icon"/>
                        <span class="sr-only">"Search roles"</span>
                        <input
                            type="search"
                            placeholder="Search roles"
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
                            <th scope="col">"Role"</th>
                            <th class="data-table__cell--center" scope="col">"Capabilities"</th>
                            <th class="data-table__cell--center" scope="col">"Users"</th>
                            <th class="data-table__cell--center" scope="col">"Actions"</th>
                        </tr>
                    </thead>
                    <tbody>
                        {move || {
                            if table_roles.is_empty() {
                                view! {
                                    <tr>
                                        <td class="data-table__empty" colspan="4">"No Roles to Display"</td>
                                    </tr>
                                }
                                .into_any()
                            } else {
                                table_roles
                                    .iter()
                                    .skip(pagination_page_start(total_count.get(), page_size.get(), page_index.get()))
                                    .take(page_size.get())
                                    .cloned()
                                    .map(|role| {
                                        let role_id = role.id.clone();
                                        let role_name = role.name.clone();
                                        view! {
                                            <tr>
                                                <th scope="row">
                                                    <button
                                                        class="link-button data-table__primary-link"
                                                        type="button"
                                                        on:click=move |_| selected_role_id.set(Some(role_id.clone()))
                                                    >
                                                        {role.name}
                                                    </button>
                                                </th>
                                                <td class="data-table__cell--center">{role.capability_count}</td>
                                                <td class="data-table__cell--center">{role.account_count}</td>
                                                <td class="data-table__cell--center">
                                                    <DropdownMenu label=format!("Open actions for {role_name}")>
                                                        <button
                                                            class="dropdown-menu__item"
                                                            type="button"
                                                            role="menuitem"
                                                            on:click=move |_| selected_role_id.set(Some(role.id.clone()))
                                                        >
                                                            <PanelRight class="dropdown-menu__item-icon"/>
                                                            <span>"View Details"</span>
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
                    aria_label="Administration roles table pagination"
                    item_label="roles"
                    total_count=total_count
                    page_size=page_size
                    page_index=page_index
                />
            </div>
            <div class="forms-list-mobile-cards administration-roles-mobile-cards">
                {move || {
                    if card_roles.is_empty() {
                        view! { <p class="forms-list-mobile-empty">"No Roles to Display"</p> }.into_any()
                    } else {
                        card_roles
                            .iter()
                            .skip(pagination_page_start(total_count.get(), page_size.get(), page_index.get()))
                            .take(page_size.get())
                            .cloned()
                            .map(|role| {
                                let role_id = role.id.clone();
                                view! {
                                    <article class="forms-list-mobile-card administration-role-mobile-card">
                                        <div class="forms-list-mobile-card__header">
                                            <div>
                                                <h3>{role.name}</h3>
                                            </div>
                                        </div>
                                        <dl>
                                            <div>
                                                <dt>"Capabilities"</dt>
                                                <dd>{role.capability_count}</dd>
                                            </div>
                                            <div>
                                                <dt>"Users"</dt>
                                                <dd>{role.account_count}</dd>
                                            </div>
                                        </dl>
                                        <div class="workflow-assignment-mobile-card__actions">
                                            <button
                                                class="button button--compact"
                                                type="button"
                                                on:click=move |_| selected_role_id.set(Some(role_id.clone()))
                                            >
                                                "View Details"
                                            </button>
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
