//! Administration user account edit form.

use crate::features::administration::api::submit_update_admin_user;
use crate::features::administration::display::admin_capability_scope_label;
use crate::features::administration::models::AdminRoleSummary;
use crate::features::administration::state::toggle_string_selection;
use crate::ui::{PageHeader, Timestamp};
use leptos::prelude::*;

#[component]
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
                    <div class="form-field administration-user-status-field">
                        <span id="admin-user-status-label">"Status"</span>
                        <button
                            id="admin-user-status"
                            class=move || {
                                if is_active.get() {
                                    "button administration-user-status-toggle is-active"
                                } else {
                                    "button administration-user-status-toggle is-inactive"
                                }
                            }
                            type="button"
                            aria-pressed=move || is_active.get()
                            aria-labelledby="admin-user-status-label admin-user-status"
                            on:click=move |_| is_active.update(|active| *active = !*active)
                        >
                            {move || if is_active.get() { "Active" } else { "Inactive" }}
                        </button>
                    </div>
                </div>

                <section class="form-section">
                    <h3>"Roles"</h3>
                    <aside class="administration-role-assignment-guidance" role="note">
                        <strong>"Role scope behavior"</strong>
                        <p>"Installation-global roles are always assigned across this installation; organization scope nodes do not limit them. Use a dedicated global module role for modules:read or modules:manage_navigation. It can coexist with separate scoped product roles on the same user."</p>
                    </aside>
                    <div class="checkbox-list administration-role-assignment-list">
                        <div class="administration-role-assignment__header" aria-hidden="true">
                            <span>"Role"</span>
                            <span>"Scope"</span>
                            <span>"Assigned on"</span>
                            <span>"Details"</span>
                        </div>
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
                                            <span class="administration-role-assignment__row">
                                                <span class="administration-role-assignment__identity">
                                                    <strong>{role.name}</strong>
                                                </span>
                                                <span data-label="Scope">
                                                    {role.scope_mode
                                                        .map(admin_capability_scope_label)
                                                        .unwrap_or("No capabilities")}
                                                </span>
                                                <span data-label="Assigned on">
                                                    {if checked {
                                                        match role.assigned_at {
                                                            Some(assigned_at) => view! { <Timestamp value=assigned_at/> }.into_any(),
                                                            None => view! { <strong>"Pending save"</strong> }.into_any(),
                                                        }
                                                    } else {
                                                        view! { <span aria-label="Not assigned">"—"</span> }.into_any()
                                                    }}
                                                </span>
                                                <span data-label="Details">
                                                    {format!("{} capabilities", role.capability_count)}
                                                </span>
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

#[cfg(test)]
mod tests {
    use leptos::prelude::*;

    use super::AdministrationUserAccountForm;
    use crate::features::administration::models::AdminRoleSummary;

    #[test]
    fn account_role_assignment_explains_global_module_and_scoped_role_composition() {
        let html = Owner::new().with(|| {
            let roles = RwSignal::new(vec![AdminRoleSummary {
                id: "module-reader".into(),
                name: "Module reader".into(),
                capability_count: 1,
                account_count: 0,
                scope_mode: Some(crate::features::administration::models::AdminCapabilityScopeMode::InstallationGlobal),
                assigned_at: None,
            }]);

            view! {
                <AdministrationUserAccountForm
                    account_id="account-1".to_string()
                    roles
                    email=RwSignal::new("reader@example.com".to_string())
                    display_name=RwSignal::new("Module Reader".to_string())
                    password=RwSignal::new(String::new())
                    is_active=RwSignal::new(true)
                    selected_role_ids=RwSignal::new(Vec::new())
                    is_saving=RwSignal::new(false)
                    message=RwSignal::new(None::<String>)
                />
            }
            .to_html()
        });

        assert!(html.contains("Role scope behavior"));
        assert!(html.contains("always assigned across this installation"));
        assert!(html.contains("dedicated global module role"));
        assert!(html.contains("modules:read"));
        assert!(html.contains("modules:manage_navigation"));
        assert!(html.contains("separate scoped product roles"));
        assert!(html.contains("Assigned on"));
        assert!(html.contains("Installation-global"));
        assert!(html.contains("Details"));
        assert!(html.contains("administration-user-status-toggle is-active"));
        assert!(html.contains("Active"));
    }
}
