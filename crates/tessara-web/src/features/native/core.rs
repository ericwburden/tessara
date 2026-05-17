use super::*;

#[component]
pub fn HomePage() -> impl IntoView {
    view! {
        <AppShell active_route="home" title="Home">
            <section class="route-panel">
                <PageHeader title="Native UI Route Inventory" description="Native Tessara routes are tracked here while each product area settles into its long-term implementation."/>
                <RouteMigrationOverview/>
            </section>
        </AppShell>
    }
}

#[component]
fn RouteMigrationOverview() -> impl IntoView {
    let route_rows = ROUTE_MIGRATIONS
        .iter()
        .map(|route| {
            view! {
                <tr>
                    <th scope="row">{route.name}</th>
                    <td><a href=route.href>{route.route}</a></td>
                    <td><StatusBadge label=route.status/></td>
                    <td><StatusBadge label=route.rbac_status/></td>
                </tr>
            }
        })
        .collect_view();

    let route_cards = ROUTE_MIGRATIONS
        .iter()
        .map(|route| {
            view! {
                <article class="route-migration-card">
                    <div class="route-migration-card__header">
                        <h3>{route.name}</h3>
                        <a href=route.href>{route.route}</a>
                    </div>
                    <dl class="route-migration-card__meta">
                        <div>
                            <dt>"Status"</dt>
                            <dd><StatusBadge label=route.status/></dd>
                        </div>
                        <div>
                            <dt>"RBAC"</dt>
                            <dd><StatusBadge label=route.rbac_status/></dd>
                        </div>
                    </dl>
                </article>
            }
        })
        .collect_view();

    view! {
        <div class="route-migration-overview">
            <div class="route-migration-overview__table">
                <DataTable>
                    <thead>
                        <tr>
                            <th scope="col">"Name"</th>
                            <th scope="col">"Route"</th>
                            <th scope="col">"Status"</th>
                            <th scope="col">"RBAC"</th>
                        </tr>
                    </thead>
                    <tbody>{route_rows}</tbody>
                </DataTable>
            </div>
            <div class="route-migration-overview__cards" aria-label="Route migration status">
                {route_cards}
            </div>
        </div>
    }
}

#[component]
pub fn LoginPage() -> impl IntoView {
    let email = RwSignal::new(String::new());
    let password = RwSignal::new(String::new());
    let is_submitting = RwSignal::new(false);
    let error_message = RwSignal::new(None::<String>);

    let submit = move |event: leptos::ev::SubmitEvent| {
        event.prevent_default();
        error_message.set(None);
        is_submitting.set(true);
        submit_login(
            email.get_untracked(),
            password.get_untracked(),
            error_message,
            is_submitting,
        );
    };

    view! {
        <main class="login-shell">
            <section class="login-panel blurred-surface" aria-labelledby="login-title">
                <a class="login-brand" href="/" aria-label="Tessara home">
                    <img src="/assets/tessara-icon-256.svg" alt=""/>
                    <span>"Tessara"</span>
                </a>
                <div class="login-panel__header">
                    <h1 id="login-title">"Welcome back"</h1>
                    <p>"Sign in to continue to the Tessara workspace."</p>
                </div>
                <form class="login-form" on:submit=submit>
                    <label class="login-field">
                        <span class="login-field__label">"Email"</span>
                        <span class="login-input-shell">
                            <Mail class="login-field__icon"/>
                            <input
                                type="email"
                                autocomplete="username"
                                placeholder="admin@tessara.local"
                                required
                                prop:value=move || email.get()
                                on:input=move |event| email.set(event_target_value(&event))
                            />
                        </span>
                    </label>
                    <label class="login-field">
                        <span class="login-field__label">"Password"</span>
                        <span class="login-input-shell">
                            <LockKeyhole class="login-field__icon"/>
                            <input
                                type="password"
                                autocomplete="current-password"
                                placeholder="Password"
                                required
                                prop:value=move || password.get()
                                on:input=move |event| password.set(event_target_value(&event))
                            />
                        </span>
                    </label>
                    <Show when=move || error_message.get().is_some()>
                        <p class="login-error" role="alert">
                            {move || error_message.get().unwrap_or_default()}
                        </p>
                    </Show>
                    <button class="button login-submit" type="submit" disabled=move || is_submitting.get()>
                        {move || if is_submitting.get() { "Signing in..." } else { "Sign In" }}
                    </button>
                </form>
            </section>
        </main>
    }
}

fn submit_login(
    email: String,
    password: String,
    error_message: RwSignal<Option<String>>,
    is_submitting: RwSignal<bool>,
) {
    #[cfg(feature = "hydrate")]
    {
        leptos::task::spawn_local(async move {
            let body = serde_json::json!({
                "email": email,
                "password": password,
            })
            .to_string();

            let response = match gloo_net::http::Request::post("/api/auth/login")
                .header("Content-Type", "application/json")
                .body(body)
            {
                Ok(request) => request.send().await,
                Err(error) => Err(error),
            };

            match response {
                Ok(response) if response.ok() => {
                    if let Some(window) = web_sys::window() {
                        let _ = window.location().set_href("/");
                    }
                }
                Ok(_) => {
                    error_message.set(Some("Email or password did not match.".into()));
                    is_submitting.set(false);
                }
                Err(_) => {
                    error_message.set(Some("Could not reach Tessara. Try again.".into()));
                    is_submitting.set(false);
                }
            }
        });
    }

    #[cfg(not(feature = "hydrate"))]
    {
        let _ = (email, password, error_message, is_submitting);
    }
}
