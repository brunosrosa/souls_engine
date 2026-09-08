# PLAN.md — Inventário Forense e Triagem de Componentes

## 1. Objetivo da Autópsia

Este documento detalha o estado do código presente em `_donor/` e estabelece a classificação clínica de seus módulos para o transplante sistemático para o ecossistema do **Souls Engine v7**, respeitando a matriz topológica de 9 crates canônicas e erradicando os componentes necróticos do legado desktop.

## 2. Diagnóstico e Resolução de Anomalias

### 2.1 Duplicação do Subsistema `harvester`
**Análise:** Foi identificada a presença simultânea do subsistema `harvester` em dois locais:
- `_donor/souls_mc_anthropophagy/src/harvester/`
- `_donor/souls_mc_core/src/harvester/`

**Diagnóstico:** Ao invés de suprimir cegamente um dos diretórios, a auditoria revelou que ambos contêm implementações valiosas e arquivos base semelhantes (ex: `repo_radar.rs` e `ast_parser.rs`). O diretório `souls_mc_core` pode abrigar otimizações de concorrência ou filtros que faltam na versão SAST expandida.
**Resolução:** Será realizada uma **fusão estruturada e unificação (merge)** de ambos os diretórios. As joias funcionais de concorrência e tratamento de erros do `mc_core` serão absorvidas no chassi da `souls_anthropophagy` antes do transplante final.

### 2.2 Motores de Gramática (WASM vs Nativo)
**Análise:** O legado em `_donor/souls_mc_core/src/harvester/ast_parser.rs` carrega blobs de WASM via caminhos como `src-tauri/resources/wasm_grammars/tree_sitter_rust.wasm`.
**Diagnóstico:** É inegociável a manutenção da sandbox WASM. Substituir por FFI nativo direto exporia o processo daemon a crashes fatais (segfaults em gramáticas C mal-comportadas).
**Resolução:** A arquitetura do `souls_ast` preservará estritamente a execução enjaulada em `Wasmtime` (WASI 0.2/0.3). O parser rodará com limites rígidos de combustível (fuel metering) e RAM linear de 16MB. A performance será garantida por `ts_set_allocator` (bump-arena) e paginação lazy via `memmap2`, com graceful shutdown de `Trap::StructuredFailure`.

### 2.3 Fragmentação da Memória
**Análise:** O hipocampo encontrava-se cindido entre `cognition/memory/` e `cognition/state_thinking/memory_graph/`.
**Resolução:** Unificação sistêmica dentro da crate `souls_memory`, ancorando a autoridade de I/O em um único pool relacional (FrankenSQLite) e colunar (LanceDB).

## 3. Inventário Forense e Classificação Clínica

### Cluster 1: Hardware & Termodinâmica
- `vram_hardware/hardware_watchdog.rs` -> `[ORGAN_REPAIR]` (Adaptação para `nvml-wrapper` e `souls_inference_runtime`)
- `vram_hardware/peak_ewma.rs` -> `[HEALTHY_ORGAN]`
- `vram_hardware/headroom_engine.rs` -> `[UPGRADE_CANDIDATE]` (Restaurar predição)

### Cluster 2: Hipocampo & Memória Triad L3
- `memory/langevin_decay.rs` -> `[HEALTHY_ORGAN]` (Matemática pura estocástica)
- `memory/rrf_fusion.rs` -> `[HEALTHY_ORGAN]` (Matemática de indexação RRF pura)
- `memory/chyros_daemon.rs` -> `[ORGAN_REPAIR]` (Refatorar SQL síncrono para Tokio em `souls_memory`)
- `memory_graph/ops.rs`, `fts.rs`, `vector_store.rs` -> `[ORGAN_REPAIR]` (Consolidar SQLite STRICT WAL e LanceDB 0MB VRAM)

### Cluster 3: Motores de Inferência & Probing
- `inference/ort_scorer.rs`, `gliclass_engine.rs` -> `[ORGAN_REPAIR]` (Refatorar para Tier 0 ONNX na `souls_inference_runtime`)
- `inference/llama_logit_probing.rs` -> `[ORGAN_REPAIR]` (Refatorar para Tier 0.5 CPU via llama.cpp)
- `inference/bitnet_daemon.rs` -> `[UPGRADE_CANDIDATE]` (Potencial para a `souls_llm_local_arena`)

### Cluster 4: Contexto, AST e Compressão LEAN
- `context/myers_diff.rs`, `ansi_filter.rs`, `dedup.rs`, `souls_smart_read.rs`, `multi_read.rs` -> `[HEALTHY_ORGAN]` (Transplante direto para `souls_ast` e `souls_core`, higienizando `\r\n`)
- `ast/repo_heatmap.rs`, `ast/repo_impact.rs` -> `[ORGAN_REPAIR]` (Garantir conversão para `gitoxide` em vez de shell `git.exe` dentro de `souls_ast`)

### Cluster 5: Toxicidade e Necrose (Expurgo Definitivo)
- `core/socratic/socratic_thought_stream.rs`, `terminal_drawer_stream.rs` -> `[NECROSIS_INCINERATE]` (Dependências de Tauri IPC)
- Qualquer referência a `:3001`, `winit`, `wry` e proxies L7 -> `[NECROSIS_INCINERATE]`
