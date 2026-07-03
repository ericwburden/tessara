//! Permission helpers for Datasets feature screens.

use super::types::SessionAccount;

/// Returns whether the account can manage dataset definitions.
pub(crate) fn can_manage_datasets(account: &SessionAccount) -> bool {
    account
        .capabilities
        .iter()
        .any(|capability| capability == "admin:all")
}
