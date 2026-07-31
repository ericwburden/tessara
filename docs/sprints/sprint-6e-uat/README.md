# Sprint 6E UAT Scripts

Run these scripts in order against the dedicated Sprint 6E stack at
`http://127.0.0.1:8080`. Each script covers one business scenario.

1. [Use Dashboard routes normally](./uat-6e-01-dashboard-experience.md)
2. [Create, edit, view, and retain a Dashboard](./uat-6e-02-authoring-persistence.md)
3. [Use SSR, hydration, and responsive layouts](./uat-6e-03-document-presentation.md)
4. [Preserve authorization and nondisclosure](./uat-6e-04-authorization.md)
5. [Contain lifecycle and provider outages](./uat-6e-05-outage-containment.md)
6. [Upgrade and roll back only Dashboard](./uat-6e-06-upgrade-rollback.md)

## Sprint Acceptance Criteria

Sprint 6E is accepted only when all of the following are observable:

1. All five Dashboard routes remain usable through normal same-origin Tessara
   navigation and direct loads.
2. Directory, create, detail, editor, and viewer pages provide useful,
   redaction-safe server-rendered content and hydrate without visible errors.
3. Dashboard creation, metadata, placement layout, viewing, and deletion retain
   their existing behavior and persisted values.
4. Administrator, scoped-manager, and reader access remain limited to their
   authorized actions and organization scope; unknown and unauthorized
   resources disclose no distinguishing information.
5. Dashboard lifecycle changes and Components-provider degradation remain
   contained; unrelated Core and module experiences remain available.
6. A healthy `2.0.1` candidate can replace `2.0.0` and be rolled back without
   losing Dashboard data or restarting unrelated services.
7. Normal Dashboard documents, diagnostics, immutable assets, and image labels
   expose the active release identity without adding product UI clutter.
8. Any failed or blocked step is retained explicitly and prevents unconditional
   business acceptance.

Completed manual results are retained at
`artifacts/sprint-6e-closeout/manual-uat.md`.
