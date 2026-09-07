# CRATES_SPECIFICATION.md — Dicionário e Matriz Arquitetural das 9 Crates Canônicas (v7)

> **ESTATUS DO DOCUMENTO:** CANÔNICO E OBRIGATÓRIO (DOCS-AS-GUARDRAILS)
> **APLICAÇÃO:** Todos os agentes de IA (Cursor, Windsurf, Claude Code) e engenheiros humanos.
> **PROPÓSITO:** Congelar as fronteiras de software, eliminar acoplamentos circulares e erradicar o _Context Rot_ por invasão de escopo no Cargo Workspace do **Souls Engine v7**.

## 1. PREÂMBULO E TOPOLOGIA GERAL DO WORKSPACE

O workspace do Souls Engine é concebido como um **Grafo Acíclico Dirigido (DAG)** de dependências estáticas de alta performance, projetado para compilação nativa no Windows 11 MSVC (`x86_64-pc-windows-msvc`) sobre Dev Drive ReFS (`Z:\souls_engine`).

### 1.1 A Lei Inegociável da Topologia de 9 Crates

O workspace é composto por **exatamente 9 crates canônicas**. É expressamente proibido:
1. Adicionar crates satélites temporárias ou auxiliares.
2. Fundir ou colapsar crates existentes.
3. Criar dependências circulares diretas ou indiretas.

### 1.2 Grafo Canônico de Dependências (Matriz de Fluxo)

A tabela abaixo define a matriz estrita de importações permitidas no workspace. Qualquer adição de dependência cruzada fora desta matriz requer aprovação humana prévia e emissão de ADR.


| **Crate de Origem**           | **Dependências Permitidas no Workspace (Pode Importar)**                  | **Crates que Podem Importá-la (Pode ser Importada por)**                                                                                     | **Nível Arquitetural**                 |
| ----------------------------- | ------------------------------------------------------------------------- | -------------------------------------------------------------------------------------------------------------------------------------------- | -------------------------------------- |
| **`souls_protocol`**          | _(Nenhuma dependência de workspace)_                                      | Todas as 8 crates do workspace                                                                                                               | **Nível 0 (Fundação Pura)**            |
| **`souls_core`**              | `souls_protocol`                                                          | `souls_ast`, `souls_memory`, `souls_inference_runtime`, `souls_model_router`, `souls_llm_local_arena`, `souls_anthropophagy`, `souls_server` | **Nível 1 (Infraestrutura Base)**      |
| **`souls_ast`**               | `souls_protocol`, `souls_core`                                            | `souls_anthropophagy`, `souls_server`                                                                                                        | **Nível 2 (Análise Estática)**         |
| **`souls_memory`**            | `souls_protocol`, `souls_core`                                            | `souls_anthropophagy`, `souls_model_router`, `souls_server`                                                                                  | **Nível 2 (Persistência & Hipocampo)** |
| **`souls_inference_runtime`** | `souls_protocol`, `souls_core`                                            | `souls_llm_local_arena`, `souls_model_router`, `souls_server`                                                                                | **Nível 2 (Silício & Runtimes)**       |
| **`souls_model_router`**      | `souls_protocol`, `souls_core`, `souls_memory`, `souls_inference_runtime` | `souls_server`                                                                                                                               | **Nível 3 (Inteligência FinOps)**      |
| **`souls_llm_local_arena`**   | `souls_protocol`, `souls_core`, `souls_inference_runtime`                 | `souls_server`                                                                                                                               | **Nível 3 (Benchmarking & Harness)**   |
| **`souls_anthropophagy`**     | `souls_protocol`, `souls_core`, `souls_ast`, `souls_memory`               | `souls_server`                                                                                                                               | **Nível 3 (Engenharia Reversa)**       |
| **`souls_server`**            | Todas as 8 crates anteriores                                              | _(Nenhuma — Binário Executável Raiz)_                                                                                                        | **Nível 4 (Aplicação Executável)**     |

## 2. ESPECIFICAÇÃO DETALHADA DAS 9 CRATES

### 2.1 `souls_protocol`

#### Missão Canônica Única

Centralizar os contratos de dados agnósticos, envelopes de comunicação JSON-RPC 2.0, esquemas formais de ferramentas MCP e definições estruturais Arrow, garantindo interoperabilidade com zero dependências de I/O de rede ou disco.

#### Grafo de Dependências

- **Dependências do Workspace:** Nenhuma.
- **Crates Dependentes:** Todas (`souls_core`, `souls_ast`, `souls_memory`, `souls_inference_runtime`, `souls_model_router`, `souls_llm_local_arena`, `souls_anthropophagy`, `souls_server`).
- **Crates Externas Homologadas:** `serde` (com feature `derive`), `serde_json`, `arrow` (versão alinhada ao LanceDB), `thiserror`.

#### Estrutura Canônica de Módulos (`crates/souls_protocol/src/`)

- `lib.rs`: Reexportação canônica e declaração da macro `#![forbid(unsafe_code)]`.
- `dto.rs`: Data Transfer Objects universais (telemetria, métrica $E^3$, status do hardware).
- `mcp.rs`: Estruturas do protocolo Model Context Protocol (RPC request, response, error, tool definitions).
- `lean.rs`: Tipos de dados compactados sob a notação LEAN (representação sintética sem ruído).
- `error.rs`: Enumeração canônica de códigos de erro JSON-RPC e erros de serialização.

#### Superfície Pública Canônica

- `struct JsonRpcRequest<T>` / `struct JsonRpcResponse<T>` / `struct JsonRpcError`
- `struct McpToolDefinition` / `struct McpToolCall` / `struct McpToolResult`
- `enum EpistemicPartition { Stable, Evolving }`
- `struct TelemetrySnapshot` / `struct FinOpsMetricE3`

#### Anti-Patterns Estritos (Linhas Vermelhas)

- **PROIBIDO** importar `tokio`, `std::net`, `reqwest` ou qualquer crate de runtime de I/O.
- **PROIBIDO** acessar o sistema de arquivos (`std::fs`).
- **PROIBIDO** introduzir tipos dependentes de arquiteturas específicas (ex: Win32 FFI).
- **PROIBIDO** qualquer uso de código `unsafe`.

#### Comando de Validação

```
cargo check -p souls_protocol
cargo test -p souls_protocol
```

### 2.2 `souls_core`

#### Missão Canônica Única

Prover serviços bare-metal de baixo nível, inicialização do Tokio Runtime, telemetria estruturada, barreiras assíncronas contra pânico, buffers de agregação MPSC para proteção do NVMe e abstrações de I/O no Dev Drive ReFS.

#### Grafo de Dependências

- **Dependências do Workspace:** `souls_protocol`.
- **Crates Dependentes:** `souls_ast`, `souls_memory`, `souls_inference_runtime`, `souls_model_router`, `souls_llm_local_arena`, `souls_anthropophagy`, `souls_server`.
- **Crates Externas Homologadas:** `tokio` (features `full`), `tracing`, `tracing-subscriber`, `tracing-appender`, `windows-sys` (`0.59`), `thiserror`.

#### Estrutura Canônica de Módulos (`crates/souls_core/src/`)

- `lib.rs`: Ponto de entrada da infraestrutura e exportação de guardrails de inicialização.
- `config.rs`: Leitor e validador de configurações centrais com ancoragem estrita em `Z:\souls_engine\.souls_data\`.
- `logging.rs`: Tracing estruturado assíncrono com rotação não bloqueante.
- `panic.rs`: Envoltórios `catch_unwind` assíncronos para contenção de pânicos em tarefas do Tokio.
- `mpsc_flusher.rs`: Canal produtor-consumidor MPSC para amortização de escritas em disco (Batch Flushing de 5s).
- `fs.rs`: Primitivos de manipulação no Dev Drive ReFS com validação de caminho contra vazamento para `C:\`.

#### Superfície Pública Canônica

- `pub fn init_tracing(log_level: &str) -> Result<WorkerGuard, CoreError>`
- `pub async fn run_safe_task<F, T>(future: F) -> Result<T, CoreError> where F: Future<Output = T>`
- `pub struct MpscBatchBuffer<T> { tx: tokio::sync::mpsc::Sender<T> }`
- `pub fn ensure_refs_directory(path: &Path) -> Result<(), CoreError>`

#### Anti-Patterns Estritos (Linhas Vermelhas)

- **PROIBIDO** importar runtimes de IA (`llama.cpp`, `ort`) ou bibliotecas de parsing sintático.
- **PROIBIDO** referenciar bibliotecas de banco de dados (`sqlx`, `lancedb`).
- **PROIBIDO** utilizar `winapi v0.3.9` ou `core_affinity` (usar exclusivamente `windows-sys = "0.59"`).
- **PROIBIDO** realizar escritas síncronas bloqueantes de telemetria direto no disco.

#### Comando de Validação

```
cargo check -p souls_core
cargo test -p souls_core
```

### 2.3 `souls_ast`

#### Missão Canônica Única

Executar análise sintática estática de código-fonte em alta velocidade e em memória, extraindo esqueletos estruturais desidratados (outlines), recortes semânticos de escopo (slices), Myers Diffs seguros e métricas de calor via Frecency gitoxide.

#### Grafo de Dependências

- **Dependências do Workspace:** `souls_protocol`, `souls_core`.
- **Crates Dependentes:** `souls_anthropophagy`, `souls_server`.
- **Crates Externas Homologadas:** `tree-sitter`, `tree-sitter-rust`, `tree-sitter-python`, `tree-sitter-typescript`, `tree-sitter-go`, `similar` (algoritmo Myers Diff), `gix` (gitoxide v0.66+).

#### Estrutura Canônica de Módulos (`crates/souls_ast/src/`)

- `lib.rs`: Interface pública para desidratação e fatiamento sintático.
- `treesitter.rs`: Gerenciador de parsers, compilação de gramáticas nativas e cursor pools.
- `outline.rs`: Algoritmo de desidratação na origem (Source-Side Dehydration) eliminando corpos de funções.
- `slice.rs`: Localizador pontual de nós sintáticos por símbolo (`AIAgent::step`, `struct GgufHeader`).
- `myers.rs`: Computador de diferenças mínimas com verificação de invariantes estruturais de fechamento.
- `heatmap.rs`: Extrator de histórico de commits via `gitoxide` para cálculo da métrica de Frecency ($F$).

#### Superfície Pública Canônica

- `pub fn generate_outline(code: &str, language: SupportedLang) -> Result<String, AstError>`
- `pub fn extract_symbol_slice(code: &str, language: SupportedLang, query: &str) -> Result<String, AstError>`
- `pub fn compute_safe_myers_diff(old_content: &str, new_content: &str) -> Result<MyersPatch, AstError>`
- `pub fn calculate_repo_frecency(repo_path: &Path, time_window_days: u32, half_life_days: f64) -> Result<Vec<FileHeatEntry>, AstError>`

#### Anti-Patterns Estritos (Linhas Vermelhas)

- **PROIBIDO** ler arquivos de disco de forma descontrolada dentro dos parsers (a função deve receber `&str` ou `&[u8]`).
- **PROIBIDO** gerar código através de heurísticas regex ingênuas onde Tree-sitter for obrigatório.
- **PROIBIDO** salvar ou emitir saídas contendo quebras de linha CRLF (`\r\n`).
- **PROIBIDO** invocar o binário `git.exe` via `std::process::Command` (utilizar estritamente a API bare-metal do `gitoxide`).

#### Comando de Validação

```
cargo check -p souls_ast
cargo test -p souls_ast
```

### 2.4 `souls_memory`

#### Missão Canônica Única

Operar como o Hipocampo L3 soberano, gerenciando com exclusividade o banco relacional FrankenSQLite em modo STRICT WAL, o repositório vetorial LanceDB em Dev Drive, o grafo ontológico LadybugDB e as rotinas noturnas do Chyros Daemon.

#### Grafo de Dependências

- **Dependências do Workspace:** `souls_protocol`, `souls_core`.
- **Crates Dependentes:** `souls_anthropophagy`, `souls_model_router`, `souls_server`.
- **Crates Externas Homologadas:** `sqlx` (com SQLite, runtime-tokio-rustls), `lancedb`, `arrow`, `petgraph` (base do grafo LadybugDB), `thiserror`.

#### Estrutura Canônica de Módulos (`crates/souls_memory/src/`)

- `lib.rs`: Coordenação do subsistema de memória e pooling de conexões.
- `sqlite.rs`: Pool de conexões FrankenSQLite STRICT WAL, aplicação de PRAGMAs NT e migrations DDL.
- `lancedb.rs`: Camada colunar vetorial com governança de arquivos `mmap` e proteção Win32.
- `ladybug.rs`: Grafo ontológico de dependências em memória RAM com busca em largura (BFS).
- `chyros.rs`: Daemon assíncrono noturno executando Langevin Decay e `VACUUM INTO`.
- `rrf.rs`: Reciprocal Rank Fusion combinando resultados BM25 (SQLite FTS5) e cosseno (LanceDB).

#### Superfície Pública Canônica

- `pub struct SoulsMemoryStore { sqlite_pool: SqlitePool, lance_table: lancedb::Table, ... }`
- `pub async fn hybrid_recall(&self, query: &str, limit: usize) -> Result<Vec<MemoryRecallEntry>, MemoryError>`
- `pub async fn persist_turn(&self, session_id: &str, user: &str, assistant: &str) -> Result<(), MemoryError>`
- `pub async fn execute_checkpoint_v2(&self, session_id: &str, messages: &[MessageDto]) -> Result<(), MemoryError>`
- `pub async fn run_chyros_metabolism(&self) -> Result<ChyrosReport, MemoryError>`

#### Anti-Patterns Estritos (Linhas Vermelhas)

- **PROIBIDO** permitir que processos externos abram arquivos LanceDB em `Z:\souls_engine\.souls_data\vectors\` diretamente.
- **PROIBIDO** criar tabelas no SQLite sem a cláusula obrigatória `STRICT`.
- **PROIBIDO** executar `PRAGMA journal_mode = DELETE` ou alterar o modo WAL.
- **PROIBIDO** aplicar decaimento estocástico (Langevin Decay) sobre a partição `STABLE`.
- **PROIBIDO** alocar tensores em memória de vídeo (VRAM). O LanceDB deve rodar com **0 MB de VRAM**.

#### Comando de Validação

```
cargo check -p souls_memory
cargo test -p souls_memory
```

### 2.5 `souls_inference_runtime`

#### Missão Canônica Única

Prover a camada bare-metal de execução de silício local, gerenciando o ONNX Runtime na CPU (Tier 0), llama.cpp para logit probing na CPU (Tier 0.5), inferência acelerada CUDA na RTX 2060m (Tier 1), leitor GGUF zero-copy em $\mathcal{O}(1)$, monitor térmico NVML e esteira de Response Healing.

#### Grafo de Dependências

- **Dependências do Workspace:** `souls_protocol`, `souls_core`.
- **Crates Dependentes:** `souls_llm_local_arena`, `souls_model_router`, `souls_server`.
- **Crates Externas Homologadas:** `ort` (ONNX Runtime com CPU AVX2), `memmap2`, `nvml-wrapper`, `serde_json`, `thiserror`.

#### Estrutura Canônica de Módulos (`crates/souls_inference_runtime/src/`)

- `lib.rs`: Interface unificada de despacho de inferência local e alocação de tensores.
- `onnx.rs`: Driver de inferência rápida em CPU para ModernBERT / GLiClass (Tier 0).
- `llamacpp.rs`: Interface nativa com o backend llama.cpp para CPU (Tier 0.5) e dGPU CUDA (Tier 1).
- `gguf_mmap.rs`: Parser de cabeçalhos binários GGUF em $\mathcal{O}(1)$ via visualizações de memória virtual.
- `nvml.rs`: Watchdog termodinâmico com amostragem a cada 500 ms de VRAM e temperatura da RTX 2060m.
- `healing.rs`: Algoritmo determinístico baseado em pilha para correção em voo de JSONs truncados ou defeituosos.

#### Superfície Pública Canônica

- `pub struct HardwareWatchdog { ... }`
- `pub fn inspect_gguf_metadata_o1(path: &Path) -> Result<GgufMetadataInfo, InferenceError>`
- `pub async fn infer_tier0_classify(&self, input: &str) -> Result<ClassificationOutput, InferenceError>`
- `pub async fn infer_tier1_generate(&self, prompt: &str, params: &GenParams) -> Result<String, InferenceError>`
- `pub fn heal_json_response(raw: &str) -> Result<serde_json::Value, InferenceError>`

#### Anti-Patterns Estritos (Linhas Vermelhas)

- **PROIBIDO** alocar mais de 5.294 MB de VRAM sob qualquer hipótese na RTX 2060m (teto útil WDDM).
- **PROIBIDO** manter Tier 1 e Tier 2 simultaneamente ativos na memória dedicada da GPU (Exclusão Mútua de VRAM).
- **PROIBIDO** acionar shared memory fallback via barramento PCIe (o Watchdog deve abortar antes do spillover).
- **PROIBIDO** código `unsafe` sem a justificativa documental obrigatória `// SAFETY: <racional>` (delimitado a FFI e `memmap2`).

#### Comando de Validação

```
cargo check -p souls_inference_runtime
cargo test -p souls_inference_runtime
```

### 2.6 `souls_model_router`

#### Missão Canônica Única

Implementar o motor de inteligência FinOps e roteamento contextual bayesiano multiobjetivo (**ParetoBandit**), calculando a fronteira de Pareto com barreira termodinâmica $\Phi$ e métrica $E^3$ exclusivamente para subagentes (`delegate_task`) e slots auxiliares (`auxiliary.*`).

#### Grafo de Dependências

- **Dependências do Workspace:** `souls_protocol`, `souls_core`, `souls_memory`, `souls_inference_runtime`.
- **Crates Dependentes:** `souls_server`.
- **Crates Externas Homologadas:** `rand` (amostragem bayesiana), `rand_distr` (distribuição Beta), `serde`, `thiserror`.

#### Estrutura Canônica de Módulos (`crates/souls_model_router/src/`)

- `lib.rs`: Orquestração da decisão de roteamento e registro de recompensas.
- `bandit.rs`: Algoritmo Contextual Multi-Armed Bandit com Thompson Sampling e barreira termodinâmica $\Phi$.
- `finops.rs`: Calculadora da Métrica $E^3$ (Eficácia, Economia, Eficiência) e gestão de custos por token.
- `context.rs`: Extrator de recursos contextuais da tarefa ($x_i \in \mathbb{R}^d$: contagem de tokens, densidade de código, complexidade ciclomática).
- `priors.rs`: Persistência e atualização contínua dos hiperparâmetros $\alpha$ e $\beta$ no SQLite.

#### Superfície Pública Canônica

- `pub struct ParetoBanditRouter { ... }`
- `pub async fn route_task(&self, task: &TaskContext) -> Result<RoutingDecision, RouterError>`
- `pub async fn record_outcome(&self, task_id: &str, outcome: &TaskOutcome) -> Result<(), RouterError>`
- `pub fn calculate_e3_metric(structural_score: f64, direct_cost: f64, latency_sec: f64) -> f64`

#### Anti-Patterns Estritos (Linhas Vermelhas)

- **PROIBIDO** aplicar chaveamento de modelo turno-a-turno dentro da conversa principal (_intradialogue turn-level switching_).
- **PROIBIDO** carregar pesos de modelos de linguagem ou executar inferência dentro desta crate.
- **PROIBIDO** hardcodar preços de APIs ou priors bayesianos sem possibilidade de calibração dinâmica.

#### Comando de Validação

```
cargo check -p souls_model_router
cargo test -p souls_model_router
```

### 2.7 `souls_llm_local_arena`

#### Missão Canônica Única

Executar suítes de benchmarking empírico contínuo, aferição de tokens por segundo reais no hardware da máquina (Intel i9 + RTX 2060m), testes de rigidez sintática contra gramáticas JSON com `llguidance` e alimentação empírica dos priors do ParetoBandit.

#### Grafo de Dependências

- **Dependências do Workspace:** `souls_protocol`, `souls_core`, `souls_inference_runtime`.
- **Crates Dependentes:** `souls_server`.
- **Crates Externas Homologadas:** `llguidance`, `tokio`, `serde`, `serde_json`, `thiserror`.

#### Estrutura Canônica de Módulos (`crates/souls_llm_local_arena/src/`)

- `lib.rs`: Ponto de entrada das baterias de estresse de modelos locais.
- `benchmarks.rs`: Harness de velocidade (TTFT, tokens/s sustentados, vazão com prompt de preenchimento).
- `rigidity.rs`: Bateria de validação de quebra sintática contra esquemas estruturados complexos.
- `harness.rs`: Orquestrador de estresse de hardware com injeção de cargas simultâneas CPU/GPU.

#### Superfície Pública Canônica

- `pub async fn run_performance_benchmark(model_path: &Path, test_suite: &BenchmarkSuite) -> Result<BenchmarkReport, ArenaError>`
- `pub async fn evaluate_json_rigidity(tier: ModelTier, schema: &serde_json::Value) -> Result<RigidityScore, ArenaError>`

#### Anti-Patterns Estritos (Linhas Vermelhas)

- **PROIBIDO** executar testes de estresse sem monitoramento simultâneo do Watchdog NVML.
- **PROIBIDO** introduzir dependências de crates de visualização gráfica local.
- **PROIBIDO** executar testes que deliberadamente ignorem os tetos de segurança térmica ($\ge 82^\circ\text{C}$).

#### Comando de Validação

```
cargo check -p souls_llm_local_arena
cargo test -p souls_llm_local_arena
```

### 2.8 `souls_anthropophagy`

#### Missão Canônica Única

Executar a esteira automatizada de engenharia de software reversa, dissecação sintática de repositórios abertos, filtragem de conformidade de licenças (SPDX Checker), traçamento de grafos de chamadas (call-graphs) e extração de algoritmos puros para o catálogo "The Organ Vault" no SQLite.

#### Grafo de Dependências

- **Dependências do Workspace:** `souls_protocol`, `souls_core`, `souls_ast`, `souls_memory`.
- **Crates Dependentes:** `souls_server`.
- **Crates Externas Homologadas:** `spdx-rs`, `petgraph`, `serde`, `thiserror`.

#### Estrutura Canônica de Módulos (`crates/souls_anthropophagy/src/`)

- `lib.rs`: Interface executiva da esteira de dissecação.
- `spdx.rs`: Firewall de licenças (aprovação estrita de MIT, Apache-2.0, BSD; rejeição categórica de GPL/AGPL virais).
- `dissector.rs`: Rastreador de dependências sintáticas e traçador de call-graphs completos via AST.
- `slicer.rs`: Extrator de órgãos (isola funções e estruturas puras removendo dependências contextuais desnecessárias).
- `vault.rs`: Persistência dos algoritmos purificados e metadados de proveniência no FrankenSQLite.

#### Superfície Pública Canônica

- `pub fn verify_license_compliance(repo_path: &Path) -> Result<LicenseStatus, AnthropophagyError>`
- `pub fn dissect_target_repository(repo_path: &Path) -> Result<DissectionReport, AnthropophagyError>`
- `pub async fn slice_and_preserve_algorithm(&self, symbol: &str, ast_ref: &AstNode) -> Result<OrganRecord, AnthropophagyError>`

#### Anti-Patterns Estritos (Linhas Vermelhas)

- **PROIBIDO** incorporar código de repositórios sem validação formal positiva do SPDX Checker.
- **PROIBIDO** clonar repositórios diretamente para fora da partição Dev Drive `Z:\souls_engine\.souls_data\`.
- **PROIBIDO** executar código extraído dentro do ambiente de produção (a dissecação é estritamente estática).

#### Comando de Validação

```
cargo check -p souls_anthropophagy
cargo test -p souls_anthropophagy
```

### 2.9 `souls_server`

#### Missão Canônica Única

Atuar como o daemon executável bare-metal **100% headless** (`souls_server.exe`), unificando o servidor HTTP Axum em `127.0.0.1:9123`, o catálogo MCP via HTTP/SSE, os endpoints REST para o Thin Adapter de memória e Web Dashboard, e a orquestração do Graceful Shutdown de 5 fases no Windows NT.

#### Grafo de Dependências

- **Dependências do Workspace:** Todas as 8 crates anteriores (`souls_protocol`, `souls_core`, `souls_ast`, `souls_memory`, `souls_inference_runtime`, `souls_model_router`, `souls_llm_local_arena`, `souls_anthropophagy`).
- **Crates Dependentes:** Nenhuma (é a raiz executável do binário).
- **Crates Externas Homologadas:** `axum`, `tower-http`, `tokio`, `tracing`, `windows-sys` (`0.59`), `serde_json`.

#### Estrutura Canônica de Módulos (`crates/souls_server/src/`)

- `main.rs`: Ponto de entrada Win32, inicialização do Tokio Runtime bare-metal e orquestração de shutdown.
- `routes_mcp.rs`: Transporte Streamable HTTP/SSE servindo o protocolo MCP na rota `/mcp`.
- `routes_api.rs`: Endpoints REST para o Thin Adapter Python (`/api/v1/memory/*`) e telemetria (`/api/v1/telemetry`).
- `routes_openai.rs`: Endpoint de inferência local compatível com a API da OpenAI para subagentes delegados.
- `shutdown.rs`: Protocolo ordenado de encerramento em 5 fases (Axum -> MPSC -> SQLite WAL -> LanceDB mmap -> CUDA reset).

#### Superfície Pública Canônica

- `fn main() -> Result<(), Box<dyn std::error::Error>>`
- `pub async fn start_axum_server(config: ServerConfig) -> Result<(), ServerError>`
- `pub async fn handle_graceful_shutdown(state: AppState) -> Result<(), ServerError>`

#### Anti-Patterns Estritos (Linhas Vermelhas)

- **PROIBIDO** introduzir qualquer elemento gráfico local (Zero UI, Zero Systray, sem Winit, sem Tauri, sem Svelte embutido).
- **PROIBIDO** escutar em interfaces públicas (vincular estritamente em `127.0.0.1:9123`).
- **PROIBIDO** atuar como proxy interceptador de LLMs na porta `:3001` para o chat mestre do Hermes.
- **PROIBIDO** finalizar o processo sem drenar o canal MPSC e executar o checkpoint do SQLite WAL.

#### Comando de Validação

```
cargo check -p souls_server
cargo test -p souls_server
```

## 3. MATRIZ DE ISOLAMENTO E NÃO-INVASÃO DE ESCOPO

Para impedir o _Context Rot_ e evitar que agentes misturem responsabilidades de baixo nível, a matriz abaixo define com precisão cirúrgica quais crates têm permissão para manipular determinados subsistemas físicos ou lógicos:

| **Subsistema / Recurso Técnico**          | **Crates AUTORIZADAS**                       | **Crates ESTRITAMENTE PROIBIDAS**                   |
| ----------------------------------------- | -------------------------------------------- | --------------------------------------------------- |
| **I/O Físico no Disco (Dev Drive)**       | `souls_core`, `souls_memory`, `souls_server` | `souls_protocol`, `souls_ast`, `souls_model_router` |
| **Aceleração CUDA & FFI do llama.cpp**    | `souls_inference_runtime`                    | Todas as outras 8 crates                            |
| **Acesso FFI à NVML (NVIDIA Driver)**     | `souls_inference_runtime`                    | Todas as outras 8 crates                            |
| **Conexões SQLite & Consultas SQL**       | `souls_memory`                               | Todas as outras 8 crates                            |
| **Manipulação de Arquivos Arrow/LanceDB** | `souls_memory`                               | Todas as outras 8 crates                            |
| **Parsing Tree-sitter & Myers Diff**      | `souls_ast`                                  | Todas as outras 8 crates                            |
| **Servidor HTTP & Rotas de Rede**         | `souls_server`                               | Todas as outras 8 crates                            |
| **Modelagem Thompson Sampling / Pareto**  | `souls_model_router`                         | Todas as outras 8 crates                            |
| **Tipagem e Serialização Pura**           | `souls_protocol` (canônica), Todas           | _(Todas consomem `souls_protocol`)_                 |

## 4. PROTOCOLO DE EXPANSÃO E HIGIENE DO WORKSPACE

1. **Adição de Novos Tipos:** Se um tipo precisar ser compartilhado entre duas ou mais crates, ele **pertence compulsoriamente a `souls_protocol`**. Nunca crie structs duplicadas em crates diferentes.
2. **Uso de Dependências Externas no `Cargo.toml`:** A adição de qualquer nova dependência externa nos arquivos `Cargo.toml` deve:
    - Ser declarada preferencialmente no `[workspace.dependencies]` do arquivo raiz.
    - Não violar a compatibilidade com a compilação MSVC pura (`windows-sys`).
    - Ser compatível com as flags de arquitetura do processador (`AVX2`).
3. **Isolamento de Testes Unitários:** Cada crate deve manter seus testes em sua própria pasta `tests/` ou dentro de módulos `#[cfg(test)]` nos arquivos de código, garantindo que `cargo test -p <crate>` execute em menos de 10 segundos para manter o ciclo de feedback do desenvolvedor ultraveloz.