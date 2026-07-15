//! Display helpers for Administration feature screens.

use crate::features::administration::models::{
    AdminCapabilityProvenanceSourceKind, AdminCapabilityProvenanceSummary,
    AdminCapabilityProviderState, AdminCapabilityScopeMode, AdminUserSummary,
};

/// Returns the filter key for an admin user's active status.
pub(crate) fn admin_user_status_key(user: &AdminUserSummary) -> &'static str {
    if user.is_active { "active" } else { "inactive" }
}

/// Returns the visible label for an admin user's active status.
pub(crate) fn admin_user_status_label(user: &AdminUserSummary) -> &'static str {
    if user.is_active { "Active" } else { "Inactive" }
}

/// Formats the role names assigned to an admin user.
pub(crate) fn admin_user_role_names(user: &AdminUserSummary) -> String {
    if user.roles.is_empty() {
        "No roles".to_string()
    } else {
        user.roles
            .iter()
            .map(|role| role.name.as_str())
            .collect::<Vec<_>>()
            .join(", ")
    }
}

/// Returns the visible label for an editable administration scope.
pub(crate) fn admin_editable_label(is_editable: bool) -> &'static str {
    if is_editable {
        "Editable"
    } else {
        "Not editable"
    }
}

/// Human-readable scope metadata; this does not perform authorization.
pub(crate) const fn admin_capability_scope_label(
    scope_mode: AdminCapabilityScopeMode,
) -> &'static str {
    match scope_mode {
        AdminCapabilityScopeMode::ScopeAware => "Scope-aware",
        AdminCapabilityScopeMode::InstallationGlobal => "Installation-global",
    }
}

pub(crate) const fn admin_capability_provider_state_label(
    provider_state: AdminCapabilityProviderState,
) -> &'static str {
    match provider_state {
        AdminCapabilityProviderState::CoreAuthoritative => "Core authoritative",
        AdminCapabilityProviderState::TransitionalInProcess => "Transitional in-process",
    }
}

pub(crate) fn admin_capability_provenance_source_label(
    provenance: &AdminCapabilityProvenanceSummary,
) -> String {
    match provenance.source_kind {
        AdminCapabilityProvenanceSourceKind::Core => "Core".into(),
        AdminCapabilityProvenanceSourceKind::TransitionContribution => {
            let definition_id = provenance
                .definition_id
                .as_deref()
                .unwrap_or(provenance.source_key.as_str());
            provenance
                .definition_display_name
                .as_deref()
                .map(|name| format!("{name} ({definition_id})"))
                .unwrap_or_else(|| definition_id.to_string())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{
        admin_capability_provenance_source_label, admin_capability_provider_state_label,
        admin_capability_scope_label,
    };
    use crate::features::administration::models::{
        AdminCapabilityProvenanceSourceKind, AdminCapabilityProvenanceSummary,
        AdminCapabilityProviderState, AdminCapabilityScopeMode,
    };

    #[test]
    fn capability_scope_and_provenance_labels_do_not_collapse_dimensions() {
        let provenance = AdminCapabilityProvenanceSummary {
            source_kind: AdminCapabilityProvenanceSourceKind::TransitionContribution,
            source_key: "tessara.forms".into(),
            definition_id: Some("tessara.forms".into()),
            definition_display_name: Some("Forms".into()),
            provider_state: AdminCapabilityProviderState::TransitionalInProcess,
            source_digest: Some("sha256:fixture".into()),
        };

        assert_eq!(
            admin_capability_scope_label(AdminCapabilityScopeMode::ScopeAware),
            "Scope-aware"
        );
        assert_eq!(
            admin_capability_scope_label(AdminCapabilityScopeMode::InstallationGlobal),
            "Installation-global"
        );
        assert_eq!(
            admin_capability_provenance_source_label(&provenance),
            "Forms (tessara.forms)"
        );
        assert_eq!(
            admin_capability_provider_state_label(provenance.provider_state),
            "Transitional in-process"
        );
    }
}
