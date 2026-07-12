# Tessara Component Editor UX Guide

## Recommendation

Use one Component editor route and lifecycle shell, with a separate typed sub-editor for each Component kind.

The shared shell owns:

- Component identity and description
- Dataset and major-line binding
- Dataset context
- Component-kind selection
- validation findings returned by save attempts
- the existing Save Draft and Save and Publish version actions
- preview placement

The kind-specific panel owns only the relevant configuration and preview. Do not build one generic form that conditionally hides dozens of unrelated individual controls.

Suggested implementation shape:

```text
ComponentEditor
├── ComponentIdentity
├── DatasetBinding
├── ComponentKindPicker
├── KindConfigEditor
│   ├── TableConfigEditor
│   ├── BarConfigEditor
│   ├── LineConfigEditor
│   ├── PieDonutConfigEditor
│   └── StatCardConfigEditor
├── ComponentPreview
└── ComponentLifecycleActions
```

## Save and validation behavior

Preserve the current deployed footer:

- `Save Draft`
- `Save and Publish` with `Update Existing Version` and `Create New Version`

Do not add a separate Validate action. Both draft and publish saves run validation as part of the save request. When validation fails, keep the author on the editor, preserve their entered values, render the returned findings near the relevant controls, and do not complete the requested lifecycle transition.

The preview may still show whether the current configuration appears valid, but that status is informational and must not imply a separate validation workflow.

## Bar editor structure

Present controls in the order authors form the question:

1. Category: what should each group of bars represent?
2. Series: should each category be split for comparison?
3. Measure: what determines bar length and how is it calculated?
4. Filters: which Dataset rows participate?
5. Order and display: how should the resulting categories be presented?
6. Labels and colors: optional polish after the data mapping is valid.

Use author-facing labels rather than storage terminology:

| UI label | Contract concept |
| --- | --- |
| Category field | category field |
| Split bars | presence of series field |
| Series field | comparison/series field |
| Calculation | measure operation |
| Value field | measure field |
| Count rows | row_count |
| Count non-empty values | count |
| Count unique values | unique_count |
| Do not summarize | none |
| Category limit | number of retained categories |

## Progressive disclosure

- Keep Category, Series, Measure, ordering, orientation, layout, and value format visible.
- Collapse Filters when empty or already understood.
- Collapse Labels and colors by default.
- Show null handling next to the role it affects, not as one global Missing Values control.
- Hide Value field for Count rows.
- For Do not summarize, show a warning that every category/series group must resolve to exactly one row.
- Disable or reject Stacked layout for non-additive calculations such as average, median, unique count, and do-not-summarize.
- When Series is enabled, Category limit always counts categories, not individual series bars.

## Component-kind switching

- Keep a single editor route.
- Render the kind picker as compact tabs/cards while the supported set remains small.
- Each kind choice should include a one-sentence description.
- When an existing draft contains kind-specific changes, require confirmation before discarding incompatible config.
- On intentional kind change, retain only values that have an unambiguous shared meaning, such as Dataset binding and compatible saved filters.
- Do not try to translate Bar category/series choices automatically into Table columns or Stat Card config.

## Preview

- Keep a sticky preview beside the controls on desktop and below them on narrower screens.
- Preview the exact post-filter, post-group, post-sort, post-limit result.
- Include a plain-language execution summary below the preview.
- Use stable validation, empty, pending-materialization, and execution-error states in the same preview frame.
- The preview should never silently repair duplicate groups for Do not summarize.

## Suggested Rust/Leptos boundary

Keep the outer `ComponentEditorContent` responsible for loading Dataset/component state and lifecycle actions. Delegate config parsing and controls to a kind-specific enum and components:

```rust
enum ComponentConfigDraft {
    Table(TableConfigDraft),
    Bar(BarConfigDraft),
    Line(LineConfigDraft),
    PieDonut(PieDonutConfigDraft),
    StatCard(StatCardConfigDraft),
}
```

The parent should switch on `component_type`, render the appropriate editor, and receive a typed config update callback. Serialize to JSON only at the API boundary. This avoids a large set of unrelated string signals in the shared page component.

## Accessibility and responsive behavior

- Use a real radiogroup or tablist for the kind picker, with a select fallback on narrow screens.
- Preserve visible labels; do not rely on tooltips for essential meaning.
- Announce preview and validation changes through an `aria-live="polite"` region.
- Keep focus in a predictable location after kind changes or conditional control changes.
- At smaller widths, move Preview below configuration and render the kind choices as a select.
