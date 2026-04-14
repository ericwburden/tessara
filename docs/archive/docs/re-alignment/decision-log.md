# Tessara Decision Log

## Final Core Model
`Dataset → Component → Dashboard`

## Summary of Settled Decisions
- Roles are reusable permission bundles applied via scoped role assignments.
- Scope flows through one organization tree and includes descendants.
- Forms are versioned publishable assets.
- Workflow steps reference published FormVersions.
- Canonical responses are stored as structured payloads keyed by published field keys.
- Query-backed selects use constrained Lookup Sources.
- Datasets are editable logical assets with internal immutable DatasetRevisions.
- DatasetRevisions may be materialized and evicted/rebuilt on demand.
- Reports and Aggregations were collapsed into Dataset + Component responsibilities.
- Components are versioned assets over DatasetRevisions.
- Dashboards are mutable compositions of ComponentVersions.

## Reporting / Presentation Evolution
### Superseded
`Dataset → Report → Aggregation → Chart → Dashboard`

### Final
`Dataset → Component → Dashboard`
