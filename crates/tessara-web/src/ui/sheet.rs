use leptos::prelude::*;

#[component]
pub fn Sheet(#[prop(optional)] children: Option<Children>) -> impl IntoView {
    view! {
        <div class="sheet">
            {children.map(|children| children())}
        </div>
    }
}

#[component]
pub fn Drawer(#[prop(optional)] children: Option<Children>) -> impl IntoView {
    view! {
        <aside class="drawer">
            {children.map(|children| children())}
        </aside>
    }
}
