# Tessara UI Prototype Review Standard

This document defines the required review workflow for sprint work that adds or materially changes Tessara screens. Its quality reference is the Sprint 6B2 review suite under [`docs/sprints/sprint-6b2-ui-review/`](./sprints/sprint-6b2-ui-review/).

The standard exists to make proposed UI changes concrete before production implementation while preserving the visual and behavioral continuity of the running application.

## Core Principle

The current running Tessara application is the primary visual baseline. The canonical [`ui-guidance.md`](./ui-guidance.md) and [`ui-guidance-spec.md`](./ui-guidance-spec.md) define the durable product rules. A sprint prototype proposes bounded deltas to those sources; it is not permission to redesign unrelated surfaces.

Every review should make three things easy to distinguish:

1. what Tessara already does
2. what the sprint proposes to add or change
3. what explicitly remains unchanged

## Required Workflow

### 1. Inspect the current product

Before designing:

- run the current application and inspect the closest analogous shell, routes, forms, tables, detail views, empty states, errors, and responsive behavior
- inspect the source styles, tokens, shared components, icons, and assets that create those surfaces
- capture dated reference screenshots at consistent, recorded viewport sizes
- record mismatches between the running UI and canonical UI guidance rather than silently normalizing them in the prototype

Do not rely on memory, generic dashboard conventions, or an earlier mockup when the current application can be inspected.

### 2. Define a bounded screen inventory

Create a screen-and-state matrix before implementation. Include:

- each new or edited route
- loading, empty, success, validation, denied, disabled, degraded, stale, and recovery states that are material to the sprint
- desktop, tablet, and mobile coverage
- light and dark themes when the affected production surface supports both
- long labels, identifiers, and other realistic overflow risks

The matrix may mark a combination not applicable, but it must not leave important states implicit.

### 3. Write per-screen delta records

For every proposed screen, record:

- current analogous screen or pattern
- proposed additions
- proposed changes
- explicitly preserved shell, navigation, layout, component, icon, and interaction behavior
- responsive behavior
- important authorization, lifecycle, or failure states
- unresolved product decisions

Approval applies only to the recorded deltas. Unmentioned parts of Tessara remain governed by the current application and canonical guidance.

### 4. Build a runnable review suite

The prototype must:

- mirror the running application's typography, density, spacing, colors, borders, controls, tables, badges, shell, and responsive patterns
- reuse actual product assets and icons when available
- use realistic Tessara vocabulary and representative data
- make primary journeys and important state changes interactive
- keep any review navigator or scenario switcher visibly separate from proposed production UI
- avoid implying backend persistence, security enforcement, or lifecycle behavior that the prototype does not implement
- live outside production application code unless the sprint explicitly requires an implementation spike

The prototype technology is not part of the product contract. Choose the smallest runnable implementation that supports faithful review.

### 5. Perform blocking design QA

Before presenting the prototype:

- capture every review state at its specified viewport
- compare prototype captures with the current-product references and delta records
- inspect typography, spacing, alignment, color, borders, assets, icons, copy, overflow, and responsive behavior
- exercise primary controls using both pointer and keyboard
- inspect the browser console
- verify the prototype build and its repository-owned tests
- fix all known P0, P1, and P2 visual or interaction issues

Record the result in `design-qa.md`. The report must identify the tested states, viewports, commands, remaining lower-severity differences, and end with either `final result: passed` or `final result: failed`.

A failed or missing design-QA report blocks UI approval.

### 6. Hand off for explicit approval

The review handoff must include:

- a live local preview when available
- a compact screenshot set or contact sheet
- the current-UI review
- the screen-and-state inventory
- the per-screen delta records
- the design-QA report

Product-owner approval is required before production UI implementation begins. Quality-process praise does not by itself approve every proposed product decision; record screen approval explicitly.

## Accessibility And Resilience Checks

For affected production patterns, the review must cover:

- visible keyboard focus and logical focus order
- accessible names for icon-only controls
- useful heading and landmark structure
- color-independent status communication
- 200% browser zoom without hidden actions or destructive overlap
- long content and narrow-width containment
- native SSR or no-JavaScript usefulness when the route is required to support it

Prototype-only limitations must be documented rather than mistaken for approved production behavior.

## Standard Artifact Layout

Use this structure for a UI-bearing sprint:

```text
docs/sprints/<sprint>-ui-review/
├── current-ui-review.md
├── screen-delta-records.md
├── reference/
│   └── current-product screenshots
└── prototype/
    ├── AGENTS.md
    ├── design-qa.md
    ├── screenshots/
    └── runnable prototype source
```

Keep accepted evidence in the repository so a future implementer can reproduce the design intent without relying on a conversation transcript.

## Why Sprint 6B2 Is The Reference

The Sprint 6B2 suite established the benchmark by:

- inspecting and capturing the live Tessara shell and analogous screens first
- preserving the existing visual language instead of introducing a parallel design system
- describing each proposal as a bounded screen delta
- covering the full workflow and its security/lifecycle states in one runnable suite
- separating the review controls from the proposed product UI
- completing screenshot-based visual QA, interaction checks, console checks, build verification, and repository-owned tests before handoff

Future prototypes may improve on this benchmark, but they should not omit those qualities without recording why a requirement is inapplicable.
