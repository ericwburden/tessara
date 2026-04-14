//! Shared theme metadata and shell chrome for Tessara web documents.

use crate::pipeline;

pub const STORAGE_KEY: &str = "tessara.themePreference";
pub const LIGHT_THEME_COLOR: &str = "#F8FAFC";
pub const DARK_THEME_COLOR: &str = "#0F172A";

pub fn stylesheet_links() -> String {
    format!(r#"<link rel="stylesheet" href="{}">"#, pipeline::css_path())
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

pub fn control_html() -> &'static str {
    r#"<div class="theme-toggle" role="group" aria-label="Theme preference">
      <button class="theme-toggle-button" type="button" data-theme-choice="system" aria-pressed="true">System</button>
      <button class="theme-toggle-button" type="button" data-theme-choice="light" aria-pressed="false">Light</button>
      <button class="theme-toggle-button" type="button" data-theme-choice="dark" aria-pressed="false">Dark</button>
    </div>"#
}
