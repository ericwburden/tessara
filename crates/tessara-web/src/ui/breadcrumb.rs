//! Owns the ui::breadcrumb module behavior.

use icons::ChevronRight;
use leptos::prelude::*;

#[component]
/// Renders the breadcrumb view.
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
/// Renders the breadcrumb item view.
pub fn BreadcrumbItem(children: Children) -> impl IntoView {
    view! {
        <li class="breadcrumb__item">
            {children()}
        </li>
    }
}

#[component]
/// Renders the breadcrumb link view.
pub fn BreadcrumbLink(#[prop(into)] href: String, children: Children) -> impl IntoView {
    view! {
        <a class="breadcrumb__link" href=href>
            {children()}
        </a>
    }
}

#[component]
/// Renders the breadcrumb page view.
pub fn BreadcrumbPage(children: Children) -> impl IntoView {
    view! {
        <span class="breadcrumb__page" aria-current="page">
            {children()}
        </span>
    }
}

#[component]
/// Renders the breadcrumb separator view.
pub fn BreadcrumbSeparator() -> impl IntoView {
    view! {
        <li class="breadcrumb__separator" aria-hidden="true">
            <ChevronRight class="breadcrumb__separator-icon"/>
        </li>
    }
}
