//! Editable form surface for existing forms.

use crate::features::forms::builder::{FormBuilderCanvas, FormBuilderEditorState};
use crate::features::forms::{
    FormEditableVersionSummary, FormIdentityFields, FormSummary, RenderedForm,
};
use crate::features::organization::NodeTypeCatalogEntry;
use leptos::prelude::*;

use super::save::submit_update_form;

#[allow(clippy::too_many_arguments)]
#[component]
pub(in crate::features::forms) fn FormEditForm(
    form_id: String,
    cancel_href: String,
    name: RwSignal<String>,
    workflow_node_type_id: RwSignal<String>,
    node_types: RwSignal<Vec<NodeTypeCatalogEntry>>,
    existing_forms: RwSignal<Vec<FormSummary>>,
    rendered_form: RwSignal<Option<RenderedForm>>,
    edit_version_id: RwSignal<Option<String>>,
    edit_version_status: RwSignal<Option<String>>,
    builder_state: FormBuilderEditorState,
    is_loading: RwSignal<bool>,
    is_saving: RwSignal<bool>,
    message: RwSignal<Option<String>>,
) -> impl IntoView {
    let form_id_for_draft_submit = form_id.clone();
    let form_id_for_publish_submit = form_id;
    let builder_field_count = Memo::new(move |_| builder_state.builder_fields.get().len());
    let can_submit = move || !is_loading.get() && !is_saving.get() && !name.get().trim().is_empty();

    view! {
        <div class="form-create-workspace">
            <form
                class="native-form form-create-form"
                on:submit=move |event| {
                    event.prevent_default();
                    submit_update_form(
                        form_id_for_draft_submit.clone(),
                        name,
                        workflow_node_type_id,
                        builder_state.builder_sections,
                        builder_state.builder_fields,
                        existing_forms,
                        edit_version_id,
                        edit_version_status,
                        rendered_form,
                        is_saving,
                        message,
                        false,
                    );
                }
            >
                <FormIdentityFields
                    name=name
                    workflow_node_type_id=workflow_node_type_id
                    node_types=node_types
                />

                <FormEditableVersionSummary
                    edit_version_status=edit_version_status
                    builder_field_count=builder_field_count
                />

                <FormBuilderCanvas state=builder_state/>
                {move || message.get().map(|message| view! {
                    <p class="form-message" role="status">{message}</p>
                })}

                <div class="form-actions">
                    <a class="button" href=cancel_href.clone()>"Cancel"</a>
                    <button class="button button--secondary" type="submit" disabled=move || !can_submit()>
                        {move || if is_saving.get() { "Saving..." } else { "Save as Draft" }}
                    </button>
                    <button
                        class="button"
                        type="button"
                        disabled=move || !can_submit()
                        on:click=move |_| {
                            submit_update_form(
                                form_id_for_publish_submit.clone(),
                                name,
                                workflow_node_type_id,
                                builder_state.builder_sections,
                                builder_state.builder_fields,
                                existing_forms,
                                edit_version_id,
                                edit_version_status,
                                rendered_form,
                                is_saving,
                                message,
                                true,
                            );
                        }
                    >
                        {move || if is_saving.get() { "Publishing..." } else { "Save and Publish" }}
                    </button>
                </div>
            </form>
        </div>
    }
}
