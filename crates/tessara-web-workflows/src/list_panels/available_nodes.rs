//! Available-node panels for the workflow list.

use crate::shared::{FormAttachmentLink, WorkflowAvailableNodesSheetData, node_count_label};
use icons::{ExternalLink, PanelRight};
use leptos::prelude::*;
use tessara_module_ui::{SideSheet, TableSearch, empty_view};

const AVAILABLE_NODES_SHEET_ID: &str = "workflow-available-nodes-sheet";

#[component]
pub(crate) fn WorkflowAvailableNodesList(
    nodes: Vec<FormAttachmentLink>,
    workflow_name: String,
    workflow_href: String,
    sheet: RwSignal<Option<WorkflowAvailableNodesSheetData>>,
) -> impl IntoView {
    let total_nodes = nodes.len();
    let nodes_for_sheet = nodes.clone();
    let workflow_name_for_sheet = workflow_name.clone();
    let workflow_href_for_sheet = workflow_href.clone();
    let workflow_href_for_expanded = workflow_href;

    view! {
        <div class="forms-attached-list">
            {if total_nodes == 0 {
                view! { <p>"Not available"</p> }.into_any()
            } else {
                view! {
                    <button
                        class="forms-attached-list__more"
                        type="button"
                        aria-label=format!("View available organization nodes for {workflow_name_for_sheet}")
                        aria-haspopup="dialog"
                        aria-controls=AVAILABLE_NODES_SHEET_ID
                        aria-expanded=move || sheet
                            .get()
                            .as_ref()
                            .is_some_and(|detail| detail.workflow_href == workflow_href_for_expanded)
                            .to_string()
                        title="Opens detail panel"
                        on:click=move |_| {
                            sheet.set(Some(WorkflowAvailableNodesSheetData {
                                workflow_name: workflow_name_for_sheet.clone(),
                                workflow_href: workflow_href_for_sheet.clone(),
                                nodes: nodes_for_sheet.clone(),
                            }));
                        }
                    >
                        <span>{node_count_label(total_nodes)}</span>
                        <PanelRight class="forms-attached-list__icon"/>
                    </button>
                }
                .into_any()
            }}
        </div>
    }
}

#[component]
pub(crate) fn WorkflowAvailableNodesSheet(
    detail: RwSignal<Option<WorkflowAvailableNodesSheetData>>,
) -> impl IntoView {
    let search = RwSignal::new(String::new());
    let close = Callback::new(move |_| {
        detail.set(None);
        search.set(String::new());
    });
    let open = Signal::derive(move || detail.get().is_some());
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
        <SideSheet
            id=AVAILABLE_NODES_SHEET_ID
            title="Available organization nodes"
            open
            on_close=close
            close_label="Close available nodes"
            class="forms-attached-sheet"
        >
            {move || {
                detail
                    .get()
                    .map(|data| {
                        let total = data.nodes.len();
                        view! {
                            <a class="icon-button sheet-panel__open forms-attached-sheet__open" href=data.workflow_href aria-label="Open workflow detail" title="Open workflow detail">
                                <ExternalLink class="icon-button__icon"/>
                            </a>
                            <header class="sheet-panel__header">
                                <p>"Available Nodes"</p>
                                <h2>{data.workflow_name}</h2>
                                <span class="forms-attached-sheet__count">{format!("{total} nodes")}</span>
                            </header>
                            <section class="sheet-panel__section">
                                <TableSearch
                                    value=Signal::derive(move || search.get())
                                    on_input=Callback::new(move |value| search.set(value))
                                    label="Search available nodes"
                                    placeholder="Search available nodes"
                                    class="forms-attached-sheet__search"
                                />
                                <div class="forms-attached-sheet__list">
                                    {move || {
                                        let nodes = filtered_nodes();
                                        if nodes.is_empty() {
                                            view! { <p class="forms-attached-sheet__empty">"No Available Nodes to Display"</p> }.into_any()
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
        </SideSheet>
    }
}

#[cfg(all(test, feature = "ssr"))]
mod tests {
    use super::*;

    #[test]
    fn available_nodes_trigger_exposes_its_dialog_relationship() {
        let html = Owner::new().with(|| {
            let href = "/workflows/workflow-1".to_string();
            let node = FormAttachmentLink {
                href: "/organization/nodes/node-1".to_string(),
                label: "Operations".to_string(),
                title: "Organization node".to_string(),
            };
            let sheet = RwSignal::new(Some(WorkflowAvailableNodesSheetData {
                workflow_name: "Inspection".to_string(),
                workflow_href: href.clone(),
                nodes: vec![node.clone()],
            }));

            view! {
                <WorkflowAvailableNodesList
                    nodes=vec![node]
                    workflow_name="Inspection".to_string()
                    workflow_href=href
                    sheet
                />
            }
            .to_html()
        });

        assert!(html.contains("aria-haspopup=\"dialog\""));
        assert!(html.contains(&format!("aria-controls=\"{AVAILABLE_NODES_SHEET_ID}\"")));
        assert!(html.contains("aria-expanded=\"true\""));
    }
}
