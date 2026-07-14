//! Form-to-organization attachment views.

mod list;
mod related_table;
mod sheet;

const ATTACHED_NODES_SHEET_ID: &str = "forms-attached-nodes-sheet";

pub(crate) use list::FormsAttachedNodesList;
pub(crate) use related_table::FormAttachedNodesRelatedTable;
pub(crate) use sheet::FormsAttachedNodesSheet;
