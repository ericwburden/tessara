//! Shared theme metadata and shell chrome for Tessara web documents.

use crate::pipeline;

pub const STORAGE_KEY: &str = "tessara.themePreference";
pub const LIGHT_THEME_COLOR: &str = "#F8FAFC";
pub const DARK_THEME_COLOR: &str = "#0F172A";
const DM_SANS_URL: &str = "https://fonts.googleapis.com/css2?family=DM+Sans:opsz,wght@9..40,500;9..40,650;9..40,750&display=swap";

pub fn stylesheet_links() -> String {
    format!(
        "<link rel=\"preconnect\" href=\"https://fonts.googleapis.com\">\
<link rel=\"preconnect\" href=\"https://fonts.gstatic.com\" crossorigin>\
<link rel=\"stylesheet\" href=\"{DM_SANS_URL}\">\
<link rel=\"stylesheet\" href=\"{}\">",
        pipeline::css_path()
    )
}

pub fn bootstrap_script() -> String {
    format!(
        r#"(function() {{
  const storageKey = "{STORAGE_KEY}";
  const root = document.documentElement;
  const metaThemeColor = document.querySelector('meta[name="theme-color"]');

  function systemTheme() {{
    return window.matchMedia && window.matchMedia('(prefers-color-scheme: dark)').matches
      ? 'dark'
      : 'light';
  }}

  let preference = 'system';
  try {{
    const stored = window.localStorage.getItem(storageKey);
    if (stored === 'light' || stored === 'dark' || stored === 'system') {{
      preference = stored;
    }}
  }} catch (_error) {{
    preference = 'system';
  }}

  const theme = preference === 'system' ? systemTheme() : preference;
  root.dataset.themePreference = preference;
  root.dataset.theme = theme;

  if (metaThemeColor) {{
    metaThemeColor.setAttribute('content', theme === 'dark' ? '{DARK_THEME_COLOR}' : '{LIGHT_THEME_COLOR}');
  }}
}})();"#,
    )
}
