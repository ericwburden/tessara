//! Module Management directory presentation.

use icons::{BoxIcon, ChevronRight, Clock, Copy, Info, List, Search, Tag, TriangleAlert};
use leptos::prelude::*;
use tessara_web_ui::{SideSheet, SideSheetSide};

use super::models::{ModuleInventoryEntryV1, ModuleInventoryResponseV1, TransitionAvailabilityV1};
use crate::ui::{DataTable, TablePaginationFooter};

pub const TRANSITION_PRESENTATION_LABEL: &str = "Transitional — not independently deployable";
pub const NO_MODULE_RELEASE_LABEL: &str = "No Module Release";
pub const NO_MODULE_INSTANCE_LABEL: &str = "No Module Instance";

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
enum ModuleDirectoryStatusFilter {
    #[default]
    All,
    ActiveInCoreProcess,
    Unavailable,
    Retired,
    Ready,
}

impl ModuleDirectoryStatusFilter {
    fn from_value(value: &str) -> Self {
        match value {
            "active_in_core_process" => Self::ActiveInCoreProcess,
            "unavailable" => Self::Unavailable,
            "retired" => Self::Retired,
            "ready" => Self::Ready,
            _ => Self::All,
        }
    }

    const fn value(self) -> &'static str {
        match self {
            Self::All => "all",
            Self::ActiveInCoreProcess => "active_in_core_process",
            Self::Unavailable => "unavailable",
            Self::Retired => "retired",
            Self::Ready => "ready",
        }
    }

    const fn matches(self, availability: TransitionAvailabilityV1) -> bool {
        match self {
            Self::All => true,
            Self::ActiveInCoreProcess => {
                matches!(availability, TransitionAvailabilityV1::ActiveInProcess)
            }
            Self::Unavailable => matches!(availability, TransitionAvailabilityV1::Unavailable),
            Self::Retired => matches!(availability, TransitionAvailabilityV1::Retired),
            Self::Ready => false,
        }
    }
}

fn module_matches_filters(
    entry: &ModuleInventoryEntryV1,
    query: &str,
    status: ModuleDirectoryStatusFilter,
) -> bool {
    let normalized_query = query.trim().to_lowercase();
    let matches_text = normalized_query.is_empty()
        || entry
            .display_name()
            .to_lowercase()
            .contains(&normalized_query)
        || entry
            .definition_id()
            .to_lowercase()
            .contains(&normalized_query);
    let matches_status = match entry {
        ModuleInventoryEntryV1::TransitionalInProcess { descriptor, .. } => {
            status.matches(descriptor.availability)
        }
        ModuleInventoryEntryV1::IndependentlyDeployed { .. } => {
            status == ModuleDirectoryStatusFilter::All
                || (status == ModuleDirectoryStatusFilter::Ready
                    && entry.serving_state().is_ready())
        }
    };
    matches_text && matches_status
}

#[component]
pub fn ModuleInventoryDirectory(
    inventory: ModuleInventoryResponseV1,
    #[prop(optional)] on_view_deployment: Option<Callback<()>>,
) -> impl IntoView {
    let installation = inventory.installation;
    let core_runtime = inventory.core_runtime;
    let deployment = inventory.deployment;
    let deployment_for_installation = deployment.clone();
    let deployment_for_runtime_details = deployment.clone();
    let entries = inventory.entries;
    let entry_count = entries.len();
    let search = RwSignal::new(String::new());
    let status = RwSignal::new(ModuleDirectoryStatusFilter::All);
    let page_size = RwSignal::new(10_usize);
    let page_index = RwSignal::new(0_usize);
    let runtime_details_open = RwSignal::new(false);
    let close_runtime_details = Callback::new(move |_| runtime_details_open.set(false));
    let entries_for_results = entries.clone();
    let filtered_entries = Memo::new(move |_| {
        let query = search.get();
        let selected_status = status.get();
        entries_for_results
            .iter()
            .filter(|entry| module_matches_filters(entry, &query, selected_status))
            .cloned()
            .collect::<Vec<_>>()
    });
    let total_count = Memo::new(move |_| filtered_entries.get().len());

    view! {
        <div class="module-management-directory">
            <section class="module-runtime-strip" aria-labelledby="module-core-runtime-heading">
                <h2 id="module-core-runtime-heading" class="sr-only">"Core runtime context"</h2>
                <div class="module-runtime-strip__item">
                    <Info/>
                    <div>
                        <span>"Installation ID"</span>
                        <span class="module-runtime-strip__identity">
                            <code>{compact_value(&installation.id)}</code>
                            <CopyValue value=installation.id.clone() label="Copy installation ID"/>
                        </span>
                        {deployment_for_installation.map(|receipt| {
                            let healthy = receipt.components.iter().all(|component| component.healthy);
                            view! {
                                <span class=if healthy { "status-badge is-success" } else { "status-badge is-danger" } title=if healthy { "All deployed components passed their current health checks." } else { "One or more deployed components failed their current health check." }>
                                    {if healthy { "Healthy" } else { "Attention required" }}
                                </span>
                            }
                        })}
                    </div>
                </div>
                <div class="module-runtime-strip__item">
                    <BoxIcon/>
                    <div>
                        <span>"Observed Core version"</span>
                        <strong>{core_runtime.observed_version.clone()}</strong>
                    </div>
                </div>
                <div class="module-runtime-strip__item">
                    <Tag/>
                    <div>
                        <span>"Release provenance"</span>
                        <strong>{core_runtime.provenance.clone()}</strong>
                    </div>
                </div>
                <div class="module-runtime-strip__item">
                    <List/>
                    <div><span>{format!("{entry_count} definitions")}</span></div>
                </div>
                <div class="module-runtime-strip__item">
                    <Clock/>
                    <div>
                        <span>"Observed at"</span>
                        <time datetime=core_runtime.observed_at.clone()>{core_runtime.observed_at.clone()}</time>
                    </div>
                </div>
                <button
                    class="module-runtime-strip__details-button"
                    type="button"
                    on:click=move |_| runtime_details_open.set(true)
                >
                    "Runtime details" <ChevronRight/>
                </button>
            </section>

            <SideSheet
                id="module-runtime-details"
                title="Core runtime details"
                description="This installation is the stable owner of current in-process transition resources. The observed runtime is not presented as a fabricated exact Core Release."
                eyebrow="Module Management"
                open=Signal::derive(move || runtime_details_open.get())
                on_close=close_runtime_details
                side=SideSheetSide::End
                close_label="Close Core runtime details"
                class="module-runtime-details-sheet"
            >
                <section class="sheet-panel__section">
                    <h3>"Runtime observation"</h3>
                    <table class="info-list-table module-runtime-details-table">
                        <tbody>
                            <tr>
                                <th scope="row">"Application Installation"</th>
                                <td><code>{installation.id.clone()}</code></td>
                            </tr>
                            <tr>
                                <th scope="row">"Installation created"</th>
                                <td><time datetime=installation.created_at.clone()>{installation.created_at.clone()}</time></td>
                            </tr>
                            <tr>
                                <th scope="row">"Observed Core version"</th>
                                <td>{core_runtime.observed_version.clone()}</td>
                            </tr>
                            <tr>
                                <th scope="row">"Release provenance"</th>
                                <td>{core_runtime.provenance.clone()}</td>
                            </tr>
                            <tr>
                                <th scope="row">"Runtime finding"</th>
                                <td><code>{core_runtime.finding_code.clone()}</code></td>
                            </tr>
                            <tr>
                                <th scope="row">"Observed at"</th>
                                <td><time datetime=core_runtime.observed_at.clone()>{core_runtime.observed_at.clone()}</time></td>
                            </tr>
                        </tbody>
                    </table>
                </section>
                {deployment_for_runtime_details.clone().map(|receipt| {
                    let healthy_count = receipt.components.iter().filter(|component| component.healthy).count();
                    let database_count = receipt.modules.len() + 2;
                    view! {
                        <section class="sheet-panel__section">
                            <h3>"Deployment runtime"</h3>
                            <table class="info-list-table module-runtime-details-table">
                                <tbody>
                                    <tr>
                                        <th scope="row">"Containers"</th>
                                        <td>{format!("{healthy_count} of {} healthy", receipt.components.len())}</td>
                                    </tr>
                                    <tr>
                                        <th scope="row">"Database"</th>
                                        <td>{format!("{database_count} databases · 1 cluster")}</td>
                                    </tr>
                                    <tr>
                                        <th scope="row">"Applied Receipt:"</th>
                                        <td>
                                            <button
                                                class="link-button"
                                                type="button"
                                                on:click=move |_| {
                                                    if let Some(callback) = on_view_deployment {
                                                        callback.run(());
                                                    }
                                                }
                                            >{format!("Revision {}", receipt.revision)}</button>
                                        </td>
                                    </tr>
                                </tbody>
                            </table>
                        </section>
                    }
                })}
            </SideSheet>

            <section class="module-directory" aria-labelledby="module-directory-heading">
                <div class="module-directory__heading">
                    <h2 id="module-directory-heading">"Module definitions"</h2>
                </div>
                <p class="module-directory__legend">
                    <span class="status-badge is-info">{TRANSITION_PRESENTATION_LABEL}</span>
                </p>

                {if entries.is_empty() {
                    view! {
                        <section class="empty-state module-directory__empty" aria-live="polite">
                            <span class="module-state__label">"Directory · empty"</span>
                            <h3>"No module definitions"</h3>
                            <p>"No transition contributions are currently registered."</p>
                        </section>
                    }
                    .into_any()
                } else {
                    view! {
                        <div class="module-directory__toolbar" aria-label="Filter module definitions">
                            <label class="searchable-data-table__control searchable-data-table__search">
                                <Search class="searchable-data-table__control-icon"/>
                                <span class="sr-only">"Search module name or ID"</span>
                                <input
                                    type="search"
                                    placeholder="Search module name or ID"
                                    prop:value=move || search.get()
                                    on:input=move |event| {
                                        search.set(event_target_value(&event));
                                        page_index.set(0);
                                    }
                                />
                            </label>
                            <label class="searchable-data-table__control searchable-data-table__filter">
                                <span class="sr-only">"Filter by module status"</span>
                                <select
                                    prop:value=move || status.get().value()
                                    on:change=move |event| {
                                        status.set(ModuleDirectoryStatusFilter::from_value(
                                            &event_target_value(&event),
                                        ));
                                        page_index.set(0);
                                    }
                                >
                                    <option value="all">"All statuses"</option>
                                    <option value="active_in_core_process">"Active in Core process"</option>
                                    <option value="unavailable">"Unavailable"</option>
                                    <option value="retired">"Retired"</option>
                                    <option value="ready">"Ready"</option>
                                </select>
                            </label>
                        </div>
                        {move || {
                            let results = filtered_entries.get();
                            if results.is_empty() {
                                view! {
                                    <section class="empty-state module-directory__no-match" aria-live="polite">
                                        <span class="module-state__label">"Directory · filtered"</span>
                                        <h3>"No module definitions match the current filters"</h3>
                                        <p>"Try another module name, definition ID, or status."</p>
                                        <button
                                            class="button button--secondary"
                                            type="button"
                                            on:click=move |_| {
                                                search.set(String::new());
                                                status.set(ModuleDirectoryStatusFilter::All);
                                                page_index.set(0);
                                            }
                                        >
                                            "Clear filters"
                                        </button>
                                    </section>
                                }
                                .into_any()
                            } else {
                                let page_size_value = page_size.get().max(1);
                                let page_count = results.len().max(1).div_ceil(page_size_value);
                                let current_page =
                                    page_index.get().min(page_count.saturating_sub(1));
                                let start = current_page * page_size_value;
                                let end = (start + page_size_value).min(results.len());
                                let page_results = results[start..end].to_vec();
                                let mobile_results = page_results.clone();
                                view! {
                                    <div class="module-directory__table">
                                        <DataTable>
                                            <thead>
                                                <tr>
                                                    <th scope="col">"Module / definition and source"</th>
                                                    <th scope="col">"Availability"</th>
                                                    <th scope="col">"Release / Instance"</th>
                                                    <th scope="col">"Serving state"</th>
                                                    <th scope="col">"Findings"</th>
                                                    <th scope="col" class="data-table__actions">
                                                        <span class="sr-only">"Details"</span>
                                                    </th>
                                                </tr>
                                            </thead>
                                            <tbody>
                                                {page_results.into_iter().map(module_directory_row).collect_view()}
                                            </tbody>
                                        </DataTable>
                                    </div>
                                    <div class="module-directory__mobile-cards">
                                        {mobile_results.into_iter().map(module_directory_mobile_card).collect_view()}
                                    </div>
                                    <TablePaginationFooter
                                        aria_label="Module definitions table pagination"
                                        item_label="module definitions"
                                        empty_item_label="module definitions"
                                        class="module-directory__pagination"
                                        total_count
                                        page_size
                                        page_index
                                    />
                                }
                                .into_any()
                            }
                        }}
                    }
                    .into_any()
                }}
            </section>
        </div>
    }
}

fn module_directory_row(entry: ModuleInventoryEntryV1) -> impl IntoView {
    let serving_state = entry.serving_state();
    if let ModuleInventoryEntryV1::IndependentlyDeployed {
        definition,
        release,
        instance,
        findings,
        ..
    } = entry
    {
        let detail_href = format!("/administration/modules/{}", definition.id);
        let detail_href_row = detail_href.clone();
        let detail_href_key = detail_href.clone();
        let warning = !findings.is_empty();
        return view! {
            <tr class="module-directory__row-link" data-module-definition=definition.id.clone() role="link" tabindex="0"
                on:click=move |_| navigate_to_module_detail(detail_href_row.clone())
                on:keydown=move |event| { if event.key() == "Enter" || event.key() == " " { event.prevent_default(); navigate_to_module_detail(detail_href_key.clone()); } }>
                <th scope="row" class="data-table__stacked-label"><span><a href=detail_href.clone()><strong>{definition.display_name.clone()}</strong></a>{if warning { Some(view! { <span title="Configuration needs review" aria-label="Configuration needs review"><TriangleAlert class="module-directory__configuration-warning"/></span> }) } else { None }}</span><code class="data-table__secondary-text">{definition.id.clone()}</code><span class="module-directory__digest"><code class="data-table__secondary-text">{compact_value(&release.manifest_digest)}</code><CopyValue value=release.manifest_digest label="Copy complete manifest digest"/></span></th>
                <td><span class="status-badge is-success" title="This module runs outside the Core process as its own deployment.">"Independently deployed"</span></td>
                <td><strong>{release.version}</strong><span class="data-table__secondary-text">{format!("Live · {}", compact_value(&instance.id))}</span></td>
                <td><span class=serving_state.badge_class() title=serving_state.explanation()>{serving_state.label()}</span></td>
                <td>{findings.len()}</td>
                <td class="data-table__actions">
                    <a href=detail_href aria-label=format!("Open {} module detail", definition.display_name)>
                        <ChevronRight/>
                    </a>
                </td>
            </tr>
        }.into_any();
    }
    let descriptor = entry.descriptor().clone();
    let definition_id = descriptor.reserved_definition_id.clone();
    let detail_href = format!("/administration/modules/{definition_id}");
    let finding_count = entry.findings().len();
    let source_digest = entry.source_digest().to_string();
    let source_digest_preview = compact_value(&source_digest);
    let source_digest_for_copy = source_digest.clone();
    let detail_href_for_row = detail_href.clone();
    let detail_href_for_key = detail_href.clone();
    let availability_class = match descriptor.availability {
        TransitionAvailabilityV1::ActiveInProcess => "status-badge is-success",
        TransitionAvailabilityV1::Unavailable => "status-badge is-warning",
        TransitionAvailabilityV1::Retired => "status-badge is-danger",
    };

    view! {
        <tr
            class="module-directory__row-link"
            data-module-definition=definition_id.clone()
            role="link"
            tabindex="0"
            on:click=move |_| navigate_to_module_detail(detail_href_for_row.clone())
            on:keydown=move |event| {
                if event.key() == "Enter" || event.key() == " " {
                    event.prevent_default();
                    navigate_to_module_detail(detail_href_for_key.clone());
                }
            }
        >
            <th scope="row" class="data-table__stacked-label">
                <a href=detail_href.clone()><strong>{descriptor.display_name.clone()}</strong></a>
                <code class="data-table__secondary-text">{definition_id.clone()}</code>
                <span class="module-directory__digest">
                    <code class="data-table__secondary-text">{source_digest_preview}</code>
                    <span class="sr-only data-table__secondary-text">{source_digest.clone()}</span>
                    <CopyValue value=source_digest_for_copy label="Copy complete source digest"/>
                </span>
            </th>
            <td>
                <span class=availability_class>{descriptor.availability.label()}</span>
            </td>
            <td>
                <span>{NO_MODULE_RELEASE_LABEL}</span>
                <span class="data-table__secondary-text">{NO_MODULE_INSTANCE_LABEL}</span>
            </td>
            <td><span class=serving_state.badge_class() title=serving_state.explanation()>{serving_state.label()}</span></td>
            <td>{finding_count}</td>
            <td class="data-table__actions">
                <a href=detail_href aria-label=format!("Open {} module detail", descriptor.display_name)>
                    <ChevronRight/>
                </a>
            </td>
        </tr>
    }.into_any()
}

#[component]
pub(super) fn CopyValue(value: String, label: &'static str) -> impl IntoView {
    let value_for_copy = value.clone();
    view! {
        <button
            class="icon-button module-directory__copy"
            type="button"
            aria-label=label
            title=label
            on:click=move |event| {
                event.stop_propagation();
                copy_value(value_for_copy.clone());
            }
        >
            <Copy/>
        </button>
    }
}

pub(super) fn compact_value(value: &str) -> String {
    let value = value.strip_prefix("sha256:").unwrap_or(value);
    if value.chars().count() > 13 {
        let start = value.chars().take(8).collect::<String>();
        let end = value
            .chars()
            .rev()
            .take(5)
            .collect::<String>()
            .chars()
            .rev()
            .collect::<String>();
        format!("{start}…{end}")
    } else {
        value.to_string()
    }
}

#[cfg(feature = "hydrate")]
fn copy_value(value: String) {
    if let Some(window) = web_sys::window() {
        let clipboard = window.navigator().clipboard();
        wasm_bindgen_futures::spawn_local(async move {
            let _ = wasm_bindgen_futures::JsFuture::from(clipboard.write_text(&value)).await;
        });
    }
}

#[cfg(not(feature = "hydrate"))]
fn copy_value(_value: String) {}

#[cfg(feature = "hydrate")]
fn navigate_to_module_detail(href: String) {
    if let Some(window) = web_sys::window() {
        let _ = window.location().set_href(&href);
    }
}

#[cfg(not(feature = "hydrate"))]
fn navigate_to_module_detail(_href: String) {}

fn module_directory_mobile_card(entry: ModuleInventoryEntryV1) -> impl IntoView {
    let serving_state = entry.serving_state();
    if let ModuleInventoryEntryV1::IndependentlyDeployed {
        definition,
        release,
        findings,
        ..
    } = entry
    {
        let detail_href = format!("/administration/modules/{}", definition.id);
        return view! {
            <article class="module-directory-card" data-module-definition=definition.id.clone()>
                <header><div><h3><a href=detail_href.clone()>{definition.display_name}</a></h3><code>{definition.id.clone()}</code></div><a class="module-directory-card__detail" href=detail_href aria-label="Open module detail"><ChevronRight/></a></header>
                <div class="module-directory-card__badges"><span class="status-badge is-success" title="This module runs outside the Core process as its own deployment.">"Independently deployed"</span><span class=serving_state.badge_class() title=serving_state.explanation()>{serving_state.label()}</span></div>
                <p>{release.version} " · Live instance"</p><footer><span>{format!("{} findings", findings.len())}</span><span><code>{compact_value(&release.manifest_digest)}</code><CopyValue value=release.manifest_digest label="Copy complete manifest digest"/></span></footer>
            </article>
        }.into_any();
    }
    let descriptor = entry.descriptor().clone();
    let definition_id = descriptor.reserved_definition_id.clone();
    let detail_href = format!("/administration/modules/{definition_id}");
    let detail_href_for_heading = detail_href.clone();
    let display_name = descriptor.display_name.clone();
    let finding_count = entry.findings().len();
    let source_digest = entry.source_digest().to_string();
    let source_digest_preview = compact_value(&source_digest);
    let source_digest_for_copy = source_digest.clone();
    let finding_label = if finding_count == 1 {
        "finding"
    } else {
        "findings"
    };
    let availability_class = match descriptor.availability {
        TransitionAvailabilityV1::ActiveInProcess => "status-badge is-success",
        TransitionAvailabilityV1::Unavailable => "status-badge is-warning",
        TransitionAvailabilityV1::Retired => "status-badge is-danger",
    };

    view! {
        <article class="module-directory-card" data-module-definition=definition_id.clone()>
            <header>
                <div>
                    <h3><a href=detail_href_for_heading>{display_name.clone()}</a></h3>
                    <span class="module-directory-card__identity">
                        <code>{definition_id.clone()}</code>
                        <CopyValue value=definition_id.clone() label="Copy module definition ID"/>
                    </span>
                </div>
                <a
                    class="module-directory-card__detail"
                    href=detail_href
                    aria-label=format!("Open {display_name} module detail")
                >
                    <ChevronRight/>
                </a>
            </header>
            <div class="module-directory-card__badges">
                <span class="status-badge is-info">"Transitional"</span>
                <span class=availability_class>{descriptor.availability.label()}</span>
            </div>
            <p class="module-directory-card__release">
                {NO_MODULE_RELEASE_LABEL} " · " {NO_MODULE_INSTANCE_LABEL}
            </p>
            <footer>
                <span>{format!("{finding_count} {finding_label}")}</span>
                <span class="module-directory-card__digest">
                    <code>{source_digest_preview}</code>
                    <CopyValue value=source_digest_for_copy label="Copy complete source digest"/>
                </span>
            </footer>
        </article>
    }
    .into_any()
}

#[cfg(test)]
mod tests {
    use leptos::prelude::*;
    use serde_json::json;

    use super::{
        ModuleDirectoryStatusFilter, ModuleInventoryDirectory, NO_MODULE_INSTANCE_LABEL,
        NO_MODULE_RELEASE_LABEL, TRANSITION_PRESENTATION_LABEL, compact_value,
        module_matches_filters,
    };
    use crate::features::modules::models::{
        ApplicationInstallationV1, CoreRuntimeObservationV1, ModuleInventoryEntryV1,
        ModuleInventoryResponseV1, TransitionalContributionDescriptorV1,
    };

    fn inventory() -> ModuleInventoryResponseV1 {
        let descriptor: TransitionalContributionDescriptorV1 = serde_json::from_str(include_str!(
            "../../../../tessara-module-contract/tests/fixtures/transition-forms-v1.json"
        ))
        .expect("fixture parses");
        let entry: ModuleInventoryEntryV1 = serde_json::from_value(json!({
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
        .expect("entry parses");
        ModuleInventoryResponseV1 {
            deployment: None,
            deployment_history: Vec::new(),
            schema_version: 1,
            installation: ApplicationInstallationV1 {
                id: "installation-1".into(),
                created_at: "2026-07-14T12:00:00Z".into(),
            },
            core_runtime: CoreRuntimeObservationV1 {
                provenance: "unresolved_release_provenance".into(),
                observed_version: "0.1.0".into(),
                finding_code: "core_release_provenance_unresolved".into(),
                observed_at: "2026-07-14T12:00:01Z".into(),
            },
            entries: vec![entry],
        }
    }

    #[test]
    fn directory_html_never_turns_a_transition_into_installed_state() {
        let html = Owner::new()
            .with(|| view! { <ModuleInventoryDirectory inventory=inventory()/> }.to_html());

        assert!(html.contains(TRANSITION_PRESENTATION_LABEL));
        assert!(html.contains(NO_MODULE_RELEASE_LABEL));
        assert!(html.contains(NO_MODULE_INSTANCE_LABEL));
        assert!(!html.contains("Install module"));
        assert!(!html.contains("Enable module"));
        assert!(html.contains("tessara.forms"));
        assert!(html.contains("71bebdd0…b493e"));
        assert!(html.contains("Copy complete source digest"));
        assert!(html.contains("href=\"/administration/modules/tessara.forms\""));
        assert!(html.contains("aria-label=\"Open Forms module detail\""));
        assert!(html.contains("Module definitions table pagination"));
        assert!(html.contains("Showing 1-1 of 1 module definitions"));
        assert!(html.contains("Runtime details"));
        assert!(!html.contains("<details class=\"module-runtime-details\""));
    }

    #[test]
    fn empty_inventory_has_an_explicit_empty_state() {
        let mut inventory = inventory();
        inventory.entries.clear();
        let html = Owner::new().with(|| view! { <ModuleInventoryDirectory inventory/> }.to_html());

        assert!(html.contains("No module definitions"));
        assert!(html.contains("No transition contributions are currently registered."));
        assert!(html.contains("Directory · empty"));
        assert!(!html.contains("Module Management restricted"));
    }

    #[test]
    fn populated_directory_exposes_filter_controls() {
        let html = Owner::new()
            .with(|| view! { <ModuleInventoryDirectory inventory=inventory()/> }.to_html());

        assert!(html.contains("Search module name or ID"));
        assert!(html.contains("All statuses"));
        assert!(html.contains("Active in Core process"));
        assert!(html.contains("Unavailable"));
        assert!(html.contains("Retired"));
    }

    #[test]
    fn directory_filters_trim_case_fold_combine_and_preserve_entry_identity() {
        let entry = inventory().entries.remove(0);

        assert!(module_matches_filters(
            &entry,
            "  FoRmS  ",
            ModuleDirectoryStatusFilter::All,
        ));
        assert!(module_matches_filters(
            &entry,
            " TESSARA.FORMS ",
            ModuleDirectoryStatusFilter::ActiveInCoreProcess,
        ));
        assert!(!module_matches_filters(
            &entry,
            "forms",
            ModuleDirectoryStatusFilter::Retired,
        ));
        assert!(!module_matches_filters(
            &entry,
            "migration",
            ModuleDirectoryStatusFilter::ActiveInCoreProcess,
        ));
    }

    #[test]
    fn compact_value_uses_the_directory_digest_presentation() {
        assert_eq!(
            compact_value(
                "sha256:71bebdd07ff0028cc0da8bbd9707c393bade9951e5cedb265a4b8465d54b493e"
            ),
            "71bebdd0…b493e"
        );
        assert_eq!(compact_value("short-value"), "short-value");
    }
}
