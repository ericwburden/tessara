//! Administration user detail page.

use super::super::super::api::{load_admin_capability_catalog, load_admin_user_access};
use super::super::super::components::AdministrationUserAccessForm;
use super::super::super::state::AdminUserAccessState;
use crate::types::AccountRouteParams;
use crate::types::route_params::require_route_params;
use crate::ui::{
    AppShell, Breadcrumb, BreadcrumbItem, BreadcrumbLink, BreadcrumbPage, BreadcrumbSeparator,
};
use leptos::prelude::*;

#[component]
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
pub fn AdministrationUserAccessPage() -> impl IntoView {
    AdministrationUserDetailPage()
}
