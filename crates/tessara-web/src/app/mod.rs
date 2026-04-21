use leptos::prelude::*;
use leptos_router::components::{Route, Router, Routes};
use leptos_router::{SsrMode, path};
#[cfg(feature = "hydrate")]
use wasm_bindgen::{JsCast, prelude::wasm_bindgen};

pub(crate) mod transitional;

use crate::features::{
    administration, components, dashboards, datasets, forms, home, migration, organization,
    reporting, responses, workflows,
};

const PRIMARY_SSR_MODE: SsrMode = SsrMode::InOrder;

#[cfg(feature = "hydrate")]
fn install_document_navigation_fallback() {
    use wasm_bindgen::closure::Closure;
    use web_sys::{Element, MouseEvent, window};

    let Some(document) = window().and_then(|window| window.document()) else {
        return;
    };
    let Some(body) = document.body() else {
        return;
    };
    if body.get_attribute("data-hybrid-shell").as_deref() != Some("true") {
        return;
    }

    let closure = Closure::wrap(Box::new(move |event: MouseEvent| {
        if event.default_prevented()
            || event.button() != 0
            || event.alt_key()
            || event.ctrl_key()
            || event.meta_key()
            || event.shift_key()
        {
            return;
        }

        let Some(target) = event.target() else {
            return;
        };
        let Ok(target) = target.dyn_into::<Element>() else {
            return;
        };
        let Ok(Some(anchor)) = target.closest("a[href]") else {
            return;
        };
        let Some(href) = anchor.get_attribute("href") else {
            return;
        };
        if href == "#" || !(href == "/" || href.starts_with("/app")) {
            return;
        }
        if anchor.has_attribute("download") {
            return;
        }
        if anchor
            .get_attribute("target")
            .is_some_and(|target| !target.is_empty() && target != "_self")
        {
            return;
        }

        event.prevent_default();
        if let Some(window) = window() {
            let _ = window.location().set_href(&href);
        }
    }) as Box<dyn FnMut(_)>);

    let _ = document.add_event_listener_with_callback_and_bool(
        "click",
        closure.as_ref().unchecked_ref(),
        true,
    );
    closure.forget();
}

#[cfg(feature = "hydrate")]
#[wasm_bindgen]
pub fn hydrate_app(root_id: &str) {
    use leptos::mount::hydrate_from_async;
    use web_sys::window;

    install_document_navigation_fallback();
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
    // Route inventory note:
    // `/app/login`, `/app/organization/*`, `/app/datasets/*`, `/app/components/*`,
    // `/app/administration/*`, `/app/dashboards/*`, and `/app/migration`
    // have explicit route ownership.
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
                <Route path=path!("/app/workflows") view=workflows::WorkflowsPage ssr=PRIMARY_SSR_MODE />
                <Route path=path!("/app/workflows/new") view=workflows::WorkflowCreatePage ssr=PRIMARY_SSR_MODE />
                <Route path=path!("/app/workflows/assignments") view=workflows::WorkflowAssignmentsPage ssr=PRIMARY_SSR_MODE />
                <Route path=path!("/app/workflows/:workflow_id") view=workflows::WorkflowDetailPage ssr=PRIMARY_SSR_MODE />
                <Route path=path!("/app/workflows/:workflow_id/edit") view=workflows::WorkflowEditPage ssr=PRIMARY_SSR_MODE />
                <Route path=path!("/app/responses") view=responses::ResponsesListPage ssr=PRIMARY_SSR_MODE />
                <Route path=path!("/app/responses/new") view=responses::ResponseCreatePage ssr=PRIMARY_SSR_MODE />
                <Route path=path!("/app/responses/:submission_id") view=responses::ResponseDetailPage ssr=PRIMARY_SSR_MODE />
                <Route path=path!("/app/responses/:submission_id/edit") view=responses::ResponseEditPage ssr=PRIMARY_SSR_MODE />
                <Route path=path!("/app/submissions") view=responses::SubmissionsPage ssr=PRIMARY_SSR_MODE />
                <Route path=path!("/app/reports") view=reporting::ReportsPage ssr=PRIMARY_SSR_MODE />
                <Route path=path!("/app/reports/new") view=reporting::ReportCreatePage ssr=PRIMARY_SSR_MODE />
                <Route path=path!("/app/reports/:report_id") view=reporting::ReportDetailPage ssr=PRIMARY_SSR_MODE />
                <Route path=path!("/app/reports/:report_id/edit") view=reporting::ReportEditPage ssr=PRIMARY_SSR_MODE />
                <Route path=path!("/app/components") view=components::ComponentsPage ssr=PRIMARY_SSR_MODE />
                <Route path=path!("/app/components/:component_ref") view=components::ComponentDetailPage ssr=PRIMARY_SSR_MODE />
                <Route path=path!("/app/dashboards") view=dashboards::DashboardsPage ssr=PRIMARY_SSR_MODE />
                <Route path=path!("/app/dashboards/new") view=dashboards::DashboardCreatePage ssr=PRIMARY_SSR_MODE />
                <Route path=path!("/app/dashboards/:dashboard_id") view=dashboards::DashboardDetailPage ssr=PRIMARY_SSR_MODE />
                <Route path=path!("/app/dashboards/:dashboard_id/edit") view=dashboards::DashboardEditPage ssr=PRIMARY_SSR_MODE />
                <Route path=path!("/app/datasets") view=datasets::DatasetsPage ssr=PRIMARY_SSR_MODE />
                <Route path=path!("/app/datasets/:dataset_id") view=datasets::DatasetDetailPage ssr=PRIMARY_SSR_MODE />
                <Route path=path!("/app/administration") view=administration::AdministrationPage ssr=PRIMARY_SSR_MODE />
                <Route path=path!("/app/administration/users") view=administration::UsersPage ssr=PRIMARY_SSR_MODE />
                <Route path=path!("/app/administration/users/new") view=administration::UserCreatePage ssr=PRIMARY_SSR_MODE />
                <Route path=path!("/app/administration/users/:account_id") view=administration::UserDetailPage ssr=PRIMARY_SSR_MODE />
                <Route path=path!("/app/administration/users/:account_id/edit") view=administration::UserEditPage ssr=PRIMARY_SSR_MODE />
                <Route path=path!("/app/administration/users/:account_id/access") view=administration::UserAccessPage ssr=PRIMARY_SSR_MODE />
                <Route path=path!("/app/administration/node-types") view=administration::NodeTypesPage ssr=PRIMARY_SSR_MODE />
                <Route path=path!("/app/administration/node-types/new") view=administration::NodeTypeCreatePage ssr=PRIMARY_SSR_MODE />
                <Route path=path!("/app/administration/node-types/:node_type_id") view=administration::NodeTypeDetailPage ssr=PRIMARY_SSR_MODE />
                <Route path=path!("/app/administration/node-types/:node_type_id/edit") view=administration::NodeTypeEditPage ssr=PRIMARY_SSR_MODE />
                <Route path=path!("/app/administration/roles") view=administration::RolesPage ssr=PRIMARY_SSR_MODE />
                <Route path=path!("/app/administration/roles/new") view=administration::RoleCreatePage ssr=PRIMARY_SSR_MODE />
                <Route path=path!("/app/administration/roles/:role_id") view=administration::RoleDetailPage ssr=PRIMARY_SSR_MODE />
                <Route path=path!("/app/administration/roles/:role_id/edit") view=administration::RoleEditPage ssr=PRIMARY_SSR_MODE />
                <Route path=path!("/app/admin") view=administration::LegacyAdminPage ssr=PRIMARY_SSR_MODE />
                <Route path=path!("/app/migration") view=migration::MigrationPage ssr=PRIMARY_SSR_MODE />
            </Routes>
        </Router>
    }
}
