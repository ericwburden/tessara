//! Administration users list page.

use super::super::super::api::load_admin_users;
use super::super::super::components::AdministrationUsersList;
use super::super::super::state::{admin_user_role_filter_options, filtered_admin_users};
use crate::features::administration::models::AdminUserSummary;
use crate::ui::{
    AppShell, Breadcrumb, BreadcrumbItem, BreadcrumbLink, BreadcrumbPage, BreadcrumbSeparator,
    PageHeader,
};
use leptos::prelude::*;

#[component]
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
