# TASKS.md — Plano Sequencial de Tarefas da Sessão 3: Conexão Viva com Hermes Agent (v7)

> **ESTATUS DO DOCUMENTO:** CANÔNICO E APROVADO (SDD)
> **SUB-SISTEMA:** Simbiose Hermes Agent Desktop & Ativação de Handlers MCP de Memória
> **CRATES DIRETAMENTE AFETADAS:** `crates/souls_server` (Single Crate Rule)

---

## 1. QUADRO DE TAREFAS ATÔMICAS

- [X] **TASK-LIVE-001: Unificação Física de Modelos via Junction Point NTFS**
  - **Ação:** Verificar o estado de `%LOCALAPPDATA%\hermes\models` e criar o Junction Point apontando para `C:\Users\rosas\llm_local_models`.
  - **DoD:** O diretório `%LOCALAPPDATA%\hermes\models` reflete imediatamente o inventário de 15 modelos GGUF com zero bytes duplicados. (CONCLUÍDO)

- [X] **TASK-LIVE-002: Injeção de MCP e Provider Local no `config.yaml` do Hermes**
  - **Ação:** Ler o `config.yaml` do Hermes e anexar o bloco `mcp_servers.souls_engine` (`http://127.0.0.1:9123/mcp`) e o provider `souls_local`.
  - **DoD:** Arquivo `config.yaml` atualizado validamente sem quebrar configurações preexistentes. (CONCLUÍDO)

- [X] **TASK-LIVE-003: Injeção de Governança FinOps no `SOUL.md` do Hermes**
  - **Ação:** Inserir a diretriz mandatória de consulta prévia ao `route_subtask` antes de `delegate_task` no arquivo `SOUL.md`.
  - **DoD:** `SOUL.md` contém a diretriz formal de FinOps. (CONCLUÍDO)

- [X] **TASK-LIVE-004: Implementação dos Handlers MCP Reais de Memória no `souls_server`**
  - **Destino:** `crates/souls_server/src/mcp/handlers.rs`.
  - **Ação:** Implementar funções conectadas ao `state.memory` para `handoff`, `sub_agent`, `thinking`, `session`, `analyze_session`, `export_session`, `merge_sessions`, `knowledge`, `sqlite_query`, `semantic_search`, `mem_*`.
  - **DoD:** Handlers compilam sem erros e manipulam instâncias reais do `SoulsMemoryStore`. (CONCLUÍDO)

- [X] **TASK-LIVE-005: Roteamento no Despachante `crates/souls_server/src/mcp/dispatcher.rs`**
  - **Destino:** `crates/souls_server/src/mcp/dispatcher.rs`.
  - **Ação:** Mapear a invocação das ferramentas MCP para seus respectivos handlers reais em vez de `handle_staging_fallback`.
  - **DoD:** Chamadas JSON-RPC `tools/call` para ferramentas de memória executam código real. (CONCLUÍDO)

- [X] **TASK-LIVE-006: Testes Automatizados de Handlers de Memória em `souls_server`**
  - **Destino:** `crates/souls_server/src/main.rs`.
  - **Ação:** Adicionar testes de integração cobrindo `tools/call` para `knowledge`, `sqlite_query`, `sub_agent`, `thinking` e `handoff`.
  - **DoD:** Testes executam e passam 100% limpos. (CONCLUÍDO - 18/18 testes passando)

- [X] **TASK-LIVE-007: Verificação Compulsória e Validação da Sessão**
  - **Ação:** Executar `cargo check -p souls_server` e `cargo test -p souls_server`.
  - **DoD:** 0 erros de compilação, 0 warnings do compilador e 100% dos testes aprovados. (CONCLUÍDO)
