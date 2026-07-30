//! Controlled, presentation-only search control for tables and directories.

use icons::Search;
use leptos::prelude::*;

#[component]
pub fn TableSearch(
    value: Signal<String>,
    on_input: Callback<String>,
    #[prop(default = "Search table")] label: &'static str,
    #[prop(default = "Search")] placeholder: &'static str,
    #[prop(default = Signal::derive(|| false), into)] disabled: Signal<bool>,
    #[prop(optional, into)] class: String,
) -> impl IntoView {
    let class = if class.trim().is_empty() {
        "searchable-data-table__search searchable-data-table__control".to_string()
    } else {
        format!(
            "searchable-data-table__search searchable-data-table__control {}",
            class.trim()
        )
    };

    view! {
        <label class=class>
            <Search class="searchable-data-table__control-icon"/>
            <span class="sr-only">{label}</span>
            <input
                type="search"
                aria-label=label
                placeholder=placeholder
                disabled=move || disabled.get()
                prop:value=move || value.get()
                on:input=move |event| {
                    if !disabled.get_untracked() {
                        on_input.run(event_target_value(&event));
                    }
                }
            />
        </label>
    }
}

#[cfg(all(test, feature = "ssr"))]
mod tests {
    use super::*;

    #[test]
    fn search_renders_shared_control_contract() {
        let html = Owner::new().with(|| {
            view! {
                <TableSearch
                    value=Signal::derive(|| "orders".to_string())
                    on_input=Callback::new(|_| {})
                    label="Search components"
                    placeholder="Filter by name"
                    class="fixture-search"
                />
            }
            .to_html()
        });

        assert!(html.contains("searchable-data-table__search"));
        assert!(html.contains("fixture-search"));
        assert!(html.contains("aria-label=\"Search components\""));
        assert!(html.contains("placeholder=\"Filter by name\""));
        assert!(html.contains("type=\"search\""));
    }
}
