use leptos::prelude::*;

use crate::features::native_shell::{
    BreadcrumbItem, MetadataStrip, NativePage, PageHeader, Panel, use_account_session,
};
use crate::infra::routing::{FormRouteParams, require_route_params};

#[cfg(feature = "hydrate")]
mod hydrate {
    use std::{cell::RefCell, rc::Rc};

    use crate::features::native_runtime::{
        by_id, delete_json, escape_html, get_json, input_value, post_json, put_json, redirect,
        select_value, set_html, set_input_value, set_page_context, set_select_value, set_text,
        textarea_value,
    };
    use serde::Deserialize;
    use serde_json::json;
    use wasm_bindgen::{JsCast, closure::Closure};
    use wasm_bindgen_futures::spawn_local;

    #[derive(Clone, Deserialize)]
    struct NodeTypeSummary {
        id: String,
        name: String,
    }

    #[derive(Clone, Deserialize)]
    struct FormPublishPreview {
        version_label: String,
        version_major: i32,
        version_minor: i32,
        version_patch: i32,
        semantic_bump: String,
        compatibility_label: String,
        starts_new_major_line: bool,
        dependency_warnings: Vec<String>,
    }

    #[derive(Clone, Deserialize)]
    struct FormVersionSummary {
        id: String,
        version_label: Option<String>,
        status: String,
        field_count: i64,
        semantic_bump: Option<String>,
        version_major: Option<i32>,
        version_minor: Option<i32>,
        version_patch: Option<i32>,
        published_at: Option<String>,
        compatibility_group_name: Option<String>,
        publish_preview: Option<FormPublishPreview>,
    }

    #[derive(Clone, Deserialize)]
    struct FormSummary {
        id: String,
        name: String,
        slug: String,
        scope_node_type_name: Option<String>,
        versions: Vec<FormVersionSummary>,
    }

    #[derive(Clone, Deserialize)]
    struct FormWorkflowLink {
        id: String,
        name: String,
        current_version_label: Option<String>,
        current_status: Option<String>,
        assignment_count: i64,
    }

    #[derive(Clone, Deserialize)]
    struct FormReportLink {
        id: String,
        name: String,
    }

    #[derive(Clone, Deserialize)]
    struct FormDatasetSourceLink {
        dataset_name: String,
        source_alias: String,
        selection_rule: String,
    }

    #[derive(Clone, Deserialize)]
    struct FormDefinition {
        id: String,
        name: String,
        slug: String,
        scope_node_type_id: Option<String>,
        scope_node_type_name: Option<String>,
        versions: Vec<FormVersionSummary>,
        workflows: Vec<FormWorkflowLink>,
        reports: Vec<FormReportLink>,
        dataset_sources: Vec<FormDatasetSourceLink>,
    }

    #[derive(Clone, Deserialize)]
    struct RenderedForm {
        sections: Vec<RenderedSection>,
    }

    #[derive(Clone, Deserialize)]
    struct RenderedSection {
        id: String,
        title: String,
        description: String,
        column_count: i32,
        position: i32,
        fields: Vec<RenderedField>,
    }

    #[derive(Clone, Deserialize)]
    struct RenderedField {
        id: String,
        key: String,
        label: String,
        field_type: String,
        required: bool,
        position: i32,
    }

    #[derive(Clone, Deserialize)]
    struct IdResponse {
        id: String,
    }

    #[derive(Clone, Deserialize)]
    struct PublishFormVersionResponse {
        version_label: String,
        dependency_warnings: Vec<String>,
    }

    #[derive(Clone, Default)]
    struct FormBuilderState {
        form: Option<FormDefinition>,
        rendered_version: Option<RenderedForm>,
        selected_version_id: Option<String>,
        selected_section_id: Option<String>,
        selected_field_id: Option<String>,
    }

    fn options_html<T>(
        items: &[T],
        value: impl Fn(&T) -> &str,
        label: impl Fn(&T) -> String,
        placeholder: &str,
    ) -> String {
        let mut html = format!(r#"<option value="">{}</option>"#, escape_html(placeholder));
        for item in items {
            html.push_str(&format!(
                r#"<option value="{}">{}</option>"#,
                escape_html(value(item)),
                escape_html(&label(item)),
            ));
        }
        html
    }

    fn form_version_label(version: &FormVersionSummary) -> String {
        version
            .version_label
            .clone()
            .or_else(|| {
                version
                    .publish_preview
                    .as_ref()
                    .map(|preview| preview.version_label.clone())
            })
            .unwrap_or_else(|| "Draft version".into())
    }

    fn form_version_compatibility(version: &FormVersionSummary) -> String {
        if let Some(group) = version.compatibility_group_name.as_ref() {
            return group.clone();
        }
        if let Some(preview) = version.publish_preview.as_ref() {
            return preview.compatibility_label.clone();
        }
        match (
            version.version_major,
            version.version_minor,
            version.version_patch,
        ) {
            (Some(major), Some(minor), Some(patch)) => format!("v{major}.{minor}.{patch}"),
            (Some(major), _, _) => format!("Compatible with v{major}.x"),
            _ => "Compatibility assigned on publish".into(),
        }
    }

    fn preferred_version(
        form: &FormDefinition,
        preferred_version_id: Option<&str>,
    ) -> Option<FormVersionSummary> {
        if let Some(preferred_version_id) = preferred_version_id {
            if let Some(version) = form
                .versions
                .iter()
                .find(|version| version.id == preferred_version_id)
            {
                return Some(version.clone());
            }
        }
        form.versions
            .iter()
            .rev()
            .find(|version| version.status == "draft")
            .cloned()
            .or_else(|| {
                form.versions
                    .iter()
                    .rev()
                    .find(|version| version.status == "published")
                    .cloned()
            })
            .or_else(|| form.versions.last().cloned())
    }

    fn active_draft_version(form: &FormDefinition) -> Option<FormVersionSummary> {
        form.versions
            .iter()
            .rev()
            .find(|version| version.status == "draft")
            .cloned()
    }

    fn render_form_cards(forms: &[FormSummary]) -> String {
        if forms.is_empty() {
            return r#"<p class="muted">No form records found.</p>"#.into();
        }
        forms.iter()
            .map(|form| {
                let published = form
                    .versions
                    .iter()
                    .rev()
                    .find(|version| version.status == "published")
                    .map(form_version_label)
                    .unwrap_or_else(|| "None".into());
                let draft_count = form
                    .versions
                    .iter()
                    .filter(|version| version.status == "draft")
                    .count();
                format!(
                    r#"<article class="record-card form-directory-card"><div class="page-title-row compact-title-row"><div><p class="eyebrow">Form</p><h4>{}</h4><p class="muted">/{}</p></div><div class="actions"><a class="button-link" href="/app/forms/{}">Open</a><a class="button-link" href="/app/forms/{}/edit">Edit Drafts</a></div></div><div class="record-detail form-directory-card__meta"><p class="muted">Scope root: {}</p><p class="muted">Published line: {}</p><p class="muted">Draft versions ready: {}</p></div></article>"#,
                    escape_html(&form.name),
                    escape_html(&form.slug),
                    escape_html(&form.id),
                    escape_html(&form.id),
                    escape_html(form.scope_node_type_name.as_deref().unwrap_or("Unscoped")),
                    escape_html(&published),
                    draft_count,
                )
            })
            .collect::<Vec<_>>()
            .join("")
    }

    fn render_version_summary(
        versions: &[FormVersionSummary],
        selected_id: Option<&str>,
    ) -> String {
        if versions.is_empty() {
            return r#"<p class="muted">No form versions are available.</p>"#.into();
        }
        versions
            .iter()
            .map(|version| {
                let preview = version
                    .publish_preview
                    .as_ref()
                    .map(|preview| {
                        format!(
                            r#"<p class="muted">Publish preview: {} ({})</p>"#,
                            escape_html(&preview.version_label),
                            escape_html(&preview.semantic_bump),
                        )
                    })
                    .unwrap_or_default();
                format!(
                    r#"<article class="record-card form-version-summary-card {}"><div class="page-title-row compact-title-row"><div><p class="eyebrow">Version</p><h4>{}</h4><p class="muted">Status: {} · Compatibility: {}</p></div></div><div class="record-detail"><p class="muted">Fields authored: {}</p><p class="muted">Published: {}</p>{}{}</div></article>"#,
                    if Some(version.id.as_str()) == selected_id {
                        "compact-record-card"
                    } else {
                        ""
                    },
                    escape_html(&form_version_label(version)),
                    escape_html(&version.status),
                    escape_html(&form_version_compatibility(version)),
                    version.field_count,
                    escape_html(version.published_at.as_deref().unwrap_or("Not published")),
                    version
                        .semantic_bump
                        .as_ref()
                        .map(|value| format!(
                            r#"<p class="muted">Semantic bump: {}</p>"#,
                            escape_html(value)
                        ))
                        .unwrap_or_default(),
                    preview,
                )
            })
            .collect::<Vec<_>>()
            .join("")
    }

    fn render_form_preview(version: &FormVersionSummary, rendered: &RenderedForm) -> String {
        let sections = if rendered.sections.is_empty() {
            r#"<p class="muted">No sections were added to this version yet.</p>"#.into()
        } else {
            rendered
                .sections
                .iter()
                .enumerate()
                .map(|(section_index, section)| {
                    let fields = if section.fields.is_empty() {
                        r#"<p class="muted">No fields in this section yet.</p>"#.into()
                    } else {
                        section
                            .fields
                            .iter()
                            .map(|field| {
                                format!(
                                    r#"<article class="record-card compact-record-card form-preview-field"><h4>{}</h4><p class="muted">Key: {}</p><p class="muted">Type: {}</p><p class="muted">{}</p></article>"#,
                                    escape_html(&field.label),
                                    escape_html(&field.key),
                                    escape_html(&field.field_type),
                                    if field.required { "Required field" } else { "Optional field" },
                                )
                            })
                            .collect::<Vec<_>>()
                            .join("")
                    };
                    format!(
                        r#"<section class="nested-form-panel form-preview-section"><div class="page-title-row compact-title-row"><div><p class="eyebrow">Section {}</p><h3>{}</h3>{}</div><div class="actions"><span class="muted">Display order {}</span><span class="muted">{}</span></div></div><div class="record-list">{}</div></section>"#,
                        section_index + 1,
                        escape_html(&section.title),
                        if section.description.trim().is_empty() {
                            String::new()
                        } else {
                            format!(r#"<p class="muted">{}</p>"#, escape_html(&section.description))
                        },
                        section.position,
                        escape_html(&form_section_layout_label(section.column_count)),
                        fields,
                    )
                })
                .collect::<Vec<_>>()
                .join("")
        };

        format!(
            r#"<article class="record-card form-version-preview-card"><div class="page-title-row compact-title-row"><div><p class="eyebrow">Preview</p><h4>{}</h4><p class="muted">Status: {} · Compatibility: {}</p></div></div></article>{}"#,
            escape_html(&form_version_label(version)),
            escape_html(&version.status),
            escape_html(&form_version_compatibility(version)),
            sections,
        )
    }

    fn render_workflow_attachments(form: &FormDefinition) -> String {
        let workflows = if form.workflows.is_empty() {
            r#"<li class="muted">No related workflows.</li>"#.into()
        } else {
            form.workflows
                .iter()
                .map(|workflow| {
                    format!(
                        r#"<li><a href="/app/workflows/{}">{}</a> <span class="muted">{} | {} | Assignments: {}</span> <span><a href="/app/workflows/assignments?workflowId={}">Assignments</a></span></li>"#,
                        escape_html(&workflow.id),
                        escape_html(&workflow.name),
                        escape_html(workflow.current_version_label.as_deref().unwrap_or("No version")),
                        escape_html(workflow.current_status.as_deref().unwrap_or("draft")),
                        workflow.assignment_count,
                        escape_html(&workflow.id),
                    )
                })
                .collect::<Vec<_>>()
                .join("")
        };
        let reports = if form.reports.is_empty() {
            r#"<li class="muted">No related reports.</li>"#.into()
        } else {
            form.reports
                .iter()
                .map(|report| {
                    format!(
                        r#"<li><a href="/app/reports/{}">{}</a></li>"#,
                        escape_html(&report.id),
                        escape_html(&report.name),
                    )
                })
                .collect::<Vec<_>>()
                .join("")
        };
        let datasets = if form.dataset_sources.is_empty() {
            r#"<li class="muted">No related dataset sources.</li>"#.into()
        } else {
            form.dataset_sources
                .iter()
                .map(|dataset| {
                    format!(
                        r#"<li>{} ({}, {})</li>"#,
                        escape_html(&dataset.dataset_name),
                        escape_html(&dataset.source_alias),
                        escape_html(&dataset.selection_rule),
                    )
                })
                .collect::<Vec<_>>()
                .join("")
        };

        format!(
            r#"<section class="detail-section form-related-work-group"><div class="page-title-row compact-title-row"><div><p class="eyebrow">Related Work</p><h4>Workflows</h4></div><div class="actions"><a class="button-link" href="/app/workflows/assignments?formId={}">Open Assignment Console</a></div></div><ul class="app-list">{}</ul></section><section class="detail-section form-related-work-group"><div class="page-title-row compact-title-row"><div><p class="eyebrow">Related Work</p><h4>Reports</h4></div></div><ul class="app-list">{}</ul></section><section class="detail-section form-related-work-group"><div class="page-title-row compact-title-row"><div><p class="eyebrow">Related Work</p><h4>Dataset Sources</h4></div></div><ul class="app-list">{}</ul></section>"#,
            escape_html(&form.id),
            workflows,
            reports,
            datasets,
        )
    }

    fn render_form_version_cards(
        form: &FormDefinition,
        selected_version_id: Option<&str>,
        editable: bool,
    ) -> String {
        if form.versions.is_empty() {
            return r#"<p class="muted">No form versions are available yet.</p>"#.into();
        }
        form.versions
            .iter()
            .map(|version| {
                let publish = if editable && version.status == "draft" {
                    format!(
                        r#"<button class="button is-light" type="button" data-publish-form-version="{}">Publish</button>"#,
                        escape_html(&version.id),
                    )
                } else {
                    String::new()
                };
                let preview = version
                    .publish_preview
                    .as_ref()
                    .map(|preview| {
                        format!(
                            r#"<p class="muted">Publish preview: {} ({})</p><p class="muted">Compatibility: {}</p><p class="muted">Dependency warnings: {}</p>"#,
                            escape_html(&preview.version_label),
                            escape_html(&preview.semantic_bump),
                            escape_html(&preview.compatibility_label),
                            preview.dependency_warnings.len(),
                        )
                    })
                    .unwrap_or_default();
                format!(
                    r#"<article class="record-card form-version-card {}"><div class="page-title-row compact-title-row"><div><p class="eyebrow">Version</p><h4>{}</h4><p class="muted">Status: {} · Compatibility: {}</p></div><div class="actions"><button class="button is-light" type="button" data-preview-form-version="{}">{}</button>{}</div></div><div class="record-detail"><p class="muted">Published: {}</p><p class="muted">Fields authored: {}</p>{}</div></article>"#,
                    if Some(version.id.as_str()) == selected_version_id {
                        "compact-record-card"
                    } else {
                        ""
                    },
                    escape_html(&form_version_label(version)),
                    escape_html(&version.status),
                    escape_html(&form_version_compatibility(version)),
                    escape_html(&version.id),
                    if editable { "Open Workspace" } else { "Preview" },
                    publish,
                    escape_html(version.published_at.as_deref().unwrap_or("Not published")),
                    version.field_count,
                    preview,
                )
            })
            .collect::<Vec<_>>()
            .join("")
    }

    fn render_form_field_type_options(selected_value: &str) -> String {
        ["text", "number", "boolean", "date", "multi_choice"]
            .iter()
            .map(|field_type| {
                format!(
                    r#"<option value="{}" {}>{}</option>"#,
                    escape_html(field_type),
                    if *field_type == selected_value {
                        "selected"
                    } else {
                        ""
                    },
                    escape_html(field_type),
                )
            })
            .collect::<Vec<_>>()
            .join("")
    }

    fn render_form_section_options(rendered: &RenderedForm, section_id: &str) -> String {
        rendered
            .sections
            .iter()
            .map(|section| {
                format!(
                    r#"<option value="{}" {}>{}</option>"#,
                    escape_html(&section.id),
                    if section.id == section_id {
                        "selected"
                    } else {
                        ""
                    },
                    escape_html(&section.title),
                )
            })
            .collect::<Vec<_>>()
            .join("")
    }

    fn render_form_section_column_count_options(selected: i32) -> String {
        [1, 2]
            .into_iter()
            .map(|column_count| {
                format!(
                    r#"<option value="{column_count}" {}>{column_count} {}</option>"#,
                    if selected == column_count {
                        "selected"
                    } else {
                        ""
                    },
                    if column_count == 1 {
                        "column"
                    } else {
                        "columns"
                    },
                )
            })
            .collect::<Vec<_>>()
            .join("")
    }

    fn form_section_layout_label(column_count: i32) -> String {
        format!(
            "{column_count} {}",
            if column_count == 1 {
                "column"
            } else {
                "columns"
            }
        )
    }

    fn render_form_version_lifecycle_summary(version: &FormVersionSummary) -> String {
        let preview = version
            .publish_preview
            .as_ref()
            .map(|preview| {
                format!(
                    r#"<p class="muted">Publish preview: {} | {} | {}</p><p class="muted">Starts new major line: {}</p>"#,
                    escape_html(&preview.version_label),
                    escape_html(&preview.semantic_bump),
                    escape_html(&preview.compatibility_label),
                    if preview.starts_new_major_line { "Yes" } else { "No" },
                )
            })
            .unwrap_or_default();
        let semantic_bump = version
            .semantic_bump
            .as_ref()
            .map(|value| {
                format!(
                    r#"<p class="muted">Last semantic bump: {}</p>"#,
                    escape_html(value)
                )
            })
            .unwrap_or_default();
        format!("{preview}{semantic_bump}")
    }

    fn render_form_version_create_panel(form: &FormDefinition) -> String {
        if let Some(draft) = active_draft_version(form) {
            return format!(
                r#"<p class="muted">One draft version is allowed at a time. Open the active draft workspace or publish it before creating another.</p><div class="actions"><button class="button is-primary" type="submit" disabled>Draft Already Open ({})</button></div>"#,
                escape_html(&form_version_label(&draft))
            );
        }

        r#"<p class="muted">Draft versions stay unlabeled until publish. Semantic version and major-line compatibility are assigned automatically when you publish.</p><div class="actions"><button class="button is-primary" type="submit">Create Draft Version</button></div>"#
            .into()
    }

    fn form_field_kind_label(field_type: &str) -> &str {
        match field_type {
            "text" => "Short answer",
            "number" => "Number",
            "boolean" => "Yes / No",
            "date" => "Date",
            "single_choice" => "Single choice",
            "multi_choice" => "Multiple choice",
            _ => field_type,
        }
    }

    fn form_field_preview_copy(field_type: &str) -> &str {
        match field_type {
            "text" => "Respondents will see a single-line text input here.",
            "number" => "Respondents will see a numeric input here.",
            "boolean" => "Respondents will see a yes / no toggle here.",
            "date" => "Respondents will see a date picker here.",
            "single_choice" => "Respondents will choose one option from a list here.",
            "multi_choice" => "Respondents will choose one or more options here.",
            _ => "Respondents will complete this control from the generated response form.",
        }
    }

    fn render_builder_section_navigation(
        rendered: &RenderedForm,
        selected_section_id: Option<&str>,
    ) -> String {
        if rendered.sections.is_empty() {
            return r#"<p class="muted form-builder-rail-empty">Add a first section to activate the canvas rail.</p>"#
                .into();
        }

        rendered
            .sections
            .iter()
            .enumerate()
            .map(|(section_index, section)| {
                format!(
                    r##"<a class="form-builder-section-link{}" href="#builder-section-{}" data-select-form-section="{}"><span>Section {}</span><strong>{}</strong><small>{}</small></a>"##,
                    if Some(section.id.as_str()) == selected_section_id {
                        " is-selected"
                    } else {
                        ""
                    },
                    escape_html(&section.id),
                    escape_html(&section.id),
                    section_index + 1,
                    escape_html(&section.title),
                    escape_html(&form_section_layout_label(section.column_count)),
                )
            })
            .collect::<Vec<_>>()
            .join("")
    }

    fn render_builder_insert_rail(selected_section: Option<&RenderedSection>) -> String {
        let add_field_button = if let Some(section) = selected_section {
            format!(
                r#"<button class="button is-light" type="button" data-form-field-create="{}">Add Field To Selected Section</button><p class="muted">New fields open in the properties panel with generated defaults so authors can refine them in context.</p>"#,
                escape_html(&section.id),
            )
        } else {
            r#"<button class="button is-light" type="button" disabled>Select A Section First</button><p class="muted">Choose a section from the rail or canvas before inserting a new field.</p>"#
                .into()
        };

        format!(
            r#"<aside class="detail-section box form-builder-insert-rail"><div class="page-title-row compact-title-row"><div><p class="form-builder-eyebrow">Insert Rail</p><h4>Add To Draft</h4><p class="muted">Keep the next authoring move near the canvas instead of in the global shell.</p></div></div><ul class="app-list form-builder-insert-catalog"><li><strong>Short answer</strong><span class="muted">Single-line response fields and short notes</span></li><li><strong>Number</strong><span class="muted">Quantities, counts, and numeric checks</span></li><li><strong>Date</strong><span class="muted">Schedule and reporting-period inputs</span></li><li><strong>Multiple choice</strong><span class="muted">Inline option lists until reusable sources land</span></li></ul><div class="actions form-builder-insert-actions"><button class="button" type="button" data-form-section-create="quick">Add Section</button>{}</div></aside>"#,
            add_field_button
        )
    }

    fn render_builder_properties_panel(
        rendered: &RenderedForm,
        selected_field_id: Option<&str>,
    ) -> String {
        let Some(selected_field_id) = selected_field_id else {
            return String::new();
        };
        let Some((field, section)) = current_rendered_field(rendered, selected_field_id) else {
            return String::new();
        };

        format!(
            r#"<aside class="detail-section box form-builder-properties" id="form-builder-properties"><div class="page-title-row compact-title-row"><div><p class="form-builder-eyebrow">Properties</p><h3>{}</h3><p class="muted">Edit deeper field configuration here while keeping the stacked section canvas in view.</p></div><div class="actions"><button class="button is-light" type="button" data-clear-form-field-selection="close">Close</button></div></div><div class="form-grid"><div class="form-field wide-field"><label for="form-field-label-{}">Field Label</label><input class="input" id="form-field-label-{}" type="text" value="{}" /></div><div class="form-field"><label for="form-field-key-{}">Field Key</label><input class="input" id="form-field-key-{}" type="text" value="{}" /></div><div class="form-field"><label for="form-field-type-{}">Field Type</label><select class="input" id="form-field-type-{}">{}</select></div><div class="form-field"><label for="form-field-required-{}">Required</label><select class="input" id="form-field-required-{}"><option value="true" {}>Required</option><option value="false" {}>Optional</option></select></div><div class="form-field"><label for="form-field-position-{}">Display Order</label><input class="input" id="form-field-position-{}" type="number" value="{}" /></div><div class="form-field"><label for="form-field-section-{}">Section</label><select class="input" id="form-field-section-{}">{}</select></div></div><p class="muted">Current section: {}. Choice-source and lookup metadata remain read-only until backend metadata support lands.</p><div class="actions"><button class="button" type="button" data-form-field-save="{}">Save Field</button></div></aside>"#,
            escape_html(&field.label),
            escape_html(&field.id),
            escape_html(&field.id),
            escape_html(&field.label),
            escape_html(&field.id),
            escape_html(&field.id),
            escape_html(&field.key),
            escape_html(&field.id),
            escape_html(&field.id),
            render_form_field_type_options(&field.field_type),
            escape_html(&field.id),
            escape_html(&field.id),
            if field.required { "selected" } else { "" },
            if field.required { "" } else { "selected" },
            escape_html(&field.id),
            escape_html(&field.id),
            field.position,
            escape_html(&field.id),
            escape_html(&field.id),
            render_form_section_options(rendered, &section.id),
            escape_html(&section.title),
            escape_html(&field.id),
        )
    }

    fn render_editable_form_workspace(
        form: &FormDefinition,
        version: &FormVersionSummary,
        rendered: &RenderedForm,
        selected_section_id: Option<&str>,
        selected_field_id: Option<&str>,
    ) -> String {
        if version.status != "draft" {
            return format!(
                r#"<section class="detail-section box form-builder-readonly"><div class="page-title-row compact-title-row"><div><p class="form-builder-eyebrow">Read-only version</p><h3>{}</h3><p class="muted">This version is {} and is read-only.</p></div></div><p class="muted">Create a new draft version to change sections, fields, or ordering.</p>{}</section>"#,
                escape_html(&form_version_label(version)),
                escape_html(&version.status),
                render_form_preview(version, rendered),
            );
        }

        let resolved_field_id = resolve_selected_field_id(rendered, selected_field_id);
        let resolved_section_id = resolve_selected_section_id(
            rendered,
            selected_section_id,
            resolved_field_id.as_deref(),
        );
        let selected_section = resolved_section_id
            .as_deref()
            .and_then(|section_id| current_rendered_section(rendered, section_id));
        let properties_panel =
            render_builder_properties_panel(rendered, resolved_field_id.as_deref());
        let sections = if rendered.sections.is_empty() {
            r#"<section class="detail-section box form-builder-empty-state"><div class="page-title-row compact-title-row"><div><p class="form-builder-eyebrow">Canvas</p><h4>Start with a section</h4><p class="muted">This draft has no authored sections yet. Add a section first, then use the insert rail to create fields inside the selected section.</p></div></div><div class="actions"><button class="button" type="button" data-form-section-create="empty">Add First Section</button></div></section>"#
                .into()
        } else {
            rendered
                .sections
                .iter()
                .enumerate()
                .map(|(section_index, section)| {
                    let section_is_selected = Some(section.id.as_str())
                        == resolved_section_id.as_deref()
                        || section
                            .fields
                            .iter()
                            .any(|field| Some(field.id.as_str()) == resolved_field_id.as_deref());
                    let fields = if section.fields.is_empty() {
                        format!(
                            r#"<section class="form-builder-empty-fields"><p class="muted">No fields were added to this section yet. Select the section, then use the insert rail to add the first field.</p><div class="actions"><button class="button is-light" type="button" data-select-form-section="{}">Select Section</button></div></section>"#,
                            escape_html(&section.id),
                        )
                    } else {
                        section
                            .fields
                            .iter()
                            .enumerate()
                            .map(|(field_index, field)| {
                                let field_is_selected =
                                    Some(field.id.as_str()) == resolved_field_id.as_deref();
                                format!(
                                    r#"<article class="record-card form-builder-field-card{}"><div class="page-title-row compact-title-row"><div><p class="eyebrow">{}</p><button class="button button-link form-builder-field-title" type="button" data-select-form-field="{}">{}</button><p class="muted">Field {} · {}</p></div><div class="actions form-builder-field-actions"><button class="button is-light" type="button" data-form-field-move-up="{}">Up</button><button class="button is-light" type="button" data-form-field-move-down="{}">Down</button><button class="button is-light" type="button" data-form-field-delete="{}">Delete</button></div></div><p class="form-builder-field-key">Key: {}</p><p class="muted">{}</p></article>"#,
                                    if field_is_selected {
                                        " is-selected"
                                    } else {
                                        ""
                                    },
                                    escape_html(form_field_kind_label(&field.field_type)),
                                    escape_html(&field.id),
                                    escape_html(&field.label),
                                    field_index + 1,
                                    if field.required {
                                        "Required"
                                    } else {
                                        "Optional"
                                    },
                                    escape_html(&field.id),
                                    escape_html(&field.id),
                                    escape_html(&field.id),
                                    escape_html(&field.key),
                                    escape_html(form_field_preview_copy(&field.field_type)),
                                )
                            })
                            .collect::<Vec<_>>()
                            .join("")
                    };
                    format!(
                        r##"<section class="detail-section box form-builder-section{}" id="builder-section-{}"><div class="page-title-row compact-title-row"><div><p class="form-builder-eyebrow">Section {}</p><h3>{}</h3><p class="muted">Display order {} · {} · {} field(s)</p></div><div class="actions"><a class="button is-light" href="#builder-section-{}" data-select-form-section="{}">Focus</a><button class="button is-light" type="button" data-form-section-move-up="{}">Up</button><button class="button is-light" type="button" data-form-section-move-down="{}">Down</button><button class="button is-light" type="button" data-form-section-delete="{}">Delete</button><button class="button" type="button" data-form-section-save="{}">Save Section</button></div></div><div class="form-grid"><div class="form-field wide-field"><label for="form-section-title-{}">Section Title</label><input class="input" id="form-section-title-{}" type="text" value="{}" /></div><div class="form-field wide-field"><label for="form-section-description-{}">Section Description</label><textarea class="textarea" id="form-section-description-{}" rows="3">{}</textarea></div><div class="form-field"><label for="form-section-column-count-{}">Columns</label><select class="input" id="form-section-column-count-{}">{}</select></div><div class="form-field"><label for="form-section-position-{}">Display Order</label><input class="input" id="form-section-position-{}" type="number" value="{}" /></div></div><div class="form-builder-field-grid form-builder-field-grid--cols-{}">{}</div></section>"##,
                        if section_is_selected {
                            " is-selected"
                        } else {
                            ""
                        },
                        escape_html(&section.id),
                        section_index + 1,
                        escape_html(&section.title),
                        section.position,
                        escape_html(&form_section_layout_label(section.column_count)),
                        section.fields.len(),
                        escape_html(&section.id),
                        escape_html(&section.id),
                        escape_html(&section.id),
                        escape_html(&section.id),
                        escape_html(&section.id),
                        escape_html(&section.id),
                        escape_html(&section.id),
                        escape_html(&section.id),
                        escape_html(&section.title),
                        escape_html(&section.id),
                        escape_html(&section.id),
                        escape_html(&section.description),
                        escape_html(&section.id),
                        escape_html(&section.id),
                        render_form_section_column_count_options(section.column_count),
                        escape_html(&section.id),
                        escape_html(&section.id),
                        section.position,
                        section.column_count,
                        fields,
                    )
                })
                .collect::<Vec<_>>()
                .join("")
        };

        let layout_class = if properties_panel.is_empty() {
            "form-builder-layout"
        } else {
            "form-builder-layout form-builder-layout--with-properties"
        };
        let field_count = rendered
            .sections
            .iter()
            .map(|section| section.fields.len())
            .sum::<usize>();
        format!(
            r#"<section class="form-builder-shell"><section class="form-builder-lifecycle-strip"><div class="form-builder-lifecycle-strip__copy"><p class="form-builder-eyebrow">Lifecycle</p><h3>{}</h3><p class="muted">Draft workspace for {}. Publish and version status stay visible here while the canvas remains primary.</p></div><div class="form-builder-lifecycle-strip__meta"><span class="form-builder-lifecycle-chip">Draft</span><span class="form-builder-lifecycle-chip">Version line: {}</span><span class="form-builder-lifecycle-chip">Sections: {}</span><span class="form-builder-lifecycle-chip">Fields: {}</span></div><div class="form-builder-lifecycle-strip__actions"><div class="form-builder-lifecycle-strip__summary">{}</div><div class="actions"><button class="button" type="button" data-publish-form-version="{}">Publish Draft Version</button></div></div></section><div class="{}"><div class="form-builder-sidebar"><aside class="detail-section box form-builder-rail"><div class="page-title-row compact-title-row"><div><p class="form-builder-eyebrow">Sections</p><h4>Jump Between Sections</h4><p class="muted">Use the rail to switch context without losing the stacked canvas flow.</p></div></div><div class="form-builder-section-links">{}</div></aside>{}</div><div class="form-builder-main"><section class="detail-section box form-builder-canvas"><div class="page-title-row compact-title-row"><div><p class="form-builder-eyebrow">Section Stack</p><h4>Canvas</h4><p class="muted">Draft sections stay stacked in one authoring flow.</p></div></div>{}</section></div>{}</div></section>"#,
            escape_html(&form_version_label(version)),
            escape_html(&form.name),
            escape_html(&form_version_label(version)),
            rendered.sections.len(),
            field_count,
            render_form_version_lifecycle_summary(version),
            escape_html(&version.id),
            layout_class,
            render_builder_section_navigation(rendered, resolved_section_id.as_deref()),
            render_builder_insert_rail(selected_section.as_ref()),
            sections,
            properties_panel,
        )
    }

    fn attach_submit_handler(element_id: &str, handler: impl Fn() + 'static) {
        if let Some(element) = by_id(element_id) {
            let closure = Closure::wrap(Box::new(move |event: web_sys::Event| {
                event.prevent_default();
                handler();
            }) as Box<dyn FnMut(_)>);
            element
                .add_event_listener_with_callback("submit", closure.as_ref().unchecked_ref())
                .ok();
            closure.forget();
        }
    }

    fn attach_click_handler_by_attr(attr: &str, handler: impl Fn(String) + Clone + 'static) {
        if let Some(document) = web_sys::window().and_then(|window| window.document()) {
            let selector = format!("[{attr}]");
            if let Ok(nodes) = document.query_selector_all(&selector) {
                for index in 0..nodes.length() {
                    if let Some(node) = nodes.get(index) {
                        if let Ok(element) = node.dyn_into::<web_sys::Element>() {
                            let value = element.get_attribute(attr).unwrap_or_default();
                            let callback = handler.clone();
                            let closure = Closure::wrap(Box::new(move |_event: web_sys::Event| {
                                callback(value.clone());
                            })
                                as Box<dyn FnMut(_)>);
                            element
                                .add_event_listener_with_callback(
                                    "click",
                                    closure.as_ref().unchecked_ref(),
                                )
                                .ok();
                            closure.forget();
                        }
                    }
                }
            }
        }
    }

    fn next_section_position(rendered: &RenderedForm) -> i32 {
        rendered
            .sections
            .iter()
            .map(|section| section.position)
            .max()
            .unwrap_or(0)
            + 1
    }

    fn next_field_position(rendered: &RenderedForm, section_id: &str) -> i32 {
        rendered
            .sections
            .iter()
            .find(|section| section.id == section_id)
            .map(|section| {
                section
                    .fields
                    .iter()
                    .map(|field| field.position)
                    .max()
                    .unwrap_or(0)
                    + 1
            })
            .unwrap_or(1)
    }

    fn current_rendered_field(
        rendered: &RenderedForm,
        field_id: &str,
    ) -> Option<(RenderedField, RenderedSection)> {
        for section in &rendered.sections {
            if let Some(field) = section.fields.iter().find(|field| field.id == field_id) {
                return Some((field.clone(), section.clone()));
            }
        }
        None
    }

    fn current_rendered_section(
        rendered: &RenderedForm,
        section_id: &str,
    ) -> Option<RenderedSection> {
        rendered
            .sections
            .iter()
            .find(|section| section.id == section_id)
            .cloned()
    }

    fn resolve_selected_field_id(
        rendered: &RenderedForm,
        preferred_field_id: Option<&str>,
    ) -> Option<String> {
        preferred_field_id
            .filter(|field_id| current_rendered_field(rendered, field_id).is_some())
            .map(str::to_string)
    }

    fn resolve_selected_section_id(
        rendered: &RenderedForm,
        preferred_section_id: Option<&str>,
        preferred_field_id: Option<&str>,
    ) -> Option<String> {
        if let Some(field_id) = preferred_field_id {
            if let Some((_, section)) = current_rendered_field(rendered, field_id) {
                return Some(section.id);
            }
        }

        if let Some(section_id) = preferred_section_id {
            if current_rendered_section(rendered, section_id).is_some() {
                return Some(section_id.to_string());
            }
        }

        rendered.sections.first().map(|section| section.id.clone())
    }

    fn next_inserted_field_identity(rendered: &RenderedForm) -> (String, String) {
        let mut candidate = 1;
        loop {
            let key = format!("field_{candidate}");
            if rendered
                .sections
                .iter()
                .flat_map(|section| section.fields.iter())
                .all(|field| field.key != key)
            {
                let label = if candidate == 1 {
                    "Untitled Field".to_string()
                } else {
                    format!("Untitled Field {candidate}")
                };
                return (key, label);
            }
            candidate += 1;
        }
    }

    async fn load_scope_node_types() -> Result<Vec<NodeTypeSummary>, String> {
        get_json::<Vec<NodeTypeSummary>>("/api/admin/node-types").await
    }

    fn render_node_type_options(node_types: &[NodeTypeSummary]) -> String {
        options_html(
            node_types,
            |item| &item.id,
            |item| item.name.clone(),
            "No scope",
        )
    }

    pub fn load_list_page() {
        spawn_local(async move {
            match get_json::<Vec<FormSummary>>("/api/forms").await {
                Ok(forms) => set_html("form-list", &render_form_cards(&forms)),
                Err(error) => set_html(
                    "form-list",
                    &format!(r#"<p class="muted">{}</p>"#, escape_html(&error)),
                ),
            }
        });
    }

    pub fn load_detail_page(form_id: String) {
        spawn_local(async move {
            match get_json::<FormDefinition>(&format!("/api/forms/{form_id}")).await {
                Ok(form) => {
                    let published = form
                        .versions
                        .iter()
                        .rev()
                        .find(|version| version.status == "published")
                        .map(form_version_label)
                        .unwrap_or_else(|| "None".into());
                    let draft_count = form
                        .versions
                        .iter()
                        .filter(|version| version.status == "draft")
                        .count();
                    set_html(
                        "form-detail",
                        &format!(
                            r#"<article class="record-card form-detail-summary-card"><div class="page-title-row compact-title-row"><div><p class="eyebrow">Selected form</p><h3>{}</h3><p class="muted">/{}</p></div><div class="actions"><a class="button-link" href="/app/forms/{}/edit">Open Draft Authoring</a></div></div><div class="record-detail"><p class="muted">Scope root: {}</p><p class="muted">Published version: {}</p><p class="muted">Draft versions available: {}</p></div></article>"#,
                            escape_html(&form.name),
                            escape_html(&form.slug),
                            escape_html(&form.id),
                            escape_html(form.scope_node_type_name.as_deref().unwrap_or("Unscoped")),
                            escape_html(&published),
                            draft_count,
                        ),
                    );

                    let preferred = preferred_version(&form, None);
                    set_html(
                        "form-version-summary",
                        &render_version_summary(
                            &form.versions,
                            preferred.as_ref().map(|version| version.id.as_str()),
                        ),
                    );
                    set_html("form-workflow-links", &render_workflow_attachments(&form));

                    if let Some(version) = preferred {
                        match get_json::<RenderedForm>(&format!(
                            "/api/form-versions/{}/render",
                            version.id
                        ))
                        .await
                        {
                            Ok(rendered) => {
                                set_html(
                                    "form-version-preview",
                                    &render_form_preview(&version, &rendered),
                                );
                            }
                            Err(error) => set_html(
                                "form-version-preview",
                                &format!(r#"<p class="muted">{}</p>"#, escape_html(&error)),
                            ),
                        }
                    } else {
                        set_html(
                            "form-version-preview",
                            r#"<p class="muted">No form versions are available to preview.</p>"#,
                        );
                    }
                }
                Err(error) => {
                    let message = format!(r#"<p class="muted">{}</p>"#, escape_html(&error));
                    set_html("form-detail", &message);
                    set_html("form-version-summary", &message);
                    set_html("form-version-preview", &message);
                    set_html("form-workflow-links", &message);
                }
            }
        });
    }

    pub fn load_create_page() {
        spawn_local(async move {
            match load_scope_node_types().await {
                Ok(node_types) => {
                    set_html(
                        "form-scope-node-type",
                        &render_node_type_options(&node_types),
                    );
                }
                Err(error) => set_text("form-editor-status", &error),
            }
        });

        attach_submit_handler("form-entity-form", move || {
            spawn_local(async move {
                let scope_node_type_id = match select_value("form-scope-node-type") {
                    Some(value) if !value.is_empty() => Some(value),
                    _ => None,
                };
                let payload = json!({
                    "name": input_value("form-name").unwrap_or_default(),
                    "slug": input_value("form-slug").unwrap_or_default(),
                    "scope_node_type_id": scope_node_type_id,
                });
                match post_json::<IdResponse>("/api/admin/forms", &payload).await {
                    Ok(response) => redirect(&format!("/app/forms/{}/edit", response.id)),
                    Err(error) => set_text("form-editor-status", &error),
                }
            });
        });
    }

    fn wire_editable_version_actions(form_id: String, state: Rc<RefCell<FormBuilderState>>) {
        let preview_form_id = form_id.clone();
        let preview_state = state.clone();
        attach_click_handler_by_attr("data-preview-form-version", move |version_id| {
            let form_id = preview_form_id.clone();
            let state = preview_state.clone();
            spawn_local(async move {
                if let Err(error) = load_edit_surface(form_id, state, Some(version_id)).await {
                    set_text("form-version-status", &error);
                }
            });
        });

        let publish_form_id = form_id;
        let publish_state = state;
        attach_click_handler_by_attr("data-publish-form-version", move |version_id| {
            let form_id = publish_form_id.clone();
            let state = publish_state.clone();
            spawn_local(async move {
                let payload = json!({});
                match post_json::<PublishFormVersionResponse>(
                    &format!("/api/admin/form-versions/{version_id}/publish"),
                    &payload,
                )
                .await
                {
                    Ok(response) => {
                        let warning_suffix = if response.dependency_warnings.is_empty() {
                            String::new()
                        } else {
                            format!(
                                " {} direct dependency warning(s) need review.",
                                response.dependency_warnings.len()
                            )
                        };
                        set_text(
                            "form-version-status",
                            &format!(
                                "Draft version published as {}.{}",
                                response.version_label, warning_suffix
                            ),
                        );
                        if let Err(error) =
                            load_edit_surface(form_id, state, Some(version_id)).await
                        {
                            set_text("form-version-status", &error);
                        }
                    }
                    Err(error) => set_text("form-version-status", &error),
                }
            });
        });
    }

    fn rerender_workspace(
        form_id: String,
        state: Rc<RefCell<FormBuilderState>>,
    ) -> Result<(), String> {
        let (form, rendered, selected_version_id, selected_section_id, selected_field_id) = {
            let state_ref = state.borrow();
            (
                state_ref.form.clone(),
                state_ref.rendered_version.clone(),
                state_ref.selected_version_id.clone(),
                state_ref.selected_section_id.clone(),
                state_ref.selected_field_id.clone(),
            )
        };
        let Some(form) = form else {
            return Ok(());
        };
        let Some(rendered) = rendered else {
            return Ok(());
        };
        let Some(selected_version_id) = selected_version_id else {
            return Ok(());
        };
        let version = form
            .versions
            .iter()
            .find(|version| version.id == selected_version_id)
            .cloned()
            .ok_or_else(|| "The selected form version is no longer available.".to_string())?;

        set_html(
            "form-version-workspace",
            &render_editable_form_workspace(
                &form,
                &version,
                &rendered,
                selected_section_id.as_deref(),
                selected_field_id.as_deref(),
            ),
        );
        wire_workspace_actions(form_id, state);
        Ok(())
    }

    fn wire_workspace_actions(form_id: String, state: Rc<RefCell<FormBuilderState>>) {
        wire_editable_version_actions(form_id.clone(), state.clone());

        let create_section_form_id = form_id.clone();
        let create_section_state = state.clone();
        attach_click_handler_by_attr("data-form-section-create", move |_action| {
            let form_id = create_section_form_id.clone();
            let state = create_section_state.clone();
            spawn_local(async move {
                let selected_version_id = state.borrow().selected_version_id.clone();
                let rendered = state.borrow().rendered_version.clone();
                let Some(selected_version_id) = selected_version_id else {
                    set_text("form-version-status", "Select a draft version first.");
                    return;
                };
                let next_position = rendered.as_ref().map(next_section_position).unwrap_or(1);
                let payload = json!({
                    "title": format!("Section {next_position}"),
                    "description": "",
                    "column_count": 1,
                    "position": next_position,
                });
                match post_json::<IdResponse>(
                    &format!("/api/admin/form-versions/{selected_version_id}/sections"),
                    &payload,
                )
                .await
                {
                    Ok(response) => {
                        {
                            let mut state_mut = state.borrow_mut();
                            state_mut.selected_section_id = Some(response.id.clone());
                            state_mut.selected_field_id = None;
                        }
                        set_text("form-version-status", "Section created.");
                        if let Err(error) =
                            load_edit_surface(form_id, state, Some(selected_version_id)).await
                        {
                            set_text("form-version-status", &error);
                        }
                    }
                    Err(error) => set_text("form-version-status", &error),
                }
            });
        });

        let save_section_form_id = form_id.clone();
        let save_section_state = state.clone();
        attach_click_handler_by_attr("data-form-section-save", move |section_id| {
            let form_id = save_section_form_id.clone();
            let state = save_section_state.clone();
            spawn_local(async move {
                let selected_version_id = state.borrow().selected_version_id.clone();
                let payload = json!({
                    "title": input_value(&format!("form-section-title-{section_id}")).unwrap_or_default(),
                    "description": textarea_value(&format!("form-section-description-{section_id}")).unwrap_or_default(),
                    "column_count": select_value(&format!("form-section-column-count-{section_id}"))
                        .and_then(|value| value.parse::<i32>().ok())
                        .unwrap_or(1),
                    "position": input_value(&format!("form-section-position-{section_id}"))
                        .and_then(|value| value.parse::<i32>().ok())
                        .unwrap_or(0),
                });
                match put_json::<IdResponse>(
                    &format!("/api/admin/form-sections/{section_id}"),
                    &payload,
                )
                .await
                {
                    Ok(_) => {
                        {
                            let mut state_mut = state.borrow_mut();
                            state_mut.selected_section_id = Some(section_id.clone());
                            state_mut.selected_field_id = None;
                        }
                        set_text("form-version-status", "Section saved.");
                        if let Err(error) =
                            load_edit_surface(form_id, state, selected_version_id).await
                        {
                            set_text("form-version-status", &error);
                        }
                    }
                    Err(error) => set_text("form-version-status", &error),
                }
            });
        });

        let select_section_form_id = form_id.clone();
        let select_section_state = state.clone();
        attach_click_handler_by_attr("data-select-form-section", move |section_id| {
            let form_id = select_section_form_id.clone();
            let state = select_section_state.clone();
            {
                let mut state_mut = state.borrow_mut();
                state_mut.selected_section_id = Some(section_id);
                state_mut.selected_field_id = None;
            }
            if let Err(error) = rerender_workspace(form_id, state) {
                set_text("form-version-status", &error);
            }
        });

        let delete_section_form_id = form_id.clone();
        let delete_section_state = state.clone();
        attach_click_handler_by_attr("data-form-section-delete", move |section_id| {
            let form_id = delete_section_form_id.clone();
            let state = delete_section_state.clone();
            spawn_local(async move {
                let selected_version_id = state.borrow().selected_version_id.clone();
                match delete_json::<IdResponse>(&format!("/api/admin/form-sections/{section_id}"))
                    .await
                {
                    Ok(_) => {
                        {
                            let mut state_mut = state.borrow_mut();
                            if state_mut.selected_section_id.as_deref() == Some(section_id.as_str())
                            {
                                state_mut.selected_section_id = None;
                                state_mut.selected_field_id = None;
                            }
                        }
                        set_text("form-version-status", "Section deleted.");
                        if let Err(error) =
                            load_edit_surface(form_id, state, selected_version_id).await
                        {
                            set_text("form-version-status", &error);
                        }
                    }
                    Err(error) => set_text("form-version-status", &error),
                }
            });
        });

        for (attr, direction) in [
            ("data-form-section-move-up", -1),
            ("data-form-section-move-down", 1),
        ] {
            let move_section_form_id = form_id.clone();
            let move_section_state = state.clone();
            attach_click_handler_by_attr(attr, move |section_id| {
                let form_id = move_section_form_id.clone();
                let state = move_section_state.clone();
                spawn_local(async move {
                    let selected_version_id = state.borrow().selected_version_id.clone();
                    let rendered = state.borrow().rendered_version.clone();
                    let Some(rendered) = rendered else {
                        set_text("form-version-status", "Reload the form and try again.");
                        return;
                    };
                    let sections = {
                        let mut sections = rendered.sections.clone();
                        sections.sort_by_key(|section| section.position);
                        sections
                    };
                    let Some(current_index) =
                        sections.iter().position(|section| section.id == section_id)
                    else {
                        return;
                    };
                    let target_index = current_index as i32 + direction;
                    if target_index < 0 || target_index >= sections.len() as i32 {
                        return;
                    }
                    let current = &sections[current_index];
                    let target = &sections[target_index as usize];
                    let current_payload = json!({
                        "title": current.title,
                        "description": current.description,
                        "column_count": current.column_count,
                        "position": target.position,
                    });
                    let target_payload = json!({
                        "title": target.title,
                        "description": target.description,
                        "column_count": target.column_count,
                        "position": current.position,
                    });
                    let current_result = put_json::<IdResponse>(
                        &format!("/api/admin/form-sections/{}", current.id),
                        &current_payload,
                    )
                    .await;
                    let target_result = put_json::<IdResponse>(
                        &format!("/api/admin/form-sections/{}", target.id),
                        &target_payload,
                    )
                    .await;
                    match (current_result, target_result) {
                        (Ok(_), Ok(_)) => {
                            set_text("form-version-status", "Section order updated.");
                            if let Err(error) =
                                load_edit_surface(form_id, state, selected_version_id).await
                            {
                                set_text("form-version-status", &error);
                            }
                        }
                        (Err(error), _) | (_, Err(error)) => {
                            set_text("form-version-status", &error)
                        }
                    }
                });
            });
        }

        let create_field_form_id = form_id.clone();
        let create_field_state = state.clone();
        attach_click_handler_by_attr("data-form-field-create", move |section_id| {
            let form_id = create_field_form_id.clone();
            let state = create_field_state.clone();
            spawn_local(async move {
                let selected_version_id = state.borrow().selected_version_id.clone();
                let rendered = state.borrow().rendered_version.clone();
                let Some(selected_version_id) = selected_version_id else {
                    set_text("form-version-status", "Select a draft version first.");
                    return;
                };
                let Some(rendered) = rendered else {
                    set_text("form-version-status", "Reload the form and try again.");
                    return;
                };
                let (key, label) = next_inserted_field_identity(&rendered);
                let payload = json!({
                    "section_id": section_id,
                    "key": key,
                    "label": label,
                    "field_type": "text",
                    "required": false,
                    "position": next_field_position(&rendered, &section_id),
                });
                match post_json::<IdResponse>(
                    &format!("/api/admin/form-versions/{selected_version_id}/fields"),
                    &payload,
                )
                .await
                {
                    Ok(response) => {
                        {
                            let mut state_mut = state.borrow_mut();
                            state_mut.selected_section_id = Some(section_id.clone());
                            state_mut.selected_field_id = Some(response.id.clone());
                        }
                        set_text("form-version-status", "Field created.");
                        if let Err(error) =
                            load_edit_surface(form_id, state, Some(selected_version_id)).await
                        {
                            set_text("form-version-status", &error);
                        }
                    }
                    Err(error) => set_text("form-version-status", &error),
                }
            });
        });

        let save_field_form_id = form_id.clone();
        let save_field_state = state.clone();
        attach_click_handler_by_attr("data-form-field-save", move |field_id| {
            let form_id = save_field_form_id.clone();
            let state = save_field_state.clone();
            spawn_local(async move {
                let selected_version_id = state.borrow().selected_version_id.clone();
                let payload = json!({
                    "section_id": select_value(&format!("form-field-section-{field_id}")).unwrap_or_default(),
                    "key": input_value(&format!("form-field-key-{field_id}")).unwrap_or_default(),
                    "label": input_value(&format!("form-field-label-{field_id}")).unwrap_or_default(),
                    "field_type": select_value(&format!("form-field-type-{field_id}")).unwrap_or_else(|| "text".into()),
                    "required": select_value(&format!("form-field-required-{field_id}")).is_some_and(|value| value == "true"),
                    "position": input_value(&format!("form-field-position-{field_id}"))
                        .and_then(|value| value.parse::<i32>().ok())
                        .unwrap_or(0),
                });
                match put_json::<IdResponse>(
                    &format!("/api/admin/form-fields/{field_id}"),
                    &payload,
                )
                .await
                {
                    Ok(_) => {
                        {
                            let mut state_mut = state.borrow_mut();
                            state_mut.selected_field_id = Some(field_id.clone());
                        }
                        set_text("form-version-status", "Field saved.");
                        if let Err(error) =
                            load_edit_surface(form_id, state, selected_version_id).await
                        {
                            set_text("form-version-status", &error);
                        }
                    }
                    Err(error) => set_text("form-version-status", &error),
                }
            });
        });

        let select_field_form_id = form_id.clone();
        let select_field_state = state.clone();
        attach_click_handler_by_attr("data-select-form-field", move |field_id| {
            let form_id = select_field_form_id.clone();
            let state = select_field_state.clone();
            let selected_section_id = state
                .borrow()
                .rendered_version
                .as_ref()
                .and_then(|rendered| current_rendered_field(rendered, &field_id))
                .map(|(_, section)| section.id);
            {
                let mut state_mut = state.borrow_mut();
                state_mut.selected_field_id = Some(field_id);
                state_mut.selected_section_id = selected_section_id;
            }
            if let Err(error) = rerender_workspace(form_id, state) {
                set_text("form-version-status", &error);
            }
        });

        let clear_field_form_id = form_id.clone();
        let clear_field_state = state.clone();
        attach_click_handler_by_attr("data-clear-form-field-selection", move |_value| {
            let form_id = clear_field_form_id.clone();
            let state = clear_field_state.clone();
            state.borrow_mut().selected_field_id = None;
            if let Err(error) = rerender_workspace(form_id, state) {
                set_text("form-version-status", &error);
            }
        });

        let delete_field_form_id = form_id.clone();
        let delete_field_state = state.clone();
        attach_click_handler_by_attr("data-form-field-delete", move |field_id| {
            let form_id = delete_field_form_id.clone();
            let state = delete_field_state.clone();
            spawn_local(async move {
                let selected_version_id = state.borrow().selected_version_id.clone();
                let selected_section_id = state
                    .borrow()
                    .rendered_version
                    .as_ref()
                    .and_then(|rendered| current_rendered_field(rendered, &field_id))
                    .map(|(_, section)| section.id);
                match delete_json::<IdResponse>(&format!("/api/admin/form-fields/{field_id}")).await
                {
                    Ok(_) => {
                        {
                            let mut state_mut = state.borrow_mut();
                            state_mut.selected_field_id = None;
                            state_mut.selected_section_id = selected_section_id;
                        }
                        set_text("form-version-status", "Field deleted.");
                        if let Err(error) =
                            load_edit_surface(form_id, state, selected_version_id).await
                        {
                            set_text("form-version-status", &error);
                        }
                    }
                    Err(error) => set_text("form-version-status", &error),
                }
            });
        });

        for (attr, direction) in [
            ("data-form-field-move-up", -1),
            ("data-form-field-move-down", 1),
        ] {
            let move_field_form_id = form_id.clone();
            let move_field_state = state.clone();
            attach_click_handler_by_attr(attr, move |field_id| {
                let form_id = move_field_form_id.clone();
                let state = move_field_state.clone();
                spawn_local(async move {
                    let selected_version_id = state.borrow().selected_version_id.clone();
                    let rendered = state.borrow().rendered_version.clone();
                    let Some(rendered) = rendered else {
                        set_text("form-version-status", "Reload the form and try again.");
                        return;
                    };
                    let Some((current, section)) = current_rendered_field(&rendered, &field_id)
                    else {
                        set_text(
                            "form-version-status",
                            "The selected field is no longer available. Reload the page and try again.",
                        );
                        return;
                    };
                    let fields = {
                        let mut fields = section.fields.clone();
                        fields.sort_by_key(|field| field.position);
                        fields
                    };
                    let Some(current_index) = fields.iter().position(|field| field.id == field_id)
                    else {
                        return;
                    };
                    let target_index = current_index as i32 + direction;
                    if target_index < 0 || target_index >= fields.len() as i32 {
                        return;
                    }
                    let target = &fields[target_index as usize];
                    let current_payload = json!({
                        "section_id": section.id,
                        "key": current.key,
                        "label": current.label,
                        "field_type": current.field_type,
                        "required": current.required,
                        "position": target.position,
                    });
                    let target_payload = json!({
                        "section_id": section.id,
                        "key": target.key,
                        "label": target.label,
                        "field_type": target.field_type,
                        "required": target.required,
                        "position": current.position,
                    });
                    let current_result = put_json::<IdResponse>(
                        &format!("/api/admin/form-fields/{}", current.id),
                        &current_payload,
                    )
                    .await;
                    let target_result = put_json::<IdResponse>(
                        &format!("/api/admin/form-fields/{}", target.id),
                        &target_payload,
                    )
                    .await;
                    match (current_result, target_result) {
                        (Ok(_), Ok(_)) => {
                            state.borrow_mut().selected_field_id = Some(field_id.clone());
                            set_text("form-version-status", "Field order updated.");
                            if let Err(error) =
                                load_edit_surface(form_id, state, selected_version_id).await
                            {
                                set_text("form-version-status", &error);
                            }
                        }
                        (Err(error), _) | (_, Err(error)) => {
                            set_text("form-version-status", &error)
                        }
                    }
                });
            });
        }
    }

    async fn load_edit_surface(
        form_id: String,
        state: Rc<RefCell<FormBuilderState>>,
        preferred_version_id: Option<String>,
    ) -> Result<(), String> {
        let node_types = load_scope_node_types().await?;
        let form = get_json::<FormDefinition>(&format!("/api/admin/forms/{form_id}")).await?;

        set_html(
            "form-scope-node-type",
            &render_node_type_options(&node_types),
        );
        set_input_value("form-name", &form.name);
        set_input_value("form-slug", &form.slug);
        set_select_value(
            "form-scope-node-type",
            form.scope_node_type_id.as_deref().unwrap_or(""),
        );

        let selected_version = preferred_version(&form, preferred_version_id.as_deref());
        set_html(
            "form-version-list",
            &render_form_version_cards(
                &form,
                selected_version.as_ref().map(|version| version.id.as_str()),
                true,
            ),
        );
        set_html(
            "form-version-create-form",
            &render_form_version_create_panel(&form),
        );

        {
            let mut state_mut = state.borrow_mut();
            state_mut.form = Some(form.clone());
            state_mut.selected_version_id =
                selected_version.as_ref().map(|version| version.id.clone());
            state_mut.rendered_version = None;
        }

        if let Some(version) = selected_version {
            let (preferred_section_id, preferred_field_id) = {
                let state_ref = state.borrow();
                (
                    state_ref.selected_section_id.clone(),
                    state_ref.selected_field_id.clone(),
                )
            };
            let rendered =
                get_json::<RenderedForm>(&format!("/api/form-versions/{}/render", version.id))
                    .await?;
            let resolved_field_id =
                resolve_selected_field_id(&rendered, preferred_field_id.as_deref());
            let resolved_section_id = resolve_selected_section_id(
                &rendered,
                preferred_section_id.as_deref(),
                resolved_field_id.as_deref(),
            );
            set_html(
                "form-version-workspace",
                &render_editable_form_workspace(
                    &form,
                    &version,
                    &rendered,
                    resolved_section_id.as_deref(),
                    resolved_field_id.as_deref(),
                ),
            );
            {
                let mut state_mut = state.borrow_mut();
                state_mut.selected_section_id = resolved_section_id;
                state_mut.selected_field_id = resolved_field_id;
                state_mut.rendered_version = Some(rendered);
            }
            wire_workspace_actions(form_id, state);
        } else {
            {
                let mut state_mut = state.borrow_mut();
                state_mut.selected_section_id = None;
                state_mut.selected_field_id = None;
            }
            set_html(
                "form-version-workspace",
                r#"<p class="muted">Create a draft version to start authoring sections and fields.</p>"#,
            );
            wire_editable_version_actions(form_id, state);
        }

        Ok(())
    }

    pub fn load_edit_page(form_id: String) {
        let state = Rc::new(RefCell::new(FormBuilderState::default()));
        let form_id_for_surface = form_id.clone();
        let state_for_surface = state.clone();
        spawn_local(async move {
            if let Err(error) =
                load_edit_surface(form_id_for_surface, state_for_surface, None).await
            {
                set_text("form-editor-status", &error);
                set_text("form-version-status", &error);
            }
        });

        let submit_form_id = form_id.clone();
        let submit_state = state.clone();
        attach_submit_handler("form-entity-form", move || {
            let form_id = submit_form_id.clone();
            let state = submit_state.clone();
            spawn_local(async move {
                let scope_node_type_id = match select_value("form-scope-node-type") {
                    Some(value) if !value.is_empty() => Some(value),
                    _ => None,
                };
                let payload = json!({
                    "name": input_value("form-name").unwrap_or_default(),
                    "slug": input_value("form-slug").unwrap_or_default(),
                    "scope_node_type_id": scope_node_type_id,
                });
                match put_json::<IdResponse>(&format!("/api/admin/forms/{form_id}"), &payload).await
                {
                    Ok(_) => {
                        set_text("form-editor-status", "Form metadata saved.");
                        if let Err(error) = load_edit_surface(form_id, state, None).await {
                            set_text("form-editor-status", &error);
                        }
                    }
                    Err(error) => set_text("form-editor-status", &error),
                }
            });
        });

        if let Some(element) = by_id("form-version-create-form") {
            let create_version_form_id = form_id.clone();
            let create_version_state = state.clone();
            let closure = Closure::wrap(Box::new(move |event: web_sys::Event| {
                event.prevent_default();
                let form_id = create_version_form_id.clone();
                let state = create_version_state.clone();
                spawn_local(async move {
                    let payload = json!({});
                    match post_json::<IdResponse>(
                        &format!("/api/admin/forms/{form_id}/versions"),
                        &payload,
                    )
                    .await
                    {
                        Ok(response) => {
                            set_text("form-version-status", "Draft version created.");
                            if let Err(error) =
                                load_edit_surface(form_id, state, Some(response.id)).await
                            {
                                set_text("form-version-status", &error);
                            }
                        }
                        Err(error) => set_text("form-version-status", &error),
                    }
                });
            }) as Box<dyn FnMut(_)>);
            element
                .add_event_listener_with_callback("submit", closure.as_ref().unchecked_ref())
                .ok();
            closure.forget();
        }
    }

    pub fn set_context(page_key: &'static str, record_id: Option<String>) {
        set_page_context(page_key, "forms", record_id);
    }
}

#[component]
pub fn FormsListPage() -> impl IntoView {
    let session = use_account_session();
    #[cfg(not(feature = "hydrate"))]
    let _ = session;
    #[cfg(feature = "hydrate")]
    Effect::new(move |_| {
        if session.loaded.get() && session.account.get().is_some() {
            hydrate::set_context("form-list", None);
            hydrate::load_list_page();
        }
    });

    view! {
        <NativePage
            title="Forms"
            description="Tessara forms list screen."
            page_key="form-list"
            active_route="forms"
            workspace_label="Product Area"
            required_capability="forms:read"
            breadcrumbs=vec![
                BreadcrumbItem::link("Home", "/app"),
                BreadcrumbItem::current("Forms"),
            ]
        >
            <section class="forms-route forms-catalog-screen">
                <header class="detail-section box forms-screen-header">
                    <div class="page-title-row">
                        <div>
                            <p class="eyebrow">"Forms"</p>
                            <h1>"Form Directory"</h1>
                            <p class="muted">
                                "Browse forms by lifecycle state, then move directly into detail or draft authoring without a metadata-first detour."
                            </p>
                        </div>
                        <div class="actions">
                            <a class="button-link button is-primary" href="/app/forms/new">"Create Form"</a>
                        </div>
                    </div>
                    <div class="record-detail">
                        <p class="muted">
                            "Use this screen as the directory. Each record should make scope, published line, and available draft work visible before you drill in."
                        </p>
                    </div>
                </header>
                <section class="detail-section box forms-directory-shell">
                    <div class="page-title-row compact-title-row">
                        <div>
                            <p class="eyebrow">"Directory"</p>
                            <h3>"Current Form Set"</h3>
                            <p class="muted">
                                "Current form records and their version posture appear here."
                            </p>
                        </div>
                    </div>
                    <div id="form-list" class="record-list forms-directory-list">
                        <p class="muted">"Loading form records..."</p>
                    </div>
                </section>
            </section>
        </NativePage>
    }
}

#[component]
pub fn FormCreatePage() -> impl IntoView {
    let session = use_account_session();
    #[cfg(not(feature = "hydrate"))]
    let _ = session;
    #[cfg(feature = "hydrate")]
    Effect::new(move |_| {
        if session.loaded.get() && session.account.get().is_some() {
            hydrate::set_context("form-create", None);
            hydrate::load_create_page();
        }
    });

    view! {
        <NativePage
            title="Create Form"
            description="Create a Tessara form."
            page_key="form-create"
            active_route="forms"
            workspace_label="Product Area"
            required_capability="forms:write"
            breadcrumbs=vec![
                BreadcrumbItem::link("Home", "/app"),
                BreadcrumbItem::link("Forms", "/app/forms"),
                BreadcrumbItem::current("Create Form"),
            ]
        >
            <PageHeader
                eyebrow="Forms"
                title="Create Form"
                description="Create a top-level form and continue directly into draft version authoring."
            />
            <MetadataStrip items=vec![
                ("Mode", "Create".into()),
                ("Surface", "Form authoring".into()),
                ("State", "Metadata entry".into()),
            ]/>
            <Panel
                title="Form Metadata"
                description="Complete the fields below to create a top-level form. Version authoring opens after the form is saved."
            >
                <p id="form-editor-status" class="muted">
                    "Create the form record first. Version authoring opens after the form is saved."
                </p>
                <form id="form-entity-form" class="entity-form">
                    <div class="form-grid">
                        <div class="form-field wide-field">
                            <label for="form-name">"Name"</label>
                            <input class="input" id="form-name" type="text" autocomplete="off" />
                        </div>
                        <div class="form-field">
                            <label for="form-slug">"Slug"</label>
                            <input class="input" id="form-slug" type="text" autocomplete="off" />
                        </div>
                        <div class="form-field">
                            <label for="form-scope-node-type">"Scope Node Type"</label>
                            <select class="input" id="form-scope-node-type"></select>
                        </div>
                    </div>
                    <div class="actions">
                        <button class="button is-primary" type="submit">"Create Form"</button>
                        <a class="button-link button is-light" href="/app/forms">"Cancel"</a>
                    </div>
                </form>
            </Panel>
            <Panel
                title="Authoring Flow"
                description="After save, this route continues directly into draft version lifecycle, section authoring, and field authoring without returning to the transitional shell."
            >
                <div class="record-list">
                    <article class="record-card compact-record-card">
                        <h4>"Next Step"</h4>
                        <p class="muted">
                            "Create first, then author versions, sections, and fields from the dedicated edit surface."
                        </p>
                    </article>
                </div>
            </Panel>
        </NativePage>
    }
}

#[component]
pub fn FormDetailPage() -> impl IntoView {
    let FormRouteParams { form_id } = require_route_params();
    let _form_id_for_load = form_id.clone();
    let session = use_account_session();
    #[cfg(not(feature = "hydrate"))]
    let _ = session;
    #[cfg(feature = "hydrate")]
    Effect::new(move |_| {
        if session.loaded.get() && session.account.get().is_some() {
            hydrate::set_context("form-detail", Some(_form_id_for_load.clone()));
            hydrate::load_detail_page(_form_id_for_load.clone());
        }
    });

    view! {
        <NativePage
            title="Form Detail"
            description="Inspect a Tessara form."
            page_key="form-detail"
            active_route="forms"
            workspace_label="Product Area"
            record_id=form_id.clone()
            required_capability="forms:read"
            breadcrumbs=vec![
                BreadcrumbItem::link("Home", "/app"),
                BreadcrumbItem::link("Forms", "/app/forms"),
                BreadcrumbItem::current("Form Detail"),
            ]
        >
            <section class="forms-route form-detail-screen">
                <header class="detail-section box forms-screen-header">
                    <div class="page-title-row">
                        <div>
                            <p class="eyebrow">"Forms"</p>
                            <h1>"Form Detail"</h1>
                            <p class="muted">
                                "Review the selected form, its active version history, and the downstream work attached to it."
                            </p>
                        </div>
                        <div class="actions">
                            <a class="button-link button is-light" href="/app/forms">"Back To Directory"</a>
                            <a class="button-link button is-primary" href=format!("/app/forms/{form_id}/edit")>
                                "Open Draft Authoring"
                            </a>
                        </div>
                    </div>
                </header>
                <section class="form-detail-layout">
                    <section class="detail-section box form-detail-primary">
                        <div class="page-title-row compact-title-row">
                            <div>
                                <p class="eyebrow">"Selected Form"</p>
                                <h3>"Summary And Versions"</h3>
                                <p class="muted">
                                    "Keep the form summary and lifecycle together before branching into preview and related work."
                                </p>
                            </div>
                        </div>
                        <div id="form-detail" class="record-detail form-detail-summary">
                            <p class="muted">"Loading form summary..."</p>
                        </div>
                        <div id="form-version-summary" class="record-list form-detail-version-list">
                            <p class="muted">"Loading version summary..."</p>
                        </div>
                    </section>
                    <aside class="form-detail-secondary">
                        <section class="detail-section box form-detail-preview-shell">
                            <div class="page-title-row compact-title-row">
                                <div>
                                    <p class="eyebrow">"Preview"</p>
                                    <h3>"Section Stack"</h3>
                                    <p class="muted">
                                        "Preview the preferred version's sections and fields without switching routes."
                                    </p>
                                </div>
                            </div>
                            <div id="form-version-preview" class="record-detail form-detail-preview">
                                <p class="muted">"Loading section preview..."</p>
                            </div>
                        </section>
                        <section class="detail-section box form-detail-related-shell">
                            <div class="page-title-row compact-title-row">
                                <div>
                                    <p class="eyebrow">"Attachments"</p>
                                    <h3>"Related Work"</h3>
                                    <p class="muted">
                                        "Workflow, report, and dataset links stay visible from the form detail route."
                                    </p>
                                </div>
                            </div>
                            <div id="form-workflow-links" class="record-detail form-detail-related-work">
                                <p class="muted">"Loading workflow attachments..."</p>
                            </div>
                        </section>
                    </aside>
                </section>
            </section>
        </NativePage>
    }
}

#[component]
pub fn FormEditPage() -> impl IntoView {
    let FormRouteParams { form_id } = require_route_params();
    let _form_id_for_load = form_id.clone();
    let session = use_account_session();
    #[cfg(not(feature = "hydrate"))]
    let _ = session;
    #[cfg(feature = "hydrate")]
    Effect::new(move |_| {
        if session.loaded.get() && session.account.get().is_some() {
            hydrate::set_context("form-edit", Some(_form_id_for_load.clone()));
            hydrate::load_edit_page(_form_id_for_load.clone());
        }
    });

    view! {
        <NativePage
            title="Draft Authoring"
            description="Edit a Tessara form."
            page_key="form-edit"
            active_route="forms"
            workspace_label="Product Area"
            record_id=form_id.clone()
            required_capability="forms:write"
            breadcrumbs=vec![
                BreadcrumbItem::link("Home", "/app"),
                BreadcrumbItem::link("Forms", "/app/forms"),
                BreadcrumbItem::link("Form Detail", format!("/app/forms/{form_id}")),
                BreadcrumbItem::current("Edit Form"),
            ]
        >
            <section class="forms-route form-edit-screen">
                <section class="form-edit-layout">
                    <section class="form-edit-primary">
                        <div class="form-edit-toolbar">
                            <div class="form-edit-toolbar__copy">
                                <p class="eyebrow">"Forms"</p>
                                <h1>"Draft Authoring"</h1>
                                <p class="muted">
                                    "Keep version choice nearby, but let the builder canvas stay visually primary."
                                </p>
                            </div>
                            <div class="actions">
                                <a class="button-link button is-light" href=format!("/app/forms/{form_id}")>
                                    "Back To Detail"
                                </a>
                                <a class="button-link button is-light" href="/app/forms">
                                    "All Forms"
                                </a>
                            </div>
                        </div>
                        <section class="detail-section box form-edit-workspace-shell">
                            <div class="page-title-row compact-title-row">
                                <div>
                                    <p class="eyebrow">"Builder"</p>
                                    <h3>"Draft Workspace"</h3>
                                    <p class="muted">
                                        "The authoring canvas stays primary. Lifecycle and metadata remain available, but secondary."
                                    </p>
                                </div>
                            </div>
                            <div id="form-version-workspace" class="record-detail form-edit-workspace">
                                <p class="muted">"Loading draft workspace..."</p>
                            </div>
                        </section>
                    </section>
                    <aside class="form-edit-secondary">
                        <section class="detail-section box form-edit-lifecycle-shell">
                            <div class="page-title-row compact-title-row">
                                <div>
                                    <p class="eyebrow">"Lifecycle"</p>
                                    <h3>"Draft Versions"</h3>
                                    <p class="muted">
                                        "Keep version choice available without taking over the route before authoring begins."
                                    </p>
                                </div>
                            </div>
                            <p id="form-version-status" class="muted">
                                "Select or create a version to start authoring."
                            </p>
                            <form id="form-version-create-form" class="entity-form form-edit-version-create">
                                <p class="muted">
                                    "Draft versions stay unlabeled until publish. Semantic version and major-line compatibility are assigned automatically when you publish."
                                </p>
                                <div class="actions">
                                    <button class="button is-primary" type="submit">"Create Draft Version"</button>
                                </div>
                            </form>
                            <div id="form-version-list" class="record-list form-edit-version-list">
                                <p class="muted">"Loading form versions..."</p>
                            </div>
                        </section>
                        <section class="detail-section box form-edit-metadata-shell">
                            <div class="page-title-row compact-title-row">
                                <div>
                                    <p class="eyebrow">"Form Record"</p>
                                    <h3>"Metadata"</h3>
                                    <p class="muted">
                                        "Top-level form metadata stays editable here without taking over the authoring surface."
                                    </p>
                                </div>
                            </div>
                            <p id="form-editor-status" class="muted">"Loading form metadata..."</p>
                            <form id="form-entity-form" class="entity-form">
                                <div class="form-grid">
                                    <div class="form-field wide-field">
                                        <label for="form-name">"Name"</label>
                                        <input class="input" id="form-name" type="text" autocomplete="off" />
                                    </div>
                                    <div class="form-field">
                                        <label for="form-slug">"Slug"</label>
                                        <input class="input" id="form-slug" type="text" autocomplete="off" />
                                    </div>
                                    <div class="form-field">
                                        <label for="form-scope-node-type">"Scope Node Type"</label>
                                        <select class="input" id="form-scope-node-type"></select>
                                    </div>
                                </div>
                                <div class="actions">
                                    <button class="button is-primary" type="submit">"Save Form"</button>
                                    <a class="button-link button is-light" href=format!("/app/forms/{form_id}")>
                                        "Cancel"
                                    </a>
                                </div>
                            </form>
                        </section>
                    </aside>
                </section>
            </section>
        </NativePage>
    }
}
