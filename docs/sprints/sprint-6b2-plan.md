# Sprint 6B2 Plan: Secure Module Operation Slice

Status: kicked off on 2026-07-23 from clean `main` commit `7ed779d6`.

- Branch: `codex/sprint-6b2`
- Worktree: `C:\Users\eric-dev\Projects\tessara-sprint-6b2`
- Roadmap source: `Sprint 6B2: Secure Module Operation Slice (Next)`
- Predecessor: `Sprint 6B1: Container Deployment Foundation Slice (Complete)`

## Sprint Summary

Sprint 6B2 turns the independently deployed boundary proven in Sprint 6B1 into a secure, user-operable vertical slice. An operator can issue a once-displayed installation-bound enrollment claim, establish the first viable administrator through a local or signed fixture-external identity, configure and enable Scoped Records, and grant different read/manage authority over disjoint Organization subtrees. Core remains the authority for identity, roles, Organization closure, and authorization revisions; the deployment-control boundary owns claim lifecycle outside the Core backup boundary; and Scoped Records remains the authority for its configuration and product records.

The slice must fail closed across process boundaries. Browser cookies and Core credentials never cross into the module. Core supplies a signed, short-lived, audience/action/installation-bound authorization decision or grant and a separate non-secret `ShellContextV1`; the module validates the presenting service, declared dependency/contract, original actor, capability-to-scope binding, ownership/delegation assertions, revisions, expiry, and replay state before reading or mutating module data.

Container planning and lifecycle remain in `tessara-deploy`. The new installation-control implementation is limited to the enrollment lifecycle and the private, mutually authenticated reservation/finalization contract required by Core; it does not absorb Docker, Compose, gateway, or verified-distribution work.

## Sprint Specifications

### Installation Control And Enrollment Claims

- Add a narrow versioned installation-control contract and local process/CLI surface backed by the `tessara_deployment` database. Keep its runtime and migration identities separate from Core and all Module Instance identities.
- Support initial claim issue, non-secret status, reserve, consume, expire, revoke, replace, and reconcile operations. At most one claim generation may be `issued` or `reserved`; replacement terminally records the prior generation and issues a new one only when eligibility still holds.
- Bind every claim to the Application Installation, claim kind (`initial` or audited `recovery`), generation, issue/expiry times, and reservation identity. Persist only a one-way secret verifier and signed non-secret metadata. Return the secret exactly once from issue and exclude it from later output, receipts, logs, diagnostics, audit details, and errors.
- Require a current signed Core eligibility decision that no Viable Core Administrator exists before initial issuance. Recovery additionally requires explicit audited break-glass evidence. Initial issuance closes permanently after the installation has ever established a viable administrator.
- Use one non-disclosing rejection envelope for expired, revoked, replaced, replayed, consumed, reserved-by-another-redemption, malformed, and cross-installation claims.
- Make redemption idempotent: Core reserves one generation, applies identity/binding, global enrollment-role assignment, and redemption evidence in one Core database transaction, and returns a signed result that installation control consumes or reconciles. A Core restore predating redemption must re-check the external lifecycle and cannot reuse the claim.
- Add a guided local Supervisor command equivalent to `tessara enrollment issue --open`. It obtains the signed Core eligibility decision, issues or replaces the eligible initial claim, displays the secret once, and opens the enrollment page with installation ID, claim ID, generation, and claim kind already populated. It must not require operators to hand-author eligibility JSON or assemble Docker commands.
- Add a guided recovery command equivalent to `tessara enrollment recover --reason <reason> --open`. It captures the local operator identity and reason, creates and records the required signed operator authorization, issues or replaces the eligible recovery claim, and opens the same prepared browser flow. This convenience command does not bypass recovery eligibility, signature, audit, or nondisclosure requirements.
- Use a short-lived, single-use local browser handoff for prefilled non-secret metadata. Keep the claim secret write-only; never place it in a URL, browser history, automatic clipboard operation, persistent browser storage, logs, or later command/status output.

### Capability Floor And Administrator Viability

- Define a versioned Core Administration Capability Floor in Core-owned contract data and persist the selected floor version plus designated Administrator Enrollment Role.
- Validate that the designated role exists, is installation-global, and covers every capability in the selected floor. Apply the same validator to seed/composition input, enrollment, role edits, and designation changes; block any change that would leave no compliant designation.
- Define a Viable Core Administrator as an active, authenticable identity with an active installation-global assignment to a role covering the complete current floor. Do not treat a disabled identity, unusable external binding, scoped assignment, or partial/admin-like role as viable.
- Track authorization revision inputs needed to invalidate previously issued decisions after role assignment, role capability, account state, ownership, or delegation changes.

### Enrollment Identity Paths And UI

- Add a dedicated bare administrator-enrollment route outside `AppShell` and distinct from `/login`. It identifies initial versus recovery enrollment, accepts the claim as write-only input, and closes when a viable administrator exists.
- Local enrollment creates an active local identity with an Argon2id password hash. The signed fixture-external path verifies a versioned fixture issuer assertion and binds its stable external subject without introducing production OIDC scope.
- Both paths use the same reservation and local transaction, assign only the locked floor-compliant enrollment role globally, and never offer capability selection or claim-secret redisplay.
- Show local password requirements before submission and enforce the same rules on the client and server. Leave identity fields blank unless the signed external assertion supplies them; do not default the local form to an email address that may already exist.
- Successful enrollment closes the route and continues to normal sign-in. Interrupted redemption resumes only the same reservation. Failed submissions and all terminal or invalid claim states remain inside the designed enrollment surface, use the same non-disclosing unavailable treatment where required, and offer only safe retry or claim-reissue guidance. The browser must not fall through to a raw API JSON document.
- Add Core administration readback for the current capability-floor version, designated enrollment role, compliance/viability state, and non-secret enrollment lifecycle. No control or support screen may reveal a claim verifier or secret.

### `ShellContextV1` And Module SDK

- Define `ShellContextV1` in the shared module contract with installation identity, module audience/instance, original actor display projection, theme, authorized navigation projection, return destination, locale/time-zone context, correlation identifier, issued/expiry times, schema version, and Core signature metadata.
- Keep `ShellContextV1` non-authoritative for product actions. Authorization is carried separately and must be evaluated even when shell context is valid.
- Deliver module SDK helpers that validate the Core signer, installation, audience, schema version, expiry, and correlation binding and render a complete native SSR document matching the Core shell. Do not forward browser cookies, Core session tokens, Core database credentials, or reusable downstream authority.
- Define explicit disabled, degraded, stale-context, and recovery document states that preserve shell and return-path context without claiming product authorization.

### Core Authorization Decisions And Grants

- Add a versioned Core authorization exchange for online decisions and short-lived signed grants. Each result binds installation, original actor, presenting service, target Module Instance audience, declared dependency/contract/action, individual capability-to-scope bindings, requested resource/owner assertions when applicable, delegation basis, authorization revision, Organization revision, issued/expiry times, and a unique replay identifier.
- Preserve each capability and its authorized Organization subtree as an independent binding. Never flatten multiple capabilities/scopes into a cross-product. Core performs descendant expansion; modules do not reconstruct Organization closure from module data.
- Permit exchange only for a presenting service and dependency/contract/action declared by the installed module manifests. Reject undeclared callers, wrong audience, wrong action, wrong installation, and attempts to forward a grant as downstream authority.
- Re-evaluate current account, role, Organization, ownership, and delegation state during exchange. Providers validate the actor and presenting service. Mutating operations require replay consumption; read decisions remain short-lived and revision-bound.
- Return a stable restricted result before resource existence details for unauthorized or unevaluated callers. Detailed denial reasons are limited to separately authorized diagnostics.

### Scoped Records Product Slice

- Expand the reference module into an organization-owned record directory and detail workflow. Every record stores its authoritative Organization owner/scope identifier in the module database.
- Declare separate scope-aware `scoped_records:read` and `scoped_records:manage` capabilities. Directory results include only records covered by the read binding; create/update operations require manage authority for the record's exact Organization subtree.
- Replace the free-form Sprint 6B1 API trust with the Core exchange contract for browser and machine clients. Enforce installation, audience, action, presenting service, scope, revisions, expiry, and replay rules inside the module.
- Define one versioned module-owned configuration schema and validator used by the module UI, Core configuration orchestration, CLI/machine clients, and readiness. Persist normalized configuration and expose the same findings everywhere.
- Deliver complete module-owned configuration, authorization, enablement, directory, detail, health, and diagnostics routes using native SSR and the shared shell contract. Keep Core responsible for desired configuration/enablement orchestration and cross-module status; keep record fields and validation inside Scoped Records.
- Preserve the Sprint 6B1 Module Instance identity, database binding, and stored records through compatible upgrade and rollback.

### HTML/CSS Product Contract

- Create and review the Sprint 6B2 runnable HTML/CSS screens and per-screen delta records before implementing visual changes. Use the current application, `docs/ui-guidance.md`, and `docs/ui-guidance-spec.md` as the authoritative baseline.
- Cover administrator enrollment; capability-floor/enrollment-role readback; Scoped Records configuration, authorization, enablement, directory, detail, health, and diagnostics; and scoped denial, stale authorization, disabled, degraded, and recovery states.
- Treat mockups as review tools, not replacement application specifications. Preserve established routes, shell components, responsive patterns, icons, and behavior unless an approved delta explicitly changes them.
- Validate light/dark themes at 1280, 768, and 390 pixels, keyboard-only operation, 200% zoom, long labels/identifiers, no-JavaScript SSR usefulness, and overflow containment.

## Acceptance Criteria

1. Initial and recovery claim issuance enforce their different eligibility rules, persist only a one-way verifier, display a secret once, and maintain an auditable issued/reserved/consumed/expired/revoked/replaced generation history outside the Core backup boundary. Guided issue and recovery commands acquire and record the required eligibility or operator authorization and open a prepared enrollment page without exposing the secret through the handoff.
2. Local Argon2id and signed fixture-external enrollment paths atomically create or bind one identity, assign the designated floor-compliant role globally, and reconcile the same reservation idempotently.
3. Expired, revoked, replaced, replayed, consumed, cross-installation, wrong-reservation, and pre-redemption-restore attempts fail with the same non-disclosing public outcome.
4. The versioned capability floor rejects missing, scoped, or incomplete enrollment roles and prevents role/designation edits that would leave no viable compliant administrator path.
5. `ShellContextV1` lets Scoped Records render a complete native SSR document consistent with Core without receiving browser cookies, Core sessions, or Core credentials.
6. Authorization exchange rejects wrong installation, audience, action, presenting service, undeclared dependency/contract, stale role/Organization/ownership/delegation revisions, expired grants, and mutation replay.
7. An A/X versus B/Y fixture proves that independent capability-to-scope bindings do not become an A/Y or B/X cross-product.
8. Scoped Records directory/detail and machine APIs return only authorized records/actions and preserve known-versus-random nondisclosure for callers without existence-disclosure authority.
9. The same module-owned configuration validator and normalized result drive Core orchestration, module UI, machine clients, readiness, and diagnostics.
10. Core and module routes expose clear enrollment, floor compliance, configuration, authorization, enablement, disabled, degraded, stale, health, diagnostics, and recovery states from the approved UI contract. Enrollment shows password requirements, does not prefill an existing local email, and renders failed redemption with safe retry/reissue guidance rather than raw API JSON.
11. Compatible upgrade and rollback preserve the Sprint 6B1 Module Instance identity, database binding, configuration, and product records.
12. Existing Core authentication, application routes, Module Management, hydration, SSR, and deployment/database isolation remain intact.

## Manual Test Plan

1. Start a fresh Sprint 6B2 installation, run the guided local issue-and-open command, confirm the browser receives prefilled non-secret claim metadata, enter the once-only secret, and confirm URLs, history, clipboard, later status, logs, and diagnostics contain no secret.
2. Enroll a local administrator on the prepared bare enrollment route, confirm password requirements are visible and the email begins blank, sign in normally, inspect the designated role and floor readback, and confirm claim replay and a second initial issue are unavailable.
3. Repeat on a clean fixture using the signed external-identity path; exercise interrupted redemption and reconciliation without duplicate identity or role assignment.
4. Exercise expired, revoked, replaced, cross-installation, and pre-redemption-Core-restore cases and compare their public status, shape, and message. Confirm each browser submission remains in the designed enrollment page with appropriate retry/reissue guidance and never navigates to raw API JSON.
5. Configure and enable Scoped Records, then inspect its module-owned configuration, directory, detail, health, and diagnostics routes through the Tessara origin with JavaScript enabled and disabled.
6. Assign user X read authority under subtree A and manage authority under subtree B; assign comparison user Y the inverse. Prove each user sees and performs only the intended record/action combinations.
7. Change a role, Organization subtree, ownership assertion, and delegation after issuing authorization. Confirm stale grants fail and fresh exchange reflects the new state.
8. Stop/disable Scoped Records, inspect contained degraded/recovery states in Core and the module shell, restore it, and verify stable identity/data.
9. Upgrade and roll back the compatible module release; confirm Module Instance identity, database binding, configuration, and records are unchanged.
10. Review every changed UI against the approved delta record at desktop, tablet, and mobile widths in both themes, with keyboard navigation and 200% zoom.
11. Remove the last viable administrator in a recovery fixture, run the guided reason-bearing recovery-and-open command, verify its immutable operator authorization record, and complete recovery without manual eligibility files or Docker command assembly.

## Automated Test Plan

- Contract/unit: claim and authorization canonical serialization/signatures, state transitions, generation replacement, verifier redaction, capability-floor validation, `ShellContextV1`, scope-binding separation, revision checks, expiry, and replay.
- Installation control: isolated deployment-database integration tests for issue/status/reserve/consume/revoke/replace/reconcile, concurrent reservation, recovery authorization, and Core-backup restore safety.
- Core integration: atomic local/external enrollment, Argon2id verification, fixture assertion validation, viable-administrator closure, role/designation mutation guards, and signed result reconciliation.
- Authorization conformance: A/X versus B/Y, wrong/undeclared caller-audience-action combinations, downstream exchange, stale role/subtree/ownership/delegation, mutation replay, and bounded known-versus-random timing methodology.
- Module integration: configuration-validation parity, directory/detail filtering, manage containment, machine-client behavior, disabled/degraded/recovery states, probes, and diagnostics redaction.
- Browser acceptance: guided issue/recovery handoff, prefilled non-secret claim metadata, write-only secret handling, visible password requirements, blank local identity defaults, in-page failed redemption and retry/reissue guidance, bare enrollment, normal sign-in continuation, floor/role readback, module configuration and enablement, scoped directory/detail actions, denial/stale/recovery states, native SSR/no-JavaScript behavior, hydration, console, accessibility, and responsive containment.
- Deployment lifecycle: fresh start, isolated credentials, module stop/recovery, compatible upgrade/rollback, stable Module Instance/database identity, and retained configuration/records.
- Required planned commands:
  - `cargo fmt --all`
  - `cargo test -p tessara-module-contract`
  - targeted tests for the new installation-control crate/process
  - `cargo test -p tessara-reference-scoped-records`
  - `cargo test -p tessara-api` with an isolated disposable `TEST_DATABASE_URL`
  - `cargo test -p tessara-web`
  - `npx playwright test` is retained as a baseline requirement but is targeted through the repository-owned package runner because Playwright is package-local
  - `npm --prefix .\end2end test`
  - `.\scripts\validate-e2e.ps1` for canonical manifest-bound acceptance evidence
  - `.\scripts\smoke.ps1`
  - `.\scripts\local-launch.ps1`
  - `.\scripts\uat-sprint.ps1 -BaseUrl "http://localhost:8080"`

## Ordered Implementation Plan

1. Freeze the installation-control, claim, signer/trust, `ShellContextV1`, authorization decision/grant, and module configuration contracts with canonical fixtures and negative vectors.
2. Build and approve the runnable Sprint 6B2 HTML/CSS review suite and bounded per-screen delta records before application UI work.
3. Add deployment-control migrations and the narrow installation-control process/CLI for claim lifecycle, concurrency, redaction, private Core reservation/finalization, and the guided issue/recovery browser handoff.
4. Add Core capability-floor/designated-role persistence and validation, viable-administrator decisions, authorization/Organization revision tracking, and edit guards.
5. Implement local Argon2id and signed fixture-external enrollment transactions, reconciliation, the prepared bare enrollment route, in-page error/reissue treatment, and administrative readback.
6. Implement `ShellContextV1` and module SDK validation/rendering helpers; convert Scoped Records to complete native SSR documents without Core cookies or credentials.
7. Implement Core authorization exchange and module verification/replay enforcement, beginning with fail-closed contract tests and the A/X versus B/Y matrix.
8. Add the Scoped Records configuration schema/validator and organization-owned directory/detail persistence, APIs, capabilities, and module-owned UI.
9. Implement only the approved Core and module UI deltas for configuration, authorization, enablement, health, diagnostics, denial, stale, disabled, degraded, and recovery states.
10. Prove nondisclosure, restore safety, outage/recovery, upgrade/rollback, isolation, regression, and final retained evidence; reconcile development migrations with the repository's baseline policy at closeout.

## Dependencies And Blockers

- The Sprint 6B2 runnable HTML/CSS review suite and per-screen delta records under `docs/sprints/sprint-6b2-ui-review/` were product-owner approved on 2026-07-23 after annotation feedback was applied. Production UI implementation is authorized only for those recorded deltas; unlisted shell and interaction behavior remains unchanged.
- The installation-control process needs a private mutually authenticated Core channel and signing-key handling that are not present in Sprint 6B1. Development trust fixtures must be explicit and must not be described as a production key-management solution.
- The existing `tessara_deployment` database is provisioned but has no product schema. Sprint 6B2 must add its own migration/runtime identities and cannot give Core or Scoped Records direct database access.
- The external-identity path is a signed fixture conformance path only. Production OIDC/SAML provider discovery, browser redirect, token refresh, and federation administration are deferred.
- Verified third-party container admission/distribution, production destructive lifecycle workflows, multi-host scheduling, and a general-purpose container control plane remain future platform work.
- Docker Engine with Compose, PostgreSQL 17, Rust/Cargo, Node/npm, and the package-local Playwright runtime are required for full local acceptance.
