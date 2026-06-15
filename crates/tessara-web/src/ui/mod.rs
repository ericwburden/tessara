//! Shared UI component registry.
//!
//! Re-export reusable, domain-neutral components from here; feature-specific views and workflows should stay under `features`.

use leptos::prelude::{AnyView, Fragment};

/// Returns an empty Leptos view for conditional branches that render nothing.
pub(crate) fn empty_view() -> AnyView {
    Fragment::new(Vec::<AnyView>::new()).into()
}

mod breadcrumb;
mod button;
mod data_table;
pub mod dropdown;
mod empty_state;
mod info_list;
mod page_header;
mod segmented_toggle;
pub mod shell;
mod status_badge;
mod table_filter;
mod table_pagination;
mod tabs;
mod timestamp;

pub(crate) use breadcrumb::*;
pub(crate) use button::*;
pub(crate) use data_table::{DataTable, SearchableDataTable};
pub(crate) use dropdown::DropdownMenu;
pub(crate) use empty_state::*;
pub(crate) use info_list::{InfoListTable, InfoRow};
pub(crate) use page_header::*;
pub(crate) use segmented_toggle::{SegmentedToggle, SegmentedToggleOption};
pub(crate) use shell::AppShell;
pub(crate) use status_badge::*;
pub(crate) use table_filter::TableFilterHeader;
pub(crate) use table_pagination::TablePaginationFooter;
pub(crate) use tabs::{Tabs, TabsContent, TabsList, TabsTrigger};
pub(crate) use timestamp::*;
