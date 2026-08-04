import { useEffect, useState } from "react";
import {
  Archive,
  Bell,
  Boxes,
  ChevronDown,
  ChevronRight,
  CircleHelp,
  Database,
  EllipsisVertical,
  FileText,
  GitBranch,
  Grid2X2,
  Home,
  LayoutDashboard,
  ListChecks,
  Menu,
  Moon,
  Pencil,
  RefreshCw,
  Search,
  Settings,
  ShieldCheck,
  Sun,
  Users,
  X,
} from "lucide-react";

const screens = [
  { id: "dashboard", label: "Dashboard dependency findings", group: "Dashboard editor" },
  { id: "versions", label: "Component lifecycle", group: "Component Versions" },
];

const scenarios = {
  updated: {
    label: "Published version changed",
    title: "Reference Metric Card changed",
    description: "The pinned ComponentVersion was updated in place after this Dashboard last observed it.",
    impact: "Review layout compatibility before keeping the updated presentation.",
    revision: "Revision 18 → 19",
    tone: "warning",
    actions: ["Defer", "Replace", "Remove"],
  },
  successor: {
    label: "Successor available",
    title: "A successor is available",
    description: "Version 1.1.0 is the provider-declared active published successor.",
    impact: "The Dashboard remains pinned to 1.0.0 until you explicitly upgrade.",
    revision: "Revision 19 → 20",
    tone: "info",
    actions: ["Defer", "Upgrade", "Replace", "Remove"],
  },
  inactive: {
    label: "Inactive dependency",
    title: "Reference Metric Card is inactive",
    description: "The pinned version is retained and visible, but it cannot render while inactive.",
    impact: "Reactivate the version, replace it, or remove the placement.",
    revision: "Revision 20 → 21",
    tone: "warning",
    actions: ["Defer", "Replace", "Remove"],
  },
  archived: {
    label: "Archived dependency",
    title: "Reference Metric Card is archived",
    description: "Authorized metadata remains available, but the version cannot render or reactivate.",
    impact: "Replace or remove this placement.",
    revision: "Revision 21 → 22",
    tone: "danger",
    actions: ["Defer", "Replace", "Remove"],
  },
  tombstoned: {
    label: "Tombstoned dependency",
    title: "ComponentVersion was tombstoned",
    description: "Only the typed tombstone resolution remains visible. Component metadata is no longer available.",
    impact: "Replace or remove the placement.",
    revision: "Revision 22 → 23",
    tone: "danger",
    actions: ["Defer", "Replace", "Remove"],
  },
};

function currentScreen() {
  const id = window.location.hash.replace("#", "");
  return screens.some((screen) => screen.id === id) ? id : "dashboard";
}

export function App() {
  const [screen, setScreen] = useState(currentScreen);
  const [theme, setTheme] = useState("dark");
  const [reviewOpen, setReviewOpen] = useState(false);
  const [scenario, setScenario] = useState("successor");

  useEffect(() => {
    const syncScreen = () => setScreen(currentScreen());
    window.addEventListener("hashchange", syncScreen);
    return () => window.removeEventListener("hashchange", syncScreen);
  }, []);

  function navigate(id) {
    window.location.hash = id;
    setScreen(id);
    setReviewOpen(false);
    window.scrollTo({ top: 0, behavior: "instant" });
  }

  return (
    <div data-theme={theme}>
      {screen === "dashboard" ? (
        <DashboardEditor theme={theme} setTheme={setTheme} scenario={scenario} />
      ) : (
        <ComponentVersions theme={theme} setTheme={setTheme} />
      )}
      <ReviewNavigator
        screen={screen}
        open={reviewOpen}
        setOpen={setReviewOpen}
        navigate={navigate}
        theme={theme}
        setTheme={setTheme}
        scenario={scenario}
        setScenario={setScenario}
      />
    </div>
  );
}

function ReviewNavigator({ screen, open, setOpen, navigate, theme, setTheme, scenario, setScenario }) {
  const active = screens.find((item) => item.id === screen);
  return (
    <aside className={`review-nav ${open ? "is-open" : ""}`} aria-label="Sprint 7B prototype controls">
      <button className="review-trigger" type="button" onClick={() => setOpen(!open)}>
        <span><small>Prototype control · Sprint 7B</small><strong>{active.label}</strong></span>
        <span>{screens.findIndex((item) => item.id === screen) + 1} / {screens.length}</span>
        <ChevronDown size={16} />
      </button>
      {open && (
        <div className="review-menu">
          <div className="review-menu__head">
            <span><strong>Review screens</strong><small>Not proposed product UI</small></span>
            <button type="button" className="icon quiet" aria-label="Close review controls" onClick={() => setOpen(false)}><X size={18} /></button>
          </div>
          {screens.map((item, index) => (
            <button className={item.id === screen ? "review-item active" : "review-item"} type="button" key={item.id} onClick={() => navigate(item.id)}>
              <span>{String(index + 1).padStart(2, "0")}</span>
              <span><strong>{item.label}</strong><small>{item.group}</small></span>
              <ChevronRight size={16} />
            </button>
          ))}
          {screen === "dashboard" && (
            <label className="review-scenario">
              <span>Review finding state</span>
              <select value={scenario} onChange={(event) => setScenario(event.target.value)}>
                {Object.entries(scenarios).map(([id, item]) => <option value={id} key={id}>{item.label}</option>)}
              </select>
              <small>Prototype-only data selector</small>
            </label>
          )}
          <button className="review-theme" type="button" onClick={() => setTheme(theme === "dark" ? "light" : "dark")}> 
            {theme === "dark" ? <Sun size={16} /> : <Moon size={16} />}
            Review {theme === "dark" ? "light" : "dark"} theme
          </button>
        </div>
      )}
    </aside>
  );
}

function SdkShell({ activeRoute, title, theme, setTheme, contentClass = "", children }) {
  const [menuOpen, setMenuOpen] = useState(false);

  return (
    <main className="core-shell" data-shell-owner="tessara-module-ui">
      <aside className={menuOpen ? "core-sidebar open" : "core-sidebar"}>
        <a className="core-brand" href={activeRoute === "dashboard" ? "#dashboard" : "#versions"}><img src="/tessara-icon.svg" alt="" /><strong>Tessara</strong></a>
        <nav>
          <p>Main</p>
          <CoreLink icon={Home} label="Home" />
          <CoreLink icon={GitBranch} label="Organization" />
          <CoreLink icon={FileText} label="Forms" />
          <CoreLink icon={Grid2X2} label="Workflows" />
          <CoreLink icon={CircleHelp} label="Responses" />
          <CoreLink icon={ListChecks} label="Operations" />
          <CoreLink icon={Database} label="Datasets" />
          <CoreLink icon={Pencil} label="Components" href="#versions" active={activeRoute === "versions"} />
          <CoreLink icon={LayoutDashboard} label="Dashboards" href="#dashboard" active={activeRoute === "dashboard"} />
          <CoreLink icon={FileText} label="Scoped Records" />
          <p>Admin</p>
          <CoreLink icon={Users} label="User Management" />
          <CoreLink icon={ShieldCheck} label="Roles & Access" />
          <CoreLink icon={Settings} label="Node Types" />
          <CoreLink icon={Boxes} label="Module Management" />
        </nav>
      </aside>
      <section className="core-main">
        <header className="core-top">
          <div><button className="icon mobile-menu" type="button" aria-label="Open menu" onClick={() => setMenuOpen(!menuOpen)}><Menu size={20} /></button><h1>{title}</h1></div>
          <div className="core-actions"><label><Search size={15} /><input type="search" placeholder="Search Tessara" /></label><button className="icon accent" type="button" aria-label="Toggle theme" onClick={() => setTheme(theme === "dark" ? "light" : "dark")}>{theme === "dark" ? <Moon size={18} /> : <Sun size={18} />}</button><button className="icon accent" type="button" aria-label="Notifications"><Bell size={18} /></button><button className="icon accent" type="button" aria-label="Help"><CircleHelp size={18} /></button></div>
        </header>
        <div className={`core-scroll ${contentClass}`.trim()}>{children}</div>
      </section>
    </main>
  );
}

function DashboardEditor({ theme, setTheme, scenario }) {
  const [sheetOpen, setSheetOpen] = useState(true);
  const [selected, setSelected] = useState("successor");
  const [deferred, setDeferred] = useState(false);
  const [action, setAction] = useState(null);
  const [resolved, setResolved] = useState(false);
  const finding = scenarios[selected];

  useEffect(() => {
    setSelected(scenario);
    setDeferred(false);
    setResolved(false);
    setSheetOpen(true);
    setAction(null);
  }, [scenario]);

  function completeAction(kind) {
    if (kind === "Defer") {
      setDeferred(true);
      setAction(null);
      return;
    }
    if (kind === "Upgrade" || kind === "Replace" || kind === "Remove") {
      setResolved(true);
      setAction(null);
      setSheetOpen(false);
    }
  }

  return (
    <>
      <SdkShell activeRoute="dashboard" title="Edit Dashboard" theme={theme} setTheme={setTheme} contentClass="dashboard-stage">
          <section className="dashboard-workspace">
            <div className="builder-heading">
              <div>
                <p className="eyebrow">Dashboard builder</p>
                <h1>Reference Operations</h1>
                <p>4 local placements · reading order is derived</p>
                <span className="saved"><i /> Layout changes saved</span>
              </div>
              <div className="button-row">
                <button className="button secondary" type="button">Details</button>
                <button className="button secondary" type="button">Preview Dashboard</button>
                <button className="button" type="button" disabled>Save layout</button>
              </div>
            </div>
            <details className="settings-row"><summary>Dashboard settings</summary></details>
            <div className="editor-tools">
              <div className="segmented">
                <button type="button">Components</button>
                <button type="button">Placement details</button>
                <button className="dependency-button" type="button" onClick={() => setSheetOpen(true)}>
                  Dependency health
                  <span className={resolved ? "count healthy" : "count"}>{resolved ? "Healthy" : "1 issue"}</span>
                </button>
              </div>
            </div>
            <div className="canvas-heading"><div><p className="eyebrow">Canvas</p><h2>12-column layout</h2></div><strong>4 placements</strong></div>
            <div className="dashboard-canvas">
              <Placement index="1" title="Reference Metric Card" subtitle="Stat Card · v1 · 1.0.0" affected={!resolved} onOpen={() => setSheetOpen(true)} />
              <Placement index="25" title="Reference Records Table" subtitle="Table · v1 · 1.0.0" />
              <Placement index="97" title="Sprint 7A Tier Chart" subtitle="Bar · v1 · 1.0.0" />
              <Placement index="145" title="Sprint 7A Blocked Component" subtitle="Table · v1 · 1.0.0" />
            </div>
          </section>
      </SdkShell>
      {sheetOpen && !resolved && (
        <div className="sheet-layer" role="presentation">
          <aside className="dependency-sheet" role="dialog" aria-modal="true" aria-labelledby="dependency-sheet-title">
            <div className="sheet-head">
              <div><p className="eyebrow">Dashboard dependencies</p><h2 id="dependency-sheet-title">Dependency health</h2><p>1 of 4 placements needs review</p></div>
              <button className="icon quiet" type="button" aria-label="Close dependency health" onClick={() => setSheetOpen(false)}><X size={19} /></button>
            </div>
            <div className="sheet-filter"><button className="filter active" type="button">Needs review <span>1</span></button><button className="filter" type="button">Deferred <span>{deferred ? 1 : 0}</span></button><button className="filter" type="button">Healthy <span>3</span></button><button className="icon quiet" type="button" aria-label="Refresh dependency health"><RefreshCw size={17} /></button></div>
            <div className="sheet-body">
              <article className={`finding-card ${deferred ? "deferred" : ""}`}>
                <button type="button" className="finding-select" onClick={() => setSelected(scenario)}>
                  <span className={`status-dot ${finding.tone}`} />
                  <span><strong>Reference Metric Card</strong><small>{deferred ? "Deferred · health remains degraded" : finding.label}</small></span>
                  <ChevronRight size={17} />
                </button>
              </article>
              <section className="finding-detail">
                <div className="finding-meta"><span className={`badge ${finding.tone}`}>{deferred ? "Deferred" : "Needs review"}</span><code>{finding.revision}</code></div>
                <h3>{finding.title}</h3>
                <p>{finding.description}</p>
                <div className="impact"><strong>Dashboard impact</strong><span>{finding.impact}</span></div>
                <dl><div><dt>Placement</dt><dd>Reference Metric Card · position 1</dd></div><div><dt>Observed</dt><dd>Aug 3, 2026 · 2:18 PM</dd></div><div><dt>Reference</dt><dd><code>ComponentVersion · 1.0.0</code></dd></div></dl>
                <div className="action-row">
                  {finding.actions.map((item) => <button key={item} className={item === "Upgrade" ? "button" : "button secondary"} type="button" onClick={() => item === "Defer" ? completeAction(item) : setAction(item)} disabled={item === "Defer" && deferred}>{item === "Defer" && deferred ? "Deferred" : item}</button>)}
                </div>
              </section>
            </div>
          </aside>
        </div>
      )}
      {action && <ActionDialog action={action} finding={finding} onCancel={() => setAction(null)} onConfirm={() => completeAction(action)} />}
    </>
  );
}

function Placement({ index, title, subtitle, affected = false, onOpen }) {
  return (
    <article className={affected ? "placement affected" : "placement"}>
      <div><span className="placement-index">{index}</span><span><strong>{title}</strong><small>{subtitle}</small></span></div>
      {affected ? <button className="placement-issue" type="button" aria-label="Open Reference Metric Card dependency issue" onClick={onOpen}><CircleHelp size={31} /></button> : <Grid2X2 size={31} />}
      <small>12 × 1</small>
    </article>
  );
}

function ActionDialog({ action, finding, onCancel, onConfirm }) {
  const replace = action === "Replace";
  return (
    <div className="dialog-layer" role="presentation">
      <section className="confirm-dialog" role="dialog" aria-modal="true" aria-labelledby="action-title">
        <div className="dialog-head"><div><p className="eyebrow">Dependency action</p><h2 id="action-title">{action} ComponentVersion</h2></div><button className="icon quiet" type="button" aria-label={`Cancel ${action}`} onClick={onCancel}><X size={18} /></button></div>
        <div className="dialog-body">
          <p>{action === "Upgrade" ? "Upgrade to the provider-declared successor. The saved Dashboard changes only after confirmation." : action === "Replace" ? "Choose any authorized renderable ComponentVersion." : "Remove this placement from the Dashboard layout."}</p>
          {action !== "Remove" && <label><span>{replace ? "Replacement" : "Successor"}</span><select><option>{replace ? "Reference Records Table · 1.0.0" : "Reference Metric Card · 1.1.0"}</option>{replace && <option>Demo Session Total Participants · v1</option>}</select></label>}
          <div className="receipt-preview"><strong>Atomic result</strong><span>{action === "Remove" ? "Placement removed" : `Reference ${action === "Upgrade" ? "upgraded" : "replaced"}`} · finding resolved · receipt retained</span><code>{finding.revision}</code></div>
        </div>
        <div className="dialog-actions"><button className="button secondary" type="button" onClick={onCancel}>Cancel</button><button className={action === "Remove" ? "button danger" : "button"} type="button" onClick={onConfirm}>Confirm {action}</button></div>
      </section>
    </div>
  );
}

function ComponentVersions({ theme, setTheme }) {
  const [versions, setVersions] = useState([
    { version: "1.1.0", publication: "Published", lifecycle: "Active", kind: "Stat Card", dataset: "v1", note: "Provider-declared successor" },
    { version: "1.0.0", publication: "Superseded", lifecycle: "Active", kind: "Stat Card", dataset: "v1", note: "Application composition bootstrap" },
  ]);
  const [confirm, setConfirm] = useState(null);
  const [actionMenuVersion, setActionMenuVersion] = useState(null);

  function applyLifecycle() {
    setVersions((items) => items.map((item) => item.version === confirm.version ? { ...item, lifecycle: confirm.action === "Tombstone" ? "Tombstoned" : confirm.action === "Archive" ? "Archived" : confirm.action === "Deactivate" ? "Inactive" : "Active" } : item));
    setConfirm(null);
  }

  function beginLifecycle(version, action) {
    setActionMenuVersion(null);
    setConfirm({ version, action });
  }

  return (
    <>
      <SdkShell activeRoute="versions" title="Component Versions" theme={theme} setTheme={setTheme}>
          <section className="versions-panel">
            <nav className="breadcrumb"><a href="#versions">Components</a><ChevronRight size={14} /><a href="#versions">Reference Metric Card</a><ChevronRight size={14} /><strong>Versions</strong></nav>
            <div className="versions-heading"><h2>Reference Metric Card</h2><div><button className="button secondary" type="button">Edit</button><button className="button" type="button">View</button></div></div>
            <h3>Versions</h3>
            <div className="table-scroll"><table className="versions-table"><thead><tr><th>Version</th><th>Publication</th><th>Lifecycle</th><th>Kind</th><th>Dataset Version</th><th>Note</th><th><span className="visually-hidden">Actions</span></th></tr></thead><tbody>{versions.map((version) => {
              const actions = lifecycleActions(version.lifecycle);
              const menuOpen = actionMenuVersion === version.version;
              return <tr key={version.version}><td>{version.version}</td><td>{version.publication}</td><td><span className={`lifecycle ${version.lifecycle.toLowerCase()}`}>{version.lifecycle}</span></td><td>{version.kind}</td><td>{version.dataset}</td><td>{version.note}</td><td className="actions-cell">{actions.length > 0 && <div className="version-actions"><button className="version-actions__trigger" type="button" aria-label={`Actions for version ${version.version}`} aria-expanded={menuOpen} onClick={() => setActionMenuVersion(menuOpen ? null : version.version)}><EllipsisVertical size={20} /></button>{menuOpen && <div className="version-actions__menu" role="menu" aria-label={`Lifecycle actions for version ${version.version}`}>{actions.map((action) => <button className={action === "Tombstone" ? "danger-link" : ""} role="menuitem" type="button" key={action} onClick={() => beginLifecycle(version.version, action)}>{action}</button>)}</div>}</div>}</td></tr>;
            })}</tbody></table></div>
          </section>
      </SdkShell>
      {confirm && <LifecycleDialog change={confirm} onCancel={() => setConfirm(null)} onConfirm={applyLifecycle} />}
    </>
  );
}

function lifecycleActions(lifecycle) {
  if (lifecycle === "Active") return ["Deactivate", "Archive"];
  if (lifecycle === "Inactive") return ["Activate", "Archive"];
  if (lifecycle === "Archived") return ["Tombstone"];
  return [];
}

function CoreLink({ icon: Icon, label, href = "#versions", active = false }) {
  return <a href={href} className={active ? "active" : ""}><Icon size={18} /><span>{label}</span></a>;
}

function LifecycleDialog({ change, onCancel, onConfirm }) {
  const irreversible = change.action === "Archive" || change.action === "Tombstone";
  return (
    <div className="dialog-layer" role="presentation">
      <section className="confirm-dialog" role="dialog" aria-modal="true" aria-labelledby="lifecycle-title">
        <div className="dialog-head"><div><p className="eyebrow">Component lifecycle</p><h2 id="lifecycle-title">{change.action} version {change.version}?</h2></div><button className="icon quiet" type="button" aria-label={`Cancel ${change.action}`} onClick={onCancel}><X size={18} /></button></div>
        <div className="dialog-body"><p>{change.action === "Deactivate" ? "This version remains visible to authorized callers but cannot render until reactivated." : change.action === "Activate" ? "This version becomes renderable again for authorized consumers." : change.action === "Archive" ? "Archived versions cannot render or reactivate. Their metadata remains visible to authorized callers." : "Tombstoning is terminal. External metadata and render payloads will no longer be available."}</p>{irreversible && <div className="warning-note"><Archive size={20} /><span><strong>{change.action === "Archive" ? "This version cannot be reactivated." : "This action cannot be reversed."}</strong><small>The internal record and immutable audit history are retained.</small></span></div>}</div>
        <div className="dialog-actions"><button className="button secondary" type="button" onClick={onCancel}>Cancel</button><button className={change.action === "Tombstone" ? "button danger" : "button"} type="button" onClick={onConfirm}>Confirm {change.action}</button></div>
      </section>
    </div>
  );
}
