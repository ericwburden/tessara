//! Shared segmented toggle control.

use leptos::prelude::*;

#[derive(Clone)]
pub(crate) struct SegmentedToggleOption {
    pub(crate) value: &'static str,
    pub(crate) label: &'static str,
}

#[component]
pub(crate) fn SegmentedToggle(
    active: Signal<String>,
    options: Vec<SegmentedToggleOption>,
    on_select: Callback<String>,
    #[prop(default = "Segmented options")] aria_label: &'static str,
    #[prop(optional)] class: &'static str,
) -> impl IntoView {
    let select_options = options.clone();
    let class_name = if class.is_empty() {
        "segmented-toggle".to_string()
    } else {
        format!("segmented-toggle {class}")
    };

    view! {
        <div class=class_name role="group" aria-label=aria_label>
            {options
                .into_iter()
                .map(|option| {
                    let value = option.value.to_string();
                    let value_for_class = value.clone();
                    let value_for_click = value.clone();
                    view! {
                        <button
                            class=move || if active.get() == value_for_class {
                                "segmented-toggle__option is-active"
                            } else {
                                "segmented-toggle__option"
                            }
                            type="button"
                            on:click=move |_| on_select.run(value_for_click.clone())
                        >
                            {option.label}
                        </button>
                    }
                })
                .collect_view()}
            <select
                class="segmented-toggle__select"
                aria-label=aria_label
                prop:value=move || active.get()
                on:change=move |event| on_select.run(event_target_value(&event))
            >
                {select_options
                    .into_iter()
                    .map(|option| {
                        view! {
                            <option value=option.value>
                                {option.label}
                            </option>
                        }
                    })
                    .collect_view()}
            </select>
        </div>
    }
}
