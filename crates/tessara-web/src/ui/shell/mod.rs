//! Application shell component registry.
//!
//! Keep layout, navigation, sidebar, mobile navigation, and top-app-bar components together as the reusable frame around feature pages.

mod app_shell;
mod mobile_nav;
mod nav;
mod sidebar;
mod top_app_bar;

pub use app_shell::AppShell;
pub(crate) use mobile_nav::MobileNav;
pub use sidebar::Sidebar;
pub use top_app_bar::{IconButton, TopAppBar};
