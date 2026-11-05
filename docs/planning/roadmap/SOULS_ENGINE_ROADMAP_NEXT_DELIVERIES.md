# SOULS_ENGINE_ROADMAP_NEXT_DELIVERIES.md — Roadmap Canônico de Entregas & Evolução Rumo à v1.0

> **STATUS DO DOCUMENTO:** CANÔNICO & VINCULANTE (DOCS-AS-GUARDRAILS)  
> **VERSÃO CONSOLIDADA:** v0.5.0 (Novembro/2026) ➔ Meta Final: v1.0.0 Real  
> **ARQUITETURA:** 11 Crates Canônicas Bare-Metal (Rust/Tokio) • Loopback HTTP/SSE `127.0.0.1:9123`  
> **SIMBIOSE:** Hermes Agent Desktop (Python 3.11) + Antigravity CLI + Web Dashboard SPA  
> **SSOT DE DADOS:** `Z:\souls_engine\.souls_data\db\souls_heuristic_vault.db` (FrankenSQLite STRICT WAL)

---

## 1. VISÃO GERAL & PRINCÍPIOS ARQUITETURAIS VINCULANTES

O **Souls Engine** opera como uma infraestrutura bare-metal headless (Rust/Tokio) em loopback TCP local `127.0.0.1:9123`. Ele atua como acelerador de silício, motor de AST, hipocampo de memória L3 e orquestrador de subagentes para o **Hermes Agent** e IDEs compatíveis (Antigravity, Cursor).

```mermaid
flowchart TD
    subgraph HermesAgent ["Hermes Agent Desktop (Python 3.11)"]
        MasterChat["Chat Interativo Mestre (Nuvem Direta)"]
        Delegator["Orquestrador de Subagentes (delegate_task)"]
    end

    subgraph SoulsEngineDaemon ["Souls Engine Daemon (:9123 Headless Bare-Metal)"]
        MCPTrans["Transporte MCP (JSON-RPC 2.0 / Streamable HTTP & SSE)"]
        Router["ParetoBandit & model_task_profiles"]
        Sessions["Gerenciador de Sessões L3 (handoff / sub_agent)"]
        Inference["souls_inference_runtime (llama.cpp / NVML Watchdog)"]
        Memory["souls_memory (SQLite STRICT / LanceDB / LadybugDB)"]
        Inventory["souls_llm_inventory (Model Hub / Hardware Profiler)"]
        Core["souls_core (LEAN / CCR / Sandbox / DuckDuckGo Lite)"]
        AST["souls_ast (Tree-sitter / Myers Diff / gitoxide)"]
    end

    subgraph HardwareHost ["Silício Físico (Host Windows 11)"]
        dGPU["NVIDIA GeForce RTX 2060m (6GB VRAM / Teto WDDM 5.2GB)"]
        HostCPU["Intel Core i9 (8C/16T, AVX2, 32GB RAM DDR4)"]
    end

    MasterChat -->|SSE Direto sem Proxy L7| CloudAPI["Anthropic / OpenAI / DeepSeek / Google"]
    Delegator -->|1. Consulta souls_route_subtask| MCPTrans
    MCPTrans --> Router
    Router -->|Retorna Modelo Ótimo & Config| Delegator
    Delegator -->|2. Inicia Subagente com Modelo Prescrito| Sessions
    Delegator -->|3. Inferência Local /v1/chat/completions| Inference
    Sessions --> Memory
    Inference --> dGPU
    Inference --> HostCPU
    MCPTrans --> AST
    MCPTrans --> Core
```

### As 6 Linhas Vermelhas (AGENTS.md)
1. **Zero Desktop UI Local / Systray:** O binário `souls_server.exe` / `souls_engine.exe` é 100% headless. A observabilidade visual pertence ao Web Dashboard SPA servido na porta `9123`.
2. **Nenhum Proxy L7 para o Chat Mestre:** O Hermes Agent resolve suas APIs de nuvem diretamente, preservando o *prefix caching*.
3. **Nenhum Chaveamento Intradiálogo:** O ParetoBandit atua exclusivamente em subagentes (`delegate_task`) e tarefas de suporte (`auxiliary.*`).
4. **Sem Poluição fora do Dev Drive ReFS (`Z:\`):** Toda persistência reside em `Z:\souls_engine\.souls_data\`.
5. **Topologia Controlada:** Workspace estruturado estritamente nas crates canônicas homologadas.
6. **Zero `unsafe` Arbitrário:** Workspace sob `#![forbid(unsafe_code)]`, exceto delimitações auditadas com `// SAFETY:`.

---

## 2. HISTÓRICO CONSOLIDADO DE ENTREGAS (O QUE JÁ FOI REALIZADO)

Até a consolidação da **Milestone v0.5.0**, todo o ciclo fundacional de infraestrutura, inferência, governança e interface foi construído e homologado:

### 2.1 Tabela Histórica de Sessões e Marcos (v0.1.0 ➔ v0.5.0)

| Sessão / Marco | Escopo Principal | Estado | Principais Entregas |
| :--- | :--- | :---: | :--- |
| **Sessões 1 & 2** | Daemon Base & Cérebro L3 | **100% CONCLUÍDO** | `souls_server` (:9123), Axum HTTP/SSE, FrankenSQLite STRICT WAL, LanceDB e LadybugDB. |
| **Sessão 3** | Conexão Viva Hermes Desktop | **100% CONCLUÍDO** | Configuração do `config.yaml`, junction ReFS de modelos, FinOps guardrails e MCP live. |
| **Sessão 4** | AST Tree-sitter & NVML | **100% CONCLUÍDO** | Transplante `souls_ast` (outlines, slices, callers), watchdog NVML bare-metal e cálculo EWMA. |
| **Sessão 5** | Cloud Arena Benchmarking | **100% CONCLUÍDO** | Crate `souls_llm_cloud_arena`, 9 perfis cognitivos, ingestão Nous/OpenRouter e sync diário. |
| **Sessão 6** | Governança Master Brain | **100% CONCLUÍDO** | Tríade de Líderes na nuvem, cálculo de Fronteira de Pareto e alternância de perfis. |
| **Sessão 7** | Web Dashboard SPA (V1) | **100% CONCLUÍDO** | Interface web inicial servida via Axum com abas de Overview, Arenas e Settings globais. |
| **Sessão 8** | Homologação Clínica E2E | **100% CONCLUÍDO** | 156/156 cenários clínicos aprovados (100%), 52 tools, 15 skills, zero leak de VRAM. |
| **Sessão 9 & 10** | Silício Neural & Des-staging | **100% CONCLUÍDO** | 15 modelos físicos mapeados, sandbox Win32 segura, LEAN compression e dedup SipHash. |
| **Sessão 11** | Skills & Curadoria Obsidian | **100% CONCLUÍDO** | 15 Skills sincronizadas, pipeline de curadoria `raw/` ➔ `wiki/` com Privacy Shield local. |
| **Sessão 12 & 13** | Telemetria SSE & Simbiose | **100% CONCLUÍDO** | Streaming SSE em tempo real, persistência em `souls_config.toml` e integração de plugins Hermes. |
| **Sessão 14 & 15** | Settings UX & Design System | **100% CONCLUÍDO** | Sub-abas com tooltips explicativos, modal de aplicação rápida de modelos e ranking Top 50. |
| **Sessão 16** | Context LEAN & Sandbox | **100% CONCLUÍDO** | Crate `souls_core` (DuckDuckGo Lite na RAM, truncagem head+tail 50/70, 65/65 testes PASS). |
| **Sessão 17** | Model Hub & Profiler | **100% CONCLUÍDO** | Crate `souls_llm_inventory` (RFC-007), download sequencial com HTTP Range e verificação SHA-256. |
| **Sessão 18** | Local Silicon Arena | **100% CONCLUÍDO** | Crate `souls_llm_local_arena`, 5 baterias empíricas ADR-007 no silício real (32/32 testes PASS). |
| **Sessão 19** | Two-Stage Switchboard | **100% CONCLUÍDO** | Switchboard em 2 estágios (<1.7ms na CPU AVX2), projeção dinâmica no `tools/list` (33/33 PASS). |
| **Milestone v0.5.0** | **Modularização UI & SSOT Rust** | **100% CONCLUÍDO** | **ADR-012:** Monólito de 15k linhas decomposto em 25 componentes em `templates/` via `concat!(include_str!(...))` em `.rodata`. Unificação de dependências no root `Cargo.toml`. Teto de contexto dGPU dinâmico baseado em VRAM real. Revisão total das sub-abas de Settings (zero mocks). |

### 2.2 Inventário de Ativos Consolidados na v0.5.0
* **Topologia de Crates:** 11 Crates canônicas estritamente sincronizadas via `[workspace.dependencies]`.
* **Catálogo MCP:** 55 Ferramentas de revelação progressiva divididas em 6 namespaces (`code.*`, `context.*`, `memory.*`, `think.*`, `ops.*`, `sys.*`).
* **Matriz de Skills:** 15 Skills canônicas procedimentais ativas em `.agents/skills` e `%LOCALAPPDATA%\hermes\skills`.
* **Desempenho de Compilação:** `cargo check -p souls_server` acelerado de ~8.7s para **2.06s** (~4.2x mais rápido).
* **Binário de Produção:** `souls_engine.exe` compilado em release MSVC (`lto = "thin"`, `codegen-units = 1`, `panic = "abort"`), pesando menos de 45 MB com zero dependências de DLLs externas.

---

## 3. MATRIZ DE DEMANDAS, SCRATCHPAD & ANÁLISE DE GAPS

Cruzamento do draft `.souls_scratchpad/lembretes.txt` com o estado do repositório:

| Item do Lembrete | Descrição Sintética | Estado Atual | Destino no Roadmap |
| :--- | :--- | :---: | :--- |
| **#1 Refatoração de UI** | Quebrar monólito HTML em módulos para manutenção ágil | **JÁ REALIZADO (v0.5.0)** | Concluído via ADR-012 (25 parciais atômicos sob `templates/`). |
| **#2 Revisão de Deps Rust** | Unificar versões e auditar flags de compilação | **JÁ REALIZADO (v0.5.0)** | Concluído via SSOT em `[workspace.dependencies]`. |
| **#3 Skills Hub & Forge** | Visualização, descoberta, busca em APIs e criação agêntica local de skills | **GAP IDENTIFICADO** | **Milestone v0.6.0** (Aba dedicada + Agêntico Forge). |
| **#4 Perfis do Hermes** | Inspecionar/editar `AGENTS.md`, `SOUL.md` e configs dos perfis no Dashboard | **GAP IDENTIFICADO** | **Milestone v0.6.0** (Hermes Personas & Governance Inspector). |
| **#5 Deep Research Engine** | Capability/Tool base de pesquisa profunda em árvore com síntese e fontes | **GAP IDENTIFICADO** | **Milestone v0.7.0** (Deep Research Nativo & Subagente). |
| **#6 Gateway Cloud Multi-Provedor** | Universalizar despacho de subagentes para além de OpenAI/Claude/Gemini | **GAP IDENTIFICADO** | **Milestone v0.7.0** (DeepSeek API, Groq, Together, Bedrock). |
| **#7 Google Universe Bridge** | Ferramentas MCP para Google Docs, Sheets, Drive, Calendar e Gmail | **GAP IDENTIFICADO** | **Milestone v0.8.0** (Google Universe MCP Bridge). |
| **#8 & #10 Souls Anthropophagy 2.0** | Ingestão mecânica (Blobs) ➔ Essências ➔ Enxame (85+ colunas) ➔ Auto-Evolução | **GAP IDENTIFICADO** | **Milestone v0.9.0** (Pipeline em 4 camadas & Dashboard de Evolução). |
| **#9 Universal Installer & Hardware Adaptability** | Instalador zero-atrito, piso mínimo na 2060m adaptável a qualquer hardware | **GAP IDENTIFICADO** | **Milestone v1.0.0** (Universal Installer & Release Oficial). |

---

## 4. A TRILHA CANÔNICA RUMO À VERSÃO 1.0 REAL

A jornada da versão 0.5.0 até a 1.0.0 é dividida em **5 Milestones Sequenciais**, organizadas de forma lógica para evitar retrabalho e garantir que cada camada sirva de base sólida para a seguinte:

```
┌──────────────────────────────────────────────────────────────────────────────────┐
│                     JORNADA DE DESENVOLVIMENTO RUMO À v1.0                       │
├───────────────┬──────────────────────────────────┬───────────────────────────────┤
│ v0.5.0 (HOJE) │ Base Bare-Metal & UI Modular     │ [CONCLUÍDO] 11 Crates, ADR-012│
├───────────────┼──────────────────────────────────┼───────────────────────────────┤
│ v0.6.0        │ Governança Visual & Skills Hub   │ Skills Forge + Hermes Personas│
├───────────────┼──────────────────────────────────┼───────────────────────────────┤
│ v0.7.0        │ Cognição Profunda & Gateway Cloud│ Deep Research + Multi-Provider│
├───────────────┼──────────────────────────────────┼───────────────────────────────┤
│ v0.8.0        │ Conectividade & Integrações      │ Google Universe + Obsidian DB │
├───────────────┼──────────────────────────────────┼───────────────────────────────┤
│ v0.9.0        │ Souls Anthropophagy 2.0          │ 4 Camadas, Blobs & 85+ Colunas│
├───────────────┼──────────────────────────────────┼───────────────────────────────┤
│ v1.0.0 REAL   │ Universalização & Distribuição   │ Auto-Tuning, Installer 1-Click│
└───────────────┴──────────────────────────────────┴───────────────────────────────┘
```

---

### MILESTONE v0.6.0: GOVERNANÇA VISUAL, SKILLS HUB & HERMES PERSONAS

O objetivo desta etapa é transformar o Dashboard em uma central completa de gestão comportamental dos agentes e subagentes, sem depender de edição manual de arquivos no terminal.

#### 1. Módulo Skills Hub & Agêntico Forge (`templates/tabs/skills_hub.html`)
* **Catálogo Visual de Skills:** Listagem interativa das 15+ skills instaladas nos diretórios do sistema (`.agents/skills/`, `%LOCALAPPDATA%\hermes\skills/`, `~/.gemini/config/skills/`).
* **Visualizador de Metadados:** Exibição do nome, descrição, gatilhos de ativação, ferramentas MCP associadas e regras operacionais de cada `SKILL.md`.
* **Skills Marketplace & Explorer:** Interface de busca integrada para consultar repositórios de skills comunitárias (ex: Agent Marketplace, GitHub) via API do `souls_core`.
* **Skills Forge (Gerador Agêntico Local):** Assistente no dashboard que recebe uma necessidade em linguagem natural e utiliza o Tier 1 local ou subagente de nuvem para compor um novo `SKILL.md` padronizado, com validação sintática estrita antes da gravação.

#### 2. Hermes Personas & Governance Inspector (`templates/settings/subtab_personas.html`)
* **Inspeção de Perfis do Hermes:** Mapeamento dinâmico dos perfis existentes em `%LOCALAPPDATA%\hermes\profiles/` e perfil `default`.
* **Editor/Visualizador Seguro de `AGENTS.md` e `SOUL.md`:** Visualização formatada em Markdown com realce de sintaxe dos guardrails e identidade de cada persona, com controle de versão e diff antes da mutação.
* **Sincronização de Defaults:** Ajuste dos parâmetros de temperatura, max_tokens e restrições de cada persona com reflexo automático nas sessões do Hermes.

---

### MILESTONE v0.7.0: COGNIÇÃO PROFUNDA, DEEP RESEARCH & GATEWAY UNIVERSAL

Expansão das capacidades analíticas fundamentais e eliminação do aprisionamento em poucos provedores de nuvem.

#### 1. Deep Research Engine Nativo (`souls_core::research` & Ferramenta MCP `deep_research`)
* **Pesquisa Recursiva em Árvore (Graph of Queries):** Algoritmo que quebra uma pergunta complexa em sub-consultas investigativas, explora ramificações paralelas e aprofunda em nós de maior densidade factual.
* **Scraping Limpo & Zero-Bloat na RAM:** Utilização do parser `souls_core` (DuckDuckGo Lite + extração direta de HTML para Markdown desidratado), poupando mais de 80% da janela de contexto.
* **Extração Epistêmica no LadybugDB:** Entidades e fatos descobertos durante a pesquisa são persistidos no grafo ontológico com arestas causais (`relates_to`, `supports`, `refutes`).
* **Relatório Estruturado com Citações:** Síntese final com referências rastreáveis, grau de certeza e categorização por fontes primárias.

#### 2. Universal Cloud Dispatch Gateway para Subagentes (`souls_model_router` & `souls_llm_cloud_arena`)
* **Expansão Multi-Provedor Oficial:** Além de OpenAI, Anthropic e Google Gemini, suporte nativo direto com telemetria para:
  - **DeepSeek API:** Endpoints nativos v3 e r1 com cálculo real de cache hit/miss de tokens.
  - **Groq & Cerebras:** Despacho de ultra-baixa latência (300+ tokens/s) para tarefas de triagem rápida.
  - **Together AI & Mistral AI:** Modelos open-weights em nuvem com alta largura de banda.
  - **OpenRouter & Nous Portal:** Roteadores universais com balanceamento automático.
  - **AWS Bedrock / Azure OpenAI:** Conectores corporativos com autenticação IAM/Bearer.
* **Fallback Chain Inteligente por Perfil:** Se o provedor principal sofrer *rate limit* (HTTP 429) ou indisponibilidade (HTTP 500), o gateway chaveia automaticamente para o segundo colocado da Fronteira de Pareto com overhead inferior a 150ms.

---

### MILESTONE v0.8.0: CONECTIVIDADE GOOGLE UNIVERSE & CURADORIA CONTÍNUA

Abertura do ecossistema para a vida pessoal e profissional do operador, integrando suas fontes primárias de dados.

#### 1. Google Universe MCP Bridge (`souls_core::google_bridge`)
* **Camada de Ferramentas MCP Autenticadas:**
  - `gdocs.*`: Leitura, extração de texto, inclusão de seções e formatação em documentos do Google Docs.
  - `gsheets.*`: Consulta de dados tabulares, filtros, inserção de linhas e cálculos em planilhas do Google Sheets.
  - `gdrive.*`: Busca por nome/conteúdo, upload de relatórios gerados e organização de pastas.
  - `gcalendar.*`: Inspeção de agenda, detecção de conflitos de horário e agendamento de eventos.
  - `gmail.*`: Triagem de emails, resumo de threads e preparação de rascunhos (*drafts* imutáveis sem envio automático acidental).
* **Segurança & Privacy Guard:** Todas as chamadas passam pelo `PiiRedactor` local antes de transitar por modelos analíticos.

#### 2. Ativação Plena da Esteira de Curadoria Obsidian DB ("The 30-Minute Drip")
* **Orquestração Autônoma em Segundo Plano:** Execução do script `curate_obsidian.ps1` via cron a cada 30 minutos, destilando 2 a 3 notas de `raw/` para `wiki/` com frontmatter YAML e links bidirecionais `[[...]]`.
* **Blindagem `obsidian-privacy-shield`:** Higienização de documentos sensíveis (médicos e financeiros) no silício local antes de qualquer síntese em contexto longo.
* **Monitoramento no Dashboard:** Exibição do backlog restante (dos 498 arquivos) e log de auditoria em tempo real na sub-aba de conhecimento.

---

### MILESTONE v0.9.0: SOULS ANTHROPOPHAGY 2.0 & ENXAME DE AUTO-EVOLUÇÃO

Transformação do módulo de antropofagia em um pipeline estruturado de engenharia reversa contínua e absorção de avanços da comunidade open-source.

#### 1. O Pipeline Antropofágico em 4 Camadas
```
┌─────────────────────────────────────────────────────────────────────────────┐
│                 SOULS ANTHROPOPHAGY 2.0 — PIPELINE DE 4 CAMADAS             │
├─────────────────────────────────────────────────────────────────────────────┤
│ 1. Raw Blueprints & Blobs: Ingestão mecânica do repositório alvo (Git / AST) │
│    Extração de assinaturas, topologia de arquivos e dependências no SQLite. │
├──────────────────────────────────────┬──────────────────────────────────────┤
│                                      ▼
│ 2. Essences & Dehydration: Destilação semântica de algoritmos e padrões.     │
│    Remoção de ruído estrutural e redução de 85% de tokens via Tree-sitter.  │
├──────────────────────────────────────┬──────────────────────────────────────┤
│                                      ▼
│ 3. Swarm Critical Analysis: Enxame multiagente avaliando a Matriz de 85+ Colunas│
│    Mapeamento de maturidade, robustez, licença, compatibilidade e gaps reais│
├──────────────────────────────────────┬──────────────────────────────────────┤
│                                      ▼
│ 4. Continuous Evolution Pipeline: Disparo de Deep Analysis & Planos Cirúrgicos│
│    Geração de ADRs, contratos de transplante e PRs automáticos para o Souls.│
└─────────────────────────────────────────────────────────────────────────────┘
```

#### 2. Dashboard de Engenharia Reversa e Controle de Repositórios
* **Adição de Repositórios e Links:** Interface visual para submeter novos repositórios Git ou URLs de harnesses/runtimes de IA (ex: novos frameworks de agentes, motores de contexto, papers).
* **Tabela Ontológica de 85+ Colunas:** Visualização tabular no Dashboard com scores de arquitetura, concorrência, pegada de memória, licença e sinergia com o Souls Engine.
* **Mecanismo de Disparo de Deep Analysis:** Botão de 1-clique para ordenar ao enxame de subagentes uma análise aprofundada de um módulo doador, gerando a especificação formal de transplante (seguindo o padrão do `DONOR_EXTRACTION_MAP.md`).

---

### MILESTONE v1.0.0 REAL: UNIVERSALIZAÇÃO, AUTO-TUNING & DISTRIBUIÇÃO PÚBLICA

A consagração do Souls Engine como um produto final robusto, instalável em qualquer máquina Windows 11 com o mínimo de atrito.

#### 1. Universal Hardware Profiler & Adaptabilidade Dinâmica
* **Agnosticismo de Silício com Piso Homologado:**
  - **Piso Mínimo Canônico:** RTX 2060m (6GB VRAM), CPU i9 (8C/16T), 32GB RAM.
  - **Adaptação para Hardwares Superiores (RTX 3060, 4070, 4090, Apple Silicon/DirectML):** O motor aloca automaticamente modelos maiores (7B–14B) e orçamentos maiores de KV-cache (até 64k tokens) sem alterar uma linha de código.
  - **Adaptação para Hardwares Modestos ou Sem dGPU (CPU Only / Intel iGPU / AMD):** Detecção automática de ausência de VRAM e desvio gracioso de todos os Tiers para instruções AVX2/AVX-512 na CPU, utilizando quantizações ultra-leves (Q4_0, Q3_K_S) e orçamentos de contexto conservadores (4k–8k tokens).

#### 2. Zero-Friction Universal Installer (`setup.ps1` & Bootstrapper Nativo)
* **Requisito Único do Usuário:** Apenas ter o **Hermes Agent Desktop** instalado na máquina.
* **Automação Completa do Script de Instalação:**
  1. **Verificação de Ambiente:** Validação do Windows 11, arquitetura x64 e permissões.
  2. **Estrutura de Armazenamento:** Criação automática do diretório de dados `Z:\souls_engine\.souls_data\` ou `%LOCALAPPDATA%\SoulsEngine\data\` (com suporte a fallback NTFS caso Dev Drive ReFS não esteja configurado).
  3. **Symlinks & Junctions Transparentes:** Linkagem do catálogo de modelos `%LOCALAPPDATA%\hermes\models` e instalação dos plugins do Hermes (`hermes_dashboard_souls` e `hermes_memory_souls`).
  4. **Injeção de Configurações no Hermes:** Configuração idempotente do `config.yaml` do Hermes registrando o Souls Engine como servidor MCP SSE na porta `9123`.
  5. **Auto-Calibração de Primeiro Boot:** Execução do profiler de hardware, cálculo dos limites de silício e geração do `souls_config.toml` personalizado.
  6. **Launch & Onboarding:** Inicialização limpa do daemon em segundo plano e abertura automática do Web Dashboard na página de boas-vindas com tour guiado.

#### 3. Descomissionamento Total do Legado (`_donor/`)
* **Remoção Definitiva da Pasta `_donor/`:** Todo o código doador legado em quarentena é permanentemente deletado da árvore git, consolidando a independência arquitetural e autoria 100% nativa do Souls Engine v7.
* **Selo de Estabilidade Bare-Metal de Produção:** Binários assinados, documentação completa em formato de livro digital/wiki e pacote de release GitHub.

---

## 5. MAPA DE DEPENDÊNCIAS & ORDEM CRÍTICA DE EXECUÇÃO (DAG)

Para assegurar uma esteira de entrega limpa e sem regressões (em estrita obediência à Regra da Crate Única por Turno):

```mermaid
flowchart TD
    v050["v0.5.0: Base Consolidada (UI Modular + SSOT Rust)"]
    
    subgraph M060 ["Milestone v0.6.0 (Governança Visual & Agentes)"]
        SkillsHub["Skills Hub & Forge (souls_server / souls_core)"]
        Personas["Hermes Personas & AGENTS.md Inspector"]
    end

    subgraph M070 ["Milestone v0.7.0 (Cognição & Gateway)"]
        DeepResearch["Deep Research Nativo em Árvore"]
        MultiCloud["Universal Cloud Dispatch Gateway"]
    end

    subgraph M080 ["Milestone v0.8.0 (Conectividade & Memória)"]
        GoogleBridge["Google Universe MCP Bridge"]
        ObsidianDrip["Esteira Contínua The 30-Minute Drip"]
    end

    subgraph M090 ["Milestone v0.9.0 (Engenharia Reversa & Auto-Evolução)"]
        Anthropophagy["Souls Anthropophagy 2.0 (4 Camadas / 85+ Colunas)"]
        EvolutionDash["Dashboard de Análise Crítica e Auto-Harness"]
    end

    subgraph M100 ["Milestone v1.0.0 (Universalização & Distribuição)"]
        AutoTuning["Universal Hardware Profiler (Agnóstico)"]
        Installer["One-Click Installer (setup.ps1)"]
        DecomDonor["Descomissionamento Definitivo do _donor/"]
    end

    v050 --> SkillsHub
    v050 --> Personas
    SkillsHub --> DeepResearch
    Personas --> MultiCloud
    DeepResearch --> GoogleBridge
    MultiCloud --> GoogleBridge
    GoogleBridge --> ObsidianDrip
    ObsidianDrip --> Anthropophagy
    Anthropophagy --> EvolutionDash
    EvolutionDash --> AutoTuning
    AutoTuning --> Installer
    Installer --> DecomDonor
```

---

## 6. DEFINITION OF DONE (DoD) PARA A VERSÃO 1.0 REAL

O Souls Engine só será declarado na versão **1.0.0 Real** quando atender cumulativamente a:
1. **Instalador Zero-Atrito:** Instalação e calibração automática em um computador Windows 11 virgem apenas com Hermes Agent instalado, sem erros e sem intervenção manual em arquivos JSON/YAML/TOML.
2. **Auto-Adaptação de Silício:** Execução suave e sem pânico em no mínimo 3 cenários físicos:
   - GPU NVIDIA com VRAM limitada (RTX 2060m / 3050);
   - GPU NVIDIA topo de linha (RTX 3090 / 4080 / 4090);
   - Máquina sem GPU dedicada (CPU-only Intel/AMD com AVX2).
3. **Catálogo de 60+ MCP Tools:** 100% das ferramentas com validação de tipagem, revelação progressiva via Two-Stage Switchboard e zero impacto em prefix cache do modelo mestre.
4. **Isolamento de Memória & Privacidade:** Zero vazamento de PII pessoal de saúde/finanças para provedores de nuvem, atestado pelo `PiiRedactor` e testes E2E.
5. **Zero Monólitos e Zero Código Morto:** Código-fonte limpo, pasta `_donor/` extinta, documentação sincronizada em ADRs e cobertura de testes automatizados superior a 95% em todas as crates.
