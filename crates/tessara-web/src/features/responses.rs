use leptos::prelude::*;

use crate::features::native_shell::{BreadcrumbItem, MetadataStrip, NativePage, PageHeader, Panel};
use crate::infra::routing::{SubmissionRouteParams, require_route_params};

#[cfg(feature = "hydrate")]
mod hydrate {
    use crate::features::native_runtime::{
        by_id, current_search_param, escape_html, get_json, post_json, put_json, redirect,
        select_value, set_html, set_page_context, set_text,
    };
    use serde::Deserialize;
    use serde_json::{Value, json};
    use wasm_bindgen::{JsCast, closure::Closure};
    use wasm_bindgen_futures::spawn_local;
    use web_sys::window;

    #[derive(Clone, Deserialize, PartialEq, Eq)]
    #[serde(rename_all = "snake_case")]
    enum UiAccessProfile {
        Admin,
        Operator,
        ResponseUser,
    }

    #[derive(Clone, Deserialize)]
    struct PendingWork {
        workflow_assignment_id: String,
        workflow_name: String,
        workflow_version_label: Option<String>,
        workflow_step_title: String,
        form_name: String,
        form_version_label: Option<String>,
        node_name: String,
        account_display_name: String,
    }

    #[derive(Clone, Deserialize)]
    struct SubmissionSummary {
        id: String,
        form_name: String,
        version_label: String,
        node_name: String,
        status: String,
    }

    #[derive(Clone, Deserialize)]
    struct SubmissionValueDetail {
        key: String,
        label: String,
        field_type: String,
        required: bool,
        value: Option<Value>,
    }

    #[derive(Clone, Deserialize)]
    struct SubmissionAuditEventSummary {
        event_type: String,
        account_email: Option<String>,
        created_at: String,
    }

    #[derive(Clone, Deserialize)]
    struct SubmissionDetail {
        id: String,
        form_name: String,
        version_label: String,
        form_version_id: String,
        node_name: String,
        status: String,
        values: Vec<SubmissionValueDetail>,
        audit_events: Vec<SubmissionAuditEventSummary>,
    }

    #[derive(Clone, Deserialize)]
    struct RenderedForm {
        sections: Vec<RenderedSection>,
    }

    #[derive(Clone, Deserialize)]
    struct RenderedSection {
        title: String,
        fields: Vec<RenderedField>,
    }

    #[derive(Clone, Deserialize)]
    struct RenderedField {
        id: String,
        key: String,
        label: String,
        field_type: String,
        required: bool,
    }

    #[derive(Clone, Deserialize)]
    struct AccountContext {
        account_id: String,
        display_name: String,
        email: String,
        ui_access_profile: UiAccessProfile,
        delegations: Vec<DelegationSummary>,
    }

    #[derive(Clone, Deserialize)]
    struct DelegationSummary {
        account_id: String,
        display_name: String,
        email: String,
    }

    #[derive(Clone, Deserialize)]
    struct IdResponse {
        id: String,
    }

    #[derive(Clone, Deserialize)]
    struct PublishedFormVersionSummary {
        form_name: String,
        form_version_id: String,
        version_label: String,
    }

    #[derive(Clone, Deserialize)]
    struct ResponseNodeSummary {
        id: String,
        name: String,
    }

    #[derive(Clone, Deserialize)]
    struct ResponseStartOptions {
        published_forms: Vec<PublishedFormVersionSummary>,
        nodes: Vec<ResponseNodeSummary>,
    }

    fn field_input_id(field: &RenderedField) -> String {
        format!("response-field-{}", field.id)
    }

    fn render_field_input(field: &RenderedField) -> String {
        let required = if field.required { " required" } else { "" };
        if field.field_type == "boolean" {
            return format!(
                r#"<input id="{}" type="checkbox"{}>"#,
                escape_html(&field_input_id(field)),
                required
            );
        }
        let input_type = if field.field_type == "number" {
            "number"
        } else if field.field_type == "date" {
            "date"
        } else {
            "text"
        };
        let placeholder = if field.field_type == "multi_choice" {
            "Comma-separated choices"
        } else {
            &field.label
        };
        format!(
            r#"<input class="input" id="{}" type="{}" placeholder="{}"{}>"#,
            escape_html(&field_input_id(field)),
            input_type,
            escape_html(placeholder),
            required
        )
    }

    fn render_pending_cards(items: &[PendingWork], delegate_account_id: Option<&str>) -> String {
        if items.is_empty() {
            return r#"<p class="muted">No assigned responses are ready to start.</p>"#.into();
        }
        let delegate_label = delegate_account_id
            .map(|value| format!(r#" data-delegate-account-id="{}""#, escape_html(value)))
            .unwrap_or_default();
        items.iter()
            .map(|item| {
                format!(
                    r#"<article class="record-card"><h4>{}</h4><p>{}</p><p class="muted">Form: {} {}</p><p class="muted">Acting for: {}</p><div class="actions"><button class="button-link" type="button" data-workflow-assignment-id="{}"{}>Start</button></div></article>"#,
                    escape_html(&item.workflow_name),
                    escape_html(&item.node_name),
                    escape_html(&item.form_name),
                    escape_html(item.form_version_label.as_deref().unwrap_or("")),
                    escape_html(&item.account_display_name),
                    escape_html(&item.workflow_assignment_id),
                    delegate_label,
                )
            })
            .collect::<Vec<_>>()
            .join("")
    }

    fn render_submission_cards(items: &[SubmissionSummary], show_edit: bool) -> String {
        if items.is_empty() {
            return r#"<p class="muted">No responses found.</p>"#.into();
        }
        items.iter()
            .map(|item| {
                let edit = if show_edit {
                    format!(r#"<a class="button-link" href="/app/responses/{}/edit">Edit</a>"#, escape_html(&item.id))
                } else {
                    String::new()
                };
                format!(
                    r#"<article class="record-card"><h4>{} {}</h4><p>{}</p><p class="muted">{}</p><div class="actions"><a class="button-link" href="/app/responses/{}">View</a>{}</div></article>"#,
                    escape_html(&item.form_name),
                    escape_html(&item.version_label),
                    escape_html(&item.node_name),
                    escape_html(&item.status),
                    escape_html(&item.id),
                    edit
                )
            })
            .collect::<Vec<_>>()
            .join("")
    }

    fn render_detail(detail: &SubmissionDetail) -> String {
        let values = if detail.values.is_empty() {
            "<li class=\"muted\">No saved values.</li>".to_string()
        } else {
            detail
                .values
                .iter()
                .map(|item| {
                    let value = item
                        .value
                        .as_ref()
                        .map(|value| escape_html(&value.to_string()))
                        .unwrap_or_else(|| "<span class=\"muted\">missing</span>".into());
                    format!(r#"<li>{}: {}</li>"#, escape_html(&item.label), value)
                })
                .collect::<Vec<_>>()
                .join("")
        };
        let audit = if detail.audit_events.is_empty() {
            "<li class=\"muted\">No audit events.</li>".to_string()
        } else {
            detail
                .audit_events
                .iter()
                .map(|item| {
                    format!(
                        r#"<li>{} by {} on {}</li>"#,
                        escape_html(&item.event_type),
                        escape_html(item.account_email.as_deref().unwrap_or("system")),
                        escape_html(&item.created_at)
                    )
                })
                .collect::<Vec<_>>()
                .join("")
        };
        format!(
            r#"<section class="page-panel nested-form-panel"><h3>Summary</h3><p>{} {}</p><p>{}</p><p class="muted">Status: {}</p>{}</section><section class="page-panel nested-form-panel"><h3>Values</h3><ul class="app-list">{}</ul></section><section class="page-panel nested-form-panel"><h3>Audit Trail</h3><ul class="app-list">{}</ul></section>"#,
            escape_html(&detail.form_name),
            escape_html(&detail.version_label),
            escape_html(&detail.node_name),
            escape_html(&detail.status),
            if detail.status == "draft" {
                format!(
                    r#"<p><a class="button-link" href="/app/responses/{}/edit">Edit Draft</a></p>"#,
                    escape_html(&detail.id)
                )
            } else {
                String::new()
            },
            values,
            audit
        )
    }

    fn render_edit_surface(detail: &SubmissionDetail, rendered: &RenderedForm) -> String {
        format!(
            r#"<p id="response-edit-status" class="muted">Save or submit the current draft here.</p><form id="response-edit-form" class="entity-form">{}{}</form>"#,
            rendered
                .sections
                .iter()
                .map(|section| {
                    format!(
                        r#"<section class="page-panel nested-form-panel"><h3>{}</h3><div class="form-grid">{}</div></section>"#,
                        escape_html(&section.title),
                        section
                            .fields
                            .iter()
                            .map(|field| {
                                format!(
                                    r#"<div class="form-field"><label for="{}">{}{} </label>{}</div>"#,
                                    escape_html(&field_input_id(field)),
                                    escape_html(&field.label),
                                    if field.required { " *" } else { "" },
                                    render_field_input(field)
                                )
                            })
                            .collect::<Vec<_>>()
                            .join("")
                    )
                })
                .collect::<Vec<_>>()
                .join(""),
            format!(
                r#"<div class="actions form-actions"><button type="submit" class="button is-light">Save</button><button type="button" class="button is-primary" id="response-submit-button">Submit</button><a class="button-link" href="/app/responses/{}">Cancel</a></div>"#,
                escape_html(&detail.id)
            )
        )
    }

    fn prefill_values(detail: &SubmissionDetail, rendered: &RenderedForm) {
        let values_by_key = detail
            .values
            .iter()
            .filter_map(|item| item.value.clone().map(|value| (item.key.clone(), value)))
            .collect::<std::collections::HashMap<_, _>>();
        for section in &rendered.sections {
            for field in &section.fields {
                let Some(element) = by_id(&field_input_id(field)) else {
                    continue;
                };
                let Some(value) = values_by_key.get(&field.key) else {
                    continue;
                };
                if let Ok(input) = element.clone().dyn_into::<web_sys::HtmlInputElement>() {
                    if field.field_type == "boolean" {
                        input.set_checked(value.as_bool().unwrap_or(false));
                    } else if let Some(array) = value.as_array() {
                        input.set_value(
                            &array
                                .iter()
                                .map(|item| item.as_str().unwrap_or_default())
                                .collect::<Vec<_>>()
                                .join(", "),
                        );
                    } else if let Some(string) = value.as_str() {
                        input.set_value(string);
                    } else {
                        input.set_value(&value.to_string());
                    }
                }
            }
        }
    }

    fn collect_values(rendered: &RenderedForm) -> Result<Value, String> {
        let mut values = serde_json::Map::new();
        for section in &rendered.sections {
            for field in &section.fields {
                let Some(element) = by_id(&field_input_id(field)) else {
                    continue;
                };
                let input = element
                    .dyn_into::<web_sys::HtmlInputElement>()
                    .map_err(|_| "response input was not available".to_string())?;
                if field.field_type == "boolean" {
                    values.insert(field.key.clone(), Value::Bool(input.checked()));
                    continue;
                }
                let raw = input.value().trim().to_string();
                if raw.is_empty() {
                    continue;
                }
                if field.field_type == "number" {
                    let parsed = raw
                        .parse::<f64>()
                        .map_err(|_| format!("{} must be a number", field.label))?;
                    values.insert(field.key.clone(), json!(parsed));
                } else if field.field_type == "multi_choice" {
                    values.insert(
                        field.key.clone(),
                        Value::Array(
                            raw.split(',')
                                .map(|item| Value::String(item.trim().to_string()))
                                .filter(|item| {
                                    item.as_str()
                                        .map(|value| !value.is_empty())
                                        .unwrap_or(false)
                                })
                                .collect(),
                        ),
                    );
                } else {
                    values.insert(field.key.clone(), Value::String(raw));
                }
            }
        }
        for section in &rendered.sections {
            for field in &section.fields {
                if field.required && !values.contains_key(&field.key) {
                    return Err(format!("Required fields missing: {}", field.label));
                }
            }
        }
        Ok(Value::Object(values))
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

    fn render_delegate_switcher(
        account: &AccountContext,
        current_delegate: Option<&str>,
    ) -> String {
        let mut options = vec![(account.account_id.clone(), account.display_name.clone())];
        for delegate in &account.delegations {
            options.push((
                delegate.account_id.clone(),
                if delegate.display_name.is_empty() {
                    delegate.email.clone()
                } else {
                    delegate.display_name.clone()
                },
            ));
        }
        if options.len() <= 1 {
            return String::new();
        }
        format!(
            r#"<section class="app-screen page-panel compact-panel"><h3>Delegated Response Context</h3><p class="muted">Choose which delegated account's assigned responses you are currently viewing.</p><div class="form-field"><label for="delegate-context-select">Acting For</label><select id="delegate-context-select">{}</select></div></section>"#,
            options
                .iter()
                .map(|(id, label)| {
                    format!(
                        r#"<option value="{}" {}>{}</option>"#,
                        escape_html(id),
                        if Some(id.as_str()) == current_delegate {
                            "selected"
                        } else {
                            ""
                        },
                        escape_html(label)
                    )
                })
                .collect::<Vec<_>>()
                .join("")
        )
    }

    fn render_start_actions(account: &AccountContext) -> String {
        match account.ui_access_profile {
            UiAccessProfile::Admin | UiAccessProfile::Operator => {
                r#"<a class="button-link button is-light" href="/app/responses/new">Start Response</a>"#
                    .into()
            }
            UiAccessProfile::ResponseUser => String::new(),
        }
    }

    fn responses_list_path(delegate_account_id: Option<&str>) -> String {
        delegate_account_id
            .filter(|value| !value.is_empty())
            .map(|value| format!("/app/responses?delegateAccountId={value}"))
            .unwrap_or_else(|| "/app/responses".into())
    }

    fn pending_path(delegate_account_id: Option<&str>) -> String {
        delegate_account_id
            .filter(|value| !value.is_empty())
            .map(|value| format!("/api/workflow-assignments/pending?delegate_account_id={value}"))
            .unwrap_or_else(|| "/api/workflow-assignments/pending".into())
    }

    fn drafts_path(delegate_account_id: Option<&str>) -> String {
        delegate_account_id
            .filter(|value| !value.is_empty())
            .map(|value| format!("/api/submissions?status=draft&delegate_account_id={value}"))
            .unwrap_or_else(|| "/api/submissions?status=draft".into())
    }

    fn submitted_path(delegate_account_id: Option<&str>) -> String {
        delegate_account_id
            .filter(|value| !value.is_empty())
            .map(|value| format!("/api/submissions?status=submitted&delegate_account_id={value}"))
            .unwrap_or_else(|| "/api/submissions?status=submitted".into())
    }

    fn replace_response_list_location(delegate_account_id: Option<&str>) {
        let Some(window) = window() else {
            return;
        };
        let Ok(history) = window.history() else {
            return;
        };
        let path = responses_list_path(delegate_account_id);
        let _ = history.replace_state_with_url(&wasm_bindgen::JsValue::NULL, "", Some(&path));
    }

    async fn refresh_response_queue(delegate_account_id: Option<String>) -> Result<(), String> {
        set_text("response-pending-feedback", "Loading response work...");

        let pending =
            get_json::<Vec<PendingWork>>(&pending_path(delegate_account_id.as_deref())).await;
        let drafts =
            get_json::<Vec<SubmissionSummary>>(&drafts_path(delegate_account_id.as_deref())).await;
        let submitted =
            get_json::<Vec<SubmissionSummary>>(&submitted_path(delegate_account_id.as_deref()))
                .await;

        match (pending, drafts, submitted) {
            (Ok(pending), Ok(drafts), Ok(submitted)) => {
                set_text("response-pending-feedback", "");
                set_html(
                    "response-pending-list",
                    &render_pending_cards(&pending, delegate_account_id.as_deref()),
                );
                set_html(
                    "response-draft-list",
                    &render_submission_cards(&drafts, true),
                );
                set_html(
                    "response-submitted-list",
                    &render_submission_cards(&submitted, false),
                );
                attach_pending_start_handlers();
                Ok(())
            }
            (Err(error), _, _) | (_, Err(error), _) | (_, _, Err(error)) => {
                set_html(
                    "response-pending-list",
                    &format!(r#"<p class="muted">{}</p>"#, escape_html(&error)),
                );
                set_html(
                    "response-draft-list",
                    &format!(r#"<p class="muted">{}</p>"#, escape_html(&error)),
                );
                set_html(
                    "response-submitted-list",
                    &format!(r#"<p class="muted">{}</p>"#, escape_html(&error)),
                );
                set_text("response-pending-feedback", &error);
                Err(error)
            }
        }
    }

    fn attach_pending_start_handlers() {
        let Some(document) = window().and_then(|window| window.document()) else {
            return;
        };
        let Ok(buttons) =
            document.query_selector_all("#response-pending-list [data-workflow-assignment-id]")
        else {
            return;
        };

        for index in 0..buttons.length() {
            let Some(button) = buttons.item(index) else {
                continue;
            };
            let Ok(button) = button.dyn_into::<web_sys::Element>() else {
                continue;
            };
            let Some(assignment_id) = button.get_attribute("data-workflow-assignment-id") else {
                continue;
            };
            let closure = Closure::wrap(Box::new(move |event: web_sys::Event| {
                event.prevent_default();
                set_text("response-pending-feedback", "Starting assigned response...");
                let assignment_id = assignment_id.clone();
                spawn_local(async move {
                    match post_json::<IdResponse>(
                        &format!("/api/workflow-assignments/{assignment_id}/start"),
                        &json!({}),
                    )
                    .await
                    {
                        Ok(response) => redirect(&format!("/app/responses/{}/edit", response.id)),
                        Err(error) => set_text("response-pending-feedback", &error),
                    }
                });
            }) as Box<dyn FnMut(_)>);
            button
                .add_event_listener_with_callback("click", closure.as_ref().unchecked_ref())
                .ok();
            closure.forget();
        }
    }

    pub fn load_list_page() {
        let delegate_account_id = current_search_param("delegateAccountId");
        spawn_local(async move {
            let account = get_json::<AccountContext>("/api/me").await;
            match account {
                Ok(account) => {
                    set_html("response-start-actions", &render_start_actions(&account));
                    set_text("response-pending-feedback", "");
                    set_html(
                        "response-context-switcher",
                        &render_delegate_switcher(&account, delegate_account_id.as_deref()),
                    );
                    let _ = refresh_response_queue(delegate_account_id.clone()).await;
                    if let Some(select) = by_id("delegate-context-select") {
                        let closure = Closure::wrap(Box::new(move |_event: web_sys::Event| {
                            let value = select_value("delegate-context-select").unwrap_or_default();
                            let delegate_account_id = (!value.is_empty()).then_some(value.clone());
                            replace_response_list_location(delegate_account_id.as_deref());
                            spawn_local(async move {
                                let _ = refresh_response_queue(delegate_account_id).await;
                            });
                        })
                            as Box<dyn FnMut(_)>);
                        select
                            .add_event_listener_with_callback(
                                "change",
                                closure.as_ref().unchecked_ref(),
                            )
                            .ok();
                        closure.forget();
                    }
                }
                Err(error) => {
                    set_html(
                        "response-pending-list",
                        &format!(r#"<p class="muted">{}</p>"#, escape_html(&error)),
                    );
                    set_html(
                        "response-draft-list",
                        &format!(r#"<p class="muted">{}</p>"#, escape_html(&error)),
                    );
                    set_html(
                        "response-submitted-list",
                        &format!(r#"<p class="muted">{}</p>"#, escape_html(&error)),
                    );
                }
            }
        });
    }

    pub fn load_create_page() {
        let workflow_assignment_id = current_search_param("workflowAssignmentId");
        let delegate_account_id = current_search_param("delegateAccountId");
        if let Some(workflow_assignment_id) = workflow_assignment_id {
            spawn_local(async move {
                let path = format!("/api/workflow-assignments/{workflow_assignment_id}/start");
                match post_json::<IdResponse>(&path, &json!({})).await {
                    Ok(response) => redirect(&format!("/app/responses/{}/edit", response.id)),
                    Err(error) => set_html(
                        "response-create-context-switcher",
                        &format!(r#"<p class="muted">{}</p>"#, escape_html(&error)),
                    ),
                }
            });
            return;
        }

        spawn_local(async move {
            match get_json::<AccountContext>("/api/me").await {
                Ok(account) if account.ui_access_profile == UiAccessProfile::ResponseUser => {
                    redirect(&responses_list_path(delegate_account_id.as_deref()));
                }
                Ok(_) => match get_json::<ResponseStartOptions>("/api/responses/options").await {
                    Ok(options) => {
                        let form_options = options
                            .published_forms
                            .iter()
                            .map(|item| {
                                format!(
                                    r#"<option value="{}">{} | {}</option>"#,
                                    escape_html(&item.form_version_id),
                                    escape_html(&item.form_name),
                                    escape_html(&item.version_label)
                                )
                            })
                            .collect::<Vec<_>>()
                            .join("");
                        let node_options = options
                            .nodes
                            .iter()
                            .map(|item| {
                                format!(
                                    r#"<option value="{}">{}</option>"#,
                                    escape_html(&item.id),
                                    escape_html(&item.name)
                                )
                            })
                            .collect::<Vec<_>>()
                            .join("");
                        set_html(
                            "response-form-version",
                            &format!(
                                r#"<option value="">Select published form</option>{form_options}"#
                            ),
                        );
                        set_html(
                            "response-node",
                            &format!(
                                r#"<option value="">Select target organization</option>{node_options}"#
                            ),
                        );
                        set_text(
                            "response-create-status",
                            "Choose a published form and target organization to create a draft response.",
                        );
                        attach_submit_handler("response-start-form", move || {
                            spawn_local(async move {
                                let form_version_id =
                                    select_value("response-form-version").unwrap_or_default();
                                let node_id = select_value("response-node").unwrap_or_default();
                                if form_version_id.is_empty() || node_id.is_empty() {
                                    set_text(
                                        "response-create-status",
                                        "Select both a published form and target organization.",
                                    );
                                    return;
                                }
                                match post_json::<IdResponse>(
                                    "/api/submissions/drafts",
                                    &json!({
                                        "form_version_id": form_version_id,
                                        "node_id": node_id
                                    }),
                                )
                                .await
                                {
                                    Ok(response) => {
                                        redirect(&format!("/app/responses/{}/edit", response.id))
                                    }
                                    Err(error) => set_text("response-create-status", &error),
                                }
                            });
                        });
                    }
                    Err(error) => {
                        set_text("response-create-status", &error);
                    }
                },
                Err(error) => {
                    set_html(
                        "response-create-context-switcher",
                        &format!(r#"<p class="muted">{}</p>"#, escape_html(&error)),
                    );
                }
            }
        });
    }

    pub fn load_detail_page(submission_id: String) {
        spawn_local(async move {
            match get_json::<SubmissionDetail>(&format!("/api/submissions/{submission_id}")).await {
                Ok(detail) => set_html("response-detail", &render_detail(&detail)),
                Err(error) => set_html(
                    "response-detail",
                    &format!(r#"<p class="muted">{}</p>"#, escape_html(&error)),
                ),
            }
        });
    }

    pub fn load_edit_page(submission_id: String) {
        spawn_local(async move {
            let detail =
                get_json::<SubmissionDetail>(&format!("/api/submissions/{submission_id}")).await;
            match detail {
                Ok(detail) => {
                    if detail.status != "draft" {
                        set_html(
                            "response-edit-surface",
                            &format!(
                                r#"<p class="muted">This response is submitted and cannot be edited.</p><div class="actions"><a class="button-link" href="/app/responses/{}">Back to Detail</a></div>"#,
                                escape_html(&detail.id)
                            ),
                        );
                        return;
                    }
                    match get_json::<RenderedForm>(&format!(
                        "/api/form-versions/{}/render",
                        detail.form_version_id
                    ))
                    .await
                    {
                        Ok(rendered) => {
                            set_html(
                                "response-edit-surface",
                                &render_edit_surface(&detail, &rendered),
                            );
                            prefill_values(&detail, &rendered);
                            let rendered_for_save = rendered.clone();
                            let submission_id_for_save = submission_id.clone();
                            attach_submit_handler("response-edit-form", move || {
                                let rendered = rendered_for_save.clone();
                                let submission_id = submission_id_for_save.clone();
                                spawn_local(async move {
                                    match collect_values(&rendered) {
                                        Ok(values) => match put_json::<IdResponse>(
                                            &format!("/api/submissions/{submission_id}/values"),
                                            &json!({ "values": values }),
                                        )
                                        .await
                                        {
                                            Ok(_) => {
                                                redirect(&format!("/app/responses/{submission_id}"))
                                            }
                                            Err(error) => set_text("response-edit-status", &error),
                                        },
                                        Err(error) => set_text("response-edit-status", &error),
                                    }
                                });
                            });
                            if let Some(button) = by_id("response-submit-button") {
                                let rendered_for_submit = rendered.clone();
                                let submission_id_for_submit = submission_id.clone();
                                let closure = Closure::wrap(Box::new(
                                    move |_event: web_sys::Event| {
                                        let rendered = rendered_for_submit.clone();
                                        let submission_id = submission_id_for_submit.clone();
                                        spawn_local(async move {
                                            match collect_values(&rendered) {
                                                Ok(values) => {
                                                    let save = put_json::<IdResponse>(&format!("/api/submissions/{submission_id}/values"), &json!({ "values": values })).await;
                                                    match save {
                                                    Ok(_) => match post_json::<IdResponse>(&format!("/api/submissions/{submission_id}/submit"), &json!({})).await {
                                                        Ok(_) => redirect(&format!("/app/responses/{submission_id}")),
                                                        Err(error) => set_text("response-edit-status", &error),
                                                    },
                                                    Err(error) => set_text("response-edit-status", &error),
                                                }
                                                }
                                                Err(error) => {
                                                    set_text("response-edit-status", &error)
                                                }
                                            }
                                        });
                                    },
                                )
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
                        Err(error) => set_html(
                            "response-edit-surface",
                            &format!(r#"<p class="muted">{}</p>"#, escape_html(&error)),
                        ),
                    }
                }
                Err(error) => set_html(
                    "response-edit-surface",
                    &format!(r#"<p class="muted">{}</p>"#, escape_html(&error)),
                ),
            }
        });
    }

    pub fn set_context(page_key: &'static str, record_id: Option<String>) {
        set_page_context(page_key, "responses", record_id);
    }
}

#[component]
pub fn ResponsesListPage() -> impl IntoView {
    #[cfg(feature = "hydrate")]
    hydrate::set_context("native-response-list", None);
    #[cfg(feature = "hydrate")]
    hydrate::load_list_page();
    view! {
        <NativePage
            title="Tessara Responses"
            description="Tessara responses list screen."
            page_key="native-response-list"
            active_route="responses"
            workspace_label="Product Area"
            required_capability="submissions:write"
            breadcrumbs=vec![
                BreadcrumbItem::link("Home", "/app"),
                BreadcrumbItem::current("Responses"),
            ]
        >
            <PageHeader
                eyebrow="Responses"
                title="Responses"
                description="This list screen separates pending work, drafts, and submitted responses."
            />
            <MetadataStrip items=vec![
                ("Mode", "Directory".into()),
                ("Surface", "Response work queue".into()),
                ("State", "Loading drafts and submitted work".into()),
            ]/>
            <div id="response-context-switcher"></div>
            <Panel
                title="Pending Work"
                description="Start New Response items assigned through the workflow runtime appear here."
            >
                <div id="response-start-actions" class="actions"></div>
                <p id="response-pending-feedback" class="muted"></p>
                <div id="response-pending-list" class="record-list">
                    <p class="muted">"Loading published forms..."</p>
                </div>
            </Panel>
            <Panel
                title="Draft Responses"
                description="Draft responses link to detail and edit screens."
            >
                <div id="response-draft-list" class="record-list">
                    <p class="muted">"Loading draft responses..."</p>
                </div>
            </Panel>
            <Panel
                title="Submitted Responses"
                description="Submitted responses remain read-only and link to their detail screens."
            >
                <div id="response-submitted-list" class="record-list">
                    <p class="muted">"Loading submitted responses..."</p>
                </div>
            </Panel>
        </NativePage>
    }
}

#[component]
pub fn SubmissionsPage() -> impl IntoView {
    ResponsesListPage()
}

#[component]
pub fn ResponseCreatePage() -> impl IntoView {
    #[cfg(feature = "hydrate")]
    hydrate::set_context("native-response-create", None);
    #[cfg(feature = "hydrate")]
    hydrate::load_create_page();
    view! {
        <NativePage
            title="Start Response"
            description="Start a Tessara response."
            page_key="native-response-create"
            active_route="responses"
            workspace_label="Product Area"
            required_capability="submissions:write"
            breadcrumbs=vec![
                BreadcrumbItem::link("Home", "/app"),
                BreadcrumbItem::link("Responses", "/app/responses"),
                BreadcrumbItem::current("New Response"),
            ]
        >
            <PageHeader
                eyebrow="Responses"
                title="Start Response"
                description="Create a draft response from a published form and target organization."
            />
            <MetadataStrip items=vec![
                ("Mode", "Create".into()),
                ("Surface", "Response start".into()),
                ("State", "Manual start".into()),
            ]/>
            <Panel
                title="Start Response"
                description="Choose a published form and target organization to create a draft response."
            >
                <div id="response-create-context-switcher"></div>
                <p id="response-create-status" class="muted">"Loading available response start options..."</p>
                <form id="response-start-form" class="entity-form">
                    <div class="form-grid">
                        <div class="form-field wide-field">
                            <label for="response-form-version">"Published Form"</label>
                            <select class="input" id="response-form-version"></select>
                        </div>
                        <div class="form-field">
                            <label for="response-node">"Target Organization"</label>
                            <select class="input" id="response-node"></select>
                        </div>
                    </div>
                    <div class="actions">
                        <button class="button is-primary" type="submit">"Start Draft"</button>
                        <a class="button-link button is-light" href="/app/responses">"Cancel"</a>
                    </div>
                </form>
            </Panel>
        </NativePage>
    }
}

#[component]
pub fn ResponseDetailPage() -> impl IntoView {
    let SubmissionRouteParams { submission_id } = require_route_params();
    #[cfg(feature = "hydrate")]
    hydrate::set_context("native-response-detail", Some(submission_id.clone()));
    #[cfg(feature = "hydrate")]
    hydrate::load_detail_page(submission_id.clone());
    view! {
        <NativePage
            title="Response Detail"
            description="Inspect a Tessara response."
            page_key="native-response-detail"
            active_route="responses"
            workspace_label="Product Area"
            record_id=submission_id.clone()
            required_capability="submissions:write"
            breadcrumbs=vec![
                BreadcrumbItem::link("Home", "/app"),
                BreadcrumbItem::link("Responses", "/app/responses"),
                BreadcrumbItem::current("Response Detail"),
            ]
        >
            <PageHeader
                eyebrow="Responses"
                title="Response Detail"
                description="Review the selected response and its audit history."
            />
            <MetadataStrip items=vec![
                ("Mode", "Detail".into()),
                ("Surface", "Response record".into()),
                ("State", "Loading record".into()),
            ]/>
            <Panel
                title="Response Record"
                description="Response values and audit trail appear here."
            >
                <div class="actions">
                    <a class="button-link button is-light" href="/app/responses">"Back to List"</a>
                </div>
                <div id="response-detail" class="record-detail">
                    <p class="muted">"Loading record detail..."</p>
                </div>
            </Panel>
        </NativePage>
    }
}

#[component]
pub fn ResponseEditPage() -> impl IntoView {
    let SubmissionRouteParams { submission_id } = require_route_params();
    #[cfg(feature = "hydrate")]
    hydrate::set_context("native-response-edit", Some(submission_id.clone()));
    #[cfg(feature = "hydrate")]
    hydrate::load_edit_page(submission_id.clone());
    view! {
        <NativePage
            title="Edit Response"
            description="Edit a Tessara response."
            page_key="native-response-edit"
            active_route="responses"
            workspace_label="Product Area"
            record_id=submission_id.clone()
            required_capability="submissions:write"
            breadcrumbs=vec![
                BreadcrumbItem::link("Home", "/app"),
                BreadcrumbItem::link("Responses", "/app/responses"),
                BreadcrumbItem::link("Response Detail", format!("/app/responses/{submission_id}")),
                BreadcrumbItem::current("Edit Response"),
            ]
        >
            <PageHeader
                eyebrow="Responses"
                title="Edit Response"
                description="Save changes to the current draft or submit it from this dedicated response form screen."
            />
            <MetadataStrip items=vec![
                ("Mode", "Edit".into()),
                ("Surface", "Draft response".into()),
                ("State", "Loading editable submission".into()),
            ]/>
            <Panel
                title="Draft Response Form"
                description="The current draft loads here. Submitted responses show a read-only guard instead of editable controls."
            >
                <div id="response-edit-surface" class="record-detail">
                    <p class="muted">"Loading response form..."</p>
                </div>
            </Panel>
        </NativePage>
    }
}
