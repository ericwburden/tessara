//! Pagination math helpers for table-like views.
//!
//! Keep page count, current-page clamping, and row-range calculations here so feature tables share the same pagination semantics.

pub(crate) fn pagination_page_count(total_count: usize, page_size: usize) -> usize {
    if total_count == 0 {
        1
    } else {
        total_count.div_ceil(page_size)
    }
}

/// Clamps a requested page index to the available page range.
pub(crate) fn pagination_current_page(
    total_count: usize,
    page_size: usize,
    page_index: usize,
) -> usize {
    page_index.min(pagination_page_count(total_count, page_size) - 1)
}

/// Returns the zero-based row offset for the current page.
pub(crate) fn pagination_page_start(
    total_count: usize,
    page_size: usize,
    page_index: usize,
) -> usize {
    if total_count == 0 {
        0
    } else {
        pagination_current_page(total_count, page_size, page_index) * page_size
    }
}

/// Returns the exclusive row end offset for the current page.
pub(crate) fn pagination_page_end(
    total_count: usize,
    page_size: usize,
    page_index: usize,
) -> usize {
    (pagination_page_start(total_count, page_size, page_index) + page_size).min(total_count)
}
