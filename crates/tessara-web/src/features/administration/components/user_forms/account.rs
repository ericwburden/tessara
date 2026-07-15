//! Administration user account edit form.

use crate::features::administration::api::submit_update_admin_user;
use crate::features::administration::models::AdminRoleSummary;
use crate::features::administration::state::toggle_string_selection;
use crate::ui::PageHeader;
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
                    <aside class="administration-role-assignment-guidance" role="note">
                        <strong>"Role scope behavior"</strong>
                        <p>"Installation-global roles are always assigned across this installation; organization scope nodes do not limit them. Use a dedicated global module role for modules:read or modules:manage_navigation. It can coexist with separate scoped product roles on the same user."</p>
                    </aside>
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
    }
}
