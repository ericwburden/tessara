//! Static document head and SVG asset lookup helpers.
//!
//! Keep favicon, stylesheet, preload, and embedded SVG lookup concerns here so native rendering has a single asset metadata source.

/// Route prefix used for static assets served by the API crate.
pub const ASSET_PREFIX: &str = "/assets";

#[cfg(any(feature = "ssr", test))]
pub(crate) struct StaticAsset {
    pub(crate) content: &'static str,
    pub(crate) content_type: &'static str,
}

/// Returns the HTML tags that connect Tessara icons and social previews to a document head.
pub(crate) fn document_head_tags(title: &str, description: &str) -> String {
    let asset_version = crate::pipeline::asset_version();
    format!(
        r##"<meta name="description" content="{description}">
    <meta name="theme-color" content="#F8FAFC">
    <meta name="color-scheme" content="light dark">
    <meta property="og:type" content="website">
    <meta property="og:title" content="{title}">
    <meta property="og:description" content="{description}">
    <meta property="og:image" content="{ASSET_PREFIX}/tessara-icon-512.svg">
    <meta name="twitter:card" content="summary">
    <meta name="twitter:title" content="{title}">
    <meta name="twitter:description" content="{description}">
    <meta name="twitter:image" content="{ASSET_PREFIX}/tessara-icon-512.svg">
    <link rel="stylesheet" href="https://cdnjs.cloudflare.com/ajax/libs/font-awesome/6.7.2/css/all.min.css">
    <link rel="icon" type="image/svg+xml" sizes="16x16" href="{ASSET_PREFIX}/tessara-favicon-16.svg">
    <link rel="icon" type="image/svg+xml" sizes="32x32" href="{ASSET_PREFIX}/tessara-favicon-32.svg">
    <link rel="icon" type="image/svg+xml" sizes="64x64" href="{ASSET_PREFIX}/tessara-favicon-64.svg">
    <link rel="mask-icon" href="{ASSET_PREFIX}/tessara-favicon-mono.svg" color="#0F172A">
    <link rel="apple-touch-icon" href="{ASSET_PREFIX}/tessara-icon-256.svg">
    <script src="{ASSET_PREFIX}/d3.v7.9.0.min.js" defer></script>
    <script src="{ASSET_PREFIX}/tessara-d3-charts.js?v={asset_version}" defer></script>"##
    )
}

#[cfg(any(feature = "ssr", test))]
pub(crate) fn static_asset(name: &str) -> Option<StaticAsset> {
    match name {
        "d3.v7.9.0.min.js" => Some(StaticAsset {
            content: include_str!("../../assets/d3.v7.9.0.min.js"),
            content_type: "application/javascript; charset=utf-8",
        }),
        "tessara-d3-charts.js" => Some(StaticAsset {
            content: include_str!("../../assets/tessara-d3-charts.js"),
            content_type: "application/javascript; charset=utf-8",
        }),
        _ => svg_asset(name).map(|content| StaticAsset {
            content,
            content_type: "image/svg+xml; charset=utf-8",
        }),
    }
}

/// Returns an embedded SVG asset by public asset filename.
pub(crate) fn svg_asset(name: &str) -> Option<&'static str> {
    match name {
        "tessara-favicon-16.svg" => Some(include_str!("../../assets/tessara-favicon-16.svg")),
        "tessara-favicon-32.svg" => Some(include_str!("../../assets/tessara-favicon-32.svg")),
        "tessara-favicon-64.svg" => Some(include_str!("../../assets/tessara-favicon-64.svg")),
        "tessara-favicon-mono.svg" => Some(include_str!("../../assets/tessara-favicon-mono.svg")),
        "tessara-icon-256.svg" => Some(include_str!("../../assets/tessara-icon-256.svg")),
        "tessara-icon-512.svg" => Some(include_str!("../../assets/tessara-icon-512.svg")),
        "tessara-icon-1024.svg" => Some(include_str!("../../assets/tessara-icon-1024.svg")),
        "tessara-wordmark.svg" => Some(include_str!("../../assets/tessara-wordmark.svg")),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::{document_head_tags, static_asset, svg_asset};

    #[test]
    /// Verifies the document head exposes brand assets behavior.
    fn document_head_exposes_brand_assets() {
        let head = document_head_tags("Tessara", "Test description");

        assert!(head.contains("tessara-favicon-16.svg"));
        assert!(head.contains("tessara-favicon-32.svg"));
        assert!(head.contains("tessara-favicon-64.svg"));
        assert!(head.contains("tessara-favicon-mono.svg"));
        assert!(head.contains("tessara-icon-256.svg"));
        assert!(head.contains("tessara-icon-512.svg"));
        assert!(head.contains("font-awesome"));
        assert!(head.contains("d3.v7.9.0.min.js"));
        assert!(head.contains("tessara-d3-charts.js"));
        assert!(head.contains(crate::pipeline::asset_version()));
        assert!(head.contains("theme-color"));
        assert!(head.contains("#F8FAFC"));
        assert!(head.contains("light dark"));
    }

    #[test]
    /// Verifies the svg asset lookup serves expected assets behavior.
    fn svg_asset_lookup_serves_expected_assets() {
        assert!(svg_asset("tessara-favicon-32.svg").is_some());
        assert!(svg_asset("tessara-wordmark.svg").is_some());
        assert!(svg_asset("missing.svg").is_none());
    }

    #[test]
    /// Verifies the static asset lookup serves non-SVG assets behavior.
    fn static_asset_lookup_serves_chart_renderer() {
        let d3 = static_asset("d3.v7.9.0.min.js").expect("pinned D3 asset");
        assert!(d3.content.contains("d3js.org v7.9.0"));

        let asset = static_asset("tessara-d3-charts.js").expect("chart renderer asset");

        assert_eq!(asset.content_type, "application/javascript; charset=utf-8");
        assert!(asset.content.contains("TessaraCharts"));
        assert!(static_asset("missing.js").is_none());
    }
}
