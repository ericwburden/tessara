//! Editor helper functions for Datasets feature screens.

use super::super::types::*;

pub(crate) fn confirm_action(message: &str) -> bool {
    #[cfg(feature = "hydrate")]
    {
        web_sys::window()
            .and_then(|window| window.confirm_with_message(message).ok())
            .unwrap_or(false)
    }

    #[cfg(not(feature = "hydrate"))]
    {
        let _ = message;
        true
    }
}

pub(crate) fn version_label(version: &DatasetFormVersionOption) -> String {
    version
        .version_label
        .clone()
        .unwrap_or_else(|| format!("Major {}", version.version_major.unwrap_or(1)))
}
