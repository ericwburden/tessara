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
use web_sys::{KeyboardEvent, UrlSearchParams, window};

#[cfg(not(feature = "hydrate"))]
fn redirect(_path: &str) {}

#[cfg(feature = "hydrate")]
std::thread_local! {
    static SHELL_CHROME_BOUND: Cell<bool> = const { Cell::new(false) };
    static TABLET_SIDEBAR_EXPANDED: Cell<bool> = const { Cell::new(false) };
    static TABLET_PROFILE_OPEN: Cell<bool> = const { Cell::new(false) };
    static MOBILE_SIDEBAR_OPEN: Cell<bool> = const { Cell::new(false) };
    static MOBILE_SEARCH_OPEN: Cell<bool> = const { Cell::new(false) };
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
        key: "components",
        href: "/app/components",
        label: "Components",
        icon: "fa-cubes",
        native: true,
        required_capability: Some("reports:read"),
        home_description: "Inspect dashboard component definitions and their linked charts.",
        home_action_label: "Open Components",
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

pub(crate) const ADMIN_LINKS: &[NavLinkSpec] = &[
    NavLinkSpec {
        key: "datasets",
        href: "/app/datasets",
        label: "Datasets",
        icon: "fa-database",
        native: true,
        required_capability: Some("datasets:read"),
        home_description: "Review dataset definitions, source structure, and current read-ready status.",
        home_action_label: "Open Datasets",
    },
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

#[derive(Clone, Copy, PartialEq, Eq)]
enum ShellNotice {
    AccessDenied,
}

impl ShellNotice {
    fn from_key(key: &str) -> Option<Self> {
        match key {
            "access-denied" => Some(Self::AccessDenied),
            _ => None,
        }
    }

    fn title(self) -> &'static str {
        match self {
            Self::AccessDenied => "Access limited",
        }
    }

    fn message(self) -> &'static str {
        match self {
            Self::AccessDenied => {
                "You do not have access to that screen. Tessara returned you to Home."
            }
        }
    }
}

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

fn query_value(search: &str, key: &str) -> Option<String> {
    search
        .trim_start_matches('?')
        .split('&')
        .filter(|segment| !segment.is_empty())
        .find_map(|segment| {
            let mut parts = segment.splitn(2, '=');
            let name = parts.next()?;
            let value = parts.next().unwrap_or_default();
            (name == key && !value.is_empty()).then(|| value.to_string())
        })
}

fn preferred_account_label(display_name: &str, email: &str) -> String {
    let trimmed = display_name.trim();
    if trimmed.is_empty() {
        email.to_string()
    } else {
        trimmed.to_string()
    }
}

fn ui_profile_label(profile: UiAccessProfile) -> &'static str {
    match profile {
        UiAccessProfile::Admin => "Administrator",
        UiAccessProfile::Operator => "Scoped operator",
        UiAccessProfile::ResponseUser => "Response user",
    }
}

fn active_delegate(account: &AccountContext, search: &str) -> Option<DelegationSummary> {
    let delegate_account_id = query_value(search, "delegateAccountId")?;
    account
        .delegations
        .iter()
        .find(|delegate| delegate.account_id == delegate_account_id)
        .cloned()
}

fn account_initials(label: &str) -> String {
    let initials = label
        .split_whitespace()
        .filter_map(|part| part.chars().find(|character| character.is_alphanumeric()))
        .take(2)
        .collect::<String>();

    if initials.is_empty() {
        "TS".into()
    } else {
        initials.to_uppercase()
    }
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
fn read_shell_notice() -> Option<ShellNotice> {
    let location = window()?.location();
    let search = location.search().ok()?;
    let params = UrlSearchParams::new_with_str(&search).ok()?;
    params
        .get("notice")
        .and_then(|value| ShellNotice::from_key(&value))
}

#[cfg(feature = "hydrate")]
fn clear_shell_notice_query() {
    let Some(window) = window() else {
        return;
    };
    let location = window.location();
    let pathname = location.pathname().ok().unwrap_or_else(|| "/app".into());
    let search = location.search().ok().unwrap_or_default();
    let hash = location.hash().ok().unwrap_or_default();
    let Ok(params) = UrlSearchParams::new_with_str(&search) else {
        return;
    };
    params.delete("notice");

    let query = params.to_string().as_string().unwrap_or_default();
    let mut next = pathname;
    if !query.is_empty() {
        next.push('?');
        next.push_str(&query);
    }
    if !hash.is_empty() {
        next.push_str(&hash);
    }

    if let Ok(history) = window.history() {
        let _ = history.replace_state_with_url(&wasm_bindgen::JsValue::NULL, "", Some(&next));
    }
}

#[cfg(feature = "hydrate")]
fn queue_shell_notice_dismiss(notice: RwSignal<Option<ShellNotice>>) {
    let closure = wasm_bindgen::closure::Closure::wrap(Box::new(move || {
        notice.set(None);
    }) as Box<dyn FnMut()>);

    if let Some(window) = window() {
        let _ = window.set_timeout_with_callback_and_timeout_and_arguments_0(
            closure.as_ref().unchecked_ref(),
            4200,
        );
    }

    closure.forget();
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
fn sync_shell_control_visibility(element: &web_sys::Element, visible: bool) {
    if visible {
        element.remove_attribute("hidden").ok();
        element.set_attribute("aria-hidden", "false").ok();
        element.remove_attribute("tabindex").ok();
    } else {
        element.set_attribute("hidden", "").ok();
        element.set_attribute("aria-hidden", "true").ok();
        element.set_attribute("tabindex", "-1").ok();
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

    if viewport != "mobile" {
        MOBILE_SIDEBAR_OPEN.with(|open| open.set(false));
        MOBILE_SEARCH_OPEN.with(|open| open.set(false));
    }
    if viewport != "tablet" {
        TABLET_PROFILE_OPEN.with(|open| open.set(false));
    }

    let state = shell_sidebar_state(viewport);
    let search_state = MOBILE_SEARCH_OPEN.with(|open| {
        if viewport == "mobile" && open.get() {
            "open"
        } else {
            "closed"
        }
    });
    let profile_state = TABLET_PROFILE_OPEN.with(|open| {
        if viewport == "tablet" && state == "collapsed" && open.get() {
            "open"
        } else {
            "closed"
        }
    });

    root.set_attribute("data-shell-ready", "true").ok();
    body.set_attribute("data-shell-viewport", viewport).ok();
    body.set_attribute("data-sidebar-state", state).ok();
    body.set_attribute("data-search-state", search_state).ok();
    body.set_attribute("data-profile-state", profile_state).ok();

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
        sync_shell_control_visibility(&toggle, viewport != "desktop");
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

    if let Ok(Some(search_toggle)) = document.query_selector(".app-utility-button--search") {
        search_toggle
            .set_attribute(
                "aria-expanded",
                if search_state == "open" {
                    "true"
                } else {
                    "false"
                },
            )
            .ok();
        search_toggle
            .set_attribute(
                "aria-label",
                if search_state == "open" {
                    "Close search"
                } else {
                    "Open search"
                },
            )
            .ok();
        search_toggle
            .set_attribute(
                "title",
                if search_state == "open" {
                    "Close search"
                } else {
                    "Open search"
                },
            )
            .ok();
    }

    if let Ok(Some(close_button)) = document.query_selector(".app-sidebar-close") {
        sync_shell_control_visibility(
            &close_button,
            viewport == "mobile" && state == "overlay-open",
        );
    }

    if let Ok(Some(backdrop)) = document.query_selector(".app-sidebar-backdrop") {
        let backdrop_visible = (viewport == "mobile" && state == "overlay-open")
            || (viewport == "tablet" && profile_state == "open");
        sync_shell_control_visibility(&backdrop, backdrop_visible);
        backdrop
            .set_attribute(
                "aria-label",
                if viewport == "tablet" && profile_state == "open" {
                    "Close account menu"
                } else {
                    "Close navigation"
                },
            )
            .ok();
    }
}

#[cfg(feature = "hydrate")]
fn toggle_shell_sidebar() {
    match shell_viewport() {
        "mobile" => {
            MOBILE_SEARCH_OPEN.with(|open| open.set(false));
            MOBILE_SIDEBAR_OPEN.with(|open| open.set(!open.get()));
        }
        "tablet" => {
            TABLET_PROFILE_OPEN.with(|open| open.set(false));
            TABLET_SIDEBAR_EXPANDED.with(|expanded| expanded.set(!expanded.get()));
        }
        _ => {}
    }
    apply_shell_chrome_state();
}

#[cfg(feature = "hydrate")]
fn close_shell_sidebar() {
    match shell_viewport() {
        "mobile" => MOBILE_SIDEBAR_OPEN.with(|open| open.set(false)),
        "tablet" => {
            let profile_open = TABLET_PROFILE_OPEN.with(|open| open.get());
            if profile_open {
                TABLET_PROFILE_OPEN.with(|open| open.set(false));
            } else {
                TABLET_SIDEBAR_EXPANDED.with(|expanded| expanded.set(false));
            }
        }
        _ => {}
    }
    apply_shell_chrome_state();
}

#[cfg(feature = "hydrate")]
fn close_mobile_search() {
    if shell_viewport() != "mobile" {
        return;
    }

    MOBILE_SEARCH_OPEN.with(|open| open.set(false));
    apply_shell_chrome_state();
}

#[cfg(feature = "hydrate")]
fn toggle_mobile_search() {
    if shell_viewport() != "mobile" {
        return;
    }

    let should_focus = MOBILE_SEARCH_OPEN.with(|open| {
        let next = !open.get();
        open.set(next);
        next
    });
    MOBILE_SIDEBAR_OPEN.with(|open| open.set(false));
    apply_shell_chrome_state();

    if should_focus {
        if let Some(document) = window().and_then(|window| window.document()) {
            if let Some(input) = document.get_element_by_id("global-search") {
                if let Ok(input) = input.dyn_into::<web_sys::HtmlInputElement>() {
                    input.focus().ok();
                }
            }
        }
    }
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

    let click = wasm_bindgen::closure::Closure::wrap(Box::new(move |event: web_sys::Event| {
        let Some(target) = event.target() else {
            return;
        };
        let Some(element) = target.dyn_into::<web_sys::Element>().ok() else {
            return;
        };

        if element.closest("#app-nav-toggle").ok().flatten().is_some() {
            toggle_shell_sidebar();
            return;
        }

        if element
            .closest(".app-utility-button--search")
            .ok()
            .flatten()
            .is_some()
        {
            toggle_mobile_search();
            return;
        }

        if element
            .closest(".app-sidebar__rail-profile")
            .ok()
            .flatten()
            .is_some()
        {
            if shell_viewport() == "tablet" {
                let sidebar_expanded = TABLET_SIDEBAR_EXPANDED.with(|expanded| expanded.get());
                if !sidebar_expanded {
                    TABLET_PROFILE_OPEN.with(|open| open.set(!open.get()));
                }
                apply_shell_chrome_state();
            }
            return;
        }

        if element
            .closest("[data-sidebar-dismiss]")
            .ok()
            .flatten()
            .is_some()
        {
            close_shell_sidebar();
            return;
        }

        let search_open = MOBILE_SEARCH_OPEN.with(|open| open.get());
        if shell_viewport() == "mobile"
            && search_open
            && element
                .closest(".top-app-bar__search")
                .ok()
                .flatten()
                .is_none()
            && element
                .closest(".app-utility-button--search")
                .ok()
                .flatten()
                .is_none()
        {
            close_mobile_search();
        }
    }) as Box<dyn FnMut(_)>);
    let _ = document.add_event_listener_with_callback("click", click.as_ref().unchecked_ref());
    click.forget();

    if let Some(window) = window() {
        let resize = wasm_bindgen::closure::Closure::wrap(Box::new(move |_event: web_sys::Event| {
            apply_shell_chrome_state();
        }) as Box<dyn FnMut(_)>);
        let _ = window.add_event_listener_with_callback("resize", resize.as_ref().unchecked_ref());
        resize.forget();
    }

    let keydown = wasm_bindgen::closure::Closure::wrap(Box::new(move |event: KeyboardEvent| {
        if event.key() == "Escape" {
            close_mobile_search();
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
    let popover_open = RwSignal::new(false);

    let apply_preference = Callback::new(move |choice: String| {
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
            choice.as_str()
        };
        root.set_attribute("data-theme-preference", &choice).ok();
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
            let _ = storage.set_item(STORAGE_KEY, &choice);
        }
        preference.set(choice);
    });

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

    let theme_choice_icon = Memo::new(move |_| {
        let choice = preference.get();
        match choice.as_str() {
            "dark" => "fa-solid fa-moon".to_string(),
            "light" => "fa-solid fa-sun".to_string(),
            _ => "fa-solid fa-circle-half-stroke".to_string(),
        }
    });

    view! {
        <div class="theme-toggle theme-toggle--popover">
            <button
                class="app-sidebar__icon-button theme-toggle-button"
                type="button"
                aria-label="Theme options"
                title=move || {
                    let current = preference.get();
                    let label = match current.as_str() {
                        "light" => "Light",
                        "dark" => "Dark",
                        _ => "System",
                    };
                    format!("Theme: {label}")
                }
                aria-haspopup="menu"
                aria-expanded=move || if popover_open.get() { "true" } else { "false" }
                on:click=move |_| popover_open.update(|open| *open = !*open)
            >
                <span aria-hidden="true">
                    <i class=move || theme_choice_icon.get()></i>
                </span>
            </button>
            <div
                class="theme-toggle__popover"
                class:is-open=move || popover_open.get()
                role="menu"
                aria-label="Theme options"
                hidden=move || !popover_open.get()
            >
                <button
                    class="theme-toggle__option"
                    class:is-active=move || preference.get() == "system"
                    type="button"
                    role="menuitemradio"
                    aria-checked=move || if preference.get() == "system" { "true" } else { "false" }
                    on:click=move |_| {
                        apply_preference.run("system".into());
                        popover_open.set(false);
                    }
                >
                    <span class="theme-toggle__option-icon" aria-hidden="true">
                        <i class="fa-solid fa-circle-half-stroke"></i>
                    </span>
                    <span class="theme-toggle__option-label">"System"</span>
                </button>
                <button
                    class="theme-toggle__option"
                    class:is-active=move || preference.get() == "light"
                    type="button"
                    role="menuitemradio"
                    aria-checked=move || if preference.get() == "light" { "true" } else { "false" }
                    on:click=move |_| {
                        apply_preference.run("light".into());
                        popover_open.set(false);
                    }
                >
                    <span class="theme-toggle__option-icon" aria-hidden="true">
                        <i class="fa-solid fa-sun"></i>
                    </span>
                    <span class="theme-toggle__option-label">"Light"</span>
                </button>
                <button
                    class="theme-toggle__option"
                    class:is-active=move || preference.get() == "dark"
                    type="button"
                    role="menuitemradio"
                    aria-checked=move || if preference.get() == "dark" { "true" } else { "false" }
                    on:click=move |_| {
                        apply_preference.run("dark".into());
                        popover_open.set(false);
                    }
                >
                    <span class="theme-toggle__option-icon" aria-hidden="true">
                        <i class="fa-solid fa-moon"></i>
                    </span>
                    <span class="theme-toggle__option-label">"Dark"</span>
                </button>
            </div>
        </div>
    }
}

#[cfg(not(feature = "hydrate"))]
#[component]
fn ThemeToggle() -> impl IntoView {
    view! {
        <div class="theme-toggle theme-toggle--popover">
            <button
                class="app-sidebar__icon-button theme-toggle-button"
                type="button"
                aria-label="Theme options"
                title="Theme: System"
                aria-haspopup="menu"
                aria-expanded="false"
            >
                <span aria-hidden="true">
                    <i class="fa-solid fa-circle-half-stroke"></i>
                </span>
            </button>
            <div
                class="theme-toggle__popover"
                role="menu"
                aria-label="Theme options"
                hidden
            >
                <button
                    class="theme-toggle__option is-active"
                    type="button"
                    role="menuitemradio"
                    aria-checked="true"
                >
                    <span class="theme-toggle__option-icon" aria-hidden="true">
                        <i class="fa-solid fa-circle-half-stroke"></i>
                    </span>
                    <span class="theme-toggle__option-label">"System"</span>
                </button>
                <button
                    class="theme-toggle__option"
                    type="button"
                    role="menuitemradio"
                    aria-checked="false"
                >
                    <span class="theme-toggle__option-icon" aria-hidden="true">
                        <i class="fa-solid fa-sun"></i>
                    </span>
                    <span class="theme-toggle__option-label">"Light"</span>
                </button>
                <button
                    class="theme-toggle__option"
                    type="button"
                    role="menuitemradio"
                    aria-checked="false"
                >
                    <span class="theme-toggle__option-icon" aria-hidden="true">
                        <i class="fa-solid fa-moon"></i>
                    </span>
                    <span class="theme-toggle__option-label">"Dark"</span>
                </button>
            </div>
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
    #[prop(optional)] required_capability: Option<&'static str>,
) -> impl IntoView {
    if let Some(required_capability) = required_capability {
        if !route_visible(loaded, account.as_ref(), Some(required_capability)) {
            return view! { <></> }.into_any();
        }
    }

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
fn TopBarUtilityButton(
    aria_label: &'static str,
    icon: &'static str,
    #[prop(optional)] class_name: &'static str,
    #[prop(optional)] disabled: bool,
) -> impl IntoView {
    let button_class = if class_name.is_empty() {
        "app-utility-button".to_string()
    } else {
        format!("app-utility-button {class_name}")
    };

    view! {
        <button
            class=button_class
            type="button"
            aria-label=aria_label
            title=aria_label
            disabled=disabled
        >
            <span class="app-utility-button__icon" aria-hidden="true">
                <i class=format!("fa-solid {}", icon)></i>
            </span>
        </button>
    }
}

#[component]
fn SidebarFooterContext(
    account: Option<AccountContext>,
    error: Option<String>,
    search: String,
    session: AccountSession,
) -> impl IntoView {
    match account {
        Some(account) => {
            let active_delegate = active_delegate(&account, &search);
            let has_active_delegate = active_delegate.is_some();
            let active_delegate_name = active_delegate
                .as_ref()
                .map(|delegate| preferred_account_label(&delegate.display_name, &delegate.email));
            let identity_label = preferred_account_label(&account.display_name, &account.email);
            let identity_initials = account_initials(&identity_label);
            let profile_label = ui_profile_label(account.ui_access_profile.clone());
            let rail_profile_label = format!("{identity_label} profile");
            let active_delegate_name = StoredValue::new(active_delegate_name.unwrap_or_default());
            let footer_error = StoredValue::new(error.unwrap_or_default());
            let has_footer_error = !footer_error.get_value().is_empty();

            view! {
                <section class="app-sidebar__footer-context" id="shell-footer-context">
                    <button
                        class="app-sidebar__rail-profile"
                        type="button"
                        title=identity_label.clone()
                        aria-label=rail_profile_label
                        data-sidebar-profile
                        style="display:inline-flex;width:2.5rem;min-width:2.5rem;height:2.5rem;min-height:2.5rem;padding:0;border:0;background:transparent;appearance:none;box-shadow:none;"
                    >
                        <span class="app-sidebar__rail-profile-avatar" aria-hidden="true">
                            {identity_initials.clone()}
                        </span>
                    </button>
                    <section id="shell-account-context" class="selection-panel app-sidebar__supplemental app-sidebar__context-composite">
                        <div class="app-sidebar__account-summary">
                            <span class="app-sidebar__avatar" aria-hidden="true">{identity_initials}</span>
                            <div class="app-sidebar__identity">
                                <div class="app-sidebar__identity-row">
                                    <strong class="app-sidebar__identity-name">{identity_label}</strong>
                                    <span class="app-sidebar__context-badge">{profile_label}</span>
                                </div>
                                <span class="app-sidebar__identity-email">{account.email.clone()}</span>
                            </div>
                        </div>
                        <Show when=move || has_active_delegate>
                            <div class="app-sidebar__context-stack">
                                <p class="muted app-sidebar__context-note">
                                    <strong>"Acting for "</strong>
                                    {move || active_delegate_name.get_value()}
                                </p>
                            </div>
                        </Show>
                        <div class="app-sidebar__footer-toolbar">
                            <ThemeToggle/>
                            <SignOutButton session=session/>
                        </div>
                        <Show when=move || has_footer_error>
                            <p class="muted app-sidebar__footer-error">
                                {move || footer_error.get_value()}
                            </p>
                        </Show>
                    </section>
                </section>
            }
            .into_any()
        }
        None => view! {
            <section class="app-sidebar__footer-context" id="shell-footer-context">
                <button
                    class="app-sidebar__rail-profile"
                    type="button"
                    title="Account context"
                    aria-label="Account profile"
                    data-sidebar-profile
                    style="display:inline-flex;width:2.5rem;min-width:2.5rem;height:2.5rem;min-height:2.5rem;padding:0;border:0;background:transparent;appearance:none;box-shadow:none;"
                >
                    <span class="app-sidebar__rail-profile-avatar" aria-hidden="true">"TS"</span>
                </button>
                <section id="shell-account-context" class="selection-panel app-sidebar__supplemental app-sidebar__context-composite">
                    <div class="app-sidebar__account-summary">
                        <span class="app-sidebar__avatar" aria-hidden="true">"TS"</span>
                        <div class="app-sidebar__identity">
                            <div class="app-sidebar__identity-row">
                                <strong class="app-sidebar__identity-name">"Account Context"</strong>
                            </div>
                            <span class="app-sidebar__identity-email">
                                {error.unwrap_or_else(|| "Account context loads after session verification.".into())}
                            </span>
                        </div>
                    </div>
                    <div class="app-sidebar__footer-toolbar">
                        <ThemeToggle/>
                    </div>
                </section>
            </section>
        }
        .into_any(),
    }
}

#[component]
fn ShellToast(notice: ShellNotice, on_dismiss: Callback<()>) -> impl IntoView {
    view! {
        <aside class="shell-toast shell-toast--warning" data-shell-toast role="alert" aria-live="assertive">
            <div class="shell-toast__copy">
                <strong class="shell-toast__title">{notice.title()}</strong>
                <p class="shell-toast__message">{notice.message()}</p>
            </div>
            <button
                class="shell-toast__dismiss"
                type="button"
                aria-label="Dismiss notification"
                on:click=move |_| on_dismiss.run(())
            >
                <span aria-hidden="true"><i class="fa-solid fa-xmark"></i></span>
            </button>
        </aside>
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
                class="button is-light app-sidebar__icon-button app-session-actions__button"
                type="button"
                aria-label=move || if busy.get() { "Signing out" } else { "Sign out" }
                title=move || if busy.get() { "Signing out" } else { "Sign out" }
                on:click=sign_out
                disabled=move || busy.get()
            >
                <span aria-hidden="true">
                    <i class=move || if busy.get() {
                        "fa-solid fa-spinner fa-spin".to_string()
                    } else {
                        "fa-solid fa-right-from-bracket".to_string()
                    }></i>
                </span>
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
        || href == "/app/components"
        || href.starts_with("/app/components/")
        || href == "/app/dashboards"
        || href.starts_with("/app/dashboards/")
        || href == "/app/datasets"
        || href.starts_with("/app/datasets/")
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
    let notice = RwSignal::new(None::<ShellNotice>);
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

    #[cfg(feature = "hydrate")]
    Effect::new(move |_| {
        let _ = location.pathname.get();
        let _ = location.search.read();

        if let Some(next_notice) = read_shell_notice() {
            notice.set(Some(next_notice));
            clear_shell_notice_query();
            queue_shell_notice_dismiss(notice);
        }
    });

    #[cfg(feature = "hydrate")]
    Effect::new(move |_| {
        let _ = location.pathname.get();
        let _ = location.search.read();

        if allow_unauthenticated || !session.loaded.get() {
            return;
        }

        let account = session.account.get();
        if let Some(required_capability) = required_capability {
            if account.is_some() && !has_capability(account.as_ref(), required_capability) {
                redirect("/app?notice=access-denied");
            }
        }
    });

    view! {
        <main class=format!("shell app-shell app-shell--{active_route}")>
            <Show when=move || notice.get().is_some()>
                {move || {
                    notice
                        .get()
                        .map(|next_notice| {
                            view! {
                                <ShellToast
                                    notice=next_notice
                                    on_dismiss=Callback::new(move |_| notice.set(None))
                                />
                            }
                            .into_any()
                        })
                        .unwrap_or_else(|| view! { <></> }.into_any())
                }}
            </Show>
            <button class="app-sidebar-backdrop" type="button" aria-label="Close navigation" aria-hidden="true" data-sidebar-dismiss tabindex="-1" hidden></button>
            <section class="app-layout">
                <aside id="app-sidebar" class="panel box app-sidebar" aria-label="Application navigation">
                    <a class="app-sidebar__brand" href="/app" aria-label="Tessara home">
                        <span class="app-sidebar__brand-mark" aria-hidden="true">
                            <img class="app-sidebar__brand-mark-image" src="/assets/tessara-icon-256.svg" alt="" />
                        </span>
                        <span class="app-sidebar__brand-copy">
                            <strong class="app-sidebar__brand-name">"Tessara"</strong>
                            <span class="app-sidebar__brand-tag">"Shared workspace"</span>
                        </span>
                    </a>
                    {move || {
                        let loaded = session.loaded.get();
                        let account = session.account.get();
                        let error = session.error.get();

                        view! {
                            <div class="app-sidebar__content">
                                <NavSection
                                    heading="Main"
                                    aria_label="Primary navigation"
                                    links=PRODUCT_LINKS
                                    active_route=active_route
                                    loaded=loaded
                                    account=account.clone()
                                    section_class="nav-panel-primary"
                                />
                                <NavSection
                                    heading="Admin"
                                    aria_label="Admin navigation"
                                    links=ADMIN_LINKS
                                    active_route=active_route
                                    loaded=loaded
                                    account=account.clone()
                                    section_class="nav-panel-analytics"
                                    required_capability="admin:all"
                                />
                                <div class="app-sidebar__footer">
                                    <SidebarFooterContext
                                        account=account
                                        error=error
                                        search=location.search.read().to_string()
                                        session=session
                                    />
                                </div>
                            </div>
                        }
                    }}
                </aside>
                <section class="panel box app-main">
                    <header class="top-app-bar">
                        <div class="top-app-bar__row">
                            <div class="top-app-bar__brand">
                                <button id="app-nav-toggle" class="app-nav-toggle" type="button" aria-label="Toggle navigation" aria-controls="app-sidebar" aria-expanded="false" hidden>
                                    <span class="app-nav-toggle__icon" aria-hidden="true"><i class="fa-solid fa-bars"></i></span>
                                </button>
                                <div class="top-app-bar__context-group">
                                    <span class="top-app-bar__context-label">"Workspace"</span>
                                    <span class="top-app-bar__context">{workspace_label}</span>
                                </div>
                            </div>
                            <div class="top-app-bar__utilities">
                                <div class="top-app-bar__search">
                                    <label class="is-sr-only" for="global-search">"Global search"</label>
                                    <input id="global-search" class="input app-search-input" type="search" placeholder="Search Tessara" autocomplete="off" />
                                </div>
                                <TopBarUtilityButton aria_label="Open search" icon="fa-magnifying-glass" class_name="app-utility-button--search"/>
                                <TopBarUtilityButton aria_label="Notifications" icon="fa-bell" disabled=true/>
                                <TopBarUtilityButton aria_label="Help" icon="fa-circle-question" disabled=true/>
                            </div>
                        </div>
                    </header>
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
                                    <Panel title="Redirecting Home" description="Returning to Home with access feedback from the shared shell.">
                                        <p class="muted">"Redirecting..."</p>
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
