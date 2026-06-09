use icons::{Bell, CircleHelp, Moon, Sun};
use leptos::prelude::*;
use super::MobileNav;

#[cfg(feature = "hydrate")]
use crate::state::theme::{DARK_THEME_COLOR, LIGHT_THEME_COLOR, STORAGE_KEY};

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
pub fn IconButton(label: &'static str, children: Children) -> impl IntoView {
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
                    <Sun class="theme-toggle__option-icon"/>
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
