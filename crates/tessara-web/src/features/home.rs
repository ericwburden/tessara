use leptos::prelude::*;

use crate::app::transitional::{TransitionalPage, extract_app_root, render_transitional_route};
use crate::features::native_shell::{
    ANALYTICS_LINKS, AccountContext, BreadcrumbItem, INTERNAL_LINKS, MetadataStrip, NativePage,
    NavLinkSpec, PRODUCT_LINKS, PageHeader, Panel, TRANSITIONAL_LINKS, use_account_session,
    visible_links,
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
        body_html: extract_app_root(crate::admin_shell_html()),
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
                    let mut visible = visible_links(loaded, account.as_ref(), ANALYTICS_LINKS);
                    visible.extend(
                        visible_links(loaded, account.as_ref(), PRODUCT_LINKS)
                            .into_iter()
                            .filter(|link| link.key == "dashboards"),
                    );
                    visible.extend(visible_links(loaded, account.as_ref(), INTERNAL_LINKS));
                    home_visible_cards(loaded, visible)
                }}
            </Panel>
        </NativePage>
    }
}

#[component]
pub fn LoginPage() -> impl IntoView {
    render_transitional_route(TransitionalPage {
        title: "Tessara Sign In",
        description: "Sign in to the Tessara application shell.",
        body_html: extract_app_root(crate::login_application_html()),
        page_key: "login",
        active_route: "login",
        record_id: None,
    })
}
