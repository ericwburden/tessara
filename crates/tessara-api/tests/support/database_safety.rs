pub const DISPOSABLE_DATABASE_NAME_TOKENS: &[&str] = &[
    "test", "tests", "testing", "upgrade", "clone", "rollback", "sprint6a",
];

/// Returns whether a database name contains an explicit disposable-use token.
///
/// Tokens are bounded by the start/end of the name or a non-alphanumeric
/// character. This deliberately accepts names such as
/// `tessara_sprint6a_upgrade_test` while rejecting accidental substrings such
/// as `latest` and `contest`.
pub fn is_disposable_database_name(database_name: &str) -> bool {
    let tokens = database_name
        .split(|character: char| !character.is_alphanumeric())
        .filter(|token| !token.is_empty())
        .map(str::to_ascii_lowercase)
        .collect::<Vec<_>>();

    tokens.iter().any(|token| {
        DISPOSABLE_DATABASE_NAME_TOKENS
            .iter()
            .any(|marker| token == marker)
    }) || tokens
        .windows(2)
        .any(|pair| pair[0] == "sprint" && pair[1] == "6a")
}
