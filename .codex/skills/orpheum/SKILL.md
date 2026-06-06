---
name: orpheum
description: Use the local Orpheum CLI and session files to discover scenarios, apply them, inspect state, and run checks without inferring workflow state from prose.
---

# Orpheum

Use this skill when working in a project that has the `orpheum` CLI available and may use Orpheum scenarios for AI-assisted SDLC work.

This file is installed by `orpheum init` and represents the local agent contract for Orpheum usage in a consumer project.

## Purpose

This skill teaches the local operating contract for Orpheum in a consumer project.

Use Orpheum to:

- discover available scenarios
- inspect scenario structure before applying it
- apply one scenario into the current project
- read authoritative session state from `.orpheum/`
- generate the current recommended prompt
- run scenario checks

## Authoritative Sources

Prefer these sources in order:

1. `orpheum ... --json`
2. `.orpheum/session.json`
3. `.orpheum/scenario.json`
4. `.orpheum/state.json`
5. `.orpheum/logs/checks.json`

Treat these as derived views rather than source of truth:

- `.orpheum/ACTIVE.md`
- `.orpheum/prompts/current.md`
- surrounding catalog prose
- stale chat context

## Core Rules

- Prefer `orpheum ... --json` when you need authoritative machine-readable state.
- Treat `.orpheum/session.json`, `.orpheum/scenario.json`, and `.orpheum/state.json` as authoritative state.
- Treat `.orpheum/prompts/current.md` and `orpheum prompt current` as derived guidance, not source of truth.
- Do not infer scenario dependencies from prose files when the CLI or session JSON can tell you directly.
- Do not create `.orpheum/` by hand. Use `orpheum scenario apply <id>`.
- Run `orpheum check run` before claiming scenario-associated outputs are ready.
- Use `orpheum session finalize --json` as the explicit lifecycle step that marks a completed scenario finalized.

## What `orpheum init` Does

`orpheum init` is the project-onboarding step for agents.

It:

- installs this local skill at `.codex/skills/orpheum/SKILL.md`
- refreshes the project to use the embedded Orpheum catalog by default
- persists an external catalog override in `.codex/orpheum/config.json` only when one is explicitly selected or discovered
- writes a repo-root `ORPHEUM.md` onboarding file
- appends `.orpheum/` to an existing `.gitignore` if that line is missing

It does not:

- apply a scenario
- create `.orpheum/` session files
- infer or select an active scenario
- create a new `.gitignore` when none exists

## Normal Command Loop

1. Discover scenarios:
   - `orpheum scenario list --json`
2. Inspect one scenario:
   - `orpheum scenario show <id> --json`
3. Apply a scenario to the current project:
   - `orpheum scenario apply <id> --json`
4. Inspect current session state:
   - `orpheum status --json`
5. Get the current recommended prompt:
   - `orpheum prompt current --json`
6. Run validation checks:
   - `orpheum check run --json`
7. Finalize the session when checks pass and the scenario is complete:
   - `orpheum session finalize --json`
8. Close a finalized session safely when ready:
   - `orpheum session close --json`
9. Diagnose catalog or project setup:
   - `orpheum doctor --json`

## Session Files

When a scenario is active, expect these files under `.orpheum/`:

- `ACTIVE.md`
- `session.json`
- `scenario.json`
- `state.json`
- `prompts/current.md`
- `logs/checks.json`

Use them like this:

- `session.json`: session identity and project-local control metadata
- `scenario.json`: resolved snapshot of the applied scenario
- `state.json`: mutable progress, workflow state, artifact state, and check state
- `logs/checks.json`: latest check report

## Decision Guidance

- If there is no active session, use `orpheum scenario list` or `orpheum scenario show`, then apply a scenario if appropriate.
- If a session exists, start with `orpheum status --json`.
- If check status is unclear or stale, run `orpheum check run --json`.
- If `status --json` reports `finalize_ready`, run `orpheum session finalize --json`.
- If a session is finalized and `cleanup_ready` is true, run `orpheum session close --json` before applying a new scenario.
- If the environment looks misconfigured, run `orpheum doctor --json`.
- During semantic artifact review for discovery or planning scenarios, use Planning Mode or the host environment's nearest equivalent before changing the artifact set.

## What Not To Do

- Do not guess the active scenario from nearby docs when `.orpheum/scenario.json` exists.
- Do not treat stale chat context as authoritative when current session files disagree.
- Do not silently skip scenario checks.
- Do not treat `.orpheum/` as the durable home for finished project artifacts.
