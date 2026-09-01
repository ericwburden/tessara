//! Related work views for organization nodes.
//!
//! Keep tables and pagination for Core-owned work linked to organization nodes here.

use super::related_work_tables::RelatedFormsTable;
use crate::types::OrganizationNodeDetail;
use leptos::prelude::*;

#[component]
pub(crate) fn RelatedWorkSummary(
    detail: OrganizationNodeDetail,
    #[prop(optional)] cards_only: bool,
) -> impl IntoView {
    let summary_class = if cards_only {
        "related-work-summary related-work-summary--cards-only"
    } else {
        "related-work-summary"
    };
    view! {
        <div class=summary_class>
            <RelatedFormsTable forms=detail.related_forms/>
        </div>
    }
}
