use leptos::prelude::*;
use leptos_router::hooks::use_params;
use leptos_router::params::Params;

#[derive(Params, PartialEq, Clone, Debug)]
pub struct NodeRouteParams {
    pub node_id: String,
}

#[derive(Params, PartialEq, Clone, Debug)]
pub struct FormRouteParams {
    pub form_id: String,
}

#[derive(Params, PartialEq, Clone, Debug)]
pub struct SubmissionRouteParams {
    pub submission_id: String,
}

#[derive(Params, PartialEq, Clone, Debug)]
pub struct ReportRouteParams {
    pub report_id: String,
}

#[derive(Params, PartialEq, Clone, Debug)]
pub struct DashboardRouteParams {
    pub dashboard_id: String,
}

#[derive(Params, PartialEq, Clone, Debug)]
pub struct AccountRouteParams {
    pub account_id: String,
}

#[derive(Params, PartialEq, Clone, Debug)]
pub struct RoleRouteParams {
    pub role_id: String,
}

pub fn require_route_params<T>() -> T
where
    T: Params + Clone + PartialEq + Send + Sync + 'static,
{
    use_params::<T>().with(|params| {
        params
            .clone()
            .unwrap_or_else(|error| panic!("failed to parse current route params: {error:?}"))
    })
}
