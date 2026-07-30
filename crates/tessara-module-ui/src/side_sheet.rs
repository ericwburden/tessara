//! Controlled, domain-neutral side sheet.
//!
//! The sheet owns common dialog semantics and dismissal behavior while its
//! caller remains the single source of truth for whether it is open.

use icons::X;
use leptos::{portal::Portal, prelude::*};

use crate::modal_dialog::{handle_dialog_keydown, manage_dialog};

/// Edge from which a [`SideSheet`] is presented.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum SideSheetSide {
    Start,
    #[default]
    End,
}

impl SideSheetSide {
    fn modifier(self) -> &'static str {
        match self {
            Self::Start => "sheet-panel--start",
            Self::End => "sheet-panel--end",
        }
    }
}

#[component]
pub fn SideSheet(
    /// Stable DOM identifier used to associate the dialog and its title.
    #[prop(into)]
    id: String,
    #[prop(into)] title: Signal<String>,
    open: Signal<bool>,
    on_close: Callback<()>,
    children: ChildrenFn,
    #[prop(optional, into)] description: Option<String>,
    #[prop(optional, into)] eyebrow: Option<String>,
    #[prop(optional)] side: SideSheetSide,
    #[prop(default = "Close panel")] close_label: &'static str,
    /// Invoked after a dismissal request for caller-owned cleanup. Focus is
    /// restored automatically by the shared dialog behavior.
    #[prop(optional)]
    on_after_close: Option<Callback<()>>,
    /// Feature-owned actions rendered before the standard close control.
    #[prop(optional, into)]
    header_actions: ViewFn,
    #[prop(optional, into)] class: String,
) -> impl IntoView {
    view! {
        <Portal>
            <SideSheetSurface
                id=id.clone()
                title
                open
                on_close
                description=description.clone()
                eyebrow=eyebrow.clone()
                side
                close_label
                on_after_close
                header_actions=header_actions.clone()
                class=class.clone()
                children=children.clone()
            />
        </Portal>
    }
}

#[component]
fn SideSheetSurface(
    id: String,
    title: Signal<String>,
    open: Signal<bool>,
    on_close: Callback<()>,
    children: ChildrenFn,
    description: Option<String>,
    eyebrow: Option<String>,
    side: SideSheetSide,
    close_label: &'static str,
    on_after_close: Option<Callback<()>>,
    #[prop(optional, into)] header_actions: ViewFn,
    class: String,
) -> impl IntoView {
    let title_id = format!("{id}-title");
    let description_id = description.as_ref().map(|_| format!("{id}-description"));
    let title_id_for_dialog = title_id.clone();
    let description_id_for_dialog = description_id.clone();
    let description_id_for_content = description_id.clone();
    let panel_class = if class.trim().is_empty() {
        format!("sheet-panel blurred-surface {}", side.modifier())
    } else {
        format!(
            "sheet-panel blurred-surface {} {}",
            side.modifier(),
            class.trim()
        )
    };

    let dismiss = move || {
        on_close.run(());
        if let Some(on_after_close) = on_after_close {
            on_after_close.run(());
        }
    };
    let dismiss_from_scrim = dismiss;
    let dismiss_from_button = dismiss;
    let dismiss_from_keyboard = dismiss;
    let dismiss_from_document = dismiss;
    let close_button = NodeRef::<leptos::html::Button>::new();
    manage_dialog(open, id.clone(), close_button, dismiss_from_document);

    view! {
        <section
            id=id
            class="sheet-overlay"
            hidden=move || !open.get()
            inert=move || !open.get()
            aria-hidden=move || (!open.get()).to_string()
        >
                <button
                    class="sheet-overlay__scrim"
                    type="button"
                    aria-label=close_label
                    tabindex="-1"
                    on:click=move |_| dismiss_from_scrim()
                ></button>
                <aside
                    class=panel_class.clone()
                    role="dialog"
                    aria-modal="true"
                    aria-labelledby=title_id_for_dialog.clone()
                    aria-describedby=description_id_for_dialog.clone()
                    tabindex="-1"
                    on:keydown=move |event| {
                        handle_dialog_keydown(event, dismiss_from_keyboard);
                    }
                >
                    <div class="sheet-panel__actions">
                        {header_actions.run()}
                        <button
                            node_ref=close_button
                            class="icon-button sheet-panel__close"
                            type="button"
                            aria-label=close_label
                            title=close_label
                            on:click=move |_| dismiss_from_button()
                        >
                            <X class="icon-button__icon"/>
                        </button>
                    </div>
                    <header class="sheet-panel__header">
                        {eyebrow.clone().map(|eyebrow| view! { <p>{eyebrow}</p> })}
                        <h2 id=title_id.clone()>{move || title.get()}</h2>
                        {description.clone().map(|description| {
                            view! { <p id=description_id_for_content.clone()>{description}</p> }
                        })}
                    </header>
                    {children()}
                </aside>
        </section>
    }
}

#[cfg(all(test, feature = "ssr"))]
mod tests {
    use super::*;

    #[test]
    fn open_sheet_renders_labeled_modal_structure() {
        let html = Owner::new().with(|| {
            view! {
                <SideSheetSurface
                    id="placement-details".to_string()
                    title=Signal::derive(|| "Placement details".to_string())
                    description=Some("Change the selected placement.".to_string())
                    eyebrow=Some("Dashboard editor".to_string())
                    open=Signal::derive(|| true)
                    on_close=Callback::new(|_| {})
                    side=SideSheetSide::End
                    close_label="Close panel"
                    on_after_close=None
                    header_actions=ViewFn::default()
                    class=String::new()
                >
                    <section class="sheet-panel__section">"Fields"</section>
                </SideSheetSurface>
            }
            .to_html()
        });

        assert!(html.contains("id=\"placement-details\""));
        assert!(html.contains("role=\"dialog\""));
        assert!(html.contains("aria-modal=\"true\""));
        assert!(html.contains("aria-labelledby=\"placement-details-title\""));
        assert!(html.contains("aria-describedby=\"placement-details-description\""));
        assert!(html.contains("sheet-panel--end"));
        assert!(html.contains("Change the selected placement."));
    }

    #[test]
    fn closed_sheet_is_removed_from_accessibility_tree() {
        let html = Owner::new().with(|| {
            view! {
                <SideSheetSurface
                    id="components".to_string()
                    title=Signal::derive(|| "Components".to_string())
                    open=Signal::derive(|| false)
                    on_close=Callback::new(|_| {})
                    description=None
                    eyebrow=None
                    side=SideSheetSide::Start
                    close_label="Close panel"
                    on_after_close=None
                    header_actions=ViewFn::default()
                    class=String::new()
                >
                    <p>"Palette"</p>
                </SideSheetSurface>
            }
            .to_html()
        });

        assert!(html.contains("hidden"));
        assert!(html.contains("aria-hidden=\"true\""));
        assert!(html.contains("sheet-panel--start"));
    }
}
