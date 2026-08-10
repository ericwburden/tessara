//! Route-free visual execution and shared stat/chart presentation.

use leptos::prelude::*;
use tessara_module_ui::{EmptyState, Skeleton};

#[cfg(feature = "hydrate")]
use crate::api;
#[cfg(feature = "hydrate")]
use crate::request::{RequestActivityGuard, schedule_bounded_retry};
use crate::{
    request::{
        BoundedRetry, RequestCompletion, RequestLifecycle, RequestLifecycleDecision,
        notify_request_activity,
    },
    types::ComponentVisual,
    viewer::{
        ComponentRequestActivity, ComponentRequestActivityCallback, ComponentVersionKind,
        ComponentVersionTarget,
    },
};

/// Renders an already-loaded visual response without route or request state.
///
/// The Components editor uses the same renderer for draft previews that the
/// exact-version viewer uses for published and superseded versions.
#[component]
pub fn ComponentVisualPresentation(visual: ComponentVisual) -> impl IntoView {
    if visual.component_type == "stat_card" {
        view! { <ComponentStatCard visual/> }.into_any()
    } else {
        view! { <ComponentD3Chart visual/> }.into_any()
    }
}

#[component]
fn ComponentStatCard(visual: ComponentVisual) -> impl IntoView {
    let class_name = format!(
        "component-stat-card component-stat-card--{}",
        visual
            .stat
            .as_ref()
            .map(|stat| stat.panel_style.as_str())
            .unwrap_or("default")
    );
    view! {
        <div class=class_name>
            {if let Some(stat) = visual.stat {
                view! {
                    <p>{stat.label}</p>
                    <strong>{stat.display_value.unwrap_or_else(|| "-".into())}</strong>
                    {stat.supporting_text.map(|text| view! { <span>{text}</span> })}
                }
                .into_any()
            } else {
                view! {
                    <EmptyState
                        title="No stat value"
                        message="The Component version returned no value."
                    />
                }
                .into_any()
            }}
        </div>
    }
}

#[component]
fn ComponentD3Chart(visual: ComponentVisual) -> impl IntoView {
    let kind = visual.component_type.clone();
    let item_count = if matches!(kind.as_str(), "pie" | "donut") {
        visual.slices.len()
    } else {
        visual.points.len()
    };
    let payload = serde_json::to_string(&visual).unwrap_or_else(|_| "{}".into());
    let aria_label = format!(
        "{} chart preview",
        ComponentVersionKind::from_api_kind(&kind)
            .map(ComponentVersionKind::label)
            .unwrap_or("Component")
    );
    view! {
        <div class="component-chart component-d3-chart" data-chart=payload>
            {if item_count == 0 {
                view! {
                    <EmptyState
                        title="No visual data"
                        message="The Component version returned no grouped values."
                    />
                }
                .into_any()
            } else {
                view! {
                    <div class="component-d3-chart__surface" role="img" aria-label=aria_label>
                        <div class="component-d3-chart__loading" aria-label="Loading chart preview">
                            <Skeleton class="skeleton--text skeleton--short"/>
                            <Skeleton class="skeleton--chart"/>
                        </div>
                    </div>
                }
                .into_any()
            }}
        </div>
    }
}

#[component]
pub(crate) fn ComponentVisualViewer(
    target: ComponentVersionTarget,
    execution_active: Signal<bool>,
    on_request_activity: Option<ComponentRequestActivityCallback>,
) -> impl IntoView {
    let visual = ArcRwSignal::new(None::<ComponentVisual>);
    let loading = ArcRwSignal::new(true);
    let error = ArcRwSignal::new(None::<String>);
    let active_request_id = ArcRwSignal::new(0_u64);
    let request_lifecycle = ArcRwSignal::new(RequestLifecycle::<((), u64)>::default());
    let retry = ArcRwSignal::new(BoundedRetry::<()>::default());

    Effect::new({
        let target = target.clone();
        let visual = visual.clone();
        let loading = loading.clone();
        let error = error.clone();
        let active_request_id = active_request_id.clone();
        let request_lifecycle = request_lifecycle.clone();
        let retry = retry.clone();
        let on_request_activity = on_request_activity.clone();
        move |_| {
            // Both signals are tracked: settlement starts a queued activation,
            // while the retry token starts only after bounded backoff.
            let _request_state = request_lifecycle.get();
            let retry_state = retry.get();
            let retry_attempt = retry_state.attempt_for(&());
            let retry_epoch = retry_state.epoch();
            let decision = request_lifecycle
                .try_maybe_update(|lifecycle| {
                    let (changed, decision) =
                        lifecycle.prepare(execution_active.get(), ((), retry_epoch));
                    (changed, decision)
                })
                .unwrap_or(RequestLifecycleDecision::None);
            match decision {
                RequestLifecycleDecision::None => return,
                RequestLifecycleDecision::RequestActivation => {
                    notify_request_activity(
                        on_request_activity.as_ref(),
                        ComponentRequestActivity::Requested,
                    );
                    return;
                }
                RequestLifecycleDecision::Start => {}
            }
            let request_id = active_request_id.get_untracked().wrapping_add(1);
            active_request_id.set(request_id);
            if retry_attempt == 0 {
                loading.set(true);
                error.set(None);
            }
            load_component_visual(ComponentVisualRequest {
                target: target.clone(),
                request_id,
                active_request_id: active_request_id.clone(),
                visual: visual.clone(),
                loading: loading.clone(),
                error: error.clone(),
                request_lifecycle: request_lifecycle.clone(),
                retry: retry.clone(),
                retry_attempt,
                retry_epoch,
                on_request_activity: on_request_activity.clone(),
            });
        }
    });

    let loading_for_busy = loading.clone();

    view! {
        <div
            class="component-visual-viewer component-visual-preview"
            aria-busy=move || loading_for_busy.get().to_string()
        >
            {move || {
                if loading.get() && visual.get().is_none() && error.get().is_none() {
                    view! {
                        <EmptyState
                            title="Loading preview"
                            message="Fetching the exact Component version."
                        />
                    }
                    .into_any()
                } else if let Some(visual) = visual.get() {
                    if visual.materialization_state != "ready" {
                        let (title, message) =
                            visual_materialization_empty_state(&visual.materialization_state);
                        view! { <EmptyState title message/> }.into_any()
                    } else {
                        view! { <ComponentVisualPresentation visual/> }.into_any()
                    }
                } else if let Some(message) = error.get() {
                    view! { <EmptyState title="Preview unavailable" message/> }.into_any()
                } else {
                    view! {
                        <EmptyState
                            title="Preview unavailable"
                            message="Component visual data could not be loaded."
                        />
                    }
                    .into_any()
                }
            }}
        </div>
    }
}

fn visual_materialization_empty_state(state: &str) -> (&'static str, String) {
    match state {
        "failed" | "error" => (
            "Visual materialization failed",
            "The Component configuration is valid, but its bound Dataset table could not be materialized. Retry after the Dataset materialization is rebuilt."
                .into(),
        ),
        "pending" => (
            "Visual materializing",
            "The Component configuration is valid, but its bound Dataset table is still being prepared."
                .into(),
        ),
        other => (
            "Visual materializing",
            format!(
                "The Component configuration is valid, but its bound Dataset table is not ready yet. Materialization state: {other}"
            ),
        ),
    }
}

#[cfg(any(feature = "hydrate", test))]
fn materialization_is_retryable(state: &str) -> bool {
    !matches!(state, "ready" | "failed" | "error")
}

struct ComponentVisualRequest {
    target: ComponentVersionTarget,
    request_id: u64,
    active_request_id: ArcRwSignal<u64>,
    visual: ArcRwSignal<Option<ComponentVisual>>,
    loading: ArcRwSignal<bool>,
    error: ArcRwSignal<Option<String>>,
    request_lifecycle: ArcRwSignal<RequestLifecycle<((), u64)>>,
    retry: ArcRwSignal<BoundedRetry<()>>,
    retry_attempt: usize,
    retry_epoch: u64,
    on_request_activity: Option<ComponentRequestActivityCallback>,
}

fn load_component_visual(request: ComponentVisualRequest) {
    #[cfg(feature = "hydrate")]
    let ComponentVisualRequest {
        target,
        request_id,
        active_request_id,
        visual,
        loading,
        error,
        request_lifecycle,
        retry,
        retry_attempt,
        retry_epoch,
        on_request_activity,
    } = request;
    #[cfg(feature = "hydrate")]
    {
        let request_guard = RequestActivityGuard::new(request_lifecycle, on_request_activity);
        leptos::task::spawn_local(async move {
            let mut request_guard = request_guard;
            let expected_kind = target.kind().as_api_value();
            let expected_version_id = target.component_version_id().to_owned();
            let endpoint = target.endpoint_path();
            let result = api::fetch_component_visual_endpoint(&endpoint).await;
            if active_request_id.get_untracked() != request_id {
                return;
            }
            loading.set(false);
            let completion = match result {
                Ok(Some(response))
                    if response.component_type == expected_kind
                        && response.component_version_id == expected_version_id =>
                {
                    let retryable = materialization_is_retryable(&response.materialization_state);
                    visual.set(Some(response));
                    error.set(None);
                    if retryable {
                        schedule_bounded_retry(
                            retry,
                            (),
                            retry_attempt,
                            retry_epoch,
                            request_id,
                            active_request_id,
                        );
                        RequestCompletion::Retryable
                    } else {
                        RequestCompletion::Successful
                    }
                }
                Ok(Some(_)) => {
                    visual.set(None);
                    error.set(Some(
                    "Component visual response did not match the requested exact version and kind."
                        .into(),
                ));
                    RequestCompletion::TerminalFailure
                }
                Ok(None) => {
                    visual.set(None);
                    RequestCompletion::TerminalFailure
                }
                Err(request_error) => {
                    let retryable = request_error.is_retryable();
                    visual.set(None);
                    error.set(Some(request_error.into_message()));
                    if retryable {
                        schedule_bounded_retry(
                            retry,
                            (),
                            retry_attempt,
                            retry_epoch,
                            request_id,
                            active_request_id,
                        );
                        RequestCompletion::Retryable
                    } else {
                        RequestCompletion::TerminalFailure
                    }
                }
            };
            request_guard.complete(completion);
        });
    }
    #[cfg(not(feature = "hydrate"))]
    {
        let ComponentVisualRequest {
            target,
            request_id,
            active_request_id,
            visual,
            loading,
            error,
            request_lifecycle,
            retry,
            retry_attempt,
            retry_epoch,
            on_request_activity,
        } = request;
        let _ = (
            target,
            request_id,
            active_request_id,
            visual,
            loading,
            error,
            retry,
            retry_attempt,
            retry_epoch,
            on_request_activity,
        );
        request_lifecycle.update(|lifecycle| lifecycle.settle(RequestCompletion::Successful));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn materialization_polling_stops_for_ready_and_failed_states() {
        assert!(materialization_is_retryable("pending"));
        assert!(materialization_is_retryable("building"));
        assert!(!materialization_is_retryable("ready"));
        assert!(!materialization_is_retryable("failed"));
        assert!(!materialization_is_retryable("error"));
    }

    #[test]
    fn materialization_message_distinguishes_failed_from_pending() {
        let (pending_title, pending_message) = visual_materialization_empty_state("pending");
        assert_eq!(pending_title, "Visual materializing");
        assert!(pending_message.contains("still being prepared"));

        let (failed_title, failed_message) = visual_materialization_empty_state("failed");
        assert_eq!(failed_title, "Visual materialization failed");
        assert!(failed_message.contains("could not be materialized"));
    }
}
