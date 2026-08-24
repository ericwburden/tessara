//! Canonical Forms-owned immutable FormVersion catalog and schema contract.

use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};
use uuid::Uuid;

pub const FORM_VERSION_SCHEMA_CONTRACT_ID: &str = "tessara.forms.form-version-schema";
pub const FORM_VERSION_SCHEMA_CONTRACT_VERSION: &str = "1.0.0";
pub const FORM_VERSION_SCHEMA_VERSION: u16 = 1;
pub const FORM_VERSION_SCHEMA_BINDING_KEY: &str = "tessara.datasets.form-version-schema";
pub const FORM_VERSION_SCHEMA_MEDIA_TYPE: &str =
    "application/vnd.tessara.forms.form-version-schema+json;version=1";
pub const FORM_VERSION_CATALOG_ACTION: &str = "forms.form_version_catalog";
pub const FORM_VERSION_SCHEMA_ACTION: &str = "forms.form_version_schema";
pub const FORM_VERSION_CATALOG_PATH: &str = "/api/private/forms/form-version-catalog";
pub const FORM_VERSION_SCHEMA_PATH: &str = "/api/private/forms/form-version-schema";

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FormVersionSchemaAction {
    Catalog,
    ResolveSchema,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FormVersionCatalogRequest {
    #[serde(deserialize_with = "deserialize_schema_version")]
    pub schema_version: u16,
    pub action: FormVersionSchemaAction,
    pub cursor: Option<String>,
    pub limit: u16,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FormVersionCatalogItem {
    pub form_id: Uuid,
    pub form_version_id: Uuid,
    pub form_name: String,
    pub form_slug: String,
    pub version_label: Option<String>,
    pub version_major: Option<i32>,
    pub published: bool,
    pub field_count: i64,
    pub content_revision: String,
    pub content_digest: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FormVersionCatalogResponse {
    #[serde(deserialize_with = "deserialize_schema_version")]
    pub schema_version: u16,
    pub items: Vec<FormVersionCatalogItem>,
    pub next_cursor: Option<String>,
    pub requested_set_digest: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FormVersionSchemaRequest {
    #[serde(deserialize_with = "deserialize_schema_version")]
    pub schema_version: u16,
    pub action: FormVersionSchemaAction,
    pub form_version_id: Uuid,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FormVersionField {
    pub field_id: Uuid,
    pub key: String,
    pub label: String,
    pub field_type: String,
    pub required: bool,
    pub options: Vec<Value>,
    pub section_id: Option<Uuid>,
    pub position: i32,
    pub grid_row: i32,
    pub grid_column: i32,
    pub grid_width: i32,
    pub grid_height: i32,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FormVersionSection {
    pub section_id: Uuid,
    pub key: String,
    pub label: String,
    pub description: String,
    pub position: i32,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FormVersionSchemaResponse {
    #[serde(deserialize_with = "deserialize_schema_version")]
    pub schema_version: u16,
    pub form_id: Uuid,
    pub form_version_id: Uuid,
    pub form_name: String,
    pub form_slug: String,
    pub version_label: Option<String>,
    pub version_major: Option<i32>,
    pub source_scope_node_ids: Vec<Uuid>,
    pub source_scope_revision: String,
    pub source_scope_digest: String,
    pub content_revision: String,
    pub content_digest: String,
    pub sections: Vec<FormVersionSection>,
    pub fields: Vec<FormVersionField>,
}

impl FormVersionCatalogResponse {
    /// Validates the catalog's canonical identities, ordering, cursor, and
    /// digest-shaped revision facts. Schema content is validated by resolving
    /// an item and calling [`FormVersionSchemaResponse::validate_for`].
    pub fn validate(&self) -> Result<(), FormVersionSchemaValidationError> {
        if self.schema_version != FORM_VERSION_SCHEMA_VERSION {
            return Err(FormVersionSchemaValidationError::SchemaVersion);
        }
        if !is_sha256_digest(&self.requested_set_digest) {
            return Err(FormVersionSchemaValidationError::RequestedSetDigest);
        }
        let mut previous = None;
        for item in &self.items {
            if item.form_id.is_nil()
                || item.form_version_id.is_nil()
                || is_blank(&item.form_name)
                || is_blank(&item.form_slug)
                || item.version_label.as_deref().is_some_and(is_blank)
                || item.version_major.is_some_and(|major| major <= 0)
                || !item.published
                || item.field_count < 0
            {
                return Err(FormVersionSchemaValidationError::CatalogIdentity);
            }
            if previous.is_some_and(|id| id >= item.form_version_id) {
                return Err(FormVersionSchemaValidationError::CatalogOrdering);
            }
            if item.content_revision != item.content_digest
                || !is_sha256_digest(&item.content_digest)
            {
                return Err(FormVersionSchemaValidationError::ContentDigestMismatch);
            }
            previous = Some(item.form_version_id);
        }
        if let Some(cursor) = self.next_cursor.as_deref() {
            let cursor = Uuid::parse_str(cursor)
                .map_err(|_| FormVersionSchemaValidationError::CatalogCursor)?;
            if self.items.last().map(|item| item.form_version_id) != Some(cursor) {
                return Err(FormVersionSchemaValidationError::CatalogCursor);
            }
        }
        Ok(())
    }
}

impl FormVersionSchemaResponse {
    /// Recomputes both source-scope and complete-schema digests after first
    /// enforcing the canonical identity and ordering invariants.
    pub fn with_recomputed_digests(mut self) -> Result<Self, FormVersionSchemaValidationError> {
        self.validate_identity_and_shape(self.form_version_id)?;
        let source_scope_digest = self.canonical_source_scope_digest()?;
        self.source_scope_revision = source_scope_digest.clone();
        self.source_scope_digest = source_scope_digest;
        let content_digest = self.canonical_content_digest()?;
        self.content_revision = content_digest.clone();
        self.content_digest = content_digest;
        self.validate_for(self.form_version_id)?;
        Ok(self)
    }

    /// Validates that the response is the exact requested immutable
    /// FormVersion and that neither its source scope nor schema content was
    /// altered after the provider computed its canonical digest.
    pub fn validate_for(
        &self,
        requested_form_version_id: Uuid,
    ) -> Result<(), FormVersionSchemaValidationError> {
        self.validate_identity_and_shape(requested_form_version_id)?;
        let expected_scope_digest = self.canonical_source_scope_digest()?;
        if self.source_scope_digest != expected_scope_digest {
            return Err(FormVersionSchemaValidationError::SourceScopeDigestMismatch);
        }
        if self.source_scope_revision != self.source_scope_digest {
            return Err(FormVersionSchemaValidationError::SourceScopeRevisionMismatch);
        }
        let expected_content_digest = self.canonical_content_digest()?;
        if self.content_digest != expected_content_digest {
            return Err(FormVersionSchemaValidationError::ContentDigestMismatch);
        }
        if self.content_revision != self.content_digest {
            return Err(FormVersionSchemaValidationError::ContentRevisionMismatch);
        }
        Ok(())
    }

    pub fn canonical_source_scope_digest(
        &self,
    ) -> Result<String, FormVersionSchemaValidationError> {
        canonical_digest(&self.source_scope_node_ids)
    }

    pub fn canonical_content_digest(&self) -> Result<String, FormVersionSchemaValidationError> {
        #[derive(Serialize)]
        struct Content<'a> {
            schema_version: u16,
            form_id: &'a Uuid,
            form_version_id: &'a Uuid,
            form_name: &'a str,
            form_slug: &'a str,
            version_label: &'a Option<String>,
            version_major: Option<i32>,
            source_scope_node_ids: &'a [Uuid],
            source_scope_revision: &'a str,
            source_scope_digest: &'a str,
            sections: &'a [FormVersionSection],
            fields: &'a [FormVersionField],
        }

        canonical_digest(&Content {
            schema_version: self.schema_version,
            form_id: &self.form_id,
            form_version_id: &self.form_version_id,
            form_name: &self.form_name,
            form_slug: &self.form_slug,
            version_label: &self.version_label,
            version_major: self.version_major,
            source_scope_node_ids: &self.source_scope_node_ids,
            source_scope_revision: &self.source_scope_revision,
            source_scope_digest: &self.source_scope_digest,
            sections: &self.sections,
            fields: &self.fields,
        })
    }

    fn validate_identity_and_shape(
        &self,
        requested_form_version_id: Uuid,
    ) -> Result<(), FormVersionSchemaValidationError> {
        if self.schema_version != FORM_VERSION_SCHEMA_VERSION {
            return Err(FormVersionSchemaValidationError::SchemaVersion);
        }
        if requested_form_version_id.is_nil()
            || self.form_id.is_nil()
            || self.form_version_id != requested_form_version_id
        {
            return Err(FormVersionSchemaValidationError::FormVersionIdentity);
        }
        if is_blank(&self.form_name)
            || is_blank(&self.form_slug)
            || self.version_label.as_deref().is_some_and(is_blank)
            || self.version_major.is_some_and(|major| major <= 0)
        {
            return Err(FormVersionSchemaValidationError::FormIdentityContent);
        }
        if self.source_scope_node_ids.is_empty() {
            return Err(FormVersionSchemaValidationError::EmptySourceScope);
        }
        let mut previous_scope = None;
        for scope_id in &self.source_scope_node_ids {
            if scope_id.is_nil() || previous_scope.is_some_and(|previous| previous >= *scope_id) {
                return Err(FormVersionSchemaValidationError::SourceScopeOrdering);
            }
            previous_scope = Some(*scope_id);
        }

        let mut section_ids = BTreeSet::new();
        let mut section_order = BTreeMap::new();
        let mut previous_section = None;
        for section in &self.sections {
            if section.section_id.is_nil()
                || section.position < 0
                || is_blank(&section.key)
                || is_blank(&section.label)
                || !section_ids.insert(section.section_id)
            {
                return Err(FormVersionSchemaValidationError::SectionIdentity);
            }
            let order = (section.position, section.section_id);
            if previous_section.is_some_and(|previous| previous >= order) {
                return Err(FormVersionSchemaValidationError::SectionOrdering);
            }
            previous_section = Some(order);
            section_order.insert(section.section_id, order);
        }

        let mut field_ids = BTreeSet::new();
        let mut field_keys = BTreeSet::new();
        let mut previous_field = None;
        for field in &self.fields {
            let Some(section_id) = field.section_id else {
                return Err(FormVersionSchemaValidationError::FieldSectionIdentity);
            };
            let Some((section_position, _)) = section_order.get(&section_id).copied() else {
                return Err(FormVersionSchemaValidationError::FieldSectionIdentity);
            };
            if field.field_id.is_nil()
                || field.position < 0
                || field.grid_row < 1
                || field.grid_column < 1
                || field.grid_width < 1
                || field.grid_height < 1
                || is_blank(&field.key)
                || is_blank(&field.label)
                || is_blank(&field.field_type)
                || !field_ids.insert(field.field_id)
                || !field_keys.insert(field.key.as_str())
            {
                return Err(FormVersionSchemaValidationError::FieldIdentity);
            }
            let order = (
                section_position,
                section_id,
                field.position,
                field.grid_row,
                field.grid_column,
                field.label.as_str(),
                field.key.as_str(),
                field.field_id,
            );
            if previous_field.is_some_and(|previous| previous >= order) {
                return Err(FormVersionSchemaValidationError::FieldOrdering);
            }
            previous_field = Some(order);
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, thiserror::Error)]
pub enum FormVersionSchemaValidationError {
    #[error("FormVersion schema version is unsupported")]
    SchemaVersion,
    #[error("FormVersion catalog identity is invalid")]
    CatalogIdentity,
    #[error("FormVersion catalog ordering is invalid")]
    CatalogOrdering,
    #[error("FormVersion catalog cursor is invalid")]
    CatalogCursor,
    #[error("FormVersion requested-set digest is invalid")]
    RequestedSetDigest,
    #[error("FormVersion identity does not match the request")]
    FormVersionIdentity,
    #[error("Form identity content is invalid")]
    FormIdentityContent,
    #[error("Form source scope must not be empty")]
    EmptySourceScope,
    #[error("Form source scope identities must be non-nil, unique, and ordered")]
    SourceScopeOrdering,
    #[error("Form source scope digest is invalid")]
    SourceScopeDigestMismatch,
    #[error("Form source scope revision is invalid")]
    SourceScopeRevisionMismatch,
    #[error("Form section identity is invalid")]
    SectionIdentity,
    #[error("Form sections are not canonically ordered")]
    SectionOrdering,
    #[error("Form field identity or layout is invalid")]
    FieldIdentity,
    #[error("Form field section identity is invalid")]
    FieldSectionIdentity,
    #[error("Form fields are not canonically ordered")]
    FieldOrdering,
    #[error("FormVersion content digest is invalid")]
    ContentDigestMismatch,
    #[error("FormVersion content revision is invalid")]
    ContentRevisionMismatch,
    #[error("FormVersion content could not be canonicalized")]
    Canonicalization,
}

fn canonical_digest<T: Serialize + ?Sized>(
    value: &T,
) -> Result<String, FormVersionSchemaValidationError> {
    let encoded =
        serde_jcs::to_vec(value).map_err(|_| FormVersionSchemaValidationError::Canonicalization)?;
    Ok(format!("sha256:{:x}", Sha256::digest(encoded)))
}

fn is_blank(value: &str) -> bool {
    value.trim().is_empty()
}

fn is_sha256_digest(value: &str) -> bool {
    value.strip_prefix("sha256:").is_some_and(|digest| {
        digest.len() == 64
            && digest
                .bytes()
                .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    })
}

fn deserialize_schema_version<'de, D>(deserializer: D) -> Result<u16, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let version = u16::deserialize(deserializer)?;
    if version != FORM_VERSION_SCHEMA_VERSION {
        return Err(serde::de::Error::custom(format!(
            "FormVersion schema version {version} is unsupported; expected {FORM_VERSION_SCHEMA_VERSION}"
        )));
    }
    Ok(version)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn schema() -> FormVersionSchemaResponse {
        let section_id = Uuid::from_u128(3);
        FormVersionSchemaResponse {
            schema_version: FORM_VERSION_SCHEMA_VERSION,
            form_id: Uuid::from_u128(1),
            form_version_id: Uuid::from_u128(2),
            form_name: "Enrollment".into(),
            form_slug: "enrollment".into(),
            version_label: Some("Published".into()),
            version_major: Some(1),
            source_scope_node_ids: vec![Uuid::from_u128(10), Uuid::from_u128(11)],
            source_scope_revision: String::new(),
            source_scope_digest: String::new(),
            content_revision: String::new(),
            content_digest: String::new(),
            sections: vec![FormVersionSection {
                section_id,
                key: section_id.to_string(),
                label: "Applicant".into(),
                description: "Applicant details".into(),
                position: 0,
            }],
            fields: vec![FormVersionField {
                field_id: Uuid::from_u128(4),
                key: "given_name".into(),
                label: "Given name".into(),
                field_type: "text".into(),
                required: true,
                options: Vec::new(),
                section_id: Some(section_id),
                position: 0,
                grid_row: 1,
                grid_column: 1,
                grid_width: 6,
                grid_height: 1,
            }],
        }
        .with_recomputed_digests()
        .unwrap()
    }

    #[test]
    fn schema_is_content_bound_and_strict() {
        let wire = json!({
            "schema_version": 1,
            "action": "resolve_schema",
            "form_version_id": Uuid::from_u128(1)
        });
        assert!(serde_json::from_value::<FormVersionSchemaRequest>(wire.clone()).is_ok());
        let mut unknown = wire;
        unknown["fallback_to_latest"] = json!(true);
        assert!(serde_json::from_value::<FormVersionSchemaRequest>(unknown).is_err());
    }

    #[test]
    fn catalog_exposes_only_real_form_version_identity_and_editor_facts() {
        let content_digest = format!("sha256:{}", "a".repeat(64));
        let item = FormVersionCatalogItem {
            form_id: Uuid::from_u128(1),
            form_version_id: Uuid::from_u128(2),
            form_name: "Enrollment".into(),
            form_slug: "enrollment".into(),
            version_label: Some("Published".into()),
            version_major: Some(1),
            published: true,
            field_count: 4,
            content_revision: content_digest.clone(),
            content_digest,
        };
        let wire = serde_json::to_value(&item).unwrap();
        assert!(wire.get("version_number").is_none());
        assert_eq!(wire["version_label"], "Published");
        assert_eq!(wire["field_count"], 4);
        assert_eq!(
            serde_json::from_value::<FormVersionCatalogItem>(wire).unwrap(),
            item
        );

        let catalog = FormVersionCatalogResponse {
            schema_version: FORM_VERSION_SCHEMA_VERSION,
            items: vec![item.clone()],
            next_cursor: Some(item.form_version_id.to_string()),
            requested_set_digest: format!("sha256:{}", "b".repeat(64)),
        };
        assert!(catalog.validate().is_ok());
    }

    #[test]
    fn schema_validation_binds_request_scope_identity_layout_and_content() {
        let schema = schema();
        assert_eq!(schema.source_scope_revision, schema.source_scope_digest);
        assert_eq!(schema.content_revision, schema.content_digest);
        assert!(schema.validate_for(Uuid::from_u128(2)).is_ok());
        assert_eq!(
            schema.validate_for(Uuid::from_u128(99)),
            Err(FormVersionSchemaValidationError::FormVersionIdentity)
        );

        let mut tampered_scope = schema.clone();
        tampered_scope.source_scope_node_ids[0] = Uuid::from_u128(9);
        assert_eq!(
            tampered_scope.validate_for(Uuid::from_u128(2)),
            Err(FormVersionSchemaValidationError::SourceScopeDigestMismatch)
        );

        let mut tampered_content = schema.clone();
        tampered_content.form_slug = "different".into();
        assert_eq!(
            tampered_content.validate_for(Uuid::from_u128(2)),
            Err(FormVersionSchemaValidationError::ContentDigestMismatch)
        );

        let mut invalid_layout = schema;
        invalid_layout.fields[0].grid_column = 0;
        assert_eq!(
            invalid_layout.validate_for(Uuid::from_u128(2)),
            Err(FormVersionSchemaValidationError::FieldIdentity)
        );
    }

    #[test]
    fn schema_requires_canonical_complete_source_scope() {
        let mut unordered = schema();
        unordered.source_scope_node_ids.reverse();
        assert_eq!(
            unordered.validate_for(Uuid::from_u128(2)),
            Err(FormVersionSchemaValidationError::SourceScopeOrdering)
        );

        let mut empty = schema();
        empty.source_scope_node_ids.clear();
        assert_eq!(
            empty.validate_for(Uuid::from_u128(2)),
            Err(FormVersionSchemaValidationError::EmptySourceScope)
        );
    }
}
