//! Owns the types::route_params module behavior.

use leptos::prelude::With;
use leptos_router::hooks::use_params;
use leptos_router::params::{Params, ParamsError, ParamsMap};

#[derive(PartialEq, Clone, Debug)]
pub(crate) struct NodeRouteParams {
    pub node_id: String,
}

#[derive(PartialEq, Clone, Debug)]
pub(crate) struct FormRouteParams {
    pub form_id: String,
}

#[derive(PartialEq, Clone, Debug)]
pub(crate) struct WorkflowRouteParams {
    pub workflow_id: String,
}

#[derive(PartialEq, Clone, Debug)]
pub(crate) struct SubmissionRouteParams {
    pub submission_id: String,
}

#[derive(PartialEq, Clone, Debug)]
pub(crate) struct DatasetRouteParams {
    pub dataset_id: String,
}

#[derive(PartialEq, Clone, Debug)]
pub(crate) struct AccountRouteParams {
    pub account_id: String,
}

/// Handles the require map value behavior.
fn require_map_value(map: &ParamsMap, key: &'static str) -> Result<String, ParamsError> {
    map.get(key)
        .ok_or_else(|| ParamsError::MissingParam(key.to_string()))
}

impl Params for NodeRouteParams {
    /// Handles the from map behavior.
    fn from_map(map: &ParamsMap) -> Result<Self, ParamsError> {
        Ok(Self {
            node_id: require_map_value(map, "node_id")?,
        })
    }
}

impl Params for FormRouteParams {
    /// Handles the from map behavior.
    fn from_map(map: &ParamsMap) -> Result<Self, ParamsError> {
        Ok(Self {
            form_id: require_map_value(map, "form_id")?,
        })
    }
}

impl Params for WorkflowRouteParams {
    /// Handles the from map behavior.
    fn from_map(map: &ParamsMap) -> Result<Self, ParamsError> {
        Ok(Self {
            workflow_id: require_map_value(map, "workflow_id")?,
        })
    }
}

impl Params for SubmissionRouteParams {
    /// Handles the from map behavior.
    fn from_map(map: &ParamsMap) -> Result<Self, ParamsError> {
        Ok(Self {
            submission_id: require_map_value(map, "submission_id")?,
        })
    }
}

impl Params for DatasetRouteParams {
    /// Handles the from map behavior.
    fn from_map(map: &ParamsMap) -> Result<Self, ParamsError> {
        Ok(Self {
            dataset_id: require_map_value(map, "dataset_id")?,
        })
    }
}

impl Params for AccountRouteParams {
    /// Handles the from map behavior.
    fn from_map(map: &ParamsMap) -> Result<Self, ParamsError> {
        Ok(Self {
            account_id: require_map_value(map, "account_id")?,
        })
    }
}

/// Handles the require route params behavior.
pub(crate) fn require_route_params<T>() -> T
where
    T: Params + Clone + PartialEq + Send + Sync + 'static,
{
    use_params::<T>().with(|params| {
        params
            .clone()
            .unwrap_or_else(|error| panic!("failed to parse current route params: {error:?}"))
    })
}
