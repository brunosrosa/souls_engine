# PLAN.md — Plano Diretor da Sessão 5: Cloud Arena & Ingestão de APIs de Nuvem (v7)

> **ESTATUS DO DOCUMENTO:** CANÔNICO & VINCULANTE (SDD)  
> **DATA DE CRIAÇÃO:** 2026-10-12  
> **CRATES DIRETAMENTE AFETADAS:** `crates/souls_llm_cloud_arena`, `crates/souls_model_router`, `crates/souls_server` (Single Crate Rule por turno)  
> **ALVO OPERACIONAL:** Ingestão de mercado (Nous Portal e OpenRouter), 5 perfis cognitivos, cálculo da Fronteira de Pareto e sync diário autônomo no daemon.

---

## 1. OBJETIVO ESTRATÉGICO

A **Sessão 5** estabelece o braço de inteligência em nuvem do Souls Engine v7:
1. **Ingestão Automática de Mercado:** Consumir dados em tempo real de precificação detalhada ($P_{\text{cache}}$, $P_{\text{in}}$, $P_{\text{out}}$) e telemetria de latência/throughput do Nous Research Portal e do OpenRouter (com fallback anônimo público).
2. **Avaliação Cognitiva Multidimensional em 5 Perfis:**
   - **Coding & SWE ($S_{\text{coding}}$):** Geração de código e engenharia de software real.
   - **Autonomous Agentic ($S_{\text{agentic}}$):** Chamadas de ferramentas (BFCL) e conformidade de instruções (IFEval).
   - **Deep Reasoning ($S_{\text{reasoning}}$):** Raciocínio abstrato complexo e problemas formais (GPQA).
   - **Fast / Triage ($S_{\text{fast}}$):** Baixa latência, alto throughput e triagem heurística.
   - **Design & Multimodal UI ($S_{\text{design}}$):** Front-end, geração de interfaces SVG, layouts e visão computacional.
3. **Fronteira de Pareto e ROI Amortecido ($V_{\text{task}}$):** Cálculo de modelos dominantes e unificação analítica com a fórmula de silício local ($C_{\text{silicon}}$).
4. **Sincronização Diária Resiliente:** Disparo em background ao iniciar o `souls_server` (se `last_update < today`), sem travar o daemon e preservando o cache local em caso de falha de conexão.
5. **Integração de Consumo:** Exposição no `souls_server` via endpoint REST `/api/v1/cloud/champions` e ferramentas MCP `cloud_champions` e `cloud_arena_sync`.

---

## 2. ARQUITETURA EM 4 ETAPAS SEQUENCIAIS (SINGLE CRATE RULE)

Para respeitar a **Cláusula 2.1 do AGENTS.md (A Regra de Crate Única)**:

```
┌────────────────────────────────────────────────────────────────────────┐
│                        ESTEIRA SEQUENCIAL SDD                          │
├────────────────────────────────────────────────────────────────────────┤
│ ETAPA 5A: crates/souls_llm_cloud_arena                                 │
│ • Criação da crate e registro no Cargo workspace (#![forbid(unsafe)]) │
│ • Módulos domain, config, error e storage (SQLite STRICT WAL)          │
│ • Resolução canônica de modelos e aliases (resolver.rs)                │
│ • Clientes HTTP assíncronos (nous.rs, openrouter.rs, benchmarks.rs)    │
│ • Motor analítico dos 5 perfis, cálculo de custos e Pareto             │
│ • Orquestrador de sync diário (sync.rs) e relatórios                   │
│ • CLI standalone bin/souls_cloud_arena.rs                              │
│ • Testes unitários limpos (cargo test -p souls_llm_cloud_arena)        │
├────────────────────────────────────────────────────────────────────────┤
│ ETAPA 5B: crates/souls_model_router                                    │
│ • Integração das tabelas de nuvem e campeões no ParetoBandit           │
│ • Testes unitários limpos (cargo test -p souls_model_router)           │
├────────────────────────────────────────────────────────────────────────┤
│ ETAPA 5C: crates/souls_server                                          │
│ • Registro da tarefa background de boot para sincronização diária      │
│ • Endpoint REST /api/v1/cloud/champions e /api/v1/cloud/sync           │
│ • Ferramentas MCP cloud_champions e cloud_arena_sync                   │
│ • Testes de integração (cargo test -p souls_server)                    │
├────────────────────────────────────────────────────────────────────────┤
│ ETAPA 5D: Validação e Governança                                       │
│ • Verificação compulsória de workspace (cargo check --workspace)       │
│ • Atualização de TASKS.md e Roadmap                                    │
└────────────────────────────────────────────────────────────────────────┘
```

---

## 3. DEFINITION OF DONE (CRITÉRIOS DE ACEITAÇÃO)

1. `cargo check -p souls_llm_cloud_arena` e `cargo test -p souls_llm_cloud_arena` passam com 0 erros e 0 warnings.
2. Migrations SQLite STRICT aplicadas com sucesso em `Z:\souls_engine\.souls_data\db\souls_heuristic_vault.db`.
3. Sincronização diária testa com mock/fixtures e opera offline com elegância.
4. Endpoint REST `/api/v1/cloud/champions` e ferramenta MCP `cloud_champions` retornam os líderes dos 5 perfis.
5. `cargo check --workspace` e `cargo test --workspace` executam limpos.
