//! Form-to-organization attachment views.

mod list;
mod related_table;
mod sheet;

pub(in crate::features::forms) use list::FormsAttachedNodesList;
pub(crate) use related_table::FormAttachedNodesRelatedTable;
pub(in crate::features::forms) use sheet::FormsAttachedNodesSheet;
