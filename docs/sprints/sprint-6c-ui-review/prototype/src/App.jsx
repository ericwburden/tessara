import { useEffect, useState } from "react";
import {
  Activity,
  AlertTriangle,
  Bell,
  Blocks,
  ChartNoAxesCombined,
  Check,
  CheckCircle2,
  ChevronDown,
  ChevronRight,
  CircleHelp,
  Database,
  ExternalLink,
  FileText,
  GitBranch,
  Grid2X2,
  HeartPulse,
  Home,
  LayoutDashboard,
  ListChecks,
  LockKeyhole,
  LogOut,
  Menu,
  Moon,
  Pencil,
  RefreshCw,
  Search,
  Settings,
  ShieldAlert,
  ShieldCheck,
  Sun,
  Unplug,
  Users,
  X,
} from "lucide-react";

const reviewScreens = [
  { id: "module", label: "Module configuration", group: "Module Management" },
  { id: "diagnostics", label: "Health and diagnostics", group: "Module Management" },
  { id: "editor", label: "Placement degradation", group: "Dashboard editor" },
  { id: "viewer", label: "Contained viewer states", group: "Dashboard viewer" },
  { id: "unavailable", label: "Dashboard unavailable", group: "Core fallback" },
];

const placementStates = {
  healthy: {
    label: "Available",
    short: "Available",
    tone: "success",
    title: "Program Snapshot",
    description: "Showing 2 rows across 1 visible column.",
    detail: "ComponentVersion resolved through the transition compatibility contract.",
  },
  unauthorized: {
    label: "Restricted",
    short: "Restricted",
    tone: "danger",
    title: "Placement unavailable",
    description: "You do not have access to this placement.",
    detail: "The resource was not resolved, so no Component identity or lifecycle detail is disclosed.",
  },
  providerUnavailable: {
    label: "Components unavailable",
    short: "Provider unavailable",
    tone: "warning",
    title: "Component provider unavailable",
    description: "This placement cannot be rendered right now.",
    detail: "Dashboard remains available. Try again after the Components provider recovers.",
  },
  inactive: {
    label: "Inactive version",
    short: "Inactive",
    tone: "warning",
    title: "ComponentVersion is inactive",
    description: "The pinned version is retained but no longer active.",
    detail: "Choose a compatible active version in Placement details or retain the historical reference.",
  },
  superseded: {
    label: "Superseded version",
    short: "Superseded",
    tone: "info",
    title: "A newer ComponentVersion is available",
    description: "This Dashboard remains pinned to the version it was authored with.",
    detail: "Review the replacement before rebinding. Nothing changes automatically.",
  },
  tombstoned: {
    label: "Resource tombstoned",
    short: "Tombstoned",
    tone: "danger",
    title: "ComponentVersion was removed",
    description: "The provider retained a tombstone for this resource.",
    detail: "Replace or remove the placement. The Dashboard reference is preserved for diagnosis.",
  },
  missing: {
    label: "Missing resource",
    short: "Missing",
    tone: "danger",
    title: "ComponentVersion could not be found",
    description: "The authorized provider returned no matching resource.",
    detail: "Confirm the binding and deployment receipt, then replace or remove this placement.",
  },
  incompatible: {
    label: "Incompatible contract",
    short: "Incompatible",
    tone: "warning",
    title: "Component contract is incompatible",
    description: "The provider is reachable, but this Dashboard cannot use its contract version.",
    detail: "Review Module diagnostics before upgrading or rebinding the placement.",
  },
  notEvaluated: {
    label: "Not evaluated",
    short: "Not evaluated",
    tone: "neutral",
    title: "Placement was not evaluated",
    description: "No resolution decision is available for this request.",
    detail: "Refresh the Dashboard to request a current authorization and compatibility decision.",
  },
};

function hashScreen() {
  const value = window.location.hash.replace("#", "");
  return reviewScreens.some((item) => item.id === value) ? value : "module";
}

export function App() {
  const [screen, setScreen] = useState(hashScreen);
  const [reviewOpen, setReviewOpen] = useState(false);
  const [theme, setTheme] = useState("dark");
  const [placementState, setPlacementState] = useState("providerUnavailable");

  useEffect(() => {
    const sync = () => setScreen(hashScreen());
    window.addEventListener("hashchange", sync);
    return () => window.removeEventListener("hashchange", sync);
  }, []);

  function navigate(id) {
    window.location.hash = id;
    setScreen(id);
    setReviewOpen(false);
    window.scrollTo({ top: 0, behavior: "instant" });
  }

  const activeIndex = reviewScreens.findIndex((item) => item.id === screen);

  return (
    <div data-theme={theme}>
      <Shell screen={screen} navigate={navigate} theme={theme} setTheme={setTheme}>
        {screen === "module" && <ModuleConfiguration navigate={navigate} />}
        {screen === "diagnostics" && <Diagnostics />}
        {screen === "editor" && (
          <DashboardEditor state={placementState} setState={setPlacementState} navigate={navigate} />
        )}
        {screen === "viewer" && (
          <DashboardViewer state={placementState} setState={setPlacementState} navigate={navigate} />
        )}
        {screen === "unavailable" && <DashboardUnavailable navigate={navigate} />}
      </Shell>
      <ReviewNavigator
        active={reviewScreens[activeIndex]}
        activeIndex={activeIndex}
        isOpen={reviewOpen}
        setOpen={setReviewOpen}
        navigate={navigate}
      />
    </div>
  );
}

function ReviewNavigator({ active, activeIndex, isOpen, setOpen, navigate }) {
  return (
    <aside className={`review-nav ${isOpen ? "is-open" : ""}`} aria-label="Sprint 6C review navigation">
      <button className="review-nav__trigger" type="button" onClick={() => setOpen(!isOpen)}>
        <span>
          <small>Sprint 6C review</small>
          <strong>{active.label}</strong>
        </span>
        <span className="review-nav__count">{activeIndex + 1} / {reviewScreens.length}</span>
        <ChevronDown size={16} />
      </button>
      {isOpen && (
        <div className="review-nav__menu">
          <div className="review-nav__menu-header">
            <div>
              <strong>Review screens</strong>
              <small>Prototype-only navigator</small>
            </div>
            <button className="icon-button quiet" type="button" aria-label="Close review navigation" onClick={() => setOpen(false)}>
              <X size={16} />
            </button>
          </div>
          {reviewScreens.map((item, index) => (
            <button
              key={item.id}
              type="button"
              className={item.id === active.id ? "review-nav__item is-active" : "review-nav__item"}
              onClick={() => navigate(item.id)}
            >
              <span className="review-nav__number">{String(index + 1).padStart(2, "0")}</span>
              <span><strong>{item.label}</strong><small>{item.group}</small></span>
              {item.id === active.id && <Check size={16} />}
            </button>
          ))}
        </div>
      )}
    </aside>
  );
}

function Shell({ screen, navigate, theme, setTheme, children }) {
  const dashboardsActive = ["editor", "viewer", "unavailable"].includes(screen);
  const pageTitle = screen === "editor"
    ? "Edit Dashboard"
    : screen === "viewer"
      ? "Dashboard Viewer"
      : screen === "unavailable"
        ? "Dashboards"
        : "Module Management";

  return (
    <main className="app-shell">
      <aside className="sidebar" aria-label="Primary navigation">
        <a className="sidebar-brand" href="#module" onClick={(event) => { event.preventDefault(); navigate("module"); }}>
          <img src="/tessara-icon.svg" alt="" />
          <strong>Tessara</strong>
        </a>
        <nav className="sidebar-nav">
          <p className="sidebar-section">Main</p>
          <SidebarLink icon={Home} label="Home" />
          <SidebarLink icon={GitBranch} label="Organization" />
          <SidebarLink icon={FileText} label="Forms" />
          <SidebarLink icon={Grid2X2} label="Workflows" />
          <SidebarLink icon={CircleHelp} label="Responses" />
          <SidebarLink icon={ListChecks} label="Operations" />
          <SidebarLink icon={Database} label="Datasets" />
          <SidebarLink icon={Pencil} label="Components" />
          <SidebarLink icon={LayoutDashboard} label="Dashboards" active={dashboardsActive} onClick={() => navigate("viewer")} />
          <p className="sidebar-section">Admin</p>
          <SidebarLink icon={Users} label="User Management" />
          <SidebarLink icon={ShieldCheck} label="Roles & Access" />
          <SidebarLink icon={Settings} label="Node Types" />
          <SidebarLink icon={Blocks} label="Module Management" active={!dashboardsActive} onClick={() => navigate("module")} />
        </nav>
        <div className="account-card">
          <span className="avatar">TA</span>
          <span><strong>Tessara Admin</strong><small>ADMIN@TESSARA.LOCAL</small></span>
          <button className="icon-button" type="button" aria-label="Sign out"><LogOut size={18} /></button>
        </div>
      </aside>
      <section className="application-content">
        <header className="top-app-bar">
          <div className="title-row">
            <button className="icon-button mobile-menu" type="button" aria-label="Open menu"><Menu size={19} /></button>
            <h1>{pageTitle}</h1>
          </div>
          <div className="top-actions">
            <label className="global-search"><Search size={16} /><input type="search" placeholder="Search Tessara" /></label>
            <button className="icon-button accent" type="button" aria-label="Toggle theme" onClick={() => setTheme(theme === "dark" ? "light" : "dark")}>
              {theme === "dark" ? <Moon size={18} /> : <Sun size={18} />}
            </button>
            <button className="icon-button accent" type="button" aria-label="Notifications"><Bell size={18} /></button>
            <button className="icon-button accent" type="button" aria-label="Help"><CircleHelp size={18} /></button>
          </div>
        </header>
        <div className="content-scroll">{children}</div>
      </section>
    </main>
  );
}

function SidebarLink({ icon: Icon, label, active = false, onClick }) {
  return (
    <button type="button" className={active ? "sidebar-link is-active" : "sidebar-link"} onClick={onClick}>
      <Icon size={18} /><span>{label}</span>
    </button>
  );
}

function Breadcrumb({ items }) {
  return (
    <nav className="breadcrumb" aria-label="Breadcrumb">
      <a href="#module">Home</a>
      {items.map((item) => <span key={item}><ChevronRight size={14} /><strong>{item}</strong></span>)}
    </nav>
  );
}

function Badge({ tone = "info", children }) {
  return <span className={`badge is-${tone}`}>{children}</span>;
}

function ModuleHeader({ active = "Configuration" }) {
  const tabs = ["Overview", "Configuration", "Declarations", "Contracts", "Capabilities", "Dependencies", "Resources", "Navigation", "Findings"];
  return (
    <>
      <Breadcrumb items={["Module Management", "Dashboards"]} />
      <div className="module-heading">
        <div>
          <h1>Dashboards</h1>
          <code>tessara.dashboards</code>
          <div className="badge-row"><Badge>Independently deployed</Badge><Badge tone="success">Healthy and enabled</Badge></div>
        </div>
        <div className="button-row">
          <button className="button secondary" type="button"><ExternalLink size={16} /> View source descriptor (JSON)</button>
          <button className="button secondary" type="button"><ExternalLink size={16} /> View deployment receipt</button>
        </div>
      </div>
      <div className="tabs" role="tablist">
        {tabs.map((tab) => <button key={tab} className={tab === active ? "tab is-active" : "tab"} type="button">{tab}</button>)}
      </div>
      <label className="mobile-tab-select"><span>Module section</span><select value={active} readOnly>{tabs.map((tab) => <option key={tab}>{tab}</option>)}</select></label>
    </>
  );
}

function ModuleConfiguration({ navigate }) {
  return (
    <section className="route-panel">
      <ModuleHeader active="Configuration" />
      <div className="module-grid">
        <section className="card">
          <div className="card-heading">
            <div><h2>Configuration</h2><p>Validated by the Dashboard-owned configuration contract.</p></div>
            <button className="button secondary" type="button"><Pencil size={16} /> Edit configuration</button>
          </div>
          <dl className="definition-list">
            <div><dt>Schema version</dt><dd>1</dd></div>
            <div><dt>Display label</dt><dd>Dashboards</dd></div>
            <div><dt>Default page size</dt><dd>20 dashboards</dd></div>
            <div><dt>Validation</dt><dd><Badge tone="success">Valid</Badge> Release 0.1.0 · no findings</dd></div>
            <div><dt>Authoritative validator</dt><dd>Dashboard configuration contract</dd></div>
          </dl>
          <div className="transition-note">
            <LockKeyhole size={18} />
            <div><strong>Transition Components binding</strong><span>First-party Core Release adapter · unavailable to external Blueprints · explicit migration required in Sprint 8A.</span></div>
          </div>
        </section>
        <section className="card lifecycle-card">
          <div className="card-heading"><div><h2>Application state</h2><p>Configuration remains separate from operation.</p></div></div>
          <div className="state-line"><span>Configured</span><Badge tone="success">Valid</Badge></div>
          <div className="state-line"><span>Module health</span><Badge tone="success">Healthy</Badge></div>
          <div className="state-line"><span>Navigation</span><Badge>Visible</Badge></div>
          <div className="state-line"><span>Product route</span><Badge tone="success">Enabled</Badge></div>
          <div className="state-line"><span>Component adapter</span><Badge tone="success">Compatible</Badge></div>
          <button className="button secondary full" type="button" onClick={() => navigate("diagnostics")}><HeartPulse size={16} /> Open health and diagnostics</button>
        </section>
      </div>
    </section>
  );
}

function Diagnostics() {
  return (
    <section className="route-panel">
      <ModuleHeader active="Findings" />
      <div className="page-heading">
        <div><p className="eyebrow">Dashboard module</p><h2>Health and diagnostics</h2><p>Sanitized operational context for the independently deployed Dashboard service.</p></div>
        <div className="button-row"><button className="button secondary" type="button"><RefreshCw size={16} /> Refresh status</button><button className="button secondary" type="button">Download diagnostics</button></div>
      </div>
      <div className="metric-grid">
        <Metric icon={HeartPulse} label="Readiness" value="Ready" detail="Configuration, database, and Core authorization exchange are available." />
        <Metric icon={Activity} label="Liveness" value="Healthy" detail="Dashboard service responded 18 seconds ago." />
        <Metric icon={Database} label="Dashboard database" value="Connected" detail="Instance-scoped runtime identity · baseline 1 applied." />
        <Metric icon={ShieldCheck} label="Core authorization" value="Current" detail="Authorization revision 42 · Organization revision 18." />
      </div>
      <section className="card dependency-card">
        <div className="card-heading"><div><h2>Components compatibility dependency</h2><p>Transition-only first-party Core Release binding.</p></div><Badge tone="success">Compatible</Badge></div>
        <dl className="definition-list">
          <div><dt>Binding</dt><dd><code>tessara.dashboards.component-version</code></dd></div>
          <div><dt>Contract</dt><dd><code>tessara.components.component-version</code></dd></div>
          <div><dt>Provider</dt><dd>Core installation · in-process transition contribution</dd></div>
          <div><dt>Declared actions</dt><dd>Resolve metadata · Render</dd></div>
          <div><dt>Last successful check</dt><dd>Jul 26, 2026 · 9:42 AM</dd></div>
        </dl>
      </section>
    </section>
  );
}

function Metric({ icon: Icon, label, value, detail }) {
  return (
    <article className="metric"><Icon size={21} /><div><span>{label}</span><strong>{value}</strong><small>{detail}</small></div></article>
  );
}

function StatePicker({ value, onChange }) {
  return (
    <label className="state-picker">
      <span>Prototype control · placement state</span>
      <select value={value} onChange={(event) => onChange(event.target.value)}>
        {Object.entries(placementStates).map(([key, state]) => <option key={key} value={key}>{state.label}</option>)}
      </select>
    </label>
  );
}

function DashboardEditor({ state, setState, navigate }) {
  return (
    <section className="dashboard-workspace editor-workspace">
      <div className="builder-heading">
        <div><p className="eyebrow">Dashboard builder</p><h1>Demo Operations Dashboard</h1><p>9 local placements · reading order is derived</p><span className="saved"><i /> Layout changes saved</span></div>
        <div className="button-row"><button className="button secondary" type="button">Details</button><button className="button secondary" type="button" onClick={() => navigate("viewer")}>Preview Dashboard</button><button className="button" type="button">Save layout</button></div>
      </div>
      <details className="settings-row"><summary>Dashboard settings</summary></details>
      <div className="editor-tools">
        <div className="segmented"><button type="button">Components</button><button className="is-active" type="button">Placement details</button></div>
        <p>A placement is selected. Review its provider state without changing the saved layout.</p>
        <StatePicker value={state} onChange={setState} />
      </div>
      <div className="canvas-heading"><div><p className="eyebrow">Canvas</p><h2>12-column layout</h2></div><strong>9 placements</strong></div>
      <div className="dashboard-canvas">
        <PlacementCard index="1" title="Partner Profile" />
        <PlacementCard index="2" title="Program Snapshot" state={state} selected />
        <PlacementCard index="3" title="Activity Plan" />
      </div>
    </section>
  );
}

function PlacementCard({ index, title, state, selected = false }) {
  const current = state ? placementStates[state] : placementStates.healthy;
  const [isSheetOpen, setIsSheetOpen] = useState(false);
  const [isRetrying, setIsRetrying] = useState(false);

  useEffect(() => {
    setIsSheetOpen(false);
    setIsRetrying(false);
  }, [state]);

  const retryResolution = () => {
    setIsRetrying(true);
    window.setTimeout(() => {
      setIsRetrying(false);
      setIsSheetOpen(false);
    }, 900);
  };

  return (
    <>
      <article className={`placement-card ${selected ? "is-selected" : ""} ${state && state !== "healthy" ? `is-${current.tone}` : ""}`}>
        <div className="placement-header"><span className="placement-index">{index}</span><div><strong>{title}</strong><small>Table · v1 · 1</small></div><span className="drag-handle">•••</span></div>
      {state && state !== "healthy" ? (
        <div className="placement-state is-condensed">
          <button
            className="placement-warning-trigger"
            type="button"
            aria-label={`Open ${current.title} details`}
            aria-haspopup="dialog"
            onClick={() => setIsSheetOpen(true)}
          >
            {current.tone === "danger" ? <ShieldAlert size={42} /> : current.tone === "warning" ? <AlertTriangle size={42} /> : <Activity size={42} />}
          </button>
        </div>
      ) : (
        <div className="placement-placeholder"><Grid2X2 size={34} /><span>6 × 4</span></div>
      )}
      </article>
      {isSheetOpen && (
        <div className="placement-sheet-layer" role="presentation" onMouseDown={() => setIsSheetOpen(false)}>
          <aside className={`placement-sheet is-${current.tone}`} role="dialog" aria-modal="true" aria-labelledby="placement-sheet-title" onMouseDown={(event) => event.stopPropagation()}>
            <div className="placement-sheet__header">
              <div>
                <p className="eyebrow">Placement issue</p>
                <h2 id="placement-sheet-title">{current.title}</h2>
              </div>
              <button className="icon-button quiet" type="button" aria-label="Close placement issue" onClick={() => setIsSheetOpen(false)}><X size={20} /></button>
            </div>
            <div className="placement-sheet__body">
              <span className="placement-sheet__icon">
                {current.tone === "danger" ? <ShieldAlert size={28} /> : current.tone === "warning" ? <AlertTriangle size={28} /> : <Activity size={28} />}
              </span>
              <Badge tone={current.tone}>{current.short}</Badge>
              <p>{current.description}</p>
              <small>{current.detail}</small>
            </div>
            <div className="placement-sheet__footer">
              <button className="button" type="button" disabled={isRetrying} onClick={retryResolution}>
                <RefreshCw size={16} className={isRetrying ? "is-spinning" : ""} />
                {isRetrying ? "Retrying…" : "Retry resolution"}
              </button>
            </div>
          </aside>
        </div>
      )}
    </>
  );
}

function DashboardViewer({ state, setState, navigate }) {
  const current = placementStates[state];
  return (
    <section className="dashboard-workspace viewer-workspace">
      <Breadcrumb items={["Dashboards", "Detail", "Viewer"]} />
      <div className="viewer-heading">
        <div><h1>Demo Operations Dashboard</h1><p>Operational view of partner, program, activity, and session data.</p></div>
        <Badge>9 placements</Badge>
      </div>
      <StatePicker value={state} onChange={setState} />
      <div className="viewer-grid">
        <ViewerCard index="1" title="Partner Profile" />
        <article className={`viewer-card is-${current.tone}`}>
          <div className="viewer-card__header"><span className="placement-index">2</span><strong>Program Snapshot</strong><Badge tone={current.tone}>{current.short}</Badge></div>
          {state === "healthy" ? <MiniTable /> : (
            <div className="viewer-state">
              {current.tone === "danger" ? <ShieldAlert size={28} /> : current.tone === "warning" ? <AlertTriangle size={28} /> : <Unplug size={28} />}
              <div><h2>{current.title}</h2><p>{current.description}</p><small>{current.detail}</small></div>
              {(state === "providerUnavailable" || state === "notEvaluated") && <button className="button secondary compact" type="button">Retry</button>}
            </div>
          )}
        </article>
        <ViewerCard index="3" title="Activity Plan" />
      </div>
      <div className="contained-note"><CheckCircle2 size={18} /><span><strong>Dashboard remains available.</strong> One placement state does not interrupt other resolved placements.</span><button className="link-button" type="button" onClick={() => navigate("diagnostics")}>Open diagnostics</button></div>
    </section>
  );
}

function ViewerCard({ index, title }) {
  return (
    <article className="viewer-card">
      <div className="viewer-card__header"><span className="placement-index">{index}</span><strong>{title}</strong><button className="button secondary compact" type="button">View fullscreen</button></div>
      <MiniTable />
    </article>
  );
}

function MiniTable() {
  return (
    <div className="mini-table">
      <p>Showing 2 rows across 1 visible column.</p>
      <label><Search size={15} /><input type="search" placeholder="Search table" /></label>
      <table><thead><tr><th>Contact name</th></tr></thead><tbody><tr><td>Avery Johnson</td></tr><tr><td>Morgan Lee</td></tr></tbody></table>
      <small>Page 1 · 2 rows</small>
    </div>
  );
}

function DashboardUnavailable({ navigate }) {
  return (
    <section className="route-panel unavailable-page">
      <Breadcrumb items={["Dashboards"]} />
      <div className="unavailable-state">
        <div className="unavailable-icon"><Unplug size={30} /></div>
        <div>
          <p className="eyebrow">Dashboard module unavailable</p>
          <h1>Dashboards cannot be reached right now</h1>
          <p>Core and the rest of Tessara are still available. Your Dashboard data remains in the Dashboard Module Instance database.</p>
          <div className="protection-list">
            <span><Check size={16} /> No Dashboard request was sent to another provider.</span>
            <span><Check size={16} /> Core credentials and browser cookies were not forwarded.</span>
            <span><Check size={16} /> Existing Dashboard references and configuration are preserved.</span>
          </div>
          <div className="button-row"><button className="button" type="button"><RefreshCw size={16} /> Try Dashboards again</button><button className="button secondary" type="button" onClick={() => navigate("diagnostics")}>Open Module diagnostics</button></div>
        </div>
        <aside><span>Last known state</span><strong>Healthy · 2 minutes ago</strong><span>Module Instance</span><code>dashboards-primary</code><span>Route</span><code>/dashboards</code></aside>
      </div>
    </section>
  );
}
