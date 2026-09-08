# TASKS.md — Plano de Execução do Transplante (Gemini 3.8 Flash)

## Diretrizes de Execução
Para cada tarefa numerada abaixo, o engenheiro executor deve operar no ciclo **Red-Green-Refactor**:
1. Escrever o teste unitário (`cargo test -p <crate> -- test_nome`).
2. Transplantar o órgão aplicando higienização CRLF -> LF.
3. Obter verificação sem erros: `cargo check -p <crate>`.

---

## ONDA 1: Estruturas Universais (souls_protocol)

### TASK-PRT-001: Mapear Tipos de Telemetria e FinOps
- **Destino:** `crates/souls_protocol/src/dto.rs`
- **Origem:** `_donor/souls_mc_core/src/finops/iron_cost.rs`
- **Ação:** Refatorar enums e structs relacionados a custos (`e3_score`) extraindo a matemática para DTOs puros.
- **DoD:** `cargo check -p souls_protocol` sem avisos.

---

## ONDA 2: Infraestrutura Base (souls_core)

### TASK-COR-001: Filtros de ANSI e Telemetria
- **Destino:** `crates/souls_core/src/logging.rs` e `fs.rs`
- **Origem:** `_donor/souls_mc_core/src/cognition/context/ansi_filter.rs`
- **Ação:** Integrar purificação de caracteres escape ansi nas proteções de logs assíncronos.
- **DoD:** Teste falhando (RED) para entrada com caracteres ANSI e passando limpo (GREEN) pós-transplante.

### TASK-COR-002: Motor de Leitura LEAN (Smart Read)
- **Destino:** `crates/souls_core/src/fs.rs` e `context_lean.rs`
- **Origem:** `_donor/souls_mc_core/src/cognition/context/souls_smart_read.rs` e `multi_read.rs`
- **Ação:** Incorporar medição nativa com `tiktoken` e shrink iterativo no `spawn_blocking`.
- **DoD:** Testes unitários com simulação de blocos densos retornando limites esperados.

### TASK-COR-003: Integração de ReFS Block Cloning
- **Destino:** `crates/souls_core/src/fs.rs` e `crates/souls_memory/src/chyros.rs`
- **Origem:** Nova funcionalidade obrigatória (ADR-005).
- **Ação:** Implementar Block Cloning via chamadas Win32 `DeviceIoControl` e `FSCTL_DUPLICATE_EXTENTS_TO_FILE` usando a crate `windows-sys`. Aplicar na rotação do `VACUUM INTO` do SQLite.
- **DoD:** Testes unitários comprovando que a cópia de um arquivo grande consome tempo próximo a zero (Zero-Copy) sem falhar em integridade.

---

## ONDA 3: Parsing e Diferenças Sintáticas (souls_ast)

### TASK-AST-001: Integração de Diferencial (Myers Diff)
- **Destino:** `crates/souls_ast/src/myers.rs`
- **Origem:** `_donor/souls_mc_core/src/cognition/context/myers_diff.rs`
- **Ação:** Limpar qualquer importação externa não canônica, adaptando ao Myers Diff da crate `similar`. Forçar retornos a `AstError`.
- **DoD:** `cargo check -p souls_ast` e teste `test_myers_safe_closure`.

### TASK-AST-002: Motor de Heatmap (Frecency gitoxide)
- **Destino:** `crates/souls_ast/src/heatmap.rs`
- **Origem:** `_donor/souls_mc_core/src/cognition/context/repo_heatmap.rs`
- **Ação:** Acoplar o extrator bare-metal da crate `gix`.
- **DoD:** Teste sintético criando histórico local via `gix` em diretório temporário isolado.

### TASK-AST-003: Sandbox Wasmtime para Parsers Tree-sitter
- **Destino:** `crates/souls_ast/src/treesitter.rs`
- **Origem:** `_donor/souls_mc_core/src/harvester/ast_parser.rs` e gramáticas WASM (`_donor/resources/wasm_grammars/`).
- **Ação:** Setup de segurança enjaulado via `Wasmtime::Store`, com limite de combustível e `memmap2` (WASI 0.2/0.3). Descarte dos stubs C nativos instáveis e graceful shutdown via `Trap::StructuredFailure`.
- **DoD:** Teste injetando uma gramática WASM mal-comportada (ou arquivo corrompido) atestando contenção de crash sem derrubar o processo de testes.

---

## ONDA 4: Memória Triad (souls_memory)

### TASK-MEM-001: Fusão RRF e Decaimento Langevin
- **Destino:** `crates/souls_memory/src/rrf.rs` e `chyros.rs`
- **Origem:** `_donor/souls_mc_core/src/cognition/memory/rrf_fusion.rs` e `langevin_decay.rs`
- **Ação:** Embutir matemática pura do decaimento ao ciclo assíncrono do `chyros.rs`.
- **DoD:** Coeficiente calculando dissipação estocástica testada com valores mock de $\Delta t$.

### TASK-MEM-002: Consolidar Grafo Ladybug
- **Destino:** `crates/souls_memory/src/ladybug.rs`
- **Origem:** `_donor/souls_mc_core/src/cognition/state_thinking/memory_graph/ladybug_firewall.rs` e `ops.rs`
- **Ação:** Integrar o grafo `petgraph` com a tabela `ladybug_nodes` no SQLite STRICT WAL.
- **DoD:** Inserção relacional e de arestas confirmadas limpas via pool `sqlx`.

---

## ONDA 5: Watchdogs da RTX 2060m e Inferência (souls_inference_runtime)

### TASK-INF-001: NVML Hardware Watchdog
- **Destino:** `crates/souls_inference_runtime/src/nvml.rs`
- **Origem:** `_donor/souls_mc_core/src/core/vram_hardware/hardware_watchdog.rs` e `peak_ewma.rs`
- **Ação:** Instanciar monitor a 500ms via `nvml-wrapper` e EWMA.
- **DoD:** Teste com mocking do WDDM abortando sob pressão (spillover limit 5.2GB).

### TASK-INF-002: Integração de Tier 0 (ONNX / Llama.cpp)
- **Destino:** `crates/souls_inference_runtime/src/onnx.rs` e `llamacpp.rs`
- **Origem:** `_donor/souls_mc_core/src/core/inference/gliclass_engine.rs` e `llama_logit_probing.rs`
- **Ação:** Acoplar a carga de pesos via ORT/llama.cpp para sondagem de logit zero-copy.
- **DoD:** Carregamento simulado de pesos leves atestando funcionamento limpo de memória.

---

## ONDA 6: Anthropophagy (souls_anthropophagy)

### TASK-ANT-001: Fusão e Transplante do Harvester
- **Destino:** `crates/souls_anthropophagy/src/`
- **Origem:** `_donor/souls_mc_anthropophagy/src/harvester/` e `_donor/souls_mc_core/src/harvester/`
- **Ação:** Realizar a unificação (merge) dos dois diretórios. Incorporar o integrador SAST e sandbox estendida de `anthropophagy` em conjunto com otimizações de I/O, concorrência e tratamentos de caminhos locais exclusivos de `mc_core`.
- **DoD:** Testes completos de verificação SPDX e varredura de projeto passando de ponta a ponta.

---

## ONDA 7: Descarte Final (Clean-up)

### TASK-CLN-001: Expurgo Físico de Necrose
- **Alvo:** O diretório clone legatório temporário na máquina de montagem.
- **Ação:** Descarte irrecuperável de referências Tauri v2, Wry e gateways :3001.
- **DoD:** Grep total do repositório canônico não encontrando qualquer resíduo destas dependências.
