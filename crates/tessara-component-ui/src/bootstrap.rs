use leptos::context::use_context;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct ComponentDirectoryItem {
    pub component_id: String,
    pub name: String,
    pub slug: String,
    pub description: Option<String>,
    pub component_type: String,
    pub publication_state: String,
    pub current_version_id: Option<String>,
    pub current_version_label: Option<String>,
    pub draft_version_id: Option<String>,
    pub draft_version_label: Option<String>,
    pub manageable: bool,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct ComponentVersionBootstrap {
    pub component_version_id: String,
    pub component_type: String,
    pub publication_state: String,
    pub lifecycle_state: String,
    pub resource_revision: u64,
    pub version_label: String,
    #[serde(default)]
    pub version_note: String,
    #[serde(default)]
    pub dataset_reference: Value,
    #[serde(default)]
    pub config: Value,
}

pub(crate) fn component_route_bootstrap() -> Option<ComponentRouteBootstrap> {
    use_context::<ComponentRouteBootstrap>()
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct ComponentDefinitionBootstrap {
    pub component_id: String,
    pub name: String,
    pub slug: String,
    pub description: Option<String>,
    #[serde(default)]
    pub versions: Vec<ComponentVersionBootstrap>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(tag = "route", rename_all = "snake_case")]
pub enum ComponentRouteBootstrap {
    Directory {
        components: Vec<ComponentDirectoryItem>,
        can_manage: bool,
    },
    Create {
        datasets: Value,
        dataset_error: Option<String>,
    },
    Detail {
        component: ComponentDefinitionBootstrap,
        manageable: bool,
    },
    Edit {
        component: ComponentDefinitionBootstrap,
        datasets: Value,
        dataset_error: Option<String>,
    },
    Versions {
        component: ComponentDefinitionBootstrap,
        manageable: bool,
    },
    View {
        component: ComponentDefinitionBootstrap,
        manageable: bool,
    },
    DeferredDetail {
        component_ref: String,
    },
    DeferredVersions {
        component_ref: String,
    },
    DeferredView {
        component_ref: String,
    },
}

impl ComponentRouteBootstrap {
    pub fn component_ref(&self) -> Option<String> {
        match self {
            Self::Detail { component, .. }
            | Self::Edit { component, .. }
            | Self::Versions { component, .. }
            | Self::View { component, .. } => Some(component.slug.clone()),
            Self::DeferredDetail { component_ref }
            | Self::DeferredVersions { component_ref }
            | Self::DeferredView { component_ref } => Some(component_ref.clone()),
            Self::Directory { .. } | Self::Create { .. } => None,
        }
    }
}
