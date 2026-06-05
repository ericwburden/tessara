import { expect, test, type APIResponse, type Page } from "@playwright/test";

type IdResponse = {
  id: string;
};

type NodeTypeSummary = {
  id: string;
  name: string;
  slug: string;
};

type FormWorkflowLink = {
  id: string;
  name: string;
  source: string;
  current_version_id: string | null;
  current_status: string | null;
};

type FormDefinition = {
  id: string;
  name: string;
  workflows: FormWorkflowLink[];
};

type WorkflowStepSummary = {
  form_version_id: string;
  title: string;
};

type WorkflowVersionSummary = {
  id: string;
  status: string;
  step_count: number;
  steps: WorkflowStepSummary[];
};

type WorkflowDefinition = {
  id: string;
  name: string;
  source: string;
  source_form_id: string | null;
  versions: WorkflowVersionSummary[];
};

type WorkflowAssignmentCandidate = {
  workflow_version_id: string;
  workflow_id: string;
  workflow_name: string;
  node_id: string;
  node_name: string;
};

type WorkflowAssigneeOption = {
  account_id: string;
  email: string;
  display_name: string;
};

type WorkflowAssignmentSummary = {
  id: string;
  workflow_id: string;
  workflow_name: string;
  workflow_version_id: string;
  form_id: string;
  form_name: string;
  node_id: string;
  node_name: string;
  account_id: string;
  account_email: string;
  has_draft: boolean;
  has_submitted: boolean;
};

type PendingWorkflowWork = {
  workflow_assignment_id: string;
  workflow_id: string;
  workflow_name: string;
  form_id: string;
  form_name: string;
  node_id: string;
  node_name: string;
  account_id: string;
  account_display_name: string;
};

type SubmissionDetail = {
  id: string;
  form_id: string;
  form_version_id: string;
  form_name: string;
  status: string;
  runtime: {
    workflow_name: string;
    current_step_title: string;
    current_step_position: number;
    step_count: number;
  } | null;
};

type AssignmentResponseStartOptions = {
  assignments: Array<{
    workflow_assignment_id: string;
    workflow_name: string;
    form_id: string;
    form_name: string;
    node_id: string;
    node_name: string;
    account_id: string;
    account_display_name: string;
  }>;
};

async function expectJson<T>(response: APIResponse): Promise<T> {
  const text = await response.text();
  expect(
    response.ok(),
    `${response.url()} returned ${response.status()}: ${text}`,
  ).toBeTruthy();
  return JSON.parse(text) as T;
}

async function apiGet<T>(page: Page, url: string): Promise<T> {
  return expectJson<T>(await page.request.get(url));
}

async function apiPost<T>(
  page: Page,
  url: string,
  data?: Record<string, unknown>,
): Promise<T> {
  return expectJson<T>(await page.request.post(url, { data }));
}

async function signIn(page: Page, email: string, password: string) {
  await expectJson(
    await page.request.post("/api/auth/login", {
      data: { email, password },
    }),
  );
}

async function signInAsAdmin(page: Page) {
  await signIn(page, "admin@tessara.local", "tessara-dev-admin");
}

async function signInAsDelegate(page: Page) {
  await signIn(page, "delegate@tessara.local", "tessara-dev-delegate");
}

// PERMISSIONS TODO:
// - Add signInAsOperator, signInAsRespondent, and signInAsDelegator helpers
//   when their Playwright scenarios are added.
// - Scoped operator scenarios should prove in-scope workflow assignments can be
//   seen/started and out-of-scope assignments cannot be seen or started by URL.
// - Respondent/delegate/delegator scenarios should prove response access remains
//   ownership/delegation-based rather than persona-based.

function uniqueSuffix() {
  return `${Date.now()}-${Math.random().toString(36).slice(2, 8)}`;
}

function generatedWorkflowFor(form: FormDefinition): FormWorkflowLink {
  const workflow = form.workflows.find(
    (item) =>
      item.source === "generated_form" &&
      item.current_status === "published" &&
      item.current_version_id,
  );
  expect(workflow, `${form.name} should expose a published generated workflow`).toBeTruthy();
  return workflow!;
}

async function activityNodeTypeId(page: Page): Promise<string> {
  const nodeTypes = await apiGet<NodeTypeSummary[]>(page, "/api/admin/node-types");
  const activity = nodeTypes.find((item) => item.slug === "activity");
  expect(activity, "demo seed should expose the Activity node type").toBeTruthy();
  return activity!.id;
}

async function createDraftForm(page: Page, name: string, slug: string) {
  const scopeNodeTypeId = await activityNodeTypeId(page);
  const form = await apiPost<IdResponse>(page, "/api/admin/forms", {
    name,
    slug,
    scope_node_type_id: scopeNodeTypeId,
  });
  const version = await apiPost<IdResponse>(
    page,
    `/api/admin/forms/${form.id}/versions`,
    {},
  );
  return { formId: form.id, formVersionId: version.id };
}

async function addMinimalField(
  page: Page,
  formVersionId: string,
  keySuffix: string,
) {
  const section = await apiPost<IdResponse>(
    page,
    `/api/admin/form-versions/${formVersionId}/sections`,
    {
      title: "Main",
      description: "",
      position: 0,
    },
  );
  await apiPost<IdResponse>(
    page,
    `/api/admin/form-versions/${formVersionId}/fields`,
    {
      section_id: section.id,
      key: `uat_field_${keySuffix}`.replace(/-/g, "_"),
      label: "UAT Field",
      field_type: "text",
      required: true,
      position: 0,
      grid_row: 1,
      grid_column: 1,
      grid_width: 12,
      grid_height: 2,
    },
  );
}

async function createPublishedForm(page: Page) {
  const suffix = uniqueSuffix();
  const name = `UAT Single Form ${suffix}`;
  const slug = `uat-single-form-${suffix}`;
  const { formId, formVersionId } = await createDraftForm(page, name, slug);
  await addMinimalField(page, formVersionId, suffix);
  await apiPost(page, `/api/admin/form-versions/${formVersionId}/publish`, {});
  const form = await apiGet<FormDefinition>(page, `/api/forms/${formId}`);
  const workflow = generatedWorkflowFor(form);
  return {
    formId,
    formVersionId,
    formName: name,
    workflowName: workflow.name,
    workflowId: workflow.id,
    workflowVersionId: workflow.current_version_id!,
  };
}

async function assignWorkflowToDelegate(
  page: Page,
  workflowId: string,
  workflowVersionId: string,
) {
  const candidates = await apiGet<WorkflowAssignmentCandidate[]>(
    page,
    "/api/workflow-assignment-candidates",
  );
  const candidate =
    candidates.find((item) => item.workflow_version_id === workflowVersionId) ??
    candidates.find((item) => item.workflow_id === workflowId);
  expect(candidate, "generated workflow should be assignable to at least one node").toBeTruthy();

  const assignees = await apiGet<WorkflowAssigneeOption[]>(
    page,
    `/api/workflow-assignment-candidates/assignees?workflow_version_id=${candidate!.workflow_version_id}&node_id=${candidate!.node_id}`,
  );
  const delegate = assignees.find((item) => item.email === "delegate@tessara.local");
  expect(delegate, "delegate account should be eligible for the assignment").toBeTruthy();

  const assignment = await apiPost<IdResponse>(page, "/api/workflow-assignments", {
    workflow_version_id: candidate!.workflow_version_id,
    node_id: candidate!.node_id,
    account_id: delegate!.account_id,
  });
  return {
    assignmentId: assignment.id,
    nodeId: candidate!.node_id,
    nodeName: candidate!.node_name,
    accountId: delegate!.account_id,
  };
}

// PERMISSIONS TODO:
// - Add a scoped assignment helper that picks one workflow candidate inside the
//   operator's effective scope and one outside it.
// - Assert assignment candidates and assignee options are filtered by workflow
//   available nodes plus assigner scope.

async function workflowDetail(page: Page, workflowId: string) {
  return apiGet<WorkflowDefinition>(page, `/api/workflows/${workflowId}`);
}

// FUTURE PERMISSION SUITES (comment-only scaffold; do not convert these to
// test.skip placeholders because skipped tests pollute reports):
//
// form-visibility-permissions.spec.ts
// - Admin can create, publish, and read forms.
// - Scoped operator can list/read only forms whose visibility nodes overlap
//   effective scope.
// - Scoped operator cannot read an out-of-scope form by direct URL/API.
// - Future UI: scoped form create/edit requires all visibility nodes inside
//   forms:manage scope.
//
// dataset-component-dashboard-permissions.spec.ts
// - Dataset list/detail/table rows respect dataset visibility scope.
// - Dataset table rows are filtered to user scope.
// - Component visibility follows the attached dataset revision.
// - Dashboard list/detail respects dashboard visibility.
// - Dashboard components render only when the component dataset scope is
//   compatible with dashboard visibility.
//
// admin-user-role-permissions.spec.ts
// - Admin-only routes cover users, roles, capabilities, and node types.
// - Future New User Screen: admin creates a user in-app and reaches access
//   assignment for scope/delegation setup.
// - Non-admin users cannot see Administration nav or load admin routes.

test.describe("workflow-mediated form shortcuts", () => {
  // PERMISSIONS TODO:
  // - Keep these admin flows proving that admin@tessara.local can manage
  //   generated workflows and workflow assignments globally through admin:all.
  // - Add matching non-admin denial checks once the admin/user/role permission
  //   suite has stable UI entry points.

  test("publishing a form creates a visible generated single-form workflow", async ({
    page,
  }) => {
    await signInAsAdmin(page);
    const setup = await createPublishedForm(page);

    const form = await apiGet<FormDefinition>(page, `/api/forms/${setup.formId}`);
    const generated = generatedWorkflowFor(form);
    expect(generated.id).toBe(setup.workflowId);

    const workflow = await workflowDetail(page, setup.workflowId);
    const current = workflow.versions.find((item) => item.id === setup.workflowVersionId);
    expect(workflow.source).toBe("generated_form");
    expect(workflow.source_form_id).toBe(setup.formId);
    expect(current?.status).toBe("published");
    expect(current?.step_count).toBe(1);
    expect(current?.steps[0]?.form_version_id).toBe(setup.formVersionId);

    await page.goto(`/forms/${setup.formId}`);
    await expect(page.getByRole("heading", { name: setup.formName })).toBeVisible();
    await expect(page.getByRole("link", { name: "Assign Form" })).toBeVisible();
    await expect(page.getByRole("link", { name: "Create Workflow" })).toBeVisible();

    await page.goto("/workflows");
    await page
      .getByRole("searchbox", { name: "Search workflows" })
      .fill(setup.formName);
    await expect(page.getByRole("link", { name: setup.workflowName })).toBeVisible();
    await expect(
      page.getByLabel("Single-Form, Generated Workflow").first(),
    ).toBeVisible();
  });

  test("Assign Form uses workflow assignments and delegates start assigned work", async ({
    page,
  }) => {
    await signInAsAdmin(page);
    const setup = await createPublishedForm(page);
    const assignment = await assignWorkflowToDelegate(
      page,
      setup.workflowId,
      setup.workflowVersionId,
    );

    const workflowAssignments = await apiGet<WorkflowAssignmentSummary[]>(
      page,
      `/api/workflow-assignments?workflow_id=${setup.workflowId}`,
    );
    expect(workflowAssignments.some((item) => item.id === assignment.assignmentId)).toBe(
      true,
    );

    await signInAsDelegate(page);
    const pending = await apiGet<PendingWorkflowWork[]>(
      page,
      "/api/workflow-assignments/pending",
    );
    expect(
      pending.some((item) => item.workflow_assignment_id === assignment.assignmentId),
      "delegate pending work should include the workflow assignment",
    ).toBe(true);

    await page.goto("/");
    await expect(page.getByRole("heading", { name: "Assigned to Me" })).toBeVisible();
    await expect(page.getByRole("link", { name: setup.formName })).toBeVisible();

    const submission = await apiPost<IdResponse>(
      page,
      `/api/workflow-assignments/${assignment.assignmentId}/start`,
      {},
    );
    const detail = await apiGet<SubmissionDetail>(
      page,
      `/api/submissions/${submission.id}`,
    );
    expect(detail.status).toBe("draft");
    expect(detail.form_id).toBe(setup.formId);
    expect(detail.form_version_id).toBe(setup.formVersionId);
    expect(detail.form_name).toBe(setup.formName);
    expect(detail.runtime?.workflow_name).toBe(setup.workflowName);
    expect(detail.runtime?.current_step_position).toBe(0);
    expect(detail.runtime?.step_count).toBe(1);
  });

  test("response start options are assignment-only", async ({
    page,
  }) => {
    await signInAsAdmin(page);
    const setup = await createPublishedForm(page);
    const assignment = await assignWorkflowToDelegate(
      page,
      setup.workflowId,
      setup.workflowVersionId,
    );

    await signInAsDelegate(page);
    const options = await apiGet<AssignmentResponseStartOptions>(
      page,
      "/api/responses/options",
    );
    const option = options.assignments.find(
      (item) => item.workflow_assignment_id === assignment.assignmentId,
    );
    expect(option, "response start options should expose the workflow assignment").toBeTruthy();
    expect(option?.form_id).toBe(setup.formId);
    expect(option?.workflow_name).toContain(setup.formName);

    const removedStart = await page.request.post("/api/responses/start", {
      data: {
        form_id: setup.formId,
        node_id: assignment.nodeId,
      },
    });
    expect([404, 405]).toContain(removedStart.status());
  });

  test("editing a generated workflow into multiple steps promotes it and form shortcut creates a fresh workflow", async ({
    page,
  }) => {
    await signInAsAdmin(page);
    const setup = await createPublishedForm(page);

    const multiStepRevision = await apiPost<IdResponse>(
      page,
      `/api/workflows/${setup.workflowId}/versions`,
      {
        steps: [
          {
            title: "Initial response",
            form_version_id: setup.formVersionId,
          },
          {
            title: "Follow-up response",
            form_version_id: setup.formVersionId,
          },
        ],
      },
    );
    await apiPost(page, `/api/workflow-versions/${multiStepRevision.id}/publish`, {});

    const promoted = await workflowDetail(page, setup.workflowId);
    expect(promoted.source).toBe("authored");
    expect(promoted.source_form_id).toBeNull();

    const newVersion = await apiPost<IdResponse>(
      page,
      `/api/admin/forms/${setup.formId}/versions`,
      {},
    );
    await addMinimalField(page, newVersion.id, uniqueSuffix());
    await apiPost(page, `/api/admin/form-versions/${newVersion.id}/publish`, {});

    const form = await apiGet<FormDefinition>(page, `/api/forms/${setup.formId}`);
    const regenerated = generatedWorkflowFor(form);
    expect(regenerated.id).not.toBe(setup.workflowId);

    const regeneratedDetail = await workflowDetail(page, regenerated.id);
    const published = regeneratedDetail.versions.find(
      (item) => item.id === regenerated.current_version_id,
    );
    expect(regeneratedDetail.source).toBe("generated_form");
    expect(regeneratedDetail.source_form_id).toBe(setup.formId);
    expect(published?.step_count).toBe(1);
    expect(published?.steps[0]?.form_version_id).toBe(newVersion.id);
  });
});
