//! Dataset editor identity fields.

use leptos::prelude::*;

#[component]
pub(crate) fn DatasetIdentitySection(
    name: RwSignal<String>,
    slug: RwSignal<String>,
) -> impl IntoView {
    view! {
        <section class="route-panel__section dataset-editor-section">
            <h3>"Dataset Definition"</h3>
            <div class="form-grid">
                <label class="form-field">
                    <span>"Name"</span>
                    <input
                        required
                        prop:value=move || name.get()
                        on:change=move |event| {
                            commit_name(name, slug, event_target_value(&event));
                        }
                        on:blur=move |event| {
                            commit_name(name, slug, event_target_value(&event));
                        }
                    />
                </label>
                <label class="form-field">
                    <span>"Slug"</span>
                    <input
                        required
                        prop:value=move || slug.get()
                        on:change=move |event| slug.set(event_target_value(&event))
                        on:blur=move |event| slug.set(event_target_value(&event))
                    />
                </label>
            </div>
        </section>
    }
}

fn commit_name(name: RwSignal<String>, slug: RwSignal<String>, value: String) {
    let derived_slug = snake_case_slug(&value);
    name.set(value);

    if slug.get_untracked().trim().is_empty() && !derived_slug.is_empty() {
        slug.set(derived_slug);
    }
}

fn snake_case_slug(value: &str) -> String {
    let mut slug = String::new();
    let mut previous_was_separator = true;

    for character in value.chars() {
        if character.is_ascii_alphanumeric() {
            slug.push(character.to_ascii_lowercase());
            previous_was_separator = false;
        } else if !previous_was_separator && !slug.is_empty() {
            slug.push('_');
            previous_was_separator = true;
        }
    }

    slug.trim_end_matches('_').to_string()
}

#[cfg(test)]
mod tests {
    use super::snake_case_slug;

    #[test]
    fn snake_case_slug_normalizes_dataset_names() {
        assert_eq!(snake_case_slug("UAT Dataset"), "uat_dataset");
        assert_eq!(
            snake_case_slug(" Demo Partner: Snapshot 2026 "),
            "demo_partner_snapshot_2026"
        );
        assert_eq!(snake_case_slug("Already_snake"), "already_snake");
    }
}
