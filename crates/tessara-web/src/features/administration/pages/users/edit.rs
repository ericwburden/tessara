//! Administration user edit page.

use super::super::super::api::load_admin_user_edit_context;
use super::super::super::components::AdministrationUserAccountForm;
use super::super::super::state::AdminUserEditState;
use crate::types::AccountRouteParams;
use crate::types::route_params::require_route_params;
use crate::ui::{
    AppShell, Breadcrumb, BreadcrumbItem, BreadcrumbLink, BreadcrumbPage, BreadcrumbSeparator,
};
use leptos::prelude::*;

#[component]
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
