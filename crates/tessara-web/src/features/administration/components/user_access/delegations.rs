//! Delegated-account access display and picker components.

use super::super::super::state::toggle_string_selection;
use crate::features::administration::models::AdminDelegationSummary;
use crate::utils::text::text_matches;
use icons::Search;
use leptos::prelude::*;

#[component]
pub(crate) fn AdminDelegationList(
    delegations: Vec<AdminDelegationSummary>,
    empty_label: &'static str,
) -> impl IntoView {
    if delegations.is_empty() {
        view! { <p>{empty_label}</p> }.into_any()
    } else {
        view! {
            <table class="info-list-table">
                <tbody>
                    {delegations
                        .into_iter()
                        .map(|delegation| {
                            view! {
                                <tr>
                                    <th scope="row">
                                        <a class="data-table__primary-link" href=format!("/administration/users/{}", delegation.account_id)>
                                            {delegation.display_name}
                                        </a>
                                    </th>
                                    <td>{delegation.email}</td>
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
pub(crate) fn AdminDelegationChecklist(
    delegations: Vec<AdminDelegationSummary>,
    selected_delegate_account_ids: RwSignal<Vec<String>>,
) -> impl IntoView {
    let search = RwSignal::new(String::new());
    let delegation_count = delegations.len();
    if delegations.is_empty() {
        view! { <p>"No delegate accounts are available."</p> }.into_any()
    } else {
        view! {
            <div class="permission-picker">
                <label class="searchable-data-table__search searchable-data-table__control">
                    <Search class="searchable-data-table__control-icon"/>
                    <span class="sr-only">"Search delegate accounts"</span>
                    <input
                        type="search"
                        placeholder=format!("Search {delegation_count} accounts")
                        prop:value=move || search.get()
                        on:input=move |event| search.set(event_target_value(&event))
                    />
                </label>
                <table class="info-list-table permission-picker__table">
                    <tbody>
                    {move || {
                        let query = search.get();
                        delegations
                            .iter()
                            .filter(|delegation| {
                                text_matches(
                                    &query,
                                    &[delegation.display_name.as_str(), delegation.email.as_str()],
                                )
                            })
                            .cloned()
                            .map(|delegation| {
                                let account_id = delegation.account_id.clone();
                                let selected = selected_delegate_account_ids
                                    .get()
                                    .iter()
                                    .any(|selected_id| selected_id == &delegation.account_id);
                                let checkbox_label = format!(
                                    "Delegate access to {} ({})",
                                    delegation.display_name,
                                    delegation.email
                                );
                                view! {
                                    <tr>
                                        <td class="data-table__cell--center">
                                            <input
                                                type="checkbox"
                                                aria-label=checkbox_label
                                                prop:checked=selected
                                                on:change=move |event| {
                                                    toggle_string_selection(
                                                        selected_delegate_account_ids,
                                                        account_id.clone(),
                                                        event_target_checked(&event),
                                                    );
                                                }
                                            />
                                        </td>
                                        <th scope="row">{delegation.display_name}</th>
                                        <td>{delegation.email}</td>
                                    </tr>
                                }
                            })
                            .collect_view()
                    }}
                    </tbody>
                </table>
            </div>
        }
        .into_any()
    }
}
