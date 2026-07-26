//! Core shell integration for the independently deployed Scoped Records module.

use leptos::prelude::*;
use tessara_module_contract::ShellContentV1;

use crate::ui::AppShell;

#[component]
pub fn ScopedRecordsPage() -> impl IntoView {
    let content = RwSignal::new(use_context::<ShellContentV1>());

    #[cfg(all(feature = "hydrate", target_arch = "wasm32"))]
    {
        let location = leptos_router::hooks::use_location();
        Effect::new(move |_| {
            let path = location.pathname.get();
            let search = location.search.get();
            leptos::task::spawn_local(async move {
                let suffix = path
                    .strip_prefix("/reference/scoped-records")
                    .unwrap_or_default()
                    .trim_start_matches('/');
                let endpoint = if suffix.is_empty() {
                    format!("/api/reference/scoped-records/content{search}")
                } else {
                    format!("/api/reference/scoped-records/content/{suffix}")
                };
                if let Ok(payload) =
                    tessara_web_http::fetch_json::<ShellContentV1>(&endpoint, "Scoped Records")
                        .await
                {
                    content.set(Some(payload));
                }
            });
        });
    }

    view! {
        {move || match content.get() {
            Some(content) => {
                let title = content.title.clone();
                view! {
                    <AppShell active_route="scoped_records" title=title>
                        <section class="route-panel scoped-records-page">
                            <div class="scoped-records-module-content" inner_html=content.body_html></div>
                        </section>
                    </AppShell>
                }.into_any()
            }
            None => view! {
                <AppShell active_route="scoped_records" title="Scoped Records">
                    <section class="route-panel scoped-records-page" aria-busy="true">
                        <div class="organization-detail-card empty-state">
                            <h1>"Loading Scoped Records"</h1>
                            <p>"Requesting module content through Core."</p>
                        </div>
                    </section>
                </AppShell>
            }.into_any(),
        }}
    }
}
