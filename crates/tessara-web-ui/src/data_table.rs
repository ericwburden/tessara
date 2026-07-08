//! Shared data-table components.

use std::collections::{BTreeMap, BTreeSet};

use icons::{
    ArrowDown, ArrowUp, ChevronLeft, ChevronRight, Columns3Cog, ListFilter, RotateCcw, Search,
};
use leptos::prelude::*;

#[component]
pub fn DataTable(children: Children) -> impl IntoView {
    view! {
        <div class="table-wrap">
            <table class="data-table">
                {children()}
            </table>
        </div>
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct InteractiveTableColumn {
    pub key: String,
    pub label: String,
    pub data_type: String,
}

impl InteractiveTableColumn {
    pub fn new(
        key: impl Into<String>,
        label: impl Into<String>,
        data_type: impl Into<String>,
    ) -> Self {
        Self {
            key: key.into(),
            label: label.into(),
            data_type: data_type.into(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct InteractiveTableRow {
    pub id: String,
    pub values: BTreeMap<String, String>,
}

impl InteractiveTableRow {
    pub fn new(id: impl Into<String>, values: BTreeMap<String, String>) -> Self {
        Self {
            id: id.into(),
            values,
        }
    }
}

#[component]
pub fn InteractiveDataTable(
    columns: Vec<InteractiveTableColumn>,
    rows: Vec<InteractiveTableRow>,
    search_label: &'static str,
    search_placeholder: &'static str,
    item_label: &'static str,
    empty_message: &'static str,
) -> impl IntoView {
    let columns = RwSignal::new(columns);
    let rows = RwSignal::new(rows);
    let search = RwSignal::new(String::new());
    let visible_column_keys = RwSignal::new(
        columns
            .get_untracked()
            .into_iter()
            .map(|column| column.key)
            .collect::<Vec<_>>(),
    );
    let sort_key = RwSignal::new(None::<String>);
    let sort_ascending = RwSignal::new(true);
    let column_filters = RwSignal::new(BTreeMap::<String, String>::new());
    let page_size = RwSignal::new(10_usize);
    let page_index = RwSignal::new(0_usize);

    let visible_columns = Memo::new(move |_| {
        let selected = visible_column_keys.get();
        columns
            .get()
            .into_iter()
            .filter(|column| selected.iter().any(|key| key == &column.key))
            .collect::<Vec<_>>()
    });

    let filtered_rows = Memo::new(move |_| {
        let query = search.get().trim().to_lowercase();
        let filters = column_filters.get();
        let mut filtered = rows
            .get()
            .into_iter()
            .filter(|row| {
                let matches_search = query.is_empty()
                    || row
                        .values
                        .values()
                        .any(|value| value.to_lowercase().contains(&query));
                let matches_filters = filters.iter().all(|(key, selected)| {
                    selected.is_empty()
                        || row.values.get(key).is_some_and(|value| value == selected)
                });
                matches_search && matches_filters
            })
            .collect::<Vec<_>>();

        if let Some(key) = sort_key.get() {
            filtered.sort_by(|left, right| {
                let left_value = left.values.get(&key).cloned().unwrap_or_default();
                let right_value = right.values.get(&key).cloned().unwrap_or_default();
                compare_table_values(&left_value, &right_value)
            });
            if !sort_ascending.get() {
                filtered.reverse();
            }
        }

        filtered
    });

    let paged_rows = Memo::new(move |_| {
        let rows = filtered_rows.get();
        let page_size = page_size.get().max(1);
        let page_count = rows.len().max(1).div_ceil(page_size);
        let page = page_index.get().min(page_count.saturating_sub(1));
        let start = page * page_size;
        let end = (start + page_size).min(rows.len());
        rows[start..end].to_vec()
    });

    let total_count = Memo::new(move |_| filtered_rows.get().len());
    let page_count = Memo::new(move |_| total_count.get().max(1).div_ceil(page_size.get().max(1)));
    let current_page = Memo::new(move |_| page_index.get().min(page_count.get().saturating_sub(1)));
    let previous_disabled = Memo::new(move |_| current_page.get() == 0);
    let next_disabled = Memo::new(move |_| current_page.get() + 1 >= page_count.get());
    let page_summary = move || {
        let total = total_count.get();
        if total == 0 {
            format!("No {item_label} to display")
        } else {
            let start = current_page.get() * page_size.get();
            let end = (start + page_size.get()).min(total);
            format!("Showing {}-{} of {} {item_label}", start + 1, end, total)
        }
    };

    let reset_page = move || page_index.set(0);
    let reset_table = move || {
        search.set(String::new());
        visible_column_keys.set(
            columns
                .get()
                .into_iter()
                .map(|column| column.key)
                .collect::<Vec<_>>(),
        );
        sort_key.set(None);
        sort_ascending.set(true);
        column_filters.set(BTreeMap::new());
        page_index.set(0);
    };

    view! {
        <div class="interactive-data-table">
            <div class="interactive-data-table__toolbar">
                <label class="searchable-data-table__search searchable-data-table__control interactive-data-table__search">
                    <Search class="searchable-data-table__control-icon"/>
                    <span class="sr-only">{search_label}</span>
                    <input
                        type="search"
                        placeholder=search_placeholder
                        prop:value=move || search.get()
                        on:input=move |event| {
                            search.set(event_target_value(&event));
                            reset_page();
                        }
                    />
                </label>
                <div class="interactive-data-table__toolbar-actions">
                    <button
                        class="icon-button icon-button--control interactive-data-table__reset"
                        type="button"
                        aria-label="Reset table controls"
                        title="Reset table controls"
                        on:click=move |_| reset_table()
                    >
                        <RotateCcw/>
                    </button>
                    <ColumnVisibilityMenu columns visible_column_keys/>
                </div>
            </div>
            <div class="table-wrap interactive-data-table__table">
                <table class="data-table">
                    <thead>
                        <tr>
                            {move || visible_columns.get().into_iter().map(|column| {
                                let filter_options = filter_options_for_column(rows.get(), &column.key);
                                view! {
                                    <InteractiveTableHeader
                                        column
                                        filter_options
                                        sort_key
                                        sort_ascending
                                        column_filters
                                        reset_page
                                    />
                                }
                            }).collect_view()}
                        </tr>
                    </thead>
                    <tbody>
                        {move || {
                            let visible = visible_columns.get();
                            let rows = paged_rows.get();
                            if visible.is_empty() {
                                view! {
                                    <tr>
                                        <td class="data-table__empty" colspan="1">"Select at least one column to display."</td>
                                    </tr>
                                }.into_any()
                            } else if rows.is_empty() {
                                view! {
                                    <tr>
                                        <td class="data-table__empty" colspan=visible.len().to_string()>{empty_message}</td>
                                    </tr>
                                }.into_any()
                            } else {
                                rows.into_iter().map(|row| {
                                    let visible = visible.clone();
                                    view! {
                                        <tr>
                                            {visible.into_iter().map(|column| {
                                                let value = row.values.get(&column.key).cloned().unwrap_or_default();
                                                view! { <td>{value}</td> }
                                            }).collect_view()}
                                        </tr>
                                    }
                                }).collect_view().into_any()
                            }
                        }}
                    </tbody>
                </table>
            </div>
            <div class="directory-table-pagination interactive-data-table__pagination" aria-label="Table pagination">
                <p>{page_summary}</p>
                <div class="directory-table-pagination__actions">
                    <label class="directory-table-pagination__page-size searchable-data-table__filter searchable-data-table__control">
                        <span>"Rows"</span>
                        <select
                            prop:value=move || page_size.get().to_string()
                            on:change=move |event| {
                                if let Ok(size) = event_target_value(&event).parse::<usize>() {
                                    page_size.set(size);
                                    page_index.set(0);
                                }
                            }
                        >
                            <option value="10">"10"</option>
                            <option value="25">"25"</option>
                            <option value="50">"50"</option>
                        </select>
                    </label>
                    <button
                        class="interactive-data-table__page-button"
                        type="button"
                        aria-label="Previous page"
                        title="Previous page"
                        disabled=move || previous_disabled.get()
                        on:click=move |_| page_index.update(|page| *page = page.saturating_sub(1))
                    >
                        <ChevronLeft/>
                    </button>
                    <span>{move || format!("Page {} of {}", current_page.get() + 1, page_count.get())}</span>
                    <button
                        class="interactive-data-table__page-button"
                        type="button"
                        aria-label="Next page"
                        title="Next page"
                        disabled=move || next_disabled.get()
                        on:click=move |_| {
                            let last_page = page_count.get().saturating_sub(1);
                            page_index.update(|page| *page = (*page + 1).min(last_page));
                        }
                    >
                        <ChevronRight/>
                    </button>
                </div>
            </div>
        </div>
    }
}

#[component]
fn ColumnVisibilityMenu(
    columns: RwSignal<Vec<InteractiveTableColumn>>,
    visible_column_keys: RwSignal<Vec<String>>,
) -> impl IntoView {
    let is_open = RwSignal::new(false);
    let toggle_key = move |key: String| {
        visible_column_keys.update(|keys| {
            if keys.iter().any(|selected| selected == &key) {
                keys.retain(|selected| selected != &key);
            } else {
                keys.push(key.clone());
            }
        });
    };

    view! {
        <div class=move || if is_open.get() { "interactive-data-table__columns is-open" } else { "interactive-data-table__columns" }>
            <button
                class="icon-button icon-button--control interactive-data-table__columns-trigger"
                type="button"
                aria-label="Choose visible columns"
                title="Choose visible columns"
                aria-haspopup="menu"
                aria-expanded=move || is_open.get().to_string()
                on:click=move |_| is_open.update(|open| *open = !*open)
            >
                <Columns3Cog/>
            </button>
            <button
                class="data-table-filter__scrim"
                type="button"
                aria-label="Close column selector"
                on:click=move |_| is_open.set(false)
            ></button>
            <div class="interactive-data-table__columns-menu blurred-surface" role="menu">
                <div class="interactive-data-table__columns-actions">
                    <button
                        class="button button--compact button--secondary"
                        type="button"
                        on:click=move |_| visible_column_keys.set(columns.get().into_iter().map(|column| column.key).collect())
                    >
                        "Show All"
                    </button>
                    <button
                        class="button button--compact button--secondary"
                        type="button"
                        on:click=move |_| visible_column_keys.set(Vec::new())
                    >
                        "Hide All"
                    </button>
                </div>
                {move || columns.get().into_iter().map(|column| {
                    let key = column.key.clone();
                    let checked_key = column.key.clone();
                    view! {
                        <label class="interactive-data-table__column-option">
                            <input
                                type="checkbox"
                                prop:checked=move || visible_column_keys.get().iter().any(|selected| selected == &checked_key)
                                on:change=move |_| toggle_key(key.clone())
                            />
                            <span>{column.label}</span>
                        </label>
                    }
                }).collect_view()}
            </div>
        </div>
    }
}

#[component]
fn InteractiveTableHeader(
    column: InteractiveTableColumn,
    filter_options: Vec<String>,
    sort_key: RwSignal<Option<String>>,
    sort_ascending: RwSignal<bool>,
    column_filters: RwSignal<BTreeMap<String, String>>,
    reset_page: impl Fn() + Copy + 'static,
) -> impl IntoView {
    let is_open = RwSignal::new(false);
    let key = column.key;
    let label = column.label;
    let is_filtered_key = key.clone();
    let sort_key_for_asc = key.clone();
    let sort_key_for_desc = key.clone();
    let clear_class_key = key.clone();
    let clear_checked_key = key.clone();
    let clear_click_key = key.clone();
    let menu_class = move || {
        if is_open.get() {
            "interactive-data-table__header-menu is-open"
        } else {
            "interactive-data-table__header-menu"
        }
    };
    let is_filtered = move || {
        column_filters
            .get()
            .get(&is_filtered_key)
            .is_some_and(|value| !value.is_empty())
    };
    let sort_icon_key = key.clone();
    let sort_trigger_key = key.clone();

    view! {
        <th scope="col">
            <div class="interactive-data-table__header-cell">
                <span>{label.clone()}</span>
                <div class=menu_class>
                    <button
                        class=move || if is_filtered() || sort_key.get().as_ref() == Some(&sort_trigger_key) {
                            "icon-button data-table-filter__trigger is-filtered"
                        } else {
                            "icon-button data-table-filter__trigger"
                        }
                        type="button"
                        aria-label=format!("Sort and filter {label}")
                        title=format!("Sort and filter {label}")
                        aria-haspopup="menu"
                        aria-expanded=move || is_open.get().to_string()
                        on:click=move |_| is_open.update(|open| *open = !*open)
                    >
                        {move || if sort_key.get().as_ref() == Some(&sort_icon_key) {
                            if sort_ascending.get() {
                                view! { <ArrowUp/> }.into_any()
                            } else {
                                view! { <ArrowDown/> }.into_any()
                            }
                        } else {
                            view! { <ListFilter/> }.into_any()
                        }}
                    </button>
                    <button
                        class="data-table-filter__scrim"
                        type="button"
                        aria-label=format!("Close {label} controls")
                        on:click=move |_| is_open.set(false)
                    ></button>
                    <div class="data-table-filter__menu blurred-surface interactive-data-table__header-controls" role="menu">
                        <button
                            class="data-table-filter__option"
                            type="button"
                            role="menuitem"
                            on:click=move |_| {
                                sort_key.set(Some(sort_key_for_asc.clone()));
                                sort_ascending.set(true);
                                reset_page();
                                is_open.set(false);
                            }
                        >
                            "Sort ascending"
                        </button>
                        <button
                            class="data-table-filter__option"
                            type="button"
                            role="menuitem"
                            on:click=move |_| {
                                sort_key.set(Some(sort_key_for_desc.clone()));
                                sort_ascending.set(false);
                                reset_page();
                                is_open.set(false);
                            }
                        >
                            "Sort descending"
                        </button>
                        <button
                            class="data-table-filter__option"
                            type="button"
                            role="menuitem"
                            on:click=move |_| {
                                sort_key.set(None);
                                reset_page();
                                is_open.set(false);
                            }
                        >
                            "Clear sort"
                        </button>
                        <hr class="interactive-data-table__menu-rule"/>
                        <button
                            class=move || if column_filters.get().get(&clear_class_key).is_none_or(|value| value.is_empty()) {
                                "data-table-filter__option is-active"
                            } else {
                                "data-table-filter__option"
                            }
                            type="button"
                            role="menuitemradio"
                            aria-checked=move || column_filters.get().get(&clear_checked_key).is_none_or(|value| value.is_empty()).to_string()
                            on:click=move |_| {
                                column_filters.update(|filters| {
                                    filters.remove(&clear_click_key);
                                });
                                reset_page();
                                is_open.set(false);
                            }
                        >
                            "All values"
                        </button>
                        {filter_options.into_iter().map(|option| {
                            let option_for_class = option.clone();
                            let option_for_checked = option.clone();
                            let option_for_click = option.clone();
                            let option_for_title = option.clone();
                            let key_for_class = key.clone();
                            let key_for_checked = key.clone();
                            let key_for_click = key.clone();
                            view! {
                                <button
                                    class=move || if column_filters.get().get(&key_for_class) == Some(&option_for_class) {
                                        "data-table-filter__option is-active"
                                    } else {
                                        "data-table-filter__option"
                                    }
                                    title=option_for_title
                                    type="button"
                                    role="menuitemradio"
                                    aria-checked=move || (column_filters.get().get(&key_for_checked) == Some(&option_for_checked)).to_string()
                                    on:click=move |_| {
                                        column_filters.update(|filters| {
                                            filters.insert(key_for_click.clone(), option_for_click.clone());
                                        });
                                        reset_page();
                                        is_open.set(false);
                                    }
                                >
                                    {option}
                                </button>
                            }
                        }).collect_view()}
                    </div>
                </div>
            </div>
        </th>
    }
}

fn filter_options_for_column(rows: Vec<InteractiveTableRow>, key: &str) -> Vec<String> {
    rows.into_iter()
        .filter_map(|row| row.values.get(key).cloned())
        .filter(|value| !value.trim().is_empty())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}

fn compare_table_values(left: &str, right: &str) -> std::cmp::Ordering {
    match (left.parse::<f64>(), right.parse::<f64>()) {
        (Ok(left_number), Ok(right_number)) => left_number
            .partial_cmp(&right_number)
            .unwrap_or(std::cmp::Ordering::Equal),
        _ => left.to_lowercase().cmp(&right.to_lowercase()),
    }
}
