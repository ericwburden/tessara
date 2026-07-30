//! Policy-neutral shell presentation and UI primitives for Tessara modules.

#[cfg(feature = "components")]
mod breadcrumb;
#[cfg(feature = "components")]
mod button;
#[cfg(feature = "components")]
mod combobox;
#[cfg(feature = "components")]
mod data_table;
#[cfg(feature = "components")]
mod draggable_panel_list;
#[cfg(feature = "components")]
mod dropdown;
#[cfg(feature = "components")]
mod empty_state;
pub use tessara_module_contract::grid_layout;
#[cfg(feature = "components")]
mod info_list;
#[cfg(feature = "components")]
mod modal_dialog;
#[cfg(feature = "components")]
mod page_header;
#[cfg(feature = "components")]
pub mod placement_editor;
#[cfg(feature = "components")]
mod searchable_data_table;
#[cfg(feature = "components")]
mod segmented_toggle;
#[cfg(feature = "components")]
mod side_sheet;
#[cfg(feature = "components")]
mod skeleton;
#[cfg(feature = "components")]
mod table_controls;
#[cfg(feature = "components")]
mod table_filter;
#[cfg(feature = "components")]
mod table_pagination;
#[cfg(feature = "components")]
mod table_search;
#[cfg(feature = "components")]
mod tabs;
#[cfg(feature = "components")]
mod timestamp;

#[cfg(feature = "components")]
use leptos::prelude::{AnyView, Fragment};
use tessara_module_contract::{
    NavigationProjectionV1, OriginalActorProjectionV1, ShellContextV1, ShellDocumentStateV1,
    ShellThemeV1,
};
use uuid::Uuid;

#[cfg(feature = "components")]
pub use breadcrumb::{
    Breadcrumb, BreadcrumbItem, BreadcrumbLink, BreadcrumbPage, BreadcrumbSeparator,
};
#[cfg(feature = "components")]
pub use button::{Button, ButtonSize, ButtonType, ButtonVariant};
#[cfg(feature = "components")]
pub use combobox::{Combobox, ComboboxOption};
#[cfg(feature = "components")]
pub use data_table::{
    DataTable, InteractiveDataTable, InteractiveTableColumn, InteractiveTableDataType,
    InteractiveTableRow,
};
#[cfg(feature = "components")]
pub use draggable_panel_list::{
    DraggablePanelList, DraggablePanelListAnchor, DraggablePanelListDraggable,
    DraggablePanelListDropZone, DraggablePanelListItem, DraggablePanelListMove,
};
#[cfg(feature = "components")]
pub use dropdown::DropdownMenu;
#[cfg(feature = "components")]
pub use empty_state::EmptyState;
#[cfg(feature = "components")]
pub use info_list::{InfoListTable, InfoRow};
#[cfg(feature = "components")]
pub use modal_dialog::{FullscreenDialog, ModalDialog, ModalDialogSize};
#[cfg(feature = "components")]
pub use page_header::PageHeader;
#[cfg(feature = "components")]
pub use searchable_data_table::SearchableDataTable;
#[cfg(feature = "components")]
pub use segmented_toggle::{SegmentedToggle, SegmentedToggleOption};
#[cfg(feature = "components")]
pub use side_sheet::{SideSheet, SideSheetSide};
#[cfg(feature = "components")]
pub use skeleton::Skeleton;
#[cfg(feature = "components")]
pub use table_controls::{
    TableColumnOption, TableColumnSelector, TablePopoverController, TableToolbar,
    TableToolbarActions,
};
#[cfg(feature = "components")]
pub use table_filter::TableFilterHeader;
#[cfg(feature = "components")]
pub use table_pagination::{TablePaginationBar, TablePaginationFooter};
#[cfg(feature = "components")]
pub use table_search::TableSearch;
#[cfg(feature = "components")]
pub use tabs::{Tabs, TabsContent, TabsList, TabsTrigger};
#[cfg(feature = "components")]
pub use timestamp::Timestamp;

/// Returns an empty Leptos view for conditional branches that render nothing.
#[cfg(feature = "components")]
pub fn empty_view() -> AnyView {
    Fragment::new(Vec::<AnyView>::new()).into()
}

pub const MODULE_UI_VERSION: &str = env!("CARGO_PKG_VERSION");
pub const MODULE_SHELL_CSS: &str = include_str!("../assets/module-shell.css");
pub const MODULE_SHELL_CSS_SHA256: &str =
    "434af171e5fa0f16dc4864ef9bef3a3e524a6feb1828aa6c1a1468256dd9e83d";
pub const MODULE_SHELL_JS: &str = include_str!("../assets/module-shell.js");
pub const MODULE_SHELL_JS_SHA256: &str =
    "8265b868960d45fc50fa3fc8173968b94b6d36f1d9ce12e027ab6599942682ff";

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ShellPresentation {
    pub actor: OriginalActorProjectionV1,
    pub theme: ShellThemeV1,
    pub locale: String,
    pub time_zone: String,
    pub navigation: Vec<NavigationProjectionV1>,
    pub return_destination: String,
    pub correlation_id: Uuid,
    pub document_state: ShellDocumentStateV1,
    pub current_destination: String,
    pub document_title: String,
}

impl ShellPresentation {
    pub fn from_verified_context(
        context: &ShellContextV1,
        current_destination: impl Into<String>,
        document_title: impl Into<String>,
    ) -> Self {
        Self {
            actor: context.original_actor.clone(),
            theme: context.theme,
            locale: context.locale.clone(),
            time_zone: context.time_zone.clone(),
            navigation: context.navigation.clone(),
            return_destination: context.return_destination.clone(),
            correlation_id: context.correlation_id,
            document_state: context.document_state,
            current_destination: current_destination.into(),
            document_title: document_title.into(),
        }
    }
}

pub fn render_module_document(
    presentation: &ShellPresentation,
    stylesheet_href: &str,
    hydration_script_href: Option<&str>,
    body_html: &str,
) -> String {
    let navigation = presentation
        .navigation
        .iter()
        .map(|item| {
            format!(
                r#"<li><a href="{}">{}</a></li>"#,
                escape_attribute(&item.href),
                escape_text(&item.label)
            )
        })
        .collect::<String>();
    let hydration = hydration_script_href
        .map(|href| {
            format!(
                r#"<script type="module" src="{}"></script>"#,
                escape_attribute(href)
            )
        })
        .unwrap_or_default();
    format!(
        r#"<!doctype html><html lang="{}" data-theme="{}"><head><meta charset="utf-8"><meta name="viewport" content="width=device-width,initial-scale=1"><title>{} · Tessara</title><link rel="stylesheet" href="{}"></head><body data-shell-state="{}" data-correlation-id="{}"><aside><a href="{}">Tessara</a><nav aria-label="Main navigation"><ul>{}</ul></nav></aside><header><strong>{}</strong><span>{}</span></header><main id="module-content">{}</main>{}</body></html>"#,
        escape_attribute(&presentation.locale),
        theme_name(presentation.theme),
        escape_text(&presentation.document_title),
        escape_attribute(stylesheet_href),
        document_state_name(presentation.document_state),
        presentation.correlation_id,
        escape_attribute(&presentation.return_destination),
        navigation,
        escape_text(&presentation.document_title),
        escape_text(&presentation.actor.display_name),
        body_html,
        hydration,
    )
}

fn theme_name(theme: ShellThemeV1) -> &'static str {
    match theme {
        ShellThemeV1::System => "system",
        ShellThemeV1::Light => "light",
        ShellThemeV1::Dark => "dark",
    }
}

fn document_state_name(state: ShellDocumentStateV1) -> &'static str {
    match state {
        ShellDocumentStateV1::Active => "active",
        ShellDocumentStateV1::Disabled => "disabled",
        ShellDocumentStateV1::Degraded => "degraded",
        ShellDocumentStateV1::StaleContext => "stale_context",
        ShellDocumentStateV1::Recovery => "recovery",
    }
}

pub fn escape_text(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

pub fn escape_attribute(value: &str) -> String {
    escape_text(value)
}

#[cfg(test)]
mod tests {
    use chrono::{Duration, Utc};
    use sha2::{Digest, Sha256};
    use tessara_module_contract::{
        ModuleDefinitionId, OriginalActorProjectionV1, ShellDocumentStateV1, ShellThemeV1,
    };

    use super::*;

    #[test]
    fn complete_document_is_escaped_and_no_javascript_useful() {
        let now = Utc::now();
        let context = ShellContextV1 {
            schema_version: 1,
            installation_id: Uuid::from_u128(1),
            module_definition_id: ModuleDefinitionId::new("tessara.reference.module-sdk").unwrap(),
            module_instance_id: Uuid::from_u128(2),
            original_actor: OriginalActorProjectionV1 {
                actor_id: Uuid::from_u128(3),
                display_name: "<Operator>".into(),
                email: None,
            },
            theme: ShellThemeV1::Dark,
            navigation: vec![],
            return_destination: "/administration/modules".into(),
            locale: "en-US".into(),
            time_zone: "America/New_York".into(),
            correlation_id: Uuid::from_u128(4),
            document_state: ShellDocumentStateV1::Recovery,
            issued_at: now,
            expires_at: now + Duration::seconds(60),
        };
        let presentation = ShellPresentation::from_verified_context(
            &context,
            "/reference/module-sdk",
            "Reference",
        );
        let html = render_module_document(
            &presentation,
            "/_tessara/modules/example/1.0.0/sha256:abc/module-shell.css",
            None,
            "<p>Recovery</p>",
        );
        assert!(html.starts_with("<!doctype html>"));
        assert!(html.contains("data-shell-state=\"recovery\""));
        assert!(html.contains("&lt;Operator&gt;"));
        assert!(html.contains("<main id=\"module-content\"><p>Recovery</p></main>"));
        assert!(!html.contains("<script"));
    }

    #[test]
    fn published_stylesheet_digest_matches_canonical_bytes() {
        assert_eq!(
            format!("{:x}", Sha256::digest(MODULE_SHELL_CSS.as_bytes())),
            MODULE_SHELL_CSS_SHA256,
        );
        assert_eq!(
            format!("{:x}", Sha256::digest(MODULE_SHELL_JS.as_bytes())),
            MODULE_SHELL_JS_SHA256,
        );
    }
}
