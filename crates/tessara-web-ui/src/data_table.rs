//! Shared data-table layout component.

use leptos::prelude::*;

#[component]
pub fn DataTable(children: Children) -> impl IntoView {
    view! {
        <div class="table-wrap">
            <table class="data-table">
                {children()}
            </table>
        </div>
    }
}
