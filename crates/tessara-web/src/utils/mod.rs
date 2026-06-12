//! Small cross-feature utility registry.
//!
//! Only expose general formatting, metadata, pagination, and text helpers here; domain-specific helpers should live with their owning feature.

pub mod metadata;
pub mod pagination;
pub mod slug;
pub mod text;
pub mod url;
