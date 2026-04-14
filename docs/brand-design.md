# Tessara Brand And Design

This document is a secondary brand reference for Tessara naming background, positioning, and asset guidance.

The canonical source for active UI implementation guidance is now [ui-guidance.md](./ui-guidance.md).

## Product Name And Positioning

The product name is **Tessara**.

Positioning:

- Tessara is a configurable data platform for structuring, collecting, and analyzing complex hierarchical data.
- The product should read as a platform for structured composition and insight, not as a one-off admin utility.
- Naming and copy should reinforce structure, composition, and clarity rather than legacy-program jargon.

## Naming Guidance

- Use `Tessara` as the root product name everywhere.
- Prefer clear functional labels in the product UI over overly abstract internal branding.
- Keep module and area names consistent with the application information architecture.
- Avoid unnecessary hardcoded legacy labels when configurable terminology is available.

Useful naming posture:

- product name: `Tessara`
- asset names: `Dataset`, `Component`, `Dashboard`
- platform tone: structured, precise, modular, not playful

## Brand Concept

The visual identity is built around a tessera/tesseract-inspired cube:

- modular geometry
- composition from smaller pieces
- one structured whole assembled from distinct parts

This should reinforce the product's architectural themes:

- organization and hierarchy
- configurable forms and responses
- analytical composition into higher-level views

## Visual System

### Core palette

| Name | Hex | Usage |
| --- | --- | --- |
| Ink | `#0F172A` | text, outlines, code/pre backgrounds |
| Slate Dark | `#334155` | secondary text, darker surface accents |
| Slate Mid | `#64748B` | muted text and helper information |
| Neutral | `#E2E8F0` | borders and light structural accents |
| Light | `#F8FAFC` | page background |
| Surface | `#FFFFFF` | cards, panels, inputs |
| Teal | `#14B8A6` | primary action, primary accent |
| Orange | `#F59E0B` | focus outline and highlight accent |
| Lime | `#84CC16` | secondary workflow accent |

### UI styling rules

- Prefer light application surfaces with `Light` backgrounds and `Surface` cards.
- Use `Ink` for primary text and `Slate Mid` for muted text.
- Use `Teal` for primary buttons and major action emphasis.
- Use `Orange` for keyboard focus outlines.
- Use `Teal`, `Lime`, and `Orange` deliberately; do not introduce unrelated accent colors without updating this document.
- Use rounded modular cards and panels rather than heavy boxed layouts.
- Tessara supports both light and dark themes through the same shared shell and component system.
- The dark theme must use only this approved palette plus opacity variants; do not introduce new hues without updating this document.
- In dark theme, use `Ink` for shell background, `Slate Dark` for surfaces, `Light` for primary foreground text, `Slate Mid` for muted text, `Neutral` for borders, `Teal` for links and primary actions, `Orange` for focus, and `Lime` for secondary accent states.

## Icon And Wordmark Guidance

Brand assets already tracked in the repo:

- [tessara-favicon-16.svg](../crates/tessara-web/assets/tessara-favicon-16.svg)
- [tessara-favicon-32.svg](../crates/tessara-web/assets/tessara-favicon-32.svg)
- [tessara-favicon-64.svg](../crates/tessara-web/assets/tessara-favicon-64.svg)
- [tessara-favicon-mono.svg](../crates/tessara-web/assets/tessara-favicon-mono.svg)
- [tessara-icon-256.svg](../crates/tessara-web/assets/tessara-icon-256.svg)
- [tessara-icon-512.svg](../crates/tessara-web/assets/tessara-icon-512.svg)
- [tessara-icon-1024.svg](../crates/tessara-web/assets/tessara-icon-1024.svg)
- [tessara-wordmark.svg](../crates/tessara-web/assets/tessara-wordmark.svg)

Usage rules:

- keep one accent tile per face in the brand mark
- use simplified favicons at small sizes
- avoid distorting icon proportions
- keep sufficient contrast on light backgrounds
- treat application CSS tokens as the implementation expression of this palette

## Application Implications

Apply these brand decisions through [ui-guidance.md](./ui-guidance.md):

- the shell should read as a product, not a utility console
- typography, spacing, and card structure should support calm inspection and modular authoring
- product and internal areas should feel visually related but clearly distinct in emphasis
- naming in navigation, detail pages, and editors should align with the target model and avoid stale asset language

## Metadata And Asset Integration

When app metadata is updated, prefer:

- route-specific `meta name="description"`
- `meta name="theme-color" content="#F8FAFC"`
- `meta name="color-scheme" content="light dark"`
- Open Graph and Twitter summary metadata that references the 512px icon
- SVG favicon links for the tracked favicon assets

## Relationship To Historical Sources

This file preserves useful naming, palette, icon, and positioning guidance as supporting brand context. For active UI decisions, use [ui-guidance.md](./ui-guidance.md).
