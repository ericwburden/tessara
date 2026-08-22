//! Policy-neutral Core control-plane provider contracts used by modules.
//!
//! These responses contain display and validation facts only. They do not grant
//! authority and deliberately omit credentials and principal contact data.

use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub const CONTROL_PLANE_SCHEMA_VERSION: u16 = 1;
pub const SCOPE_CATALOG_CONTRACT_ID: &str = "tessara.core.scope-catalog";
pub const SCOPE_CATALOG_CONTRACT_VERSION: &str = "1.0.0";
pub const SCOPE_CATALOG_BINDING_KEY: &str = "tessara.datasets.scope-catalog";
pub const SCOPE_CATALOG_ACTION: &str = "core.scope_catalog";
pub const SCOPE_CATALOG_PATH: &str = "/api/private/core/scope-catalog";
pub const SCOPE_CATALOG_MEDIA_TYPE: &str =
    "application/vnd.tessara.core.scope-catalog+json;version=1";
pub const PRINCIPAL_DISPLAY_CONTRACT_ID: &str = "tessara.core.principal-display-catalog";
pub const PRINCIPAL_DISPLAY_CONTRACT_VERSION: &str = "1.0.0";
pub const PRINCIPAL_DISPLAY_BINDING_KEY: &str = "tessara.datasets.principal-display-catalog";
pub const PRINCIPAL_DISPLAY_ACTION: &str = "core.principal_display_catalog";
pub const PRINCIPAL_DISPLAY_PATH: &str = "/api/private/core/principal-display-catalog";
pub const PRINCIPAL_DISPLAY_MEDIA_TYPE: &str =
    "application/vnd.tessara.core.principal-display-catalog+json;version=1";

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ControlPlaneCatalogAction {
    Catalog,
    ResolveRequestedSet,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ScopeCatalogRequest {
    #[serde(deserialize_with = "deserialize_schema_version")]
    pub schema_version: u16,
    pub action: ControlPlaneCatalogAction,
    #[serde(default)]
    pub node_ids: Vec<Uuid>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ScopeCatalogNode {
    pub node_id: Uuid,
    pub parent_node_id: Option<Uuid>,
    pub node_type_key: String,
    pub node_type_name: String,
    pub display_label: String,
    pub node_path: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ScopeCatalogResponse {
    #[serde(deserialize_with = "deserialize_schema_version")]
    pub schema_version: u16,
    pub nodes: Vec<ScopeCatalogNode>,
    pub requested_set_revision: String,
    pub requested_set_digest: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PrincipalDisplayCatalogRequest {
    #[serde(deserialize_with = "deserialize_schema_version")]
    pub schema_version: u16,
    pub action: ControlPlaneCatalogAction,
    #[serde(default)]
    pub principal_ids: Vec<Uuid>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PrincipalDisplayCatalogItem {
    pub principal_id: Uuid,
    pub display_name: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PrincipalDisplayCatalogResponse {
    #[serde(deserialize_with = "deserialize_schema_version")]
    pub schema_version: u16,
    pub principals: Vec<PrincipalDisplayCatalogItem>,
    pub requested_set_revision: String,
    pub requested_set_digest: String,
}

fn deserialize_schema_version<'de, D>(deserializer: D) -> Result<u16, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let version = u16::deserialize(deserializer)?;
    if version != CONTROL_PLANE_SCHEMA_VERSION {
        return Err(serde::de::Error::custom(format!(
            "Control-plane schema version {version} is unsupported; expected {CONTROL_PLANE_SCHEMA_VERSION}"
        )));
    }
    Ok(version)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn principal_contract_has_no_contact_or_authority_facade() {
        let wire = json!({
            "schema_version": 1,
            "principals": [],
            "requested_set_revision": "7",
            "requested_set_digest": "sha256:requested"
        });
        assert!(serde_json::from_value::<PrincipalDisplayCatalogResponse>(wire.clone()).is_ok());
        let mut leaked = wire;
        leaked["email"] = json!("hidden@example.test");
        assert!(serde_json::from_value::<PrincipalDisplayCatalogResponse>(leaked).is_err());

        let catalog: ScopeCatalogRequest = serde_json::from_value(json!({
            "schema_version": 1,
            "action": "catalog"
        }))
        .unwrap();
        assert_eq!(catalog.action, ControlPlaneCatalogAction::Catalog);
        assert!(catalog.node_ids.is_empty());
        assert!(
            serde_json::from_value::<ScopeCatalogRequest>(json!({
                "schema_version": 1,
                "action": "catalog",
                "include_unauthorized_ancestors": true
            }))
            .is_err()
        );
    }
}
