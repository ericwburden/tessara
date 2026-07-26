use crate::{
    ProtocolEnvelopeError, PurposeBoundVerifyingKeyV1, ShellContextV1,
    ShellContextValidationContextV1, ShellContextValidationError, SignedEnvelopeV1,
};
use serde::{Deserialize, Serialize};

pub const SHELL_CONTENT_MEDIA_TYPE: &str = "application/vnd.tessara.shell-content+json";

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ShellContentV1 {
    pub schema_version: u16,
    pub title: String,
    pub body_html: String,
}

pub fn verify_shell_context(
    envelope: &SignedEnvelopeV1<ShellContextV1>,
    verifier: &PurposeBoundVerifyingKeyV1,
    expected: &ShellContextValidationContextV1,
) -> Result<(), ModuleShellError> {
    verifier.verify(envelope)?;
    envelope.payload.validate_for(expected)?;
    Ok(())
}

/// Renders a complete, no-JavaScript-useful native module document from a
/// previously verified shell context. Product authorization is intentionally
/// absent: callers must evaluate it separately before supplying product body.
pub fn render_native_module_document(
    context: &ShellContextV1,
    product_title: &str,
    body_html: &str,
) -> String {
    let navigation = context
        .navigation
        .iter()
        .map(|item| {
            format!(
                r#"<li><a href="{}">{}</a></li>"#,
                escape_attribute(&item.href),
                escape_text(&item.label)
            )
        })
        .collect::<String>();
    format!(
        r#"<!doctype html><html lang="{}" data-theme="{}"><head><meta charset="utf-8"><meta name="viewport" content="width=device-width,initial-scale=1"><title>{} · Tessara</title><style>:root{{color-scheme:dark}}*{{box-sizing:border-box}}body{{min-height:100vh;margin:0;display:grid;grid-template-columns:15rem 1fr;grid-template-rows:4rem 1fr;background:#0c1528;color:#f3f7ff;font-family:Inter,system-ui}}aside{{grid-row:1/3;padding:1.25rem;background:#2b3c55;border-right:1px solid #4a5c74}}aside>a{{display:block;margin-bottom:2rem;color:#fff;font-size:1.35rem;font-weight:800;text-decoration:none}}nav ul{{display:grid;gap:.4rem;margin:0;padding:0;list-style:none}}nav a{{display:block;padding:.65rem .75rem;border-radius:.35rem;color:#dfe8f5;text-decoration:none}}nav a:hover{{background:#245365;color:#22d3c5}}header{{display:flex;align-items:center;justify-content:space-between;padding:0 1.5rem;border-bottom:1px solid #263851}}header span{{color:#b8c5d8}}main{{align-self:start;width:min(74rem,calc(100% - 2rem));margin:2rem auto;padding:1.5rem;background:#2b3c55;border:1px solid #4a5c74;border-radius:.5rem}}h1{{font-size:1.45rem;border-bottom:2px solid #18b8ac;padding-bottom:.5rem}}h2{{font-size:1.15rem}}.metric,.diagnostic{{border:1px solid #4a5c74;border-radius:.4rem;padding:1.25rem;margin-top:1rem}}code,pre{{overflow-wrap:anywhere}}a,button{{color:#22d3c5}}@media(max-width:48rem){{body{{display:block}}aside{{padding:1rem;border:0;border-bottom:1px solid #4a5c74}}aside>a{{margin-bottom:.75rem}}nav ul{{display:flex;flex-wrap:wrap}}header{{padding:1rem}}main{{width:100%;margin:0;padding:1rem;border:0;border-radius:0}}}}</style></head><body data-shell-state="{}" data-correlation-id="{}"><aside><a href="{}">Tessara</a><nav aria-label="Main navigation"><ul>{}</ul></nav></aside><header><strong>{}</strong><span>{}</span></header><main>{}</main></body></html>"#,
        escape_attribute(&context.locale),
        theme_name(context),
        escape_text(product_title),
        document_state_name(context),
        context.correlation_id,
        escape_attribute(&context.return_destination),
        navigation,
        escape_text(product_title),
        escape_text(&context.original_actor.display_name),
        body_html,
    )
}

fn theme_name(context: &ShellContextV1) -> &'static str {
    match context.theme {
        crate::ShellThemeV1::System => "system",
        crate::ShellThemeV1::Light => "light",
        crate::ShellThemeV1::Dark => "dark",
    }
}

fn document_state_name(context: &ShellContextV1) -> &'static str {
    match context.document_state {
        crate::ShellDocumentStateV1::Active => "active",
        crate::ShellDocumentStateV1::Disabled => "disabled",
        crate::ShellDocumentStateV1::Degraded => "degraded",
        crate::ShellDocumentStateV1::StaleContext => "stale_context",
        crate::ShellDocumentStateV1::Recovery => "recovery",
    }
}

fn escape_text(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

fn escape_attribute(value: &str) -> String {
    escape_text(value)
}

#[derive(Debug, thiserror::Error)]
pub enum ModuleShellError {
    #[error("shell signature validation failed: {0}")]
    Signature(#[from] ProtocolEnvelopeError),
    #[error("shell context validation failed: {0}")]
    Context(#[from] ShellContextValidationError),
}

#[cfg(test)]
mod tests {
    use chrono::{Duration, Utc};
    use uuid::Uuid;

    use super::*;
    use crate::{
        ModuleDefinitionId, OriginalActorProjectionV1, ProtocolSignaturePurposeV1,
        PurposeBoundSigningKeyV1, ShellDocumentStateV1, ShellThemeV1,
    };

    #[test]
    fn renderer_is_complete_escaped_and_preserves_recovery_state() {
        let now = Utc::now();
        let context = ShellContextV1 {
            schema_version: 1,
            installation_id: Uuid::from_u128(1),
            module_definition_id: ModuleDefinitionId::new("tessara.reference.scoped-records")
                .unwrap(),
            module_instance_id: Uuid::from_u128(2),
            original_actor: OriginalActorProjectionV1 {
                actor_id: Uuid::from_u128(3),
                display_name: "<Operator>".into(),
                email: None,
            },
            theme: ShellThemeV1::Dark,
            navigation: vec![],
            return_destination: "/administration/modules".into(),
            locale: "en-US".into(),
            time_zone: "America/New_York".into(),
            correlation_id: Uuid::from_u128(4),
            document_state: ShellDocumentStateV1::Recovery,
            issued_at: now,
            expires_at: now + Duration::seconds(60),
        };
        let html = render_native_module_document(&context, "Scoped Records", "<p>Recovery</p>");
        assert!(html.starts_with("<!doctype html>"));
        assert!(html.contains("data-shell-state=\"recovery\""));
        assert!(html.contains("&lt;Operator&gt;"));
        assert!(html.contains("<p>Recovery</p>"));
    }

    #[test]
    fn verifier_checks_signature_before_context() {
        let signer = PurposeBoundSigningKeyV1::from_secret_bytes(
            "tessara.core",
            "shell-v1",
            ProtocolSignaturePurposeV1::ShellContext,
            [44; 32],
        )
        .unwrap();
        let now = Utc::now();
        let context = ShellContextV1 {
            schema_version: 1,
            installation_id: Uuid::from_u128(1),
            module_definition_id: ModuleDefinitionId::new("tessara.reference.records").unwrap(),
            module_instance_id: Uuid::from_u128(2),
            original_actor: OriginalActorProjectionV1 {
                actor_id: Uuid::from_u128(3),
                display_name: "Operator".into(),
                email: None,
            },
            theme: ShellThemeV1::System,
            navigation: vec![],
            return_destination: "/".into(),
            locale: "en-US".into(),
            time_zone: "UTC".into(),
            correlation_id: Uuid::from_u128(4),
            document_state: ShellDocumentStateV1::Active,
            issued_at: now,
            expires_at: now + Duration::seconds(60),
        };
        let envelope = signer.sign(context.clone()).unwrap();
        verify_shell_context(
            &envelope,
            &signer.verifier(),
            &ShellContextValidationContextV1 {
                installation_id: context.installation_id,
                module_definition_id: context.module_definition_id,
                module_instance_id: context.module_instance_id,
                correlation_id: context.correlation_id,
                now,
            },
        )
        .unwrap();
    }
}
