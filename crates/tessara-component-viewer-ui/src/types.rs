//! Canonical exact-version Component execution response types.
#![cfg_attr(not(feature = "hydrate"), allow(dead_code))]

pub use tessara_components_contract::{
    ComponentRenderResponse, ComponentStatValue, ComponentVisualPoint,
    ComponentVisualResponse as ComponentVisual, ComponentVisualSlice,
};
pub(crate) use tessara_components_contract::{
    ComponentTableColumn, ComponentTableResponse as ComponentTable,
};
#[cfg(test)]
pub(crate) use tessara_components_contract::{ComponentTablePagination, ComponentTableRow};
