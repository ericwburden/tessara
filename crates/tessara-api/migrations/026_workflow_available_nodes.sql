CREATE TABLE IF NOT EXISTS workflow_available_nodes (
    workflow_id uuid NOT NULL REFERENCES workflows(id) ON DELETE CASCADE,
    node_id uuid NOT NULL REFERENCES nodes(id) ON DELETE CASCADE,
    created_at timestamptz NOT NULL DEFAULT now(),
    PRIMARY KEY (workflow_id, node_id)
);

INSERT INTO workflow_available_nodes (workflow_id, node_id)
SELECT workflows.id, nodes.id
FROM workflows
JOIN nodes ON nodes.node_type_id = workflows.workflow_node_type_id
ON CONFLICT DO NOTHING;
