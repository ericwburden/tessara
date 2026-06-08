use leptos::prelude::{AnyView, Fragment};

pub(crate) fn empty_view() -> AnyView {
    Fragment::new(Vec::<AnyView>::new()).into()
}

pub mod components;
