use leptos::prelude::*;

use crate::features::native_shell::{
    BreadcrumbItem, DelegationSummary, MetadataStrip, NativePage, PageHeader, Panel,
    ScopeNodeSummary, UiAccessProfile,
};
use crate::infra::routing::{
    AccountRouteParams, NodeTypeRouteParams, RoleRouteParams, require_route_params,
};

#[cfg(feature = "hydrate")]
use crate::features::native_runtime::{get_json, post_json, put_json, redirect};
use serde::Deserialize;
#[cfg(feature = "hydrate")]
use serde_json::json;
#[cfg(feature = "hydrate")]
use wasm_bindgen_futures::spawn_local;

#[derive(Clone, Deserialize)]
struct IdResponse {
    id: String,
}

#[derive(Clone, Deserialize)]
struct RoleSummary {
    id: String,
    name: String,
    capability_count: i64,
    account_count: i64,
}

#[derive(Clone, Deserialize)]
struct CapabilitySummary {
    id: String,
    key: String,
    description: String,
}

#[derive(Clone, Deserialize)]
struct AccountAssignmentSummary {
    account_id: String,
    email: String,
    display_name: String,
}

#[derive(Clone, Deserialize)]
struct RoleDetail {
    id: String,
    name: String,
    capabilities: Vec<CapabilitySummary>,
    assigned_accounts: Vec<AccountAssignmentSummary>,
}

#[derive(Clone, Deserialize)]
struct UserSummary {
    id: String,
    email: String,
    display_name: String,
    is_active: bool,
    roles: Vec<RoleSummary>,
}

#[derive(Clone, Deserialize)]
struct UserDetail {
    id: String,
    email: String,
    display_name: String,
    is_active: bool,
    ui_access_profile: UiAccessProfile,
    capabilities: Vec<String>,
    roles: Vec<RoleSummary>,
    scope_nodes: Vec<ScopeNodeSummary>,
    delegations: Vec<DelegationSummary>,
    delegated_by: Vec<DelegationSummary>,
}

#[derive(Clone, Deserialize)]
struct UserAccessDetail {
    account_id: String,
    email: String,
    display_name: String,
    ui_access_profile: UiAccessProfile,
    capabilities: Vec<String>,
    scope_nodes: Vec<ScopeNodeSummary>,
    available_scope_nodes: Vec<ScopeNodeSummary>,
    delegations: Vec<DelegationSummary>,
    available_delegate_accounts: Vec<DelegationSummary>,
    scope_assignments_editable: bool,
    delegation_assignments_editable: bool,
}

#[derive(Clone, Deserialize)]
struct NodeTypeSummary {
    id: String,
    name: String,
    slug: String,
    singular_label: String,
    plural_label: String,
    is_root_type: bool,
    node_count: i64,
}

#[derive(Clone, Deserialize)]
struct NodeTypePeerLink {
    node_type_id: String,
    singular_label: String,
    plural_label: String,
}

#[derive(Clone, Deserialize)]
struct NodeMetadataFieldSummary {
    id: String,
    key: String,
    label: String,
    field_type: String,
    required: bool,
}

#[derive(Clone, Deserialize)]
struct NodeTypeFormLink {
    form_id: String,
    form_name: String,
    form_slug: String,
}

#[derive(Clone, Deserialize)]
struct NodeTypeDefinition {
    id: String,
    name: String,
    slug: String,
    singular_label: String,
    plural_label: String,
    is_root_type: bool,
    node_count: i64,
    parent_relationships: Vec<NodeTypePeerLink>,
    child_relationships: Vec<NodeTypePeerLink>,
    metadata_fields: Vec<NodeMetadataFieldSummary>,
    scoped_forms: Vec<NodeTypeFormLink>,
}

fn access_profile_label(profile: &UiAccessProfile) -> &'static str {
    match profile {
        UiAccessProfile::Admin => "Admin",
        UiAccessProfile::Operator => "Operator",
        UiAccessProfile::ResponseUser => "Response User",
    }
}

fn node_scope_label(node: &ScopeNodeSummary) -> String {
    match node.parent_node_name.as_deref() {
        Some(parent) => format!(
            "{} ({}, under {})",
            node.node_name, node.node_type_name, parent
        ),
        None => format!("{} ({})", node.node_name, node.node_type_name),
    }
}

fn delegation_label(item: &DelegationSummary) -> String {
    format!("{} ({})", item.display_name, item.email)
}

fn toggle_selection(values: &mut Vec<String>, id: &str, checked: bool) {
    if checked {
        if !values.iter().any(|value| value == id) {
            values.push(id.to_string());
        }
    } else {
        values.retain(|value| value != id);
    }
}

#[component]
pub fn AdministrationPage() -> impl IntoView {
    view! {
        <NativePage
            title="Administration Workspace"
            description="Tessara internal administration landing page."
            page_key="administration"
            active_route="administration"
            workspace_label="Internal Area"
            required_capability="admin:all"
            breadcrumbs=vec![
                BreadcrumbItem::link("Home", "/app"),
                BreadcrumbItem::current("Administration"),
            ]
        >
            <section class="app-screen box entity-page admin-workspace-shell">
                <div class="admin-workspace-intro">
                    <div class="admin-workspace-intro__copy">
                        <p class="eyebrow">"Internal Area"</p>
                        <h1>"Administration Workspace"</h1>
                        <p class="muted">
                            "Keep account access, role bundles, hierarchy definitions, and migration entry points together in one deliberate internal workspace."
                        </p>
                    </div>
                    <div class="actions">
                        <a class="button-link button is-light" href="/app/migration">"Open Migration"</a>
                    </div>
                </div>
                <div class="admin-workspace-grid">
                    <article class="admin-workspace-card">
                        <p class="eyebrow">"Accounts"</p>
                        <h3>"User Management"</h3>
                        <p>"Manage application accounts, assigned roles, and scoped access."</p>
                        <div class="actions">
                            <a class="button-link" href="/app/administration/users">"Open Users"</a>
                        </div>
                    </article>
                    <article class="admin-workspace-card">
                        <p class="eyebrow">"Authorization"</p>
                        <h3>"Role Management"</h3>
                        <p>"Review capability bundles and the accounts currently assigned to them."</p>
                        <div class="actions">
                            <a class="button-link" href="/app/administration/roles">"Open Roles"</a>
                        </div>
                    </article>
                    <article class="admin-workspace-card">
                        <p class="eyebrow">"Hierarchy"</p>
                        <h3>"Organization Node Types"</h3>
                        <p>"Inspect hierarchy rules, metadata fields, and form scoping constraints."</p>
                        <div class="actions">
                            <a class="button-link" href="/app/administration/node-types">"Open Node Types"</a>
                        </div>
                    </article>
                    <article class="admin-workspace-card admin-workspace-card--subtle">
                        <p class="eyebrow">"Migration"</p>
                        <h3>"Legacy Validation"</h3>
                        <p>"Legacy fixture validation and import rehearsal stay available as a subordinate internal route."</p>
                        <div class="actions">
                            <a class="button-link button is-light" href="/app/migration">"Open Migration"</a>
                        </div>
                    </article>
                </div>
            </section>
        </NativePage>
    }
}

#[component]
pub fn UsersPage() -> impl IntoView {
    let users = RwSignal::new(Vec::<UserSummary>::new());
    let loading = RwSignal::new(true);
    let error = RwSignal::new(None::<String>);

    #[cfg(feature = "hydrate")]
    Effect::new(move |_| {
        spawn_local(async move {
            loading.set(true);
            match get_json::<Vec<UserSummary>>("/api/admin/users").await {
                Ok(items) => {
                    users.set(items);
                    error.set(None);
                }
                Err(message) => error.set(Some(message)),
            }
            loading.set(false);
        });
    });

    view! {
        <NativePage
            title="Users"
            description="Browse and manage Tessara user accounts."
            page_key="user-list"
            active_route="administration"
            workspace_label="Internal Area"
            required_capability="admin:all"
            breadcrumbs=vec![
                BreadcrumbItem::link("Home", "/app"),
                BreadcrumbItem::link("Administration", "/app/administration"),
                BreadcrumbItem::current("Users"),
            ]
        >
            <PageHeader
                eyebrow="Internal Area"
                title="User Management"
                description="Manage application users, active status, and assigned roles from dedicated native screens."
            />
            <MetadataStrip items=vec![
                ("Mode", "Directory".into()),
                ("Surface", "Account administration".into()),
                ("State", "Native SSR shell".into()),
            ]/>
            <Panel title="Accounts" description="Application accounts and their current role assignments appear here.">
                <div id="admin-user-list" class="record-list">
                    <Show
                        when=move || !loading.get()
                        fallback=|| view! { <p class="muted">"Loading users..."</p> }
                    >
                        {move || {
                            if let Some(message) = error.get() {
                                return view! { <p class="muted">{message}</p> }.into_any();
                            }
                            let items = users.get();
                            if items.is_empty() {
                                return view! { <p class="muted">"No application users are configured."</p> }.into_any();
                            }
                            view! {
                                {items
                                    .into_iter()
                                    .map(|user| {
                                        let detail_href = format!("/app/administration/users/{}", user.id);
                                        let edit_href = format!("{detail_href}/edit");
                                        let access_href = format!("{detail_href}/access");
                                        let role_summary = if user.roles.is_empty() {
                                            "No roles".to_string()
                                        } else {
                                            user.roles
                                                .iter()
                                                .map(|role| role.name.clone())
                                                .collect::<Vec<_>>()
                                                .join(", ")
                                        };
                                        view! {
                                            <article class="record-card">
                                                <h4>{user.display_name}</h4>
                                                <p>{user.email}</p>
                                                <p class="muted">{format!("Status: {}", if user.is_active { "active" } else { "inactive" })}</p>
                                                <p class="muted">{format!("Roles: {role_summary}")}</p>
                                                <div class="actions">
                                                    <a class="button-link" href=detail_href.clone()>"View"</a>
                                                    <a class="button-link button is-light" href=edit_href>"Edit"</a>
                                                    <a class="button-link button is-light" href=access_href>"Access"</a>
                                                </div>
                                            </article>
                                        }
                                    })
                                    .collect_view()}
                            }.into_any()
                        }}
                    </Show>
                </div>
            </Panel>
        </NativePage>
    }
}

#[component]
pub fn UserCreatePage() -> impl IntoView {
    user_form_page(None)
}

#[component]
pub fn UserDetailPage() -> impl IntoView {
    let AccountRouteParams { account_id } = require_route_params();
    let detail = RwSignal::new(None::<UserDetail>);
    let loading = RwSignal::new(true);
    let error = RwSignal::new(None::<String>);
    let record_id = account_id.clone();

    let _account_id_for_load = account_id.clone();
    #[cfg(feature = "hydrate")]
    Effect::new(move |_| {
        let account_id = _account_id_for_load.clone();
        spawn_local(async move {
            loading.set(true);
            match get_json::<UserDetail>(&format!("/api/admin/users/{account_id}")).await {
                Ok(user) => {
                    detail.set(Some(user));
                    error.set(None);
                }
                Err(message) => error.set(Some(message)),
            }
            loading.set(false);
        });
    });

    view! {
        <NativePage
            title="User Detail"
            description="Inspect a Tessara application account."
            page_key="user-detail"
            active_route="administration"
            workspace_label="Internal Area"
            record_id=record_id.clone()
            required_capability="admin:all"
            breadcrumbs=vec![
                BreadcrumbItem::link("Home", "/app"),
                BreadcrumbItem::link("Administration", "/app/administration"),
                BreadcrumbItem::link("Users", "/app/administration/users"),
                BreadcrumbItem::current("User Detail"),
            ]
        >
            <PageHeader
                eyebrow="Internal Area"
                title="User Detail"
                description="Inspect the selected account, its role grants, and current scope/delegation state."
            />
            <MetadataStrip items=vec![
                ("Mode", "Detail".into()),
                ("Surface", "Account inspection".into()),
                ("State", "Native SSR shell".into()),
            ]/>
            <Panel title="Account Summary" description="Identity, role, and capability information appears here.">
                <Show when=move || !loading.get() fallback=|| view! { <p class="muted">"Loading account detail..."</p> }>
                    {move || {
                        if let Some(message) = error.get() {
                            return view! { <p class="muted">{message}</p> }.into_any();
                        }
                        match detail.get() {
                            Some(user) => view! {
                                <div id="user-detail-summary" class="record-list">
                                    <article class="record-card">
                                        <h4>{user.display_name.clone()}</h4>
                                        <p>{user.email.clone()}</p>
                                        <p class="muted">{format!("Status: {}", if user.is_active { "active" } else { "inactive" })}</p>
                                        <p class="muted">{format!("UI access profile: {}", access_profile_label(&user.ui_access_profile))}</p>
                                    </article>
                                </div>
                                <div class="detail-grid">
                                    <section class="detail-section box">
                                        <h4>"Assigned Roles"</h4>
                                        <ul class="app-list">
                                            {if user.roles.is_empty() {
                                                view! { <li class="muted">"No roles assigned."</li> }.into_any()
                                            } else {
                                                view! { {user.roles.into_iter().map(|role| view! { <li>{role.name}</li> }).collect_view()} }.into_any()
                                            }}
                                        </ul>
                                    </section>
                                    <section class="detail-section box">
                                        <h4>"Effective Capabilities"</h4>
                                        <ul class="app-list">
                                            {if user.capabilities.is_empty() {
                                                view! { <li class="muted">"No capabilities resolved."</li> }.into_any()
                                            } else {
                                                view! { {user.capabilities.into_iter().map(|capability| view! { <li>{capability}</li> }).collect_view()} }.into_any()
                                            }}
                                        </ul>
                                    </section>
                                    <section class="detail-section box">
                                        <h4>"Scope Assignments"</h4>
                                        <ul class="app-list">
                                            {if user.scope_nodes.is_empty() {
                                                view! { <li class="muted">"No scope nodes assigned."</li> }.into_any()
                                            } else {
                                                view! { {user.scope_nodes.into_iter().map(|node| view! { <li>{node_scope_label(&node)}</li> }).collect_view()} }.into_any()
                                            }}
                                        </ul>
                                    </section>
                                    <section class="detail-section box">
                                        <h4>"Delegations"</h4>
                                        <ul class="app-list">
                                            {if user.delegations.is_empty() {
                                                view! { <li class="muted">"No delegate accounts assigned."</li> }.into_any()
                                            } else {
                                                view! { {user.delegations.into_iter().map(|account| view! { <li>{delegation_label(&account)}</li> }).collect_view()} }.into_any()
                                            }}
                                        </ul>
                                    </section>
                                    <section class="detail-section box">
                                        <h4>"Delegated By"</h4>
                                        <ul class="app-list">
                                            {if user.delegated_by.is_empty() {
                                                view! { <li class="muted">"No delegators currently target this account."</li> }.into_any()
                                            } else {
                                                view! { {user.delegated_by.into_iter().map(|account| view! { <li>{delegation_label(&account)}</li> }).collect_view()} }.into_any()
                                            }}
                                        </ul>
                                    </section>
                                </div>
                            }.into_any(),
                            None => view! { <p class="muted">"Account detail is unavailable."</p> }.into_any(),
                        }
                    }}
                </Show>
            </Panel>
        </NativePage>
    }
}

#[component]
pub fn UserEditPage() -> impl IntoView {
    let AccountRouteParams { account_id } = require_route_params();
    user_form_page(Some(account_id))
}

#[component]
pub fn UserAccessPage() -> impl IntoView {
    let AccountRouteParams { account_id } = require_route_params();
    let detail = RwSignal::new(None::<UserAccessDetail>);
    let selected_scope_ids = RwSignal::new(Vec::<String>::new());
    let selected_delegate_ids = RwSignal::new(Vec::<String>::new());
    let loading = RwSignal::new(true);
    let busy = RwSignal::new(false);
    let error = RwSignal::new(None::<String>);
    let status = RwSignal::new(None::<String>);
    let record_id = account_id.clone();

    let _account_id_for_load = account_id.clone();
    #[cfg(feature = "hydrate")]
    Effect::new(move |_| {
        let account_id = _account_id_for_load.clone();
        spawn_local(async move {
            loading.set(true);
            match get_json::<UserAccessDetail>(&format!("/api/admin/users/{account_id}/access"))
                .await
            {
                Ok(payload) => {
                    selected_scope_ids.set(
                        payload
                            .scope_nodes
                            .iter()
                            .map(|node| node.node_id.clone())
                            .collect(),
                    );
                    selected_delegate_ids.set(
                        payload
                            .delegations
                            .iter()
                            .map(|item| item.account_id.clone())
                            .collect(),
                    );
                    detail.set(Some(payload));
                    error.set(None);
                    status.set(Some("Access assignments loaded.".into()));
                }
                Err(message) => error.set(Some(message)),
            }
            loading.set(false);
        });
    });

    let cancel_href = StoredValue::new(format!("/app/administration/users/{record_id}"));
    let _account_id_for_submit = account_id.clone();
    let submit = StoredValue::new(std::sync::Arc::new(
        move |event: leptos::ev::SubmitEvent| {
            event.prevent_default();
            if busy.get_untracked() {
                return;
            }
            busy.set(true);
            error.set(None);
            status.set(Some("Saving access assignments...".into()));

            #[cfg(feature = "hydrate")]
            {
                let account_id = _account_id_for_submit.clone();
                let scope_ids = selected_scope_ids.get_untracked();
                let delegate_ids = selected_delegate_ids.get_untracked();
                spawn_local(async move {
                    let payload = json!({
                        "scope_node_ids": scope_ids,
                        "delegate_account_ids": delegate_ids,
                    });
                    match put_json::<IdResponse>(
                        &format!("/api/admin/users/{account_id}/access"),
                        &payload,
                    )
                    .await
                    {
                        Ok(_) => redirect(&format!("/app/administration/users/{account_id}")),
                        Err(message) => {
                            error.set(Some(message));
                            status.set(Some("Unable to save access assignments.".into()));
                        }
                    }
                    busy.set(false);
                });
            }
        },
    ));

    view! {
        <NativePage
            title="User Access"
            description="Manage scoped access assignments for a Tessara application account."
            page_key="user-access"
            active_route="administration"
            workspace_label="Internal Area"
            record_id=record_id.clone()
            required_capability="admin:all"
            breadcrumbs=vec![
                BreadcrumbItem::link("Home", "/app"),
                BreadcrumbItem::link("Administration", "/app/administration"),
                BreadcrumbItem::link("Users", "/app/administration/users"),
                BreadcrumbItem::link("User Detail", format!("/app/administration/users/{record_id}")),
                BreadcrumbItem::current("Access"),
            ]
        >
            <PageHeader
                eyebrow="Internal Area"
                title="User Access"
                description="Manage scope-node and delegate assignments from a dedicated native screen."
            />
            <MetadataStrip items=vec![
                ("Mode", "Edit".into()),
                ("Surface", "Access assignments".into()),
                ("State", "Native SSR shell".into()),
            ]/>
            <Panel title="Access Assignments" description="Scope and delegation changes save here.">
                <Show when=move || !loading.get() fallback=|| view! { <p class="muted">"Loading access assignments..."</p> }>
                    {move || {
                        if let Some(message) = error.get() {
                            return view! { <p class="muted">{message}</p> }.into_any();
                        }
                        match detail.get() {
                            Some(detail) => view! {
                                <form id="user-access-form" class="entity-form" on:submit=move |event| submit.with_value(|submit| submit(event))>
                                    <div class="detail-grid">
                                        <section class="detail-section box">
                                            <h4>{format!("{} ({})", detail.display_name, detail.email)}</h4>
                                            <p class="muted">{format!("UI access profile: {}", access_profile_label(&detail.ui_access_profile))}</p>
                                            <p class="muted">{format!("Effective capabilities: {}", detail.capabilities.join(", "))}</p>
                                        </section>
                                        <section class="detail-section box">
                                            <h4>"Scope Assignments"</h4>
                                            {if detail.scope_assignments_editable {
                                                view! {
                                                    <div class="selection-grid">
                                                        {detail
                                                            .available_scope_nodes
                                                            .into_iter()
                                                            .map(|node| {
                                                                let node_id = node.node_id.clone();
                                                                let changed_node_id = node_id.clone();
                                                                view! {
                                                                    <label class="selection-item box">
                                                                        <input
                                                                            type="checkbox"
                                                                            prop:checked=move || selected_scope_ids.get().iter().any(|value| value == &node_id)
                                                                            on:change=move |event| {
                                                                                let checked = event_target_checked(&event);
                                                                                selected_scope_ids.update(|values| toggle_selection(values, &changed_node_id, checked));
                                                                            }
                                                                        />
                                                                        <span>{node_scope_label(&node)}</span>
                                                                    </label>
                                                                }
                                                            })
                                                            .collect_view()}
                                                    </div>
                                                }.into_any()
                                            } else {
                                                view! { <p class="muted">"Scope assignments are not meaningful for this account profile."</p> }.into_any()
                                            }}
                                        </section>
                                        <section class="detail-section box">
                                            <h4>"Delegations"</h4>
                                            {if detail.delegation_assignments_editable {
                                                view! {
                                                    <div class="selection-grid">
                                                        {detail
                                                            .available_delegate_accounts
                                                            .into_iter()
                                                            .map(|account| {
                                                                let account_id = account.account_id.clone();
                                                                let changed_account_id = account_id.clone();
                                                                view! {
                                                                    <label class="selection-item box">
                                                                        <input
                                                                            type="checkbox"
                                                                            prop:checked=move || selected_delegate_ids.get().iter().any(|value| value == &account_id)
                                                                            on:change=move |event| {
                                                                                let checked = event_target_checked(&event);
                                                                                selected_delegate_ids.update(|values| toggle_selection(values, &changed_account_id, checked));
                                                                            }
                                                                        />
                                                                        <span>{delegation_label(&account)}</span>
                                                                    </label>
                                                                }
                                                            })
                                                            .collect_view()}
                                                    </div>
                                                }.into_any()
                                            } else {
                                                view! { <p class="muted">"Delegation assignments are not editable on this account."</p> }.into_any()
                                            }}
                                        </section>
                                    </div>
                                    <p id="user-access-status" class="muted">
                                        {move || error.get().or_else(|| status.get()).unwrap_or_else(|| "Update scope and delegation assignments here.".into())}
                                    </p>
                                    <div class="actions">
                                        <button class="button-link" type="submit" disabled=move || busy.get()>
                                            {move || if busy.get() { "Saving..." } else { "Save Access" }}
                                        </button>
                                        <a class="button-link button is-light" href=move || cancel_href.get_value()>"Cancel"</a>
                                    </div>
                                </form>
                            }.into_any(),
                            None => view! { <p class="muted">"Access detail is unavailable."</p> }.into_any(),
                        }
                    }}
                </Show>
            </Panel>
        </NativePage>
    }
}

#[component]
pub fn RolesPage() -> impl IntoView {
    let roles = RwSignal::new(Vec::<RoleSummary>::new());
    let loading = RwSignal::new(true);
    let error = RwSignal::new(None::<String>);

    #[cfg(feature = "hydrate")]
    Effect::new(move |_| {
        spawn_local(async move {
            loading.set(true);
            match get_json::<Vec<RoleSummary>>("/api/admin/roles").await {
                Ok(items) => {
                    roles.set(items);
                    error.set(None);
                }
                Err(message) => error.set(Some(message)),
            }
            loading.set(false);
        });
    });

    view! {
        <NativePage
            title="Roles"
            description="Browse and inspect Tessara role bundles."
            page_key="role-list"
            active_route="administration"
            workspace_label="Internal Area"
            required_capability="admin:all"
            breadcrumbs=vec![
                BreadcrumbItem::link("Home", "/app"),
                BreadcrumbItem::link("Administration", "/app/administration"),
                BreadcrumbItem::current("Roles"),
            ]
        >
            <PageHeader
                eyebrow="Internal Area"
                title="Roles"
                description="Review the current role bundles and the capabilities each one grants."
            />
            <MetadataStrip items=vec![
                ("Mode", "Directory".into()),
                ("Surface", "Role administration".into()),
                ("State", "Native SSR shell".into()),
            ]/>
            <Panel title="Role Catalog" description="Role bundles and assignment counts appear here.">
                <div id="admin-role-list" class="record-list">
                    <Show when=move || !loading.get() fallback=|| view! { <p class="muted">"Loading roles..."</p> }>
                        {move || {
                            if let Some(message) = error.get() {
                                return view! { <p class="muted">{message}</p> }.into_any();
                            }
                            let items = roles.get();
                            if items.is_empty() {
                                return view! { <p class="muted">"No roles are configured."</p> }.into_any();
                            }
                            view! {
                                {items
                                    .into_iter()
                                    .map(|role| {
                                        let detail_href = format!("/app/administration/roles/{}", role.id);
                                        let edit_href = format!("{detail_href}/edit");
                                        view! {
                                            <article class="record-card">
                                                <h4>{role.name}</h4>
                                                <p>{format!("{} capabilities", role.capability_count)}</p>
                                                <p class="muted">{format!("{} assigned accounts", role.account_count)}</p>
                                                <div class="actions">
                                                    <a class="button-link" href=detail_href.clone()>"View"</a>
                                                    <a class="button-link button is-light" href=edit_href>"Edit"</a>
                                                </div>
                                            </article>
                                        }
                                    })
                                    .collect_view()}
                            }.into_any()
                        }}
                    </Show>
                </div>
            </Panel>
        </NativePage>
    }
}

#[component]
pub fn NodeTypesPage() -> impl IntoView {
    let node_types = RwSignal::new(Vec::<NodeTypeSummary>::new());
    let loading = RwSignal::new(true);
    let error = RwSignal::new(None::<String>);

    #[cfg(feature = "hydrate")]
    Effect::new(move |_| {
        spawn_local(async move {
            loading.set(true);
            match get_json::<Vec<NodeTypeSummary>>("/api/admin/node-types").await {
                Ok(items) => {
                    node_types.set(items);
                    error.set(None);
                }
                Err(message) => error.set(Some(message)),
            }
            loading.set(false);
        });
    });

    view! {
        <NativePage
            title="Node Types"
            description="Browse and manage Tessara organization node types."
            page_key="node-type-list"
            active_route="administration"
            workspace_label="Internal Area"
            required_capability="admin:all"
            breadcrumbs=vec![
                BreadcrumbItem::link("Home", "/app"),
                BreadcrumbItem::link("Administration", "/app/administration"),
                BreadcrumbItem::current("Organization Node Types"),
            ]
        >
            <PageHeader
                eyebrow="Internal Area"
                title="Organization Node Types"
                description="Manage organization node-type naming and hierarchy rules from dedicated native screens."
            />
            <MetadataStrip items=vec![
                ("Mode", "Directory".into()),
                ("Surface", "Hierarchy administration".into()),
                ("State", "Native SSR shell".into()),
            ]/>
            <Panel title="Node Type Catalog" description="Configured node types and current usage counts appear here.">
                <div id="admin-node-type-list" class="record-list">
                    <Show when=move || !loading.get() fallback=|| view! { <p class="muted">"Loading node types..."</p> }>
                        {move || {
                            if let Some(message) = error.get() {
                                return view! { <p class="muted">{message}</p> }.into_any();
                            }
                            let items = node_types.get();
                            if items.is_empty() {
                                return view! { <p class="muted">"No node types are configured."</p> }.into_any();
                            }
                            view! {
                                {items
                                    .into_iter()
                                    .map(|node_type| {
                                        let detail_href = format!("/app/administration/node-types/{}", node_type.id);
                                        let edit_href = format!("{detail_href}/edit");
                                        view! {
                                            <article class="record-card">
                                                <h4>{node_type.name}</h4>
                                                <p>{node_type.slug}</p>
                                                <p class="muted">{format!("Plural label: {}", node_type.plural_label)}</p>
                                                <p class="muted">{format!("Top-level: {}", if node_type.is_root_type { "yes" } else { "no" })}</p>
                                                <p class="muted">{format!("Nodes: {}", node_type.node_count)}</p>
                                                <div class="actions">
                                                    <a class="button-link" href=detail_href.clone()>"View"</a>
                                                    <a class="button-link button is-light" href=edit_href>"Edit"</a>
                                                </div>
                                            </article>
                                        }
                                    })
                                    .collect_view()}
                            }.into_any()
                        }}
                    </Show>
                </div>
            </Panel>
        </NativePage>
    }
}

#[component]
pub fn NodeTypeCreatePage() -> impl IntoView {
    node_type_form_page(None)
}

#[component]
pub fn NodeTypeDetailPage() -> impl IntoView {
    let NodeTypeRouteParams { node_type_id } = require_route_params();
    let detail = RwSignal::new(None::<NodeTypeDefinition>);
    let loading = RwSignal::new(true);
    let error = RwSignal::new(None::<String>);
    let record_id = node_type_id.clone();

    let _node_type_id_for_load = node_type_id.clone();
    #[cfg(feature = "hydrate")]
    Effect::new(move |_| {
        let node_type_id = _node_type_id_for_load.clone();
        spawn_local(async move {
            loading.set(true);
            match get_json::<NodeTypeDefinition>(&format!("/api/admin/node-types/{node_type_id}"))
                .await
            {
                Ok(payload) => {
                    detail.set(Some(payload));
                    error.set(None);
                }
                Err(message) => error.set(Some(message)),
            }
            loading.set(false);
        });
    });

    view! {
        <NativePage
            title="Organization Node Type Detail"
            description="Inspect a Tessara organization node type."
            page_key="node-type-detail"
            active_route="administration"
            workspace_label="Internal Area"
            record_id=record_id.clone()
            required_capability="admin:all"
            breadcrumbs=vec![
                BreadcrumbItem::link("Home", "/app"),
                BreadcrumbItem::link("Administration", "/app/administration"),
                BreadcrumbItem::link("Organization Node Types", "/app/administration/node-types"),
                BreadcrumbItem::current("Node Type Detail"),
            ]
        >
            <PageHeader
                eyebrow="Internal Area"
                title="Organization Node Type Detail"
                description="Inspect hierarchy relationships, metadata fields, and scoped forms for the selected node type."
            />
            <MetadataStrip items=vec![
                ("Mode", "Detail".into()),
                ("Surface", "Hierarchy inspection".into()),
                ("State", "Native SSR shell".into()),
            ]/>
            <Panel title="Node Type Detail" description="Current display metadata and related relationships appear here.">
                <Show when=move || !loading.get() fallback=|| view! { <p class="muted">"Loading node type detail..."</p> }>
                    {move || {
                        if let Some(message) = error.get() {
                            return view! { <p class="muted">{message}</p> }.into_any();
                        }
                        match detail.get() {
                            Some(node_type) => view! {
                                <div id="node-type-detail-summary" class="record-list">
                                    <article class="record-card">
                                        <h4>{node_type.name.clone()}</h4>
                                        <p>{node_type.slug.clone()}</p>
                                        <p class="muted">{format!("Plural label: {}", node_type.plural_label)}</p>
                                        <p class="muted">{format!("Top-level: {}", if node_type.is_root_type { "yes" } else { "no" })}</p>
                                        <p class="muted">{format!("Nodes: {}", node_type.node_count)}</p>
                                    </article>
                                </div>
                                <div class="detail-grid">
                                    <section class="detail-section box">
                                        <h4>"Allowed Under"</h4>
                                        <ul class="app-list">
                                            {if node_type.parent_relationships.is_empty() {
                                                view! { <li class="muted">"This is a top-level node type."</li> }.into_any()
                                            } else {
                                                view! { {node_type.parent_relationships.into_iter().map(|parent| view! { <li>{parent.singular_label}</li> }).collect_view()} }.into_any()
                                            }}
                                        </ul>
                                    </section>
                                    <section class="detail-section box">
                                        <h4>"Can Contain"</h4>
                                        <ul class="app-list">
                                            {if node_type.child_relationships.is_empty() {
                                                view! { <li class="muted">"No child node types configured."</li> }.into_any()
                                            } else {
                                                view! { {node_type.child_relationships.into_iter().map(|child| view! { <li>{child.singular_label}</li> }).collect_view()} }.into_any()
                                            }}
                                        </ul>
                                    </section>
                                    <section class="detail-section box">
                                        <h4>"Metadata Fields"</h4>
                                        <ul class="app-list">
                                            {if node_type.metadata_fields.is_empty() {
                                                view! { <li class="muted">"No metadata fields configured."</li> }.into_any()
                                            } else {
                                                view! { {node_type.metadata_fields.into_iter().map(|field| view! { <li>{format!("{} ({}, required: {})", field.label, field.field_type, if field.required { "yes" } else { "no" })}</li> }).collect_view()} }.into_any()
                                            }}
                                        </ul>
                                    </section>
                                    <section class="detail-section box">
                                        <h4>"Scoped Forms"</h4>
                                        <ul class="app-list">
                                            {if node_type.scoped_forms.is_empty() {
                                                view! { <li class="muted">"No forms are scoped to this node type yet."</li> }.into_any()
                                            } else {
                                                view! { {node_type.scoped_forms.into_iter().map(|form| view! { <li>{format!("{} ({})", form.form_name, form.form_slug)}</li> }).collect_view()} }.into_any()
                                            }}
                                        </ul>
                                    </section>
                                </div>
                            }.into_any(),
                            None => view! { <p class="muted">"Node type detail is unavailable."</p> }.into_any(),
                        }
                    }}
                </Show>
            </Panel>
        </NativePage>
    }
}

#[component]
pub fn NodeTypeEditPage() -> impl IntoView {
    let NodeTypeRouteParams { node_type_id } = require_route_params();
    node_type_form_page(Some(node_type_id))
}

#[component]
pub fn RoleCreatePage() -> impl IntoView {
    role_form_page(None)
}

#[component]
pub fn RoleDetailPage() -> impl IntoView {
    let RoleRouteParams { role_id } = require_route_params();
    let detail = RwSignal::new(None::<RoleDetail>);
    let loading = RwSignal::new(true);
    let error = RwSignal::new(None::<String>);
    let record_id = role_id.clone();

    let _role_id_for_load = role_id.clone();
    #[cfg(feature = "hydrate")]
    Effect::new(move |_| {
        let role_id = _role_id_for_load.clone();
        spawn_local(async move {
            loading.set(true);
            match get_json::<RoleDetail>(&format!("/api/admin/roles/{role_id}")).await {
                Ok(payload) => {
                    detail.set(Some(payload));
                    error.set(None);
                }
                Err(message) => error.set(Some(message)),
            }
            loading.set(false);
        });
    });

    view! {
        <NativePage
            title="Role Detail"
            description="Inspect a Tessara role bundle."
            page_key="role-detail"
            active_route="administration"
            workspace_label="Internal Area"
            record_id=record_id.clone()
            required_capability="admin:all"
            breadcrumbs=vec![
                BreadcrumbItem::link("Home", "/app"),
                BreadcrumbItem::link("Administration", "/app/administration"),
                BreadcrumbItem::link("Roles", "/app/administration/roles"),
                BreadcrumbItem::current("Role Detail"),
            ]
        >
            <PageHeader
                eyebrow="Internal Area"
                title="Role Detail"
                description="Inspect role capabilities and the accounts currently assigned to this bundle."
            />
            <MetadataStrip items=vec![
                ("Mode", "Detail".into()),
                ("Surface", "Role inspection".into()),
                ("State", "Native SSR shell".into()),
            ]/>
            <Panel title="Role Detail" description="Role grants and assigned accounts appear here.">
                <Show when=move || !loading.get() fallback=|| view! { <p class="muted">"Loading role detail..."</p> }>
                    {move || {
                        if let Some(message) = error.get() {
                            return view! { <p class="muted">{message}</p> }.into_any();
                        }
                        match detail.get() {
                            Some(role) => view! {
                                <div id="role-detail-summary" class="record-list">
                                    <article class="record-card">
                                        <h4>{role.name.clone()}</h4>
                                        <p class="muted">{format!("{} capabilities", role.capabilities.len())}</p>
                                        <p class="muted">{format!("{} assigned accounts", role.assigned_accounts.len())}</p>
                                    </article>
                                </div>
                                <div class="detail-grid">
                                    <section class="detail-section box">
                                        <h4>"Capabilities"</h4>
                                        <ul class="app-list">
                                            {if role.capabilities.is_empty() {
                                                view! { <li class="muted">"No capabilities assigned."</li> }.into_any()
                                            } else {
                                                view! { {role.capabilities.into_iter().map(|capability| view! { <li>{format!("{} — {}", capability.key, capability.description)}</li> }).collect_view()} }.into_any()
                                            }}
                                        </ul>
                                    </section>
                                    <section class="detail-section box">
                                        <h4>"Assigned Accounts"</h4>
                                        <ul class="app-list">
                                            {if role.assigned_accounts.is_empty() {
                                                view! { <li class="muted">"No accounts are assigned to this role."</li> }.into_any()
                                            } else {
                                                view! { {role.assigned_accounts.into_iter().map(|account| view! { <li>{format!("{} ({})", account.display_name, account.email)}</li> }).collect_view()} }.into_any()
                                            }}
                                        </ul>
                                    </section>
                                </div>
                            }.into_any(),
                            None => view! { <p class="muted">"Role detail is unavailable."</p> }.into_any(),
                        }
                    }}
                </Show>
            </Panel>
        </NativePage>
    }
}

#[component]
pub fn RoleEditPage() -> impl IntoView {
    let RoleRouteParams { role_id } = require_route_params();
    role_form_page(Some(role_id))
}

#[component]
pub fn LegacyAdminPage() -> impl IntoView {
    view! {
        <NativePage
            title="Administration"
            description="Tessara internal admin workbench summary."
            page_key="admin-shell"
            active_route="administration"
            workspace_label="Internal Area"
            required_capability="admin:all"
            breadcrumbs=vec![
                BreadcrumbItem::link("Home", "/app"),
                BreadcrumbItem::link("Administration", "/app/administration"),
                BreadcrumbItem::current("Admin Console"),
            ]
        >
            <PageHeader
                eyebrow="Internal Area"
                title="Admin Console"
                description="Legacy administrative workflows have been split into dedicated native routes. Use this summary page as the compatibility entry point."
            />
            <MetadataStrip items=vec![
                ("Mode", "Compatibility".into()),
                ("Surface", "Administrative summary".into()),
                ("State", "Native SSR shell".into()),
            ]/>
            <Panel title="Available Destinations" description="Use the dedicated native routes below instead of the older combined console.">
                <div class="record-list">
                    <article class="record-card">
                        <h4>"Users"</h4>
                        <div class="actions">
                            <a class="button-link" href="/app/administration/users">"Open Users"</a>
                        </div>
                    </article>
                    <article class="record-card">
                        <h4>"Roles"</h4>
                        <div class="actions">
                            <a class="button-link" href="/app/administration/roles">"Open Roles"</a>
                        </div>
                    </article>
                    <article class="record-card">
                        <h4>"Organization Node Types"</h4>
                        <div class="actions">
                            <a class="button-link" href="/app/administration/node-types">"Open Node Types"</a>
                        </div>
                    </article>
                    <article class="record-card">
                        <h4>"Migration Workbench"</h4>
                        <div class="actions">
                            <a class="button-link button is-light" href="/app/migration">"Open Migration"</a>
                        </div>
                    </article>
                </div>
            </Panel>
        </NativePage>
    }
}

fn user_form_page(account_id: Option<String>) -> impl IntoView {
    let email = RwSignal::new(String::new());
    let display_name = RwSignal::new(String::new());
    let password = RwSignal::new(String::new());
    let is_active = RwSignal::new(true);
    let roles = RwSignal::new(Vec::<RoleSummary>::new());
    let selected_role_ids = RwSignal::new(Vec::<String>::new());
    let loading = RwSignal::new(true);
    let busy = RwSignal::new(false);
    let error = RwSignal::new(None::<String>);
    let status = RwSignal::new(None::<String>);
    let is_edit = account_id.is_some();
    let record_id = account_id.clone();
    let _account_id_for_load = account_id.clone();

    #[cfg(feature = "hydrate")]
    Effect::new(move |_| {
        let account_id = _account_id_for_load.clone();
        spawn_local(async move {
            loading.set(true);
            let roles_result = get_json::<Vec<RoleSummary>>("/api/admin/roles").await;
            let user_result = match account_id.clone() {
                Some(account_id) => {
                    Some(get_json::<UserDetail>(&format!("/api/admin/users/{account_id}")).await)
                }
                None => None,
            };

            match roles_result {
                Ok(items) => roles.set(items),
                Err(message) => error.set(Some(message)),
            }

            if let Some(result) = user_result {
                match result {
                    Ok(user) => {
                        email.set(user.email);
                        display_name.set(user.display_name);
                        is_active.set(user.is_active);
                        selected_role_ids.set(user.roles.into_iter().map(|role| role.id).collect());
                        status.set(Some("User detail loaded.".into()));
                    }
                    Err(message) => error.set(Some(message)),
                }
            } else {
                status.set(Some(
                    "Provide account details and assign at least one role.".into(),
                ));
            }
            loading.set(false);
        });
    });

    let cancel_href = StoredValue::new(
        account_id
            .as_ref()
            .map(|account_id| format!("/app/administration/users/{account_id}"))
            .unwrap_or_else(|| "/app/administration/users".into()),
    );
    let _account_id_for_submit = account_id.clone();
    let submit = StoredValue::new(std::sync::Arc::new(
        move |event: leptos::ev::SubmitEvent| {
            event.prevent_default();
            if busy.get_untracked() {
                return;
            }
            busy.set(true);
            error.set(None);
            status.set(Some(
                if is_edit {
                    "Saving user changes..."
                } else {
                    "Creating user..."
                }
                .into(),
            ));

            #[cfg(feature = "hydrate")]
            {
                let account_id = _account_id_for_submit.clone();
                let payload = json!({
                    "email": email.get_untracked(),
                    "display_name": display_name.get_untracked(),
                    "password": if is_edit {
                        let password = password.get_untracked();
                        if password.trim().is_empty() { serde_json::Value::Null } else { json!(password) }
                    } else {
                        json!(password.get_untracked())
                    },
                    "is_active": is_active.get_untracked(),
                    "role_ids": selected_role_ids.get_untracked(),
                });
                spawn_local(async move {
                    let result = match account_id {
                        Some(account_id) => {
                            put_json::<IdResponse>(
                                &format!("/api/admin/users/{account_id}"),
                                &payload,
                            )
                            .await
                        }
                        None => post_json::<IdResponse>("/api/admin/users", &payload).await,
                    };
                    match result {
                        Ok(response) => {
                            redirect(&format!("/app/administration/users/{}", response.id))
                        }
                        Err(message) => {
                            error.set(Some(message));
                            status.set(Some("Unable to save the user.".into()));
                        }
                    }
                    busy.set(false);
                });
            }
        },
    ));

    view! {
        <NativePage
            title=if is_edit { "Edit User" } else { "Create User" }
            description=if is_edit { "Edit a Tessara application account." } else { "Create a Tessara application account." }
            page_key=if is_edit { "user-edit" } else { "user-create" }
            active_route="administration"
            workspace_label="Internal Area"
            record_id=record_id.clone().unwrap_or_default()
            required_capability="admin:all"
            breadcrumbs={
                let mut items = vec![
                    BreadcrumbItem::link("Home", "/app"),
                    BreadcrumbItem::link("Administration", "/app/administration"),
                    BreadcrumbItem::link("Users", "/app/administration/users"),
                ];
                if is_edit {
                    if let Some(account_id) = account_id.clone() {
                        items.push(BreadcrumbItem::link("User Detail", format!("/app/administration/users/{account_id}")));
                    }
                }
                items.push(BreadcrumbItem::current(if is_edit { "Edit User" } else { "Create User" }));
                items
            }
        >
            <PageHeader
                eyebrow="Internal Area"
                title=if is_edit { "Edit User" } else { "Create User" }
                description=if is_edit { "Update account identity, status, and role assignments here." } else { "Provide account identity, a password, and at least one role." }
            />
            <MetadataStrip items=vec![
                ("Mode", if is_edit { "Edit".into() } else { "Create".into() }),
                ("Surface", "Account authoring".into()),
                ("State", "Native SSR shell".into()),
            ]/>
            <Panel title="User Form" description="User changes save here.">
                <form id="user-form" class="entity-form" on:submit=move |event| submit.with_value(|submit| submit(event))>
                    <div class="form-grid">
                        <div class="form-field">
                            <label for="user-email">"Email"</label>
                            <input id="user-email" class="input" type="email" prop:value=move || email.get() on:input=move |event| email.set(event_target_value(&event)) />
                        </div>
                        <div class="form-field">
                            <label for="user-display-name">"Display Name"</label>
                            <input id="user-display-name" class="input" type="text" prop:value=move || display_name.get() on:input=move |event| display_name.set(event_target_value(&event)) />
                        </div>
                        <div class="form-field">
                            <label for="user-password">{if is_edit { "Password (optional)" } else { "Password" }}</label>
                            <input id="user-password" class="input" type="password" prop:value=move || password.get() on:input=move |event| password.set(event_target_value(&event)) />
                        </div>
                        <div class="form-field">
                            <label class="checkbox">
                                <input type="checkbox" prop:checked=move || is_active.get() on:change=move |event| is_active.set(event_target_checked(&event)) />
                                <span>"Active account"</span>
                            </label>
                        </div>
                    </div>
                    <section class="detail-section box">
                        <h4>"Assigned Roles"</h4>
                        <div class="selection-grid">
                            {move || {
                                if loading.get() {
                                    return view! { <p class="muted">"Loading roles..."</p> }.into_any();
                                }
                                let role_items = roles.get();
                                if role_items.is_empty() {
                                    return view! { <p class="muted">"No roles are available."</p> }.into_any();
                                }
                                view! {
                                    {role_items
                                        .into_iter()
                                        .map(|role| {
                                            let role_id = role.id.clone();
                                            let changed_role_id = role_id.clone();
                                            let label = format!("{} ({} capabilities)", role.name, role.capability_count);
                                            view! {
                                                <label class="selection-item box">
                                                    <input
                                                        type="checkbox"
                                                        prop:checked=move || selected_role_ids.get().iter().any(|value| value == &role_id)
                                                        on:change=move |event| {
                                                            let checked = event_target_checked(&event);
                                                            selected_role_ids.update(|values| toggle_selection(values, &changed_role_id, checked));
                                                        }
                                                    />
                                                    <span>{label}</span>
                                                </label>
                                            }
                                        })
                                        .collect_view()}
                                }.into_any()
                            }}
                        </div>
                    </section>
                    <p id="user-form-status" class="muted">
                        {move || error.get().or_else(|| status.get()).unwrap_or_else(|| "Update the account and save changes here.".into())}
                    </p>
                    <div class="actions">
                        <button class="button-link" type="submit" disabled=move || busy.get() || loading.get()>
                            {move || if busy.get() { "Saving..." } else if is_edit { "Save User" } else { "Create User" }}
                        </button>
                        <a class="button-link button is-light" href=move || cancel_href.get_value()>"Cancel"</a>
                    </div>
                </form>
            </Panel>
        </NativePage>
    }
}

fn role_form_page(role_id: Option<String>) -> impl IntoView {
    let name = RwSignal::new(String::new());
    let capabilities = RwSignal::new(Vec::<CapabilitySummary>::new());
    let selected_capability_ids = RwSignal::new(Vec::<String>::new());
    let loading = RwSignal::new(true);
    let busy = RwSignal::new(false);
    let error = RwSignal::new(None::<String>);
    let status = RwSignal::new(None::<String>);
    let is_edit = role_id.is_some();
    let record_id = role_id.clone();
    let _role_id_for_load = role_id.clone();

    #[cfg(feature = "hydrate")]
    Effect::new(move |_| {
        let role_id = _role_id_for_load.clone();
        spawn_local(async move {
            loading.set(true);
            let capabilities_result =
                get_json::<Vec<CapabilitySummary>>("/api/admin/capabilities").await;
            let role_result = match role_id.clone() {
                Some(role_id) => {
                    Some(get_json::<RoleDetail>(&format!("/api/admin/roles/{role_id}")).await)
                }
                None => None,
            };

            match capabilities_result {
                Ok(items) => capabilities.set(items),
                Err(message) => error.set(Some(message)),
            }
            if let Some(result) = role_result {
                match result {
                    Ok(role) => {
                        name.set(role.name);
                        selected_capability_ids.set(
                            role.capabilities
                                .into_iter()
                                .map(|capability| capability.id)
                                .collect(),
                        );
                        status.set(Some("Role detail loaded.".into()));
                    }
                    Err(message) => error.set(Some(message)),
                }
            } else {
                status.set(Some(
                    "Provide a role name and choose one or more capabilities.".into(),
                ));
            }
            loading.set(false);
        });
    });

    let cancel_href = StoredValue::new(
        role_id
            .as_ref()
            .map(|role_id| format!("/app/administration/roles/{role_id}"))
            .unwrap_or_else(|| "/app/administration/roles".into()),
    );
    let _role_id_for_submit = role_id.clone();
    let submit = StoredValue::new(std::sync::Arc::new(
        move |event: leptos::ev::SubmitEvent| {
            event.prevent_default();
            if busy.get_untracked() {
                return;
            }
            busy.set(true);
            error.set(None);
            status.set(Some(
                if is_edit {
                    "Saving role changes..."
                } else {
                    "Creating role..."
                }
                .into(),
            ));

            #[cfg(feature = "hydrate")]
            {
                let role_id = _role_id_for_submit.clone();
                let payload = if is_edit {
                    json!({ "capability_ids": selected_capability_ids.get_untracked() })
                } else {
                    json!({
                        "name": name.get_untracked(),
                        "capability_ids": selected_capability_ids.get_untracked(),
                    })
                };
                spawn_local(async move {
                    let result = match role_id {
                        Some(role_id) => {
                            put_json::<IdResponse>(&format!("/api/admin/roles/{role_id}"), &payload)
                                .await
                        }
                        None => post_json::<IdResponse>("/api/admin/roles", &payload).await,
                    };
                    match result {
                        Ok(response) => {
                            redirect(&format!("/app/administration/roles/{}", response.id))
                        }
                        Err(message) => {
                            error.set(Some(message));
                            status.set(Some("Unable to save the role.".into()));
                        }
                    }
                    busy.set(false);
                });
            }
        },
    ));

    view! {
        <NativePage
            title=if is_edit { "Edit Role" } else { "Create Role" }
            description=if is_edit { "Edit a Tessara role bundle." } else { "Create a Tessara role bundle." }
            page_key=if is_edit { "role-edit" } else { "role-create" }
            active_route="administration"
            workspace_label="Internal Area"
            record_id=record_id.clone().unwrap_or_default()
            required_capability="admin:all"
            breadcrumbs={
                let mut items = vec![
                    BreadcrumbItem::link("Home", "/app"),
                    BreadcrumbItem::link("Administration", "/app/administration"),
                    BreadcrumbItem::link("Roles", "/app/administration/roles"),
                ];
                if is_edit {
                    if let Some(role_id) = role_id.clone() {
                        items.push(BreadcrumbItem::link("Role Detail", format!("/app/administration/roles/{role_id}")));
                    }
                }
                items.push(BreadcrumbItem::current(if is_edit { "Edit Role" } else { "Create Role" }));
                items
            }
        >
            <PageHeader
                eyebrow="Internal Area"
                title=if is_edit { "Edit Role" } else { "Create Role" }
                description=if is_edit { "Update capability grants for the selected role." } else { "Define a role bundle and assign one or more capabilities." }
            />
            <MetadataStrip items=vec![
                ("Mode", if is_edit { "Edit".into() } else { "Create".into() }),
                ("Surface", "Role authoring".into()),
                ("State", "Native SSR shell".into()),
            ]/>
            <Panel title="Role Form" description="Role changes save here.">
                <form id="role-form" class="entity-form" on:submit=move |event| submit.with_value(|submit| submit(event))>
                    <div class="form-grid">
                        <div class="form-field wide-field">
                            <label for="role-name">"Role Name"</label>
                            <input
                                id="role-name"
                                class="input"
                                type="text"
                                prop:value=move || name.get()
                                on:input=move |event| name.set(event_target_value(&event))
                                disabled=is_edit
                            />
                        </div>
                    </div>
                    <section class="detail-section box">
                        <h4>"Capabilities"</h4>
                        <div class="selection-grid">
                            {move || {
                                if loading.get() {
                                    return view! { <p class="muted">"Loading capabilities..."</p> }.into_any();
                                }
                                let capability_items = capabilities.get();
                                if capability_items.is_empty() {
                                    return view! { <p class="muted">"No capabilities are available."</p> }.into_any();
                                }
                                view! {
                                    {capability_items
                                        .into_iter()
                                        .map(|capability| {
                                            let capability_id = capability.id.clone();
                                            let changed_capability_id = capability_id.clone();
                                            let label = format!("{} — {}", capability.key, capability.description);
                                            view! {
                                                <label class="selection-item box">
                                                    <input
                                                        type="checkbox"
                                                        prop:checked=move || selected_capability_ids.get().iter().any(|value| value == &capability_id)
                                                        on:change=move |event| {
                                                            let checked = event_target_checked(&event);
                                                            selected_capability_ids.update(|values| toggle_selection(values, &changed_capability_id, checked));
                                                        }
                                                    />
                                                    <span>{label}</span>
                                                </label>
                                            }
                                        })
                                        .collect_view()}
                                }.into_any()
                            }}
                        </div>
                    </section>
                    <p id="role-form-status" class="muted">
                        {move || error.get().or_else(|| status.get()).unwrap_or_else(|| "Choose capability grants and save changes here.".into())}
                    </p>
                    <div class="actions">
                        <button class="button-link" type="submit" disabled=move || busy.get() || loading.get()>
                            {move || if busy.get() { "Saving..." } else if is_edit { "Save Role" } else { "Create Role" }}
                        </button>
                        <a class="button-link button is-light" href=move || cancel_href.get_value()>"Cancel"</a>
                    </div>
                </form>
            </Panel>
        </NativePage>
    }
}

fn node_type_form_page(node_type_id: Option<String>) -> impl IntoView {
    let name = RwSignal::new(String::new());
    let slug = RwSignal::new(String::new());
    let plural_label = RwSignal::new(String::new());
    let catalog = RwSignal::new(Vec::<NodeTypeSummary>::new());
    let selected_parent_ids = RwSignal::new(Vec::<String>::new());
    let selected_child_ids = RwSignal::new(Vec::<String>::new());
    let loading = RwSignal::new(true);
    let busy = RwSignal::new(false);
    let error = RwSignal::new(None::<String>);
    let status = RwSignal::new(None::<String>);
    let is_edit = node_type_id.is_some();
    let record_id = node_type_id.clone();
    let _node_type_id_for_load = node_type_id.clone();

    #[cfg(feature = "hydrate")]
    Effect::new(move |_| {
        let node_type_id = _node_type_id_for_load.clone();
        spawn_local(async move {
            loading.set(true);
            let catalog_result = get_json::<Vec<NodeTypeSummary>>("/api/admin/node-types").await;
            let detail_result = match node_type_id.clone() {
                Some(node_type_id) => Some(
                    get_json::<NodeTypeDefinition>(&format!(
                        "/api/admin/node-types/{node_type_id}"
                    ))
                    .await,
                ),
                None => None,
            };

            match catalog_result {
                Ok(items) => catalog.set(items),
                Err(message) => error.set(Some(message)),
            }
            if let Some(result) = detail_result {
                match result {
                    Ok(detail) => {
                        name.set(detail.name);
                        slug.set(detail.slug);
                        plural_label.set(detail.plural_label);
                        selected_parent_ids.set(
                            detail
                                .parent_relationships
                                .into_iter()
                                .map(|item| item.node_type_id)
                                .collect(),
                        );
                        selected_child_ids.set(
                            detail
                                .child_relationships
                                .into_iter()
                                .map(|item| item.node_type_id)
                                .collect(),
                        );
                        status.set(Some("Node type detail loaded.".into()));
                    }
                    Err(message) => error.set(Some(message)),
                }
            } else {
                status.set(Some(
                    "Provide node type labels and configure allowed relationships.".into(),
                ));
            }
            loading.set(false);
        });
    });

    let cancel_href = StoredValue::new(
        node_type_id
            .as_ref()
            .map(|node_type_id| format!("/app/administration/node-types/{node_type_id}"))
            .unwrap_or_else(|| "/app/administration/node-types".into()),
    );
    let parent_current_id = StoredValue::new(node_type_id.clone());
    let child_current_id = StoredValue::new(node_type_id.clone());
    let _node_type_id_for_submit = node_type_id.clone();
    let submit = StoredValue::new(std::sync::Arc::new(
        move |event: leptos::ev::SubmitEvent| {
            event.prevent_default();
            if busy.get_untracked() {
                return;
            }
            busy.set(true);
            error.set(None);
            status.set(Some(
                if is_edit {
                    "Saving node type changes..."
                } else {
                    "Creating node type..."
                }
                .into(),
            ));

            #[cfg(feature = "hydrate")]
            {
                let node_type_id = _node_type_id_for_submit.clone();
                let plural_label_value = plural_label.get_untracked();
                let plural_label_payload = if plural_label_value.trim().is_empty() {
                    serde_json::Value::Null
                } else {
                    json!(plural_label_value)
                };
                let payload = json!({
                    "name": name.get_untracked(),
                    "slug": slug.get_untracked(),
                    "plural_label": plural_label_payload,
                    "parent_node_type_ids": selected_parent_ids.get_untracked(),
                    "child_node_type_ids": selected_child_ids.get_untracked(),
                });
                spawn_local(async move {
                    let result = match node_type_id {
                        Some(node_type_id) => {
                            put_json::<IdResponse>(
                                &format!("/api/admin/node-types/{node_type_id}"),
                                &payload,
                            )
                            .await
                        }
                        None => post_json::<IdResponse>("/api/admin/node-types", &payload).await,
                    };
                    match result {
                        Ok(response) => {
                            redirect(&format!("/app/administration/node-types/{}", response.id))
                        }
                        Err(message) => {
                            error.set(Some(message));
                            status.set(Some("Unable to save the node type.".into()));
                        }
                    }
                    busy.set(false);
                });
            }
        },
    ));

    view! {
        <NativePage
            title=if is_edit { "Edit Organization Node Type" } else { "Create Organization Node Type" }
            description=if is_edit { "Edit a Tessara organization node type." } else { "Create a Tessara organization node type." }
            page_key=if is_edit { "node-type-edit" } else { "node-type-create" }
            active_route="administration"
            workspace_label="Internal Area"
            record_id=record_id.clone().unwrap_or_default()
            required_capability="admin:all"
            breadcrumbs={
                let mut items = vec![
                    BreadcrumbItem::link("Home", "/app"),
                    BreadcrumbItem::link("Administration", "/app/administration"),
                    BreadcrumbItem::link("Organization Node Types", "/app/administration/node-types"),
                ];
                if is_edit {
                    if let Some(node_type_id) = node_type_id.clone() {
                        items.push(BreadcrumbItem::link("Node Type Detail", format!("/app/administration/node-types/{node_type_id}")));
                    }
                }
                items.push(BreadcrumbItem::current(if is_edit { "Edit Node Type" } else { "Create Node Type" }));
                items
            }
        >
            <PageHeader
                eyebrow="Internal Area"
                title=if is_edit { "Edit Organization Node Type" } else { "Create Organization Node Type" }
                description=if is_edit { "Update node type labels and allowed relationships here." } else { "Define a new node type and the hierarchy relationships it participates in." }
            />
            <MetadataStrip items=vec![
                ("Mode", if is_edit { "Edit".into() } else { "Create".into() }),
                ("Surface", "Hierarchy authoring".into()),
                ("State", "Native SSR shell".into()),
            ]/>
            <Panel title="Node Type Form" description="Node type changes save here.">
                <form id="node-type-form" class="entity-form" on:submit=move |event| submit.with_value(|submit| submit(event))>
                    <div class="form-grid">
                        <div class="form-field">
                            <label for="node-type-name">"Name"</label>
                            <input id="node-type-name" class="input" type="text" prop:value=move || name.get() on:input=move |event| name.set(event_target_value(&event)) />
                        </div>
                        <div class="form-field">
                            <label for="node-type-slug">"Slug"</label>
                            <input id="node-type-slug" class="input" type="text" prop:value=move || slug.get() on:input=move |event| slug.set(event_target_value(&event)) />
                        </div>
                        <div class="form-field wide-field">
                            <label for="node-type-plural-label">"Plural Label"</label>
                            <input id="node-type-plural-label" class="input" type="text" prop:value=move || plural_label.get() on:input=move |event| plural_label.set(event_target_value(&event)) />
                        </div>
                    </div>
                    <div class="detail-grid">
                        <section class="detail-section box">
                            <h4>"Allowed Under"</h4>
                            <div class="selection-grid">
                                {move || {
                                    if loading.get() {
                                        return view! { <p class="muted">"Loading node type catalog..."</p> }.into_any();
                                    }
                                    let items = catalog.get();
                                    if items.is_empty() {
                                        return view! { <p class="muted">"No node types are available."</p> }.into_any();
                                    }
                                    let current_id = parent_current_id.get_value();
                                    view! {
                                        {items
                                            .into_iter()
                                            .filter(|item| current_id.as_ref().is_none_or(|current_id| current_id != &item.id))
                                            .map(|item| {
                                                let peer_id = item.id.clone();
                                                let changed_peer_id = peer_id.clone();
                                                let label = format!("{} ({})", item.name, item.plural_label);
                                                view! {
                                                    <label class="selection-item box">
                                                        <input
                                                            type="checkbox"
                                                            prop:checked=move || selected_parent_ids.get().iter().any(|value| value == &peer_id)
                                                            on:change=move |event| {
                                                                let checked = event_target_checked(&event);
                                                                selected_parent_ids.update(|values| toggle_selection(values, &changed_peer_id, checked));
                                                            }
                                                        />
                                                        <span>{label}</span>
                                                    </label>
                                                }
                                            })
                                            .collect_view()}
                                    }.into_any()
                                }}
                            </div>
                        </section>
                        <section class="detail-section box">
                            <h4>"Can Contain"</h4>
                            <div class="selection-grid">
                                {move || {
                                    if loading.get() {
                                        return view! { <p class="muted">"Loading node type catalog..."</p> }.into_any();
                                    }
                                    let items = catalog.get();
                                    if items.is_empty() {
                                        return view! { <p class="muted">"No node types are available."</p> }.into_any();
                                    }
                                    let current_id = child_current_id.get_value();
                                    view! {
                                        {items
                                            .into_iter()
                                            .filter(|item| current_id.as_ref().is_none_or(|current_id| current_id != &item.id))
                                            .map(|item| {
                                                let peer_id = item.id.clone();
                                                let changed_peer_id = peer_id.clone();
                                                let label = format!("{} ({})", item.name, item.plural_label);
                                                view! {
                                                    <label class="selection-item box">
                                                        <input
                                                            type="checkbox"
                                                            prop:checked=move || selected_child_ids.get().iter().any(|value| value == &peer_id)
                                                            on:change=move |event| {
                                                                let checked = event_target_checked(&event);
                                                                selected_child_ids.update(|values| toggle_selection(values, &changed_peer_id, checked));
                                                            }
                                                        />
                                                        <span>{label}</span>
                                                    </label>
                                                }
                                            })
                                            .collect_view()}
                                    }.into_any()
                                }}
                            </div>
                        </section>
                    </div>
                    <p id="node-type-form-status" class="muted">
                        {move || error.get().or_else(|| status.get()).unwrap_or_else(|| "Update the node type and save changes here.".into())}
                    </p>
                    <div class="actions">
                        <button class="button-link" type="submit" disabled=move || busy.get() || loading.get()>
                            {move || if busy.get() { "Saving..." } else if is_edit { "Save Node Type" } else { "Create Node Type" }}
                        </button>
                        <a class="button-link button is-light" href=move || cancel_href.get_value()>"Cancel"</a>
                    </div>
                </form>
            </Panel>
        </NativePage>
    }
}
