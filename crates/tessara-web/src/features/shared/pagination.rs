//! Pagination helper compatibility exports.
//!
//! Pagination helpers are implemented in `crate::utils::pagination` and reused by
//! shared feature modules. This module preserves the shared boundary while keeping
//! the implementation in one utility location.

pub(crate) use crate::utils::pagination::{
    pagination_current_page,
    pagination_page_count,
    pagination_page_end,
    pagination_page_start,
};

