//! Route-level page composition for the Datasets feature.
//!
//! Keep Leptos page components that correspond directly to routes here; reusable widgets, API calls, and DTOs should live in sibling modules.

use leptos::prelude::*;

use crate::types::route_params::{DatasetRouteParams, require_route_params};
use crate::ui::{AppShell, EmptyState, PageHeader};
use crate::utils::text::text_matches;

use super::components::{DatasetDetailSurface, DatasetDirectoryTable, DatasetPreviewTable};
use super::editor::DatasetEditorSurface;
use super::loaders::*;
use super::permissions::can_manage_datasets;
use super::types::*;

#[component]
pub fn DatasetsPage() -> impl IntoView {
    let datasets = RwSignal::new(Vec::<DatasetSummary>::new());
    let account = RwSignal::new(None::<SessionAccount>);
    let is_loading = RwSignal::new(true);
    let load_error = RwSignal::new(None::<String>);
    let search = RwSignal::new(String::new());
    let page_index = RwSignal::new(0usize);
    let page_size = RwSignal::new(10usize);

    Effect::new(move |_| {
        load_account(account);
        load_datasets(datasets, is_loading, load_error);
    });

    let filtered = Memo::new(move |_| {
        let query = search.get();
        datasets
            .get()
            .into_iter()
            .filter(|dataset| {
                text_matches(
                    &query,
                    &[
                        dataset.name.as_str(),
                        dataset.slug.as_str(),
                        dataset.grain.as_str(),
                        dataset.composition_mode.as_str(),
                    ],
                )
            })
            .collect::<Vec<_>>()
    });
    let can_manage = move || {
        account
            .get()
            .is_some_and(|account| can_manage_datasets(&account))
    };

    view! {
        <AppShell active_route="datasets" title="Datasets">
            <section class="route-panel datasets-page">
                <PageHeader title="Datasets">
                    {move || if can_manage() {
                        view! { <a class="button" href="/datasets/new">"Create Dataset"</a> }.into_any()
                    } else {
                        view! { <span></span> }.into_any()
                    }}
                </PageHeader>

                {move || {
                    if is_loading.get() {
                        view! { <EmptyState title="Loading datasets" message="Fetching visible datasets."/> }.into_any()
                    } else if let Some(message) = load_error.get() {
                        view! { <EmptyState title="Datasets unavailable" message=Box::leak(message.into_boxed_str())/> }.into_any()
                    } else if datasets.get().is_empty() {
                        view! { <EmptyState title="No visible datasets" message="No datasets are visible for the current account."/> }.into_any()
                    } else {
                        view! {
                            <DatasetDirectoryTable
                                datasets=filtered.get()
                                search
                                page_index
                                page_size
                            />
                        }.into_any()
                    }
                }}
            </section>
        </AppShell>
    }
}

#[component]
pub fn DatasetsDetailPage() -> impl IntoView {
    let params = require_route_params::<DatasetRouteParams>();
    let dataset_id = params.dataset_id;
    view! { <DatasetDetailSurface dataset_id edit=false/> }
}

#[component]
pub fn DatasetsEditPage() -> impl IntoView {
    let params = require_route_params::<DatasetRouteParams>();
    let dataset_id = params.dataset_id;
    view! { <DatasetEditorSurface dataset_id=Some(dataset_id)/> }
}

#[component]
pub fn DatasetsNewPage() -> impl IntoView {
    view! { <DatasetEditorSurface dataset_id=None/> }
}

#[component]
pub fn DatasetsPreviewPage() -> impl IntoView {
    let params = require_route_params::<DatasetRouteParams>();
    let dataset_id = params.dataset_id;
    let dataset = RwSignal::new(None::<DatasetDefinition>);
    let table = RwSignal::new(None::<DatasetTable>);
    let is_loading = RwSignal::new(true);
    let load_error = RwSignal::new(None::<String>);
    let table_error = RwSignal::new(None::<String>);

    Effect::new({
        let dataset_id = dataset_id.clone();
        move |_| {
            load_dataset_detail(dataset_id.clone(), dataset, is_loading, load_error);
            load_dataset_table(dataset_id.clone(), table, table_error);
        }
    });

    view! {
        <main class="dataset-preview-page">
            {move || {
                if is_loading.get() {
                    view! { <EmptyState title="Loading preview" message="Fetching dataset preview rows."/> }.into_any()
                } else if let Some(message) = load_error.get() {
                    view! { <EmptyState title="Preview unavailable" message=Box::leak(message.into_boxed_str())/> }.into_any()
                } else if let Some(loaded) = dataset.get() {
                    view! {
                        <section class="dataset-preview-page__content">
                            <header class="dataset-preview-page__header">
                                <p>"Dataset Preview"</p>
                                <h1>{loaded.name.clone()}</h1>
                            </header>
                            <DatasetPreviewTable dataset=loaded table=table.get() error=table_error.get()/ >
                        </section>
                    }.into_any()
                } else {
                    view! { <EmptyState title="Preview unavailable" message="Dataset details could not be loaded."/> }.into_any()
                }
            }}
        </main>
    }
}
