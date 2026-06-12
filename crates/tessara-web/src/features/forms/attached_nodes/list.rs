//! Compact attached-node list trigger for form rows.

use crate::features::shared::{FormAttachmentLink, FormsAttachedNodesSheetData, node_count_label};
use crate::ui::empty_view;
use icons::PanelRight;
use leptos::prelude::*;

#[component]
pub(in crate::features::forms) fn FormsAttachedNodesList(
    nodes: Vec<FormAttachmentLink>,
    form_name: String,
    form_href: String,
    sheet: RwSignal<Option<FormsAttachedNodesSheetData>>,
) -> impl IntoView {
    let total_nodes = nodes.len();
    let nodes_for_sheet = nodes.clone();
    let form_name_for_sheet = form_name.clone();
    let form_href_for_sheet = form_href.clone();

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
