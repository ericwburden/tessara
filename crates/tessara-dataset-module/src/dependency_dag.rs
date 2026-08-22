//! Dataset-owned dependency graph validation and atomic materialization rebuilds.
//!
//! Published revision source snapshots are the canonical dependency graph. An
//! exact-revision source is affected only when that exact revision is rebuilt;
//! a major-line source is affected only when that Dataset major line is
//! rebuilt. Callers keep source import promotion and this rebuild in one outer
//! transaction so a downstream failure cannot expose a partial generation.

use std::collections::{BTreeMap, BTreeSet};

use sqlx::{Postgres, Row, Transaction};
use tessara_datasets_contract::DatasetProductSourceV1;
use uuid::Uuid;

use crate::{DatasetModuleError, materialization};

/// Serializes owner-local graph mutations and rebuild planning. Product
/// transactions may already hold their target Dataset row; this advisory lock
/// deliberately avoids acquiring other Dataset row locks and therefore cannot
/// form a cross-Dataset row-lock cycle.
const DATASET_DEPENDENCY_GRAPH_LOCK: i64 = 0x4453_4441_475f_5631;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DatasetDependencyRebuildTarget {
    pub dataset_id: Uuid,
    pub revision_id: Uuid,
    pub version_major: i32,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct DatasetDependencyRebuildPlan {
    /// Current published revisions in deterministic base-to-dependent order.
    pub targets: Vec<DatasetDependencyRebuildTarget>,
}

/// Acquires the transaction-scoped Dataset dependency-graph writer lock.
///
/// Every create, draft-save, publish, and refresh transaction must invoke this
/// immediately after replay handling and before taking any Dataset row lock.
/// Keeping that lock order prevents a refresh rebuilding a dependent Dataset
/// from deadlocking with a concurrent authoring transaction that already owns
/// the dependent Dataset row.
pub async fn lock_dependency_graph_in_transaction(
    transaction: &mut Transaction<'_, Postgres>,
) -> Result<(), DatasetModuleError> {
    sqlx::query("SELECT pg_advisory_xact_lock($1)")
        .bind(DATASET_DEPENDENCY_GRAPH_LOCK)
        .execute(&mut **transaction)
        .await?;
    Ok(())
}

#[derive(Clone, Debug)]
struct PublishedTarget {
    dataset_id: Uuid,
    revision_id: Uuid,
    version_major: i32,
    generated_sql: String,
}

impl PublishedTarget {
    fn public(&self) -> DatasetDependencyRebuildTarget {
        DatasetDependencyRebuildTarget {
            dataset_id: self.dataset_id,
            revision_id: self.revision_id,
            version_major: self.version_major,
        }
    }
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
enum DependencySelector {
    ExactRevision {
        dataset_id: Uuid,
        revision_id: Uuid,
    },
    MajorLine {
        dataset_id: Uuid,
        version_major: i32,
    },
}

impl DependencySelector {
    fn dataset_id(&self) -> Uuid {
        match self {
            Self::ExactRevision { dataset_id, .. } | Self::MajorLine { dataset_id, .. } => {
                *dataset_id
            }
        }
    }

    fn matches(&self, target: &PublishedTarget) -> bool {
        match self {
            Self::ExactRevision {
                dataset_id,
                revision_id,
            } => *dataset_id == target.dataset_id && *revision_id == target.revision_id,
            Self::MajorLine {
                dataset_id,
                version_major,
            } => *dataset_id == target.dataset_id && *version_major == target.version_major,
        }
    }
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
struct DependencyEdge {
    upstream: DependencySelector,
    dependent_dataset_id: Uuid,
}

#[derive(Debug, Default)]
struct PublishedGraph {
    targets: BTreeMap<Uuid, PublishedTarget>,
    edges: BTreeSet<DependencyEdge>,
}

/// Rejects a candidate Dataset definition when replacing the target Dataset's
/// current dependencies would create any direct or transitive cycle.
///
/// Invoke this after compilation but before source synchronization or any
/// publication/materialization write. The caller's transaction retains the
/// advisory lock until commit, so a competing publication cannot pass a stale
/// cycle check and then create a cycle.
pub async fn validate_candidate_sources_in_transaction(
    transaction: &mut Transaction<'_, Postgres>,
    dataset_id: Uuid,
    candidate_sources: &[DatasetProductSourceV1],
) -> Result<(), DatasetModuleError> {
    if dataset_id.is_nil() {
        return Err(DatasetModuleError::ValidationFailed(
            "Dataset dependency identity is invalid".into(),
        ));
    }
    lock_dependency_graph_in_transaction(transaction).await?;
    let published = load_published_graph(transaction).await?;
    let candidate_edges = candidate_sources
        .iter()
        .filter_map(|source| candidate_edge(dataset_id, source).transpose())
        .collect::<Result<BTreeSet<_>, _>>()?;
    let graph = dataset_adjacency(&published, Some((dataset_id, &candidate_edges)));
    topological_dataset_order(&graph).map_err(|_| {
        DatasetModuleError::ValidationFailed(
            "Dataset dependencies contain a direct or transitive cycle".into(),
        )
    })?;
    Ok(())
}

/// Plans the affected current-published Dataset closure without mutating
/// materializations. Each root denotes a current published revision and its
/// major line that the caller intends to rebuild.
pub async fn plan_affected_published_closure_in_transaction(
    transaction: &mut Transaction<'_, Postgres>,
    root_dataset_ids: &[Uuid],
) -> Result<DatasetDependencyRebuildPlan, DatasetModuleError> {
    lock_dependency_graph_in_transaction(transaction).await?;
    let published = load_published_graph(transaction).await?;
    let targets = affected_targets(&published, root_dataset_ids)?;
    Ok(DatasetDependencyRebuildPlan {
        targets: targets.iter().map(|target| target.public()).collect(),
    })
}

/// Rebuilds every affected current published revision and its current major
/// line in deterministic topological order inside the caller's transaction.
///
/// The function never commits. A returned error must abort the surrounding
/// source-promotion/publication transaction; ordinary `?` propagation drops
/// and rolls it back. PostgreSQL transactional DDL then restores all prior
/// revision and major-line tables together with the prior imported projection,
/// cursor, and receipt state.
pub async fn rebuild_affected_published_closure_in_transaction(
    transaction: &mut Transaction<'_, Postgres>,
    root_dataset_ids: &[Uuid],
) -> Result<DatasetDependencyRebuildPlan, DatasetModuleError> {
    lock_dependency_graph_in_transaction(transaction).await?;
    let published = load_published_graph(transaction).await?;
    let targets = affected_targets(&published, root_dataset_ids)?;
    for target in &targets {
        materialization::materialize_revision(
            transaction,
            target.revision_id,
            &target.generated_sql,
        )
        .await?;
        materialization::rebuild_major_line(transaction, target.dataset_id, target.version_major)
            .await?;
    }
    Ok(DatasetDependencyRebuildPlan {
        targets: targets.iter().map(|target| target.public()).collect(),
    })
}

async fn load_published_graph(
    transaction: &mut Transaction<'_, Postgres>,
) -> Result<PublishedGraph, DatasetModuleError> {
    let target_rows = sqlx::query(
        "SELECT r.dataset_id,r.id AS revision_id,r.version_major,r.generated_sql
         FROM dataset_revisions r
         JOIN datasets d ON d.id=r.dataset_id
         WHERE r.status='published'
           AND r.lifecycle_state <> 'tombstoned'
           AND d.lifecycle_state <> 'tombstoned'
         ORDER BY r.dataset_id",
    )
    .fetch_all(&mut **transaction)
    .await?;
    let mut targets = BTreeMap::new();
    for row in target_rows {
        let dataset_id: Uuid = row.try_get("dataset_id")?;
        let revision_id: Uuid = row.try_get("revision_id")?;
        let version_major = row
            .try_get::<Option<i32>, _>("version_major")?
            .filter(|major| *major > 0)
            .ok_or_else(|| {
                DatasetModuleError::Internal(
                    "A published Dataset has no valid major version".into(),
                )
            })?;
        let generated_sql = row
            .try_get::<Option<String>, _>("generated_sql")?
            .filter(|sql| !sql.trim().is_empty())
            .ok_or_else(|| {
                DatasetModuleError::Internal(
                    "A published Dataset has no materialization SQL".into(),
                )
            })?;
        let target = PublishedTarget {
            dataset_id,
            revision_id,
            version_major,
            generated_sql,
        };
        if targets.insert(dataset_id, target).is_some() {
            return Err(DatasetModuleError::Internal(
                "A Dataset has more than one current published revision".into(),
            ));
        }
    }

    let edge_rows = sqlx::query(
        "SELECT r.dataset_id AS dependent_dataset_id,
                s.source_kind,s.source_reference
         FROM dataset_revisions r
         JOIN datasets d ON d.id=r.dataset_id
         JOIN dataset_revision_sources s ON s.revision_id=r.id
         WHERE r.status='published'
           AND r.lifecycle_state <> 'tombstoned'
           AND d.lifecycle_state <> 'tombstoned'
         ORDER BY r.dataset_id,s.position,s.source_alias",
    )
    .fetch_all(&mut **transaction)
    .await?;
    let mut edges = BTreeSet::new();
    for row in edge_rows {
        let dependent_dataset_id: Uuid = row.try_get("dependent_dataset_id")?;
        let source_kind: String = row.try_get("source_kind")?;
        let source_reference: serde_json::Value = row.try_get("source_reference")?;
        if let Some(upstream) = stored_selector(&source_kind, source_reference)? {
            edges.insert(DependencyEdge {
                upstream,
                dependent_dataset_id,
            });
        }
    }
    Ok(PublishedGraph { targets, edges })
}

fn stored_selector(
    source_kind: &str,
    source_reference: serde_json::Value,
) -> Result<Option<DependencySelector>, DatasetModuleError> {
    let source =
        serde_json::from_value::<DatasetProductSourceV1>(source_reference).map_err(|error| {
            DatasetModuleError::Internal(format!(
                "Stored Dataset source snapshot is invalid: {error}"
            ))
        })?;
    match (source_kind, source) {
        ("form_version", DatasetProductSourceV1::Form { .. }) => Ok(None),
        (
            "dataset_revision",
            DatasetProductSourceV1::Dataset {
                dataset_id,
                dataset_revision_id,
                ..
            },
        ) => Ok(Some(DependencySelector::ExactRevision {
            dataset_id: parse_stored_uuid("Dataset", &dataset_id)?,
            revision_id: parse_stored_uuid("Dataset revision", &dataset_revision_id)?,
        })),
        (
            "dataset_major_line",
            DatasetProductSourceV1::DatasetMajor {
                dataset_id,
                version_major,
                ..
            },
        ) if version_major > 0 => Ok(Some(DependencySelector::MajorLine {
            dataset_id: parse_stored_uuid("Dataset", &dataset_id)?,
            version_major,
        })),
        ("dataset_major_line", DatasetProductSourceV1::DatasetMajor { .. }) => {
            Err(DatasetModuleError::Internal(
                "Stored Dataset major-line source has an invalid major version".into(),
            ))
        }
        _ => Err(DatasetModuleError::Internal(
            "Stored Dataset source kind does not match its canonical reference".into(),
        )),
    }
}

fn candidate_edge(
    dependent_dataset_id: Uuid,
    source: &DatasetProductSourceV1,
) -> Result<Option<DependencyEdge>, DatasetModuleError> {
    let upstream = match source {
        DatasetProductSourceV1::Form { .. } => return Ok(None),
        DatasetProductSourceV1::Dataset {
            dataset_id,
            dataset_revision_id,
            ..
        } => DependencySelector::ExactRevision {
            dataset_id: parse_candidate_uuid("source Dataset", dataset_id)?,
            revision_id: parse_candidate_uuid("source Dataset revision", dataset_revision_id)?,
        },
        DatasetProductSourceV1::DatasetMajor {
            dataset_id,
            version_major,
            ..
        } => {
            if *version_major <= 0 {
                return Err(DatasetModuleError::ValidationFailed(
                    "Dataset source major version must be greater than zero".into(),
                ));
            }
            DependencySelector::MajorLine {
                dataset_id: parse_candidate_uuid("source Dataset", dataset_id)?,
                version_major: *version_major,
            }
        }
    };
    Ok(Some(DependencyEdge {
        upstream,
        dependent_dataset_id,
    }))
}

fn parse_stored_uuid(label: &str, value: &str) -> Result<Uuid, DatasetModuleError> {
    Uuid::parse_str(value)
        .ok()
        .filter(|id| !id.is_nil())
        .ok_or_else(|| {
            DatasetModuleError::Internal(format!(
                "Stored {label} source identity is not a canonical UUID"
            ))
        })
}

fn parse_candidate_uuid(label: &str, value: &str) -> Result<Uuid, DatasetModuleError> {
    Uuid::parse_str(value)
        .ok()
        .filter(|id| !id.is_nil())
        .ok_or_else(|| {
            DatasetModuleError::ValidationFailed(format!(
                "{label} identity must be a canonical UUID"
            ))
        })
}

fn dataset_adjacency(
    published: &PublishedGraph,
    candidate: Option<(Uuid, &BTreeSet<DependencyEdge>)>,
) -> BTreeMap<Uuid, BTreeSet<Uuid>> {
    let mut graph = BTreeMap::<Uuid, BTreeSet<Uuid>>::new();
    for dataset_id in published.targets.keys() {
        graph.entry(*dataset_id).or_default();
    }
    let replaced_dataset_id = candidate.map(|(dataset_id, _)| dataset_id);
    for edge in &published.edges {
        if Some(edge.dependent_dataset_id) == replaced_dataset_id {
            continue;
        }
        graph.entry(edge.upstream.dataset_id()).or_default();
        graph
            .entry(edge.upstream.dataset_id())
            .or_default()
            .insert(edge.dependent_dataset_id);
        graph.entry(edge.dependent_dataset_id).or_default();
    }
    if let Some((dataset_id, candidate_edges)) = candidate {
        graph.entry(dataset_id).or_default();
        for edge in candidate_edges {
            graph.entry(edge.upstream.dataset_id()).or_default();
            graph
                .entry(edge.upstream.dataset_id())
                .or_default()
                .insert(dataset_id);
        }
    }
    graph
}

fn topological_dataset_order(graph: &BTreeMap<Uuid, BTreeSet<Uuid>>) -> Result<Vec<Uuid>, ()> {
    let mut indegree = graph
        .keys()
        .copied()
        .map(|dataset_id| (dataset_id, 0_usize))
        .collect::<BTreeMap<_, _>>();
    for dependents in graph.values() {
        for dependent in dependents {
            *indegree.entry(*dependent).or_default() += 1;
        }
    }
    let mut ready = indegree
        .iter()
        .filter_map(|(dataset_id, degree)| (*degree == 0).then_some(*dataset_id))
        .collect::<BTreeSet<_>>();
    let mut ordered = Vec::with_capacity(indegree.len());
    while let Some(dataset_id) = ready.pop_first() {
        ordered.push(dataset_id);
        if let Some(dependents) = graph.get(&dataset_id) {
            for dependent in dependents {
                let degree = indegree
                    .get_mut(dependent)
                    .expect("all graph dependents have an indegree entry");
                *degree -= 1;
                if *degree == 0 {
                    ready.insert(*dependent);
                }
            }
        }
    }
    if ordered.len() == indegree.len() {
        Ok(ordered)
    } else {
        Err(())
    }
}

fn affected_targets<'a>(
    published: &'a PublishedGraph,
    root_dataset_ids: &[Uuid],
) -> Result<Vec<&'a PublishedTarget>, DatasetModuleError> {
    let roots = root_dataset_ids.iter().copied().collect::<BTreeSet<_>>();
    for root in &roots {
        if !published.targets.contains_key(root) {
            return Err(DatasetModuleError::Conflict(
                "A requested Dataset has no current published revision".into(),
            ));
        }
    }
    let mut affected = roots;
    loop {
        let mut changed = false;
        for edge in &published.edges {
            let upstream_dataset_id = edge.upstream.dataset_id();
            let Some(upstream_target) = published.targets.get(&upstream_dataset_id) else {
                continue;
            };
            if affected.contains(&upstream_dataset_id)
                && edge.upstream.matches(upstream_target)
                && published.targets.contains_key(&edge.dependent_dataset_id)
                && affected.insert(edge.dependent_dataset_id)
            {
                changed = true;
            }
        }
        if !changed {
            break;
        }
    }

    let mut induced = affected
        .iter()
        .copied()
        .map(|dataset_id| (dataset_id, BTreeSet::new()))
        .collect::<BTreeMap<_, _>>();
    for edge in &published.edges {
        let upstream_dataset_id = edge.upstream.dataset_id();
        let Some(upstream_target) = published.targets.get(&upstream_dataset_id) else {
            continue;
        };
        if affected.contains(&upstream_dataset_id)
            && affected.contains(&edge.dependent_dataset_id)
            && edge.upstream.matches(upstream_target)
        {
            induced
                .get_mut(&upstream_dataset_id)
                .expect("affected upstream is present")
                .insert(edge.dependent_dataset_id);
        }
    }
    let ordered = topological_dataset_order(&induced).map_err(|_| {
        DatasetModuleError::Internal(
            "Published Dataset dependencies contain a materialization cycle".into(),
        )
    })?;
    Ok(ordered
        .into_iter()
        .map(|dataset_id| {
            published
                .targets
                .get(&dataset_id)
                .expect("affected Dataset has a published target")
        })
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn id(value: u128) -> Uuid {
        Uuid::from_u128(value)
    }

    fn target(dataset_id: Uuid, revision_id: Uuid, major: i32) -> PublishedTarget {
        PublishedTarget {
            dataset_id,
            revision_id,
            version_major: major,
            generated_sql: "SELECT 1".into(),
        }
    }

    #[test]
    fn candidate_cycle_detection_is_transitive() {
        let first = id(1);
        let second = id(2);
        let third = id(3);
        let mut published = PublishedGraph::default();
        published.targets.insert(first, target(first, id(11), 1));
        published.targets.insert(second, target(second, id(12), 1));
        published.targets.insert(third, target(third, id(13), 1));
        published.edges.insert(DependencyEdge {
            upstream: DependencySelector::MajorLine {
                dataset_id: first,
                version_major: 1,
            },
            dependent_dataset_id: second,
        });
        published.edges.insert(DependencyEdge {
            upstream: DependencySelector::ExactRevision {
                dataset_id: second,
                revision_id: id(12),
            },
            dependent_dataset_id: third,
        });
        let candidate_edges = BTreeSet::from([DependencyEdge {
            upstream: DependencySelector::MajorLine {
                dataset_id: third,
                version_major: 1,
            },
            dependent_dataset_id: first,
        }]);

        let graph = dataset_adjacency(&published, Some((first, &candidate_edges)));
        assert!(topological_dataset_order(&graph).is_err());
    }

    #[test]
    fn affected_closure_distinguishes_exact_revision_and_major_line_edges() {
        let base = id(1);
        let derived = id(2);
        let second_hop = id(3);
        let historical_exact = id(4);
        let other_major = id(5);
        let independent = id(6);
        let mut published = PublishedGraph::default();
        for (dataset_id, revision_id, major) in [
            (base, id(11), 2),
            (derived, id(12), 1),
            (second_hop, id(13), 1),
            (historical_exact, id(14), 1),
            (other_major, id(15), 1),
            (independent, id(16), 1),
        ] {
            published
                .targets
                .insert(dataset_id, target(dataset_id, revision_id, major));
        }
        published.edges.extend([
            DependencyEdge {
                upstream: DependencySelector::ExactRevision {
                    dataset_id: base,
                    revision_id: id(11),
                },
                dependent_dataset_id: derived,
            },
            DependencyEdge {
                upstream: DependencySelector::MajorLine {
                    dataset_id: derived,
                    version_major: 1,
                },
                dependent_dataset_id: second_hop,
            },
            DependencyEdge {
                upstream: DependencySelector::ExactRevision {
                    dataset_id: base,
                    revision_id: id(99),
                },
                dependent_dataset_id: historical_exact,
            },
            DependencyEdge {
                upstream: DependencySelector::MajorLine {
                    dataset_id: base,
                    version_major: 1,
                },
                dependent_dataset_id: other_major,
            },
        ]);

        let plan = affected_targets(&published, &[base]).unwrap();
        assert_eq!(
            plan.iter()
                .map(|target| target.dataset_id)
                .collect::<Vec<_>>(),
            vec![base, derived, second_hop]
        );
        assert!(!plan.iter().any(|target| target.dataset_id == independent));
    }

    #[test]
    fn topological_order_is_stable_for_independent_siblings() {
        let root = id(1);
        let lower_id = id(2);
        let higher_id = id(3);
        let graph = BTreeMap::from([
            (root, BTreeSet::from([higher_id, lower_id])),
            (lower_id, BTreeSet::new()),
            (higher_id, BTreeSet::new()),
        ]);

        assert_eq!(
            topological_dataset_order(&graph).unwrap(),
            vec![root, lower_id, higher_id]
        );
    }

    #[test]
    fn stored_snapshot_kind_must_match_its_exact_reference_variant() {
        let source = serde_json::to_value(DatasetProductSourceV1::DatasetMajor {
            alias: "upstream".into(),
            dataset_id: id(1).to_string(),
            version_major: 1,
        })
        .unwrap();

        let error = stored_selector("dataset_revision", source).unwrap_err();
        assert!(matches!(error, DatasetModuleError::Internal(_)));
    }
}
