# UAT-7B-05 — Replace, remove, and conflict

With two authorized renderable ComponentVersions, replace the affected
placement with a target outside the current Component identity. Verify the
placement and finding converge atomically. Replay stale and mismatched requests;
verify deterministic conflict/no-op behavior and no partial write. Exercise
Remove against its dedicated fixture and verify placement removal plus finding
closure are atomic. Restore both fixtures.
