use leptos::prelude::*;

use crate::features::native_shell::{
    BreadcrumbItem, MetadataStrip, NativePage, PageHeader, Panel, use_account_session,
};
use crate::infra::routing::{WorkflowRouteParams, require_route_params};

#[cfg(feature = "hydrate")]
mod hydrate {
    use crate::features::native_runtime::{
        by_id, current_search_param, delete_json, escape_html, get_json, input_value, post_json,
        put_json, redirect, select_value, set_html, set_input_value, set_page_context,
        set_select_value, set_text, set_textarea_value, textarea_value,
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
        version_count: i64,
        assignment_node_names: Vec<String>,
    }

    #[derive(Clone, Deserialize)]
    struct WorkflowVersionSummary {
        id: String,
        form_version_id: String,
        form_version_label: Option<String>,
        title: String,
        status: String,
        step_count: i64,
        steps: Vec<WorkflowStepSummary>,
    }

    #[derive(Clone, Deserialize)]
    struct WorkflowStepSummary {
        form_name: String,
        form_version_id: String,
        form_version_label: Option<String>,
        title: String,
        position: i32,
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
        scope_node_type_id: Option<String>,
        scope_node_type_name: Option<String>,
        versions: Vec<FormVersionLite>,
    }

    #[derive(Clone, Deserialize)]
    struct FormVersionLite {
        id: String,
        version_label: Option<String>,
        status: String,
        assignment_nodes: Vec<FormVersionAssignmentNode>,
    }

    #[derive(Clone, Deserialize)]
    struct FormVersionAssignmentNode {
        node_id: String,
        node_name: String,
        node_type_name: String,
        parent_node_id: Option<String>,
        node_path: String,
    }

    #[derive(Clone, Deserialize)]
    struct NodeTypeRelationship {
        parent_node_type_id: String,
        child_node_type_id: String,
    }

    #[derive(Clone, Deserialize)]
    struct NodeSummary {
        id: String,
        name: String,
        parent_node_id: Option<String>,
    }

    #[derive(Clone, Deserialize)]
    struct UserSummary {
        id: String,
        display_name: String,
        email: String,
    }

    #[derive(Clone, Deserialize)]
    struct AssignmentCandidate {
        workflow_version_id: String,
        workflow_id: String,
        label: String,
        step_count: i64,
        node_id: String,
    }

    #[derive(Clone, Deserialize)]
    struct AssigneeOption {
        account_id: String,
        display_name: String,
        email: String,
    }

    #[derive(Clone, Deserialize)]
    struct BulkAssignmentResult {
        status: String,
    }

    #[derive(Clone, Deserialize)]
    struct BulkAssignmentResponse {
        results: Vec<BulkAssignmentResult>,
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

    fn candidate_value(candidate: &AssignmentCandidate) -> String {
        format!("{}|{}", candidate.workflow_version_id, candidate.node_id)
    }

    fn selected_assignee_values() -> Vec<String> {
        let Some(document) = web_sys::window().and_then(|window| window.document()) else {
            return Vec::new();
        };
        let Ok(options) = document.query_selector_all("[data-assignee-option]:checked") else {
            return Vec::new();
        };
        let mut selected = Vec::new();
        for index in 0..options.length() {
            if let Some(option) = options.get(index) {
                if let Some(value) = option
                    .dyn_into::<web_sys::Element>()
                    .ok()
                    .and_then(|element| element.get_attribute("value"))
                {
                    if !value.is_empty() {
                        selected.push(value);
                    }
                }
            }
        }
        selected
    }

    fn render_assignee_picker(items: &[AssigneeOption]) -> String {
        if items.is_empty() {
            return r#"<div class="workflow-assignee-picker workflow-assignee-picker--empty"><p class="muted">No eligible assignees</p></div>"#.into();
        }
        let options = items
            .iter()
            .map(|item| {
                format!(
                    r#"<label class="workflow-assignee-option" role="listitem"><span><input type="checkbox" data-assignee-option value="{}"></span><span class="workflow-assignee-option__identity"><strong>{}</strong><span class="workflow-assignee-option__email">{}</span></span></label>"#,
                    escape_html(&item.account_id),
                    escape_html(&item.display_name),
                    escape_html(&item.email)
                )
            })
            .collect::<String>();
        format!(
            r#"<div class="workflow-assignee-picker"><input class="input workflow-assignee-search" id="workflow-assignee-search" type="search" placeholder="Search assignees"><div class="workflow-assignee-picker__chips" id="workflow-assignee-chips"><span class="muted">No assignees selected</span></div><div class="workflow-assignee-picker__options" role="list">{options}</div></div>"#
        )
    }

    fn render_assignment_candidate_options(items: &[AssignmentCandidate]) -> String {
        let mut html = r#"<option value="">Choose node path - workflow</option>"#.to_string();
        for item in items {
            html.push_str(&format!(
                r#"<option value="{}">{} ({} steps)</option>"#,
                escape_html(&candidate_value(item)),
                escape_html(&item.label),
                item.step_count
            ));
        }
        html
    }

    fn load_assignees_for_assignment_candidate() {
        let Some(value) = select_value("workflow-assignment-candidate") else {
            return;
        };
        let mut parts = value.split('|');
        let workflow_version_id = parts.next().unwrap_or_default().to_string();
        let node_id = parts.next().unwrap_or_default().to_string();
        if workflow_version_id.is_empty() || node_id.is_empty() {
            set_html(
                "workflow-assignment-assignees",
                r#"<div class="workflow-assignee-picker workflow-assignee-picker--empty"><p class="muted">Choose a workflow candidate first</p></div>"#,
            );
            return;
        }
        spawn_local(async move {
            let path = format!(
                "/api/workflow-assignment-candidates/assignees?workflow_version_id={workflow_version_id}&node_id={node_id}"
            );
            match get_json::<Vec<AssigneeOption>>(&path).await {
                Ok(options) => set_html(
                    "workflow-assignment-assignees",
                    &render_assignee_picker(&options),
                ),
                Err(error) => set_text("workflow-assignment-status", &error),
            }
        });
    }

    fn refresh_assignee_chips() {
        let Some(document) = web_sys::window().and_then(|window| window.document()) else {
            return;
        };
        let Ok(selected) = document.query_selector_all("[data-assignee-option]:checked") else {
            return;
        };
        let mut chips = Vec::new();
        for index in 0..selected.length() {
            if let Some(option) = selected.get(index) {
                if let Ok(input) = option.dyn_into::<web_sys::HtmlInputElement>() {
                    let option = input.closest(".workflow-assignee-option").ok().flatten();
                    let name = option
                        .as_ref()
                        .and_then(|element| element.query_selector("strong").ok().flatten())
                        .and_then(|element| element.text_content())
                        .unwrap_or_else(|| "Selected assignee".to_string());
                    let email = option
                        .as_ref()
                        .and_then(|element| {
                            element
                                .query_selector(".workflow-assignee-option__email")
                                .ok()
                                .flatten()
                        })
                        .and_then(|element| element.text_content())
                        .unwrap_or_default();
                    chips.push(format!(
                        r#"<div class="tags has-addons workflow-assignee-chip"><span class="tag workflow-assignee-chip__label" title="{}">{}</span><a class="tag is-delete workflow-assignee-chip__delete" href="" role="button" aria-label="Remove {}" data-assignee-chip-remove="{}"></a></div>"#,
                        escape_html(email.trim()),
                        escape_html(name.trim()),
                        escape_html(name.trim()),
                        escape_html(&input.value())
                    ));
                }
            }
        }
        let html = if chips.is_empty() {
            r#"<span class="muted">No assignees selected</span>"#.to_string()
        } else {
            chips.join("")
        };
        set_html("workflow-assignee-chips", &html);
    }

    fn filter_assignee_options() {
        let Some(document) = web_sys::window().and_then(|window| window.document()) else {
            return;
        };
        let query = by_id("workflow-assignee-search")
            .and_then(|element| element.dyn_into::<web_sys::HtmlInputElement>().ok())
            .map(|input| input.value().trim().to_lowercase())
            .unwrap_or_default();
        let Ok(options) = document.query_selector_all(".workflow-assignee-option") else {
            return;
        };
        for index in 0..options.length() {
            let Some(option) = options.get(index) else {
                continue;
            };
            let Ok(option) = option.dyn_into::<web_sys::Element>() else {
                continue;
            };
            let label = option.text_content().unwrap_or_default().to_lowercase();
            if query.is_empty() || label.contains(&query) {
                option.remove_attribute("hidden").ok();
            } else {
                option.set_attribute("hidden", "").ok();
            }
        }
    }

    fn published_form_version_options(forms: &[FormSummary]) -> String {
        let mut html = r#"<option value="">Choose published form version</option>"#.to_string();
        for form in forms {
            for version in form
                .versions
                .iter()
                .filter(|version| version.status == "published")
            {
                let scope_id = form.scope_node_type_id.as_deref().unwrap_or_default();
                let scope_name = form.scope_node_type_name.as_deref().unwrap_or("Unscoped");
                let assignment_node_ids = version
                    .assignment_nodes
                    .iter()
                    .map(|node| node.node_id.as_str())
                    .collect::<Vec<_>>()
                    .join("|");
                let assignment_hint = if version.assignment_nodes.is_empty() {
                    "No linked nodes".to_string()
                } else if version.assignment_nodes.len() == 1 {
                    let node = &version.assignment_nodes[0];
                    format!(
                        "{}: {}",
                        node.node_type_name,
                        if node.node_path.is_empty() {
                            node.node_name.as_str()
                        } else {
                            node.node_path.as_str()
                        }
                    )
                } else {
                    format!("{} linked nodes", version.assignment_nodes.len())
                };
                html.push_str(&format!(
                    r#"<option value="{}" data-scope-node-type-id="{}" data-assignment-node-ids="{}">{} - {} ({}, {})</option>"#,
                    escape_html(&version.id),
                    escape_html(scope_id),
                    escape_html(&assignment_node_ids),
                    escape_html(&form.name),
                    escape_html(
                        version
                            .version_label
                            .as_deref()
                            .unwrap_or("Published version")
                    ),
                    escape_html(scope_name),
                    escape_html(&assignment_hint)
                ));
            }
        }
        html
    }

    fn option_scope_id(select: &web_sys::HtmlSelectElement) -> String {
        let index = select.selected_index();
        if index < 0 {
            return String::new();
        }
        let Ok(options) = select.query_selector_all("option") else {
            return String::new();
        };
        options
            .get(index as u32)
            .and_then(|option| {
                option
                    .dyn_into::<web_sys::Element>()
                    .ok()
                    .and_then(|element| element.get_attribute("data-scope-node-type-id"))
            })
            .unwrap_or_default()
    }

    fn option_assignment_node_ids(select: &web_sys::HtmlSelectElement) -> Vec<String> {
        let index = select.selected_index();
        if index < 0 {
            return Vec::new();
        }
        let Ok(options) = select.query_selector_all("option") else {
            return Vec::new();
        };
        options
            .get(index as u32)
            .and_then(|option| {
                option
                    .dyn_into::<web_sys::Element>()
                    .ok()
                    .and_then(|element| element.get_attribute("data-assignment-node-ids"))
            })
            .unwrap_or_default()
            .split('|')
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(str::to_string)
            .collect()
    }

    fn is_ancestor_scope(
        ancestor_scope_id: &str,
        descendant_scope_id: &str,
        relationships: &[NodeTypeRelationship],
    ) -> bool {
        if ancestor_scope_id.is_empty() || descendant_scope_id.is_empty() {
            return true;
        }
        if ancestor_scope_id == descendant_scope_id {
            return true;
        }

        let mut stack = vec![ancestor_scope_id.to_string()];
        let mut seen = Vec::<String>::new();
        while let Some(current) = stack.pop() {
            if seen.iter().any(|scope_id| scope_id == &current) {
                continue;
            }
            seen.push(current.clone());
            for relationship in relationships
                .iter()
                .filter(|relationship| relationship.parent_node_type_id == current)
            {
                if relationship.child_node_type_id == descendant_scope_id {
                    return true;
                }
                stack.push(relationship.child_node_type_id.clone());
            }
        }
        false
    }

    fn scopes_are_comparable(
        left_scope_id: &str,
        right_scope_id: &str,
        relationships: &[NodeTypeRelationship],
    ) -> bool {
        left_scope_id.is_empty()
            || right_scope_id.is_empty()
            || is_ancestor_scope(left_scope_id, right_scope_id, relationships)
            || is_ancestor_scope(right_scope_id, left_scope_id, relationships)
    }

    fn is_ancestor_node(
        ancestor_node_id: &str,
        descendant_node_id: &str,
        nodes: &[NodeSummary],
    ) -> bool {
        if ancestor_node_id.is_empty() || descendant_node_id.is_empty() {
            return false;
        }
        if ancestor_node_id == descendant_node_id {
            return true;
        }

        let mut current = Some(descendant_node_id.to_string());
        while let Some(node_id) = current {
            if node_id == ancestor_node_id {
                return true;
            }
            current = nodes
                .iter()
                .find(|node| node.id == node_id)
                .and_then(|node| node.parent_node_id.clone());
        }
        false
    }

    fn node_ids_are_comparable(
        left_node_id: &str,
        right_node_id: &str,
        nodes: &[NodeSummary],
    ) -> bool {
        left_node_id == right_node_id
            || is_ancestor_node(left_node_id, right_node_id, nodes)
            || is_ancestor_node(right_node_id, left_node_id, nodes)
    }

    fn lineage_refs_are_comparable(
        left_scope_id: &str,
        left_node_ids: &[String],
        right_scope_id: &str,
        right_node_ids: &[String],
        relationships: &[NodeTypeRelationship],
        nodes: &[NodeSummary],
    ) -> bool {
        if !left_node_ids.is_empty() && !right_node_ids.is_empty() {
            return left_node_ids.iter().any(|left_node_id| {
                right_node_ids.iter().any(|right_node_id| {
                    node_ids_are_comparable(left_node_id, right_node_id, nodes)
                })
            });
        }
        scopes_are_comparable(left_scope_id, right_scope_id, relationships)
    }

    fn selected_workflow_step_lineage_refs(
        excluded_select: Option<&web_sys::HtmlSelectElement>,
    ) -> Vec<(String, Vec<String>)> {
        let Some(document) = web_sys::window().and_then(|window| window.document()) else {
            return Vec::new();
        };
        let Ok(selects) =
            document.query_selector_all("#workflow-step-rows [data-workflow-step-form-version]")
        else {
            return Vec::new();
        };

        let mut scopes = Vec::new();
        for index in 0..selects.length() {
            let Some(node) = selects.get(index) else {
                continue;
            };
            let Ok(select) = node.dyn_into::<web_sys::HtmlSelectElement>() else {
                continue;
            };
            if excluded_select
                .as_ref()
                .is_some_and(|excluded| excluded.is_same_node(Some(&select)))
            {
                continue;
            }
            if select.value().is_empty() {
                continue;
            }
            let scope_id = option_scope_id(&select);
            let assignment_node_ids = option_assignment_node_ids(&select);
            scopes.push((scope_id, assignment_node_ids));
        }
        scopes
    }

    fn refresh_workflow_step_form_options(
        relationships: &[NodeTypeRelationship],
        nodes: &[NodeSummary],
    ) {
        let Some(document) = web_sys::window().and_then(|window| window.document()) else {
            return;
        };
        let Ok(selects) =
            document.query_selector_all("#workflow-step-rows [data-workflow-step-form-version]")
        else {
            return;
        };
        for index in 0..selects.length() {
            let Some(node) = selects.get(index) else {
                continue;
            };
            let Ok(select) = node.dyn_into::<web_sys::HtmlSelectElement>() else {
                continue;
            };
            let selected_refs = selected_workflow_step_lineage_refs(Some(&select));
            let Ok(options) = select.query_selector_all("option") else {
                continue;
            };
            let mut current_value_still_allowed = select.value().is_empty();
            for option_index in 0..options.length() {
                let Some(option) = options.get(option_index) else {
                    continue;
                };
                let Ok(option) = option.dyn_into::<web_sys::HtmlOptionElement>() else {
                    continue;
                };
                let value = option.value();
                let scope_id = option
                    .get_attribute("data-scope-node-type-id")
                    .unwrap_or_default();
                let assignment_node_ids = option
                    .get_attribute("data-assignment-node-ids")
                    .unwrap_or_default()
                    .split('|')
                    .map(str::trim)
                    .filter(|value| !value.is_empty())
                    .map(str::to_string)
                    .collect::<Vec<_>>();
                let allowed = value.is_empty()
                    || selected_refs
                        .iter()
                        .all(|(selected_scope, selected_node_ids)| {
                            lineage_refs_are_comparable(
                                &scope_id,
                                &assignment_node_ids,
                                selected_scope,
                                selected_node_ids,
                                relationships,
                                nodes,
                            )
                        });
                option.set_disabled(!allowed);
                option.set_hidden(!allowed);
                if allowed && value == select.value() {
                    current_value_still_allowed = true;
                }
            }
            if !current_value_still_allowed {
                select.set_value("");
            }
        }
    }

    fn collect_workflow_steps() -> Vec<serde_json::Value> {
        let Some(document) = web_sys::window().and_then(|window| window.document()) else {
            return Vec::new();
        };
        let Ok(rows) = document.query_selector_all("#workflow-step-rows [data-workflow-step-row]")
        else {
            return Vec::new();
        };

        let mut steps = Vec::new();
        for index in 0..rows.length() {
            let Some(node) = rows.get(index) else {
                continue;
            };
            let Ok(row) = node.dyn_into::<web_sys::Element>() else {
                continue;
            };
            let form_version_id = row
                .query_selector("[data-workflow-step-form-version]")
                .ok()
                .flatten()
                .and_then(|element| element.dyn_into::<web_sys::HtmlSelectElement>().ok())
                .map(|select| select.value())
                .unwrap_or_default();
            if form_version_id.is_empty() {
                continue;
            }
            let position = steps.len() + 1;
            let title = row
                .query_selector("[data-workflow-step-title]")
                .ok()
                .flatten()
                .and_then(|element| element.dyn_into::<web_sys::HtmlInputElement>().ok())
                .map(|input| input.value().trim().to_string())
                .filter(|value| !value.is_empty())
                .unwrap_or_else(|| format!("Step {position}"));
            steps.push(json!({
                "title": title,
                "form_version_id": form_version_id,
            }));
        }
        steps
    }

    fn select_options_with_selected(version_options: &str, selected: Option<&str>) -> String {
        let Some(selected) = selected.filter(|value| !value.is_empty()) else {
            return version_options.to_string();
        };
        version_options.replacen(
            &format!(r#"value="{}""#, escape_html(selected)),
            &format!(r#"value="{}" selected"#, escape_html(selected)),
            1,
        )
    }

    fn rust_ui_icon(name: &str) -> &'static str {
        match name {
            "arrow-up" => {
                r#"<svg class="rust-ui-icon" aria-hidden="true" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M12 19V5"></path><path d="m5 12 7-7 7 7"></path></svg>"#
            }
            "arrow-down" => {
                r#"<svg class="rust-ui-icon" aria-hidden="true" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M12 5v14"></path><path d="m19 12-7 7-7-7"></path></svg>"#
            }
            "clipboard-pen" => {
                r#"<svg class="rust-ui-icon" aria-hidden="true" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M16 4h2a2 2 0 0 1 2 2v14a2 2 0 0 1-2 2h-5"></path><path d="M8 4H6a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h2"></path><rect width="8" height="4" x="8" y="2" rx="1"></rect><path d="M17.5 10.5 19 12l-6.5 6.5-3 1 1-3 7-7Z"></path></svg>"#
            }
            "eye" => {
                r#"<svg class="rust-ui-icon" aria-hidden="true" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M2 12s3.5-7 10-7 10 7 10 7-3.5 7-10 7S2 12 2 12Z"></path><circle cx="12" cy="12" r="3"></circle></svg>"#
            }
            "pencil" => {
                r#"<svg class="rust-ui-icon" aria-hidden="true" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M12 20h9"></path><path d="M16.5 3.5a2.12 2.12 0 0 1 3 3L7 19l-4 1 1-4Z"></path></svg>"#
            }
            "trash" => {
                r#"<svg class="rust-ui-icon" aria-hidden="true" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M3 6h18"></path><path d="M8 6V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2"></path><path d="m19 6-1 14a2 2 0 0 1-2 2H8a2 2 0 0 1-2-2L5 6"></path><path d="M10 11v6"></path><path d="M14 11v6"></path></svg>"#
            }
            _ => "",
        }
    }

    fn workflow_step_row_html(
        index: usize,
        title: &str,
        version_options: &str,
        selected_form_version_id: Option<&str>,
        placeholder: Option<&str>,
    ) -> String {
        let options = select_options_with_selected(version_options, selected_form_version_id);
        format!(
            r#"<tr data-workflow-step-row><td><input class="input" type="text" aria-label="Step {0} title" value="{1}" placeholder="{2}" data-workflow-step-title></td><td><select class="input workflow-step-form-select" aria-label="Step {0} linked form" data-workflow-step-form-version>{3}</select></td><td class="workflow-step-table__actions"><button class="button is-light icon-button workflow-step-icon" type="button" data-workflow-step-action="up" title="Move up" aria-label="Move step up">{4}</button><button class="button is-light icon-button workflow-step-icon" type="button" data-workflow-step-action="down" title="Move down" aria-label="Move step down">{5}</button><button class="button is-danger icon-button workflow-step-icon" type="button" data-workflow-step-action="remove" title="Remove step" aria-label="Remove step">{6}</button></td></tr>"#,
            index,
            escape_html(title),
            escape_html(placeholder.unwrap_or("")),
            options,
            rust_ui_icon("arrow-up"),
            rust_ui_icon("arrow-down"),
            rust_ui_icon("trash"),
        )
    }

    fn render_workflow_step_editor(
        version_options: &str,
        versions_html: &str,
        draft: Option<&WorkflowVersionSummary>,
    ) -> String {
        let rows = draft
            .filter(|version| !version.steps.is_empty())
            .map(|version| {
                version
                    .steps
                    .iter()
                    .enumerate()
                    .map(|(index, step)| {
                        workflow_step_row_html(
                            index + 1,
                            &step.title,
                            version_options,
                            Some(&step.form_version_id),
                            None,
                        )
                    })
                    .collect::<Vec<_>>()
                    .join("")
            })
            .unwrap_or_else(|| {
                workflow_step_row_html(1, "Primary Response", version_options, None, None)
            });
        let draft_id = draft
            .filter(|version| version.status != "published")
            .map(|version| version.id.as_str())
            .unwrap_or_default();
        format!(
            r#"<input type="hidden" id="workflow-draft-version-id" value="{}"><div class="workflow-version-feedback"><div id="workflow-version-toast" class="tessara-inline-toast" hidden></div></div><div class="table-container"><table class="data-grid workflow-step-table"><thead><tr><th>Title</th><th>Linked Form</th><th>Actions</th></tr></thead><tbody id="workflow-step-rows">{}</tbody></table></div><div class="actions workflow-version-actions form-button-container"><button class="button is-light" type="button" id="workflow-step-add">Add Step</button><button class="button is-light" type="button" id="workflow-version-save-draft">Save as Draft</button><button class="button is-primary" type="button" id="workflow-version-create">Publish Workflow Version</button></div><div id="workflow-version-list-inline" class="record-list workflow-version-list">{}</div>"#,
            escape_html(draft_id),
            rows,
            versions_html
        )
    }

    fn workflow_step_row_count() -> usize {
        web_sys::window()
            .and_then(|window| window.document())
            .and_then(|document| {
                document
                    .query_selector_all("#workflow-step-rows [data-workflow-step-row]")
                    .ok()
            })
            .map(|rows| rows.length() as usize)
            .unwrap_or(0)
    }

    fn attach_workflow_step_editor(
        version_options: String,
        relationships: Vec<NodeTypeRelationship>,
        nodes: Vec<NodeSummary>,
    ) {
        if let Some(button) = by_id("workflow-step-add") {
            let options = version_options.clone();
            let add_relationships = relationships.clone();
            let add_nodes = nodes.clone();
            let closure = Closure::wrap(Box::new(move |_event: web_sys::Event| {
                let next_index = workflow_step_row_count() + 1;
                if let Some(container) = by_id("workflow-step-rows") {
                    let _ = container.insert_adjacent_html(
                        "beforeend",
                        &workflow_step_row_html(
                            next_index,
                            "",
                            &options,
                            None,
                            Some(&format!("Step {next_index} title")),
                        ),
                    );
                }
                refresh_workflow_step_form_options(&add_relationships, &add_nodes);
            }) as Box<dyn FnMut(_)>);
            button
                .add_event_listener_with_callback("click", closure.as_ref().unchecked_ref())
                .ok();
            closure.forget();
        }

        if let Some(container) = by_id("workflow-step-rows") {
            let click_relationships = relationships.clone();
            let click_nodes = nodes.clone();
            let closure = Closure::wrap(Box::new(move |event: web_sys::Event| {
                let Some(target) = event
                    .target()
                    .and_then(|target| target.dyn_into::<web_sys::Element>().ok())
                else {
                    return;
                };
                let Some(action_element) =
                    target.closest("[data-workflow-step-action]").ok().flatten()
                else {
                    return;
                };
                let action = action_element
                    .get_attribute("data-workflow-step-action")
                    .unwrap_or_default();
                let Some(row) = action_element
                    .closest("[data-workflow-step-row]")
                    .ok()
                    .flatten()
                else {
                    return;
                };
                match action.as_str() {
                    "remove" => row.remove(),
                    "up" => {
                        if let Some(previous) = row.previous_element_sibling() {
                            if let Some(parent) = row.parent_node() {
                                let _ = parent.insert_before(&row, Some(&previous));
                            }
                        }
                    }
                    "down" => {
                        if let Some(next) = row.next_element_sibling() {
                            if let Some(parent) = row.parent_node() {
                                let _ = parent.insert_before(&next, Some(&row));
                            }
                        }
                    }
                    _ => {}
                }
                refresh_workflow_step_form_options(&click_relationships, &click_nodes);
            }) as Box<dyn FnMut(_)>);
            container
                .add_event_listener_with_callback("click", closure.as_ref().unchecked_ref())
                .ok();
            closure.forget();

            let change_relationships = relationships.clone();
            let change_nodes = nodes.clone();
            let closure = Closure::wrap(Box::new(move |_event: web_sys::Event| {
                refresh_workflow_step_form_options(&change_relationships, &change_nodes);
            }) as Box<dyn FnMut(_)>);
            container
                .add_event_listener_with_callback("change", closure.as_ref().unchecked_ref())
                .ok();
            closure.forget();
        }
        refresh_workflow_step_form_options(&relationships, &nodes);
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

    fn workflow_summary_status_label(summary: &WorkflowSummary) -> &str {
        match summary.current_status.as_deref() {
            Some("published") => "Published",
            Some("draft") => "Draft",
            Some("superseded") => "Retired",
            Some(value) => value,
            None => "Draft",
        }
    }

    fn workflow_summary_description_html(summary: &WorkflowSummary) -> String {
        format!(
            r#"<div class="workflow-directory-description"><p>{}</p><p><strong>Current Version:</strong> {}</p><p><strong>Versions:</strong> {}</p><p><strong>Active Assignments:</strong> {}</p><p><strong>Slug:</strong> {}</p></div>"#,
            escape_html(workflow_summary_description(summary)),
            escape_html(workflow_summary_version(summary)),
            summary.version_count,
            summary.assignment_count,
            escape_html(&summary.slug),
        )
    }

    fn workflow_assignment_nodes_html(summary: &WorkflowSummary) -> String {
        if summary.assignment_node_names.is_empty() {
            return r#"<span class="muted">No active assignment nodes</span>"#.into();
        }
        format!(
            r#"<ul class="workflow-directory-node-list__items">{}</ul>"#,
            summary
                .assignment_node_names
                .iter()
                .map(|node| format!(r#"<li>{}</li>"#, escape_html(node)))
                .collect::<Vec<_>>()
                .join("")
        )
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

    fn render_workflow_directory(items: &[WorkflowSummary]) -> String {
        if items.is_empty() {
            return r#"<p class="muted">No workflow records found.</p>"#.into();
        }
        let rows = items
            .iter()
            .map(|item| {
                let current_version = workflow_summary_version(item);
                let status = workflow_summary_status_label(item);
                let search_text = format!(
                    "{} {} {} {} {} {}",
                    item.name,
                    current_version,
                    workflow_summary_description(item),
                    item.slug,
                    status,
                    item.assignment_node_names.join(" ")
                )
                .to_lowercase();
                format!(
                    r#"<tr class="workflow-directory-row" data-workflow-directory-row data-workflow-directory-name="{}" data-workflow-directory-status="{}" data-workflow-directory-assignments="{}" data-workflow-directory-search="{}"><td><div class="workflow-directory-name"><strong>{}</strong><span>{}</span><span class="status-pill status-pill--{}">{}</span></div></td><td>{}</td><td><div class="workflow-directory-node-list">{}</div></td><td class="workflow-directory-actions"><a class="button-link button is-light icon-button directory-icon-button" href="/app/workflows/{}" title="View Workflow" aria-label="View Workflow">{}</a><a class="button-link button is-light icon-button directory-icon-button" href="/app/workflows/{}/edit" title="Edit" aria-label="Edit">{}</a><a class="button-link button is-primary icon-button directory-icon-button" href="{}" title="Assign" aria-label="Assign">{}</a></td></tr>"#,
                    escape_html(&item.name.to_lowercase()),
                    escape_html(&status.to_lowercase()),
                    item.assignment_count,
                    escape_html(&search_text),
                    escape_html(&item.name),
                    escape_html(current_version),
                    escape_html(&status.to_lowercase()),
                    escape_html(status),
                    workflow_summary_description_html(item),
                    workflow_assignment_nodes_html(item),
                    escape_html(&item.id),
                    rust_ui_icon("eye"),
                    escape_html(&item.id),
                    rust_ui_icon("pencil"),
                    escape_html(&assignment_console_path(Some(&item.id))),
                    rust_ui_icon("clipboard-pen"),
                )
            })
            .collect::<Vec<_>>()
            .join("");
        format!(
            r#"<section class="rust-data-table" id="workflow-directory-data-table" data-rust-data-table data-table-page="0"><div class="rust-data-table__toolbar"><div class="rust-data-table__filters"><input class="input" id="workflow-directory-search" type="search" placeholder="Search workflows"><select class="input" id="workflow-directory-status-filter" aria-label="Filter workflow status"><option value="">All statuses</option><option value="published">Published</option><option value="draft">Draft</option><option value="retired">Retired</option></select><select class="input" id="workflow-directory-sort" aria-label="Sort workflows"><option value="name">Sort by name</option><option value="status">Sort by status</option><option value="assignments">Sort by active assignments</option></select><select class="input rust-data-table__page-size" id="workflow-directory-page-size" aria-label="Rows per page"><option value="10">10 rows</option><option value="25">25 rows</option><option value="50">50 rows</option></select></div></div><div class="table-container rust-data-table__viewport"><table class="data-grid workflow-directory-table data-table-like"><thead><tr><th><button class="table-sort-button" type="button" data-workflow-directory-sort-button="name">Name</button></th><th>Description</th><th>Node</th><th>Actions</th></tr></thead><tbody id="workflow-directory-table-body">{rows}</tbody></table></div><div class="rust-data-table__footer"><p class="muted" id="workflow-directory-table-summary"></p><div class="rust-data-table__pagination"><button class="button is-light" type="button" id="workflow-directory-prev">Previous</button><button class="button is-light" type="button" id="workflow-directory-next">Next</button></div></div></section>"#
        )
    }

    fn assignment_console_path(workflow_id: Option<&str>) -> String {
        workflow_id
            .filter(|value| !value.is_empty())
            .map(|value| format!("/app/workflows/assignments?workflowId={value}"))
            .unwrap_or_else(|| "/app/workflows/assignments".into())
    }

    fn render_workflow_detail(detail: &WorkflowDefinition) -> String {
        format!(
            r#"<dl class="metadata-list"><div><dt>Name</dt><dd>{}</dd></div><div><dt>Slug</dt><dd>{}</dd></div><div><dt>Version Count</dt><dd>{}</dd></div><div><dt>Assignment Count</dt><dd>{}</dd></div></dl><p>{}</p>"#,
            escape_html(&detail.name),
            escape_html(&detail.slug),
            detail.versions.len(),
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
                        r#"<article class="record-card compact-record-card"><div class="page-title-row compact-title-row"><div><h4>{}</h4><p class="muted">{}</p></div><p class="workflow-directory-card__status">{}</p></div><div class="workflow-directory-card__meta"><p><strong>Step collection:</strong> {} step(s)</p></div></article>"#,
                        escape_html(&version.title),
                        escape_html(
                            version
                                .form_version_label
                                .as_deref()
                                .unwrap_or("Draft version")
                        ),
                        escape_html(&version.status),
                        version.step_count,
                    )
                })
                .collect::<Vec<_>>()
                .join("")
        };

        format!(
            r#"<article class="record-detail workflow-selected-detail"><div class="page-title-row compact-title-row"><div><p class="eyebrow">Selected Workflow</p><h3>{}</h3><p>{}</p></div><p class="workflow-directory-card__status">{}</p></div><div class="workflow-directory-card__meta workflow-selected-detail__meta"><p><strong>Slug:</strong> {}</p><p><strong>Assignments:</strong> {}</p><p><strong>Versions:</strong> {}</p></div><div class="actions"><a class="button-link button is-light" href="/app/workflows/{}">Open Detail</a><a class="button-link button is-light" href="/app/workflows/{}/edit">Edit Workflow</a><a class="button-link button is-primary" href="{}">Manage Assignments</a>{}</div><section class="record-detail workflow-selected-detail__section"><div><p class="eyebrow">Assignments</p><h4>Current assignment footprint</h4></div><div class="record-list workflow-selected-detail__assignments">{}</div></section><section class="record-detail workflow-selected-detail__section"><div><p class="eyebrow">Versions</p><h4>Workflow version lifecycle</h4></div><div class="record-list workflow-selected-detail__versions">{}</div></section></article>"#,
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
        r#"<div class="record-detail workflow-selected-detail"><p class="muted">Select a workflow from the directory to inspect its assignments and step-based versions.</p></div>"#.into()
    }

    fn render_versions(detail: &WorkflowDefinition, show_publish_button: bool) -> String {
        if detail.versions.is_empty() {
            return r#"<p class="muted">No workflow versions exist yet.</p>"#.into();
        }
        detail
            .versions
            .iter()
            .map(|version| {
                let publish = if version.status != "published" {
                    if show_publish_button {
                        format!(
                            r#"<button class="button is-light" type="button" data-publish-workflow-version="{}">Publish</button>"#,
                            escape_html(&version.id),
                        )
                    } else {
                        String::new()
                    }
                } else {
                    String::new()
                };
                let delete = if version.status != "published" && !show_publish_button {
                    format!(
                        r#"<button class="button is-danger icon-button workflow-version-delete-button" type="button" data-delete-workflow-version="{}" title="Delete draft" aria-label="Delete draft">{}</button>"#,
                        escape_html(&version.id),
                        rust_ui_icon("trash")
                    )
                } else {
                    String::new()
                };
                let steps = if version.steps.is_empty() {
                    format!(r#"<p class="muted">{}</p>"#, escape_html(&version.title))
                } else {
                    version
                        .steps
                        .iter()
                        .map(|step| {
                            format!(
                                r#"<li><strong>{}</strong><span>{} - {}</span></li>"#,
                                escape_html(&step.title),
                                escape_html(&step.form_name),
                                escape_html(
                                    step.form_version_label
                                        .as_deref()
                                        .unwrap_or("Published version")
                                )
                            )
                        })
                        .collect::<Vec<_>>()
                        .join("")
                };
                format!(
                    r#"<article class="record-card workflow-version-card"><div class="workflow-version-card__delete">{}</div><h4>{}</h4><p class="muted">{} steps - Status: {}</p><ul class="app-list">{}</ul><div class="actions workflow-version-card__actions">{}</div></article>"#,
                    delete,
                    escape_html(version.form_version_label.as_deref().unwrap_or("Draft")),
                    version.step_count,
                    escape_html(&version.status),
                    steps,
                    publish
                )
            })
            .collect::<Vec<_>>()
            .join("")
    }

    fn publish_failure_message(error: &str) -> String {
        if error.contains(
            "workflow step form is not compatible with the assignment node or descendants",
        ) {
            "Publish failed: every step form must be compatible with the selected assignment node or one of its descendant nodes.".into()
        } else if error.contains("workflow step form scopes must stay on one hierarchy lineage") {
            "Publish failed: workflow step forms must stay on one straight hierarchy lineage without branching.".into()
        } else if error
            .contains("workflow step form assignments must stay on one hierarchy lineage")
        {
            "Publish failed: selected step forms must stay on one concrete node lineage; sibling node forms cannot be composed in the same workflow.".into()
        } else {
            format!("Publish failed: {error}")
        }
    }

    fn show_workflow_version_toast(message: &str) {
        set_text("workflow-version-toast", message);
        if let Some(toast) = by_id("workflow-version-toast") {
            toast.remove_attribute("hidden").ok();
        }
    }

    fn confirm_draft_delete() -> bool {
        web_sys::window()
            .and_then(|window| {
                window
                    .confirm_with_message(
                        "Are you sure you want to delete this draft workflow version?",
                    )
                    .ok()
            })
            .unwrap_or(false)
    }

    async fn refresh_workflow_version_list(workflow_id: &str) -> Result<(), String> {
        let detail =
            get_json::<WorkflowDefinition>(&format!("/api/workflows/{workflow_id}")).await?;
        set_html(
            "workflow-version-list-inline",
            &render_versions(&detail, false),
        );
        Ok(())
    }

    fn attach_workflow_version_delete_handler(workflow_id: String) {
        let Some(container) = by_id("workflow-version-list-inline") else {
            return;
        };
        let closure = Closure::wrap(Box::new(move |event: web_sys::Event| {
            let Some(target) = event
                .target()
                .and_then(|target| target.dyn_into::<web_sys::Element>().ok())
            else {
                return;
            };
            let Some(button) = target
                .closest("[data-delete-workflow-version]")
                .ok()
                .flatten()
            else {
                return;
            };
            event.prevent_default();
            if !confirm_draft_delete() {
                return;
            }
            let Some(version_id) = button.get_attribute("data-delete-workflow-version") else {
                return;
            };
            let workflow_id = workflow_id.clone();
            spawn_local(async move {
                match delete_json::<IdResponse>(&format!("/api/workflow-versions/{version_id}"))
                    .await
                {
                    Ok(_) => {
                        set_input_value("workflow-draft-version-id", "");
                        match refresh_workflow_version_list(&workflow_id).await {
                            Ok(_) => show_workflow_version_toast("Draft deleted."),
                            Err(error) => set_text("workflow-form-status", &error),
                        }
                    }
                    Err(error) => set_text("workflow-form-status", &error),
                }
            });
        }) as Box<dyn FnMut(_)>);
        container
            .add_event_listener_with_callback("click", closure.as_ref().unchecked_ref())
            .ok();
        closure.forget();
    }

    fn render_assignment_rows(items: &[AssignmentSummary]) -> String {
        if items.is_empty() {
            return r#"<p class="muted">No workflow assignments found for the current filters.</p>"#.into();
        }
        let rows = items
            .iter()
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
                let search_text = format!(
                    "{} {} {} {} {} {}",
                    item.workflow_name,
                    item.account_display_name,
                    item.account_email,
                    item.node_name,
                    work_state,
                    state
                )
                .to_lowercase();
                format!(
                    r#"<tr data-workflow-assignment-row data-workflow-assignment-search="{}" data-workflow-assignment-status="{}" data-workflow-assignment-work-state="{}" data-workflow-assignment-workflow="{}" data-workflow-assignment-assignee="{}" data-workflow-assignment-node="{}"><td><strong>{}</strong></td><td>{}</td><td>{}</td><td>{}</td><td>{}</td><td><button class="button is-light" type="button" data-toggle-workflow-assignment="{}">{}</button></td></tr>"#,
                    escape_html(&search_text),
                    escape_html(&state.to_lowercase()),
                    escape_html(&work_state.to_lowercase()),
                    escape_html(&item.workflow_name.to_lowercase()),
                    escape_html(&item.account_display_name.to_lowercase()),
                    escape_html(&item.node_name.to_lowercase()),
                    escape_html(&item.workflow_name),
                    escape_html(&item.account_display_name),
                    escape_html(&item.node_name),
                    escape_html(work_state),
                    escape_html(state),
                    item.id,
                    toggle_label
                )
            })
            .collect::<Vec<_>>()
            .join("");
        format!(
            r#"<section class="rust-data-table" id="workflow-assignment-data-table" data-rust-data-table data-table-page="0"><div class="rust-data-table__toolbar"><div class="rust-data-table__filters workflow-assignment-directory-controls"><input class="input" id="workflow-assignment-directory-search" type="search" placeholder="Search assignments"><select class="input" id="workflow-assignment-directory-status-filter" aria-label="Filter assignment status"><option value="">All statuses</option><option value="active">Active</option><option value="inactive">Inactive</option></select><select class="input" id="workflow-assignment-directory-work-state-filter" aria-label="Filter work state"><option value="">All work states</option><option value="pending">Pending</option><option value="draft exists">Draft exists</option><option value="submitted">Submitted</option></select><select class="input" id="workflow-assignment-directory-sort" aria-label="Sort assignments"><option value="workflow">Sort by workflow</option><option value="assignee">Sort by assignee</option><option value="node">Sort by node</option><option value="status">Sort by status</option></select><select class="input rust-data-table__page-size" id="workflow-assignment-directory-page-size" aria-label="Rows per page"><option value="10">10 rows</option><option value="25">25 rows</option><option value="50">50 rows</option></select></div></div><div class="table-container rust-data-table__viewport"><table class="data-grid workflow-assignment-table data-table-like"><thead><tr><th><button class="table-sort-button" type="button" data-workflow-assignment-sort-button="workflow">Workflow</button></th><th><button class="table-sort-button" type="button" data-workflow-assignment-sort-button="assignee">Assignee</button></th><th><button class="table-sort-button" type="button" data-workflow-assignment-sort-button="node">Node</button></th><th>Work State</th><th>Status</th><th>Actions</th></tr></thead><tbody id="workflow-assignment-table-body">{rows}</tbody></table></div><div class="rust-data-table__footer"><p class="muted" id="workflow-assignment-table-summary"></p><div class="rust-data-table__pagination"><button class="button is-light" type="button" id="workflow-assignment-directory-prev">Previous</button><button class="button is-light" type="button" id="workflow-assignment-directory-next">Next</button></div></div></section>"#
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

    fn data_table_page(container_id: &str) -> usize {
        by_id(container_id)
            .and_then(|element| element.get_attribute("data-table-page"))
            .and_then(|value| value.parse::<usize>().ok())
            .unwrap_or(0)
    }

    fn set_data_table_page(container_id: &str, page: usize) {
        if let Some(element) = by_id(container_id) {
            let _ = element.set_attribute("data-table-page", &page.to_string());
        }
    }

    fn data_table_page_size(select_id: &str) -> usize {
        select_value(select_id)
            .and_then(|value| value.parse::<usize>().ok())
            .filter(|value| *value > 0)
            .unwrap_or(10)
    }

    fn set_button_disabled(button_id: &str, disabled: bool) {
        if let Some(button) = by_id(button_id) {
            if disabled {
                let _ = button.set_attribute("disabled", "");
            } else {
                button.remove_attribute("disabled").ok();
            }
        }
    }

    fn apply_workflow_directory_filters() {
        let Some(document) = web_sys::window().and_then(|window| window.document()) else {
            return;
        };
        let query = input_value("workflow-directory-search")
            .unwrap_or_default()
            .trim()
            .to_lowercase();
        let status = select_value("workflow-directory-status-filter").unwrap_or_default();
        let sort = select_value("workflow-directory-sort").unwrap_or_else(|| "name".into());
        let page_size = data_table_page_size("workflow-directory-page-size");
        let mut page = data_table_page("workflow-directory-data-table");
        let Some(body) = by_id("workflow-directory-table-body") else {
            return;
        };
        let Ok(nodes) = document.query_selector_all("[data-workflow-directory-row]") else {
            return;
        };
        let mut rows = Vec::new();
        for index in 0..nodes.length() {
            let Some(node) = nodes.get(index) else {
                continue;
            };
            let Ok(row) = node.dyn_into::<web_sys::Element>() else {
                continue;
            };
            let search = row
                .get_attribute("data-workflow-directory-search")
                .unwrap_or_default();
            let row_status = row
                .get_attribute("data-workflow-directory-status")
                .unwrap_or_default();
            let visible = (query.is_empty() || search.contains(&query))
                && (status.is_empty() || status == row_status);
            let key = match sort.as_str() {
                "status" => row_status,
                "assignments" => format!(
                    "{:08}",
                    99999999_i64
                        - row
                            .get_attribute("data-workflow-directory-assignments")
                            .and_then(|value| value.parse::<i64>().ok())
                            .unwrap_or(0)
                ),
                _ => row
                    .get_attribute("data-workflow-directory-name")
                    .unwrap_or_default(),
            };
            rows.push((key, visible, row));
        }
        rows.sort_by(|left, right| left.0.cmp(&right.0));
        let visible_count = rows.iter().filter(|(_, visible, _)| *visible).count();
        let total_pages = visible_count.div_ceil(page_size).max(1);
        if page >= total_pages {
            page = total_pages - 1;
            set_data_table_page("workflow-directory-data-table", page);
        }
        let start = page * page_size;
        let end = (start + page_size).min(visible_count);
        let mut visible_index = 0;
        for (_, visible, row) in rows {
            if visible && visible_index >= start && visible_index < end {
                row.remove_attribute("hidden").ok();
            } else {
                row.set_attribute("hidden", "").ok();
            }
            if visible {
                visible_index += 1;
            }
            let _ = body.append_child(&row);
        }
        if visible_count == 0 {
            set_text(
                "workflow-directory-table-summary",
                "No workflows match the current filters.",
            );
        } else {
            set_text(
                "workflow-directory-table-summary",
                &format!(
                    "Showing {}-{} of {} workflows",
                    start + 1,
                    end,
                    visible_count
                ),
            );
        }
        set_button_disabled("workflow-directory-prev", page == 0);
        set_button_disabled("workflow-directory-next", page + 1 >= total_pages);
    }

    fn bind_workflow_directory_controls() {
        for id in [
            "workflow-directory-search",
            "workflow-directory-status-filter",
            "workflow-directory-sort",
            "workflow-directory-page-size",
        ] {
            if let Some(element) = by_id(id) {
                let closure = Closure::wrap(Box::new(move |_event: web_sys::Event| {
                    set_data_table_page("workflow-directory-data-table", 0);
                    apply_workflow_directory_filters();
                }) as Box<dyn FnMut(_)>);
                element
                    .add_event_listener_with_callback("input", closure.as_ref().unchecked_ref())
                    .ok();
                element
                    .add_event_listener_with_callback("change", closure.as_ref().unchecked_ref())
                    .ok();
                closure.forget();
            }
        }
        if let Some(document) = web_sys::window().and_then(|window| window.document()) {
            if let Ok(buttons) =
                document.query_selector_all("[data-workflow-directory-sort-button]")
            {
                for index in 0..buttons.length() {
                    let Some(node) = buttons.get(index) else {
                        continue;
                    };
                    let Ok(button) = node.dyn_into::<web_sys::Element>() else {
                        continue;
                    };
                    let Some(sort) = button.get_attribute("data-workflow-directory-sort-button")
                    else {
                        continue;
                    };
                    let closure = Closure::wrap(Box::new(move |_event: web_sys::Event| {
                        set_select_value("workflow-directory-sort", &sort);
                        set_data_table_page("workflow-directory-data-table", 0);
                        apply_workflow_directory_filters();
                    }) as Box<dyn FnMut(_)>);
                    button
                        .add_event_listener_with_callback("click", closure.as_ref().unchecked_ref())
                        .ok();
                    closure.forget();
                }
            }
        }
        if let Some(previous) = by_id("workflow-directory-prev") {
            let closure = Closure::wrap(Box::new(move |_event: web_sys::Event| {
                let page = data_table_page("workflow-directory-data-table");
                set_data_table_page("workflow-directory-data-table", page.saturating_sub(1));
                apply_workflow_directory_filters();
            }) as Box<dyn FnMut(_)>);
            previous
                .add_event_listener_with_callback("click", closure.as_ref().unchecked_ref())
                .ok();
            closure.forget();
        }
        if let Some(next) = by_id("workflow-directory-next") {
            let closure = Closure::wrap(Box::new(move |_event: web_sys::Event| {
                let page = data_table_page("workflow-directory-data-table");
                set_data_table_page("workflow-directory-data-table", page + 1);
                apply_workflow_directory_filters();
            }) as Box<dyn FnMut(_)>);
            next.add_event_listener_with_callback("click", closure.as_ref().unchecked_ref())
                .ok();
            closure.forget();
        }
        apply_workflow_directory_filters();
    }

    fn apply_assignment_directory_filters() {
        let Some(document) = web_sys::window().and_then(|window| window.document()) else {
            return;
        };
        let query = input_value("workflow-assignment-directory-search")
            .unwrap_or_default()
            .trim()
            .to_lowercase();
        let status =
            select_value("workflow-assignment-directory-status-filter").unwrap_or_default();
        let work_state =
            select_value("workflow-assignment-directory-work-state-filter").unwrap_or_default();
        let sort =
            select_value("workflow-assignment-directory-sort").unwrap_or_else(|| "workflow".into());
        let page_size = data_table_page_size("workflow-assignment-directory-page-size");
        let mut page = data_table_page("workflow-assignment-data-table");
        let Some(body) = by_id("workflow-assignment-table-body") else {
            return;
        };
        let Ok(nodes) = document.query_selector_all("[data-workflow-assignment-row]") else {
            return;
        };
        let mut rows = Vec::new();
        for index in 0..nodes.length() {
            let Some(node) = nodes.get(index) else {
                continue;
            };
            let Ok(row) = node.dyn_into::<web_sys::Element>() else {
                continue;
            };
            let search = row
                .get_attribute("data-workflow-assignment-search")
                .unwrap_or_default();
            let row_status = row
                .get_attribute("data-workflow-assignment-status")
                .unwrap_or_default();
            let row_work_state = row
                .get_attribute("data-workflow-assignment-work-state")
                .unwrap_or_default();
            let visible = (query.is_empty() || search.contains(&query))
                && (status.is_empty() || status == row_status)
                && (work_state.is_empty() || work_state == row_work_state);
            let key_attr = match sort.as_str() {
                "assignee" => "data-workflow-assignment-assignee",
                "node" => "data-workflow-assignment-node",
                "status" => "data-workflow-assignment-status",
                _ => "data-workflow-assignment-workflow",
            };
            rows.push((
                row.get_attribute(key_attr).unwrap_or_default(),
                visible,
                row,
            ));
        }
        rows.sort_by(|left, right| left.0.cmp(&right.0));
        let visible_count = rows.iter().filter(|(_, visible, _)| *visible).count();
        let total_pages = visible_count.div_ceil(page_size).max(1);
        if page >= total_pages {
            page = total_pages - 1;
            set_data_table_page("workflow-assignment-data-table", page);
        }
        let start = page * page_size;
        let end = (start + page_size).min(visible_count);
        let mut visible_index = 0;
        for (_, visible, row) in rows {
            if visible && visible_index >= start && visible_index < end {
                row.remove_attribute("hidden").ok();
            } else {
                row.set_attribute("hidden", "").ok();
            }
            if visible {
                visible_index += 1;
            }
            let _ = body.append_child(&row);
        }
        if visible_count == 0 {
            set_text(
                "workflow-assignment-table-summary",
                "No assignments match the current filters.",
            );
        } else {
            set_text(
                "workflow-assignment-table-summary",
                &format!(
                    "Showing {}-{} of {} assignments",
                    start + 1,
                    end,
                    visible_count
                ),
            );
        }
        set_button_disabled("workflow-assignment-directory-prev", page == 0);
        set_button_disabled(
            "workflow-assignment-directory-next",
            page + 1 >= total_pages,
        );
    }

    fn bind_assignment_directory_controls() {
        for id in [
            "workflow-assignment-directory-search",
            "workflow-assignment-directory-status-filter",
            "workflow-assignment-directory-work-state-filter",
            "workflow-assignment-directory-sort",
            "workflow-assignment-directory-page-size",
        ] {
            if let Some(element) = by_id(id) {
                let closure = Closure::wrap(Box::new(move |_event: web_sys::Event| {
                    set_data_table_page("workflow-assignment-data-table", 0);
                    apply_assignment_directory_filters();
                }) as Box<dyn FnMut(_)>);
                element
                    .add_event_listener_with_callback("input", closure.as_ref().unchecked_ref())
                    .ok();
                element
                    .add_event_listener_with_callback("change", closure.as_ref().unchecked_ref())
                    .ok();
                closure.forget();
            }
        }
        if let Some(document) = web_sys::window().and_then(|window| window.document()) {
            if let Ok(buttons) =
                document.query_selector_all("[data-workflow-assignment-sort-button]")
            {
                for index in 0..buttons.length() {
                    let Some(node) = buttons.get(index) else {
                        continue;
                    };
                    let Ok(button) = node.dyn_into::<web_sys::Element>() else {
                        continue;
                    };
                    let Some(sort) = button.get_attribute("data-workflow-assignment-sort-button")
                    else {
                        continue;
                    };
                    let closure = Closure::wrap(Box::new(move |_event: web_sys::Event| {
                        set_select_value("workflow-assignment-directory-sort", &sort);
                        set_data_table_page("workflow-assignment-data-table", 0);
                        apply_assignment_directory_filters();
                    }) as Box<dyn FnMut(_)>);
                    button
                        .add_event_listener_with_callback("click", closure.as_ref().unchecked_ref())
                        .ok();
                    closure.forget();
                }
            }
        }
        if let Some(previous) = by_id("workflow-assignment-directory-prev") {
            let closure = Closure::wrap(Box::new(move |_event: web_sys::Event| {
                let page = data_table_page("workflow-assignment-data-table");
                set_data_table_page("workflow-assignment-data-table", page.saturating_sub(1));
                apply_assignment_directory_filters();
            }) as Box<dyn FnMut(_)>);
            previous
                .add_event_listener_with_callback("click", closure.as_ref().unchecked_ref())
                .ok();
            closure.forget();
        }
        if let Some(next) = by_id("workflow-assignment-directory-next") {
            let closure = Closure::wrap(Box::new(move |_event: web_sys::Event| {
                let page = data_table_page("workflow-assignment-data-table");
                set_data_table_page("workflow-assignment-data-table", page + 1);
                apply_assignment_directory_filters();
            }) as Box<dyn FnMut(_)>);
            next.add_event_listener_with_callback("click", closure.as_ref().unchecked_ref())
                .ok();
            closure.forget();
        }
        apply_assignment_directory_filters();
    }

    fn filter_assignment_candidate_options() {
        let query = input_value("workflow-assignment-candidate-search")
            .unwrap_or_default()
            .trim()
            .to_lowercase();
        let Some(document) = web_sys::window().and_then(|window| window.document()) else {
            return;
        };
        let Ok(options) = document.query_selector_all("#workflow-assignment-candidate option")
        else {
            return;
        };
        for index in 0..options.length() {
            let Some(node) = options.get(index) else {
                continue;
            };
            let Ok(option) = node.dyn_into::<web_sys::Element>() else {
                continue;
            };
            let text = option.text_content().unwrap_or_default().to_lowercase();
            if query.is_empty()
                || text.contains(&query)
                || option.get_attribute("value").unwrap_or_default().is_empty()
            {
                option.remove_attribute("hidden").ok();
            } else {
                option.set_attribute("hidden", "").ok();
            }
        }
    }

    fn bind_assignment_candidate_search() {
        if let Some(element) = by_id("workflow-assignment-candidate-search") {
            let closure = Closure::wrap(Box::new(move |_event: web_sys::Event| {
                filter_assignment_candidate_options();
            }) as Box<dyn FnMut(_)>);
            element
                .add_event_listener_with_callback("input", closure.as_ref().unchecked_ref())
                .ok();
            closure.forget();
        }
    }

    fn render_workflow_directory_surface(
        items: Vec<WorkflowSummary>,
        requested_workflow_id: Option<String>,
    ) {
        set_html(
            "workflow-directory-metrics",
            &render_workflow_directory_metrics(&items),
        );
        set_html("workflow-list", &render_workflow_directory(&items));
        bind_workflow_directory_controls();
        if requested_workflow_id.is_some() {
            replace_workflow_list_location(requested_workflow_id.as_deref());
        }
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
                }
            }
        });
    }

    pub fn load_create_page() {
        attach_submit_handler("workflow-form", move || {
            spawn_local(async move {
                let existing_id = input_value("workflow-created-id").unwrap_or_default();
                let payload = json!({
                    "name": input_value("workflow-name").unwrap_or_default(),
                    "slug": input_value("workflow-slug").unwrap_or_default(),
                    "description": textarea_value("workflow-description").unwrap_or_default(),
                });
                let result = if existing_id.is_empty() {
                    post_json::<IdResponse>("/api/workflows", &payload).await
                } else {
                    put_json::<IdResponse>(&format!("/api/workflows/{existing_id}"), &payload).await
                };
                match result {
                    Ok(response) => {
                        let workflow_id = if existing_id.is_empty() {
                            response.id
                        } else {
                            existing_id
                        };
                        set_input_value("workflow-created-id", &workflow_id);
                        set_text("workflow-submit-button", "Save Workflow");
                        set_text("workflow-form-status", "Workflow saved.");
                        if let Some(panel) = by_id("workflow-created-version-panel") {
                            panel.remove_attribute("hidden").ok();
                            panel.scroll_into_view();
                        }
                        if let Some(window) = web_sys::window() {
                            if let Ok(history) = window.history() {
                                let _ = history.replace_state_with_url(
                                    &JsValue::NULL,
                                    "",
                                    Some(&format!(
                                        "/app/workflows/{workflow_id}/edit#workflow-version-editor"
                                    )),
                                );
                            }
                        }
                        load_edit_page(workflow_id);
                    }
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
                    set_html("workflow-version-list", &render_versions(&detail, true));
                    let workflow_id_for_publish = workflow_id.clone();
                    attach_click_handler_by_attr(
                        "data-publish-workflow-version",
                        move |version_id| {
                            let workflow_id = workflow_id_for_publish.clone();
                            spawn_local(async move {
                                match post_json::<IdResponse>(
                                    &format!("/api/workflow-versions/{version_id}/publish"),
                                    &json!({}),
                                )
                                .await
                                {
                                    Ok(_) => redirect(&format!("/app/workflows/{workflow_id}")),
                                    Err(error) => set_html(
                                        "workflow-version-list",
                                        &format!(
                                            r#"<p class="muted">{}</p>"#,
                                            escape_html(&publish_failure_message(&error))
                                        ),
                                    ),
                                }
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
            let relationships =
                get_json::<Vec<NodeTypeRelationship>>("/api/admin/node-type-relationships").await;
            let nodes = get_json::<Vec<NodeSummary>>("/api/nodes").await;
            let detail =
                get_json::<WorkflowDefinition>(&format!("/api/workflows/{workflow_id_for_load}"))
                    .await;
            match (forms, relationships, nodes, detail) {
                (Ok(forms), Ok(relationships), Ok(nodes), Ok(detail)) => {
                    set_input_value("workflow-name", &detail.name);
                    set_input_value("workflow-slug", &detail.slug);
                    set_textarea_value("workflow-description", &detail.description);

                    let version_options = published_form_version_options(&forms);
                    let editable_draft = detail
                        .versions
                        .iter()
                        .find(|version| version.status != "published");
                    let authoring_source = editable_draft.or_else(|| {
                        detail
                            .versions
                            .iter()
                            .find(|version| version.status == "published")
                    });
                    set_html(
                        "workflow-version-editor",
                        &render_workflow_step_editor(
                            &version_options,
                            &render_versions(&detail, false),
                            authoring_source,
                        ),
                    );
                    attach_workflow_step_editor(version_options, relationships, nodes);
                    let workflow_id_for_publish = workflow_id.clone();
                    attach_click_handler_by_attr(
                        "data-publish-workflow-version",
                        move |version_id| {
                            let workflow_id = workflow_id_for_publish.clone();
                            spawn_local(async move {
                                match post_json::<IdResponse>(
                                    &format!("/api/workflow-versions/{version_id}/publish"),
                                    &json!({}),
                                )
                                .await
                                {
                                    Ok(_) => redirect(&format!("/app/workflows/{workflow_id}")),
                                    Err(error) => set_text(
                                        "workflow-form-status",
                                        &publish_failure_message(&error),
                                    ),
                                }
                            });
                        },
                    );
                    attach_workflow_version_delete_handler(workflow_id.clone());
                    if let Some(button) = by_id("workflow-version-save-draft") {
                        let workflow_id = workflow_id.clone();
                        let closure = Closure::wrap(Box::new(move |_event: web_sys::Event| {
                            let workflow_id = workflow_id.clone();
                            spawn_local(async move {
                                let steps = collect_workflow_steps();
                                let draft_id =
                                    input_value("workflow-draft-version-id").unwrap_or_default();
                                let payload = json!({ "steps": steps });
                                let result = if draft_id.is_empty() {
                                    post_json::<IdResponse>(
                                        &format!("/api/workflows/{workflow_id}/versions"),
                                        &payload,
                                    )
                                    .await
                                } else {
                                    put_json::<IdResponse>(
                                        &format!("/api/workflow-versions/{draft_id}/steps"),
                                        &payload,
                                    )
                                    .await
                                };
                                match result {
                                    Ok(response) => {
                                        set_input_value("workflow-draft-version-id", &response.id);
                                        match refresh_workflow_version_list(&workflow_id).await {
                                            Ok(_) => show_workflow_version_toast("Draft saved."),
                                            Err(error) => set_text("workflow-form-status", &error),
                                        }
                                    }
                                    Err(error) => set_text("workflow-form-status", &error),
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
                    if let Some(button) = by_id("workflow-version-create") {
                        let workflow_id = workflow_id.clone();
                        let closure = Closure::wrap(Box::new(move |_event: web_sys::Event| {
                            let workflow_id = workflow_id.clone();
                            spawn_local(async move {
                                let steps = collect_workflow_steps();
                                let draft_id =
                                    input_value("workflow-draft-version-id").unwrap_or_default();
                                let payload = json!({ "steps": steps });
                                let version_result = if draft_id.is_empty() {
                                    post_json::<IdResponse>(
                                        &format!("/api/workflows/{workflow_id}/versions"),
                                        &payload,
                                    )
                                    .await
                                } else {
                                    put_json::<IdResponse>(
                                        &format!("/api/workflow-versions/{draft_id}/steps"),
                                        &payload,
                                    )
                                    .await
                                };
                                match version_result {
                                    Ok(response) => {
                                        match post_json::<IdResponse>(
                                            &format!(
                                                "/api/workflow-versions/{}/publish",
                                                response.id
                                            ),
                                            &json!({}),
                                        )
                                        .await
                                        {
                                            Ok(_) => {
                                                redirect(&format!("/app/workflows/{workflow_id}"))
                                            }
                                            Err(error) => set_text(
                                                "workflow-form-status",
                                                &publish_failure_message(&error),
                                            ),
                                        }
                                    }
                                    Err(error) => set_text("workflow-form-status", &error),
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
                | (_, _, _, Err(error)) => set_text("workflow-form-status", &error),
            }
        });

        attach_submit_handler("workflow-form", move || {
            let workflow_id = workflow_id_for_form.clone();
            spawn_local(async move {
                let payload = json!({
                    "name": input_value("workflow-name").unwrap_or_default(),
                    "slug": input_value("workflow-slug").unwrap_or_default(),
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
            let workflow_id = current_search_param("workflowId");
            let form_id = current_search_param("formId");
            let node_id = current_search_param("nodeId");
            let candidate_path = node_id
                .as_ref()
                .map(|node_id| format!("/api/workflow-assignment-candidates?node_id={node_id}"))
                .unwrap_or_else(|| "/api/workflow-assignment-candidates".into());
            let candidates = get_json::<Vec<AssignmentCandidate>>(&candidate_path).await;
            let mut filter = String::from("/api/workflow-assignments");
            if let Some(workflow_id) = workflow_id.as_ref() {
                filter.push_str(&format!("?workflow_id={workflow_id}"));
            } else if let Some(form_id) = form_id.as_ref() {
                filter.push_str(&format!("?form_id={form_id}"));
            } else if let Some(node_id) = node_id.as_ref() {
                filter.push_str(&format!("?node_id={node_id}"));
            }
            let assignments = get_json::<Vec<AssignmentSummary>>(&filter).await;
            match (candidates, assignments) {
                (Ok(candidates), Ok(assignments)) => {
                    let visible_candidates = candidates
                        .iter()
                        .filter(|candidate| {
                            workflow_id
                                .as_ref()
                                .map(|workflow_id| candidate.workflow_id == *workflow_id)
                                .unwrap_or(true)
                        })
                        .cloned()
                        .collect::<Vec<_>>();
                    set_html(
                        "workflow-assignment-toolbar",
                        &format!(
                            r#"<p id="workflow-assignment-status" class="muted"></p><div class="ui-toolbar workflow-assignment-toolbar"><div class="ui-toolbar__primary"><div class="form-grid workflow-assignment-toolbar__grid"><div class="form-field wide-field"><label for="workflow-assignment-candidate">Node path - Workflow</label><input class="input" id="workflow-assignment-candidate-search" type="search" placeholder="Search node path or workflow"{}><select class="input" id="workflow-assignment-candidate"{}>{}</select></div><div class="form-field wide-field"><label>Assignees</label><div id="workflow-assignment-assignees" class="workflow-assignment-assignees"><div class="workflow-assignee-picker workflow-assignee-picker--empty"><p class="muted">Choose a workflow candidate first</p></div></div></div></div></div><div class="ui-toolbar__secondary workflow-assignment-toolbar__actions form-button-container"><button class="button is-primary" type="button" id="workflow-assignment-create">Create Assignments</button></div></div>"#,
                            if workflow_id.is_some() {
                                " disabled"
                            } else {
                                ""
                            },
                            if workflow_id.is_some() {
                                " disabled"
                            } else {
                                ""
                            },
                            render_assignment_candidate_options(&visible_candidates),
                        ),
                    );
                    if node_id.is_some() || workflow_id.is_some() {
                        if let Some(candidate) = visible_candidates.first() {
                            set_select_value(
                                "workflow-assignment-candidate",
                                &candidate_value(candidate),
                            );
                            load_assignees_for_assignment_candidate();
                        }
                    }
                    set_html(
                        "workflow-assignment-list",
                        &render_assignment_rows(&assignments),
                    );
                    bind_assignment_directory_controls();
                    bind_assignment_candidate_search();
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
                    if let Some(candidate_select) = by_id("workflow-assignment-candidate") {
                        let closure = Closure::wrap(Box::new(move |_event: web_sys::Event| {
                            load_assignees_for_assignment_candidate();
                        })
                            as Box<dyn FnMut(_)>);
                        candidate_select
                            .add_event_listener_with_callback(
                                "change",
                                closure.as_ref().unchecked_ref(),
                            )
                            .ok();
                        closure.forget();
                    }
                    if let Some(assignees) = by_id("workflow-assignment-assignees") {
                        let closure = Closure::wrap(Box::new(move |event: web_sys::Event| {
                            let Some(target) = event
                                .target()
                                .and_then(|target| target.dyn_into::<web_sys::Element>().ok())
                            else {
                                return;
                            };
                            let Some(button) =
                                target.closest("[data-assignee-chip-remove]").ok().flatten()
                            else {
                                return;
                            };
                            event.prevent_default();
                            let Some(account_id) =
                                button.get_attribute("data-assignee-chip-remove")
                            else {
                                return;
                            };
                            if let Some(document) =
                                web_sys::window().and_then(|window| window.document())
                            {
                                if let Ok(Some(input)) = document.query_selector(&format!(
                                    r#"[data-assignee-option][value="{}"]"#,
                                    account_id
                                )) {
                                    if let Ok(input) = input.dyn_into::<web_sys::HtmlInputElement>()
                                    {
                                        input.set_checked(false);
                                        refresh_assignee_chips();
                                    }
                                }
                            }
                        })
                            as Box<dyn FnMut(_)>);
                        assignees
                            .add_event_listener_with_callback(
                                "click",
                                closure.as_ref().unchecked_ref(),
                            )
                            .ok();
                        closure.forget();

                        let closure = Closure::wrap(Box::new(move |_event: web_sys::Event| {
                            refresh_assignee_chips();
                        })
                            as Box<dyn FnMut(_)>);
                        assignees
                            .add_event_listener_with_callback(
                                "change",
                                closure.as_ref().unchecked_ref(),
                            )
                            .ok();
                        closure.forget();
                        let closure = Closure::wrap(Box::new(move |_event: web_sys::Event| {
                            filter_assignee_options();
                        })
                            as Box<dyn FnMut(_)>);
                        assignees
                            .add_event_listener_with_callback(
                                "input",
                                closure.as_ref().unchecked_ref(),
                            )
                            .ok();
                        closure.forget();
                    }
                    if let Some(button) = by_id("workflow-assignment-create") {
                        let closure = Closure::wrap(Box::new(move |_event: web_sys::Event| {
                            spawn_local(async move {
                                let candidate = select_value("workflow-assignment-candidate")
                                    .unwrap_or_default();
                                let mut parts = candidate.split('|');
                                let workflow_version_id = parts.next().unwrap_or_default();
                                let node_id = parts.next().unwrap_or_default();
                                let account_ids = selected_assignee_values();
                                let payload = json!({
                                    "workflow_version_id": workflow_version_id,
                                    "node_id": node_id,
                                    "account_ids": account_ids,
                                });
                                match post_json::<BulkAssignmentResponse>(
                                    "/api/workflow-assignments/bulk",
                                    &payload,
                                )
                                .await
                                {
                                    Ok(response) => {
                                        let created = response
                                            .results
                                            .iter()
                                            .filter(|item| item.status == "created")
                                            .count();
                                        let reactivated = response
                                            .results
                                            .iter()
                                            .filter(|item| item.status == "reactivated")
                                            .count();
                                        let skipped = response
                                            .results
                                            .iter()
                                            .filter(|item| item.status == "skipped")
                                            .count();
                                        set_text(
                                            "workflow-assignment-status",
                                            &format!(
                                                "{created} created, {reactivated} reactivated, {skipped} already active."
                                            ),
                                        );
                                        load_assignment_page();
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
                (Err(error), _) | (_, Err(error)) => {
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
            title="Workflows"
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
            <MetadataStrip items=vec![
                ("Mode", "Directory".into()),
                ("Surface", "Workflow directory".into()),
                ("State", "Loading workflow records".into()),
            ]/>
            <section class="app-screen box workflow-directory-screen">
                <div id="workflow-directory-metrics" class="record-detail workflow-directory-overview">
                    <p class="muted">"Loading workflow summary..."</p>
                </div>
                <section class="record-detail workflow-directory-panel">
                    <div class="page-title-row compact-title-row workflow-directory-toolbar">
                        <div></div>
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
            </section>
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
                description="Create workflow metadata, then publish an ordered collection of step forms."
            />
            <MetadataStrip items=vec![
                ("Mode", "Create".into()),
                ("Surface", "Workflow authoring".into()),
                ("State", "Metadata entry".into()),
            ]/>
            <section class="app-screen box workflow-authoring-screen">
                <p id="workflow-form-status" class="muted" hidden></p>
                <form id="workflow-form" class="entity-form">
                    <input id="workflow-created-id" type="hidden" />
                    <div class="form-grid">
                        <div class="form-field wide-field">
                            <label for="workflow-name">"Name"</label>
                            <input class="input" id="workflow-name" type="text" autocomplete="off" />
                        </div>
                        <div class="form-field">
                            <label for="workflow-slug">"Slug"</label>
                            <input class="input" id="workflow-slug" type="text" autocomplete="off" />
                        </div>
                        <div class="form-field wide-field">
                            <label for="workflow-description">"Description"</label>
                            <textarea class="textarea" id="workflow-description" rows="3"></textarea>
                        </div>
                    </div>
                    <div class="actions form-button-container">
                        <button id="workflow-submit-button" class="button is-primary" type="submit">"Create Workflow"</button>
                        <a class="button-link button is-light" href="/app/workflows">"Cancel"</a>
                    </div>
                </form>
            </section>
            <section id="workflow-created-version-panel" class="app-screen box workflow-authoring-screen" hidden>
                <div id="workflow-version-editor" class="record-detail">
                </div>
            </section>
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
                description="Inspect the selected workflow, step versions, and its assignment footprint."
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
                description="Workflow versions are ordered collections of published form versions."
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
                description="Update workflow metadata and publish ordered step collections from published form versions."
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
                        <div class="form-field wide-field">
                            <label for="workflow-description">"Description"</label>
                            <textarea class="textarea" id="workflow-description" rows="3"></textarea>
                        </div>
                    </div>
                    <div class="actions form-button-container">
                        <button class="button is-primary" type="submit">"Save Workflow"</button>
                    </div>
                </form>
            </Panel>
            <Panel
                title="Workflow Version Lifecycle"
                description="Publish a workflow version from ordered steps. Each step chooses its own published form version."
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
