pub(crate) const FORM_BUILDER_COLUMN_COUNT: i32 = 12;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct FormBuilderDragPreview {
    pub(crate) field_id: usize,
    pub(crate) section_id: usize,
    pub(crate) row: i32,
    pub(crate) column: i32,
}

#[derive(Clone, Copy)]
pub(crate) enum FormBuilderResizeAxis {
    Width,
    Height,
}
