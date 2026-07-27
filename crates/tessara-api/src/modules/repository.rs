//! PostgreSQL persistence for the Core-owned transition catalog.

use chrono::{DateTime, Utc};
use serde_json::Value;
use sqlx::{PgPool, Postgres, Row, Transaction};
use uuid::Uuid;

use super::catalog::CatalogFinding;

const CATALOG_SYNC_ADVISORY_LOCK: i64 = 0x5445_5353_4152_3641;

#[derive(Clone, Debug)]
pub(crate) struct CurrentCatalogEntry {
    pub(crate) source_id: Uuid,
    pub(crate) projection_id: Uuid,
    pub(crate) source_digest: String,
    pub(crate) source_bytes: Vec<u8>,
    pub(crate) schema_version: i32,
    pub(crate) content_type: String,
    pub(crate) projection_source_id: Uuid,
    pub(crate) projection_installation_id: Uuid,
    pub(crate) normalized_projection: Value,
    pub(crate) provider_eligible: bool,
    pub(crate) supervisor_materializable: bool,
}

#[derive(Clone, Debug)]
pub(crate) struct InstallationRow {
    pub(crate) id: Uuid,
    pub(crate) created_at: DateTime<Utc>,
}

#[derive(Clone, Debug)]
pub(crate) struct CoreRuntimeObservationRow {
    pub(crate) installation_id: Uuid,
    pub(crate) provenance: String,
    pub(crate) observed_version: String,
    pub(crate) finding_code: String,
    pub(crate) observed_at: DateTime<Utc>,
}

#[derive(Clone, Debug)]
pub(crate) struct CatalogProjectionRow {
    pub(crate) definition_id: String,
    pub(crate) display_name: String,
    pub(crate) current: CurrentCatalogEntry,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct StoredNavigationContribution {
    pub(crate) contribution_id: String,
    pub(crate) definition_id: String,
    pub(crate) descriptor_source_id: Uuid,
    pub(crate) destination: String,
    pub(crate) label: String,
    pub(crate) group_name: String,
    pub(crate) reorder_band: String,
    pub(crate) source_order_hint: i32,
    pub(crate) default_policy_order: i32,
    pub(crate) required_capabilities_any_of: Value,
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg(test)]
pub(crate) struct NavigationPolicyEntryRow {
    pub(crate) contribution_id: String,
    pub(crate) definition_id: String,
    pub(crate) destination: String,
    pub(crate) label: String,
    pub(crate) group_name: String,
    pub(crate) reorder_band: String,
    pub(crate) source_order_hint: i32,
    pub(crate) default_policy_order: i32,
    pub(crate) required_capabilities_any_of: Value,
    pub(crate) visible: bool,
    pub(crate) policy_order: i32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct NavigationGroupRow {
    pub(crate) group_id: String,
    pub(crate) label: String,
    pub(crate) display_order: i32,
    pub(crate) owner: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct NavigationPlacementRow {
    pub(crate) destination_id: String,
    pub(crate) group_id: String,
    pub(crate) visible: bool,
    pub(crate) display_order: i32,
}

#[derive(Clone, Debug)]
pub(crate) struct CapabilityRow {
    pub(crate) id: Uuid,
    pub(crate) key: String,
    pub(crate) description: String,
    pub(crate) scope_mode: String,
}

#[derive(Clone, Debug)]
pub(crate) struct DescriptorSourceRow {
    pub(crate) id: Uuid,
    pub(crate) definition_id: String,
    pub(crate) schema_version: i32,
    pub(crate) source_digest: String,
    pub(crate) source_bytes: Vec<u8>,
    pub(crate) content_type: String,
}

#[derive(Clone, Debug)]
pub(crate) struct StoredProjectionRow {
    pub(crate) id: Uuid,
    pub(crate) source_id: Uuid,
    pub(crate) installation_id: Uuid,
    pub(crate) normalized_projection: Value,
    pub(crate) provider_eligible: bool,
    pub(crate) supervisor_materializable: bool,
}

#[derive(Clone, Debug)]
pub(crate) struct NavigationRecord<'a> {
    pub(crate) contribution_id: &'a str,
    pub(crate) definition_id: &'a str,
    pub(crate) descriptor_source_id: Uuid,
    pub(crate) destination: &'a str,
    pub(crate) label: &'a str,
    pub(crate) group_name: &'a str,
    pub(crate) reorder_band: &'a str,
    pub(crate) source_order_hint: i32,
    pub(crate) default_policy_order: i32,
    pub(crate) required_capabilities_any_of: Value,
}

pub(crate) async fn acquire_catalog_sync_lock(
    tx: &mut Transaction<'_, Postgres>,
) -> Result<(), sqlx::Error> {
    sqlx::query("SELECT pg_advisory_xact_lock($1)")
        .bind(CATALOG_SYNC_ADVISORY_LOCK)
        .execute(&mut **tx)
        .await?;
    Ok(())
}

pub(crate) async fn installation_id(
    tx: &mut Transaction<'_, Postgres>,
) -> Result<Uuid, sqlx::Error> {
    sqlx::query_scalar("SELECT id FROM application_installations WHERE singleton = true")
        .fetch_one(&mut **tx)
        .await
}

pub(crate) async fn load_installation(
    tx: &mut Transaction<'_, Postgres>,
) -> Result<InstallationRow, sqlx::Error> {
    let row =
        sqlx::query("SELECT id, created_at FROM application_installations WHERE singleton = true")
            .fetch_one(&mut **tx)
            .await?;
    Ok(InstallationRow {
        id: row.try_get("id")?,
        created_at: row.try_get("created_at")?,
    })
}

pub(crate) async fn load_core_runtime_observation(
    tx: &mut Transaction<'_, Postgres>,
    installation_id: Uuid,
) -> Result<CoreRuntimeObservationRow, sqlx::Error> {
    let row = sqlx::query(
        r#"
        SELECT installation_id, provenance, observed_version, finding_code, observed_at
        FROM core_runtime_observations
        WHERE installation_id = $1
        "#,
    )
    .bind(installation_id)
    .fetch_one(&mut **tx)
    .await?;
    Ok(CoreRuntimeObservationRow {
        installation_id: row.try_get("installation_id")?,
        provenance: row.try_get("provenance")?,
        observed_version: row.try_get("observed_version")?,
        finding_code: row.try_get("finding_code")?,
        observed_at: row.try_get("observed_at")?,
    })
}

pub(crate) async fn ensure_core_runtime_observation(
    tx: &mut Transaction<'_, Postgres>,
    installation_id: Uuid,
    observed_version: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        INSERT INTO core_runtime_observations (
            installation_id,
            provenance,
            observed_version,
            finding_code
        )
        VALUES ($1, 'development_unresolved', $2, 'core_release_provenance_unresolved')
        ON CONFLICT (installation_id) DO UPDATE SET
            observed_version = EXCLUDED.observed_version,
            observed_at = now()
        WHERE core_runtime_observations.observed_version IS DISTINCT FROM EXCLUDED.observed_version
        "#,
    )
    .bind(installation_id)
    .bind(observed_version)
    .execute(&mut **tx)
    .await?;
    Ok(())
}

pub(crate) async fn ensure_module_capability(
    tx: &mut Transaction<'_, Postgres>,
    key: &str,
    description: &str,
) -> Result<CapabilityRow, sqlx::Error> {
    sqlx::query(
        r#"
        INSERT INTO capabilities (key, description, scope_mode)
        VALUES ($1, $2, 'installation_global')
        ON CONFLICT (key) DO NOTHING
        "#,
    )
    .bind(key)
    .bind(description)
    .execute(&mut **tx)
    .await?;

    load_capability(tx, key)
        .await?
        .ok_or(sqlx::Error::RowNotFound)
}

/// Registers a product capability declared by an installed module manifest.
///
/// Module product capabilities are scope-aware by default: Core binds them to
/// Organization roots when roles are assigned. Updating the description keeps
/// the assignable catalog aligned with the accepted manifest while preserving
/// the stable capability identity and existing role memberships.
pub(crate) async fn ensure_declared_module_capability(
    tx: &mut Transaction<'_, Postgres>,
    key: &str,
    description: &str,
) -> Result<CapabilityRow, sqlx::Error> {
    sqlx::query(
        r#"
        INSERT INTO capabilities (key, description, scope_mode)
        VALUES ($1, $2, 'scope_aware')
        ON CONFLICT (key) DO UPDATE SET
            description = EXCLUDED.description,
            scope_mode = EXCLUDED.scope_mode
        "#,
    )
    .bind(key)
    .bind(description)
    .execute(&mut **tx)
    .await?;

    load_capability(tx, key)
        .await?
        .ok_or(sqlx::Error::RowNotFound)
}

pub(crate) async fn load_capability(
    tx: &mut Transaction<'_, Postgres>,
    key: &str,
) -> Result<Option<CapabilityRow>, sqlx::Error> {
    let row = sqlx::query(
        r#"
        SELECT id, key, description, scope_mode
        FROM capabilities
        WHERE key = $1
        "#,
    )
    .bind(key)
    .fetch_optional(&mut **tx)
    .await?;

    row.map(|row| {
        Ok(CapabilityRow {
            id: row.try_get("id")?,
            key: row.try_get("key")?,
            description: row.try_get("description")?,
            scope_mode: row.try_get("scope_mode")?,
        })
    })
    .transpose()
}

pub(crate) async fn ensure_core_capability_provenance(
    tx: &mut Transaction<'_, Postgres>,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        INSERT INTO capability_provenance (
            capability_id,
            source_kind,
            source_key,
            provider_state
        )
        SELECT id, 'core', 'core', 'core_authoritative'
        FROM capabilities
        ON CONFLICT (capability_id, source_key) DO NOTHING
        "#,
    )
    .execute(&mut **tx)
    .await?;
    Ok(())
}

pub(crate) async fn ensure_reservation(
    tx: &mut Transaction<'_, Postgres>,
    definition_id: &str,
    display_name: &str,
) -> Result<String, sqlx::Error> {
    sqlx::query(
        r#"
        INSERT INTO module_definition_reservations (definition_id, display_name)
        VALUES ($1, $2)
        ON CONFLICT (definition_id) DO NOTHING
        "#,
    )
    .bind(definition_id)
    .bind(display_name)
    .execute(&mut **tx)
    .await?;
    sqlx::query_scalar(
        "SELECT display_name FROM module_definition_reservations WHERE definition_id = $1",
    )
    .bind(definition_id)
    .fetch_one(&mut **tx)
    .await
}

pub(crate) async fn load_current_catalog_entry(
    tx: &mut Transaction<'_, Postgres>,
    definition_id: &str,
) -> Result<Option<CurrentCatalogEntry>, sqlx::Error> {
    let row = sqlx::query(
        r#"
        SELECT
            transition_catalog_current.source_id,
            transition_catalog_current.projection_id,
            transition_descriptor_sources.source_digest,
            transition_descriptor_sources.source_bytes,
            transition_descriptor_sources.schema_version,
            transition_descriptor_sources.content_type,
            transition_catalog_projections.source_id AS projection_source_id,
            transition_catalog_projections.installation_id AS projection_installation_id,
            transition_catalog_projections.normalized_projection,
            transition_catalog_projections.provider_eligible,
            transition_catalog_projections.supervisor_materializable
        FROM transition_catalog_current
        JOIN transition_descriptor_sources
          ON transition_descriptor_sources.id = transition_catalog_current.source_id
        JOIN transition_catalog_projections
          ON transition_catalog_projections.id = transition_catalog_current.projection_id
        WHERE transition_catalog_current.definition_id = $1
        "#,
    )
    .bind(definition_id)
    .fetch_optional(&mut **tx)
    .await?;

    row.map(|row| {
        Ok(CurrentCatalogEntry {
            source_id: row.try_get("source_id")?,
            projection_id: row.try_get("projection_id")?,
            source_digest: row.try_get("source_digest")?,
            source_bytes: row.try_get("source_bytes")?,
            schema_version: row.try_get("schema_version")?,
            content_type: row.try_get("content_type")?,
            projection_source_id: row.try_get("projection_source_id")?,
            projection_installation_id: row.try_get("projection_installation_id")?,
            normalized_projection: row.try_get("normalized_projection")?,
            provider_eligible: row.try_get("provider_eligible")?,
            supervisor_materializable: row.try_get("supervisor_materializable")?,
        })
    })
    .transpose()
}

pub(crate) async fn load_findings(
    tx: &mut Transaction<'_, Postgres>,
    projection_id: Uuid,
) -> Result<Vec<CatalogFinding>, sqlx::Error> {
    let rows = sqlx::query(
        r#"
        SELECT code, path, message
        FROM module_catalog_findings
        WHERE projection_id = $1
        ORDER BY ordinal
        "#,
    )
    .bind(projection_id)
    .fetch_all(&mut **tx)
    .await?;
    rows.into_iter()
        .map(|row| {
            Ok(CatalogFinding {
                code: row.try_get("code")?,
                path: row.try_get("path")?,
                message: row.try_get("message")?,
            })
        })
        .collect()
}

pub(crate) async fn load_current_definition_ids(
    tx: &mut Transaction<'_, Postgres>,
) -> Result<Vec<String>, sqlx::Error> {
    sqlx::query_scalar(
        "SELECT definition_id FROM transition_catalog_current ORDER BY definition_id",
    )
    .fetch_all(&mut **tx)
    .await
}

pub(crate) async fn load_catalog_projections(
    tx: &mut Transaction<'_, Postgres>,
) -> Result<Vec<CatalogProjectionRow>, sqlx::Error> {
    let definition_rows = sqlx::query(
        r#"
        SELECT current.definition_id, reservations.display_name
        FROM transition_catalog_current AS current
        JOIN module_definition_reservations AS reservations
          ON reservations.definition_id = current.definition_id
        ORDER BY current.definition_id
        "#,
    )
    .fetch_all(&mut **tx)
    .await?;

    let mut projections = Vec::with_capacity(definition_rows.len());
    for row in definition_rows {
        let definition_id: String = row.try_get("definition_id")?;
        let current = load_current_catalog_entry(tx, &definition_id)
            .await?
            .ok_or(sqlx::Error::RowNotFound)?;
        projections.push(CatalogProjectionRow {
            definition_id,
            display_name: row.try_get("display_name")?,
            current,
        });
    }
    Ok(projections)
}

pub(crate) async fn load_independent_module_navigation_availability(
    tx: &mut Transaction<'_, Postgres>,
) -> Result<Vec<(String, bool)>, sqlx::Error> {
    sqlx::query_as(
        r#"
        SELECT
            definition_id,
            installed AND deployed AND configured AND enabled AS available
        FROM module_instances
        ORDER BY definition_id
        "#,
    )
    .fetch_all(&mut **tx)
    .await
}

pub(crate) async fn load_available_independent_module_navigation_labels(
    tx: &mut Transaction<'_, Postgres>,
) -> Result<Vec<(String, Option<String>)>, sqlx::Error> {
    sqlx::query_as(
        r#"
        SELECT definition_id, configuration ->> 'display_label' AS display_label
        FROM module_instances
        WHERE installed
          AND deployed
          AND configured
          AND enabled
        ORDER BY definition_id
        "#,
    )
    .fetch_all(&mut **tx)
    .await
}

pub(crate) async fn ensure_source(
    tx: &mut Transaction<'_, Postgres>,
    definition_id: &str,
    source_digest: &str,
    source_bytes: &[u8],
) -> Result<DescriptorSourceRow, sqlx::Error> {
    sqlx::query(
        r#"
        INSERT INTO transition_descriptor_sources (
            definition_id,
            schema_version,
            source_digest,
            source_bytes
        )
        VALUES ($1, 1, $2, $3)
        ON CONFLICT (definition_id, source_digest) DO NOTHING
        "#,
    )
    .bind(definition_id)
    .bind(source_digest)
    .bind(source_bytes)
    .execute(&mut **tx)
    .await?;

    let row = sqlx::query(
        r#"
        SELECT id, definition_id, schema_version, source_digest, source_bytes, content_type
        FROM transition_descriptor_sources
        WHERE definition_id = $1 AND source_digest = $2
        "#,
    )
    .bind(definition_id)
    .bind(source_digest)
    .fetch_one(&mut **tx)
    .await?;
    Ok(DescriptorSourceRow {
        id: row.try_get("id")?,
        definition_id: row.try_get("definition_id")?,
        schema_version: row.try_get("schema_version")?,
        source_digest: row.try_get("source_digest")?,
        source_bytes: row.try_get("source_bytes")?,
        content_type: row.try_get("content_type")?,
    })
}

pub(crate) async fn insert_projection(
    tx: &mut Transaction<'_, Postgres>,
    source_id: Uuid,
    installation_id: Uuid,
    normalized_projection: &Value,
) -> Result<Uuid, sqlx::Error> {
    sqlx::query_scalar(
        r#"
        INSERT INTO transition_catalog_projections (
            source_id,
            installation_id,
            normalized_projection
        )
        VALUES ($1, $2, $3)
        RETURNING id
        "#,
    )
    .bind(source_id)
    .bind(installation_id)
    .bind(normalized_projection)
    .fetch_one(&mut **tx)
    .await
}

pub(crate) async fn load_projection_by_source(
    tx: &mut Transaction<'_, Postgres>,
    source_id: Uuid,
) -> Result<Option<StoredProjectionRow>, sqlx::Error> {
    let row = sqlx::query(
        r#"
        SELECT
            id,
            source_id,
            installation_id,
            normalized_projection,
            provider_eligible,
            supervisor_materializable
        FROM transition_catalog_projections
        WHERE source_id = $1
        "#,
    )
    .bind(source_id)
    .fetch_optional(&mut **tx)
    .await?;
    row.map(|row| {
        Ok(StoredProjectionRow {
            id: row.try_get("id")?,
            source_id: row.try_get("source_id")?,
            installation_id: row.try_get("installation_id")?,
            normalized_projection: row.try_get("normalized_projection")?,
            provider_eligible: row.try_get("provider_eligible")?,
            supervisor_materializable: row.try_get("supervisor_materializable")?,
        })
    })
    .transpose()
}

pub(crate) async fn insert_findings(
    tx: &mut Transaction<'_, Postgres>,
    projection_id: Uuid,
    findings: &[CatalogFinding],
) -> Result<(), sqlx::Error> {
    for (ordinal, finding) in findings.iter().enumerate() {
        sqlx::query(
            r#"
            INSERT INTO module_catalog_findings (
                projection_id,
                ordinal,
                code,
                path,
                message
            )
            VALUES ($1, $2, $3, $4, $5)
            "#,
        )
        .bind(projection_id)
        .bind(ordinal as i32)
        .bind(&finding.code)
        .bind(&finding.path)
        .bind(&finding.message)
        .execute(&mut **tx)
        .await?;
    }
    Ok(())
}

pub(crate) async fn set_current_catalog_entry(
    tx: &mut Transaction<'_, Postgres>,
    definition_id: &str,
    source_id: Uuid,
    projection_id: Uuid,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        INSERT INTO transition_catalog_current (definition_id, source_id, projection_id)
        VALUES ($1, $2, $3)
        ON CONFLICT (definition_id) DO UPDATE SET
            source_id = EXCLUDED.source_id,
            projection_id = EXCLUDED.projection_id,
            updated_at = now()
        WHERE transition_catalog_current.source_id IS DISTINCT FROM EXCLUDED.source_id
           OR transition_catalog_current.projection_id IS DISTINCT FROM EXCLUDED.projection_id
        "#,
    )
    .bind(definition_id)
    .bind(source_id)
    .bind(projection_id)
    .execute(&mut **tx)
    .await?;
    Ok(())
}

pub(crate) async fn upsert_transition_capability_provenance(
    tx: &mut Transaction<'_, Postgres>,
    capability_id: Uuid,
    definition_id: &str,
    descriptor_source_id: Uuid,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        INSERT INTO capability_provenance (
            capability_id,
            source_kind,
            source_key,
            definition_id,
            descriptor_source_id,
            provider_state
        )
        VALUES (
            $1,
            'transition_contribution',
            $2,
            $2,
            $3,
            'transitional_in_process'
        )
        ON CONFLICT (capability_id, source_key) DO UPDATE SET
            descriptor_source_id = EXCLUDED.descriptor_source_id,
            updated_at = now()
        WHERE capability_provenance.descriptor_source_id
              IS DISTINCT FROM EXCLUDED.descriptor_source_id
        "#,
    )
    .bind(capability_id)
    .bind(definition_id)
    .bind(descriptor_source_id)
    .execute(&mut **tx)
    .await?;
    Ok(())
}

pub(crate) async fn ensure_navigation_policy(
    tx: &mut Transaction<'_, Postgres>,
    installation_id: Uuid,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        INSERT INTO navigation_policies (installation_id)
        VALUES ($1)
        ON CONFLICT (installation_id) DO NOTHING
        "#,
    )
    .bind(installation_id)
    .execute(&mut **tx)
    .await?;
    Ok(())
}

pub(crate) async fn load_navigation_groups(
    tx: &mut Transaction<'_, Postgres>,
    installation_id: Uuid,
) -> Result<Vec<NavigationGroupRow>, sqlx::Error> {
    sqlx::query(
        r#"
        SELECT group_id, label, display_order, owner
        FROM navigation_groups
        WHERE installation_id = $1
        ORDER BY display_order, group_id
        "#,
    )
    .bind(installation_id)
    .fetch_all(&mut **tx)
    .await?
    .into_iter()
    .map(|row| {
        Ok(NavigationGroupRow {
            group_id: row.try_get("group_id")?,
            label: row.try_get("label")?,
            display_order: row.try_get("display_order")?,
            owner: row.try_get("owner")?,
        })
    })
    .collect()
}

pub(crate) async fn load_navigation_placements(
    tx: &mut Transaction<'_, Postgres>,
    installation_id: Uuid,
) -> Result<Vec<NavigationPlacementRow>, sqlx::Error> {
    sqlx::query(
        r#"
        SELECT destination_id, group_id, visible, display_order
        FROM navigation_destination_placements
        WHERE installation_id = $1
        ORDER BY group_id, display_order, destination_id
        "#,
    )
    .bind(installation_id)
    .fetch_all(&mut **tx)
    .await?
    .into_iter()
    .map(|row| {
        Ok(NavigationPlacementRow {
            destination_id: row.try_get("destination_id")?,
            group_id: row.try_get("group_id")?,
            visible: row.try_get("visible")?,
            display_order: row.try_get("display_order")?,
        })
    })
    .collect()
}

pub(crate) async fn seed_navigation_composition(
    tx: &mut Transaction<'_, Postgres>,
    installation_id: Uuid,
    groups: &[NavigationGroupRow],
    placements: &[NavigationPlacementRow],
) -> Result<(), sqlx::Error> {
    for group in groups {
        sqlx::query(
            r#"
            INSERT INTO navigation_groups (
                installation_id, group_id, label, display_order, owner
            )
            VALUES ($1, $2, $3, $4, $5)
            ON CONFLICT (installation_id, group_id) DO NOTHING
            "#,
        )
        .bind(installation_id)
        .bind(&group.group_id)
        .bind(&group.label)
        .bind(group.display_order)
        .bind(&group.owner)
        .execute(&mut **tx)
        .await?;
    }
    for placement in placements {
        sqlx::query(
            r#"
            INSERT INTO navigation_destination_placements (
                installation_id, destination_id, group_id, visible, display_order
            )
            VALUES ($1, $2, $3, $4, $5)
            ON CONFLICT (installation_id, destination_id) DO NOTHING
            "#,
        )
        .bind(installation_id)
        .bind(&placement.destination_id)
        .bind(&placement.group_id)
        .bind(placement.visible)
        .bind(placement.display_order)
        .execute(&mut **tx)
        .await?;
    }
    Ok(())
}

pub(crate) async fn replace_navigation_composition(
    tx: &mut Transaction<'_, Postgres>,
    installation_id: Uuid,
    groups: &[NavigationGroupRow],
    placements: &[NavigationPlacementRow],
) -> Result<(), sqlx::Error> {
    // Move existing orders out of the requested dense range so swaps never
    // violate the installation/group uniqueness constraints mid-update.
    sqlx::query(
        "UPDATE navigation_groups SET display_order = display_order + 1000000 WHERE installation_id = $1",
    )
    .bind(installation_id)
    .execute(&mut **tx)
    .await?;
    sqlx::query(
        "UPDATE navigation_destination_placements SET display_order = display_order + 1000000 WHERE installation_id = $1",
    )
    .bind(installation_id)
    .execute(&mut **tx)
    .await?;

    for group in groups {
        sqlx::query(
            r#"
            INSERT INTO navigation_groups (
                installation_id, group_id, label, display_order, owner
            )
            VALUES ($1, $2, $3, $4, $5)
            ON CONFLICT (installation_id, group_id) DO UPDATE SET
                label = EXCLUDED.label,
                display_order = EXCLUDED.display_order,
                updated_at = now()
            "#,
        )
        .bind(installation_id)
        .bind(&group.group_id)
        .bind(&group.label)
        .bind(group.display_order)
        .bind(&group.owner)
        .execute(&mut **tx)
        .await?;
    }
    for placement in placements {
        sqlx::query(
            r#"
            UPDATE navigation_destination_placements
            SET group_id = $3,
                visible = $4,
                display_order = $5,
                updated_at = now()
            WHERE installation_id = $1 AND destination_id = $2
            "#,
        )
        .bind(installation_id)
        .bind(&placement.destination_id)
        .bind(&placement.group_id)
        .bind(placement.visible)
        .bind(placement.display_order)
        .execute(&mut **tx)
        .await?;
    }

    let retained_group_ids = groups
        .iter()
        .map(|group| group.group_id.as_str())
        .collect::<Vec<_>>();
    sqlx::query(
        r#"
        DELETE FROM navigation_groups
        WHERE installation_id = $1
          AND owner = 'custom'
          AND NOT (group_id = ANY($2))
        "#,
    )
    .bind(installation_id)
    .bind(&retained_group_ids)
    .execute(&mut **tx)
    .await?;
    Ok(())
}

pub(crate) async fn ensure_navigation_contribution(
    tx: &mut Transaction<'_, Postgres>,
    record: NavigationRecord<'_>,
) -> Result<StoredNavigationContribution, sqlx::Error> {
    sqlx::query(
        r#"
        INSERT INTO module_navigation_contributions (
            contribution_id,
            definition_id,
            descriptor_source_id,
            destination,
            label,
            group_name,
            reorder_band,
            source_order_hint,
            default_policy_order,
            required_capabilities_any_of
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
        ON CONFLICT (contribution_id) DO NOTHING
        "#,
    )
    .bind(record.contribution_id)
    .bind(record.definition_id)
    .bind(record.descriptor_source_id)
    .bind(record.destination)
    .bind(record.label)
    .bind(record.group_name)
    .bind(record.reorder_band)
    .bind(record.source_order_hint)
    .bind(record.default_policy_order)
    .bind(record.required_capabilities_any_of)
    .execute(&mut **tx)
    .await?;

    let row = sqlx::query(
        r#"
        SELECT
            contribution_id,
            definition_id,
            descriptor_source_id,
            destination,
            label,
            group_name,
            reorder_band,
            source_order_hint,
            default_policy_order,
            required_capabilities_any_of
        FROM module_navigation_contributions
        WHERE contribution_id = $1
        "#,
    )
    .bind(record.contribution_id)
    .fetch_one(&mut **tx)
    .await?;
    Ok(StoredNavigationContribution {
        contribution_id: row.try_get("contribution_id")?,
        definition_id: row.try_get("definition_id")?,
        descriptor_source_id: row.try_get("descriptor_source_id")?,
        destination: row.try_get("destination")?,
        label: row.try_get("label")?,
        group_name: row.try_get("group_name")?,
        reorder_band: row.try_get("reorder_band")?,
        source_order_hint: row.try_get("source_order_hint")?,
        default_policy_order: row.try_get("default_policy_order")?,
        required_capabilities_any_of: row.try_get("required_capabilities_any_of")?,
    })
}

pub(crate) async fn update_navigation_descriptor_source(
    tx: &mut Transaction<'_, Postgres>,
    contribution_id: &str,
    descriptor_source_id: Uuid,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        UPDATE module_navigation_contributions
        SET descriptor_source_id = $2, updated_at = now()
        WHERE contribution_id = $1
          AND descriptor_source_id IS DISTINCT FROM $2
        "#,
    )
    .bind(contribution_id)
    .bind(descriptor_source_id)
    .execute(&mut **tx)
    .await?;
    Ok(())
}

pub(crate) async fn ensure_navigation_policy_entry(
    tx: &mut Transaction<'_, Postgres>,
    installation_id: Uuid,
    contribution_id: &str,
    policy_order: i32,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        INSERT INTO navigation_policy_entries (
            installation_id,
            contribution_id,
            visible,
            policy_order
        )
        VALUES ($1, $2, true, $3)
        ON CONFLICT (installation_id, contribution_id) DO NOTHING
        "#,
    )
    .bind(installation_id)
    .bind(contribution_id)
    .bind(policy_order)
    .execute(&mut **tx)
    .await?;
    Ok(())
}

pub(crate) async fn lock_navigation_policy(
    tx: &mut Transaction<'_, Postgres>,
    installation_id: Uuid,
) -> Result<i64, sqlx::Error> {
    sqlx::query_scalar(
        r#"
        SELECT revision
        FROM navigation_policies
        WHERE installation_id = $1
        FOR UPDATE
        "#,
    )
    .bind(installation_id)
    .fetch_one(&mut **tx)
    .await
}

pub(crate) async fn load_navigation_policy_revision(
    tx: &mut Transaction<'_, Postgres>,
    installation_id: Uuid,
) -> Result<i64, sqlx::Error> {
    sqlx::query_scalar("SELECT revision FROM navigation_policies WHERE installation_id = $1")
        .bind(installation_id)
        .fetch_one(&mut **tx)
        .await
}

#[cfg(test)]
pub(crate) async fn load_navigation_policy_entries(
    tx: &mut Transaction<'_, Postgres>,
    installation_id: Uuid,
) -> Result<Vec<NavigationPolicyEntryRow>, sqlx::Error> {
    let rows = sqlx::query(
        r#"
        SELECT
            contributions.contribution_id,
            contributions.definition_id,
            contributions.destination,
            contributions.label,
            contributions.group_name,
            contributions.reorder_band,
            contributions.source_order_hint,
            contributions.default_policy_order,
            contributions.required_capabilities_any_of,
            entries.visible,
            entries.policy_order
        FROM navigation_policy_entries AS entries
        JOIN module_navigation_contributions AS contributions
          ON contributions.contribution_id = entries.contribution_id
        WHERE entries.installation_id = $1
        ORDER BY
            CASE contributions.reorder_band
                WHEN 'main_between_organization_and_operations' THEN 0
                WHEN 'main_after_operations' THEN 1
                WHEN 'admin_between_administration_and_module_management' THEN 2
                ELSE 3
            END,
            entries.policy_order,
            contributions.contribution_id
        "#,
    )
    .bind(installation_id)
    .fetch_all(&mut **tx)
    .await?;
    rows.into_iter()
        .map(|row| {
            Ok(NavigationPolicyEntryRow {
                contribution_id: row.try_get("contribution_id")?,
                definition_id: row.try_get("definition_id")?,
                destination: row.try_get("destination")?,
                label: row.try_get("label")?,
                group_name: row.try_get("group_name")?,
                reorder_band: row.try_get("reorder_band")?,
                source_order_hint: row.try_get("source_order_hint")?,
                default_policy_order: row.try_get("default_policy_order")?,
                required_capabilities_any_of: row.try_get("required_capabilities_any_of")?,
                visible: row.try_get("visible")?,
                policy_order: row.try_get("policy_order")?,
            })
        })
        .collect()
}

#[cfg(test)]
pub(crate) async fn update_navigation_policy_entry(
    tx: &mut Transaction<'_, Postgres>,
    installation_id: Uuid,
    contribution_id: &str,
    visible: bool,
    policy_order: i32,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        UPDATE navigation_policy_entries
        SET visible = $3, policy_order = $4, updated_at = now()
        WHERE installation_id = $1
          AND contribution_id = $2
          AND (visible IS DISTINCT FROM $3 OR policy_order IS DISTINCT FROM $4)
        "#,
    )
    .bind(installation_id)
    .bind(contribution_id)
    .bind(visible)
    .bind(policy_order)
    .execute(&mut **tx)
    .await?;
    Ok(())
}

pub(crate) async fn increment_navigation_policy_revision(
    tx: &mut Transaction<'_, Postgres>,
    installation_id: Uuid,
    current_revision: i64,
) -> Result<i64, sqlx::Error> {
    sqlx::query_scalar(
        r#"
        UPDATE navigation_policies
        SET revision = revision + 1, updated_at = now()
        WHERE installation_id = $1 AND revision = $2
        RETURNING revision
        "#,
    )
    .bind(installation_id)
    .bind(current_revision)
    .fetch_one(&mut **tx)
    .await
}

pub(crate) async fn insert_audit_event(
    tx: &mut Transaction<'_, Postgres>,
    installation_id: Option<Uuid>,
    event_type: &str,
    correlation_id: Uuid,
    payload: &Value,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        INSERT INTO core_control_plane_audit_events (
            installation_id,
            event_type,
            actor_kind,
            correlation_id,
            payload
        )
        VALUES ($1, $2, 'system', $3, $4)
        "#,
    )
    .bind(installation_id)
    .bind(event_type)
    .bind(correlation_id)
    .bind(payload)
    .execute(&mut **tx)
    .await?;
    Ok(())
}

pub(crate) async fn insert_account_audit_event(
    tx: &mut Transaction<'_, Postgres>,
    installation_id: Uuid,
    event_type: &str,
    actor_account_id: Uuid,
    correlation_id: Uuid,
    payload: &Value,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        INSERT INTO core_control_plane_audit_events (
            installation_id,
            event_type,
            actor_kind,
            actor_account_id,
            correlation_id,
            payload
        )
        VALUES ($1, $2, 'account', $3, $4, $5)
        "#,
    )
    .bind(installation_id)
    .bind(event_type)
    .bind(actor_account_id)
    .bind(correlation_id)
    .bind(payload)
    .execute(&mut **tx)
    .await?;
    Ok(())
}

pub(crate) async fn record_navigation_policy_denial(
    pool: &PgPool,
    actor_account_id: Uuid,
    correlation_id: Uuid,
    presented_revision: Option<i64>,
    stable_code: &str,
) -> Result<(), sqlx::Error> {
    let mut tx = pool.begin().await?;
    let installation_id = installation_id(&mut tx).await?;
    let payload = navigation_policy_denial_payload(presented_revision, stable_code);
    insert_account_audit_event(
        &mut tx,
        installation_id,
        "navigation_policy.update_denied",
        actor_account_id,
        correlation_id,
        &payload,
    )
    .await?;
    tx.commit().await
}

fn navigation_policy_denial_payload(presented_revision: Option<i64>, stable_code: &str) -> Value {
    serde_json::json!({
        "schema_version": 1,
        "action": "navigation_policy.update",
        "presented_revision": presented_revision,
        "denial_code": stable_code,
        "success": false,
    })
}

pub(crate) async fn record_rejected_sync(
    pool: &PgPool,
    correlation_id: Uuid,
    stable_code: &str,
    attempted_digests: &[String],
) -> Result<(), sqlx::Error> {
    let mut tx = pool.begin().await?;
    let installation_id = sqlx::query_scalar::<_, Uuid>(
        "SELECT id FROM application_installations WHERE singleton = true",
    )
    .fetch_optional(&mut *tx)
    .await?;
    let payload = serde_json::json!({
        "schema_version": 1,
        "attempted_source_digests": attempted_digests,
        "rejection_code": stable_code,
        "success": false,
    });
    insert_audit_event(
        &mut tx,
        installation_id,
        "module_catalog.sync_rejected",
        correlation_id,
        &payload,
    )
    .await?;
    tx.commit().await
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::navigation_policy_denial_payload;

    #[test]
    fn authorization_denial_payload_has_stable_action_code_and_no_fabricated_revision() {
        assert_eq!(
            navigation_policy_denial_payload(None, "modules_manage_navigation_global_required",),
            json!({
                "schema_version": 1,
                "action": "navigation_policy.update",
                "presented_revision": null,
                "denial_code": "modules_manage_navigation_global_required",
                "success": false,
            })
        );
    }
}
