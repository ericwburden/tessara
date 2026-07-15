//! Core-owned registry for current same-origin semantic destinations.

use tessara_module_contract::{
    ResourceOwner, RouteDeclaration, RouteKind, RouteParameterDeclaration, RouteParameterType,
    SemanticDestination, SemanticDestinationValidationError, SemanticParameterValue,
    SemanticRouteName,
};
use uuid::Uuid;

use crate::auth::AccountContext;

use super::dto::{
    DestinationResolutionResponseV1, DestinationResolutionStatusV1, MODULE_HTTP_SCHEMA_VERSION_V1,
    PlatformFindingV1,
};

#[derive(Clone, Copy)]
struct ParameterSpec {
    name: &'static str,
    value_type: RouteParameterType,
}

#[derive(Clone, Copy)]
struct RouteSpec {
    name: &'static str,
    parameters: &'static [ParameterSpec],
    capabilities_any_of: &'static [&'static str],
}

const NONE: &[ParameterSpec] = &[];
const FORM_ID: &[ParameterSpec] = &[ParameterSpec {
    name: "form_id",
    value_type: RouteParameterType::Uuid,
}];
const WORKFLOW_ID: &[ParameterSpec] = &[ParameterSpec {
    name: "workflow_id",
    value_type: RouteParameterType::Uuid,
}];
const SUBMISSION_ID: &[ParameterSpec] = &[ParameterSpec {
    name: "submission_id",
    value_type: RouteParameterType::Uuid,
}];
const DATASET_ID: &[ParameterSpec] = &[ParameterSpec {
    name: "dataset_id",
    value_type: RouteParameterType::Uuid,
}];
const DATASET_REVISION_ID: &[ParameterSpec] = &[
    ParameterSpec {
        name: "dataset_id",
        value_type: RouteParameterType::Uuid,
    },
    ParameterSpec {
        name: "revision_id",
        value_type: RouteParameterType::Uuid,
    },
];
const COMPONENT_REF: &[ParameterSpec] = &[ParameterSpec {
    name: "component_ref",
    value_type: RouteParameterType::String,
}];
const DASHBOARD_ID: &[ParameterSpec] = &[ParameterSpec {
    name: "dashboard_id",
    value_type: RouteParameterType::Uuid,
}];

const FORMS_READ: &[&str] = &["forms:read", "forms:manage"];
const FORMS_MANAGE: &[&str] = &["forms:manage"];
const WORKFLOWS_READ: &[&str] = &["workflows:read", "workflows:manage"];
const WORKFLOWS_MANAGE: &[&str] = &["workflows:manage"];
const RESPONSES_READ: &[&str] = &[
    "submissions:read_own",
    "submissions:respond",
    "submissions:manage",
];
const RESPONSES_WRITE: &[&str] = &["submissions:respond", "submissions:manage"];
const DATASETS_READ: &[&str] = &["datasets:read", "datasets:manage"];
const DATASETS_MANAGE: &[&str] = &["datasets:manage"];
const COMPONENTS_READ: &[&str] = &["components:read", "components:manage"];
const COMPONENTS_MANAGE: &[&str] = &["components:manage"];
const DASHBOARDS_READ: &[&str] = &["dashboards:read"];
const DASHBOARDS_MANAGE: &[&str] = &["dashboards:manage"];

/// Resolves only names in the frozen Sprint 6A Core route registry.
pub(crate) fn resolve(
    destination: &SemanticDestination,
    installation_id: Uuid,
    account: &AccountContext,
) -> DestinationResolutionResponseV1 {
    if !matches!(
        &destination.owner,
        ResourceOwner::CoreInstallation {
            installation_id: owner_installation_id
        } if *owner_installation_id == installation_id
    ) {
        return rejected(
            "semantic_destination_owner_mismatch",
            "destination.owner",
            "The destination owner is not the current Core installation.",
        );
    }

    let Some(spec) = route_spec(destination.route.as_str()) else {
        return rejected(
            "semantic_destination_unknown",
            "destination.route",
            "The named semantic destination is not registered.",
        );
    };

    let declaration = route_declaration(spec);
    if let Err(error) = destination.validate_against(&declaration) {
        return invalid_parameters(error);
    }

    if !spec
        .capabilities_any_of
        .iter()
        .any(|capability| account.has_capability(capability))
    {
        return rejected(
            "semantic_destination_unauthorized",
            "destination",
            "The current account is not authorized for the named destination.",
        );
    }

    match render_path(spec.name, &destination.parameters) {
        Some(path) if path.starts_with('/') && !path.starts_with("//") => {
            DestinationResolutionResponseV1 {
                schema_version: MODULE_HTTP_SCHEMA_VERSION_V1,
                status: DestinationResolutionStatusV1::Resolved,
                path: Some(path),
                finding: None,
            }
        }
        _ => rejected(
            "semantic_destination_registry_invalid",
            "destination.route",
            "The registered destination could not be resolved to a same-origin path.",
        ),
    }
}

fn rejected(
    code: &'static str,
    path: &'static str,
    message: &'static str,
) -> DestinationResolutionResponseV1 {
    DestinationResolutionResponseV1 {
        schema_version: MODULE_HTTP_SCHEMA_VERSION_V1,
        status: DestinationResolutionStatusV1::Rejected,
        path: None,
        finding: Some(PlatformFindingV1 {
            code,
            path,
            message,
        }),
    }
}

fn invalid_parameters(
    error: SemanticDestinationValidationError,
) -> DestinationResolutionResponseV1 {
    let (code, message) = match error {
        SemanticDestinationValidationError::RouteMismatch => (
            "semantic_destination_route_mismatch",
            "The destination route does not match its Core declaration.",
        ),
        SemanticDestinationValidationError::DuplicateRouteParameter(_) => (
            "semantic_destination_registry_invalid",
            "The Core route declaration contains a duplicate parameter.",
        ),
        SemanticDestinationValidationError::MissingRequiredParameter(_) => (
            "semantic_destination_parameter_missing",
            "A required destination parameter is missing.",
        ),
        SemanticDestinationValidationError::UnknownParameter(_) => (
            "semantic_destination_parameter_unknown",
            "The destination contains an unknown parameter.",
        ),
        SemanticDestinationValidationError::ParameterTypeMismatch(_) => (
            "semantic_destination_parameter_type_mismatch",
            "A destination parameter has the wrong type.",
        ),
    };
    rejected(code, "destination.parameters", message)
}

fn route_declaration(spec: RouteSpec) -> RouteDeclaration {
    RouteDeclaration {
        name: SemanticRouteName::new(spec.name).expect("static route names are valid"),
        kind: RouteKind::Product,
        parameters: spec
            .parameters
            .iter()
            .map(|parameter| RouteParameterDeclaration {
                name: parameter.name.to_string(),
                value_type: parameter.value_type,
                required: true,
            })
            .collect(),
    }
}

fn route_spec(name: &str) -> Option<RouteSpec> {
    let (parameters, capabilities_any_of) = match name {
        "forms.directory" | "forms.detail" => (FORM_ID_OR_NONE(name), FORMS_READ),
        "forms.create" | "forms.edit" => (FORM_ID_OR_NONE(name), FORMS_MANAGE),
        "workflows.directory" | "workflows.assignments" | "workflows.detail" => {
            (WORKFLOW_ID_OR_NONE(name), WORKFLOWS_READ)
        }
        "workflows.create" | "workflows.edit" => (WORKFLOW_ID_OR_NONE(name), WORKFLOWS_MANAGE),
        "responses.directory" | "responses.detail" => (SUBMISSION_ID_OR_NONE(name), RESPONSES_READ),
        "responses.start" | "responses.edit" => (SUBMISSION_ID_OR_NONE(name), RESPONSES_WRITE),
        "datasets.directory"
        | "datasets.detail"
        | "datasets.preview"
        | "datasets.revisions"
        | "datasets.revision_detail" => (dataset_parameters(name), DATASETS_READ),
        "datasets.create" | "datasets.edit" | "datasets.revision_edit" => {
            (dataset_parameters(name), DATASETS_MANAGE)
        }
        "components.directory"
        | "components.detail"
        | "components.versions"
        | "components.view" => (COMPONENT_REF_OR_NONE(name), COMPONENTS_READ),
        "components.create" | "components.edit" => (COMPONENT_REF_OR_NONE(name), COMPONENTS_MANAGE),
        "dashboards.directory" | "dashboards.detail" | "dashboards.view" => {
            (DASHBOARD_ID_OR_NONE(name), DASHBOARDS_READ)
        }
        "dashboards.create" | "dashboards.edit" => (DASHBOARD_ID_OR_NONE(name), DASHBOARDS_MANAGE),
        _ => return None,
    };
    Some(RouteSpec {
        name: route_name(name),
        parameters,
        capabilities_any_of,
    })
}

// The match above accepts only string literals. Returning the corresponding
// literal keeps the registry borrowed for the whole process without leaking a
// caller-owned route string.
fn route_name(name: &str) -> &'static str {
    match name {
        "forms.directory" => "forms.directory",
        "forms.create" => "forms.create",
        "forms.detail" => "forms.detail",
        "forms.edit" => "forms.edit",
        "workflows.directory" => "workflows.directory",
        "workflows.create" => "workflows.create",
        "workflows.assignments" => "workflows.assignments",
        "workflows.detail" => "workflows.detail",
        "workflows.edit" => "workflows.edit",
        "responses.directory" => "responses.directory",
        "responses.start" => "responses.start",
        "responses.detail" => "responses.detail",
        "responses.edit" => "responses.edit",
        "datasets.directory" => "datasets.directory",
        "datasets.create" => "datasets.create",
        "datasets.detail" => "datasets.detail",
        "datasets.preview" => "datasets.preview",
        "datasets.revisions" => "datasets.revisions",
        "datasets.revision_detail" => "datasets.revision_detail",
        "datasets.revision_edit" => "datasets.revision_edit",
        "datasets.edit" => "datasets.edit",
        "components.directory" => "components.directory",
        "components.create" => "components.create",
        "components.detail" => "components.detail",
        "components.edit" => "components.edit",
        "components.versions" => "components.versions",
        "components.view" => "components.view",
        "dashboards.directory" => "dashboards.directory",
        "dashboards.create" => "dashboards.create",
        "dashboards.detail" => "dashboards.detail",
        "dashboards.edit" => "dashboards.edit",
        "dashboards.view" => "dashboards.view",
        _ => unreachable!("route_name is called only for registered routes"),
    }
}

#[allow(non_snake_case)]
fn FORM_ID_OR_NONE(name: &str) -> &'static [ParameterSpec] {
    if matches!(name, "forms.detail" | "forms.edit") {
        FORM_ID
    } else {
        NONE
    }
}

#[allow(non_snake_case)]
fn WORKFLOW_ID_OR_NONE(name: &str) -> &'static [ParameterSpec] {
    if matches!(name, "workflows.detail" | "workflows.edit") {
        WORKFLOW_ID
    } else {
        NONE
    }
}

#[allow(non_snake_case)]
fn SUBMISSION_ID_OR_NONE(name: &str) -> &'static [ParameterSpec] {
    if matches!(name, "responses.detail" | "responses.edit") {
        SUBMISSION_ID
    } else {
        NONE
    }
}

fn dataset_parameters(name: &str) -> &'static [ParameterSpec] {
    match name {
        "datasets.detail" | "datasets.preview" | "datasets.revisions" | "datasets.edit" => {
            DATASET_ID
        }
        "datasets.revision_detail" | "datasets.revision_edit" => DATASET_REVISION_ID,
        _ => NONE,
    }
}

#[allow(non_snake_case)]
fn COMPONENT_REF_OR_NONE(name: &str) -> &'static [ParameterSpec] {
    if matches!(
        name,
        "components.detail" | "components.edit" | "components.versions" | "components.view"
    ) {
        COMPONENT_REF
    } else {
        NONE
    }
}

#[allow(non_snake_case)]
fn DASHBOARD_ID_OR_NONE(name: &str) -> &'static [ParameterSpec] {
    if matches!(
        name,
        "dashboards.detail" | "dashboards.edit" | "dashboards.view"
    ) {
        DASHBOARD_ID
    } else {
        NONE
    }
}

fn render_path(
    route: &str,
    parameters: &std::collections::BTreeMap<String, SemanticParameterValue>,
) -> Option<String> {
    let uuid = |name: &str| match parameters.get(name) {
        Some(SemanticParameterValue::Uuid(value)) => Some(value.to_string()),
        _ => None,
    };
    let string = |name: &str| match parameters.get(name) {
        Some(SemanticParameterValue::String(value)) => Some(encode_path_segment(value)),
        _ => None,
    };

    Some(match route {
        "forms.directory" => "/forms".to_string(),
        "forms.create" => "/forms/new".to_string(),
        "forms.detail" => format!("/forms/{}", uuid("form_id")?),
        "forms.edit" => format!("/forms/{}/edit", uuid("form_id")?),
        "workflows.directory" => "/workflows".to_string(),
        "workflows.create" => "/workflows/new".to_string(),
        "workflows.assignments" => "/workflows/assignments".to_string(),
        "workflows.detail" => format!("/workflows/{}", uuid("workflow_id")?),
        "workflows.edit" => format!("/workflows/{}/edit", uuid("workflow_id")?),
        "responses.directory" => "/responses".to_string(),
        "responses.start" => "/responses/new".to_string(),
        "responses.detail" => format!("/responses/{}", uuid("submission_id")?),
        "responses.edit" => format!("/responses/{}/edit", uuid("submission_id")?),
        "datasets.directory" => "/datasets".to_string(),
        "datasets.create" => "/datasets/new".to_string(),
        "datasets.detail" => format!("/datasets/{}", uuid("dataset_id")?),
        "datasets.preview" => format!("/datasets/{}/preview", uuid("dataset_id")?),
        "datasets.revisions" => format!("/datasets/{}/revisions", uuid("dataset_id")?),
        "datasets.revision_detail" => format!(
            "/datasets/{}/revisions/{}",
            uuid("dataset_id")?,
            uuid("revision_id")?
        ),
        "datasets.revision_edit" => format!(
            "/datasets/{}/revisions/{}/edit",
            uuid("dataset_id")?,
            uuid("revision_id")?
        ),
        "datasets.edit" => format!("/datasets/{}/edit", uuid("dataset_id")?),
        "components.directory" => "/components".to_string(),
        "components.create" => "/components/new".to_string(),
        "components.detail" => format!("/components/{}", string("component_ref")?),
        "components.edit" => format!("/components/{}/edit", string("component_ref")?),
        "components.versions" => {
            format!("/components/{}/versions", string("component_ref")?)
        }
        "components.view" => format!("/components/{}/view", string("component_ref")?),
        "dashboards.directory" => "/dashboards".to_string(),
        "dashboards.create" => "/dashboards/new".to_string(),
        "dashboards.detail" => format!("/dashboards/{}", uuid("dashboard_id")?),
        "dashboards.edit" => format!("/dashboards/{}/edit", uuid("dashboard_id")?),
        "dashboards.view" => format!("/dashboards/{}/view", uuid("dashboard_id")?),
        _ => return None,
    })
}

fn encode_path_segment(value: &str) -> String {
    let mut encoded = String::with_capacity(value.len());
    for byte in value.bytes() {
        if byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'.' | b'_' | b'~') {
            encoded.push(char::from(byte));
        } else {
            use std::fmt::Write as _;
            write!(&mut encoded, "%{byte:02X}").expect("writing to String cannot fail");
        }
    }
    encoded
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use tessara_module_contract::{
        ResourceOwner, SemanticDestination, SemanticParameterValue, SemanticRouteName,
    };
    use uuid::Uuid;

    use crate::auth::{AccountContext, CapabilityScope};

    use super::{DestinationResolutionStatusV1, resolve};

    #[test]
    fn resolves_registered_path_and_encodes_caller_string_as_one_segment() {
        let installation_id = Uuid::new_v4();
        let destination = SemanticDestination {
            owner: ResourceOwner::CoreInstallation { installation_id },
            route: SemanticRouteName::new("components.detail").expect("route"),
            parameters: BTreeMap::from([(
                "component_ref".to_string(),
                SemanticParameterValue::String("slug/../../admin?x=1".to_string()),
            )]),
        };

        let result = resolve(&destination, installation_id, &account("components:read"));
        assert_eq!(result.status, DestinationResolutionStatusV1::Resolved);
        assert_eq!(
            result.path.as_deref(),
            Some("/components/slug%2F..%2F..%2Fadmin%3Fx%3D1")
        );
    }

    #[test]
    fn unknown_wrong_owner_and_unauthorized_destinations_never_return_a_path() {
        let installation_id = Uuid::new_v4();
        let cases = [
            (
                SemanticDestination {
                    owner: ResourceOwner::CoreInstallation { installation_id },
                    route: SemanticRouteName::new("unknown.route").expect("route"),
                    parameters: BTreeMap::new(),
                },
                account("forms:read"),
                "semantic_destination_unknown",
            ),
            (
                SemanticDestination {
                    owner: ResourceOwner::CoreInstallation {
                        installation_id: Uuid::new_v4(),
                    },
                    route: SemanticRouteName::new("forms.directory").expect("route"),
                    parameters: BTreeMap::new(),
                },
                account("forms:read"),
                "semantic_destination_owner_mismatch",
            ),
            (
                SemanticDestination {
                    owner: ResourceOwner::CoreInstallation { installation_id },
                    route: SemanticRouteName::new("forms.directory").expect("route"),
                    parameters: BTreeMap::new(),
                },
                account("datasets:read"),
                "semantic_destination_unauthorized",
            ),
        ];

        for (destination, account, expected_code) in cases {
            let result = resolve(&destination, installation_id, &account);
            assert_eq!(result.status, DestinationResolutionStatusV1::Rejected);
            assert_eq!(result.path, None);
            assert_eq!(
                result.finding.as_ref().map(|finding| finding.code),
                Some(expected_code)
            );
        }
    }

    #[test]
    fn dashboard_manage_does_not_resolve_reader_destination() {
        let installation_id = Uuid::new_v4();
        let destination = SemanticDestination {
            owner: ResourceOwner::CoreInstallation { installation_id },
            route: SemanticRouteName::new("dashboards.directory").expect("route"),
            parameters: BTreeMap::new(),
        };

        let result = resolve(&destination, installation_id, &account("dashboards:manage"));
        assert_eq!(result.status, DestinationResolutionStatusV1::Rejected);
        assert_eq!(
            result.finding.as_ref().map(|finding| finding.code),
            Some("semantic_destination_unauthorized")
        );
    }

    #[test]
    fn frozen_registry_resolves_every_declared_route_to_its_exact_core_path() {
        let installation_id =
            Uuid::parse_str("aaaaaaaa-aaaa-4aaa-8aaa-aaaaaaaaaaaa").expect("fixed installation id");
        let resource_id =
            Uuid::parse_str("bbbbbbbb-bbbb-4bbb-8bbb-bbbbbbbbbbbb").expect("fixed resource id");
        let cases = [
            ("forms.directory", "/forms"),
            ("forms.create", "/forms/new"),
            (
                "forms.detail",
                "/forms/bbbbbbbb-bbbb-4bbb-8bbb-bbbbbbbbbbbb",
            ),
            (
                "forms.edit",
                "/forms/bbbbbbbb-bbbb-4bbb-8bbb-bbbbbbbbbbbb/edit",
            ),
            ("workflows.directory", "/workflows"),
            ("workflows.create", "/workflows/new"),
            ("workflows.assignments", "/workflows/assignments"),
            (
                "workflows.detail",
                "/workflows/bbbbbbbb-bbbb-4bbb-8bbb-bbbbbbbbbbbb",
            ),
            (
                "workflows.edit",
                "/workflows/bbbbbbbb-bbbb-4bbb-8bbb-bbbbbbbbbbbb/edit",
            ),
            ("responses.directory", "/responses"),
            ("responses.start", "/responses/new"),
            (
                "responses.detail",
                "/responses/bbbbbbbb-bbbb-4bbb-8bbb-bbbbbbbbbbbb",
            ),
            (
                "responses.edit",
                "/responses/bbbbbbbb-bbbb-4bbb-8bbb-bbbbbbbbbbbb/edit",
            ),
            ("datasets.directory", "/datasets"),
            ("datasets.create", "/datasets/new"),
            (
                "datasets.detail",
                "/datasets/bbbbbbbb-bbbb-4bbb-8bbb-bbbbbbbbbbbb",
            ),
            (
                "datasets.preview",
                "/datasets/bbbbbbbb-bbbb-4bbb-8bbb-bbbbbbbbbbbb/preview",
            ),
            (
                "datasets.revisions",
                "/datasets/bbbbbbbb-bbbb-4bbb-8bbb-bbbbbbbbbbbb/revisions",
            ),
            (
                "datasets.revision_detail",
                "/datasets/bbbbbbbb-bbbb-4bbb-8bbb-bbbbbbbbbbbb/revisions/bbbbbbbb-bbbb-4bbb-8bbb-bbbbbbbbbbbb",
            ),
            (
                "datasets.revision_edit",
                "/datasets/bbbbbbbb-bbbb-4bbb-8bbb-bbbbbbbbbbbb/revisions/bbbbbbbb-bbbb-4bbb-8bbb-bbbbbbbbbbbb/edit",
            ),
            (
                "datasets.edit",
                "/datasets/bbbbbbbb-bbbb-4bbb-8bbb-bbbbbbbbbbbb/edit",
            ),
            ("components.directory", "/components"),
            ("components.create", "/components/new"),
            ("components.detail", "/components/component-slug"),
            ("components.edit", "/components/component-slug/edit"),
            ("components.versions", "/components/component-slug/versions"),
            ("components.view", "/components/component-slug/view"),
            ("dashboards.directory", "/dashboards"),
            ("dashboards.create", "/dashboards/new"),
            (
                "dashboards.detail",
                "/dashboards/bbbbbbbb-bbbb-4bbb-8bbb-bbbbbbbbbbbb",
            ),
            (
                "dashboards.edit",
                "/dashboards/bbbbbbbb-bbbb-4bbb-8bbb-bbbbbbbbbbbb/edit",
            ),
            (
                "dashboards.view",
                "/dashboards/bbbbbbbb-bbbb-4bbb-8bbb-bbbbbbbbbbbb/view",
            ),
        ];

        for (route, expected) in cases {
            let parameters = match route {
                "forms.detail" | "forms.edit" => BTreeMap::from([(
                    "form_id".to_string(),
                    SemanticParameterValue::Uuid(resource_id),
                )]),
                "workflows.detail" | "workflows.edit" => BTreeMap::from([(
                    "workflow_id".to_string(),
                    SemanticParameterValue::Uuid(resource_id),
                )]),
                "responses.detail" | "responses.edit" => BTreeMap::from([(
                    "submission_id".to_string(),
                    SemanticParameterValue::Uuid(resource_id),
                )]),
                "datasets.detail" | "datasets.preview" | "datasets.revisions" | "datasets.edit" => {
                    BTreeMap::from([(
                        "dataset_id".to_string(),
                        SemanticParameterValue::Uuid(resource_id),
                    )])
                }
                "datasets.revision_detail" | "datasets.revision_edit" => BTreeMap::from([
                    (
                        "dataset_id".to_string(),
                        SemanticParameterValue::Uuid(resource_id),
                    ),
                    (
                        "revision_id".to_string(),
                        SemanticParameterValue::Uuid(resource_id),
                    ),
                ]),
                "components.detail"
                | "components.edit"
                | "components.versions"
                | "components.view" => BTreeMap::from([(
                    "component_ref".to_string(),
                    SemanticParameterValue::String("component-slug".to_string()),
                )]),
                "dashboards.detail" | "dashboards.edit" | "dashboards.view" => BTreeMap::from([(
                    "dashboard_id".to_string(),
                    SemanticParameterValue::Uuid(resource_id),
                )]),
                _ => BTreeMap::new(),
            };
            let destination = SemanticDestination {
                owner: ResourceOwner::CoreInstallation { installation_id },
                route: SemanticRouteName::new(route).expect("registered route name"),
                parameters,
            };

            let result = resolve(&destination, installation_id, &account("admin:all"));
            assert_eq!(
                result.path.as_deref(),
                Some(expected),
                "semantic route {route}"
            );
        }
    }

    fn account(capability: &str) -> AccountContext {
        AccountContext {
            account_id: Uuid::nil(),
            email: "destination@example.test".to_string(),
            display_name: "Destination".to_string(),
            is_active: true,
            roles: Vec::new(),
            capabilities: vec![capability.to_string()],
            capability_scopes: vec![CapabilityScope {
                capability: capability.to_string(),
                global: true,
                node_ids: Vec::new(),
            }],
            scope_nodes: Vec::new(),
            delegations: Vec::new(),
        }
    }
}
