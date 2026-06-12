//! Role-management Administration components.

use crate::features::administration::models::{
    AdminAccountAssignmentSummary, AdminCapabilitySummary, AdminRoleDetail,
};
use crate::features::organization::AdminRoleSummary;
use crate::ui::{DataTable, DropdownMenu, TablePaginationFooter};
use crate::utils::pagination::pagination_page_start;
use icons::{PanelRight, Search};
use leptos::prelude::*;

#[component]
/// Renders the administration roles list view.
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

#[component]
/// Renders the administration role detail panel view.
pub(crate) fn AdministrationRoleDetailPanel(
    detail: Option<AdminRoleDetail>,
    is_loading: bool,
    on_edit: impl Fn(leptos::ev::MouseEvent) + 'static + Copy,
) -> impl IntoView {
    if is_loading {
        view! {
            <section class="organization-state" aria-live="polite">
                <h3>"Loading role"</h3>
                <p>"Fetching role details."</p>
            </section>
        }
        .into_any()
    } else if let Some(detail) = detail {
        let capabilities = detail.capabilities.clone();
        let accounts = detail.assigned_accounts.clone();
        view! {
            <section
                class="organization-detail-card organization-detail-card--wide administration-role-detail-card"
                style="margin-top: 1rem;"
            >
                <div class="organization-detail-card__header">
                    <h2>{detail.name}</h2>
                    <button class="button button--secondary" type="button" on:click=on_edit>
                        "Edit Capabilities"
                    </button>
                </div>
                <table class="info-list-table">
                    <tbody>
                        <tr>
                            <th scope="row">"Capabilities"</th>
                            <td>{detail.capabilities.len()}</td>
                        </tr>
                        <tr>
                            <th scope="row">"Assigned Users"</th>
                            <td>{detail.assigned_accounts.len()}</td>
                        </tr>
                    </tbody>
                </table>
                <div class="administration-role-detail-grid">
                    <section class="organization-detail-card">
                        <h3>"Capabilities"</h3>
                        <AdminRoleCapabilityList capabilities/>
                    </section>
                    <section class="organization-detail-card">
                        <h3>"Assigned Users"</h3>
                        <AdminRoleAssignedAccounts accounts/>
                    </section>
                </div>
            </section>
        }
        .into_any()
    } else {
        view! {
            <section class="organization-state">
                <h3>"Select a role"</h3>
                <p>"Choose a role to review its capabilities and assigned users."</p>
            </section>
        }
        .into_any()
    }
}

#[component]
/// Renders the admin role capability list view.
fn AdminRoleCapabilityList(capabilities: Vec<AdminCapabilitySummary>) -> impl IntoView {
    if capabilities.is_empty() {
        view! { <p class="muted">"No capabilities assigned."</p> }.into_any()
    } else {
        view! {
            <table class="info-list-table">
                <tbody>
                {capabilities
                    .into_iter()
                    .map(|capability| view! {
                        <tr>
                            <th scope="row">{capability.key}</th>
                            <td>{capability.description}</td>
                        </tr>
                    })
                    .collect_view()}
                </tbody>
            </table>
        }
        .into_any()
    }
}

#[component]
/// Renders the admin role assigned accounts view.
fn AdminRoleAssignedAccounts(accounts: Vec<AdminAccountAssignmentSummary>) -> impl IntoView {
    if accounts.is_empty() {
        view! { <p class="muted">"No users assigned."</p> }.into_any()
    } else {
        view! {
            <table class="info-list-table">
                <tbody>
                {accounts
                    .into_iter()
                    .map(|account| view! {
                        <tr>
                            <th scope="row">{account.display_name}</th>
                            <td>{account.email}</td>
                        </tr>
                    })
                    .collect_view()}
                </tbody>
            </table>
        }
        .into_any()
    }
}
