ALTER TABLE IF EXISTS account_subordinate_relationships
    RENAME TO account_delegations;

ALTER TABLE IF EXISTS account_delegations
    RENAME COLUMN parent_account_id TO delegator_account_id;

ALTER TABLE IF EXISTS account_delegations
    RENAME COLUMN respondent_account_id TO delegate_account_id;
