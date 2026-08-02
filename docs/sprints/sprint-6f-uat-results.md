# Sprint 6F UAT Results

Execution date: 2026-08-01/02  
Technical tester: Codex validation agent  
Candidate commit/tree: `599680992771fb2ac05633e36cae2ad84026318d` /
`773b15f3798c79e7223de75de9e956d033cae901`  
Environment: disposable local Sprint 6F Compose project  
Browser: Chromium through the connected Chrome browser and Playwright  
Overall result: **Passed**  
Defects: None open

These are retained execution results for the tester-ready source scripts in
[`sprint-6f-uat`](./sprint-6f-uat/README.md). The source templates remain
unchanged as required by their execution rules.

## UAT-6F-01 — Complete reference composition

- Fresh signed reference bootstrap completed with exact source labels.
- Reference plan was
  `sha256:dcc3a54d48231425e921d638d4966067ffc40a17d8b2f3353d26f3d6cd67e2af`.
- Core, Dashboard, and Scoped Records navigation/routes were present.
- `Reference record` existed exactly once with ID
  `01980000-0004-7000-8000-000000000001`.
- Result: **Pass**.

## UAT-6F-02 — Signed reduced composition

- A new detached resolved-composition envelope was signed from the frozen
  reference release catalog and verified during bootstrap.
- Applied lockfile byte hash matched the detached source lockfile.
- Reduced plan was
  `sha256:1807fec0e50e7911e03cb8780b421e490eeaa9ad253f38d1b103893905d7d897`.
- Locked modules: 0. Module bootstrap receipt entries: 0.
- Running services: Core, Gateway, PostgreSQL, Supervisor only.
- Dashboard returned 404, Scoped Records returned 403, and both navigation
  destinations were absent before and after Core restart.
- Result: **Pass**.

## UAT-6F-03 — Planning, approval, and access separation

- Temporary roles/accounts were provisioned for reader, planner, and
  constrained access.
- Valid-body reader create and resolve requests returned 403; reader approve,
  apply, and emergency requests returned 403.
- Constrained composition read returned 403 and browser navigation was denied
  without composition actions, digests, or secrets.
- Planner created Blueprint revision 2 (201) and resolved it (200) to
  `sha256:399c1c1bea799c2239762f201db45b4d5b9a274c50d75206dcfa47a4e10e5743`;
  planner approve/apply each returned 403.
- Administrator approved and applied the exact same digest (200/200), creating
  receipt revision 3.
- No signing secret, private key, or authorization token appeared in the UI.
- Result: **Pass**.

## UAT-6F-04 — Core restart recovery

- Core alone was restarted on the applied reference composition.
- Module instance IDs, Core bootstrap receipts, receipt revision, plan digest,
  exact composed roles, and unrelated container IDs were unchanged.
- Public readiness returned 200 and reference smoke passed after restart.
- Result: **Pass**.

## UAT-6F-05 — Drift management

- Changed Scoped Records label from `Scoped Records` to
  `Scoped Records — Adopted UAT`; one configuration drift finding exposed the
  exact desired/observed values and was adopted.
- Changed the label to `Scoped Records — Reconcile UAT`; a second finding used
  the adopted value as desired and was reconciled.
- Owner configuration returned to `Scoped Records — Adopted UAT`.
- Audit counts: one adopted, one reconciled, zero open.
- Result: **Pass**.

## UAT-6F-06 — Emergency disable

- Emergency-disabled `tessara.reference.scoped-records` with a reason and
  15-minute expiry.
- Drift showed desired `true`, observed `false` at the exact enabled path.
- Scoped Records returned 403 while Core and Dashboards remained 200.
- Reconcile restored Scoped Records to 200, retained the starter record, closed
  the finding, and stamped `reconciled_at` on the override.
- Result: **Pass**.

## UAT-6F-07 — Owner bootstrap

- Fresh reference apply produced Core and module owner bootstrap receipts.
- The declared starter record was created exactly once.
- Unchanged replay produced `no_op=true`; Core restart preserved bootstrap
  identities and the owner record.
- Reduced composition produced zero module bootstrap receipt entries and no
  module services.
- Contract/workspace validation covered content-digest tamper rejection.
- Result: **Pass**.

## UAT-6F-08 — Responsive and failure states

- At 1280, 768, and 390 pixels, the page retained the
  `Application Composition` heading, all four revision cards, and actions with
  `scrollWidth == clientWidth`.
- Keyboard sequence was Open navigation, Search, Theme options, Notifications,
  Help, Blueprint editor, Create draft, Catalog editor; focus outline was
  visible (`auto`) throughout.
- Theme was coherent, browser back/forward restored the expected titles/routes,
  hydration completed, and no console errors were recorded.
- With Supervisor stopped, Supervisor status failed explicitly while Core
  readiness, shell navigation, and composition page remained 200. Supervisor
  recovered to 204 with the same plan/receipt and post-recovery smoke passed.
- Read-only state exposed no secrets; constrained access was denied.
- Result: **Pass**.

## Acceptance decision

- Technical UAT decision: **Accepted**
- Business acceptance decision: **Accepted for closeout** under the user's
  instruction to proceed with testing and validation.
- Final environment cleanup: temporary UAT identities and revision-2 state were
  removed by recreating the disposable reference installation; canonical final
  smoke passed.
