use std::collections::BTreeMap;

use axum::{
    Json, Router,
    body::{Body, Bytes},
    extract::{Form, Path, Query, State},
    http::{HeaderMap, StatusCode, header},
    response::{Html, IntoResponse, Redirect, Response},
    routing::{get, post},
};
use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
use chrono::{Duration, Utc};
use rand_core::{OsRng, RngCore};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use sqlx::{PgPool, Postgres, Row, Transaction};
use tessara_module_contract::{
    AdministratorEligibilityDecisionV1, AdministratorEnrollmentClaimKindV1,
    AuthorizationGrantOperationV1, AuthorizationGrantV1, CapabilityScopeBindingV1,
    DependencyBindingKey, EnrollmentRedemptionResultV1, EnrollmentReservationV1,
    ExternalIdentityAssertionV1, FunctionalContractId, LocalOperatorAuthorizationV1,
    ModuleDefinitionId, ModuleManifest, NavigationContributionId, NavigationProjectionV1,
    OriginalActorProjectionV1, ProtocolSignaturePurposeV1, PurposeBoundSigningKeyV1,
    PurposeBoundVerifyingKeyV1, ResourceAuthorizationAssertionV1, SecurityCapabilityId,
    ShellContextV1, ShellDocumentStateV1, ShellThemeV1, SignedEnvelopeV1,
};
use uuid::Uuid;

use crate::{
    auth::AuthenticatedRequest,
    db::AppState,
    error::{ApiError, ApiResult},
};

const FLOOR_VERSION: &str = "core-administration-v1";
const FLOOR_CAPABILITIES: &[&str] = &["core:admin"];

#[derive(Clone, Default, Deserialize, Serialize)]
struct ScopedRecordsDirectoryQuery {
    #[serde(default)]
    q: String,
    organization: Option<String>,
}

pub(crate) fn routes() -> Router<AppState> {
    Router::new()
        .route(
            "/api/private/installation-control/administrator-eligibility",
            post(administrator_eligibility),
        )
        .route(
            "/api/private/installation-control/context",
            get(installation_control_context),
        )
        .route(
            "/api/private/installation-control/enrollment-handoffs",
            post(create_enrollment_handoff),
        )
        .route(
            "/api/admin/core-administration",
            get(core_administration_readback),
        )
        .route(
            "/api/admin/core-administration/designated-role/{role_id}",
            axum::routing::put(designate_enrollment_role),
        )
        .route(
            "/api/modules/authorization/exchange",
            post(exchange_authorization),
        )
        .route(
            "/api/administrator-enrollment/redeem",
            post(redeem_administrator),
        )
        .route(
            "/api/administrator-enrollment/redeem/local",
            get(legacy_enrollment_redirect).post(redeem_local_form),
        )
        .route(
            "/api/modules/instances/{instance_id}/configuration",
            axum::routing::put(update_module_configuration),
        )
        .route(
            "/api/modules/instances/{instance_id}/configuration/form",
            post(update_module_configuration_form),
        )
        .route(
            "/api/modules/instances/{instance_id}/enablement/form",
            post(update_module_enablement_form),
        )
        .route(
            "/reference/scoped-records/api/records",
            get(proxy_record_list).post(proxy_record_create),
        )
        .route(
            "/reference/scoped-records/api/records/{record_id}",
            get(proxy_record_detail).put(proxy_record_update),
        )
        .route(
            "/reference/scoped-records/records",
            post(create_scoped_record_form),
        )
        .route(
            "/reference/scoped-records/records/{record_id}",
            get(proxy_scoped_record_page).post(update_scoped_record_form),
        )
        .route("/reference/scoped-records", get(proxy_scoped_records_root))
        .route(
            "/reference/scoped-records/{*module_path}",
            get(proxy_scoped_records_page),
        )
        .route(
            "/reference/{*module_path}",
            get(proxy_manifest_module_document),
        )
        .route(
            "/_tessara/modules/{definition}/{release}/{digest}/{*asset_path}",
            get(proxy_manifest_module_asset),
        )
}

#[derive(Default, Deserialize)]
pub(crate) struct EnrollmentPageQuery {
    handoff: Option<String>,
    complete: Option<String>,
}

async fn legacy_enrollment_redirect() -> Redirect {
    Redirect::to("/enrollment")
}

#[derive(Clone, Default)]
struct EnrollmentPrefill {
    claim_id: Option<Uuid>,
    generation: Option<u32>,
    claim_kind: Option<AdministratorEnrollmentClaimKindV1>,
}

pub(crate) async fn enrollment_page(
    State(state): State<AppState>,
    Query(query): Query<EnrollmentPageQuery>,
) -> Html<String> {
    let viable_administrator = viable_administrator_exists(&state.pool)
        .await
        .unwrap_or(false);
    if viable_administrator {
        if query.complete.as_deref() == Some("1") {
            Html(enrollment_success_document())
        } else {
            Html(enrollment_closed_document())
        }
    } else {
        let installation_id = sqlx::query_scalar::<_, Uuid>(
            "SELECT id FROM application_installations WHERE singleton=true",
        )
        .fetch_optional(&state.pool)
        .await
        .ok()
        .flatten()
        .unwrap_or_default();
        let prefill = match query.handoff.as_deref() {
            Some(token) => match consume_enrollment_handoff(&state.pool, token).await {
                Ok(Some(prefill)) => prefill,
                _ => {
                    return Html(enrollment_unavailable_document(
                        "This enrollment handoff is unavailable or has expired.",
                    ));
                }
            },
            None => EnrollmentPrefill::default(),
        };
        Html(enrollment_form_document(installation_id, &prefill))
    }
}

fn enrollment_document(title: &str, content: &str) -> String {
    format!(
        r#"<!doctype html><html lang="en"><head><meta charset="utf-8"><meta name="viewport" content="width=device-width,initial-scale=1"><title>{title} · Tessara</title><style>
:root{{color-scheme:dark;--bg:#0c1528;--panel:#2b3c55;--control:#172238;--border:#4a5c74;--text:#f3f7ff;--muted:#b5c0d1;--teal:#19bdb0;--teal-soft:rgb(25 189 176 / 12%);--indigo-soft:rgb(99 102 241 / 13%);--warning:#f59e0b}}*{{box-sizing:border-box}}body{{margin:0;min-height:100vh;background:radial-gradient(circle at 35% 20%,rgb(20 184 166 / 8%),transparent 35%),var(--bg);color:var(--text);font:16px/1.45 Inter,ui-sans-serif,system-ui,sans-serif}}button,input,textarea{{font:inherit}}.shell{{min-height:100vh;display:grid;place-items:center;padding:2rem 1rem}}.panel{{width:min(35rem,100%);border:1px solid var(--border);border-radius:.6rem;background:var(--panel);padding:1.6rem;box-shadow:0 1.5rem 4rem rgb(0 0 0 / 24%)}}.brand{{display:inline-flex;align-items:center;gap:.75rem;color:var(--text);text-decoration:none;font-size:1.1rem;font-weight:850}}.brand img{{width:2.5rem;height:2.5rem}}.kicker{{display:inline-flex;align-items:center;margin-top:1.25rem;border:1px solid rgb(99 102 241 / 32%);border-radius:.4rem;background:var(--indigo-soft);padding:.35rem .6rem;font-size:.78rem;font-weight:800}}h1{{margin:1.35rem 0 .25rem;color:var(--teal);font-size:1.75rem;line-height:1.2}}p{{margin:.25rem 0;color:var(--muted)}}.segmented,.identity-choice{{display:grid;grid-template-columns:repeat(2,minmax(0,1fr));gap:.45rem;margin-top:1.1rem}}.segmented button,.identity-card{{border:1px solid var(--border);border-radius:.45rem;background:var(--control);color:var(--muted);padding:.7rem;font-weight:800;cursor:pointer}}.segmented button.is-active,.identity-card.is-active{{border-color:rgb(25 189 176 / 52%);background:var(--teal-soft);color:var(--teal)}}.segmented button:disabled{{cursor:default;opacity:.72}}.identity-card{{display:grid;gap:.15rem;text-align:left;color:var(--text)}}.identity-card small{{color:var(--muted);font-weight:500}}.notice,.assignment,.success,.error-panel{{display:grid;gap:.2rem;margin-top:1rem;border:1px solid rgb(25 189 176 / 30%);border-radius:.45rem;background:var(--teal-soft);padding:.8rem .9rem}}.notice{{border-color:rgb(245 158 11 / 35%);background:rgb(245 158 11 / 10%)}}.notice[hidden],.identity-fields[hidden],.error-panel[hidden]{{display:none}}.error-panel{{border-color:rgb(244 63 94 / 38%);background:rgb(244 63 94 / 10%)}}form,.identity-fields{{display:grid;gap:.9rem}}form{{margin-top:1rem}}label{{display:grid;gap:.35rem;font-size:.86rem;font-weight:800}}input,textarea{{width:100%;border:1px solid var(--border);border-radius:.4rem;background:var(--control);color:var(--text);padding:.72rem .8rem}}input[readonly]{{color:var(--muted)}}textarea{{resize:vertical}}.claim-grid{{display:grid;grid-template-columns:minmax(0,1fr) 7rem;gap:.65rem}}.claim-context{{display:grid;grid-template-columns:1fr 1fr;gap:.35rem .8rem;margin-top:1rem;padding:.7rem .8rem;border:1px solid var(--border);border-radius:.45rem;background:rgb(12 21 40 / 25%);font-size:.76rem}}.claim-context>div{{display:grid;min-width:0;gap:.15rem}}.claim-context span{{color:var(--muted)}}.claim-context strong{{overflow-wrap:anywhere}}.help{{color:var(--muted);font-size:.75rem;font-weight:500}}.submit,.continue{{display:inline-flex;justify-content:center;width:100%;border:0;border-radius:.4rem;background:var(--teal);color:#071722;padding:.78rem;font-weight:900;text-decoration:none;cursor:pointer}}.submit:disabled{{cursor:wait;opacity:.68}}.error{{min-height:1.2rem;color:#fda4af;font-size:.8rem}}.reissue{{display:block;margin-top:.35rem;color:var(--text)}}code{{overflow-wrap:anywhere}}@media(max-width:34rem){{.panel{{padding:1.15rem}}.identity-choice,.claim-grid,.claim-context{{grid-template-columns:1fr}}}}
</style></head><body><main class="shell"><section class="panel" aria-labelledby="enrollment-title"><a class="brand" href="/login" aria-label="Tessara sign in"><img src="/assets/tessara-icon-256.svg" alt=""><strong>Tessara</strong></a>{content}</section></main></body></html>"#
    )
}

fn enrollment_success_document() -> String {
    enrollment_document(
        "Enrollment Successful",
        r#"<div role="status"><h1 id="enrollment-title">Enrollment successful</h1><p>The Core Administrator account is ready. Redirecting you to sign in…</p></div><p style="margin-top:1.25rem"><a class="continue" href="/login">Continue to sign in</a></p><script>setTimeout(()=>location.replace('/login'),1800);</script>"#,
    )
}

fn enrollment_closed_document() -> String {
    enrollment_document(
        "Administrator Enrollment Closed",
        r#"<h1 id="enrollment-title">Administrator enrollment closed</h1><p>A viable Core Administrator already exists. Enrollment claims are unavailable until recovery is required.</p><p style="margin-top:1rem"><a class="continue" href="/login">Continue to sign in</a></p>"#,
    )
}

fn enrollment_unavailable_document(message: &str) -> String {
    enrollment_document(
        "Administrator Enrollment Unavailable",
        &format!(
            r#"<h1 id="enrollment-title">Enrollment unavailable</h1><p>{message}</p><div class="error-panel"><strong>The claim was not used</strong><span class="help">Return to the enrollment page to try an active claim, or issue a replacement from the local Supervisor.</span><code>.\scripts\tessara.ps1 enrollment issue -Open</code></div><p style="margin-top:1rem"><a class="continue" href="/enrollment">Try another claim</a></p>"#
        ),
    )
}

fn enrollment_form_document(installation_id: Uuid, prefill: &EnrollmentPrefill) -> String {
    let claim_id = prefill
        .claim_id
        .map(|value| value.to_string())
        .unwrap_or_default();
    let generation = prefill
        .generation
        .map(|value| value.to_string())
        .unwrap_or_default();
    let claim_kind = prefill.claim_kind.map(claim_kind_text).unwrap_or("initial");
    let initial_active = if claim_kind == "initial" {
        " is-active"
    } else {
        ""
    };
    let recovery_active = if claim_kind == "recovery" {
        " is-active"
    } else {
        ""
    };
    let recovery_hidden = if claim_kind == "recovery" {
        ""
    } else {
        " hidden"
    };
    let prepared = prefill.claim_id.is_some();
    let claim_readonly = if prepared { " readonly" } else { "" };
    let kind_disabled = if prepared { " disabled" } else { "" };
    let heading = if claim_kind == "recovery" {
        "Recover administrator access"
    } else {
        "Establish an administrator"
    };
    let submit_label = if claim_kind == "recovery" {
        "Recover administrator access"
    } else {
        "Enroll administrator"
    };
    let content = format!(
        r#"<h1 id="enrollment-title">{heading}</h1><p>Use the installation claim once to establish a floor-compliant Core administrator.</p>
<div class="segmented" aria-label="Enrollment claim kind"><button class="{initial_active}" type="button" data-kind="initial"{kind_disabled}>Initial</button><button class="{recovery_active}" type="button" data-kind="recovery"{kind_disabled}>Recovery</button></div>
<div class="notice" id="recovery-notice"{recovery_hidden}><strong>Audited recovery claim</strong><span class="help">Recovery claims require an explicit local operator authorization recorded by installation control.</span></div>
<div class="claim-context"><div><span>Installation ID</span><strong>{installation_id}</strong></div><div><span>Claim kind</span><strong id="claim-kind-readback">{claim_kind}</strong></div></div>
<div class="identity-choice" role="group" aria-label="Identity path"><button class="identity-card is-active" type="button" data-identity="local"><strong>Local account</strong><small>Create a Tessara password</small></button><button class="identity-card" type="button" data-identity="fixture_external"><strong>Fixture external identity</strong><small>Bind a signed assertion</small></button></div>
<form id="enrollment-form" method="post" action="/api/administrator-enrollment/redeem/local">
<input name="schema_version" type="hidden" value="1"><input id="claim-kind" name="claim_kind" type="hidden" value="{claim_kind}">
<div class="claim-grid"><label>Claim ID<input name="claim_id" autocomplete="off" value="{claim_id}"{claim_readonly} required></label><label>Generation<input name="generation" inputmode="numeric" min="1" value="{generation}"{claim_readonly} required></label></div>
<label>Claim secret<input name="claim_secret" type="password" autocomplete="one-time-code" required><span class="help">Write-only. The claim will not appear again after submission.</span></label>
<div class="identity-fields" id="local-fields"><label>Email<input name="email" type="email" autocomplete="email" required></label><label>Display name<input name="display_name" autocomplete="name" required></label><label>Password<input name="password" type="password" autocomplete="new-password" minlength="12" required><span class="help">Use at least 12 characters. A longer, unique passphrase is recommended.</span></label></div>
<div class="identity-fields" id="external-fields" hidden><label>Signed fixture assertion<textarea id="external-assertion" rows="5" placeholder="Paste the one-time signed assertion"></textarea><span class="help">Development conformance path only. No email-based account merging.</span></label></div>
<div class="assignment"><strong>Core Administrator</strong><span class="help">Meets Core Administration Capability Floor v1 · installation-global</span></div><div class="error-panel" id="enrollment-error-panel" hidden><strong>Enrollment unavailable</strong><span class="help" id="enrollment-error" role="alert">Confirm the active one-time claim and identity details, then try again.</span><span class="help">If the claim expired or was already used, issue a replacement from the local Supervisor.</span><code id="reissue-command">.\scripts\tessara.ps1 enrollment issue -Open</code></div><button class="submit" id="enrollment-submit" type="submit">{submit_label}</button></form>
<script>
(()=>{{if(location.search)history.replaceState(null,'','/enrollment');const form=document.querySelector('#enrollment-form'),kind=document.querySelector('#claim-kind'),kindReadback=document.querySelector('#claim-kind-readback'),heading=document.querySelector('#enrollment-title'),notice=document.querySelector('#recovery-notice'),local=document.querySelector('#local-fields'),external=document.querySelector('#external-fields'),errorPanel=document.querySelector('#enrollment-error-panel'),error=document.querySelector('#enrollment-error'),reissue=document.querySelector('#reissue-command'),submit=document.querySelector('#enrollment-submit');let identity='local';document.querySelectorAll('[data-kind]').forEach(button=>button.addEventListener('click',()=>{{document.querySelectorAll('[data-kind]').forEach(item=>item.classList.toggle('is-active',item===button));kind.value=button.dataset.kind;kindReadback.textContent=kind.value;notice.hidden=kind.value!=='recovery';heading.textContent=kind.value==='recovery'?'Recover administrator access':'Establish an administrator';submit.textContent=kind.value==='recovery'?'Recover administrator access':'Enroll administrator';reissue.textContent=kind.value==='recovery'?'.\\scripts\\tessara.ps1 enrollment recover -Reason \"...\" -Open':'.\\scripts\\tessara.ps1 enrollment issue -Open';}}));document.querySelectorAll('[data-identity]').forEach(button=>button.addEventListener('click',()=>{{identity=button.dataset.identity;document.querySelectorAll('[data-identity]').forEach(item=>item.classList.toggle('is-active',item===button));local.hidden=identity!=='local';external.hidden=identity==='local';local.querySelectorAll('input').forEach(input=>input.disabled=identity!=='local');}}));form.addEventListener('submit',async event=>{{event.preventDefault();errorPanel.hidden=true;submit.disabled=true;const data=new FormData(form);try{{const assertion=identity==='fixture_external'?JSON.parse(document.querySelector('#external-assertion').value):null,response=await fetch('/api/administrator-enrollment/redeem',{{method:'POST',headers:{{'content-type':'application/json'}},body:JSON.stringify({{schema_version:1,installation_id:'{installation_id}',claim_id:data.get('claim_id'),generation:Number(data.get('generation')),claim_kind:kind.value,claim_secret:data.get('claim_secret'),idempotency_key:crypto.randomUUID(),identity_path:identity,email:identity==='local'?data.get('email'):null,display_name:identity==='local'?data.get('display_name'):null,password:identity==='local'?data.get('password'):null,external_assertion:assertion}})}});if(!response.ok)throw new Error();location.assign('/enrollment?complete=1');}}catch(_error){{error.textContent='Enrollment is unavailable. Confirm the active one-time claim and identity details, then try again.';errorPanel.hidden=false;submit.disabled=false;form.querySelector('[name=claim_secret]').value='';}}}});}})();
</script>"#
    );
    enrollment_document("Administrator Enrollment", &content)
}

async fn consume_enrollment_handoff(
    pool: &PgPool,
    token: &str,
) -> ApiResult<Option<EnrollmentPrefill>> {
    if token.len() > 200 || token.is_empty() {
        return Ok(None);
    }
    let row = sqlx::query(
        "UPDATE administrator_enrollment_handoffs
         SET consumed_at=now()
         WHERE token_digest=$1 AND consumed_at IS NULL AND expires_at > now()
         RETURNING claim_id,generation,claim_kind",
    )
    .bind(handoff_token_digest(token))
    .fetch_optional(pool)
    .await?;
    row.map(|row| {
        let generation = u32::try_from(row.try_get::<i32, _>("generation")?)
            .map_err(|_| enrollment_unavailable())?;
        let claim_kind = match row.try_get::<String, _>("claim_kind")?.as_str() {
            "initial" => AdministratorEnrollmentClaimKindV1::Initial,
            "recovery" => AdministratorEnrollmentClaimKindV1::Recovery,
            _ => return Err(enrollment_unavailable()),
        };
        Ok(EnrollmentPrefill {
            claim_id: Some(row.try_get("claim_id")?),
            generation: Some(generation),
            claim_kind: Some(claim_kind),
        })
    })
    .transpose()
}

fn handoff_token_digest(token: &str) -> String {
    use sha2::{Digest, Sha256};
    format!("sha256:{:x}", Sha256::digest(token.as_bytes()))
}

#[derive(Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct EligibilityRequest {
    schema_version: u16,
    installation_id: Option<Uuid>,
    kind: AdministratorEnrollmentClaimKindV1,
    recovery_authorization: Option<SignedEnvelopeV1<LocalOperatorAuthorizationV1>>,
}

#[derive(Serialize)]
struct InstallationControlContext {
    schema_version: u16,
    installation_id: Uuid,
}

async fn installation_control_context(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> ApiResult<Json<InstallationControlContext>> {
    require_installation_control(&headers)?;
    let installation_id =
        sqlx::query_scalar("SELECT id FROM application_installations WHERE singleton=true")
            .fetch_one(&state.pool)
            .await?;
    Ok(Json(InstallationControlContext {
        schema_version: 1,
        installation_id,
    }))
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct CreateEnrollmentHandoffRequest {
    schema_version: u16,
    installation_id: Uuid,
    claim_id: Uuid,
    generation: u32,
    claim_kind: AdministratorEnrollmentClaimKindV1,
}

#[derive(Serialize)]
struct CreatedEnrollmentHandoff {
    schema_version: u16,
    handoff_token: String,
    expires_at: chrono::DateTime<Utc>,
}

async fn create_enrollment_handoff(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<CreateEnrollmentHandoffRequest>,
) -> ApiResult<Json<CreatedEnrollmentHandoff>> {
    require_installation_control(&headers)?;
    if payload.schema_version != 1 || payload.generation == 0 {
        return Err(ApiError::BadRequest(
            "unsupported enrollment handoff".into(),
        ));
    }
    require_installation(&state.pool, payload.installation_id).await?;
    let mut token_bytes = [0_u8; 32];
    OsRng.fill_bytes(&mut token_bytes);
    let handoff_token = URL_SAFE_NO_PAD.encode(token_bytes);
    let expires_at = Utc::now() + Duration::seconds(120);
    sqlx::query(
        "DELETE FROM administrator_enrollment_handoffs
         WHERE expires_at <= now() OR consumed_at IS NOT NULL",
    )
    .execute(&state.pool)
    .await?;
    sqlx::query(
        "INSERT INTO administrator_enrollment_handoffs
         (token_digest,installation_id,claim_id,generation,claim_kind,expires_at)
         VALUES ($1,$2,$3,$4,$5,$6)",
    )
    .bind(handoff_token_digest(&handoff_token))
    .bind(payload.installation_id)
    .bind(payload.claim_id)
    .bind(payload.generation as i32)
    .bind(claim_kind_text(payload.claim_kind))
    .bind(expires_at)
    .execute(&state.pool)
    .await?;
    Ok(Json(CreatedEnrollmentHandoff {
        schema_version: 1,
        handoff_token,
        expires_at,
    }))
}

async fn administrator_eligibility(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<EligibilityRequest>,
) -> ApiResult<Json<SignedEnvelopeV1<AdministratorEligibilityDecisionV1>>> {
    require_installation_control(&headers)?;
    if payload.schema_version != 1
        || (payload.kind == AdministratorEnrollmentClaimKindV1::Initial
            && payload.recovery_authorization.is_some())
        || (payload.kind == AdministratorEnrollmentClaimKindV1::Recovery
            && payload.recovery_authorization.is_none())
    {
        return Err(ApiError::BadRequest("unsupported schema version".into()));
    }
    let installation_id = match payload.installation_id {
        Some(installation_id) => installation_id,
        None => {
            sqlx::query_scalar("SELECT id FROM application_installations WHERE singleton=true")
                .fetch_one(&state.pool)
                .await?
        }
    };
    require_installation(&state.pool, installation_id).await?;
    let viable = viable_administrator_exists(&state.pool).await?;
    if viable {
        sqlx::query(
            "UPDATE core_administration_state
             SET has_ever_had_viable_administrator=true, updated_at=now()
             WHERE singleton=true",
        )
        .execute(&state.pool)
        .await?;
    }
    let has_ever: bool = sqlx::query_scalar(
        "SELECT has_ever_had_viable_administrator
         FROM core_administration_state WHERE singleton=true",
    )
    .fetch_one(&state.pool)
    .await?;
    let now = Utc::now();
    let decision = AdministratorEligibilityDecisionV1 {
        schema_version: 1,
        installation_id,
        viable_administrator_exists: viable,
        has_ever_had_viable_administrator: has_ever,
        recovery_authorization: payload.recovery_authorization,
        nonce: Uuid::new_v4(),
        issued_at: now,
        expires_at: now + Duration::seconds(30),
    };
    // Recovery evidence is supplied by installation control and verified there;
    // Core never manufactures the local operator authorization.
    let envelope = protocol_signer(ProtocolSignaturePurposeV1::EnrollmentEligibility)?
        .sign(decision)
        .map_err(|error| ApiError::Internal(error.into()))?;
    Ok(Json(envelope))
}

#[derive(Serialize)]
struct CoreAdministrationReadback {
    schema_version: u16,
    floor_version: String,
    floor_capabilities: Vec<String>,
    designated_role_id: Uuid,
    designated_role_name: String,
    designated_role_compliant: bool,
    viable_administrator_exists: bool,
    has_ever_had_viable_administrator: bool,
    authorization_revision: i64,
    organization_revision: i64,
}

async fn core_administration_readback(
    State(state): State<AppState>,
    request: AuthenticatedRequest,
) -> ApiResult<Json<CoreAdministrationReadback>> {
    request.require_capability("admin:all")?;
    let row = sqlx::query(
        "SELECT state.floor_version, state.designated_enrollment_role_id,
                state.has_ever_had_viable_administrator, roles.name,
                revisions.authorization_revision, revisions.organization_revision
         FROM core_administration_state state
         JOIN roles ON roles.id=state.designated_enrollment_role_id
         CROSS JOIN core_security_revisions revisions
         WHERE state.singleton=true AND revisions.singleton=true",
    )
    .fetch_one(&state.pool)
    .await?;
    let role_id: Uuid = row.try_get("designated_enrollment_role_id")?;
    Ok(Json(CoreAdministrationReadback {
        schema_version: 1,
        floor_version: row.try_get("floor_version")?,
        floor_capabilities: FLOOR_CAPABILITIES
            .iter()
            .map(|value| (*value).to_string())
            .collect(),
        designated_role_id: role_id,
        designated_role_name: row.try_get("name")?,
        designated_role_compliant: role_is_floor_compliant(&state.pool, role_id).await?,
        viable_administrator_exists: viable_administrator_exists(&state.pool).await?,
        has_ever_had_viable_administrator: row.try_get("has_ever_had_viable_administrator")?,
        authorization_revision: row.try_get("authorization_revision")?,
        organization_revision: row.try_get("organization_revision")?,
    }))
}

async fn designate_enrollment_role(
    State(state): State<AppState>,
    request: AuthenticatedRequest,
    Path(role_id): Path<Uuid>,
) -> ApiResult<StatusCode> {
    request.require_capability("admin:all")?;
    let mut transaction = state.pool.begin().await?;
    let compliant: bool = sqlx::query_scalar(
        "SELECT COUNT(DISTINCT c.key)=$2
         FROM roles r
         JOIN role_capabilities rc ON rc.role_id=r.id
         JOIN capabilities c ON c.id=rc.capability_id
         WHERE r.id=$1 AND c.key=ANY($3)",
    )
    .bind(role_id)
    .bind(FLOOR_CAPABILITIES.len() as i64)
    .bind(FLOOR_CAPABILITIES)
    .fetch_one(&mut *transaction)
    .await?;
    let has_scoped_assignments: bool = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM role_assignments WHERE role_id=$1 AND node_id IS NOT NULL)",
    )
    .bind(role_id)
    .fetch_one(&mut *transaction)
    .await?;
    if !compliant || has_scoped_assignments {
        return Err(ApiError::BadRequest(format!(
            "the designated role must be installation-global and satisfy {FLOOR_VERSION}"
        )));
    }
    sqlx::query(
        "UPDATE core_administration_state
         SET designated_enrollment_role_id=$1,updated_at=now() WHERE singleton=true",
    )
    .bind(role_id)
    .execute(&mut *transaction)
    .await?;
    sqlx::query(
        "INSERT INTO core_security_events
         (event_kind,actor_account_id,subject_id,evidence)
         VALUES ('designated_enrollment_role_changed',$1,$2,$3)",
    )
    .bind(request.account.account_id)
    .bind(role_id)
    .bind(serde_json::json!({"floor_version": FLOOR_VERSION}))
    .execute(&mut *transaction)
    .await?;
    sqlx::query("SELECT advance_authorization_revision()")
        .execute(&mut *transaction)
        .await?;
    transaction.commit().await?;
    Ok(StatusCode::NO_CONTENT)
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct AuthorizationExchangeRequest {
    schema_version: u16,
    installation_id: Uuid,
    audience_module_instance_id: Uuid,
    dependency_binding: DependencyBindingKey,
    functional_contract: FunctionalContractId,
    action: String,
    operation: AuthorizationGrantOperationV1,
    resource_assertion: Option<ResourceAuthorizationAssertionV1>,
}

async fn exchange_authorization(
    State(state): State<AppState>,
    request: AuthenticatedRequest,
    Json(payload): Json<AuthorizationExchangeRequest>,
) -> ApiResult<Json<SignedEnvelopeV1<AuthorizationGrantV1>>> {
    if payload.schema_version != 1 {
        return Err(restricted_authorization());
    }
    let instance = sqlx::query(
        "SELECT definition_id FROM module_instances
         WHERE id=$1 AND installation_id=$2 AND identity_state='live'
           AND installed=true AND deployed=true AND configured=true
           AND ready=true AND enabled=true AND healthy=true",
    )
    .bind(payload.audience_module_instance_id)
    .bind(payload.installation_id)
    .fetch_optional(&state.pool)
    .await?
    .ok_or_else(restricted_authorization)?;
    let target_definition: String = instance.try_get("definition_id")?;
    let operation = operation_text(payload.operation);
    let required_capability: String = sqlx::query_scalar(
        "SELECT required_capability FROM core_module_action_declarations
         WHERE target_definition_id=$1 AND dependency_binding=$2
           AND functional_contract=$3 AND action=$4 AND operation=$5",
    )
    .bind(&target_definition)
    .bind(payload.dependency_binding.as_str())
    .bind(payload.functional_contract.as_str())
    .bind(&payload.action)
    .bind(operation)
    .fetch_optional(&state.pool)
    .await?
    .ok_or_else(restricted_authorization)?;

    let bindings = capability_bindings(
        &state.pool,
        request.account.account_id,
        &required_capability,
    )
    .await?;
    if bindings.is_empty() {
        return Err(restricted_authorization());
    }
    if let Some(resource) = &payload.resource_assertion {
        let capability = SecurityCapabilityId::new(required_capability.clone())
            .map_err(|error| ApiError::Internal(error.into()))?;
        if !bindings
            .iter()
            .any(|binding| binding.authorizes(&capability, resource.owner_organization_id))
        {
            return Err(restricted_authorization());
        }
    }
    let revisions = sqlx::query(
        "SELECT authorization_revision, organization_revision
         FROM core_security_revisions WHERE singleton=true",
    )
    .fetch_one(&state.pool)
    .await?;
    let authorization_revision = revisions.try_get::<i64, _>("authorization_revision")?;
    let organization_revision = revisions.try_get::<i64, _>("organization_revision")?;
    if target_definition == tessara_reference_scoped_records::MODULE_DEFINITION_ID {
        sync_scoped_records_security_state(
            payload.installation_id,
            payload.audience_module_instance_id,
            authorization_revision,
            organization_revision,
            true,
            "enabled",
        )
        .await?;
    }
    let now = Utc::now();
    let grant = AuthorizationGrantV1 {
        schema_version: 1,
        installation_id: payload.installation_id,
        original_actor_id: request.account.account_id,
        presenting_service: ModuleDefinitionId::new("tessara.core")
            .map_err(|error| ApiError::Internal(error.into()))?,
        audience_module_instance_id: payload.audience_module_instance_id,
        dependency_binding: payload.dependency_binding,
        functional_contract: payload.functional_contract,
        action: payload.action,
        operation: payload.operation,
        capability_scope_bindings: bindings,
        resource_assertion: payload.resource_assertion,
        delegation_basis: Vec::new(),
        authorization_revision: authorization_revision as u64,
        organization_revision: organization_revision as u64,
        jti: Uuid::new_v4(),
        issued_at: now,
        expires_at: now
            + Duration::seconds(match payload.operation {
                AuthorizationGrantOperationV1::Read => 60,
                AuthorizationGrantOperationV1::Mutation => 30,
            }),
    };
    let signed = protocol_signer(ProtocolSignaturePurposeV1::AuthorizationGrant)?
        .sign(grant)
        .map_err(|error| ApiError::Internal(error.into()))?;
    Ok(Json(signed))
}

pub(crate) async fn capability_bindings(
    pool: &PgPool,
    account_id: Uuid,
    capability: &str,
) -> ApiResult<Vec<CapabilityScopeBindingV1>> {
    let global: bool = sqlx::query_scalar(
        "SELECT EXISTS(
           SELECT 1 FROM role_assignments ra
           JOIN role_capabilities rc ON rc.role_id=ra.role_id
           JOIN capabilities c ON c.id=rc.capability_id
           WHERE ra.account_id=$1 AND ra.node_id IS NULL
             AND (c.key=$2 OR c.key='admin:all')
         )",
    )
    .bind(account_id)
    .bind(capability)
    .fetch_one(pool)
    .await?;
    let roots: Vec<Uuid> = if global {
        sqlx::query_scalar("SELECT id FROM nodes WHERE parent_node_id IS NULL ORDER BY id")
            .fetch_all(pool)
            .await?
    } else {
        sqlx::query_scalar(
            "SELECT DISTINCT ra.node_id FROM role_assignments ra
             JOIN role_capabilities rc ON rc.role_id=ra.role_id
             JOIN capabilities c ON c.id=rc.capability_id
             WHERE ra.account_id=$1 AND ra.node_id IS NOT NULL
               AND (c.key=$2 OR ($2 LIKE '%:read' AND c.key=replace($2, ':read', ':manage')))
             ORDER BY ra.node_id",
        )
        .bind(account_id)
        .bind(capability)
        .fetch_all(pool)
        .await?
    };
    let capability = SecurityCapabilityId::new(capability.to_string())
        .map_err(|error| ApiError::Internal(error.into()))?;
    let mut bindings = Vec::with_capacity(roots.len());
    for root in roots {
        let descendants: Vec<Uuid> = sqlx::query_scalar(
            "WITH RECURSIVE descendants(id) AS (
                SELECT id FROM nodes WHERE id=$1
                UNION
                SELECT nodes.id FROM nodes JOIN descendants ON nodes.parent_node_id=descendants.id
             ) SELECT id FROM descendants WHERE id <> $1 ORDER BY id",
        )
        .bind(root)
        .fetch_all(pool)
        .await?;
        bindings.push(CapabilityScopeBindingV1 {
            capability: capability.clone(),
            organization_root_id: root,
            authorized_organization_ids: descendants,
        });
    }
    Ok(bindings)
}

pub(crate) async fn ensure_designated_role_update_compliant(
    transaction: &mut Transaction<'_, Postgres>,
    role_id: Uuid,
    proposed_capability_ids: &[Uuid],
) -> ApiResult<()> {
    let designated: bool = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM core_administration_state
         WHERE singleton=true AND designated_enrollment_role_id=$1)",
    )
    .bind(role_id)
    .fetch_one(&mut **transaction)
    .await?;
    if !designated {
        return Ok(());
    }
    let complete: bool = sqlx::query_scalar(
        "SELECT COUNT(DISTINCT key) = $2
         FROM capabilities WHERE id=ANY($1) AND key=ANY($3)",
    )
    .bind(proposed_capability_ids)
    .bind(FLOOR_CAPABILITIES.len() as i64)
    .bind(FLOOR_CAPABILITIES)
    .fetch_one(&mut **transaction)
    .await?;
    if !complete {
        return Err(ApiError::BadRequest(format!(
            "the designated enrollment role must satisfy {FLOOR_VERSION}"
        )));
    }
    Ok(())
}

async fn role_is_floor_compliant(pool: &PgPool, role_id: Uuid) -> ApiResult<bool> {
    Ok(sqlx::query_scalar(
        "SELECT COUNT(DISTINCT c.key) = $2
         FROM role_capabilities rc JOIN capabilities c ON c.id=rc.capability_id
         WHERE rc.role_id=$1 AND c.key=ANY($3)",
    )
    .bind(role_id)
    .bind(FLOOR_CAPABILITIES.len() as i64)
    .bind(FLOOR_CAPABILITIES)
    .fetch_one(pool)
    .await?)
}

pub(crate) async fn viable_administrator_exists(pool: &PgPool) -> ApiResult<bool> {
    Ok(sqlx::query_scalar(
        "SELECT EXISTS(
          SELECT 1 FROM core_administration_state state
          JOIN role_assignments ra ON ra.node_id IS NULL
          JOIN accounts a ON a.id=ra.account_id AND a.is_active=true
          WHERE state.singleton=true
            AND (
              EXISTS(SELECT 1 FROM account_credentials ac WHERE ac.account_id=a.id)
              OR EXISTS(SELECT 1 FROM external_identity_bindings e
                        WHERE e.account_id=a.id AND e.is_usable=true)
            )
            AND NOT EXISTS(
              SELECT 1 FROM unnest($1::text[]) floor_capability
              WHERE NOT EXISTS(
                SELECT 1 FROM role_capabilities rc
                JOIN capabilities c ON c.id=rc.capability_id
                WHERE rc.role_id=ra.role_id
                  AND c.key=floor_capability
              )
            )
        )",
    )
    .bind(FLOOR_CAPABILITIES)
    .fetch_one(pool)
    .await?)
}

async fn require_installation(pool: &PgPool, installation_id: Uuid) -> ApiResult<()> {
    let exists: bool =
        sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM application_installations WHERE id=$1)")
            .bind(installation_id)
            .fetch_one(pool)
            .await?;
    if exists {
        Ok(())
    } else {
        Err(ApiError::NotFound("application installation".into()))
    }
}

fn require_installation_control(headers: &HeaderMap) -> ApiResult<()> {
    let expected = std::env::var("TESSARA_INSTALLATION_CONTROL_SHARED_KEY")
        .unwrap_or_else(|_| "development-installation-control-only".into());
    let presented = headers
        .get("x-tessara-installation-control-key")
        .and_then(|value| value.to_str().ok());
    if presented == Some(expected.as_str()) {
        Ok(())
    } else {
        Err(ApiError::Unauthorized)
    }
}

pub(crate) fn protocol_signer(
    purpose: ProtocolSignaturePurposeV1,
) -> ApiResult<PurposeBoundSigningKeyV1> {
    let secret = match std::env::var("TESSARA_CORE_PROTOCOL_SIGNING_KEY") {
        Ok(encoded) => {
            let decoded = URL_SAFE_NO_PAD
                .decode(encoded)
                .map_err(|error| ApiError::Internal(error.into()))?;
            decoded.try_into().map_err(|_| {
                ApiError::Internal(anyhow::anyhow!(
                    "TESSARA_CORE_PROTOCOL_SIGNING_KEY must be 32 base64url bytes"
                ))
            })?
        }
        Err(_) => [12_u8; 32],
    };
    PurposeBoundSigningKeyV1::from_secret_bytes(
        "tessara.core",
        "core-development-v1",
        purpose,
        secret,
    )
    .map_err(|error| ApiError::Internal(error.into()))
}

fn operation_text(operation: AuthorizationGrantOperationV1) -> &'static str {
    match operation {
        AuthorizationGrantOperationV1::Read => "read",
        AuthorizationGrantOperationV1::Mutation => "mutation",
    }
}

fn restricted_authorization() -> ApiError {
    ApiError::Forbidden("module action unavailable".into())
}

async fn proxy_scoped_records_root(
    State(state): State<AppState>,
    request: AuthenticatedRequest,
    Query(query): Query<ScopedRecordsDirectoryQuery>,
) -> ApiResult<Response> {
    require_module_page_access(&request)?;
    scoped_records_document(&state.pool, &request, "", Some(&query)).await
}

async fn proxy_scoped_records_page(
    State(state): State<AppState>,
    request: AuthenticatedRequest,
    Path(module_path): Path<String>,
) -> ApiResult<Response> {
    require_module_page_access(&request)?;
    if module_path.starts_with("api/") || module_path.contains("..") {
        return Err(ApiError::NotFound("module route".into()));
    }
    scoped_records_document(&state.pool, &request, &module_path, None).await
}

async fn proxy_scoped_record_page(
    State(state): State<AppState>,
    request: AuthenticatedRequest,
    Path(record_id): Path<String>,
) -> ApiResult<Response> {
    require_module_page_access(&request)?;
    let module_path = if record_id == "new" {
        "records/new".to_string()
    } else {
        let record_id =
            Uuid::parse_str(&record_id).map_err(|_| ApiError::NotFound("module route".into()))?;
        format!("records/{record_id}")
    };
    scoped_records_document(&state.pool, &request, &module_path, None).await
}

fn require_module_page_access(request: &AuthenticatedRequest) -> ApiResult<()> {
    if request
        .account
        .has_capability("tessara.reference.scoped-records:read")
    {
        Ok(())
    } else {
        request.require_capability("admin:all").map(|_| ())
    }
}

async fn proxy_manifest_module_document(
    State(state): State<AppState>,
    request: AuthenticatedRequest,
    Path(module_path): Path<String>,
) -> ApiResult<Response> {
    let requested_path = format!("/reference/{module_path}");
    let rows = sqlx::query(
        "SELECT instances.id,instances.installation_id,instances.enabled,instances.healthy,
                releases.manifest
         FROM module_instances instances
         JOIN module_releases releases ON releases.id=instances.release_id
         WHERE instances.identity_state='live' AND instances.installed=true
           AND instances.deployed=true AND instances.configured=true AND instances.ready=true
         ORDER BY instances.definition_id,instances.id",
    )
    .fetch_all(&state.pool)
    .await?;
    let mut matched = None;
    for row in rows {
        let instance_id: Uuid = row.try_get("id")?;
        let installation_id: Uuid = row.try_get("installation_id")?;
        let enabled: bool = row.try_get("enabled")?;
        let healthy: bool = row.try_get("healthy")?;
        let manifest: ModuleManifest = serde_json::from_value(row.try_get("manifest")?)
            .map_err(|error| ApiError::Internal(error.into()))?;
        for route in &manifest.browser_routes {
            if let Some(parameters) = match_browser_path(&route.path_template, &requested_path) {
                if matched.is_some() {
                    return Err(ApiError::ServiceUnavailable(
                        "module route registration is ambiguous".into(),
                    ));
                }
                matched = Some((
                    instance_id,
                    installation_id,
                    enabled,
                    healthy,
                    manifest.clone(),
                    route.clone(),
                    parameters,
                ));
            }
        }
    }
    let Some((instance_id, installation_id, enabled, healthy, manifest, route, parameters)) =
        matched
    else {
        return Err(ApiError::NotFound("module route".into()));
    };
    if !request
        .account
        .has_capability(route.required_capability.as_str())
    {
        return Err(restricted_authorization());
    }
    let bindings = capability_bindings(
        &state.pool,
        request.account.account_id,
        route.required_capability.as_str(),
    )
    .await?;
    if bindings.is_empty() {
        return Err(restricted_authorization());
    }
    if let Some(scope_parameter) = &route.organization_scope_parameter {
        let organization_id = parameters
            .get(scope_parameter)
            .and_then(|value| Uuid::parse_str(value).ok())
            .ok_or_else(restricted_authorization)?;
        if !bindings.iter().any(|binding| {
            binding.organization_root_id == organization_id
                || binding
                    .authorized_organization_ids
                    .contains(&organization_id)
        }) {
            return Err(restricted_authorization());
        }
    }
    if !enabled {
        return Ok(crate::module_unavailable_fallback_response());
    }
    let revisions = sqlx::query(
        "SELECT authorization_revision,organization_revision
         FROM core_security_revisions WHERE singleton=true",
    )
    .fetch_one(&state.pool)
    .await?;
    let authorization_revision = revisions.try_get::<i64, _>("authorization_revision")?;
    let organization_revision = revisions.try_get::<i64, _>("organization_revision")?;
    sync_module_security_state(
        manifest.definition_id.as_str(),
        installation_id,
        instance_id,
        authorization_revision,
        organization_revision,
        enabled,
        if healthy { "enabled" } else { "degraded" },
    )
    .await?;
    let correlation_id = Uuid::new_v4();
    let now = Utc::now();
    let navigation = manifest
        .navigation
        .iter()
        .filter(|item| {
            item.required_capabilities_any_of
                .iter()
                .any(|capability| request.account.has_capability(capability.as_str()))
        })
        .filter_map(|item| {
            let route = manifest.browser_routes.iter().find(|route| {
                route.destination == item.destination && !route.path_template.contains('{')
            })?;
            Some(NavigationProjectionV1 {
                contribution_id: item.id.clone(),
                label: item.label.clone(),
                href: route.path_template.clone(),
            })
        })
        .collect();
    let shell = protocol_signer(ProtocolSignaturePurposeV1::ShellContext)?
        .sign(ShellContextV1 {
            schema_version: 1,
            installation_id,
            module_definition_id: manifest.definition_id.clone(),
            module_instance_id: instance_id,
            original_actor: OriginalActorProjectionV1 {
                actor_id: request.account.account_id,
                display_name: request.account.display_name.clone(),
                email: Some(request.account.email.clone()),
            },
            theme: ShellThemeV1::Dark,
            navigation,
            return_destination: "/".into(),
            locale: "en-US".into(),
            time_zone: "UTC".into(),
            correlation_id,
            document_state: if healthy {
                ShellDocumentStateV1::Active
            } else {
                ShellDocumentStateV1::Degraded
            },
            issued_at: now,
            expires_at: now + Duration::seconds(60),
        })
        .map_err(|error| ApiError::Internal(error.into()))?;
    let grant = protocol_signer(ProtocolSignaturePurposeV1::AuthorizationGrant)?
        .sign(AuthorizationGrantV1 {
            schema_version: 1,
            installation_id,
            original_actor_id: request.account.account_id,
            presenting_service: ModuleDefinitionId::new("tessara.core")
                .map_err(|error| ApiError::Internal(error.into()))?,
            audience_module_instance_id: instance_id,
            dependency_binding: DependencyBindingKey::new("tessara.core.module-document")
                .map_err(|error| ApiError::Internal(error.into()))?,
            functional_contract: route.functional_contract,
            action: route.authorization_action,
            operation: AuthorizationGrantOperationV1::Read,
            capability_scope_bindings: bindings,
            resource_assertion: None,
            delegation_basis: Vec::new(),
            authorization_revision: authorization_revision as u64,
            organization_revision: organization_revision as u64,
            jti: Uuid::new_v4(),
            issued_at: now,
            expires_at: now + Duration::seconds(60),
        })
        .map_err(|error| ApiError::Internal(error.into()))?;
    let endpoint = match module_control_url(manifest.definition_id.as_str()) {
        Ok(endpoint) => endpoint,
        Err(_) => return Ok(crate::module_unavailable_fallback_response()),
    };
    let response = match reqwest::Client::new()
        .get(format!("{endpoint}{requested_path}"))
        .header(
            "x-tessara-shell-context",
            URL_SAFE_NO_PAD.encode(
                serde_json::to_vec(&shell).map_err(|error| ApiError::Internal(error.into()))?,
            ),
        )
        .header(
            "x-tessara-authorization",
            URL_SAFE_NO_PAD.encode(
                serde_json::to_vec(&grant).map_err(|error| ApiError::Internal(error.into()))?,
            ),
        )
        .header("x-tessara-correlation-id", correlation_id.to_string())
        .send()
        .await
    {
        Ok(response) => response,
        Err(_) => return Ok(crate::module_unavailable_fallback_response()),
    };
    if response.status().is_server_error() {
        return Ok(crate::module_unavailable_fallback_response());
    }
    module_response(response, Some("no-store")).await
}

async fn proxy_manifest_module_asset(
    State(state): State<AppState>,
    Path((definition, release, digest, asset_path)): Path<(String, String, String, String)>,
) -> ApiResult<Response> {
    let row = sqlx::query(
        "SELECT releases.manifest
         FROM module_instances instances
         JOIN module_releases releases ON releases.id=instances.release_id
         WHERE instances.identity_state='live' AND instances.installed=true
           AND instances.deployed=true AND instances.definition_id=$1
           AND releases.version=$2
         ORDER BY instances.id
         LIMIT 1",
    )
    .bind(&definition)
    .bind(&release)
    .fetch_optional(&state.pool)
    .await?
    .ok_or_else(|| ApiError::NotFound("module asset".into()))?;
    let manifest: ModuleManifest = serde_json::from_value(row.try_get("manifest")?)
        .map_err(|error| ApiError::Internal(error.into()))?;
    let requested_path = format!("/_tessara/modules/{definition}/{release}/{digest}/{asset_path}");
    let asset = manifest
        .assets
        .iter()
        .find(|asset| asset.path == requested_path && asset.digest.as_str() == digest)
        .ok_or_else(|| ApiError::NotFound("module asset".into()))?;
    let endpoint = module_control_url(&definition)?;
    let response = reqwest::Client::new()
        .get(format!("{endpoint}{}", asset.path))
        .send()
        .await
        .map_err(|_| ApiError::ServiceUnavailable("module asset unavailable".into()))?;
    let response = module_response(response, Some("public, max-age=31536000, immutable")).await?;
    if response
        .headers()
        .get(header::CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        != Some(asset.content_type.as_str())
    {
        return Err(ApiError::ServiceUnavailable(
            "module asset content type mismatch".into(),
        ));
    }
    Ok(response)
}

fn match_browser_path(template: &str, requested: &str) -> Option<BTreeMap<String, String>> {
    let template_segments = template.trim_matches('/').split('/').collect::<Vec<_>>();
    let requested_segments = requested.trim_matches('/').split('/').collect::<Vec<_>>();
    if template_segments.len() != requested_segments.len() {
        return None;
    }
    let mut parameters = BTreeMap::new();
    for (template, requested) in template_segments.into_iter().zip(requested_segments) {
        if let Some(name) = template
            .strip_prefix('{')
            .and_then(|value| value.strip_suffix('}'))
        {
            if name.is_empty() || requested.is_empty() {
                return None;
            }
            parameters.insert(name.to_string(), requested.to_string());
        } else if template != requested {
            return None;
        }
    }
    Some(parameters)
}

async fn proxy_record_list(
    State(state): State<AppState>,
    request: AuthenticatedRequest,
) -> ApiResult<Response> {
    proxy_authorized_module_request(
        &state.pool,
        &request,
        "records.list",
        AuthorizationGrantOperationV1::Read,
        reqwest::Method::GET,
        "api/records",
        None,
    )
    .await
}

async fn proxy_record_detail(
    State(state): State<AppState>,
    request: AuthenticatedRequest,
    Path(record_id): Path<Uuid>,
) -> ApiResult<Response> {
    proxy_authorized_module_request(
        &state.pool,
        &request,
        "records.get",
        AuthorizationGrantOperationV1::Read,
        reqwest::Method::GET,
        &format!("api/records/{record_id}"),
        None,
    )
    .await
}

async fn proxy_record_create(
    State(state): State<AppState>,
    request: AuthenticatedRequest,
    body: Bytes,
) -> ApiResult<Response> {
    proxy_authorized_module_request(
        &state.pool,
        &request,
        "records.create",
        AuthorizationGrantOperationV1::Mutation,
        reqwest::Method::POST,
        "api/records",
        Some(body),
    )
    .await
}

async fn proxy_record_update(
    State(state): State<AppState>,
    request: AuthenticatedRequest,
    Path(record_id): Path<Uuid>,
    body: Bytes,
) -> ApiResult<Response> {
    proxy_authorized_module_request(
        &state.pool,
        &request,
        "records.update",
        AuthorizationGrantOperationV1::Mutation,
        reqwest::Method::PUT,
        &format!("api/records/{record_id}"),
        Some(body),
    )
    .await
}

async fn proxy_module_get(
    pool: &PgPool,
    request: &AuthenticatedRequest,
    path: &str,
    query: Option<&ScopedRecordsDirectoryQuery>,
) -> ApiResult<Response> {
    let (shell_context, correlation_id) = scoped_records_shell_context(pool, request).await?;
    let encoded = URL_SAFE_NO_PAD.encode(
        serde_json::to_vec(&shell_context).map_err(|error| ApiError::Internal(error.into()))?,
    );
    let client = reqwest::Client::new();
    let mut outbound = client
        .get(format!("{}/{}", scoped_records_url(), path))
        .header("x-tessara-shell-context", encoded)
        .header("x-tessara-correlation-id", correlation_id.to_string())
        .header(
            "x-tessara-original-path",
            format!(
                "/reference/scoped-records{}",
                if path.is_empty() {
                    String::new()
                } else {
                    format!("/{path}")
                }
            ),
        )
        .header(
            "x-tessara-organization-access",
            URL_SAFE_NO_PAD.encode(
                serde_json::to_vec(&scoped_records_organization_access(pool, request).await?)
                    .map_err(|error| ApiError::Internal(error.into()))?,
            ),
        );
    if let Some(query) = query {
        outbound = outbound.query(query);
    }
    let content_action = if path.is_empty() {
        Some("records.list")
    } else if path.starts_with("records/") && !path.ends_with("/new") {
        Some("records.get")
    } else {
        None
    };
    if let Some(action) = content_action {
        let authorization = scoped_records_authorization(
            pool,
            request,
            action,
            AuthorizationGrantOperationV1::Read,
        )
        .await?;
        outbound = outbound.header(
            "x-tessara-authorization",
            URL_SAFE_NO_PAD.encode(
                serde_json::to_vec(&authorization)
                    .map_err(|error| ApiError::Internal(error.into()))?,
            ),
        );
    }
    let response = outbound
        .send()
        .await
        .map_err(|_| ApiError::NotFound("module route unavailable".into()))?;
    module_response(response, Some("no-store")).await
}

#[derive(Deserialize)]
struct ScopedRecordForm {
    label: String,
    organization_owner_id: Uuid,
    idempotency_key: String,
}

async fn create_scoped_record_form(
    State(state): State<AppState>,
    request: AuthenticatedRequest,
    Form(input): Form<ScopedRecordForm>,
) -> ApiResult<Response> {
    let response = proxy_authorized_module_request(
        &state.pool,
        &request,
        "records.create",
        AuthorizationGrantOperationV1::Mutation,
        reqwest::Method::POST,
        "api/records",
        Some(Bytes::from(
            serde_json::to_vec(&tessara_reference_scoped_records::RecordInput {
                label: input.label,
                organization_owner_id: input.organization_owner_id,
                idempotency_key: input.idempotency_key,
            })
            .map_err(|error| ApiError::Internal(error.into()))?,
        )),
    )
    .await?;
    if response.status().is_success() {
        Ok(Redirect::to("/reference/scoped-records").into_response())
    } else {
        Ok(response)
    }
}

async fn update_scoped_record_form(
    State(state): State<AppState>,
    request: AuthenticatedRequest,
    Path(record_id): Path<Uuid>,
    Form(input): Form<ScopedRecordForm>,
) -> ApiResult<Response> {
    let response = proxy_authorized_module_request(
        &state.pool,
        &request,
        "records.update",
        AuthorizationGrantOperationV1::Mutation,
        reqwest::Method::PUT,
        &format!("api/records/{record_id}"),
        Some(Bytes::from(
            serde_json::to_vec(&tessara_reference_scoped_records::RecordInput {
                label: input.label,
                organization_owner_id: input.organization_owner_id,
                idempotency_key: input.idempotency_key,
            })
            .map_err(|error| ApiError::Internal(error.into()))?,
        )),
    )
    .await?;
    if response.status().is_success() {
        Ok(Redirect::to(&format!("/reference/scoped-records/records/{record_id}")).into_response())
    } else {
        Ok(response)
    }
}

async fn scoped_records_document(
    pool: &PgPool,
    request: &AuthenticatedRequest,
    path: &str,
    query: Option<&ScopedRecordsDirectoryQuery>,
) -> ApiResult<Response> {
    proxy_module_get(pool, request, path, query).await
}

async fn scoped_records_organization_access(
    pool: &PgPool,
    request: &AuthenticatedRequest,
) -> ApiResult<Vec<tessara_reference_scoped_records::OrganizationAccessProjectionV1>> {
    let read = capability_bindings(
        pool,
        request.account.account_id,
        tessara_reference_scoped_records::READ_CAPABILITY,
    )
    .await?;
    let manage = capability_bindings(
        pool,
        request.account.account_id,
        tessara_reference_scoped_records::MANAGE_CAPABILITY,
    )
    .await?;
    let read_ids = read
        .iter()
        .flat_map(|binding| {
            std::iter::once(binding.organization_root_id)
                .chain(binding.authorized_organization_ids.iter().copied())
        })
        .collect::<std::collections::BTreeSet<_>>();
    let manage_ids = manage
        .iter()
        .flat_map(|binding| {
            std::iter::once(binding.organization_root_id)
                .chain(binding.authorized_organization_ids.iter().copied())
        })
        .collect::<std::collections::BTreeSet<_>>();
    let rows = sqlx::query("SELECT id,name FROM nodes WHERE id=ANY($1) ORDER BY name,id")
        .bind(read_ids.iter().copied().collect::<Vec<_>>())
        .fetch_all(pool)
        .await?;
    rows.into_iter()
        .map(|row| {
            let organization_id: Uuid = row.try_get("id")?;
            Ok(
                tessara_reference_scoped_records::OrganizationAccessProjectionV1 {
                    organization_id,
                    label: row.try_get("name")?,
                    can_manage: manage_ids.contains(&organization_id),
                },
            )
        })
        .collect::<Result<Vec<_>, sqlx::Error>>()
        .map_err(ApiError::from)
}

async fn scoped_records_shell_context(
    pool: &PgPool,
    request: &AuthenticatedRequest,
) -> ApiResult<(SignedEnvelopeV1<ShellContextV1>, Uuid)> {
    let instance = sqlx::query(
        "SELECT id,installation_id,enabled,healthy
         FROM module_instances
         WHERE definition_id=$1 AND identity_state='live' AND installed=true
           AND deployed=true AND configured=true AND ready=true",
    )
    .bind(tessara_reference_scoped_records::MODULE_DEFINITION_ID)
    .fetch_optional(pool)
    .await?
    .ok_or_else(restricted_authorization)?;
    let module_instance_id: Uuid = instance.try_get("id")?;
    let installation_id: Uuid = instance.try_get("installation_id")?;
    let enabled: bool = instance.try_get("enabled")?;
    let healthy: bool = instance.try_get("healthy")?;
    let revisions = sqlx::query(
        "SELECT authorization_revision,organization_revision
         FROM core_security_revisions WHERE singleton=true",
    )
    .fetch_one(pool)
    .await?;
    let authorization_revision = revisions.try_get::<i64, _>("authorization_revision")?;
    let organization_revision = revisions.try_get::<i64, _>("organization_revision")?;
    let module_document_state = if !enabled {
        "disabled"
    } else if !healthy {
        "degraded"
    } else {
        "enabled"
    };
    sync_scoped_records_security_state(
        installation_id,
        module_instance_id,
        authorization_revision,
        organization_revision,
        enabled,
        module_document_state,
    )
    .await?;
    let correlation_id = Uuid::new_v4();
    let now = Utc::now();
    let document_state = if !enabled {
        ShellDocumentStateV1::Disabled
    } else if !healthy {
        ShellDocumentStateV1::Degraded
    } else {
        ShellDocumentStateV1::Active
    };
    let context = ShellContextV1 {
        schema_version: 1,
        installation_id,
        module_definition_id: ModuleDefinitionId::new(
            tessara_reference_scoped_records::MODULE_DEFINITION_ID,
        )
        .map_err(|error| ApiError::Internal(error.into()))?,
        module_instance_id,
        original_actor: OriginalActorProjectionV1 {
            actor_id: request.account.account_id,
            display_name: request.account.display_name.clone(),
            email: Some(request.account.email.clone()),
        },
        theme: ShellThemeV1::Dark,
        navigation: vec![
            NavigationProjectionV1 {
                contribution_id: NavigationContributionId::new("tessara.core.home")
                    .map_err(|error| ApiError::Internal(error.into()))?,
                label: "Home".into(),
                href: "/".into(),
            },
            NavigationProjectionV1 {
                contribution_id: NavigationContributionId::new(
                    "tessara.reference.scoped-records.directory",
                )
                .map_err(|error| ApiError::Internal(error.into()))?,
                label: "Scoped Records".into(),
                href: "/reference/scoped-records".into(),
            },
        ],
        return_destination: "/".into(),
        locale: "en-US".into(),
        time_zone: "UTC".into(),
        correlation_id,
        document_state,
        issued_at: now,
        expires_at: now + Duration::seconds(60),
    };
    let envelope = protocol_signer(ProtocolSignaturePurposeV1::ShellContext)?
        .sign(context)
        .map_err(|error| ApiError::Internal(error.into()))?;
    Ok((envelope, correlation_id))
}

async fn proxy_authorized_module_request(
    pool: &PgPool,
    request: &AuthenticatedRequest,
    action: &str,
    operation: AuthorizationGrantOperationV1,
    method: reqwest::Method,
    path: &str,
    body: Option<Bytes>,
) -> ApiResult<Response> {
    let envelope = scoped_records_authorization(pool, request, action, operation).await?;
    let encoded = URL_SAFE_NO_PAD
        .encode(serde_json::to_vec(&envelope).map_err(|error| ApiError::Internal(error.into()))?);
    let client = reqwest::Client::new();
    let mut outbound = client
        .request(method, format!("{}/{}", scoped_records_url(), path))
        .header("x-tessara-authorization", encoded)
        .header(header::CONTENT_TYPE.as_str(), "application/json");
    if let Some(body) = body {
        outbound = outbound.body(body.to_vec());
    }
    let response = outbound
        .send()
        .await
        .map_err(|_| ApiError::NotFound("module route unavailable".into()))?;
    module_response(response, None).await
}

async fn scoped_records_authorization(
    pool: &PgPool,
    request: &AuthenticatedRequest,
    action: &str,
    operation: AuthorizationGrantOperationV1,
) -> ApiResult<SignedEnvelopeV1<AuthorizationGrantV1>> {
    let instance = sqlx::query(
        "SELECT id,installation_id,definition_id,enabled
         FROM module_instances
         WHERE definition_id=$1 AND identity_state='live' AND installed=true
           AND deployed=true AND configured=true AND ready=true AND healthy=true",
    )
    .bind(tessara_reference_scoped_records::MODULE_DEFINITION_ID)
    .fetch_optional(pool)
    .await?
    .ok_or_else(restricted_authorization)?;
    let instance_id: Uuid = instance.try_get("id")?;
    let installation_id: Uuid = instance.try_get("installation_id")?;
    let enabled: bool = instance.try_get("enabled")?;
    if !enabled {
        return Err(ApiError::NotFound("module route unavailable".into()));
    }
    let operation_name = operation_text(operation);
    let required_capability: String = sqlx::query_scalar(
        "SELECT required_capability FROM core_module_action_declarations
         WHERE target_definition_id=$1 AND dependency_binding=$2
           AND functional_contract=$3 AND action=$4 AND operation=$5",
    )
    .bind(tessara_reference_scoped_records::MODULE_DEFINITION_ID)
    .bind("tessara.core.scoped-records")
    .bind("tessara.reference.scoped-records.record")
    .bind(action)
    .bind(operation_name)
    .fetch_optional(pool)
    .await?
    .ok_or_else(restricted_authorization)?;
    let bindings =
        capability_bindings(pool, request.account.account_id, &required_capability).await?;
    if bindings.is_empty() {
        return Err(restricted_authorization());
    }
    let revisions = sqlx::query(
        "SELECT authorization_revision,organization_revision
         FROM core_security_revisions WHERE singleton=true",
    )
    .fetch_one(pool)
    .await?;
    let authorization_revision = revisions.try_get::<i64, _>("authorization_revision")?;
    let organization_revision = revisions.try_get::<i64, _>("organization_revision")?;
    sync_scoped_records_security_state(
        installation_id,
        instance_id,
        authorization_revision,
        organization_revision,
        true,
        "enabled",
    )
    .await?;
    let now = Utc::now();
    let grant = AuthorizationGrantV1 {
        schema_version: 1,
        installation_id,
        original_actor_id: request.account.account_id,
        presenting_service: ModuleDefinitionId::new("tessara.core")
            .map_err(|error| ApiError::Internal(error.into()))?,
        audience_module_instance_id: instance_id,
        dependency_binding: DependencyBindingKey::new("tessara.core.scoped-records")
            .map_err(|error| ApiError::Internal(error.into()))?,
        functional_contract: FunctionalContractId::new("tessara.reference.scoped-records.record")
            .map_err(|error| ApiError::Internal(error.into()))?,
        action: action.into(),
        operation,
        capability_scope_bindings: bindings,
        resource_assertion: None,
        delegation_basis: Vec::new(),
        authorization_revision: authorization_revision as u64,
        organization_revision: organization_revision as u64,
        jti: Uuid::new_v4(),
        issued_at: now,
        expires_at: now
            + Duration::seconds(match operation {
                AuthorizationGrantOperationV1::Read => 60,
                AuthorizationGrantOperationV1::Mutation => 30,
            }),
    };
    protocol_signer(ProtocolSignaturePurposeV1::AuthorizationGrant)?
        .sign(grant)
        .map_err(|error| ApiError::Internal(error.into()))
}

async fn sync_scoped_records_security_state(
    installation_id: Uuid,
    instance_id: Uuid,
    authorization_revision: i64,
    organization_revision: i64,
    enabled: bool,
    document_state: &str,
) -> ApiResult<()> {
    sync_module_security_state(
        tessara_reference_scoped_records::MODULE_DEFINITION_ID,
        installation_id,
        instance_id,
        authorization_revision,
        organization_revision,
        enabled,
        document_state,
    )
    .await
}

async fn sync_module_security_state(
    definition: &str,
    installation_id: Uuid,
    instance_id: Uuid,
    authorization_revision: i64,
    organization_revision: i64,
    enabled: bool,
    document_state: &str,
) -> ApiResult<()> {
    let base_url = module_control_url(definition)?;
    reqwest::Client::new()
        .put(format!("{base_url}/api/private/security-state"))
        .header(
            "x-tessara-module-control-key",
            std::env::var("TESSARA_MODULE_CONTROL_SHARED_KEY")
                .unwrap_or_else(|_| "development-module-control-only".into()),
        )
        .json(&serde_json::json!({
            "schema_version": 1,
            "installation_id": installation_id,
            "module_instance_id": instance_id,
            "authorization_revision": authorization_revision,
            "organization_revision": organization_revision,
            "enabled": enabled,
            "document_state": document_state
        }))
        .send()
        .await
        .map_err(|_| ApiError::NotFound("module route unavailable".into()))?
        .error_for_status()
        .map_err(|_| ApiError::NotFound("module route unavailable".into()))?;
    Ok(())
}

async fn module_response(
    response: reqwest::Response,
    cache_control: Option<&'static str>,
) -> ApiResult<Response> {
    let status = StatusCode::from_u16(response.status().as_u16())
        .map_err(|error| ApiError::Internal(error.into()))?;
    let content_type = response
        .headers()
        .get(reqwest::header::CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .unwrap_or("application/octet-stream")
        .to_string();
    let bytes = response
        .bytes()
        .await
        .map_err(|error| ApiError::Internal(error.into()))?;
    let mut builder = Response::builder()
        .status(status)
        .header(header::CONTENT_TYPE, content_type);
    if let Some(cache_control) = cache_control {
        builder = builder.header(header::CACHE_CONTROL, cache_control);
    }
    builder
        .body(Body::from(bytes))
        .map_err(|error| ApiError::Internal(error.into()))
}

fn scoped_records_url() -> String {
    std::env::var("TESSARA_SCOPED_RECORDS_URL")
        .unwrap_or_else(|_| "http://scoped-records:8090".into())
        .trim_end_matches('/')
        .to_string()
}

fn module_control_url(definition: &str) -> ApiResult<String> {
    let configured = std::env::var("TESSARA_MODULE_CONTROL_ENDPOINTS")
        .ok()
        .filter(|value| !value.trim().is_empty())
        .map(|value| parse_module_control_endpoints(&value))
        .transpose()?
        .and_then(|endpoints| endpoints.get(definition).cloned());
    let endpoint = configured.or_else(|| match definition {
        tessara_reference_scoped_records::MODULE_DEFINITION_ID => Some(scoped_records_url()),
        "tessara.dashboards" => Some(dashboard_module_url()),
        _ => None,
    });
    endpoint
        .filter(|endpoint| !endpoint.trim().is_empty())
        .map(|endpoint| endpoint.trim_end_matches('/').to_string())
        .ok_or_else(|| {
            ApiError::BadRequest(format!(
                "module {definition} does not have a configured control endpoint"
            ))
        })
}

fn parse_module_control_endpoints(value: &str) -> ApiResult<BTreeMap<String, String>> {
    serde_json::from_str(value).map_err(|_| {
        ApiError::Internal(anyhow::anyhow!(
            "TESSARA_MODULE_CONTROL_ENDPOINTS must be a JSON object"
        ))
    })
}

async fn update_module_configuration(
    State(state): State<AppState>,
    request: AuthenticatedRequest,
    Path(instance_id): Path<Uuid>,
    Json(input): Json<serde_json::Value>,
) -> ApiResult<Json<serde_json::Value>> {
    request.require_capability("admin:all")?;
    let validation = persist_module_configuration(&state.pool, instance_id, input).await?;
    Ok(Json(validation))
}

async fn update_module_configuration_form(
    State(state): State<AppState>,
    request: AuthenticatedRequest,
    Path(instance_id): Path<Uuid>,
    Form(input): Form<BTreeMap<String, String>>,
) -> ApiResult<Redirect> {
    request.require_capability("admin:all")?;
    let module = sqlx::query(
        "SELECT instances.definition_id,releases.manifest
         FROM module_instances instances
         JOIN module_releases releases ON releases.id=instances.release_id
         WHERE instances.id=$1",
    )
    .bind(instance_id)
    .fetch_optional(&state.pool)
    .await?
    .ok_or_else(|| ApiError::NotFound(format!("module instance {instance_id}")))?;
    let definition: String = module.try_get("definition_id")?;
    let manifest: Value = module.try_get("manifest")?;
    let schema = manifest
        .get("configuration_schema")
        .ok_or_else(|| ApiError::BadRequest("module configuration schema is unavailable".into()))?;
    let payload = configuration_form_payload(schema, input)?;
    let validation = persist_module_configuration(&state.pool, instance_id, payload).await?;
    if validation.get("valid").and_then(Value::as_bool) != Some(true) {
        return Err(ApiError::BadRequest(
            validation
                .get("findings")
                .and_then(Value::as_array)
                .and_then(|findings| findings.first())
                .and_then(|finding| finding.get("message"))
                .and_then(Value::as_str)
                .unwrap_or("Module configuration is invalid.")
                .into(),
        ));
    }
    Ok(Redirect::to(&format!(
        "/administration/modules/{definition}#configuration"
    )))
}

#[derive(Deserialize)]
struct ModuleEnablementForm {
    enabled: bool,
}

async fn update_module_enablement_form(
    State(state): State<AppState>,
    request: AuthenticatedRequest,
    Path(instance_id): Path<Uuid>,
    Form(input): Form<ModuleEnablementForm>,
) -> ApiResult<Redirect> {
    request.require_capability("admin:all")?;
    let instance = sqlx::query(
        "SELECT definition_id,installation_id,installed,deployed,configured,healthy
         FROM module_instances WHERE id=$1 AND identity_state='live'",
    )
    .bind(instance_id)
    .fetch_optional(&state.pool)
    .await?
    .ok_or_else(|| ApiError::NotFound(format!("module instance {instance_id}")))?;
    let definition: String = instance.try_get("definition_id")?;
    let installation_id: Uuid = instance.try_get("installation_id")?;
    let installed: bool = instance.try_get("installed")?;
    let deployed: bool = instance.try_get("deployed")?;
    let configured: bool = instance.try_get("configured")?;
    let healthy: bool = instance.try_get("healthy")?;
    if input.enabled && !(installed && deployed && configured && healthy) {
        return Err(ApiError::BadRequest(
            "the module must be installed, deployed, configured, and healthy before it can be enabled"
                .into(),
        ));
    }
    let revisions = sqlx::query(
        "SELECT authorization_revision,organization_revision
         FROM core_security_revisions WHERE singleton=true",
    )
    .fetch_one(&state.pool)
    .await?;
    let authorization_revision = revisions.try_get::<i64, _>("authorization_revision")?;
    let organization_revision = revisions.try_get::<i64, _>("organization_revision")?;
    sync_module_enablement(
        &definition,
        installation_id,
        instance_id,
        authorization_revision,
        organization_revision,
        input.enabled,
    )
    .await?;
    sqlx::query(
        "UPDATE module_instances SET enabled=$2,last_observed_at=now()
         WHERE id=$1 AND identity_state='live'",
    )
    .bind(instance_id)
    .bind(input.enabled)
    .execute(&state.pool)
    .await?;
    Ok(Redirect::to(&format!(
        "/administration/modules/{definition}#configuration"
    )))
}

async fn sync_module_enablement(
    definition: &str,
    installation_id: Uuid,
    instance_id: Uuid,
    authorization_revision: i64,
    organization_revision: i64,
    enabled: bool,
) -> ApiResult<()> {
    let document_state = module_document_state(enabled);
    sync_module_security_state(
        definition,
        installation_id,
        instance_id,
        authorization_revision,
        organization_revision,
        enabled,
        document_state,
    )
    .await
}

fn module_document_state(enabled: bool) -> &'static str {
    if enabled { "enabled" } else { "disabled" }
}

async fn persist_module_configuration(
    pool: &PgPool,
    instance_id: Uuid,
    input: serde_json::Value,
) -> ApiResult<serde_json::Value> {
    let definition: String =
        sqlx::query_scalar("SELECT definition_id FROM module_instances WHERE id=$1")
            .bind(instance_id)
            .fetch_optional(pool)
            .await?
            .ok_or_else(|| ApiError::NotFound(format!("module instance {instance_id}")))?;
    let base_url = module_control_url(&definition)?;
    let validation: Value = reqwest::Client::new()
        .post(format!("{base_url}/api/configuration/validate"))
        .header(
            "x-tessara-module-control-key",
            std::env::var("TESSARA_MODULE_CONTROL_SHARED_KEY")
                .unwrap_or_else(|_| "development-module-control-only".into()),
        )
        .json(&input)
        .send()
        .await
        .map_err(|_| {
            ApiError::ServiceUnavailable("module configuration validator unavailable".into())
        })?
        .error_for_status()
        .map_err(|_| {
            ApiError::ServiceUnavailable("module configuration validator unavailable".into())
        })?
        .json()
        .await
        .map_err(|error| ApiError::Internal(error.into()))?;
    if let Some(normalized) = validation
        .get("normalized")
        .filter(|_| validation.get("valid").and_then(Value::as_bool) == Some(true))
    {
        let applied = reqwest::Client::new()
            .put(format!("{base_url}/api/configuration"))
            .header(
                "x-tessara-module-control-key",
                std::env::var("TESSARA_MODULE_CONTROL_SHARED_KEY")
                    .unwrap_or_else(|_| "development-module-control-only".into()),
            )
            .json(normalized)
            .send()
            .await
            .map_err(|_| {
                ApiError::ServiceUnavailable("module configuration route unavailable".into())
            })?
            .error_for_status()
            .map_err(|_| {
                ApiError::ServiceUnavailable("module configuration route unavailable".into())
            })?
            .json::<serde_json::Value>()
            .await
            .map_err(|error| ApiError::Internal(error.into()))?;
        if applied.get("valid").and_then(serde_json::Value::as_bool) != Some(true)
            || applied.get("normalized") != Some(normalized)
        {
            return Err(ApiError::BadRequest(
                "the module rejected its normalized configuration".into(),
            ));
        }
        sqlx::query(
            "UPDATE module_instances SET configuration=$2,configured=true,last_observed_at=now()
             WHERE id=$1",
        )
        .bind(instance_id)
        .bind(normalized)
        .execute(pool)
        .await?;
    }
    Ok(validation)
}

fn configuration_form_payload(
    schema: &Value,
    mut fields: BTreeMap<String, String>,
) -> ApiResult<Value> {
    let properties = schema
        .get("properties")
        .and_then(Value::as_object)
        .ok_or_else(|| ApiError::BadRequest("module configuration schema is invalid".into()))?;
    let required = schema
        .get("required")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(Value::as_str)
        .collect::<Vec<_>>();
    let schema_version = fields
        .remove("schema_version")
        .unwrap_or_else(|| "1".into())
        .parse::<u16>()
        .map_err(|_| ApiError::BadRequest("schema_version must be an integer".into()))?;
    let mut configuration =
        serde_json::Map::from_iter([("schema_version".into(), json!(schema_version))]);
    for (name, property) in properties {
        let Some(raw) = fields.remove(name) else {
            if required.contains(&name.as_str()) {
                return Err(ApiError::BadRequest(format!(
                    "configuration field {name} is required"
                )));
            }
            continue;
        };
        let value = match property.get("type").and_then(Value::as_str) {
            Some("string") | None => Value::String(raw),
            Some("integer") => Value::Number(
                raw.parse::<i64>()
                    .map_err(|_| {
                        ApiError::BadRequest(format!(
                            "configuration field {name} must be an integer"
                        ))
                    })?
                    .into(),
            ),
            Some("number") => serde_json::Number::from_f64(raw.parse::<f64>().map_err(|_| {
                ApiError::BadRequest(format!("configuration field {name} must be a number"))
            })?)
            .map(Value::Number)
            .ok_or_else(|| {
                ApiError::BadRequest(format!("configuration field {name} must be finite"))
            })?,
            Some("boolean") => Value::Bool(raw.parse::<bool>().map_err(|_| {
                ApiError::BadRequest(format!("configuration field {name} must be a boolean"))
            })?),
            Some(kind) => {
                return Err(ApiError::BadRequest(format!(
                    "configuration field {name} uses unsupported type {kind}"
                )));
            }
        };
        configuration.insert(name.clone(), value);
    }
    if !fields.is_empty() {
        return Err(ApiError::BadRequest(format!(
            "unknown module configuration fields: {}",
            fields.keys().cloned().collect::<Vec<_>>().join(", ")
        )));
    }
    Ok(Value::Object(configuration))
}

fn dashboard_module_url() -> String {
    std::env::var("TESSARA_DASHBOARD_MODULE_URL")
        .unwrap_or_else(|_| "http://dashboards:8091".into())
        .trim_end_matches('/')
        .to_string()
}

#[derive(Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct AdministratorRedemptionRequest {
    schema_version: u16,
    installation_id: Uuid,
    claim_id: Uuid,
    generation: u32,
    claim_kind: AdministratorEnrollmentClaimKindV1,
    claim_secret: String,
    idempotency_key: String,
    identity_path: String,
    email: Option<String>,
    display_name: Option<String>,
    password: Option<String>,
    external_assertion: Option<SignedEnvelopeV1<ExternalIdentityAssertionV1>>,
}

#[derive(Deserialize)]
struct LocalEnrollmentForm {
    schema_version: u16,
    claim_id: Uuid,
    generation: u32,
    claim_kind: AdministratorEnrollmentClaimKindV1,
    claim_secret: String,
    email: String,
    display_name: String,
    password: String,
}

async fn redeem_local_form(
    State(state): State<AppState>,
    Form(form): Form<LocalEnrollmentForm>,
) -> Response {
    let installation_id: Uuid =
        match sqlx::query_scalar("SELECT id FROM application_installations WHERE singleton=true")
            .fetch_one(&state.pool)
            .await
        {
            Ok(value) => value,
            Err(_) => {
                return (
                    StatusCode::NOT_FOUND,
                    Html(enrollment_unavailable_document(
                        "The enrollment request could not be completed.",
                    )),
                )
                    .into_response();
            }
        };
    let idempotency_key = format!(
        "local:{}:{}:{}",
        form.claim_id,
        form.generation,
        form.email.trim().to_lowercase()
    );
    let result = redeem(
        &state,
        AdministratorRedemptionRequest {
            schema_version: form.schema_version,
            installation_id,
            claim_id: form.claim_id,
            generation: form.generation,
            claim_kind: form.claim_kind,
            claim_secret: form.claim_secret,
            idempotency_key,
            identity_path: "local".into(),
            email: Some(form.email),
            display_name: Some(form.display_name),
            password: Some(form.password),
            external_assertion: None,
        },
    )
    .await;
    match result {
        Ok(_) => Html(enrollment_success_document()).into_response(),
        Err(_) => (
            StatusCode::NOT_FOUND,
            Html(enrollment_unavailable_document(
                "The enrollment request could not be completed.",
            )),
        )
            .into_response(),
    }
}

async fn redeem_administrator(
    State(state): State<AppState>,
    Json(payload): Json<AdministratorRedemptionRequest>,
) -> ApiResult<Json<SignedEnvelopeV1<EnrollmentRedemptionResultV1>>> {
    redeem(&state, payload).await.map(Json)
}

async fn redeem(
    state: &AppState,
    payload: AdministratorRedemptionRequest,
) -> ApiResult<SignedEnvelopeV1<EnrollmentRedemptionResultV1>> {
    validate_redemption_shape(&payload)?;
    require_installation(&state.pool, payload.installation_id).await?;
    let reservation_id = deterministic_reservation_id(
        payload.installation_id,
        payload.claim_id,
        payload.generation,
        &payload.idempotency_key,
    );
    if let Some(envelope) = existing_redemption(
        &state.pool,
        payload.installation_id,
        payload.claim_id,
        payload.generation,
        reservation_id,
    )
    .await?
    {
        finalize_external_claim(&envelope).await?;
        return Ok(envelope);
    }
    if viable_administrator_exists(&state.pool).await? {
        return Err(enrollment_unavailable());
    }
    let reservation = reserve_external_claim(&payload, reservation_id).await?;
    if reservation.expires_at <= Utc::now() {
        return Err(enrollment_unavailable());
    }

    let mut transaction = state.pool.begin().await?;
    let existing = sqlx::query(
        "SELECT reservation_id,result_envelope
         FROM administrator_enrollment_redemptions
         WHERE installation_id=$1 AND claim_id=$2 AND generation=$3 FOR UPDATE",
    )
    .bind(payload.installation_id)
    .bind(payload.claim_id)
    .bind(payload.generation as i32)
    .fetch_optional(&mut *transaction)
    .await?;
    if let Some(row) = existing {
        if row.try_get::<Uuid, _>("reservation_id")? != reservation_id {
            return Err(enrollment_unavailable());
        }
        let envelope: SignedEnvelopeV1<EnrollmentRedemptionResultV1> =
            serde_json::from_value(row.try_get("result_envelope")?)
                .map_err(|_| enrollment_unavailable())?;
        transaction.commit().await?;
        finalize_external_claim(&envelope).await?;
        return Ok(envelope);
    }

    let role_id: Uuid = sqlx::query_scalar(
        "SELECT designated_enrollment_role_id
         FROM core_administration_state WHERE singleton=true FOR UPDATE",
    )
    .fetch_one(&mut *transaction)
    .await?;
    let role_complete: bool = sqlx::query_scalar(
        "SELECT COUNT(DISTINCT c.key)=$2 FROM role_capabilities rc
         JOIN capabilities c ON c.id=rc.capability_id
         WHERE rc.role_id=$1 AND c.key=ANY($3)",
    )
    .bind(role_id)
    .bind(FLOOR_CAPABILITIES.len() as i64)
    .bind(FLOOR_CAPABILITIES)
    .fetch_one(&mut *transaction)
    .await?;
    if !role_complete {
        return Err(enrollment_unavailable());
    }

    let account_id = match payload.identity_path.as_str() {
        "local" => create_local_enrollment_identity(&mut transaction, &payload).await?,
        "fixture_external" => {
            create_external_enrollment_identity(&mut transaction, &payload).await?
        }
        _ => return Err(enrollment_unavailable()),
    };
    sqlx::query(
        "INSERT INTO role_assignments (account_id,role_id,node_id)
         VALUES ($1,$2,NULL) ON CONFLICT DO NOTHING",
    )
    .bind(account_id)
    .bind(role_id)
    .execute(&mut *transaction)
    .await?;
    let completed_at = Utc::now();
    let result = EnrollmentRedemptionResultV1 {
        schema_version: 1,
        installation_id: payload.installation_id,
        claim_id: payload.claim_id,
        generation: payload.generation,
        reservation_id,
        account_id,
        role_id,
        completed_at,
    };
    let envelope = protocol_signer(ProtocolSignaturePurposeV1::EnrollmentRedemption)?
        .sign(result)
        .map_err(|_| enrollment_unavailable())?;
    sqlx::query(
        "INSERT INTO administrator_enrollment_redemptions
         (installation_id,claim_id,generation,reservation_id,claim_kind,identity_path,
          account_id,role_id,completed_at,result_envelope)
         VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10)",
    )
    .bind(payload.installation_id)
    .bind(payload.claim_id)
    .bind(payload.generation as i32)
    .bind(reservation_id)
    .bind(claim_kind_text(payload.claim_kind))
    .bind(&payload.identity_path)
    .bind(account_id)
    .bind(role_id)
    .bind(completed_at)
    .bind(serde_json::to_value(&envelope).map_err(|_| enrollment_unavailable())?)
    .execute(&mut *transaction)
    .await?;
    sqlx::query(
        "UPDATE core_administration_state
         SET has_ever_had_viable_administrator=true,updated_at=now()
         WHERE singleton=true",
    )
    .execute(&mut *transaction)
    .await?;
    transaction.commit().await?;
    finalize_external_claim(&envelope).await?;
    Ok(envelope)
}

async fn existing_redemption(
    pool: &PgPool,
    installation_id: Uuid,
    claim_id: Uuid,
    generation: u32,
    reservation_id: Uuid,
) -> ApiResult<Option<SignedEnvelopeV1<EnrollmentRedemptionResultV1>>> {
    let row = sqlx::query(
        "SELECT reservation_id,result_envelope
         FROM administrator_enrollment_redemptions
         WHERE installation_id=$1 AND claim_id=$2 AND generation=$3",
    )
    .bind(installation_id)
    .bind(claim_id)
    .bind(generation as i32)
    .fetch_optional(pool)
    .await?;
    match row {
        None => Ok(None),
        Some(row) if row.try_get::<Uuid, _>("reservation_id")? == reservation_id => {
            serde_json::from_value(row.try_get("result_envelope")?)
                .map(Some)
                .map_err(|_| enrollment_unavailable())
        }
        Some(_) => Err(enrollment_unavailable()),
    }
}

fn validate_redemption_shape(payload: &AdministratorRedemptionRequest) -> ApiResult<()> {
    if payload.schema_version != 1
        || payload.generation == 0
        || payload.claim_secret.is_empty()
        || payload.idempotency_key.trim().is_empty()
        || payload.idempotency_key.len() > 200
    {
        return Err(enrollment_unavailable());
    }
    match payload.identity_path.as_str() {
        "local"
            if payload.external_assertion.is_none()
                && payload
                    .email
                    .as_deref()
                    .is_some_and(|value| !value.trim().is_empty())
                && payload
                    .display_name
                    .as_deref()
                    .is_some_and(|value| !value.trim().is_empty())
                && payload
                    .password
                    .as_deref()
                    .is_some_and(|value| value.len() >= 12) =>
        {
            Ok(())
        }
        "fixture_external"
            if payload.external_assertion.is_some()
                && payload.email.is_none()
                && payload.display_name.is_none()
                && payload.password.is_none() =>
        {
            Ok(())
        }
        _ => Err(enrollment_unavailable()),
    }
}

async fn create_local_enrollment_identity(
    transaction: &mut Transaction<'_, Postgres>,
    payload: &AdministratorRedemptionRequest,
) -> ApiResult<Uuid> {
    let email = payload.email.as_deref().unwrap().trim().to_lowercase();
    let display_name = payload.display_name.as_deref().unwrap().trim();
    let password_hash =
        crate::auth::hash_password_for_storage(payload.password.as_deref().unwrap())
            .map_err(|_| enrollment_unavailable())?;
    let account_id: Uuid = sqlx::query_scalar(
        "INSERT INTO accounts (email,display_name,is_active)
         VALUES ($1,$2,true) RETURNING id",
    )
    .bind(email)
    .bind(display_name)
    .fetch_one(&mut **transaction)
    .await
    .map_err(|_| enrollment_unavailable())?;
    sqlx::query(
        "INSERT INTO account_credentials
         (account_id,password_hash,password_scheme) VALUES ($1,$2,$3)",
    )
    .bind(account_id)
    .bind(password_hash)
    .bind(crate::auth::password_scheme())
    .execute(&mut **transaction)
    .await
    .map_err(|_| enrollment_unavailable())?;
    Ok(account_id)
}

async fn create_external_enrollment_identity(
    transaction: &mut Transaction<'_, Postgres>,
    payload: &AdministratorRedemptionRequest,
) -> ApiResult<Uuid> {
    let assertion = payload.external_assertion.as_ref().unwrap();
    fixture_external_verifier()?
        .verify(assertion)
        .map_err(|_| enrollment_unavailable())?;
    let identity = &assertion.payload;
    let now = Utc::now();
    if identity.schema_version != 1
        || identity.installation_id != payload.installation_id
        || identity.audience != "tessara.core.administrator-enrollment"
        || identity.external_subject.trim().is_empty()
        || identity.email.trim().is_empty()
        || identity.display_name.trim().is_empty()
        || identity.expires_at <= now
        || identity.issued_at > now
        || identity.expires_at - identity.issued_at > Duration::seconds(60)
    {
        return Err(enrollment_unavailable());
    }
    let account_id: Uuid = sqlx::query_scalar(
        "INSERT INTO accounts (email,display_name,is_active)
         VALUES ($1,$2,true) RETURNING id",
    )
    .bind(identity.email.trim().to_lowercase())
    .bind(identity.display_name.trim())
    .fetch_one(&mut **transaction)
    .await
    .map_err(|_| enrollment_unavailable())?;
    sqlx::query(
        "INSERT INTO external_identity_bindings
         (issuer,external_subject,account_id,is_usable,assertion_nonce)
         VALUES ($1,$2,$3,true,$4)",
    )
    .bind(&assertion.issuer)
    .bind(identity.external_subject.trim())
    .bind(account_id)
    .bind(identity.nonce)
    .execute(&mut **transaction)
    .await
    .map_err(|_| enrollment_unavailable())?;
    Ok(account_id)
}

async fn reserve_external_claim(
    payload: &AdministratorRedemptionRequest,
    reservation_id: Uuid,
) -> ApiResult<EnrollmentReservationV1> {
    let base = std::env::var("TESSARA_INSTALLATION_CONTROL_URL")
        .unwrap_or_else(|_| "http://installation-control:8075".into());
    reqwest::Client::new()
        .post(format!("{}/v1/reservations", base.trim_end_matches('/')))
        .header(
            "x-tessara-installation-control-key",
            installation_control_shared_key(),
        )
        .json(&serde_json::json!({
            "schema_version": 1,
            "installation_id": payload.installation_id,
            "claim_id": payload.claim_id,
            "generation": payload.generation,
            "claim_secret": payload.claim_secret,
            "reservation_id": reservation_id,
        }))
        .send()
        .await
        .map_err(|_| enrollment_unavailable())?
        .error_for_status()
        .map_err(|_| enrollment_unavailable())?
        .json()
        .await
        .map_err(|_| enrollment_unavailable())
}

async fn finalize_external_claim(
    result: &SignedEnvelopeV1<EnrollmentRedemptionResultV1>,
) -> ApiResult<()> {
    let base = std::env::var("TESSARA_INSTALLATION_CONTROL_URL")
        .unwrap_or_else(|_| "http://installation-control:8075".into());
    reqwest::Client::new()
        .post(format!("{}/v1/redemptions", base.trim_end_matches('/')))
        .header(
            "x-tessara-installation-control-key",
            installation_control_shared_key(),
        )
        .json(result)
        .send()
        .await
        .map_err(|_| enrollment_unavailable())?
        .error_for_status()
        .map_err(|_| enrollment_unavailable())?;
    Ok(())
}

fn deterministic_reservation_id(
    installation_id: Uuid,
    claim_id: Uuid,
    generation: u32,
    idempotency_key: &str,
) -> Uuid {
    use sha2::{Digest, Sha256};
    let mut digest = Sha256::new();
    digest.update(installation_id.as_bytes());
    digest.update(claim_id.as_bytes());
    digest.update(generation.to_be_bytes());
    digest.update(idempotency_key.as_bytes());
    let bytes: [u8; 16] = digest.finalize()[..16].try_into().unwrap();
    Uuid::from_bytes(bytes)
}

fn fixture_external_verifier() -> ApiResult<PurposeBoundVerifyingKeyV1> {
    let encoded = std::env::var("TESSARA_FIXTURE_EXTERNAL_PUBLIC_KEY")
        .map_err(|_| enrollment_unavailable())?;
    let bytes: [u8; 32] = URL_SAFE_NO_PAD
        .decode(encoded)
        .map_err(|_| enrollment_unavailable())?
        .try_into()
        .map_err(|_| enrollment_unavailable())?;
    PurposeBoundVerifyingKeyV1::from_public_bytes(
        "tessara.fixture-identity",
        "fixture-external-v1",
        ProtocolSignaturePurposeV1::FixtureExternalIdentity,
        bytes,
    )
    .map_err(|_| enrollment_unavailable())
}

fn installation_control_shared_key() -> String {
    std::env::var("TESSARA_INSTALLATION_CONTROL_SHARED_KEY")
        .unwrap_or_else(|_| "development-installation-control-only".into())
}

fn claim_kind_text(kind: AdministratorEnrollmentClaimKindV1) -> &'static str {
    match kind {
        AdministratorEnrollmentClaimKindV1::Initial => "initial",
        AdministratorEnrollmentClaimKindV1::Recovery => "recovery",
    }
}

fn enrollment_unavailable() -> ApiError {
    ApiError::NotFound("administrator enrollment is unavailable".into())
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::routing::post;

    #[test]
    fn floor_is_versioned_and_deliberately_small() {
        assert_eq!(FLOOR_VERSION, "core-administration-v1");
        assert_eq!(FLOOR_CAPABILITIES, &["core:admin"]);
    }

    #[test]
    fn operation_names_are_stable() {
        assert_eq!(operation_text(AuthorizationGrantOperationV1::Read), "read");
        assert_eq!(
            operation_text(AuthorizationGrantOperationV1::Mutation),
            "mutation"
        );
    }

    #[test]
    fn module_enablement_maps_to_explicit_document_states() {
        assert_eq!(module_document_state(true), "enabled");
        assert_eq!(module_document_state(false), "disabled");
    }

    #[test]
    fn module_control_registry_is_definition_driven() {
        let endpoints = parse_module_control_endpoints(
            r#"{
                "tessara.reference.scoped-records": "http://scoped-records:8090",
                "tessara.dashboards": "http://dashboards:8091",
                "example.third-module": "http://third-module:8092"
            }"#,
        )
        .expect("control endpoint registry is valid");
        assert_eq!(
            endpoints.get("example.third-module").map(String::as_str),
            Some("http://third-module:8092")
        );
    }

    #[test]
    fn browser_path_matching_is_generic_and_segment_bounded() {
        assert_eq!(
            match_browser_path(
                "/reference/module-sdk/scopes/{organization_id}",
                "/reference/module-sdk/scopes/00000000-0000-0000-0000-000000000007"
            )
            .and_then(|parameters| parameters.get("organization_id").cloned()),
            Some("00000000-0000-0000-0000-000000000007".into())
        );
        assert!(
            match_browser_path(
                "/reference/module-sdk/scopes/{organization_id}",
                "/reference/module-sdk/scopes/one/extra"
            )
            .is_none()
        );
        assert!(match_browser_path("/reference/module-sdk", "/reference/scoped-records").is_none());
    }

    #[test]
    fn configuration_forms_are_coerced_from_manifest_schema() {
        let dashboard = configuration_form_payload(
            &json!({
                "type": "object",
                "properties": {
                    "display_label": {"type": "string"},
                    "default_page_size": {"type": "integer"}
                },
                "required": ["display_label", "default_page_size"]
            }),
            BTreeMap::from([
                ("schema_version".into(), "1".into()),
                ("display_label".into(), "Dashboards".into()),
                ("default_page_size".into(), "25".into()),
            ]),
        )
        .expect("Dashboard schema is supported");
        assert_eq!(
            dashboard,
            json!({
                "schema_version": 1,
                "display_label": "Dashboards",
                "default_page_size": 25
            })
        );

        let scoped_records = configuration_form_payload(
            &json!({
                "type": "object",
                "properties": {
                    "display_label": {"type": "string"},
                    "retention_mode": {
                        "type": "string",
                        "enum": ["retain_on_undeploy"]
                    }
                }
            }),
            BTreeMap::from([
                ("schema_version".into(), "1".into()),
                ("display_label".into(), "Scoped Records".into()),
                ("retention_mode".into(), "retain_on_undeploy".into()),
            ]),
        )
        .expect("Scoped Records schema is supported");
        assert_eq!(
            scoped_records,
            json!({
                "schema_version": 1,
                "display_label": "Scoped Records",
                "retention_mode": "retain_on_undeploy"
            })
        );
    }

    #[test]
    fn configuration_forms_reject_fields_outside_the_manifest_schema() {
        let error = configuration_form_payload(
            &json!({
                "type": "object",
                "properties": {"display_label": {"type": "string"}}
            }),
            BTreeMap::from([
                ("display_label".into(), "Example".into()),
                ("core_only_override".into(), "not allowed".into()),
            ]),
        )
        .expect_err("Core-only fields must not bypass the module manifest");
        assert!(
            error
                .to_string()
                .contains("unknown module configuration fields")
        );
    }

    #[test]
    fn reservation_identity_is_stable_and_input_bound() {
        let first = deterministic_reservation_id(
            Uuid::from_u128(1),
            Uuid::from_u128(2),
            1,
            "same-redemption",
        );
        assert_eq!(
            first,
            deterministic_reservation_id(
                Uuid::from_u128(1),
                Uuid::from_u128(2),
                1,
                "same-redemption"
            )
        );
        assert_ne!(
            first,
            deterministic_reservation_id(
                Uuid::from_u128(1),
                Uuid::from_u128(2),
                1,
                "another-redemption"
            )
        );
    }

    #[test]
    fn enrollment_handoff_digest_is_stable_and_not_plaintext() {
        let digest = handoff_token_digest("one-time-browser-handoff");
        assert_eq!(
            digest,
            "sha256:0a35e58db5a95dbc80054052f8839c1d94ec796eb188cd13f0382e266f0ff5bb"
        );
        assert!(!digest.contains("one-time-browser-handoff"));
        assert_ne!(digest, handoff_token_digest("another-handoff"));
    }

    #[test]
    fn prepared_enrollment_form_prefills_only_non_secret_claim_context() {
        let installation_id = Uuid::from_u128(1);
        let claim_id = Uuid::from_u128(2);
        let html = enrollment_form_document(
            installation_id,
            &EnrollmentPrefill {
                claim_id: Some(claim_id),
                generation: Some(3),
                claim_kind: Some(AdministratorEnrollmentClaimKindV1::Recovery),
            },
        );
        assert!(html.contains(&installation_id.to_string()));
        assert!(html.contains(&format!("value=\"{claim_id}\" readonly")));
        assert!(html.contains("value=\"3\" readonly"));
        assert!(html.contains("value=\"recovery\""));
        assert!(html.contains("minlength=\"12\""));
        assert!(html.contains("Use at least 12 characters"));
        assert!(!html.contains("value=\"admin@tessara.local\""));
        assert!(!html.contains("value=\"Tessara Administrator\""));
        assert!(html.contains("event.preventDefault()"));
        assert!(html.contains("enrollment-error-panel"));
        assert!(html.contains("<h1 id=\"enrollment-title\">Recover administrator access</h1>"));
        assert!(!html.contains("class=\"kicker\""));
    }

    #[test]
    fn unavailable_enrollment_is_a_designed_page_with_reissue_guidance() {
        let html = enrollment_unavailable_document("Enrollment is unavailable.");
        assert!(html.contains("<h1 id=\"enrollment-title\">Enrollment unavailable</h1>"));
        assert!(html.contains("tessara.ps1 enrollment issue -Open"));
        assert!(!html.contains("\"code\":\"not_found\""));
        assert!(!html.contains("class=\"kicker\""));
    }

    #[test]
    fn closed_enrollment_has_no_badge() {
        let html = enrollment_closed_document();
        assert!(html.contains("<h1 id=\"enrollment-title\">Administrator enrollment closed</h1>"));
        assert!(!html.contains("class=\"kicker\""));
    }

    #[test]
    fn successful_enrollment_is_unambiguous_and_redirects_to_sign_in() {
        let html = enrollment_success_document();
        assert!(html.contains("<h1 id=\"enrollment-title\">Enrollment successful</h1>"));
        assert!(html.contains("The Core Administrator account is ready."));
        assert!(html.contains("location.replace('/login')"));
        assert!(html.contains("Continue to sign in"));
        assert!(!html.contains("class=\"kicker\""));
        assert!(!html.contains("Core administration remains protected"));
        assert!(!html.contains("Administrator enrollment closed"));
    }

    #[tokio::test]
    async fn legacy_local_enrollment_get_redirects_to_guided_page() {
        let response = legacy_enrollment_redirect().await.into_response();
        assert_eq!(response.status(), StatusCode::SEE_OTHER);
        assert_eq!(
            response.headers().get(header::LOCATION).unwrap(),
            "/enrollment"
        );
    }

    #[tokio::test]
    async fn local_enrollment_is_atomic_global_and_idempotent() {
        let database_url = std::env::var("TEST_ENROLLMENT_DATABASE_URL")
            .expect("TEST_ENROLLMENT_DATABASE_URL is required for enrollment integration tests");
        let config = crate::config::Config {
            database_url,
            bind_addr: "127.0.0.1:0".into(),
            dev_admin_email: "existing-breakglass@tessara.local".into(),
            dev_admin_password: "existing-breakglass-password".into(),
            auth_cookie_name: "enrollment_test".into(),
            auth_cookie_secure: false,
            auth_session_ttl_hours: 1,
        };
        let pool = crate::db::connect_and_prepare(&config)
            .await
            .expect("enrollment database prepares");
        let installation_id: Uuid =
            sqlx::query_scalar("SELECT id FROM application_installations WHERE singleton=true")
                .fetch_one(&pool)
                .await
                .unwrap();
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        let mock = Router::new()
            .route(
                "/v1/reservations",
                post(|Json(input): Json<serde_json::Value>| async move {
                    Json(serde_json::json!({
                        "schema_version": 1,
                        "installation_id": input["installation_id"],
                        "claim_id": input["claim_id"],
                        "generation": input["generation"],
                        "reservation_id": input["reservation_id"],
                        "reserved_at": Utc::now(),
                        "expires_at": Utc::now() + Duration::seconds(120)
                    }))
                }),
            )
            .route(
                "/v1/redemptions",
                post(|| async { Json(serde_json::json!({"state": "consumed"})) }),
            );
        tokio::spawn(async move {
            axum::serve(listener, mock).await.unwrap();
        });
        unsafe {
            std::env::set_var(
                "TESSARA_INSTALLATION_CONTROL_URL",
                format!("http://{address}"),
            );
        }
        let claim_id = Uuid::new_v4();
        let request = AdministratorRedemptionRequest {
            schema_version: 1,
            installation_id,
            claim_id,
            generation: 1,
            claim_kind: AdministratorEnrollmentClaimKindV1::Initial,
            claim_secret: "write-only-claim-secret".into(),
            idempotency_key: "local-enrollment-one".into(),
            identity_path: "local".into(),
            email: Some("first-admin@tessara.local".into()),
            display_name: Some("First Administrator".into()),
            password: Some("long-enough-local-password".into()),
            external_assertion: None,
        };
        let state = AppState {
            pool: pool.clone(),
            config,
        };
        let first = redeem(&state, request.clone())
            .await
            .expect("first redemption succeeds");
        let repeated = redeem(&state, request)
            .await
            .expect("same redemption resumes after viability closes");
        assert_eq!(first.payload, repeated.payload);
        let row = sqlx::query(
            "SELECT a.id,ac.password_hash,ra.node_id,r.name
             FROM accounts a
             JOIN account_credentials ac ON ac.account_id=a.id
             JOIN role_assignments ra ON ra.account_id=a.id
             JOIN roles r ON r.id=ra.role_id
             WHERE a.email='first-admin@tessara.local'",
        )
        .fetch_one(&pool)
        .await
        .unwrap();
        assert!(
            row.try_get::<String, _>("password_hash")
                .unwrap()
                .starts_with("$argon2")
        );
        assert_eq!(row.try_get::<Option<Uuid>, _>("node_id").unwrap(), None);
        assert_eq!(
            row.try_get::<String, _>("name").unwrap(),
            "Core Administrator"
        );
        assert!(viable_administrator_exists(&pool).await.unwrap());

        sqlx::query("UPDATE accounts SET is_active=false WHERE id=$1")
            .bind(first.payload.account_id)
            .execute(&pool)
            .await
            .unwrap();
        assert!(!viable_administrator_exists(&pool).await.unwrap());
        let fixture_signer = PurposeBoundSigningKeyV1::from_secret_bytes(
            "tessara.fixture-identity",
            "fixture-external-v1",
            ProtocolSignaturePurposeV1::FixtureExternalIdentity,
            [13; 32],
        )
        .unwrap();
        unsafe {
            std::env::set_var(
                "TESSARA_FIXTURE_EXTERNAL_PUBLIC_KEY",
                URL_SAFE_NO_PAD.encode(fixture_signer.verifier().public_key_bytes()),
            );
        }
        let assertion_now = Utc::now();
        let assertion = fixture_signer
            .sign(ExternalIdentityAssertionV1 {
                schema_version: 1,
                installation_id,
                audience: "tessara.core.administrator-enrollment".into(),
                external_subject: "fixture-subject-001".into(),
                email: "external-admin@tessara.local".into(),
                display_name: "External Administrator".into(),
                nonce: Uuid::new_v4(),
                issued_at: assertion_now,
                expires_at: assertion_now + Duration::seconds(60),
            })
            .unwrap();
        let external = redeem(
            &state,
            AdministratorRedemptionRequest {
                schema_version: 1,
                installation_id,
                claim_id: Uuid::new_v4(),
                generation: 2,
                claim_kind: AdministratorEnrollmentClaimKindV1::Recovery,
                claim_secret: "another-write-only-claim-secret".into(),
                idempotency_key: "fixture-external-enrollment-one".into(),
                identity_path: "fixture_external".into(),
                email: None,
                display_name: None,
                password: None,
                external_assertion: Some(assertion),
            },
        )
        .await
        .expect("signed fixture external enrollment succeeds");
        let binding: (String, String, bool) = sqlx::query_as(
            "SELECT issuer,external_subject,is_usable
             FROM external_identity_bindings WHERE account_id=$1",
        )
        .bind(external.payload.account_id)
        .fetch_one(&pool)
        .await
        .unwrap();
        assert_eq!(
            binding,
            (
                "tessara.fixture-identity".into(),
                "fixture-subject-001".into(),
                true
            )
        );
    }
}
