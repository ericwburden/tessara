//! Shared searchable combobox control.

use icons::{ChevronsUpDown, Search};
use leptos::prelude::*;

#[derive(Clone)]
pub(crate) struct ComboboxOption {
    pub(crate) value: String,
    pub(crate) label: String,
}

#[component]
pub(crate) fn Combobox(
    options: Signal<Vec<ComboboxOption>>,
    on_select: Callback<String>,
    #[prop(optional)] selected_label: Option<Signal<String>>,
    #[prop(default = "Select...")] placeholder: &'static str,
    #[prop(default = "Search...")] search_placeholder: &'static str,
    #[prop(default = "No options found.")] empty_label: &'static str,
    #[prop(default = "Combobox options")] aria_label: &'static str,
) -> impl IntoView {
    let is_open = RwSignal::new(false);
    let search = RwSignal::new(String::new());
    let search_input = NodeRef::<leptos::html::Input>::new();

    Effect::new(move |_| {
        if is_open.get()
            && let Some(input) = search_input.get()
        {
            let _ = input.focus();
        }
    });

    view! {
        <div class=move || if is_open.get() { "combobox is-open" } else { "combobox" }>
            <button
                class="combobox__trigger"
                type="button"
                aria-haspopup="listbox"
                aria-expanded=move || is_open.get().to_string()
                aria-label=aria_label
                on:click=move |_| is_open.update(|open| *open = !*open)
            >
                <span class="truncate">
                    {move || {
                        selected_label
                            .as_ref()
                            .map(|label| label.get())
                            .filter(|label| !label.trim().is_empty())
                            .unwrap_or_else(|| placeholder.to_string())
                    }}
                </span>
                <ChevronsUpDown class="combobox__trigger-icon"/>
            </button>
            <button
                class="combobox__scrim"
                type="button"
                aria-label="Close combobox"
                on:click=move |_| is_open.set(false)
            ></button>
            <div class="combobox__content blurred-surface">
                <div class="combobox__search">
                    <Search class="combobox__search-icon"/>
                    <input
                        class="combobox__input"
                        type="search"
                        placeholder=search_placeholder
                        node_ref=search_input
                        prop:value=move || search.get()
                        on:input=move |event| search.set(event_target_value(&event))
                    />
                </div>
                <div class="combobox__list" role="listbox">
                    {move || {
                        let query = search.get().trim().to_lowercase();
                        let filtered = options
                            .get()
                            .into_iter()
                            .filter(|option| {
                                query.is_empty() || option.label.to_lowercase().contains(&query)
                            })
                            .collect::<Vec<_>>();
                        if filtered.is_empty() {
                            view! {
                                <div class="combobox__empty">{empty_label}</div>
                            }.into_any()
                        } else {
                            view! {
                                <div class="combobox__group">
                                    {filtered.into_iter().map(|option| {
                                        let value = option.value.clone();
                                        view! {
                                            <button
                                                class="combobox__item"
                                                type="button"
                                                role="option"
                                                on:click=move |_| {
                                                    on_select.run(value.clone());
                                                    search.set(String::new());
                                                    is_open.set(false);
                                                }
                                            >
                                                {option.label}
                                            </button>
                                        }
                                    }).collect_view()}
                                </div>
                            }.into_any()
                        }
                    }}
                </div>
            </div>
        </div>
    }
}
