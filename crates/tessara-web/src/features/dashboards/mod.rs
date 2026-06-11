//! Public boundary for the Dashboards feature.
//!
//! Re-export only the pages, types, and helpers other modules need; keep Dashboards-specific implementation details in child modules.

mod pages;

pub(crate) use pages::{
    DashboardsDetailPage, DashboardsEditPage, DashboardsNewPage, DashboardsPage,
};
