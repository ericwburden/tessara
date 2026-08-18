//! Canonical Tessara sidebar presentation shared by Core and complete module documents.

use icons::{
    Blocks, CircleHelp, Database, File, FileText, GitBranch, House, LayoutDashboard, ListChecks,
    LogOut, Network, PanelRight, Pencil, ShieldCheck, Users,
};
use leptos::prelude::*;
use tessara_module_contract::{OriginalActorProjectionV1, ShellNavigationGroupProjectionV2};

use crate::navigation_path_matches;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ShellAccountPresentation {
    pub display_name: String,
    pub email: Option<String>,
}

impl From<OriginalActorProjectionV1> for ShellAccountPresentation {
    fn from(actor: OriginalActorProjectionV1) -> Self {
        Self {
            display_name: actor.display_name,
            email: actor.email,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ShellNavigationItemPresentation {
    pub key: String,
    pub label: String,
    pub href: String,
    pub document_navigation: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ShellNavigationGroupPresentation {
    pub id: String,
    pub label: String,
    pub items: Vec<ShellNavigationItemPresentation>,
}

impl From<ShellNavigationGroupProjectionV2> for ShellNavigationGroupPresentation {
    fn from(group: ShellNavigationGroupProjectionV2) -> Self {
        Self {
            id: group.id,
            label: group.label,
            items: group
                .items
                .into_iter()
                .map(|item| ShellNavigationItemPresentation {
                    key: item.key,
                    label: item.label,
                    href: item.href,
                    document_navigation: false,
                })
                .collect(),
        }
    }
}

#[component]
pub fn ShellSidebar(
    actor: ShellAccountPresentation,
    navigation: Vec<ShellNavigationGroupPresentation>,
    #[prop(into)] current_destination: String,
    #[prop(into)] return_destination: String,
    navigation_status: Option<String>,
    #[prop(optional)] on_sign_out: Option<Callback<()>>,
) -> impl IntoView {
    let desktop_actor = actor.clone();
    view! {
        <a class="brand-lockup" href=return_destination>
            <span class="brand-mark" aria-hidden="true">
                <img src="/assets/tessara-icon-256.svg" alt=""/>
            </span>
            <span class="brand-copy"><strong>"Tessara"</strong></span>
        </a>
        <nav class="sidebar-nav" aria-label="Primary">
            <div class="sidebar-navigation-projection">
                {navigation
                    .into_iter()
                    .map(|group| shell_navigation_group(group, current_destination.clone()))
                    .collect_view()}
                {navigation_status.map(|message| view! {
                    <p class="sidebar-navigation-status" role="status">{message}</p>
                })}
            </div>
        </nav>
        <section class="account-card" aria-label="Account context">
            <span class="account-avatar">{account_initials(&desktop_actor)}</span>
            <span class="account-copy">
                <strong>{desktop_actor.display_name}</strong>
                <small>{desktop_actor.email.unwrap_or_else(|| "Active session".into())}</small>
            </span>
            <button
                class="icon-button account-card__logout"
                type="button"
                aria-label="Sign out"
                title="Sign out"
                data-shell-sign-out="true"
                on:click=move |_| {
                    if let Some(callback) = on_sign_out {
                        callback.run(());
                    }
                }
            >
                <LogOut class="icon-button__icon"/>
            </button>
        </section>
    }
}

fn shell_navigation_group(
    group: ShellNavigationGroupPresentation,
    current_destination: String,
) -> impl IntoView {
    view! {
        <p class="sidebar-section">{group.label}</p>
        {group
            .items
            .into_iter()
            .map(|item| shell_navigation_item(item, current_destination.clone()))
            .collect_view()}
    }
}

fn shell_navigation_item(
    item: ShellNavigationItemPresentation,
    current_destination: String,
) -> impl IntoView {
    let class = if current_destination == item.key
        || navigation_path_matches(&current_destination, &item.href)
    {
        "sidebar-link is-active"
    } else {
        "sidebar-link"
    };
    let key = item.key;
    let label = item.label;
    let title = label.clone();
    let aria_label = label.clone();
    let rel = item.document_navigation.then_some("external");
    view! {
        <a
            class=class
            href=item.href
            rel=rel
            title=title
            aria-label=aria_label
        >
            <ShellNavigationIcon navigation_key=key/>
            <span class="sidebar-link__label">{label}</span>
        </a>
    }
}

#[component]
pub fn ShellNavigationIcon(#[prop(into)] navigation_key: String) -> impl IntoView {
    match navigation_key.as_str() {
        "home" => view! { <span class="sidebar-link__icon-wrap" aria-hidden="true"><House class="sidebar-link__icon"/></span> }.into_any(),
        "organization" => view! { <span class="sidebar-link__icon-wrap" aria-hidden="true"><GitBranch class="sidebar-link__icon"/></span> }.into_any(),
        "forms" => view! { <span class="sidebar-link__icon-wrap" aria-hidden="true"><FileText class="sidebar-link__icon"/></span> }.into_any(),
        "workflows" => view! { <span class="sidebar-link__icon-wrap" aria-hidden="true"><PanelRight class="sidebar-link__icon"/></span> }.into_any(),
        "responses" => view! { <span class="sidebar-link__icon-wrap" aria-hidden="true"><CircleHelp class="sidebar-link__icon"/></span> }.into_any(),
        "operations" => view! { <span class="sidebar-link__icon-wrap" aria-hidden="true"><ListChecks class="sidebar-link__icon"/></span> }.into_any(),
        "datasets" => view! { <span class="sidebar-link__icon-wrap" aria-hidden="true"><Database class="sidebar-link__icon"/></span> }.into_any(),
        "components" => view! { <span class="sidebar-link__icon-wrap" aria-hidden="true"><Pencil class="sidebar-link__icon"/></span> }.into_any(),
        "dashboards" => view! { <span class="sidebar-link__icon-wrap" aria-hidden="true"><LayoutDashboard class="sidebar-link__icon"/></span> }.into_any(),
        "user_management" => view! { <span class="sidebar-link__icon-wrap" aria-hidden="true"><Users class="sidebar-link__icon"/></span> }.into_any(),
        "roles_access" => view! { <span class="sidebar-link__icon-wrap" aria-hidden="true"><ShieldCheck class="sidebar-link__icon"/></span> }.into_any(),
        "node_types" => view! { <span class="sidebar-link__icon-wrap" aria-hidden="true"><Network class="sidebar-link__icon"/></span> }.into_any(),
        "module_management" => view! { <span class="sidebar-link__icon-wrap" aria-hidden="true"><Blocks class="sidebar-link__icon"/></span> }.into_any(),
        _ => view! { <span class="sidebar-link__icon-wrap" aria-hidden="true"><File class="sidebar-link__icon"/></span> }.into_any(),
    }
}

fn account_initials(actor: &ShellAccountPresentation) -> String {
    let initials = actor
        .display_name
        .split_whitespace()
        .filter_map(|part| part.chars().next())
        .take(2)
        .collect::<String>()
        .to_uppercase();
    if !initials.is_empty() {
        return initials;
    }
    actor
        .email
        .as_deref()
        .unwrap_or_default()
        .chars()
        .take(2)
        .collect::<String>()
        .to_uppercase()
}
