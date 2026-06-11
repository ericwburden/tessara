//! List view components for the Forms feature.
//!
//! Keep collection tables, list filters, and list-page presentation here; detail/editor flows should stay in their dedicated modules.

use crate::features::forms::FormSummary;
use crate::features::forms::{form_attached_nodes, form_field_count_label, form_status_label};
use crate::features::organization::{active_form_version, form_version_label};
use crate::features::shared::{
    FilterHeader as SharedFilterHeader, FormAttachmentLink, FormNodeFilterOption,
    FormsAttachedNodesSheetData, indented_node_label, node_count_label, status_badge_class,
    visible_form_node_filter_options,
};
use crate::ui::{DataTable, empty_view};
use icons::{ChevronDown, ExternalLink, ListFilter, PanelRight, Search, X};
use leptos::portal::Portal;
use leptos::prelude::*;

#[component]
/// Renders the forms node lineage filter view.
pub(crate) fn FormsNodeLineageFilter(
    options: Vec<FormNodeFilterOption>,
    selected_node_id: RwSignal<Option<String>>,
    query: RwSignal<String>,
) -> impl IntoView {
    let is_open = RwSignal::new(false);
    let options_for_visible = options.clone();
    let options_for_label = options.clone();
    let options_for_selected = options.clone();
    let trigger_label = move || {
        let selected = selected_node_id.get();
        selected
            .as_deref()
            .and_then(|id| {
                options_for_label
                    .iter()
                    .find(|option| option.id == id)
                    .map(|option| option.name.clone())
            })
            .unwrap_or_else(|| "Filter by node".to_string())
    };
    let trigger_class = move || {
        if selected_node_id.get().is_none() {
            "forms-node-filter__trigger"
        } else {
            "forms-node-filter__trigger is-filtered"
        }
    };
    let visible_options = move || {
        visible_form_node_filter_options(
            &options_for_visible,
            selected_node_id.get().as_deref(),
            &query.get(),
        )
    };
    let selected_options = move || {
        selected_node_id
            .get()
            .as_deref()
            .and_then(|selected| {
                options_for_selected
                    .iter()
                    .find(|option| option.id == selected)
                    .cloned()
            })
            .into_iter()
            .collect::<Vec<_>>()
    };

    view! {
        <div class=move || if is_open.get() { "forms-node-filter is-open" } else { "forms-node-filter" }>
            <button
                class=trigger_class
                type="button"
                role="combobox"
                aria-haspopup="listbox"
                aria-expanded=move || is_open.get().to_string()
                aria-label="Filter forms by organization node"
                title="Filter forms by organization node"
                on:click=move |_| is_open.update(|open| *open = !*open)
            >
                <ListFilter/>
                <span>{trigger_label}</span>
                <ChevronDown/>
            </button>
            <button
                class="forms-node-filter__scrim"
                type="button"
                aria-label="Close node filter"
                on:click=move |_| is_open.set(false)
            ></button>
            <div
                class="forms-node-filter__menu blurred-surface floating-layer"
                data-mobile-behavior="dialog"
                role="dialog"
                aria-label="Filter by organization node"
            >
                <label class="forms-node-filter__search">
                    <Search/>
                    <span class="sr-only">"Search organization nodes"</span>
                    <input
                        type="search"
                        placeholder="Search organization nodes"
                        prop:value=move || query.get()
                        on:input=move |event| query.set(event_target_value(&event))
                    />
                </label>
                <div class="forms-node-filter__selected">
                    {move || {
                        let selected = selected_options();
                        if selected.is_empty() {
                            view! { <p class="forms-node-filter__empty">"No node selected"</p> }.into_any()
                        } else {
                            view! {
                                <div class="forms-node-filter__chips">
                                    {selected
                                        .into_iter()
                                        .map(|option| {
                                            let option_id = option.id.clone();
                                            let selected_node_id_for_chip = selected_node_id;
                                            let query_for_chip = query;
                                            view! {
                                                <button
                                                    class="forms-node-filter__chip"
                                                    type="button"
                                                    on:click=move |_| {
                                                        selected_node_id_for_chip.set(Some(option_id.clone()));
                                                        query_for_chip.set(String::new());
                                                    }
                                                >
                                                    <span>{option.name}</span>
                                                </button>
                                            }
                                        })
                                        .collect_view()}
                                </div>
                            }
                            .into_any()
                        }
                    }}
                    {move || {
                        if selected_node_id.get().is_some() {
                            view! {
                                <button
                                    class="forms-node-filter__clear"
                                    type="button"
                                    on:click=move |_| {
                                        selected_node_id.set(None);
                                        query.set(String::new());
                                    }
                                >
                                    "Clear node filter"
                                </button>
                            }
                            .into_any()
                        } else {
                            view! { <span></span> }.into_any()
                        }
                    }}
                </div>
                <div class="forms-node-filter__options" role="listbox">
                    {move || {
                        let visible = visible_options();
                        if visible.is_empty() {
                            view! { <p class="forms-node-filter__empty">"No matching nodes"</p> }.into_any()
                        } else {
                            visible
                                .into_iter()
                                .map(|option| {
                                    let option_id = option.id.clone();
                                    let selected_node_id_for_option = selected_node_id;
                                    let query_for_option = query;
                                    let is_open_for_option = is_open;
                                    let is_selected = selected_node_id
                                        .get()
                                        .as_deref()
                                        .is_some_and(|selected| selected == option_id.as_str());
                                    view! {
                                        <button
                                            class=if is_selected { "forms-node-filter__option is-active" } else { "forms-node-filter__option" }
                                            type="button"
                                            role="option"
                                            aria-selected=is_selected.to_string()
                                            on:click=move |_| {
                                                selected_node_id_for_option.set(Some(option_id.clone()));
                                                query_for_option.set(String::new());
                                                is_open_for_option.set(false);
                                            }
                                        >
                                            <span>{indented_node_label(&option)}</span>
                                        </button>
                                    }
                                })
                                .collect_view()
                                .into_any()
                        }
                    }}
                </div>
            </div>
        </div>
    }
}

#[component]
/// Renders the forms list view.
pub(crate) fn FormsList(
    forms: Vec<FormSummary>,
    search: RwSignal<String>,
    status_filter: RwSignal<String>,
    node_filter_query: RwSignal<String>,
    selected_node_id: RwSignal<Option<String>>,
    status_options: Vec<String>,
    node_filter_options: Vec<FormNodeFilterOption>,
) -> impl IntoView {
    let mut table_forms = forms.clone();
    table_forms.sort_by(|left, right| {
        left.name
            .to_lowercase()
            .cmp(&right.name.to_lowercase())
            .then(left.id.cmp(&right.id))
    });
    let card_forms = table_forms.clone();
    let page_size = RwSignal::new(10usize);
    let page_index = RwSignal::new(0usize);
    let total_count = table_forms.len();
    let page_count = move || {
        if total_count == 0 {
            1
        } else {
            total_count.div_ceil(page_size.get()).max(1)
        }
    };
    let current_page = move || page_index.get().min(page_count() - 1);
    let page_start = move || {
        if total_count == 0 {
            0
        } else {
            current_page() * page_size.get()
        }
    };
    let page_end = move || (page_start() + page_size.get()).min(total_count);
    let page_summary = move || {
        if total_count == 0 {
            "No forms to display".to_string()
        } else {
            format!(
                "Showing {}-{} of {} forms",
                page_start() + 1,
                page_end(),
                total_count
            )
        }
    };
    let attached_nodes_sheet = RwSignal::new(None::<FormsAttachedNodesSheetData>);

    view! {
        <div class="forms-list forms-list-responsive-table">
            <div class="searchable-data-table">
                <div class="searchable-data-table__toolbar forms-list__toolbar">
                    <label class="searchable-data-table__search searchable-data-table__control">
                        <Search class="searchable-data-table__control-icon"/>
                        <span class="sr-only">"Search forms"</span>
                        <input
                            type="search"
                            placeholder="Search forms"
                            prop:value=move || search.get()
                            on:input=move |event| search.set(event_target_value(&event))
                        />
                    </label>
                </div>
                <DataTable>
                <thead>
                    <tr>
                        <th scope="col">"Form name"</th>
                        <th scope="col">
                            <div class="data-table-filter">
                                <span>"Attached To"</span>
                                <FormsNodeLineageFilter
                                    options=node_filter_options
                                    selected_node_id
                                    query=node_filter_query
                                />
                            </div>
                        </th>
                        <th class="data-table__cell--center" scope="col">"Active version"</th>
                        <th class="data-table__cell--center" scope="col">
                            <SharedFilterHeader
                                label="Status"
                                all_label="All statuses"
                                filter=status_filter
                                options=status_options
                            />
                        </th>
                        <th class="data-table__cell--center" scope="col">"Fields"</th>
                    </tr>
                </thead>
                <tbody>
                    {move || if table_forms.is_empty() {
                        view! {
                            <tr>
                                <td class="data-table__empty" colspan="5">"No Forms to Display"</td>
                            </tr>
                        }
                        .into_any()
                    } else {
                        table_forms
                            .iter()
                            .skip(page_start())
                            .take(page_size.get())
                            .cloned()
                            .map(|form| {
                                let href = format!("/forms/{}", form.id);
                                let active_version = active_form_version(&form);
                                let status = active_version
                                    .map(|version| version.status.as_str())
                                    .unwrap_or("none");
                                let name = form.name.clone();
                                let attached_nodes = form_attached_nodes(active_version);
                                let attached_nodes_form_name = name.clone();
                                let version_label = form_version_label(active_version);
                                let status_label = form_status_label(active_version);
                                let field_count = form_field_count_label(active_version);
                                view! {
                                    <tr>
                                        <th scope="row">
                                            <a class="data-table__primary-link" href=href.clone()>{name}</a>
                                        </th>
                                        <td>
                                            <FormsAttachedNodesList
                                                nodes=attached_nodes
                                                form_name=attached_nodes_form_name
                                                form_href=href
                                                sheet=attached_nodes_sheet
                                            />
                                        </td>
                                        <td class="data-table__cell--center">{version_label}</td>
                                        <td class="data-table__cell--center"><span class=status_badge_class(status)>{status_label}</span></td>
                                        <td class="data-table__cell--center">{field_count}</td>
                                    </tr>
                                }
                            })
                            .collect_view()
                            .into_any()
                    }}
                </tbody>
                </DataTable>
                <div class="directory-table-pagination" aria-label="Forms table pagination">
                    <p>{move || page_summary()}</p>
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
                            class="button button--compact button--secondary"
                            type="button"
                            disabled=move || current_page() == 0
                            on:click=move |_| {
                                page_index.update(|page| *page = page.saturating_sub(1));
                            }
                        >
                            "Previous"
                        </button>
                        <span>{move || format!("Page {} of {}", current_page() + 1, page_count())}</span>
                        <button
                            class="button button--compact button--secondary"
                            type="button"
                            disabled=move || { current_page() + 1 >= page_count() }
                            on:click=move |_| {
                                let last_page = page_count().saturating_sub(1);
                                page_index.update(|page| *page = (*page + 1).min(last_page));
                            }
                        >
                            "Next"
                        </button>
                    </div>
                </div>
            </div>
            <div class="forms-list-mobile-cards">
                {move || if card_forms.is_empty() {
                    view! { <p class="forms-list-mobile-empty">"No Forms to Display"</p> }.into_any()
                } else {
                    card_forms
                        .iter()
                        .skip(page_start())
                        .take(page_size.get())
                        .cloned()
                        .map(|form| {
                            let href = format!("/forms/{}", form.id);
                            let active_version = active_form_version(&form);
                            let status = active_version
                                .map(|version| version.status.as_str())
                                .unwrap_or("none");
                            let name = form.name.clone();
                            let attached_nodes = form_attached_nodes(active_version);
                            let attached_nodes_form_name = name.clone();
                            let version_label = form_version_label(active_version);
                            let status_label = form_status_label(active_version);
                            let field_count = form_field_count_label(active_version);
                            view! {
                                <article class="forms-list-mobile-card">
                                    <div class="forms-list-mobile-card__header">
                                        <div>
                                            <h3><a href=href.clone()>{name}</a></h3>
                                        </div>
                                    </div>
                                    <dl>
                                        <div>
                                            <dt>"Attached To"</dt>
                                            <dd>
                                                <FormsAttachedNodesList
                                                    nodes=attached_nodes
                                                    form_name=attached_nodes_form_name
                                                    form_href=href
                                                    sheet=attached_nodes_sheet
                                                />
                                            </dd>
                                        </div>
                                        <div>
                                            <dt>"Active version"</dt>
                                            <dd>{version_label}</dd>
                                        </div>
                                        <div>
                                            <dt>"Status"</dt>
                                            <dd><span class=status_badge_class(status)>{status_label}</span></dd>
                                        </div>
                                        <div>
                                            <dt>"Fields"</dt>
                                            <dd>{field_count}</dd>
                                        </div>
                                    </dl>
                                </article>
                            }
                        })
                        .collect_view()
                        .into_any()
                }}
            </div>
            <FormsAttachedNodesSheet detail=attached_nodes_sheet/>
        </div>
    }
}

#[component]
/// Renders the forms attached nodes list view.
pub(crate) fn FormsAttachedNodesList(
    nodes: Vec<FormAttachmentLink>,
    form_name: String,
    form_href: String,
    sheet: RwSignal<Option<FormsAttachedNodesSheetData>>,
) -> impl IntoView {
    let total_nodes = nodes.len();
    let nodes_for_sheet = nodes.clone();
    let form_name_for_sheet = form_name.clone();
    let form_href_for_sheet = form_href.clone();

    view! {
        <div class="forms-attached-list">
            {if total_nodes == 0 {
                view! { <p>"Not attached"</p> }.into_any()
            } else if total_nodes > 0 {
                view! {
                    <button
                        class="forms-attached-list__more"
                        type="button"
                        aria-label=format!("View attached organization nodes for {form_name_for_sheet}")
                        title="Opens detail panel"
                        on:click=move |_| {
                            sheet.set(Some(FormsAttachedNodesSheetData {
                                form_name: form_name_for_sheet.clone(),
                                form_href: form_href_for_sheet.clone(),
                                nodes: nodes_for_sheet.clone(),
                            }));
                        }
                    >
                        <span>{node_count_label(total_nodes)}</span>
                        <PanelRight class="forms-attached-list__icon"/>
                    </button>
                }
                .into_any()
            } else {
                empty_view()
            }}
        </div>
    }
}

#[component]
/// Renders the forms attached nodes sheet view.
pub(crate) fn FormsAttachedNodesSheet(
    detail: RwSignal<Option<FormsAttachedNodesSheetData>>,
) -> impl IntoView {
    let search = RwSignal::new(String::new());
    let close = move |_| {
        detail.set(None);
        search.set(String::new());
    };
    let filtered_nodes = move || {
        let query = search.get().trim().to_lowercase();
        detail
            .get()
            .map(|data| {
                data.nodes
                    .into_iter()
                    .filter(|node| {
                        query.is_empty()
                            || node.label.to_lowercase().contains(&query)
                            || node.title.to_lowercase().contains(&query)
                    })
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default()
    };

    view! {
        <Portal>
            <Show when=move || detail.get().is_some()>
                <section class="sheet-overlay forms-attached-overlay" aria-label="Attached organization nodes">
                    <button class="sheet-overlay__scrim" type="button" aria-label="Close attached nodes" on:click=close></button>
                    <aside class="sheet-panel blurred-surface forms-attached-sheet" role="dialog" aria-modal="true" aria-label="Attached organization nodes">
                        <div class="sheet-panel__actions">
                            {move || {
                                detail
                                    .get()
                                    .map(|data| {
                                        view! {
                                            <a class="icon-button sheet-panel__open" href=data.form_href aria-label="Open form detail" title="Open form detail">
                                                <ExternalLink class="icon-button__icon"/>
                                            </a>
                                        }
                                        .into_any()
                                    })
                                    .unwrap_or_else(empty_view)
                            }}
                            <button class="icon-button sheet-panel__close" type="button" aria-label="Close attached nodes" title="Close attached nodes" on:click=close>
                                <X class="icon-button__icon"/>
                            </button>
                        </div>
                        {move || {
                            detail
                                .get()
                                .map(|data| {
                                    let total = data.nodes.len();
                                    view! {
                                        <header class="sheet-panel__header">
                                            <p>"Attached Nodes"</p>
                                            <h2>{data.form_name}</h2>
                                            <span class="forms-attached-sheet__count">{format!("{total} nodes")}</span>
                                        </header>
                                        <section class="sheet-panel__section">
                                            <label class="searchable-data-table__search searchable-data-table__control forms-attached-sheet__search">
                                                <Search/>
                                                <span class="sr-only">"Search attached nodes"</span>
                                                <input
                                                    type="search"
                                                    placeholder="Search attached nodes"
                                                    prop:value=move || search.get()
                                                    on:input=move |event| search.set(event_target_value(&event))
                                                />
                                            </label>
                                            <div class="forms-attached-sheet__list">
                                                {move || {
                                                    let nodes = filtered_nodes();
                                                    if nodes.is_empty() {
                                                        view! { <p class="forms-attached-sheet__empty">"No Attached Nodes to Display"</p> }.into_any()
                                                    } else {
                                                        nodes
                                                            .into_iter()
                                                            .map(|node| {
                                                                let node_title = node.title.clone();
                                                                view! {
                                                                    <a class="forms-attached-sheet__item" href=node.href title=node_title>
                                                                        <span>{node.label}</span>
                                                                        <small>{node.title}</small>
                                                                    </a>
                                                                }
                                                            })
                                                            .collect_view()
                                                            .into_any()
                                                    }
                                                }}
                                            </div>
                                        </section>
                                    }
                                    .into_any()
                                })
                                .unwrap_or_else(empty_view)
                        }}
                    </aside>
                </section>
            </Show>
        </Portal>
    }
}
