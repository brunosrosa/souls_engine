# PLAN.md — Plano da Sessão 3: Conexão Viva & Alinhamento com Hermes Agent Desktop (v7)

> **ESTATUS DO DOCUMENTO:** CANÔNICO E APROVADO (SDD)
> **SUB-SISTEMA:** Simbiose Hermes Agent Desktop & Ativação de Handlers MCP de Memória
> **CRATES DIRETAMENTE AFETADAS:** `crates/souls_server` (Single Crate Rule)
> **SISTEMA OPERACIONAL:** Windows 11 MSVC / NTFS Dev Drive ReFS (`Z:\`)

---

## 1. RACIONAL ARQUITETURAL E OBJETIVO

A **Sessão 3** concretiza a simbiose entre o **Hermes Agent Desktop** (Python 3.11 / Electron) e o **Souls Engine v7** (Rust/Tokio Daemon headless na porta `:9123`).

Na Sessão 1, levantamos o daemon `souls_server` com transporte dual-mount HTTP/SSE e catálogo de 51 ferramentas.
Na Sessão 2, construímos o Cérebro L3 em `souls_memory` (FrankenSQLite STRICT, `SubAgentManager`, `ThinkingEngine`, `MemoryMpscBridge`, `LadybugOntologyGraph`).

### Objetivos Canônicos da Sessão 3:
1. **Unificação Física de Modelos (Junction Point NTFS):**
   - Apontar `%LOCALAPPDATA%\hermes\models` $\to$ `C:\Users\rosas\llm_local_models` via Junction Point transparente, garantindo que o Hermes enxergue imediatamente os 15 modelos locais quantizados sem duplicar 32+ GB de armazenamento físico.
2. **Amarração MCP no `souls_server`:**
   - Conectar os handlers de memória no `crates/souls_server/src/mcp/handlers.rs` e `dispatcher.rs` diretamente ao `AppState.memory` ([`SoulsMemoryStore`](file:///Z:/souls_engine/crates/souls_memory/src/lib.rs)), ativando 19 ferramentas reais (`handoff`, `sub_agent`, `thinking`, `knowledge`, `sqlite_query`, `mem_*`, etc.).
3. **Configuração do Hermes Agent Desktop:**
   - Inserir o registro do servidor MCP no `%LOCALAPPDATA%\hermes\config.yaml` (`http://127.0.0.1:9123/mcp`).
   - Configurar o provider local `llama.cpp` apontando para a porta `:9123`.
   - Injetar o guardrail de FinOps no `%LOCALAPPDATA%\hermes\SOUL.md` (obrigando a consulta a `route_subtask` antes de `delegate_task`).
4. **Verificação de Aperto de Mão & Testes:**
   - Confirmar compilação limpa e suíte de testes de integração em `souls_server`.

---

## 2. GUARDRAILS OPERACIONAIS

- **Regra de Crate Única:** As alterações de código em Rust neste turno pertencem **exclusivamente a `crates/souls_server`**.
- **Preservação da Soberania do Hermes:** Não sobrescrever o arquivo `config.yaml` inteiro; apenas injetar cirurgicamente o bloco `mcp_servers.souls_engine` e provedores locais.
- **Zero Vibe Coding:** Toda alteração é precedida por contrato formal e testes unitários.
