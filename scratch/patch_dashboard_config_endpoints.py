# -*- coding: utf-8 -*-
import sys

def main():
    dashboard_rs = "crates/souls_server/src/routes/dashboard.rs"
    with open(dashboard_rs, "r", encoding="utf-8") as f:
        dash = f.read()

    # 1. Enrich get_config_handler and add validate_config_toml_handler & restore_config_backup_handler
    target = """    Json(json!({
        "status": "success",
        "path": p.to_string_lossy(),
        "raw_toml": content,
        "parsed": parsed
    }))
}"""

    replacement = """    let backup_path = p.with_extension("toml.bak");
    let file_size_bytes = fs::metadata(&p).map(|m| m.len()).unwrap_or(content.len() as u64);
    let line_count = content.lines().count();
    let sections_count = content.lines().filter(|l| l.trim().starts_with('[') && l.trim().ends_with(']')).count();

    Json(json!({
        "status": "success",
        "path": p.to_string_lossy(),
        "backup_path": backup_path.to_string_lossy(),
        "backup_exists": backup_path.exists(),
        "file_size_bytes": file_size_bytes,
        "line_count": line_count,
        "sections_count": sections_count,
        "raw_toml": content,
        "parsed": parsed
    }))
}

/// Validates raw TOML syntax and returns section names and line metrics.
pub async fn validate_config_toml_handler(
    Json(payload): Json<UpdateConfigRequest>,
) -> Json<Value> {
    let text = payload.raw_toml.unwrap_or_default();
    let mut errors = Vec::new();
    let mut sections = Vec::new();
    let mut line_no = 0;

    for line in text.lines() {
        line_no += 1;
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }

        if trimmed.starts_with('[') {
            if !trimmed.ends_with(']') {
                errors.push(format!("Linha {}: Seção TOML com colchete não fechado: '{}'", line_no, trimmed));
            } else {
                let sec_name = trimmed[1..trimmed.len() - 1].trim();
                if sec_name.is_empty() {
                    errors.push(format!("Linha {}: Nome de seção TOML vazio '[]'", line_no));
                } else {
                    sections.push(sec_name.to_string());
                }
            }
            continue;
        }

        if !trimmed.contains('=') {
            errors.push(format!("Linha {}: Linha de configuração sem operador de atribuição '=': '{}'", line_no, trimmed));
        } else {
            let (k, v) = trimmed.split_once('=').unwrap();
            if k.trim().is_empty() {
                errors.push(format!("Linha {}: Chave de configuração vazia antes do '='", line_no));
            }
            let v_trim = v.trim();
            if (v_trim.starts_with('"') && !v_trim.ends_with('"')) || (v_trim.starts_with('\'') && !v_trim.ends_with('\'')) {
                errors.push(format!("Linha {}: String TOML com aspas não terminadas: {}", line_no, v_trim));
            }
        }
    }

    if errors.is_empty() {
        Json(json!({
            "status": "success",
            "valid": true,
            "message": "✓ Sintaxe TOML v1.0 válida e estrita",
            "section_count": sections.len(),
            "sections": sections,
            "line_count": text.lines().count(),
            "byte_count": text.len()
        }))
    } else {
        Json(json!({
            "status": "error",
            "valid": false,
            "error": errors.join(" | "),
            "errors": errors,
            "section_count": sections.len(),
            "sections": sections,
            "line_count": text.lines().count(),
            "byte_count": text.len()
        }))
    }
}

/// Restores souls_config.toml from the safety backup (.toml.bak).
pub async fn restore_config_backup_handler() -> Json<Value> {
    let p = get_config_file_path();
    let backup_path = p.with_extension("toml.bak");

    if !backup_path.exists() {
        return Json(json!({
            "status": "error",
            "message": "Nenhum arquivo de backup (.toml.bak) encontrado no diretório de configuração."
        }));
    }

    match fs::copy(&backup_path, &p) {
        Ok(_) => {
            let content = fs::read_to_string(&p).unwrap_or_default();
            let parsed = parse_toml_to_json(&content);
            info!("Restored souls_config.toml from backup: {:?}", backup_path);
            Json(json!({
                "status": "success",
                "message": "Backup souls_config.toml.bak restaurado com sucesso!",
                "path": p.to_string_lossy(),
                "backup_path": backup_path.to_string_lossy(),
                "raw_toml": content,
                "parsed": parsed
            }))
        }
        Err(e) => {
            Json(json!({
                "status": "error",
                "message": format!("Falha ao restaurar backup: {}", e)
            }))
        }
    }
}"""

    assert target in dash, "get_config_handler target not found in dashboard.rs"
    dash = dash.replace(target, replacement, 1)

    dash = dash.replace("\r\n", "\n")
    with open(dashboard_rs, "w", encoding="utf-8") as f:
        f.write(dash)
    print("Updated dashboard.rs!")

    # 2. Register routes in routes/mod.rs
    mod_rs = "crates/souls_server/src/routes/mod.rs"
    with open(mod_rs, "r", encoding="utf-8") as f:
        mod_content = f.read()

    old_route = '.route("/api/v1/settings/config", get(dashboard::get_config_handler).post(dashboard::post_config_handler))'
    new_route = '''.route("/api/v1/settings/config", get(dashboard::get_config_handler).post(dashboard::post_config_handler))
        .route("/api/v1/settings/config/validate", post(dashboard::validate_config_toml_handler))
        .route("/api/v1/settings/config/restore", post(dashboard::restore_config_backup_handler))'''

    assert old_route in mod_content, "old_route not found in mod.rs"
    mod_content = mod_content.replace(old_route, new_route, 1)

    mod_content = mod_content.replace("\r\n", "\n")
    with open(mod_rs, "w", encoding="utf-8") as f:
        f.write(mod_content)
    print("Updated routes/mod.rs!")

if __name__ == "__main__":
    main()

