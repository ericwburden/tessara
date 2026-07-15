use semver::{Version, VersionReq};
use tessara_module_contract::{
    DependencyBindingKey, DependencyEvaluationFindingCode, DependencyEvaluationInput,
    DependencyRelationshipKind, FunctionalContractDeclaration, FunctionalContractId,
    FunctionalContractKind, FunctionalDependency, FunctionalDependencyEvaluation,
    FunctionalProviderCandidate, FunctionalProviderCandidateOrigin, ModuleDefinitionId,
    ResolvedFunctionalDependencyBinding, evaluate_functional_dependency,
};

const EXPECTED_FINDING_CODES: [(&str, DependencyEvaluationFindingCode); 8] = [
    (
        "dependency_missing_provider",
        DependencyEvaluationFindingCode::MissingProvider,
    ),
    (
        "dependency_incompatible_contract",
        DependencyEvaluationFindingCode::IncompatibleContract,
    ),
    (
        "dependency_incompatible_version",
        DependencyEvaluationFindingCode::IncompatibleVersion,
    ),
    (
        "dependency_ambiguous_binding",
        DependencyEvaluationFindingCode::AmbiguousBinding,
    ),
    ("dependency_cycle", DependencyEvaluationFindingCode::Cycle),
    (
        "transition_internal_only",
        DependencyEvaluationFindingCode::TransitionInternalOnly,
    ),
    (
        "dependency_provider_ineligible",
        DependencyEvaluationFindingCode::ProviderIneligible,
    ),
    (
        "dependency_not_evaluated",
        DependencyEvaluationFindingCode::NotEvaluated,
    ),
];

fn definition(value: &str) -> ModuleDefinitionId {
    ModuleDefinitionId::new(value).expect("test definition id")
}

fn binding(value: &str) -> DependencyBindingKey {
    DependencyBindingKey::new(value).expect("test binding key")
}

fn contract(value: &str) -> FunctionalContractId {
    FunctionalContractId::new(value).expect("test contract id")
}

fn dependency() -> FunctionalDependency {
    FunctionalDependency {
        contract_id: contract("example.records.query"),
        version_requirement: VersionReq::parse("^2.1").expect("test version requirement"),
        binding_key: binding("example.consumer.records"),
        optional: false,
    }
}

fn declaration(id: &str, version: &str) -> FunctionalContractDeclaration {
    FunctionalContractDeclaration {
        id: contract(id),
        version: Version::parse(version).expect("test contract version"),
        kind: FunctionalContractKind::Api,
        description: "Synthetic contract used only by the pure evaluator.".into(),
    }
}

fn release_candidate(
    definition_id: &str,
    binding_key: &str,
    contract_id: &str,
    contract_version: &str,
) -> FunctionalProviderCandidate {
    FunctionalProviderCandidate {
        provider_definition_id: definition(definition_id),
        binding_key: binding(binding_key),
        origin: FunctionalProviderCandidateOrigin::ModuleRelease {
            release_version: Version::parse("7.0.0").expect("test release version"),
            provider_eligible: true,
        },
        provided_contracts: vec![declaration(contract_id, contract_version)],
    }
}

fn evaluate<'a>(
    dependency: &'a FunctionalDependency,
    candidates: &'a [FunctionalProviderCandidate],
    bindings: &'a [ResolvedFunctionalDependencyBinding],
) -> FunctionalDependencyEvaluation {
    let consumer = definition("example.consumer");
    evaluate_functional_dependency(DependencyEvaluationInput {
        evaluation_requested: true,
        relationship: DependencyRelationshipKind::ModuleRelease,
        consumer_definition_id: &consumer,
        dependency,
        candidates,
        resolved_bindings: bindings,
    })
}

#[test]
fn missing_provider_is_distinct_from_a_candidate_under_another_binding() {
    let dependency = dependency();
    let candidates = [release_candidate(
        "example.provider",
        "example.consumer.another-binding",
        "example.records.query",
        "2.1.0",
    )];

    assert_eq!(
        evaluate(&dependency, &candidates, &[]),
        FunctionalDependencyEvaluation::MissingProvider
    );
}

#[test]
fn contract_identity_and_semver_incompatibility_are_separate_results() {
    let dependency = dependency();
    let wrong_contract = [release_candidate(
        "example.provider",
        "example.consumer.records",
        "example.records.write",
        "2.1.0",
    )];
    let wrong_version = [release_candidate(
        "example.provider",
        "example.consumer.records",
        "example.records.query",
        "3.0.0",
    )];

    assert_eq!(
        evaluate(&dependency, &wrong_contract, &[]),
        FunctionalDependencyEvaluation::IncompatibleContract
    );
    assert_eq!(
        evaluate(&dependency, &wrong_version, &[]),
        FunctionalDependencyEvaluation::IncompatibleVersion
    );
}

#[test]
fn two_eligible_semver_compatible_candidates_are_an_ambiguous_binding() {
    let dependency = dependency();
    let candidates = [
        release_candidate(
            "example.provider-a",
            "example.consumer.records",
            "example.records.query",
            "2.1.0",
        ),
        release_candidate(
            "example.provider-b",
            "example.consumer.records",
            "example.records.query",
            "2.4.0",
        ),
    ];

    assert_eq!(
        evaluate(&dependency, &candidates, &[]),
        FunctionalDependencyEvaluation::AmbiguousBinding
    );
}

#[test]
fn direct_and_transitive_cycles_are_reported_after_unique_binding() {
    let dependency = dependency();
    let candidate = release_candidate(
        "example.provider",
        "example.consumer.records",
        "example.records.query",
        "2.3.0",
    );
    let direct = [ResolvedFunctionalDependencyBinding {
        consumer_definition_id: definition("example.provider"),
        provider_definition_id: definition("example.consumer"),
    }];
    let transitive = [
        ResolvedFunctionalDependencyBinding {
            consumer_definition_id: definition("example.provider"),
            provider_definition_id: definition("example.middle"),
        },
        ResolvedFunctionalDependencyBinding {
            consumer_definition_id: definition("example.middle"),
            provider_definition_id: definition("example.consumer"),
        },
    ];
    let expected = FunctionalDependencyEvaluation::Cycle {
        provider_definition_id: definition("example.provider"),
    };

    assert_eq!(
        evaluate(&dependency, std::slice::from_ref(&candidate), &direct),
        expected
    );
    assert_eq!(evaluate(&dependency, &[candidate], &transitive), expected);
}

#[test]
fn transition_relationship_and_transition_provider_ineligibility_stay_distinct() {
    let dependency = dependency();
    let consumer = definition("example.consumer");
    let transition_candidate = FunctionalProviderCandidate {
        provider_definition_id: definition("example.transition-provider"),
        binding_key: binding("example.consumer.records"),
        origin: FunctionalProviderCandidateOrigin::TransitionalContribution {},
        provided_contracts: vec![declaration("example.records.query", "2.2.0")],
    };

    let internal = evaluate_functional_dependency(DependencyEvaluationInput {
        evaluation_requested: true,
        relationship: DependencyRelationshipKind::TransitionInternal,
        consumer_definition_id: &consumer,
        dependency: &dependency,
        candidates: &[],
        resolved_bindings: &[],
    });
    let provider_attempt = evaluate(&dependency, &[transition_candidate], &[]);

    assert_eq!(
        internal,
        FunctionalDependencyEvaluation::TransitionInternalOnly
    );
    assert_eq!(
        provider_attempt,
        FunctionalDependencyEvaluation::ProviderIneligible
    );
    assert!(
        serde_json::from_value::<FunctionalProviderCandidateOrigin>(serde_json::json!({
            "kind": "transitional_contribution",
            "provider_eligible": true
        }))
        .is_err(),
        "transition candidates cannot claim eligibility on the wire"
    );
}

#[test]
fn not_evaluated_precedes_all_other_classifications() {
    let dependency = dependency();
    let consumer = definition("example.consumer");

    let result = evaluate_functional_dependency(DependencyEvaluationInput {
        evaluation_requested: false,
        relationship: DependencyRelationshipKind::TransitionInternal,
        consumer_definition_id: &consumer,
        dependency: &dependency,
        candidates: &[],
        resolved_bindings: &[],
    });

    assert_eq!(result, FunctionalDependencyEvaluation::NotEvaluated);
}

#[test]
fn a_unique_eligible_candidate_returns_the_exact_compatible_contract_version() {
    let dependency = dependency();
    let candidate = release_candidate(
        "example.provider",
        "example.consumer.records",
        "example.records.query",
        "2.7.1",
    );

    assert_eq!(
        evaluate(&dependency, &[candidate], &[]),
        FunctionalDependencyEvaluation::Satisfied {
            provider_definition_id: definition("example.provider"),
            contract_version: Version::parse("2.7.1").expect("expected version"),
        }
    );
}

#[test]
fn every_non_satisfied_result_has_one_durable_closed_finding_code() {
    for (stable_code, finding) in EXPECTED_FINDING_CODES {
        assert_eq!(finding.stable_code(), stable_code);
        assert_eq!(
            serde_json::to_value(finding).expect("finding serializes"),
            serde_json::Value::String(stable_code.to_string())
        );
    }

    let results = [
        FunctionalDependencyEvaluation::MissingProvider,
        FunctionalDependencyEvaluation::IncompatibleContract,
        FunctionalDependencyEvaluation::IncompatibleVersion,
        FunctionalDependencyEvaluation::AmbiguousBinding,
        FunctionalDependencyEvaluation::Cycle {
            provider_definition_id: definition("example.provider"),
        },
        FunctionalDependencyEvaluation::TransitionInternalOnly,
        FunctionalDependencyEvaluation::ProviderIneligible,
        FunctionalDependencyEvaluation::NotEvaluated,
    ];
    assert_eq!(
        results
            .iter()
            .map(FunctionalDependencyEvaluation::finding_code)
            .collect::<Vec<_>>(),
        EXPECTED_FINDING_CODES
            .iter()
            .map(|(_, code)| Some(*code))
            .collect::<Vec<_>>()
    );
    assert_eq!(
        serde_json::to_value(FunctionalDependencyEvaluation::NotEvaluated)
            .expect("result serializes"),
        serde_json::json!({"state": "not_evaluated"})
    );
    assert!(
        serde_json::from_value::<DependencyEvaluationFindingCode>(serde_json::json!(
            "dependency_future_state"
        ))
        .is_err(),
        "unknown finding codes must fail closed"
    );
    assert!(
        serde_json::from_value::<FunctionalDependencyEvaluation>(serde_json::json!({
            "state": "future_state"
        }))
        .is_err(),
        "unknown result states must fail closed"
    );
}
