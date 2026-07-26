import { useEffect, useMemo, useState } from "react";
import {
  Activity,
  AlertTriangle,
  Bell,
  Blocks,
  Check,
  ChevronDown,
  ChevronLeft,
  ChevronRight,
  CircleHelp,
  ClipboardCheck,
  Database,
  ExternalLink,
  FileText,
  GitBranch,
  HeartPulse,
  Home,
  KeyRound,
  LayoutDashboard,
  ListChecks,
  LockKeyhole,
  LogOut,
  Menu,
  Moon,
  Network,
  PanelRight,
  Pencil,
  Plus,
  RefreshCw,
  Search,
  Settings,
  ShieldCheck,
  Sun,
  UserRound,
  Users,
  X,
} from "lucide-react";

const reviewScreens = [
  { id: "enrollment", label: "Administrator enrollment", group: "Bare route" },
  { id: "roles", label: "Capability Floor", group: "Core edit" },
  { id: "module", label: "Module configuration", group: "Core edit" },
  { id: "records", label: "Records directory", group: "Module route" },
  { id: "record", label: "Record detail", group: "Module route" },
  { id: "edit", label: "Record edit", group: "Module route" },
  { id: "create", label: "Record create", group: "Module route" },
  { id: "diagnostics", label: "Health and diagnostics", group: "Module route" },
  { id: "states", label: "Denied and recovery states", group: "State set" },
];

const records = [
  { id: "SR-1048", label: "North intake review", owner: "North Region", updated: "Jul 23, 2026 · 2:18 PM" },
  { id: "SR-1041", label: "West field verification", owner: "West Region", updated: "Jul 23, 2026 · 11:42 AM" },
  { id: "SR-1036", label: "North quarterly evidence", owner: "North Region", updated: "Jul 22, 2026 · 4:05 PM" },
  { id: "SR-1029", label: "Central operating record", owner: "Central Region", updated: "Jul 21, 2026 · 9:16 AM" },
];

function currentHash() {
  const id = window.location.hash.replace("#", "");
  return reviewScreens.some((item) => item.id === id) ? id : "enrollment";
}

export function App() {
  const [screen, setScreen] = useState(currentHash);
  const [reviewOpen, setReviewOpen] = useState(false);

  useEffect(() => {
    const sync = () => setScreen(currentHash());
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
  const active = reviewScreens[activeIndex];

  return (
    <>
      {screen === "enrollment" ? (
        <EnrollmentScreen />
      ) : (
        <Shell screen={screen} navigate={navigate}>
          {screen === "roles" && <RolesScreen />}
          {screen === "module" && <ModuleScreen navigate={navigate} />}
          {screen === "records" && <RecordsScreen navigate={navigate} />}
          {screen === "record" && <RecordDetailScreen navigate={navigate} />}
          {screen === "edit" && <RecordFormScreen key="edit" navigate={navigate} mode="edit" />}
          {screen === "create" && <RecordFormScreen key="create" navigate={navigate} mode="create" />}
          {screen === "diagnostics" && <DiagnosticsScreen />}
          {screen === "states" && <StatesScreen navigate={navigate} />}
        </Shell>
      )}
      <ReviewNavigator
        active={active}
        activeIndex={activeIndex}
        isOpen={reviewOpen}
        setOpen={setReviewOpen}
        navigate={navigate}
      />
    </>
  );
}

function ReviewNavigator({ active, activeIndex, isOpen, setOpen, navigate }) {
  return (
    <aside className={`review-nav ${isOpen ? "is-open" : ""}`} aria-label="Mockup review navigation">
      <button className="review-nav__trigger" type="button" onClick={() => setOpen(!isOpen)}>
        <span>
          <small>Sprint 6B2 review</small>
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
            <button className="icon-button icon-button--quiet" type="button" aria-label="Close review navigation" onClick={() => setOpen(false)}>
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
              <span>
                <strong>{item.label}</strong>
                <small>{item.group}</small>
              </span>
              {item.id === active.id && <Check size={16} />}
            </button>
          ))}
        </div>
      )}
    </aside>
  );
}

function EnrollmentScreen() {
  const [claimKind, setClaimKind] = useState("initial");
  const [identityPath, setIdentityPath] = useState("local");
  const [submitted, setSubmitted] = useState(false);

  return (
    <main className="login-shell enrollment-shell">
      <section className="login-panel enrollment-panel" aria-labelledby="enrollment-title">
        <a className="login-brand" href="#enrollment" aria-label="Tessara enrollment">
          <img src="/tessara-icon.svg" alt="" />
          <strong>Tessara</strong>
        </a>
        <div className="login-panel__header">
          <h1 id="enrollment-title">{submitted ? "Administrator enrolled" : "Establish an administrator"}</h1>
          <p>
            {submitted
              ? "Enrollment is complete. This one-time surface is now closed."
              : "Use the installation claim once to establish a floor-compliant Core administrator."}
          </p>
        </div>

        {submitted ? (
          <div className="enrollment-success">
            <span className="success-icon"><Check size={22} /></span>
            <div>
              <strong>Core Administrator assigned globally</strong>
              <p>The submitted claim is no longer available and has not been retained in this screen.</p>
            </div>
            <button className="button" type="button" onClick={() => setSubmitted(false)}>Continue to sign in</button>
          </div>
        ) : (
          <>
            <div className="segmented" aria-label="Enrollment claim kind">
              <button type="button" className={claimKind === "initial" ? "is-active" : ""} onClick={() => setClaimKind("initial")}>Initial</button>
              <button type="button" className={claimKind === "recovery" ? "is-active" : ""} onClick={() => setClaimKind("recovery")}>Recovery</button>
            </div>
            {claimKind === "recovery" && (
              <div className="inline-notice is-warning">
                <AlertTriangle size={18} />
                <div>
                  <strong>Audited recovery claim</strong>
                  <span>Authorized locally by operator eric-dev · expires in 12 minutes</span>
                </div>
              </div>
            )}
            <div className="identity-choice" role="group" aria-label="Identity path">
              <button type="button" className={identityPath === "local" ? "identity-card is-active" : "identity-card"} onClick={() => setIdentityPath("local")}>
                <LockKeyhole size={20} />
                <span><strong>Local account</strong><small>Create a Tessara password</small></span>
              </button>
              <button type="button" className={identityPath === "external" ? "identity-card is-active" : "identity-card"} onClick={() => setIdentityPath("external")}>
                <ExternalLink size={20} />
                <span><strong>Fixture external identity</strong><small>Bind a signed assertion</small></span>
              </button>
            </div>
            <form className="login-form" onSubmit={(event) => { event.preventDefault(); setSubmitted(true); }}>
              <label className="login-field">
                <span className="login-field__label">Enrollment claim</span>
                <span className="login-input-shell">
                  <KeyRound className="login-field__icon" />
                  <input required type="password" placeholder="Enter the once-displayed claim" autoComplete="off" />
                </span>
                <small className="field-help">Write-only. The claim will not appear again after submission.</small>
              </label>
              {identityPath === "local" ? (
                <>
                  <label className="login-field">
                    <span className="login-field__label">Email</span>
                    <span className="login-input-shell">
                      <UserRound className="login-field__icon" />
                      <input required type="email" defaultValue="admin@tessara.local" />
                    </span>
                  </label>
                  <label className="login-field">
                    <span className="login-field__label">Password</span>
                    <span className="login-input-shell">
                      <LockKeyhole className="login-field__icon" />
                      <input required type="password" placeholder="Create a password" />
                    </span>
                  </label>
                </>
              ) : (
                <label className="login-field">
                  <span className="login-field__label">Signed fixture assertion</span>
                  <textarea required rows="4" placeholder="Paste the one-time signed assertion" />
                  <small className="field-help">Development conformance path only. No email-based account merging.</small>
                </label>
              )}
              <div className="assignment-summary">
                <ShieldCheck size={18} />
                <div>
                  <strong>Core Administrator</strong>
                  <span>Meets Core Administration Capability Floor v1 · installation-global</span>
                </div>
              </div>
              <button className="button login-submit" type="submit">
                {claimKind === "initial" ? "Enroll administrator" : "Recover administrator access"}
              </button>
            </form>
          </>
        )}
      </section>
    </main>
  );
}

function Shell({ screen, navigate, children }) {
  const active = screen === "roles" ? "roles" : screen === "module" ? "modules" : "scoped";
  const title = screen === "roles" ? "Roles" : screen === "module" ? "Module Management" : "Scoped Records";
  return (
    <main className="app-shell">
      <aside className="sidebar" aria-label="Primary navigation">
        <a className="sidebar-brand" href="#records" onClick={(event) => { event.preventDefault(); navigate("records"); }}>
          <img src="/tessara-icon.svg" alt="" />
          <strong>Tessara</strong>
        </a>
        <nav className="sidebar-nav" aria-label="Primary">
          <p className="sidebar-section">Main</p>
          <SidebarLink icon={Home} label="Home" />
          <SidebarLink icon={GitBranch} label="Organization" />
          <SidebarLink icon={FileText} label="Forms" />
          <SidebarLink icon={PanelRight} label="Workflows" />
          <SidebarLink icon={CircleHelp} label="Responses" />
          <SidebarLink icon={ListChecks} label="Operations" />
          <SidebarLink icon={Database} label="Datasets" />
          <SidebarLink icon={Pencil} label="Components" />
          <SidebarLink icon={LayoutDashboard} label="Dashboards" />
          <SidebarLink icon={ClipboardCheck} label="Scoped Records" active={active === "scoped"} onClick={() => navigate("records")} />
          <p className="sidebar-section">Admin</p>
          <SidebarLink icon={Users} label="User Management" />
          <SidebarLink icon={ShieldCheck} label="Roles & Access" active={active === "roles"} onClick={() => navigate("roles")} />
          <SidebarLink icon={Network} label="Node Types" />
          <SidebarLink icon={Blocks} label="Module Management" active={active === "modules"} onClick={() => navigate("module")} />
        </nav>
        <div className="account-card">
          <span className="account-card__avatar">TA</span>
          <span className="account-card__identity">
            <strong>Tessara Admin</strong>
            <small>ADMIN@TESSARA.LOCAL</small>
          </span>
          <button className="icon-button" type="button" aria-label="Sign out"><LogOut size={18} /></button>
        </div>
      </aside>
      <section className="application-content">
        <header className="top-app-bar">
          <div className="top-app-bar__title-row">
            <button className="icon-button mobile-menu" type="button" aria-label="Open menu"><Menu size={19} /></button>
            <h1 className="top-app-bar__title">{title}</h1>
          </div>
          <div className="top-app-bar__actions">
            <label className="global-search">
              <Search size={16} />
              <input type="search" placeholder="Search Tessara" />
            </label>
            <button className="icon-button" type="button" aria-label="Theme options"><Moon size={18} /></button>
            <button className="icon-button" type="button" aria-label="Notifications"><Bell size={18} /></button>
            <button className="icon-button" type="button" aria-label="Help"><CircleHelp size={18} /></button>
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
      <Icon size={18} />
      <span>{label}</span>
    </button>
  );
}

function Breadcrumb({ items }) {
  return (
    <nav className="breadcrumb" aria-label="Breadcrumb">
      <a href="#records">Home</a>
      {items.map((item) => (
        <span key={item}><ChevronRight size={14} /> <strong>{item}</strong></span>
      ))}
    </nav>
  );
}

function PageHeading({ title, description, action }) {
  return (
    <div className="page-heading">
      <div>
        <h1>{title}</h1>
        {description && <p>{description}</p>}
      </div>
      {action}
    </div>
  );
}

function StatusBadge({ tone = "info", children }) {
  return <span className={`status-badge is-${tone}`}>{children}</span>;
}

function RolesScreen() {
  const [selectedRole, setSelectedRole] = useState("Core Administrator");
  return (
    <section className="route-panel">
      <Breadcrumb items={["Roles"]} />
      <PageHeading title="Roles" action={<button className="button" type="button"><Plus size={17} /> New Role</button>} />
      <section className="floor-banner" aria-labelledby="floor-title">
        <div className="floor-banner__icon"><ShieldCheck size={24} /></div>
        <div className="floor-banner__copy">
          <div className="eyebrow">Core Administration Capability Floor</div>
          <h2 id="floor-title">Floor v1 is covered</h2>
          <p>The designated enrollment role remains installation-global and satisfies all current Core administration obligations.</p>
        </div>
        <div className="floor-banner__meta">
          <StatusBadge tone="success">Compliant</StatusBadge>
          <span><small>Designated role</small><strong>Core Administrator</strong></span>
        </div>
      </section>
      <div className="table-toolbar">
        <label className="table-search"><Search size={16} /><input type="search" placeholder="Search roles" /></label>
      </div>
      <div className="data-table-wrap">
        <table className="data-table">
          <thead><tr><th>Role</th><th>Capabilities</th><th>Users</th><th>Enrollment</th></tr></thead>
          <tbody>
            {[
              ["Core Administrator", "1", "1", "Designated"],
              ["admin", "1", "0", "Break-glass"],
              ["operator", "10", "1", "—"],
              ["respondent", "2", "3", "—"],
            ].map((row) => (
              <tr key={row[0]} className={selectedRole === row[0] ? "is-selected" : ""}>
                <th><button className="link-button" type="button" onClick={() => setSelectedRole(row[0])}>{row[0]}</button></th>
                <td>{row[1]}</td><td>{row[2]}</td>
                <td>{row[3]}</td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>
      <section className="detail-card role-detail">
        <div className="detail-card__heading">
          <div><h2>{selectedRole}</h2><p>{selectedRole === "Core Administrator" ? "1 capability · 1 assigned user · designated for enrollment" : "Role detail"}</p></div>
          <div className="button-row">
            <button className="button button--secondary" type="button"><Pencil size={16} /> Edit Capabilities</button>
            <button className="button button--secondary" type="button">Assigned Users</button>
          </div>
        </div>
        {selectedRole === "Core Administrator" ? (
          <>
            <div className="capability-row">
              <div><code>core:admin</code><span>Core installation administration</span></div>
              <StatusBadge>Installation-global</StatusBadge>
              <span className="provenance"><i></i>Authoritative source: Core</span>
            </div>
            <div className="floor-obligations">
              <strong>Floor v1 obligations</strong>
              <span><Check size={15} /> Users and identity</span>
              <span><Check size={15} /> Roles and assignments</span>
              <span><Check size={15} /> Organization administration</span>
              <span><Check size={15} /> Module configuration and enablement</span>
              <span><Check size={15} /> Installation health and recovery</span>
            </div>
            <div className="inline-notice is-info">
              <CircleHelp size={18} />
              <div><strong>Module product capabilities remain separate</strong><span>Enrollment does not grant Scoped Records read or manage authority.</span></div>
            </div>
          </>
        ) : (
          <div className="empty-state">Select Core Administrator to review the Sprint 6B2 floor treatment.</div>
        )}
      </section>
    </section>
  );
}

function ModuleScreen({ navigate }) {
  const [editing, setEditing] = useState(false);
  const [label, setLabel] = useState("Scoped Records");
  const [enabled, setEnabled] = useState(true);
  const [saved, setSaved] = useState(false);
  return (
    <section className="route-panel">
      <Breadcrumb items={["Module Management", "scoped records"]} />
      <div className="module-heading">
        <div>
          <h1>scoped records</h1>
          <code>tessara.reference.scoped-records</code>
          <div className="badge-row"><StatusBadge>Independently deployed</StatusBadge><StatusBadge tone="success">Healthy and enabled</StatusBadge></div>
        </div>
        <div className="button-row">
          <button className="button button--secondary" type="button"><ExternalLink size={16} /> View source descriptor (JSON)</button>
          <button className="button button--secondary" type="button">View deployment receipt</button>
        </div>
      </div>
      <div className="tabs-list">
        {["Overview", "Configuration", "Declarations", "Contracts", "Capabilities", "Dependencies", "Resources", "Navigation", "Findings"].map((tab) => (
          <button key={tab} type="button" className={tab === "Configuration" ? "tabs-trigger is-active" : "tabs-trigger"}>{tab}</button>
        ))}
      </div>
      <label className="mobile-tab-select">
        <span>Module detail section</span>
        <select defaultValue="Configuration">
          {["Overview", "Configuration", "Declarations", "Contracts", "Capabilities", "Dependencies", "Resources", "Navigation", "Findings"].map((tab) => <option key={tab}>{tab}</option>)}
        </select>
      </label>
      <div className="module-config-grid">
        <section className="detail-card">
          <div className="detail-card__heading">
            <div><h2>Configuration</h2><p>Validated by the module-owned configuration contract.</p></div>
            {!editing && <button className="button button--secondary" type="button" onClick={() => { setEditing(true); setSaved(false); }}><Pencil size={16} /> Edit configuration</button>}
          </div>
          {editing ? (
            <form className="form-stack" onSubmit={(event) => { event.preventDefault(); setEditing(false); setSaved(true); }}>
              <label className="form-field"><span>Display label</span><input value={label} onChange={(event) => setLabel(event.target.value)} /></label>
              <div className="validation-preview">
                <ClipboardCheck size={19} />
                <div><strong>Configuration is valid</strong><span>Schema v1 · normalized label “{label || "—"}” · no findings</span></div>
              </div>
              <div className="form-actions">
                <button className="button button--secondary" type="button" onClick={() => setEditing(false)}>Cancel</button>
                <button className="button" type="submit">Save configuration</button>
              </div>
            </form>
          ) : (
            <dl className="definition-list">
              <div><dt>Schema version</dt><dd><code>1</code></dd></div>
              <div><dt>Display label</dt><dd>{label}</dd></div>
              <div><dt>Validation</dt><dd><StatusBadge tone="success">Valid</StatusBadge> Release 1.0.0 · no findings</dd></div>
              <div><dt>Authoritative validator</dt><dd>Scoped Records configuration contract</dd></div>
            </dl>
          )}
          {saved && <div className="inline-notice is-success"><Check size={18} /><div><strong>Configuration saved</strong><span>Core and module readback now agree on the normalized value.</span></div></div>}
        </section>
        <aside className="detail-card lifecycle-card">
          <div className="detail-card__heading"><div><h2>Application state</h2><p>Enablement remains separate from configuration and navigation.</p></div></div>
          <div className="state-line"><span>Configured</span><StatusBadge tone="success">Valid</StatusBadge></div>
          <div className="state-line"><span>Module health</span><StatusBadge tone="success">Healthy</StatusBadge></div>
          <div className="state-line"><span>Navigation</span><StatusBadge>Visible</StatusBadge></div>
          <div className="enablement-control">
            <div><strong>Product route enabled</strong><span>{enabled ? "Authorized users can open the module." : "Configuration and diagnostics remain available."}</span></div>
            <button type="button" role="switch" aria-checked={enabled} className={enabled ? "switch is-on" : "switch"} onClick={() => setEnabled(!enabled)}><span></span></button>
          </div>
          <button className="button button--secondary button--full" type="button" onClick={() => navigate("diagnostics")}><HeartPulse size={16} /> Open health and diagnostics</button>
        </aside>
      </div>
    </section>
  );
}

function RecordsScreen({ navigate }) {
  const [query, setQuery] = useState("");
  const visible = useMemo(() => records.filter((record) => `${record.label} ${record.owner} ${record.id}`.toLowerCase().includes(query.toLowerCase())), [query]);
  return (
    <section className="route-panel">
      <Breadcrumb items={["Scoped Records"]} />
      <PageHeading
        title="Scoped Records"
        description="Organization-owned reference records available within your assigned read scope."
        action={<button className="button" type="button" onClick={() => navigate("create")}><Plus size={17} /> New Record</button>}
      />
      <div className="scope-summary">
        <ShieldCheck size={18} />
        <div><strong>Read access across 3 Organization subtrees</strong><span>North Region · West Region · Central Region</span></div>
        <button className="button button--quiet" type="button">View access</button>
      </div>
      <div className="table-toolbar">
        <label className="table-search"><Search size={16} /><input type="search" value={query} onChange={(event) => setQuery(event.target.value)} placeholder="Search record label, ID, or Organization" /></label>
        <select aria-label="Filter by Organization"><option>All accessible Organizations</option><option>North Region</option><option>West Region</option><option>Central Region</option></select>
      </div>
      <div className="data-table-wrap">
        <table className="data-table">
          <thead><tr><th>Record</th><th>Organization owner</th><th>Updated</th><th>Authority</th></tr></thead>
          <tbody>
            {visible.map((record, index) => (
              <tr key={record.id}>
                <th><button type="button" className="link-button stacked" onClick={() => navigate("record")}><strong>{record.label}</strong><code>{record.id}</code></button></th>
                <td>{record.owner}</td><td>{record.updated}</td>
                <td><StatusBadge tone={index === 1 ? "info" : "success"}>{index === 1 ? "Read" : "Read · Manage"}</StatusBadge></td>
              </tr>
            ))}
          </tbody>
        </table>
        {visible.length === 0 && <div className="empty-state">No records match this search within your accessible Organizations.</div>}
      </div>
      <div className="pagination"><span>Showing 1-{visible.length} of {visible.length} records</span><span>Rows <strong>10</strong> <ChevronLeft size={16} /> Page 1 of 1 <ChevronRight size={16} /></span></div>
    </section>
  );
}

function RecordDetailScreen({ navigate }) {
  return (
    <section className="route-panel">
      <Breadcrumb items={["Scoped Records", "SR-1048"]} />
      <PageHeading
        title="North intake review"
        description="SR-1048"
        action={<div className="button-row"><button className="button button--secondary" type="button" onClick={() => navigate("records")}>Back to Records</button><button className="button" type="button" onClick={() => navigate("edit")}><Pencil size={16} /> Edit Record</button></div>}
      />
      <div className="record-detail-grid">
        <section className="detail-card">
          <div className="detail-card__heading"><div><h2>Record</h2><p>Product data owned by the Scoped Records Module Instance.</p></div><StatusBadge tone="success">Read · Manage</StatusBadge></div>
          <dl className="definition-list">
            <div><dt>Record ID</dt><dd><code>SR-1048</code></dd></div>
            <div><dt>Label</dt><dd>North intake review</dd></div>
            <div><dt>Organization owner</dt><dd>North Region <code>org_north</code></dd></div>
            <div><dt>Created</dt><dd>Jul 19, 2026 · 9:14 AM</dd></div>
            <div><dt>Last updated</dt><dd>Jul 23, 2026 · 2:18 PM</dd></div>
          </dl>
        </section>
        <aside className="detail-card">
          <div className="detail-card__heading"><div><h2>Authorization context</h2><p>Current Core decision for this module action.</p></div></div>
          <div className="auth-context">
            <div><span>Capability</span><code>scoped_records:read</code></div>
            <div><span>Authorized subtree</span><strong>North Region and descendants</strong></div>
            <div><span>Decision freshness</span><StatusBadge tone="success">Current · 42s remaining</StatusBadge></div>
            <div><span>Presenting service</span><code>tessara.reference.scoped-records</code></div>
          </div>
          <div className="inline-notice is-info"><ShieldCheck size={18} /><div><strong>Core credentials are not shared</strong><span>This module received only a short-lived, audience-bound decision.</span></div></div>
        </aside>
      </div>
    </section>
  );
}

function RecordFormScreen({ navigate, mode }) {
  const [owner, setOwner] = useState("North Region");
  const [saved, setSaved] = useState(false);
  return (
    <section className="route-panel">
      <Breadcrumb items={["Scoped Records", mode === "edit" ? "SR-1048" : "New Record", mode === "edit" ? "Edit" : "Create"]} />
      <PageHeading title={mode === "edit" ? "Edit Record" : "New Record"} description="Manage authority is checked against the selected Organization subtree when saved." />
      <form className="detail-card form-workspace" onSubmit={(event) => { event.preventDefault(); setSaved(true); }}>
        <div className="detail-card__heading"><div><h2>Record details</h2><p>Fields and validation belong to Scoped Records.</p></div>{mode === "edit" && <code>SR-1048</code>}</div>
        <div className="form-grid">
          <label className="form-field form-field--wide"><span>Label</span><input required defaultValue={mode === "edit" ? "North intake review" : ""} placeholder="Enter a clear record label" /></label>
          <label className="form-field form-field--wide"><span>Organization owner</span><select value={owner} onChange={(event) => { setOwner(event.target.value); setSaved(false); }}><option>North Region</option><option>West Region</option><option>Central Region</option></select></label>
        </div>
        {owner === "West Region" ? (
          <div className="inline-notice is-danger"><LockKeyhole size={18} /><div><strong>Manage authority is not available for West Region</strong><span>You may read West Region records, but cannot create or move records into that subtree.</span></div></div>
        ) : (
          <div className="validation-preview"><ShieldCheck size={19} /><div><strong>Manage authority confirmed</strong><span>{owner} and descendants · decision valid for this save only</span></div></div>
        )}
        {saved && <div className="inline-notice is-success"><Check size={18} /><div><strong>{mode === "edit" ? "Record updated" : "Record created"}</strong><span>The mutation authorization was consumed with this save.</span></div></div>}
        <div className="form-actions">
          <button className="button button--secondary" type="button" onClick={() => navigate(mode === "edit" ? "record" : "records")}>Cancel</button>
          <button className="button" type="submit" disabled={owner === "West Region"}>{mode === "edit" ? "Save Record" : "Create Record"}</button>
        </div>
      </form>
    </section>
  );
}

function DiagnosticsScreen() {
  const [section, setSection] = useState("health");
  return (
    <section className="route-panel">
      <Breadcrumb items={["Scoped Records", "Health & diagnostics"]} />
      <PageHeading title="Scoped Records health" description="Module-owned operational detail with Core installation context." action={<button className="button button--secondary" type="button"><RefreshCw size={16} /> Refresh status</button>} />
      <div className="tabs-list">
        <button type="button" className={section === "health" ? "tabs-trigger is-active" : "tabs-trigger"} onClick={() => setSection("health")}>Health</button>
        <button type="button" className={section === "diagnostics" ? "tabs-trigger is-active" : "tabs-trigger"} onClick={() => setSection("diagnostics")}>Diagnostics</button>
      </div>
      {section === "health" ? (
        <div className="diagnostic-grid">
          {[
            ["Readiness", "Passing", "Module can serve authorized product requests."],
            ["Liveness", "Passing", "Process heartbeat observed 18 seconds ago."],
            ["Configuration", "Valid", "Schema v1 · no findings."],
            ["Core authorization", "Connected", "Last decision exchange completed in 24 ms."],
          ].map(([name, status, detail]) => (
            <article className="metric-card" key={name}><Activity size={20} /><div><h2>{name}</h2><strong>{status}</strong><small>{detail}</small></div></article>
          ))}
        </div>
      ) : (
        <div className="record-detail-grid diagnostics-detail-grid">
          <section className="detail-card">
            <div className="detail-card__heading"><div><h2>Diagnostic context</h2><p>Shareable values are sanitized and contain no claim secrets or Core credentials.</p></div></div>
            <dl className="definition-list">
              <div><dt>Module version</dt><dd>1.0.0</dd></div>
              <div><dt>Module Instance</dt><dd><code>0c840876…6057f</code></dd></div>
              <div><dt>Database binding</dt><dd><code>tessara_module_scoped_records</code></dd></div>
              <div><dt>Authorization revision</dt><dd><code>auth:42</code></dd></div>
              <div><dt>Organization revision</dt><dd><code>org:17</code></dd></div>
            </dl>
          </section>
          <aside className="detail-card">
            <div className="detail-card__heading"><div><h2>Recent findings</h2><p>Stable codes from module-owned validation and health checks.</p></div></div>
            <div className="empty-state compact-empty"><Check size={22} /><strong>No active findings</strong><span>All required contracts and probes currently pass.</span></div>
            <button className="button button--secondary button--full" type="button">Download sanitized diagnostics</button>
          </aside>
        </div>
      )}
    </section>
  );
}

function StatesScreen({ navigate }) {
  const [state, setState] = useState("denied");
  const states = {
    denied: {
      icon: LockKeyhole, tone: "danger", eyebrow: "Scoped action unavailable", title: "You can’t manage this record",
      message: "Your current access permits reading records in West Region, but not creating or changing them.",
      detail: "Ask an administrator for Scoped Records manage access to this Organization subtree.",
      action: "Back to Records",
    },
    stale: {
      icon: RefreshCw, tone: "warning", eyebrow: "Authorization changed", title: "Refresh your access",
      message: "Your role or Organization access changed after this screen was opened.",
      detail: "No changes were saved. Refresh to request a current authorization decision from Core.",
      action: "Refresh access",
    },
    disabled: {
      icon: Blocks, tone: "info", eyebrow: "Module disabled", title: "Scoped Records is not serving product routes",
      message: "Configuration, health, and diagnostics remain available to authorized administrators.",
      detail: "Enable the module from Module Management after reviewing its current health.",
      action: "Open Module Management",
    },
    degraded: {
      icon: HeartPulse, tone: "warning", eyebrow: "Module degraded", title: "Scoped Records needs attention",
      message: "The module is reachable, but its readiness check reports an authorization dependency timeout.",
      detail: "Existing data remains isolated and retained. Review diagnostics before retrying.",
      action: "Open diagnostics",
    },
  };
  const current = states[state];
  const StateIcon = current.icon;
  return (
    <section className="route-panel">
      <Breadcrumb items={["Scoped Records", "State treatments"]} />
      <PageHeading title="Scoped Records state treatments" description="Review-only collection of the bounded denial, stale, disabled, and degraded states." />
      <div className="segmented state-selector">
        {Object.keys(states).map((key) => <button type="button" key={key} className={state === key ? "is-active" : ""} onClick={() => setState(key)}>{key[0].toUpperCase() + key.slice(1)}</button>)}
      </div>
      <section className={`state-treatment is-${current.tone}`}>
        <div className="state-treatment__icon"><StateIcon size={30} /></div>
        <div className="state-treatment__body">
          <div className="eyebrow">{current.eyebrow}</div>
          <h2>{current.title}</h2>
          <p>{current.message}</p>
          <span>{current.detail}</span>
          <div className="button-row">
            <button className="button" type="button" onClick={() => state === "degraded" ? navigate("diagnostics") : state === "disabled" ? navigate("module") : navigate("records")}>{current.action}</button>
            {state !== "denied" && <button className="button button--secondary" type="button" onClick={() => navigate("records")}>Return to Records</button>}
          </div>
        </div>
        <aside className="state-treatment__context">
          <strong>What remains protected</strong>
          <span><Check size={15} /> No record existence disclosed before authorization</span>
          <span><Check size={15} /> Core session credentials not shared</span>
          <span><Check size={15} /> Module Instance identity and data retained</span>
          <span><Check size={15} /> Correlation context available to diagnostics</span>
        </aside>
      </section>
    </section>
  );
}
