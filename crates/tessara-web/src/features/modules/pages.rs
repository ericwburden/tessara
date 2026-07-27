//! Native Module Management route pages.

use icons::{
    Activity, Database, Download, ExternalLink, HeartPulse, Pencil, RefreshCw, ShieldCheck,
};
use leptos::prelude::*;
use leptos_router::hooks::use_params;
use leptos_router::params::{Params, ParamsError, ParamsMap};
use serde_json::Value;

#[cfg(all(feature = "hydrate", target_arch = "wasm32"))]
use super::api::{ModuleManagementClientError, fetch_module_detail, fetch_module_directory};
use super::bootstrap::{
    ModuleManagementRouteBootstrapV1, NavigationPolicyBootstrapV1,
    module_management_route_bootstrap,
};
use super::deployment::DeploymentLedger;
use super::detail::ModuleDetailPeerSections;
use super::directory::ModuleInventoryDirectory;
use super::models::{
    ModuleDetailPresentationV1, ModuleDetailResponseV1, ModuleDetailViewModelV1,
    ModuleInventoryEntryV1, ModuleInventoryResponseV1, ModuleManagementAccessV1, ModuleManifestV1,
    NavigationPolicyResponseV2,
};
use super::policy::ModuleNavigationPolicyView;
#[cfg(all(feature = "hydrate", target_arch = "wasm32"))]
use crate::state::session::{refresh_shell_navigation, shell_navigation_state};
use crate::ui::{
    AppShell, Breadcrumb, BreadcrumbItem, BreadcrumbLink, BreadcrumbPage, BreadcrumbSeparator,
    DropdownMenu, PageHeader,
};

#[derive(Clone, Debug, Eq, PartialEq)]
enum ModuleRouteState {
    Loading,
    Ready,
    Restricted,
    NotFound,
    Unavailable(String),
    Error(String),
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct ModuleDefinitionRouteParams {
    definition_id: String,
}

impl Params for ModuleDefinitionRouteParams {
    fn from_map(map: &ParamsMap) -> Result<Self, ParamsError> {
        Ok(Self {
            definition_id: map
                .get("definition_id")
                .ok_or_else(|| ParamsError::MissingParam("definition_id".into()))?,
        })
    }
}

#[component]
pub fn ModuleManagementDirectoryPage() -> impl IntoView {
    let inventory = RwSignal::new(None::<ModuleInventoryResponseV1>);
    let access = RwSignal::new(ModuleManagementAccessV1::restricted());
    let policy = RwSignal::new(None::<NavigationPolicyResponseV2>);
    let persisted_policy = RwSignal::new(None::<NavigationPolicyResponseV2>);
    let policy_unavailable = RwSignal::new(None::<String>);
    let initial = initialize_directory(
        module_management_route_bootstrap(),
        inventory,
        access,
        policy,
        persisted_policy,
        policy_unavailable,
    );
    let route_state = RwSignal::new(initial);
    let active_section = RwSignal::new("modules");

    #[cfg(all(feature = "hydrate", target_arch = "wasm32"))]
    {
        let shell_navigation = shell_navigation_state();
        Effect::new(move |_| refresh_shell_navigation(shell_navigation));
    }

    #[cfg(all(feature = "hydrate", target_arch = "wasm32"))]
    Effect::new(move |_| {
        if let Some(window) = web_sys::window()
            && let Ok(hash) = window.location().hash()
        {
            match hash.as_str() {
                "#navigation" => active_section.set("navigation"),
                "#deployment" => active_section.set("deployment"),
                _ => {}
            }
        }
    });

    #[cfg(all(feature = "hydrate", target_arch = "wasm32"))]
    if route_state.get_untracked() == ModuleRouteState::Loading {
        leptos::task::spawn_local(async move {
            match fetch_module_directory().await {
                Ok(payload) => {
                    access.set(payload.access);
                    inventory.set(Some(payload.inventory));
                    set_policy_bootstrap(
                        payload.navigation_policy,
                        policy,
                        persisted_policy,
                        policy_unavailable,
                    );
                    route_state.set(ModuleRouteState::Ready);
                }
                Err(error) => apply_client_error(error, route_state),
            }
        });
    }

    view! {
        <AppShell active_route="module_management" title="Module Management">
            <section class="route-panel module-management-page">
                <Breadcrumb>
                    <BreadcrumbItem><BreadcrumbLink href="/">"Home"</BreadcrumbLink></BreadcrumbItem>
                    <BreadcrumbSeparator/>
                    <BreadcrumbItem><BreadcrumbPage>"Module Management"</BreadcrumbPage></BreadcrumbItem>
                </Breadcrumb>

                <PageHeader
                    title="Module Management"
                    description="Inspect Core runtime context, transition contributions, exact descriptors, and navigation display policy."
                />

                <div class="module-section-switcher" aria-label="Module Management section">
                    <div class="module-section-tabs--directory tabs-list" role="tablist" aria-label="Module Management sections">
                        <button
                            class="tabs-trigger"
                            type="button"
                            role="tab"
                            aria-selected=move || active_section.get() == "modules"
                            class:is-active=move || active_section.get() == "modules"
                            on:click=move |_| active_section.set("modules")
                        >"Modules"</button>
                        <button
                            class="tabs-trigger"
                            type="button"
                            role="tab"
                            aria-selected=move || active_section.get() == "navigation"
                            class:is-active=move || active_section.get() == "navigation"
                            on:click=move |_| active_section.set("navigation")
                        >"Navigation"</button>
                        <button
                            class="tabs-trigger"
                            type="button"
                            role="tab"
                            aria-selected=move || active_section.get() == "deployment"
                            class:is-active=move || active_section.get() == "deployment"
                            on:click=move |_| active_section.set("deployment")
                        >"Deployment"</button>
                    </div>
                    <label class="module-section-select module-section-select--directory">
                        <span>"Section"</span>
                        <select
                            prop:value=move || active_section.get()
                            on:change=move |event| {
                                active_section.set(match event_target_value(&event).as_str() {
                                    "navigation" => "navigation",
                                    "deployment" => "deployment",
                                    _ => "modules",
                                });
                            }
                        >
                            <option value="modules">"Modules"</option>
                            <option value="navigation">"Navigation"</option>
                            <option value="deployment">"Deployment"</option>
                        </select>
                    </label>
                </div>

                {move || match route_state.get() {
                    ModuleRouteState::Loading => loading_state(
                        "Loading module inventory",
                        "Fetching the authorized installation inventory and navigation policy.",
                    ),
                    ModuleRouteState::Restricted => restricted_state(),
                    ModuleRouteState::NotFound => error_state(
                        "Module inventory not found",
                        "No module inventory was returned for this installation.",
                        "Retry",
                        "/administration/modules",
                    ),
                    ModuleRouteState::Unavailable(message) => unavailable_state(
                        "Module Management unavailable",
                        &message,
                        "/administration/modules",
                    ),
                    ModuleRouteState::Error(message) => error_state(
                        "Unable to load Module Management",
                        &message,
                        "Retry",
                        "/administration/modules",
                    ),
                    ModuleRouteState::Ready => {
                        let Some(current_inventory) = inventory.get() else {
                            return error_state(
                                "Module inventory unavailable",
                                "The authorized route returned no inventory projection.",
                                "Retry",
                                "/administration/modules",
                            );
                        };
                        let deployment = current_inventory.deployment.clone();
                        let deployment_history = current_inventory.deployment_history.clone();
                        view! {
                            <div
                                class="organization-detail-content module-management-content module-management-section"
                                data-section-visible=move || (active_section.get() == "modules").to_string()
                            >
                                <ModuleInventoryDirectory
                                    inventory=current_inventory
                                    on_view_deployment=Callback::new(move |_| active_section.set("deployment"))
                                />
                            </div>
                            <div
                                class="module-management-section"
                                data-section-visible=move || (active_section.get() == "navigation").to_string()
                            >
                                <ModuleNavigationPolicyView
                                    policy
                                    persisted_policy
                                    unavailable_message=policy_unavailable
                                    access=access.get()
                                />
                            </div>
                            <div
                                class="module-management-section"
                                data-section-visible=move || (active_section.get() == "deployment").to_string()
                            >
                                <DeploymentLedger receipt=deployment.clone() history=deployment_history.clone()/>
                            </div>
                        }
                        .into_any()
                    }
                }}
            </section>
        </AppShell>
    }
}

#[component]
pub fn ModuleManagementDetailPage() -> impl IntoView {
    let definition_id = use_params::<ModuleDefinitionRouteParams>()
        .get_untracked()
        .map(|params| params.definition_id)
        .unwrap_or_default();
    let detail = RwSignal::new(None::<ModuleDetailResponseV1>);
    let access = RwSignal::new(ModuleManagementAccessV1::restricted());
    let policy = RwSignal::new(None::<NavigationPolicyResponseV2>);
    let persisted_policy = RwSignal::new(None::<NavigationPolicyResponseV2>);
    let policy_unavailable = RwSignal::new(None::<String>);
    let initial = initialize_detail(
        module_management_route_bootstrap(),
        &definition_id,
        detail,
        access,
        policy,
        persisted_policy,
        policy_unavailable,
    );
    let route_state = RwSignal::new(if definition_id.is_empty() {
        ModuleRouteState::NotFound
    } else {
        initial
    });
    let active_detail_section = RwSignal::new("overview");
    let detail_retry_href = format!("/administration/modules/{definition_id}");

    #[cfg(all(feature = "hydrate", target_arch = "wasm32"))]
    {
        let shell_navigation = shell_navigation_state();
        Effect::new(move |_| refresh_shell_navigation(shell_navigation));
    }

    #[cfg(all(feature = "hydrate", target_arch = "wasm32"))]
    {
        let location = leptos_router::hooks::use_location();
        Effect::new(move |_| {
            let hash = location.hash.get();
            if let Some(section) = module_detail_section_from_hash(&hash) {
                active_detail_section.set(section);
            }
        });
    }

    #[cfg(all(feature = "hydrate", target_arch = "wasm32"))]
    if route_state.get_untracked() == ModuleRouteState::Loading {
        let requested_definition_id = definition_id.clone();
        leptos::task::spawn_local(async move {
            match fetch_module_detail(&requested_definition_id).await {
                Ok(payload) => {
                    access.set(payload.access);
                    detail.set(Some(payload.detail));
                    set_policy_bootstrap(
                        payload.navigation_policy,
                        policy,
                        persisted_policy,
                        policy_unavailable,
                    );
                    route_state.set(ModuleRouteState::Ready);
                }
                Err(error) => apply_client_error(error, route_state),
            }
        });
    }

    view! {
        <AppShell active_route="module_management" title="Module Management">
            <section class="route-panel module-management-page module-management-detail-page">
                <Breadcrumb>
                    <BreadcrumbItem><BreadcrumbLink href="/">"Home"</BreadcrumbLink></BreadcrumbItem>
                    <BreadcrumbSeparator/>
                    <BreadcrumbItem>
                        <BreadcrumbLink href="/administration/modules">"Module Management"</BreadcrumbLink>
                    </BreadcrumbItem>
                    <BreadcrumbSeparator/>
                    <BreadcrumbItem>
                        <BreadcrumbPage>
                            {move || detail
                                .get()
                                .map(|current| current.entry.display_name().to_string())
                                .unwrap_or_else(|| "Contribution detail".into())}
                        </BreadcrumbPage>
                    </BreadcrumbItem>
                </Breadcrumb>

                {move || match route_state.get() {
                    ModuleRouteState::Loading => loading_state(
                        "Loading contribution detail",
                        "Fetching the authorized descriptor projection and navigation policy.",
                    ),
                    ModuleRouteState::Restricted => restricted_state(),
                    ModuleRouteState::NotFound => not_found_state(
                        "Module definition not found",
                        "No transition contribution exists for this definition identifier.",
                    ),
                    ModuleRouteState::Unavailable(message) => unavailable_state(
                        "Contribution detail unavailable",
                        &message,
                        &detail_retry_href,
                    ),
                    ModuleRouteState::Error(message) => error_state(
                        "Unable to load contribution detail",
                        &message,
                        "Retry",
                        &detail_retry_href,
                    ),
                    ModuleRouteState::Ready => {
                        let Some(current_detail) = detail.get() else {
                            return error_state(
                                "Contribution detail unavailable",
                                "The authorized route returned no detail projection.",
                                "Retry",
                                &detail_retry_href,
                            );
                        };
                        module_detail_page(
                            current_detail.entry.into(),
                            active_detail_section,
                            policy,
                            persisted_policy,
                            policy_unavailable,
                            access.get(),
                        )
                    }
                }}
            </section>
        </AppShell>
    }
}

fn module_detail_page(
    detail: ModuleDetailViewModelV1,
    active_section: RwSignal<&'static str>,
    policy: RwSignal<Option<NavigationPolicyResponseV2>>,
    persisted_policy: RwSignal<Option<NavigationPolicyResponseV2>>,
    policy_unavailable: RwSignal<Option<String>>,
    access: ModuleManagementAccessV1,
) -> AnyView {
    let definition_id = detail.definition_id;
    let display_name = detail.display_name;
    let entry = detail.entry;
    let descriptor_href = format!("/api/admin/modules/{definition_id}/descriptor");
    let mobile_descriptor_href = descriptor_href.clone();
    let desktop_descriptor_href = descriptor_href.clone();
    let definition_id_for_copy = definition_id.clone();
    let is_independent = matches!(
        detail.presentation,
        ModuleDetailPresentationV1::IndependentlyDeployed
    );
    let declarations = match &entry {
        ModuleInventoryEntryV1::TransitionalInProcess { descriptor, .. } => {
            descriptor.navigation.clone()
        }
        ModuleInventoryEntryV1::IndependentlyDeployed { .. } => Vec::new(),
    };
    let lifecycle = match detail.presentation {
        ModuleDetailPresentationV1::Transitional { availability } => {
            let availability_class = match availability {
                super::models::TransitionAvailabilityV1::ActiveInProcess => {
                    "status-badge is-success"
                }
                super::models::TransitionAvailabilityV1::Unavailable => {
                    "status-badge is-warning"
                }
                super::models::TransitionAvailabilityV1::Retired => "status-badge is-danger",
            };
            view! {
                <span class="status-badge is-info">"Transitional"</span>
                <span class=availability_class>{availability.label()}</span>
                <p>{super::directory::TRANSITION_PRESENTATION_LABEL}</p>
            }
            .into_any()
        }
        ModuleDetailPresentationV1::IndependentlyDeployed => view! {
            <span class="status-badge is-info" title="This module runs as an independently deployed service.">"Independently deployed"</span>
            <span class=detail.serving_state.badge_class() title=detail.serving_state.explanation()>
                {detail.serving_state.detail_label()}
            </span>
        }
        .into_any(),
    };

    view! {
        <div class="module-detail-page-heading">
            <div>
                <div class="module-detail-page-heading__title-row">
                    <h1>{display_name}</h1>
                    <div class="module-detail-page-heading__actions-menu">
                        <DropdownMenu
                            label="Open module actions"
                            trigger_icon=view! { <ExternalLink class="icon-button__icon"/> }.into_any()
                        >
                            <a class="dropdown-menu__item" role="menuitem" href=mobile_descriptor_href>
                                <ExternalLink class="dropdown-menu__item-icon"/>
                                <span>"View source descriptor (JSON)"</span>
                            </a>
                            {is_independent.then(|| view! {
                                <a class="dropdown-menu__item" role="menuitem" href="/administration/modules#deployment">
                                    <ExternalLink class="dropdown-menu__item-icon"/>
                                    <span>"View deployment receipt"</span>
                                </a>
                            })}
                        </DropdownMenu>
                    </div>
                </div>
                <div class="module-detail-page-heading__identity">
                    <code>{definition_id.clone()}</code>
                    <super::directory::CopyValue value=definition_id_for_copy label="Copy module definition ID"/>
                </div>
                <div class="module-detail-page-heading__lifecycle">{lifecycle}</div>
            </div>
            <div class="module-detail-page-heading__actions">
                <div class="module-detail-page-heading__actions-desktop">
                    <a class="button button--secondary" href=desktop_descriptor_href>
                        <ExternalLink/>"View source descriptor (JSON)"
                    </a>
                    {is_independent.then(|| view! {
                        <a class="button button--secondary" href="/administration/modules#deployment">
                            <ExternalLink/>"View deployment receipt"
                        </a>
                    })}
                </div>
            </div>
        </div>
        <div class="module-section-switcher" aria-label="Module detail section">
            <div class="module-section-tabs--detail tabs-list" role="tablist" aria-label="Module detail sections">
                {MODULE_DETAIL_SECTIONS.into_iter().map(|(value,label)| view! {
                    <a class="tabs-trigger" href=format!("#{value}") role="tab"
                        aria-selected=move || (active_section.get() == value).to_string()
                        class:is-active=move || active_section.get()==value
                        on:click=move |_| active_section.set(value)>{label}</a>
                }).collect_view()}
            </div>
            <select class="module-section-select module-section-select--detail form-control"
                aria-label="Module detail section"
                prop:value=move || active_section.get()
                on:change=move |event| {
                    let value = event_target_value(&event);
                    let section = MODULE_DETAIL_SECTIONS.into_iter()
                        .find_map(|(candidate, _)| (candidate == value).then_some(candidate))
                        .unwrap_or("overview");
                    select_module_detail_section(active_section, section, section);
                }>
                {MODULE_DETAIL_SECTIONS.into_iter().map(|(value, label)| view! {
                    <option value=value>{label}</option>
                }).collect_view()}
            </select>
        </div>
        {if is_independent {
            independent_module_sections(entry, active_section)
        } else {
            view! {
                <div class="organization-detail-content module-detail-peer-sections module-detail-sections"
                    data-active-section=move || active_section.get()>
                    <div class="organization-detail-content__grid">
                        <ModuleDetailPeerSections entry=entry policy active_detail_section=active_section/>
                        <ModuleNavigationPolicyView
                            policy
                            persisted_policy
                            unavailable_message=policy_unavailable
                            access
                            definition_id
                            declared_navigation=declarations
                        />
                    </div>
                </div>
            }.into_any()
        }}
    }
    .into_any()
}

const MODULE_DETAIL_SECTIONS: [(&str, &str); 9] = [
    ("overview", "Overview"),
    ("configuration", "Configuration"),
    ("declarations", "Declarations"),
    ("contracts", "Contracts"),
    ("capabilities", "Capabilities"),
    ("dependencies", "Dependencies"),
    ("resources", "Resources"),
    ("navigation", "Navigation"),
    ("findings", "Findings"),
];

#[cfg(any(test, all(feature = "hydrate", target_arch = "wasm32")))]
fn module_detail_section_from_hash(hash: &str) -> Option<&'static str> {
    let requested = hash.trim_start_matches('#');
    if requested == "diagnostics" {
        return Some("findings");
    }
    MODULE_DETAIL_SECTIONS
        .into_iter()
        .find_map(|(section, _)| (section == requested).then_some(section))
}

fn select_module_detail_section(
    active_section: RwSignal<&'static str>,
    section: &'static str,
    _hash: &'static str,
) {
    active_section.set(section);
    #[cfg(all(feature = "hydrate", target_arch = "wasm32"))]
    if let Some(window) = web_sys::window() {
        let _ = window.location().set_hash(_hash);
    }
}

#[derive(Clone)]
struct ConfigurationField {
    name: String,
    label: String,
    value: Value,
    kind: String,
    minimum: Option<String>,
    maximum: Option<String>,
    choices: Vec<String>,
    required: bool,
}

fn configuration_fields(
    manifest: Option<&ModuleManifestV1>,
    values: &std::collections::BTreeMap<String, Value>,
) -> Vec<ConfigurationField> {
    let Some(properties) = manifest
        .and_then(|manifest| manifest.configuration_schema.get("properties"))
        .and_then(Value::as_object)
    else {
        return Vec::new();
    };
    let required = manifest
        .and_then(|manifest| manifest.configuration_schema.get("required"))
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(Value::as_str)
        .collect::<Vec<_>>();
    properties
        .iter()
        .map(|(name, schema)| ConfigurationField {
            name: name.clone(),
            label: schema
                .get("title")
                .and_then(Value::as_str)
                .map(str::to_owned)
                .unwrap_or_else(|| {
                    name.split('_')
                        .enumerate()
                        .map(|(index, word)| {
                            if index == 0 {
                                let mut chars = word.chars();
                                chars
                                    .next()
                                    .map(|first| {
                                        first.to_uppercase().collect::<String>() + chars.as_str()
                                    })
                                    .unwrap_or_default()
                            } else {
                                word.to_string()
                            }
                        })
                        .collect::<Vec<_>>()
                        .join(" ")
                }),
            value: values.get(name).cloned().unwrap_or(Value::Null),
            kind: schema
                .get("type")
                .and_then(Value::as_str)
                .unwrap_or("string")
                .to_string(),
            minimum: schema.get("minimum").map(value_text),
            maximum: schema.get("maximum").map(value_text),
            choices: schema
                .get("enum")
                .and_then(Value::as_array)
                .into_iter()
                .flatten()
                .map(value_text)
                .collect(),
            required: required.contains(&name.as_str()),
        })
        .collect()
}

fn value_text(value: &Value) -> String {
    match value {
        Value::Null => "Not configured".into(),
        Value::String(value) => value.clone(),
        Value::Bool(value) => value.to_string(),
        Value::Number(value) => value.to_string(),
        value => value.to_string(),
    }
}

fn configuration_value_row(field: ConfigurationField) -> AnyView {
    view! {
        <div>
            <dt>{field.label}</dt>
            <dd>{value_text(&field.value)}</dd>
        </div>
    }
    .into_any()
}

fn configuration_form_field(field: ConfigurationField) -> AnyView {
    let id = format!("module-configuration-{}", field.name);
    let value = value_text(&field.value);
    let label = field.label;
    let name = field.name;
    if !field.choices.is_empty() {
        let options = field
            .choices
            .into_iter()
            .map(|choice| {
                let selected = choice == value;
                view! { <option value=choice.clone() selected=selected>{choice.clone()}</option> }
            })
            .collect::<Vec<_>>();
        return view! {
            <label class="field" for=id.clone()>
                <span>{label}</span>
                <select id=id.clone() name=name required=field.required>{options}</select>
            </label>
        }
        .into_any();
    }
    if field.kind == "boolean" {
        return view! {
            <label class="field" for=id.clone()>
                <span>{label}</span>
                <select id=id.clone() name=name required=field.required>
                    <option value="true" selected=value == "true">"Enabled"</option>
                    <option value="false" selected=value != "true">"Disabled"</option>
                </select>
            </label>
        }
        .into_any();
    }
    let input_type = if matches!(field.kind.as_str(), "integer" | "number") {
        "number"
    } else {
        "text"
    };
    view! {
        <label class="field" for=id.clone()>
            <span>{label}</span>
            <input
                id=id.clone()
                name=name
                type=input_type
                value=value
                min=field.minimum
                max=field.maximum
                required=field.required
            />
        </label>
    }
    .into_any()
}

fn independent_module_sections(
    entry: ModuleInventoryEntryV1,
    active_section: RwSignal<&'static str>,
) -> AnyView {
    let findings = entry.findings().to_vec();
    let (definition, release, instance, configuration, diagnostics) =
        entry.independent().expect("independent module projection");
    let manifest = entry.manifest().cloned();
    let definition = definition.clone();
    let release = release.clone();
    let instance = instance.clone();
    let configuration = configuration.clone();
    let diagnostics = diagnostics.clone();
    let manifest_digest = release.manifest_digest.clone();
    let runtime_image = release.runtime_image.clone();
    let instance_id = instance.id.clone();
    let configuration_action = format!("/api/modules/instances/{}/configuration/form", instance.id);
    let enablement_action = format!("/api/modules/instances/{}/enablement/form", instance.id);
    let configuration_editing = RwSignal::new(false);
    let fields = configuration_fields(manifest.as_ref(), &configuration.values);
    let configuration_rows = fields
        .clone()
        .into_iter()
        .map(configuration_value_row)
        .collect::<Vec<_>>();
    let configuration_form_fields = fields
        .into_iter()
        .map(configuration_form_field)
        .collect::<Vec<_>>();
    let health_href = format!("/administration/modules/{}#diagnostics", definition.id);
    let diagnostics_download_href = format!("/api/admin/modules/{}", definition.id);
    let diagnostics_download_name = format!("{}-diagnostics.json", definition.id.replace('.', "-"));
    let dependency_rows = manifest
        .as_ref()
        .map(|manifest| {
            manifest
                .dependencies
                .iter()
                .map(|dependency| {
                    let contract = dependency.contract_id.to_string();
                    let binding = dependency.binding_key.to_string();
                    let requirement = dependency.version_requirement.to_string();
                    view! {
                        <div>
                            <dt>{contract}</dt>
                            <dd>{format!("Binding {binding} · {requirement}")}</dd>
                        </div>
                    }
                    .into_any()
                })
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    let has_dependencies = !dependency_rows.is_empty();
    let serving_state = entry.serving_state();
    let manifest_sections = independent_manifest_sections(manifest.as_ref());
    view! {
        <div class="module-detail-sections" data-active-section=move || active_section.get()>
            <div data-module-section="overview" class="module-detail-independent-overview">
                <div class="organization-detail-content__grid module-detail-independent-grid">
                    <div class="module-detail-independent-stack">
                        <section class="organization-detail-card module-detail-overview-card">
                            <div class="module-detail__heading">
                                <div>
                                    <h2>"Definition"</h2>
                                    <p>{definition.description}</p>
                                </div>
                            </div>
                            <dl class="module-detail-overview__list">
                                <div><dt>"Definition ID"</dt><dd class="module-detail-overview__value-with-action"><code>{definition.id.clone()}</code><super::directory::CopyValue value=definition.id label="Copy module definition ID"/></dd></div>
                                <div><dt>"Source digest"</dt><dd class="module-detail-overview__value-with-action"><code>{super::directory::compact_value(&manifest_digest)}</code><super::directory::CopyValue value=manifest_digest.clone() label="Copy complete source digest"/></dd></div>
                                <div><dt>"Descriptor configuration schema"</dt><dd>{if configuration.declared { "Declared" } else { "Not declared" }}</dd></div>
                                <div><dt>"Module Release"</dt><dd>{format!("{} · {} · {}", release.version, release.trust, release.compatibility)}</dd></div>
                                <div><dt>"Module Instance"</dt><dd class="module-detail-overview__value-with-action">{format!("Live · {}", super::directory::compact_value(&instance_id))}<super::directory::CopyValue value=instance_id.clone() label="Copy module instance ID"/></dd></div>
                            </dl>
                        </section>
                        <section class="organization-detail-card module-detail-overview-card">
                            <header class="module-detail__heading"><div><h2>"Lifecycle assessment"</h2><p>"Independent dimensions explain why the route is available."</p></div><span class=serving_state.badge_class() title=serving_state.explanation()>{serving_state.label()}</span></header>
                            <dl class="module-detail-overview__assessment-list">
                                <div><dt>"Dependencies"</dt><dd>"All required contracts satisfied"</dd></div>
                                <div><dt>"Compatibility"</dt><dd>{format!("Release {} is {} with this installation", release.version, release.compatibility)}</dd></div>
                                <div><dt>"Instance continuity"</dt><dd>{format!("{} · Durable instance retained", instance.identity)}</dd></div>
                                <div><dt>"Deployment"</dt><dd>{format!("{} · {} · Container {}", if instance.installed { "Installed" } else { "Not installed" }, if instance.deployed { "Deployed" } else { "Not deployed" }, if instance.healthy { "healthy" } else { "unhealthy" })}</dd></div>
                                <div><dt>"Configuration"</dt><dd>{if configuration.valid { "Valid" } else { "Finding reported" }}</dd></div>
                                <div><dt>"Readiness"</dt><dd>{if instance.ready { "Passing" } else { "Failing" }}</dd></div>
                                <div><dt>"Health"</dt><dd>{if instance.healthy { "Healthy" } else { "Unhealthy" }}</dd></div>
                                <div><dt>"Application"</dt><dd>{format!("{} · Route {}", state(instance.enabled), if instance.enabled && instance.ready && instance.healthy { "available" } else { "unavailable" })}</dd></div>
                                <div><dt>"Data"</dt><dd>{format!("{} in {}", instance.data, instance.database_name)}</dd></div>
                            </dl>
                        </section>
                        <section class="organization-detail-card module-detail-overview-card">
                            <h2>"Diagnostics"</h2>
                            <dl class="module-detail-overview__list">
                                <div><dt>"Readiness"</dt><dd class="module-detail-status-value"><span class=if instance.ready { "status-badge is-success" } else { "status-badge is-danger" } title="Result of the module readiness probe.">{if instance.ready { "Passing" } else { "Failing" }}</span><code>{diagnostics.readiness_path.clone()}</code></dd></div>
                                <div><dt>"Liveness"</dt><dd class="module-detail-status-value"><span class=if instance.healthy { "status-badge is-success" } else { "status-badge is-danger" } title="Result of the module liveness probe.">{if instance.healthy { "Passing" } else { "Failing" }}</span><code>{diagnostics.liveness_path.clone()}</code></dd></div>
                                <div><dt>"Last observation"</dt><dd><time datetime=instance.observed_at.clone()>{instance.observed_at.clone()}</time></dd></div>
                                <div><dt>"Public route"</dt><dd><code>{diagnostics.public_route}</code></dd></div>
                                <div><dt>"Database"</dt><dd><code>{instance.database_name.clone()}</code></dd></div>
                            </dl>
                        </section>
                    </div>
                    <aside class="module-detail-independent-stack"><section class="organization-detail-card module-detail-overview-card module-detail-artifact-verification"><h2>"Artifact provenance"</h2><dl class="module-detail-overview__summary-list"><div><dt>"Manifest"</dt><dd class="module-detail-overview__value-with-action"><code>{super::directory::compact_value(&manifest_digest)}</code><super::directory::CopyValue value=manifest_digest label="Copy manifest digest"/></dd></div><div><dt>"Runtime image"</dt><dd class="module-detail-overview__value-with-action"><code>{super::directory::compact_value(&runtime_image)}</code><super::directory::CopyValue value=runtime_image label="Copy runtime image digest"/></dd></div><div><dt>"Publisher"</dt><dd>{format!("{} · curated release", release.publisher)}</dd></div></dl></section></aside>
                </div>
            </div>
            <div data-module-section="configuration" class="module-configuration-grid">
                <section class="organization-detail-card module-detail-overview-card module-configuration-card">
                    <div class="module-detail__heading">
                        <div>
                            <h2>"Configuration"</h2>
                            <p>"Validated by the module-owned configuration contract."</p>
                        </div>
                        <button
                            class="button button--secondary"
                            type="button"
                            hidden=move || configuration_editing.get()
                            on:click=move |_| {
                                configuration_editing.set(true);
                            }
                        >
                            <Pencil class="button__icon"/>
                            "Edit configuration"
                        </button>
                    </div>
                    <dl class="module-detail-overview__list" hidden=move || configuration_editing.get()>
                        <div><dt>"Schema version"</dt><dd><code>"1"</code></dd></div>
                        {configuration_rows}
                        <div><dt>"Validation"</dt><dd><span class=if configuration.valid { "status-badge is-success" } else { "status-badge is-danger" }>{if configuration.valid { "Valid" } else { "Finding" }}</span>" " {format!("Release {} · {}", release.version, if configuration.valid { "no findings" } else { "review findings" })}</dd></div>
                        <div><dt>"Authoritative validator"</dt><dd>"Module-owned configuration contract"</dd></div>
                    </dl>
                    <form
                        class="module-configuration-form"
                        method="post"
                        action=configuration_action
                        hidden=move || !configuration_editing.get()
                    >
                        <input type="hidden" name="schema_version" value="1"/>
                        {configuration_form_fields}
                        <div class="module-configuration-validation">
                            <strong>"Configuration is valid"</strong>
                            <span>{format!("Schema v1 · release {} · normalized by the module · no findings", release.version)}</span>
                        </div>
                        <div class="form-actions module-configuration-form__actions">
                            <button
                                class="button button--secondary"
                                type="button"
                                on:click=move |_| {
                                    configuration_editing.set(false);
                                }
                            >
                                "Cancel"
                            </button>
                            <button class="button" type="submit">"Save configuration"</button>
                        </div>
                    </form>
                </section>
                <aside class="organization-detail-card module-detail-overview-card module-application-state">
                    <div class="module-detail__heading">
                        <div>
                            <h2>"Application state"</h2>
                            <p>"Enablement remains separate from configuration and navigation."</p>
                        </div>
                    </div>
                    <div class="module-application-state__line"><span>"Configured"</span><span class=if configuration.valid { "status-badge is-success" } else { "status-badge is-danger" }>{if configuration.valid { "Valid" } else { "Finding" }}</span></div>
                    <div class="module-application-state__line"><span>"Module health"</span><span class=if instance.healthy { "status-badge is-success" } else { "status-badge is-danger" }>{if instance.healthy { "Healthy" } else { "Degraded" }}</span></div>
                    <div class="module-application-state__line"><span>"Navigation"</span><span class="status-badge is-info">{if instance.enabled { "Visible" } else { "Hidden" }}</span></div>
                    <div class="module-application-state__enablement">
                        <div>
                            <strong>{if instance.enabled { "Product route enabled" } else { "Product route disabled" }}</strong>
                            <span>{if instance.enabled { "Authorized users can open the module." } else { "The product route and navigation are disabled. Configuration and diagnostics remain available." }}</span>
                        </div>
                        <form class="module-application-state__enablement-form" method="post" action=enablement_action>
                            <input type="hidden" name="enabled" value=if instance.enabled { "false" } else { "true" }/>
                            <button
                                type="submit"
                                role="switch"
                                aria-label=if instance.enabled { "Disable product route" } else { "Enable product route" }
                                aria-checked=if instance.enabled { "true" } else { "false" }
                                class=if instance.enabled { "module-application-state__switch is-on" } else { "module-application-state__switch" }
                                title=if instance.enabled { "Disable the module product route." } else { "Enable the module product route." }
                            >
                                <span></span>
                            </button>
                        </form>
                    </div>
                    <a
                        class="button button--secondary module-application-state__action"
                        href=health_href.clone()
                        on:click=move |_| {
                            select_module_detail_section(active_section, "findings", "diagnostics");
                        }
                    >
                        <HeartPulse class="button__icon"/>
                        "Open health and diagnostics"
                    </a>
                </aside>
            </div>
            <section data-module-section="findings" class="module-dashboard-diagnostics">
                <div class="module-dashboard-diagnostics__heading">
                    <div>
                        <p class="module-dashboard-diagnostics__eyebrow">{definition.display_name.clone()}</p>
                        <h2>"Health and diagnostics"</h2>
                        <p>"Sanitized operational context from the shared independent-module contract."</p>
                    </div>
                    <div class="module-dashboard-diagnostics__actions">
                        <a class="button button--secondary" href=health_href>
                            <RefreshCw class="button__icon"/>
                            "Refresh status"
                        </a>
                        <a
                            class="button button--secondary"
                            href=diagnostics_download_href
                            download=diagnostics_download_name
                        >
                            <Download class="button__icon"/>
                            "Download diagnostics"
                        </a>
                    </div>
                </div>
                <div class="module-dashboard-diagnostics__metrics">
                    <article class="module-dashboard-diagnostic-metric">
                        <HeartPulse/>
                        <div>
                            <span>"Readiness"</span>
                            <strong>{if instance.ready { "Ready" } else { "Not ready" }}</strong>
                            <small>{if instance.ready { "Configuration, data, and Core authorization exchange are available." } else { "The module readiness probe is not passing." }}</small>
                        </div>
                    </article>
                    <article class="module-dashboard-diagnostic-metric">
                        <Activity/>
                        <div>
                            <span>"Liveness"</span>
                            <strong>{if instance.healthy { "Healthy" } else { "Unhealthy" }}</strong>
                            <small>"The module responded during the "<time datetime=instance.observed_at.clone()>"latest health check"</time>"."</small>
                        </div>
                    </article>
                    <article class="module-dashboard-diagnostic-metric">
                        <Database/>
                        <div>
                            <span>"Module database"</span>
                            <strong>{if instance.deployed { "Connected" } else { "Unavailable" }}</strong>
                            <small><code>{instance.database_name}</code>" · instance-scoped runtime identity."</small>
                        </div>
                    </article>
                    <article class="module-dashboard-diagnostic-metric">
                        <ShieldCheck/>
                        <div>
                            <span>"Core authorization"</span>
                            <strong>{if instance.enabled { "Current" } else { "Disabled" }}</strong>
                            <small>"Core-signed action grants · no reusable credential exposed."</small>
                        </div>
                    </article>
                </div>
                <section class="organization-detail-card module-dashboard-dependency-card">
                    <div class="module-detail__heading">
                        <div>
                            <h2>"Dependency assessment"</h2>
                            <p>"Contract requirements declared by the module manifest."</p>
                        </div>
                        <span class="status-badge is-success">"Satisfied"</span>
                    </div>
                    {if has_dependencies {
                        view! { <dl class="module-detail-overview__list">{dependency_rows}</dl> }.into_any()
                    } else {
                        view! { <p>"No external functional dependencies were declared."</p> }.into_any()
                    }}
                </section>
                <section class="organization-detail-card module-dashboard-dependency-card">
                    <div class="module-detail__heading">
                        <div>
                            <h2>"Findings"</h2>
                            <p>"Actionable observations from the shared independent-module projection."</p>
                        </div>
                        <span class=if findings.is_empty() { "status-badge is-success" } else { "status-badge is-danger" }>
                            {if findings.is_empty() { "None" } else { "Review" }}
                        </span>
                    </div>
                    {if findings.is_empty() {
                        view! { <p>"No module findings were reported."</p> }.into_any()
                    } else {
                        view! {
                            <dl class="module-detail-overview__list">
                                {findings.into_iter().map(|finding| view! {
                                    <div data-finding-code=finding.code.clone()>
                                        <dt><code>{finding.code.clone()}</code></dt>
                                        <dd>{finding.message}" · "<code>{finding.path}</code></dd>
                                    </div>
                                }).collect_view()}
                            </dl>
                        }.into_any()
                    }}
                </section>
            </section>
            {manifest_sections}
        </div>
    }.into_any()
}

fn independent_manifest_sections(
    manifest: Option<&super::models::ModuleManifestV1>,
) -> Vec<AnyView> {
    let Some(manifest) = manifest else {
        return [
            ("declarations", "Declarations", "No persisted module manifest is available."),
            ("contracts", "Contracts", "No persisted module manifest is available."),
            ("capabilities", "Capabilities", "No persisted module manifest is available."),
            ("dependencies", "Dependencies", "No persisted module manifest is available."),
            ("resources", "Resources", "No persisted module manifest is available."),
            ("navigation", "Navigation", "No persisted module manifest is available."),
        ]
        .into_iter()
        .map(|(section, title, message)| view! {
            <section data-module-section=section class="organization-detail-card module-detail-empty-section">
                <h2>{title}</h2><p>{message}</p>
            </section>
        }.into_any())
        .collect();
    };

    let mut declaration_rows = manifest
        .features
        .iter()
        .map(|item| {
            (
                item.name.clone(),
                format!("Feature {} · {}", item.id, item.description),
            )
        })
        .collect::<Vec<_>>();
    declaration_rows.extend(manifest.routes.iter().map(|route| {
        (
            format!("Route · {}", route.name),
            format!("{:?} · {} parameter(s)", route.kind, route.parameters.len()),
        )
    }));
    declaration_rows.extend(manifest.typed_reference_schemas.iter().map(|reference| {
        (
            format!("Typed reference · {}", reference.resource_type),
            format!("{:?}", reference.resource_id),
        )
    }));
    declaration_rows.extend([
        (
            "Configuration validation".into(),
            manifest
                .operational_routes
                .configuration_validation
                .to_string(),
        ),
        (
            "Compatibility".into(),
            manifest.operational_routes.compatibility.to_string(),
        ),
        (
            "Status".into(),
            manifest.operational_routes.status.to_string(),
        ),
        (
            "Diagnostics".into(),
            manifest.operational_routes.diagnostics.to_string(),
        ),
    ]);
    if let Some(shell) = &manifest.shell_contribution_contracts {
        for (surface, contract) in [
            ("Home contribution", shell.home.as_ref()),
            ("Work discovery contribution", shell.work_discovery.as_ref()),
            ("Search contribution", shell.search.as_ref()),
        ] {
            if let Some(contract) = contract {
                declaration_rows.push((surface.into(), contract.to_string()));
            }
        }
    }
    let sections = vec![
        (
            "declarations",
            "Declarations",
            declaration_rows,
            "No feature declarations were reported.",
        ),
        (
            "contracts",
            "Contracts",
            manifest
                .provided_contracts
                .iter()
                .map(|item| {
                    (
                        item.id.to_string(),
                        format!("{} · {}", item.version, item.description),
                    )
                })
                .collect(),
            "No contracts were declared.",
        ),
        (
            "capabilities",
            "Capabilities",
            manifest
                .security_capabilities
                .iter()
                .map(|item| (item.id.to_string(), item.description.clone()))
                .collect(),
            "No capabilities were declared.",
        ),
        (
            "dependencies",
            "Dependencies",
            manifest
                .dependencies
                .iter()
                .map(|item| {
                    (
                        item.contract_id.to_string(),
                        format!(
                            "{} · binding {}",
                            item.version_requirement, item.binding_key
                        ),
                    )
                })
                .collect(),
            "No dependencies were declared.",
        ),
        (
            "resources",
            "Resources",
            manifest
                .resource_types
                .iter()
                .map(|item| (item.id.to_string(), item.description.clone()))
                .collect(),
            "No resources were declared.",
        ),
        (
            "navigation",
            "Navigation",
            manifest
                .navigation
                .iter()
                .map(|item| {
                    (
                        item.label.clone(),
                        format!("{} · {}", item.destination, item.group),
                    )
                })
                .collect(),
            "No navigation contribution was declared.",
        ),
    ];
    sections
        .into_iter()
        .map(|(section, title, rows, empty)| {
            if rows.is_empty() {
                view! {
                    <section data-module-section=section class="organization-detail-card module-detail-empty-section">
                        <h2>{title}</h2><p>{empty}</p>
                    </section>
                }
                .into_any()
            } else {
                view! {
                    <section data-module-section=section class="organization-detail-card module-detail-overview-card">
                        <h2>{title}</h2>
                        <dl class="module-detail-overview__list">
                            {rows.into_iter().map(|(label, value)| view! {
                                <div><dt>{label}</dt><dd>{value}</dd></div>
                            }).collect_view()}
                        </dl>
                    </section>
                }
                .into_any()
            }
        })
        .collect()
}

fn state(value: bool) -> &'static str {
    if value { "Enabled" } else { "Disabled" }
}

fn initialize_directory(
    bootstrap: Option<ModuleManagementRouteBootstrapV1>,
    inventory: RwSignal<Option<ModuleInventoryResponseV1>>,
    access: RwSignal<ModuleManagementAccessV1>,
    policy: RwSignal<Option<NavigationPolicyResponseV2>>,
    persisted_policy: RwSignal<Option<NavigationPolicyResponseV2>>,
    policy_unavailable: RwSignal<Option<String>>,
) -> ModuleRouteState {
    match bootstrap {
        Some(ModuleManagementRouteBootstrapV1::Directory {
            access: authorized_access,
            inventory: initial_inventory,
            navigation_policy,
        }) if authorized_access.may_read() => {
            access.set(authorized_access);
            inventory.set(Some(*initial_inventory));
            set_policy_bootstrap(
                navigation_policy,
                policy,
                persisted_policy,
                policy_unavailable,
            );
            ModuleRouteState::Ready
        }
        Some(ModuleManagementRouteBootstrapV1::Restricted { .. })
        | Some(ModuleManagementRouteBootstrapV1::Directory { .. }) => ModuleRouteState::Restricted,
        Some(ModuleManagementRouteBootstrapV1::Unavailable { message, .. }) => {
            ModuleRouteState::Unavailable(message)
        }
        Some(ModuleManagementRouteBootstrapV1::NotFound { .. }) => ModuleRouteState::NotFound,
        Some(ModuleManagementRouteBootstrapV1::Detail { .. })
            if cfg!(all(feature = "hydrate", target_arch = "wasm32")) =>
        {
            ModuleRouteState::Loading
        }
        Some(ModuleManagementRouteBootstrapV1::Detail { .. }) => ModuleRouteState::Error(
            "The server supplied a detail bootstrap for the directory route.".into(),
        ),
        None if cfg!(all(feature = "hydrate", target_arch = "wasm32")) => ModuleRouteState::Loading,
        None => ModuleRouteState::Unavailable(
            "Module inventory was not supplied for this server-rendered request.".into(),
        ),
    }
}

fn initialize_detail(
    bootstrap: Option<ModuleManagementRouteBootstrapV1>,
    requested_definition_id: &str,
    detail: RwSignal<Option<ModuleDetailResponseV1>>,
    access: RwSignal<ModuleManagementAccessV1>,
    policy: RwSignal<Option<NavigationPolicyResponseV2>>,
    persisted_policy: RwSignal<Option<NavigationPolicyResponseV2>>,
    policy_unavailable: RwSignal<Option<String>>,
) -> ModuleRouteState {
    match bootstrap {
        Some(ModuleManagementRouteBootstrapV1::Detail {
            access: authorized_access,
            detail: initial_detail,
            navigation_policy,
        }) if authorized_access.may_read()
            && initial_detail.entry.definition_id() == requested_definition_id =>
        {
            access.set(authorized_access);
            detail.set(Some(*initial_detail));
            set_policy_bootstrap(
                navigation_policy,
                policy,
                persisted_policy,
                policy_unavailable,
            );
            ModuleRouteState::Ready
        }
        Some(ModuleManagementRouteBootstrapV1::Restricted { .. }) => ModuleRouteState::Restricted,
        Some(ModuleManagementRouteBootstrapV1::Detail { access, .. }) if !access.may_read() => {
            ModuleRouteState::Restricted
        }
        Some(ModuleManagementRouteBootstrapV1::Detail { .. })
            if cfg!(all(feature = "hydrate", target_arch = "wasm32")) =>
        {
            // A request-scoped bootstrap must never render under another
            // definition URL during client-side navigation.
            ModuleRouteState::Loading
        }
        Some(ModuleManagementRouteBootstrapV1::Detail { .. }) => ModuleRouteState::Error(
            "The server supplied detail for a different module definition.".into(),
        ),
        Some(ModuleManagementRouteBootstrapV1::NotFound { .. }) => ModuleRouteState::NotFound,
        Some(ModuleManagementRouteBootstrapV1::Unavailable { message, .. }) => {
            ModuleRouteState::Unavailable(message)
        }
        Some(ModuleManagementRouteBootstrapV1::Directory { .. })
            if cfg!(all(feature = "hydrate", target_arch = "wasm32")) =>
        {
            ModuleRouteState::Loading
        }
        Some(ModuleManagementRouteBootstrapV1::Directory { .. }) => ModuleRouteState::Error(
            "The server supplied a directory bootstrap for the detail route.".into(),
        ),
        None if cfg!(all(feature = "hydrate", target_arch = "wasm32")) => ModuleRouteState::Loading,
        None => ModuleRouteState::Unavailable(
            "Module detail was not supplied for this server-rendered request.".into(),
        ),
    }
}

fn set_policy_bootstrap(
    bootstrap: NavigationPolicyBootstrapV1,
    policy: RwSignal<Option<NavigationPolicyResponseV2>>,
    persisted_policy: RwSignal<Option<NavigationPolicyResponseV2>>,
    unavailable: RwSignal<Option<String>>,
) {
    match bootstrap {
        NavigationPolicyBootstrapV1::Ready { policy: initial } => {
            persisted_policy.set(Some(initial.clone()));
            policy.set(Some(initial));
            unavailable.set(None);
        }
        NavigationPolicyBootstrapV1::Unavailable { message } => {
            policy.set(None);
            persisted_policy.set(None);
            unavailable.set(Some(message));
        }
    }
}

#[cfg(all(feature = "hydrate", target_arch = "wasm32"))]
fn apply_client_error(error: ModuleManagementClientError, state: RwSignal<ModuleRouteState>) {
    match error {
        ModuleManagementClientError::Authentication => {
            state.set(ModuleRouteState::Restricted);
            crate::http::redirect_to_login();
        }
        ModuleManagementClientError::Restricted => state.set(ModuleRouteState::Restricted),
        ModuleManagementClientError::NotFound => state.set(ModuleRouteState::NotFound),
        ModuleManagementClientError::Conflict(message) => {
            state.set(ModuleRouteState::Error(message));
        }
        ModuleManagementClientError::Unavailable(message) => {
            state.set(ModuleRouteState::Unavailable(message));
        }
        ModuleManagementClientError::Failed(message) => {
            state.set(ModuleRouteState::Error(message));
        }
    }
}

fn loading_state(title: &'static str, message: &'static str) -> AnyView {
    module_route_state(
        title,
        message,
        "Directory · loading",
        "is-loading",
        None,
        true,
    )
}

fn restricted_state() -> AnyView {
    module_route_state(
        "Module Management restricted",
        "This account does not have installation-global Module Management read access.",
        "Route · restricted",
        "module-management-restricted",
        Some(("Return home", "/".into())),
        false,
    )
}

fn unavailable_state(title: &'static str, message: &str, retry_href: &str) -> AnyView {
    module_route_state(
        title,
        message,
        "Directory · unavailable",
        "module-management-unavailable",
        Some(("Retry", retry_href.into())),
        false,
    )
}

fn error_state(
    title: &'static str,
    message: &str,
    action_label: &'static str,
    action_href: &str,
) -> AnyView {
    module_route_state(
        title,
        message,
        "Route · error",
        "is-error",
        Some((action_label, action_href.into())),
        false,
    )
}

fn not_found_state(title: &'static str, message: &str) -> AnyView {
    module_route_state(
        title,
        message,
        "Detail · not found",
        "module-management-not-found",
        Some((
            "Back to Module Management",
            "/administration/modules".into(),
        )),
        false,
    )
}

fn module_route_state(
    title: &'static str,
    message: &str,
    label: &'static str,
    class_name: &'static str,
    action: Option<(&'static str, String)>,
    is_loading: bool,
) -> AnyView {
    let class = format!("organization-state module-management-state {class_name}");
    let message = message.to_string();

    view! {
        <section
            class=class
            role=if class_name == "is-error" || class_name == "module-management-restricted" { "alert" } else { "status" }
            aria-live="polite"
            aria-busy=if is_loading { "true" } else { "false" }
        >
            <span class="module-state__label">{label}</span>
            <div class="module-management-state__copy">
                <h2>{title}</h2>
                <p>{message}</p>
                {action.map(|(action_label, href)| view! {
                    <a class="button button--secondary" href=href>{action_label}</a>
                })}
            </div>
            {is_loading.then(|| view! {
                <div class="module-management-state__skeleton" aria-hidden="true">
                    <span></span><span></span><span></span>
                </div>
            })}
        </section>
    }
    .into_any()
}

#[cfg(test)]
mod tests {
    use leptos::prelude::*;

    use super::{
        error_state, loading_state, module_detail_section_from_hash, not_found_state,
        restricted_state, unavailable_state,
    };

    #[test]
    fn diagnostics_hash_selects_the_mockup_findings_destination() {
        assert_eq!(
            module_detail_section_from_hash("#diagnostics"),
            Some("findings")
        );
        assert_eq!(
            module_detail_section_from_hash("#configuration"),
            Some("configuration")
        );
        assert_eq!(module_detail_section_from_hash("#unknown"), None);
    }

    #[test]
    fn route_states_remain_semantically_distinct() {
        let (loading, restricted, unavailable, not_found, error) = Owner::new().with(|| {
            (
                loading_state("Loading module inventory", "Fetching modules.").to_html(),
                restricted_state().to_html(),
                unavailable_state(
                    "Module Management unavailable",
                    "Try again later.",
                    "/administration/modules",
                )
                .to_html(),
                not_found_state(
                    "Module definition not found",
                    "No transition contribution exists for this definition identifier.",
                )
                .to_html(),
                error_state(
                    "Unable to load Module Management",
                    "Invalid response.",
                    "Retry",
                    "/administration/modules",
                )
                .to_html(),
            )
        });

        assert!(loading.contains("aria-busy=\"true\""));
        assert!(restricted.contains("restricted"));
        assert!(restricted.contains("installation-global"));
        assert!(restricted.contains("href=\"/\""));
        assert!(unavailable.contains("Try again later"));
        assert!(!unavailable.contains("access"));
        assert!(unavailable.contains("href=\"/administration/modules\""));
        assert!(not_found.contains("Detail · not found"));
        assert!(not_found.contains("Back to Module Management"));
        assert!(error.contains("is-error"));
    }
}
