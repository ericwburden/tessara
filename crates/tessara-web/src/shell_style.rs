//! CSS for the local Tessara shell.

/// Styles applied to the local shell document.
pub const STYLE: &str = r#"
      :root {
        color-scheme: light dark;
        font-family: Inter, ui-sans-serif, system-ui, -apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif;
      }
      body {
        margin: 0;
        background: #111827;
        color: #f9fafb;
      }
      main {
        max-width: 960px;
        margin: 0 auto;
        padding: 48px 24px;
      }
      .shell {
        display: grid;
        gap: 24px;
      }
      .app-shell {
        max-width: 1200px;
      }
      .app-layout {
        display: grid;
        gap: 24px;
        grid-template-columns: minmax(260px, 320px) minmax(0, 1fr);
      }
      .app-main {
        display: grid;
        gap: 16px;
      }
      .app-screen {
        border: 1px solid #374151;
        border-radius: 12px;
        background: #111827;
        padding: 16px;
      }
      .app-sidebar {
        align-self: start;
        display: grid;
        gap: 16px;
      }
      .app-nav {
        display: grid;
        gap: 8px;
      }
      .app-nav a, .button-link {
        border: 1px solid #38bdf8;
        border-radius: 999px;
        color: #bae6fd;
        display: inline-block;
        font-weight: 700;
        padding: 10px 16px;
        text-decoration: none;
      }
      .selection-panel {
        display: grid;
        gap: 8px;
      }
      .eyebrow {
        color: #38bdf8;
        font-size: 0.75rem;
        font-weight: 800;
        letter-spacing: 0.08em;
        text-transform: uppercase;
      }
      .panel {
        border: 1px solid #374151;
        border-radius: 16px;
        background: #1f2937;
        padding: 24px;
      }
      .hero .workflow-section {
        border: 0;
        background: transparent;
        padding: 0;
      }
      .workflow-grid {
        display: grid;
        gap: 16px;
        grid-template-columns: repeat(auto-fit, minmax(320px, 1fr));
        margin-top: 16px;
      }
      .workflow-section {
        border: 1px solid #374151;
        border-radius: 12px;
        background: #111827;
        padding: 16px;
      }
      .test-guide {
        display: grid;
        gap: 8px;
        margin: 16px 0 0;
        padding-left: 24px;
      }
      .selection-grid {
        display: grid;
        gap: 12px;
        grid-template-columns: repeat(auto-fit, minmax(220px, 1fr));
        margin-top: 16px;
      }
      .selection-item {
        border: 1px solid #374151;
        border-radius: 12px;
        background: #111827;
        padding: 12px;
      }
      .cards {
        display: grid;
        gap: 12px;
        grid-template-columns: repeat(auto-fit, minmax(240px, 1fr));
      }
      .card {
        border: 1px solid #374151;
        border-radius: 12px;
        background: #111827;
        padding: 16px;
      }
      .form-screen {
        display: grid;
        gap: 16px;
        grid-column: 1 / -1;
      }
      .form-section {
        border-top: 1px solid #374151;
        padding-top: 12px;
      }
      .form-fields {
        display: grid;
        gap: 12px;
      }
      .form-field {
        display: grid;
        gap: 6px;
      }
      .form-actions {
        border-top: 1px solid #374151;
        padding-top: 16px;
      }
      .actions {
        display: flex;
        flex-wrap: wrap;
        gap: 12px;
        margin-top: 16px;
      }
      .inputs {
        display: grid;
        gap: 12px;
        margin-top: 16px;
      }
      label {
        display: grid;
        gap: 6px;
        color: #d1d5db;
        font-size: 0.875rem;
        font-weight: 700;
      }
      button {
        border: 0;
        border-radius: 999px;
        background: #38bdf8;
        color: #082f49;
        cursor: pointer;
        font-weight: 700;
        padding: 10px 16px;
      }
      input, textarea {
        border: 1px solid #4b5563;
        border-radius: 12px;
        background: #111827;
        color: #f9fafb;
        padding: 10px 12px;
      }
      textarea {
        min-height: 160px;
      }
      pre {
        overflow: auto;
        border-radius: 12px;
        background: #030712;
        color: #d1d5db;
        padding: 16px;
      }
      .muted {
        color: #9ca3af;
      }
"#;
