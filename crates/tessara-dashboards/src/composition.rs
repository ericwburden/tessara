use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use tessara_core::grid_layout::{
    GridConstraints, GridLayoutError, GridPlacement, GridRect, GridSize, reflow_movement,
    validate_resize,
};

/// Editor operation explicitly permitted for one Dashboard placement.
///
/// This shared vocabulary is serialized at the API boundary and consumed by
/// the web editor, keeping authorization-relevant actions out of stringly
/// typed transport and UI logic.
#[derive(Debug, Clone, Copy, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DashboardPlacementOperation {
    Retain,
    Move,
    Resize,
    Retitle,
    Replace,
    Preview,
    Repair,
    Remove,
}

/// The Sprint 5A Dashboard grid policy: 12 columns, 240 rows, and at most 240
/// placements. A placement may use every remaining row from its starting row;
/// the grid boundary, rather than a smaller independent height cap, bounds it.
pub const DASHBOARD_GRID_CONSTRAINTS: GridConstraints = GridConstraints::new(12, 240, 240, 240);

/// Dashboard's global hard minimum. Component-kind rules may raise it.
pub const DASHBOARD_HARD_MINIMUM: GridSize = GridSize::ONE;

/// Minimum and recommended sizes for one component kind.
///
/// The recommendation may be larger than the enforced minimum while remaining
/// nonbinding.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct DashboardPlacementSizeRule {
    pub minimum: GridSize,
    pub recommended: GridSize,
}

impl Default for DashboardPlacementSizeRule {
    fn default() -> Self {
        Self {
            minimum: DASHBOARD_HARD_MINIMUM,
            recommended: DASHBOARD_HARD_MINIMUM,
        }
    }
}

/// Code-defined placement sizing rules keyed by canonical component kind.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct DashboardPlacementSizePolicy {
    rules: BTreeMap<String, DashboardPlacementSizeRule>,
}

impl Default for DashboardPlacementSizePolicy {
    fn default() -> Self {
        let minimum = DASHBOARD_HARD_MINIMUM;
        let rules = [
            (
                "table",
                DashboardPlacementSizeRule {
                    minimum: GridSize::new(6, 4),
                    recommended: GridSize::new(6, 4),
                },
            ),
            (
                "bar",
                DashboardPlacementSizeRule {
                    minimum,
                    recommended: GridSize::new(6, 3),
                },
            ),
            (
                "line",
                DashboardPlacementSizeRule {
                    minimum,
                    recommended: GridSize::new(6, 2),
                },
            ),
            (
                "pie",
                DashboardPlacementSizeRule {
                    minimum,
                    recommended: GridSize::new(3, 3),
                },
            ),
            (
                "donut",
                DashboardPlacementSizeRule {
                    minimum,
                    recommended: GridSize::new(3, 3),
                },
            ),
            (
                "stat_card",
                DashboardPlacementSizeRule {
                    minimum,
                    recommended: GridSize::new(3, 2),
                },
            ),
        ]
        .into_iter()
        .map(|(component_kind, rule)| (component_kind.to_string(), rule))
        .collect();
        Self { rules }
    }
}

impl DashboardPlacementSizePolicy {
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds a kind-specific minimum while preserving any larger recommendation.
    pub fn with_kind_minimum(
        mut self,
        component_kind: impl Into<String>,
        minimum: GridSize,
    ) -> Result<Self, CompositionError> {
        let component_kind = component_kind.into();
        let current = self.rule_for(&component_kind);
        self.set_kind_rule(
            component_kind,
            DashboardPlacementSizeRule {
                minimum,
                recommended: GridSize::new(
                    current.recommended.width.max(minimum.width),
                    current.recommended.height.max(minimum.height),
                ),
            },
        )?;
        Ok(self)
    }

    /// Adds a complete kind-specific rule after validating it against the
    /// dashboard's global grid bounds.
    pub fn with_kind_rule(
        mut self,
        component_kind: impl Into<String>,
        rule: DashboardPlacementSizeRule,
    ) -> Result<Self, CompositionError> {
        self.set_kind_rule(component_kind, rule)?;
        Ok(self)
    }

    pub fn set_kind_rule(
        &mut self,
        component_kind: impl Into<String>,
        rule: DashboardPlacementSizeRule,
    ) -> Result<(), CompositionError> {
        let component_kind = component_kind.into();
        let canonical_kind = component_kind.trim().to_ascii_lowercase();
        if canonical_kind.is_empty()
            || DASHBOARD_GRID_CONSTRAINTS
                .validate_rect_with_minimum(
                    tessara_core::grid_layout::GridRect::new(
                        1,
                        1,
                        rule.minimum.width,
                        rule.minimum.height,
                    ),
                    DASHBOARD_HARD_MINIMUM,
                )
                .is_err()
            || DASHBOARD_GRID_CONSTRAINTS
                .validate_rect_with_minimum(
                    tessara_core::grid_layout::GridRect::new(
                        1,
                        1,
                        rule.recommended.width,
                        rule.recommended.height,
                    ),
                    rule.minimum,
                )
                .is_err()
        {
            return Err(CompositionError::InvalidSizePolicy {
                component_kind,
                minimum: rule.minimum,
                recommended: rule.recommended,
            });
        }
        self.rules.insert(canonical_kind, rule);
        Ok(())
    }

    pub fn rule_for(&self, component_kind: &str) -> DashboardPlacementSizeRule {
        self.rules
            .get(&component_kind.trim().to_ascii_lowercase())
            .copied()
            .unwrap_or_default()
    }

    pub fn minimum_for(&self, component_kind: &str) -> GridSize {
        self.rule_for(component_kind).minimum
    }

    pub fn recommended_for(&self, component_kind: &str) -> GridSize {
        self.rule_for(component_kind).recommended
    }
}

/// Validates a complete dashboard layout with the global 1x1 hard minimum.
pub fn validate_dashboard_layout<Id: PartialEq>(
    placements: &[GridPlacement<Id>],
) -> Result<(), CompositionError> {
    DASHBOARD_GRID_CONSTRAINTS
        .validate_layout(placements)
        .map_err(CompositionError::from)
}

/// Validates a dashboard layout using caller-selected component-kind minimums.
pub fn validate_dashboard_layout_with<Id: PartialEq, MinimumFor>(
    placements: &[GridPlacement<Id>],
    minimum_for: MinimumFor,
) -> Result<(), CompositionError>
where
    MinimumFor: FnMut(&GridPlacement<Id>) -> GridSize,
{
    DASHBOARD_GRID_CONSTRAINTS
        .validate_layout_with(placements, minimum_for)
        .map_err(CompositionError::from)
}

/// Applies Dashboard's deterministic occupied-target movement policy.
pub fn reflow_dashboard_movement<Id: Clone + Eq + Ord>(
    placements: &[GridPlacement<Id>],
    moved_id: &Id,
    target_row: i32,
    target_column: i32,
) -> Result<Vec<GridPlacement<Id>>, CompositionError> {
    reflow_movement(
        DASHBOARD_GRID_CONSTRAINTS,
        placements,
        moved_id,
        target_row,
        target_column,
    )
    .map_err(CompositionError::from)
}

/// Validates a Dashboard resize without reflowing occupied placements.
pub fn validate_dashboard_resize<Id: Clone + PartialEq>(
    placements: &[GridPlacement<Id>],
    placement_id: &Id,
    requested: GridSize,
    component_kind: &str,
    policy: &DashboardPlacementSizePolicy,
) -> Result<GridPlacement<Id>, CompositionError> {
    validate_resize(
        DASHBOARD_GRID_CONSTRAINTS,
        placements,
        placement_id,
        requested,
        policy.minimum_for(component_kind),
    )
    .map_err(CompositionError::from)
}

/// Pure dashboard composition errors suitable for explicit service/API mapping.
#[derive(Debug, Clone, Eq, PartialEq, thiserror::Error)]
pub enum CompositionError {
    #[error("invalid dashboard geometry: {0}")]
    InvalidGeometry(GridLayoutError),
    #[error("dashboard placements overlap")]
    Overlap,
    #[error("dashboard placement count {count} exceeds the limit of {max}")]
    PlacementLimitExceeded { count: usize, max: usize },
    #[error("dashboard placement identifiers must be unique")]
    DuplicatePlacementId,
    #[error("dashboard placement was not found")]
    PlacementNotFound,
    #[error("no dashboard grid space remains")]
    NoSpace,
    #[error(
        "invalid size policy for component kind '{component_kind}': minimum {minimum:?}, recommended {recommended:?}"
    )]
    InvalidSizePolicy {
        component_kind: String,
        minimum: GridSize,
        recommended: GridSize,
    },
    #[error("invalid dashboard fallback geometry: {rect:?}")]
    InvalidFallbackGeometry { rect: GridRect },
    #[error("dashboard placement config needs repair")]
    ConfigNeedsRepair,
    #[error("dashboard placement schema {schema_version} is not supported")]
    UnsupportedConfigSchema { schema_version: String },
    #[error("dashboard placement config could not be encoded: {message}")]
    ConfigEncoding { message: String },
}

impl From<GridLayoutError> for CompositionError {
    fn from(error: GridLayoutError) -> Self {
        match error {
            GridLayoutError::Overlap => Self::Overlap,
            GridLayoutError::PlacementLimitExceeded { count, max } => {
                Self::PlacementLimitExceeded { count, max }
            }
            GridLayoutError::DuplicatePlacementId => Self::DuplicatePlacementId,
            GridLayoutError::PlacementNotFound => Self::PlacementNotFound,
            GridLayoutError::NoSpace => Self::NoSpace,
            other => Self::InvalidGeometry(other),
        }
    }
}

#[cfg(test)]
mod tests {
    use tessara_core::grid_layout::{GridLayoutError, GridPlacement, GridRect, GridSize};

    use super::{
        CompositionError, DASHBOARD_GRID_CONSTRAINTS, DashboardPlacementOperation,
        DashboardPlacementSizePolicy, DashboardPlacementSizeRule, reflow_dashboard_movement,
        validate_dashboard_layout, validate_dashboard_resize,
    };

    fn placement(
        id: &'static str,
        row: i32,
        column: i32,
        width: i32,
        height: i32,
    ) -> GridPlacement<&'static str> {
        GridPlacement::new(id, GridRect::new(row, column, width, height))
    }

    #[test]
    fn placement_operations_have_a_stable_snake_case_transport_vocabulary() {
        assert_eq!(
            serde_json::to_value(DashboardPlacementOperation::Retitle).expect("serialize"),
            serde_json::json!("retitle")
        );
        assert_eq!(
            serde_json::from_value::<DashboardPlacementOperation>(serde_json::json!("repair"))
                .expect("deserialize"),
            DashboardPlacementOperation::Repair
        );
    }

    #[test]
    fn publishes_sprint_5a_dashboard_bounds() {
        assert_eq!(DASHBOARD_GRID_CONSTRAINTS.columns(), 12);
        assert_eq!(DASHBOARD_GRID_CONSTRAINTS.max_rows(), 240);
        assert_eq!(DASHBOARD_GRID_CONSTRAINTS.max_placements(), 240);
        assert_eq!(DASHBOARD_GRID_CONSTRAINTS.max_height(), 240);
        assert!(
            DASHBOARD_GRID_CONSTRAINTS
                .validate_rect(GridRect::new(1, 1, 12, 240))
                .is_ok()
        );
        assert!(
            DASHBOARD_GRID_CONSTRAINTS
                .validate_rect(GridRect::new(2, 1, 12, 239))
                .is_ok()
        );
        assert!(matches!(
            DASHBOARD_GRID_CONSTRAINTS.validate_rect(GridRect::new(1, 1, 12, 241)),
            Err(GridLayoutError::HeightOutOfRange { max: 240, .. })
        ));
        assert!(matches!(
            DASHBOARD_GRID_CONSTRAINTS.validate_rect(GridRect::new(2, 1, 12, 240)),
            Err(GridLayoutError::RowOverflow { .. })
        ));
    }

    #[test]
    fn table_enforces_six_by_four_minimum_while_other_kinds_keep_global_minimums() {
        let policy = DashboardPlacementSizePolicy::new();
        assert_eq!(policy.minimum_for("table"), GridSize::new(6, 4));
        assert_eq!(policy.recommended_for("table"), GridSize::new(6, 4));
        assert_eq!(policy.minimum_for("bar"), GridSize::ONE);
        assert_eq!(policy.minimum_for("line"), GridSize::ONE);
        assert_eq!(policy.minimum_for("pie"), GridSize::ONE);
        assert_eq!(policy.minimum_for("donut"), GridSize::ONE);
        assert_eq!(policy.minimum_for("stat_card"), GridSize::ONE);
        assert_eq!(policy.recommended_for("bar"), GridSize::new(6, 3));
        assert_eq!(policy.recommended_for("line"), GridSize::new(6, 2));
        assert_eq!(policy.recommended_for("pie"), GridSize::new(3, 3));
        assert_eq!(policy.recommended_for("donut"), GridSize::new(3, 3));
        assert_eq!(policy.recommended_for("stat_card"), GridSize::new(3, 2));
        assert_eq!(policy.recommended_for("unknown_future_kind"), GridSize::ONE);
    }

    #[test]
    fn supports_code_defined_kind_rules() {
        let policy = DashboardPlacementSizePolicy::new()
            .with_kind_rule(
                "table",
                DashboardPlacementSizeRule {
                    minimum: GridSize::new(6, 2),
                    recommended: GridSize::new(12, 4),
                },
            )
            .expect("valid rule");
        assert_eq!(policy.minimum_for("table"), GridSize::new(6, 2));
        assert_eq!(policy.recommended_for("table"), GridSize::new(12, 4));
        assert_eq!(policy.minimum_for("bar"), GridSize::ONE);
    }

    #[test]
    fn kind_rules_use_canonical_trimmed_lowercase_keys() {
        let policy = DashboardPlacementSizePolicy::new()
            .with_kind_minimum("  Future_Kind  ", GridSize::new(2, 2))
            .expect("valid custom kind minimum");

        assert_eq!(policy.minimum_for("future_kind"), GridSize::new(2, 2));
        assert_eq!(policy.minimum_for(" FUTURE_KIND "), GridSize::new(2, 2));
    }

    #[test]
    fn raising_a_minimum_preserves_a_larger_shipped_recommendation() {
        let policy = DashboardPlacementSizePolicy::new()
            .with_kind_minimum("bar", GridSize::new(4, 2))
            .expect("valid minimum");
        assert_eq!(policy.minimum_for("bar"), GridSize::new(4, 2));
        assert_eq!(policy.recommended_for("bar"), GridSize::new(6, 3));
    }

    #[test]
    fn rejects_invalid_kind_rules() {
        let result =
            DashboardPlacementSizePolicy::new().with_kind_minimum("table", GridSize::new(13, 1));
        assert!(matches!(
            result,
            Err(CompositionError::InvalidSizePolicy { .. })
        ));
    }

    #[test]
    fn layout_errors_are_stable_domain_errors() {
        let overlapping = [placement("a", 1, 1, 7, 1), placement("b", 1, 7, 6, 1)];
        assert_eq!(
            validate_dashboard_layout(&overlapping),
            Err(CompositionError::Overlap)
        );

        let invalid = [placement("a", 241, 1, 1, 1)];
        assert!(matches!(
            validate_dashboard_layout(&invalid),
            Err(CompositionError::InvalidGeometry(
                GridLayoutError::RowOverflow { .. }
            ))
        ));
    }

    #[test]
    fn dashboard_move_reflows_but_resize_rejects_collision() {
        let placements = [placement("a", 1, 1, 6, 4), placement("b", 1, 7, 6, 4)];
        assert_eq!(
            reflow_dashboard_movement(&placements, &"b", 1, 1).expect("reflow"),
            vec![placement("b", 1, 1, 6, 4), placement("a", 1, 7, 6, 4)]
        );

        assert_eq!(
            validate_dashboard_resize(
                &placements,
                &"a",
                GridSize::new(7, 4),
                "table",
                &DashboardPlacementSizePolicy::new()
            ),
            Err(CompositionError::Overlap)
        );
    }

    #[test]
    fn dashboard_resize_uses_component_kind_minimum() {
        let placements = [placement("table", 1, 1, 6, 3)];
        let policy = DashboardPlacementSizePolicy::new()
            .with_kind_minimum("table", GridSize::new(6, 3))
            .expect("valid minimum");
        assert!(matches!(
            validate_dashboard_resize(&placements, &"table", GridSize::new(5, 3), "table", &policy),
            Err(CompositionError::InvalidGeometry(
                GridLayoutError::WidthOutOfRange { min: 6, .. }
            ))
        ));
    }

    #[test]
    fn dashboard_resize_can_replace_repair_fallback_with_valid_kind_geometry() {
        let placements = [placement("table", 1, 1, 12, 1)];
        let policy = DashboardPlacementSizePolicy::new();

        assert_eq!(
            validate_dashboard_resize(
                &placements,
                &"table",
                GridSize::new(12, 4),
                "table",
                &policy,
            )
            .expect("repair may replace the fallback with a valid Table rectangle")
            .rect,
            GridRect::new(1, 1, 12, 4)
        );
    }

    #[test]
    fn table_minimum_does_not_impose_an_independent_height_maximum() {
        let placements = [placement("table", 1, 1, 12, 7)];
        let policy = DashboardPlacementSizePolicy::new();

        assert_eq!(
            validate_dashboard_resize(
                &placements,
                &"table",
                GridSize::new(12, 240),
                "table",
                &policy,
            )
            .expect("Table may grow through the remaining Dashboard rows")
            .rect,
            GridRect::new(1, 1, 12, 240)
        );
    }
}
