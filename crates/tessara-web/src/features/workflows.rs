use leptos::prelude::*;

use crate::features::native_shell::{
    BreadcrumbItem, MetadataStrip, NativePage, PageHeader, Panel, use_account_session,
};
use crate::infra::routing::{WorkflowRouteParams, require_route_params};

#[cfg(feature = "hydrate")]
mod hydrate {
    use crate::features::native_runtime::{
        by_id, current_search_param, escape_html, get_json, input_value, post_json, put_json,
        redirect, select_value, set_html, set_input_value, set_page_context, set_select_value,
        set_text, set_textarea_value, textarea_value,
    };
    use serde::Deserialize;
    use serde_json::json;
    use wasm_bindgen::{JsCast, JsValue, closure::Closure};
    use wasm_bindgen_futures::spawn_local;

    #[derive(Clone, Deserialize)]
    struct WorkflowSummary {
        id: String,
        form_name: String,
        name: String,
        slug: String,
        description: String,
        current_version_label: Option<String>,
        current_status: Option<String>,
        assignment_count: i64,
    }

    #[derive(Clone, Deserialize)]
    struct WorkflowVersionSummary {
        id: String,
        form_version_id: String,
        form_version_label: Option<String>,
        title: String,
        status: String,
    }

    #[derive(Clone, Deserialize)]
    struct WorkflowAssignmentSummary {
        id: String,
        node_name: String,
        account_display_name: String,
        account_email: String,
        is_active: bool,
        has_draft: bool,
        has_submitted: bool,
    }

    #[derive(Clone, Deserialize)]
    struct WorkflowDefinition {
        id: String,
        form_id: String,
        form_name: String,
        form_slug: String,
        name: String,
        slug: String,
        description: String,
        versions: Vec<WorkflowVersionSummary>,
        assignments: Vec<WorkflowAssignmentSummary>,
    }

    #[derive(Clone, Deserialize)]
    struct FormSummary {
        id: String,
        name: String,
        slug: String,
        versions: Vec<FormVersionLite>,
    }

    #[derive(Clone, Deserialize)]
    struct FormVersionLite {
        id: String,
        version_label: Option<String>,
        status: String,
    }

    #[derive(Clone, Deserialize)]
    struct NodeSummary {
        id: String,
        name: String,
    }

    #[derive(Clone, Deserialize)]
    struct UserSummary {
        id: String,
        display_name: String,
        email: String,
    }

    #[derive(Clone, Deserialize)]
    struct AssignmentSummary {
        id: String,
        workflow_id: String,
        workflow_name: String,
        node_id: String,
        node_name: String,
        account_id: String,
        account_display_name: String,
        account_email: String,
        is_active: bool,
        has_draft: bool,
        has_submitted: bool,
    }

    #[derive(Clone, Deserialize)]
    struct IdResponse {
        id: String,
    }

    fn options_html<T>(
        items: &[T],
        value: impl Fn(&T) -> &str,
        label: impl Fn(&T) -> String,
        placeholder: &str,
    ) -> String {
        options_html_selected(items, value, label, placeholder, None)
    }

    fn options_html_selected<T>(
        items: &[T],
        value: impl Fn(&T) -> &str,
        label: impl Fn(&T) -> String,
        placeholder: &str,
        selected: Option<&str>,
    ) -> String {
        let mut html = format!(r#"<option value="">{}</option>"#, escape_html(placeholder));
        for item in items {
            let option_value = value(item);
            html.push_str(&format!(
                r#"<option value="{}"{}>{}</option>"#,
                escape_html(option_value),
                if selected == Some(option_value) {
                    " selected"
                } else {
                    ""
                },
                escape_html(&label(item))
            ));
        }
        html
    }

    fn workflow_summary_description(summary: &WorkflowSummary) -> &str {
        if summary.description.trim().is_empty() {
            "This workflow is ready to link form versions into assignment-backed response work."
        } else {
            summary.description.as_str()
        }
    }

    fn workflow_summary_status(summary: &WorkflowSummary) -> &str {
        summary.current_status.as_deref().unwrap_or("Draft only")
    }

    fn workflow_summary_version(summary: &WorkflowSummary) -> &str {
        summary
            .current_version_label
            .as_deref()
            .unwrap_or("Not published")
    }

    fn workflow_list_path(selected_workflow_id: Option<&str>) -> String {
        selected_workflow_id
            .filter(|value| !value.is_empty())
            .map(|value| format!("/app/workflows?workflowId={value}"))
            .unwrap_or_else(|| "/app/workflows".into())
    }

    fn replace_workflow_list_location(selected_workflow_id: Option<&str>) {
        let Some(window) = web_sys::window() else {
            return;
        };
        let Ok(history) = window.history() else {
            return;
        };
        let path = workflow_list_path(selected_workflow_id);
        let _ = history.replace_state_with_url(&JsValue::NULL, "", Some(&path));
    }

    fn select_directory_workflow_id(
        items: &[WorkflowSummary],
        requested_workflow_id: Option<&str>,
    ) -> Option<String> {
        requested_workflow_id
            .filter(|requested| items.iter().any(|item| item.id == *requested))
            .map(str::to_owned)
            .or_else(|| items.first().map(|item| item.id.clone()))
    }

    fn render_workflow_directory_metrics(items: &[WorkflowSummary]) -> String {
        if items.is_empty() {
            return r#"<p class="muted">No workflows are available yet.</p>"#.into();
        }

        let draft_only = items
            .iter()
            .filter(|item| workflow_summary_status(item).eq_ignore_ascii_case("draft only"))
            .count();
        let assignment_total: i64 = items.iter().map(|item| item.assignment_count).sum();
        let published = items
            .iter()
            .filter(|item| !workflow_summary_version(item).eq_ignore_ascii_case("not published"))
            .count();

        format!(
            r#"<div class="binding-row workflow-directory-overview__metrics"><p><strong>{}</strong> workflows in the directory.</p><p><strong>{}</strong> currently have a published version.</p><p><strong>{}</strong> are draft-only and still need a publishable runtime version.</p><p><strong>{}</strong> assignment-backed work items are attached across the directory.</p></div>"#,
            items.len(),
            published,
            draft_only,
            assignment_total
        )
    }

    fn render_workflow_directory(
        items: &[WorkflowSummary],
        selected_workflow_id: Option<&str>,
    ) -> String {
        if items.is_empty() {
            return r#"<p class="muted">No workflow records found.</p>"#.into();
        }
        items.iter()
            .map(|item| {
                let is_selected = selected_workflow_id == Some(item.id.as_str());
                format!(
                    r#"<button class="record-card compact-record-card workflow-directory-card workflow-directory-row{}" type="button" data-workflow-directory-select="{}" aria-pressed="{}"><div class="page-title-row compact-title-row"><div><p class="eyebrow">Workflow</p><h4>{}</h4><p class="muted">{}</p></div><p class="workflow-directory-card__status">{}</p></div><div class="workflow-directory-card__meta workflow-directory-row__meta"><p><strong>Linked Form:</strong> {}</p><p><strong>Current Version:</strong> {}</p><p><strong>Assignments:</strong> {}</p><p><strong>Slug:</strong> {}</p></div></button>"#,
                    if is_selected {
                        " workflow-directory-row--selected"
                    } else {
                        ""
                    },
                    escape_html(&item.id),
                    if is_selected { "true" } else { "false" },
                    escape_html(&item.name),
                    escape_html(workflow_summary_description(item)),
                    escape_html(workflow_summary_status(item)),
                    escape_html(&item.form_name),
                    escape_html(workflow_summary_version(item)),
                    item.assignment_count,
                    escape_html(&item.slug),
                )
            })
            .collect::<Vec<_>>()
            .join("")
    }

    fn assignment_console_path(workflow_id: Option<&str>) -> String {
        workflow_id
            .filter(|value| !value.is_empty())
            .map(|value| format!("/app/workflows/assignments?workflowId={value}"))
            .unwrap_or_else(|| "/app/workflows/assignments".into())
    }

    fn render_workflow_detail(detail: &WorkflowDefinition) -> String {
        format!(
            r#"<dl class="metadata-list"><div><dt>Name</dt><dd>{}</dd></div><div><dt>Slug</dt><dd>{}</dd></div><div><dt>Linked Form</dt><dd><a href="/app/forms/{}">{}</a></dd></div><div><dt>Assignment Count</dt><dd>{}</dd></div></dl><p>{}</p>"#,
            escape_html(&detail.name),
            escape_html(&detail.slug),
            escape_html(&detail.form_id),
            escape_html(&detail.form_name),
            detail.assignments.len(),
            escape_html(&detail.description),
        )
    }

    fn render_workflow_assignment_snapshot(items: &[WorkflowAssignmentSummary]) -> String {
        if items.is_empty() {
            return r#"<p class="muted">No assignments are attached to this workflow yet.</p>"#
                .into();
        }
        items.iter()
            .take(3)
            .map(|assignment| {
                let work_state = if assignment.has_draft {
                    "Draft exists"
                } else if assignment.has_submitted {
                    "Submitted"
                } else {
                    "Pending"
                };
                let active_state = if assignment.is_active { "Active" } else { "Inactive" };
                format!(
                    r#"<article class="record-card compact-record-card workflow-assignment-card"><div class="page-title-row compact-title-row"><div><h4>{}</h4><p class="muted">Assigned to {} ({})</p></div><p class="workflow-directory-card__status">{}</p></div><div class="workflow-directory-card__meta"><p><strong>Node:</strong> {}</p><p><strong>Work State:</strong> {}</p></div></article>"#,
                    escape_html(&assignment.node_name),
                    escape_html(&assignment.account_display_name),
                    escape_html(&assignment.account_email),
                    escape_html(active_state),
                    escape_html(&assignment.node_name),
                    escape_html(work_state),
                )
            })
            .collect::<Vec<_>>()
            .join("")
    }

    fn render_workflow_directory_detail(detail: &WorkflowDefinition) -> String {
        let publish_button = detail
            .versions
            .iter()
            .find(|version| version.status != "published")
            .map(|version| {
                format!(
                    r#"<button class="button is-light" type="button" data-publish-workflow-version="{}">Publish Draft Version</button>"#,
                    escape_html(&version.id)
                )
            })
            .unwrap_or_default();

        let versions = if detail.versions.is_empty() {
            r#"<p class="muted">No workflow versions exist yet.</p>"#.into()
        } else {
            detail
                .versions
                .iter()
                .take(3)
                .map(|version| {
                    format!(
                        r#"<article class="record-card compact-record-card"><div class="page-title-row compact-title-row"><div><h4>{}</h4><p class="muted">{}</p></div><p class="workflow-directory-card__status">{}</p></div><div class="workflow-directory-card__meta"><p><strong>Linked Form Version:</strong> {}</p></div></article>"#,
                        escape_html(&version.title),
                        escape_html(
                            version
                                .form_version_label
                                .as_deref()
                                .unwrap_or("Draft version")
                        ),
                        escape_html(&version.status),
                        escape_html(
                            version
                                .form_version_label
                                .as_deref()
                                .unwrap_or("Draft version")
                        ),
                    )
                })
                .collect::<Vec<_>>()
                .join("")
        };

        format!(
            r#"<article class="record-detail workflow-selected-detail"><div class="page-title-row compact-title-row"><div><p class="eyebrow">Selected Workflow</p><h3>{}</h3><p>{}</p></div><p class="workflow-directory-card__status">{}</p></div><div class="workflow-directory-card__meta workflow-selected-detail__meta"><p><strong>Linked Form:</strong> <a href="/app/forms/{}">{}</a></p><p><strong>Slug:</strong> {}</p><p><strong>Assignments:</strong> {}</p><p><strong>Versions:</strong> {}</p></div><div class="actions"><a class="button-link button is-light" href="/app/workflows/{}">Open Detail</a><a class="button-link button is-light" href="/app/workflows/{}/edit">Edit Workflow</a><a class="button-link button is-primary" href="{}">Manage Assignments</a>{}</div><section class="record-detail workflow-selected-detail__section"><div><p class="eyebrow">Assignments</p><h4>Current assignment footprint</h4></div><div class="record-list workflow-selected-detail__assignments">{}</div></section><section class="record-detail workflow-selected-detail__section"><div><p class="eyebrow">Versions</p><h4>Workflow version lifecycle</h4></div><div class="record-list workflow-selected-detail__versions">{}</div></section></article>"#,
            escape_html(&detail.name),
            escape_html(&detail.description),
            escape_html(
                detail
                    .versions
                    .iter()
                    .find(|version| version.status == "published")
                    .and_then(|version| version.form_version_label.as_deref())
                    .unwrap_or("Draft only")
            ),
            escape_html(&detail.form_id),
            escape_html(&detail.form_name),
            escape_html(&detail.slug),
            detail.assignments.len(),
            detail.versions.len(),
            escape_html(&detail.id),
            escape_html(&detail.id),
            escape_html(&assignment_console_path(Some(&detail.id))),
            publish_button,
            render_workflow_assignment_snapshot(&detail.assignments),
            versions,
        )
    }

    fn render_workflow_directory_empty_detail() -> String {
        r#"<div class="record-detail workflow-selected-detail"><p class="muted">Select a workflow from the directory to inspect its linked form, assignments, and versions.</p></div>"#.into()
    }

    fn render_versions(detail: &WorkflowDefinition) -> String {
        if detail.versions.is_empty() {
            return r#"<p class="muted">No workflow versions exist yet.</p>"#.into();
        }
        detail
            .versions
            .iter()
            .map(|version| {
                let publish = if version.status != "published" {
                    format!(r#"<button class="button is-light" type="button" data-publish-workflow-version="{}">Publish</button>"#, escape_html(&version.id))
                } else {
                    String::new()
                };
                format!(
                    r#"<article class="record-card"><h4>{}</h4><p class="muted">{}</p><p class="muted">Status: {}</p><div class="actions">{}</div></article>"#,
                    escape_html(version.form_version_label.as_deref().unwrap_or("Draft")),
                    escape_html(&version.title),
                    escape_html(&version.status),
                    publish
                )
            })
            .collect::<Vec<_>>()
            .join("")
    }

    fn render_assignment_rows(items: &[AssignmentSummary]) -> String {
        if items.is_empty() {
            return r#"<p class="muted">No workflow assignments found for the current filters.</p>"#.into();
        }
        items.iter()
            .map(|item| {
                let state = if item.is_active { "Active" } else { "Inactive" };
                let work_state = if item.has_draft {
                    "Draft exists"
                } else if item.has_submitted {
                    "Submitted"
                } else {
                    "Pending"
                };
                let toggle_label = if item.is_active { "Deactivate" } else { "Activate" };
                format!(
                    r#"<article class="record-card workflow-assignment-card"><div class="page-title-row compact-title-row"><div><p class="eyebrow">Assignment</p><h4>{}</h4><p class="muted">{}</p></div><p class="workflow-directory-card__status">{}</p></div><div class="workflow-directory-card__meta"><p><strong>Assignee:</strong> {} ({})</p><p><strong>Node:</strong> {}</p><p><strong>Work State:</strong> {}</p></div><div class="actions"><button class="button is-light" type="button" data-toggle-workflow-assignment="{}">{}</button></div></article>"#,
                    escape_html(&item.workflow_name),
                    escape_html("Assignment-backed response entry point"),
                    escape_html(state),
                    escape_html(&item.account_display_name),
                    escape_html(&item.account_email),
                    escape_html(&item.node_name),
                    escape_html(work_state),
                    item.id,
                    toggle_label
                )
            })
            .collect::<Vec<_>>()
            .join("")
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

    fn bind_workflow_directory_selection(items: Vec<WorkflowSummary>) {
        attach_click_handler_by_attr("data-workflow-directory-select", move |workflow_id| {
            let items = items.clone();
            render_workflow_directory_surface(items, Some(workflow_id));
        });
    }

    fn render_workflow_directory_surface(
        items: Vec<WorkflowSummary>,
        requested_workflow_id: Option<String>,
    ) {
        let selected_workflow_id =
            select_directory_workflow_id(&items, requested_workflow_id.as_deref());
        set_html(
            "workflow-directory-metrics",
            &render_workflow_directory_metrics(&items),
        );
        set_html(
            "workflow-list",
            &render_workflow_directory(&items, selected_workflow_id.as_deref()),
        );
        bind_workflow_directory_selection(items.clone());

        let Some(selected_workflow_id) = selected_workflow_id else {
            replace_workflow_list_location(None);
            set_html(
                "workflow-directory-detail",
                &render_workflow_directory_empty_detail(),
            );
            return;
        };

        replace_workflow_list_location(Some(&selected_workflow_id));
        set_html(
            "workflow-directory-detail",
            r#"<div class="record-detail workflow-selected-detail"><p class="muted">Loading selected workflow...</p></div>"#,
        );

        spawn_local(async move {
            match get_json::<WorkflowDefinition>(&format!("/api/workflows/{selected_workflow_id}"))
                .await
            {
                Ok(detail) => {
                    set_html(
                        "workflow-directory-detail",
                        &render_workflow_directory_detail(&detail),
                    );
                    attach_click_handler_by_attr(
                        "data-publish-workflow-version",
                        move |version_id| {
                            let selected_workflow_id = selected_workflow_id.clone();
                            spawn_local(async move {
                                let _ = post_json::<IdResponse>(
                                    &format!("/api/workflow-versions/{version_id}/publish"),
                                    &json!({}),
                                )
                                .await;
                                redirect(&workflow_list_path(Some(&selected_workflow_id)));
                            });
                        },
                    );
                }
                Err(error) => set_html(
                    "workflow-directory-detail",
                    &format!(
                        r#"<div class="record-detail workflow-selected-detail"><p class="muted">{}</p></div>"#,
                        escape_html(&error)
                    ),
                ),
            }
        });
    }

    pub fn load_list_page() {
        spawn_local(async move {
            match get_json::<Vec<WorkflowSummary>>("/api/workflows").await {
                Ok(items) => {
                    let selected_workflow_id = current_search_param("workflowId");
                    render_workflow_directory_surface(items, selected_workflow_id);
                }
                Err(error) => {
                    set_html(
                        "workflow-directory-metrics",
                        &format!(r#"<p class="muted">{}</p>"#, escape_html(&error)),
                    );
                    set_html(
                        "workflow-list",
                        &format!(r#"<p class="muted">{}</p>"#, escape_html(&error)),
                    );
                    set_html(
                        "workflow-directory-detail",
                        &format!(
                            r#"<div class="record-detail workflow-selected-detail"><p class="muted">{}</p></div>"#,
                            escape_html(&error)
                        ),
                    );
                }
            }
        });
    }

    pub fn load_create_page() {
        spawn_local(async move {
            match get_json::<Vec<FormSummary>>("/api/admin/forms").await {
                Ok(forms) => {
                    set_html(
                        "workflow-form-id",
                        &options_html(
                            &forms,
                            |item| &item.id,
                            |item| format!("{} ({})", item.name, item.slug),
                            "Choose linked form",
                        ),
                    );
                }
                Err(error) => set_text("workflow-form-status", &error),
            }
        });

        attach_submit_handler("workflow-form", move || {
            spawn_local(async move {
                let payload = json!({
                    "name": input_value("workflow-name").unwrap_or_default(),
                    "slug": input_value("workflow-slug").unwrap_or_default(),
                    "form_id": select_value("workflow-form-id").unwrap_or_default(),
                    "description": textarea_value("workflow-description").unwrap_or_default(),
                });
                match post_json::<IdResponse>("/api/workflows", &payload).await {
                    Ok(response) => redirect(&format!("/app/workflows/{}/edit", response.id)),
                    Err(error) => set_text("workflow-form-status", &error),
                }
            });
        });
    }

    pub fn load_detail_page(workflow_id: String) {
        let workflow_id_for_load = workflow_id.clone();
        spawn_local(async move {
            match get_json::<WorkflowDefinition>(&format!("/api/workflows/{workflow_id_for_load}"))
                .await
            {
                Ok(detail) => {
                    set_html("workflow-detail", &render_workflow_detail(&detail));
                    set_html("workflow-version-list", &render_versions(&detail));
                    let workflow_id_for_publish = workflow_id.clone();
                    attach_click_handler_by_attr(
                        "data-publish-workflow-version",
                        move |version_id| {
                            let workflow_id = workflow_id_for_publish.clone();
                            spawn_local(async move {
                                let _ = post_json::<IdResponse>(
                                    &format!("/api/workflow-versions/{version_id}/publish"),
                                    &json!({}),
                                )
                                .await;
                                redirect(&format!("/app/workflows/{workflow_id}"));
                            });
                        },
                    );
                }
                Err(error) => {
                    set_html(
                        "workflow-detail",
                        &format!(r#"<p class="muted">{}</p>"#, escape_html(&error)),
                    );
                    set_html(
                        "workflow-version-list",
                        &format!(r#"<p class="muted">{}</p>"#, escape_html(&error)),
                    );
                }
            }
        });
    }

    pub fn load_edit_page(workflow_id: String) {
        let workflow_id_for_form = workflow_id.clone();
        let workflow_id_for_load = workflow_id.clone();
        spawn_local(async move {
            let forms = get_json::<Vec<FormSummary>>("/api/admin/forms").await;
            let detail =
                get_json::<WorkflowDefinition>(&format!("/api/workflows/{workflow_id_for_load}"))
                    .await;
            match (forms, detail) {
                (Ok(forms), Ok(detail)) => {
                    set_input_value("workflow-name", &detail.name);
                    set_input_value("workflow-slug", &detail.slug);
                    set_textarea_value("workflow-description", &detail.description);
                    set_html(
                        "workflow-form-id",
                        &options_html(
                            &forms,
                            |item| &item.id,
                            |item| format!("{} ({})", item.name, item.slug),
                            "Choose linked form",
                        ),
                    );
                    set_select_value("workflow-form-id", &detail.form_id);

                    let version_options = forms
                        .iter()
                        .find(|form| form.id == detail.form_id)
                        .map(|form| {
                            options_html(
                                &form.versions,
                                |item| &item.id,
                                |item| {
                                    format!(
                                        "{} ({})",
                                        item.version_label
                                            .clone()
                                            .unwrap_or_else(|| "Draft".into()),
                                        item.status
                                    )
                                },
                                "Choose form version",
                            )
                        })
                        .unwrap_or_else(|| {
                            r#"<option value="">No versions available</option>"#.into()
                        });
                    set_html(
                        "workflow-version-editor",
                        &format!(
                            r#"<div class="form-grid"><div class="form-field"><label for="workflow-version-form-version">Linked Form Version</label><select class="input" id="workflow-version-form-version">{}</select></div></div><div class="actions"><button class="button is-light" type="button" id="workflow-version-create">Create Workflow Version</button></div><div class="record-list">{}</div>"#,
                            version_options,
                            render_versions(&detail)
                        ),
                    );
                    let workflow_id_for_publish = workflow_id.clone();
                    attach_click_handler_by_attr(
                        "data-publish-workflow-version",
                        move |version_id| {
                            let workflow_id = workflow_id_for_publish.clone();
                            spawn_local(async move {
                                let _ = post_json::<IdResponse>(
                                    &format!("/api/workflow-versions/{version_id}/publish"),
                                    &json!({}),
                                )
                                .await;
                                redirect(&format!("/app/workflows/{workflow_id}"));
                            });
                        },
                    );
                    if let Some(button) = by_id("workflow-version-create") {
                        let workflow_id = workflow_id.clone();
                        let closure = Closure::wrap(Box::new(move |_event: web_sys::Event| {
                            let workflow_id = workflow_id.clone();
                            spawn_local(async move {
                                let payload = json!({
                                    "form_version_id": select_value("workflow-version-form-version").unwrap_or_default(),
                                    "title": "Primary Response",
                                });
                                let _ = post_json::<IdResponse>(
                                    &format!("/api/workflows/{workflow_id}/versions"),
                                    &payload,
                                )
                                .await;
                                redirect(&format!("/app/workflows/{workflow_id}/edit"));
                            });
                        })
                            as Box<dyn FnMut(_)>);
                        button
                            .add_event_listener_with_callback(
                                "click",
                                closure.as_ref().unchecked_ref(),
                            )
                            .ok();
                        closure.forget();
                    }
                }
                (Err(error), _) | (_, Err(error)) => set_text("workflow-form-status", &error),
            }
        });

        attach_submit_handler("workflow-form", move || {
            let workflow_id = workflow_id_for_form.clone();
            spawn_local(async move {
                let payload = json!({
                    "name": input_value("workflow-name").unwrap_or_default(),
                    "slug": input_value("workflow-slug").unwrap_or_default(),
                    "form_id": select_value("workflow-form-id").unwrap_or_default(),
                    "description": textarea_value("workflow-description").unwrap_or_default(),
                });
                match put_json::<IdResponse>(&format!("/api/workflows/{workflow_id}"), &payload)
                    .await
                {
                    Ok(_) => redirect(&format!("/app/workflows/{workflow_id}")),
                    Err(error) => set_text("workflow-form-status", &error),
                }
            });
        });
    }

    pub fn load_assignment_page() {
        spawn_local(async move {
            let workflows = get_json::<Vec<WorkflowSummary>>("/api/workflows").await;
            let users = get_json::<Vec<UserSummary>>("/api/admin/users").await;
            let nodes = get_json::<Vec<NodeSummary>>("/api/nodes").await;
            let workflow_id = current_search_param("workflowId");
            let form_id = current_search_param("formId");
            let mut filter = String::from("/api/workflow-assignments");
            if let Some(workflow_id) = workflow_id.as_ref() {
                filter.push_str(&format!("?workflow_id={workflow_id}"));
            } else if let Some(form_id) = form_id.as_ref() {
                filter.push_str(&format!("?form_id={form_id}"));
            }
            let assignments = get_json::<Vec<AssignmentSummary>>(&filter).await;
            match (workflows, users, nodes, assignments) {
                (Ok(workflows), Ok(users), Ok(nodes), Ok(assignments)) => {
                    let selected_workflow_id = workflow_id.as_deref();
                    let default_node_id = workflow_id.as_ref().and_then(|workflow_id| {
                        assignments
                            .iter()
                            .find(|item| item.workflow_id == *workflow_id)
                            .map(|item| item.node_id.as_str())
                    });
                    set_html(
                        "workflow-assignment-toolbar",
                        &format!(
                            r#"<p id="workflow-assignment-status" class="muted">Assignment changes save here.</p><div class="ui-toolbar workflow-assignment-toolbar"><div class="ui-toolbar__primary"><div class="form-grid workflow-assignment-toolbar__grid"><div class="form-field"><label for="workflow-assignment-workflow">Workflow</label><select class="input" id="workflow-assignment-workflow">{}</select></div><div class="form-field"><label for="workflow-assignment-node">Node</label><select class="input" id="workflow-assignment-node">{}</select></div><div class="form-field"><label for="workflow-assignment-account">Assignee</label><select class="input" id="workflow-assignment-account">{}</select></div></div></div><div class="ui-toolbar__secondary"><button class="button is-primary" type="button" id="workflow-assignment-create">Create Assignment</button></div></div>"#,
                            options_html_selected(
                                &workflows,
                                |item| &item.id,
                                |item| item.name.clone(),
                                "Choose workflow",
                                selected_workflow_id,
                            ),
                            options_html_selected(
                                &nodes,
                                |item| &item.id,
                                |item| item.name.clone(),
                                "Choose node",
                                default_node_id,
                            ),
                            options_html(
                                &users,
                                |item| &item.id,
                                |item| format!("{} ({})", item.display_name, item.email),
                                "Choose assignee"
                            ),
                        ),
                    );
                    set_html(
                        "workflow-assignment-list",
                        &render_assignment_rows(&assignments),
                    );
                    attach_click_handler_by_attr(
                        "data-toggle-workflow-assignment",
                        move |assignment_id| {
                            let assignments = assignments.clone();
                            spawn_local(async move {
                                if let Some(current) =
                                    assignments.iter().find(|item| item.id == assignment_id)
                                {
                                    let payload = json!({
                                        "node_id": current.node_id,
                                        "account_id": current.account_id,
                                        "is_active": !current.is_active,
                                    });
                                    let _ = put_json::<IdResponse>(
                                        &format!("/api/workflow-assignments/{}", current.id),
                                        &payload,
                                    )
                                    .await;
                                    redirect(&assignment_console_path(Some(&current.workflow_id)));
                                }
                            });
                        },
                    );
                    if let Some(button) = by_id("workflow-assignment-create") {
                        let closure = Closure::wrap(Box::new(move |_event: web_sys::Event| {
                            spawn_local(async move {
                                let workflow_id = select_value("workflow-assignment-workflow")
                                    .unwrap_or_default();
                                let node_id =
                                    select_value("workflow-assignment-node").unwrap_or_default();
                                let account_id =
                                    select_value("workflow-assignment-account").unwrap_or_default();
                                let detail = get_json::<WorkflowDefinition>(&format!(
                                    "/api/workflows/{workflow_id}"
                                ))
                                .await;
                                match detail {
                                    Ok(detail) => {
                                        let version_id = detail
                                            .versions
                                            .iter()
                                            .find(|version| version.status == "published")
                                            .map(|version| version.id.clone())
                                            .or_else(|| {
                                                detail
                                                    .versions
                                                    .first()
                                                    .map(|version| version.id.clone())
                                            })
                                            .unwrap_or_default();
                                        let payload = json!({
                                            "workflow_version_id": version_id,
                                            "node_id": node_id,
                                            "account_id": account_id,
                                        });
                                        match post_json::<IdResponse>(
                                            "/api/workflow-assignments",
                                            &payload,
                                        )
                                        .await
                                        {
                                            Ok(_) => redirect(&assignment_console_path(Some(
                                                &workflow_id,
                                            ))),
                                            Err(error) => {
                                                set_text("workflow-assignment-status", &error)
                                            }
                                        }
                                    }
                                    Err(error) => set_text("workflow-assignment-status", &error),
                                }
                            });
                        })
                            as Box<dyn FnMut(_)>);
                        button
                            .add_event_listener_with_callback(
                                "click",
                                closure.as_ref().unchecked_ref(),
                            )
                            .ok();
                        closure.forget();
                    }
                }
                (Err(error), _, _, _)
                | (_, Err(error), _, _)
                | (_, _, Err(error), _)
                | (_, _, _, Err(error)) => {
                    set_html(
                        "workflow-assignment-list",
                        &format!(r#"<p class="muted">{}</p>"#, escape_html(&error)),
                    );
                }
            }
        });
    }

    pub fn set_context(page_key: &'static str, record_id: Option<String>) {
        set_page_context(page_key, "workflows", record_id);
    }
}

#[component]
pub fn WorkflowsPage() -> impl IntoView {
    let session = use_account_session();
    #[cfg(not(feature = "hydrate"))]
    let _ = session;
    #[cfg(feature = "hydrate")]
    Effect::new(move |_| {
        if session.loaded.get() && session.account.get().is_some() {
            hydrate::set_context("workflow-list", None);
            hydrate::load_list_page();
        }
    });
    view! {
        <NativePage
            title="Tessara Workflows"
            description="Tessara workflows list screen."
            page_key="workflow-list"
            active_route="workflows"
            workspace_label="Product Area"
            required_capability="workflows:read"
            breadcrumbs=vec![
                BreadcrumbItem::link("Home", "/app"),
                BreadcrumbItem::current("Workflows"),
            ]
        >
            <PageHeader
                eyebrow="Workflows"
                title="Workflow Directory"
                description="Browse workflow records, inspect linked form runtime status, and branch into assignment management from one directory-first route."
            />
            <MetadataStrip items=vec![
                ("Mode", "Directory".into()),
                ("Surface", "Workflow directory".into()),
                ("State", "Loading workflow records".into()),
            ]/>
            <Panel
                title="Workflow Directory"
                description="Select a workflow from the directory, inspect the active runtime shell, and branch into assignment management from the selected detail rail."
            >
                <div id="workflow-directory-metrics" class="record-detail workflow-directory-overview">
                    <p class="muted">"Loading workflow summary..."</p>
                </div>
                <div class="form-grid workflow-directory-layout">
                    <section class="record-detail workflow-directory-panel">
                        <div class="page-title-row compact-title-row">
                            <div>
                                <p class="eyebrow">"Directory"</p>
                                <h3>"Current workflow set"</h3>
                                <p class="muted">
                                    "Choose a workflow to inspect its linked form, version posture, and assignment load without leaving the directory."
                                </p>
                            </div>
                            <div class="actions">
                                <a class="button-link button is-light" href="/app/workflows/assignments">
                                    "Open Assignment Management"
                                </a>
                                <a class="button-link button is-primary" href="/app/workflows/new">
                                    "Create Workflow"
                                </a>
                            </div>
                        </div>
                        <div id="workflow-list" class="record-list workflow-directory-list">
                            <p class="muted">"Loading workflow records..."</p>
                        </div>
                    </section>
                    <aside id="workflow-directory-detail" class="record-detail workflow-directory-detail-panel">
                        <p class="muted">"Loading selected workflow..."</p>
                    </aside>
                </div>
            </Panel>
        </NativePage>
    }
}

#[component]
pub fn WorkflowCreatePage() -> impl IntoView {
    let session = use_account_session();
    #[cfg(not(feature = "hydrate"))]
    let _ = session;
    #[cfg(feature = "hydrate")]
    Effect::new(move |_| {
        if session.loaded.get() && session.account.get().is_some() {
            hydrate::set_context("workflow-create", None);
            hydrate::load_create_page();
        }
    });
    view! {
        <NativePage
            title="Create Workflow"
            description="Create a Tessara workflow."
            page_key="workflow-create"
            active_route="workflows"
            workspace_label="Product Area"
            required_capability="workflows:write"
            breadcrumbs=vec![
                BreadcrumbItem::link("Home", "/app"),
                BreadcrumbItem::link("Workflows", "/app/workflows"),
                BreadcrumbItem::current("Create Workflow"),
            ]
        >
            <PageHeader
                eyebrow="Workflows"
                title="Create Workflow"
                description="Create a workflow that wraps one form as a single-step runtime shell."
            />
            <MetadataStrip items=vec![
                ("Mode", "Create".into()),
                ("Surface", "Workflow authoring".into()),
                ("State", "Metadata entry".into()),
            ]/>
            <Panel
                title="Workflow Metadata"
                description="Create a form-backed workflow shell and then publish versions from linked form versions."
            >
                <p id="workflow-form-status" class="muted">"Workflow metadata saves here."</p>
                <form id="workflow-form" class="entity-form">
                    <div class="form-grid">
                        <div class="form-field wide-field">
                            <label for="workflow-name">"Name"</label>
                            <input class="input" id="workflow-name" type="text" autocomplete="off" />
                        </div>
                        <div class="form-field">
                            <label for="workflow-slug">"Slug"</label>
                            <input class="input" id="workflow-slug" type="text" autocomplete="off" />
                        </div>
                        <div class="form-field">
                            <label for="workflow-form-id">"Linked Form"</label>
                            <select class="input" id="workflow-form-id"></select>
                        </div>
                        <div class="form-field wide-field">
                            <label for="workflow-description">"Description"</label>
                            <textarea class="textarea" id="workflow-description" rows="3"></textarea>
                        </div>
                    </div>
                    <div class="actions">
                        <button class="button is-primary" type="submit">"Create Workflow"</button>
                        <a class="button-link button is-light" href="/app/workflows">"Cancel"</a>
                    </div>
                </form>
            </Panel>
        </NativePage>
    }
}

#[component]
fn WorkflowAssignmentConsoleLink(workflow_id: RwSignal<String>) -> impl IntoView {
    view! {
        <a
            class="button-link button is-light"
            href=move || format!("/app/workflows/assignments?workflowId={}", workflow_id.get())
        >
            "Assignment Console"
        </a>
    }
}

#[component]
pub fn WorkflowDetailPage() -> impl IntoView {
    let WorkflowRouteParams { workflow_id } = require_route_params();
    let record_id = workflow_id.clone();
    let assignment_console_workflow_id = RwSignal::new(workflow_id.clone());
    let _workflow_id_for_load = workflow_id.clone();
    let session = use_account_session();
    #[cfg(not(feature = "hydrate"))]
    let _ = session;
    #[cfg(feature = "hydrate")]
    Effect::new(move |_| {
        if session.loaded.get() && session.account.get().is_some() {
            hydrate::set_context("workflow-detail", Some(_workflow_id_for_load.clone()));
            hydrate::load_detail_page(_workflow_id_for_load.clone());
        }
    });
    view! {
        <NativePage
            title="Workflow Detail"
            description="Inspect a Tessara workflow."
            page_key="workflow-detail"
            active_route="workflows"
            workspace_label="Product Area"
            record_id=record_id
            required_capability="workflows:read"
            breadcrumbs=vec![
                BreadcrumbItem::link("Home", "/app"),
                BreadcrumbItem::link("Workflows", "/app/workflows"),
                BreadcrumbItem::current("Workflow Detail"),
            ]
        >
            <PageHeader
                eyebrow="Workflows"
                title="Workflow Detail"
                description="Inspect the selected workflow, linked form versions, and its assignment footprint."
            />
            <MetadataStrip items=vec![
                ("Mode", "Detail".into()),
                ("Surface", "Workflow runtime shell".into()),
                ("State", "Loading record".into()),
            ]/>
            <Panel
                title="Workflow Summary"
                description="Core workflow metadata and current runtime status appear here."
            >
                <div class="actions">
                    <a class="button-link button is-light" href="/app/workflows">"Back to List"</a>
                    <WorkflowAssignmentConsoleLink workflow_id=assignment_console_workflow_id />
                </div>
                <div id="workflow-detail" class="record-detail">
                    <p class="muted">"Loading workflow detail..."</p>
                </div>
            </Panel>
            <Panel
                title="Workflow Versions"
                description="Single-step workflow versions mirror linked form versions for Sprint 2A."
            >
                <div id="workflow-version-list" class="record-list">
                    <p class="muted">"Loading workflow versions..."</p>
                </div>
            </Panel>
        </NativePage>
    }
}

#[component]
pub fn WorkflowEditPage() -> impl IntoView {
    let WorkflowRouteParams { workflow_id } = require_route_params();
    let record_id = workflow_id.clone();
    let workflow_detail_href = format!("/app/workflows/{workflow_id}");
    let assignment_console_workflow_id = RwSignal::new(workflow_id.clone());
    let _workflow_id_for_load = workflow_id.clone();
    let session = use_account_session();
    #[cfg(not(feature = "hydrate"))]
    let _ = session;
    #[cfg(feature = "hydrate")]
    Effect::new(move |_| {
        if session.loaded.get() && session.account.get().is_some() {
            hydrate::set_context("workflow-edit", Some(_workflow_id_for_load.clone()));
            hydrate::load_edit_page(_workflow_id_for_load.clone());
        }
    });
    view! {
        <NativePage
            title="Edit Workflow"
            description="Edit a Tessara workflow."
            page_key="workflow-edit"
            active_route="workflows"
            workspace_label="Product Area"
            record_id=record_id
            required_capability="workflows:write"
            breadcrumbs=vec![
                BreadcrumbItem::link("Home", "/app"),
                BreadcrumbItem::link("Workflows", "/app/workflows"),
                BreadcrumbItem::link("Workflow Detail", workflow_detail_href),
                BreadcrumbItem::current("Edit Workflow"),
            ]
        >
            <PageHeader
                eyebrow="Workflows"
                title="Edit Workflow"
                description="Update workflow metadata and create or publish single-step workflow versions from published form versions."
            />
            <MetadataStrip items=vec![
                ("Mode", "Edit".into()),
                ("Surface", "Workflow authoring".into()),
                ("State", "Metadata and version lifecycle".into()),
            ]/>
            <Panel
                title="Workflow Metadata"
                description="Workflow metadata stays separate from version creation and assignment management."
            >
                <div class="actions">
                    <a class="button-link button is-light" href="/app/workflows">
                        "Back to Workflows"
                    </a>
                    <WorkflowAssignmentConsoleLink workflow_id=assignment_console_workflow_id />
                </div>
                <p id="workflow-form-status" class="muted">"Workflow metadata saves here."</p>
                <form id="workflow-form" class="entity-form">
                    <div class="form-grid">
                        <div class="form-field wide-field">
                            <label for="workflow-name">"Name"</label>
                            <input class="input" id="workflow-name" type="text" autocomplete="off" />
                        </div>
                        <div class="form-field">
                            <label for="workflow-slug">"Slug"</label>
                            <input class="input" id="workflow-slug" type="text" autocomplete="off" />
                        </div>
                        <div class="form-field">
                            <label for="workflow-form-id">"Linked Form"</label>
                            <select class="input" id="workflow-form-id"></select>
                        </div>
                        <div class="form-field wide-field">
                            <label for="workflow-description">"Description"</label>
                            <textarea class="textarea" id="workflow-description" rows="3"></textarea>
                        </div>
                    </div>
                    <div class="actions">
                        <button class="button is-primary" type="submit">"Save Workflow"</button>
                    </div>
                </form>
            </Panel>
            <Panel
                title="Workflow Version Lifecycle"
                description="Create one single-step workflow version from a linked form version, then publish it when the form version is already published."
            >
                <div id="workflow-version-editor" class="record-detail">
                    <p class="muted">"Loading version lifecycle..."</p>
                </div>
            </Panel>
        </NativePage>
    }
}

#[component]
pub fn WorkflowAssignmentsPage() -> impl IntoView {
    let session = use_account_session();
    #[cfg(not(feature = "hydrate"))]
    let _ = session;
    #[cfg(feature = "hydrate")]
    Effect::new(move |_| {
        if session.loaded.get() && session.account.get().is_some() {
            hydrate::set_context("workflow-assignments", None);
            hydrate::load_assignment_page();
        }
    });
    view! {
        <NativePage
            title="Workflow Assignments"
            description="Workflow assignment console."
            page_key="workflow-assignments"
            active_route="workflows"
            workspace_label="Product Area"
            required_capability="workflows:read"
            breadcrumbs=vec![
                BreadcrumbItem::link("Home", "/app"),
                BreadcrumbItem::link("Workflows", "/app/workflows"),
                BreadcrumbItem::current("Assignment Console"),
            ]
        >
            <PageHeader
                eyebrow="Workflows"
                title="Assignment Management"
                description="Create and update assignment-backed workflow work from a dedicated management route."
            />
            <MetadataStrip items=vec![
                ("Mode", "Management".into()),
                ("Surface", "Workflow assignments".into()),
                ("State", "Loading workflow assignments".into()),
            ]/>
            <Panel
                title="Assignment Filters"
                description="Choose the workflow, target node, and assignee before creating new work assignments."
            >
                <div class="actions">
                    <a class="button-link button is-light" href="/app/workflows">"Back to Workflows"</a>
                </div>
                <div id="workflow-assignment-toolbar"></div>
            </Panel>
            <Panel
                title="Assignment Directory"
                description="Assignments stay on this route so activation state and work-progress signals remain easy to scan."
            >
                <div id="workflow-assignment-list" class="record-list">
                    <p class="muted">"Loading workflow assignments..."</p>
                </div>
            </Panel>
        </NativePage>
    }
}
