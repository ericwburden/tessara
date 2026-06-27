//! Restriction-tier SQL helpers.
//!
//! Dataset row tiers are ordered by sensitivity:
//! `confidential > restricted > internal > public`.

use crate::auth;

use super::{ValidatedRestrictionPolicy, boolean_expression_sql, quote_identifier};

pub(super) fn restriction_policy_tier_sql(
    restriction_policy: &ValidatedRestrictionPolicy,
) -> String {
    let internal_predicate = restriction_policy
        .internal_field_key
        .as_deref()
        .map(boolean_tier_predicate_sql)
        .unwrap_or_else(|| "FALSE".to_string());
    let restricted_predicate = restriction_policy
        .restricted_field_key
        .as_deref()
        .map(boolean_tier_predicate_sql)
        .unwrap_or_else(|| "FALSE".to_string());
    let confidential_predicate = restriction_policy
        .confidential_field_key
        .as_deref()
        .map(boolean_tier_predicate_sql)
        .unwrap_or_else(|| "FALSE".to_string());

    format!(
        r#"CASE
                    WHEN {confidential_predicate} THEN 'confidential'
                    WHEN {restricted_predicate} THEN 'restricted'
                    WHEN {internal_predicate} THEN 'internal'
                    ELSE 'public'
                END"#
    )
}

pub(super) fn effective_restriction_tier_sql(
    restriction_policy: &ValidatedRestrictionPolicy,
) -> String {
    greatest_restriction_tier_sql(&[
        quote_identifier("__restriction_tier"),
        restriction_policy_tier_sql(restriction_policy),
    ])
}

pub(super) fn greatest_restriction_tier_sql(expressions: &[String]) -> String {
    let rank_args = expressions
        .iter()
        .map(|expression| restriction_tier_rank_sql(expression))
        .collect::<Vec<_>>()
        .join(", ");
    format!(
        r#"CASE GREATEST({rank_args})
                    WHEN 3 THEN 'confidential'
                    WHEN 2 THEN 'restricted'
                    WHEN 1 THEN 'internal'
                    ELSE 'public'
                END"#
    )
}

pub(super) fn max_restriction_tier_sql(expression: &str) -> String {
    format!(
        r#"CASE COALESCE(MAX({}), 0)
                    WHEN 3 THEN 'confidential'
                    WHEN 2 THEN 'restricted'
                    WHEN 1 THEN 'internal'
                    ELSE 'public'
                END"#,
        restriction_tier_rank_sql(expression)
    )
}

pub(super) fn tier_access_predicate(account: &auth::AccountContext) -> &'static str {
    if account.has_capability("admin:all") || account.has_capability("datasets:read_confidential") {
        "TRUE"
    } else if account.has_capability("datasets:read_restricted") {
        "COALESCE(\"__restriction_tier\", 'public') IN ('public', 'internal', 'restricted')"
    } else {
        "COALESCE(\"__restriction_tier\", 'public') IN ('public', 'internal')"
    }
}

fn boolean_tier_predicate_sql(field_key: &str) -> String {
    boolean_expression_sql(&quote_identifier(field_key))
}

fn restriction_tier_rank_sql(expression: &str) -> String {
    format!(
        r#"CASE {expression}
                    WHEN 'confidential' THEN 3
                    WHEN 'restricted' THEN 2
                    WHEN 'internal' THEN 1
                    ELSE 0
                END"#
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    fn account_without_tier_capability() -> auth::AccountContext {
        auth::AccountContext {
            account_id: Uuid::nil(),
            email: "test@example.com".into(),
            display_name: "Test User".into(),
            is_active: true,
            roles: Vec::new(),
            capabilities: Vec::new(),
            capability_scopes: Vec::new(),
            scope_nodes: Vec::new(),
            delegations: Vec::new(),
        }
    }

    #[test]
    fn tier_access_predicate_defaults_to_public_and_internal_rows() {
        assert_eq!(
            tier_access_predicate(&account_without_tier_capability()),
            "COALESCE(\"__restriction_tier\", 'public') IN ('public', 'internal')"
        );
    }
}
