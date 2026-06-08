use icons::{
    Bell, Blocks, ChevronRight, CircleHelp, ClipboardCheck, Database, Ellipsis, File, FileText,
    GitBranch, House, LayoutDashboard, LogOut, Menu, Monitor, Moon, Search, SlidersHorizontal, Sun,
    Workflow,
};
use leptos::prelude::*;
use crate::ui::empty_view;

#[cfg(feature = "hydrate")]
use serde::Deserialize;
#[cfg(feature = "hydrate")]
use wasm_bindgen::JsCast;
#[cfg(feature = "hydrate")]
use wasm_bindgen::JsValue;

#[cfg(feature = "hydrate")]
use crate::theme::{DARK_THEME_COLOR, LIGHT_THEME_COLOR, STORAGE_KEY};

#[derive(Clone, Copy)]
struct NavItem {
    key: &'static str,
    href: &'static str,
    label: &'static str,
    section: &'static str,
    capabilities: &'static [&'static str],
}

const NAV_ITEMS: [NavItem; 10] = [
    NavItem {
        key: "home",
        href: "/",
        label: "Home",
        section: "Main",
        capabilities: &[],
    },
    NavItem {
        key: "organization",
        href: "/organization",
        label: "Organization",
        section: "Main",
        capabilities: &["hierarchy:read", "hierarchy:manage"],
    },
    NavItem {
        key: "forms",
        href: "/forms",
        label: "Forms",
        section: "Main",
        capabilities: &["forms:read", "forms:manage"],
    },
    NavItem {
        key: "workflows",
        href: "/workflows",
        label: "Workflows",
        section: "Main",
        capabilities: &["workflows:read", "workflows:manage"],
    },
    NavItem {
        key: "responses",
        href: "/responses",
        label: "Responses",
        section: "Main",
        capabilities: &[
            "submissions:read_own",
            "submissions:respond",
            "submissions:manage",
        ],
    },
    NavItem {
        key: "operations",
        href: "/operations",
        label: "Operations",
        section: "Main",
        capabilities: &["operations:view"],
    },
    NavItem {
        key: "components",
        href: "/components",
        label: "Components",
        section: "Main",
        capabilities: &["components:read", "components:manage"],
    },
    NavItem {
        key: "dashboards",
        href: "/dashboards",
        label: "Dashboards",
        section: "Main",
        capabilities: &["dashboards:read", "dashboards:manage"],
    },
    NavItem {
        key: "administration",
        href: "/administration",
        label: "Administration",
        section: "Admin",
        capabilities: &["admin:all"],
    },
    NavItem {
        key: "datasets",
        href: "/datasets",
        label: "Datasets",
        section: "Admin",
        capabilities: &["datasets:read", "datasets:manage"],
    },
];

#[component]
pub fn AppShell(
    active_route: &'static str,
    title: &'static str,
    children: Children,
) -> impl IntoView {
    require_authenticated_route(active_route);

    view! {
        <main class="app-shell">
            <Sidebar active_route/>
            <section class="app-main" aria-label="Application content">
                <TopAppBar active_route title/>
                <div class="app-page">
                    {children()}
                </div>
            </section>
        </main>
    }
}

fn require_authenticated_route(active_route: &'static str) {
    #[cfg(feature = "hydrate")]
    {
        if active_route == "home" {
            return;
        }

        leptos::task::spawn_local(async move {
            let response = gloo_net::http::Request::get("/api/auth/session")
                .send()
                .await;

            let session = match response {
                Ok(response) if response.ok() => response.json::<SessionStateResponse>().await.ok(),
                _ => None,
            };

            let authenticated = session
                .as_ref()
                .map(|session| session.authenticated)
                .unwrap_or(false);

            if !authenticated {
                if let Some(window) = web_sys::window() {
                    let _ = window.location().set_href("/login");
                }
                return;
            }

            if let Some(item) = nav_item_for_route(active_route) {
                let capabilities = session
                    .and_then(|session| session.account)
                    .map(|account| account.capabilities)
                    .unwrap_or_default();
                if !nav_item_is_allowed(item, &capabilities) {
                    if let Some(window) = web_sys::window() {
                        let _ = window.location().set_href("/");
                    }
                }
            }
        });
    }

    #[cfg(not(feature = "hydrate"))]
    {
        let _ = active_route;
    }
}

#[cfg(feature = "hydrate")]
#[derive(Deserialize)]
struct SessionStateResponse {
    authenticated: bool,
    account: Option<ShellAccountContext>,
}

#[cfg(feature = "hydrate")]
#[derive(Clone, Deserialize)]
struct ShellAccountContext {
    email: String,
    display_name: String,
    capabilities: Vec<String>,
}

#[component]
pub fn Sidebar(active_route: &'static str) -> impl IntoView {
    view! {
        <aside class="sidebar" aria-label="Primary navigation">
            <SidebarContent active_route/>
        </aside>
    }
}

#[component]
fn SidebarContent(active_route: &'static str) -> impl IntoView {
    let account = RwSignal::new(None::<ShellAccountSummary>);

    Effect::new(move |_| {
        load_shell_account(account);
    });

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

#[derive(Clone, Debug, PartialEq)]
struct ShellAccountSummary {
    email: String,
    display_name: String,
    capabilities: Vec<String>,
}

#[component]
fn AccountCard(account: RwSignal<Option<ShellAccountSummary>>) -> impl IntoView {
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

fn load_shell_account(account: RwSignal<Option<ShellAccountSummary>>) {
    #[cfg(feature = "hydrate")]
    {
        leptos::task::spawn_local(async move {
            let response = gloo_net::http::Request::get("/api/auth/session")
                .send()
                .await;

            match response {
                Ok(response) if response.ok() => {
                    let session = response.json::<SessionStateResponse>().await.ok();
                    account.set(session.and_then(|session| {
                        if !session.authenticated {
                            return None;
                        }
                        session.account.map(|account| ShellAccountSummary {
                            email: account.email,
                            display_name: account.display_name,
                            capabilities: account.capabilities,
                        })
                    }));
                }
                _ => account.set(None),
            }
        });
    }

    #[cfg(not(feature = "hydrate"))]
    {
        let _ = account;
    }
}

fn submit_logout() {
    #[cfg(feature = "hydrate")]
    {
        leptos::task::spawn_local(async move {
            let _ = gloo_net::http::Request::delete("/api/auth/logout")
                .send()
                .await;

            if let Some(window) = web_sys::window() {
                let _ = window.location().set_href("/login");
            }
        });
    }
}

#[cfg_attr(not(feature = "hydrate"), allow(dead_code))]
fn nav_item_for_route(route_key: &str) -> Option<&'static NavItem> {
    NAV_ITEMS.iter().find(|item| item.key == route_key)
}

fn nav_item_is_allowed(item: &NavItem, capabilities: &[String]) -> bool {
    item.capabilities.is_empty()
        || capabilities
            .iter()
            .any(|capability| capability == "admin:all")
        || item
            .capabilities
            .iter()
            .any(|required| capabilities.iter().any(|capability| capability == required))
}

fn nav_section_for(
    section: &'static str,
    active_route: &'static str,
    account: Option<ShellAccountSummary>,
) -> impl IntoView {
    let capabilities = account
        .as_ref()
        .map(|account| account.capabilities.as_slice())
        .unwrap_or(&[]);
    let items = NAV_ITEMS
        .iter()
        .filter(move |item| item.section == section)
        .filter(|item| nav_item_is_allowed(item, capabilities))
        .collect::<Vec<_>>();

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

fn nav_item_link(item: &'static NavItem, active_route: &'static str) -> impl IntoView {
    let class = if item.key == active_route {
        "sidebar-link is-active"
    } else {
        "sidebar-link"
    };
    view! {
        <a class=class href=item.href title=item.label aria-label=item.label>
            <NavIcon route_key=item.key/>
            <span class="sidebar-link__label">{item.label}</span>
        </a>
    }
}

#[component]
fn NavIcon(route_key: &'static str) -> impl IntoView {
    match route_key {
        "home" => view! { <span class="sidebar-link__icon-wrap" aria-hidden="true"><House class="sidebar-link__icon"/></span> }.into_any(),
        "organization" => view! { <span class="sidebar-link__icon-wrap" aria-hidden="true"><GitBranch class="sidebar-link__icon"/></span> }.into_any(),
        "forms" => view! { <span class="sidebar-link__icon-wrap" aria-hidden="true"><FileText class="sidebar-link__icon"/></span> }.into_any(),
        "workflows" => view! { <span class="sidebar-link__icon-wrap" aria-hidden="true"><Workflow class="sidebar-link__icon"/></span> }.into_any(),
        "responses" => view! { <span class="sidebar-link__icon-wrap" aria-hidden="true"><ClipboardCheck class="sidebar-link__icon"/></span> }.into_any(),
        "operations" => view! { <span class="sidebar-link__icon-wrap" aria-hidden="true"><Monitor class="sidebar-link__icon"/></span> }.into_any(),
        "components" => view! { <span class="sidebar-link__icon-wrap" aria-hidden="true"><Blocks class="sidebar-link__icon"/></span> }.into_any(),
        "dashboards" => view! { <span class="sidebar-link__icon-wrap" aria-hidden="true"><LayoutDashboard class="sidebar-link__icon"/></span> }.into_any(),
        "datasets" => view! { <span class="sidebar-link__icon-wrap" aria-hidden="true"><Database class="sidebar-link__icon"/></span> }.into_any(),
        "administration" => view! { <span class="sidebar-link__icon-wrap" aria-hidden="true"><SlidersHorizontal class="sidebar-link__icon"/></span> }.into_any(),
        _ => view! { <span class="sidebar-link__icon-wrap" aria-hidden="true"><File class="sidebar-link__icon"/></span> }.into_any(),
    }
}

#[component]
pub fn TopAppBar(active_route: &'static str, title: &'static str) -> impl IntoView {
    view! {
        <header class="top-app-bar">
            <div class="top-app-bar__title-row">
                <MobileNav active_route/>
                <h1>{title}</h1>
            </div>
            <div class="top-app-bar__actions">
                <label class="search-field">
                    <span class="sr-only">"Search Tessara"</span>
                    <input type="search" placeholder="Search Tessara"/>
                </label>
                <ThemeToggle/>
                <IconButton label="Notifications">
                    <Bell class="icon-button__icon"/>
                </IconButton>
                <IconButton label="Help">
                    <CircleHelp class="icon-button__icon"/>
                </IconButton>
            </div>
        </header>
    }
}

#[component]
fn ThemeToggle() -> impl IntoView {
    let preference = RwSignal::new("system");
    let is_open = RwSignal::new(false);
    let toggle_class = move || {
        if is_open.get() {
            "theme-toggle is-open"
        } else {
            "theme-toggle"
        }
    };

    Effect::new(move |_| {
        preference.set(read_theme_preference());
    });

    view! {
        <div class=toggle_class>
            <button
                class="icon-button theme-toggle__trigger"
                type="button"
                aria-label="Theme options"
                aria-haspopup="menu"
                aria-expanded=move || is_open.get().to_string()
                on:click=move |_| is_open.update(|open| *open = !*open)
            >
                <Sun class="icon-button__icon theme-toggle__icon theme-toggle__icon--sun"/>
                <Moon class="icon-button__icon theme-toggle__icon theme-toggle__icon--moon"/>
            </button>
            <button
                class="theme-toggle__scrim"
                type="button"
                aria-label="Close theme options"
                on:click=move |_| is_open.set(false)
            ></button>
            <div class="theme-toggle__menu blurred-surface" role="menu" aria-label="Theme options">
                <button
                    class=move || theme_option_class(preference.get() == "system")
                    type="button"
                    role="menuitemradio"
                    aria-checked=move || (preference.get() == "system").to_string()
                    on:click=move |_| {
                        preference.set("system");
                        set_theme_preference("system");
                        is_open.set(false);
                    }
                >
                    <Monitor class="theme-toggle__option-icon"/>
                    <span>"System"</span>
                </button>
                <button
                    class=move || theme_option_class(preference.get() == "light")
                    type="button"
                    role="menuitemradio"
                    aria-checked=move || (preference.get() == "light").to_string()
                    on:click=move |_| {
                        preference.set("light");
                        set_theme_preference("light");
                        is_open.set(false);
                    }
                >
                    <Sun class="theme-toggle__option-icon"/>
                    <span>"Light"</span>
                </button>
                <button
                    class=move || theme_option_class(preference.get() == "dark")
                    type="button"
                    role="menuitemradio"
                    aria-checked=move || (preference.get() == "dark").to_string()
                    on:click=move |_| {
                        preference.set("dark");
                        set_theme_preference("dark");
                        is_open.set(false);
                    }
                >
                    <Moon class="theme-toggle__option-icon"/>
                    <span>"Dark"</span>
                </button>
            </div>
        </div>
    }
}

fn theme_option_class(is_active: bool) -> &'static str {
    if is_active {
        "theme-toggle__option is-active"
    } else {
        "theme-toggle__option"
    }
}

fn read_theme_preference() -> &'static str {
    #[cfg(feature = "hydrate")]
    {
        let stored = web_sys::window()
            .and_then(|window| window.local_storage().ok().flatten())
            .and_then(|storage| storage.get_item(STORAGE_KEY).ok().flatten());

        match stored.as_deref() {
            Some("light") => "light",
            Some("dark") => "dark",
            _ => "system",
        }
    }

    #[cfg(not(feature = "hydrate"))]
    {
        "system"
    }
}

fn set_theme_preference(preference: &'static str) {
    #[cfg(feature = "hydrate")]
    {
        let Some(window) = web_sys::window() else {
            return;
        };

        if let Ok(Some(storage)) = window.local_storage() {
            let _ = storage.set_item(STORAGE_KEY, preference);
        }

        let resolved_theme = if preference == "system" {
            window
                .match_media("(prefers-color-scheme: dark)")
                .ok()
                .flatten()
                .map(|media_query| {
                    if media_query.matches() {
                        "dark"
                    } else {
                        "light"
                    }
                })
                .unwrap_or("light")
        } else {
            preference
        };

        let Some(document) = window.document() else {
            return;
        };
        let Some(root) = document.document_element() else {
            return;
        };

        let _ = root.set_attribute("data-theme-preference", preference);
        let _ = root.set_attribute("data-theme", resolved_theme);

        if let Ok(Some(meta_theme_color)) = document.query_selector("meta[name=\"theme-color\"]") {
            let theme_color = if resolved_theme == "dark" {
                DARK_THEME_COLOR
            } else {
                LIGHT_THEME_COLOR
            };
            let _ = meta_theme_color.set_attribute("content", theme_color);
        }
    }

    #[cfg(not(feature = "hydrate"))]
    {
        let _ = preference;
    }
}

#[component]
fn MobileNav(active_route: &'static str) -> impl IntoView {
    let is_open = RwSignal::new(false);
    let nav_class = move || {
        if is_open.get() {
            "mobile-nav is-open"
        } else {
            "mobile-nav"
        }
    };

    view! {
        <div class=nav_class>
            <button
                class="icon-button mobile-nav__toggle"
                type="button"
                aria-label="Open navigation"
                aria-expanded=move || is_open.get().to_string()
                on:click=move |_| is_open.set(true)
            >
                <Menu class="icon-button__icon"/>
            </button>
            <button
                class="mobile-nav__scrim"
                type="button"
                aria-label="Close navigation"
                on:click=move |_| is_open.set(false)
            ></button>
            <aside class="mobile-nav__panel blurred-surface" aria-label="Primary navigation">
                <SidebarContent active_route/>
            </aside>
        </div>
    }
}

#[component]
pub fn PageHeader(
    title: &'static str,
    #[prop(optional)] description: Option<&'static str>,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    view! {
        <header class="page-header">
            <div>
                <h2>{title}</h2>
                {description
                    .map(|description| view! { <p>{description}</p> })}
            </div>
            <div class="page-header__actions">
                {children.map(|children| children())}
            </div>
        </header>
    }
}

#[component]
pub fn Button(label: &'static str, #[prop(optional)] href: Option<&'static str>) -> impl IntoView {
    match href {
        Some(href) => view! { <a class="button" href=href>{label}</a> }.into_any(),
        None => view! { <button class="button" type="button">{label}</button> }.into_any(),
    }
}

#[component]
pub fn Breadcrumb(children: Children) -> impl IntoView {
    view! {
        <nav class="breadcrumb" aria-label="Breadcrumb">
            <ol class="breadcrumb__list">
                {children()}
            </ol>
        </nav>
    }
}

#[component]
pub fn BreadcrumbItem(children: Children) -> impl IntoView {
    view! {
        <li class="breadcrumb__item">
            {children()}
        </li>
    }
}

#[component]
pub fn BreadcrumbLink(#[prop(into)] href: String, children: Children) -> impl IntoView {
    view! {
        <a class="breadcrumb__link" href=href>
            {children()}
        </a>
    }
}

#[component]
pub fn BreadcrumbPage(children: Children) -> impl IntoView {
    view! {
        <span class="breadcrumb__page" aria-current="page">
            {children()}
        </span>
    }
}

#[component]
pub fn BreadcrumbSeparator() -> impl IntoView {
    view! {
        <li class="breadcrumb__separator" aria-hidden="true">
            <ChevronRight class="breadcrumb__separator-icon"/>
        </li>
    }
}

#[component]
pub fn IconButton(label: &'static str, children: Children) -> impl IntoView {
    view! {
        <button class="icon-button" type="button" aria-label=label title=label>
            {children()}
        </button>
    }
}

#[component]
pub fn Timestamp(value: String) -> impl IntoView {
    let datetime = value.clone();
    let display_value = RwSignal::new(value.clone());

    #[cfg(feature = "hydrate")]
    Effect::new(move |_| {
        display_value.set(format_local_timestamp(&value));
    });

    view! {
        <time datetime=datetime>{move || display_value.get()}</time>
    }
}

#[cfg(feature = "hydrate")]
fn format_local_timestamp(value: &str) -> String {
    let date = js_sys::Date::new(&JsValue::from_str(value));
    if date.get_time().is_nan() {
        return value.to_string();
    }

    let month = date.get_month() + 1;
    let day = date.get_date();
    let year = date.get_full_year() % 100;
    let mut hour = date.get_hours();
    let minute = date.get_minutes();
    let second = date.get_seconds();
    let meridiem = if hour >= 12 { "PM" } else { "AM" };

    hour %= 12;
    if hour == 0 {
        hour = 12;
    }

    format!("{month:02}/{day:02}/{year:02} {hour:02}:{minute:02}:{second:02} {meridiem}")
}

#[cfg(feature = "hydrate")]
fn scroll_app_main_by(delta_y: f64) {
    let Some(scroller) = web_sys::window()
        .and_then(|window| window.document())
        .and_then(|document| document.query_selector(".app-main").ok().flatten())
        .and_then(|element| element.dyn_into::<web_sys::HtmlElement>().ok())
    else {
        return;
    };

    let next_scroll_top = (scroller.scroll_top() as f64 + delta_y).round().max(0.0) as i32;
    scroller.set_scroll_top(next_scroll_top);
}

#[component]
pub fn DropdownMenu(#[prop(into)] label: String, children: Children) -> impl IntoView {
    let is_open = RwSignal::new(false);
    let menu_class = move || {
        if is_open.get() {
            "dropdown-menu is-open"
        } else {
            "dropdown-menu"
        }
    };

    view! {
        <div class=menu_class>
            <button
                class="icon-button"
                type="button"
                aria-label=label.clone()
                title=label
                aria-haspopup="menu"
                aria-expanded=move || is_open.get().to_string()
                on:click=move |_| is_open.update(|open| *open = !*open)
            >
                <span aria-hidden="true">
                    <Ellipsis class="icon-button__icon"/>
                </span>
            </button>
            <button
                class="dropdown-menu__scrim"
                type="button"
                aria-label="Close menu"
                on:wheel=move |event| {
                    let _ = &event;
                    #[cfg(feature = "hydrate")]
                    {
                        event.prevent_default();
                        scroll_app_main_by(event.delta_y());
                    }
                }
                on:click=move |_| is_open.set(false)
            ></button>
            <div
                class="dropdown-menu__content blurred-surface"
                role="menu"
                on:click=move |_| is_open.set(false)
            >
                {children()}
            </div>
        </div>
    }
}

#[component]
pub fn Drawer(title: &'static str, children: Children) -> impl IntoView {
    view! {
        <section class="drawer blurred-surface" aria-label=title>
            <h2>{title}</h2>
            {children()}
        </section>
    }
}

#[component]
pub fn Sheet(title: &'static str, children: Children) -> impl IntoView {
    view! {
        <aside class="sheet blurred-surface" role="dialog" aria-label=title>
            <h2>{title}</h2>
            {children()}
        </aside>
    }
}

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

#[component]
pub fn SearchableDataTable(
    search_label: &'static str,
    placeholder: &'static str,
    search: RwSignal<String>,
    children: Children,
) -> impl IntoView {
    view! {
        <div class="searchable-data-table">
            <label class="searchable-data-table__search searchable-data-table__control">
                <Search class="searchable-data-table__control-icon"/>
                <span class="sr-only">{search_label}</span>
                <input
                    type="search"
                    placeholder=placeholder
                    prop:value=move || search.get()
                    on:input=move |event| search.set(event_target_value(&event))
                />
            </label>
            <DataTable>{children()}</DataTable>
        </div>
    }
}

#[component]
pub fn Tabs(active: RwSignal<String>, children: Children) -> impl IntoView {
    view! {
        <div class="tabs" data-active=move || active.get()>
            {children()}
        </div>
    }
}

#[component]
pub fn TabsList(children: Children) -> impl IntoView {
    view! {
        <div class="tabs-list" role="tablist">
            {children()}
        </div>
    }
}

#[component]
pub fn TabsTrigger(
    active: RwSignal<String>,
    value: &'static str,
    children: Children,
) -> impl IntoView {
    view! {
        <button
            class=move || {
                if active.get() == value {
                    "tabs-trigger is-active"
                } else {
                    "tabs-trigger"
                }
            }
            type="button"
            role="tab"
            aria-selected=move || (active.get() == value).to_string()
            on:click=move |_| active.set(value.to_string())
        >
            {children()}
        </button>
    }
}

#[component]
pub fn TabsContent(
    active: RwSignal<String>,
    value: &'static str,
    children: Children,
) -> impl IntoView {
    view! {
        <section
            class="tabs-content"
            role="tabpanel"
            hidden=move || active.get() != value
        >
            {children()}
        </section>
    }
}

#[component]
pub fn InfoListTable(children: Children) -> impl IntoView {
    view! {
        <table class="info-list-table">
            <tbody>{children()}</tbody>
        </table>
    }
}

#[component]
pub fn InfoRow(label: &'static str, value: &'static str) -> impl IntoView {
    view! {
        <tr>
            <th scope="row">{label}</th>
            <td>{value}</td>
        </tr>
    }
}

#[component]
pub fn StatusBadge(#[prop(into)] label: String) -> impl IntoView {
    let class = match label.as_str() {
        "Available" | "Done" | "Ready" | "Steps Complete" => "status-badge is-success",
        "Error" => "status-badge is-danger",
        "In Progress" => "status-badge is-warning",
        "Pending" => "status-badge is-info",
        _ => "status-badge is-info",
    };

    view! { <span class=class>{label}</span> }
}

#[component]
pub fn EmptyState(title: &'static str, message: &'static str) -> impl IntoView {
    view! {
        <section class="empty-state">
            <h3>{title}</h3>
            <p>{message}</p>
        </section>
    }
}
