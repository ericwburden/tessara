//! Top-level Leptos application composition.
//!
//! Keep router mounting, global context provisioning, and wasm hydration setup here; route definitions and screen behavior belong in `routes` and `features`.

#[cfg(feature = "hydrate")]
use leptos::context::Provider;
use leptos::{children::ToChildren, prelude::*};
use leptos_router::components::{Router, Routes};
#[cfg(feature = "hydrate")]
use wasm_bindgen::{JsCast, closure::Closure, prelude::wasm_bindgen};

#[cfg(any(feature = "hydrate", test))]
use crate::features::modules::ModuleManagementRouteBootstrapV1;
use crate::routes;
use crate::state::session::provide_shell_session;
use crate::state::shell_navigation::ShellNavigationResponseV1;
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
    let module_management_bootstrap = document
        .get_element_by_id(crate::MODULE_MANAGEMENT_BOOTSTRAP_SCRIPT_ID)
        .and_then(|script| script.text_content())
        .and_then(|json| parse_module_bootstrap_json(&json));
    let shell_navigation_bootstrap = document
        .get_element_by_id(crate::SHELL_NAVIGATION_BOOTSTRAP_SCRIPT_ID)
        .and_then(|script| script.text_content())
        .and_then(|json| parse_shell_navigation_bootstrap_json(&json))
        .filter(ShellNavigationResponseV1::is_supported);
    let has_shell_navigation_bootstrap = shell_navigation_bootstrap.is_some();

    match (dashboard_bootstrap, module_management_bootstrap) {
        (Some(dashboard), Some(module_management)) => {
            let handle = hydrate_from(root, move || {
                view! {
                    <Provider value=dashboard>
                        <Provider value=module_management>
                            <App initial_shell_navigation=shell_navigation_bootstrap/>
                        </Provider>
                    </Provider>
                }
            });
            schedule_hydration_ready(true, true, has_shell_navigation_bootstrap);
            handle.forget();
        }
        (Some(dashboard), None) => {
            let handle = hydrate_from(root, move || {
                view! {
                    <Provider value=dashboard>
                        <App initial_shell_navigation=shell_navigation_bootstrap/>
                    </Provider>
                }
            });
            schedule_hydration_ready(true, false, has_shell_navigation_bootstrap);
            handle.forget();
        }
        (None, Some(module_management)) => {
            let handle = hydrate_from(root, move || {
                view! {
                    <Provider value=module_management>
                        <App initial_shell_navigation=shell_navigation_bootstrap/>
                    </Provider>
                }
            });
            schedule_hydration_ready(false, true, has_shell_navigation_bootstrap);
            handle.forget();
        }
        (None, None) => {
            let handle = if has_shell_navigation_bootstrap {
                hydrate_from(root, move || {
                    view! { <App initial_shell_navigation=shell_navigation_bootstrap/> }
                })
            } else {
                root.set_inner_html("");
                mount_to(root, || view! { <App initial_shell_navigation=None/> })
            };
            schedule_hydration_ready(false, false, has_shell_navigation_bootstrap);
            handle.forget();
        }
    }
}

#[cfg(feature = "hydrate")]
fn schedule_hydration_ready(
    clear_dashboard_bootstrap: bool,
    clear_module_management_bootstrap: bool,
    clear_shell_navigation_bootstrap: bool,
) {
    let second_frame = Closure::once(move |_: f64| {
        mark_hydration_ready(
            clear_dashboard_bootstrap,
            clear_module_management_bootstrap,
            clear_shell_navigation_bootstrap,
        );
    });
    let first_frame = Closure::once(move |_: f64| {
        let Some(window) = web_sys::window() else {
            mark_hydration_ready(
                clear_dashboard_bootstrap,
                clear_module_management_bootstrap,
                clear_shell_navigation_bootstrap,
            );
            return;
        };
        if window
            .request_animation_frame(second_frame.as_ref().unchecked_ref())
            .is_ok()
        {
            second_frame.forget();
        } else {
            mark_hydration_ready(
                clear_dashboard_bootstrap,
                clear_module_management_bootstrap,
                clear_shell_navigation_bootstrap,
            );
        }
    });
    let Some(window) = web_sys::window() else {
        mark_hydration_ready(
            clear_dashboard_bootstrap,
            clear_module_management_bootstrap,
            clear_shell_navigation_bootstrap,
        );
        return;
    };
    if window
        .request_animation_frame(first_frame.as_ref().unchecked_ref())
        .is_ok()
    {
        first_frame.forget();
    } else {
        mark_hydration_ready(
            clear_dashboard_bootstrap,
            clear_module_management_bootstrap,
            clear_shell_navigation_bootstrap,
        );
    }
}

#[cfg(feature = "hydrate")]
fn mark_hydration_ready(
    clear_dashboard_bootstrap: bool,
    clear_module_management_bootstrap: bool,
    clear_shell_navigation_bootstrap: bool,
) {
    if clear_dashboard_bootstrap {
        tessara_web_dashboards::clear_dashboard_route_bootstrap();
    }
    if clear_module_management_bootstrap {
        crate::features::modules::clear_module_management_route_bootstrap();
    }
    if clear_shell_navigation_bootstrap
        && let Some(script) = web_sys::window()
            .and_then(|window| window.document())
            .and_then(|document| {
                document.get_element_by_id(crate::SHELL_NAVIGATION_BOOTSTRAP_SCRIPT_ID)
            })
    {
        script.remove();
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

#[cfg(any(feature = "hydrate", test))]
fn parse_module_bootstrap_json(json: &str) -> Option<ModuleManagementRouteBootstrapV1> {
    crate::features::modules::parse_module_management_bootstrap_json(json)
}

#[cfg(any(feature = "hydrate", test))]
fn parse_shell_navigation_bootstrap_json(json: &str) -> Option<ShellNavigationResponseV1> {
    serde_json::from_str(json).ok()
}

#[component]
pub fn App(initial_shell_navigation: Option<ShellNavigationResponseV1>) -> impl IntoView {
    let _ = provide_shell_session(initial_shell_navigation);

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
    use crate::{
        features::modules::{ModuleManagementRouteBootstrapV1, ModuleManagementSurfaceV1},
        state::shell_navigation::{
            ShellNavigationGroupV1, ShellNavigationItemOwnerV1, ShellNavigationItemV1,
            ShellNavigationResponseV1, ShellNavigationStateV1,
        },
    };
    use tessara_web_dashboards::{DashboardRouteBootstrap, DashboardSummary, SessionAccount};

    use super::{
        parse_dashboard_bootstrap_json, parse_module_bootstrap_json,
        parse_shell_navigation_bootstrap_json,
    };

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

    #[test]
    fn hydration_parsers_preserve_module_and_shell_request_state() {
        let module =
            ModuleManagementRouteBootstrapV1::restricted(ModuleManagementSurfaceV1::Directory);
        let module_json = serde_json::to_string(&module).expect("serialize module bootstrap");
        assert_eq!(parse_module_bootstrap_json(&module_json), Some(module));

        let shell = ShellNavigationResponseV1 {
            schema_version: 2,
            policy_revision: Some(0),
            state: ShellNavigationStateV1::Available,
            groups: vec![ShellNavigationGroupV1 {
                id: "core.main".into(),
                name: "Main".into(),
                items: vec![ShellNavigationItemV1 {
                    key: "home".into(),
                    label: "Home".into(),
                    href: "/".into(),
                    owner: ShellNavigationItemOwnerV1::Core,
                    contribution_id: None,
                }],
            }],
            unavailable: None,
        };
        let shell_json = serde_json::to_string(&shell).expect("serialize shell bootstrap");
        assert_eq!(
            parse_shell_navigation_bootstrap_json(&shell_json),
            Some(shell)
        );
        assert_eq!(parse_module_bootstrap_json("{"), None);
        assert_eq!(parse_shell_navigation_bootstrap_json("{"), None);
    }
}
