//! Public boundary for the Login feature.
//!
//! Re-export only the pages, types, and helpers other modules need; keep Login-specific implementation details in child modules.

mod pages;

pub(crate) use pages::LoginPage;
