//! Module Management detail peer sections.

use icons::{ChevronRight, CircleAlert, CircleCheck, Eye, Info, Settings};
use leptos::prelude::*;
use tessara_web_ui::ModalDialog;

use super::directory::{
    CopyValue, NO_MODULE_INSTANCE_LABEL, NO_MODULE_RELEASE_LABEL, compact_value,
};
use super::models::{
    FeatureDeclarationV1, ModuleDetailDimensionV1, ModuleInventoryEntryV1,
    NavigationContributionDeclarationV1, NavigationPolicyResponseV2, RouteDeclarationV1,
    TransitionAvailabilityV1,
};

#[component]
pub fn ModuleDetailPeerSections(
    entry: ModuleInventoryEntryV1,
    policy: RwSignal<Option<NavigationPolicyResponseV2>>,
    active_detail_section: RwSignal<&'static str>,
) -> impl IntoView {
    let descriptor = entry.descriptor().clone();
    let definition_id = descriptor.reserved_definition_id.clone();
    let source_digest = entry.source_digest().to_string();
    let findings = entry.findings().to_vec();
    let dimensions = entry.detail_dimensions();
    let features = descriptor.features.clone();
    let contracts = descriptor.provided_contracts.clone();
    let capabilities = descriptor.security_capabilities.clone();
    let dependencies = descriptor.dependencies.clone();
    let resources = descriptor.resource_types.clone();
    let routes = descriptor.routes.clone();
    let declared_navigation = descriptor.navigation.clone();
    let availability = descriptor.availability;
    let source_digest_preview = compact_value(&source_digest);
    let source_digest_reveal_open = RwSignal::new(false);
    let close_source_digest_reveal = Callback::new(move |_| source_digest_reveal_open.set(false));
    let definition_id_for_navigation = definition_id.clone();
    let overview_findings = findings.clone();
    let findings_for_dependencies = findings.clone();
    let declaration_summary = [
        ("Features", descriptor.features.len()),
        ("Contracts", descriptor.provided_contracts.len()),
        ("Capabilities", descriptor.security_capabilities.len()),
        ("Resources", descriptor.resource_types.len()),
        ("Destinations", descriptor.routes.len()),
    ];

    view! {
        <>
            <div class="module-detail-overview" data-module-section="overview">
                {if availability == TransitionAvailabilityV1::Retired {
                    view! {
                        <section class="organization-state is-error module-detail-overview__retired" role="status" aria-labelledby="module-retired-heading">
                            <CircleAlert class="module-detail-overview__retired-icon"/>
                            <div>
                                <h2 id="module-retired-heading">"Contribution retired"</h2>
                                <p>{availability.explanation()}</p>
                            </div>
                        </section>
                    }.into_any()
                } else {
                    ().into_any()
                }}

                <section class="organization-detail-card module-detail-overview__definition" aria-labelledby="module-overview-heading">
                    <div class="module-detail__heading">
                        <div>
                            <h2 id="module-overview-heading">"Definition"</h2>
                            <p>{descriptor.description}</p>
                        </div>
                    </div>
                    <dl class="organization-detail-list module-detail-overview__list">
                        <div>
                            <dt>"Definition ID"</dt>
                            <dd class="module-detail-overview__value-with-action">
                                <code>{definition_id.clone()}</code>
                                <CopyValue value=definition_id.clone() label="Copy module definition ID"/>
                            </dd>
                        </div>
                        <div>
                            <dt>"Source digest"</dt>
                            <dd class="module-detail-overview__value-with-action">
                                <code>{source_digest_preview}</code>
                                <CopyValue value=source_digest.clone() label="Copy complete source digest"/>
                                <button
                                    class="icon-button module-directory__copy module-detail-digest__reveal"
                                    type="button"
                                    aria-label="View complete source digest"
                                    title="View complete source digest"
                                    on:click=move |_| source_digest_reveal_open.set(true)
                                >
                                    <Eye/>
                                </button>
                            </dd>
                        </div>
                        <div>
                            <dt>"Descriptor configuration schema"</dt>
                            <dd>{if descriptor.configuration_schema.is_some() { "Declared" } else { "Not declared" }}</dd>
                        </div>
                        <div>
                            <dt>"Module Release"</dt>
                            <dd>{NO_MODULE_RELEASE_LABEL}</dd>
                        </div>
                        <div>
                            <dt>"Module Instance"</dt>
                            <dd>{NO_MODULE_INSTANCE_LABEL}</dd>
                        </div>
                    </dl>
                </section>

                <ModalDialog
                    id="module-source-digest"
                    title="Source digest"
                    description="Complete source digest for this module definition."
                    open=Signal::derive(move || source_digest_reveal_open.get())
                    on_close=close_source_digest_reveal
                    close_label="Close source digest"
                    class="module-detail-digest-dialog"
                >
                    <code>{source_digest.clone()}</code>
                </ModalDialog>

                <section class="organization-detail-card module-detail-overview__lifecycle" aria-labelledby="module-lifecycle-assessment-heading">
                    <h2 id="module-lifecycle-assessment-heading">"Lifecycle assessment"</h2>
                    <dl class="module-detail-overview__assessment-list">
                        {overview_dimension("Dependencies", dimensions.dependency.clone())}
                        {overview_dimension("Compatibility", dimensions.compatibility.clone())}
                        {overview_dimension("Configuration", dimensions.configuration.clone())}
                        {overview_dimension("Readiness", dimensions.readiness.clone())}
                        {overview_dimension("Health", dimensions.health.clone())}
                    </dl>
                </section>

                <section class="organization-detail-card" aria-labelledby="module-declaration-summary-heading">
                    <h2 id="module-declaration-summary-heading">"Declaration summary"</h2>
                    <dl class="module-detail-overview__summary-list">
                        {declaration_summary.into_iter().map(|(label, count)| view! {
                            <div>
                                <dt>{label}</dt>
                                <dd>{count}</dd>
                            </div>
                        }).collect_view()}
                    </dl>
                </section>

                <section class="organization-detail-card" aria-labelledby="module-current-navigation-heading">
                    <h2 id="module-current-navigation-heading">"Current navigation"</h2>
                    {move || overview_navigation_summary(
                        &definition_id_for_navigation,
                        declared_navigation.clone(),
                        policy.get_untracked(),
                    )}
                    <button
                        class="module-detail-overview__navigation-action"
                        type="button"
                        on:click=move |_| active_detail_section.set("navigation")
                    >
                        <Settings class="module-detail-overview__navigation-action-icon"/>
                        "Configure navigation"
                    </button>
                </section>

                {if overview_findings.is_empty() {
                    view! {
                        <section class="organization-detail-card module-detail-overview__findings" aria-live="polite">
                            <p class="module-detail-overview__findings-message">
                                <CircleCheck class="module-detail-overview__findings-icon"/>
                                <span>"No catalog findings were reported."</span>
                            </p>
                        </section>
                    }.into_any()
                } else if availability == TransitionAvailabilityV1::Retired {
                    view! {
                        <section class="organization-detail-card module-detail-overview__catalog-finding" aria-labelledby="module-catalog-finding-heading">
                            <h2 id="module-catalog-finding-heading">"Catalog finding"</h2>
                            <ul class="module-detail-overview__catalog-finding-list">
                                {overview_findings.into_iter().map(|finding| {
                                    let code = finding.code;
                                    let path = finding.path;
                                    let message = finding.message;
                                    view! {
                                        <li data-finding-code=code.clone() data-finding-path=path.clone()>
                                            <CircleAlert class="module-detail-overview__catalog-finding-icon"/>
                                            <dl>
                                                <div><dt>"Code"</dt><dd><code>{code.clone()}</code></dd></div>
                                                <div><dt>"Path"</dt><dd><code>{path.clone()}</code></dd></div>
                                                <div><dt>"Message"</dt><dd>{message}</dd></div>
                                            </dl>
                                        </li>
                                    }
                                }).collect_view()}
                            </ul>
                        </section>
                    }.into_any()
                } else {
                    view! {
                        <section class="organization-detail-card module-detail-overview__findings" aria-live="polite">
                            <p>{format!("{} catalog finding(s) were reported.", overview_findings.len())}</p>
                        </section>
                    }.into_any()
                }}

                {match availability {
                    TransitionAvailabilityV1::Unavailable => Some(view! {
                        <section class="organization-state" role="status">
                            <h3>"Contribution unavailable"</h3>
                            <p>{availability.explanation()}</p>
                        </section>
                    }),
                    TransitionAvailabilityV1::Retired | TransitionAvailabilityV1::ActiveInProcess => None,
                }}
            </div>

            <section
                class="organization-detail-card module-detail-empty-section"
                data-module-section="configuration"
                aria-labelledby="module-configuration-heading"
            >
                <h2 id="module-configuration-heading">"Configuration"</h2>
                <p>"No Module Instance configuration exists for this transitional contribution."</p>
            </section>

            <section class="organization-detail-card" data-module-section="declarations" aria-labelledby="module-features-heading">
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

            <section class="organization-detail-card" data-module-section="contracts" aria-labelledby="module-contracts-heading">
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

            {if dependencies.is_empty() {
                view! {
                    <>
                        <section
                            class="organization-detail-card"
                            data-module-section="dependencies"
                            aria-labelledby="module-dependencies-heading"
                            data-module-dimension="dependency"
                        >
                            <h2 id="module-dependencies-heading">"Dependencies"</h2>
                            {dimension_state(dimensions.dependency.clone())}
                            <h3>"Declared dependencies"</h3>
                            <p>"No functional dependencies are declared."</p>
                        </section>

                        <section
                            class="organization-detail-card"
                            data-module-section="dependencies"
                            aria-labelledby="module-compatibility-heading"
                            data-module-dimension="compatibility"
                        >
                            <h2 id="module-compatibility-heading">"Compatibility"</h2>
                            {dimension_state(dimensions.compatibility.clone())}
                        </section>
                    </>
                }.into_any()
            } else {
                view! {
                    <div class="module-detail-dependencies" data-module-section="dependencies">
                        <section class="organization-detail-card module-detail-dependencies__assessment" aria-labelledby="module-dependency-assessment-heading">
                            <div class="module-detail-dependencies__assessment-heading">
                                <Info class="module-detail-dependencies__info-icon"/>
                                <h2 id="module-dependency-assessment-heading">"Dependency assessment"</h2>
                                <span class="status-badge is-info">{dimensions.dependency.state.label()}</span>
                            </div>
                            <p>{dimensions.dependency.evidence.clone()}</p>
                        </section>

                        <section class="organization-detail-card module-detail-dependencies__declared" aria-labelledby="module-declared-dependencies-heading">
                            <h2 id="module-declared-dependencies-heading">"Declared dependencies"</h2>
                            <div class="table-wrap">
                                <table class="data-table module-detail-dependencies__table">
                                    <thead>
                                        <tr>
                                            <th scope="col">"Binding key"</th>
                                            <th scope="col">"Required contract"</th>
                                            <th scope="col">"Version"</th>
                                            <th scope="col">"Requirement"</th>
                                        </tr>
                                    </thead>
                                    <tbody>
                                        {dependencies.into_iter().map(|dependency| {
                                            let binding_key = dependency.binding_key.clone();
                                            let contract_id = dependency.contract_id.clone();
                                            let version_requirement = dependency.version_requirement.clone();
                                            view! {
                                                <tr data-dependency-binding=binding_key.clone()>
                                                    <th scope="row">
                                                        <span class="module-detail-dependencies__value-with-copy">
                                                            <code>{binding_key.clone()}</code>
                                                            <CopyValue value=binding_key.clone() label="Copy dependency binding key"/>
                                                        </span>
                                                    </th>
                                                    <td>
                                                        <span class="module-detail-dependencies__value-with-copy">
                                                            <code>{contract_id.clone()}</code>
                                                            <CopyValue value=contract_id label="Copy required contract ID"/>
                                                        </span>
                                                    </td>
                                                    <td><code>{version_requirement}</code></td>
                                                    <td>{if dependency.optional { "Optional" } else { "Required" }}</td>
                                                </tr>
                                            }
                                        }).collect_view()}
                                    </tbody>
                                </table>
                            </div>
                        </section>

                        <section class="organization-detail-card module-detail-dependencies__findings" aria-labelledby="module-dependency-findings-heading">
                            <h2 id="module-dependency-findings-heading">"Catalog findings"</h2>
                            {if findings_for_dependencies.is_empty() {
                                view! { <p>"No catalog findings were reported."</p> }.into_any()
                            } else {
                                view! {
                                    <ul class="module-detail-dependencies__findings-list">
                                        {findings_for_dependencies.into_iter().map(|finding| {
                                            let code = finding.code;
                                            let path = finding.path;
                                            let message = finding.message;
                                            view! {
                                                <li data-finding-code=code.clone() data-finding-path=path.clone()>
                                                    <span class="status-badge is-danger module-detail-dependencies__finding-code">{code.clone()}</span>
                                                    <code>{path.clone()}</code>
                                                    <p>{message}</p>
                                                </li>
                                            }
                                        }).collect_view()}
                                    </ul>
                                }.into_any()
                            }}
                        </section>

                        <section class="organization-detail-card module-detail-dependencies__runtime" aria-label="Release and instance status">
                            <Info class="module-detail-dependencies__info-icon"/>
                            <span>{NO_MODULE_RELEASE_LABEL}</span>
                            <span class="module-detail-dependencies__runtime-separator" aria-hidden="true"><ChevronRight/></span>
                            <span>{NO_MODULE_INSTANCE_LABEL}</span>
                            <span class="module-detail-dependencies__runtime-separator" aria-hidden="true"><ChevronRight/></span>
                            <span>{dimensions.compatibility.state.label()}</span>
                        </section>
                    </div>
                }.into_any()
            }}

            <section
                class="organization-detail-card"
                data-module-section="findings"
                aria-labelledby="module-configuration-heading"
                data-module-dimension="configuration"
            >
                <h2 id="module-configuration-heading">"Configuration"</h2>
                {dimension_state(dimensions.configuration)}
            </section>

            <section
                class="organization-detail-card"
                data-module-section="findings"
                aria-labelledby="module-readiness-heading"
                data-module-dimension="readiness"
            >
                <h2 id="module-readiness-heading">"Readiness"</h2>
                {dimension_state(dimensions.readiness)}
            </section>

            <section
                class="organization-detail-card"
                data-module-section="findings"
                aria-labelledby="module-health-heading"
                data-module-dimension="health"
            >
                <h2 id="module-health-heading">"Health"</h2>
                {dimension_state(dimensions.health)}
            </section>

            <section class="organization-detail-card" data-module-section="findings" aria-labelledby="module-findings-heading">
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

fn overview_dimension(label: &'static str, dimension: ModuleDetailDimensionV1) -> impl IntoView {
    view! {
        <div>
            <dt>{label}</dt>
            <dd>{dimension.state.label()}</dd>
        </div>
    }
}

fn overview_navigation_summary(
    definition_id: &str,
    declared_navigation: Vec<NavigationContributionDeclarationV1>,
    policy: Option<NavigationPolicyResponseV2>,
) -> AnyView {
    let declared = declared_navigation.into_iter().next();
    let policy_destination = policy.as_ref().and_then(|current| {
        current
            .destinations
            .iter()
            .find(|destination| destination.definition_id.as_deref() == Some(definition_id))
            .map(|destination| {
                let group_label = current
                    .groups
                    .iter()
                    .find(|group| group.id == destination.group_id)
                    .map(|group| group.label.clone())
                    .unwrap_or_else(|| destination.group_id.clone());
                let mut group_destinations = current
                    .destinations
                    .iter()
                    .filter(|candidate| candidate.group_id == destination.group_id)
                    .collect::<Vec<_>>();
                group_destinations.sort_by_key(|candidate| candidate.order);
                let placement = group_destinations
                    .iter()
                    .position(|candidate| candidate.id == destination.id)
                    .and_then(|index| {
                        index
                            .checked_sub(1)
                            .map(|prior| group_destinations[prior].label.clone())
                    })
                    .map(|prior_label| format!("{group_label} · after {prior_label}"))
                    .unwrap_or_else(|| format!("{group_label} · first"));
                (
                    placement,
                    destination.visible,
                    destination
                        .semantic_destination
                        .clone()
                        .unwrap_or_else(|| destination.key.clone()),
                )
            })
    });

    let (placement, visible, destination) = match (policy_destination, declared) {
        (Some((placement, visible, destination)), _) => (placement, visible, destination),
        (None, Some(declaration)) => (
            format!(
                "{} · declared order {}",
                declaration.group, declaration.order_hint
            ),
            true,
            declaration.destination,
        ),
        (None, None) => (
            "No current navigation placement".into(),
            false,
            "No destination declared".into(),
        ),
    };

    view! {
        <dl class="module-detail-overview__navigation-list">
            <div>
                <dt>"Placement"</dt>
                <dd>{placement}</dd>
            </div>
            <div>
                <dt>"Visibility"</dt>
                <dd>{if visible { "Shown" } else { "Hidden" }}</dd>
            </div>
            <div>
                <dt>"Destination"</dt>
                <dd><code>{destination}</code></dd>
            </div>
        </dl>
    }
    .into_any()
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
    use crate::features::modules::directory::{NO_MODULE_INSTANCE_LABEL, NO_MODULE_RELEASE_LABEL};
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

    fn migration_entry() -> ModuleInventoryEntryV1 {
        let descriptor: TransitionalContributionDescriptorV1 = serde_json::from_str(include_str!(
            "../../../../tessara-module-contract/tests/fixtures/transition-migration-v1.json"
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
                    "code": "transition_destination_retired",
                    "path": "availability",
                    "message": "The former transition destination was deliberately withdrawn and has no live route."
                }
            ]
        }))
        .expect("entry parses")
    }

    #[test]
    fn detail_html_contains_every_peer_section_and_exact_transition_language() {
        let html = Owner::new().with(|| {
            let policy = RwSignal::new(None);
            let active_detail_section = RwSignal::new("overview");
            view! {
                <ModuleDetailPeerSections entry=forms_entry() policy active_detail_section/>
            }
            .to_html()
        });

        for heading in [
            "Definition",
            "Lifecycle assessment",
            "Declaration summary",
            "Current navigation",
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
            8
        );
        assert!(html.contains(NO_MODULE_RELEASE_LABEL));
        assert!(html.contains(NO_MODULE_INSTANCE_LABEL));
        assert!(html.contains("data-feature-id=\"tessara.forms.authoring\""));
        assert!(html.contains("data-feature-field=\"use_cases\""));
        assert!(html.contains("data-contract-id=\"tessara.forms.form\""));
        assert!(html.contains("data-capability-id=\"forms:read\""));
        assert!(html.contains("data-resource-type-id=\"tessara.transition.form\""));
        assert!(html.contains("data-destination-name=\"forms.directory\""));
        assert!(html.contains("Declaration summary"));
        assert!(html.contains("Current navigation"));
        assert!(html.contains("Configure navigation"));
        assert!(html.contains("Copy complete source digest"));
        assert!(html.contains("View complete source digest"));
        assert!(!html.contains("Install module"));
        assert!(!html.contains(">Healthy<"));
        assert!(!html.contains(">Unhealthy<"));
    }

    #[test]
    fn dependency_bearing_transition_is_internal_only_and_findings_remain_separate() {
        let html = Owner::new().with(|| {
            let policy = RwSignal::new(None);
            let active_detail_section = RwSignal::new("overview");
            view! {
                <ModuleDetailPeerSections entry=responses_entry() policy active_detail_section/>
            }
            .to_html()
        });

        assert!(html.contains("Transition-internal only"));
        assert!(html.contains(
            "2 declared relationships describe current in-process coupling and cannot be satisfied by a transition contribution provider."
        ));
        assert!(html.contains("data-dependency-binding=\"tessara.responses.workflow-version\""));
        assert!(html.contains("transition_internal_only"));
        assert!(html.contains("data-finding-code=\"transition_internal_only\""));
        assert!(html.contains("data-finding-path=\"dependencies[0]\""));
        assert_eq!(
            html.matches("data-module-section=\"dependencies\"").count(),
            1
        );
        assert!(html.contains("Dependency assessment"));
        assert!(html.contains("Declared dependencies"));
        assert!(html.contains("Catalog findings"));
        assert!(html.contains("Copy dependency binding key"));
        assert!(html.contains("Copy required contract ID"));
        assert_eq!(html.matches("id=\"module-findings-heading\"").count(), 1);
    }

    #[test]
    fn retired_transition_uses_the_overview_retirement_and_catalog_finding_panels() {
        let html = Owner::new().with(|| {
            let policy = RwSignal::new(None);
            let active_detail_section = RwSignal::new("overview");
            view! {
                <ModuleDetailPeerSections entry=migration_entry() policy active_detail_section/>
            }
            .to_html()
        });

        assert!(html.contains("module-detail-overview__retired"));
        assert!(html.contains("Contribution retired"));
        assert!(html.contains("module-detail-overview__catalog-finding"));
        assert!(html.contains("transition_destination_retired"));
        assert!(html.contains("availability"));
        assert!(html.contains("Lifecycle assessment"));
        assert!(
            html.find("Contribution retired").expect("retirement panel")
                < html.find("Definition").expect("definition panel")
        );
    }
}
