use super::*;

#[component]
pub fn AdministrationPage() -> impl IntoView {
    view! {
        <AppShell active_route="administration" title="Administration">
            <section class="route-panel administration-page">
                <PageHeader
                    title="Administration"
                    description="Internal configuration routes are registered natively while their detailed management screens are restored."
                />

                <div class="organization-detail-content administration-landing">
                    <div class="organization-detail-content__grid">
                        <section class="organization-detail-card">
                            <h3>"User Management"</h3>
                            <p>"Manage local users, passwords, role memberships, and active status."</p>
                            <div class="form-actions">
                                <a class="button" href="/administration/users">"Open Users"</a>
                            </div>
                        </section>

                        <section class="organization-detail-card">
                            <h3>"Roles And Access"</h3>
                            <p>"Review reusable capability bundles and the access assignments that control application visibility."</p>
                            <div class="form-actions">
                                <a class="button" href="/administration/roles">"Open Roles"</a>
                            </div>
                        </section>

                        <section class="organization-detail-card">
                            <h3>"Organization Schema"</h3>
                            <p>"Manage node type labels and hierarchy rules for the organization model."</p>
                            <div class="form-actions">
                                <a class="button" href="/administration/node-types">"Open Node Types"</a>
                            </div>
                        </section>

                        <section class="organization-detail-card">
                            <h3>"Datasets"</h3>
                            <p>"Review imported dataset catalogs and supporting reference data."</p>
                            <div class="form-actions">
                                <a class="button" href="/datasets">"Open Datasets"</a>
                            </div>
                        </section>

                        <section class="organization-detail-card">
                            <h3>"Migration"</h3>
                            <p>"Validate, dry-run, and import representative legacy fixtures."</p>
                            <div class="form-actions">
                                <a class="button button--secondary" href="/migration">"Open Migration"</a>
                            </div>
                        </section>
                    </div>
                </div>
            </section>
        </AppShell>
    }
}

#[component]
pub fn AdministrationUsersPage() -> impl IntoView {
    let users = RwSignal::new(Vec::<AdminUserSummary>::new());
    let is_loading = RwSignal::new(true);
    let load_error = RwSignal::new(None::<String>);
    let search = RwSignal::new(String::new());
    let status_filter = RwSignal::new("all".to_string());
    let role_filter = RwSignal::new("all".to_string());

    Effect::new(move |_| {
        load_admin_users(users, is_loading, load_error);
    });

    let filtered_users = move || {
        let query = search.get();
        let status = status_filter.get();
        let role = role_filter.get();
        users
            .get()
            .into_iter()
            .filter(|user| {
                let status_key = admin_user_status_key(user);
                let role_names = admin_user_role_names(user);
                let matches_status = status == "all" || status == status_key;
                let matches_role =
                    role == "all" || user.roles.iter().any(|user_role| user_role.name == role);
                matches_status
                    && matches_role
                    && text_matches(
                        &query,
                        &[
                            user.display_name.as_str(),
                            user.email.as_str(),
                            status_key,
                            role_names.as_str(),
                        ],
                    )
            })
            .collect::<Vec<_>>()
    };
    let role_options = move || {
        unique_filter_options(users.get().iter().flat_map(|user| {
            user.roles
                .iter()
                .map(|role| role.name.clone())
                .collect::<Vec<_>>()
        }))
    };

    view! {
        <AppShell active_route="administration" title="Users">
            <section class="route-panel administration-users-page">
                <Breadcrumb>
                    <BreadcrumbItem>
                        <BreadcrumbLink href="/administration">"Administration"</BreadcrumbLink>
                    </BreadcrumbItem>
                    <BreadcrumbSeparator/>
                    <BreadcrumbItem>
                        <BreadcrumbPage>"Users"</BreadcrumbPage>
                    </BreadcrumbItem>
                </Breadcrumb>

                <PageHeader
                    title="Users"
                    description="Manage local Tessara users, active status, and assigned roles."
                />

                {move || {
                    if is_loading.get() {
                        view! {
                            <section class="organization-state" aria-live="polite">
                                <h3>"Loading users"</h3>
                                <p>"Fetching administrative user records."</p>
                            </section>
                        }
                        .into_any()
                    } else if let Some(message) = load_error.get() {
                        view! {
                            <section class="organization-state is-error" role="alert">
                                <h3>"Users unavailable"</h3>
                                <p>{message}</p>
                            </section>
                        }
                        .into_any()
                    } else {
                        view! {
                            <AdministrationUsersList
                                users=filtered_users()
                                search
                                status_filter
                                role_filter
                                role_options=role_options()
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
fn AdministrationUsersList(
    users: Vec<AdminUserSummary>,
    search: RwSignal<String>,
    status_filter: RwSignal<String>,
    role_filter: RwSignal<String>,
    role_options: Vec<String>,
) -> impl IntoView {
    let table_users = users.clone();
    let card_users = users;

    view! {
        <div class="forms-list forms-list-responsive-table administration-users-list">
            <div class="searchable-data-table">
                <div class="searchable-data-table__toolbar forms-list__toolbar">
                    <label class="searchable-data-table__search searchable-data-table__control">
                        <Search class="searchable-data-table__control-icon"/>
                        <span class="sr-only">"Search users"</span>
                        <input
                            type="search"
                            placeholder="Search users"
                            prop:value=move || search.get()
                            on:input=move |event| search.set(event_target_value(&event))
                        />
                    </label>
                </div>

                <DataTable>
                    <thead>
                        <tr>
                            <th scope="col">"User"</th>
                            <th scope="col">
                                <FilterHeader
                                    label="Role"
                                    all_label="All Roles"
                                    filter=role_filter
                                    options=role_options
                                />
                            </th>
                            <th class="data-table__cell--center" scope="col">
                                <FilterHeader
                                    label="Status"
                                    all_label="All Statuses"
                                    filter=status_filter
                                    options=vec!["active".to_string(), "inactive".to_string()]
                                />
                            </th>
                            <th class="data-table__cell--center" scope="col">"Roles"</th>
                            <th class="data-table__cell--center" scope="col">"Actions"</th>
                        </tr>
                    </thead>
                    <tbody>
                        {if table_users.is_empty() {
                            view! {
                                <tr>
                                    <td class="data-table__empty" colspan="5">"No Users to Display"</td>
                                </tr>
                            }
                            .into_any()
                        } else {
                            table_users
                                .into_iter()
                                .map(|user| {
                                    let status_key = admin_user_status_key(&user);
                                    let status_label = admin_user_status_label(&user);
                                    let role_names = admin_user_role_names(&user);
                                    let detail_href = format!("/administration/users/{}", user.id);
                                    let edit_href = format!("/administration/users/{}/edit", user.id);
                                    let access_href = format!("/administration/users/{}/access", user.id);
                                    let display_name = user.display_name.clone();
                                    let detail_href_for_click = detail_href.clone();
                                    let edit_href_for_click = edit_href.clone();
                                    let access_href_for_click = access_href.clone();
                                    view! {
                                        <tr>
                                            <th scope="row">
                                                <a class="data-table__primary-link" href=detail_href>{user.display_name}</a>
                                                <small class="workflow-assignment-step-meta">{user.email}</small>
                                            </th>
                                            <td>{role_names}</td>
                                            <td class="data-table__cell--center">
                                                <span class=status_badge_class(status_key)>{status_label}</span>
                                            </td>
                                            <td class="data-table__cell--center">{user.roles.len()}</td>
                                            <td class="data-table__cell--center">
                                                <DropdownMenu label=format!("Open actions for {display_name}")>
                                                    <button
                                                        class="dropdown-menu__item"
                                                        type="button"
                                                        role="menuitem"
                                                        on:click=move |_| {
                                                            #[cfg(feature = "hydrate")]
                                                            navigate_to_href(&detail_href_for_click);
                                                            #[cfg(not(feature = "hydrate"))]
                                                            let _ = &detail_href_for_click;
                                                        }
                                                    >
                                                        <PanelRight class="dropdown-menu__item-icon"/>
                                                        <span>"View Details"</span>
                                                    </button>
                                                    <button
                                                        class="dropdown-menu__item"
                                                        type="button"
                                                        role="menuitem"
                                                        on:click=move |_| {
                                                            #[cfg(feature = "hydrate")]
                                                            navigate_to_href(&edit_href_for_click);
                                                            #[cfg(not(feature = "hydrate"))]
                                                            let _ = &edit_href_for_click;
                                                        }
                                                    >
                                                        <Pencil class="dropdown-menu__item-icon"/>
                                                        <span>"Edit Account"</span>
                                                    </button>
                                                    <button
                                                        class="dropdown-menu__item"
                                                        type="button"
                                                        role="menuitem"
                                                        on:click=move |_| {
                                                            #[cfg(feature = "hydrate")]
                                                            navigate_to_href(&access_href_for_click);
                                                            #[cfg(not(feature = "hydrate"))]
                                                            let _ = &access_href_for_click;
                                                        }
                                                    >
                                                        <LockKeyhole class="dropdown-menu__item-icon"/>
                                                        <span>"Manage Permissions"</span>
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
            </div>

            <div class="forms-list-mobile-cards administration-users-mobile-cards">
                {if card_users.is_empty() {
                    view! { <p class="forms-list-mobile-empty">"No Users to Display"</p> }.into_any()
                } else {
                    card_users
                        .into_iter()
                        .map(|user| {
                            let status_key = admin_user_status_key(&user);
                            let status_label = admin_user_status_label(&user);
                            let role_names = admin_user_role_names(&user);
                            let detail_href = format!("/administration/users/{}", user.id);
                            let edit_href = format!("/administration/users/{}/edit", user.id);
                            let access_href = format!("/administration/users/{}/access", user.id);
                            view! {
                                <article class="forms-list-mobile-card administration-user-mobile-card">
                                    <div class="forms-list-mobile-card__header">
                                        <div>
                                            <h3><a href=detail_href.clone()>{user.display_name}</a></h3>
                                            <span>{user.email}</span>
                                        </div>
                                        <span class=status_badge_class(status_key)>{status_label}</span>
                                    </div>
                                    <dl>
                                        <div>
                                            <dt>"Roles"</dt>
                                            <dd>{role_names}</dd>
                                        </div>
                                        <div>
                                            <dt>"Role Count"</dt>
                                            <dd>{user.roles.len()}</dd>
                                        </div>
                                    </dl>
                                    <div class="workflow-assignment-mobile-card__actions">
                                        <a class="button button--compact" href=detail_href>"View Details"</a>
                                        <a class="button button--compact button--secondary" href=edit_href>"Edit Account"</a>
                                        <a class="button button--compact button--secondary" href=access_href>"Manage Permissions"</a>
                                    </div>
                                </article>
                            }
                        })
                        .collect_view()
                        .into_any()
                }}
            </div>
        </div>
    }
}

fn admin_user_status_key(user: &AdminUserSummary) -> &'static str {
    if user.is_active { "active" } else { "inactive" }
}

fn admin_user_status_label(user: &AdminUserSummary) -> &'static str {
    if user.is_active { "Active" } else { "Inactive" }
}

fn admin_user_role_names(user: &AdminUserSummary) -> String {
    if user.roles.is_empty() {
        "No roles".to_string()
    } else {
        user.roles
            .iter()
            .map(|role| role.name.as_str())
            .collect::<Vec<_>>()
            .join(", ")
    }
}

fn toggle_string_selection(selection: RwSignal<Vec<String>>, value: String, selected: bool) {
    selection.update(|values| {
        if selected {
            if !values.iter().any(|existing| existing == &value) {
                values.push(value);
            }
        } else {
            values.retain(|existing| existing != &value);
        }
    });
}

#[component]
pub fn AdministrationUserDetailPage() -> impl IntoView {
    let params = require_route_params::<AccountRouteParams>();
    let account_id = params.account_id;
    let detail = RwSignal::new(None::<AdminUserDetail>);
    let is_loading = RwSignal::new(true);
    let load_error = RwSignal::new(None::<String>);

    Effect::new({
        let account_id = account_id.clone();
        move |_| {
            load_admin_user_detail(account_id.clone(), detail, is_loading, load_error);
        }
    });

    view! {
        <AppShell active_route="administration" title="User Detail">
            <section class="route-panel administration-user-detail-page">
                <Breadcrumb>
                    <BreadcrumbItem>
                        <BreadcrumbLink href="/administration">"Administration"</BreadcrumbLink>
                    </BreadcrumbItem>
                    <BreadcrumbSeparator/>
                    <BreadcrumbItem>
                        <BreadcrumbLink href="/administration/users">"Users"</BreadcrumbLink>
                    </BreadcrumbItem>
                    <BreadcrumbSeparator/>
                    <BreadcrumbItem>
                        <BreadcrumbPage>"User Detail"</BreadcrumbPage>
                    </BreadcrumbItem>
                </Breadcrumb>

                {move || {
                    if is_loading.get() {
                        view! {
                            <section class="organization-state" aria-live="polite">
                                <h3>"Loading user"</h3>
                                <p>"Fetching user profile and effective access."</p>
                            </section>
                        }
                        .into_any()
                    } else if let Some(message) = load_error.get() {
                        view! {
                            <section class="organization-state is-error" role="alert">
                                <h3>"User unavailable"</h3>
                                <p>{message}</p>
                            </section>
                        }
                        .into_any()
                    } else if let Some(user) = detail.get() {
                        let access_href = format!("/administration/users/{}/access", user.id);
                        let edit_href = format!("/administration/users/{}/edit", user.id);
                        let status_key = admin_user_status_key_from_bool(user.is_active);
                        let status_label = admin_user_status_label_from_bool(user.is_active);
                        let access_profile_label = admin_access_profile_label(&user.ui_access_profile);
                        let role_names = admin_role_names(&user.roles);
                        let capability_count = user.capabilities.len().to_string();
                        view! {
                            <header class="page-header">
                                <div>
                                    <h2>{user.display_name.clone()}</h2>
                                    <p>{user.email.clone()}</p>
                                </div>
                                <div class="page-header__actions">
                                    <span class=status_badge_class(status_key)>{status_label}</span>
                                </div>
                            </header>

                            <div class="organization-detail-content">
                                <section class="organization-detail-card organization-detail-card--wide">
                                    <h3>"Account"</h3>
                                    <InfoListTable>
                                        <tr>
                                            <th scope="row">"Email"</th>
                                            <td>{user.email.clone()}</td>
                                        </tr>
                                        <tr>
                                            <th scope="row">"Access Profile"</th>
                                            <td>{access_profile_label}</td>
                                        </tr>
                                        <tr>
                                            <th scope="row">"Roles"</th>
                                            <td>{role_names}</td>
                                        </tr>
                                        <tr>
                                            <th scope="row">"Capabilities"</th>
                                            <td>{capability_count}</td>
                                        </tr>
                                    </InfoListTable>
                                    <div class="form-actions">
                                        <a class="button" href=edit_href>"Edit Account"</a>
                                        <a class="button" href=access_href>"Manage Permissions"</a>
                                        <a class="button button--secondary" href="/administration/users">"Back to Users"</a>
                                    </div>
                                </section>

                                <section class="organization-detail-card">
                                    <h3>"Scope Nodes"</h3>
                                    <AdminScopeNodeList nodes=user.scope_nodes/>
                                </section>

                                <section class="organization-detail-card">
                                    <h3>"Delegations"</h3>
                                    <AdminDelegationList delegations=user.delegations empty_label="No delegations assigned."/>
                                </section>
                            </div>
                        }
                        .into_any()
                    } else {
                        view! {
                            <section class="organization-state">
                                <h3>"User not found"</h3>
                                <p>"No user record was returned for this account."</p>
                            </section>
                        }
                        .into_any()
                    }
                }}
            </section>
        </AppShell>
    }
}

#[component]
pub fn AdministrationUserEditPage() -> impl IntoView {
    let params = require_route_params::<AccountRouteParams>();
    let account_id = params.account_id;
    let detail = RwSignal::new(None::<AdminUserDetail>);
    let roles = RwSignal::new(Vec::<AdminRoleSummary>::new());
    let email = RwSignal::new(String::new());
    let display_name = RwSignal::new(String::new());
    let password = RwSignal::new(String::new());
    let is_active = RwSignal::new(true);
    let selected_role_ids = RwSignal::new(Vec::<String>::new());
    let is_loading = RwSignal::new(true);
    let is_saving = RwSignal::new(false);
    let load_error = RwSignal::new(None::<String>);
    let message = RwSignal::new(None::<String>);

    Effect::new({
        let account_id = account_id.clone();
        move |_| {
            load_admin_user_edit_context(
                account_id.clone(),
                detail,
                roles,
                email,
                display_name,
                is_active,
                selected_role_ids,
                is_loading,
                load_error,
            );
        }
    });

    view! {
        <AppShell active_route="administration" title="Edit User">
            <section class="route-panel administration-user-edit-page">
                <Breadcrumb>
                    <BreadcrumbItem>
                        <BreadcrumbLink href="/administration">"Administration"</BreadcrumbLink>
                    </BreadcrumbItem>
                    <BreadcrumbSeparator/>
                    <BreadcrumbItem>
                        <BreadcrumbLink href="/administration/users">"Users"</BreadcrumbLink>
                    </BreadcrumbItem>
                    <BreadcrumbSeparator/>
                    <BreadcrumbItem>
                        <BreadcrumbPage>"Edit User"</BreadcrumbPage>
                    </BreadcrumbItem>
                </Breadcrumb>

                {move || {
                    if is_loading.get() {
                        view! {
                            <section class="organization-state" aria-live="polite">
                                <h3>"Loading user"</h3>
                                <p>"Fetching account and role options."</p>
                            </section>
                        }
                        .into_any()
                    } else if let Some(error) = load_error.get() {
                        view! {
                            <section class="organization-state is-error" role="alert">
                                <h3>"User unavailable"</h3>
                                <p>{error}</p>
                            </section>
                        }
                        .into_any()
                    } else {
                        let cancel_href = format!("/administration/users/{account_id}");
                        view! {
                            <PageHeader
                                title="Edit User"
                                description="Update the account profile, active status, password, and assigned roles."
                            />
                            <form
                                class="native-form administration-user-form"
                                on:submit={
                                    let account_id = account_id.clone();
                                    move |event| {
                                        event.prevent_default();
                                        submit_update_admin_user(
                                            account_id.clone(),
                                            email,
                                            display_name,
                                            password,
                                            is_active,
                                            selected_role_ids,
                                            is_saving,
                                            message,
                                        );
                                    }
                                }
                            >
                                <div class="form-grid">
                                    <label class="form-field" for="admin-user-display-name">
                                        <span>"Display Name"</span>
                                        <input
                                            id="admin-user-display-name"
                                            type="text"
                                            autocomplete="name"
                                            prop:value=move || display_name.get()
                                            on:input=move |event| display_name.set(event_target_value(&event))
                                            required
                                        />
                                    </label>
                                    <label class="form-field" for="admin-user-email">
                                        <span>"Email"</span>
                                        <input
                                            id="admin-user-email"
                                            type="email"
                                            autocomplete="email"
                                            prop:value=move || email.get()
                                            on:input=move |event| email.set(event_target_value(&event))
                                            required
                                        />
                                    </label>
                                    <label class="form-field" for="admin-user-password">
                                        <span>"New Password"</span>
                                        <input
                                            id="admin-user-password"
                                            type="password"
                                            autocomplete="new-password"
                                            placeholder="Leave blank to keep current password"
                                            prop:value=move || password.get()
                                            on:input=move |event| password.set(event_target_value(&event))
                                        />
                                    </label>
                                    <label class="form-field">
                                        <span>"Active"</span>
                                        <label class="toggle-row toggle-row--compact">
                                            <input
                                                type="checkbox"
                                                prop:checked=move || is_active.get()
                                                on:change=move |event| is_active.set(event_target_checked(&event))
                                            />
                                            <span>{move || if is_active.get() { "Active" } else { "Inactive" }}</span>
                                        </label>
                                    </label>
                                </div>

                                <section class="form-section">
                                    <h3>"Roles"</h3>
                                    <div class="checkbox-list">
                                        {move || {
                                            let selected = selected_role_ids.get();
                                            roles
                                                .get()
                                                .into_iter()
                                                .map(|role| {
                                                    let role_id = role.id.clone();
                                                    let checked = selected.iter().any(|id| id == &role.id);
                                                    view! {
                                                        <label class="checkbox-list__item">
                                                            <input
                                                                type="checkbox"
                                                                prop:checked=checked
                                                                on:change=move |event| {
                                                                    toggle_string_selection(
                                                                        selected_role_ids,
                                                                        role_id.clone(),
                                                                        event_target_checked(&event),
                                                                    );
                                                                }
                                                            />
                                                            <span>
                                                                <strong>{role.name}</strong>
                                                                <small>{format!("{} capabilities, {} users", role.capability_count, role.account_count)}</small>
                                                            </span>
                                                        </label>
                                                    }
                                                })
                                                .collect_view()
                                        }}
                                    </div>
                                </section>

                                {move || message
                                    .get()
                                    .map(|text| view! { <p class="form-message" role="status">{text}</p> })}

                                <div class="form-actions">
                                    <a class="button button--secondary" href=cancel_href.clone()>"Cancel"</a>
                                    <button class="button" type="submit" disabled=move || is_saving.get()>
                                        {move || if is_saving.get() { "Saving..." } else { "Save User" }}
                                    </button>
                                </div>
                            </form>
                        }
                        .into_any()
                    }
                }}
            </section>
        </AppShell>
    }
}

#[component]
pub fn AdministrationUserAccessPage() -> impl IntoView {
    let params = require_route_params::<AccountRouteParams>();
    let account_id = params.account_id;
    let detail = RwSignal::new(None::<AdminUserAccessDetail>);
    let selected_scope_node_ids = RwSignal::new(Vec::<String>::new());
    let selected_delegate_account_ids = RwSignal::new(Vec::<String>::new());
    let is_loading = RwSignal::new(true);
    let is_saving = RwSignal::new(false);
    let load_error = RwSignal::new(None::<String>);
    let message = RwSignal::new(None::<String>);

    Effect::new({
        let account_id = account_id.clone();
        move |_| {
            load_admin_user_access(
                account_id.clone(),
                detail,
                selected_scope_node_ids,
                selected_delegate_account_ids,
                is_loading,
                load_error,
            );
        }
    });

    view! {
        <AppShell active_route="administration" title="User Permissions">
            <section class="route-panel administration-user-access-page">
                <Breadcrumb>
                    <BreadcrumbItem>
                        <BreadcrumbLink href="/administration">"Administration"</BreadcrumbLink>
                    </BreadcrumbItem>
                    <BreadcrumbSeparator/>
                    <BreadcrumbItem>
                        <BreadcrumbLink href="/administration/users">"Users"</BreadcrumbLink>
                    </BreadcrumbItem>
                    <BreadcrumbSeparator/>
                    <BreadcrumbItem>
                        <BreadcrumbPage>"Permissions"</BreadcrumbPage>
                    </BreadcrumbItem>
                </Breadcrumb>

                {move || {
                    if is_loading.get() {
                        view! {
                            <section class="organization-state" aria-live="polite">
                                <h3>"Loading permissions"</h3>
                                <p>"Fetching effective capabilities, scope nodes, and delegations."</p>
                            </section>
                        }
                        .into_any()
                    } else if let Some(message) = load_error.get() {
                        view! {
                            <section class="organization-state is-error" role="alert">
                                <h3>"Permissions unavailable"</h3>
                                <p>{message}</p>
                            </section>
                        }
                        .into_any()
                    } else if let Some(access) = detail.get() {
                        let detail_href = format!("/administration/users/{}", access.account_id);
                        let edit_href = format!("/administration/users/{}/edit", access.account_id);
                        let page_title = format!("{} Permissions", access.display_name);
                        let access_profile_label = admin_access_profile_label(&access.ui_access_profile);
                        let capability_count = access.capabilities.len().to_string();
                        let scope_editing = admin_editable_label(access.scope_assignments_editable);
                        let delegation_editing =
                            admin_editable_label(access.delegation_assignments_editable);
                        view! {
                            <header class="page-header">
                                <div>
                                    <h2>{page_title}</h2>
                                    <p>{access.email.clone()}</p>
                                </div>
                            </header>

                            <form
                                class="native-form administration-user-access-form"
                                on:submit={
                                    let account_id = account_id.clone();
                                    move |event| {
                                        event.prevent_default();
                                        submit_update_admin_user_access(
                                            account_id.clone(),
                                            selected_scope_node_ids,
                                            selected_delegate_account_ids,
                                            is_saving,
                                            message,
                                        );
                                    }
                                }
                            >
                            <div class="organization-detail-content">
                                <section class="organization-detail-card organization-detail-card--wide">
                                    <h3>"Effective Access"</h3>
                                    <InfoListTable>
                                        <tr>
                                            <th scope="row">"Access Profile"</th>
                                            <td>{access_profile_label}</td>
                                        </tr>
                                        <tr>
                                            <th scope="row">"Capabilities"</th>
                                            <td>{capability_count}</td>
                                        </tr>
                                        <tr>
                                            <th scope="row">"Scope Editing"</th>
                                            <td>{scope_editing}</td>
                                        </tr>
                                        <tr>
                                            <th scope="row">"Delegation Editing"</th>
                                            <td>{delegation_editing}</td>
                                        </tr>
                                    </InfoListTable>
                                </section>

                                <section class="organization-detail-card">
                                    <h3>"Scope Nodes"</h3>
                                    {if access.scope_assignments_editable {
                                        view! {
                                            <AdminScopeNodeChecklist
                                                nodes=access.available_scope_nodes
                                                selected_node_ids=selected_scope_node_ids
                                            />
                                        }
                                        .into_any()
                                    } else {
                                        view! { <AdminScopeNodeList nodes=access.scope_nodes/> }.into_any()
                                    }}
                                </section>

                                <section class="organization-detail-card">
                                    <h3>"Delegations"</h3>
                                    {if access.delegation_assignments_editable {
                                        view! {
                                            <AdminDelegationChecklist
                                                delegations=access.available_delegate_accounts
                                                selected_delegate_account_ids=selected_delegate_account_ids
                                            />
                                        }
                                        .into_any()
                                    } else {
                                        view! { <AdminDelegationList delegations=access.delegations empty_label="No delegated accounts."/> }.into_any()
                                    }}
                                </section>

                                <section class="organization-detail-card organization-detail-card--wide">
                                    <h3>"Capabilities"</h3>
                                    <AdminCapabilityList capabilities=access.capabilities/>
                                    <p class="helper-text">"Capabilities are inherited from assigned roles."</p>
                                    <div class="form-actions">
                                        <a class="button button--secondary" href=edit_href>"Edit Account Roles"</a>
                                    </div>
                                </section>
                            </div>
                            {move || message
                                .get()
                                .map(|text| view! { <p class="form-message" role="status">{text}</p> })}
                            <div class="form-actions">
                                <a class="button button--secondary" href=detail_href>"Back to User"</a>
                                <button class="button" type="submit" disabled=move || is_saving.get()>
                                    {move || if is_saving.get() { "Saving..." } else { "Save Permissions" }}
                                </button>
                            </div>
                            </form>
                        }
                        .into_any()
                    } else {
                        view! {
                            <section class="organization-state">
                                <h3>"Permissions not found"</h3>
                                <p>"No permissions record was returned for this account."</p>
                            </section>
                        }
                        .into_any()
                    }
                }}
            </section>
        </AppShell>
    }
}

#[component]
fn AdminScopeNodeList(nodes: Vec<AdminScopeNodeSummary>) -> impl IntoView {
    if nodes.is_empty() {
        view! { <p>"No scope nodes assigned."</p> }.into_any()
    } else {
        view! {
            <DataTable>
                <thead>
                    <tr>
                        <th scope="col">"Node"</th>
                        <th scope="col">"Type"</th>
                        <th scope="col">"Parent"</th>
                    </tr>
                </thead>
                <tbody>
                    {nodes
                        .into_iter()
                        .map(|node| {
                            view! {
                                <tr>
                                    <th scope="row">
                                        <a class="data-table__primary-link" href=format!("/organization/{}", node.node_id)>{node.node_name}</a>
                                    </th>
                                    <td>{node.node_type_name}</td>
                                    <td>{node.parent_node_name.unwrap_or_else(|| "-".to_string())}</td>
                                </tr>
                            }
                        })
                        .collect_view()}
                </tbody>
            </DataTable>
        }
        .into_any()
    }
}

#[component]
fn AdminScopeNodeChecklist(
    nodes: Vec<AdminScopeNodeSummary>,
    selected_node_ids: RwSignal<Vec<String>>,
) -> impl IntoView {
    let search = RwSignal::new(String::new());
    let node_count = nodes.len();
    if nodes.is_empty() {
        view! { <p>"No scope nodes are available."</p> }.into_any()
    } else {
        view! {
            <div class="permission-picker">
                <label class="searchable-data-table__search searchable-data-table__control">
                    <Search class="searchable-data-table__control-icon"/>
                    <span class="sr-only">"Search scope nodes"</span>
                    <input
                        type="search"
                        placeholder=format!("Search {node_count} scope nodes")
                        prop:value=move || search.get()
                        on:input=move |event| search.set(event_target_value(&event))
                    />
                </label>
                <div class="checkbox-list permission-picker__list">
                    {move || {
                        let query = search.get();
                        nodes
                            .iter()
                            .filter(|node| {
                                text_matches(
                                    &query,
                                    &[
                                        node.node_name.as_str(),
                                        node.node_type_name.as_str(),
                                        node.parent_node_name.as_deref().unwrap_or(""),
                                    ],
                                )
                            })
                            .cloned()
                            .map(|node| {
                                let node_id = node.node_id.clone();
                                let selected = selected_node_ids
                                    .get()
                                    .iter()
                                    .any(|selected_id| selected_id == &node.node_id);
                                let parent = node
                                    .parent_node_name
                                    .clone()
                                    .unwrap_or_else(|| "No parent".to_string());
                                view! {
                                    <label class="checkbox-list__item permission-picker__item">
                                        <input
                                            type="checkbox"
                                            prop:checked=selected
                                            on:change=move |event| {
                                                toggle_string_selection(
                                                    selected_node_ids,
                                                    node_id.clone(),
                                                    event_target_checked(&event),
                                                );
                                            }
                                        />
                                        <span>
                                            <strong>{node.node_name}</strong>
                                            <small>{node.node_type_name}</small>
                                            <small>{parent}</small>
                                        </span>
                                    </label>
                                }
                            })
                            .collect_view()
                    }}
                </div>
            </div>
        }
        .into_any()
    }
}

#[component]
fn AdminDelegationList(
    delegations: Vec<AdminDelegationSummary>,
    empty_label: &'static str,
) -> impl IntoView {
    if delegations.is_empty() {
        view! { <p>{empty_label}</p> }.into_any()
    } else {
        view! {
            <DataTable>
                <thead>
                    <tr>
                        <th scope="col">"Account"</th>
                        <th scope="col">"Email"</th>
                    </tr>
                </thead>
                <tbody>
                    {delegations
                        .into_iter()
                        .map(|delegation| {
                            view! {
                                <tr>
                                    <th scope="row">
                                        <a class="data-table__primary-link" href=format!("/administration/users/{}", delegation.account_id)>{delegation.display_name}</a>
                                    </th>
                                    <td>{delegation.email}</td>
                                </tr>
                            }
                        })
                        .collect_view()}
                </tbody>
            </DataTable>
        }
        .into_any()
    }
}

#[component]
fn AdminDelegationChecklist(
    delegations: Vec<AdminDelegationSummary>,
    selected_delegate_account_ids: RwSignal<Vec<String>>,
) -> impl IntoView {
    let search = RwSignal::new(String::new());
    let delegation_count = delegations.len();
    if delegations.is_empty() {
        view! { <p>"No delegate accounts are available."</p> }.into_any()
    } else {
        view! {
            <div class="permission-picker">
                <label class="searchable-data-table__search searchable-data-table__control">
                    <Search class="searchable-data-table__control-icon"/>
                    <span class="sr-only">"Search delegate accounts"</span>
                    <input
                        type="search"
                        placeholder=format!("Search {delegation_count} accounts")
                        prop:value=move || search.get()
                        on:input=move |event| search.set(event_target_value(&event))
                    />
                </label>
                <div class="checkbox-list permission-picker__list permission-picker__list--compact">
                    {move || {
                        let query = search.get();
                        delegations
                            .iter()
                            .filter(|delegation| {
                                text_matches(
                                    &query,
                                    &[delegation.display_name.as_str(), delegation.email.as_str()],
                                )
                            })
                            .cloned()
                            .map(|delegation| {
                                let account_id = delegation.account_id.clone();
                                let selected = selected_delegate_account_ids
                                    .get()
                                    .iter()
                                    .any(|selected_id| selected_id == &delegation.account_id);
                                view! {
                                    <label class="checkbox-list__item permission-picker__item">
                                        <input
                                            type="checkbox"
                                            prop:checked=selected
                                            on:change=move |event| {
                                                toggle_string_selection(
                                                    selected_delegate_account_ids,
                                                    account_id.clone(),
                                                    event_target_checked(&event),
                                                );
                                            }
                                        />
                                        <span>
                                            <strong>{delegation.display_name}</strong>
                                            <small>{delegation.email}</small>
                                        </span>
                                    </label>
                                }
                            })
                            .collect_view()
                    }}
                </div>
            </div>
        }
        .into_any()
    }
}

#[component]
fn AdminCapabilityList(capabilities: Vec<String>) -> impl IntoView {
    if capabilities.is_empty() {
        view! { <p>"No effective capabilities."</p> }.into_any()
    } else {
        view! {
            <ul class="capability-list">
                {capabilities
                    .into_iter()
                    .map(|capability| view! { <li class="capability-list__item">{capability}</li> })
                    .collect_view()}
            </ul>
        }
        .into_any()
    }
}

fn admin_user_status_key_from_bool(is_active: bool) -> &'static str {
    if is_active { "active" } else { "inactive" }
}

fn admin_user_status_label_from_bool(is_active: bool) -> &'static str {
    if is_active { "Active" } else { "Inactive" }
}

fn admin_role_names(roles: &[AdminRoleSummary]) -> String {
    if roles.is_empty() {
        "No roles".to_string()
    } else {
        roles
            .iter()
            .map(|role| role.name.as_str())
            .collect::<Vec<_>>()
            .join(", ")
    }
}

fn admin_access_profile_label(profile: &str) -> String {
    metadata_label(profile)
}

fn admin_editable_label(is_editable: bool) -> &'static str {
    if is_editable {
        "Editable"
    } else {
        "Not editable"
    }
}

#[component]
pub fn AdministrationNodeTypesPage() -> impl IntoView {
    let node_types = RwSignal::new(Vec::<NodeTypeCatalogEntry>::new());
    let selected_node_type_id = RwSignal::new(None::<String>);
    let selected_detail = RwSignal::new(None::<NodeTypeDefinition>);
    let search = RwSignal::new(String::new());
    let is_loading = RwSignal::new(true);
    let detail_loading = RwSignal::new(false);
    let is_saving = RwSignal::new(false);
    let is_creating = RwSignal::new(false);
    let message = RwSignal::new(None::<String>);
    let name = RwSignal::new(String::new());
    let slug = RwSignal::new(String::new());
    let plural_label = RwSignal::new(String::new());
    let parent_node_type_ids = RwSignal::new(HashSet::<String>::new());
    let child_node_type_ids = RwSignal::new(HashSet::<String>::new());

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

#[component]
fn AdministrationNodeTypesList(
    node_types: Vec<NodeTypeCatalogEntry>,
    search: RwSignal<String>,
    selected_node_type_id: RwSignal<Option<String>>,
    is_creating: bool,
) -> impl IntoView {
    let is_open = RwSignal::new(false);
    let options_for_label = node_types.clone();
    let options_for_visible = node_types;
    let selected_label = move || {
        if is_creating {
            "New node type".to_string()
        } else {
            selected_node_type_id
                .get()
                .as_deref()
                .and_then(|selected| {
                    options_for_label
                        .iter()
                        .find(|node_type| node_type.id == selected)
                })
                .map(|node_type| {
                    format!(
                        "{} ({}, {} nodes)",
                        node_type.name, node_type.slug, node_type.node_count
                    )
                })
                .unwrap_or_else(|| "Select node type".to_string())
        }
    };
    let visible_node_types = move || {
        let query = search.get().trim().to_lowercase();
        options_for_visible
            .iter()
            .filter(|node_type| {
                query.is_empty()
                    || node_type.name.to_lowercase().contains(&query)
                    || node_type.slug.to_lowercase().contains(&query)
                    || node_type.singular_label.to_lowercase().contains(&query)
                    || node_type.plural_label.to_lowercase().contains(&query)
            })
            .cloned()
            .collect::<Vec<_>>()
    };
    view! {
        <section class="administration-node-types-selector">
            <h2>"Node Type Catalog"</h2>
            <div class=move || if is_open.get() { "forms-node-filter node-type-selector is-open" } else { "forms-node-filter node-type-selector" }>
                <button
                    class=move || if selected_node_type_id.get().is_some() && !is_creating { "forms-node-filter__trigger is-filtered" } else { "forms-node-filter__trigger" }
                    type="button"
                    role="combobox"
                    aria-haspopup="listbox"
                    aria-expanded=move || is_open.get().to_string()
                    aria-label="Select node type"
                    title="Select node type"
                    on:click=move |_| is_open.update(|open| *open = !*open)
                >
                    <ListFilter/>
                    <span>{selected_label}</span>
                    <ChevronDown/>
                </button>
                <button
                    class="forms-node-filter__scrim"
                    type="button"
                    aria-label="Close node type selector"
                    on:click=move |_| is_open.set(false)
                ></button>
                <div
                    class="forms-node-filter__menu blurred-surface floating-layer"
                    data-mobile-behavior="dialog"
                    role="dialog"
                    aria-label="Select node type"
                >
                    <label class="forms-node-filter__search">
                        <Search/>
                        <span class="sr-only">"Search node types"</span>
                        <input
                            type="search"
                            placeholder="Search node types"
                            prop:value=move || search.get()
                            on:input=move |event| search.set(event_target_value(&event))
                        />
                    </label>
                    <div class="forms-node-filter__options" role="listbox">
                        {move || {
                            let visible = visible_node_types();
                            if visible.is_empty() {
                                view! {
                                    <p class="forms-node-filter__empty">"No matching node types to display"</p>
                                }
                                .into_any()
                            } else {
                                visible
                                    .into_iter()
                                    .map(|node_type| {
                                        let node_type_id = node_type.id.clone();
                                        let selected_node_type_id_for_option = selected_node_type_id;
                                        let search_for_option = search;
                                        let is_selected = selected_node_type_id
                                            .get()
                                            .map(|selected| selected == node_type_id)
                                            .unwrap_or(false);
                                        view! {
                                            <button
                                                class=if is_selected { "forms-node-filter__option is-active node-type-selector__option" } else { "forms-node-filter__option node-type-selector__option" }
                                                type="button"
                                                role="option"
                                                aria-selected=is_selected.to_string()
                                                on:click=move |_| {
                                                    selected_node_type_id_for_option.set(Some(node_type_id.clone()));
                                                    search_for_option.set(String::new());
                                                    is_open.set(false);
                                                }
                                            >
                                                <span>
                                                    <strong>{node_type.name}</strong>
                                                    <small>{node_type.slug}</small>
                                                </span>
                                                <span class="node-type-list__meta">{node_type.node_count} " nodes"</span>
                                            </button>
                                        }
                                    })
                                    .collect_view()
                                    .into_any()
                            }
                        }}
                    </div>
                </div>
            </div>
        </section>
    }
}

#[component]
fn AdministrationNodeTypeEditor(
    all_node_types: Vec<NodeTypeCatalogEntry>,
    selected_detail: Option<NodeTypeDefinition>,
    is_creating: bool,
    detail_loading: bool,
    is_saving: bool,
    message: Option<String>,
    selected_node_type_id: RwSignal<Option<String>>,
    name: RwSignal<String>,
    slug: RwSignal<String>,
    plural_label: RwSignal<String>,
    parent_node_type_ids: RwSignal<HashSet<String>>,
    child_node_type_ids: RwSignal<HashSet<String>>,
    on_cancel: impl Fn(leptos::ev::MouseEvent) + 'static + Copy,
    on_submit: impl Fn(leptos::ev::SubmitEvent) + 'static + Copy,
    on_metadata_changed: impl Fn() + 'static + Copy + Send + Sync,
) -> impl IntoView {
    let title = if is_creating {
        "Create Node Type".to_string()
    } else {
        selected_detail
            .as_ref()
            .map(|detail| detail.name.clone())
            .unwrap_or_else(|| "Select a Node Type".to_string())
    };
    let current_id = selected_node_type_id.get();
    let ancestor_ids = current_id
        .as_deref()
        .map(|id| node_type_ancestor_ids(id, &all_node_types))
        .unwrap_or_default();
    let descendant_ids = current_id
        .as_deref()
        .map(|id| node_type_descendant_ids(id, &all_node_types))
        .unwrap_or_default();
    let node_type_kind_label = move || {
        if parent_node_type_ids.get().is_empty() {
            "Root Type"
        } else {
            "Child Type"
        }
    };
    let node_type_kind_status = move || {
        if parent_node_type_ids.get().is_empty() {
            status_badge_class("active")
        } else {
            status_badge_class("inactive")
        }
    };
    let parent_picker_node_types = {
        let all_node_types = all_node_types.clone();
        let current_id = current_id.clone();
        let descendant_ids = descendant_ids.clone();
        move || {
            let selected_child_ids = child_node_type_ids.get();
            let mut disqualified_parent_ids = descendant_ids.clone();
            for child_id in &selected_child_ids {
                disqualified_parent_ids.insert(child_id.clone());
                disqualified_parent_ids.extend(node_type_descendant_ids(child_id, &all_node_types));
            }
            all_node_types
                .iter()
                .filter(|node_type| current_id.as_ref() != Some(&node_type.id))
                .filter(|node_type| !disqualified_parent_ids.contains(&node_type.id))
                .cloned()
                .collect::<Vec<_>>()
        }
    };
    let child_picker_node_types = {
        let all_node_types = all_node_types.clone();
        let current_id = current_id.clone();
        let ancestor_ids = ancestor_ids.clone();
        move || {
            let selected_parent_ids = parent_node_type_ids.get();
            let mut disqualified_child_ids = ancestor_ids.clone();
            for parent_id in &selected_parent_ids {
                disqualified_child_ids.insert(parent_id.clone());
                disqualified_child_ids.extend(node_type_ancestor_ids(parent_id, &all_node_types));
            }
            all_node_types
                .iter()
                .filter(|node_type| current_id.as_ref() != Some(&node_type.id))
                .filter(|node_type| !disqualified_child_ids.contains(&node_type.id))
                .cloned()
                .collect::<Vec<_>>()
        }
    };

    view! {
        <form class="native-form administration-node-type-editor" on:submit=on_submit>
            <section class="organization-detail-card organization-detail-card--wide">
                <div class="organization-detail-card__header">
                    <h2>{title}</h2>
                    {if selected_detail.is_some() || is_creating {
                        view! {
                            <span class=move || node_type_kind_status()>{move || node_type_kind_label()}</span>
                        }
                        .into_any()
                    } else {
                        view! { <span></span> }.into_any()
                    }}
                </div>

                {if detail_loading {
                    view! {
                        <section class="organization-state" aria-live="polite">
                            <h3>"Loading node type"</h3>
                            <p>"Fetching node type details."</p>
                        </section>
                    }
                    .into_any()
                } else {
                    view! {
                        <div class="form-grid administration-node-type-fields">
                            <label class="form-field">
                                <span>"Name"</span>
                                <input
                                    type="text"
                                    placeholder="Program"
                                    prop:value=move || name.get()
                                    on:input=move |event| name.set(event_target_value(&event))
                                />
                            </label>
                            <label class="form-field">
                                <span>"Slug"</span>
                                <input
                                    type="text"
                                    placeholder="program"
                                    prop:value=move || slug.get()
                                    on:input=move |event| slug.set(event_target_value(&event))
                                />
                            </label>
                            <label class="form-field">
                                <span>"Plural Label"</span>
                                <input
                                    type="text"
                                    placeholder="Programs"
                                    prop:value=move || plural_label.get()
                                    on:input=move |event| plural_label.set(event_target_value(&event))
                                />
                            </label>
                            {if let Some(detail) = selected_detail.as_ref() {
                                let node_count = detail.node_count.to_string();
                                let metadata_count = detail.metadata_fields.len().to_string();
                                let scoped_form_count = detail.scoped_forms.len().to_string();
                                view! {
                                    <div class="node-type-count-badges" aria-label="Node type counts">
                                        <span class="node-type-count-badge">
                                            <strong>{node_count}</strong>
                                            <span>"Nodes"</span>
                                        </span>
                                        <span class="node-type-count-badge">
                                            <strong>{metadata_count}</strong>
                                            <span>"Metadata Fields"</span>
                                        </span>
                                        <span class="node-type-count-badge">
                                            <strong>{scoped_form_count}</strong>
                                            <span>"Scoped Forms"</span>
                                        </span>
                                    </div>
                                }
                                .into_any()
                            } else {
                                view! { <span></span> }.into_any()
                            }}
                        </div>
                        {if selected_detail.is_none() {
                            view! { <p class="muted">"Configure labels and hierarchy relationships, then save this node type."</p> }.into_any()
                        } else {
                            view! { <span></span> }.into_any()
                        }}

                        <div class="administration-node-type-relationships">
                            {move || {
                                view! {
                                    <NodeTypeRelationshipPicker
                                        title="Allowed Parent Types"
                                        empty_message="No eligible parent types are available."
                                        node_types=parent_picker_node_types()
                                        selected_ids=parent_node_type_ids
                                        opposite_selected_ids=child_node_type_ids
                                    />
                                }
                            }}
                            {move || {
                                view! {
                                    <NodeTypeRelationshipPicker
                                        title="Allowed Child Types"
                                        empty_message="No eligible child types are available."
                                        node_types=child_picker_node_types()
                                        selected_ids=child_node_type_ids
                                        opposite_selected_ids=parent_node_type_ids
                                    />
                                }
                            }}
                        </div>

                        <NodeTypeDetailCollections detail=selected_detail.clone() on_metadata_changed/>

                        {if let Some(message) = message {
                            view! { <p class="form-message">{message}</p> }.into_any()
                        } else {
                            view! { <span></span> }.into_any()
                        }}

                        <div class="form-actions">
                            {if is_creating {
                                view! {
                                    <button class="button button--secondary" type="button" on:click=on_cancel>"Cancel"</button>
                                }
                                .into_any()
                            } else {
                                view! { <span></span> }.into_any()
                            }}
                            <button class="button" type="submit" disabled=is_saving>
                                {if is_saving { "Saving Node Type" } else { "Save Node Type" }}
                            </button>
                        </div>
                    }
                    .into_any()
                }}
            </section>
        </form>
    }
}

#[component]
fn NodeTypeRelationshipPicker(
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

#[component]
fn NodeTypeDetailCollections(
    detail: Option<NodeTypeDefinition>,
    on_metadata_changed: impl Fn() + 'static + Copy + Send + Sync,
) -> impl IntoView {
    if let Some(detail) = detail {
        view! {
            <div class="administration-node-type-collections">
                <NodeTypeMetadataList
                    node_type_id=detail.id
                    fields=detail.metadata_fields
                    on_metadata_changed
                />
                <NodeTypeScopedFormsList forms=detail.scoped_forms/>
            </div>
        }
        .into_any()
    } else {
        view! { <div></div> }.into_any()
    }
}

#[component]
fn NodeTypeMetadataList(
    node_type_id: String,
    fields: Vec<NodeMetadataFieldSummary>,
    on_metadata_changed: impl Fn() + 'static + Copy + Send + Sync,
) -> impl IntoView {
    #[cfg(not(feature = "hydrate"))]
    let _ = (&node_type_id, &on_metadata_changed);
    let node_type_id_value = RwSignal::new(node_type_id);
    let search = RwSignal::new(String::new());
    let editing_field_id = RwSignal::new(None::<String>);
    let field_label = RwSignal::new(String::new());
    let field_key = RwSignal::new(String::new());
    let field_type = RwSignal::new("text".to_string());
    let field_required = RwSignal::new(false);
    let is_saving_field = RwSignal::new(false);
    let field_message = RwSignal::new(None::<String>);
    let sheet_open = RwSignal::new(false);
    let has_fields = !fields.is_empty();
    let table_searchable_fields = fields.clone();
    let card_searchable_fields = fields;
    let table_fields = move || {
        let query = search.get().trim().to_lowercase();
        table_searchable_fields
            .iter()
            .filter(|field| {
                query.is_empty()
                    || field.label.to_lowercase().contains(&query)
                    || field.key.to_lowercase().contains(&query)
                    || field.field_type.to_lowercase().contains(&query)
            })
            .cloned()
            .collect::<Vec<_>>()
    };
    let card_fields = move || {
        let query = search.get().trim().to_lowercase();
        card_searchable_fields
            .iter()
            .filter(|field| {
                query.is_empty()
                    || field.label.to_lowercase().contains(&query)
                    || field.key.to_lowercase().contains(&query)
                    || field.field_type.to_lowercase().contains(&query)
            })
            .cloned()
            .collect::<Vec<_>>()
    };
    let clear_field_editor = move || {
        editing_field_id.set(None);
        field_label.set(String::new());
        field_key.set(String::new());
        field_type.set("text".to_string());
        field_required.set(false);
        field_message.set(None);
    };
    let close_field_sheet = move |_| {
        sheet_open.set(false);
        clear_field_editor();
    };
    let open_new_field_sheet = move |_| {
        clear_field_editor();
        sheet_open.set(true);
    };

    view! {
        <section class="organization-detail-card node-type-detail-list node-type-detail-list--wide">
            <div class="node-type-detail-list__header">
                <h3>"Metadata Fields"</h3>
            </div>
            <div class="forms-list forms-list-responsive-table node-type-metadata-list">
                <div class="searchable-data-table">
                    <div class="searchable-data-table__toolbar forms-list__toolbar">
                        <label class="searchable-data-table__search searchable-data-table__control">
                            <Search class="searchable-data-table__control-icon"/>
                            <span class="sr-only">"Search metadata fields"</span>
                            <input
                                type="search"
                                placeholder="Search metadata"
                                prop:value=move || search.get()
                                on:input=move |event| search.set(event_target_value(&event))
                            />
                        </label>
                        <button
                            class="button button--compact button--secondary node-type-add-field-button"
                            type="button"
                            on:click=open_new_field_sheet
                        >
                            <Plus class="button__icon"/>
                            "Add Field"
                        </button>
                    </div>
                    <Show when=move || field_message.get().is_some() && !sheet_open.get()>
                        <p class="form-message">{move || field_message.get().unwrap_or_default()}</p>
                    </Show>
                    {if !has_fields {
                        view! { <p class="muted">"No metadata fields configured."</p> }.into_any()
                    } else {
                        view! {
                            <DataTable>
                                <thead>
                                    <tr>
                                        <th scope="col">"Field"</th>
                                        <th scope="col">"Key"</th>
                                        <th scope="col">"Type"</th>
                                        <th scope="col">"Required"</th>
                                        <th class="data-table__cell--center" scope="col">"Actions"</th>
                                    </tr>
                                </thead>
                                <tbody>
                                    {move || {
                                        let visible_fields = table_fields();
                                        if visible_fields.is_empty() {
                                            view! {
                                                <tr>
                                                    <td class="data-table__empty" colspan="5">"No Metadata Fields to Display"</td>
                                                </tr>
                                            }
                                            .into_any()
                                        } else {
                                            visible_fields
                                                .into_iter()
                                                .map(|field| {
                                                    let edit_field = field.clone();
                                                    let delete_field = field.clone();
                                                    let row_label = field.label.clone();
                                                    view! {
                                                        <tr>
                                                            <th scope="row">{field.label}</th>
                                                            <td>{field.key}</td>
                                                            <td>{metadata_label(&field.field_type)}</td>
                                                            <td>{if field.required { "Yes" } else { "No" }}</td>
                                                            <td class="data-table__cell--center">
                                                                <DropdownMenu label=format!("Open actions for {row_label}")>
                                                                    <button
                                                                        class="dropdown-menu__item"
                                                                        type="button"
                                                                        role="menuitem"
                                                                        on:click=move |_| {
                                                                            editing_field_id.set(Some(edit_field.id.clone()));
                                                                            field_label.set(edit_field.label.clone());
                                                                            field_key.set(edit_field.key.clone());
                                                                            field_type.set(edit_field.field_type.clone());
                                                                            field_required.set(edit_field.required);
                                                                            field_message.set(None);
                                                                            sheet_open.set(true);
                                                                        }
                                                                    >
                                                                        <Pencil class="dropdown-menu__item-icon"/>
                                                                        <span>"Edit Field"</span>
                                                                    </button>
                                                                    <button
                                                                        class="dropdown-menu__item"
                                                                        type="button"
                                                                        role="menuitem"
                                                                        on:click=move |_| {
                                                                            #[cfg(feature = "hydrate")]
                                                                            {
                                                                                let field_id = delete_field.id.clone();
                                                                                leptos::task::spawn_local(async move {
                                                                                    field_message.set(None);
                                                                                    match send_json_id_request(
                                                                                        gloo_net::http::Request::delete(&format!(
                                                                                            "/api/admin/node-metadata-fields/{field_id}"
                                                                                        )),
                                                                                        None,
                                                                                        "Delete metadata field",
                                                                                    )
                                                                                    .await
                                                                                    {
                                                                                        Ok(_) => {
                                                                                            sheet_open.set(false);
                                                                                            clear_field_editor();
                                                                                            on_metadata_changed();
                                                                                        }
                                                                                        Err(error) => field_message.set(Some(error)),
                                                                                    }
                                                                                });
                                                                            }
                                                                            #[cfg(not(feature = "hydrate"))]
                                                                            let _ = &delete_field;
                                                                        }
                                                                    >
                                                                        <Trash2 class="dropdown-menu__item-icon"/>
                                                                        <span>"Delete Field"</span>
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
                        }
                        .into_any()
                    }}
                </div>
                <div class="forms-list-mobile-cards node-type-metadata-mobile-cards">
                    {if !has_fields {
                        view! { <p class="forms-list-mobile-empty">"No Metadata Fields to Display"</p> }.into_any()
                    } else {
                        view! {
                            {move || {
                                let visible_fields = card_fields();
                                if visible_fields.is_empty() {
                                    view! { <p class="forms-list-mobile-empty">"No Metadata Fields to Display"</p> }.into_any()
                                } else {
                                    visible_fields
                                        .into_iter()
                                        .map(|field| {
                                            let edit_field = field.clone();
                                            let delete_field = field.clone();
                                            view! {
                                                <article class="forms-list-mobile-card node-type-metadata-mobile-card">
                                                    <div class="forms-list-mobile-card__header">
                                                        <div>
                                                            <h3>{field.label}</h3>
                                                            <span>{field.key}</span>
                                                        </div>
                                                        <span class=status_badge_class(if field.required { "active" } else { "inactive" })>
                                                            {if field.required { "Required" } else { "Optional" }}
                                                        </span>
                                                    </div>
                                                    <dl>
                                                        <div>
                                                            <dt>"Type"</dt>
                                                            <dd>{metadata_label(&field.field_type)}</dd>
                                                        </div>
                                                    </dl>
                                                    <div class="workflow-assignment-mobile-card__actions">
                                                        <button
                                                            class="button button--compact button--secondary"
                                                            type="button"
                                                            on:click=move |_| {
                                                                editing_field_id.set(Some(edit_field.id.clone()));
                                                                field_label.set(edit_field.label.clone());
                                                                field_key.set(edit_field.key.clone());
                                                                field_type.set(edit_field.field_type.clone());
                                                                field_required.set(edit_field.required);
                                                                field_message.set(None);
                                                                sheet_open.set(true);
                                                            }
                                                        >
                                                            "Edit Field"
                                                        </button>
                                                        <button
                                                            class="button button--compact button--secondary"
                                                            type="button"
                                                            on:click=move |_| {
                                                                #[cfg(feature = "hydrate")]
                                                                {
                                                                    let field_id = delete_field.id.clone();
                                                                    leptos::task::spawn_local(async move {
                                                                        field_message.set(None);
                                                                        match send_json_id_request(
                                                                            gloo_net::http::Request::delete(&format!(
                                                                                "/api/admin/node-metadata-fields/{field_id}"
                                                                            )),
                                                                            None,
                                                                            "Delete metadata field",
                                                                        )
                                                                        .await
                                                                        {
                                                                            Ok(_) => {
                                                                                sheet_open.set(false);
                                                                                clear_field_editor();
                                                                                on_metadata_changed();
                                                                            }
                                                                            Err(error) => field_message.set(Some(error)),
                                                                        }
                                                                    });
                                                                }
                                                                #[cfg(not(feature = "hydrate"))]
                                                                let _ = &delete_field;
                                                            }
                                                        >
                                                            "Delete Field"
                                                        </button>
                                                    </div>
                                                </article>
                                            }
                                        })
                                        .collect_view()
                                        .into_any()
                                }
                            }}
                        }
                        .into_any()
                    }}
                </div>
            </div>
            <Portal>
                <Show when=move || sheet_open.get()>
                    <section class="sheet-overlay node-type-metadata-overlay" aria-label="Metadata field editor">
                        <button class="sheet-overlay__scrim" type="button" aria-label="Close metadata field editor" on:click=close_field_sheet></button>
                        <aside class="sheet-panel blurred-surface node-type-metadata-sheet" role="dialog" aria-modal="true" aria-label="Metadata field editor">
                            <div class="sheet-panel__actions">
                                <button class="icon-button sheet-panel__close" type="button" aria-label="Close metadata field editor" title="Close metadata field editor" on:click=close_field_sheet>
                                    <X/>
                                </button>
                            </div>

                            <header class="sheet-panel__header">
                                <p>"Node Type Metadata"</p>
                                <h2>{move || if editing_field_id.get().is_some() { "Edit Metadata Field" } else { "Add Metadata Field" }}</h2>
                            </header>

                            <section class="sheet-panel__section">
                                <div class="form-grid node-type-metadata-sheet__fields">
                                    <label class="form-field">
                                        <span>"Label"</span>
                                        <input
                                            type="text"
                                            placeholder="Display label"
                                            prop:value=move || field_label.get()
                                            on:input=move |event| field_label.set(event_target_value(&event))
                                        />
                                    </label>
                                    <label class="form-field">
                                        <span>"Key"</span>
                                        <input
                                            type="text"
                                            placeholder="metadata_key"
                                            prop:value=move || field_key.get()
                                            on:input=move |event| field_key.set(event_target_value(&event))
                                        />
                                    </label>
                                    <label class="form-field">
                                        <span>"Type"</span>
                                        <select
                                            prop:value=move || field_type.get()
                                            on:change=move |event| field_type.set(event_target_value(&event))
                                        >
                                            <option value="text">"Text"</option>
                                            <option value="number">"Number"</option>
                                            <option value="boolean">"Boolean"</option>
                                            <option value="date">"Date"</option>
                                            <option value="single_choice">"Single Choice"</option>
                                            <option value="multi_choice">"Multi Choice"</option>
                                        </select>
                                    </label>
                                    <label class="toggle-row toggle-row--compact metadata-field-editor__required">
                                        <input
                                            type="checkbox"
                                            prop:checked=move || field_required.get()
                                            on:change=move |event| field_required.set(event_target_checked(&event))
                                        />
                                        <span>"Required"</span>
                                    </label>
                                </div>
                                <Show when=move || field_message.get().is_some()>
                                    <p class="form-message">{move || field_message.get().unwrap_or_default()}</p>
                                </Show>
                            </section>

                            <div class="form-actions">
                                <button
                                    class="button button--secondary"
                                    type="button"
                                    on:click=close_field_sheet
                                >
                                    "Cancel"
                                </button>
                                <button
                                    class="button"
                                    type="button"
                                    disabled=move || is_saving_field.get()
                                    on:click=move |_| {
                                        save_node_type_metadata_field(
                                            node_type_id_value.get_untracked(),
                                            editing_field_id,
                                            field_label,
                                            field_key,
                                            field_type,
                                            field_required,
                                            is_saving_field,
                                            field_message,
                                            sheet_open,
                                            clear_field_editor,
                                            on_metadata_changed,
                                        );
                                    }
                                >
                                    {move || {
                                        if is_saving_field.get() {
                                            "Saving"
                                        } else if editing_field_id.get().is_some() {
                                            "Save Field"
                                        } else {
                                            "Create Field"
                                        }
                                    }}
                                </button>
                            </div>
                        </aside>
                    </section>
                </Show>
            </Portal>
        </section>
    }
}

fn save_node_type_metadata_field(
    node_type_id: String,
    editing_field_id: RwSignal<Option<String>>,
    field_label: RwSignal<String>,
    field_key: RwSignal<String>,
    field_type: RwSignal<String>,
    field_required: RwSignal<bool>,
    is_saving_field: RwSignal<bool>,
    field_message: RwSignal<Option<String>>,
    sheet_open: RwSignal<bool>,
    clear_field_editor: impl Fn() + 'static + Copy + Send + Sync,
    on_metadata_changed: impl Fn() + 'static + Copy + Send + Sync,
) {
    let label = field_label.get().trim().to_string();
    let key = field_key.get().trim().to_string();
    let field_type_value = field_type.get();
    let required = field_required.get();
    if label.is_empty() || key.is_empty() {
        field_message.set(Some("Metadata label and key are required.".into()));
        return;
    }

    #[cfg(feature = "hydrate")]
    {
        let editing_id = editing_field_id.get_untracked();
        leptos::task::spawn_local(async move {
            is_saving_field.set(true);
            field_message.set(None);
            let result = if let Some(field_id) = editing_id {
                let request = UpdateNodeMetadataFieldRequest {
                    key,
                    label,
                    field_type: field_type_value,
                    required,
                };
                match serde_json::to_string(&request) {
                    Ok(body) => {
                        send_json_id_request(
                            gloo_net::http::Request::put(&format!(
                                "/api/admin/node-metadata-fields/{field_id}"
                            )),
                            Some(body),
                            "Save metadata field",
                        )
                        .await
                    }
                    Err(_) => Err("Metadata field request could not be prepared.".into()),
                }
            } else {
                let request = CreateNodeMetadataFieldRequest {
                    node_type_id,
                    key,
                    label,
                    field_type: field_type_value,
                    required,
                };
                match serde_json::to_string(&request) {
                    Ok(body) => {
                        send_json_id_request(
                            gloo_net::http::Request::post("/api/admin/node-metadata-fields"),
                            Some(body),
                            "Create metadata field",
                        )
                        .await
                    }
                    Err(_) => Err("Metadata field request could not be prepared.".into()),
                }
            };

            match result {
                Ok(_) => {
                    sheet_open.set(false);
                    clear_field_editor();
                    on_metadata_changed();
                }
                Err(error) => field_message.set(Some(error)),
            }
            is_saving_field.set(false);
        });
    }
    #[cfg(not(feature = "hydrate"))]
    let _ = (
        node_type_id,
        editing_field_id,
        label,
        key,
        field_type_value,
        required,
        is_saving_field,
        sheet_open,
        clear_field_editor,
        on_metadata_changed,
    );
}

#[component]
fn NodeTypeScopedFormsList(forms: Vec<NodeTypeFormLink>) -> impl IntoView {
    let search = RwSignal::new(String::new());
    let has_forms = !forms.is_empty();
    let searchable_forms = forms.clone();
    let filtered_forms = move || {
        let query = search.get().trim().to_lowercase();
        searchable_forms
            .iter()
            .filter(|form| {
                query.is_empty()
                    || form.form_name.to_lowercase().contains(&query)
                    || form.form_slug.to_lowercase().contains(&query)
            })
            .cloned()
            .collect::<Vec<_>>()
    };
    view! {
        <section class="organization-detail-card node-type-detail-list node-type-detail-list--wide">
            <div class="node-type-detail-list__header">
                <h3>"Scoped Forms"</h3>
                <label class="searchable-data-table__search searchable-data-table__control node-type-detail-list__search">
                    <Search class="searchable-data-table__control-icon"/>
                    <span class="sr-only">"Search scoped forms"</span>
                    <input
                        type="search"
                        placeholder="Search forms"
                        prop:value=move || search.get()
                        on:input=move |event| search.set(event_target_value(&event))
                    />
                </label>
            </div>
            {if !has_forms {
                view! { <p class="muted">"No forms are scoped to this node type."</p> }.into_any()
            } else {
                view! {
                    <div class="capability-list node-type-scoped-forms-list">
                        {move || {
                            let visible_forms = filtered_forms();
                            if visible_forms.is_empty() {
                                view! { <div class="capability-list__item">"No scoped forms match this search."</div> }.into_any()
                            } else {
                                visible_forms
                                    .into_iter()
                                    .map(|form| view! {
                                        <div class="capability-list__item">
                                            <strong>{form.form_name}</strong>
                                            <small>{form.form_slug}</small>
                                        </div>
                                    })
                                    .collect_view()
                                    .into_any()
                            }
                        }}
                    </div>
                }
                .into_any()
            }}
        </section>
    }
}

fn node_type_ancestor_ids(
    node_type_id: &str,
    node_types: &[NodeTypeCatalogEntry],
) -> HashSet<String> {
    let mut ancestors = HashSet::new();
    let mut stack = node_types
        .iter()
        .find(|node_type| node_type.id == node_type_id)
        .map(|node_type| {
            node_type
                .parent_relationships
                .iter()
                .map(|peer| peer.node_type_id.clone())
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();

    while let Some(candidate_id) = stack.pop() {
        if ancestors.insert(candidate_id.clone()) {
            if let Some(candidate) = node_types
                .iter()
                .find(|node_type| node_type.id == candidate_id)
            {
                stack.extend(
                    candidate
                        .parent_relationships
                        .iter()
                        .map(|peer| peer.node_type_id.clone()),
                );
            }
        }
    }

    ancestors
}

fn node_type_descendant_ids(
    node_type_id: &str,
    node_types: &[NodeTypeCatalogEntry],
) -> HashSet<String> {
    let mut descendants = HashSet::new();
    let mut stack = node_types
        .iter()
        .find(|node_type| node_type.id == node_type_id)
        .map(|node_type| {
            node_type
                .child_relationships
                .iter()
                .map(|peer| peer.node_type_id.clone())
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();

    while let Some(candidate_id) = stack.pop() {
        if descendants.insert(candidate_id.clone()) {
            if let Some(candidate) = node_types
                .iter()
                .find(|node_type| node_type.id == candidate_id)
            {
                stack.extend(
                    candidate
                        .child_relationships
                        .iter()
                        .map(|peer| peer.node_type_id.clone()),
                );
            }
        }
    }

    descendants
}

fn load_admin_node_type_catalog(
    node_types: RwSignal<Vec<NodeTypeCatalogEntry>>,
    selected_node_type_id: RwSignal<Option<String>>,
    is_loading: RwSignal<bool>,
    message: RwSignal<Option<String>>,
    preferred_id: Option<String>,
) {
    #[cfg(feature = "hydrate")]
    {
        leptos::task::spawn_local(async move {
            is_loading.set(true);
            match gloo_net::http::Request::get("/api/node-types").send().await {
                Ok(response) if response.status() == 401 => {
                    is_loading.set(false);
                    redirect_to_login();
                }
                Ok(response) if response.ok() => {
                    match response.json::<Vec<NodeTypeCatalogEntry>>().await {
                        Ok(items) => {
                            let selected = preferred_id
                                .or_else(|| {
                                    selected_node_type_id
                                        .get_untracked()
                                        .filter(|id| items.iter().any(|item| item.id == *id))
                                })
                                .or_else(|| items.first().map(|item| item.id.clone()));
                            node_types.set(items);
                            selected_node_type_id.set(selected);
                            message.set(None);
                        }
                        Err(_) => {
                            node_types.set(Vec::new());
                            message.set(Some("Node type response could not be read.".into()));
                        }
                    }
                    is_loading.set(false);
                }
                Ok(response) => {
                    let status = response.status();
                    node_types.set(Vec::new());
                    message.set(Some(format!(
                        "Load node types failed with status {status}."
                    )));
                    is_loading.set(false);
                }
                Err(_) => {
                    node_types.set(Vec::new());
                    message.set(Some("Could not reach the node type API.".into()));
                    is_loading.set(false);
                }
            }
        });
    }
    #[cfg(not(feature = "hydrate"))]
    {
        let _ = (node_types, selected_node_type_id, message, preferred_id);
        is_loading.set(false);
    }
}

fn load_admin_node_type_detail(
    node_type_id: String,
    selected_detail: RwSignal<Option<NodeTypeDefinition>>,
    detail_loading: RwSignal<bool>,
    message: RwSignal<Option<String>>,
    name: RwSignal<String>,
    slug: RwSignal<String>,
    plural_label: RwSignal<String>,
    parent_node_type_ids: RwSignal<HashSet<String>>,
    child_node_type_ids: RwSignal<HashSet<String>>,
) {
    #[cfg(feature = "hydrate")]
    {
        leptos::task::spawn_local(async move {
            detail_loading.set(true);
            match gloo_net::http::Request::get(&format!("/api/admin/node-types/{node_type_id}"))
                .send()
                .await
            {
                Ok(response) if response.status() == 401 => {
                    detail_loading.set(false);
                    redirect_to_login();
                }
                Ok(response) if response.ok() => {
                    match response.json::<NodeTypeDefinition>().await {
                        Ok(detail) => {
                            name.set(detail.name.clone());
                            slug.set(detail.slug.clone());
                            plural_label.set(detail.plural_label.clone());
                            parent_node_type_ids.set(
                                detail
                                    .parent_relationships
                                    .iter()
                                    .map(|peer| peer.node_type_id.clone())
                                    .collect(),
                            );
                            child_node_type_ids.set(
                                detail
                                    .child_relationships
                                    .iter()
                                    .map(|peer| peer.node_type_id.clone())
                                    .collect(),
                            );
                            selected_detail.set(Some(detail));
                            message.set(None);
                        }
                        Err(_) => {
                            selected_detail.set(None);
                            message
                                .set(Some("Node type detail response could not be read.".into()));
                        }
                    }
                    detail_loading.set(false);
                }
                Ok(response) => {
                    selected_detail.set(None);
                    message.set(Some(format!(
                        "Load node type detail failed with status {}.",
                        response.status()
                    )));
                    detail_loading.set(false);
                }
                Err(_) => {
                    selected_detail.set(None);
                    message.set(Some("Could not reach the node type detail API.".into()));
                    detail_loading.set(false);
                }
            }
        });
    }
    #[cfg(not(feature = "hydrate"))]
    {
        let _ = (
            node_type_id,
            selected_detail,
            message,
            name,
            slug,
            plural_label,
            parent_node_type_ids,
            child_node_type_ids,
        );
        detail_loading.set(false);
    }
}

#[component]
pub fn AdministrationRolesPage() -> impl IntoView {
    let roles = RwSignal::new(Vec::<AdminRoleSummary>::new());
    let capabilities = RwSignal::new(Vec::<AdminCapabilitySummary>::new());
    let selected_role_id = RwSignal::new(None::<String>);
    let selected_role_detail = RwSignal::new(None::<AdminRoleDetail>);
    let search = RwSignal::new(String::new());
    let is_loading = RwSignal::new(true);
    let detail_loading = RwSignal::new(false);
    let is_saving = RwSignal::new(false);
    let message = RwSignal::new(None::<String>);
    let sheet_open = RwSignal::new(false);
    let editing_role_id = RwSignal::new(None::<String>);
    let role_name = RwSignal::new(String::new());
    let selected_capability_ids = RwSignal::new(Vec::<String>::new());
    let capability_search = RwSignal::new(String::new());

    load_admin_roles_context(
        roles,
        capabilities,
        selected_role_id,
        is_loading,
        message,
        None,
    );

    Effect::new(move |_| {
        if let Some(role_id) = selected_role_id.get() {
            load_admin_role_detail(role_id, selected_role_detail, detail_loading, message);
        } else {
            selected_role_detail.set(None);
        }
    });

    let filtered_roles = move || {
        let query = search.get().trim().to_lowercase();
        roles
            .get()
            .into_iter()
            .filter(|role| query.is_empty() || role.name.to_lowercase().contains(&query))
            .collect::<Vec<_>>()
    };
    let open_create_sheet = move |_| {
        editing_role_id.set(None);
        role_name.set(String::new());
        selected_capability_ids.set(Vec::new());
        capability_search.set(String::new());
        message.set(None);
        sheet_open.set(true);
    };
    let open_edit_sheet = move |_| {
        if let Some(detail) = selected_role_detail.get() {
            editing_role_id.set(Some(detail.id));
            role_name.set(detail.name);
            selected_capability_ids.set(
                detail
                    .capabilities
                    .into_iter()
                    .map(|capability| capability.id)
                    .collect(),
            );
            capability_search.set(String::new());
            message.set(None);
            sheet_open.set(true);
        }
    };
    let close_sheet = move |_| {
        sheet_open.set(false);
        editing_role_id.set(None);
        role_name.set(String::new());
        selected_capability_ids.set(Vec::new());
        capability_search.set(String::new());
    };
    let save_role = move |_| {
        save_admin_role(
            editing_role_id,
            role_name,
            selected_capability_ids,
            is_saving,
            message,
            sheet_open,
            roles,
            capabilities,
            selected_role_id,
            selected_role_detail,
            detail_loading,
        );
    };

    view! {
        <AppShell active_route="administration" title="Roles">
            <section class="route-panel administration-roles-page">
                <Breadcrumb>
                    <BreadcrumbItem>
                        <BreadcrumbLink href="/administration">"Administration"</BreadcrumbLink>
                    </BreadcrumbItem>
                    <BreadcrumbSeparator/>
                    <BreadcrumbItem>
                        <BreadcrumbPage>"Roles"</BreadcrumbPage>
                    </BreadcrumbItem>
                </Breadcrumb>

                <PageHeader
                    title="Roles"
                    description="Manage reusable capability templates for Tessara users."
                >
                    <button class="button" type="button" on:click=open_create_sheet>
                        "New Role"
                    </button>
                </PageHeader>

                {move || {
                    if is_loading.get() {
                        view! {
                            <section class="organization-state" aria-live="polite">
                                <h3>"Loading roles"</h3>
                                <p>"Fetching role and capability configuration."</p>
                            </section>
                        }
                        .into_any()
                    } else if let Some(error) = message.get().filter(|_| roles.get().is_empty()) {
                        view! {
                            <section class="organization-state is-error" role="alert">
                                <h3>"Roles unavailable"</h3>
                                <p>{error}</p>
                            </section>
                        }
                        .into_any()
                    } else {
                        view! {
                            <div class="administration-roles-layout">
                                <AdministrationRolesList
                                    roles=filtered_roles()
                                    search
                                    selected_role_id
                                />
                                <AdministrationRoleDetailPanel
                                    detail=selected_role_detail.get()
                                    is_loading=detail_loading.get()
                                    on_edit=open_edit_sheet
                                />
                            </div>
                        }
                        .into_any()
                    }
                }}

                <AdminRoleSheet
                    is_open=sheet_open
                    editing_role_id
                    role_name
                    capabilities
                    selected_capability_ids
                    capability_search
                    is_saving
                    message
                    on_close=close_sheet
                    on_save=save_role
                />
            </section>
        </AppShell>
    }
}

#[component]
fn AdministrationRolesList(
    roles: Vec<AdminRoleSummary>,
    search: RwSignal<String>,
    selected_role_id: RwSignal<Option<String>>,
) -> impl IntoView {
    let table_roles = roles.clone();
    let card_roles = roles;

    view! {
        <div class="forms-list forms-list-responsive-table administration-roles-list">
            <div class="searchable-data-table">
                <div class="searchable-data-table__toolbar forms-list__toolbar">
                    <label class="searchable-data-table__search searchable-data-table__control">
                        <Search class="searchable-data-table__control-icon"/>
                        <span class="sr-only">"Search roles"</span>
                        <input
                            type="search"
                            placeholder="Search roles"
                            prop:value=move || search.get()
                            on:input=move |event| search.set(event_target_value(&event))
                        />
                    </label>
                </div>
                <DataTable>
                    <thead>
                        <tr>
                            <th scope="col">"Role"</th>
                            <th class="data-table__cell--center" scope="col">"Capabilities"</th>
                            <th class="data-table__cell--center" scope="col">"Users"</th>
                            <th class="data-table__cell--center" scope="col">"Actions"</th>
                        </tr>
                    </thead>
                    <tbody>
                        {if table_roles.is_empty() {
                            view! {
                                <tr>
                                    <td class="data-table__empty" colspan="4">"No Roles to Display"</td>
                                </tr>
                            }
                            .into_any()
                        } else {
                            table_roles
                                .into_iter()
                                .map(|role| {
                                    let role_id = role.id.clone();
                                    let role_name = role.name.clone();
                                    view! {
                                        <tr>
                                            <th scope="row">
                                                <button
                                                    class="link-button data-table__primary-link"
                                                    type="button"
                                                    on:click=move |_| selected_role_id.set(Some(role_id.clone()))
                                                >
                                                    {role.name}
                                                </button>
                                            </th>
                                            <td class="data-table__cell--center">{role.capability_count}</td>
                                            <td class="data-table__cell--center">{role.account_count}</td>
                                            <td class="data-table__cell--center">
                                                <DropdownMenu label=format!("Open actions for {role_name}")>
                                                    <button
                                                        class="dropdown-menu__item"
                                                        type="button"
                                                        role="menuitem"
                                                        on:click=move |_| selected_role_id.set(Some(role.id.clone()))
                                                    >
                                                        <PanelRight class="dropdown-menu__item-icon"/>
                                                        <span>"View Details"</span>
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
            </div>
            <div class="forms-list-mobile-cards administration-roles-mobile-cards">
                {if card_roles.is_empty() {
                    view! { <p class="forms-list-mobile-empty">"No Roles to Display"</p> }.into_any()
                } else {
                    card_roles
                        .into_iter()
                        .map(|role| {
                            let role_id = role.id.clone();
                            view! {
                                <article class="forms-list-mobile-card administration-role-mobile-card">
                                    <div class="forms-list-mobile-card__header">
                                        <div>
                                            <h3>{role.name}</h3>
                                        </div>
                                    </div>
                                    <dl>
                                        <div>
                                            <dt>"Capabilities"</dt>
                                            <dd>{role.capability_count}</dd>
                                        </div>
                                        <div>
                                            <dt>"Users"</dt>
                                            <dd>{role.account_count}</dd>
                                        </div>
                                    </dl>
                                    <div class="workflow-assignment-mobile-card__actions">
                                        <button
                                            class="button button--compact"
                                            type="button"
                                            on:click=move |_| selected_role_id.set(Some(role_id.clone()))
                                        >
                                            "View Details"
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
    }
}

#[component]
fn AdministrationRoleDetailPanel(
    detail: Option<AdminRoleDetail>,
    is_loading: bool,
    on_edit: impl Fn(leptos::ev::MouseEvent) + 'static + Copy,
) -> impl IntoView {
    if is_loading {
        view! {
            <section class="organization-state" aria-live="polite">
                <h3>"Loading role"</h3>
                <p>"Fetching role details."</p>
            </section>
        }
        .into_any()
    } else if let Some(detail) = detail {
        let capabilities = detail.capabilities.clone();
        let accounts = detail.assigned_accounts.clone();
        view! {
            <section class="organization-detail-card organization-detail-card--wide administration-role-detail-card">
                <div class="organization-detail-card__header">
                    <h2>{detail.name}</h2>
                    <button class="button button--secondary" type="button" on:click=on_edit>
                        "Edit Capabilities"
                    </button>
                </div>
                <table class="info-list-table">
                    <tbody>
                        <tr>
                            <th scope="row">"Capabilities"</th>
                            <td>{detail.capabilities.len()}</td>
                        </tr>
                        <tr>
                            <th scope="row">"Assigned Users"</th>
                            <td>{detail.assigned_accounts.len()}</td>
                        </tr>
                    </tbody>
                </table>
                <div class="administration-role-detail-grid">
                    <section class="organization-detail-card">
                        <h3>"Capabilities"</h3>
                        <AdminRoleCapabilityList capabilities/>
                    </section>
                    <section class="organization-detail-card">
                        <h3>"Assigned Users"</h3>
                        <AdminRoleAssignedAccounts accounts/>
                    </section>
                </div>
            </section>
        }
        .into_any()
    } else {
        view! {
            <section class="organization-state">
                <h3>"Select a role"</h3>
                <p>"Choose a role to review its capabilities and assigned users."</p>
            </section>
        }
        .into_any()
    }
}

#[component]
fn AdminRoleCapabilityList(capabilities: Vec<AdminCapabilitySummary>) -> impl IntoView {
    if capabilities.is_empty() {
        view! { <p class="muted">"No capabilities assigned."</p> }.into_any()
    } else {
        view! {
            <div class="capability-list">
                {capabilities
                    .into_iter()
                    .map(|capability| view! {
                        <div class="capability-list__item">
                            <strong>{capability.key}</strong>
                            <small>{capability.description}</small>
                        </div>
                    })
                    .collect_view()}
            </div>
        }
        .into_any()
    }
}

#[component]
fn AdminRoleAssignedAccounts(accounts: Vec<AdminAccountAssignmentSummary>) -> impl IntoView {
    if accounts.is_empty() {
        view! { <p class="muted">"No users assigned."</p> }.into_any()
    } else {
        view! {
            <div class="capability-list">
                {accounts
                    .into_iter()
                    .map(|account| view! {
                        <div class="capability-list__item">
                            <strong>{account.display_name}</strong>
                            <small>{account.email}</small>
                        </div>
                    })
                    .collect_view()}
            </div>
        }
        .into_any()
    }
}

#[component]
fn AdminRoleSheet(
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
                                    let visible = capabilities
                                        .get()
                                        .into_iter()
                                        .filter(|capability| {
                                            text_matches(&query, &[capability.key.as_str(), capability.description.as_str()])
                                        })
                                        .collect::<Vec<_>>();
                                    if visible.is_empty() {
                                        view! { <p class="forms-list-mobile-empty">"No Capabilities to Display"</p> }.into_any()
                                    } else {
                                        visible
                                            .into_iter()
                                            .map(|capability| {
                                                let capability_id = capability.id.clone();
                                                let checked = selected.iter().any(|id| id == &capability.id);
                                                view! {
                                                    <label class="checkbox-list__item permission-picker__item">
                                                        <input
                                                            type="checkbox"
                                                            prop:checked=checked
                                                            on:change=move |event| {
                                                                toggle_string_selection(
                                                                    selected_capability_ids,
                                                                    capability_id.clone(),
                                                                    event_target_checked(&event),
                                                                );
                                                            }
                                                        />
                                                        <span>
                                                            <strong>{capability.key}</strong>
                                                            <small>{capability.description}</small>
                                                        </span>
                                                    </label>
                                                }
                                            })
                                            .collect_view()
                                            .into_any()
                                    }
                                }}
                            </div>
                            <Show when=move || message.get().is_some()>
                                <p class="form-message" role="status">{move || message.get().unwrap_or_default()}</p>
                            </Show>
                        </section>
                        <div class="form-actions">
                            <button class="button button--secondary" type="button" on:click=on_close>
                                "Cancel"
                            </button>
                            <button class="button" type="button" disabled=move || is_saving.get() on:click=on_save>
                                {move || if is_saving.get() { "Saving..." } else { "Save Role" }}
                            </button>
                        </div>
                    </aside>
                </section>
            </Show>
        </Portal>
    }
}

fn load_admin_roles_context(
    roles: RwSignal<Vec<AdminRoleSummary>>,
    capabilities: RwSignal<Vec<AdminCapabilitySummary>>,
    selected_role_id: RwSignal<Option<String>>,
    is_loading: RwSignal<bool>,
    message: RwSignal<Option<String>>,
    preferred_role_id: Option<String>,
) {
    #[cfg(feature = "hydrate")]
    {
        leptos::task::spawn_local(async move {
            is_loading.set(true);
            message.set(None);
            let roles_response = gloo_net::http::Request::get("/api/admin/roles")
                .send()
                .await;
            let capabilities_response = gloo_net::http::Request::get("/api/admin/capabilities")
                .send()
                .await;

            match (roles_response, capabilities_response) {
                (Ok(roles_response), _) if roles_response.status() == 401 => {
                    is_loading.set(false);
                    redirect_to_login();
                }
                (Ok(roles_response), Ok(capabilities_response))
                    if roles_response.ok() && capabilities_response.ok() =>
                {
                    let loaded_roles = roles_response.json::<Vec<AdminRoleSummary>>().await;
                    let loaded_capabilities = capabilities_response
                        .json::<Vec<AdminCapabilitySummary>>()
                        .await;
                    match (loaded_roles, loaded_capabilities) {
                        (Ok(role_items), Ok(capability_items)) => {
                            let selected = preferred_role_id
                                .or_else(|| {
                                    selected_role_id
                                        .get_untracked()
                                        .filter(|id| role_items.iter().any(|role| role.id == *id))
                                })
                                .or_else(|| role_items.first().map(|role| role.id.clone()));
                            roles.set(role_items);
                            capabilities.set(capability_items);
                            selected_role_id.set(selected);
                            message.set(None);
                        }
                        (Err(error), _) => {
                            roles.set(Vec::new());
                            message.set(Some(format!("Unable to parse roles: {error}")));
                        }
                        (_, Err(error)) => {
                            capabilities.set(Vec::new());
                            message.set(Some(format!("Unable to parse capabilities: {error}")));
                        }
                    }
                    is_loading.set(false);
                }
                (Ok(roles_response), _) if !roles_response.ok() => {
                    roles.set(Vec::new());
                    message.set(Some(format!(
                        "Unable to load roles. Server returned {}.",
                        roles_response.status()
                    )));
                    is_loading.set(false);
                }
                (_, Ok(capabilities_response)) if !capabilities_response.ok() => {
                    capabilities.set(Vec::new());
                    message.set(Some(format!(
                        "Unable to load capabilities. Server returned {}.",
                        capabilities_response.status()
                    )));
                    is_loading.set(false);
                }
                (Err(error), _) => {
                    roles.set(Vec::new());
                    message.set(Some(format!("Unable to load roles: {error}")));
                    is_loading.set(false);
                }
                (_, Err(error)) => {
                    capabilities.set(Vec::new());
                    message.set(Some(format!("Unable to load capabilities: {error}")));
                    is_loading.set(false);
                }
                _ => {
                    message.set(Some("Unable to load role context.".into()));
                    is_loading.set(false);
                }
            }
        });
    }
    #[cfg(not(feature = "hydrate"))]
    {
        let _ = (
            roles,
            capabilities,
            selected_role_id,
            message,
            preferred_role_id,
        );
        is_loading.set(false);
    }
}

fn load_admin_role_detail(
    role_id: String,
    detail: RwSignal<Option<AdminRoleDetail>>,
    is_loading: RwSignal<bool>,
    message: RwSignal<Option<String>>,
) {
    #[cfg(feature = "hydrate")]
    {
        leptos::task::spawn_local(async move {
            is_loading.set(true);
            match gloo_net::http::Request::get(&format!("/api/admin/roles/{role_id}"))
                .send()
                .await
            {
                Ok(response) if response.status() == 401 => {
                    is_loading.set(false);
                    redirect_to_login();
                }
                Ok(response) if response.ok() => match response.json::<AdminRoleDetail>().await {
                    Ok(loaded_detail) => {
                        detail.set(Some(loaded_detail));
                        message.set(None);
                        is_loading.set(false);
                    }
                    Err(error) => {
                        detail.set(None);
                        message.set(Some(format!("Unable to parse role detail: {error}")));
                        is_loading.set(false);
                    }
                },
                Ok(response) => {
                    detail.set(None);
                    message.set(Some(format!(
                        "Unable to load role detail. Server returned {}.",
                        response.status()
                    )));
                    is_loading.set(false);
                }
                Err(error) => {
                    detail.set(None);
                    message.set(Some(format!("Unable to load role detail: {error}")));
                    is_loading.set(false);
                }
            }
        });
    }
    #[cfg(not(feature = "hydrate"))]
    {
        let _ = (role_id, detail, message);
        is_loading.set(false);
    }
}

#[allow(clippy::too_many_arguments)]
fn save_admin_role(
    editing_role_id: RwSignal<Option<String>>,
    role_name: RwSignal<String>,
    selected_capability_ids: RwSignal<Vec<String>>,
    is_saving: RwSignal<bool>,
    message: RwSignal<Option<String>>,
    sheet_open: RwSignal<bool>,
    roles: RwSignal<Vec<AdminRoleSummary>>,
    capabilities: RwSignal<Vec<AdminCapabilitySummary>>,
    selected_role_id: RwSignal<Option<String>>,
    selected_role_detail: RwSignal<Option<AdminRoleDetail>>,
    detail_loading: RwSignal<bool>,
) {
    #[cfg(feature = "hydrate")]
    {
        leptos::task::spawn_local(async move {
            is_saving.set(true);
            message.set(None);
            let capability_ids = selected_capability_ids.get();
            let editing_id = editing_role_id.get_untracked();
            let result = if let Some(role_id) = editing_id.clone() {
                let payload = UpdateAdminRolePayload { capability_ids };
                match serde_json::to_string(&payload) {
                    Ok(body) => {
                        send_json_id_request(
                            gloo_net::http::Request::put(&format!("/api/admin/roles/{role_id}")),
                            Some(body),
                            "Save role",
                        )
                        .await
                    }
                    Err(_) => Err("Role update could not be prepared.".into()),
                }
            } else {
                let name = role_name.get().trim().to_string();
                if name.is_empty() {
                    is_saving.set(false);
                    message.set(Some("Role name is required.".into()));
                    return;
                }
                let payload = CreateAdminRolePayload {
                    name,
                    capability_ids,
                };
                match serde_json::to_string(&payload) {
                    Ok(body) => {
                        send_json_id_request(
                            gloo_net::http::Request::post("/api/admin/roles"),
                            Some(body),
                            "Create role",
                        )
                        .await
                    }
                    Err(_) => Err("Role create request could not be prepared.".into()),
                }
            };

            match result {
                Ok(response) => {
                    let next_role_id = editing_id.unwrap_or(response.id);
                    sheet_open.set(false);
                    load_admin_roles_context(
                        roles,
                        capabilities,
                        selected_role_id,
                        is_saving,
                        message,
                        Some(next_role_id.clone()),
                    );
                    load_admin_role_detail(
                        next_role_id,
                        selected_role_detail,
                        detail_loading,
                        message,
                    );
                }
                Err(error) => {
                    message.set(Some(error));
                    is_saving.set(false);
                }
            }
        });
    }
    #[cfg(not(feature = "hydrate"))]
    let _ = (
        editing_role_id,
        role_name,
        selected_capability_ids,
        is_saving,
        message,
        sheet_open,
        roles,
        capabilities,
        selected_role_id,
        selected_role_detail,
        detail_loading,
    );
}
