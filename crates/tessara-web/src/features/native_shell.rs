#![cfg_attr(not(feature = "hydrate"), allow(dead_code, unused_imports))]

use leptos::prelude::*;
use leptos_router::{
    NavigateOptions,
    hooks::{use_location, use_navigate},
};
use serde::Deserialize;

use crate::features::native_runtime::set_page_context;
#[cfg(feature = "hydrate")]
use crate::features::native_runtime::{delete_json, get_json, redirect};

#[cfg(feature = "hydrate")]
use std::cell::Cell;
#[cfg(feature = "hydrate")]
use wasm_bindgen::JsCast;
#[cfg(feature = "hydrate")]
use wasm_bindgen_futures::spawn_local;
#[cfg(feature = "hydrate")]
use web_sys::{KeyboardEvent, window};

#[cfg(not(feature = "hydrate"))]
fn redirect(_path: &str) {}

#[cfg(feature = "hydrate")]
std::thread_local! {
    static SHELL_CHROME_BOUND: Cell<bool> = const { Cell::new(false) };
    static TABLET_SIDEBAR_EXPANDED: Cell<bool> = const { Cell::new(false) };
    static MOBILE_SIDEBAR_OPEN: Cell<bool> = const { Cell::new(false) };
}

#[derive(Clone, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum UiAccessProfile {
    Admin,
    Operator,
    ResponseUser,
}

#[derive(Clone, Deserialize)]
pub struct DelegationSummary {
    pub account_id: String,
    pub display_name: String,
    pub email: String,
}

#[derive(Clone, Deserialize)]
pub struct ScopeNodeSummary {
    pub node_id: String,
    pub node_name: String,
    pub node_type_name: String,
    pub parent_node_id: Option<String>,
    pub parent_node_name: Option<String>,
}

#[derive(Clone, Deserialize)]
pub struct AccountContext {
    pub account_id: String,
    pub email: String,
    pub display_name: String,
    pub ui_access_profile: UiAccessProfile,
    pub capabilities: Vec<String>,
    pub scope_nodes: Vec<ScopeNodeSummary>,
    pub delegations: Vec<DelegationSummary>,
}

#[derive(Clone, Deserialize)]
struct SessionStateResponse {
    authenticated: bool,
    account: Option<AccountContext>,
}

#[derive(Clone, Copy)]
pub struct AccountSession {
    pub loaded: RwSignal<bool>,
    pub account: RwSignal<Option<AccountContext>>,
    pub error: RwSignal<Option<String>>,
}

pub fn use_account_session() -> AccountSession {
    if let Some(session) = use_context::<AccountSession>() {
        return session;
    }

    let session = AccountSession {
        loaded: RwSignal::new(false),
        account: RwSignal::new(None),
        error: RwSignal::new(None),
    };

    provide_context(session);
    session
}

#[derive(Clone)]
pub struct BreadcrumbItem {
    pub href: Option<String>,
    pub label: String,
}

impl BreadcrumbItem {
    pub fn current(label: impl Into<String>) -> Self {
        Self {
            href: None,
            label: label.into(),
        }
    }

    pub fn link(label: impl Into<String>, href: impl Into<String>) -> Self {
        Self {
            href: Some(href.into()),
            label: label.into(),
        }
    }
}

#[derive(Clone, Copy)]
pub(crate) struct NavLinkSpec {
    pub href: &'static str,
    pub icon: &'static str,
    pub key: &'static str,
    pub label: &'static str,
    pub native: bool,
    pub required_capability: Option<&'static str>,
    pub home_description: &'static str,
    pub home_action_label: &'static str,
}

pub(crate) const PRODUCT_LINKS: &[NavLinkSpec] = &[
    NavLinkSpec {
        key: "home",
        href: "/app",
        label: "Home",
        icon: "fa-house",
        native: true,
        required_capability: None,
        home_description: "Return to the shared application overview.",
        home_action_label: "Go Home",
    },
    NavLinkSpec {
        key: "organization",
        href: "/app/organization",
        label: "Organization",
        icon: "fa-sitemap",
        native: true,
        required_capability: Some("hierarchy:read"),
        home_description: "Browse runtime organization records and move into related forms and responses.",
        home_action_label: "Go to Organization",
    },
    NavLinkSpec {
        key: "forms",
        href: "/app/forms",
        label: "Forms",
        icon: "fa-clipboard-list",
        native: true,
        required_capability: Some("forms:read"),
        home_description: "Browse forms, inspect lifecycle state, and review workflow attachments.",
        home_action_label: "Go to Forms",
    },
    NavLinkSpec {
        key: "workflows",
        href: "/app/workflows",
        label: "Workflows",
        icon: "fa-diagram-project",
        native: true,
        required_capability: Some("workflows:read"),
        home_description: "Create, publish, and assign form-backed workflows from the native runtime shell.",
        home_action_label: "Go to Workflows",
    },
    NavLinkSpec {
        key: "responses",
        href: "/app/responses",
        label: "Responses",
        icon: "fa-square-check",
        native: true,
        required_capability: Some("submissions:write"),
        home_description: "Start response work, resume drafts, and review submitted responses.",
        home_action_label: "Go to Responses",
    },
    NavLinkSpec {
        key: "dashboards",
        href: "/app/dashboards",
        label: "Dashboards",
        icon: "fa-table-columns",
        native: true,
        required_capability: Some("reports:read"),
        home_description: "Browse dashboards and previews from the shared application shell.",
        home_action_label: "Open Dashboards",
    },
];

pub(crate) const ANALYTICS_LINKS: &[NavLinkSpec] = &[
    NavLinkSpec {
        key: "datasets",
        href: "/app/datasets",
        label: "Datasets",
        icon: "fa-database",
        native: false,
        required_capability: Some("datasets:read"),
        home_description: "Review dataset definitions, source structure, and current read-ready status.",
        home_action_label: "Open Datasets",
    },
    NavLinkSpec {
        key: "components",
        href: "/app/components",
        label: "Components",
        icon: "fa-cubes",
        native: false,
        required_capability: Some("reports:read"),
        home_description: "Inspect dashboard component definitions and their linked charts.",
        home_action_label: "Open Components",
    },
];

pub(crate) const INTERNAL_LINKS: &[NavLinkSpec] = &[
    NavLinkSpec {
        key: "administration",
        href: "/app/administration",
        label: "Administration",
        icon: "fa-sliders",
        native: true,
        required_capability: Some("admin:all"),
        home_description: "Internal configuration and legacy builder access stay here.",
        home_action_label: "Open Administration",
    },
    NavLinkSpec {
        key: "migration",
        href: "/app/migration",
        label: "Migration",
        icon: "fa-truck-fast",
        native: true,
        required_capability: Some("admin:all"),
        home_description: "Validate and import representative legacy fixtures from the operator workbench.",
        home_action_label: "Open Migration",
    },
];

pub(crate) const TRANSITIONAL_LINKS: &[NavLinkSpec] = &[NavLinkSpec {
    key: "reports",
    href: "/app/reports",
    label: "Reports",
    icon: "fa-chart-line",
    native: false,
    required_capability: Some("reports:read"),
    home_description: "Browse, inspect, and run the transitional report surfaces.",
    home_action_label: "Open Reports",
}];

pub fn has_capability(account: Option<&AccountContext>, capability: &str) -> bool {
    account.is_some_and(|account| {
        account
            .capabilities
            .iter()
            .any(|item| item == "admin:all" || item == capability)
    })
}

pub(crate) fn route_visible(
    loaded: bool,
    account: Option<&AccountContext>,
    required_capability: Option<&'static str>,
) -> bool {
    match required_capability {
        None => true,
        Some(_) if !loaded => false,
        Some(_) if account.is_none() => false,
        Some(capability) => has_capability(account, capability),
    }
}

pub(crate) fn visible_links<'a>(
    loaded: bool,
    account: Option<&AccountContext>,
    links: &'a [NavLinkSpec],
) -> Vec<&'a NavLinkSpec> {
    links
        .iter()
        .filter(|link| route_visible(loaded, account, link.required_capability))
        .collect()
}

#[cfg(feature = "hydrate")]
fn shell_viewport() -> &'static str {
    let width = window()
        .and_then(|window| window.inner_width().ok())
        .and_then(|width| width.as_f64())
        .unwrap_or(1280.0);

    if width < 768.0 {
        "mobile"
    } else if width < 1024.0 {
        "tablet"
    } else {
        "desktop"
    }
}

#[cfg(feature = "hydrate")]
fn shell_sidebar_state(viewport: &str) -> &'static str {
    match viewport {
        "mobile" => MOBILE_SIDEBAR_OPEN.with(|open| {
            if open.get() {
                "overlay-open"
            } else {
                "overlay-closed"
            }
        }),
        "tablet" => TABLET_SIDEBAR_EXPANDED.with(|expanded| {
            if expanded.get() {
                "expanded"
            } else {
                "collapsed"
            }
        }),
        _ => "expanded",
    }
}

#[cfg(feature = "hydrate")]
fn apply_shell_chrome_state() {
    let Some(document) = window().and_then(|window| window.document()) else {
        return;
    };
    let Some(root) = document.document_element() else {
        return;
    };
    let Some(body) = document.body() else {
        return;
    };

    let viewport = shell_viewport();
    let state = shell_sidebar_state(viewport);

    root.set_attribute("data-shell-ready", "true").ok();
    body.set_attribute("data-shell-viewport", viewport).ok();
    body.set_attribute("data-sidebar-state", state).ok();

    if let Some(sidebar) = document.get_element_by_id("app-sidebar") {
        sidebar.set_attribute("data-sidebar-state", state).ok();
        sidebar
            .set_attribute(
                "aria-hidden",
                if viewport == "mobile" && state == "overlay-closed" {
                    "true"
                } else {
                    "false"
                },
            )
            .ok();
    }

    if let Some(toggle) = document.get_element_by_id("app-nav-toggle") {
        toggle
            .set_attribute(
                "aria-expanded",
                if state == "overlay-open" || state == "expanded" {
                    "true"
                } else {
                    "false"
                },
            )
            .ok();
        let label = if viewport == "mobile" {
            if state == "overlay-open" {
                "Close navigation"
            } else {
                "Open navigation"
            }
        } else if state == "collapsed" {
            "Expand navigation"
        } else {
            "Collapse navigation"
        };
        toggle.set_attribute("aria-label", label).ok();
    }
}

#[cfg(feature = "hydrate")]
fn toggle_shell_sidebar() {
    match shell_viewport() {
        "mobile" => MOBILE_SIDEBAR_OPEN.with(|open| open.set(!open.get())),
        "tablet" => TABLET_SIDEBAR_EXPANDED.with(|expanded| expanded.set(!expanded.get())),
        _ => {}
    }
    apply_shell_chrome_state();
}

#[cfg(feature = "hydrate")]
fn close_shell_sidebar() {
    match shell_viewport() {
        "mobile" => MOBILE_SIDEBAR_OPEN.with(|open| open.set(false)),
        "tablet" => TABLET_SIDEBAR_EXPANDED.with(|expanded| expanded.set(false)),
        _ => {}
    }
    apply_shell_chrome_state();
}

#[cfg(feature = "hydrate")]
fn install_shell_chrome() {
    let already_bound = SHELL_CHROME_BOUND.with(|bound| {
        let was_bound = bound.get();
        if !was_bound {
            bound.set(true);
        }
        was_bound
    });

    if already_bound {
        apply_shell_chrome_state();
        return;
    }

    let Some(document) = window().and_then(|window| window.document()) else {
        return;
    };

    if let Some(toggle) = document.get_element_by_id("app-nav-toggle") {
        let closure =
            wasm_bindgen::closure::Closure::wrap(Box::new(move |_event: web_sys::Event| {
                toggle_shell_sidebar();
            }) as Box<dyn FnMut(_)>);
        let _ = toggle.add_event_listener_with_callback("click", closure.as_ref().unchecked_ref());
        closure.forget();
    }

    let dismiss_buttons = document.query_selector_all("[data-sidebar-dismiss]").ok();
    if let Some(buttons) = dismiss_buttons {
        for index in 0..buttons.length() {
            if let Some(button) = buttons.item(index) {
                let closure =
                    wasm_bindgen::closure::Closure::wrap(Box::new(move |_event: web_sys::Event| {
                        close_shell_sidebar();
                    })
                        as Box<dyn FnMut(_)>);
                let _ = button
                    .add_event_listener_with_callback("click", closure.as_ref().unchecked_ref());
                closure.forget();
            }
        }
    }

    if let Some(window) = window() {
        let resize = wasm_bindgen::closure::Closure::wrap(Box::new(move |_event: web_sys::Event| {
            apply_shell_chrome_state();
        }) as Box<dyn FnMut(_)>);
        let _ = window.add_event_listener_with_callback("resize", resize.as_ref().unchecked_ref());
        resize.forget();
    }

    let keydown = wasm_bindgen::closure::Closure::wrap(Box::new(move |event: KeyboardEvent| {
        if event.key() == "Escape" {
            close_shell_sidebar();
        }
    }) as Box<dyn FnMut(_)>);
    let _ = document.add_event_listener_with_callback("keydown", keydown.as_ref().unchecked_ref());
    keydown.forget();

    apply_shell_chrome_state();
}

#[cfg(feature = "hydrate")]
#[component]
fn ThemeToggle() -> impl IntoView {
    use crate::theme::{DARK_THEME_COLOR, LIGHT_THEME_COLOR, STORAGE_KEY};
    use web_sys::window;

    let preference = RwSignal::new(String::from("system"));

    let apply_preference = move |choice: &'static str| {
        let Some(window) = window() else {
            return;
        };
        let Some(document) = window.document() else {
            return;
        };
        let Some(root) = document.document_element() else {
            return;
        };
        let theme = if choice == "system" {
            match window.match_media("(prefers-color-scheme: dark)") {
                Ok(Some(query)) if query.matches() => "dark",
                _ => "light",
            }
        } else {
            choice
        };
        root.set_attribute("data-theme-preference", choice).ok();
        root.set_attribute("data-theme", theme).ok();
        if let Ok(Some(meta)) = document.query_selector("meta[name=\"theme-color\"]") {
            meta.set_attribute(
                "content",
                if theme == "dark" {
                    DARK_THEME_COLOR
                } else {
                    LIGHT_THEME_COLOR
                },
            )
            .ok();
        }
        if let Ok(Some(storage)) = window.local_storage() {
            let _ = storage.set_item(STORAGE_KEY, choice);
        }
        preference.set(choice.to_string());
    };

    Effect::new(move |_| {
        let Some(window) = window() else {
            return;
        };
        let Some(document) = window.document() else {
            return;
        };
        let choice = document
            .document_element()
            .and_then(|root| root.get_attribute("data-theme-preference"))
            .unwrap_or_else(|| "system".into());
        preference.set(choice);
    });

    let button = move |choice: &'static str, label: &'static str| {
        let apply_preference = apply_preference.clone();
        view! {
            <button
                class="theme-toggle-button"
                type="button"
                aria-pressed=move || if preference.get() == choice { "true" } else { "false" }
                data-active=move || if preference.get() == choice { "true" } else { "false" }
                on:click=move |_| apply_preference(choice)
            >
                {label}
            </button>
        }
    };

    view! {
        <div class="theme-toggle" role="group" aria-label="Theme preference">
            {button("system", "System")}
            {button("light", "Light")}
            {button("dark", "Dark")}
        </div>
    }
}

#[cfg(not(feature = "hydrate"))]
#[component]
fn ThemeToggle() -> impl IntoView {
    view! {
        <div class="theme-toggle" role="group" aria-label="Theme preference">
            <button class="theme-toggle-button" type="button" aria-pressed="true">"System"</button>
            <button class="theme-toggle-button" type="button" aria-pressed="false">"Light"</button>
            <button class="theme-toggle-button" type="button" aria-pressed="false">"Dark"</button>
        </div>
    }
}

#[component]
fn ShellLink(spec: &'static NavLinkSpec, active_route: &'static str) -> impl IntoView {
    let class_name = if spec.key == active_route {
        "active"
    } else {
        ""
    };

    if spec.native {
        let navigate = use_navigate();
        let href = spec.href.to_string();
        view! {
            <li>
                <a
                    class=class_name
                    href=href.clone()
                    title=spec.label
                    aria-label=spec.label
                    on:click=move |event| {
                        event.prevent_default();
                        navigate(&href, NavigateOptions::default());
                    }
                >
                    <span class="app-nav__icon" aria-hidden="true">
                        <i class=format!("fa-solid {}", spec.icon)></i>
                    </span>
                    <span class="app-nav__label">{spec.label}</span>
                </a>
            </li>
        }
        .into_any()
    } else {
        let href = spec.href.to_string();
        view! {
            <li>
                <a
                    class=class_name
                    href=href.clone()
                    title=spec.label
                    aria-label=spec.label
                    on:click=move |event| {
                        event.prevent_default();
                        redirect(&href);
                    }
                >
                    <span class="app-nav__icon" aria-hidden="true">
                        <i class=format!("fa-solid {}", spec.icon)></i>
                    </span>
                    <span class="app-nav__label">{spec.label}</span>
                </a>
            </li>
        }
        .into_any()
    }
}

#[component]
fn NavSection(
    heading: &'static str,
    aria_label: &'static str,
    links: &'static [NavLinkSpec],
    active_route: &'static str,
    loaded: bool,
    account: Option<AccountContext>,
    section_class: &'static str,
) -> impl IntoView {
    let visible_links = visible_links(loaded, account.as_ref(), links);

    if visible_links.is_empty() {
        return view! { <></> }.into_any();
    }

    view! {
        <section class=format!("nav-panel menu {section_class}")>
            <p class="menu-label">{heading}</p>
            <nav aria-label=aria_label>
                <ul class="menu-list app-nav">
                    {visible_links
                        .into_iter()
                        .map(|link| view! { <ShellLink spec=link active_route=active_route/> })
                        .collect_view()}
                </ul>
            </nav>
        </section>
    }
    .into_any()
}

#[component]
fn SessionSummary(account: Option<AccountContext>, error: Option<String>) -> impl IntoView {
    let status = match account.as_ref() {
        Some(account) => format!(
            "Signed in as {} ({}).",
            account.email,
            match account.ui_access_profile {
                UiAccessProfile::Admin => "admin",
                UiAccessProfile::Operator => "operator",
                UiAccessProfile::ResponseUser => "response user",
            }
        ),
        None if error.is_some() => "Session load failed.".to_string(),
        None => "Not signed in.".to_string(),
    };

    view! {
        <section class="selection-panel app-sidebar__supplemental">
            <h3>"Session And Selections"</h3>
            <p class="muted">"The current signed-in account and your selected records appear here."</p>
            <p id="session-status" class="muted">{status}</p>
            <div id="current-user-summary" class="selection-grid">
                {match account {
                    Some(account) => view! {
                        <article class="selection-item box">
                            <h3>{account.display_name}</h3>
                            <p>{account.email}</p>
                            <p class="muted">{format!("Delegations: {}", account.delegations.len())}</p>
                        </article>
                    }
                    .into_any(),
                    None => view! { <p class="muted">"Sign in to load account context."</p> }.into_any(),
                }}
            </div>
            <div id="selection-state" class="selection-grid">
                <p class="muted">"No records selected yet."</p>
            </div>
            {error
                .map(|error| view! { <p class="muted">{error}</p> }.into_any())
                .unwrap_or_else(|| view! { <></> }.into_any())}
        </section>
    }
}

#[derive(Clone, Deserialize)]
struct LogoutResponse {
    signed_out: bool,
}

#[cfg_attr(not(feature = "hydrate"), allow(unused_variables))]
#[component]
fn SignOutButton(session: AccountSession) -> impl IntoView {
    let busy = RwSignal::new(false);
    let feedback = RwSignal::new(None::<String>);

    let sign_out = move |_| {
        if busy.get_untracked() {
            return;
        }

        busy.set(true);
        feedback.set(None);

        #[cfg(feature = "hydrate")]
        {
            let session = session;
            spawn_local(async move {
                let result = delete_json::<LogoutResponse>("/api/auth/logout").await;

                session.account.set(None);
                session.error.set(None);
                session.loaded.set(true);

                match result {
                    Ok(response) if response.signed_out => redirect("/app/login"),
                    Ok(_) => {
                        feedback.set(Some("Signed out locally. Redirecting to sign-in.".into()));
                        redirect("/app/login");
                    }
                    Err(_) => {
                        feedback.set(Some("Signed out locally. Redirecting to sign-in.".into()));
                        redirect("/app/login");
                    }
                }

                busy.set(false);
            });
        }
    };

    view! {
        <div class="app-session-actions">
            <button
                class="button is-light"
                type="button"
                on:click=sign_out
                disabled=move || busy.get()
            >
                {move || if busy.get() { "Signing Out..." } else { "Sign Out" }}
            </button>
            <Show when=move || feedback.get().is_some()>
                <p class="muted">{move || feedback.get().unwrap_or_default()}</p>
            </Show>
        </div>
    }
}

#[component]
fn BreadcrumbTrail(items: Vec<BreadcrumbItem>) -> impl IntoView {
    if items.len() <= 2 {
        return view! { <nav class="breadcrumb-trail" aria-label="Breadcrumb"></nav> }.into_any();
    }

    let navigate = use_navigate();

    view! {
        <nav class="breadcrumb-trail" aria-label="Breadcrumb">
            {items
                .into_iter()
                .map(|item| {
                    let label = item.label;
                    match item.href {
                        Some(href) if is_native_href(&href) =>
                        {
                            let href_clone = href.clone();
                            let navigate = navigate.clone();
                            view! {
                                <span class="breadcrumb-item">
                                    <a href=href_clone.clone() on:click=move |event| {
                                        event.prevent_default();
                                        navigate(&href_clone, NavigateOptions::default());
                                    }>{label}</a>
                                </span>
                            }
                            .into_any()
                        }
                        Some(href) => view! {
                            <span class="breadcrumb-item">
                                <a href=href.clone() on:click=move |event| {
                                    event.prevent_default();
                                    redirect(&href);
                                }>{label}</a>
                            </span>
                        }
                        .into_any(),
                        None => view! {
                            <span class="breadcrumb-item">
                                <span>{label}</span>
                            </span>
                        }
                        .into_any(),
                    }
                })
                .collect_view()}
        </nav>
    }
    .into_any()
}

fn is_native_href(href: &str) -> bool {
    href == "/app"
        || href == "/app/organization"
        || href.starts_with("/app/organization/")
        || href == "/app/forms"
        || href.starts_with("/app/forms/")
        || href == "/app/workflows"
        || href.starts_with("/app/workflows/")
        || href == "/app/responses"
        || href.starts_with("/app/responses/")
        || href == "/app/dashboards"
        || href.starts_with("/app/dashboards/")
        || href == "/app/administration"
        || href.starts_with("/app/administration/")
        || href == "/app/admin"
        || href == "/app/migration"
}

#[component]
pub fn PageHeader(
    eyebrow: &'static str,
    title: &'static str,
    description: &'static str,
    #[prop(optional)] actions: Option<ChildrenFn>,
) -> impl IntoView {
    view! {
        <section class="app-screen box entity-page ui-page-header">
            <p class="eyebrow ui-page-header__eyebrow">{eyebrow}</p>
            <div class="page-title-row ui-page-header__row">
                <div class="ui-page-header__copy">
                    <h1>{title}</h1>
                    <p class="muted ui-page-header__description">{description}</p>
                </div>
                <div class="actions ui-action-group">
                    {actions
                        .map(|children| children())
                        .unwrap_or_else(|| view! { <></> }.into_any())}
                </div>
            </div>
        </section>
    }
}

#[component]
pub fn Panel(
    title: impl Into<String>,
    description: impl Into<String>,
    children: ChildrenFn,
) -> impl IntoView {
    let title = title.into();
    let description = description.into();

    view! {
        <section class="app-screen box page-panel ui-panel">
            <h3 class="ui-panel__title">{title}</h3>
            <p class="muted ui-panel__description">{description}</p>
            {children()}
        </section>
    }
}

#[component]
pub fn MetadataStrip(items: Vec<(&'static str, String)>) -> impl IntoView {
    view! {
        <div class="ui-metadata-strip">
            {items
                .into_iter()
                .map(|(label, value)| {
                    view! {
                        <div class="ui-metadata-strip__item">
                            <span class="ui-metadata-strip__label">{label}</span>
                            <strong class="ui-metadata-strip__value">{value}</strong>
                        </div>
                    }
                })
                .collect_view()}
        </div>
    }
}

#[cfg_attr(not(feature = "hydrate"), allow(unused_variables))]
#[component]
pub fn NativePage(
    title: &'static str,
    description: &'static str,
    page_key: &'static str,
    active_route: &'static str,
    workspace_label: &'static str,
    #[prop(optional)] record_id: Option<String>,
    #[prop(optional)] required_capability: Option<&'static str>,
    #[prop(optional)] allow_unauthenticated: bool,
    #[prop(optional)] breadcrumbs: Vec<BreadcrumbItem>,
    children: ChildrenFn,
) -> impl IntoView {
    let session = use_account_session();
    let location = use_location();
    let _ = title;
    let _ = description;

    set_page_context(page_key, active_route, record_id);

    #[cfg(feature = "hydrate")]
    Effect::new(move |_| {
        install_shell_chrome();
        let _ = location.pathname.get();
        let _ = location.search.read();

        let session = session;
        spawn_local(async move {
            match get_json::<SessionStateResponse>("/api/auth/session").await {
                Ok(response) => {
                    session.account.set(response.account);
                    session.error.set(None);
                    session.loaded.set(true);
                    if !allow_unauthenticated && !response.authenticated {
                        redirect("/app/login");
                    }
                }
                Err(error) => {
                    session.account.set(None);
                    session.error.set(Some(error));
                    session.loaded.set(true);
                }
            }
        });
    });

    view! {
        <main class=format!("shell app-shell app-shell--{active_route}")>
            <header class="top-app-bar">
                <div class="top-app-bar__brand">
                    <button id="app-nav-toggle" class="app-nav-toggle" type="button" aria-label="Toggle navigation" aria-controls="app-sidebar" aria-expanded="false" hidden>
                        <span class="app-nav-toggle__icon" aria-hidden="true"><i class="fa-solid fa-bars"></i></span>
                    </button>
                    <a class="top-app-home-link" href="/app">
                        <img class="top-app-bar__mark" src="/assets/tessara-icon-256.svg" alt="" />
                        <span class="top-app-bar__name">"Tessara"</span>
                    </a>
                    <span class="top-app-bar__context">{workspace_label}</span>
                </div>
                <div class="top-app-bar__utilities">
                    <div class="top-app-bar__search">
                        <label class="is-sr-only" for="global-search">"Global search"</label>
                        <input id="global-search" class="input app-search-input" type="search" placeholder="Search Tessara" autocomplete="off" />
                    </div>
                    <Show when=move || session.account.get().is_some()>
                        <SignOutButton session=session/>
                    </Show>
                    <ThemeToggle/>
                </div>
            </header>
            <button class="app-sidebar-backdrop" type="button" aria-label="Close navigation" data-sidebar-dismiss tabindex="-1" hidden></button>
            <section class="app-layout">
                <aside id="app-sidebar" class="panel box app-sidebar" aria-label="Application navigation">
                    {move || {
                        let loaded = session.loaded.get();
                        let account = session.account.get();
                        let error = session.error.get();

                        view! {
                            <div class="app-sidebar__content">
                                <NavSection
                                    heading="Product Areas"
                                    aria_label="Product navigation"
                                    links=PRODUCT_LINKS
                                    active_route=active_route
                                    loaded=loaded
                                    account=account.clone()
                                    section_class="nav-panel-primary"
                                />
                                <NavSection
                                    heading="Data And Components"
                                    aria_label="Data and component navigation"
                                    links=ANALYTICS_LINKS
                                    active_route=active_route
                                    loaded=loaded
                                    account=account.clone()
                                    section_class="nav-panel-analytics"
                                />
                                <NavSection
                                    heading="Transitional Analytics"
                                    aria_label="Transitional reporting navigation"
                                    links=TRANSITIONAL_LINKS
                                    active_route=active_route
                                    loaded=loaded
                                    account=account.clone()
                                    section_class="nav-panel-transitional"
                                />
                                <NavSection
                                    heading="Internal Areas"
                                    aria_label="Internal navigation"
                                    links=INTERNAL_LINKS
                                    active_route=active_route
                                    loaded=loaded
                                    account=account.clone()
                                    section_class="nav-panel-secondary"
                                />
                                <SessionSummary account=account error=error/>
                            </div>
                        }
                    }}
                </aside>
                <section class="panel box app-main">
                    <BreadcrumbTrail items=breadcrumbs/>
                    {move || {
                        let account = session.account.get();
                        if !allow_unauthenticated && !session.loaded.get() {
                            return view! {
                                <Panel title="Loading Session" description="Confirming the current browser session before this screen loads.">
                                    <p class="muted">"Loading session state..."</p>
                                </Panel>
                            }
                            .into_any();
                        }
                        if !allow_unauthenticated && session.loaded.get() && account.is_none() {
                            return view! {
                                <Panel title="Redirecting To Sign In" description="This route requires an authenticated browser session.">
                                    <div class="actions">
                                        <a class="button-link button is-light" href="/app/login">"Open Sign In"</a>
                                    </div>
                                </Panel>
                            }
                            .into_any();
                        }
                        if let Some(required_capability) = required_capability {
                            if account.is_some()
                                && session.loaded.get()
                                && !has_capability(account.as_ref(), required_capability)
                            {
                                return view! {
                                    <Panel title="Access Restricted" description="Your current role does not have access to this screen.">
                                        <div class="actions">
                                            <a class="button-link button is-light" href="/app/login">"Sign In"</a>
                                            <a class="button-link button is-light" href="/app">"Go Home"</a>
                                        </div>
                                    </Panel>
                                }
                                .into_any();
                            }
                        }
                        children().into_any()
                    }}
                </section>
            </section>
            <pre id="output" hidden></pre>
        </main>
    }
}
