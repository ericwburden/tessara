//! Core-owned routes whose title and content are supplied by Scoped Records.

use leptos::prelude::*;
use leptos_router::components::Route;
use leptos_router::{MatchNestedRoutes, path};

use crate::features::scoped_records::ScopedRecordsPage;
use crate::routes::PRIMARY_SSR_MODE;

pub fn scoped_records_routes() -> impl MatchNestedRoutes + Clone {
    view! {
        <>
            <Route path=path!("/reference/scoped-records") view=ScopedRecordsPage ssr=PRIMARY_SSR_MODE/>
            <Route path=path!("/reference/scoped-records/records/new") view=ScopedRecordsPage ssr=PRIMARY_SSR_MODE/>
            <Route path=path!("/reference/scoped-records/records/:record_id") view=ScopedRecordsPage ssr=PRIMARY_SSR_MODE/>
            <Route path=path!("/reference/scoped-records/records/:record_id/edit") view=ScopedRecordsPage ssr=PRIMARY_SSR_MODE/>
            <Route path=path!("/reference/scoped-records/health") view=ScopedRecordsPage ssr=PRIMARY_SSR_MODE/>
            <Route path=path!("/reference/scoped-records/diagnostics") view=ScopedRecordsPage ssr=PRIMARY_SSR_MODE/>
        </>
    }
}
