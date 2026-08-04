# UAT-7B-08 — Dataset equivalence

Exercise the existing Dataset impact/carry-forward flow and the Component
observation flow through their typed adapters. Verify both expose the same
reference, authorization-first nondisclosure, resolution, revision, and
observation semantics while retaining provider-specific actions. Audit process
and database access to confirm neither adapter reads another module database and
no new workspace or compatibility bridge exists. Restore both resources.
