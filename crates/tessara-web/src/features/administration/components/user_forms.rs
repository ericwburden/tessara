//! User administration form components.

use super::{
    AdminCapabilityList, AdminDelegationChecklist, AdminDelegationList, AdminScopeNodeChecklist,
    AdminScopeNodeList,
};
use crate::features::administration::api::{
    submit_update_admin_user, submit_update_admin_user_access,
};
use crate::features::administration::display::admin_editable_label;
use crate::features::administration::models::{AdminCapabilitySummary, AdminUserAccessDetail};
use crate::features::administration::state::toggle_string_selection;
use crate::features::organization::AdminRoleSummary;
use crate::ui::{InfoListTable, PageHeader};
use leptos::prelude::*;

#[component]
/// Renders the administration user access form.
pub(crate) fn AdministrationUserAccessForm(
    account_id: String,
    access: AdminUserAccessDetail,
    capability_catalog: RwSignal<Vec<AdminCapabilitySummary>>,
    selected_scope_node_ids: RwSignal<Vec<String>>,
    selected_delegate_account_ids: RwSignal<Vec<String>>,
    is_saving: RwSignal<bool>,
    message: RwSignal<Option<String>>,
) -> impl IntoView {
    let edit_href = format!("/administration/users/{}/edit", access.account_id);
    let capability_count = access.capabilities.len().to_string();
    let scope_editing = admin_editable_label(access.scope_assignments_editable);
    let delegation_editing = admin_editable_label(access.delegation_assignments_editable);

    view! {
        <>
            <header class="page-header">
                <div>
                    <h2>{access.display_name.clone()}</h2>
                    <p>{access.email.clone()}</p>
                </div>
            </header>

            <form
                class="native-form administration-user-access-form"
                on:submit=move |event| {
                    event.prevent_default();
                    submit_update_admin_user_access(
                        account_id.clone(),
                        selected_scope_node_ids,
                        selected_delegate_account_ids,
                        is_saving,
                        message,
                    );
                }
            >
                <div class="organization-detail-content">
                    <section class="organization-detail-card organization-detail-card--wide">
                        <h3>"Effective Access"</h3>
                        <InfoListTable>
                            <tr>
                                <th scope="row">"Capabilities"</th>
                                <td>{capability_count}</td>
                            </tr>
                            <tr>
                                <th scope="row">"Scope Editing"</th>
                                <td>{scope_editing}</td>
                            </tr>
                            <tr>
                                <th scope="row">"Delegation Editing"</th>
                                <td>{delegation_editing}</td>
                            </tr>
                        </InfoListTable>
                    </section>

                    <section class="organization-detail-card">
                        <h3>"Scope Nodes"</h3>
                        {if access.scope_assignments_editable {
                            view! {
                                <AdminScopeNodeChecklist
                                    nodes=access.available_scope_nodes
                                    selected_node_ids=selected_scope_node_ids
                                />
                            }
                            .into_any()
                        } else {
                            view! { <AdminScopeNodeList nodes=access.scope_nodes/> }.into_any()
                        }}
                    </section>

                    <section class="organization-detail-card">
                        <h3>"Delegations"</h3>
                        {if access.delegation_assignments_editable {
                            view! {
                                <AdminDelegationChecklist
                                    delegations=access.available_delegate_accounts
                                    selected_delegate_account_ids=selected_delegate_account_ids
                                />
                            }
                            .into_any()
                        } else {
                            view! { <AdminDelegationList delegations=access.delegations empty_label="No delegated accounts."/> }.into_any()
                        }}
                    </section>

                    <section class="organization-detail-card organization-detail-card--wide">
                        <h3>"Capabilities"</h3>
                        <AdminCapabilityList
                            capabilities=access.capabilities
                            capability_catalog=capability_catalog.get()
                        />
                        <div class="form-actions">
                            <a class="button button--secondary" href=edit_href>"Edit Account Roles"</a>
                        </div>
                    </section>
                </div>
                {move || message
                    .get()
                    .map(|text| view! { <p class="form-message" role="status">{text}</p> })}
                <div class="form-actions">
                    <a class="button button--secondary" href="/administration/users">"Back to Users"</a>
                    <button class="button" type="submit" disabled=move || is_saving.get()>
                        {move || if is_saving.get() { "Saving..." } else { "Save Permissions" }}
                    </button>
                </div>
            </form>
        </>
    }
}

#[component]
/// Renders the administration user account edit form.
pub(crate) fn AdministrationUserAccountForm(
    account_id: String,
    roles: RwSignal<Vec<AdminRoleSummary>>,
    email: RwSignal<String>,
    display_name: RwSignal<String>,
    password: RwSignal<String>,
    is_active: RwSignal<bool>,
    selected_role_ids: RwSignal<Vec<String>>,
    is_saving: RwSignal<bool>,
    message: RwSignal<Option<String>>,
) -> impl IntoView {
    let cancel_href = format!("/administration/users/{account_id}");

    view! {
        <>
            <PageHeader
                title="Edit User"
                description="Update the account details, active status, password, and assigned roles."
            />
            <form
                class="native-form administration-user-form"
                on:submit=move |event| {
                    event.prevent_default();
                    submit_update_admin_user(
                        account_id.clone(),
                        email,
                        display_name,
                        password,
                        is_active,
                        selected_role_ids,
                        is_saving,
                        message,
                    );
                }
            >
                <div class="form-grid">
                    <label class="form-field" for="admin-user-display-name">
                        <span>"Display Name"</span>
                        <input
                            id="admin-user-display-name"
                            type="text"
                            autocomplete="name"
                            prop:value=move || display_name.get()
                            on:input=move |event| display_name.set(event_target_value(&event))
                            required
                        />
                    </label>
                    <label class="form-field" for="admin-user-email">
                        <span>"Email"</span>
                        <input
                            id="admin-user-email"
                            type="email"
                            autocomplete="email"
                            prop:value=move || email.get()
                            on:input=move |event| email.set(event_target_value(&event))
                            required
                        />
                    </label>
                    <label class="form-field" for="admin-user-password">
                        <span>"New Password"</span>
                        <input
                            id="admin-user-password"
                            type="password"
                            autocomplete="new-password"
                            placeholder="Leave blank to keep current password"
                            prop:value=move || password.get()
                            on:input=move |event| password.set(event_target_value(&event))
                        />
                    </label>
                    <label class="form-field">
                        <span>"Active"</span>
                        <label class="toggle-row toggle-row--compact">
                            <input
                                type="checkbox"
                                prop:checked=move || is_active.get()
                                on:change=move |event| is_active.set(event_target_checked(&event))
                            />
                            <span>{move || if is_active.get() { "Active" } else { "Inactive" }}</span>
                        </label>
                    </label>
                </div>

                <section class="form-section">
                    <h3>"Roles"</h3>
                    <div class="checkbox-list">
                        {move || {
                            let selected = selected_role_ids.get();
                            roles
                                .get()
                                .into_iter()
                                .map(|role| {
                                    let role_id = role.id.clone();
                                    let checked = selected.iter().any(|id| id == &role.id);
                                    view! {
                                        <label class="checkbox-list__item">
                                            <input
                                                type="checkbox"
                                                prop:checked=checked
                                                on:change=move |event| {
                                                    toggle_string_selection(
                                                        selected_role_ids,
                                                        role_id.clone(),
                                                        event_target_checked(&event),
                                                    );
                                                }
                                            />
                                            <span>
                                                <strong>{role.name}</strong>
                                                <small>{format!("{} capabilities, {} users", role.capability_count, role.account_count)}</small>
                                            </span>
                                        </label>
                                    }
                                })
                                .collect_view()
                        }}
                    </div>
                </section>

                {move || message
                    .get()
                    .map(|text| view! { <p class="form-message" role="status">{text}</p> })}

                <div class="form-actions">
                    <a class="button button--secondary" href=cancel_href.clone()>"Cancel"</a>
                    <button class="button" type="submit" disabled=move || is_saving.get()>
                        {move || if is_saving.get() { "Saving..." } else { "Save User" }}
                    </button>
                </div>
            </form>
        </>
    }
}
