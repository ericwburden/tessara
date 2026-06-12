//! Node-type administration pages and helpers.
//!
//! Keep node type catalog, relationship editing, metadata fields, and scoped form displays here.

mod state;

use super::super::api::{load_admin_node_type_catalog, load_admin_node_type_detail};
use super::super::components::{AdministrationNodeTypeEditor, AdministrationNodeTypesList};
use crate::features::organization::NodeTypeUpsertRequest;
#[cfg(feature = "hydrate")]
use crate::http::send_json_id_request;
use crate::ui::{
    AppShell, Breadcrumb, BreadcrumbItem, BreadcrumbLink, BreadcrumbPage, BreadcrumbSeparator,
    PageHeader,
};
use std::collections::HashSet;

use leptos::prelude::*;
use state::AdministrationNodeTypesPageState;

#[component]
pub fn AdministrationNodeTypesPage() -> impl IntoView {
    let AdministrationNodeTypesPageState {
        node_types,
        selected_node_type_id,
        selected_detail,
        search,
        is_loading,
        detail_loading,
        is_saving,
        is_creating,
        message,
        name,
        slug,
        plural_label,
        parent_node_type_ids,
        child_node_type_ids,
    } = AdministrationNodeTypesPageState::new();

    load_admin_node_type_catalog(node_types, selected_node_type_id, is_loading, message, None);

    Effect::new(move |_| {
        if let Some(node_type_id) = selected_node_type_id.get() {
            if !node_type_id.is_empty() {
                load_admin_node_type_detail(
                    node_type_id,
                    selected_detail,
                    detail_loading,
                    message,
                    name,
                    slug,
                    plural_label,
                    parent_node_type_ids,
                    child_node_type_ids,
                );
            }
        } else {
            selected_detail.set(None);
        }
    });

    let begin_new = move |_| {
        selected_node_type_id.set(None);
        selected_detail.set(None);
        is_creating.set(true);
        message.set(None);
        name.set(String::new());
        slug.set(String::new());
        plural_label.set(String::new());
        parent_node_type_ids.set(HashSet::new());
        child_node_type_ids.set(HashSet::new());
    };

    let cancel_new = move |_| {
        is_creating.set(false);
        message.set(None);
        let first_id = node_types.with(|items| items.first().map(|item| item.id.clone()));
        selected_node_type_id.set(first_id);
    };
    let refresh_selected_node_type = move || {
        if let Some(node_type_id) = selected_node_type_id.get_untracked() {
            load_admin_node_type_detail(
                node_type_id,
                selected_detail,
                detail_loading,
                message,
                name,
                slug,
                plural_label,
                parent_node_type_ids,
                child_node_type_ids,
            );
        }
    };

    let save_node_type = move |event: leptos::ev::SubmitEvent| {
        event.prevent_default();
        let trimmed_name = name.get().trim().to_string();
        let trimmed_slug = slug.get().trim().to_string();
        let trimmed_plural = plural_label.get().trim().to_string();
        if trimmed_name.is_empty() || trimmed_slug.is_empty() {
            message.set(Some("Name and slug are required.".into()));
            return;
        }

        let request = NodeTypeUpsertRequest {
            name: trimmed_name,
            slug: trimmed_slug,
            plural_label: if trimmed_plural.is_empty() {
                None
            } else {
                Some(trimmed_plural)
            },
            parent_node_type_ids: parent_node_type_ids.get().into_iter().collect::<Vec<_>>(),
            child_node_type_ids: child_node_type_ids.get().into_iter().collect::<Vec<_>>(),
        };
        let body = match serde_json::to_string(&request) {
            Ok(body) => body,
            Err(_) => {
                message.set(Some("Node type request could not be prepared.".into()));
                return;
            }
        };
        let selected_id = selected_node_type_id.get_untracked();
        let creating = is_creating.get_untracked() || selected_id.is_none();

        #[cfg(feature = "hydrate")]
        {
            leptos::task::spawn_local(async move {
                is_saving.set(true);
                message.set(None);
                let builder = if creating {
                    gloo_net::http::Request::post("/api/admin/node-types")
                } else if let Some(node_type_id) = selected_id {
                    gloo_net::http::Request::put(&format!("/api/admin/node-types/{node_type_id}"))
                } else {
                    is_saving.set(false);
                    message.set(Some("Select a node type before saving.".into()));
                    return;
                };

                match send_json_id_request(builder, Some(body), "Save node type").await {
                    Ok(response) => {
                        is_creating.set(false);
                        load_admin_node_type_catalog(
                            node_types,
                            selected_node_type_id,
                            is_loading,
                            message,
                            Some(response.id),
                        );
                    }
                    Err(error) => message.set(Some(error)),
                }
                is_saving.set(false);
            });
        }
        #[cfg(not(feature = "hydrate"))]
        let _ = (body, creating, is_saving);
    };

    view! {
        <AppShell active_route="administration" title="Node Types">
            <section class="route-panel administration-node-types-page">
                <Breadcrumb>
                    <BreadcrumbItem>
                        <BreadcrumbLink href="/administration">"Administration"</BreadcrumbLink>
                    </BreadcrumbItem>
                    <BreadcrumbSeparator/>
                    <BreadcrumbItem>
                        <BreadcrumbPage>"Node Types"</BreadcrumbPage>
                    </BreadcrumbItem>
                </Breadcrumb>

                <PageHeader
                    title="Node Types"
                    description="Manage organization node type labels and parent-child hierarchy rules."
                >
                    <button class="button" type="button" on:click=begin_new>
                        "New Node Type"
                    </button>
                </PageHeader>

                {move || {
                    if is_loading.get() {
                        view! {
                            <section class="organization-state" aria-live="polite">
                                <h3>"Loading node types"</h3>
                                <p>"Fetching hierarchy configuration."</p>
                            </section>
                        }
                        .into_any()
                    } else if let Some(error) = message.get().filter(|_| node_types.get().is_empty()) {
                        view! {
                            <section class="organization-state is-error" role="alert">
                                <h3>"Node types unavailable"</h3>
                                <p>{error}</p>
                            </section>
                        }
                        .into_any()
                    } else {
                        view! {
                            <div class="administration-node-types-layout">
                                <AdministrationNodeTypesList
                                    node_types=node_types.get()
                                    search
                                    selected_node_type_id
                                    is_creating=is_creating.get()
                                />
                                <AdministrationNodeTypeEditor
                                    all_node_types=node_types.get()
                                    selected_detail=selected_detail.get()
                                    is_creating=is_creating.get()
                                    detail_loading=detail_loading.get()
                                    is_saving=is_saving.get()
                                    message=message.get()
                                    selected_node_type_id
                                    name
                                    slug
                                    plural_label
                                    parent_node_type_ids
                                    child_node_type_ids
                                    on_cancel=cancel_new
                                    on_submit=save_node_type
                                    on_metadata_changed=refresh_selected_node_type
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
