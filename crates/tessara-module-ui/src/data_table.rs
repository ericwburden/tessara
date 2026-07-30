//! Shared data-table components.

use std::cmp::Ordering;
use std::collections::{BTreeMap, BTreeSet};

use icons::{ArrowDown, ArrowUp, ListFilter, RotateCcw};
use leptos::prelude::*;

use crate::{
    TableColumnOption, TableColumnSelector, TablePaginationFooter, TablePopoverController,
    TableSearch, TableToolbar, TableToolbarActions,
};

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
    pub data_type: InteractiveTableDataType,
}

/// Value semantics used by [`InteractiveDataTable`] when ordering a column.
///
/// Keeping this typed prevents one column from switching comparators from row
/// to row, which would violate the total-order contract required by Rust's
/// sorting algorithms.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum InteractiveTableDataType {
    Boolean,
    Date,
    DateTime,
    Number,
    #[default]
    Text,
}

impl InteractiveTableDataType {
    /// Maps Tessara field metadata to the table's domain-neutral sort type.
    pub fn from_field_type(field_type: &str) -> Self {
        match field_type.trim().to_ascii_lowercase().as_str() {
            "boolean" => Self::Boolean,
            "date" => Self::Date,
            "datetime" | "timestamp" => Self::DateTime,
            "number" | "integer" | "decimal" | "float" => Self::Number,
            _ => Self::Text,
        }
    }
}

impl InteractiveTableColumn {
    pub fn new(
        key: impl Into<String>,
        label: impl Into<String>,
        data_type: InteractiveTableDataType,
    ) -> Self {
        Self {
            key: key.into(),
            label: label.into(),
            data_type,
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
            let data_type = columns
                .get()
                .into_iter()
                .find(|column| column.key == key)
                .map(|column| column.data_type)
                .unwrap_or_default();
            filtered.sort_by(|left, right| {
                let left_value = left.values.get(&key).cloned().unwrap_or_default();
                let right_value = right.values.get(&key).cloned().unwrap_or_default();
                compare_table_values(data_type, &left_value, &right_value)
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
            <TableToolbar>
                <TableSearch
                    value=Signal::derive(move || search.get())
                    on_input=Callback::new(move |value| {
                            search.set(value);
                            reset_page();
                    })
                    label=search_label
                    placeholder=search_placeholder
                    class="interactive-data-table__search"
                />
                <TableToolbarActions>
                    <button
                        class="icon-button icon-button--control interactive-data-table__reset"
                        type="button"
                        aria-label="Reset table controls"
                        title="Reset table controls"
                        on:click=move |_| reset_table()
                    >
                        <RotateCcw/>
                    </button>
                    <TableColumnSelector
                        columns=Signal::derive(move || {
                            columns
                                .get()
                                .into_iter()
                                .map(|column| TableColumnOption::new(column.key, column.label))
                                .collect()
                        })
                        visible_column_keys=Signal::derive(move || visible_column_keys.get())
                        on_change=Callback::new(move |keys| visible_column_keys.set(keys))
                    />
                </TableToolbarActions>
            </TableToolbar>
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
            <TablePaginationFooter
                aria_label="Table pagination"
                item_label
                class="interactive-data-table__pagination"
                total_count
                page_size
                page_index
            />
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
    let popover = TablePopoverController::new();
    let key = column.key;
    let label = column.label;
    let is_filtered_key = key.clone();
    let sort_key_for_asc = key.clone();
    let sort_key_for_desc = key.clone();
    let clear_class_key = key.clone();
    let clear_checked_key = key.clone();
    let clear_click_key = key.clone();
    let menu_class = move || {
        if popover.open.get() {
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
                        node_ref=popover.trigger
                        aria-expanded=move || popover.open.get().to_string()
                        on:click=move |_| popover.toggle()
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
                        on:click=move |_| popover.close()
                    ></button>
                    <div
                        node_ref=popover.panel
                        class="data-table-filter__menu blurred-surface interactive-data-table__header-controls"
                        role="dialog"
                        aria-label=format!("Sort and filter {label}")
                        tabindex="-1"
                        on:keydown=move |event| popover.handle_keydown(event)
                    >
                        <button
                            class="data-table-filter__option"
                            type="button"
                            role="menuitem"
                            on:click=move |_| {
                                sort_key.set(Some(sort_key_for_asc.clone()));
                                sort_ascending.set(true);
                                reset_page();
                                popover.close();
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
                                popover.close();
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
                                popover.close();
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
                                popover.close();
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
                                        popover.close();
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

fn compare_table_values(data_type: InteractiveTableDataType, left: &str, right: &str) -> Ordering {
    match data_type {
        InteractiveTableDataType::Number => compare_numeric_table_values(left, right),
        InteractiveTableDataType::Boolean
        | InteractiveTableDataType::Date
        | InteractiveTableDataType::DateTime
        | InteractiveTableDataType::Text => compare_text_table_values(left, right),
    }
}

fn compare_numeric_table_values(left: &str, right: &str) -> Ordering {
    match (left.trim().parse::<f64>(), right.trim().parse::<f64>()) {
        (Ok(left_number), Ok(right_number)) => left_number
            .total_cmp(&right_number)
            .then_with(|| compare_text_table_values(left, right)),
        (Ok(_), Err(_)) => Ordering::Less,
        (Err(_), Ok(_)) => Ordering::Greater,
        (Err(_), Err(_)) => compare_text_table_values(left, right),
    }
}

fn compare_text_table_values(left: &str, right: &str) -> Ordering {
    left.to_lowercase()
        .cmp(&right.to_lowercase())
        .then_with(|| left.cmp(right))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sorted_values(data_type: InteractiveTableDataType, values: &[&str]) -> Vec<String> {
        let mut values = values
            .iter()
            .map(|value| (*value).to_string())
            .collect::<Vec<_>>();
        values.sort_by(|left, right| compare_table_values(data_type, left, right));
        values
    }

    #[test]
    fn text_and_number_columns_use_one_comparator_for_the_entire_column() {
        let values = ["10", "2", "15a"];

        assert_eq!(
            sorted_values(InteractiveTableDataType::Number, &values),
            ["2", "10", "15a"]
        );
        assert_eq!(
            sorted_values(InteractiveTableDataType::Text, &values),
            ["10", "15a", "2"]
        );
    }

    #[test]
    fn numeric_comparator_is_transitive_for_mixed_values() {
        let two_to_ten = compare_table_values(InteractiveTableDataType::Number, "2", "10");
        let ten_to_text = compare_table_values(InteractiveTableDataType::Number, "10", "15a");
        let two_to_text = compare_table_values(InteractiveTableDataType::Number, "2", "15a");

        assert_eq!(two_to_ten, Ordering::Less);
        assert_eq!(ten_to_text, Ordering::Less);
        assert_eq!(two_to_text, Ordering::Less);
    }

    #[test]
    fn numeric_comparator_orders_special_floats_and_invalid_values_deterministically() {
        let values = ["invalid", "NaN", "-inf", "0", "inf", "nan"];
        let first = sorted_values(InteractiveTableDataType::Number, &values);
        let second = sorted_values(InteractiveTableDataType::Number, &values);

        assert_eq!(first, second);
        assert_eq!(first.last().map(String::as_str), Some("invalid"));
        assert!(
            first.iter().position(|value| value == "-inf")
                < first.iter().position(|value| value == "0")
        );
        assert!(
            first.iter().position(|value| value == "0")
                < first.iter().position(|value| value == "inf")
        );
    }

    #[test]
    fn field_type_mapping_is_case_insensitive_and_defaults_to_text() {
        assert_eq!(
            InteractiveTableDataType::from_field_type(" Number "),
            InteractiveTableDataType::Number
        );
        assert_eq!(
            InteractiveTableDataType::from_field_type("TIMESTAMP"),
            InteractiveTableDataType::DateTime
        );
        assert_eq!(
            InteractiveTableDataType::from_field_type("static_text"),
            InteractiveTableDataType::Text
        );
    }
}
