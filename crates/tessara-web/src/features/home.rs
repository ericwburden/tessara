use leptos::prelude::*;

#[cfg(feature = "hydrate")]
use serde_json::{Value, json};

use crate::app::transitional::{TransitionalPage, render_transitional_route};
#[cfg(feature = "hydrate")]
use crate::features::native_runtime::{post_json, redirect};
use crate::features::native_shell::{
    ADMIN_LINKS, AccountContext, BreadcrumbItem, MetadataStrip, NativePage, NavLinkSpec,
    PRODUCT_LINKS, PageHeader, Panel, TRANSITIONAL_LINKS, use_account_session, visible_links,
};

fn directory_card(spec: &NavLinkSpec) -> AnyView {
    view! {
        <article class="directory-card card">
            <div class="card-content">
                <h3>{spec.label}</h3>
                <p>{spec.home_description}</p>
                <a class="button-link button is-light" href=spec.href>{spec.home_action_label}</a>
            </div>
        </article>
    }
    .into_any()
}

fn home_section_cards(
    loaded: bool,
    account: Option<&AccountContext>,
    links: &'static [NavLinkSpec],
) -> AnyView {
    if !loaded {
        return view! { <p class="muted">"Loading available destinations..."</p> }.into_any();
    }

    let visible = visible_links(loaded, account, links);

    if visible.is_empty() {
        return view! { <p class="muted">"No destinations are available for the current account."</p> }
            .into_any();
    }

    view! {
        <div class="home-grid">
            {visible
                .into_iter()
                .map(|link| directory_card(link))
                .collect_view()}
        </div>
    }
    .into_any()
}

fn home_visible_cards(loaded: bool, visible: Vec<&NavLinkSpec>) -> AnyView {
    if !loaded {
        return view! { <p class="muted">"Loading available destinations..."</p> }.into_any();
    }

    if visible.is_empty() {
        return view! { <p class="muted">"No destinations are available for the current account."</p> }
            .into_any();
    }

    view! {
        <div class="home-grid">
            {visible.into_iter().map(directory_card).collect_view()}
        </div>
    }
    .into_any()
}

#[component]
pub fn AdminWorkbenchPage() -> impl IntoView {
    render_transitional_route(TransitionalPage {
        title: "Tessara",
        description: "Tessara local admin workbench for migration setup and workflow testing.",
        body_html: crate::admin_shell_html(),
        page_key: "admin-shell",
        active_route: "administration",
        record_id: None,
    })
}

#[component]
pub fn HomePage() -> impl IntoView {
    let session = use_account_session();
    view! {
        <NativePage
            title="Tessara Home"
            description="Tessara application home for local replacement workflow testing."
            page_key="home"
            active_route="home"
            workspace_label="Shared Home"
            breadcrumbs=vec![BreadcrumbItem::current("Home")]
        >
            <PageHeader
                eyebrow="Shared Home"
                title="Application Overview"
                description="Use this shared home as the entry point for product areas, workflow queues, and internal workspaces."
            />
            <MetadataStrip items=vec![
                ("Mode", "Shared Home".into()),
                ("Surface", "Role-aware overview".into()),
                ("State", "Native SSR shell".into()),
            ]/>
            <Panel
                title="Role-Ready Home Modules"
                description="These modules define the shared home shape for admin, scoped-operator, and respondent variants."
            >
                <div class="home-grid">
                    <article class="home-card">
                        <h3>"Scoped Operations"</h3>
                        <p>"Organization and form access for partner, program, activity, and session-style work."</p>
                    </article>
                    <article class="home-card">
                        <h3>"Response Delivery"</h3>
                        <p>"Start, edit, submit, and review response work without exposing builder-first navigation."</p>
                    </article>
                    <article class="home-card">
                        <h3>"Oversight and Insight"</h3>
                        <p>"Dashboards, reports, datasets, and internal oversight remain available without collapsing back into a control console."</p>
                    </article>
                </div>
            </Panel>
            <Panel
                title="Product Areas"
                description="These are the primary destinations for top-level entity browsing and workflow entry."
            >
                {move || {
                    let account = session.account.get();
                    let visible = visible_links(session.loaded.get(), account.as_ref(), PRODUCT_LINKS)
                        .into_iter()
                        .filter(|link| link.key != "home" && link.key != "dashboards")
                        .collect::<Vec<_>>();
                    home_visible_cards(session.loaded.get(), visible)
                }}
            </Panel>
            <Panel
                title="Transitional Reporting"
                description="Reporting remains reachable while dashboards stay in the primary navigation and the target component model continues to replace the older asset shape."
            >
                {move || {
                    let account = session.account.get();
                    home_section_cards(session.loaded.get(), account.as_ref(), TRANSITIONAL_LINKS)
                }}
            </Panel>
            <Panel
                title="Current Deployment Readiness"
                description="Refresh Summary confirms the current stack has enough configured data for response, reporting, and dashboard workflows."
            >
                <div class="record-list">
                    <article class="record-card compact-record-card">
                        <h4>"Current Counters"</h4>
                        <p class="muted">"Runtime counters and readiness checks load after hydration."</p>
                    </article>
                </div>
            </Panel>
            <Panel
                title="Current Workflow Context"
                description="Current selections appear here and in the shared sidebar."
            >
                <div class="selection-grid">
                    <p class="muted">"No records selected yet."</p>
                </div>
            </Panel>
            <Panel
                title="Internal Workspaces"
                description="Internal Areas and reporting destinations remain available, but secondary to the main product journey."
            >
                {move || {
                    let account = session.account.get();
                    let loaded = session.loaded.get();
                    let mut visible = visible_links(loaded, account.as_ref(), ADMIN_LINKS);
                    visible.extend(visible_links(loaded, account.as_ref(), PRODUCT_LINKS)
                        .into_iter()
                        .filter(|link| link.key == "dashboards" || link.key == "components"));
                    home_visible_cards(loaded, visible)
                }}
            </Panel>
        </NativePage>
    }
}

#[component]
pub fn LoginPage() -> impl IntoView {
    let session = use_account_session();
    #[cfg(not(feature = "hydrate"))]
    let _ = session;
    let email = RwSignal::new(String::new());
    let password = RwSignal::new(String::new());
    let feedback = RwSignal::new(None::<String>);
    let busy = RwSignal::new(false);

    #[cfg(feature = "hydrate")]
    Effect::new(move |_| {
        if session.loaded.get() && session.account.get().is_some() {
            redirect("/app");
        }
    });

    let submit = move |_| {
        if busy.get_untracked() {
            return;
        }

        let email_value = email.get_untracked().trim().to_string();
        let password_value = password.get_untracked();
        if email_value.is_empty() || password_value.is_empty() {
            feedback.set(Some("Enter an email address and password.".into()));
            return;
        }

        busy.set(true);
        feedback.set(None);

        #[cfg(feature = "hydrate")]
        leptos::task::spawn_local(async move {
            let body = json!({
                "email": email_value,
                "password": password_value,
            });

            match post_json::<Value>("/api/auth/login", &body).await {
                Ok(_) => {
                    session.account.set(None);
                    session.error.set(None);
                    session.loaded.set(false);
                    redirect("/app");
                }
                Err(error) => feedback.set(Some(error)),
            }

            busy.set(false);
        });
    };

    view! {
        <NativePage
            title="Tessara Sign In"
            description="Sign in to the Tessara application shell."
            page_key="login"
            active_route="login"
            workspace_label="Sign In"
            allow_unauthenticated=true
            breadcrumbs=vec![BreadcrumbItem::current("Sign In")]
        >
            <PageHeader
                eyebrow="Access"
                title="Sign In"
                description="Use your Tessara account to open the native application shell."
            />
            <MetadataStrip items=vec![
                ("Mode", "Authentication".into()),
                ("Surface", "Native sign-in".into()),
                ("State", "Cookie session contract".into()),
            ]/>
            <Panel
                title="Application Sign-In"
                description="Enter your account credentials to continue. Development credentials are documented outside the public sign-in surface."
            >
                <form
                    id="login-form"
                    class="stacked-form"
                    on:submit=move |event| {
                        event.prevent_default();
                        submit(());
                    }
                >
                    <div id="login-feedback" class="notification is-light" class:is-hidden=move || feedback.get().is_none()>
                        {move || feedback.get().unwrap_or_default()}
                    </div>
                    <div class="form-grid">
                        <label class="field">
                            <span>"Email"</span>
                            <input
                                id="login-email"
                                class="input"
                                type="email"
                                autocomplete="username"
                                prop:value=email
                                on:input=move |event| email.set(event_target_value(&event))
                            />
                        </label>
                        <label class="field">
                            <span>"Password"</span>
                            <input
                                id="login-password"
                                class="input"
                                type="password"
                                autocomplete="current-password"
                                prop:value=password
                                on:input=move |event| password.set(event_target_value(&event))
                            />
                        </label>
                    </div>
                    <div class="actions">
                        <button class="button-link button is-light" type="submit" disabled=move || busy.get()>
                            {move || if busy.get() { "Signing In..." } else { "Sign In" }}
                        </button>
                    </div>
                </form>
            </Panel>
        </NativePage>
    }
}
