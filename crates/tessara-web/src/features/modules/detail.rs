//! Module Management detail peer sections.

use leptos::prelude::*;

use super::directory::{
    NO_MODULE_INSTANCE_LABEL, NO_MODULE_RELEASE_LABEL, TRANSITION_PRESENTATION_LABEL,
};
use super::models::{
    FeatureDeclarationV1, ModuleDetailDimensionV1, ModuleInventoryEntryV1, RouteDeclarationV1,
    TransitionAvailabilityV1,
};

#[component]
pub fn ModuleDetailPeerSections(entry: ModuleInventoryEntryV1) -> impl IntoView {
    let descriptor = entry.descriptor().clone();
    let definition_id = descriptor.reserved_definition_id.clone();
    let descriptor_href = format!("/api/admin/modules/{definition_id}/descriptor");
    let source_digest = entry.source_digest().to_string();
    let findings = entry.findings().to_vec();
    let dimensions = entry.detail_dimensions();
    let features = descriptor.features.clone();
    let contracts = descriptor.provided_contracts.clone();
    let capabilities = descriptor.security_capabilities.clone();
    let dependencies = descriptor.dependencies.clone();
    let resources = descriptor.resource_types.clone();
    let routes = descriptor.routes.clone();
    let availability = descriptor.availability;

    view! {
        <>
            <section class="organization-detail-card" data-module-section="overview" aria-labelledby="module-overview-heading">
                <div class="module-detail__heading">
                    <div>
                        <h2 id="module-overview-heading">"Overview"</h2>
                        <p>{descriptor.description}</p>
                    </div>
                    <span class="status-badge is-info">{TRANSITION_PRESENTATION_LABEL}</span>
                </div>

                {match availability {
                    TransitionAvailabilityV1::Unavailable => Some(view! {
                        <section class="organization-state" role="status">
                            <h3>"Contribution unavailable"</h3>
                            <p>{availability.explanation()}</p>
                        </section>
                    }),
                    TransitionAvailabilityV1::Retired => Some(view! {
                        <section class="organization-state" role="status">
                            <h3>"Contribution retired"</h3>
                            <p>{availability.explanation()}</p>
                        </section>
                    }),
                    TransitionAvailabilityV1::ActiveInProcess => None,
                }}

                <dl class="organization-detail-list">
                    <div>
                        <dt>"Reserved Module Definition"</dt>
                        <dd><code>{definition_id.clone()}</code></dd>
                    </div>
                    <div>
                        <dt>"Availability"</dt>
                        <dd>{availability.label()}</dd>
                    </div>
                    <div>
                        <dt>"Release"</dt>
                        <dd>{NO_MODULE_RELEASE_LABEL}</dd>
                    </div>
                    <div>
                        <dt>"Instance"</dt>
                        <dd>{NO_MODULE_INSTANCE_LABEL}</dd>
                    </div>
                    <div>
                        <dt>"Source digest"</dt>
                        <dd><code>{source_digest}</code></dd>
                    </div>
                    <div>
                        <dt>"Descriptor configuration schema"</dt>
                        <dd>{if descriptor.configuration_schema.is_some() { "Declared" } else { "Not declared" }}</dd>
                    </div>
                </dl>
                <div class="form-actions">
                    <a class="button button--secondary" href=descriptor_href>
                        "View source descriptor (JSON)"
                    </a>
                </div>
            </section>

            <section class="organization-detail-card" data-module-section="overview" aria-labelledby="module-features-heading">
                <h2 id="module-features-heading">"Feature Declarations"</h2>
                {if features.is_empty() {
                    empty_declaration("No feature declarations", "This contribution declares no current features.")
                } else {
                    view! {
                        <div class="module-declaration-stack">
                            {features.into_iter().map(feature_declaration).collect_view()}
                        </div>
                    }.into_any()
                }}
            </section>

            <section class="organization-detail-card" data-module-section="overview" aria-labelledby="module-contracts-heading">
                <h2 id="module-contracts-heading">"Contracts"</h2>
                {if contracts.is_empty() {
                    empty_declaration("No provided contracts", "This contribution provides no functional contracts.")
                } else {
                    view! {
                        <ul class="module-metadata-list">
                            {contracts.into_iter().map(|contract| view! {
                                <li data-contract-id=contract.id.clone()>
                                    <strong><code>{contract.id.clone()}</code></strong>
                                    <span>{format!("{} {}", contract.kind.label(), contract.version)}</span>
                                    <p>{contract.description}</p>
                                </li>
                            }).collect_view()}
                        </ul>
                    }.into_any()
                }}
            </section>

            <section class="organization-detail-card" data-module-section="capabilities" aria-labelledby="module-capabilities-heading">
                <h2 id="module-capabilities-heading">"Capabilities"</h2>
                <p>"Core owns role assignment and capability authorization; this descriptor supplies provenance only."</p>
                {if capabilities.is_empty() {
                    empty_declaration("No contributed capabilities", "This contribution declares no security capabilities.")
                } else {
                    view! {
                        <ul class="module-metadata-list">
                            {capabilities.into_iter().map(|capability| view! {
                                <li data-capability-id=capability.id.clone()>
                                    <strong><code>{capability.id.clone()}</code></strong>
                                    <p>{capability.description}</p>
                                </li>
                            }).collect_view()}
                        </ul>
                    }.into_any()
                }}
            </section>

            <section
                class="organization-detail-card"
                data-module-section="dependencies"
                aria-labelledby="module-dependencies-heading"
                data-module-dimension="dependency"
            >
                <h2 id="module-dependencies-heading">"Dependencies"</h2>
                {dimension_state(dimensions.dependency)}
                <h3>"Declared dependencies"</h3>
                {if dependencies.is_empty() {
                    view! { <p>"No functional dependencies are declared."</p> }.into_any()
                } else {
                    view! {
                        <ul class="module-metadata-list">
                            {dependencies.into_iter().map(|dependency| view! {
                                <li data-dependency-binding=dependency.binding_key.clone()>
                                    <strong><code>{dependency.binding_key.clone()}</code></strong>
                                    <span>
                                        {format!(" requires {} {}", dependency.contract_id, dependency.version_requirement)}
                                    </span>
                                    <span class="data-table__secondary-text">
                                        {if dependency.optional { "Optional" } else { "Required" }}
                                    </span>
                                </li>
                            }).collect_view()}
                        </ul>
                    }.into_any()
                }}
            </section>

            <section
                class="organization-detail-card"
                data-module-section="dependencies"
                aria-labelledby="module-compatibility-heading"
                data-module-dimension="compatibility"
            >
                <h2 id="module-compatibility-heading">"Compatibility"</h2>
                {dimension_state(dimensions.compatibility)}
            </section>

            <section
                class="organization-detail-card"
                data-module-section="dependencies"
                aria-labelledby="module-configuration-heading"
                data-module-dimension="configuration"
            >
                <h2 id="module-configuration-heading">"Configuration"</h2>
                {dimension_state(dimensions.configuration)}
            </section>

            <section
                class="organization-detail-card"
                data-module-section="dependencies"
                aria-labelledby="module-readiness-heading"
                data-module-dimension="readiness"
            >
                <h2 id="module-readiness-heading">"Readiness"</h2>
                {dimension_state(dimensions.readiness)}
            </section>

            <section
                class="organization-detail-card"
                data-module-section="dependencies"
                aria-labelledby="module-health-heading"
                data-module-dimension="health"
            >
                <h2 id="module-health-heading">"Health"</h2>
                {dimension_state(dimensions.health)}
            </section>

            <section class="organization-detail-card" data-module-section="dependencies" aria-labelledby="module-findings-heading">
                <h2 id="module-findings-heading">"Findings"</h2>
                {if findings.is_empty() {
                    view! { <p>"No catalog findings were reported."</p> }.into_any()
                } else {
                    view! {
                        <ul class="module-metadata-list">
                            {findings.into_iter().map(|finding| view! {
                                <li
                                    data-finding-code=finding.code.clone()
                                    data-finding-path=finding.path.clone()
                                >
                                    <strong><code>{finding.code.clone()}</code></strong>
                                    <span class="data-table__secondary-text"><code>{finding.path.clone()}</code></span>
                                    <p>{finding.message}</p>
                                </li>
                            }).collect_view()}
                        </ul>
                    }.into_any()
                }}
            </section>

            <section class="organization-detail-card" data-module-section="resources" aria-labelledby="module-resources-heading">
                <h2 id="module-resources-heading">"Resources/Destinations"</h2>
                <section aria-labelledby="module-resource-types-heading">
                    <h3 id="module-resource-types-heading">"Resource types"</h3>
                    {if resources.is_empty() {
                        view! { <p>"No resource types are declared."</p> }.into_any()
                    } else {
                        view! {
                            <ul class="module-metadata-list">
                                {resources.into_iter().map(|resource| view! {
                                    <li data-resource-type-id=resource.id.clone()>
                                        <strong><code>{resource.id.clone()}</code></strong>
                                        <p>{resource.description}</p>
                                    </li>
                                }).collect_view()}
                            </ul>
                        }.into_any()
                    }}
                </section>
                <section aria-labelledby="module-destinations-heading">
                    <h3 id="module-destinations-heading">"Semantic destinations"</h3>
                    {if routes.is_empty() {
                        view! { <p>"No executable destination is declared."</p> }.into_any()
                    } else {
                        view! {
                            <ul class="module-metadata-list">
                                {routes.into_iter().map(route_declaration).collect_view()}
                            </ul>
                        }.into_any()
                    }}
                </section>
            </section>
        </>
    }
}

fn dimension_state(dimension: ModuleDetailDimensionV1) -> impl IntoView {
    view! {
        <div class="module-detail-dimension-state">
            <strong>{dimension.state.label()}</strong>
            <p>{dimension.evidence}</p>
        </div>
    }
}

fn empty_declaration(title: &'static str, message: &'static str) -> AnyView {
    view! {
        <div class="empty-state">
            <h3>{title}</h3>
            <p>{message}</p>
        </div>
    }
    .into_any()
}

fn feature_declaration(feature: FeatureDeclarationV1) -> impl IntoView {
    view! {
        <article
            class="module-feature-declaration"
            data-feature-id=feature.id.clone()
        >
            <h3>{feature.name}</h3>
            <code>{feature.id.clone()}</code>
            <p>{feature.description}</p>
            {feature_string_list("use_cases", "Use cases", feature.use_cases)}
            {feature_string_list("inputs", "Inputs", feature.inputs)}
            {feature_string_list("outcomes", "Outcomes", feature.outcomes)}
            {feature_string_list("constraints", "Constraints", feature.constraints)}
            {feature_string_list("contracts", "Contracts", feature.contracts)}
            {feature_string_list("resource_types", "Resource types", feature.resource_types)}
            {feature_string_list("destinations", "Destinations", feature.destinations)}
            {feature_string_list("capabilities", "Capabilities", feature.capabilities)}
            {feature_string_list(
                "configuration_pointers",
                "Configuration pointers",
                feature.configuration_pointers,
            )}
        </article>
    }
}

fn feature_string_list(field: &'static str, label: &'static str, values: Vec<String>) -> AnyView {
    if values.is_empty() {
        return ().into_any();
    }
    view! {
        <div class="module-feature-declaration__list" data-feature-field=field>
            <h4>{label}</h4>
            <ul>{values.into_iter().map(|value| view! { <li>{value}</li> }).collect_view()}</ul>
        </div>
    }
    .into_any()
}

fn route_declaration(route: RouteDeclarationV1) -> impl IntoView {
    let parameters = route.parameters;
    let semantic_name = route.name;
    let destination = route.resolved_path.map(|path| {
        view! { <a href=path>"Open resolved destination"</a> }
    });
    view! {
        <li data-destination-name=semantic_name.clone()>
            <strong><code>{semantic_name.clone()}</code></strong>
            <span>{route.kind.label()}</span>
            {if parameters.is_empty() {
                view! { <span class="data-table__secondary-text">"No parameters"</span> }.into_any()
            } else {
                view! {
                    <ul>
                        {parameters.into_iter().map(|parameter| view! {
                            <li data-destination-parameter=parameter.name.clone()>
                                <code>{parameter.name.clone()}</code>
                                {format!(" — {} ({})", parameter.value_type.label(), if parameter.required { "required" } else { "optional" })}
                            </li>
                        }).collect_view()}
                    </ul>
                }.into_any()
            }}
            {destination}
        </li>
    }
}

#[cfg(test)]
mod tests {
    use leptos::prelude::*;
    use serde_json::json;

    use super::ModuleDetailPeerSections;
    use crate::features::modules::directory::{
        NO_MODULE_INSTANCE_LABEL, NO_MODULE_RELEASE_LABEL, TRANSITION_PRESENTATION_LABEL,
    };
    use crate::features::modules::models::{
        ModuleInventoryEntryV1, NOT_APPLICABLE_NO_MODULE_RELEASE_INSTANCE_LABEL,
        TransitionalContributionDescriptorV1,
    };

    fn forms_entry() -> ModuleInventoryEntryV1 {
        let descriptor: TransitionalContributionDescriptorV1 = serde_json::from_str(include_str!(
            "../../../../tessara-module-contract/tests/fixtures/transition-forms-v1.json"
        ))
        .expect("fixture parses");
        serde_json::from_value(json!({
            "kind": "transitional_in_process",
            "descriptor": descriptor,
            "source_digest": "sha256:71bebdd07ff0028cc0da8bbd9707c393bade9951e5cedb265a4b8465d54b493e",
            "resource_owner": {
                "kind": "core_installation",
                "installation_id": "installation-1"
            },
            "provider_eligible": false,
            "supervisor_materializable": false,
            "findings": []
        }))
        .expect("entry parses")
    }

    fn responses_entry() -> ModuleInventoryEntryV1 {
        let descriptor: TransitionalContributionDescriptorV1 = serde_json::from_str(include_str!(
            "../../../../tessara-module-contract/tests/fixtures/transition-responses-v1.json"
        ))
        .expect("fixture parses");
        serde_json::from_value(json!({
            "kind": "transitional_in_process",
            "descriptor": descriptor,
            "source_digest": "sha256:synthetic",
            "resource_owner": {
                "kind": "core_installation",
                "installation_id": "installation-1"
            },
            "provider_eligible": false,
            "supervisor_materializable": false,
            "findings": [
                {
                    "code": "transition_internal_only",
                    "path": "dependencies[0]",
                    "message": "Current in-process coupling only."
                }
            ]
        }))
        .expect("entry parses")
    }

    #[test]
    fn detail_html_contains_every_peer_section_and_exact_transition_language() {
        let html = Owner::new()
            .with(|| view! { <ModuleDetailPeerSections entry=forms_entry()/> }.to_html());

        for heading in [
            "Overview",
            "Feature Declarations",
            "Contracts",
            "Capabilities",
            "Dependencies",
            "Compatibility",
            "Configuration",
            "Readiness",
            "Health",
            "Findings",
            "Resources/Destinations",
        ] {
            assert!(html.contains(heading), "missing {heading}");
        }
        for dimension in [
            "dependency",
            "compatibility",
            "configuration",
            "readiness",
            "health",
        ] {
            assert!(
                html.contains(&format!("data-module-dimension=\"{dimension}\"")),
                "missing {dimension} dimension"
            );
        }
        assert_eq!(
            html.matches(NOT_APPLICABLE_NO_MODULE_RELEASE_INSTANCE_LABEL)
                .count(),
            4
        );
        assert!(html.contains(TRANSITION_PRESENTATION_LABEL));
        assert!(html.contains(NO_MODULE_RELEASE_LABEL));
        assert!(html.contains(NO_MODULE_INSTANCE_LABEL));
        assert!(html.contains("data-feature-id=\"tessara.forms.authoring\""));
        assert!(html.contains("data-feature-field=\"use_cases\""));
        assert!(html.contains("data-contract-id=\"tessara.forms.form\""));
        assert!(html.contains("data-capability-id=\"forms:read\""));
        assert!(html.contains("data-resource-type-id=\"tessara.transition.form\""));
        assert!(html.contains("data-destination-name=\"forms.directory\""));
        assert!(!html.contains("Install module"));
        assert!(!html.contains(">Healthy<"));
        assert!(!html.contains(">Unhealthy<"));
    }

    #[test]
    fn dependency_bearing_transition_is_internal_only_and_findings_remain_separate() {
        let html = Owner::new()
            .with(|| view! { <ModuleDetailPeerSections entry=responses_entry()/> }.to_html());

        assert!(html.contains("Transition-internal only"));
        assert!(html.contains(
            "2 declared relationships describe current in-process coupling and cannot be satisfied by a transition contribution provider."
        ));
        assert!(html.contains("data-dependency-binding=\"tessara.responses.workflow-version\""));
        assert!(html.contains("transition_internal_only"));
        assert!(html.contains("data-finding-code=\"transition_internal_only\""));
        assert!(html.contains("data-finding-path=\"dependencies[0]\""));
        assert_eq!(
            html.matches("data-module-dimension=\"dependency\"").count(),
            1
        );
        assert_eq!(html.matches("id=\"module-findings-heading\"").count(), 1);
    }
}
