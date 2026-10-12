# DESIGN.md — Especificação de Arquitetura e Contratos Técnicos (`souls_server_mcp`)

> **SUBSISTEMA ALVO:** `crates/souls_server` (Módulo MCP)  
> **ESTATUS DO DOCUMENTO:** DESIGN TÉCNICO VINCULANTE (DOCS-AS-GUARDRAILS)  
> **COMPILADOR ALVO:** MSVC (`x86_64-pc-windows-msvc`) sobre Dev Drive ReFS  
> **DEPENDÊNCIAS RESTRITAS:** Exclusivamente crates do workspace e dependências homologadas no Cargo.toml raiz.

---

## 1. TOPOLOGIA DE MÓDULOS DE `crates/souls_server`

```
crates/souls_server/src/
├── main.rs                  // Bootstrap, CLI flags (--data-dir), socket listener :9123
├── state.rs                 // AppState com injeção do McpSessionManager
├── shutdown.rs              // Sequência atômica de 5 fases de encerramento
├── routes/                  // Rotas HTTP existentes
│   ├── mod.rs               // Router Axum integrando endpoints REST e MCP
│   ├── health.rs            // GET /health (NVML, SQLite, Conexões)
│   ├── models.rs            // GET /v1/models (Model Arms do SQLite)
│   └── chat.rs              // POST /v1/chat/completions (OpenAI Compatible)
└── mcp/                     // Novo subsistema soberano de Model Context Protocol
    ├── mod.rs               // Ponto de entrada, exports e montagem de rotas MCP
    ├── session.rs           // Gerenciador de sessões SSE, channels MPSC, Keep-Alive (15s) e TTL (30min)
    ├── registry.rs          // Catálogo canônico das 52 ferramentas, Nível 0/1 e meta tool_search
    ├── dispatcher.rs        // Despachante JSON-RPC 2.0, normalização de aliases, sanitização ReFS e catch_unwind
    ├── handlers.rs          // Executores de ferramentas ativas e fallbacks estruturados de staging
    └── transport.rs         // Handlers Axum para Streamable HTTP (/mcp) e Legado SSE (/sse + /messages)
```

---

## 2. GERENCIAMENTO DE SESSÕES E TRANSPORTE DUAL-MOUNT

### 2.1 Estrutura de Sessão (`session.rs`)

```rust
pub struct McpSession {
    pub session_id: String,
    pub tx: tokio::sync::mpsc::Sender<axum::response::sse::Event>,
    pub created_at: std::time::Instant,
    pub last_active: std::sync::atomic::AtomicU64,
    pub client_info: parking_lot::RwLock<Option<serde_json::Value>>,
}

pub struct McpSessionManager {
    sessions: dashmap::DashMap<String, std::sync::Arc<McpSession>>,
}
```

- **Keep-Alive Ativo (15s):** Cada stream SSE despacha um comentário de ping (`: keepalive\n\n`) a cada 15 segundos para impedir que firewalls locais ou o subsistema Winsock2 encerrem conexões ociosas.
- **TTL de 30 minutos:** Tarefa periódica assíncrona drena da memória sessões órfãs cujo último frame foi recebido há mais de 1.800 segundos.

### 2.2 Endpoints Dual-Mount (`transport.rs`)

1. **Streamable HTTP (2025/2026):**
   - `POST /mcp`: Recebe payload JSON-RPC 2.0. Processa o comando imediatamente. Se o cliente solicitar SSE (`Accept: text/event-stream`), estabelece canal de stream escopado; caso contrário, responde com `application/json`.
   - `GET /mcp`: Estabelece conexão persistente SSE com header `Mcp-Session-Id`.
2. **Legado SSE (2024-11-05):**
   - `GET /sse`: Inicia stream SSE e emite como primeiro evento:
     ```
     event: endpoint
     data: /messages?sessionId=<uuid>
     ```
   - `POST /messages?sessionId=<uuid>`: Recebe mensagens do cliente e despacha respostas pelo stream SSE da respectiva sessão.

---

## 3. CATÁLOGO DE FERRAMENTAS E REVELAÇÃO PROGRESSIVA (`registry.rs`)

O catálogo expõe os contratos congelados em `MCP_TOOL_CONTRACTS.md`.

### 3.1 Nível 0 — Boot Compacto (Sempre Visível)
O `tools/list` sem modificadores retorna exclusivamente 7 ferramentas essenciais:
1. `tool_search` (Meta-ferramenta de descoberta dinâmica)
2. `outline` (alias: `souls_ast_outline`)
3. `symbol` (alias: `souls_ast_slice`)
4. `read` (Leitura estruturada TOON)
5. `edit` (Edição cirúrgica com trava de rollback)
6. `thinking` (alias: `core_think` — Cognição socrática em grafo DAG)
7. `route_subtask` (alias: `souls_route_subtask` — Despacho FinOps ParetoBandit)

### 3.2 Nível 1 — Descoberta Sob Demanda
Ao invocar `tool_search(namespace, query)` ou ao chamar `tools/list?mode=full`, o servidor revela as demais ferramentas especializadas pertencentes aos namespaces `code.*`, `context.*`, `memory.*`, `think.*`, `ops.*` e `sys.*`.

---

## 4. DESPACHANTE JSON-RPC 2.0 & GUARDRAILS DE SEGURANÇA (`dispatcher.rs`)

### 4.1 Métodos RPC Obrigatórios
- `initialize`: Responde com `protocolVersion: "2024-11-05"` (compatibilidade universal) ou `2025-03-26` (se solicitado pelo cliente), identificador do servidor (`name: "souls_mcp", version: "7.2.0"`) e capacidades (`tools: { listChanged: true }`).
- `notifications/initialized`: Confirmação sem resposta de erro.
- `ping`: Retorna objeto vazio `{}`.
- `tools/list`: Lista de ferramentas com esquemas JSON Schema de entrada.
- `tools/call`: Execução de ferramenta com normalização de nome e extração de parâmetros.

### 4.2 Normalização de Nomes e Aliases
```rust
pub fn normalize_tool_name(mut name: &str) -> &str {
    loop {
        if let Some(rest) = name.strip_prefix("souls_mcp.") {
            name = rest;
        } else if let Some(rest) = name.strip_prefix("souls_") {
            name = rest;
        } else if let Some(rest) = name.strip_prefix("ctx_") {
            name = rest;
        } else {
            break;
        }
    }
    match name {
        "core_think" | "sequential_thinking" | "sequentialthinking" => "thinking",
        "ast_outline" => "outline",
        "ast_slice" => "symbol",
        "myers_diff" => "delta_diff",
        other => other,
    }
}
```

### 4.3 Sanitização ReFS Estrita
Qualquer argumento contendo `path` ou referências de arquivo é validado:
- Se contiver `..` que escape a partição `Z:\souls_engine` ou apontar para `C:\`, `C:\Users` ou `%APPDATA%`, o despachante aborta imediatamente com:
  `McpDomainError::RefsPathLeakViolation` (Código JSON-RPC `-32008`).

---

## 5. INTEGRAÇÃO COM APPSTATE E EXECUÇÃO DE HANDLERS (`handlers.rs`)

1. **`route_subtask`:**
   - Consome `state.router` (`ParetoBanditRouter`).
   - Recebe `task_type`, `estimated_tokens`, `latency_sensitivity`.
   - Executa `router.route(&task)` e retorna a decisão determinística da fronteira de Pareto com barreira térmica $\Phi$.
2. **`sys_time`:**
   - Retorna timestamp ISO-8601 com microssegundos e Unix Epoch.
3. **`metrics`:**
   - Retorna telemetria da RTX 2060m via `state.runtime.watchdog()` e conexões ativas.
4. **`routes`:**
   - Lista os endpoints expostos pelo daemon Axum e status do transporte.
5. **Ferramentas de Transplante Futuro (Staging):**
   - Retornam envelope padronizado JSON-RPC:
     ```json
     {
       "content": [
         {
           "type": "text",
           "text": "[Staging] Tool 'handoff' engine scheduled for Roadmap Session 2 (L3 Brain Transplant)."
         }
       ],
       "isError": false,
       "staging": true
     }
     ```

---

## 6. TELEMETRIA NÃO-BLOQUEANTE NO FRANKENSQLITE

Após a conclusão de cada `tools/call`, um worker assíncrono disparado via `tokio::spawn` insere um registro na tabela `telemetry_events` do banco SQLite STRICT:
```sql
INSERT INTO telemetry_events (
    id, timestamp, tool_name, duration_ms, is_error, session_id
) VALUES (?, ?, ?, ?, ?, ?);
```
Se a inserção falhar por lock transitório, a falha é registrada via `tracing::warn` sem propagar erro para o cliente MCP.
