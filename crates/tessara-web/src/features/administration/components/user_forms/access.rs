//! Administration user access form.

use super::super::{
    AdminCapabilityList, AdminDelegationChecklist, AdminDelegationList, AdminScopeNodeChecklist,
    AdminScopeNodeList,
};
use crate::features::administration::api::submit_update_admin_user_access;
use crate::features::administration::display::admin_editable_label;
use crate::features::administration::models::{AdminCapabilitySummary, AdminUserAccessDetail};
use crate::ui::InfoListTable;
use leptos::prelude::*;

#[component]
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
