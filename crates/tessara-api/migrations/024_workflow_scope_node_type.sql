ALTER TABLE workflows
    ADD COLUMN IF NOT EXISTS scope_node_type_id uuid REFERENCES node_types(id) ON DELETE RESTRICT;

UPDATE workflows
SET scope_node_type_id = forms.scope_node_type_id
FROM forms
WHERE workflows.form_id = forms.id
  AND workflows.scope_node_type_id IS NULL;

