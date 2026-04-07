//! CSS for the local Tessara shell.

/// Styles applied to the local shell document.
pub const STYLE: &str = r#"
      :root {
        --ink: #0F172A;
        --slate-dark: #334155;
        --slate-mid: #64748B;
        --neutral: #E2E8F0;
        --light: #F8FAFC;
        --surface: #FFFFFF;
        --teal: #14B8A6;
        --teal-soft: #CCFBF1;
        --orange: #F59E0B;
        --orange-soft: #FEF3C7;
        --lime: #84CC16;
        --lime-soft: #ECFCCB;
        --shadow: 0 18px 45px rgb(15 23 42 / 0.10);
        color-scheme: light;
        font-family: Inter, ui-sans-serif, system-ui, -apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif;
      }
      * {
        box-sizing: border-box;
      }
      body {
        margin: 0;
        background:
          radial-gradient(circle at 18% 12%, rgb(20 184 166 / 0.14), transparent 28rem),
          radial-gradient(circle at 86% 4%, rgb(245 158 11 / 0.14), transparent 22rem),
          var(--light);
        color: var(--ink);
      }
      main {
        max-width: 980px;
        margin: 0 auto;
        padding: 48px 24px;
      }
      h1, h2, h3, h4, p {
        margin-top: 0;
      }
      h1 {
        font-size: clamp(2.4rem, 6vw, 4.6rem);
        letter-spacing: -0.06em;
        line-height: 0.95;
        margin-bottom: 18px;
      }
      h2 {
        letter-spacing: -0.035em;
      }
      code {
        color: var(--slate-dark);
        overflow-wrap: anywhere;
      }
      .shell {
        display: grid;
        gap: 24px;
      }
      .app-shell {
        max-width: 1220px;
      }
      .brand-lockup {
        align-items: center;
        color: var(--ink);
        display: inline-flex;
        font-size: 1.15rem;
        font-weight: 800;
        gap: 10px;
        letter-spacing: -0.02em;
        margin-bottom: 20px;
      }
      .brand-mark {
        border: 1px solid var(--neutral);
        border-radius: 14px;
        box-shadow: 0 8px 20px rgb(15 23 42 / 0.12);
        height: 42px;
        width: 42px;
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
        border: 1px solid var(--neutral);
        border-radius: 18px;
        background: rgb(255 255 255 / 0.88);
        padding: 18px;
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
        border: 1px solid var(--teal);
        border-radius: 999px;
        color: #0F766E;
        display: inline-block;
        font-weight: 800;
        padding: 10px 16px;
        text-decoration: none;
      }
      .app-nav a:hover, .button-link:hover {
        background: var(--teal-soft);
      }
      .selection-panel {
        display: grid;
        gap: 8px;
      }
      .eyebrow {
        color: #0F766E;
        font-size: 0.75rem;
        font-weight: 900;
        letter-spacing: 0.1em;
        text-transform: uppercase;
      }
      .panel {
        border: 1px solid var(--neutral);
        border-radius: 22px;
        background: rgb(255 255 255 / 0.92);
        box-shadow: var(--shadow);
        padding: 24px;
      }
      .hero {
        background:
          linear-gradient(135deg, rgb(255 255 255 / 0.96), rgb(248 250 252 / 0.88)),
          linear-gradient(90deg, var(--teal-soft), var(--orange-soft));
      }
      .hero .workflow-section {
        border: 0;
        background: transparent;
        box-shadow: none;
        padding: 0;
      }
      .workflow-grid {
        display: grid;
        gap: 16px;
        grid-template-columns: repeat(auto-fit, minmax(320px, 1fr));
        margin-top: 16px;
      }
      .workflow-section {
        border: 1px solid var(--neutral);
        border-radius: 18px;
        background: var(--surface);
        padding: 16px;
      }
      .workflow-section:nth-child(3n + 1) {
        border-top: 4px solid var(--teal);
      }
      .workflow-section:nth-child(3n + 2) {
        border-top: 4px solid var(--lime);
      }
      .workflow-section:nth-child(3n + 3) {
        border-top: 4px solid var(--orange);
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
        border: 1px solid var(--neutral);
        border-radius: 16px;
        background: var(--light);
        padding: 12px;
      }
      .cards {
        display: grid;
        gap: 12px;
        grid-template-columns: repeat(auto-fit, minmax(240px, 1fr));
      }
      .card {
        border: 1px solid var(--neutral);
        border-left: 4px solid var(--teal);
        border-radius: 16px;
        background: var(--surface);
        padding: 16px;
      }
      .form-screen {
        display: grid;
        gap: 16px;
        grid-column: 1 / -1;
      }
      .form-section {
        border-top: 1px solid var(--neutral);
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
        border-top: 1px solid var(--neutral);
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
        color: var(--slate-dark);
        font-size: 0.875rem;
        font-weight: 800;
      }
      button {
        border: 0;
        border-radius: 999px;
        background: var(--teal);
        color: #042F2E;
        cursor: pointer;
        font-weight: 800;
        padding: 10px 16px;
      }
      button:hover {
        background: #2DD4BF;
      }
      input, textarea {
        border: 1px solid var(--neutral);
        border-radius: 14px;
        background: var(--surface);
        color: var(--ink);
        padding: 10px 12px;
      }
      input:focus, textarea:focus, button:focus, a:focus {
        outline: 3px solid var(--orange);
        outline-offset: 2px;
      }
      textarea {
        min-height: 160px;
      }
      pre {
        overflow: auto;
        border: 1px solid var(--neutral);
        border-radius: 16px;
        background: var(--ink);
        color: var(--light);
        padding: 16px;
      }
      .table-wrap {
        overflow-x: auto;
      }
      table {
        border-collapse: collapse;
        min-width: 100%;
      }
      th, td {
        border-bottom: 1px solid var(--neutral);
        padding: 10px 12px;
        text-align: left;
        vertical-align: top;
      }
      th {
        color: var(--slate-dark);
        font-size: 0.78rem;
        letter-spacing: 0.08em;
        text-transform: uppercase;
      }
      .muted {
        color: var(--slate-mid);
      }
      @media (max-width: 820px) {
        main {
          padding: 24px 16px;
        }
        .app-layout {
          grid-template-columns: 1fr;
        }
      }
"#;
