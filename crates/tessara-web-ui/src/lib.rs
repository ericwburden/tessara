//! Shared, domain-neutral UI primitives for Tessara web feature crates.

mod breadcrumb;
mod button;
mod combobox;
mod data_table;
mod draggable_panel_list;
mod dropdown;
mod empty_state;
mod info_list;
mod page_header;
mod searchable_data_table;
mod segmented_toggle;
mod skeleton;
mod table_filter;
mod table_pagination;
mod tabs;
mod timestamp;

use leptos::prelude::{AnyView, Fragment};

pub use breadcrumb::{
    Breadcrumb, BreadcrumbItem, BreadcrumbLink, BreadcrumbPage, BreadcrumbSeparator,
};
pub use button::Button;
pub use combobox::{Combobox, ComboboxOption};
pub use data_table::{
    DataTable, InteractiveDataTable, InteractiveTableColumn, InteractiveTableRow,
};
pub use draggable_panel_list::{
    DraggablePanelList, DraggablePanelListAnchor, DraggablePanelListDraggable,
    DraggablePanelListDropZone, DraggablePanelListItem, DraggablePanelListMove,
};
pub use dropdown::DropdownMenu;
pub use empty_state::EmptyState;
pub use info_list::{InfoListTable, InfoRow};
pub use page_header::PageHeader;
pub use searchable_data_table::SearchableDataTable;
pub use segmented_toggle::{SegmentedToggle, SegmentedToggleOption};
pub use skeleton::Skeleton;
pub use table_filter::TableFilterHeader;
pub use table_pagination::TablePaginationFooter;
pub use tabs::{Tabs, TabsContent, TabsList, TabsTrigger};
pub use timestamp::Timestamp;

/// Returns an empty Leptos view for conditional branches that render nothing.
pub fn empty_view() -> AnyView {
    Fragment::new(Vec::<AnyView>::new()).into()
}
