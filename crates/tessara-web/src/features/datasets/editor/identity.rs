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
                    <input required prop:value=move || name.get() on:input=move |event| name.set(event_target_value(&event))/>
                </label>
                <label class="form-field">
                    <span>"Slug"</span>
                    <input required prop:value=move || slug.get() on:input=move |event| slug.set(event_target_value(&event))/>
                </label>
            </div>
        </section>
    }
}
