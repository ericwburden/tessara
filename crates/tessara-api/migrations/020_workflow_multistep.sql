ALTER TABLE workflow_instances
    ADD COLUMN status text NOT NULL DEFAULT 'in_progress';

ALTER TABLE workflow_versions
    DROP CONSTRAINT IF EXISTS workflow_versions_form_version_id_key;

ALTER TABLE workflows
    DROP CONSTRAINT IF EXISTS workflows_form_id_key;

ALTER TABLE workflow_instances
    ADD CONSTRAINT workflow_instances_status_check CHECK (
        status IN ('in_progress', 'completed')
    );

CREATE INDEX workflow_instances_status_idx
    ON workflow_instances (workflow_version_id, node_id, assignee_account_id, status);

CREATE INDEX workflow_step_instances_instance_status_idx
    ON workflow_step_instances (workflow_instance_id, status, started_at);
