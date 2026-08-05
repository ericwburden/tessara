//! Related work views for organization nodes.
//!
//! Keep tables and pagination for Core-owned work linked to organization nodes here.

use super::related_work_tables::{RelatedFormsTable, RelatedResponsesTable};
use crate::types::OrganizationNodeDetail;
use leptos::prelude::*;
use tessara_module_ui::{Tabs, TabsContent, TabsList, TabsTrigger};

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
                </TabsList>
                <TabsContent active=active_tab value="forms">
                    <RelatedFormsTable forms=detail.related_forms/>
                </TabsContent>
                <TabsContent active=active_tab value="responses">
                    <RelatedResponsesTable responses=detail.related_responses/>
                </TabsContent>
            </Tabs>
        </div>
    }
}
