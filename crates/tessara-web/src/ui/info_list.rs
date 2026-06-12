//! Definition-list style information components.
//!
//! Keep reusable label/value display primitives here for detail panels and summary sections across features.

use leptos::prelude::*;

#[component]
pub fn InfoListTable(children: Children) -> impl IntoView {
    view! {
        <table class="info-list-table">
            <tbody>{children()}</tbody>
        </table>
    }
}

#[component]
pub fn InfoRow(label: &'static str, value: &'static str) -> impl IntoView {
    view! {
        <tr>
            <th scope="row">{label}</th>
            <td>{value}</td>
        </tr>
    }
}
