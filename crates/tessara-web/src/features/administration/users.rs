//! User-management administration pages and related helpers.

use super::api::{
    load_admin_user_access, load_admin_user_edit_context, load_admin_users,
    submit_update_admin_user, submit_update_admin_user_access,
};
#[cfg(feature = "hydrate")]
use crate::api::redirect_to_login;
use crate::features::administration::models::*;
use crate::features::organization::AdminRoleSummary;
#[cfg(feature = "hydrate")]
use crate::features::shared::navigate_to_href;
use crate::features::shared::{FilterHeader, status_badge_class, unique_filter_options};
use crate::types::AccountRouteParams;
use crate::types::route_params::require_route_params;
use crate::ui::{
    AppShell, Breadcrumb, BreadcrumbItem, BreadcrumbLink, BreadcrumbPage, BreadcrumbSeparator,
    DataTable, DropdownMenu, InfoListTable, PageHeader,
};
use crate::utils::pagination::pagination_page_start;
use crate::utils::text::text_matches;

use icons::{PanelRight, Pencil, Search};
use leptos::prelude::*;

#[component]
/// Renders the administration users page view.
pub fn AdministrationUsersPage() -> impl IntoView {
    let users = RwSignal::new(Vec::<AdminUserSummary>::new());
    let is_loading = RwSignal::new(true);
    let load_error = RwSignal::new(None::<String>);
    let search = RwSignal::new(String::new());
    let status_filter = RwSignal::new("all".to_string());
    let role_filter = RwSignal::new("all".to_string());

    Effect::new(move |_| {
        load_admin_users(users, is_loading, load_error);
    });

    let filtered_users = move || {
        let query = search.get();
        let status = status_filter.get();
        let role = role_filter.get();
        users
            .get()
            .into_iter()
            .filter(|user| {
                let status_key = admin_user_status_key(user);
                let role_names = admin_user_role_names(user);
                let matches_status = status == "all" || status == status_key;
                let matches_role =
                    role == "all" || user.roles.iter().any(|user_role| user_role.name == role);
                matches_status
                    && matches_role
                    && text_matches(
                        &query,
                        &[
                            user.display_name.as_str(),
                            user.email.as_str(),
                            status_key,
                            role_names.as_str(),
                        ],
                    )
            })
            .collect::<Vec<_>>()
    };
    let role_options = move || {
        unique_filter_options(users.get().iter().flat_map(|user| {
            user.roles
                .iter()
                .map(|role| role.name.clone())
                .collect::<Vec<_>>()
        }))
    };

    view! {
        <AppShell active_route="administration" title="Users">
            <section class="route-panel administration-users-page">
                <Breadcrumb>
                    <BreadcrumbItem>
                        <BreadcrumbLink href="/administration">"Administration"</BreadcrumbLink>
                    </BreadcrumbItem>
                    <BreadcrumbSeparator/>
                    <BreadcrumbItem>
                        <BreadcrumbPage>"Users"</BreadcrumbPage>
                    </BreadcrumbItem>
                </Breadcrumb>

                <PageHeader
                    title="Users"
                    description="Manage local Tessara users, active status, and assigned roles."
                />

                {move || {
                    if is_loading.get() {
                        view! {
                            <section class="organization-state" aria-live="polite">
                                <h3>"Loading users"</h3>
                                <p>"Fetching administrative user records."</p>
                            </section>
                        }
                        .into_any()
                    } else if let Some(message) = load_error.get() {
                        view! {
                            <section class="organization-state is-error" role="alert">
                                <h3>"Users unavailable"</h3>
                                <p>{message}</p>
                            </section>
                        }
                        .into_any()
                    } else {
                        view! {
                            <AdministrationUsersList
                                users=filtered_users()
                                search
                                status_filter
                                role_filter
                                role_options=role_options()
                            />
                        }
                        .into_any()
                    }
                }}
            </section>
        </AppShell>
    }
}

#[component]
/// Renders the administration users list view.
fn AdministrationUsersList(
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
    let page_count = move || {
        if total_count == 0 {
            1
        } else {
            total_count.div_ceil(page_size.get()).max(1)
        }
    };
    let current_page = move || page_index.get().min(page_count() - 1);
    let page_start = move || {
        if total_count == 0 {
            0
        } else {
            current_page() * page_size.get()
        }
    };
    let page_end = move || (page_start() + page_size.get()).min(total_count);
    let page_summary = move || {
        if total_count == 0 {
            "No users to display".to_string()
        } else {
            format!(
                "Showing {}-{} of {} users",
                page_start() + 1,
                page_end(),
                total_count
            )
        }
    };

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
                                <FilterHeader
                                    label="Role"
                                    all_label="All Roles"
                                    filter=role_filter
                                    options=role_options
                                />
                            </th>
                            <th class="data-table__cell--center" scope="col">
                                <FilterHeader
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
                <div class="directory-table-pagination" aria-label="Administration users table pagination">
                    <p>{move || page_summary()}</p>
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
                            disabled=move || current_page() == 0
                            on:click=move |_| {
                                page_index.update(|page| *page = page.saturating_sub(1));
                            }
                        >
                            "Previous"
                        </button>
                        <span>{move || format!("Page {} of {}", current_page() + 1, page_count())}</span>
                        <button
                            class="button button--compact button--secondary"
                            type="button"
                            disabled=move || { current_page() + 1 >= page_count() }
                            on:click=move |_| {
                                let last_page = page_count().saturating_sub(1);
                                page_index.update(|page| *page = (*page + 1).min(last_page));
                            }
                        >
                            "Next"
                        </button>
                    </div>
                </div>
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

/// Handles the admin user status key behavior.
fn admin_user_status_key(user: &AdminUserSummary) -> &'static str {
    if user.is_active { "active" } else { "inactive" }
}

/// Handles the admin user status label behavior.
fn admin_user_status_label(user: &AdminUserSummary) -> &'static str {
    if user.is_active { "Active" } else { "Inactive" }
}

/// Handles the admin user role names behavior.
fn admin_user_role_names(user: &AdminUserSummary) -> String {
    if user.roles.is_empty() {
        "No roles".to_string()
    } else {
        user.roles
            .iter()
            .map(|role| role.name.as_str())
            .collect::<Vec<_>>()
            .join(", ")
    }
}

/// Toggles the toggle string selection state.
pub(super) fn toggle_string_selection(
    selection: RwSignal<Vec<String>>,
    value: String,
    selected: bool,
) {
    selection.update(|values| {
        if selected {
            if !values.iter().any(|existing| existing == &value) {
                values.push(value);
            }
        } else {
            values.retain(|existing| existing != &value);
        }
    });
}

#[component]
/// Renders the administration user detail page view.
pub fn AdministrationUserDetailPage() -> impl IntoView {
    let params = require_route_params::<AccountRouteParams>();
    let account_id = params.account_id;
    let detail = RwSignal::new(None::<AdminUserAccessDetail>);
    let capability_catalog = RwSignal::new(Vec::<AdminCapabilitySummary>::new());
    let selected_scope_node_ids = RwSignal::new(Vec::<String>::new());
    let selected_delegate_account_ids = RwSignal::new(Vec::<String>::new());
    let is_loading = RwSignal::new(true);
    let is_saving = RwSignal::new(false);
    let load_error = RwSignal::new(None::<String>);
    let message = RwSignal::new(None::<String>);

    Effect::new({
        let account_id = account_id.clone();
        move |_| {
            load_admin_user_access(
                account_id.clone(),
                detail,
                selected_scope_node_ids,
                selected_delegate_account_ids,
                is_loading,
                load_error,
            );
        }
    });
    Effect::new(move |_| {
        load_admin_capability_catalog(capability_catalog);
    });

    view! {
        <AppShell active_route="administration" title="User Detail">
            <section class="route-panel administration-user-detail-page">
                <Breadcrumb>
                    <BreadcrumbItem>
                        <BreadcrumbLink href="/administration">"Administration"</BreadcrumbLink>
                    </BreadcrumbItem>
                    <BreadcrumbSeparator/>
                    <BreadcrumbItem>
                        <BreadcrumbLink href="/administration/users">"Users"</BreadcrumbLink>
                    </BreadcrumbItem>
                    <BreadcrumbSeparator/>
                    <BreadcrumbItem>
                        <BreadcrumbPage>"User Detail"</BreadcrumbPage>
                    </BreadcrumbItem>
                </Breadcrumb>

                {move || {
                    if is_loading.get() {
                        view! {
                            <section class="organization-state" aria-live="polite">
                                <h3>"Loading user"</h3>
                                <p>"Fetching account permissions, scope nodes, and delegations."</p>
                            </section>
                        }
                        .into_any()
                    } else if let Some(message) = load_error.get() {
                        view! {
                            <section class="organization-state is-error" role="alert">
                                <h3>"User unavailable"</h3>
                                <p>{message}</p>
                            </section>
                        }
                        .into_any()
                    } else if let Some(access) = detail.get() {
                        let edit_href = format!("/administration/users/{}/edit", access.account_id);
                        let capability_count = access.capabilities.len().to_string();
                        let scope_editing = admin_editable_label(access.scope_assignments_editable);
                        let delegation_editing =
                            admin_editable_label(access.delegation_assignments_editable);
                        view! {
                            <header class="page-header">
                                <div>
                                    <h2>{access.display_name.clone()}</h2>
                                    <p>{access.email.clone()}</p>
                                </div>
                            </header>

                            <form
                                class="native-form administration-user-access-form"
                                on:submit={
                                    let account_id = account_id.clone();
                                    move |event| {
                                        event.prevent_default();
                                        submit_update_admin_user_access(
                                            account_id.clone(),
                                            selected_scope_node_ids,
                                            selected_delegate_account_ids,
                                            is_saving,
                                            message,
                                        );
                                    }
                                }
                            >
                            <div class="organization-detail-content">
                                <section class="organization-detail-card organization-detail-card--wide">
                                    <h3>"Effective Access"</h3>
                                    <InfoListTable>
                                        <tr>
                                            <th scope="row">"Capabilities"</th>
                                            <td>{capability_count}</td>
                                        </tr>
                                        <tr>
                                            <th scope="row">"Scope Editing"</th>
                                            <td>{scope_editing}</td>
                                        </tr>
                                        <tr>
                                            <th scope="row">"Delegation Editing"</th>
                                            <td>{delegation_editing}</td>
                                        </tr>
                                    </InfoListTable>
                                </section>

                                <section class="organization-detail-card">
                                    <h3>"Scope Nodes"</h3>
                                    {if access.scope_assignments_editable {
                                        view! {
                                            <AdminScopeNodeChecklist
                                                nodes=access.available_scope_nodes
                                                selected_node_ids=selected_scope_node_ids
                                            />
                                        }
                                        .into_any()
                                    } else {
                                        view! { <AdminScopeNodeList nodes=access.scope_nodes/> }.into_any()
                                    }}
                                </section>

                                <section class="organization-detail-card">
                                    <h3>"Delegations"</h3>
                                    {if access.delegation_assignments_editable {
                                        view! {
                                            <AdminDelegationChecklist
                                                delegations=access.available_delegate_accounts
                                                selected_delegate_account_ids=selected_delegate_account_ids
                                            />
                                        }
                                        .into_any()
                                    } else {
                                        view! { <AdminDelegationList delegations=access.delegations empty_label="No delegated accounts."/> }.into_any()
                                    }}
                                </section>

                                <section class="organization-detail-card organization-detail-card--wide">
                                    <h3>"Capabilities"</h3>
                                    <AdminCapabilityList
                                        capabilities=access.capabilities
                                        capability_catalog=capability_catalog.get()
                                    />
                                    <div class="form-actions">
                                        <a class="button button--secondary" href=edit_href>"Edit Account Roles"</a>
                                    </div>
                                </section>
                            </div>
                            {move || message
                                .get()
                                .map(|text| view! { <p class="form-message" role="status">{text}</p> })}
                            <div class="form-actions">
                                <a class="button button--secondary" href="/administration/users">"Back to Users"</a>
                                <button class="button" type="submit" disabled=move || is_saving.get()>
                                    {move || if is_saving.get() { "Saving..." } else { "Save Permissions" }}
                                </button>
                            </div>
                            </form>
                        }
                        .into_any()
                    } else {
                        view! {
                            <section class="organization-state">
                                <h3>"User not found"</h3>
                                <p>"No user record was returned for this account."</p>
                            </section>
                        }
                        .into_any()
                    }
                }}
            </section>
        </AppShell>
    }
}

#[component]
/// Renders the administration user edit page view.
pub fn AdministrationUserEditPage() -> impl IntoView {
    let params = require_route_params::<AccountRouteParams>();
    let account_id = params.account_id;
    let detail = RwSignal::new(None::<AdminUserDetail>);
    let roles = RwSignal::new(Vec::<AdminRoleSummary>::new());
    let email = RwSignal::new(String::new());
    let display_name = RwSignal::new(String::new());
    let password = RwSignal::new(String::new());
    let is_active = RwSignal::new(true);
    let selected_role_ids = RwSignal::new(Vec::<String>::new());
    let is_loading = RwSignal::new(true);
    let is_saving = RwSignal::new(false);
    let load_error = RwSignal::new(None::<String>);
    let message = RwSignal::new(None::<String>);

    Effect::new({
        let account_id = account_id.clone();
        move |_| {
            load_admin_user_edit_context(
                account_id.clone(),
                detail,
                roles,
                email,
                display_name,
                is_active,
                selected_role_ids,
                is_loading,
                load_error,
            );
        }
    });

    view! {
        <AppShell active_route="administration" title="Edit User">
            <section class="route-panel administration-user-edit-page">
                <Breadcrumb>
                    <BreadcrumbItem>
                        <BreadcrumbLink href="/administration">"Administration"</BreadcrumbLink>
                    </BreadcrumbItem>
                    <BreadcrumbSeparator/>
                    <BreadcrumbItem>
                        <BreadcrumbLink href="/administration/users">"Users"</BreadcrumbLink>
                    </BreadcrumbItem>
                    <BreadcrumbSeparator/>
                    <BreadcrumbItem>
                        <BreadcrumbPage>"Edit User"</BreadcrumbPage>
                    </BreadcrumbItem>
                </Breadcrumb>

                {move || {
                    if is_loading.get() {
                        view! {
                            <section class="organization-state" aria-live="polite">
                                <h3>"Loading user"</h3>
                                <p>"Fetching account and role options."</p>
                            </section>
                        }
                        .into_any()
                    } else if let Some(error) = load_error.get() {
                        view! {
                            <section class="organization-state is-error" role="alert">
                                <h3>"User unavailable"</h3>
                                <p>{error}</p>
                            </section>
                        }
                        .into_any()
                    } else {
                        let cancel_href = format!("/administration/users/{account_id}");
                        view! {
                            <PageHeader
                                title="Edit User"
                                description="Update the account details, active status, password, and assigned roles."
                            />
                            <form
                                class="native-form administration-user-form"
                                on:submit={
                                    let account_id = account_id.clone();
                                    move |event| {
                                        event.prevent_default();
                                        submit_update_admin_user(
                                            account_id.clone(),
                                            email,
                                            display_name,
                                            password,
                                            is_active,
                                            selected_role_ids,
                                            is_saving,
                                            message,
                                        );
                                    }
                                }
                            >
                                <div class="form-grid">
                                    <label class="form-field" for="admin-user-display-name">
                                        <span>"Display Name"</span>
                                        <input
                                            id="admin-user-display-name"
                                            type="text"
                                            autocomplete="name"
                                            prop:value=move || display_name.get()
                                            on:input=move |event| display_name.set(event_target_value(&event))
                                            required
                                        />
                                    </label>
                                    <label class="form-field" for="admin-user-email">
                                        <span>"Email"</span>
                                        <input
                                            id="admin-user-email"
                                            type="email"
                                            autocomplete="email"
                                            prop:value=move || email.get()
                                            on:input=move |event| email.set(event_target_value(&event))
                                            required
                                        />
                                    </label>
                                    <label class="form-field" for="admin-user-password">
                                        <span>"New Password"</span>
                                        <input
                                            id="admin-user-password"
                                            type="password"
                                            autocomplete="new-password"
                                            placeholder="Leave blank to keep current password"
                                            prop:value=move || password.get()
                                            on:input=move |event| password.set(event_target_value(&event))
                                        />
                                    </label>
                                    <label class="form-field">
                                        <span>"Active"</span>
                                        <label class="toggle-row toggle-row--compact">
                                            <input
                                                type="checkbox"
                                                prop:checked=move || is_active.get()
                                                on:change=move |event| is_active.set(event_target_checked(&event))
                                            />
                                            <span>{move || if is_active.get() { "Active" } else { "Inactive" }}</span>
                                        </label>
                                    </label>
                                </div>

                                <section class="form-section">
                                    <h3>"Roles"</h3>
                                    <div class="checkbox-list">
                                        {move || {
                                            let selected = selected_role_ids.get();
                                            roles
                                                .get()
                                                .into_iter()
                                                .map(|role| {
                                                    let role_id = role.id.clone();
                                                    let checked = selected.iter().any(|id| id == &role.id);
                                                    view! {
                                                        <label class="checkbox-list__item">
                                                            <input
                                                                type="checkbox"
                                                                prop:checked=checked
                                                                on:change=move |event| {
                                                                    toggle_string_selection(
                                                                        selected_role_ids,
                                                                        role_id.clone(),
                                                                        event_target_checked(&event),
                                                                    );
                                                                }
                                                            />
                                                            <span>
                                                                <strong>{role.name}</strong>
                                                                <small>{format!("{} capabilities, {} users", role.capability_count, role.account_count)}</small>
                                                            </span>
                                                        </label>
                                                    }
                                                })
                                                .collect_view()
                                        }}
                                    </div>
                                </section>

                                {move || message
                                    .get()
                                    .map(|text| view! { <p class="form-message" role="status">{text}</p> })}

                                <div class="form-actions">
                                    <a class="button button--secondary" href=cancel_href.clone()>"Cancel"</a>
                                    <button class="button" type="submit" disabled=move || is_saving.get()>
                                        {move || if is_saving.get() { "Saving..." } else { "Save User" }}
                                    </button>
                                </div>
                            </form>
                        }
                        .into_any()
                    }
                }}
            </section>
        </AppShell>
    }
}

#[component]
/// Renders the administration user access page view.
pub fn AdministrationUserAccessPage() -> impl IntoView {
    AdministrationUserDetailPage()
}

#[component]
/// Renders the admin scope node list view.
fn AdminScopeNodeList(nodes: Vec<AdminScopeNodeSummary>) -> impl IntoView {
    if nodes.is_empty() {
        view! { <p>"No scope nodes assigned."</p> }.into_any()
    } else {
        view! {
            <table class="info-list-table">
                <tbody>
                    {nodes
                        .into_iter()
                        .map(|node| {
                            let node_context = admin_scope_node_context(&node);
                            view! {
                                <tr>
                                    <th scope="row">
                                        <a class="data-table__primary-link" href=format!("/organization/{}", node.node_id)>
                                            {node.node_name}
                                        </a>
                                    </th>
                                    <td>{node_context}</td>
                                </tr>
                            }
                        })
                        .collect_view()}
                </tbody>
            </table>
        }
        .into_any()
    }
}

#[component]
/// Renders the admin scope node checklist view.
fn AdminScopeNodeChecklist(
    nodes: Vec<AdminScopeNodeSummary>,
    selected_node_ids: RwSignal<Vec<String>>,
) -> impl IntoView {
    let search = RwSignal::new(String::new());
    let node_count = nodes.len();
    if nodes.is_empty() {
        view! { <p>"No scope nodes are available."</p> }.into_any()
    } else {
        view! {
            <div class="permission-picker">
                <label class="searchable-data-table__search searchable-data-table__control">
                    <Search class="searchable-data-table__control-icon"/>
                    <span class="sr-only">"Search scope nodes"</span>
                    <input
                        type="search"
                        placeholder=format!("Search {node_count} scope nodes")
                        prop:value=move || search.get()
                        on:input=move |event| search.set(event_target_value(&event))
                    />
                </label>
                <table class="info-list-table permission-picker__table">
                    <tbody>
                    {move || {
                        let query = search.get();
                        nodes
                            .iter()
                            .filter(|node| {
                                text_matches(
                                    &query,
                                    &[
                                        node.node_name.as_str(),
                                        node.node_type_name.as_str(),
                                        node.parent_node_name.as_deref().unwrap_or(""),
                                    ],
                                )
                            })
                            .cloned()
                            .map(|node| {
                                let node_id = node.node_id.clone();
                                let node_context = admin_scope_node_context(&node);
                                let selected = selected_node_ids
                                    .get()
                                    .iter()
                                    .any(|selected_id| selected_id == &node.node_id);
                                let checkbox_label =
                                    format!("Assign scope node {} ({node_context})", node.node_name);
                                view! {
                                    <tr>
                                        <td class="data-table__cell--center">
                                            <input
                                                type="checkbox"
                                                aria-label=checkbox_label
                                                prop:checked=selected
                                                on:change=move |event| {
                                                    toggle_string_selection(
                                                        selected_node_ids,
                                                        node_id.clone(),
                                                        event_target_checked(&event),
                                                    );
                                                }
                                            />
                                        </td>
                                        <th scope="row">{node.node_name}</th>
                                        <td>{node_context}</td>
                                    </tr>
                                }
                            })
                            .collect_view()
                    }}
                    </tbody>
                </table>
            </div>
        }
        .into_any()
    }
}

/// Handles the admin scope node context behavior.
fn admin_scope_node_context(node: &AdminScopeNodeSummary) -> String {
    match node.parent_node_name.as_deref() {
        Some(parent) if !parent.is_empty() => {
            format!("{} - Parent: {parent}", node.node_type_name)
        }
        _ => format!("{} - No parent", node.node_type_name),
    }
}

#[component]
/// Renders the admin delegation list view.
fn AdminDelegationList(
    delegations: Vec<AdminDelegationSummary>,
    empty_label: &'static str,
) -> impl IntoView {
    if delegations.is_empty() {
        view! { <p>{empty_label}</p> }.into_any()
    } else {
        view! {
            <table class="info-list-table">
                <tbody>
                    {delegations
                        .into_iter()
                        .map(|delegation| {
                            view! {
                                <tr>
                                    <th scope="row">
                                        <a class="data-table__primary-link" href=format!("/administration/users/{}", delegation.account_id)>
                                            {delegation.display_name}
                                        </a>
                                    </th>
                                    <td>{delegation.email}</td>
                                </tr>
                            }
                        })
                        .collect_view()}
                </tbody>
            </table>
        }
        .into_any()
    }
}

#[component]
/// Renders the admin delegation checklist view.
fn AdminDelegationChecklist(
    delegations: Vec<AdminDelegationSummary>,
    selected_delegate_account_ids: RwSignal<Vec<String>>,
) -> impl IntoView {
    let search = RwSignal::new(String::new());
    let delegation_count = delegations.len();
    if delegations.is_empty() {
        view! { <p>"No delegate accounts are available."</p> }.into_any()
    } else {
        view! {
            <div class="permission-picker">
                <label class="searchable-data-table__search searchable-data-table__control">
                    <Search class="searchable-data-table__control-icon"/>
                    <span class="sr-only">"Search delegate accounts"</span>
                    <input
                        type="search"
                        placeholder=format!("Search {delegation_count} accounts")
                        prop:value=move || search.get()
                        on:input=move |event| search.set(event_target_value(&event))
                    />
                </label>
                <table class="info-list-table permission-picker__table">
                    <tbody>
                    {move || {
                        let query = search.get();
                        delegations
                            .iter()
                            .filter(|delegation| {
                                text_matches(
                                    &query,
                                    &[delegation.display_name.as_str(), delegation.email.as_str()],
                                )
                            })
                            .cloned()
                            .map(|delegation| {
                                let account_id = delegation.account_id.clone();
                                let selected = selected_delegate_account_ids
                                    .get()
                                    .iter()
                                    .any(|selected_id| selected_id == &delegation.account_id);
                                let checkbox_label = format!(
                                    "Delegate access to {} ({})",
                                    delegation.display_name,
                                    delegation.email
                                );
                                view! {
                                    <tr>
                                        <td class="data-table__cell--center">
                                            <input
                                                type="checkbox"
                                                aria-label=checkbox_label
                                                prop:checked=selected
                                                on:change=move |event| {
                                                    toggle_string_selection(
                                                        selected_delegate_account_ids,
                                                        account_id.clone(),
                                                        event_target_checked(&event),
                                                    );
                                                }
                                            />
                                        </td>
                                        <th scope="row">{delegation.display_name}</th>
                                        <td>{delegation.email}</td>
                                    </tr>
                                }
                            })
                            .collect_view()
                    }}
                    </tbody>
                </table>
            </div>
        }
        .into_any()
    }
}

#[component]
/// Renders the admin capability list view.
fn AdminCapabilityList(
    capabilities: Vec<String>,
    capability_catalog: Vec<AdminCapabilitySummary>,
) -> impl IntoView {
    if capabilities.is_empty() {
        view! { <p>"No effective capabilities."</p> }.into_any()
    } else {
        view! {
            <table class="info-list-table">
                <tbody>
                {capabilities
                    .into_iter()
                    .map(|capability| {
                        let description = capability_catalog
                            .iter()
                            .find(|summary| summary.key == capability)
                            .map(|summary| summary.description.clone())
                            .unwrap_or_else(|| "Granted".to_string());
                        view! {
                        <tr>
                            <th scope="row">{capability}</th>
                            <td>{description}</td>
                        </tr>
                        }
                    })
                    .collect_view()}
                </tbody>
            </table>
        }
        .into_any()
    }
}

/// Loads the load admin capability catalog data.
fn load_admin_capability_catalog(capabilities: RwSignal<Vec<AdminCapabilitySummary>>) {
    #[cfg(feature = "hydrate")]
    {
        leptos::task::spawn_local(async move {
            match gloo_net::http::Request::get("/api/admin/capabilities")
                .send()
                .await
            {
                Ok(response) if response.status() == 401 => {
                    redirect_to_login();
                }
                Ok(response) if response.ok() => {
                    if let Ok(items) = response.json::<Vec<AdminCapabilitySummary>>().await {
                        capabilities.set(items);
                    }
                }
                _ => {}
            }
        });
    }

    #[cfg(not(feature = "hydrate"))]
    {
        let _ = capabilities;
    }
}

/// Handles the admin editable label behavior.
fn admin_editable_label(is_editable: bool) -> &'static str {
    if is_editable {
        "Editable"
    } else {
        "Not editable"
    }
}
