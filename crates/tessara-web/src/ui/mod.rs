//! Shared UI component registry.
//!
//! Re-export reusable, domain-neutral components from here; feature-specific views and workflows should stay under `features`.

use leptos::prelude::{AnyView, Fragment};

/// Returns an empty Leptos view for conditional branches that render nothing.
pub(crate) fn empty_view() -> AnyView {
    Fragment::new(Vec::<AnyView>::new()).into()
}

pub mod dropdown;
pub mod shell;
mod status_badge;

pub(crate) use dropdown::DropdownMenu;
pub(crate) use shell::AppShell;
pub(crate) use status_badge::*;
pub(crate) use tessara_web_ui::{
    Breadcrumb, BreadcrumbItem, BreadcrumbLink, BreadcrumbPage, BreadcrumbSeparator, Button,
    DataTable, EmptyState, InfoListTable, InfoRow, PageHeader, SearchableDataTable,
    TableFilterHeader, TablePaginationFooter, Tabs, TabsContent, TabsList, TabsTrigger, Timestamp,
};
