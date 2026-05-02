//! Shared SSR-first UI primitives for Tessara application markup.

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ActionStyle {
    Default,
    Light,
    Primary,
}

impl ActionStyle {
    fn class_name(&self) -> &'static str {
        match self {
            Self::Default => "",
            Self::Light => " is-light",
            Self::Primary => " is-primary",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ActionTarget {
    Link { href: String },
    Button { onclick: String },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ActionItem {
    label: String,
    target: ActionTarget,
    style: ActionStyle,
}

impl ActionItem {
    pub fn link(label: impl Into<String>, href: impl Into<String>, style: ActionStyle) -> Self {
        Self {
            label: label.into(),
            target: ActionTarget::Link { href: href.into() },
            style,
        }
    }

    pub fn button(
        label: impl Into<String>,
        onclick: impl Into<String>,
        style: ActionStyle,
    ) -> Self {
        Self {
            label: label.into(),
            target: ActionTarget::Button {
                onclick: onclick.into(),
            },
            style,
        }
    }

    pub fn render(&self) -> String {
        let style = self.style.class_name();

        match &self.target {
            ActionTarget::Link { href } => format!(
                r#"<a class="button-link button ui-action tessara-shell-button{style}" href="{href}">{label}</a>"#,
                label = self.label,
            ),
            ActionTarget::Button { onclick } => format!(
                r#"<button class="button ui-action tessara-shell-button{style}" type="button" onclick="{onclick}">{label}</button>"#,
                label = self.label,
            ),
        }
    }
}

pub fn action_group(actions: &[ActionItem]) -> String {
    if actions.is_empty() {
        return String::new();
    }

    let content = actions
        .iter()
        .map(ActionItem::render)
        .collect::<Vec<_>>()
        .join("");

    format!(r#"<div class="actions ui-action-group">{content}</div>"#)
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MetadataItem {
    label: String,
    value: String,
}

impl MetadataItem {
    pub fn new(label: impl Into<String>, value: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            value: value.into(),
        }
    }
}

pub fn metadata_strip(items: &[MetadataItem]) -> String {
    if items.is_empty() {
        return String::new();
    }

    let content = items
        .iter()
        .map(|item| {
            format!(
                r#"<div class="ui-metadata-strip__item"><span class="ui-metadata-strip__label">{}</span><strong class="ui-metadata-strip__value">{}</strong></div>"#,
                item.label, item.value
            )
        })
        .collect::<Vec<_>>()
        .join("");

    format!(r#"<div class="ui-metadata-strip">{content}</div>"#)
}

pub fn page_header(
    eyebrow: &str,
    title: &str,
    description: &str,
    metadata_html: Option<&str>,
    actions_html: Option<&str>,
) -> String {
    let description_html = if description.trim().is_empty() {
        String::new()
    } else {
        format!(r#"<p class="muted ui-page-header__description">{description}</p>"#)
    };
    let metadata_html = metadata_html.unwrap_or_default();
    let actions_html = actions_html.unwrap_or_default();

    format!(
        r#"
        <section class="app-screen box entity-page ui-page-header tessara-surface-panel">
          <p class="eyebrow ui-page-header__eyebrow">{eyebrow}</p>
          <div class="page-title-row ui-page-header__row">
            <div class="ui-page-header__copy">
              <h1>{title}</h1>
              {description_html}
            </div>
            {actions_html}
          </div>
          {metadata_html}
        </section>
        "#
    )
}

pub fn panel(title: &str, description: &str, body: &str) -> String {
    let description_html = if description.trim().is_empty() {
        String::new()
    } else {
        format!(r#"<p class="muted ui-panel__description">{description}</p>"#)
    };

    format!(
        r#"
        <section class="app-screen box page-panel ui-panel tessara-surface-panel">
          <h3 class="ui-panel__title">{title}</h3>
          {description_html}
          {body}
        </section>
        "#
    )
}

pub fn panel_with_header(
    title: &str,
    description: &str,
    actions_html: Option<&str>,
    body: &str,
) -> String {
    let description_html = if description.trim().is_empty() {
        String::new()
    } else {
        format!(r#"<p class="muted ui-panel__description">{description}</p>"#)
    };
    let actions_html = actions_html.unwrap_or_default();

    format!(
        r#"
        <section class="app-screen box page-panel ui-panel tessara-surface-panel">
          <div class="page-title-row compact-title-row ui-panel__header">
            <div class="ui-panel__copy">
              <h3 class="ui-panel__title">{title}</h3>
              {description_html}
            </div>
            {actions_html}
          </div>
          {body}
        </section>
        "#
    )
}

pub fn card(class_name: &str, title: &str, body_html: &str) -> String {
    format!(
        r#"<article class="{class_name} card ui-card tessara-surface-card"><div class="card-content ui-card__content"><h3>{title}</h3>{body_html}</div></article>"#
    )
}

pub fn text_input(
    id: &str,
    input_type: &str,
    autocomplete: &str,
    placeholder: &str,
    extra_attrs: &str,
) -> String {
    format!(
        r#"<input class="input ui-field__input" id="{id}" type="{input_type}" autocomplete="{autocomplete}" placeholder="{placeholder}" {extra_attrs}/>"#
    )
}

pub fn select_control(id: &str, options_html: &str, extra_attrs: &str) -> String {
    format!(
        r#"<select class="input ui-field__input" id="{id}" {extra_attrs}>{options_html}</select>"#
    )
}

pub fn textarea_control(id: &str, rows: usize, placeholder: &str, extra_attrs: &str) -> String {
    format!(
        r#"<textarea class="textarea ui-field__input" id="{id}" rows="{rows}" placeholder="{placeholder}" {extra_attrs}></textarea>"#
    )
}

pub fn field_wrapper(
    label_for: &str,
    label: &str,
    control_html: &str,
    helper_text: Option<&str>,
    extra_classes: &str,
) -> String {
    let helper_html = helper_text
        .map(|helper| format!(r#"<p class="muted ui-field__helper">{helper}</p>"#))
        .unwrap_or_default();
    let classes = if extra_classes.trim().is_empty() {
        "form-field ui-field".to_string()
    } else {
        format!("form-field ui-field {extra_classes}")
    };

    format!(
        r#"
        <div class="{classes}">
          <label class="ui-field__label" for="{label_for}">{label}</label>
          <div class="ui-field__control">{control_html}</div>
          {helper_html}
        </div>
        "#
    )
}

pub fn checkbox_field(id: &str, label: &str, checked: bool, helper_text: Option<&str>) -> String {
    let checked_attr = if checked { " checked" } else { "" };
    let helper_html = helper_text
        .map(|helper| format!(r#"<p class="muted ui-field__helper">{helper}</p>"#))
        .unwrap_or_default();

    format!(
        r#"
        <div class="form-field ui-field">
          <label class="checkbox-label ui-checkbox-field" for="{id}">
            <input id="{id}" type="checkbox"{checked_attr}>
            <span>{label}</span>
          </label>
          {helper_html}
        </div>
        "#
    )
}

pub fn toolbar(primary_html: &str, secondary_html: &str) -> String {
    let secondary = if secondary_html.trim().is_empty() {
        String::new()
    } else {
        format!(r#"<div class="ui-toolbar__secondary">{secondary_html}</div>"#)
    };

    format!(
        r#"
        <div class="page-title-row compact-title-row ui-toolbar">
          <div class="ui-toolbar__primary">{primary_html}</div>
          {secondary}
        </div>
        "#
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn action_group_renders_links_and_buttons() {
        let html = action_group(&[
            ActionItem::link("Open", "/app/forms", ActionStyle::Light),
            ActionItem::button("Refresh", "refresh()", ActionStyle::Primary),
        ]);

        assert!(html.contains("ui-action-group"));
        assert!(html.contains(r#"href="/app/forms""#));
        assert!(html.contains(r#"onclick="refresh()""#));
        assert!(html.contains("is-primary"));
    }

    #[test]
    fn metadata_strip_renders_all_items() {
        let html = metadata_strip(&[
            MetadataItem::new("Mode", "Detail"),
            MetadataItem::new("State", "Loading"),
        ]);

        assert!(html.contains("ui-metadata-strip"));
        assert!(html.contains("Mode"));
        assert!(html.contains("Loading"));
    }

    #[test]
    fn field_wrapper_renders_helper_text() {
        let html = field_wrapper(
            "form-name",
            "Name",
            &text_input("form-name", "text", "off", "", ""),
            Some("Use the top-level label shown in navigation."),
            "wide-field",
        );

        assert!(html.contains("ui-field"));
        assert!(html.contains("wide-field"));
        assert!(html.contains("ui-field__helper"));
    }

    #[test]
    fn panel_with_header_keeps_actions_outside_copy_block() {
        let html = panel_with_header(
            "Bindings",
            "Each binding defines one logical field in the report output.",
            Some(&action_group(&[ActionItem::button(
                "Add Binding",
                "addReportBindingRow()",
                ActionStyle::Default,
            )])),
            r#"<div id="report-binding-rows"></div>"#,
        );

        assert!(html.contains("ui-panel__header"));
        assert!(html.contains("Add Binding"));
        assert!(html.contains("report-binding-rows"));
    }
}
