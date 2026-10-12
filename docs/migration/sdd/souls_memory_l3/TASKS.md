# TASKS.md — Plano Sequencial de Tarefas do Cérebro L3 (`souls_memory`) (v7 Canônica)

> **ESTATUS DO DOCUMENTO:** CANÔNICO E APROVADO (SDD)
> **SUB-SISTEMA:** Hipocampo L3 Soberano & Memória de Sessões de Subagentes
> **CRATE ALVO:** `crates/souls_memory`

---

## 1. QUADRO DE TAREFAS ATÔMICAS

- [X] **TASK-MEM-001: Extensão do DDL Canônico STRICT & Migrações em `sqlite.rs`**
  - **Origem:** `docs/storage/SCHEMAS_DDL_MANIFEST.md` e `_donor/souls_mc_core/src/cognition/state_thinking/thinking/ops.rs`.
  - **Destino:** `crates/souls_memory/src/sqlite.rs`.
  - **Ação:** Adicionar ao `CANONICAL_DDL_MANIFEST` as tabelas `sub_agent_sessions`, `handoff_payloads`, `sub_agent_diaries`, `socratic_sessions` e `socratic_thoughts`.
  - **DoD:** Pool SQLite inicializa e executa migrações sem erro no in-memory e em disco ReFS. Teste unitário confirmando rejeição `STRICT` nessas novas tabelas. [X] CONCLUÍDA

- [X] **TASK-MEM-002: Métodos de Persistência Assíncrona no `sqlite.rs`**
  - **Destino:** `crates/souls_memory/src/sqlite.rs`.
  - **Ação:** Implementar métodos CRUD assíncronos (`sqlx`):
    - `insert_sub_agent_session`, `get_sub_agent_session`, `update_sub_agent_status`, `list_sub_agents_by_parent`.
    - `insert_handoff_payload`, `get_handoff_payload`.
    - `insert_sub_agent_diary`, `list_diaries_for_agent`.
    - `upsert_socratic_session`, `insert_socratic_thought`, `list_thoughts_for_session`.
    - `execute_read_only_query` (validando AST SQL para barrar mutações).
  - **DoD:** Testes unitários cobrindo inserção, consulta e constraints relacionais com ON DELETE CASCADE. [X] CONCLUÍDA

- [X] **TASK-MEM-003: Módulo `session_manager.rs` (Governança de Subagentes e Handoff)**
  - **Origem:** `_donor/souls_mc_core/src/cognition/state_thinking/thinking/worker.rs`.
  - **Destino:** `crates/souls_memory/src/session_manager.rs`.
  - **Ação:** Criar struct `SubAgentManager` com cache em RAM via `DashMap` e persistência no SQLite. Implementar spawn de subagentes, registro de diários, criação e consumo de handoff payloads.
  - **DoD:** Testes unitários simulando ciclo de vida: SPAWNED -> RUNNING -> COMPLETED com persistência de diários e handoff. [X] CONCLUÍDA

- [X] **TASK-MEM-004: Módulo `thinking.rs` (Motor Socrático, Paradigmas e DAG)**
  - **Origem:** `_donor/souls_mc_core/src/cognition/state_thinking/thinking/engine.rs`.
  - **Destino:** `crates/souls_memory/src/thinking.rs`.
  - **Ação:** Implementar `ThinkingEngine`, `ThinkingParadigm` (LinearCoT, TreeToT, GraphGoT, CouncilMAD), `ThinkingContext`, bifurcação (`branch_from_thought`), retificação (`is_revision`), verificação de limites (overthinking threshold), `export_session` em Markdown/JSON e `merge_sessions`.
  - **DoD:** Testes unitários validando determinação de paradigma, avanço de passos, disparo de erro se overthinking sem autorização HITL, e fusão consistente de branches. [X] CONCLUÍDA

- [X] **TASK-MEM-005: Módulo `mpsc_bridge.rs` (Ponte Assíncrona Não-Bloqueante)**
  - **Origem:** `_donor/souls_mc_core/src/cognition/state_thinking/memory_graph/mpsc_bridge.rs`.
  - **Destino:** `crates/souls_memory/src/mpsc_bridge.rs`.
  - **Ação:** Implementar canal assíncrono Tokio `mpsc::channel` com worker em background para gravação não-bloqueante de diários, pensamentos e telemetria.
  - **DoD:** Teste de estresse com envio concorrente de centenas de operações sem travar workers Tokio e confirmando persistência correta no banco. [X] CONCLUÍDA

- [X] **TASK-MEM-006: Orquestração e Exposição no `SoulsMemoryStore` (`lib.rs`)**
  - **Destino:** `crates/souls_memory/src/lib.rs`.
  - **Ação:** Integrar `SubAgentManager`, `ThinkingEngine` (via `DashMap` de sessões ativas) e `MemoryMpscBridge` dentro do `SoulsMemoryStore`. Exportar tipos canônicos publicamente.
  - **DoD:** Compilação com `cargo check -p souls_memory`. [X] CONCLUÍDA

- [X] **TASK-MEM-007: Verificação Compulsória e Testes de Integração L3**
  - **Destino:** `crates/souls_memory/tests/` e testes unitários.
  - **Ação:** Executar `cargo check -p souls_memory` e `cargo test -p souls_memory`.
  - **DoD:** 0 erros de compilação, 0 warnings do compilador e 100% dos testes passando limpos. [X] CONCLUÍDA
