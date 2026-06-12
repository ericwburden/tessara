//! Role-management administration route surface.

use super::state::{AdministrationRolesPageState, filtered_admin_roles};
use crate::features::administration::api::{
    load_admin_role_detail, load_admin_roles_context, save_admin_role,
};
use crate::features::administration::components::{
    AdminRoleSheet, AdministrationRoleDetailPanel, AdministrationRolesList,
};
use crate::ui::{
    AppShell, Breadcrumb, BreadcrumbItem, BreadcrumbLink, BreadcrumbPage, BreadcrumbSeparator,
    PageHeader,
};
use leptos::prelude::*;

#[component]
pub(super) fn AdministrationRolesSurface() -> impl IntoView {
    let state = AdministrationRolesPageState::new();
    let AdministrationRolesPageState {
        roles,
        capabilities,
        selected_role_id,
        selected_role_detail,
        search,
        is_loading,
        detail_loading,
        is_saving,
        message,
        sheet_open,
        editing_role_id,
        role_name,
        selected_capability_ids,
        capability_search,
    } = state;

    load_admin_roles_context(
        roles,
        capabilities,
        selected_role_id,
        is_loading,
        message,
        None,
    );

    Effect::new(move |_| {
        if let Some(role_id) = selected_role_id.get() {
            load_admin_role_detail(role_id, selected_role_detail, detail_loading, message);
        } else {
            selected_role_detail.set(None);
        }
    });

    let open_create_sheet = move |_| {
        editing_role_id.set(None);
        role_name.set(String::new());
        selected_capability_ids.set(Vec::new());
        capability_search.set(String::new());
        message.set(None);
        sheet_open.set(true);
    };
    let open_edit_sheet = move |_| {
        if let Some(detail) = selected_role_detail.get() {
            editing_role_id.set(Some(detail.id));
            role_name.set(detail.name);
            selected_capability_ids.set(
                detail
                    .capabilities
                    .into_iter()
                    .map(|capability| capability.id)
                    .collect(),
            );
            capability_search.set(String::new());
            message.set(None);
            sheet_open.set(true);
        }
    };
    let close_sheet = move |_| {
        sheet_open.set(false);
        editing_role_id.set(None);
        role_name.set(String::new());
        selected_capability_ids.set(Vec::new());
        capability_search.set(String::new());
    };
    let save_role = move |_| {
        save_admin_role(
            editing_role_id,
            role_name,
            selected_capability_ids,
            is_saving,
            message,
            sheet_open,
            roles,
            capabilities,
            selected_role_id,
            selected_role_detail,
            detail_loading,
        );
    };

    view! {
        <AppShell active_route="administration" title="Roles">
            <section class="route-panel administration-roles-page">
                <Breadcrumb>
                    <BreadcrumbItem>
                        <BreadcrumbLink href="/administration">"Administration"</BreadcrumbLink>
                    </BreadcrumbItem>
                    <BreadcrumbSeparator/>
                    <BreadcrumbItem>
                        <BreadcrumbPage>"Roles"</BreadcrumbPage>
                    </BreadcrumbItem>
                </Breadcrumb>

                <PageHeader
                    title="Roles"
                    description="Manage reusable capability templates for Tessara users."
                >
                    <button class="button" type="button" on:click=open_create_sheet>
                        "New Role"
                    </button>
                </PageHeader>

                {move || {
                    if is_loading.get() {
                        view! {
                            <section class="organization-state" aria-live="polite">
                                <h3>"Loading roles"</h3>
                                <p>"Fetching role and capability configuration."</p>
                            </section>
                        }
                        .into_any()
                    } else if let Some(error) = message.get().filter(|_| roles.get().is_empty()) {
                        view! {
                            <section class="organization-state is-error" role="alert">
                                <h3>"Roles unavailable"</h3>
                                <p>{error}</p>
                            </section>
                        }
                        .into_any()
                    } else {
                        view! {
                            <div class="administration-roles-stack">
                                <AdministrationRolesList
                                    roles=filtered_admin_roles(roles, search)
                                    search
                                    selected_role_id
                                />
                                <AdministrationRoleDetailPanel
                                    detail=selected_role_detail.get()
                                    is_loading=detail_loading.get()
                                    on_edit=open_edit_sheet
                                />
                            </div>
                        }
                        .into_any()
                    }
                }}

                <AdminRoleSheet
                    is_open=sheet_open
                    editing_role_id
                    role_name
                    capabilities
                    selected_capability_ids
                    capability_search
                    is_saving
                    message
                    on_close=close_sheet
                    on_save=save_role
                />
            </section>
        </AppShell>
    }
}
