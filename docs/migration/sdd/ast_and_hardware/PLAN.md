# PLAN.md — Plano Diretor da Sessão 4: AST Tree-Sitter & Hardware Watchdog (v7)

> **ESTATUS DO DOCUMENTO:** CANÔNICO & VINCULANTE (SDD)  
> **DATA DE CRIAÇÃO:** 2026-10-12  
> **CRATES DIRETAMENTE AFETADAS:** `crates/souls_ast`, `crates/souls_inference_runtime`, `crates/souls_server` (Single Crate Rule por turno)  
> **ALVO OPERACIONAL:** Ativação plena em produção das ferramentas MCP de AST, Code Surgery, Gitoxide Frecency e Epistemic Logit Probing.

---

## 1. OBJETIVO ESTRATÉGICO

A **Sessão 4** conecta o cérebro agêntico às duas maiores fontes de economia de recursos e proteção de máquina do Souls Engine v7:
1. **Source-Side Dehydration (FinOps Mandatório):** O parser de AST Tree-sitter em WebAssembly isolado e o algoritmo de Myers Diff permitem inspecionar assinaturas de código e aplicar patches cirúrgicos com economia comprovada de **70% a 85% de tokens de contexto**, eliminando o vício de ler arquivos inteiros (`read_file`).
2. **Gitoxide Frecency & Blast Radius:** Cálculo termodinâmico de calor de arquivos (`repo_heatmap`) via decaimento de Langevin e rastreio de dependências reversas (`repo_impact`) com zero I/O de banco de dados.
3. **Hardware Watchdog & Epistemic Probing:** Exposição da ferramenta MCP `intent` conectada ao sensor Tier 0/0.5 (`smollm2-135m`, `smollm2-360m` e `gliclass_multilang`), avaliando ambiguidade e calculando entropia de Shannon em <15ms na CPU (0 MB de VRAM).

---

## 2. ARQUITETURA EM 4 ETAPAS SEQUENCIAIS (SINGLE CRATE RULE)

Para respeitar a **Cláusula 2.1 do AGENTS.md (A Regra de Crate Única)** e evitar contaminação de contexto, a sessão opera em fases atômicas sequenciais:

```
┌────────────────────────────────────────────────────────────────────────┐
│                        ESTEIRA SEQUENCIAL SDD                          │
├────────────────────────────────────────────────────────────────────────┤
│ ETAPA 4A: crates/souls_ast                                             │
│ • Enriquecer API de alto nível para outline com omissão de corpos      │
│ • Implementar localizador de símbolos (AST slice)                      │
│ • Implementar callers/callees e grafo de dependência reversa (impact)  │
│ • Testes unitários limpos (cargo test -p souls_ast)                    │
├────────────────────────────────────────────────────────────────────────┤
│ ETAPA 4B: crates/souls_inference_runtime                               │
│ • Consolidar API de intent probing e cálculo unificado de entropia     │
│ • Testes unitários limpos (cargo test -p souls_inference_runtime)      │
├────────────────────────────────────────────────────────────────────────┤
│ ETAPA 4C: crates/souls_server                                          │
│ • Substituir os stubs de staging por chamadas reais aos novos motores  │
│ • Implementar handlers para: outline, symbol, callers, callees, edit,  │
│   replace, delta_diff, repo_heatmap, repo_impact, intent               │
│ • Testes de integração MCP (cargo test -p souls_server)                │
├────────────────────────────────────────────────────────────────────────┤
│ ETAPA 4D: Validação e Governança                                       │
│ • Verificação compulsória de workspace (cargo check --workspace)       │
│ • Atualização de TASKS.md e Roadmap                                    │
└────────────────────────────────────────────────────────────────────────┘
```

---

## 3. DEFINITION OF DONE (CRITÉRIOS DE ACEITAÇÃO)

1. `cargo check -p souls_ast` e `cargo test -p souls_ast` executam com 0 erros e 0 warnings.
2. `cargo check -p souls_inference_runtime` e `cargo test -p souls_inference_runtime` executam com 0 erros e 0 warnings.
3. Chamadas MCP `tools/call` para `outline`, `symbol`, `delta_diff`, `repo_heatmap`, `repo_impact` e `intent` respondem com dados reais de silício e AST, sem o marcador `"staging": true`.
4. Os 18+ testes de integração de `souls_server` passam limpos, sem quebras nas ferramentas de memória preexistentes.
