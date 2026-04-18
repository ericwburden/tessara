# Tessara Canonical Docs

This `/docs` folder is the authoritative source for Tessara's active roadmap, requirements, architecture, UI guidance, and supporting brand/design references.

## Canonical Files

| File | Role |
| --- | --- |
| [roadmap.md](./roadmap.md) | Current implementation baseline and forward-looking delivery plan |
| [requirements.md](./requirements.md) | Product and system requirements |
| [architecture.md](./architecture.md) | Target architecture, transition model, and technical design direction |
| [development-workflow.md](./development-workflow.md) | Local development loops for fast host-run iteration, API-only refresh, and full-stack relaunch |
| [ui-guidance.md](./ui-guidance.md) | Canonical UI guidance for naming, shell behavior, rendering, layout, components, states, and transitional UI rules |
| [brand-design.md](./brand-design.md) | Secondary brand background and asset reference for Tessara identity materials |
| [ui-direction.md](./ui-direction.md) | Compatibility pointer to the canonical UI guidance |

## Authority Rules

- Treat this folder as the only active planning and design authority.
- If a historical document outside `/docs` disagrees with a file in `/docs`, the file in `/docs` wins.
- Current implementation references under `re-alignment/db/`, `re-alignment/rust/`, and `progress-report.md` remain useful inputs, but they are not canonical project direction.
- The target analytical asset model is `Dataset -> Component -> Dashboard`.
- Current code paths that still use `Report`, `Aggregation`, or `Chart` are transitional implementation details, not the target model.

## Current Direction

The active direction for Tessara is:

- preserve the implemented baseline as of April 12, 2026
- transition from the current reporting stack into the `Dataset -> Component -> Dashboard` model
- use a `cargo-leptos` SSR-first frontend pipeline with selective hydration and selective lazy loading
- plan future delivery as explicit vertical-slice sprints
- require every sprint to leave the application user-testable through usable application UI

Start here if you are orienting:

1. Read [roadmap.md](./roadmap.md) for current status and next sprints.
2. Read [requirements.md](./requirements.md) for scope and system expectations.
3. Read [architecture.md](./architecture.md) for the target model and transition rules.
4. Read [ui-guidance.md](./ui-guidance.md) for the canonical UI specification.
5. Read [brand-design.md](./brand-design.md) for brand background and asset references when needed.

## Reference Inputs

These remain useful but are not active direction documents:

- [progress-report.md](./progress-report.md)
- [re-alignment/db](./re-alignment/db)
- [re-alignment/rust](./re-alignment/rust)
- [../README.md](../README.md)

## Archived Sources

Historical planning and design sources have been moved to `archive/docs/`.

| Archived File | Replacement |
| --- | --- |
| [archive/docs/roadmap.md](./archive/docs/roadmap.md) | [roadmap.md](./roadmap.md) |
| [archive/docs/blueprint.md](./archive/docs/blueprint.md) | [requirements.md](./requirements.md), [architecture.md](./architecture.md) |
| [archive/docs/ui-direction.md](./archive/docs/ui-direction.md) | [ui-guidance.md](./ui-guidance.md) |
| [archive/docs/user-interface-design.md](./archive/docs/user-interface-design.md) | [ui-guidance.md](./ui-guidance.md) |
| [archive/docs/tessara_design_guide.md](./archive/docs/tessara_design_guide.md) | [ui-guidance.md](./ui-guidance.md), [brand-design.md](./brand-design.md) |
| [archive/docs/tessara_naming.md](./archive/docs/tessara_naming.md) | [ui-guidance.md](./ui-guidance.md), [brand-design.md](./brand-design.md) |
| [archive/docs/tessara_dataset_addendum.md](./archive/docs/tessara_dataset_addendum.md) | [architecture.md](./architecture.md) |
| [archive/docs/re-alignment](./archive/docs/re-alignment) | [roadmap.md](./roadmap.md), [architecture.md](./architecture.md) |
