//! Shared button primitives.
//!
//! The primitive deliberately follows the canonical `button--*` classes from
//! the application stylesheet. Feature crates should select a semantic variant
//! here instead of creating parallel button class dialects.

use leptos::prelude::*;

/// Visual emphasis for a [`Button`].
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum ButtonVariant {
    /// The application's primary action treatment.
    #[default]
    Primary,
    Secondary,
    Warning,
    Danger,
    Quiet,
}

impl ButtonVariant {
    fn classes(self) -> &'static [&'static str] {
        match self {
            Self::Primary => &[],
            Self::Secondary => &["button--secondary"],
            Self::Warning => &["button--secondary", "button--warning"],
            Self::Danger => &["button--danger"],
            Self::Quiet => &["button--quiet"],
        }
    }
}

/// Size treatment for a [`Button`].
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum ButtonSize {
    #[default]
    Default,
    Compact,
}

impl ButtonSize {
    fn class(self) -> Option<&'static str> {
        match self {
            Self::Default => None,
            Self::Compact => Some("button--compact"),
        }
    }
}

/// Native type used when [`Button`] renders a `<button>` element.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum ButtonType {
    #[default]
    Button,
    Submit,
    Reset,
}

impl ButtonType {
    fn attribute(self) -> &'static str {
        match self {
            Self::Button => "button",
            Self::Submit => "submit",
            Self::Reset => "reset",
        }
    }
}

#[component]
pub fn Button(
    /// Plain-text content retained for compatibility and simple actions.
    #[prop(optional, into)]
    label: String,
    /// When present, renders an anchor. Accepts owned route strings as well as
    /// string literals, so callers are not limited to static destinations.
    #[prop(optional, into)]
    href: Option<String>,
    #[prop(optional)] variant: ButtonVariant,
    #[prop(optional)] size: ButtonSize,
    #[prop(optional)] button_type: ButtonType,
    /// Reactive disabled state. Disabled links omit `href` and expose
    /// `aria-disabled` in addition to leaving the tab order.
    #[prop(default = Signal::derive(|| false), into)]
    disabled: Signal<bool>,
    #[prop(optional)] on_click: Option<Callback<()>>,
    #[prop(optional, into)] id: Option<String>,
    #[prop(optional, into)] title: Option<String>,
    #[prop(optional, into)] aria_label: Option<String>,
    #[prop(optional, into)] aria_controls: Option<String>,
    #[prop(optional, into)] aria_haspopup: Option<String>,
    /// Reactive disclosure state for controls that reveal another surface.
    #[prop(optional)]
    aria_expanded: Option<Signal<bool>>,
    /// Reactive toggle state for controls that behave as toggle buttons.
    #[prop(optional)]
    aria_pressed: Option<Signal<bool>>,
    /// Optional rich content (for example an icon and text). When supplied it
    /// replaces `label`.
    #[prop(optional)]
    children: Option<Children>,
    /// A feature-specific class may be appended without replacing canonical
    /// button classes.
    #[prop(optional, into)]
    class: String,
) -> impl IntoView {
    let mut classes = vec!["button"];
    classes.extend_from_slice(variant.classes());
    if let Some(size) = size.class() {
        classes.push(size);
    }
    let classes = if class.trim().is_empty() {
        classes.join(" ")
    } else {
        format!("{} {}", classes.join(" "), class.trim())
    };
    let content = match children {
        Some(children) => children().into_any(),
        None => label.into_any(),
    };

    match href {
        Some(href) => view! {
            <a
                id=id
                class=classes
                title=title
                href=move || (!disabled.get()).then(|| href.clone())
                aria-label=aria_label
                aria-controls=aria_controls
                aria-haspopup=aria_haspopup
                aria-expanded=move || aria_expanded.map(|expanded| expanded.get().to_string())
                aria-pressed=move || aria_pressed.map(|pressed| pressed.get().to_string())
                aria-disabled=move || disabled.get().then_some("true")
                tabindex=move || disabled.get().then_some("-1")
                on:click=move |event| {
                    if disabled.get_untracked() {
                        event.prevent_default();
                    } else if let Some(on_click) = on_click {
                        on_click.run(());
                    }
                }
            >
                {content}
            </a>
        }
        .into_any(),
        None => view! {
            <button
                id=id
                class=classes
                type=button_type.attribute()
                title=title
                aria-label=aria_label
                aria-controls=aria_controls
                aria-haspopup=aria_haspopup
                aria-expanded=move || aria_expanded.map(|expanded| expanded.get().to_string())
                aria-pressed=move || aria_pressed.map(|pressed| pressed.get().to_string())
                disabled=move || disabled.get()
                on:click=move |_| {
                    if !disabled.get_untracked()
                        && let Some(on_click) = on_click
                    {
                        on_click.run(());
                    }
                }
            >
                {content}
            </button>
        }
        .into_any(),
    }
}

#[cfg(all(test, feature = "ssr"))]
mod tests {
    use super::*;

    #[test]
    fn button_variants_and_sizes_use_canonical_classes() {
        let html = Owner::new().with(|| {
            view! {
                <Button
                    label="Remove"
                    variant=ButtonVariant::Danger
                    size=ButtonSize::Compact
                />
            }
            .to_html()
        });

        assert!(html.contains("class=\"button button--danger button--compact\""));
        assert!(html.contains("type=\"button\""));
        assert!(html.contains(">Remove</button>"));
    }

    #[test]
    fn disabled_dynamic_link_is_not_navigable() {
        let html = Owner::new().with(|| {
            view! {
                <Button
                    label="Open"
                    href="/dashboards/example".to_string()
                    disabled=Signal::derive(|| true)
                />
            }
            .to_html()
        });

        assert!(!html.contains("href="));
        assert!(html.contains("aria-disabled=\"true\""));
        assert!(html.contains("tabindex=\"-1\""));
    }

    #[test]
    fn rich_content_replaces_plain_label() {
        let html = Owner::new().with(|| {
            view! {
                <Button label="Fallback">
                    <span class="fixture-icon">"Icon"</span>
                    <span>"Continue"</span>
                </Button>
            }
            .to_html()
        });

        assert!(!html.contains("Fallback"));
        assert!(html.contains("fixture-icon"));
        assert!(html.contains("Continue"));
    }

    #[test]
    fn disclosure_attributes_support_accessible_triggers() {
        let html = Owner::new().with(|| {
            view! {
                <Button
                    label="Components"
                    id="component-palette-trigger"
                    title="Open component palette"
                    aria_label="Open component palette"
                    aria_controls="component-palette"
                    aria_haspopup="dialog"
                    aria_expanded=Signal::derive(|| true)
                />
            }
            .to_html()
        });

        assert!(html.contains("id=\"component-palette-trigger\""));
        assert!(html.contains("aria-label=\"Open component palette\""));
        assert!(html.contains("aria-controls=\"component-palette\""));
        assert!(html.contains("aria-haspopup=\"dialog\""));
        assert!(html.contains("aria-expanded=\"true\""));
    }
}
