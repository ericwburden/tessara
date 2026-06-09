use icons::ChevronRight;
use leptos::prelude::*;

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
