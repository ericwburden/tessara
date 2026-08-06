//! Versioned, actor-filtered shell navigation received from the application API.
//!
//! The browser treats this projection as display state only. Route guards remain
//! independent authorization boundaries, and an unsupported projection fails
//! closed to the small Core fallback owned by the shell.

use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};

pub const SHELL_NAVIGATION_SCHEMA_VERSION_V1: u16 = 3;

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ShellNavigationStateV1 {
    Available,
    Unavailable,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ShellNavigationResponseV1 {
    pub schema_version: u16,
    pub policy_revision: Option<i64>,
    pub state: ShellNavigationStateV1,
    pub groups: Vec<ShellNavigationGroupV1>,
    pub unavailable: Option<ShellNavigationUnavailableV1>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ShellNavigationGroupV1 {
    pub id: String,
    pub name: String,
    pub items: Vec<ShellNavigationItemV1>,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ShellNavigationItemOwnerV1 {
    Core,
    Contribution,
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ShellNavigationModeV1 {
    Shell,
    #[default]
    Document,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ShellNavigationItemV1 {
    pub key: String,
    pub label: String,
    pub href: String,
    pub owner: ShellNavigationItemOwnerV1,
    pub contribution_id: Option<String>,
    #[serde(default)]
    pub navigation_mode: ShellNavigationModeV1,
}

impl ShellNavigationItemV1 {
    pub(crate) fn requires_document_navigation(&self) -> bool {
        self.navigation_mode == ShellNavigationModeV1::Document
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ShellNavigationUnavailableV1 {
    pub code: String,
    pub message: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ShellNavigationLoadState {
    Loading,
    Ready(ShellNavigationResponseV1),
    Failed,
}

impl ShellNavigationResponseV1 {
    /// Rejects schema drift, malformed ownership, unknown destinations, Core
    /// displacement, and contribution leakage from a fail-closed response.
    pub fn is_supported(&self) -> bool {
        if self.schema_version != SHELL_NAVIGATION_SCHEMA_VERSION_V1
            || self.policy_revision.is_some_and(|revision| revision < 0)
        {
            return false;
        }

        match self.state {
            ShellNavigationStateV1::Available
                if self.policy_revision.is_none() || self.unavailable.is_some() =>
            {
                return false;
            }
            ShellNavigationStateV1::Unavailable => {
                let Some(unavailable) = &self.unavailable else {
                    return false;
                };
                if unavailable.code != "shell_navigation_unavailable"
                    || unavailable.message.trim().is_empty()
                {
                    return false;
                }
            }
            ShellNavigationStateV1::Available => {}
        }

        if self.groups.is_empty() {
            return false;
        }

        let mut seen_keys = BTreeSet::new();
        let mut seen_hrefs = BTreeSet::new();
        let mut seen_contribution_ids = BTreeSet::new();
        let mut seen_group_ids = BTreeSet::new();
        let mut seen_group_names = BTreeSet::new();
        for group in &self.groups {
            if group.id.trim().is_empty()
                || group.name.trim().is_empty()
                || group.items.is_empty()
                || !seen_group_ids.insert(group.id.as_str())
                || !seen_group_names.insert(group.name.to_lowercase())
                || (group.id == "core.main" && group.name != "Main")
                || (group.id == "core.admin" && group.name != "Admin")
            {
                return false;
            }

            for item in &group.items {
                if !seen_keys.insert(item.key.as_str())
                    || !seen_hrefs.insert(item.href.as_str())
                    || (self.state == ShellNavigationStateV1::Unavailable
                        && item.owner == ShellNavigationItemOwnerV1::Contribution)
                {
                    return false;
                }
                if let Some(spec) = item_spec(&item.key) {
                    if spec.locked_group.is_some_and(|id| id != group.id)
                        || spec.label != item.label
                        || spec.href != item.href
                        || item.owner != ShellNavigationItemOwnerV1::Core
                        || item.contribution_id.is_some()
                    {
                        return false;
                    }
                } else {
                    if !manifest_contribution_is_supported(item)
                        || core_href_is_reserved(&item.href)
                    {
                        return false;
                    }
                    let contribution_id = item
                        .contribution_id
                        .as_deref()
                        .expect("validated manifest contribution identity");
                    if !seen_contribution_ids.insert(contribution_id) {
                        return false;
                    }
                }
            }
        }

        seen_keys.contains("home")
    }
}

fn core_href_is_reserved(href: &str) -> bool {
    matches!(
        href,
        "/" | "/organization"
            | "/operations"
            | "/administration"
            | "/administration/users"
            | "/administration/roles"
            | "/administration/node-types"
            | "/administration/modules"
            | "/administration/composition"
    )
}

fn manifest_contribution_is_supported(item: &ShellNavigationItemV1) -> bool {
    let Some(contribution_id) = item.contribution_id.as_deref() else {
        return false;
    };
    item.owner == ShellNavigationItemOwnerV1::Contribution
        && item.key == item.key.trim()
        && !item.key.is_empty()
        && item.key.chars().all(|character| {
            character.is_ascii_lowercase() || character.is_ascii_digit() || "_-".contains(character)
        })
        && contribution_id == contribution_id.trim()
        && contribution_id.contains('.')
        && contribution_id.chars().all(|character| {
            character.is_ascii_lowercase()
                || character.is_ascii_digit()
                || ".:_-".contains(character)
        })
        && item.label == item.label.trim()
        && (1..=80).contains(&item.label.chars().count())
        && !item.label.chars().any(char::is_control)
        && item.href.starts_with('/')
        && !item.href.starts_with("//")
        && !item.href.contains(['\r', '\n'])
}

#[derive(Clone, Copy)]
struct ItemSpec {
    label: &'static str,
    href: &'static str,
    locked_group: Option<&'static str>,
}

fn item_spec(key: &str) -> Option<ItemSpec> {
    Some(match key {
        "home" => ItemSpec {
            label: "Home",
            href: "/",
            locked_group: Some("core.main"),
        },
        "organization" => ItemSpec {
            label: "Organization",
            href: "/organization",
            locked_group: Some("core.main"),
        },
        "operations" => ItemSpec {
            label: "Operations",
            href: "/operations",
            locked_group: None,
        },
        "user_management" => ItemSpec {
            label: "User Management",
            href: "/administration/users",
            locked_group: Some("core.admin"),
        },
        "roles_access" => ItemSpec {
            label: "Roles & Access",
            href: "/administration/roles",
            locked_group: Some("core.admin"),
        },
        "node_types" => ItemSpec {
            label: "Node Types",
            href: "/administration/node-types",
            locked_group: Some("core.admin"),
        },
        "module_management" => ItemSpec {
            label: "Module Management",
            href: "/administration/modules",
            locked_group: Some("core.admin"),
        },
        "application_composition" => ItemSpec {
            label: "Application Composition",
            href: "/administration/composition",
            locked_group: Some("core.admin"),
        },
        _ => return None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn core_item(key: &str) -> ShellNavigationItemV1 {
        let spec = item_spec(key).expect("known item");
        ShellNavigationItemV1 {
            key: key.to_string(),
            label: spec.label.to_string(),
            href: spec.href.to_string(),
            owner: ShellNavigationItemOwnerV1::Core,
            contribution_id: None,
            navigation_mode: ShellNavigationModeV1::Shell,
        }
    }

    fn contribution(
        key: &str,
        label: &str,
        href: &str,
        contribution_id: &str,
        navigation_mode: ShellNavigationModeV1,
    ) -> ShellNavigationItemV1 {
        ShellNavigationItemV1 {
            key: key.to_string(),
            label: label.to_string(),
            href: href.to_string(),
            owner: ShellNavigationItemOwnerV1::Contribution,
            contribution_id: Some(contribution_id.to_string()),
            navigation_mode,
        }
    }

    fn available() -> ShellNavigationResponseV1 {
        ShellNavigationResponseV1 {
            schema_version: 3,
            policy_revision: Some(3),
            state: ShellNavigationStateV1::Available,
            groups: vec![
                ShellNavigationGroupV1 {
                    id: "core.main".into(),
                    name: "Main".into(),
                    items: vec![
                        core_item("home"),
                        core_item("organization"),
                        contribution(
                            "workflows",
                            "Workflows",
                            "/workflows",
                            "tessara.workflows.navigation",
                            ShellNavigationModeV1::Shell,
                        ),
                        contribution(
                            "forms",
                            "Forms",
                            "/forms",
                            "tessara.forms.navigation",
                            ShellNavigationModeV1::Shell,
                        ),
                        core_item("operations"),
                        contribution(
                            "dashboards",
                            "Dashboards",
                            "/dashboards",
                            "tessara.dashboards.navigation",
                            ShellNavigationModeV1::Shell,
                        ),
                        contribution(
                            "scoped_records",
                            "Scoped Records",
                            "/reference/scoped-records",
                            "tessara.reference.scoped-records.navigation",
                            ShellNavigationModeV1::Document,
                        ),
                    ],
                },
                ShellNavigationGroupV1 {
                    id: "core.admin".into(),
                    name: "Admin".into(),
                    items: vec![
                        contribution(
                            "datasets",
                            "Datasets",
                            "/datasets",
                            "tessara.datasets.navigation",
                            ShellNavigationModeV1::Shell,
                        ),
                        core_item("user_management"),
                        core_item("roles_access"),
                        core_item("node_types"),
                        core_item("module_management"),
                        core_item("application_composition"),
                    ],
                },
            ],
            unavailable: None,
        }
    }

    #[test]
    fn delivery_mode_controls_document_navigation_independently_of_ownership() {
        let dashboards = contribution(
            "dashboards",
            "Dashboards",
            "/dashboards",
            "tessara.dashboards.navigation",
            ShellNavigationModeV1::Shell,
        );
        let scoped_records = contribution(
            "scoped_records",
            "Scoped Records",
            "/reference/scoped-records",
            "tessara.reference.scoped-records.navigation",
            ShellNavigationModeV1::Document,
        );
        assert!(!dashboards.requires_document_navigation());
        assert!(scoped_records.requires_document_navigation());
        assert!(!core_item("home").requires_document_navigation());
    }

    #[test]
    fn supported_projection_accepts_policy_order_and_cross_group_movement() {
        assert!(available().is_supported());

        let mut reordered = available();
        reordered.groups[0].items.swap(2, 4);
        assert!(reordered.is_supported());

        let workflow = reordered.groups[0].items.remove(2);
        reordered.groups[1].items.insert(0, workflow);
        assert!(reordered.is_supported());

        let home = reordered.groups[0].items.remove(0);
        reordered.groups[1].items.insert(0, home);
        assert!(!reordered.is_supported());
    }

    #[test]
    fn unavailable_projection_must_be_explicit_and_core_only() {
        let mut response = available();
        response.state = ShellNavigationStateV1::Unavailable;
        response.policy_revision = None;
        response.unavailable = Some(ShellNavigationUnavailableV1 {
            code: "shell_navigation_unavailable".into(),
            message: "Contribution navigation is temporarily unavailable.".into(),
        });
        response.groups[0]
            .items
            .retain(|item| item.owner == ShellNavigationItemOwnerV1::Core);
        response.groups[1]
            .items
            .retain(|item| item.owner == ShellNavigationItemOwnerV1::Core);
        assert!(response.is_supported());

        response.groups[0].items.insert(
            1,
            contribution(
                "forms",
                "Forms",
                "/forms",
                "tessara.forms.navigation",
                ShellNavigationModeV1::Shell,
            ),
        );
        assert!(!response.is_supported());
    }

    #[test]
    fn ownership_destination_and_schema_drift_fail_closed() {
        let mut response = available();
        response.groups[0].items[0].href = "https://example.invalid".into();
        assert!(!response.is_supported());

        let mut response = available();
        response.groups[0].items[2].contribution_id = None;
        assert!(!response.is_supported());

        let mut response = available();
        response.schema_version = 4;
        assert!(!response.is_supported());
    }

    #[test]
    fn scoped_records_accepts_a_valid_configured_navigation_label() {
        let mut response = available();
        response.groups[0]
            .items
            .iter_mut()
            .find(|item| item.key == "scoped_records")
            .expect("scoped records item")
            .label = "Regional Records".into();
        assert!(response.is_supported());

        response.groups[0]
            .items
            .iter_mut()
            .find(|item| item.key == "scoped_records")
            .expect("scoped records item")
            .label = " ".into();
        assert!(!response.is_supported());
    }

    #[test]
    fn manifest_contributions_are_bounded_without_a_product_specific_key() {
        let mut response = available();
        response.groups[0].items.push(contribution(
            "example_module",
            "Example Module",
            "/reference/example",
            "example.module.navigation",
            ShellNavigationModeV1::Document,
        ));
        assert!(response.is_supported());

        response.groups[0]
            .items
            .last_mut()
            .expect("manifest contribution")
            .href = "https://example.invalid".into();
        assert!(!response.is_supported());
    }

    #[test]
    fn manifest_contributions_cannot_claim_a_hidden_core_href() {
        let mut response = available();
        response.groups[0]
            .items
            .retain(|item| item.key != "operations");
        response.groups[0].items.push(contribution(
            "example_module",
            "Example Module",
            "/operations",
            "example.module.navigation",
            ShellNavigationModeV1::Shell,
        ));

        assert!(!response.is_supported());
    }

    #[test]
    fn dashboard_is_accepted_only_once_as_its_manifest_contribution() {
        let mut response = available();
        let dashboard = response.groups[0]
            .items
            .iter()
            .find(|item| item.contribution_id.as_deref() == Some("tessara.dashboards.navigation"))
            .expect("Dashboard manifest contribution")
            .clone();

        let mut duplicate_identity = dashboard.clone();
        duplicate_identity.key = "dashboard_copy".into();
        duplicate_identity.href = "/dashboard-copy".into();
        response.groups[0].items.push(duplicate_identity);
        assert!(!response.is_supported());

        let mut core_owned = available();
        let dashboard = core_owned.groups[0]
            .items
            .iter_mut()
            .find(|item| item.key == "dashboards")
            .expect("Dashboard manifest contribution");
        dashboard.owner = ShellNavigationItemOwnerV1::Core;
        dashboard.contribution_id = None;
        assert!(!core_owned.is_supported());
    }

    #[test]
    fn unknown_http_fields_are_rejected_instead_of_ignored() {
        let mut value = serde_json::to_value(available()).expect("serialize projection");
        value
            .as_object_mut()
            .expect("object")
            .insert("future_field".into(), serde_json::Value::Bool(true));
        assert!(serde_json::from_value::<ShellNavigationResponseV1>(value).is_err());
    }
}
