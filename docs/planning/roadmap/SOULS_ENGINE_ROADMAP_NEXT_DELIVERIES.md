# SOULS_ENGINE_ROADMAP_NEXT_DELIVERIES.md — Roadmap Canônico de Entregas & Pendências Técnicas (v7)

> **ESTATUS DO DOCUMENTO:** CANÔNICO & VINCULANTE (DOCS-AS-GUARDRAILS)  
> **DATA DE CONSOLIDAÇÃO:** 2026-10-11  
> **INVENTÁRIO ATIVO HOMOLOGADO:** 15 Modelos de Elite (32,57 GB em `C:\Users\rosas\llm_local_models\`)  
> **BANCO RELACIONAL SSOT:** `Z:\souls_engine\.souls_data\db\souls_heuristic_vault.db`  

---

## 1. VISÃO GERAL DA ARQUITETURA SIMBIÓTICA

O **Souls Engine v7** opera como uma infraestrutura bare-metal headless (Rust/Tokio) em loopback TCP `127.0.0.1:9123`, atuando como acelerador de silício, motor de AST e hipocampo de memória L3 para o **Hermes Agent** (Python 3.11).

```mermaid
flowchart TD
    subgraph HermesAgent ["Hermes Agent (Desktop / Python 3.11)"]
        MasterChat["Chat Interativo Mestre (Nuvem Direta)"]
        Delegator["Orquestrador de Subagentes (delegate_task)"]
    end

    subgraph SoulsEngineDaemon ["Souls Engine Daemon (:9123 Bare-Metal)"]
        MCPTrans["Transporte MCP (JSON-RPC 2.0 / SSE Loopback)"]
        Router["ParetoBandit & model_task_profiles"]
        Sessions["Gerenciador de Sessões (handoff / sub_agent)"]
        Inference["souls_inference_runtime (llama.cpp / NVML Watchdog)"]
        Memory["souls_memory (SQLite STRICT / LanceDB)"]
    end

    MasterChat -->|SSE Direto sem Proxy L7| CloudAPI["Anthropic / Nous Portal"]
    Delegator -->|1. Consulta souls_route_subtask| MCPTrans
    MCPTrans --> Router
    Router -->|Retorna Modelo Ótimo & Config| Delegator
    Delegator -->|2. Inicia Subagente com Modelo Prescrito| Sessions
    Delegator -->|3. Inferência Local /v1/chat/completions| Inference
    Sessions --> Memory
```

---

## 2. MATRIZ DE PENDÊNCIAS, ENTREGAS & MAPA DE TRANSPLANTE (`_donor/`)

### Bloco A: `souls_llm_cloud_arena` (Benchmarking e Rastreio de APIs de Nuvem)
* **Status Atual:** 100% Concluído e Homologado (10 crates canônicas, 9 testes unitários/integração na crate e 21 no souls_server).
* **Fontes Canônicas no `_donor/`:**
  * `_donor/souls_mc_core/src/finops/iron_cost.rs`: Lógica de precificação ponderada e conversão de tokens.
  * `_donor/souls_mc_protocol/src/metrics.rs`: DTOs de telemetria e custo por milhão de tokens.
* **Entregas Canônicas da Sessão 5:**
  1. **Topologia:** Criação formal da 10ª crate do workspace: `crates/souls_llm_cloud_arena` (`#![forbid(unsafe_code)]`).
  2. **Ingestores de Mercado:** Nous Portal API (`/v1/models`), OpenRouter API (`/api/v1/models` com modo público anônimo e suporte a token) e ingestão de benchmarks cognitivos consolidados (LMSYS Arena Elo, SWE-bench, BFCL, IFEval, GPQA, LiveBench).
  3. **A Taxonomia Cognitiva Unificada (ADR-009):**
     * **Executive / Master Brain ($S_{\text{executive}}$):** Modelo de fronteira e voz mestre do Hermes. 0.30 Hard Elo + 0.25 IFEval + 0.20 BFCL + 0.15 GPQA + 0.10 Context Coherence.
     * **Coding & SWE ($S_{\text{coding}}$):** 0.40 Coding + 0.25 BFCL + 0.20 Hard Elo + 0.15 IFEval.
     * **Autonomous Agentic ($S_{\text{agentic}}$):** 0.45 BFCL + 0.30 IFEval + 0.15 Hard Elo + 0.10 TPS.
     * **Deep Reasoning ($S_{\text{reasoning}}$):** 0.45 GPQA + 0.35 Hard Elo + 0.20 IFEval.
     * **Fast / Triage ($S_{\text{fast}}$):** 0.40 TPS Norm + 0.30 TTFT Norm + 0.15 IFEval + 0.15 Coding Elo.
     * **Design & Multimodal UI ($S_{\text{design}}$):** 0.40 Frontend/Visual SWE + 0.30 VQA/Multimodal + 0.15 IFEval + 0.15 Throughput.
     * **Facetas Especializadas de Suporte:** `Research & Deep Synthesis` ($S_{\text{research}}$), `Security & Code Audit` ($S_{\text{security}}$) e `Technical Writing & Obsidian Curator` ($S_{\text{curation}}$).
  4. **Ciclo de Atualização e Resiliência:** Sincronização diária assíncrona em segundo plano na inicialização do daemon `souls_server` (se `last_update < today`), com fallback gracioso mantendo cache local se houver falha de rede + ferramenta MCP `cloud_arena_sync` e endpoint REST `/api/v1/cloud/champions`.
  5. **Persistência SSOT:** Tabelas SQLite STRICT WAL em `Z:\souls_engine\.souls_data\db\souls_heuristic_vault.db` (`canonical_models`, `model_aliases`, `pricing_telemetry_snapshots`, `benchmark_snapshots`, `daily_evaluations`).
  6. **Integração e Visualização Futura (Sessão 7):** As recomendações do Master Brain e Campeões de Nuvem serão consumidas visualmente no Web Dashboard do Hermes (`hermes dashboard`) e como fonte SSOT para os arquivos de configuração do Hermes Agent.

---

### Bloco B: `souls_server` (O Daemon Bare-Metal Headless & Servidor MCP)
* **Status Atual:** Topologia definida em `CRATES_SPECIFICATION.md` e contratos estabelecidos em `MCP_TOOL_CONTRACTS.md`.
* **Fontes Canônicas no `_donor/`:**
  * `_donor/souls_mc_core/src/bin/souls_mcp_server/`: Despachante JSON-RPC 2.0 e isolamento stdio contra ANSI.
  * `_donor/souls_mc_core/src/core/governance/mcp_transport.rs`: Envelopes canônicos e ganchos de stream SSE.
* **Entregas Pendentes:**
  1. **Transporte SSE Loopback (`127.0.0.1:9123`):** Servidor HTTP/SSE nativo em Tokio/Axum expondo o catálogo JSON-RPC 2.0 sem dependências de interface gráfica (zero Tauri, zero systray).
  2. **Implementação dos Endpoints MCP Canônicos (30 Ferramentas):**
     * `ops.route_subtask` (`souls_route_subtask`): Endpoint consultado pelo Hermes antes de acionar `delegate_task`.
     * `memory.handoff` & `memory.sub_agent`: Persistência de payloads de transição e controle de ciclo de vida de subagentes.
     * `ast.souls_ast_outline` & `ast.souls_ast_slice`: Desidratação de código (FinOps mandatório de economia de contexto).
     * `think.core_think` (`thinking`): Grafo Acíclico Dirigido (DAG) para cognição socrática multi-passo.
  3. **Gateway de Inferência Local (`/v1/chat/completions`):** Endpoint compatível com a API da OpenAI para que os subagentes delegados executem inferência local direcionada para o modelo físico ativo na GPU/CPU.

---

### Bloco C: Configurações & Alinhamento no Hermes Agent Desktop
* **Status Atual:** Diretórios e modelos físicos consolidados; integração de configuração pendente.
* **Entregas Pendentes:**
  1. **Configuração do `config.yaml` do Hermes:**
     * Registro do Souls Engine como servidor MCP com conexão local SSE:
       ```yaml
       mcp_servers:
         souls_engine:
           url: "http://127.0.0.1:9123/sse"
           transport: "sse"
       ```
     * Configuração do provider local `llama.cpp` apontando para a porta de loopback `:9123` com flags de aceleração física:
       `-ngl 99 -ctk f16 -ctv q4_0 -fa on -c 8192`
  2. **Unificação de Diretórios de Modelos:**
     * O Hermes Agent utiliza por padrão `%LOCALAPPDATA%\hermes\models`.
     * Criação de um *Junction Point* ReFS/NTFS transparente apontando `%LOCALAPPDATA%\hermes\models` $\to$ `C:\Users\rosas\llm_local_models` para evitar qualquer duplicação física de arquivos GGUF.
  3. **Injeção de Governança Agêntica (`souls-ops-finops`):**
     * Atualização do prompt de sistema do Hermes Agent condicionando o uso de `delegate_task` à consulta prévia ao `souls_route_subtask`.

---

### Bloco D: Transplante do Código Legado (`_donor/`) para as Crates Canônicas
* **Status Atual:** Mapeamento forense detalhado em `DONOR_EXTRACTION_MAP.md`.
* **Fontes Canônicas no `_donor/` por Crate:**
  1. **`crates/souls_memory`:**
     * `_donor/souls_mc_core/src/cognition/state_thinking/session_manager.rs`: Gerenciador de sessões concorrentes de subagentes, controle de bifurcação (`branchFromThought`) e fusão (`merge_sessions`).
     * `_donor/souls_mc_core/src/cognition/state_thinking/mpsc_bridge.rs`: Ponte multithread assíncrona não-bloqueante entre o servidor MCP e o banco de dados.
     * `_donor/souls_mc_core/src/cognition/memory/`: FrankenSQLite STRICT WAL (`souls_state.db`), LanceDB vetorial e Chyros Daemon (Langevin Decay).
  2. **`crates/souls_model_router`:**
     * `_donor/souls_mc_core/src/finops/pareto_bandit.rs`: Otimizador bayesiano multiobjetivo Thompson Sampling e cálculo da fronteira de Pareto com barreira térmica $\Phi$.
     * `_donor/souls_mc_core/src/finops/finops_router.rs`: Roteamento determinístico por perfil e cálculo de fallback chains.
  3. **`crates/souls_ast`:**
     * `_donor/souls_mc_core/src/cognition/ast/`: Parser Tree-sitter multilinguagem, gerador de outlines de funções/structs e fatiamento semântico.
     * `_donor/souls_mc_core/src/cognition/context/myers_diff.rs`: Algoritmo matemático de diff cirúrgico via crate `similar`.
     * `_donor/souls_mc_core/src/cognition/context/repo_heatmap.rs`: Motor de frecency via `gitoxide`.
  4. **`crates/souls_inference_runtime`:**
     * `_donor/souls_mc_core/src/core/vram_hardware/hardware_watchdog.rs`: Monitor NVML bare-metal e cálculo de média móvel exponencial de pico (`peak_ewma.rs`).
     * `_donor/souls_mc_core/src/core/vram_hardware/headroom_engine.rs`: Garantia do teto útil de 5.2GB no Windows 11 WDDM (ADR-004).

---

## 3. ARQUITETURA DE CONTROLE DE SESSÕES DE SUBAGENTES

### 3.1 Divisão de Responsabilidades (Hermes vs. Souls Engine)
* **Papel do Hermes Agent (Orquestrador Executivo):**
  * O Hermes cria as threads de subagentes e gerencia a lógica da conversa no topo.
  * O Hermes solicita a recomendação de modelo ao Souls Engine e passa as mensagens.
* **Papel do Souls Engine (Infraestrutura Bare-Metal & Hipocampo L3):**
  * **Arbitragem de Silício:** Decide qual modelo deve atender a demanda baseado em VRAM, temperatura da RTX 2060m e afinidade cognitiva da tarefa.
  * **Persistência de Sessão & Handoff:** Armazena o estado intermediário e histórico do subagente em `sub_agent_sessions` e `handoff_payloads` no SQLite STRICT, impedindo que o subagente polua o contexto do modelo mestre.
  * **Telemetria e Avaliação Pós-Execução:** Registra a latência, tokens gerados e taxa de sucesso do subagente na tabela `arena_telemetry` para calibração contínua do algoritmo bayesiano.

---

## 4. ESTRATÉGIA DE SESSÕES DE TRABALHO & ORDEM CRÍTICA DE EXECUÇÃO

Para blindar o desenvolvimento contra o *Context Rot* e garantir feedback funcional imediato sem quebras colaterais, o trabalho deve ser decomposto em **sessões sequenciais atômicas**:

```
┌────────────────────────────────────────────────────────────────────────────────┐
│                   SEQUÊNCIA CANÔNICA DE SESSÕES DE TRABALHO                    │
├──────────┬───────────────────────┬─────────────────────────────────────────────┤
│ Sessão 1 │ Bloco B: Daemon Base  │ [CONCLUÍDA] souls_server.exe (:9123)        │
│          │                       │ Transporte Axum SSE loopback + health check │
├──────────┼───────────────────────┼─────────────────────────────────────────────┤
│ Sessão 2 │ Bloco D: Cérebro L3   │ [CONCLUÍDA] Cérebro souls_memory (SQLite    │
│          │                       │ STRICT WAL + LanceDB + LadybugDB)           │
├──────────┼───────────────────────┼─────────────────────────────────────────────┤
│ Sessão 3 │ Bloco C: Conexão Viva │ [CONCLUÍDA] Hermes Desktop config.yaml,     │
│          │                       │ SOUL.md FinOps, models junction & MCP live  │
├──────────┼───────────────────────┼─────────────────────────────────────────────┤
│ Sessão 4 │ Bloco D: AST & NVML   │ [CONCLUÍDA] Transplante souls_ast (outlines)│
│          │                       │ e souls_inference_runtime (NVML Watchdog)   │
├──────────┼───────────────────────┼─────────────────────────────────────────────┤
│ Sessão 5 │ Bloco A: Cloud Arena  │ [CONCLUÍDA] Ingestão Nous/OpenRouter,       │
│          │                       │ 9 perfis, fronteira Pareto e sync diário    │
├──────────┼───────────────────────┼─────────────────────────────────────────────┤
│ Sessão 6 │ Bloco E: Skills &     │ [CONCLUÍDA] 15 Skills sincronizadas no C:,  │
│          │ Governança Obsidian   │ esteira raw->wiki (498 docs) e ciclo /learn │
├──────────┼───────────────────────┼─────────────────────────────────────────────┤
│ Sessão 7 │ Bloco F: Web Dashboard│ [CONCLUÍDA] Hermes Web Dashboard (REST),    │
│          │ & Souls Settings      │ Souls Settings globais, Arenas e Branding   │
├──────────┼───────────────────────┼─────────────────────────────────────────────┤
│ Sessão 8 │ Homologação Clínica & │ [CONCLUÍDA] Homologação Clínica Completa:   │
│          │ Testes E2E Completos  │ 156/156 Cenários Aprovados (100%), 52 Tools,│
│          │                       │ 15 Skills, Zero VRAM Waste, ReFS Seguro     │
├──────────┼───────────────────────┼─────────────────────────────────────────────┤
│ Sessão 9 │ Bloco G: Silício      │ [CONCLUÍDA] 15 modelos físicos (32.57 GB)   │
│          │ Neural & Switchboard  │ conectados, Switchboard 2-Estágios (<1.7ms) │
├──────────┼───────────────────────┼─────────────────────────────────────────────┤
│ Sessão 10│ Bloco H: Des-staging  │ [CONCLUÍDA] 14 tools des-stagadas, sandbox, │
│          │ Ferramentas Domínio 6 │ LEAN compress/dedup/fill, 23/23 testes PASS │
├──────────┼───────────────────────┼─────────────────────────────────────────────┤
│ Sessão 11│ Bloco I: Skills &     │ [CONCLUÍDA] 15 Skills testadas (100% PASS), │
│          │ Curadoria Viva Cofre  │ esteira viva 498 notas Obsidian raw->wiki   │
├──────────┼───────────────────────┼─────────────────────────────────────────────┤
│ Sessão 12│ Bloco J: Fix Banco,   │ [CONCLUÍDA] FK 787 resolvida, telemetria SSE│
│          │ Dashboard SSE & Config│ viva, ordenação e souls_config.toml persist.│
├──────────┼───────────────────────┼─────────────────────────────────────────────┤
│ Sessão 13│ Bloco K: Simbiose     │ [CONCLUÍDA] Linkagem desktop-plugins Hermes,│
│          │ Hermes & Systray Win32│ launcher Systray olhos RGB, 7/7 E2E PASS    │
└──────────┴───────────────────────┴─────────────────────────────────────────────┘
```

### Racional da Ordem:
1. **Por que o Bloco B vem primeiro?** Porque o Hermes Agent não pode ser configurado para se conectar a uma porta vazia. O servidor HTTP/SSE bare-metal precisa estar de pé e respondendo em `127.0.0.1:9123` antes de qualquer outra coisa.
2. **Por que a Sessão 2 implementa o Cérebro L3 antes do Hermes?** Para que, quando o Hermes fizer sua primeira chamada MCP (`souls_route_subtask` ou `sub_agent`), o backend já responda com dados reais do SQLite STRICT e do ParetoBandit em vez de stubs vazios.
3. **Por que o Bloco C (Hermes Desktop) entra na Sessão 3?** Porque neste ponto a conexão simbiótica é ativada de verdade! O usuário verá o Hermes Desktop conectado ao Souls Engine com feedback visual e funcional imediato.
4. **Por que o Bloco A (Cloud Arena) fica para o final?** Porque a infraestrutura local (dGPU RTX 2060m e CPU Host) é a fundação bare-metal do projeto. A nuvem atua como camada de fallback complementar que consome a mesma régua de Pareto já validada no silício local.

---

## 5. EVOLUÇÃO FUTURA: CALIBRAÇÃO DINÂMICA DE PRESETS & SELEÇÃO DE MODELOS DEFAULT NO HERMES

A integração do `souls_llm_cloud_arena` com o `souls_llm_local_arena` abre uma avenida para transformar o `config.yaml` do Hermes Agent de um arquivo estático para um sistema de governança orientado a telemetrias empíricas de custo e qualidade:

### 5.1 Seleção Baseada em Evidências do Modelo Mestre (Default Interactive Chat)
Ao auditar os modelos de nuvem sob os mesmos 4 perfis canônicos e normalizar seu Retorno de Silício Amortecido ($V_{\text{task}}$) versus o custo em dólares ($C_{\text{task}}$), o usuário obtém uma base científica para definir o modelo padrão do chat interativo do Hermes:
* **Preset "Heavy SWE & Architecture":** Define `claude-3-7-sonnet` como mestre para sessões de engenharia crítica, onde o custo de API ($3.40/1M) é justificado pela quase ausência de alucinações.
* **Preset "FinOps / Balanced Daily":** Define `deepseek-v3` ou `qwen-2.5-coder-32b` como mestre para sessões de exploração e prototipagem de alta cadência com custo até 10x menor ($0.18 - $0.28/1M).
* **Preset "Deep Theorem & Math":** Define `deepseek-r1` ou `o3-mini` como mestre para resolução de algoritmos formais.

### 5.2 Automação de Presets no `config.yaml` via Script / CLI do Souls Engine
Implementação futura de um comando utilitário (`souls_profile_sync` ou `souls_config_export`) capaz de:
1. Ler os dados consolidados de `model_task_profiles` e `cloud_models` no SQLite STRICT.
2. Gerar ou atualizar blocos nomeados de perfis no `config.yaml` do Hermes Agent.
3. Alternar entre presets operacionais com um único comando, alinhando simultaneamente as portas locais, fallbacks em nuvem e restrições de tokens.

### 5.3 Despacho Híbrido Dinâmico para Subagentes e Cron Jobs
Diferente do chat mestre interativo (que permanece fixo intradiálogo conforme o ADR-003 para preservar o *prefix caching*), as tarefas delegadas (`delegate_task`) e os agendamentos em segundo plano (*cron jobs*) operam com chaveamento 100% dinâmico:
* **Cron Jobs de Manutenção:** Rotinas periódicas de limpeza, indexação e sumarização são absorvidas compulsoriamente pelo Tier 0 (`smollm2-135m` a 149 TPS no i9) com custo financeiro zero e impacto térmico nulo na dGPU.
* **Subagentes de Suporte:** O `souls_route_subtask` decide dinamicamente se despacha a microtarefa para a RTX 2060m (`qwen3.5-agentic-coder` a 46 TPS) ou para a nuvem mais eficiente da Fronteira de Pareto.

### 5.4 Evolução Futura: Launcher Systray & Observabilidade de Bandeja (Windows Companion)
Como evolução ergonômica pós-consolidação headless, o ecossistema ganhará um **Windows Companion Launcher** (`souls_launcher.exe` / systray opcional):
1. **Papel de Encapsulamento:** Atua como maestro e guardião de processo, inicializando o daemon mestre `souls_server.exe` em segundo plano e garantindo que todas as 9 crates estejam orquestradas sob o mesmo ciclo de vida.
2. **Ícone & Tooltip Dinâmico na Bandeja:**
   * Ícone temático no Windows Systray com indicação visual de status (Verde: Online/Ocioso, Âmbar: Subagente em execução, Vermelho: Alerta térmico $\ge 78^\circ\text{C}$).
   * Tooltip ao pairar o mouse (*hover*): exibe em tempo real o status do daemon, porta ativa (`127.0.0.1:9123`), VRAM consumida da RTX 2060m (ex: `1.8 GB / 5.2 GB`) e temperatura da GPU.
3. **Menu de Botão Direito (Context Menu):**
   * *Status & Métricas:* Abre relatório rápido de FinOps e acertos de prefix cache.
   * *Settings (Configurações Futuras):* Alternar presets operacionais do `config.yaml` do Hermes.
   * *Ver Logs do Daemon:* Atalho direto para `Z:\souls_engine\.souls_data\spool\souls_server.log`.
   * *Reiniciar Daemon:* Flush atômico do WAL (`PRAGMA wal_checkpoint(TRUNCATE)`) e reinicialização limpa.
   * *Quit (Sair):* Executa o teardown gracioso com `shutdown()` em todas as crates e encerra os serviços.
4. **Isolamento Arquitetural (Preservação da Linha Vermelha 1):** O binário mestre `souls_server.exe` continua sendo **100% headless** e compilável isoladamente para servidores/containers. O Systray Companion opera como um processo cliente Win32 leve (`windows-sys = "0.59"`) desacoplado, sem introduzir frameworks pesados como Tauri ou Electron no daemon principal.

### 5.5 Bloco F: Hermes Web Dashboard & Souls Settings (Observabilidade & Governança Unificada)
Como evolução futura formalizada para a **Sessão 7**:
1. **Web Dashboard Oficial do Hermes Agent (Plugin REST):**
   * Interface web integrada consumindo os endpoints REST do `souls_server` (`/api/v1/cloud/champions`, `/api/v1/telemetry`, `/api/v1/memory/*`).
   * Visualização interativa dos 5 perfis cognitivos, tabelas comparativas de preços Nous Portal vs. OpenRouter e Fronteira de Pareto em tempo real.
   * Gráficos comparativos de ROI Amortecido ($V_{\text{task}}$), economia obtida com modelos locais na RTX 2060m versus custo evitado em nuvem ($C_{\text{task}}$).
2. **Souls Settings & Governança de Agentes:**
   * Painel de configuração geral para o Souls Engine e perfis do Hermes Agent.
   * Alternância dinâmica de modelos para subagentes, perfis especializados e chat mestre.
   * Gerenciamento e calibração de API keys, limiares de fallback térmico da dGPU e políticas de retenção do Langevin Decay.
### 5.6 RFC-007: Crate souls_llm_inventory (Model Hub, Profiling de Silício & Download Manager)
Como evolução formalizada via RFC-007 (`docs/architecture/RFC_SOULS_LLM_INVENTORY.md`):
1. **Desacoplamento de Ciclo de Vida de Pesos:** Separa a gestão física de arquivos `.gguf`/`.onnx` em disco do runtime de execução (`souls_inference_runtime`).
2. **Padrão de Armazenamento Universal:** Catálogo canônico universal em `%USERPROFILE%\llm_local_models` (sem caminhos hardcoded de usuário), tolerante a diretório vazio e com criação idempotente.
3. **Profiling de Silício Adaptativo:** Mapeamento automático de viabilidade na GPU física (ex: teto útil de 5.294 MB sob WDDM na RTX 2060m) classificando modelos em `OptimalVram`, `PartialCpuOffload` (AVX2) ou `IncompatibleSpillover`.
4. **Download Manager Assíncrono:** Suporte nativo a HTTP Range (resume de downloads pesados), verificação de hash SHA-256 e broadcast de progresso em tempo real via SSE no Web Dashboard.

---

## 6. MAPA EXAUSTIVO DE STATUS CLÍNICO DAS 51 FERRAMENTAS MCP

Para erradicar qualquer ambiguidade sobre stubs e implementações profundas, o quadro abaixo mapeia o ciclo de entrega de cada uma das 51 ferramentas operacionais:

| Garra MCP | Crate Rust | Status Atual | Sessão de Ativação Plena |
| :--- | :--- | :--- | :--- |
| `tool_search`, `routes` | `souls_server` | **LIVE_PRODUCTION** | Sessão 1 (Concluída) |
| `route_subtask` | `souls_model_router` | **LIVE_PRODUCTION** | Sessão 1 (Concluída) |
| `metrics`, `sys_time` | `souls_inference_runtime` / `souls_core` | **LIVE_PRODUCTION** | Sessão 1 (Concluída) |
| `handoff`, `sub_agent` | `souls_memory` | **LIVE_PRODUCTION** | Sessão 3 (Concluída) |
| `thinking`, `session`, `analyze_session`, `export_session`, `merge_sessions` | `souls_memory` | **LIVE_PRODUCTION** | Sessão 3 (Concluída) |
| `knowledge`, `sqlite_query` | `souls_memory` | **LIVE_PRODUCTION** | Sessão 3 (Concluída) |
| `mem_search`, `mem_open_nodes`, `mem_read_graph`, `mem_create_*`, `mem_delete_*` | `souls_memory` | **LIVE_PRODUCTION** | Sessão 3 (Concluída) |
| `semantic_search` | `souls_memory` | **LIVE_PRODUCTION** | Sessão 3 (Concluída) |
| `outline`, `symbol`, `callers`, `callees`, `get_ast`, `edit`, `replace`, `delta_diff` | `souls_ast` | **LIVE_PRODUCTION** | Sessão 4 (Concluída) |
| `repo_heatmap`, `repo_impact` | `souls_ast` | **LIVE_PRODUCTION** | Sessão 4 (Concluída) |
| `intent` | `souls_inference_runtime` | **LIVE_PRODUCTION** | Sessão 4 (Concluída) |
| `read`, `smart_read`, `multi_read`, `compress`, `dedup`, `fill`, `stub_fill`, `headroom_retrieve` | `souls_core` | Staging estruturado; transplante compressão LEAN & CCR | **Sessão 5 (Contexto & Sandbox)** |
| `tree`, `search`, `shell`, `execute`, `fetch_web`, `web_search`, `repo_meta` | `souls_core` | Staging estruturado; transplante sandbox e HTTP scraper | **Sessão 5 (Contexto & Sandbox)** |
| `feedback` | `souls_llm_cloud_arena` | Staging estruturado; transplante Cloud Arena & APIs externas | **Sessão 6 (Cloud Arena)** |

---

## 7. INTEGRAÇÃO MULTI-CLIENTE: HERMES AGENT & IDES (ANTIGRAVITY / CURSOR)

Após a conclusão da **Sessão 3**, o Souls Engine não atenderá apenas o Hermes Agent Desktop, mas se tornará imediatamente utilizável por **qualquer ambiente de desenvolvimento**:

1. **Hermes Agent Desktop (Python 3.11):** Consome o endpoint `http://127.0.0.1:9123/mcp` (Streamable HTTP) ou `/sse` para orquestração executiva de subagentes e memória L3.
2. **Antigravity IDE / Cursor / Claude Desktop / Windsurf:**
   * Basta registrar o Souls Engine na configuração de MCP (`mcp_config.json` ou UI de MCP):
     ```json
     {
       "mcpServers": {
         "souls_engine": {
           "url": "http://127.0.0.1:9123/mcp",
           "transport": "http"
         }
       }
     }
     ```
   * **Benefícios Imediatos na IDE:**
     - A IDE ganha acesso nativo ao `route_subtask` para orientar modelos locais sem gastar créditos de API.
     - Acesso ao raciocínio socrático em grafo DAG (`thinking`).
     - Acesso ao cofre de memórias e fatos de arquitetura no SQLite ReFS (`knowledge`, `mem_*`).
     - Acesso ao profiler de hardware em tempo real da RTX 2060m (`metrics`).

---

## 8. BLOCO E: ENGENHARIA DE SKILLS, PROTOCOLOS AGÊNTICOS & CURADORIA DO OBSIDIAN DB

A maturidade do ecossistema transcende as ferramentas atômicas MCP (`tools`): ele requer **Skills procedimentais** que orquestrem o comportamento dos agentes (Hermes Agent e Antigravity CLI/IDE), assegurem disciplina de protocolos e unifiquem a base viva de conhecimento sem necessidade de microgerenciamento humano:

### 8.1 Matriz de 22 Skills Canônicas (7 Domínios Operacionais)
Para cobrir as 52 ferramentas MCP, 3 camadas de memória e fluxos de auto-aprendizado:
1. **Domínio A (Engenharia de Código & AST):** `souls-code-surgeon`, `souls-structural-diff`, `souls-blast-radius`, `souls-safe-sandbox`.
2. **Domínio B (Hipocampo L2 & Grafo Ontológico):** `souls-memory-triad`, `souls-knowledge-vault`, `souls-ontology-graph`, `souls-langevin-decay`.
3. **Domínio C (Deliberação Cognitiva & Socrático):** `souls-socratic-thinking`, `souls-intent-probing`, `souls-speculative-branch`.
4. **Domínio D (Silício, FinOps & Orquestração):** `souls-subagent-dispatch`, `souls-hardware-watchdog`, `souls-cloud-arena-advisor`.
5. **Domínio E (Contexto Lean, Compressão & Web):** `souls-context-lean`, `souls-web-intelligence`.
6. **Domínio F (Memória Fria do Obsidian DB):** `obsidian-vault-curator`, `obsidian-manifest-tracker`, `obsidian-privacy-shield`, `obsidian-session-logger`.
7. **Domínio G (Auto-Refinamento & Aprendizado):** `agent-self-learning-loop`, `agent-skill-evolution`.

### 8.2 Topologia Resiliente de Persistência no Disco `C:` (ADR-005 Emenda 3.3)
Para garantir imunidade a desmontagens da unidade virtual `Z:` (Dev Drive ReFS):
* **Lar Primário das Skills (Hermes):** `%LOCALAPPDATA%\hermes\skills\` (subpastas `souls-engine/` e `obsidian-curator/`).
* **Espelho Global (Antigravity):** `C:\Users\rosas\.gemini\config\skills/`.
* **Backup Soberano (Git):** `Z:\souls_engine\.agents\skills/`.
* **Script de Curadoria:** `C:\Users\rosas\OneDrive\Documentos\Obisidian DB\scripts\curate_obsidian.ps1` (reside diretamente no cofre e viaja com o OneDrive).

### 8.3 Curadoria Enciclopédica do Obsidian DB (`raw/` $\to$ `wiki/`)
O cofre pessoal em `C:\Users\rosas\OneDrive\Documentos\Obisidian DB` atua como a Memória Fria de Longo Prazo do Arquiteto:
1. **Zona Sagrada `raw/` (Fontes Brutas Imutáveis):**
   * O diretório `raw/` abriga anotações livres, transcrições brutas, recortes e fontes externas.
   * O Antigravity CLI e os subagentes do Hermes tratam o `raw/` em modo estritamente **Read-Only**.
   * Respeito absoluto às regras de privacidade: dados de `raw/financas/` e `raw/saude/` são processados **100% localmente** (GPU/CPU bare-metal, sem envio para nuvens abertas).
2. **Esteira de Curadoria e Criação de Verbetes:**
   * Sintetiza "verbetes" hiperconectados (wikilinks `[[Conceito]]`), notas conceituais atômicas em `wiki/concepts/`, entidades em `wiki/entities/` e resumos em `wiki/summaries/`.
   * Adiciona compulsoriamente frontmatter YAML padronizado (`type`, `tags`, `last_updated`, `reviewed`).
   * Registra todas as operações autônomas no diário incremental `wiki/log.md`.
3. **Manifesto de Progresso & Rastreamento com SHA-256:**
   * O arquivo `C:\Users\rosas\OneDrive\Documentos\Obisidian DB\wiki\raw_manifest.json` mapeia cada arquivo de `raw/` com seu hash SHA-256, status (`PENDING`, `IN_PROGRESS`, `CURATED`, `SKIPPED`) e lista de verbetes derivados.
4. **Modelo de Disparo Híbrido:**
   * **Sob Demanda:** Executável via PowerShell / CLI a qualquer momento (`pwsh scripts/curate_obsidian.ps1`).
   * **Agendamento Automático:** Suporte a disparo periódico via cron do Hermes Agent (`%LOCALAPPDATA%\hermes\cron`).
   * **Lotes Seguros:** Processamento estritamente limitado a no máximo 5 arquivos por rodada (respeitando o limite de autonomia de $\le 10$ arquivos do `AGENTS.md` do cofre).

### 8.4 Ciclo Evolutivo de Aprendizado Contínuo (`/learn` e Auto-Refinamento)
* As correções operacionais e novos padrões descobertos em sessões de trabalho são capturados via comando `/learn`.
* A esteira de governança classifica o destino da aprendizagem:
  - Convenção de código $\to$ `.agents/rules/` ou `AGENTS.md`.
  - Procedimento de ferramenta $\to$ arquivo `SKILL.md` correspondente.
  - Fato ou decisão duradoura $\to$ SQLite `epistemic_memories` (L2) e Obsidian `wiki/` (L3).

---

## 9. CRITÉRIOS DE CONCLUSÃO (DEFINITION OF DONE)
1. Binário `souls_server.exe` compilando sem warnings e respondendo no `localhost:9123`.
2. O Hermes Agent Desktop se conecta ao servidor MCP e visualiza as ferramentas ativas.
3. A IDE (Antigravity / Cursor) se conecta com sucesso ao mesmo endpoint MCP loopback.
4. Um comando `delegate_task` no Hermes consulta com sucesso o `souls_route_subtask` e executa um subagente no modelo local `qwen3-5-4b-agentic-coder`.
5. Os dados da sessão do subagente são persistidos e auditáveis no banco `souls_heuristic_vault.db`.
6. Esteira de Skills e curadoria Obsidian documentadas e alinhadas para implementação na Sessão 6.

---

## 10. SESSÃO 8 — BATERIA DE HOMOLOGAÇÃO CLÍNICA & TESTES PONTA-A-PONTA (E2E) COMPLETOS DO SOULS ENGINE

A **Sessão 8** consolida o teste supremo de integração de todo o ecossistema Souls Engine, atuando como o gate formal de homologação de produção ("Teste Finalzão"). O objetivo é garantir que todas as engrenagens — desde o launcher de bandeja até a inferência local e curadoria de memória — operem harmonicamente em regime de longa duração:

### 10.1 Fase 1: Ciclo de Vida do Daemon, Launcher & Watchdog
1. **Inicialização pelo Launcher / Systray:**
   * Subida do `souls_launcher.exe` com docking no Windows Systray (ícone verde).
   * Verificação do tooltip dinâmico: status `ONLINE`, porta `127.0.0.1:9123`, VRAM consumida da RTX 2060m e temperatura NVML.
   * Validação da regra bare-metal: `souls_server.exe` executando 100% headless em segundo plano.
2. **Graceful Shutdown de 5 Fases:**
   * Disparo de desligamento via menu de bandeja ("Quit") ou sinal Ctrl+C.
   * Validação das 5 fases: recusa de novas conexões HTTP $\to$ dreno de subagentes $\to$ flush do WAL SQLite (`PRAGMA wal_checkpoint(TRUNCATE)`) $\to$ descarregamento de VRAM $\to$ liberação de locks ReFS.

### 10.2 Fase 2: Conexão Simbiótica & Handshake Multi-Cliente
1. **Conexão Hermes Desktop (Python 3.11):**
   * Handshake bem-sucedido via Streamable HTTP/SSE em `http://127.0.0.1:9123/mcp`.
   * Carregamento do catálogo completo de ferramentas MCP no Hermes Agent.
2. **Conexão Antigravity IDE / Cursor:**
   * Verificação de detecção automática via `mcp_config.json`.
   * Consumo concorrente das ferramentas de AST e busca semântica sem contenção de locks de banco.
3. **Conexão Hermes Web Dashboard / Navegador:**
   * Acesso simultâneo via navegador local em `http://127.0.0.1:9123/` e via plugin do Hermes.

### 10.3 Fase 3: Matriz Modular de 3 Cenários Clínicos nas 52 Ferramentas MCP (6 Domínios)
Para cada ferramenta das 52 canônicas, a homologação avalia a robustez através de **3 níveis de complexidade**:
* **Nível 1 (Fácil / Nominal):** Chamada padrão com parâmetros simples para validar schema, tipagem e resposta 200 OK.
* **Nível 2 (Médio / Repositório Real):** Chamada contra dados reais da base de código `Z:\souls_engine`, exigindo processamento, diffs e consultas no SQLite ReFS.
* **Nível 3 (Difícil / Borda, Estresse & Segurança):** Casos extremos: arquivos inexistentes, tentativa de escape fora do Dev Drive ReFS (`C:\...`), concorrência de escrita, inputs vazios ou payloads malformados.

#### Divisão dos 6 Domínios Funcionais:
1. **Domínio 1: Engenharia de Código, AST & Cirurgia Sintática (8 Tools):** `outline`, `symbol`, `callers`, `callees`, `get_ast`, `edit`, `replace`, `delta_diff`.
2. **Domínio 2: Blast Radius, Frecency & Gitoxide (2 Tools):** `repo_heatmap`, `repo_impact`.
3. **Domínio 3: Hipocampo L2, FTS5 & Grafo Ontológico LadybugDB (10 Tools):** `knowledge`, `sqlite_query`, `mem_search`, `semantic_search`, `mem_open_nodes`, `mem_read_graph`, `mem_create_entity`, `mem_create_observation`, `mem_create_relation`, `mem_delete_entity`.
4. **Domínio 4: Sessão, Raciocínio Socrático & Handoff (7 Tools):** `thinking`, `session`, `analyze_session`, `export_session`, `merge_sessions`, `handoff`, `sub_agent`.
5. **Domínio 5: Silício, FinOps & Cloud Arena (7 Tools):** `metrics`, `sys_time`, `route_subtask`, `intent`, `cloud_champions`, `cloud_sync`, `feedback`.
6. **Domínio 6: Contexto LEAN, Safe Sandbox & Web Intelligence (18 Tools):** `tool_search`, `read`, `smart_read`, `multi_read`, `compress`, `dedup`, `fill`, `stub_fill`, `headroom_retrieve`, `tree`, `search`, `shell`, `execute`, `fetch_web`, `web_search`, `repo_meta`...

### 10.4 Fase 4: Homologação Comportamental das 15 Skills Canônicas
1. **Ativação Autônoma Natural:**
   * O Hermes Agent deve ativar a skill apropriada (ex: `souls-code-surgeon`, `souls-socratic-thinking`) com base na intenção da mensagem, sem necessidade de prompting coercitivo.
2. **Escudo de Soberania (`obsidian-privacy-shield`):**
   * Submissão de pergunta envolvendo finanças ou saúde: o agente deve recusar sumariamente despacho para nuvem aberta e forçar execução local na RTX 2060m / i9.
3. **Esteira de Curadoria Viva do Obsidian (`obsidian-vault-curator`):**
   * Execução do script `curate_obsidian.ps1` processando lote de 5 notas de `raw/`: validação do frontmatter YAML, wikilinks `[[...]]`, log em `wiki/log.md` e atualização do `raw_manifest.json` com SHA-256.

### 10.5 Fase 5: Ciclo Fechado de Auto-Aprendizado (`/learn`)
1. **Captura de Correção:** Emissão de uma instrução corretiva pelo Arquiteto.
2. **Execução de `/learn`:** O agente invoca a skill `agent-self-learning-loop`, analisa a causa-raiz e gera patch atômico permanente na regra correspondente ou memória L2.
3. **Verificação de Regressão:** Em diálogo subsequente, o agente obedece à nova diretriz sem reincidir no erro anterior.

### 10.6 Fase 6: Eficiência de Silício, Respostas & FinOps
1. **Teto de VRAM Respeitado:** Consumo sob WDDM mantido rigorosamente $\le 5.2$ GB durante geração local.
2. **Temperatura sob Controle:** GPU $\le 78^\circ\text{C}$; se ultrapassar, acionamento imediato do freio térmico pelo Watchdog com desvio para AVX2.
3. **Latência & Throughput:** Validação de $\ge 40$ tokens/s no Tier 1 (dGPU) e $\ge 120$ tokens/s no Tier 0 (i9 AVX2).

### 10.7 Fase 7: Hermes Web Dashboard & Governança de Settings
1. **Painel de Overview:** Renderização precisa dos gráficos de silício, FinOps e status dos bancos.
2. **Tabelas de Arenas:** Navegação pelos 9 perfis cognitivos, ordenação por $/1M, notas compostas e badges de Pareto.
3. **Painel de Settings:**
   * Alternância de presets do Master Brain com atualização atômica do `config.yaml` do Hermes.
   * Toggles liga/desliga de recursos operando e persistindo no `souls_config.toml`.
   * Botões de ação rápida (WAL Checkpoint, Cloud Sync, Obsidian Curate) respondendo com toasts de sucesso.

### 10.8 Critérios de Homologação Clínica (Sign-off Final)
* Taxa de erro de ferramentas MCP: $0\%$.
* Vazamento de VRAM ou PCIe Spillover: Zero ocorrências.
* Corrupção de finais de linha (CRLF): Zero ocorrências (100% LF verificado via git check).
* Integridade de Dados no ReFS: Arquivos SQLite sem locks zumbis ou corrupção de WAL.
* Emissão do **Relatório de Homologação Clínica do Souls Engine (E2E Production Sign-off)**.

### 10.9 Evolução Arquitetural: Two-Stage Semantic Switchboard (Descoberta Inteligente de Tools e Skills)
Para mitigar a saturação de contexto quando o catálogo atingir 50+ ferramentas e 20+ skills, o `tool_search` evolui para um **Switchboard Semântico em 2 Estágios** operando na CPU i9 com 0 MB VRAM:
1. **Estágio 1 (Dense Vector Match na RAM):** Embeddings compactos pré-computados (ModernBERT / ONNX) comparam a intenção do agente contra o índice vetorial em $\le 5$ ms, recuperando os Top-3 candidatos.
2. **Estágio 2 (Gating de Entropia via Tier 0.5):** Se a confiança for menor que 0.75, o Logit Prober do Tier 0.5 desempata em 8 ms, entregando as ferramentas e a Skill exata a ser ativada, sem poluir a janela de contexto com schemas desnecessários.

### 10.10 Débito Técnico Mapeado: Refinamento Contínuo de UX & Telemetria em Tempo Real do Web Dashboard
Como evolução de usabilidade pós-homologação:
* Substituição dos fallbacks estáticos por fluxos contínuos de Server-Sent Events (SSE) dedicados ao frontend.
* Implementação de ordenação dinâmica interativa nas tabelas de modelos.
* Polimento de microanimações e reconexão silenciosa com retry exponencial em caso de reinicialização do daemon.

### 10.11 Resultado Consolidado da Homologação Clínica (Sessão 8 Concluída com Louvor)
* **Total de Ferramentas Homologadas:** 52 de 52 (100% das garras canônicas do Souls Engine).
* **Matriz de Cenários Clínicos:** 156 cenários avaliados nos 6 domínios funcionais (Fácil, Médio, Difícil).
* **Taxa de Sucesso:** **100.0% (156 / 156 cenários aprovados)**.
* **Segurança ReFS (Linha Vermelha 4):** 100% das tentativas de escape barradas via `RefsPathLeakViolation` em tempo real (< 3.5 ms).
* **Integridade Epistêmica:** Zero corrupção de banco, zero locks zumbis no FrankenSQLite STRICT WAL, zero vazamento de VRAM.
* **Relatório Oficial:** [SOULS_ENGINE_E2E_HOMOLOGATION_REPORT.md](file:///z:/souls_engine/docs/homologation/SOULS_ENGINE_E2E_HOMOLOGATION_REPORT.md).

---

## 11. PLANO DE CONCLUSÃO FINAL, INTEGRAÇÃO PROFUNDA & POLIMENTO (SESSÕES 9 A 13)

Com a fundação bare-metal do daemon e o contrato MCP 100% homologados na Sessão 8, este plano consolida as **5 Frentes Estratégicas** e os **5 Pontos de Auditoria Críticos** para levar o Souls Engine v7 do estado funcional para a prontidão simbiótica absoluta com o Hermes Agent Desktop.

```mermaid
flowchart LR
    S9["Sessão 9: Bloco G\nSilício Neural & Switchboard"] --> S10["Sessão 10: Bloco H\nDes-staging Domínio 6"]
    S10 --> S11["Sessão 11: Bloco I\nSkills & Obsidian Viva"]
    S11 --> S12["Sessão 12: Bloco J\nFix Banco & Dashboard SSE"]
    S12 --> S13["Sessão 13: Bloco K\nSimbiose Hermes & Systray"]
```

### 11.1 Sessão 9 (Bloco G): Silício Neural Real & Two-Stage Semantic Switchboard [CONCLUÍDA]
* **Objetivo:** Tirar o cérebro local do modo *fail-soft determinístico* e ativar a inferência neural real com 0 MB de VRAM na CPU Host (AVX2).
* **Ponto de Auditoria 1 (Desalinhamento de Caminhos dos 15 Modelos) [RESOLVIDO]:**
  * O código procurava modelos hardcoded em caminhos legados inexistentes.
  * **Ação Executada:** Atualizados `llamacpp.rs`, `inference.rs` e `onnx.rs` em `souls_inference_runtime` para mapear dinamicamente os 15 modelos de elite físicos (32.57 GB) existentes em `C:\Users\rosas\llm_local_models\`:
    - **Tier 0:** `SmolLM2-135M-Reasoning-5K-Q4_K_M.gguf`
    - **Tier 0.5:** `smollm2-360m-instruct-q8_0.gguf`
    - **Tier 1 (dGPU):** `qwen3.5-4b-agentic-coder-v4.i1-Q4_K_S.gguf`
    - **Tier 3 (Vision):** `Qwen2.5-VL-3B-Instruct-Q4_K_M.gguf`
* **Implementação do Switchboard Semântico em 2 Estágios (`switchboard.rs`) [RESOLVIDO]:**
  - **Estágio 1 (RAM Dense Match):** Similaridade de cosseno ultrarrápida no espaço léxico-semântico na memória RAM em **< 1.3 ms**.
  - **Estágio 2 (Gating de Entropia via Tier 0.5 Logits):** Executado via `infer_tier05_probe` com o modelo real `SmolLM2-135M` no silício do i9 (AVX2), calculando entropia de Shannon e sugerindo a Skill canônica exata + Top-3 ferramentas relevantes.
  - **Zero Poluição de Contexto:** 100% de preservação de *Prefix Caching* e zero MB de VRAM desperdiçada na dGPU.
* **Validação de Testes Compulsória:**
  - `souls_inference_runtime`: **33/33 testes unitários aprovados (100% PASS)**.
  - `souls_server`: **22/22 testes unitários aprovados (100% PASS)**.
  - Teste ponta a ponta via Streamable HTTP (`/mcp`): resposta em **1.23 ms a 1.65 ms**, recomendando perfeitamente as skills `souls-hardware-watchdog` e `souls-code-surgeon`.

### 11.2 Sessão 10 (Bloco H): Transplante & Des-staging das 14 Ferramentas do Domínio 6 [CONCLUÍDA]
* **Objetivo:** Substituir as respostas de envelope `staging_ready` pela execução de baixo nível real bare-metal.
* **Ferramentas Entregues (100% Des-stagadas):**
  - **Contexto LEAN (`context.*`):** `read`, `smart_read`, `multi_read`, `compress`, `dedup`, `fill`, `stub_fill`, `headroom_retrieve`.
  - **Sistema & Sandbox (`sys.*`):** `tree`, `search`, `shell`, `execute`, `fetch_web` (conversão HTML $\to$ Markdown na RAM), `web_search` (DuckDuckGo Lite), `repo_meta`.
  - **Memória & Grafo (`memory.*`):** `mem_search` (FTS5), `mem_open_nodes`, `mem_read_graph`, `mem_add_observations`, `mem_delete_entities`, `mem_delete_relations`, `mem_delete_observations`.
  - **Deliberação & FinOps (`think.*`, `ops.*`):** `session`, `merge_sessions`, `feedback` (cálculo de retorno de silício e telemetria SQLite STRICT).
* **Entregas Técnicas Bare-Metal:**
  - `crates/souls_server/src/mcp/tools_extended.rs`: Implementação modular com isolamento de caminhos ReFS (`validate_refs_path`) e zero unsafe arbitrário.
  - `compress`: Algoritmo LEAN com remoção de comentários sintáticos (`//`, `/* */`), preservação de docstrings e colapso de linhas em branco.
  - `dedup` & `fill`: Cache seguro thread-safe na RAM com hashing de blocos repetidos de 5+ linhas e reidratação lossless.
  - `fetch_web` & `web_search`: Raspador leve em buffer de memória, convertendo HTML para Markdown diretamente na RAM com economia drástica de contexto.
* **Validação de Testes Compulsória:**
  - `souls_server`: **23/23 testes unitários e de integração aprovados (100% PASS)**, incluindo `test_mcp_domain6_and_extended_tools`.
  - Zero warnings de formatação ou CRLF (`git diff --check` limpo). Binário `souls_server.exe` reconstruído e operando em `127.0.0.1:9123`.


### 11.3 Sessão 11 (Bloco I): Homologação Comportamental das 15 Skills & Curadoria Viva do Obsidian [CONCLUÍDA]
* **Objetivo:** Validar o alinhamento ético e os reflexos operacionais autônomos dos agentes no Two-Stage Switchboard e esteira do cofre Obsidian.
* **Bateria de Verificação de Skills:**
  - `tools/test_skills_behavior.ps1`: **15/15 Skills Aprovadas (100% PASS)** com latência média de **18.73 ms**.
  - `obsidian-privacy-shield`: Dados de saúde e finanças interceptados; garantia de roteamento estritamente local (`LOCAL_ONLY`) na dGPU / CPU i9 bare-metal.
  - `souls-code-surgeon`: Proibição estrita de leitura de arquivos inteiros; indução do fluxo `outline` $\rightarrow$ `symbol` $\rightarrow$ `edit`.
  - `souls-hardware-watchdog`: Monitoramento termodinâmico contínuo e contenção de VRAM sob o teto útil de 5.2GB do WDDM.
  - `agent-self-learning-loop`: Teste de ciclo de auto-patch `/learn`, registrando lições aprendidas em regras duradouras.
* **Ponto de Auditoria 5 (Esteira Viva do Obsidian Vault):**
  - Execução de `tools/curate_obsidian.ps1` no acervo real do Arquiteto em `C:\Users\rosas\OneDrive\Documentos\Obisidian DB\raw`: **498 documentos escaneados e catalogados**.
  - 2 arquivos sensíveis demarcados com isolamento criptográfico `LOCAL_ONLY 🔒`.
  - Manifesto criptográfico `wiki/raw_manifest.json` com cálculo SHA-256 de integridade e log cronológico em `wiki/log.md`.
  - Relatório oficial consolidado em [docs/homologation/SOULS_ENGINE_SKILLS_AND_OBSIDIAN_HOMOLOGATION.md](file:///z:/souls_engine/docs/homologation/SOULS_ENGINE_SKILLS_AND_OBSIDIAN_HOMOLOGATION.md).


### 11.4 Sessão 12 (Bloco J): Correção de Banco, UX do Dashboard & Configuração Centralizada [CONCLUÍDA]
* **Ponto de Auditoria 2 (Bug de Foreign Key no Cloud Arena Sync — RESOLVIDO):**
  - Causa raiz identificada: catálogo público da OpenRouter contendo 300+ modelos desconhecidos sem pai em `canonical_models`, violando a constraint referencial STRICT `FOREIGN KEY (code: 787)`.
  - Solução bare-metal: implementação de `try_resolve` e filtragem estrita em `CanonicalResolver`, aceitando exclusivamente os modelos canônicos mapeados da arena.
  - Resultado no silício: **12 modelos, 18 snapshots de preço, 12 benchmarks e 50 avaliações de Pareto gravadas com sucesso** no boot sem warnings de Foreign Key.
* **Ponto de Auditoria 4 (Persistência Central no souls_config.toml — ENTREGUE):**
  - Criação e governança de `Z:\souls_engine\.souls_data\config\souls_config.toml`.
  - Endpoints REST `GET /api/v1/settings/config` e `POST /api/v1/settings/config` operando de forma atômica no Dev Drive ReFS.
* **UX & Streaming em Tempo Real do Web Dashboard (ENTREGUE):**
  - Implementado stream contínuo Server-Sent Events em `GET /api/v1/telemetry/stream` (1000ms heartbeat) com métricas de silício vivas.
  - Frontend reconecta dinamicamente via `EventSource` com fallback transparente de polling.
  - Ordenação interativa bidirecional (asc/desc) por clique em cabeçalhos de coluna nas tabelas da Cloud Arena e Silicon Arena.
* **Validação de Testes Compulsória:**
  - `souls_llm_cloud_arena`: **6/6 testes de integração e 4/4 unitários aprovados (100% PASS)**.
  - `souls_server`: **23/23 testes de integração aprovados (100% PASS)**.
  - Daemon reconstruído e saudável em `127.0.0.1:9123`.


### 11.5 Sessão 13 (Bloco K): Simbiose Viva Hermes Desktop & Windows Systray Companion [CONCLUÍDA]
* **Ponto de Auditoria 3 (Instalação dos Adaptadores no Hermes — ENTREGUE):**
  - Implementação completa dos adaptadores em `adapters/hermes_dashboard_souls` e `adapters/hermes_memory_souls` com manifestos `plugin.yaml` e adapter Python assíncrono.
  - Script `tools/install_hermes_plugins.ps1` executado: junções vinculadas com sucesso em `%LOCALAPPDATA%\hermes\desktop-plugins\`.
* **Windows Systray Companion (`souls_launcher.ps1` — ENTREGUE):**
  - Utilitário Win32/PowerShell desacoplado com `NotifyIcon` nativo, preservando a **Linha Vermelha 1** (daemon `souls_server.exe` 100% headless).
  - Alternância dinâmica dos olhos do mascote com ícones multi-resolução de [assets/branding/](file:///z:/souls_engine/assets/branding/) (cyan = nominal, green = ativo, amber = preventivo, red = crítico, purple = offline).
  - Tooltip vivo exibindo zona térmica, temperatura da GPU, VRAM alocada sob teto de 5.2GB e conexões ativas.
  - Menu de contexto rápido: abrir dashboard, forçar sincronização da Cloud Arena e compactar WAL do SQLite.
* **O Teste Finalzão de Simbiose (`tools/test_symbiosis_e2e.ps1`):**
  - Execução e validação dos 7 pilares fundamentais da fábrica bare-metal: **7/7 PASS (100% de Aprovação Geral)**.
  - 1. Daemon Bare-Metal Health (:9123) 🟢
  - 2. Cloud Arena & SQLite STRICT WAL (50 avaliações de Pareto, zero violações) 🟢
  - 3. Two-Stage Semantic Switchboard MCP (<5ms) 🟢
  - 4. Domínio 6: Compressão LEAN Real 🟢
  - 5. Adaptadores Hermes Desktop Plugin Links 🟢
  - 6. Configuração Centralizada `souls_config.toml` 🟢
  - 7. Esteira Viva de Curadoria do Obsidian (`raw/` $\rightarrow$ `wiki/` com 498 arquivos) 🟢

---

## 12. MATRIZ CONSOLIDADA: SESSÕES 1 A 13 ENTREGUES (100% HOMOLOGADO)

O **Souls Engine v7** atingiu o ápice de sua especificação funcional e homologação clínica:
- **Topologia:** Exatamente 10 crates canônicas compilando com MSVC e zero warnings.
- **Catálogo MCP:** 52 ferramentas reais ativas, des-stagadas e testadas ponta a ponta (156/156 cenários aprovados).
- **Skills Agênticas:** 15 skills homologadas com reflexos éticos e finops.
- **Silício Físico:** 15 modelos neurais em disco (32.57 GB) acoplados ao i9 (AVX2) e RTX 2060m.
- **Simbiose:** Hermes Agent Desktop e Souls Engine operando em uníssono bare-metal.

---

## 13. SESSÃO EXTRA DE AUDITORIA, FIX DE CONFIGURAÇÃO & BINÁRIO RELEASE

### 13.1 Post-Mortem & Fix Definitivo do `config.yaml` do Hermes Agent
* **Diagnóstico de Causa-Raiz:** O antigo handler `hermes_preset_handler` em `crates/souls_server/src/routes/dashboard.rs` utilizava substituição de texto cega (`line.trim_start().starts_with("model:")`), convertendo o cabeçalho mapeado hierárquico `model:` em um escalar puro (`model: "deepseek/..."`). A linha seguinte indentada (`  default: ...`) gerava erro de gramática YAML na inicialização do Hermes e quebrava o `hermes update`:
  `Details: expected <block end>, but found '<block mapping start>'`
* **Solução Entregue (Commit `e3269f8`):** O handler foi refatorado para navegar conscientemente na árvore YAML e atualizar **exclusivamente** a linha `  default:` sob o bloco `model:` de nível superior. O arquivo do usuário em `%LOCALAPPDATA%\hermes\config.yaml` foi restaurado a partir do backup limpo com o bloco canônico de transporte `mcp_servers.souls_engine` injetado no topo, validado e aprovado com 100% de sucesso por `hermes config check`.

### 13.2 Blindagem de Porta TCP (Single-Instance Probe & Zombie Recovery)
* **Solução Entregue (Commit `1163afb`):** Diante de colisão de porta TCP (`os error 10048` / WSAEADDRINUSE):
  - O daemon emite uma sonda assíncrona imediata para `http://127.0.0.1:9123/health` (timeout de 800ms).
  - Se responder **200 OK Saudável**: O processo reconhece que o Souls Engine já está ativo, exibe mensagem amigável e encerra com `ExitCode::SUCCESS` (código 0, sem panic ou janelas de erro no Windows).
  - Se a porta estiver presa por um processo órfão que não responde: emite alerta claro diagnosticando zumbi e instrui o uso de `stop_souls.bat`.
  - Gravação determinística de `Z:\souls_engine\.souls_data\souls_server.pid` para rastreamento de processo.

### 13.3 Compilação de Produção Release & Dupla Identidade de Binários
* **Entrega:** Execução de `cargo build --release -p souls_server` gerando o binário nativo MSVC x86_64 otimizado (reduzido de 64 MB para 34.9 MB).
* **Esteira `tools/build_souls_release.ps1` (Commit `bd56fc8`):** Disponibiliza simultaneamente:
  - `target\release\souls_server.exe` (identidade formal de especificação headless).
  - `target\release\souls_engine.exe` (identidade de produto executável esperada pelo usuário).
* **Launchers Dinâmicos:** `start_souls.bat` e `tools/souls_launcher.ps1` atualizados para priorizar a versão de release `souls_engine.exe`, com parada em lote de ambos em `stop_souls.bat`.
* **Governança Canônica:** Registro formal das lições aprendidas em `docs/architecture/LESSONS_LEARNED_AND_POSTMORTEM.md` e Seção 8 do `AGENTS.md` (Commit `0f3b17a`).

---

## 14. VARREDURA FORENSE DO ACERVO LEGADO (`_donor/`) & DECISÃO ESTRATÉGICA

Uma dissecação profunda sobre os 100+ arquivos em `Z:\souls_engine\_donor` versus `docs/architecture/CRATES_SPECIFICATION.md` revelou:

### 14.1 Decisão Executiva: Crate `souls_anthropophagy` em Sessão Monográfica Dedicada
* **Diagnóstico:** Atualmente a crate `crates/souls_anthropophagy` possui apenas 1 arquivo placeholder (`add(2, 2)`). No doador `_donor/souls_mc_anthropophagy`, residem subsistemas densos de alta complexidade:
  - 11 linters SAST poliglotas (`harvester/sast/`: bandit, biome, clippy, cppcheck, govulncheck, oxc, ruff, sobelow).
  - Firewall de conformidade de licenças SPDX (`spdx.rs`: rejeição de GPL/AGPL virais).
  - Dissecador de AST e Amputador cirúrgico de funções (`dissector.rs` e `slicer.rs`).
  - Catálogo "The Organ Vault" no SQLite (`vault.rs` e `persist/`).
  - Motores de síntese e orquestração (`synthesizer.rs` [167 KB] e `swarm.rs` [66 KB]).
* **Diretriz Aprovada:** Por se tratar de um módulo volumoso e especializado, **a transposição e consolidação de `souls_anthropophagy` será realizada em uma sessão própria e exclusiva**, evitando inchaço de escopo e dispersão de foco.

### 14.2 Órgãos Saudáveis Identificados para Conclusão do Restante do Workspace
Para todas as demais crates, os seguintes módulos de alto valor do `_donor` foram selecionados para a esteira final:
1. **`_donor/.../vector_store.rs` (537 linhas):** O reator físico colunar LanceDB (`arrow`, `FixedSizeListArray`, `mmap`) para persistir embeddings de 384d em `.souls_data/souls_vectors.lance` com 0 MB de VRAM na `souls_memory`.
2. **`_donor/.../subprocess_guard.rs` (505 linhas):** Guardião RAII de subprocessos com `TerminateProcess` atômico Win32 para blindar as ferramentas `shell` e `execute` de zumbis em `souls_core`.
3. **`_donor/.../pii_redactor.rs` (391 linhas):** Autômato linear Aho-Corasick $\mathcal{O}(N)$ para mascaramento e desmascaramento reversível de caminhos/chaves locais antes de envio a APIs em `souls_core`.
4. **`_donor/.../cohomology.rs` (746 linhas):** Álgebra matricial pura sobre Galois Field $GF(2)$ em `[u64; 4]` via Rouché-Capelli ($H^1 = \text{rank}([A \mid b]) - \text{rank}(A)$) para detecção de contradições de premissas em grafos socráticos no `souls_inference_runtime`.
5. **`_donor/.../call_graph.rs`:** Rastreamento estático de grafos de invocação baseado na árvore sintática do Tree-sitter para elevar a precisão de `callers` e `callees` em `souls_ast`.
6. **`_donor/.../context_stitcher.rs`:** Composição em 4 Zonas com Block Boundary Padding em múltiplos de 64 tokens para maximização de Prefix Caching.

### 14.3 Órgãos Proibidos / Incineração Compulsória (Linhas Vermelhas)
- `agentgateway_tcp_proxy.rs` e `l7_shield.rs`: Proxy interceptador L7 na porta :3001 (Violação da Linha Vermelha 2).
- Servidor Axum Svelte na porta :3002 e referências a Tauri v2 / Wry / Winit (Violação da Linha Vermelha 1).
- `boot.ps1` legado e caminhos fora do Dev Drive ReFS (Violação da Linha Vermelha 4).

---

## 15. A ESTEIRA DE FECHAMENTO DEFINITIVO EM 5 FASES (100% PRONTIDÃO DE PRODUTO)

```mermaid
flowchart TD
    F1["Fase 1: Fundações de Segurança & Protocolo\n(souls_protocol + souls_core)"] --> F2["Fase 2: Hipocampo L3 Vetorial Físico\n(souls_memory + LanceDB ReFS)"]
    F2 --> F3["Fase 3: Inteligência Matemática & AST\n(cohomology.rs + call_graph.rs)"]
    F3 --> F4["Fase 4: Amarração de Conexões no Servidor\n(souls_server endpoints & MCP)"]
    F4 --> F5["Fase 5: Empacotamento, Instalador & Produto\n(install_souls.ps1 + Inno Setup + Teste em Voo)"]
```

### 🔹 Fase 1: Fundações de Segurança, Protocolo & Proteção de Processos
* **Crates:** `souls_protocol` (Nível 0) e `souls_core` (Nível 1).
* **Entregas:**
  1. Desmembramento modular de `souls_protocol/src/` em `dto.rs`, `mcp.rs`, `lean.rs`, `error.rs` conforme `CRATES_SPECIFICATION.md`.
  2. Transplante de `subprocess_guard.rs` para `souls_core` (RAII process lifecycle guard contra processos zumbis).
  3. Transplante de `pii_redactor.rs` para `souls_core` (Aho-Corasick linear $\mathcal{O}(N)$ para soberania PII).
  4. Implementação de `panic.rs` em `souls_core` (`run_safe_task` assíncrono sobre `catch_unwind`).
* **DoD:** `cargo test -p souls_protocol` e `cargo test -p souls_core` com 100% PASS.

### 🔹 Fase 2: Hipocampo L3 Vetorial Físico (O Reator LanceDB em ReFS)
* **Crate:** `souls_memory` (Nível 2).
* **Entregas:**
  1. Transplante de `vector_store.rs` para `crates/souls_memory/src/lancedb.rs`.
  2. Abertura e gestão da tabela Arrow em `Z:\souls_engine\.souls_data\souls_vectors.lance` com 0 MB de VRAM.
  3. Amarração do RRF Hybrid Fusion para mesclar consultas FTS5 do SQLite com busca vetorial colunar do LanceDB em disco.
* **DoD:** `cargo test -p souls_memory` gravando e recuperando vetores do disco ReFS sem vazamento de memória.

### 🔹 Fase 3: Inteligência Matemática Avançada & AST Determinística
* **Crates:** `souls_inference_runtime` (Nível 2) e `souls_ast` (Nível 2).
* **Entregas:**
  1. Transplante de `cohomology.rs` para `souls_inference_runtime` (eliminação Gaussiana em GF(2) `[u64; 4]` para detecção de contradições em grafos DAG socráticos).
  2. Conexão da álgebra de cohomologia com o `ThinkingEngine` socrático.
  3. Transplante de `call_graph.rs` para `souls_ast` baseado em parsing Tree-sitter estático.
* **DoD:** `cargo test -p souls_inference_runtime` e `cargo test -p souls_ast` aprovados.

### 🔹 Fase 4: Amarração de Conexões no Servidor Headless
* **Crate:** `souls_server` (Nível 4).
* **Entregas:**
  1. Atualização das ferramentas `shell` e `execute` em `tools_extended.rs` para usar o `SubprocessGuard` RAII.
  2. Plugar o LanceDB físico na garra `semantic_search` e `mem_search`.
  3. Plugar a detecção de contradições na garra `thinking`.
  4. Plugar o call-graph sintático nas garras `callers` e `callees`.
* **DoD:** 23/23 testes de integração do `souls_server` aprovados (`cargo test -p souls_server`).

### 🔹 Fase 5: Empacotamento, Produto & Distribuição (Ready-to-Ship)
* **Entregas de Produto & Ferramental:**
  1. **Instalador Autônomo de Produção (`tools/install_souls.ps1`):**
     - Instalação dos executáveis release (`souls_engine.exe` e `souls_server.exe`) em `%LOCALAPPDATA%\Programs\SoulsEngine\`.
     - Validação e criação da estrutura ReFS em `Z:\souls_engine\.souls_data\`.
     - Injeção idempotente e segura no `config.yaml` do Hermes Agent com verificação via `hermes config check`.
     - Criação de links simbólicos e cópia dos plugins em `%LOCALAPPDATA%\hermes\desktop-plugins\`.
     - Criação de atalhos no Menu Iniciar e na pasta Startup do Windows com ícones `.ico` do mascote Souls Engine.
  2. **Manifesto Inno Setup (`installer/souls_engine_setup.iss`):** Script para geração opcional do instalador `.exe` gráfico do Windows com assistente guiado.
  3. **Sessão Prática Simbiótica ao Vivo com o Hermes Agent:**
     - Ligar o engine via `start_souls.bat` no modo release.
     - Abrir o Hermes Agent e emitir comandos reais consumindo a memória L3 profunda, checando a telemetria ao vivo no Web Dashboard e o status visual na bandeja.

---

## 16. GUIA DEFINITIVO DE RESILIÊNCIA, DISTRIBUIÇÃO E RETOMADA PARA AGENTES DE IA

> **AVISO PARA O PRÓXIMO AGENTE DE IA (HANDOFF DE MEMÓRIA):**
> Este documento é o **mapa canônico e a fonte suprema da verdade** do Souls Engine. Se você está iniciando um novo turno ou retomando a evolução do projeto, leia atentamente esta seção antes de sugerir ou alterar qualquer código.

### 16.1 Arquitetura de Resiliência a Falhas, Zumbis & Crash Recovery
O usuário questionou formalmente: *"Se o souls_engine cair, travar ou ficar parado no sistema... abrir um novo souls_engine está tratando para limpar com segurança? E mesmo se quebrar, o souls_engine está tratando para evitar perdas?"*

A governança bare-metal do Souls Engine resolve essa preocupação através de 4 barreiras físicas:
1. **Single-Instance Health Probe Dinâmica (Porta 9123):**
   - Ao iniciar, o `souls_server.exe` (ou `souls_engine.exe`) tenta abrir o listener TCP em `127.0.0.1:9123`.
   - Se ocorrer colisão (`WSAEADDRINUSE` / erro 10048), o daemon dispara imediatamente uma sonda assíncrona HTTP em `http://127.0.0.1:9123/health` (timeout 800ms).
   - Se responder **200 OK Saudável**: O processo reconhece que o daemon legítimo já está ativo em segundo plano, emite mensagem amigável no console e encerra com `ExitCode::SUCCESS` (código 0, sem panic e sem alertas de erro do Windows).
   - Se a porta estiver presa por um processo órfão que não responde (zumbi): emite diagnóstico de alerta no console e orienta o uso do limpador atômico.
2. **Matador Atômico de Processos Órfãos (`stop_souls.bat`):**
   - O utilitário `stop_souls.bat` executa terminação forçada em árvore de processos (`taskkill /F /T /IM souls_server.exe` e `/IM souls_engine.exe`), liberando instantaneamente as portas Win32 e os file locks.
3. **Integridade de Dados & Imunidade a Quedas Repentinas no ReFS:**
   - **FrankenSQLite STRICT WAL:** Todo banco em `Z:\souls_engine\.souls_data\` opera com `PRAGMA journal_mode = WAL` e `PRAGMA synchronous = NORMAL`. As escritas são registradas sequencialmente em append-only no arquivo `-wal`. Mesmo em caso de queda de energia ou terminação forçada por `taskkill`, o SQLite recupera o banco de forma automática e atômica na próxima abertura sem perda de transações comitadas.
   - **Graceful Shutdown:** O comando de encerramento do daemon executa flush atômico `PRAGMA wal_checkpoint(TRUNCATE)` consolidando todos os dados no banco principal e liberando espaço de disco.
   - **LanceDB (Apache Arrow em ReFS):** A camada vetorial do LanceDB em `.souls_data/souls_vectors.lance` utiliza fragmentos colunares imutáveis com manifestos versionados. Operações de append nunca sobrescrevem blocos existentes; uma escrita interrompida simplesmente descarta o fragmento não indexado sem corromper o índice anterior.
4. **Isolamento de Tarefas com `catch_unwind`:**
   - Na Fase 1 da esteira, o módulo `panic.rs` em `souls_core` (`run_safe_task`) envolve a execução assíncrona de subagentes e ferramentas para que um eventual pânico isolado em um parser ou regex nunca derrube o daemon principal.

### 16.2 Topologia de Pastas e Diretórios Padrão do Produto
Para manter a estrita conformidade com as Linhas Vermelhas do `AGENTS.md` e a higiene de disco do Windows 11:

| Componente | Diretório Canônico | Finalidade Arquitetural |
| :--- | :--- | :--- |
| **Executáveis de Release** | `%LOCALAPPDATA%\Programs\SoulsEngine\` | Binários nativos `souls_engine.exe` e `souls_server.exe` (34.9 MB). |
| **Dados Bare-Metal & Bancos** | `Z:\souls_engine\.souls_data\` | Dev Drive ReFS isolado: SQLite STRICT WAL, LanceDB colunar, logs e PIDs. |
| **Modelos Físicos de Silício** | `C:\Users\rosas\llm_local_models\` | 15 modelos GGUF locais (32.57 GB) mapeados no `souls_inference_runtime`. |
| **Configuração Centralizada** | `Z:\souls_engine\.souls_data\config\souls_config.toml` | Toggles e configurações de governança lidos e gravados pelos endpoints REST. |
| **Integração com Hermes Agent** | `%LOCALAPPDATA%\hermes\config.yaml` | Injeção do bloco de conexão `mcp_servers.souls_engine` (Streamable HTTP :9123). |
| **Plugins do Hermes Desktop** | `%LOCALAPPDATA%\hermes\desktop-plugins\` | Junções simbólicas apontando para os adaptadores de dashboard e memória do Souls Engine. |
| **Skills dos Agentes** | `%LOCALAPPDATA%\hermes\skills\` e `.agents\skills\` | 15 Skills canônicas procedimentais para direcionar comportamento ético e finops. |
| **Memória Fria (Cofre Pessoal)** | `C:\Users\rosas\OneDrive\Documentos\Obisidian DB\` | Esteira de curadoria viva `raw/` $\rightarrow$ `wiki/` (498 arquivos) com isolamento `LOCAL_ONLY`. |

### 16.3 Status de `souls_anthropophagy` (A Crate Monográfica)
- A crate `crates/souls_anthropophagy` está **propositalmente mantida como placeholder** no momento.
- Todo o material rico do doador (`_donor/souls_mc_anthropophagy` com 11 linters SAST, verificador SPDX, Organ Vault, dissector e swarm) está preservado e catalogado para ser **transplantado em uma sessão monográfica própria e dedicada** após a conclusão das 5 Fases deste roadmap.
- Nenhum agente deve tentar adiantar ou misturar `souls_anthropophagy` com as demais crates para evitar explosão de contexto e violação da Regra de Crate Única do `AGENTS.md`.

### 16.4 Status da Esteira de Conclusão dos 100% (5 Fases Canônicas)
1. **Fase 1: Fundações de Segurança & Protocolo [CONCLUÍDA & COMMITADA]:**
   - `souls_protocol`: Modularizado em `dto.rs`, `mcp.rs`, `lean.rs`, `error.rs` com reexportações retrocompatíveis. 15/15 testes PASS. Commit `07275a9`.
   - `souls_core`: Implementados `subprocess_guard.rs` (RAII process lifecycle contra zumbis), `pii_redactor.rs` (Aho-Corasick linear O(N) com tokens reversíveis em RAM) e `panic.rs` (`run_safe_task` e `run_safe_sync` sobre `catch_unwind`). 22/22 testes PASS. Commit `26a5a07`.
2. **Fase 2: Hipocampo L3 Vetorial Físico [CONCLUÍDA & COMMITADA]:**
   - `souls_memory`: Implementado `lancedb.rs` conectando a tabela Arrow colunar `observations_vector` em `Z:\souls_engine\.souls_data\souls_vectors.lance` operando com 0 MB de VRAM via Arrow memory model em CPU/RAM. Integrado `hybrid_recall` no `SoulsMemoryStore` fundindo FTS5 lexical BM25 e busca vetorial densa LanceDB via RRF (k=60.0). 26/26 testes PASS. Commit `ef24093`.
3. **Fase 3: Inteligência Matemática Avançada & AST Determinística [CONCLUÍDA & COMMITADA]:**
   - `souls_inference_runtime`: Implementado `cohomology.rs` com álgebra linear sobre Galois Field GF(2) em `[u64; 4]` ocupando apenas 8.2 KB na stack para detecção de contradições em premissas de grafos socráticos via Rouché-Capelli. Commit `bb75427`.
   - `souls_ast`: Implementado `call_graph.rs` com grafo de chamadas direcionado (`CallGraphNode`, callers e callees) e índice de símbolos em DashMap lock-free na RAM host. 22/22 testes PASS. Commit `b1b4979`.
   - **Correção Crítica de Concorrência dGPU:** Implementado `gpu_execution_lock()` (`OnceLock<Mutex<()>>`) em `inference.rs` garantindo exclusão mútua em nível de processo antes de qualquer chamada a `llama-completion.exe`. Commit `f48e033`.
4. **Fase 4: Amarração de Conexões no Servidor Headless (`souls_server`) [CONCLUÍDA & COMMITADA]:**
   - Conectado `SubprocessGuard` nos handlers de `shell` e `execute` em `mcp/tools_extended.rs`, com imunidade a zumbis e terminação limpa via RAII Drop.
   - Conectado `hybrid_recall` no handler `semantic_search` e busca vetorial LanceDB em `mem_search`.
   - Conectada a avaliação de cohomologia de feixes socráticos (`cohomology::compute_cohomology`) nos handlers `thinking` e `analyze_session`.
   - Conectado o `CallGraphStore` de `souls_ast` nos handlers `callers` e `callees` em `mcp/handlers.rs`.
   - Teste de integração de voo da Fase 4 adicionado (`test_mcp_phase4_connections_integration`).
   - 24/24 testes PASS. Commit `464efe2`.
5. **Fase 5: Empacotamento, Produto & Distribuição [CONCLUÍDA & HOMOLOGADA]:**
   - Criado e executado o script mestre `tools/install_souls.ps1` instalando o ecossistema em `C:\Users\rosas\AppData\Local\SoulsEngine\`.
   - Gerados os binários gêmeos release `souls_server.exe` e `souls_engine.exe` (27.1 MB otimizados MSVC LTO/O3).
   - Auto-montagem transparente do Dev Drive ReFS (`C:\dev_projects\Souls_FORGE.vhdx`) implementada tanto no binário nativo Rust quanto no launcher PowerShell.
   - Criados atalhos de alta resolução no Menu Iniciar e Desktop com os ícones multi-resolução do mascote Souls Engine.
   - Injeção cirúrgica no `%LOCALAPPDATA%\hermes\config.yaml` com backup `.bak` e 100% aprovado no `hermes config check`.
   - Links Junction dos adaptadores de plugins ativos em `%LOCALAPPDATA%\hermes\desktop-plugins\`.
   - Criado o manifesto Inno Setup `installer/souls_engine_setup.iss` para instalador gráfico de distribuição.
   - Teste de voo do daemon release instalado aprovado com status `healthy` e VRAM 4503 MB livre no `http://127.0.0.1:9123/health`.

---

## 17. SESSÃO EXTRAORDINÁRIA: AUDITORIA, EXPURGO ANACRÔNICO & EQUALIZAÇÃO DA CLOUD ARENA [CONCLUÍDA & HOMOLOGADA]

Durante os testes de observabilidade do Web Dashboard, identificou-se que o modelo `Claude 3.7 Sonnet` figurava incorretamente como Quality Leader na Cloud Arena. Uma auditoria profunda revelou distorções matemáticas de escalas e presença de sementes estáticas antigas.

### 17.1 Diagnóstico da Causa-Raiz (O Bug da Escala 10-60 vs LMSYS Elo)
1. **Conflito de Escalas em `scoring.rs`:**
   - Modelos contemporâneos de 2026 (`anthropic/claude-opus-5.5` [57.6], `anthropic/claude-sonnet-5.5` [56.0], `google/gemini-4-argon` [52.6]) utilizam o **Artificial Analysis Intelligence Index**, cuja escala global do estado da arte varia entre **10 e 60**.
   - A rotina antiga de normalização em `scoring.rs` (`normalize_score_or_elo`) tratava cegamente valores $\le 100.0$ como percentuais diretos (`val / 1.0`), atribuindo scores de apenas 52% a 57% aos modelos mais inteligentes do mundo.
   - Em contrapartida, o registro legado de `claude-3-7-sonnet` possuía um Elo antigo (1420.0). Por ser $> 100$, caía na conversão de Elo $(1420 - 1000) / 450 \times 100 = 93.3\%$. Isso criava a ilusão matemática de que o modelo antigo superava a fronteira contemporânea.
2. **Sementes Legadas Re-injetando Modelos Obsoletos:**
   - O arquivo `seed_aliases.rs` re-inseria a cada sincronização aliases sem namespace de 2024 (`claude-3-7-sonnet`, `claude-3-5-sonnet`, `deepseek-v3`, etc.), poluindo o catálogo canônico.

### 17.2 Solução Matemática e Expurgo Definitivo
1. **Calibração de Escalas em `crates/souls_llm_cloud_arena/src/scoring.rs`:**
   - Mapeamento linear rigoroso do índice Artificial Analysis $[10.0, 60.0] \to [0.0, 100.0]$:
     - Score $57.6 \to 95.2$
     - Score $56.0 \to 92.0$
     - Score $52.6 \to 85.2$
   - Preservação direta de benchmarks percentuais reais quando $> 65.0$ (ex: SWE-bench, BFCL).
   - Calibração do LMSYS Arena Elo para o intervalo realista da fronteira $[1200.0, 1550.0]$.
2. **Expurgo e Substituição de Sementes:**
   - Sementes de `seed_aliases.rs` atualizadas para a fronteira real (`anthropic/claude-opus-5.5`, `anthropic/claude-sonnet-5.5`, `google/gemini-4-argon`, `openai/gpt-6-astra`, `deepseek/deepseek-v4.1-flash`, etc.).
   - Executado expurgo via SQLite dos 12 IDs legados no banco `souls_heuristic_vault.db`.
3. **Ingestão Massiva dos 335 Modelos Canônicos Reais:**
   - Sincronização executada via `souls_cloud_arena sync --force`: **335 modelos canônicos do Nous Portal**, **797 tarifas do OpenRouter**, **335 benchmarks** e **1.675 avaliações ponderadas** persistidas no SQLite STRICT WAL.
   - Relatório analítico oficial regenerado em [docs/observability/reports/souls_cloud_llms_inventory_summary.md](file:///z:/souls_engine/docs/observability/reports/souls_cloud_llms_inventory_summary.md).
   - **Novo Pódio de Campeões Homologado:**
     - **Coding & SWE:** 🏆 **Google: Gemini 4 Argon** (Score: 95.21 | $2.84/M) e 💎 **DeepSeek V4 Flash Vision Exp** (Score: 88.75 | $0.22/M).
     - **Deep Reasoning:** 🏆 **Google: Gemini 4 Argon** (Score: 95.21 | $4.82/M) e 💎 **Qwen 3.8 Max (0902)** (Score: 88.84 | $3.25/M).
     - **Agentic Core:** 🏆 **Google: Gemini 4 Argon** (Score: 94.26 | $1.66/M) e 💎 **DeepSeek V4 Flash Vision Exp** (Score: 83.57 | $0.13/M).
     - **Fast Triage:** 🏆 **DeepSeek V4 Flash Vision Exp** (Score: 93.15 | $0.24/M) e 💎 **inclusionAI: Ling 3.0 Flash VL** (Score: 91.35 | $0.07/M).
     - **Design & UI:** 🏆 **DeepSeek V4 Flash Vision Exp** (Score: 85.86 | $0.28/M) e 💎 **inclusionAI: Ling 3.0 Flash VL** (Score: 82.56 | $0.08/M).
   - Reconstrução release de `souls_server.exe` / `souls_engine.exe` com binário atualizado e daemon respondendo em `127.0.0.1:9123/api/v1/cloud/champions`.

---

## 18. MATRIZ DE PENDÊNCIAS RESTANTES & BACKLOG PARA PRÓXIMAS SESSÕES

Com a base bare-metal e a Cloud Arena estabilizadas, este quadro elenca com precisão cirúrgica o que resta realizar, categorizado por necessidade de criação do zero, transplante ou refinamento:

| Componente / Funcionalidade | Natureza da Tarefa | Status Atual | Crate Envolvida | Prioridade / Próxima Sessão |
| :--- | :--- | :--- | :--- | :--- |
| **Crate `souls_llm_inventory` (RFC-007)** | **Criação Nova & Desacoplamento** | Proposta e especificada em `RFC_SOULS_LLM_INVENTORY.md`; varredura básica reside hoje em `souls_llm_local_arena` | `crates/souls_llm_inventory` | **Sessão Próxima A (Hub & Downloads)** |
| **Crate `souls_anthropophagy`** | **Transplante Monográfico** | Placeholder (`add(2, 2)`); 100+ arquivos preservados no doador `_donor/` | `crates/souls_anthropophagy` | **Sessão Próxima B (Monográfica Dedicada)** |
| **Cockpit Integrador de Modelos no Dashboard** | **Refinamento de Frontend** | APIs REST `/champions` e `/telemetry` prontas; falta card unificado no topo com seleção Local vs Cloud Default/Fallback | Web Dashboard (`crates/souls_server`) | **Sessão Próxima C (UX & Presets)** |
| **Seletor de Presets do Hermes Agent** | **Refinamento de UX** | Handler seguro de `config.yaml` pronto; falta seletor em 1-clique na interface visual | Web Dashboard / REST | **Sessão Próxima C (UX & Presets)** |
| **Instalador Gráfico Inno Setup** | **Empacotamento de Produto** | Manifesto `.iss` pronto; falta compilar o binário `.exe` com assistente guiado | Ferramentas / `installer/` | **Sessão Final (Deploy)** |

### 18.1 Detalhamento da Crate `souls_llm_inventory` (RFC-007)
A ser criada formalmente para cumprir os 4 papéis de gestão de pesos:
1. **Desacoplamento do Runtime:** Separar a gestão de arquivos em disco (`.gguf`, `.onnx`) do runtime de execução bare-metal (`souls_inference_runtime`).
2. **Download Manager Assíncrono com SSE:** Implementar download com suporte a HTTP Range (pausa e retomada sem corrupção), validação de hash SHA-256 e emissão de eventos SSE de progresso para o Web Dashboard.
3. **Profiling de Silício Adaptativo:** Inspeção dinâmica da VRAM livre (NVML) cruzada com os tensores do modelo, classificando em:
   - `OptimalVram`: Cabe 100% na dGPU RTX 2060m sob o teto WDDM (40-50 tok/s).
   - `PartialCpuOffload`: Offload parcial para CPU Intel i9 com instruções AVX2 (15-25 tok/s).
   - `IncompatibleSpillover`: Alerta de risco de barramento PCIe (< 2 tok/s).
4. **Universalidade do Armazenamento:** Uso canônico de `%USERPROFILE%\llm_local_models` com criação idempotente e tolerância a diretório vazio.

### 18.2 Detalhamento de `souls_anthropophagy` (Transplante Monográfico)
A ser transplantada em sessão exclusiva para preservar a Regra de Crate Única:
1. **Harvester & 11 Linters SAST Poliglotas:** Ingestão de Bandit, Biome, Clippy, Cppcheck, Govulncheck, Oxc, Ruff, Sobelow.
2. **Firewall SPDX de Licenças:** Detecção de licenças permissivas (MIT, Apache-2.0, BSD) e bloqueio de licenças virais (GPL/AGPL).
3. **The Organ Vault:** Catálogo SQLite para indexação de funções, structs e métodos desacoplados.
4. **Dissector & Slicer Sintático:** Amputação e reconstituição de código via Tree-sitter.

---

## 19. PROPOSTA ARQUITETURAL & FINOPS: DYNAMIC REASONING EFFORT & TASK-ADAPTIVE THINKING BUDGET

> **NOTA DE PROJETO (IDEIA DO ARQUITETO):**
> *"Sabe quando tem o 'reasoning'/'effort' do modelo (minimum, low, medium, high, max)? Como isso pode ser dinâmico? E como impacta o output e o valor final da chamada? Como equilibrar para ter a melhor qualidade no menor preço on-the-go entre sessões no Hermes Agent?"*

### 19.1 O Dilema FinOps dos Thinking Tokens
Modelos contemporâneos de raciocínio profundo (famílias OpenAI `o-series`, Google `Gemini Thinking`, Anthropic `Claude Thinking`, DeepSeek `R1`) introduziram o parâmetro de **esforço de raciocínio** (`reasoning_effort`: `minimum`, `low`, `medium`, `high`, `max` ou `thinking_budget_tokens` de 1.024 a 32.768):

1. **A Assimetria de Custo (The Reasoning Tax):**
   - Os tokens gerados na cadeia de pensamento interna (*thinking/reasoning tokens*) são cobrados integralmente como **tokens de saída (output tokens)**.
   - Na maioria dos provedores, os tokens de saída custam de **3x a 5x mais caro** que os tokens de entrada.
   - Uma chamada com `reasoning_effort: max` pode queimar entre 16.000 e 32.000 tokens invisíveis antes de produzir uma única linha de resposta visível. Uma interação que custaria $0.002 pode saltar para $0.150!
2. **Impacto na Latência (TTFT e Latência Total):**
   - Um esforço `high`/`max` faz o Time To First Token (TTFT) disparar de 1.2s para 15s - 30s. Para micro-tarefas ágeis de triagem ou preenchimento de campos, isso colapsa a ergonomia do agente.
3. **A Armadilha do Super-Raciocínio (Overthinking Trap):**
   - Para tarefas de baixa entropia cognitiva (ex: formatação de JSON, extração de entidades, docstrings, consultas sintáticas), reasoning alto frequentemente degrada o resultado: o modelo delibera em círculos, inventa exceções teóricas inexistentes e alucina complexidade onde havia simplicidade.
   - Em contrapartida, tarefas de alta entropia (provas matemáticas, verificação formal de segurança, arquitetura de concorrência multithread) falham catastroficamente se executadas com `reasoning_effort: minimum`.

### 19.2 Mecânica de Calibração Dinâmica Inter-Sessão no Hermes Agent

Como o Hermes Agent opera em loop de orquestração com o Souls Engine, a modulação de esforço de raciocínio pode ser automatizada de forma elegante e transparente:

```mermaid
flowchart TD
    UserPrompt["Prompt do Usuário / Tarefa do Hermes"] --> IntentProbe["souls_intent & souls_route_subtask\n(Tier 0.5 SmolLM2 no i9 AVX2 - < 2ms)"]
    IntentProbe --> EntropyCalc["Cálculo de Entropia de Shannon (H)\n& Complexidade Sintática AST"]

    EntropyCalc -->|H < 0.35 (Trivial/Procedural)| EffortMin["Reasoning: MINIMUM / LOW\nBudget: 1.024 tokens\nEconomia: ~90% em Output"]
    EntropyCalc -->|0.35 <= H <= 0.70 (Standard SWE)| EffortMed["Reasoning: MEDIUM\nBudget: 4.096 tokens\nEquilíbrio Qualidade/Preço"]
    EntropyCalc -->|H > 0.70 (Alta Entropia/Formal)| EffortHigh["Reasoning: HIGH / MAX\nBudget: 16.384+ tokens\nResolução Matemática/Segurança"]

    EffortMin --> Dispatch["Despacho no Hermes Agent\n(Parametrização Dinâmica da Chamada)"]
    EffortMed --> Dispatch
    EffortHigh --> Dispatch
```

#### Níveis de Modulação Dinâmica Propostos:
1. **Nível 1 — Inter-Session Dynamic Calibration (Mudança entre Sessões):**
   - Antes de iniciar uma nova sessão ou ao receber um novo prompt mestre, o Hermes Agent envia a intenção para o `souls_intent`.
   - O Souls Engine analisa a entropia e retorna a recomendação formal de modelo e de `recommended_reasoning_effort`.
   - O Hermes Agent ajusta seus parâmetros de cliente HTTP para aquela sessão específica.
2. **Nível 2 — Dynamic Subagent Dispatch (Chaveamento Dinâmico em Subagentes):**
   - Quando o Hermes Agent aciona `delegate_task`, ele obrigatoriamente consulta `souls_route_subtask`.
   - A resposta JSON do Souls Engine passa a fornecer o DTO estendido:
     ```json
     {
       "canonical_id": "google/gemini-4-argon",
       "provider": "nous_portal",
       "recommended_reasoning_effort": "low",
       "thinking_budget_tokens": 2048,
       "rationale": "Tarefa procedural de fatiamento de código: entropia 0.28 dispensa deliberação profunda."
     }
     ```
   - O subagente nasce configurado com o budget ideal, prevenindo desperdício orçamentário.
3. **Nível 3 — Dynamic Fallback Escalation (Escalonamento por Falha):**
   - Se um subagente rodando em `reasoning: low` falhar em passar nos testes de verificação sintática, o Souls Engine escalona automaticamente a próxima tentativa para `reasoning: high` na mesma sessão, aplicando deliberação profunda apenas como remédio corretivo.

### 19.3 Próximos Passos de Pesquisa & Implementação
- [ ] Adicionar os campos `reasoning_effort_levels` e `thinking_token_pricing` na tabela `pricing_telemetry_snapshots` do SQLite.
- [ ] Adicionar o campo `recommended_reasoning_effort` no retorno da ferramenta MCP `souls_route_subtask`.
- [ ] Conectar o medidor de entropia de Shannon de `souls_inference_runtime` à decisão de budget.

---

## 20. GUIA PRÁTICO DE HOMOLOGAÇÃO IMEDIATA DO HERMES AGENT (O PRIMEIRO VOO SIMBIÓTICO)

Para o Arquiteto Humano que deseja operar o sistema imediatamente com o Hermes Agent:

### 20.1 Garantias de Confiabilidade do Harness Atual
* **Servidor Headless:** Daemon `souls_engine.exe` ativo em `127.0.0.1:9123` com proteção Single-Instance e detecção assíncrona de colisões TCP.
* **Integridade Epistêmica:** FrankenSQLite STRICT WAL em ReFS, imune a corrupção por encerramentos forçados (`taskkill`).
* **Catálogo MCP:** 52 ferramentas ativas e homologadas (156/156 cenários aprovados).
* **Cloud Arena:** 335 modelos reais do Nous Portal e OpenRouter com scores equalizados contemporâneos.
* **Reflexos Éticos e FinOps:** 15 Skills sincronizadas em `%LOCALAPPDATA%\hermes\skills\` e `.agents\skills\`.
* **Memória Fria do Obsidian:** 498 notas mapeadas no `wiki/raw_manifest.json` com integridade SHA-256 e blindagem de privacidade `LOCAL_ONLY`.

### 20.2 Roteiro do Primeiro Voo (Checklist de Validação)
1. **Conexão:** Inicie o Hermes Agent Desktop (ou terminal). Ele detectará automaticamente o bloco `mcp_servers.souls_engine` configurado em `%LOCALAPPDATA%\hermes\config.yaml`.
2. **Telemetria de Silício:** Pergunte ao Hermes sobre a GPU e VRAM livre (ele invocará a ferramenta MCP `metrics`).
3. **Deliberação Socrática:** Solicite uma análise arquitetural de um módulo (ele invocará a ferramenta MCP `thinking`).
4. **Cirurgia Sintática:** Peça para ele inspecionar uma função (ele obedecerá à skill `souls-code-surgeon`, usando `outline` e `symbol` sem ler arquivos inteiros).
5. **Auto-Aprendizado (`/learn`):** Diante de qualquer correção ou novo padrão, comande `/learn` para que o Hermes ative a skill `agent-self-learning-loop` e persista o aprendizado permanentemente.
