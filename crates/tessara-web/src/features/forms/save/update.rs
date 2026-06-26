//! Form update save orchestration.

use crate::features::forms::builder::{FormBuilderFieldDraft, FormBuilderSectionDraft};
#[cfg(feature = "hydrate")]
use crate::features::forms::save::api::{
    create_draft_form_version, delete_form_field, delete_form_section, publish_form_version,
    update_form,
};
#[cfg(feature = "hydrate")]
use crate::features::forms::save::drafts::prepare_update_form_save;
#[cfg(feature = "hydrate")]
use crate::features::forms::save::structure::{save_form_fields, save_form_sections};
use crate::features::forms::types::{FormSummary, RenderedForm};
use leptos::prelude::*;

#[cfg_attr(not(feature = "hydrate"), allow(dead_code))]
pub(crate) struct SubmitUpdateFormInput {
    pub(crate) form_id: String,
    pub(crate) name: RwSignal<String>,
    pub(crate) workflow_node_type_id: RwSignal<String>,
    pub(crate) sections: RwSignal<Vec<FormBuilderSectionDraft>>,
    pub(crate) fields: RwSignal<Vec<FormBuilderFieldDraft>>,
    pub(crate) existing_forms: RwSignal<Vec<FormSummary>>,
    pub(crate) edit_version_id: RwSignal<Option<String>>,
    pub(crate) edit_version_status: RwSignal<Option<String>>,
    pub(crate) rendered_form: RwSignal<Option<RenderedForm>>,
    pub(crate) is_saving: RwSignal<bool>,
    pub(crate) message: RwSignal<Option<String>>,
    pub(crate) publish_after_save: bool,
}

/// Submits the submit update form request.
pub(crate) fn submit_update_form(input: SubmitUpdateFormInput) {
    #[cfg(feature = "hydrate")]
    {
        let SubmitUpdateFormInput {
            form_id,
            name,
            workflow_node_type_id,
            sections,
            fields,
            existing_forms,
            edit_version_id,
            edit_version_status,
            rendered_form,
            is_saving,
            message,
            publish_after_save,
        } = input;

        if is_saving.get() {
            return;
        }

        let prepared_save = match prepare_update_form_save(
            &form_id,
            name.get().trim().to_string(),
            workflow_node_type_id.get().trim().to_string(),
            &sections.get_untracked(),
            &fields.get_untracked(),
            existing_forms.get_untracked().as_slice(),
            rendered_form.get_untracked(),
        ) {
            Ok(prepared_save) => prepared_save,
            Err(error) => {
                message.set(Some(error));
                return;
            }
        };
        let payload = prepared_save.payload;
        let prepared_sections = prepared_save.sections;
        let prepared_fields = prepared_save.fields;
        let original_section_ids = prepared_save.original_section_ids;
        let original_field_ids = prepared_save.original_field_ids;
        let kept_section_ids = prepared_save.kept_section_ids;
        let kept_field_ids = prepared_save.kept_field_ids;
        let update_existing_draft = edit_version_status.get_untracked().as_deref() == Some("draft");
        let existing_version_id = edit_version_id.get_untracked();

        leptos::task::spawn_local(async move {
            is_saving.set(true);
            message.set(None);

            if let Err(error) = update_form(&form_id, payload).await {
                message.set(Some(error));
                is_saving.set(false);
                return;
            }

            let version_id = if update_existing_draft {
                match existing_version_id {
                    Some(version_id) => version_id,
                    None => {
                        message.set(Some("No editable draft version was available.".into()));
                        is_saving.set(false);
                        return;
                    }
                }
            } else {
                match create_draft_form_version(&form_id).await {
                    Ok(created) => created.id,
                    Err(error) => {
                        message.set(Some(error));
                        is_saving.set(false);
                        return;
                    }
                }
            };

            if update_existing_draft {
                for field_id in original_field_ids.difference(&kept_field_ids) {
                    if let Err(error) = delete_form_field(&version_id, field_id).await {
                        message.set(Some(error));
                        is_saving.set(false);
                        return;
                    }
                }

                for section_id in original_section_ids.difference(&kept_section_ids) {
                    if let Err(error) = delete_form_section(section_id).await {
                        message.set(Some(error));
                        is_saving.set(false);
                        return;
                    }
                }
            }

            let section_ids =
                match save_form_sections(&version_id, &prepared_sections, update_existing_draft)
                    .await
                {
                    Ok(section_ids) => section_ids,
                    Err(error) => {
                        message.set(Some(error));
                        is_saving.set(false);
                        return;
                    }
                };

            if let Err(error) = save_form_fields(
                &version_id,
                &prepared_fields,
                &section_ids,
                update_existing_draft,
            )
            .await
            {
                message.set(Some(error));
                is_saving.set(false);
                return;
            }

            if publish_after_save && let Err(error) = publish_form_version(&version_id).await {
                message.set(Some(error));
                is_saving.set(false);
                return;
            }

            if let Some(window) = web_sys::window() {
                let _ = window.location().set_href(&format!("/forms/{form_id}"));
            }
        });
    }

    #[cfg(not(feature = "hydrate"))]
    {
        let _ = input;
    }
}
