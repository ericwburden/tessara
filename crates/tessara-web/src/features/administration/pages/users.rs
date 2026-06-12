//! User-management administration pages.

mod detail;
mod edit;
mod list;

pub(crate) use detail::{AdministrationUserAccessPage, AdministrationUserDetailPage};
pub(crate) use edit::AdministrationUserEditPage;
pub(crate) use list::AdministrationUsersPage;
