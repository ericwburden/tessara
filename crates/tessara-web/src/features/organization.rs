use leptos::prelude::*;

use crate::features::native_shell::{
    BreadcrumbItem, MetadataStrip, NativePage, PageHeader, Panel, use_account_session,
};
use crate::infra::routing::{NodeRouteParams, require_route_params};

#[derive(Clone)]
pub(crate) struct ScopeNodeContext {
    pub node_id: Option<String>,
    pub node_type_name: Option<String>,
    pub scope_node_type_name: Option<String>,
}

#[derive(Clone)]
pub(crate) struct TreeNodeContext {
    pub id: String,
    pub node_type_name: Option<String>,
    pub parent_node_id: Option<String>,
}

fn normalize_type_label(raw: &str) -> String {
    let parts = raw
        .trim()
        .split(&['_', '-'][..])
        .filter_map(|part| {
            let part = part.trim();
            if part.is_empty() {
                None
            } else {
                let mut chars = part.chars();
                Some(match chars.next() {
                    Some(first) => {
                        let mut out = String::new();
                        out.push(first.to_ascii_uppercase());
                        out.push_str(&chars.as_str().to_ascii_lowercase());
                        out
                    }
                    None => String::new(),
                })
            }
        })
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>();

    if parts.is_empty() {
        "Organization".to_string()
    } else {
        parts.join(" ")
    }
}

pub(crate) fn derive_destination_label(
    scopes: &[ScopeNodeContext],
    nodes: &[TreeNodeContext],
) -> String {
    use std::collections::{HashMap, HashSet};

    let node_by_id = nodes
        .iter()
        .map(|node| (node.id.as_str(), node.clone()))
        .collect::<HashMap<_, _>>();

    let mut scored = Vec::new();
    for scope in scopes {
        let raw_id = scope.node_id.as_deref();
        let node = raw_id.and_then(|scope_id| node_by_id.get(scope_id).cloned());
        let type_label = node
            .as_ref()
            .and_then(|node| node.node_type_name.as_deref())
            .or(scope.node_type_name.as_deref())
            .or(scope.scope_node_type_name.as_deref())
            .unwrap_or("Organization")
            .to_string();

        let mut depth = 0usize;
        let mut cursor = node;
        let mut seen = HashSet::new();
        while let Some(current_node) = cursor {
            if seen.contains(current_node.id.as_str()) {
                break;
            }
            seen.insert(current_node.id.clone());
            let next = current_node
                .parent_node_id
                .as_deref()
                .and_then(|parent_id| node_by_id.get(parent_id).cloned());
            if let Some(parent_node) = next {
                depth += 1;
                cursor = Some(parent_node);
            } else {
                break;
            }
        }

        scored.push((depth, type_label));
    }

    let mut filtered = scored
        .into_iter()
        .filter(|(_, label)| !label.trim().is_empty())
        .collect::<Vec<_>>();
    if filtered.is_empty() {
        return "Organization Explorer".to_string();
    }

    filtered.sort_by(|left, right| left.0.cmp(&right.0).then_with(|| left.1.cmp(&right.1)));
    format!("{} Explorer", normalize_type_label(&filtered[0].1))
}

#[cfg(feature = "hydrate")]
mod hydrate {
    use std::{
        cell::RefCell,
        collections::{HashMap, HashSet},
        rc::Rc,
    };

    use crate::features::native_runtime::{
        by_id, current_search_param, escape_html, get_json, input_value, post_json, put_json,
        redirect, select_value, set_html, set_input_value, set_page_context, set_text,
    };
    use serde::Deserialize;
    use serde_json::{Map, Value, json};
    use wasm_bindgen::{JsCast, closure::Closure};
    use wasm_bindgen_futures::spawn_local;
    use web_sys::HtmlInputElement;

    use super::{ScopeNodeContext, TreeNodeContext, derive_destination_label};

    #[derive(Clone, Deserialize)]
    struct ScopeNodeSummary {
        node_id: String,
        node_name: String,
        node_type_name: String,
        parent_node_id: Option<String>,
        parent_node_name: Option<String>,
    }

    #[derive(Clone, Deserialize)]
    struct AccountContext {
        display_name: String,
        capabilities: Vec<String>,
        scope_nodes: Vec<ScopeNodeSummary>,
    }

    #[derive(Clone, Deserialize)]
    struct NodeSummary {
        id: String,
        node_type_id: String,
        node_type_name: String,
        node_type_singular_label: String,
        node_type_plural_label: String,
        parent_node_id: Option<String>,
        parent_node_name: Option<String>,
        name: String,
        metadata: Value,
    }

    #[derive(Clone, Deserialize)]
    struct NodeDetail {
        id: String,
        node_type_id: String,
        node_type_name: String,
        node_type_singular_label: String,
        node_type_plural_label: String,
        parent_node_id: Option<String>,
        parent_node_name: Option<String>,
        name: String,
        metadata: Value,
        related_forms: Vec<NodeFormLink>,
        related_responses: Vec<NodeSubmissionLink>,
        related_dashboards: Vec<NodeDashboardLink>,
    }

    #[derive(Clone, Deserialize)]
    struct NodeFormLink {
        form_id: String,
        form_name: String,
        form_slug: String,
        published_version_count: i64,
    }

    #[derive(Clone, Deserialize)]
    struct NodeSubmissionLink {
        submission_id: String,
        form_name: String,
        version_label: String,
        status: String,
        created_at: String,
    }

    #[derive(Clone, Deserialize)]
    struct NodeDashboardLink {
        dashboard_id: String,
        dashboard_name: String,
        component_count: i64,
    }

    #[derive(Clone, Deserialize)]
    struct NodeTypeCatalogEntry {
        id: String,
        name: String,
        singular_label: String,
        plural_label: String,
        is_root_type: bool,
        parent_relationships: Vec<NodeTypePeerLink>,
        child_relationships: Vec<NodeTypePeerLink>,
    }

    #[derive(Clone, Deserialize)]
    struct NodeTypePeerLink {
        node_type_id: String,
        singular_label: String,
    }

    #[derive(Clone, Deserialize)]
    struct NodeTypeDefinition {
        id: String,
        singular_label: String,
        plural_label: String,
        parent_relationships: Vec<NodeTypePeerLink>,
        metadata_fields: Vec<NodeMetadataFieldSummary>,
    }

    #[derive(Clone, Deserialize)]
    struct NodeMetadataFieldSummary {
        key: String,
        label: String,
        field_type: String,
        required: bool,
    }

    #[derive(Clone, Deserialize)]
    struct IdResponse {
        id: String,
    }

    #[derive(Clone, Default)]
    struct OrganizationListState {
        nodes: Vec<NodeSummary>,
        node_types: HashMap<String, NodeTypeCatalogEntry>,
        selected_node_id: Option<String>,
        expanded_node_ids: HashSet<String>,
        can_write: bool,
    }

    #[derive(Clone, Default)]
    struct OrganizationFormState {
        nodes: Vec<NodeSummary>,
        node_types: Vec<NodeTypeCatalogEntry>,
        selected_node_type_id: Option<String>,
        metadata_fields: Vec<NodeMetadataFieldSummary>,
        current_node_id: Option<String>,
    }

    fn has_capability(capabilities: &[String], required: &str) -> bool {
        capabilities
            .iter()
            .any(|capability| capability == "admin:all" || capability == required)
    }

    fn parent_key(parent_node_id: Option<&str>) -> String {
        parent_node_id.unwrap_or("__root__").to_string()
    }

    fn tree_by_parent(nodes: &[NodeSummary]) -> HashMap<String, Vec<NodeSummary>> {
        let mut by_parent = HashMap::new();
        for node in nodes {
            by_parent
                .entry(parent_key(node.parent_node_id.as_deref()))
                .or_insert_with(Vec::new)
                .push(node.clone());
        }
        for children in by_parent.values_mut() {
            children.sort_by(|left, right| left.name.cmp(&right.name).then(left.id.cmp(&right.id)));
        }
        by_parent
    }

    fn collapse_branch(
        expanded_node_ids: &mut HashSet<String>,
        by_parent: &HashMap<String, Vec<NodeSummary>>,
        node_id: &str,
    ) {
        expanded_node_ids.remove(node_id);
        if let Some(children) = by_parent.get(&parent_key(Some(node_id))) {
            for child in children {
                collapse_branch(expanded_node_ids, by_parent, &child.id);
            }
        }
    }

    fn expand_path_to_node(
        expanded_node_ids: &mut HashSet<String>,
        nodes: &[NodeSummary],
        node_id: &str,
    ) {
        let by_id = nodes
            .iter()
            .map(|node| (node.id.as_str(), node))
            .collect::<HashMap<_, _>>();
        let mut current = by_id.get(node_id).copied();
        while let Some(node) = current {
            if nodes
                .iter()
                .any(|candidate| candidate.parent_node_id.as_deref() == Some(node.id.as_str()))
            {
                expanded_node_ids.insert(node.id.clone());
            }
            current = node
                .parent_node_id
                .as_deref()
                .and_then(|parent_id| by_id.get(parent_id).copied());
        }
    }

    fn format_metadata_value(value: &Value) -> String {
        if let Some(boolean) = value.as_bool() {
            return if boolean { "Yes".into() } else { "No".into() };
        }
        if let Some(string) = value.as_str() {
            return string.to_string();
        }
        if let Some(array) = value.as_array() {
            return array
                .iter()
                .map(format_metadata_value)
                .collect::<Vec<_>>()
                .join(", ");
        }
        value.to_string()
    }

    fn render_metadata_list(metadata: &Value) -> String {
        let Some(values) = metadata.as_object() else {
            return r#"<p class="muted">No metadata values defined.</p>"#.into();
        };
        if values.is_empty() {
            return r#"<p class="muted">No metadata values defined.</p>"#.into();
        }

        let mut entries = values
            .iter()
            .map(|(key, value)| {
                format!(
                    r#"<li><strong>{}</strong>: {}</li>"#,
                    escape_html(key),
                    escape_html(&format_metadata_value(value))
                )
            })
            .collect::<Vec<_>>();
        entries.sort();
        format!(r#"<ul class="app-list">{}</ul>"#, entries.join(""))
    }

    fn build_path(node_id: &str, nodes: &[NodeSummary]) -> Vec<NodeSummary> {
        let node_by_id = nodes
            .iter()
            .map(|node| (node.id.as_str(), node.clone()))
            .collect::<HashMap<_, _>>();
        let mut path = Vec::new();
        let mut cursor = node_by_id.get(node_id).cloned();
        let mut seen = std::collections::HashSet::new();

        while let Some(node) = cursor {
            if seen.contains(node.id.as_str()) {
                break;
            }
            seen.insert(node.id.clone());
            let parent_id = node.parent_node_id.clone();
            path.push(node);
            cursor = parent_id
                .as_deref()
                .and_then(|parent_id| node_by_id.get(parent_id).cloned());
        }

        path.reverse();
        path
    }

    fn render_path(path: &[NodeSummary]) -> String {
        if path.is_empty() {
            return r#"<p class="muted">Visible path is not available for this record.</p>"#.into();
        }

        let items = path
            .iter()
            .enumerate()
            .map(|(index, node)| {
                format!(
                    r#"<span class="organization-path-item">{}{}</span>"#,
                    if index > 0 {
                        r#"<span class="organization-path-separator" aria-hidden="true">&#8250;</span>"#
                    } else {
                        ""
                    },
                    format!(
                        r#"<a href="/app/organization/{}">{}</a>"#,
                        escape_html(&node.id),
                        escape_html(&node.name)
                    )
                )
            })
            .collect::<Vec<_>>()
            .join("");

        format!(
            r#"<nav class="organization-path-trail" aria-label="Visible organization path">{items}</nav>"#
        )
    }

    fn render_child_actions(
        node_id: &str,
        node_type_id: &str,
        node_types: &HashMap<String, NodeTypeCatalogEntry>,
        can_write: bool,
    ) -> String {
        if !can_write {
            return String::new();
        }
        let Some(node_type) = node_types.get(node_type_id) else {
            return String::new();
        };
        if node_type.child_relationships.is_empty() {
            return r#"<p class="muted">No child record types are configured for this node.</p>"#
                .into();
        }

        let actions = node_type
            .child_relationships
            .iter()
            .map(|child_type| {
                format!(
                    r#"<a class="button-link button is-light is-small organization-action-button" href="/app/organization/new?parent_id={}&node_type_id={}">Add {}</a>"#,
                    escape_html(node_id),
                    escape_html(&child_type.node_type_id),
                    escape_html(&child_type.singular_label)
                )
            })
            .collect::<Vec<_>>()
            .join("");

        format!(r#"<div class="actions organization-card-actions-visible">{actions}</div>"#)
    }

    fn render_related_work(detail: &NodeDetail) -> String {
        let forms = if detail.related_forms.is_empty() {
            r#"<li class="muted">No related forms.</li>"#.into()
        } else {
            detail
                .related_forms
                .iter()
                .map(|form| {
                    format!(
                        r#"<li><a href="/app/forms/{}">{}</a> <span class="muted">{} | Published versions: {}</span></li>"#,
                        escape_html(&form.form_id),
                        escape_html(&form.form_name),
                        escape_html(&form.form_slug),
                        form.published_version_count,
                    )
                })
                .collect::<Vec<_>>()
                .join("")
        };
        let responses = if detail.related_responses.is_empty() {
            r#"<li class="muted">No recent responses.</li>"#.into()
        } else {
            detail
                .related_responses
                .iter()
                .map(|submission| {
                    format!(
                        r#"<li><a href="/app/responses/{}">{}</a> <span class="muted">{} | {} | {}</span></li>"#,
                        escape_html(&submission.submission_id),
                        escape_html(&submission.form_name),
                        escape_html(&submission.version_label),
                        escape_html(&submission.status),
                        escape_html(&submission.created_at),
                    )
                })
                .collect::<Vec<_>>()
                .join("")
        };
        let dashboards = if detail.related_dashboards.is_empty() {
            r#"<li class="muted">No related dashboards.</li>"#.into()
        } else {
            detail
                .related_dashboards
                .iter()
                .map(|dashboard| {
                    format!(
                        r#"<li><a href="/app/dashboards/{}">{}</a> <span class="muted">Components: {}</span></li>"#,
                        escape_html(&dashboard.dashboard_id),
                        escape_html(&dashboard.dashboard_name),
                        dashboard.component_count,
                    )
                })
                .collect::<Vec<_>>()
                .join("")
        };

        format!(
            r#"<section class="organization-related-work"><div class="organization-section-heading"><h4>Related Work</h4><p class="muted">Forms, responses, and dashboards stay primary for the selected node.</p></div><div class="organization-related-work__totals"><span class="organization-related-work__total"><strong>{}</strong><span>Forms</span></span><span class="organization-related-work__total"><strong>{}</strong><span>Responses</span></span><span class="organization-related-work__total"><strong>{}</strong><span>Dashboards</span></span></div><div class="organization-related-work__lists"><section class="organization-related-work__group"><h5>Forms</h5><ul id="organization-related-forms" class="app-list">{forms}</ul></section><section class="organization-related-work__group"><h5>Responses</h5><ul id="organization-related-responses" class="app-list">{responses}</ul></section><section class="organization-related-work__group"><h5>Dashboards</h5><ul id="organization-related-dashboards" class="app-list">{dashboards}</ul></section></div></section>"#,
            detail.related_forms.len(),
            detail.related_responses.len(),
            detail.related_dashboards.len(),
        )
    }

    fn render_selection_preview(
        detail: &NodeDetail,
        nodes: &[NodeSummary],
        node_types: &HashMap<String, NodeTypeCatalogEntry>,
        can_write: bool,
    ) -> String {
        let child_actions =
            render_child_actions(&detail.id, &detail.node_type_id, node_types, can_write);
        let path = render_path(&build_path(&detail.id, nodes));
        let management_actions = format!(
            r#"<div class="actions organization-selected-work__actions"><a class="button-link button is-light" href="/app/organization/{}">Open Detail</a>{}</div>"#,
            escape_html(&detail.id),
            if can_write {
                format!(
                    r#"<a class="button-link button is-light" href="/app/organization/{}/edit">Edit</a>"#,
                    escape_html(&detail.id)
                )
            } else {
                String::new()
            }
        );
        let management_note = if can_write && child_actions.is_empty() {
            r#"<p class="muted">No child record types are configured for this node.</p>"#
                .to_string()
        } else if !can_write {
            r#"<p class="muted">Read-only scope. Management actions stay on the detail route.</p>"#
                .to_string()
        } else {
            String::new()
        };
        format!(
            r#"<section class="organization-selected-work"><div class="organization-selected-work__header"><p class="eyebrow">{}</p><h3>{}</h3><p class="muted">{}</p><div class="organization-selected-work__path">{}</div></div><dl class="organization-selected-work__facts"><div><dt>Parent</dt><dd>{}</dd></div><div><dt>Type</dt><dd>{}</dd></div><div><dt>Plural</dt><dd>{}</dd></div></dl>{}<section class="organization-selected-work__section organization-selected-work__metadata"><div class="organization-section-heading"><h4>Metadata</h4><p class="muted">Current node metadata stays visible without expanding a separate detail card.</p></div>{}</section><section class="organization-selected-work__section organization-selected-work__management"><div class="organization-section-heading"><h4>Management</h4><p class="muted">Open the detail route for deeper edits, or continue directly into child-record actions from here.</p></div><div class="organization-selected-work__management-body">{}{}{}</div></section></section>"#,
            escape_html(&detail.node_type_singular_label),
            escape_html(&detail.name),
            escape_html(&detail.node_type_singular_label),
            path,
            escape_html(detail.parent_node_name.as_deref().unwrap_or("Top-level")),
            escape_html(&detail.node_type_name),
            escape_html(&detail.node_type_plural_label),
            render_related_work(detail),
            render_metadata_list(&detail.metadata),
            management_actions,
            child_actions,
            management_note,
        )
    }

    fn render_node_tree(
        nodes: &[NodeSummary],
        selected_node_id: Option<&str>,
        expanded_node_ids: &HashSet<String>,
    ) -> String {
        fn render_branch(
            node: &NodeSummary,
            by_parent: &HashMap<String, Vec<NodeSummary>>,
            selected_node_id: Option<&str>,
            expanded_node_ids: &HashSet<String>,
            depth: usize,
        ) -> String {
            let children = by_parent
                .get(&parent_key(Some(node.id.as_str())))
                .cloned()
                .unwrap_or_default();
            let is_expanded = expanded_node_ids.contains(&node.id);
            let child_html = if children.is_empty() || !is_expanded {
                String::new()
            } else {
                format!(
                    r#"<div class="organization-explorer-children">{}</div>"#,
                    children
                        .iter()
                        .map(|child| {
                            render_branch(
                                child,
                                by_parent,
                                selected_node_id,
                                expanded_node_ids,
                                depth + 1,
                            )
                        })
                        .collect::<Vec<_>>()
                        .join("")
                )
            };
            let child_count = children.len();
            let summary = if child_count == 0 {
                "No visible children".to_string()
            } else {
                format!(
                    "{} visible {}",
                    child_count,
                    if child_count == 1 {
                        "child"
                    } else {
                        "children"
                    }
                )
            };
            let toggle = if child_count == 0 {
                r#"<span class="organization-explorer-toggle-spacer" aria-hidden="true"></span>"#
                    .to_string()
            } else {
                format!(
                    r#"<button class="organization-explorer-toggle" type="button" data-toggle-node-id="{}" data-tree-depth="{}" aria-label="{} {}" aria-expanded="{}"><span class="organization-explorer-toggle__glyph" aria-hidden="true"><i class="fa-solid fa-chevron-right"></i></span></button>"#,
                    escape_html(&node.id),
                    depth,
                    if is_expanded { "Collapse" } else { "Expand" },
                    escape_html(&node.name),
                    if is_expanded { "true" } else { "false" },
                )
            };

            format!(
                r#"<div class="organization-explorer-branch" style="--organization-depth:{}"><div class="organization-explorer-branch-header">{}<button class="organization-explorer-row" type="button" data-select-node-id="{}" data-selected="{}" data-tree-depth="{}" aria-selected="{}"><span class="organization-explorer-row__copy"><span class="organization-explorer-row__type">{}</span><span class="organization-explorer-row__name">{}</span></span><span class="organization-explorer-row__summary">{}</span></button></div>{}</div>"#,
                depth,
                toggle,
                escape_html(&node.id),
                if Some(node.id.as_str()) == selected_node_id {
                    "true"
                } else {
                    "false"
                },
                depth,
                if Some(node.id.as_str()) == selected_node_id {
                    "true"
                } else {
                    "false"
                },
                escape_html(&node.node_type_singular_label),
                escape_html(&node.name),
                escape_html(&summary),
                child_html,
            )
        }

        let by_parent = tree_by_parent(nodes);

        let visible_ids = nodes
            .iter()
            .map(|node| node.id.as_str())
            .collect::<std::collections::HashSet<_>>();
        let mut roots = nodes
            .iter()
            .filter(|node| {
                node.parent_node_id
                    .as_deref()
                    .is_none_or(|parent_id| !visible_ids.contains(parent_id))
            })
            .cloned()
            .collect::<Vec<_>>();
        roots.sort_by(|left, right| left.name.cmp(&right.name).then(left.id.cmp(&right.id)));
        if roots.is_empty() {
            return r#"<p class="muted">No organization records are visible for this scope.</p>"#
                .into();
        }

        roots
            .iter()
            .map(|node| render_branch(node, &by_parent, selected_node_id, expanded_node_ids, 0))
            .collect::<Vec<_>>()
            .join("")
    }

    fn render_node_type_options(
        node_types: &[NodeTypeCatalogEntry],
        selected_value: Option<&str>,
        disabled: bool,
    ) -> String {
        let options = node_types
            .iter()
            .map(|node_type| {
                format!(
                    r#"<option value="{}" {}>{}</option>"#,
                    escape_html(&node_type.id),
                    if Some(node_type.id.as_str()) == selected_value {
                        "selected"
                    } else {
                        ""
                    },
                    escape_html(&node_type.singular_label),
                )
            })
            .collect::<Vec<_>>()
            .join("");
        format!(
            r#"<option value="">Select node type</option>{options}{}"#,
            if disabled { "" } else { "" }
        )
    }

    fn render_parent_options(
        state: &OrganizationFormState,
        selected_parent_id: Option<&str>,
    ) -> String {
        let Some(node_type_id) = state.selected_node_type_id.as_deref() else {
            return r#"<option value="">Select parent organization</option>"#.into();
        };
        let Some(node_type) = state
            .node_types
            .iter()
            .find(|entry| entry.id == node_type_id)
        else {
            return r#"<option value="">Select parent organization</option>"#.into();
        };
        if node_type.parent_relationships.is_empty() {
            return r#"<option value="">Top-level record</option>"#.into();
        }

        let allowed_parent_ids = node_type
            .parent_relationships
            .iter()
            .map(|relationship| relationship.node_type_id.as_str())
            .collect::<Vec<_>>();
        let mut nodes = state
            .nodes
            .iter()
            .filter(|node| {
                allowed_parent_ids.contains(&node.node_type_id.as_str())
                    && state
                        .current_node_id
                        .as_deref()
                        .is_none_or(|current_id| current_id != node.id)
            })
            .cloned()
            .collect::<Vec<_>>();
        nodes.sort_by(|left, right| left.name.cmp(&right.name).then(left.id.cmp(&right.id)));

        let mut options = vec![r#"<option value="">Top-level record</option>"#.to_string()];
        options.extend(nodes.into_iter().map(|node| {
            format!(
                r#"<option value="{}" {}>{} ({})</option>"#,
                escape_html(&node.id),
                if Some(node.id.as_str()) == selected_parent_id {
                    "selected"
                } else {
                    ""
                },
                escape_html(&node.name),
                escape_html(&node.node_type_singular_label),
            )
        }));
        options.join("")
    }

    fn metadata_input_id(key: &str) -> String {
        format!("organization-metadata-{key}")
    }

    fn render_metadata_fields(
        metadata_fields: &[NodeMetadataFieldSummary],
        metadata: Option<&Map<String, Value>>,
    ) -> String {
        if metadata_fields.is_empty() {
            return r#"<p class="muted">No metadata fields are configured for this node type.</p>"#
                .into();
        }

        metadata_fields
            .iter()
            .map(|field| {
                let input_id = metadata_input_id(&field.key);
                let value = metadata.and_then(|metadata| metadata.get(&field.key));
                let required = if field.required { " required" } else { "" };
                let content = match field.field_type.as_str() {
                    "boolean" => format!(
                        r#"<input id="{}" type="checkbox"{} {}>"#,
                        escape_html(&input_id),
                        required,
                        if value.and_then(Value::as_bool).unwrap_or(false) {
                            "checked"
                        } else {
                            ""
                        }
                    ),
                    "number" => format!(
                        r#"<input class="input" id="{}" type="number" value="{}"{}>"#,
                        escape_html(&input_id),
                        escape_html(&value.map(format_metadata_value).unwrap_or_default()),
                        required,
                    ),
                    "date" => format!(
                        r#"<input class="input" id="{}" type="date" value="{}"{}>"#,
                        escape_html(&input_id),
                        escape_html(value.and_then(Value::as_str).unwrap_or_default()),
                        required,
                    ),
                    _ => format!(
                        r#"<input class="input" id="{}" type="text" value="{}"{}>"#,
                        escape_html(&input_id),
                        escape_html(&value.map(format_metadata_value).unwrap_or_default()),
                        required,
                    ),
                };
                format!(
                    r#"<div class="form-field"><label for="{}">{}{}</label>{}</div>"#,
                    escape_html(&input_id),
                    escape_html(&field.label),
                    if field.required { " *" } else { "" },
                    content,
                )
            })
            .collect::<Vec<_>>()
            .join("")
    }

    fn collect_metadata_values(
        metadata_fields: &[NodeMetadataFieldSummary],
    ) -> Result<Value, String> {
        let mut values = Map::new();

        for field in metadata_fields {
            let Some(element) = by_id(&metadata_input_id(&field.key)) else {
                continue;
            };
            let input = element
                .dyn_into::<HtmlInputElement>()
                .map_err(|_| format!("{} input was not available", field.label))?;

            match field.field_type.as_str() {
                "boolean" => {
                    values.insert(field.key.clone(), Value::Bool(input.checked()));
                }
                "number" => {
                    let raw = input.value().trim().to_string();
                    if raw.is_empty() {
                        if field.required {
                            return Err(format!("{} is required", field.label));
                        }
                    } else {
                        let parsed = raw
                            .parse::<f64>()
                            .map_err(|_| format!("{} must be a number", field.label))?;
                        values.insert(field.key.clone(), json!(parsed));
                    }
                }
                _ => {
                    let raw = input.value().trim().to_string();
                    if raw.is_empty() {
                        if field.required {
                            return Err(format!("{} is required", field.label));
                        }
                    } else {
                        values.insert(field.key.clone(), Value::String(raw));
                    }
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

    async fn load_node_type_definition(node_type_id: &str) -> Result<NodeTypeDefinition, String> {
        get_json::<NodeTypeDefinition>(&format!("/api/admin/node-types/{node_type_id}")).await
    }

    async fn select_list_node(
        node_id: String,
        state: Rc<RefCell<OrganizationListState>>,
    ) -> Result<(), String> {
        let detail = get_json::<NodeDetail>(&format!("/api/nodes/{node_id}")).await?;
        {
            let mut state_mut = state.borrow_mut();
            let nodes = state_mut.nodes.clone();
            state_mut.selected_node_id = Some(node_id.clone());
            expand_path_to_node(&mut state_mut.expanded_node_ids, &nodes, &node_id);
            let tree_html = render_node_tree(
                &nodes,
                state_mut.selected_node_id.as_deref(),
                &state_mut.expanded_node_ids,
            );
            set_html("organization-directory-tree", &tree_html);
        }

        let snapshot = state.borrow().clone();
        let preview = render_selection_preview(
            &detail,
            &snapshot.nodes,
            &snapshot.node_types,
            snapshot.can_write,
        );
        set_html("organization-selection-preview", &preview);
        set_text(
            "organization-list-status",
            &format!("Selected {}.", detail.name),
        );

        Ok(())
    }

    fn toggle_list_node(node_id: String, state: Rc<RefCell<OrganizationListState>>) {
        let mut state_mut = state.borrow_mut();
        let by_parent = tree_by_parent(&state_mut.nodes);
        let parent_node_id = state_mut
            .nodes
            .iter()
            .find(|node| node.id == node_id)
            .and_then(|node| node.parent_node_id.clone());
        if state_mut.expanded_node_ids.contains(&node_id) {
            collapse_branch(&mut state_mut.expanded_node_ids, &by_parent, &node_id);
            set_text("organization-list-status", "Collapsed hierarchy branch.");
        } else {
            let sibling_key = parent_key(parent_node_id.as_deref());
            if let Some(siblings) = by_parent.get(&sibling_key) {
                for sibling in siblings {
                    if sibling.id != node_id {
                        collapse_branch(&mut state_mut.expanded_node_ids, &by_parent, &sibling.id);
                    }
                }
            }
            state_mut.expanded_node_ids.insert(node_id.clone());
            set_text("organization-list-status", "Expanded hierarchy branch.");
        }
        let tree_html = render_node_tree(
            &state_mut.nodes,
            state_mut.selected_node_id.as_deref(),
            &state_mut.expanded_node_ids,
        );
        set_html("organization-directory-tree", &tree_html);
    }

    fn wire_list_actions(state: Rc<RefCell<OrganizationListState>>) {
        let Some(tree_root) = by_id("organization-directory-tree") else {
            return;
        };
        let closure = Closure::wrap(Box::new(move |event: web_sys::Event| {
            let Some(target) = event.target() else {
                return;
            };
            let Ok(target) = target.dyn_into::<web_sys::Element>() else {
                return;
            };
            if let Ok(Some(toggle)) = target.closest("[data-toggle-node-id]") {
                if let Some(node_id) = toggle.get_attribute("data-toggle-node-id") {
                    toggle_list_node(node_id, state.clone());
                }
                return;
            }
            let Ok(Some(button)) = target.closest("[data-select-node-id]") else {
                return;
            };
            let Some(node_id) = button.get_attribute("data-select-node-id") else {
                return;
            };
            let state = state.clone();
            spawn_local(async move {
                if let Err(error) = select_list_node(node_id, state).await {
                    set_text("organization-list-status", &error);
                }
            });
        }) as Box<dyn FnMut(_)>);
        tree_root
            .add_event_listener_with_callback("click", closure.as_ref().unchecked_ref())
            .ok();
        closure.forget();
    }

    pub fn load_list_page() {
        let state = Rc::new(RefCell::new(OrganizationListState::default()));
        spawn_local(async move {
            match (
                get_json::<AccountContext>("/api/me").await,
                get_json::<Vec<NodeSummary>>("/api/nodes").await,
                get_json::<Vec<NodeTypeCatalogEntry>>("/api/node-types").await,
            ) {
                (Ok(account), Ok(nodes), Ok(node_types)) => {
                    let title = derive_destination_label(
                        &account
                            .scope_nodes
                            .iter()
                            .map(|scope| ScopeNodeContext {
                                node_id: Some(scope.node_id.clone()),
                                node_type_name: Some(scope.node_type_name.clone()),
                                scope_node_type_name: Some(scope.node_type_name.clone()),
                            })
                            .collect::<Vec<_>>(),
                        &nodes
                            .iter()
                            .map(|node| TreeNodeContext {
                                id: node.id.clone(),
                                node_type_name: Some(node.node_type_name.clone()),
                                parent_node_id: node.parent_node_id.clone(),
                            })
                            .collect::<Vec<_>>(),
                    );
                    let can_write = has_capability(&account.capabilities, "hierarchy:write");
                    let node_type_map = node_types
                        .iter()
                        .map(|node_type| (node_type.id.clone(), node_type.clone()))
                        .collect::<HashMap<_, _>>();
                    let initial_selection = account
                        .scope_nodes
                        .first()
                        .map(|scope| scope.node_id.clone())
                        .or_else(|| nodes.first().map(|node| node.id.clone()));
                    let mut expanded_node_ids = HashSet::new();
                    if let Some(node_id) = initial_selection.as_deref() {
                        expand_path_to_node(&mut expanded_node_ids, &nodes, node_id);
                    }
                    let tree_html =
                        render_node_tree(&nodes, initial_selection.as_deref(), &expanded_node_ids);

                    set_text("organization-page-title", &title);
                    set_text(
                        "organization-page-description",
                        &format!(
                            "{} keeps hierarchy traversal quiet and related work close at hand.",
                            title
                        ),
                    );
                    set_text("organization-list-title", "Visible Hierarchy");
                    set_text(
                        "organization-list-context",
                        &format!(
                            "Signed in as {}. Traverse your visible hierarchy and keep selected-node work in context.",
                            account.display_name
                        ),
                    );
                    set_text(
                        "organization-list-status",
                        &format!("Showing {} visible hierarchy records.", nodes.len()),
                    );
                    set_html("organization-directory-tree", &tree_html);
                    if can_write {
                        let default_node_type = node_types
                            .iter()
                            .find(|node_type| node_type.is_root_type)
                            .map(|node_type| node_type.id.clone());
                        let href = default_node_type
                            .map(|node_type_id| {
                                format!("/app/organization/new?node_type_id={node_type_id}")
                            })
                            .unwrap_or_else(|| "/app/organization/new".into());
                        set_html(
                            "organization-page-actions",
                            &format!(
                                r#"<a id="organization-create-link" class="button-link button is-primary" href="{}">Create Record</a>"#,
                                escape_html(&href)
                            ),
                        );
                    } else {
                        set_html(
                            "organization-page-actions",
                            r#"<p class="muted">Read-only scope. Record creation is not available.</p>"#,
                        );
                    }

                    {
                        let mut state_mut = state.borrow_mut();
                        state_mut.nodes = nodes.clone();
                        state_mut.node_types = node_type_map;
                        state_mut.selected_node_id = initial_selection.clone();
                        state_mut.expanded_node_ids = expanded_node_ids;
                        state_mut.can_write = can_write;
                    }
                    wire_list_actions(state.clone());

                    if let Some(node_id) = initial_selection {
                        let _ = select_list_node(node_id, state).await;
                    } else {
                        set_html(
                            "organization-selection-preview",
                            r#"<p class="muted">No visible organization record is selected.</p>"#,
                        );
                    }
                }
                (Err(error), _, _) | (_, Err(error), _) | (_, _, Err(error)) => {
                    set_text("organization-list-status", &error);
                    set_html(
                        "organization-directory-tree",
                        &format!(r#"<p class="muted">{}</p>"#, escape_html(&error)),
                    );
                    set_html(
                        "organization-selection-preview",
                        r#"<p class="muted">Selection preview is unavailable.</p>"#,
                    );
                }
            }
        });
    }

    fn update_form_schema(
        state: Rc<RefCell<OrganizationFormState>>,
        node_type_id: String,
        selected_parent_id: Option<String>,
        metadata: Option<Map<String, Value>>,
    ) {
        spawn_local(async move {
            match load_node_type_definition(&node_type_id).await {
                Ok(definition) => {
                    {
                        let mut state_mut = state.borrow_mut();
                        state_mut.selected_node_type_id = Some(definition.id.clone());
                        state_mut.metadata_fields = definition.metadata_fields.clone();
                    }
                    set_text(
                        "organization-form-status",
                        &format!(
                            "Editing {} records. Parent choices and metadata fields are ready.",
                            definition.singular_label
                        ),
                    );
                    set_html(
                        "organization-metadata-fields",
                        &render_metadata_fields(&definition.metadata_fields, metadata.as_ref()),
                    );
                    set_html(
                        "organization-parent-node",
                        &render_parent_options(&state.borrow(), selected_parent_id.as_deref()),
                    );
                    set_text(
                        "organization-metadata-context",
                        &format!(
                            "Metadata inputs are generated from the {} record type.",
                            definition.singular_label.to_lowercase()
                        ),
                    );
                    set_text(
                        "organization-parent-node-label",
                        &if definition.parent_relationships.is_empty() {
                            "Parent Organization (optional top-level)".into()
                        } else {
                            format!("Parent {}", escape_html(&definition.plural_label))
                        },
                    );
                }
                Err(error) => set_text("organization-form-status", &error),
            }
        });
    }

    fn wire_node_type_change(state: Rc<RefCell<OrganizationFormState>>) {
        if let Some(select) = by_id("organization-node-type") {
            let closure = Closure::wrap(Box::new(move |_event: web_sys::Event| {
                let selected = select_value("organization-node-type").unwrap_or_default();
                if selected.is_empty() {
                    set_html(
                        "organization-metadata-fields",
                        r#"<p class="muted">Select a node type to load metadata fields.</p>"#,
                    );
                    return;
                }
                update_form_schema(state.clone(), selected, None, None);
            }) as Box<dyn FnMut(_)>);
            select
                .add_event_listener_with_callback("change", closure.as_ref().unchecked_ref())
                .ok();
            closure.forget();
        }
    }

    pub fn load_create_page() {
        let state = Rc::new(RefCell::new(OrganizationFormState::default()));
        let preselected_node_type_id = current_search_param("node_type_id");
        let preselected_parent_id = current_search_param("parent_id");

        let state_for_load = state.clone();
        spawn_local(async move {
            match (
                get_json::<Vec<NodeTypeCatalogEntry>>("/api/node-types").await,
                get_json::<Vec<NodeSummary>>("/api/nodes").await,
            ) {
                (Ok(node_types), Ok(nodes)) => {
                    let selected_node_type_id = preselected_node_type_id
                        .clone()
                        .or_else(|| {
                            node_types
                                .iter()
                                .find(|node_type| node_type.is_root_type)
                                .map(|node_type| node_type.id.clone())
                        })
                        .or_else(|| node_types.first().map(|node_type| node_type.id.clone()));

                    {
                        let mut state_mut = state_for_load.borrow_mut();
                        state_mut.nodes = nodes;
                        state_mut.node_types = node_types.clone();
                        state_mut.selected_node_type_id = selected_node_type_id.clone();
                    }

                    set_html(
                        "organization-node-type",
                        &render_node_type_options(
                            &node_types,
                            selected_node_type_id.as_deref(),
                            false,
                        ),
                    );
                    if let Some(node_type_id) = selected_node_type_id {
                        update_form_schema(
                            state_for_load.clone(),
                            node_type_id,
                            preselected_parent_id.clone(),
                            None,
                        );
                    } else {
                        set_text(
                            "organization-form-status",
                            "No node types are available for organization creation.",
                        );
                    }
                    wire_node_type_change(state_for_load.clone());
                }
                (Err(error), _) | (_, Err(error)) => set_text("organization-form-status", &error),
            }
        });

        attach_submit_handler("organization-form", move || {
            let state = state.clone();
            spawn_local(async move {
                let state_snapshot = state.borrow().clone();
                let Some(node_type_id) = select_value("organization-node-type")
                    .filter(|value| !value.is_empty())
                    .or(state_snapshot.selected_node_type_id.clone())
                else {
                    set_text(
                        "organization-form-status",
                        "Select a node type before saving.",
                    );
                    return;
                };
                let metadata = match collect_metadata_values(&state_snapshot.metadata_fields) {
                    Ok(metadata) => metadata,
                    Err(error) => {
                        set_text("organization-form-status", &error);
                        return;
                    }
                };
                let payload = json!({
                    "node_type_id": node_type_id,
                    "parent_node_id": select_value("organization-parent-node").filter(|value| !value.is_empty()),
                    "name": input_value("organization-name").unwrap_or_default(),
                    "metadata": metadata,
                });
                match post_json::<IdResponse>("/api/admin/nodes", &payload).await {
                    Ok(response) => redirect(&format!("/app/organization/{}", response.id)),
                    Err(error) => set_text("organization-form-status", &error),
                }
            });
        });
    }

    pub fn load_detail_page(node_id: String) {
        spawn_local(async move {
            match (
                get_json::<NodeDetail>(&format!("/api/nodes/{node_id}")).await,
                get_json::<Vec<NodeSummary>>("/api/nodes").await,
                get_json::<Vec<NodeTypeCatalogEntry>>("/api/node-types").await,
            ) {
                (Ok(detail), Ok(nodes), Ok(node_types)) => {
                    let node_type_map = node_types
                        .into_iter()
                        .map(|node_type| (node_type.id.clone(), node_type))
                        .collect::<HashMap<_, _>>();
                    let path = build_path(&detail.id, &nodes);
                    set_text("organization-detail-status", "Organization detail loaded.");
                    set_text("organization-detail-heading", &detail.name);
                    set_text(
                        "organization-detail-context",
                        &format!(
                            "{} detail with visible hierarchy context and related records.",
                            detail.node_type_singular_label
                        ),
                    );
                    set_html("organization-detail-path", &render_path(&path));
                    set_html(
                        "organization-summary",
                        &format!(
                            r#"<ul class="app-list"><li><strong>Type</strong>: {} ({})</li><li><strong>Parent</strong>: {}</li><li><strong>Plural label</strong>: {}</li></ul>"#,
                            escape_html(&detail.node_type_name),
                            escape_html(&detail.node_type_singular_label),
                            escape_html(detail.parent_node_name.as_deref().unwrap_or("Top-level")),
                            escape_html(&detail.node_type_plural_label),
                        ),
                    );
                    set_html(
                        "organization-metadata",
                        &render_metadata_list(&detail.metadata),
                    );
                    set_html("organization-related", &render_related_work(&detail));
                    set_html(
                        "organization-child-actions",
                        &render_child_actions(
                            &detail.id,
                            &detail.node_type_id,
                            &node_type_map,
                            true,
                        ),
                    );
                }
                (Err(error), _, _) | (_, Err(error), _) | (_, _, Err(error)) => {
                    set_text("organization-detail-status", &error);
                    set_html(
                        "organization-summary",
                        &format!(r#"<p class="muted">{}</p>"#, escape_html(&error)),
                    );
                }
            }
        });
    }

    pub fn load_edit_page(node_id: String) {
        let state = Rc::new(RefCell::new(OrganizationFormState::default()));
        let state_for_load = state.clone();
        let node_id_for_load = node_id.clone();
        spawn_local(async move {
            match (
                get_json::<NodeDetail>(&format!("/api/nodes/{node_id_for_load}")).await,
                get_json::<Vec<NodeTypeCatalogEntry>>("/api/node-types").await,
                get_json::<Vec<NodeSummary>>("/api/nodes").await,
            ) {
                (Ok(detail), Ok(node_types), Ok(nodes)) => {
                    {
                        let mut state_mut = state_for_load.borrow_mut();
                        state_mut.nodes = nodes;
                        state_mut.node_types = node_types.clone();
                        state_mut.selected_node_type_id = Some(detail.node_type_id.clone());
                        state_mut.current_node_id = Some(detail.id.clone());
                    }

                    set_input_value("organization-name", &detail.name);
                    set_html(
                        "organization-node-type",
                        &render_node_type_options(&node_types, Some(&detail.node_type_id), true),
                    );
                    if let Some(element) = by_id("organization-node-type") {
                        let _ = element.set_attribute("disabled", "disabled");
                    }

                    update_form_schema(
                        state_for_load.clone(),
                        detail.node_type_id.clone(),
                        detail.parent_node_id.clone(),
                        detail.metadata.as_object().cloned(),
                    );
                }
                (Err(error), _, _) | (_, Err(error), _) | (_, _, Err(error)) => {
                    set_text("organization-form-status", &error)
                }
            }
        });

        attach_submit_handler("organization-form", move || {
            let node_id = node_id.clone();
            let state = state.clone();
            spawn_local(async move {
                let state_snapshot = state.borrow().clone();
                let metadata = match collect_metadata_values(&state_snapshot.metadata_fields) {
                    Ok(metadata) => metadata,
                    Err(error) => {
                        set_text("organization-form-status", &error);
                        return;
                    }
                };
                let payload = json!({
                    "parent_node_id": select_value("organization-parent-node").filter(|value| !value.is_empty()),
                    "name": input_value("organization-name").unwrap_or_default(),
                    "metadata": metadata,
                });
                match put_json::<IdResponse>(&format!("/api/admin/nodes/{node_id}"), &payload).await
                {
                    Ok(_) => redirect(&format!("/app/organization/{node_id}")),
                    Err(error) => set_text("organization-form-status", &error),
                }
            });
        });
    }

    pub fn set_context(page_key: &'static str, record_id: Option<String>) {
        set_page_context(page_key, "organization", record_id);
    }
}

#[component]
pub fn OrganizationListPage() -> impl IntoView {
    let session = use_account_session();
    #[cfg(not(feature = "hydrate"))]
    let _ = session;
    #[cfg(feature = "hydrate")]
    Effect::new(move |_| {
        if session.loaded.get() && session.account.get().is_some() {
            hydrate::set_context("organization-list", None);
            hydrate::load_list_page();
        }
    });

    view! {
        <NativePage
            title="Organization Explorer"
            description="Tessara organization list screen."
            page_key="organization-list"
            active_route="organization"
            workspace_label="Product Area"
            required_capability="hierarchy:read"
            breadcrumbs=vec![
                BreadcrumbItem::link("Home", "/app"),
                BreadcrumbItem::current("Organization"),
            ]
        >
            <div class="organization-explorer-layout">
                <Panel
                    title="Explorer"
                    description="Traverse the visible hierarchy with a quiet indented explorer instead of card-per-node navigation."
                >
                    <div id="organization-page-actions" class="actions ui-action-group">
                        <p class="muted">"Loading organization actions..."</p>
                    </div>
                    <div class="organization-explorer-panel">
                        <div class="organization-section-heading organization-explorer-panel__header">
                            <h2 id="organization-list-title">"Visible Hierarchy"</h2>
                            <p id="organization-list-context" class="muted">
                                "Loading scope-aware hierarchy context."
                            </p>
                        </div>
                        <p id="organization-list-status" class="muted">
                            "Loading organization hierarchy..."
                        </p>
                        <div id="organization-directory-tree" class="organization-explorer-tree">
                            <p class="muted">"Loading scoped organization records..."</p>
                        </div>
                    </div>
                </Panel>
                <Panel
                    title="Selected Node"
                    description="Related work leads here so the explorer stays connected to forms, responses, dashboards, and management actions."
                >
                    <div
                        id="organization-selection-preview"
                        class="record-detail organization-selected-panel"
                    >
                        <p class="muted">
                            "Select an organization record to open its lower detail sheet and related work."
                        </p>
                    </div>
                </Panel>
            </div>
        </NativePage>
    }
}

#[component]
pub fn OrganizationCreatePage() -> impl IntoView {
    let session = use_account_session();
    #[cfg(not(feature = "hydrate"))]
    let _ = session;
    #[cfg(feature = "hydrate")]
    Effect::new(move |_| {
        if session.loaded.get() && session.account.get().is_some() {
            hydrate::set_context("organization-create", None);
            hydrate::load_create_page();
        }
    });

    view! {
        <NativePage
            title="Create Organization"
            description="Create a runtime organization record."
            page_key="organization-create"
            active_route="organization"
            workspace_label="Product Area"
            required_capability="admin:all"
            breadcrumbs=vec![
                BreadcrumbItem::link("Home", "/app"),
                BreadcrumbItem::link("Organization", "/app/organization"),
                BreadcrumbItem::current("Create Organization"),
            ]
        >
            <PageHeader
                eyebrow="Organization"
                title="Create Organization"
                description="Create a scoped hierarchy record and return to the explorer with the new node in context."
            />
            <MetadataStrip items=vec![
                ("Mode", "Create".into()),
                ("Surface", "Organization authoring".into()),
                ("State", "Metadata entry".into()),
            ]/>
            <Panel
                title="Organization Record"
                description="Choose a node type, set a parent when required, and provide metadata without leaving the explorer workflow."
            >
                <p id="organization-form-status" class="muted">
                    "Loading organization schema."
                </p>
                <form id="organization-form" class="entity-form">
                    <div class="form-grid">
                        <div class="form-field">
                            <label id="organization-node-type-label" for="organization-node-type">
                                "Node Type"
                            </label>
                            <select class="input" id="organization-node-type"></select>
                        </div>
                        <div class="form-field">
                            <label id="organization-parent-node-label" for="organization-parent-node">
                                "Parent Organization"
                            </label>
                            <select class="input" id="organization-parent-node"></select>
                        </div>
                        <div class="form-field wide-field">
                            <label id="organization-name-label" for="organization-name">"Name"</label>
                            <input class="input" id="organization-name" type="text" autocomplete="off" />
                        </div>
                    </div>
                    <section class="page-panel nested-form-panel">
                        <h3 id="organization-metadata-title">"Metadata"</h3>
                        <p id="organization-metadata-context" class="muted">
                            "Metadata inputs are generated from the selected node type."
                        </p>
                        <div id="organization-metadata-fields" class="form-grid">
                            <p class="muted">"Select a node type to load metadata fields."</p>
                        </div>
                    </section>
                    <div class="actions">
                        <button class="button is-primary" type="submit">"Create Record"</button>
                        <a class="button-link button is-light" href="/app/organization">"Cancel"</a>
                    </div>
                </form>
            </Panel>
        </NativePage>
    }
}

#[component]
pub fn OrganizationDetailPage() -> impl IntoView {
    let NodeRouteParams { node_id } = require_route_params();
    let _node_id_for_load = node_id.clone();
    let node_id_value = StoredValue::new(node_id.clone());
    let session = use_account_session();
    #[cfg(not(feature = "hydrate"))]
    let _ = session;
    #[cfg(feature = "hydrate")]
    Effect::new(move |_| {
        if session.loaded.get() && session.account.get().is_some() {
            hydrate::set_context("organization-detail", Some(_node_id_for_load.clone()));
            hydrate::load_detail_page(_node_id_for_load.clone());
        }
    });

    view! {
        <NativePage
            title="Organization Detail"
            description="Organization detail screen."
            page_key="organization-detail"
            active_route="organization"
            workspace_label="Product Area"
            record_id=node_id.clone()
            required_capability="hierarchy:read"
            breadcrumbs=vec![
                BreadcrumbItem::link("Home", "/app"),
                BreadcrumbItem::link("Organization", "/app/organization"),
                BreadcrumbItem::current("Organization Detail"),
            ]
        >
            <PageHeader
                eyebrow="Organization"
                title="Organization Detail"
                description="Review the selected hierarchy record, its visible path, and related work from the explorer model."
            />
            <MetadataStrip items=vec![
                ("Mode", "Detail".into()),
                ("Surface", "Hierarchy record".into()),
                ("State", "Loading record".into()),
            ]/>
            <Panel
                title="Organization Detail"
                description="Summary, path, metadata, and related work for the selected organization record load here."
            >
                <div class="actions">
                    <a class="button-link button is-light" href="/app/organization">"Back to Explorer"</a>
                    <a
                        class="button-link button is-light"
                        href=move || format!("/app/organization/{}/edit", node_id_value.get_value())
                    >
                        "Edit"
                    </a>
                </div>
                <p id="organization-detail-status" class="muted">"Loading organization detail..."</p>
                <section id="organization-detail" class="app-screen box entity-page">
                    <div class="page-title-row">
                        <div>
                            <h2 id="organization-detail-heading">"Organization Detail"</h2>
                            <p id="organization-detail-context" class="muted">
                                "Loading visible hierarchy context."
                            </p>
                        </div>
                    </div>
                </section>
                <section class="page-panel nested-form-panel">
                    <h3>"Visible Path"</h3>
                    <div id="organization-detail-path">
                        <p class="muted">"Loading path..."</p>
                    </div>
                </section>
                <section class="page-panel nested-form-panel">
                    <h3>"Summary"</h3>
                    <div id="organization-summary">
                        <p class="muted">"Loading summary..."</p>
                    </div>
                </section>
                <section class="page-panel nested-form-panel">
                    <h3>"Metadata"</h3>
                    <div id="organization-metadata">
                        <p class="muted">"Loading metadata..."</p>
                    </div>
                </section>
                <section class="page-panel nested-form-panel">
                    <h3>"Child Actions"</h3>
                    <div id="organization-child-actions">
                        <p class="muted">"Loading available child actions..."</p>
                    </div>
                </section>
                <section class="page-panel nested-form-panel">
                    <h3>"Related Records"</h3>
                    <div id="organization-related">
                        <p class="muted">"Loading related records..."</p>
                    </div>
                </section>
            </Panel>
        </NativePage>
    }
}

#[component]
pub fn OrganizationEditPage() -> impl IntoView {
    let NodeRouteParams { node_id } = require_route_params();
    let _node_id_for_load = node_id.clone();
    let node_id_value = StoredValue::new(node_id.clone());
    let session = use_account_session();
    #[cfg(not(feature = "hydrate"))]
    let _ = session;
    #[cfg(feature = "hydrate")]
    Effect::new(move |_| {
        if session.loaded.get() && session.account.get().is_some() {
            hydrate::set_context("organization-edit", Some(_node_id_for_load.clone()));
            hydrate::load_edit_page(_node_id_for_load.clone());
        }
    });

    view! {
        <NativePage
            title="Edit Organization"
            description="Edit a runtime organization record."
            page_key="organization-edit"
            active_route="organization"
            workspace_label="Product Area"
            record_id=node_id.clone()
            required_capability="admin:all"
            breadcrumbs=vec![
                BreadcrumbItem::link("Home", "/app"),
                BreadcrumbItem::link("Organization", "/app/organization"),
                BreadcrumbItem::link("Organization Detail", format!("/app/organization/{node_id}")),
                BreadcrumbItem::current("Edit Organization"),
            ]
        >
            <PageHeader
                eyebrow="Organization"
                title="Edit Organization"
                description="Update the selected hierarchy record and return to the explorer with changes in context."
            />
            <MetadataStrip items=vec![
                ("Mode", "Edit".into()),
                ("Surface", "Organization authoring".into()),
                ("State", "Loading record".into()),
            ]/>
            <Panel
                title="Organization Record"
                description="Update the selected hierarchy record from a calmer edit surface that stays aligned with the explorer flow."
            >
                <p id="organization-form-status" class="muted">"Loading organization schema."</p>
                <form id="organization-form" class="entity-form">
                    <div class="form-grid">
                        <div class="form-field">
                            <label id="organization-node-type-label" for="organization-node-type">
                                "Node Type"
                            </label>
                            <select class="input" id="organization-node-type"></select>
                        </div>
                        <div class="form-field">
                            <label id="organization-parent-node-label" for="organization-parent-node">
                                "Parent Organization"
                            </label>
                            <select class="input" id="organization-parent-node"></select>
                        </div>
                        <div class="form-field wide-field">
                            <label id="organization-name-label" for="organization-name">"Name"</label>
                            <input class="input" id="organization-name" type="text" autocomplete="off" />
                        </div>
                    </div>
                    <section class="page-panel nested-form-panel">
                        <h3 id="organization-metadata-title">"Metadata"</h3>
                        <p id="organization-metadata-context" class="muted">
                            "Metadata inputs are generated from the selected node type."
                        </p>
                        <div id="organization-metadata-fields" class="form-grid">
                            <p class="muted">"Loading metadata fields..."</p>
                        </div>
                    </section>
                    <div class="actions">
                        <button class="button is-primary" type="submit">"Save Record"</button>
                        <a
                            class="button-link button is-light"
                            href=move || format!("/app/organization/{}", node_id_value.get_value())
                        >
                            "Cancel"
                        </a>
                    </div>
                </form>
            </Panel>
        </NativePage>
    }
}

#[cfg(test)]
mod tests {
    use super::{ScopeNodeContext, TreeNodeContext, derive_destination_label};

    #[test]
    fn organization_scope_title_prefers_top_level_scope_over_deeper_scope_nodes() {
        let nodes = vec![
            TreeNodeContext {
                id: "partner-id".into(),
                node_type_name: Some("Partner".into()),
                parent_node_id: None,
            },
            TreeNodeContext {
                id: "program-id".into(),
                node_type_name: Some("Program".into()),
                parent_node_id: Some("partner-id".into()),
            },
            TreeNodeContext {
                id: "activity-id".into(),
                node_type_name: Some("Activity".into()),
                parent_node_id: Some("program-id".into()),
            },
        ];
        let scopes = vec![
            ScopeNodeContext {
                node_id: Some("activity-id".into()),
                node_type_name: Some("Activity".into()),
                scope_node_type_name: None,
            },
            ScopeNodeContext {
                node_id: Some("partner-id".into()),
                node_type_name: Some("Partner".into()),
                scope_node_type_name: None,
            },
        ];

        let label = derive_destination_label(&scopes, &nodes);

        assert_eq!(label, "Partner Explorer");
    }

    #[test]
    fn organization_scope_title_handles_missing_tree_rows_with_scope_fallbacks() {
        let nodes = vec![TreeNodeContext {
            id: "orphan-child-id".into(),
            node_type_name: Some("Session".into()),
            parent_node_id: Some("missing-parent-id".into()),
        }];
        let scopes = vec![
            ScopeNodeContext {
                node_id: Some("orphan-child-id".into()),
                node_type_name: Some("Session".into()),
                scope_node_type_name: None,
            },
            ScopeNodeContext {
                node_id: Some("missing-scope-id".into()),
                node_type_name: None,
                scope_node_type_name: Some("Partner".into()),
            },
        ];

        let label = derive_destination_label(&scopes, &nodes);

        assert_eq!(label, "Partner Explorer");
    }
}
