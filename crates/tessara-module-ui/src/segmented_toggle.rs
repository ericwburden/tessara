//! Shared segmented toggle control.

use leptos::prelude::*;

#[derive(Clone)]
pub struct SegmentedToggleOption {
    pub value: &'static str,
    pub label: &'static str,
}

#[component]
pub fn SegmentedToggle(
    active: Signal<String>,
    options: Vec<SegmentedToggleOption>,
    on_select: Callback<String>,
    #[prop(default = "Segmented options")] aria_label: &'static str,
    #[prop(optional)] class: &'static str,
    /// Disables all options while preserving the current selection semantics.
    #[prop(default = Signal::derive(|| false), into)]
    disabled: Signal<bool>,
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
                            aria-pressed=move || (active.get() == value).to_string()
                            disabled=move || disabled.get()
                            on:click=move |_| {
                                if !disabled.get_untracked() {
                                    on_select.run(value_for_click.clone());
                                }
                            }
                        >
                            {option.label}
                        </button>
                    }
                })
                .collect_view()}
            <select
                class="segmented-toggle__select"
                aria-label=aria_label
                disabled=move || disabled.get()
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

#[cfg(all(test, feature = "ssr"))]
mod tests {
    use super::*;

    #[test]
    fn options_expose_toggle_button_selection_semantics() {
        let html = Owner::new().with(|| {
            view! {
                <SegmentedToggle
                    active=Signal::derive(|| "table".to_string())
                    options=vec![
                        SegmentedToggleOption { value: "all", label: "All" },
                        SegmentedToggleOption { value: "table", label: "Tables" },
                    ]
                    on_select=Callback::new(|_| {})
                    aria_label="Component kind"
                />
            }
            .to_html()
        });

        assert!(html.contains("role=\"group\""));
        assert!(html.contains("aria-label=\"Component kind\""));
        assert!(html.contains("aria-pressed=\"false\""));
        assert!(html.contains("aria-pressed=\"true\""));
        assert!(html.contains("segmented-toggle__option is-active"));
    }
}
