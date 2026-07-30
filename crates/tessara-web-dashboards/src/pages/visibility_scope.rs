use leptos::prelude::*;
use tessara_module_ui::{SideSheet, SideSheetSide, TableSearch};

use crate::types::DashboardVisibilityNode;

#[component]
pub(super) fn DashboardVisibilityScopeSheet(
    #[prop(into)] id: String,
    #[prop(into)] dashboard_name: String,
    nodes: Vec<DashboardVisibilityNode>,
    open: Signal<bool>,
    on_close: Callback<()>,
) -> impl IntoView {
    let search = RwSignal::new(String::new());
    let nodes = StoredValue::new(nodes);
    let close = Callback::new(move |_| {
        search.set(String::new());
        on_close.run(());
    });

    view! {
        <SideSheet
            id
            title="Visibility scope"
            description=dashboard_name
            eyebrow="Dashboard"
            open
            on_close=close
            side=SideSheetSide::End
            close_label="Close visibility scope"
            class="dashboard-scope-sheet"
        >
            <section class="sheet-panel__section dashboard-scope-sheet__section">
                <TableSearch
                    value=Signal::derive(move || search.get())
                    on_input=Callback::new(move |value| search.set(value))
                    label="Search visibility nodes"
                    placeholder="Search visibility nodes"
                />
                <p class="dashboard-scope-sheet__count" aria-live="polite">
                    {move || {
                        let nodes = nodes.get_value();
                        let shown = filter_visibility_nodes(&nodes, &search.get()).len();
                        format!("{} of {} shown", shown, nodes.len())
                    }}
                </p>
                {move || {
                    let all_nodes = nodes.get_value();
                    let shown_nodes = filter_visibility_nodes(&all_nodes, &search.get());
                    if shown_nodes.is_empty() {
                        let message = if all_nodes.is_empty() {
                            "This Dashboard has no visibility scope."
                        } else {
                            "No visibility nodes match this search."
                        };
                        view! { <p>{message}</p> }.into_any()
                    } else {
                        view! {
                            <ul class="dashboard-scope-sheet__list">
                                {shown_nodes.into_iter().map(|node| {
                                    let href = visibility_node_href(&node.node_id);
                                    view! {
                                        <li>
                                            <a href=href><strong>{node.node_name}</strong></a>
                                            <span>{format!("{} · {}", node.node_type_name, node.node_path)}</span>
                                        </li>
                                    }
                                }).collect_view()}
                            </ul>
                        }.into_any()
                    }
                }}
            </section>
        </SideSheet>
    }
}

fn visibility_node_href(node_id: &str) -> String {
    format!("/organization/{node_id}")
}

fn filter_visibility_nodes(
    nodes: &[DashboardVisibilityNode],
    query: &str,
) -> Vec<DashboardVisibilityNode> {
    let query = query.trim().to_lowercase();
    nodes
        .iter()
        .filter(|node| {
            query.is_empty()
                || node.node_name.to_lowercase().contains(&query)
                || node.node_path.to_lowercase().contains(&query)
                || node.node_type_name.to_lowercase().contains(&query)
        })
        .cloned()
        .collect()
}

#[cfg(test)]
mod tests {
    use super::{filter_visibility_nodes, visibility_node_href};
    use crate::types::DashboardVisibilityNode;

    fn node(id: &str, name: &str, kind: &str, path: &str) -> DashboardVisibilityNode {
        DashboardVisibilityNode {
            node_id: id.to_string(),
            node_name: name.to_string(),
            node_type_name: kind.to_string(),
            parent_node_id: None,
            node_path: path.to_string(),
        }
    }

    #[test]
    fn search_matches_name_type_and_path() {
        let nodes = vec![
            node(
                "north",
                "North Star Services",
                "Partner",
                "Demo Program / North Star Services",
            ),
            node(
                "mentoring",
                "Youth Mentoring",
                "Program",
                "Demo Program / Youth Mentoring",
            ),
        ];

        assert_eq!(filter_visibility_nodes(&nodes, "north").len(), 1);
        assert_eq!(filter_visibility_nodes(&nodes, "partner").len(), 1);
        assert_eq!(filter_visibility_nodes(&nodes, "demo program").len(), 2);
        assert_eq!(filter_visibility_nodes(&nodes, "missing").len(), 0);
    }

    #[test]
    fn nodes_link_to_the_canonical_application_route() {
        assert_eq!(visibility_node_href("node-123"), "/organization/node-123");
    }
}
