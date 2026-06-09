//! Pagination helpers.

pub fn clamp_page(current: usize, total: usize) -> usize {
    if total == 0 || current < total {
        current
    } else {
        total - 1
    }
}

pub fn page_bounds(page: usize, page_size: usize, total_items: usize) -> (usize, usize) {
    let start = page.saturating_mul(page_size);
    let end = (start.saturating_add(page_size)).min(total_items);
    (start, end)
}
