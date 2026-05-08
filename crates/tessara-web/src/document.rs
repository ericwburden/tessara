use leptos::context::Provider;
use leptos::prelude::*;
use leptos_router::location::RequestUrl;

use crate::{app, brand::document_head_tags, pipeline, theme};

pub fn render_native_app_document(title: &str, description: &str, _path: &str) -> String {
    let shell = Owner::new().with(|| {
        view! {
            <Provider value=RequestUrl::new(_path)>
                <app::App/>
            </Provider>
        }
        .to_html()
    });
    let brand = document_head_tags(title, description);
    let theme_bootstrap = theme::bootstrap_script();
    let stylesheets = theme::stylesheet_links();
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
