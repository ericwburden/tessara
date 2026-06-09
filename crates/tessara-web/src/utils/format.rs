//! Text-formatting helpers that are shared across UI feature modules.
//!
//! Kept intentionally small and stable so callers can route through one boundary.

pub(crate) use crate::utils::metadata::metadata_label;
pub(crate) use crate::utils::text::{nonempty_text, sentence_label as sentence_case, text_matches};
