//! Controlled, data-source-neutral table controls.
//!
//! These primitives own repeated table-control markup and browser focus
//! behavior while feature crates retain their own local or server-backed state.

use icons::Columns3Cog;
use leptos::prelude::*;

/// A column exposed by [`TableColumnSelector`].
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TableColumnOption {
    pub key: String,
    pub label: String,
}

impl TableColumnOption {
    pub fn new(key: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            label: label.into(),
        }
    }
}

/// Shared table-toolbar layout. Callers provide their own search and actions.
#[component]
pub fn TableToolbar(children: Children, #[prop(optional, into)] class: String) -> impl IntoView {
    let class = if class.trim().is_empty() {
        "interactive-data-table__toolbar".to_string()
    } else {
        format!("interactive-data-table__toolbar {}", class.trim())
    };
    view! { <div class=class>{children()}</div> }
}

/// Right-aligned action group used within a [`TableToolbar`].
#[component]
pub fn TableToolbarActions(children: Children) -> impl IntoView {
    view! { <div class="interactive-data-table__toolbar-actions">{children()}</div> }
}

/// DOM and focus controller for table header and toolbar popovers.
///
/// The caller owns the popover markup and decides when an action closes it;
/// this controller consistently handles initial focus, Escape, scrim dismissal,
/// and restoration of focus to the trigger.
#[derive(Clone, Copy)]
pub struct TablePopoverController {
    pub open: RwSignal<bool>,
    pub trigger: NodeRef<leptos::html::Button>,
    pub panel: NodeRef<leptos::html::Div>,
}

impl TablePopoverController {
    pub fn new() -> Self {
        let controller = Self {
            open: RwSignal::new(false),
            trigger: NodeRef::new(),
            panel: NodeRef::new(),
        };
        install_table_popover_initial_focus(controller.open, controller.panel);
        controller
    }

    pub fn toggle(self) {
        self.open.update(|open| *open = !*open);
    }

    pub fn close(self) {
        self.open.set(false);
        focus_table_popover_trigger(self.trigger);
    }

    pub fn handle_keydown(self, event: leptos::ev::KeyboardEvent) {
        if event.key() == "Escape" {
            event.prevent_default();
            event.stop_propagation();
            self.close();
        }
    }
}

impl Default for TablePopoverController {
    fn default() -> Self {
        Self::new()
    }
}

/// Controlled visible-column selector for local or server-backed tables.
#[component]
pub fn TableColumnSelector(
    columns: Signal<Vec<TableColumnOption>>,
    visible_column_keys: Signal<Vec<String>>,
    on_change: Callback<Vec<String>>,
    #[prop(optional, into)] id: Option<String>,
    /// Prevents callers such as paged server tables from hiding every column.
    #[prop(default = 0)]
    minimum_visible_columns: usize,
) -> impl IntoView {
    let popover = TablePopoverController::new();
    let trigger_controls = id.clone();
    let show_hide_all = minimum_visible_columns == 0;

    view! {
        <div class=move || {
            if popover.open.get() {
                "interactive-data-table__columns is-open"
            } else {
                "interactive-data-table__columns"
            }
        }>
            <button
                node_ref=popover.trigger
                class="icon-button icon-button--control interactive-data-table__columns-trigger"
                type="button"
                aria-label="Choose visible columns"
                title="Choose visible columns"
                aria-haspopup="dialog"
                aria-controls=trigger_controls
                aria-expanded=move || popover.open.get().to_string()
                on:click=move |_| popover.toggle()
            >
                <Columns3Cog/>
            </button>
            <button
                class="data-table-filter__scrim"
                type="button"
                aria-label="Close column selector"
                on:click=move |_| popover.close()
            ></button>
            <div
                node_ref=popover.panel
                id=id
                class="interactive-data-table__columns-menu blurred-surface"
                role="dialog"
                aria-label="Visible columns"
                tabindex="-1"
                on:keydown=move |event| popover.handle_keydown(event)
            >
                <div class="interactive-data-table__columns-actions">
                    <button
                        class="button button--compact button--secondary"
                        type="button"
                        on:click=move |_| {
                            on_change.run(
                                columns
                                    .get_untracked()
                                    .into_iter()
                                    .map(|column| column.key)
                                    .collect(),
                            );
                        }
                    >
                        "Show All"
                    </button>
                    <Show when=move || show_hide_all>
                        <button
                            class="button button--compact button--secondary"
                            type="button"
                            on:click=move |_| on_change.run(Vec::new())
                        >
                            "Hide All"
                        </button>
                    </Show>
                </div>
                {move || {
                    columns
                        .get()
                        .into_iter()
                        .map(|column| {
                            let key = column.key.clone();
                            let checked_key = column.key.clone();
                            view! {
                                <label class="interactive-data-table__column-option">
                                    <input
                                        type="checkbox"
                                        prop:checked=move || visible_column_keys
                                            .get()
                                            .iter()
                                            .any(|selected| selected == &checked_key)
                                        on:change=move |_| {
                                            let next = toggled_columns(
                                                &columns.get_untracked(),
                                                visible_column_keys.get_untracked(),
                                                &key,
                                                minimum_visible_columns,
                                            );
                                            on_change.run(next);
                                        }
                                    />
                                    <span>{column.label}</span>
                                </label>
                            }
                        })
                        .collect_view()
                }}
            </div>
        </div>
    }
}

fn toggled_columns(
    columns: &[TableColumnOption],
    mut visible: Vec<String>,
    key: &str,
    minimum_visible_columns: usize,
) -> Vec<String> {
    if let Some(position) = visible.iter().position(|selected| selected == key) {
        if visible.len() > minimum_visible_columns {
            visible.remove(position);
        }
    } else if columns.iter().any(|column| column.key == key) {
        visible.push(key.to_string());
    }

    visible.sort_by_key(|selected| {
        columns
            .iter()
            .position(|column| column.key == *selected)
            .unwrap_or(usize::MAX)
    });
    visible
}

#[cfg(all(feature = "hydrate", target_arch = "wasm32"))]
fn focus_table_popover_trigger(trigger: NodeRef<leptos::html::Button>) {
    if let Some(trigger) = trigger.get() {
        let _ = trigger.focus();
    }
}

#[cfg(not(all(feature = "hydrate", target_arch = "wasm32")))]
fn focus_table_popover_trigger(_: NodeRef<leptos::html::Button>) {}

#[cfg(all(feature = "hydrate", target_arch = "wasm32"))]
fn install_table_popover_initial_focus(is_open: RwSignal<bool>, panel: NodeRef<leptos::html::Div>) {
    Effect::new(move |_| {
        if is_open.get()
            && let Some(panel) = panel.get()
        {
            let _ = panel.focus();
        }
    });
}

#[cfg(not(all(feature = "hydrate", target_arch = "wasm32")))]
fn install_table_popover_initial_focus(_: RwSignal<bool>, _: NodeRef<leptos::html::Div>) {}

#[cfg(test)]
mod tests {
    use super::*;

    fn columns() -> Vec<TableColumnOption> {
        vec![
            TableColumnOption::new("a", "A"),
            TableColumnOption::new("b", "B"),
            TableColumnOption::new("c", "C"),
        ]
    }

    #[test]
    fn toggling_preserves_source_column_order() {
        assert_eq!(
            toggled_columns(&columns(), vec!["c".into(), "a".into()], "b", 0),
            ["a", "b", "c"]
        );
    }

    #[test]
    fn minimum_visible_columns_prevents_empty_server_table() {
        assert_eq!(toggled_columns(&columns(), vec!["a".into()], "a", 1), ["a"]);
        assert!(toggled_columns(&columns(), vec!["a".into()], "a", 0).is_empty());
    }
}
