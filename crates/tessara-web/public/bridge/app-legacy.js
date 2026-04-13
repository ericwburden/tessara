
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
      let organizationFormState = {
        nodeTypes: [],
        nodes: [],
        metadataFields: [],
        metadataValues: {},
        editNodeId: null,
        editNodeTypeId: null
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
      let renderedResponseForm = null;
      let currentResponseDetail = null;
      let currentDelegateContext = window.sessionStorage.getItem('tessara.delegateAccountId');

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
        const payload = text ? JSON.parse(text) : null;
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
            ? href === '/app'
            : !requiredCapability || hasCapability(requiredCapability);
          link.style.display = visible ? '' : 'none';
        }
      }

      function canAccessCurrentPage() {
        if (page.key === 'login') return true;
        if (!currentAccount) return false;
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
          'role-list': 'admin:all',
          'role-detail': 'admin:all',
          'role-create': 'admin:all',
          'role-edit': 'admin:all'
        };
        const requiredCapability = pageCapabilities[page.key];
        return !requiredCapability || hasCapability(requiredCapability);
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

      async function loadOrganizationsList() {
        try {
          await ensureAuthenticated();
          const payload = await request('/api/nodes');
          const html = payload.length
            ? payload.map((node) => recordCard(
                node.name,
              `<p>${escapeHtml(node.node_type_name)}</p><p class=\"muted\">${escapeHtml(node.parent_node_name || 'No parent')}</p>`,
                `<a class=\"button-link\" href=\"/app/organization/${node.id}\">View</a>${isAdmin() ? `<a class=\"button-link\" href=\"/app/organization/${node.id}/edit\">Edit</a>` : ''}`
              )).join('')
            : emptyState('No organization records found.');
          setHtml('organization-list', html);
          show(payload);
        } catch (error) {
          setHtml('organization-list', emptyState(error.message));
        }
      }

      async function loadOrganizationDetail(id) {
        try {
          await ensureAuthenticated();
          const payload = await request(`/api/nodes/${id}`);
          selectRecord('organization', payload.name, payload.id);
          const forms = payload.related_forms.length
            ? payload.related_forms.map((form) => `<li><a href=\"/app/forms/${form.form_id}\">${escapeHtml(form.form_name)}</a></li>`).join('')
            : '<li class=\"muted\">No related forms.</li>';
          const responses = payload.related_responses.length
            ? payload.related_responses.map((response) => `<li><a href=\"/app/responses/${response.submission_id}\">${escapeHtml(response.form_name)} ${escapeHtml(response.version_label)}</a></li>`).join('')
            : '<li class=\"muted\">No related responses.</li>';
          const dashboards = payload.related_dashboards.length
            ? payload.related_dashboards.map((dashboard) => `<li><a href=\"/app/dashboards/${dashboard.dashboard_id}\">${escapeHtml(dashboard.dashboard_name)}</a></li>`).join('')
            : '<li class=\"muted\">No related dashboards.</li>';
          const metadata = Object.entries(payload.metadata || {});
          setHtml('organization-detail', `
            ${detailSection('Summary', `<p>${escapeHtml(payload.name)}</p><p class=\"muted\">${escapeHtml(payload.node_type_name)}</p><p class=\"muted\">Parent: ${escapeHtml(payload.parent_node_name || 'None')}</p>`)}
            ${detailSection('Metadata', metadata.length ? `<dl class=\"detail-list\">${metadata.map(([key, value]) => `<div><dt>${escapeHtml(key)}</dt><dd>${escapeHtml(JSON.stringify(value))}</dd></div>`).join('')}</dl>` : '<p class=\"muted\">No metadata values.</p>')}
            ${detailSection('Related Forms', `<ul class=\"app-list\">${forms}</ul>`)}
            ${detailSection('Related Responses', `<ul class=\"app-list\">${responses}</ul>`)}
            ${detailSection('Related Dashboards', `<ul class=\"app-list\">${dashboards}</ul>`)}
          `);
          show(payload);
        } catch (error) {
          setHtml('organization-detail', emptyState(error.message));
        }
      }

      async function initOrganizationForm(mode, id) {
        try {
          await ensureAuthenticated();
          const [nodeTypes, nodes, metadataFields] = await Promise.all([
            request('/api/admin/node-types'),
            request('/api/nodes'),
            request('/api/admin/node-metadata-fields')
          ]);
          organizationFormState = {
            nodeTypes,
            nodes,
            metadataFields,
            metadataValues: {},
            editNodeId: id || null,
            editNodeTypeId: null
          };
          setSelectOptions(
            'organization-node-type',
            nodeTypes.map((item) => ({ value: item.id, label: item.name })),
            'Choose node type'
          );
          setSelectOptions(
            'organization-parent-node',
            nodes
              .filter((item) => item.id !== id)
              .map((item) => ({ value: item.id, label: item.name })),
            'No parent'
          );
          const nodeTypeSelect = byId('organization-node-type');
          if (nodeTypeSelect) {
            nodeTypeSelect.onchange = () => renderOrganizationMetadataFields(nodeTypeSelect.value);
          }
          if (mode === 'edit' && id) {
            const payload = await request(`/api/nodes/${id}`);
            organizationFormState.metadataValues = payload.metadata || {};
            organizationFormState.editNodeTypeId = payload.node_type_id;
            if (nodeTypeSelect) nodeTypeSelect.value = payload.node_type_id;
            byId('organization-parent-node').value = payload.parent_node_id || '';
            byId('organization-name').value = payload.name || '';
            renderOrganizationMetadataFields(payload.node_type_id);
          } else if (nodeTypeSelect && nodeTypeSelect.value) {
            renderOrganizationMetadataFields(nodeTypeSelect.value);
          }
          const form = byId('organization-form');
          if (form) {
            form.onsubmit = async (event) => {
              event.preventDefault();
              await submitOrganizationForm(mode, id);
            };
          }
        } catch (error) {
          setHtml('organization-metadata-fields', emptyState(error.message));
        }
      }

      function renderOrganizationMetadataFields(nodeTypeId) {
        const fields = organizationFormState.metadataFields.filter((field) => field.node_type_id === nodeTypeId);
        const html = fields.length
          ? fields.map((field) => {
              const value = organizationFormState.metadataValues[field.key];
              const inputId = `organization-metadata-${field.key}`;
              const hint = field.required ? 'required' : 'optional';
              if (field.field_type === 'boolean') {
                return `
                  <div class=\"form-field\">
                    <label for=\"${escapeHtml(inputId)}\">${escapeHtml(field.label)} (${escapeHtml(hint)})</label>
                    <input id=\"${escapeHtml(inputId)}\" type=\"checkbox\" ${value ? 'checked' : ''}>
                  </div>
                `;
              }
              const inputType = field.field_type === 'number'
                ? 'number'
                : field.field_type === 'date'
                  ? 'date'
                  : 'text';
              const renderedValue = Array.isArray(value) ? value.join(', ') : (value ?? '');
              const placeholder = field.field_type === 'multi_choice' ? 'Comma-separated values' : field.label;
              return `
                <div class=\"form-field\">
                  <label for=\"${escapeHtml(inputId)}\">${escapeHtml(field.label)} (${escapeHtml(hint)})</label>
                  <input id=\"${escapeHtml(inputId)}\" type=\"${inputType}\" value=\"${escapeHtml(renderedValue)}\" placeholder=\"${escapeHtml(placeholder)}\">
                </div>
              `;
            }).join('')
          : '<p class=\"muted\">No metadata fields are defined for this node type.</p>';
        setHtml('organization-metadata-fields', html);
      }

      function collectOrganizationMetadata(nodeTypeId) {
        const metadata = {};
        const fields = organizationFormState.metadataFields.filter((field) => field.node_type_id === nodeTypeId);
        for (const field of fields) {
          const element = byId(`organization-metadata-${field.key}`);
          if (!element) continue;
          if (field.field_type === 'boolean') {
            metadata[field.key] = element.checked;
            continue;
          }
          const raw = element.value.trim();
          if (raw === '') continue;
          if (field.field_type === 'number') {
            metadata[field.key] = Number(raw);
          } else if (field.field_type === 'multi_choice') {
            metadata[field.key] = raw.split(',').map((item) => item.trim()).filter(Boolean);
          } else {
            metadata[field.key] = raw;
          }
        }
        return metadata;
      }

      async function submitOrganizationForm(mode, id) {
        const nodeTypeId = byId('organization-node-type').value;
        const payload = {
          parent_node_id: byId('organization-parent-node').value || null,
          name: byId('organization-name').value.trim(),
          metadata: collectOrganizationMetadata(nodeTypeId || organizationFormState.editNodeTypeId)
        };
        if (mode === 'create') {
          payload.node_type_id = nodeTypeId;
        }
        const response = await request(
          mode === 'create' ? '/api/admin/nodes' : `/api/admin/nodes/${id}`,
          {
            method: mode === 'create' ? 'POST' : 'PUT',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify(payload)
          }
        );
        redirect(`/app/organization/${response.id}`);
      }

      async function loadFormsList() {
        try {
          await ensureAuthenticated();
          const payload = await request('/api/forms');
          const html = payload.length
            ? payload.map((form) => recordCard(
                form.name,
                `<p>${escapeHtml(form.slug)}</p><p class=\"muted\">${escapeHtml(form.scope_node_type_name || 'Unscoped')}</p>`,
                `<a class=\"button-link\" href=\"/app/forms/${form.id}\">View</a>${isAdmin() ? `<a class=\"button-link\" href=\"/app/forms/${form.id}/edit\">Edit</a>` : ''}`
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
          const payload = await request(`/api/forms/${id}`);
          selectRecord('form', payload.name, payload.id);
          const versions = payload.versions.length
            ? payload.versions.map((version) => `<li>${escapeHtml(version.version_label)} (${escapeHtml(version.status)})</li>`).join('')
            : '<li class=\"muted\">No versions.</li>';
          const reports = payload.reports.length
            ? payload.reports.map((report) => `<li><a href=\"/app/reports/${report.id}\">${escapeHtml(report.name)}</a></li>`).join('')
            : '<li class=\"muted\">No related reports.</li>';
          const datasets = payload.dataset_sources.length
            ? payload.dataset_sources.map((dataset) => `<li>${escapeHtml(dataset.dataset_name)} (${escapeHtml(dataset.source_alias)})</li>`).join('')
            : '<li class=\"muted\">No related dataset sources.</li>';
          const publishedCount = payload.versions.filter((version) => version.status === 'published').length;
          setHtml('form-detail', `
            ${detailSection('Summary', `<p>${escapeHtml(payload.name)}</p><p>${escapeHtml(payload.slug)}</p><p class=\"muted\">Scope: ${escapeHtml(payload.scope_node_type_name || 'Unscoped')}</p><p class=\"muted\">Published versions: ${publishedCount}</p>`)}
            ${detailSection('Versions', `<ul class=\"app-list\">${versions}</ul>`)}
            ${detailSection('Related Reports', `<ul class=\"app-list\">${reports}</ul>`)}
            ${detailSection('Related Dataset Sources', `<ul class=\"app-list\">${datasets}</ul>`)}
          `);
          show(payload);
        } catch (error) {
          setHtml('form-detail', emptyState(error.message));
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
          if (mode === 'edit' && id) {
            const payload = await request(`/api/forms/${id}`);
            byId('form-name').value = payload.name || '';
            byId('form-slug').value = payload.slug || '';
            byId('form-scope-node-type').value = payload.scope_node_type_id || '';
          }
          const form = byId('form-entity-form');
          if (form) {
            form.onsubmit = async (event) => {
              event.preventDefault();
              const response = await request(
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
              redirect(`/app/forms/${response.id}`);
            };
          }
        } catch (error) {
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

        try {
          await bootstrapCurrentAccount();
        } catch (error) {
          renderAccessState('Sign In Required', 'This screen requires an authenticated local account.');
          show(error.message);
          return;
        }

        if (!canAccessCurrentPage()) {
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
          case 'organization-list':
            await loadOrganizationsList();
            break;
          case 'organization-detail':
            await loadOrganizationDetail(page.recordId);
            break;
          case 'organization-create':
            await initOrganizationForm('create');
            break;
          case 'organization-edit':
            await initOrganizationForm('edit', page.recordId);
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
