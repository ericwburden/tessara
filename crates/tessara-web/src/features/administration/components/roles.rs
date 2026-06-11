//! Role-management Administration components.

use super::super::state::toggle_string_selection;
use crate::features::administration::models::{
    AdminAccountAssignmentSummary, AdminCapabilitySummary, AdminRoleDetail,
};
use crate::features::organization::AdminRoleSummary;
use crate::ui::{DataTable, DropdownMenu, TablePaginationFooter};
use crate::utils::pagination::pagination_page_start;
use crate::utils::text::text_matches;
use icons::{PanelRight, Search, X};
use leptos::portal::Portal;
use leptos::prelude::*;

#[component]
/// Renders the administration roles list view.
pub(crate) fn AdministrationRolesList(
    roles: Vec<AdminRoleSummary>,
    search: RwSignal<String>,
    selected_role_id: RwSignal<Option<String>>,
) -> impl IntoView {
    let table_roles = roles.clone();
    let card_roles = roles.clone();
    let page_size = RwSignal::new(10usize);
    let page_index = RwSignal::new(0usize);
    let total_count_value = roles.len();
    let total_count = Memo::new(move |_| total_count_value);

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
                                    .skip(pagination_page_start(total_count.get(), page_size.get(), page_index.get()))
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
                <TablePaginationFooter
                    aria_label="Administration roles table pagination"
                    item_label="roles"
                    total_count=total_count
                    page_size=page_size
                    page_index=page_index
                />
            </div>
            <div class="forms-list-mobile-cards administration-roles-mobile-cards">
                {move || {
                    if card_roles.is_empty() {
                        view! { <p class="forms-list-mobile-empty">"No Roles to Display"</p> }.into_any()
                    } else {
                        card_roles
                            .iter()
                            .skip(pagination_page_start(total_count.get(), page_size.get(), page_index.get()))
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
pub(crate) fn AdministrationRoleDetailPanel(
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
pub(crate) fn AdminRoleSheet(
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
