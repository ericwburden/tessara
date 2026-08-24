use std::{collections::BTreeMap, time::Duration};

use chrono::{DateTime, Utc};
use tessara_module_contract::{
    AuthorizationAudienceV1, MODULE_PROVIDER_COMPATIBILITY_MEDIA_TYPE,
    MODULE_PROVIDER_COMPATIBILITY_PATH, MODULE_PROVIDER_COMPATIBILITY_SCHEMA_VERSION_V1,
    ModuleDefinitionId, ModuleProviderCompatibilityExpectationV1,
    ModuleProviderCompatibilityRequestV1, ModuleProviderCompatibilityResponseV1,
    ModuleServicePrincipalV1, ServiceActionMethod, SignedEnvelopeV1,
};
use tessara_module_runtime::{ProjectedSecurityState, RuntimeCheck, SecurityStateProvider};
use uuid::Uuid;

use crate::{
    RESPONSE_FORM_BINDING, RESPONSE_WORKFLOW_ASSIGNMENTS_BINDING,
    RESPONSE_WORKFLOW_CONTEXT_BINDING, ResponseConfiguration, ResponseRuntime,
};

const CONSUMER_LAG_GRACE_SECONDS: i64 = 60;
const OPERATIONAL_PROJECTION_CACHE_SECONDS: u64 = 5;
#[cfg(test)]
pub(crate) const EXPORT_HEAD_SEQUENCE_QUERY: &str =
    "SELECT COALESCE(MAX(sequence),0) FROM response_export_changes";

#[derive(Clone, Debug)]
pub(crate) struct ResponseOperationalProjection {
    pub checks: Vec<RuntimeCheck>,
    pub diagnostic_checks: Vec<RuntimeCheck>,
    pub forms_binding_compatible: bool,
    pub workflow_binding_compatible: bool,
    pub pending_workflow_event_count: u64,
    pub export_head_sequence: u64,
    pub sanitized_findings: Vec<String>,
    pub facts: BTreeMap<String, String>,
}

impl ResponseOperationalProjection {
    pub async fn load_cached(runtime: &ResponseRuntime) -> Self {
        let mut cache = runtime.operational_projection_cache.lock().await;
        if let Some((captured_at, projection)) = cache.as_ref()
            && captured_at.elapsed() <= Duration::from_secs(OPERATIONAL_PROJECTION_CACHE_SECONDS)
        {
            return projection.clone();
        }
        let projection = Self::load(runtime).await;
        *cache = Some((tokio::time::Instant::now(), projection.clone()));
        projection
    }

    pub async fn load(runtime: &ResponseRuntime) -> Self {
        let database = sqlx::query_scalar::<_, i32>("SELECT 1")
            .fetch_one(&runtime.pool)
            .await
            .is_ok();
        let configuration = sqlx::query_as::<_, ResponseConfiguration>(
            "SELECT schema_version,display_label,provider_request_timeout_seconds,
                    workflow_event_page_size
             FROM response_module_configuration WHERE singleton=true",
        )
        .fetch_optional(&runtime.pool)
        .await
        .ok()
        .flatten();
        let timeout_seconds = configuration
            .as_ref()
            .and_then(|value| u64::try_from(value.provider_request_timeout_seconds).ok())
            .filter(|value| (1..=30).contains(value))
            .unwrap_or(5);
        let security = runtime.current_security_state().await.ok();
        let security_enabled = security.as_ref().is_some_and(|state| state.enabled);

        let (forms_origin_ready, workflow_context_origin_ready, workflow_assignments_origin_ready) = tokio::join!(
            provider_origin_compatible(
                runtime,
                security.as_ref(),
                RESPONSE_FORM_BINDING,
                timeout_seconds,
            ),
            provider_origin_compatible(
                runtime,
                security.as_ref(),
                RESPONSE_WORKFLOW_CONTEXT_BINDING,
                timeout_seconds,
            ),
            provider_origin_compatible(
                runtime,
                security.as_ref(),
                RESPONSE_WORKFLOW_ASSIGNMENTS_BINDING,
                timeout_seconds,
            ),
        );
        let provider_observations = sqlx::query_as::<_, (String, String, Option<String>)>(
            "SELECT binding_key,compatibility_state,last_stable_finding
             FROM response_provider_observations",
        )
        .fetch_all(&runtime.pool)
        .await
        .unwrap_or_default()
        .into_iter()
        .map(|(binding, state, finding)| (binding, (state, finding)))
        .collect::<BTreeMap<_, _>>();
        let forms_observation = provider_observations.get(RESPONSE_FORM_BINDING);
        let workflow_context_observation =
            provider_observations.get(RESPONSE_WORKFLOW_CONTEXT_BINDING);
        let workflow_assignments_observation =
            provider_observations.get(RESPONSE_WORKFLOW_ASSIGNMENTS_BINDING);
        let forms_binding_compatible = forms_origin_ready && forms_observation.is_some();
        let workflow_binding_compatible = workflow_context_origin_ready
            && workflow_assignments_origin_ready
            && workflow_context_observation.is_some()
            && workflow_assignments_observation.is_some();

        let event_state = sqlx::query_as::<
            _,
            (
                i64,
                Option<DateTime<Utc>>,
                i64,
                i64,
                Option<DateTime<Utc>>,
            ),
        >(
            "SELECT state.workflow_consumer_committed_sequence,state.workflow_consumer_acknowledged_at,
                    COALESCE(MAX(events.sequence),0),
                    COUNT(events.sequence) FILTER (
                        WHERE events.sequence>state.workflow_consumer_committed_sequence),
                    MIN(events.occurred_at) FILTER (
                        WHERE events.sequence>state.workflow_consumer_committed_sequence)
             FROM response_workflow_event_state state
             LEFT JOIN response_workflow_events events ON true
             WHERE state.singleton=true
             GROUP BY state.workflow_consumer_committed_sequence,
                      state.workflow_consumer_acknowledged_at",
        )
        .fetch_optional(&runtime.pool)
        .await
        .ok()
        .flatten();
        let (event_publication_ready, pending_workflow_event_count, event_acknowledged_at) =
            event_state.as_ref().map_or(
                (false, 0, None),
                |(committed, acknowledged_at, head, pending, _)| {
                    let valid = *committed >= 0 && *head >= *committed;
                    let pending = valid
                        .then(|| u64::try_from(*pending).ok())
                        .flatten()
                        .unwrap_or(0);
                    (valid, pending, *acknowledged_at)
                },
            );
        let event_consumer_ready = event_publication_ready
            && event_state
                .as_ref()
                .is_some_and(|(_, _, _, _, oldest_pending)| {
                    pending_workflow_event_count == 0 || lag_is_within_grace(*oldest_pending)
                });

        let export_epoch = sqlx::query_scalar::<_, Uuid>(
            "SELECT provider_epoch FROM response_export_state WHERE singleton=true",
        )
        .fetch_optional(&runtime.pool)
        .await
        .ok()
        .flatten();
        let export_head = sqlx::query_scalar::<_, i64>(
            "SELECT COALESCE(MAX(sequence),0) FROM response_export_changes",
        )
        .fetch_one(&runtime.pool)
        .await
        .ok();
        let export_publication_ready = export_epoch.is_some() && export_head.is_some();
        let export_head_sequence = export_head
            .and_then(|head| u64::try_from(head).ok())
            .unwrap_or(0);

        let mut diagnostic_checks = vec![
            check(
                "response.database",
                database,
                "Response database and migrations must be available",
            ),
            check(
                "response.configuration",
                configuration.is_some(),
                "Response configuration must be available and valid",
            ),
            check(
                "response.security_state",
                security_enabled,
                "Response security projection must be enabled",
            ),
            check(
                "response.provider.forms",
                forms_binding_compatible,
                "The resolved Forms provider must be reachable and contract-compatible",
            ),
            check(
                "response.provider.workflow",
                workflow_binding_compatible,
                "The resolved Workflow providers must be reachable and contract-compatible",
            ),
            check(
                "response.events.publication",
                event_publication_ready,
                "The Response Workflow event outbox must have a valid durable cursor",
            ),
            check(
                "response.events.consumer",
                event_consumer_ready,
                "The Workflow event consumer must remain within the bounded retry window",
            ),
            check(
                "response.export.publication",
                export_publication_ready,
                "The submitted-Response export must have a valid durable cursor",
            ),
        ];
        if !database {
            for check in diagnostic_checks.iter_mut().skip(1) {
                check.passing = false;
            }
        }
        let checks = diagnostic_checks
            .iter()
            .filter(|check| check.code != "response.events.consumer")
            .cloned()
            .collect::<Vec<_>>();

        let mut sanitized_findings = diagnostic_checks
            .iter()
            .filter(|check| !check.passing)
            .map(|check| check.code.clone())
            .collect::<Vec<_>>();
        if !forms_binding_compatible && let Some((_, Some(finding))) = forms_observation {
            sanitized_findings.push(finding.clone());
        }
        if !workflow_binding_compatible {
            for observation in [
                workflow_context_observation,
                workflow_assignments_observation,
            ] {
                if let Some((_, Some(finding))) = observation {
                    sanitized_findings.push(finding.clone());
                }
            }
        }
        sanitized_findings.sort();
        sanitized_findings.dedup();

        let mut facts = BTreeMap::from([
            (
                "display_label".into(),
                configuration
                    .as_ref()
                    .map_or_else(|| "unavailable".into(), |value| value.display_label.clone()),
            ),
            (
                "workflow_event_pending_count".into(),
                pending_workflow_event_count.to_string(),
            ),
            (
                "export_head_sequence".into(),
                export_head_sequence.to_string(),
            ),
        ]);
        facts.insert(
            "workflow_event_consumer_last_stable_at".into(),
            timestamp_fact(event_acknowledged_at),
        );
        Self {
            checks,
            diagnostic_checks,
            forms_binding_compatible,
            workflow_binding_compatible,
            pending_workflow_event_count,
            export_head_sequence,
            sanitized_findings,
            facts,
        }
    }

    pub fn ready(&self) -> bool {
        self.checks.iter().all(|check| check.passing)
    }
}

fn check(code: &str, passing: bool, message: &str) -> RuntimeCheck {
    RuntimeCheck {
        code: code.into(),
        passing,
        message: message.into(),
    }
}

fn lag_is_within_grace(oldest_pending: Option<DateTime<Utc>>) -> bool {
    oldest_pending.is_some_and(|oldest| {
        Utc::now().signed_duration_since(oldest).num_seconds() <= CONSUMER_LAG_GRACE_SECONDS
    })
}

fn timestamp_fact(value: Option<DateTime<Utc>>) -> String {
    value.map_or_else(|| "never".into(), |value| value.to_rfc3339())
}

async fn provider_origin_compatible(
    runtime: &ResponseRuntime,
    security: Option<&ProjectedSecurityState>,
    binding: &str,
    timeout_seconds: u64,
) -> bool {
    let Some(security) = security else {
        return false;
    };
    let Some(origin) = runtime.service_endpoints.provider_url(binding) else {
        return false;
    };
    let Some(expectation) = provider_compatibility_expectation(binding) else {
        return false;
    };
    let request = ModuleProviderCompatibilityRequestV1 {
        schema_version: MODULE_PROVIDER_COMPATIBILITY_SCHEMA_VERSION_V1,
        expectations: vec![expectation],
    };
    let Ok(body) = serde_json::to_vec(&request) else {
        return false;
    };
    let correlation_id = Uuid::new_v4();
    let Ok(service_request) = crate::provider_client::signed_operational_request(
        runtime,
        MODULE_PROVIDER_COMPATIBILITY_PATH,
        &body,
        security.installation_id,
        security.module_instance_id,
        correlation_id,
    ) else {
        return false;
    };
    let Ok(response) = runtime
        .provider_client
        .post(format!("{origin}{MODULE_PROVIDER_COMPATIBILITY_PATH}"))
        .timeout(Duration::from_secs(timeout_seconds))
        .header(
            reqwest::header::CONTENT_TYPE,
            MODULE_PROVIDER_COMPATIBILITY_MEDIA_TYPE,
        )
        .header(
            reqwest::header::ACCEPT,
            MODULE_PROVIDER_COMPATIBILITY_MEDIA_TYPE,
        )
        .header("x-tessara-module-service-request", service_request)
        .header("x-tessara-correlation-id", correlation_id.to_string())
        .body(body)
        .send()
        .await
    else {
        return false;
    };
    let status = response.status();
    let content_type = response
        .headers()
        .get(reqwest::header::CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .map(str::to_owned);
    if status != reqwest::StatusCode::OK
        || content_type.as_deref() != Some(MODULE_PROVIDER_COMPATIBILITY_MEDIA_TYPE)
    {
        return false;
    }
    let Ok(body) = crate::provider_client::bounded_response_body(response).await else {
        return false;
    };
    let Ok(envelope) =
        serde_json::from_slice::<SignedEnvelopeV1<ModuleProviderCompatibilityResponseV1>>(&body)
    else {
        return false;
    };
    if runtime
        .core_provider_compatibility_verifier
        .verify(&envelope)
        .is_err()
    {
        return false;
    }
    let provider = AuthorizationAudienceV1::CoreInstallation {
        installation_id: security.installation_id,
    };
    let Ok(definition) = ModuleDefinitionId::new(crate::MODULE_DEFINITION_ID) else {
        return false;
    };
    let consumer = ModuleServicePrincipalV1::ModuleInstance {
        module_instance_id: security.module_instance_id,
        module_definition_id: definition,
    };
    envelope
        .payload
        .validate_for(&request, &provider, &consumer, correlation_id, Utc::now())
        .is_ok()
}

fn provider_compatibility_expectation(
    binding: &str,
) -> Option<ModuleProviderCompatibilityExpectationV1> {
    let (functional_contract, contract_version, authorization_action, path) = match binding {
        RESPONSE_FORM_BINDING => (
            tessara_forms_contract::FORM_VERSION_SCHEMA_CONTRACT_ID,
            tessara_forms_contract::FORM_VERSION_SCHEMA_CONTRACT_VERSION,
            tessara_forms_contract::RESPONSE_FORM_VERSION_SCHEMA_ACTION,
            tessara_forms_contract::RESPONSE_FORM_VERSION_SCHEMA_PATH,
        ),
        RESPONSE_WORKFLOW_CONTEXT_BINDING => (
            tessara_workflows_contract::WORKFLOW_RESPONSE_CONTEXT_CONTRACT_ID,
            tessara_workflows_contract::WORKFLOW_RESPONSE_CONTEXT_VERSION,
            tessara_workflows_contract::WORKFLOW_RESPONSE_CONTEXT_ACTION,
            tessara_workflows_contract::WORKFLOW_RESPONSE_CONTEXT_PATH,
        ),
        RESPONSE_WORKFLOW_ASSIGNMENTS_BINDING => (
            tessara_workflows_contract::WORKFLOW_RESPONSE_ASSIGNMENT_CATALOG_CONTRACT_ID,
            tessara_workflows_contract::WORKFLOW_RESPONSE_ASSIGNMENT_CATALOG_VERSION,
            tessara_workflows_contract::WORKFLOW_RESPONSE_ASSIGNMENT_CATALOG_ACTION,
            tessara_workflows_contract::WORKFLOW_RESPONSE_ASSIGNMENT_CATALOG_PATH,
        ),
        _ => return None,
    };
    Some(ModuleProviderCompatibilityExpectationV1 {
        dependency_binding: binding.into(),
        functional_contract: functional_contract.into(),
        contract_version: contract_version.into(),
        authorization_action: authorization_action.into(),
        method: ServiceActionMethod::Post,
        path: path.into(),
    })
}

#[cfg(test)]
mod tests {
    use std::{
        collections::BTreeMap,
        sync::{
            Arc,
            atomic::{AtomicBool, Ordering},
        },
    };

    use axum::{
        Router,
        body::{Body, Bytes},
        extract::State,
        http::{HeaderMap, StatusCode, header},
        response::{IntoResponse, Response},
        routing::post,
    };
    use tessara_module_contract::{
        ModuleServiceIdentityRegistryV1, ProtocolSignaturePurposeV1, PurposeBoundSigningKeyV1,
    };
    use tessara_module_runtime::CoreVerifiers;
    use uuid::Uuid;

    use super::*;
    use crate::{ResponseCoreVerifiers, ResponseServiceEndpoints, ResponseValidationFaultControl};

    #[derive(Clone)]
    struct ProviderCompatibilityTestState {
        enabled: Arc<AtomicBool>,
        signer: Arc<PurposeBoundSigningKeyV1>,
        installation_id: Uuid,
        module_instance_id: Uuid,
    }

    async fn signed_provider_compatibility(
        State(state): State<ProviderCompatibilityTestState>,
        headers: HeaderMap,
        body: Bytes,
    ) -> Response {
        if !state.enabled.load(Ordering::SeqCst)
            || headers
                .get("x-tessara-module-service-request")
                .and_then(|value| value.to_str().ok())
                .is_none_or(str::is_empty)
        {
            return StatusCode::NOT_FOUND.into_response();
        }
        let Some(correlation_id) = headers
            .get("x-tessara-correlation-id")
            .and_then(|value| value.to_str().ok())
            .and_then(|value| Uuid::parse_str(value).ok())
        else {
            return StatusCode::NOT_FOUND.into_response();
        };
        let Ok(request) = serde_json::from_slice::<ModuleProviderCompatibilityRequestV1>(&body)
        else {
            return StatusCode::NOT_FOUND.into_response();
        };
        if request.validate().is_err() {
            return StatusCode::NOT_FOUND.into_response();
        }
        let issued_at = Utc::now();
        let Ok(request_digest) = request.canonical_digest() else {
            return StatusCode::NOT_FOUND.into_response();
        };
        let Ok(module_definition_id) = ModuleDefinitionId::new(crate::MODULE_DEFINITION_ID) else {
            return StatusCode::NOT_FOUND.into_response();
        };
        let payload = ModuleProviderCompatibilityResponseV1 {
            schema_version: MODULE_PROVIDER_COMPATIBILITY_SCHEMA_VERSION_V1,
            provider: AuthorizationAudienceV1::CoreInstallation {
                installation_id: state.installation_id,
            },
            consumer: ModuleServicePrincipalV1::ModuleInstance {
                module_instance_id: state.module_instance_id,
                module_definition_id,
            },
            request_digest,
            expectations: request.expectations,
            correlation_id,
            issued_at,
            expires_at: issued_at + chrono::Duration::seconds(30),
        };
        let Ok(envelope) = state.signer.sign(payload) else {
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        };
        Response::builder()
            .status(StatusCode::OK)
            .header(
                header::CONTENT_TYPE,
                MODULE_PROVIDER_COMPATIBILITY_MEDIA_TYPE,
            )
            .body(Body::from(serde_json::to_vec(&envelope).unwrap()))
            .unwrap()
    }

    #[test]
    fn provider_observations_and_consumer_lag_fail_closed() {
        for binding in [
            RESPONSE_FORM_BINDING,
            RESPONSE_WORKFLOW_CONTEXT_BINDING,
            RESPONSE_WORKFLOW_ASSIGNMENTS_BINDING,
        ] {
            let expectation = provider_compatibility_expectation(binding)
                .expect("required binding compatibility identity");
            assert_eq!(expectation.dependency_binding, binding);
            assert_eq!(expectation.contract_version, "1.0.0");
            assert_eq!(expectation.method, ServiceActionMethod::Post);
        }
        assert!(provider_compatibility_expectation("unknown").is_none());

        assert!(lag_is_within_grace(Some(Utc::now())));
        assert!(!lag_is_within_grace(Some(
            Utc::now() - chrono::Duration::seconds(CONSUMER_LAG_GRACE_SECONDS + 1)
        )));
        assert!(!lag_is_within_grace(None));
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn operational_projection_shares_provider_and_consumer_truth(sql_pool: sqlx::PgPool) {
        let installation_id = Uuid::new_v4();
        let module_instance_id = Uuid::new_v4();
        let core_compatibility = Arc::new(signing_key(
            "tessara.core",
            "core-compatibility",
            ProtocolSignaturePurposeV1::ProviderCompatibilityResponse,
            7,
        ));
        let provider_compatibility_enabled = Arc::new(AtomicBool::new(false));
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
            .await
            .expect("provider health listener");
        let origin = format!(
            "http://{}",
            listener.local_addr().expect("listener address")
        );
        let provider_state = ProviderCompatibilityTestState {
            enabled: provider_compatibility_enabled.clone(),
            signer: core_compatibility.clone(),
            installation_id,
            module_instance_id,
        };
        let server = tokio::spawn(async move {
            axum::serve(
                listener,
                Router::new()
                    .route(
                        MODULE_PROVIDER_COMPATIBILITY_PATH,
                        post(signed_provider_compatibility),
                    )
                    .with_state(provider_state),
            )
            .await
            .expect("provider health server");
        });
        sqlx::query(
            "INSERT INTO response_module_security_state
             (singleton,schema_version,installation_id,module_instance_id,
              authorization_revision,organization_revision,enabled,document_state)
             VALUES(true,1,$1,$2,1,1,true,'enabled')",
        )
        .bind(installation_id)
        .bind(module_instance_id)
        .execute(&sql_pool)
        .await
        .expect("security state");

        let authorization = signing_key(
            "tessara.core",
            "core-authorization",
            ProtocolSignaturePurposeV1::AuthorizationGrant,
            1,
        );
        let shell = signing_key(
            "tessara.core",
            "core-shell",
            ProtocolSignaturePurposeV1::ShellContext,
            2,
        );
        let core_service = signing_key(
            "tessara.core",
            "core-service",
            ProtocolSignaturePurposeV1::ModuleServiceRequest,
            3,
        );
        let core_bootstrap = signing_key(
            "tessara.core",
            "core-bootstrap",
            ProtocolSignaturePurposeV1::OwnerBootstrapAuthorization,
            4,
        );
        let response_service = Arc::new(signing_key(
            "tessara.responses",
            "response-service",
            ProtocolSignaturePurposeV1::ModuleServiceRequest,
            5,
        ));
        let response_bootstrap = Arc::new(signing_key(
            "tessara.responses",
            "response-bootstrap",
            ProtocolSignaturePurposeV1::OwnerBootstrapReceipt,
            6,
        ));
        let endpoints = ResponseServiceEndpoints::new(
            &origin,
            BTreeMap::from([
                (RESPONSE_FORM_BINDING.into(), origin.clone()),
                (RESPONSE_WORKFLOW_CONTEXT_BINDING.into(), origin.clone()),
                (RESPONSE_WORKFLOW_ASSIGNMENTS_BINDING.into(), origin.clone()),
            ]),
        )
        .expect("provider endpoints");
        let registry = ModuleServiceIdentityRegistryV1::from_json(
            r#"{"schema_version":1,"identities":{"tessara.components":{"key_id":"component-development-v1","public_key":"11qYAYKxCrfVS_7TyWQHOg7hcvPapiMlrwIaaPcHURo"}}}"#,
        )
        .expect("service registry");
        let runtime = ResponseRuntime::new(
            sql_pool.clone(),
            ResponseCoreVerifiers {
                runtime: CoreVerifiers {
                    authorization: authorization.verifier(),
                    shell: shell.verifier(),
                },
                service_request: core_service.verifier(),
                provider_compatibility: core_compatibility.verifier(),
                owner_bootstrap: core_bootstrap.verifier(),
            },
            response_service,
            registry,
            endpoints,
            response_bootstrap,
            ResponseValidationFaultControl::disabled(),
        );

        let unresolved = ResponseOperationalProjection::load(&runtime).await;
        assert!(!unresolved.ready());
        assert!(!unresolved.forms_binding_compatible);
        assert!(!unresolved.workflow_binding_compatible);
        sqlx::query(
            "UPDATE response_provider_observations
             SET compatibility_state='compatible',last_observed_at=now(),
                 last_compatible_at=now(),last_stable_finding=NULL",
        )
        .execute(&sql_pool)
        .await
        .expect("authenticated bootstrap provider compatibility");
        provider_compatibility_enabled.store(true, Ordering::SeqCst);
        let initial = ResponseOperationalProjection::load(&runtime).await;
        assert!(initial.ready());
        assert!(initial.forms_binding_compatible);
        assert!(initial.workflow_binding_compatible);
        assert_eq!(initial.pending_workflow_event_count, 0);
        assert_eq!(initial.export_head_sequence, 0);
        assert_eq!(initial.facts["display_label"], "Responses");

        sqlx::query(
            "UPDATE response_provider_observations
             SET compatibility_state='incompatible',last_observed_at=now(),
                 last_stable_finding='response.provider.forms.incompatible'
             WHERE binding_key=$1",
        )
        .bind(RESPONSE_FORM_BINDING)
        .execute(&sql_pool)
        .await
        .expect("incompatible Forms observation");
        let recovered = ResponseOperationalProjection::load(&runtime).await;
        assert!(recovered.ready());
        assert!(recovered.forms_binding_compatible);
        provider_compatibility_enabled.store(false, Ordering::SeqCst);
        let incompatible = ResponseOperationalProjection::load(&runtime).await;
        assert!(!incompatible.ready());
        assert!(!incompatible.forms_binding_compatible);
        assert!(
            incompatible
                .sanitized_findings
                .contains(&"response.provider.forms.incompatible".into())
        );
        provider_compatibility_enabled.store(true, Ordering::SeqCst);

        sqlx::query(
            "UPDATE response_provider_observations
             SET compatibility_state='compatible',last_observed_at=now(),
                 last_compatible_at=now(),last_stable_finding=NULL
             WHERE binding_key=$1",
        )
        .bind(RESPONSE_FORM_BINDING)
        .execute(&sql_pool)
        .await
        .expect("compatible Forms observation");
        sqlx::query(
            "INSERT INTO response_export_changes
             (response_id,form_version_id,node_id,change_kind,payload,content_digest,occurred_at)
             VALUES($1,$2,$3,'upsert','{}'::jsonb,$4,now()-interval '61 seconds')",
        )
        .bind(Uuid::new_v4())
        .bind(Uuid::new_v4())
        .bind(Uuid::new_v4())
        .bind(format!("sha256:{}", "a".repeat(64)))
        .execute(&sql_pool)
        .await
        .expect("stale export backlog");
        sqlx::query(
            "INSERT INTO response_export_changes
             (response_id,form_version_id,node_id,change_kind,payload,content_digest,occurred_at)
             VALUES($1,$2,$3,'upsert','{}'::jsonb,$4,now())",
        )
        .bind(Uuid::new_v4())
        .bind(Uuid::new_v4())
        .bind(Uuid::new_v4())
        .bind(format!("sha256:{}", "c".repeat(64)))
        .execute(&sql_pool)
        .await
        .expect("unrelated export sequence gap");
        let publication = ResponseOperationalProjection::load(&runtime).await;
        assert_eq!(publication.export_head_sequence, 2);
        assert!(publication.ready());
        assert!(
            publication
                .diagnostic_checks
                .iter()
                .any(|check| { check.code == "response.export.publication" && check.passing })
        );
        server.abort();
    }

    fn signing_key(
        issuer: &str,
        key_id: &str,
        purpose: ProtocolSignaturePurposeV1,
        seed: u8,
    ) -> PurposeBoundSigningKeyV1 {
        PurposeBoundSigningKeyV1::from_secret_bytes(issuer, key_id, purpose, [seed; 32])
            .expect("test signing key")
    }
}
