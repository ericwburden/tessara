//! Top-level Leptos application composition.
//!
//! Keep router mounting, global context provisioning, and wasm hydration setup here; route definitions and screen behavior belong in `routes` and `features`.

#[cfg(feature = "hydrate")]
use leptos::context::Provider;
use leptos::{children::ToChildren, prelude::*};
use leptos_router::components::{Router, Routes};
#[cfg(feature = "hydrate")]
use wasm_bindgen::{JsCast, closure::Closure, prelude::wasm_bindgen};

use crate::routes;
use crate::state::session::provide_shell_session;
#[cfg(any(feature = "hydrate", test))]
use tessara_web_dashboards::DashboardRouteBootstrap;

#[cfg(feature = "hydrate")]
#[wasm_bindgen]
pub fn hydrate_app(root_id: &str) {
    use leptos::mount::{hydrate_from, mount_to};
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

    let dashboard_bootstrap = document
        .get_element_by_id(crate::DASHBOARD_BOOTSTRAP_SCRIPT_ID)
        .and_then(|script| script.text_content())
        .and_then(|json| parse_dashboard_bootstrap_json(&json));

    if let Some(bootstrap) = dashboard_bootstrap {
        let handle = hydrate_from(root, move || {
            view! {
                <Provider value=bootstrap>
                    <App/>
                </Provider>
            }
        });
        schedule_hydration_ready(true);
        handle.forget();
    } else {
        root.set_inner_html("");
        let handle = mount_to(root, App);
        schedule_hydration_ready(false);
        handle.forget();
    }
}

#[cfg(feature = "hydrate")]
fn schedule_hydration_ready(clear_dashboard_bootstrap: bool) {
    let second_frame = Closure::once(move |_: f64| {
        mark_hydration_ready(clear_dashboard_bootstrap);
    });
    let first_frame = Closure::once(move |_: f64| {
        let Some(window) = web_sys::window() else {
            mark_hydration_ready(clear_dashboard_bootstrap);
            return;
        };
        if window
            .request_animation_frame(second_frame.as_ref().unchecked_ref())
            .is_ok()
        {
            second_frame.forget();
        } else {
            mark_hydration_ready(clear_dashboard_bootstrap);
        }
    });
    let Some(window) = web_sys::window() else {
        mark_hydration_ready(clear_dashboard_bootstrap);
        return;
    };
    if window
        .request_animation_frame(first_frame.as_ref().unchecked_ref())
        .is_ok()
    {
        first_frame.forget();
    } else {
        mark_hydration_ready(clear_dashboard_bootstrap);
    }
}

#[cfg(feature = "hydrate")]
fn mark_hydration_ready(clear_dashboard_bootstrap: bool) {
    if clear_dashboard_bootstrap {
        tessara_web_dashboards::clear_dashboard_route_bootstrap();
    }
    if let Some(root) = web_sys::window()
        .and_then(|window| window.document())
        .and_then(|document| document.get_element_by_id(crate::pipeline::APP_ROOT_ID))
    {
        let _ = root.set_attribute("data-hydration", "ready");
    }
}

#[cfg(any(feature = "hydrate", test))]
fn parse_dashboard_bootstrap_json(json: &str) -> Option<DashboardRouteBootstrap> {
    serde_json::from_str(json).ok()
}

#[component]
pub fn App() -> impl IntoView {
    let _ = provide_shell_session();

    view! {
        <Router>
            <Routes
                fallback=|| view! { <routes::NotFoundPage/> }
                children=ToChildren::to_children(routes::routes)
            />
        </Router>
    }
}

#[cfg(test)]
mod tests {
    use tessara_web_dashboards::{DashboardRouteBootstrap, DashboardSummary, SessionAccount};

    use super::parse_dashboard_bootstrap_json;

    #[test]
    fn hydration_parser_accepts_serialized_route_state_and_rejects_invalid_json() {
        let bootstrap = DashboardRouteBootstrap::directory(
            SessionAccount {
                capabilities: vec!["dashboards:read".into()],
            },
            vec![DashboardSummary {
                id: "dashboard-7".into(),
                name: "Operations".into(),
                description: None,
                visibility_nodes: Vec::new(),
                placement_count: 4,
                can_manage: false,
            }],
        );
        let json = serde_json::to_string(&bootstrap).expect("serialize bootstrap");

        assert_eq!(parse_dashboard_bootstrap_json(&json), Some(bootstrap));
        assert_eq!(parse_dashboard_bootstrap_json("{"), None);
    }
}
