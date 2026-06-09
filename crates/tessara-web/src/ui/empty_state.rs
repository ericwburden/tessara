use leptos::prelude::*;

#[component]
pub fn EmptyState(title: &'static str, message: &'static str) -> impl IntoView {
    view! {
        <section class="empty-state">
            <h3>{title}</h3>
            <p>{message}</p>
        </section>
    }
}
