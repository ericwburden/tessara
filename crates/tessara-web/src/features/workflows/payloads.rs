//! Owns the features::workflows::payloads module behavior.

use serde::Serialize;

#[derive(Serialize)]
#[cfg_attr(not(feature = "hydrate"), allow(dead_code))]
pub(crate) struct CreateWorkflowPayload {
    pub(crate) available_node_ids: Vec<String>,
    pub(crate) name: String,
    pub(crate) slug: String,
    pub(crate) description: Option<String>,
}

#[derive(Serialize)]
#[cfg_attr(not(feature = "hydrate"), allow(dead_code))]
pub(crate) struct UpdateWorkflowPayload {
    pub(crate) available_node_ids: Vec<String>,
    pub(crate) name: String,
    pub(crate) slug: String,
    pub(crate) description: Option<String>,
}

#[derive(Serialize)]
#[cfg_attr(not(feature = "hydrate"), allow(dead_code))]
pub(crate) struct CreateWorkflowRevisionPayload {
    pub(crate) steps: Vec<CreateWorkflowStepPayload>,
}

#[derive(Serialize)]
#[cfg_attr(not(feature = "hydrate"), allow(dead_code))]
pub(crate) struct UpdateWorkflowRevisionStepsPayload {
    pub(crate) steps: Vec<CreateWorkflowStepPayload>,
}

#[derive(Clone, Debug, Serialize)]
#[cfg_attr(not(feature = "hydrate"), allow(dead_code))]
pub(crate) struct CreateWorkflowStepPayload {
    pub(crate) title: String,
    pub(crate) form_version_id: String,
}
