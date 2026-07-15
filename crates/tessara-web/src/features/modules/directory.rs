//! Module Management directory presentation.

use leptos::prelude::*;

use super::models::{ModuleInventoryEntryV1, ModuleInventoryResponseV1, TransitionAvailabilityV1};
use crate::ui::{DataTable, EmptyState};

pub const TRANSITION_PRESENTATION_LABEL: &str = "Transitional — not independently deployable";
pub const NO_MODULE_RELEASE_LABEL: &str = "No Module Release";
pub const NO_MODULE_INSTANCE_LABEL: &str = "No Module Instance";

#[component]
pub fn ModuleInventoryDirectory(inventory: ModuleInventoryResponseV1) -> impl IntoView {
    let installation = inventory.installation;
    let core_runtime = inventory.core_runtime;
    let entries = inventory.entries;

    view! {
        <div class="module-management-directory">
            <section class="organization-detail-card" aria-labelledby="module-core-runtime-heading">
                <h2 id="module-core-runtime-heading">"Core runtime context"</h2>
                <p>
                    "This installation is the stable owner of current in-process transition resources. "
                    "The observed runtime is not presented as a fabricated exact Core Release."
                </p>
                <dl class="organization-detail-list">
                    <div>
                        <dt>"Application Installation"</dt>
                        <dd><code>{installation.id}</code></dd>
                    </div>
                    <div>
                        <dt>"Installation created"</dt>
                        <dd><time datetime=installation.created_at.clone()>{installation.created_at.clone()}</time></dd>
                    </div>
                    <div>
                        <dt>"Observed Core version"</dt>
                        <dd>{core_runtime.observed_version}</dd>
                    </div>
                    <div>
                        <dt>"Release provenance"</dt>
                        <dd>{core_runtime.provenance}</dd>
                    </div>
                    <div>
                        <dt>"Runtime finding"</dt>
                        <dd><code>{core_runtime.finding_code}</code></dd>
                    </div>
                    <div>
                        <dt>"Observed at"</dt>
                        <dd><time datetime=core_runtime.observed_at.clone()>{core_runtime.observed_at.clone()}</time></dd>
                    </div>
                </dl>
            </section>

            <section class="organization-detail-card" aria-labelledby="module-directory-heading">
                <div class="module-directory__heading">
                    <div>
                        <h2 id="module-directory-heading">"Module inventory"</h2>
                        <p>"Inspect Core-owned transition contributions and their exact descriptor provenance."</p>
                    </div>
                    <span aria-live="polite">{format!("{} contributions", entries.len())}</span>
                </div>

                {if entries.is_empty() {
                    view! {
                        <EmptyState
                            title="No module contributions"
                            message="The current installation returned an empty module inventory."
                        />
                    }
                    .into_any()
                } else {
                    view! {
                        <DataTable>
                            <thead>
                                <tr>
                                    <th scope="col">"Contribution"</th>
                                    <th scope="col">"Type"</th>
                                    <th scope="col">"Availability"</th>
                                    <th scope="col">"Release / Instance"</th>
                                    <th scope="col">"Findings"</th>
                                </tr>
                            </thead>
                            <tbody>
                                {entries.into_iter().map(module_directory_row).collect_view()}
                            </tbody>
                        </DataTable>
                    }
                    .into_any()
                }}
            </section>
        </div>
    }
}

fn module_directory_row(entry: ModuleInventoryEntryV1) -> impl IntoView {
    let descriptor = entry.descriptor().clone();
    let definition_id = descriptor.reserved_definition_id.clone();
    let detail_href = format!("/administration/modules/{definition_id}");
    let finding_count = entry.findings().len();
    let source_digest = entry.source_digest().to_string();
    let availability_class = match descriptor.availability {
        TransitionAvailabilityV1::ActiveInProcess => "status-badge is-success",
        TransitionAvailabilityV1::Unavailable => "status-badge is-warning",
        TransitionAvailabilityV1::Retired => "status-badge is-info",
    };

    view! {
        <tr data-module-definition=definition_id.clone()>
            <th scope="row" class="data-table__stacked-label">
                <a href=detail_href><strong>{descriptor.display_name}</strong></a>
                <code class="data-table__secondary-text">{definition_id.clone()}</code>
                <span class="data-table__secondary-text">{source_digest}</span>
            </th>
            <td>
                <span class="status-badge is-info">{TRANSITION_PRESENTATION_LABEL}</span>
            </td>
            <td>
                <span class=availability_class>{descriptor.availability.label()}</span>
                <span class="data-table__secondary-text">{descriptor.availability.explanation()}</span>
            </td>
            <td>
                <span>{NO_MODULE_RELEASE_LABEL}</span>
                <span class="data-table__secondary-text">{NO_MODULE_INSTANCE_LABEL}</span>
            </td>
            <td>{if finding_count == 1 {
                "1 finding".to_string()
            } else {
                format!("{finding_count} findings")
            }}</td>
        </tr>
    }
}

#[cfg(test)]
mod tests {
    use leptos::prelude::*;
    use serde_json::json;

    use super::{
        ModuleInventoryDirectory, NO_MODULE_INSTANCE_LABEL, NO_MODULE_RELEASE_LABEL,
        TRANSITION_PRESENTATION_LABEL,
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
    }

    #[test]
    fn empty_inventory_has_an_explicit_empty_state() {
        let mut inventory = inventory();
        inventory.entries.clear();
        let html = Owner::new().with(|| view! { <ModuleInventoryDirectory inventory/> }.to_html());

        assert!(html.contains("No module contributions"));
        assert!(html.contains("empty module inventory"));
        assert!(!html.contains("Module Management restricted"));
    }
}
