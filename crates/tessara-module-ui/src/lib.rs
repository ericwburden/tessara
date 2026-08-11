//! Policy-neutral shell presentation and UI primitives for Tessara modules.

#[cfg(feature = "components")]
mod application_shell;
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
#[cfg(all(feature = "hydrate", target_arch = "wasm32"))]
mod lifecycle;
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
use leptos::prelude::{AnyView, Fragment, IntoView, Owner};
use tessara_module_contract::{
    NavigationProjectionV1, OriginalActorProjectionV1, ShellContextV1, ShellDocumentStateV1,
    ShellThemeV1,
};
use uuid::Uuid;

#[cfg(feature = "components")]
pub use application_shell::ApplicationShell;
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
#[cfg(all(feature = "hydrate", target_arch = "wasm32"))]
pub use lifecycle::{LeptosLifecycleAdapter, LeptosLifecycleRoot};
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
pub const MODULE_UI_CSS: &str = include_str!("../assets/module-ui.css");
pub const MODULE_UI_CSS_SHA256: &str =
    "76e0cb7b9ffa09ed5029daa87578e11043d7df966ce248282d9f619d17375abe";
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

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ModuleDocumentAssets {
    pub stylesheets: Vec<String>,
    pub deferred_scripts: Vec<String>,
    pub hydration_script: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ModuleReleaseMetadata {
    pub definition_id: String,
    pub release_version: String,
    pub asset_digest: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ModuleBootstrapData {
    pub script_id: String,
    pub json: String,
}

/// Renders a complete module document from a typed product view. Product
/// crates supply view state and namespaced assets; only the SDK assembles the
/// outer document, canonical shell, release metadata, and bootstrap payload.
#[cfg(feature = "components")]
pub fn render_module_view_document<F, V>(
    presentation: &ShellPresentation,
    assets: &ModuleDocumentAssets,
    release: &ModuleReleaseMetadata,
    bootstrap: Option<&ModuleBootstrapData>,
    product_view: F,
) -> String
where
    F: FnOnce() -> V,
    V: IntoView + 'static,
{
    let content = Owner::new().with(|| product_view().to_html());
    let mut document = render_document_markup(presentation, assets, release, &content);
    let bootstrap_markup = bootstrap
        .map(|bootstrap| {
            format!(
                r#"<script id="{}" type="application/json">{}</script>"#,
                escape_attribute(&bootstrap.script_id),
                bootstrap.json,
            )
        })
        .unwrap_or_default();
    let metadata = format!(
        r#"<meta name="tessara-module-definition" content="{}"><meta name="tessara-module-release" content="{}"><meta name="tessara-module-asset-digest" content="{}">{}"#,
        escape_attribute(&release.definition_id),
        escape_attribute(&release.release_version),
        escape_attribute(&release.asset_digest),
        bootstrap_markup,
    );
    document = document.replacen("</head>", &format!("{metadata}</head>"), 1);
    document
}

#[cfg(feature = "components")]
fn render_document_markup(
    presentation: &ShellPresentation,
    assets: &ModuleDocumentAssets,
    release: &ModuleReleaseMetadata,
    body_html: &str,
) -> String {
    let theme = theme_name(presentation.theme);
    let theme_bootstrap = theme_bootstrap_script(theme);
    let top_bar_title = presentation
        .navigation
        .iter()
        .filter(|item| navigation_path_matches(&presentation.current_destination, &item.href))
        .max_by_key(|item| item.href.len())
        .map_or(presentation.document_title.as_str(), |item| {
            item.label.as_str()
        });
    let navigation = presentation
        .navigation
        .iter()
        .map(|item| {
            let active = if navigation_path_matches(&presentation.current_destination, &item.href) {
                " sidebar-link is-active"
            } else {
                ""
            };
            format!(
                r#"<a class="sidebar-link{}" href="{}" title="{}"><span class="sidebar-link__icon-wrap" aria-hidden="true">{}</span><span class="sidebar-link__label">{}</span></a>"#,
                active,
                escape_attribute(&item.href),
                escape_attribute(&item.label),
                navigation_icon(),
                escape_text(&item.label)
            )
        })
        .collect::<String>();
    let stylesheets = assets
        .stylesheets
        .iter()
        .map(|href| {
            format!(
                r#"<link rel="stylesheet" href="{}">"#,
                escape_attribute(href)
            )
        })
        .collect::<String>();
    let deferred_scripts = assets
        .deferred_scripts
        .iter()
        .map(|src| format!(r#"<script src="{}" defer></script>"#, escape_attribute(src)))
        .collect::<String>();
    let hydration = assets
        .hydration_script
        .as_deref()
        .map(|href| {
            format!(
                r#"<script type="module" src="{}"></script>"#,
                escape_attribute(href)
            )
        })
        .unwrap_or_default();
    format!(
        r##"<!doctype html><html lang="{}" data-theme="{}" data-theme-preference="{}"><head><meta charset="utf-8"><meta name="viewport" content="width=device-width,initial-scale=1"><meta name="theme-color" content="#0F172A"><title>{} · Tessara</title><script>{}</script>{}{}</head><body class="tessara-app {}" data-shell-state="{}" data-correlation-id="{}"><main class="app-shell"><aside class="sidebar" aria-label="Primary navigation">{}<nav class="sidebar-nav" aria-label="Primary"><div class="sidebar-navigation-projection"><p class="sidebar-section">Main</p>{}</div></nav>{}</aside><section class="app-main" aria-label="Application content"><header class="top-app-bar"><div class="top-app-bar__title-row"><button class="icon-button mobile-nav__toggle" type="button" aria-label="Open navigation" aria-expanded="false">{}</button><span class="top-app-bar__title">{}</span></div><div class="top-app-bar__actions"><label class="search-field"><span class="sr-only">Search Tessara</span><input type="search" placeholder="Search Tessara"></label><div class="theme-toggle"><button class="icon-button theme-toggle__trigger" type="button" aria-label="Theme options" aria-haspopup="menu" aria-expanded="false">{}</button><button class="theme-toggle__scrim" type="button" aria-label="Close theme options"></button><div class="theme-toggle__menu blurred-surface" role="menu" aria-label="Theme options"><button class="theme-toggle__option" type="button" role="menuitemradio" data-theme-value="system">System</button><button class="theme-toggle__option" type="button" role="menuitemradio" data-theme-value="light">Light</button><button class="theme-toggle__option" type="button" role="menuitemradio" data-theme-value="dark">Dark</button></div></div><button class="icon-button" type="button" aria-label="Notifications" title="Notifications">{}</button><button class="icon-button" type="button" aria-label="Help" title="Help">{}</button></div></header><div class="app-page"><div id="module-content">{}</div></div></section><button class="mobile-nav__scrim" type="button" aria-label="Close navigation"></button><aside class="mobile-nav__panel blurred-surface" aria-label="Primary navigation">{}<nav class="sidebar-nav" aria-label="Primary"><div class="sidebar-navigation-projection"><p class="sidebar-section">Main</p>{}</div></nav>{}</aside></main>{}<script>{}</script></body></html>"##,
        escape_attribute(&presentation.locale),
        theme,
        theme,
        escape_text(&presentation.document_title),
        theme_bootstrap,
        stylesheets,
        deferred_scripts,
        escape_attribute(&module_scope_class(&release.definition_id)),
        document_state_name(presentation.document_state),
        presentation.correlation_id,
        brand_markup(&presentation.return_destination),
        navigation,
        account_markup(&presentation.actor.display_name),
        menu_icon(),
        escape_text(top_bar_title),
        theme_icon(),
        bell_icon(),
        help_icon(),
        body_html,
        brand_markup(&presentation.return_destination),
        presentation.navigation.iter().map(|item| {
            let active = if navigation_path_matches(&presentation.current_destination, &item.href) { " is-active" } else { "" };
            format!(r#"<a class="sidebar-link{}" href="{}"><span class="sidebar-link__icon-wrap" aria-hidden="true">{}</span><span class="sidebar-link__label">{}</span></a>"#, active, escape_attribute(&item.href), navigation_icon(), escape_text(&item.label))
        }).collect::<String>(),
        account_markup(&presentation.actor.display_name),
        hydration,
        shell_interaction_script(),
    )
}

/// Returns the stable CSS ownership scope shared by direct documents and the
/// Core lifecycle host for one module definition.
pub fn module_scope_class(definition_id: &str) -> String {
    let mut slug = String::with_capacity(definition_id.len());
    let mut previous_separator = false;
    for character in definition_id.chars() {
        if character.is_ascii_alphanumeric() {
            slug.push(character.to_ascii_lowercase());
            previous_separator = false;
        } else if !previous_separator && !slug.is_empty() {
            slug.push('-');
            previous_separator = true;
        }
    }
    while slug.ends_with('-') {
        slug.pop();
    }
    format!("module-scope--{slug}")
}

/// Matches a current document route to its manifest navigation destination.
/// Segment boundaries prevent `/components-old` from activating Components.
pub fn navigation_path_matches(current_path: &str, navigation_href: &str) -> bool {
    let current = current_path
        .split(['?', '#'])
        .next()
        .unwrap_or(current_path)
        .trim_end_matches('/');
    let href = navigation_href
        .split(['?', '#'])
        .next()
        .unwrap_or(navigation_href)
        .trim_end_matches('/');
    current == href
        || (!href.is_empty()
            && href != "/"
            && current
                .strip_prefix(href)
                .is_some_and(|suffix| suffix.starts_with('/')))
}

#[cfg(feature = "components")]
fn brand_markup(href: &str) -> String {
    format!(
        r#"<a class="brand-lockup" href="{}"><span class="brand-mark" aria-hidden="true"><img src="/assets/tessara-icon-256.svg" alt=""></span><span class="brand-copy"><strong>Tessara</strong></span></a>"#,
        escape_attribute(href)
    )
}

#[cfg(feature = "components")]
fn account_markup(display_name: &str) -> String {
    let initials = display_name
        .split_whitespace()
        .filter_map(|part| part.chars().next())
        .take(2)
        .collect::<String>()
        .to_uppercase();
    format!(
        r#"<section class="account-card" aria-label="Account context"><span class="account-avatar">{}</span><span class="account-copy"><strong>{}</strong><small>Active session</small></span></section>"#,
        escape_text(&initials),
        escape_text(display_name)
    )
}

#[cfg(feature = "components")]
fn navigation_icon() -> &'static str {
    r#"<svg class="sidebar-link__icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><rect x="3" y="3" width="7" height="7" rx="1"/><rect x="14" y="3" width="7" height="7" rx="1"/><rect x="3" y="14" width="7" height="7" rx="1"/><rect x="14" y="14" width="7" height="7" rx="1"/></svg>"#
}
#[cfg(feature = "components")]
fn menu_icon() -> &'static str {
    r#"<svg class="icon-button__icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M4 6h16M4 12h16M4 18h16"/></svg>"#
}
#[cfg(feature = "components")]
fn theme_icon() -> &'static str {
    r#"<svg class="icon-button__icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M12 3a9 9 0 1 0 9 9 7 7 0 0 1-9-9Z"/></svg>"#
}
#[cfg(feature = "components")]
fn bell_icon() -> &'static str {
    r#"<svg class="icon-button__icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M18 8a6 6 0 0 0-12 0c0 7-3 7-3 9h18c0-2-3-2-3-9M10 21h4"/></svg>"#
}
#[cfg(feature = "components")]
fn help_icon() -> &'static str {
    r#"<svg class="icon-button__icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="12" cy="12" r="9"/><path d="M9.5 9a2.5 2.5 0 1 1 3.5 2.3c-.7.3-1 1-1 1.7M12 17h.01"/></svg>"#
}

#[cfg(feature = "components")]
fn shell_interaction_script() -> &'static str {
    r#"(function(){const root=document.documentElement;const shell=document.querySelector('.app-shell');const theme=document.querySelector('.theme-toggle');const themeButton=document.querySelector('.theme-toggle__trigger');const closeTheme=()=>{theme?.classList.remove('is-open');themeButton?.setAttribute('aria-expanded','false')};themeButton?.addEventListener('click',()=>{const open=!theme?.classList.contains('is-open');theme?.classList.toggle('is-open',open);themeButton.setAttribute('aria-expanded',String(open))});document.querySelector('.theme-toggle__scrim')?.addEventListener('click',closeTheme);document.querySelectorAll('[data-theme-value]').forEach(button=>button.addEventListener('click',()=>{const preference=button.dataset.themeValue;try{localStorage.setItem('tessara.themePreference',preference)}catch(_error){}const dark=window.matchMedia&&window.matchMedia('(prefers-color-scheme: dark)').matches;root.dataset.themePreference=preference;root.dataset.theme=preference==='system'?(dark?'dark':'light'):preference;closeTheme()}));const menuButton=document.querySelector('.mobile-nav__toggle');const closeMenu=()=>{shell?.classList.remove('mobile-nav-open');menuButton?.setAttribute('aria-expanded','false')};menuButton?.addEventListener('click',()=>{shell?.classList.add('mobile-nav-open');menuButton.setAttribute('aria-expanded','true')});document.querySelector('.mobile-nav__scrim')?.addEventListener('click',closeMenu)})();"#
}

#[cfg(feature = "components")]
fn theme_bootstrap_script(fallback: &str) -> String {
    format!(
        r#"(function(){{const root=document.documentElement;const fallback="{fallback}";let preference=fallback;try{{const stored=window.localStorage.getItem("tessara.themePreference");if(stored==="light"||stored==="dark"||stored==="system"){{preference=stored;}}}}catch(_error){{preference=fallback;}}const systemDark=window.matchMedia&&window.matchMedia("(prefers-color-scheme: dark)").matches;root.dataset.themePreference=preference;root.dataset.theme=preference==="system"?(systemDark?"dark":"light"):preference;}})();"#
    )
}

#[cfg(feature = "components")]
fn theme_name(theme: ShellThemeV1) -> &'static str {
    match theme {
        ShellThemeV1::System => "system",
        ShellThemeV1::Light => "light",
        ShellThemeV1::Dark => "dark",
    }
}

#[cfg(feature = "components")]
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
    #[cfg(feature = "components")]
    use chrono::{Duration, Utc};
    use sha2::{Digest, Sha256};
    #[cfg(feature = "components")]
    use tessara_module_contract::{
        ModuleDefinitionId, NavigationContributionId, NavigationProjectionV1,
        OriginalActorProjectionV1, ShellDocumentStateV1, ShellThemeV1,
    };

    use super::*;

    #[cfg(feature = "components")]
    #[test]
    fn complete_document_is_escaped_and_no_javascript_useful() {
        use leptos::prelude::*;
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
            navigation: vec![NavigationProjectionV1 {
                contribution_id: NavigationContributionId::new(
                    "tessara.reference.module-sdk.navigation",
                )
                .unwrap(),
                label: "SDK Reference".into(),
                href: "/reference/module-sdk".into(),
            }],
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
            "/reference/module-sdk/diagnostics",
            "Module SDK diagnostics",
        );
        let html = render_module_view_document(
            &presentation,
            &ModuleDocumentAssets {
                stylesheets: vec![
                    "/_tessara/modules/example/1.0.0/sha256:abc/module-ui.css".into(),
                ],
                deferred_scripts: vec!["/assets/shared.js".into()],
                hydration_script: None,
            },
            &ModuleReleaseMetadata {
                definition_id: "tessara.reference.module-sdk".into(),
                release_version: "1.0.1".into(),
                asset_digest: "sha256:abc".into(),
            },
            None,
            || view! { <p>"Recovery"</p> },
        );
        assert!(html.starts_with("<!doctype html>"));
        assert!(html.contains("data-shell-state=\"recovery\""));
        assert!(html.contains("data-theme-preference=\"dark\""));
        assert!(html.contains("tessara.themePreference"));
        assert!(html.contains("&lt;Operator&gt;"));
        assert!(html.contains("<div id=\"module-content\"><p>Recovery</p></div>"));
        assert!(html.contains("module-ui.css"));
        assert!(html.contains("<script src=\"/assets/shared.js\" defer></script>"));
        assert!(html.contains("module-scope--tessara-reference-module-sdk"));
        assert!(html.contains(r#"class="top-app-bar__title">SDK Reference</span>"#));
        assert_eq!(html.matches("sidebar-link is-active").count(), 2);
        assert!(html.contains(r#"name="tessara-module-release" content="1.0.1""#));
        assert!(!html.contains("type=\"module\""));
    }

    #[test]
    fn published_stylesheet_digest_matches_canonical_bytes() {
        assert!(MODULE_UI_CSS.contains("@media (max-width: 780px)"));
        assert!(MODULE_UI_CSS.contains(".app-shell"));
        assert!(MODULE_UI_CSS.contains("background: var(--color-bg);"));
        assert!(MODULE_UI_CSS.contains(".data-table"));
        assert_eq!(
            format!("{:x}", Sha256::digest(MODULE_UI_CSS.as_bytes())),
            MODULE_UI_CSS_SHA256,
        );
        assert_eq!(
            format!("{:x}", Sha256::digest(MODULE_SHELL_JS.as_bytes())),
            MODULE_SHELL_JS_SHA256,
        );
    }

    #[test]
    fn navigation_matching_uses_route_boundaries_and_nested_routes() {
        assert!(navigation_path_matches("/dashboards", "/dashboards"));
        assert!(navigation_path_matches(
            "/dashboards/42/edit?tab=layout",
            "/dashboards"
        ));
        assert!(!navigation_path_matches("/dashboards-old", "/dashboards"));
        assert!(!navigation_path_matches("/", "/dashboards"));
    }

    #[test]
    fn module_scope_is_stable_and_selector_safe() {
        assert_eq!(
            module_scope_class("Tessara.Reference/Module SDK"),
            "module-scope--tessara-reference-module-sdk"
        );
    }
}
