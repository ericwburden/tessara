
      let token = window.sessionStorage.getItem('tessara.devToken');
      let currentAccount = null;
      const THEME_STORAGE_KEY = 'tessara.themePreference';
      const LIGHT_THEME_COLOR = '#F8FAFC';
      const DARK_THEME_COLOR = '#0F172A';
      const selections = {};
      const page = {
        key: document.body.dataset.pageKey || 'home',
        recordId: document.body.dataset.recordId || '',
        search: new URLSearchParams(window.location.search)
      };
      let reportFormState = {
        forms: [],
        datasets: [],
        bindings: []
      };
      let userFormState = {
        roles: []
      };
      let roleFormState = {
        capabilities: []
      };
      let nodeTypeFormState = {
        nodeTypes: [],
        excludeId: '',
        parentSelection: [],
        childSelection: [],
        metadataFields: [],
        activeMetadataIndex: -1
      };
      let renderedResponseForm = null;
      let currentResponseDetail = null;
      let currentDelegateContext = window.sessionStorage.getItem('tessara.delegateAccountId');
      let formBuilderState = {
        form: null,
        renderedVersion: null,
        selectedVersionId: ''
      };
      const formActionLocks = new Set();

      function storedThemePreference() {
        try {
          const stored = window.localStorage.getItem(THEME_STORAGE_KEY);
          return stored === 'light' || stored === 'dark' || stored === 'system' ? stored : 'system';
        } catch (_error) {
          return 'system';
        }
      }

      function systemTheme() {
        return window.matchMedia && window.matchMedia('(prefers-color-scheme: dark)').matches
          ? 'dark'
          : 'light';
      }

      function syncThemeColorMeta(theme) {
        const element = document.querySelector('meta[name=\"theme-color\"]');
        if (element) {
          element.setAttribute('content', theme === 'dark' ? DARK_THEME_COLOR : LIGHT_THEME_COLOR);
        }
      }

      function syncThemeToggleState() {
        const preference = document.documentElement.dataset.themePreference || 'system';
        document.querySelectorAll('[data-theme-choice]').forEach((button) => {
          const active = button.dataset.themeChoice === preference;
          button.setAttribute('aria-pressed', String(active));
          button.dataset.active = active ? 'true' : 'false';
        });
      }

      function applyThemePreference(preference, persist = true) {
        const normalized = preference === 'light' || preference === 'dark' || preference === 'system'
          ? preference
          : 'system';
        const theme = normalized === 'system' ? systemTheme() : normalized;
        document.documentElement.dataset.themePreference = normalized;
        document.documentElement.dataset.theme = theme;
        syncThemeColorMeta(theme);
        syncThemeToggleState();
        if (persist) {
          try {
            window.localStorage.setItem(THEME_STORAGE_KEY, normalized);
          } catch (_error) {
          }
        }
      }

      function initThemeControls() {
        document.querySelectorAll('[data-theme-choice]').forEach((button) => {
          button.addEventListener('click', () => applyThemePreference(button.dataset.themeChoice || 'system'));
        });

        if (window.matchMedia) {
          const query = window.matchMedia('(prefers-color-scheme: dark)');
          const handleChange = () => {
            if ((document.documentElement.dataset.themePreference || 'system') === 'system') {
              applyThemePreference('system', false);
            }
          };
          if (query.addEventListener) {
            query.addEventListener('change', handleChange);
          } else if (query.addListener) {
            query.addListener(handleChange);
          }
        }

        applyThemePreference(
          document.documentElement.dataset.themePreference || storedThemePreference(),
          false
        );
      }

      function byId(id) {
        return document.getElementById(id);
      }

      function escapeHtml(value) {
        return String(value ?? '')
          .replaceAll('&', '&amp;')
          .replaceAll('<', '&lt;')
          .replaceAll('>', '&gt;')
          .replaceAll('\"', '&quot;')
          .replaceAll("'", '&#39;');
      }

      function show(value) {
        const output = byId('output');
        if (!output) return;
        output.textContent = typeof value === 'string' ? value : JSON.stringify(value, null, 2);
      }

      function setHtml(id, html) {
        const element = byId(id);
        if (element) element.innerHTML = html;
      }

      function currentPath() {
        return window.location.pathname;
      }

      function emptyState(message) {
        return `<p class=\"muted\">${escapeHtml(message)}</p>`;
      }

      function setLoginFeedback(message = '') {
        const element = byId('login-feedback');
        if (!element) return;
        element.textContent = message;
        element.classList.toggle('is-hidden', !message);
      }

      function recordCard(title, body, actions) {
        return `
          <article class=\"record-card card\">
            <div class=\"card-content\">
              <h4>${escapeHtml(title)}</h4>
              ${body}
              <div class=\"actions\">${actions}</div>
            </div>
          </article>
        `;
      }

      function detailSection(title, body) {
        return `
          <section class=\"detail-section box\">
            <h4>${escapeHtml(title)}</h4>
            ${body}
          </section>
        `;
      }

      function formatDateTime(value) {
        if (!value) return 'Not published';
        try {
          return new Intl.DateTimeFormat('en-US', {
            dateStyle: 'medium',
            timeStyle: 'short'
          }).format(new Date(value));
        } catch (_error) {
          return String(value);
        }
      }

      function formStatusLabel(status) {
        return String(status || 'unknown').replaceAll('_', ' ');
      }

      function setFormStatus(id, message = '', tone = 'muted') {
        const element = byId(id);
        if (!element) return;
        element.textContent = message;
        element.className = tone === 'error'
          ? 'notification is-danger is-light'
          : tone === 'success'
            ? 'notification is-success is-light'
            : 'muted';
      }

      async function runLockedFormAction(lockKey, action) {
        if (formActionLocks.has(lockKey)) {
          throw new Error('A form update is already in progress. Wait for the current request to finish.');
        }
        formActionLocks.add(lockKey);
        try {
          return await action();
        } finally {
          formActionLocks.delete(lockKey);
        }
      }

      function latestMatchingVersion(versions, predicate) {
        for (let index = versions.length - 1; index >= 0; index -= 1) {
          if (predicate(versions[index])) {
            return versions[index];
          }
        }
        return null;
      }

      function choosePreferredFormVersion(payload, preferredVersionId = '') {
        if (!payload?.versions?.length) return null;
        const explicit = preferredVersionId
          ? payload.versions.find((version) => version.id === preferredVersionId)
          : null;
        if (explicit) return explicit;
        return latestMatchingVersion(payload.versions, (version) => version.status === 'published')
          || payload.versions[payload.versions.length - 1];
      }

      function renderFormFieldTypeOptions(selectedValue = 'text') {
        return ['text', 'number', 'boolean', 'date', 'multi_choice']
          .map((fieldType) => `
            <option value=\"${escapeHtml(fieldType)}\" ${fieldType === selectedValue ? 'selected' : ''}>
              ${escapeHtml(formStatusLabel(fieldType))}
            </option>
          `)
          .join('');
      }

      function renderFormSectionOptions(sectionId = '') {
        return (formBuilderState.renderedVersion?.sections || [])
          .map((section) => `
            <option value=\"${escapeHtml(section.id)}\" ${section.id === sectionId ? 'selected' : ''}>
              ${escapeHtml(section.title)}
            </option>
          `)
          .join('');
      }

      function formVersionLabel(version) {
        return version?.version_label || version?.publish_preview?.version_label || 'Draft version';
      }

      function formVersionCompatibility(version) {
        if (version?.publish_preview?.compatibility_label) {
          return version.publish_preview.compatibility_label;
        }
        if (Number.isInteger(version?.version_major)) {
          return `Compatible with v${version.version_major}.x`;
        }
        return 'Compatibility line assigned on publish';
      }

      function formVersionSemanticBump(version) {
        return version?.semantic_bump || version?.publish_preview?.semantic_bump || '';
      }

      function formVersionLifecycleSummary(version) {
        if (version?.publish_preview) {
          const preview = version.publish_preview;
          return `
            <p class=\"muted\">Publish preview: ${escapeHtml(preview.semantic_bump)} -> ${escapeHtml(preview.version_label)}</p>
            <p class=\"muted\">${escapeHtml(preview.compatibility_label)}${preview.starts_new_major_line ? ' (new major line)' : ''}</p>
            ${preview.dependency_warnings.length
              ? `<ul class=\"app-list\">${preview.dependency_warnings.map((warning) => `<li>${escapeHtml(warning)}</li>`).join('')}</ul>`
              : '<p class=\"muted\">No direct-consumer warnings for this publish.</p>'}
          `;
        }
        const bump = formVersionSemanticBump(version);
        return `
          ${bump ? `<p class=\"muted\">Semantic bump: ${escapeHtml(bump)}</p>` : ''}
          <p class=\"muted\">${escapeHtml(formVersionCompatibility(version))}${version?.started_new_major_line ? ' (new major line)' : ''}</p>
        `;
      }

      function renderFormVersionCards(payload, selectedVersionId, editable) {
        return payload.versions.length
          ? payload.versions.map((version) => `
              <article class=\"record-card ${version.id === selectedVersionId ? 'compact-record-card' : ''}\">
                <h4>${escapeHtml(formVersionLabel(version))}</h4>
                <p class=\"muted\">Status: ${escapeHtml(formStatusLabel(version.status))}</p>
                ${formVersionSemanticBump(version)
                  ? `<p class=\"muted\">Semantic bump: ${escapeHtml(formVersionSemanticBump(version))}</p>`
                  : ''}
                <p class=\"muted\">Compatibility: ${escapeHtml(formVersionCompatibility(version))}</p>
                <p class=\"muted\">Published: ${escapeHtml(formatDateTime(version.published_at))}</p>
                <p class=\"muted\">Fields: ${escapeHtml(version.field_count)}</p>
                ${version.status === 'published' ? '<p class=\"muted\">Current published version.</p>' : ''}
                ${version.id === selectedVersionId ? '<p class=\"muted\">Selected version.</p>' : ''}
                <div class=\"actions\">
                  <button type=\"button\" onclick=\"previewFormVersion('${escapeHtml(version.id)}')\">${editable ? 'Open Workspace' : 'Preview'}</button>
                  ${editable && hasCapability('forms:write') && version.status === 'draft'
                    ? `<button type=\"button\" onclick=\"publishSelectedFormVersion('${escapeHtml(version.id)}')\">Publish</button>`
                    : ''}
                </div>
              </article>
            `).join('')
          : emptyState('No versions have been created for this form yet.');
      }

      function renderFormPreview(versionSummary, rendered) {
        const sections = rendered.sections.length
          ? rendered.sections.map((section) => `
              <section class=\"page-panel nested-form-panel\">
                <h3>${escapeHtml(section.title)}</h3>
                <p class=\"muted\">Section order: ${escapeHtml(section.position)}</p>
                <div class=\"record-list\">
                  ${section.fields.length
                    ? section.fields.map((field) => `
                        <article class=\"record-card compact-record-card\">
                          <h4>${escapeHtml(field.label)}</h4>
                          <p class=\"muted\">Key: ${escapeHtml(field.key)}</p>
                          <p class=\"muted\">Type: ${escapeHtml(formStatusLabel(field.field_type))}</p>
                          <p class=\"muted\">${field.required ? 'Required' : 'Optional'} field</p>
                          <p class=\"muted\">Option-set and lookup touchpoints remain read-only in this slice.</p>
                        </article>
                      `).join('')
                    : emptyState('No fields in this section yet.')}
                </div>
              </section>
            `).join('')
          : emptyState('No sections were added to this version yet.');

        return `
          <article class=\"record-card\">
            <h4>${escapeHtml(formVersionLabel(versionSummary))}</h4>
            <p class=\"muted\">Status: ${escapeHtml(formStatusLabel(versionSummary.status))}</p>
            <p class=\"muted\">Compatibility: ${escapeHtml(formVersionCompatibility(versionSummary))}</p>
            <p class=\"muted\">Published: ${escapeHtml(formatDateTime(versionSummary.published_at))}</p>
            ${formVersionLifecycleSummary(versionSummary)}
          </article>
          ${sections}
        `;
      }

      function renderFormWorkflowAttachments(payload) {
        const reports = payload.reports.length
          ? payload.reports.map((report) => `<li><a href=\"/app/reports/${report.id}\">${escapeHtml(report.name)}</a></li>`).join('')
          : '<li class=\"muted\">No related reports.</li>';
        const datasets = payload.dataset_sources.length
          ? payload.dataset_sources.map((dataset) => `<li>${escapeHtml(dataset.dataset_name)} (${escapeHtml(dataset.source_alias)}; ${escapeHtml(dataset.selection_rule)})</li>`).join('')
          : '<li class=\"muted\">No related dataset sources.</li>';

        return `
          ${detailSection('Related Reports', `<ul class=\"app-list\">${reports}</ul>`)}
          ${detailSection('Related Dataset Sources', `<ul class=\"app-list\">${datasets}</ul>`)}
        `;
      }

      function summaryRecords(payload) {
        return [
          ['Published forms', payload.published_form_versions],
          ['Draft submissions', payload.draft_submissions],
          ['Submitted submissions', payload.submitted_submissions],
          ['Datasets', payload.datasets],
          ['Reports', payload.reports],
          ['Aggregations', payload.aggregations],
          ['Dashboards', payload.dashboards],
          ['Charts', payload.charts]
        ];
      }

      function renderSummaryCards(payload) {
        return summaryRecords(payload).map(([label, count]) => `
          <article class=\"card summary-card\">
            <div class=\"card-content\">
              <h3>${escapeHtml(label)}</h3>
              <p>${escapeHtml(count)}</p>
            </div>
          </article>
        `).join('');
      }

      function updateSessionStatus(account = null) {
        const element = byId('session-status');
        if (!element) return;
        if (!token) {
          element.textContent = 'Not signed in.';
          return;
        }
        element.textContent = account?.email
          ? `Signed in as ${account.email} (${String(account.ui_access_profile || '').replaceAll('_', ' ')}).`
          : 'Authenticated for local testing.';
      }

      function renderCurrentUserSummary(account = null) {
        const html = !account
          ? '<p class=\"muted\">Sign in to load account context.</p>'
          : `
              <article class=\"selection-item box\">
                <h3>${escapeHtml(account.display_name || account.email)}</h3>
                <p>${escapeHtml(account.email)}</p>
                <p class=\"muted\">Profile: ${escapeHtml(String(account.ui_access_profile || '').replaceAll('_', ' '))}</p>
                <p class=\"muted\">Roles: ${escapeHtml((account.roles || []).join(', ') || 'None')}</p>
                <p class=\"muted\">Capabilities: ${escapeHtml((account.capabilities || []).length)}</p>
                <p class=\"muted\">Scope nodes: ${escapeHtml((account.scope_nodes || []).length)}</p>
                <p class=\"muted\">Delegations: ${escapeHtml((account.delegations || []).length)}</p>
              </article>
            `;
        for (const id of ['current-user-summary', 'home-current-user-summary']) {
          setHtml(id, html);
        }
      }

      function selectRecord(kind, label, id) {
        selections[kind] = { label, id };
        renderSelections();
      }

      function renderSelections() {
        const entries = Object.entries(selections);
        const html = entries.length
          ? entries.map(([kind, record]) => `
              <article class=\"selection-item box\">
                <h3>${escapeHtml(kind)}</h3>
                <p>${escapeHtml(record.label)}</p>
              </article>
            `).join('')
          : '<p class=\"muted\">No records selected yet.</p>';
        for (const id of ['selection-state', 'home-selection-state']) {
          setHtml(id, html);
        }
      }

      function isAdmin() {
        return currentAccount?.ui_access_profile === 'admin';
      }

      function isOperator() {
        return currentAccount?.ui_access_profile === 'operator';
      }

      function isResponseUser() {
        return currentAccount?.ui_access_profile === 'response_user';
      }

      function canUseDelegatedResponses() {
        return hasCapability('submissions:write');
      }

      function hasCapability(capability) {
        if (!currentAccount) return false;
        return currentAccount.capabilities?.includes('admin:all') || currentAccount.capabilities?.includes(capability);
      }

      function setDelegateContext(accountId) {
        currentDelegateContext = accountId || '';
        if (currentDelegateContext) {
          window.sessionStorage.setItem('tessara.delegateAccountId', currentDelegateContext);
        } else {
          window.sessionStorage.removeItem('tessara.delegateAccountId');
        }
      }

      function delegateQuerySuffix() {
        return currentDelegateContext ? `delegate_account_id=${encodeURIComponent(currentDelegateContext)}` : '';
      }

      function withDelegateQuery(path) {
        const suffix = delegateQuerySuffix();
        if (!suffix) return path;
        return path.includes('?') ? `${path}&${suffix}` : `${path}?${suffix}`;
      }

      async function request(path, options = {}) {
        const headers = { ...(options.headers || {}) };
        if (token) {
          headers.Authorization = `Bearer ${token}`;
        }
        const response = await fetch(path, { ...options, headers });
        const text = await response.text();
        let payload = null;
        try {
          payload = text ? JSON.parse(text) : null;
        } catch (_error) {
          payload = null;
        }
        if (!response.ok) {
          throw new Error((payload && payload.error) || text || `Request failed: ${response.status}`);
        }
        return payload;
      }

      function openLogin() {
        redirect('/app/login');
      }

      async function login(silent = false, email = 'admin@tessara.local', password = 'tessara-dev-admin') {
        const payload = await request('/api/auth/login', {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({
            email,
            password
          })
        });
        token = payload.token;
        window.sessionStorage.setItem('tessara.devToken', token);
        currentAccount = await request('/api/me');
        if (!canUseDelegatedResponses()) {
          setDelegateContext('');
        } else if (!currentDelegateContext) {
          setDelegateContext(currentAccount.account_id);
        }
        updateSessionStatus(currentAccount);
        renderCurrentUserSummary(currentAccount);
        applyRoleVisibility();
        if (!silent) show({ authenticated: true, account: currentAccount });
        return currentAccount;
      }

      async function ensureAuthenticated() {
        if (!token) {
          throw new Error('Sign in required.');
        }
        return token;
      }

      function logout() {
        token = null;
        currentAccount = null;
        setDelegateContext('');
        window.sessionStorage.removeItem('tessara.devToken');
        updateSessionStatus();
        renderCurrentUserSummary();
        show({ authenticated: false });
        redirect('/app/login');
      }

      async function bootstrapCurrentAccount() {
        if (!token) {
          currentAccount = null;
          updateSessionStatus();
          renderCurrentUserSummary();
          applyRoleVisibility();
          return null;
        }

        try {
          currentAccount = await request('/api/me');
          if (!canUseDelegatedResponses()) {
            setDelegateContext('');
          } else if (
            currentDelegateContext
            && currentDelegateContext !== currentAccount.account_id
            && !currentAccount.delegations.some((delegate) => delegate.account_id === currentDelegateContext)
          ) {
            setDelegateContext(currentAccount.account_id);
          } else if (!currentDelegateContext) {
            setDelegateContext(currentAccount.account_id);
          }
          updateSessionStatus(currentAccount);
          renderCurrentUserSummary(currentAccount);
          applyRoleVisibility();
          return currentAccount;
        } catch (error) {
          token = null;
          currentAccount = null;
          window.sessionStorage.removeItem('tessara.devToken');
          setDelegateContext('');
          updateSessionStatus();
          renderCurrentUserSummary();
          applyRoleVisibility();
          throw error;
        }
      }

      async function loadCurrentUser() {
        try {
          await ensureAuthenticated();
          const payload = await bootstrapCurrentAccount();
          show(payload);
        } catch (error) {
          show(error.message);
        }
      }

      async function loadAppSummary() {
        try {
          await ensureAuthenticated();
          const payload = await request('/api/app/summary');
          setHtml('home-summary-cards', renderSummaryCards(payload));
          show(payload);
        } catch (error) {
          show(error.message);
        }
      }

      function isPublicReadableCapability(requiredCapability) {
        return ['hierarchy:read', 'forms:read', 'reports:read'].includes(requiredCapability);
      }

      function applyRoleVisibility() {
        const routeCapabilities = {
          '/app': null,
          '/app/organization': 'hierarchy:read',
          '/app/forms': 'forms:read',
          '/app/responses': 'submissions:write',
          '/app/reports': 'reports:read',
          '/app/dashboards': 'reports:read',
          '/app/administration': 'admin:all',
          '/app/migration': 'admin:all'
        };
        for (const link of document.querySelectorAll('.app-nav a')) {
          const href = link.getAttribute('href') || '';
          const requiredCapability = routeCapabilities[href];
          const visible = !currentAccount
            ? href === '/app' || isPublicReadableCapability(requiredCapability)
            : !requiredCapability || hasCapability(requiredCapability);
          link.style.display = visible ? '' : 'none';
        }
      }

      function canAccessCurrentPage() {
        if (page.key === 'login') return true;
        const pageCapabilities = {
          home: null,
          administration: 'admin:all',
          migration: 'admin:all',
          'organization-list': 'hierarchy:read',
          'organization-detail': 'hierarchy:read',
          'organization-create': 'admin:all',
          'organization-edit': 'admin:all',
          'form-list': 'forms:read',
          'form-detail': 'forms:read',
          'form-create': 'admin:all',
          'form-edit': 'admin:all',
          'response-list': 'submissions:write',
          'response-detail': 'submissions:write',
          'response-create': 'submissions:write',
          'response-edit': 'submissions:write',
          'report-list': 'reports:read',
          'report-detail': 'reports:read',
          'report-create': 'admin:all',
          'report-edit': 'admin:all',
          'dashboard-list': 'reports:read',
          'dashboard-detail': 'reports:read',
          'dashboard-create': 'admin:all',
          'dashboard-edit': 'admin:all',
          'user-list': 'admin:all',
          'user-detail': 'admin:all',
          'user-create': 'admin:all',
          'user-edit': 'admin:all',
          'user-access': 'admin:all',
          'node-type-list': 'admin:all',
          'node-type-detail': 'admin:all',
          'node-type-create': 'admin:all',
          'node-type-edit': 'admin:all',
          'role-list': 'admin:all',
          'role-detail': 'admin:all',
          'role-create': 'admin:all',
          'role-edit': 'admin:all'
        };
        const requiredCapability = pageCapabilities[page.key];
        if (!requiredCapability) return true;
        if (!currentAccount) return isPublicReadableCapability(requiredCapability);
        return hasCapability(requiredCapability);
      }

      function renderAccessState(title, message) {
        const appMain = document.querySelector('.app-main');
        if (!appMain) return;
        appMain.innerHTML = `
          <section class="app-screen entity-page">
            <p class="eyebrow">Access</p>
            <div class="page-title-row">
              <div>
                <h2>${escapeHtml(title)}</h2>
                <p class="muted">${escapeHtml(message)}</p>
              </div>
            </div>
          </section>
          <section class="app-screen page-panel">
            <div class="actions">
              ${currentAccount ? '<a class="button-link" href="/app">Go Home</a>' : '<a class="button-link" href="/app/login">Sign In</a>'}
            </div>
          </section>
        `;
      }

      function setSelectOptions(id, options, blankLabel = '') {
        const element = byId(id);
        if (!element) return;
        const blank = blankLabel ? `<option value=\"\">${escapeHtml(blankLabel)}</option>` : '';
        element.innerHTML = blank + options.map((option) => `
          <option value=\"${escapeHtml(option.value)}\">${escapeHtml(option.label)}</option>
        `).join('');
      }

      function redirect(path) {
        window.location.assign(path);
      }

      function fieldInputId(field) {
        return `response-field-${field.id}`;
      }

      function renderFieldInput(field) {
        const required = field.required ? ' required' : '';
        if (field.field_type === 'boolean') {
          return `<input id=\"${escapeHtml(fieldInputId(field))}\" type=\"checkbox\"${required}>`;
        }
        const inputType = field.field_type === 'number'
          ? 'number'
          : field.field_type === 'date'
            ? 'date'
            : 'text';
        const placeholder = field.field_type === 'multi_choice'
          ? 'Comma-separated choices'
          : field.label;
        return `<input class=\"input\" id=\"${escapeHtml(fieldInputId(field))}\" type=\"${inputType}\" placeholder=\"${escapeHtml(placeholder)}\"${required}>`;
      }

      function collectRenderedResponseValues() {
        const values = {};
        if (!renderedResponseForm) return values;
        for (const section of renderedResponseForm.sections) {
          for (const field of section.fields) {
            const element = byId(fieldInputId(field));
            if (!element) continue;
            if (field.field_type === 'boolean') {
              values[field.key] = element.checked;
              continue;
            }
            const raw = element.value.trim();
            if (raw === '') continue;
            if (field.field_type === 'number') {
              values[field.key] = Number(raw);
            } else if (field.field_type === 'multi_choice') {
              values[field.key] = raw.split(',').map((item) => item.trim()).filter(Boolean);
            } else {
              values[field.key] = raw;
            }
          }
        }
        return values;
      }

      function prefillRenderedResponseValues(detail) {
        if (!renderedResponseForm || !detail) return;
        const valuesByKey = Object.fromEntries(
          (detail.values || [])
            .filter((item) => item.value !== null)
            .map((item) => [item.key, item.value])
        );
        for (const section of renderedResponseForm.sections) {
          for (const field of section.fields) {
            const element = byId(fieldInputId(field));
            const value = valuesByKey[field.key];
            if (!element || value === undefined || value === null) continue;
            if (field.field_type === 'boolean') {
              element.checked = Boolean(value);
            } else if (Array.isArray(value)) {
              element.value = value.join(', ');
            } else {
              element.value = String(value);
            }
          }
        }
      }

      function validateRenderedResponseValues(values) {
        if (!renderedResponseForm) return;
        const missing = renderedResponseForm.sections
          .flatMap((section) => section.fields)
          .filter((field) => field.required)
          .filter((field) => {
            const value = values[field.key];
            return value === undefined
              || value === null
              || value === ''
              || (Array.isArray(value) && value.length === 0);
          })
          .map((field) => field.label);
        if (missing.length > 0) {
          throw new Error(`Required fields missing: ${missing.join(', ')}`);
        }
      }

      async function initHomePage() {
        updateSessionStatus();
        renderSelections();
        try {
          await ensureAuthenticated();
          await Promise.all([bootstrapCurrentAccount(), loadAppSummary()]);
        } catch (error) {
          show(error.message);
        }
      }

      function renderDelegateContextSwitcher(targetId) {
        const container = byId(targetId);
        if (!container) return;
        if (!canUseDelegatedResponses()) {
          container.innerHTML = '';
          return;
        }
        const options = [
          {
            account_id: currentAccount.account_id,
            display_name: currentAccount.display_name || currentAccount.email
          },
          ...(currentAccount.delegations || [])
        ];
        if (options.length <= 1) {
          container.innerHTML = '';
          return;
        }
        container.innerHTML = `
          <section class="app-screen page-panel compact-panel">
            <h3>Delegated Response Context</h3>
            <p class="muted">Choose which delegated account's assigned responses you are currently viewing.</p>
            <div class="form-field">
              <label for="delegate-context-select">Acting For</label>
              <select id="delegate-context-select">
                ${options.map((option) => `<option value="${escapeHtml(option.account_id)}" ${option.account_id === currentDelegateContext ? 'selected' : ''}>${escapeHtml(option.display_name)}</option>`).join('')}
              </select>
            </div>
          </section>
        `;
        const select = byId('delegate-context-select');
        if (select) {
          select.onchange = () => {
            setDelegateContext(select.value);
            initPage().catch((error) => show(error.message));
          };
        }
      }

      async function initLoginPage() {
        updateSessionStatus();
        setLoginFeedback('');
        const form = byId('login-form');
        if (!form) return;
        form.onsubmit = async (event) => {
          event.preventDefault();
          try {
            setLoginFeedback('');
            const account = await login(
              false,
              byId('login-email').value.trim(),
              byId('login-password').value
            );
            redirect(account.ui_access_profile === 'response_user' ? '/app/responses' : '/app');
          } catch (error) {
            setLoginFeedback(error.message || 'Sign in failed.');
            byId('login-password').value = '';
            byId('login-password').focus();
          }
        };
      }

      function userStatusLabel(user) {
        return user?.is_active ? 'Active' : 'Inactive';
      }

      function renderUserRoleOptions(selectedRoleIds = []) {
        const html = userFormState.roles.length
          ? userFormState.roles.map((role) => `
              <label class="checkbox-label" for="user-role-${escapeHtml(role.id)}">
                <input id="user-role-${escapeHtml(role.id)}" class="user-role-checkbox" type="checkbox" value="${escapeHtml(role.id)}" ${selectedRoleIds.includes(role.id) ? 'checked' : ''}>
                ${escapeHtml(role.name)}
              </label>
            `).join('')
          : '<p class="muted">No roles are available.</p>';
        setHtml('user-role-options', html);
      }

      function collectSelectedUserRoleIds() {
        return Array.from(document.querySelectorAll('.user-role-checkbox:checked'))
          .map((input) => input.value);
      }

      async function loadUsersList() {
        try {
          await ensureAuthenticated();
          const payload = await request('/api/admin/users');
          const html = payload.length
            ? payload.map((user) => recordCard(
                user.display_name,
                `<p>${escapeHtml(user.email)}</p><p class=\"muted\">${escapeHtml(userStatusLabel(user))}</p><p class=\"muted\">Roles: ${escapeHtml(user.roles.map((role) => role.name).join(', ') || 'None')}</p>`,
                `<a class=\"button-link\" href=\"/app/administration/users/${user.id}\">View</a><a class=\"button-link\" href=\"/app/administration/users/${user.id}/access\">Access</a><a class=\"button-link\" href=\"/app/administration/users/${user.id}/edit\">Edit</a>`
              )).join('')
            : emptyState('No user accounts found.');
          setHtml('user-list', html);
          show(payload);
        } catch (error) {
          setHtml('user-list', emptyState(error.message));
        }
      }

      async function loadUserDetail(id) {
        try {
          await ensureAuthenticated();
          const payload = await request(`/api/admin/users/${id}`);
          selectRecord('user', payload.display_name || payload.email, payload.id);
          const roles = payload.roles.length
            ? payload.roles.map((role) => `<li>${escapeHtml(role.name)}</li>`).join('')
            : '<li class=\"muted\">No roles assigned.</li>';
          const scopeNodes = payload.scope_nodes.length
            ? payload.scope_nodes.map((node) => `<li>${escapeHtml(node.node_name)} <span class=\"muted\">(${escapeHtml(node.node_type_name)})</span></li>`).join('')
            : '<li class=\"muted\">No scope nodes assigned.</li>';
          const delegations = payload.delegations.length
            ? payload.delegations.map((account) => `<li>${escapeHtml(account.display_name)} <span class=\"muted\">${escapeHtml(account.email)}</span></li>`).join('')
            : '<li class=\"muted\">No delegated accounts.</li>';
          const delegatedBy = payload.delegated_by.length
            ? payload.delegated_by.map((account) => `<li>${escapeHtml(account.display_name)} <span class=\"muted\">${escapeHtml(account.email)}</span></li>`).join('')
            : '<li class=\"muted\">No delegators.</li>';
          const statusNote = payload.id === currentAccount?.account_id ? 'This is the currently signed-in account.' : 'Use Edit to change status or password.';
          const accessActions = isAdmin()
            ? `<div class="actions"><a class="button-link" href="/app/administration/users/${payload.id}/access">Manage Access</a></div>`
            : '';
          setHtml('user-detail', `
            ${detailSection('Summary', `<p>${escapeHtml(payload.display_name)}</p><p>${escapeHtml(payload.email)}</p><p class=\"muted\">Status: ${escapeHtml(userStatusLabel(payload))}</p><p class=\"muted\">UI Profile: ${escapeHtml(String(payload.ui_access_profile || '').replaceAll('_', ' '))}</p><p class=\"muted\">${escapeHtml(statusNote)}</p>`)}
            ${detailSection('Assigned Roles', `<ul class=\"app-list\">${roles}</ul>`)}
            ${detailSection('Effective Capabilities', `<p>${escapeHtml((payload.capabilities || []).join(', ') || 'None')}</p>`)}
            ${detailSection('Scope Nodes', `<ul class=\"app-list\">${scopeNodes}</ul>`)}
            ${detailSection('Can Act For', `<ul class=\"app-list\">${delegations}</ul>`)}
            ${detailSection('Delegated By', `<ul class=\"app-list\">${delegatedBy}</ul>`)}
            ${accessActions}
          `);
          show(payload);
        } catch (error) {
          setHtml('user-detail', emptyState(error.message));
        }
      }

      async function initUserForm(mode, id) {
        try {
          await ensureAuthenticated();
          userFormState = {
            roles: await request('/api/admin/roles')
          };
          renderUserRoleOptions([]);
          byId('user-is-active').checked = true;
          if (mode === 'edit' && id) {
            const payload = await request(`/api/admin/users/${id}`);
            byId('user-display-name').value = payload.display_name || '';
            byId('user-email').value = payload.email || '';
            byId('user-is-active').checked = Boolean(payload.is_active);
            renderUserRoleOptions(payload.roles.map((role) => role.id));
          }
          const form = byId('user-form');
          if (form) {
            form.onsubmit = async (event) => {
              event.preventDefault();
              const payload = {
                display_name: byId('user-display-name').value.trim(),
                email: byId('user-email').value.trim(),
                password: byId('user-password').value,
                is_active: byId('user-is-active').checked,
                role_ids: collectSelectedUserRoleIds()
              };
              const requestBody = mode === 'create'
                ? payload
                : { ...payload, password: payload.password.trim() ? payload.password : null };
              const response = await request(
                mode === 'create' ? '/api/admin/users' : `/api/admin/users/${id}`,
                {
                  method: mode === 'create' ? 'POST' : 'PUT',
                  headers: { 'Content-Type': 'application/json' },
                  body: JSON.stringify(requestBody)
                }
              );
              redirect(`/app/administration/users/${response.id}`);
            };
          }
        } catch (error) {
          setHtml('user-role-options', emptyState(error.message));
          show(error.message);
        }
      }

      function normalizeFilterValue(value) {
        return String(value || '').trim().toLowerCase();
      }

      function renderRoleCapabilityOptions(selectedCapabilityIds = []) {
        const filter = normalizeFilterValue(byId('role-capability-filter')?.value);
        const capabilities = roleFormState.capabilities.filter((capability) => {
          if (!filter) return true;
          return normalizeFilterValue(capability.key).includes(filter)
            || normalizeFilterValue(capability.description).includes(filter);
        });
        const html = capabilities.length
          ? capabilities.map((capability) => `
              <tr>
                <td>
                  <input id="role-capability-${escapeHtml(capability.id)}" class="role-capability-checkbox" type="checkbox" value="${escapeHtml(capability.id)}" ${selectedCapabilityIds.includes(capability.id) ? 'checked' : ''}>
                </td>
                <td><label for="role-capability-${escapeHtml(capability.id)}">${escapeHtml(capability.key)}</label></td>
                <td>${escapeHtml(capability.description || '')}</td>
              </tr>
            `).join('')
          : '<tr><td colspan="3" class="muted">No capabilities match the current filter.</td></tr>';
        setHtml('role-capability-options', html);
      }

      function collectSelectedCapabilityIds() {
        return Array.from(document.querySelectorAll('.role-capability-checkbox:checked'))
          .map((input) => input.value);
      }

      async function loadRolesList() {
        try {
          await ensureAuthenticated();
          const payload = await request('/api/admin/roles');
          const html = payload.length
            ? payload.map((role) => recordCard(
                role.name,
                `<p class=\"muted\">Capabilities: ${escapeHtml(role.capability_count)}</p><p class=\"muted\">Assigned accounts: ${escapeHtml(role.account_count)}</p>`,
                `<a class=\"button-link\" href=\"/app/administration/roles/${role.id}\">View</a><a class=\"button-link\" href=\"/app/administration/roles/${role.id}/edit\">Edit</a>`
              )).join('')
            : emptyState('No roles found.');
          setHtml('role-list', html);
          show(payload);
        } catch (error) {
          setHtml('role-list', emptyState(error.message));
        }
      }

      async function loadRoleDetail(id) {
        try {
          await ensureAuthenticated();
          const payload = await request(`/api/admin/roles/${id}`);
          selectRecord('role', payload.name, payload.id);
          const capabilities = payload.capabilities.length
            ? payload.capabilities.map((capability) => `<li><strong>${escapeHtml(capability.key)}</strong><br><span class=\"muted\">${escapeHtml(capability.description || '')}</span></li>`).join('')
            : '<li class=\"muted\">No capabilities assigned.</li>';
          const assignedAccounts = payload.assigned_accounts.length
            ? payload.assigned_accounts.map((account) => `<li><a href=\"/app/administration/users/${account.account_id}\">${escapeHtml(account.display_name)}</a> <span class=\"muted\">${escapeHtml(account.email)}</span></li>`).join('')
            : '<li class=\"muted\">No users currently assigned.</li>';
          setHtml('role-detail', `
            ${detailSection('Summary', `<p>${escapeHtml(payload.name)}</p><p class=\"muted\">Capabilities: ${escapeHtml(payload.capabilities.length)}</p><p class=\"muted\">Assigned accounts: ${escapeHtml(payload.assigned_accounts.length)}</p>`)}
            ${detailSection('Capabilities', `<ul class=\"app-list\">${capabilities}</ul>`)}
            ${detailSection('Assigned Accounts', `<ul class=\"app-list\">${assignedAccounts}</ul>`)}
          `);
          show(payload);
        } catch (error) {
          setHtml('role-detail', emptyState(error.message));
        }
      }

      async function initRoleForm(mode, id = null) {
        try {
          await ensureAuthenticated();
          const capabilities = await request('/api/admin/capabilities');
          let role = null;
          roleFormState = { capabilities };
          if (mode === 'edit' && id) {
            role = await request(`/api/admin/roles/${id}`);
            byId('role-name').value = role.name || '';
            byId('role-name').disabled = true;
          } else {
            byId('role-name').disabled = false;
          }
          renderRoleCapabilityOptions((role?.capabilities || []).map((capability) => capability.id));
          const filterInput = byId('role-capability-filter');
          if (filterInput) {
            filterInput.oninput = () => renderRoleCapabilityOptions(collectSelectedCapabilityIds());
          }
          const form = byId('role-form');
          if (form) {
            form.onsubmit = async (event) => {
              event.preventDefault();
              const response = await request(
                mode === 'create' ? '/api/admin/roles' : `/api/admin/roles/${id}`,
                {
                  method: mode === 'create' ? 'POST' : 'PUT',
                  headers: { 'Content-Type': 'application/json' },
                  body: JSON.stringify(
                    mode === 'create'
                      ? {
                          name: byId('role-name').value.trim(),
                          capability_ids: collectSelectedCapabilityIds()
                        }
                      : {
                          capability_ids: collectSelectedCapabilityIds()
                        }
                  )
                }
              );
              redirect(`/app/administration/roles/${response.id}`);
            };
          }
        } catch (error) {
          setHtml('role-capability-options', `<tr><td colspan="3" class="muted">${escapeHtml(error.message)}</td></tr>`);
          show(error.message);
        }
      }

      function renderNodeTypeCheckboxes(containerId, nodeTypes, selectedIds = [], excludeId = '') {
        const element = byId(containerId);
        if (!element) return;
        const html = nodeTypes.length
          ? nodeTypes.map((nodeType) => `
              <label class="checkbox-label" for="${escapeHtml(containerId)}-${escapeHtml(nodeType.id)}">
                <input id="${escapeHtml(containerId)}-${escapeHtml(nodeType.id)}" class="${escapeHtml(containerId)}-checkbox" type="checkbox" value="${escapeHtml(nodeType.id)}" ${selectedIds.includes(nodeType.id) ? 'checked' : ''} ${excludeId && excludeId === nodeType.id ? 'disabled' : ''}>
                <span>
                  <strong>${escapeHtml(nodeType.singular_label || nodeType.name)}</strong>
                  <span class="muted">(${escapeHtml(nodeType.plural_label || nodeType.name)})</span>
                </span>
              </label>
            `).join('')
          : '<p class="muted">No node types are available.</p>';
        element.innerHTML = html;
      }

      function collectNodeTypeSelections(containerId) {
        return Array.from(document.querySelectorAll(`.${containerId}-checkbox:checked`))
          .filter((input) => !input.disabled)
          .map((input) => input.value);
      }

      async function loadNodeTypesList() {
        try {
          await ensureAuthenticated();
          const payload = await request('/api/node-types');
          const html = payload.length
            ? payload.map((nodeType) => recordCard(
                nodeType.singular_label || nodeType.name,
                `<p>${escapeHtml(nodeType.slug)}</p><p class=\"muted\">Plural: ${escapeHtml(nodeType.plural_label || nodeType.name)}</p><p class=\"muted\">Top-level: ${nodeType.is_root_type ? 'yes' : 'no'}</p><p class=\"muted\">Nodes: ${escapeHtml(nodeType.node_count)}</p>`,
                `<a class=\"button-link\" href=\"/app/administration/node-types/${nodeType.id}\">View</a><a class=\"button-link\" href=\"/app/administration/node-types/${nodeType.id}/edit\">Edit</a>`
              )).join('')
            : emptyState('No node types found.');
          setHtml('node-type-list', html);
          show(payload);
        } catch (error) {
          setHtml('node-type-list', emptyState(error.message));
        }
      }

      async function loadNodeTypeDetail(id) {
        try {
          await ensureAuthenticated();
          const payload = await request(`/api/admin/node-types/${id}`);
          selectRecord('node type', payload.singular_label || payload.name, payload.id);
          const parentTypes = payload.parent_relationships.length
            ? payload.parent_relationships.map((nodeType) => `<li>${escapeHtml(nodeType.singular_label || nodeType.node_type_name)}</li>`).join('')
            : '<li class=\"muted\">This is a top-level node type.</li>';
          const childTypes = payload.child_relationships.length
            ? payload.child_relationships.map((nodeType) => `<li>${escapeHtml(nodeType.singular_label || nodeType.node_type_name)}</li>`).join('')
            : '<li class=\"muted\">No child node types configured.</li>';
          const metadataFields = payload.metadata_fields.length
            ? payload.metadata_fields.map((field) => `<li>${escapeHtml(field.label)} <span class=\"muted\">(${escapeHtml(field.key)} · ${escapeHtml(field.field_type)}${field.required ? ' · required' : ''})</span></li>`).join('')
            : '<li class=\"muted\">No metadata fields configured.</li>';
          const forms = payload.scoped_forms.length
            ? payload.scoped_forms.map((form) => `<li><a href=\"/app/forms/${form.form_id}\">${escapeHtml(form.form_name)}</a></li>`).join('')
            : '<li class=\"muted\">No forms scoped to this node type.</li>';
          setHtml('node-type-detail', `
            ${detailSection('Summary', `<p>${escapeHtml(payload.name)}</p><p class=\"muted\">Slug: ${escapeHtml(payload.slug)}</p><p class=\"muted\">Singular: ${escapeHtml(payload.singular_label)}</p><p class=\"muted\">Plural: ${escapeHtml(payload.plural_label)}</p><p class=\"muted\">Top-level: ${payload.is_root_type ? 'yes' : 'no'}</p><p class=\"muted\">Nodes: ${escapeHtml(payload.node_count)}</p>`)}
            ${detailSection('Allowed Parents', `<ul class=\"app-list\">${parentTypes}</ul>`)}
            ${detailSection('Allowed Children', `<ul class=\"app-list\">${childTypes}</ul>`)}
            ${detailSection('Metadata Fields', `<ul class=\"app-list\">${metadataFields}</ul>`)}
            ${detailSection('Scoped Forms', `<ul class=\"app-list\">${forms}</ul>`)}
          `);
          show(payload);
        } catch (error) {
          setHtml('node-type-detail', emptyState(error.message));
        }
      }

      async function initNodeTypeForm(mode, id = null) {
        try {
          await ensureAuthenticated();
          const nodeTypes = await request('/api/node-types');
          nodeTypeFormState = { nodeTypes };
          let payload = null;
          if (mode === 'edit' && id) {
            payload = await request(`/api/admin/node-types/${id}`);
            byId('node-type-name').value = payload.name || '';
            byId('node-type-slug').value = payload.slug || '';
            byId('node-type-singular-label').value = payload.singular_label || '';
            byId('node-type-plural-label').value = payload.plural_label || '';
          }

          renderNodeTypeCheckboxes(
            'node-type-parent-options',
            nodeTypes.filter((nodeType) => nodeType.id !== id),
            (payload?.parent_relationships || []).map((nodeType) => nodeType.node_type_id),
            id || ''
          );
          renderNodeTypeCheckboxes(
            'node-type-child-options',
            nodeTypes.filter((nodeType) => nodeType.id !== id),
            (payload?.child_relationships || []).map((nodeType) => nodeType.node_type_id),
            id || ''
          );

          const form = byId('node-type-form');
          if (form) {
            form.onsubmit = async (event) => {
              event.preventDefault();
              const name = byId('node-type-name').value.trim();
              const slug = byId('node-type-slug').value.trim();
              if (!name || !slug) {
                byId('node-type-form-status').textContent = 'Name and slug are required.';
                return;
              }
              const response = await request(
                mode === 'create' ? '/api/admin/node-types' : `/api/admin/node-types/${id}`,
                {
                  method: mode === 'create' ? 'POST' : 'PUT',
                  headers: { 'Content-Type': 'application/json' },
                  body: JSON.stringify({
                    name,
                    slug,
                    singular_label: byId('node-type-singular-label').value.trim() || null,
                    plural_label: byId('node-type-plural-label').value.trim() || null,
                    parent_node_type_ids: collectNodeTypeSelections('node-type-parent-options'),
                    child_node_type_ids: collectNodeTypeSelections('node-type-child-options')
                  })
                }
              );
              redirect(`/app/administration/node-types/${response.id}`);
            };
          }
          byId('node-type-form-status').textContent = 'Node-type configuration loaded.';
        } catch (error) {
          setHtml('node-type-parent-options', emptyState(error.message));
          setHtml('node-type-child-options', emptyState(error.message));
          if (byId('node-type-form-status')) {
            byId('node-type-form-status').textContent = error.message;
          }
          show(error.message);
        }
      }

      function renderOrganizationNodeTypeRelationshipTags(title, items, emptyLabel) {
        const tags = items.length
          ? items.map((item) => `<span class=\"tag organization-node-type-tag\">${escapeHtml(item.singular_label || item.node_type_name || item.name)}</span>`).join('')
          : `<span class=\"tag organization-node-type-tag organization-node-type-tag-placeholder\">${escapeHtml(emptyLabel)}</span>`;
        return `<p class=\"muted\">${escapeHtml(title)}</p><div class=\"tags organization-node-type-relationship-tags\">${tags}</div>`;
      }

      function organizationNodeTypeSelectionKey(kind) {
        return kind === 'parent' ? 'parentSelection' : 'childSelection';
      }

      function organizationNodeMetadataFieldTypeOptions(selectedValue) {
        const fieldTypes = [
          ['text', 'Text'],
          ['number', 'Number'],
          ['boolean', 'Boolean'],
          ['date', 'Date'],
          ['single_choice', 'Single Choice'],
          ['multi_choice', 'Multi Choice']
        ];
        return fieldTypes.map(([value, label]) => `
          <option value="${escapeHtml(value)}" ${selectedValue === value ? 'selected' : ''}>${escapeHtml(label)}</option>
        `).join('');
      }

      function syncOrganizationNodeTypeMetadataStateFromGrid() {
        const rows = Array.from(document.querySelectorAll('.node-type-metadata-grid-row[data-metadata-index]'));
        if (!rows.length) return;
        nodeTypeFormState.metadataFields = rows.map((row) => ({
          id: row.dataset.fieldId || '',
          label: row.querySelector('.node-type-metadata-label')?.value.trim() || '',
          key: row.querySelector('.node-type-metadata-key')?.value.trim() || '',
          field_type: row.querySelector('.node-type-metadata-type')?.value || 'text',
          required: Boolean(row.dataset.required === 'true')
        }));
      }

      function renderOrganizationNodeTypeMetadataFieldRows() {
        const fields = nodeTypeFormState.metadataFields || [];
        const html = fields.length
          ? `
              <div class=\"node-type-metadata-grid\" role=\"rowgroup\">
                <div class=\"node-type-metadata-grid-header\" role=\"row\">
                  <div role=\"columnheader\">Label</div>
                  <div role=\"columnheader\">Key</div>
                  <div role=\"columnheader\">Field Type</div>
                  <div role=\"columnheader\">Settings</div>
                </div>
                ${fields.map((field, index) => `
                  <div class=\"node-type-metadata-grid-row\" role=\"row\" data-metadata-index=\"${index}\" data-field-id=\"${escapeHtml(field.id || '')}\" data-required=\"${field.required ? 'true' : 'false'}\">
                    <div class=\"node-type-metadata-grid-cell\" role=\"gridcell\">
                      <input class=\"input node-type-metadata-label\" id=\"node-type-metadata-label-${index}\" type=\"text\" autocomplete=\"off\" value=\"${escapeHtml(field.label || '')}\" placeholder=\"Display label\">
                    </div>
                    <div class=\"node-type-metadata-grid-cell\" role=\"gridcell\">
                      <input class=\"input node-type-metadata-key\" id=\"node-type-metadata-key-${index}\" type=\"text\" autocomplete=\"off\" value=\"${escapeHtml(field.key || '')}\" placeholder=\"metadata_key\">
                    </div>
                    <div class=\"node-type-metadata-grid-cell\" role=\"gridcell\">
                      <div class=\"select is-fullwidth\">
                        <select class=\"node-type-metadata-type\" id=\"node-type-metadata-type-${index}\">
                          ${organizationNodeMetadataFieldTypeOptions(field.field_type || 'text')}
                        </select>
                      </div>
                    </div>
                    <div class=\"node-type-metadata-grid-cell node-type-metadata-grid-cell-actions\" role=\"gridcell\">
                      <button type=\"button\" class=\"button is-light is-small node-type-metadata-settings-button\" data-metadata-index=\"${index}\">Settings</button>
                    </div>
                  </div>
                `).join('')}
              </div>
            `
          : '<p class=\"muted\">No metadata fields configured yet.</p>';
        setHtml('node-type-metadata-fields-editor', html);
      }

      function addOrganizationNodeTypeMetadataField() {
        nodeTypeFormState.metadataFields = [
          ...(nodeTypeFormState.metadataFields || []),
          { id: '', label: '', key: '', field_type: 'text', required: false }
        ];
        renderOrganizationNodeTypeMetadataFieldRows();
      }

      function removeOrganizationNodeTypeMetadataField(index) {
        nodeTypeFormState.metadataFields = (nodeTypeFormState.metadataFields || []).filter((_field, fieldIndex) => fieldIndex !== index);
        renderOrganizationNodeTypeMetadataFieldRows();
      }

      async function syncOrganizationNodeTypeMetadataFields(nodeTypeId) {
        syncOrganizationNodeTypeMetadataStateFromGrid();
        const fields = (nodeTypeFormState.metadataFields || []).filter((field) => field.id || field.label || field.key);
        for (const [index, field] of fields.entries()) {
          if (!field.label || !field.key) {
            throw new Error(`Metadata field ${index + 1} requires both a label and key.`);
          }
          if (field.id) {
            await request(`/api/admin/node-metadata-fields/${field.id}`, {
              method: 'PUT',
              headers: { 'Content-Type': 'application/json' },
              body: JSON.stringify({
                key: field.key,
                label: field.label,
                field_type: field.field_type,
                required: field.required
              })
            });
          } else {
            await request('/api/admin/node-metadata-fields', {
              method: 'POST',
              headers: { 'Content-Type': 'application/json' },
              body: JSON.stringify({
                node_type_id: nodeTypeId,
                key: field.key,
                label: field.label,
                field_type: field.field_type,
                required: field.required
              })
            });
          }
        }
      }

      async function removePersistedOrganizationNodeTypeMetadataField(index) {
        const field = (nodeTypeFormState.metadataFields || [])[index];
        if (!field) return;
        if (field.id) {
          await request(`/api/admin/node-metadata-fields/${field.id}`, {
            method: 'DELETE'
          });
        }
        removeOrganizationNodeTypeMetadataField(index);
      }

      function closeOrganizationNodeTypeMetadataSettingsModal() {
        const modal = byId('node-type-metadata-settings-modal');
        if (modal) {
          modal.classList.remove('is-active');
        }
        nodeTypeFormState.activeMetadataIndex = -1;
      }

      function openOrganizationNodeTypeMetadataSettingsModal(index) {
        syncOrganizationNodeTypeMetadataStateFromGrid();
        const field = (nodeTypeFormState.metadataFields || [])[index];
        const modal = byId('node-type-metadata-settings-modal');
        const title = byId('node-type-metadata-settings-title');
        const required = byId('node-type-metadata-settings-required');
        const removeButton = byId('node-type-metadata-settings-remove');
        if (!field || !modal || !title || !required || !removeButton) return;
        nodeTypeFormState.activeMetadataIndex = index;
        title.textContent = field.label || field.key || `Metadata Field ${index + 1}`;
        required.checked = Boolean(field.required);
        removeButton.textContent = field.id ? 'Remove Field' : 'Remove Draft';
        modal.classList.add('is-active');
      }

      function initOrganizationNodeTypeMetadataSettingsModal() {
        const modal = byId('node-type-metadata-settings-modal');
        const saveButton = byId('node-type-metadata-settings-save');
        const removeButton = byId('node-type-metadata-settings-remove');
        const required = byId('node-type-metadata-settings-required');
        if (modal) {
          modal.onclick = (event) => {
            const dismiss = event.target.closest('[data-dismiss=\"modal\"]');
            if (dismiss) {
              closeOrganizationNodeTypeMetadataSettingsModal();
            }
          };
        }
        if (saveButton) {
          saveButton.onclick = () => {
            const index = nodeTypeFormState.activeMetadataIndex;
            if (index < 0) return;
            syncOrganizationNodeTypeMetadataStateFromGrid();
            if (nodeTypeFormState.metadataFields[index]) {
              nodeTypeFormState.metadataFields[index].required = Boolean(required?.checked);
            }
            renderOrganizationNodeTypeMetadataFieldRows();
            closeOrganizationNodeTypeMetadataSettingsModal();
          };
        }
        if (removeButton) {
          removeButton.onclick = async () => {
            const index = nodeTypeFormState.activeMetadataIndex;
            if (index < 0) return;
            try {
              await removePersistedOrganizationNodeTypeMetadataField(index);
              closeOrganizationNodeTypeMetadataSettingsModal();
            } catch (error) {
              if (byId('node-type-form-status')) {
                byId('node-type-form-status').textContent = error.message;
              }
            }
          };
        }
      }

      function initOrganizationNodeTypeMetadataFieldControls() {
        const addButton = byId('node-type-metadata-add');
        const editor = byId('node-type-metadata-fields-editor');
        if (addButton) {
          addButton.onclick = () => addOrganizationNodeTypeMetadataField();
        }
        if (editor) {
          editor.onclick = (event) => {
            const button = event.target.closest('.node-type-metadata-settings-button');
            if (!button || !editor.contains(button)) return;
            openOrganizationNodeTypeMetadataSettingsModal(Number(button.dataset.metadataIndex || '-1'));
          };
        }
        initOrganizationNodeTypeMetadataSettingsModal();
      }

      function organizationNodeTypeFilterId(kind) {
        return `node-type-${kind}-filter`;
      }

      function organizationNodeTypeTagsId(kind) {
        return `node-type-${kind}-tags`;
      }

      function organizationNodeTypeOptionsId(kind) {
        return `node-type-${kind}-options`;
      }

      function selectedOrganizationNodeTypeIds(kind) {
        return nodeTypeFormState[organizationNodeTypeSelectionKey(kind)] || [];
      }

      function setSelectedOrganizationNodeTypeIds(kind, values) {
        nodeTypeFormState[organizationNodeTypeSelectionKey(kind)] = [...values].sort();
      }

      function availableOrganizationNodeTypesForSelection(kind) {
        const filter = normalizeFilterValue(byId(organizationNodeTypeFilterId(kind))?.value);
        const selectedIds = new Set(selectedOrganizationNodeTypeIds(kind));
        const oppositeKind = kind === 'parent' ? 'child' : 'parent';
        const oppositeSelectedIds = new Set(selectedOrganizationNodeTypeIds(oppositeKind));
        return (nodeTypeFormState.nodeTypes || []).filter((nodeType) => {
          if (nodeType.id === nodeTypeFormState.excludeId) return false;
          if (selectedIds.has(nodeType.id)) return false;
          if (oppositeSelectedIds.has(nodeType.id)) return false;
          if (!filter) return true;
          return normalizeFilterValue(nodeType.name).includes(filter)
            || normalizeFilterValue(nodeType.plural_label || nodeType.name).includes(filter)
            || normalizeFilterValue(nodeType.slug).includes(filter);
        });
      }

      function renderOrganizationNodeTypeSelectionTags(kind) {
        const selectedIds = selectedOrganizationNodeTypeIds(kind);
        const nodeTypesById = new Map((nodeTypeFormState.nodeTypes || []).map((nodeType) => [nodeType.id, nodeType]));
        const html = selectedIds.length
          ? selectedIds.map((nodeTypeId) => {
              const nodeType = nodeTypesById.get(nodeTypeId);
              if (!nodeType) return '';
              return `
                <div class=\"tags has-addons organization-node-type-selection-tag\">
                  <span class=\"tag organization-node-type-tag\">${escapeHtml(nodeType.name)}</span>
                  <a class=\"tag is-delete node-type-tag-remove\" data-kind=\"${escapeHtml(kind)}\" data-node-type-id=\"${escapeHtml(nodeTypeId)}\" aria-label=\"Remove ${escapeHtml(nodeType.name)}\"></a>
                </div>
              `;
            }).join('')
          : `<span class=\"tag organization-node-type-tag organization-node-type-tag-placeholder\">${kind === 'parent' ? 'Top-level' : 'No child node types selected'}</span>`;
        setHtml(organizationNodeTypeTagsId(kind), html);
      }

      function renderOrganizationNodeTypeSelectionOptions(kind) {
        const options = availableOrganizationNodeTypesForSelection(kind);
        const html = options.length
          ? options.map((nodeType) => `
              <button type=\"button\" class=\"node-type-option-button\" data-kind=\"${escapeHtml(kind)}\" data-node-type-id=\"${escapeHtml(nodeType.id)}\">
                <strong>${escapeHtml(nodeType.name)}</strong>
                <span class=\"muted\">${escapeHtml(nodeType.plural_label || nodeType.name)} · ${escapeHtml(nodeType.slug)}</span>
              </button>
            `).join('')
          : '<p class=\"muted\">No organization node types match the current search.</p>';
        setHtml(organizationNodeTypeOptionsId(kind), html);
      }

      function renderOrganizationNodeTypeSelection(kind) {
        renderOrganizationNodeTypeSelectionTags(kind);
        renderOrganizationNodeTypeSelectionOptions(kind);
      }

      function addOrganizationNodeTypeSelection(kind, nodeTypeId) {
        const selectedIds = new Set(selectedOrganizationNodeTypeIds(kind));
        const oppositeKind = kind === 'parent' ? 'child' : 'parent';
        const oppositeIds = new Set(selectedOrganizationNodeTypeIds(oppositeKind));
        if (oppositeIds.has(nodeTypeId)) {
          if (byId('node-type-form-status')) {
            byId('node-type-form-status').textContent = 'A node type cannot be both a parent and child of the same organization node type.';
          }
          return;
        }
        selectedIds.add(nodeTypeId);
        setSelectedOrganizationNodeTypeIds(kind, Array.from(selectedIds));
        renderOrganizationNodeTypeSelection(kind);
        renderOrganizationNodeTypeSelection(oppositeKind);
        const input = byId(organizationNodeTypeFilterId(kind));
        if (input) {
          input.value = '';
          renderOrganizationNodeTypeSelection(kind);
          input.focus();
        }
      }

      function removeOrganizationNodeTypeSelection(kind, nodeTypeId) {
        const oppositeKind = kind === 'parent' ? 'child' : 'parent';
        setSelectedOrganizationNodeTypeIds(
          kind,
          selectedOrganizationNodeTypeIds(kind).filter((value) => value !== nodeTypeId)
        );
        renderOrganizationNodeTypeSelection(kind);
        renderOrganizationNodeTypeSelection(oppositeKind);
      }

      function initOrganizationNodeTypeSelectionControls(kind) {
        const input = byId(organizationNodeTypeFilterId(kind));
        const options = byId(organizationNodeTypeOptionsId(kind));
        const tags = byId(organizationNodeTypeTagsId(kind));
        if (input) {
          input.oninput = () => renderOrganizationNodeTypeSelection(kind);
          input.onkeydown = (event) => {
            if (event.key !== 'Enter') return;
            event.preventDefault();
            const firstOption = availableOrganizationNodeTypesForSelection(kind)[0];
            if (firstOption) {
              addOrganizationNodeTypeSelection(kind, firstOption.id);
            }
          };
        }
        if (options) {
          options.onclick = (event) => {
            const button = event.target.closest('.node-type-option-button');
            if (!button || !options.contains(button)) return;
            addOrganizationNodeTypeSelection(kind, button.dataset.nodeTypeId || '');
          };
        }
        if (tags) {
          tags.onclick = (event) => {
            const button = event.target.closest('.node-type-tag-remove');
            if (!button || !tags.contains(button)) return;
            removeOrganizationNodeTypeSelection(kind, button.dataset.nodeTypeId || '');
          };
        }
      }

      async function loadOrganizationNodeTypesList() {
        try {
          await ensureAuthenticated();
          const payload = await request('/api/node-types');
          const html = payload.length
            ? payload.map((nodeType) => recordCard(
                nodeType.name,
                `<p>${escapeHtml(nodeType.slug)}</p><p class=\"muted\">Plural Label: ${escapeHtml(nodeType.plural_label || nodeType.name)}</p><p class=\"muted\">Top-level: ${nodeType.is_root_type ? 'yes' : 'no'}</p><p class=\"muted\">Nodes: ${escapeHtml(nodeType.node_count)}</p>${renderOrganizationNodeTypeRelationshipTags('Allowed Under', nodeType.parent_relationships || [], 'Top-level')}${renderOrganizationNodeTypeRelationshipTags('Can Contain', nodeType.child_relationships || [], 'No child node types')}`,
                `<a class=\"button-link\" href=\"/app/administration/node-types/${nodeType.id}\">View</a><a class=\"button-link\" href=\"/app/administration/node-types/${nodeType.id}/edit\">Edit</a>`
              )).join('')
            : emptyState('No organization node types found.');
          setHtml('node-type-list', html);
          show(payload);
        } catch (error) {
          setHtml('node-type-list', emptyState(error.message));
        }
      }

      async function loadOrganizationNodeTypeDetail(id) {
        try {
          await ensureAuthenticated();
          const payload = await request(`/api/admin/node-types/${id}`);
          selectRecord('organization node type', payload.name, payload.id);
          const parentTypes = payload.parent_relationships.length
            ? payload.parent_relationships.map((nodeType) => `<li>${escapeHtml(nodeType.singular_label || nodeType.node_type_name)}</li>`).join('')
            : '<li class=\"muted\">This is a top-level organization node type.</li>';
          const childTypes = payload.child_relationships.length
            ? payload.child_relationships.map((nodeType) => `<li>${escapeHtml(nodeType.singular_label || nodeType.node_type_name)}</li>`).join('')
            : '<li class=\"muted\">No child organization node types configured.</li>';
          const metadataFields = payload.metadata_fields.length
            ? payload.metadata_fields.map((field) => `<li>${escapeHtml(field.label)} <span class=\"muted\">(${escapeHtml(field.key)} · ${escapeHtml(field.field_type)}${field.required ? ' · required' : ''})</span></li>`).join('')
            : '<li class=\"muted\">No metadata fields configured.</li>';
          const forms = payload.scoped_forms.length
            ? payload.scoped_forms.map((form) => `<li><a href=\"/app/forms/${form.form_id}\">${escapeHtml(form.form_name)}</a></li>`).join('')
            : '<li class=\"muted\">No forms scoped to this node type.</li>';
          setHtml('node-type-detail', `
            ${detailSection('Summary', `<p>${escapeHtml(payload.name)}</p><p class=\"muted\">Slug: ${escapeHtml(payload.slug)}</p><p class=\"muted\">Plural Label: ${escapeHtml(payload.plural_label)}</p><p class=\"muted\">Top-level: ${payload.is_root_type ? 'yes' : 'no'}</p><p class=\"muted\">Nodes: ${escapeHtml(payload.node_count)}</p>`)}
            ${detailSection('Allowed Parents', `<ul class=\"app-list\">${parentTypes}</ul>`)}
            ${detailSection('Allowed Children', `<ul class=\"app-list\">${childTypes}</ul>`)}
            ${detailSection('Metadata Fields', `<ul class=\"app-list\">${metadataFields}</ul>`)}
            ${detailSection('Scoped Forms', `<ul class=\"app-list\">${forms}</ul>`)}
          `);
          show(payload);
        } catch (error) {
          setHtml('node-type-detail', emptyState(error.message));
        }
      }

      async function initOrganizationNodeTypeForm(mode, id = null) {
        try {
          await ensureAuthenticated();
          const nodeTypes = await request('/api/node-types');
          nodeTypeFormState = {
            nodeTypes,
            excludeId: id || '',
            parentSelection: [],
            childSelection: [],
            metadataFields: []
          };

          if (mode === 'edit' && id) {
            const payload = await request(`/api/admin/node-types/${id}`);
            byId('node-type-name').value = payload.name || '';
            byId('node-type-slug').value = payload.slug || '';
            byId('node-type-plural-label').value = payload.plural_label || '';
            nodeTypeFormState.parentSelection = (payload.parent_relationships || []).map((nodeType) => nodeType.node_type_id);
            nodeTypeFormState.childSelection = (payload.child_relationships || []).map((nodeType) => nodeType.node_type_id);
            nodeTypeFormState.metadataFields = (payload.metadata_fields || []).map((field) => ({
              id: field.id,
              label: field.label,
              key: field.key,
              field_type: field.field_type,
              required: field.required
            }));
          }

          initOrganizationNodeTypeSelectionControls('parent');
          initOrganizationNodeTypeSelectionControls('child');
          initOrganizationNodeTypeMetadataFieldControls();
          renderOrganizationNodeTypeSelection('parent');
          renderOrganizationNodeTypeSelection('child');
          renderOrganizationNodeTypeMetadataFieldRows();

          const form = byId('node-type-form');
          if (form) {
            form.onsubmit = async (event) => {
              event.preventDefault();
              try {
                const name = byId('node-type-name').value.trim();
                const slug = byId('node-type-slug').value.trim();
                if (!name || !slug) {
                  byId('node-type-form-status').textContent = 'Name and slug are required.';
                  return;
                }
                if (selectedOrganizationNodeTypeIds('parent').some((nodeTypeId) => selectedOrganizationNodeTypeIds('child').includes(nodeTypeId))) {
                  byId('node-type-form-status').textContent = 'A node type cannot be both a parent and child of the same organization node type.';
                  return;
                }
                byId('node-type-form-status').textContent = mode === 'create'
                  ? 'Saving organization node type.'
                  : 'Saving organization node type changes.';
                const response = await request(
                  mode === 'create' ? '/api/admin/node-types' : `/api/admin/node-types/${id}`,
                  {
                    method: mode === 'create' ? 'POST' : 'PUT',
                    headers: { 'Content-Type': 'application/json' },
                    body: JSON.stringify({
                      name,
                      slug,
                      plural_label: byId('node-type-plural-label').value.trim() || null,
                      parent_node_type_ids: selectedOrganizationNodeTypeIds('parent'),
                      child_node_type_ids: selectedOrganizationNodeTypeIds('child')
                    })
                  }
                );
                byId('node-type-form-status').textContent = 'Saving metadata field configuration.';
                await syncOrganizationNodeTypeMetadataFields(response.id);
                redirect(`/app/administration/node-types/${response.id}`);
              } catch (error) {
                byId('node-type-form-status').textContent = error.message;
              }
            };
          }

          byId('node-type-form-status').textContent = 'Organization node-type configuration loaded.';
        } catch (error) {
          setHtml('node-type-parent-options', emptyState(error.message));
          setHtml('node-type-child-options', emptyState(error.message));
          setHtml('node-type-parent-tags', `<span class=\"tag is-danger is-light\">${escapeHtml(error.message)}</span>`);
          setHtml('node-type-child-tags', `<span class=\"tag is-danger is-light\">${escapeHtml(error.message)}</span>`);
          if (byId('node-type-form-status')) {
            byId('node-type-form-status').textContent = error.message;
          }
          show(error.message);
        }
      }

      function renderUserScopeOptions(nodes, selectedNodeIds = [], editable = true) {
        const filter = normalizeFilterValue(byId('user-scope-filter')?.value);
        const filteredNodes = nodes.filter((node) => {
          if (!filter) return true;
          return normalizeFilterValue(node.node_name).includes(filter)
            || normalizeFilterValue(node.node_type_name).includes(filter)
            || normalizeFilterValue(node.parent_node_name).includes(filter);
        });
        const html = filteredNodes.length
          ? filteredNodes.map((node) => `
              <tr>
                <td>
                  <input id="user-scope-${escapeHtml(node.node_id)}" class="user-scope-checkbox" type="checkbox" value="${escapeHtml(node.node_id)}" ${selectedNodeIds.includes(node.node_id) ? 'checked' : ''} ${editable ? '' : 'disabled'}>
                </td>
                <td><label for="user-scope-${escapeHtml(node.node_id)}">${escapeHtml(node.node_name)}</label></td>
                <td>${escapeHtml(node.node_type_name)}</td>
                <td>${escapeHtml(node.parent_node_name || 'No parent')}</td>
              </tr>
            `).join('')
          : '<tr><td colspan="4" class="muted">No organization nodes match the current filter.</td></tr>';
        setHtml('user-scope-options', html);
      }

      function collectSelectedScopeNodeIds() {
        return Array.from(document.querySelectorAll('.user-scope-checkbox:checked'))
          .map((input) => input.value);
      }

      function renderUserDelegationOptions(accounts, selectedAccountIds = [], editable = true) {
        const filter = normalizeFilterValue(byId('user-delegation-filter')?.value);
        const filteredAccounts = accounts.filter((account) => {
          if (!filter) return true;
          return normalizeFilterValue(account.display_name).includes(filter)
            || normalizeFilterValue(account.email).includes(filter);
        });
        const html = filteredAccounts.length
          ? filteredAccounts.map((account) => `
              <tr>
                <td>
                  <input id="user-delegation-${escapeHtml(account.account_id)}" class="user-delegation-checkbox" type="checkbox" value="${escapeHtml(account.account_id)}" ${selectedAccountIds.includes(account.account_id) ? 'checked' : ''} ${editable ? '' : 'disabled'}>
                </td>
                <td><label for="user-delegation-${escapeHtml(account.account_id)}">${escapeHtml(account.display_name)}</label></td>
                <td>${escapeHtml(account.email)}</td>
              </tr>
            `).join('')
          : '<tr><td colspan="3" class="muted">No delegate accounts match the current filter.</td></tr>';
        setHtml('user-delegation-options', html);
      }

      function collectSelectedDelegateAccountIds() {
        return Array.from(document.querySelectorAll('.user-delegation-checkbox:checked'))
          .map((input) => input.value);
      }

      function renderUserAccessSummary(payload) {
        setHtml('user-access-summary', `
          <dl class="detail-list">
            <div><dt>Display Name</dt><dd>${escapeHtml(payload.display_name)}</dd></div>
            <div><dt>Email</dt><dd>${escapeHtml(payload.email)}</dd></div>
            <div><dt>UI Profile</dt><dd>${escapeHtml(String(payload.ui_access_profile || '').replaceAll('_', ' '))}</dd></div>
            <div><dt>Capabilities</dt><dd>${escapeHtml((payload.capabilities || []).join(', ') || 'None')}</dd></div>
            <div><dt>Scope Summary</dt><dd>${escapeHtml(`${(payload.scope_nodes || []).length} assigned node(s)`)}</dd></div>
            <div><dt>Delegation Summary</dt><dd>${escapeHtml(`${(payload.delegations || []).length} delegated account(s)`)}</dd></div>
          </dl>
        `);
      }

      async function initUserAccessForm(id) {
        try {
          await ensureAuthenticated();
          const payload = await request(`/api/admin/users/${id}/access`);
          selectRecord('user', payload.display_name || payload.email, payload.account_id);
          renderUserAccessSummary(payload);
          renderUserScopeOptions(
            payload.available_scope_nodes || [],
            (payload.scope_nodes || []).map((node) => node.node_id),
            payload.scope_assignments_editable
          );
          renderUserDelegationOptions(
            payload.available_delegate_accounts || [],
            (payload.delegations || []).map((account) => account.account_id),
            payload.delegation_assignments_editable
          );
          const scopeEditability = byId('user-scope-editability');
          if (scopeEditability) {
            scopeEditability.textContent = payload.scope_assignments_editable
              ? 'Scope assignments are editable for this account because its current capability set supports scoped product access.'
              : 'Scope assignments are read-only for this account because its current capability set does not use scoped product access.';
          }
          const scopeFilter = byId('user-scope-filter');
          if (scopeFilter) {
            scopeFilter.oninput = () => {
              renderUserScopeOptions(
                payload.available_scope_nodes || [],
                collectSelectedScopeNodeIds(),
                payload.scope_assignments_editable
              );
            };
          }
          const delegationFilter = byId('user-delegation-filter');
          if (delegationFilter) {
            delegationFilter.oninput = () => {
              renderUserDelegationOptions(
                payload.available_delegate_accounts || [],
                collectSelectedDelegateAccountIds(),
                payload.delegation_assignments_editable
              );
            };
          }
          const form = byId('user-access-form');
          if (form) {
            form.onsubmit = async (event) => {
              event.preventDefault();
              const response = await request(`/api/admin/users/${id}/access`, {
                method: 'PUT',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({
                  scope_node_ids: collectSelectedScopeNodeIds(),
                  delegate_account_ids: collectSelectedDelegateAccountIds()
                })
              });
              redirect(`/app/administration/users/${response.id}`);
            };
          }
        } catch (error) {
          setHtml('user-scope-options', `<tr><td colspan="4" class="muted">${escapeHtml(error.message)}</td></tr>`);
          setHtml('user-delegation-options', `<tr><td colspan="3" class="muted">${escapeHtml(error.message)}</td></tr>`);
          show(error.message);
        }
      }

      async function loadFormsList() {
        try {
          await ensureAuthenticated();
          const payload = await request('/api/forms');
          const html = payload.length
            ? payload.map((form) => recordCard(
                form.name,
                `<p>${escapeHtml(form.slug)}</p><p class=\"muted\">Scope: ${escapeHtml(form.scope_node_type_name || 'Unscoped')}</p><p class=\"muted\">Published: ${escapeHtml((latestMatchingVersion(form.versions, (version) => version.status === 'published') || {}).version_label || 'None')}</p><p class=\"muted\">Drafts: ${escapeHtml(form.versions.filter((version) => version.status === 'draft').length)}</p>`,
                `<a class=\"button-link\" href=\"/app/forms/${form.id}\">View</a>${hasCapability('forms:write') ? `<a class=\"button-link\" href=\"/app/forms/${form.id}/edit\">Edit</a>` : ''}`
              )).join('')
            : emptyState('No form records found.');
          setHtml('form-list', html);
          show(payload);
        } catch (error) {
          setHtml('form-list', emptyState(error.message));
        }
      }

      async function loadFormDetail(id) {
        try {
          await ensureAuthenticated();
          await loadReadableFormSurface(id, formBuilderState.selectedVersionId);
          show(formBuilderState.form);
        } catch (error) {
          setHtml('form-detail', emptyState(error.message));
          setHtml('form-version-summary', emptyState(error.message));
          setHtml('form-version-preview', emptyState(error.message));
          setHtml('form-workflow-links', emptyState(error.message));
        }
      }

      async function initFormEntityForm(mode, id) {
        try {
          await ensureAuthenticated();
          const nodeTypes = await request('/api/admin/node-types');
          setSelectOptions(
            'form-scope-node-type',
            nodeTypes.map((item) => ({ value: item.id, label: item.name })),
            'No scope'
          );
          const form = byId('form-entity-form');
          if (form) {
            form.onsubmit = async (event) => {
              event.preventDefault();
              try {
                const response = await runLockedFormAction('form-metadata', async () => {
                  setFormStatus('form-editor-status', mode === 'create' ? 'Creating form...' : 'Saving form metadata...');
                  return request(
                    mode === 'create' ? '/api/admin/forms' : `/api/admin/forms/${id}`,
                    {
                      method: mode === 'create' ? 'POST' : 'PUT',
                      headers: { 'Content-Type': 'application/json' },
                      body: JSON.stringify({
                        name: byId('form-name').value.trim(),
                        slug: byId('form-slug').value.trim(),
                        scope_node_type_id: byId('form-scope-node-type').value || null
                      })
                    }
                  );
                });
                setFormStatus('form-editor-status', 'Form metadata saved.', 'success');
                if (mode === 'create') {
                  redirect(`/app/forms/${response.id}/edit`);
                  return;
                }
                await loadEditableFormSurface(response.id, formBuilderState.selectedVersionId);
              } catch (error) {
                setFormStatus('form-editor-status', error.message, 'error');
                show(error.message);
              }
            };
          }
          if (mode === 'edit' && id) {
            const versionForm = byId('form-version-create-form');
            if (versionForm) {
              versionForm.onsubmit = async (event) => {
                event.preventDefault();
                await createFormVersion();
              };
            }
            await loadEditableFormSurface(id);
          } else {
            setFormStatus('form-editor-status', 'Create the form record first. Version authoring opens after the form is saved.');
          }
        } catch (error) {
          setFormStatus('form-editor-status', error.message, 'error');
          show(error.message);
        }
      }

      async function loadReadableFormSurface(id, preferredVersionId = '') {
        const payload = await request(`/api/forms/${id}`);
        formBuilderState.form = payload;
        selectRecord('form', payload.name, payload.id);
        const publishedVersion = latestMatchingVersion(payload.versions, (version) => version.status === 'published');
        const draftCount = payload.versions.filter((version) => version.status === 'draft').length;
        setHtml('form-detail', `
          ${detailSection('Summary', `<p>${escapeHtml(payload.name)}</p><p>${escapeHtml(payload.slug)}</p><p class=\"muted\">Scope: ${escapeHtml(payload.scope_node_type_name || 'Unscoped')}</p><p class=\"muted\">Published version: ${escapeHtml(publishedVersion ? formVersionLabel(publishedVersion) : 'None')}</p><p class=\"muted\">Draft versions: ${escapeHtml(draftCount)}</p>`)}
        `);
        const selectedVersion = choosePreferredFormVersion(payload, preferredVersionId);
        setHtml('form-version-summary', renderFormVersionCards(payload, selectedVersion?.id || '', false));
        setHtml('form-workflow-links', renderFormWorkflowAttachments(payload));
        if (!selectedVersion) {
          formBuilderState.selectedVersionId = '';
          formBuilderState.renderedVersion = null;
          setHtml('form-version-preview', emptyState('No form versions are available to preview.'));
          return;
        }
        formBuilderState.selectedVersionId = selectedVersion.id;
        formBuilderState.renderedVersion = await request(`/api/form-versions/${selectedVersion.id}/render`);
        setHtml('form-version-preview', renderFormPreview(selectedVersion, formBuilderState.renderedVersion));
      }

      function renderEditableFormWorkspace() {
        const version = formBuilderState.form?.versions?.find((item) => item.id === formBuilderState.selectedVersionId);
        const rendered = formBuilderState.renderedVersion;
        if (!version || !rendered) {
          setHtml('form-version-workspace', emptyState('Create a draft version to start authoring sections and fields.'));
          return;
        }

        if (version.status !== 'draft') {
          setHtml('form-version-workspace', `
            <article class=\"record-card\">
              <h4>${escapeHtml(formVersionLabel(version))}</h4>
              <p class=\"muted\">This version is ${escapeHtml(formStatusLabel(version.status))} and is read-only.</p>
              <p class=\"muted\">Create a new draft version to change sections, fields, or ordering.</p>
            </article>
            ${renderFormPreview(version, rendered)}
          `);
          return;
        }

        const sections = rendered.sections.length
          ? rendered.sections.map((section, sectionIndex) => `
              <article class=\"record-card\">
                <div class=\"page-title-row compact-title-row\">
                  <div>
                    <h4>${escapeHtml(section.title)}</h4>
                    <p class=\"muted\">Draft section ${sectionIndex + 1}</p>
                  </div>
                  <div class=\"actions\">
                    <button type=\"button\" onclick=\"moveFormSection('${escapeHtml(section.id)}', -1)\">Move Up</button>
                    <button type=\"button\" onclick=\"moveFormSection('${escapeHtml(section.id)}', 1)\">Move Down</button>
                    <button type=\"button\" onclick=\"deleteFormSection('${escapeHtml(section.id)}')\">Delete</button>
                  </div>
                </div>
                <div class=\"form-grid\">
                  <div class=\"form-field wide-field\">
                    <label for=\"form-section-title-${escapeHtml(section.id)}\">Section Title</label>
                    <input class=\"input\" id=\"form-section-title-${escapeHtml(section.id)}\" type=\"text\" value=\"${escapeHtml(section.title)}\" />
                  </div>
                  <div class=\"form-field\">
                    <label for=\"form-section-position-${escapeHtml(section.id)}\">Display Order</label>
                    <input class=\"input\" id=\"form-section-position-${escapeHtml(section.id)}\" type=\"number\" value=\"${escapeHtml(section.position)}\" />
                  </div>
                </div>
                <div class=\"actions\">
                  <button type=\"button\" onclick=\"updateFormSection('${escapeHtml(section.id)}')\">Save Section</button>
                </div>
                <div class=\"record-list\">
                  ${section.fields.length
                    ? section.fields.map((field, fieldIndex) => `
                        <article class=\"record-card compact-record-card\">
                          <div class=\"page-title-row compact-title-row\">
                            <div>
                              <h4>${escapeHtml(field.label)}</h4>
                              <p class=\"muted\">Field ${fieldIndex + 1}</p>
                            </div>
                            <div class=\"actions\">
                              <button type=\"button\" onclick=\"moveFormField('${escapeHtml(field.id)}', -1)\">Move Up</button>
                              <button type=\"button\" onclick=\"moveFormField('${escapeHtml(field.id)}', 1)\">Move Down</button>
                              <button type=\"button\" onclick=\"deleteFormField('${escapeHtml(field.id)}')\">Delete</button>
                            </div>
                          </div>
                          <div class=\"form-grid\">
                            <div class=\"form-field\">
                              <label for=\"form-field-key-${escapeHtml(field.id)}\">Field Key</label>
                              <input class=\"input\" id=\"form-field-key-${escapeHtml(field.id)}\" type=\"text\" value=\"${escapeHtml(field.key)}\" />
                            </div>
                            <div class=\"form-field\">
                              <label for=\"form-field-label-${escapeHtml(field.id)}\">Label</label>
                              <input class=\"input\" id=\"form-field-label-${escapeHtml(field.id)}\" type=\"text\" value=\"${escapeHtml(field.label)}\" />
                            </div>
                            <div class=\"form-field\">
                              <label for=\"form-field-type-${escapeHtml(field.id)}\">Field Type</label>
                              <select class=\"input\" id=\"form-field-type-${escapeHtml(field.id)}\">
                                ${renderFormFieldTypeOptions(field.field_type)}
                              </select>
                            </div>
                            <div class=\"form-field\">
                              <label for=\"form-field-required-${escapeHtml(field.id)}\">Required</label>
                              <select class=\"input\" id=\"form-field-required-${escapeHtml(field.id)}\">
                                <option value=\"true\" ${field.required ? 'selected' : ''}>Required</option>
                                <option value=\"false\" ${field.required ? '' : 'selected'}>Optional</option>
                              </select>
                            </div>
                            <div class=\"form-field\">
                              <label for=\"form-field-position-${escapeHtml(field.id)}\">Display Order</label>
                              <input class=\"input\" id=\"form-field-position-${escapeHtml(field.id)}\" type=\"number\" value=\"${escapeHtml(field.position)}\" />
                            </div>
                            <div class=\"form-field\">
                              <label for=\"form-field-section-${escapeHtml(field.id)}\">Section</label>
                              <select class=\"input\" id=\"form-field-section-${escapeHtml(field.id)}\">
                                ${renderFormSectionOptions(section.id)}
                              </select>
                            </div>
                          </div>
                          <p class=\"muted\">Option-set and lookup touchpoints remain visible but read-only until backend metadata is available.</p>
                          <div class=\"actions\">
                            <button type=\"button\" onclick=\"updateFormField('${escapeHtml(field.id)}')\">Save Field</button>
                          </div>
                        </article>
                      `).join('')
                    : emptyState('No fields were added to this section yet.')}
                </div>
                <section class=\"page-panel nested-form-panel\">
                  <div class=\"page-title-row compact-title-row\">
                    <div>
                      <h4>Add Field</h4>
                      <p class=\"muted\">Create a new field inside this section.</p>
                    </div>
                  </div>
                  <div class=\"form-grid\">
                    <div class=\"form-field\">
                      <label for=\"new-form-field-key-${escapeHtml(section.id)}\">Field Key</label>
                      <input class=\"input\" id=\"new-form-field-key-${escapeHtml(section.id)}\" type=\"text\" />
                    </div>
                    <div class=\"form-field\">
                      <label for=\"new-form-field-label-${escapeHtml(section.id)}\">Label</label>
                      <input class=\"input\" id=\"new-form-field-label-${escapeHtml(section.id)}\" type=\"text\" />
                    </div>
                    <div class=\"form-field\">
                      <label for=\"new-form-field-type-${escapeHtml(section.id)}\">Field Type</label>
                      <select class=\"input\" id=\"new-form-field-type-${escapeHtml(section.id)}\">
                        ${renderFormFieldTypeOptions('text')}
                      </select>
                    </div>
                    <div class=\"form-field\">
                      <label for=\"new-form-field-required-${escapeHtml(section.id)}\">Required</label>
                      <select class=\"input\" id=\"new-form-field-required-${escapeHtml(section.id)}\">
                        <option value=\"false\" selected>Optional</option>
                        <option value=\"true\">Required</option>
                      </select>
                    </div>
                  </div>
                  <p class=\"muted\">Option-set and lookup anchors remain informational until backend metadata support lands.</p>
                  <div class=\"actions\">
                    <button type=\"button\" onclick=\"createFormField('${escapeHtml(section.id)}')\">Add Field</button>
                  </div>
                </section>
              </article>
            `).join('')
          : emptyState('No sections were added to this draft yet.');

        setHtml('form-version-workspace', `
          <section class=\"page-panel nested-form-panel\">
            <div class=\"page-title-row compact-title-row\">
              <div>
                <h3>${escapeHtml(formVersionLabel(version))}</h3>
                <p class=\"muted\">Draft version workspace for ${escapeHtml(formBuilderState.form.name)}</p>
              </div>
              <div class=\"actions\">
                <button type=\"button\" onclick=\"publishSelectedFormVersion()\">Publish Draft Version</button>
              </div>
            </div>
            <p class=\"muted\">Compatibility: ${escapeHtml(formVersionCompatibility(version))}</p>
            ${formVersionLifecycleSummary(version)}
            <p class=\"muted\">Publish attempts surface validation errors here before the route reloads.</p>
          </section>
          <section class=\"page-panel nested-form-panel\">
            <div class=\"page-title-row compact-title-row\">
              <div>
                <h3>Add Section</h3>
                <p class=\"muted\">Create a new section for the selected draft version.</p>
              </div>
            </div>
            <div class=\"form-grid\">
              <div class=\"form-field wide-field\">
                <label for=\"new-form-section-title\">Section Title</label>
                <input class=\"input\" id=\"new-form-section-title\" type=\"text\" />
              </div>
            </div>
            <div class=\"actions\">
              <button type=\"button\" onclick=\"createFormSection()\">Add Section</button>
            </div>
          </section>
          ${sections}
        `);
      }

      async function loadEditableFormSurface(id, preferredVersionId = '') {
        const payload = await request(`/api/admin/forms/${id}`);
        formBuilderState.form = payload;
        selectRecord('form', payload.name, payload.id);
        byId('form-name').value = payload.name || '';
        byId('form-slug').value = payload.slug || '';
        byId('form-scope-node-type').value = payload.scope_node_type_id || '';
        const selectedVersion = choosePreferredFormVersion(payload, preferredVersionId);
        setHtml('form-version-list', renderFormVersionCards(payload, selectedVersion?.id || '', true));
        if (!selectedVersion) {
          formBuilderState.selectedVersionId = '';
          formBuilderState.renderedVersion = null;
          setHtml('form-version-workspace', emptyState('Create a draft version to start authoring sections and fields.'));
          return;
        }
        formBuilderState.selectedVersionId = selectedVersion.id;
        formBuilderState.renderedVersion = await request(`/api/form-versions/${selectedVersion.id}/render`);
        setHtml('form-version-list', renderFormVersionCards(payload, selectedVersion.id, true));
        renderEditableFormWorkspace();
      }

      async function previewFormVersion(formVersionId) {
        try {
          if (!page.recordId) throw new Error('Choose a form first.');
          if (page.key === 'form-edit') {
            await loadEditableFormSurface(page.recordId, formVersionId);
          } else {
            await loadReadableFormSurface(page.recordId, formVersionId);
          }
        } catch (error) {
          setFormStatus('form-version-status', error.message, 'error');
          show(error.message);
        }
      }

      function currentRenderedSection(sectionId) {
        return (formBuilderState.renderedVersion?.sections || []).find((section) => section.id === sectionId);
      }

      function currentRenderedField(fieldId) {
        for (const section of formBuilderState.renderedVersion?.sections || []) {
          const field = section.fields.find((item) => item.id === fieldId);
          if (field) {
            return { field, section };
          }
        }
        return null;
      }

      function nextSectionPosition() {
        return (formBuilderState.renderedVersion?.sections || []).reduce((max, section) => Math.max(max, Number(section.position) || 0), 0) + 1;
      }

      function nextFieldPosition(sectionId) {
        const section = currentRenderedSection(sectionId);
        return (section?.fields || []).reduce((max, field) => Math.max(max, Number(field.position) || 0), 0) + 1;
      }

      async function createFormVersion() {
        try {
          if (!page.recordId) throw new Error('Choose a form first.');
          const response = await runLockedFormAction('form-version-create', async () => {
            setFormStatus('form-version-status', 'Creating draft version...');
            return request(`/api/admin/forms/${page.recordId}/versions`, {
              method: 'POST',
              headers: { 'Content-Type': 'application/json' },
              body: JSON.stringify({})
            });
          });
          setFormStatus('form-version-status', 'Draft version created.', 'success');
          await loadEditableFormSurface(page.recordId, response.id);
        } catch (error) {
          setFormStatus('form-version-status', error.message, 'error');
          show(error.message);
        }
      }

      async function publishSelectedFormVersion(versionId = '') {
        try {
          const selectedVersionId = versionId || formBuilderState.selectedVersionId;
          if (!selectedVersionId) throw new Error('Select a draft version first.');
          const response = await runLockedFormAction('form-version-publish', async () => {
            setFormStatus('form-version-status', 'Publishing draft version...');
            return request(`/api/admin/form-versions/${selectedVersionId}/publish`, {
              method: 'POST'
            });
          });
          const warningSuffix = response.dependency_warnings?.length
            ? ` ${response.dependency_warnings.length} direct dependency warning(s) need review.`
            : '';
          setFormStatus(
            'form-version-status',
            `Draft version published as ${response.version_label}.${warningSuffix}`.trim(),
            'success'
          );
          await loadEditableFormSurface(page.recordId, selectedVersionId);
        } catch (error) {
          setFormStatus('form-version-status', error.message, 'error');
          show(error.message);
        }
      }

      async function createFormSection() {
        try {
          if (!formBuilderState.selectedVersionId) throw new Error('Select a draft version first.');
          await runLockedFormAction('form-section-create', async () => {
            setFormStatus('form-version-status', 'Creating section...');
            await request(`/api/admin/form-versions/${formBuilderState.selectedVersionId}/sections`, {
              method: 'POST',
              headers: { 'Content-Type': 'application/json' },
              body: JSON.stringify({
                title: byId('new-form-section-title').value.trim(),
                position: nextSectionPosition()
              })
            });
          });
          byId('new-form-section-title').value = '';
          setFormStatus('form-version-status', 'Section created.', 'success');
          await loadEditableFormSurface(page.recordId, formBuilderState.selectedVersionId);
        } catch (error) {
          setFormStatus('form-version-status', error.message, 'error');
          show(error.message);
        }
      }

      async function updateFormSection(sectionId) {
        try {
          await runLockedFormAction(`form-section-update:${sectionId}`, async () => {
            setFormStatus('form-version-status', 'Saving section...');
            await request(`/api/admin/form-sections/${sectionId}`, {
              method: 'PUT',
              headers: { 'Content-Type': 'application/json' },
              body: JSON.stringify({
                title: byId(`form-section-title-${sectionId}`).value.trim(),
                position: Number(byId(`form-section-position-${sectionId}`).value || 0)
              })
            });
          });
          setFormStatus('form-version-status', 'Section saved.', 'success');
          await loadEditableFormSurface(page.recordId, formBuilderState.selectedVersionId);
        } catch (error) {
          setFormStatus('form-version-status', error.message, 'error');
          show(error.message);
        }
      }

      async function deleteFormSection(sectionId) {
        try {
          await runLockedFormAction(`form-section-delete:${sectionId}`, async () => {
            setFormStatus('form-version-status', 'Deleting section...');
            await request(`/api/admin/form-sections/${sectionId}`, {
              method: 'DELETE'
            });
          });
          setFormStatus('form-version-status', 'Section deleted.', 'success');
          await loadEditableFormSurface(page.recordId, formBuilderState.selectedVersionId);
        } catch (error) {
          setFormStatus('form-version-status', error.message, 'error');
          show(error.message);
        }
      }

      async function moveFormSection(sectionId, direction) {
        try {
          const sections = [...(formBuilderState.renderedVersion?.sections || [])].sort((left, right) => left.position - right.position);
          const currentIndex = sections.findIndex((section) => section.id === sectionId);
          const targetIndex = currentIndex + direction;
          if (currentIndex < 0 || targetIndex < 0 || targetIndex >= sections.length) {
            return;
          }
          const current = sections[currentIndex];
          const target = sections[targetIndex];
          await runLockedFormAction(`form-section-move:${sectionId}`, async () => {
            setFormStatus('form-version-status', 'Reordering sections...');
            await request(`/api/admin/form-sections/${current.id}`, {
              method: 'PUT',
              headers: { 'Content-Type': 'application/json' },
              body: JSON.stringify({ title: current.title, position: target.position })
            });
            await request(`/api/admin/form-sections/${target.id}`, {
              method: 'PUT',
              headers: { 'Content-Type': 'application/json' },
              body: JSON.stringify({ title: target.title, position: current.position })
            });
          });
          setFormStatus('form-version-status', 'Section order updated.', 'success');
          await loadEditableFormSurface(page.recordId, formBuilderState.selectedVersionId);
        } catch (error) {
          setFormStatus('form-version-status', error.message, 'error');
          show(error.message);
        }
      }

      async function createFormField(sectionId) {
        try {
          if (!formBuilderState.selectedVersionId) throw new Error('Select a draft version first.');
          await runLockedFormAction(`form-field-create:${sectionId}`, async () => {
            setFormStatus('form-version-status', 'Creating field...');
            await request(`/api/admin/form-versions/${formBuilderState.selectedVersionId}/fields`, {
              method: 'POST',
              headers: { 'Content-Type': 'application/json' },
              body: JSON.stringify({
                section_id: sectionId,
                key: byId(`new-form-field-key-${sectionId}`).value.trim(),
                label: byId(`new-form-field-label-${sectionId}`).value.trim(),
                field_type: byId(`new-form-field-type-${sectionId}`).value,
                required: byId(`new-form-field-required-${sectionId}`).value === 'true',
                position: nextFieldPosition(sectionId)
              })
            });
          });
          byId(`new-form-field-key-${sectionId}`).value = '';
          byId(`new-form-field-label-${sectionId}`).value = '';
          byId(`new-form-field-type-${sectionId}`).value = 'text';
          byId(`new-form-field-required-${sectionId}`).value = 'false';
          setFormStatus('form-version-status', 'Field created.', 'success');
          await loadEditableFormSurface(page.recordId, formBuilderState.selectedVersionId);
        } catch (error) {
          setFormStatus('form-version-status', error.message, 'error');
          show(error.message);
        }
      }

      async function updateFormField(fieldId) {
        try {
          await runLockedFormAction(`form-field-update:${fieldId}`, async () => {
            setFormStatus('form-version-status', 'Saving field...');
            await request(`/api/admin/form-fields/${fieldId}`, {
              method: 'PUT',
              headers: { 'Content-Type': 'application/json' },
              body: JSON.stringify({
                section_id: byId(`form-field-section-${fieldId}`).value,
                key: byId(`form-field-key-${fieldId}`).value.trim(),
                label: byId(`form-field-label-${fieldId}`).value.trim(),
                field_type: byId(`form-field-type-${fieldId}`).value,
                required: byId(`form-field-required-${fieldId}`).value === 'true',
                position: Number(byId(`form-field-position-${fieldId}`).value || 0)
              })
            });
          });
          setFormStatus('form-version-status', 'Field saved.', 'success');
          await loadEditableFormSurface(page.recordId, formBuilderState.selectedVersionId);
        } catch (error) {
          setFormStatus('form-version-status', error.message, 'error');
          show(error.message);
        }
      }

      async function deleteFormField(fieldId) {
        try {
          await runLockedFormAction(`form-field-delete:${fieldId}`, async () => {
            setFormStatus('form-version-status', 'Deleting field...');
            await request(`/api/admin/form-fields/${fieldId}`, {
              method: 'DELETE'
            });
          });
          setFormStatus('form-version-status', 'Field deleted.', 'success');
          await loadEditableFormSurface(page.recordId, formBuilderState.selectedVersionId);
        } catch (error) {
          setFormStatus('form-version-status', error.message, 'error');
          show(error.message);
        }
      }

      async function moveFormField(fieldId, direction) {
        try {
          const existing = currentRenderedField(fieldId);
          if (!existing) throw new Error('The selected field is no longer available. Reload the page and try again.');
          const fields = [...existing.section.fields].sort((left, right) => left.position - right.position);
          const currentIndex = fields.findIndex((field) => field.id === fieldId);
          const targetIndex = currentIndex + direction;
          if (currentIndex < 0 || targetIndex < 0 || targetIndex >= fields.length) {
            return;
          }
          const current = fields[currentIndex];
          const target = fields[targetIndex];
          await runLockedFormAction(`form-field-move:${fieldId}`, async () => {
            setFormStatus('form-version-status', 'Reordering fields...');
            await request(`/api/admin/form-fields/${current.id}`, {
              method: 'PUT',
              headers: { 'Content-Type': 'application/json' },
              body: JSON.stringify({
                section_id: existing.section.id,
                key: current.key,
                label: current.label,
                field_type: current.field_type,
                required: current.required,
                position: target.position
              })
            });
            await request(`/api/admin/form-fields/${target.id}`, {
              method: 'PUT',
              headers: { 'Content-Type': 'application/json' },
              body: JSON.stringify({
                section_id: existing.section.id,
                key: target.key,
                label: target.label,
                field_type: target.field_type,
                required: target.required,
                position: current.position
              })
            });
          });
          setFormStatus('form-version-status', 'Field order updated.', 'success');
          await loadEditableFormSurface(page.recordId, formBuilderState.selectedVersionId);
        } catch (error) {
          setFormStatus('form-version-status', error.message, 'error');
          show(error.message);
        }
      }

      async function loadResponsesList() {
        try {
          await ensureAuthenticated();
          renderDelegateContextSwitcher('response-context-switcher');
          const [responseOptions, drafts, submitted] = await Promise.all([
            request(withRespondentQuery('/api/responses/options')),
            request(withRespondentQuery('/api/submissions?status=draft')),
            request(withRespondentQuery('/api/submissions?status=submitted'))
          ]);
          setHtml(
            'response-pending-list',
            responseOptions.mode === 'assignment'
              ? (
                  responseOptions.assignments.length
                    ? responseOptions.assignments.map((item) => recordCard(
                        `${item.form_name} ${item.version_label}`,
                        `<p>${escapeHtml(item.node_name)}</p><p class=\"muted\">${escapeHtml(item.delegate_display_name || 'Assigned account')}</p>`,
                        `<a class=\"button-link\" href=\"/app/responses/new?formVersionId=${item.form_version_id}&nodeId=${item.node_id}${item.delegate_account_id ? `&delegateAccountId=${item.delegate_account_id}` : ''}\">Start</a>`
                      )).join('')
                    : emptyState('No assigned responses are ready to start.')
                )
              : (
                  responseOptions.published_forms.length
                    ? responseOptions.published_forms.map((item) => recordCard(
                        `${item.form_name} ${item.version_label}`,
                        `<p class=\"muted\">Published form</p>`,
                        `<a class=\"button-link\" href=\"/app/responses/new?formVersionId=${item.form_version_id}\">Start</a>`
                      )).join('')
                    : emptyState('No published forms are ready for new responses.')
                )
          );
          setHtml(
            'response-draft-list',
            drafts.length
              ? drafts.map((item) => recordCard(
                  `${item.form_name} ${item.version_label}`,
                  `<p>${escapeHtml(item.node_name)}</p><p class=\"muted\">Draft</p>`,
                  `<a class=\"button-link\" href=\"/app/responses/${item.id}\">View</a><a class=\"button-link\" href=\"/app/responses/${item.id}/edit\">Edit</a>`
                )).join('')
              : emptyState('No draft responses found.')
          );
          setHtml(
            'response-submitted-list',
            submitted.length
              ? submitted.map((item) => recordCard(
                  `${item.form_name} ${item.version_label}`,
                  `<p>${escapeHtml(item.node_name)}</p><p class=\"muted\">Submitted</p>`,
                  `<a class=\"button-link\" href=\"/app/responses/${item.id}\">View</a>`
                )).join('')
              : emptyState('No submitted responses found.')
          );
          show({ responseOptions, drafts, submitted });
        } catch (error) {
          setHtml('response-pending-list', emptyState(error.message));
          setHtml('response-draft-list', emptyState(error.message));
          setHtml('response-submitted-list', emptyState(error.message));
        }
      }

      async function initResponseCreateForm() {
        try {
          await ensureAuthenticated();
          const queryDelegateAccountId = page.search.get('delegateAccountId');
          if (queryDelegateAccountId) setDelegateContext(queryDelegateAccountId);
          renderDelegateContextSwitcher('response-create-context-switcher');
          const options = await request(withRespondentQuery('/api/responses/options'));
          if (options.mode === 'assignment') {
            setSelectOptions(
              'response-form-version',
              options.assignments.map((item) => ({ value: item.form_version_id, label: `${item.form_name} ${item.version_label}` })),
              'Choose assigned form'
            );
            setSelectOptions(
              'response-node',
              options.assignments.map((item) => ({ value: item.node_id, label: item.node_name })),
              'Choose assigned organization'
            );
          } else {
            setSelectOptions(
              'response-form-version',
              options.published_forms.map((item) => ({ value: item.form_version_id, label: `${item.form_name} ${item.version_label}` })),
              'Choose published form'
            );
            setSelectOptions(
              'response-node',
              options.nodes.map((item) => ({ value: item.id, label: item.name })),
              'Choose target organization'
            );
          }
          const queryFormVersion = page.search.get('formVersionId');
          const queryNodeId = page.search.get('nodeId');
          if (queryFormVersion) byId('response-form-version').value = queryFormVersion;
          if (queryNodeId) byId('response-node').value = queryNodeId;
          const form = byId('response-start-form');
          if (form) {
            form.onsubmit = async (event) => {
              event.preventDefault();
              const response = await request('/api/submissions/drafts', {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({
                  form_version_id: byId('response-form-version').value,
                  node_id: byId('response-node').value,
                  delegate_account_id: currentDelegateContext || null
                })
              });
              redirect(`/app/responses/${response.id}/edit`);
            };
          }
        } catch (error) {
          show(error.message);
        }
      }

      async function loadResponseDetail(id) {
        try {
          await ensureAuthenticated();
          const payload = await request(`/api/submissions/${id}`);
          currentResponseDetail = payload;
          selectRecord('response', `${payload.form_name} ${payload.version_label}`, payload.id);
          const values = payload.values.length
            ? payload.values.map((item) => `<li>${escapeHtml(item.label)}: ${item.value === null ? '<span class=\"muted\">missing</span>' : escapeHtml(JSON.stringify(item.value))}</li>`).join('')
            : '<li class=\"muted\">No saved values.</li>';
          const audit = payload.audit_events.length
            ? payload.audit_events.map((item) => `<li>${escapeHtml(item.event_type)} by ${escapeHtml(item.account_email || 'system')} on ${escapeHtml(item.created_at)}</li>`).join('')
            : '<li class=\"muted\">No audit events.</li>';
          setHtml('response-detail', `
            ${detailSection('Summary', `<p>${escapeHtml(payload.form_name)} ${escapeHtml(payload.version_label)}</p><p>${escapeHtml(payload.node_name)}</p><p class=\"muted\">Status: ${escapeHtml(payload.status)}</p>${payload.status === 'draft' ? `<p><a class=\"button-link\" href=\"/app/responses/${payload.id}/edit\">Edit Draft</a></p>` : ''}`)}
            ${detailSection('Values', `<ul class=\"app-list\">${values}</ul>`)}
            ${detailSection('Audit Trail', `<ul class=\"app-list\">${audit}</ul>`)}
          `);
          show(payload);
        } catch (error) {
          setHtml('response-detail', emptyState(error.message));
        }
      }

      async function initResponseEditPage(id) {
        try {
          await ensureAuthenticated();
          const detail = await request(`/api/submissions/${id}`);
          currentResponseDetail = detail;
          if (detail.status !== 'draft') {
            setHtml('response-edit-surface', `
              <p class=\"muted\">This response is submitted and cannot be edited.</p>
              <div class=\"actions\"><a class=\"button-link\" href=\"/app/responses/${detail.id}\">Back to Detail</a></div>
            `);
            return;
          }
          const rendered = await request(`/api/form-versions/${detail.form_version_id}/render`);
          renderedResponseForm = rendered;
          selectRecord('response', `${detail.form_name} ${detail.version_label}`, detail.id);
          setHtml('response-edit-surface', `
            <form id=\"response-edit-form\" class=\"entity-form\">
              ${rendered.sections.map((section) => `
                <section class=\"page-panel nested-form-panel\">
                  <h3>${escapeHtml(section.title)}</h3>
                  <div class=\"form-grid\">
                    ${section.fields.map((field) => `
                      <div class=\"form-field\">
                        <label for=\"${escapeHtml(fieldInputId(field))}\">${escapeHtml(field.label)}${field.required ? ' *' : ''}</label>
                        ${renderFieldInput(field)}
                      </div>
                    `).join('')}
                  </div>
                </section>
              `).join('')}
              <div class=\"actions form-actions\">
                <button type=\"button\" onclick=\"saveCurrentResponseDraft()\">Save</button>
                <button type=\"button\" onclick=\"submitCurrentResponseDraft()\">Submit</button>
                <a class=\"button-link\" href=\"/app/responses/${detail.id}\">Cancel</a>
              </div>
            </form>
          `);
          prefillRenderedResponseValues(detail);
        } catch (error) {
          setHtml('response-edit-surface', emptyState(error.message));
        }
      }

      async function saveCurrentResponseDraft() {
        try {
          const values = collectRenderedResponseValues();
          validateRenderedResponseValues(values);
          await request(`/api/submissions/${page.recordId}/values`, {
            method: 'PUT',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({ values })
          });
          redirect(`/app/responses/${page.recordId}`);
        } catch (error) {
          show(error.message);
        }
      }

      async function submitCurrentResponseDraft() {
        try {
          const values = collectRenderedResponseValues();
          validateRenderedResponseValues(values);
          await request(`/api/submissions/${page.recordId}/values`, {
            method: 'PUT',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({ values })
          });
          await request(`/api/submissions/${page.recordId}/submit`, { method: 'POST' });
          redirect(`/app/responses/${page.recordId}`);
        } catch (error) {
          show(error.message);
        }
      }

      async function loadReportsList() {
        try {
          await ensureAuthenticated();
          const payload = await request('/api/reports');
          const html = payload.length
            ? payload.map((report) => recordCard(
                report.name,
                `<p class=\"muted\">${escapeHtml(report.dataset_name || report.form_name || 'Unknown source')}</p>`,
                `<a class=\"button-link\" href=\"/app/reports/${report.id}\">View</a>${isAdmin() ? `<a class=\"button-link\" href=\"/app/reports/${report.id}/edit\">Edit</a>` : ''}`
              )).join('')
            : emptyState('No report records found.');
          setHtml('report-list', html);
          show(payload);
        } catch (error) {
          setHtml('report-list', emptyState(error.message));
        }
      }

      async function loadReportDetail(id) {
        try {
          await ensureAuthenticated();
          const payload = await request(`/api/reports/${id}`);
          selectRecord('report', payload.name, payload.id);
          const bindings = payload.bindings.length
            ? payload.bindings.map((binding) => `<li>${escapeHtml(binding.logical_key)} -> ${escapeHtml(binding.source_field_key || binding.computed_expression || 'computed')} (${escapeHtml(binding.missing_policy)})</li>`).join('')
            : '<li class=\"muted\">No bindings configured.</li>';
          const aggregations = payload.aggregations.length
            ? payload.aggregations.map((aggregation) => `<li>${escapeHtml(aggregation.name)}</li>`).join('')
            : '<li class=\"muted\">No related aggregations.</li>';
          const charts = payload.charts.length
            ? payload.charts.map((chart) => `<li>${escapeHtml(chart.name)}</li>`).join('')
            : '<li class=\"muted\">No related charts.</li>';
          setHtml('report-detail', `
            ${detailSection('Summary', `<p>${escapeHtml(payload.name)}</p><p class=\"muted\">Source: ${escapeHtml(payload.dataset_name || payload.form_name || 'Unknown')}</p>`)}
            ${detailSection('Bindings', `<ul class=\"app-list\">${bindings}</ul>`)}
            ${detailSection('Related Aggregations', `<ul class=\"app-list\">${aggregations}</ul>`)}
            ${detailSection('Related Charts', `<ul class=\"app-list\">${charts}</ul>`)}
          `);
          show(payload);
        } catch (error) {
          setHtml('report-detail', emptyState(error.message));
        }
      }

      function blankBinding() {
        return {
          logical_key: '',
          source_field_key: '',
          computed_expression: '',
          missing_policy: 'null'
        };
      }

      function renderReportBindingRows() {
        setHtml(
          'report-binding-rows',
          reportFormState.bindings.map((binding, index) => `
            <article class=\"binding-row\">
              <div class=\"form-grid\">
                <div class=\"form-field\">
                  <label for=\"binding-logical-${index}\">Logical Key</label>
                  <input id=\"binding-logical-${index}\" type=\"text\" value=\"${escapeHtml(binding.logical_key || '')}\">
                </div>
                <div class=\"form-field\">
                  <label for=\"binding-source-${index}\">Source Field</label>
                  <input id=\"binding-source-${index}\" type=\"text\" value=\"${escapeHtml(binding.source_field_key || '')}\">
                </div>
                <div class=\"form-field\">
                  <label for=\"binding-computed-${index}\">Computed Expression</label>
                  <input id=\"binding-computed-${index}\" type=\"text\" value=\"${escapeHtml(binding.computed_expression || '')}\">
                </div>
                <div class=\"form-field\">
                  <label for=\"binding-policy-${index}\">Missing Policy</label>
                  <select id=\"binding-policy-${index}\">
                    <option value=\"null\" ${binding.missing_policy === 'null' ? 'selected' : ''}>null</option>
                    <option value=\"exclude_row\" ${binding.missing_policy === 'exclude_row' ? 'selected' : ''}>exclude_row</option>
                    <option value=\"bucket_unknown\" ${binding.missing_policy === 'bucket_unknown' ? 'selected' : ''}>bucket_unknown</option>
                  </select>
                </div>
              </div>
              <div class=\"actions\">
                <button type=\"button\" onclick=\"removeReportBindingRow(${index})\">Remove</button>
              </div>
            </article>
          `).join('')
        );
      }

      function addReportBindingRow() {
        reportFormState.bindings.push(blankBinding());
        renderReportBindingRows();
      }

      function removeReportBindingRow(index) {
        reportFormState.bindings.splice(index, 1);
        if (reportFormState.bindings.length === 0) {
          reportFormState.bindings.push(blankBinding());
        }
        renderReportBindingRows();
      }

      function collectReportBindings() {
        return reportFormState.bindings.map((_, index) => ({
          logical_key: byId(`binding-logical-${index}`).value.trim(),
          source_field_key: byId(`binding-source-${index}`).value.trim() || null,
          computed_expression: byId(`binding-computed-${index}`).value.trim() || null,
          missing_policy: byId(`binding-policy-${index}`).value
        }));
      }

      function renderReportSourceOptions() {
        const sourceType = byId('report-source-type').value;
        const options = (sourceType === 'dataset' ? reportFormState.datasets : reportFormState.forms)
          .map((item) => ({
            value: item.id,
            label: sourceType === 'dataset'
              ? item.name
              : item.name
          }));
        setSelectOptions('report-source-id', options, `Choose ${sourceType}`);
      }

      async function initReportForm(mode, id) {
        try {
          await ensureAuthenticated();
          const [forms, datasets] = await Promise.all([
            request('/api/forms'),
            request('/api/datasets')
          ]);
          reportFormState = {
            forms,
            datasets,
            bindings: [blankBinding()]
          };
          byId('report-source-type').onchange = renderReportSourceOptions;
          renderReportSourceOptions();
          if (mode === 'edit' && id) {
            const payload = await request(`/api/reports/${id}`);
            byId('report-name').value = payload.name || '';
            if (payload.dataset_id) {
              byId('report-source-type').value = 'dataset';
            }
            renderReportSourceOptions();
            byId('report-source-id').value = payload.dataset_id || payload.form_id || '';
            reportFormState.bindings = payload.bindings.map((binding) => ({
              logical_key: binding.logical_key,
              source_field_key: binding.source_field_key || '',
              computed_expression: binding.computed_expression || '',
              missing_policy: binding.missing_policy
            }));
          }
          renderReportBindingRows();
          const form = byId('report-form');
          if (form) {
            form.onsubmit = async (event) => {
              event.preventDefault();
              const sourceType = byId('report-source-type').value;
              const sourceId = byId('report-source-id').value;
              const payload = {
                name: byId('report-name').value.trim(),
                form_id: sourceType === 'form' ? sourceId : null,
                dataset_id: sourceType === 'dataset' ? sourceId : null,
                fields: collectReportBindings()
              };
              const response = await request(
                mode === 'create' ? '/api/admin/reports' : `/api/admin/reports/${id}`,
                {
                  method: mode === 'create' ? 'POST' : 'PUT',
                  headers: { 'Content-Type': 'application/json' },
                  body: JSON.stringify(payload)
                }
              );
              redirect(`/app/reports/${response.id}`);
            };
          }
        } catch (error) {
          show(error.message);
        }
      }

      async function runCurrentReport() {
        try {
          await ensureAuthenticated();
          const payload = await request(`/api/reports/${page.recordId}/table`);
          setHtml('report-results', payload.rows.length
            ? `<div class=\"table-wrap\"><table><thead><tr><th>Node</th><th>Source</th><th>Field</th><th>Value</th><th>Response</th></tr></thead><tbody>${payload.rows.map((row) => `<tr><td>${escapeHtml(row.node_name || 'Unknown node')}</td><td>${escapeHtml(row.source_alias || 'Direct')}</td><td>${escapeHtml(row.logical_key || '')}</td><td>${escapeHtml(row.field_value ?? '')}</td><td>${row.submission_id ? `<a href=\"/app/responses/${row.submission_id}\">View</a>` : '<span class=\"muted\">None</span>'}</td></tr>`).join('')}</tbody></table></div>`
            : emptyState('No submitted rows matched this report.'));
          show(payload);
        } catch (error) {
          setHtml('report-results', emptyState(error.message));
        }
      }

      async function loadDashboardsList() {
        try {
          await ensureAuthenticated();
          const payload = await request('/api/dashboards');
          const html = payload.length
            ? payload.map((dashboard) => recordCard(
                dashboard.name,
                `<p class=\"muted\">${dashboard.component_count} components</p>`,
                `<a class=\"button-link\" href=\"/app/dashboards/${dashboard.id}\">View</a>${isAdmin() ? `<a class=\"button-link\" href=\"/app/dashboards/${dashboard.id}/edit\">Edit</a>` : ''}`
              )).join('')
            : emptyState('No dashboard records found.');
          setHtml('dashboard-list', html);
          show(payload);
        } catch (error) {
          setHtml('dashboard-list', emptyState(error.message));
        }
      }

      async function loadDashboardDetail(id) {
        try {
          await ensureAuthenticated();
          const payload = await request(`/api/dashboards/${id}`);
          selectRecord('dashboard', payload.name, payload.id);
          const components = payload.components.length
            ? payload.components.map((component) => `
                <article class=\"record-card compact-record-card\">
                  <h4>${escapeHtml(component.config?.title || component.chart.name)}</h4>
                  <p>${escapeHtml(component.chart.chart_type)} chart</p>
                  <p class=\"muted\">Chart: ${escapeHtml(component.chart.name)}</p>
                  <p class=\"muted\">Source: ${escapeHtml(component.chart.report_name || component.chart.aggregation_name || 'Unknown')}</p>
                </article>
              `).join('')
            : emptyState('No dashboard components found.');
          setHtml('dashboard-detail', `
            ${detailSection('Summary', `<p>${escapeHtml(payload.name)}</p><p class=\"muted\">${payload.components.length} components</p>`)}
            ${detailSection('Component Summary', components)}
          `);
          show(payload);
        } catch (error) {
          setHtml('dashboard-detail', emptyState(error.message));
        }
      }

      function viewCurrentDashboard() {
        loadDashboardDetail(page.recordId);
      }

      async function initDashboardForm(mode, id) {
        try {
          await ensureAuthenticated();
          if (mode === 'edit' && id) {
            const payload = await request(`/api/dashboards/${id}`);
            byId('dashboard-name').value = payload.name || '';
          }
          const form = byId('dashboard-form');
          if (form) {
            form.onsubmit = async (event) => {
              event.preventDefault();
              const response = await request(
                mode === 'create' ? '/api/admin/dashboards' : `/api/admin/dashboards/${id}`,
                {
                  method: mode === 'create' ? 'POST' : 'PUT',
                  headers: { 'Content-Type': 'application/json' },
                  body: JSON.stringify({ name: byId('dashboard-name').value.trim() })
                }
              );
              redirect(`/app/dashboards/${response.id}`);
            };
          }
        } catch (error) {
          show(error.message);
        }
      }

      async function loadLegacyFixtureExamples() {
        try {
          await ensureAuthenticated();
          const payload = await request('/api/admin/legacy-fixtures/examples');
          window.legacyFixtureExamples = Object.fromEntries(payload.map((fixture) => [fixture.name, fixture.fixture_json]));
          setHtml(
            'migration-list',
            payload.length
              ? payload.map((fixture) => recordCard(
                  fixture.name,
                  `<p class=\"muted\">${fixture.fixture_json.length} bytes</p>`,
                  `<button type=\"button\" onclick=\"useLegacyFixture('${escapeHtml(fixture.name)}')\">Use Fixture</button>`
                )).join('')
              : emptyState('No fixture examples available.')
          );
          show(payload.map((fixture) => ({ name: fixture.name, bytes: fixture.fixture_json.length })));
        } catch (error) {
          setHtml('migration-list', emptyState(error.message));
        }
      }

      function useLegacyFixture(name) {
        const fixture = window.legacyFixtureExamples?.[name];
        if (!fixture) {
          show(`Fixture example '${name}' has not been loaded.`);
          return;
        }
        byId('legacy-fixture-json').value = fixture;
        show({ selected_fixture: name });
      }

      async function validateLegacyFixture() {
        try {
          await ensureAuthenticated();
          const payload = await request('/api/admin/legacy-fixtures/validate', {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({ fixture_json: byId('legacy-fixture-json').value.trim() })
          });
          setHtml(
            'migration-results',
            payload.issue_count
              ? payload.issues.map((issue) => recordCard(issue.code, `<p>${escapeHtml(issue.path)}</p><p>${escapeHtml(issue.message)}</p>`, '')).join('')
              : '<p class=\"muted\">Legacy fixture validation passed.</p>'
          );
          show(payload);
        } catch (error) {
          setHtml('migration-results', emptyState(error.message));
        }
      }

      async function dryRunLegacyFixture() {
        try {
          await ensureAuthenticated();
          const payload = await request('/api/admin/legacy-fixtures/dry-run', {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({ fixture_json: byId('legacy-fixture-json').value.trim() })
          });
          setHtml('migration-results', `<article class=\"record-card\"><h4>${escapeHtml(payload.fixture_name)}</h4><p>Would import: ${payload.would_import ? 'yes' : 'no'}</p><p class=\"muted\">Validation issues: ${payload.validation.issue_count}</p></article>`);
          show(payload);
        } catch (error) {
          setHtml('migration-results', emptyState(error.message));
        }
      }

      async function importLegacyFixture() {
        try {
          await ensureAuthenticated();
          const payload = await request('/api/admin/legacy-fixtures/import', {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({ fixture_json: byId('legacy-fixture-json').value.trim() })
          });
          setHtml('migration-results', `<article class=\"record-card\"><h4>Import Complete</h4><p>Form version: ${escapeHtml(payload.form_version_id)}</p><p>Submission: ${escapeHtml(payload.submission_id)}</p><p class=\"muted\">Dashboard: ${escapeHtml(payload.dashboard_id)}</p></article>`);
          show(payload);
        } catch (error) {
          setHtml('migration-results', emptyState(error.message));
        }
      }

      function applyPageActionVisibility() {
        if (!currentAccount) return;
        if (!isAdmin()) {
          for (const link of document.querySelectorAll('.page-title-row .actions a')) {
            const href = link.getAttribute('href') || '';
            if (
              href.includes('/edit')
              || href.endsWith('/new')
              || href === '/app/organization/new'
              || href === '/app/forms/new'
              || href === '/app/reports/new'
              || href === '/app/dashboards/new'
              || href === '/app/administration/node-types/new'
            ) {
              link.remove();
            }
          }
        }
      }

      async function initPage() {
        updateSessionStatus();
        renderSelections();
        if (page.key === 'login') {
          await initLoginPage();
          return;
        }

        let bootstrapError = null;
        try {
          await bootstrapCurrentAccount();
        } catch (error) {
          bootstrapError = error;
          currentAccount = null;
          applyRoleVisibility();
          updateSessionStatus();
        }

        if (!canAccessCurrentPage()) {
          if (bootstrapError && !currentAccount) {
            renderAccessState('Sign In Required', 'This screen requires an authenticated local account.');
            show(bootstrapError.message);
            return;
          }
          renderAccessState('Access Restricted', 'Your current role does not have access to this screen.');
          return;
        }

        applyPageActionVisibility();

        switch (page.key) {
          case 'home':
            await initHomePage();
            break;
          case 'administration':
            await ensureAuthenticated();
            updateSessionStatus();
            break;
          case 'user-list':
            await loadUsersList();
            break;
          case 'user-detail':
            await loadUserDetail(page.recordId);
            break;
          case 'user-create':
            await initUserForm('create');
            break;
          case 'user-edit':
            await initUserForm('edit', page.recordId);
            break;
          case 'user-access':
            await initUserAccessForm(page.recordId);
            break;
          case 'node-type-list':
            await loadOrganizationNodeTypesList();
            break;
          case 'node-type-detail':
            await loadOrganizationNodeTypeDetail(page.recordId);
            break;
          case 'node-type-create':
            await initOrganizationNodeTypeForm('create');
            break;
          case 'node-type-edit':
            await initOrganizationNodeTypeForm('edit', page.recordId);
            break;
          case 'role-list':
            await loadRolesList();
            break;
          case 'role-detail':
            await loadRoleDetail(page.recordId);
            break;
          case 'role-create':
            await initRoleForm('create');
            break;
          case 'role-edit':
            await initRoleForm('edit', page.recordId);
            break;
          case 'migration':
            updateSessionStatus(currentAccount);
            break;
          case 'form-list':
            await loadFormsList();
            break;
          case 'form-detail':
            await loadFormDetail(page.recordId);
            break;
          case 'form-create':
            await initFormEntityForm('create');
            break;
          case 'form-edit':
            await initFormEntityForm('edit', page.recordId);
            break;
          case 'response-list':
            await loadResponsesList();
            break;
          case 'response-create':
            await initResponseCreateForm();
            break;
          case 'response-detail':
            await loadResponseDetail(page.recordId);
            break;
          case 'response-edit':
            await initResponseEditPage(page.recordId);
            break;
          case 'report-list':
            await loadReportsList();
            break;
          case 'report-detail':
            await loadReportDetail(page.recordId);
            break;
          case 'report-create':
            await initReportForm('create');
            break;
          case 'report-edit':
            await initReportForm('edit', page.recordId);
            break;
          case 'dashboard-list':
            await loadDashboardsList();
            break;
          case 'dashboard-detail':
            await loadDashboardDetail(page.recordId);
            break;
          case 'dashboard-create':
            await initDashboardForm('create');
            break;
          case 'dashboard-edit':
            await initDashboardForm('edit', page.recordId);
            break;
        }
      }

      document.addEventListener('DOMContentLoaded', () => {
        initThemeControls();
        initPage().catch((error) => show(error.message));
      });
