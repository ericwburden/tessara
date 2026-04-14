# Architecture Spec

## Layers
1. Capture
   - Fields
   - Forms
   - Workflows

2. Runtime
   - Assignments
   - WorkflowInstances
   - FormResponses

3. Materialization
   - reporting parent rows
   - multi-select child rows

4. Modeling
   - Dataset
   - DatasetRevision
   - Dataset contract

5. Presentation
   - Component
   - ComponentVersion

6. Composition
   - Dashboard

## Key principles
- Stable dependency edges bind to immutable revisions or versions.
- User-facing authoring should prefer automatic derivation over manual metadata entry.
- Archived/inactive records remain resolvable for historical integrity.
- Physical materializations may be evicted while semantic revision metadata remains.

## Data Flow
Forms/Workflows → Responses → Materialized Sources → DatasetRevision → ComponentVersion → Dashboard
