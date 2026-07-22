//! Group-aware navigation composer for reader and manager modes.

use icons::{
    ArrowDown, ArrowUp, Blocks, ChevronRight, CircleAlert, CircleCheck, CircleHelp, Database,
    Ellipsis, Eye, EyeOff, File, FileText, GitBranch, House, Info, LayoutDashboard, ListChecks,
    Lock, PanelRight, Pencil, Plus, Trash2,
};
use leptos::prelude::*;
use tessara_web_ui::ModalDialog;
#[cfg(feature = "hydrate")]
use wasm_bindgen::{JsCast, closure::Closure};

use super::api::{ModuleManagementClientError, fetch_navigation_policy, put_navigation_policy};
use super::models::{
    ModuleManagementAccessV1, NavigationContributionDeclarationV1, NavigationDestinationV2,
    NavigationGroupOwnerV2, NavigationGroupV2, NavigationPolicyResponseV2,
};
use crate::state::session::{refresh_shell_navigation, shell_navigation_state};
use crate::ui::DropdownMenu;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PolicyMove {
    Earlier,
    Later,
}

pub fn ordered_groups(policy: &NavigationPolicyResponseV2) -> Vec<NavigationGroupV2> {
    let mut groups = policy.groups.clone();
    groups.sort_by(|left, right| {
        left.order
            .cmp(&right.order)
            .then_with(|| left.id.cmp(&right.id))
    });
    groups
}

pub fn destinations_for_group(
    policy: &NavigationPolicyResponseV2,
    group_id: &str,
) -> Vec<NavigationDestinationV2> {
    let mut destinations = policy
        .destinations
        .iter()
        .filter(|destination| destination.group_id == group_id)
        .cloned()
        .collect::<Vec<_>>();
    destinations.sort_by(|left, right| {
        left.order
            .cmp(&right.order)
            .then_with(|| left.id.cmp(&right.id))
    });
    destinations
}

pub fn set_destination_visibility(
    policy: &mut NavigationPolicyResponseV2,
    destination_id: &str,
    visible: bool,
) -> bool {
    let Some(destination) = policy
        .destinations
        .iter_mut()
        .find(|candidate| candidate.id == destination_id)
    else {
        return false;
    };
    if !destination.can_hide || destination.visible == visible {
        return false;
    }
    destination.visible = visible;
    true
}

pub fn move_destination(
    policy: &mut NavigationPolicyResponseV2,
    destination_id: &str,
    direction: PolicyMove,
) -> bool {
    let Some(current) = policy
        .destinations
        .iter()
        .find(|candidate| candidate.id == destination_id)
        .cloned()
    else {
        return false;
    };
    let mut siblings = destinations_for_group(policy, &current.group_id)
        .into_iter()
        .map(|destination| destination.id)
        .collect::<Vec<_>>();
    let Some(index) = siblings.iter().position(|id| id == destination_id) else {
        return false;
    };
    let target = match direction {
        PolicyMove::Earlier => index.checked_sub(1),
        PolicyMove::Later => (index + 1 < siblings.len()).then_some(index + 1),
    };
    let Some(target) = target else {
        return false;
    };
    siblings.swap(index, target);
    for (order, id) in siblings.iter().enumerate() {
        if let Some(destination) = policy
            .destinations
            .iter_mut()
            .find(|candidate| candidate.id == *id)
        {
            destination.order = order as i32;
        }
    }
    true
}

pub fn move_destination_to_group(
    policy: &mut NavigationPolicyResponseV2,
    destination_id: &str,
    group_id: &str,
) -> bool {
    if !policy.groups.iter().any(|group| group.id == group_id) {
        return false;
    }
    let Some(current) = policy
        .destinations
        .iter()
        .find(|candidate| candidate.id == destination_id)
        .cloned()
    else {
        return false;
    };
    if !current.can_move_between_groups || current.group_id == group_id {
        return false;
    }
    let old_group = current.group_id;
    let next_order = policy
        .destinations
        .iter()
        .filter(|candidate| candidate.group_id == group_id)
        .count() as i32;
    if let Some(destination) = policy
        .destinations
        .iter_mut()
        .find(|candidate| candidate.id == destination_id)
    {
        destination.group_id = group_id.to_string();
        destination.order = next_order;
    }
    redensify_group(policy, &old_group);
    true
}

pub fn move_group(
    policy: &mut NavigationPolicyResponseV2,
    group_id: &str,
    direction: PolicyMove,
) -> bool {
    let mut groups = ordered_groups(policy)
        .into_iter()
        .map(|group| group.id)
        .collect::<Vec<_>>();
    let Some(index) = groups.iter().position(|id| id == group_id) else {
        return false;
    };
    let target = match direction {
        PolicyMove::Earlier => index.checked_sub(1),
        PolicyMove::Later => (index + 1 < groups.len()).then_some(index + 1),
    };
    let Some(target) = target else {
        return false;
    };
    groups.swap(index, target);
    for (order, id) in groups.iter().enumerate() {
        if let Some(group) = policy.groups.iter_mut().find(|group| group.id == *id) {
            group.order = order as i32;
        }
    }
    true
}

pub fn add_custom_group(
    policy: &mut NavigationPolicyResponseV2,
    id: String,
    label: String,
) -> bool {
    if policy.groups.iter().any(|group| group.id == id)
        || policy
            .groups
            .iter()
            .any(|group| group.label.eq_ignore_ascii_case(&label))
    {
        return false;
    }
    policy.groups.push(NavigationGroupV2 {
        id,
        label,
        order: policy.groups.len() as i32,
        owner: NavigationGroupOwnerV2::Custom,
        can_rename: true,
        can_move: true,
        can_delete: true,
    });
    true
}

pub fn delete_custom_group(policy: &mut NavigationPolicyResponseV2, group_id: &str) -> bool {
    let Some(group) = policy.groups.iter().find(|group| group.id == group_id) else {
        return false;
    };
    if group.owner != NavigationGroupOwnerV2::Custom
        || policy
            .destinations
            .iter()
            .any(|destination| destination.group_id == group_id)
    {
        return false;
    }
    policy.groups.retain(|group| group.id != group_id);
    let ordered = ordered_groups(policy);
    for (order, ordered_group) in ordered.iter().enumerate() {
        if let Some(group) = policy
            .groups
            .iter_mut()
            .find(|group| group.id == ordered_group.id)
        {
            group.order = order as i32;
        }
    }
    true
}

fn redensify_group(policy: &mut NavigationPolicyResponseV2, group_id: &str) {
    let ordered = destinations_for_group(policy, group_id);
    for (order, ordered_destination) in ordered.iter().enumerate() {
        if let Some(destination) = policy
            .destinations
            .iter_mut()
            .find(|destination| destination.id == ordered_destination.id)
        {
            destination.order = order as i32;
        }
    }
}

#[component]
pub fn ModuleNavigationPolicyView(
    policy: RwSignal<Option<NavigationPolicyResponseV2>>,
    persisted_policy: RwSignal<Option<NavigationPolicyResponseV2>>,
    unavailable_message: RwSignal<Option<String>>,
    access: ModuleManagementAccessV1,
    #[prop(optional)] definition_id: Option<String>,
    #[prop(optional)] declared_navigation: Vec<NavigationContributionDeclarationV1>,
) -> impl IntoView {
    let is_saving = RwSignal::new(false);
    let is_dirty = RwSignal::new(false);
    let has_conflict = RwSignal::new(false);
    let message = RwSignal::new(None::<String>);
    let active_mobile_destination_actions = RwSignal::new(None::<String>);
    let shell_navigation = shell_navigation_state();
    let can_manage = access.may_manage_navigation()
        && policy
            .get_untracked()
            .is_some_and(|value| value.can_manage_navigation);

    let save = move |_| {
        if !can_manage || is_saving.get_untracked() {
            return;
        }
        let Some(draft) = policy.get_untracked() else {
            return;
        };
        is_saving.set(true);
        message.set(None);
        leptos::task::spawn_local(async move {
            match put_navigation_policy(&draft).await {
                Ok(saved) => {
                    policy.set(Some(saved.clone()));
                    persisted_policy.set(Some(saved));
                    is_dirty.set(false);
                    has_conflict.set(false);
                    refresh_shell_navigation(shell_navigation);
                    message.set(Some("Navigation policy saved.".into()));
                }
                Err(error) => {
                    has_conflict.set(matches!(error, ModuleManagementClientError::Conflict(_)));
                    message.set(Some(error.display_message()));
                }
            }
            is_saving.set(false);
        });
    };
    let discard = move |_| {
        policy.set(persisted_policy.get_untracked());
        is_dirty.set(false);
        has_conflict.set(false);
        message.set(None);
    };
    let reload = move |_| {
        if is_saving.get_untracked() {
            return;
        }
        is_saving.set(true);
        message.set(None);
        leptos::task::spawn_local(async move {
            match fetch_navigation_policy().await {
                Ok(current) => {
                    policy.set(Some(current.clone()));
                    persisted_policy.set(Some(current));
                    is_dirty.set(false);
                    has_conflict.set(false);
                    message.set(Some("Current navigation policy reloaded.".into()));
                }
                Err(error) => message.set(Some(error.display_message())),
            }
            is_saving.set(false);
        });
    };

    view! {
        <section class="organization-detail-card module-navigation-policy" aria-labelledby="module-navigation-policy-heading">
            <h2 id="module-navigation-policy-heading" class="sr-only">"Navigation policy"</h2>

            {(!declared_navigation.is_empty()).then(|| descriptor_declarations(declared_navigation))}

            {move || {
                if let Some(unavailable) = unavailable_message.get() {
                    view! {
                        <section class="organization-state module-navigation-policy__localized-state" aria-live="polite">
                            <span class="module-state__label">"Navigation · localized"</span>
                            <h3>"Navigation policy unavailable"</h3>
                            <p>{unavailable}</p>
                            <button
                                class="button button--warning"
                                type="button"
                                disabled=move || is_saving.get()
                                on:click=reload
                            >"Retry navigation"</button>
                        </section>
                    }.into_any()
                } else if let Some(current) = policy.get() {
                    let groups = ordered_groups(&current);
                    let revision = current.revision;
                    let visible_destinations = current
                        .destinations
                        .iter()
                        .filter(|destination| destination.visible)
                        .count();
                    let detail_definition_id = definition_id.clone();
                    let may_edit_composition = can_manage && detail_definition_id.is_none();
                    view! {
                        <div class="module-navigation-policy__content">
                            <div class="module-navigation-policy__toolbar">
                                {may_edit_composition.then(|| view! {
                                    <button
                                        id="navigation-add-group"
                                        class="button module-navigation-policy__add-group"
                                        type="button"
                                        on:click=move |_| {
                                            policy.update(|current| {
                                                let Some(current) = current else { return; };
                                                let index = current.groups.len() + 1;
                                                if add_custom_group(
                                                    current,
                                                    new_custom_group_id(),
                                                    format!("Group {index}"),
                                                ) {
                                                    is_dirty.set(true);
                                                    message.set(None);
                                                }
                                            });
                                        }
                                    ><Plus class="button__icon"/>"Add group"</button>
                                })}
                            </div>
                            <section class="module-navigation-policy__info-strip" aria-label="Navigation policy summary">
                                <Info class="module-navigation-policy__info-icon"/>
                                <span>{format!("Revision {revision} · {} groups · {visible_destinations} visible destinations", groups.len())}</span>
                                <span class="module-navigation-policy__info-disclosure">"Display configuration does not grant access."</span>
                                {(!can_manage).then(|| view! { <span class="status-badge is-info">"Read-only"</span> })}
                            </section>
                            {move || has_conflict.get().then(|| view! {
                                <section class="module-navigation-conflict" role="alert" aria-labelledby="navigation-conflict-heading">
                                    <CircleAlert class="module-navigation-conflict__icon"/>
                                    <div class="module-navigation-conflict__copy">
                                        <h3 id="navigation-conflict-heading">"Navigation changed elsewhere"</h3>
                                        <p>"Your draft is based on an earlier policy revision. Reload the current policy before making further changes; saving will not overwrite newer changes."</p>
                                        <ul class="module-navigation-conflict__protections">
                                            <li>"Main and Admin are required groups and cannot be deleted."</li>
                                            <li>"Home is protected and cannot be hidden or moved from its required placement."</li>
                                        </ul>
                                    </div>
                                    <div class="module-navigation-conflict__actions">
                                        <button
                                            class="button button--secondary"
                                            type="button"
                                            disabled=move || is_saving.get()
                                            on:click=discard
                                        >"Discard local draft"</button>
                                        <button
                                            class="button button--secondary"
                                            type="button"
                                            disabled=move || is_saving.get()
                                            on:click=reload
                                        >"Reload current policy"</button>
                                    </div>
                                </section>
                            })}
                            <div class="module-navigation-groups">
                                {groups.into_iter().filter_map(|group| {
                                    let rows = destinations_for_group(&current, &group.id)
                                        .into_iter()
                                        .filter(|destination| detail_definition_id.as_ref().is_none_or(|id| destination.definition_id.as_ref() == Some(id)))
                                        .collect::<Vec<_>>();
                                    if detail_definition_id.is_some() && rows.is_empty() {
                                        None
                                    } else {
                                        Some(group_view(
                                            group,
                                            rows,
                                            policy,
                                            is_dirty,
                                            message,
                                            may_edit_composition,
                                            active_mobile_destination_actions,
                                        ))
                                    }
                                }).collect_view()}
                            </div>
                            {may_edit_composition.then(|| view! {
                                <div class="module-navigation-policy__action-bar">
                                    <span class="module-navigation-policy__dirty-state">
                                        {move || (has_conflict.get() || is_dirty.get()).then(|| view! {
                                            <CircleAlert class="module-navigation-policy__dirty-icon"/>
                                        })}
                                        <span>{move || if has_conflict.get() { "Revision conflict — reload the current policy before saving" } else if is_dirty.get() { "Unsaved navigation changes" } else { "No unsaved changes" }}</span>
                                    </span>
                                    <div class="form-actions">
                                        {move || (!has_conflict.get()).then(|| view! {
                                            <button
                                                class="button button--secondary"
                                                type="button"
                                                disabled=move || !is_dirty.get() || is_saving.get()
                                                on:click=discard
                                            >"Discard changes"</button>
                                        })}
                                        <button
                                            class="button"
                                            type="button"
                                            disabled=move || !is_dirty.get() || is_saving.get() || has_conflict.get()
                                            on:click=save
                                        >{move || if is_saving.get() { "Saving…" } else { "Save navigation" }}</button>
                                    </div>
                                </div>
                            })}
                        </div>
                    }.into_any()
                } else {
                    view! {
                        <section class="organization-state" aria-live="polite">
                            <h3>"Loading navigation policy"</h3>
                            <p>"Fetching the current group and placement policy."</p>
                        </section>
                    }.into_any()
                }
            }}
            {move || {
                if is_saving.get() {
                    return Some(
                        view! {
                            <section class="module-navigation-save-state is-saving" aria-live="polite" aria-busy="true">
                                <span class="module-state__label">"Navigation · saving"</span>
                                <div>
                                    <h3>"Saving navigation…"</h3>
                                    <p>"The current policy revision is being updated atomically."</p>
                                </div>
                            </section>
                        }
                        .into_any(),
                    );
                }

                if message.get().as_deref() == Some("Navigation policy saved.") {
                    let revision = policy.get().map(|current| current.revision).unwrap_or_default();
                    return Some(
                        view! {
                            <section class="module-navigation-save-state is-saved" aria-live="polite">
                                <CircleCheck class="module-navigation-save-state__icon"/>
                                <div>
                                    <span class="module-state__label">"Navigation · saved"</span>
                                    <h3>"Navigation saved"</h3>
                                    <p>{format!("Revision {revision} is now active.")}</p>
                                </div>
                                <button
                                    class="button button--secondary"
                                    type="button"
                                    on:click=move |_| message.set(None)
                                >"Continue"</button>
                            </section>
                        }
                        .into_any(),
                    );
                }

                None
            }}
            {move || {
                (!has_conflict.get())
                    .then(|| message.get())
                    .flatten()
                    .filter(|message| {
                        !message.trim().is_empty() && message != "Navigation policy saved."
                    })
                    .map(|message| view! {
                        <p class="form-message" aria-live="polite">{message}</p>
                    })
            }}
        </section>
    }
}

fn descriptor_declarations(
    declared_navigation: Vec<NavigationContributionDeclarationV1>,
) -> impl IntoView {
    view! {
        <section aria-labelledby="module-declared-navigation-heading">
            <h3 id="module-declared-navigation-heading">"Descriptor declarations"</h3>
            <ul class="module-metadata-list">
                {declared_navigation.into_iter().map(|declaration| view! {
                    <li data-navigation-declaration=declaration.id.clone()>
                        <strong>{declaration.label}</strong>
                        <code>{declaration.destination}</code>
                        <span class="data-table__secondary-text">
                            {format!("Discovery hint: {} group, source order {}", declaration.group, declaration.order_hint)}
                        </span>
                        <ul class="module-navigation-eligibility">
                            {declaration.required_capabilities_any_of.into_iter().map(|capability| view! {
                                <li><code>{capability}</code></li>
                            }).collect_view()}
                        </ul>
                    </li>
                }).collect_view()}
            </ul>
        </section>
    }
}

fn group_view(
    group: NavigationGroupV2,
    rows: Vec<NavigationDestinationV2>,
    policy: RwSignal<Option<NavigationPolicyResponseV2>>,
    is_dirty: RwSignal<bool>,
    message: RwSignal<Option<String>>,
    can_manage: bool,
    active_mobile_destination_actions: RwSignal<Option<String>>,
) -> impl IntoView {
    let group_id = group.id.clone();
    let group_id_for_earlier = group.id.clone();
    let group_id_for_later = group.id.clone();
    let group_id_for_delete = group.id.clone();
    let group_id_for_rename = RwSignal::new(group.id.clone());
    let label = group.label.clone();
    let is_custom = group.owner == NavigationGroupOwnerV2::Custom;
    let can_rename = can_manage && is_custom && group.can_rename;
    let can_manage_custom_group = can_manage && is_custom;
    let initially_open = group.id != "core.admin";
    let has_items = !rows.is_empty();
    let delete_blocker_message = match rows.as_slice() {
        [] => None,
        [destination] => Some(format!(
            "Move {} before deleting this group.",
            destination.label
        )),
        _ => Some("Move all destinations before deleting this group.".to_string()),
    };
    let group_slug = group.id.replace(['.', ':'], "-");
    let earlier_control_id = format!("navigation-group-{group_slug}-earlier");
    let later_control_id = format!("navigation-group-{group_slug}-later");
    let earlier_focus_id = earlier_control_id.clone();
    let later_focus_id = later_control_id.clone();
    let rename_dialog_id = format!("navigation-group-{group_slug}-rename");
    let rename_open = RwSignal::new(false);
    let rename_value = RwSignal::new(group.label.clone());
    let label_for_rename = label.clone();
    view! {
        <details class="module-navigation-group" open=initially_open>
            <summary>
                <span class="module-navigation-group__caret" aria-hidden="true"><ChevronRight/></span>
                <span class="module-navigation-group__identity">
                    <strong>{label.clone()}</strong>
                    <span class="module-navigation-group__context">
                        <span>{if is_custom { "Custom group" } else { "Required group" }}</span>
                        <span aria-hidden="true">" · "</span>
                        <code>{group_id.clone()}</code>
                    </span>
                </span>
                {can_manage.then(|| view! {
                    <span class="module-navigation-group__actions" on:click=|event| event.stop_propagation()>
                        {can_manage_custom_group.then(|| view! {
                            <DropdownMenu label=format!("Open actions for {label}")>
                                {can_rename.then(|| view! {
                                    <button
                                        class="dropdown-menu__item"
                                        type="button"
                                        role="menuitem"
                                        on:click=move |_| {
                                            rename_value.set(label_for_rename.clone());
                                            rename_open.set(true);
                                        }
                                    ><Pencil class="dropdown-menu__item-icon"/><span>"Rename group"</span></button>
                                })}
                                <button
                                    class="dropdown-menu__item dropdown-menu__item--danger"
                                    type="button"
                                    role="menuitem"
                                    disabled=has_items
                                    title=delete_blocker_message.clone().unwrap_or_else(|| "Delete empty group".to_string())
                                    on:click=move |_| mark_change(
                                        policy,
                                        is_dirty,
                                        message,
                                        |current| delete_custom_group(current, &group_id_for_delete),
                                    )
                                >
                                    <Trash2 class="dropdown-menu__item-icon"/>
                                    <span>"Delete group"</span>
                                </button>
                                {delete_blocker_message.map(|message| view! {
                                    <p class="module-navigation-group__delete-hint" role="note">{message}</p>
                                })}
                            </DropdownMenu>
                        })}
                        <button
                            id=earlier_control_id
                            class="icon-button module-navigation-icon-button"
                            type="button"
                            aria-label=format!("Move {label} group up")
                            title=format!("Move {label} group up")
                            disabled=group.order == 0
                            on:click=move |_| {
                                mark_change(
                                    policy,
                                    is_dirty,
                                    message,
                                    |current| move_group(current, &group_id_for_earlier, PolicyMove::Earlier),
                                );
                                restore_navigation_policy_focus(earlier_focus_id.clone(), "navigation-add-group".into());
                            }
                        ><ArrowUp class="icon-button__icon"/></button>
                        <button
                            id=later_control_id
                            class="icon-button module-navigation-icon-button"
                            type="button"
                            aria-label=format!("Move {label} group down")
                            title=format!("Move {label} group down")
                            on:click=move |_| {
                                mark_change(
                                    policy,
                                    is_dirty,
                                    message,
                                    |current| move_group(current, &group_id_for_later, PolicyMove::Later),
                                );
                                restore_navigation_policy_focus(later_focus_id.clone(), "navigation-add-group".into());
                            }
                        ><ArrowDown class="icon-button__icon"/></button>
                    </span>
                })}
            </summary>
            {can_rename.then(|| view! {
                <ModalDialog
                    id=rename_dialog_id
                    title=format!("Rename {label}")
                    description="Update the display label for this custom navigation group."
                    open=Signal::derive(move || rename_open.get())
                    on_close=Callback::new(move |_| rename_open.set(false))
                    close_label="Close rename dialog"
                    class="module-navigation-rename-dialog"
                >
                    <form on:submit=move |event| {
                        event.prevent_default();
                        let next = rename_value.get_untracked().trim().to_string();
                        if next.is_empty() {
                            return;
                        }
                        let group_id_for_update = group_id_for_rename.get_untracked();
                        policy.update(|current| {
                            if let Some(group) = current.as_mut().and_then(|current| current.groups.iter_mut().find(|group| group.id == group_id_for_update))
                                && group.label != next
                            {
                                group.label = next;
                                is_dirty.set(true);
                                message.set(None);
                            }
                        });
                        rename_open.set(false);
                    }>
                        <label class="form-field">
                            <span>"Group name"</span>
                            <input
                                id=format!("{group_slug}-group-name")
                                type="text"
                                maxlength="64"
                                prop:value=move || rename_value.get()
                                on:input=move |event| rename_value.set(event_target_value(&event))
                            />
                        </label>
                        <div class="form-actions module-navigation-rename-dialog__actions">
                            <button class="button button--secondary" type="button" on:click=move |_| rename_open.set(false)>"Cancel"</button>
                            <button class="button" type="submit" disabled=move || rename_value.get().trim().is_empty()>"Save name"</button>
                        </div>
                    </form>
                </ModalDialog>
            })}
            {if rows.is_empty() {
                view! { <p class="empty-state">"This group has no destinations."</p> }.into_any()
            } else {
                view! {
                    <div class="module-navigation-items" aria-label=format!("{label} navigation destinations")>
                        <div class="module-navigation-items__header">
                            <span>"#"</span>
                            <span aria-hidden="true"></span>
                            <span>"Destination"</span>
                            <span>"Owner"</span>
                            <span>"Route"</span>
                            <span>"Visible"</span>
                            <span>"Order / placement"</span>
                        </div>
                        {rows.into_iter().map(|row| destination_view(
                            row,
                            policy,
                            is_dirty,
                            message,
                            can_manage,
                            active_mobile_destination_actions,
                        )).collect_view()}
                    </div>
                }.into_any()
            }}
        </details>
    }
}

fn destination_view(
    row: NavigationDestinationV2,
    policy: RwSignal<Option<NavigationPolicyResponseV2>>,
    is_dirty: RwSignal<bool>,
    message: RwSignal<Option<String>>,
    can_manage: bool,
    active_mobile_destination_actions: RwSignal<Option<String>>,
) -> impl IntoView {
    let destination_id = row.id.clone();
    let destination_id_for_action = row.id.clone();
    let destination_id_for_action_expanded = row.id.clone();
    let destination_id_for_action_toggle = row.id.clone();
    let visibility_id = format!("navigation-visible-{}", row.id.replace(['.', ':'], "-"));
    let id_for_visibility = row.id.clone();
    let id_for_mobile_visibility = row.id.clone();
    let id_for_earlier = row.id.clone();
    let id_for_later = row.id.clone();
    let id_for_group = row.id.clone();
    let destination_slug = row.id.replace(['.', ':'], "-");
    let earlier_control_id = format!("navigation-destination-{destination_slug}-earlier");
    let later_control_id = format!("navigation-destination-{destination_slug}-later");
    let group_control_id = format!("navigation-destination-{destination_slug}-group");
    let actions_control_id = format!("navigation-destination-{destination_slug}-actions");
    let earlier_focus_id = earlier_control_id.clone();
    let later_focus_id = later_control_id.clone();
    let group_focus_id = group_control_id.clone();
    let earlier_fallback_id = visibility_id.clone();
    let later_fallback_id = visibility_id.clone();
    let group_fallback_id = visibility_id.clone();
    let label = row.label.clone();
    let owner_label = match row.owner {
        super::models::NavigationDestinationOwnerV2::Core => "Core",
        super::models::NavigationDestinationOwnerV2::Contribution => "Module",
    };
    let route = row.route.clone();
    let groups = policy
        .get_untracked()
        .map(|policy| ordered_groups(&policy))
        .unwrap_or_default();
    // Module Management is a canonical protected placement.  The reader view
    // must retain the same lock treatment even though it does not expose the
    // manager-only action metadata used for the generic protected predicate.
    let protected =
        row.id == "core.admin.modules" || (!row.can_hide && !row.can_move_between_groups);
    let next_visible = !row.visible;
    view! {
        <article class="module-navigation-item" data-navigation-destination=destination_id.clone()>
            <span class="module-navigation-item__order" data-label="#">{row.order + 1}</span>
            <span class="module-navigation-item__mobile-icon" aria-hidden="true">
                {navigation_destination_icon(&row)}
            </span>
            <div class="module-navigation-item__identity">
                <div>
                    <strong>{row.label.clone()}</strong>
                    <code>{row.id.clone()}</code>
                    <code class="module-navigation-item__mobile-route">{route.clone()}</code>
                </div>
            </div>
            <span class="module-navigation-item__owner" data-label="Owner">{owner_label}</span>
            <code class="module-navigation-item__route" data-label="Route">{row.route.clone()}</code>
            {if can_manage {
                view! {
                    {if row.can_hide {
                        view! {
                            <button
                                class=if row.visible { "button module-navigation-visibility is-shown module-navigation-item__mobile-visibility" } else { "button module-navigation-visibility is-hidden module-navigation-item__mobile-visibility" }
                                type="button"
                                aria-pressed=row.visible
                                aria-label=if row.visible { "Shown" } else { "Hidden" }
                                title=if row.visible { "Shown" } else { "Hidden" }
                                on:click=move |_| mark_change(
                                    policy,
                                    is_dirty,
                                    message,
                                    |current| set_destination_visibility(current, &id_for_mobile_visibility, next_visible),
                                )
                            >{if row.visible {
                                view! { <Eye class="icon-button__icon"/> }.into_any()
                            } else {
                                view! { <EyeOff class="icon-button__icon"/> }.into_any()
                            }}</button>
                        }.into_any()
                    } else {
                        view! {
                            <span class="module-navigation-item__mobile-visibility module-navigation-item__mobile-protected" aria-label="Shown; visibility is protected" title="Protected placement"><Lock/></span>
                        }.into_any()
                    }}
                    <div class=move || if active_mobile_destination_actions.get().as_deref() == Some(destination_id_for_action.as_str()) { "module-navigation-item__actions is-open" } else { "module-navigation-item__actions" }>
                        <button
                            class="icon-button module-navigation-item__summary"
                            type="button"
                            aria-expanded=move || active_mobile_destination_actions.get().as_deref() == Some(destination_id_for_action_expanded.as_str())
                            aria-controls=actions_control_id.clone()
                            aria-label=format!("Manage {label}")
                            title=format!("Manage {label}")
                            on:click=move |_| active_mobile_destination_actions.update(|active| {
                                if active.as_deref() == Some(destination_id_for_action_toggle.as_str()) {
                                    *active = None;
                                } else {
                                    *active = Some(destination_id_for_action_toggle.clone());
                                }
                            })
                        ><Ellipsis class="icon-button__icon"/></button>
                        <div id=actions_control_id class="module-navigation-item__controls">
                        <button
                            class="module-navigation-item__sheet-scrim"
                            type="button"
                            aria-label="Close destination actions"
                            on:click=move |_| active_mobile_destination_actions.set(None)
                        ></button>
                        <section class="module-navigation-item__sheet" aria-label=format!("Manage {label}")>
                            <header class="module-navigation-item__sheet-header">
                                <span class="module-navigation-item__sheet-icon" aria-hidden="true">{navigation_destination_icon(&row)}</span>
                                <span>
                                    <strong>{row.label.clone()}</strong>
                                    <span>{format!("{} · {owner_label}", route.clone())}</span>
                                </span>
                            </header>
                        {if row.can_hide {
                            view! {
                                <button
                                    id=visibility_id.clone()
                                    class=if row.visible { "button module-navigation-visibility is-shown module-navigation-item__visibility module-navigation-item__sheet-action" } else { "button module-navigation-visibility is-hidden module-navigation-item__visibility module-navigation-item__sheet-action" }
                                    type="button"
                                    aria-pressed=row.visible
                                    aria-label=if row.visible { "Shown" } else { "Hidden" }
                                    title=if row.visible { "Shown" } else { "Hidden" }
                                    on:click=move |_| mark_change(
                                            policy,
                                            is_dirty,
                                            message,
                                            |current| set_destination_visibility(current, &id_for_visibility, next_visible),
                                        )
                                >{if row.visible {
                                    view! { <Eye class="icon-button__icon"/><span class="module-navigation-item__sheet-action-label">"Hide"</span> }.into_any()
                                } else {
                                    view! { <EyeOff class="icon-button__icon"/><span class="module-navigation-item__sheet-action-label">"Show"</span> }.into_any()
                                }}</button>
                            }.into_any()
                        } else {
                            view! {
                                <p class="module-navigation-item__sheet-protected"><Lock/>"This destination is protected and must remain shown in its required placement."</p>
                            }.into_any()
                        }}
                        <div
                            class=if protected {
                                "module-navigation-item__placement is-protected"
                            } else {
                                "module-navigation-item__placement"
                            }
                            data-label="Order / placement"
                        >
                            {if protected {
                                view! { <span class="module-navigation-item__protected" aria-label="Protected placement" title="Protected placement"><Lock/></span> }.into_any()
                            } else {
                                view! {
                            <label class="form-field module-navigation-item__move-group">
                                <span class="module-navigation-item__mobile-action-label">"Move to another group"</span>
                                <select
                                    id=group_control_id
                                    prop:value=""
                                    aria-label=format!("Move {label} to group")
                                    on:change=move |event| {
                                        let group_id = event_target_value(&event);
                                        if group_id.is_empty() {
                                            return;
                                        }
                                        mark_change(
                                            policy,
                                            is_dirty,
                                            message,
                                            |current| move_destination_to_group(current, &id_for_group, &group_id),
                                        );
                                        restore_navigation_policy_focus(group_focus_id.clone(), group_fallback_id.clone());
                                    }
                                >
                                    <option value="">"Move to…"</option>
                                    {groups.into_iter().filter(|group| group.id != row.group_id).map(|group| view! {
                                        <option value=group.id>{group.label}</option>
                                    }).collect_view()}
                                </select>
                            </label>
                            <button
                            id=earlier_control_id
                            class="icon-button module-navigation-icon-button"
                            type="button"
                            aria-label=format!("Move {label} earlier")
                            title=format!("Move {label} earlier")
                            disabled=row.order == 0
                            on:click=move |_| {
                                mark_change(
                                    policy,
                                    is_dirty,
                                    message,
                                    |current| move_destination(current, &id_for_earlier, PolicyMove::Earlier),
                                );
                                restore_navigation_policy_focus(earlier_focus_id.clone(), earlier_fallback_id.clone());
                            }
                        ><ArrowUp class="icon-button__icon"/><span class="module-navigation-item__mobile-action-label">"Move earlier"</span></button>
                        <button
                            id=later_control_id
                            class="icon-button module-navigation-icon-button"
                            type="button"
                            aria-label=format!("Move {label} later")
                            title=format!("Move {label} later")
                            on:click=move |_| {
                                mark_change(
                                    policy,
                                    is_dirty,
                                    message,
                                    |current| move_destination(current, &id_for_later, PolicyMove::Later),
                                );
                                restore_navigation_policy_focus(later_focus_id.clone(), later_fallback_id.clone());
                            }
                        ><ArrowDown class="icon-button__icon"/><span class="module-navigation-item__mobile-action-label">"Move later"</span></button>
                                }.into_any()
                            }}
                        </div>
                        <button
                            class="button button--secondary module-navigation-item__sheet-close"
                            type="button"
                            on:click=move |_| active_mobile_destination_actions.set(None)
                        >"Close"</button>
                        </section>
                        </div>
                    </div>
                }.into_any()
            } else {
                view! {
                    {if protected {
                        view! {
                            <span
                                class="module-navigation-item__reader-visibility module-navigation-item__protected"
                                data-label="Visible"
                                aria-label="Protected placement"
                                title="Protected placement"
                            ><Lock/></span>
                        }.into_any()
                    } else {
                        view! {
                            <span class="module-navigation-item__reader-visibility" data-label="Visible">
                                {if row.visible { "Shown" } else { "Hidden" }}
                            </span>
                        }.into_any()
                    }}
                    <div class="module-navigation-eligibility" data-label="Eligibility">
                        {if row.required_capabilities_any_of.is_empty() {
                            view! { <span>"Always eligible"</span> }.into_any()
                        } else {
                            view! {
                                <ul>
                                {row.required_capabilities_any_of.into_iter().map(|capability| view! {
                                    <li><code>{capability}</code></li>
                                }).collect_view()}
                                </ul>
                            }.into_any()
                        }}
                    </div>
                }.into_any()
            }}
        </article>
    }
}

fn navigation_destination_icon(destination: &NavigationDestinationV2) -> AnyView {
    match destination.route.as_str() {
        "/" => view! { <House/> }.into_any(),
        "/organization" => view! { <GitBranch/> }.into_any(),
        "/forms" => view! { <FileText/> }.into_any(),
        "/workflows" => view! { <PanelRight/> }.into_any(),
        "/responses" => view! { <CircleHelp/> }.into_any(),
        "/operations" => view! { <ListChecks/> }.into_any(),
        "/datasets" => view! { <Database/> }.into_any(),
        "/dashboards" => view! { <LayoutDashboard/> }.into_any(),
        "/administration/modules" => view! { <Blocks/> }.into_any(),
        _ => view! { <File/> }.into_any(),
    }
}

fn mark_change(
    policy: RwSignal<Option<NavigationPolicyResponseV2>>,
    is_dirty: RwSignal<bool>,
    message: RwSignal<Option<String>>,
    change: impl FnOnce(&mut NavigationPolicyResponseV2) -> bool,
) {
    policy.update(|current| {
        if let Some(current) = current
            && change(current)
        {
            is_dirty.set(true);
            message.set(None);
        }
    });
}

#[cfg(feature = "hydrate")]
fn new_custom_group_id() -> String {
    web_sys::window()
        .and_then(|window| window.crypto().ok())
        .map(|crypto| format!("custom.{}", crypto.random_uuid()))
        .unwrap_or_else(|| "custom.00000000-0000-4000-8000-000000000000".to_string())
}

#[cfg(not(feature = "hydrate"))]
fn new_custom_group_id() -> String {
    "custom.00000000-0000-4000-8000-000000000000".to_string()
}

#[cfg(not(feature = "hydrate"))]
fn restore_navigation_policy_focus(_control_id: String, _fallback_id: String) {}

#[cfg(feature = "hydrate")]
fn restore_navigation_policy_focus(control_id: String, fallback_id: String) {
    let Some(window) = web_sys::window() else {
        return;
    };
    let callback = Closure::once(Box::new(move || {
        let Some(document) = web_sys::window().and_then(|window| window.document()) else {
            return;
        };
        if let Some(element) = [control_id, fallback_id]
            .into_iter()
            .filter_map(|id| document.get_element_by_id(&id))
            .filter_map(|element| element.dyn_into::<web_sys::HtmlElement>().ok())
            .find(|element| !element.has_attribute("disabled"))
        {
            let _ = element.focus();
        }
    }) as Box<dyn FnOnce()>);
    let _ = window.request_animation_frame(callback.as_ref().unchecked_ref());
    callback.forget();
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::features::modules::models::{
        ModuleManagementAccessV1, NavigationDestinationOwnerV2, NavigationGroupOwnerV2,
    };

    fn policy() -> NavigationPolicyResponseV2 {
        NavigationPolicyResponseV2 {
            schema_version: 2,
            installation_id: "installation-1".into(),
            revision: 3,
            can_manage_navigation: true,
            groups: vec![
                NavigationGroupV2 {
                    id: "core.main".into(),
                    label: "Main".into(),
                    order: 0,
                    owner: NavigationGroupOwnerV2::Core,
                    can_rename: false,
                    can_move: true,
                    can_delete: false,
                },
                NavigationGroupV2 {
                    id: "core.admin".into(),
                    label: "Admin".into(),
                    order: 1,
                    owner: NavigationGroupOwnerV2::Core,
                    can_rename: false,
                    can_move: true,
                    can_delete: false,
                },
            ],
            destinations: vec![
                destination("forms", "core.main", 0, true),
                destination("workflows", "core.main", 1, true),
                destination("modules", "core.admin", 0, false),
            ],
        }
    }

    fn destination(id: &str, group_id: &str, order: i32, mutable: bool) -> NavigationDestinationV2 {
        NavigationDestinationV2 {
            id: id.into(),
            key: id.into(),
            label: id.into(),
            route: format!("/{id}"),
            semantic_destination: None,
            definition_id: Some(format!("tessara.{id}")),
            owner: NavigationDestinationOwnerV2::Contribution,
            required_capabilities_any_of: vec![format!("{id}:read")],
            group_id: group_id.into(),
            visible: true,
            order,
            available: true,
            can_hide: mutable,
            can_move_between_groups: mutable,
            can_reorder: true,
        }
    }

    #[test]
    fn destinations_move_within_and_between_groups_without_duplication() {
        let mut policy = policy();
        assert!(move_destination(
            &mut policy,
            "workflows",
            PolicyMove::Earlier
        ));
        assert_eq!(
            destinations_for_group(&policy, "core.main")[0].id,
            "workflows"
        );
        assert!(move_destination_to_group(
            &mut policy,
            "forms",
            "core.admin"
        ));
        assert_eq!(
            policy
                .destinations
                .iter()
                .filter(|item| item.id == "forms")
                .count(),
            1
        );
        assert_eq!(destinations_for_group(&policy, "core.main").len(), 1);
        assert_eq!(destinations_for_group(&policy, "core.admin").len(), 2);
    }

    #[test]
    fn protected_destination_rejects_hide_and_cross_group_move() {
        let mut policy = policy();
        assert!(!set_destination_visibility(&mut policy, "modules", false));
        assert!(!move_destination_to_group(
            &mut policy,
            "modules",
            "core.main"
        ));
    }

    #[test]
    fn manager_navigation_uses_the_compact_strip_and_explicit_placement_controls() {
        let html = Owner::new().with(|| {
            let current = policy();
            let policy = RwSignal::new(Some(current.clone()));
            let persisted_policy = RwSignal::new(Some(current));
            let unavailable_message = RwSignal::new(None);
            view! {
                <ModuleNavigationPolicyView
                    policy
                    persisted_policy
                    unavailable_message
                    access=ModuleManagementAccessV1::manager()
                />
            }
            .to_html()
        });

        assert!(html.contains("Revision 3 · 2 groups · 3 visible destinations"));
        assert!(html.contains("Display configuration does not grant access."));
        assert!(html.contains("Order / placement"));
        assert!(html.contains("Move to…"));
        assert!(html.contains("module-navigation-visibility is-shown"));
        assert!(html.contains("Move forms earlier"));
        assert!(html.contains("Required group"));
        assert!(html.contains("core.main"));
        assert!(html.contains(">Module<"));
        assert!(!html.contains("Core owned"));
        assert!(!html.contains("Module contribution"));
        assert!(!html.contains("form-message"));
    }

    #[test]
    fn nonempty_custom_group_must_be_emptied_before_deletion() {
        let mut policy = policy();
        assert!(add_custom_group(
            &mut policy,
            "custom.123e4567-e89b-42d3-a456-426614174000".into(),
            "Insights".into(),
        ));
        assert!(move_destination_to_group(
            &mut policy,
            "forms",
            "custom.123e4567-e89b-42d3-a456-426614174000",
        ));
        assert!(!delete_custom_group(
            &mut policy,
            "custom.123e4567-e89b-42d3-a456-426614174000",
        ));
        assert!(move_destination_to_group(&mut policy, "forms", "core.main"));
        assert!(delete_custom_group(
            &mut policy,
            "custom.123e4567-e89b-42d3-a456-426614174000",
        ));
    }

    #[test]
    fn manager_custom_group_uses_an_overflow_menu_without_an_inline_editor() {
        let mut current = policy();
        assert!(add_custom_group(
            &mut current,
            "custom.123e4567-e89b-42d3-a456-426614174000".into(),
            "Insights".into(),
        ));
        assert!(move_destination_to_group(
            &mut current,
            "forms",
            "custom.123e4567-e89b-42d3-a456-426614174000",
        ));

        let html = Owner::new().with(|| {
            let policy = RwSignal::new(Some(current.clone()));
            let persisted_policy = RwSignal::new(Some(current));
            let unavailable_message = RwSignal::new(None);
            view! {
                <ModuleNavigationPolicyView
                    policy
                    persisted_policy
                    unavailable_message
                    access=ModuleManagementAccessV1::manager()
                />
            }
            .to_html()
        });

        assert!(html.contains("Open actions for Insights"));
        assert!(html.contains("Rename group"));
        assert!(html.contains("Delete group"));
        assert!(html.contains("Move forms before deleting this group."));
        assert!(!html.contains("Group name"));
    }
}
