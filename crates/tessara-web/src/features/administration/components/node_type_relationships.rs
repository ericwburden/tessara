//! Node type relationship picker components.

use crate::features::organization::NodeTypeCatalogEntry;
use leptos::prelude::*;
use std::collections::HashSet;

#[component]
pub(crate) fn NodeTypeRelationshipPicker(
    title: &'static str,
    empty_message: &'static str,
    node_types: Vec<NodeTypeCatalogEntry>,
    selected_ids: RwSignal<HashSet<String>>,
    opposite_selected_ids: RwSignal<HashSet<String>>,
) -> impl IntoView {
    view! {
        <section class="organization-detail-card node-type-relationship-picker">
            <h3>{title}</h3>
            <div class="checkbox-list node-type-relationship-picker__list">
                {if node_types.is_empty() {
                    view! { <p class="muted">{empty_message}</p> }.into_any()
                } else {
                    node_types
                        .into_iter()
                        .map(|node_type| {
                            let node_type_id = node_type.id.clone();
                            let checked_id = node_type_id.clone();
                            let change_id = node_type_id;
                            view! {
                                <label class="checkbox-list__item node-type-relationship-picker__item">
                                    <input
                                        type="checkbox"
                                        prop:checked=move || selected_ids.get().contains(&checked_id)
                                        on:change=move |event| {
                                            let is_checked = event_target_checked(&event);
                                            selected_ids.update(|ids| {
                                                if is_checked {
                                                    ids.insert(change_id.clone());
                                                } else {
                                                    ids.remove(&change_id);
                                                }
                                            });
                                            if is_checked {
                                                opposite_selected_ids.update(|ids| {
                                                    ids.remove(&change_id);
                                                });
                                            }
                                        }
                                    />
                                    <span>
                                        <strong>{node_type.name}</strong>
                                        <small>{node_type.singular_label} " - " {node_type.plural_label}</small>
                                    </span>
                                </label>
                            }
                        })
                        .collect_view()
                        .into_any()
                }}
            </div>
        </section>
    }
}
