use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};
use serde_json::Value;
use tessara_core::grid_layout::{GridPlacement, GridRect, GridSize};

use crate::composition::{
    CompositionError, DASHBOARD_GRID_CONSTRAINTS, DASHBOARD_HARD_MINIMUM, validate_dashboard_layout,
};

/// The only Dashboard placement schema this slice reads and writes.
pub const DASHBOARD_PLACEMENT_SCHEMA_VERSION: u32 = 1;

/// Typed Dashboard placement configuration persisted inside placement config.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct DashboardPlacementConfigV1 {
    pub schema_version: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    pub grid_row: i32,
    pub grid_column: i32,
    pub grid_width: i32,
    pub grid_height: i32,
}

impl DashboardPlacementConfigV1 {
    pub fn new(title: Option<String>, rect: GridRect) -> Result<Self, CompositionError> {
        Self::new_with_minimum(title, rect, DASHBOARD_HARD_MINIMUM)
    }

    /// Constructs V1 config while applying a code-defined component minimum.
    pub fn new_with_minimum(
        title: Option<String>,
        rect: GridRect,
        minimum: GridSize,
    ) -> Result<Self, CompositionError> {
        DASHBOARD_GRID_CONSTRAINTS
            .validate_rect_with_minimum(rect, minimum)
            .map_err(CompositionError::from)?;
        Ok(Self {
            schema_version: DASHBOARD_PLACEMENT_SCHEMA_VERSION,
            title,
            grid_row: rect.row,
            grid_column: rect.column,
            grid_width: rect.width,
            grid_height: rect.height,
        })
    }

    pub const fn rect(&self) -> GridRect {
        GridRect::new(
            self.grid_row,
            self.grid_column,
            self.grid_width,
            self.grid_height,
        )
    }
}

/// Classification returned when persisted placement config is decoded.
#[derive(Debug, Clone, Copy, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DashboardPlacementConfigState {
    /// A valid, executable V1 payload.
    Valid,
    /// Untagged pre-V1 data displayed with deterministic fallback geometry and
    /// normalized to V1 on the next authorized save.
    Legacy,
    /// A malformed or invalid V1 payload. It is displayable using fallback
    /// geometry but must not execute until a manager repairs it.
    NeedsRepair,
    /// A tagged schema newer than this service understands. Raw JSON remains
    /// opaque and must not be rewritten by this version.
    FutureSchema,
}

/// Decoded config plus the non-persisted display fallback and preserved source.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct ParsedDashboardPlacementConfig {
    pub raw_config: Value,
    pub title: Option<String>,
    pub display_rect: GridRect,
    pub config_state: DashboardPlacementConfigState,
    /// Present only for valid V1 or safely normalizable legacy config.
    pub normalized_config: Option<DashboardPlacementConfigV1>,
    /// String form avoids losing future JSON integer versions wider than i64.
    pub unsupported_schema_version: Option<String>,
}

impl ParsedDashboardPlacementConfig {
    pub fn is_executable(&self) -> bool {
        matches!(
            self.config_state,
            DashboardPlacementConfigState::Valid | DashboardPlacementConfigState::Legacy
        )
    }

    pub fn should_normalize_on_save(&self) -> bool {
        self.config_state == DashboardPlacementConfigState::Legacy
            && self.normalized_config.is_some()
    }

    /// Returns executable typed config or a stable domain error.
    pub fn require_executable(&self) -> Result<&DashboardPlacementConfigV1, CompositionError> {
        match self.config_state {
            DashboardPlacementConfigState::Valid | DashboardPlacementConfigState::Legacy => self
                .normalized_config
                .as_ref()
                .ok_or(CompositionError::ConfigNeedsRepair),
            DashboardPlacementConfigState::NeedsRepair => Err(CompositionError::ConfigNeedsRepair),
            DashboardPlacementConfigState::FutureSchema => {
                Err(CompositionError::UnsupportedConfigSchema {
                    schema_version: self
                        .unsupported_schema_version
                        .clone()
                        .unwrap_or_else(|| "unknown".to_string()),
                })
            }
        }
    }
}

/// Placement identity and signed legacy position used for fallback ordering.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct LegacyPlacementKey<Id> {
    pub placement_id: Id,
    pub position: i32,
}

impl<Id> LegacyPlacementKey<Id> {
    pub const fn new(placement_id: Id, position: i32) -> Self {
        Self {
            placement_id,
            position,
        }
    }
}

/// One stored row supplied to the batch config decoder.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct DashboardPlacementConfigInput<Id> {
    pub placement_id: Id,
    pub position: i32,
    pub raw_config: Value,
    pub minimum: GridSize,
}

impl<Id> DashboardPlacementConfigInput<Id> {
    pub const fn new(
        placement_id: Id,
        position: i32,
        raw_config: Value,
        minimum: GridSize,
    ) -> Self {
        Self {
            placement_id,
            position,
            raw_config,
            minimum,
        }
    }
}

/// One identity-preserving result from the batch config decoder.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct ParsedDashboardPlacement<Id> {
    pub placement_id: Id,
    pub config: ParsedDashboardPlacementConfig,
}

/// Assigns deterministic full-width, one-row fallback geometry.
///
/// Rows are sorted by signed `(position, placement_id)` before consecutive rows
/// are assigned from one. Negative and duplicate legacy positions are therefore
/// safe and deterministic. Callers may use the same function for malformed or
/// future-schema rows because all three states share this display footprint.
pub fn legacy_fallback_layout<Id: Clone + Ord>(
    rows: &[LegacyPlacementKey<Id>],
) -> Result<Vec<GridPlacement<Id>>, CompositionError> {
    fallback_layout_avoiding_rows(rows, &BTreeSet::new())
}

fn fallback_layout_avoiding_rows<Id: Clone + Ord>(
    rows: &[LegacyPlacementKey<Id>],
    occupied_rows: &BTreeSet<i32>,
) -> Result<Vec<GridPlacement<Id>>, CompositionError> {
    if rows.len() > DASHBOARD_GRID_CONSTRAINTS.max_placements() {
        return Err(CompositionError::PlacementLimitExceeded {
            count: rows.len(),
            max: DASHBOARD_GRID_CONSTRAINTS.max_placements(),
        });
    }

    let mut ordered = rows.to_vec();
    ordered.sort_by(|left, right| {
        (&left.position, &left.placement_id).cmp(&(&right.position, &right.placement_id))
    });
    let mut available_rows =
        (1..=DASHBOARD_GRID_CONSTRAINTS.max_rows()).filter(|row| !occupied_rows.contains(row));
    let mut placements = Vec::with_capacity(ordered.len());
    for row in ordered {
        let Some(grid_row) = available_rows.next() else {
            return Err(CompositionError::NoSpace);
        };
        placements.push(GridPlacement::new(
            row.placement_id,
            GridRect::new(grid_row, 1, DASHBOARD_GRID_CONSTRAINTS.columns(), 1),
        ));
    }
    DASHBOARD_GRID_CONSTRAINTS
        .validate_layout(&placements)
        .map_err(CompositionError::from)?;
    Ok(placements)
}

/// Classifies config without requiring fallback geometry.
///
/// This is useful to inspect one row, while
/// [`parse_dashboard_placement_configs`] is the preferred complete-layout path.
pub fn classify_dashboard_placement_config(
    raw_config: &Value,
    minimum: GridSize,
) -> DashboardPlacementConfigState {
    match raw_config.get("schema_version") {
        None => DashboardPlacementConfigState::Legacy,
        Some(version)
            if version.as_u64() == Some(u64::from(DASHBOARD_PLACEMENT_SCHEMA_VERSION)) =>
        {
            match serde_json::from_value::<DashboardPlacementConfigV1>(raw_config.clone()) {
                Ok(config)
                    if config.schema_version == DASHBOARD_PLACEMENT_SCHEMA_VERSION
                        && DASHBOARD_GRID_CONSTRAINTS
                            .validate_rect_with_minimum(config.rect(), minimum)
                            .is_ok() =>
                {
                    DashboardPlacementConfigState::Valid
                }
                _ => DashboardPlacementConfigState::NeedsRepair,
            }
        }
        Some(version) if version.as_u64().is_some_and(|value| value > 1) => {
            DashboardPlacementConfigState::FutureSchema
        }
        Some(_) => DashboardPlacementConfigState::NeedsRepair,
    }
}

/// Classifies, assigns consecutive fallback rows, parses, and validates a
/// complete display layout in one deterministic operation.
///
/// Only legacy, invalid V1, and future-schema rows participate in fallback
/// ordering. Explicit valid V1 geometry is retained, and fallback assignment
/// skips every row occupied by valid geometry. This keeps mixed-version data
/// displayable without altering either the valid rectangle or fallback order.
pub fn parse_dashboard_placement_configs<Id: Clone + Ord>(
    inputs: &[DashboardPlacementConfigInput<Id>],
) -> Result<Vec<ParsedDashboardPlacement<Id>>, CompositionError> {
    if inputs.len() > DASHBOARD_GRID_CONSTRAINTS.max_placements() {
        return Err(CompositionError::PlacementLimitExceeded {
            count: inputs.len(),
            max: DASHBOARD_GRID_CONSTRAINTS.max_placements(),
        });
    }

    let mut ids = BTreeSet::new();
    for input in inputs {
        if !ids.insert(input.placement_id.clone()) {
            return Err(CompositionError::DuplicatePlacementId);
        }
    }

    let states: Vec<_> = inputs
        .iter()
        .map(|input| classify_dashboard_placement_config(&input.raw_config, input.minimum))
        .collect();
    let fallback_keys: Vec<_> = inputs
        .iter()
        .zip(&states)
        .filter(|(_, state)| **state != DashboardPlacementConfigState::Valid)
        .map(|(input, _)| LegacyPlacementKey::new(input.placement_id.clone(), input.position))
        .collect();
    let occupied_rows = inputs
        .iter()
        .zip(&states)
        .filter(|(_, state)| **state == DashboardPlacementConfigState::Valid)
        .try_fold(BTreeSet::new(), |mut occupied, (input, _)| {
            let config =
                serde_json::from_value::<DashboardPlacementConfigV1>(input.raw_config.clone())
                    .map_err(|_| CompositionError::ConfigNeedsRepair)?;
            let bottom = config.grid_row.checked_add(config.grid_height - 1).ok_or(
                CompositionError::InvalidGeometry(
                    tessara_core::grid_layout::GridLayoutError::ArithmeticOverflow,
                ),
            )?;
            occupied.extend(config.grid_row..=bottom);
            Ok::<_, CompositionError>(occupied)
        })?;
    let fallback_by_id: BTreeMap<_, _> =
        fallback_layout_avoiding_rows(&fallback_keys, &occupied_rows)?
            .into_iter()
            .map(|placement| (placement.id, placement.rect))
            .collect();
    let valid_placeholder = GridRect::new(1, 1, DASHBOARD_GRID_CONSTRAINTS.columns(), 1);

    let parsed: Vec<_> = inputs
        .iter()
        .map(|input| {
            let fallback_rect = fallback_by_id
                .get(&input.placement_id)
                .copied()
                .unwrap_or(valid_placeholder);
            parse_dashboard_placement_config(input.raw_config.clone(), fallback_rect, input.minimum)
                .map(|config| ParsedDashboardPlacement {
                    placement_id: input.placement_id.clone(),
                    config,
                })
        })
        .collect::<Result<_, _>>()?;
    let display_layout: Vec<_> = parsed
        .iter()
        .map(|placement| {
            GridPlacement::new(
                placement.placement_id.clone(),
                placement.config.display_rect,
            )
        })
        .collect();
    validate_dashboard_layout(&display_layout)?;
    Ok(parsed)
}

/// Decodes persisted config without executing or silently repairing invalid
/// tagged payloads.
///
/// Untagged JSON is legacy regardless of any geometry-looking keys it contains;
/// only a string `title` is preserved. Invalid V1 uses the supplied display
/// fallback with `needs_repair`. Unknown future schemas use the same fallback
/// while retaining the exact raw JSON and offering no V1 normalization. Use
/// [`parse_dashboard_placement_configs`] for multiple rows. Callers using this
/// single-row function must validate the assembled display layout themselves;
/// mixed explicit/fallback conflicts are errors, not permission to alter the
/// mandated fallback footprint.
pub fn parse_dashboard_placement_config(
    raw_config: Value,
    fallback_rect: GridRect,
    minimum: GridSize,
) -> Result<ParsedDashboardPlacementConfig, CompositionError> {
    validate_fallback_rect(fallback_rect)?;
    let title = string_title(&raw_config);
    match classify_dashboard_placement_config(&raw_config, minimum) {
        DashboardPlacementConfigState::Legacy => {
            match DashboardPlacementConfigV1::new_with_minimum(
                title.clone(),
                fallback_rect,
                minimum,
            ) {
                Ok(normalized) => Ok(ParsedDashboardPlacementConfig {
                    raw_config,
                    title,
                    display_rect: fallback_rect,
                    config_state: DashboardPlacementConfigState::Legacy,
                    normalized_config: Some(normalized),
                    unsupported_schema_version: None,
                }),
                Err(_) => Ok(needs_repair(raw_config, title, fallback_rect)),
            }
        }
        DashboardPlacementConfigState::Valid => {
            match serde_json::from_value::<DashboardPlacementConfigV1>(raw_config.clone()) {
                Ok(config) => Ok(ParsedDashboardPlacementConfig {
                    title: config.title.clone(),
                    display_rect: config.rect(),
                    raw_config,
                    config_state: DashboardPlacementConfigState::Valid,
                    normalized_config: Some(config),
                    unsupported_schema_version: None,
                }),
                _ => Ok(needs_repair(raw_config, title, fallback_rect)),
            }
        }
        DashboardPlacementConfigState::FutureSchema => {
            let schema_version = raw_config
                .get("schema_version")
                .map(Value::to_string)
                .unwrap_or_else(|| "unknown".to_string());
            Ok(ParsedDashboardPlacementConfig {
                raw_config,
                title: None,
                display_rect: fallback_rect,
                config_state: DashboardPlacementConfigState::FutureSchema,
                normalized_config: None,
                unsupported_schema_version: Some(schema_version),
            })
        }
        DashboardPlacementConfigState::NeedsRepair => {
            Ok(needs_repair(raw_config, title, fallback_rect))
        }
    }
}

/// Encodes canonical V1 config after revalidating the hard Dashboard bounds.
pub fn encode_dashboard_placement_config(
    config: &DashboardPlacementConfigV1,
) -> Result<Value, CompositionError> {
    if config.schema_version != DASHBOARD_PLACEMENT_SCHEMA_VERSION {
        return Err(CompositionError::ConfigNeedsRepair);
    }
    DASHBOARD_GRID_CONSTRAINTS
        .validate_rect(config.rect())
        .map_err(CompositionError::from)?;
    serde_json::to_value(config).map_err(|error| CompositionError::ConfigEncoding {
        message: error.to_string(),
    })
}

fn needs_repair(
    raw_config: Value,
    title: Option<String>,
    display_rect: GridRect,
) -> ParsedDashboardPlacementConfig {
    ParsedDashboardPlacementConfig {
        raw_config,
        title,
        display_rect,
        config_state: DashboardPlacementConfigState::NeedsRepair,
        normalized_config: None,
        unsupported_schema_version: None,
    }
}

fn validate_fallback_rect(fallback_rect: GridRect) -> Result<(), CompositionError> {
    DASHBOARD_GRID_CONSTRAINTS
        .validate_rect(fallback_rect)
        .map_err(CompositionError::from)?;
    if fallback_rect.column != 1
        || fallback_rect.width != DASHBOARD_GRID_CONSTRAINTS.columns()
        || fallback_rect.height != 1
    {
        return Err(CompositionError::InvalidFallbackGeometry {
            rect: fallback_rect,
        });
    }
    Ok(())
}

fn string_title(value: &Value) -> Option<String> {
    value
        .get("title")
        .and_then(Value::as_str)
        .map(str::to_owned)
}

#[cfg(test)]
mod tests {
    use serde_json::{Value, json};
    use tessara_core::grid_layout::{GridRect, GridSize};

    use super::{
        DashboardPlacementConfigInput, DashboardPlacementConfigState, DashboardPlacementConfigV1,
        LegacyPlacementKey, encode_dashboard_placement_config, legacy_fallback_layout,
        parse_dashboard_placement_config, parse_dashboard_placement_configs,
    };
    use crate::composition::CompositionError;

    const FALLBACK: GridRect = GridRect::new(4, 1, 12, 1);

    #[test]
    fn v1_config_round_trips_with_canonical_schema() {
        let config =
            DashboardPlacementConfigV1::new(Some("Revenue".to_string()), GridRect::new(2, 3, 6, 4))
                .expect("valid config");
        let encoded = encode_dashboard_placement_config(&config).expect("encode");
        assert_eq!(encoded["schema_version"], json!(1));

        let parsed = parse_dashboard_placement_config(encoded.clone(), FALLBACK, GridSize::ONE)
            .expect("parse");
        assert_eq!(parsed.config_state, DashboardPlacementConfigState::Valid);
        assert_eq!(parsed.display_rect, GridRect::new(2, 3, 6, 4));
        assert_eq!(parsed.raw_config, encoded);
        assert_eq!(parsed.require_executable(), Ok(&config));
        assert!(!parsed.should_normalize_on_save());
    }

    #[test]
    fn v1_config_round_trips_above_the_old_six_row_limit() {
        let config = DashboardPlacementConfigV1::new(
            Some("Tall placement".to_string()),
            GridRect::new(2, 1, 12, 239),
        )
        .expect("remaining Dashboard rows are valid");
        let encoded = encode_dashboard_placement_config(&config).expect("encode");
        let parsed = parse_dashboard_placement_config(encoded, FALLBACK, GridSize::ONE)
            .expect("parse tall V1 config");

        assert_eq!(parsed.config_state, DashboardPlacementConfigState::Valid);
        assert_eq!(parsed.display_rect, GridRect::new(2, 1, 12, 239));
        assert_eq!(parsed.require_executable(), Ok(&config));
    }

    #[test]
    fn empty_and_arbitrary_untagged_json_use_legacy_fallback() {
        for raw in [json!({}), json!(["arbitrary"]), json!("old config")] {
            let parsed = parse_dashboard_placement_config(raw.clone(), FALLBACK, GridSize::ONE)
                .expect("legacy parse");
            assert_eq!(parsed.config_state, DashboardPlacementConfigState::Legacy);
            assert_eq!(parsed.display_rect, FALLBACK);
            assert_eq!(parsed.raw_config, raw);
            assert!(parsed.is_executable());
            assert!(parsed.should_normalize_on_save());
        }
    }

    #[test]
    fn legacy_preserves_only_string_title_and_ignores_untyped_geometry() {
        let raw = json!({
            "title": "Legacy title",
            "grid_row": -99,
            "grid_column": 400,
            "grid_width": 999,
            "grid_height": 0
        });
        let parsed =
            parse_dashboard_placement_config(raw, FALLBACK, GridSize::ONE).expect("legacy parse");
        let normalized = parsed.normalized_config.expect("normalizable");
        assert_eq!(parsed.title.as_deref(), Some("Legacy title"));
        assert_eq!(normalized.title.as_deref(), Some("Legacy title"));
        assert_eq!(normalized.rect(), FALLBACK);

        let numeric_title =
            parse_dashboard_placement_config(json!({"title": 42}), FALLBACK, GridSize::ONE)
                .expect("legacy parse");
        assert_eq!(numeric_title.title, None);
    }

    #[test]
    fn malformed_and_invalid_v1_need_repair_and_cannot_execute() {
        for raw in [
            json!({"schema_version": 1, "title": "missing geometry"}),
            json!({"schema_version": 1, "grid_row": "one", "grid_column": 1, "grid_width": 12, "grid_height": 1}),
            json!({"schema_version": 1, "grid_row": 240, "grid_column": 1, "grid_width": 12, "grid_height": 2}),
            json!({"schema_version": "1", "grid_row": 1, "grid_column": 1, "grid_width": 12, "grid_height": 1}),
        ] {
            let parsed = parse_dashboard_placement_config(raw, FALLBACK, GridSize::ONE)
                .expect("invalid config is a readable state");
            assert_eq!(
                parsed.config_state,
                DashboardPlacementConfigState::NeedsRepair
            );
            assert_eq!(parsed.display_rect, FALLBACK);
            assert!(!parsed.is_executable());
            assert_eq!(
                parsed.require_executable(),
                Err(CompositionError::ConfigNeedsRepair)
            );
        }
    }

    #[test]
    fn v1_respects_code_defined_component_minimum() {
        let raw = json!({
            "schema_version": 1,
            "grid_row": 1,
            "grid_column": 1,
            "grid_width": 5,
            "grid_height": 2
        });
        let parsed =
            parse_dashboard_placement_config(raw, FALLBACK, GridSize::new(6, 2)).expect("parse");
        assert_eq!(
            parsed.config_state,
            DashboardPlacementConfigState::NeedsRepair
        );
    }

    #[test]
    fn unknown_future_schema_is_opaque_and_raw_json_is_unchanged() {
        let raw = json!({
            "schema_version": 2,
            "title": "Future",
            "geometry_v2": {"x": 42},
            "unknown": [1, 2, 3]
        });
        let parsed = parse_dashboard_placement_config(raw.clone(), FALLBACK, GridSize::ONE)
            .expect("future config is displayable");
        assert_eq!(
            parsed.config_state,
            DashboardPlacementConfigState::FutureSchema
        );
        assert_eq!(parsed.raw_config, raw);
        assert_eq!(parsed.title, None);
        assert_eq!(parsed.normalized_config, None);
        assert_eq!(parsed.display_rect, FALLBACK);
        assert!(!parsed.is_executable());
        assert_eq!(
            parsed.require_executable(),
            Err(CompositionError::UnsupportedConfigSchema {
                schema_version: "2".to_string()
            })
        );
    }

    #[test]
    fn fallback_order_handles_negative_and_duplicate_legacy_positions() {
        let rows = [
            LegacyPlacementKey::new("b", -1),
            LegacyPlacementKey::new("c", 2),
            LegacyPlacementKey::new("a", -1),
            LegacyPlacementKey::new("z", -3),
        ];
        let layout = legacy_fallback_layout(&rows).expect("fallback");
        assert_eq!(
            layout
                .iter()
                .map(|placement| (placement.id, placement.rect.row))
                .collect::<Vec<_>>(),
            vec![("z", 1), ("a", 2), ("b", 3), ("c", 4)]
        );
        assert!(
            layout
                .iter()
                .all(|placement| placement.rect == GridRect::new(placement.rect.row, 1, 12, 1))
        );
    }

    #[test]
    fn batch_parser_ranks_only_fallback_state_rows() {
        let future_raw = json!({"schema_version": 2, "opaque": true});
        let inputs = [
            DashboardPlacementConfigInput::new(
                "valid",
                -10,
                json!({
                    "schema_version": 1,
                    "grid_row": 10,
                    "grid_column": 1,
                    "grid_width": 3,
                    "grid_height": 2
                }),
                GridSize::ONE,
            ),
            DashboardPlacementConfigInput::new(
                "legacy",
                -1,
                json!({"title": "Legacy"}),
                GridSize::ONE,
            ),
            DashboardPlacementConfigInput::new(
                "invalid",
                -1,
                json!({"schema_version": 1, "grid_row": "bad"}),
                GridSize::ONE,
            ),
            DashboardPlacementConfigInput::new("future", -3, future_raw.clone(), GridSize::ONE),
        ];
        let parsed = parse_dashboard_placement_configs(&inputs).expect("batch parse");
        let config_for = |id| {
            &parsed
                .iter()
                .find(|placement| placement.placement_id == id)
                .expect("placement")
                .config
        };

        assert_eq!(config_for("valid").display_rect.row, 10);
        assert_eq!(config_for("future").display_rect.row, 1);
        assert_eq!(config_for("invalid").display_rect.row, 2);
        assert_eq!(config_for("legacy").display_rect.row, 3);
        assert_eq!(
            config_for("future").config_state,
            DashboardPlacementConfigState::FutureSchema
        );
        assert_eq!(config_for("future").raw_config, future_raw);
    }

    #[test]
    fn batch_parser_skips_rows_occupied_by_explicit_geometry() {
        let inputs = [
            DashboardPlacementConfigInput::new(
                "valid",
                0,
                json!({
                    "schema_version": 1,
                    "grid_row": 1,
                    "grid_column": 1,
                    "grid_width": 1,
                    "grid_height": 1
                }),
                GridSize::ONE,
            ),
            DashboardPlacementConfigInput::new("legacy", 1, json!({}), GridSize::ONE),
        ];
        let parsed = parse_dashboard_placement_configs(&inputs).expect("mixed layout");
        let legacy = parsed
            .iter()
            .find(|placement| placement.placement_id == "legacy")
            .expect("legacy placement");
        assert_eq!(legacy.config.display_rect, GridRect::new(2, 1, 12, 1));
    }

    #[test]
    fn fallback_preserves_order_while_skipping_multirow_valid_footprints() {
        let inputs = [
            DashboardPlacementConfigInput::new(
                "valid",
                0,
                json!({
                    "schema_version": 1,
                    "grid_row": 1,
                    "grid_column": 4,
                    "grid_width": 3,
                    "grid_height": 3
                }),
                GridSize::ONE,
            ),
            DashboardPlacementConfigInput::new("later", 10, json!({}), GridSize::ONE),
            DashboardPlacementConfigInput::new("earlier", -10, json!({}), GridSize::ONE),
        ];
        let parsed = parse_dashboard_placement_configs(&inputs).expect("mixed layout");
        let row_for = |id| {
            parsed
                .iter()
                .find(|placement| placement.placement_id == id)
                .expect("placement")
                .config
                .display_rect
                .row
        };
        assert_eq!(row_for("earlier"), 4);
        assert_eq!(row_for("later"), 5);
    }

    #[test]
    fn mixed_layout_rejects_fallback_when_valid_geometry_occupies_every_row() {
        let inputs = [
            DashboardPlacementConfigInput::new(
                "valid",
                0,
                json!({
                    "schema_version": 1,
                    "grid_row": 1,
                    "grid_column": 1,
                    "grid_width": 1,
                    "grid_height": 240
                }),
                GridSize::ONE,
            ),
            DashboardPlacementConfigInput::new("legacy", 1, json!({}), GridSize::ONE),
        ];

        assert_eq!(
            parse_dashboard_placement_configs(&inputs),
            Err(CompositionError::NoSpace)
        );
    }

    #[test]
    fn fallback_accepts_240_rows_and_rejects_241() {
        let at_limit: Vec<_> = (0..240).map(|id| LegacyPlacementKey::new(id, id)).collect();
        let layout = legacy_fallback_layout(&at_limit).expect("240 rows fit");
        assert_eq!(layout.len(), 240);
        assert_eq!(layout.last().expect("last").rect.row, 240);

        let over_limit: Vec<_> = (0..241).map(|id| LegacyPlacementKey::new(id, id)).collect();
        assert_eq!(
            legacy_fallback_layout(&over_limit),
            Err(CompositionError::PlacementLimitExceeded {
                count: 241,
                max: 240
            })
        );
    }

    #[test]
    fn encoder_rejects_non_v1_or_invalid_geometry() {
        let wrong_schema = DashboardPlacementConfigV1 {
            schema_version: 2,
            title: None,
            grid_row: 1,
            grid_column: 1,
            grid_width: 12,
            grid_height: 1,
        };
        assert_eq!(
            encode_dashboard_placement_config(&wrong_schema),
            Err(CompositionError::ConfigNeedsRepair)
        );

        let invalid_geometry = DashboardPlacementConfigV1 {
            schema_version: 1,
            grid_row: 1,
            grid_column: 12,
            grid_width: 2,
            grid_height: 1,
            title: None,
        };
        assert!(matches!(
            encode_dashboard_placement_config(&invalid_geometry),
            Err(CompositionError::InvalidGeometry(_))
        ));
    }

    #[test]
    fn invalid_fallback_is_a_domain_geometry_error() {
        let result = parse_dashboard_placement_config(
            Value::Object(Default::default()),
            GridRect::new(241, 1, 12, 1),
            GridSize::ONE,
        );
        assert!(matches!(result, Err(CompositionError::InvalidGeometry(_))));

        let wrong_footprint = parse_dashboard_placement_config(
            Value::Object(Default::default()),
            GridRect::new(1, 2, 11, 1),
            GridSize::ONE,
        );
        assert_eq!(
            wrong_footprint,
            Err(CompositionError::InvalidFallbackGeometry {
                rect: GridRect::new(1, 2, 11, 1)
            })
        );
    }
}
