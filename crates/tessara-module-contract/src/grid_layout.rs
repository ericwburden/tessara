//! Framework-free grid geometry and deterministic layout operations.
//!
//! Coordinates are one-based. Persisted display positions are deliberately not
//! part of these types: callers derive zero-based positions from row-major
//! order after a layout operation succeeds.

use std::{cmp::Ordering, collections::BTreeSet};

use serde::{Deserialize, Serialize};

/// Width and height in grid cells.
#[derive(Debug, Clone, Copy, Eq, PartialEq, Serialize, Deserialize)]
pub struct GridSize {
    pub width: i32,
    pub height: i32,
}

impl GridSize {
    /// The hard minimum supported by the generic grid contract.
    pub const ONE: Self = Self::new(1, 1);

    pub const fn new(width: i32, height: i32) -> Self {
        Self { width, height }
    }
}

/// A one-based rectangle in a bounded grid.
#[derive(Debug, Clone, Copy, Eq, PartialEq, Serialize, Deserialize)]
pub struct GridRect {
    pub row: i32,
    pub column: i32,
    pub width: i32,
    pub height: i32,
}

impl GridRect {
    pub const fn new(row: i32, column: i32, width: i32, height: i32) -> Self {
        Self {
            row,
            column,
            width,
            height,
        }
    }

    pub const fn size(self) -> GridSize {
        GridSize::new(self.width, self.height)
    }

    /// Returns whether two non-empty rectangles share at least one cell.
    ///
    /// This arithmetic predicate does not validate grid bounds. Empty or
    /// overflowing rectangles return false; callers must validate untrusted
    /// rectangles before using the result as a layout decision.
    pub fn overlaps(self, other: Self) -> bool {
        let Some(self_right) = inclusive_end(self.column, self.width) else {
            return false;
        };
        let Some(self_bottom) = inclusive_end(self.row, self.height) else {
            return false;
        };
        let Some(other_right) = inclusive_end(other.column, other.width) else {
            return false;
        };
        let Some(other_bottom) = inclusive_end(other.row, other.height) else {
            return false;
        };

        self.column <= other_right
            && other.column <= self_right
            && self.row <= other_bottom
            && other.row <= self_bottom
    }

    /// Expands a non-empty rectangle into its occupied cells without applying
    /// grid bounds. This is useful while adapting legacy UI drafts that may
    /// overlap but must still reserve every represented cell.
    pub fn occupied_cells(self) -> Result<BTreeSet<(i32, i32)>, GridLayoutError> {
        if self.row < 1 {
            return Err(GridLayoutError::InvalidRow { row: self.row });
        }
        if self.column < 1 {
            return Err(GridLayoutError::InvalidColumn {
                column: self.column,
            });
        }
        let bottom = inclusive_end(self.row, self.height).ok_or({
            if self.height < 1 {
                GridLayoutError::HeightOutOfRange {
                    height: self.height,
                    min: 1,
                    max: i32::MAX,
                }
            } else {
                GridLayoutError::ArithmeticOverflow
            }
        })?;
        let right = inclusive_end(self.column, self.width).ok_or({
            if self.width < 1 {
                GridLayoutError::WidthOutOfRange {
                    width: self.width,
                    min: 1,
                    max: i32::MAX,
                }
            } else {
                GridLayoutError::ArithmeticOverflow
            }
        })?;
        let mut occupied = BTreeSet::new();
        for row in self.row..=bottom {
            for column in self.column..=right {
                occupied.insert((row, column));
            }
        }
        Ok(occupied)
    }
}

/// Cardinal direction used by keyboard-accessible placement movement.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum GridMoveDirection {
    Up,
    Down,
    Left,
    Right,
}

/// A direct inspector edit or a one-cell keyboard movement request.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum GridMoveRequest {
    Direct { row: i32, column: i32 },
    Keyboard(GridMoveDirection),
}

/// Dimension changed by a keyboard-accessible resize request.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum GridResizeAxis {
    Width,
    Height,
}

/// Direction of a one-cell keyboard-accessible resize request.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum GridResizeStep {
    Decrease,
    Increase,
}

/// A direct inspector edit or a one-cell keyboard resize request.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum GridResizeRequest {
    Direct(GridSize),
    Keyboard {
        axis: GridResizeAxis,
        step: GridResizeStep,
    },
}

/// A rectangle associated with an application-owned identifier.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct GridPlacement<Id> {
    pub id: Id,
    pub rect: GridRect,
}

impl<Id> GridPlacement<Id> {
    pub const fn new(id: Id, rect: GridRect) -> Self {
        Self { id, rect }
    }
}

/// Bounds and capacity for a generic grid.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct GridConstraints {
    columns: i32,
    max_rows: i32,
    max_placements: usize,
    max_height: i32,
}

impl GridConstraints {
    /// Creates bounded constraints with the generic hard minimum of 1x1.
    ///
    /// Definition errors are returned by validation operations. This is a
    /// `const fn` so bounded contexts can publish one canonical policy value.
    pub const fn new(columns: i32, max_rows: i32, max_placements: usize, max_height: i32) -> Self {
        Self {
            columns,
            max_rows,
            max_placements,
            max_height,
        }
    }

    pub const fn columns(self) -> i32 {
        self.columns
    }

    pub const fn max_rows(self) -> i32 {
        self.max_rows
    }

    pub const fn max_placements(self) -> usize {
        self.max_placements
    }

    pub const fn max_height(self) -> i32 {
        self.max_height
    }

    /// Verifies that the constraint definition can describe a usable grid.
    pub fn validate_definition(self) -> Result<(), GridLayoutError> {
        for (name, value) in [
            ("columns", i64::from(self.columns)),
            ("max_rows", i64::from(self.max_rows)),
            ("max_height", i64::from(self.max_height)),
        ] {
            if value < 1 {
                return Err(GridLayoutError::InvalidConstraint { name, value });
            }
        }
        if self.max_placements == 0 {
            return Err(GridLayoutError::InvalidConstraint {
                name: "max_placements",
                value: 0,
            });
        }
        if self.max_height > self.max_rows {
            return Err(GridLayoutError::InvalidConstraint {
                name: "max_height",
                value: i64::from(self.max_height),
            });
        }
        Ok(())
    }

    /// Validates a rectangle against the generic 1x1 hard minimum.
    pub fn validate_rect(self, rect: GridRect) -> Result<(), GridLayoutError> {
        self.validate_rect_with_minimum(rect, GridSize::ONE)
    }

    /// Validates a rectangle and a bounded-context minimum size.
    pub fn validate_rect_with_minimum(
        self,
        rect: GridRect,
        minimum: GridSize,
    ) -> Result<(), GridLayoutError> {
        self.validate_definition()?;
        self.validate_minimum(minimum)?;

        if rect.row < 1 {
            return Err(GridLayoutError::InvalidRow { row: rect.row });
        }
        if rect.column < 1 {
            return Err(GridLayoutError::InvalidColumn {
                column: rect.column,
            });
        }
        if rect.width < minimum.width || rect.width > self.columns {
            return Err(GridLayoutError::WidthOutOfRange {
                width: rect.width,
                min: minimum.width,
                max: self.columns,
            });
        }
        if rect.height < minimum.height || rect.height > self.max_height {
            return Err(GridLayoutError::HeightOutOfRange {
                height: rect.height,
                min: minimum.height,
                max: self.max_height,
            });
        }

        let right =
            inclusive_end(rect.column, rect.width).ok_or(GridLayoutError::ArithmeticOverflow)?;
        if right > self.columns {
            return Err(GridLayoutError::ColumnOverflow {
                column: rect.column,
                width: rect.width,
                columns: self.columns,
            });
        }

        let bottom =
            inclusive_end(rect.row, rect.height).ok_or(GridLayoutError::ArithmeticOverflow)?;
        if bottom > self.max_rows {
            return Err(GridLayoutError::RowOverflow {
                row: rect.row,
                height: rect.height,
                max_rows: self.max_rows,
            });
        }

        Ok(())
    }

    /// Validates capacity, identifiers, rectangles, and pairwise occupancy.
    pub fn validate_layout<Id: PartialEq>(
        self,
        placements: &[GridPlacement<Id>],
    ) -> Result<(), GridLayoutError> {
        self.validate_layout_with(placements, |_| GridSize::ONE)
    }

    /// Validates a layout with a caller-provided minimum for each placement.
    pub fn validate_layout_with<Id: PartialEq, MinimumFor>(
        self,
        placements: &[GridPlacement<Id>],
        mut minimum_for: MinimumFor,
    ) -> Result<(), GridLayoutError>
    where
        MinimumFor: FnMut(&GridPlacement<Id>) -> GridSize,
    {
        self.validate_definition()?;
        self.validate_count(placements.len())?;

        for placement in placements {
            self.validate_rect_with_minimum(placement.rect, minimum_for(placement))?;
        }

        for (index, placement) in placements.iter().enumerate() {
            for other in &placements[index + 1..] {
                if placement.id == other.id {
                    return Err(GridLayoutError::DuplicatePlacementId);
                }
                if placement.rect.overlaps(other.rect) {
                    return Err(GridLayoutError::Overlap);
                }
            }
        }

        Ok(())
    }

    /// Expands a valid layout into its occupied cells.
    pub fn occupancy<Id: PartialEq>(
        self,
        placements: &[GridPlacement<Id>],
    ) -> Result<BTreeSet<(i32, i32)>, GridLayoutError> {
        self.validate_layout(placements)?;
        let mut occupied = BTreeSet::new();
        for placement in placements {
            occupied.extend(placement.rect.occupied_cells()?);
        }
        Ok(occupied)
    }

    fn validate_count(self, count: usize) -> Result<(), GridLayoutError> {
        if count > self.max_placements {
            Err(GridLayoutError::PlacementLimitExceeded {
                count,
                max: self.max_placements,
            })
        } else {
            Ok(())
        }
    }

    fn validate_minimum(self, minimum: GridSize) -> Result<(), GridLayoutError> {
        if minimum.width < 1 || minimum.width > self.columns {
            return Err(GridLayoutError::InvalidMinimum {
                width: minimum.width,
                height: minimum.height,
            });
        }
        if minimum.height < 1 || minimum.height > self.max_height {
            return Err(GridLayoutError::InvalidMinimum {
                width: minimum.width,
                height: minimum.height,
            });
        }
        Ok(())
    }
}

/// Sorts placements by row, column, and identifier.
///
/// The identifier tie-break keeps the function deterministic for malformed or
/// partially constructed layouts before overlap validation runs.
pub fn sort_row_major<Id: Ord>(placements: &mut [GridPlacement<Id>]) {
    placements.sort_by(row_major_cmp);
}

/// Derives stable zero-based display positions without persisting them as
/// independent layout state.
pub fn derive_row_major_positions<Id: Clone + Ord>(
    placements: &[GridPlacement<Id>],
) -> Vec<(Id, usize)> {
    let mut ordered = placements.to_vec();
    sort_row_major(&mut ordered);
    ordered
        .into_iter()
        .enumerate()
        .map(|(position, placement)| (placement.id, position))
        .collect()
}

/// Moves one placement to an explicit target and deterministically reflows
/// collisions forward in row-major order.
///
/// Input rectangles must be individually valid and identifiers must be unique,
/// but the input may already contain overlaps. This deliberately preserves the
/// Form builder's ability to repair older overlapping drafts while producing a
/// fully validated, non-overlapping result.
///
/// Placements are considered from their requested row-major start. The moved
/// placement wins ties at the same top-left cell, matching the existing Form
/// builder policy. A colliding rectangle scans forward from its requested
/// top-left cell for the first available fit. The returned layout is canonical
/// row-major order.
pub fn reflow_movement<Id: Clone + Eq + Ord>(
    constraints: GridConstraints,
    placements: &[GridPlacement<Id>],
    moved_id: &Id,
    target_row: i32,
    target_column: i32,
) -> Result<Vec<GridPlacement<Id>>, GridLayoutError> {
    constraints.validate_definition()?;
    constraints.validate_count(placements.len())?;
    for (index, placement) in placements.iter().enumerate() {
        constraints.validate_rect(placement.rect)?;
        if placements[index + 1..]
            .iter()
            .any(|other| placement.id == other.id)
        {
            return Err(GridLayoutError::DuplicatePlacementId);
        }
    }

    let moved = placements
        .iter()
        .find(|placement| &placement.id == moved_id)
        .ok_or(GridLayoutError::PlacementNotFound)?;
    let moved_rect = GridRect::new(
        target_row,
        target_column,
        moved.rect.width,
        moved.rect.height,
    );
    constraints.validate_rect(moved_rect)?;

    let mut requested: Vec<_> = placements
        .iter()
        .cloned()
        .map(|mut placement| {
            if &placement.id == moved_id {
                placement.rect = moved_rect;
            }
            placement
        })
        .collect();
    requested.sort_by(|left, right| {
        (left.rect.row, left.rect.column)
            .cmp(&(right.rect.row, right.rect.column))
            .then_with(|| {
                let left_moved = &left.id == moved_id;
                let right_moved = &right.id == moved_id;
                right_moved.cmp(&left_moved)
            })
            .then_with(|| left.id.cmp(&right.id))
    });

    let mut placed: Vec<GridPlacement<Id>> = Vec::with_capacity(requested.len());
    for placement in requested {
        let start = linear_index(constraints, placement.rect.row, placement.rect.column)?;
        let cell_count = i64::from(constraints.columns())
            .checked_mul(i64::from(constraints.max_rows()))
            .ok_or(GridLayoutError::ArithmeticOverflow)?;
        let mut replacement = None;
        for index in start..cell_count {
            let row = i32::try_from(index / i64::from(constraints.columns()) + 1)
                .map_err(|_| GridLayoutError::ArithmeticOverflow)?;
            let column = i32::try_from(index % i64::from(constraints.columns()) + 1)
                .map_err(|_| GridLayoutError::ArithmeticOverflow)?;
            let candidate = GridRect::new(row, column, placement.rect.width, placement.rect.height);
            if constraints.validate_rect(candidate).is_ok()
                && !placed
                    .iter()
                    .any(|accepted| candidate.overlaps(accepted.rect))
            {
                replacement = Some(candidate);
                break;
            }
        }

        let rect = replacement.ok_or(GridLayoutError::NoSpace)?;
        placed.push(GridPlacement::new(placement.id, rect));
    }

    sort_row_major(&mut placed);
    constraints.validate_layout(&placed)?;
    Ok(placed)
}

/// Validates a direct resize without moving any other placement.
pub fn validate_resize<Id: Clone + PartialEq>(
    constraints: GridConstraints,
    placements: &[GridPlacement<Id>],
    placement_id: &Id,
    requested: GridSize,
    minimum: GridSize,
) -> Result<GridPlacement<Id>, GridLayoutError> {
    constraints.validate_layout(placements)?;
    let existing = placements
        .iter()
        .find(|placement| &placement.id == placement_id)
        .ok_or(GridLayoutError::PlacementNotFound)?;
    let candidate = GridPlacement::new(
        existing.id.clone(),
        GridRect::new(
            existing.rect.row,
            existing.rect.column,
            requested.width,
            requested.height,
        ),
    );
    constraints.validate_rect_with_minimum(candidate.rect, minimum)?;
    if placements
        .iter()
        .any(|placement| &placement.id != placement_id && candidate.rect.overlaps(placement.rect))
    {
        return Err(GridLayoutError::Overlap);
    }
    Ok(candidate)
}

/// Resolves direct and keyboard movement through one bounded geometry path.
///
/// Collision reflow remains a separate operation because this function only
/// answers which rectangle the interaction requested.
pub fn resolve_move_request(
    constraints: GridConstraints,
    current: GridRect,
    request: GridMoveRequest,
) -> Result<GridRect, GridLayoutError> {
    constraints.validate_rect(current)?;
    let (row, column) = match request {
        GridMoveRequest::Direct { row, column } => (row, column),
        GridMoveRequest::Keyboard(direction) => match direction {
            GridMoveDirection::Up => (
                current
                    .row
                    .checked_sub(1)
                    .ok_or(GridLayoutError::ArithmeticOverflow)?,
                current.column,
            ),
            GridMoveDirection::Down => (
                current
                    .row
                    .checked_add(1)
                    .ok_or(GridLayoutError::ArithmeticOverflow)?,
                current.column,
            ),
            GridMoveDirection::Left => (
                current.row,
                current
                    .column
                    .checked_sub(1)
                    .ok_or(GridLayoutError::ArithmeticOverflow)?,
            ),
            GridMoveDirection::Right => (
                current.row,
                current
                    .column
                    .checked_add(1)
                    .ok_or(GridLayoutError::ArithmeticOverflow)?,
            ),
        },
    };
    let requested = GridRect::new(row, column, current.width, current.height);
    constraints.validate_rect(requested)?;
    Ok(requested)
}

/// Resolves direct and keyboard sizing through one bounded geometry path.
///
/// Pairwise collision validation remains in [`validate_resize`]; this helper
/// normalizes all interaction modes to one requested size first. The current
/// rectangle must satisfy the grid's hard geometry bounds, while only the
/// requested result must satisfy the caller's bounded-context minimum. That
/// distinction permits an explicit repair to transition legacy geometry below
/// a newly enforced minimum directly into a valid size.
pub fn resolve_resize_request(
    constraints: GridConstraints,
    current: GridRect,
    request: GridResizeRequest,
    minimum: GridSize,
) -> Result<GridSize, GridLayoutError> {
    constraints.validate_rect(current)?;
    let requested = match request {
        GridResizeRequest::Direct(size) => size,
        GridResizeRequest::Keyboard { axis, step } => {
            let delta = match step {
                GridResizeStep::Decrease => -1,
                GridResizeStep::Increase => 1,
            };
            match axis {
                GridResizeAxis::Width => GridSize::new(
                    current
                        .width
                        .checked_add(delta)
                        .ok_or(GridLayoutError::ArithmeticOverflow)?,
                    current.height,
                ),
                GridResizeAxis::Height => GridSize::new(
                    current.width,
                    current
                        .height
                        .checked_add(delta)
                        .ok_or(GridLayoutError::ArithmeticOverflow)?,
                ),
            }
        }
    };
    constraints.validate_rect_with_minimum(
        GridRect::new(
            current.row,
            current.column,
            requested.width,
            requested.height,
        ),
        minimum,
    )?;
    Ok(requested)
}

fn row_major_cmp<Id: Ord>(left: &GridPlacement<Id>, right: &GridPlacement<Id>) -> Ordering {
    (left.rect.row, left.rect.column, &left.id).cmp(&(right.rect.row, right.rect.column, &right.id))
}

fn inclusive_end(start: i32, length: i32) -> Option<i32> {
    if length < 1 {
        return None;
    }
    start.checked_add(length.checked_sub(1)?)
}

fn linear_index(
    constraints: GridConstraints,
    row: i32,
    column: i32,
) -> Result<i64, GridLayoutError> {
    if row < 1 {
        return Err(GridLayoutError::InvalidRow { row });
    }
    if column < 1 || column > constraints.columns() {
        return Err(GridLayoutError::InvalidColumn { column });
    }
    i64::from(row - 1)
        .checked_mul(i64::from(constraints.columns()))
        .and_then(|offset| offset.checked_add(i64::from(column - 1)))
        .ok_or(GridLayoutError::ArithmeticOverflow)
}

/// Stable failures produced by generic grid operations.
#[derive(Debug, Clone, Eq, PartialEq, thiserror::Error)]
pub enum GridLayoutError {
    #[error("invalid grid constraint '{name}': {value}")]
    InvalidConstraint { name: &'static str, value: i64 },
    #[error("invalid placement minimum {width}x{height}")]
    InvalidMinimum { width: i32, height: i32 },
    #[error("row must be at least 1, got {row}")]
    InvalidRow { row: i32 },
    #[error("column must be within the grid, got {column}")]
    InvalidColumn { column: i32 },
    #[error("width {width} is outside the supported range {min}..={max}")]
    WidthOutOfRange { width: i32, min: i32, max: i32 },
    #[error("height {height} is outside the supported range {min}..={max}")]
    HeightOutOfRange { height: i32, min: i32, max: i32 },
    #[error("rectangle at column {column} with width {width} exceeds {columns} columns")]
    ColumnOverflow {
        column: i32,
        width: i32,
        columns: i32,
    },
    #[error("rectangle at row {row} with height {height} exceeds {max_rows} rows")]
    RowOverflow {
        row: i32,
        height: i32,
        max_rows: i32,
    },
    #[error("placement count {count} exceeds the limit of {max}")]
    PlacementLimitExceeded { count: usize, max: usize },
    #[error("placement identifiers must be unique")]
    DuplicatePlacementId,
    #[error("placements overlap")]
    Overlap,
    #[error("placement was not found")]
    PlacementNotFound,
    #[error("no grid space remains for deterministic reflow")]
    NoSpace,
    #[error("grid arithmetic overflowed")]
    ArithmeticOverflow,
}

#[cfg(test)]
mod tests {
    use super::{
        GridConstraints, GridLayoutError, GridMoveDirection, GridMoveRequest, GridPlacement,
        GridRect, GridResizeAxis, GridResizeRequest, GridResizeStep, GridSize,
        derive_row_major_positions, reflow_movement, resolve_move_request, resolve_resize_request,
        sort_row_major, validate_resize,
    };

    const GRID: GridConstraints = GridConstraints::new(12, 240, 240, 6);

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
    fn validates_boundaries_and_every_supported_size() {
        for width in 1..=12 {
            for height in 1..=6 {
                assert!(
                    GRID.validate_rect(GridRect::new(1, 1, width, height))
                        .is_ok()
                );
            }
        }
        assert!(GRID.validate_rect(GridRect::new(240, 12, 1, 1)).is_ok());
    }

    #[test]
    fn rejects_invalid_coordinates_sizes_and_overflow() {
        assert_eq!(
            GRID.validate_rect(GridRect::new(0, 1, 1, 1)),
            Err(GridLayoutError::InvalidRow { row: 0 })
        );
        assert_eq!(
            GRID.validate_rect(GridRect::new(1, 0, 1, 1)),
            Err(GridLayoutError::InvalidColumn { column: 0 })
        );
        assert!(matches!(
            GRID.validate_rect(GridRect::new(1, 1, 0, 1)),
            Err(GridLayoutError::WidthOutOfRange { .. })
        ));
        assert!(matches!(
            GRID.validate_rect(GridRect::new(1, 1, 1, 7)),
            Err(GridLayoutError::HeightOutOfRange { .. })
        ));
        assert!(matches!(
            GRID.validate_rect(GridRect::new(1, 12, 2, 1)),
            Err(GridLayoutError::ColumnOverflow { .. })
        ));
        assert!(matches!(
            GRID.validate_rect(GridRect::new(240, 1, 1, 2)),
            Err(GridLayoutError::RowOverflow { .. })
        ));
    }

    #[test]
    fn detects_overlap_but_allows_touching_edges() {
        let touching = [placement("a", 1, 1, 6, 2), placement("b", 1, 7, 6, 2)];
        assert!(GRID.validate_layout(&touching).is_ok());

        let overlapping = [placement("a", 1, 1, 6, 2), placement("b", 2, 6, 2, 2)];
        assert_eq!(
            GRID.validate_layout(&overlapping),
            Err(GridLayoutError::Overlap)
        );
    }

    #[test]
    fn rejects_duplicate_ids_and_excess_capacity() {
        let duplicate = [placement("a", 1, 1, 1, 1), placement("a", 2, 1, 1, 1)];
        assert_eq!(
            GRID.validate_layout(&duplicate),
            Err(GridLayoutError::DuplicatePlacementId)
        );

        let too_many: Vec<_> = (0..241)
            .map(|id| GridPlacement::new(id, GridRect::new(id + 1, 1, 1, 1)))
            .collect();
        assert_eq!(
            GRID.validate_layout(&too_many),
            Err(GridLayoutError::PlacementLimitExceeded {
                count: 241,
                max: 240
            })
        );
    }

    #[test]
    fn expands_occupancy_for_valid_layouts() {
        let occupied = GRID
            .occupancy(&[placement("a", 2, 3, 2, 2)])
            .expect("valid occupancy");
        assert_eq!(
            occupied.into_iter().collect::<Vec<_>>(),
            vec![(2, 3), (2, 4), (3, 3), (3, 4)]
        );
    }

    #[test]
    fn rectangle_occupancy_can_characterize_overlapping_legacy_drafts() {
        let first = GridRect::new(1, 1, 2, 1)
            .occupied_cells()
            .expect("first rectangle");
        let second = GridRect::new(1, 2, 2, 1)
            .occupied_cells()
            .expect("second rectangle");
        let occupied = first.union(&second).copied().collect::<Vec<_>>();
        assert_eq!(occupied, vec![(1, 1), (1, 2), (1, 3)]);
    }

    #[test]
    fn sorts_and_derives_zero_based_row_major_positions() {
        let mut placements = vec![
            placement("c", 2, 1, 1, 1),
            placement("b", 1, 7, 1, 1),
            placement("a", 1, 1, 1, 1),
        ];
        assert_eq!(
            derive_row_major_positions(&placements),
            vec![("a", 0), ("b", 1), ("c", 2)]
        );
        sort_row_major(&mut placements);
        assert_eq!(
            placements
                .into_iter()
                .map(|item| item.id)
                .collect::<Vec<_>>(),
            vec!["a", "b", "c"]
        );
    }

    #[test]
    fn occupied_target_move_keeps_moved_item_and_reflows_collision() {
        let placements = [placement("a", 1, 1, 6, 1), placement("b", 1, 7, 6, 1)];
        let result = reflow_movement(GRID, &placements, &"b", 1, 1).expect("move should reflow");
        assert_eq!(
            result,
            vec![placement("b", 1, 1, 6, 1), placement("a", 1, 7, 6, 1)]
        );
    }

    #[test]
    fn movement_preserves_form_row_major_precedence_outside_ties() {
        let placements = [
            placement("wide", 1, 1, 6, 1),
            placement("moved", 2, 1, 1, 1),
        ];
        let result =
            reflow_movement(GRID, &placements, &"moved", 1, 3).expect("move should reflow");
        assert_eq!(
            result,
            vec![
                placement("wide", 1, 1, 6, 1),
                placement("moved", 1, 7, 1, 1)
            ]
        );
    }

    #[test]
    fn movement_reflow_is_deterministic_across_input_order() {
        let first = [
            placement("a", 1, 1, 12, 1),
            placement("b", 2, 1, 12, 1),
            placement("c", 3, 1, 12, 1),
        ];
        let second = [first[2].clone(), first[0].clone(), first[1].clone()];
        let expected = reflow_movement(GRID, &first, &"c", 1, 1).expect("first move");
        assert_eq!(
            reflow_movement(GRID, &second, &"c", 1, 1).expect("second move"),
            expected
        );
        assert_eq!(
            expected,
            vec![
                placement("c", 1, 1, 12, 1),
                placement("a", 2, 1, 12, 1),
                placement("b", 3, 1, 12, 1)
            ]
        );
    }

    #[test]
    fn movement_rejects_out_of_bounds_and_full_grid() {
        let placements = [placement("a", 1, 1, 1, 1)];
        assert!(matches!(
            reflow_movement(GRID, &placements, &"a", 241, 1),
            Err(GridLayoutError::RowOverflow { .. })
        ));

        let tiny = GridConstraints::new(1, 2, 2, 1);
        let full = [placement("a", 1, 1, 1, 1), placement("b", 2, 1, 1, 1)];
        assert_eq!(
            reflow_movement(tiny, &full, &"a", 2, 1),
            Err(GridLayoutError::NoSpace)
        );
    }

    #[test]
    fn movement_rejects_when_collision_reflow_would_cross_row_240() {
        let placements = [
            placement("moved", 239, 1, 12, 1),
            placement("last-row", 240, 1, 12, 1),
        ];
        assert_eq!(
            reflow_movement(GRID, &placements, &"moved", 240, 1),
            Err(GridLayoutError::NoSpace)
        );
    }

    #[test]
    fn resize_accepts_valid_size_and_rejects_collision() {
        let placements = [placement("a", 1, 1, 4, 1), placement("b", 1, 7, 6, 1)];
        assert_eq!(
            validate_resize(GRID, &placements, &"a", GridSize::new(6, 1), GridSize::ONE)
                .expect("valid resize"),
            placement("a", 1, 1, 6, 1)
        );
        assert_eq!(
            validate_resize(GRID, &placements, &"a", GridSize::new(7, 1), GridSize::ONE),
            Err(GridLayoutError::Overlap)
        );
    }

    #[test]
    fn resize_enforces_caller_minimum() {
        let placements = [placement("table", 1, 1, 6, 3)];
        assert!(matches!(
            validate_resize(
                GRID,
                &placements,
                &"table",
                GridSize::new(5, 3),
                GridSize::new(6, 3)
            ),
            Err(GridLayoutError::WidthOutOfRange { min: 6, .. })
        ));
    }

    #[test]
    fn rejects_invalid_constraint_definitions_and_minimums() {
        assert!(matches!(
            GridConstraints::new(0, 10, 10, 2).validate_definition(),
            Err(GridLayoutError::InvalidConstraint {
                name: "columns",
                ..
            })
        ));
        assert_eq!(
            GRID.validate_rect_with_minimum(GridRect::new(1, 1, 1, 1), GridSize::new(13, 1)),
            Err(GridLayoutError::InvalidMinimum {
                width: 13,
                height: 1
            })
        );
    }

    #[test]
    fn direct_and_keyboard_movement_resolve_to_the_same_geometry() {
        let current = GridRect::new(2, 2, 3, 2);
        let direct =
            resolve_move_request(GRID, current, GridMoveRequest::Direct { row: 2, column: 3 })
                .expect("direct move");
        let keyboard = resolve_move_request(
            GRID,
            current,
            GridMoveRequest::Keyboard(GridMoveDirection::Right),
        )
        .expect("keyboard move");

        assert_eq!(direct, keyboard);
        assert_eq!(direct, GridRect::new(2, 3, 3, 2));
    }

    #[test]
    fn direct_and_keyboard_resize_resolve_to_the_same_size() {
        let current = GridRect::new(2, 2, 3, 2);
        let direct = resolve_resize_request(
            GRID,
            current,
            GridResizeRequest::Direct(GridSize::new(4, 2)),
            GridSize::ONE,
        )
        .expect("direct resize");
        let keyboard = resolve_resize_request(
            GRID,
            current,
            GridResizeRequest::Keyboard {
                axis: GridResizeAxis::Width,
                step: GridResizeStep::Increase,
            },
            GridSize::ONE,
        )
        .expect("keyboard resize");

        assert_eq!(direct, keyboard);
        assert_eq!(direct, GridSize::new(4, 2));
    }

    #[test]
    fn direct_resize_can_repair_current_geometry_below_the_context_minimum() {
        let current = GridRect::new(1, 1, 12, 1);

        assert_eq!(
            resolve_resize_request(
                GRID,
                current,
                GridResizeRequest::Direct(GridSize::new(12, 4)),
                GridSize::new(6, 4),
            ),
            Ok(GridSize::new(12, 4))
        );
    }

    #[test]
    fn repair_resize_still_requires_valid_current_and_requested_geometry() {
        let minimum = GridSize::new(6, 4);

        assert!(matches!(
            resolve_resize_request(
                GRID,
                GridRect::new(1, 1, 12, 1),
                GridResizeRequest::Direct(GridSize::new(12, 3)),
                minimum,
            ),
            Err(GridLayoutError::HeightOutOfRange { min: 4, .. })
        ));
        assert!(matches!(
            resolve_resize_request(
                GRID,
                GridRect::new(0, 1, 12, 1),
                GridResizeRequest::Direct(GridSize::new(12, 4)),
                minimum,
            ),
            Err(GridLayoutError::InvalidRow { row: 0 })
        ));
    }
}
