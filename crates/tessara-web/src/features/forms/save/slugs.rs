//! Slug lookup helpers for form save validation.

use crate::features::forms::FormSummary;

pub(super) fn existing_form_slugs(forms: &[FormSummary]) -> Vec<String> {
    forms.iter().map(|form| form.slug.clone()).collect()
}

pub(super) fn existing_form_slugs_for_update(
    forms: &[FormSummary],
    current_form_id: &str,
) -> Vec<String> {
    forms
        .iter()
        .filter(|form| form.id != current_form_id)
        .map(|form| form.slug.clone())
        .collect()
}
