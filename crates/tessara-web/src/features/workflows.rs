use leptos::prelude::*;

use crate::features::native_shell::{BreadcrumbItem, MetadataStrip, NativePage, PageHeader, Panel};
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
    use wasm_bindgen::{JsCast, closure::Closure};
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

    fn render_workflow_cards(items: &[WorkflowSummary]) -> String {
        if items.is_empty() {
            return r#"<p class="muted">No workflow records found.</p>"#.into();
        }
        items.iter()
            .map(|item| {
                format!(
                    r#"<article class="record-card"><h4>{}</h4><p>{}</p><p class="muted">Linked form: {}</p><p class="muted">Current version: {}</p><p class="muted">Status: {}</p><p class="muted">Assignments: {}</p><div class="actions"><a class="button-link" href="/app/workflows/{}">View</a><a class="button-link" href="/app/workflows/{}/edit">Edit</a><a class="button-link" href="/app/workflows/assignments?workflowId={}">Assignments</a></div></article>"#,
                    escape_html(&item.name),
                    escape_html(&item.description),
                    escape_html(&item.form_name),
                    escape_html(item.current_version_label.as_deref().unwrap_or("None")),
                    escape_html(item.current_status.as_deref().unwrap_or("draft")),
                    item.assignment_count,
                    item.id,
                    item.id,
                    item.id,
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
                    r#"<article class="record-card"><h4>{}</h4><p>{}</p><p class="muted">{}</p><p class="muted">State: {} | Work: {}</p><div class="actions"><button class="button is-light" type="button" data-toggle-workflow-assignment="{}">{}</button></div></article>"#,
                    escape_html(&item.workflow_name),
                    escape_html(&item.node_name),
                    escape_html(&format!("{} ({})", item.account_display_name, item.account_email)),
                    state,
                    work_state,
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

    pub fn load_list_page() {
        spawn_local(async move {
            match get_json::<Vec<WorkflowSummary>>("/api/workflows").await {
                Ok(items) => set_html("workflow-list", &render_workflow_cards(&items)),
                Err(error) => set_html(
                    "workflow-list",
                    &format!(r#"<p class="muted">{}</p>"#, escape_html(&error)),
                ),
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
                            r#"<p id="workflow-assignment-status" class="muted">Assignment changes save here.</p><div class="form-grid"><div class="form-field"><label for="workflow-assignment-workflow">Workflow</label><select class="input" id="workflow-assignment-workflow">{}</select></div><div class="form-field"><label for="workflow-assignment-node">Node</label><select class="input" id="workflow-assignment-node">{}</select></div><div class="form-field"><label for="workflow-assignment-account">Assignee</label><select class="input" id="workflow-assignment-account">{}</select></div></div><div class="actions"><button class="button is-primary" type="button" id="workflow-assignment-create">Create Assignment</button></div>"#,
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
    #[cfg(feature = "hydrate")]
    hydrate::set_context("workflow-list", None);
    #[cfg(feature = "hydrate")]
    hydrate::load_list_page();
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
                title="Workflows"
                description="Create, publish, and inspect form-backed workflow shells from the native runtime surface."
            />
            <MetadataStrip items=vec![
                ("Mode", "Directory".into()),
                ("Surface", "Workflow runtime shell".into()),
                ("State", "Loading workflow records".into()),
            ]/>
            <Panel
                title="Workflow Directory"
                description="Current workflow records, linked forms, and assignment counts appear here."
            >
                <div class="actions">
                    <a class="button-link button is-primary" href="/app/workflows/new">
                        "Create Workflow"
                    </a>
                </div>
                <div id="workflow-list" class="record-list">
                    <p class="muted">"Loading workflow records..."</p>
                </div>
            </Panel>
        </NativePage>
    }
}

#[component]
pub fn WorkflowCreatePage() -> impl IntoView {
    #[cfg(feature = "hydrate")]
    hydrate::set_context("workflow-create", None);
    #[cfg(feature = "hydrate")]
    hydrate::load_create_page();
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
    #[cfg(feature = "hydrate")]
    hydrate::set_context("workflow-detail", Some(workflow_id.clone()));
    #[cfg(feature = "hydrate")]
    hydrate::load_detail_page(workflow_id.clone());
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
    #[cfg(feature = "hydrate")]
    hydrate::set_context("workflow-edit", Some(workflow_id.clone()));
    #[cfg(feature = "hydrate")]
    hydrate::load_edit_page(workflow_id.clone());
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
    #[cfg(feature = "hydrate")]
    hydrate::set_context("workflow-assignments", None);
    #[cfg(feature = "hydrate")]
    hydrate::load_assignment_page();
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
                title="Assignment Console"
                description="Use one shared surface to assign workflow work to any active user."
            />
            <MetadataStrip items=vec![
                ("Mode", "Console".into()),
                ("Surface", "Assignment management".into()),
                ("State", "Loading workflow assignments".into()),
            ]/>
            <Panel
                title="Assignment Toolbar"
                description="Create and update workflow assignments in one shared console, with optional workflow or form deep links."
            >
                <div class="actions">
                    <a class="button-link button is-light" href="/app/workflows">"Back to Workflows"</a>
                </div>
                <div id="workflow-assignment-toolbar"></div>
            </Panel>
            <Panel
                title="Workflow Assignments"
                description="Assignments show the current workflow, assignee, node context, and whether draft or submitted work already exists."
            >
                <div id="workflow-assignment-list" class="record-list">
                    <p class="muted">"Loading workflow assignments..."</p>
                </div>
            </Panel>
        </NativePage>
    }
}
