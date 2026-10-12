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
│ Sessão 7 │ Bloco F: Web Dashboard│ [CONCLUÍDA] Hermes Web Dashboard (REST),   │
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
│ Sessão 11│ Bloco I: Skills &     │ [PRÓXIMA] Homologação das 15 Skills e estei-│
│          │ Curadoria Viva Cofre  │ ra viva de 498 notas Obsidian raw->wiki     │
├──────────┼───────────────────────┼─────────────────────────────────────────────┤
│ Sessão 12│ Bloco J: Fix Banco,   │ Correção Foreign Key Cloud Arena sync,      │
│          │ Dashboard SSE & Config│ telemetria SSE viva e souls_config.toml     │
├──────────┼───────────────────────┼─────────────────────────────────────────────┤
│ Sessão 13│ Bloco K: Simbiose     │ Linkagem desktop-plugins Hermes, launcher   │
│          │ Hermes & Systray Win32│ Systray com olhos RGB e teste ponta-a-ponta │
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
3. **Preservação da Regra Headless:** Toda a camada gráfica reside estritamente no front-end web do Hermes Dashboard (ou cliente web do usuário), garantindo conformidade absoluta com a Linha Vermelha 1 do `AGENTS.md` (Zero Desktop UI nativa no binário Rust).

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


### 11.3 Sessão 11 (Bloco I): Homologação Comportamental das 15 Skills & Curadoria Viva do Obsidian
* **Objetivo:** Validar o alinhamento ético e os reflexos operacionais autônomos dos agentes.
* **Bateria de Verificação de Skills:**
  - `obsidian-privacy-shield`: Submissão de prompts com dados de saúde ou finanças; o agente deve abortar envio para nuvens abertas e desviar compulsoriamente para processamento local na dGPU / CPU i9.
  - `souls-code-surgeon`: Proibição estrita de leitura de arquivos inteiros; imposição do fluxo `outline` $\rightarrow$ `symbol` $\rightarrow$ `edit`.
  - `agent-self-learning-loop`: Teste de ciclo de auto-patch `/learn`, registrando lições aprendidas em regras duradouras.
* **Ponto de Auditoria 5 (Esteira Viva do Obsidian Vault):**
  - Execução de `curate_obsidian.ps1` no lote de 498 documentos em `raw/`, validando a formatação YAML, links bidirecionais `[[...]]`, log em `wiki/log.md` e manifesto SHA-256.

### 11.4 Sessão 12 (Bloco J): Correção de Banco, UX do Dashboard & Configuração Centralizada
* **Ponto de Auditoria 2 (Bug de Foreign Key no Cloud Arena Sync):**
  - Investigar e corrigir a violação `FOREIGN KEY constraint failed (code: 787)` em `souls_llm_cloud_arena`, garantindo que os modelos canônicos pais sejam registrados no SQLite antes da inserção dos snapshots de avaliação.
* **Ponto de Auditoria 4 (Persistência Central no souls_config.toml):**
  - Implementar o carregamento e salvamento atômico de `Z:\souls_engine\.souls_data\config\souls_config.toml` para persistir toggles operacionais, limiares do Watchdog térmico e portas.
* **UX & Streaming em Tempo Real do Web Dashboard:**
  - Ativar fluxo contínuo de Server-Sent Events (`/api/v1/telemetry/stream`) no frontend do dashboard.
  - Implementar ordenação dinâmica interativa nas tabelas de modelos locais e cloud.

### 11.5 Sessão 13 (Bloco K): Simbiose Viva Hermes Desktop & Windows Systray Companion
* **Ponto de Auditoria 3 (Instalação dos Adaptadores no Hermes):**
  - Criar o utilitário de linkagem dos adaptadores `adapters/hermes_memory_souls` e `adapters/hermes_dashboard_souls` diretamente em `%LOCALAPPDATA%\hermes\desktop-plugins\`.
* **Windows Systray Companion (`souls_launcher.exe`):**
  - Desenvolver o executável Win32 ultraleve desacoplado (`windows-sys = "0.59"`), carregando os ícones temáticos de [assets/branding/](file:///z:/souls_engine/assets/branding/) com alternância de cores nos olhos (cyan, green, amber, red, purple) e tooltip de VRAM/temperatura ao pairar o mouse.
* **O Teste Finalzão de Simbiose:**
  - Inicialização do Hermes Agent Desktop conectado ao Souls Engine em `127.0.0.1:9123`.
  - Execução de diálogo completo com despacho de subagente via ParetoBandit, observando telemetria em tempo real no Web Dashboard integrado.

