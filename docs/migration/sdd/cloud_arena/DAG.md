# DAG.md — Grafo de Dependências de Execução: Sessão 5 (v7)

> **ESTATUS DO DOCUMENTO:** CANÔNICO & VINCULANTE (SDD)  
> **DATA DE CRIAÇÃO:** 2026-10-12  

---

## 1. SEQUÊNCIA DE BLOQUEIOS E DEPENDÊNCIAS

```mermaid
flowchart TD
    T1[Fase 1: Setup da Crate souls_llm_cloud_arena] --> T2[Fase 2: Storage SQLite STRICT WAL]
    T1 --> T3[Fase 3: Resolver Canônico de Entidades]
    T2 --> T4[Fase 4: Clientes HTTP Nous, OpenRouter & Bench]
    T3 --> T4
    T4 --> T5[Fase 5: Analytics 5 Perfis & Fronteira de Pareto]
    T5 --> T6[Fase 6: Sincronizador Diário, CLI & Lib Facade]
    T6 --> T7[Fase 7: Integração souls_server REST & MCP]
    T7 --> T8[Fase 8: Verificação Final do Workspace]
```

## 2. PONTOS DE CONTROLE CRÍTICO (GATEKEEPING)

1. **Gate 1 (Após Fase 1 e 2):** As tabelas `canonical_models`, `model_aliases`, `pricing_telemetry_snapshots`, `benchmark_snapshots` e `daily_evaluations` devem criar limpas com modo `STRICT` no SQLite.
2. **Gate 2 (Após Fase 5):** O cálculo dos 5 perfis cognitivos e algoritmo de Pareto deve passar nos testes matemáticos e isolamento de dominância.
3. **Gate 3 (Após Fase 6):** `cargo test -p souls_llm_cloud_arena` passa com 0 erros e 0 warnings antes de iniciar a Fase 7 (Single Crate Rule).
4. **Gate 4 (Após Fase 7):** `cargo check --workspace` e `cargo test --workspace` passam 100%.
