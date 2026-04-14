# Tessara Design Guide

## Overview
Tessara's visual identity is built around a tessera/tesseract-inspired cube:

- A 3D isometric cube composed of square tesserae.
- One colored tile per visible face to represent multi-dimensional data.
- Clean, modular geometry that scales from app icon to favicon.

## Icon System
### App Icon
- Isometric cube with 2x2 tiles per face.
- One colored tile per face:
- Top face: Teal.
- Left face: Lime.
- Right face: Orange.
- Remaining tiles are neutral, consistent per face.
- Use a subtle Ink outline for clarity.

### Favicon
- No internal grid or tile divisions.
- Each face is a single solid color.
- Use the same color mapping as the app icon.
- Use a slightly thicker outline for legibility at 16-32px.

### Wordmark
- Icon paired with "Tessara".
- Typeface: Inter, with system sans-serif fallback.
- Medium/Semibold weight.
- Tight letter spacing.

## Color Palette
| Name | Hex | Usage |
| --- | --- | --- |
| Ink | `#0F172A` | Text, outlines, code/pre backgrounds |
| Slate Dark | `#334155` | Secondary text, right face base |
| Slate Mid | `#64748B` | Muted text, left face base |
| Neutral | `#E2E8F0` | Borders, top face base |
| Light | `#F8FAFC` | App background, icon background |
| Surface | `#FFFFFF` | Panels, cards, inputs |
| Teal | `#14B8A6` | Primary action, top accent tile |
| Orange | `#F59E0B` | Focus ring, right accent tile |
| Lime | `#84CC16` | Secondary workflow accent, left accent tile |

## Geometry And Spacing
- Cube uses isometric projection with 30-degree axes.
- Faces are diamonds composed of 4 points.
- App icon tiles use a 2x2 grid per face.
- Favicons have no subdivisions.
- App icon container radius should be roughly 18-20% of size.
- Application panels should use rounded, modular blocks that visually echo the tesserae system.

## Line And Shading
- Outline color: Ink (`#0F172A`).
- App icon stroke width: roughly 1-1.2% of canvas size.
- Favicon stroke width: roughly 5-7% of icon size.
- Face shading:
- Top: lightest.
- Left: mid.
- Right: darkest.

## Application UI Styling
- Prefer a light application surface using `#F8FAFC` as the page background and `#FFFFFF` for cards and panels.
- Use Ink for primary text and Slate Mid for muted helper text.
- Use Teal for primary buttons and link pills.
- Use Orange for keyboard focus outlines so focus states are visible but remain in the Tessara palette.
- Use Teal, Lime, and Orange as workflow accents. Do not introduce unrelated accent colors unless the design guide is updated first.
- Avoid dense rainbow status systems. If status color is needed, map it deliberately to existing palette colors or add a documented semantic token.
- Use rounded cards and panels with subtle shadows to keep the interface modular without looking heavy.

## Web Asset Integration
The web app serves tracked SVG brand assets from:

- `/assets/tessara-favicon-16.svg`
- `/assets/tessara-favicon-32.svg`
- `/assets/tessara-favicon-64.svg`
- `/assets/tessara-favicon-mono.svg`
- `/assets/tessara-icon-256.svg`
- `/assets/tessara-icon-512.svg`
- `/assets/tessara-icon-1024.svg`
- `/assets/tessara-wordmark.svg`

Document heads should include:

- `meta name="description"` with a route-specific description.
- `meta name="theme-color" content="#F8FAFC"`.
- `meta name="color-scheme" content="light"`.
- Open Graph and Twitter summary metadata that references `/assets/tessara-icon-512.svg`.
- SVG favicon links for 16, 32, and 64 sizes.
- A monochrome mask icon link using Ink as the mask color.
- An Apple touch icon link using the 256px app icon.

## Usage Guidelines
### Do
- Keep one accent tile per face.
- Maintain the palette above across app and brand assets.
- Use simplified favicons at small sizes.
- Ensure sufficient contrast on light backgrounds.
- Treat CSS variables in the web app as the canonical implementation tokens.

### Don't
- Do not add more colored tiles.
- Do not use full rainbow palettes.
- Do not include internal grid lines in favicons.
- Do not distort cube proportions.
- Do not add a dark theme until the contrast and brand usage are specified.

## Files Included
### App Icons
- `tessara-icon-1024.svg`
- `tessara-icon-512.svg`
- `tessara-icon-256.svg`

### Favicons
- `tessara-favicon-64.svg`
- `tessara-favicon-32.svg`
- `tessara-favicon-16.svg`
- `tessara-favicon-mono.svg`

### Wordmark
- `tessara-wordmark.svg`

## Concept Summary
Tessara represents a system where individual pieces, or tesserae, form a structured whole. Each colored tile signals a dimension of data, unified through clean geometry and thoughtful composition.
