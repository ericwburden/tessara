use std::{
    collections::BTreeMap,
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    },
};

use axum::{
    Json, Router,
    body::{Body, to_bytes},
    extract::State,
    http::{HeaderMap, Request, StatusCode},
    routing::post,
};
use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
use chrono::{Duration, Utc};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use sqlx::{PgPool, Row, postgres::PgPoolOptions};
use tessara_component_module::{ComponentModuleState, MANAGE_CAPABILITY, READ_CAPABILITY, router};
use tessara_components_contract::{
    COMPONENT_CONTRACT_SCHEMA_VERSION, COMPONENT_RESOURCE_TYPE, ComponentAction,
    ComponentRenderKind, ComponentRenderRequest, ComponentResolutionRequest,
    ComponentVersionReference,
};
use tessara_datasets_contract::{
    DATASET_CONTRACT_SCHEMA_VERSION, DatasetAction, DatasetCompatibilityFinding,
    DatasetCompatibilityRequest, DatasetCompatibilityResponse, DatasetExecutionRequest,
    DatasetExecutionResponse, DatasetExecutionRow, DatasetFieldContract, DatasetMajorLineMetadata,
    DatasetMajorLineReference, DatasetSchemaRequest,
};
use tessara_module_contract::{
    AUTHORIZATION_EXCHANGE_SCHEMA_VERSION_V2, AUTHORIZATION_GRANT_SCHEMA_VERSION_V2,
    AUTHORIZATION_GRANT_SCHEMA_VERSION_V3, AuthorizationAudienceV1, AuthorizationExchangeRequestV2,
    AuthorizationExchangeResponseV2, AuthorizationGrantOperationV1, AuthorizationGrantV2,
    AuthorizationGrantV3, CapabilityScopeBindingV1, DependencyBindingKey, FunctionalContractId,
    ModuleDefinitionId, ModuleServiceIdentityRegistryV1, ModuleServicePrincipalV1,
    ModuleServiceRequestV1, ProtocolSignaturePurposeV1, PurposeBoundSigningKeyV1, ResourceOwner,
    SecurityCapabilityId, SignedEnvelopeV1, TypedResourceReference,
};
use tower::ServiceExt;
use uuid::Uuid;

#[tokio::test]
async fn extracted_component_product_owns_crud_versions_lifecycle_render_and_nondisclosure() {
    let database_url = std::env::var("TEST_COMPONENT_MODULE_DATABASE_URL")
        .expect("TEST_COMPONENT_MODULE_DATABASE_URL is required for Component integration tests");
    assert_database_url_names_a_database(&database_url);
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .expect("Component test database is reachable");
    let database_name: String = sqlx::query_scalar("SELECT current_database()")
        .fetch_one(&pool)
        .await
        .expect("Component test database identity is readable");
    assert_disposable_database_name(&database_name);
    sqlx::migrate!()
        .run(&pool)
        .await
        .expect("Component migrations apply");
    reset_component_product(&pool).await;

    let installation_id = Uuid::new_v4();
    let module_instance_id = Uuid::new_v4();
    let dashboard_instance_id = Uuid::new_v4();
    let actor_id = Uuid::new_v4();
    let allowed_scope = Uuid::new_v4();
    let hidden_scope = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO component_security_state
         (singleton,installation_id,module_instance_id,authorization_revision,
          organization_revision,enabled,document_state)
         VALUES(true,$1,$2,7,11,true,'enabled')",
    )
    .bind(installation_id)
    .bind(module_instance_id)
    .execute(&pool)
    .await
    .expect("Component security state is installed");

    let dataset_reference =
        DatasetMajorLineReference::from_parts(installation_id, Uuid::new_v4(), 1)
            .expect("canonical Dataset major-line reference");
    let dataset = DatasetStub::new(
        dataset_reference.clone(),
        allowed_scope,
        Arc::new(signing_key(
            "tessara.core",
            "component-product-test",
            ProtocolSignaturePurposeV1::AuthorizationGrant,
            81,
        )),
    );
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind Dataset stub");
    let dataset_address = listener.local_addr().expect("Dataset stub address");
    let dataset_server = tokio::spawn({
        let dataset = dataset.clone();
        async move {
            axum::serve(
                listener,
                Router::new()
                    .route("/api/private/datasets/schema", post(dataset_schema))
                    .route(
                        "/api/private/datasets/compatibility",
                        post(dataset_compatibility),
                    )
                    .route("/api/private/datasets/execute", post(dataset_execute))
                    .route(
                        "/api/private/module-authorization/exchange",
                        post(core_authorization_exchange),
                    )
                    .with_state(dataset),
            )
            .await
            .expect("serve Dataset stub");
        }
    });

    let core_signer = signing_key(
        "tessara.core",
        "component-product-test",
        ProtocolSignaturePurposeV1::AuthorizationGrant,
        81,
    );
    let shell_signer = signing_key(
        "tessara.core",
        "component-shell-test",
        ProtocolSignaturePurposeV1::ShellContext,
        82,
    );
    let dashboard_signer = signing_key(
        "tessara.dashboards",
        "dashboard-service-test",
        ProtocolSignaturePurposeV1::ModuleServiceRequest,
        83,
    );
    let component_service_signer = Arc::new(signing_key(
        "tessara.components",
        "component-service-test",
        ProtocolSignaturePurposeV1::ModuleServiceRequest,
        84,
    ));
    let service_identity_registry = ModuleServiceIdentityRegistryV1::from_json(
        &json!({
            "schema_version": 1,
            "identities": {
                "tessara.dashboards": {
                    "key_id": "dashboard-service-test",
                    "public_key": URL_SAFE_NO_PAD.encode(
                        dashboard_signer.verifier().public_key_bytes()
                    )
                }
            }
        })
        .to_string(),
    )
    .expect("test service identity registry");
    let app = router(
        ComponentModuleState::new(
            pool.clone(),
            core_signer.verifier(),
            shell_signer.verifier(),
            service_identity_registry,
            component_service_signer,
            format!("http://{dataset_address}"),
        )
        .expect("Component module state"),
    );
    let grants = GrantContext {
        installation_id,
        module_instance_id,
        actor_id,
        correlation_id: Uuid::new_v4(),
    };
    let cached_bootstrap_input = json!({
        "schema_version": "tessara.io/component-bootstrap/v1",
        "components": []
    });
    let cached_bootstrap_digest = tessara_composition::canonical_digest(&cached_bootstrap_input)
        .expect("cached bootstrap input digest");
    let cached_bootstrap_result =
        tessara_composition::canonical_digest(&BTreeMap::<String, String>::new())
            .expect("cached bootstrap result digest");
    let cached_bootstrap_receipt = json!({
        "receipt": {
            "owner": "tessara.components",
            "schema_version": "tessara.io/component-bootstrap/v1",
            "input_digest": cached_bootstrap_digest,
            "result_digest": cached_bootstrap_result,
            "changed": true,
            "resource_ids": {}
        }
    });
    sqlx::query(
        "INSERT INTO component_bootstrap_receipts
         (idempotency_key,input_digest,desired_revision,receipt)
         VALUES($1,$2,1,$3)",
    )
    .bind("cross-installation-replay")
    .bind(cached_bootstrap_digest.to_string())
    .bind(cached_bootstrap_receipt)
    .execute(&pool)
    .await
    .expect("cached Component bootstrap receipt");
    let cross_installation_replay = control_request(
        &app,
        "/api/private/bootstrap",
        json!({
            "installation_id": Uuid::new_v4(),
            "desired_revision": 1,
            "idempotency_key": "cross-installation-replay",
            "input_digest": cached_bootstrap_digest,
            "input": cached_bootstrap_input
        }),
    )
    .await;
    assert_eq!(
        cross_installation_replay.status,
        StatusCode::BAD_REQUEST,
        "a cached Component receipt must remain installation-bound: {}",
        cross_installation_replay.body
    );
    let table_config = json!({
        "visible_columns": ["label", "amount"],
        "search_fields": ["label"],
        "default_sort": {"field_key": "label", "direction": "asc"},
        "page_size": 25,
        "display_labels": {"amount": "Total"}
    });

    let created = module_request(
        &app,
        &core_signer,
        &grants,
        GrantSpec::mutation("components.create", MANAGE_CAPABILITY, allowed_scope),
        "POST",
        "/api/admin/components",
        Some("component-create-1"),
        json!({
            "schema_version": 1,
            "name": "Revenue table",
            "slug": "revenue-table",
            "description": "Extracted Component product coverage.",
            "version": {
                "dataset_reference": dataset_reference,
                "component_type": "table",
                "config": table_config,
                "version_note": "Initial draft"
            }
        }),
    )
    .await;
    assert_eq!(created.status, StatusCode::OK, "{}", created.body);
    let component_id = uuid_at(&created.body, "/component_id");
    let first_version_id = uuid_at(&created.body, "/versions/0/component_version_id");
    assert_eq!(created.body["versions"][0]["publication_state"], "draft");

    let updated = module_request(
        &app,
        &core_signer,
        &grants,
        GrantSpec::mutation("components.update", MANAGE_CAPABILITY, allowed_scope),
        "PUT",
        &format!("/api/admin/components/{component_id}"),
        Some("component-update-1"),
        json!({
            "schema_version": 1,
            "name": "Revenue table renamed",
            "slug": "revenue-table",
            "description": "Updated through the extracted owner."
        }),
    )
    .await;
    assert_eq!(updated.status, StatusCode::OK, "{}", updated.body);
    assert_eq!(updated.body["outcome"], "updated");

    publish(
        &app,
        &core_signer,
        &grants,
        component_id,
        first_version_id,
        allowed_scope,
        "component-publish-1",
    )
    .await;

    let public_detail = module_request(
        &app,
        &core_signer,
        &grants,
        GrantSpec::read("components.get", READ_CAPABILITY, allowed_scope),
        "GET",
        "/api/components/revenue-table",
        None,
        Value::Null,
    )
    .await;
    assert_eq!(
        public_detail.status,
        StatusCode::OK,
        "{}",
        public_detail.body
    );
    assert_eq!(public_detail.body["name"], "Revenue table renamed");
    assert_eq!(
        public_detail.body["versions"][0]["publication_state"],
        "published"
    );

    // A Component is manageable only when the grant contains every scope in
    // its complete history. Public reads remain an overlap projection and
    // never disclose drafts, even when the caller can read the draft's scope.
    let mixed_scope_draft_id = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO component_versions
         (id,component_id,dataset_reference,dataset_scope_node_ids,component_type,status,
          lifecycle_state,version_number,version_label,version_note,config)
         VALUES($1,$2,$3,$4,'table','draft','active',2,'2.0.0',$5,$6)",
    )
    .bind(mixed_scope_draft_id)
    .bind(component_id)
    .bind(serde_json::to_value(&dataset_reference).expect("Dataset reference JSON"))
    .bind(vec![hidden_scope])
    .bind("Mixed-scope draft")
    .bind(&table_config)
    .execute(&pool)
    .await
    .expect("insert mixed-scope draft");

    let reader_projection = module_request(
        &app,
        &core_signer,
        &grants,
        GrantSpec::read("components.get", READ_CAPABILITY, allowed_scope),
        "GET",
        &format!("/api/components/{component_id}"),
        None,
        Value::Null,
    )
    .await;
    assert_eq!(
        reader_projection.status,
        StatusCode::OK,
        "{}",
        reader_projection.body
    );
    assert_eq!(
        reader_projection.body["versions"].as_array().unwrap().len(),
        1
    );
    assert_eq!(
        reader_projection.body["versions"][0]["publication_state"],
        "published"
    );

    let draft_scope_reader = module_request(
        &app,
        &core_signer,
        &grants,
        GrantSpec::read("components.get", READ_CAPABILITY, hidden_scope),
        "GET",
        &format!("/api/components/{component_id}"),
        None,
        Value::Null,
    )
    .await;
    assert_eq!(draft_scope_reader.status, StatusCode::NOT_FOUND);

    for scope in [allowed_scope, hidden_scope] {
        let partial_manage = module_request(
            &app,
            &core_signer,
            &grants,
            GrantSpec::read("components.edit", MANAGE_CAPABILITY, scope),
            "GET",
            &format!("/api/admin/components/{component_id}"),
            None,
            Value::Null,
        )
        .await;
        assert_eq!(partial_manage.status, StatusCode::NOT_FOUND);
    }
    let partial_manage_list = module_request(
        &app,
        &core_signer,
        &grants,
        GrantSpec::read(
            "components.list_manageable",
            MANAGE_CAPABILITY,
            allowed_scope,
        ),
        "GET",
        "/api/admin/components",
        None,
        Value::Null,
    )
    .await;
    assert_eq!(partial_manage_list.status, StatusCode::OK);
    assert!(partial_manage_list.body.as_array().unwrap().is_empty());

    let full_manage = module_request(
        &app,
        &core_signer,
        &grants,
        GrantSpec::read("components.edit", MANAGE_CAPABILITY, allowed_scope)
            .with_additional_scope(hidden_scope),
        "GET",
        &format!("/api/admin/components/{component_id}"),
        None,
        Value::Null,
    )
    .await;
    assert_eq!(full_manage.status, StatusCode::OK, "{}", full_manage.body);
    assert_eq!(full_manage.body["versions"].as_array().unwrap().len(), 2);
    assert!(
        full_manage.body["versions"]
            .as_array()
            .unwrap()
            .iter()
            .any(|version| {
                version["component_version_id"] == mixed_scope_draft_id.to_string()
                    && version["publication_state"] == "draft"
            })
    );

    let denied_partial_mutation = module_request(
        &app,
        &core_signer,
        &grants,
        GrantSpec::mutation("components.update", MANAGE_CAPABILITY, allowed_scope),
        "PUT",
        &format!("/api/admin/components/{component_id}"),
        Some("mixed-scope-metadata-update"),
        json!({
            "schema_version": 1,
            "name": "Must remain undisclosed",
            "slug": "must-remain-undisclosed",
            "description": null
        }),
    )
    .await;
    assert_eq!(denied_partial_mutation.status, StatusCode::NOT_FOUND);
    assert_eq!(
        sqlx::query_scalar::<_, String>("SELECT name FROM components WHERE id=$1")
            .bind(component_id)
            .fetch_one(&pool)
            .await
            .expect("unchanged Component metadata"),
        "Revenue table renamed"
    );

    let draft_reference =
        component_version_reference(installation_id, module_instance_id, mixed_scope_draft_id);
    let absent_draft_reference =
        component_version_reference(installation_id, module_instance_id, Uuid::new_v4());
    let draft_resolution = private_provider_request(
        &app,
        &core_signer,
        &dashboard_signer,
        &grants,
        dashboard_instance_id,
        hidden_scope,
        "/api/private/components/resolve",
        serde_json::to_value(ComponentResolutionRequest::new(
            ComponentAction::ResolveMetadata,
            draft_reference.clone(),
            None,
        ))
        .expect("draft Component resolution request"),
    )
    .await;
    let absent_draft_resolution = private_provider_request(
        &app,
        &core_signer,
        &dashboard_signer,
        &grants,
        dashboard_instance_id,
        hidden_scope,
        "/api/private/components/resolve",
        serde_json::to_value(ComponentResolutionRequest::new(
            ComponentAction::ResolveMetadata,
            absent_draft_reference.clone(),
            None,
        ))
        .expect("absent Component resolution request"),
    )
    .await;
    assert_eq!(draft_resolution.status, StatusCode::OK);
    assert_eq!(draft_resolution.status, absent_draft_resolution.status);
    assert_eq!(draft_resolution.body, absent_draft_resolution.body);
    assert_eq!(
        draft_resolution.body["resolution"]["access_state"],
        "not_evaluated"
    );
    assert!(draft_resolution.body["metadata"].is_null());

    let draft_render = private_provider_request(
        &app,
        &core_signer,
        &dashboard_signer,
        &grants,
        dashboard_instance_id,
        hidden_scope,
        "/api/private/components/render",
        serde_json::to_value(ComponentRenderRequest {
            schema_version: COMPONENT_CONTRACT_SCHEMA_VERSION,
            action: ComponentAction::Render,
            reference: draft_reference,
            kind: ComponentRenderKind::Table,
            resource_authority_revision: 1,
            query: String::new(),
            dashboard_scope_node_ids: vec![hidden_scope],
        })
        .expect("draft Component render request"),
    )
    .await;
    let absent_draft_render = private_provider_request(
        &app,
        &core_signer,
        &dashboard_signer,
        &grants,
        dashboard_instance_id,
        hidden_scope,
        "/api/private/components/render",
        serde_json::to_value(ComponentRenderRequest {
            schema_version: COMPONENT_CONTRACT_SCHEMA_VERSION,
            action: ComponentAction::Render,
            reference: absent_draft_reference,
            kind: ComponentRenderKind::Table,
            resource_authority_revision: 1,
            query: String::new(),
            dashboard_scope_node_ids: vec![hidden_scope],
        })
        .expect("absent Component render request"),
    )
    .await;
    assert_eq!(draft_render.status, StatusCode::FORBIDDEN);
    assert_eq!(draft_render.status, absent_draft_render.status);
    assert_eq!(draft_render.body, absent_draft_render.body);

    sqlx::query("DELETE FROM component_versions WHERE id=$1")
        .bind(mixed_scope_draft_id)
        .execute(&pool)
        .await
        .expect("remove mixed-scope draft fixture");

    let first_render = render_current(
        &app,
        &core_signer,
        &grants,
        allowed_scope,
        "/api/components/revenue-table/table?search=north",
    )
    .await;
    assert_eq!(first_render.status, StatusCode::OK, "{}", first_render.body);
    assert_eq!(
        uuid_at(&first_render.body, "/component_version_id"),
        first_version_id
    );
    assert_eq!(first_render.body["materialization_state"], "ready");
    assert_eq!(first_render.body["columns"][1]["label"], "Total");
    assert_eq!(first_render.body["rows"][0]["values"]["amount"], "42");

    let unsigned_known = unsigned_get(&app, "/api/components/revenue-table/table").await;
    let unsigned_missing =
        unsigned_get(&app, &format!("/api/components/{}/table", Uuid::new_v4())).await;
    assert_eq!(unsigned_known.status, StatusCode::FORBIDDEN);
    assert_eq!(unsigned_known.status, unsigned_missing.status);
    assert_eq!(unsigned_known.body, unsigned_missing.body);

    let hidden_known_execute = render_current(
        &app,
        &core_signer,
        &grants,
        hidden_scope,
        "/api/components/revenue-table/table",
    )
    .await;
    let hidden_missing_execute = render_current(
        &app,
        &core_signer,
        &grants,
        hidden_scope,
        &format!("/api/components/{}/table", Uuid::new_v4()),
    )
    .await;
    assert_eq!(hidden_known_execute.status, StatusCode::NOT_FOUND);
    assert_eq!(hidden_known_execute.status, hidden_missing_execute.status);
    assert_eq!(hidden_known_execute.body, hidden_missing_execute.body);

    let known_reference =
        component_version_reference(installation_id, module_instance_id, first_version_id);
    let random_reference =
        component_version_reference(installation_id, module_instance_id, Uuid::new_v4());
    let hidden_known_resolution = private_provider_request(
        &app,
        &core_signer,
        &dashboard_signer,
        &grants,
        dashboard_instance_id,
        hidden_scope,
        "/api/private/components/resolve",
        serde_json::to_value(ComponentResolutionRequest::new(
            ComponentAction::ResolveMetadata,
            known_reference.clone(),
            None,
        ))
        .expect("known Component resolution request"),
    )
    .await;
    let hidden_random_resolution = private_provider_request(
        &app,
        &core_signer,
        &dashboard_signer,
        &grants,
        dashboard_instance_id,
        hidden_scope,
        "/api/private/components/resolve",
        serde_json::to_value(ComponentResolutionRequest::new(
            ComponentAction::ResolveMetadata,
            random_reference,
            None,
        ))
        .expect("random Component resolution request"),
    )
    .await;
    assert_eq!(hidden_known_resolution.status, StatusCode::OK);
    assert_eq!(
        hidden_known_resolution.status,
        hidden_random_resolution.status
    );
    assert_eq!(hidden_known_resolution.body, hidden_random_resolution.body);
    assert_eq!(
        hidden_known_resolution.body["resolution"]["access_state"],
        "unauthorized"
    );
    assert!(hidden_known_resolution.body["metadata"].is_null());

    let replay_body = serde_json::to_value(ComponentResolutionRequest::new(
        ComponentAction::ResolveMetadata,
        known_reference.clone(),
        None,
    ))
    .expect("replay Component resolution request");
    let replay_message = private_provider_message(
        &core_signer,
        &dashboard_signer,
        &grants,
        dashboard_instance_id,
        dashboard_instance_id,
        allowed_scope,
        "/api/private/components/resolve",
        replay_body.clone(),
    );
    let replay_first =
        send_private_provider_message(&app, "/api/private/components/resolve", &replay_message)
            .await;
    assert_eq!(replay_first.status, StatusCode::OK, "{}", replay_first.body);
    let replay_second =
        send_private_provider_message(&app, "/api/private/components/resolve", &replay_message)
            .await;
    assert_eq!(replay_second.status, StatusCode::FORBIDDEN);

    let wrong_instance = private_provider_message(
        &core_signer,
        &dashboard_signer,
        &grants,
        dashboard_instance_id,
        Uuid::new_v4(),
        allowed_scope,
        "/api/private/components/resolve",
        replay_body.clone(),
    );
    assert_eq!(
        send_private_provider_message(&app, "/api/private/components/resolve", &wrong_instance,)
            .await
            .status,
        StatusCode::FORBIDDEN,
        "the service-request instance must exactly match the Core-signed presenting principal"
    );

    let unregistered_key = signing_key(
        "tessara.dashboards",
        "dashboard-service-test",
        ProtocolSignaturePurposeV1::ModuleServiceRequest,
        85,
    );
    let wrong_signature = private_provider_message(
        &core_signer,
        &unregistered_key,
        &grants,
        dashboard_instance_id,
        dashboard_instance_id,
        allowed_scope,
        "/api/private/components/resolve",
        replay_body,
    );
    assert_eq!(
        send_private_provider_message(&app, "/api/private/components/resolve", &wrong_signature,)
            .await
            .status,
        StatusCode::FORBIDDEN,
        "a caller key absent from the materialization-projected identity must fail closed"
    );

    let legacy_message = legacy_private_provider_message(
        &core_signer,
        &dashboard_signer,
        &grants,
        dashboard_instance_id,
        allowed_scope,
        "/api/private/components/resolve",
        serde_json::to_value(ComponentResolutionRequest::new(
            ComponentAction::ResolveMetadata,
            known_reference.clone(),
            None,
        ))
        .expect("historical Component resolution request"),
    );
    assert_eq!(
        send_private_provider_message(&app, "/api/private/components/resolve", &legacy_message,)
            .await
            .status,
        StatusCode::FORBIDDEN,
        "the normal Component provider must reject historical authorization grants"
    );

    let authority_revision = public_detail.body["versions"][0]["authority_revision"]
        .as_u64()
        .expect("published Component authority revision");
    let random_render = private_provider_request(
        &app,
        &core_signer,
        &dashboard_signer,
        &grants,
        dashboard_instance_id,
        allowed_scope,
        "/api/private/components/render",
        serde_json::to_value(ComponentRenderRequest {
            schema_version: COMPONENT_CONTRACT_SCHEMA_VERSION,
            action: ComponentAction::Render,
            reference: component_version_reference(
                installation_id,
                module_instance_id,
                Uuid::new_v4(),
            ),
            kind: ComponentRenderKind::Table,
            resource_authority_revision: authority_revision,
            query: String::new(),
            dashboard_scope_node_ids: vec![allowed_scope],
        })
        .expect("random Component render request"),
    )
    .await;
    assert_eq!(random_render.status, StatusCode::FORBIDDEN);
    let hidden_known_render = private_provider_request(
        &app,
        &core_signer,
        &dashboard_signer,
        &grants,
        dashboard_instance_id,
        hidden_scope,
        "/api/private/components/render",
        serde_json::to_value(ComponentRenderRequest {
            schema_version: COMPONENT_CONTRACT_SCHEMA_VERSION,
            action: ComponentAction::Render,
            reference: known_reference.clone(),
            kind: ComponentRenderKind::Table,
            resource_authority_revision: authority_revision,
            query: String::new(),
            dashboard_scope_node_ids: vec![hidden_scope],
        })
        .expect("known hidden Component render request"),
    )
    .await;
    assert_eq!(hidden_known_render.status, random_render.status);
    assert_eq!(hidden_known_render.body, random_render.body);
    for invalid_reference in [
        component_version_reference(installation_id, Uuid::new_v4(), first_version_id),
        component_version_reference(Uuid::new_v4(), Uuid::new_v4(), first_version_id),
    ] {
        let invalid_resolution = private_provider_request(
            &app,
            &core_signer,
            &dashboard_signer,
            &grants,
            dashboard_instance_id,
            allowed_scope,
            "/api/private/components/resolve",
            serde_json::to_value(ComponentResolutionRequest::new(
                ComponentAction::ResolveMetadata,
                invalid_reference.clone(),
                None,
            ))
            .expect("wrong-owner Component resolution request"),
        )
        .await;
        assert_eq!(invalid_resolution.status, StatusCode::OK);
        assert_eq!(
            invalid_resolution.body["resolution"]["access_state"],
            "not_evaluated"
        );
        assert!(invalid_resolution.body["metadata"].is_null());

        let invalid_render = private_provider_request(
            &app,
            &core_signer,
            &dashboard_signer,
            &grants,
            dashboard_instance_id,
            allowed_scope,
            "/api/private/components/render",
            serde_json::to_value(ComponentRenderRequest {
                schema_version: COMPONENT_CONTRACT_SCHEMA_VERSION,
                action: ComponentAction::Render,
                reference: invalid_reference,
                kind: ComponentRenderKind::Table,
                resource_authority_revision: authority_revision,
                query: String::new(),
                dashboard_scope_node_ids: vec![allowed_scope],
            })
            .expect("wrong-owner Component render request"),
        )
        .await;
        assert_eq!(invalid_render.status, StatusCode::FORBIDDEN);
        assert_eq!(invalid_render.status, random_render.status);
        assert_eq!(invalid_render.body, random_render.body);
    }

    let hidden_existing = module_request(
        &app,
        &core_signer,
        &grants,
        GrantSpec::read("components.get", READ_CAPABILITY, hidden_scope),
        "GET",
        &format!("/api/components/{component_id}"),
        None,
        Value::Null,
    )
    .await;
    let hidden_missing = module_request(
        &app,
        &core_signer,
        &grants,
        GrantSpec::read("components.get", READ_CAPABILITY, hidden_scope),
        "GET",
        &format!("/api/components/{}", Uuid::new_v4()),
        None,
        Value::Null,
    )
    .await;
    assert_eq!(hidden_existing.status, StatusCode::NOT_FOUND);
    assert_eq!(hidden_existing.status, hidden_missing.status);
    assert_eq!(hidden_existing.body, hidden_missing.body);

    let second = module_request(
        &app,
        &core_signer,
        &grants,
        GrantSpec::mutation("components.save_version", MANAGE_CAPABILITY, allowed_scope),
        "POST",
        &format!("/api/admin/components/{component_id}/versions"),
        Some("component-version-create-2"),
        json!({
            "schema_version": 1,
            "version": {
                "dataset_reference": dataset_reference,
                "component_type": "table",
                "config": table_config,
                "version_note": "Second draft"
            }
        }),
    )
    .await;
    assert_eq!(second.status, StatusCode::OK, "{}", second.body);
    let second_version_id = uuid_at(&second.body, "/component_version_id");

    let revised_config = json!({
        "visible_columns": ["label", "amount"],
        "search_fields": ["label"],
        "page_size": 10,
        "display_labels": {"amount": "Revised total"}
    });
    let revised = module_request(
        &app,
        &core_signer,
        &grants,
        GrantSpec::mutation("components.save_version", MANAGE_CAPABILITY, allowed_scope),
        "PUT",
        &format!("/api/admin/components/{component_id}/versions/{second_version_id}"),
        Some("component-version-update-2"),
        json!({
            "schema_version": 1,
            "version": {
                "dataset_reference": dataset_reference,
                "component_type": "table",
                "config": revised_config,
                "version_note": "Second draft revised"
            }
        }),
    )
    .await;
    assert_eq!(revised.status, StatusCode::OK, "{}", revised.body);

    publish(
        &app,
        &core_signer,
        &grants,
        component_id,
        second_version_id,
        allowed_scope,
        "component-publish-2",
    )
    .await;
    let current = render_current(
        &app,
        &core_signer,
        &grants,
        allowed_scope,
        "/api/components/revenue-table/table",
    )
    .await;
    assert_eq!(
        uuid_at(&current.body, "/component_version_id"),
        second_version_id
    );

    let historical = render_current(
        &app,
        &core_signer,
        &grants,
        allowed_scope,
        &format!("/api/components/revenue-table/versions/{first_version_id}/table"),
    )
    .await;
    assert_eq!(historical.status, StatusCode::OK, "{}", historical.body);
    assert_eq!(
        uuid_at(&historical.body, "/component_version_id"),
        first_version_id
    );

    let detail = manageable_detail(&app, &core_signer, &grants, allowed_scope, component_id).await;
    let second_revision = detail["versions"]
        .as_array()
        .expect("versions")
        .iter()
        .find(|version| version["component_version_id"] == second_version_id.to_string())
        .and_then(|version| version["resource_revision"].as_u64())
        .expect("second version resource revision");
    let deactivated = lifecycle(
        &app,
        &core_signer,
        &grants,
        component_id,
        second_version_id,
        allowed_scope,
        "deactivate",
        second_revision,
        "component-deactivate-2",
    )
    .await;
    assert_eq!(deactivated.status, StatusCode::OK, "{}", deactivated.body);
    assert_eq!(deactivated.body["outcome"], "lifecycle_inactive");
    let inactive_render = render_current(
        &app,
        &core_signer,
        &grants,
        allowed_scope,
        "/api/components/revenue-table/table",
    )
    .await;
    assert_eq!(inactive_render.status, StatusCode::NOT_FOUND);

    let inactive_authority_revision = sqlx::query_scalar::<_, i64>(
        "SELECT authority_revision FROM component_versions WHERE id=$1",
    )
    .bind(second_version_id)
    .fetch_one(&pool)
    .await
    .expect("inactive Component authority revision") as u64;
    let inactive_provider_render = private_provider_request(
        &app,
        &core_signer,
        &dashboard_signer,
        &grants,
        dashboard_instance_id,
        allowed_scope,
        "/api/private/components/render",
        serde_json::to_value(ComponentRenderRequest {
            schema_version: COMPONENT_CONTRACT_SCHEMA_VERSION,
            action: ComponentAction::Render,
            reference: component_version_reference(
                installation_id,
                module_instance_id,
                second_version_id,
            ),
            kind: ComponentRenderKind::Table,
            resource_authority_revision: inactive_authority_revision,
            query: String::new(),
            dashboard_scope_node_ids: vec![allowed_scope],
        })
        .expect("inactive published Component render request"),
    )
    .await;
    assert_eq!(inactive_provider_render.status, StatusCode::FORBIDDEN);

    let reactivated = lifecycle(
        &app,
        &core_signer,
        &grants,
        component_id,
        second_version_id,
        allowed_scope,
        "activate",
        second_revision + 1,
        "component-activate-2",
    )
    .await;
    assert_eq!(reactivated.status, StatusCode::OK, "{}", reactivated.body);
    let active_render = render_current(
        &app,
        &core_signer,
        &grants,
        allowed_scope,
        "/api/components/revenue-table/table",
    )
    .await;
    assert_eq!(
        active_render.status,
        StatusCode::OK,
        "{}",
        active_render.body
    );
    assert_eq!(
        uuid_at(&active_render.body, "/component_version_id"),
        second_version_id
    );

    let canonical_save_body = json!({
        "schema_version": 1,
        "component_id": component_id,
        "draft_version_id": null,
        "published_version_id": second_version_id,
        "action": "update_existing_version",
        "component": {
            "schema_version": 1,
            "name": "Revenue table atomically saved",
            "slug": "revenue-table",
            "description": "Metadata and the current version share one transaction."
        },
        "version": {
            "dataset_reference": dataset_reference,
            "component_type": "table",
            "config": revised_config,
            "version_note": "Atomic current-version update"
        }
    });
    let canonical_save = module_request(
        &app,
        &core_signer,
        &grants,
        GrantSpec::mutation("components.save", MANAGE_CAPABILITY, allowed_scope),
        "POST",
        "/api/admin/components/save",
        Some("component-atomic-save"),
        canonical_save_body.clone(),
    )
    .await;
    assert_eq!(
        canonical_save.status,
        StatusCode::OK,
        "{}",
        canonical_save.body
    );
    assert_eq!(canonical_save.body["outcome"], "published_version_updated");
    assert_eq!(
        sqlx::query_scalar::<_, String>("SELECT name FROM components WHERE id=$1")
            .bind(component_id)
            .fetch_one(&pool)
            .await
            .expect("atomically saved Component metadata"),
        "Revenue table atomically saved"
    );

    assert_eq!(dataset.schema_calls.load(Ordering::Relaxed), 8);
    assert!(
        (1..=dataset.schema_calls.load(Ordering::Relaxed))
            .contains(&dataset.compatibility_calls.load(Ordering::Relaxed))
    );
    assert_eq!(dataset.execute_calls.load(Ordering::Relaxed), 4);
    dataset_server.abort();
    let _ = dataset_server.await;

    // Simulate a committed response being lost: an exact retry must replay
    // before any Dataset call and therefore still succeed during the outage.
    let replay = module_request(
        &app,
        &core_signer,
        &grants,
        GrantSpec::mutation("components.save", MANAGE_CAPABILITY, allowed_scope),
        "POST",
        "/api/admin/components/save",
        Some("component-atomic-save"),
        canonical_save_body,
    )
    .await;
    assert_eq!(replay.status, StatusCode::OK, "{}", replay.body);
    assert_eq!(replay.body, canonical_save.body);

    // A new mutation identity cannot reach provider validation, and neither
    // shell metadata nor the version payload may be partially changed.
    let failed_save = module_request(
        &app,
        &core_signer,
        &grants,
        GrantSpec::mutation("components.save", MANAGE_CAPABILITY, allowed_scope),
        "POST",
        "/api/admin/components/save",
        Some("component-atomic-save-provider-outage"),
        json!({
            "schema_version": 1,
            "component_id": component_id,
            "draft_version_id": null,
            "published_version_id": second_version_id,
            "action": "update_existing_version",
            "component": {
                "schema_version": 1,
                "name": "Must not partially persist",
                "slug": "must-not-partially-persist",
                "description": null
            },
            "version": {
                "dataset_reference": dataset.metadata.reference.clone(),
                "component_type": "table",
                "config": {"visible_columns": ["label"]},
                "version_note": "Must not persist"
            }
        }),
    )
    .await;
    assert_eq!(failed_save.status, StatusCode::SERVICE_UNAVAILABLE);
    let persisted = sqlx::query(
        "SELECT c.name,c.slug,v.version_note,v.config
         FROM components c JOIN component_versions v ON v.component_id=c.id
         WHERE c.id=$1 AND v.id=$2",
    )
    .bind(component_id)
    .bind(second_version_id)
    .fetch_one(&pool)
    .await
    .expect("Component remains readable after provider outage");
    assert_eq!(
        persisted.try_get::<String, _>("name").unwrap(),
        "Revenue table atomically saved"
    );
    assert_eq!(
        persisted.try_get::<String, _>("slug").unwrap(),
        "revenue-table"
    );
    assert_eq!(
        persisted.try_get::<String, _>("version_note").unwrap(),
        "Atomic current-version update"
    );
    assert_eq!(
        persisted.try_get::<Value, _>("config").unwrap(),
        revised_config
    );

    reset_component_product(&pool).await;
    pool.close().await;
}

#[derive(Clone)]
struct DatasetStub {
    metadata: DatasetMajorLineMetadata,
    execution: DatasetExecutionResponse,
    core_authorization_signer: Arc<PurposeBoundSigningKeyV1>,
    schema_calls: Arc<AtomicUsize>,
    compatibility_calls: Arc<AtomicUsize>,
    execute_calls: Arc<AtomicUsize>,
}

impl DatasetStub {
    fn new(
        reference: DatasetMajorLineReference,
        scope: Uuid,
        core_authorization_signer: Arc<PurposeBoundSigningKeyV1>,
    ) -> Self {
        let fields = vec![
            DatasetFieldContract {
                key: "label".into(),
                label: "Label".into(),
                field_type: "text".into(),
                restriction_tier: "public".into(),
            },
            DatasetFieldContract {
                key: "amount".into(),
                label: "Amount".into(),
                field_type: "number".into(),
                restriction_tier: "public".into(),
            },
        ];
        let values = BTreeMap::from([
            ("label".into(), Some(json!("North"))),
            ("amount".into(), Some(json!(42))),
        ]);
        Self {
            metadata: DatasetMajorLineMetadata {
                reference,
                dataset_name: "Revenue".into(),
                dataset_slug: "revenue".into(),
                grain: "submission".into(),
                tags: vec!["finance".into(), "revenue".into()],
                provenance: tessara_datasets_contract::DatasetProvenanceSummary {
                    forms: vec![tessara_datasets_contract::DatasetProvenanceItem {
                        id: Uuid::from_u128(0xd001),
                        name: "Revenue intake".into(),
                        slug: None,
                    }],
                    datasets: Vec::new(),
                },
                materialization_state: "ready".into(),
                fields: fields.clone(),
                scope_node_ids: vec![scope],
            },
            execution: DatasetExecutionResponse {
                schema_version: DATASET_CONTRACT_SCHEMA_VERSION,
                materialization_state: "ready".into(),
                fields,
                rows: vec![DatasetExecutionRow {
                    row_id: "north".into(),
                    values,
                }],
                next_cursor: None,
            },
            core_authorization_signer,
            schema_calls: Arc::new(AtomicUsize::new(0)),
            compatibility_calls: Arc::new(AtomicUsize::new(0)),
            execute_calls: Arc::new(AtomicUsize::new(0)),
        }
    }
}

async fn dataset_schema(
    State(state): State<DatasetStub>,
    headers: HeaderMap,
    Json(request): Json<DatasetSchemaRequest>,
) -> Result<Json<DatasetMajorLineMetadata>, StatusCode> {
    require_forwarded_contract_headers(&headers)?;
    if request.schema_version != DATASET_CONTRACT_SCHEMA_VERSION
        || request.action != DatasetAction::ResolveSchema
        || request.reference != state.metadata.reference
    {
        return Err(StatusCode::BAD_REQUEST);
    }
    state.schema_calls.fetch_add(1, Ordering::Relaxed);
    Ok(Json(state.metadata))
}

async fn dataset_execute(
    State(state): State<DatasetStub>,
    headers: HeaderMap,
    Json(request): Json<DatasetExecutionRequest>,
) -> Result<Json<DatasetExecutionResponse>, StatusCode> {
    require_forwarded_contract_headers(&headers)?;
    if request.schema_version != DATASET_CONTRACT_SCHEMA_VERSION
        || request.action != DatasetAction::Execute
        || request.reference != state.metadata.reference
    {
        return Err(StatusCode::BAD_REQUEST);
    }
    state.execute_calls.fetch_add(1, Ordering::Relaxed);
    Ok(Json(state.execution))
}

async fn dataset_compatibility(
    State(state): State<DatasetStub>,
    headers: HeaderMap,
    Json(request): Json<DatasetCompatibilityRequest>,
) -> Result<Json<DatasetCompatibilityResponse>, StatusCode> {
    require_forwarded_contract_headers(&headers)?;
    if request.schema_version != DATASET_CONTRACT_SCHEMA_VERSION
        || request.action != DatasetAction::CheckCompatibility
        || request.reference != state.metadata.reference
    {
        return Err(StatusCode::BAD_REQUEST);
    }
    let fields = state
        .metadata
        .fields
        .iter()
        .map(|field| (field.key.as_str(), field.field_type.as_str()))
        .collect::<BTreeMap<_, _>>();
    let findings = request
        .required_fields
        .into_iter()
        .filter_map(
            |requirement| match fields.get(requirement.field_key.as_str()) {
                None => Some(DatasetCompatibilityFinding {
                    code: "field_missing".into(),
                    field_key: requirement.field_key,
                }),
                Some(actual)
                    if !requirement
                        .accepted_types
                        .iter()
                        .any(|field_type| field_type == actual) =>
                {
                    Some(DatasetCompatibilityFinding {
                        code: "field_type_incompatible".into(),
                        field_key: requirement.field_key,
                    })
                }
                Some(_) => None,
            },
        )
        .collect::<Vec<_>>();
    state.compatibility_calls.fetch_add(1, Ordering::Relaxed);
    Ok(Json(DatasetCompatibilityResponse {
        schema_version: DATASET_CONTRACT_SCHEMA_VERSION,
        compatible: findings.is_empty(),
        findings,
    }))
}

async fn core_authorization_exchange(
    State(state): State<DatasetStub>,
    headers: HeaderMap,
    Json(request): Json<AuthorizationExchangeRequestV2>,
) -> Result<Json<AuthorizationExchangeResponseV2>, StatusCode> {
    require_forwarded_contract_headers(&headers)?;
    request.validate().map_err(|_| StatusCode::BAD_REQUEST)?;
    let encoded = headers
        .get("x-tessara-authorization")
        .and_then(|value| value.to_str().ok())
        .ok_or(StatusCode::FORBIDDEN)?;
    let inbound: SignedEnvelopeV1<AuthorizationGrantV3> = serde_json::from_slice(
        &URL_SAFE_NO_PAD
            .decode(encoded)
            .map_err(|_| StatusCode::FORBIDDEN)?,
    )
    .map_err(|_| StatusCode::FORBIDDEN)?;
    let capability_scope_bindings = inbound
        .payload
        .capability_scope_bindings
        .iter()
        .map(|binding| CapabilityScopeBindingV1 {
            capability: SecurityCapabilityId::new("datasets:read")
                .expect("Dataset read capability"),
            organization_root_id: binding.organization_root_id,
            authorized_organization_ids: binding.authorized_organization_ids.clone(),
        })
        .collect();
    let now = Utc::now();
    let authorization = state
        .core_authorization_signer
        .sign(AuthorizationGrantV3 {
            schema_version: AUTHORIZATION_GRANT_SCHEMA_VERSION_V3,
            installation_id: inbound.payload.installation_id,
            original_actor_id: inbound.payload.original_actor_id,
            correlation_id: inbound.payload.correlation_id,
            presenting_service: ModuleServicePrincipalV1::ModuleInstance {
                module_instance_id: match &inbound.payload.audience {
                    AuthorizationAudienceV1::ModuleInstance {
                        module_instance_id, ..
                    } => *module_instance_id,
                    AuthorizationAudienceV1::CoreInstallation { .. } => {
                        return Err(StatusCode::FORBIDDEN);
                    }
                },
                module_definition_id: ModuleDefinitionId::new("tessara.components")
                    .expect("Component identity"),
            },
            audience: request.target,
            dependency_binding: request.dependency_binding,
            functional_contract: request.functional_contract,
            action: request.action,
            operation: AuthorizationGrantOperationV1::Read,
            capability_scope_bindings,
            resource_assertion: request.resource_assertion,
            delegation_basis: inbound.payload.delegation_basis,
            authorization_revision: inbound.payload.authorization_revision,
            organization_revision: inbound.payload.organization_revision,
            jti: Uuid::new_v4(),
            issued_at: now,
            expires_at: now + Duration::seconds(30),
        })
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(AuthorizationExchangeResponseV2 {
        schema_version: AUTHORIZATION_EXCHANGE_SCHEMA_VERSION_V2,
        authorization,
    }))
}

fn require_forwarded_contract_headers(headers: &HeaderMap) -> Result<(), StatusCode> {
    for name in [
        "x-tessara-authorization",
        "x-tessara-module-service-request",
        "x-tessara-correlation-id",
    ] {
        if headers
            .get(name)
            .and_then(|value| value.to_str().ok())
            .is_none_or(str::is_empty)
        {
            return Err(StatusCode::FORBIDDEN);
        }
    }
    Ok(())
}

struct GrantContext {
    installation_id: Uuid,
    module_instance_id: Uuid,
    actor_id: Uuid,
    correlation_id: Uuid,
}

struct GrantSpec<'a> {
    action: &'a str,
    operation: AuthorizationGrantOperationV1,
    capability: &'a str,
    scope: Uuid,
    additional_scopes: Vec<Uuid>,
}

impl<'a> GrantSpec<'a> {
    fn read(action: &'a str, capability: &'a str, scope: Uuid) -> Self {
        Self {
            action,
            operation: AuthorizationGrantOperationV1::Read,
            capability,
            scope,
            additional_scopes: Vec::new(),
        }
    }

    fn mutation(action: &'a str, capability: &'a str, scope: Uuid) -> Self {
        Self {
            action,
            operation: AuthorizationGrantOperationV1::Mutation,
            capability,
            scope,
            additional_scopes: Vec::new(),
        }
    }

    fn with_additional_scope(mut self, scope: Uuid) -> Self {
        self.additional_scopes.push(scope);
        self
    }
}

#[allow(clippy::too_many_arguments)]
async fn module_request(
    app: &Router,
    signer: &PurposeBoundSigningKeyV1,
    context: &GrantContext,
    grant: GrantSpec<'_>,
    method: &str,
    path: &str,
    idempotency_key: Option<&str>,
    body: Value,
) -> TestResponse {
    let authorization = signed_grant(signer, context, &grant);
    let mut request = Request::builder()
        .method(method)
        .uri(path)
        .header("content-type", "application/json")
        .header("x-tessara-authorization", authorization)
        .header(
            "x-tessara-correlation-id",
            context.correlation_id.to_string(),
        );
    if let Some(idempotency_key) = idempotency_key {
        request = request.header("x-idempotency-key", idempotency_key);
    }
    test_response(
        app,
        request
            .body(Body::from(if method == "GET" {
                Vec::new()
            } else {
                serde_json::to_vec(&body).expect("serialize request body")
            }))
            .expect("module request"),
    )
    .await
}

async fn unsigned_get(app: &Router, path: &str) -> TestResponse {
    test_response(
        app,
        Request::builder()
            .method("GET")
            .uri(path)
            .body(Body::empty())
            .expect("unsigned Component request"),
    )
    .await
}

async fn control_request(app: &Router, path: &str, body: Value) -> TestResponse {
    let control_key = std::env::var("TESSARA_MODULE_CONTROL_SHARED_KEY")
        .unwrap_or_else(|_| "development-module-control-only".into());
    test_response(
        app,
        Request::builder()
            .method("POST")
            .uri(path)
            .header("content-type", "application/json")
            .header("x-tessara-module-control-key", control_key)
            .body(Body::from(
                serde_json::to_vec(&body).expect("serialize control request"),
            ))
            .expect("Component control request"),
    )
    .await
}

#[allow(clippy::too_many_arguments)]
async fn private_provider_request(
    app: &Router,
    core_signer: &PurposeBoundSigningKeyV1,
    dashboard_signer: &PurposeBoundSigningKeyV1,
    context: &GrantContext,
    dashboard_instance_id: Uuid,
    scope: Uuid,
    path: &str,
    body: Value,
) -> TestResponse {
    let message = private_provider_message(
        core_signer,
        dashboard_signer,
        context,
        dashboard_instance_id,
        dashboard_instance_id,
        scope,
        path,
        body,
    );
    send_private_provider_message(app, path, &message).await
}

struct PrivateProviderMessage {
    authorization: String,
    service_request: String,
    correlation_id: Uuid,
    body: Vec<u8>,
}

#[allow(clippy::too_many_arguments)]
fn private_provider_message(
    core_signer: &PurposeBoundSigningKeyV1,
    service_signer: &PurposeBoundSigningKeyV1,
    context: &GrantContext,
    presenting_module_instance_id: Uuid,
    service_module_instance_id: Uuid,
    scope: Uuid,
    path: &str,
    body: Value,
) -> PrivateProviderMessage {
    let now = Utc::now();
    let action = match path {
        "/api/private/components/resolve" => "components.resolve",
        "/api/private/components/catalog" => "components.catalog",
        "/api/private/components/render" => "components.render",
        _ => panic!("unexpected Component provider path {path}"),
    };
    let inbound = core_signer
        .sign(AuthorizationGrantV3 {
            schema_version: AUTHORIZATION_GRANT_SCHEMA_VERSION_V3,
            installation_id: context.installation_id,
            original_actor_id: context.actor_id,
            correlation_id: context.correlation_id,
            presenting_service: ModuleServicePrincipalV1::ModuleInstance {
                module_instance_id: presenting_module_instance_id,
                module_definition_id: ModuleDefinitionId::new("tessara.dashboards")
                    .expect("Dashboard identity"),
            },
            audience: AuthorizationAudienceV1::ModuleInstance {
                module_instance_id: context.module_instance_id,
                module_definition_id: ModuleDefinitionId::new("tessara.components")
                    .expect("Component identity"),
            },
            dependency_binding: DependencyBindingKey::new("tessara.dashboards.component-version")
                .expect("Component dependency binding"),
            functional_contract: FunctionalContractId::new("tessara.components.component-version")
                .expect("Component contract"),
            action: action.into(),
            operation: AuthorizationGrantOperationV1::Read,
            capability_scope_bindings: vec![CapabilityScopeBindingV1 {
                capability: SecurityCapabilityId::new("components:read")
                    .expect("Component read capability"),
                organization_root_id: scope,
                authorized_organization_ids: Vec::new(),
            }],
            resource_assertion: None,
            delegation_basis: Vec::new(),
            authorization_revision: 7,
            organization_revision: 11,
            jti: Uuid::new_v4(),
            issued_at: now,
            expires_at: now + Duration::seconds(60),
        })
        .expect("sign Dashboard authorization grant");
    let authorization = URL_SAFE_NO_PAD
        .encode(serde_json::to_vec(&inbound).expect("serialize Dashboard authorization grant"));
    let body = serde_json::to_vec(&body).expect("serialize private provider request");
    let service = service_signer
        .sign(ModuleServiceRequestV1 {
            schema_version: 1,
            installation_id: context.installation_id,
            module_instance_id: service_module_instance_id,
            module_definition_id: ModuleDefinitionId::new("tessara.dashboards")
                .expect("Dashboard identity"),
            method: "POST".into(),
            path: path.into(),
            canonical_body_digest: sha256_hex(&body),
            inbound_grant_digest: sha256_hex(authorization.as_bytes()),
            correlation_id: context.correlation_id.to_string(),
            nonce: Uuid::new_v4(),
            issued_at: now,
            expires_at: now + Duration::seconds(30),
        })
        .expect("sign Dashboard service request");
    let service = URL_SAFE_NO_PAD
        .encode(serde_json::to_vec(&service).expect("serialize Dashboard service request"));
    PrivateProviderMessage {
        authorization,
        service_request: service,
        correlation_id: context.correlation_id,
        body,
    }
}

#[allow(clippy::too_many_arguments)]
fn legacy_private_provider_message(
    core_signer: &PurposeBoundSigningKeyV1,
    dashboard_signer: &PurposeBoundSigningKeyV1,
    context: &GrantContext,
    dashboard_instance_id: Uuid,
    scope: Uuid,
    path: &str,
    body: Value,
) -> PrivateProviderMessage {
    let now = Utc::now();
    let legacy = core_signer
        .sign(AuthorizationGrantV2 {
            schema_version: AUTHORIZATION_GRANT_SCHEMA_VERSION_V2,
            installation_id: context.installation_id,
            original_actor_id: context.actor_id,
            presenting_service: ModuleDefinitionId::new("tessara.core").expect("Core identity"),
            audience_module_instance_id: dashboard_instance_id,
            dependency_binding: DependencyBindingKey::new("tessara.core.dashboards")
                .expect("legacy Dashboard binding"),
            functional_contract: FunctionalContractId::new("tessara.dashboards.dashboard")
                .expect("legacy Dashboard contract"),
            action: "dashboards.render_placement".into(),
            operation: AuthorizationGrantOperationV1::Read,
            capability_scope_bindings: vec![CapabilityScopeBindingV1 {
                capability: SecurityCapabilityId::new("dashboards:read")
                    .expect("legacy Dashboard capability"),
                organization_root_id: scope,
                authorized_organization_ids: Vec::new(),
            }],
            resource_assertion: None,
            delegation_basis: Vec::new(),
            authorization_revision: 7,
            organization_revision: 11,
            jti: Uuid::new_v4(),
            issued_at: now,
            expires_at: now + Duration::seconds(60),
        })
        .expect("sign historical Dashboard grant");
    let authorization = URL_SAFE_NO_PAD
        .encode(serde_json::to_vec(&legacy).expect("serialize historical Dashboard grant"));
    let body = serde_json::to_vec(&body).expect("serialize private provider request");
    let service = dashboard_signer
        .sign(ModuleServiceRequestV1 {
            schema_version: 1,
            installation_id: context.installation_id,
            module_instance_id: dashboard_instance_id,
            module_definition_id: ModuleDefinitionId::new("tessara.dashboards")
                .expect("Dashboard identity"),
            method: "POST".into(),
            path: path.into(),
            canonical_body_digest: sha256_hex(&body),
            inbound_grant_digest: sha256_hex(authorization.as_bytes()),
            correlation_id: context.correlation_id.to_string(),
            nonce: Uuid::new_v4(),
            issued_at: now,
            expires_at: now + Duration::seconds(30),
        })
        .expect("sign historical Dashboard service request");
    PrivateProviderMessage {
        authorization,
        service_request: URL_SAFE_NO_PAD
            .encode(serde_json::to_vec(&service).expect("serialize service request")),
        correlation_id: context.correlation_id,
        body,
    }
}

async fn send_private_provider_message(
    app: &Router,
    path: &str,
    message: &PrivateProviderMessage,
) -> TestResponse {
    test_response(
        app,
        Request::builder()
            .method("POST")
            .uri(path)
            .header("content-type", "application/json")
            .header("x-tessara-authorization", &message.authorization)
            .header("x-tessara-module-service-request", &message.service_request)
            .header(
                "x-tessara-correlation-id",
                message.correlation_id.to_string(),
            )
            .body(Body::from(message.body.clone()))
            .expect("private Component provider request"),
    )
    .await
}

async fn test_response(app: &Router, request: Request<Body>) -> TestResponse {
    let response = app
        .clone()
        .oneshot(request)
        .await
        .expect("Component router response");
    let status = response.status();
    let bytes = to_bytes(response.into_body(), 1024 * 1024)
        .await
        .expect("read Component response");
    let body = serde_json::from_slice(&bytes).unwrap_or_else(|error| {
        panic!(
            "Component response is not JSON ({error}): {}",
            String::from_utf8_lossy(&bytes)
        )
    });
    TestResponse { status, body }
}

fn component_version_reference(
    installation_id: Uuid,
    module_instance_id: Uuid,
    version_id: Uuid,
) -> ComponentVersionReference {
    ComponentVersionReference::new(
        TypedResourceReference::new(
            installation_id,
            ResourceOwner::ModuleInstance {
                installation_id,
                module_instance_id,
            },
            COMPONENT_RESOURCE_TYPE
                .parse()
                .expect("Component resource type"),
            version_id.to_string(),
        )
        .expect("typed Component reference"),
    )
    .expect("Component version reference")
}

fn sha256_hex(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

struct TestResponse {
    status: StatusCode,
    body: Value,
}

fn signed_grant(
    signer: &PurposeBoundSigningKeyV1,
    context: &GrantContext,
    spec: &GrantSpec<'_>,
) -> String {
    let now = Utc::now();
    let contract = if matches!(
        spec.action,
        "components.list" | "components.get" | "components.resolve" | "components.execute"
    ) {
        "tessara.components.component-version"
    } else {
        "tessara.components.authoring"
    };
    let envelope = signer
        .sign(AuthorizationGrantV3 {
            schema_version: AUTHORIZATION_GRANT_SCHEMA_VERSION_V3,
            installation_id: context.installation_id,
            original_actor_id: context.actor_id,
            correlation_id: context.correlation_id,
            presenting_service: ModuleServicePrincipalV1::CoreGateway,
            audience: AuthorizationAudienceV1::ModuleInstance {
                module_instance_id: context.module_instance_id,
                module_definition_id: ModuleDefinitionId::new("tessara.components")
                    .expect("Component identity"),
            },
            dependency_binding: DependencyBindingKey::new("tessara.core.components")
                .expect("Component binding"),
            functional_contract: FunctionalContractId::new(contract).expect("Component contract"),
            action: spec.action.into(),
            operation: spec.operation,
            capability_scope_bindings: vec![CapabilityScopeBindingV1 {
                capability: SecurityCapabilityId::new(spec.capability).expect("capability"),
                organization_root_id: spec.scope,
                authorized_organization_ids: spec.additional_scopes.clone(),
            }],
            resource_assertion: None,
            delegation_basis: vec![],
            authorization_revision: 7,
            organization_revision: 11,
            jti: Uuid::new_v4(),
            issued_at: now,
            expires_at: now
                + Duration::seconds(match spec.operation {
                    AuthorizationGrantOperationV1::Read => 60,
                    AuthorizationGrantOperationV1::Mutation => 30,
                }),
        })
        .expect("sign Component authorization grant");
    URL_SAFE_NO_PAD.encode(serde_json::to_vec(&envelope).expect("serialize grant"))
}

async fn publish(
    app: &Router,
    signer: &PurposeBoundSigningKeyV1,
    context: &GrantContext,
    component_id: Uuid,
    version_id: Uuid,
    scope: Uuid,
    idempotency_key: &str,
) {
    let response = module_request(
        app,
        signer,
        context,
        GrantSpec::mutation("components.publish_version", MANAGE_CAPABILITY, scope),
        "POST",
        &format!("/api/admin/components/{component_id}/versions/{version_id}/publish"),
        Some(idempotency_key),
        Value::Null,
    )
    .await;
    assert_eq!(response.status, StatusCode::OK, "{}", response.body);
    assert_eq!(response.body["outcome"], "published");
}

async fn render_current(
    app: &Router,
    signer: &PurposeBoundSigningKeyV1,
    context: &GrantContext,
    scope: Uuid,
    path: &str,
) -> TestResponse {
    module_request(
        app,
        signer,
        context,
        GrantSpec::read("components.execute", READ_CAPABILITY, scope),
        "GET",
        path,
        None,
        Value::Null,
    )
    .await
}

async fn manageable_detail(
    app: &Router,
    signer: &PurposeBoundSigningKeyV1,
    context: &GrantContext,
    scope: Uuid,
    component_id: Uuid,
) -> Value {
    let response = module_request(
        app,
        signer,
        context,
        GrantSpec::read("components.edit", MANAGE_CAPABILITY, scope),
        "GET",
        &format!("/api/admin/components/{component_id}"),
        None,
        Value::Null,
    )
    .await;
    assert_eq!(response.status, StatusCode::OK, "{}", response.body);
    response.body
}

#[allow(clippy::too_many_arguments)]
async fn lifecycle(
    app: &Router,
    signer: &PurposeBoundSigningKeyV1,
    context: &GrantContext,
    component_id: Uuid,
    version_id: Uuid,
    scope: Uuid,
    action: &str,
    expected_resource_revision: u64,
    idempotency_key: &str,
) -> TestResponse {
    module_request(
        app,
        signer,
        context,
        GrantSpec::mutation("components.change_lifecycle", MANAGE_CAPABILITY, scope),
        "POST",
        &format!("/api/admin/components/{component_id}/versions/{version_id}/lifecycle"),
        Some(idempotency_key),
        json!({
            "schema_version": 1,
            "action": action,
            "expected_resource_revision": expected_resource_revision
        }),
    )
    .await
}

fn uuid_at(value: &Value, pointer: &str) -> Uuid {
    Uuid::parse_str(
        value
            .pointer(pointer)
            .and_then(Value::as_str)
            .unwrap_or_else(|| panic!("missing UUID at {pointer}: {value}")),
    )
    .unwrap_or_else(|error| panic!("invalid UUID at {pointer}: {error}"))
}

fn signing_key(
    issuer: &str,
    key_id: &str,
    purpose: ProtocolSignaturePurposeV1,
    byte: u8,
) -> PurposeBoundSigningKeyV1 {
    PurposeBoundSigningKeyV1::from_secret_bytes(issuer, key_id, purpose, [byte; 32])
        .expect("fixed signing key")
}

async fn reset_component_product(pool: &PgPool) {
    sqlx::query(
        "TRUNCATE TABLE
            component_bootstrap_receipts,
            component_mutation_replays,
            component_version_change_events,
            component_versions,
            components,
            component_consumed_service_nonces
         RESTART IDENTITY CASCADE",
    )
    .execute(pool)
    .await
    .expect("reset Component-owned product tables");
    sqlx::query("DELETE FROM component_security_state")
        .execute(pool)
        .await
        .expect("reset Component security fixture");
    sqlx::query(
        "UPDATE component_configuration
         SET display_label='Components',dataset_request_timeout_seconds=5,updated_at=now()
         WHERE singleton=true",
    )
    .execute(pool)
    .await
    .expect("reset Component configuration fixture");
}

fn assert_database_url_names_a_database(database_url: &str) {
    database_url
        .split('?')
        .next()
        .and_then(|url| url.rsplit('/').next())
        .filter(|name| !name.is_empty())
        .expect("TEST_COMPONENT_MODULE_DATABASE_URL must name a database");
}

fn assert_disposable_database_name(database_name: &str) {
    let disposable = database_name
        .split(|character: char| !character.is_alphanumeric())
        .any(|token| {
            matches!(
                token.to_ascii_lowercase().as_str(),
                "test" | "tests" | "testing"
            )
        });
    assert!(
        disposable,
        "TEST_COMPONENT_MODULE_DATABASE_URL resolved to non-disposable server database \
         '{database_name}'; expected a token-bounded test, tests, or testing marker"
    );
}
