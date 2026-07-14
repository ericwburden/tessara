#[cfg(any(feature = "hydrate", test))]
use leptos::prelude::*;

/// The single editor-wide operation which currently owns mutable composition state.
///
/// Dashboard metadata updates refresh the composition payload, including placements,
/// so they must be serialized with layout saves and freeze all editor mutations.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(super) enum EditorOperation {
    #[default]
    Idle,
    SavingLayout,
    SavingSettings,
}

impl EditorOperation {
    pub(super) const fn is_busy(self) -> bool {
        !matches!(self, Self::Idle)
    }
}

#[cfg(any(feature = "hydrate", test))]
pub(super) fn try_begin_operation(
    operation: RwSignal<EditorOperation>,
    requested: EditorOperation,
) -> bool {
    let mut started = false;
    operation.update(|current| {
        if *current == EditorOperation::Idle && requested != EditorOperation::Idle {
            *current = requested;
            started = true;
        }
    });
    started
}

#[cfg(any(feature = "hydrate", test))]
pub(super) fn finish_operation(operation: RwSignal<EditorOperation>, completed: EditorOperation) {
    operation.update(|current| {
        if *current == completed {
            *current = EditorOperation::Idle;
        }
    });
}

#[cfg(test)]
mod tests {
    use super::{EditorOperation, finish_operation, try_begin_operation};
    use leptos::prelude::*;

    #[test]
    fn operations_are_serialized_and_only_the_owner_can_finish() {
        Owner::new().with(|| {
            let operation = RwSignal::new(EditorOperation::Idle);

            assert!(try_begin_operation(
                operation,
                EditorOperation::SavingSettings
            ));
            assert!(!try_begin_operation(
                operation,
                EditorOperation::SavingLayout
            ));

            finish_operation(operation, EditorOperation::SavingLayout);
            assert_eq!(operation.get_untracked(), EditorOperation::SavingSettings);

            finish_operation(operation, EditorOperation::SavingSettings);
            assert_eq!(operation.get_untracked(), EditorOperation::Idle);
        });
    }
}
