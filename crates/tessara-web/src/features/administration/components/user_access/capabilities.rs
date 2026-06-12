//! Effective capability list for Administration user access.

use crate::features::administration::models::AdminCapabilitySummary;
use leptos::prelude::*;

#[component]
pub(crate) fn AdminCapabilityList(
    capabilities: Vec<String>,
    capability_catalog: Vec<AdminCapabilitySummary>,
) -> impl IntoView {
    if capabilities.is_empty() {
        view! { <p>"No effective capabilities."</p> }.into_any()
    } else {
        view! {
            <table class="info-list-table">
                <tbody>
                {capabilities
                    .into_iter()
                    .map(|capability| {
                        let description = capability_catalog
                            .iter()
                            .find(|summary| summary.key == capability)
                            .map(|summary| summary.description.clone())
                            .unwrap_or_else(|| "Granted".to_string());
                        view! {
                        <tr>
                            <th scope="row">{capability}</th>
                            <td>{description}</td>
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
