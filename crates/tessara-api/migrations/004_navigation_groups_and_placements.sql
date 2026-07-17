-- Sprint 6A-UI: replace the effective reorder-band policy with generic groups
-- and one complete placement collection. Descriptor group/band columns remain
-- immutable catalog provenance and the legacy policy rows remain rollback data;
-- neither is an effective navigation source after this migration.

CREATE TABLE navigation_groups (
    installation_id uuid NOT NULL
        REFERENCES navigation_policies(installation_id) ON DELETE RESTRICT,
    group_id text NOT NULL,
    label text NOT NULL,
    display_order integer NOT NULL CHECK (display_order >= 0),
    owner text NOT NULL,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now(),
    PRIMARY KEY (installation_id, group_id),
    UNIQUE (installation_id, display_order),
    CONSTRAINT navigation_groups_owner_chk CHECK (owner IN ('core', 'custom')),
    CONSTRAINT navigation_groups_identity_chk CHECK (
        (owner = 'core' AND group_id IN ('core.main', 'core.admin'))
        OR
        (
            owner = 'custom'
            AND group_id ~ '^custom\.[0-9a-f]{8}-[0-9a-f]{4}-4[0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$'
        )
    ),
    CONSTRAINT navigation_groups_label_chk CHECK (
        label = btrim(label)
        AND char_length(label) BETWEEN 1 AND 64
        AND label !~ '[[:cntrl:]]'
    )
);

CREATE UNIQUE INDEX navigation_groups_label_unique
    ON navigation_groups (installation_id, lower(label));

CREATE TABLE navigation_destination_placements (
    installation_id uuid NOT NULL
        REFERENCES navigation_policies(installation_id) ON DELETE RESTRICT,
    destination_id text NOT NULL,
    group_id text NOT NULL,
    visible boolean NOT NULL,
    display_order integer NOT NULL CHECK (display_order >= 0),
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now(),
    PRIMARY KEY (installation_id, destination_id),
    UNIQUE (installation_id, group_id, display_order),
    CONSTRAINT navigation_destination_placements_group_fk
        FOREIGN KEY (installation_id, group_id)
        REFERENCES navigation_groups(installation_id, group_id) ON DELETE RESTRICT,
    CONSTRAINT navigation_destination_placements_identity_chk CHECK (
        destination_id ~ '^[a-z0-9]+([.:_-][a-z0-9]+)*$'
        AND destination_id !~ '[.:_-]{2}'
    )
);

-- A fresh database has no installation at migration time; startup catalog
-- reconciliation seeds the same collection. A populated Sprint 6A database
-- receives the approved deterministic two-group layout atomically here.
INSERT INTO navigation_groups (installation_id, group_id, label, display_order, owner)
SELECT installation_id, 'core.main', 'Main', 0, 'core'
FROM navigation_policies
UNION ALL
SELECT installation_id, 'core.admin', 'Admin', 1, 'core'
FROM navigation_policies;

INSERT INTO navigation_destination_placements (
    installation_id,
    destination_id,
    group_id,
    visible,
    display_order
)
SELECT installation_id, destination_id, group_id, visible, display_order
FROM (
    SELECT installation_id, 'core.home'::text AS destination_id,
           'core.main'::text AS group_id, true AS visible, 0 AS display_order
    FROM navigation_policies
    UNION ALL
    SELECT installation_id, 'core.organization', 'core.main', true, 1
    FROM navigation_policies
    UNION ALL
    SELECT entries.installation_id, entries.contribution_id, 'core.main', entries.visible,
           2 + entries.policy_order
    FROM navigation_policy_entries AS entries
    JOIN module_navigation_contributions AS contributions
      ON contributions.contribution_id = entries.contribution_id
    WHERE contributions.reorder_band = 'main_between_organization_and_operations'
    UNION ALL
    SELECT installation_id, 'core.operations', 'core.main', true, 5
    FROM navigation_policies
    UNION ALL
    SELECT entries.installation_id, entries.contribution_id, 'core.main', entries.visible, 6
    FROM navigation_policy_entries AS entries
    WHERE entries.contribution_id = 'tessara.datasets.navigation'
    UNION ALL
    SELECT entries.installation_id, entries.contribution_id, 'core.main', entries.visible,
           7 + entries.policy_order
    FROM navigation_policy_entries AS entries
    JOIN module_navigation_contributions AS contributions
      ON contributions.contribution_id = entries.contribution_id
    WHERE contributions.reorder_band = 'main_after_operations'
    UNION ALL
    SELECT installation_id, 'core.admin.users', 'core.admin', true, 0
    FROM navigation_policies
    UNION ALL
    SELECT installation_id, 'core.admin.roles', 'core.admin', true, 1
    FROM navigation_policies
    UNION ALL
    SELECT installation_id, 'core.admin.node_types', 'core.admin', true, 2
    FROM navigation_policies
    UNION ALL
    SELECT installation_id, 'core.admin.modules', 'core.admin', true, 3
    FROM navigation_policies
) AS approved_layout;

INSERT INTO core_control_plane_audit_events (
    installation_id,
    event_type,
    actor_kind,
    actor_account_id,
    correlation_id,
    payload
)
SELECT
    policies.installation_id,
    'navigation_policy_schema_migrated',
    'system',
    NULL,
    uuid_generate_v4(),
    jsonb_build_object(
        'from_schema_version', 1,
        'to_schema_version', 2,
        'old_policy_fingerprint', md5(COALESCE((
            SELECT string_agg(
                concat_ws(':', contribution_id, visible, policy_order),
                '|' ORDER BY contribution_id
            )
            FROM navigation_policy_entries
            WHERE installation_id = policies.installation_id
        ), '')),
        'new_policy_fingerprint', md5(COALESCE((
            SELECT string_agg(
                concat_ws(':', destination_id, group_id, visible, display_order),
                '|' ORDER BY destination_id
            )
            FROM navigation_destination_placements
            WHERE installation_id = policies.installation_id
        ), ''))
    )
FROM navigation_policies AS policies;
