//! Form create save orchestration.

use crate::features::forms::builder::{FormBuilderFieldDraft, FormBuilderSectionDraft};
#[cfg(feature = "hydrate")]
use crate::features::forms::save::api::{
    FormSaveApiError, create_form, create_initial_form_version, publish_form_version,
};
#[cfg(feature = "hydrate")]
use crate::features::forms::save::create_structure::{
    FormStructureSaveError, create_form_fields_for_new_form, create_form_sections_for_new_form,
};
#[cfg(feature = "hydrate")]
use crate::features::forms::save::drafts::prepare_create_form_save;
use crate::features::forms::types::FormSummary;
#[cfg(feature = "hydrate")]
use crate::http::redirect_to_login;
use leptos::prelude::*;

#[cfg_attr(not(feature = "hydrate"), allow(dead_code))]
pub(crate) struct SubmitCreateFormInput {
    pub(crate) name: RwSignal<String>,
    pub(crate) workflow_node_type_id: RwSignal<String>,
    pub(crate) sections: RwSignal<Vec<FormBuilderSectionDraft>>,
    pub(crate) fields: RwSignal<Vec<FormBuilderFieldDraft>>,
    pub(crate) existing_forms: RwSignal<Vec<FormSummary>>,
    pub(crate) is_saving: RwSignal<bool>,
    pub(crate) message: RwSignal<Option<String>>,
    pub(crate) publish_after_save: bool,
}

/// Submits the submit create form request.
pub(crate) fn submit_create_form(input: SubmitCreateFormInput) {
    #[cfg(feature = "hydrate")]
    {
        let SubmitCreateFormInput {
            name,
            workflow_node_type_id,
            sections,
            fields,
            existing_forms,
            is_saving,
            message,
            publish_after_save,
        } = input;

        if is_saving.get() {
            return;
        }

        let prepared_save = match prepare_create_form_save(
            name.get().trim().to_string(),
            workflow_node_type_id.get().trim().to_string(),
            &sections.get_untracked(),
            &fields.get_untracked(),
            existing_forms.get_untracked().as_slice(),
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

        leptos::task::spawn_local(async move {
            is_saving.set(true);
            message.set(None);

            match create_form(payload).await {
                Ok(created) => {
                    let created_version = match create_initial_form_version(&created.id).await {
                        Ok(created_version) => created_version,
                        Err(FormSaveApiError::Unauthorized) => {
                            is_saving.set(false);
                            redirect_to_login();
                            return;
                        }
                        Err(FormSaveApiError::Message(error)) => {
                            message.set(Some(error));
                            is_saving.set(false);
                            return;
                        }
                    };

                    let section_ids = match create_form_sections_for_new_form(
                        &created_version.id,
                        &prepared_sections,
                    )
                    .await
                    {
                        Ok(section_ids) => section_ids,
                        Err(FormStructureSaveError::Unauthorized) => {
                            is_saving.set(false);
                            redirect_to_login();
                            return;
                        }
                        Err(FormStructureSaveError::Message(error)) => {
                            message.set(Some(error));
                            is_saving.set(false);
                            return;
                        }
                    };

                    if let Err(error) = create_form_fields_for_new_form(
                        &created_version.id,
                        &prepared_fields,
                        &section_ids,
                    )
                    .await
                    {
                        match error {
                            FormStructureSaveError::Unauthorized => {
                                is_saving.set(false);
                                redirect_to_login();
                            }
                            FormStructureSaveError::Message(error) => {
                                message.set(Some(error));
                                is_saving.set(false);
                            }
                        }
                        return;
                    }

                    if publish_after_save
                        && let Err(error) = publish_form_version(&created_version.id).await
                    {
                        message.set(Some(error));
                        is_saving.set(false);
                        return;
                    }

                    if let Some(window) = web_sys::window() {
                        let _ = window
                            .location()
                            .set_href(&format!("/forms/{}", created.id));
                    }
                }
                Err(FormSaveApiError::Unauthorized) => {
                    is_saving.set(false);
                    redirect_to_login();
                }
                Err(FormSaveApiError::Message(error)) => {
                    message.set(Some(error));
                    is_saving.set(false);
                }
            }
        });
    }

    #[cfg(not(feature = "hydrate"))]
    {
        let _ = input;
    }
}
