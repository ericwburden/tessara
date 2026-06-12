//! Option loader for the form edit page.

#[cfg(feature = "hydrate")]
use crate::features::forms::api::{FormsApiError, fetch_form_edit_options};
#[cfg(feature = "hydrate")]
use crate::features::forms::builder::hydrate_form_builder_from_rendered;
use crate::features::forms::builder::{FormBuilderFieldDraft, FormBuilderSectionDraft};
use crate::features::forms::{FormDefinition, FormSummary, RenderedForm};
use crate::features::organization::NodeTypeCatalogEntry;
#[cfg(feature = "hydrate")]
use crate::http::redirect_to_login;
use leptos::prelude::*;

#[allow(clippy::too_many_arguments)]
pub(crate) fn load_form_edit_options(
    form_id: String,
    node_types: RwSignal<Vec<NodeTypeCatalogEntry>>,
    existing_forms: RwSignal<Vec<FormSummary>>,
    detail: RwSignal<Option<FormDefinition>>,
    rendered_form: RwSignal<Option<RenderedForm>>,
    edit_version_id: RwSignal<Option<String>>,
    edit_version_status: RwSignal<Option<String>>,
    name: RwSignal<String>,
    workflow_node_type_id: RwSignal<String>,
    builder_sections: RwSignal<Vec<FormBuilderSectionDraft>>,
    builder_fields: RwSignal<Vec<FormBuilderFieldDraft>>,
    active_builder_section: RwSignal<String>,
    next_builder_section_id: RwSignal<usize>,
    next_builder_field_id: RwSignal<usize>,
    is_loading: RwSignal<bool>,
    message: RwSignal<Option<String>>,
) {
    #[cfg(feature = "hydrate")]
    {
        leptos::task::spawn_local(async move {
            is_loading.set(true);
            message.set(None);
            detail.set(None);
            rendered_form.set(None);
            edit_version_id.set(None);
            edit_version_status.set(None);

            match fetch_form_edit_options(&form_id).await {
                Ok(options) => {
                    let (sections, fields, next_section, next_field) =
                        hydrate_form_builder_from_rendered(options.rendered_form.as_ref());
                    let active_section = sections
                        .first()
                        .map(|section| section.id.to_string())
                        .unwrap_or_else(|| "1".to_string());

                    name.set(options.detail.name.clone());
                    workflow_node_type_id.set(
                        options
                            .detail
                            .scope_node_type_id
                            .clone()
                            .unwrap_or_default(),
                    );
                    edit_version_id.set(options.edit_version_id);
                    edit_version_status.set(options.edit_version_status);
                    active_builder_section.set(active_section);
                    next_builder_section_id.set(next_section);
                    next_builder_field_id.set(next_field);
                    builder_sections.set(sections);
                    builder_fields.set(fields);
                    rendered_form.set(options.rendered_form);
                    detail.set(Some(options.detail));
                    node_types.set(options.node_types);
                    existing_forms.set(options.existing_forms);
                    is_loading.set(false);
                }
                Err(FormsApiError::Unauthorized) => {
                    is_loading.set(false);
                    redirect_to_login();
                }
                Err(FormsApiError::Message(error)) => {
                    is_loading.set(false);
                    message.set(Some(error));
                }
            }
        });
    }

    #[cfg(not(feature = "hydrate"))]
    {
        let _ = (
            form_id,
            node_types,
            existing_forms,
            detail,
            rendered_form,
            edit_version_id,
            edit_version_status,
            name,
            workflow_node_type_id,
            builder_sections,
            builder_fields,
            active_builder_section,
            next_builder_section_id,
            next_builder_field_id,
            is_loading,
            message,
        );
    }
}
