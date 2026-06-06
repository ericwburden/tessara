use leptos::prelude::*;
use leptos_router::components::{Route, Router, Routes};
use leptos_router::{SsrMode, path};
#[cfg(feature = "hydrate")]
use wasm_bindgen::{JsCast, prelude::wasm_bindgen};

use crate::features::native;

const PRIMARY_SSR_MODE: SsrMode = SsrMode::InOrder;

#[cfg(feature = "hydrate")]
#[wasm_bindgen]
pub fn hydrate_app(root_id: &str) {
    use leptos::mount::mount_to;
    use web_sys::window;

    let _ = any_spawner::Executor::init_wasm_bindgen();

    let Some(document) = window().and_then(|window| window.document()) else {
        return;
    };
    let Some(root) = document.get_element_by_id(root_id) else {
        return;
    };
    let Ok(root) = root.dyn_into::<web_sys::HtmlElement>() else {
        return;
    };

    root.set_inner_html("");
    let handle = mount_to(root, App);
    handle.forget();
}

#[component]
pub fn App() -> impl IntoView {
    view! {
        <Router>
            <Routes fallback=|| view! { <native::NotFoundPage/> }>
                <Route path=path!("/") view=native::HomePage ssr=PRIMARY_SSR_MODE/>
                <Route path=path!("/login") view=native::LoginPage ssr=PRIMARY_SSR_MODE/>
                <Route path=path!("/organization") view=native::OrganizationPage ssr=PRIMARY_SSR_MODE/>
                <Route path=path!("/organization/new") view=native::OrganizationNewPage ssr=PRIMARY_SSR_MODE/>
                <Route path=path!("/organization/:node_id") view=native::OrganizationDetailPage ssr=PRIMARY_SSR_MODE/>
                <Route path=path!("/organization/:node_id/edit") view=native::OrganizationEditPage ssr=PRIMARY_SSR_MODE/>
                <Route path=path!("/forms") view=native::FormsPage ssr=PRIMARY_SSR_MODE/>
                <Route path=path!("/forms/new") view=native::FormsNewPage ssr=PRIMARY_SSR_MODE/>
                <Route path=path!("/forms/:form_id") view=native::FormsDetailPage ssr=PRIMARY_SSR_MODE/>
                <Route path=path!("/forms/:form_id/edit") view=native::FormsEditPage ssr=PRIMARY_SSR_MODE/>
                <Route path=path!("/workflows") view=native::WorkflowsPage ssr=PRIMARY_SSR_MODE/>
                <Route path=path!("/workflows/new") view=native::WorkflowsNewPage ssr=PRIMARY_SSR_MODE/>
                <Route path=path!("/workflows/assignments") view=native::WorkflowAssignmentsPage ssr=PRIMARY_SSR_MODE/>
                <Route path=path!("/workflows/:workflow_id") view=native::WorkflowsDetailPage ssr=PRIMARY_SSR_MODE/>
                <Route path=path!("/workflows/:workflow_id/edit") view=native::WorkflowsEditPage ssr=PRIMARY_SSR_MODE/>
                <Route path=path!("/responses") view=native::ResponsesPage ssr=PRIMARY_SSR_MODE/>
                <Route path=path!("/responses/new") view=native::ResponsesNewPage ssr=PRIMARY_SSR_MODE/>
                <Route path=path!("/responses/:submission_id") view=native::ResponsesDetailPage ssr=PRIMARY_SSR_MODE/>
                <Route path=path!("/responses/:submission_id/edit") view=native::ResponsesEditPage ssr=PRIMARY_SSR_MODE/>
                <Route path=path!("/operations") view=native::OperationsPage ssr=PRIMARY_SSR_MODE/>
                <Route path=path!("/components") view=native::ComponentsPage ssr=PRIMARY_SSR_MODE/>
                <Route path=path!("/components/:component_ref") view=native::ComponentsDetailPage ssr=PRIMARY_SSR_MODE/>
                <Route path=path!("/dashboards") view=native::DashboardsPage ssr=PRIMARY_SSR_MODE/>
                <Route path=path!("/dashboards/new") view=native::DashboardsNewPage ssr=PRIMARY_SSR_MODE/>
                <Route path=path!("/dashboards/:dashboard_id") view=native::DashboardsDetailPage ssr=PRIMARY_SSR_MODE/>
                <Route path=path!("/dashboards/:dashboard_id/edit") view=native::DashboardsEditPage ssr=PRIMARY_SSR_MODE/>
                <Route path=path!("/datasets") view=native::DatasetsPage ssr=PRIMARY_SSR_MODE/>
                <Route path=path!("/datasets/:dataset_id") view=native::DatasetsDetailPage ssr=PRIMARY_SSR_MODE/>
                <Route path=path!("/administration") view=native::AdministrationPage ssr=PRIMARY_SSR_MODE/>
                <Route path=path!("/administration/users/:account_id/access") view=native::AdministrationUserAccessPage ssr=PRIMARY_SSR_MODE/>
                <Route path=path!("/administration/users/:account_id/edit") view=native::AdministrationUserEditPage ssr=PRIMARY_SSR_MODE/>
                <Route path=path!("/administration/users/:account_id") view=native::AdministrationUserDetailPage ssr=PRIMARY_SSR_MODE/>
                <Route path=path!("/administration/users") view=native::AdministrationUsersPage ssr=PRIMARY_SSR_MODE/>
                <Route path=path!("/administration/node-types") view=native::AdministrationNodeTypesPage ssr=PRIMARY_SSR_MODE/>
                <Route path=path!("/administration/roles") view=native::AdministrationRolesPage ssr=PRIMARY_SSR_MODE/>
            </Routes>
        </Router>
    }
}
