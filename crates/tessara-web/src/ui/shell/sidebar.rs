//! Desktop sidebar shell navigation.
//!
//! Keep persistent navigation, section grouping, account affordances, and desktop shell links here.

use crate::ui::shell::nav::SidebarContent;
use leptos::prelude::*;

#[component]
pub fn Sidebar(active_route: &'static str) -> impl IntoView {
    view! {
        <aside class="sidebar" aria-label="Primary navigation">
            <SidebarContent active_route/>
        </aside>
    }
}
