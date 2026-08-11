//! Theme bootstrap markup for native documents.
//!
//! Keep early stylesheet links and pre-hydration theme scripts here so theme state is applied before the interactive shell mounts.

use crate::pipeline;
pub(crate) use crate::state::theme::{DARK_THEME_COLOR, LIGHT_THEME_COLOR, STORAGE_KEY};

pub(crate) fn stylesheet_links() -> String {
    format!(
        "<link rel=\"stylesheet\" href=\"{}\">",
        pipeline::css_path()
    )
}

pub(crate) fn bootstrap_script() -> String {
    format!(
        r#"(function() {{
  const storageKey = "{STORAGE_KEY}";
  const root = document.documentElement;
  const metaThemeColor = document.querySelector('meta[name=\"theme-color\"]');

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

#[cfg(test)]
mod tests {
    use super::stylesheet_links;

    #[test]
    fn native_documents_load_only_repository_owned_stylesheets() {
        let links = stylesheet_links();
        assert!(links.contains("/pkg/tessara-web.css"));
        assert!(!links.contains("https://"));
        assert!(!links.contains("fonts.googleapis.com"));
        assert!(!links.contains("fonts.gstatic.com"));
    }
}
