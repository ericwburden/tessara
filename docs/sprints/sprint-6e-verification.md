# Sprint 6E Verification Contract

Status: UAT fixes are implemented and the administrator product flow plus
candidate and maintained-baseline rollback flows pass. Final acceptance remains
blocked on scoped-manager/nondisclosure fixtures, the explicit presentation
sweep, provider degradation, and a new source-exact retained chronology.

This document is the retained acceptance map for Dashboard SDK adoption and
source independence. Final run outputs belong under
`artifacts/sprint-6e-closeout/`.

## Acceptance Map

| Roadmap clause | Automated proof | Manual proof |
| --- | --- | --- |
| Dashboard release has no root Core/web or Components feature implementation dependency | `scripts/verify-sprint-6e-boundaries.ps1`; native and WASM Cargo checks | Inspect Dashboard image package/source inventory |
| Dashboard owns five complete documents and immutable assets | manifest-schema tests; Dashboard document tests; same-origin route checks | Direct-load all five routes with JavaScript disabled and enabled |
| Core retains auth, navigation, lifecycle, and fallback ownership generically | lifecycle manifest/bootstrap validation; gateway negotiation tests; Core host route and shell projection tests | Run UAT-6E-07 across Core, Dashboard, history, guard, direct-load, and recovery paths |
| Product, authorization, redaction, and provider degradation remain stable | Dashboard/UI tests; existing API, UAT, and Playwright suites | Administrator, scoped-manager, reader, provider-outage, and recovery walkthroughs |
| Only Dashboard upgrades and rolls back | Compose config; route-switch refusal/success records; chronology validator | Observe `2.0.0`, refuse unhealthy `2.0.2`, switch healthy candidate, then restore baseline |
| Unrelated services do not restart or change image | before/after Docker inspection captured in chronology | Compare Core, gateway, installation-control, Scoped Records, reference SDK, and PostgreSQL |
| Existing Dashboard persistence is preserved | migration byte pin; disposable baseline apply; before/after data digest | Edit before upgrade, confirm after upgrade and rollback |
| Candidate is observable without product redesign | document metadata, diagnostics, image labels, and asset digest assertions | Inspect normal Dashboard document metadata and Module diagnostics |

## Required Retained Files

- `source-provenance-baseline.json`
- `source-provenance-candidate.json`
- `bootstrap-first.json`
- `bootstrap-second-noop.json`
- `migration-checkpoint.json`
- `package-boundaries.json`
- `dashboard-product-regression.json`
- `authorization.json`
- `provider-outage-recovery.json`
- `route-switch-refused-candidate.json`
- `route-switch-candidate.json`
- `route-switch-baseline.json`
- `upgrade-rollback-chronology.json`
- `smoke.json`
- `uat.json`
- `playwright-summary.json`
- `manual-uat.json`

## Commands

```powershell
cargo fmt --all -- --check
cargo test -p tessara-module-contract --locked
cargo test -p tessara-dashboard-module --locked
cargo test -p tessara-dashboard-ui --locked
cargo leptos build -p tessara-dashboard-ui --release
cargo check -p tessara-dashboard-ui --target wasm32-unknown-unknown --no-default-features --features hydrate --locked
cargo test -p tessara-api --locked
cargo test -p tessara-web --locked
.\scripts\verify-sprint-6e-boundaries.ps1
docker compose -f deploy/sprint-6e/compose.yaml --profile candidate config --quiet
.\scripts\bootstrap-sprint-6e-deployment.ps1
npm --prefix .\end2end test
.\scripts\smoke.ps1
.\scripts\uat-sprint.ps1 -BaseUrl "http://localhost:8080"
```

Database-backed `tessara-api` tests require their existing disposable
`TEST_API_DATABASE_URL` and `TEST_API_ENROLLMENT_DATABASE_URL` bindings; they
must not be reported as passing when those variables are absent.
