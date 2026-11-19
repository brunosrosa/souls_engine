# ADR-030: Google Personal & Life Hub, Governança BMAD Agent Inbox e Arquitetura Híbrida Segura

## Status
**Aprovado e Implementado** (Milestone v0.9.2)

## Data
2026-11-18

## Contexto e Forças em Conflito
Na evolução do Souls Engine da fundação v0.8.2 (`Google Universe Bridge` / `GV1`) e do Design System v0.9.1 (`Precision Dark UI`) para a Milestone v0.9.2 (**Google Personal & Life Hub**), identificou-se que a automação de produtividade pessoal e profissional no ecossistema Google de 2026 enfrenta cinco tensões críticas entre utilidade, confiabilidade de APIs e segurança soberana:

1. **Risco de Bloqueio de Conta Primária por Scraping/Cookies:** O uso de protocolos internos não documentados (como `StreamGenerate` via cookies de sessão) em contas pessoais primárias `@gmail.com` expõe a identidade principal do operador a bloqueios anti-bot e revogações de sessão, ao mesmo tempo em que o Google oferece APIs REST oficiais para Workspace, Tasks, People, Maps e Gemini (`generativelanguage.googleapis.com`).
2. **Fricções Reais das APIs Google em 2026:**
   - A **Google Keep API** permanece restrita exclusivamente a domínios **Google Workspace Enterprise** com *Domain-Wide Delegation*, falhando em contas `@gmail.com` pessoais.
   - A **Directions API Legacy** entrou em *End of Sale*, exigindo migração para a **Google Maps Routes API v2** (`computeRoutes`) com `X-Goog-FieldMask` obrigatório.
   - Aplicativos OAuth2 no Google Cloud Console em estado **Testing** têm *refresh tokens* expirados a cada **7 dias**, degradando a automação contínua caso o operador não alterne para **In Production**.
   - A tabela de RPCs Wiz do NotebookLM na v0.8.2 exigia cobertura completa e mapeamento exato dos 22+ RPCs reais (`wiz_rpc.rs`) e integração com o `nlm` CLI (`NlmCliBridge`) + streaming local de áudio HTTP Range.
3. **Injeção de Prompt Indireta e Mutações Irreversíveis:** Ler e-mails de terceiros, arquivos compartilhados do Drive ou legendas do YouTube dentro da mesma sessão em que o agente possui ferramentas de envio de e-mail, exclusão ou compartilhamento cria um vetor clássico de *Indirect Prompt Injection* (exfiltração ou sabotagem autônoma).
4. **Higiene de Interface & Coesão Visual:** A v0.9.1 consolidou os **6 Pilares de Produto**, o sprite SVG e eliminou atributos `style="..."` inline. Introduzir novas abas de Life Hub, Gemini Notebooks e Agent Inbox sem respeitar essa arquitetura causaria regressão visual ou fragmentação da navegação.
5. **Single Source of Truth (SSOT) & Regra da Crate Única (`AGENTS.md` §2.1):** Os novos contratos precisam fluir com fronteiras limpas sem criar dependências cíclicas nem espalhar dados fora do diretório canônico de dados.

---

## Decisão de Arquitetura

Consolidamos cinco decisões estruturantes oriundas da deliberação `/grill-me`, implementadas em etapas atômicas ao longo de `souls_protocol` ➔ `souls_core` ➔ `souls_server`:

### 1. Híbrido Seguro de Transporte (APIs Oficiais + OAuth2 PKCE + Contorno Dual-Layer)
- **Workspace & Life Hub via REST Oficial + OAuth2 PKCE (`RFC 7636 S256`):**
  - `GoogleWorkspaceClient` (`Gmail`, `Calendar`, `Drive`, `Docs`, `Sheets`, `Slides`), `GoogleTasksClient`, `PeopleClient`, `RoutesV2Client` e `YouTubeClient` implementados em Rust puro (`reqwest`) com suporte a `base_url` override para testes de integração offline com `TcpListener` mock.
  - Fluxo nativo PKCE `S256` (`/api/v1/google/oauth/start` e `/api/v1/google/oauth/callback`), importação de credenciais `~/.config/gws/` (`import_gws_credentials`) e diagnóstico unificado `run_google_doctor` para `gws`, `nlm`, `gemini` e `clasp`.
- **NotebookLM Híbrido (Wiz `batchexecute` + `nlm` CLI):**
  - Emulação bare-metal de `WizRpcClient` com autenticação `SAPISIDHASH` (SHA-1 RFC 3174 puro em Rust) + ponte `NlmCliBridge` e cache local de áudio com streaming HTTP `Range` (`/api/v1/google/notebooks/:id/audio/:artifact_id`).
- **Custom Gems Seguro:**
  - Catálogo local `GemsCatalogStore` (`gems.json`) com despacho padrão via **Gemini API Oficial** (`POST /v1beta/models/{model}:generateContent` em `dispatch_gem_official_api`).
  - O protocolo `StreamGenerate` baseado em cookies (`build_secondary_stream_generate_payload`) é **terminantemente bloqueado** em contas primárias (`is_primary == true`) ou sem `secondary_web_opt_in == true`, retornando `CoreError::DangerousCommandBlocked` / `McpDomainError::HitlDenied`.

### 2. Escopo Life Hub & Contornos Oficiais Validados para 2026
- **Google Keep Enterprise ➔ Google Tasks + Obsidian Quick Capture:** `check_keep_eligibility` bloqueia de forma explicativa contas `PersonalGmail` / `SecondarySandbox` antes de tentar chamadas Keep inválidas; como contorno oficial soberano, `google_tasks(action="quick_capture")` / `POST /api/v1/google/quick-capture` grava notas Markdown com frontmatter YAML e finais de linha **LF puro** diretamente em `raw/inbox/` do cofre Obsidian, após higienização via `sanitize_for_external_llm`.
- **Directions Legacy ➔ Google Maps Routes API v2:** `RoutesV2Client` consome `POST /directions/v2:computeRoutes` com `routingPreference: "TRAFFIC_AWARE"` e header obrigatório `X-Goog-FieldMask` (`routes.duration,routes.staticDuration,routes.distanceMeters,routes.localizedValues`), alimentando `compute_commute_alert` e o `DailyBriefingDto`.
- **OAuth Testing 7-Day Expiration ➔ Orientação In Production:** `GoogleAccountProfileDto` expõe `oauth_testing_mode_warning`, e `run_google_doctor` emite orientação explícita para alternar o consent screen no Google Cloud Console de *Testing* para *In Production* (permitindo até 100 usuários pessoais sem revisão pública da Google).
- **Correção Completa dos 22+ RPCs Wiz (`wiz_rpc.rs`):** Mapeamento completo e resolução por ID ou alias em `WizRpcId` / `resolve_wiz_rpc_id` (incluindo `wXbhsf`, `rLM1Ne`, `CCqFvf`, `s0tc2d`, `WWINqb`, `JFMDGd`, `izAoDd`, `ozz5Z`, `hizoJc`, `yR9Yof`, `FLmJqe`, `b7Wfje`, `tGMBJ`, `tr032e`, `hPTbtc`, `khqZz`, `Ljjv0c`, `QA9ei`, `e3bVqc`, `LBwxtb`, `R7cb6c`, `gArtLc`, `v9rmvd`, `rc3d8d`, `KmcKPe`, `yyryJe`, `CYK0Xb`, `cFji9`, `orqRsd`, além dos aliases legados `TYPJF`/`cXJTyd` preservados para retrocompatibilidade).
- **Produtividade Determinística Zero-LLM:**
  - Triagem prioritária de Gmail, busca de *Deep Focus Slots* no Calendar, edição incremental por linhas em `DocsScratchpadStore` antes do push, análise quantitativa de planilhas no Sheets (totais, categorias, anomalias `> 2x` média, projeção 30d), planejamento estruturado de Slides (5 a 15 slides), aniversários próximos no People API e extração/cache de legendas YouTube (Innertube `ANDROID` + parser XML/SRV3 em Rust puro).

### 3. Governança em 3 Tiers, Taint Guard, Perfis Multi-Conta e BMAD Agent Inbox
- **3 Tiers de Blast Radius (`BlastRadiusTier`):**
  - `Tier0ReadOnly`: Leituras e análises determinísticas (`AllowDirect`).
  - `Tier1SafeReversible`: Rascunhos, criação de tasks, quick capture, criação de eventos com `send_updates="none"` e edições locais no scratchpad (**HOTL** em sessão limpa; **HITL + Gravity Lock** se a sessão estiver *tainted*).
  - `Tier2ExternalDestructive`: Envios de e-mail, operações em lote no Gmail, compartilhamento/exclusão no Drive, exclusão ou convite externo no Calendar, `push` no Docs, `write` no Sheets e `create` no Slides (**HITL obrigatório** via BMAD Agent Inbox).
- **4 Políticas Multi-Conta (`AccountPermissionPolicy`):** `ReadOnly`, `DraftsAndTasksOnly`, `BalancedHotlHitl` e `WhitelistWrite` avaliadas deterministicamente em `evaluate_action_policy`.
- **Untrusted Content Framing & Session Taint Tracker (`security.rs`):**
  - `wrap_untrusted_content` isola todo conteúdo externo com delimitadores nonce aleatórios de 16 hex (`<<<SOULS_UNTRUSTED_DATA:<nonce> source="...">>>`) e neutraliza spoofing (`[REDACTED_MARKER]`).
  - `SessionTaintTracker` (TTL padrão de **1800s / 30 min**) marca sessões contaminadas após leituras externas, escalando escritas subsequentes para HITL com Gravity Lock.
- **BMAD Agent Inbox (`AgentInboxStore` + `GoogleHubService::propose_with_diff`):**
  - Propostas computam **Myers Diff** real via `souls_ast::compute_safe_myers_diff` e exigem confirmação explícita (`gravity_lock_confirmed: true` + comparação em tempo constante do token `glock_*`) para aprovação e execução via `/api/v1/inbox/proposals/:id/approve`.
  - Toda aprovação, rejeição ou falha gera evento redigido (`redact_google_secrets` + `PiiRedactor`) em `audit_log.jsonl`.

### 4. UX Integrada aos 6 Pilares & Badge Global de Inbox
- Sem criar poluição visual ou estilos inline (`style="..."` zero):
  - **Pilar Cockpit:** Sub-navegação contextual integrando `Cockpit & Pulso`, `Google Life Hub` (`tabs/google_hub.html`) e `Agent Inbox` (`tabs/agent_inbox.html`), além do **Badge Global de Inbox** persistente na `topbar.html` (`#pill-agent-inbox` / `#top-inbox-count`) e na barra contextual (`#badge-inbox-count`).
  - **Pilar Pesquisa:** Integração da sub-aba `Gemini Notebooks` (`tabs/gemini_notebooks.html`) com galeria, sincronização Drive, RAG query, Studio Audio Overview e player HTML5 conectado ao streaming HTTP Range local.
  - **Pilar Sistema:** Painel de contas GV2, OAuth2 PKCE, importação `gws` e Google Doctor integrados em `tabs/connections.html` + `scripts/connections.js` e `scripts/google_hub.js`.

### 5. SSOT Atômico em `.souls_data/google/` & Regra da Crate Única
- Persistência isolada sob `default_google_data_dir()` (`SOULS_GOOGLE_DATA_DIR`, `SOULS_DATA_DIR/google`, ou `Z:\souls_engine\.souls_data\google`):
  - `accounts_v2.vault` (cofre multi-conta encriptado **GV2**, preservando retrocompatibilidade com **GV1** `google_sessions.vault`)
  - `inbox/*.json` + `audit_log.jsonl`
  - `scratchpads/*.json`, `gems.json`, `audio_cache/`, `exchange/`, `youtube_cache/`
  - Todas as gravações JSON/texto usam escrita atômica via arquivo temporário irmão + `rename` com **finais de linha LF puro (`\n`)**.
- **Camadas em Crate Única:**
  1. `crates/souls_protocol/src/google.rs`: DTOs e enums puros sem I/O e sem tokens em claro.
  2. `crates/souls_core/src/google_bridge/`: Clientes REST, Wiz RPC, OAuth2 PKCE, segurança BMAD, cofre GV2, inbox e testes offline.
  3. `crates/souls_server/`: Serviço `GoogleHubService` (`google_hub.rs`), 39 rotas REST (`routes/google.rs` + 5 rotas `routes/inbox.rs`), 17 novas ferramentas MCP (`mcp/tools_google.rs` + `mcp/registry.rs`), Switchboard (`google-life-hub` como 16ª skill em `mcp/switchboard.rs`) e UI SPA modular.

---

## Consequências

### Positivas
- **Soberania & Segurança contra Indirect Prompt Injection:** Nenhuma leitura externa pode disparar mutação externa silenciosa graças ao tripé *Untrusted Content Framing + SessionTaintTracker + BMAD Gravity Lock*.
- **Proteção da Conta Principal:** Banimento arquitetural de `StreamGenerate` via cookies na conta primária e uso de OAuth2 PKCE + APIs oficiais + contorno Tasks/Obsidian para Keep.
- **Catálogo Unificado e Descoberta Progressiva:** O barramento MCP cresce para **80 ferramentas canônicas** (7 Boot Level 0 + 73 progressivas) e **16 skills canônicas** sem degradar o *Prefix Caching* do chat principal.
- **Testabilidade Offline Determinística:** Todos os clientes REST e fluxos BMAD operam contra mocks HTTP `TcpListener` locais sem depender de rede externa ou credenciais reais em CI.

### Negativas / Riscos Mitigados
- **Complexidade Adicional de Configuração Inicial (Google Cloud OAuth & Routes Key):** Mitigada pelo `google_accounts(action="doctor")`, importação automática de `~/.config/gws/` (`import_gws`) e degradação graciosa do `DailyBriefingDto` quando offline.
- **Expiração Semanal de Refresh Tokens em Apps Testing:** Mitigada com flag explícita `oauth_testing_mode_warning` na UI/Doctor e instrução passo a passo para alternar para *In Production*.
