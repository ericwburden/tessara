//! Response values table component.

use crate::display::response_value_label;
use crate::metadata::metadata_label;
use crate::types::SubmissionValueDetail;
use leptos::prelude::*;
use tessara_module_ui::DataTable;

#[component]
pub(crate) fn ResponseValuesTable(values: Vec<SubmissionValueDetail>) -> impl IntoView {
    view! {
        <DataTable>
            <thead>
                <tr>
                    <th scope="col">"Field"</th>
                    <th scope="col">"Type"</th>
                    <th scope="col">"Value"</th>
                </tr>
            </thead>
            <tbody>
                {if values.is_empty() {
                    view! {
                        <tr>
                            <td class="data-table__empty" colspan="3">"No Response Values to Display"</td>
                        </tr>
                    }
                    .into_any()
                } else {
                    values
                        .into_iter()
                        .map(|value| {
                            let rendered_value = response_value_label(value.value.as_ref());
                            view! {
                                <tr>
                                    <th scope="row">{value.label}</th>
                                    <td>{metadata_label(&value.field_type)}</td>
                                    <td>{rendered_value}</td>
                                </tr>
                            }
                        })
                        .collect_view()
                        .into_any()
                }}
            </tbody>
        </DataTable>
    }
}
