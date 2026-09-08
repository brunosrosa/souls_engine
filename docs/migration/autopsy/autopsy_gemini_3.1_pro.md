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

------

# DESIGN.md — Arquitetura de Transplante e Sinergia de Crates

## 1. Mapeamento Topológico de Transplante

Os componentes viáveis de `_donor/` serão integrados dentro da estrita topologia de 9 crates canônicas do Souls Engine v7. **A criação de uma 10ª crate foi categoricamente descartada**, pois a matriz das 9 crates atuais é suficiente para abraçar todos os domínios mapeados sem introduzir acoplamentos cíclicos ou "Context Rot".

### Mapeamento Direto por Crate de Destino

1. **`souls_anthropophagy`:**
   - Absorverá integralmente `souls_mc_anthropophagy/src/harvester/*`.
   - Inclui execução isolada do SAST (`sast/*.rs`) e as verificações estáticas de SPDX.

2. **`souls_inference_runtime`:**
   - Recebe a lógica refinada de `vram_hardware/*` (Watchdogs de memória via NVML).
   - Incorpora `inference/ort_scorer.rs` e `gliclass_engine.rs` como motores Tier 0.
   - Incorpora `inference/llama_logit_probing.rs` como Tier 0.5 para sondagem de tokens locais.

3. **`souls_ast`:**
   - Incorpora `ast/repo_heatmap.rs` e `ast/repo_impact.rs` ancorados firmemente sobre a crate auxiliar `gix`.
   - Incorpora o algoritmo matemático limpo contido em `context/myers_diff.rs`.
   - Preserva o isolamento de segurança WASM (WASI 0.2/0.3 via `Wasmtime`) para os parsers Tree-sitter, descartando o uso de chamadas FFI C nativas para evitar crashes no processo principal, mantendo `ts_set_allocator` (bump-arena) e `memmap2` para performance sub-milissegundo.
   - Incorpora `context/souls_symbol.rs`.

4. **`souls_core`:**
   - Acolhe o motor de leitura inteligente: `context/souls_smart_read.rs` e `multi_read.rs`.
   - Integra as primitivas de compressão LEAN: `context/ansi_filter.rs`, `dedup.rs` e `ccr_dedup.rs`.
   - Concentra a barreira de falha e `mpsc_flusher`.

5. **`souls_memory`:**
   - Funde os arquivos fragmentados de `memory/` (Chyros Daemon, Langevin Decay, RRF Fusion).
   - Engloba a manipulação do LadybugDB (atualmente em `state_thinking/memory_graph/`) e acesso vetorial `vector_store.rs`.

6. **`souls_protocol`:**
   - Concentrará todos os DTOs universais legados transferidos durante a depuração de tipagem.

## 2. Padrões de Assinaturas, Concorrência e Tipagem

### 2.1 Isolamento de Inanição de Threads (`spawn_blocking`)
A execução de I/O bloqueante (fora do escopo natural assíncrono), rotinas computacionais densas (Myers Diff, inferência de CPU pura) ou parsing (Tree-sitter) deverão utilizar envoltórios para proteção do pool do Tokio:

```rust
// Exemplo canônico de assinatura transplantada:
pub async fn compute_safe_myers_diff(old_content: &str, new_content: &str) -> Result<MyersPatch, AstError> {
    let old_copy = old_content.to_owned();
    let new_copy = new_content.to_owned();
    tokio::task::spawn_blocking(move || {
        // Computação cpu-bound isolada
        similar_engine::diff(&old_copy, &new_copy)
    }).await.map_err(|_| AstError::InternalEnginePanic)?
}
```

### 2.2 ReFS Win32 e Block Cloning (ADR-005)

Sempre que um transplante (notadamente na crate `souls_memory` e manipulações do LanceDB ou cópia massiva do `Chyros Daemon`) tocar no disco `Z:\souls_engine\.souls_data\`, chamadas puras da biblioteca padrão (`std::fs::copy`) deverão ser evitadas em favor de Block Cloning nativo via Kernel NT para evitar SSD Write Amplification.

O código deve utilizar a API Win32 assíncrona (com `windows-sys` v0.59+ `Win32_Storage_FileSystem` e `Win32_System_Threading`) para executar o control code `FSCTL_DUPLICATE_EXTENTS_TO_FILE` via `DeviceIoControl`:

```rust
use std::os::windows::io::AsRawHandle;
use windows_sys::Win32::Storage::FileSystem::{FILE_SHARE_READ, FILE_SHARE_WRITE, FILE_SHARE_DELETE};
use windows_sys::Win32::System::Ioctl::FSCTL_DUPLICATE_EXTENTS_TO_FILE;
use windows_sys::Win32::System::IO::DeviceIoControl;

pub async fn clone_file_refs(src: &Path, dest: &Path) -> Result<(), CoreError> {
    // ... abertura de handles (CreateFileW) ...
    // Exemplo de ioctl para Block Cloning Copy-on-Write (tempo zero)
    // DeviceIoControl(
    //     dest_handle,
    //     FSCTL_DUPLICATE_EXTENTS_TO_FILE,
    //     &duplicate_request as *const _ as *const c_void,
    //     ...
    // );
}
```

### 2.3 Matriz de Domínio de Erros MCP (`thiserror`)

Qualquer transplante envolverá a purga absoluta de `anyhow` do código de produção. As respostas das funções migrarão de `anyhow::Result<T>` para uma tipagem estrita via `thiserror` (v2) em `souls_protocol`, de modo que os erros sejam mapeáveis diretamente para os códigos JSON-RPC padronizados (faixa `-32000` a `-32099`):

```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum McpDomainError {
    #[error("InternalEnginePanic: Pânico recuperado na thread Tokio")]
    InternalEnginePanic, // Código -32000

    #[error("FileTooLargeForAST: O arquivo excedeu o limite físico")]
    FileTooLargeForAST, // Código -32001

    #[error("VramThermalThrottled: Watchdog NVML disparou barreira")]
    VramThermalThrottled, // Código -32002
    
    #[error("SymbolNotFound: O identificador não foi achado na AST")]
    SymbolNotFound, // Código -32003
}
```

------

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


------

# Mermaid Graph

graph TD
    %% Nós Raízes e Meta-Dependências
    TASK-PRT-001[TASK-PRT-001: Mapear Tipos de Telemetria e FinOps]

    %% Infraestrutura e Contexto
    TASK-COR-001[TASK-COR-001: Filtros de ANSI e Telemetria]
    TASK-COR-002[TASK-COR-002: Motor de Leitura LEAN]
    TASK-COR-003[TASK-COR-003: Integração de ReFS Block Cloning]

    %% AST e Análise
    TASK-AST-001[TASK-AST-001: Integração de Diferencial Myers]
    TASK-AST-002[TASK-AST-002: Motor de Heatmap Frecency]
    TASK-AST-003[TASK-AST-003: Sandbox Wasmtime para Parsers Tree-sitter]

    %% Persistência L3
    TASK-MEM-001[TASK-MEM-001: Fusão RRF e Decaimento Langevin]
    TASK-MEM-002[TASK-MEM-002: Consolidar Grafo Ladybug]

    %% Inferência e VRAM
    TASK-INF-001[TASK-INF-001: NVML Hardware Watchdog e EWMA]
    TASK-INF-002[TASK-INF-002: Integração de Tier 0 ONNX/Llamacpp]

    %% Esteira Automática SAST
    TASK-ANT-001[TASK-ANT-001: Purga e Transplante do Harvester]

    %% Fim de processo
    TASK-CLN-001[TASK-CLN-001: Expurgo Físico de Necrose]

    %% Relacionamentos Topológicos Rigorosos
    TASK-PRT-001 --> TASK-COR-001
    TASK-PRT-001 --> TASK-AST-001
    TASK-PRT-001 --> TASK-MEM-001

    TASK-COR-001 --> TASK-COR-002
    TASK-COR-002 --> TASK-COR-003
    TASK-COR-003 --> TASK-MEM-002

    TASK-AST-001 --> TASK-AST-002
    TASK-AST-002 --> TASK-AST-003
    TASK-AST-003 --> TASK-ANT-001

    TASK-MEM-001 --> TASK-MEM-002
    TASK-MEM-002 --> TASK-INF-001

    TASK-INF-001 --> TASK-INF-002

    %% Cleanup depende da compilação e teste finalizados de todas as crates vitais
    TASK-COR-003 --> TASK-CLN-001
    TASK-ANT-001 --> TASK-CLN-001
    TASK-INF-002 --> TASK-CLN-001
