# Tessara UI Primitive Contracts

This document is the checked-in reference for the shared UI primitives introduced in Sprint 1G and still relied on by the current SSR application shell.

Use it together with [ui-guidance.md](../ui-guidance.md). This file is narrower: it documents the actual primitive contracts that exist today, where they currently belong, and the intended usage shape for new shared-surface work.

## Current Source Of Truth

There are two real primitive layers in the repo today:

1. `crates/tessara-web/src/features/native_shell.rs`
   - Leptos-native wrappers for shared SSR route structure.
   - Current stable surface primitives:
     - `NativePage`
     - `PageHeader`
     - `MetadataStrip`
     - `Panel`
   - These are the default choice for native `/app` routes.

2. `crates/tessara-ui/src/lib.rs`
   - HTML helper primitives used by compatibility surfaces and older string-rendered route builders.
   - Current stable helpers:
     - `action_group`
     - `page_header`
     - `panel`
     - `panel_with_header`
     - `card`
     - `field_wrapper`
     - `checkbox_field`
     - `toolbar`

These layers are not fully converged yet. New native SSR route work should prefer the `native_shell` components. Use raw `tessara-ui` HTML helpers only when extending a compatibility surface or when extracting a shared contract that does not yet have a Leptos-native wrapper.

## Approved Shared Contracts

### Native page shell

Use `NativePage` as the outer frame for product-facing SSR routes.

```rust
view! {
    <NativePage
        title="Tessara Home"
        description="Tessara application home for local replacement workflow testing."
        page_key="home"
        active_route="home"
        workspace_label="Shared Home"
        breadcrumbs=vec![BreadcrumbItem::current("Home")]
    >
        // page content
    </NativePage>
}
```

Contract:
- route title, active route, and page key are explicit
- shared shell chrome comes from the native application shell, not page-local markup
- product routes should not rebuild sidebar or top-bar structure locally

### Page header

Use `PageHeader` once per route as the top route summary.

```rust
view! {
    <PageHeader
        eyebrow="Workflows"
        title="Workflow Directory"
        description="Current workflow records, linked forms, and assignment counts appear here."
    />
}
```

Contract:
- one eyebrow
- one page title
- one concise route summary
- page-level actions belong in the header area, not repeated in later panels

### Metadata strip

Use `MetadataStrip` for compact route state.

```rust
view! {
    <MetadataStrip items=vec![
        ("Mode", "Directory".into()),
        ("Surface", "Workflow runtime shell".into()),
        ("State", "Loading workflow records".into()),
    ]/>
}
```

Contract:
- use short label/value pairs only
- route mode, surface, and state are the common first choices
- do not turn this into a general-purpose detail grid

### Panel

Use `Panel` for substantive route sections.

```rust
view! {
    <Panel
        title="Product Areas"
        description="These are the primary destinations for top-level entity browsing and workflow entry."
    >
        // section content
    </Panel>
}
```

Contract:
- title is required
- description is short and explanatory
- body holds the route-local content
- use multiple panels instead of one oversized page body

### Action group

Use `tessara_ui::action_group` when a string-rendered or compatibility-owned surface needs shared action markup.

```rust
let actions = tessara_ui::action_group(&[
    ActionItem::link("Create Workflow", "/app/workflows/new", ActionStyle::Primary),
    ActionItem::link(
        "Open Assignment Console",
        "/app/workflows/assignments",
        ActionStyle::Light,
    ),
]);
```

Contract:
- group route-level or section-level actions only
- prefer links for navigation and buttons for in-place operations
- do not create page-local action wrappers when this shared shape is sufficient

### Card

`tessara_ui::card` exists, but native route adoption is incomplete. Today it is best treated as the compatibility-layer card contract, not as a fully adopted native route primitive.

```rust
let card_html = tessara_ui::card(
    "directory-card",
    "Forms",
    r#"<p>Browse forms, inspect lifecycle state, and review workflow attachments.</p>"#,
);
```

Contract:
- use for concise summary or navigation blocks
- keep content short
- do not invent a new card class family when `record-card`, `directory-card`, or `home-card` already covers the need

### Field wrapper and checkbox field

Use `field_wrapper` or `checkbox_field` when a shared form control block is sufficient.

```rust
let name_field = tessara_ui::field_wrapper(
    "form-name",
    "Name",
    &tessara_ui::text_input("form-name", "text", "off", "Form name", ""),
    Some("Use the top-level label shown in navigation."),
    "wide-field",
);

let active_field =
    tessara_ui::checkbox_field("assignment-active", "Assignment is active", true, None);
```

Contract:
- label above control
- helper text only when it adds meaning
- `wide-field` is the current shared full-width modifier

### Toolbar

Use `toolbar` for compact filter/search/action rows.

```rust
let toolbar_html = tessara_ui::toolbar(
    &tessara_ui::field_wrapper(
        "delegate-context-select",
        "Acting For",
        r#"<select id="delegate-context-select"></select>"#,
        None,
        "",
    ),
    "",
);
```

Contract:
- primary zone holds the main filter/search control
- secondary zone is optional
- do not create new ad hoc filter-row wrappers when this layout works

## Current Adoption Notes

Current strong adoption:
- native shared page shell on `Home`, `Forms`, `Workflows`, and `Responses`
- role-aware navigation driven from shared link specifications in `native_shell.rs`
- shared route framing through `NativePage`, `PageHeader`, `MetadataStrip`, and `Panel`

Current partial adoption:
- cards, field grids, and action rows inside `Forms`, `Workflows`, and `Responses` still rely heavily on page-local `record-card`, `form-field`, `button-link`, and `page-title-row` markup
- `Home` still uses route-local `home-card` and `directory-card` markup for destination tiles

Current non-adoption:
- `Organization` routes still render through `inner_html` and retained application-shell builders
- `Administration`, `Reporting`, `Dashboards`, and `Migration` remain compatibility surfaces

## Review Rule

When touching a shared `/app` surface:
- use an approved primitive if one already exists
- if no approved primitive exists, reuse the existing shared class family instead of inventing a new one
- if a bespoke structure is unavoidable, document the reason and the intended replacement target in the PR or issue
