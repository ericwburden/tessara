//! Owns the features::shared::types module behavior.

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct FormAttachmentLink {
    pub(crate) href: String,
    pub(crate) label: String,
    pub(crate) title: String,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct FormsAttachedNodesSheetData {
    pub(crate) form_name: String,
    pub(crate) form_href: String,
    pub(crate) nodes: Vec<FormAttachmentLink>,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct WorkflowAssignedUsersSheetData {
    pub(crate) workflow_name: String,
    pub(crate) workflow_href: String,
    pub(crate) users: Vec<FormAttachmentLink>,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct WorkflowAvailableNodesSheetData {
    pub(crate) workflow_name: String,
    pub(crate) workflow_href: String,
    pub(crate) nodes: Vec<FormAttachmentLink>,
}
