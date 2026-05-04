# Tessara Canonical Docs

This `/docs` folder is the authoritative source for Tessara's active roadmap, requirements, architecture, and UI guidance.

## Canonical Files

| File | Role |
| --- | --- |
| [roadmap.md](./roadmap.md) | Current implementation baseline and forward-looking delivery plan |
| [requirements.md](./requirements.md) | Product and system requirements |
| [architecture.md](./architecture.md) | Target architecture, transition model, and technical design direction |
| [development-workflow.md](./development-workflow.md) | Local development loops for fast host-run iteration, API-only refresh, and full-stack relaunch |
| [ui-guidance.md](./ui-guidance.md) | Canonical UI guidance for naming, brand expression, shell behavior, rendering, layout, components, states, shared primitive contracts, and transitional UI rules |
| [ui-guidance-spec.md](./ui-guidance-spec.md) | Allium behavioral specification companion to the canonical UI guidance |

## Authority Rules

- Treat this folder as the only active planning and design authority.
- If a historical document outside `/docs` disagrees with a file in `/docs`, the file in `/docs` wins.
- Current implementation references under `re-alignment/db/`, `re-alignment/rust/`, and `progress-report.md` remain useful inputs, but they are not canonical project direction.
- The target analytical asset model is `Dataset -> Component -> Dashboard`.
- Current code paths that still use `Report`, `Aggregation`, or `Chart` are transitional implementation details, not the target model.

## Current Direction

The active direction for Tessara is:

- preserve the implemented baseline as of May 3, 2026
- transition from the current reporting stack into the `Dataset -> Component -> Dashboard` model
- use a `cargo-leptos` SSR-first frontend pipeline with selective hydration and selective lazy loading
- plan future delivery as explicit vertical-slice sprints
- require every sprint to leave the application user-testable through usable application UI

Start here if you are orienting:

1. Read [roadmap.md](./roadmap.md) for current status and next sprints.
2. Read [requirements.md](./requirements.md) for scope and system expectations.
3. Read [architecture.md](./architecture.md) for the target model and transition rules.
4. Read [ui-guidance.md](./ui-guidance.md) for the canonical UI specification.
5. Read [ui-guidance-spec.md](./ui-guidance-spec.md) when you need the formal Allium behavior contract for the UI guidance.

## Reference Inputs

These remain useful but are not active direction documents:

- [progress-report.md](./progress-report.md)
- [re-alignment/db](./re-alignment/db)
- [re-alignment/rust](./re-alignment/rust)
- [../README.md](../README.md)

## Historical Sources

Some older planning and design sources were consolidated into the canonical files above and are not present in this checkout. Treat `roadmap.md`, `requirements.md`, `architecture.md`, `ui-guidance.md`, and `ui-guidance-spec.md` as the active replacements for archived planning material.
