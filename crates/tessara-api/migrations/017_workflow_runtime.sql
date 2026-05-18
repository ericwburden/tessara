CREATE TABLE workflows (
    id uuid PRIMARY KEY DEFAULT uuid_generate_v4(),
    workflow_node_type_id uuid NOT NULL REFERENCES node_types(id) ON DELETE RESTRICT,
    name text NOT NULL,
    slug text NOT NULL UNIQUE,
    description text NOT NULL DEFAULT '',
    source text NOT NULL DEFAULT 'authored',
    source_form_id uuid REFERENCES forms(id) ON DELETE SET NULL,
    created_at timestamptz NOT NULL DEFAULT now(),
    CONSTRAINT workflows_source_check CHECK (source IN ('authored', 'generated_form'))
);

CREATE TABLE workflow_versions (
    id uuid PRIMARY KEY DEFAULT uuid_generate_v4(),
    workflow_id uuid NOT NULL REFERENCES workflows(id) ON DELETE CASCADE,
    version_label text,
    status form_version_status NOT NULL DEFAULT 'draft',
    published_at timestamptz,
    created_at timestamptz NOT NULL DEFAULT now()
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
    ADD COLUMN workflow_assignment_id uuid REFERENCES workflow_assignments(id) ON DELETE RESTRICT,
    ADD COLUMN workflow_instance_id uuid REFERENCES workflow_instances(id) ON DELETE SET NULL,
    ADD COLUMN workflow_step_instance_id uuid REFERENCES workflow_step_instances(id) ON DELETE SET NULL;

CREATE INDEX workflow_versions_workflow_idx
    ON workflow_versions (workflow_id, status, created_at);

CREATE INDEX workflow_assignments_account_idx
    ON workflow_assignments (account_id, is_active, created_at);

CREATE INDEX workflow_assignments_workflow_idx
    ON workflow_assignments (workflow_version_id, is_active, created_at);

CREATE UNIQUE INDEX workflows_generated_form_source_idx
    ON workflows (source_form_id)
    WHERE source = 'generated_form' AND source_form_id IS NOT NULL;

CREATE INDEX workflow_instances_assignment_idx
    ON workflow_instances (workflow_assignment_id, created_at);

INSERT INTO workflows (workflow_node_type_id, name, slug, description, source, source_form_id)
SELECT
    forms.scope_node_type_id,
    forms.name || ' Workflow',
    forms.slug || '-workflow',
    'Generated single-form workflow.',
    'generated_form',
    forms.id
FROM forms
WHERE forms.scope_node_type_id IS NOT NULL
ON CONFLICT (slug) DO NOTHING;

INSERT INTO workflow_versions (workflow_id, version_label, status, published_at, created_at)
SELECT
    workflows.id,
    form_versions.version_label,
    form_versions.status,
    form_versions.published_at,
    form_versions.created_at
FROM form_versions
JOIN forms ON forms.id = form_versions.form_id
JOIN workflows ON workflows.slug = forms.slug || '-workflow';

INSERT INTO workflow_steps (workflow_version_id, form_version_id, title, position)
SELECT
    workflow_versions.id,
    form_versions.id,
    COALESCE(forms.name, 'Workflow') || ' Response',
    0
FROM workflow_versions
JOIN workflows ON workflows.id = workflow_versions.workflow_id
JOIN forms ON workflows.slug = forms.slug || '-workflow'
JOIN form_versions
    ON form_versions.form_id = forms.id
   AND form_versions.version_label IS NOT DISTINCT FROM workflow_versions.version_label
   AND form_versions.status = workflow_versions.status
   AND form_versions.created_at = workflow_versions.created_at
ON CONFLICT (workflow_version_id, position) DO NOTHING;
