CREATE TABLE workflows (
    id uuid PRIMARY KEY DEFAULT uuid_generate_v4(),
    form_id uuid NOT NULL REFERENCES forms(id) ON DELETE CASCADE,
    name text NOT NULL,
    slug text NOT NULL UNIQUE,
    description text NOT NULL DEFAULT '',
    created_at timestamptz NOT NULL DEFAULT now(),
    UNIQUE (form_id)
);

CREATE TABLE workflow_versions (
    id uuid PRIMARY KEY DEFAULT uuid_generate_v4(),
    workflow_id uuid NOT NULL REFERENCES workflows(id) ON DELETE CASCADE,
    form_version_id uuid NOT NULL REFERENCES form_versions(id) ON DELETE RESTRICT,
    version_label text,
    status form_version_status NOT NULL DEFAULT 'draft',
    published_at timestamptz,
    created_at timestamptz NOT NULL DEFAULT now(),
    UNIQUE (form_version_id)
);

CREATE TABLE workflow_steps (
    id uuid PRIMARY KEY DEFAULT uuid_generate_v4(),
    workflow_version_id uuid NOT NULL REFERENCES workflow_versions(id) ON DELETE CASCADE,
    form_version_id uuid NOT NULL REFERENCES form_versions(id) ON DELETE RESTRICT,
    title text NOT NULL,
    position integer NOT NULL DEFAULT 0,
    UNIQUE (workflow_version_id, position)
);

CREATE TABLE workflow_assignments (
    id uuid PRIMARY KEY DEFAULT uuid_generate_v4(),
    workflow_version_id uuid NOT NULL REFERENCES workflow_versions(id) ON DELETE RESTRICT,
    workflow_step_id uuid NOT NULL REFERENCES workflow_steps(id) ON DELETE RESTRICT,
    node_id uuid NOT NULL REFERENCES nodes(id) ON DELETE RESTRICT,
    account_id uuid NOT NULL REFERENCES accounts(id) ON DELETE CASCADE,
    form_assignment_id uuid REFERENCES form_assignments(id) ON DELETE SET NULL,
    is_active boolean NOT NULL DEFAULT true,
    created_at timestamptz NOT NULL DEFAULT now(),
    UNIQUE (workflow_step_id, node_id, account_id)
);

CREATE TABLE workflow_instances (
    id uuid PRIMARY KEY DEFAULT uuid_generate_v4(),
    workflow_assignment_id uuid NOT NULL REFERENCES workflow_assignments(id) ON DELETE RESTRICT,
    workflow_version_id uuid NOT NULL REFERENCES workflow_versions(id) ON DELETE RESTRICT,
    node_id uuid NOT NULL REFERENCES nodes(id) ON DELETE RESTRICT,
    assignee_account_id uuid NOT NULL REFERENCES accounts(id) ON DELETE CASCADE,
    started_by_account_id uuid REFERENCES accounts(id) ON DELETE SET NULL,
    created_at timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE workflow_step_instances (
    id uuid PRIMARY KEY DEFAULT uuid_generate_v4(),
    workflow_instance_id uuid NOT NULL REFERENCES workflow_instances(id) ON DELETE CASCADE,
    workflow_step_id uuid NOT NULL REFERENCES workflow_steps(id) ON DELETE RESTRICT,
    submission_id uuid UNIQUE REFERENCES submissions(id) ON DELETE SET NULL,
    status text NOT NULL DEFAULT 'in_progress',
    started_at timestamptz NOT NULL DEFAULT now(),
    completed_at timestamptz,
    CONSTRAINT workflow_step_instances_status_check CHECK (
        status IN ('in_progress', 'completed')
    )
);

ALTER TABLE submissions
    ADD COLUMN workflow_assignment_id uuid REFERENCES workflow_assignments(id) ON DELETE SET NULL,
    ADD COLUMN workflow_instance_id uuid REFERENCES workflow_instances(id) ON DELETE SET NULL,
    ADD COLUMN workflow_step_instance_id uuid REFERENCES workflow_step_instances(id) ON DELETE SET NULL;

CREATE INDEX workflow_versions_workflow_idx
    ON workflow_versions (workflow_id, status, created_at);

CREATE INDEX workflow_assignments_account_idx
    ON workflow_assignments (account_id, is_active, created_at);

CREATE INDEX workflow_assignments_workflow_idx
    ON workflow_assignments (workflow_version_id, is_active, created_at);

CREATE INDEX workflow_instances_assignment_idx
    ON workflow_instances (workflow_assignment_id, created_at);

INSERT INTO workflows (form_id, name, slug, description)
SELECT
    forms.id,
    forms.name || ' Workflow',
    forms.slug || '-workflow',
    'Generated from the linked form for Sprint 2A runtime compatibility.'
FROM forms
ON CONFLICT (form_id) DO NOTHING;

INSERT INTO workflow_versions (workflow_id, form_version_id, version_label, status, published_at, created_at)
SELECT
    workflows.id,
    form_versions.id,
    form_versions.version_label,
    form_versions.status,
    form_versions.published_at,
    form_versions.created_at
FROM form_versions
JOIN workflows ON workflows.form_id = form_versions.form_id
ON CONFLICT (form_version_id) DO NOTHING;

INSERT INTO workflow_steps (workflow_version_id, form_version_id, title, position)
SELECT
    workflow_versions.id,
    workflow_versions.form_version_id,
    COALESCE(forms.name, 'Workflow') || ' Response',
    0
FROM workflow_versions
JOIN workflows ON workflows.id = workflow_versions.workflow_id
JOIN forms ON forms.id = workflows.form_id
ON CONFLICT (workflow_version_id, position) DO NOTHING;

INSERT INTO workflow_assignments (
    workflow_version_id,
    workflow_step_id,
    node_id,
    account_id,
    form_assignment_id,
    is_active,
    created_at
)
SELECT
    workflow_versions.id,
    workflow_steps.id,
    form_assignments.node_id,
    form_assignments.account_id,
    form_assignments.id,
    true,
    form_assignments.created_at
FROM form_assignments
JOIN workflow_versions ON workflow_versions.form_version_id = form_assignments.form_version_id
JOIN workflow_steps
    ON workflow_steps.workflow_version_id = workflow_versions.id
   AND workflow_steps.position = 0
WHERE form_assignments.account_id IS NOT NULL
ON CONFLICT (workflow_step_id, node_id, account_id) DO UPDATE
SET
    form_assignment_id = COALESCE(workflow_assignments.form_assignment_id, EXCLUDED.form_assignment_id),
    is_active = true;

WITH submission_assignment_links AS (
    SELECT
        submissions.id AS submission_id,
        submissions.node_id,
        submissions.status,
        submissions.created_at,
        submissions.submitted_at,
        workflow_versions.id AS workflow_version_id,
        workflow_steps.id AS workflow_step_id,
        workflow_assignments.id AS workflow_assignment_id,
        workflow_assignments.account_id
    FROM submissions
    JOIN form_assignments ON form_assignments.id = submissions.assignment_id
    JOIN workflow_versions ON workflow_versions.form_version_id = submissions.form_version_id
    JOIN workflow_steps
        ON workflow_steps.workflow_version_id = workflow_versions.id
       AND workflow_steps.position = 0
    JOIN workflow_assignments
        ON workflow_assignments.workflow_step_id = workflow_steps.id
       AND workflow_assignments.node_id = submissions.node_id
       AND workflow_assignments.account_id = form_assignments.account_id
    WHERE submissions.workflow_assignment_id IS NULL
),
seeded_instances AS (
    INSERT INTO workflow_instances (
        workflow_assignment_id,
        workflow_version_id,
        node_id,
        assignee_account_id,
        started_by_account_id,
        created_at
    )
    SELECT
        submission_assignment_links.workflow_assignment_id,
        submission_assignment_links.workflow_version_id,
        submission_assignment_links.node_id,
        submission_assignment_links.account_id,
        submission_assignment_links.account_id,
        submission_assignment_links.created_at
    FROM submission_assignment_links
    RETURNING id, workflow_assignment_id, workflow_version_id, node_id, assignee_account_id, created_at
),
instance_links AS (
    SELECT
        submission_assignment_links.submission_id,
        seeded_instances.id AS workflow_instance_id,
        submission_assignment_links.workflow_assignment_id,
        submission_assignment_links.workflow_step_id,
        submission_assignment_links.status,
        submission_assignment_links.created_at,
        submission_assignment_links.submitted_at
    FROM submission_assignment_links
    JOIN seeded_instances
        ON seeded_instances.workflow_assignment_id = submission_assignment_links.workflow_assignment_id
       AND seeded_instances.node_id = submission_assignment_links.node_id
       AND seeded_instances.assignee_account_id = submission_assignment_links.account_id
       AND seeded_instances.created_at = submission_assignment_links.created_at
)
INSERT INTO workflow_step_instances (
    workflow_instance_id,
    workflow_step_id,
    submission_id,
    status,
    started_at,
    completed_at
)
SELECT
    instance_links.workflow_instance_id,
    instance_links.workflow_step_id,
    instance_links.submission_id,
    CASE
        WHEN instance_links.status = 'submitted'::submission_status THEN 'completed'
        ELSE 'in_progress'
    END,
    instance_links.created_at,
    CASE
        WHEN instance_links.status = 'submitted'::submission_status THEN instance_links.submitted_at
        ELSE NULL
    END
FROM instance_links
ON CONFLICT (submission_id) DO NOTHING;

WITH submission_assignment_links AS (
    SELECT
        submissions.id AS submission_id,
        submissions.node_id,
        submissions.created_at,
        workflow_versions.id AS workflow_version_id,
        workflow_steps.id AS workflow_step_id,
        workflow_assignments.id AS workflow_assignment_id,
        workflow_assignments.account_id
    FROM submissions
    JOIN form_assignments ON form_assignments.id = submissions.assignment_id
    JOIN workflow_versions ON workflow_versions.form_version_id = submissions.form_version_id
    JOIN workflow_steps
        ON workflow_steps.workflow_version_id = workflow_versions.id
       AND workflow_steps.position = 0
    JOIN workflow_assignments
        ON workflow_assignments.workflow_step_id = workflow_steps.id
       AND workflow_assignments.node_id = submissions.node_id
       AND workflow_assignments.account_id = form_assignments.account_id
    WHERE submissions.workflow_assignment_id IS NULL
)
UPDATE submissions AS target
SET
    workflow_assignment_id = workflow_assignments.id,
    workflow_instance_id = workflow_instances.id,
    workflow_step_instance_id = workflow_step_instances.id
FROM submission_assignment_links
JOIN workflow_assignments
    ON workflow_assignments.id = submission_assignment_links.workflow_assignment_id
JOIN workflow_instances
    ON workflow_instances.workflow_assignment_id = workflow_assignments.id
   AND workflow_instances.node_id = submission_assignment_links.node_id
   AND workflow_instances.assignee_account_id = submission_assignment_links.account_id
   AND workflow_instances.created_at = submission_assignment_links.created_at
JOIN workflow_step_instances
    ON workflow_step_instances.workflow_instance_id = workflow_instances.id
   AND workflow_step_instances.submission_id = submission_assignment_links.submission_id
WHERE target.id = submission_assignment_links.submission_id
  AND target.workflow_assignment_id IS NULL;
