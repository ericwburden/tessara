//! Role-management administration pages and helpers.
//!
//! Keep role list, role detail, capability selection, and role save workflows here.

use super::users::toggle_string_selection;
use crate::features::administration::models::*;
use crate::features::organization::AdminRoleSummary;
#[cfg(feature = "hydrate")]
use crate::http::{redirect_to_login, send_json_id_request};
use crate::ui::{
    AppShell, Breadcrumb, BreadcrumbItem, BreadcrumbLink, BreadcrumbPage, BreadcrumbSeparator,
    DataTable, DropdownMenu, PageHeader,
};
use crate::utils::text::text_matches;
use leptos::portal::Portal;

use icons::{PanelRight, Search, X};
use leptos::prelude::*;

#[component]
/// Renders the administration roles page view.
pub fn AdministrationRolesPage() -> impl IntoView {
    let roles = RwSignal::new(Vec::<AdminRoleSummary>::new());
    let capabilities = RwSignal::new(Vec::<AdminCapabilitySummary>::new());
    let selected_role_id = RwSignal::new(None::<String>);
    let selected_role_detail = RwSignal::new(None::<AdminRoleDetail>);
    let search = RwSignal::new(String::new());
    let is_loading = RwSignal::new(true);
    let detail_loading = RwSignal::new(false);
    let is_saving = RwSignal::new(false);
    let message = RwSignal::new(None::<String>);
    let sheet_open = RwSignal::new(false);
    let editing_role_id = RwSignal::new(None::<String>);
    let role_name = RwSignal::new(String::new());
    let selected_capability_ids = RwSignal::new(Vec::<String>::new());
    let capability_search = RwSignal::new(String::new());

    load_admin_roles_context(
        roles,
        capabilities,
        selected_role_id,
        is_loading,
        message,
        None,
    );

    Effect::new(move |_| {
        if let Some(role_id) = selected_role_id.get() {
            load_admin_role_detail(role_id, selected_role_detail, detail_loading, message);
        } else {
            selected_role_detail.set(None);
        }
    });

    let filtered_roles = move || {
        let query = search.get().trim().to_lowercase();
        roles
            .get()
            .into_iter()
            .filter(|role| query.is_empty() || role.name.to_lowercase().contains(&query))
            .collect::<Vec<_>>()
    };
    let open_create_sheet = move |_| {
        editing_role_id.set(None);
        role_name.set(String::new());
        selected_capability_ids.set(Vec::new());
        capability_search.set(String::new());
        message.set(None);
        sheet_open.set(true);
    };
    let open_edit_sheet = move |_| {
        if let Some(detail) = selected_role_detail.get() {
            editing_role_id.set(Some(detail.id));
            role_name.set(detail.name);
            selected_capability_ids.set(
                detail
                    .capabilities
                    .into_iter()
                    .map(|capability| capability.id)
                    .collect(),
            );
            capability_search.set(String::new());
            message.set(None);
            sheet_open.set(true);
        }
    };
    let close_sheet = move |_| {
        sheet_open.set(false);
        editing_role_id.set(None);
        role_name.set(String::new());
        selected_capability_ids.set(Vec::new());
        capability_search.set(String::new());
    };
    let save_role = move |_| {
        save_admin_role(
            editing_role_id,
            role_name,
            selected_capability_ids,
            is_saving,
            message,
            sheet_open,
            roles,
            capabilities,
            selected_role_id,
            selected_role_detail,
            detail_loading,
        );
    };

    view! {
        <AppShell active_route="administration" title="Roles">
            <section class="route-panel administration-roles-page">
                <Breadcrumb>
                    <BreadcrumbItem>
                        <BreadcrumbLink href="/administration">"Administration"</BreadcrumbLink>
                    </BreadcrumbItem>
                    <BreadcrumbSeparator/>
                    <BreadcrumbItem>
                        <BreadcrumbPage>"Roles"</BreadcrumbPage>
                    </BreadcrumbItem>
                </Breadcrumb>

                <PageHeader
                    title="Roles"
                    description="Manage reusable capability templates for Tessara users."
                >
                    <button class="button" type="button" on:click=open_create_sheet>
                        "New Role"
                    </button>
                </PageHeader>

                {move || {
                    if is_loading.get() {
                        view! {
                            <section class="organization-state" aria-live="polite">
                                <h3>"Loading roles"</h3>
                                <p>"Fetching role and capability configuration."</p>
                            </section>
                        }
                        .into_any()
                    } else if let Some(error) = message.get().filter(|_| roles.get().is_empty()) {
                        view! {
                            <section class="organization-state is-error" role="alert">
                                <h3>"Roles unavailable"</h3>
                                <p>{error}</p>
                            </section>
                        }
                        .into_any()
                    } else {
                        view! {
                            <div class="administration-roles-stack">
                                <AdministrationRolesList
                                    roles=filtered_roles()
                                    search
                                    selected_role_id
                                />
                                <AdministrationRoleDetailPanel
                                    detail=selected_role_detail.get()
                                    is_loading=detail_loading.get()
                                    on_edit=open_edit_sheet
                                />
                            </div>
                        }
                        .into_any()
                    }
                }}

                <AdminRoleSheet
                    is_open=sheet_open
                    editing_role_id
                    role_name
                    capabilities
                    selected_capability_ids
                    capability_search
                    is_saving
                    message
                    on_close=close_sheet
                    on_save=save_role
                />
            </section>
        </AppShell>
    }
}

#[component]
/// Renders the administration roles list view.
fn AdministrationRolesList(
    roles: Vec<AdminRoleSummary>,
    search: RwSignal<String>,
    selected_role_id: RwSignal<Option<String>>,
) -> impl IntoView {
    let table_roles = roles.clone();
    let card_roles = roles.clone();
    let page_size = RwSignal::new(10usize);
    let page_index = RwSignal::new(0usize);
    let total_count = roles.len();
    let page_count = move || {
        if total_count == 0 {
            1
        } else {
            total_count.div_ceil(page_size.get()).max(1)
        }
    };
    let current_page = move || page_index.get().min(page_count() - 1);
    let page_start = move || {
        if total_count == 0 {
            0
        } else {
            current_page() * page_size.get()
        }
    };
    let page_end = move || (page_start() + page_size.get()).min(total_count);
    let page_summary = move || {
        if total_count == 0 {
            "No roles to display".to_string()
        } else {
            format!(
                "Showing {}-{} of {} roles",
                page_start() + 1,
                page_end(),
                total_count
            )
        }
    };

    view! {
        <div class="forms-list forms-list-responsive-table administration-roles-list">
            <div class="searchable-data-table">
                <div class="searchable-data-table__toolbar forms-list__toolbar">
                    <label class="searchable-data-table__search searchable-data-table__control">
                        <Search class="searchable-data-table__control-icon"/>
                        <span class="sr-only">"Search roles"</span>
                        <input
                            type="search"
                            placeholder="Search roles"
                            prop:value=move || search.get()
                            on:input=move |event| {
                                search.set(event_target_value(&event));
                                page_index.set(0);
                            }
                        />
                    </label>
                </div>
                <DataTable>
                    <thead>
                        <tr>
                            <th scope="col">"Role"</th>
                            <th class="data-table__cell--center" scope="col">"Capabilities"</th>
                            <th class="data-table__cell--center" scope="col">"Users"</th>
                            <th class="data-table__cell--center" scope="col">"Actions"</th>
                        </tr>
                    </thead>
                    <tbody>
                        {move || {
                            if table_roles.is_empty() {
                                view! {
                                    <tr>
                                        <td class="data-table__empty" colspan="4">"No Roles to Display"</td>
                                    </tr>
                                }
                                .into_any()
                            } else {
                                table_roles
                                    .iter()
                                    .skip(page_start())
                                    .take(page_size.get())
                                    .cloned()
                                    .map(|role| {
                                        let role_id = role.id.clone();
                                        let role_name = role.name.clone();
                                        view! {
                                            <tr>
                                                <th scope="row">
                                                    <button
                                                        class="link-button data-table__primary-link"
                                                        type="button"
                                                        on:click=move |_| selected_role_id.set(Some(role_id.clone()))
                                                    >
                                                        {role.name}
                                                    </button>
                                                </th>
                                                <td class="data-table__cell--center">{role.capability_count}</td>
                                                <td class="data-table__cell--center">{role.account_count}</td>
                                                <td class="data-table__cell--center">
                                                    <DropdownMenu label=format!("Open actions for {role_name}")>
                                                        <button
                                                            class="dropdown-menu__item"
                                                            type="button"
                                                            role="menuitem"
                                                            on:click=move |_| selected_role_id.set(Some(role.id.clone()))
                                                        >
                                                            <PanelRight class="dropdown-menu__item-icon"/>
                                                            <span>"View Details"</span>
                                                        </button>
                                                    </DropdownMenu>
                                                </td>
                                            </tr>
                                        }
                                    })
                                    .collect_view()
                                    .into_any()
                            }
                        }}
                    </tbody>
                </DataTable>
                <div class="directory-table-pagination" aria-label="Administration roles table pagination">
                    <p>{move || page_summary()}</p>
                    <div class="directory-table-pagination__actions">
                        <label class="directory-table-pagination__page-size searchable-data-table__filter searchable-data-table__control">
                            <span>"Rows"</span>
                            <select
                                prop:value=move || page_size.get().to_string()
                                on:change=move |event| {
                                    if let Ok(size) = event_target_value(&event).parse::<usize>() {
                                        page_size.set(size);
                                        page_index.set(0);
                                    }
                                }
                            >
                                <option value="10">"10"</option>
                                <option value="25">"25"</option>
                                <option value="50">"50"</option>
                            </select>
                        </label>
                        <button
                            class="button button--compact button--secondary"
                            type="button"
                            disabled=move || current_page() == 0
                            on:click=move |_| {
                                page_index.update(|page| *page = page.saturating_sub(1));
                            }
                        >
                            "Previous"
                        </button>
                        <span>{move || format!("Page {} of {}", current_page() + 1, page_count())}</span>
                        <button
                            class="button button--compact button--secondary"
                            type="button"
                            disabled=move || { current_page() + 1 >= page_count() }
                            on:click=move |_| {
                                let last_page = page_count().saturating_sub(1);
                                page_index.update(|page| *page = (*page + 1).min(last_page));
                            }
                        >
                            "Next"
                        </button>
                    </div>
                </div>
            </div>
            <div class="forms-list-mobile-cards administration-roles-mobile-cards">
                {move || {
                    if card_roles.is_empty() {
                        view! { <p class="forms-list-mobile-empty">"No Roles to Display"</p> }.into_any()
                    } else {
                        card_roles
                            .iter()
                            .skip(page_start())
                            .take(page_size.get())
                            .cloned()
                            .map(|role| {
                                let role_id = role.id.clone();
                                view! {
                                    <article class="forms-list-mobile-card administration-role-mobile-card">
                                        <div class="forms-list-mobile-card__header">
                                            <div>
                                                <h3>{role.name}</h3>
                                            </div>
                                        </div>
                                        <dl>
                                            <div>
                                                <dt>"Capabilities"</dt>
                                                <dd>{role.capability_count}</dd>
                                            </div>
                                            <div>
                                                <dt>"Users"</dt>
                                                <dd>{role.account_count}</dd>
                                            </div>
                                        </dl>
                                        <div class="workflow-assignment-mobile-card__actions">
                                            <button
                                                class="button button--compact"
                                                type="button"
                                                on:click=move |_| selected_role_id.set(Some(role_id.clone()))
                                            >
                                                "View Details"
                                            </button>
                                        </div>
                                    </article>
                                }
                            })
                            .collect_view()
                            .into_any()
                    }
                }}
            </div>
        </div>
    }
}

#[component]
/// Renders the administration role detail panel view.
fn AdministrationRoleDetailPanel(
    detail: Option<AdminRoleDetail>,
    is_loading: bool,
    on_edit: impl Fn(leptos::ev::MouseEvent) + 'static + Copy,
) -> impl IntoView {
    if is_loading {
        view! {
            <section class="organization-state" aria-live="polite">
                <h3>"Loading role"</h3>
                <p>"Fetching role details."</p>
            </section>
        }
        .into_any()
    } else if let Some(detail) = detail {
        let capabilities = detail.capabilities.clone();
        let accounts = detail.assigned_accounts.clone();
        view! {
            <section
                class="organization-detail-card organization-detail-card--wide administration-role-detail-card"
                style="margin-top: 1rem;"
            >
                <div class="organization-detail-card__header">
                    <h2>{detail.name}</h2>
                    <button class="button button--secondary" type="button" on:click=on_edit>
                        "Edit Capabilities"
                    </button>
                </div>
                <table class="info-list-table">
                    <tbody>
                        <tr>
                            <th scope="row">"Capabilities"</th>
                            <td>{detail.capabilities.len()}</td>
                        </tr>
                        <tr>
                            <th scope="row">"Assigned Users"</th>
                            <td>{detail.assigned_accounts.len()}</td>
                        </tr>
                    </tbody>
                </table>
                <div class="administration-role-detail-grid">
                    <section class="organization-detail-card">
                        <h3>"Capabilities"</h3>
                        <AdminRoleCapabilityList capabilities/>
                    </section>
                    <section class="organization-detail-card">
                        <h3>"Assigned Users"</h3>
                        <AdminRoleAssignedAccounts accounts/>
                    </section>
                </div>
            </section>
        }
        .into_any()
    } else {
        view! {
            <section class="organization-state">
                <h3>"Select a role"</h3>
                <p>"Choose a role to review its capabilities and assigned users."</p>
            </section>
        }
        .into_any()
    }
}

#[component]
/// Renders the admin role capability list view.
fn AdminRoleCapabilityList(capabilities: Vec<AdminCapabilitySummary>) -> impl IntoView {
    if capabilities.is_empty() {
        view! { <p class="muted">"No capabilities assigned."</p> }.into_any()
    } else {
        view! {
            <table class="info-list-table">
                <tbody>
                {capabilities
                    .into_iter()
                    .map(|capability| view! {
                        <tr>
                            <th scope="row">{capability.key}</th>
                            <td>{capability.description}</td>
                        </tr>
                    })
                    .collect_view()}
                </tbody>
            </table>
        }
        .into_any()
    }
}

#[component]
/// Renders the admin role assigned accounts view.
fn AdminRoleAssignedAccounts(accounts: Vec<AdminAccountAssignmentSummary>) -> impl IntoView {
    if accounts.is_empty() {
        view! { <p class="muted">"No users assigned."</p> }.into_any()
    } else {
        view! {
            <table class="info-list-table">
                <tbody>
                {accounts
                    .into_iter()
                    .map(|account| view! {
                        <tr>
                            <th scope="row">{account.display_name}</th>
                            <td>{account.email}</td>
                        </tr>
                    })
                    .collect_view()}
                </tbody>
            </table>
        }
        .into_any()
    }
}

#[component]
/// Renders the admin role sheet view.
fn AdminRoleSheet(
    is_open: RwSignal<bool>,
    editing_role_id: RwSignal<Option<String>>,
    role_name: RwSignal<String>,
    capabilities: RwSignal<Vec<AdminCapabilitySummary>>,
    selected_capability_ids: RwSignal<Vec<String>>,
    capability_search: RwSignal<String>,
    is_saving: RwSignal<bool>,
    message: RwSignal<Option<String>>,
    on_close: impl Fn(leptos::ev::MouseEvent) + 'static + Copy + Send + Sync,
    on_save: impl Fn(leptos::ev::MouseEvent) + 'static + Copy + Send + Sync,
) -> impl IntoView {
    view! {
        <Portal>
            <Show when=move || is_open.get()>
                <section class="sheet-overlay administration-role-overlay" aria-label="Role editor">
                    <button class="sheet-overlay__scrim" type="button" aria-label="Close role editor" on:click=on_close></button>
                    <aside class="sheet-panel blurred-surface administration-role-sheet" role="dialog" aria-modal="true" aria-label="Role editor">
                        <div class="sheet-panel__actions">
                            <button class="icon-button sheet-panel__close" type="button" aria-label="Close role editor" title="Close role editor" on:click=on_close>
                                <X/>
                            </button>
                        </div>
                        <header class="sheet-panel__header">
                            <p>"Role Template"</p>
                            <h2>{move || if editing_role_id.get().is_some() { "Edit Role Capabilities" } else { "New Role" }}</h2>
                        </header>
                        <section class="sheet-panel__section">
                            <Show when=move || editing_role_id.get().is_none()>
                                <label class="form-field">
                                    <span>"Role Name"</span>
                                    <input
                                        type="text"
                                        placeholder="coordinator"
                                        prop:value=move || role_name.get()
                                        on:input=move |event| role_name.set(event_target_value(&event))
                                    />
                                </label>
                            </Show>
                            <label class="searchable-data-table__search searchable-data-table__control administration-role-sheet__search">
                                <Search class="searchable-data-table__control-icon"/>
                                <span class="sr-only">"Search capabilities"</span>
                                <input
                                    type="search"
                                    placeholder="Search capabilities"
                                    prop:value=move || capability_search.get()
                                    on:input=move |event| capability_search.set(event_target_value(&event))
                                />
                            </label>
                            <div class="checkbox-list permission-picker__list administration-role-capability-picker">
                                {move || {
                                    let query = capability_search.get();
                                    let selected = selected_capability_ids.get();
                                    let visible = capabilities
                                        .get()
                                        .into_iter()
                                        .filter(|capability| {
                                            text_matches(&query, &[capability.key.as_str(), capability.description.as_str()])
                                        })
                                        .collect::<Vec<_>>();
                                    if visible.is_empty() {
                                        view! { <p class="forms-list-mobile-empty">"No Capabilities to Display"</p> }.into_any()
                                    } else {
                                        visible
                                            .into_iter()
                                            .map(|capability| {
                                                let capability_id = capability.id.clone();
                                                let checked = selected.iter().any(|id| id == &capability.id);
                                                view! {
                                                    <label class="checkbox-list__item permission-picker__item">
                                                        <input
                                                            type="checkbox"
                                                            prop:checked=checked
                                                            on:change=move |event| {
                                                                toggle_string_selection(
                                                                    selected_capability_ids,
                                                                    capability_id.clone(),
                                                                    event_target_checked(&event),
                                                                );
                                                            }
                                                        />
                                                        <span>
                                                            <strong>{capability.key}</strong>
                                                            <small>{capability.description}</small>
                                                        </span>
                                                    </label>
                                                }
                                            })
                                            .collect_view()
                                            .into_any()
                                    }
                                }}
                            </div>
                            <Show when=move || message.get().is_some()>
                                <p class="form-message" role="status">{move || message.get().unwrap_or_default()}</p>
                            </Show>
                        </section>
                        <div class="form-actions">
                            <button class="button button--secondary" type="button" on:click=on_close>
                                "Cancel"
                            </button>
                            <button class="button" type="button" disabled=move || is_saving.get() on:click=on_save>
                                {move || if is_saving.get() { "Saving..." } else { "Save Role" }}
                            </button>
                        </div>
                    </aside>
                </section>
            </Show>
        </Portal>
    }
}

/// Loads the load admin roles context data.
fn load_admin_roles_context(
    roles: RwSignal<Vec<AdminRoleSummary>>,
    capabilities: RwSignal<Vec<AdminCapabilitySummary>>,
    selected_role_id: RwSignal<Option<String>>,
    is_loading: RwSignal<bool>,
    message: RwSignal<Option<String>>,
    preferred_role_id: Option<String>,
) {
    #[cfg(feature = "hydrate")]
    {
        leptos::task::spawn_local(async move {
            is_loading.set(true);
            message.set(None);
            let roles_response = gloo_net::http::Request::get("/api/admin/roles")
                .send()
                .await;
            let capabilities_response = gloo_net::http::Request::get("/api/admin/capabilities")
                .send()
                .await;

            match (roles_response, capabilities_response) {
                (Ok(roles_response), _) if roles_response.status() == 401 => {
                    is_loading.set(false);
                    redirect_to_login();
                }
                (Ok(roles_response), Ok(capabilities_response))
                    if roles_response.ok() && capabilities_response.ok() =>
                {
                    let loaded_roles = roles_response.json::<Vec<AdminRoleSummary>>().await;
                    let loaded_capabilities = capabilities_response
                        .json::<Vec<AdminCapabilitySummary>>()
                        .await;
                    match (loaded_roles, loaded_capabilities) {
                        (Ok(role_items), Ok(capability_items)) => {
                            let selected = preferred_role_id
                                .or_else(|| {
                                    selected_role_id
                                        .get_untracked()
                                        .filter(|id| role_items.iter().any(|role| role.id == *id))
                                })
                                .or_else(|| role_items.first().map(|role| role.id.clone()));
                            roles.set(role_items);
                            capabilities.set(capability_items);
                            selected_role_id.set(selected);
                            message.set(None);
                        }
                        (Err(error), _) => {
                            roles.set(Vec::new());
                            message.set(Some(format!("Unable to parse roles: {error}")));
                        }
                        (_, Err(error)) => {
                            capabilities.set(Vec::new());
                            message.set(Some(format!("Unable to parse capabilities: {error}")));
                        }
                    }
                    is_loading.set(false);
                }
                (Ok(roles_response), _) if !roles_response.ok() => {
                    roles.set(Vec::new());
                    message.set(Some(format!(
                        "Unable to load roles. Server returned {}.",
                        roles_response.status()
                    )));
                    is_loading.set(false);
                }
                (_, Ok(capabilities_response)) if !capabilities_response.ok() => {
                    capabilities.set(Vec::new());
                    message.set(Some(format!(
                        "Unable to load capabilities. Server returned {}.",
                        capabilities_response.status()
                    )));
                    is_loading.set(false);
                }
                (Err(error), _) => {
                    roles.set(Vec::new());
                    message.set(Some(format!("Unable to load roles: {error}")));
                    is_loading.set(false);
                }
                (_, Err(error)) => {
                    capabilities.set(Vec::new());
                    message.set(Some(format!("Unable to load capabilities: {error}")));
                    is_loading.set(false);
                }
                _ => {
                    message.set(Some("Unable to load role context.".into()));
                    is_loading.set(false);
                }
            }
        });
    }
    #[cfg(not(feature = "hydrate"))]
    {
        let _ = (
            roles,
            capabilities,
            selected_role_id,
            message,
            preferred_role_id,
        );
        is_loading.set(false);
    }
}

/// Loads the load admin role detail data.
fn load_admin_role_detail(
    role_id: String,
    detail: RwSignal<Option<AdminRoleDetail>>,
    is_loading: RwSignal<bool>,
    message: RwSignal<Option<String>>,
) {
    #[cfg(feature = "hydrate")]
    {
        leptos::task::spawn_local(async move {
            is_loading.set(true);
            match gloo_net::http::Request::get(&format!("/api/admin/roles/{role_id}"))
                .send()
                .await
            {
                Ok(response) if response.status() == 401 => {
                    is_loading.set(false);
                    redirect_to_login();
                }
                Ok(response) if response.ok() => match response.json::<AdminRoleDetail>().await {
                    Ok(loaded_detail) => {
                        detail.set(Some(loaded_detail));
                        message.set(None);
                        is_loading.set(false);
                    }
                    Err(error) => {
                        detail.set(None);
                        message.set(Some(format!("Unable to parse role detail: {error}")));
                        is_loading.set(false);
                    }
                },
                Ok(response) => {
                    detail.set(None);
                    message.set(Some(format!(
                        "Unable to load role detail. Server returned {}.",
                        response.status()
                    )));
                    is_loading.set(false);
                }
                Err(error) => {
                    detail.set(None);
                    message.set(Some(format!("Unable to load role detail: {error}")));
                    is_loading.set(false);
                }
            }
        });
    }
    #[cfg(not(feature = "hydrate"))]
    {
        let _ = (role_id, detail, message);
        is_loading.set(false);
    }
}

#[allow(clippy::too_many_arguments)]
/// Handles the save admin role behavior.
fn save_admin_role(
    editing_role_id: RwSignal<Option<String>>,
    role_name: RwSignal<String>,
    selected_capability_ids: RwSignal<Vec<String>>,
    is_saving: RwSignal<bool>,
    message: RwSignal<Option<String>>,
    sheet_open: RwSignal<bool>,
    roles: RwSignal<Vec<AdminRoleSummary>>,
    capabilities: RwSignal<Vec<AdminCapabilitySummary>>,
    selected_role_id: RwSignal<Option<String>>,
    selected_role_detail: RwSignal<Option<AdminRoleDetail>>,
    detail_loading: RwSignal<bool>,
) {
    #[cfg(feature = "hydrate")]
    {
        leptos::task::spawn_local(async move {
            is_saving.set(true);
            message.set(None);
            let capability_ids = selected_capability_ids.get();
            let editing_id = editing_role_id.get_untracked();
            let result = if let Some(role_id) = editing_id.clone() {
                let payload = UpdateAdminRolePayload { capability_ids };
                match serde_json::to_string(&payload) {
                    Ok(body) => {
                        send_json_id_request(
                            gloo_net::http::Request::put(&format!("/api/admin/roles/{role_id}")),
                            Some(body),
                            "Save role",
                        )
                        .await
                    }
                    Err(_) => Err("Role update could not be prepared.".into()),
                }
            } else {
                let name = role_name.get().trim().to_string();
                if name.is_empty() {
                    is_saving.set(false);
                    message.set(Some("Role name is required.".into()));
                    return;
                }
                let payload = CreateAdminRolePayload {
                    name,
                    capability_ids,
                };
                match serde_json::to_string(&payload) {
                    Ok(body) => {
                        send_json_id_request(
                            gloo_net::http::Request::post("/api/admin/roles"),
                            Some(body),
                            "Create role",
                        )
                        .await
                    }
                    Err(_) => Err("Role create request could not be prepared.".into()),
                }
            };

            match result {
                Ok(response) => {
                    let next_role_id = editing_id.unwrap_or(response.id);
                    sheet_open.set(false);
                    load_admin_roles_context(
                        roles,
                        capabilities,
                        selected_role_id,
                        is_saving,
                        message,
                        Some(next_role_id.clone()),
                    );
                    load_admin_role_detail(
                        next_role_id,
                        selected_role_detail,
                        detail_loading,
                        message,
                    );
                }
                Err(error) => {
                    message.set(Some(error));
                    is_saving.set(false);
                }
            }
        });
    }
    #[cfg(not(feature = "hydrate"))]
    let _ = (
        editing_role_id,
        role_name,
        selected_capability_ids,
        is_saving,
        message,
        sheet_open,
        roles,
        capabilities,
        selected_role_id,
        selected_role_detail,
        detail_loading,
    );
}
