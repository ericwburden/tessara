//! Selected role detail panel for Administration role management.

use crate::features::administration::models::{
    AdminAccountAssignmentSummary, AdminCapabilitySummary, AdminRoleDetail,
};
use leptos::prelude::*;

use super::super::capability_metadata::AdminCapabilityProvenance;
use crate::features::administration::display::admin_capability_scope_label;

#[component]
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
                    <section class="organization-detail-card administration-role-capabilities">
                        <h3>"Capabilities"</h3>
                        <AdminRoleCapabilityList capabilities/>
                    </section>
                    <section class="organization-detail-card administration-role-assigned-users">
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
fn AdminRoleCapabilityList(capabilities: Vec<AdminCapabilitySummary>) -> impl IntoView {
    if capabilities.is_empty() {
        view! { <p class="muted">"No capabilities assigned."</p> }.into_any()
    } else {
        view! {
            <table class="data-table administration-role-capability-table">
                <thead>
                    <tr>
                        <th scope="col">"Capability"</th>
                        <th scope="col">"Scope"</th>
                        <th scope="col">"Provenance"</th>
                    </tr>
                </thead>
                <tbody>
                {capabilities
                    .into_iter()
                    .map(|capability| {
                        let scope_mode = capability.scope_mode;
                        let provenance = capability.provenance;
                        view! {
                            <tr>
                                <th scope="row">
                                    <code>{capability.key}</code>
                                    <span class="data-table__secondary-text">{capability.description}</span>
                                </th>
                                <td><span class="status-badge">{admin_capability_scope_label(scope_mode)}</span></td>
                                <td><AdminCapabilityProvenance provenance show_digest=true/></td>
                            </tr>
                        }
                    })
                    .collect_view()}
                </tbody>
            </table>
        }
        .into_any()
    }
}

#[component]
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
