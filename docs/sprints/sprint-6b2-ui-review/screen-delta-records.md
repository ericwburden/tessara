# Sprint 6B2 UI Screen Delta Records

Status: approved for Sprint 6B2 implementation on 2026-07-23 after product-owner annotation feedback was applied. Approval covers the recorded deltas only. Unlisted Tessara shell, component, route, responsive, and interaction behavior remains unchanged.

## Baseline Sources

- Running application captured on 2026-07-23 at `http://localhost:8080`.
- Production visual tokens and component patterns from `style/main.css`.
- Product behavior from `docs/ui-guidance.md`, `docs/ui-guidance-spec.md`, and `docs/sprints/sprint-6b2-plan.md`.
- Reference captures under `reference/`.
- Runnable review prototype under `prototype/`.

## Screen 1: Administrator Enrollment

Route class: dedicated bare route outside `AppShell`.

Baseline preserved:

- centered Tessara authentication card;
- current mark, typography, dark surface, field construction, semantic focus, error placement, and primary action;
- no authenticated navigation or account shell.

Proposed additions and changes:

- distinguish Initial and audited Recovery claim kinds;
- choose Local account or signed Fixture external identity binding;
- accept the enrollment claim as write-only password input;
- summarize the assigned Core Administrator role and Capability Floor v1 without offering capability selection;
- show a terminal success state that confirms the claim is closed without redisplaying it;
- use one non-disclosing unavailable treatment for invalid or terminal claim states.

Explicitly unchanged:

- ordinary sign-in remains `/login`;
- successful enrollment continues to normal sign-in;
- no production OIDC or provider configuration is introduced.

## Screen 2: Roles And Capability Floor

Route class: bounded edit to existing Roles & Access.

Baseline preserved:

- current role directory, selection, detail card, capability table, actions, pagination, and shell;
- `admin:all` remains visible as the explicit break-glass superuser capability.

Proposed additions and changes:

- add a compact Capability Floor v1 status banner;
- identify Core Administrator as the designated enrollment role;
- present every Enrollment-column value as text rather than mixing text and badges;
- use the role-name link as the single detail affordance and omit a duplicate row action;
- show compliance and block weakening or removal that would leave no compliant designation;
- add `core:admin` as the installation-global umbrella capability;
- explain that module product capabilities remain separately assigned.

## Screen 3: Scoped Records Module Configuration

Route class: bounded edit to existing independently deployed module detail.

Baseline preserved:

- existing heading, definition identity, deployment/health badges, descriptor and receipt actions, tabs, and Module Management shell;
- configuration remains distinct from enablement, navigation visibility, health, and deployment state.

Proposed additions and changes:

- make `display_label` editable through the module-owned schema-v1 validator;
- show normalized validation readback and stable findings;
- remove retention mode from product configuration and leave it in deployment/lifecycle declarations;
- add a compact application-state card with configuration, health, navigation, and product-route enablement;
- link to module-owned health and diagnostics.

## Screen 4: Scoped Records Directory

Route class: new independently deployed module product route.

Baseline preserved:

- current Tessara authenticated shell, header actions, breadcrumbs, compact table/filter patterns, badges, and responsive card conversion;
- existing Core navigation remains present with one Scoped Records contribution added to Main.

Proposed additions:

- show only records within the actor’s `scoped_records:read` Organization bindings;
- search by label, record ID, or accessible Organization;
- filter only among accessible Organizations;
- show read versus read/manage authority without exposing unavailable records;
- use the record-name link as the single detail affordance and omit a duplicate row action;
- open record detail and create flows.

## Screen 5: Scoped Records Detail

Route class: new independently deployed module product route.

Proposed additions:

- show the minimal reference record: ID, label, Organization owner, and timestamps;
- show edit only when `scoped_records:manage` covers the owning subtree;
- provide a concise authorization-context card with capability, subtree, freshness, and presenting service;
- explicitly state that Core credentials are not shared with the module.

## Screen 6: Scoped Records Create

Route class: new standalone independently deployed module product route.

Proposed additions:

- open from the directory’s New Record action without passing through a record-specific edit route;
- enter the new record label and Organization owner;
- validate manage authority for the selected Organization before create;
- block a read-only Organization selection with an inline non-disclosing denial;
- consume the one-time mutation authorization in the same transaction as create;
- do not introduce delete.

## Screen 7: Scoped Records Edit

Route class: new independently deployed module product route.

Proposed additions:

- edit the record label and Organization owner only;
- validate manage authority for the selected Organization before save;
- block a read-only Organization selection with an inline non-disclosing denial;
- consume the one-time mutation authorization in the same transaction as the save;
- remain a record-specific route with no create/edit mode switch;
- do not introduce delete.

## Screen 8: Health And Diagnostics

Route class: new independently deployed module administration route.

Proposed additions:

- show module readiness, liveness, configuration validity, and Core authorization connectivity;
- use each probe name as the card heading and show its status once, without a duplicate badge;
- separate diagnostic cards from the tab divider with the standard section gap;
- expose sanitized Module Instance, database-binding, authorization-revision, and Organization-revision context;
- show stable module-owned findings;
- permit sanitized diagnostic download without claim secrets, Core credentials, or browser cookies.

## Screen 9: Denied, Stale, Disabled, And Degraded States

Route class: review collection; each treatment appears in its owning production route.

Proposed additions:

- scoped denial explains the missing action without disclosing inaccessible records;
- stale authorization preserves unsaved work and asks for a fresh Core decision;
- disabled state keeps configuration and diagnostics recoverable;
- degraded state preserves shell, data-retention, and a path to diagnostics;
- every state names what remains protected and provides one primary recovery action.

## Responsive Contract

- Desktop: current fixed sidebar, top header, dense table, tabs, and two-column detail composition.
- Tablet: narrower sidebar, stacked detail cards, wrapped floor metadata.
- Mobile: compact top bar, hidden desktop sidebar, table-to-card conversion, full-width actions, and a full-width Module detail selector instead of overflowing tabs.
- All widths: keyboard-visible focus, semantic status treatment, no hidden persistent action, and no horizontal page overflow.
