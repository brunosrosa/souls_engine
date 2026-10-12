# TASKS.md — Plano Sequencial de Tarefas da Sessão 4: AST & Hardware Watchdog (v7)

> **ESTATUS DO DOCUMENTO:** CANÔNICO E APROVADO (SDD)  
> **SUB-SISTEMA:** Tree-Sitter AST Dehydration, Myers Diff, Gitoxide Heatmap e Epistemic Logit Probing  
> **GOVERNANÇA:** Single Crate Rule (uma crate modificada por turno)

---

## 1. QUADRO DE TAREFAS ATÔMICAS

- [X] **TASK-AST-001: Enriquecimento de APIs em `crates/souls_ast`**
  - **Destino:** `crates/souls_ast/src/lib.rs`, `treesitter.rs`, `heatmap.rs`.
  - **Ação:** Expor funções públicas de desidratação (`dehydrate_outline`), extração de símbolo (`extract_symbol_slice`) e cálculo de impacto (`calculate_blast_radius`).
  - **DoD:** `cargo test -p souls_ast` compila limpo com 100% dos testes aprovados. (CONCLUÍDO - 20/20 testes)

- [X] **TASK-AST-002: Consolidação da API de Probing em `crates/souls_inference_runtime`**
  - **Destino:** `crates/souls_inference_runtime/src/lib.rs`, `inference.rs`.
  - **Ação:** Expor método unificado `evaluate_intent_and_entropy` combinando classificação ONNX e sonda de logits do `LlamaLogitProber`.
  - **DoD:** `cargo test -p souls_inference_runtime` compila limpo com 100% dos testes aprovados. (CONCLUÍDO - 33/33 testes)

- [X] **TASK-AST-003: Implementação dos Handlers Reais de AST no `souls_server`**
  - **Destino:** `crates/souls_server/src/mcp/handlers.rs`.
  - **Ação:** Implementar `handle_outline`, `handle_symbol`, `handle_callers`, `handle_callees`, `handle_get_ast`, `handle_delta_diff`, `handle_edit`, `handle_replace`, `handle_repo_heatmap`, `handle_repo_impact`.
  - **DoD:** Handlers compilam sem erros e manipulam instâncias reais do `souls_ast`. (CONCLUÍDO)

- [X] **TASK-AST-004: Implementação do Handler de `intent` no `souls_server`**
  - **Destino:** `crates/souls_server/src/mcp/handlers.rs`.
  - **Ação:** Implementar `handle_intent` conectando ao `state.runtime` para calcular entropia de Shannon e classificação de intenção a zero VRAM.
  - **DoD:** Handler compila sem erros. (CONCLUÍDO)

- [X] **TASK-AST-005: Roteamento no Despachante `crates/souls_server/src/mcp/dispatcher.rs`**
  - **Destino:** `crates/souls_server/src/mcp/dispatcher.rs`.
  - **Ação:** Mapear os 11 novos nomes de ferramentas para seus respectivos handlers reais em vez de `handle_staging_fallback`.
  - **DoD:** Chamadas JSON-RPC executam código real para todas as 11 ferramentas. (CONCLUÍDO)

- [X] **TASK-AST-006: Testes Automatizados de Integração no `souls_server`**
  - **Destino:** `crates/souls_server/src/main.rs`.
  - **Ação:** Adicionar testes cobrindo `outline`, `symbol`, `delta_diff`, `repo_heatmap`, `repo_impact` e `intent`.
  - **DoD:** Novos testes passam e suite completa de `souls_server` atinge 20 testes aprovados. (CONCLUÍDO - 20/20 testes)

- [X] **TASK-AST-007: Verificação Compulsória e Validação do Workspace**
  - **Ação:** Executar `cargo check --workspace` e `cargo test --workspace`.
  - **DoD:** 0 erros, 0 warnings do compilador e 100% dos testes do workspace aprovados. (CONCLUÍDO)
