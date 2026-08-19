use leptos::{context::Provider, prelude::*};
use tessara_module_contract::ShellContextV2;
use tessara_module_ui::{
    MODULE_UI_CSS_SHA256, ModuleBootstrapData, ModuleDocumentAssets, ModuleReleaseMetadata,
    ShellPresentation, render_module_view_document,
};

use crate::{DatasetRouteBootstrap, dataset_content};

pub const DATASET_BOOTSTRAP_SCRIPT_ID: &str = "tessara-dataset-bootstrap";
pub const DATASET_CSS: &str = include_str!("../assets/dataset.css");
pub const DATASET_LIFECYCLE_CSS: &str = concat!(
    include_str!("../assets/dataset.css"),
    "\n",
    include_str!("../assets/dataset-lifecycle.css")
);
pub const DATASET_JS: &str = include_str!("../assets/dataset.js");
pub const DATASET_CSS_SHA256: &str =
    "6a19ec80265c6c0c530e1e95928cf8e0eabe4a0cab8e4f52739c5caaba1bf10c";
pub const DATASET_LIFECYCLE_CSS_SHA256: &str =
    "64117ab4979e9f9138b3538a6b0c67f9d138fbc478478cfe42262d9b26dd81a0";
pub const DATASET_JS_SHA256: &str =
    "775566089cea931240cd9cb05c7b047f027538e18e56596d8faced82ca4eac0e";
pub const DATASET_BINDINGS_JS_SHA256: &str =
    "63efa6f1f1c7e50a7516a4294e6dfc719f7d1b96eec79286aacc64a46aece6e8";
pub const DATASET_WASM_SHA256: &str =
    "a0285bf275a541588e5c7b469052802df2bcc0615ad1e8d298eaa0691afb8660";

pub fn dataset_asset_path(release: &str, digest: &str, name: &str) -> String {
    format!("/_tessara/modules/tessara.datasets/{release}/sha256:{digest}/{name}")
}

pub fn render_dataset_document(
    context: &ShellContextV2,
    path: &str,
    title: &str,
    bootstrap: &DatasetRouteBootstrap,
    release: &str,
) -> String {
    #[cfg(all(feature = "hydrate", not(target_arch = "wasm32")))]
    let _ = any_spawner::Executor::init_futures_executor();

    let presentation = ShellPresentation::from_verified_context(context, path, title);
    let bootstrap_for_view = bootstrap.clone();
    render_module_view_document(
        &presentation,
        &ModuleDocumentAssets {
            stylesheets: vec![
                dataset_asset_path(release, MODULE_UI_CSS_SHA256, "module-ui.css"),
                dataset_asset_path(release, DATASET_CSS_SHA256, "dataset.css"),
            ],
            deferred_scripts: Vec::new(),
            hydration_script: Some(dataset_asset_path(release, DATASET_JS_SHA256, "dataset.js")),
        },
        &ModuleReleaseMetadata {
            definition_id: "tessara.datasets".into(),
            release_version: release.into(),
            asset_digest: format!("sha256:{DATASET_JS_SHA256}"),
        },
        Some(&ModuleBootstrapData {
            script_id: DATASET_BOOTSTRAP_SCRIPT_ID.into(),
            json: escaped_bootstrap_json(bootstrap),
        }),
        move || {
            view! {
                <Provider value=bootstrap_for_view.clone()>
                    {dataset_content(&bootstrap_for_view)}
                </Provider>
            }
        },
    )
}

fn escaped_bootstrap_json(bootstrap: &DatasetRouteBootstrap) -> String {
    serde_json::to_string(bootstrap)
        .expect("Dataset route bootstrap should serialize")
        .replace('&', "\\u0026")
        .replace('<', "\\u003c")
        .replace('>', "\\u003e")
        .replace('\u{2028}', "\\u2028")
        .replace('\u{2029}', "\\u2029")
}

#[cfg(test)]
mod tests {
    use chrono::{Duration, Utc};
    use tessara_datasets_contract::{
        DatasetFreshnessState, DatasetProductFreshnessV1, DatasetProductProvenanceSummaryV1,
        DatasetProductSummaryV1,
    };
    use tessara_module_contract::{
        ModuleDefinitionId, NavigationContributionId, OriginalActorProjectionV1,
        SHELL_CONTEXT_SCHEMA_VERSION_V2, ShellDocumentStateV1, ShellNavigationGroupProjectionV2,
        ShellNavigationItemProjectionV2, ShellThemeV1,
    };
    use uuid::Uuid;

    use super::*;

    #[test]
    fn direct_dataset_document_uses_sdk_shell_and_only_release_owned_assets() {
        let now = Utc::now();
        let context = ShellContextV2 {
            schema_version: SHELL_CONTEXT_SCHEMA_VERSION_V2,
            installation_id: Uuid::from_u128(1),
            module_definition_id: ModuleDefinitionId::new("tessara.datasets").unwrap(),
            module_instance_id: Uuid::from_u128(2),
            original_actor: OriginalActorProjectionV1 {
                actor_id: Uuid::from_u128(3),
                display_name: "Admin".into(),
                email: None,
            },
            theme: ShellThemeV1::Dark,
            navigation: vec![ShellNavigationGroupProjectionV2 {
                id: "core.main".into(),
                label: "Main".into(),
                items: vec![ShellNavigationItemProjectionV2 {
                    contribution_id: NavigationContributionId::new("tessara.datasets.navigation")
                        .unwrap(),
                    key: "datasets".into(),
                    label: "Datasets".into(),
                    href: "/datasets".into(),
                }],
            }],
            return_destination: "/".into(),
            locale: "en-US".into(),
            time_zone: "America/New_York".into(),
            correlation_id: Uuid::from_u128(4),
            document_state: ShellDocumentStateV1::Active,
            issued_at: now,
            expires_at: now + Duration::seconds(60),
        };
        let dataset = DatasetProductSummaryV1 {
            id: Uuid::from_u128(5).to_string(),
            current_revision_id: Some(Uuid::from_u128(6).to_string()),
            current_version_major: Some(1),
            current_version_minor: Some(0),
            current_version_patch: Some(0),
            major_versions: vec![1],
            name: "Delivery responses".into(),
            slug: "delivery-responses".into(),
            grain: "submission".into(),
            tags: vec!["delivery".into()],
            provenance: DatasetProductProvenanceSummaryV1::default(),
            materialized_row_count: Some(12),
            materialized_at: None,
            freshness: DatasetProductFreshnessV1 {
                state: DatasetFreshnessState::Current,
                last_checked_at: None,
                last_succeeded_at: None,
                sanitized_failure_code: None,
            },
            visibility_nodes: Vec::new(),
            source_count: 1,
            field_count: 2,
            output_fields: Vec::new(),
            revisions: Vec::new(),
        };
        let html = render_dataset_document(
            &context,
            "/datasets",
            "Datasets",
            &DatasetRouteBootstrap::Directory {
                datasets: vec![dataset],
                can_manage: true,
            },
            "1.0.0",
        );
        assert!(html.contains(r#"class="top-app-bar__title">Datasets</span>"#));
        assert!(html.contains("tessara-dataset-bootstrap"));
        assert!(html.contains("module-scope--tessara-datasets"));
        assert!(html.contains(DATASET_CSS_SHA256));
        assert!(html.contains(DATASET_JS_SHA256));
        assert!(html.contains("Delivery responses"));
        assert!(!html.contains("Loading datasets"));
        assert!(!html.contains("cdnjs.cloudflare.com"));
        assert!(!html.contains("/api/me"));
        assert!(DATASET_LIFECYCLE_CSS.contains(".dataset-detail-summary"));
        assert!(DATASET_LIFECYCLE_CSS.contains(".dataset-editor"));
        assert!(DATASET_LIFECYCLE_CSS.contains(
            ".dataset-tags-editor {\n  align-content: start;\n  border: 0;\n  margin: 0;\n  min-inline-size: 0;\n  padding: 0;"
        ));
        assert!(
            DATASET_LIFECYCLE_CSS
                .contains(".module-scope--tessara-datasets [data-dataset-preview]")
        );
    }
}
