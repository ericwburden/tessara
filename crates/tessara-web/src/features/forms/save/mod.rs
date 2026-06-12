//! Form save orchestration entrypoints.

mod create;
#[cfg(feature = "hydrate")]
mod drafts;
#[cfg(feature = "hydrate")]
mod payloads;
#[cfg(feature = "hydrate")]
mod structure;
mod update;

pub(crate) use create::submit_create_form;
pub(crate) use update::submit_update_form;
