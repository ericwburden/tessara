//! Loading placeholder primitive.

use leptos::prelude::*;

#[component]
pub fn Skeleton(#[prop(optional)] class: &'static str) -> impl IntoView {
    let class_name = if class.is_empty() {
        "skeleton".to_string()
    } else {
        format!("skeleton {class}")
    };

    view! { <span class=class_name aria-hidden="true"></span> }
}
