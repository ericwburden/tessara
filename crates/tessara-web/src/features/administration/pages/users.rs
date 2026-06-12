//! User-management administration pages and helpers.
//!
//! Keep account list, access editing, and user administration workflows here.

use super::super::api::{
    load_admin_capability_catalog, load_admin_user_access, load_admin_user_edit_context,
    load_admin_users,
};
use super::super::components::{
    AdministrationUserAccessForm, AdministrationUserAccountForm, AdministrationUsersList,
};
use super::super::state::{
    AdminUserAccessState, AdminUserEditState, admin_user_role_filter_options, filtered_admin_users,
};
use crate::features::administration::models::*;
use crate::types::AccountRouteParams;
use crate::types::route_params::require_route_params;
use crate::ui::{
    AppShell, Breadcrumb, BreadcrumbItem, BreadcrumbLink, BreadcrumbPage, BreadcrumbSeparator,
    PageHeader,
};

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
        filtered_admin_users(
            users.get(),
            &search.get(),
            &status_filter.get(),
            &role_filter.get(),
        )
    };
    let role_options = move || admin_user_role_filter_options(&users.get());

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
    let access_state = AdminUserAccessState::new();

    Effect::new({
        let account_id = account_id.clone();
        move |_| {
            load_admin_user_access(
                account_id.clone(),
                access_state.detail,
                access_state.selected_scope_node_ids,
                access_state.selected_delegate_account_ids,
                access_state.is_loading,
                access_state.load_error,
            );
        }
    });
    Effect::new(move |_| {
        load_admin_capability_catalog(access_state.capability_catalog);
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
                    if access_state.is_loading.get() {
                        view! {
                            <section class="organization-state" aria-live="polite">
                                <h3>"Loading user"</h3>
                                <p>"Fetching account permissions, scope nodes, and delegations."</p>
                            </section>
                        }
                        .into_any()
                    } else if let Some(message) = access_state.load_error.get() {
                        view! {
                            <section class="organization-state is-error" role="alert">
                                <h3>"User unavailable"</h3>
                                <p>{message}</p>
                            </section>
                        }
                        .into_any()
                    } else if let Some(access) = access_state.detail.get() {
                        view! {
                            <AdministrationUserAccessForm
                                account_id=account_id.clone()
                                access=access
                                capability_catalog=access_state.capability_catalog
                                selected_scope_node_ids=access_state.selected_scope_node_ids
                                selected_delegate_account_ids=access_state.selected_delegate_account_ids
                                is_saving=access_state.is_saving
                                message=access_state.message
                            />
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
    let edit_state = AdminUserEditState::new();

    Effect::new({
        let account_id = account_id.clone();
        move |_| {
            load_admin_user_edit_context(
                account_id.clone(),
                edit_state.detail,
                edit_state.roles,
                edit_state.email,
                edit_state.display_name,
                edit_state.is_active,
                edit_state.selected_role_ids,
                edit_state.is_loading,
                edit_state.load_error,
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
                    if edit_state.is_loading.get() {
                        view! {
                            <section class="organization-state" aria-live="polite">
                                <h3>"Loading user"</h3>
                                <p>"Fetching account and role options."</p>
                            </section>
                        }
                        .into_any()
                    } else if let Some(error) = edit_state.load_error.get() {
                        view! {
                            <section class="organization-state is-error" role="alert">
                                <h3>"User unavailable"</h3>
                                <p>{error}</p>
                            </section>
                        }
                        .into_any()
                    } else {
                        view! {
                            <AdministrationUserAccountForm
                                account_id=account_id.clone()
                                roles=edit_state.roles
                                email=edit_state.email
                                display_name=edit_state.display_name
                                password=edit_state.password
                                is_active=edit_state.is_active
                                selected_role_ids=edit_state.selected_role_ids
                                is_saving=edit_state.is_saving
                                message=edit_state.message
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
/// Renders the administration user access page view.
pub fn AdministrationUserAccessPage() -> impl IntoView {
    AdministrationUserDetailPage()
}
