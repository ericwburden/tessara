mod api;
mod assignments;
mod detail;
mod editor;
mod list;
mod pages;
pub(crate) mod submission;
mod types;

// Keep this module self-contained as workflow pages are now owned by
// the workflows feature domain.
pub(crate) use crate::features::organization::*;

pub(crate) use assignments::*;
pub(crate) use detail::*;
pub(crate) use editor::*;
pub(crate) use list::*;
