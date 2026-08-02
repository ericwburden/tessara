//! Shared authorization decisions for Dataset-backed analytics execution.

use uuid::Uuid;

use crate::{
    auth::{self, CapabilityBoundary},
    datasets::load_dataset_scope_node_ids,
    error::ApiResult,
};

pub(crate) async fn tier_access_predicate_for_dataset(
    pool: &sqlx::PgPool,
    account: &auth::AccountContext,
    dataset_id: Uuid,
    base_read_capability: &str,
) -> ApiResult<&'static str> {
    let governing_nodes = load_dataset_scope_node_ids(pool, dataset_id).await?;
    let base = auth::capability_boundary(pool, account, base_read_capability).await?;
    let confidential =
        auth::capability_boundary(pool, account, "datasets:read_confidential").await?;
    if boundaries_intersect_on_governing_node(&base, &confidential, &governing_nodes) {
        return Ok("TRUE");
    }

    let restricted = auth::capability_boundary(pool, account, "datasets:read_restricted").await?;
    if boundaries_intersect_on_governing_node(&base, &restricted, &governing_nodes) {
        Ok("COALESCE(\"__restriction_tier\", 'public') IN ('public', 'internal', 'restricted')")
    } else {
        Ok("COALESCE(\"__restriction_tier\", 'public') IN ('public', 'internal')")
    }
}

fn boundaries_intersect_on_governing_node(
    left: &CapabilityBoundary,
    right: &CapabilityBoundary,
    governing_nodes: &[Uuid],
) -> bool {
    governing_nodes
        .iter()
        .any(|node_id| boundary_contains(left, *node_id) && boundary_contains(right, *node_id))
}

fn boundary_contains(boundary: &CapabilityBoundary, node_id: Uuid) -> bool {
    match boundary {
        CapabilityBoundary::Global => true,
        CapabilityBoundary::Scoped(node_ids) => node_ids.contains(&node_id),
        CapabilityBoundary::None => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn disjoint_scopes_do_not_cross_product() {
        let read_node = Uuid::new_v4();
        let tier_node = Uuid::new_v4();
        assert!(!boundaries_intersect_on_governing_node(
            &CapabilityBoundary::Scoped(vec![read_node]),
            &CapabilityBoundary::Scoped(vec![tier_node]),
            &[read_node, tier_node],
        ));
    }

    #[test]
    fn same_governing_node_intersects() {
        let shared = Uuid::new_v4();
        assert!(boundaries_intersect_on_governing_node(
            &CapabilityBoundary::Scoped(vec![shared]),
            &CapabilityBoundary::Scoped(vec![shared]),
            &[shared],
        ));
    }

    #[test]
    fn global_still_requires_a_governing_node() {
        assert!(!boundaries_intersect_on_governing_node(
            &CapabilityBoundary::Global,
            &CapabilityBoundary::Global,
            &[],
        ));
    }
}
