use leptos::prelude::*;

use crate::app::transitional::{TransitionalPage, extract_app_root, render_transitional_route};
use crate::infra::routing::{AccountRouteParams, RoleRouteParams, require_route_params};

#[component]
pub fn AdministrationPage() -> impl IntoView {
    render_transitional_route(TransitionalPage {
        title: "Tessara Administration",
        description: "Tessara internal administration landing page.",
        body_html: extract_app_root(crate::administration_application_shell_html()),
        page_key: "administration",
        active_route: "administration",
        record_id: None,
    })
}

#[component]
pub fn UsersPage() -> impl IntoView {
    render_transitional_route(TransitionalPage {
        title: "Tessara User Management",
        description: "Browse and manage Tessara user accounts.",
        body_html: extract_app_root(crate::users_application_shell_html()),
        page_key: "user-list",
        active_route: "administration",
        record_id: None,
    })
}

#[component]
pub fn UserCreatePage() -> impl IntoView {
    render_transitional_route(TransitionalPage {
        title: "Create User",
        description: "Create a Tessara application account.",
        body_html: extract_app_root(crate::user_create_application_html()),
        page_key: "user-create",
        active_route: "administration",
        record_id: None,
    })
}

#[component]
pub fn UserDetailPage() -> impl IntoView {
    let AccountRouteParams { account_id } = require_route_params();

    render_transitional_route(TransitionalPage {
        title: "User Detail",
        description: "Inspect a Tessara application account.",
        body_html: extract_app_root(crate::user_detail_application_html(&account_id)),
        page_key: "user-detail",
        active_route: "administration",
        record_id: Some(account_id),
    })
}

#[component]
pub fn UserEditPage() -> impl IntoView {
    let AccountRouteParams { account_id } = require_route_params();

    render_transitional_route(TransitionalPage {
        title: "Edit User",
        description: "Edit a Tessara application account.",
        body_html: extract_app_root(crate::user_edit_application_html(&account_id)),
        page_key: "user-edit",
        active_route: "administration",
        record_id: Some(account_id),
    })
}

#[component]
pub fn UserAccessPage() -> impl IntoView {
    let AccountRouteParams { account_id } = require_route_params();

    render_transitional_route(TransitionalPage {
        title: "User Access",
        description: "Manage scoped access assignments for a Tessara application account.",
        body_html: extract_app_root(crate::user_access_application_html(&account_id)),
        page_key: "user-access",
        active_route: "administration",
        record_id: Some(account_id),
    })
}

#[component]
pub fn RolesPage() -> impl IntoView {
    render_transitional_route(TransitionalPage {
        title: "Tessara Roles",
        description: "Browse and inspect Tessara role bundles.",
        body_html: extract_app_root(crate::roles_application_shell_html()),
        page_key: "role-list",
        active_route: "administration",
        record_id: None,
    })
}

#[component]
pub fn RoleCreatePage() -> impl IntoView {
    render_transitional_route(TransitionalPage {
        title: "Create Role",
        description: "Create a Tessara role bundle.",
        body_html: extract_app_root(crate::role_create_application_html()),
        page_key: "role-create",
        active_route: "administration",
        record_id: None,
    })
}

#[component]
pub fn RoleDetailPage() -> impl IntoView {
    let RoleRouteParams { role_id } = require_route_params();

    render_transitional_route(TransitionalPage {
        title: "Role Detail",
        description: "Inspect a Tessara role bundle.",
        body_html: extract_app_root(crate::role_detail_application_html(&role_id)),
        page_key: "role-detail",
        active_route: "administration",
        record_id: Some(role_id),
    })
}

#[component]
pub fn RoleEditPage() -> impl IntoView {
    let RoleRouteParams { role_id } = require_route_params();

    render_transitional_route(TransitionalPage {
        title: "Edit Role",
        description: "Edit a Tessara role bundle.",
        body_html: extract_app_root(crate::role_edit_application_html(&role_id)),
        page_key: "role-edit",
        active_route: "administration",
        record_id: Some(role_id),
    })
}

#[component]
pub fn LegacyAdminPage() -> impl IntoView {
    render_transitional_route(TransitionalPage {
        title: "Tessara",
        description: "Tessara local admin workbench for migration setup and workflow testing.",
        body_html: extract_app_root(crate::admin_application_shell_html()),
        page_key: "admin-shell",
        active_route: "administration",
        record_id: None,
    })
}
