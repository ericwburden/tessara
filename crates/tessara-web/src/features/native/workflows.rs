use super::*;


#[component]
fn WorkflowsList(
    workflows: Vec<WorkflowSummary>,
    search: RwSignal<String>,
    status_filter: RwSignal<String>,
    status_options: Vec<String>,
    organization_nodes: Vec<OrganizationNode>,
) -> impl IntoView {
    let mut table_workflows = workflows.clone();
    table_workflows.sort_by(|left, right| {
        left.name
            .to_lowercase()
            .cmp(&right.name.to_lowercase())
            .then(left.id.cmp(&right.id))
    });
    let card_workflows = table_workflows.clone();
    let _ = organization_nodes;
    let page_size = RwSignal::new(10usize);
    let page_index = RwSignal::new(0usize);
    let total_count = table_workflows.len();
    let page_count = move || {
        if total_count == 0 {
            1
        } else {
            ((total_count + page_size.get() - 1) / page_size.get()).max(1)
        }
    };
    let current_page = move || page_index.get().min(page_count() - 1);
    let page_start = move || {
        if total_count == 0 {
            0
        } else {
            current_page() * page_size.get()
        }
    };
    let page_end = move || (page_start() + page_size.get()).min(total_count);
    let page_summary = move || {
        if total_count == 0 {
            "No workflows to display".to_string()
        } else {
            format!(
                "Showing {}-{} of {} workflows",
                page_start() + 1,
                page_end(),
                total_count
            )
        }
    };
    let available_nodes_sheet = RwSignal::new(None::<WorkflowAvailableNodesSheetData>);
    let assigned_users_sheet = RwSignal::new(None::<WorkflowAssignedUsersSheetData>);

    view! {
        <div class="forms-list forms-list-responsive-table">
            <div class="searchable-data-table">
                <div class="searchable-data-table__toolbar forms-list__toolbar">
                    <label class="searchable-data-table__search searchable-data-table__control">
                        <Search class="searchable-data-table__control-icon"/>
                        <span class="sr-only">"Search workflows"</span>
                        <input
                            type="search"
                            placeholder="Search workflows"
                            prop:value=move || search.get()
                            on:input=move |event| search.set(event_target_value(&event))
                        />
                    </label>
                </div>
                <DataTable>
                    <thead>
                        <tr>
                            <th scope="col">"Workflow name"</th>
                            <th scope="col">"Available at"</th>
                            <th class="data-table__cell--center" scope="col">"Active revision"</th>
                            <th class="data-table__cell--center" scope="col">
                                <FilterHeader
                                    label="Status"
                                    all_label="All statuses"
                                    filter=status_filter
                                    options=status_options
                                />
                            </th>
                            <th scope="col">"Active assignments"</th>
                        </tr>
                    </thead>
                    <tbody>
                        {move || if table_workflows.is_empty() {
                            view! {
                                <tr>
                                    <td class="data-table__empty" colspan="5">"No Workflows to Display"</td>
                                </tr>
                            }
                            .into_any()
                        } else {
                            table_workflows
                                .iter()
                                .skip(page_start())
                                .take(page_size.get())
                                .cloned()
                                .map(|workflow| {
                                    let workflow_href = format!("/workflows/{}", workflow.id);
                                    let status_key = workflow_status_key(&workflow).to_string();
                                    let status_label = workflow_status_label(&workflow);
                                    let version_label = workflow_version_label(&workflow);
                                    let available_at = workflow_available_node_links(&workflow.available_nodes);
                                    let assigned_users = workflow_assigned_user_links(&workflow);
                                    let workflow_name = workflow.name.clone();
                                    let workflow_source = workflow.source.clone();
                                    view! {
                                        <tr>
                                            <th scope="row">
                                                <a class="data-table__primary-link" href=workflow_href.clone()>{workflow.name}</a>
                                                <WorkflowSourceMarker source=workflow_source/>
                                            </th>
                                            <td>
                                                <WorkflowAvailableNodesList
                                                    nodes=available_at
                                                    workflow_name=workflow_name.clone()
                                                    workflow_href=workflow_href.clone()
                                                    sheet=available_nodes_sheet
                                                />
                                            </td>
                                            <td class="data-table__cell--center">{version_label}</td>
                                            <td class="data-table__cell--center">
                                                <span class=status_badge_class(&status_key)>{status_label}</span>
                                            </td>
                                            <td>
                                                <WorkflowAssignedUsersList
                                                    users=assigned_users
                                                    workflow_name=workflow_name
                                                    workflow_href=workflow_href
                                                    sheet=assigned_users_sheet
                                                />
                                            </td>
                                        </tr>
                                    }
                                })
                                .collect_view()
                                .into_any()
                        }}
                    </tbody>
                </DataTable>
                <div class="directory-table-pagination" aria-label="Workflow table pagination">
                    <p>{move || page_summary()}</p>
                    <div class="directory-table-pagination__actions">
                        <label class="directory-table-pagination__page-size searchable-data-table__filter searchable-data-table__control">
                            <span>"Rows"</span>
                            <select
                                prop:value=move || page_size.get().to_string()
                                on:change=move |event| {
                                    if let Ok(size) = event_target_value(&event).parse::<usize>() {
                                        page_size.set(size);
                                        page_index.set(0);
                                    }
                                }
                            >
                                <option value="10">"10"</option>
                                <option value="25">"25"</option>
                                <option value="50">"50"</option>
                            </select>
                        </label>
                        <button
                            class="button button--compact button--secondary"
                            type="button"
                            disabled=move || current_page() == 0
                            on:click=move |_| {
                                page_index.update(|page| *page = page.saturating_sub(1));
                            }
                        >
                            "Previous"
                        </button>
                        <span>{move || format!("Page {} of {}", current_page() + 1, page_count())}</span>
                        <button
                            class="button button--compact button--secondary"
                            type="button"
                            disabled=move || { current_page() + 1 >= page_count() }
                            on:click=move |_| {
                                let last_page = page_count().saturating_sub(1);
                                page_index.update(|page| *page = (*page + 1).min(last_page));
                            }
                        >
                            "Next"
                        </button>
                    </div>
                </div>
            </div>
            <div class="forms-list-mobile-cards">
                {move || if card_workflows.is_empty() {
                    view! { <p class="forms-list-mobile-empty">"No Workflows to Display"</p> }.into_any()
                } else {
                    card_workflows
                        .iter()
                        .skip(page_start())
                        .take(page_size.get())
                        .cloned()
                        .map(|workflow| {
                            let workflow_href = format!("/workflows/{}", workflow.id);
                            let status_key = workflow_status_key(&workflow).to_string();
                            let status_label = workflow_status_label(&workflow);
                            let version_label = workflow_version_label(&workflow);
                            let available_at = workflow_available_node_links(&workflow.available_nodes);
                            let assigned_users = workflow_assigned_user_links(&workflow);
                            let workflow_name = workflow.name.clone();
                            let workflow_source = workflow.source.clone();
                            view! {
                                <article class="forms-list-mobile-card">
                                    <div class="forms-list-mobile-card__header">
                                        <div class="forms-list-mobile-card__title-row">
                                            <h3><a href=workflow_href.clone()>{workflow.name}</a></h3>
                                            <WorkflowSourceMarker source=workflow_source/>
                                        </div>
                                    </div>
                                    <dl>
                                        <div>
                                            <dt>"Available at"</dt>
                                            <dd>
                                                <WorkflowAvailableNodesList
                                                    nodes=available_at
                                                    workflow_name=workflow_name.clone()
                                                    workflow_href=workflow_href.clone()
                                                    sheet=available_nodes_sheet
                                                />
                                            </dd>
                                        </div>
                                        <div>
                                            <dt>"Active revision"</dt>
                                            <dd>{version_label}</dd>
                                        </div>
                                        <div>
                                            <dt>"Status"</dt>
                                            <dd><span class=status_badge_class(&status_key)>{status_label}</span></dd>
                                        </div>
                                        <div>
                                            <dt>"Active assignments"</dt>
                                            <dd>
                                                <WorkflowAssignedUsersList
                                                    users=assigned_users
                                                    workflow_name=workflow_name
                                                    workflow_href=workflow_href
                                                    sheet=assigned_users_sheet
                                                />
                                            </dd>
                                        </div>
                                    </dl>
                                </article>
                            }
                        })
                        .collect_view()
                        .into_any()
                }}
            </div>
            <WorkflowAvailableNodesSheet detail=available_nodes_sheet/>
            <WorkflowAssignedUsersSheet detail=assigned_users_sheet/>
        </div>
    }
}

#[component]
fn WorkflowAvailableNodesList(
    nodes: Vec<FormAttachmentLink>,
    workflow_name: String,
    workflow_href: String,
    sheet: RwSignal<Option<WorkflowAvailableNodesSheetData>>,
) -> impl IntoView {
    let total_nodes = nodes.len();
    let nodes_for_sheet = nodes.clone();
    let workflow_name_for_sheet = workflow_name.clone();
    let workflow_href_for_sheet = workflow_href.clone();

    view! {
        <div class="forms-attached-list">
            {if total_nodes == 0 {
                view! { <p>"Not available"</p> }.into_any()
            } else {
                view! {
                    <button
                        class="forms-attached-list__more"
                        type="button"
                        aria-label=format!("View available organization nodes for {workflow_name_for_sheet}")
                        title="Opens detail panel"
                        on:click=move |_| {
                            sheet.set(Some(WorkflowAvailableNodesSheetData {
                                workflow_name: workflow_name_for_sheet.clone(),
                                workflow_href: workflow_href_for_sheet.clone(),
                                nodes: nodes_for_sheet.clone(),
                            }));
                        }
                    >
                        <span>{node_count_label(total_nodes)}</span>
                        <PanelRight class="forms-attached-list__icon"/>
                    </button>
                }
                .into_any()
            }}
        </div>
    }
}

#[component]
fn WorkflowAvailableNodesSheet(
    detail: RwSignal<Option<WorkflowAvailableNodesSheetData>>,
) -> impl IntoView {
    let search = RwSignal::new(String::new());
    let close = move |_| {
        detail.set(None);
        search.set(String::new());
    };
    let filtered_nodes = move || {
        let query = search.get().trim().to_lowercase();
        detail
            .get()
            .map(|data| {
                data.nodes
                    .into_iter()
                    .filter(|node| {
                        query.is_empty()
                            || node.label.to_lowercase().contains(&query)
                            || node.title.to_lowercase().contains(&query)
                    })
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default()
    };

    view! {
        <Portal>
            <Show when=move || detail.get().is_some()>
                <section class="sheet-overlay forms-attached-overlay" aria-label="Available organization nodes">
                    <button class="sheet-overlay__scrim" type="button" aria-label="Close available nodes" on:click=close></button>
                    <aside class="sheet-panel blurred-surface forms-attached-sheet" role="dialog" aria-modal="true" aria-label="Available organization nodes">
                        <div class="sheet-panel__actions">
                            {move || {
                                detail
                                    .get()
                                    .map(|data| {
                                        view! {
                                            <a class="icon-button sheet-panel__open" href=data.workflow_href aria-label="Open workflow detail" title="Open workflow detail">
                                                <ExternalLink class="icon-button__icon"/>
                                            </a>
                                        }
                                        .into_any()
                                    })
                                    .unwrap_or_else(|| empty_view())
                            }}
                            <button class="icon-button sheet-panel__close" type="button" aria-label="Close available nodes" title="Close available nodes" on:click=close>
                                <X class="icon-button__icon"/>
                            </button>
                        </div>
                        {move || {
                            detail
                                .get()
                                .map(|data| {
                                    let total = data.nodes.len();
                                    view! {
                                        <header class="sheet-panel__header">
                                            <p>"Available Nodes"</p>
                                            <h2>{data.workflow_name}</h2>
                                            <span class="forms-attached-sheet__count">{format!("{total} nodes")}</span>
                                        </header>
                                        <section class="sheet-panel__section">
                                            <label class="searchable-data-table__search searchable-data-table__control forms-attached-sheet__search">
                                                <Search class="searchable-data-table__control-icon"/>
                                                <span class="sr-only">"Search available nodes"</span>
                                                <input
                                                    type="search"
                                                    placeholder="Search available nodes"
                                                    prop:value=move || search.get()
                                                    on:input=move |event| search.set(event_target_value(&event))
                                                />
                                            </label>
                                            <div class="forms-attached-sheet__list">
                                                {move || {
                                                    let nodes = filtered_nodes();
                                                    if nodes.is_empty() {
                                                        view! { <p class="forms-attached-sheet__empty">"No Available Nodes to Display"</p> }.into_any()
                                                    } else {
                                                        nodes
                                                            .into_iter()
                                                            .map(|node| {
                                                                let node_title = node.title.clone();
                                                                view! {
                                                                    <a class="forms-attached-sheet__item" href=node.href title=node_title>
                                                                        <span>{node.label}</span>
                                                                        <small>{node.title}</small>
                                                                    </a>
                                                                }
                                                            })
                                                            .collect_view()
                                                            .into_any()
                                                    }
                                                }}
                                            </div>
                                        </section>
                                    }
                                    .into_any()
                                })
                                .unwrap_or_else(|| empty_view())
                        }}
                    </aside>
                </section>
            </Show>
        </Portal>
    }
}

#[component]
fn WorkflowAssignedUsersList(
    users: Vec<FormAttachmentLink>,
    workflow_name: String,
    workflow_href: String,
    sheet: RwSignal<Option<WorkflowAssignedUsersSheetData>>,
) -> impl IntoView {
    let total_users = users.len();
    let users_for_sheet = users.clone();
    let workflow_name_for_sheet = workflow_name.clone();
    let workflow_href_for_sheet = workflow_href.clone();

    view! {
        <div class="forms-attached-list">
            {if total_users == 0 {
                view! { <p>"No active assignments"</p> }.into_any()
            } else {
                view! {
                    <button
                        class="forms-attached-list__more"
                        type="button"
                        aria-label=format!("View assigned users for {workflow_name_for_sheet}")
                        title="Opens detail panel"
                        on:click=move |_| {
                            sheet.set(Some(WorkflowAssignedUsersSheetData {
                                workflow_name: workflow_name_for_sheet.clone(),
                                workflow_href: workflow_href_for_sheet.clone(),
                                users: users_for_sheet.clone(),
                            }));
                        }
                    >
                        <span>{user_count_label(total_users)}</span>
                        <PanelRight class="forms-attached-list__icon"/>
                    </button>
                }
                .into_any()
            }}
        </div>
    }
}

#[component]
fn WorkflowAssignedUsersSheet(
    detail: RwSignal<Option<WorkflowAssignedUsersSheetData>>,
) -> impl IntoView {
    let search = RwSignal::new(String::new());
    let close = move |_| {
        detail.set(None);
        search.set(String::new());
    };
    let filtered_nodes = move || {
        let query = search.get().trim().to_lowercase();
        detail
            .get()
            .map(|data| {
                data.users
                    .into_iter()
                    .filter(|user| {
                        query.is_empty()
                            || user.label.to_lowercase().contains(&query)
                            || user.title.to_lowercase().contains(&query)
                    })
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default()
    };

    view! {
        <Portal>
            <Show when=move || detail.get().is_some()>
                <section class="sheet-overlay forms-attached-overlay" aria-label="Assigned users">
                    <button class="sheet-overlay__scrim" type="button" aria-label="Close assigned users" on:click=close></button>
                    <aside class="sheet-panel blurred-surface forms-attached-sheet" role="dialog" aria-modal="true" aria-label="Assigned users">
                        <div class="sheet-panel__actions">
                            {move || {
                                detail
                                    .get()
                                    .map(|data| {
                                        view! {
                                            <a class="icon-button sheet-panel__open" href=data.workflow_href aria-label="Open workflow detail" title="Open workflow detail">
                                                <ExternalLink class="icon-button__icon"/>
                                            </a>
                                        }
                                        .into_any()
                                    })
                                    .unwrap_or_else(|| empty_view())
                            }}
                            <button class="icon-button sheet-panel__close" type="button" aria-label="Close assigned users" title="Close assigned users" on:click=close>
                                <X class="icon-button__icon"/>
                            </button>
                        </div>
                        {move || {
                            detail
                                .get()
                                .map(|data| {
                                    let total = data.users.len();
                                    view! {
                                        <header class="sheet-panel__header">
                                            <p>"Assigned Users"</p>
                                            <h2>{data.workflow_name}</h2>
                                            <span class="forms-attached-sheet__count">{user_count_label(total)}</span>
                                        </header>
                                        <section class="sheet-panel__section">
                                            <label class="searchable-data-table__search searchable-data-table__control forms-attached-sheet__search">
                                                <Search class="searchable-data-table__control-icon"/>
                                                <span class="sr-only">"Search assigned users"</span>
                                                <input
                                                    type="search"
                                                    placeholder="Search assigned users"
                                                    prop:value=move || search.get()
                                                    on:input=move |event| search.set(event_target_value(&event))
                                                />
                                            </label>
                                            <div class="forms-attached-sheet__list">
                                                {move || {
                                                    let users = filtered_nodes();
                                                    if users.is_empty() {
                                                        view! { <p class="forms-attached-sheet__empty">"No Assigned Users to Display"</p> }.into_any()
                                                    } else {
                                                        users
                                                            .into_iter()
                                                            .map(|user| {
                                                                let user_title = user.title.clone();
                                                                view! {
                                                                    <a class="forms-attached-sheet__item" href=user.href title=user_title>
                                                                        <span>{user.label}</span>
                                                                        <small>{user.title}</small>
                                                                    </a>
                                                                }
                                                            })
                                                            .collect_view()
                                                            .into_any()
                                                    }
                                                }}
                                            </div>
                                        </section>
                                    }
                                    .into_any()
                                })
                                .unwrap_or_else(|| empty_view())
                        }}
                    </aside>
                </section>
            </Show>
        </Portal>
    }
}

#[component]
pub fn WorkflowsPage() -> impl IntoView {
    let workflows = RwSignal::new(Vec::<WorkflowSummary>::new());
    let organization_nodes = RwSignal::new(Vec::<OrganizationNode>::new());
    let search = RwSignal::new(String::new());
    let status_filter = RwSignal::new("all".to_string());
    let is_loading = RwSignal::new(true);
    let load_error = RwSignal::new(None::<String>);

    Effect::new(move |_| {
        load_workflows(workflows, is_loading, load_error);
        load_workflow_assignment_nodes(organization_nodes);
    });

    let filtered_workflows = move || {
        let query = search.get();
        let selected_status = status_filter.get();
        workflows
            .get()
            .into_iter()
            .filter(|workflow| {
                let version_label = workflow_version_label(workflow);
                let status_label = workflow_status_label(workflow);
                let assigned_to = workflow_assigned_users_label(workflow);
                let description = workflow_description_label(workflow);
                let available_at = workflow_available_nodes_label(&workflow.available_nodes);
                text_matches(
                    &query,
                    &[
                        workflow.name.as_str(),
                        workflow.slug.as_str(),
                        description.as_str(),
                        version_label.as_str(),
                        status_label.as_str(),
                        assigned_to.as_str(),
                        available_at.as_str(),
                    ],
                ) && (selected_status == "all" || selected_status == status_label)
            })
            .collect::<Vec<_>>()
    };

    let status_options = move || {
        unique_filter_options(
            workflows
                .get()
                .iter()
                .map(workflow_status_label)
                .collect::<Vec<_>>(),
        )
    };

    view! {
        <AppShell active_route="workflows" title="Workflows">
            <section class="route-panel workflows-page">
                <PageHeader title="Workflows">
                    <Button label="Create Workflow" href="/workflows/new"/>
                </PageHeader>

                {move || {
                    if is_loading.get() {
                        view! {
                            <section class="organization-state" aria-live="polite">
                                <h3>"Loading workflows"</h3>
                                <p>"Fetching workflow definitions."</p>
                            </section>
                        }
                        .into_any()
                    } else if let Some(error) = load_error.get() {
                        view! {
                            <section class="organization-state is-error" role="alert">
                                <h3>"Workflows unavailable"</h3>
                                <p>{error}</p>
                            </section>
                        }
                        .into_any()
                    } else {
                        view! {
                            <WorkflowsList
                                workflows=filtered_workflows()
                                search=search
                                status_filter=status_filter
                                status_options=status_options()
                                organization_nodes=organization_nodes.get()
                            />
                        }
                        .into_any()
                    }
                }}
            </section>
        </AppShell>
    }
}

#[component]
fn WorkflowAvailableNodesPicker(
    nodes: Vec<OrganizationNode>,
    selected_node_ids: RwSignal<HashSet<String>>,
) -> impl IntoView {
    let search = RwSignal::new(String::new());
    let filtered_nodes = move || {
        let query = search.get();
        nodes
            .clone()
            .into_iter()
            .filter(|node| {
                text_matches(
                    &query,
                    &[
                        node.name.as_str(),
                        node.node_type_singular_label.as_str(),
                        node.parent_node_name.as_deref().unwrap_or(""),
                    ],
                )
            })
            .collect::<Vec<_>>()
    };

    view! {
        <div class="permission-picker workflow-available-node-picker">
            <label class="searchable-data-table__search searchable-data-table__control">
                <Search class="searchable-data-table__control-icon"/>
                <span class="sr-only">"Search available nodes"</span>
                <input
                    type="search"
                    placeholder="Search available nodes"
                    prop:value=move || search.get()
                    on:input=move |event| search.set(event_target_value(&event))
                />
            </label>
            <div class="checkbox-list permission-picker__list permission-picker__list--compact">
                {move || {
                    let nodes = filtered_nodes();
                    if nodes.is_empty() {
                        return view! {
                            <section class="organization-state">
                                <h3>"No nodes found"</h3>
                                <p>"Adjust the search to choose where this workflow is available."</p>
                            </section>
                        }
                        .into_any();
                    }

                    nodes
                        .into_iter()
                        .map(|node| {
                            let node_id = node.id.clone();
                            let node_id_for_checked = node_id.clone();
                            let node_id_for_change = node_id.clone();
                            let node_name = node.name.clone();
                            let node_type = node.node_type_singular_label.clone();
                            let node_path = node_display_path(&node);
                            view! {
                                <label class="checkbox-list__item permission-picker__item">
                                    <input
                                        type="checkbox"
                                        prop:checked=move || selected_node_ids.get().contains(&node_id_for_checked)
                                        on:change=move |event| {
                                            let checked = event_target_checked(&event);
                                            selected_node_ids.update(|ids| {
                                                if checked {
                                                    ids.insert(node_id_for_change.clone());
                                                } else {
                                                    ids.remove(&node_id_for_change);
                                                }
                                            });
                                        }
                                    />
                                    <span>
                                        <strong>{node_name}</strong>
                                        <small>{format!("{node_type} - {node_path}")}</small>
                                    </span>
                                </label>
                            }
                        })
                        .collect_view()
                        .into_any()
                }}
            </div>
        </div>
    }
}

#[component]
pub fn WorkflowsNewPage() -> impl IntoView {
    let node_types = RwSignal::new(Vec::<NodeTypeCatalogEntry>::new());
    let organization_nodes = RwSignal::new(Vec::<OrganizationNode>::new());
    let forms = RwSignal::new(Vec::<FormSummary>::new());
    let existing_workflows = RwSignal::new(Vec::<WorkflowSummary>::new());
    let seeded_from_form = RwSignal::new(false);
    let name = RwSignal::new(String::new());
    let available_node_ids = RwSignal::new(HashSet::<String>::new());
    let steps = RwSignal::new(Vec::<WorkflowStepDraft>::new());
    let next_step_id = RwSignal::new(1_usize);
    let description = RwSignal::new(String::new());
    let is_loading = RwSignal::new(true);
    let is_saving = RwSignal::new(false);
    let message = RwSignal::new(None::<String>);

    Effect::new(move |_| {
        load_workflow_create_options(
            node_types,
            organization_nodes,
            forms,
            existing_workflows,
            is_loading,
            message,
        );
    });

    Effect::new(move |_| {
        if is_loading.get() || seeded_from_form.get_untracked() {
            return;
        }

        let form_id: Option<String> = {
            #[cfg(feature = "hydrate")]
            {
                current_search_param("form_id")
            }
            #[cfg(not(feature = "hydrate"))]
            {
                None
            }
        };
        let Some(form_id) = form_id else {
            seeded_from_form.set(true);
            return;
        };

        let available_forms = forms.get();
        let Some(form) = available_forms.iter().find(|form| form.id == form_id) else {
            seeded_from_form.set(true);
            return;
        };
        let Some(version) = form
            .versions
            .iter()
            .find(|version| version.status == "published")
        else {
            seeded_from_form.set(true);
            return;
        };

        name.set(format!("{} Workflow", form.name));
        description.set(format!("Workflow for {}.", form.name));
        steps.set(vec![WorkflowStepDraft {
            id: 1,
            title: format!("{} Response", form.name),
            form_version_id: version.id.clone(),
        }]);
        next_step_id.set(2);
        seeded_from_form.set(true);
    });

    Effect::new(move |_| {
        if is_loading.get() {
            return;
        }
        let available_options = workflow_form_version_options(&forms.get(), &node_types.get(), "");
        steps.update(|steps| {
            steps.retain(|step| {
                step.form_version_id.is_empty()
                    || available_options
                        .iter()
                        .any(|(id, _, _)| id == &step.form_version_id)
            });
        });
    });

    let add_step = move || {
        let id = next_step_id.get_untracked();
        next_step_id.set(id + 1);
        steps.update(|steps| {
            steps.push(WorkflowStepDraft {
                id,
                title: format!("Step {}", steps.len() + 1),
                form_version_id: String::new(),
            });
        });
    };

    let can_submit = move || {
        !is_saving.get()
            && !name.get().trim().is_empty()
            && !available_node_ids.get().is_empty()
            && {
                let current_steps = steps.get();
                !current_steps.is_empty()
                    && current_steps
                        .iter()
                        .all(|step| !step.form_version_id.trim().is_empty())
            }
    };

    view! {
        <AppShell active_route="workflows" title="Workflows">
            <div class="app-page">
                <Breadcrumb>
                    <BreadcrumbItem>
                        <BreadcrumbLink href="/workflows">"Workflows"</BreadcrumbLink>
                    </BreadcrumbItem>
                    <BreadcrumbSeparator/>
                    <BreadcrumbItem>
                        <BreadcrumbPage>"Create Workflow"</BreadcrumbPage>
                    </BreadcrumbItem>
                </Breadcrumb>

                <section class="route-panel workflows-page">
                    <PageHeader title="Create Workflow"/>

                    {move || {
                        if is_loading.get() {
                            view! {
                                <section class="organization-state" aria-live="polite">
                                    <h3>"Loading workflow options"</h3>
                                    <p>"Fetching forms and workflow names."</p>
                                </section>
                            }
                            .into_any()
                        } else {
                            view! {
                                <form
                                    class="native-form workflow-create-form"
                                    on:submit=move |event| {
                                        event.prevent_default();
                                        submit_create_workflow(
                                            name,
                                            available_node_ids,
                                            steps,
                                            description,
                                            existing_workflows,
                                            is_saving,
                                            message,
                                        );
                                    }
                                >
                                    <div class="form-grid">
                                        <label class="form-field">
                                            <span>"Workflow Name"</span>
                                            <input
                                                type="text"
                                                value=move || name.get()
                                                on:input=move |event| {
                                                    name.set(event_target_value(&event));
                                                }
                                            />
                                        </label>
                                        <label class="form-field">
                                            <span>"Description"</span>
                                            <textarea
                                                prop:value=move || description.get()
                                                on:input=move |event| {
                                                    description.set(event_target_value(&event));
                                                }
                                            ></textarea>
                                        </label>
                                    </div>

                                    <section class="form-section">
                                        <h3>"Available At"</h3>
                                        <WorkflowAvailableNodesPicker
                                            nodes=organization_nodes.get()
                                            selected_node_ids=available_node_ids
                                        />
                                    </section>

                                    <section class="form-section">
                                        <div class="form-builder-section-card__header">
                                            <h3>"Workflow Steps"</h3>
                                            <button
                                                class="button button--secondary"
                                                type="button"
                                                disabled=move || {
                                                    workflow_form_version_options(
                                                        &forms.get(),
                                                        &node_types.get(),
                                                        "",
                                                    ).is_empty()
                                                }
                                                on:click=move |_| add_step()
                                            >
                                                "+ Add Step"
                                            </button>
                                        </div>
                                        {move || {
                                            let options = workflow_form_version_options(
                                                &forms.get(),
                                                &node_types.get(),
                                                "",
                                            );
                                            if options.is_empty() {
                                                return view! {
                                                    <section class="organization-state">
                                                        <h3>"No published forms available"</h3>
                                                        <p>"Publish at least one form version before creating a workflow."</p>
                                                    </section>
                                                }
                                                .into_any();
                                            }

                                            if steps.get().is_empty() {
                                                return view! {
                                                    <section class="organization-state">
                                                        <h3>"No workflow steps yet"</h3>
                                                        <p>"Add one or more form steps to define the workflow."</p>
                                                    </section>
                                                }
                                                .into_any();
                                            }

                                            view! {
                                                <div class="workflow-step-list">
                                                    <For
                                                        each=move || {
                                                            steps.get().into_iter().enumerate().collect::<Vec<_>>()
                                                        }
                                                        key=|(_, step)| step.id
                                                        children=move |(index, step)| {
                                                            let step_id = step.id;
                                                            let step_position = move || {
                                                                steps
                                                                    .get()
                                                                    .iter()
                                                                    .position(|step| step.id == step_id)
                                                                    .map(|index| index + 1)
                                                                    .unwrap_or(index + 1)
                                                            };
                                                            view! {
                                                                <article class="workflow-step-card">
                                                                    <header class="workflow-step-card__header">
                                                                        <span class="workflow-step-card__position">{move || format!("Step {}", step_position())}</span>
                                                                        <div class="workflow-step-card__actions">
                                                                            <button
                                                                                class="icon-button icon-button--control"
                                                                                type="button"
                                                                                title="Move step up"
                                                                                disabled=move || step_position() <= 1
                                                                                on:click=move |_| {
                                                                                    steps.update(|steps| {
                                                                                        if let Some(index) = steps.iter().position(|step| step.id == step_id) {
                                                                                            if index > 0 {
                                                                                                steps.swap(index, index - 1);
                                                                                            }
                                                                                        }
                                                                                    });
                                                                                }
                                                                            >
                                                                                <ArrowUp/>
                                                                            </button>
                                                                            <button
                                                                                class="icon-button icon-button--control"
                                                                                type="button"
                                                                                title="Move step down"
                                                                                disabled=move || {
                                                                                    let step_count = steps.get().len();
                                                                                    step_position() >= step_count
                                                                                }
                                                                                on:click=move |_| {
                                                                                    steps.update(|steps| {
                                                                                        if let Some(index) = steps.iter().position(|step| step.id == step_id) {
                                                                                            if index + 1 < steps.len() {
                                                                                                steps.swap(index, index + 1);
                                                                                            }
                                                                                        }
                                                                                    });
                                                                                }
                                                                            >
                                                                                <ArrowDown/>
                                                                            </button>
                                                                            <button
                                                                                class="icon-button icon-button--danger"
                                                                                type="button"
                                                                                title="Remove step"
                                                                                on:click=move |_| {
                                                                                    steps.update(|steps| {
                                                                                        steps.retain(|step| step.id != step_id);
                                                                                    });
                                                                                }
                                                                            >
                                                                                <Trash2/>
                                                                            </button>
                                                                        </div>
                                                                    </header>
                                                                    <div class="form-grid">
                                                                        <label class="form-field">
                                                                            <span>"Step Title"</span>
                                                                            <input
                                                                                type="text"
                                                                                prop:value=move || {
                                                                                    workflow_step_title_by_id(&steps.get(), step_id)
                                                                                }
                                                                                on:input=move |event| {
                                                                                    let value = event_target_value(&event);
                                                                                    steps.update(|steps| {
                                                                                        if let Some(step) = steps.iter_mut().find(|step| step.id == step_id) {
                                                                                            step.title = value;
                                                                                        }
                                                                                    });
                                                                                }
                                                                            />
                                                                        </label>
                                                                        <label class="form-field">
                                                                            <span>"Form Version"</span>
                                                                            <select
                                                                                prop:value=move || {
                                                                                    workflow_step_form_version_id_by_id(&steps.get(), step_id)
                                                                                }
                                                                                on:change=move |event| {
                                                                                    let value = event_target_value(&event);
                                                                                    steps.update(|steps| {
                                                                                        if let Some(step) = steps.iter_mut().find(|step| step.id == step_id) {
                                                                                            step.form_version_id = value;
                                                                                        }
                                                                                    });
                                                                                }
                                                                            >
                                                                                <option value="">"Select form version"</option>
                                                                                {workflow_form_version_options(
                                                                                    &forms.get(),
                                                                                    &node_types.get(),
                                                                                    "",
                                                                                )
                                                                                    .into_iter()
                                                                                    .map(|(id, label, _)| {
                                                                                        view! {
                                                                                            <option value=id>{label}</option>
                                                                                        }
                                                                                    })
                                                                                    .collect_view()}
                                                                            </select>
                                                                        </label>
                                                                    </div>
                                                                    <div class="workflow-step-card__footer">
                                                                        <span>{move || {
                                                                            let selected_form_version_id = steps
                                                                                .get()
                                                                                .into_iter()
                                                                                .find(|step| step.id == step_id)
                                                                                .map(|step| step.form_version_id)
                                                                                .unwrap_or_default();
                                                                            workflow_step_form_label(&forms.get(), &selected_form_version_id)
                                                                        }}</span>
                                                                    </div>
                                                                </article>
                                                            }
                                                        }
                                                    />
                                                </div>
                                            }
                                            .into_any()
                                        }}
                                    </section>

                                    {move || message.get().map(|message| view! {
                                        <p class="form-message" role="status">{message}</p>
                                    })}

                                    <div class="form-actions">
                                        <a class="button" href="/workflows">"Cancel"</a>
                                        <button class="button button--secondary" type="submit" disabled=move || !can_submit()>
                                            {move || if is_saving.get() { "Creating..." } else { "Create Workflow" }}
                                        </button>
                                    </div>
                                </form>
                            }
                            .into_any()
                        }
                    }}
                </section>
            </div>
        </AppShell>
    }
}

#[component]
pub fn WorkflowAssignmentsPage() -> impl IntoView {
    let assignments = RwSignal::new(Vec::<WorkflowAssignmentSummary>::new());
    let candidates = RwSignal::new(Vec::<WorkflowAssignmentCandidate>::new());
    let assignees = RwSignal::new(Vec::<WorkflowAssigneeOption>::new());
    let selected_candidate_id = RwSignal::new(String::new());
    let selected_workflow_version_id = RwSignal::new(String::new());
    let selected_node_id = RwSignal::new(String::new());
    let requested_workflow_id = RwSignal::new(String::new());
    let selected_account_ids = RwSignal::new(HashSet::<String>::new());
    let workflow_search = RwSignal::new(String::new());
    let node_search = RwSignal::new(String::new());
    let assignee_search = RwSignal::new(String::new());
    let assignment_search = RwSignal::new(String::new());
    let status_filter = RwSignal::new("all".to_string());
    let state_filter = RwSignal::new("all".to_string());
    let assignee_filter = RwSignal::new("all".to_string());
    let assignments_loading = RwSignal::new(true);
    let assignments_error = RwSignal::new(None::<String>);
    let candidates_loading = RwSignal::new(true);
    let candidates_error = RwSignal::new(None::<String>);
    let assignees_loading = RwSignal::new(false);
    let assignees_error = RwSignal::new(None::<String>);
    let is_saving = RwSignal::new(false);
    let message = RwSignal::new(None::<String>);

    Effect::new(move |_| {
        load_workflow_assignments(assignments, assignments_loading, assignments_error);
        load_workflow_assignment_candidates(candidates, candidates_loading, candidates_error);
        #[cfg(feature = "hydrate")]
        {
            if let Some(assignment_id) = current_search_param("assignment_id") {
                assignment_search.set(assignment_id);
            }
        }
    });

    Effect::new(move |_| {
        let available_candidates = candidates.get();
        let workflow_id = requested_workflow_id
            .get()
            .into_nonempty()
            .or({
                #[cfg(feature = "hydrate")]
                {
                    current_search_param("workflow_id")
                }
                #[cfg(not(feature = "hydrate"))]
                {
                    None
                }
            })
            .unwrap_or_default();
        if workflow_id.is_empty() || !selected_workflow_version_id.get_untracked().is_empty() {
            return;
        }

        if let Some(candidate) = available_candidates.into_iter().find(|candidate| {
            candidate.workflow_id == workflow_id || candidate.workflow_version_id == workflow_id
        }) {
            selected_workflow_version_id.set(candidate.workflow_version_id);
            workflow_search.set(String::new());
            requested_workflow_id.set(String::new());
        }
    });

    Effect::new(move |_| {
        let workflow_version_id = selected_workflow_version_id.get();
        let node_id = selected_node_id.get();
        let next_candidate_id = if workflow_version_id.is_empty() || node_id.is_empty() {
            String::new()
        } else {
            candidates
                .get()
                .into_iter()
                .find(|candidate| {
                    candidate.workflow_version_id == workflow_version_id
                        && candidate.node_id == node_id
                })
                .map(|candidate| workflow_assignment_candidate_key(&candidate))
                .unwrap_or_default()
        };

        if selected_candidate_id.get_untracked() != next_candidate_id {
            selected_candidate_id.set(next_candidate_id);
        }
    });

    Effect::new(move |_| {
        let selected_id = selected_candidate_id.get();
        selected_account_ids.set(HashSet::new());
        let selected_candidate = candidates
            .get()
            .into_iter()
            .find(|candidate| workflow_assignment_candidate_key(candidate) == selected_id);

        if let Some(candidate) = selected_candidate {
            load_workflow_assignment_assignees(
                candidate.workflow_version_id,
                candidate.node_id,
                assignees,
                assignees_loading,
                assignees_error,
            );
        } else {
            assignees.set(Vec::new());
            assignees_loading.set(false);
            assignees_error.set(None);
        }
    });

    let filtered_workflow_candidates = move || {
        let query = workflow_search.get();
        let selected_node_id = selected_node_id.get();
        let mut seen = HashSet::new();
        let mut workflows = candidates
            .get()
            .into_iter()
            .filter(|candidate| {
                (selected_node_id.is_empty() || candidate.node_id == selected_node_id)
                    && seen.insert(candidate.workflow_version_id.clone())
                    && text_matches(
                        &query,
                        &[
                            candidate.workflow_name.as_str(),
                            candidate
                                .workflow_version_label
                                .as_deref()
                                .unwrap_or_default(),
                        ],
                    )
            })
            .collect::<Vec<_>>();
        workflows.sort_by(|left, right| {
            left.workflow_name
                .cmp(&right.workflow_name)
                .then(left.workflow_version_id.cmp(&right.workflow_version_id))
        });
        workflows
    };
    let filtered_node_candidates = move || {
        let query = node_search.get();
        let selected_workflow_version_id = selected_workflow_version_id.get();
        let mut seen = HashSet::new();
        let mut nodes = candidates
            .get()
            .into_iter()
            .filter(|candidate| {
                (selected_workflow_version_id.is_empty()
                    || candidate.workflow_version_id == selected_workflow_version_id)
                    && seen.insert(candidate.node_id.clone())
                    && text_matches(
                        &query,
                        &[candidate.node_name.as_str(), candidate.node_path.as_str()],
                    )
            })
            .collect::<Vec<_>>();
        nodes.sort_by(|left, right| left.node_path.cmp(&right.node_path));
        nodes
    };
    let selected_pair_is_valid = move || {
        let workflow_version_id = selected_workflow_version_id.get();
        let node_id = selected_node_id.get();
        !workflow_version_id.is_empty()
            && !node_id.is_empty()
            && candidates.get().into_iter().any(|candidate| {
                candidate.workflow_version_id == workflow_version_id && candidate.node_id == node_id
            })
    };
    let invalid_pair_message = move || {
        if selected_workflow_version_id.get().is_empty()
            || selected_node_id.get().is_empty()
            || selected_pair_is_valid()
        {
            None
        } else {
            Some("That workflow is not valid for the selected node.".to_string())
        }
    };
    let selected_workflow_summary = move || {
        let selected_id = selected_workflow_version_id.get();
        candidates
            .get()
            .into_iter()
            .find(|candidate| candidate.workflow_version_id == selected_id)
            .map(|candidate| {
                let revision =
                    workflow_assignment_revision_label(candidate.workflow_version_label.as_deref());
                (candidate.workflow_name, format!("Revision {revision}"))
            })
    };
    let selected_node_summary = move || {
        let selected_id = selected_node_id.get();
        candidates
            .get()
            .into_iter()
            .find(|candidate| candidate.node_id == selected_id)
            .map(|candidate| {
                let node_path = if candidate.node_path.trim().is_empty() {
                    candidate.node_name.clone()
                } else {
                    candidate.node_path.clone()
                };
                (candidate.node_name, node_path)
            })
    };
    let filtered_assignees = move || {
        let query = assignee_search.get();
        assignees
            .get()
            .into_iter()
            .filter(|assignee| {
                text_matches(
                    &query,
                    &[assignee.display_name.as_str(), assignee.email.as_str()],
                )
            })
            .collect::<Vec<_>>()
    };
    let filtered_assignments = move || {
        let query = assignment_search.get();
        let status = status_filter.get();
        let state = state_filter.get();
        let assignee = assignee_filter.get();
        assignments
            .get()
            .into_iter()
            .filter(|assignment| {
                let matches_status =
                    status == "all" || workflow_assignment_status_key(assignment) == status;
                let matches_state =
                    state == "all" || workflow_assignment_state(assignment) == state;
                let matches_assignee =
                    assignee == "all" || workflow_assignment_assignee_label(assignment) == assignee;
                matches_status
                    && matches_state
                    && matches_assignee
                    && text_matches(
                        &query,
                        &[
                            assignment.workflow_name.as_str(),
                            assignment.workflow_step_title.as_str(),
                            assignment.form_name.as_str(),
                            assignment.node_name.as_str(),
                            assignment.account_display_name.as_str(),
                            assignment.account_email.as_str(),
                            assignment.id.as_str(),
                        ],
                    )
            })
            .collect::<Vec<_>>()
    };
    let assignee_filter_options = move || {
        unique_filter_options(
            assignments
                .get()
                .iter()
                .map(workflow_assignment_assignee_label)
                .collect::<Vec<_>>(),
        )
    };
    let can_create = move || {
        !is_saving.get()
            && !selected_candidate_id.get().is_empty()
            && !selected_account_ids.get().is_empty()
    };

    view! {
        <AppShell active_route="workflows" title="Workflow Assignments">
            <div class="app-page">
                <Breadcrumb>
                    <BreadcrumbItem>
                        <BreadcrumbLink href="/workflows">"Workflows"</BreadcrumbLink>
                    </BreadcrumbItem>
                    <BreadcrumbSeparator/>
                    <BreadcrumbItem>
                        <BreadcrumbPage>"Assignments"</BreadcrumbPage>
                    </BreadcrumbItem>
                </Breadcrumb>

                <section class="route-panel workflows-page workflow-assignments-page">
                    <PageHeader title="Workflow Assignments"/>

                    <form
                        class="native-form workflow-assignment-create-form"
                        on:submit=move |event| {
                            event.prevent_default();
                            submit_workflow_assignment_bulk(
                                selected_candidate_id,
                                candidates,
                                selected_account_ids,
                                assignments,
                                assignments_loading,
                                assignments_error,
                                is_saving,
                                message,
                            );
                        }
                    >
                        <section class="form-section">
                            <div class="form-builder-section-card__header">
                                <h3>"Create Assignment"</h3>
                            </div>
                            <div class="workflow-assignment-create-grid">
                                <section class="workflow-assignment-pair-list" aria-labelledby="workflow-assignment-workflow-list">
                                    <div class="workflow-assignment-pair-list__header">
                                        <h4 id="workflow-assignment-workflow-list">"Workflow"</h4>
                                    </div>
                                    {move || {
                                        if let Some((workflow_name, version)) = selected_workflow_summary() {
                                            view! {
                                                <div class="workflow-assignment-selected-option">
                                                    <div>
                                                        <strong>{workflow_name}</strong>
                                                        <span>{version}</span>
                                                    </div>
                                                    <button
                                                        class="icon-button icon-button--control"
                                                        type="button"
                                                        aria-label="Clear selected workflow"
                                                        on:click=move |_| {
                                                            selected_workflow_version_id.set(String::new());
                                                            selected_candidate_id.set(String::new());
                                                            selected_account_ids.set(HashSet::new());
                                                        }
                                                    >
                                                        <X/>
                                                    </button>
                                                </div>
                                            }.into_any()
                                        } else {
                                            view! {
                                                <label class="searchable-data-table__search searchable-data-table__control workflow-assignment-candidate-search">
                                                    <Search class="searchable-data-table__control-icon"/>
                                                    <span class="sr-only">"Search workflows"</span>
                                                    <input
                                                        type="search"
                                                        placeholder="Search workflows"
                                                        prop:value=move || workflow_search.get()
                                                        on:input=move |event| workflow_search.set(event_target_value(&event))
                                                    />
                                                </label>
                                                <div class="workflow-assignment-pair-list__options">
                                                    {move || {
                                                        let options = filtered_workflow_candidates();
                                                        if options.is_empty() {
                                                            view! { <p class="workflow-assignee-picker__empty">"No Workflows to Display"</p> }.into_any()
                                                        } else {
                                                            options.into_iter().map(|candidate| {
                                                                let workflow_version_id = candidate.workflow_version_id.clone();
                                                                let workflow_version_id_for_class = workflow_version_id.clone();
                                                                let revision = workflow_assignment_revision_label(candidate.workflow_version_label.as_deref());
                                                                view! {
                                                                    <button
                                                                        class=move || if selected_workflow_version_id.get() == workflow_version_id_for_class {
                                                                            "workflow-assignment-pair-option is-selected"
                                                                        } else {
                                                                            "workflow-assignment-pair-option"
                                                                        }
                                                                        type="button"
                                                                        disabled=move || candidates_loading.get()
                                                                        on:click=move |_| {
                                                                            let workflow_version_id = workflow_version_id.clone();
                                                                            selected_workflow_version_id.set(workflow_version_id.clone());
                                                                            let selected_node = selected_node_id.get_untracked();
                                                                            if !selected_node.is_empty()
                                                                                && !candidates.get_untracked().into_iter().any(|candidate| {
                                                                                    candidate.workflow_version_id == workflow_version_id
                                                                                        && candidate.node_id == selected_node
                                                                                })
                                                                            {
                                                                                selected_node_id.set(String::new());
                                                                            }
                                                                        }
                                                                    >
                                                                        <strong>{candidate.workflow_name}</strong>
                                                                        <span>{format!("Revision {revision}")}</span>
                                                                    </button>
                                                                }
                                                            }).collect_view().into_any()
                                                        }
                                                    }}
                                                </div>
                                            }.into_any()
                                        }
                                    }}
                                </section>
                                <section class="workflow-assignment-pair-list" aria-labelledby="workflow-assignment-node-list">
                                    <div class="workflow-assignment-pair-list__header">
                                        <h4 id="workflow-assignment-node-list">"Node"</h4>
                                    </div>
                                    {move || {
                                        if let Some((node_name, node_path)) = selected_node_summary() {
                                            view! {
                                                <div class="workflow-assignment-selected-option">
                                                    <div>
                                                        <strong>{node_name}</strong>
                                                        <span>{node_path}</span>
                                                    </div>
                                                    <button
                                                        class="icon-button icon-button--control"
                                                        type="button"
                                                        aria-label="Clear selected node"
                                                        on:click=move |_| {
                                                            selected_node_id.set(String::new());
                                                            selected_candidate_id.set(String::new());
                                                            selected_account_ids.set(HashSet::new());
                                                        }
                                                    >
                                                        <X/>
                                                    </button>
                                                </div>
                                            }.into_any()
                                        } else {
                                            view! {
                                                <label class="searchable-data-table__search searchable-data-table__control workflow-assignment-candidate-search">
                                                    <Search class="searchable-data-table__control-icon"/>
                                                    <span class="sr-only">"Search nodes"</span>
                                                    <input
                                                        type="search"
                                                        placeholder="Search nodes"
                                                        prop:value=move || node_search.get()
                                                        on:input=move |event| node_search.set(event_target_value(&event))
                                                    />
                                                </label>
                                                <div class="workflow-assignment-pair-list__options">
                                                    {move || {
                                                        let options = filtered_node_candidates();
                                                        if options.is_empty() {
                                                            view! { <p class="workflow-assignee-picker__empty">"No Nodes to Display"</p> }.into_any()
                                                        } else {
                                                            options.into_iter().map(|candidate| {
                                                                let node_id = candidate.node_id.clone();
                                                                let node_id_for_class = node_id.clone();
                                                                view! {
                                                                    <button
                                                                        class=move || if selected_node_id.get() == node_id_for_class {
                                                                            "workflow-assignment-pair-option is-selected"
                                                                        } else {
                                                                            "workflow-assignment-pair-option"
                                                                        }
                                                                        type="button"
                                                                        disabled=move || candidates_loading.get()
                                                                        on:click=move |_| {
                                                                            let node_id = node_id.clone();
                                                                            selected_node_id.set(node_id.clone());
                                                                            let selected_workflow = selected_workflow_version_id.get_untracked();
                                                                            if !selected_workflow.is_empty()
                                                                                && !candidates.get_untracked().into_iter().any(|candidate| {
                                                                                    candidate.workflow_version_id == selected_workflow
                                                                                        && candidate.node_id == node_id
                                                                                })
                                                                            {
                                                                                selected_workflow_version_id.set(String::new());
                                                                            }
                                                                        }
                                                                    >
                                                                        <strong>{candidate.node_name}</strong>
                                                                        <span>{candidate.node_path}</span>
                                                                    </button>
                                                                }
                                                            }).collect_view().into_any()
                                                        }
                                                    }}
                                                </div>
                                            }.into_any()
                                        }
                                    }}
                                </section>
                            </div>
                            {move || invalid_pair_message().map(|message| view! {
                                <p class="form-message" role="alert">{message}</p>
                            })}
                            {move || {
                                if let Some(message) = candidates_error.get() {
                                    view! {
                                        <section class="organization-state is-error" role="alert">
                                            <h3>"Assignment candidates unavailable"</h3>
                                            <p>{message}</p>
                                        </section>
                                    }
                                    .into_any()
                                } else if candidates_loading.get() {
                                    view! {
                                        <section class="organization-state" aria-live="polite">
                                            <h3>"Loading candidates"</h3>
                                            <p>"Fetching eligible workflow and node combinations."</p>
                                        </section>
                                    }
                                    .into_any()
                                } else {
                                    empty_view()
                                }
                            }}
                            <div class="workflow-assignee-picker">
                                <h4>"Eligible Assignees"</h4>
                                {move || if selected_candidate_id.get().is_empty() {
                                    empty_view()
                                } else {
                                    view! {
                                        <label class="searchable-data-table__search searchable-data-table__control workflow-assignment-candidate-search">
                                            <Search class="searchable-data-table__control-icon"/>
                                            <span class="sr-only">"Search assignees"</span>
                                            <input
                                                type="search"
                                                placeholder="Search assignees"
                                                prop:value=move || assignee_search.get()
                                                on:input=move |event| assignee_search.set(event_target_value(&event))
                                            />
                                        </label>
                                    }.into_any()
                                }}
                                {move || {
                                    if selected_candidate_id.get().is_empty() {
                                        view! { <p class="workflow-assignee-picker__empty">"Select a candidate to load assignees."</p> }.into_any()
                                    } else if assignees_loading.get() {
                                        view! { <p class="workflow-assignee-picker__empty">"Loading assignees."</p> }.into_any()
                                    } else if let Some(message) = assignees_error.get() {
                                        view! { <p class="workflow-assignee-picker__empty">{message}</p> }.into_any()
                                    } else {
                                        let options = filtered_assignees();
                                        if options.is_empty() {
                                            view! { <p class="workflow-assignee-picker__empty">"No eligible assignees to display."</p> }.into_any()
                                        } else {
                                            options
                                                .into_iter()
                                                .map(|assignee| {
                                                    let account_id = assignee.account_id.clone();
                                                    let account_id_for_checked = account_id.clone();
                                                    let label = workflow_assignee_label(&assignee);
                                                    view! {
                                                        <label class="workflow-assignee-option">
                                                            <input
                                                                type="checkbox"
                                                                prop:checked=move || selected_account_ids.get().contains(&account_id_for_checked)
                                                                on:change=move |event| {
                                                                    let is_checked = event_target_checked(&event);
                                                                    let account_id = account_id.clone();
                                                                    selected_account_ids.update(|selected| {
                                                                        if is_checked {
                                                                            selected.insert(account_id);
                                                                        } else {
                                                                            selected.remove(&account_id);
                                                                        }
                                                                    });
                                                                }
                                                            />
                                                            <span>{label}</span>
                                                        </label>
                                                    }
                                                })
                                                .collect_view()
                                                .into_any()
                                        }
                                    }
                                }}
                            </div>
                            <div class="form-actions">
                                <button class="button button--secondary" type="submit" disabled=move || !can_create()>
                                    {move || if is_saving.get() { "Creating..." } else { "Create Assignments" }}
                                </button>
                            </div>
                        </section>
                    </form>

                    {move || message.get().map(|message| view! {
                        <p class="form-message" role="status">{message}</p>
                    })}

                    {move || {
                        if assignments_loading.get() {
                            view! {
                                <section class="organization-state" aria-live="polite">
                                    <h3>"Loading assignments"</h3>
                                    <p>"Fetching workflow assignment records."</p>
                                </section>
                            }
                            .into_any()
                        } else if let Some(message) = assignments_error.get() {
                            view! {
                                <section class="organization-state is-error" role="alert">
                                    <h3>"Workflow assignments unavailable"</h3>
                                    <p>{message}</p>
                                </section>
                            }
                            .into_any()
                        } else {
                            view! {
                                <WorkflowAssignmentsList
                                    assignments=filtered_assignments()
                                    search=assignment_search
                                    status_filter=status_filter
                                    state_filter=state_filter
                                    assignee_filter=assignee_filter
                                    assignee_options=assignee_filter_options()
                                    assignments_signal=assignments
                                    assignments_loading=assignments_loading
                                    assignments_error=assignments_error
                                    message=message
                                />
                            }
                            .into_any()
                        }
                    }}
                </section>
            </div>
        </AppShell>
    }
}

#[component]
fn WorkflowAssignmentsList(
    assignments: Vec<WorkflowAssignmentSummary>,
    search: RwSignal<String>,
    status_filter: RwSignal<String>,
    state_filter: RwSignal<String>,
    assignee_filter: RwSignal<String>,
    assignee_options: Vec<String>,
    assignments_signal: RwSignal<Vec<WorkflowAssignmentSummary>>,
    assignments_loading: RwSignal<bool>,
    assignments_error: RwSignal<Option<String>>,
    message: RwSignal<Option<String>>,
) -> impl IntoView {
    let mut table_assignments = assignments.clone();
    table_assignments.sort_by(|left, right| {
        left.workflow_name
            .to_lowercase()
            .cmp(&right.workflow_name.to_lowercase())
            .then(
                left.account_display_name
                    .to_lowercase()
                    .cmp(&right.account_display_name.to_lowercase()),
            )
            .then(left.id.cmp(&right.id))
    });
    let card_assignments = table_assignments.clone();
    let page_size = RwSignal::new(10usize);
    let page_index = RwSignal::new(0usize);
    let total_count = table_assignments.len();
    let page_count = move || {
        if total_count == 0 {
            1
        } else {
            ((total_count + page_size.get() - 1) / page_size.get()).max(1)
        }
    };
    let current_page = move || page_index.get().min(page_count() - 1);
    let page_start = move || {
        if total_count == 0 {
            0
        } else {
            current_page() * page_size.get()
        }
    };
    let page_end = move || (page_start() + page_size.get()).min(total_count);
    let page_summary = move || {
        if total_count == 0 {
            "No workflow assignments to display".to_string()
        } else {
            format!(
                "Showing {}-{} of {} workflow assignments",
                page_start() + 1,
                page_end(),
                total_count
            )
        }
    };
    let selected_detail = RwSignal::new(None::<WorkflowAssignmentSummary>);
    let close_detail = move |_| selected_detail.set(None);

    view! {
        <div class="forms-list forms-list-responsive-table workflow-assignments-list">
            <div class="searchable-data-table">
                <div class="searchable-data-table__toolbar forms-list__toolbar">
                    <label class="searchable-data-table__search searchable-data-table__control">
                        <Search class="searchable-data-table__control-icon"/>
                        <span class="sr-only">"Search assignments"</span>
                        <input
                            type="search"
                            placeholder="Search assignments"
                            prop:value=move || search.get()
                            on:input=move |event| search.set(event_target_value(&event))
                        />
                    </label>
                </div>
                <DataTable>
                    <thead>
                        <tr>
                            <th scope="col">"Workflow"</th>
                            <th scope="col">
                                <FilterHeader
                                    label="Assignee"
                                    all_label="All Assignees"
                                    filter=assignee_filter
                                    options=assignee_options
                                    always_searchable=true
                                />
                            </th>
                            <th class="data-table__cell--center" scope="col">
                                <FilterHeader
                                    label="Work State"
                                    all_label="All States"
                                    filter=state_filter
                                    options=vec!["pending".into(), "draft".into(), "submitted".into()]
                                />
                            </th>
                            <th class="data-table__cell--center" scope="col">
                                <FilterHeader
                                    label="Status"
                                    all_label="All Statuses"
                                    filter=status_filter
                                    options=vec!["active".into(), "inactive".into()]
                                />
                            </th>
                            <th scope="col">"Assigned"</th>
                            <th class="data-table__cell--center" scope="col">"Actions"</th>
                        </tr>
                    </thead>
                    <tbody>
                        {move || if table_assignments.is_empty() {
                            view! {
                                <tr>
                                    <td class="data-table__empty" colspan="6">"No Workflow Assignments to Display"</td>
                                </tr>
                            }
                            .into_any()
                        } else {
                            table_assignments
                                .iter()
                                .skip(page_start())
                                .take(page_size.get())
                                .cloned()
                                .map(|assignment| {
                                    let workflow_href = format!("/workflows/{}", assignment.workflow_id);
                                    let state_label = workflow_assignment_state_label(&assignment);
                                    let state_key = workflow_assignment_state(&assignment);
                                    let status_key = workflow_assignment_status_key(&assignment);
                                    let status_label = workflow_assignment_status_label(&assignment);
                                    let action_label = if assignment.is_active { "Deactivate" } else { "Activate" };
                                    let assignment_for_toggle = assignment.clone();
                                    let assignment_for_detail = assignment.clone();
                                    view! {
                                        <tr>
                                            <th scope="row">
                                                <a class="data-table__primary-link" href=workflow_href>{assignment.workflow_name.clone()}</a>
                                            </th>
                                            <td>
                                                <span>{assignment.account_display_name}</span>
                                                <small class="workflow-assignment-step-meta">{assignment.account_email}</small>
                                            </td>
                                            <td class="data-table__cell--center">
                                                <span class=status_badge_class(state_key)>{state_label}</span>
                                            </td>
                                            <td class="data-table__cell--center">
                                                <span class=status_badge_class(status_key)>{status_label}</span>
                                            </td>
                                            <td><Timestamp value=assignment.created_at/></td>
                                            <td class="data-table__cell--center">
                                                <DropdownMenu label=format!("Open actions for {}", assignment.workflow_name)>
                                                    <button
                                                        class="dropdown-menu__item"
                                                        type="button"
                                                        role="menuitem"
                                                        on:click=move |_| selected_detail.set(Some(assignment_for_detail.clone()))
                                                    >
                                                        <PanelRight class="dropdown-menu__item-icon"/>
                                                        <span>"View Details"</span>
                                                    </button>
                                                    <button
                                                        class="dropdown-menu__item"
                                                        type="button"
                                                        role="menuitem"
                                                        on:click=move |_| {
                                                            toggle_workflow_assignment(
                                                                assignment_for_toggle.clone(),
                                                                assignments_signal,
                                                                assignments_loading,
                                                                assignments_error,
                                                                message,
                                                            );
                                                        }
                                                    >
                                                        <X class="dropdown-menu__item-icon"/>
                                                        <span>{action_label}</span>
                                                    </button>
                                                </DropdownMenu>
                                            </td>
                                        </tr>
                                    }
                                })
                                .collect_view()
                                .into_any()
                        }}
                    </tbody>
                </DataTable>
                <div class="directory-table-pagination" aria-label="Workflow assignments table pagination">
                    <p>{move || page_summary()}</p>
                    <div class="directory-table-pagination__actions">
                        <label class="directory-table-pagination__page-size searchable-data-table__filter searchable-data-table__control">
                            <span>"Rows"</span>
                            <select
                                prop:value=move || page_size.get().to_string()
                                on:change=move |event| {
                                    if let Ok(size) = event_target_value(&event).parse::<usize>() {
                                        page_size.set(size);
                                        page_index.set(0);
                                    }
                                }
                            >
                                <option value="10">"10"</option>
                                <option value="25">"25"</option>
                                <option value="50">"50"</option>
                            </select>
                        </label>
                        <button
                            class="button button--compact button--secondary"
                            type="button"
                            disabled=move || current_page() == 0
                            on:click=move |_| {
                                page_index.update(|page| *page = page.saturating_sub(1));
                            }
                        >
                            "Previous"
                        </button>
                        <span>{move || format!("Page {} of {}", current_page() + 1, page_count())}</span>
                        <button
                            class="button button--compact button--secondary"
                            type="button"
                            disabled=move || { current_page() + 1 >= page_count() }
                            on:click=move |_| {
                                let last_page = page_count().saturating_sub(1);
                                page_index.update(|page| *page = (*page + 1).min(last_page));
                            }
                        >
                            "Next"
                        </button>
                    </div>
                </div>
            </div>
            <div class="forms-list-mobile-cards workflow-assignment-mobile-cards">
                {move || if card_assignments.is_empty() {
                    view! { <p class="forms-list-mobile-empty">"No Workflow Assignments to Display"</p> }.into_any()
                } else {
                    card_assignments
                        .iter()
                        .skip(page_start())
                        .take(page_size.get())
                        .cloned()
                        .map(|assignment| {
                            let workflow_href = format!("/workflows/{}", assignment.workflow_id);
                            let node_href = format!("/organization/{}", assignment.node_id);
                            let state_label = workflow_assignment_state_label(&assignment);
                            let state_key = workflow_assignment_state(&assignment);
                            let status_key = workflow_assignment_status_key(&assignment);
                            let status_label = workflow_assignment_status_label(&assignment);
                            let action_label = if assignment.is_active { "Deactivate" } else { "Activate" };
                            let assignment_for_toggle = assignment.clone();
                            let assignment_for_detail = assignment.clone();
                            view! {
                                <article class="forms-list-mobile-card workflow-assignment-mobile-card">
                                    <div class="forms-list-mobile-card__header">
                                        <div class="forms-list-mobile-card__title-row">
                                            <h3><a href=workflow_href>{assignment.workflow_name}</a></h3>
                                        </div>
                                    </div>
                                    <dl>
                                        <div>
                                            <dt>"Assignee"</dt>
                                            <dd>
                                                <span>{assignment.account_display_name}</span>
                                                <small class="workflow-assignment-step-meta">{assignment.account_email}</small>
                                            </dd>
                                        </div>
                                        <div>
                                            <dt>"Form"</dt>
                                            <dd>{assignment.form_name}</dd>
                                        </div>
                                        <div>
                                            <dt>"Node"</dt>
                                            <dd><a href=node_href>{assignment.node_name}</a></dd>
                                        </div>
                                        <div>
                                            <dt>"Step"</dt>
                                            <dd>{assignment.workflow_step_title}</dd>
                                        </div>
                                        <div>
                                            <dt>"Work State"</dt>
                                            <dd><span class=status_badge_class(state_key)>{state_label}</span></dd>
                                        </div>
                                        <div>
                                            <dt>"Status"</dt>
                                            <dd><span class=status_badge_class(status_key)>{status_label}</span></dd>
                                        </div>
                                        <div>
                                            <dt>"Assigned"</dt>
                                            <dd><Timestamp value=assignment.created_at/></dd>
                                        </div>
                                    </dl>
                                    <div class="workflow-assignment-mobile-card__actions">
                                        <button
                                            class="button button--compact"
                                            type="button"
                                            on:click=move |_| selected_detail.set(Some(assignment_for_detail.clone()))
                                        >
                                            "View Details"
                                        </button>
                                        <button
                                            class="button button--compact"
                                            type="button"
                                            on:click=move |_| {
                                                toggle_workflow_assignment(
                                                    assignment_for_toggle.clone(),
                                                    assignments_signal,
                                                    assignments_loading,
                                                    assignments_error,
                                                    message,
                                                );
                                            }
                                        >
                                            {action_label}
                                        </button>
                                    </div>
                                </article>
                            }
                        })
                        .collect_view()
                        .into_any()
                }}
            </div>
        </div>
        {move || selected_detail.get().map(|assignment| {
            let workflow_href = format!("/workflows/{}", assignment.workflow_id);
            let node_href = format!("/organization/{}", assignment.node_id);
            let state_key = workflow_assignment_state(&assignment);
            let state_label = workflow_assignment_state_label(&assignment);
            let status_key = workflow_assignment_status_key(&assignment);
            let status_label = workflow_assignment_status_label(&assignment);

            view! {
                <Portal>
                    <section class="sheet-overlay workflow-assignment-detail-overlay" aria-label="Workflow assignment detail">
                        <button class="sheet-overlay__scrim" type="button" aria-label="Close assignment details" on:click=close_detail></button>
                        <aside class="sheet-panel blurred-surface workflow-assignment-detail-sheet" role="dialog" aria-modal="true" aria-label="Workflow assignment details">
                            <div class="sheet-panel__actions">
                                <button class="icon-button sheet-panel__close" type="button" aria-label="Close assignment details" title="Close assignment details" on:click=close_detail>
                                    <X class="icon-button__icon"/>
                                </button>
                            </div>
                            <header class="sheet-panel__header">
                                <p>"Assignment Detail"</p>
                                <h2>{assignment.workflow_name.clone()}</h2>
                            </header>
                            <section class="sheet-panel__section">
                                <h3>"Workflow"</h3>
                                <table class="info-list-table">
                                    <tbody>
                                        <tr>
                                            <th scope="row">"Workflow"</th>
                                                <td><a class="data-table__primary-link" href=workflow_href.clone()>{assignment.workflow_name.clone()}</a></td>
                                        </tr>
                                        <tr>
                                            <th scope="row">"Revision"</th>
                                            <td>{workflow_assignment_revision_label(assignment.workflow_version_label.as_deref())}</td>
                                        </tr>
                                        <tr>
                                            <th scope="row">"Step"</th>
                                            <td>{assignment.workflow_step_title.clone()}</td>
                                        </tr>
                                        <tr>
                                            <th scope="row">"Form"</th>
                                            <td>{assignment.form_name.clone()}</td>
                                        </tr>
                                        <tr>
                                            <th scope="row">"Form Version"</th>
                                            <td>{nonempty_text(assignment.form_version_label.as_deref(), "-")}</td>
                                        </tr>
                                    </tbody>
                                </table>
                            </section>
                            <section class="sheet-panel__section">
                                <h3>"Assignment"</h3>
                                <table class="info-list-table">
                                    <tbody>
                                        <tr>
                                            <th scope="row">"Node"</th>
                                                <td><a class="data-table__primary-link" href=node_href.clone()>{assignment.node_name.clone()}</a></td>
                                        </tr>
                                        <tr>
                                            <th scope="row">"Assignee"</th>
                                            <td>{assignment.account_display_name.clone()}</td>
                                        </tr>
                                        <tr>
                                            <th scope="row">"Email"</th>
                                            <td>{assignment.account_email.clone()}</td>
                                        </tr>
                                        <tr>
                                            <th scope="row">"Work State"</th>
                                            <td><span class=status_badge_class(state_key)>{state_label}</span></td>
                                        </tr>
                                        <tr>
                                            <th scope="row">"Status"</th>
                                            <td><span class=status_badge_class(status_key)>{status_label}</span></td>
                                        </tr>
                                        <tr>
                                            <th scope="row">"Assigned"</th>
                                            <td><Timestamp value=assignment.created_at.clone()/></td>
                                        </tr>
                                    </tbody>
                                </table>
                            </section>
                        </aside>
                    </section>
                </Portal>
            }
        })}
    }
}

#[component]
pub fn WorkflowsDetailPage() -> impl IntoView {
    let params = require_route_params::<WorkflowRouteParams>();
    let workflow_id = params.workflow_id;
    let detail = RwSignal::new(None::<WorkflowDefinition>);
    let is_loading = RwSignal::new(true);
    let error = RwSignal::new(None::<String>);

    Effect::new(move |_| {
        load_workflow_detail(workflow_id.clone(), detail, is_loading, error);
    });

    view! {
        <AppShell active_route="workflows" title="Workflows">
            <Breadcrumb>
                <BreadcrumbItem>
                    <BreadcrumbLink href="/workflows">"Workflows"</BreadcrumbLink>
                </BreadcrumbItem>
                <BreadcrumbSeparator/>
                {move || {
                    detail.get().map(|workflow| {
                        view! {
                            <BreadcrumbItem>
                                <BreadcrumbPage>{workflow.name}</BreadcrumbPage>
                            </BreadcrumbItem>
                        }
                    })
                }}
                {move || {
                    if detail.get().is_none() {
                        view! {
                            <BreadcrumbItem>
                                <BreadcrumbPage>"Detail"</BreadcrumbPage>
                            </BreadcrumbItem>
                        }
                        .into_any()
                    } else {
                        empty_view()
                    }
                }}
            </Breadcrumb>

            <section class="route-panel workflows-page workflow-detail-page">
                {move || {
                    if is_loading.get() {
                        view! {
                            <section class="organization-state" aria-live="polite">
                                <h3>"Loading workflow"</h3>
                                <p>"Fetching workflow details."</p>
                            </section>
                        }
                        .into_any()
                    } else if let Some(message) = error.get() {
                        view! {
                            <section class="organization-state is-error" role="alert">
                                <h3>"Workflow detail unavailable"</h3>
                                <p>{message}</p>
                            </section>
                        }
                        .into_any()
                    } else if let Some(workflow) = detail.get() {
                        let assignments_href =
                            format!("/workflows/assignments?workflow_id={}", workflow.id);
                        view! {
                            <PageHeader title="Workflow Detail">
                                <a class="button button--secondary" href=assignments_href>"Manage Assignments"</a>
                            </PageHeader>
                            <WorkflowDetailContent workflow/>
                        }
                        .into_any()
                    } else {
                        view! {
                            <EmptyState
                                title="Workflow detail unavailable"
                                message="The selected workflow could not be loaded."
                            />
                        }
                        .into_any()
                    }
                }}
            </section>
        </AppShell>
    }
}

#[component]
fn WorkflowDetailContent(workflow: WorkflowDefinition) -> impl IntoView {
    let steps_expanded = RwSignal::new(false);
    let revisions_expanded = RwSignal::new(false);
    let assignments_expanded = RwSignal::new(false);
    let active_version = active_workflow_definition_version(&workflow).cloned();
    let active_status = active_version
        .as_ref()
        .map(|version| version.status.clone())
        .unwrap_or_else(|| "none".to_string());
    let active_version_label = workflow_definition_version_label(active_version.as_ref());
    let active_status_label = workflow_definition_status_label(active_version.as_ref());
    let active_step_count = active_version
        .as_ref()
        .map(|version| version.step_count.to_string())
        .unwrap_or_else(|| "-".to_string());
    let steps_toggle_count = active_step_count.clone();
    let published_at = active_version
        .as_ref()
        .and_then(|version| version.published_at.clone());
    let workflow_id = workflow.id.clone();
    let workflow_name = workflow.name.clone();
    let workflow_slug = workflow.slug.clone();
    let workflow_description = nonempty_text(Some(workflow.description.as_str()), "No description");
    let workflow_available_at = workflow_available_nodes_label(&workflow.available_nodes);
    let workflow_source = workflow_source_label(&workflow.source)
        .unwrap_or("Authored")
        .to_string();
    let revision_count = workflow.versions.len().to_string();
    let assignment_count = workflow.assignments.len().to_string();
    let revisions_toggle_count = revision_count.clone();
    let assignments_toggle_count = assignment_count.clone();
    let steps = active_version
        .as_ref()
        .map(|version| version.steps.clone())
        .unwrap_or_default();
    let versions = workflow.versions.clone();
    let assignments = workflow.assignments.clone();

    view! {
        <div class="organization-detail-content workflow-detail-content">
            <header class="organization-detail-content__header">
                <p>"Workflow Detail"</p>
                <h2>{workflow_name}</h2>
            </header>

            <div class="organization-detail-content__grid">
                <section class="organization-detail-card">
                    <h3>"Details"</h3>
                    <InfoListTable>
                        <tr>
                            <th scope="row">"Slug"</th>
                            <td>{workflow_slug}</td>
                        </tr>
                        <tr>
                            <th scope="row">"Description"</th>
                            <td>{workflow_description}</td>
                        </tr>
                        <tr>
                            <th scope="row">"Available At"</th>
                            <td>{workflow_available_at}</td>
                        </tr>
                        <tr>
                            <th scope="row">"Source"</th>
                            <td>{workflow_source}</td>
                        </tr>
                        <tr>
                            <th scope="row">"Revisions"</th>
                            <td>{revision_count}</td>
                        </tr>
                        <tr>
                            <th scope="row">"Assignments"</th>
                            <td>{assignment_count}</td>
                        </tr>
                    </InfoListTable>
                </section>

                <section class="organization-detail-card">
                    <h3>"Active Revision"</h3>
                    <InfoListTable>
                        <tr>
                            <th scope="row">"Revision"</th>
                            <td>{active_version_label}</td>
                        </tr>
                        <tr>
                            <th scope="row">"Status"</th>
                            <td><span class=status_badge_class(&active_status)>{active_status_label}</span></td>
                        </tr>
                        <tr>
                            <th scope="row">"Steps"</th>
                            <td>{active_step_count}</td>
                        </tr>
                        <tr>
                            <th scope="row">"Published"</th>
                            <td>
                                {published_at
                                    .map(|value| view! { <Timestamp value/> }.into_any())
                                    .unwrap_or_else(|| view! { <span>"-"</span> }.into_any())}
                            </td>
                        </tr>
                    </InfoListTable>
                </section>

                <section class="organization-detail-card organization-detail-card--wide form-detail-fields-card">
                    <header class="form-detail-disclosure-header">
                        <h3>"Steps"</h3>
                        <button
                            class="link-button form-detail-disclosure-toggle"
                            type="button"
                            aria-expanded=move || steps_expanded.get().to_string()
                            on:click=move |_| steps_expanded.update(|expanded| *expanded = !*expanded)
                        >
                            {move || {
                                if steps_expanded.get() {
                                    "Hide Steps".to_string()
                                } else {
                                    format!("Show {steps_toggle_count} Steps")
                                }
                            }}
                        </button>
                    </header>
                    {move || {
                        if steps_expanded.get() {
                            view! { <WorkflowStepsTable steps=steps.clone()/> }.into_any()
                        } else {
                            empty_view()
                        }
                    }}
                </section>

                <section class="organization-detail-card organization-detail-card--wide form-detail-fields-card">
                    <header class="form-detail-disclosure-header">
                        <h3>"Revisions"</h3>
                        <button
                            class="link-button form-detail-disclosure-toggle"
                            type="button"
                            aria-expanded=move || revisions_expanded.get().to_string()
                            on:click=move |_| revisions_expanded.update(|expanded| *expanded = !*expanded)
                        >
                            {move || {
                                if revisions_expanded.get() {
                                    "Hide Revisions".to_string()
                                } else {
                                    format!("Show {revisions_toggle_count} Revisions")
                                }
                            }}
                        </button>
                    </header>
                    {move || {
                        if revisions_expanded.get() {
                            view! { <WorkflowVersionsTable workflow_id=workflow_id.clone() versions=versions.clone()/> }.into_any()
                        } else {
                            empty_view()
                        }
                    }}
                </section>

                <section class="organization-detail-card organization-detail-card--wide form-detail-fields-card workflow-detail-assignments-card">
                    <header class="form-detail-disclosure-header">
                        <h3>"Assignments"</h3>
                        <button
                            class="link-button form-detail-disclosure-toggle"
                            type="button"
                            aria-expanded=move || assignments_expanded.get().to_string()
                            on:click=move |_| assignments_expanded.update(|expanded| *expanded = !*expanded)
                        >
                            {move || {
                                if assignments_expanded.get() {
                                    "Hide Assignments".to_string()
                                } else {
                                    format!("Show {assignments_toggle_count} Assignments")
                                }
                            }}
                        </button>
                    </header>
                    {move || {
                        if assignments_expanded.get() {
                            view! { <WorkflowDetailAssignmentsTable assignments=assignments.clone()/> }.into_any()
                        } else {
                            empty_view()
                        }
                    }}
                </section>
            </div>
        </div>
    }
}

#[component]
fn WorkflowStepsTable(steps: Vec<WorkflowStepSummary>) -> impl IntoView {
    view! {
        <DataTable>
            <thead>
                <tr>
                    <th scope="col">"Step"</th>
                    <th scope="col">"Form"</th>
                    <th scope="col">"Form Version"</th>
                </tr>
            </thead>
            <tbody>
                {if steps.is_empty() {
                    view! {
                        <tr>
                            <td class="data-table__empty" colspan="3">"No Workflow Steps to Display"</td>
                        </tr>
                    }
                    .into_any()
                } else {
                    steps
                        .into_iter()
                        .map(|step| {
                            let form_href = format!("/forms/{}", step.form_id);
                            let step_title = nonempty_text(Some(&step.title), "Untitled step");
                            view! {
                                <tr>
                                    <th scope="row">{step_title}</th>
                                    <td><a class="data-table__primary-link" href=form_href>{step.form_name}</a></td>
                                    <td>{nonempty_text(step.form_version_label.as_deref(), "-")}</td>
                                </tr>
                            }
                        })
                        .collect_view()
                        .into_any()
                }}
            </tbody>
        </DataTable>
    }
}

#[component]
fn WorkflowVersionsTable(
    workflow_id: String,
    versions: Vec<WorkflowVersionSummary>,
) -> impl IntoView {
    view! {
        <DataTable>
            <thead>
                <tr>
                    <th scope="col">"Revision"</th>
                    <th scope="col">"Status"</th>
                    <th scope="col">"Published"</th>
                    <th class="data-table__cell--center" scope="col">"Steps"</th>
                    <th class="data-table__cell--center" scope="col">"Actions"</th>
                </tr>
            </thead>
            <tbody>
                {if versions.is_empty() {
                    view! {
                        <tr>
                            <td class="data-table__empty" colspan="5">"No Revisions to Display"</td>
                        </tr>
                    }
                    .into_any()
                } else {
                    versions
                        .into_iter()
                        .map(|version| {
                            let status = version.status.clone();
                            let published_at = version.published_at.clone();
                            let version_label = workflow_revision_label_from_option(version.workflow_revision_label);
                            let edit_href = format!("/workflows/{}/edit?version_id={}", workflow_id, version.id);
                            let edit_title = format!("Edit {} workflow revision", sentence_label(&status));
                            view! {
                                <tr>
                                    <th scope="row">{version_label}</th>
                                    <td><span class=status_badge_class(&status)>{sentence_label(&status)}</span></td>
                                    <td>
                                        {published_at
                                            .map(|value| view! { <Timestamp value/> }.into_any())
                                            .unwrap_or_else(|| view! { <span>"-"</span> }.into_any())}
                                    </td>
                                    <td class="data-table__cell--center">{version.step_count.to_string()}</td>
                                    <td class="data-table__cell--center">
                                        <a class="data-table__action" href=edit_href aria-label=edit_title.clone() title=edit_title>
                                            <Pencil class="icon-button__icon"/>
                                        </a>
                                    </td>
                                </tr>
                            }
                        })
                        .collect_view()
                        .into_any()
                }}
            </tbody>
        </DataTable>
    }
}

#[component]
fn WorkflowDetailAssignmentsTable(assignments: Vec<WorkflowAssignmentSummary>) -> impl IntoView {
    let assignments_signal = RwSignal::new(assignments);
    let selected_detail = RwSignal::new(None::<WorkflowAssignmentSummary>);
    let assignments_loading = RwSignal::new(false);
    let assignments_error = RwSignal::new(None::<String>);
    let message = RwSignal::new(None::<String>);
    let close_detail = move |_| selected_detail.set(None);

    view! {
        <DataTable>
            <thead>
                <tr>
                    <th scope="col">"Assignee"</th>
                    <th class="data-table__cell--center" scope="col">"Work State"</th>
                    <th class="data-table__cell--center" scope="col">"Status"</th>
                    <th scope="col">"Assigned"</th>
                    <th class="data-table__cell--center" scope="col">"Actions"</th>
                </tr>
            </thead>
            <tbody>
                {move || {
                    let assignments = assignments_signal.get();
                    if assignments.is_empty() {
                        view! {
                            <tr>
                                <td class="data-table__empty" colspan="5">"No Assignments to Display"</td>
                            </tr>
                        }
                        .into_any()
                    } else {
                        assignments
                            .into_iter()
                            .map(|assignment| {
                                let state_key = workflow_assignment_state(&assignment);
                                let state_label = workflow_assignment_state_label(&assignment);
                                let status_key = workflow_assignment_status_key(&assignment);
                                let status_label = workflow_assignment_status_label(&assignment);
                                let action_label = if assignment.is_active { "Deactivate" } else { "Activate" };
                                let assignment_for_detail = assignment.clone();
                                let assignment_for_toggle = assignment.clone();
                                view! {
                                    <tr>
                                        <th scope="row">
                                            <span>{assignment.account_display_name.clone()}</span>
                                            <small class="workflow-assignment-step-meta">{assignment.account_email}</small>
                                        </th>
                                        <td class="data-table__cell--center">
                                            <span class=status_badge_class(state_key)>{state_label}</span>
                                        </td>
                                        <td class="data-table__cell--center">
                                            <span class=status_badge_class(status_key)>{status_label}</span>
                                        </td>
                                        <td><Timestamp value=assignment.created_at/></td>
                                        <td class="data-table__cell--center">
                                            <DropdownMenu label=format!("Open actions for {}", assignment.account_display_name)>
                                                <button
                                                    class="dropdown-menu__item"
                                                    type="button"
                                                    role="menuitem"
                                                    on:click=move |_| selected_detail.set(Some(assignment_for_detail.clone()))
                                                >
                                                    <PanelRight class="dropdown-menu__item-icon"/>
                                                    <span>"View Details"</span>
                                                </button>
                                                <button
                                                    class="dropdown-menu__item"
                                                    type="button"
                                                    role="menuitem"
                                                    on:click=move |_| {
                                                        toggle_workflow_assignment(
                                                            assignment_for_toggle.clone(),
                                                            assignments_signal,
                                                            assignments_loading,
                                                            assignments_error,
                                                            message,
                                                        );
                                                    }
                                                >
                                                    <X class="dropdown-menu__item-icon"/>
                                                    <span>{action_label}</span>
                                                </button>
                                            </DropdownMenu>
                                        </td>
                                    </tr>
                                }
                            })
                            .collect_view()
                            .into_any()
                    }
                }}
            </tbody>
        </DataTable>
        {move || selected_detail.get().map(|assignment| {
            let workflow_href = format!("/workflows/{}", assignment.workflow_id);
            let node_href = format!("/organization/{}", assignment.node_id);
            let state_key = workflow_assignment_state(&assignment);
            let state_label = workflow_assignment_state_label(&assignment);
            let status_key = workflow_assignment_status_key(&assignment);
            let status_label = workflow_assignment_status_label(&assignment);

            view! {
                <Portal>
                    <section class="sheet-overlay workflow-assignment-detail-overlay" aria-label="Workflow assignment detail">
                        <button class="sheet-overlay__scrim" type="button" aria-label="Close assignment details" on:click=close_detail></button>
                        <aside class="sheet-panel blurred-surface workflow-assignment-detail-sheet" role="dialog" aria-modal="true" aria-label="Workflow assignment details">
                            <div class="sheet-panel__actions">
                                <button class="icon-button sheet-panel__close" type="button" aria-label="Close assignment details" title="Close assignment details" on:click=close_detail>
                                    <X class="icon-button__icon"/>
                                </button>
                            </div>
                            <header class="sheet-panel__header">
                                <p>"Assignment Detail"</p>
                                <h2>{assignment.workflow_name.clone()}</h2>
                            </header>
                            <section class="sheet-panel__section">
                                <h3>"Workflow"</h3>
                                <table class="info-list-table">
                                    <tbody>
                                        <tr>
                                            <th scope="row">"Workflow"</th>
                                            <td><a class="data-table__primary-link" href=workflow_href.clone()>{assignment.workflow_name.clone()}</a></td>
                                        </tr>
                                        <tr>
                                            <th scope="row">"Revision"</th>
                                            <td>{workflow_assignment_revision_label(assignment.workflow_version_label.as_deref())}</td>
                                        </tr>
                                        <tr>
                                            <th scope="row">"Step"</th>
                                            <td>{assignment.workflow_step_title.clone()}</td>
                                        </tr>
                                        <tr>
                                            <th scope="row">"Form"</th>
                                            <td>{assignment.form_name.clone()}</td>
                                        </tr>
                                        <tr>
                                            <th scope="row">"Form Version"</th>
                                            <td>{nonempty_text(assignment.form_version_label.as_deref(), "-")}</td>
                                        </tr>
                                    </tbody>
                                </table>
                            </section>
                            <section class="sheet-panel__section">
                                <h3>"Assignment"</h3>
                                <table class="info-list-table">
                                    <tbody>
                                        <tr>
                                            <th scope="row">"Node"</th>
                                            <td><a class="data-table__primary-link" href=node_href.clone()>{assignment.node_name.clone()}</a></td>
                                        </tr>
                                        <tr>
                                            <th scope="row">"Assignee"</th>
                                            <td>{assignment.account_display_name.clone()}</td>
                                        </tr>
                                        <tr>
                                            <th scope="row">"Email"</th>
                                            <td>{assignment.account_email.clone()}</td>
                                        </tr>
                                        <tr>
                                            <th scope="row">"Work State"</th>
                                            <td><span class=status_badge_class(state_key)>{state_label}</span></td>
                                        </tr>
                                        <tr>
                                            <th scope="row">"Status"</th>
                                            <td><span class=status_badge_class(status_key)>{status_label}</span></td>
                                        </tr>
                                        <tr>
                                            <th scope="row">"Assigned"</th>
                                            <td><Timestamp value=assignment.created_at.clone()/></td>
                                        </tr>
                                    </tbody>
                                </table>
                            </section>
                        </aside>
                    </section>
                </Portal>
            }
        })}
    }
}

#[component]
pub fn WorkflowsEditPage() -> impl IntoView {
    let params = require_route_params::<WorkflowRouteParams>();
    let workflow_id = params.workflow_id;
    let detail = RwSignal::new(None::<WorkflowDefinition>);
    let node_types = RwSignal::new(Vec::<NodeTypeCatalogEntry>::new());
    let organization_nodes = RwSignal::new(Vec::<OrganizationNode>::new());
    let forms = RwSignal::new(Vec::<FormSummary>::new());
    let existing_workflows = RwSignal::new(Vec::<WorkflowSummary>::new());
    let name = RwSignal::new(String::new());
    let slug = RwSignal::new(String::new());
    let available_node_ids = RwSignal::new(HashSet::<String>::new());
    let description = RwSignal::new(String::new());
    let steps = RwSignal::new(Vec::<WorkflowStepDraft>::new());
    let original_steps = RwSignal::new(Vec::<WorkflowStepDraft>::new());
    let next_step_id = RwSignal::new(1_usize);
    let edit_version_id = RwSignal::new(None::<String>);
    let edit_version_label = RwSignal::new(String::new());
    let edit_version_status = RwSignal::new(String::new());
    let version_is_draft = RwSignal::new(false);
    let initialized = RwSignal::new(false);
    let detail_loading = RwSignal::new(true);
    let options_loading = RwSignal::new(true);
    let detail_error = RwSignal::new(None::<String>);
    let message = RwSignal::new(None::<String>);
    let is_saving = RwSignal::new(false);
    let save_intent = RwSignal::new(None::<WorkflowSaveIntent>);

    {
        let workflow_id = workflow_id.clone();
        Effect::new(move |_| {
            load_workflow_detail(workflow_id.clone(), detail, detail_loading, detail_error);
        });
    }

    Effect::new(move |_| {
        load_workflow_create_options(
            node_types,
            organization_nodes,
            forms,
            existing_workflows,
            options_loading,
            message,
        );
    });

    Effect::new(move |_| {
        if initialized.get_untracked() {
            return;
        }
        let Some(workflow) = detail.get() else {
            return;
        };

        name.set(workflow.name.clone());
        slug.set(workflow.slug.clone());
        available_node_ids.set(
            workflow
                .available_nodes
                .iter()
                .map(|node| node.id.clone())
                .collect(),
        );
        description.set(workflow.description.clone());

        let requested_version_id = {
            #[cfg(feature = "hydrate")]
            {
                current_search_param("version_id")
            }
            #[cfg(not(feature = "hydrate"))]
            {
                None::<String>
            }
        };
        let edit_version = requested_version_id
            .as_ref()
            .and_then(|version_id| {
                workflow
                    .versions
                    .iter()
                    .find(|version| version.id == *version_id)
                    .cloned()
            })
            .or_else(|| active_workflow_definition_version(&workflow).cloned());

        edit_version_id.set(edit_version.as_ref().map(|version| version.id.clone()));
        edit_version_label.set(
            edit_version
                .as_ref()
                .and_then(|version| version.workflow_revision_label.clone())
                .as_deref()
                .map(workflow_revision_label_from_raw)
                .unwrap_or_else(|| "-".to_string()),
        );
        edit_version_status.set(
            edit_version
                .as_ref()
                .map(|version| sentence_label(&version.status))
                .unwrap_or_else(|| "No revisions".to_string()),
        );
        version_is_draft.set(
            edit_version
                .as_ref()
                .map(|version| version.status.eq_ignore_ascii_case("draft"))
                .unwrap_or(false),
        );

        let mut step_summaries = edit_version
            .as_ref()
            .map(|version| version.steps.clone())
            .unwrap_or_default();
        step_summaries.sort_by_key(|step| step.position);
        let draft_steps = step_summaries
            .into_iter()
            .enumerate()
            .map(|(index, step)| WorkflowStepDraft {
                id: index + 1,
                title: step.title,
                form_version_id: step.form_version_id,
            })
            .collect::<Vec<_>>();
        let next_id = draft_steps.len() + 1;
        original_steps.set(draft_steps.clone());
        steps.set(draft_steps);
        next_step_id.set(next_id);
        initialized.set(true);
    });

    Effect::new(move |_| {
        if !initialized.get() {
            return;
        }
        if options_loading.get() {
            return;
        }
        let available_options = workflow_form_version_options(&forms.get(), &node_types.get(), "");
        steps.update(|steps| {
            steps.retain(|step| {
                step.form_version_id.is_empty()
                    || available_options
                        .iter()
                        .any(|(id, _, _)| id == &step.form_version_id)
            });
        });
    });

    let add_step = move || {
        let id = next_step_id.get_untracked();
        next_step_id.set(id + 1);
        steps.update(|steps| {
            steps.push(WorkflowStepDraft {
                id,
                title: format!("Step {}", steps.len() + 1),
                form_version_id: String::new(),
            });
        });
    };

    let can_submit = move || {
        if is_saving.get() || name.get().trim().is_empty() {
            return false;
        }
        if available_node_ids.get().is_empty() {
            return false;
        }
        let current_steps = steps.get();
        !current_steps.is_empty()
            && current_steps
                .iter()
                .all(|step| !step.form_version_id.trim().is_empty())
    };
    let has_step_changes = move || {
        workflow_step_signature(&steps.get()) != workflow_step_signature(&original_steps.get())
    };

    view! {
        <AppShell active_route="workflows" title="Workflows">
            <div class="app-page">
                <Breadcrumb>
                    <BreadcrumbItem>
                        <BreadcrumbLink href="/workflows">"Workflows"</BreadcrumbLink>
                    </BreadcrumbItem>
                    {move || detail.get().map(|workflow| view! {
                        <>
                            <BreadcrumbSeparator/>
                            <BreadcrumbItem>
                                <BreadcrumbLink href=format!("/workflows/{}", workflow.id)>{workflow.name}</BreadcrumbLink>
                            </BreadcrumbItem>
                        </>
                    })}
                    <BreadcrumbSeparator/>
                    <BreadcrumbItem>
                        <BreadcrumbPage>"Edit Workflow"</BreadcrumbPage>
                    </BreadcrumbItem>
                </Breadcrumb>

                <section class="route-panel workflows-page workflow-edit-page">
                    <PageHeader title="Edit Workflow"/>

                    {move || {
                        if detail_loading.get() || options_loading.get() {
                            view! {
                                <section class="organization-state" aria-live="polite">
                                    <h3>"Loading workflow"</h3>
                                    <p>"Fetching workflow details and form versions."</p>
                                </section>
                            }
                            .into_any()
                        } else if let Some(error) = detail_error.get() {
                            view! {
                                <section class="organization-state is-error" role="alert">
                                    <h3>"Workflow unavailable"</h3>
                                    <p>{error}</p>
                                </section>
                            }
                            .into_any()
                        } else {
                            let workflow_id_for_href = workflow_id.clone();
                            let workflow_id_for_submit = workflow_id.clone();
                            let workflow_id_for_publish = workflow_id.clone();
                            let workflow_href = format!("/workflows/{}", workflow_id_for_href);
                            view! {
                                <form
                                    class="native-form workflow-create-form"
                                    on:submit=move |event| {
                                        event.prevent_default();
                                        submit_update_workflow(
                                            workflow_id_for_submit.clone(),
                                            edit_version_id.get_untracked(),
                                            version_is_draft.get_untracked(),
                                            name,
                                            slug,
                                            available_node_ids,
                                            steps,
                                            original_steps,
                                            description,
                                            is_saving,
                                            save_intent,
                                            message,
                                            WorkflowSaveIntent::Draft,
                                        );
                                    }
                                >
                                    <div class="form-grid">
                                        <label class="form-field">
                                            <span>"Workflow Name"</span>
                                            <input
                                                type="text"
                                                value=move || name.get()
                                                on:input=move |event| {
                                                    name.set(event_target_value(&event));
                                                }
                                            />
                                        </label>
                                        <label class="form-field">
                                            <span>"Description"</span>
                                            <textarea
                                                prop:value=move || description.get()
                                                on:input=move |event| {
                                                    description.set(event_target_value(&event));
                                                }
                                            ></textarea>
                                        </label>
                                    </div>

                                    <section class="form-section">
                                        <h3>"Available At"</h3>
                                        <WorkflowAvailableNodesPicker
                                            nodes=organization_nodes.get()
                                            selected_node_ids=available_node_ids
                                        />
                                    </section>

                                    <section class="form-section">
                                        <h3>"Active Revision"</h3>
                                        <table class="info-list-table">
                                            <tbody>
                                                <tr>
                                                    <th scope="row">"Revision"</th>
                                                    <td>{move || edit_version_label.get()}</td>
                                                </tr>
                                                <tr>
                                                    <th scope="row">"Status"</th>
                                                    <td>{move || {
                                                        let status = edit_version_status.get();
                                                        let key = status.to_lowercase().replace(' ', "-");
                                                        view! { <span class=status_badge_class(&key)>{status}</span> }
                                                    }}</td>
                                                </tr>
                                            </tbody>
                                        </table>
                                    </section>

                                    <section class="form-section">
                                        <div class="form-builder-section-card__header">
                                            <h3>"Workflow Steps"</h3>
                                            <button
                                                class="button button--secondary"
                                                type="button"
                                                disabled=move || {
                                                    workflow_form_version_options(
                                                        &forms.get(),
                                                        &node_types.get(),
                                                        "",
                                                    )
                                                    .is_empty()
                                                }
                                                on:click=move |_| add_step()
                                            >
                                                "+ Add Step"
                                            </button>
                                        </div>

                                        {move || {
                                            if workflow_form_version_options(
                                                &forms.get(),
                                                &node_types.get(),
                                                "",
                                            ).is_empty() {
                                                return view! {
                                                    <section class="organization-state">
                                                        <h3>"No published forms available"</h3>
                                                        <p>"Publish at least one form version before editing workflow steps."</p>
                                                    </section>
                                                }
                                                .into_any();
                                            }

                                            if !version_is_draft.get() {
                                                view! {
                                                    <p class="form-message" role="status">
                                                        "Step changes will create a new draft workflow revision."
                                                    </p>
                                                }
                                                .into_any()
                                            } else {
                                                view! { <></> }.into_any()
                                            }
                                        }}

                                        {move || {
                                            if steps.get().is_empty() {
                                                return view! {
                                                    <section class="organization-state">
                                                        <h3>"No workflow steps"</h3>
                                                        <p>"This workflow revision does not have steps yet."</p>
                                                    </section>
                                                }
                                                .into_any();
                                            }

                                            view! {
                                                <div class="workflow-step-list">
                                                    <For
                                                        each=move || {
                                                            steps.get().into_iter().enumerate().collect::<Vec<_>>()
                                                        }
                                                        key=|(_, step)| step.id
                                                        children=move |(index, step)| {
                                                            let step_id = step.id;
                                                            let step_position = move || {
                                                                steps
                                                                    .get()
                                                                    .iter()
                                                                    .position(|step| step.id == step_id)
                                                                    .map(|index| index + 1)
                                                                    .unwrap_or(index + 1)
                                                            };
                                                            view! {
                                                                <article class="workflow-step-card">
                                                                    <header class="workflow-step-card__header">
                                                                        <span class="workflow-step-card__position">{move || format!("Step {}", step_position())}</span>
                                                                        <div class="workflow-step-card__actions">
                                                                            <button
                                                                                class="icon-button icon-button--control"
                                                                                type="button"
                                                                                title="Move step up"
                                                                                disabled=move || step_position() <= 1
                                                                                on:click=move |_| {
                                                                                    steps.update(|steps| {
                                                                                        if let Some(index) = steps.iter().position(|step| step.id == step_id) {
                                                                                            if index > 0 {
                                                                                                steps.swap(index, index - 1);
                                                                                            }
                                                                                        }
                                                                                    });
                                                                                }
                                                                            >
                                                                                <ArrowUp/>
                                                                            </button>
                                                                            <button
                                                                                class="icon-button icon-button--control"
                                                                                type="button"
                                                                                title="Move step down"
                                                                                disabled=move || {
                                                                                    let step_count = steps.get().len();
                                                                                    step_position() >= step_count
                                                                                }
                                                                                on:click=move |_| {
                                                                                    steps.update(|steps| {
                                                                                        if let Some(index) = steps.iter().position(|step| step.id == step_id) {
                                                                                            if index + 1 < steps.len() {
                                                                                                steps.swap(index, index + 1);
                                                                                            }
                                                                                        }
                                                                                    });
                                                                                }
                                                                            >
                                                                                <ArrowDown/>
                                                                            </button>
                                                                            <button
                                                                                class="icon-button icon-button--danger"
                                                                                type="button"
                                                                                title="Remove step"
                                                                                on:click=move |_| {
                                                                                    steps.update(|steps| {
                                                                                        steps.retain(|step| step.id != step_id);
                                                                                    });
                                                                                }
                                                                            >
                                                                                <Trash2/>
                                                                            </button>
                                                                        </div>
                                                                    </header>
                                                                    <div class="form-grid">
                                                                        <label class="form-field">
                                                                            <span>"Step Title"</span>
                                                                            <input
                                                                                type="text"
                                                                                prop:value=move || {
                                                                                    workflow_step_title_by_id(&steps.get(), step_id)
                                                                                }
                                                                                on:input=move |event| {
                                                                                    let value = event_target_value(&event);
                                                                                    steps.update(|steps| {
                                                                                        if let Some(step) = steps.iter_mut().find(|step| step.id == step_id) {
                                                                                            step.title = value;
                                                                                        }
                                                                                    });
                                                                                }
                                                                            />
                                                                        </label>
                                                                        <label class="form-field">
                                                                            <span>"Form Version"</span>
                                                                            <select
                                                                                prop:value=move || {
                                                                                    workflow_step_form_version_id_by_id(&steps.get(), step_id)
                                                                                }
                                                                                on:change=move |event| {
                                                                                    let value = event_target_value(&event);
                                                                                    steps.update(|steps| {
                                                                                        if let Some(step) = steps.iter_mut().find(|step| step.id == step_id) {
                                                                                            step.form_version_id = value;
                                                                                        }
                                                                                    });
                                                                                }
                                                                            >
                                                                                <option value="">"Select form version"</option>
                                                                                {workflow_form_version_options(
                                                                                    &forms.get(),
                                                                                    &node_types.get(),
                                                                                    "",
                                                                                )
                                                                                    .into_iter()
                                                                                    .map(|(id, label, _)| {
                                                                                        view! {
                                                                                            <option value=id>{label}</option>
                                                                                        }
                                                                                    })
                                                                                    .collect_view()}
                                                                            </select>
                                                                        </label>
                                                                    </div>
                                                                    <div class="workflow-step-card__footer">
                                                                        <span>{move || {
                                                                            let selected_form_version_id = steps
                                                                                .get()
                                                                                .into_iter()
                                                                                .find(|step| step.id == step_id)
                                                                                .map(|step| step.form_version_id)
                                                                                .unwrap_or_default();
                                                                            workflow_step_form_label(&forms.get(), &selected_form_version_id)
                                                                        }}</span>
                                                                    </div>
                                                                </article>
                                                            }
                                                        }
                                                    />
                                                </div>
                                            }
                                            .into_any()
                                        }}
                                    </section>

                                    {move || message.get().map(|message| view! {
                                        <p class="form-message" role="status">{message}</p>
                                    })}

                                    <div class="form-actions">
                                        <a class="button" href=workflow_href>"Cancel"</a>
                                        <button class="button button--secondary" type="submit" disabled=move || !can_submit()>
                                            {move || {
                                                if save_intent.get() == Some(WorkflowSaveIntent::Draft) {
                                                    "Saving..."
                                                } else if has_step_changes() {
                                                    "Save as Draft"
                                                } else {
                                                    "Save Changes"
                                                }
                                            }}
                                        </button>
                                        <button
                                            class="button button--secondary"
                                            type="button"
                                            disabled=move || {
                                                !can_submit()
                                                    || (!version_is_draft.get() && !has_step_changes())
                                            }
                                            on:click=move |_| {
                                                submit_update_workflow(
                                                    workflow_id_for_publish.clone(),
                                                    edit_version_id.get_untracked(),
                                                    version_is_draft.get_untracked(),
                                                    name,
                                                    slug,
                                                    available_node_ids,
                                                    steps,
                                                    original_steps,
                                                    description,
                                                    is_saving,
                                                    save_intent,
                                                    message,
                                                    WorkflowSaveIntent::Publish,
                                                );
                                            }
                                        >
                                            {move || {
                                                if save_intent.get() == Some(WorkflowSaveIntent::Publish) {
                                                    "Publishing..."
                                                } else {
                                                    "Save and Publish"
                                                }
                                            }}
                                        </button>
                                    </div>
                                </form>
                            }
                            .into_any()
                        }
                    }}
                </section>
            </div>
        </AppShell>
    }
}


