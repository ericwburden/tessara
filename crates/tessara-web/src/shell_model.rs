//! Static shell metadata rendered by the Leptos shell components.

/// Button action exposed by the browser-side workflow controller.
pub(crate) struct Action {
    pub(crate) handler: &'static str,
    pub(crate) label: &'static str,
}

impl Action {
    pub(crate) const fn new(handler: &'static str, label: &'static str) -> Self {
        Self { handler, label }
    }
}

/// Text input rendered in a workflow section.
pub(crate) struct TextInput {
    pub(crate) id: &'static str,
    pub(crate) label: &'static str,
    pub(crate) placeholder: &'static str,
    pub(crate) value: &'static str,
}

impl TextInput {
    pub(crate) const fn new(
        id: &'static str,
        label: &'static str,
        placeholder: &'static str,
        value: &'static str,
    ) -> Self {
        Self {
            id,
            label,
            placeholder,
            value,
        }
    }
}

/// Text area rendered in a workflow section.
pub(crate) struct TextArea {
    pub(crate) id: &'static str,
    pub(crate) label: &'static str,
    pub(crate) placeholder: &'static str,
}

impl TextArea {
    pub(crate) const fn new(
        id: &'static str,
        label: &'static str,
        placeholder: &'static str,
    ) -> Self {
        Self {
            id,
            label,
            placeholder,
        }
    }
}

/// Roadmap-aligned set of controls for the local test shell.
pub(crate) struct WorkflowSection {
    pub(crate) title: &'static str,
    pub(crate) description: &'static str,
    pub(crate) inputs: &'static [TextInput],
    pub(crate) text_area: Option<&'static TextArea>,
    pub(crate) actions: &'static [Action],
}

impl WorkflowSection {
    pub(crate) const fn new(
        title: &'static str,
        description: &'static str,
        inputs: &'static [TextInput],
        text_area: Option<&'static TextArea>,
        actions: &'static [Action],
    ) -> Self {
        Self {
            title,
            description,
            inputs,
            text_area,
            actions,
        }
    }
}

const PRIMARY_ACTIONS: &[Action] = &[
    Action::new("login()", "Log In"),
    Action::new("seedDemo()", "Seed Demo"),
    Action::new("loadNodeTypes()", "Hierarchy Screen"),
    Action::new("loadForms()", "Forms Screen"),
    Action::new("loadNodes()", "Load Nodes"),
    Action::new("loadSubmissions()", "Load Submissions"),
    Action::new("loadDashboards()", "Load Dashboards"),
    Action::new("loadReports()", "Load Reports"),
    Action::new("loadDashboard()", "Load Demo Dashboard"),
];

const HIERARCHY_INPUTS: &[TextInput] = &[
    TextInput::new("node-type-name", "Node type name", "Node type name", ""),
    TextInput::new("node-type-slug", "Node type slug", "Node type slug", ""),
    TextInput::new(
        "parent-node-type-id",
        "Parent node type ID",
        "Parent node type ID",
        "",
    ),
    TextInput::new(
        "child-node-type-id",
        "Child node type ID",
        "Child node type ID",
        "",
    ),
    TextInput::new(
        "metadata-node-type-id",
        "Metadata node type ID",
        "Metadata node type ID",
        "",
    ),
    TextInput::new("metadata-key", "Metadata key", "Metadata key", "region"),
    TextInput::new(
        "metadata-label",
        "Metadata label",
        "Metadata label",
        "Region",
    ),
    TextInput::new(
        "metadata-field-type",
        "Metadata field type",
        "Metadata field type",
        "text",
    ),
    TextInput::new(
        "node-type-id",
        "Node type ID",
        "Node type ID for node creation",
        "",
    ),
    TextInput::new(
        "parent-node-id",
        "Parent node ID",
        "Optional parent node ID",
        "",
    ),
    TextInput::new("node-name", "Node name", "Node name", "Local Organization"),
    TextInput::new(
        "node-metadata-json",
        "Node metadata JSON",
        "Node metadata JSON, e.g. {\"region\":\"North\"}",
        "{\"region\":\"North\"}",
    ),
    TextInput::new("node-search", "Node search", "Search nodes", ""),
];

const HIERARCHY_ACTIONS: &[Action] = &[
    Action::new("createNodeType()", "Create Node Type"),
    Action::new("loadRelationships()", "Load Relationships"),
    Action::new("createRelationship()", "Create Relationship"),
    Action::new("loadMetadataFields()", "Load Metadata Fields"),
    Action::new("createMetadataField()", "Create Metadata Field"),
    Action::new("createNode()", "Create Node"),
];

const FORM_INPUTS: &[TextInput] = &[
    TextInput::new("form-name", "Form name", "Form name", ""),
    TextInput::new("form-slug", "Form slug", "Form slug", ""),
    TextInput::new(
        "form-scope-node-type-id",
        "Scope node type ID",
        "Optional form scope node type ID",
        "",
    ),
    TextInput::new("form-id", "Form ID", "Form ID", ""),
    TextInput::new(
        "form-version-label",
        "Version label",
        "Form version label",
        "v1",
    ),
    TextInput::new(
        "compatibility-group-name",
        "Compatibility group",
        "Compatibility group name",
        "Default compatibility",
    ),
    TextInput::new(
        "form-version-id",
        "Published form version ID",
        "Published form version ID",
        "",
    ),
    TextInput::new("section-id", "Section ID", "Section ID", ""),
    TextInput::new("section-title", "Section title", "Section title", "Main"),
    TextInput::new("field-key", "Field key", "Field key", "participants"),
    TextInput::new("field-label", "Field label", "Field label", "Participants"),
    TextInput::new("field-type", "Field type", "Field type", "number"),
];

const FORM_ACTIONS: &[Action] = &[
    Action::new("createForm()", "Create Form"),
    Action::new("createFormVersion()", "Create Version"),
    Action::new("createSection()", "Create Section"),
    Action::new("createField()", "Create Field"),
    Action::new("publishVersion()", "Publish Version"),
];

const SUBMISSION_INPUTS: &[TextInput] = &[
    TextInput::new("node-id", "Target node ID", "Target node ID", ""),
    TextInput::new(
        "submission-id",
        "Draft submission ID",
        "Draft submission ID",
        "",
    ),
    TextInput::new(
        "participants-value",
        "Participants value",
        "Participants value",
        "42",
    ),
];

const SUBMISSION_ACTIONS: &[Action] = &[
    Action::new("createDraft()", "Create Draft"),
    Action::new("saveRenderedFormValues()", "Save Rendered Values"),
    Action::new("saveParticipants()", "Save Participants"),
    Action::new("submitDraft()", "Submit Draft"),
    Action::new("loadSubmissionById()", "Load Submission By ID"),
    Action::new("refreshAnalytics()", "Refresh Analytics"),
];

const REPORTING_INPUTS: &[TextInput] = &[
    TextInput::new(
        "report-name",
        "Report name",
        "Report name",
        "Participants Report",
    ),
    TextInput::new(
        "report-logical-key",
        "Report logical key",
        "Report logical key",
        "participants",
    ),
    TextInput::new(
        "report-source-field-key",
        "Report source field key",
        "Report source field key",
        "participants",
    ),
    TextInput::new(
        "report-fields-json",
        "Report bindings JSON",
        "Optional report bindings JSON",
        "",
    ),
    TextInput::new(
        "report-id",
        "Report ID",
        "Report ID from seed or import output",
        "",
    ),
    TextInput::new("chart-id", "Chart ID", "Chart ID", ""),
    TextInput::new(
        "chart-name",
        "Chart name",
        "Chart name",
        "Participants Table",
    ),
    TextInput::new("chart-type", "Chart type", "Chart type", "table"),
    TextInput::new(
        "dashboard-name",
        "Dashboard name",
        "Dashboard name",
        "Local Dashboard",
    ),
    TextInput::new(
        "dashboard-id",
        "Dashboard ID",
        "Dashboard ID from seed or import output",
        "",
    ),
];

const REPORTING_ACTIONS: &[Action] = &[
    Action::new("createReport()", "Create Report"),
    Action::new("loadReportById()", "Load Report By ID"),
    Action::new("loadReportDefinitionById()", "Inspect Report By ID"),
    Action::new("createChart()", "Create Chart"),
    Action::new("loadCharts()", "Load Charts"),
    Action::new("createDashboard()", "Create Dashboard"),
    Action::new("addDashboardComponent()", "Add Component"),
    Action::new("loadDashboardById()", "Load Dashboard By ID"),
];

const LEGACY_FIXTURE_INPUT: TextArea = TextArea::new(
    "legacy-fixture-json",
    "Legacy fixture JSON",
    "Paste legacy fixture JSON for validation",
);

const MIGRATION_ACTIONS: &[Action] = &[Action::new(
    "validateLegacyFixture()",
    "Validate Legacy Fixture",
)];

/// Top-level shell actions for login, seeding, and navigation.
pub(crate) const PRIMARY_SECTION: WorkflowSection = WorkflowSection::new(
    "Session and Navigation",
    "Use the development admin session, seed a deterministic demo, and jump to the main read screens.",
    &[],
    None,
    PRIMARY_ACTIONS,
);

/// Roadmap-aligned workflow sections for the local shell.
pub(crate) const WORKFLOW_SECTIONS: &[WorkflowSection] = &[
    WorkflowSection::new(
        "Hierarchy Builder",
        "Configure node types, parent-child relationships, metadata fields, and nodes.",
        HIERARCHY_INPUTS,
        None,
        HIERARCHY_ACTIONS,
    ),
    WorkflowSection::new(
        "Form Builder",
        "Create a form, draft a compatible version, add a section and field, then publish it.",
        FORM_INPUTS,
        None,
        FORM_ACTIONS,
    ),
    WorkflowSection::new(
        "Submission Workflow",
        "Create a draft for the selected node and form version, save values, submit, and refresh analytics.",
        SUBMISSION_INPUTS,
        None,
        SUBMISSION_ACTIONS,
    ),
    WorkflowSection::new(
        "Reports and Dashboards",
        "Create report bindings, charts, dashboards, and dashboard components, then inspect outputs.",
        REPORTING_INPUTS,
        None,
        REPORTING_ACTIONS,
    ),
    WorkflowSection::new(
        "Migration Workbench",
        "Validate representative legacy fixture JSON before running a rehearsal import.",
        &[],
        Some(&LEGACY_FIXTURE_INPUT),
        MIGRATION_ACTIONS,
    ),
];

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use super::{PRIMARY_SECTION, WORKFLOW_SECTIONS};

    #[test]
    fn workflow_sections_have_unique_dom_ids() {
        let mut ids = HashSet::new();

        for section in WORKFLOW_SECTIONS {
            for input in section.inputs {
                assert!(ids.insert(input.id), "duplicate input id {}", input.id);
            }

            if let Some(text_area) = section.text_area {
                assert!(
                    ids.insert(text_area.id),
                    "duplicate text area id {}",
                    text_area.id
                );
            }
        }
    }

    #[test]
    fn workflow_sections_cover_testable_surfaces() {
        assert!(!PRIMARY_SECTION.actions.is_empty());

        let titles = WORKFLOW_SECTIONS
            .iter()
            .map(|section| section.title)
            .collect::<Vec<_>>();

        assert!(titles.contains(&"Hierarchy Builder"));
        assert!(titles.contains(&"Form Builder"));
        assert!(titles.contains(&"Submission Workflow"));
        assert!(titles.contains(&"Reports and Dashboards"));
        assert!(titles.contains(&"Migration Workbench"));
    }

    #[test]
    fn workflow_actions_remain_bound_to_javascript_handlers() {
        for section in std::iter::once(&PRIMARY_SECTION).chain(WORKFLOW_SECTIONS.iter()) {
            assert!(
                !section.actions.is_empty(),
                "{} should expose at least one action",
                section.title
            );

            for action in section.actions {
                assert!(
                    action.handler.ends_with("()"),
                    "{} action should call a JavaScript function",
                    action.label
                );
            }
        }
    }
}
