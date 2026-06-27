//! Shared, domain-neutral UI primitives for Tessara web feature crates.

mod breadcrumb;
mod combobox;
mod data_table;
mod draggable_panel_list;
mod empty_state;
mod page_header;
mod segmented_toggle;
mod table_pagination;

use leptos::prelude::{AnyView, Fragment};

pub use breadcrumb::{
    Breadcrumb, BreadcrumbItem, BreadcrumbLink, BreadcrumbPage, BreadcrumbSeparator,
};
pub use combobox::{Combobox, ComboboxOption};
pub use data_table::DataTable;
pub use draggable_panel_list::{
    DraggablePanelList, DraggablePanelListAnchor, DraggablePanelListDraggable,
    DraggablePanelListDropZone, DraggablePanelListItem, DraggablePanelListMove,
};
pub use empty_state::EmptyState;
pub use page_header::PageHeader;
pub use segmented_toggle::{SegmentedToggle, SegmentedToggleOption};
pub use table_pagination::TablePaginationFooter;

/// Returns an empty Leptos view for conditional branches that render nothing.
pub fn empty_view() -> AnyView {
    Fragment::new(Vec::<AnyView>::new()).into()
}
