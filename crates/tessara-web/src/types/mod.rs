//! Type-level compatibility surface for route params and other shared frontend
//! DTO-like value types used across feature modules.

pub(crate) mod route_params;

pub(crate) use route_params::{
    AccountRouteParams, ComponentRouteParams, DashboardRouteParams, DatasetRouteParams,
    FormRouteParams, NodeRouteParams, NodeTypeRouteParams, ReportRouteParams, RoleRouteParams,
    SubmissionRouteParams, WorkflowRouteParams,
};

pub(crate) mod ids;
pub(crate) mod pagination;
