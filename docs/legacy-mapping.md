# Legacy MMI-DMS to Tessara Mapping

This document is the first Slice 9 migration inventory. It maps product intent
from the legacy Django application into Tessara concepts without treating the
legacy schema as the target design.

## Source

- Legacy repository inspected at `D:\Projects\mmi-dms`.
- Primary source files:
  - `app/mmi/models.py`
  - `app/mmi/forms.py`
  - `app/mmi/views.py`
  - `app/mmi/urls.py`
- The legacy working tree had an unrelated modification in
  `deploy/dev/Dockerfile`; it was not read or changed for this mapping.

## Mapping Principle

Legacy polymorphic classes and screen routes are evidence of product behavior,
not a model to reproduce. Tessara should migrate durable concepts into
configurable hierarchy, versioned forms, submissions, analytics, reports, and
dashboards.

## Identity and Access

| Legacy concept | Tessara target | Notes |
| --- | --- | --- |
| `User` | `accounts` | Preserve email/name/login identity where available. |
| `MMIUserData` | role or capability set | Likely maps to global admin/manager capabilities. |
| `PartnerUserData` | role assignment scoped to Partner node | Preserve active flag as account status or scoped membership metadata. |
| `ClientUserData` | account plus Participant role scoped to Partner/Program nodes | Enrollments become node/account relationships or assignment eligibility. |
| `ParentUserData` | account plus Caregiver role scoped to child participant accounts | Parent-child relationship needs explicit target model before import. |
| `partner`, `children`, `enrollments` fields | scoped grants, memberships, or relationship tables | Do not encode as polymorphic user classes. |

Open decision: Tessara currently has RBAC primitives but not participant,
caregiver, or enrollment relationship tables. Importing users beyond admin
accounts should wait until those target relationships are modeled.

## Hierarchy

| Legacy concept | Tessara target | Notes |
| --- | --- | --- |
| `Partner` | NodeType `Partner`, runtime `nodes` | Preserve `name`, active/locked status as metadata or lifecycle flags. |
| `Program` | NodeType `Program`, child of Partner | Preserve enrollment relationship separately. |
| `Activity` | NodeType `Activity`, child of Program | Preserve active/locked metadata. |
| `Session` | NodeType `Session`, child of Activity | Preserve `date` as required metadata and active/locked metadata. |
| MMI global scope | Optional root node or global form/report scope | Prefer explicit root node if legacy global dashboards/forms need scoping. |

Initial Tessara hierarchy configuration for migration rehearsal:

```text
Partner -> Program -> Activity -> Session
```

Recommended metadata fields:

| Node type | Metadata |
| --- | --- |
| Partner | `legacy_id`, `is_active`, `locked` |
| Program | `legacy_id`, `is_active`, `locked` |
| Activity | `legacy_id`, `is_active`, `locked` |
| Session | `legacy_id`, `session_date`, `is_active`, `locked` |

## Forms

| Legacy concept | Tessara target | Notes |
| --- | --- | --- |
| `Form` base fields | `forms` plus `form_versions` | `is_active`, `is_public`, `expires_after`, `editable`, and `locked_after` need lifecycle/access handling. |
| `MMIForm` | global form or root-node form | `shared` should become explicit assignment/visibility configuration. |
| `PartnerForm` | form scoped to Partner node type or node | Legacy `partner` field maps to form scope. |
| `ProgramForm` | form scoped to Program node type or node | Copy behavior should not be imported as implicit duplication. |
| `ActivityForm` | form scoped to Activity node type or node | Same as above. |
| `SessionForm` | form scoped to Session node type or node | Same as above. |
| `FormSection` | `form_sections` | Preserve order, label, description; optional/display question needs a conditional-section rule model. |

Versioning decision: legacy forms are mutable objects. In Tessara, import each
legacy form as `form version 1` and treat any future edits as new versions.

## Field Types

| Legacy field | Tessara field type | Migration note |
| --- | --- | --- |
| `DateField` | `date` | Preserve default/min/max later as validation metadata. |
| `NumericField` | `number` | Preserve default/min/max later as validation metadata. |
| `TextField` | `text` | Preserve hint/display label later as presentation metadata. |
| `TextAreaField` | `text` | Needs multiline/presentation metadata, not a new storage type initially. |
| `SingleChoiceField` | `single_choice` | Requires choice list import. |
| `MultiChoiceField` | `multi_choice` | Requires multi-value support and choice list import. |
| `EnrolledClientSelectField` | relationship selector | Not supported by current target field enum. Model as a future account/node selector. |
| `MultipleEnrolledClientSelectField` | relationship selector multi-value | Same as above. |
| `ProgramSelectField` | node selector | Future node selector constrained to Program nodes. |
| `ActivitySelectField` | node selector | Future node selector constrained to Activity nodes. |
| `SessionSelectField` | node selector | Future node selector constrained to Session nodes. |

Open decision: add first-class `node_selector`, `account_selector`, and
multi-selector field types before importing selector-based legacy forms.

## Choice Lists

| Legacy concept | Tessara target | Notes |
| --- | --- | --- |
| `ChoiceList` | `choice_lists` | Preserve `legacy_id`, name, active flag, and partner scope. |
| `ChoiceListChoice` | `choice_list_items` | Preserve ordering, label, description, active flag. |
| Partner-scoped choice lists | scope metadata or ownership relationship | Needs target scope model before import. |
| MMI/global choice lists | global reusable choice lists | Read-only behavior for partners should be permission driven. |

## Assignments and Submissions

| Legacy concept | Tessara target | Notes |
| --- | --- | --- |
| `ClientFormAssignment` | `form_assignments` | `assigned_to` maps to account/participant relationship after participant modeling. |
| `uuid` assignment link | assignment public token or invitation table | Current Tessara assignment model does not expose public tokens yet. |
| `FormEntry` | `submissions` | Import as submitted records unless a legacy incomplete state is found. |
| `FormFieldEntry` subclasses | `submission_values` and `submission_value_multi` | Values must be normalized through target field definitions. |
| `editable` and `locked_after` | submission edit policy | Current Tessara treats submitted records as immutable; editable legacy records need a policy decision. |

Migration risk: legacy entry deletion manually deletes every polymorphic entry
type. Import tooling should validate orphaned or inconsistent field-entry rows
before creating Tessara submissions.

## Reporting

| Legacy concept | Tessara target | Notes |
| --- | --- | --- |
| `Report` | `reports` | One report is tied to one form in legacy; Tessara report bindings should use logical fields for version compatibility. |
| `report_table()` | analytics projection plus DataFusion table report | Legacy expands multi-value fields into row combinations in application code. |
| `Aggregation` | `aggregations` | Summary field plus grouping fields maps directly, but execution belongs in DataFusion. |
| `SummaryType` count/unique/total/mean/median/identity | aggregation kind | `identity` likely maps to detail-table/no aggregation. |
| Missing values as `None` or `NO DATA` | missing-data policies | Map consciously to `null`, `exclude_row`, or `bucket_unknown`. |

Migration risk: compatibility groups need field-level mapping where a report is
copied across forms or a form was edited in-place. Import should not bind reports
only by physical field IDs.

## Dashboards and Charts

| Legacy concept | Tessara target | Notes |
| --- | --- | --- |
| `MmiDashboard` | dashboard scoped globally/root | Use explicit scope once dashboard scoping is modeled. |
| `PartnerDashboard` | dashboard scoped to Partner node | Available reports/charts filtered by partner form scope. |
| `ProgramDashboard` | dashboard scoped to Program node | Includes Program, Activity, and Session forms under that program. |
| `DashboardComponent` | `dashboard_components` | Preserve row, column, width, height in config/layout metadata. |
| `DashboardDetailTable` | dashboard component using report table chart | Already close to Tessara table chart. |
| `DashboardSummaryTable` | dashboard component using aggregation/report chart | Needs aggregation execution support. |
| `DashboardChart` | dashboard component using chart | Maps directly. |

Chart mapping:

| Legacy chart | Tessara chart target |
| --- | --- |
| `Badge` | summary metric/card |
| `AggregateGauge` | gauge |
| `PercentComparisonGauge` | gauge with threshold/comparison config |
| `ComparisonBar` | grouped bar chart |
| `TrendLine` | line chart |
| `PieChart` | pie chart |
| `SummaryBar` | ranked bar chart |

Current Tessara only supports a table-style chart domain rule. Do not import
legacy chart definitions until chart config schemas exist for the target chart
types.

## Workflows to Preserve

- MMI admin can manage partners and global configuration.
- Partner users can manage partner programs, activities, sessions, users, forms,
  reports, charts, and dashboards within scope.
- Client users see assigned forms and complete them.
- Parent users see assigned forms for their children.
- Forms can be completed through assigned or public/unassigned flows.
- Reports can be viewed as detail tables and aggregated summaries.
- Dashboards exist at global, partner, and program scope.

## Do Not Port Directly

- Django polymorphic user/form/field/chart inheritance.
- Route parameters such as `form_type` and `object_id` as domain logic.
- Application-side report aggregation as the final reporting architecture.
- Program copy behavior as an implicit migration primitive.
- Manual cascading delete workarounds from legacy `FormEntry.delete`.

## First Migration Fixture

Use one small representative fixture before building generalized import:

1. One Partner.
2. One Program under that Partner.
3. One Activity under that Program.
4. One Session under that Activity with a date.
5. One Partner or Program form with:
   - one date field,
   - one numeric field,
   - one text field,
   - one single-choice field,
   - one multi-choice field.
6. One assigned Client account and one submitted form entry.
7. One table report and one aggregation that group by a choice field.
8. One dashboard component that displays the report or aggregation.

Acceptance checks for the fixture:

- Imported hierarchy appears in the Tessara node browser.
- Imported form renders as a published version.
- Imported submission appears after analytics refresh.
- Imported report returns the same representative values as legacy output.
- Imported dashboard can display the report-backed component.

## Open Decisions Before Slice 10

- How to model participant enrollment and parent-child relationships.
- Whether public forms become anonymous submission links, assignment tokens, or
  scoped public form availability.
- How to represent form editability and `locked_after` in a versioned submission
  model.
- Whether selector fields should be generalized as node/account selectors before
  import.
- Which legacy chart types are required for the first migration rehearsal.
- Whether global MMI scope should be represented as a root node or a separate
  global scope.
