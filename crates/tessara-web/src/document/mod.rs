//! Owns the document module behavior.

pub(crate) mod assets;
pub(crate) mod html;
pub(crate) mod theme_bootstrap;

pub(crate) use assets::{document_head_tags, svg_asset};
pub(crate) use html::render_native_app_document;
pub(crate) use theme_bootstrap::{bootstrap_script, stylesheet_links};
