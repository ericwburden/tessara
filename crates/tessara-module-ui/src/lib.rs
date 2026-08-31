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
mod shell_sidebar;
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
use leptos::prelude::{
    AnyView, Fragment, GlobalAttributes, InnerHtmlAttribute, IntoAny, IntoView, Owner, RenderHtml,
    view,
};
#[cfg(feature = "components")]
use std::sync::Arc;
use tessara_module_contract::{
    OriginalActorProjectionV1, ShellContextV2, ShellDocumentStateV1,
    ShellNavigationGroupProjectionV2, ShellThemeV1,
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
pub use shell_sidebar::{
    ShellAccountPresentation, ShellNavigationGroupPresentation, ShellNavigationIcon,
    ShellNavigationItemPresentation, ShellSidebar,
};
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
    "dff9a5085d85d9e535b0fc0d4ba37233891e24e0241d9ca7c4ccd3c906ea1f9f";
pub const MODULE_SHELL_JS: &str = include_str!("../assets/module-shell.js");
pub const MODULE_SHELL_JS_SHA256: &str =
    "8265b868960d45fc50fa3fc8173968b94b6d36f1d9ce12e027ab6599942682ff";

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ShellPresentation {
    pub actor: OriginalActorProjectionV1,
    pub theme: ShellThemeV1,
    pub locale: String,
    pub time_zone: String,
    pub navigation: Vec<ShellNavigationGroupProjectionV2>,
    pub return_destination: String,
    pub correlation_id: Uuid,
    pub document_state: ShellDocumentStateV1,
    pub current_destination: String,
    pub document_title: String,
}

impl ShellPresentation {
    pub fn from_verified_context(
        context: &ShellContextV2,
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
    let _ = any_spawner::Executor::init_futures_executor();
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
    let actor = ShellAccountPresentation::from(presentation.actor.clone());
    let navigation = presentation
        .navigation
        .clone()
        .into_iter()
        .map(ShellNavigationGroupPresentation::from)
        .collect::<Vec<_>>();
    let current_destination = presentation.current_destination.clone();
    let return_destination = presentation.return_destination.clone();
    let shell_title = presentation.document_title.clone();
    let body_html = body_html.to_string();
    let shell = Owner::new().with(|| {
        let navigation_view = Arc::new(move || {
            view! {
                <ShellSidebar
                    actor=actor.clone()
                    navigation=navigation.clone()
                    current_destination=current_destination.clone()
                    return_destination=return_destination.clone()
                    navigation_status=None
                />
            }
            .into_any()
        });
        view! {
            <ApplicationShell title=shell_title navigation=navigation_view>
                <div id="module-content" inner_html=body_html></div>
            </ApplicationShell>
        }
        .to_html()
    });
    format!(
        r##"<!doctype html><html lang="{}" data-theme="{}" data-theme-preference="{}"><head><meta charset="utf-8"><meta name="viewport" content="width=device-width,initial-scale=1"><meta name="theme-color" content="#0F172A"><title>{} · Tessara</title><script>{}</script>{}{}</head><body class="tessara-app {}" data-shell-state="{}" data-correlation-id="{}">{}{}<script>{}</script></body></html>"##,
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
        shell,
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
fn shell_interaction_script() -> &'static str {
    r#"(function(){const root=document.documentElement;const theme=document.querySelector('.theme-toggle');const themeButton=document.querySelector('.theme-toggle__trigger');const themeOptions=document.querySelectorAll('[data-theme-value]');const setThemeSelection=(preference)=>themeOptions.forEach(option=>{const active=option.dataset.themeValue===preference;option.classList.toggle('is-active',active);option.setAttribute('aria-checked',String(active))});const closeTheme=()=>{theme?.classList.remove('is-open');themeButton?.setAttribute('aria-expanded','false')};setThemeSelection(root.dataset.themePreference||'system');themeButton?.addEventListener('click',()=>{const open=!theme?.classList.contains('is-open');theme?.classList.toggle('is-open',open);themeButton.setAttribute('aria-expanded',String(open))});document.querySelector('.theme-toggle__scrim')?.addEventListener('click',closeTheme);themeOptions.forEach(button=>button.addEventListener('click',()=>{const preference=button.dataset.themeValue;try{localStorage.setItem('tessara.themePreference',preference)}catch(_error){}const dark=window.matchMedia&&window.matchMedia('(prefers-color-scheme: dark)').matches;const resolved=preference==='system'?(dark?'dark':'light'):preference;root.dataset.themePreference=preference;root.dataset.theme=resolved;setThemeSelection(preference);const meta=document.querySelector('meta[name="theme-color"]');meta?.setAttribute('content',resolved==='dark'?'#0F172A':'#F8FAFC');closeTheme()}));const mobileNavigation=document.querySelector('.mobile-nav');const menuButton=document.querySelector('.mobile-nav__toggle');const closeMenu=()=>{mobileNavigation?.classList.remove('is-open');menuButton?.setAttribute('aria-expanded','false')};menuButton?.addEventListener('click',()=>{mobileNavigation?.classList.add('is-open');menuButton.setAttribute('aria-expanded','true')});document.querySelector('.mobile-nav__scrim')?.addEventListener('click',closeMenu);const signOutButton=document.querySelector('[data-shell-sign-out]');signOutButton?.addEventListener('click',async()=>{signOutButton.disabled=true;try{const response=await fetch('/api/auth/logout',{method:'DELETE',credentials:'same-origin',headers:{accept:'application/json'}});if(!response.ok){throw new Error('Sign out failed')}const result=await response.json();if(result?.signed_out!==true){throw new Error('Sign out failed')}window.location.assign('/login')}catch(_error){signOutButton.disabled=false}})})();"#
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
        ModuleDefinitionId, NavigationContributionId, OriginalActorProjectionV1,
        SHELL_CONTEXT_SCHEMA_VERSION_V2, ShellDocumentStateV1, ShellNavigationGroupProjectionV2,
        ShellNavigationItemProjectionV2, ShellThemeV1,
    };

    use super::*;

    #[cfg(feature = "components")]
    #[test]
    fn complete_document_is_escaped_and_no_javascript_useful() {
        use leptos::prelude::*;
        let now = Utc::now();
        let context = ShellContextV2 {
            schema_version: SHELL_CONTEXT_SCHEMA_VERSION_V2,
            installation_id: Uuid::from_u128(1),
            module_definition_id: ModuleDefinitionId::new("tessara.reference.module-sdk").unwrap(),
            module_instance_id: Uuid::from_u128(2),
            original_actor: OriginalActorProjectionV1 {
                actor_id: Uuid::from_u128(3),
                display_name: "<Operator>".into(),
                email: Some("operator@tessara.local".into()),
            },
            theme: ShellThemeV1::Dark,
            navigation: vec![
                ShellNavigationGroupProjectionV2 {
                    id: "core.main".into(),
                    label: "Main".into(),
                    items: vec![ShellNavigationItemProjectionV2 {
                        contribution_id: NavigationContributionId::new(
                            "tessara.reference.module-sdk.navigation",
                        )
                        .unwrap(),
                        key: "reference_sdk".into(),
                        label: "SDK Reference".into(),
                        href: "/reference/module-sdk".into(),
                    }],
                },
                ShellNavigationGroupProjectionV2 {
                    id: "core.admin".into(),
                    label: "Admin".into(),
                    items: vec![ShellNavigationItemProjectionV2 {
                        contribution_id: NavigationContributionId::new("core.module-management")
                            .unwrap(),
                        key: "module_management".into(),
                        label: "Module Management".into(),
                        href: "/administration/modules".into(),
                    }],
                },
            ],
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
        assert_eq!(html.matches("data-theme-value=").count(), 3);
        assert!(html.contains("&lt;Operator&gt;"));
        assert!(html.contains("operator@tessara.local"));
        assert_eq!(html.matches(">Main</p>").count(), 2);
        assert_eq!(html.matches(">Admin</p>").count(), 2);
        assert!(html.contains("Module Management"));
        assert!(html.contains("<div id=\"module-content\"><p>Recovery</p></div>"));
        assert!(html.contains("module-ui.css"));
        assert!(html.contains("<script src=\"/assets/shared.js\" defer></script>"));
        assert!(html.contains("module-scope--tessara-reference-module-sdk"));
        assert!(html.contains(r#"class="top-app-bar__title">Module SDK diagnostics</span>"#));
        assert_eq!(html.matches("sidebar-link is-active").count(), 2);
        assert!(html.contains("mobileNavigation?.classList.add('is-open')"));
        assert!(!html.contains("shell?.classList.add('mobile-nav-open')"));
        assert!(html.contains(r#"name="tessara-module-release" content="1.0.1""#));
        assert!(!html.contains("type=\"module\""));
    }

    #[cfg(feature = "components")]
    #[test]
    fn shell_interaction_script_uses_canonical_logout_contract() {
        let script = shell_interaction_script();

        assert!(script.contains("fetch('/api/auth/logout'"));
        assert!(script.contains("method:'DELETE'"));
        assert!(script.contains("credentials:'same-origin'"));
        assert!(script.contains("result?.signed_out!==true"));
        assert!(script.contains("window.location.assign('/login')"));
        assert!(!script.contains("'/api/logout'"));
        assert!(!script.contains("form.method='post'"));
    }

    #[cfg(feature = "components")]
    #[test]
    fn shell_interaction_script_updates_theme_selection_semantics() {
        let script = shell_interaction_script();

        assert!(script.contains("setThemeSelection(root.dataset.themePreference||'system')"));
        assert!(script.contains("option.classList.toggle('is-active',active)"));
        assert!(script.contains("option.setAttribute('aria-checked',String(active))"));
        assert!(script.contains("setThemeSelection(preference)"));
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

    #[cfg(feature = "components")]
    #[test]
    fn dataset_navigation_uses_database_icon_in_complete_module_documents() {
        let dataset = Owner::new()
            .with(|| view! { <ShellNavigationIcon navigation_key="datasets"/> }.to_html());
        let generic = Owner::new()
            .with(|| view! { <ShellNavigationIcon navigation_key="reference_sdk"/> }.to_html());

        assert!(dataset.contains("<ellipse cx=\"12\" cy=\"5\" rx=\"9\" ry=\"3\""));
        assert_ne!(dataset, generic);
        assert!(generic.contains("M15 2H6a2 2 0 0 0-2 2v16"));
    }

    #[test]
    fn module_scope_is_stable_and_selector_safe() {
        assert_eq!(
            module_scope_class("Tessara.Reference/Module SDK"),
            "module-scope--tessara-reference-module-sdk"
        );
    }
}
