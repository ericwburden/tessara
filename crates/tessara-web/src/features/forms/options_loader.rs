//! Signal-aware option loaders for form create and edit pages.

mod create;
mod edit;

pub(crate) use create::load_form_create_options;
pub(crate) use edit::load_form_edit_options;
