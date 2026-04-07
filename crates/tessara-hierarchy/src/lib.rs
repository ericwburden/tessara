//! Hierarchy configuration and runtime domain logic for Tessara.
//!
//! This crate owns pure hierarchy rules that are useful outside the HTTP layer.
//! Database-backed orchestration still lives in `tessara-api` until repository
//! and service seams stabilize.

use std::{
    collections::{HashMap, HashSet},
    hash::Hash,
};

/// Validates that a proposed node-type relationship is safe to persist.
///
/// The rule is independent of database IDs: callers provide the proposed
/// parent/child identifiers and the existing directed relationships. A
/// relationship is valid when it is not self-referential and would not make the
/// existing relationship graph cyclic.
pub fn validate_node_type_relationship<T>(
    parent: T,
    child: T,
    existing_relationships: &[(T, T)],
) -> Result<(), NodeTypeRelationshipError>
where
    T: Copy + Eq + Hash,
{
    if parent == child {
        return Err(NodeTypeRelationshipError::SelfReference);
    }

    let mut graph = HashMap::<T, Vec<T>>::new();
    for (existing_parent, existing_child) in existing_relationships {
        graph
            .entry(*existing_parent)
            .or_default()
            .push(*existing_child);
    }

    if can_reach(child, parent, &graph) {
        Err(NodeTypeRelationshipError::Cycle)
    } else {
        Ok(())
    }
}

fn can_reach<T>(start: T, target: T, graph: &HashMap<T, Vec<T>>) -> bool
where
    T: Copy + Eq + Hash,
{
    let mut stack = vec![start];
    let mut visited = HashSet::new();

    while let Some(current) = stack.pop() {
        if current == target {
            return true;
        }
        if !visited.insert(current) {
            continue;
        }
        if let Some(children) = graph.get(&current) {
            stack.extend(children.iter().copied());
        }
    }

    false
}

/// Error returned for invalid node-type relationship definitions.
#[derive(Debug, Clone, Eq, PartialEq, thiserror::Error)]
pub enum NodeTypeRelationshipError {
    /// The relationship uses the same node type as parent and child.
    #[error("node type relationships cannot point to the same type")]
    SelfReference,
    /// Persisting the relationship would introduce a cycle.
    #[error("node type relationship would create a cycle")]
    Cycle,
}

#[cfg(test)]
mod tests {
    use super::{NodeTypeRelationshipError, validate_node_type_relationship};

    #[test]
    fn accepts_acyclic_relationships() {
        let existing = [("partner", "program")];

        assert_eq!(
            validate_node_type_relationship("program", "activity", &existing),
            Ok(())
        );
    }

    #[test]
    fn rejects_self_references() {
        assert_eq!(
            validate_node_type_relationship("program", "program", &[]),
            Err(NodeTypeRelationshipError::SelfReference)
        );
    }

    #[test]
    fn rejects_cycles_through_existing_descendants() {
        let existing = [("partner", "program"), ("program", "activity")];

        assert_eq!(
            validate_node_type_relationship("activity", "partner", &existing),
            Err(NodeTypeRelationshipError::Cycle)
        );
    }
}
