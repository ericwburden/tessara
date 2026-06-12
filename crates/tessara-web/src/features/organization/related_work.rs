//! Related work views for organization nodes.
//!
//! Keep tables and pagination for forms, responses, dashboards, and other work linked to organization nodes here.

use super::related_work_tables::{
    RelatedDashboardsTable, RelatedFormsTable, RelatedResponsesTable,
};
use crate::features::organization::types::OrganizationNodeDetail;
use crate::ui::{Tabs, TabsContent, TabsList, TabsTrigger};
use leptos::prelude::*;

#[component]
pub(crate) fn RelatedWorkSummary(
    detail: OrganizationNodeDetail,
    #[prop(optional)] cards_only: bool,
) -> impl IntoView {
    let active_tab = RwSignal::new("forms".to_string());
    let summary_class = if cards_only {
        "related-work-summary related-work-summary--cards-only"
    } else {
        "related-work-summary"
    };
    let forms_count = detail.related_forms.len();
    let responses_count = detail.related_responses.len();
    let dashboards_count = detail.related_dashboards.len();

    view! {
        <div class=summary_class>
            <Tabs active=active_tab>
                <TabsList>
                    <TabsTrigger active=active_tab value="forms">
                        {format!("Forms ({forms_count})")}
                    </TabsTrigger>
                    <TabsTrigger active=active_tab value="responses">
                        {format!("Responses ({responses_count})")}
                    </TabsTrigger>
                    <TabsTrigger active=active_tab value="dashboards">
                        {format!("Dashboards ({dashboards_count})")}
                    </TabsTrigger>
                </TabsList>
                <TabsContent active=active_tab value="forms">
                    <RelatedFormsTable forms=detail.related_forms/>
                </TabsContent>
                <TabsContent active=active_tab value="responses">
                    <RelatedResponsesTable responses=detail.related_responses/>
                </TabsContent>
                <TabsContent active=active_tab value="dashboards">
                    <RelatedDashboardsTable dashboards=detail.related_dashboards/>
                </TabsContent>
            </Tabs>
        </div>
    }
}
