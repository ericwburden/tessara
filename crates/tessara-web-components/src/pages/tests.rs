//! Tests for the parent module.

use super::{
    BarConfigDraft, ComponentConfigDraft, ComponentDefinition, ComponentVersionSummary,
    DatasetSummary, LineConfigDraft, PieDonutConfigDraft, StatCardConfigDraft, VisualSharedDraft,
    build_table_component_config, dataset_fields_for_major, dataset_picker_majors, toggle_csv_key,
    toggle_visible_column,
};
use super::{
    ComponentTableQueryInput, build_component_table_query, percent_encode_query_component,
};
use super::{
    component_kind_filter_options, component_matches_filters, component_status_filter_options,
};
use super::{
    component_redirect_ref, component_summary_kind_label, component_summary_revision_label,
    component_summary_status_label, dataset_catalog_option_label, dataset_picker_rows,
    dataset_provenance_label, editable_component_version, materialization_empty_state,
    selected_dataset_major_value, selected_dataset_picker_label, snake_case_component_slug,
    table_page_size_from_config, table_sort_from_config, table_visible_columns_from_config,
    visual_summary_field_ready,
};
use crate::types::{
    ComponentSummary, DatasetFieldDefinition, DatasetProvenanceItem, DatasetProvenanceSummary,
    DatasetRevisionFieldSummary,
};
use tessara_web_data_ops::{
    DatasetFieldDraft as DataOpsDatasetFieldDraft, DatasetRowFilterDraft as DataOpsRowFilterDraft,
};

fn dataset(major_versions: Vec<i32>, current_version_major: Option<i32>) -> DatasetSummary {
    DatasetSummary {
        id: "dataset-1".into(),
        current_version_major,
        major_versions,
        name: "Dataset".into(),
        slug: "dataset".into(),
        grain: "submission".into(),
        tags: Vec::new(),
        provenance: Default::default(),
        output_fields: Vec::new(),
        revisions: Vec::new(),
    }
}

fn dataset_field(key: &str) -> DatasetFieldDefinition {
    DatasetFieldDefinition {
        key: key.into(),
        label: key.into(),
        field_type: "text".into(),
    }
}

#[test]
fn dataset_picker_filters_every_displayed_column() {
    let mut item = dataset(vec![1, 2], Some(2));
    item.name = "Demo Session Log".into();
    item.tags = vec!["demo".into(), "session".into()];
    item.provenance.forms.push(DatasetProvenanceItem {
        id: "form-1".into(),
        name: "Session Intake".into(),
        slug: Some("session-intake".into()),
    });
    let datasets = vec![item];

    assert_eq!(dataset_picker_rows(&datasets, "session log").len(), 2);
    assert_eq!(dataset_picker_rows(&datasets, "v2").len(), 1);
    assert_eq!(dataset_picker_rows(&datasets, "demo").len(), 2);
    assert_eq!(dataset_picker_rows(&datasets, "intake").len(), 2);
    assert!(dataset_picker_rows(&datasets, "missing").is_empty());
}

#[test]
fn dataset_picker_label_uses_the_selected_dataset_and_version() {
    let mut item = dataset(vec![1, 2], Some(2));
    item.name = "Demo Session Log".into();
    assert_eq!(
        selected_dataset_picker_label(&[item], "dataset-1", "2"),
        "Demo Session Log · v2"
    );
    assert_eq!(
        selected_dataset_picker_label(&[], "dataset-1", "2"),
        "Select a Dataset version"
    );
}

fn visual_shared_draft() -> VisualSharedDraft {
    VisualSharedDraft {
        summary_field: "amount".into(),
        summary_type: "sum".into(),
        value_format: "integer".into(),
        value_missing_policy: "zero".into(),
        sort_field: "summary_value".into(),
        sort_direction: "desc".into(),
        filters: Vec::new(),
        limit: 12,
    }
}

#[test]
fn typed_component_drafts_serialize_only_kind_specific_contracts() {
    let bar = ComponentConfigDraft::Bar(BarConfigDraft {
        shared: visual_shared_draft(),
        category_field: "program".into(),
        category_missing_policy: "explicit_missing".into(),
        comparison_field: "region".into(),
        comparison_missing_policy: "omit".into(),
        comparison_layout: "stacked".into(),
        orientation: "vertical".into(),
        x_axis_label: "Program".into(),
        y_axis_label: "Amount".into(),
        category_labels: "East = Eastern".into(),
        category_colors: String::new(),
        legend_title: "Region".into(),
    })
    .into_json();
    assert_eq!(bar["mode"], "comparison");
    assert_eq!(bar["category_missing_policy"], "explicit_missing");
    assert_eq!(bar["comparison_missing_policy"], "omit");
    assert_eq!(bar["number_of_points"], 12);
    assert!(bar.get("x_field").is_none());

    let line = ComponentConfigDraft::Line(LineConfigDraft {
        shared: visual_shared_draft(),
        x_field: "period".into(),
        x_missing_policy: "omit".into(),
        smoothing: false,
    })
    .into_json();
    assert_eq!(line["x_field"], "period");
    assert_eq!(line["smoothing"], false);
    assert!(line.get("category_field").is_none());

    let pie = ComponentConfigDraft::Pie(PieDonutConfigDraft {
        shared: visual_shared_draft(),
        category_field: "program".into(),
        category_missing_policy: "omit".into(),
        category_labels: String::new(),
        category_colors: String::new(),
        legend_title: String::new(),
    })
    .into_json();
    assert_eq!(pie["max_slices"], 12);
    assert!(pie.get("orientation").is_none());

    let stat = ComponentConfigDraft::StatCard(StatCardConfigDraft {
        shared: visual_shared_draft(),
        label: "Total".into(),
        supporting_text: "Current period".into(),
        panel_style: "accent".into(),
    })
    .into_json();
    assert_eq!(stat["panel_style"], "accent");
    assert!(stat.get("sort_field").is_none());
    assert!(stat.get("number_of_points").is_none());
}

fn component_summary(
    name: &str,
    component_type: Option<&str>,
    published: bool,
) -> ComponentSummary {
    ComponentSummary {
        id: format!("{name}-id"),
        name: name.into(),
        slug: name.to_lowercase().replace(' ', "-"),
        description: None,
        current_version_id: published.then(|| format!("{name}-version")),
        current_version_label: published.then(|| "1".into()),
        current_component_type: component_type.map(str::to_string),
        draft_version_id: (!published).then(|| format!("{name}-draft")),
        draft_version_label: (!published).then(|| "1".into()),
    }
}

#[test]
fn dataset_picker_prefers_major_versions_from_list_response() {
    assert_eq!(
        dataset_picker_majors(&dataset(vec![1, 2], Some(3))),
        vec![1, 2]
    );
}

#[test]
fn dataset_picker_falls_back_to_current_major() {
    assert_eq!(
        dataset_picker_majors(&dataset(Vec::new(), Some(4))),
        vec![4]
    );
}

#[test]
fn dataset_fields_are_scoped_to_the_selected_major_line() {
    let mut item = dataset(vec![1, 2], Some(2));
    item.output_fields = vec![dataset_field("current")];
    item.revisions = vec![
        DatasetRevisionFieldSummary {
            version_number: 1,
            version_major: Some(1),
            output_fields: vec![dataset_field("legacy")],
        },
        DatasetRevisionFieldSummary {
            version_number: 2,
            version_major: Some(2),
            output_fields: vec![dataset_field("current")],
        },
    ];

    assert_eq!(dataset_fields_for_major(&item, 1)[0].key, "legacy");
    assert_eq!(dataset_fields_for_major(&item, 2)[0].key, "current");
    assert!(dataset_fields_for_major(&item, 3).is_empty());
}

#[test]
fn row_count_preview_does_not_require_a_value_field() {
    assert!(visual_summary_field_ready("row_count", ""));
    assert!(!visual_summary_field_ready("sum", ""));
    assert!(visual_summary_field_ready("sum", "amount"));
}

#[test]
fn dataset_catalog_option_includes_tags_and_provenance() {
    let mut dataset = dataset(vec![1], Some(1));
    dataset.tags = vec!["finance".into(), "display".into()];
    dataset.provenance = DatasetProvenanceSummary {
        forms: vec![DatasetProvenanceItem {
            id: "form-1".into(),
            name: "Intake Form".into(),
            slug: Some("intake".into()),
        }],
        datasets: vec![DatasetProvenanceItem {
            id: "dataset-2".into(),
            name: "Analytical Source".into(),
            slug: Some("analytical-source".into()),
        }],
    };

    assert_eq!(
        dataset_catalog_option_label(&dataset, 1),
        "Dataset · v1 · finance, display · Intake Form, Dataset: Analytical Source"
    );
    assert_eq!(
        dataset_provenance_label(&dataset.provenance),
        "Intake Form, Dataset: Analytical Source"
    );
}

#[test]
fn component_list_filters_match_name_kind_and_status() {
    let published_table = component_summary("Program Snapshot", Some("table"), true);
    let draft_component = component_summary("Program Draft", None, false);
    let mut updating_component = component_summary("Program Update", Some("table"), true);
    updating_component.draft_version_id = Some("Program Update-draft".into());
    updating_component.draft_version_label = Some("2".into());
    let components = vec![
        published_table.clone(),
        draft_component.clone(),
        updating_component.clone(),
    ];

    assert_eq!(component_kind_filter_options(&components), vec!["Table"]);
    assert_eq!(
        component_status_filter_options(&components),
        vec!["Draft", "Published", "Updating"]
    );
    assert!(component_matches_filters(
        &published_table,
        "snapshot",
        "Table",
        "Published"
    ));
    assert!(!component_matches_filters(
        &published_table,
        "snapshot",
        "Draft",
        "Published"
    ));
    assert!(component_matches_filters(
        &draft_component,
        "program",
        "Table",
        "Draft"
    ));
    assert!(component_matches_filters(
        &updating_component,
        "program",
        "Table",
        "Updating"
    ));
}

#[test]
fn reader_component_summary_without_draft_metadata_stays_published() {
    let reader_summary = component_summary("Reader Visible Table", Some("table"), true);

    assert_eq!(component_summary_kind_label(&reader_summary), "Table");
    assert_eq!(component_summary_status_label(&reader_summary), "Published");
    assert_eq!(component_summary_revision_label(&reader_summary), "v1");
}

#[test]
fn table_config_uses_visible_columns_and_defaults() {
    let config = build_table_component_config(
        &projection_fields(&["program", "amount"]),
        &[filter_draft(1, "program", "equals", "Afterschool")],
        "program",
        "desc",
        "25",
    );

    assert_eq!(
        config["visible_columns"],
        serde_json::json!(["program", "amount"])
    );
    assert_eq!(config["display_labels"]["program"], "program");
    assert_eq!(config["default_sort"]["field_key"], "program");
    assert_eq!(config["default_sort"]["direction"], "desc");
    assert_eq!(config["page_size"], 25);
    assert_eq!(config["filters"][0]["field_key"], "program");
    assert_eq!(config["filters"][0]["operator"], "equals");
    assert_eq!(config["filters"][0]["value"], "Afterschool");
}

#[test]
fn toggle_csv_key_adds_and_removes_keys() {
    let mut value = "program".to_string();
    toggle_csv_key(&mut value, "amount");
    assert_eq!(value, "program, amount");
    toggle_csv_key(&mut value, "program");
    assert_eq!(value, "amount");
}

#[test]
fn toggle_visible_column_treats_blank_as_all_selected() {
    let all_keys = vec!["program".into(), "amount".into(), "status".into()];
    let mut value = String::new();

    toggle_visible_column(&mut value, "amount", &all_keys);
    assert_eq!(value, "program, status");

    toggle_visible_column(&mut value, "amount", &all_keys);
    assert_eq!(value, "");
}

#[test]
fn table_visible_columns_parse_string_and_object_configs() {
    let config = serde_json::json!({
        "visible_columns": [
            "program",
            { "field_key": "amount" },
            { "key": "status" }
        ]
    });

    assert_eq!(
        table_visible_columns_from_config(&config),
        "program, amount, status"
    );
}

#[test]
fn table_config_extracts_sort_and_page_size() {
    let config = serde_json::json!({
        "default_sort": {
            "field_key": "program",
            "direction": "desc"
        },
        "page_size": 25
    });

    assert_eq!(
        table_sort_from_config(&config),
        ("program".into(), "desc".into())
    );
    assert_eq!(table_page_size_from_config(&config), "25");
}

#[test]
fn component_table_query_encodes_server_driven_view_state() {
    let query = build_component_table_query(ComponentTableQueryInput {
        search: "family outreach",
        page_size: "25",
        cursor: "offset:25",
        sort_field: "program",
        sort_direction: "desc",
        filter_field: "row_count",
        filter_operator: "between",
        filter_value: "1,10",
        visible_columns: "program, row_count",
    });

    assert_eq!(
        query,
        "q=family%20outreach&page_size=25&cursor=offset%3A25&sort=program%3Adesc&filter%5Brow_count%5D%5Boperator%5D=between&filter%5Brow_count%5D%5Bvalue%5D=1%2C10&visible_columns=program%2C%20row_count"
    );
}

#[test]
fn component_table_query_omits_blank_optional_params() {
    assert_eq!(
        build_component_table_query(ComponentTableQueryInput {
            search: "",
            page_size: "50",
            cursor: "",
            sort_field: "",
            sort_direction: "asc",
            filter_field: "",
            filter_operator: "equals",
            filter_value: "",
            visible_columns: "",
        }),
        "page_size=50"
    );
    assert_eq!(percent_encode_query_component("a/b?c"), "a%2Fb%3Fc");
}

#[test]
fn materialization_empty_state_distinguishes_failed_from_pending() {
    let (pending_title, pending_message) = materialization_empty_state("pending");
    assert_eq!(pending_title, "Table materializing");
    assert!(pending_message.contains("still being prepared"));

    let (failed_title, failed_message) = materialization_empty_state("failed");
    assert_eq!(failed_title, "Table materialization failed");
    assert!(failed_message.contains("configuration is valid"));

    let (retry_title, retry_message) = materialization_empty_state("retry");
    assert_eq!(retry_title, "Table materializing");
    assert!(retry_message.contains("retry"));
}

#[test]
fn selected_dataset_major_value_is_empty_until_complete() {
    assert_eq!(selected_dataset_major_value("", "1"), "");
    assert_eq!(selected_dataset_major_value("dataset-1", ""), "");
    assert_eq!(
        selected_dataset_major_value("dataset-1", "2"),
        "dataset-1|2"
    );
}

#[test]
fn component_redirect_ref_uses_trimmed_slug() {
    assert_eq!(
        component_redirect_ref("  family-outreach-table  "),
        "family-outreach-table"
    );
}

#[test]
fn snake_case_component_slug_normalizes_component_names() {
    assert_eq!(
        snake_case_component_slug("UAT Table Component"),
        "uat_table_component"
    );
    assert_eq!(
        snake_case_component_slug(" Demo Partner: Snapshot 2026 "),
        "demo_partner_snapshot_2026"
    );
    assert_eq!(snake_case_component_slug("Already_snake"), "already_snake");
}

#[test]
fn editable_component_version_prefers_draft() {
    let component = ComponentDefinition {
        id: "component-1".into(),
        name: "Component".into(),
        slug: "component".into(),
        description: None,
        versions: vec![
            component_version("published", "published-version"),
            component_version("draft", "draft-version"),
        ],
    };

    assert_eq!(
        editable_component_version(&component)
            .expect("editable version")
            .id,
        "draft-version"
    );
}

fn component_version(status: &str, id: &str) -> ComponentVersionSummary {
    ComponentVersionSummary {
        id: id.into(),
        component_id: "component-1".into(),
        dataset_id: "dataset-1".into(),
        dataset_version_major: 1,
        binding_mode: "major_line".into(),
        component_type: "table".into(),
        status: status.into(),
        version_label: "1".into(),
        version_note: String::new(),
        config: serde_json::json!({ "visible_columns": ["program"] }),
    }
}

fn projection_fields(keys: &[&str]) -> Vec<DataOpsDatasetFieldDraft> {
    keys.iter()
        .map(|key| DataOpsDatasetFieldDraft {
            key: (*key).into(),
            label: (*key).into(),
            source_alias: "dataset".into(),
            source_field_key: (*key).into(),
            field_type: "text".into(),
        })
        .collect()
}

fn filter_draft(id: u64, field_key: &str, operator: &str, value: &str) -> DataOpsRowFilterDraft {
    DataOpsRowFilterDraft {
        id,
        field_key: field_key.into(),
        operator: operator.into(),
        value: value.into(),
        value_mode: "value".into(),
        value_field_key: String::new(),
    }
}
