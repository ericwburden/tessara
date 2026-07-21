//! Selected role detail panel for Administration role management.

use crate::features::administration::models::{
    AdminAccountAssignmentSummary, AdminCapabilitySummary, AdminRoleDetail,
};
use icons::Pencil;
use leptos::prelude::*;
use tessara_web_ui::{SideSheet, SideSheetSide};

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
        let accounts_for_sheet = accounts.clone();
        let assigned_user_count = accounts.len();
        let assigned_users_open = RwSignal::new(false);
        let close_assigned_users = Callback::new(move |_| assigned_users_open.set(false));
        let role_name = detail.name.clone();
        let assigned_users_sheet_id = format!("role-{}-assigned-users", detail.id);
        view! {
            <section
                class="organization-detail-card organization-detail-card--wide administration-role-detail-card"
                style="margin-top: 1rem;"
            >
                <div class="organization-detail-card__header administration-role-detail-card__header">
                    <div class="administration-role-detail-card__heading">
                        <h2>{detail.name}</h2>
                        <p class="administration-role-detail-card__summary">
                            {detail.capabilities.len()} " Capabilities"
                            <span aria-hidden="true">" · "</span>
                            {assigned_user_count} " Assigned User" {if assigned_user_count == 1 { "" } else { "s" }}
                        </p>
                    </div>
                    <div class="administration-role-detail-card__actions">
                        <button class="button button--secondary" type="button" on:click=on_edit>
                            <Pencil class="button__icon"/>
                            "Edit Capabilities"
                        </button>
                        <button
                            class="button button--secondary"
                            type="button"
                            on:click=move |_| assigned_users_open.set(true)
                        >
                            "Assigned Users"
                        </button>
                    </div>
                </div>
                <section class="organization-detail-card administration-role-capabilities">
                    <h3>"Capabilities"</h3>
                    <AdminRoleCapabilityList capabilities/>
                </section>
            </section>

            <SideSheet
                id=assigned_users_sheet_id
                title=format!("{role_name} assigned users")
                description="Accounts currently assigned this role."
                eyebrow="Roles"
                open=Signal::derive(move || assigned_users_open.get())
                on_close=close_assigned_users
                side=SideSheetSide::End
                close_label="Close assigned users"
                class="administration-role-assigned-users-sheet"
            >
                <section class="sheet-panel__section">
                    <h3>"Assigned users"</h3>
                    <AdminRoleAssignedAccounts accounts=accounts_for_sheet.clone()/>
                </section>
            </SideSheet>
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
                        let capability_key = capability.key.clone();
                        let scope_mode = capability.scope_mode;
                        let provenance = capability.provenance;
                        view! {
                            <tr>
                                <th scope="row">
                                    <code>{capability.key}</code>
                                    <span class="data-table__secondary-text">{capability.description}</span>
                                </th>
                                <td><span class="status-badge">{admin_capability_scope_label(scope_mode)}</span></td>
                                <td><AdminCapabilityProvenance provenance show_digest=true context_id=capability_key/></td>
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
