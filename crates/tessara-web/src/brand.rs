//! Brand assets and document metadata for Tessara web shells.

/// Route prefix used for SVG brand assets served by the API crate.
pub const ASSET_PREFIX: &str = "/assets";

/// Returns the HTML tags that connect Tessara icons and social previews to a document head.
pub(crate) fn document_head_tags(title: &str, description: &str) -> String {
    format!(
        r##"<meta name="description" content="{description}">
    <meta name="theme-color" content="#F8FAFC">
    <meta name="color-scheme" content="light">
    <meta property="og:type" content="website">
    <meta property="og:title" content="{title}">
    <meta property="og:description" content="{description}">
    <meta property="og:image" content="{ASSET_PREFIX}/tessara-icon-512.svg">
    <meta name="twitter:card" content="summary">
    <meta name="twitter:title" content="{title}">
    <meta name="twitter:description" content="{description}">
    <meta name="twitter:image" content="{ASSET_PREFIX}/tessara-icon-512.svg">
    <link rel="icon" type="image/svg+xml" sizes="16x16" href="{ASSET_PREFIX}/tessara-favicon-16.svg">
    <link rel="icon" type="image/svg+xml" sizes="32x32" href="{ASSET_PREFIX}/tessara-favicon-32.svg">
    <link rel="icon" type="image/svg+xml" sizes="64x64" href="{ASSET_PREFIX}/tessara-favicon-64.svg">
    <link rel="mask-icon" href="{ASSET_PREFIX}/tessara-favicon-mono.svg" color="#0F172A">
    <link rel="apple-touch-icon" href="{ASSET_PREFIX}/tessara-icon-256.svg">"##
    )
}

/// Returns an embedded SVG asset by public asset filename.
pub fn svg_asset(name: &str) -> Option<&'static str> {
    match name {
        "tessara-favicon-16.svg" => Some(include_str!("../assets/tessara-favicon-16.svg")),
        "tessara-favicon-32.svg" => Some(include_str!("../assets/tessara-favicon-32.svg")),
        "tessara-favicon-64.svg" => Some(include_str!("../assets/tessara-favicon-64.svg")),
        "tessara-favicon-mono.svg" => Some(include_str!("../assets/tessara-favicon-mono.svg")),
        "tessara-icon-256.svg" => Some(include_str!("../assets/tessara-icon-256.svg")),
        "tessara-icon-512.svg" => Some(include_str!("../assets/tessara-icon-512.svg")),
        "tessara-icon-1024.svg" => Some(include_str!("../assets/tessara-icon-1024.svg")),
        "tessara-wordmark.svg" => Some(include_str!("../assets/tessara-wordmark.svg")),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::{document_head_tags, svg_asset};

    #[test]
    fn document_head_exposes_brand_assets() {
        let head = document_head_tags("Tessara", "Test description");

        assert!(head.contains("tessara-favicon-16.svg"));
        assert!(head.contains("tessara-favicon-32.svg"));
        assert!(head.contains("tessara-favicon-64.svg"));
        assert!(head.contains("tessara-favicon-mono.svg"));
        assert!(head.contains("tessara-icon-256.svg"));
        assert!(head.contains("tessara-icon-512.svg"));
        assert!(head.contains("theme-color"));
        assert!(head.contains("#F8FAFC"));
    }

    #[test]
    fn svg_asset_lookup_serves_expected_assets() {
        assert!(svg_asset("tessara-favicon-32.svg").is_some());
        assert!(svg_asset("tessara-wordmark.svg").is_some());
        assert!(svg_asset("missing.svg").is_none());
    }
}
