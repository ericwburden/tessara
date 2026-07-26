//! Shared shell navigation rendering helpers.
//!
//! This module owns reusable navigation item markup and active-state mapping used by both desktop and mobile shell surfaces.

use crate::features::auth;
use crate::state::navigation;
use crate::state::session::{shell_navigation_state, shell_session_account, submit_logout};
use crate::state::shell_navigation::{
    ShellNavigationGroupV1, ShellNavigationItemV1, ShellNavigationLoadState, ShellNavigationStateV1,
};
use crate::ui::empty_view;
use icons::{
    Blocks, CircleHelp, Database, File, FileText, GitBranch, House, LayoutDashboard, ListChecks,
    LogOut, Network, PanelRight, Pencil, ShieldCheck, SlidersHorizontal, Users,
};
use leptos::prelude::*;

#[component]
pub(crate) fn SidebarContent(active_route: &'static str) -> impl IntoView {
    let account = shell_session_account();
    let shell_navigation = shell_navigation_state();

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
            {move || {
                navigation_view(
                    active_route,
                    account.get(),
                    shell_navigation.get(),
                )
            }}
        </nav>
        <AccountCard account/>
    }
}

fn navigation_view(
    active_route: &'static str,
    account: Option<auth::ShellAccountSummary>,
    shell_navigation: ShellNavigationLoadState,
) -> AnyView {
    match shell_navigation {
        ShellNavigationLoadState::Ready(response) => {
            let unavailable = (response.state == ShellNavigationStateV1::Unavailable).then(|| {
                response
                    .unavailable
                    .map(|state| state.message)
                    .unwrap_or_else(unavailable_message)
            });
            view! {
                <div class="sidebar-navigation-projection">
                    {response
                        .groups
                        .into_iter()
                        .map(move |group| projected_nav_section(group, active_route))
                        .collect_view()}
                    {unavailable.map(navigation_unavailable_view)}
                </div>
            }
            .into_any()
        }
        ShellNavigationLoadState::Loading => navigation_loading_view().into_any(),
        ShellNavigationLoadState::Failed => {
            fallback_navigation(active_route, account, true).into_any()
        }
    }
}

fn navigation_loading_view() -> impl IntoView {
    view! {
        <div
            class="sidebar-navigation-projection sidebar-navigation-skeleton"
            aria-label="Loading navigation"
            aria-busy="true"
        >
            {navigation_loading_section("Main", 9)}
            {navigation_loading_section("Admin", 4)}
        </div>
    }
}

fn navigation_loading_section(label: &'static str, item_count: usize) -> impl IntoView {
    view! {
        <p class="sidebar-section">{label}</p>
        {(0..item_count)
            .map(|index| {
                let width_class = match index % 3 {
                    0 => "skeleton--wide",
                    1 => "skeleton--medium",
                    _ => "skeleton--short",
                };
                view! {
                    <div class="sidebar-link sidebar-navigation-skeleton__item" aria-hidden="true">
                        <span class="skeleton sidebar-navigation-skeleton__icon"></span>
                        <span class=format!("skeleton skeleton--text sidebar-navigation-skeleton__label {width_class}")></span>
                    </div>
                }
            })
            .collect_view()}
    }
}

fn projected_nav_section(
    group: ShellNavigationGroupV1,
    active_route: &'static str,
) -> impl IntoView {
    view! {
        <p class="sidebar-section">{group.name}</p>
        {group
            .items
            .into_iter()
            .map(move |item| projected_nav_item_link(item, active_route))
            .collect_view()}
    }
}

fn projected_nav_item_link(
    item: ShellNavigationItemV1,
    active_route: &'static str,
) -> impl IntoView {
    let class = if item.key == active_route {
        "sidebar-link is-active"
    } else {
        "sidebar-link"
    };
    let icon = nav_icon_for(&item.key);
    let label = item.label;
    let title = label.clone();
    let aria_label = label.clone();
    view! {
        <a
            class=class
            href=item.href
            title=title
            aria-label=aria_label
        >
            {icon}
            <span class="sidebar-link__label">{label}</span>
        </a>
    }
}

fn fallback_navigation(
    active_route: &'static str,
    account: Option<auth::ShellAccountSummary>,
    unavailable: bool,
) -> impl IntoView {
    view! {
        <div class="sidebar-navigation-projection">
            {nav_section_for("Main", active_route, account.clone())}
            {nav_section_for("Admin", active_route, account)}
            {unavailable.then(|| navigation_unavailable_view(unavailable_message()))}
        </div>
    }
}

fn unavailable_message() -> String {
    "Contribution navigation is temporarily unavailable.".to_string()
}

fn navigation_unavailable_view(message: String) -> impl IntoView {
    view! {
        <p class="sidebar-navigation-status" role="status">
            {message}
        </p>
    }
}

#[component]
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
pub(crate) fn nav_section_for(
    section: &'static str,
    active_route: &'static str,
    account: Option<auth::ShellAccountSummary>,
) -> impl IntoView {
    let capabilities = account
        .as_ref()
        .map(|account| account.capabilities.as_slice())
        .unwrap_or(&[]);
    // Product navigation retains the established flat effective-capability
    // behavior. Module Management is different: its direct capabilities are
    // installation-global, so accept those keys only from the authoritative
    // global companion set while the actor-filtered shell projection loads or
    // is unavailable. `admin:all` remains in the flat set as the universal
    // global sentinel.
    let mut fallback_capabilities = capabilities
        .iter()
        .filter(|capability| {
            capability.as_str() != "modules:read"
                && capability.as_str() != "modules:manage_navigation"
        })
        .cloned()
        .collect::<Vec<_>>();
    if let Some(account) = &account {
        fallback_capabilities.extend(
            account
                .global_capabilities
                .iter()
                .filter(|capability| {
                    matches!(
                        capability.as_str(),
                        "modules:read" | "modules:manage_navigation"
                    )
                })
                .cloned(),
        );
    }
    fallback_capabilities.sort();
    fallback_capabilities.dedup();
    let mut items = navigation::resolve_navigation(&[], None, &fallback_capabilities)
        .items
        .into_iter()
        .filter(|item| item.key != "administration")
        .filter(|item| item.section.as_str() == section)
        .collect::<Vec<_>>();
    if section == "Admin"
        && fallback_capabilities
            .iter()
            .any(|capability| matches!(capability.as_str(), "admin:all" | "core:admin"))
    {
        let insert_at = items
            .iter()
            .position(|item| item.key == "module_management")
            .unwrap_or(items.len());
        items.splice(
            insert_at..insert_at,
            [
                (
                    "user_management",
                    "/administration/users",
                    "User Management",
                ),
                ("roles_access", "/administration/roles", "Roles & Access"),
                ("node_types", "/administration/node-types", "Node Types"),
            ]
            .into_iter()
            .map(|(key, href, label)| navigation::ResolvedNavigationItem {
                key: key.to_string(),
                href: href.to_string(),
                label: label.to_string(),
                section: navigation::NavigationSection::Admin,
                owner: navigation::NavigationItemOwner::Core,
                contribution_id: None,
            }),
        );
    }

    if items.is_empty() {
        return empty_view();
    }

    view! {
        <p class="sidebar-section">{section}</p>
        {items
            .into_iter()
            .map(move |item| {
                resolved_nav_item_link(item, active_route)
            })
            .collect_view()}
    }
    .into_any()
}

fn resolved_nav_item_link(
    item: navigation::ResolvedNavigationItem,
    active_route: &'static str,
) -> impl IntoView {
    let class = if item.key == active_route {
        "sidebar-link is-active"
    } else {
        "sidebar-link"
    };
    let icon = nav_icon_for(&item.key);
    let label = item.label;
    let title = label.clone();
    let aria_label = label.clone();
    view! {
        <a class=class href=item.href title=title aria-label=aria_label>
            {icon}
            <span class="sidebar-link__label">{label}</span>
        </a>
    }
}

fn nav_icon_for(route_key: &str) -> impl IntoView + use<> {
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
        "user_management" => view! { <span class="sidebar-link__icon-wrap" aria-hidden="true"><Users class="sidebar-link__icon"/></span> }.into_any(),
        "roles_access" => view! { <span class="sidebar-link__icon-wrap" aria-hidden="true"><ShieldCheck class="sidebar-link__icon"/></span> }.into_any(),
        "node_types" => view! { <span class="sidebar-link__icon-wrap" aria-hidden="true"><Network class="sidebar-link__icon"/></span> }.into_any(),
        "module_management" => view! { <span class="sidebar-link__icon-wrap" aria-hidden="true"><Blocks class="sidebar-link__icon"/></span> }.into_any(),
        _ => view! { <span class="sidebar-link__icon-wrap" aria-hidden="true"><File class="sidebar-link__icon"/></span> }.into_any(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn account(capabilities: &[&str], global_capabilities: &[&str]) -> auth::ShellAccountSummary {
        auth::ShellAccountSummary {
            email: "reader@tessara.local".into(),
            display_name: "Module Reader".into(),
            capabilities: capabilities
                .iter()
                .map(|capability| (*capability).to_string())
                .collect(),
            global_capabilities: global_capabilities
                .iter()
                .map(|capability| (*capability).to_string())
                .collect(),
        }
    }

    #[test]
    fn loading_and_failed_fallbacks_do_not_treat_scope_opaque_module_keys_as_global() {
        for unavailable in [false, true] {
            let html = Owner::new().with(|| {
                fallback_navigation(
                    "module_management",
                    Some(account(
                        &[
                            "modules:read",
                            "modules:manage_navigation",
                            "forms:read",
                            "datasets:read",
                        ],
                        &[],
                    )),
                    unavailable,
                )
                .to_html()
            });

            assert!(!html.contains("Module Management"));
            assert!(!html.contains(">Forms<"));
            assert!(!html.contains(">Datasets<"));
            assert!(!html.contains(">Administration<"));
            assert_eq!(
                html.contains("Contribution navigation is temporarily unavailable."),
                unavailable
            );
        }
    }

    #[test]
    fn loading_and_failed_fallbacks_retain_globally_proven_read_and_manage() {
        for capability in ["modules:read", "modules:manage_navigation"] {
            for unavailable in [false, true] {
                let html = Owner::new().with(|| {
                    fallback_navigation(
                        "module_management",
                        Some(account(&[capability], &[capability])),
                        unavailable,
                    )
                    .to_html()
                });

                assert!(html.contains("Module Management"), "{capability}");
                assert!(!html.contains(">Administration<"), "{capability}");
                assert_eq!(
                    html.contains("Contribution navigation is temporarily unavailable."),
                    unavailable
                );
            }
        }
    }

    #[test]
    fn failed_fallback_retains_module_management_for_the_global_admin_sentinel() {
        let html = Owner::new().with(|| {
            fallback_navigation(
                "module_management",
                Some(account(&["admin:all"], &["admin:all"])),
                true,
            )
            .to_html()
        });

        assert!(html.contains("Module Management"));
        assert!(html.contains("User Management"));
        assert!(html.contains("Roles &amp; Access"));
        assert!(html.contains("Node Types"));
        assert!(!html.contains("href=\"/administration\""));
        assert!(html.contains("Contribution navigation is temporarily unavailable."));
    }

    #[test]
    fn failed_fallback_matches_the_core_administrator_floor() {
        let html = Owner::new().with(|| {
            fallback_navigation(
                "module_management",
                Some(account(&["core:admin"], &["core:admin"])),
                true,
            )
            .to_html()
        });

        assert!(html.contains(">Organization<"));
        assert!(html.contains("Module Management"));
        assert!(html.contains("User Management"));
        assert!(html.contains("Roles &amp; Access"));
        assert!(html.contains("Node Types"));
        assert!(!html.contains(">Forms<"));
        assert!(!html.contains(">Workflows<"));
    }

    #[test]
    fn module_management_uses_the_canonical_blocks_icon() {
        let html = Owner::new().with(|| nav_icon_for("module_management").to_html());

        assert!(html.contains("M10 22V7a1 1 0 0 0-1-1H4a2 2 0 0 0-2 2v12a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2v-5a1 1 0 0 0-1-1H2"));
        assert!(html.contains("x=\"14\""));
        assert!(html.contains("y=\"2\""));
    }

    #[test]
    fn independently_deployed_module_navigation_stays_inside_the_core_shell() {
        let html = Owner::new().with(|| {
            projected_nav_item_link(
                ShellNavigationItemV1 {
                    key: "scoped_records".into(),
                    label: "Scoped Records".into(),
                    href: "/reference/scoped-records".into(),
                    owner: crate::state::shell_navigation::ShellNavigationItemOwnerV1::Contribution,
                    contribution_id: Some("tessara.reference.scoped-records.directory".into()),
                },
                "home",
            )
            .to_html()
        });

        assert!(html.contains("href=\"/reference/scoped-records\""));
        assert!(!html.contains("rel=\"external\""));
    }
}
