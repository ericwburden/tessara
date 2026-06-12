//! Feature-local reusable Administration components.

mod node_type_detail;
mod node_type_metadata_field_actions;
mod node_type_metadata_field_sheet;
mod node_type_metadata_field_table;
mod node_type_metadata_fields;
mod node_type_metadata_mobile_cards;
mod node_type_relationships;
mod node_type_selector;
mod node_types;
mod role_sheet;
mod roles;
mod user_access;
mod user_forms;
mod users;

pub(crate) use node_type_detail::NodeTypeDetailCollections;
pub(crate) use node_type_relationships::NodeTypeRelationshipPicker;
pub(crate) use node_type_selector::AdministrationNodeTypesList;
pub(crate) use node_types::AdministrationNodeTypeEditor;
pub(crate) use role_sheet::AdminRoleSheet;
pub(crate) use roles::{AdministrationRoleDetailPanel, AdministrationRolesList};
pub(crate) use user_access::{
    AdminCapabilityList, AdminDelegationChecklist, AdminDelegationList, AdminScopeNodeChecklist,
    AdminScopeNodeList,
};
pub(crate) use user_forms::{AdministrationUserAccessForm, AdministrationUserAccountForm};
pub(crate) use users::AdministrationUsersList;
