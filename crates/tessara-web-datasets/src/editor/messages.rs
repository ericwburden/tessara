//! Dataset editor status messages.

use leptos::prelude::*;

#[component]
pub(crate) fn DatasetEditorMessages(
    load_error: RwSignal<Option<String>>,
    save_error: RwSignal<Option<String>>,
    save_message: RwSignal<Option<String>>,
) -> impl IntoView {
    view! {
        {move || load_error.get().map(|message| view! { <p class="form-status is-error">{message}</p> })}
        {move || save_error.get().map(|message| view! { <p class="form-status is-error">{message}</p> })}
        {move || save_message.get().map(|message| view! { <p class="form-status is-success">{message}</p> })}
    }
}
