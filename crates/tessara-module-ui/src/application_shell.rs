//! Canonical policy-neutral Tessara application shell presentation.

use icons::{Bell, CircleHelp, Menu, Moon, Sun};
use leptos::prelude::*;

#[cfg(all(feature = "hydrate", target_arch = "wasm32"))]
const STORAGE_KEY: &str = "tessara.themePreference";
#[cfg(all(feature = "hydrate", target_arch = "wasm32"))]
const LIGHT_THEME_COLOR: &str = "#F8FAFC";
#[cfg(all(feature = "hydrate", target_arch = "wasm32"))]
const DARK_THEME_COLOR: &str = "#0F172A";

/// Renders the canonical desktop and mobile shell around caller-owned
/// navigation policy and product content.
#[component]
pub fn ApplicationShell(
    #[prop(into)] title: String,
    navigation: ChildrenFn,
    children: Children,
) -> impl IntoView {
    #[cfg(all(feature = "hydrate", target_arch = "wasm32"))]
    {
        let document_title = title.clone();
        Effect::new(move |_| {
            if let Some(document) = web_sys::window().and_then(|window| window.document()) {
                document.set_title(&format!("{document_title} · Tessara"));
            }
        });
    }

    let desktop_navigation = navigation.clone();
    let mobile_navigation = navigation;
    view! {
        <main class="app-shell">
            <aside class="sidebar" aria-label="Primary navigation">
                {desktop_navigation()}
            </aside>
            <section class="app-main" aria-label="Application content">
                <header class="top-app-bar">
                    <div class="top-app-bar__title-row">
                        <MobileNavigation navigation=mobile_navigation/>
                        <span class="top-app-bar__title">{title}</span>
                    </div>
                    <div class="top-app-bar__actions">
                        <label class="search-field">
                            <span class="sr-only">"Search Tessara"</span>
                            <input type="search" placeholder="Search Tessara"/>
                        </label>
                        <ThemeToggle/>
                        <ShellIconButton label="Notifications">
                            <Bell class="icon-button__icon"/>
                        </ShellIconButton>
                        <ShellIconButton label="Help">
                            <CircleHelp class="icon-button__icon"/>
                        </ShellIconButton>
                    </div>
                </header>
                <div class="app-page">{children()}</div>
            </section>
        </main>
    }
}

#[component]
fn MobileNavigation(navigation: ChildrenFn) -> impl IntoView {
    let is_open = RwSignal::new(false);
    view! {
        <div class=move || if is_open.get() { "mobile-nav is-open" } else { "mobile-nav" }>
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
                {navigation()}
            </aside>
        </div>
    }
}

#[component]
fn ShellIconButton(label: &'static str, children: Children) -> impl IntoView {
    view! {
        <button class="icon-button" type="button" aria-label=label title=label>
            {children()}
        </button>
    }
}

#[component]
fn ThemeToggle() -> impl IntoView {
    let preference = RwSignal::new("system");
    let is_open = RwSignal::new(false);
    Effect::new(move |_| preference.set(read_theme_preference()));

    view! {
        <div class=move || if is_open.get() { "theme-toggle is-open" } else { "theme-toggle" }>
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
            <button class="theme-toggle__scrim" type="button" aria-label="Close theme options" on:click=move |_| is_open.set(false)></button>
            <div class="theme-toggle__menu blurred-surface" role="menu" aria-label="Theme options">
                {[("system", "System"), ("light", "Light"), ("dark", "Dark")]
                    .into_iter()
                    .map(|(value, label)| view! {
                        <button
                            class=move || if preference.get() == value { "theme-toggle__option is-active" } else { "theme-toggle__option" }
                            type="button"
                            role="menuitemradio"
                            aria-checked=move || (preference.get() == value).to_string()
                            on:click=move |_| {
                                preference.set(value);
                                set_theme_preference(value);
                                is_open.set(false);
                            }
                        >
                            {if value == "dark" {
                                view! { <Moon class="theme-toggle__option-icon"/> }.into_any()
                            } else {
                                view! { <Sun class="theme-toggle__option-icon"/> }.into_any()
                            }}
                            <span>{label}</span>
                        </button>
                    })
                    .collect_view()}
            </div>
        </div>
    }
}

fn read_theme_preference() -> &'static str {
    #[cfg(all(feature = "hydrate", target_arch = "wasm32"))]
    {
        return match web_sys::window()
            .and_then(|window| window.local_storage().ok().flatten())
            .and_then(|storage| storage.get_item(STORAGE_KEY).ok().flatten())
            .as_deref()
        {
            Some("light") => "light",
            Some("dark") => "dark",
            _ => "system",
        };
    }
    #[cfg(not(all(feature = "hydrate", target_arch = "wasm32")))]
    "system"
}

fn set_theme_preference(preference: &'static str) {
    #[cfg(all(feature = "hydrate", target_arch = "wasm32"))]
    {
        let Some(window) = web_sys::window() else {
            return;
        };
        if let Ok(Some(storage)) = window.local_storage() {
            let _ = storage.set_item(STORAGE_KEY, preference);
        }
        let resolved = if preference == "system" {
            window
                .match_media("(prefers-color-scheme: dark)")
                .ok()
                .flatten()
                .map(|query| if query.matches() { "dark" } else { "light" })
                .unwrap_or("light")
        } else {
            preference
        };
        let Some(document) = window.document() else {
            return;
        };
        if let Some(root) = document.document_element() {
            let _ = root.set_attribute("data-theme-preference", preference);
            let _ = root.set_attribute("data-theme", resolved);
        }
        if let Ok(Some(meta)) = document.query_selector("meta[name=\"theme-color\"]") {
            let color = if resolved == "dark" {
                DARK_THEME_COLOR
            } else {
                LIGHT_THEME_COLOR
            };
            let _ = meta.set_attribute("content", color);
        }
    }
    #[cfg(not(all(feature = "hydrate", target_arch = "wasm32")))]
    let _ = preference;
}
