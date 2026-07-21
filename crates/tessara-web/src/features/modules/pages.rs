//! Native Module Management route pages.

use icons::ExternalLink;
use leptos::prelude::*;
use leptos_router::hooks::use_params;
use leptos_router::params::{Params, ParamsError, ParamsMap};

#[cfg(all(feature = "hydrate", target_arch = "wasm32"))]
use super::api::{ModuleManagementClientError, fetch_module_detail, fetch_module_directory};
use super::bootstrap::{
    ModuleManagementRouteBootstrapV1, NavigationPolicyBootstrapV1,
    module_management_route_bootstrap,
};
use super::detail::ModuleDetailPeerSections;
use super::directory::ModuleInventoryDirectory;
use super::models::{
    ModuleDetailResponseV1, ModuleInventoryResponseV1, ModuleManagementAccessV1,
    NavigationContributionDeclarationV1, NavigationPolicyResponseV2, TransitionAvailabilityV1,
};
use super::policy::ModuleNavigationPolicyView;
use crate::ui::{
    AppShell, Breadcrumb, BreadcrumbItem, BreadcrumbLink, BreadcrumbPage, BreadcrumbSeparator,
    PageHeader,
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
                    </div>
                    <label class="module-section-select module-section-select--directory">
                        <span>"Section"</span>
                        <select
                            prop:value=move || active_section.get()
                            on:change=move |event| {
                                active_section.set(if event_target_value(&event) == "navigation" {
                                    "navigation"
                                } else {
                                    "modules"
                                });
                            }
                        >
                            <option value="modules">"Modules"</option>
                            <option value="navigation">"Navigation"</option>
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
                        view! {
                            <div
                                class="organization-detail-content module-management-content module-management-section"
                                data-section-visible=move || (active_section.get() == "modules").to_string()
                            >
                                <ModuleInventoryDirectory inventory=current_inventory/>
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
                                .map(|current| current.entry.descriptor().display_name.clone())
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
                        let descriptor = current_detail.entry.descriptor().clone();
                        let display_name = descriptor.display_name.clone();
                        let definition_id_for_heading = descriptor.reserved_definition_id.clone();
                        let definition_id_for_copy = definition_id_for_heading.clone();
                        let availability_label = descriptor.availability.label();
                        let availability_class = match descriptor.availability {
                            TransitionAvailabilityV1::ActiveInProcess => "status-badge is-success",
                            TransitionAvailabilityV1::Unavailable => "status-badge is-warning",
                            TransitionAvailabilityV1::Retired => "status-badge is-danger",
                        };
                        let declarations: Vec<NavigationContributionDeclarationV1> =
                            descriptor.navigation;
                        view! {
                            <div class="module-detail-page-heading">
                                <div>
                                    <h1>{display_name.clone()}</h1>
                                    <div class="module-detail-page-heading__identity">
                                        <code>{definition_id_for_heading}</code>
                                        <super::directory::CopyValue
                                            value=definition_id_for_copy
                                            label="Copy module definition ID"
                                        />
                                    </div>
                                    <div class="module-detail-page-heading__lifecycle">
                                        <span class="status-badge is-info">"Transitional"</span>
                                        <span class=availability_class>{availability_label}</span>
                                        <p>{super::directory::TRANSITION_PRESENTATION_LABEL}</p>
                                    </div>
                                </div>
                                <a
                                    class="button button--secondary module-detail-page-heading__descriptor"
                                    href=format!("/api/admin/modules/{}/descriptor", definition_id.clone())
                                >
                                    <ExternalLink class="module-detail-page-heading__descriptor-icon"/>
                                    "View source descriptor (JSON)"
                                </a>
                            </div>
                            <div class="module-section-switcher" aria-label="Module detail section">
                                <div class="module-section-tabs--detail tabs-list" role="tablist" aria-label="Module detail sections">
                                    {[
                                        ("overview", "Overview"),
                                        ("declarations", "Declarations"),
                                        ("contracts", "Contracts"),
                                        ("capabilities", "Capabilities"),
                                        ("dependencies", "Dependencies"),
                                        ("resources", "Resources"),
                                        ("navigation", "Navigation"),
                                        ("findings", "Findings"),
                                    ].into_iter().map(|(value, label)| view! {
                                        <button
                                            class="tabs-trigger"
                                            type="button"
                                            role="tab"
                                            aria-selected=move || active_detail_section.get() == value
                                            class:is-active=move || active_detail_section.get() == value
                                            on:click=move |_| active_detail_section.set(value)
                                        >{label}</button>
                                    }).collect_view()}
                                </div>
                                <select
                                    class="module-section-select module-section-select--detail form-control"
                                    aria-label="Module detail section"
                                        prop:value=move || active_detail_section.get()
                                        on:change=move |event| {
                                            let value = event_target_value(&event);
                                            active_detail_section.set(match value.as_str() {
                                                "declarations" => "declarations",
                                                "contracts" => "contracts",
                                                "dependencies" => "dependencies",
                                                "capabilities" => "capabilities",
                                                "resources" => "resources",
                                                "navigation" => "navigation",
                                                "findings" => "findings",
                                                _ => "overview",
                                            });
                                        }
                                    >
                                        <option value="overview">"Overview"</option>
                                        <option value="declarations">"Declarations"</option>
                                        <option value="contracts">"Contracts"</option>
                                        <option value="dependencies">"Dependencies"</option>
                                        <option value="capabilities">"Capabilities"</option>
                                        <option value="resources">"Resources"</option>
                                        <option value="navigation">"Navigation"</option>
                                        <option value="findings">"Findings"</option>
                                </select>
                            </div>
                            <div
                                class="organization-detail-content module-detail-peer-sections module-detail-sections"
                                data-active-section=move || active_detail_section.get()
                            >
                                <div class="organization-detail-content__grid">
                                    <ModuleDetailPeerSections
                                        entry=current_detail.entry
                                        policy
                                        active_detail_section
                                    />
                                    <ModuleNavigationPolicyView
                                        policy
                                        persisted_policy
                                        unavailable_message=policy_unavailable
                                        access=access.get()
                                        definition_id=definition_id.clone()
                                        declared_navigation=declarations
                                    />
                                </div>
                            </div>
                        }
                        .into_any()
                    }
                }}
            </section>
        </AppShell>
    }
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
            inventory.set(Some(initial_inventory));
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

    use super::{error_state, loading_state, not_found_state, restricted_state, unavailable_state};

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
