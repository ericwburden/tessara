# Independent Module Pathway

Status: Sprint 6D current module-authoring contract. Dashboard completes its
adoption in Sprint 6E.

The non-product `tessara.reference.module-sdk` release is the canonical
source/build conformance fixture. Scoped Records proves adoption while
retaining real product persistence. Dashboard proves the independent
process/database boundary but remains explicitly nonconforming at the
source/runtime edge until Sprint 6E. A later module must use the same generic
path without adding a definition-ID branch to Core or Module Management.

Sprint 6C proved the independent process, database, control, product, and
operational boundary. Sprint 6D makes the canonical source/build contract
concrete in the
[Module SDK Implementation Contract](./module-sdk-implementation-contract.md).
Dashboard still links root `tessara-web`; that explicit nonconforming finding
is removed in Sprint 6E and must not be copied by later modules.

## Shared Boundary

An independently deployed module owns:

- its product API, native SSR product pages, data, migration, runtime, and
  database identities;
- its route tree, hydration/SSR entrypoints, and versioned frontend assets,
  built from canonical module runtime/UI SDK source without linking the Core
  application or root web application;
- its manifest, configuration schema, configuration validator, readiness,
  liveness, status, and diagnostics;
- its product capabilities, routes, navigation, resources, contracts, and
  dependencies;
- applying normalized configuration and Core-projected enablement/security
  state.

Every new module uses the sole current manifest and exact current
contract/runtime/UI tuple. Pre-production authoring does not target an older
manifest or SDK window.

Core owns:

- installation, actor, authorization, release and instance inventory, desired
  configuration and enablement, navigation projection, and same-origin
  gatewaying;
- rendering the shared Module Management template from the persisted manifest
  and instance observation;
- exchanging short-lived, audience-bound authorization grants.

Core does not own module product tables, module configuration types, or
definition-specific Module Management components.

## Required Module Runtime And Control Contract

Every independently deployed module implements the canonical runtime provider
interfaces for definition/routes/assets, configuration, projected security
state, readiness, and sanitized diagnostics. The shared runtime owns request
authentication, protocol validation, control/probe routes, tracing, standard
errors, startup, and shutdown; module providers own semantics and persistence.

Every module registered in `TESSARA_MODULE_CONTROL_ENDPOINTS` exposes the same
private service protocol:

| Method | Path | Responsibility |
| --- | --- | --- |
| `POST` | `/api/configuration/validate` | Validate input and return `{valid, normalized, findings}`. |
| `PUT` | `/api/configuration` | Apply normalized configuration; requires `x-tessara-module-control-key`. |
| `PUT` | `/api/private/security-state` | Apply installation, instance, revision, enablement, and document state. |
| `GET` | Manifest readiness path | Report whether the module can serve authorized product work. |
| `GET` | Manifest liveness path | Report process health without exposing secrets. |

Core selects the endpoint by definition ID from the deployment registry. The
registry is deployment wiring, not product behavior:

```json
{
  "tessara.reference.scoped-records": "http://scoped-records:8090",
  "tessara.dashboards": "http://dashboards:8091",
  "example.third-module": "http://third-module:8092"
}
```

The two legacy per-module URL variables remain temporary compatibility
fallbacks. New modules must use the registry.

## Managed Configuration Contract

`manifest.configuration_schema` is the sole source of Module Management
configuration fields. The shared renderer supports:

- `string`, including `enum`;
- `integer` and `number`, including minimum and maximum;
- `boolean`;
- JSON Schema `title` for an explicit UI label, otherwise a label derived from
  the property name;
- `required`.

Core coerces the submitted HTML form according to the persisted schema,
rejects unknown or unsupported fields, sends the result to the module-owned
validator, persists only the validator's normalized value, and verifies that
the module applied that exact normalized value.

Module-specific values belong in the schema and current configuration. They do
not justify a Core branch. Complex configuration that cannot fit the supported
declarative field set requires a versioned shared control extension before a
module adopts it.

## Shared Module Management Template

Every independent module receives the same:

- directory serving-state vocabulary;
- overview and lifecycle assessment;
- schema-driven configuration display and editor;
- configuration, health, navigation, and enablement state;
- enable/disable control;
- in-product health and diagnostics surface;
- manifest-driven dependency assessment;
- declaration, contract, capability, dependency, resource, and navigation
  sections;
- sanitized diagnostics download.

Definition-specific display names, property values, dependencies, routes,
capabilities, and other manifest facts are data. Definition-specific control
flow or markup is a conformance failure.

## Adoption Checklist

1. Create a module crate/service with its own database baseline and distinct
   owner, migration, and runtime identities.
2. Publish the sole current manifest, including exact platform/SDK versions,
   browser routes, configuration schema, operational routes, capabilities,
   navigation, assets, and deployment probes.
3. Implement module-owned providers behind `tessara-module-runtime`; do not
   copy the shared configuration, security-state, probe, tracing, startup, or
   shutdown implementation.
4. Register the service endpoint in `TESSARA_MODULE_CONTROL_ENDPOINTS`.
5. Materialize the Module Release and Module Instance through the deployment
   receipt/bootstrap path.
6. Route approved GET/HEAD documents and immutable assets through the generic
   manifest-driven Core seam. Product APIs remain explicitly owned/routed by
   the module until a later generic API contract exists.
7. Add the module to the parameterized independent-module acceptance fixture;
   do not add a module-specific Module Management test branch.
8. Pass manifest, configuration, enablement, navigation, diagnostics,
   no-JavaScript, outage-containment, and database-isolation gates.
9. Pass native/WASM source/package-graph checks proving the module does not depend on the
   Core application binary, root `tessara-web`, Core-private DTOs, or another
   module implementation; repeated compiled SDK/runtime code and assets are
   allowed.
10. Prove an image-only module upgrade and rollback while Core, gateway, and
   unrelated module image digests remain unchanged.
11. Search Core and the web shell for the new definition ID. Matches in
    fixtures, seed, or routing registration must be explainable; Module
    Management and control-plane branches are not allowed.
12. When the current manifest or SDK tuple advances before production, update
    and rebuild the module. Do not add an old-manifest reader, compatibility
    facade, or deprecated API to avoid that update.

## Conformance Rule

The pathway is reusable only while this check remains true:

> Adding a conforming third module requires deployment registration and
> module-owned implementation built on the canonical SDK/runtime, but no
> definition-specific changes to Core's configuration, enablement,
> diagnostics, Module Management rendering, or root web application and no
> dependency from the module back to those Core implementations.

The Sprint 6D reference module is the conformance fixture for this statement.
Dashboard remains an expected failing transition until Sprint 6E.
