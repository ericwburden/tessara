//! Workflow editor identity fields.

use leptos::prelude::*;

#[component]
pub(in crate::features::workflows) fn WorkflowIdentityFields(
    name: RwSignal<String>,
    description: RwSignal<String>,
) -> impl IntoView {
    view! {
        <div class="form-grid">
            <label class="form-field">
                <span>"Workflow Name"</span>
                <input
                    type="text"
                    value=move || name.get()
                    on:input=move |event| {
                        name.set(event_target_value(&event));
                    }
                />
            </label>
            <label class="form-field">
                <span>"Description"</span>
                <textarea
                    prop:value=move || description.get()
                    on:input=move |event| {
                        description.set(event_target_value(&event));
                    }
                ></textarea>
            </label>
        </div>
    }
}
