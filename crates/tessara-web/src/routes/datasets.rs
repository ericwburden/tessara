use leptos::prelude::*;
use leptos_router::components::Route;
use leptos_router::{MatchNestedRoutes, path};

use crate::features::datasets::{
    DatasetsDetailPage, DatasetsEditPage, DatasetsNewPage, DatasetsPage, DatasetsPreviewPage,
};

use crate::routes::PRIMARY_SSR_MODE;

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
