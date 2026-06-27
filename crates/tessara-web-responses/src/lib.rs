//! Public boundary for the Responses feature.
//!
//! Re-export only route content components; keep Responses-specific
//! implementation details in child modules.

mod actions;
mod api;
mod components;
mod detail;
pub(crate) mod display;
mod edit;
mod filtering;
mod http;
mod list;
mod loaders;
mod metadata;
mod pagination;
mod start;
mod status;
mod text;
pub(crate) mod types;
mod url;
pub(crate) mod value_collection;

pub(crate) use display::workflow_revision_label_from_option;

pub use detail::ResponseDetailContent;
pub use edit::ResponseEditContent;
pub use list::ResponsesIndexContent;
pub use start::ResponseStartContent;
