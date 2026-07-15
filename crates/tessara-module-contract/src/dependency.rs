//! Pure functional-dependency evaluation shared by Core and future modules.
//!
//! The evaluator deliberately consumes synthetic discovery inputs. Sprint 6A
//! defines how dependency evidence is classified without introducing provider
//! bindings, Module Release persistence, or Module Instance runtime state.

use std::collections::BTreeSet;

use semver::Version;
use serde::{Deserialize, Serialize};

use crate::{
    DependencyBindingKey, FunctionalContractDeclaration, FunctionalDependency, ModuleDefinitionId,
};

/// Whether the dependency is a future Module Release relationship or a
/// description of current in-process transition coupling.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DependencyRelationshipKind {
    /// A dependency declared by a real Module Release.
    ModuleRelease,
    /// A discovery-only relationship declared by an in-process transition.
    TransitionInternal,
}

/// Candidate provenance keeps transition contributions structurally
/// ineligible even if a caller accidentally tries to treat one as healthy.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum FunctionalProviderCandidateOrigin {
    /// A synthetic observation of a future Module Release candidate.
    ModuleRelease {
        /// Release version identifies a candidate without persisting a release.
        release_version: Version,
        /// Eligibility is evaluated outside this pure contract boundary from
        /// trust, compatibility, and lifecycle evidence.
        provider_eligible: bool,
    },
    /// Current transition contributions are never provider candidates.
    TransitionalContribution {},
}

impl FunctionalProviderCandidateOrigin {
    const fn is_provider_eligible(&self) -> bool {
        matches!(
            self,
            Self::ModuleRelease {
                provider_eligible: true,
                ..
            }
        )
    }
}

/// One synthetic provider candidate presented for a named dependency binding.
///
/// Feature declarations and security capabilities cannot be supplied here, so
/// matching labels or capability keys cannot satisfy a functional dependency.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FunctionalProviderCandidate {
    pub provider_definition_id: ModuleDefinitionId,
    pub binding_key: DependencyBindingKey,
    pub origin: FunctionalProviderCandidateOrigin,
    #[serde(default)]
    pub provided_contracts: Vec<FunctionalContractDeclaration>,
}

/// One already-resolved consumer-to-provider edge used only for pure cycle
/// detection. Sprint 6A does not persist this binding.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ResolvedFunctionalDependencyBinding {
    pub consumer_definition_id: ModuleDefinitionId,
    pub provider_definition_id: ModuleDefinitionId,
}

/// Borrowed inputs for one deterministic dependency evaluation.
#[derive(Clone, Copy, Debug)]
pub struct DependencyEvaluationInput<'a> {
    /// False means the caller lacks sufficient evidence to run evaluation.
    pub evaluation_requested: bool,
    pub relationship: DependencyRelationshipKind,
    pub consumer_definition_id: &'a ModuleDefinitionId,
    pub dependency: &'a FunctionalDependency,
    pub candidates: &'a [FunctionalProviderCandidate],
    pub resolved_bindings: &'a [ResolvedFunctionalDependencyBinding],
}

/// Closed dependency results. A caller cannot collapse missing, incompatible,
/// ambiguous, cyclic, transition-only, ineligible, or unevaluated evidence into
/// one generic warning.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "state", rename_all = "snake_case", deny_unknown_fields)]
pub enum FunctionalDependencyEvaluation {
    Satisfied {
        provider_definition_id: ModuleDefinitionId,
        contract_version: Version,
    },
    MissingProvider,
    IncompatibleContract,
    IncompatibleVersion,
    AmbiguousBinding,
    Cycle {
        provider_definition_id: ModuleDefinitionId,
    },
    TransitionInternalOnly,
    ProviderIneligible,
    NotEvaluated,
}

/// Stable, closed finding codes for every non-satisfied evaluation result.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum DependencyEvaluationFindingCode {
    #[serde(rename = "dependency_missing_provider")]
    MissingProvider,
    #[serde(rename = "dependency_incompatible_contract")]
    IncompatibleContract,
    #[serde(rename = "dependency_incompatible_version")]
    IncompatibleVersion,
    #[serde(rename = "dependency_ambiguous_binding")]
    AmbiguousBinding,
    #[serde(rename = "dependency_cycle")]
    Cycle,
    #[serde(rename = "transition_internal_only")]
    TransitionInternalOnly,
    #[serde(rename = "dependency_provider_ineligible")]
    ProviderIneligible,
    #[serde(rename = "dependency_not_evaluated")]
    NotEvaluated,
}

impl DependencyEvaluationFindingCode {
    /// Stable catalog/API code. Keep these values durable once emitted.
    pub const fn stable_code(self) -> &'static str {
        match self {
            Self::MissingProvider => "dependency_missing_provider",
            Self::IncompatibleContract => "dependency_incompatible_contract",
            Self::IncompatibleVersion => "dependency_incompatible_version",
            Self::AmbiguousBinding => "dependency_ambiguous_binding",
            Self::Cycle => "dependency_cycle",
            Self::TransitionInternalOnly => "transition_internal_only",
            Self::ProviderIneligible => "dependency_provider_ineligible",
            Self::NotEvaluated => "dependency_not_evaluated",
        }
    }
}

impl FunctionalDependencyEvaluation {
    /// Returns a closed finding code for every result except `Satisfied`.
    pub const fn finding_code(&self) -> Option<DependencyEvaluationFindingCode> {
        match self {
            Self::Satisfied { .. } => None,
            Self::MissingProvider => Some(DependencyEvaluationFindingCode::MissingProvider),
            Self::IncompatibleContract => {
                Some(DependencyEvaluationFindingCode::IncompatibleContract)
            }
            Self::IncompatibleVersion => Some(DependencyEvaluationFindingCode::IncompatibleVersion),
            Self::AmbiguousBinding => Some(DependencyEvaluationFindingCode::AmbiguousBinding),
            Self::Cycle { .. } => Some(DependencyEvaluationFindingCode::Cycle),
            Self::TransitionInternalOnly => {
                Some(DependencyEvaluationFindingCode::TransitionInternalOnly)
            }
            Self::ProviderIneligible => Some(DependencyEvaluationFindingCode::ProviderIneligible),
            Self::NotEvaluated => Some(DependencyEvaluationFindingCode::NotEvaluated),
        }
    }
}

/// Evaluates one functional dependency without HTTP, persistence, process, or
/// runtime-health inputs.
///
/// Classification precedence is intentional: evidence availability and
/// transition relationships are decided first, then binding, contract,
/// semantic-version, eligibility, ambiguity, and finally cycle checks.
pub fn evaluate_functional_dependency(
    input: DependencyEvaluationInput<'_>,
) -> FunctionalDependencyEvaluation {
    if !input.evaluation_requested {
        return FunctionalDependencyEvaluation::NotEvaluated;
    }
    if input.relationship == DependencyRelationshipKind::TransitionInternal {
        return FunctionalDependencyEvaluation::TransitionInternalOnly;
    }

    let binding_candidates = input
        .candidates
        .iter()
        .filter(|candidate| candidate.binding_key == input.dependency.binding_key)
        .collect::<Vec<_>>();
    if binding_candidates.is_empty() {
        return FunctionalDependencyEvaluation::MissingProvider;
    }

    let contract_candidates = binding_candidates
        .into_iter()
        .filter_map(|candidate| {
            let contract_versions = candidate
                .provided_contracts
                .iter()
                .filter(|contract| contract.id == input.dependency.contract_id)
                .map(|contract| &contract.version)
                .collect::<Vec<_>>();
            (!contract_versions.is_empty()).then_some((candidate, contract_versions))
        })
        .collect::<Vec<_>>();
    if contract_candidates.is_empty() {
        return FunctionalDependencyEvaluation::IncompatibleContract;
    }

    let version_candidates = contract_candidates
        .into_iter()
        .filter_map(|(candidate, versions)| {
            versions
                .into_iter()
                .filter(|version| input.dependency.version_requirement.matches(version))
                .max()
                .map(|version| (candidate, version))
        })
        .collect::<Vec<_>>();
    if version_candidates.is_empty() {
        return FunctionalDependencyEvaluation::IncompatibleVersion;
    }

    let eligible_candidates = version_candidates
        .into_iter()
        .filter(|(candidate, _)| candidate.origin.is_provider_eligible())
        .collect::<Vec<_>>();
    if eligible_candidates.is_empty() {
        return FunctionalDependencyEvaluation::ProviderIneligible;
    }

    let distinct_candidates = eligible_candidates
        .iter()
        .map(|(candidate, _)| (&candidate.provider_definition_id, &candidate.origin))
        .collect::<BTreeSet<_>>();
    if distinct_candidates.len() > 1 {
        return FunctionalDependencyEvaluation::AmbiguousBinding;
    }

    let (candidate, contract_version) = eligible_candidates
        .into_iter()
        .next()
        .expect("the eligible candidate set was proven non-empty");
    if creates_cycle(
        input.consumer_definition_id,
        &candidate.provider_definition_id,
        input.resolved_bindings,
    ) {
        return FunctionalDependencyEvaluation::Cycle {
            provider_definition_id: candidate.provider_definition_id.clone(),
        };
    }

    FunctionalDependencyEvaluation::Satisfied {
        provider_definition_id: candidate.provider_definition_id.clone(),
        contract_version: contract_version.clone(),
    }
}

fn creates_cycle(
    consumer: &ModuleDefinitionId,
    provider: &ModuleDefinitionId,
    resolved_bindings: &[ResolvedFunctionalDependencyBinding],
) -> bool {
    let mut pending = vec![provider];
    let mut visited = BTreeSet::new();

    while let Some(current) = pending.pop() {
        if current == consumer {
            return true;
        }
        if !visited.insert(current) {
            continue;
        }
        pending.extend(
            resolved_bindings
                .iter()
                .filter(|binding| &binding.consumer_definition_id == current)
                .map(|binding| &binding.provider_definition_id),
        );
    }

    false
}
