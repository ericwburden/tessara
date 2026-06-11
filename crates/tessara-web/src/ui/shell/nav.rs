//! Shared shell navigation rendering helpers.
//!
//! This module owns reusable navigation item markup and active-state mapping used by both desktop and mobile shell surfaces.

use crate::features::auth;
use crate::state::navigation;
use crate::state::session::{shell_session_account, submit_logout};
use crate::ui::empty_view;
use icons::{
    CircleHelp, Database, File, FileText, GitBranch, House, LayoutDashboard, ListChecks, LogOut,
    PanelRight, Pencil, SlidersHorizontal,
};
use leptos::prelude::*;

#[component]
/// Renders the sidebar content view.
pub(crate) fn SidebarContent(active_route: &'static str) -> impl IntoView {
    let account = shell_session_account();

    view! {
        <a class="brand-lockup" href="/">
            <span class="brand-mark" aria-hidden="true">
                <img src="/assets/tessara-icon-256.svg" alt=""/>
            </span>
            <span class="brand-copy">
                <strong>"Tessara"</strong>
            </span>
        </a>
        <nav class="sidebar-nav" aria-label="Primary">
            {move || nav_section_for("Main", active_route, account.get())}
            {move || nav_section_for("Admin", active_route, account.get())}
        </nav>
        <AccountCard account/>
    }
}

#[component]
/// Renders the account card view.
fn AccountCard(account: RwSignal<Option<auth::ShellAccountSummary>>) -> impl IntoView {
    view! {
        <section class="account-card" aria-label="Account context">
            <span class="account-avatar">
                {move || {
                    account
                        .get()
                        .as_ref()
                        .map(|account| account_initials(&account.display_name, &account.email))
                        .unwrap_or_else(|| "--".to_string())
                }}
            </span>
            <span class="account-copy">
                <strong>
                    {move || {
                        account
                            .get()
                            .as_ref()
                            .map(|account| account.display_name.clone())
                            .unwrap_or_else(|| "Signed out".to_string())
                    }}
                </strong>
                <small>
                    {move || {
                        account
                            .get()
                            .as_ref()
                            .map(|account| account.email.clone())
                            .unwrap_or_else(|| "No active session".to_string())
                    }}
                </small>
            </span>
            <button
                class="icon-button account-card__logout"
                type="button"
                aria-label="Sign out"
                title="Sign out"
                on:click=move |_| submit_logout()
            >
                <LogOut class="icon-button__icon"/>
            </button>
        </section>
    }
}

/// Handles the account initials behavior.
fn account_initials(display_name: &str, email: &str) -> String {
    let initials = display_name
        .split_whitespace()
        .filter_map(|part| part.chars().next())
        .take(2)
        .collect::<String>()
        .to_uppercase();

    if !initials.is_empty() {
        return initials;
    }

    email.chars().take(2).collect::<String>().to_uppercase()
}

#[cfg_attr(not(feature = "hydrate"), allow(dead_code))]
/// Handles the nav section for behavior.
pub(crate) fn nav_section_for(
    section: &'static str,
    active_route: &'static str,
    account: Option<auth::ShellAccountSummary>,
) -> impl IntoView {
    let capabilities = account
        .as_ref()
        .map(|account| account.capabilities.as_slice())
        .unwrap_or(&[]);
    let items = navigation::nav_items_for_section(section, capabilities);

    if items.is_empty() {
        return empty_view();
    }

    view! {
        <p class="sidebar-section">{section}</p>
        {items
            .into_iter()
            .map(move |item| {
                nav_item_link(item, active_route)
            })
            .collect_view()}
    }
    .into_any()
}

/// Handles the nav item link behavior.
fn nav_item_link(item: &'static navigation::NavItem, active_route: &'static str) -> impl IntoView {
    let class = if item.key == active_route {
        "sidebar-link is-active"
    } else {
        "sidebar-link"
    };
    view! {
        <a class=class href=item.href title=item.label aria-label=item.label>
            {nav_icon_for(item.key)}
            <span class="sidebar-link__label">{item.label}</span>
        </a>
    }
}

/// Handles the nav icon for behavior.
fn nav_icon_for(route_key: &'static str) -> impl IntoView {
    match route_key {
        "home" => view! { <span class="sidebar-link__icon-wrap" aria-hidden="true"><House class="sidebar-link__icon"/></span> }.into_any(),
        "organization" => view! { <span class="sidebar-link__icon-wrap" aria-hidden="true"><GitBranch class="sidebar-link__icon"/></span> }.into_any(),
        "forms" => view! { <span class="sidebar-link__icon-wrap" aria-hidden="true"><FileText class="sidebar-link__icon"/></span> }.into_any(),
        "workflows" => view! { <span class="sidebar-link__icon-wrap" aria-hidden="true"><PanelRight class="sidebar-link__icon"/></span> }.into_any(),
        "responses" => view! { <span class="sidebar-link__icon-wrap" aria-hidden="true"><CircleHelp class="sidebar-link__icon"/></span> }.into_any(),
        "operations" => view! { <span class="sidebar-link__icon-wrap" aria-hidden="true"><ListChecks class="sidebar-link__icon"/></span> }.into_any(),
        "components" => view! { <span class="sidebar-link__icon-wrap" aria-hidden="true"><Pencil class="sidebar-link__icon"/></span> }.into_any(),
        "dashboards" => view! { <span class="sidebar-link__icon-wrap" aria-hidden="true"><LayoutDashboard class="sidebar-link__icon"/></span> }.into_any(),
        "datasets" => view! { <span class="sidebar-link__icon-wrap" aria-hidden="true"><Database class="sidebar-link__icon"/></span> }.into_any(),
        "administration" => view! { <span class="sidebar-link__icon-wrap" aria-hidden="true"><SlidersHorizontal class="sidebar-link__icon"/></span> }.into_any(),
        _ => view! { <span class="sidebar-link__icon-wrap" aria-hidden="true"><File class="sidebar-link__icon"/></span> }.into_any(),
    }
}
