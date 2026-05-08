use leptos::prelude::*;
use leptos_router::components::{Route, Router, Routes};
use leptos_router::{SsrMode, path};
#[cfg(feature = "hydrate")]
use wasm_bindgen::{JsCast, prelude::wasm_bindgen};

use crate::features::reset;

const PRIMARY_SSR_MODE: SsrMode = SsrMode::InOrder;

#[cfg(feature = "hydrate")]
#[wasm_bindgen]
pub fn hydrate_app(root_id: &str) {
    use leptos::mount::hydrate_from_async;
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

    leptos::task::spawn_local(async move {
        let handle = hydrate_from_async(root, App).await;
        handle.forget();
    });
}

#[component]
pub fn App() -> impl IntoView {
    view! {
        <Router>
            <Routes fallback=|| view! { <reset::NotFoundPage/> }>
                <Route path=path!("/") view=reset::HomePage ssr=PRIMARY_SSR_MODE/>
                <Route path=path!("/login") view=reset::LoginPage ssr=PRIMARY_SSR_MODE/>
                <Route path=path!("/organization") view=reset::OrganizationPage ssr=PRIMARY_SSR_MODE/>
                <Route path=path!("/organization/new") view=reset::OrganizationNewPage ssr=PRIMARY_SSR_MODE/>
                <Route path=path!("/organization/:node_id") view=reset::OrganizationDetailPage ssr=PRIMARY_SSR_MODE/>
                <Route path=path!("/organization/:node_id/edit") view=reset::OrganizationEditPage ssr=PRIMARY_SSR_MODE/>
                <Route path=path!("/forms") view=reset::FormsPage ssr=PRIMARY_SSR_MODE/>
                <Route path=path!("/forms/new") view=reset::FormsNewPage ssr=PRIMARY_SSR_MODE/>
                <Route path=path!("/forms/:form_id") view=reset::FormsDetailPage ssr=PRIMARY_SSR_MODE/>
                <Route path=path!("/forms/:form_id/edit") view=reset::FormsEditPage ssr=PRIMARY_SSR_MODE/>
                <Route path=path!("/workflows") view=reset::WorkflowsPage ssr=PRIMARY_SSR_MODE/>
                <Route path=path!("/workflows/new") view=reset::WorkflowsNewPage ssr=PRIMARY_SSR_MODE/>
                <Route path=path!("/workflows/assignments") view=reset::WorkflowAssignmentsPage ssr=PRIMARY_SSR_MODE/>
                <Route path=path!("/workflows/:workflow_id") view=reset::WorkflowsDetailPage ssr=PRIMARY_SSR_MODE/>
                <Route path=path!("/workflows/:workflow_id/edit") view=reset::WorkflowsEditPage ssr=PRIMARY_SSR_MODE/>
                <Route path=path!("/responses") view=reset::ResponsesPage ssr=PRIMARY_SSR_MODE/>
                <Route path=path!("/responses/new") view=reset::ResponsesNewPage ssr=PRIMARY_SSR_MODE/>
                <Route path=path!("/responses/:submission_id") view=reset::ResponsesDetailPage ssr=PRIMARY_SSR_MODE/>
                <Route path=path!("/responses/:submission_id/edit") view=reset::ResponsesEditPage ssr=PRIMARY_SSR_MODE/>
                <Route path=path!("/components") view=reset::ComponentsPage ssr=PRIMARY_SSR_MODE/>
                <Route path=path!("/components/:component_ref") view=reset::ComponentsDetailPage ssr=PRIMARY_SSR_MODE/>
                <Route path=path!("/dashboards") view=reset::DashboardsPage ssr=PRIMARY_SSR_MODE/>
                <Route path=path!("/dashboards/new") view=reset::DashboardsNewPage ssr=PRIMARY_SSR_MODE/>
                <Route path=path!("/dashboards/:dashboard_id") view=reset::DashboardsDetailPage ssr=PRIMARY_SSR_MODE/>
                <Route path=path!("/dashboards/:dashboard_id/edit") view=reset::DashboardsEditPage ssr=PRIMARY_SSR_MODE/>
                <Route path=path!("/datasets") view=reset::DatasetsPage ssr=PRIMARY_SSR_MODE/>
                <Route path=path!("/datasets/:dataset_id") view=reset::DatasetsDetailPage ssr=PRIMARY_SSR_MODE/>
                <Route path=path!("/administration") view=reset::AdministrationPage ssr=PRIMARY_SSR_MODE/>
                <Route path=path!("/administration/users") view=reset::AdministrationUsersPage ssr=PRIMARY_SSR_MODE/>
                <Route path=path!("/administration/node-types") view=reset::AdministrationNodeTypesPage ssr=PRIMARY_SSR_MODE/>
                <Route path=path!("/administration/roles") view=reset::AdministrationRolesPage ssr=PRIMARY_SSR_MODE/>
                <Route path=path!("/migration") view=reset::MigrationPage ssr=PRIMARY_SSR_MODE/>
            </Routes>
        </Router>
    }
}
