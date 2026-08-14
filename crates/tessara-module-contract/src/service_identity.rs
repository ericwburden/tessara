use std::{collections::BTreeMap, fmt};

use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
use ed25519_dalek::VerifyingKey;
use serde::{Deserialize, Deserializer, Serialize, de};
use sha2::{Digest, Sha256};

use crate::{ModuleDefinitionId, ProtocolSignaturePurposeV1, PurposeBoundVerifyingKeyV1};

/// Environment variable carrying the versioned, definition-keyed module
/// service identity registry used by every process in one materialization.
pub const MODULE_SERVICE_IDENTITIES_ENVIRONMENT: &str = "TESSARA_MODULE_SERVICE_IDENTITIES";
pub const MODULE_SERVICE_IDENTITY_REGISTRY_SCHEMA_VERSION_V1: u16 = 1;

/// Public verification material for one independently deployed module
/// service. Materialization binds this definition-level declaration to the
/// exact selected Module Instance before runtime traffic is accepted.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ModuleServiceIdentityRegistrationV1 {
    pub key_id: String,
    pub public_key: String,
}

impl ModuleServiceIdentityRegistrationV1 {
    pub fn public_key_bytes(&self) -> Result<[u8; 32], ModuleServiceIdentityRegistryError> {
        URL_SAFE_NO_PAD
            .decode(&self.public_key)
            .map_err(|_| ModuleServiceIdentityRegistryError::InvalidPublicKey)?
            .try_into()
            .map_err(|_| ModuleServiceIdentityRegistryError::InvalidPublicKey)
    }

    pub fn public_key_fingerprint(&self) -> Result<String, ModuleServiceIdentityRegistryError> {
        let bytes = self.public_key_bytes()?;
        Ok(format!("sha256:{:x}", Sha256::digest(bytes)))
    }
}

/// One source-exact service identity registry shared by Core and module
/// providers. Unknown fields and duplicate definition keys are rejected.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ModuleServiceIdentityRegistryV1 {
    pub schema_version: u16,
    pub identities: BTreeMap<ModuleDefinitionId, ModuleServiceIdentityRegistrationV1>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ModuleServiceIdentityRegistryWireV1 {
    schema_version: u16,
    #[serde(deserialize_with = "deserialize_unique_identities")]
    identities: BTreeMap<ModuleDefinitionId, ModuleServiceIdentityRegistrationV1>,
}

impl<'de> Deserialize<'de> for ModuleServiceIdentityRegistryV1 {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let wire = ModuleServiceIdentityRegistryWireV1::deserialize(deserializer)?;
        let registry = Self {
            schema_version: wire.schema_version,
            identities: wire.identities,
        };
        registry.validate().map_err(de::Error::custom)?;
        Ok(registry)
    }
}

fn deserialize_unique_identities<'de, D>(
    deserializer: D,
) -> Result<BTreeMap<ModuleDefinitionId, ModuleServiceIdentityRegistrationV1>, D::Error>
where
    D: Deserializer<'de>,
{
    struct UniqueIdentityMapVisitor;

    impl<'de> de::Visitor<'de> for UniqueIdentityMapVisitor {
        type Value = BTreeMap<ModuleDefinitionId, ModuleServiceIdentityRegistrationV1>;

        fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
            formatter.write_str("a definition-keyed module service identity object")
        }

        fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
        where
            A: de::MapAccess<'de>,
        {
            let mut identities = BTreeMap::new();
            while let Some((definition_id, identity)) = map.next_entry()? {
                if identities.insert(definition_id, identity).is_some() {
                    return Err(de::Error::custom(
                        "module service identity definitions must be unique",
                    ));
                }
            }
            Ok(identities)
        }
    }

    deserializer.deserialize_map(UniqueIdentityMapVisitor)
}

impl ModuleServiceIdentityRegistryV1 {
    pub fn from_json(value: &str) -> Result<Self, ModuleServiceIdentityRegistryError> {
        serde_json::from_str(value)
            .map_err(|error| ModuleServiceIdentityRegistryError::Malformed(error.to_string()))
    }

    pub fn validate(&self) -> Result<(), ModuleServiceIdentityRegistryError> {
        if self.schema_version != MODULE_SERVICE_IDENTITY_REGISTRY_SCHEMA_VERSION_V1 {
            return Err(
                ModuleServiceIdentityRegistryError::UnsupportedSchemaVersion(self.schema_version),
            );
        }
        if self.identities.is_empty() {
            return Err(ModuleServiceIdentityRegistryError::EmptyRegistry);
        }
        for (definition_id, identity) in &self.identities {
            let key_id = identity.key_id.trim();
            if key_id.is_empty() || key_id.len() > 128 || key_id != identity.key_id {
                return Err(ModuleServiceIdentityRegistryError::InvalidKeyId(
                    definition_id.clone(),
                ));
            }
            let public_key = identity.public_key_bytes().map_err(|_| {
                ModuleServiceIdentityRegistryError::InvalidDefinitionPublicKey(
                    definition_id.clone(),
                )
            })?;
            VerifyingKey::from_bytes(&public_key).map_err(|_| {
                ModuleServiceIdentityRegistryError::InvalidDefinitionPublicKey(
                    definition_id.clone(),
                )
            })?;
        }
        Ok(())
    }

    pub fn identity(
        &self,
        definition_id: &ModuleDefinitionId,
    ) -> Option<&ModuleServiceIdentityRegistrationV1> {
        self.identities.get(definition_id)
    }

    pub fn module_service_verifier(
        &self,
        definition_id: &ModuleDefinitionId,
    ) -> Result<PurposeBoundVerifyingKeyV1, ModuleServiceIdentityRegistryError> {
        self.verifier_for_purpose(
            definition_id,
            ProtocolSignaturePurposeV1::ModuleServiceRequest,
        )
    }

    pub fn verifier_for_purpose(
        &self,
        definition_id: &ModuleDefinitionId,
        purpose: ProtocolSignaturePurposeV1,
    ) -> Result<PurposeBoundVerifyingKeyV1, ModuleServiceIdentityRegistryError> {
        let identity = self.identities.get(definition_id).ok_or_else(|| {
            ModuleServiceIdentityRegistryError::UnknownDefinition(definition_id.clone())
        })?;
        PurposeBoundVerifyingKeyV1::from_public_bytes(
            definition_id.as_str(),
            identity.key_id.clone(),
            purpose,
            identity.public_key_bytes().map_err(|_| {
                ModuleServiceIdentityRegistryError::InvalidDefinitionPublicKey(
                    definition_id.clone(),
                )
            })?,
        )
        .map_err(|_| {
            ModuleServiceIdentityRegistryError::InvalidDefinitionPublicKey(definition_id.clone())
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq, thiserror::Error)]
pub enum ModuleServiceIdentityRegistryError {
    #[error("module service identity registry is malformed: {0}")]
    Malformed(String),
    #[error("module service identity registry schema version {0} is unsupported")]
    UnsupportedSchemaVersion(u16),
    #[error("module service identity registry cannot be empty")]
    EmptyRegistry,
    #[error("module service identity key ID for '{0}' is invalid")]
    InvalidKeyId(ModuleDefinitionId),
    #[error("module service public key is invalid")]
    InvalidPublicKey,
    #[error("module service public key for '{0}' is invalid")]
    InvalidDefinitionPublicKey(ModuleDefinitionId),
    #[error("module service identity for '{0}' is not registered")]
    UnknownDefinition(ModuleDefinitionId),
}

#[cfg(test)]
mod tests {
    use super::*;

    const VALID_KEY: &str = "11qYAYKxCrfVS_7TyWQHOg7hcvPapiMlrwIaaPcHURo";

    #[test]
    fn exact_registry_parses_and_builds_a_purpose_bound_verifier() {
        let registry = ModuleServiceIdentityRegistryV1::from_json(&format!(
            r#"{{"schema_version":1,"identities":{{"tessara.components":{{"key_id":"component-development-v1","public_key":"{VALID_KEY}"}}}}}}"#
        ))
        .expect("valid registry");
        let definition = ModuleDefinitionId::new("tessara.components").unwrap();
        assert_eq!(
            registry.identity(&definition).unwrap().key_id,
            "component-development-v1"
        );
        registry
            .module_service_verifier(&definition)
            .expect("valid verifier");
    }

    #[test]
    fn registry_rejects_unknown_fields_duplicates_and_invalid_keys() {
        for invalid in [
            format!(
                r#"{{"schema_version":1,"identities":{{"tessara.components":{{"key_id":"component-development-v1","public_key":"{VALID_KEY}","unexpected":true}}}}}}"#
            ),
            format!(
                r#"{{"schema_version":1,"identities":{{"tessara.components":{{"key_id":"component-development-v1","public_key":"{VALID_KEY}"}},"tessara.components":{{"key_id":"other","public_key":"{VALID_KEY}"}}}}}}"#
            ),
            r#"{"schema_version":1,"identities":{"tessara.components":{"key_id":" component-development-v1","public_key":"bad"}}}"#.into(),
        ] {
            assert!(ModuleServiceIdentityRegistryV1::from_json(&invalid).is_err());
        }
    }
}
