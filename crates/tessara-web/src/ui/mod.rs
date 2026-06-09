use leptos::prelude::{AnyView, Fragment};

pub(crate) fn empty_view() -> AnyView {
    Fragment::new(Vec::<AnyView>::new()).into()
}

pub mod shell;
mod button;
mod breadcrumb;
mod data_table;
mod empty_state;
mod info_list;
pub mod dropdown;
mod page_header;
mod status_badge;
mod tabs;
mod timestamp;
mod sheet;
pub mod components;
