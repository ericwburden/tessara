# Prototype Instructions

Run the local server yourself and open the preview in the browser available to this environment. Do not give the user server-start instructions when you can run it.

Before making substantial visual changes, use the Product Design plugin's `get-context` skill when the visual source is unclear or no longer matches the current goal. When the user gives durable prototype-specific design feedback, preferences, or decisions, record them in `AGENTS.md`.

When implementing from a selected generated mock, treat that image as the source of truth for layout, component anatomy, density, spacing, color, typography, visible content, and hierarchy.

Build app UI in `src/`. Keep `.openai/hosting.json`, `worker/index.js`, `scripts/prepare-sites-build.mjs`, and `tests/sites-worker.test.mjs` intact so the same local prototype can be handed to Sites. Before a Sites handoff, run `npm run build` and `npm run test:sites`; the build must leave `dist/client/index.html`, `dist/server/index.js`, and `dist/.openai/hosting.json`.

## Tessara Sprint 6B2 Review Contract

- The product owner identified this suite as Tessara's current UI-prototype quality benchmark. Preserve its evidence-first workflow and follow `docs/ui-prototype-review-standard.md` for future UI-bearing sprints.
- Match the running Tessara application rather than redesigning it.
- Preserve the dark slate shell, teal actions, compact bordered tables, status badges, native-feeling forms, and existing responsive patterns.
- Treat every screen as a bounded Sprint 6B2 delta. Existing shell navigation, header actions, spacing, icons, and table patterns remain the baseline.
- Scoped Records is a deliberately small reference/conformance module: organization-owned labeled records, separate scoped read/manage authority, configuration validation, health, and diagnostics.
- Administrator enrollment is a bare route outside the authenticated shell and must remain visually distinct from ordinary sign-in.
- Keep administrator enrollment visually direct: the Tessara brand is followed by the page heading, without a redundant enrollment badge or kicker.
- A module's validated `display_label` is the authoritative shell-navigation label; the descriptor label is only its fallback.
- Roles list and detail panels must remain contained by the route panel at intermediate desktop widths, not just at mobile and wide-desktop breakpoints.
- Prototype interactions are review aids only; do not imply production persistence or completed backend behavior.
- In directory tables, the primary text link is the sole detail affordance; do not add a duplicate trailing arrow action.
- Use one visual treatment within a semantic table column; the Roles Enrollment column uses text for every value.
- Record creation and record editing are separate routes and screens, not modes of one record-specific form.
- Diagnostic probe names are the card headings, statuses appear once, and tabbed content retains a clear section gap below the divider.
