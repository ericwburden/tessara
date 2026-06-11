//! Native HTML document assembly.
//!
//! This module owns the server-rendered document wrapper around the Leptos app root, overlay root, head tags, and hydration script tags.

use leptos::context::Provider;
use leptos::prelude::*;
use leptos_router::location::RequestUrl;

use crate::{app, pipeline};

/// Handles the render native app document behavior.
pub(crate) fn render_native_app_document(title: &str, description: &str, _path: &str) -> String {
    let shell = Owner::new().with(|| {
        view! {
            <Provider value=RequestUrl::new(_path)>
                <app::App/>
            </Provider>
        }
        .to_html()
    });
    let brand = crate::document::document_head_tags(title, description);
    let theme_bootstrap = crate::document::bootstrap_script();
    let stylesheets = crate::document::stylesheet_links();
    let hydration = pipeline::hydration_module_tag();

    format!(
        "<!doctype html>\n\
<html lang=\"en\" data-theme=\"light\" data-theme-preference=\"system\">\n\
  <head>\n\
    <meta charset=\"utf-8\">\n\
    <meta name=\"viewport\" content=\"width=device-width, initial-scale=1\">\n\
    <title>{title}</title>\n\
    {brand}\n\
    <script>{theme_bootstrap}</script>\n\
    {stylesheets}\n\
  </head>\n\
  <body class=\"tessara-app\">\n\
    <div id=\"app-overlays\"></div>\n\
    <div id=\"{app_root_id}\">{shell}</div>\n\
    {hydration}\n\
  </body>\n\
</html>",
        app_root_id = pipeline::APP_ROOT_ID,
    )
}
