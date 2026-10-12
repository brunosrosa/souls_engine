# TASKS.md — Plano de Tarefas Detalhado: Cloud Arena (v7)

> **ESTATUS DO DOCUMENTO:** CANÔNICO & VINCULANTE (SDD)  
> **DATA DE CRIAÇÃO:** 2026-10-12  
> **CRATES:** `crates/souls_llm_cloud_arena`, `crates/souls_server`  

---

## TAREFAS DE IMPLEMENTAÇÃO SEQUENCIAL

### FASE 1: Setup da Crate e Contratos Básicos (`souls_llm_cloud_arena`)
- [x] **1.1** Criar `crates/souls_llm_cloud_arena/Cargo.toml` com dependências aprovadas (`reqwest`, `sqlx`, `chrono`, etc.).
- [x] **1.2** Atualizar `Cargo.toml` raiz do workspace registrando `"crates/souls_llm_cloud_arena"`.
- [x] **1.3** Criar `crates/souls_llm_cloud_arena/src/error.rs` com tipagem formal via `thiserror`.
- [x] **1.4** Criar `crates/souls_llm_cloud_arena/src/domain.rs` com structs de domínio (snapshots, DTOs e enums).
- [x] **1.5** Criar `crates/souls_llm_cloud_arena/src/config.rs` lendo chaves de ambiente e arquivos locais com fallbacks.

### FASE 2: Armazenamento Relacional SQLite STRICT WAL (`src/storage/`)
- [x] **2.1** Implementar `src/storage/migrations.rs` com o DDL SQL completo das 5 tabelas em modo `STRICT`.
- [x] **2.2** Implementar `src/storage/repository.rs` para inserção atômica de snapshots e consultas de campeões.
- [x] **2.3** Testes unitários de persistência e migrations em banco de teste isolado.

### FASE 3: Resolução Canônica de Entidades (`src/canonical/`)
- [x] **3.1** Implementar `src/canonical/seed_aliases.rs` contendo os aliases padrão dos 30+ modelos principais.
- [x] **3.2** Implementar `src/canonical/resolver.rs` resolvendo slugs heterogêneos de diferentes provedores.
- [x] **3.3** Testes unitários do resolvedor canônico.

### FASE 4: Clientes HTTP e Ingestão de Provedores (`src/sources/`)
- [x] **4.1** Implementar `src/sources/openrouter.rs` consumindo `/api/v1/models` com suporte anônimo e autenticado.
- [x] **4.2** Implementar `src/sources/nous.rs` consumindo `/v1/models` da Nous Portal.
- [x] **4.3** Implementar `src/sources/benchmarks.rs` com seed embutido e conector para scores externos.
- [x] **4.4** Testes unitários dos clientes com payloads simulados (mocks/fixtures).

### FASE 5: Motor Analítico, 5 Perfis e Fronteira de Pareto (`src/analytics/`)
- [x] **5.1** Implementar `src/analytics/profiles.rs` definindo pesos dos 5 perfis (Coding, Agentic, Reasoning, Fast, Design).
- [x] **5.2** Implementar `src/analytics/scoring.rs` calculando $S_{\text{task}}$, custo $C_{\text{task}}$ e ROI $V_{\text{task}}$.
- [x] **5.3** Implementar `src/analytics/pareto.rs` com filtro de dominância não-superada e tags de liderança.
- [x] **5.4** Testes unitários de cálculo de notas e optimalidade de Pareto.

### FASE 6: Sincronizador Diário, CLI e Relatórios (`src/sync.rs`, `src/report/`)
- [x] **6.1** Implementar `src/sync.rs` com checagem `last_update < today_utc` e orquestração do pipeline.
- [x] **6.2** Implementar `src/report/markdown.rs` e `src/report/terminal.rs` para visualização tabular.
- [x] **6.3** Criar binário executável `src/bin/souls_cloud_arena.rs` (CLI com comandos `sync`, `champions`, `report`).
- [x] **6.4** Consolidar `src/lib.rs` com fachada unificada `CloudArenaCoordinator`.
- [x] **6.5** Validar `cargo test -p souls_llm_cloud_arena` com 100% de sucesso e 0 warnings.

### FASE 7: Integração no Daemon `souls_server`
- [x] **7.1** Adicionar `souls_llm_cloud_arena` ao `Cargo.toml` de `souls_server`.
- [x] **7.2** Acionar sincronização diária em background Tokio durante o boot do servidor Axum.
- [x] **7.3** Implementar endpoints REST `/api/v1/cloud/champions` e `/api/v1/cloud/sync`.
- [x] **7.4** Implementar ferramentas MCP `cloud_champions` e `cloud_arena_sync`.
- [x] **7.5** Validar `cargo test -p souls_server` garantindo que todos os testes passem.

### FASE 8: Verificação de Governança Compulsória
- [x] **8.1** Executar `cargo check --workspace` e `cargo test --workspace`.
- [x] **8.2** Atualizar documentação e consolidar status no Roadmap.
