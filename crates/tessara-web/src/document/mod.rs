//! Native document rendering boundary.
//!
//! Group document shell HTML, asset tags, and theme bootstrap helpers here; feature screens should only receive the completed document through crate-level helpers.

pub(crate) mod assets;
pub(crate) mod html;
pub(crate) mod theme_bootstrap;

#[cfg(any(feature = "ssr", test))]
pub(crate) use assets::static_asset;
pub(crate) use assets::{document_head_tags, svg_asset};
pub(crate) use html::{
    render_native_app_document, render_native_app_document_with_dashboard_bootstrap,
    render_native_app_document_with_module_management_and_shell_navigation_bootstrap,
    render_native_app_document_with_module_management_bootstrap,
    render_native_app_document_with_scoped_records_bootstrap,
};
pub(crate) use theme_bootstrap::{bootstrap_script, stylesheet_links};
