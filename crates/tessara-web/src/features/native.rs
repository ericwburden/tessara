use std::collections::BTreeMap;
use std::collections::HashMap;
use std::collections::HashSet;
#[cfg(feature = "hydrate")]
use std::{cell::Cell, cell::RefCell, rc::Rc};

use icons::{
    ArrowDown, ArrowUp, CalendarDays, ChevronDown, ChevronRight, CircleDot, ExternalLink, FileText,
    Hash, ListChecks, ListFilter, LockKeyhole, Mail, PanelRight, Pencil, Plus, Search,
    SquareCheckBig, TextCursorInput, TextQuote, Trash2, X,
};
use leptos::portal::Portal;
use leptos::prelude::*;
use crate::ui::empty_view;

use serde::{Deserialize, Serialize};
use serde_json::Value;
#[cfg(feature = "hydrate")]
use wasm_bindgen::JsCast;
#[cfg(feature = "hydrate")]
use wasm_bindgen::closure::Closure;
use crate::features::native::organization::IntoNonemptyString;

#[cfg(feature = "hydrate")]
use crate::infra::http::{redirect_to_login, send_json_request};
use crate::infra::routing::{
    AccountRouteParams, FormRouteParams, NodeRouteParams, SubmissionRouteParams,
    WorkflowRouteParams, require_route_params,
};
use crate::ui::components::{
    AppShell, Breadcrumb, BreadcrumbItem, BreadcrumbLink, BreadcrumbPage, BreadcrumbSeparator,
    Button, DataTable, DropdownMenu, EmptyState, InfoListTable, InfoRow, PageHeader,
    SearchableDataTable, StatusBadge, Tabs, TabsContent, TabsList, TabsTrigger, Timestamp,
};

#[path = "native/core.rs"]
mod core;
pub use core::{HomePage, LoginPage};

#[path = "native/organization.rs"]
mod organization;
pub use organization::*;

#[path = "native/admin_models.rs"]
mod admin_models;
pub(crate) use admin_models::*;

#[path = "native/workflow_submission.rs"]
mod workflow_submission;
pub(crate) use workflow_submission::*;

#[path = "native/form_builder.rs"]
mod form_builder;
pub(crate) use form_builder::*;

#[path = "native/shared.rs"]
mod shared;
pub use shared::*;

#[cfg(feature = "hydrate")]
fn navigate_to_href(href: &str) {
    if let Some(window) = web_sys::window() {
        let _ = window.location().set_href(href);
    }
}

#[path = "native/forms.rs"]
mod forms;
pub use forms::{
    FormsDetailPage,
    FormsEditPage,
    FormsNewPage,
};
pub(crate) use forms::FormsList;



#[path = "native/workflows.rs"]
mod workflows;
pub use workflows::{
    WorkflowAssignmentsPage,
    WorkflowsDetailPage,
    WorkflowsEditPage,
    WorkflowsNewPage,
    WorkflowsPage,
};


#[path = "native/placeholders.rs"]
mod placeholders;
pub use placeholders::{
    ComponentsDetailPage, ComponentsPage, DashboardsDetailPage, DashboardsEditPage,
    DashboardsNewPage, DashboardsPage, NotFoundPage,
};

#[path = "native/datasets.rs"]
mod datasets;
pub use datasets::{
    DatasetsDetailPage, DatasetsEditPage, DatasetsNewPage, DatasetsPage, DatasetsPreviewPage,
};

#[path = "native/operations.rs"]
mod operations;
pub use operations::OperationsPage;

#[path = "native/responses.rs"]
mod responses;
pub use responses::{ResponsesDetailPage, ResponsesEditPage, ResponsesNewPage, ResponsesPage};

#[path = "native/administration.rs"]
mod administration;
pub use administration::{
    AdministrationNodeTypesPage, AdministrationPage, AdministrationRolesPage,
    AdministrationUserAccessPage, AdministrationUserDetailPage, AdministrationUserEditPage,
    AdministrationUsersPage,
};





