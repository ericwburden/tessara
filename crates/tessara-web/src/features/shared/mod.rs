use crate::ui::empty_view;
use crate::utils::metadata::metadata_label as filter_metadata_label;
use crate::utils::text::text_matches as filter_text_matches;
use icons::{ListFilter, Search};
use leptos::prelude::*;

mod utility_exports {
    use crate::utils::{metadata, text};

    pub(crate) use metadata::{metadata_label, metadata_rows};
    pub(crate) use text::{nonempty_text, sentence_label};
}
pub(crate) use utility_exports::*;

#[cfg(feature = "hydrate")]
pub(crate) fn navigate_to_href(href: &str) {
    if let Some(window) = web_sys::window() {
        let _ = window.location().set_href(href);
    }
}

mod shared {
    use crate::features::shared_data;

    pub(crate) use shared_data::*;
}
pub(crate) use shared::*;

mod helpers;
mod placeholder;
mod ui;
pub(crate) use placeholder::NativePlaceholderRoute;
pub(crate) use ui::{
    FormBuilderGridCell, FormBuilderSectionLayout, WorkflowSourceMarker,
    active_workflow_definition_version, blank_form_builder_field_at, form_attached_nodes,
    form_builder_field_default_label, form_builder_field_type_icon, form_builder_occupancy_map,
    form_builder_section_fields, form_builder_section_layout, form_definition_scope_label,
    form_field_count_label, form_version_desc_sort_key, node_count_label, node_display_path,
    rendered_field_layout_label, rendered_field_type_label, response_selected_assignment,
    response_start_can_submit, submission_assignee_label, submission_progress_label,
    submission_status_key, submission_status_label, submission_step_label,
    submission_workflow_label, user_count_label, workflow_assigned_user_links,
    workflow_assignee_label, workflow_assignment_assignee_label, workflow_assignment_candidate_key,
    workflow_assignment_revision_label, workflow_assignment_state, workflow_assignment_state_label,
    workflow_assignment_status_key, workflow_assignment_status_label,
    workflow_available_node_links, workflow_available_nodes_label,
    workflow_definition_status_label, workflow_definition_version_label,
    workflow_description_label, workflow_source_label, workflow_status_key, workflow_status_label,
    workflow_version_label,
};
#[cfg(feature = "hydrate")]
pub(crate) use ui::{
    collect_response_values, prepared_form_builder_fields, prepared_form_builder_sections,
    submission_value_maps,
};

mod filtering;
pub(crate) use filtering::*;

#[component]
pub(crate) fn FilterHeader(
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
            format!("Filter {label}: {}", filter_metadata_label(&current))
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
                filter_text_matches(
                    &query,
                    &[option.as_str(), filter_metadata_label(option).as_str()],
                )
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
                                let option_label = filter_metadata_label(&option);
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
