use leptos::context::use_context;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use tessara_datasets_contract::{
    DatasetEditorFormOptionV1, DatasetEditorPrincipalOptionV1, DatasetEditorRenderedFormV1,
    DatasetEditorScopeOptionV1, DatasetProductDefinitionV1, DatasetProductRevisionDetailV1,
    DatasetProductRevisionSummaryV1, DatasetProductSummaryV1, DatasetProductTableV1,
};

pub(crate) fn dataset_route_bootstrap() -> Option<DatasetRouteBootstrap> {
    use_context::<DatasetRouteBootstrap>()
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct DatasetEditorBootstrap {
    pub dataset: Option<DatasetProductDefinitionV1>,
    pub revision: Option<DatasetProductRevisionDetailV1>,
    pub datasets: Vec<DatasetProductSummaryV1>,
    pub forms: Vec<DatasetEditorFormOptionV1>,
    pub nodes: Vec<DatasetEditorScopeOptionV1>,
    pub principals: Vec<DatasetEditorPrincipalOptionV1>,
    pub rendered_forms: BTreeMap<String, DatasetEditorRenderedFormV1>,
    pub provider_error: Option<String>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(tag = "route", rename_all = "snake_case", deny_unknown_fields)]
pub enum DatasetRouteBootstrap {
    Directory {
        datasets: Vec<DatasetProductSummaryV1>,
        can_manage: bool,
    },
    Create {
        editor: DatasetEditorBootstrap,
        can_manage: bool,
    },
    Detail {
        dataset_id: String,
        dataset: DatasetProductDefinitionV1,
        table: Option<DatasetProductTableV1>,
        table_error: Option<String>,
        can_manage: bool,
    },
    Preview {
        dataset_id: String,
        dataset: DatasetProductDefinitionV1,
        table: Option<DatasetProductTableV1>,
        table_error: Option<String>,
        can_manage: bool,
    },
    Edit {
        dataset_id: String,
        editor: DatasetEditorBootstrap,
        can_manage: bool,
    },
    Revisions {
        dataset_id: String,
        dataset: DatasetProductDefinitionV1,
        revisions: Vec<DatasetProductRevisionSummaryV1>,
        can_manage: bool,
    },
    RevisionDetail {
        dataset_id: String,
        revision_id: String,
        revision: DatasetProductRevisionDetailV1,
        can_manage: bool,
    },
    RevisionUnavailable {
        dataset_id: String,
        revision_id: String,
        message: String,
        can_manage: bool,
    },
    RevisionDeferred {
        dataset_id: String,
        revision_id: String,
        can_manage: bool,
    },
    RevisionEdit {
        dataset_id: String,
        revision_id: String,
        editor: DatasetEditorBootstrap,
        can_manage: bool,
    },
}

impl DatasetRouteBootstrap {
    pub fn can_manage(&self) -> bool {
        match self {
            Self::Directory { can_manage, .. }
            | Self::Create { can_manage, .. }
            | Self::Detail { can_manage, .. }
            | Self::Preview { can_manage, .. }
            | Self::Edit { can_manage, .. }
            | Self::Revisions { can_manage, .. }
            | Self::RevisionDetail { can_manage, .. }
            | Self::RevisionUnavailable { can_manage, .. }
            | Self::RevisionDeferred { can_manage, .. }
            | Self::RevisionEdit { can_manage, .. } => *can_manage,
        }
    }

    pub(crate) fn directory_datasets(&self) -> Option<&[DatasetProductSummaryV1]> {
        match self {
            Self::Directory { datasets, .. } => Some(datasets),
            _ => None,
        }
    }

    pub(crate) fn dataset(&self) -> Option<&DatasetProductDefinitionV1> {
        match self {
            Self::Detail { dataset, .. }
            | Self::Preview { dataset, .. }
            | Self::Revisions { dataset, .. } => Some(dataset),
            Self::Edit { editor, .. } | Self::RevisionEdit { editor, .. } => {
                editor.dataset.as_ref()
            }
            _ => None,
        }
    }

    pub(crate) fn table(&self) -> Option<(&Option<DatasetProductTableV1>, &Option<String>)> {
        match self {
            Self::Detail {
                table, table_error, ..
            }
            | Self::Preview {
                table, table_error, ..
            } => Some((table, table_error)),
            _ => None,
        }
    }

    pub(crate) fn revisions(&self) -> Option<&[DatasetProductRevisionSummaryV1]> {
        match self {
            Self::Revisions { revisions, .. } => Some(revisions),
            _ => None,
        }
    }

    pub(crate) fn revision(&self) -> Option<&DatasetProductRevisionDetailV1> {
        match self {
            Self::RevisionDetail { revision, .. } => Some(revision),
            Self::RevisionEdit { editor, .. } => editor.revision.as_ref(),
            _ => None,
        }
    }

    pub(crate) fn editor(&self) -> Option<&DatasetEditorBootstrap> {
        match self {
            Self::Create { editor, .. }
            | Self::Edit { editor, .. }
            | Self::RevisionEdit { editor, .. } => Some(editor),
            _ => None,
        }
    }
}
