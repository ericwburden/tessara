//! Generic filter-header component for table controls.
//!
//! Keep generic filter dropdown rendering here so feature tables can supply labels, options, and selected state without duplicating menu markup.

use icons::{ListFilter, Search};
use leptos::prelude::*;

use crate::empty_view;

#[component]
pub fn TableFilterHeader(
    label: &'static str,
    all_label: &'static str,
    filter: RwSignal<String>,
    options: Vec<String>,
    #[prop(default = false)] always_searchable: bool,
) -> impl IntoView {
    const FILTER_SEARCH_THRESHOLD: usize = 8;

    let is_open = RwSignal::new(false);
    let filter_query = RwSignal::new(String::new());
    let is_searchable = always_searchable || options.len() > FILTER_SEARCH_THRESHOLD;
    let options_for_buttons = options.clone();
    let menu_class = move || {
        if is_open.get() {
            "data-table-filter is-open"
        } else {
            "data-table-filter"
        }
    };
    let button_label = move || {
        let current = filter.get();
        if current == "all" {
            format!("Filter {label}")
        } else {
            format!("Filter {label}: {}", metadata_label(&current))
        }
    };
    let trigger_class = move || {
        if filter.get() == "all" {
            "icon-button data-table-filter__trigger"
        } else {
            "icon-button data-table-filter__trigger is-filtered"
        }
    };
    let filter_option_class = |current: &str, value: &str| {
        if current == value {
            "data-table-filter__option is-active"
        } else {
            "data-table-filter__option"
        }
    };
    let filtered_options = move || {
        let query = filter_query.get();
        options_for_buttons
            .iter()
            .filter(|option| {
                text_matches(&query, &[option.as_str(), metadata_label(option).as_str()])
            })
            .cloned()
            .collect::<Vec<_>>()
    };

    view! {
        <div class=menu_class>
            <span>{label}</span>
            <button
                class=trigger_class
                type="button"
                aria-label=button_label
                title=button_label
                aria-haspopup="menu"
                aria-expanded=move || is_open.get().to_string()
                on:click=move |_| is_open.update(|open| *open = !*open)
            >
                <ListFilter/>
            </button>
            <button
                class="data-table-filter__scrim"
                type="button"
                aria-label=format!("Close {label} filter")
                on:click=move |_| {
                    is_open.set(false);
                    filter_query.set(String::new());
                }
            ></button>
            <div class="data-table-filter__menu blurred-surface" role="menu">
                {if is_searchable {
                    view! {
                        <label class="data-table-filter__search">
                            <Search/>
                            <span class="sr-only">{format!("Search {label} filters")}</span>
                            <input
                                type="search"
                                placeholder=format!("Search {label}")
                                prop:value=move || filter_query.get()
                                on:input=move |event| filter_query.set(event_target_value(&event))
                            />
                        </label>
                    }
                    .into_any()
                } else {
                    empty_view()
                }}
                <button
                    class=move || filter_option_class(&filter.get(), "all")
                    type="button"
                    role="menuitemradio"
                    aria-checked=move || (filter.get() == "all").to_string()
                    on:click=move |_| {
                        filter.set("all".to_string());
                        is_open.set(false);
                        filter_query.set(String::new());
                    }
                >
                    {all_label}
                </button>
                {move || {
                    let visible_options = filtered_options();
                    if visible_options.is_empty() {
                        view! {
                            <p class="data-table-filter__empty">"No matching filters"</p>
                        }
                        .into_any()
                    } else {
                        visible_options
                            .into_iter()
                            .map(|option| {
                                let option_for_class = option.clone();
                                let option_for_checked = option.clone();
                                let option_for_click = option.clone();
                                let option_label = metadata_label(&option);
                                view! {
                                    <button
                                        class=move || filter_option_class(&filter.get(), &option_for_class)
                                        type="button"
                                        role="menuitemradio"
                                        aria-checked=move || (filter.get() == option_for_checked).to_string()
                                        on:click=move |_| {
                                            filter.set(option_for_click.clone());
                                            is_open.set(false);
                                            filter_query.set(String::new());
                                        }
                                    >
                                        {option_label}
                                    </button>
                                }
                            })
                            .collect_view()
                            .into_any()
                    }
                }}
            </div>
        </div>
    }
}

fn metadata_label(key: &str) -> String {
    key.split('_')
        .filter(|part| !part.is_empty())
        .map(|part| {
            let mut chars = part.chars();
            match chars.next() {
                Some(first) => format!("{}{}", first.to_uppercase(), chars.as_str()),
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn text_matches(query: &str, values: &[&str]) -> bool {
    let query = query.trim().to_lowercase();
    if query.is_empty() {
        return true;
    }

    values
        .iter()
        .any(|value| value.to_lowercase().contains(&query))
}
