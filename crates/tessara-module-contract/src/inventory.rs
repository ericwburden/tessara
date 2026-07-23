//! Shared wire projections for independently deployed module inventory.
//!
//! Core persistence and the browser may each use richer internal models, but
//! the versioned HTTP object is declared once here so they cannot silently
//! diverge.

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct IndependentDefinitionV1 {
    pub id: String,
    pub display_name: String,
    pub description: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct IndependentReleaseV1 {
    pub id: String,
    pub version: String,
    pub manifest_digest: String,
    pub runtime_image: String,
    pub publisher: String,
    pub trust: String,
    pub compatibility: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct IndependentInstanceV1 {
    pub id: String,
    pub identity: String,
    pub data: String,
    pub database_name: String,
    pub installed: bool,
    pub deployed: bool,
    pub configured: bool,
    pub ready: bool,
    pub enabled: bool,
    pub healthy: bool,
    pub observed_at: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct IndependentConfigurationV1 {
    pub declared: bool,
    pub valid: bool,
    pub display_label: String,
    pub retention_mode: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct IndependentDiagnosticsV1 {
    pub readiness_path: String,
    pub liveness_path: String,
    pub public_route: String,
}
