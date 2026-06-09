use leptos::prelude::*;
use leptos_router::{path, MatchNestedRoutes, SsrMode};
use leptos_router::components::Route;

use crate::features::administration::{
    AdministrationNodeTypesPage, AdministrationPage, AdministrationRolesPage, AdministrationUserAccessPage,
    AdministrationUserDetailPage, AdministrationUserEditPage, AdministrationUsersPage,
};

const PRIMARY_SSR_MODE: SsrMode = SsrMode::InOrder;

pub fn administration_routes() -> impl MatchNestedRoutes + Clone {
    view! {
        <>
            <Route
                path=path!("/administration")
                view=AdministrationPage
                ssr=PRIMARY_SSR_MODE
            />
            <Route
                path=path!("/administration/users")
                view=AdministrationUsersPage
                ssr=PRIMARY_SSR_MODE
            />
            <Route
                path=path!("/administration/users/:account_id")
                view=AdministrationUserDetailPage
                ssr=PRIMARY_SSR_MODE
            />
            <Route
                path=path!("/administration/users/:account_id/edit")
                view=AdministrationUserEditPage
                ssr=PRIMARY_SSR_MODE
            />
            <Route
                path=path!("/administration/users/:account_id/access")
                view=AdministrationUserAccessPage
                ssr=PRIMARY_SSR_MODE
            />
            <Route
                path=path!("/administration/node-types")
                view=AdministrationNodeTypesPage
                ssr=PRIMARY_SSR_MODE
            />
            <Route
                path=path!("/administration/roles")
                view=AdministrationRolesPage
                ssr=PRIMARY_SSR_MODE
            />
        </>
    }
}
