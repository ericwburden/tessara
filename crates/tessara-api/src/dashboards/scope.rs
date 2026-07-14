//! Shared Dashboard capability-boundary predicates.

use uuid::Uuid;

use crate::{auth::CapabilityBoundary, error::ApiError};

use super::error::DashboardResult;

pub(super) fn overlaps(boundary: &CapabilityBoundary, node_ids: &[Uuid]) -> bool {
    match boundary {
        CapabilityBoundary::Global => true,
        CapabilityBoundary::Scoped(scope_ids) => {
            node_ids.iter().any(|node_id| scope_ids.contains(node_id))
        }
        CapabilityBoundary::None => false,
    }
}

pub(super) fn contains(boundary: &CapabilityBoundary, node_ids: &[Uuid]) -> bool {
    match boundary {
        CapabilityBoundary::Global => true,
        CapabilityBoundary::Scoped(scope_ids) => {
            !node_ids.is_empty() && node_ids.iter().all(|node_id| scope_ids.contains(node_id))
        }
        CapabilityBoundary::None => false,
    }
}

pub(super) fn require_contains(
    boundary: &CapabilityBoundary,
    node_ids: &[Uuid],
    capability: &str,
) -> DashboardResult<()> {
    if contains(boundary, node_ids) {
        Ok(())
    } else {
        Err(ApiError::Forbidden(capability.to_string()).into())
    }
}
