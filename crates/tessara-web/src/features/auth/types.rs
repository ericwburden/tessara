//! Owns the features::auth::types module behavior.

use serde::Deserialize;

#[derive(Deserialize)]
pub struct SessionStateResponse {
    pub authenticated: bool,
    pub account: Option<ShellAccountContext>,
}

#[derive(Clone, Deserialize)]
pub struct ShellAccountContext {
    pub email: String,
    pub display_name: String,
    pub capabilities: Vec<String>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ShellAccountSummary {
    pub email: String,
    pub display_name: String,
    pub capabilities: Vec<String>,
}

impl From<ShellAccountContext> for ShellAccountSummary {
    /// Handles the from behavior.
    fn from(context: ShellAccountContext) -> Self {
        Self {
            email: context.email,
            display_name: context.display_name,
            capabilities: context.capabilities,
        }
    }
}
