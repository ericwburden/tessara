use leptos::{context::Provider, prelude::*};
use tessara_module_contract::ShellContextV1;
use tessara_module_ui::{
    MODULE_UI_CSS_SHA256, ModuleBootstrapData, ModuleDocumentAssets, ModuleReleaseMetadata,
    ShellPresentation, render_module_view_document,
};

use crate::{DatasetRouteBootstrap, dataset_content};

pub const DATASET_BOOTSTRAP_SCRIPT_ID: &str = "tessara-dataset-bootstrap";
pub const DATASET_CSS: &str = include_str!("../assets/dataset.css");
pub const DATASET_LIFECYCLE_CSS: &str = include_str!("../assets/dataset-lifecycle.css");
pub const DATASET_JS: &str = include_str!("../assets/dataset.js");
pub const DATASET_CSS_SHA256: &str =
    "7dd795cde57e7772cb3670988d660fdacea823888059e9d8ccaecdd70472e511";
pub const DATASET_LIFECYCLE_CSS_SHA256: &str =
    "da84cc6e3e0f359be1e7f3276e73ae11fa21963f948ad15c30df32dcdeecadfa";
pub const DATASET_JS_SHA256: &str =
    "f064e77331ab5e90d8741da3dc1c78745656ff21ccc99afcec1f0db72b48ac63";
pub const DATASET_BINDINGS_JS_SHA256: &str =
    "b5ca9979ad3b1d83904bd36f63f7deddf1737764c75f791807ccdeee953b9463";
pub const DATASET_WASM_SHA256: &str =
    "b2969c4d1f1d4c074d4f861bbcb443e1de7bc933e7fa6a691412f8c1c1fa7ce5";

pub fn dataset_asset_path(release: &str, digest: &str, name: &str) -> String {
    format!("/_tessara/modules/tessara.datasets/{release}/sha256:{digest}/{name}")
}

pub fn render_dataset_document(
    context: &ShellContextV1,
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
        ModuleDefinitionId, NavigationContributionId, NavigationProjectionV1,
        OriginalActorProjectionV1, ShellDocumentStateV1, ShellThemeV1,
    };
    use uuid::Uuid;

    use super::*;

    #[test]
    fn direct_dataset_document_uses_sdk_shell_and_only_release_owned_assets() {
        let now = Utc::now();
        let context = ShellContextV1 {
            schema_version: 1,
            installation_id: Uuid::from_u128(1),
            module_definition_id: ModuleDefinitionId::new("tessara.datasets").unwrap(),
            module_instance_id: Uuid::from_u128(2),
            original_actor: OriginalActorProjectionV1 {
                actor_id: Uuid::from_u128(3),
                display_name: "Admin".into(),
                email: None,
            },
            theme: ShellThemeV1::Dark,
            navigation: vec![NavigationProjectionV1 {
                contribution_id: NavigationContributionId::new("tessara.datasets.navigation")
                    .unwrap(),
                label: "Datasets".into(),
                href: "/datasets".into(),
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
    }
}
