//! General text-formatting helpers shared across feature modules.
//!
//! Keep presentation-neutral string formatting here when the logic is not tied to a specific domain or UI component.

pub(crate) use crate::utils::metadata::metadata_label;
pub(crate) use crate::utils::text::{nonempty_text, sentence_label as sentence_case, text_matches};
