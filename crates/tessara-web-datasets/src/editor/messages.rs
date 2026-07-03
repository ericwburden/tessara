//! Dataset editor status messages.

use leptos::prelude::*;

#[component]
pub(crate) fn DatasetEditorMessages(
    load_error: RwSignal<Option<String>>,
    save_error: RwSignal<Option<String>>,
    save_message: RwSignal<Option<String>>,
    editor_ready: RwSignal<bool>,
) -> impl IntoView {
    view! {
        {move || {
            if !editor_ready.get() && load_error.get().is_none() {
                Some(view! { <p class="form-status">"Loading dataset editor data."</p> })
            } else {
                None
            }
        }}
        {move || load_error.get().map(|message| view! { <p class="form-status is-error">{message}</p> })}
        {move || save_error.get().map(|message| view! { <p class="form-status is-error">{message}</p> })}
        {move || save_message.get().map(|message| view! { <p class="form-status is-success">{message}</p> })}
    }
}
