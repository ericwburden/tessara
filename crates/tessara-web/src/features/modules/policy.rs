//! Navigation-policy presentation and bounded editing.

use leptos::prelude::*;
#[cfg(feature = "hydrate")]
use wasm_bindgen::{JsCast, closure::Closure};

use super::api::put_navigation_policy;
use super::models::{
    ModuleManagementAccessV1, NavigationContributionDeclarationV1, NavigationPolicyContributionV1,
    NavigationPolicyResponseV1,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PolicyMove {
    Earlier,
    Later,
}

pub fn ordered_contributions(
    policy: &NavigationPolicyResponseV1,
) -> Vec<NavigationPolicyContributionV1> {
    let mut contributions = policy.contributions.clone();
    contributions.sort_by(|left, right| {
        left.group
            .cmp(&right.group)
            .then_with(|| left.reorder_band.cmp(&right.reorder_band))
            .then_with(|| left.order.cmp(&right.order))
            .then_with(|| left.id.cmp(&right.id))
    });
    contributions
}

pub fn set_contribution_visibility(
    policy: &mut NavigationPolicyResponseV1,
    contribution_id: &str,
    visible: bool,
) -> bool {
    let Some(contribution) = policy
        .contributions
        .iter_mut()
        .find(|candidate| candidate.id == contribution_id)
    else {
        return false;
    };
    if contribution.visible == visible {
        return false;
    }
    contribution.visible = visible;
    true
}

pub fn can_move_contribution(
    policy: &NavigationPolicyResponseV1,
    contribution_id: &str,
    direction: PolicyMove,
) -> bool {
    let Some(current) = policy
        .contributions
        .iter()
        .find(|candidate| candidate.id == contribution_id)
    else {
        return false;
    };
    let band_len = policy
        .contributions
        .iter()
        .filter(|candidate| candidate.reorder_band == current.reorder_band)
        .count();
    match direction {
        PolicyMove::Earlier => current.order > 0,
        PolicyMove::Later => (current.order as usize) + 1 < band_len,
    }
}

/// Moves one contribution exactly one slot inside its immutable Core-assigned
/// band and re-densifies that band. No group or band field is changed.
pub fn move_contribution(
    policy: &mut NavigationPolicyResponseV1,
    contribution_id: &str,
    direction: PolicyMove,
) -> bool {
    let Some(current) = policy
        .contributions
        .iter()
        .find(|candidate| candidate.id == contribution_id)
        .cloned()
    else {
        return false;
    };
    let mut band = policy
        .contributions
        .iter()
        .filter(|candidate| candidate.reorder_band == current.reorder_band)
        .map(|candidate| candidate.id.clone())
        .collect::<Vec<_>>();
    band.sort_by(|left_id, right_id| {
        let left = policy
            .contributions
            .iter()
            .find(|candidate| candidate.id == *left_id)
            .expect("band id belongs to policy");
        let right = policy
            .contributions
            .iter()
            .find(|candidate| candidate.id == *right_id)
            .expect("band id belongs to policy");
        left.order
            .cmp(&right.order)
            .then_with(|| left.id.cmp(&right.id))
    });
    let Some(index) = band.iter().position(|id| id == contribution_id) else {
        return false;
    };
    let target = match direction {
        PolicyMove::Earlier => index.checked_sub(1),
        PolicyMove::Later => (index + 1 < band.len()).then_some(index + 1),
    };
    let Some(target) = target else {
        return false;
    };
    band.swap(index, target);
    for (order, id) in band.iter().enumerate() {
        if let Some(contribution) = policy
            .contributions
            .iter_mut()
            .find(|candidate| candidate.id == *id)
        {
            contribution.order = order as i32;
        }
    }
    true
}

#[component]
pub fn ModuleNavigationPolicyView(
    policy: RwSignal<Option<NavigationPolicyResponseV1>>,
    persisted_policy: RwSignal<Option<NavigationPolicyResponseV1>>,
    unavailable_message: RwSignal<Option<String>>,
    access: ModuleManagementAccessV1,
    #[prop(optional)] definition_id: Option<String>,
    #[prop(optional)] declared_navigation: Vec<NavigationContributionDeclarationV1>,
) -> impl IntoView {
    let is_saving = RwSignal::new(false);
    let is_dirty = RwSignal::new(false);
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
                    message.set(Some("Navigation policy saved.".into()));
                }
                Err(error) => message.set(Some(error.display_message())),
            }
            is_saving.set(false);
        });
    };
    let discard = move |_| {
        policy.set(persisted_policy.get_untracked());
        is_dirty.set(false);
        message.set(None);
    };

    view! {
        <section class="organization-detail-card module-navigation-policy" aria-labelledby="module-navigation-policy-heading">
            <div class="module-navigation-policy__heading">
                <div>
                    <h2 id="module-navigation-policy-heading">"Navigation"</h2>
                    <p>"Display choices do not grant access. Route authorization remains authoritative."</p>
                </div>
                {(!can_manage).then(|| view! {
                    <span class="status-badge is-info">"Read-only"</span>
                })}
            </div>

            {(!declared_navigation.is_empty()).then(|| view! {
                <section aria-labelledby="module-declared-navigation-heading">
                    <h3 id="module-declared-navigation-heading">"Descriptor declarations"</h3>
                    <ul class="module-metadata-list">
                        {declared_navigation.into_iter().map(|declaration| view! {
                            <li data-navigation-declaration=declaration.id.clone()>
                                <strong>{declaration.label}</strong>
                                " — " <code>{declaration.destination}</code>
                                <span class="data-table__secondary-text">
                                    {format!("{} group; source order hint {}", declaration.group, declaration.order_hint)}
                                </span>
                                <span class="data-table__secondary-text">
                                    {format!("Eligible with any of: {}", declaration.required_capabilities_any_of.join(", "))}
                                </span>
                            </li>
                        }).collect_view()}
                    </ul>
                </section>
            })}

            {move || {
                if let Some(unavailable) = unavailable_message.get() {
                    view! {
                        <section class="organization-state" aria-live="polite">
                            <h3>"Navigation policy unavailable"</h3>
                            <p>{unavailable}</p>
                        </section>
                    }
                    .into_any()
                } else if let Some(current) = policy.get() {
                    let immutable_items = current.immutable_core_items.clone();
                    let mut rows = ordered_contributions(&current);
                    let empty_contributions_message = if definition_id.is_some() {
                        "This contribution has no mutable navigation destination."
                    } else {
                        "No mutable navigation contributions were returned."
                    };
                    if let Some(definition_id) = definition_id.as_ref() {
                        if let Some(reorder_band) = rows
                            .iter()
                            .find(|row| row.definition_id == *definition_id)
                            .map(|row| row.reorder_band.clone())
                        {
                            rows.retain(|row| row.reorder_band == reorder_band);
                        } else {
                            rows.clear();
                        }
                    }
                    view! {
                        <div class="module-navigation-policy__content">
                            <p class="data-table__secondary-text">
                                {format!("Policy revision {}. Contributions may move only within their existing Core-assigned band.", current.revision)}
                            </p>

                            <section aria-labelledby="module-core-navigation-heading">
                                <h3 id="module-core-navigation-heading">"Permanent Core destinations"</h3>
                                {if immutable_items.is_empty() {
                                    view! { <p>"No Core destination metadata was returned."</p> }.into_any()
                                } else {
                                    view! {
                                        <ul class="module-metadata-list">
                                            {immutable_items.into_iter().map(|item| view! {
                                                <li>
                                                    <strong>{item.label}</strong>
                                                    " — fixed in " {item.group} ". "
                                                    <span class="data-table__secondary-text">{item.route}</span>
                                                </li>
                                            }).collect_view()}
                                        </ul>
                                    }.into_any()
                                }}
                            </section>

                            <section aria-labelledby="module-contributed-navigation-heading">
                                <h3 id="module-contributed-navigation-heading">"Contributed destinations"</h3>
                                {if rows.is_empty() {
                                    view! {
                                        <p class="empty-state">{empty_contributions_message}</p>
                                    }
                                    .into_any()
                                } else {
                                    view! {
                                        <div class="data-table-container">
                                            <table class="data-table">
                                                <thead>
                                                    <tr>
                                                        <th scope="col">"Destination"</th>
                                                        <th scope="col">"Placement band"</th>
                                                        <th scope="col">"Visibility"</th>
                                                        <th scope="col">"Order"</th>
                                                    </tr>
                                                </thead>
                                                <tbody>
                                                    {rows.into_iter().map(|row| {
                                                        let contribution_id = row.id.clone();
                                                        let visibility_id = format!("module-navigation-visible-{}", row.id.replace('.', "-"));
                                                        let visibility_for = visibility_id.clone();
                                                        let visibility_input_id = visibility_id;
                                                        let earlier_disabled_id = row.id.clone();
                                                        let earlier_click_id = row.id.clone();
                                                        let later_disabled_id = row.id.clone();
                                                        let later_click_id = row.id.clone();
                                                        let control_id = row.id.replace('.', "-");
                                                        let earlier_button_id = format!("module-navigation-move-earlier-{control_id}");
                                                        let earlier_focus_id = earlier_button_id.clone();
                                                        let later_button_id = format!("module-navigation-move-later-{control_id}");
                                                        let later_focus_id = later_button_id.clone();
                                                        let earlier_fallback_id = later_button_id.clone();
                                                        let later_fallback_id = earlier_button_id.clone();
                                                        let checked_id = row.id.clone();
                                                        let earlier_label = row.label.clone();
                                                        let later_label = row.label.clone();
                                                        view! {
                                                            <tr data-navigation-contribution=row.id>
                                                                <th scope="row">
                                                                    <strong>{row.label}</strong>
                                                                    <span class="data-table__secondary-text">{row.destination}</span>
                                                                </th>
                                                                <td>
                                                                    <span>{row.after_core_anchor}</span>
                                                                    " — "
                                                                    <span>{row.before_core_anchor}</span>
                                                                </td>
                                                                <td>
                                                                    {if can_manage {
                                                                        view! {
                                                                            <label for=visibility_for>
                                                                                <input
                                                                                    id=visibility_input_id
                                                                                    type="checkbox"
                                                                                    prop:checked=move || policy.get().and_then(|value| value.contributions.into_iter().find(|candidate| candidate.id == checked_id)).is_some_and(|candidate| candidate.visible)
                                                                                    on:change=move |event| {
                                                                                        let visible = event_target_checked(&event);
                                                                                        policy.update(|current| {
                                                                                            if let Some(current) = current
                                                                                                && set_contribution_visibility(current, &contribution_id, visible)
                                                                                            {
                                                                                                is_dirty.set(true);
                                                                                                message.set(None);
                                                                                            }
                                                                                        });
                                                                                    }
                                                                                />
                                                                                " Show"
                                                                            </label>
                                                                        }.into_any()
                                                                    } else {
                                                                        view! {
                                                                            <span>{if row.visible { "Shown" } else { "Hidden" }}</span>
                                                                        }.into_any()
                                                                    }}
                                                                </td>
                                                                <td>
                                                                    {if can_manage {
                                                                        view! {
                                                                            <div class="data-table__action-group">
                                                                                <button
                                                                                    id=earlier_button_id
                                                                                    class="button button--secondary"
                                                                                    type="button"
                                                                                    aria-label=format!("Move {earlier_label} earlier within its band")
                                                                                    disabled=move || policy.get().as_ref().is_none_or(|value| !can_move_contribution(value, &earlier_disabled_id, PolicyMove::Earlier))
                                                                                    on:click=move |_| {
                                                                                        policy.update(|current| {
                                                                                            if let Some(current) = current
                                                                                                && move_contribution(current, &earlier_click_id, PolicyMove::Earlier)
                                                                                            {
                                                                                                is_dirty.set(true);
                                                                                                message.set(None);
                                                                                            }
                                                                                        });
                                                                                        restore_navigation_policy_focus(
                                                                                            earlier_focus_id.clone(),
                                                                                            earlier_fallback_id.clone(),
                                                                                        );
                                                                                    }
                                                                                >"Move earlier"</button>
                                                                                <button
                                                                                    id=later_button_id
                                                                                    class="button button--secondary"
                                                                                    type="button"
                                                                                    aria-label=format!("Move {later_label} later within its band")
                                                                                    disabled=move || policy.get().as_ref().is_none_or(|value| !can_move_contribution(value, &later_disabled_id, PolicyMove::Later))
                                                                                    on:click=move |_| {
                                                                                        policy.update(|current| {
                                                                                            if let Some(current) = current
                                                                                                && move_contribution(current, &later_click_id, PolicyMove::Later)
                                                                                            {
                                                                                                is_dirty.set(true);
                                                                                                message.set(None);
                                                                                            }
                                                                                        });
                                                                                        restore_navigation_policy_focus(
                                                                                            later_focus_id.clone(),
                                                                                            later_fallback_id.clone(),
                                                                                        );
                                                                                    }
                                                                                >"Move later"</button>
                                                                            </div>
                                                                        }.into_any()
                                                                    } else {
                                                                        view! { <span>{format!("Position {} in band", row.order + 1)}</span> }.into_any()
                                                                    }}
                                                                </td>
                                                            </tr>
                                                        }
                                                    }).collect_view()}
                                                </tbody>
                                            </table>
                                        </div>
                                    }.into_any()
                                }}
                            </section>

                            {can_manage.then(|| view! {
                                <div class="form-actions">
                                    <button
                                        class="button"
                                        type="button"
                                        disabled=move || !is_dirty.get() || is_saving.get()
                                        on:click=save
                                    >{move || if is_saving.get() { "Saving…" } else { "Save navigation" }}</button>
                                    <button
                                        class="button button--secondary"
                                        type="button"
                                        disabled=move || !is_dirty.get() || is_saving.get()
                                        on:click=discard
                                    >"Discard changes"</button>
                                </div>
                            })}
                        </div>
                    }
                    .into_any()
                } else {
                    view! {
                        <section class="organization-state" aria-live="polite">
                            <h3>"Loading navigation policy"</h3>
                            <p>"Fetching the current contribution display policy."</p>
                        </section>
                    }
                    .into_any()
                }
            }}

            <p class="form-message" aria-live="polite">{move || message.get().unwrap_or_default()}</p>
        </section>
    }
}

#[cfg(feature = "hydrate")]
fn restore_navigation_policy_focus(control_id: String, fallback_id: String) {
    let Some(window) = web_sys::window() else {
        return;
    };
    let callback = Closure::once(Box::new(move || {
        let Some(document) = web_sys::window().and_then(|window| window.document()) else {
            return;
        };
        let element = [control_id, fallback_id]
            .into_iter()
            .filter_map(|id| document.get_element_by_id(&id))
            .filter_map(|element| element.dyn_into::<web_sys::HtmlElement>().ok())
            .find(|element| !element.has_attribute("disabled"));
        if let Some(element) = element {
            let _ = element.focus();
        }
    }) as Box<dyn FnOnce()>);
    let _ = window.request_animation_frame(callback.as_ref().unchecked_ref());
    callback.forget();
}

#[cfg(not(feature = "hydrate"))]
fn restore_navigation_policy_focus(_: String, _: String) {}

#[cfg(test)]
mod tests {
    use leptos::prelude::*;

    use super::{
        ModuleNavigationPolicyView, PolicyMove, can_move_contribution, move_contribution,
        set_contribution_visibility,
    };
    use crate::features::modules::models::{
        ModuleManagementAccessV1, NavigationContributionDeclarationV1,
        NavigationPolicyContributionV1, NavigationPolicyResponseV1,
    };

    fn contribution(id: &str, band: &str, order: i32) -> NavigationPolicyContributionV1 {
        NavigationPolicyContributionV1 {
            id: id.into(),
            definition_id: format!("tessara.{id}"),
            label: id.into(),
            destination: format!("{id}.directory"),
            group: "Main".into(),
            reorder_band: band.into(),
            before_core_anchor: "operations".into(),
            after_core_anchor: "organization".into(),
            visible: true,
            order,
            required_capabilities_any_of: vec![format!("{id}:read")],
        }
    }

    fn policy() -> NavigationPolicyResponseV1 {
        NavigationPolicyResponseV1 {
            schema_version: 1,
            installation_id: "installation-1".into(),
            revision: 3,
            can_manage_navigation: true,
            immutable_core_items: Vec::new(),
            contributions: vec![
                contribution("forms", "authoring", 0),
                contribution("workflows", "authoring", 1),
                contribution("dashboards", "after_operations", 0),
            ],
        }
    }

    #[test]
    fn visibility_changes_only_the_selected_display_flag() {
        let mut policy = policy();
        assert!(set_contribution_visibility(&mut policy, "forms", false));
        assert!(!policy.contributions[0].visible);
        assert!(policy.contributions[1].visible);
        assert_eq!(policy.revision, 3);
        assert!(!set_contribution_visibility(&mut policy, "forms", false));
    }

    #[test]
    fn movement_stays_inside_the_existing_band_and_remains_dense() {
        let mut policy = policy();
        assert!(move_contribution(
            &mut policy,
            "workflows",
            PolicyMove::Earlier
        ));
        let forms = policy
            .contributions
            .iter()
            .find(|entry| entry.id == "forms")
            .expect("forms");
        let workflows = policy
            .contributions
            .iter()
            .find(|entry| entry.id == "workflows")
            .expect("workflows");
        let dashboards = policy
            .contributions
            .iter()
            .find(|entry| entry.id == "dashboards")
            .expect("dashboards");
        assert_eq!((workflows.order, forms.order), (0, 1));
        assert_eq!(dashboards.order, 0);
        assert_eq!(dashboards.reorder_band, "after_operations");
        assert!(!can_move_contribution(
            &policy,
            "workflows",
            PolicyMove::Earlier
        ));
        assert!(!move_contribution(
            &mut policy,
            "dashboards",
            PolicyMove::Earlier
        ));
    }

    #[test]
    fn read_only_policy_html_has_no_mutation_control() {
        let html = Owner::new().with(|| {
            let policy = RwSignal::new(Some(policy()));
            let persisted_policy = RwSignal::new(policy.get_untracked());
            let unavailable_message = RwSignal::new(None::<String>);
            view! {
                <ModuleNavigationPolicyView
                    policy
                    persisted_policy
                    unavailable_message
                    access=ModuleManagementAccessV1::read_only()
                    declared_navigation=vec![NavigationContributionDeclarationV1 {
                        id: "tessara.forms.navigation".into(),
                        destination: "forms.directory".into(),
                        label: "Forms".into(),
                        group: "Main".into(),
                        order_hint: 100,
                        required_capabilities_any_of: vec!["forms:read".into()],
                    }]
                />
            }
            .to_html()
        });

        assert!(html.contains("Read-only"));
        assert!(html.contains("Display choices do not grant access"));
        assert!(html.contains("data-navigation-declaration=\"tessara.forms.navigation\""));
        assert!(html.contains("Shown"));
        assert!(!html.contains("type=\"checkbox\""));
        assert!(!html.contains("Save navigation"));
        assert!(!html.contains("Move earlier"));
    }
}
