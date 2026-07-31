use semver::Version;
use sha2::{Digest, Sha256};
use tessara_module_contract::{
    ArtifactDigest, DeploymentProfile, FunctionalContractKind, InventoryEntry,
    ManifestNamespaceAuthority, ModuleDefinitionId, ModuleInstance, ModuleManifest,
    OciImageDeclaration, PublisherId, ResourceResolutionV1, RouteKind, RouteParameterType,
    SemanticDestination, TransitionAvailability, TransitionalContributionDescriptorV1,
    ValidationFinding,
};

const VALID_MANIFEST: &str = include_str!("fixtures/valid-manifest.json");
const VALID_MANIFEST_DIGEST_SIDECAR: &str = include_str!("fixtures/valid-manifest.json.sha256");
const VALID_MANIFEST_SHA256: &str =
    "sha256:2f3b838209a45fd51efb437b7a3e88ed1cdd7a97f65f65b571de2837215da9bc";
const INVALID_MANIFEST_PROFILE: &str =
    include_str!("fixtures/invalid-manifest-unsupported-profile.json");
const INVALID_TRANSITION_DEPLOYMENT: &str =
    include_str!("fixtures/invalid-transition-deployment-v1.json");
const INVALID_MANIFEST_SCHEMA: &str =
    include_str!("fixtures/invalid-manifest-unsupported-schema.json");
const INVALID_TRANSITION_SCHEMA: &str =
    include_str!("fixtures/invalid-transition-unsupported-schema-v1.json");
const INVALID_ARTIFACT_DIGEST: &str =
    include_str!("fixtures/invalid-artifact-digest-uppercase-v1.json");
const INVALID_SEMANTIC_DESTINATION: &str =
    include_str!("fixtures/invalid-semantic-destination-url-v1.json");
const INVALID_MODULE_INSTANCE: &str =
    include_str!("fixtures/invalid-module-instance-unknown-field-v1.json");
const INVALID_RESOURCE_RESOLUTION: &str =
    include_str!("fixtures/invalid-resource-resolution-authorized-undisclosed-v1.json");
const INVALID_OCI_IMAGE_REFERENCE: &str =
    include_str!("fixtures/invalid-oci-image-reference-v1.json");

struct FixtureSource {
    name: &'static str,
    bytes: &'static [u8],
    digest_sidecar: &'static str,
    expected_digest: &'static str,
}

const TRANSITION_SOURCES: &[FixtureSource] = &[
    FixtureSource {
        name: "Forms",
        bytes: include_bytes!("fixtures/transition-forms-v1.json"),
        digest_sidecar: include_str!("fixtures/transition-forms-v1.json.sha256"),
        expected_digest: "sha256:71bebdd07ff0028cc0da8bbd9707c393bade9951e5cedb265a4b8465d54b493e",
    },
    FixtureSource {
        name: "Workflows",
        bytes: include_bytes!("fixtures/transition-workflows-v1.json"),
        digest_sidecar: include_str!("fixtures/transition-workflows-v1.json.sha256"),
        expected_digest: "sha256:e9bdf51896700ffb982a00e4c80ea198bbdb98056705036a1a948347a71c04cf",
    },
    FixtureSource {
        name: "Responses",
        bytes: include_bytes!("fixtures/transition-responses-v1.json"),
        digest_sidecar: include_str!("fixtures/transition-responses-v1.json.sha256"),
        expected_digest: "sha256:e491986ed43b0f290f0c2ee763e60afb03e5b7babc7117a11e280e37de7b91bc",
    },
    FixtureSource {
        name: "Datasets",
        bytes: include_bytes!("fixtures/transition-datasets-v1.json"),
        digest_sidecar: include_str!("fixtures/transition-datasets-v1.json.sha256"),
        expected_digest: "sha256:ca301f4ac9a589d498bc25c77de4223b33de90569ecf54974976424c07fb4614",
    },
    FixtureSource {
        name: "Components",
        bytes: include_bytes!("fixtures/transition-components-v1.json"),
        digest_sidecar: include_str!("fixtures/transition-components-v1.json.sha256"),
        expected_digest: "sha256:344388304b015421ea71b5e303e7b9699264aef51c116b56d7f52e1b92443499",
    },
    FixtureSource {
        name: "Dashboards",
        bytes: include_bytes!("fixtures/transition-dashboards-v1.json"),
        digest_sidecar: include_str!("fixtures/transition-dashboards-v1.json.sha256"),
        expected_digest: "sha256:c82ecc7c3d121d1e1498c130133e487c8a68899b9255951e97955ce0de76bbe5",
    },
    FixtureSource {
        name: "Migration",
        bytes: include_bytes!("fixtures/transition-migration-v1.json"),
        digest_sidecar: include_str!("fixtures/transition-migration-v1.json.sha256"),
        expected_digest: "sha256:de48eeb3edb4a432e5060b817ef50c34c5316879b44aef0ad3d6877c5895b42e",
    },
];

fn assert_exact_utf8_lf_fixture<'a>(
    name: &str,
    bytes: &'a [u8],
    digest_sidecar: &str,
    expected_digest: &str,
) -> &'a str {
    assert!(!bytes.is_empty(), "{name} fixture must not be empty");
    assert!(
        !bytes.starts_with(&[0xef, 0xbb, 0xbf]),
        "{name} fixture must not contain a UTF-8 BOM"
    );
    assert!(
        !bytes.contains(&b'\r'),
        "{name} fixture must use LF-only newlines"
    );
    assert!(
        !bytes.contains(&0),
        "{name} fixture must not contain NUL bytes"
    );
    assert!(
        bytes.ends_with(b"\n"),
        "{name} fixture must end with one LF"
    );
    assert!(
        bytes.len() == 1 || bytes[bytes.len() - 2] != b'\n',
        "{name} fixture must end with exactly one LF"
    );
    let source_text = std::str::from_utf8(bytes).expect("canonical fixture is UTF-8");

    ArtifactDigest::new(expected_digest).expect("hard-coded digest uses canonical syntax");
    let lower_case_digest = expected_digest.to_ascii_lowercase();
    assert_eq!(expected_digest, lower_case_digest.as_str());
    let expected_sidecar = format!("{expected_digest}\n");
    assert_eq!(
        digest_sidecar,
        expected_sidecar.as_str(),
        "{name} sidecar must contain the independently pinned digest and one LF"
    );
    let actual_digest = format!("sha256:{:x}", Sha256::digest(bytes));
    assert_eq!(
        actual_digest.as_str(),
        expected_digest,
        "{name} fixture bytes differ from the executable digest pin"
    );
    source_text
}

struct ExpectedFeature {
    id: &'static str,
    contracts: &'static [&'static str],
    resources: &'static [&'static str],
    destinations: &'static [&'static str],
    capabilities: &'static [&'static str],
}

struct ExpectedContract {
    id: &'static str,
    kind: FunctionalContractKind,
}

struct ExpectedDependency {
    binding_key: &'static str,
    contract_id: &'static str,
}

struct ExpectedParameter {
    name: &'static str,
    value_type: RouteParameterType,
}

struct ExpectedRoute {
    name: &'static str,
    parameters: &'static [ExpectedParameter],
}

struct ExpectedNavigation {
    id: &'static str,
    destination: &'static str,
    label: &'static str,
    group: &'static str,
    order_hint: i32,
    capabilities: &'static [&'static str],
}

struct ExpectedCapability {
    id: &'static str,
    description: &'static str,
}

struct ExpectedCatalogEntry {
    display_name: &'static str,
    definition_id: &'static str,
    availability: TransitionAvailability,
    features: &'static [ExpectedFeature],
    contracts: &'static [ExpectedContract],
    dependencies: &'static [ExpectedDependency],
    resources: &'static [&'static str],
    routes: &'static [ExpectedRoute],
    navigation: Option<ExpectedNavigation>,
    capabilities: &'static [ExpectedCapability],
}

const EXPECTED_CATALOG: &[ExpectedCatalogEntry] = &[
    ExpectedCatalogEntry {
        display_name: "Forms",
        definition_id: "tessara.forms",
        availability: TransitionAvailability::ActiveInProcess,
        features: &[
            ExpectedFeature {
                id: "tessara.forms.authoring",
                contracts: &["tessara.forms.form", "tessara.forms.form-version"],
                resources: &["tessara.transition.form", "tessara.transition.form_version"],
                destinations: &["forms.directory", "forms.create", "forms.edit"],
                capabilities: &["forms:read", "forms:manage"],
            },
            ExpectedFeature {
                id: "tessara.forms.publication",
                contracts: &["tessara.forms.form-version"],
                resources: &["tessara.transition.form_version"],
                destinations: &["forms.edit"],
                capabilities: &["forms:manage"],
            },
            ExpectedFeature {
                id: "tessara.forms.lookup",
                contracts: &["tessara.forms.form", "tessara.forms.form-version"],
                resources: &["tessara.transition.form", "tessara.transition.form_version"],
                destinations: &["forms.directory", "forms.detail"],
                capabilities: &["forms:read"],
            },
        ],
        contracts: &[
            ExpectedContract {
                id: "tessara.forms.form",
                kind: FunctionalContractKind::Resource,
            },
            ExpectedContract {
                id: "tessara.forms.form-version",
                kind: FunctionalContractKind::Resource,
            },
        ],
        dependencies: &[],
        resources: &["tessara.transition.form", "tessara.transition.form_version"],
        routes: &[
            ExpectedRoute {
                name: "forms.directory",
                parameters: &[],
            },
            ExpectedRoute {
                name: "forms.create",
                parameters: &[],
            },
            ExpectedRoute {
                name: "forms.detail",
                parameters: &[ExpectedParameter {
                    name: "form_id",
                    value_type: RouteParameterType::Uuid,
                }],
            },
            ExpectedRoute {
                name: "forms.edit",
                parameters: &[ExpectedParameter {
                    name: "form_id",
                    value_type: RouteParameterType::Uuid,
                }],
            },
        ],
        navigation: Some(ExpectedNavigation {
            id: "tessara.forms.navigation",
            destination: "forms.directory",
            label: "Forms",
            group: "Main",
            order_hint: 20,
            capabilities: &["forms:read", "forms:manage"],
        }),
        capabilities: &[
            ExpectedCapability {
                id: "forms:read",
                description: "Browse top-level form records",
            },
            ExpectedCapability {
                id: "forms:manage",
                description: "Manage form definitions and versions",
            },
        ],
    },
    ExpectedCatalogEntry {
        display_name: "Workflows",
        definition_id: "tessara.workflows",
        availability: TransitionAvailability::ActiveInProcess,
        features: &[
            ExpectedFeature {
                id: "tessara.workflows.authoring",
                contracts: &[
                    "tessara.workflows.workflow",
                    "tessara.workflows.workflow-version",
                ],
                resources: &[
                    "tessara.transition.workflow",
                    "tessara.transition.workflow_version",
                ],
                destinations: &["workflows.directory", "workflows.create", "workflows.edit"],
                capabilities: &["workflows:manage"],
            },
            ExpectedFeature {
                id: "tessara.workflows.assignment",
                contracts: &["tessara.workflows.assignment"],
                resources: &[],
                destinations: &["workflows.assignments"],
                capabilities: &["workflows:read", "workflows:manage"],
            },
            ExpectedFeature {
                id: "tessara.workflows.execution",
                contracts: &[
                    "tessara.workflows.workflow-version",
                    "tessara.forms.form-version",
                ],
                resources: &["tessara.transition.workflow_version"],
                destinations: &["workflows.detail"],
                capabilities: &["workflows:read"],
            },
        ],
        contracts: &[
            ExpectedContract {
                id: "tessara.workflows.workflow",
                kind: FunctionalContractKind::Resource,
            },
            ExpectedContract {
                id: "tessara.workflows.workflow-version",
                kind: FunctionalContractKind::Resource,
            },
            ExpectedContract {
                id: "tessara.workflows.assignment",
                kind: FunctionalContractKind::Behavior,
            },
        ],
        dependencies: &[ExpectedDependency {
            binding_key: "tessara.workflows.form-version",
            contract_id: "tessara.forms.form-version",
        }],
        resources: &[
            "tessara.transition.workflow",
            "tessara.transition.workflow_version",
        ],
        routes: &[
            ExpectedRoute {
                name: "workflows.directory",
                parameters: &[],
            },
            ExpectedRoute {
                name: "workflows.create",
                parameters: &[],
            },
            ExpectedRoute {
                name: "workflows.assignments",
                parameters: &[],
            },
            ExpectedRoute {
                name: "workflows.detail",
                parameters: &[ExpectedParameter {
                    name: "workflow_id",
                    value_type: RouteParameterType::Uuid,
                }],
            },
            ExpectedRoute {
                name: "workflows.edit",
                parameters: &[ExpectedParameter {
                    name: "workflow_id",
                    value_type: RouteParameterType::Uuid,
                }],
            },
        ],
        navigation: Some(ExpectedNavigation {
            id: "tessara.workflows.navigation",
            destination: "workflows.directory",
            label: "Workflows",
            group: "Main",
            order_hint: 30,
            capabilities: &["workflows:read", "workflows:manage"],
        }),
        capabilities: &[
            ExpectedCapability {
                id: "workflows:read",
                description: "Browse workflow definitions and assignments",
            },
            ExpectedCapability {
                id: "workflows:manage",
                description: "Manage workflow definitions and assignments",
            },
        ],
    },
    ExpectedCatalogEntry {
        display_name: "Responses",
        definition_id: "tessara.responses",
        availability: TransitionAvailability::ActiveInProcess,
        features: &[
            ExpectedFeature {
                id: "tessara.responses.start",
                contracts: &[
                    "tessara.workflows.workflow-version",
                    "tessara.forms.form-version",
                ],
                resources: &[],
                destinations: &["responses.start"],
                capabilities: &["submissions:respond"],
            },
            ExpectedFeature {
                id: "tessara.responses.draft",
                contracts: &[
                    "tessara.responses.response",
                    "tessara.responses.response-lifecycle",
                ],
                resources: &["tessara.transition.response"],
                destinations: &["responses.edit"],
                capabilities: &["submissions:respond"],
            },
            ExpectedFeature {
                id: "tessara.responses.submit",
                contracts: &[
                    "tessara.responses.response",
                    "tessara.responses.response-lifecycle",
                ],
                resources: &["tessara.transition.response"],
                destinations: &["responses.edit"],
                capabilities: &["submissions:respond"],
            },
            ExpectedFeature {
                id: "tessara.responses.review",
                contracts: &["tessara.responses.response"],
                resources: &["tessara.transition.response"],
                destinations: &["responses.detail"],
                capabilities: &["submissions:read_own", "submissions:manage"],
            },
        ],
        contracts: &[
            ExpectedContract {
                id: "tessara.responses.response",
                kind: FunctionalContractKind::Resource,
            },
            ExpectedContract {
                id: "tessara.responses.response-lifecycle",
                kind: FunctionalContractKind::Behavior,
            },
        ],
        dependencies: &[
            ExpectedDependency {
                binding_key: "tessara.responses.workflow-version",
                contract_id: "tessara.workflows.workflow-version",
            },
            ExpectedDependency {
                binding_key: "tessara.responses.form-version",
                contract_id: "tessara.forms.form-version",
            },
        ],
        resources: &["tessara.transition.response"],
        routes: &[
            ExpectedRoute {
                name: "responses.directory",
                parameters: &[],
            },
            ExpectedRoute {
                name: "responses.start",
                parameters: &[],
            },
            ExpectedRoute {
                name: "responses.detail",
                parameters: &[ExpectedParameter {
                    name: "submission_id",
                    value_type: RouteParameterType::Uuid,
                }],
            },
            ExpectedRoute {
                name: "responses.edit",
                parameters: &[ExpectedParameter {
                    name: "submission_id",
                    value_type: RouteParameterType::Uuid,
                }],
            },
        ],
        navigation: Some(ExpectedNavigation {
            id: "tessara.responses.navigation",
            destination: "responses.directory",
            label: "Responses",
            group: "Main",
            order_hint: 40,
            capabilities: &[
                "submissions:read_own",
                "submissions:respond",
                "submissions:manage",
            ],
        }),
        capabilities: &[
            ExpectedCapability {
                id: "submissions:read_own",
                description: "Read own and delegated response work",
            },
            ExpectedCapability {
                id: "submissions:respond",
                description: "Start and complete assigned response work",
            },
            ExpectedCapability {
                id: "submissions:manage",
                description: "Manage submissions by hierarchy scope",
            },
        ],
    },
    ExpectedCatalogEntry {
        display_name: "Datasets",
        definition_id: "tessara.datasets",
        availability: TransitionAvailability::ActiveInProcess,
        features: &[
            ExpectedFeature {
                id: "tessara.datasets.authoring",
                contracts: &[
                    "tessara.datasets.dataset",
                    "tessara.datasets.dataset-revision",
                ],
                resources: &[
                    "tessara.transition.dataset",
                    "tessara.transition.dataset_revision",
                ],
                destinations: &["datasets.directory", "datasets.create", "datasets.edit"],
                capabilities: &["datasets:manage"],
            },
            ExpectedFeature {
                id: "tessara.datasets.publication",
                contracts: &[
                    "tessara.datasets.dataset",
                    "tessara.datasets.dataset-revision",
                ],
                resources: &[
                    "tessara.transition.dataset",
                    "tessara.transition.dataset_revision",
                ],
                destinations: &[
                    "datasets.revisions",
                    "datasets.revision_detail",
                    "datasets.revision_edit",
                ],
                capabilities: &["datasets:manage"],
            },
            ExpectedFeature {
                id: "tessara.datasets.materialization",
                contracts: &[
                    "tessara.datasets.dataset-major-line",
                    "tessara.datasets.materialization",
                    "tessara.responses.response",
                    "tessara.forms.form-version",
                ],
                resources: &["tessara.transition.dataset_major_line"],
                destinations: &["datasets.revisions"],
                capabilities: &["datasets:manage"],
            },
            ExpectedFeature {
                id: "tessara.datasets.preview",
                contracts: &[
                    "tessara.datasets.dataset",
                    "tessara.datasets.dataset-revision",
                    "tessara.datasets.dataset-major-line",
                ],
                resources: &[
                    "tessara.transition.dataset",
                    "tessara.transition.dataset_revision",
                    "tessara.transition.dataset_major_line",
                ],
                destinations: &["datasets.detail", "datasets.preview"],
                capabilities: &[
                    "datasets:read",
                    "datasets:read_restricted",
                    "datasets:read_confidential",
                ],
            },
        ],
        contracts: &[
            ExpectedContract {
                id: "tessara.datasets.dataset",
                kind: FunctionalContractKind::Resource,
            },
            ExpectedContract {
                id: "tessara.datasets.dataset-revision",
                kind: FunctionalContractKind::Resource,
            },
            ExpectedContract {
                id: "tessara.datasets.dataset-major-line",
                kind: FunctionalContractKind::Resource,
            },
            ExpectedContract {
                id: "tessara.datasets.materialization",
                kind: FunctionalContractKind::Behavior,
            },
        ],
        dependencies: &[
            ExpectedDependency {
                binding_key: "tessara.datasets.response",
                contract_id: "tessara.responses.response",
            },
            ExpectedDependency {
                binding_key: "tessara.datasets.form-version",
                contract_id: "tessara.forms.form-version",
            },
        ],
        resources: &[
            "tessara.transition.dataset",
            "tessara.transition.dataset_revision",
            "tessara.transition.dataset_major_line",
        ],
        routes: &[
            ExpectedRoute {
                name: "datasets.directory",
                parameters: &[],
            },
            ExpectedRoute {
                name: "datasets.create",
                parameters: &[],
            },
            ExpectedRoute {
                name: "datasets.detail",
                parameters: &[ExpectedParameter {
                    name: "dataset_id",
                    value_type: RouteParameterType::Uuid,
                }],
            },
            ExpectedRoute {
                name: "datasets.preview",
                parameters: &[ExpectedParameter {
                    name: "dataset_id",
                    value_type: RouteParameterType::Uuid,
                }],
            },
            ExpectedRoute {
                name: "datasets.revisions",
                parameters: &[ExpectedParameter {
                    name: "dataset_id",
                    value_type: RouteParameterType::Uuid,
                }],
            },
            ExpectedRoute {
                name: "datasets.revision_detail",
                parameters: &[
                    ExpectedParameter {
                        name: "dataset_id",
                        value_type: RouteParameterType::Uuid,
                    },
                    ExpectedParameter {
                        name: "revision_id",
                        value_type: RouteParameterType::Uuid,
                    },
                ],
            },
            ExpectedRoute {
                name: "datasets.revision_edit",
                parameters: &[
                    ExpectedParameter {
                        name: "dataset_id",
                        value_type: RouteParameterType::Uuid,
                    },
                    ExpectedParameter {
                        name: "revision_id",
                        value_type: RouteParameterType::Uuid,
                    },
                ],
            },
            ExpectedRoute {
                name: "datasets.edit",
                parameters: &[ExpectedParameter {
                    name: "dataset_id",
                    value_type: RouteParameterType::Uuid,
                }],
            },
        ],
        navigation: Some(ExpectedNavigation {
            id: "tessara.datasets.navigation",
            destination: "datasets.directory",
            label: "Datasets",
            group: "Admin",
            order_hint: 20,
            capabilities: &["datasets:read", "datasets:manage"],
        }),
        capabilities: &[
            ExpectedCapability {
                id: "datasets:read",
                description: "Inspect dataset definitions",
            },
            ExpectedCapability {
                id: "datasets:manage",
                description: "Manage dataset definitions",
            },
            ExpectedCapability {
                id: "datasets:read_restricted",
                description: "Read restricted dataset rows when dataset visibility allows access",
            },
            ExpectedCapability {
                id: "datasets:read_confidential",
                description: "Read confidential and restricted dataset rows when dataset visibility allows access",
            },
        ],
    },
    ExpectedCatalogEntry {
        display_name: "Components",
        definition_id: "tessara.components",
        availability: TransitionAvailability::ActiveInProcess,
        features: &[
            ExpectedFeature {
                id: "tessara.components.authoring",
                contracts: &[
                    "tessara.components.component-version",
                    "tessara.datasets.dataset-major-line",
                ],
                resources: &["tessara.transition.component_version"],
                destinations: &[
                    "components.directory",
                    "components.create",
                    "components.edit",
                ],
                capabilities: &["components:manage"],
            },
            ExpectedFeature {
                id: "tessara.components.publication",
                contracts: &[
                    "tessara.components.component-version",
                    "tessara.datasets.dataset-major-line",
                ],
                resources: &["tessara.transition.component_version"],
                destinations: &["components.edit", "components.versions"],
                capabilities: &["components:manage"],
            },
            ExpectedFeature {
                id: "tessara.components.execution",
                contracts: &[
                    "tessara.components.component-version",
                    "tessara.components.execution",
                ],
                resources: &["tessara.transition.component_version"],
                destinations: &["components.view"],
                capabilities: &["components:read"],
            },
            ExpectedFeature {
                id: "tessara.components.viewing",
                contracts: &["tessara.components.component-version"],
                resources: &["tessara.transition.component_version"],
                destinations: &["components.detail", "components.view"],
                capabilities: &["components:read"],
            },
        ],
        contracts: &[
            ExpectedContract {
                id: "tessara.components.component-version",
                kind: FunctionalContractKind::Resource,
            },
            ExpectedContract {
                id: "tessara.components.execution",
                kind: FunctionalContractKind::Behavior,
            },
        ],
        dependencies: &[ExpectedDependency {
            binding_key: "tessara.components.dataset-major-line",
            contract_id: "tessara.datasets.dataset-major-line",
        }],
        resources: &["tessara.transition.component_version"],
        routes: &[
            ExpectedRoute {
                name: "components.directory",
                parameters: &[],
            },
            ExpectedRoute {
                name: "components.create",
                parameters: &[],
            },
            ExpectedRoute {
                name: "components.detail",
                parameters: &[ExpectedParameter {
                    name: "component_ref",
                    value_type: RouteParameterType::String,
                }],
            },
            ExpectedRoute {
                name: "components.edit",
                parameters: &[ExpectedParameter {
                    name: "component_ref",
                    value_type: RouteParameterType::String,
                }],
            },
            ExpectedRoute {
                name: "components.versions",
                parameters: &[ExpectedParameter {
                    name: "component_ref",
                    value_type: RouteParameterType::String,
                }],
            },
            ExpectedRoute {
                name: "components.view",
                parameters: &[ExpectedParameter {
                    name: "component_ref",
                    value_type: RouteParameterType::String,
                }],
            },
        ],
        navigation: Some(ExpectedNavigation {
            id: "tessara.components.navigation",
            destination: "components.directory",
            label: "Components",
            group: "Main",
            order_hint: 60,
            capabilities: &["components:read", "components:manage"],
        }),
        capabilities: &[
            ExpectedCapability {
                id: "components:read",
                description: "Inspect component definitions",
            },
            ExpectedCapability {
                id: "components:manage",
                description: "Manage component definitions",
            },
        ],
    },
    ExpectedCatalogEntry {
        display_name: "Dashboards",
        definition_id: "tessara.dashboards",
        availability: TransitionAvailability::ActiveInProcess,
        features: &[
            ExpectedFeature {
                id: "tessara.dashboards.composition",
                contracts: &[
                    "tessara.dashboards.dashboard",
                    "tessara.dashboards.composition",
                    "tessara.components.component-version",
                ],
                resources: &["tessara.transition.dashboard"],
                destinations: &["dashboards.create", "dashboards.edit"],
                capabilities: &["dashboards:manage"],
            },
            ExpectedFeature {
                id: "tessara.dashboards.viewing",
                contracts: &["tessara.dashboards.dashboard"],
                resources: &["tessara.transition.dashboard"],
                destinations: &["dashboards.detail", "dashboards.view"],
                capabilities: &["dashboards:read"],
            },
        ],
        contracts: &[
            ExpectedContract {
                id: "tessara.dashboards.dashboard",
                kind: FunctionalContractKind::Resource,
            },
            ExpectedContract {
                id: "tessara.dashboards.composition",
                kind: FunctionalContractKind::Behavior,
            },
        ],
        dependencies: &[ExpectedDependency {
            binding_key: "tessara.dashboards.component-version",
            contract_id: "tessara.components.component-version",
        }],
        resources: &["tessara.transition.dashboard"],
        routes: &[
            ExpectedRoute {
                name: "dashboards.directory",
                parameters: &[],
            },
            ExpectedRoute {
                name: "dashboards.create",
                parameters: &[],
            },
            ExpectedRoute {
                name: "dashboards.detail",
                parameters: &[ExpectedParameter {
                    name: "dashboard_id",
                    value_type: RouteParameterType::Uuid,
                }],
            },
            ExpectedRoute {
                name: "dashboards.edit",
                parameters: &[ExpectedParameter {
                    name: "dashboard_id",
                    value_type: RouteParameterType::Uuid,
                }],
            },
            ExpectedRoute {
                name: "dashboards.view",
                parameters: &[ExpectedParameter {
                    name: "dashboard_id",
                    value_type: RouteParameterType::Uuid,
                }],
            },
        ],
        navigation: Some(ExpectedNavigation {
            id: "tessara.dashboards.navigation",
            destination: "dashboards.directory",
            label: "Dashboards",
            group: "Main",
            order_hint: 70,
            capabilities: &["dashboards:read"],
        }),
        capabilities: &[
            ExpectedCapability {
                id: "dashboards:read",
                description: "Inspect dashboard definitions",
            },
            ExpectedCapability {
                id: "dashboards:manage",
                description: "Manage dashboard definitions",
            },
        ],
    },
    ExpectedCatalogEntry {
        display_name: "Migration",
        definition_id: "tessara.migration",
        availability: TransitionAvailability::Retired,
        features: &[],
        contracts: &[],
        dependencies: &[],
        resources: &[],
        routes: &[],
        navigation: None,
        capabilities: &[],
    },
];

fn forms_authority() -> ManifestNamespaceAuthority {
    ManifestNamespaceAuthority::new(
        ModuleDefinitionId::new("tessara.forms").unwrap(),
        PublisherId::new("tessara.first_party").unwrap(),
        ["tessara.forms", "forms"],
    )
    .unwrap()
}

fn assert_exact_catalog_entry(
    descriptor: &TransitionalContributionDescriptorV1,
    expected: &ExpectedCatalogEntry,
) {
    assert_eq!(
        descriptor.schema_version, 1,
        "{} schema",
        expected.display_name
    );
    assert_eq!(descriptor.display_name, expected.display_name);
    assert_eq!(
        descriptor.reserved_definition_id.as_str(),
        expected.definition_id
    );
    assert_eq!(descriptor.availability, expected.availability);
    assert_eq!(descriptor.features.len(), expected.features.len());

    for (feature, expected_feature) in descriptor.features.iter().zip(expected.features) {
        assert_eq!(feature.id.as_str(), expected_feature.id);
        assert!(!feature.name.trim().is_empty());
        assert!(!feature.description.trim().is_empty());
        assert!(!feature.use_cases.is_empty());
        assert!(!feature.inputs.is_empty());
        assert!(!feature.outcomes.is_empty());
        assert!(!feature.constraints.is_empty());
        assert_eq!(
            feature
                .contracts
                .iter()
                .map(|value| value.as_str())
                .collect::<Vec<_>>(),
            expected_feature.contracts
        );
        assert_eq!(
            feature
                .resource_types
                .iter()
                .map(|value| value.as_str())
                .collect::<Vec<_>>(),
            expected_feature.resources
        );
        assert_eq!(
            feature
                .destinations
                .iter()
                .map(|value| value.as_str())
                .collect::<Vec<_>>(),
            expected_feature.destinations
        );
        assert_eq!(
            feature
                .capabilities
                .iter()
                .map(|value| value.as_str())
                .collect::<Vec<_>>(),
            expected_feature.capabilities
        );
        assert!(feature.configuration_pointers.is_empty());
    }

    assert_eq!(
        descriptor.provided_contracts.len(),
        expected.contracts.len()
    );
    for (contract, expected_contract) in
        descriptor.provided_contracts.iter().zip(expected.contracts)
    {
        assert_eq!(contract.id.as_str(), expected_contract.id);
        assert_eq!(contract.kind, expected_contract.kind);
        assert_eq!(contract.version, Version::new(1, 0, 0));
    }

    assert_eq!(descriptor.dependencies.len(), expected.dependencies.len());
    for (dependency, expected_dependency) in
        descriptor.dependencies.iter().zip(expected.dependencies)
    {
        assert_eq!(
            dependency.binding_key.as_str(),
            expected_dependency.binding_key
        );
        assert_eq!(
            dependency.contract_id.as_str(),
            expected_dependency.contract_id
        );
        assert_eq!(dependency.version_requirement.to_string(), "^1.0");
        assert!(!dependency.optional);
    }

    assert_eq!(
        descriptor
            .resource_types
            .iter()
            .map(|resource| resource.id.as_str())
            .collect::<Vec<_>>(),
        expected.resources
    );

    assert_eq!(descriptor.routes.len(), expected.routes.len());
    for (route, expected_route) in descriptor.routes.iter().zip(expected.routes) {
        assert_eq!(route.name.as_str(), expected_route.name);
        assert_eq!(route.kind, RouteKind::Product);
        assert_eq!(route.parameters.len(), expected_route.parameters.len());
        for (parameter, expected_parameter) in
            route.parameters.iter().zip(expected_route.parameters)
        {
            assert_eq!(parameter.name, expected_parameter.name);
            assert_eq!(parameter.value_type, expected_parameter.value_type);
            assert!(parameter.required);
        }
    }

    match (&descriptor.navigation[..], expected.navigation.as_ref()) {
        ([], None) => {}
        ([navigation], Some(expected_navigation)) => {
            assert_eq!(navigation.id.as_str(), expected_navigation.id);
            assert_eq!(
                navigation.destination.as_str(),
                expected_navigation.destination
            );
            assert_eq!(navigation.label, expected_navigation.label);
            assert_eq!(navigation.group, expected_navigation.group);
            assert_eq!(navigation.order_hint, expected_navigation.order_hint);
            assert_eq!(
                navigation
                    .required_capabilities_any_of
                    .iter()
                    .map(|capability| capability.as_str())
                    .collect::<Vec<_>>(),
                expected_navigation.capabilities
            );
        }
        _ => panic!(
            "{} navigation shape differs from catalog",
            expected.display_name
        ),
    }

    assert_eq!(
        descriptor.security_capabilities.len(),
        expected.capabilities.len()
    );
    for (capability, expected_capability) in descriptor
        .security_capabilities
        .iter()
        .zip(expected.capabilities)
    {
        assert_eq!(capability.id.as_str(), expected_capability.id);
        assert_eq!(capability.description, expected_capability.description);
        assert_ne!(capability.id.as_str(), "admin:all");
    }
    assert!(descriptor.configuration_schema.is_none());
}

#[test]
fn canonical_valid_manifest_fixture_round_trips_and_validates() {
    assert_exact_utf8_lf_fixture(
        "valid Manifest",
        VALID_MANIFEST.as_bytes(),
        VALID_MANIFEST_DIGEST_SIDECAR,
        VALID_MANIFEST_SHA256,
    );
    let manifest: ModuleManifest = serde_json::from_slice(VALID_MANIFEST.as_bytes()).unwrap();
    manifest.validate(&forms_authority()).unwrap();

    let encoded = serde_json::to_string(&manifest).unwrap();
    let decoded: ModuleManifest = serde_json::from_str(&encoded).unwrap();
    assert_eq!(decoded, manifest);
}

#[test]
fn canonical_transition_sources_are_byte_pinned_valid_and_exact() {
    assert_eq!(TRANSITION_SOURCES.len(), 7);
    assert_eq!(EXPECTED_CATALOG.len(), 7);

    for (source, expected) in TRANSITION_SOURCES.iter().zip(EXPECTED_CATALOG) {
        assert_eq!(source.name, expected.display_name);
        let source_text = assert_exact_utf8_lf_fixture(
            source.name,
            source.bytes,
            source.digest_sidecar,
            source.expected_digest,
        );
        assert!(!source_text.contains("admin:all"));
        assert!(!source_text.contains("://"));

        let descriptor: TransitionalContributionDescriptorV1 =
            serde_json::from_slice(source.bytes).expect("canonical descriptor decodes");
        descriptor
            .validate()
            .expect("canonical descriptor validates");
        assert_exact_catalog_entry(&descriptor, expected);

        let semantic_round_trip: TransitionalContributionDescriptorV1 =
            serde_json::from_slice(&serde_json::to_vec(&descriptor).unwrap()).unwrap();
        assert_eq!(semantic_round_trip, descriptor);

        let inventory = InventoryEntry::TransitionalInProcess {
            descriptor: descriptor.clone(),
        };
        inventory.validate_integrity().unwrap();
        assert!(!inventory.provider_eligible());
        assert!(!inventory.supervisor_materializable());
        assert_eq!(
            serde_json::to_value(&inventory).unwrap()["kind"],
            "transitional_in_process"
        );
    }
}

#[test]
fn migration_is_the_only_retired_empty_transition_source() {
    for (source, expected) in TRANSITION_SOURCES.iter().zip(EXPECTED_CATALOG) {
        let descriptor: TransitionalContributionDescriptorV1 =
            serde_json::from_slice(source.bytes).unwrap();
        assert_ne!(descriptor.availability, TransitionAvailability::Unavailable);
        if expected.display_name == "Migration" {
            assert_eq!(descriptor.availability, TransitionAvailability::Retired);
            assert!(descriptor.features.is_empty());
            assert!(descriptor.provided_contracts.is_empty());
            assert!(descriptor.dependencies.is_empty());
            assert!(descriptor.resource_types.is_empty());
            assert!(descriptor.routes.is_empty());
            assert!(descriptor.navigation.is_empty());
            assert!(descriptor.security_capabilities.is_empty());
            assert!(descriptor.configuration_schema.is_none());
        } else {
            assert_eq!(
                descriptor.availability,
                TransitionAvailability::ActiveInProcess
            );
        }
    }
}

#[test]
fn invalid_fixtures_have_specific_wire_rejections() {
    let manifest_error = serde_json::from_str::<ModuleManifest>(INVALID_MANIFEST_PROFILE)
        .expect_err("unsupported deployment profile must fail at the wire boundary");
    assert_eq!(manifest_error.classify(), serde_json::error::Category::Data);
    assert!(manifest_error.to_string().contains("tessara-oci-v2"));
    assert!(manifest_error.to_string().contains("tessara-oci-v1"));

    let transition_error =
        serde_json::from_str::<TransitionalContributionDescriptorV1>(INVALID_TRANSITION_DEPLOYMENT)
            .expect_err("transition deployment claim must fail at the wire boundary");
    assert_eq!(
        transition_error.classify(),
        serde_json::error::Category::Data
    );
    assert!(
        transition_error
            .to_string()
            .contains("unknown field `deployment`")
    );

    let manifest_schema_error = serde_json::from_str::<ModuleManifest>(INVALID_MANIFEST_SCHEMA)
        .expect_err("unsupported Manifest schema must fail during deserialization");
    assert_eq!(
        manifest_schema_error.classify(),
        serde_json::error::Category::Data
    );
    assert!(
        manifest_schema_error
            .to_string()
            .contains("module manifest schema version 1 is unsupported; expected 3")
    );

    let transition_schema_error =
        serde_json::from_str::<TransitionalContributionDescriptorV1>(INVALID_TRANSITION_SCHEMA)
            .expect_err("unsupported transition schema must fail during deserialization");
    assert_eq!(
        transition_schema_error.classify(),
        serde_json::error::Category::Data
    );
    assert!(
        transition_schema_error
            .to_string()
            .contains("schema version 2 is unsupported; expected 1")
    );

    let digest_error = serde_json::from_str::<ArtifactDigest>(INVALID_ARTIFACT_DIGEST)
        .expect_err("upper-case artifact digest must fail during deserialization");
    assert!(
        digest_error
            .to_string()
            .contains("must use lower-case sha256:<64 hex characters>")
    );

    let destination_error =
        serde_json::from_str::<SemanticDestination>(INVALID_SEMANTIC_DESTINATION)
            .expect_err("a semantic destination cannot carry a deployment URL");
    assert!(
        destination_error
            .to_string()
            .contains("unknown field `url`")
    );

    let instance_error = serde_json::from_str::<ModuleInstance>(INVALID_MODULE_INSTANCE)
        .expect_err("nested lifecycle fields must reject unknown fields");
    assert!(
        instance_error
            .to_string()
            .contains("unknown field `paused`")
    );

    let resolution_error =
        serde_json::from_str::<ResourceResolutionV1>(INVALID_RESOURCE_RESOLUTION)
            .expect_err("authorized dimensions cannot remain undisclosed");
    assert!(
        resolution_error.to_string().contains(
            "an authorized resource-resolution envelope must not contain undisclosed state"
        )
    );
}

#[test]
fn invalid_oci_image_reference_fixture_has_one_exact_validation_finding() {
    let mut manifest: ModuleManifest = serde_json::from_str(VALID_MANIFEST).unwrap();
    let invalid_image: OciImageDeclaration =
        serde_json::from_str(INVALID_OCI_IMAGE_REFERENCE).unwrap();
    let DeploymentProfile::TessaraOciV1(deployment) = &mut manifest.deployment;
    deployment.runtime_image = invalid_image;

    let error = manifest
        .validate(&forms_authority())
        .expect_err("URL-shaped OCI image reference must fail validation");
    assert_eq!(
        error.findings,
        vec![ValidationFinding {
            code: "invalid_oci_image_reference".into(),
            path: "deployment.declaration.runtime_image.image_reference".into(),
            message: "OCI image reference must be a non-URL repository reference pinned with '@' to the declared digest".into(),
        }]
    );
}
