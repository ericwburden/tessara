//! Compact attached-node list trigger for form rows.

use crate::{FormAttachmentLink, FormsAttachedNodesSheetData, node_count_label};
use icons::PanelRight;
use leptos::prelude::*;
use tessara_module_ui::empty_view;

use super::ATTACHED_NODES_SHEET_ID;

#[component]
pub(crate) fn FormsAttachedNodesList(
    nodes: Vec<FormAttachmentLink>,
    form_name: String,
    form_href: String,
    sheet: RwSignal<Option<FormsAttachedNodesSheetData>>,
) -> impl IntoView {
    let total_nodes = nodes.len();
    let nodes_for_sheet = nodes.clone();
    let form_name_for_sheet = form_name.clone();
    let form_href_for_sheet = form_href.clone();
    let form_href_for_expanded = form_href;

    view! {
        <div class="forms-attached-list">
            {if total_nodes == 0 {
                view! { <p>"Not attached"</p> }.into_any()
            } else if total_nodes > 0 {
                view! {
                    <button
                        class="forms-attached-list__more"
                        type="button"
                        aria-label=format!("View attached organization nodes for {form_name_for_sheet}")
                        aria-haspopup="dialog"
                        aria-controls=ATTACHED_NODES_SHEET_ID
                        aria-expanded=move || sheet
                            .get()
                            .as_ref()
                            .is_some_and(|detail| detail.form_href == form_href_for_expanded)
                            .to_string()
                        title="Opens detail panel"
                        on:click=move |_| {
                            sheet.set(Some(FormsAttachedNodesSheetData {
                                form_name: form_name_for_sheet.clone(),
                                form_href: form_href_for_sheet.clone(),
                                nodes: nodes_for_sheet.clone(),
                            }));
                        }
                    >
                        <span>{node_count_label(total_nodes)}</span>
                        <PanelRight class="forms-attached-list__icon"/>
                    </button>
                }
                .into_any()
            } else {
                empty_view()
            }}
        </div>
    }
}

#[cfg(all(test, feature = "ssr"))]
mod tests {
    use super::*;

    #[test]
    fn attached_nodes_trigger_exposes_its_dialog_relationship() {
        let html = Owner::new().with(|| {
            let href = "/forms/form-1".to_string();
            let node = FormAttachmentLink {
                href: "/organization/nodes/node-1".to_string(),
                label: "Operations".to_string(),
                title: "Organization node".to_string(),
            };
            let sheet = RwSignal::new(Some(FormsAttachedNodesSheetData {
                form_name: "Inspection".to_string(),
                form_href: href.clone(),
                nodes: vec![node.clone()],
            }));

            view! {
                <FormsAttachedNodesList
                    nodes=vec![node]
                    form_name="Inspection".to_string()
                    form_href=href
                    sheet
                />
            }
            .to_html()
        });

        assert!(html.contains("aria-haspopup=\"dialog\""));
        assert!(html.contains(&format!("aria-controls=\"{ATTACHED_NODES_SHEET_ID}\"")));
        assert!(html.contains("aria-expanded=\"true\""));
    }
}
