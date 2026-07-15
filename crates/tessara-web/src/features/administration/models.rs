//! View models for the Administration feature.
//!
//! Keep derived frontend models and lightweight state shapes here when they are shared by multiple Administration pages or helpers.

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub(crate) struct AdminCapabilitySummary {
    pub(crate) id: String,
    pub(crate) key: String,
    pub(crate) description: String,
    pub(crate) scope_mode: AdminCapabilityScopeMode,
    #[serde(default)]
    pub(crate) provenance: Vec<AdminCapabilityProvenanceSummary>,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "snake_case")]
pub(crate) enum AdminCapabilityScopeMode {
    ScopeAware,
    InstallationGlobal,
}

/// The scope profile of the capabilities currently selected for one role.
///
/// This is an editor model only. The API remains authoritative and validates
/// the submitted capability identifiers independently.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum AdminRoleCapabilityScopeSelection {
    Empty,
    ScopeAware,
    InstallationGlobal,
    AdminAllMixedException,
    Mixed,
}

impl AdminRoleCapabilityScopeSelection {
    pub(crate) const fn is_invalid(self) -> bool {
        matches!(self, Self::Mixed)
    }
}

/// Stable client-side rejection shown before a mixed-scope role can be sent.
pub(crate) const MIXED_ROLE_CAPABILITY_SCOPE_MESSAGE: &str = "A role cannot combine scope-aware and installation-global capabilities unless it contains admin:all. Create a dedicated installation-global role for module permissions and keep scoped product capabilities in a separate role.";

/// Classifies selected capabilities using the catalog's explicit scope-mode
/// metadata. Unknown identifiers are left to the API's identifier validation;
/// they must not be guessed from capability names.
pub(crate) fn admin_role_capability_scope_selection(
    catalog: &[AdminCapabilitySummary],
    selected_capability_ids: &[String],
) -> AdminRoleCapabilityScopeSelection {
    let mut has_scope_aware = false;
    let mut has_installation_global = false;
    let mut has_admin_all = false;

    for capability in catalog.iter().filter(|capability| {
        selected_capability_ids
            .iter()
            .any(|selected_id| selected_id == &capability.id)
    }) {
        has_admin_all |= capability.key == "admin:all";
        match capability.scope_mode {
            AdminCapabilityScopeMode::ScopeAware => has_scope_aware = true,
            AdminCapabilityScopeMode::InstallationGlobal => has_installation_global = true,
        }
    }

    match (has_scope_aware, has_installation_global, has_admin_all) {
        (false, false, _) => AdminRoleCapabilityScopeSelection::Empty,
        (true, false, _) => AdminRoleCapabilityScopeSelection::ScopeAware,
        (false, true, _) => AdminRoleCapabilityScopeSelection::InstallationGlobal,
        (true, true, true) => AdminRoleCapabilityScopeSelection::AdminAllMixedException,
        (true, true, false) => AdminRoleCapabilityScopeSelection::Mixed,
    }
}

pub(crate) fn validate_admin_role_capability_scope_selection(
    catalog: &[AdminCapabilitySummary],
    selected_capability_ids: &[String],
) -> Result<AdminRoleCapabilityScopeSelection, &'static str> {
    let selection = admin_role_capability_scope_selection(catalog, selected_capability_ids);
    if selection.is_invalid() {
        Err(MIXED_ROLE_CAPABILITY_SCOPE_MESSAGE)
    } else {
        Ok(selection)
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
pub(crate) struct AdminCapabilityProvenanceSummary {
    pub(crate) source_kind: AdminCapabilityProvenanceSourceKind,
    pub(crate) source_key: String,
    pub(crate) definition_id: Option<String>,
    pub(crate) definition_display_name: Option<String>,
    pub(crate) provider_state: AdminCapabilityProviderState,
    pub(crate) source_digest: Option<String>,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "snake_case")]
pub(crate) enum AdminCapabilityProvenanceSourceKind {
    Core,
    TransitionContribution,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "snake_case")]
pub(crate) enum AdminCapabilityProviderState {
    CoreAuthoritative,
    TransitionalInProcess,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub(crate) struct AdminRoleSummary {
    pub(crate) id: String,
    pub(crate) name: String,
    pub(crate) capability_count: i64,
    pub(crate) account_count: i64,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub(crate) struct AdminAccountAssignmentSummary {
    pub(crate) account_id: String,
    pub(crate) email: String,
    pub(crate) display_name: String,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub(crate) struct AdminRoleDetail {
    pub(crate) id: String,
    pub(crate) name: String,
    #[serde(default)]
    pub(crate) capabilities: Vec<AdminCapabilitySummary>,
    #[serde(default)]
    pub(crate) assigned_accounts: Vec<AdminAccountAssignmentSummary>,
}

#[derive(Clone, Debug, Serialize)]
#[cfg_attr(not(feature = "hydrate"), allow(dead_code))]
pub(crate) struct CreateAdminRolePayload {
    pub(crate) name: String,
    pub(crate) capability_ids: Vec<String>,
}

#[derive(Clone, Debug, Serialize)]
#[cfg_attr(not(feature = "hydrate"), allow(dead_code))]
pub(crate) struct UpdateAdminRolePayload {
    pub(crate) capability_ids: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub(crate) struct AdminUserSummary {
    pub(crate) id: String,
    pub(crate) email: String,
    pub(crate) display_name: String,
    pub(crate) is_active: bool,
    #[serde(default)]
    pub(crate) roles: Vec<AdminRoleSummary>,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub(crate) struct AdminScopeNodeSummary {
    pub(crate) node_id: String,
    pub(crate) node_name: String,
    pub(crate) node_type_name: String,
    pub(crate) parent_node_id: Option<String>,
    pub(crate) parent_node_name: Option<String>,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub(crate) struct AdminDelegationSummary {
    pub(crate) account_id: String,
    pub(crate) email: String,
    pub(crate) display_name: String,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub(crate) struct AdminUserDetail {
    pub(crate) id: String,
    pub(crate) email: String,
    pub(crate) display_name: String,
    pub(crate) is_active: bool,
    #[serde(default)]
    pub(crate) capabilities: Vec<String>,
    #[serde(default)]
    pub(crate) roles: Vec<AdminRoleSummary>,
    #[serde(default)]
    pub(crate) scope_nodes: Vec<AdminScopeNodeSummary>,
    #[serde(default)]
    pub(crate) delegations: Vec<AdminDelegationSummary>,
    #[serde(default)]
    pub(crate) delegated_by: Vec<AdminDelegationSummary>,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub(crate) struct AdminUserAccessDetail {
    pub(crate) account_id: String,
    pub(crate) email: String,
    pub(crate) display_name: String,
    #[serde(default)]
    pub(crate) capabilities: Vec<String>,
    #[serde(default)]
    pub(crate) scope_nodes: Vec<AdminScopeNodeSummary>,
    #[serde(default)]
    pub(crate) available_scope_nodes: Vec<AdminScopeNodeSummary>,
    #[serde(default)]
    pub(crate) delegations: Vec<AdminDelegationSummary>,
    #[serde(default)]
    pub(crate) available_delegate_accounts: Vec<AdminDelegationSummary>,
    pub(crate) scope_assignments_editable: bool,
    pub(crate) delegation_assignments_editable: bool,
}

#[derive(Clone, Debug, Serialize)]
#[cfg_attr(not(feature = "hydrate"), allow(dead_code))]
pub(crate) struct UpdateAdminUserPayload {
    pub(crate) email: String,
    pub(crate) display_name: String,
    pub(crate) password: Option<String>,
    pub(crate) is_active: bool,
    pub(crate) role_ids: Vec<String>,
}

#[derive(Clone, Debug, Serialize)]
#[cfg_attr(not(feature = "hydrate"), allow(dead_code))]
pub(crate) struct UpdateAdminUserAccessPayload {
    pub(crate) scope_node_ids: Vec<String>,
    pub(crate) delegate_account_ids: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub(crate) struct NodeTypeCatalogEntry {
    pub(crate) id: String,
    pub(crate) name: String,
    pub(crate) slug: String,
    pub(crate) singular_label: String,
    pub(crate) plural_label: String,
    pub(crate) is_root_type: bool,
    pub(crate) node_count: i64,
    #[serde(default)]
    pub(crate) parent_relationships: Vec<NodeTypePeerLink>,
    #[serde(default)]
    pub(crate) child_relationships: Vec<NodeTypePeerLink>,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub(crate) struct NodeTypePeerLink {
    pub(crate) node_type_id: String,
    pub(crate) node_type_name: String,
    pub(crate) node_type_slug: String,
    pub(crate) singular_label: String,
    pub(crate) plural_label: String,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
#[cfg_attr(not(feature = "hydrate"), allow(dead_code))]
pub(crate) struct NodeTypeDefinition {
    pub(crate) id: String,
    pub(crate) name: String,
    pub(crate) slug: String,
    pub(crate) singular_label: String,
    pub(crate) plural_label: String,
    pub(crate) is_root_type: bool,
    pub(crate) node_count: i64,
    #[serde(default)]
    pub(crate) parent_relationships: Vec<NodeTypePeerLink>,
    #[serde(default)]
    pub(crate) child_relationships: Vec<NodeTypePeerLink>,
    #[serde(default)]
    pub(crate) metadata_fields: Vec<NodeMetadataFieldSummary>,
    #[serde(default)]
    pub(crate) scoped_forms: Vec<NodeTypeFormLink>,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub(crate) struct NodeTypeFormLink {
    pub(crate) form_id: String,
    pub(crate) form_name: String,
    pub(crate) form_slug: String,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub(crate) struct NodeMetadataFieldSummary {
    pub(crate) id: String,
    pub(crate) node_type_id: String,
    pub(crate) node_type_name: String,
    pub(crate) key: String,
    pub(crate) label: String,
    pub(crate) field_type: String,
    pub(crate) required: bool,
}

#[derive(Clone, Debug, Serialize)]
pub(crate) struct NodeTypeUpsertRequest {
    pub(crate) name: String,
    pub(crate) slug: String,
    pub(crate) plural_label: Option<String>,
    pub(crate) parent_node_type_ids: Vec<String>,
    pub(crate) child_node_type_ids: Vec<String>,
}

#[derive(Clone, Debug, Serialize)]
#[cfg_attr(not(feature = "hydrate"), allow(dead_code))]
pub(crate) struct CreateNodeMetadataFieldRequest {
    pub(crate) node_type_id: String,
    pub(crate) key: String,
    pub(crate) label: String,
    pub(crate) field_type: String,
    pub(crate) required: bool,
}

#[derive(Clone, Debug, Serialize)]
#[cfg_attr(not(feature = "hydrate"), allow(dead_code))]
pub(crate) struct UpdateNodeMetadataFieldRequest {
    pub(crate) key: String,
    pub(crate) label: String,
    pub(crate) field_type: String,
    pub(crate) required: bool,
}

#[cfg(test)]
mod tests {
    use super::{
        AdminCapabilityScopeMode, AdminCapabilitySummary, AdminRoleCapabilityScopeSelection,
        MIXED_ROLE_CAPABILITY_SCOPE_MESSAGE, admin_role_capability_scope_selection,
        validate_admin_role_capability_scope_selection,
    };

    fn capability(
        id: &str,
        key: &str,
        scope_mode: AdminCapabilityScopeMode,
    ) -> AdminCapabilitySummary {
        AdminCapabilitySummary {
            id: id.into(),
            key: key.into(),
            description: format!("{key} description"),
            scope_mode,
            provenance: Vec::new(),
        }
    }

    #[test]
    fn role_scope_selection_uses_catalog_metadata_and_detects_mixed_bundles() {
        let catalog = vec![
            capability(
                "forms-read",
                "forms:read",
                AdminCapabilityScopeMode::ScopeAware,
            ),
            capability(
                "modules-read",
                "modules:read",
                AdminCapabilityScopeMode::InstallationGlobal,
            ),
            capability(
                "admin-all",
                "admin:all",
                AdminCapabilityScopeMode::InstallationGlobal,
            ),
        ];

        assert_eq!(
            admin_role_capability_scope_selection(&catalog, &[]),
            AdminRoleCapabilityScopeSelection::Empty
        );
        assert_eq!(
            admin_role_capability_scope_selection(&catalog, &["forms-read".into()]),
            AdminRoleCapabilityScopeSelection::ScopeAware
        );
        assert_eq!(
            admin_role_capability_scope_selection(&catalog, &["modules-read".into()]),
            AdminRoleCapabilityScopeSelection::InstallationGlobal
        );
        assert_eq!(
            admin_role_capability_scope_selection(
                &catalog,
                &["forms-read".into(), "modules-read".into()],
            ),
            AdminRoleCapabilityScopeSelection::Mixed
        );
        assert_eq!(
            validate_admin_role_capability_scope_selection(
                &catalog,
                &["forms-read".into(), "modules-read".into()],
            ),
            Err(MIXED_ROLE_CAPABILITY_SCOPE_MESSAGE)
        );
        assert_eq!(
            admin_role_capability_scope_selection(
                &catalog,
                &[
                    "forms-read".into(),
                    "modules-read".into(),
                    "admin-all".into(),
                ],
            ),
            AdminRoleCapabilityScopeSelection::AdminAllMixedException
        );
        assert_eq!(
            validate_admin_role_capability_scope_selection(
                &catalog,
                &[
                    "forms-read".into(),
                    "modules-read".into(),
                    "admin-all".into(),
                ],
            ),
            Ok(AdminRoleCapabilityScopeSelection::AdminAllMixedException)
        );
    }

    #[test]
    fn role_scope_selection_does_not_infer_scope_from_unknown_identifiers() {
        let catalog = vec![capability(
            "forms-read",
            "forms:read",
            AdminCapabilityScopeMode::ScopeAware,
        )];

        assert_eq!(
            admin_role_capability_scope_selection(&catalog, &["unknown-global-looking-id".into()],),
            AdminRoleCapabilityScopeSelection::Empty
        );
    }
}
