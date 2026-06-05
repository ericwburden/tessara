//! Generated workflow orchestration entry points.
//!
//! Forms call these functions when publishing scoped form versions. The SQL
//! bodies still live in `handlers` for now because they share transaction
//! helpers with workflow authoring; this facade gives that concern a stable
//! module boundary for the next extraction pass.

pub use super::handlers::{
    ensure_workflow_assignment_for_form_version, ensure_workflow_for_published_form_version_tx,
};
