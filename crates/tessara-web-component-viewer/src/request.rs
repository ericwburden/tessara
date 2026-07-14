//! Request coordination shared by Table and visual Component viewers.
//!
//! A lifecycle prevents duplicate requests for the same attempt, while a
//! bounded retry trigger supplies a new attempt key only after backoff. This
//! keeps retry delays outside the embedding scheduler's active request slots.

#[cfg(any(feature = "hydrate", test))]
use leptos::prelude::*;

use crate::viewer::{ComponentRequestActivity, ComponentRequestActivityCallback};

/// Backoff delays after the initial request. Five total attempts cap both
/// materialization polling and recovery from transient transport failures.
#[cfg(any(test, all(feature = "hydrate", target_arch = "wasm32")))]
const RETRY_DELAYS_MS: [u32; 4] = [1_000, 2_000, 4_000, 8_000];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum RequestCompletion {
    Successful,
    #[cfg(any(feature = "hydrate", test))]
    TerminalFailure,
    #[cfg(any(feature = "hydrate", test))]
    Retryable,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct RequestLifecycle<Key> {
    in_flight: bool,
    last_started: Option<Key>,
    last_completion: Option<RequestCompletion>,
    activation_requested: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum RequestLifecycleDecision {
    None,
    RequestActivation,
    Start,
}

impl<Key: PartialEq> RequestLifecycle<Key> {
    pub(crate) fn prepare(
        &mut self,
        execution_active: bool,
        key: Key,
    ) -> (bool, RequestLifecycleDecision) {
        if self.in_flight || self.last_started.as_ref() == Some(&key) {
            return (false, RequestLifecycleDecision::None);
        }
        if !execution_active {
            if self.activation_requested {
                return (false, RequestLifecycleDecision::None);
            }
            self.activation_requested = true;
            return (true, RequestLifecycleDecision::RequestActivation);
        }
        self.in_flight = true;
        self.activation_requested = false;
        self.last_started = Some(key);
        self.last_completion = None;
        (true, RequestLifecycleDecision::Start)
    }
}

impl<Key> RequestLifecycle<Key> {
    pub(crate) fn settle(&mut self, completion: RequestCompletion) {
        self.in_flight = false;
        self.last_completion = Some(completion);
    }
}

pub(crate) fn notify_request_activity(
    callback: Option<&ComponentRequestActivityCallback>,
    activity: ComponentRequestActivity,
) {
    if let Some(callback) = callback {
        callback.run(activity);
    }
}

/// Settles scheduler activity even when a fetch future is dropped because its
/// Leptos owner disappeared. Dropped/unfinished work remains retryable.
#[cfg(any(feature = "hydrate", test))]
pub(crate) struct RequestActivityGuard<Key: Send + Sync + 'static> {
    lifecycle: ArcRwSignal<RequestLifecycle<Key>>,
    callback: Option<ComponentRequestActivityCallback>,
    completion: RequestCompletion,
}

#[cfg(any(feature = "hydrate", test))]
impl<Key: Send + Sync + 'static> RequestActivityGuard<Key> {
    pub(crate) fn new(
        lifecycle: ArcRwSignal<RequestLifecycle<Key>>,
        callback: Option<ComponentRequestActivityCallback>,
    ) -> Self {
        notify_request_activity(callback.as_ref(), ComponentRequestActivity::Started);
        Self {
            lifecycle,
            callback,
            completion: RequestCompletion::Retryable,
        }
    }

    pub(crate) fn complete(&mut self, completion: RequestCompletion) {
        self.completion = completion;
    }
}

#[cfg(any(feature = "hydrate", test))]
impl<Key: Send + Sync + 'static> Drop for RequestActivityGuard<Key> {
    fn drop(&mut self) {
        notify_request_activity(self.callback.as_ref(), ComponentRequestActivity::Settled);
        let completion = self.completion;
        self.lifecycle
            .update(move |lifecycle| lifecycle.settle(completion));
    }
}

/// Reactive token that advances only after one of the bounded delays. The
/// subject prevents a delayed response for an obsolete Table query from
/// waking the current query.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct BoundedRetry<Subject> {
    epoch: u64,
    subject: Option<Subject>,
    attempt: usize,
}

impl<Subject: PartialEq> BoundedRetry<Subject> {
    pub(crate) fn attempt_for(&self, subject: &Subject) -> usize {
        if self.subject.as_ref() == Some(subject) {
            self.attempt
        } else {
            0
        }
    }

    pub(crate) const fn epoch(&self) -> u64 {
        self.epoch
    }

    #[cfg(any(test, all(feature = "hydrate", target_arch = "wasm32")))]
    pub(crate) fn next_delay(&self, subject: &Subject, completed_attempt: usize) -> Option<u32> {
        if completed_attempt != 0
            && self
                .subject
                .as_ref()
                .is_some_and(|current| current != subject)
        {
            return None;
        }
        RETRY_DELAYS_MS.get(completed_attempt).copied()
    }

    #[cfg(any(test, all(feature = "hydrate", target_arch = "wasm32")))]
    fn advance(&mut self, subject: Subject, completed_attempt: usize, expected_epoch: u64) -> bool {
        if self.epoch != expected_epoch
            || (completed_attempt != 0
                && self
                    .subject
                    .as_ref()
                    .is_some_and(|current| current != &subject))
            || self.attempt_for(&subject) != completed_attempt
            || completed_attempt >= RETRY_DELAYS_MS.len()
        {
            return false;
        }
        self.subject = Some(subject);
        self.attempt = completed_attempt + 1;
        self.epoch = self.epoch.wrapping_add(1);
        true
    }
}

/// Wakes the viewer after backoff without retaining a dashboard execution
/// slot. The request id and retry epoch together discard stale timers.
#[cfg(all(feature = "hydrate", target_arch = "wasm32"))]
pub(crate) fn schedule_bounded_retry<Subject>(
    retry: ArcRwSignal<BoundedRetry<Subject>>,
    subject: Subject,
    completed_attempt: usize,
    expected_epoch: u64,
    request_id: u64,
    active_request_id: ArcRwSignal<u64>,
) where
    Subject: Clone + PartialEq + Send + Sync + 'static,
{
    let Some(delay_ms) = retry
        .get_untracked()
        .next_delay(&subject, completed_attempt)
    else {
        return;
    };
    leptos::task::spawn_local(async move {
        gloo_timers::future::TimeoutFuture::new(delay_ms).await;
        if active_request_id.get_untracked() != request_id {
            return;
        }
        retry.update(move |state| {
            state.advance(subject, completed_attempt, expected_epoch);
        });
    });
}

#[cfg(all(feature = "hydrate", not(target_arch = "wasm32")))]
pub(crate) fn schedule_bounded_retry<Subject>(
    _: ArcRwSignal<BoundedRetry<Subject>>,
    _: Subject,
    _: usize,
    _: u64,
    _: u64,
    _: ArcRwSignal<u64>,
) where
    Subject: Clone + PartialEq + Send + Sync + 'static,
{
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lifecycle_requires_a_new_attempt_key_after_retryable_completion() {
        let mut lifecycle = RequestLifecycle::<(&str, u64)>::default();
        assert_eq!(
            lifecycle.prepare(true, ("visual", 0)).1,
            RequestLifecycleDecision::Start
        );
        lifecycle.settle(RequestCompletion::Retryable);
        assert_eq!(
            lifecycle.prepare(true, ("visual", 0)).1,
            RequestLifecycleDecision::None
        );
        assert_eq!(
            lifecycle.prepare(true, ("visual", 1)).1,
            RequestLifecycleDecision::Start
        );
    }

    #[test]
    fn lifecycle_records_all_completion_outcomes_separately() {
        let mut lifecycle = RequestLifecycle::<u64>::default();
        lifecycle.prepare(true, 0);
        lifecycle.settle(RequestCompletion::Successful);
        assert_eq!(
            lifecycle.last_completion,
            Some(RequestCompletion::Successful)
        );
        lifecycle.prepare(true, 1);
        lifecycle.settle(RequestCompletion::TerminalFailure);
        assert_eq!(
            lifecycle.last_completion,
            Some(RequestCompletion::TerminalFailure)
        );
        lifecycle.prepare(true, 2);
        lifecycle.settle(RequestCompletion::Retryable);
        assert_eq!(
            lifecycle.last_completion,
            Some(RequestCompletion::Retryable)
        );
    }

    #[test]
    fn bounded_retry_has_four_delayed_attempts_and_rejects_stale_subjects() {
        let mut retry = BoundedRetry::<String>::default();
        let subject = "page=1".to_string();

        for (attempt, expected_delay) in RETRY_DELAYS_MS.iter().copied().enumerate() {
            assert_eq!(retry.next_delay(&subject, attempt), Some(expected_delay));
            let epoch = retry.epoch();
            assert!(retry.advance(subject.clone(), attempt, epoch));
            assert_eq!(retry.attempt_for(&subject), attempt + 1);
        }
        assert_eq!(retry.next_delay(&subject, RETRY_DELAYS_MS.len()), None);
        assert!(!retry.advance("page=2".into(), RETRY_DELAYS_MS.len(), retry.epoch()));
    }

    #[test]
    fn bounded_retry_starts_a_fresh_budget_for_a_new_subject() {
        let mut retry = BoundedRetry::<String>::default();
        let first = "page=1".to_string();
        let second = "page=2".to_string();
        let epoch = retry.epoch();
        assert!(retry.advance(first, 0, epoch));

        assert_eq!(retry.attempt_for(&second), 0);
        assert_eq!(retry.next_delay(&second, 0), Some(RETRY_DELAYS_MS[0]));
        let epoch = retry.epoch();
        assert!(retry.advance(second.clone(), 0, epoch));
        assert_eq!(retry.attempt_for(&second), 1);
    }

    #[test]
    fn guard_reports_one_started_and_settled_pair() {
        let events = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
        let callback_events = events.clone();
        let callback = ComponentRequestActivityCallback::new(move |event| {
            callback_events.lock().expect("event lock").push(event);
        });
        let lifecycle = ArcRwSignal::new(RequestLifecycle::<String>::default());
        lifecycle.update(|lifecycle| {
            lifecycle.prepare(true, "request".into());
        });
        {
            let mut guard = RequestActivityGuard::new(lifecycle.clone(), Some(callback));
            guard.complete(RequestCompletion::Successful);
        }
        assert_eq!(
            *events.lock().expect("event lock"),
            vec![
                ComponentRequestActivity::Started,
                ComponentRequestActivity::Settled
            ]
        );
        assert!(!lifecycle.get_untracked().in_flight);
    }

    #[test]
    fn every_completion_outcome_reports_a_balanced_scheduler_pair() {
        let events = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
        let callback_events = events.clone();
        let callback = ComponentRequestActivityCallback::new(move |event| {
            callback_events.lock().expect("event lock").push(event);
        });
        let lifecycle = ArcRwSignal::new(RequestLifecycle::<u64>::default());

        for (attempt, completion) in [
            RequestCompletion::Retryable,
            RequestCompletion::TerminalFailure,
            RequestCompletion::Successful,
        ]
        .into_iter()
        .enumerate()
        {
            lifecycle.update(|state| {
                assert_eq!(
                    state.prepare(true, attempt as u64).1,
                    RequestLifecycleDecision::Start
                );
            });
            let mut guard = RequestActivityGuard::new(lifecycle.clone(), Some(callback.clone()));
            guard.complete(completion);
            drop(guard);
        }

        assert_eq!(
            *events.lock().expect("event lock"),
            vec![
                ComponentRequestActivity::Started,
                ComponentRequestActivity::Settled,
                ComponentRequestActivity::Started,
                ComponentRequestActivity::Settled,
                ComponentRequestActivity::Started,
                ComponentRequestActivity::Settled,
            ]
        );
    }
}
