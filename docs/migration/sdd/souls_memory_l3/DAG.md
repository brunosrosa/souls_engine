# DAG.md — Ordem Topológica de Execução do Cérebro L3 (`souls_memory`)

> **SESSÃO:** Sessão 2 (Bloco D: Cérebro L3)
> **CRATE:** `crates/souls_memory`

## Sequência Topológica Canônica:

```
[TASK-MEM-001: Extensão DDL STRICT SQLite]
                   │
                   ▼
[TASK-MEM-002: CRUD Assíncrono sqlx no sqlite.rs]
                   │
    ┌──────────────┼──────────────┐
    ▼              ▼              ▼
[TASK-MEM-003] [TASK-MEM-004] [TASK-MEM-005]
session_mgr     thinking.rs    mpsc_bridge.rs
    │              │              │
    └──────────────┼──────────────┘
                   ▼
[TASK-MEM-006: SoulsMemoryStore lib.rs]
                   │
                   ▼
[TASK-MEM-007: Verificação Compulsória & Testes TDD]
```

## Racional de Concorrência:
- `TASK-MEM-001` e `TASK-MEM-002` estabelecem a base de persistência relacional relacional STRICT.
- `TASK-MEM-003` (subagentes e handoff), `TASK-MEM-004` (cognição socrática) e `TASK-MEM-005` (ponte assíncrona) consomem o pool do SQLite mas são ortogonais entre si, podendo ser implementados em sequência direta.
- `TASK-MEM-006` unifica tudo no agregador de alto nível `SoulsMemoryStore`.
- `TASK-MEM-007` valida a compilação e a suíte completa de testes.
