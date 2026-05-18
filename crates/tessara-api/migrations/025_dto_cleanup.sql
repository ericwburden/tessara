ALTER TABLE workflows
    ADD COLUMN IF NOT EXISTS workflow_node_type_id uuid REFERENCES node_types(id) ON DELETE RESTRICT;

ALTER TABLE workflows
    ADD COLUMN IF NOT EXISTS source text NOT NULL DEFAULT 'authored',
    ADD COLUMN IF NOT EXISTS source_form_id uuid REFERENCES forms(id) ON DELETE SET NULL;

ALTER TABLE workflows
    DROP CONSTRAINT IF EXISTS workflows_source_check,
    ADD CONSTRAINT workflows_source_check CHECK (source IN ('authored', 'generated_form'));

CREATE UNIQUE INDEX IF NOT EXISTS workflows_generated_form_source_idx
    ON workflows (source_form_id)
    WHERE source = 'generated_form' AND source_form_id IS NOT NULL;

DO $$
BEGIN
  IF EXISTS (
      SELECT 1
      FROM information_schema.columns
      WHERE table_schema = current_schema()
        AND table_name = 'workflows'
        AND column_name = 'scope_node_type_id'
  ) THEN
    EXECUTE '
      UPDATE workflows
      SET workflow_node_type_id = COALESCE(workflows.workflow_node_type_id, workflows.scope_node_type_id)
      WHERE workflows.workflow_node_type_id IS NULL
    ';
  END IF;
END $$;

DELETE FROM workflows
WHERE workflow_node_type_id IS NULL;

DO $$
BEGIN
  IF EXISTS (
      SELECT 1
      FROM information_schema.columns
      WHERE table_schema = current_schema()
        AND table_name = 'workflows'
        AND column_name = 'form_id'
  ) THEN
    EXECUTE '
      UPDATE workflows
      SET workflow_node_type_id = forms.scope_node_type_id
      FROM forms
      WHERE workflows.form_id = forms.id
        AND workflows.workflow_node_type_id IS NULL
        AND forms.scope_node_type_id IS NOT NULL
    ';
    EXECUTE '
      UPDATE workflows
      SET source = ''generated_form'',
          source_form_id = forms.id
      FROM forms
      WHERE workflows.form_id = forms.id
        AND workflows.source_form_id IS NULL
    ';
  END IF;
END $$;

DO $$
BEGIN
  IF EXISTS (
      SELECT 1
      FROM information_schema.columns
      WHERE table_schema = current_schema()
        AND table_name = 'workflow_versions'
        AND column_name = 'form_version_id'
  ) THEN
    EXECUTE '
      INSERT INTO workflow_steps (workflow_version_id, form_version_id, title, position)
      SELECT
          workflow_versions.id,
          workflow_versions.form_version_id,
          COALESCE(forms.name, ''Workflow'') || '' Response'',
          0
      FROM workflow_versions
      JOIN form_versions ON form_versions.id = workflow_versions.form_version_id
      JOIN forms ON forms.id = form_versions.form_id
      WHERE workflow_versions.form_version_id IS NOT NULL
      ON CONFLICT (workflow_version_id, position) DO NOTHING
    ';
  END IF;
END $$;

DO $$
BEGIN
  IF EXISTS (
      SELECT 1
      FROM information_schema.columns
      WHERE table_schema = current_schema()
        AND table_name = 'workflow_assignments'
        AND column_name = 'form_assignment_id'
  ) THEN
    EXECUTE '
      UPDATE workflow_assignments
      SET node_id = form_assignments.node_id,
          account_id = form_assignments.account_id
      FROM form_assignments
      WHERE workflow_assignments.form_assignment_id = form_assignments.id
        AND form_assignments.account_id IS NOT NULL
    ';
  END IF;
END $$;

DO $$
BEGIN
  IF EXISTS (
      SELECT 1
      FROM information_schema.tables
      WHERE table_schema = current_schema()
        AND table_name = 'form_assignments'
  ) THEN
    EXECUTE '
      INSERT INTO workflow_assignments (
          workflow_version_id,
          workflow_step_id,
          node_id,
          account_id,
          is_active,
          created_at
      )
      SELECT
          workflow_versions.id,
          workflow_steps.id,
          form_assignments.node_id,
          form_assignments.account_id,
          true,
          form_assignments.created_at
      FROM form_assignments
      JOIN workflow_steps
          ON workflow_steps.form_version_id = form_assignments.form_version_id
         AND workflow_steps.position = 0
      JOIN workflow_versions ON workflow_versions.id = workflow_steps.workflow_version_id
      WHERE form_assignments.account_id IS NOT NULL
      ON CONFLICT (workflow_step_id, node_id, account_id) DO UPDATE
      SET is_active = true
    ';
  END IF;
END $$;

DO $$
BEGIN
  IF EXISTS (
      SELECT 1
      FROM information_schema.columns
      WHERE table_schema = current_schema()
        AND table_name = 'submissions'
        AND column_name = 'assignment_id'
  ) THEN
    EXECUTE '
      UPDATE submissions
      SET workflow_assignment_id = workflow_assignments.id
      FROM form_assignments
      JOIN workflow_steps
          ON workflow_steps.form_version_id = form_assignments.form_version_id
         AND workflow_steps.position = 0
      JOIN workflow_assignments
          ON workflow_assignments.workflow_step_id = workflow_steps.id
         AND workflow_assignments.node_id = form_assignments.node_id
         AND workflow_assignments.account_id = form_assignments.account_id
      WHERE submissions.assignment_id = form_assignments.id
        AND submissions.workflow_assignment_id IS NULL
    ';
  END IF;
END $$;

DELETE FROM submissions
WHERE workflow_assignment_id IS NULL;

ALTER TABLE workflow_assignments
    DROP COLUMN IF EXISTS form_assignment_id;

ALTER TABLE submissions
    DROP COLUMN IF EXISTS assignment_id,
    ALTER COLUMN workflow_assignment_id SET NOT NULL;

DROP TABLE IF EXISTS form_assignments;

ALTER TABLE workflows
    ALTER COLUMN workflow_node_type_id SET NOT NULL,
    DROP CONSTRAINT IF EXISTS workflows_form_id_key,
    DROP COLUMN IF EXISTS scope_node_type_id,
    DROP COLUMN IF EXISTS form_id;

ALTER TABLE workflow_versions
    DROP CONSTRAINT IF EXISTS workflow_versions_form_version_id_key,
    DROP COLUMN IF EXISTS form_version_id;

ALTER TABLE form_sections
    DROP CONSTRAINT IF EXISTS form_sections_column_count_check,
    DROP COLUMN IF EXISTS column_count;
