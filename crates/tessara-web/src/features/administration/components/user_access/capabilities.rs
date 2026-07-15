//! Effective capability list for Administration user access.

use crate::features::administration::models::AdminCapabilitySummary;
use leptos::prelude::*;

use super::super::capability_metadata::AdminCapabilityMetadata;

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
                        let summary = capability_catalog
                            .iter()
                            .find(|summary| summary.key == capability)
                            .cloned();
                        view! {
                        <tr>
                            <th scope="row">{capability}</th>
                            <td>{if let Some(summary) = summary {
                                view! {
                                    <p>{summary.description}</p>
                                    <AdminCapabilityMetadata
                                        scope_mode=summary.scope_mode
                                        provenance=summary.provenance
                                        show_digest=true
                                    />
                                }.into_any()
                            } else {
                                view! { <span>"Granted"</span> }.into_any()
                            }}</td>
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
