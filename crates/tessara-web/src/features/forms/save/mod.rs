//! Form save orchestration entrypoints.

#[cfg(feature = "hydrate")]
mod api;
mod create;
#[cfg(feature = "hydrate")]
mod create_structure;
#[cfg(feature = "hydrate")]
mod drafts;
#[cfg(feature = "hydrate")]
mod payloads;
#[cfg(feature = "hydrate")]
mod slugs;
#[cfg(feature = "hydrate")]
mod structure;
mod update;

pub(in crate::features::forms) use create::submit_create_form;
pub(in crate::features::forms) use update::submit_update_form;
