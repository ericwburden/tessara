//! Shared UI component registry.
//!
//! Re-export reusable, domain-neutral components from here; feature-specific views and workflows should stay under `features`.

use leptos::prelude::{AnyView, Fragment};

/// Returns an empty Leptos view for conditional branches that render nothing.
pub(crate) fn empty_view() -> AnyView {
    Fragment::new(Vec::<AnyView>::new()).into()
}

mod button;
mod data_table;
pub mod dropdown;
mod info_list;
pub mod shell;
mod status_badge;
mod table_filter;
mod tabs;
mod timestamp;

pub(crate) use button::*;
pub(crate) use data_table::SearchableDataTable;
pub(crate) use dropdown::DropdownMenu;
pub(crate) use info_list::{InfoListTable, InfoRow};
pub(crate) use shell::AppShell;
pub(crate) use status_badge::*;
pub(crate) use table_filter::TableFilterHeader;
pub(crate) use tabs::{Tabs, TabsContent, TabsList, TabsTrigger};
pub(crate) use tessara_web_ui::{
    Breadcrumb, BreadcrumbItem, BreadcrumbLink, BreadcrumbPage, BreadcrumbSeparator, DataTable,
    EmptyState, PageHeader, TablePaginationFooter,
};
pub(crate) use timestamp::*;
