# Sprint 6E Verification Contract

Status: **Complete and accepted on 2026-07-31.** The final runtime candidate
is Dashboard `2.0.2`; the source-exact closeout commit is `815d24b5` with tree
`f2e71bbf`. The immutable rollback release remains Dashboard `2.0.0` from
`27ae979c`.

Retained outputs are under `artifacts/sprint-6e-closeout/`. The application is
left running at `http://127.0.0.1:8080` with the candidate route active.

## Acceptance Mapping

| Roadmap clause | Automated proof | Manual proof | Result |
| --- | --- | --- | --- |
| Dashboard release has no root Core/web or Components implementation dependency | `verify-sprint-6e-boundaries.ps1`; native/WASM checks | Image package/source inspection | Pass |
| Dashboard owns five complete documents and immutable assets | Dashboard document/manifest tests; asset digest checks | Direct-load, JavaScript-disabled, hydration, theme, keyboard, and responsive sweep | Pass |
| Core generically owns auth, navigation, lifecycle, and fallback | lifecycle/bootstrap tests; gateway negotiation tests | Soft navigation, history, dirty guard, repeat mount/unmount, failure containment, recovery | Pass |
| Product, authorization, redaction, and provider degradation remain stable | API/web/Playwright suites; smoke; scripted UAT | Administrator, scoped manager, reader, provider-outage, and recovery walkthroughs | Pass |
| Only Dashboard upgrades and rolls back | health-gated refusal/success records and chronology | `2.0.0` → `2.0.2` → `2.0.0` → `2.0.2` walkthrough | Pass |
| Unrelated services remain unchanged during route switch | before/after container, image, and restart tuples | Operator comparison across Core, gateway, installation control, modules, and PostgreSQL | Pass |
| Existing Dashboard persistence is preserved | pinned migration and database regression | Edit before switch; read after upgrade and rollback | Pass |
| Candidate is observable without product redesign | document metadata, diagnostics, labels, and asset digest | Normal Dashboard documents and Module diagnostics | Pass |

## Retained Evidence

- source provenance for baseline and candidate;
- first bootstrap and exact second-bootstrap no-op;
- fresh deployment capture and migration checkpoint;
- package boundaries and Dashboard product regression;
- authorization/nondisclosure and provider outage/recovery;
- refused switch, candidate switch, baseline rollback, and full chronology;
- smoke and scripted UAT with SHA-256 sidecars;
- 62/62 Playwright summary and accepted 7/7 manual UAT record.

## Closing Validation

- `cargo fmt --all -- --check`: pass.
- `scripts/verify-sprint-6e-boundaries.ps1`: pass.
- candidate Compose configuration: pass.
- fresh deployment evidence: pass at `815d24b5`.
- acceptance smoke: pass.
- scripted sprint UAT: pass.
- complete pre-closeout SIT: pass, including Rust workspace and 62/62 Playwright.
- complete manual UAT: 7/7 pass.

Per the requested operating sequence, the repository is closed out before the
next full SIT cycle begins. That post-closeout SIT/UAT cycle is a validation
loop and does not reopen Sprint 6E scope unless it finds a defect.
