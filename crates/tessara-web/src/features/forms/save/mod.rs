//! Form save orchestration entrypoints.

mod create;
mod update;

pub(crate) use create::submit_create_form;
pub(crate) use update::submit_update_form;
