//! Forms-local pagination math helpers.

fn pagination_page_count(total_count: usize, page_size: usize) -> usize {
    if total_count == 0 {
        1
    } else {
        total_count.div_ceil(page_size)
    }
}

fn pagination_current_page(total_count: usize, page_size: usize, page_index: usize) -> usize {
    page_index.min(pagination_page_count(total_count, page_size) - 1)
}

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
