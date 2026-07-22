//! Role editor sheet.

use super::super::state::toggle_string_selection;
use crate::features::administration::display::admin_capability_scope_label;
use crate::features::administration::models::{
    AdminCapabilityScopeMode, AdminCapabilitySummary, AdminRoleCapabilityScopeSelection,
    MIXED_ROLE_CAPABILITY_SCOPE_MESSAGE, admin_role_capability_scope_selection,
};
use crate::utils::text::text_matches;
use icons::{Search, X};
use leptos::portal::Portal;
use leptos::prelude::*;

use super::capability_metadata::AdminCapabilityProvenance;

#[component]
pub(crate) fn AdminRoleSheet(
    is_open: RwSignal<bool>,
    editing_role_id: RwSignal<Option<String>>,
    role_name: RwSignal<String>,
    capabilities: RwSignal<Vec<AdminCapabilitySummary>>,
    selected_capability_ids: RwSignal<Vec<String>>,
    capability_search: RwSignal<String>,
    is_saving: RwSignal<bool>,
    message: RwSignal<Option<String>>,
    on_close: impl Fn(leptos::ev::MouseEvent) + 'static + Copy + Send + Sync,
    on_save: impl Fn(leptos::ev::MouseEvent) + 'static + Copy + Send + Sync,
) -> impl IntoView {
    view! {
        <Portal>
            <Show when=move || is_open.get()>
                <section class="sheet-overlay administration-role-overlay" aria-label="Role editor">
                    <button class="sheet-overlay__scrim" type="button" aria-label="Close role editor" on:click=on_close></button>
                    <aside class="sheet-panel blurred-surface administration-role-sheet" role="dialog" aria-modal="true" aria-label="Role editor">
                        <div class="sheet-panel__actions">
                            <button class="icon-button sheet-panel__close" type="button" aria-label="Close role editor" title="Close role editor" on:click=on_close>
                                <X/>
                            </button>
                        </div>
                        <header class="sheet-panel__header">
                            <p>"Role Template"</p>
                            <h2>{move || if editing_role_id.get().is_some() { "Edit Role Capabilities" } else { "New Role" }}</h2>
                        </header>
                        <section class="sheet-panel__section">
                            <Show when=move || editing_role_id.get().is_none()>
                                <label class="form-field">
                                    <span>"Role Name"</span>
                                    <input
                                        type="text"
                                        placeholder="coordinator"
                                        prop:value=move || role_name.get()
                                        on:input=move |event| role_name.set(event_target_value(&event))
                                    />
                                </label>
                            </Show>
                            <section class="administration-role-scope-guidance" aria-labelledby="administration-role-scope-heading">
                                <h3 id="administration-role-scope-heading">"Capability scope"</h3>
                                <p>"Keep scope modes in separate roles. Installation-global roles are assigned across the entire installation. A user can have a dedicated global module role alongside separate scoped product roles. admin:all is the sole mixed-scope exception and makes the complete role installation-global."</p>
                            </section>
                            <label class="searchable-data-table__search searchable-data-table__control administration-role-sheet__search">
                                <Search class="searchable-data-table__control-icon"/>
                                <span class="sr-only">"Search capabilities"</span>
                                <input
                                    type="search"
                                    placeholder="Search capabilities"
                                    prop:value=move || capability_search.get()
                                    on:input=move |event| capability_search.set(event_target_value(&event))
                                />
                            </label>
                            <div class="checkbox-list permission-picker__list administration-role-capability-picker">
                                {move || {
                                    let query = capability_search.get();
                                    let selected = selected_capability_ids.get();
                                    let catalog = capabilities.get();
                                    let visible = visible_role_capabilities(&catalog, &selected, &query);
                                    if visible.is_empty() {
                                        view! { <p class="forms-list-mobile-empty">"No Capabilities to Display"</p> }.into_any()
                                    } else {
                                        visible
                                            .into_iter()
                                            .map(|capability| {
                                                let capability_id = capability.id.clone();
                                                let input_id = format!("administration-role-capability-{}", capability.id);
                                                let metadata_id = format!("{input_id}-metadata");
                                                let checked = selected.iter().any(|id| id == &capability.id);
                                                let scope_mode = capability.scope_mode;
                                                let scope_label = admin_capability_scope_label(scope_mode);
                                                let provenance = capability.provenance;
                                                view! {
                                                    <div class="checkbox-list__item permission-picker__item administration-role-capability-entry">
                                                        <input
                                                            id=input_id.clone()
                                                            type="checkbox"
                                                            prop:checked=checked
                                                            aria-describedby=metadata_id.clone()
                                                            on:change=move |event| {
                                                                toggle_string_selection(
                                                                    selected_capability_ids,
                                                                    capability_id.clone(),
                                                                    event_target_checked(&event),
                                                                );
                                                            }
                                                        />
                                                        <div class="administration-role-capability-content">
                                                            <label for=input_id>
                                                                <span class="administration-role-capability-heading">
                                                                    <strong>{capability.key}</strong>
                                                                    <small class="administration-role-capability-scope">{scope_label}</small>
                                                                </span>
                                                                <small>{capability.description}</small>
                                                            </label>
                                                            <div id=metadata_id class="administration-role-capability-metadata">
                                                                <small>"Provenance"</small>
                                                                <AdminCapabilityProvenance provenance show_digest=false/>
                                                            </div>
                                                        </div>
                                                    </div>
                                                }
                                            })
                                            .collect_view()
                                            .into_any()
                                    }
                                }}
                            </div>
                            {move || {
                                let selection = admin_role_capability_scope_selection(
                                    &capabilities.get(),
                                    &selected_capability_ids.get(),
                                );
                                view! { <AdminRoleScopeSelectionNotice selection/> }
                            }}
                            <Show when=move || message.get().is_some()>
                                <p class="form-message" role="status">{move || message.get().unwrap_or_default()}</p>
                            </Show>
                        </section>
                        <div class="form-actions">
                            <button class="button button--secondary" type="button" on:click=on_close>
                                "Cancel"
                            </button>
                            <button
                                class="button"
                                type="button"
                                disabled=move || {
                                    is_saving.get()
                                        || admin_role_capability_scope_selection(
                                            &capabilities.get(),
                                            &selected_capability_ids.get(),
                                        )
                                        .is_invalid()
                                }
                                on:click=on_save
                            >
                                {move || if is_saving.get() { "Saving..." } else { "Save Role" }}
                            </button>
                        </div>
                    </aside>
                </section>
            </Show>
        </Portal>
    }
}

/// Returns the capability choices that remain valid for the role's current
/// scope mode. Once a role has selected its first ordinary capability, the
/// picker retains only that scope mode. This keeps either invalid mixed-scope
/// path out of the editor rather than waiting until save to reject it. Empty
/// roles and the `admin:all` exception retain the complete catalog.
fn visible_role_capabilities(
    catalog: &[AdminCapabilitySummary],
    selected_capability_ids: &[String],
    query: &str,
) -> Vec<AdminCapabilitySummary> {
    let admin_all_selected = catalog.iter().any(|capability| {
        capability.key == "admin:all"
            && selected_capability_ids
                .iter()
                .any(|selected_id| selected_id == &capability.id)
    });
    let allowed_scope =
        match admin_role_capability_scope_selection(catalog, selected_capability_ids) {
            AdminRoleCapabilityScopeSelection::ScopeAware => {
                Some(AdminCapabilityScopeMode::ScopeAware)
            }
            AdminRoleCapabilityScopeSelection::InstallationGlobal => {
                Some(AdminCapabilityScopeMode::InstallationGlobal)
            }
            AdminRoleCapabilityScopeSelection::Empty
            | AdminRoleCapabilityScopeSelection::AdminAllMixedException
            | AdminRoleCapabilityScopeSelection::Mixed => None,
        };

    catalog
        .iter()
        .filter(|capability| {
            let scope_matches = admin_all_selected
                || capability.key == "admin:all"
                || match allowed_scope {
                    Some(scope_mode) => capability.scope_mode == scope_mode,
                    None => true,
                };

            scope_matches
                && text_matches(
                    query,
                    &[capability.key.as_str(), capability.description.as_str()],
                )
        })
        .cloned()
        .collect()
}

#[component]
fn AdminRoleScopeSelectionNotice(selection: AdminRoleCapabilityScopeSelection) -> impl IntoView {
    let (heading, detail, is_error) = match selection {
        AdminRoleCapabilityScopeSelection::Empty => (
            "No scope mode selected",
            "Choose capabilities from one scope mode to define this role.",
            false,
        ),
        AdminRoleCapabilityScopeSelection::ScopeAware => (
            "Scope-aware role",
            "Assignments for this role may be limited to selected organization scope nodes.",
            false,
        ),
        AdminRoleCapabilityScopeSelection::InstallationGlobal => (
            "Installation-global role",
            "Assignments for this role always apply across the entire installation; organization scope nodes do not limit it.",
            false,
        ),
        AdminRoleCapabilityScopeSelection::AdminAllMixedException => (
            "Global admin exception",
            "This role contains admin:all, the sole mixed-scope exception. The complete role is installation-global; additional product capabilities are redundant.",
            false,
        ),
        AdminRoleCapabilityScopeSelection::Mixed => (
            "Mixed scope modes are not allowed",
            MIXED_ROLE_CAPABILITY_SCOPE_MESSAGE,
            true,
        ),
    };
    let class = if is_error {
        "administration-role-scope-selection form-message administration-role-scope-selection--error"
    } else {
        "administration-role-scope-selection"
    };
    let role = if is_error { "alert" } else { "status" };

    view! {
        <section class=class role=role aria-live="polite">
            <strong>{heading}</strong>
            <p>{detail}</p>
        </section>
    }
}

#[cfg(test)]
mod tests {
    use leptos::prelude::*;

    use super::{AdminRoleScopeSelectionNotice, visible_role_capabilities};
    use crate::features::administration::models::{
        AdminCapabilityScopeMode, AdminCapabilitySummary, AdminRoleCapabilityScopeSelection,
    };

    fn capability(
        id: &str,
        key: &str,
        scope_mode: AdminCapabilityScopeMode,
    ) -> AdminCapabilitySummary {
        AdminCapabilitySummary {
            id: id.into(),
            key: key.into(),
            description: key.into(),
            scope_mode,
            provenance: Vec::new(),
        }
    }

    #[test]
    fn scope_selection_notice_explains_global_and_rejects_mixed_roles() {
        let html = Owner::new().with(|| {
            view! {
                <div>
                    <AdminRoleScopeSelectionNotice
                        selection=AdminRoleCapabilityScopeSelection::InstallationGlobal
                    />
                    <AdminRoleScopeSelectionNotice
                        selection=AdminRoleCapabilityScopeSelection::AdminAllMixedException
                    />
                    <AdminRoleScopeSelectionNotice
                        selection=AdminRoleCapabilityScopeSelection::Mixed
                    />
                </div>
            }
            .to_html()
        });

        assert!(html.contains("Installation-global role"));
        assert!(html.contains("always apply across the entire installation"));
        assert!(html.contains("Global admin exception"));
        assert!(html.contains("additional product capabilities are redundant"));
        assert!(html.contains("Mixed scope modes are not allowed"));
        assert!(html.contains("dedicated installation-global role for module permissions"));
        assert!(html.contains("role=\"alert\""));
    }

    #[test]
    fn scope_aware_role_picker_omits_installation_global_capabilities() {
        let catalog = vec![
            capability(
                "forms-read",
                "forms:read",
                AdminCapabilityScopeMode::ScopeAware,
            ),
            capability(
                "modules-read",
                "modules:read",
                AdminCapabilityScopeMode::InstallationGlobal,
            ),
        ];

        let visible = visible_role_capabilities(&catalog, &["forms-read".into()], "");
        assert_eq!(
            visible
                .iter()
                .map(|capability| capability.key.as_str())
                .collect::<Vec<_>>(),
            vec!["forms:read"]
        );
    }

    #[test]
    fn installation_global_role_picker_omits_scope_aware_capabilities() {
        let catalog = vec![
            capability(
                "forms-read",
                "forms:read",
                AdminCapabilityScopeMode::ScopeAware,
            ),
            capability(
                "modules-read",
                "modules:read",
                AdminCapabilityScopeMode::InstallationGlobal,
            ),
        ];

        let visible = visible_role_capabilities(&catalog, &["modules-read".into()], "");
        assert_eq!(
            visible
                .iter()
                .map(|capability| capability.key.as_str())
                .collect::<Vec<_>>(),
            vec!["modules:read"]
        );
    }
}
