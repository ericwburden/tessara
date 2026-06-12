//! Mobile card renderer for administration users.

use super::super::super::display::{
    admin_user_role_names, admin_user_status_key, admin_user_status_label,
};
use crate::features::administration::models::AdminUserSummary;
use crate::features::shared::status_badge_class;
use crate::utils::pagination::pagination_page_start;
use leptos::prelude::*;

#[component]
pub(super) fn AdministrationUserMobileCards(
    users: Vec<AdminUserSummary>,
    page_size: RwSignal<usize>,
    page_index: RwSignal<usize>,
) -> impl IntoView {
    view! {
        <div class="forms-list-mobile-cards administration-users-mobile-cards">
            {move || {
                if users.is_empty() {
                    view! { <p class="forms-list-mobile-empty">"No Users to Display"</p> }.into_any()
                } else {
                    let total_count = users.len();
                    let start = pagination_page_start(total_count, page_size.get(), page_index.get());
                    users
                        .iter()
                        .skip(start)
                        .take(page_size.get())
                        .cloned()
                        .map(|user| {
                            let status_key = admin_user_status_key(&user);
                            let status_label = admin_user_status_label(&user);
                            let role_names = admin_user_role_names(&user);
                            let detail_href = format!("/administration/users/{}", user.id);
                            let edit_href = format!("/administration/users/{}/edit", user.id);
                            view! {
                                <article class="forms-list-mobile-card administration-user-mobile-card">
                                    <div class="forms-list-mobile-card__header">
                                        <div>
                                            <h3><a href=detail_href.clone()>{user.display_name}</a></h3>
                                            <span>{user.email}</span>
                                        </div>
                                        <span class=status_badge_class(status_key)>{status_label}</span>
                                    </div>
                                    <dl>
                                        <div>
                                            <dt>"Roles"</dt>
                                            <dd>{role_names}</dd>
                                        </div>
                                        <div>
                                            <dt>"Role Count"</dt>
                                            <dd>{user.roles.len()}</dd>
                                        </div>
                                    </dl>
                                    <div class="workflow-assignment-mobile-card__actions">
                                        <a class="button button--compact" href=detail_href>"View Details"</a>
                                        <a class="button button--compact button--secondary" href=edit_href>"Edit Account"</a>
                                    </div>
                                </article>
                            }
                        })
                        .collect_view()
                        .into_any()
                }
            }}
        </div>
    }
}
