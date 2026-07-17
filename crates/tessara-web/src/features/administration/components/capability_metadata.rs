//! Read-only capability scope and provenance metadata.

use crate::features::administration::display::{
    admin_capability_provenance_source_label, admin_capability_scope_label,
};
use crate::features::administration::models::{
    AdminCapabilityProvenanceSourceKind, AdminCapabilityProvenanceSummary, AdminCapabilityScopeMode,
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
                <dd><AdminCapabilityProvenance provenance show_digest/></dd>
            </div>
        </dl>
    }
}

#[component]
pub(crate) fn AdminCapabilityProvenance(
    provenance: Vec<AdminCapabilityProvenanceSummary>,
    #[prop(default = false)] show_digest: bool,
) -> impl IntoView {
    if provenance.is_empty() {
        view! { <span>"No provenance recorded"</span> }.into_any()
    } else {
        view! {
            <ul class="module-metadata-list capability-provenance-list">
                {provenance.into_iter().map(|record| {
                    let source_label = admin_capability_provenance_source_label(&record);
                    let source_digest = record.source_digest;
                    let source_kind = record.source_kind;
                    view! {
                        <li>
                            {match source_kind {
                                AdminCapabilityProvenanceSourceKind::Core => view! {
                                    <span><strong>"Authoritative source: "</strong>{source_label}</span>
                                }.into_any(),
                                AdminCapabilityProvenanceSourceKind::TransitionContribution => view! {
                                    <span>
                                        <strong>"Also declared by: "</strong>{source_label}
                                        " — Transitional in-process"
                                    </span>
                                }.into_any(),
                            }}
                            {show_digest.then(|| source_digest.map(|digest| view! {
                                <CapabilityDigest digest/>
                            }))}
                        </li>
                    }
                }).collect_view()}
            </ul>
        }.into_any()
    }
}

#[component]
fn CapabilityDigest(digest: String) -> impl IntoView {
    let preview = if digest.chars().count() > 19 {
        format!("{}…", digest.chars().take(19).collect::<String>())
    } else {
        digest.clone()
    };
    let digest_for_copy = digest.clone();
    view! {
        <span class="capability-digest">
            <details>
                <summary><code>{preview}</code></summary>
                <code class="data-table__secondary-text">{digest}</code>
            </details>
            <button
                class="button button--quiet button--compact"
                type="button"
                aria-label="Copy complete source digest"
                on:click=move |_| copy_digest(digest_for_copy.clone())
            >"Copy"</button>
        </span>
    }
}

#[cfg(feature = "hydrate")]
fn copy_digest(digest: String) {
    if let Some(window) = web_sys::window() {
        let clipboard = window.navigator().clipboard();
        wasm_bindgen_futures::spawn_local(async move {
            let _ = wasm_bindgen_futures::JsFuture::from(clipboard.write_text(&digest)).await;
        });
    }
}

#[cfg(not(feature = "hydrate"))]
fn copy_digest(_digest: String) {}

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
        assert!(html.contains("Also declared by:"));
        assert!(html.contains("Transitional in-process"));
        assert!(html.contains("sha256:fixture"));
    }
}
