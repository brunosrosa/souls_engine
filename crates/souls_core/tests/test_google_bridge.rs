use souls_core::google_bridge::{
    GoogleSessionCredentials, GoogleSessionVault, JulesMigrationRouter, LegacyJulesCommand,
    WizRpcId, WizRpcPayload, WizRpcResponse,
};
use souls_core::harness::AgenticCliKind;
use std::path::PathBuf;
use tempfile::tempdir;

#[test]
fn test_google_session_vault_encryption_roundtrip() {
    let tmp = tempdir().unwrap();
    let vault_path = tmp.path().join("test_google.vault");

    let creds = GoogleSessionCredentials::new()
        .with_psid("g.a000test_psid_secure_token_12345")
        .with_psidts("sidts-001-rotational-key-98765")
        .with_sapisid("sapisid_hash_val_abcde")
        .with_csrf_token("SNlM0e")
        .with_oauth_token("ya29.a0ARrdaM_test_bearer_token")
        .with_email("architect@soulsengine.ai");

    assert!(creds.is_configured());
    assert!(!creds.is_expired(1000));

    let cookie_header = creds.cookie_header();
    assert!(cookie_header.contains("__Secure-1PSID=g.a000test_psid_secure_token_12345"));
    assert!(cookie_header.contains("__Secure-1PSIDTS=sidts-001-rotational-key-98765"));
    assert!(cookie_header.contains("SAPISID=sapisid_hash_val_abcde"));

    let masked = creds.masked();
    assert!(masked.psid.unwrap().contains("..."));
    assert_eq!(masked.account_email.unwrap(), "architect@soulsengine.ai");

    // Save encrypted to disk
    GoogleSessionVault::save(&vault_path, &creds).unwrap();
    assert!(vault_path.exists());

    // Verify raw bytes are encrypted (not plain JSON)
    let raw_bytes = std::fs::read(&vault_path).unwrap();
    assert!(raw_bytes.starts_with(b"GV1:"));
    assert!(!String::from_utf8_lossy(&raw_bytes).contains("architect@soulsengine.ai"));

    // Load and decrypt
    let loaded = GoogleSessionVault::load(&vault_path).unwrap().expect("Loaded credentials");
    assert_eq!(loaded, creds);
}

#[test]
fn test_wiz_rpc_payload_f_req_encoding() {
    let args = serde_json::json!({
        "notebook_id": "nb_uuid_999",
        "title": "Nota de Arquitetura",
        "content": "Conclusão da auditoria bare-metal..."
    });

    let payload = WizRpcPayload::new(WizRpcId::CreateNote, args);
    assert_eq!(payload.rpc_id.as_str(), "wXbhsf");

    let f_req = payload.to_f_req_json().unwrap();
    assert!(f_req.contains("wXbhsf"));
    assert!(f_req.contains("nb_uuid_999"));

    let form_body = payload.to_form_body("SNlM0e").unwrap();
    assert!(form_body.starts_with("f.req="));
    assert!(form_body.contains("&at=SNlM0e"));
}

#[test]
fn test_wiz_rpc_batchexecute_anti_xss_response_parser() {
    // Realistic Google batchexecute response with anti-XSS header and chunk envelopes
    let raw_google_response = r#")]}'
128
[[["wrb.fr","wXbhsf","{\"note_id\":\"note_777_created\",\"status\":\"SUCCESS\"}",null,null,null,"generic"]]]
"#;

    let response = WizRpcResponse::parse(raw_google_response, "wXbhsf").unwrap();
    assert!(response.success);
    assert_eq!(response.rpc_id, "wXbhsf");

    let payload = response.payload.expect("Payload extracted");
    assert_eq!(payload["note_id"], "note_777_created");
    assert_eq!(payload["status"], "SUCCESS");
}

#[test]
fn test_jules_migration_router_to_antigravity() {
    assert!(JulesMigrationRouter::is_legacy_jules_command("jules run refactor"));
    assert!(JulesMigrationRouter::is_legacy_jules_command("jules.exe --prompt='test'"));
    assert!(!JulesMigrationRouter::is_legacy_jules_command("agy -p 'test'"));

    let jules_cmd = LegacyJulesCommand {
        prompt: "refactor memory indices".to_string(),
        working_dir: Some(PathBuf::from("Z:\\souls_engine")),
        auto_approve: true,
        model: Some("gemini-2.5-pro".to_string()),
    };

    let agy_cmd = JulesMigrationRouter::migrate_to_antigravity(
        jules_cmd,
        &PathBuf::from("Z:\\souls_engine"),
    );

    assert_eq!(agy_cmd.cli, AgenticCliKind::Antigravity);
    assert_eq!(agy_cmd.prompt, "refactor memory indices");
    assert_eq!(agy_cmd.model.as_deref(), Some("gemini-2.5-pro"));
    assert!(agy_cmd.extra_args.contains(&"--dangerously-skip-permissions".to_string()));
}

