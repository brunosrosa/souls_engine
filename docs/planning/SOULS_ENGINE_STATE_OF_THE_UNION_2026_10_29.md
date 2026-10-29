# Diagnóstico Canônico e Estado da União do Souls Engine v7

> **ESTATUS DO DOCUMENTO:** SSOT DE ALINHAMENTO ESTRATÉGICO & REFERÊNCIA PARA GEMINI NOTEBOOK  
> **DATA DE RATIFICAÇÃO:** 29 de Outubro de 2026 (Linha de Base v7 Canônica)  
> **AUTORES:** Engenheiro Principal Bare-Metal & Arquiteto Humano  
> **ESPECIFICAÇÕES REGULADORAS:** [AGENTS.md](file:///Z:/souls_engine/AGENTS.md), [CRATES_SPECIFICATION.md](file:///Z:/souls_engine/docs/architecture/CRATES_SPECIFICATION.md), [ADR-001](file:///Z:/souls_engine/docs/decisions/adrs/ADR-001-Transplante-Morfologico-e-Topologia-Crates.md) a [ADR-010](file:///Z:/souls_engine/docs/decisions/adrs/ADR-010-Evolucao-Topologica-11-Crates-e-Ponte-Memoria-Adaptativa.md).

---

## 1. CABEÇALHO EXECUTIVO: A LINHA DO TEMPO (ANTES $\rightarrow$ AGORA $\rightarrow$ DEPOIS)

O **Souls Engine** atingiu a sua maturidade estrutural bare-metal após uma evolução profunda de arquitetura. O projeto deixou definitivamente para trás as premissas experimentais e monolíticas do passado para se consolidar como uma infraestrutura headless de alta performance em simbiose com o **Hermes Agent**.

```
┌─────────────────────────────────────────────────────────────────────────────────────────────┐
│                             EVOLUÇÃO TEMPORAL DO SOULS ENGINE                               │
├───────────────────────────────┬───────────────────────────────┬─────────────────────────────┤
│      ERA PASSADA (HISTÓRICO)  │       ERA ATUAL (SSOT v7)     │    PRÓXIMO HORIZONTE        │
│    "Souls MC / SODA / Genesis"│      "Souls Engine Canônico"  │    "Autonomia & Expansão"   │
├───────────────────────────────┼───────────────────────────────┼─────────────────────────────┤
│ • Desktop UI local (Tauri)    │ • 100% Headless Daemon        │ • Refatoração da 11ª crate  │
│ • Janelas Winit/Svelte/Systray│ • Axum TCP Loopback :9123     │   (souls_anthropophagy)     │
│ • Proxy L7 interceptador :3001│ • Hermes fala direto com cloud│ • Ciclo /learn fechado com  │
│ • Poluição em %APPDATA%       │ • Zero poluição (Z:\ ReFS)    │   avaliação ELO/EMA nativa  │
│ • Monólito frágil             │ • 11 Crates modulares no DAG  │ • Voos clínicos em produção │
│ • 0 MB VRAM ignorada          │ • WDDM 5.2GB + ASUS ATKACPI   │   com long-context (1M ctx) │
└───────────────────────────────┴───────────────────────────────┴─────────────────────────────┘
```

### O Que Era (Legado / Inspiração)
- **Souls MC / SODA:** Uma aplicação desktop híbrida baseada em Rust + Tauri + Svelte, que tentava gerenciar janelas visuais, ícones de bandeja do sistema (*systray*) e intervir no tráfego de rede como um proxy reverso L7 (escutando na porta `:3001`). Essa abordagem sofria de instabilidade, corrupção de streaming SSE, quebra de *prefix caching* das LLMs e invasão de escopo no sistema operacional (`%APPDATA%`).

### O Que É Agora (Realidade Operacional Concreta)
- **Souls Engine v7:** Um daemon nativo em Rust Tokio, **100% headless**, operando em loopback local `127.0.0.1:9123` e confinado ao Dev Drive ReFS (`Z:\souls_engine`). 
- Atua como o **exoesqueleto de silício** e **hipocampo L3** do Hermes Agent (Python 3.11):
  - Hermes Agent orquestra o diálogo interativo e chama APIs de nuvem diretamente (sem interceptação).
  - Souls Engine gerencia o silício físico bare-metal (GPU NVIDIA RTX 2060m e CPU Intel Core i9-9980HK AVX2), as bases de memória de alta densidade (FrankenSQLite STRICT WAL, LanceDB e Grafo LadybugDB) e disponibiliza 55 ferramentas MCP através de um **Two-Stage Semantic Switchboard** com latência inferior a 1.7 ms.

### O Que Será (Próximo Horizonte)
- Finalização da cirurgia de transplante da última crate pendente: **`souls_anthropophagy`** (análise estática SAST poliglota em 11 linguagens e engenharia reversa de repositórios).
- Execução dos Voos Clínicos E2E (Voos 1 a 4) com o Hermes Agent plenamente integrado ao novo **Recall Adaptativo L3 ("Terceira Via")**.
- Integração da malha de auto-evolução contínua (`weevolve` com cálculo bayesiano de ELO) para sintetizar aprendizados de sessões.

---

## 2. RADIOGRAFIA FÍSICA E MATURIDADE DAS 11 CRATES CANÔNICAS

O workspace é estruturado em **exatamente 11 crates canônicas** em um Grafo Acíclico Dirigido (DAG) estrito, compilado nativamente via MSVC (`x86_64-pc-windows-msvc`):

| # | Crate | Arquivos `.rs` | Linhas de Código (LOC) | Bateria de Testes | Maturidade Operacional | Responsabilidade Primária |
|:---:|:---|:---:|:---:|:---:|:---:|:---|
| **1** | [`souls_protocol`](file:///Z:/souls_engine/crates/souls_protocol) | 6 | 1.045 | Conforme | 🟢 **100% Produção** | DTOs universais, contratos JSON-RPC 2.0, estruturas LEAN e erros de domínio. Zero dependências de I/O. |
| **2** | [`souls_core`](file:///Z:/souls_engine/crates/souls_core) | 11 | 2.779 | 65 / 65 Aprovados | 🟢 **100% Produção** | SubprocessGuard RAII, PiiRedactor reversível (Aho-Corasick), CoW ReFS, DedupCache SipHash e zero-dep HTML scraper. |
| **3** | [`souls_ast`](file:///Z:/souls_engine/crates/souls_ast) | 6 | 1.700 | Conforme | 🟢 **100% Produção** | Skeletons Tree-sitter (70-85% economia de tokens), Call Graph lock-free em RAM, Myers Diff e Gitoxide Frecency Heatmap. |
| **4** | [`souls_memory`](file:///Z:/souls_engine/crates/souls_memory) | 12 | 4.622 | 29 / 29 Aprovados | 🟢 **100% Produção** | FrankenSQLite STRICT WAL, LanceDB/Arrow em disco, Grafo Ontológico LadybugDB (`petgraph`), RRF Hybrid Search e Snapshot Congelado. |
| **5** | [`souls_inference_runtime`](file:///Z:/souls_engine/crates/souls_inference_runtime) | 9 | 3.944 | 37 / 37 Aprovados | 🟢 **100% Produção** | NVML C-FFI Watchdog (teto 5.2GB WDDM), ASUS ATKACPI kernel ioctl (CPU/GPU fan tachometer), llama.cpp CUDA, Shannon Entropy e Sheaf Cohomology. |
| **6** | [`souls_model_router`](file:///Z:/souls_engine/crates/souls_model_router) | 8 | 1.465 | 11 / 11 Aprovados | 🟢 **100% Produção** | Thompson Sampling multiobjetivo (ParetoBanditRouter), Barreira Térmica $\Phi$, Métrica $E^3$ de eficiência e recomendação de esforço CoT. |
| **7** | [`souls_llm_local_arena`](file:///Z:/souls_engine/crates/souls_llm_local_arena) | 28 | 3.879 | Conforme | 🟢 **100% Produção** | 5 suítes de benchmark em silício (Hardware, Cognitive, Vision, Speculative, Pressure), 14+ modelos locais e 137 ensaios reais em banco. |
| **8** | [`souls_llm_cloud_arena`](file:///Z:/souls_engine/crates/souls_llm_cloud_arena) | 24 | 2.803 | Conforme | 🟢 **100% Produção** | Ingestão de mercado (Nous Portal 335 modelos, OpenRouter, AA/LMSYS), 9 Perfis Cognitivos (ADR-009) e Tríades de Pareto. |
| **9** | [`souls_llm_inventory`](file:///Z:/souls_engine/crates/souls_llm_inventory) | 11 | 2.136 | 17 / 17 Aprovados | 🟢 **100% Produção** | Profiler de Hardware NT (`GlobalMemoryStatusEx`), Downloader sequencial HTTP Range com SHA-256 incremental e bundles VLM (`mmproj`). |
| **10** | [`souls_server`](file:///Z:/souls_engine/crates/souls_server) | 23 | 11.435 | 35 / 35 Aprovados | 🟢 **100% Produção** | Daemon headless Axum `:9123`, Two-Stage Switchboard (<1.7ms), Gateway OpenAI `/v1/chat/completions`, Endpoints REST de Memória e Web Dashboard SPA. |
| **11** | [`souls_anthropophagy`](file:///Z:/souls_engine/crates/souls_anthropophagy) | 1 | 12 | Stub | 🟡 **Planejada / Transplante** | Dissecação profunda de código, 11 analisadores SAST poliglotas e destilação de órgãos sintáticos (a transplantar de `_donor/`). |
| **TOTAL** | **11 Crates** | **139** | **35.820 LOC** | **194+ Testes Unit/E2E** | **91% Concluído (10/11 Crates em Produção)** | |

---

## 3. AUDITORIA FORENSE DE `_donor/` (O QUE JÁ FOI TRANSPLANTADO, O QUE DELETAR E O QUE MANTER)

A pasta `_donor/` preserva o histórico do antigo monólito `Souls MC`. A análise forense abaixo estabelece o destino de cada subsistema:

### 3.1 Artefatos Já Transplantados com Sucesso (100% Absorvidos)
- **Protocolo & DTOs:** `_donor/souls_mc_protocol/` $\rightarrow$ 100% substituído por `crates/souls_protocol/`.
- **Serviços de Segurança e Sandbox:** `_donor/souls_mc_core/src/core/security/pii_redactor.rs`, `sandbox.rs`, `subprocess_guard.rs` $\rightarrow$ Refatorados e expandidos em `crates/souls_core/`.
- **Análise Sintática e Myers Diff:** `_donor/souls_mc_core/src/cognition/ast/` $\rightarrow$ 100% reescrito em `crates/souls_ast/` com suporte a tree-sitter moderno e gitoxide.
- **Hipocampo e Persistência:** `_donor/souls_mc_core/src/cognition/memory/` $\rightarrow$ Absorvido e expandido em `crates/souls_memory/` com SQLite STRICT WAL e Arrow LanceDB.
- **Silício & NVML:** `_donor/souls_mc_core/src/core/vram_hardware/` $\rightarrow$ Consolidado em `crates/souls_inference_runtime/` com ioctl kernel ASUS e teto de 5.2GB WDDM.
- **Roteamento & FinOps:** `_donor/souls_mc_core/src/finops/` $\rightarrow$ Reestruturado em `crates/souls_model_router/` e `crates/souls_llm_cloud_arena/`.
- **Cohomologia Socrática:** `_donor/souls_mc_core/src/core/socratic/cohomology.rs` $\rightarrow$ Implementado na matriz GF(2) em `crates/souls_inference_runtime/src/cohomology.rs`.

### 3.2 O Que Pode Ser DELETADO com Segurança Imediata (Resíduo Necrótico)
Estes arquivos violam as Leis Negativas do [AGENTS.md](file:///Z:/souls_engine/AGENTS.md) ou são substitutos obsoletos:

1. **Infraestrutura do Proxy L7 Interceptador (Linha Vermelha #2):**
   - `_donor/boot.ps1` (script de subida do antigo proxy e gateway).
   - `_donor/config/souls-gateway.jsonc` (configuração da porta `:3001`).
   - `_donor/resources/specs/agentgateway.yaml` (especificação OpenAPI do proxy L7).
   - `_donor/souls_mc_core/src/bin/agentgateway_tcp_proxy.rs`.
   - `_donor/souls_mc_core/src/core/security/l7_shield.rs`.
2. **Crate de Protocolo Legada Inteira:**
   - Pasta `_donor/souls_mc_protocol/` inteira (já portada para `crates/souls_protocol`).
3. **Binários Obsoletos e Servidores Legados:**
   - `_donor/souls_mc_core/src/bin/mcp_stdio_guard.rs`.
   - `_donor/souls_mc_core/src/bin/soda_mcp_tester_cli.rs`.
   - `_donor/souls_mc_core/src/bin/souls_arena_cli.rs`.
   - `_donor/souls_mc_core/src/bin/souls_vanguard_worker.rs`.
   - `_donor/souls_mc_core/src/bin/souls_mcp_server/` (servidor stdio arcaico).
4. **Relíquias de UI Desktop / Terminal Interativo (Linha Vermelha #1):**
   - `_donor/souls_mc_core/src/core/socratic/terminal_drawer_stream.rs`.
   - `_donor/souls_mc_core/src/core/socratic/socratic_cli.rs`.
   - `_donor/souls_mc_core/src/core/socratic/socratic_thought_stream.rs`.
5. **Subsistemas Antigos Já Reimplementados no v7:**
   - `_donor/souls_mc_core/src/core/vram_hardware/` (já em `souls_inference_runtime`).
   - `_donor/souls_mc_core/src/core/governance/` (`gateway_config.rs`, `ipc_bridge.rs`, `sticky_router.rs`, `mcp_transport.rs`, `semantic_search.rs`).
   - `_donor/souls_mc_core/src/cognition/ast/`, `cognition/context/`, `cognition/memory/`, `cognition/state_thinking/`, `finops/`.

### 3.3 O Que DEVE SER MANTIDO até a Conclusão do Transplante
Estes arquivos contêm a inteligência técnica que será transladada para `crates/souls_anthropophagy` ou fases futuras:

1. **Pasta Completa `_donor/souls_mc_anthropophagy/` (~1.5 MB de código de alto valor):**
   - **11 Analisadores SAST:** `harvester/sast/{bandit, biome, clippy, cppcheck, govulncheck, native_ast, opengrep, oxc, ruff, sobelow}.rs`.
   - **Motor de Extração & Inventário:** `harvester/{extract.rs, detect.rs, canon.rs, repo_radar.rs, github_tracker.rs}`.
   - **Destilação & Síntese:** Pastas `distillation/`, `synthesizer.rs` e `swarm.rs`.
2. **`_donor/souls_mc_core/src/harvester/`:**
   - Manter temporariamente para comparar a fusão com `_donor/souls_mc_anthropophagy/src/harvester/` (`ast_parser.rs`, `heuristic_vault.rs`, `repo_radar.rs`) conforme o mapa de autópsia.
3. **`_donor/resources/wasm_grammars/` (`tree_sitter_rust.wasm`, `python.wasm`, `outline_parser.wasm`):**
   - Manter para eventual execução isolada de parsers via Wasmtime em código de terceiros não-confiável.
4. **Arquivos de Referência Avançada em `souls_mc_core` (Preservar como Biblioteca Técnica):**
   - `_donor/souls_mc_core/src/cognition/learning/weevolve.rs` e `ratings.rs` (cálculo de ratings bayesianos para o ciclo de auto-aprendizado `/learn`).
   - `_donor/souls_mc_core/src/core/inference/bitnet_daemon.rs` e `bitnet_engine.rs` (inferência de 1-bit em CPU AVX2 para quando o Tier 0.5 for expandido).
   - `_donor/souls_mc_core/src/core/governance/response_healing.rs` (heurísticas de reparo de JSON corrompido para SLMs menores).

---

## 4. INVENTÁRIO DE RECURSOS ENTREGUES E HOMOLOGADOS

### 4.1 Gestão de Silício Bare-Metal e Sensores Reais (Zero Mocks)
- **NVIDIA GeForce RTX 2060m:** Leitura em tempo real via NVML C-FFI oficial (`nvmlDeviceGetMemoryInfo`, temperatura, clock, P-State e VRAM). Teto útil rigoroso de **5.294 MB úteis sob o WDDM** do Windows 11.
- **Intel Core i9-9980HK (8C/16T, AVX2):** Resolução dinâmica do nome do processador via Registro NT (`HARDWARE\DESCRIPTION\System\CentralProcessor\0`).
- **ASUS ATKACPI Kernel Ioctl Driver:** Leitura direta de hardware via `\\.\ATKACPI` (`DeviceIoControl` control code `0x0022240C`). Diagnostica fielmente a velocidade das ventoinhas em RPM:
  - **CPU Fan:** ~2.200 RPM (ativo e monitorado).
  - **GPU Fan:** 0 RPM (ventilador nativo inativo, resfriamento real assegurado por base exaustora externa dedicada).
- **Semáforo `GPU_EXECUTION_LOCK`:** Exclusão mútua garantida na dGPU para impedir concorrência entre SLM local e processos gráficos.

### 4.2 Catálogo MCP (55 Ferramentas em 6 Domínios) & Two-Stage Switchboard
- Catálogo MCP completo servido via Streamable HTTP/SSE em `127.0.0.1:9123/mcp`.
- **Two-Stage Semantic Switchboard:** Roteador semântico de intenções em dois estágios rodando em <1.7 ms na CPU Host com instruções AVX2. Elimina a necessidade de injetar 55 esquemas JSON no prompt do modelo principal, poupando **4.000 a 6.000 tokens de contexto por turno**.
- **100% de Homologação Clínica E2E:** 156 cenários de estresse executados com 0.0% de pânico e 100% de contenção contra path traversal fora do ReFS (`RefsPathLeakViolation`).

### 4.3 Ponte Bidirecional de Memória L3 & "Terceira Via" (TDAH + AH/SD)
- **Arquitetura Não-Invasiva:** Respeita a soberania de escrita do Hermes Agent sobre `MEMORY.md`, `USER.md` e `SOUL.md`. O Souls Engine faz cold-boot passivo (`HermesBootstrapFiles`).
- **`FrozenMemorySnapshot`:** Congela uma visão estática da memória na RAM por `session_id`, garantindo que o chat interativo mantenha **>95% de retenção de prefix cache**.
- **Recall Adaptativo ("A Terceira Via"):**
  - **Modo Enxuto (`lean`):** Injeta apenas diretrizes vitais e fatos consolidados (~800 caracteres), preservando a atenção executiva do operador sem sobrecarga.
  - **Modo Expandido (`expanded`):** Expande arestas causais e entidades do Grafo Ontológico LadybugDB (~2.400 caracteres), possibilitando exploração profunda sob demanda.
- **`SoulsMemoryProvider`:** Adapter nativo em Python (`@hermes/plugin-sdk`) conectando o Hermes Agent ao endpoint `/api/v1/memory/recall` com fallback transparente caso o daemon esteja indisponível.

### 4.4 Arenas Gêmeas de Benchmarking & Model Hub (RFC-007)
- **Silicon Local Arena (`souls_llm_local_arena`):** 5 baterias de ensaios padronizados (ADR-007) com 137 ensaios empíricos reais persistidos em `souls_heuristic_vault.db`. Mapeamento de 14+ modelos GGUF locais e sincronização via HardLinks/Junctions para a pasta do Hermes.
- **Cloud Arena (`souls_llm_cloud_arena`):** Ingestão contínua de 335 modelos via Nous Portal, OpenRouter e Artificial Analysis. Consolidação dos 9 Perfis Cognitivos canônicos (ADR-009) e cálculo contínuo das Tríades de Pareto (`Quality Leader`, `Best Value`, `Ultra Low Cost`).
- **Inventory & Downloader (`souls_llm_inventory`):** Profiler de compatibilidade contra a RAM NT e VRAM NVML, com download HTTP sequencial com cabeçalhos Range, gravação em `.part` e validação SHA-256.

### 4.5 Curadoria do Cofre Pessoal Obsidian & Privacy Shield
- **Curadoria `raw/` $\rightarrow$ `wiki/`:** O cofre Obsidian pessoal (`C:\Users\rosas\OneDrive\Documentos\Obisidian DB`) opera sob o princípio da imutabilidade dos dados brutos (`raw/` somente leitura). A extração para artigos destilados em `wiki/` é orquestrada por `tools/curate_obsidian.ps1` com catálogo SHA-256 (`raw_manifest.json`) e log compulsório em `wiki/log.md`.
- **Privacy Shield:** Sanitização local via `PiiRedactor` bare-metal antes de qualquer análise de dados clínicos ou financeiros por modelos externos.

### 4.6 Web Dashboard Headless & Plugin REST
- Dashboard SPA de alta densidade servido nativamente pelo Axum em `http://127.0.0.1:9123/dashboard`.
- Aba 1 (**Overview & Integrador**) totalmente integrada a sensores reais: telemetria de hardware com RPMs reais, telemetria FinOps ($E^3$) agregada do banco com filtros temporalizados (Dia, Mês, Ano), estado do Recall L3 e modal interativo do [AGENTS.md](file:///Z:/souls_engine/AGENTS.md).
- Plugin oficial do Hermes Agent (`adapters/hermes_dashboard_souls/`) expondo 5 superfícies visuais integradas no Web Dashboard oficial do Hermes.

---

## 5. MAPA DA DOCUMENTAÇÃO (`docs/`) & SSOT PARA O GEMINI NOTEBOOK

Para alimentar o **Gemini Notebook (NotebookLM / Gemini Gem)** com fontes de verdade atualizadas e livres de anacronismos, utilize a seguinte priorização:

```
docs/
├── decisions/adrs/
│   ├── ADR-001 ao ADR-006 (Fundações v7, ReFS, Single-Instance, Axum :9123)
│   ├── ADR-007 (5 Suítes de Benchmark Padronizadas no Silício Local)
│   ├── ADR-008 (Matriz de Custos da Cloud Arena e Nous Portal Ingestion)
│   ├── ADR-009 (Taxonomia Cognitiva Unificada de 9 Facetas e Master Brain)
│   └── ADR-010 (Consolidação Topológica 11 Crates & Ponte de Memória Adaptativa)  <-- [NOVO]
├── architecture/
│   ├── CRATES_SPECIFICATION.md (Matriz de Fluxo e Regras das 11 Crates)          <-- [ATUALIZADO]
│   └── LESSONS_LEARNED_AND_POSTMORTEM.md (Caderno de Cascas de Bala e Zombie)
├── contracts/
│   ├── MCP_TOOL_CONTRACTS.md (Catálogo e Schemas das 55 Ferramentas MCP)
│   └── MEMORY_BRIDGE_SPECIFICATION.md (Ponte L3, Snapshots e Terceira Via)
├── homologation/
│   └── SOULS_ENGINE_E2E_HOMOLOGATION_REPORT.md (Homologação dos 156 Cenários E2E)
└── planning/
    ├── SOULS_ENGINE_STATE_OF_THE_UNION_2026_10_29.md (Este Diagnóstico Canônico) <-- [SSOT ATUAL]
    └── roadmap/SOULS_ENGINE_ROADMAP_NEXT_DELIVERIES.md (Plano de Voo das Fases)
```

> **Alerta de Anacronismo:** Documentos antigos que citem "9 crates" ou "10 crates" foram formalmente superados pelo **ADR-010** e pela versão atual do **CRATES_SPECIFICATION.md**, que estabelecem a topologia canônica de **11 crates**.

---

## 6. VISÃO DE FUTURO & PLANO GERAL DOS PRÓXIMOS PASSOS (BACKLOG PRIORIZADO)

### Fase 1: Auditoria Fina do Web Dashboard (Imediato)
- Finalizar a inspeção elemento-a-elemento das abas restantes no Web Dashboard (`http://127.0.0.1:9123/dashboard`):
  - **Aba 2: Local Models & Silício Local:** Verificar cards de modelos GGUF, métricas de benchmark da Local Arena e links físicos.
  - **Aba 3: Cloud Arena & Nous Portal:** Validar tabela de 335 modelos, ranking dos 9 Perfis Cognitivos e chaveamento de provedores.
  - **Aba 4: Governança, Configurações & Presets:** Validar edição de `souls_config.toml` e toggles do ParetoBandit.

### Fase 2: Voos de Teste Ponta-a-Ponta com Hermes Agent Vivo (Curto Prazo)
- **Voo 1 (Cold Boot & Estabilidade Básica):** Subida conjunta do Hermes Agent e `souls_server.exe` validando leitura passiva de memória e `GET /health`.
- **Voo 2 (Prefix Caching & Recall L3):** Diálogo longo com Hermes atestando estabilidade do `FrozenMemorySnapshot` e alternância entre os modos Enxuto e Expandido.
- **Voo 3 (Despacho de Subagentes & FinOps):** Delegação de subtarefa via `route_subtask` com medição de entropia e economia de tokens.
- **Voo 4 (Curadoria do Cofre Obsidian via Hermes):** Execução do fluxo `raw/` $\rightarrow$ `wiki/` disparado pelo agente via script `curate_obsidian.ps1`.

### Fase 3: Transplante Canônico de `souls_anthropophagy` (Médio Prazo)
- Transladar o código de `_donor/souls_mc_anthropophagy` e `_donor/souls_mc_core/src/harvester/` para a crate `crates/souls_anthropophagy`.
- Implementar sob metodologia SDD/TDD os 11 runners de SAST em Rust/C/Python/JS e a destilação de repositórios em formato de grafo.
- Fechar o ciclo de auto-aprendizado `/learn` integrando o motor bayesiano ELO/EMA de `weevolve.rs`.
