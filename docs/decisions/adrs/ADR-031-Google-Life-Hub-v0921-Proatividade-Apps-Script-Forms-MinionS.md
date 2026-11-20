# ADR-031: Google Life Hub v0.9.2.1 — Pulso Proativo Multi-Cadência, Apps Script 24/7, Forms, Chat Workspace e Leitura Econômica MinionS

## Status
**Aprovado e Implementado** (Milestone v0.9.2.1)

## Data
2026-11-19

## Contexto e Forças em Conflito
Com a entrega da Milestone v0.9.2 ([ADR-030](file:///Z:/souls_engine/docs/decisions/adrs/ADR-030-Google-Personal-Life-Hub-BMAD-Inbox-e-Hibrido-Seguro.md)), o Souls Engine estabeleceu a ponte soberana e o controle BMAD Agent Inbox para o ecossistema Google. Contudo, cinco lacunas operacionais ainda separavam o Life Hub de um assistente verdadeiramente proativo e autônomo no cotidiano do operador:

1. **Reatividade Passiva (Falta de Pulso Proativo):** O motor dependia de consultas sob demanda do usuário, sem detectar antecipadamente expiração iminente de OAuth, trânsito real antes de reuniões iminentes, e-mails VIP urgentes, novas respostas de formulários ou fontes desatualizadas entre Google Drive e NotebookLM — e qualquer disparo ingênuo em loop único geraria saturação de cota (HTTP 429) e poluição de *Prefix Caching* no Hermes Agent.
2. **Automações 24/7 na Nuvem Google (Apps Script sem Blindagem):** Automações contínuas no Gmail/Drive/Forms (mesmo com o PC desligado) exigem Google Apps Script, mas a publicação cega de scripts por IA traz risco de manifesto inválido (`appsscript.json`), vazamento de chaves hardcoded no JS e permissões opacas para o operador humano.
3. **Lacuna de Coleta Estruturada e Mensageria Corporativa:** Faltavam clientes dedicados para **Google Forms API v1** (suportando contas pessoais `@gmail.com`) e **Google Chat API v1** (restrito ao Google Workspace corporativo, exigindo bloqueio explícito para contas pessoais).
4. **Custo e Latência na Leitura de Documentos Google Extensos:** Ler PDFs grandes (até 25 MB), Google Docs longos ou threads extensas do Gmail diretamente no contexto de LLMs em nuvem desperdiça tokens e dinheiro; faltava uma cascata local determinística + SLM com cache em disco.
5. **Desatualização de Vínculos Drive ➔ NotebookLM e Atrito de Escopos OAuth:** Documentos editados via Docs/Sheets não disparavam sincronização automática (`FLmJqe`) com cadernos NotebookLM vinculados, e contas autenticadas na v0.9.2 precisavam de atualização incremental (`include_granted_scopes=true`) para os novos escopos sem perder permissões já concedidas.
6. **Riscos de Ecossistema Obsoleto/Inseguro:** O Google Jules entrou em modo *read-only* em 12/11/2026 com desligamento definitivo previsto para 31/12/2026, e o scraping furtivo de cookies (`chaser-oxide` / evasão SearchGuard) continua representando risco inaceitável de bloqueio da conta Google do usuário.

---

## Decisão de Arquitetura

A Milestone **v0.9.2.1** (`version = "0.9.2+1"` em `Cargo.toml` sob SemVer 2.0, preservando `serverInfo.version = "7.2.0"` no handshake MCP) implementa cinco pilares integrados em `souls_protocol` ➔ `souls_core` ➔ `souls_server` ➔ `adapters/hermes_memory_souls`:

### 1. Pulso Proativo Multi-Cadência, Quiet Hours e Entrega Tripla (`pulse.rs`, `google_hub.rs`, `main.rs`)
- **Heartbeat Base Dinâmico de 5 Minutos (`300s`):** Uma task Tokio em `souls_server` verifica `pulse.is_due(now)` a cada 30 segundos, orquestrando cadências escalonadas por multiplicador de tick (`PulseConfigDto`):
  - **A cada 1 tick (5 min):** Saúde de OAuth (checagem em RAM; renova apenas se faltar `< 10 min` para expirar) e **Agenda + Trânsito Real (Routes v2)** disparado **somente** quando há evento geolocalizado nos próximos **90 minutos** (zero chamadas no restante do dia).
  - **A cada 2 ticks (10 min):** Triagem delta de **Gmail VIP / Urgentes** via cursor `gmail_last_seen_ts`.
  - **A cada 3 ticks (15 min):** **Google Forms monitorados** (detecção de novas respostas por cursor `seen_form_responses_diff`) e **Drive ↔ NotebookLM Freshness/Auto-Sync**.
- **Backoff Isolado por Monitor:** Falhas transitórias ou HTTP 429 em um monitor específico (`record_monitor_error`) colocam apenas aquele monitor em cooldown por **2 ciclos**, preservando os demais monitores ativos.
- **Quiet Hours Configurável e Desligável:** `quiet_hours_enabled` (padrão `true`), `quiet_hours_start` (padrão `22`) e `quiet_hours_end` (padrão `8`, ambos `0..=23`) avaliados via `is_within_quiet_hours`, com controles diretos na UI.
- **Entrega Tripla Zero-Cache-Break:**
  1. **Dashboard SPA:** Stream SSE em tempo real (`/api/v1/google/events/stream`) + feed no `Google Life Hub` + Web Notification API opt-in.
  2. **Hermes Agent (`adapters/hermes_memory_souls/__init__.py`):** `prefetch()` executa consulta leve *fail-open* (`timeout=1.0s`) em `GET /api/v1/google/pulse/pending?limit=3&drain=true` e injeta o bloco efêmero `[PULSO GOOGLE — ALERTAS RECENTES]` no turno sem jamais alterar `system_prompt_block()` (preservando 100% do *Prefix Caching*).
  3. **Canais Externos Opcionais:** Webhook / Telegram Bot (`PulseSecrets` isolados em `.souls_data/google/`) acionados exclusivamente para urgência `High`/`Critical`, respeitando Quiet Hours, cooldown (`external_cooldown_mins`, padrão `30`) e higienização via `sanitize_for_external_llm`.

### 2. Google Apps Script 24/7 via REST Nativo, Scanner Estático e Cartão Humano (`apps_script.rs`)
- **Transporte REST Nativo + Probe `clasp`:** `GoogleAppsScriptClient` opera diretamente contra `https://script.googleapis.com/v1` (com `ClaspCliBridge` opcional para diagnóstico).
- **5 Templates Curados de Menor Privilégio (`builtin_apps_script_templates`):** `gmail_bills_organizer`, `drive_attachments_archiver`, `forms_instant_notifier`, `gmail_promo_cleaner` e `morning_digest_cloud`.
- **Auditoria Estática & Cartão de Permissões em Português Humano:** `scan_apps_script_security()` valida a sintaxe JSON do manifesto `appsscript.json`, detecta chaves/segredos colados no JS (`AIza`, `ya29.`, `GOCSPX-`, `ghp_`, etc.), sinaliza chamadas `UrlFetchApp`/destrutivas e gera `human_permission_summary: Vec<String>` exibido como cartão legível na **Agent Inbox** antes da aprovação.
- **Governança BMAD Obrigatória:** Toda operação `push`, `deploy` ou `run_function` passa obrigatoriamente por proposta com **Myers Diff** + **Gravity Lock** (`Tier2ExternalDestructive`).

### 3. Google Forms API v1 e Google Chat API v1 (`forms_chat.rs`)
- **Google Forms (`GoogleFormsClient`):** Suportado nativamente em contas pessoais `@gmail.com` e Workspace. Oferece leitura de esquema, criação governada via BMAD Agent Inbox, listagem de respostas e agregação estatística determinística (`compute_form_stats`) com geração de `markdown_summary` e botões 1-clique na UI para salvar no **Obsidian (`raw/inbox/`)** ou enviar para **Gemini Notebooks**.
- **Google Chat (`GoogleChatClient`):** Restrito exclusivamente a contas corporativas (`check_chat_eligibility` bloqueia `PersonalGmail` e `SecondarySandbox` com orientação clara), além de suporte a `send_incoming_webhook` validando o prefixo oficial `https://chat.googleapis.com/v1/spaces/` e governança via BMAD Agent Inbox.

### 4. Leitura Econômica MinionS Google Deep Analyze (`minions_google.rs`, `google_hub.rs`)
- **Cascata Inteligente (`MinionAnalyzeMode`):** `auto` | `fast_deterministic` | `force_local_slm` operando sobre Google Docs, PDFs do Drive (download limitado a 25 MB), threads do Gmail, Forms ou texto bruto.
- **Reuso Bare-Metal Tier 0 + Cache em Disco:** `prepare_google_corpus_tier0()` reaproveita `souls_core::research::{pdf_extractor, html_cleaner, chunker, bm25}`, aplica `wrap_untrusted_content`, mede redução de tokens (`estimated_token_savings_pct`) e mantém cache em `.souls_data/google/minions_cache/` indexado por `(source_id, content_hash)` em SHA-256 para respostas repetidas em sub-5 ms (`cache_hit: true`).
- **Guarda de Silício Real:** Antes de acionar `MinionBatchOrchestrator` (teto WDDM de 4,8 GB ou fallback CPU AVX2), `run_google_minions_analysis` verifica a existência de modelo `.gguf` local (`resolve_local_minions_model`) e do runtime `llama-completion`, degradando graciosamente para síntese determinística se ausentes.
- **Exportação 1-Clique:** Resultados retornam `markdown_export` com ações diretas na UI para salvar em `raw/inbox/` do Obsidian ou ingerir como fonte no NotebookLM.

### 5. Auto-Sync Drive ↔ NotebookLM & Escopos OAuth Incrementais (`pulse.rs`, `oauth.rs`, `google_hub.rs`)
- **Vínculos Vivos (`DriveNotebookSyncLinkDto` em `sync_links.json`):** Permite vincular arquivos Drive a cadernos NotebookLM com `auto_sync_enabled` toggleável em 1 clique. O heartbeat verifica frescura (`yR9Yof`) a cada 15 min e um gancho pós-push dispara `trigger_drive_notebook_autosync` imediatamente após aprovações bem-sucedidas de escrita em Google Docs ou Sheets (`FLmJqe` automático nos vínculos habilitados; alerta acionável nos demais).
- **OAuth Incremental (`RFC 7636` + `include_granted_scopes=true`):** `build_google_auth_url` inclui `include_granted_scopes=true` por padrão, enquanto `recommended_scopes_for_kind` e `missing_v0921_scopes` preenchem `missing_recommended_scopes` em `GoogleAccountProfileDto`, habilitando o botão 1-clique **"Atualizar permissões (Automações 24/7 e Formulários)"** na UI sem revogar escopos prévios.

### 6. Exclusões Arquiteturais Explícitas
- **Google Jules Definitivamente Excluído:** Com o congelamento *read-only* em 12/11/2026 e encerramento em 31/12/2026, nenhum cliente Jules é adicionado; o `JulesMigrationRouter` ➔ Antigravity (`agy`) já cobre a continuidade operacional.
- **`chaser-oxide` / Cookie-Scraping Mantido Excluído:** Nenhuma evasão anti-bot ou scraping por cookies de browser é introduzida, preservando a integridade da conta Google principal do operador.

---

## Consequências

### Positivas
- **Proatividade com Custo Zero de Tokens em Ociosidade:** Checagens determinísticas em RAM/delta evitam chamadas redundantes a APIs externas e mantêm o *Prefix Caching* do Hermes Agent intacto.
- **Segurança Auditável em Automações Cloud:** Apps Script e Chat/Forms passam pelo mesmo rigor de Myers Diff + Gravity Lock + Cartão de Permissões em Português Humano na Agent Inbox.
- **Economia Mensurável em Documentos Grandes:** MinionS Tier 0 + cache SHA-256 reduzem drasticamente o envio de texto bruto para modelos em nuvem e habilitam síntese local sem spillover PCIe.

### Negativas / Riscos Mitigados
- **Reautenticação Incremental para Usuários Existentes:** Mitigada por `include_granted_scopes=true` e detecção automática de `missing_recommended_scopes` com atualização em 1 clique em `Conexões & CLIs` e banners contextuais.
- **Limitações de Conta Pessoal vs Corporativa:** A API oficial do Google Chat é bloqueada preventivamente para `@gmail.com` via `check_chat_eligibility`, mantendo apenas Webhooks opcionais e evitando erros crípticos de API.
