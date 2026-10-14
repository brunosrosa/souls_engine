# Dossiê Analítico SSOT: Cloud LLMs & Fronteira de Pareto

> **Data de Avaliação:** 2026-10-14 (UTC)  
> **Fonte Relacional SSOT:** `Z:\souls_engine\.souls_data\db\souls_heuristic_vault.db`  
> **Modelos Canônicos Homologados:** 12 modelos em nuvem  
> **Provedores Monitorados:** Nous Research Portal & OpenRouter  
> **Governança:** RFC-005, ADR-007 e LLM Cloud Arena Specification

---

## 🏆 1. RESUMO EXECUTIVO: CAMPEÕES DA FRONTEIRA DE PARETO
Matriz consolidada dos modelos dominantes em cada um dos 5 perfis cognitivos canônicos:

| Perfil Cognitivo | 🏆 Quality Leader | 💎 Best Value (Maior ROI) | ⚡ Ultra Low Cost | Custo Ref. ($/Tarefa) |
| :--- | :--- | :--- | :--- | :---: |
| 💻 **Coding / SWE (Engenharia de Software & Rust AST)** | **Claude 3.7 Sonnet** (92.8) | **DeepSeek R1** (ROI: 57.4) | — | $0.7140 |
| 🤖 **Agentic / Tools (Despacho de Ferramentas & MCP)** | **Claude 3.7 Sonnet** (88.7) | **OpenAI o3-mini** (ROI: 63.0) | — | $0.4675 |
| 🧠 **Reasoning / Math (Raciocínio Lógico & Provas Formais)** | **DeepSeek R1** (85.1) | **Qwen 2.5 Coder 32B** (ROI: 50.8) | `DeepSeek V3` ($0.5660) | $0.3300 |
| ⚡ **Fast / Triage (Triagem de Alta Cadência & Streaming)** | **OpenAI o3-mini** (82.7) | **Qwen 2.5 Coder 32B** (ROI: 65.9) | — | $0.2250 |
| 🎨 **Design / Frontend (Multimodal, UI & Coordenadas)** | **Claude 3.7 Sonnet** (86.2) | **GPT-4o** (ROI: 37.0) | — | $2.0938 |

---

## 📊 2. ANÁLISE DETALHADA POR PERFIL COGNITIVO
Modelos ranqueados pela pontuação composta multiobjetivo, métricas de custo e classificação de Pareto:

### 💻 2.1 Perfil: Coding / SWE (Engenharia de Software & Rust AST)
> **Descrição:** Refatorações complexas, geração de código Rust idiomático, resolução de erros do compilador e cirurgia de Myers Diff.  
> **Métricas Primárias:** Coding ELO & IFEval &bull; **Fórmula:** `S_coding = 0.50·CodingELO + 0.30·IFEval + 0.20·BFCL`

| Rank | Modelo Canônico | Provedor | Custo Blended ($/1M) | Score Composto | ROI Index | Throughput | Janela Ctx | Status Pareto |
| :---: | :--- | :---: | :---: | :---: | :---: | :---: | :---: | :--- |
| 1 | **Claude 3.7 Sonnet**<br>`claude-3-7-sonnet` | nous_portal | $4.5000 | **92.8** | 34.3 | 72 tps | 200k | 🏆 **Quality Leader** |
| 2 | **DeepSeek R1**<br>`deepseek-r1` | nous_portal | $0.7140 | **88.3** | 57.4 | 52 tps | 64k | 💎 **Best Value** |
| 3 | **OpenAI o3-mini**<br>`o3-mini` | openrouter | $0.7150 | **86.9** | 56.5 | 120 tps | 200k | 🚫 Dominated |
| 4 | **GPT-4o**<br>`gpt-4o` | openrouter | $1.6250 | **83.1** | 42.3 | 70 tps | 128k | 🚫 Dominated |
| 5 | **DeepSeek V3**<br>`deepseek-v3` | nous_portal | $0.3600 | **82.5** | 63.1 | 92 tps | 64k | 💎 **Best Value** |
| 6 | **Hermes 3 (Llama 3.1 405B)**<br>`hermes-3-llama-3.1-405b` | openrouter | $0.7000 | **78.6** | 51.4 | 35 tps | 128k | 🚫 Dominated |
| 7 | **Qwen 2.5 Coder 32B**<br>`qwen-2.5-coder-32b` | nous_portal | $0.2200 | **78.5** | 65.4 | 105 tps | 131k | 💎 **Best Value** |
| 8 | **Llama 3.3 70B Instruct**<br>`llama-3.3-70b` | openrouter | $0.4530 | **75.7** | 55.1 | 70 tps | 128k | 🚫 Dominated |
| 9 | **Hermes 3 (Llama 3.1 70B)**<br>`hermes-3-llama-3.1-70b` | nous_portal | $0.3560 | **72.0** | 55.2 | 78 tps | 128k | 🚫 Dominated |
| 10 | **GPT-4o Mini**<br>`gpt-4o-mini` | openrouter | $0.0975 | **70.0** | 64.0 | 120 tps | 128k | 💎 **Best Value** |

#### 🔗 Cadeia de Roteamento Preditivo para Subagentes (`coding`)
| Nível de Escalação | Modelo Designado | Provedor | Custo Blended | Condição de Acionamento |
| :--- | :--- | :--- | :---: | :--- |
| **1. Primário (Custo-Benefício)** | `DeepSeek R1` | nous_portal | $0.7140 | Modelo padrão para subtarefas com SLA normal e orçamento otimizado. |
| **2. Escalação Máxima (Líder)** | `Claude 3.7 Sonnet` | nous_portal | $4.5000 | Acionado se complexidade/entropia for crítica ou falha de teste anterior. |
| **3. Fallback de Baixo Custo** | `DeepSeek V3` | nous_portal | $0.3600 | Standby caso o provedor principal sofra throttling ou indisponibilidade. |

---

### 🤖 2.2 Perfil: Agentic / Tools (Despacho de Ferramentas & MCP)
> **Descrição:** Orquestração de subagentes autônomos, aderência estrita a JSON Schemas e despacho de chamadas de ferramentas MCP.  
> **Métricas Primárias:** BFCL (Berkeley Function Calling) & IFEval &bull; **Fórmula:** `S_agentic = 0.55·BFCL + 0.25·IFEval + 0.20·TTFT_inv`

| Rank | Modelo Canônico | Provedor | Custo Blended ($/1M) | Score Composto | ROI Index | Throughput | Janela Ctx | Status Pareto |
| :---: | :--- | :---: | :---: | :---: | :---: | :---: | :---: | :--- |
| 1 | **Claude 3.7 Sonnet**<br>`claude-3-7-sonnet` | nous_portal | $2.7300 | **88.7** | 38.3 | 72 tps | 200k | 🏆 **Quality Leader** |
| 2 | **OpenAI o3-mini**<br>`o3-mini` | openrouter | $0.4675 | **87.2** | 63.0 | 120 tps | 200k | 💎 **Best Value** |
| 3 | **DeepSeek V3**<br>`deepseek-v3` | nous_portal | $0.2360 | **83.2** | 68.7 | 92 tps | 64k | 💎 **Best Value** |
| 4 | **GPT-4o**<br>`gpt-4o` | openrouter | $1.0625 | **83.1** | 48.2 | 70 tps | 128k | 🚫 Dominated |
| 5 | **DeepSeek R1**<br>`deepseek-r1` | nous_portal | $0.4680 | **82.3** | 59.4 | 52 tps | 64k | 🚫 Dominated |
| 6 | **Qwen 2.5 Coder 32B**<br>`qwen-2.5-coder-32b` | nous_portal | $0.1500 | **79.4** | 69.6 | 105 tps | 131k | 💎 **Best Value** |
| 7 | **Llama 3.3 70B Instruct**<br>`llama-3.3-70b` | openrouter | $0.3445 | **78.5** | 60.6 | 70 tps | 128k | 🚫 Dominated |
| 8 | **Hermes 3 (Llama 3.1 405B)**<br>`hermes-3-llama-3.1-405b` | openrouter | $0.5500 | **78.2** | 54.3 | 35 tps | 128k | 🚫 Dominated |
| 9 | **GPT-4o Mini**<br>`gpt-4o-mini` | openrouter | $0.0638 | **77.7** | 73.2 | 120 tps | 128k | 💎 **Best Value** |
| 10 | **Hermes 3 (Llama 3.1 70B)**<br>`hermes-3-llama-3.1-70b` | nous_portal | $0.2490 | **76.7** | 62.8 | 78 tps | 128k | 🚫 Dominated |

#### 🔗 Cadeia de Roteamento Preditivo para Subagentes (`agentic`)
| Nível de Escalação | Modelo Designado | Provedor | Custo Blended | Condição de Acionamento |
| :--- | :--- | :--- | :---: | :--- |
| **1. Primário (Custo-Benefício)** | `OpenAI o3-mini` | openrouter | $0.4675 | Modelo padrão para subtarefas com SLA normal e orçamento otimizado. |
| **2. Escalação Máxima (Líder)** | `Claude 3.7 Sonnet` | nous_portal | $2.7300 | Acionado se complexidade/entropia for crítica ou falha de teste anterior. |
| **3. Fallback de Baixo Custo** | `DeepSeek V3` | nous_portal | $0.2360 | Standby caso o provedor principal sofra throttling ou indisponibilidade. |

---

### 🧠 2.3 Perfil: Reasoning / Math (Raciocínio Lógico & Provas Formais)
> **Descrição:** Cadeias densas de pensamento (CoT), análise matemática, provas de invariantes de concorrência e dedução de alta entropia.  
> **Métricas Primárias:** Hard Prompts ELO & GPQA &bull; **Fórmula:** `S_reasoning = 0.50·HardELO + 0.35·GPQA + 0.15·GeneralELO`

| Rank | Modelo Canônico | Provedor | Custo Blended ($/1M) | Score Composto | ROI Index | Throughput | Janela Ctx | Status Pareto |
| :---: | :--- | :---: | :---: | :---: | :---: | :---: | :---: | :--- |
| 1 | **DeepSeek R1**<br>`deepseek-r1` | nous_portal | $1.1240 | **85.1** | 48.5 | 52 tps | 64k | 🏆 **Quality Leader** |
| 2 | **Claude 3.7 Sonnet**<br>`claude-3-7-sonnet` | nous_portal | $7.4300 | **82.9** | 26.5 | 72 tps | 200k | 🚫 Dominated |
| 3 | **OpenAI o3-mini**<br>`o3-mini` | openrouter | $1.1275 | **81.9** | 46.7 | 120 tps | 200k | 🚫 Dominated |
| 4 | **GPT-4o**<br>`gpt-4o` | openrouter | $2.5625 | **72.9** | 32.1 | 70 tps | 128k | 🚫 Dominated |
| 5 | **DeepSeek V3**<br>`deepseek-v3` | nous_portal | $0.5660 | **72.4** | 50.0 | 92 tps | 64k | ⚡ **Ultra Low Cost** |
| 6 | **Hermes 3 (Llama 3.1 405B)**<br>`hermes-3-llama-3.1-405b` | openrouter | $0.8500 | **69.7** | 43.2 | 35 tps | 128k | 🚫 Dominated |
| 7 | **Llama 3.3 70B Instruct**<br>`llama-3.3-70b` | openrouter | $0.5815 | **66.1** | 45.3 | 70 tps | 128k | 🚫 Dominated |
| 8 | **Qwen 2.5 Coder 32B**<br>`qwen-2.5-coder-32b` | nous_portal | $0.3300 | **65.3** | 50.8 | 105 tps | 131k | 💎 **Best Value** |
| 9 | **Hermes 3 (Llama 3.1 70B)**<br>`hermes-3-llama-3.1-70b` | nous_portal | $0.5180 | **62.0** | 43.7 | 78 tps | 128k | 🚫 Dominated |
| 10 | **GPT-4o Mini**<br>`gpt-4o-mini` | openrouter | $0.1537 | **58.0** | 50.8 | 120 tps | 128k | 💎 **Best Value** |

#### 🔗 Cadeia de Roteamento Preditivo para Subagentes (`reasoning`)
| Nível de Escalação | Modelo Designado | Provedor | Custo Blended | Condição de Acionamento |
| :--- | :--- | :--- | :---: | :--- |
| **1. Primário (Custo-Benefício)** | `Qwen 2.5 Coder 32B` | nous_portal | $0.3300 | Modelo padrão para subtarefas com SLA normal e orçamento otimizado. |
| **2. Escalação Máxima (Líder)** | `DeepSeek R1` | nous_portal | $1.1240 | Acionado se complexidade/entropia for crítica ou falha de teste anterior. |
| **3. Fallback de Baixo Custo** | `GPT-4o Mini` | openrouter | $0.1537 | Standby caso o provedor principal sofra throttling ou indisponibilidade. |

---

### ⚡ 2.4 Perfil: Fast / Triage (Triagem de Alta Cadência & Streaming)
> **Descrição:** Sanitização de contexto, triagem de intenção preliminar, ETL de logs e streaming instantâneo de turnos rápidos.  
> **Métricas Primárias:** Throughput (TPS) & TTFT (ms) &bull; **Fórmula:** `S_fast = 0.50·TPS_norm + 0.30·TTFT_norm + 0.20·Price_inv`

| Rank | Modelo Canônico | Provedor | Custo Blended ($/1M) | Score Composto | ROI Index | Throughput | Janela Ctx | Status Pareto |
| :---: | :--- | :---: | :---: | :---: | :---: | :---: | :---: | :--- |
| 1 | **OpenAI o3-mini**<br>`o3-mini` | openrouter | $0.6738 | **82.7** | 54.6 | 120 tps | 200k | 🏆 **Quality Leader** |
| 2 | **Qwen 2.5 Coder 32B**<br>`qwen-2.5-coder-32b` | nous_portal | $0.2250 | **79.3** | 65.9 | 105 tps | 131k | 💎 **Best Value** |
| 3 | **GPT-4o Mini**<br>`gpt-4o-mini` | openrouter | $0.0919 | **78.4** | 72.0 | 120 tps | 128k | 💎 **Best Value** |
| 4 | **DeepSeek V3**<br>`deepseek-v3` | nous_portal | $0.3410 | **76.0** | 58.7 | 92 tps | 64k | 🚫 Dominated |
| 5 | **Claude 3.7 Sonnet**<br>`claude-3-7-sonnet` | nous_portal | $4.2550 | **71.4** | 26.9 | 72 tps | 200k | 🚫 Dominated |
| 6 | **Hermes 3 (Llama 3.1 70B)**<br>`hermes-3-llama-3.1-70b` | nous_portal | $0.3790 | **69.3** | 52.4 | 78 tps | 128k | 🚫 Dominated |
| 7 | **GPT-4o**<br>`gpt-4o` | openrouter | $1.5312 | **67.6** | 35.0 | 70 tps | 128k | 🚫 Dominated |
| 8 | **Llama 3.3 70B Instruct**<br>`llama-3.3-70b` | openrouter | $0.5657 | **65.7** | 45.3 | 70 tps | 128k | 🚫 Dominated |
| 9 | **DeepSeek R1**<br>`deepseek-r1` | nous_portal | $0.6730 | **61.2** | 40.4 | 52 tps | 64k | 🚫 Dominated |
| 10 | **Hermes 3 (Llama 3.1 405B)**<br>`hermes-3-llama-3.1-405b` | openrouter | $0.9250 | **56.3** | 34.0 | 35 tps | 128k | 🚫 Dominated |

#### 🔗 Cadeia de Roteamento Preditivo para Subagentes (`fast`)
| Nível de Escalação | Modelo Designado | Provedor | Custo Blended | Condição de Acionamento |
| :--- | :--- | :--- | :---: | :--- |
| **1. Primário (Custo-Benefício)** | `Qwen 2.5 Coder 32B` | nous_portal | $0.2250 | Modelo padrão para subtarefas com SLA normal e orçamento otimizado. |
| **2. Escalação Máxima (Líder)** | `OpenAI o3-mini` | openrouter | $0.6738 | Acionado se complexidade/entropia for crítica ou falha de teste anterior. |
| **3. Fallback de Baixo Custo** | `GPT-4o Mini` | openrouter | $0.0919 | Standby caso o provedor principal sofra throttling ou indisponibilidade. |

---

### 🎨 2.5 Perfil: Design / Frontend (Multimodal, UI & Coordenadas)
> **Descrição:** Compreensão de capturas de tela, layouting CSS Vanilla, coordenadas visuais e inspeção multimodal de interfaces.  
> **Métricas Primárias:** VQA Multimodal & General ELO &bull; **Fórmula:** `S_design = 0.60·VQAScore + 0.25·GeneralELO + 0.15·IFEval`

| Rank | Modelo Canônico | Provedor | Custo Blended ($/1M) | Score Composto | ROI Index | Throughput | Janela Ctx | Status Pareto |
| :---: | :--- | :---: | :---: | :---: | :---: | :---: | :---: | :--- |
| 1 | **Claude 3.7 Sonnet**<br>`claude-3-7-sonnet` | nous_portal | $5.9650 | **86.2** | 29.3 | 72 tps | 200k | 🏆 **Quality Leader** |
| 2 | **GPT-4o**<br>`gpt-4o` | openrouter | $2.0938 | **78.7** | 37.0 | 70 tps | 128k | 💎 **Best Value** |
| 3 | **OpenAI o3-mini**<br>`o3-mini` | openrouter | $0.9213 | **77.4** | 46.8 | 120 tps | 200k | 💎 **Best Value** |
| 4 | **DeepSeek V3**<br>`deepseek-v3` | nous_portal | $0.4630 | **76.9** | 55.7 | 92 tps | 64k | 💎 **Best Value** |
| 5 | **GPT-4o Mini**<br>`gpt-4o-mini` | openrouter | $0.1256 | **74.9** | 67.0 | 120 tps | 128k | 💎 **Best Value** |
| 6 | **Qwen 2.5 Coder 32B**<br>`qwen-2.5-coder-32b` | nous_portal | $0.2750 | **74.1** | 59.6 | 105 tps | 131k | 🚫 Dominated |
| 7 | **DeepSeek R1**<br>`deepseek-r1` | nous_portal | $0.9190 | **73.4** | 44.4 | 52 tps | 64k | 🚫 Dominated |
| 8 | **Llama 3.3 70B Instruct**<br>`llama-3.3-70b` | openrouter | $0.5172 | **65.3** | 46.1 | 70 tps | 128k | 🚫 Dominated |
| 9 | **Hermes 3 (Llama 3.1 70B)**<br>`hermes-3-llama-3.1-70b` | nous_portal | $0.4370 | **64.1** | 47.0 | 78 tps | 128k | 🚫 Dominated |
| 10 | **Hermes 3 (Llama 3.1 405B)**<br>`hermes-3-llama-3.1-405b` | openrouter | $0.7750 | **63.1** | 40.1 | 35 tps | 128k | 🚫 Dominated |

#### 🔗 Cadeia de Roteamento Preditivo para Subagentes (`design`)
| Nível de Escalação | Modelo Designado | Provedor | Custo Blended | Condição de Acionamento |
| :--- | :--- | :--- | :---: | :--- |
| **1. Primário (Custo-Benefício)** | `GPT-4o` | openrouter | $2.0938 | Modelo padrão para subtarefas com SLA normal e orçamento otimizado. |
| **2. Escalação Máxima (Líder)** | `Claude 3.7 Sonnet` | nous_portal | $5.9650 | Acionado se complexidade/entropia for crítica ou falha de teste anterior. |
| **3. Fallback de Baixo Custo** | `OpenAI o3-mini` | openrouter | $0.9213 | Standby caso o provedor principal sofra throttling ou indisponibilidade. |

---

## 📦 3. CATÁLOGO CANÔNICO DE MODELOS DE NUVEM & RECURSOS
Especificações arquiteturais e suporte a recursos avançados de cada modelo catalogado:

| Identificador Canônico | Nome Comercial | Desenvolvedor | Janela de Contexto | Tool Calling | Prefix Caching | Multimodal | Structured Outputs |
| :--- | :--- | :--- | :---: | :---: | :---: | :---: | :---: |
| `qwen-2.5-coder-32b` | **Qwen 2.5 Coder 32B** | Alibaba | 131,072 tokens | ✅ Sim | ✅ Sim | ❌ Não | ✅ Sim |
| `claude-3-5-sonnet` | **Claude 3.5 Sonnet** | Anthropic | 200,000 tokens | ✅ Sim | ✅ Sim | 👁️ Sim | ✅ Sim |
| `claude-3-7-sonnet` | **Claude 3.7 Sonnet** | Anthropic | 200,000 tokens | ✅ Sim | ✅ Sim | 👁️ Sim | ✅ Sim |
| `deepseek-r1` | **DeepSeek R1** | DeepSeek | 64,000 tokens | ✅ Sim | ✅ Sim | ❌ Não | ✅ Sim |
| `deepseek-v3` | **DeepSeek V3** | DeepSeek | 64,000 tokens | ✅ Sim | ✅ Sim | ❌ Não | ✅ Sim |
| `gemini-2.0-flash` | **Gemini 2.0 Flash** | Google | 1,000,000 tokens | ✅ Sim | ✅ Sim | 👁️ Sim | ✅ Sim |
| `llama-3.3-70b` | **Llama 3.3 70B Instruct** | Meta | 128,000 tokens | ✅ Sim | ❌ Não | ❌ Não | ✅ Sim |
| `hermes-3-llama-3.1-405b` | **Hermes 3 (Llama 3.1 405B)** | Nous Research | 128,000 tokens | ✅ Sim | ✅ Sim | ❌ Não | ✅ Sim |
| `hermes-3-llama-3.1-70b` | **Hermes 3 (Llama 3.1 70B)** | Nous Research | 128,000 tokens | ✅ Sim | ✅ Sim | ❌ Não | ✅ Sim |
| `gpt-4o` | **GPT-4o** | OpenAI | 128,000 tokens | ✅ Sim | ✅ Sim | 👁️ Sim | ✅ Sim |
| `gpt-4o-mini` | **GPT-4o Mini** | OpenAI | 128,000 tokens | ✅ Sim | ✅ Sim | 👁️ Sim | ✅ Sim |
| `o3-mini` | **OpenAI o3-mini** | OpenAI | 200,000 tokens | ✅ Sim | ✅ Sim | ❌ Não | ✅ Sim |

---

## 💸 4. MATRIZ COMERCIAL DE PREÇOS & TELEMETRIA POR PROVEDOR
Preços de mercado reais levantados via APIs (Input, Output, Cache Read e latências TTFT):

| Modelo Canônico | Provedor / Rota | Input $/1M | Output $/1M | Cache Read $/1M | TTFT Médio | Throughput Médio |
| :--- | :--- | :---: | :---: | :---: | :---: | :---: |
| **Claude 3.7 Sonnet** (`claude-3-7-sonnet`) | `nous_portal` | $3.40 | $15.00 | $0.3500 | 420 ms | 72.0 tok/s |
| **DeepSeek R1** (`deepseek-r1`) | `nous_portal` | $0.55 | $2.19 | $0.1400 | 580 ms | 52.0 tok/s |
| **DeepSeek R1** (`deepseek-r1`) | `openrouter` | $0.70 | $2.50 | $0.1750 | 450 ms | 45.0 tok/s |
| **DeepSeek V3** (`deepseek-v3`) | `nous_portal` | $0.28 | $1.10 | $0.0700 | 320 ms | 92.0 tok/s |
| **DeepSeek V3** (`deepseek-v3`) | `openrouter` | $0.32 | $0.89 | $0.0800 | 450 ms | 85.0 tok/s |
| **GPT-4o** (`gpt-4o`) | `openrouter` | $1.25 | $5.00 | $0.3125 | 450 ms | 70.0 tok/s |
| **GPT-4o** (`gpt-4o`) | `openrouter` | $2.50 | $10.00 | $0.6250 | 450 ms | 70.0 tok/s |
| **GPT-4o Mini** (`gpt-4o-mini`) | `openrouter` | $0.07 | $0.30 | $0.0187 | 450 ms | 120.0 tok/s |
| **GPT-4o Mini** (`gpt-4o-mini`) | `openrouter` | $0.15 | $0.60 | $0.0375 | 450 ms | 120.0 tok/s |
| **Hermes 3 (Llama 3.1 405B)** (`hermes-3-llama-3.1-405b`) | `openrouter` | $1.00 | $1.00 | $0.2500 | 450 ms | 35.0 tok/s |
| **Hermes 3 (Llama 3.1 405B)** (`hermes-3-llama-3.1-405b`) | `nous_portal` | $1.20 | $3.50 | $0.3000 | 650 ms | 38.0 tok/s |
| **Hermes 3 (Llama 3.1 70B)** (`hermes-3-llama-3.1-70b`) | `nous_portal` | $0.35 | $0.90 | $0.0900 | 310 ms | 78.0 tok/s |
| **Hermes 3 (Llama 3.1 70B)** (`hermes-3-llama-3.1-70b`) | `openrouter` | $0.70 | $0.70 | $0.1750 | 450 ms | 70.0 tok/s |
| **Llama 3.3 70B Instruct** (`llama-3.3-70b`) | `openrouter` | $0.59 | $0.79 | $0.1475 | 450 ms | 70.0 tok/s |
| **OpenAI o3-mini** (`o3-mini`) | `openrouter` | $0.55 | $2.20 | $0.1375 | 450 ms | 120.0 tok/s |
| **OpenAI o3-mini** (`o3-mini`) | `openrouter` | $1.10 | $4.40 | $0.2750 | 450 ms | 120.0 tok/s |
| **Qwen 2.5 Coder 32B** (`qwen-2.5-coder-32b`) | `nous_portal` | $0.20 | $0.60 | $0.0500 | 290 ms | 105.0 tok/s |
| **Qwen 2.5 Coder 32B** (`qwen-2.5-coder-32b`) | `openrouter` | $0.66 | $1.00 | $0.1650 | 450 ms | 95.0 tok/s |

---

## 🎯 5. BENCHMARKS EMPÍRICOS E MÉTRICAS DE ENGENHARIA
Resultados empíricos normalizados utilizados no cômputo da Fronteira de Pareto:

| Modelo Canônico | General ELO | Coding ELO | Hard ELO | BFCL (Tools) | IFEval | GPQA | VQA Score |
| :--- | :---: | :---: | :---: | :---: | :---: | :---: | :---: |
| **Claude 3.7 Sonnet** | 1380 | **1420** | 1395 | 95.5% | 93.8% | 74.2% | 94.0% |
| **Claude 3.5 Sonnet** | 1360 | **1400** | 1375 | 93.0% | 91.5% | 70.0% | 92.5% |
| **DeepSeek V3** | 1340 | **1365** | 1330 | 89.2% | 87.5% | 65.0% | 75.0% |
| **DeepSeek R1** | 1375 | **1390** | 1425 | 86.0% | 88.0% | 76.5% | 70.0% |
| **Qwen 2.5 Coder 32B** | 1300 | **1360** | 1290 | 84.5% | 83.0% | 58.0% | 65.0% |
| **GPT-4o** | 1350 | **1360** | 1340 | 91.0% | 88.5% | 64.0% | 90.0% |
| **GPT-4o Mini** | 1280 | **1290** | 1260 | 82.0% | 81.0% | 48.0% | 84.0% |
| **OpenAI o3-mini** | 1365 | **1385** | 1390 | 88.0% | 89.0% | 75.0% | 60.0% |
| **Llama 3.3 70B Instruct** | 1320 | **1315** | 1305 | 85.0% | 86.0% | 56.0% | 60.0% |
| **Hermes 3 (Llama 3.1 405B)** | 1335 | **1330** | 1320 | 88.0% | 87.0% | 61.0% | 60.0% |
| **Hermes 3 (Llama 3.1 70B)** | 1300 | **1295** | 1280 | 83.0% | 84.0% | 52.0% | 60.0% |
| **Gemini 2.0 Flash** | 1345 | **1350** | 1335 | 89.5% | 89.0% | 65.5% | 93.0% |

---

## 🛡️ 6. DIRETRIZES DE FINOPS & MATRIZ DE DECISÃO LOCAL VS CLOUD

1. **Regra de Ouro do Silício Local:** Tarefas de desidratação AST, triagem de entropia, inspeção de tipos e refatorações cirúrgicas de arquivo único pertencem **100% à dGPU RTX 2060m e CPU Host i9** (custo zero de nuvem).
2. **Critérios de Escalação para Nuvem:**
   - **Alta Entropia Cognitiva ($H > 0.85$):** Questões abertas de arquitetura de software demandam `Claude 3.7 Sonnet` ou `DeepSeek R1`.
   - **Estouro de Raio de Impacto (Blast Radius > 5 arquivos):** Refatorações cross-crate devem delegar a síntese para a Nuvem.
   - **Janela Gigante (> 32k tokens):** Contextos muito extensos devem utilizar o prefix caching do `DeepSeek V3` ou `Gemini 2.0 Flash`.
3. **Prefix Caching Obrigatório:** Sempre priorize provedores com suporte a cache read com desconto $\ge 75\%$ em sessões multi-turn longas.
