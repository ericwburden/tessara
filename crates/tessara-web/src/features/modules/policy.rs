//! Group-aware navigation composer for reader and manager modes.

use icons::Lock;
use leptos::prelude::*;
#[cfg(feature = "hydrate")]
use wasm_bindgen::{JsCast, closure::Closure};

use super::api::{ModuleManagementClientError, fetch_navigation_policy, put_navigation_policy};
use super::models::{
    ModuleManagementAccessV1, NavigationContributionDeclarationV1, NavigationDestinationV2,
    NavigationGroupOwnerV2, NavigationGroupV2, NavigationPolicyResponseV2,
};

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
            <div class="module-navigation-policy__heading">
                <div>
                    <h2 id="module-navigation-policy-heading">"Navigation"</h2>
                    <p>"Display configuration does not grant access. Route authorization remains authoritative."</p>
                </div>
                {(!can_manage).then(|| view! { <span class="status-badge is-info">"Read-only"</span> })}
            </div>

            {(!declared_navigation.is_empty()).then(|| descriptor_declarations(declared_navigation))}

            {move || {
                if let Some(unavailable) = unavailable_message.get() {
                    view! {
                        <section class="organization-state" aria-live="polite">
                            <h3>"Navigation policy unavailable"</h3>
                            <p>{unavailable}</p>
                        </section>
                    }.into_any()
                } else if let Some(current) = policy.get() {
                    let groups = ordered_groups(&current);
                    let revision = current.revision;
                    let detail_definition_id = definition_id.clone();
                    let may_edit_composition = can_manage && detail_definition_id.is_none();
                    view! {
                        <div class="module-navigation-policy__content">
                            <div class="module-navigation-policy__summary">
                                <p>{format!("Policy revision {revision}")}</p>
                                {may_edit_composition.then(|| view! {
                                    <button
                                        id="navigation-add-group"
                                        class="button button--secondary"
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
                                    >"Add group"</button>
                                })}
                            </div>
                            <div class="module-navigation-groups">
                                {groups.into_iter().filter_map(|group| {
                                    let rows = destinations_for_group(&current, &group.id)
                                        .into_iter()
                                        .filter(|destination| detail_definition_id.as_ref().is_none_or(|id| destination.definition_id.as_ref() == Some(id)))
                                        .collect::<Vec<_>>();
                                    if detail_definition_id.is_some() && rows.is_empty() {
                                        None
                                    } else {
                                        Some(group_view(group, rows, policy, is_dirty, message, may_edit_composition))
                                    }
                                }).collect_view()}
                            </div>
                            {may_edit_composition.then(|| view! {
                                <div class="module-navigation-policy__action-bar">
                                    <span>{move || if has_conflict.get() { "Revision conflict — reload the current policy before saving" } else if is_dirty.get() { "Unsaved navigation changes" } else { "No unsaved changes" }}</span>
                                    <div class="form-actions">
                                        <button
                                            class="button button--secondary"
                                            type="button"
                                            disabled=move || !is_dirty.get() || is_saving.get()
                                            on:click=discard
                                        >{move || if has_conflict.get() { "Discard local draft" } else { "Discard changes" }}</button>
                                        {move || has_conflict.get().then(|| view! {
                                            <button
                                                class="button button--secondary"
                                                type="button"
                                                disabled=move || is_saving.get()
                                                on:click=reload
                                            >"Reload current policy"</button>
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
            <p class="form-message" aria-live="polite">{move || message.get().unwrap_or_default()}</p>
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
) -> impl IntoView {
    let group_id = group.id.clone();
    let group_id_for_label = group.id.clone();
    let group_id_for_earlier = group.id.clone();
    let group_id_for_later = group.id.clone();
    let group_id_for_delete = group.id.clone();
    let label = group.label.clone();
    let is_custom = group.owner == NavigationGroupOwnerV2::Custom;
    let has_items = !rows.is_empty();
    let group_slug = group.id.replace(['.', ':'], "-");
    let earlier_control_id = format!("navigation-group-{group_slug}-earlier");
    let later_control_id = format!("navigation-group-{group_slug}-later");
    let earlier_focus_id = earlier_control_id.clone();
    let later_focus_id = later_control_id.clone();
    view! {
        <details class="module-navigation-group" open>
            <summary>
                <span>
                    <strong>{label.clone()}</strong>
                    <code>{group_id.clone()}</code>
                    <small>{if is_custom { "Custom group" } else { "Required group" }}</small>
                </span>
                {can_manage.then(|| view! {
                    <span class="module-navigation-group__actions" on:click=|event| event.stop_propagation()>
                        <button
                            id=earlier_control_id
                            class="button button--quiet"
                            type="button"
                            aria-label=format!("Move {label} group up")
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
                        >"Move up"</button>
                        <button
                            id=later_control_id
                            class="button button--quiet"
                            type="button"
                            aria-label=format!("Move {label} group down")
                            on:click=move |_| {
                                mark_change(
                                    policy,
                                    is_dirty,
                                    message,
                                    |current| move_group(current, &group_id_for_later, PolicyMove::Later),
                                );
                                restore_navigation_policy_focus(later_focus_id.clone(), "navigation-add-group".into());
                            }
                        >"Move down"</button>
                        {is_custom.then(|| view! {
                            <button
                                class="button button--quiet button--danger"
                                type="button"
                                disabled=has_items
                                title=if has_items { "Move every destination out of this group before deleting it." } else { "Delete empty group" }
                                on:click=move |_| mark_change(
                                    policy,
                                    is_dirty,
                                    message,
                                    |current| delete_custom_group(current, &group_id_for_delete),
                                )
                            >"Delete"</button>
                        })}
                    </span>
                })}
            </summary>
            {if can_manage && is_custom {
                view! {
                    <label class="form-field module-navigation-group__rename">
                        <span>"Group name"</span>
                        <input
                            type="text"
                            maxlength="64"
                            prop:value=group.label
                            on:input=move |event| {
                                let next = event_target_value(&event);
                                policy.update(|current| {
                                    if let Some(group) = current.as_mut().and_then(|current| current.groups.iter_mut().find(|group| group.id == group_id_for_label))
                                        && group.label != next
                                    {
                                        group.label = next.clone();
                                        is_dirty.set(true);
                                        message.set(None);
                                    }
                                });
                            }
                        />
                    </label>
                }.into_any()
            } else {
                ().into_any()
            }}
            {if rows.is_empty() {
                view! { <p class="empty-state">"This group has no destinations."</p> }.into_any()
            } else {
                view! {
                    <div class="module-navigation-items">
                        {rows.into_iter().map(|row| destination_view(row, policy, is_dirty, message, can_manage)).collect_view()}
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
) -> impl IntoView {
    let destination_id = row.id.clone();
    let visibility_id = format!("navigation-visible-{}", row.id.replace(['.', ':'], "-"));
    let id_for_visibility = row.id.clone();
    let id_for_earlier = row.id.clone();
    let id_for_later = row.id.clone();
    let id_for_group = row.id.clone();
    let destination_slug = row.id.replace(['.', ':'], "-");
    let earlier_control_id = format!("navigation-destination-{destination_slug}-earlier");
    let later_control_id = format!("navigation-destination-{destination_slug}-later");
    let group_control_id = format!("navigation-destination-{destination_slug}-group");
    let actions_control_id = format!("navigation-destination-{destination_slug}-actions");
    let actions_expanded = RwSignal::new(false);
    let earlier_focus_id = earlier_control_id.clone();
    let later_focus_id = later_control_id.clone();
    let group_focus_id = group_control_id.clone();
    let earlier_fallback_id = visibility_id.clone();
    let later_fallback_id = visibility_id.clone();
    let group_fallback_id = visibility_id.clone();
    let label = row.label.clone();
    let groups = policy
        .get_untracked()
        .map(|policy| ordered_groups(&policy))
        .unwrap_or_default();
    let protected = !row.can_hide && !row.can_move_between_groups;
    view! {
        <article class="module-navigation-item" data-navigation-destination=destination_id>
            <div class="module-navigation-item__identity">
                <div>
                    <strong>{row.label}</strong>
                    <code>{row.id.clone()}</code>
                    <span>{row.route}</span>
                </div>
                {protected.then(|| view! {
                    <span class="module-navigation-item__lock" aria-label="Protected placement" title="Protected placement">
                        <Lock/>
                    </span>
                })}
            </div>
            <div class="module-navigation-item__metadata">
                <span>{match row.owner {
                    super::models::NavigationDestinationOwnerV2::Core => "Core owned",
                    super::models::NavigationDestinationOwnerV2::Contribution => "Module contribution",
                }}</span>
                <span>{if row.available { "Available" } else { "Unavailable" }}</span>
                {if can_manage {
                    ().into_any()
                } else if row.required_capabilities_any_of.is_empty() {
                    view! { <span>"Always eligible"</span> }.into_any()
                } else {
                    view! {
                        <div class="module-navigation-eligibility">
                            <strong>"Any of"</strong>
                            <ul>
                            {row.required_capabilities_any_of.into_iter().map(|capability| view! {
                                <li><code>{capability}</code></li>
                            }).collect_view()}
                            </ul>
                        </div>
                    }.into_any()
                }}
            </div>
            {if can_manage {
                view! {
                    <div class=move || if actions_expanded.get() { "module-navigation-item__actions is-open" } else { "module-navigation-item__actions" }>
                        <button
                            class="button button--secondary module-navigation-item__summary"
                            type="button"
                            aria-expanded=move || actions_expanded.get()
                            aria-controls=actions_control_id.clone()
                            on:click=move |_| actions_expanded.update(|expanded| *expanded = !*expanded)
                        >"Manage "{label.clone()}</button>
                        <div id=actions_control_id class="module-navigation-item__controls">
                        {if row.can_hide {
                            view! {
                                <label for=visibility_id.clone()>
                                    <input
                                        id=visibility_id.clone()
                                        type="checkbox"
                                        prop:checked=row.visible
                                        on:change=move |event| mark_change(
                                            policy,
                                            is_dirty,
                                            message,
                                            |current| set_destination_visibility(current, &id_for_visibility, event_target_checked(&event)),
                                        )
                                    />
                                    " Show"
                                </label>
                            }.into_any()
                        } else {
                            ().into_any()
                        }}
                        <button
                            id=earlier_control_id
                            class="button button--quiet"
                            type="button"
                            aria-label=format!("Move {label} earlier")
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
                        >"Move earlier"</button>
                        <button
                            id=later_control_id
                            class="button button--quiet"
                            type="button"
                            aria-label=format!("Move {label} later")
                            on:click=move |_| {
                                mark_change(
                                    policy,
                                    is_dirty,
                                    message,
                                    |current| move_destination(current, &id_for_later, PolicyMove::Later),
                                );
                                restore_navigation_policy_focus(later_focus_id.clone(), later_fallback_id.clone());
                            }
                        >"Move later"</button>
                        {if row.can_move_between_groups {
                            view! {
                                <label class="form-field module-navigation-item__move-group">
                                    <span>"Move to group"</span>
                                    <select
                                        id=group_control_id
                                        prop:value=row.group_id
                                        on:change=move |event| {
                                            let group_id = event_target_value(&event);
                                            mark_change(
                                                policy,
                                                is_dirty,
                                                message,
                                                |current| move_destination_to_group(current, &id_for_group, &group_id),
                                            );
                                            restore_navigation_policy_focus(group_focus_id.clone(), group_fallback_id.clone());
                                        }
                                    >
                                        {groups.into_iter().map(|group| view! {
                                            <option value=group.id>{group.label}</option>
                                        }).collect_view()}
                                    </select>
                                </label>
                            }.into_any()
                        } else {
                            ().into_any()
                        }}
                        </div>
                    </div>
                }.into_any()
            } else {
                view! { <p>{if row.visible { "Shown" } else { "Hidden" }}</p> }.into_any()
            }}
        </article>
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
    use crate::features::modules::models::{NavigationDestinationOwnerV2, NavigationGroupOwnerV2};

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
}
