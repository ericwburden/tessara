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
pub(crate) struct ReportRouteParams {
    pub report_id: String,
}

#[derive(PartialEq, Clone, Debug)]
pub(crate) struct DashboardRouteParams {
    pub dashboard_id: String,
}

#[derive(PartialEq, Clone, Debug)]
pub(crate) struct DatasetRouteParams {
    pub dataset_id: String,
}

#[derive(PartialEq, Clone, Debug)]
pub(crate) struct ComponentRouteParams {
    pub component_ref: String,
}

#[derive(PartialEq, Clone, Debug)]
pub(crate) struct AccountRouteParams {
    pub account_id: String,
}

#[derive(PartialEq, Clone, Debug)]
pub(crate) struct RoleRouteParams {
    pub role_id: String,
}

#[derive(PartialEq, Clone, Debug)]
pub(crate) struct NodeTypeRouteParams {
    pub node_type_id: String,
}

fn require_map_value(map: &ParamsMap, key: &'static str) -> Result<String, ParamsError> {
    map.get(key)
        .ok_or_else(|| ParamsError::MissingParam(key.to_string()))
}

impl Params for NodeRouteParams {
    fn from_map(map: &ParamsMap) -> Result<Self, ParamsError> {
        Ok(Self {
            node_id: require_map_value(map, "node_id")?,
        })
    }
}

impl Params for FormRouteParams {
    fn from_map(map: &ParamsMap) -> Result<Self, ParamsError> {
        Ok(Self {
            form_id: require_map_value(map, "form_id")?,
        })
    }
}

impl Params for WorkflowRouteParams {
    fn from_map(map: &ParamsMap) -> Result<Self, ParamsError> {
        Ok(Self {
            workflow_id: require_map_value(map, "workflow_id")?,
        })
    }
}

impl Params for SubmissionRouteParams {
    fn from_map(map: &ParamsMap) -> Result<Self, ParamsError> {
        Ok(Self {
            submission_id: require_map_value(map, "submission_id")?,
        })
    }
}

impl Params for ReportRouteParams {
    fn from_map(map: &ParamsMap) -> Result<Self, ParamsError> {
        Ok(Self {
            report_id: require_map_value(map, "report_id")?,
        })
    }
}

impl Params for DashboardRouteParams {
    fn from_map(map: &ParamsMap) -> Result<Self, ParamsError> {
        Ok(Self {
            dashboard_id: require_map_value(map, "dashboard_id")?,
        })
    }
}

impl Params for DatasetRouteParams {
    fn from_map(map: &ParamsMap) -> Result<Self, ParamsError> {
        Ok(Self {
            dataset_id: require_map_value(map, "dataset_id")?,
        })
    }
}

impl Params for ComponentRouteParams {
    fn from_map(map: &ParamsMap) -> Result<Self, ParamsError> {
        Ok(Self {
            component_ref: require_map_value(map, "component_ref")?,
        })
    }
}

impl Params for AccountRouteParams {
    fn from_map(map: &ParamsMap) -> Result<Self, ParamsError> {
        Ok(Self {
            account_id: require_map_value(map, "account_id")?,
        })
    }
}

impl Params for RoleRouteParams {
    fn from_map(map: &ParamsMap) -> Result<Self, ParamsError> {
        Ok(Self {
            role_id: require_map_value(map, "role_id")?,
        })
    }
}

impl Params for NodeTypeRouteParams {
    fn from_map(map: &ParamsMap) -> Result<Self, ParamsError> {
        Ok(Self {
            node_type_id: require_map_value(map, "node_type_id")?,
        })
    }
}

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
