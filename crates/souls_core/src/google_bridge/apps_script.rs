//! Google Apps Script 24/7 API client, `clasp` CLI bridge, curated automation templates,
//! and static security + human-readable permissions analyzer.

use super::oauth::{find_binary_on_path, percent_encode};
use super::workspace::parse_json_response;
use crate::error::CoreError;
use serde_json::{json, Value};
use souls_protocol::google::{
    AppsScriptDeploymentDto, AppsScriptFileDto, AppsScriptProcessDto, AppsScriptProjectDto,
    AppsScriptRiskWarningDto, AppsScriptStaticScanDto, AppsScriptTemplateDto,
};
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::time::Duration;
use tokio::process::Command;

/// REST client for the Google Apps Script API v1 (`script.googleapis.com/v1`).
#[derive(Debug, Clone)]
pub struct GoogleAppsScriptClient {
    http: reqwest::Client,
    base_url: String,
}

impl Default for GoogleAppsScriptClient {
    fn default() -> Self {
        Self::new()
    }
}

impl GoogleAppsScriptClient {
    /// Creates an Apps Script client configured for production endpoints.
    pub fn new() -> Self {
        Self::with_base_url("https://script.googleapis.com/v1")
    }

    /// Creates an Apps Script client with a custom base URL (for offline `TcpListener` mocks).
    pub fn with_base_url(base_url: impl Into<String>) -> Self {
        let http = reqwest::Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .unwrap_or_default();
        Self {
            http,
            base_url: base_url.into().trim_end_matches('/').to_string(),
        }
    }

    /// Creates a new Apps Script project (`POST /projects`).
    pub async fn create_project(
        &self,
        token: &str,
        title: &str,
        parent_id: Option<&str>,
    ) -> Result<AppsScriptProjectDto, CoreError> {
        let url = format!("{}/projects", self.base_url);
        let mut body = json!({ "title": title });
        if let Some(pid) = parent_id.filter(|s| !s.trim().is_empty()) {
            body["parentId"] = json!(pid.trim());
        }

        let val = self.post_json(&url, token, &body).await?;
        Ok(parse_project_dto(&val))
    }

    /// Retrieves metadata for an Apps Script project (`GET /projects/{scriptId}`).
    pub async fn get_project(
        &self,
        token: &str,
        script_id: &str,
    ) -> Result<AppsScriptProjectDto, CoreError> {
        let url = format!("{}/projects/{}", self.base_url, percent_encode(script_id));
        let val = self.get_json(&url, token).await?;
        Ok(parse_project_dto(&val))
    }

    /// Reads the source files of an Apps Script project (`GET /projects/{scriptId}/content`).
    pub async fn get_content(
        &self,
        token: &str,
        script_id: &str,
    ) -> Result<Vec<AppsScriptFileDto>, CoreError> {
        let url = format!(
            "{}/projects/{}/content",
            self.base_url,
            percent_encode(script_id)
        );
        let val = self.get_json(&url, token).await?;
        Ok(parse_files_array(&val))
    }

    /// Replaces the source files of an Apps Script project (`PUT /projects/{scriptId}/content`).
    pub async fn update_content(
        &self,
        token: &str,
        script_id: &str,
        files: &[AppsScriptFileDto],
    ) -> Result<Vec<AppsScriptFileDto>, CoreError> {
        let url = format!(
            "{}/projects/{}/content",
            self.base_url,
            percent_encode(script_id)
        );
        let payload_files: Vec<Value> = files
            .iter()
            .map(|f| {
                json!({
                    "name": f.name,
                    "type": f.file_type,
                    "source": f.source,
                })
            })
            .collect();
        let val = self
            .put_json(&url, token, &json!({ "files": payload_files }))
            .await?;
        let parsed = parse_files_array(&val);
        if parsed.is_empty() {
            Ok(files.to_vec())
        } else {
            Ok(parsed)
        }
    }

    /// Publishes an immutable project version (`POST /projects/{scriptId}/versions`).
    pub async fn create_version(
        &self,
        token: &str,
        script_id: &str,
        description: Option<&str>,
    ) -> Result<i64, CoreError> {
        let url = format!(
            "{}/projects/{}/versions",
            self.base_url,
            percent_encode(script_id)
        );
        let mut body = json!({});
        if let Some(desc) = description.filter(|s| !s.trim().is_empty()) {
            body["description"] = json!(desc.trim());
        }
        let val = self.post_json(&url, token, &body).await?;
        val.get("versionNumber")
            .and_then(|v| v.as_i64().or_else(|| v.as_str()?.parse::<i64>().ok()))
            .ok_or_else(|| {
                CoreError::SubprocessError("Apps Script create_version missing versionNumber".into())
            })
    }

    /// Creates a deployment for an Apps Script project (`POST /projects/{scriptId}/deployments`).
    pub async fn create_deployment(
        &self,
        token: &str,
        script_id: &str,
        version_number: i64,
        description: Option<&str>,
    ) -> Result<AppsScriptDeploymentDto, CoreError> {
        let url = format!(
            "{}/projects/{}/deployments",
            self.base_url,
            percent_encode(script_id)
        );
        let mut body = json!({
            "versionNumber": version_number,
            "manifestFileName": "appsscript"
        });
        if let Some(desc) = description.filter(|s| !s.trim().is_empty()) {
            body["description"] = json!(desc.trim());
        }
        let val = self.post_json(&url, token, &body).await?;
        Ok(parse_deployment_dto(&val))
    }

    /// Lists deployments of an Apps Script project (`GET /projects/{scriptId}/deployments`).
    pub async fn list_deployments(
        &self,
        token: &str,
        script_id: &str,
    ) -> Result<Vec<AppsScriptDeploymentDto>, CoreError> {
        let url = format!(
            "{}/projects/{}/deployments",
            self.base_url,
            percent_encode(script_id)
        );
        let val = self.get_json(&url, token).await?;
        let arr = val
            .get("deployments")
            .and_then(|v| v.as_array())
            .cloned()
            .unwrap_or_default();
        Ok(arr.iter().map(parse_deployment_dto).collect())
    }

    /// Lists recent Apps Script execution processes (`GET /processes`).
    pub async fn list_processes(
        &self,
        token: &str,
        script_id: Option<&str>,
    ) -> Result<Vec<AppsScriptProcessDto>, CoreError> {
        let url = match script_id.filter(|s| !s.trim().is_empty()) {
            Some(sid) => format!(
                "{}/processes?userProcessFilter.scriptId={}",
                self.base_url,
                percent_encode(sid.trim())
            ),
            None => format!("{}/processes", self.base_url),
        };
        let val = self.get_json(&url, token).await?;
        let arr = val
            .get("processes")
            .and_then(|v| v.as_array())
            .cloned()
            .unwrap_or_default();
        Ok(arr.iter().map(parse_process_dto).collect())
    }

    /// Executes an Apps Script function via the Execution API (`POST /scripts/{scriptId}:run`).
    pub async fn run_function(
        &self,
        token: &str,
        script_id: &str,
        function_name: &str,
        parameters: Option<Vec<Value>>,
        dev_mode: bool,
    ) -> Result<Value, CoreError> {
        let url = format!(
            "{}/scripts/{}:run",
            self.base_url,
            percent_encode(script_id)
        );
        let mut body = json!({
            "function": function_name,
            "devMode": dev_mode,
        });
        if let Some(params) = parameters {
            body["parameters"] = Value::Array(params);
        }

        let val = self.post_json(&url, token, &body).await?;
        if let Some(err_obj) = val.get("error") {
            return Err(CoreError::SubprocessError(format!(
                "Apps Script execution error in '{}': {}",
                function_name, err_obj
            )));
        }
        Ok(val.get("response").cloned().unwrap_or(val))
    }

    async fn get_json(&self, url: &str, token: &str) -> Result<Value, CoreError> {
        let resp = self
            .http
            .get(url)
            .bearer_auth(token)
            .send()
            .await
            .map_err(|e| CoreError::SubprocessError(format!("Apps Script GET '{url}': {e}")))?;
        parse_json_response(resp, url).await
    }

    async fn post_json(&self, url: &str, token: &str, body: &Value) -> Result<Value, CoreError> {
        let resp = self
            .http
            .post(url)
            .bearer_auth(token)
            .json(body)
            .send()
            .await
            .map_err(|e| CoreError::SubprocessError(format!("Apps Script POST '{url}': {e}")))?;
        parse_json_response(resp, url).await
    }

    async fn put_json(&self, url: &str, token: &str, body: &Value) -> Result<Value, CoreError> {
        let resp = self
            .http
            .put(url)
            .bearer_auth(token)
            .json(body)
            .send()
            .await
            .map_err(|e| CoreError::SubprocessError(format!("Apps Script PUT '{url}': {e}")))?;
        parse_json_response(resp, url).await
    }
}

/// Optional CLI helper wrapping `clasp` when installed on host.
#[derive(Debug, Clone)]
pub struct ClaspCliBridge {
    binary_path: Option<PathBuf>,
    timeout_ms: u64,
}

impl Default for ClaspCliBridge {
    fn default() -> Self {
        Self::new()
    }
}

impl ClaspCliBridge {
    /// Auto-discovers `clasp` on PATH and standard user npm/bin directories.
    pub fn new() -> Self {
        Self {
            binary_path: find_binary_on_path("clasp"),
            timeout_ms: 8_000,
        }
    }

    /// Creates a `ClaspCliBridge` targeting an explicit binary path.
    pub fn with_binary(path: PathBuf) -> Self {
        Self {
            binary_path: Some(path),
            timeout_ms: 8_000,
        }
    }

    /// Returns `true` if `clasp` was discovered on the host.
    pub fn is_installed(&self) -> bool {
        self.binary_path.as_ref().is_some_and(|p| p.is_file())
    }

    /// Queries `clasp --version` with timeout containment.
    pub async fn version(&self) -> Option<String> {
        let out = self.run_args(None, &["--version"]).await.ok()?;
        let line = out.lines().next()?.trim().to_string();
        if line.is_empty() {
            None
        } else {
            Some(line)
        }
    }

    /// Lists recent Stackdriver/Apps Script logs via `clasp logs`.
    pub async fn list_logs(
        &self,
        project_dir: Option<&Path>,
        limit: usize,
    ) -> Result<Vec<String>, CoreError> {
        let raw = self.run_args(project_dir, &["logs"]).await?;
        let lines: Vec<String> = raw
            .lines()
            .map(str::trim)
            .filter(|l| !l.is_empty())
            .take(limit.max(1))
            .map(ToString::to_string)
            .collect();
        Ok(lines)
    }

    async fn run_args(&self, working_dir: Option<&Path>, args: &[&str]) -> Result<String, CoreError> {
        let bin = self.binary_path.as_ref().ok_or_else(|| {
            CoreError::SubprocessError("clasp CLI is not installed on host".to_string())
        })?;

        let mut cmd = build_platform_command(bin, args);
        if let Some(dir) = working_dir {
            cmd.current_dir(dir);
        }
        cmd.stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .kill_on_drop(true);

        let child = cmd
            .spawn()
            .map_err(|e| CoreError::SubprocessError(format!("Failed to spawn clasp CLI: {e}")))?;

        let output = tokio::time::timeout(
            Duration::from_millis(self.timeout_ms),
            child.wait_with_output(),
        )
        .await
        .map_err(|_| CoreError::ExecutionTimeout(self.timeout_ms))?
        .map_err(|e| CoreError::SubprocessError(format!("clasp CLI I/O failure: {e}")))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
            return Err(CoreError::SubprocessError(format!(
                "clasp CLI failed (code {:?}): {stderr}",
                output.status.code()
            )));
        }

        Ok(String::from_utf8_lossy(&output.stdout).into_owned())
    }
}

#[cfg(windows)]
fn build_platform_command(bin: &Path, args: &[&str]) -> Command {
    let is_batch = bin
        .extension()
        .and_then(|e| e.to_str())
        .is_some_and(|e| e.eq_ignore_ascii_case("cmd") || e.eq_ignore_ascii_case("bat"));
    if is_batch {
        let mut cmd = Command::new("cmd");
        cmd.arg("/C").arg(bin);
        cmd.args(args);
        cmd
    } else {
        let mut cmd = Command::new(bin);
        cmd.args(args);
        cmd
    }
}

#[cfg(not(windows))]
fn build_platform_command(bin: &Path, args: &[&str]) -> Command {
    let mut cmd = Command::new(bin);
    cmd.args(args);
    cmd
}

/// Returns the 5 curated least-privilege 24/7 Google Apps Script automation templates.
pub fn builtin_apps_script_templates() -> Vec<AppsScriptTemplateDto> {
    vec![
        AppsScriptTemplateDto {
            id: "gmail_bills_organizer".to_string(),
            name: "Organizador Automático de Boletos e Faturas".to_string(),
            tagline: "Etiqueta boletos e cobranças automaticamente mesmo com o PC desligado".to_string(),
            description: "Varre mensagens recentes no Gmail em busca de boletos, faturas e notas fiscais, aplica o marcador 'Financeiro/Boletos' e registra o resumo nos logs."
                .to_string(),
            schedule_label: "A cada 1 hora".to_string(),
            required_scopes: vec![
                "https://www.googleapis.com/auth/gmail.modify".to_string(),
                "https://www.googleapis.com/auth/script.scriptapp".to_string(),
            ],
            human_permissions: vec![
                "Ler e organizar marcadores no seu Gmail".to_string(),
                "Executar automaticamente a cada 1 hora na sua conta Google".to_string(),
                "NÃO pode enviar e-mails para terceiros".to_string(),
                "NÃO envia dados para servidores externos".to_string(),
            ],
            files: vec![
                AppsScriptFileDto {
                    name: "Code".to_string(),
                    file_type: "SERVER_JS".to_string(),
                    source: r#"function runBillsOrganizer() {
  var labelName = "Financeiro/Boletos";
  var label = GmailApp.getUserLabelByName(labelName) || GmailApp.createLabel(labelName);
  var threads = GmailApp.search('newer_than:2d (boleto OR fatura OR invoice OR "nota fiscal") -label:"Financeiro/Boletos"', 0, 25);
  for (var i = 0; i < threads.length; i++) {
    threads[i].addLabel(label);
  }
  Logger.log("Organizados " + threads.length + " e-mails de boletos/faturas.");
}

function installHourlyTrigger() {
  var triggers = ScriptApp.getProjectTriggers();
  for (var i = 0; i < triggers.length; i++) {
    if (triggers[i].getHandlerFunction() === "runBillsOrganizer") {
      return;
    }
  }
  ScriptApp.newTrigger("runBillsOrganizer").timeBased().everyHours(1).create();
}
"#
                    .to_string(),
                },
                manifest_file(&[
                    "https://www.googleapis.com/auth/gmail.modify",
                    "https://www.googleapis.com/auth/script.scriptapp",
                ]),
            ],
        },
        AppsScriptTemplateDto {
            id: "drive_attachments_archiver".to_string(),
            name: "Arquivador de Anexos Importantes no Drive".to_string(),
            tagline: "Salva PDFs e XMLs de e-mails direto em uma pasta dedicada no Drive".to_string(),
            description: "Localiza e-mails com anexos PDF ou XML das últimas 24 horas, salva os arquivos na pasta 'Anexos Arquivados (Souls)' e aplica o marcador 'Arquivado-Drive'."
                .to_string(),
            schedule_label: "A cada 6 horas".to_string(),
            required_scopes: vec![
                "https://www.googleapis.com/auth/gmail.modify".to_string(),
                "https://www.googleapis.com/auth/drive.file".to_string(),
                "https://www.googleapis.com/auth/script.scriptapp".to_string(),
            ],
            human_permissions: vec![
                "Ler anexos e aplicar marcadores no Gmail".to_string(),
                "Criar e salvar arquivos apenas na pasta criada pela automação no Drive".to_string(),
                "NÃO pode enviar e-mails para terceiros".to_string(),
                "NÃO envia dados para servidores externos".to_string(),
            ],
            files: vec![
                AppsScriptFileDto {
                    name: "Code".to_string(),
                    file_type: "SERVER_JS".to_string(),
                    source: r#"function archiveImportantAttachments() {
  var folderName = "Anexos Arquivados (Souls)";
  var folders = DriveApp.getFoldersByName(folderName);
  var targetFolder = folders.hasNext() ? folders.next() : DriveApp.createFolder(folderName);
  var label = GmailApp.getUserLabelByName("Arquivado-Drive") || GmailApp.createLabel("Arquivado-Drive");
  var threads = GmailApp.search('has:attachment newer_than:1d -label:"Arquivado-Drive"', 0, 15);
  var saved = 0;

  for (var i = 0; i < threads.length; i++) {
    var messages = threads[i].getMessages();
    for (var j = 0; j < messages.length; j++) {
      var attachments = messages[j].getAttachments();
      for (var k = 0; k < attachments.length; k++) {
        var att = attachments[k];
        var ct = (att.getContentType() || "").toLowerCase();
        if (ct.indexOf("pdf") !== -1 || ct.indexOf("xml") !== -1) {
          targetFolder.createFile(att.copyBlob());
          saved++;
        }
      }
    }
    threads[i].addLabel(label);
  }
  Logger.log("Anexos salvos no Drive: " + saved);
}

function installSixHourTrigger() {
  ScriptApp.newTrigger("archiveImportantAttachments").timeBased().everyHours(6).create();
}
"#
                    .to_string(),
                },
                manifest_file(&[
                    "https://www.googleapis.com/auth/gmail.modify",
                    "https://www.googleapis.com/auth/drive.file",
                    "https://www.googleapis.com/auth/script.scriptapp",
                ]),
            ],
        },
        AppsScriptTemplateDto {
            id: "forms_instant_notifier".to_string(),
            name: "Alerta Instantâneo de Novo Formulário".to_string(),
            tagline: "Envia um alerta imediato para seu próprio e-mail ao receber resposta no Forms".to_string(),
            description: "Executa imediatamente a cada envio de formulário configurado e envia um resumo limpo exclusivamente para o dono da conta Google."
                .to_string(),
            schedule_label: "Por evento (onFormSubmit)".to_string(),
            required_scopes: vec![
                "https://www.googleapis.com/auth/forms.responses.readonly".to_string(),
                "https://www.googleapis.com/auth/script.send_mail".to_string(),
                "https://www.googleapis.com/auth/script.scriptapp".to_string(),
            ],
            human_permissions: vec![
                "Ler novas respostas recebidas no Google Forms".to_string(),
                "Enviar alerta por e-mail para sua própria conta Google".to_string(),
                "NÃO envia dados para servidores externos".to_string(),
                "NÃO apaga formulários ou respostas".to_string(),
            ],
            files: vec![
                AppsScriptFileDto {
                    name: "Code".to_string(),
                    file_type: "SERVER_JS".to_string(),
                    source: r#"function onFormSubmitNotify(e) {
  var ownerEmail = Session.getEffectiveUser().getEmail();
  if (!ownerEmail) {
    return;
  }
  var summary = "Nova resposta recebida no formulário.";
  if (e && e.response) {
    var items = e.response.getItemResponses();
    var lines = [];
    for (var i = 0; i < items.length; i++) {
      lines.push("- " + items[i].getItem().getTitle() + ": " + items[i].getResponse());
    }
    if (lines.length > 0) {
      summary = lines.join("\n");
    }
  }
  MailApp.sendEmail(ownerEmail, "[Souls Alert] Nova resposta no Google Forms", summary);
}
"#
                    .to_string(),
                },
                manifest_file(&[
                    "https://www.googleapis.com/auth/forms.responses.readonly",
                    "https://www.googleapis.com/auth/script.send_mail",
                    "https://www.googleapis.com/auth/script.scriptapp",
                ]),
            ],
        },
        AppsScriptTemplateDto {
            id: "gmail_promo_cleaner".to_string(),
            name: "Faxina Semanal de Promoções Expiradas".to_string(),
            tagline: "Limpa automaticamente promoções antigas com mais de 14 dias sem estrelas".to_string(),
            description: "Move para a lixeira do Gmail conversas antigas da aba Promoções (mais de 14 dias e não favoritadas), mantendo sua caixa leve."
                .to_string(),
            schedule_label: "Toda semana".to_string(),
            required_scopes: vec![
                "https://www.googleapis.com/auth/gmail.modify".to_string(),
                "https://www.googleapis.com/auth/script.scriptapp".to_string(),
            ],
            human_permissions: vec![
                "Ler e mover promoções antigas para a lixeira do Gmail".to_string(),
                "Executar semanalmente por agendamento automático".to_string(),
                "NÃO pode enviar e-mails para terceiros".to_string(),
                "NÃO envia dados para servidores externos".to_string(),
            ],
            files: vec![
                AppsScriptFileDto {
                    name: "Code".to_string(),
                    file_type: "SERVER_JS".to_string(),
                    source: r#"function cleanExpiredPromotions() {
  var threads = GmailApp.search("category:promotions older_than:14d -is:starred", 0, 50);
  if (threads.length > 0) {
    GmailApp.moveThreadsToTrash(threads);
  }
  Logger.log("Promoções expiradas enviadas à lixeira: " + threads.length);
}

function installWeeklyCleaner() {
  ScriptApp.newTrigger("cleanExpiredPromotions").timeBased().everyWeeks(1).create();
}
"#
                    .to_string(),
                },
                manifest_file(&[
                    "https://www.googleapis.com/auth/gmail.modify",
                    "https://www.googleapis.com/auth/script.scriptapp",
                ]),
            ],
        },
        AppsScriptTemplateDto {
            id: "morning_digest_cloud".to_string(),
            name: "Resumo Matinal na Nuvem via E-mail".to_string(),
            tagline: "Envia agenda do dia e contagem de não lidos todo dia às 07h".to_string(),
            description: "Compila compromissos da Agenda Google de hoje e quantidade de mensagens não lidas no Gmail, enviando um briefing diário para seu próprio e-mail às 07h."
                .to_string(),
            schedule_label: "Todo dia às 07h".to_string(),
            required_scopes: vec![
                "https://www.googleapis.com/auth/calendar.readonly".to_string(),
                "https://www.googleapis.com/auth/gmail.readonly".to_string(),
                "https://www.googleapis.com/auth/script.send_mail".to_string(),
                "https://www.googleapis.com/auth/script.scriptapp".to_string(),
            ],
            human_permissions: vec![
                "Consultar compromissos da sua Agenda Google (somente leitura)".to_string(),
                "Consultar contagem de e-mails não lidos no Gmail (somente leitura)".to_string(),
                "Enviar o resumo matinal para o seu próprio e-mail".to_string(),
                "NÃO envia dados para servidores externos".to_string(),
            ],
            files: vec![
                AppsScriptFileDto {
                    name: "Code".to_string(),
                    file_type: "SERVER_JS".to_string(),
                    source: r#"function sendMorningDigest() {
  var ownerEmail = Session.getEffectiveUser().getEmail();
  if (!ownerEmail) {
    return;
  }
  var today = new Date();
  var events = CalendarApp.getDefaultCalendar().getEventsForDay(today);
  var unread = GmailApp.getInboxUnreadCount();
  var lines = ["Bom dia! Resumo rápido do dia:", "- E-mails não lidos na Inbox: " + unread, "- Compromissos hoje (" + events.length + "):"];
  for (var i = 0; i < events.length; i++) {
    lines.push("  * " + events[i].getTitle() + " (" + events[i].getStartTime().toLocaleTimeString() + ")");
  }
  MailApp.sendEmail(ownerEmail, "[Souls Digest] Agenda e Inbox de Hoje", lines.join("\n"));
}

function installDailyMorningTrigger() {
  ScriptApp.newTrigger("sendMorningDigest").timeBased().everyDays(1).atHour(7).create();
}
"#
                    .to_string(),
                },
                manifest_file(&[
                    "https://www.googleapis.com/auth/calendar.readonly",
                    "https://www.googleapis.com/auth/gmail.readonly",
                    "https://www.googleapis.com/auth/script.send_mail",
                    "https://www.googleapis.com/auth/script.scriptapp",
                ]),
            ],
        },
    ]
}

fn manifest_file(scopes: &[&str]) -> AppsScriptFileDto {
    let manifest = json!({
        "timeZone": "America/Sao_Paulo",
        "exceptionLogging": "STACKDRIVER",
        "runtimeVersion": "V8",
        "oauthScopes": scopes,
    });
    let pretty = serde_json::to_string_pretty(&manifest).unwrap_or_else(|_| "{}".to_string());
    AppsScriptFileDto {
        name: "appsscript".to_string(),
        file_type: "JSON".to_string(),
        source: format!("{}\n", pretty.replace("\r\n", "\n")),
    }
}

/// Performs static security inspection of Apps Script source and manifest files,
/// producing risk warnings and a human-readable Portuguese permissions card.
pub fn scan_apps_script_security(files: &[AppsScriptFileDto]) -> AppsScriptStaticScanDto {
    let mut warnings = Vec::new();
    let mut declared_scopes = Vec::new();
    let mut manifest_json_valid = false;
    let mut manifest_found = false;

    for file in files {
        let is_manifest = file.file_type.eq_ignore_ascii_case("JSON")
            && (file.name.eq_ignore_ascii_case("appsscript")
                || file.name.eq_ignore_ascii_case("appsscript.json"));
        if is_manifest {
            manifest_found = true;
            match serde_json::from_str::<Value>(&file.source) {
                Ok(val) => {
                    manifest_json_valid = true;
                    if let Some(arr) = val.get("oauthScopes").and_then(|v| v.as_array()) {
                        for scope in arr {
                            if let Some(s) = scope.as_str() {
                                declared_scopes.push(s.to_string());
                            }
                        }
                    }
                }
                Err(err) => {
                    manifest_json_valid = false;
                    warnings.push(AppsScriptRiskWarningDto {
                        severity: "high".to_string(),
                        code: "invalid_manifest_json".to_string(),
                        message: format!("Manifesto appsscript.json possui JSON inválido: {err}"),
                    });
                }
            }
        }
    }

    if !manifest_found {
        warnings.push(AppsScriptRiskWarningDto {
            severity: "medium".to_string(),
            code: "missing_manifest".to_string(),
            message: "Manifesto appsscript.json ausente; permissões mínimas explícitas não foram declaradas."
                .to_string(),
        });
    }

    let mut external_urls = BTreeSet::new();
    let mut destructive_calls = BTreeSet::new();
    let mut hardcoded_secrets_detected = false;
    let mut combined_code = String::new();

    for file in files {
        if file.file_type.eq_ignore_ascii_case("JSON") {
            continue;
        }
        combined_code.push_str(&file.source);
        combined_code.push('\n');

        for url in extract_http_urls(&file.source) {
            if !url.starts_with("https://www.googleapis.com/auth/") {
                external_urls.insert(url);
            }
        }

        for &(pattern, label) in DESTRUCTIVE_CALL_PATTERNS {
            if file.source.contains(pattern) {
                destructive_calls.insert(label.to_string());
            }
        }

        if let Some(secret_kind) = detect_hardcoded_secret(&file.source) {
            hardcoded_secrets_detected = true;
            warnings.push(AppsScriptRiskWarningDto {
                severity: "high".to_string(),
                code: "hardcoded_secret".to_string(),
                message: format!(
                    "Possível segredo ou chave de API em texto claro detectado em '{}' ({secret_kind}). Remova antes de publicar.",
                    file.name
                ),
            });
        }
    }

    let external_urls_detected: Vec<String> = external_urls.into_iter().collect();
    let destructive_calls_detected: Vec<String> = destructive_calls.into_iter().collect();

    if !external_urls_detected.is_empty() || combined_code.contains("UrlFetchApp.fetch") {
        warnings.push(AppsScriptRiskWarningDto {
            severity: "medium".to_string(),
            code: "external_network_call".to_string(),
            message: format!(
                "O script realiza conexões HTTP externas ({} URL(s) detectada(s)).",
                external_urls_detected.len()
            ),
        });
    }

    if destructive_calls_detected
        .iter()
        .any(|c| c.contains("Trash") || c.contains("remove"))
    {
        warnings.push(AppsScriptRiskWarningDto {
            severity: "medium".to_string(),
            code: "trash_or_delete".to_string(),
            message: "O script contém chamadas que movem e-mails ou arquivos para a lixeira."
                .to_string(),
        });
    }

    for scope in &declared_scopes {
        if scope == "https://mail.google.com/"
            || scope == "https://www.googleapis.com/auth/drive"
        {
            warnings.push(AppsScriptRiskWarningDto {
                severity: "medium".to_string(),
                code: "broad_oauth_scope".to_string(),
                message: format!("Escopo amplo declarado no manifesto: {scope}"),
            });
        }
    }

    let human_permission_summary = build_human_permission_card(
        &declared_scopes,
        &combined_code,
        &external_urls_detected,
        &destructive_calls_detected,
    );

    let mut score: i32 = 100;
    if hardcoded_secrets_detected {
        score -= 40;
    }
    if manifest_found && !manifest_json_valid {
        score -= 30;
    } else if !manifest_found {
        score -= 10;
    }
    if !external_urls_detected.is_empty() || combined_code.contains("UrlFetchApp.fetch") {
        score -= 15;
    }
    let destructive_penalty = (destructive_calls_detected.len() as i32 * 8).min(30);
    score -= destructive_penalty;
    if declared_scopes
        .iter()
        .any(|s| s == "https://mail.google.com/" || s == "https://www.googleapis.com/auth/drive")
    {
        score -= 10;
    }

    AppsScriptStaticScanDto {
        safe_score: score.clamp(0, 100) as u8,
        manifest_json_valid,
        hardcoded_secrets_detected,
        external_urls_detected,
        destructive_calls_detected,
        human_permission_summary,
        warnings,
    }
}

const DESTRUCTIVE_CALL_PATTERNS: &[(&str, &str)] = &[
    ("UrlFetchApp.fetch", "UrlFetchApp.fetch"),
    ("GmailApp.sendEmail", "GmailApp.sendEmail"),
    ("MailApp.sendEmail", "MailApp.sendEmail"),
    ("setTrashed(true)", "setTrashed(true)"),
    ("moveThreadToTrash", "moveThreadToTrash"),
    ("moveThreadsToTrash", "moveThreadsToTrash"),
    ("ScriptApp.newTrigger", "ScriptApp.newTrigger"),
    ("removeFile", "DriveApp.removeFile"),
    ("removeFolder", "DriveApp.removeFolder"),
];

fn build_human_permission_card(
    scopes: &[String],
    combined_code: &str,
    external_urls: &[String],
    destructive_calls: &[String],
) -> Vec<String> {
    let mut card = Vec::new();
    let has_scope = |needle: &str| scopes.iter().any(|s| s == needle);

    if has_scope("https://mail.google.com/") {
        card.push("Acesso completo ao Gmail (leitura, organização, envio e lixeira)".to_string());
    } else {
        if has_scope("https://www.googleapis.com/auth/gmail.readonly") {
            card.push("Ler mensagens e metadados no Gmail (somente leitura)".to_string());
        }
        if has_scope("https://www.googleapis.com/auth/gmail.modify") {
            card.push("Ler, aplicar marcadores e organizar conversas no Gmail".to_string());
        }
        if has_scope("https://www.googleapis.com/auth/gmail.send")
            || has_scope("https://www.googleapis.com/auth/script.send_mail")
            || combined_code.contains("GmailApp.sendEmail")
            || combined_code.contains("MailApp.sendEmail")
        {
            card.push("Enviar e-mails usando a sua conta Gmail".to_string());
        }
    }

    if has_scope("https://www.googleapis.com/auth/drive") {
        card.push("Ler, criar e modificar arquivos no Google Drive".to_string());
    } else if has_scope("https://www.googleapis.com/auth/drive.file") {
        card.push("Criar e gerenciar apenas arquivos criados por esta automação no Drive".to_string());
    } else if has_scope("https://www.googleapis.com/auth/drive.readonly") {
        card.push("Ler arquivos no Google Drive (somente leitura)".to_string());
    }

    if has_scope("https://www.googleapis.com/auth/calendar")
        || has_scope("https://www.googleapis.com/auth/calendar.events")
    {
        card.push("Ler e alterar eventos na Agenda Google".to_string());
    } else if has_scope("https://www.googleapis.com/auth/calendar.readonly") {
        card.push("Consultar compromissos da Agenda Google (somente leitura)".to_string());
    }

    if has_scope("https://www.googleapis.com/auth/spreadsheets")
        || has_scope("https://www.googleapis.com/auth/spreadsheets.readonly")
    {
        card.push("Acessar dados em planilhas do Google Sheets".to_string());
    }

    if has_scope("https://www.googleapis.com/auth/forms.body")
        || has_scope("https://www.googleapis.com/auth/forms.responses.readonly")
    {
        card.push("Consultar respostas e estrutura no Google Forms".to_string());
    }

    if has_scope("https://www.googleapis.com/auth/script.external_request")
        || combined_code.contains("UrlFetchApp.fetch")
        || !external_urls.is_empty()
    {
        card.push("Realizar chamadas HTTP para endereços externos".to_string());
    }

    if has_scope("https://www.googleapis.com/auth/script.scriptapp")
        || combined_code.contains("ScriptApp.newTrigger")
    {
        card.push("Criar e manter agendamentos automáticos 24/7 na sua conta Google".to_string());
    }

    if card.is_empty() {
        card.push("Execução isolada sem permissões externas declaradas no manifesto".to_string());
    }

    // Explicit negative guarantees (what it CANNOT do)
    let can_send_email = has_scope("https://mail.google.com/")
        || has_scope("https://www.googleapis.com/auth/gmail.send")
        || has_scope("https://www.googleapis.com/auth/script.send_mail")
        || combined_code.contains("GmailApp.sendEmail")
        || combined_code.contains("MailApp.sendEmail");
    if !can_send_email {
        card.push("NÃO pode enviar e-mails em seu nome".to_string());
    }

    let can_call_external = has_scope("https://www.googleapis.com/auth/script.external_request")
        || combined_code.contains("UrlFetchApp.fetch")
        || !external_urls.is_empty();
    if !can_call_external {
        card.push("NÃO envia dados para servidores externos".to_string());
    }

    let can_trash_or_delete = destructive_calls.iter().any(|c| {
        c.contains("setTrashed") || c.contains("Trash") || c.contains("remove")
    }) || has_scope("https://mail.google.com/");
    if !can_trash_or_delete {
        card.push("NÃO apaga nem move arquivos ou e-mails para a lixeira".to_string());
    }

    card
}

fn extract_http_urls(src: &str) -> Vec<String> {
    let mut urls = Vec::new();
    let mut cursor = 0usize;

    while cursor < src.len() {
        let rest = &src[cursor..];
        let next_https = rest.find("https://").map(|p| (p, 8usize));
        let next_http = rest.find("http://").map(|p| (p, 7usize));
        let found = match (next_https, next_http) {
            (Some(a), Some(b)) => Some(if a.0 <= b.0 { a } else { b }),
            (Some(a), None) => Some(a),
            (None, Some(b)) => Some(b),
            (None, None) => None,
        };
        let Some((rel_pos, scheme_len)) = found else {
            break;
        };

        let abs_start = cursor + rel_pos;
        let mut end = abs_start + scheme_len;
        let bytes = src.as_bytes();
        while end < bytes.len() {
            let b = bytes[end];
            if b.is_ascii_whitespace()
                || matches!(b, b'"' | b'\'' | b'`' | b'<' | b'>' | b')' | b']' | b'}' | b';' | b',')
            {
                break;
            }
            end += 1;
        }
        if end > abs_start + scheme_len {
            urls.push(src[abs_start..end].to_string());
        }
        cursor = end;
    }

    urls
}

fn detect_hardcoded_secret(src: &str) -> Option<&'static str> {
    if has_token_prefix(src, "AIza", 16, |b| b.is_ascii_alphanumeric() || b == b'_' || b == b'-') {
        return Some("Google API Key (AIza...)");
    }
    if has_token_prefix(src, "ya29.", 12, |b| {
        b.is_ascii_alphanumeric() || b == b'.' || b == b'_' || b == b'-'
    }) {
        return Some("Google OAuth Token (ya29....)");
    }
    if has_token_prefix(src, "GOCSPX-", 12, |b| b.is_ascii_alphanumeric() || b == b'_' || b == b'-') {
        return Some("Google Client Secret (GOCSPX-...)");
    }
    if has_token_prefix(src, "sk-", 16, |b| b.is_ascii_alphanumeric() || b == b'_' || b == b'-') {
        return Some("OpenAI / API Secret Key (sk-...)");
    }
    if has_token_prefix(src, "ghp_", 16, |b| b.is_ascii_alphanumeric() || b == b'_') {
        return Some("GitHub Personal Access Token (ghp_...)");
    }
    if has_token_prefix(src, "xoxb-", 12, |b| b.is_ascii_alphanumeric() || b == b'-') {
        return Some("Slack Bot Token (xoxb-...)");
    }
    if has_telegram_bot_token(src) {
        return Some("Telegram Bot Token (bot<id>:<secret>)");
    }
    None
}

fn has_token_prefix<F>(src: &str, prefix: &str, min_tail: usize, is_valid_char: F) -> bool
where
    F: Fn(u8) -> bool,
{
    let mut cursor = 0usize;
    while let Some(rel) = src[cursor..].find(prefix) {
        let start = cursor + rel;
        let tail = &src.as_bytes()[start + prefix.len()..];
        let count = tail.iter().take_while(|&&b| is_valid_char(b)).count();
        if count >= min_tail {
            return true;
        }
        cursor = start + prefix.len();
    }
    false
}

fn has_telegram_bot_token(src: &str) -> bool {
    let mut cursor = 0usize;
    while let Some(rel) = src[cursor..].find("bot") {
        let start = cursor + rel;
        let after = &src.as_bytes()[start + 3..];
        let digits = after.iter().take_while(|b| b.is_ascii_digit()).count();
        if (6..=14).contains(&digits) && after.get(digits) == Some(&b':') {
            let secret = &after[digits + 1..];
            let token_chars = secret
                .iter()
                .take_while(|&&b| b.is_ascii_alphanumeric() || b == b'_' || b == b'-')
                .count();
            if token_chars >= 18 {
                return true;
            }
        }
        cursor = start + 3;
    }
    false
}

fn parse_project_dto(val: &Value) -> AppsScriptProjectDto {
    let script_id = val
        .get("scriptId")
        .or_else(|| val.get("script_id"))
        .and_then(|v| v.as_str())
        .unwrap_or_default()
        .to_string();
    let title = val
        .get("title")
        .and_then(|v| v.as_str())
        .unwrap_or("Untitled Script")
        .to_string();
    let parent_id = val
        .get("parentId")
        .or_else(|| val.get("parent_id"))
        .and_then(|v| v.as_str())
        .map(ToString::to_string);
    let create_time = val
        .get("createTime")
        .or_else(|| val.get("create_time"))
        .and_then(|v| v.as_str())
        .map(ToString::to_string);
    let update_time = val
        .get("updateTime")
        .or_else(|| val.get("update_time"))
        .and_then(|v| v.as_str())
        .map(ToString::to_string);
    let editor_url = if script_id.is_empty() {
        None
    } else {
        Some(format!("https://script.google.com/d/{script_id}/edit"))
    };

    AppsScriptProjectDto {
        script_id,
        title,
        parent_id,
        create_time,
        update_time,
        editor_url,
    }
}

fn parse_files_array(val: &Value) -> Vec<AppsScriptFileDto> {
    val.get("files")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .map(|f| AppsScriptFileDto {
                    name: f
                        .get("name")
                        .and_then(|v| v.as_str())
                        .unwrap_or_default()
                        .to_string(),
                    file_type: f
                        .get("type")
                        .or_else(|| f.get("file_type"))
                        .and_then(|v| v.as_str())
                        .unwrap_or("SERVER_JS")
                        .to_string(),
                    source: f
                        .get("source")
                        .and_then(|v| v.as_str())
                        .unwrap_or_default()
                        .to_string(),
                })
                .collect()
        })
        .unwrap_or_default()
}

fn parse_deployment_dto(val: &Value) -> AppsScriptDeploymentDto {
    let deployment_id = val
        .get("deploymentId")
        .or_else(|| val.get("deployment_id"))
        .and_then(|v| v.as_str())
        .unwrap_or_default()
        .to_string();
    let cfg = val.get("deploymentConfig").unwrap_or(val);
    let version_number = cfg
        .get("versionNumber")
        .or_else(|| cfg.get("version_number"))
        .and_then(|v| v.as_i64().or_else(|| v.as_str()?.parse::<i64>().ok()));
    let description = cfg
        .get("description")
        .and_then(|v| v.as_str())
        .map(ToString::to_string);
    let entry_points = val
        .get("entryPoints")
        .or_else(|| val.get("entry_points"))
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|ep| {
                    ep.as_str().map(ToString::to_string).or_else(|| {
                        ep.get("entryPointType")
                            .and_then(|v| v.as_str())
                            .map(ToString::to_string)
                    })
                })
                .collect()
        })
        .unwrap_or_default();
    let update_time = val
        .get("updateTime")
        .or_else(|| val.get("update_time"))
        .and_then(|v| v.as_str())
        .map(ToString::to_string);

    AppsScriptDeploymentDto {
        deployment_id,
        version_number,
        description,
        entry_points,
        update_time,
    }
}

fn parse_process_dto(val: &Value) -> AppsScriptProcessDto {
    AppsScriptProcessDto {
        project_name: val
            .get("projectName")
            .or_else(|| val.get("project_name"))
            .and_then(|v| v.as_str())
            .map(ToString::to_string),
        function_name: val
            .get("functionName")
            .or_else(|| val.get("function_name"))
            .and_then(|v| v.as_str())
            .unwrap_or_default()
            .to_string(),
        process_type: val
            .get("processType")
            .or_else(|| val.get("process_type"))
            .and_then(|v| v.as_str())
            .unwrap_or("PROCESS_TYPE_UNSPECIFIED")
            .to_string(),
        process_status: val
            .get("processStatus")
            .or_else(|| val.get("process_status"))
            .and_then(|v| v.as_str())
            .unwrap_or("PROCESS_STATUS_UNSPECIFIED")
            .to_string(),
        start_time: val
            .get("startTime")
            .or_else(|| val.get("start_time"))
            .and_then(|v| v.as_str())
            .map(ToString::to_string),
        duration: val
            .get("duration")
            .and_then(|v| v.as_str())
            .map(ToString::to_string),
    }
}
