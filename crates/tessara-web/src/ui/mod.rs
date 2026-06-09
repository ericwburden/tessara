use leptos::prelude::{AnyView, Fragment};

pub(crate) fn empty_view() -> AnyView {
    Fragment::new(Vec::<AnyView>::new()).into()
}

mod breadcrumb;
mod button;
pub mod components;
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
