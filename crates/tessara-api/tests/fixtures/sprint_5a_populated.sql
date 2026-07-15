-- Representative populated Sprint 5A installation. Every durable fixture
-- identity uses the 60000000 prefix so the upgrade test can prove exact row
-- preservation independently from Sprint 6A control-plane rows. The
-- capability catalog and built-in role memberships below reproduce the exact
-- sets produced by Sprint 5A startup, before Sprint 6A is allowed to replace
-- the named built-in memberships with its versioned seed contract. Their
-- canonical historical identity is
-- sprint-5a-role-capabilities-v1+sha256.7725e889996a.

INSERT INTO accounts (id, email, display_name, is_active, created_at)
VALUES
    ('60000000-0000-0000-0000-000000000001', 'admin@tessara.local', 'Tessara Admin', true, '2026-07-01 12:00:00+00'),
    ('60000000-0000-0000-0000-000000000002', 'existing.user@tessara.local', 'Existing User', true, '2026-07-01 12:01:00+00');

INSERT INTO account_credentials (
    account_id,
    password_hash,
    password_scheme,
    password_updated_at
)
VALUES (
    '60000000-0000-0000-0000-000000000002',
    '$argon2id$v=19$m=65536,t=3,p=4$c29tZXNhbHQ$RdescudvJCsgt3ub+b+dWRWJTmaaJObG',
    'argon2id',
    '2026-07-01 12:01:30+00'
);

INSERT INTO roles (id, name, description, created_at)
VALUES
    ('60000000-0000-0000-0000-000000000101', 'admin', 'Existing administrator role', '2026-07-01 12:02:00+00'),
    ('60000000-0000-0000-0000-000000000102', 'operator', 'Existing operator role', '2026-07-01 12:03:00+00'),
    ('60000000-0000-0000-0000-000000000103', 'respondent', 'Existing respondent role', '2026-07-01 12:04:00+00'),
    ('60000000-0000-0000-0000-000000000104', 'existing-custom-role', 'User-managed role that seed updates must not change', '2026-07-01 12:04:30+00');

INSERT INTO capabilities (id, key, description)
VALUES
    ('60000000-0000-0000-0000-000000000201', 'admin:all', 'Full administration access'),
    ('60000000-0000-0000-0000-000000000202', 'hierarchy:read', 'Browse runtime hierarchy records'),
    ('60000000-0000-0000-0000-000000000203', 'hierarchy:manage', 'Manage hierarchy configuration and nodes'),
    ('60000000-0000-0000-0000-000000000204', 'forms:read', 'Browse top-level form records'),
    ('60000000-0000-0000-0000-000000000205', 'forms:manage', 'Manage form definitions and versions'),
    ('60000000-0000-0000-0000-000000000206', 'workflows:read', 'Browse workflow definitions and assignments'),
    ('60000000-0000-0000-0000-000000000207', 'workflows:manage', 'Manage workflow definitions and assignments'),
    ('60000000-0000-0000-0000-000000000208', 'submissions:read_own', 'Read own and delegated response work'),
    ('60000000-0000-0000-0000-000000000209', 'submissions:respond', 'Start and complete assigned response work'),
    ('60000000-0000-0000-0000-000000000210', 'submissions:manage', 'Manage submissions by hierarchy scope'),
    ('60000000-0000-0000-0000-000000000211', 'analytics:refresh', 'Refresh analytics projections'),
    ('60000000-0000-0000-0000-000000000212', 'operations:view', 'Inspect workflow assignment and dataset readiness status'),
    ('60000000-0000-0000-0000-000000000213', 'datasets:manage', 'Manage dataset definitions'),
    ('60000000-0000-0000-0000-000000000214', 'datasets:read', 'Inspect dataset definitions'),
    ('60000000-0000-0000-0000-000000000215', 'datasets:read_restricted', 'Read restricted dataset rows when dataset visibility allows access'),
    ('60000000-0000-0000-0000-000000000216', 'datasets:read_confidential', 'Read confidential and restricted dataset rows when dataset visibility allows access'),
    ('60000000-0000-0000-0000-000000000217', 'components:manage', 'Manage component definitions'),
    ('60000000-0000-0000-0000-000000000218', 'components:read', 'Inspect component definitions'),
    ('60000000-0000-0000-0000-000000000219', 'dashboards:manage', 'Manage dashboard definitions'),
    ('60000000-0000-0000-0000-000000000220', 'dashboards:read', 'Inspect dashboard definitions');

INSERT INTO role_capabilities (role_id, capability_id)
VALUES
    -- Sprint 5A assigned every catalog capability to the built-in admin role.
    ('60000000-0000-0000-0000-000000000101', '60000000-0000-0000-0000-000000000201'),
    ('60000000-0000-0000-0000-000000000101', '60000000-0000-0000-0000-000000000202'),
    ('60000000-0000-0000-0000-000000000101', '60000000-0000-0000-0000-000000000203'),
    ('60000000-0000-0000-0000-000000000101', '60000000-0000-0000-0000-000000000204'),
    ('60000000-0000-0000-0000-000000000101', '60000000-0000-0000-0000-000000000205'),
    ('60000000-0000-0000-0000-000000000101', '60000000-0000-0000-0000-000000000206'),
    ('60000000-0000-0000-0000-000000000101', '60000000-0000-0000-0000-000000000207'),
    ('60000000-0000-0000-0000-000000000101', '60000000-0000-0000-0000-000000000208'),
    ('60000000-0000-0000-0000-000000000101', '60000000-0000-0000-0000-000000000209'),
    ('60000000-0000-0000-0000-000000000101', '60000000-0000-0000-0000-000000000210'),
    ('60000000-0000-0000-0000-000000000101', '60000000-0000-0000-0000-000000000211'),
    ('60000000-0000-0000-0000-000000000101', '60000000-0000-0000-0000-000000000212'),
    ('60000000-0000-0000-0000-000000000101', '60000000-0000-0000-0000-000000000213'),
    ('60000000-0000-0000-0000-000000000101', '60000000-0000-0000-0000-000000000214'),
    ('60000000-0000-0000-0000-000000000101', '60000000-0000-0000-0000-000000000215'),
    ('60000000-0000-0000-0000-000000000101', '60000000-0000-0000-0000-000000000216'),
    ('60000000-0000-0000-0000-000000000101', '60000000-0000-0000-0000-000000000217'),
    ('60000000-0000-0000-0000-000000000101', '60000000-0000-0000-0000-000000000218'),
    ('60000000-0000-0000-0000-000000000101', '60000000-0000-0000-0000-000000000219'),
    ('60000000-0000-0000-0000-000000000101', '60000000-0000-0000-0000-000000000220'),
    -- Sprint 5A operator and respondent memberships were the exact sets below.
    ('60000000-0000-0000-0000-000000000102', '60000000-0000-0000-0000-000000000202'),
    ('60000000-0000-0000-0000-000000000102', '60000000-0000-0000-0000-000000000204'),
    ('60000000-0000-0000-0000-000000000102', '60000000-0000-0000-0000-000000000206'),
    ('60000000-0000-0000-0000-000000000102', '60000000-0000-0000-0000-000000000207'),
    ('60000000-0000-0000-0000-000000000102', '60000000-0000-0000-0000-000000000209'),
    ('60000000-0000-0000-0000-000000000102', '60000000-0000-0000-0000-000000000210'),
    ('60000000-0000-0000-0000-000000000102', '60000000-0000-0000-0000-000000000212'),
    ('60000000-0000-0000-0000-000000000102', '60000000-0000-0000-0000-000000000214'),
    ('60000000-0000-0000-0000-000000000102', '60000000-0000-0000-0000-000000000218'),
    ('60000000-0000-0000-0000-000000000102', '60000000-0000-0000-0000-000000000220'),
    ('60000000-0000-0000-0000-000000000103', '60000000-0000-0000-0000-000000000208'),
    ('60000000-0000-0000-0000-000000000103', '60000000-0000-0000-0000-000000000209'),
    -- User-managed membership is an invariant, not replaceable seed data.
    ('60000000-0000-0000-0000-000000000104', '60000000-0000-0000-0000-000000000204');

INSERT INTO auth_sessions (token, account_id, created_at, expires_at, last_seen_at)
VALUES (
    '60000000-0000-0000-0000-000000000301',
    '60000000-0000-0000-0000-000000000002',
    '2026-07-01 12:05:00+00',
    '2099-07-01 12:05:00+00',
    '2026-07-01 12:05:00+00'
);

INSERT INTO node_types (id, name, slug, plural_label, description, created_at)
VALUES
    (
        '60000000-0000-0000-0000-000000000400',
        'Region',
        'sprint-6a-upgrade-region',
        'Regions',
        'Existing parent hierarchy type',
        '2026-07-01 12:05:30+00'
    ),
    (
        '60000000-0000-0000-0000-000000000401',
        'Facility',
        'sprint-6a-upgrade-facility',
        'Facilities',
        'Existing hierarchy type',
        '2026-07-01 12:06:00+00'
    );

INSERT INTO node_type_relationships (parent_node_type_id, child_node_type_id)
VALUES (
    '60000000-0000-0000-0000-000000000400',
    '60000000-0000-0000-0000-000000000401'
);

INSERT INTO node_metadata_field_definitions (
    id,
    node_type_id,
    key,
    label,
    field_type,
    required,
    created_at
)
VALUES (
    '60000000-0000-0000-0000-000000000404',
    '60000000-0000-0000-0000-000000000401',
    'facility_code',
    'Facility Code',
    'text',
    true,
    '2026-07-01 12:06:30+00'
);

INSERT INTO nodes (id, node_type_id, parent_node_id, name, created_at)
VALUES
    (
        '60000000-0000-0000-0000-000000000403',
        '60000000-0000-0000-0000-000000000400',
        NULL,
        'Existing Region',
        '2026-07-01 12:06:45+00'
    ),
    (
        '60000000-0000-0000-0000-000000000402',
        '60000000-0000-0000-0000-000000000401',
        '60000000-0000-0000-0000-000000000403',
        'Existing Facility',
        '2026-07-01 12:07:00+00'
    );

INSERT INTO role_assignments (id, account_id, role_id, node_id, created_at)
VALUES
    (
        '60000000-0000-0000-0000-000000000501',
        '60000000-0000-0000-0000-000000000001',
        '60000000-0000-0000-0000-000000000101',
        NULL,
        '2026-07-01 12:08:00+00'
    ),
    (
        '60000000-0000-0000-0000-000000000502',
        '60000000-0000-0000-0000-000000000002',
        '60000000-0000-0000-0000-000000000102',
        '60000000-0000-0000-0000-000000000402',
        '2026-07-01 12:09:00+00'
    ),
    (
        '60000000-0000-0000-0000-000000000503',
        '60000000-0000-0000-0000-000000000002',
        '60000000-0000-0000-0000-000000000104',
        '60000000-0000-0000-0000-000000000402',
        '2026-07-01 12:09:30+00'
    );

INSERT INTO account_delegations (
    delegator_account_id,
    delegate_account_id,
    created_at
)
VALUES (
    '60000000-0000-0000-0000-000000000001',
    '60000000-0000-0000-0000-000000000002',
    '2026-07-01 12:09:40+00'
);

INSERT INTO node_metadata_values (node_id, field_definition_id, value)
VALUES (
    '60000000-0000-0000-0000-000000000402',
    '60000000-0000-0000-0000-000000000404',
    '"FAC-001"'::jsonb
);

INSERT INTO forms (id, name, slug, scope_node_type_id, created_at)
VALUES (
    '60000000-0000-0000-0000-000000000601',
    'Existing Upgrade Form',
    'sprint-6a-upgrade-form',
    '60000000-0000-0000-0000-000000000401',
    '2026-07-01 12:10:00+00'
);

INSERT INTO form_scope_nodes (form_id, node_id, created_at)
VALUES (
    '60000000-0000-0000-0000-000000000601',
    '60000000-0000-0000-0000-000000000402',
    '2026-07-01 12:11:00+00'
);

INSERT INTO compatibility_groups (id, form_id, name, created_at)
VALUES (
    '60000000-0000-0000-0000-000000000602',
    '60000000-0000-0000-0000-000000000601',
    'Existing major line',
    '2026-07-01 12:12:00+00'
);

INSERT INTO form_versions (
    id,
    form_id,
    compatibility_group_id,
    version_label,
    status,
    version_major,
    version_minor,
    version_patch,
    semantic_bump,
    started_new_major_line,
    published_at,
    created_at
)
VALUES (
    '60000000-0000-0000-0000-000000000603',
    '60000000-0000-0000-0000-000000000601',
    '60000000-0000-0000-0000-000000000602',
    '1.0.0',
    'published',
    1,
    0,
    0,
    'major',
    true,
    '2026-07-01 12:13:00+00',
    '2026-07-01 12:13:00+00'
);

INSERT INTO form_sections (id, form_version_id, title, description, position)
VALUES (
    '60000000-0000-0000-0000-000000000604',
    '60000000-0000-0000-0000-000000000603',
    'Existing Section',
    'Existing form section',
    0
);

INSERT INTO form_fields (
    field_id,
    form_version_id,
    section_id,
    key,
    label,
    field_type,
    required,
    position,
    grid_row,
    grid_column,
    grid_width,
    grid_height
)
VALUES
    (
        '60000000-0000-0000-0000-000000000605',
        '60000000-0000-0000-0000-000000000603',
        '60000000-0000-0000-0000-000000000604',
        'existing_value',
        'Existing Value',
        'text',
        true,
        0,
        1,
        1,
        12,
        2
    ),
    (
        '60000000-0000-0000-0000-000000000606',
        '60000000-0000-0000-0000-000000000603',
        '60000000-0000-0000-0000-000000000604',
        'existing_choices',
        'Existing Choices',
        'multi_choice',
        false,
        1,
        2,
        1,
        12,
        2
    );

INSERT INTO choice_lists (id, form_version_id, name, import_key)
VALUES (
    '60000000-0000-0000-0000-000000000607',
    '60000000-0000-0000-0000-000000000603',
    'Existing Choices',
    'existing_choices'
);

INSERT INTO choice_list_items (
    id,
    choice_list_id,
    value,
    label,
    import_key,
    position
)
VALUES
    (
        '60000000-0000-0000-0000-000000000608',
        '60000000-0000-0000-0000-000000000607',
        'alpha',
        'Alpha',
        'alpha',
        0
    ),
    (
        '60000000-0000-0000-0000-000000000609',
        '60000000-0000-0000-0000-000000000607',
        'beta',
        'Beta',
        'beta',
        1
    );

INSERT INTO workflows (
    id,
    workflow_node_type_id,
    name,
    slug,
    description,
    source,
    created_at
)
VALUES (
    '60000000-0000-0000-0000-000000000701',
    '60000000-0000-0000-0000-000000000401',
    'Existing Workflow',
    'sprint-6a-upgrade-workflow',
    'Existing workflow definition',
    'authored',
    '2026-07-01 12:14:00+00'
);

INSERT INTO workflow_available_nodes (workflow_id, node_id, created_at)
VALUES (
    '60000000-0000-0000-0000-000000000701',
    '60000000-0000-0000-0000-000000000402',
    '2026-07-01 12:15:00+00'
);

INSERT INTO workflow_versions (id, workflow_id, version_label, status, published_at, created_at)
VALUES (
    '60000000-0000-0000-0000-000000000702',
    '60000000-0000-0000-0000-000000000701',
    '1.0.0',
    'published',
    '2026-07-01 12:16:00+00',
    '2026-07-01 12:16:00+00'
);

INSERT INTO workflow_steps (id, workflow_version_id, form_version_id, title, position)
VALUES (
    '60000000-0000-0000-0000-000000000703',
    '60000000-0000-0000-0000-000000000702',
    '60000000-0000-0000-0000-000000000603',
    'Existing Step',
    0
);

INSERT INTO workflow_assignments (
    id,
    workflow_version_id,
    workflow_step_id,
    node_id,
    account_id,
    assigned_by_account_id,
    is_active,
    created_at
)
VALUES (
    '60000000-0000-0000-0000-000000000704',
    '60000000-0000-0000-0000-000000000702',
    '60000000-0000-0000-0000-000000000703',
    '60000000-0000-0000-0000-000000000402',
    '60000000-0000-0000-0000-000000000002',
    '60000000-0000-0000-0000-000000000001',
    true,
    '2026-07-01 12:17:00+00'
);

INSERT INTO workflow_instances (
    id,
    workflow_assignment_id,
    workflow_version_id,
    node_id,
    assignee_account_id,
    started_by_account_id,
    status,
    created_at
)
VALUES (
    '60000000-0000-0000-0000-000000000705',
    '60000000-0000-0000-0000-000000000704',
    '60000000-0000-0000-0000-000000000702',
    '60000000-0000-0000-0000-000000000402',
    '60000000-0000-0000-0000-000000000002',
    '60000000-0000-0000-0000-000000000002',
    'in_progress',
    '2026-07-01 12:18:00+00'
);

INSERT INTO submissions (
    id,
    form_version_id,
    node_id,
    workflow_assignment_id,
    workflow_instance_id,
    status,
    created_at
)
VALUES (
    '60000000-0000-0000-0000-000000000706',
    '60000000-0000-0000-0000-000000000603',
    '60000000-0000-0000-0000-000000000402',
    '60000000-0000-0000-0000-000000000704',
    '60000000-0000-0000-0000-000000000705',
    'draft',
    '2026-07-01 12:19:00+00'
);

INSERT INTO workflow_step_instances (
    id,
    workflow_instance_id,
    workflow_step_id,
    submission_id,
    status,
    started_at
)
VALUES (
    '60000000-0000-0000-0000-000000000707',
    '60000000-0000-0000-0000-000000000705',
    '60000000-0000-0000-0000-000000000703',
    '60000000-0000-0000-0000-000000000706',
    'in_progress',
    '2026-07-01 12:20:00+00'
);

UPDATE submissions
SET workflow_step_instance_id = '60000000-0000-0000-0000-000000000707'
WHERE id = '60000000-0000-0000-0000-000000000706';

INSERT INTO submission_values (submission_id, form_version_id, field_id, value)
VALUES (
    '60000000-0000-0000-0000-000000000706',
    '60000000-0000-0000-0000-000000000603',
    '60000000-0000-0000-0000-000000000605',
    '"preserved"'::jsonb
);

INSERT INTO submission_value_multi (submission_id, form_version_id, field_id, value)
VALUES
    (
        '60000000-0000-0000-0000-000000000706',
        '60000000-0000-0000-0000-000000000603',
        '60000000-0000-0000-0000-000000000606',
        'alpha'
    ),
    (
        '60000000-0000-0000-0000-000000000706',
        '60000000-0000-0000-0000-000000000603',
        '60000000-0000-0000-0000-000000000606',
        'beta'
    );

INSERT INTO submission_audit_events (id, submission_id, event_type, account_id, created_at)
VALUES (
    '60000000-0000-0000-0000-000000000708',
    '60000000-0000-0000-0000-000000000706',
    'created',
    '60000000-0000-0000-0000-000000000002',
    '2026-07-01 12:21:00+00'
);

INSERT INTO datasets (id, name, slug, grain, created_at)
VALUES (
    '60000000-0000-0000-0000-000000000801',
    'Existing Dataset',
    'sprint-6a-upgrade-dataset',
    'submission',
    '2026-07-01 12:22:00+00'
);

INSERT INTO dataset_scope_nodes (dataset_id, node_id, created_at)
VALUES (
    '60000000-0000-0000-0000-000000000801',
    '60000000-0000-0000-0000-000000000402',
    '2026-07-01 12:23:00+00'
);

INSERT INTO dataset_tags (dataset_id, tag, created_at)
VALUES
    (
        '60000000-0000-0000-0000-000000000801',
        'existing',
        '2026-07-01 12:23:15+00'
    ),
    (
        '60000000-0000-0000-0000-000000000801',
        'upgrade-proof',
        '2026-07-01 12:23:30+00'
    );

INSERT INTO dataset_revisions (
    id,
    dataset_id,
    version_number,
    version_label,
    version_major,
    version_minor,
    version_patch,
    semantic_bump,
    started_new_major_line,
    status,
    initial_source,
    operations,
    published_at,
    created_at
)
VALUES (
    '60000000-0000-0000-0000-000000000802',
    '60000000-0000-0000-0000-000000000801',
    1,
    '1.0.0',
    1,
    0,
    0,
    'major',
    true,
    'published',
    '{"kind":"form","form_id":"60000000-0000-0000-0000-000000000601","form_version_id":"60000000-0000-0000-0000-000000000603"}'::jsonb,
    '[]'::jsonb,
    '2026-07-01 12:24:00+00',
    '2026-07-01 12:24:00+00'
);

INSERT INTO dataset_major_materializations (
    dataset_id,
    version_major,
    materialized_schema,
    materialized_table,
    materialized_row_count,
    materialized_at,
    rebuild_status,
    created_at,
    updated_at
)
VALUES (
    '60000000-0000-0000-0000-000000000801',
    1,
    NULL,
    NULL,
    NULL,
    NULL,
    'pending',
    '2026-07-01 12:24:15+00',
    '2026-07-01 12:24:15+00'
);

INSERT INTO dataset_sources (
    id,
    dataset_id,
    source_alias,
    form_id,
    form_version_id,
    position,
    created_at
)
VALUES (
    '60000000-0000-0000-0000-000000000803',
    '60000000-0000-0000-0000-000000000801',
    'existing_form',
    '60000000-0000-0000-0000-000000000601',
    '60000000-0000-0000-0000-000000000603',
    0,
    '2026-07-01 12:25:00+00'
);

INSERT INTO dataset_fields (
    id,
    dataset_id,
    key,
    label,
    source_alias,
    source_field_key,
    source_field_id,
    field_type,
    position,
    created_at
)
VALUES (
    '60000000-0000-0000-0000-000000000804',
    '60000000-0000-0000-0000-000000000801',
    'existing_value',
    'Existing Value',
    'existing_form',
    'existing_value',
    '60000000-0000-0000-0000-000000000605',
    'text',
    0,
    '2026-07-01 12:26:00+00'
);

INSERT INTO components (id, name, slug, description, created_at)
VALUES (
    '60000000-0000-0000-0000-000000000901',
    'Existing Component',
    'sprint-6a-upgrade-component',
    'Existing component definition',
    '2026-07-01 12:27:00+00'
);

INSERT INTO component_versions (
    id,
    component_id,
    dataset_id,
    dataset_version_major,
    binding_mode,
    component_type,
    version_number,
    version_label,
    version_note,
    status,
    config,
    published_at,
    created_at
)
VALUES (
    '60000000-0000-0000-0000-000000000902',
    '60000000-0000-0000-0000-000000000901',
    '60000000-0000-0000-0000-000000000801',
    1,
    'major_line',
    'table',
    1,
    '1.0.0',
    'Existing component version',
    'published',
    '{"schema_version":1,"columns":["existing_value"]}'::jsonb,
    '2026-07-01 12:28:00+00',
    '2026-07-01 12:28:00+00'
);

INSERT INTO dashboards (id, name, description, created_at)
VALUES (
    '60000000-0000-0000-0000-000000001001',
    'Existing Dashboard',
    'Existing dashboard definition',
    '2026-07-01 12:29:00+00'
);

INSERT INTO dashboard_scope_nodes (dashboard_id, node_id, created_at)
VALUES (
    '60000000-0000-0000-0000-000000001001',
    '60000000-0000-0000-0000-000000000402',
    '2026-07-01 12:30:00+00'
);

INSERT INTO dashboard_components (
    id,
    dashboard_id,
    component_version_id,
    position,
    config
)
VALUES (
    '60000000-0000-0000-0000-000000001002',
    '60000000-0000-0000-0000-000000001001',
    '60000000-0000-0000-0000-000000000902',
    0,
    '{"schema_version":1,"grid_row":1,"grid_column":1,"grid_width":12,"grid_height":4,"title":"Existing"}'::jsonb
);
