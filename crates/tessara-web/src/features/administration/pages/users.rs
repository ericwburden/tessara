//! User-management administration pages and helpers.
//!
//! Keep account list, access editing, and user administration workflows here.

use super::super::api::{
    load_admin_capability_catalog, load_admin_user_access, load_admin_user_edit_context,
    load_admin_users, submit_update_admin_user, submit_update_admin_user_access,
};
use super::super::components::{
    AdminCapabilityList, AdminDelegationChecklist, AdminDelegationList, AdminScopeNodeChecklist,
    AdminScopeNodeList, AdministrationUsersList,
};
use super::super::display::{admin_editable_label, admin_user_role_names, admin_user_status_key};
use super::super::state::toggle_string_selection;
use crate::features::administration::models::*;
use crate::features::organization::AdminRoleSummary;
use crate::features::shared::unique_filter_options;
use crate::types::AccountRouteParams;
use crate::types::route_params::require_route_params;
use crate::ui::{
    AppShell, Breadcrumb, BreadcrumbItem, BreadcrumbLink, BreadcrumbPage, BreadcrumbSeparator,
    InfoListTable, PageHeader,
};
use crate::utils::text::text_matches;

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
