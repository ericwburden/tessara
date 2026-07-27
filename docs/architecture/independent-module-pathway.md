# Independent Module Pathway

Status: Sprint 6C reference contract.

Dashboard and Scoped Records are the two conformance fixtures for this
pathway. A third module must be able to reach the same degree of modularity
without adding a definition-ID branch to Core or Module Management.

## Shared Boundary

An independently deployed module owns:

- its product API, native SSR product pages, data, migration, runtime, and
  database identities;
- its manifest, configuration schema, configuration validator, readiness,
  liveness, status, and diagnostics;
- its product capabilities, routes, navigation, resources, contracts, and
  dependencies;
- applying normalized configuration and Core-projected enablement/security
  state.

Core owns:

- installation, actor, authorization, release and instance inventory, desired
  configuration and enablement, navigation projection, and same-origin
  gatewaying;
- rendering the shared Module Management template from the persisted manifest
  and instance observation;
- exchanging short-lived, audience-bound authorization grants.

Core does not own module product tables, module configuration types, or
definition-specific Module Management components.

## Required Module Control Contract

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
2. Publish a valid `ModuleManifestV1`, including the configuration schema,
   operational routes, capabilities, navigation, and deployment probes.
3. Implement the shared configuration and security-state endpoints.
4. Register the service endpoint in `TESSARA_MODULE_CONTROL_ENDPOINTS`.
5. Materialize the Module Release and Module Instance through the deployment
   receipt/bootstrap path.
6. Route only approved same-origin product/API paths through the gateway.
7. Add the module to the parameterized independent-module acceptance fixture;
   do not add a module-specific Module Management test branch.
8. Pass manifest, configuration, enablement, navigation, diagnostics,
   no-JavaScript, outage-containment, and database-isolation gates.
9. Search Core and the web shell for the new definition ID. Matches in
   fixtures, seed, routing registration, or explicit compatibility adapters
   must be explainable; Module Management and control-plane branches are not
   allowed.

## Conformance Rule

The pathway is reusable only while this check remains true:

> Adding a conforming third module requires deployment registration and
> module-owned implementation, but no definition-specific changes to Core's
> configuration, enablement, diagnostics, or Module Management rendering.

