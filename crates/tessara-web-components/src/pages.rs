//! Route-level page composition for the Components feature.

mod editor;
#[cfg(any(feature = "hydrate", test))]
mod editor_config;

use editor::*;
#[cfg(any(feature = "hydrate", test))]
use editor_config::*;

use std::collections::BTreeMap;

#[cfg(feature = "hydrate")]
use super::types::{
    CreateComponentVersionRequest, SaveComponentEditRequest, UpdateComponentRequest,
};
use icons::{ChevronDown, CircleHelp, History, ListFilter, PanelRight, Pencil, Search, X};
use leptos::prelude::*;
#[cfg(feature = "hydrate")]
use leptos::wasm_bindgen::{JsCast, closure::Closure};
use tessara_web_data_ops::{
    DataOpsFiltersEditor, DataOpsProjectionEditor, DatasetFieldDraft as DataOpsDatasetFieldDraft,
    DatasetRowFilterDraft as DataOpsRowFilterDraft,
};
use tessara_web_ui::{
    Breadcrumb, BreadcrumbItem, BreadcrumbLink, BreadcrumbPage, BreadcrumbSeparator, DataTable,
    EmptyState, InteractiveDataTable, InteractiveTableColumn, InteractiveTableRow, PageHeader,
    Skeleton, TableFilterHeader, TablePaginationFooter,
};

#[cfg(feature = "hydrate")]
use super::api;
use super::types::{
    ComponentDefinition, ComponentSummary, ComponentTable, ComponentValidationFinding,
    ComponentVersionSummary, ComponentVisual, DatasetFieldDefinition, DatasetSummary,
};

#[component]
pub fn ComponentsIndexContent() -> impl IntoView {
    let components = RwSignal::new(Vec::<ComponentSummary>::new());
    let is_loading = RwSignal::new(true);
    let load_error = RwSignal::new(None::<String>);
    let can_manage_components = RwSignal::new(false);

    Effect::new(move |_| {
        load_components(components, is_loading, load_error, can_manage_components);
    });

    view! {
        <section class="route-panel components-page">
            <div class="page-header">
                <div></div>
                {move || can_manage_components.get().then(|| view! {
                    <div class="page-header__actions">
                        <a class="button" href="/components/new">"Create Component"</a>
                    </div>
                })}
            </div>
            {move || {
                if is_loading.get() {
                    view! { <EmptyState title="Loading components" message="Fetching visible components."/> }.into_any()
                } else if let Some(message) = load_error.get() {
                    view! { <EmptyState title="Components unavailable" message=message/> }.into_any()
                } else if components.get().is_empty() {
                    view! { <EmptyState title="No visible components" message="No components are visible for the current account."/> }.into_any()
                } else {
                    view! { <ComponentsTable components=components.get() can_manage=can_manage_components.get()/> }.into_any()
                }
            }}
        </section>
    }
}

#[component]
pub fn ComponentVersionsContent(component_ref: String) -> impl IntoView {
    let component = RwSignal::new(None::<ComponentDefinition>);
    let is_loading = RwSignal::new(true);
    let load_error = RwSignal::new(None::<String>);
    let can_manage_component = RwSignal::new(false);

    Effect::new({
        let component_ref = component_ref.clone();
        move |_| {
            load_component(
                component_ref.clone(),
                component,
                is_loading,
                load_error,
                can_manage_component,
            )
        }
    });

    view! {
        <section class="route-panel components-page">
            {move || {
                if is_loading.get() {
                    view! { <EmptyState title="Loading component" message="Fetching component detail."/> }.into_any()
                } else if let Some(message) = load_error.get() {
                    view! { <EmptyState title="Component unavailable" message=message/> }.into_any()
                } else if let Some(component) = component.get() {
                    let edit_href = format!("/components/{}/edit", component.slug);
                    let view_href = format!("/components/{}", component.slug);
                    view! {
                        <ComponentNestedBreadcrumb
                            component_href=view_href.clone()
                            component_label=component.name.clone()
                            current="Versions"
                        />
                        <PageHeader title=component.name.clone()>
                            {can_manage_component.get().then(|| view! {
                                <a class="button button--secondary" href=edit_href>"Edit"</a>
                            })}
                            <a class="button" href=view_href>"View"</a>
                        </PageHeader>
                        <ComponentVersionsSection versions=component.versions/>
                    }.into_any()
                } else {
                    view! { <EmptyState title="Component unavailable" message="Component detail could not be loaded."/> }.into_any()
                }
            }}
        </section>
    }
}

#[component]
pub fn ComponentEditorContent(component_ref: Option<String>) -> impl IntoView {
    let title = if component_ref.is_some() {
        "Edit Component"
    } else {
        "Create Component"
    };
    let message = RwSignal::new(None::<String>);
    let error = RwSignal::new(None::<String>);
    let validation_findings = RwSignal::new(Vec::<ComponentValidationFinding>::new());
    let datasets = RwSignal::new(Vec::<DatasetSummary>::new());
    let dataset_error = RwSignal::new(None::<String>);
    let name = RwSignal::new(String::new());
    let slug = RwSignal::new(String::new());
    let slug_manually_edited = RwSignal::new(false);
    let description = RwSignal::new(String::new());
    let dataset_id = RwSignal::new(String::new());
    let dataset_major = RwSignal::new(String::from("1"));
    let dataset_picker_open = RwSignal::new(false);
    let dataset_picker_search = RwSignal::new(String::new());
    let dataset_picker_active_index = RwSignal::new(0_usize);
    let dataset_picker_search_input = NodeRef::<leptos::html::Input>::new();
    let component_type = RwSignal::new(String::from("table"));
    let columns = RwSignal::new(Vec::<DataOpsDatasetFieldDraft>::new());
    let filters = RwSignal::new(Vec::<DataOpsRowFilterDraft>::new());
    let projection_active_source_tab = RwSignal::new(None::<String>);
    let sort_field = RwSignal::new(String::new());
    let sort_direction = RwSignal::new(String::from("asc"));
    let page_size = RwSignal::new(String::from("50"));
    let visual_summary_field = RwSignal::new(String::new());
    let visual_summary_type = RwSignal::new(String::from("count"));
    let visual_category_field = RwSignal::new(String::new());
    let visual_category_labels = RwSignal::new(String::new());
    let visual_category_colors = RwSignal::new(String::new());
    let visual_legend_title = RwSignal::new(String::new());
    let visual_comparison_field = RwSignal::new(String::new());
    let visual_bar_orientation = RwSignal::new(String::from("horizontal"));
    let visual_bar_comparison_layout = RwSignal::new(String::from("grouped"));
    let visual_x_axis_label = RwSignal::new(String::new());
    let visual_y_axis_label = RwSignal::new(String::new());
    let visual_x_field = RwSignal::new(String::new());
    let visual_line_smoothing = RwSignal::new(true);
    let visual_sort_field = RwSignal::new(String::new());
    let visual_sort_direction = RwSignal::new(String::from("asc"));
    let visual_limit = RwSignal::new(String::from("20"));
    let visual_value_format = RwSignal::new(String::from("plain"));
    let visual_category_missing_policy = RwSignal::new(String::from("omit"));
    let visual_comparison_missing_policy = RwSignal::new(String::from("omit"));
    let visual_missing_policy = RwSignal::new(String::from("omit"));
    let stat_label = RwSignal::new(String::new());
    let stat_supporting_text = RwSignal::new(String::new());
    let stat_panel_style = RwSignal::new(String::from("default"));
    let editing_component_id = RwSignal::new(None::<String>);
    let editing_version_id = RwSignal::new(None::<String>);
    let current_published_version_id = RwSignal::new(None::<String>);
    let publish_menu_open = RwSignal::new(false);
    let preview_drawer_open = RwSignal::new(false);
    let consumer_modal_open = RwSignal::new(false);
    let consumer_search = RwSignal::new(String::new());
    let new_version_note = RwSignal::new(String::new());
    let draft_preview = RwSignal::new(None::<ComponentVisual>);
    let draft_preview_error = RwSignal::new(None::<String>);
    let draft_preview_loading = RwSignal::new(false);
    let draft_preview_generation = RwSignal::new(0_u64);
    let draft_preview_timeout = RwSignal::new(None::<i32>);

    let selected_fields = Memo::new(move |_| {
        let selected_major = dataset_major.get().trim().parse::<i32>().ok();
        datasets
            .get()
            .into_iter()
            .find(|dataset| dataset.id == dataset_id.get())
            .and_then(|dataset| {
                selected_major.map(|major| dataset_fields_for_major(&dataset, major))
            })
            .unwrap_or_default()
    });
    let select_dataset_version =
        Callback::new(move |(selected_dataset_id, major): (String, i32)| {
            dataset_id.set(selected_dataset_id);
            dataset_major.set(major.to_string());
            columns.set(Vec::new());
            filters.set(Vec::new());
            sort_field.set(String::new());
            visual_category_labels.set(String::new());
            visual_category_colors.set(String::new());
            dataset_picker_search.set(String::new());
            dataset_picker_active_index.set(0);
            dataset_picker_open.set(false);
        });
    Effect::new(move |_| {
        if dataset_picker_open.get()
            && let Some(input) = dataset_picker_search_input.get()
        {
            let _ = input.focus();
        }
    });
    Effect::new(move |_| load_datasets(datasets, dataset_error));
    Effect::new({
        let component_ref = component_ref.clone();
        move |_| {
            if let Some(component_ref) = component_ref.clone() {
                load_component_for_edit(
                    component_ref,
                    editing_component_id,
                    editing_version_id,
                    current_published_version_id,
                    name,
                    slug,
                    description,
                    dataset_id,
                    dataset_major,
                    component_type,
                    columns,
                    filters,
                    sort_field,
                    sort_direction,
                    page_size,
                    visual_summary_field,
                    visual_summary_type,
                    visual_category_field,
                    visual_category_labels,
                    visual_category_colors,
                    visual_legend_title,
                    visual_comparison_field,
                    visual_bar_orientation,
                    visual_bar_comparison_layout,
                    visual_x_axis_label,
                    visual_y_axis_label,
                    visual_x_field,
                    visual_line_smoothing,
                    visual_sort_field,
                    visual_sort_direction,
                    visual_limit,
                    visual_value_format,
                    visual_category_missing_policy,
                    visual_comparison_missing_policy,
                    visual_missing_policy,
                    stat_label,
                    stat_supporting_text,
                    stat_panel_style,
                    slug_manually_edited,
                    error,
                );
            }
        }
    });
    Effect::new(move |_| {
        let values = ComponentFormValues {
            name: name.get(),
            slug: slug.get(),
            description: description.get(),
            dataset_id: dataset_id.get(),
            dataset_major: dataset_major.get(),
            columns: columns.get(),
            filters: filters.get(),
            sort_field: sort_field.get(),
            sort_direction: sort_direction.get(),
            page_size: page_size.get(),
            component_type: component_type.get(),
            visual_summary_field: visual_summary_field.get(),
            visual_summary_type: visual_summary_type.get(),
            visual_category_field: visual_category_field.get(),
            visual_category_labels: visual_category_labels.get(),
            visual_category_colors: visual_category_colors.get(),
            visual_legend_title: visual_legend_title.get(),
            visual_comparison_field: visual_comparison_field.get(),
            visual_bar_orientation: visual_bar_orientation.get(),
            visual_bar_comparison_layout: visual_bar_comparison_layout.get(),
            visual_x_axis_label: visual_x_axis_label.get(),
            visual_y_axis_label: visual_y_axis_label.get(),
            visual_x_field: visual_x_field.get(),
            visual_line_smoothing: visual_line_smoothing.get(),
            visual_sort_field: visual_sort_field.get(),
            visual_sort_direction: visual_sort_direction.get(),
            visual_limit: visual_limit.get(),
            visual_value_format: visual_value_format.get(),
            visual_category_missing_policy: visual_category_missing_policy.get(),
            visual_comparison_missing_policy: visual_comparison_missing_policy.get(),
            visual_missing_policy: visual_missing_policy.get(),
            stat_label: stat_label.get(),
            stat_supporting_text: stat_supporting_text.get(),
            stat_panel_style: stat_panel_style.get(),
        };
        schedule_component_editor_preview(
            values,
            draft_preview,
            draft_preview_error,
            draft_preview_loading,
            draft_preview_generation,
            draft_preview_timeout,
        );
    });
    let has_kind_specific_changes = Signal::derive(move || match component_type.get().as_str() {
        "table" => !columns.get().is_empty() || !sort_field.get().trim().is_empty(),
        _ => {
            !visual_summary_field.get().trim().is_empty()
                || !visual_category_field.get().trim().is_empty()
                || !visual_comparison_field.get().trim().is_empty()
                || !visual_x_field.get().trim().is_empty()
                || !stat_label.get().trim().is_empty()
        }
    });
    let change_component_kind = Callback::new(move |next_kind: String| {
        if next_kind == component_type.get_untracked() {
            return;
        }
        columns.set(Vec::new());
        sort_field.set(String::new());
        sort_direction.set("asc".into());
        page_size.set("50".into());
        visual_summary_field.set(String::new());
        visual_summary_type.set("count".into());
        visual_category_field.set(String::new());
        visual_category_labels.set(String::new());
        visual_category_colors.set(String::new());
        visual_legend_title.set(String::new());
        visual_comparison_field.set(String::new());
        visual_bar_orientation.set("horizontal".into());
        visual_bar_comparison_layout.set("grouped".into());
        visual_x_axis_label.set(String::new());
        visual_y_axis_label.set(String::new());
        visual_x_field.set(String::new());
        visual_line_smoothing.set(true);
        visual_sort_field.set(String::new());
        visual_sort_direction.set("asc".into());
        visual_limit.set("20".into());
        visual_value_format.set("plain".into());
        visual_category_missing_policy.set("omit".into());
        visual_comparison_missing_policy.set("omit".into());
        visual_missing_policy.set("omit".into());
        stat_label.set(String::new());
        stat_supporting_text.set(String::new());
        stat_panel_style.set("default".into());
        component_type.set(next_kind);
        focus_component_kind_editor();
    });

    view! {
        <section
            class="route-panel components-page"
            on:click=move |_| dataset_picker_open.set(false)
        >
            <ComponentsBreadcrumb current=title/>
            <PageHeader title/>
            <form
                class="route-panel__section form-grid component-editor-form"
                on:submit=move |event| {
                event.prevent_default();
                create_component_from_form(
                    ComponentSaveIntent {
                        editing_component_id: editing_component_id.get_untracked(),
                        editing_version_id: editing_version_id.get_untracked(),
                        current_published_version_id: current_published_version_id.get_untracked(),
                        publish_action: ComponentPublishAction::SaveDraft,
                        version_note: None,
                    },
                    ComponentFormValues {
                        name: name.get_untracked(),
                        slug: slug.get_untracked(),
                        description: description.get_untracked(),
                        dataset_id: dataset_id.get_untracked(),
                        dataset_major: dataset_major.get_untracked(),
                        columns: columns.get_untracked(),
                        filters: filters.get_untracked(),
                        sort_field: sort_field.get_untracked(),
                        sort_direction: sort_direction.get_untracked(),
                        page_size: page_size.get_untracked(),
                        component_type: component_type.get_untracked(),
                        visual_summary_field: visual_summary_field.get_untracked(),
                        visual_summary_type: visual_summary_type.get_untracked(),
                        visual_category_field: visual_category_field.get_untracked(),
                        visual_category_labels: visual_category_labels.get_untracked(),
                        visual_category_colors: visual_category_colors.get_untracked(),
                        visual_legend_title: visual_legend_title.get_untracked(),
                        visual_comparison_field: visual_comparison_field.get_untracked(),
                        visual_bar_orientation: visual_bar_orientation.get_untracked(),
                        visual_bar_comparison_layout: visual_bar_comparison_layout.get_untracked(),
                        visual_x_axis_label: visual_x_axis_label.get_untracked(),
                        visual_y_axis_label: visual_y_axis_label.get_untracked(),
                        visual_x_field: visual_x_field.get_untracked(),
                        visual_line_smoothing: visual_line_smoothing.get_untracked(),
                        visual_sort_field: visual_sort_field.get_untracked(),
                        visual_sort_direction: visual_sort_direction.get_untracked(),
                        visual_limit: visual_limit.get_untracked(),
                        visual_value_format: visual_value_format.get_untracked(),
                        visual_category_missing_policy: visual_category_missing_policy.get_untracked(),
                        visual_comparison_missing_policy: visual_comparison_missing_policy.get_untracked(),
                        visual_missing_policy: visual_missing_policy.get_untracked(),
                        stat_label: stat_label.get_untracked(),
                        stat_supporting_text: stat_supporting_text.get_untracked(),
                        stat_panel_style: stat_panel_style.get_untracked(),
                    },
                    ComponentFormFeedback {
                        message,
                        error,
                        findings: validation_findings,
                    },
                );
                }
            >
                <label class="form-field">
                    <span>"Name"</span>
                    <input
                        prop:value=move || name.get()
                        on:input=move |event| name.set(event_target_value(&event))
                        on:change=move |event| commit_component_name(name, slug, slug_manually_edited, event_target_value(&event))
                        on:blur=move |event| commit_component_name(name, slug, slug_manually_edited, event_target_value(&event))
                        on:focusout=move |event| commit_component_name(name, slug, slug_manually_edited, event_target_value(&event))
                    />
                </label>
                <label class="form-field">
                    <span>"Slug"</span>
                    <input prop:value=move || slug.get() on:input=move |event| {
                        slug_manually_edited.set(true);
                        slug.set(event_target_value(&event));
                    }/>
                </label>
                <label class="form-field form-field--wide">
                    <span>"Description"</span>
                    <textarea prop:value=move || description.get() on:input=move |event| description.set(event_target_value(&event))></textarea>
                </label>
                <div
                    class="form-field form-field--wide component-dataset-picker"
                    on:click=move |event| event.stop_propagation()
                >
                    <span id="component-dataset-picker-label">"Dataset Version"</span>
                    <button
                        id="component-dataset-picker-trigger"
                        type="button"
                        class="component-dataset-picker__trigger"
                        role="combobox"
                        aria-labelledby="component-dataset-picker-label"
                        aria-controls="component-dataset-picker-options"
                        aria-haspopup="listbox"
                        aria-expanded=move || dataset_picker_open.get().to_string()
                        on:click=move |_| {
                            dataset_picker_active_index.set(0);
                            dataset_picker_open.update(|open| *open = !*open);
                        }
                        on:keydown=move |event| {
                            if event.key() == "ArrowDown" {
                                event.prevent_default();
                                dataset_picker_active_index.set(0);
                                dataset_picker_open.set(true);
                            } else if event.key() == "Escape" {
                                dataset_picker_open.set(false);
                            }
                        }
                    >
                        <span>{move || selected_dataset_picker_label(&datasets.get(), &dataset_id.get(), &dataset_major.get())}</span>
                        <ChevronDown class="component-dataset-picker__chevron"/>
                    </button>
                    {move || dataset_picker_open.get().then(|| {
                        let query = dataset_picker_search.get();
                        let rows = dataset_picker_rows(&datasets.get(), &query);
                        view! {
                            <div class="component-dataset-picker__menu" id="component-dataset-picker-options">
                                <div class="component-dataset-picker__search">
                                    <Search class="component-dataset-picker__search-icon"/>
                                    <input
                                        type="search"
                                        role="searchbox"
                                        aria-label="Filter dataset versions"
                                        aria-controls="component-dataset-picker-listbox"
                                        aria-activedescendant=move || format!("component-dataset-picker-option-{}", dataset_picker_active_index.get())
                                        placeholder="Filter datasets, versions, tags, or provenance"
                                        node_ref=dataset_picker_search_input
                                        prop:value=query
                                        on:input=move |event| {
                                            dataset_picker_active_index.set(0);
                                            dataset_picker_search.set(event_target_value(&event));
                                        }
                                        on:keydown={
                                            move |event| {
                                                let rows = dataset_picker_rows(&datasets.get(), &dataset_picker_search.get());
                                                match event.key().as_str() {
                                                    "ArrowDown" => {
                                                        event.prevent_default();
                                                        let last = rows.len().saturating_sub(1);
                                                        dataset_picker_active_index.update(|index| *index = (*index + 1).min(last));
                                                    }
                                                    "ArrowUp" => {
                                                        event.prevent_default();
                                                        dataset_picker_active_index.update(|index| *index = index.saturating_sub(1));
                                                    }
                                                    "Enter" => {
                                                        event.prevent_default();
                                                        if let Some((dataset, major)) = rows.get(dataset_picker_active_index.get_untracked()) {
                                                            select_dataset_version.run((dataset.id.clone(), *major));
                                                            focus_component_dataset_picker_trigger();
                                                        }
                                                    }
                                                    "Escape" => {
                                                        event.prevent_default();
                                                        dataset_picker_open.set(false);
                                                        focus_component_dataset_picker_trigger();
                                                    }
                                                    _ => {}
                                                }
                                            }
                                        }
                                    />
                                </div>
                                <div class="component-dataset-picker__table-wrap">
                                    <table class="component-dataset-picker__table">
                                        <thead>
                                            <tr>
                                                <th>"Dataset"</th>
                                                <th>"Version"</th>
                                                <th>"Tags"</th>
                                                <th>"Provenance"</th>
                                            </tr>
                                        </thead>
                                        <tbody id="component-dataset-picker-listbox" role="listbox" aria-label="Dataset versions">
                                            {rows.into_iter().enumerate().map(|(index, (dataset, major))| {
                                                let selected_dataset_id = dataset.id.clone();
                                                let selected_dataset_id_for_check = selected_dataset_id.clone();
                                                let selected_dataset_id_for_aria = selected_dataset_id.clone();
                                                let tags = if dataset.tags.is_empty() { "None".into() } else { dataset.tags.join(", ") };
                                                let provenance = dataset_provenance_label(&dataset.provenance);
                                                view! {
                                                    <tr
                                                        id=format!("component-dataset-picker-option-{index}")
                                                        role="option"
                                                        aria-selected=move || {
                                                            (dataset_id.get() == selected_dataset_id_for_aria
                                                                && dataset_major.get() == major.to_string()).to_string()
                                                        }
                                                        class:component-dataset-picker__row--active=move || dataset_picker_active_index.get() == index
                                                        class:component-dataset-picker__row--selected=move || {
                                                        dataset_id.get() == selected_dataset_id_for_check
                                                            && dataset_major.get() == major.to_string()
                                                        }
                                                    >
                                                        <td>
                                                            <button
                                                                type="button"
                                                                class="component-dataset-picker__option"
                                                                on:click=move |_| {
                                                                    select_dataset_version.run((selected_dataset_id.clone(), major));
                                                                    focus_component_dataset_picker_trigger();
                                                                }
                                                            >
                                                                {dataset.name.clone()}
                                                            </button>
                                                        </td>
                                                        <td>{format!("v{major}")}</td>
                                                        <td>{tags}</td>
                                                        <td>{provenance}</td>
                                                    </tr>
                                                }
                                            }).collect_view()}
                                        </tbody>
                                    </table>
                                    {(dataset_picker_rows(&datasets.get(), &dataset_picker_search.get()).is_empty()).then(|| view! {
                                        <p class="component-dataset-picker__empty">"No dataset versions match this filter."</p>
                                    })}
                                </div>
                            </div>
                        }
                    })}
                </div>
                {move || if component_type.get() == "table" {
                    view! {
                        <div class="component-editor__workbench">
                            <div class="component-editor__config-stack" tabindex="-1" data-component-kind-editor>
                                <ComponentEditorFieldset title="Filters" class="component-editor__visual-filters">
                                    <DataOpsFiltersEditor
                                        fields=Signal::derive(move || {
                                            selected_fields
                                                .get()
                                                .into_iter()
                                                .map(|field| component_data_ops_field(&field))
                                                .collect::<Vec<_>>()
                                        })
                                        row_filters=Signal::derive(move || filters.get())
                                        on_row_filters_change=Callback::new(move |row_filters| filters.set(row_filters))
                                        embedded=true
                                    />
                                </ComponentEditorFieldset>
                                <TableDefaultsControls
                                    fields=Signal::derive(move || selected_fields.get())
                                    sort_field
                                    sort_direction
                                    page_size
                                />
                            </div>
                            <div class="component-editor__right-rail">
                                <div class="component-editor__kind-stack">
                                    <ComponentKindControls
                                        component_type
                                        has_kind_specific_changes
                                        on_kind_change=change_component_kind
                                    />
                                </div>
                            </div>
                        </div>
                    }.into_any()
                } else {
                    view! {
                        <div class="component-editor__workbench">
                            <div class="component-editor__config-stack" tabindex="-1" data-component-kind-editor>
                                <ComponentEditorFieldset title="Filters" class="component-editor__visual-filters">
                                    <DataOpsFiltersEditor
                                        fields=Signal::derive(move || {
                                            selected_fields
                                                .get()
                                                .into_iter()
                                                .map(|field| component_data_ops_field(&field))
                                                .collect::<Vec<_>>()
                                        })
                                        row_filters=Signal::derive(move || filters.get())
                                        on_row_filters_change=Callback::new(move |row_filters| filters.set(row_filters))
                                        embedded=true
                                    />
                                </ComponentEditorFieldset>
                                {move || if component_type.get() == "bar" {
                                    view! {
                                        <BarConfigEditor
                                            fields=Signal::derive(move || selected_fields.get())
                                            summary_field=visual_summary_field
                                            summary_type=visual_summary_type
                                            category_field=visual_category_field
                                            category_labels=visual_category_labels
                                            category_colors=visual_category_colors
                                            legend_title=visual_legend_title
                                            comparison_field=visual_comparison_field
                                            orientation=visual_bar_orientation
                                            comparison_layout=visual_bar_comparison_layout
                                            x_axis_label=visual_x_axis_label
                                            y_axis_label=visual_y_axis_label
                                            sort_field=visual_sort_field
                                            sort_direction=visual_sort_direction
                                            limit=visual_limit
                                            value_format=visual_value_format
                                            category_missing_policy=visual_category_missing_policy
                                            comparison_missing_policy=visual_comparison_missing_policy
                                            value_missing_policy=visual_missing_policy
                                        />
                                    }.into_any()
                                } else if component_type.get() == "line" {
                                    view! { <LineConfigEditor fields=Signal::derive(move || selected_fields.get()) summary_field=visual_summary_field summary_type=visual_summary_type x_field=visual_x_field smoothing=visual_line_smoothing sort_field=visual_sort_field sort_direction=visual_sort_direction limit=visual_limit value_format=visual_value_format x_missing_policy=visual_category_missing_policy value_missing_policy=visual_missing_policy/> }.into_any()
                                } else if matches!(component_type.get().as_str(), "pie" | "donut") {
                                    view! { <PieDonutConfigEditor fields=Signal::derive(move || selected_fields.get()) summary_field=visual_summary_field summary_type=visual_summary_type category_field=visual_category_field category_labels=visual_category_labels category_colors=visual_category_colors legend_title=visual_legend_title sort_field=visual_sort_field sort_direction=visual_sort_direction limit=visual_limit value_format=visual_value_format category_missing_policy=visual_category_missing_policy value_missing_policy=visual_missing_policy/> }.into_any()
                                } else {
                                    view! { <StatCardConfigEditor fields=Signal::derive(move || selected_fields.get()) summary_field=visual_summary_field summary_type=visual_summary_type value_format=visual_value_format value_missing_policy=visual_missing_policy stat_label stat_supporting_text stat_panel_style/> }.into_any()
                                }}
                                {move || matches!(component_type.get().as_str(), "bar" | "pie" | "donut").then(|| view! {
                                    <CategoryDisplayControls
                                        dataset_id
                                        dataset_major
                                        component_type
                                        fields=Signal::derive(move || selected_fields.get())
                                        category_field=visual_category_field
                                        comparison_field=visual_comparison_field
                                        category_labels=visual_category_labels
                                        category_colors=visual_category_colors
                                        legend_title=visual_legend_title
                                    />
                                })}
                            </div>
                            <div class="component-editor__right-rail">
                                <div class="component-editor__kind-stack">
                                    <ComponentKindControls
                                        component_type
                                        has_kind_specific_changes
                                        on_kind_change=change_component_kind
                                    />
                                </div>
                                <div class="component-editor__preview-stack">
                                    <div class=move || if preview_drawer_open.get() {
                                        "component-editor__preview-drawer is-open"
                                    } else {
                                        "component-editor__preview-drawer"
                                    }>
                                        <button
                                            class="component-editor__preview-drawer-scrim"
                                            type="button"
                                            aria-label="Close preview"
                                            on:click=move |_| {
                                                preview_drawer_open.set(false);
                                                focus_component_preview_button();
                                            }
                                        ></button>
                                        <div
                                            id="component-editor-preview-drawer"
                                            class="component-editor__preview-drawer-surface"
                                            role=move || preview_drawer_open.get().then_some("dialog")
                                            aria-modal=move || preview_drawer_open.get().then_some("true")
                                            aria-label="Component preview"
                                            tabindex="-1"
                                            on:keydown=move |event| {
                                                if event.key() == "Escape" {
                                                    event.prevent_default();
                                                    preview_drawer_open.set(false);
                                                    focus_component_preview_button();
                                                } else if event.key() == "Tab" && preview_drawer_open.get_untracked() {
                                                    event.prevent_default();
                                                    focus_component_preview_close_button();
                                                }
                                            }
                                        >
                                            <header class="component-editor__preview-drawer-header">
                                                <strong>"Preview"</strong>
                                                <button
                                                    class="icon-button icon-button--compact-control"
                                                    id="component-editor-preview-close"
                                                    type="button"
                                                    aria-label="Close preview"
                                                    title="Close preview"
                                                    on:click=move |_| {
                                                        preview_drawer_open.set(false);
                                                        focus_component_preview_button();
                                                    }
                                                >
                                                    <X class="icon-button__icon"/>
                                                </button>
                                            </header>
                                            <ComponentEditorDraftPreview
                                                visual=draft_preview
                                                error=draft_preview_error
                                                loading=draft_preview_loading
                                                component_type
                                                fields=Signal::derive(move || selected_fields.get())
                                                summary_field=visual_summary_field
                                                summary_type=visual_summary_type
                                                category_field=visual_category_field
                                                comparison_field=visual_comparison_field
                                                sort_direction=visual_sort_direction
                                                limit=visual_limit
                                            />
                                        </div>
                                    </div>
                                </div>
                                <button
                                    class="component-editor__preview-fab"
                                    id="component-editor-preview-fab"
                                    type="button"
                                    aria-label="Open preview"
                                    aria-controls="component-editor-preview-drawer"
                                    aria-expanded=move || preview_drawer_open.get().to_string()
                                    title="Preview"
                                    on:click=move |_| {
                                        preview_drawer_open.set(true);
                                        focus_component_preview_drawer();
                                    }
                                >
                                    <PanelRight class="component-editor__preview-fab-icon"/>
                                </button>
                            </div>
                        </div>
                    }.into_any()
                }}
                {move || (component_type.get() == "table").then(|| view! {
                    <ComponentEditorFieldset title="Displayed Fields" class="component-editor__projection-panel">
                        <DataOpsProjectionEditor
                            available_fields=Signal::derive(move || {
                                selected_fields
                                    .get()
                                    .into_iter()
                                    .map(|field| component_data_ops_field(&field))
                                    .collect::<Vec<_>>()
                            })
                            fields=Signal::derive(move || columns.get())
                            active_source_tab=Signal::derive(move || projection_active_source_tab.get())
                            on_active_source_tab_change=Callback::new(move |tab| projection_active_source_tab.set(tab))
                            on_fields_change=Callback::new(move |fields| columns.set(fields))
                        />
                    </ComponentEditorFieldset>
                })}
                <div class="form-actions">
                    <button
                        class="button button--secondary button--warning"
                        type="button"
                        on:click=move |_| cancel_component_edit()
                    >
                        "Cancel"
                    </button>
                    {move || editing_version_id.get().map(|version_id| {
                        let component_id = editing_component_id.get_untracked().unwrap_or_default();
                        view! {
                            <button class="button button--danger" type="button" on:click=move |_| {
                                delete_component_draft(
                                    component_id.clone(),
                                    version_id.clone(),
                                    message,
                                    error,
                                    validation_findings,
                                );
                            }>"Discard Draft"</button>
                        }
                    })}
                    <button class="button button--secondary" type="submit">"Save Draft"</button>
                    <div class=move || if publish_menu_open.get() {
                        "dropdown-menu component-editor__publish-menu is-open"
                    } else {
                        "dropdown-menu component-editor__publish-menu"
                    }>
                        <button
                            class="button component-editor__publish-button"
                            type="button"
                            aria-expanded=move || publish_menu_open.get().to_string()
                            on:click=move |_| publish_menu_open.update(|open| *open = !*open)
                        >
                            "Save and Publish"
                            <ChevronDown class="button__icon"/>
                        </button>
                        <button
                            class="dropdown-menu__scrim"
                            type="button"
                            aria-label="Close publish options"
                            on:click=move |_| publish_menu_open.set(false)
                        ></button>
                        <div class="dropdown-menu__content component-editor__publish-options" role="menu">
                            <button
                                class="dropdown-menu__item"
                                type="button"
                                role="menuitem"
                                disabled=move || current_published_version_id.get().is_none()
                                on:click=move |_| {
                                publish_menu_open.set(false);
                                create_component_from_form(
                                    ComponentSaveIntent {
                                        editing_component_id: editing_component_id.get_untracked(),
                                        editing_version_id: editing_version_id.get_untracked(),
                                        current_published_version_id: current_published_version_id.get_untracked(),
                                        publish_action: ComponentPublishAction::UpdateExistingVersion,
                                        version_note: None,
                                    },
                                    ComponentFormValues {
                                        name: name.get_untracked(),
                                        slug: slug.get_untracked(),
                                        description: description.get_untracked(),
                                        dataset_id: dataset_id.get_untracked(),
                                        dataset_major: dataset_major.get_untracked(),
                                        columns: columns.get_untracked(),
                                        filters: filters.get_untracked(),
                                        sort_field: sort_field.get_untracked(),
                                        sort_direction: sort_direction.get_untracked(),
                                        page_size: page_size.get_untracked(),
                                        component_type: component_type.get_untracked(),
                                        visual_summary_field: visual_summary_field.get_untracked(),
                                        visual_summary_type: visual_summary_type.get_untracked(),
                                        visual_category_field: visual_category_field.get_untracked(),
                                        visual_category_labels: visual_category_labels.get_untracked(),
                                        visual_category_colors: visual_category_colors.get_untracked(),
                                        visual_legend_title: visual_legend_title.get_untracked(),
                                        visual_comparison_field: visual_comparison_field.get_untracked(),
                                        visual_bar_orientation: visual_bar_orientation.get_untracked(),
                                        visual_bar_comparison_layout: visual_bar_comparison_layout.get_untracked(),
                                        visual_x_axis_label: visual_x_axis_label.get_untracked(),
                                        visual_y_axis_label: visual_y_axis_label.get_untracked(),
                                        visual_x_field: visual_x_field.get_untracked(),
                                        visual_line_smoothing: visual_line_smoothing.get_untracked(),
                                        visual_sort_field: visual_sort_field.get_untracked(),
                                        visual_sort_direction: visual_sort_direction.get_untracked(),
                                        visual_limit: visual_limit.get_untracked(),
                                        visual_value_format: visual_value_format.get_untracked(),
                                        visual_category_missing_policy: visual_category_missing_policy.get_untracked(),
                                        visual_comparison_missing_policy: visual_comparison_missing_policy.get_untracked(),
                                        visual_missing_policy: visual_missing_policy.get_untracked(),
                                        stat_label: stat_label.get_untracked(),
                                        stat_supporting_text: stat_supporting_text.get_untracked(),
                                        stat_panel_style: stat_panel_style.get_untracked(),
                                    },
                                    ComponentFormFeedback {
                                        message,
                                        error,
                                        findings: validation_findings,
                                    },
                                );
                            }>
                                "Update Existing Version"
                            </button>
                            <button class="dropdown-menu__item" type="button" role="menuitem" on:click=move |_| {
                                publish_menu_open.set(false);
                                consumer_search.set(String::new());
                                new_version_note.set(String::new());
                                consumer_modal_open.set(true);
                            }>
                                "Create New Version"
                            </button>
                        </div>
                    </div>
                </div>
            </form>
            {move || consumer_modal_open.get().then(|| {
                view! {
                    <section class="component-consumers-modal" aria-label="Review component consumers">
                        <button
                            class="component-consumers-modal__scrim"
                            type="button"
                            aria-label="Close consumer review"
                            on:click=move |_| consumer_modal_open.set(false)
                        ></button>
                        <aside class="component-consumers-modal__panel blurred-surface" role="dialog" aria-modal="true" aria-label="Review component consumers">
                            <header class="component-consumers-modal__header">
                                <div>
                                    <p class="eyebrow">"Create New Version"</p>
                                    <h2>"Review Consumers"</h2>
                                </div>
                                <button
                                    class="icon-button"
                                    type="button"
                                    aria-label="Close consumer review"
                                    on:click=move |_| consumer_modal_open.set(false)
                                >
                                    <X class="icon-button__icon"/>
                                </button>
                            </header>
                            <p class="component-consumers-modal__intro">
                                "This step prepares the consumer review workflow for the new version. Consumer re-pinning will use this list when dashboard and report consumers are available."
                            </p>
                            <label class="form-field component-consumers-modal__search">
                                <span>"Search consumers"</span>
                                <input
                                    type="search"
                                    placeholder="Search by dashboard, report, or placement"
                                    prop:value=move || consumer_search.get()
                                    on:input=move |event| consumer_search.set(event_target_value(&event))
                                />
                            </label>
                            <div class="component-consumers-modal__list" role="list">
                                <EmptyState
                                    title="Consumer review placeholder"
                                    message="Consumer discovery is not wired in Sprint 4A yet. Creating a new version will not automatically repin dashboards or reports."
                                />
                            </div>
                            <label class="form-field component-consumers-modal__note">
                                <span>"New Version Note"</span>
                                <textarea
                                    placeholder="Summarize what changed in this version"
                                    prop:value=move || new_version_note.get()
                                    on:input=move |event| new_version_note.set(event_target_value(&event))
                                ></textarea>
                            </label>
                            <footer class="component-consumers-modal__footer">
                                <button
                                    class="button button--secondary"
                                    type="button"
                                    on:click=move |_| consumer_modal_open.set(false)
                                >
                                    "Cancel"
                                </button>
                                <button class="button" type="button" on:click=move |_| {
                                    if new_version_note.get_untracked().trim().is_empty() {
                                        error.set(Some("New versions require a version note.".into()));
                                        return;
                                    }
                                    consumer_modal_open.set(false);
                                    create_component_from_form(
                                        ComponentSaveIntent {
                                            editing_component_id: editing_component_id.get_untracked(),
                                            editing_version_id: editing_version_id.get_untracked(),
                                            current_published_version_id: current_published_version_id.get_untracked(),
                                            publish_action: ComponentPublishAction::CreateNewVersion,
                                            version_note: Some(new_version_note.get_untracked()),
                                        },
                                        ComponentFormValues {
                                            name: name.get_untracked(),
                                            slug: slug.get_untracked(),
                                            description: description.get_untracked(),
                                            dataset_id: dataset_id.get_untracked(),
                                            dataset_major: dataset_major.get_untracked(),
                                            columns: columns.get_untracked(),
                                            filters: filters.get_untracked(),
                                            sort_field: sort_field.get_untracked(),
                                            sort_direction: sort_direction.get_untracked(),
                                            page_size: page_size.get_untracked(),
                                            component_type: component_type.get_untracked(),
                                            visual_summary_field: visual_summary_field.get_untracked(),
                                            visual_summary_type: visual_summary_type.get_untracked(),
                                            visual_category_field: visual_category_field.get_untracked(),
                                            visual_category_labels: visual_category_labels.get_untracked(),
                                            visual_category_colors: visual_category_colors.get_untracked(),
                                            visual_legend_title: visual_legend_title.get_untracked(),
                                            visual_comparison_field: visual_comparison_field.get_untracked(),
                                            visual_bar_orientation: visual_bar_orientation.get_untracked(),
                                            visual_bar_comparison_layout: visual_bar_comparison_layout.get_untracked(),
                                            visual_x_axis_label: visual_x_axis_label.get_untracked(),
                                            visual_y_axis_label: visual_y_axis_label.get_untracked(),
                                            visual_x_field: visual_x_field.get_untracked(),
                                            visual_line_smoothing: visual_line_smoothing.get_untracked(),
                                            visual_sort_field: visual_sort_field.get_untracked(),
                                            visual_sort_direction: visual_sort_direction.get_untracked(),
                                            visual_limit: visual_limit.get_untracked(),
                                            visual_value_format: visual_value_format.get_untracked(),
                                            visual_category_missing_policy: visual_category_missing_policy.get_untracked(),
                                            visual_comparison_missing_policy: visual_comparison_missing_policy.get_untracked(),
                                            visual_missing_policy: visual_missing_policy.get_untracked(),
                                            stat_label: stat_label.get_untracked(),
                                            stat_supporting_text: stat_supporting_text.get_untracked(),
                                            stat_panel_style: stat_panel_style.get_untracked(),
                                        },
                                        ComponentFormFeedback {
                                            message,
                                            error,
                                            findings: validation_findings,
                                        },
                                    );
                                }>
                                    "Create New Version"
                                </button>
                            </footer>
                        </aside>
                    </section>
                }
            })}
            {move || dataset_error.get().map(|message| view! { <p class="form-status is-error">{message}</p> })}
            {move || message.get().map(|message| view! { <p class="form-status is-success">{message}</p> })}
            {move || error.get().map(|message| view! { <p class="form-status is-error">{message}</p> })}
            {move || {
                let findings = validation_findings.get();
                (!findings.is_empty()).then(|| view! { <ValidationFindingsPanel findings/> })
            }}
        </section>
    }
}

#[component]
pub fn ComponentViewerContent(component_ref: String) -> impl IntoView {
    let component = RwSignal::new(None::<ComponentDefinition>);
    let component_loading = RwSignal::new(true);
    let component_error = RwSignal::new(None::<String>);
    let can_manage_component = RwSignal::new(false);
    let table = RwSignal::new(None::<ComponentTable>);
    let visual = RwSignal::new(None::<ComponentVisual>);
    let error = RwSignal::new(None::<String>);
    let component_ref_for_title = component_ref.clone();

    Effect::new({
        let component_ref = component_ref.clone();
        move |_| {
            load_component(
                component_ref.clone(),
                component,
                component_loading,
                component_error,
                can_manage_component,
            )
        }
    });
    Effect::new({
        let component_ref = component_ref.clone();
        move |_| {
            if let Some(component_type) = published_component_type(component.get()).as_deref() {
                if component_type == "table" {
                    load_component_table(component_ref.clone(), String::new(), table, error);
                } else {
                    load_component_visual(
                        component_ref.clone(),
                        component_type.to_string(),
                        visual,
                        error,
                    );
                }
            }
        }
    });

    view! {
        <section class="route-panel components-page">
            <ComponentViewerBreadcrumb component_ref=component_ref.clone() component=component/>
            <header class="page-header">
                <div>
                    <h1>{move || {
                        component
                            .get()
                            .map(|component| component.name)
                            .unwrap_or_else(|| component_ref_for_title.clone())
                    }}</h1>
                </div>
                {move || can_manage_component.get().then(|| {
                    let edit_href = format!("/components/{}/edit", component_ref.clone());
                    let versions_href = format!("/components/{}/versions", component_ref.clone());
                    view! {
                        <div class="page-header__actions">
                            <a class="button button--secondary" href=versions_href>"Versions"</a>
                            <a class="button button--secondary" href=edit_href>"Edit"</a>
                        </div>
                    }
                })}
            </header>
            {move || {
                if component_loading.get() {
                    view! { <EmptyState title="Loading configuration" message="Fetching component configuration."/> }.into_any()
                } else if let Some(message) = component_error.get() {
                    view! { <EmptyState title="Configuration unavailable" message=message/> }.into_any()
                } else if let Some(component) = component.get() {
                    if let Some(component_type) = published_component_type(Some(component.clone())) {
                        if component_type == "table" {
                            view! { <ComponentTablePreviewSection table=table.get() table_error=error.get()/> }.into_any()
                        } else {
                            view! { <ComponentVisualPreviewSection visual=visual.get() visual_error=error.get()/> }.into_any()
                        }
                    } else {
                        view! { <EmptyState title="No published version" message="This component does not have a published version yet."/> }.into_any()
                    }
                } else {
                    view! { <EmptyState title="Component unavailable" message="Component data could not be loaded."/> }.into_any()
                }
            }}
        </section>
    }
}

#[component]
fn ValidationFindingsPanel(findings: Vec<ComponentValidationFinding>) -> impl IntoView {
    view! {
        <section class="route-panel__section validation-findings" aria-label="Validation Findings" aria-live="polite">
            <h2>"Validation Findings"</h2>
            <ul>
                {findings.into_iter().map(|finding| {
                    let field_path = finding.field_path.unwrap_or_else(|| "component".into());
                    let field_path_attr = field_path.clone();
                    let severity = finding.severity;
                    let code = finding.code;
                    let message = finding.message;
                    view! {
                        <li class="validation-finding" data-field-path=field_path_attr>
                            <strong>{field_path}</strong>
                            <span>{format!("{severity}: {code}")}</span>
                            <p>{message}</p>
                        </li>
                    }
                }).collect_view()}
            </ul>
        </section>
    }
}

#[component]
fn ComponentsBreadcrumb(#[prop(into)] current: String) -> impl IntoView {
    view! {
        <Breadcrumb>
            <BreadcrumbItem>
                <BreadcrumbLink href="/components">"Components"</BreadcrumbLink>
            </BreadcrumbItem>
            <BreadcrumbSeparator/>
            <BreadcrumbItem>
                <BreadcrumbPage>{current}</BreadcrumbPage>
            </BreadcrumbItem>
        </Breadcrumb>
    }
}

#[component]
fn ComponentViewerBreadcrumb(
    #[prop(into)] component_ref: String,
    component: RwSignal<Option<ComponentDefinition>>,
) -> impl IntoView {
    view! {
        <Breadcrumb>
            <BreadcrumbItem>
                <BreadcrumbLink href="/components">"Components"</BreadcrumbLink>
            </BreadcrumbItem>
            <BreadcrumbSeparator/>
            <BreadcrumbItem>
                <BreadcrumbPage>
                    {move || {
                        component
                            .get()
                            .map(|component| component.name)
                            .unwrap_or_else(|| component_ref.clone())
                    }}
                </BreadcrumbPage>
            </BreadcrumbItem>
        </Breadcrumb>
    }
}

#[component]
fn ComponentNestedBreadcrumb(
    #[prop(into)] component_href: String,
    #[prop(into)] component_label: String,
    #[prop(into)] current: String,
) -> impl IntoView {
    view! {
        <Breadcrumb>
            <BreadcrumbItem>
                <BreadcrumbLink href="/components">"Components"</BreadcrumbLink>
            </BreadcrumbItem>
            <BreadcrumbSeparator/>
            <BreadcrumbItem>
                <BreadcrumbLink href=component_href>{component_label}</BreadcrumbLink>
            </BreadcrumbItem>
            <BreadcrumbSeparator/>
            <BreadcrumbItem>
                <BreadcrumbPage>{current}</BreadcrumbPage>
            </BreadcrumbItem>
        </Breadcrumb>
    }
}

#[component]
fn ComponentTablePreviewSection(
    table: Option<ComponentTable>,
    table_error: Option<String>,
) -> impl IntoView {
    view! {
        <section class="route-panel__section component-table-preview">
            {if let Some(message) = table_error {
                view! { <EmptyState title="Preview unavailable" message=message/> }.into_any()
            } else if let Some(table) = table {
                view! { <ComponentTablePreview table/> }.into_any()
            } else {
                view! { <EmptyState title="Loading preview" message="Fetching the published table preview."/> }.into_any()
            }}
        </section>
    }
}

#[component]
fn ComponentTablePreview(table: ComponentTable) -> impl IntoView {
    let columns = table.columns.clone();
    let rows = table.rows.clone();
    let column_count = columns.len();
    let row_count = rows.len();
    let table_columns = columns
        .iter()
        .map(|column| {
            InteractiveTableColumn::new(
                column.key.clone(),
                column.label.clone(),
                sentence_label(&column.field_type),
            )
        })
        .collect::<Vec<_>>();
    let table_rows = rows
        .into_iter()
        .map(|row| {
            let values = columns
                .iter()
                .map(|column| {
                    let value = row
                        .values
                        .get(&column.key)
                        .cloned()
                        .flatten()
                        .unwrap_or_default();
                    (column.key.clone(), value)
                })
                .collect::<BTreeMap<_, _>>();
            InteractiveTableRow::new(row.row_id, values)
        })
        .collect::<Vec<_>>();

    view! {
        <div class="component-table-preview__header">
            <div>
                <h2>"Preview"</h2>
                <p>{format!("Showing {row_count} rows across {column_count} visible columns.")}</p>
            </div>
        </div>
        {if columns.is_empty() {
            view! { <EmptyState title="No visible columns" message="This component does not currently expose any table columns."/> }.into_any()
        } else if row_count == 0 {
            view! { <EmptyState title="No rows to display" message="The published table returned no rows for its current configuration."/> }.into_any()
        } else {
            view! {
                <InteractiveDataTable
                    columns=table_columns
                    rows=table_rows
                    search_label="Search component rows"
                    search_placeholder="Search table"
                    item_label="table rows"
                    empty_message="No table rows match the current controls."
                />
            }.into_any()
        }}
    }
}

#[component]
fn ComponentVisualPreviewSection(
    visual: Option<ComponentVisual>,
    visual_error: Option<String>,
) -> impl IntoView {
    view! {
        <section class="route-panel__section component-visual-preview">
            {if let Some(message) = visual_error {
                view! { <EmptyState title="Preview unavailable" message=message/> }.into_any()
            } else if let Some(visual) = visual {
                view! { <ComponentVisualPreview visual/> }.into_any()
            } else {
                view! { <EmptyState title="Loading preview" message="Fetching the published visual preview."/> }.into_any()
            }}
        </section>
    }
}

#[component]
fn ComponentEditorDraftPreview(
    visual: RwSignal<Option<ComponentVisual>>,
    error: RwSignal<Option<String>>,
    loading: RwSignal<bool>,
    component_type: RwSignal<String>,
    fields: Signal<Vec<DatasetFieldDefinition>>,
    summary_field: RwSignal<String>,
    summary_type: RwSignal<String>,
    category_field: RwSignal<String>,
    comparison_field: RwSignal<String>,
    sort_direction: RwSignal<String>,
    limit: RwSignal<String>,
) -> impl IntoView {
    view! {
        <aside class="route-panel__section component-editor-preview">
            <header class="component-editor-preview__header">
                <div>
                    <h2>"Preview"</h2>
                    <p>"Uses up to 100 rows from the current draft"</p>
                </div>
                <span class=move || if error.get().is_some() {
                    "component-editor-preview__badge is-error"
                } else if loading.get() {
                    "component-editor-preview__badge is-loading"
                } else {
                    "component-editor-preview__badge"
                }>
                    {move || if error.get().is_some() {
                        "Needs attention"
                    } else if loading.get() {
                        "Updating"
                    } else if visual.get().is_some() {
                        "Valid config"
                    } else {
                        "Incomplete"
                    }}
                </span>
            </header>
            <div class="component-editor-preview__body" aria-live="polite">
                {move || if let Some(message) = error.get() {
                    view! { <EmptyState title="Preview unavailable" message/> }.into_any()
                } else if let Some(visual) = visual.get() {
                    view! { <ComponentVisualPreview visual/> }.into_any()
                } else {
                    view! {
                        <EmptyState
                            title="Complete the data mapping"
                            message="Choose the required Dataset fields to render this draft."
                        />
                    }.into_any()
                }}
            </div>
            <p class="component-editor-preview__note">
                {move || component_editor_execution_summary(
                    &component_type.get(),
                    &fields.get(),
                    &summary_field.get(),
                    &summary_type.get(),
                    &category_field.get(),
                    &comparison_field.get(),
                    &sort_direction.get(),
                    &limit.get(),
                )}
            </p>
        </aside>
    }
}

#[allow(clippy::too_many_arguments)]
fn component_editor_execution_summary(
    component_type: &str,
    fields: &[DatasetFieldDefinition],
    summary_field: &str,
    summary_type: &str,
    category_field: &str,
    comparison_field: &str,
    sort_direction: &str,
    limit: &str,
) -> String {
    let summary_label = field_label_for_key(fields, summary_field)
        .unwrap_or_else(|| "the selected value field".into());
    let calculation = match summary_type {
        "unique_count" => "counting unique values of",
        "sum" => "summing",
        "average" => "averaging",
        "median" => "taking the median of",
        _ => "counting non-empty values of",
    };
    if component_type == "stat_card" {
        return format!("Calculates one value by {calculation} {summary_label}.");
    }
    let category_label = field_label_for_key(fields, category_field)
        .unwrap_or_else(|| "the selected category field".into());
    let comparison = field_label_for_key(fields, comparison_field)
        .map(|label| format!(", split by {label}"))
        .unwrap_or_default();
    let direction = if sort_direction == "desc" {
        "descending"
    } else {
        "ascending"
    };
    format!(
        "Shows up to {} {category_label} groups{comparison}, {calculation} {summary_label}, ordered {direction}.",
        limit.trim().parse::<usize>().unwrap_or(20)
    )
}

#[component]
fn ComponentVisualPreview(visual: ComponentVisual) -> impl IntoView {
    match visual.component_type.as_str() {
        "stat_card" => view! { <ComponentStatCardPreview visual/> }.into_any(),
        _ => view! { <ComponentD3ChartPreview visual/> }.into_any(),
    }
}

#[component]
fn ComponentStatCardPreview(visual: ComponentVisual) -> impl IntoView {
    let class_name = format!(
        "component-stat-card component-stat-card--{}",
        visual
            .stat
            .as_ref()
            .map(|stat| stat.panel_style.as_str())
            .unwrap_or("default")
    );
    view! {
        <div class=class_name>
            {if let Some(stat) = visual.stat {
                view! {
                    <p>{stat.label}</p>
                    <strong>{stat.display_value.unwrap_or_else(|| "-".into())}</strong>
                    {stat.supporting_text.map(|text| view! { <span>{text}</span> })}
                }.into_any()
            } else {
                view! { <EmptyState title="No stat value" message="The published StatCard returned no value."/> }.into_any()
            }}
        </div>
    }
}

#[component]
fn ComponentD3ChartPreview(visual: ComponentVisual) -> impl IntoView {
    let kind = visual.component_type.clone();
    let item_count = if matches!(kind.as_str(), "pie" | "donut") {
        visual.slices.len()
    } else {
        visual.points.len()
    };
    let is_empty = item_count == 0;
    let payload = serde_json::to_string(&visual).unwrap_or_else(|_| "{}".into());
    let aria_label = format!("{} chart preview", component_type_label(&kind));
    view! {
        <div class="component-chart component-d3-chart" data-chart=payload>
            {if is_empty {
                view! { <EmptyState title="No visual data" message="The published visual returned no grouped values."/> }.into_any()
            } else {
                view! {
                    <div class="component-d3-chart__surface" role="img" aria-label=aria_label>
                        <div class="component-d3-chart__loading" aria-label="Loading chart preview">
                            <Skeleton class="skeleton--text skeleton--short"/>
                            <Skeleton class="skeleton--chart"/>
                        </div>
                    </div>
                }.into_any()
            }}
        </div>
    }
}

#[component]
fn ComponentVersionsSection(versions: Vec<ComponentVersionSummary>) -> impl IntoView {
    view! {
        <section class="route-panel__section">
            <h2>"Versions"</h2>
            <DataTable>
                <thead>
                    <tr>
                        <th>"Version"</th>
                        <th>"Status"</th>
                        <th>"Kind"</th>
                        <th>"Dataset Version"</th>
                        <th>"Note"</th>
                    </tr>
                </thead>
                <tbody>
                    {versions.into_iter().map(|version| {
                        let version_note = if version.version_note.trim().is_empty() {
                            "-".to_string()
                        } else {
                            version.version_note.clone()
                        };
                        view! {
                            <tr>
                                <td>{version.version_label}</td>
                                <td>{component_status_label(&version.status)}</td>
                                <td>{component_type_label(&version.component_type)}</td>
                                <td>{format!("v{}", version.dataset_version_major)}</td>
                                <td>{version_note}</td>
                            </tr>
                        }
                    }).collect_view()}
                </tbody>
            </DataTable>
        </section>
    }
}

#[component]
fn ComponentsTable(components: Vec<ComponentSummary>, can_manage: bool) -> impl IntoView {
    let search = RwSignal::new(String::new());
    let kind_filter = RwSignal::new(String::from("all"));
    let status_filter = RwSignal::new(String::from("all"));
    let mobile_filter_open = RwSignal::new(false);
    let page_size = RwSignal::new(10usize);
    let page_index = RwSignal::new(0usize);
    let kind_options = component_kind_filter_options(&components);
    let status_options = component_status_filter_options(&components);
    let table_kind_options = kind_options.clone();
    let table_status_options = status_options.clone();
    let mobile_kind_options = kind_options.clone();
    let mobile_status_options = status_options.clone();
    let filtered_components = Memo::new(move |_| {
        let query = search.get();
        let kind = kind_filter.get();
        let status = status_filter.get();
        components
            .iter()
            .filter(|component| component_matches_filters(component, &query, &kind, &status))
            .cloned()
            .collect::<Vec<_>>()
    });
    let total_count = Memo::new(move |_| filtered_components.get().len());
    let mobile_components = Memo::new(move |_| filtered_components.get());

    Effect::new(move |_| {
        search.get();
        kind_filter.get();
        status_filter.get();
        page_index.set(0);
    });

    view! {
        <section class="route-panel__section">
            <div class="forms-list-responsive-table components-list-responsive-table">
                <div class="searchable-data-table">
                    <div class="searchable-data-table__toolbar components-list__toolbar">
                        <label class="searchable-data-table__search searchable-data-table__control">
                            <Search class="searchable-data-table__control-icon"/>
                            <span class="sr-only">"Search components by name"</span>
                            <input
                                type="search"
                                placeholder="Search components"
                                prop:value=move || search.get()
                                on:input=move |event| search.set(event_target_value(&event))
                            />
                        </label>
                        <button
                            class="icon-button components-list__mobile-filter-button"
                            type="button"
                            aria-label=move || component_mobile_filter_button_label(
                                &kind_filter.get(),
                                &status_filter.get(),
                            )
                            title=move || component_mobile_filter_button_label(
                                &kind_filter.get(),
                                &status_filter.get(),
                            )
                            on:click=move |_| mobile_filter_open.set(true)
                        >
                            <ListFilter/>
                        </button>
                    </div>
                    <DataTable>
                        <thead>
                            <tr>
                                <th scope="col">"Name"</th>
                                <th class="data-table__cell--center" scope="col">
                                    <TableFilterHeader
                                        label="Kind"
                                        all_label="All kinds"
                                        filter=kind_filter
                                        options=table_kind_options.clone()
                                    />
                                </th>
                                <th class="data-table__cell--center" scope="col">"Revision"</th>
                                <th class="data-table__cell--center" scope="col">
                                    <TableFilterHeader
                                        label="Status"
                                        all_label="All statuses"
                                        filter=status_filter
                                        options=table_status_options.clone()
                                    />
                                </th>
                                {can_manage.then(|| view! {
                                    <th class="data-table__cell--center" scope="col">"Actions"</th>
                                })}
                            </tr>
                        </thead>
                        <tbody>
                            {move || {
                                let components = filtered_components.get();
                                if components.is_empty() {
                                    let colspan = if can_manage { 5 } else { 4 };
                                    view! {
                                        <tr>
                                            <td class="data-table__empty" colspan=colspan>"No Components to Display"</td>
                                        </tr>
                                    }
                                    .into_any()
                                } else {
                                    components
                                        .into_iter()
                                        .skip(component_pagination_page_start(total_count.get(), page_size.get(), page_index.get()))
                                        .take(page_size.get())
                                        .map(|component| {
                                            let href = format!("/components/{}", component.slug);
                                            let edit_href = format!("/components/{}/edit", component.slug);
                                            let versions_href = format!("/components/{}/versions", component.slug);
                                            let kind_label = component_summary_kind_label(&component);
                                            let status_label = component_summary_status_label(&component);
                                            let revision_label = component_summary_revision_label(&component);
                                            view! {
                                                <tr>
                                                    <th scope="row">
                                                        <a class="data-table__primary-link" href=href>{component.name}</a>
                                                    </th>
                                                    <td class="data-table__cell--center">{kind_label}</td>
                                                    <td class="data-table__cell--center">{revision_label}</td>
                                                    <td class="data-table__cell--center">{status_label}</td>
                                                    {can_manage.then(|| view! {
                                                        <td class="data-table__cell--center">
                                                            <div class="components-list-actions">
                                                                <a class="icon-button" href=edit_href aria-label="Edit component" title="Edit component">
                                                                    <Pencil class="icon-button__icon"/>
                                                                </a>
                                                                <a class="icon-button" href=versions_href aria-label="View component versions" title="View component versions">
                                                                    <History class="icon-button__icon"/>
                                                                </a>
                                                            </div>
                                                        </td>
                                                    })}
                                                </tr>
                                            }
                                        })
                                        .collect_view()
                                        .into_any()
                                }
                            }}
                        </tbody>
                    </DataTable>
                </div>
                <ComponentsMobileFilterSheet
                    is_open=mobile_filter_open
                    kind_filter=kind_filter
                    status_filter=status_filter
                    kind_options=mobile_kind_options
                    status_options=mobile_status_options
                />
                <ComponentsMobileCards
                    components=mobile_components
                    total_count=total_count
                    page_size=page_size
                    page_index=page_index
                    can_manage=can_manage
                />
                <TablePaginationFooter
                    aria_label="Components table pagination"
                    item_label="components"
                    total_count=total_count
                    page_size=page_size
                    page_index=page_index
                />
            </div>
        </section>
    }
}

#[component]
fn ComponentsMobileCards(
    components: Memo<Vec<ComponentSummary>>,
    total_count: Memo<usize>,
    page_size: RwSignal<usize>,
    page_index: RwSignal<usize>,
    can_manage: bool,
) -> impl IntoView {
    view! {
        <div class="forms-list-mobile-cards components-list-mobile-cards">
            {move || {
                let components = components.get();
                if components.is_empty() {
                    view! { <p class="forms-list-mobile-empty">"No Components to Display"</p> }
                        .into_any()
                } else {
                    components
                        .into_iter()
                        .skip(component_pagination_page_start(total_count.get(), page_size.get(), page_index.get()))
                        .take(page_size.get())
                        .map(|component| {
                            let href = format!("/components/{}", component.slug);
                            let edit_href = format!("/components/{}/edit", component.slug);
                            let versions_href = format!("/components/{}/versions", component.slug);
                            let kind_label = component_summary_kind_label(&component);
                            let status_label = component_summary_status_label(&component);
                            let revision_label = component_summary_revision_label(&component);
                            view! {
                                <article class="forms-list-mobile-card components-list-mobile-card">
                                    <div class="forms-list-mobile-card__header">
                                        <h3><a href=href>{component.name}</a></h3>
                                    </div>
                                    <dl>
                                        <div>
                                            <dt>"Kind"</dt>
                                            <dd>{kind_label}</dd>
                                        </div>
                                        <div>
                                            <dt>"Status"</dt>
                                            <dd>{status_label}</dd>
                                        </div>
                                        <div>
                                            <dt>"Revision"</dt>
                                            <dd>{revision_label}</dd>
                                        </div>
                                    </dl>
                                    {can_manage.then(|| view! {
                                        <div class="forms-list-mobile-card__actions components-list-mobile-card__actions">
                                            <a class="icon-button" href=edit_href aria-label="Edit component" title="Edit component">
                                                <Pencil class="icon-button__icon"/>
                                            </a>
                                            <a class="icon-button" href=versions_href aria-label="View component versions" title="View component versions">
                                                <History class="icon-button__icon"/>
                                            </a>
                                        </div>
                                    })}
                                </article>
                            }
                        })
                        .collect_view()
                        .into_any()
                }
            }}
        </div>
    }
}

#[component]
fn ComponentsMobileFilterSheet(
    is_open: RwSignal<bool>,
    kind_filter: RwSignal<String>,
    status_filter: RwSignal<String>,
    kind_options: Vec<String>,
    status_options: Vec<String>,
) -> impl IntoView {
    view! {
        <Show when=move || is_open.get()>
            <section class="sheet-overlay components-filter-overlay" aria-label="Component filters">
                <button
                    class="sheet-overlay__scrim"
                    type="button"
                    aria-label="Close component filters"
                    on:click=move |_| is_open.set(false)
                ></button>
                <aside class="sheet-panel blurred-surface components-filter-sheet" role="dialog" aria-modal="true" aria-label="Component filters">
                    <div class="sheet-panel__actions">
                        <button
                            class="icon-button sheet-panel__close"
                            type="button"
                            aria-label="Close component filters"
                            title="Close component filters"
                            on:click=move |_| is_open.set(false)
                        >
                            <X/>
                        </button>
                    </div>
                    <header class="sheet-panel__header">
                        <span>"Filters"</span>
                        <h2>"Components"</h2>
                    </header>
                    <section class="sheet-panel__section components-filter-sheet__controls">
                        <label class="form-field">
                            <span>"Kind"</span>
                            <select
                                aria-label="Filter components by kind"
                                prop:value=move || kind_filter.get()
                                on:change=move |event| kind_filter.set(event_target_value(&event))
                            >
                                <option value="all">"All kinds"</option>
                                {kind_options.clone().into_iter().map(|option| {
                                    view! { <option value=option.clone()>{option.clone()}</option> }
                                }).collect_view()}
                            </select>
                        </label>
                        <label class="form-field">
                            <span>"Status"</span>
                            <select
                                aria-label="Filter components by status"
                                prop:value=move || status_filter.get()
                                on:change=move |event| status_filter.set(event_target_value(&event))
                            >
                                <option value="all">"All statuses"</option>
                                {status_options.clone().into_iter().map(|option| {
                                    view! { <option value=option.clone()>{option.clone()}</option> }
                                }).collect_view()}
                            </select>
                        </label>
                    </section>
                    <div class="sheet-panel__footer components-filter-sheet__footer">
                        <button
                            class="button button--secondary"
                            type="button"
                            on:click=move |_| {
                                kind_filter.set("all".to_string());
                                status_filter.set("all".to_string());
                            }
                        >
                            "Clear All"
                        </button>
                    </div>
                </aside>
            </section>
        </Show>
    }
}

#[cfg_attr(not(test), allow(dead_code))]
fn materialization_empty_state(state: &str) -> (&'static str, String) {
    match state {
        "failed" | "error" => (
            "Table materialization failed",
            "The component configuration is valid, but the bound Dataset major-line table could not be materialized. Retry after the Dataset materialization is rebuilt.".into(),
        ),
        "pending" => (
            "Table materializing",
            "The component configuration is valid, but the bound Dataset major-line table is still being prepared.".into(),
        ),
        other => (
            "Table materializing",
            format!("The component configuration is valid, but the bound Dataset major-line table is not ready yet. Materialization state: {other}"),
        ),
    }
}

fn component_type_label(component_type: &str) -> &'static str {
    match component_type {
        "table" => "Table",
        "bar" => "Bar",
        "line" => "Line",
        "pie" => "Pie",
        "donut" => "Donut",
        "stat_card" => "Stat Card",
        _ => "Component",
    }
}

fn component_status_label(status: &str) -> &'static str {
    match status {
        "draft" => "Draft",
        "published" => "Published",
        "superseded" => "Superseded",
        _ => "Unknown",
    }
}

fn component_summary_kind_label(component: &ComponentSummary) -> &'static str {
    component
        .current_component_type
        .as_deref()
        .map(component_type_label)
        .unwrap_or("Table")
}

fn component_summary_status_label(component: &ComponentSummary) -> &'static str {
    if component.current_version_id.is_some() && component.draft_version_id.is_some() {
        "Updating"
    } else if component.current_version_id.is_some() {
        "Published"
    } else {
        "Draft"
    }
}

fn component_summary_revision_label(component: &ComponentSummary) -> String {
    component
        .current_version_label
        .as_deref()
        .or(component.draft_version_label.as_deref())
        .map(|label| format!("v{label}"))
        .unwrap_or_else(|| "Draft".into())
}

fn component_kind_filter_options(components: &[ComponentSummary]) -> Vec<String> {
    let mut options = components
        .iter()
        .map(|component| component_summary_kind_label(component).to_string())
        .collect::<Vec<_>>();
    options.sort();
    options.dedup();
    options
}

fn component_status_filter_options(components: &[ComponentSummary]) -> Vec<String> {
    let mut options = components
        .iter()
        .map(|component| component_summary_status_label(component).to_string())
        .collect::<Vec<_>>();
    options.sort();
    options.dedup();
    options
}

fn component_matches_filters(
    component: &ComponentSummary,
    search: &str,
    kind_filter: &str,
    status_filter: &str,
) -> bool {
    component_text_matches(search, &[&component.name])
        && (kind_filter == "all" || component_summary_kind_label(component) == kind_filter)
        && (status_filter == "all" || component_summary_status_label(component) == status_filter)
}

fn component_mobile_filter_button_label(kind_filter: &str, status_filter: &str) -> String {
    let active_count = [kind_filter, status_filter]
        .into_iter()
        .filter(|value| *value != "all")
        .count();
    if active_count == 0 {
        "Open component filters".into()
    } else {
        format!("Open component filters ({active_count} active)")
    }
}

fn component_text_matches(query: &str, values: &[&str]) -> bool {
    let query = query.trim().to_lowercase();
    query.is_empty()
        || values
            .iter()
            .any(|value| value.to_lowercase().contains(&query))
}

fn component_pagination_page_start(
    total_count: usize,
    page_size: usize,
    page_index: usize,
) -> usize {
    if total_count == 0 {
        0
    } else {
        let page_count = total_count.div_ceil(page_size);
        page_index.min(page_count.saturating_sub(1)) * page_size
    }
}

fn component_data_ops_field(field: &DatasetFieldDefinition) -> DataOpsDatasetFieldDraft {
    DataOpsDatasetFieldDraft {
        key: field.key.clone(),
        label: field.label.clone(),
        source_alias: "dataset".into(),
        source_field_key: field.key.clone(),
        field_type: field.field_type.clone(),
    }
}

fn dataset_catalog_option_label(dataset: &DatasetSummary, major: i32) -> String {
    let mut parts = vec![format!("{} · v{}", dataset.name, major)];
    if !dataset.tags.is_empty() {
        parts.push(dataset.tags.join(", "));
    }
    let provenance = dataset_provenance_label(&dataset.provenance);
    if provenance != "No direct sources" {
        parts.push(provenance);
    }
    parts.join(" · ")
}

fn selected_dataset_picker_label(
    datasets: &[DatasetSummary],
    dataset_id: &str,
    dataset_major: &str,
) -> String {
    let Some(major) = dataset_major.parse::<i32>().ok() else {
        return "Select a Dataset version".into();
    };
    datasets
        .iter()
        .find(|dataset| dataset.id == dataset_id)
        .map(|dataset| dataset_catalog_option_label(dataset, major))
        .unwrap_or_else(|| "Select a Dataset version".into())
}

fn dataset_picker_rows(datasets: &[DatasetSummary], query: &str) -> Vec<(DatasetSummary, i32)> {
    let query = query.trim().to_lowercase();
    datasets
        .iter()
        .flat_map(|dataset| {
            dataset_picker_majors(dataset)
                .into_iter()
                .map(move |major| (dataset.clone(), major))
        })
        .filter(|(dataset, major)| {
            query.is_empty()
                || dataset.name.to_lowercase().contains(&query)
                || format!("v{major}").contains(&query)
                || major.to_string().contains(&query)
                || dataset.tags.join(", ").to_lowercase().contains(&query)
                || dataset_provenance_label(&dataset.provenance)
                    .to_lowercase()
                    .contains(&query)
        })
        .collect()
}

fn dataset_provenance_label(provenance: &super::types::DatasetProvenanceSummary) -> String {
    let forms = provenance.forms.iter().map(|item| item.name.clone());
    let datasets = provenance
        .datasets
        .iter()
        .map(|item| format!("Dataset: {}", item.name));
    let sources = forms.chain(datasets).collect::<Vec<_>>();
    if sources.is_empty() {
        "No direct sources".into()
    } else {
        sources.join(", ")
    }
}

fn sentence_label(value: &str) -> String {
    value
        .split('_')
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

fn dataset_picker_majors(dataset: &DatasetSummary) -> Vec<i32> {
    if dataset.major_versions.is_empty() {
        dataset.current_version_major.into_iter().collect()
    } else {
        dataset.major_versions.clone()
    }
}

fn dataset_fields_for_major(
    dataset: &DatasetSummary,
    version_major: i32,
) -> Vec<DatasetFieldDefinition> {
    dataset
        .revisions
        .iter()
        .filter(|revision| revision.version_major == Some(version_major))
        .max_by_key(|revision| revision.version_number)
        .map(|revision| revision.output_fields.clone())
        .or_else(|| {
            (dataset.current_version_major == Some(version_major))
                .then(|| dataset.output_fields.clone())
        })
        .unwrap_or_default()
}

fn csv_field_keys(value: &str) -> Vec<String> {
    value
        .split(',')
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
        .collect()
}

#[cfg_attr(not(test), allow(dead_code))]
fn toggle_csv_key(value: &mut String, key: &str) {
    let mut keys = csv_field_keys(value);
    if keys.iter().any(|existing| existing == key) {
        keys.retain(|existing| existing != key);
    } else {
        keys.push(key.to_string());
    }
    *value = keys.join(", ");
}

#[cfg_attr(not(test), allow(dead_code))]
fn toggle_visible_column(value: &mut String, key: &str, all_keys: &[String]) {
    let mut keys = if value.trim().is_empty() {
        all_keys.to_vec()
    } else {
        csv_field_keys(value)
    };
    if keys.iter().any(|existing| existing == key) {
        keys.retain(|existing| existing != key);
    } else {
        keys.push(key.to_string());
    }
    if keys.len() == all_keys.len() && all_keys.iter().all(|key| keys.contains(key)) {
        value.clear();
    } else {
        *value = all_keys
            .iter()
            .filter(|key| keys.contains(key))
            .cloned()
            .collect::<Vec<_>>()
            .join(", ");
    }
}

#[cfg(feature = "hydrate")]
#[cfg_attr(not(feature = "hydrate"), allow(dead_code))]
fn load_components(
    components: RwSignal<Vec<ComponentSummary>>,
    is_loading: RwSignal<bool>,
    load_error: RwSignal<Option<String>>,
    can_manage_components: RwSignal<bool>,
) {
    leptos::task::spawn_local(async move {
        is_loading.set(true);
        load_error.set(None);
        match api::fetch_admin_components().await {
            Ok(Some(response)) => {
                can_manage_components.set(true);
                components.set(response);
            }
            Ok(None) => {
                can_manage_components.set(true);
                components.set(Vec::new());
            }
            Err(_) => match api::fetch_components().await {
                Ok(Some(response)) => {
                    can_manage_components.set(false);
                    components.set(response);
                }
                Ok(None) => {
                    can_manage_components.set(false);
                    components.set(Vec::new());
                }
                Err(message) => {
                    can_manage_components.set(false);
                    load_error.set(Some(message));
                }
            },
        }
        is_loading.set(false);
    });
}

#[cfg(not(feature = "hydrate"))]
fn load_components(
    _: RwSignal<Vec<ComponentSummary>>,
    is_loading: RwSignal<bool>,
    _: RwSignal<Option<String>>,
    _: RwSignal<bool>,
) {
    is_loading.set(false);
}

#[cfg(feature = "hydrate")]
fn load_component(
    component_ref: String,
    component: RwSignal<Option<ComponentDefinition>>,
    is_loading: RwSignal<bool>,
    load_error: RwSignal<Option<String>>,
    can_manage_component: RwSignal<bool>,
) {
    leptos::task::spawn_local(async move {
        is_loading.set(true);
        load_error.set(None);
        match fetch_authoring_or_reader_component(&component_ref).await {
            Ok((response, can_manage)) => {
                can_manage_component.set(can_manage);
                component.set(response);
            }
            Err(message) => {
                can_manage_component.set(false);
                load_error.set(Some(message));
            }
        }
        is_loading.set(false);
    });
}

#[cfg(feature = "hydrate")]
async fn fetch_authoring_or_reader_component(
    component_ref: &str,
) -> Result<(Option<ComponentDefinition>, bool), String> {
    match api::fetch_admin_component(component_ref).await {
        Ok(response) => Ok((response, true)),
        Err(_) => api::fetch_component(component_ref)
            .await
            .map(|response| (response, false)),
    }
}

#[cfg(feature = "hydrate")]
#[allow(clippy::too_many_arguments)]
fn load_component_for_edit(
    component_ref: String,
    editing_component_id: RwSignal<Option<String>>,
    editing_version_id: RwSignal<Option<String>>,
    current_published_version_id: RwSignal<Option<String>>,
    name: RwSignal<String>,
    slug: RwSignal<String>,
    description: RwSignal<String>,
    dataset_id: RwSignal<String>,
    dataset_major: RwSignal<String>,
    component_type: RwSignal<String>,
    columns: RwSignal<Vec<DataOpsDatasetFieldDraft>>,
    filters: RwSignal<Vec<DataOpsRowFilterDraft>>,
    sort_field: RwSignal<String>,
    sort_direction: RwSignal<String>,
    page_size: RwSignal<String>,
    visual_summary_field: RwSignal<String>,
    visual_summary_type: RwSignal<String>,
    visual_category_field: RwSignal<String>,
    visual_category_labels: RwSignal<String>,
    visual_category_colors: RwSignal<String>,
    visual_legend_title: RwSignal<String>,
    visual_comparison_field: RwSignal<String>,
    visual_bar_orientation: RwSignal<String>,
    visual_bar_comparison_layout: RwSignal<String>,
    visual_x_axis_label: RwSignal<String>,
    visual_y_axis_label: RwSignal<String>,
    visual_x_field: RwSignal<String>,
    visual_line_smoothing: RwSignal<bool>,
    visual_sort_field: RwSignal<String>,
    visual_sort_direction: RwSignal<String>,
    visual_limit: RwSignal<String>,
    visual_value_format: RwSignal<String>,
    visual_category_missing_policy: RwSignal<String>,
    visual_comparison_missing_policy: RwSignal<String>,
    visual_missing_policy: RwSignal<String>,
    stat_label: RwSignal<String>,
    stat_supporting_text: RwSignal<String>,
    stat_panel_style: RwSignal<String>,
    slug_manually_edited: RwSignal<bool>,
    error: RwSignal<Option<String>>,
) {
    leptos::task::spawn_local(async move {
        error.set(None);
        match api::fetch_admin_component(&component_ref).await {
            Ok(Some(component)) => {
                editing_component_id.set(Some(component.id.clone()));
                current_published_version_id.set(
                    component
                        .versions
                        .iter()
                        .find(|version| version.status == "published")
                        .map(|version| version.id.clone()),
                );
                name.set(component.name.clone());
                slug.set(component.slug.clone());
                slug_manually_edited.set(true);
                description.set(component.description.clone().unwrap_or_default());
                if let Some(version) = editable_component_version(&component) {
                    editing_version_id.set((version.status == "draft").then(|| version.id.clone()));
                    dataset_id.set(version.dataset_id);
                    dataset_major.set(version.dataset_version_major.to_string());
                    component_type.set(version.component_type.clone());
                    let (loaded_sort_field, loaded_sort_direction) =
                        table_sort_from_config(&version.config);
                    sort_field.set(loaded_sort_field);
                    sort_direction.set(loaded_sort_direction);
                    page_size.set(table_page_size_from_config(&version.config));
                    columns.set(table_projection_fields_from_config_keys(&version.config));
                    filters.set(table_filter_drafts_from_config(&version.config));
                    load_visual_config_signals(
                        &version.component_type,
                        &version.config,
                        visual_summary_field,
                        visual_summary_type,
                        visual_category_field,
                        visual_category_labels,
                        visual_category_colors,
                        visual_legend_title,
                        visual_comparison_field,
                        visual_bar_orientation,
                        visual_bar_comparison_layout,
                        visual_x_axis_label,
                        visual_y_axis_label,
                        visual_x_field,
                        visual_line_smoothing,
                        visual_sort_field,
                        visual_sort_direction,
                        visual_limit,
                        visual_value_format,
                        visual_category_missing_policy,
                        visual_comparison_missing_policy,
                        visual_missing_policy,
                        stat_label,
                        stat_supporting_text,
                        stat_panel_style,
                    );
                }
            }
            Ok(None) => error.set(Some("Component could not be loaded.".into())),
            Err(message) => error.set(Some(message)),
        }
    });
}

#[cfg(not(feature = "hydrate"))]
#[allow(clippy::too_many_arguments)]
fn load_component_for_edit(
    _component_ref: String,
    _editing_component_id: RwSignal<Option<String>>,
    _editing_version_id: RwSignal<Option<String>>,
    _current_published_version_id: RwSignal<Option<String>>,
    _name: RwSignal<String>,
    _slug: RwSignal<String>,
    _description: RwSignal<String>,
    _dataset_id: RwSignal<String>,
    _dataset_major: RwSignal<String>,
    _component_type: RwSignal<String>,
    _columns: RwSignal<Vec<DataOpsDatasetFieldDraft>>,
    _filters: RwSignal<Vec<DataOpsRowFilterDraft>>,
    _sort_field: RwSignal<String>,
    _sort_direction: RwSignal<String>,
    _page_size: RwSignal<String>,
    _visual_summary_field: RwSignal<String>,
    _visual_summary_type: RwSignal<String>,
    _visual_category_field: RwSignal<String>,
    _visual_category_labels: RwSignal<String>,
    _visual_category_colors: RwSignal<String>,
    _visual_legend_title: RwSignal<String>,
    _visual_comparison_field: RwSignal<String>,
    _visual_bar_orientation: RwSignal<String>,
    _visual_bar_comparison_layout: RwSignal<String>,
    _visual_x_axis_label: RwSignal<String>,
    _visual_y_axis_label: RwSignal<String>,
    _visual_x_field: RwSignal<String>,
    _visual_line_smoothing: RwSignal<bool>,
    _visual_sort_field: RwSignal<String>,
    _visual_sort_direction: RwSignal<String>,
    _visual_limit: RwSignal<String>,
    _visual_value_format: RwSignal<String>,
    _visual_category_missing_policy: RwSignal<String>,
    _visual_comparison_missing_policy: RwSignal<String>,
    _visual_missing_policy: RwSignal<String>,
    _stat_label: RwSignal<String>,
    _stat_supporting_text: RwSignal<String>,
    _stat_panel_style: RwSignal<String>,
    _slug_manually_edited: RwSignal<bool>,
    _error: RwSignal<Option<String>>,
) {
}

#[cfg(not(feature = "hydrate"))]
fn load_component(
    _: String,
    _: RwSignal<Option<ComponentDefinition>>,
    is_loading: RwSignal<bool>,
    _: RwSignal<Option<String>>,
    _: RwSignal<bool>,
) {
    is_loading.set(false);
}

#[cfg(feature = "hydrate")]
fn load_datasets(datasets: RwSignal<Vec<DatasetSummary>>, error: RwSignal<Option<String>>) {
    leptos::task::spawn_local(async move {
        error.set(None);
        match api::fetch_datasets().await {
            Ok(Some(response)) => datasets.set(response),
            Ok(None) => datasets.set(Vec::new()),
            Err(message) => error.set(Some(message)),
        }
    });
}

#[cfg(not(feature = "hydrate"))]
fn load_datasets(_: RwSignal<Vec<DatasetSummary>>, _: RwSignal<Option<String>>) {}

#[cfg_attr(not(feature = "hydrate"), allow(dead_code))]
fn component_preview_ready(values: &ComponentFormValues) -> bool {
    if values.dataset_id.trim().is_empty()
        || values.dataset_major.trim().parse::<i32>().is_err()
        || !visual_summary_field_ready(&values.visual_summary_type, &values.visual_summary_field)
    {
        return false;
    }
    match values.component_type.as_str() {
        "bar" | "pie" | "donut" => !values.visual_category_field.trim().is_empty(),
        "line" => !values.visual_x_field.trim().is_empty(),
        "stat_card" => true,
        _ => false,
    }
}

fn visual_summary_field_ready(summary_type: &str, summary_field: &str) -> bool {
    summary_type == "row_count" || !summary_field.trim().is_empty()
}

#[cfg(feature = "hydrate")]
fn schedule_component_editor_preview(
    values: ComponentFormValues,
    preview: RwSignal<Option<ComponentVisual>>,
    error: RwSignal<Option<String>>,
    loading: RwSignal<bool>,
    generation: RwSignal<u64>,
    timeout: RwSignal<Option<i32>>,
) {
    let Some(window) = web_sys::window() else {
        return;
    };
    if let Some(handle) = timeout.get_untracked() {
        window.clear_timeout_with_handle(handle);
        timeout.set(None);
    }
    if !component_preview_ready(&values) {
        request_component_editor_preview(values, preview, error, loading, generation);
        return;
    }
    let callback = Closure::once(Box::new(move || {
        timeout.set(None);
        request_component_editor_preview(values, preview, error, loading, generation);
    }) as Box<dyn FnOnce()>);
    if let Ok(handle) = window.set_timeout_with_callback_and_timeout_and_arguments_0(
        callback.as_ref().unchecked_ref(),
        250,
    ) {
        timeout.set(Some(handle));
        callback.forget();
    }
}

#[cfg(not(feature = "hydrate"))]
fn schedule_component_editor_preview(
    _: ComponentFormValues,
    _: RwSignal<Option<ComponentVisual>>,
    _: RwSignal<Option<String>>,
    _: RwSignal<bool>,
    _: RwSignal<u64>,
    _: RwSignal<Option<i32>>,
) {
}

#[cfg(feature = "hydrate")]
fn request_component_editor_preview(
    values: ComponentFormValues,
    preview: RwSignal<Option<ComponentVisual>>,
    error: RwSignal<Option<String>>,
    loading: RwSignal<bool>,
    generation: RwSignal<u64>,
) {
    if !component_preview_ready(&values) {
        generation.update(|value| *value += 1);
        preview.set(None);
        error.set(None);
        loading.set(false);
        return;
    }
    generation.update(|value| *value += 1);
    let request_generation = generation.get_untracked();
    let payload = CreateComponentVersionRequest {
        dataset_id: Some(values.dataset_id.clone()),
        dataset_version_major: values.dataset_major.trim().parse::<i32>().ok(),
        component_type: values.component_type.clone(),
        config: build_component_config(&values),
        version_note: None,
    };
    loading.set(true);
    error.set(None);
    leptos::task::spawn_local(async move {
        let result = api::preview_component_visual(payload).await;
        if generation.get_untracked() != request_generation {
            return;
        }
        loading.set(false);
        match result {
            Ok(next_preview) => {
                preview.set(Some(next_preview));
                error.set(None);
            }
            Err(message) => {
                preview.set(None);
                error.set(Some(message));
            }
        }
    });
}

#[cfg(not(feature = "hydrate"))]
#[allow(dead_code)]
fn request_component_editor_preview(
    _: ComponentFormValues,
    _: RwSignal<Option<ComponentVisual>>,
    _: RwSignal<Option<String>>,
    _: RwSignal<bool>,
    _: RwSignal<u64>,
) {
}

#[cfg(feature = "hydrate")]
#[derive(Clone, Copy)]
struct CategoryValueLoadState {
    active_dataset_id: RwSignal<String>,
    active_dataset_major: RwSignal<String>,
    active_category_field: RwSignal<String>,
    values: RwSignal<Vec<String>>,
    error: RwSignal<Option<String>>,
}

#[cfg(feature = "hydrate")]
fn load_category_values(
    dataset_id: String,
    dataset_major: String,
    category_field: String,
    state: CategoryValueLoadState,
) {
    leptos::task::spawn_local(async move {
        state.error.set(None);
        let Ok(version_major) = dataset_major.parse::<i32>() else {
            state.values.set(Vec::new());
            return;
        };
        match api::fetch_dataset_distinct_values(&dataset_id, version_major, &category_field).await
        {
            Ok(Some(response)) => {
                if state.active_dataset_id.get_untracked() != dataset_id
                    || state.active_dataset_major.get_untracked() != dataset_major
                    || state.active_category_field.get_untracked() != category_field
                {
                    return;
                }
                state.values.set(response.values);
            }
            Ok(None) => {
                if state.active_dataset_id.get_untracked() != dataset_id
                    || state.active_dataset_major.get_untracked() != dataset_major
                    || state.active_category_field.get_untracked() != category_field
                {
                    return;
                }
                state.values.set(Vec::new());
            }
            Err(message) => {
                if state.active_dataset_id.get_untracked() != dataset_id
                    || state.active_dataset_major.get_untracked() != dataset_major
                    || state.active_category_field.get_untracked() != category_field
                {
                    return;
                }
                state.values.set(Vec::new());
                state.error.set(Some(message));
            }
        }
    });
}

#[cfg(not(feature = "hydrate"))]
#[allow(dead_code)]
#[derive(Clone, Copy)]
struct CategoryValueLoadState {
    active_dataset_id: RwSignal<String>,
    active_dataset_major: RwSignal<String>,
    active_category_field: RwSignal<String>,
    values: RwSignal<Vec<String>>,
    error: RwSignal<Option<String>>,
}

#[cfg(not(feature = "hydrate"))]
fn load_category_values(_: String, _: String, _: String, _: CategoryValueLoadState) {}

#[cfg(feature = "hydrate")]
fn load_component_table(
    component_ref: String,
    query: String,
    table: RwSignal<Option<ComponentTable>>,
    error: RwSignal<Option<String>>,
) {
    leptos::task::spawn_local(async move {
        error.set(None);
        match api::fetch_component_table(&component_ref, &query).await {
            Ok(Some(response)) => table.set(Some(response)),
            Ok(None) => table.set(None),
            Err(message) => error.set(Some(message)),
        }
    });
}

#[cfg(not(feature = "hydrate"))]
fn load_component_table(
    _: String,
    _: String,
    _: RwSignal<Option<ComponentTable>>,
    _: RwSignal<Option<String>>,
) {
}

#[cfg(feature = "hydrate")]
fn load_component_visual(
    component_ref: String,
    component_type: String,
    visual: RwSignal<Option<ComponentVisual>>,
    error: RwSignal<Option<String>>,
) {
    leptos::task::spawn_local(async move {
        error.set(None);
        match api::fetch_component_visual(&component_ref, &component_type).await {
            Ok(Some(response)) => visual.set(Some(response)),
            Ok(None) => visual.set(None),
            Err(message) => error.set(Some(message)),
        }
    });
}

#[cfg(not(feature = "hydrate"))]
fn load_component_visual(
    _: String,
    _: String,
    _: RwSignal<Option<ComponentVisual>>,
    _: RwSignal<Option<String>>,
) {
}

fn published_component_type(component: Option<ComponentDefinition>) -> Option<String> {
    component
        .and_then(|component| {
            component
                .versions
                .into_iter()
                .find(|version| version.status == "published")
        })
        .map(|version| version.component_type)
}

#[cfg_attr(not(feature = "hydrate"), allow(dead_code))]
struct ComponentFormValues {
    name: String,
    slug: String,
    description: String,
    dataset_id: String,
    dataset_major: String,
    component_type: String,
    columns: Vec<DataOpsDatasetFieldDraft>,
    filters: Vec<DataOpsRowFilterDraft>,
    sort_field: String,
    sort_direction: String,
    page_size: String,
    visual_summary_field: String,
    visual_summary_type: String,
    visual_category_field: String,
    visual_category_labels: String,
    visual_category_colors: String,
    visual_legend_title: String,
    visual_comparison_field: String,
    visual_bar_orientation: String,
    visual_bar_comparison_layout: String,
    visual_x_axis_label: String,
    visual_y_axis_label: String,
    visual_x_field: String,
    visual_line_smoothing: bool,
    visual_sort_field: String,
    visual_sort_direction: String,
    visual_limit: String,
    visual_value_format: String,
    visual_category_missing_policy: String,
    visual_comparison_missing_policy: String,
    visual_missing_policy: String,
    stat_label: String,
    stat_supporting_text: String,
    stat_panel_style: String,
}

#[cfg_attr(not(feature = "hydrate"), allow(dead_code))]
struct ComponentSaveIntent {
    editing_component_id: Option<String>,
    editing_version_id: Option<String>,
    current_published_version_id: Option<String>,
    publish_action: ComponentPublishAction,
    version_note: Option<String>,
}

#[cfg_attr(not(feature = "hydrate"), allow(dead_code))]
struct ComponentFormFeedback {
    message: RwSignal<Option<String>>,
    error: RwSignal<Option<String>>,
    findings: RwSignal<Vec<ComponentValidationFinding>>,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum ComponentPublishAction {
    SaveDraft,
    UpdateExistingVersion,
    CreateNewVersion,
}

#[cfg(feature = "hydrate")]
fn create_component_from_form(
    intent: ComponentSaveIntent,
    values: ComponentFormValues,
    feedback: ComponentFormFeedback,
) {
    leptos::task::spawn_local(async move {
        feedback.message.set(None);
        feedback.error.set(None);
        feedback.findings.set(Vec::new());
        let major = values.dataset_major.trim().parse::<i32>().unwrap_or(1);
        let config = build_component_config(&values);
        let version = CreateComponentVersionRequest {
            dataset_id: Some(values.dataset_id),
            dataset_version_major: Some(major),
            component_type: values.component_type.clone(),
            config,
            version_note: normalized_component_version_note(intent.version_note),
        };
        match api::validate_component_version(version.clone()).await {
            Ok(response) if response.valid => {}
            Ok(response) => {
                feedback.findings.set(response.findings);
                return;
            }
            Err(message) => {
                feedback.error.set(Some(message));
                return;
            }
        }
        let description = if values.description.trim().is_empty() {
            None
        } else {
            Some(values.description.trim().to_string())
        };
        let redirect_ref = component_redirect_ref(&values.slug);
        let action = match intent.publish_action {
            ComponentPublishAction::SaveDraft => "save_draft",
            ComponentPublishAction::UpdateExistingVersion => "update_existing_version",
            ComponentPublishAction::CreateNewVersion => "create_new_version",
        };
        let request = SaveComponentEditRequest {
            component_id: intent.editing_component_id,
            draft_version_id: intent.editing_version_id,
            published_version_id: intent.current_published_version_id,
            action: action.into(),
            component: UpdateComponentRequest {
                name: values.name,
                slug: values.slug,
                description,
            },
            version,
        };
        match api::save_component_edit(request).await {
            Ok(_) => {
                let saved_message = match intent.publish_action {
                    ComponentPublishAction::SaveDraft => "Component draft saved.",
                    ComponentPublishAction::UpdateExistingVersion => {
                        "Existing component version updated."
                    }
                    ComponentPublishAction::CreateNewVersion => "Component saved and published.",
                };
                feedback.message.set(Some(saved_message.into()));
                if let Some(window) = web_sys::window() {
                    let target = if intent.publish_action == ComponentPublishAction::SaveDraft {
                        format!("/components/{redirect_ref}/edit")
                    } else {
                        format!("/components/{redirect_ref}")
                    };
                    let _ = window.location().set_href(&target);
                }
            }
            Err(message) => feedback.error.set(Some(message)),
        }
    });
}

#[cfg_attr(not(any(feature = "hydrate", test)), allow(dead_code))]
fn component_redirect_ref(slug: &str) -> String {
    slug.trim().to_string()
}

#[cfg_attr(not(feature = "hydrate"), allow(dead_code))]
fn normalized_component_version_note(note: Option<String>) -> Option<String> {
    note.map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

fn commit_component_name(
    name: RwSignal<String>,
    slug: RwSignal<String>,
    slug_manually_edited: RwSignal<bool>,
    value: String,
) {
    let derived_slug = snake_case_component_slug(&value);
    name.set(value);

    if !slug_manually_edited.get_untracked() {
        slug.set(derived_slug);
    }
}

fn snake_case_component_slug(value: &str) -> String {
    let mut slug = String::new();
    let mut previous_was_separator = false;

    for character in value.trim().chars() {
        if character.is_ascii_alphanumeric() {
            slug.push(character.to_ascii_lowercase());
            previous_was_separator = false;
        } else if !previous_was_separator && !slug.is_empty() {
            slug.push('_');
            previous_was_separator = true;
        }
    }

    slug.trim_end_matches('_').to_string()
}

#[cfg(not(feature = "hydrate"))]
fn create_component_from_form(
    _: ComponentSaveIntent,
    _: ComponentFormValues,
    _: ComponentFormFeedback,
) {
}

#[cfg(feature = "hydrate")]
fn delete_component_draft(
    component_id: String,
    version_id: String,
    message: RwSignal<Option<String>>,
    error: RwSignal<Option<String>>,
    findings: RwSignal<Vec<ComponentValidationFinding>>,
) {
    let confirmed = web_sys::window()
        .and_then(|window| {
            window
                .confirm_with_message(
                    "Discard this component draft? Published versions will remain available.",
                )
                .ok()
        })
        .unwrap_or(false);
    if !confirmed {
        return;
    }

    leptos::task::spawn_local(async move {
        message.set(None);
        error.set(None);
        findings.set(Vec::new());
        match api::delete_component_version(&component_id, &version_id).await {
            Ok(_) => {
                message.set(Some("Component draft discarded.".into()));
                if let Some(window) = web_sys::window() {
                    let _ = window.location().set_href("/components");
                }
            }
            Err(message) => error.set(Some(message)),
        }
    });
}

#[cfg(not(feature = "hydrate"))]
fn delete_component_draft(
    _: String,
    _: String,
    _: RwSignal<Option<String>>,
    _: RwSignal<Option<String>>,
    _: RwSignal<Vec<ComponentValidationFinding>>,
) {
}

#[cfg(test)]
mod tests;
