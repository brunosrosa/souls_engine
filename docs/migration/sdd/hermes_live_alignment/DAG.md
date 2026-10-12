# DAG.md — Ordem Topológica de Execução da Sessão 3: Conexão Viva com Hermes Agent

> **SESSÃO:** Sessão 3 (Bloco C: Conexão Viva & Alinhamento com Hermes Agent Desktop)
> **CRATE ALVO:** `crates/souls_server` (Single Crate Rule)

## Sequência Topológica Canônica:

```
[TASK-LIVE-001: Junction Point NTFS] ──┐
[TASK-LIVE-002: Hermes config.yaml]   ──┼────────────────┐
[TASK-LIVE-003: Hermes SOUL.md FinOps] ──┤                │
                                         ▼                ▼
[TASK-LIVE-004: MCP Handlers Reais] ──► [TASK-LIVE-005] ──► [TASK-LIVE-006] ──► [TASK-LIVE-007: Verificação]
                                         Dispatcher          Testes Integração     cargo test
```

## Paralelismos Seguros:
- As tarefas de configuração do host Windows (Junction Point, `config.yaml`, `SOUL.md`) podem ser executadas imediatamente.
- As tarefas de código em Rust (`handlers.rs`, `dispatcher.rs`, `main.rs`) são contidas estritamente dentro de `crates/souls_server`.
