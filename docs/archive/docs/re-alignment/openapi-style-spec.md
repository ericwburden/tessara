# OpenAPI-Style Spec

## Main resource families
- `/users`
- `/roles`
- `/role-assignments`
- `/organization/nodes`
- `/field-definitions`
- `/option-sets`
- `/lookup-sources`
- `/forms`
- `/workflows`
- `/workflow-assignments`
- `/workflow-instances`
- `/form-responses`
- `/datasets`
- `/components`
- `/dashboards`

## Dataset endpoints
- `POST /datasets`
- `PATCH /datasets/{dataset_id}`
- `POST /datasets/{dataset_id}/revisions`
- `GET /datasets/{dataset_id}/revisions/{dataset_revision_id}/sql`

## Component endpoints
- `POST /components`
- `POST /components/{component_id}/versions`
- `POST /components/{component_id}/versions/{component_version_id}/validate`
- `POST /components/{component_id}/versions/{component_version_id}/publish`
