use leptos::prelude::{AnyView, Fragment};

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
mod sheet;
pub mod shell;
mod status_badge;
mod tabs;
mod timestamp;

pub(crate) use breadcrumb::*;
pub(crate) use button::*;
pub(crate) use data_table::{DataTable, SearchableDataTable};
pub(crate) use dropdown::DropdownMenu;
pub(crate) use empty_state::*;
pub(crate) use info_list::{InfoListTable, InfoRow};
pub(crate) use page_header::*;
pub(crate) use shell::AppShell;
pub(crate) use status_badge::*;
pub(crate) use tabs::{Tabs, TabsContent, TabsList, TabsTrigger};
pub(crate) use timestamp::*;
