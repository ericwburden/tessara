use leptos::{children::ToChildren, prelude::*};
use leptos_router::components::{Router, Routes};
#[cfg(feature = "hydrate")]
use wasm_bindgen::{JsCast, prelude::wasm_bindgen};

use crate::routes;
use crate::state::session::provide_shell_session;

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
