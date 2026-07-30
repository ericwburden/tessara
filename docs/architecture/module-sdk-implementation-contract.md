# Module SDK Implementation Contract

Status: accepted Sprint 6D implementation contract.

This document makes the
[Sprint 6D roadmap outcome](../roadmap.md#sprint-6d-canonical-module-sdk-and-runtime-extraction-slice-next)
concrete. It refines the source-ownership decision in
[Module SDK Source Ownership And Deployment](./module-sdk-source-ownership.md)
without expanding Sprint 6D into Dashboard adoption.

Tessara is pre-production. Shared-platform changes therefore advance the
repository, manifests, fixtures, deployment baselines, and consumers together.
Sprint 6D must not preserve an obsolete API, manifest, wire shape, crate, or
test expectation merely to keep an earlier development checkout compatible.
Historical behavior remains recoverable from Git.

## Required Package Graph

The canonical package names are final for Sprint 6D.

| Package | Target and features | Allowed Tessara dependencies | Canonical responsibility |
| --- | --- | --- | --- |
| `tessara-module-contract` | native and `wasm32-unknown-unknown`; no feature-selected runtime | none | Current manifest, Shell Context, signed envelopes, grants/decisions, semantic destinations, typed references, control/health/diagnostic/error wire envelopes, and exact platform version declarations |
| `tessara-module-runtime` | native only | `tessara-module-contract` | Signed-header extraction, verifier construction, standard control/probe routes, correlation/tracing, startup, standard runtime errors, and graceful shutdown |
| `tessara-module-ui` | empty default; `ssr` for native Leptos rendering; `hydrate` for WASM/browser behavior | `tessara-module-contract` | Normalized shell presentation, complete-document SSR, policy-neutral primitives, tokens/base/shell CSS, accessibility, standard states, and versioned assets |
| `tessara-module-testkit` | native test/dev support; it may compile UI hydration fixtures for WASM checks | contract, runtime, and UI | Signing fixtures, manifest/runtime/UI conformance, outage/shutdown helpers, and source/package-boundary assertions |
| `tessara-reference-module-sdk` | empty default; `ssr` enables its native binary, runtime, JSON-state persistence, and UI SSR; `hydrate` enables only its WASM entry and UI hydration | contract and UI in both feature graphs; runtime only with `ssr`; testkit only as a dev-dependency | Non-product conformance manifest, providers, documents, assets, minimal hydration, exact JSON state, and native process |
| `tessara-reference-scoped-records` | native module binary with its existing module-owned product/storage dependencies; UI hydration only where retained by its current build | contract, runtime, UI, and its own product/storage crates; testkit only as a dev-dependency | Existing Scoped Records product behavior and persistence while adopting the canonical runtime/UI |
| root `tessara-web` | existing `ssr`/`hydrate` Core shell application | contract and UI plus Core/product web dependencies | Core adapters, session/product routing, Core navigation policy, and Core-only account actions; it is not a module SDK package |
| `tessara-web-http` | WASM/browser leaf | no Tessara dependency unless a transport wire type requires contract | Policy-neutral browser transport only; no shell, product, runtime, or authorization policy |
| `tessara-web-ui` | deleted in Sprint 6D | none | No facade or transition package remains |

No canonical package may depend directly or transitively on `tessara-api`,
root `tessara-web`, `tessara-core`, `tessara-web-ui`, SQLx, a product crate, or
a module implementation. Runtime does not depend on UI, and UI does not
depend on runtime.

The `tessara-reference-module-sdk` feature graph must not select runtime or any
native persistence dependency for `wasm32-unknown-unknown`. Its `ssr` binary
depends on contract, runtime, and UI in production and testkit only in
development. Scoped Records may additionally depend on its module-owned SQLx
and product infrastructure. `tessara-web-http` remains a separate
policy-neutral browser transport leaf and is not absorbed or re-exported by
module UI.

All policy-neutral primitives move directly to `tessara-module-ui`. The
Dashboard placement editor moves to its Dashboard product/web owner because
its grid behavior is product code. `tessara-web-ui` is then deleted rather
than retained as a compatibility facade.

## Source Disposition

### Contract and verification

- `ShellContextV1`, `SignedEnvelopeV1`, purpose-bound signing/verifying keys,
  validation contexts, and pure signature/context/grant validation remain in
  `tessara-module-contract`.
- Base64 header extraction, environment decoding, construction of the trusted
  Core verifiers, request correlation binding, and HTTP rejection mapping move
  to `tessara-module-runtime`.
- Core remains the only owner of authentication, authorization decisions,
  Organization scope, Shell Context projection, navigation composition, and
  installation/module lifecycle policy.
- Runtime verifies projections and decisions; it never recreates Core policy.

### Shell, primitives, CSS, and assets

- `render_native_module_document` moves from contract to module UI and is
  replaced by the canonical complete-document renderer. The contract crate
  does not re-export it.
- `ShellContentV1`, its media type, its Core fragment bridge, its hydration
  bootstrap, and all associated rendering branches are deleted. There is no
  dual fragment/complete-document path.
- Root `AppShell` remains only as the Core adapter that enforces authentication
  and supplies normalized presentation. The pure frame, sidebar, mobile
  navigation, top bar, theme behavior, normalized navigation presentation,
  and shared accessibility behavior move to module UI.
- Product route resolution, session loading, Core navigation policy, and
  Core-only account actions remain in root web adapters.
- Token, reset, shell, primitive, and state CSS move from the monolithic Core
  stylesheet into canonical module UI sources. Product CSS remains with its
  product owner.
- Each image builds its own content-hashed CSS, JavaScript, WASM, icons, and
  asset manifest from canonical source. Repeated compiled bytes are allowed;
  copied source is not.
- Complete HTML is useful without JavaScript. Hydration may enhance theme and
  mobile navigation but cannot hide content or recovery actions.

### Existing modules

Scoped Records adopts contract, runtime, UI, and the generic complete-document
path in Sprint 6D. Its product routes, markup, configuration rules, SQLx
persistence, migrations, and diagnostic facts remain module-owned.

Dashboard does not adopt runtime or UI in Sprint 6D. Its current
`tessara-dashboard-module -> tessara-web` edge and duplicated startup/control
plumbing remain visible nonconforming findings. They are not an allowed SDK
architecture or compatibility promise; Sprint 6E removes them. Sprint 6D must
not add a facade, alternate renderer, or generic exception that makes this
edge appear conforming.

## Runtime Provider Interfaces

The implementation may refine Rust syntax, but it must preserve these public
responsibilities and data flows.

### `ModuleDefinitionProvider`

Supplies:

- current Module Definition and Module Release identity;
- the current manifest;
- the module-owned product router;
- the module-owned asset manifest and asset bytes.

It does not receive Core state or choose policy from a definition ID.

### `ConfigurationProvider`

Supplies asynchronous operations equivalent to:

- `validate(Value) -> ConfigurationValidationEnvelope`;
- `current() -> Value`;
- `apply(normalized Value) -> Value`.

The provider owns schema semantics, normalization, persistence, concurrency,
and read-back. Runtime owns request authentication, JSON/error plumbing, and
the standard envelope. Runtime calls `apply` only with a value that the same
provider returned as valid and normalized.

### `SecurityStateProvider`

Applies and reads the current standard projection containing:

- schema version;
- installation ID;
- Module Instance ID;
- authorization and Organization revisions;
- enabled state;
- document state.

The provider owns persistence and readiness impact. Runtime owns control-key
authentication, wire validation, and dispatch.

### `ReadinessProvider`

Returns typed checks that explain whether authorized product work is
serviceable. Runtime owns `/health/ready`, status aggregation, correlation, and
the public safe envelope. Runtime owns `/health/live`; a responsive process is
live even when configuration or dependencies make it unready.

### `DiagnosticsProvider`

Returns typed safe facts and findings only. Runtime adds current release,
contract/runtime/UI versions, correlation data, and standard health state.
Diagnostics must not contain secrets, raw errors, browser credentials, signed
envelopes, actor claims, database credentials, or local filesystem paths.

### Runtime errors

The standard error envelope contains:

- current schema version;
- stable code;
- safe message;
- correlation ID;
- retryable boolean.

Configuration findings and health/diagnostic checks use typed subordinate
records. Modules may select documented error codes but cannot expose an
arbitrary serialized error chain.

## Normalized UI Interface

Runtime converts a verified `ShellContextV1` into a normalized presentation
model containing only:

- actor display projection;
- theme, locale, and time zone;
- filtered navigation items and return destination;
- correlation ID;
- document state;
- current destination and document title.

Module UI renders that model plus explicitly trusted module-owned content. It
does not inspect a browser session, query Core, evaluate a capability, or
branch on a Module Definition.

SSR and hydration render the same semantic landmarks and initial state.
Hydration is optional. A hydration failure must leave navigation, content,
diagnostics links, and recovery actions usable.

## Current Manifest And Browser Routing

Sprint 6D replaces `ModuleManifestV1` with one current manifest schema. The
repository updates every production manifest, deployment input, baseline,
import path, and consumer together. Core does not ship a v1 reader,
normalizer, version enum, fallback parser, or feature flag. A development
installation containing the obsolete shape must be reset and bootstrapped
from the current baseline.

The current manifest declares exact versions for:

- Core Release;
- Shell Context schema;
- module control protocol;
- `tessara-module-contract`;
- `tessara-module-runtime`;
- `tessara-module-ui`;
- design-system/asset ABI;
- conformance suite.

It separately records the packages actually linked into the image. This keeps
release inventory truthful: a Dashboard manifest may describe the current
protocol requirements and its linked contract package while the boundary
audit reports its missing runtime/UI adoption for Sprint 6E.

A browser route declaration contains:

- semantic destination;
- absolute same-origin path template;
- `GET`/`HEAD` method set;
- one required capability;
- authorization action and functional contract;
- an optional parameter that identifies the Organization scope.

Manifest validation and receipt import reject duplicate or overlapping
templates, traversal, non-absolute paths, and paths reserved for Core
administration, APIs, or canonical module assets.

Core owns one generic module-document proxy:

1. authenticate the browser session;
2. resolve the accepted current manifest and live Module Instance;
3. match the browser route and typed parameters;
4. evaluate the declared capability and optional Organization scope;
5. synchronize projected module security state;
6. sign short-lived Shell Context and authorization-grant envelopes;
7. remove browser cookies and authorization headers;
8. forward only the signed projections and safe request metadata;
9. return the module's complete document and safe response headers.

Accepted manifest navigation contributions are resolved through this route
registry. Adding the reference module does not add a definition-specific Core
destination, navigation branch, or document renderer.

Immutable module assets use a reserved same-origin path containing definition,
release, and content digest. Asset requests carry no browser credentials.
Complete HTML uses `Cache-Control: no-store`; content-hashed assets use
immutable caching.

An upstream connection failure, timeout, or 5xx produces the Core-owned
authenticated unavailable document with retained shell/navigation and a
Module Management action. Normalized module 4xx responses pass through. Sprint
6D does not introduce generic product API or mutation proxying.

## Reference Module Contract

The minimal fixture is:

- definition `tessara.reference.module-sdk`;
- release `1.0.0`;
- capability `tessara.reference.module-sdk:read`;
- root document `/reference/module-sdk`;
- scoped probe `/reference/module-sdk/scopes/{organization_id}`.

It declares one non-product conformance feature/behavior contract, no product
resource types, no product records, and no cross-module dependency.

The root document requires at least one authorized capability binding. The
scoped probe verifies the requested Organization against the signed grant.
Known and random unauthorized Organization IDs both return the same `403`
`module action unavailable` envelope.

Configuration contains only a trimmed `display_label`. Configuration and the
projected security state are atomically persisted in one exact current JSON
state schema on a module-owned Compose volume. Contexts, grants, actors, keys,
and correlation history are never persisted.

The state command creates or validates the current schema. An incompatible
file fails loudly; the pre-production fixture volume is recreated instead of
running historical migrations.

Readiness requires valid configuration, current projected security state,
enabled operation, and a serviceable document state. Liveness reports only
process health. Diagnostics are sanitized. Shutdown stops accepting work,
finishes/abandons bounded in-flight work, flushes committed state, and exits
within the declared Compose grace period.

The reference image contains its own canonical CSS, minimal hydration entry,
WASM/JavaScript, icons, and content hashes. It builds and tests without root
web, Core API/application state, SQLx, or any product implementation.

## Fast-Forward Version And Security Policy

Canonical packages use SemVer, initially `0.1.0`, but the supported-version
window has width one: the exact tuple declared by the current Core Release.
Before production stability, a breaking change advances the affected version
and updates repository consumers, manifests, baselines, images, and tests in
the same functionality change.

Deprecation is recorded in architecture notes and release history. A
deprecated API is not retained solely to keep an obsolete consumer compiling;
it is removed when repository consumers advance.

Core rejects an exact platform/protocol/package mismatch during manifest
import, bootstrap, enablement, or upgrade. All non-current versions are
unsupported. Release inventory records the exact linked package versions so
operators can identify affected Module Releases.

Critical or high security advisories block the affected version. Moderate and
low advisories remain visible findings. Pre-production has no grandfathered
lifecycle exception for an unsupported or critical/high-vulnerable release;
the module must be rebuilt and redeployed. A running image is never silently
patched or replaced.

## Boundary Enforcement

Native and WASM graph checks enforce the required package graph before source
extraction. Source checks reject:

- imports of Core/root/product packages;
- Core-private DTO or state symbols;
- product Module Definition IDs and product terminology in canonical package
  production source;
- module-definition branches in runtime/UI/testkit;
- copied shell, control-route, verifier-construction, tracing, or shutdown
  implementations.

The Dashboard Sprint 6E debt is reported as a known failing transition, not
placed on the canonical-package allowlist. No other forbidden edge is
accepted.

## Sprint 6D Non-Goals And Sprint 6E Handoff

Sprint 6D does not:

- migrate Dashboard to runtime/UI or remove its root-web dependency;
- move Dashboard route trees, hydration, CSS, or product assets;
- add generic module product API/mutation proxying;
- redesign Core, Dashboard, or Scoped Records product flows;
- introduce a shared SQLx/database adapter;
- implement Blueprint/composition automation;
- extract Components or another product module;
- introduce remote browser code, iframes, or microfrontends;
- retain obsolete development contracts for compatibility.

Sprint 6E receives the current canonical packages, manifest, route/asset
conventions, conformance testkit, and Dashboard transition findings. It must
remove the Dashboard root-web/runtime debt and prove a Dashboard-only image
upgrade and rollback without rebuilding or restarting Core or unrelated
modules.
