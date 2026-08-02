# Sprint 8D2 Plan: Workflow Branching And Data Flow

Status: approved future planning artifact. This is not a kickoff record, does
not authorize implementation before its prerequisites, and does not claim a
branch, worktree, deployment, or validation result.

- Roadmap authority:
  [Sprint 8D2: Workflow Branching And Data Flow Slice](../roadmap.md#sprint-8d2-workflow-branching-and-data-flow-slice)
- Prerequisites: Sprint 8C Response Module Separation and Sprint 8D1 Workflow
  Module Separation are complete and their public contracts are accepted.
- Sequence: Sprint 8D2 completes before Sprint 8E Forms Module Separation.

## Outcome And Boundaries

The independently deployed Workflow module supports a versioned, exclusive
directed acyclic graph. Assigners and machine clients can supply typed inputs;
published Workflows can select the first and later form steps from those inputs
and submitted step outputs; and authors can pass approved data into later form
entries. Every execution retains enough immutable provenance to explain which
path ran and why without merging Workflow-owned context into user-entered
answers.

Sprint 8D2 includes one selected path per execution, converging paths, explicit
Start and End nodes, typed workflow variables, visual branch rules, and
explicit form-field mappings. It does not include loops, parallel paths or
joins, dynamic assignee or Organization-node routing, arbitrary JSON context,
scripts, automatic key matching, or context amendments after an instance has
started. Branching changes the next step only; the current assignee and node
remain fixed throughout the instance.

## Versioned Workflow Definition

Each WorkflowVersion owns an immutable graph definition containing:

- exactly one Start node, one or more Form Step nodes, and at least one End
  node;
- version-local stable node and transition identities;
- each Form Step's typed FormVersion reference;
- typed workflow-variable definitions for `text`, `number`, `boolean`, `date`,
  `single_choice`, and `multi_choice` values;
- assignment-input declarations, including label, help text, required state,
  and allowed choices where applicable;
- explicit form-field export bindings that populate workflow variables only
  from submitted Responses;
- directed transitions with visible priority, an optional condition tree, and
  target-step field mappings;
- destination mappings that name a workflow variable, a target FormVersion
  field, and one of `editable`, `locked`, or `hidden` presentation modes.

A variable may have only one producer on any executed path. Mutually exclusive
paths may populate the same variable when the type is identical, but a later
step may not overwrite an existing value. Authors must declare a new variable
when a later result represents a revision or transformation of earlier data.

Conditions are data, not code. The v1 condition tree supports typed equality,
inequality, numeric/date comparison, presence, and collection membership
predicates combined through nested `all` and `any` groups. Each node evaluates
outgoing conditional transitions in explicit priority order; the first match
wins. Every node with multiple possible destinations must have exactly one
final unconditional default transition. Missing optional values may be tested
with presence predicates; other predicates over a missing value evaluate
false.

Publication must reject:

- cycles, unreachable nodes, invalid Start/End cardinality, and any reachable
  path that cannot terminate at an End node;
- duplicate or ambiguous priorities, multiple defaults, or a conditional edge
  after the default;
- missing variables, incompatible predicate operands, invalid choice values,
  and variables with multiple producers on one possible path;
- missing, unavailable, or incompatible FormVersion and field references;
- mappings whose source and destination types are incompatible;
- required target fields that are not satisfiable on every path reaching the
  step; and
- scope or sibling-context combinations disallowed by the accepted Workflow,
  Forms, or Response contracts.

## Assignment, Execution, And Data Flow

Workflow assignment create and bulk-create contracts accept an `inputs` object
validated against the published WorkflowVersion. Bulk assignment applies one
shared validated input set to every selected assignee. An authorized assigner
may replace those inputs until the assignment starts. Starting freezes an
immutable input snapshot; later amendment is not supported.

Start is an executable decision node. Starting an assignment creates or
resumes one idempotent Workflow instance, evaluates Start transitions using
the frozen inputs, and either creates the selected first form entry or reaches
End. Start returns a typed result containing the Workflow instance identity,
current status, selected transition, and optional created Response reference
rather than assuming every start returns a form entry.

For a Form Step:

1. Workflows creates the Response through the versioned Workflow-to-Response
   contract, supplying the FormVersion reference, fixed assignee and node,
   idempotency/correlation identity, incoming transition, authorized
   step-visible context, and explicit destination-field mappings.
2. Response seeds mapped values. `editable` values may be changed by the
   respondent; `locked` values are visible but immutable; `hidden` values are
   stored but never returned in respondent APIs or documents.
3. Draft saves do not change Workflow context or evaluate transitions.
4. Response submission is immutable and emits a durable, idempotent event with
   the Response reference and Workflow correlation identity.
5. Workflows uses an audience-bound Response contract to read only the fields
   declared as exports, records their final submitted values as append-only
   workflow-variable events, evaluates the outgoing transitions once, and
   creates the next entry or completes at End.
6. Workflows attaches the selected outgoing transition and decision summary to
   the completed source entry. The next entry independently records the
   incoming transition and its mapped context.

Editable prefills export the respondent's final submitted value, not their
original prefill. Locked and hidden values retain their Workflow provenance.
No product artifact is created through Core or by direct access to another
module's database.

## Persistence And Public Contracts

The Workflow module owns graph definitions, assignment input snapshots,
instances, append-only variable values, transition decisions, inbox/outbox
state, retries, and operational findings. Each variable-value and transition
record includes its WorkflowVersion, source reference, actor or service,
timestamp, correlation identity, and canonical value or input digest.
Published executions remain pinned to their original WorkflowVersion and are
never silently reevaluated after a new version is published.

The Response module owns form entries, answers, mapped-value presentation and
origin, and an immutable `WorkflowEntryMetadataV1` projection containing:

- Workflow, WorkflowVersion, instance, assignment, and step references;
- incoming transition and the authorized step-visible context snapshot;
- provenance for each mapped value;
- the outgoing transition and branch-decision summary once processing
  completes; and
- correlation and idempotency identities needed for reconciliation.

Workflow metadata remains distinct from answers. Respondents can read
authorized visible context and editable/locked values. Hidden mappings are
absent from respondent payloads and SSR documents; only scope-authorized
Workflow or Response reviewers may read their values and provenance. Unknown,
unauthorized, stale-grant, wrong-audience, and wrong-scope requests retain the
platform's nondisclosure behavior.

Required versioned operations are:

- graph create/replace/validate/publish and read-back;
- assignment create/bulk-create with inputs and pre-start input replacement;
- idempotent Workflow start with a typed start result;
- Response entry creation with mappings and metadata;
- Response submitted event consumption and declared export readback;
- source-entry branch-decision attachment; and
- instance context/history readback for authorized execution and review UI.

## Reliability And Failure Semantics

Workflow start, Response creation, submitted-event consumption, export
capture, transition selection, metadata attachment, and next-entry creation
all require stable idempotency keys. Cross-module progress uses durable
inbox/outbox processing and retry rather than a distributed transaction.

If Workflows is unavailable after a Response submits, the Response remains
submitted and shows transition processing as pending. Recovery consumes the
event once and must not duplicate a branch decision, next assignment, or form
entry. An incompatible contract, unauthorized export, invalid runtime mapping,
or exhausted retry places the Workflow instance in a stable blocked state with
a non-disclosing, actionable finding. The runtime never silently selects a
different branch; a valid published graph's default transition handles the
ordinary no-match case.

Module diagnostics expose compatible contract versions, pending transitions,
retry counts, blocked instances, oldest pending age, and last successful event
processing without exposing protected workflow values.

## Application UI

Workflow authoring uses a visual DAG canvas plus a selected node or transition
inspector. The author can manage variables, assignment inputs, FormVersion
steps, output bindings, condition groups, transition priority, defaults, and
editable/locked/hidden mappings. An accessible structured outline provides
equivalent inspection and editing for keyboard, responsive, SSR, and
no-JavaScript use; spatial layout is never the sole representation of graph
meaning.

Assignment UI renders the published version's typed input form. Existing bulk
assignment applies the displayed input set to the whole batch and explains
that inputs freeze when work starts.

Execution and review surfaces show authorized current context, pending or
blocked transition state, completed path, and a human-readable explanation of
the selected branch. Response detail shows Workflow metadata separately from
answers. It never reveals hidden or out-of-scope values to respondents.

Generated single-form shortcuts are represented internally as
`Start -> Form Step -> End` but retain their current simple authoring and
assignment experience. Opening them for advanced graph editing promotes them
to authored Workflows under the existing promotion rule.

## Upgrade And Compatibility

Deploy the compatible Response contract before activating the Workflow graph
runtime. The Forms provider remains behind its typed compatibility contract
until Sprint 8E and must not become a direct database dependency.

Convert saved linear WorkflowVersions to `Start -> ordered Form Steps -> End`,
with unconditional default transitions. No legacy positional executor is
retained. Current in-progress instances are test-only and may be discarded by
an explicit non-production reset; the upgrade must refuse to proceed if it
unexpectedly finds non-test active instances. Preserve completed Response
records and their existing audit history where the retained environment
requires them.

## Verification And Acceptance

Automated coverage must include:

- graph and publisher rejection cases for every structural, typing, mapping,
  scope, and default-path rule;
- Start branching, all predicate types, nested `all`/`any`, priority, default,
  convergence, End completion, and linear/generated-workflow conversion;
- required and optional inputs, pre-start replacement, frozen snapshots, bulk
  shared inputs, invalid choices, and incompatible types;
- editable, locked, and hidden mappings, final-value exports, single-producer
  enforcement, source and target metadata, and reviewer/respondent projections;
- administrator, scoped manager, assignee, delegate, wrong-scope,
  stale-authorization, wrong-audience, and unknown-resource behavior;
- duplicate starts/events, partial processing, retries, Workflow/Response
  outage and recovery, incompatible contracts, and blocked-state recovery;
- direct SSR loads, hydration, no-JavaScript readback, keyboard operation,
  responsive layouts, accessibility, and clean browser consoles.

The manual acceptance scenario must let a tester assign a published Workflow
with validated input, have Start select the initial form, submit a value that
exports a variable, select one of at least two branches, carry approved data
into the chosen next form, and verify both source and target form-entry
metadata. Invalid or unauthorized input must not advance or disclose data;
retries must be exact no-ops after the first success; hidden values must remain
protected; and the execution must remain explainable after a newer
WorkflowVersion is published.
