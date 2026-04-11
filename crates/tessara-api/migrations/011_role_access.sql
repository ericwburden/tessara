CREATE TABLE account_credentials (
    account_id uuid PRIMARY KEY REFERENCES accounts(id) ON DELETE CASCADE,
    password text NOT NULL
);

CREATE TABLE account_node_scope_assignments (
    account_id uuid NOT NULL REFERENCES accounts(id) ON DELETE CASCADE,
    node_id uuid NOT NULL REFERENCES nodes(id) ON DELETE CASCADE,
    PRIMARY KEY (account_id, node_id)
);

CREATE TABLE account_subordinate_relationships (
    parent_account_id uuid NOT NULL REFERENCES accounts(id) ON DELETE CASCADE,
    respondent_account_id uuid NOT NULL REFERENCES accounts(id) ON DELETE CASCADE,
    PRIMARY KEY (parent_account_id, respondent_account_id),
    CHECK (parent_account_id <> respondent_account_id)
);
