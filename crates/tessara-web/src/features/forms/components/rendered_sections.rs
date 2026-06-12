//! Rendered form section detail components.

use crate::features::forms::{
    RenderedForm, rendered_field_layout_label, rendered_field_type_label,
};
use crate::ui::{DataTable, empty_view};
use leptos::prelude::*;

#[component]
pub(crate) fn RenderedFormSections(rendered_form: Option<RenderedForm>) -> impl IntoView {
    view! {
        <div class="form-detail-sections">
            {if let Some(rendered_form) = rendered_form {
                if rendered_form.sections.is_empty() {
                    view! { <p class="related-work-mobile-empty">"No Fields to Display"</p> }.into_any()
                } else {
                    rendered_form
                        .sections
                        .into_iter()
                        .map(|section| {
                            view! {
                                <article class="form-detail-section">
                                    <header>
                                        <div>
                                            <h4>{section.title}</h4>
                                            {if section.description.trim().is_empty() {
                                                empty_view()
                                            } else {
                                                view! { <p>{section.description}</p> }.into_any()
                                            }}
                                        </div>
                                    </header>
                                    <DataTable>
                                        <thead>
                                            <tr>
                                                <th scope="col">"Field"</th>
                                                <th scope="col">"Key"</th>
                                                <th scope="col">"Type"</th>
                                                <th scope="col">"Required"</th>
                                                <th scope="col">"Layout"</th>
                                            </tr>
                                        </thead>
                                        <tbody>
                                            {if section.fields.is_empty() {
                                                view! {
                                                    <tr>
                                                        <td class="data-table__empty" colspan="5">"No Fields to Display"</td>
                                                    </tr>
                                                }
                                                .into_any()
                                            } else {
                                                section
                                                    .fields
                                                    .into_iter()
                                                    .map(|field| {
                                                        let layout_label = rendered_field_layout_label(&field);
                                                        view! {
                                                            <tr>
                                                                <th scope="row">{field.label}</th>
                                                                <td>{field.key}</td>
                                                                <td>{rendered_field_type_label(&field.field_type)}</td>
                                                                <td>{if field.required { "Yes" } else { "No" }}</td>
                                                                <td>{layout_label}</td>
                                                            </tr>
                                                        }
                                                    })
                                                    .collect_view()
                                                    .into_any()
                                            }}
                                        </tbody>
                                    </DataTable>
                                </article>
                            }
                        })
                        .collect_view()
                        .into_any()
                }
            } else {
                view! { <p class="related-work-mobile-empty">"No Fields to Display"</p> }.into_any()
            }}
        </div>
    }
}
