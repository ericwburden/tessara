use leptos::prelude::*;
use leptos_meta::provide_meta_context;
use leptos_router::components::{Route, Router, Routes};
use leptos_router::{Lazy, SsrMode, path};
#[cfg(feature = "hydrate")]
use wasm_bindgen::{JsCast, prelude::wasm_bindgen};

pub(crate) mod transitional;

use crate::features::{
    administration, dashboards, forms, home, migration, organization, reporting, responses,
};

const PRIMARY_SSR_MODE: SsrMode = SsrMode::InOrder;

#[cfg(feature = "hydrate")]
#[wasm_bindgen]
pub fn hydrate_app(root_id: &str) {
    use leptos::mount::hydrate_from_async;
    use wasm_bindgen_futures::spawn_local;
    use web_sys::window;

    let Some(document) = window().and_then(|window| window.document()) else {
        return;
    };
    let Some(root) = document.get_element_by_id(root_id) else {
        return;
    };
    let Ok(root) = root.dyn_into::<web_sys::HtmlElement>() else {
        return;
    };

    spawn_local(async move {
        let handle = hydrate_from_async(root, App).await;
        handle.forget();
    });
}

#[component]
pub fn App() -> impl IntoView {
    provide_meta_context();
    let migration_route = Lazy::<migration::MigrationLazyRoute>::new();

    // Route inventory note:
    // `/app/login` and `/app/administration/*` are the first auth/admin
    // surfaces being cleaned up as explicit application routes on top of the
    // retained bridge. The remaining product routes still depend on the
    // transitional bridge controller for body-level behavior.
    view! {
        <Router>
            <Routes fallback=|| view! { <p>"Not found."</p> }>
                <Route path=path!("/") view=home::AdminWorkbenchPage ssr=PRIMARY_SSR_MODE />
                <Route path=path!("/app") view=home::HomePage ssr=PRIMARY_SSR_MODE />
                <Route path=path!("/app/login") view=home::LoginPage ssr=PRIMARY_SSR_MODE />
                <Route path=path!("/app/organization") view=organization::OrganizationListPage ssr=PRIMARY_SSR_MODE />
                <Route path=path!("/app/organization/new") view=organization::OrganizationCreatePage ssr=PRIMARY_SSR_MODE />
                <Route path=path!("/app/organization/:node_id") view=organization::OrganizationDetailPage ssr=PRIMARY_SSR_MODE />
                <Route path=path!("/app/organization/:node_id/edit") view=organization::OrganizationEditPage ssr=PRIMARY_SSR_MODE />
                <Route path=path!("/app/forms") view=forms::FormsListPage ssr=PRIMARY_SSR_MODE />
                <Route path=path!("/app/forms/new") view=forms::FormCreatePage ssr=PRIMARY_SSR_MODE />
                <Route path=path!("/app/forms/:form_id") view=forms::FormDetailPage ssr=PRIMARY_SSR_MODE />
                <Route path=path!("/app/forms/:form_id/edit") view=forms::FormEditPage ssr=PRIMARY_SSR_MODE />
                <Route path=path!("/app/responses") view=responses::ResponsesListPage ssr=PRIMARY_SSR_MODE />
                <Route path=path!("/app/responses/new") view=responses::ResponseCreatePage ssr=PRIMARY_SSR_MODE />
                <Route path=path!("/app/responses/:submission_id") view=responses::ResponseDetailPage ssr=PRIMARY_SSR_MODE />
                <Route path=path!("/app/responses/:submission_id/edit") view=responses::ResponseEditPage ssr=PRIMARY_SSR_MODE />
                <Route path=path!("/app/submissions") view=responses::SubmissionsPage ssr=PRIMARY_SSR_MODE />
                <Route path=path!("/app/reports") view=reporting::ReportsPage ssr=PRIMARY_SSR_MODE />
                <Route path=path!("/app/reports/new") view=reporting::ReportCreatePage ssr=PRIMARY_SSR_MODE />
                <Route path=path!("/app/reports/:report_id") view=reporting::ReportDetailPage ssr=PRIMARY_SSR_MODE />
                <Route path=path!("/app/reports/:report_id/edit") view=reporting::ReportEditPage ssr=PRIMARY_SSR_MODE />
                <Route path=path!("/app/dashboards") view=dashboards::DashboardsPage ssr=PRIMARY_SSR_MODE />
                <Route path=path!("/app/dashboards/new") view=dashboards::DashboardCreatePage ssr=PRIMARY_SSR_MODE />
                <Route path=path!("/app/dashboards/:dashboard_id") view=dashboards::DashboardDetailPage ssr=PRIMARY_SSR_MODE />
                <Route path=path!("/app/dashboards/:dashboard_id/edit") view=dashboards::DashboardEditPage ssr=PRIMARY_SSR_MODE />
                <Route path=path!("/app/administration") view=administration::AdministrationPage ssr=PRIMARY_SSR_MODE />
                <Route path=path!("/app/administration/users") view=administration::UsersPage ssr=PRIMARY_SSR_MODE />
                <Route path=path!("/app/administration/users/new") view=administration::UserCreatePage ssr=PRIMARY_SSR_MODE />
                <Route path=path!("/app/administration/users/:account_id") view=administration::UserDetailPage ssr=PRIMARY_SSR_MODE />
                <Route path=path!("/app/administration/users/:account_id/edit") view=administration::UserEditPage ssr=PRIMARY_SSR_MODE />
                <Route path=path!("/app/administration/users/:account_id/access") view=administration::UserAccessPage ssr=PRIMARY_SSR_MODE />
                <Route path=path!("/app/administration/roles") view=administration::RolesPage ssr=PRIMARY_SSR_MODE />
                <Route path=path!("/app/administration/roles/new") view=administration::RoleCreatePage ssr=PRIMARY_SSR_MODE />
                <Route path=path!("/app/administration/roles/:role_id") view=administration::RoleDetailPage ssr=PRIMARY_SSR_MODE />
                <Route path=path!("/app/administration/roles/:role_id/edit") view=administration::RoleEditPage ssr=PRIMARY_SSR_MODE />
                <Route path=path!("/app/admin") view=administration::LegacyAdminPage ssr=PRIMARY_SSR_MODE />
                <Route path=path!("/app/migration") view=migration_route.clone() ssr=PRIMARY_SSR_MODE />
            </Routes>
        </Router>
    }
}
