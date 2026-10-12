# HERMES_DASHBOARD_AND_SETTINGS_SPECIFICATION.md — Especificação Arquitetural do Web Dashboard & Souls Settings (Sessão 7 Canônica)

> **Status:** APROVADO PARA DESENVOLVIMENTO  
> **Versão:** 7.0.0 Canônica  
> **Autor:** Engenheiro Principal Bare-Metal & Arquiteto Humano  
> **Data:** Outubro de 2026  
> **Domínio:** Observabilidade Visual, Governança de Silício, Arenas e Configurações Simbióticas  

---

## 1. OBJETIVO E ESCOPO

Este documento estabelece o contrato arquitetural, os modelos de dados, os fluxos de integração e a especificação de interface para o **Hermes Web Dashboard & Souls Settings** (Sessão 7 — Bloco F).

O objetivo primordial é entregar uma superfície de observabilidade de alto padrão visual (Dark Mode refinado) e um centro de controle ergonômico para o Arquiteto Humano, permitindo:
1. **Monitoramento Clínico de Silício:** Visualização em tempo real de VRAM (RTX 2060m com teto WDDM de 5.2GB), temperatura da dGPU, carga de CPU Intel Core i9 (AVX2) e métricas FinOps ($E^3$, economia em dólares e tokens processados).
2. **Navegação nas Arenas de Modelos:** Visualização unificada dos modelos do silício local (Tiers 0 a 4) e da Cloud Arena (Nous Portal vs. OpenRouter sob os 9 Perfis Cognitivos com custos e tags de Pareto).
3. **Painel de Governança & Settings:** Gerenciamento dos presets do modelo mestre do Hermes Agent (`config.yaml`), toggles liga/desliga de recursos, calibração de limiares do Hardware Watchdog e ações de manutenção com um clique.

---

## 2. AS LINHAS VERMELHAS ARQUITETURAIS & REQUISITOS NÃO-FUNCIONAIS

A implementação do Web Dashboard obedece rigorosamente às leis do `AGENTS.md`:

1. **Zero Desktop UI no Binário Rust (Linha Vermelha 1):** O binário `souls_server.exe` permanece **100% headless**. Não são introduzidas dependências de janelas nativas, Tauri, Wry, Winit ou systray no daemon. O servidor Axum expõe o frontend estático e as APIs REST exclusivamente em `http://127.0.0.1:9123/`.
2. **Arquitetura de Build Duplo (Decisão de Alinhamento):**
   * **Canal A (Navegador Convencional):** Aplicação Single Page Application (SPA) em HTML5/CSS3/JavaScript moderno servida nativamente pelo Axum no endpoint `http://127.0.0.1:9123/dashboard` (e `/`).
   * **Canal B (Aba de Plugin no Hermes Desktop):** Pacote integrado em `adapters/hermes_dashboard_souls/dashboard/` estruturado com manifesto oficial (`manifest.json` e `plugin.yaml`), permitindo que a interface seja montada nativamente como uma aba de plugin dentro do Hermes Web/Desktop.
3. **Estética Mandatória (Dark Mode de Alta Fidelidade):**
   * Paleta escura profunda (Slate/Zinc 950 como fundo `#090d16` / `#0f172a`, superfícies de cards em `#131c31` com bordas sutis `rgba(255, 255, 255, 0.08)`).
   * Acentos de destaque: Ciano Elétrico (`#06b6d4`) para telemetria/links, Âmbar Néon (`#f59e0b`) para alertas de hardware/Pareto, e Esmeralda Neon (`#10b981`) para saúde do daemon/economia.
   * Tipografia: `Inter`, `Outfit` ou fontes de sistema modernas com fonte monoespaçada (`Fira Code`, `JetBrains Mono`) para números, métricas e endereços.
   * Microinterações fluidas: Badges de status pulsantes, transições suaves entre abas, toasts instantâneos de feedback para ações de configuração.

---

## 3. ARQUITETURA DE INFORMAÇÃO & LAYOUT DE NAVEGAÇÃO

A interface é estruturada em uma **Barra Superior Global (Top Bar)** e **Três Abas Principais de Conteúdo**:

```
┌────────────────────────────────────────────────────────────────────────────────────────────────────────┐
│  🟢 SOULS ENGINE v7  |  PORT: 9123  |  VRAM: 1.8 / 5.2 GB (34%)  |  GPU: 54°C  |  CPU: 24 Threads       │
│  [⚡ Sync Cloud Arena]   [💾 Flush WAL]   [Preset Ativo: Balanced Daily ⚖️]                           │
├────────────────────────────────────────────────────────────────────────────────────────────────────────┤
│   [ 📊 1. OVERVIEW & SILÍCIO ]    │    [ ⚔️ 2. MODEL ARENAS ]    │    [ ⚙️ 3. SOULS SETTINGS ]         │
└────────────────────────────────────────────────────────────────────────────────────────────────────────┘
```

---

### 3.1 Barra Superior Global (Top Bar de Telemetria Contínua)
* **Identidade do Daemon:** Logotipo "Souls Engine v7" com badge de pulso verde indicando conectividade ativa em `127.0.0.1:9123`.
* **Mini Pílula de Silício:** Resumo contínuo de consumo de VRAM da RTX 2060m (ex.: `1.8 / 5.2 GB`), temperatura NVML (`54°C`) e threads de CPU do i9.
* **Ações Rápidas de Topo:**
  - Botão `⚡ Sync Cloud Arena`: Dispara sincronização em segundo plano via `POST /api/v1/cloud/sync`.
  - Botão `💾 Flush WAL`: Executa checkpoint atômico do FrankenSQLite via `POST /api/v1/system/checkpoint`.
  - Indicador do Preset Ativo do Hermes (ex.: `Balanced Daily ⚖️`).

---

### 3.2 Aba 1: Overview (Centro de Comando & Telemetria em Tempo Real)

A primeira aba compõe o painel executivo com 4 quadrantes analíticos:

#### Quadrante 1: Silício Bare-Metal & Hardware Watchdog
* **Gauge de VRAM (NVIDIA RTX 2060m):**
  - Barra de progresso com gradiente (Verde $\to$ Âmbar $\to$ Vermelho).
  - Demarcação visual clara do **Teto WDDM (5.2 GB)** e zona de perigo de Spillover PCIe.
* **Monitor Térmico da GPU:**
  - Temperatura em tempo real medida pelo NVML.
  - Limiar de alerta demarcado em $78^\circ\text{C}$ (acionamento do desvio para AVX2).
* **CPU Host (Intel Core i9):**
  - Utilização de threads (24 threads lógicas).
  - Indicador de suporte a instruções AVX2 / FMA.

#### Quadrante 2: FinOps & Economia de Silício ($E^3$ Score)
* **Métrica de Eficiência Econômica ($E^3$ Score):** Relação entre tarefas resolvidas localmente e custo evitado.
* **Tokens Processados Hoje:** Gráfico de divisão entre tokens locais (custo $0.00) vs. tokens de nuvem.
* **Economia Estimada em Dólares:** Total em USD economizado pela execução de modelos Tier 0 a 2 no silício local.
* **Taxa de Prefix Caching & Desidratação AST:** % de tokens economizados pela árvore sintática do `souls_ast`.

#### Quadrante 3: Hipocampo L2 & Armazenamento ReFS
* **FrankenSQLite STRICT WAL:** Status das tabelas `epistemic_memories`, `model_task_profiles` e tamanho do log WAL.
* **LanceDB (Vetorial):** Total de fragmentos e vetores indexados de memória de longo prazo.
* **LadybugDB (Grafo Ontológico):** Total de entidades clínicas e arestas tipadas (`depends_on`, `implements`, `conflicts_with`).
* **Memória Fria do Obsidian DB:** Status do manifesto de curadoria (`raw_manifest.json`) e contagem de notas curadas.

#### Quadrante 4: Atividade Agêntica & Subagentes
* **Subagentes em Execução:** Lista de tarefas ativas despachadas via `delegate_task`.
* **Histórico de Roteamento ParetoBandit:** Últimas microtarefas roteadas, modelo escolhido e justificativa de Pareto.

---

### 3.3 Aba 2: Model Arenas (Visualização Integrada Local & Cloud)

Esta aba unifica o inventário dos modelos de IA disponíveis:

#### Seção Superior: Seletor dos 9 Perfis Cognitivos Canônicos
Pílulas interativas com alternância instantânea:
1. `🎯 Executive / Brain`
2. `💻 Coding / SWE`
3. `🤖 Agentic / Tools`
4. `🧠 Reasoning / Math`
5. `⚡ Fast / Triage`
6. `🎨 Design / Frontend`
7. `📚 Research / Docs`
8. `🛡️ Security / Audit`
9. `📝 Curation / Knowledge`

#### Tabela da Cloud Arena (Nous Portal vs. OpenRouter)
Para o perfil selecionado, a tabela exibe:
* **Modelo & Provedor:** Nome do modelo e endpoint (ex.: `deepseek/deepseek-chat`, `anthropic/claude-3-7-sonnet`).
* **Tarifas $/1M Tokens:** Custo de Entrada (Input) e Saída (Output).
* **Pontuação Ponderada do Perfil (Score 0-100):** Cálculo consolidado da matriz analítica.
* **Custo por Tarefa Padronizada ($C_{\text{task}}$):** Baseado na métrica canônica de 4k tokens input / 1k output.
* **Valor de Silício Amortecido ($V_{\text{task}}$):** Retorno empírico de qualidade.
* **Status da Fronteira de Pareto (Badges Visuais):**
  - `🏆 Pareto Champion` (Líder ótimo da fronteira)
  - `💎 Best Value` (Melhor relação custo-benefício)
  - `⚡ Ultra Low Cost` (Alternativa mais barata viável)

#### Tabela da Local Arena (Silício dGPU & CPU Host)
Abaixo ou em aba lateral, o inventário dos modelos locais:
* **Modelo & Tier:** Tier 0 (`smollm2-135m`), Tier 0.5 (`qwen2.5-0.5b`), Tier 1 (`qwen2.5-coder-3b`), Tier 2 (`qwen2.5-coder-32b-moe`), etc.
* **Silício Alvo:** dGPU RTX 2060m vs. CPU Host Intel i9 (AVX2).
* **Throughput Clínico:** TPS medido (tokens/segundo).
* **Latência Primeiro Token (TTFT):** ms até o primeiro token.
* **Consumo de VRAM:** GB alocados vs. capacidade útil.
* **Status Operacional:** `Carregado`, `Em Standby (RAM)`, `Disponível em Disco`.

---

### 3.4 Aba 3: Souls Settings & Controls (Governança, Presets & Chaves)

O centro de controle humano, dividido em blocos funcionais e intuitivos:

#### Bloco 1: Master Brain Preset Selector (Configuração do Hermes Agent)
Permite alternar o cérebro padrão do Hermes Agent com **1 clique**:
* **Cartões de Presets:**
  - 🚀 **Heavy SWE & Architecture:** Modelo mestre `anthropic/claude-3-7-sonnet`. Máxima precisão e raciocínio profundo para sessões de arquitetura.
  - ⚖️ **Balanced Daily / FinOps:** Modelo mestre `deepseek/deepseek-chat` ou `qwen/qwen-2.5-coder-32b`. Excelente qualidade para o dia a dia a uma fração do custo ($0.18/1M).
  - 🧠 **Deep Theorem & Math:** Modelo mestre `deepseek/deepseek-reasoner` ou `openai/o3-mini`. Raciocínio algorítmico rigoroso.
  - 🛡️ **Extreme Local Privacy:** Modelo mestre local no Tier 1 (RTX 2060m). Custo zero e isolamento total de rede.
* **Ação de Aplicação:**
  - Botão `Aplicar no Hermes config.yaml`: Lê o caminho `%LOCALAPPDATA%\hermes\config.yaml`, gera backup atômico (`config.yaml.bak`) e atualiza a chave `model` e parâmetros associados.
  - Feedback visual imediato com Toast notification de sucesso.

#### Bloco 2: Feature Toggles (Switches Liga/Desliga Globais)
Cada toggle possui título, descrição clara e status:
1. **Langevin Decay Epistêmico:** `[ON/OFF]` — Aplica decaimento contínuo em memórias antigas não acessadas no SQLite L2.
2. **Speculative Drafting Local:** `[ON/OFF]` — Utiliza o Tier 0.5 no i9 para acelerar a geração do Tier 1 na GPU.
3. **Source-Side AST Dehydration:** `[ON/OFF]` — Proíbe leitura de arquivos inteiros; impõe extração de outlines via `souls_ast`.
4. **Safe Sandbox Shell Execution:** `[ON/OFF]` — Isola comandos de terminal com timeout rígido de 30s e higienização ANSI.
5. **Obsidian Privacy Shield:** `[ON/OFF]` — Bloqueia envio de dados das pastas `raw/financas/` e `raw/saude/` para APIs de nuvem aberta.
6. **Sincronização Diária Automática:** `[ON/OFF]` — Atualiza as tarifas e modelos da Cloud Arena na inicialização diária do daemon.

#### Bloco 3: Hardware Watchdog & Limiares de Proteção
* **VRAM Ceiling Slider:** Controle deslizante para o teto de alocação de vídeo (Default: `5.2 GB`, range: `4.0` a `5.8 GB`).
* **Thermal Fallback Threshold Slider:** Limiar de temperatura da GPU para acionar desvio para CPU (Default: `78°C`, range: `70°C` a `85°C`).
* **VRAM Inactivity Auto-Unload:** Tempo de ociosidade em minutos antes de descarregar modelos da GPU para a RAM (Default: `5 min`).

#### Bloco 4: Provedores & Credenciais de Nuvem
* **Nous Portal API Key (`NOUS_API_KEY`):** Campo com máscara de senha, botão de visualização e botão `Testar Conexão`.
* **OpenRouter API Key (`OPENROUTER_API_KEY`):** Campo com máscara de senha, botão de visualização e botão `Testar Conexão`.

#### Bloco 5: Ações Rápidas de Manutenção (Quick Actions)
* **💾 WAL Checkpoint (TRUNCATE):** Força o flush de páginas do WAL para o arquivo principal SQLite, liberando espaço em disco.
* **📚 Disparar Curadoria em Lote do Obsidian:** Invoca a esteira para processar os próximos 5 arquivos pendentes em `raw/`.
* **🔄 Recarregar MCP Tools & Skills:** Envia sinal para recarregar definições e sincronizar arquivos de skills.

---

## 4. ENDPOINTS REST NO `souls_server` (CONTRATOS DE DADOS)

Para alimentar o Dashboard e persistir configurações, o daemon Axum expõe os seguintes endpoints:

| Método | Endpoint | Descrição | Formato do Payload / Resposta |
| :--- | :--- | :--- | :--- |
| `GET` | `/dashboard` ou `/` | Serve a aplicação web SPA do Dashboard | `text/html` |
| `GET` | `/api/v1/telemetry` | Retorna telemetria de silício, NVML e FinOps | `JSON: { vram_used_mb, vram_total_mb, gpu_temp_c, cpu_threads, e3_score, tokens_today, usd_saved }` |
| `GET` | `/api/v1/cloud/champions` | Retorna os líderes da Cloud Arena nos 9 perfis | `JSON: { profile, champions: [...] }` |
| `POST` | `/api/v1/cloud/sync` | Dispara sincronização com Nous e OpenRouter | `JSON: { status: "success", updated_count: 24 }` |
| `GET` | `/api/v1/models/local` | Retorna o inventário dos modelos locais e Tiers | `JSON: { models: [...] }` |
| `GET` | `/api/v1/settings` | Retorna configurações ativas e status de toggles | `JSON: { toggles: {...}, thresholds: {...}, hermes_preset: "balanced" }` |
| `POST` | `/api/v1/settings` | Salva alterações no `souls_config.toml` | `JSON: { toggles: {...}, thresholds: {...} }` |
| `POST` | `/api/v1/settings/hermes-preset` | Aplica um preset diretamente no Hermes `config.yaml` | `JSON: { preset_id: "heavy_swe" }` $\to$ `JSON: { success: true, backup: "config.yaml.bak" }` |
| `POST` | `/api/v1/system/checkpoint` | Executa `PRAGMA wal_checkpoint(TRUNCATE)` | `JSON: { success: true, busy: 0, log: 0, checkpointed: 0 }` |
| `POST` | `/api/v1/system/curate-obsidian` | Executa lote seguro de curadoria no cofre | `JSON: { processed: 5, pending: 492 }` |

---

## 5. INTEGRAÇÃO NO HERMES AGENT COMO PLUGIN (`adapters/hermes_dashboard_souls`)

A integração com o Hermes Agent obedece à estrutura modular de plugins do Hermes:

```
adapters/hermes_dashboard_souls/
├── plugin.yaml                    # Manifesto de registro do plugin para o ecossistema Hermes
└── dashboard/
    ├── manifest.json              # Metadados de UI para o Hermes Web/Desktop
    ├── plugin_api.py              # Proxy reverso FastAPI para redirecionamento REST
    └── dist/
        └── index.js               # Componente visual para a aba do Hermes Desktop
```

### 5.1 O Manifesto `plugin.yaml`
Declara metadados do plugin, autor, versão e rotas associadas:
```yaml
name: souls_dashboard
version: 7.0.0
description: Painel de Controle, Telemetria de Silício e Governança do Souls Engine
author: Arquiteto Souls & Hermes
routes:
  - path: /dashboard
    type: web_view
    target: http://127.0.0.1:9123/dashboard
```

### 5.2 O Manifesto de UI `dashboard/manifest.json`
Declara a aba na barra lateral do Hermes Desktop/Web:
```json
{
  "id": "souls_dashboard",
  "name": "Souls Engine",
  "icon": "Cpu",
  "entry": "dist/index.js",
  "route": "/souls-dashboard",
  "category": "system"
}
```

---

## 6. CRITÉRIOS DE ACEITE E DEFINITION OF DONE (SESSÃO 7)

1. A rota `http://127.0.0.1:9123/dashboard` renderiza a aplicação completa em Dark Mode com visual premium e responsivo.
2. A barra superior exibe telemetria em tempo real do silício (VRAM, Temperatura, Threads) e status do daemon.
3. A navegação entre as 3 abas (`Overview`, `Model Arenas`, `Souls Settings`) é fluida e sem recarregamento de página.
4. O seletor dos 9 Perfis Cognitivos na Cloud Arena filtra dinamicamente os modelos e exibe as tags de Pareto.
5. A alteração de preset do Master Brain na aba Settings atualiza o arquivo `config.yaml` do Hermes com geração de backup `.bak`.
6. Os botões de ação rápida (Sync Cloud, Checkpoint WAL, Curadoria Obsidian) executam com feedback visual imediato.
7. O plugin em `adapters/hermes_dashboard_souls` é devidamente estruturado e validado.
8. Todo o código compila limpo via `cargo check -p souls_server` e passa nos testes unitários.
