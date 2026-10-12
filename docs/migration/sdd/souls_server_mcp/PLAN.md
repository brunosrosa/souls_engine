# PLAN.md — Plano de Engenharia & Levantamento do Servidor MCP (`souls_server`)

> **SUBSISTEMA ALVO:** `crates/souls_server` (O Daemon Bare-Metal Headless & Servidor MCP)  
> **ESTATUS DO DOCUMENTO:** ESPECIFICAÇÃO SDD APROVADA (DOCS-AS-GUARDRAILS)  
> **DATA DE RATIFICAÇÃO:** 2026-10-11  
> **VETOR DE TRANSPORTE:** Dual-Mount Loopback TCP `127.0.0.1:9123` (`/mcp` Streamable HTTP + `/sse` & `/messages` Legado)  
> **ALINHAMENTO COM ROADMAP:** Sessão 1 (Bloco B: Daemon Base) de `SOULS_ENGINE_ROADMAP_NEXT_DELIVERIES.md`

---

## 1. OBJETIVO TÉCNICO E ESCOPO DA SESSÃO

Transformar a crate executável `crates/souls_server` no servidor de protocolo soberano do **Souls Engine v7**, adicionando o barramento MCP (Model Context Protocol) sem regredir as rotas já existentes (`/health`, `/v1/models`, `/v1/chat/completions`) e sem violar as 6 Linhas Vermelhas da Arquitetura Canônica.

Ao concluir este plano, o `souls_server.exe` estará apto a receber conexões simultâneas do **Hermes Agent Desktop (Python 3.11)** e de IDEs modernas (Cursor, Windsurf, Trae), servindo o handshake de protocolo, o catálogo de 52 ferramentas com Revelação Progressiva (Progressive Tool Disclosure) e a execução segura de RPCs.

---

## 2. DIAGNÓSTICO FORENSE DO MATERIAL LEGADO (`_donor/`)

A investigação detalhada sobre o acervo legado em `_donor/souls_mc_core/src/bin/souls_mcp_server/` revelou:

1. **`router.rs` e Normalização:**  
   O despachante original possuía suporte a múltiplos aliases (`souls_mcp.<tool>`, `souls_<tool>`, `ctx_<tool>`), essencial para interoperabilidade Zero-Brand (ADR-026).
2. **`tools.rs` e Inchaço de Prompt:**  
   O catálogo legado listava dezenas de ferramentas monoliticamente, gerando de 12.000 a 20.000 tokens de sobrecarga no prompt do agente. A v7 exige compulsoriamente a Revelação Progressiva (Nível 0 compacto com 7 ferramentas essenciais + `tool_search`).
3. **`handlers/` e Fragmentação:**  
   As 50+ ferramentas do donor estavam distribuídas em `system.rs`, `context.rs`, `observability.rs`, `thinking.rs` e `memory_graph.rs`. Muitas dessas ferramentas dependem de órgãos das Sessões 2 e 4 (LanceDB, LadybugDB, Tree-sitter WASM, etc.). Na Sessão 1, adota-se o **Fallback Gracioso Estruturado** para garantir conformidade de envelope sem quebras.
4. **Acoplamentos Necróticos Eliminados:**  
   Remoção total de referências ao proxy `:3001`, janelas Tauri, bindings Wry e escrita fora do Dev Drive ReFS `Z:\`.

---

## 3. AUDITORIA DAS ESPECIFICAÇÕES DE MERCADO (MCP 2025/2026 vs 2024)

Pesquisas recentes sobre o padrão oficial da Anthropic / MCP Working Group revelam uma evolução crítica no transporte de rede:

| Dimensão | Especificação Original (2024-11-05) | Especificação Canônica Moderna (2025-03-26+) |
| :--- | :--- | :--- |
| **Padrão de Transporte** | HTTP + SSE (Dois endpoints separados) | **Streamable HTTP** (Endpoint único consolidado) |
| **Status da Especificação** | Formalmente **Deprecada** (Março/2025) | **Vigente & Recomendada** para integrações de rede |
| **Topologia de Endpoints** | `GET /sse` (Stream) e `POST /messages` (Comandos) | `POST /mcp` (com upgrade opcional a SSE via streaming) |
| **Cancelamento** | Envio de RPC `notifications/cancelled` | Fechamento do fluxo de resposta HTTP do cliente |
| **Gerenciamento de Sessão** | `sessionId` via query string ou evento inicial | Header `Mcp-Session-Id` ou corpo estruturado |

### Decisão Arquitetural: Dual-Mount Híbrido
O `souls_server` implementará uma arquitetura de transporte unificada capaz de atender ambas as especificações:
- `/mcp` (Streamable HTTP 2025/2026): Endpoint único com suporte a POST JSON-RPC e streaming SSE.
- `/sse` e `/messages` (Legado 2024-11-05): Para retrocompatibilidade absoluta com SDKs Python do Hermes Agent e ferramentas legadas.

---

## 4. FRONTEIRAS DE SISTEMA E REGRAS OPERACIONAIS

1. **Regra de Crate Única (Single Crate Rule):** Todas as alterações de código desta sessão ocorrem estritamente dentro de `crates/souls_server/`. Tipos de envelopes já consolidados em `souls_protocol` (`JsonRpcRequest`, `JsonRpcResponse`, `JsonRpcError`, `McpDomainError`) serão consumidos diretamente.
2. **Preservação de Rotas Existentes:** `/health`, `/v1/models` e `/v1/chat/completions` permanecem inalterados e 100% funcionais.
3. **Isolamento de Erros e Pânico:** Cada chamada de ferramenta é executada sob barreira assíncrona contra pânico (`catch_unwind`), mapeando exceções para códigos de domínio JSON-RPC (-32000 a -32099).
4. **Sanitização de Caminhos ReFS:** É categoricamente proibido permitir caminhos que vazem da raiz `Z:\souls_engine` (ADR-005). Tentativas de escape disparam `RefsPathLeakViolation` (-32008).
