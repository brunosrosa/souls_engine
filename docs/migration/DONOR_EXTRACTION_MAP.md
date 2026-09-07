# DONOR_EXTRACTION_MAP.md — Protocolo Forense, Bússola de Autópsia e Roteiro de Transplante do Código Legado (`_donor/`) (v7)

> **ESTATUS DO DOCUMENTO:** CANÔNICO E OBRIGATÓRIO (DOCS-AS-GUARDRAILS)
> **APLICAÇÃO:** Agentes de IA (Gemini 3.1 Pro, Gemini 3.8 Flash, Cursor, Windsurf, Claude Code) e Engenheiros Principais de Sistemas.
> **PROPÓSITO:** Fornecer a metodologia de dissecação investigativa sobre o acervo legado em `_donor/`, estabelecer a divisão cognitiva de trabalho (Gemini 3.1 Pro para SDD/DAG e Gemini 3.8 Flash para execução TDD), classificar os órgãos de código e garantir que nenhum patrimônio técnico seja descartado sem auditoria prévia.

## 1. FILOSOFIA DA OPERAÇÃO E SIMBIOSE COGNITIVA DOS AGENTES

A migração do código legado em `_donor/` para o workspace canônico do **Souls Engine v7** não é um processo de cópia mecânica, mas uma **autópsia investigativa seguida de transplante cirúrgico**.

A base em `_donor/` contém órgãos de engenharia bare-metal de altíssimo valor (algoritmos matemáticos, drivers de hardware, compiladores AST e esteiras de segurança), mas também carrega tecidos necrosados da era desktop (Tauri v2, Wry/Winit, sockets de proxy L7 `:3001` e caminhos de arquivos legados).

Para executar essa transição com fidelidade absoluta e custo FinOps balanceado, o processo adota uma **divisão assimétrica de inteligência**:

### 1.1 O Arquiteto e Cirurgião-Chefe: Gemini 3.1 Pro (High Reasoning)

- **Papel:** Investigação forense profunda, diagnóstico de dependências circulares, desenho de arquitetura de dados e planejamento formal.
- **Metodologia:** Opera estritamente sob **Spec-Driven Development (SDD)**. Antes de qualquer linha de código ser migrada, o Gemini 3.1 Pro inspeciona o subsistema-alvo e gera:
    1. `PLAN.md`: Racional arquitetural, motivação de engenharia e inventário detalhado de structs e funções do órgão.
    2. `DESIGN.md`: Assinaturas públicas canônicas, DTOs em `souls_protocol`, interfaces de traits, isolamento de erros (`thiserror`) e esquemas DDL/Arrow.
    3. `TASKS.md`: Relação sequencial de micro-tarefas com critérios atômicos de Definition of Done (DoD).
    4. `DAG.md`: Grafo Acíclico Dirigido com a ordem topológica exata de execução das tarefas, mapeando pré-requisitos e paralelismos seguros.
- **Soberania Topológica:** O Gemini 3.1 Pro possui autorização formal para propor novas crates ou desmembrar módulos se demonstrar que a separação preserva a compilação limpa e o isolamento de escopo.

### 1.2 O Engenheiro de Implementação e Compilação: Gemini 3.8 Flash (Medium Reasoning)

- **Papel:** Execução cirúrgica, transcrição de código, adaptação de imports e verificação estrita de compilação.
- **Metodologia:** Consome nó a nó a DAG produzida pelo Pro, operando sob o ciclo **TDD (Red-Green-Refactor)**:
    1. Cria os testes unitários da unidade com base no `DESIGN.md` (Compilação do teste em estado **RED**).
    2. Transplanta o código do órgão legado, higienizando dependências e adaptando para o ecossistema v7 (Estado **GREEN**).
    3. Roda `cargo check -p <crate>` e `cargo test -p <crate>` até eliminar qualquer warning ou erro.
    4. Marca a tarefa como concluída na DAG e avança para o próximo nó.

## 2. MAPA DE AROMAS E CLUSTERS FORENSES EM `_donor/`

A árvore física de `_donor/` abriga implementações dispersas em três raízes principais: `souls_mc_anthropophagy`, `souls_mc_core` e `souls_mc_protocol`. A tabela abaixo mapeia os clusters funcionais identificados, o "aroma" técnico de sua localização e a crate canônica de destino:

| **Cluster Funcional**                          | **O Que Representa no Silício**                                                                                                            | **Localização Típica em _donor/**                                                                                                                  | **Crate Canônica v7 Alvo**                   |
| ---------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------ | -------------------------------------------------------------------------------------------------------------------------------------------------- | -------------------------------------------- |
| **Cluster 1: Hardware & Termodinâmica**        | Watchdogs térmicos NVML, predição de calor via EWMA, profilers de VRAM e isolamento PCIe contra spillover.                                 | `souls_mc_core/src/core/vram_hardware/` (`hardware_watchdog.rs`, `peak_ewma.rs`, `headroom_engine.rs`, `souls_thermal_governor.rs`)                | `souls_inference_runtime`                    |
| **Cluster 2: Hipocampo & Memória Triad L3**    | FrankenSQLite STRICT WAL, LanceDB vetorial, FTS5 BM25, Fusão RRF, Grafo LadybugDB e Chyros Daemon (Langevin Decay).                        | `souls_mc_core/src/cognition/memory/`<br><br>  <br><br>`souls_mc_core/src/cognition/state_thinking/memory_graph/`                                  | `souls_memory`                               |
| **Cluster 3: Motores de Inferência & Probing** | Drivers ONNX para ModernBERT/GLiClass, BitNet 1-bit, logit probing llama.cpp (Gemma 2B), Burn framework e parser GGUF $\mathcal{O}(1)$.    | `souls_mc_core/src/core/inference/` (`ort_scorer.rs`, `gliclass_engine.rs`, `llama_logit_probing.rs`, `bitnet_*.rs`, `burn_engine.rs`)             | `souls_inference_runtime`                    |
| **Cluster 4: FinOps & ParetoBandit**           | Otimizador bayesiano multiobjetivo Thompson Sampling, Métrica $E^3$ e calculadoras de custo de tokens.                                     | `souls_mc_core/src/finops/` (`pareto_bandit.rs`, `finops_router.rs`, `iron_cost.rs`)                                                               | `souls_model_router`                         |
| **Cluster 5: Análise Sintática AST & Diff**    | Tree-sitter nativo/WASM, extração de esqueleto (outlines), fatiamento semântico (`slice`), Myers Diff estrutural e Frecency `gitoxide`.    | `souls_mc_core/src/cognition/ast/`<br><br>  <br><br>`souls_mc_core/src/cognition/context/` (`myers_diff.rs`, `souls_symbol.rs`, `repo_heatmap.rs`) | `souls_ast`                                  |
| **Cluster 6: Engenharia de Contexto LEAN**     | Desidratação e reidratação em RAM (CCR), filtros ANSI, compressão semântica e deduplicação de blocos de 5+ linhas.                         | `souls_mc_core/src/cognition/context/` (`ansi_filter.rs`, `dedup.rs`, `ccr_dedup.rs`, `multi_read.rs`, `souls_smart_read.rs`)                      | `souls_core` e `souls_protocol`              |
| **Cluster 7: Cognição Socrática & DAG**        | Freio cognitivo `core_think`, ramificação de hipóteses, árvores de pensamento DAG e álgebra de cohomologia epistêmica.                     | `souls_mc_core/src/cognition/state_thinking/thinking/`<br><br>  <br><br>`souls_mc_core/src/core/socratic/`                                         | `souls_core` (lógica) / `souls_server` (RPC) |
| **Cluster 8: Dissecação SAST & Anthropophagy** | Matriz com 11 analisadores estáticos (oxc, ruff, clippy, bandit, etc.), radar de repositórios, verificador de licenças SPDX e Organ Vault. | `souls_mc_anthropophagy/src/` (`harvester/sast/*`, `distillation/*`, `swarm.rs`, `synthesizer.rs`)                                                 | `souls_anthropophagy`                        |
| **Cluster 9: Transporte & Catálogo MCP**       | Despachante JSON-RPC 2.0, isolamento stdio contra ANSI, ganchos de stream SSE e envelopes padronizados.                                    | `souls_mc_core/src/bin/souls_mcp_server/`<br><br>  <br><br>`souls_mc_core/src/core/governance/mcp_transport.rs`                                    | `souls_server`                               |

## 3. AUDITORIA FORENSE DE DUPLICAÇÕES E ANOMALIAS IDENTIFICADAS

A varredura preliminar da árvore `_donor/` identificou anomalias estruturais críticas que exigem atenção cirúrgica do Gemini 3.1 Pro durante o diagnóstico:

### 3.1 A Duplicação Flagrante do Subsistema `harvester`

- **Evidência:** Existem simultaneamente:
    - `_donor/souls_mc_anthropophagy/src/harvester/`
    - `_donor/souls_mc_core/src/harvester/`
- **Diagnóstico de Engenharia:** A pasta dentro de `souls_mc_anthropophagy` é muito mais completa e especializada, contendo os linters SAST poliglotas (`bandit`, `biome`, `clippy`, `cppcheck`, `govulncheck`, `oxc`, `ruff`, `sobelow`), integração git e sanitizadores de caminho. O diretório em `souls_mc_core` é um fragmento legado duplicado.
- **Diretriz Canônica:** Consolidar **100% da extração e do SAST dentro de `crates/souls_anthropophagy/`**. O conteúdo correspondente em `souls_mc_core/src/harvester/` deve ser classificado como redundante e descartado após confirmação de paridade funcional.

### 3.2 Gramáticas Tree-sitter: Compilação Nativa C/Rust versus WASM

- **Evidência:** Presença de arquivos binários compilados em `_donor/resources/wasm_grammars/` (`tree_sitter_rust.wasm`, `outline_parser.wasm`) e código de execução em `wasm_engine.rs`.
- **Diagnóstico de Engenharia:** O isolamento WASM via Wasmtime protegia o processo principal contra segmentation faults de parsers C legados. No entanto, no Windows 11 nativo, as crates modernas `tree-sitter-rust`, `tree-sitter-python`, etc., podem ser compiladas diretamente em código de máquina nativo ou encapsuladas em threads Tokio isoladas via `spawn_blocking` e `catch_unwind`.
- **Diretriz de Investigação:** O Gemini 3.1 Pro deve avaliar: se a execução nativa compilada oferece latência sub-milissegundo com contenção de pânico segura, prioriza-se a integração nativa; se houver risco de instabilidade sintática em gramáticas complexas, o módulo Wasmtime é preservado em `souls_ast`.

### 3.3 Fragmentação de Implementações de Memória

- **Evidência:** O código de memória está dividido entre `cognition/memory/` (Chyros, Langevin, RRF) e `cognition/state_thinking/memory_graph/` (Grafo Ladybug, SQLite, MPSC bridge).
- **Diretriz Canônica:** Unificar toda essa topologia em `crates/souls_memory/`, garantindo que o pool do FrankenSQLite STRICT WAL seja a autoridade relacional e o LanceDB governe com exclusividade os vetores em Dev Drive ReFS.

## 4. MATRIZ DE TRIAGEM DOS ÓRGÃOS

Durante a fase de planejamento, cada arquivo ou módulo encontrado em `_donor/` deve ser categorizado formalmente pelo Gemini 3.1 Pro sob uma das quatro classificações:

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                    MATRIZ DE TRIAGEM FORENSE DE ÓRGÃOS                      │
├──────────────────────┬──────────────────────────────────────────────────────┤
│ HEALTHY_ORGAN        │ Código puro, livre de amarras legadas, pronto para  │
│                      │ transplante com mera atualização de imports.         │
├──────────────────────┼──────────────────────────────────────────────────────┤
│ ORGAN_REPAIR         │ Algoritmo valioso, mas acoplado a structs antigas,  │
│                      │ I/O síncrono ou APIs deprecadas. Exige refatoração. │
├──────────────────────┼──────────────────────────────────────────────────────┤
│ UPGRADE_CANDIDATE    │ Stubs, mocks ou protótipos avançados que agora têm  │
│                      │ motores bare-metal prontos na arquitetura v7.        │
├──────────────────────┼──────────────────────────────────────────────────────┤
│ NECROSIS_BIOHAZARD   │ Código tóxico que viola as Linhas Vermelhas da v7.  │
│                      │ INCINERAÇÃO ABSOLUTA E PROIBIÇÃO DE MIGRAÇÃO.       │
└──────────────────────┴──────────────────────────────────────────────────────┘
```

### 4.1 Exemplos Concretos de Triagem

- **`HEALTHY_ORGAN` (Transplante Direto com Ajuste de Namespace):**
    - `myers_diff.rs`: Algoritmo matemático puro de cálculo de diff usando a crate `similar`.
    - `langevin_decay.rs`: Equação estocástica de dissipação de saliência epistêmica.
    - `ansi_filter.rs`: Sanitizador de sequências de escape ANSI em buffers de terminal.
    - `sast/*.rs`: Validadores de ferramentas SAST em linha de comando.
- **`ORGAN_REPAIR` (Refatoração de Tipos e Concorrência Obrigatória):**
    - `pareto_bandit.rs`: Otimizador matemático excelente, mas deve ser desacoplado do antigo gateway `:3001` e conectado às structs de subtarefa (`souls_protocol`).
    - `hardware_watchdog.rs` e `peak_ewma.rs`: Devem ser migrados da antiga crate `core` para `souls_inference_runtime`, amarrados à biblioteca `nvml-wrapper` e às métricas da RTX 2060m.
    - `thinking/engine.rs`: Deve ser purificado de dependências de interface gráfica e conectado ao catálogo MCP.
- **`UPGRADE_CANDIDATE` (Investigar e Conectar aos Runtimes v7):**
    - `intent`: Conectar ao Tier 0 (ModernBERT/GLiClass via ONNX) e Tier 0.5 (Gemma 2B CPU probing) na crate `souls_inference_runtime`.
    - `headroom_retrieve` e `headroom_engine.rs`: Conectar à recuperação de blocos de contexto comprimido em RAM.
    - `bitnet_engine.rs` e `burn_engine.rs`: Avaliar maturação para inclusão como engines experimentais no Tier 1 ou Arena de benchmark.
- **`NECROSIS_BIOHAZARD` (INCINERAÇÃO COMPULSÓRIA — NENHUMA LINHA PASSA):**
    - Qualquer código contendo `use tauri::...`, `use wry::...` ou referências a janelas nativas.
    - O servidor estático Axum que hospedava a UI Svelte na porta `:3002`.
    - O proxy interceptador L7 da porta `:3001` que capturava o fluxo HTTP de LLMs do Hermes.
    - Qualquer código que escreva em `%APPDATA%`, `%LOCALAPPDATA%` ou no registro do Windows (`HKCU\...\Run`).

## 5. AS 6 LEIS FÍSICAS DO TRANSPLANTE NO SILÍCIO (GUARDRAILS)

O Gemini 3.8 Flash (e qualquer outro agente executor) deve seguir compulsoriamente estas seis regras estritas ao manipular arquivos transplantados:

### Lei 1: O "Envenenamento do Cargo.toml" (Dependency Creep Proibido)

É expressamente proibido importar versões arbitrárias de crates externas diretamente no `Cargo.toml` das crates filhas.
- O arquivo raiz `Z:\souls_engine\Cargo.toml` é o **único gestor de versões** via `[workspace.dependencies]`.
- As crates individuais declaram dependências estritamente com `crate_name = { workspace = true }`.
- Se o código legado usava crates depreciadas (ex: `lazy_static`, `winapi`, `anyhow` fora de testes), o agente deve substituí-las pelas ferramentas canônicas (`once_cell`/`std::sync::OnceLock`, `windows-sys = "0.59"`, `thiserror`).

### Lei 2: Prevenção de Inanição de Threads Tokio (`tokio::task::spawn_blocking`)

Operações intensivas em CPU nunca devem travar os workers assíncronos do Tokio.

- Se a rotina executa parsing sintático Tree-sitter de arquivos grandes, cálculo de Frecency `gitoxide`, Myers Diff estrutural, compressão LEAN massiva ou medição de tokens com `tiktoken`, o código deve ser compulsoriamente encapsulado:

    ```
    let result = tokio::task::spawn_blocking(move || {
        // Computação pesada de CPU bare-metal
        parser.compute_syntax_tree(&content)
    }).await??;
    ```

### Lei 3: Erradicação Total de Singletons Globais

No modelo desktop legado, era comum o uso de `lazy_static!` ou `static mut` para gerenciar instâncias globais.
- **Teto Zero de Globais:** Nenhum estado mutável global é permitido no Souls Engine v7.
- Todo o estado (`SqlitePool`, conexões LanceDB, canais MPSC, instâncias de parsers) deve residir no `AppState` instanciado na inicialização do daemon `souls_server` e compartilhado via injeção explícita (`Arc<T>`).

### Lei 4: Higienização Forçada de Finais de Linha (CRLF $\rightarrow$ LF) e Encoding

Arquivos legados desenvolvidos em ambientes Windows mistos podem conter quebras de linha `\r\n` ou marcadores de byte order (UTF-8 com BOM).
- O parser de AST (Tree-sitter) e o motor de Frecency (`gitoxide`) dependem de contagens de bytes exatas. O caractere `\r` causa corrupção no cálculo de ranges (`byte_range`) e quebra diffs cirúrgicos.
- Todo arquivo transplantado deve ser salvo estritamente em **UTF-8 puro (sem BOM)** com quebras de linha **exclusivamente LF (`\n`)**.

### Lei 5: Ciclo TDD Mandatório de Quarentena (Red-Green-Refactor)

Nenhum arquivo de produção entra na árvore canônica sem passar pela esteira:
1. **RED:** Transplanta ou redige primeiro o teste unitário correspondente em `tests/` ou no módulo `#[cfg(test)]`. Confirma que o teste falha ou não compila pela ausência da implementação canônica.
2. **GREEN:** Transplanta e higieniza a lógica de produção do órgão até que o teste passe integralmente.
3. **REFACTOR:** Remove avisos do linter (`cargo clippy -p <crate>`) e confirma que `cargo check -p <crate>` retorna código zero.

### Lei 6: Purga de Caminhos Hardcoded Win32

Qualquer referência textual a `C:\Users\...`, `AppData\Roaming` ou diretórios temporários deve ser purgada.
- Toda a persistência de banco de dados, modelos, vetores e telemetria reside com exclusividade sob a árvore de dados do Dev Drive ReFS: `Z:\souls_engine\.souls_data\`.

## 6. ROTEIRO METODOLÓGICO SDD & DAG PARA O GEMINI 3.1 PRO

Quando o Gemini 3.1 Pro for acionado para orquestrar o transplante de um subsistema (por exemplo: "Submeter o subsistema de Memória e Grafo para Transplante"), ele deve executar deterministicamente o seguinte protocolo:
### Passo 1: Varredura de Código Cru e Levantamento de Call-Graph

Inspecionar todos os arquivos físicos do cluster em `_donor/`. Identificar dependências circulares ocultas, structs de dados compartilhadas e acoplamentos com o antigo runtime.
### Passo 2: Emissão do Pacote SDD (`docs/migration/sdd/<subsystem>/`)

Gerar no repositório os quatro artefatos canônicos:
- `PLAN.md`: Justificativa técnica, triagem dos arquivos (HEALTHY vs REPAIR vs BIOHAZARD) e estratégias de isolamento.
- `DESIGN.md`:
    - Novas structs e DTOs a serem criados em `souls_protocol`.
    - Assinaturas das funções e traits com tipos de retorno `Result<T, DomainedError>`.
    - Esquemas SQL/Arrow correspondentes conforme `SCHEMAS_DDL_MANIFEST.md`.
    - Mapeamento exato de quais ferramentas MCP do `MCP_TOOL_CONTRACTS.md` consomem este subsistema.
- `TASKS.md`: Relação sequencial de micro-tarefas (ex: `TASK-MEM-001: Mapear structs de nós LadybugDB em souls_protocol`). Cada tarefa deve conter:
    - Arquivo de origem em `_donor/`.
    - Arquivo de destino em `crates/<target_crate>/src/`.
    - Requisitos de higienização.
    - Critério de teste para validação.
- `DAG.md`: Grafo Acíclico Dirigido representando visualmente e textualmente as dependências de execução entre as tarefas:

```
[TASK-001: souls_protocol DTOs]
            │
            ▼
[TASK-002: SQLite STRICT DDL & Migrations]
            │
            ├──────────────────────────┐
            ▼                          ▼
[TASK-003: LadybugDB Petgraph]   [TASK-004: LanceDB Vector Store]
            │                          │
            └─────────────┬────────────┘
                          ▼
             [TASK-005: RRF Hybrid Fusion]
                          │
                          ▼
             [TASK-006: Chyros Daemon & Langevin]
                          │
                          ▼
             [TASK-007: Exposição nas Ferramentas MCP]
```

### Passo 3: Validação contra as Linhas Vermelhas

Revisar se o design não introduziu dependências circulares, proxies L7 ou frameworks gráficos.

## 7. PLAYBOOK DE EXECUÇÃO CIRÚRGICA PARA O GEMINI 3.8 FLASH

O Gemini 3.8 Flash atuará na ponta da linha de montagem, consumindo os pacotes SDD gerados pelo Pro. Para cada tarefa da DAG:

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                      FLUXO OPERACIONAL DO GEMINI 3.8 FLASH                  │
│                                                                             │
│  1. Ler a Task atual em TASKS.md e os contratos em DESIGN.md                │
│  2. Inspecionar o código fonte do arquivo em _donor/                        │
│  3. Criar ou atualizar o teste unitário (Fase RED)                          │
│  4. Transplantar o código para crates/<crate>/src/ com higienização:        │
│     ├── Substituir imports legados por dependências canônicas               │
│     ├── Forçar finais de linha LF (\n) e UTF-8 sem BOM                      │
│     ├── Eliminar blocos unsafe não autorizados                              │
│     └── Conectar erros ao thiserror da crate                                │
│  5. Executar verificação no terminal:                                       │
│     cargo check -p <crate_name>                                             │
│     cargo test -p <crate_name> -- <test_name>                               │
│  6. Se verde (Fase GREEN):                                                  │
│     Marcar Task como [X] CONCLUÍDA em TASKS.md                              │
│  7. Se falhar: iterar na correção pontual (Regra de Crate Única)            │
└─────────────────────────────────────────────────────────────────────────────┘
```

## 8. PROTOCOLO DE EVOLUÇÃO TOPOLÓGICA (PROPOSTA DE NOVAS CRATES)

A topologia de 9 crates canônicas é a base congelada do projeto. No entanto, o Souls Engine adota o princípio da **Evolução Arquitetural Justificada**:

Se, durante a autópsia de um subsistema de alta densidade em `_donor/`, o Gemini 3.1 Pro constatar que manter certos módulos dentro de uma crate existente provocará inchaço estrutural (_Context Rot_), lentidão crônica de compilação ou violação de responsabilidade única, ele **pode propor uma nova crate** (ex: criar `souls_sandbox` para isolar a ferramenta `execute`, ou `souls_sast` para separar os 11 linters da `souls_anthropophagy`).
### Regras para Criação de Nova Crate:

1. **Emissão Obrigatória de Mini-ADR:** O Gemini 3.1 Pro deve gerar um documento sucinto em `docs/decisions/adrs/` fundamentando:
    - Por que a nova crate é estritamente necessária.
    - Por que o código não deve residir nas crates existentes.
    - O impacto no grafo de dependências do workspace (garantindo ausência de ciclos).
2. **Aprovação do Operador Humano:** A criação de uma décima crate depende de autorização explícita do Arquiteto Humano antes de ser adicionada ao `Cargo.toml` raiz.
3. **Respeito aos Níveis Arquiteturais:** A nova crate deve se encaixar em um nível claro da matriz de importações (Nível 1, 2 ou 3) e respeitar o isolamento absoluto de `souls_protocol`.

## 9. PLANO DE UNIFICAÇÃO DO `Cargo.toml` DO WORKSPACE

Para erradicar conflitos de versões e acelerar drasticamente o tempo de compilação incremental no Windows 11 MSVC, todas as dependências das antigas crates `souls_mc_*` serão consolidadas centralizadamente no arquivo raiz `Z:\souls_engine\Cargo.toml`:

```
[workspace]
resolver = "2"
members = [
    "crates/souls_protocol",
    "crates/souls_core",
    "crates/souls_ast",
    "crates/souls_memory",
    "crates/souls_inference_runtime",
    "crates/souls_model_router",
    "crates/souls_llm_local_arena",
    "crates/souls_anthropophagy",
    "crates/souls_server",
]

[workspace.dependencies]
# Runtimes Assíncronos e Concorrência
tokio = { version = "1.40", features = ["full"] }
tokio-util = { version = "0.7" }

# Serialização e Protocolos
serde = { version = "1.0", features = ["derive"] }
serde_json = { version = "1.0" }
arrow = { version = "53.0" }

# Persistência Relacional e Vetorial
sqlx = { version = "0.8", default-features = false, features = ["sqlite", "runtime-tokio-rustls", "chrono"] }
lancedb = { version = "0.10" }
petgraph = { version = "0.6" }

# Análise de Código e AST
tree-sitter = "0.22"
tree-sitter-rust = "0.21"
tree-sitter-python = "0.21"
tree-sitter-typescript = "0.21"
tree-sitter-go = "0.21"
similar = { version = "2.6", features = ["text"] }
gix = { version = "0.66", default-features = false, features = ["status", "revision", "parallel"] }

# Silício Local, Hardware e Inferência
memmap2 = "0.9"
windows-sys = { version = "0.59", features = ["Win32_System_Threading", "Win32_Storage_FileSystem", "Win32_Foundation"] }
nvml-wrapper = "0.10"
ort = { version = "2.0.0-alpha", default-features = false, features = ["download-binaries"] }
llguidance = { version = "0.6" }

# Modelagem FinOps e Matemática
rand = "0.8"
rand_distr = "0.4"
chrono = { version = "0.4", features = ["serde"] }

# Tratamento de Erros e Observabilidade
thiserror = "1.0"
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }
tracing-appender = "0.2"

# Rede e Servidor Headless
axum = { version = "0.7", features = ["macros"] }
tower-http = { version = "0.5", features = ["cors", "trace"] }
reqwest = { version = "0.12", default-features = false, features = ["rustls-tls", "json"] }
```

## 10. CRONOGRAMA DE DISSECAÇÃO E ORDEM SUGERIDA DE TRANSPLANTE

Para maximizar a estabilidade da fábrica, o transplante dos clusters de `_donor/` deve seguir uma ordem estritamente bottom-up (das fundações para as aplicações):

1. **Onda 1: Fundação Pura (`souls_protocol`):**
    - Migrar todos os DTOs de telemetria, contratos MCP, estruturas LEAN e enums epistêmicos. Zero dependências de I/O.
2. **Onda 2: Infraestrutura Bare-Metal (`souls_core`):**
    - Migrar filtros ANSI, buffers de desidratação MPSC, rotinas de I/O em ReFS e salvaguardas de pânico.
3. **Onda 3: Inteligência Sintática (`souls_ast`):**
    - Migrar Tree-sitter, outlines desidratados, extrator de slices, algoritmo Myers Diff e Frecency `gitoxide`.
4. **Onda 4: Persistência e Tríade L3 (`souls_memory`):**
    - Migrar DDL STRICT do SQLite, LanceDB colunar, Grafo LadybugDB, fusão RRF e o Chyros Daemon (Langevin Decay).
5. **Onda 5: Silício Local e Watchdog (`souls_inference_runtime`):**
    - Migrar sensor térmico NVML, EWMA preditivo, parser GGUF $\mathcal{O}(1)$ via `memmap2`, Response Healing e drivers ONNX/llama.cpp.
6. **Onda 6: FinOps e Otimização Bayesiana (`souls_model_router`):**
    - Migrar o ParetoBandit com barreira termodinâmica $\Phi$ e métrica $E^3$ para subtarefas.
7. **Onda 7: Benchmarking Local (`souls_llm_local_arena`):**
    - Migrar suítes de estresse de hardware, medidores de tokens/s e validadores de gramática via `llguidance`.
8. **Onda 8: Engenharia Reversa e Dissecação (`souls_anthropophagy`):**
    - Consolidar os 11 linters SAST, verificador SPDX, Organ Slicer e Organ Vault no SQLite.
9. **Onda 9: Daemon Executável Headless (`souls_server`):**
    - Montar rotas Axum em `127.0.0.1:9123`, ganchos SSE do MCP, endpoints REST do Thin Adapter e o protocolo Graceful Shutdown de 5 fases.