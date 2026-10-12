# TASKS.md — Relação Sequencial de Micro-Tarefas de Implementação (`souls_server_mcp`)

> **SUBSISTEMA ALVO:** `crates/souls_server`  
> **ESTATUS DO DOCUMENTO:** CHECKLIST DE EXECUÇÃO TDD (DOCS-AS-GUARDRAILS)  
> **REGRA DE ESCOPO:** Estritamente arquivos em `crates/souls_server/` (Regra de Crate Única)

---

## 1. QUADRO DE TAREFAS

| **ID** | **Título da Tarefa** | **Arquivo(s) Alvo** | **Status** | **Critério de Aceite (DoD)** |
| :--- | :--- | :--- | :---: | :--- |
| **TASK-001** | Gerenciador de Sessões MCP & Keep-Alive | `crates/souls_server/src/mcp/session.rs` | `[X] CONCLUÍDA` | Criação de sessões, despacho via channels MPSC, ping a cada 15s e expurgo de TTL após 30m. |
| **TASK-002** | Catálogo Canônico de 51 Ferramentas | `crates/souls_server/src/mcp/registry.rs` | `[X] CONCLUÍDA` | Schemas JSON-RPC de todas as 51 ferramentas, Nível 0 (7 ferramentas) e meta-ferramenta `tool_search`. |
| **TASK-003** | Handlers de Ferramentas Ativas & Staging | `crates/souls_server/src/mcp/handlers.rs` | `[X] CONCLUÍDA` | Execução real de `route_subtask`, `sys_time`, `metrics`, `routes`, e resposta de staging para os demais. |
| **TASK-004** | Despachante JSON-RPC 2.0 & Validador ReFS | `crates/souls_server/src/mcp/dispatcher.rs` | `[X] CONCLUÍDA` | Normalização de aliases, validação de containment no Dev Drive `Z:\` e tratamento de erros de domínio (-32000 a -32099). |
| **TASK-005** | Camada de Transporte Axum Dual-Mount | `crates/souls_server/src/mcp/transport.rs` | `[X] CONCLUÍDA` | Handlers para `POST/GET /mcp` (Streamable HTTP) e `GET /sse` + `POST /messages` (Legado SSE 2024-11-05). |
| **TASK-006** | Vinculação no AppState e Roteamento Axum | `crates/souls_server/src/state.rs`<br>`crates/souls_server/src/routes/mod.rs` | `[X] CONCLUÍDA` | Registro de `McpSessionManager` no `AppState`, montagem das rotas `/mcp`, `/sse`, `/messages` no router principal. |
| **TASK-007** | Bateria de Testes TDD & Verificação Final | `crates/souls_server/src/main.rs`<br>`crates/souls_server/tests/` | `[X] CONCLUÍDA` | 15 testes unitários e de integração passando limpos em 0.27s com zero warnings. |

---

## 2. DETALHAMENTO DE CADA TAREFA

### TASK-001: Gerenciador de Sessões MCP & Keep-Alive
- Implementar `McpSession` contendo `session_id`, `mpsc::Sender<axum::response::sse::Event>`, timestamps de criação e última atividade atômica.
- Implementar `McpSessionManager` com `DashMap` para lookup $O(1)$ sem lock global.
- Suportar método `send_to_session(session_id, event)` e broadcast de ping.

### TASK-002: Catálogo Canônico de 52 Ferramentas & `tool_search`
- Registrar os metadados canônicos de todas as ferramentas catalogadas em `MCP_TOOL_CONTRACTS.md`.
- Classificar em Nível 0 (Boot Compacto: 7 ferramentas) e Nível 1.
- Implementar algoritmo de busca da meta-ferramenta `tool_search(namespace, query)`.

### TASK-003: Handlers de Ferramentas Ativas & Staging
- Conectar `route_subtask` ao `state.router` (`ParetoBanditRouter`).
- Implementar `sys_time` retornando ISO-8601 e microssegundos.
- Implementar `metrics` inspecionando o Watchdog NVML da RTX 2060m.
- Implementar `routes` inspecionando o mapa de endpoints.
- Para ferramentas pendentes de transplante das Sessões 2 e 4, implementar retorno JSON-RPC estruturado sem quebra de contrato.

### TASK-004: Despachante JSON-RPC 2.0 & Validador ReFS
- Parsear envelopes `JsonRpcRequest` de `souls_protocol`.
- Normalizar aliases de ferramentas (`souls_mcp.<tool>`, `souls_<tool>`, etc.).
- Validar caminhos de arquivos: barrar qualquer escape fora de `Z:\souls_engine` gerando `RefsPathLeakViolation`.
- Tratar métodos fundamentais: `initialize`, `notifications/initialized`, `ping`, `tools/list`, `tools/call`.
- Proteger execuções com `catch_unwind` assíncrono para conter eventuais pânicos.

### TASK-005: Camada de Transporte Axum Dual-Mount
- `POST /mcp` e `GET /mcp` atendendo à especificação moderna Streamable HTTP.
- `GET /sse` atendendo à especificação legada, gerando `sessionId` e emitindo evento inicial `endpoint: /messages?sessionId=...`.
- `POST /messages` recebendo mensagens e roteando via despachante RPC.

### TASK-006: Vinculação no AppState e Roteamento Axum
- Adicionar `session_manager: Arc<McpSessionManager>` ao `AppState`.
- Montar as rotas no `routes::build_router(state)`.
- Disparar telemetria não-bloqueante no SQLite via `tokio::spawn` para cada `tools/call`.

### TASK-007: Bateria de Testes TDD & Verificação Final
- Testar handshake `initialize` em `/mcp` e `/sse`.
- Testar `tools/list` retornando Nível 0 por padrão e Nível 1 via `?mode=full`.
- Testar execução de `route_subtask` via `tools/call`.
- Testar rejeição de path leak fora do Dev Drive ReFS `Z:\`.
- Executar `cargo check -p souls_server` e `cargo test -p souls_server` garantindo 100% de aprovação.
