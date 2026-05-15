use icons::{
    Bell, Blocks, ChevronRight, CircleHelp, ClipboardCheck, Database, Ellipsis, File, FileText,
    GitBranch, House, LayoutDashboard, LogOut, Menu, Monitor, Moon, Search, SlidersHorizontal, Sun,
    Truck, Workflow,
};
use leptos::prelude::*;
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
}

const NAV_ITEMS: [NavItem; 10] = [
    NavItem {
        key: "home",
        href: "/",
        label: "Home",
        section: "Main",
    },
    NavItem {
        key: "organization",
        href: "/organization",
        label: "Organization",
        section: "Main",
    },
    NavItem {
        key: "forms",
        href: "/forms",
        label: "Forms",
        section: "Main",
    },
    NavItem {
        key: "workflows",
        href: "/workflows",
        label: "Workflows",
        section: "Main",
    },
    NavItem {
        key: "responses",
        href: "/responses",
        label: "Responses",
        section: "Main",
    },
    NavItem {
        key: "components",
        href: "/components",
        label: "Components",
        section: "Main",
    },
    NavItem {
        key: "dashboards",
        href: "/dashboards",
        label: "Dashboards",
        section: "Main",
    },
    NavItem {
        key: "datasets",
        href: "/datasets",
        label: "Datasets",
        section: "Admin",
    },
    NavItem {
        key: "administration",
        href: "/administration",
        label: "Administration",
        section: "Admin",
    },
    NavItem {
        key: "migration",
        href: "/migration",
        label: "Migration",
        section: "Admin",
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

            let authenticated = match response {
                Ok(response) if response.ok() => response
                    .json::<SessionStateResponse>()
                    .await
                    .map(|session| session.authenticated)
                    .unwrap_or(false),
                _ => false,
            };

            if !authenticated {
                if let Some(window) = web_sys::window() {
                    let _ = window.location().set_href("/login");
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
}

#[component]
pub fn Sidebar(active_route: &'static str) -> impl IntoView {
    let main_items = nav_items_for("Main", active_route);
    let admin_items = nav_items_for("Admin", active_route);

    view! {
        <aside class="sidebar" aria-label="Primary navigation">
            <SidebarContent main_items admin_items/>
        </aside>
    }
}

#[component]
fn SidebarContent(
    main_items: impl IntoView + 'static,
    admin_items: impl IntoView + 'static,
) -> impl IntoView {
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
            <p class="sidebar-section">"Main"</p>
            {main_items}
            <p class="sidebar-section">"Admin"</p>
            {admin_items}
        </nav>
        <section class="account-card" aria-label="Account context">
            <span class="account-avatar">"TA"</span>
            <span class="account-copy">
                <strong>"Tessara Admin"</strong>
                <small>"admin@tessara.local"</small>
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

fn nav_items_for(section: &'static str, active_route: &'static str) -> impl IntoView {
    NAV_ITEMS
        .iter()
        .filter(move |item| item.section == section)
        .map(move |item| {
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
        })
        .collect_view()
}

#[component]
fn NavIcon(route_key: &'static str) -> impl IntoView {
    match route_key {
        "home" => view! { <span class="sidebar-link__icon-wrap" aria-hidden="true"><House class="sidebar-link__icon"/></span> }.into_any(),
        "organization" => view! { <span class="sidebar-link__icon-wrap" aria-hidden="true"><GitBranch class="sidebar-link__icon"/></span> }.into_any(),
        "forms" => view! { <span class="sidebar-link__icon-wrap" aria-hidden="true"><FileText class="sidebar-link__icon"/></span> }.into_any(),
        "workflows" => view! { <span class="sidebar-link__icon-wrap" aria-hidden="true"><Workflow class="sidebar-link__icon"/></span> }.into_any(),
        "responses" => view! { <span class="sidebar-link__icon-wrap" aria-hidden="true"><ClipboardCheck class="sidebar-link__icon"/></span> }.into_any(),
        "components" => view! { <span class="sidebar-link__icon-wrap" aria-hidden="true"><Blocks class="sidebar-link__icon"/></span> }.into_any(),
        "dashboards" => view! { <span class="sidebar-link__icon-wrap" aria-hidden="true"><LayoutDashboard class="sidebar-link__icon"/></span> }.into_any(),
        "datasets" => view! { <span class="sidebar-link__icon-wrap" aria-hidden="true"><Database class="sidebar-link__icon"/></span> }.into_any(),
        "administration" => view! { <span class="sidebar-link__icon-wrap" aria-hidden="true"><SlidersHorizontal class="sidebar-link__icon"/></span> }.into_any(),
        "migration" => view! { <span class="sidebar-link__icon-wrap" aria-hidden="true"><Truck class="sidebar-link__icon"/></span> }.into_any(),
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
    let main_items = nav_items_for("Main", active_route);
    let admin_items = nav_items_for("Admin", active_route);
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
                <SidebarContent main_items admin_items/>
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
pub fn StatusBadge(label: &'static str) -> impl IntoView {
    let class = match label {
        "Done" => "status-badge is-success",
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
