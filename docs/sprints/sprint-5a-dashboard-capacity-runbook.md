# Sprint 5A Dashboard Capacity Upgrade Runbook

Sprint 5A supports at most 240 stored placements per Dashboard and no placement may extend beyond grid row 240. Migration `002_dashboard_placement_capacity.sql` stops before changing the database when it finds an over-cap Dashboard or a stored layout the bounded fallback algorithm cannot display. It never truncates, repositions, or repairs placements automatically.

## Preflight Query

Run this query against the database that will be upgraded:

```sql
SELECT
    dashboards.id,
    dashboards.name,
    COUNT(dashboard_components.id) AS placement_count
FROM dashboards
JOIN dashboard_components
  ON dashboard_components.dashboard_id = dashboards.id
GROUP BY dashboards.id, dashboards.name
HAVING COUNT(dashboard_components.id) > 240
ORDER BY dashboards.name, dashboards.id;
```

An empty result passes the stored-count phase of the preflight. The migration also applies the runtime's exact V1 classifier and rejects either of these mixed-layout conditions:

- two valid V1 placement rectangles overlap;
- the rows touched by valid V1 rectangles plus the number of legacy, malformed, or future-schema fallback rows exceed 240.

The second condition matters even below the placement-count cap. For example, one valid placement that is one column wide but 240 rows tall leaves no row for a full-width legacy fallback. The migration error identifies every affected Dashboard and reports occupied and required fallback row counts. The classifier in the migration is the canonical database preflight and includes the Table-specific 6-by-4 minimum.

## Operator-Controlled Cleanup

For every Dashboard reported by any preflight phase:

1. Back up the database and export the affected Dashboard's `dashboards`, `dashboard_scope_nodes`, and `dashboard_components` rows.
2. Inspect placement ids, positions, component kinds, and raw configs. Have the product/data owner choose whether to remove a placement or have an editor repair/reposition it. The migration does not choose for them.
3. Apply only the explicitly approved removals or repairs inside a transaction.
4. Re-run the migration preflight and verify the stored count, V1 rectangles, and required fallback rows all pass.
5. Retain the export with the deployment record, then rerun the application migration.

Example transaction after the approved placement-id list has been reviewed:

```sql
BEGIN;

DELETE FROM dashboard_components
WHERE dashboard_id = '<dashboard-id>'::uuid
  AND id = ANY(ARRAY[
      '<approved-placement-id-1>'::uuid,
      '<approved-placement-id-2>'::uuid
  ]);

SELECT COUNT(*) AS remaining_placements
FROM dashboard_components
WHERE dashboard_id = '<dashboard-id>'::uuid;

-- Commit only after the count and approved deletion set have been checked.
COMMIT;
```

Use this inspection query for a Dashboard named in a display-layout error:

```sql
SELECT dashboard_components.id AS placement_id,
       dashboard_components.position,
       component_versions.component_type::text AS component_kind,
       dashboard_components.config
FROM dashboard_components
JOIN component_versions
  ON component_versions.id = dashboard_components.component_version_id
WHERE dashboard_components.dashboard_id = '<dashboard-id>'::uuid
ORDER BY dashboard_components.position, dashboard_components.id;
```

If the reviewed deletion set or resulting count is wrong, use `ROLLBACK` instead of `COMMIT` and restore from the export if necessary.

## Post-Upgrade Invariant

The API rejects a 241st placement atomically, and the database trigger independently prevents unsupported insert/move paths from exceeding the same cap. The API validates every canonical layout before writing it. Existing placement ids and remaining rows are not rewritten by the migration.
