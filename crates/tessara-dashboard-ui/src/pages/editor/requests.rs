use leptos::prelude::*;

use crate::types::{
    DashboardComposition, DashboardMetadataRequest, EditorPlacement,
    ReconcileDashboardCompositionRequest, SessionAccount, VisibilityNodeOption,
};

use super::operation::EditorOperation;
#[cfg(feature = "hydrate")]
use super::operation::{finish_operation, try_begin_operation};

pub(super) struct CompositionLoadContext {
    pub(super) composition: RwSignal<Option<DashboardComposition>>,
    pub(super) account: RwSignal<Option<SessionAccount>>,
    pub(super) placements: RwSignal<Vec<EditorPlacement>>,
    pub(super) loading: RwSignal<bool>,
    pub(super) error: RwSignal<Option<String>>,
}

pub(super) fn load_composition(dashboard_id: String, context: CompositionLoadContext) {
    let CompositionLoadContext {
        composition,
        account,
        placements,
        loading,
        error,
    } = context;
    #[cfg(feature = "hydrate")]
    leptos::task::spawn_local(async move {
        loading.set(true);
        error.set(None);
        match crate::api::fetch_account().await {
            Ok(payload) => account.set(Some(payload)),
            Err(message) => {
                error.set(Some(message));
                loading.set(false);
                return;
            }
        }
        match crate::api::fetch_composition(&dashboard_id).await {
            Ok(payload) => {
                placements.set(
                    payload
                        .dashboard
                        .placements
                        .iter()
                        .cloned()
                        .map(EditorPlacement::existing)
                        .collect(),
                );
                composition.set(Some(payload));
            }
            Err(message) => error.set(Some(message)),
        }
        loading.set(false);
    });
    #[cfg(not(feature = "hydrate"))]
    let _ = (
        dashboard_id,
        composition,
        account,
        placements,
        loading,
        error,
    );
}

pub(super) struct LayoutSaveContext {
    pub(super) composition: RwSignal<Option<DashboardComposition>>,
    pub(super) placements: RwSignal<Vec<EditorPlacement>>,
    pub(super) selected: RwSignal<Option<String>>,
    pub(super) dirty: RwSignal<bool>,
    pub(super) operation: RwSignal<EditorOperation>,
    pub(super) error: RwSignal<Option<String>>,
    pub(super) announcement: RwSignal<String>,
}

pub(super) fn save_layout(
    dashboard_id: String,
    payload: ReconcileDashboardCompositionRequest,
    context: LayoutSaveContext,
) {
    let LayoutSaveContext {
        composition,
        placements,
        selected,
        dirty,
        operation,
        error,
        announcement,
    } = context;
    #[cfg(feature = "hydrate")]
    {
        if !try_begin_operation(operation, EditorOperation::SavingLayout) {
            return;
        }
        leptos::task::spawn_local(async move {
            error.set(None);
            match crate::api::save_composition(&dashboard_id, &payload).await {
                Ok(saved) => {
                    let selected_id = selected.get_untracked();
                    placements.set(
                        saved
                            .dashboard
                            .placements
                            .iter()
                            .cloned()
                            .map(EditorPlacement::existing)
                            .collect(),
                    );
                    if selected_id
                        .as_deref()
                        .is_some_and(|key| key.starts_with("new-"))
                    {
                        selected.set(selected_id.as_deref().and_then(|client_key| {
                            saved
                                .new_placement_ids
                                .iter()
                                .find(|mapping| mapping.client_key == client_key)
                                .map(|mapping| mapping.placement_id.clone())
                        }));
                    }
                    composition.set(Some(saved));
                    dirty.set(false);
                    announcement
                        .set("Dashboard layout saved. Preview Dashboard is now available.".into());
                }
                Err(message) => {
                    error.set(Some(message));
                    announcement
                        .set("Dashboard layout was not saved; local changes remain.".into());
                }
            }
            finish_operation(operation, EditorOperation::SavingLayout);
        });
    }
    #[cfg(not(feature = "hydrate"))]
    let _ = (
        dashboard_id,
        payload,
        composition,
        placements,
        selected,
        dirty,
        operation,
        error,
        announcement,
    );
}

pub(super) fn load_settings_nodes(
    nodes: RwSignal<Vec<VisibilityNodeOption>>,
    error: RwSignal<Option<String>>,
) {
    #[cfg(feature = "hydrate")]
    leptos::task::spawn_local(async move {
        match crate::api::fetch_visibility_nodes().await {
            Ok(payload) => nodes.set(payload),
            Err(message) => error.set(Some(message)),
        }
    });
    #[cfg(not(feature = "hydrate"))]
    let _ = (nodes, error);
}

pub(super) struct SettingsSaveContext {
    pub(super) composition: RwSignal<Option<DashboardComposition>>,
    pub(super) placements: RwSignal<Vec<EditorPlacement>>,
    pub(super) operation: RwSignal<EditorOperation>,
    pub(super) error: RwSignal<Option<String>>,
    pub(super) announcement: RwSignal<String>,
    pub(super) open: RwSignal<bool>,
    pub(super) dirty: RwSignal<bool>,
}

pub(super) fn save_settings(
    dashboard_id: String,
    payload: DashboardMetadataRequest,
    context: SettingsSaveContext,
) {
    let SettingsSaveContext {
        composition,
        placements,
        operation,
        error,
        announcement,
        open,
        dirty,
    } = context;
    #[cfg(feature = "hydrate")]
    {
        if !try_begin_operation(operation, EditorOperation::SavingSettings) {
            return;
        }
        leptos::task::spawn_local(async move {
            error.set(None);
            match crate::api::update_dashboard(&dashboard_id, &payload).await {
                Ok(_) => {
                    dirty.set(false);
                    match crate::api::fetch_composition(&dashboard_id).await {
                    Ok(refreshed) => {
                        placements.set(
                            refreshed
                                .dashboard
                                .placements
                                .iter()
                                .cloned()
                                .map(EditorPlacement::existing)
                                .collect(),
                        );
                        composition.set(Some(refreshed));
                        announcement.set("Dashboard settings saved.".into());
                        open.set(false);
                    }
                    Err(message) => error.set(Some(format!(
                        "Dashboard settings saved, but refreshed composition could not be loaded: {message}"
                    ))),
                    }
                }
                Err(message) => error.set(Some(message)),
            }
            finish_operation(operation, EditorOperation::SavingSettings);
        });
    }
    #[cfg(not(feature = "hydrate"))]
    let _ = (
        dashboard_id,
        payload,
        composition,
        placements,
        operation,
        error,
        announcement,
        open,
        dirty,
    );
}
