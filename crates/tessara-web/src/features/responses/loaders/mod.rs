//! Signal-aware loaders for the Responses feature.
//!
//! Keep page loading state and response-specific fallback behavior here; endpoint transport belongs in `api`.

mod detail;
mod edit;
mod list;
mod start;

pub(crate) use detail::load_submission_detail;
pub(crate) use edit::load_submission_edit_context;
pub(crate) use list::load_submissions;
pub(crate) use start::load_response_start_options;
