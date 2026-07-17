//! Versioned, actor-filtered shell navigation received from the application API.
//!
//! The browser treats this projection as display state only. Route guards remain
//! independent authorization boundaries, and an unsupported projection fails
//! closed to the small Core fallback owned by the shell.

use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};

pub const SHELL_NAVIGATION_SCHEMA_VERSION_V1: u16 = 2;

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

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ShellNavigationItemV1 {
    pub key: String,
    pub label: String,
    pub href: String,
    pub owner: ShellNavigationItemOwnerV1,
    pub contribution_id: Option<String>,
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
                let Some(spec) = item_spec(&item.key) else {
                    return false;
                };
                if !seen_keys.insert(item.key.as_str())
                    || spec.locked_group.is_some_and(|id| id != group.id)
                    || spec.label != item.label
                    || spec.href != item.href
                    || spec.owner != item.owner
                    || spec.contribution_id != item.contribution_id.as_deref()
                    || (self.state == ShellNavigationStateV1::Unavailable
                        && item.owner == ShellNavigationItemOwnerV1::Contribution)
                {
                    return false;
                }
            }
        }

        seen_keys.contains("home")
    }
}

#[derive(Clone, Copy)]
struct ItemSpec {
    label: &'static str,
    href: &'static str,
    locked_group: Option<&'static str>,
    owner: ShellNavigationItemOwnerV1,
    contribution_id: Option<&'static str>,
}

fn item_spec(key: &str) -> Option<ItemSpec> {
    let core = ShellNavigationItemOwnerV1::Core;
    let contribution = ShellNavigationItemOwnerV1::Contribution;
    Some(match key {
        "home" => ItemSpec {
            label: "Home",
            href: "/",
            locked_group: Some("core.main"),
            owner: core,
            contribution_id: None,
        },
        "organization" => ItemSpec {
            label: "Organization",
            href: "/organization",
            locked_group: Some("core.main"),
            owner: core,
            contribution_id: None,
        },
        "forms" => ItemSpec {
            label: "Forms",
            href: "/forms",
            locked_group: None,
            owner: contribution,
            contribution_id: Some("tessara.forms.navigation"),
        },
        "workflows" => ItemSpec {
            label: "Workflows",
            href: "/workflows",
            locked_group: None,
            owner: contribution,
            contribution_id: Some("tessara.workflows.navigation"),
        },
        "responses" => ItemSpec {
            label: "Responses",
            href: "/responses",
            locked_group: None,
            owner: contribution,
            contribution_id: Some("tessara.responses.navigation"),
        },
        "operations" => ItemSpec {
            label: "Operations",
            href: "/operations",
            locked_group: None,
            owner: core,
            contribution_id: None,
        },
        "components" => ItemSpec {
            label: "Components",
            href: "/components",
            locked_group: None,
            owner: contribution,
            contribution_id: Some("tessara.components.navigation"),
        },
        "dashboards" => ItemSpec {
            label: "Dashboards",
            href: "/dashboards",
            locked_group: None,
            owner: contribution,
            contribution_id: Some("tessara.dashboards.navigation"),
        },
        "datasets" => ItemSpec {
            label: "Datasets",
            href: "/datasets",
            locked_group: None,
            owner: contribution,
            contribution_id: Some("tessara.datasets.navigation"),
        },
        "user_management" => ItemSpec {
            label: "User Management",
            href: "/administration/users",
            locked_group: Some("core.admin"),
            owner: core,
            contribution_id: None,
        },
        "roles_access" => ItemSpec {
            label: "Roles & Access",
            href: "/administration/roles",
            locked_group: Some("core.admin"),
            owner: core,
            contribution_id: None,
        },
        "node_types" => ItemSpec {
            label: "Node Types",
            href: "/administration/node-types",
            locked_group: Some("core.admin"),
            owner: core,
            contribution_id: None,
        },
        "module_management" => ItemSpec {
            label: "Module Management",
            href: "/administration/modules",
            locked_group: Some("core.admin"),
            owner: core,
            contribution_id: None,
        },
        _ => return None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn item(key: &str) -> ShellNavigationItemV1 {
        let spec = item_spec(key).expect("known item");
        ShellNavigationItemV1 {
            key: key.to_string(),
            label: spec.label.to_string(),
            href: spec.href.to_string(),
            owner: spec.owner,
            contribution_id: spec.contribution_id.map(str::to_string),
        }
    }

    fn available() -> ShellNavigationResponseV1 {
        ShellNavigationResponseV1 {
            schema_version: 2,
            policy_revision: Some(3),
            state: ShellNavigationStateV1::Available,
            groups: vec![
                ShellNavigationGroupV1 {
                    id: "core.main".into(),
                    name: "Main".into(),
                    items: vec![
                        item("home"),
                        item("organization"),
                        item("workflows"),
                        item("forms"),
                        item("operations"),
                        item("dashboards"),
                    ],
                },
                ShellNavigationGroupV1 {
                    id: "core.admin".into(),
                    name: "Admin".into(),
                    items: vec![
                        item("datasets"),
                        item("user_management"),
                        item("roles_access"),
                        item("node_types"),
                        item("module_management"),
                    ],
                },
            ],
            unavailable: None,
        }
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

        response.groups[0].items.insert(1, item("forms"));
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
        response.schema_version = 3;
        assert!(!response.is_supported());
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
