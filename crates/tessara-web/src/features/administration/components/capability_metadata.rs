//! Read-only capability scope and provenance metadata.

use crate::features::administration::display::{
    admin_capability_provenance_source_label, admin_capability_provider_state_label,
    admin_capability_scope_label,
};
use crate::features::administration::models::{
    AdminCapabilityProvenanceSummary, AdminCapabilityScopeMode,
};
use leptos::prelude::*;

#[component]
pub(crate) fn AdminCapabilityMetadata(
    scope_mode: AdminCapabilityScopeMode,
    provenance: Vec<AdminCapabilityProvenanceSummary>,
    #[prop(default = false)] show_digest: bool,
) -> impl IntoView {
    view! {
        <dl class="capability-metadata">
            <div>
                <dt>"Scope"</dt>
                <dd>{admin_capability_scope_label(scope_mode)}</dd>
            </div>
            <div>
                <dt>"Provenance"</dt>
                <dd>
                    {if provenance.is_empty() {
                        view! { <span>"No provenance recorded"</span> }.into_any()
                    } else {
                        view! {
                            <ul class="module-metadata-list">
                                {provenance.into_iter().map(|record| {
                                    let source_label = admin_capability_provenance_source_label(&record);
                                    let provider_state = admin_capability_provider_state_label(record.provider_state);
                                    let source_digest = record.source_digest;
                                    view! {
                                        <li>
                                            <span>{source_label}</span>
                                            " — "
                                            <span>{provider_state}</span>
                                            {show_digest.then(|| source_digest.map(|digest| view! {
                                                <code class="data-table__secondary-text">{digest}</code>
                                            }))}
                                        </li>
                                    }
                                }).collect_view()}
                            </ul>
                        }.into_any()
                    }}
                </dd>
            </div>
        </dl>
    }
}

#[cfg(test)]
mod tests {
    use leptos::prelude::*;

    use super::AdminCapabilityMetadata;
    use crate::features::administration::models::{
        AdminCapabilityProvenanceSourceKind, AdminCapabilityProvenanceSummary,
        AdminCapabilityProviderState, AdminCapabilityScopeMode,
    };

    #[test]
    fn metadata_html_exposes_scope_source_provider_state_and_digest() {
        let html = Owner::new().with(|| {
            view! {
                <AdminCapabilityMetadata
                    scope_mode=AdminCapabilityScopeMode::ScopeAware
                    provenance=vec![AdminCapabilityProvenanceSummary {
                        source_kind: AdminCapabilityProvenanceSourceKind::TransitionContribution,
                        source_key: "tessara.forms".into(),
                        definition_id: Some("tessara.forms".into()),
                        definition_display_name: Some("Forms".into()),
                        provider_state: AdminCapabilityProviderState::TransitionalInProcess,
                        source_digest: Some("sha256:fixture".into()),
                    }]
                    show_digest=true
                />
            }
            .to_html()
        });

        assert!(html.contains("Scope-aware"));
        assert!(html.contains("Forms (tessara.forms)"));
        assert!(html.contains("Transitional in-process"));
        assert!(html.contains("sha256:fixture"));
    }
}
