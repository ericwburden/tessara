//! Query-string seeding helpers for workflow creation.

use crate::features::forms::FormSummary;
use crate::features::workflows::types::WorkflowStepDraft;
#[cfg(feature = "hydrate")]
use crate::utils::url::current_search_param;
use leptos::prelude::*;

pub(in crate::features::workflows) fn seed_workflow_from_form_query(
    is_loading: RwSignal<bool>,
    seeded_from_form: RwSignal<bool>,
    forms: RwSignal<Vec<FormSummary>>,
    name: RwSignal<String>,
    description: RwSignal<String>,
    steps: RwSignal<Vec<WorkflowStepDraft>>,
    next_step_id: RwSignal<usize>,
) {
    if is_loading.get() || seeded_from_form.get_untracked() {
        return;
    }

    let form_id: Option<String> = {
        #[cfg(feature = "hydrate")]
        {
            current_search_param("form_id")
        }
        #[cfg(not(feature = "hydrate"))]
        {
            None
        }
    };
    let Some(form_id) = form_id else {
        seeded_from_form.set(true);
        return;
    };

    let available_forms = forms.get();
    let Some(form) = available_forms.iter().find(|form| form.id == form_id) else {
        seeded_from_form.set(true);
        return;
    };
    let Some(version) = form
        .versions
        .iter()
        .find(|version| version.status == "published")
    else {
        seeded_from_form.set(true);
        return;
    };

    name.set(format!("{} Workflow", form.name));
    description.set(format!("Workflow for {}.", form.name));
    steps.set(vec![WorkflowStepDraft {
        id: 1,
        title: format!("{} Response", form.name),
        form_version_id: version.id.clone(),
    }]);
    next_step_id.set(2);
    seeded_from_form.set(true);
}
