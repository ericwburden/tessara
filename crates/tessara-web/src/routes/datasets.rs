//! Route definitions for the Datasets feature.
//!
//! Keep URL nesting, route parameters, and route-to-page wiring here; page composition and data loading belong in features::datasets.

use leptos::prelude::*;
use leptos_router::components::Route;
use leptos_router::{MatchNestedRoutes, path};

use crate::features::auth::require_authenticated_route;
use crate::routes::PRIMARY_SSR_MODE;
use crate::types::route_params::{DatasetRouteParams, require_route_params};
use crate::ui::AppShell;
use tessara_web_datasets::{
    DatasetDetailContent, DatasetEditorContent, DatasetPreviewContent, DatasetsIndexContent,
};

pub fn dataset_routes() -> impl MatchNestedRoutes + Clone {
    view! {
        <>
            <Route path=path!("/datasets") view=DatasetsPage ssr=PRIMARY_SSR_MODE/>
            <Route path=path!("/datasets/new") view=DatasetsNewPage ssr=PRIMARY_SSR_MODE/>
            <Route
                path=path!("/datasets/:dataset_id")
                view=DatasetsDetailPage
                ssr=PRIMARY_SSR_MODE
            />
            <Route
                path=path!("/datasets/:dataset_id/preview")
                view=DatasetsPreviewPage
                ssr=PRIMARY_SSR_MODE
            />
            <Route
                path=path!("/datasets/:dataset_id/edit")
                view=DatasetsEditPage
                ssr=PRIMARY_SSR_MODE
            />
        </>
    }
}

#[component]
fn DatasetsPage() -> impl IntoView {
    view! {
        <AppShell active_route="datasets" title="Datasets">
            <DatasetsIndexContent/>
        </AppShell>
    }
}

#[component]
fn DatasetsNewPage() -> impl IntoView {
    view! {
        <AppShell active_route="datasets" title="Create Dataset">
            <DatasetEditorContent dataset_id=None/>
        </AppShell>
    }
}

#[component]
fn DatasetsDetailPage() -> impl IntoView {
    let params = require_route_params::<DatasetRouteParams>();
    let dataset_id = params.dataset_id;

    view! {
        <AppShell active_route="datasets" title="Dataset Detail">
            <DatasetDetailContent dataset_id/>
        </AppShell>
    }
}

#[component]
fn DatasetsEditPage() -> impl IntoView {
    let params = require_route_params::<DatasetRouteParams>();
    let dataset_id = params.dataset_id;

    view! {
        <AppShell active_route="datasets" title="Edit Dataset">
            <DatasetEditorContent dataset_id=Some(dataset_id)/>
        </AppShell>
    }
}

#[component]
fn DatasetsPreviewPage() -> impl IntoView {
    require_authenticated_route("datasets");
    let params = require_route_params::<DatasetRouteParams>();
    let dataset_id = params.dataset_id;

    view! { <DatasetPreviewContent dataset_id/> }
}
