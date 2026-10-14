# Dossiê Analítico SSOT: Cloud LLMs & Fronteira de Pareto

> **Data de Avaliação:** 2026-10-14 (UTC)  
> **Fonte Relacional SSOT:** `Z:\souls_engine\.souls_data\db\souls_heuristic_vault.db`  
> **Modelos Canônicos Homologados:** 347 modelos em nuvem  
> **Provedores Monitorados:** Nous Research Portal & OpenRouter  
> **Governança:** RFC-005, ADR-007 e LLM Cloud Arena Specification

---

## 🏆 1. RESUMO EXECUTIVO: CAMPEÕES DA FRONTEIRA DE PARETO
Matriz consolidada dos modelos dominantes em cada um dos 5 perfis cognitivos canônicos:

| Perfil Cognitivo | 🏆 Quality Leader | 💎 Best Value (Maior ROI) | ⚡ Ultra Low Cost | Custo Ref. ($/Tarefa) |
| :--- | :--- | :--- | :--- | :---: |
| 💻 **Coding / SWE (Engenharia de Software & Rust AST)** | **Claude 3.7 Sonnet** (92.8) | **DeepSeek R1** (ROI: 57.4) | — | $0.7140 |
| 🤖 **Agentic / Tools (Despacho de Ferramentas & MCP)** | **Claude 3.7 Sonnet** (88.7) | **OpenAI o3-mini** (ROI: 63.0) | — | $0.4675 |
| 🧠 **Reasoning / Math (Raciocínio Lógico & Provas Formais)** | **Xiaomi: MiMo-V2.6-Pro** (89.1) | **Xiaomi: MiMo-V2.6-Flash** (ROI: 72.0) | `DeepSeek V3` ($0.5660) | $0.1620 |
| ⚡ **Fast / Triage (Triagem de Alta Cadência & Streaming)** | **Google: Gemini 3 Flash Preview** (88.6) | **Xiaomi: MiMo-V2.6-Flash** (ROI: 79.5) | `Sentence Transformers: all-MiniLM-L12-v2` ($0.0041) | $0.1130 |
| 🎨 **Design / Frontend (Multimodal, UI & Coordenadas)** | **Claude 3.7 Sonnet** (86.2) | **GPT-4o** (ROI: 37.0) | — | $2.0938 |

---

## 📊 2. ANÁLISE DETALHADA POR PERFIL COGNITIVO
Modelos ranqueados pela pontuação composta multiobjetivo, métricas de custo e classificação de Pareto:

### 💻 2.1 Perfil: Coding / SWE (Engenharia de Software & Rust AST)
> **Descrição:** Refatorações complexas, geração de código Rust idiomático, resolução de erros do compilador e cirurgia de Myers Diff.  
> **Métricas Primárias:** Coding ELO & IFEval &bull; **Fórmula:** `S_coding = 0.50·CodingELO + 0.30·IFEval + 0.20·BFCL`

| Rank | Modelo Canônico | Provedor | Custo Blended ($/1M) | Score Composto | ROI Index | Throughput | Janela Ctx | Status Pareto |
| :---: | :--- | :---: | :---: | :---: | :---: | :---: | :---: | :--- |
| 1 | **Claude 3.7 Sonnet**<br>`claude-3-7-sonnet` | nous_portal | $4.5000 | **92.8** | 34.3 | 75 tps | 200k | 🏆 **Quality Leader** |
| 2 | **Xiaomi: MiMo-V2.6-Pro**<br>`xiaomi/mimo-v2.6-pro` | nous_portal | $0.3474 | **89.1** | 68.7 | 75 tps | 1050k | 🏆 **Quality Leader** |
| 3 | **DeepSeek R1**<br>`deepseek-r1` | nous_portal | $0.7140 | **88.3** | 57.4 | 75 tps | 64k | 💎 **Best Value** |
| 4 | **OpenAI o3-mini**<br>`o3-mini` | nous_portal | $0.7150 | **86.9** | 56.5 | 130 tps | 200k | 🚫 Dominated |
| 5 | **Google: Gemini 3 Flash Preview**<br>`google/gemini-3-flash-preview` | nous_portal_batch | $0.8200 | **84.7** | 53.0 | 150 tps | 1048k | 🚫 Dominated |
| 6 | **GPT-4o**<br>`gpt-4o` | nous_portal | $1.6250 | **83.1** | 42.3 | 75 tps | 128k | 🚫 Dominated |
| 7 | **Xiaomi: MiMo-V2.6-Flash**<br>`xiaomi/mimo-v2.6-flash` | nous_portal | $0.1160 | **82.8** | 74.6 | 130 tps | 1050k | 💎 **Best Value** |
| 8 | **DeepSeek V3**<br>`deepseek-v3` | nous_portal | $0.3600 | **82.5** | 63.1 | 92 tps | 64k | 💎 **Best Value** |
| 9 | **Z.ai: GLM 5**<br>`z-ai/glm-5` | nous_portal | $0.6720 | **81.1** | 53.6 | 75 tps | 204k | 🚫 Dominated |
| 10 | **Qwen: Qwen3.5 397B A17B**<br>`qwen/qwen3.5-397b-a17b` | nous_portal | $0.6552 | **79.6** | 52.9 | 75 tps | 262k | 🚫 Dominated |
| 11 | **Z.ai: GLM 5V Turbo**<br>`z-ai/glm-5v-turbo` | nous_portal | $1.3760 | **79.6** | 42.6 | 75 tps | 202k | 🚫 Dominated |
| 12 | **Tencent: Hy3**<br>`tencent/hy3` | nous_portal | $0.1716 | **79.5** | 68.7 | 75 tps | 262k | 🚫 Dominated |
| 13 | **Hermes 3 (Llama 3.1 405B)**<br>`hermes-3-llama-3.1-405b` | nous_portal | $0.7000 | **78.6** | 51.4 | 40 tps | 128k | 🚫 Dominated |
| 14 | **Qwen 2.5 Coder 32B**<br>`qwen-2.5-coder-32b` | nous_portal | $0.2200 | **78.5** | 65.4 | 105 tps | 131k | 💎 **Best Value** |
| 15 | **Z.ai: GLM 4.5**<br>`z-ai/glm-4.5` | nous_portal | $0.7240 | **78.2** | 50.6 | 75 tps | 131k | 🚫 Dominated |
| 16 | **OpenAI: GPT-5.4**<br>`openai/gpt-5.4` | nous_portal_batch | $4.1000 | **76.1** | 28.9 | 150 tps | 1050k | 🚫 Dominated |
| 17 | **Llama 3.3 70B Instruct**<br>`llama-3.3-70b` | nous_portal | $0.4530 | **75.7** | 55.1 | 75 tps | 128k | 🚫 Dominated |
| 18 | **Hermes 3 (Llama 3.1 70B)**<br>`hermes-3-llama-3.1-70b` | nous_portal | $0.3560 | **72.0** | 55.2 | 78 tps | 128k | 🚫 Dominated |
| 19 | **Anthropic: Claude Fable 5.1**<br>`anthropic/claude-fable-5.1` | nous_portal_batch | $14.1000 | **70.9** | 19.1 | 150 tps | 1000k | 🚫 Dominated |
| 20 | **GPT-4o Mini**<br>`gpt-4o-mini` | openrouter | $0.0975 | **70.0** | 64.0 | 120 tps | 128k | 💎 **Best Value** |
| 21 | **Anthropic: Claude Opus 5**<br>`anthropic/claude-opus-5` | nous_portal_batch | $7.2000 | **68.3** | 22.0 | 150 tps | 1000k | 🚫 Dominated |
| 22 | **Qwen: Qwen3.8 Max (0902)**<br>`qwen/qwen3.8-max-0902` | nous_portal | $2.1000 | **67.1** | 31.5 | 75 tps | 1000k | 🚫 Dominated |
| 23 | **Meta: Muse Spark 1.3**<br>`meta/muse-spark-1.3` | nous_portal | $1.4100 | **66.7** | 35.5 | 75 tps | 1048k | 🚫 Dominated |
| 24 | **xAI: Grok 4.6**<br>`x-ai/grok-4.6` | nous_portal | $2.2000 | **66.1** | 30.6 | 75 tps | 500k | 🚫 Dominated |
| 25 | **Qwen: Qwen3.6 Plus**<br>`qwen/qwen3.6-plus` | nous_portal | $0.5460 | **65.7** | 45.8 | 75 tps | 1000k | 🚫 Dominated |
| 26 | **OpenAI: GPT-6 Astra**<br>`openai/gpt-6-astra` | nous_portal_batch | $14.4000 | **65.2** | 17.5 | 150 tps | 1050k | 🚫 Dominated |
| 27 | **OpenAI: GPT-5.6 Sol**<br>`openai/gpt-5.6-sol` | nous_portal_batch | $2.8800 | **65.2** | 27.7 | 150 tps | 1050k | 🚫 Dominated |
| 28 | **Z.ai: GLM 5.3**<br>`z-ai/glm-5.3` | openrouter_market | $1.0036 | **65.0** | 38.4 | 70 tps | 1048k | 🚫 Dominated |
| 29 | **Claude Fable 5**<br>`anthropic/claude-fable-5` | nous_portal_batch | $14.4000 | **64.9** | 17.4 | 150 tps | 1048k | 🚫 Dominated |
| 30 | **MoonshotAI: Kimi K3**<br>`moonshotai/kimi-k3` | nous_portal | $3.4080 | **64.4** | 25.9 | 75 tps | 1048k | 🚫 Dominated |
| 31 | **Z.ai: GLM 5.3 Flash**<br>`z-ai/glm-5.3-flash` | nous_portal_batch | $0.1376 | **62.2** | 55.1 | 150 tps | 1048k | 🚫 Dominated |
| 32 | **Qwen: Qwen3.8 2.4T A95B**<br>`qwen/qwen3.8-2.4t-a95b` | nous_portal | $2.1000 | **62.1** | 29.1 | 75 tps | 1048k | 🚫 Dominated |
| 33 | **OpenAI: GPT-5.6 Terra**<br>`openai/gpt-5.6-terra` | nous_portal_batch | $3.2800 | **61.6** | 25.1 | 150 tps | 1050k | 🚫 Dominated |
| 34 | **Z.ai: GLM 4.6**<br>`z-ai/glm-4.6` | nous_portal | $0.5540 | **61.2** | 42.5 | 75 tps | 204k | 🚫 Dominated |
| 35 | **Z.ai: GLM 4.7**<br>`z-ai/glm-4.7` | nous_portal | $0.5420 | **60.5** | 42.2 | 75 tps | 204k | 🚫 Dominated |
| 36 | **Google: Gemini 3.8 Flash**<br>`google/gemini-3.8-flash` | nous_portal_batch | $1.0800 | **60.1** | 34.7 | 150 tps | 1048k | 🚫 Dominated |
| 37 | **Claude Opus 4.8**<br>`anthropic/claude-opus-4.8` | nous_portal_batch | $7.2000 | **59.7** | 19.2 | 150 tps | 1000k | 🚫 Dominated |
| 38 | **Meta: Muse Spark 1.2**<br>`meta/muse-spark-1.2` | nous_portal | $1.4100 | **59.2** | 31.5 | 75 tps | 1048k | 🚫 Dominated |
| 39 | **Claude Sonnet 5**<br>`anthropic/claude-sonnet-5` | nous_portal_batch | $2.8800 | **58.9** | 25.0 | 150 tps | 1000k | 🚫 Dominated |
| 40 | **xAI: Grok 4.5**<br>`x-ai/grok-4.5` | nous_portal | $2.1200 | **58.4** | 27.3 | 75 tps | 500k | 🚫 Dominated |
| 41 | **OpenAI: GPT-5.6 Luna**<br>`openai/gpt-5.6-luna` | nous_portal_batch | $0.3280 | **58.2** | 45.4 | 150 tps | 1050k | 🚫 Dominated |
| 42 | **Qwen: Qwen3.8 27B**<br>`qwen/qwen3.8-27b` | nous_portal | $0.2672 | **58.1** | 46.9 | 75 tps | 1000k | 🚫 Dominated |
| 43 | **Claude Opus 4.7**<br>`anthropic/claude-opus-4.7` | nous_portal_batch | $7.2000 | **57.8** | 18.6 | 150 tps | 1000k | 🚫 Dominated |
| 44 | **Anthropic: Claude Opus 5.5**<br>`anthropic/claude-opus-5.5` | nous_portal_batch | $5.6800 | **57.6** | 19.9 | 150 tps | 1000k | 🚫 Dominated |
| 45 | **OpenAI: GPT-5.5**<br>`openai/gpt-5.5` | nous_portal_batch | $8.2000 | **57.6** | 17.9 | 150 tps | 1050k | 🚫 Dominated |
| 46 | **Google: Gemini 3.7 Flash**<br>`google/gemini-3.7-flash` | nous_portal_batch | $1.0800 | **57.1** | 33.0 | 150 tps | 1048k | 🚫 Dominated |
| 47 | **DeepSeek: DeepSeek V4 Flash Vision Exp**<br>`deepseek/deepseek-v4-flash-vision-exp` | nous_portal | $0.2184 | **57.1** | 47.7 | 130 tps | 1048k | 🚫 Dominated |
| 48 | **DeepSeek: DeepSeek V4 Flash**<br>`deepseek/deepseek-v4-flash-0731` | openrouter_market | $0.0616 | **56.5** | 53.3 | 130 tps | 1048k | 💎 **Best Value** |
| 49 | **DeepSeek: DeepSeek V4 Pro 0813**<br>`deepseek/deepseek-v4-pro-0813` | nous_portal | $1.7440 | **56.4** | 28.1 | 75 tps | 1048k | 🚫 Dominated |
| 50 | **Anthropic: Claude Sonnet 5.5**<br>`anthropic/claude-sonnet-5.5` | nous_portal_batch | $2.8400 | **56.0** | 23.9 | 150 tps | 1000k | 🚫 Dominated |
| 51 | **Z.ai: GLM 5.2**<br>`z-ai/glm-5.2` | openrouter_market | $0.9900 | **55.1** | 32.7 | 70 tps | 1048k | 🚫 Dominated |
| 52 | **OpenAI: GPT Astra Latest**<br>`openai/gpt-astra-latest` | nous_portal | $14.4000 | **54.2** | 14.5 | 40 tps | 1050k | 🚫 Dominated |
| 53 | **OpenAI: GPT-6 Astra Pro Flex**<br>`openai/gpt-6-astra-pro-flex` | nous_portal | $7.2000 | **54.2** | 17.5 | 40 tps | 1050k | 🚫 Dominated |
| 54 | **OpenAI: GPT-6 Astra Pro Fast**<br>`openai/gpt-6-astra-pro-fast` | nous_portal | $28.8000 | **54.2** | 12.3 | 40 tps | 1050k | 🚫 Dominated |
| 55 | **OpenAI: GPT-6 Astra Fast**<br>`openai/gpt-6-astra-fast` | nous_portal | $28.8000 | **54.2** | 12.3 | 40 tps | 1050k | 🚫 Dominated |
| 56 | **OpenAI: GPT-6 Astra Flex**<br>`openai/gpt-6-astra-flex` | nous_portal | $7.2000 | **54.2** | 17.5 | 40 tps | 1050k | 🚫 Dominated |
| 57 | **Anthropic: Claude Opus Latest**<br>`anthropic/claude-opus-latest` | nous_portal | $5.6800 | **54.2** | 18.7 | 40 tps | 1000k | 🚫 Dominated |
| 58 | **OpenAI: GPT-6 Astra Pro**<br>`openai/gpt-6-astra-pro` | nous_portal_batch | $14.4000 | **54.2** | 14.5 | 150 tps | 1050k | 🚫 Dominated |
| 59 | **Google: Gemini 4 Argon**<br>`google/gemini-4-argon` | nous_portal_batch | $2.8400 | **52.6** | 22.4 | 150 tps | 1048k | 🚫 Dominated |
| 60 | **OpenAI: GPT-6.1 Sol**<br>`openai/gpt-6.1-sol` | nous_portal_batch | $2.8400 | **51.8** | 22.1 | 150 tps | 1050k | 🚫 Dominated |
| 61 | **Claude 3.5 Sonnet**<br>`claude-3-5-sonnet` | nous_portal | $3.8400 | **51.2** | 19.9 | 75 tps | 200k | 🏆 **Quality Leader** |
| 62 | **Claude 3.7 Sonnet**<br>`claude-3-7-sonnet` | nous_portal | $3.8400 | **51.2** | 19.9 | 75 tps | 200k | 💎 **Best Value** |
| 63 | **Claude 3.7 Sonnet**<br>`claude-3-7-sonnet` | nous_portal | $3.8400 | **51.2** | 19.9 | 75 tps | 200k | 🏆 **Quality Leader** |
| 64 | **Claude 3.5 Sonnet**<br>`claude-3-5-sonnet` | nous_portal | $3.8400 | **51.2** | 19.9 | 75 tps | 200k | 💎 **Best Value** |
| 65 | **OpenAI: GPT Sol Latest**<br>`openai/gpt-sol-latest` | nous_portal | $2.8400 | **51.2** | 21.9 | 90 tps | 1050k | 🚫 Dominated |
| 66 | **Anthropic: Claude Sonnet Latest**<br>`anthropic/claude-sonnet-latest` | nous_portal | $2.8400 | **51.2** | 21.9 | 75 tps | 1000k | 🚫 Dominated |
| 67 | **OpenAI: GPT-5.6 Sol Pro**<br>`openai/gpt-5.6-sol-pro` | nous_portal_batch | $2.8800 | **51.2** | 21.8 | 150 tps | 1050k | 🚫 Dominated |
| 68 | **OpenAI: GPT-6 Sol Pro**<br>`openai/gpt-6-sol-pro` | nous_portal_batch | $2.8800 | **51.2** | 21.8 | 150 tps | 1050k | 🚫 Dominated |
| 69 | **Z.ai: GLM 5.3 Prime**<br>`z-ai/glm-5.3-prime` | nous_portal | $3.1040 | **51.2** | 21.2 | 75 tps | 1000k | 🚫 Dominated |
| 70 | **OpenAI: GPT-6.1 Sol Pro**<br>`openai/gpt-6.1-sol-pro` | nous_portal_batch | $2.8400 | **51.2** | 21.9 | 150 tps | 1050k | 🚫 Dominated |
| 71 | **Qwen: Qwen3.8 Max Prime**<br>`qwen/qwen3.8-max-prime` | nous_portal | $4.2000 | **51.2** | 19.3 | 75 tps | 1000k | 🚫 Dominated |
| 72 | **Google: Gemini 3.6 Flash**<br>`google/gemini-3.6-flash` | nous_portal_batch | $1.0800 | **51.1** | 29.5 | 150 tps | 1048k | 🚫 Dominated |
| 73 | **Meta: Muse Spark 1.1**<br>`meta/muse-spark-1.1` | nous_portal | $1.4100 | **50.8** | 27.0 | 75 tps | 1048k | 🚫 Dominated |
| 74 | **Claude Sonnet 4.6**<br>`anthropic/claude-sonnet-4.6` | nous_portal_batch | $4.3200 | **49.0** | 18.3 | 150 tps | 1000k | 🚫 Dominated |
| 75 | **OpenAI: GPT-6 Sol**<br>`openai/gpt-6-sol` | nous_portal_batch | $2.8800 | **47.6** | 20.2 | 150 tps | 1050k | 🚫 Dominated |
| 76 | **Qwen: Qwen3.7 Max**<br>`qwen/qwen3.7-max` | nous_portal | $1.5930 | **46.4** | 23.8 | 75 tps | 1000k | 🚫 Dominated |
| 77 | **xAI: Grok 4.7**<br>`x-ai/grok-4.7` | nous_portal | $2.2000 | **46.4** | 21.5 | 75 tps | 500k | 🚫 Dominated |
| 78 | **StepFun: Step 5 Preview**<br>`stepfun/step-5-preview` | nous_portal | $0.9600 | **46.2** | 27.6 | 75 tps | 1000k | 🚫 Dominated |
| 79 | **Sakana: Fugu Max**<br>`sakana/fugu-max` | nous_portal | $2.1000 | **46.2** | 21.7 | 75 tps | 1000k | 🚫 Dominated |
| 80 | **DeepSeek: DeepSeek Pro Latest**<br>`deepseek/deepseek-pro-latest` | nous_portal | $1.7440 | **46.2** | 23.0 | 75 tps | 1048k | 🚫 Dominated |
| 81 | **OpenAI: GPT Terra Latest**<br>`openai/gpt-terra-latest` | nous_portal | $3.2800 | **46.2** | 18.8 | 75 tps | 1050k | 🚫 Dominated |
| 82 | **Z.ai: GLM Latest**<br>`z-ai/glm-latest` | nous_portal | $0.9286 | **46.2** | 27.9 | 75 tps | 1048k | 🚫 Dominated |
| 83 | **Qwen: Qwen3.5 Plus 2026-02-15**<br>`qwen/qwen3.5-plus-02-15` | nous_portal | $0.4368 | **46.2** | 33.9 | 75 tps | 1000k | 🚫 Dominated |
| 84 | **MoonshotAI: Kimi K3**<br>`moonshotai/kimi-k3:US` | nous_portal | $5.2272 | **46.2** | 16.3 | 75 tps | 1048k | 🚫 Dominated |
| 85 | **Poolside: Laguna S 2.1**<br>`poolside/laguna-s-2.1` | openrouter_market | $0.0756 | **46.2** | 43.1 | 70 tps | 1048k | 🚫 Dominated |
| 86 | **Meta: Muse Wonder**<br>`meta/muse-wonder` | nous_portal | $5.6800 | **46.2** | 16.0 | 75 tps | 1048k | 🚫 Dominated |
| 87 | **Anthropic: Claude Fable Latest**<br>`anthropic/claude-fable-latest` | nous_portal | $14.1000 | **46.2** | 12.5 | 75 tps | 1000k | 🚫 Dominated |
| 88 | **Fireworks: Ember-1**<br>`fireworks/ember-1` | nous_portal | $4.3200 | **46.2** | 17.3 | 75 tps | 1048k | 🚫 Dominated |
| 89 | **Qwen: Qwen-Plus**<br>`qwen/qwen-plus` | nous_portal | $0.2808 | **46.2** | 37.1 | 75 tps | 1000k | 🚫 Dominated |
| 90 | **Pareto 26.10 Preview**<br>`unbiased/pareto-26.10-preview` | nous_portal | $0.9720 | **46.2** | 27.5 | 75 tps | 1048k | 🚫 Dominated |
| 91 | **Tencent: Hy4 preview**<br>`tencent/hy4-preview` | nous_portal | $0.4252 | **46.2** | 34.2 | 75 tps | 1048k | 🚫 Dominated |
| 92 | **Sakana: Fugu Ultra v2**<br>`sakana/fugu-ultra-v2` | nous_portal | $8.2000 | **46.2** | 14.4 | 75 tps | 1000k | 🚫 Dominated |
| 93 | **OpenAI: GPT-5.4 Pro**<br>`openai/gpt-5.4-pro` | nous_portal_batch | $50.4000 | **46.2** | 9.4 | 150 tps | 1050k | 🚫 Dominated |
| 94 | **Mistral: Mistral Large 4**<br>`mistralai/mistral-large-4-0` | nous_portal | $0.7180 | **46.2** | 30.0 | 75 tps | 1048k | 🚫 Dominated |
| 95 | **Qwen: Qwen3.5 Plus 2026-04-20**<br>`qwen/qwen3.5-plus-20260420` | nous_portal | $0.5040 | **46.2** | 32.8 | 75 tps | 1000k | 🚫 Dominated |
| 96 | **MoonshotAI: Kimi Latest**<br>`moonshotai/kimi-latest` | nous_portal | $3.4080 | **46.2** | 18.6 | 75 tps | 1048k | 🚫 Dominated |
| 97 | **xAI: Grok Latest**<br>`x-ai/grok-latest` | nous_portal | $2.2000 | **46.2** | 21.4 | 75 tps | 500k | 🚫 Dominated |
| 98 | **OpenAI: GPT-5.6 Terra Pro**<br>`openai/gpt-5.6-terra-pro` | nous_portal_batch | $3.2800 | **46.2** | 18.8 | 150 tps | 1050k | 🚫 Dominated |
| 99 | **Sakana: Fugu Ultra**<br>`sakana/fugu-ultra` | nous_portal | $8.2000 | **46.2** | 14.4 | 75 tps | 1000k | 🚫 Dominated |
| 100 | **OpenAI: GPT-5.5 Pro**<br>`openai/gpt-5.5-pro` | nous_portal_batch | $50.4000 | **46.2** | 9.4 | 150 tps | 1050k | 🚫 Dominated |
| 101 | **Xiaomi: MiMo-V2.6-Pro-UltraSpeed**<br>`xiaomi/mimo-v2.6-pro-ultraspeed` | nous_portal | $3.4944 | **46.2** | 18.5 | 75 tps | 1048k | 🚫 Dominated |
| 102 | **Z.ai: GLM 5.3**<br>`z-ai/glm-5.3:US` | nous_portal | $2.5476 | **46.2** | 20.4 | 75 tps | 1048k | 🚫 Dominated |
| 103 | **MiniMax: MiniMax M3**<br>`minimax/minimax-m3` | nous_portal | $0.3040 | **45.5** | 36.0 | 130 tps | 1048k | 🚫 Dominated |
| 104 | **Gemini 2.0 Flash**<br>`gemini-2.0-flash` | nous_portal | $0.6400 | **45.3** | 30.3 | 130 tps | 1000k | 💎 **Best Value** |
| 105 | **Gemini 2.0 Flash**<br>`gemini-2.0-flash` | nous_portal | $0.6400 | **45.3** | 30.3 | 130 tps | 1000k | 💎 **Best Value** |
| 106 | **AionLabs: Aion 3.5 Mini**<br>`aion-labs/aion-3.5-mini` | nous_portal | $0.6320 | **45.3** | 30.4 | 130 tps | 262k | 🚫 Dominated |
| 107 | **Google: Nano Banana 2.1**<br>`google/gemini-nano-banana-2.1` | nous_portal | $2.2200 | **45.3** | 20.9 | 130 tps | 65k | 🚫 Dominated |
| 108 | **ByteDance Seed: Seed 1.6 Flash**<br>`bytedance-seed/seed-1.6-flash` | nous_portal | $0.0960 | **45.3** | 41.5 | 130 tps | 262k | 🚫 Dominated |
| 109 | **Anthropic: Claude Haiku Latest**<br>`anthropic/claude-haiku-latest` | nous_portal | $0.1440 | **45.3** | 39.9 | 130 tps | 1000k | 🚫 Dominated |
| 110 | **Google: Gemini 3.1 Pro Preview Custom Tools**<br>`google/gemini-3.1-pro-preview-customtools` | nous_portal | $3.2800 | **45.3** | 18.4 | 130 tps | 1048k | 🚫 Dominated |
| 111 | **AionLabs: Aion-3.0-Mini**<br>`aion-labs/aion-3.0-mini` | nous_portal | $0.6320 | **45.3** | 30.4 | 130 tps | 131k | 🚫 Dominated |
| 112 | **Google: Nano Banana Pro (Gemini 3 Pro Image)**<br>`google/gemini-3-pro-image` | nous_portal | $3.2800 | **45.3** | 18.4 | 130 tps | 131k | 🚫 Dominated |
| 113 | **Qwen: Qwen3.8 Flash**<br>`qwen/qwen3.8-flash` | nous_portal | $0.1604 | **45.3** | 39.4 | 130 tps | 1000k | 🚫 Dominated |
| 114 | **DeepSeek: DeepSeek V4 Flash Latest**<br>`deepseek/deepseek-v4-flash-latest` | nous_portal | $0.1253 | **45.3** | 40.5 | 130 tps | 1048k | 🚫 Dominated |
| 115 | **Z.ai: GLM 5.3 FlashX**<br>`z-ai/glm-5.3-flashx` | nous_portal | $0.4340 | **45.3** | 33.3 | 130 tps | 1048k | 🚫 Dominated |
| 116 | **Google: Gemini 3.1 Flash Lite**<br>`google/gemini-3.1-flash-lite` | nous_portal_batch | $0.4100 | **45.3** | 33.7 | 150 tps | 1048k | 🚫 Dominated |
| 117 | **OpenAI: GPT Mini Latest**<br>`openai/gpt-mini-latest` | nous_portal | $1.2300 | **45.3** | 25.1 | 130 tps | 400k | 🚫 Dominated |
| 118 | **OpenAI: GPT Audio Mini**<br>`openai/gpt-audio-mini` | nous_portal | $0.7680 | **45.3** | 28.8 | 130 tps | 128k | 🚫 Dominated |
| 119 | **OpenAI: GPT-4o-mini (2024-07-18)**<br>`openai/gpt-4o-mini-2024-07-18` | nous_portal | $0.2100 | **45.3** | 38.0 | 130 tps | 128k | 🚫 Dominated |
| 120 | **OpenAI: GPT-5.6 Luna Pro**<br>`openai/gpt-5.6-luna-pro` | nous_portal_batch | $0.3280 | **45.3** | 35.3 | 150 tps | 1050k | 🚫 Dominated |
| 121 | **DeepSeek: DeepSeek V4 Flash**<br>`deepseek/deepseek-v4-flash-0731:US` | nous_portal | $0.3676 | **45.3** | 34.5 | 130 tps | 1048k | 🚫 Dominated |
| 122 | **Qwen: Qwen3.5-Flash**<br>`qwen/qwen3.5-flash-02-23` | nous_portal | $0.0832 | **45.3** | 41.9 | 130 tps | 1000k | 🚫 Dominated |
| 123 | **DeepSeek: DeepSeek Flash Latest**<br>`deepseek/deepseek-flash-latest` | openrouter_market | $0.1269 | **45.3** | 40.4 | 130 tps | 1048k | 🚫 Dominated |
| 124 | **Qwen: Qwen3 Coder Flash**<br>`qwen/qwen3-coder-flash` | nous_portal | $0.2886 | **45.3** | 36.1 | 130 tps | 1000k | 🚫 Dominated |
| 125 | **DeepSeek: DeepSeek V4.1 Flash**<br>`deepseek/deepseek-v4.1-flash:US` | nous_portal | $0.3676 | **45.3** | 34.5 | 130 tps | 1048k | 🚫 Dominated |
| 126 | **Google: Gemini Flash Latest**<br>`google/gemini-flash-latest` | nous_portal | $1.0800 | **45.3** | 26.1 | 130 tps | 1048k | 🚫 Dominated |
| 127 | **OpenAI: GPT Luna Latest**<br>`openai/gpt-luna-latest` | nous_portal | $0.1440 | **45.3** | 39.9 | 130 tps | 1050k | 🚫 Dominated |
| 128 | **Z.ai: GLM Flash Latest**<br>`z-ai/glm-flash-latest` | nous_portal | $0.1987 | **45.3** | 38.3 | 130 tps | 1048k | 🚫 Dominated |
| 129 | **ByteDance Seed: Seed-2.0-Mini**<br>`bytedance-seed/seed-2.0-mini` | nous_portal | $0.1280 | **45.3** | 40.4 | 130 tps | 262k | 🚫 Dominated |
| 130 | **Qwen: Qwen3.6 Flash**<br>`qwen/qwen3.6-flash` | nous_portal | $0.3150 | **45.3** | 35.5 | 130 tps | 1000k | 🚫 Dominated |
| 131 | **Google: Gemini Pro Latest**<br>`google/gemini-pro-latest` | nous_portal | $3.2800 | **45.3** | 18.4 | 130 tps | 1048k | 🚫 Dominated |
| 132 | **Z.ai: GLM 5.3 Flash**<br>`z-ai/glm-5.3-flash:US` | nous_portal | $0.2838 | **45.3** | 36.2 | 130 tps | 1048k | 🚫 Dominated |
| 133 | **Qwen: Qwen3.8 Omni Flash**<br>`qwen/qwen3.8-omni-flash` | nous_portal | $0.1604 | **45.3** | 39.4 | 130 tps | 1000k | 🚫 Dominated |
| 134 | **Qwen: Qwen3.7 Flash**<br>`qwen/qwen3.7-flash` | nous_portal | $0.0404 | **45.3** | 43.5 | 130 tps | 1000k | 💎 **Best Value** |
| 135 | **OpenAI: GPT-6 Luna Pro**<br>`openai/gpt-6-luna-pro` | nous_portal_batch | $0.1440 | **45.3** | 39.9 | 150 tps | 1050k | 🚫 Dominated |
| 136 | **Google: Gemini 2.5 Pro Preview 06-05**<br>`google/gemini-2.5-pro-preview` | nous_portal | $2.5500 | **45.3** | 20.0 | 130 tps | 1048k | 🚫 Dominated |
| 137 | **OpenAI: o4 Mini High**<br>`openai/o4-mini-high` | nous_portal | $1.4300 | **45.3** | 24.0 | 130 tps | 200k | 🚫 Dominated |
| 138 | **DeepSeek: DeepSeek V4 Pro 0423**<br>`deepseek/deepseek-v4-pro` | nous_portal | $0.7860 | **44.5** | 28.2 | 75 tps | 1048k | 🚫 Dominated |
| 139 | **inclusionAI: Ling 3.0 Flash VL**<br>`inclusionai/ling-3.0-flash-vl` | nous_portal | $0.0648 | **44.3** | 41.6 | 130 tps | 131k | 🚫 Dominated |
| 140 | **Anthropic: Claude Haiku 5.5**<br>`anthropic/claude-haiku-5.5` | nous_portal_batch | $0.1440 | **43.4** | 38.3 | 150 tps | 1000k | 🚫 Dominated |
| 141 | **MoonshotAI: Kimi K2.6**<br>`moonshotai/kimi-k2.6` | nous_portal | $0.7060 | **43.2** | 28.2 | 75 tps | 262k | 🚫 Dominated |
| 142 | **Tencent: Hy3 preview**<br>`tencent/hy3-preview` | nous_portal | $0.2160 | **43.2** | 36.1 | 75 tps | 262k | 🚫 Dominated |
| 143 | **MoonshotAI: Kimi K2.7 Code**<br>`moonshotai/kimi-k2.7-code` | nous_portal | $1.0105 | **42.9** | 25.3 | 75 tps | 262k | 🚫 Dominated |
| 144 | **Xiaomi: MiMo-V2.5-Pro**<br>`xiaomi/mimo-v2.5-pro` | nous_portal | $0.2447 | **42.7** | 35.0 | 75 tps | 1050k | 🚫 Dominated |
| 145 | **Upstage: Solar Pro 4**<br>`upstage/solar-pro4` | nous_portal | $0.1152 | **41.7** | 37.6 | 90 tps | 524k | 🚫 Dominated |
| 146 | **Google: Gemini 3.1 Pro Preview**<br>`google/gemini-3.1-pro-preview` | nous_portal_batch | $3.2800 | **41.5** | 16.9 | 150 tps | 1048k | 🚫 Dominated |
| 147 | **Google: Gemini Embedding 2 Preview**<br>`google/gemini-embedding-2-preview` | nous_portal | $0.0960 | **41.5** | 38.0 | 130 tps | 8k | 🚫 Dominated |
| 148 | **Google: Gemini Embedding 001**<br>`google/gemini-embedding-001` | nous_portal | $0.0720 | **41.5** | 38.8 | 130 tps | 20k | 🚫 Dominated |
| 149 | **Google: Gemini Embedding 2**<br>`google/gemini-embedding-2` | nous_portal_batch | $0.0960 | **41.5** | 38.0 | 150 tps | 8k | 🚫 Dominated |
| 150 | **Sentence Transformers: all-MiniLM-L12-v2**<br>`sentence-transformers/all-minilm-l12-v2` | nous_portal | $0.0024 | **41.5** | 41.4 | 130 tps | 0k | 💎 **Best Value** |
| 151 | **Sentence Transformers: paraphrase-MiniLM-L6-v2**<br>`sentence-transformers/paraphrase-minilm-l6-v2` | nous_portal | $0.0024 | **41.5** | 41.4 | 130 tps | 0k | 💎 **Best Value** |
| 152 | **Sentence Transformers: all-MiniLM-L6-v2**<br>`sentence-transformers/all-minilm-l6-v2` | nous_portal | $0.0024 | **41.5** | 41.4 | 130 tps | 0k | 💎 **Best Value** |
| 153 | **Z.ai: GLM 5.1**<br>`z-ai/glm-5.1` | nous_portal | $1.0654 | **41.4** | 24.0 | 75 tps | 204k | 🚫 Dominated |
| 154 | **DeepSeek V3**<br>`deepseek-v3` | nous_portal | $0.6400 | **41.2** | 27.6 | 92 tps | 64k | 🚫 Dominated |
| 155 | **GPT-4o**<br>`gpt-4o` | nous_portal | $0.6400 | **41.2** | 27.6 | 75 tps | 128k | 🚫 Dominated |
| 156 | **GPT-4o**<br>`gpt-4o` | nous_portal | $0.6400 | **41.2** | 27.6 | 75 tps | 128k | 🚫 Dominated |
| 157 | **DeepSeek V3**<br>`deepseek-v3` | nous_portal | $0.6400 | **41.2** | 27.6 | 92 tps | 64k | 🚫 Dominated |
| 158 | **ByteDance Seed: Seed-2.0-Lite**<br>`bytedance-seed/seed-2.0-lite` | nous_portal | $0.5200 | **41.2** | 29.1 | 75 tps | 262k | 🚫 Dominated |
| 159 | **OpenAI: gpt-oss-safeguard-20b**<br>`openai/gpt-oss-safeguard-20b` | nous_portal | $0.1050 | **41.2** | 37.5 | 75 tps | 131k | 🚫 Dominated |
| 160 | **Pareto Latest**<br>`unbiased/pareto-latest` | nous_portal | $2.6000 | **41.2** | 18.1 | 75 tps | 262k | 🚫 Dominated |
| 161 | **Qwen: Qwen2.5 7B Instruct**<br>`qwen/qwen-2.5-7b-instruct` | nous_portal | $0.0880 | **41.2** | 38.0 | 75 tps | 32k | 🚫 Dominated |
| 162 | **Nex AGI: Nex-N2.5-Pro**<br>`nex-agi/nex-n2.5-pro` | nous_portal | $0.0860 | **41.2** | 38.1 | 75 tps | 262k | 🚫 Dominated |
| 163 | **Poolside: Laguna XS 2.1**<br>`poolside/laguna-xs-2.1` | openrouter_market | $0.0600 | **41.2** | 39.0 | 70 tps | 262k | 🚫 Dominated |
| 164 | **ByteDance Seed: Seed 1.6**<br>`bytedance-seed/seed-1.6` | nous_portal | $0.5200 | **41.2** | 29.1 | 75 tps | 262k | 🚫 Dominated |
| 165 | **Mistral: Mistral Small 3.2 24B**<br>`mistralai/mistral-small-3.2-24b-instruct` | nous_portal | $0.0760 | **41.2** | 38.4 | 75 tps | 256k | 🚫 Dominated |
| 166 | **OpenAI: GPT-3.5 Turbo 16k**<br>`openai/gpt-3.5-turbo-16k` | nous_portal | $2.2400 | **41.2** | 19.0 | 75 tps | 16k | 🚫 Dominated |
| 167 | **Cohere: Command A+**<br>`cohere/command-a-plus` | nous_portal | $0.4800 | **41.2** | 29.6 | 75 tps | 192k | 🚫 Dominated |
| 168 | **OpenAI: GPT-5 Pro**<br>`openai/gpt-5-pro` | nous_portal_batch | $31.2000 | **41.2** | 9.2 | 150 tps | 400k | 🚫 Dominated |
| 169 | **Pareto 26.9**<br>`unbiased/pareto-26.9` | nous_portal | $2.6000 | **41.2** | 18.1 | 75 tps | 262k | 🚫 Dominated |
| 170 | **AionLabs: Aion-2.0**<br>`aion-labs/aion-2.0` | nous_portal | $0.7200 | **41.2** | 26.7 | 75 tps | 131k | 🚫 Dominated |
| 171 | **ByteDance Seed: Seed 2.1 Turbo**<br>`bytedance-seed/seed-2-1-turbo` | nous_portal | $0.7400 | **41.2** | 26.5 | 75 tps | 262k | 🚫 Dominated |
| 172 | **OpenAI: GPT-5.2 Chat**<br>`openai/gpt-5.2-chat` | nous_portal | $3.5700 | **41.2** | 16.4 | 75 tps | 128k | 🚫 Dominated |
| 173 | **Mistral: Voxtral Small 24B 2507**<br>`mistralai/voxtral-small-24b-2507` | nous_portal | $0.1040 | **41.2** | 37.5 | 75 tps | 32k | 🚫 Dominated |
| 174 | **MissingNo**<br>`stealth/missingno` | nous_portal | $0.0010 | **41.2** | 41.2 | 75 tps | 262k | 💎 **Best Value** |
| 175 | **Cohere: Command R+ (08-2024)**<br>`cohere/command-r-plus-08-2024` | nous_portal | $3.2000 | **41.2** | 16.9 | 75 tps | 128k | 🚫 Dominated |
| 176 | **OpenAI: GPT-5.2 Pro**<br>`openai/gpt-5.2-pro` | nous_portal_batch | $43.6800 | **41.2** | 8.6 | 150 tps | 400k | 🚫 Dominated |
| 177 | **PrismML: Ternary Bonsai 2 27B**<br>`prism-ml/ternary-bonsai-2-27b` | nous_portal | $0.1450 | **41.2** | 36.3 | 75 tps | 262k | 🚫 Dominated |
| 178 | **ByteDance Seed: Seed-2.0-Code**<br>`bytedance-seed/seed-2.0-code` | nous_portal | $0.8400 | **41.2** | 25.6 | 75 tps | 262k | 🚫 Dominated |
| 179 | **DeepSeek: DeepSeek V3**<br>`deepseek/deepseek-chat` | nous_portal | $0.3293 | **41.2** | 32.1 | 75 tps | 163k | 🚫 Dominated |
| 180 | **OpenAI: GPT-5.1-Codex-Max**<br>`openai/gpt-5.1-codex-max` | nous_portal | $2.5500 | **41.2** | 18.2 | 75 tps | 400k | 🚫 Dominated |
| 181 | **OpenAI: GPT Chat Latest**<br>`openai/gpt-chat-latest` | nous_portal | $8.2000 | **41.2** | 12.8 | 75 tps | 400k | 🚫 Dominated |
| 182 | **Reka Edge**<br>`rekaai/reka-edge` | nous_portal | $0.0680 | **41.2** | 38.7 | 75 tps | 16k | 🚫 Dominated |
| 183 | **Mistral: Mistral Nemo**<br>`mistralai/mistral-nemo` | nous_portal | $0.0153 | **41.2** | 40.6 | 75 tps | 131k | 🚫 Dominated |
| 184 | **AionLabs: Aion 3.5**<br>`aion-labs/aion-3.5` | nous_portal | $2.7000 | **41.2** | 17.9 | 75 tps | 262k | 🚫 Dominated |
| 185 | **Mistral: Codestral 2508**<br>`mistralai/codestral-2508` | nous_portal_batch | $0.3120 | **41.2** | 32.4 | 150 tps | 256k | 🚫 Dominated |
| 186 | **Perceptron: Perceptron Mk1.5**<br>`perceptron/perceptron-mk1.5` | nous_portal | $0.3720 | **41.2** | 31.3 | 75 tps | 36k | 🚫 Dominated |
| 187 | **OpenAI: GPT-3.5 Turbo (older v0613)**<br>`openai/gpt-3.5-turbo-0613` | nous_portal | $0.8800 | **41.2** | 25.3 | 75 tps | 4k | 🚫 Dominated |
| 188 | **LiquidAI: LFM2.5-24B-A2B**<br>`liquid/lfm-2.5-24b-a2b` | nous_portal | $0.1700 | **41.2** | 35.7 | 75 tps | 128k | 🚫 Dominated |
| 189 | **Mistral: Mistral Small 3.1 24B**<br>`mistralai/mistral-small-3.1-24b-instruct` | nous_portal | $0.2795 | **41.2** | 33.1 | 75 tps | 128k | 🚫 Dominated |
| 190 | **Relace: Relace Search**<br>`relace/relace-search` | nous_portal | $1.0800 | **41.2** | 23.8 | 75 tps | 256k | 🚫 Dominated |
| 191 | **AionLabs: Aion-3.0**<br>`aion-labs/aion-3.0` | nous_portal | $2.7000 | **41.2** | 17.9 | 75 tps | 131k | 🚫 Dominated |
| 192 | **OpenAI: GPT-4o**<br>`openai/gpt-4o` | nous_portal_batch | $3.2000 | **41.2** | 16.9 | 150 tps | 128k | 🚫 Dominated |
| 193 | **OpenAI: GPT Audio**<br>`openai/gpt-audio` | nous_portal | $3.2000 | **41.2** | 16.9 | 75 tps | 128k | 🚫 Dominated |
| 194 | **Cohere: Command R (08-2024)**<br>`cohere/command-r-08-2024` | nous_portal | $0.1920 | **41.2** | 35.1 | 75 tps | 128k | 🚫 Dominated |
| 195 | **inclusionAI: Ling 3.1 Flash**<br>`inclusionai/ling-3.1-flash` | nous_portal | $0.0810 | **41.1** | 38.1 | 130 tps | 262k | 🚫 Dominated |
| 196 | **SpaceXAI: Grok Build 0.1**<br>`x-ai/grok-build-0.1` | nous_portal | $0.8800 | **40.6** | 24.9 | 75 tps | 256k | 🚫 Dominated |
| 197 | **DeepSeek: DeepSeek V4 Flash 0423**<br>`deepseek/deepseek-v4-flash` | nous_portal | $0.2517 | **40.4** | 33.0 | 130 tps | 1048k | 🚫 Dominated |
| 198 | **Thinking Machines: Inkling Small**<br>`thinkingmachines/inkling-small` | openrouter_market | $0.4600 | **39.7** | 28.8 | 70 tps | 524k | 🚫 Dominated |
| 199 | **DeepSeek: DeepSeek V4.1 Flash**<br>`deepseek/deepseek-v4.1-flash` | openrouter_market | $0.1466 | **39.5** | 34.7 | 130 tps | 1048k | 🚫 Dominated |
| 200 | **OpenAI: GPT-5.4 Mini**<br>`openai/gpt-5.4-mini` | nous_portal_batch | $1.2300 | **38.9** | 21.6 | 150 tps | 400k | 🚫 Dominated |
| 201 | **Thinking Machines: Inkling**<br>`thinkingmachines/inkling` | openrouter_market | $1.2540 | **38.8** | 21.4 | 70 tps | 524k | 🚫 Dominated |
| 202 | **Qwen: Qwen3.7 Plus**<br>`qwen/qwen3.7-plus` | nous_portal | $0.4096 | **38.6** | 28.7 | 75 tps | 1000k | 🚫 Dominated |
| 203 | **Xiaomi: MiMo-V2.5**<br>`xiaomi/mimo-v2.5` | nous_portal | $0.0962 | **38.3** | 35.1 | 75 tps | 1050k | 🚫 Dominated |
| 204 | **OpenAI: GPT-5.1**<br>`openai/gpt-5.1` | nous_portal_batch | $2.5520 | **38.3** | 16.9 | 150 tps | 400k | 🚫 Dominated |
| 205 | **OpenAI: GPT-6 Luna**<br>`openai/gpt-6-luna` | nous_portal_batch | $0.1440 | **38.1** | 33.6 | 150 tps | 1050k | 🚫 Dominated |
| 206 | **OpenAI: GPT-5.4 Nano**<br>`openai/gpt-5.4-nano` | nous_portal_batch | $0.3380 | **38.1** | 29.5 | 150 tps | 400k | 🚫 Dominated |
| 207 | **Qwen: Qwen3.6 27B**<br>`qwen/qwen3.6-27b` | nous_portal | $0.5100 | **37.9** | 26.8 | 75 tps | 262k | 🚫 Dominated |
| 208 | **Qwen: Qwen3 Embedding 8B**<br>`qwen/qwen3-embedding-8b` | nous_portal | $0.0048 | **37.5** | 37.3 | 75 tps | 32k | 🚫 Dominated |
| 209 | **Intfloat: E5-Base-v2**<br>`intfloat/e5-base-v2` | nous_portal | $0.0024 | **37.5** | 37.4 | 75 tps | 0k | 🚫 Dominated |
| 210 | **OpenAI: Text Embedding Ada 002**<br>`openai/text-embedding-ada-002` | nous_portal_batch | $0.0480 | **37.5** | 35.8 | 150 tps | 8k | 🚫 Dominated |
| 211 | **Sentence Transformers: multi-qa-mpnet-base-dot-v1**<br>`sentence-transformers/multi-qa-mpnet-base-dot-v1` | nous_portal | $0.0024 | **37.5** | 37.4 | 75 tps | 0k | 🚫 Dominated |
| 212 | **BAAI: bge-base-en-v1.5**<br>`baai/bge-base-en-v1.5` | nous_portal | $0.0024 | **37.5** | 37.4 | 75 tps | 0k | 🚫 Dominated |
| 213 | **Intfloat: E5-Large-v2**<br>`intfloat/e5-large-v2` | nous_portal | $0.0048 | **37.5** | 37.3 | 75 tps | 0k | 🚫 Dominated |
| 214 | **VoyageAI by MongoDB: voyage-multimodal-3.5**<br>`voyageai/voyage-multimodal-3.5` | nous_portal | $0.0576 | **37.5** | 35.5 | 75 tps | 32k | 🚫 Dominated |
| 215 | **Sentence Transformers: all-mpnet-base-v2**<br>`sentence-transformers/all-mpnet-base-v2` | nous_portal | $0.0024 | **37.5** | 37.4 | 75 tps | 0k | 🚫 Dominated |
| 216 | **Perplexity: Embed V1 4B**<br>`perplexity/pplx-embed-v1-4b` | nous_portal | $0.0144 | **37.5** | 37.0 | 75 tps | 32k | 🚫 Dominated |
| 217 | **OpenAI: Text Embedding 3 Large**<br>`openai/text-embedding-3-large` | nous_portal_batch | $0.0624 | **37.5** | 35.4 | 150 tps | 8k | 🚫 Dominated |
| 218 | **Perplexity: Embed V1 0.6B**<br>`perplexity/pplx-embed-v1-0.6b` | nous_portal | $0.0019 | **37.5** | 37.4 | 75 tps | 32k | 🚫 Dominated |
| 219 | **Intfloat: Multilingual-E5-Large**<br>`intfloat/multilingual-e5-large` | nous_portal | $0.0048 | **37.5** | 37.3 | 75 tps | 0k | 🚫 Dominated |
| 220 | **BAAI: bge-large-en-v1.5**<br>`baai/bge-large-en-v1.5` | nous_portal | $0.0048 | **37.5** | 37.3 | 75 tps | 0k | 🚫 Dominated |
| 221 | **VoyageAI by MongoDB: voyage-4-lite**<br>`voyageai/voyage-4-lite` | nous_portal | $0.0096 | **37.5** | 37.1 | 75 tps | 32k | 🚫 Dominated |
| 222 | **VoyageAI by MongoDB: voyage-code-4**<br>`voyageai/voyage-code-4` | nous_portal | $0.0576 | **37.5** | 35.5 | 75 tps | 32k | 🚫 Dominated |
| 223 | **BAAI: bge-m3**<br>`baai/bge-m3` | nous_portal | $0.0048 | **37.5** | 37.3 | 75 tps | 8k | 🚫 Dominated |
| 224 | **OpenAI: Text Embedding 3 Small**<br>`openai/text-embedding-3-small` | nous_portal | $0.0096 | **37.5** | 37.1 | 75 tps | 8k | 🚫 Dominated |
| 225 | **Thenlper: GTE-Large**<br>`thenlper/gte-large` | nous_portal | $0.0048 | **37.5** | 37.3 | 75 tps | 0k | 🚫 Dominated |
| 226 | **VoyageAI by MongoDB: voyage-4-large**<br>`voyageai/voyage-4-large` | nous_portal | $0.0576 | **37.5** | 35.5 | 75 tps | 32k | 🚫 Dominated |
| 227 | **Mistral: Codestral Embed 2505**<br>`mistralai/codestral-embed-2505` | nous_portal | $0.0720 | **37.5** | 35.1 | 75 tps | 8k | 🚫 Dominated |
| 228 | **VoyageAI by MongoDB: voyage-4**<br>`voyageai/voyage-4` | nous_portal | $0.0288 | **37.5** | 36.5 | 75 tps | 32k | 🚫 Dominated |
| 229 | **Qwen: Qwen3 Embedding 4B**<br>`qwen/qwen3-embedding-4b` | nous_portal | $0.0096 | **37.5** | 37.1 | 75 tps | 32k | 🚫 Dominated |
| 230 | **Thenlper: GTE-Base**<br>`thenlper/gte-base` | nous_portal | $0.0024 | **37.5** | 37.4 | 75 tps | 0k | 🚫 Dominated |
| 231 | **Mistral: Mistral Embed 2312**<br>`mistralai/mistral-embed-2312` | nous_portal | $0.0480 | **37.5** | 35.8 | 75 tps | 8k | 🚫 Dominated |
| 232 | **inclusionAI: Ling 3.0 Flash**<br>`inclusionai/ling-3.0-flash` | nous_portal | $0.0227 | **36.5** | 35.7 | 130 tps | 262k | 🚫 Dominated |
| 233 | **MoonshotAI: Kimi K2.5**<br>`moonshotai/kimi-k2.5` | nous_portal | $0.7760 | **36.3** | 23.1 | 75 tps | 262k | 🚫 Dominated |
| 234 | **NVIDIA: Nemotron 3 Ultra**<br>`nvidia/nemotron-3-ultra-550b-a55b` | openrouter_market | $0.6800 | **36.2** | 23.8 | 70 tps | 262k | 🚫 Dominated |
| 235 | **MiniMax: MiniMax M2.7**<br>`minimax/minimax-m2.7` | nous_portal | $0.2688 | **35.8** | 28.9 | 130 tps | 204k | 🚫 Dominated |
| 236 | **Anthropic: Claude Sonnet 4.5**<br>`anthropic/claude-sonnet-4.5` | nous_portal_batch | $4.3200 | **35.8** | 13.4 | 150 tps | 1000k | 🚫 Dominated |
| 237 | **DeepSeek: DeepSeek V3.2**<br>`deepseek/deepseek-v3.2` | nous_portal | $0.1541 | **34.0** | 29.7 | 75 tps | 163k | 🚫 Dominated |
| 238 | **Google: Gemini 3.5 Flash**<br>`google/gemini-3.5-flash` | nous_portal_batch | $2.4600 | **33.6** | 15.0 | 150 tps | 1048k | 🚫 Dominated |
| 239 | **Google: Gemini 3.5 Flash Lite**<br>`google/gemini-3.5-flash-lite` | nous_portal_batch | $0.6320 | **33.5** | 22.5 | 150 tps | 1048k | 🚫 Dominated |
| 240 | **OpenAI: GPT-5.3-Codex**<br>`openai/gpt-5.3-codex` | nous_portal | $3.5700 | **32.5** | 12.9 | 75 tps | 400k | 🚫 Dominated |
| 241 | **Claude Opus 4.6**<br>`anthropic/claude-opus-4.6` | nous_portal_batch | $7.2000 | **31.9** | 10.3 | 150 tps | 1000k | 🚫 Dominated |
| 242 | **Qwen: Qwen3.5-122B-A10B**<br>`qwen/qwen3.5-122b-a10b` | nous_portal | $0.5408 | **31.8** | 22.2 | 75 tps | 262k | 🚫 Dominated |
| 243 | **Meituan: LongCat 2.0**<br>`meituan/longcat-2.0` | nous_portal | $0.3624 | **31.2** | 23.8 | 75 tps | 1048k | 🚫 Dominated |
| 244 | **OpenAI: GPT-5**<br>`openai/gpt-5` | nous_portal_batch | $2.5500 | **31.1** | 13.7 | 150 tps | 400k | 🚫 Dominated |
| 245 | **Meta: Muse Glimmer 30B**<br>`meta/muse-glimmer-30b` | nous_portal | $0.3760 | **30.8** | 23.3 | 75 tps | 131k | 🚫 Dominated |
| 246 | **StepFun: Step 3.7 Flash**<br>`stepfun/step-3.7-flash` | nous_portal | $0.3260 | **30.6** | 23.8 | 130 tps | 262k | 🚫 Dominated |
| 247 | **OpenAI: GPT-5.2**<br>`openai/gpt-5.2` | nous_portal_batch | $3.5700 | **30.4** | 12.1 | 150 tps | 400k | 🚫 Dominated |
| 248 | **SpaceXAI: Grok 4.3**<br>`x-ai/grok-4.3` | nous_portal_batch | $1.0800 | **30.2** | 17.4 | 150 tps | 1000k | 🚫 Dominated |
| 249 | **Anthropic: Claude Sonnet 4**<br>`anthropic/claude-sonnet-4` | nous_portal | $4.3200 | **29.2** | 10.9 | 75 tps | 200k | 🚫 Dominated |
| 250 | **Anthropic: Claude Opus 4.5**<br>`anthropic/claude-opus-4.5` | nous_portal_batch | $7.2000 | **29.1** | 9.4 | 150 tps | 200k | 🚫 Dominated |
| 251 | **Mistral: Mistral Medium 3.5**<br>`mistralai/mistral-medium-3-5` | nous_portal_batch | $2.2200 | **28.9** | 13.3 | 150 tps | 262k | 🚫 Dominated |
| 252 | **Qwen: Qwen3.6 35B A3B**<br>`qwen/qwen3.6-35b-a3b` | nous_portal | $0.1700 | **28.9** | 25.0 | 75 tps | 262k | 🚫 Dominated |
| 253 | **OpenAI: o1**<br>`openai/o1` | nous_portal | $21.0000 | **28.7** | 7.0 | 75 tps | 200k | 🚫 Dominated |
| 254 | **OpenAI: GPT-5.2-Codex**<br>`openai/gpt-5.2-codex` | nous_portal | $3.5700 | **28.5** | 11.3 | 75 tps | 400k | 🚫 Dominated |
| 255 | **Claude Haiku 4.5**<br>`anthropic/claude-haiku-4.5` | nous_portal_batch | $1.4400 | **27.7** | 14.7 | 150 tps | 409k | 🚫 Dominated |
| 256 | **DeepSeek: DeepSeek V3.1 Terminus**<br>`deepseek/deepseek-v3.1-terminus` | nous_portal | $0.3296 | **27.0** | 21.0 | 75 tps | 163k | 🚫 Dominated |
| 257 | **Z.ai: GLM 5 Turbo**<br>`z-ai/glm-5-turbo` | nous_portal | $1.3760 | **26.6** | 14.3 | 75 tps | 202k | 🚫 Dominated |
| 258 | **Google: Gemma 4 31B**<br>`google/gemma-4-31b-it` | openrouter_market | $0.1240 | **25.8** | 23.1 | 70 tps | 262k | 🚫 Dominated |
| 259 | **SpaceXAI: Grok 4.20**<br>`x-ai/grok-4.20` | nous_portal | $1.0800 | **25.7** | 14.8 | 75 tps | 2000k | 🚫 Dominated |
| 260 | **Upstage: Solar Mini 4**<br>`upstage/solar-mini4` | nous_portal | $0.0620 | **24.1** | 22.7 | 130 tps | 524k | 🚫 Dominated |
| 261 | **OpenAI: GPT-5.1-Codex**<br>`openai/gpt-5.1-codex` | nous_portal | $2.5520 | **23.7** | 10.5 | 75 tps | 400k | 🚫 Dominated |
| 262 | **Google: Gemma 4 26B A4B **<br>`google/gemma-4-26b-a4b-it` | openrouter_market | $0.0692 | **22.9** | 21.5 | 70 tps | 262k | 🚫 Dominated |
| 263 | **Qwen: Qwen3.5-27B**<br>`qwen/qwen3.5-27b` | nous_portal | $0.4056 | **22.9** | 17.1 | 75 tps | 262k | 🚫 Dominated |
| 264 | **MiniMax: MiniMax M2.5**<br>`minimax/minimax-m2.5` | nous_portal | $0.3100 | **22.8** | 18.0 | 130 tps | 204k | 🚫 Dominated |
| 265 | **Anthropic: Claude Opus 4.1**<br>`anthropic/claude-opus-4.1` | nous_portal_batch | $21.6000 | **22.8** | 5.5 | 150 tps | 200k | 🚫 Dominated |
| 266 | **MoonshotAI: Kimi K2 Thinking**<br>`moonshotai/kimi-k2-thinking` | nous_portal | $0.7880 | **22.0** | 13.9 | 75 tps | 262k | 🚫 Dominated |
| 267 | **OpenAI: o3 Pro**<br>`openai/o3-pro` | nous_portal | $25.6000 | **21.9** | 5.1 | 75 tps | 200k | 🚫 Dominated |
| 268 | **NVIDIA: Nemotron 3 Super**<br>`nvidia/nemotron-3-super-120b-a12b` | openrouter_market | $0.1284 | **21.5** | 19.2 | 70 tps | 262k | 🚫 Dominated |
| 269 | **MiniMax: MiniMax M2.1**<br>`minimax/minimax-m2.1` | nous_portal | $0.3720 | **20.9** | 15.9 | 130 tps | 204k | 🚫 Dominated |
| 270 | **OpenAI: GPT-5 Mini**<br>`openai/gpt-5-mini` | nous_portal_batch | $0.5120 | **20.6** | 14.6 | 150 tps | 400k | 🚫 Dominated |
| 271 | **OpenAI: GPT-5.1-Codex-Mini**<br>`openai/gpt-5.1-codex-mini` | nous_portal | $0.5120 | **20.4** | 14.4 | 130 tps | 400k | 🚫 Dominated |
| 272 | **Qwen: Qwen3 Coder Next**<br>`qwen/qwen3-coder-next` | nous_portal | $0.2360 | **20.3** | 16.8 | 90 tps | 262k | 🚫 Dominated |
| 273 | **OpenAI: o3**<br>`openai/o3` | nous_portal_batch | $2.6000 | **20.2** | 8.9 | 150 tps | 200k | 🚫 Dominated |
| 274 | **Google: Gemini 3.1 Flash Lite Preview**<br>`google/gemini-3.1-flash-lite-preview` | nous_portal | $0.4100 | **19.8** | 14.7 | 130 tps | 1048k | 🚫 Dominated |
| 275 | **Qwen: Qwen3.5-35B-A3B**<br>`qwen/qwen3.5-35b-a3b` | nous_portal | $0.1980 | **19.3** | 16.3 | 75 tps | 262k | 🚫 Dominated |
| 276 | **Google: Gemini 2.5 Pro**<br>`google/gemini-2.5-pro` | nous_portal_batch | $2.5500 | **19.0** | 8.4 | 150 tps | 1048k | 🚫 Dominated |
| 277 | **Qwen: Qwen3.5-9B**<br>`qwen/qwen3.5-9b` | nous_portal | $0.0740 | **18.9** | 17.6 | 75 tps | 262k | 🚫 Dominated |
| 278 | **Amazon: Nova 2 Lite**<br>`amazon/nova-2-lite-v1` | nous_portal | $0.6440 | **18.7** | 12.5 | 75 tps | 1000k | 🚫 Dominated |
| 279 | **MiniMax: MiniMax M2**<br>`minimax/minimax-m2` | nous_portal | $0.3264 | **18.6** | 14.5 | 130 tps | 204k | 🚫 Dominated |
| 280 | **OpenAI: gpt-oss-120b**<br>`openai/gpt-oss-120b` | nous_portal_batch | $0.0540 | **18.4** | 17.5 | 150 tps | 131k | 🚫 Dominated |
| 281 | **Mistral: Devstral 2 2512**<br>`mistralai/devstral-2512` | nous_portal | $0.5760 | **18.2** | 12.5 | 75 tps | 262k | 🚫 Dominated |
| 282 | **Inception: Mercury 2**<br>`inception/mercury-2` | nous_portal | $0.2600 | **17.8** | 14.5 | 75 tps | 128k | 🚫 Dominated |
| 283 | **StepFun: Step 3.5 Flash**<br>`stepfun/step-3.5-flash` | nous_portal | $0.1080 | **17.0** | 15.4 | 130 tps | 262k | 🚫 Dominated |
| 284 | **OpenAI: o4 Mini**<br>`openai/o4-mini` | nous_portal_batch | $1.4300 | **16.7** | 8.8 | 150 tps | 200k | 🚫 Dominated |
| 285 | **DeepSeek: DeepSeek V3.2 Exp**<br>`deepseek/deepseek-v3.2-exp` | nous_portal | $0.2116 | **16.6** | 13.9 | 75 tps | 163k | 🚫 Dominated |
| 286 | **OpenAI: GPT-4o (2024-05-13)**<br>`openai/gpt-4o-2024-05-13` | nous_portal | $5.4000 | **16.6** | 5.8 | 75 tps | 128k | 🚫 Dominated |
| 287 | **NVIDIA: Nemotron 3.5 Lightning**<br>`nvidia/nemotron-3.5-lightning` | openrouter_market | $0.0390 | **16.3** | 15.7 | 70 tps | 262k | 🚫 Dominated |
| 288 | **OpenAI: GPT-4.1 Mini**<br>`openai/gpt-4.1-mini` | nous_portal_batch | $0.5200 | **15.7** | 11.1 | 150 tps | 1047k | 🚫 Dominated |
| 289 | **Google: Gemini 2.5 Flash**<br>`google/gemini-2.5-flash` | nous_portal_batch | $0.6320 | **15.5** | 10.4 | 150 tps | 1048k | 🚫 Dominated |
| 290 | **MoonshotAI: Kimi K2 0905**<br>`moonshotai/kimi-k2-0905` | nous_portal | $0.7880 | **15.3** | 9.7 | 75 tps | 262k | 🚫 Dominated |
| 291 | **Mistral: Mistral Small 4**<br>`mistralai/mistral-small-2603` | nous_portal_batch | $0.1860 | **15.0** | 12.8 | 150 tps | 262k | 🚫 Dominated |
| 292 | **OpenAI: GPT-4 Turbo**<br>`openai/gpt-4-turbo` | nous_portal_batch | $10.8000 | **15.0** | 4.3 | 150 tps | 128k | 🚫 Dominated |
| 293 | **Z.ai: GLM 4.7 Flash**<br>`z-ai/glm-4.7-flash` | nous_portal | $0.1080 | **14.9** | 13.5 | 130 tps | 200k | 🚫 Dominated |
| 294 | **Arcee AI: Trinity Large Thinking**<br>`arcee-ai/trinity-large-thinking` | nous_portal | $0.2840 | **14.6** | 11.7 | 75 tps | 262k | 🚫 Dominated |
| 295 | **Qwen: Qwen3 Next 80B A3B Thinking**<br>`qwen/qwen3-next-80b-a3b-thinking` | nous_portal | $0.3120 | **14.6** | 11.5 | 75 tps | 262k | 🚫 Dominated |
| 296 | **DeepSeek R1**<br>`deepseek-r1` | nous_portal | $0.6400 | **14.0** | 9.4 | 75 tps | 64k | 🚫 Dominated |
| 297 | **DeepSeek R1**<br>`deepseek-r1` | nous_portal | $0.6400 | **14.0** | 9.4 | 75 tps | 64k | 🚫 Dominated |
| 298 | **DeepSeek: R1**<br>`deepseek/deepseek-r1` | nous_portal | $0.8360 | **14.0** | 8.7 | 75 tps | 64k | 🚫 Dominated |
| 299 | **DeepSeek: DeepSeek V3.1**<br>`deepseek/deepseek-chat-v3.1` | nous_portal | $0.3420 | **13.7** | 10.6 | 75 tps | 163k | 🚫 Dominated |
| 300 | **DeepSeek: R1 0528**<br>`deepseek/deepseek-r1-0528` | nous_portal | $0.7700 | **13.1** | 8.3 | 75 tps | 163k | 🚫 Dominated |
| 301 | **OpenAI: GPT-5 Nano**<br>`openai/gpt-5-nano` | nous_portal_batch | $0.1040 | **13.0** | 11.8 | 150 tps | 400k | 🚫 Dominated |
| 302 | **IBM: Granite 4.2 8B**<br>`ibm-granite/granite-4.2-8b` | nous_portal | $0.0800 | **12.9** | 12.0 | 75 tps | 131k | 🚫 Dominated |
| 303 | **Qwen: Qwen3 235B A22B Thinking 2507**<br>`qwen/qwen3-235b-a22b-thinking-2507` | nous_portal | $0.9160 | **12.7** | 7.7 | 75 tps | 128k | 🚫 Dominated |
| 304 | **MoonshotAI: Kimi K2 0711**<br>`moonshotai/kimi-k2` | nous_portal | $0.7336 | **12.7** | 8.2 | 75 tps | 131k | 🚫 Dominated |
| 305 | **OpenAI: GPT-4.1**<br>`openai/gpt-4.1` | nous_portal_batch | $2.6000 | **12.7** | 5.6 | 150 tps | 1047k | 🚫 Dominated |
| 306 | **OpenAI o3-mini**<br>`o3-mini` | nous_portal | $0.6400 | **12.5** | 8.4 | 130 tps | 200k | 🚫 Dominated |
| 307 | **OpenAI o3-mini**<br>`o3-mini` | nous_portal | $0.6400 | **12.5** | 8.4 | 130 tps | 200k | 🚫 Dominated |
| 308 | **OpenAI: o3 Mini**<br>`openai/o3-mini` | nous_portal_batch | $1.5400 | **12.5** | 6.5 | 150 tps | 200k | 🚫 Dominated |
| 309 | **Inception: Mercury 2.5**<br>`inception/mercury-2.5` | nous_portal | $0.0476 | **12.3** | 11.8 | 75 tps | 260k | 🚫 Dominated |
| 310 | **Mistral: Mistral Medium 3.1**<br>`mistralai/mistral-medium-3.1` | nous_portal_batch | $0.5760 | **12.1** | 8.3 | 150 tps | 131k | 🚫 Dominated |
| 311 | **DeepSeek: DeepSeek V3 0324**<br>`deepseek/deepseek-chat-v3-0324` | nous_portal | $0.3200 | **12.0** | 9.4 | 75 tps | 163k | 🚫 Dominated |
| 312 | **Qwen: Qwen3 235B A22B Instruct 2507**<br>`qwen/qwen3-235b-a22b-2507` | nous_portal | $0.1120 | **12.0** | 10.8 | 75 tps | 262k | 🚫 Dominated |
| 313 | **Qwen: Qwen3 Coder 480B A35B**<br>`qwen/qwen3-coder` | nous_portal | $0.4656 | **11.9** | 8.6 | 90 tps | 262k | 🚫 Dominated |
| 314 | **MiniMax: MiniMax M1**<br>`minimax/minimax-m1` | nous_portal | $0.6320 | **11.7** | 7.9 | 130 tps | 1000k | 🚫 Dominated |
| 315 | **Mistral: Mistral Large 3 2512**<br>`mistralai/mistral-large-2512` | nous_portal_batch | $0.5200 | **11.5** | 8.1 | 150 tps | 262k | 🚫 Dominated |
| 316 | **Z.ai: GLM 4.6V**<br>`z-ai/glm-4.6v` | nous_portal | $0.3220 | **11.2** | 8.8 | 75 tps | 131k | 🚫 Dominated |
| 317 | **Z.ai: GLM 4.5 Air**<br>`z-ai/glm-4.5-air` | nous_portal | $0.2320 | **11.1** | 9.2 | 75 tps | 131k | 🚫 Dominated |
| 318 | **Google: Gemini 2.5 Flash Lite**<br>`google/gemini-2.5-flash-lite` | nous_portal_batch | $0.1240 | **10.4** | 9.3 | 150 tps | 1048k | 🚫 Dominated |
| 319 | **OpenAI: GPT-4**<br>`openai/gpt-4` | nous_portal | $26.4000 | **10.2** | 2.4 | 75 tps | 8k | 🚫 Dominated |
| 320 | **Llama 3.3 70B Instruct**<br>`llama-3.3-70b` | nous_portal | $0.6400 | **10.0** | 6.7 | 75 tps | 128k | 🚫 Dominated |
| 321 | **Llama 3.3 70B Instruct**<br>`llama-3.3-70b` | nous_portal | $0.6400 | **10.0** | 6.7 | 75 tps | 128k | 🚫 Dominated |
| 322 | **Meta: Llama 3.3 70B Instruct**<br>`meta-llama/llama-3.3-70b-instruct` | nous_portal | $0.1120 | **10.0** | 9.0 | 75 tps | 131k | 🚫 Dominated |
| 323 | **OpenAI: gpt-oss-20b**<br>`openai/gpt-oss-20b` | nous_portal | $0.0288 | **10.0** | 9.7 | 75 tps | 131k | 🚫 Dominated |
| 324 | **Qwen: Qwen3 VL 235B A22B Instruct**<br>`qwen/qwen3-vl-235b-a22b-instruct` | nous_portal | $0.3000 | **9.9** | 7.8 | 75 tps | 262k | 🚫 Dominated |
| 325 | **OpenAI: GPT-4.1 Nano**<br>`openai/gpt-4.1-nano` | nous_portal_batch | $0.1320 | **9.6** | 8.6 | 150 tps | 1047k | 🚫 Dominated |
| 326 | **Qwen: Qwen3 Coder 30B A3B Instruct**<br>`qwen/qwen3-coder-30b-a3b-instruct` | nous_portal | $0.0896 | **9.6** | 8.8 | 90 tps | 262k | 🚫 Dominated |
| 327 | **Qwen: Qwen3 Next 80B A3B Instruct**<br>`qwen/qwen3-next-80b-a3b-instruct` | nous_portal | $0.2632 | **9.6** | 7.8 | 75 tps | 262k | 🚫 Dominated |
| 328 | **Upstage: Solar Pro 3**<br>`upstage/solar-pro-3` | nous_portal | $0.1860 | **9.5** | 8.1 | 90 tps | 131k | 🚫 Dominated |
| 329 | **Qwen: Qwen3 VL 30B A3B Thinking**<br>`qwen/qwen3-vl-30b-a3b-thinking` | nous_portal | $0.3392 | **9.5** | 7.4 | 75 tps | 262k | 🚫 Dominated |
| 330 | **OpenAI: o3 Mini High**<br>`openai/o3-mini-high` | nous_portal | $1.5400 | **9.4** | 4.8 | 130 tps | 200k | 🚫 Dominated |
| 331 | **GPT-4o Mini**<br>`gpt-4o-mini` | openrouter | $0.6400 | **9.3** | 6.2 | 120 tps | 128k | 🚫 Dominated |
| 332 | **GPT-4o Mini**<br>`gpt-4o-mini` | openrouter | $0.6400 | **9.3** | 6.2 | 120 tps | 128k | 🚫 Dominated |
| 333 | **OpenAI: GPT-4o-mini**<br>`openai/gpt-4o-mini` | nous_portal_batch | $0.2100 | **9.3** | 7.8 | 150 tps | 128k | 🚫 Dominated |
| 334 | **Meta: Llama 4 Maverick**<br>`meta-llama/llama-4-maverick` | nous_portal | $0.2255 | **9.2** | 7.7 | 75 tps | 1048k | 🚫 Dominated |
| 335 | **Amazon: Nova Premier 1.0**<br>`amazon/nova-premier-v1` | nous_portal | $3.7500 | **9.2** | 3.6 | 75 tps | 1000k | 🚫 Dominated |
| 336 | **Mistral: Mistral Medium 3**<br>`mistralai/mistral-medium-3` | nous_portal | $0.5760 | **9.0** | 6.2 | 75 tps | 131k | 🚫 Dominated |
| 337 | **Qwen: Qwen3 32B**<br>`qwen/qwen3-32b` | nous_portal | $0.0944 | **8.8** | 8.1 | 75 tps | 131k | 🚫 Dominated |
| 338 | **Mistral: Ministral 3 14B 2512**<br>`mistralai/ministral-14b-2512` | nous_portal | $0.1280 | **8.4** | 7.5 | 130 tps | 262k | 🚫 Dominated |
| 339 | **OpenAI: GPT-4o (2024-11-20)**<br>`openai/gpt-4o-2024-11-20` | nous_portal | $3.5000 | **8.4** | 3.4 | 75 tps | 128k | 🚫 Dominated |
| 340 | **NVIDIA: Nemotron 3 Nano 30B A3B**<br>`nvidia/nemotron-3-nano-30b-a3b` | nous_portal | $0.0640 | **8.4** | 7.9 | 75 tps | 262k | 🚫 Dominated |
| 341 | **OpenAI: GPT-3.5 Turbo**<br>`openai/gpt-3.5-turbo` | nous_portal_batch | $0.5400 | **8.4** | 5.8 | 150 tps | 16k | 🚫 Dominated |
| 342 | **Qwen: Qwen3 14B**<br>`qwen/qwen3-14b` | nous_portal | $0.0920 | **8.0** | 7.3 | 75 tps | 40k | 🚫 Dominated |
| 343 | **Qwen: Qwen3 VL 30B A3B Instruct**<br>`qwen/qwen3-vl-30b-a3b-instruct` | nous_portal | $0.1920 | **7.9** | 6.7 | 75 tps | 262k | 🚫 Dominated |
| 344 | **OpenAI: GPT-4o (2024-08-06)**<br>`openai/gpt-4o-2024-08-06` | nous_portal | $3.5000 | **7.7** | 3.1 | 75 tps | 128k | 🚫 Dominated |
| 345 | **Qwen2.5 72B Instruct**<br>`qwen/qwen-2.5-72b-instruct` | nous_portal | $0.2528 | **7.7** | 6.3 | 75 tps | 32k | 🚫 Dominated |
| 346 | **Qwen: Qwen3 30B A3B**<br>`qwen/qwen3-30b-a3b` | nous_portal | $0.1576 | **7.6** | 6.6 | 75 tps | 40k | 🚫 Dominated |
| 347 | **Z.ai: GLM 4.5V**<br>`z-ai/glm-4.5v` | nous_portal | $0.6440 | **7.6** | 5.1 | 75 tps | 65k | 🚫 Dominated |
| 348 | **Mistral Large 2407**<br>`mistralai/mistral-large-2407` | nous_portal | $2.0800 | **7.6** | 3.6 | 75 tps | 131k | 🚫 Dominated |
| 349 | **Qwen: Qwen3 30B A3B Instruct 2507**<br>`qwen/qwen3-30b-a3b-instruct-2507` | nous_portal | $0.1032 | **7.5** | 6.8 | 75 tps | 262k | 🚫 Dominated |
| 350 | **Hermes 3 (Llama 3.1 405B)**<br>`hermes-3-llama-3.1-405b` | nous_portal | $0.6400 | **7.4** | 5.0 | 40 tps | 128k | 🚫 Dominated |
| 351 | **Hermes 3 (Llama 3.1 405B)**<br>`hermes-3-llama-3.1-405b` | nous_portal | $0.6400 | **7.4** | 5.0 | 40 tps | 128k | 🚫 Dominated |
| 352 | **Qwen: Qwen3 VL 8B Instruct**<br>`qwen/qwen3-vl-8b-instruct` | nous_portal | $0.2980 | **7.3** | 5.8 | 75 tps | 262k | 🚫 Dominated |
| 353 | **Amazon: Nova Pro 1.0**<br>`amazon/nova-pro-v1` | nous_portal | $1.0240 | **7.0** | 4.1 | 75 tps | 300k | 🚫 Dominated |
| 354 | **Qwen 2.5 Coder 32B**<br>`qwen-2.5-coder-32b` | nous_portal | $0.6400 | **6.7** | 4.5 | 105 tps | 131k | 🚫 Dominated |
| 355 | **Hermes 3 (Llama 3.1 70B)**<br>`hermes-3-llama-3.1-70b` | nous_portal | $0.6400 | **6.7** | 4.5 | 78 tps | 128k | 🚫 Dominated |
| 356 | **Hermes 3 (Llama 3.1 70B)**<br>`hermes-3-llama-3.1-70b` | nous_portal | $0.6400 | **6.7** | 4.5 | 78 tps | 128k | 🚫 Dominated |
| 357 | **Qwen 2.5 Coder 32B**<br>`qwen-2.5-coder-32b` | nous_portal | $0.6400 | **6.7** | 4.5 | 105 tps | 131k | 🚫 Dominated |
| 358 | **Amazon: Nova Lite 1.0**<br>`amazon/nova-lite-v1` | nous_portal | $0.0768 | **6.7** | 6.2 | 75 tps | 300k | 🚫 Dominated |
| 359 | **Meta: Llama 3.1 70B Instruct**<br>`meta-llama/llama-3.1-70b-instruct` | nous_portal | $0.2720 | **6.6** | 5.3 | 75 tps | 131k | 🚫 Dominated |
| 360 | **Mistral: Saba**<br>`mistralai/mistral-saba` | nous_portal | $0.2080 | **6.5** | 5.5 | 75 tps | 32k | 🚫 Dominated |
| 361 | **Meta: Llama 3.1 8B Instruct**<br>`meta-llama/llama-3.1-8b-instruct` | nous_portal | $0.0176 | **6.1** | 6.0 | 75 tps | 131k | 🚫 Dominated |
| 362 | **Amazon: Nova Micro 1.0**<br>`amazon/nova-micro-v1` | nous_portal | $0.0448 | **5.9** | 5.7 | 75 tps | 128k | 🚫 Dominated |
| 363 | **Mistral Large**<br>`mistralai/mistral-large` | nous_portal | $2.0800 | **5.8** | 2.7 | 75 tps | 128k | 🚫 Dominated |
| 364 | **Mistral: Mixtral 8x22B Instruct**<br>`mistralai/mixtral-8x22b-instruct` | nous_portal | $2.0800 | **5.7** | 2.7 | 75 tps | 65k | 🚫 Dominated |
| 365 | **Mistral: Ministral 3 8B 2512**<br>`mistralai/ministral-8b-2512` | nous_portal_batch | $0.0960 | **5.6** | 5.1 | 150 tps | 262k | 🚫 Dominated |
| 366 | **Google: Gemma 3 27B**<br>`google/gemma-3-27b-it` | nous_portal | $0.0990 | **5.6** | 5.1 | 75 tps | 131k | 🚫 Dominated |
| 367 | **Meta: Llama 4 Scout**<br>`meta-llama/llama-4-scout` | nous_portal | $0.1080 | **4.7** | 4.3 | 75 tps | 1310k | 🚫 Dominated |
| 368 | **Google: Gemma 3 12B**<br>`google/gemma-3-12b-it` | nous_portal | $0.0540 | **3.2** | 3.1 | 75 tps | 131k | 🚫 Dominated |
| 369 | **Mistral: Ministral 3 3B 2512**<br>`mistralai/ministral-3b-2512` | nous_portal | $0.0640 | **3.0** | 2.8 | 130 tps | 131k | 🚫 Dominated |

#### 🔗 Cadeia de Roteamento Preditivo para Subagentes (`coding`)
| Nível de Escalação | Modelo Designado | Provedor | Custo Blended | Condição de Acionamento |
| :--- | :--- | :--- | :---: | :--- |
| **1. Primário (Custo-Benefício)** | `DeepSeek R1` | nous_portal | $0.7140 | Modelo padrão para subtarefas com SLA normal e orçamento otimizado. |
| **2. Escalação Máxima (Líder)** | `Claude 3.7 Sonnet` | nous_portal | $4.5000 | Acionado se complexidade/entropia for crítica ou falha de teste anterior. |
| **3. Fallback de Baixo Custo** | `Xiaomi: MiMo-V2.6-Flash` | nous_portal | $0.1160 | Standby caso o provedor principal sofra throttling ou indisponibilidade. |

---

### 🤖 2.2 Perfil: Agentic / Tools (Despacho de Ferramentas & MCP)
> **Descrição:** Orquestração de subagentes autônomos, aderência estrita a JSON Schemas e despacho de chamadas de ferramentas MCP.  
> **Métricas Primárias:** BFCL (Berkeley Function Calling) & IFEval &bull; **Fórmula:** `S_agentic = 0.55·BFCL + 0.25·IFEval + 0.20·TTFT_inv`

| Rank | Modelo Canônico | Provedor | Custo Blended ($/1M) | Score Composto | ROI Index | Throughput | Janela Ctx | Status Pareto |
| :---: | :--- | :---: | :---: | :---: | :---: | :---: | :---: | :--- |
| 1 | **Claude 3.7 Sonnet**<br>`claude-3-7-sonnet` | nous_portal | $2.7300 | **88.7** | 38.3 | 75 tps | 200k | 🏆 **Quality Leader** |
| 2 | **OpenAI o3-mini**<br>`o3-mini` | nous_portal | $0.4675 | **87.2** | 63.0 | 130 tps | 200k | 💎 **Best Value** |
| 3 | **Xiaomi: MiMo-V2.6-Pro**<br>`xiaomi/mimo-v2.6-pro` | nous_portal | $0.2182 | **84.9** | 70.9 | 75 tps | 1050k | 🏆 **Quality Leader** |
| 4 | **Google: Gemini 3 Flash Preview**<br>`google/gemini-3-flash-preview` | nous_portal_batch | $0.4800 | **84.8** | 60.9 | 150 tps | 1048k | 🚫 Dominated |
| 5 | **DeepSeek V3**<br>`deepseek-v3` | nous_portal | $0.2360 | **83.2** | 68.7 | 92 tps | 64k | 💎 **Best Value** |
| 6 | **GPT-4o**<br>`gpt-4o` | nous_portal | $1.0625 | **83.1** | 48.2 | 75 tps | 128k | 🚫 Dominated |
| 7 | **Xiaomi: MiMo-V2.6-Flash**<br>`xiaomi/mimo-v2.6-flash` | nous_portal | $0.0880 | **83.1** | 76.6 | 130 tps | 1050k | 💎 **Best Value** |
| 8 | **DeepSeek R1**<br>`deepseek-r1` | nous_portal | $0.4680 | **82.3** | 59.4 | 75 tps | 64k | 🚫 Dominated |
| 9 | **Qwen 2.5 Coder 32B**<br>`qwen-2.5-coder-32b` | nous_portal | $0.1500 | **79.4** | 69.6 | 105 tps | 131k | 💎 **Best Value** |
| 10 | **Llama 3.3 70B Instruct**<br>`llama-3.3-70b` | nous_portal | $0.3445 | **78.5** | 60.6 | 75 tps | 128k | 🚫 Dominated |
| 11 | **Hermes 3 (Llama 3.1 405B)**<br>`hermes-3-llama-3.1-405b` | nous_portal | $0.5500 | **78.2** | 54.3 | 40 tps | 128k | 🚫 Dominated |
| 12 | **GPT-4o Mini**<br>`gpt-4o-mini` | openrouter | $0.0638 | **77.7** | 73.2 | 120 tps | 128k | 💎 **Best Value** |
| 13 | **Z.ai: GLM 5**<br>`z-ai/glm-5` | nous_portal | $0.4440 | **77.6** | 56.8 | 75 tps | 204k | 🚫 Dominated |
| 14 | **Hermes 3 (Llama 3.1 70B)**<br>`hermes-3-llama-3.1-70b` | nous_portal | $0.2490 | **76.7** | 62.8 | 78 tps | 128k | 🚫 Dominated |
| 15 | **Qwen: Qwen3.5 397B A17B**<br>`qwen/qwen3.5-397b-a17b` | nous_portal | $0.3978 | **76.3** | 57.2 | 75 tps | 262k | 🚫 Dominated |
| 16 | **Z.ai: GLM 5V Turbo**<br>`z-ai/glm-5v-turbo` | nous_portal | $0.9040 | **76.2** | 46.4 | 75 tps | 202k | 🚫 Dominated |
| 17 | **Tencent: Hy3**<br>`tencent/hy3` | nous_portal | $0.1122 | **76.2** | 68.9 | 75 tps | 262k | 🚫 Dominated |
| 18 | **OpenAI: GPT-5.4**<br>`openai/gpt-5.4` | nous_portal_batch | $2.4000 | **75.3** | 33.9 | 150 tps | 1050k | 🚫 Dominated |
| 19 | **Z.ai: GLM 4.5**<br>`z-ai/glm-4.5` | nous_portal | $0.4660 | **75.0** | 54.2 | 75 tps | 131k | 🚫 Dominated |
| 20 | **Qwen: Qwen3.6 Plus**<br>`qwen/qwen3.6-plus` | nous_portal | $0.3315 | **68.6** | 53.3 | 75 tps | 1000k | 🚫 Dominated |
| 21 | **Z.ai: GLM 4.6**<br>`z-ai/glm-4.6` | nous_portal | $0.3520 | **66.4** | 51.0 | 75 tps | 204k | 🚫 Dominated |
| 22 | **Z.ai: GLM 4.7**<br>`z-ai/glm-4.7` | nous_portal | $0.3430 | **65.7** | 50.8 | 75 tps | 204k | 🚫 Dominated |
| 23 | **Anthropic: Claude Fable 5.1**<br>`anthropic/claude-fable-5.1` | nous_portal_batch | $8.1500 | **63.9** | 19.9 | 150 tps | 1000k | 🚫 Dominated |
| 24 | **Qwen: Qwen3.8 Max (0902)**<br>`qwen/qwen3.8-max-0902` | nous_portal | $1.3500 | **61.1** | 32.9 | 75 tps | 1000k | 🚫 Dominated |
| 25 | **Meta: Muse Spark 1.3**<br>`meta/muse-spark-1.3` | nous_portal | $0.8900 | **60.7** | 37.1 | 75 tps | 1048k | 🚫 Dominated |
| 26 | **Z.ai: GLM 5.3 Flash**<br>`z-ai/glm-5.3-flash` | nous_portal_batch | $0.0904 | **60.6** | 55.7 | 150 tps | 1048k | 🚫 Dominated |
| 27 | **xAI: Grok 4.6**<br>`x-ai/grok-4.6` | nous_portal | $1.5000 | **59.5** | 31.0 | 75 tps | 500k | 🚫 Dominated |
| 28 | **Anthropic: Claude Opus 5**<br>`anthropic/claude-opus-5` | nous_portal_batch | $4.3000 | **59.4** | 22.3 | 150 tps | 1000k | 🚫 Dominated |
| 29 | **OpenAI: GPT-5.6 Sol**<br>`openai/gpt-5.6-sol` | nous_portal_batch | $1.7200 | **59.1** | 29.5 | 150 tps | 1050k | 🚫 Dominated |
| 30 | **Z.ai: GLM 5.3**<br>`z-ai/glm-5.3` | openrouter_market | $0.6604 | **58.9** | 39.1 | 70 tps | 1048k | 🚫 Dominated |
| 31 | **Claude Fable 5**<br>`anthropic/claude-fable-5` | nous_portal_batch | $8.6000 | **58.0** | 17.8 | 150 tps | 1048k | 🚫 Dominated |
| 32 | **MoonshotAI: Kimi K3**<br>`moonshotai/kimi-k3` | nous_portal | $1.9020 | **57.5** | 27.8 | 75 tps | 1048k | 🚫 Dominated |
| 33 | **DeepSeek: DeepSeek V4 Flash Vision Exp**<br>`deepseek/deepseek-v4-flash-vision-exp` | nous_portal | $0.1335 | **56.6** | 50.3 | 130 tps | 1048k | 🚫 Dominated |
| 34 | **Qwen: Qwen3.8 2.4T A95B**<br>`qwen/qwen3.8-2.4t-a95b` | nous_portal | $1.3500 | **56.3** | 30.3 | 75 tps | 1048k | 🚫 Dominated |
| 35 | **Google: Gemini 4 Argon**<br>`google/gemini-4-argon` | nous_portal_batch | $1.6600 | **55.9** | 28.3 | 150 tps | 1048k | 🚫 Dominated |
| 36 | **OpenAI: GPT-6 Astra**<br>`openai/gpt-6-astra` | nous_portal_batch | $8.6000 | **55.8** | 17.1 | 150 tps | 1050k | 🚫 Dominated |
| 37 | **Google: Gemini 3.8 Flash**<br>`google/gemini-3.8-flash` | nous_portal_batch | $0.6450 | **55.6** | 37.1 | 150 tps | 1048k | 🚫 Dominated |
| 38 | **OpenAI: GPT-5.6 Luna**<br>`openai/gpt-5.6-luna` | nous_portal_batch | $0.1920 | **55.3** | 47.0 | 150 tps | 1050k | 🚫 Dominated |
| 39 | **Anthropic: Claude Sonnet 5.5**<br>`anthropic/claude-sonnet-5.5` | nous_portal_batch | $1.6600 | **55.0** | 27.8 | 150 tps | 1000k | 🚫 Dominated |
| 40 | **Anthropic: Claude Opus 5.5**<br>`anthropic/claude-opus-5.5` | nous_portal_batch | $3.3200 | **54.0** | 21.9 | 150 tps | 1000k | 🚫 Dominated |
| 41 | **DeepSeek: DeepSeek V4 Flash**<br>`deepseek/deepseek-v4-flash-0731` | openrouter_market | $0.0434 | **53.9** | 51.7 | 130 tps | 1048k | 💎 **Best Value** |
| 42 | **Google: Gemini 3.7 Flash**<br>`google/gemini-3.7-flash` | nous_portal_batch | $0.6450 | **53.8** | 35.9 | 150 tps | 1048k | 🚫 Dominated |
| 43 | **OpenAI: GPT-5.6 Terra**<br>`openai/gpt-5.6-terra` | nous_portal_batch | $1.9200 | **53.6** | 25.9 | 150 tps | 1050k | 🚫 Dominated |
| 44 | **OpenAI: GPT Sol Latest**<br>`openai/gpt-sol-latest` | nous_portal | $1.6600 | **53.0** | 26.8 | 90 tps | 1050k | 🚫 Dominated |
| 45 | **OpenAI: GPT-5.6 Sol Pro**<br>`openai/gpt-5.6-sol-pro` | nous_portal_batch | $1.7200 | **53.0** | 26.5 | 150 tps | 1050k | 🚫 Dominated |
| 46 | **OpenAI: GPT-6 Sol Pro**<br>`openai/gpt-6-sol-pro` | nous_portal_batch | $1.7200 | **53.0** | 26.5 | 150 tps | 1050k | 🚫 Dominated |
| 47 | **OpenAI: GPT-6.1 Sol Pro**<br>`openai/gpt-6.1-sol-pro` | nous_portal_batch | $1.6600 | **53.0** | 26.8 | 150 tps | 1050k | 🚫 Dominated |
| 48 | **Qwen: Qwen3.8 27B**<br>`qwen/qwen3.8-27b` | nous_portal | $0.1390 | **52.6** | 46.5 | 75 tps | 1000k | 🚫 Dominated |
| 49 | **OpenAI: GPT-6.1 Sol**<br>`openai/gpt-6.1-sol` | nous_portal_batch | $1.6600 | **52.3** | 26.5 | 150 tps | 1050k | 🚫 Dominated |
| 50 | **Claude Sonnet 5**<br>`anthropic/claude-sonnet-5` | nous_portal_batch | $1.7200 | **52.3** | 26.1 | 150 tps | 1000k | 🚫 Dominated |
| 51 | **Meta: Muse Spark 1.2**<br>`meta/muse-spark-1.2` | nous_portal | $0.8900 | **52.2** | 31.9 | 75 tps | 1048k | 🚫 Dominated |
| 52 | **OpenAI: GPT Astra Latest**<br>`openai/gpt-astra-latest` | nous_portal | $8.6000 | **52.1** | 16.0 | 40 tps | 1050k | 🚫 Dominated |
| 53 | **OpenAI: GPT-6 Astra Pro Flex**<br>`openai/gpt-6-astra-pro-flex` | nous_portal | $4.3000 | **52.1** | 19.5 | 40 tps | 1050k | 🚫 Dominated |
| 54 | **OpenAI: GPT-6 Astra Pro Fast**<br>`openai/gpt-6-astra-pro-fast` | nous_portal | $17.2000 | **52.1** | 13.4 | 40 tps | 1050k | 🚫 Dominated |
| 55 | **OpenAI: GPT-6 Astra Fast**<br>`openai/gpt-6-astra-fast` | nous_portal | $17.2000 | **52.1** | 13.4 | 40 tps | 1050k | 🚫 Dominated |
| 56 | **OpenAI: GPT-6 Astra Flex**<br>`openai/gpt-6-astra-flex` | nous_portal | $4.3000 | **52.1** | 19.5 | 40 tps | 1050k | 🚫 Dominated |
| 57 | **Anthropic: Claude Opus Latest**<br>`anthropic/claude-opus-latest` | nous_portal | $3.3200 | **52.1** | 21.1 | 40 tps | 1000k | 🚫 Dominated |
| 58 | **OpenAI: GPT-6 Astra Pro**<br>`openai/gpt-6-astra-pro` | nous_portal_batch | $8.6000 | **52.1** | 16.0 | 150 tps | 1050k | 🚫 Dominated |
| 59 | **Claude 3.5 Sonnet**<br>`claude-3-5-sonnet` | nous_portal | $2.4600 | **51.9** | 23.2 | 75 tps | 200k | 🏆 **Quality Leader** |
| 60 | **Claude 3.7 Sonnet**<br>`claude-3-7-sonnet` | nous_portal | $2.4600 | **51.9** | 23.2 | 75 tps | 200k | 💎 **Best Value** |
| 61 | **Claude 3.7 Sonnet**<br>`claude-3-7-sonnet` | nous_portal | $2.4600 | **51.9** | 23.2 | 75 tps | 200k | 🏆 **Quality Leader** |
| 62 | **Claude 3.5 Sonnet**<br>`claude-3-5-sonnet` | nous_portal | $2.4600 | **51.9** | 23.2 | 75 tps | 200k | 💎 **Best Value** |
| 63 | **Anthropic: Claude Sonnet Latest**<br>`anthropic/claude-sonnet-latest` | nous_portal | $1.6600 | **51.9** | 26.2 | 75 tps | 1000k | 🚫 Dominated |
| 64 | **Z.ai: GLM 5.3 Prime**<br>`z-ai/glm-5.3-prime` | nous_portal | $2.0560 | **51.9** | 24.5 | 75 tps | 1000k | 🚫 Dominated |
| 65 | **Qwen: Qwen3.8 Max Prime**<br>`qwen/qwen3.8-max-prime` | nous_portal | $2.7000 | **51.9** | 22.5 | 75 tps | 1000k | 🚫 Dominated |
| 66 | **xAI: Grok 4.5**<br>`x-ai/grok-4.5` | nous_portal | $1.3800 | **51.1** | 27.4 | 75 tps | 500k | 🚫 Dominated |
| 67 | **Gemini 2.0 Flash**<br>`gemini-2.0-flash` | nous_portal | $0.4100 | **50.4** | 37.5 | 130 tps | 1000k | 💎 **Best Value** |
| 68 | **Gemini 2.0 Flash**<br>`gemini-2.0-flash` | nous_portal | $0.4100 | **50.4** | 37.5 | 130 tps | 1000k | 💎 **Best Value** |
| 69 | **AionLabs: Aion 3.5 Mini**<br>`aion-labs/aion-3.5-mini` | nous_portal | $0.4580 | **50.4** | 36.6 | 130 tps | 262k | 🚫 Dominated |
| 70 | **Google: Nano Banana 2.1**<br>`google/gemini-nano-banana-2.1` | nous_portal | $1.3800 | **50.4** | 27.0 | 130 tps | 65k | 🚫 Dominated |
| 71 | **ByteDance Seed: Seed 1.6 Flash**<br>`bytedance-seed/seed-1.6-flash` | nous_portal | $0.0615 | **50.4** | 47.6 | 130 tps | 262k | 🚫 Dominated |
| 72 | **Anthropic: Claude Haiku Latest**<br>`anthropic/claude-haiku-latest` | nous_portal | $0.0860 | **50.4** | 46.6 | 130 tps | 1000k | 🚫 Dominated |
| 73 | **Google: Gemini 3.1 Pro Preview Custom Tools**<br>`google/gemini-3.1-pro-preview-customtools` | nous_portal | $1.9200 | **50.4** | 24.3 | 130 tps | 1048k | 🚫 Dominated |
| 74 | **AionLabs: Aion-3.0-Mini**<br>`aion-labs/aion-3.0-mini` | nous_portal | $0.4580 | **50.4** | 36.6 | 130 tps | 131k | 🚫 Dominated |
| 75 | **Google: Nano Banana Pro (Gemini 3 Pro Image)**<br>`google/gemini-3-pro-image` | nous_portal | $1.9200 | **50.4** | 24.3 | 130 tps | 131k | 🚫 Dominated |
| 76 | **Qwen: Qwen3.8 Flash**<br>`qwen/qwen3.8-flash` | nous_portal | $0.1016 | **50.4** | 46.0 | 130 tps | 1000k | 🚫 Dominated |
| 77 | **DeepSeek: DeepSeek V4 Flash Latest**<br>`deepseek/deepseek-v4-flash-latest` | nous_portal | $0.0684 | **50.4** | 47.3 | 130 tps | 1048k | 🚫 Dominated |
| 78 | **Z.ai: GLM 5.3 FlashX**<br>`z-ai/glm-5.3-flashx` | nous_portal | $0.2900 | **50.4** | 40.2 | 130 tps | 1048k | 🚫 Dominated |
| 79 | **Google: Gemini 3.1 Flash Lite**<br>`google/gemini-3.1-flash-lite` | nous_portal_batch | $0.2400 | **50.4** | 41.5 | 150 tps | 1048k | 🚫 Dominated |
| 80 | **OpenAI: GPT Mini Latest**<br>`openai/gpt-mini-latest` | nous_portal | $0.7200 | **50.4** | 32.7 | 130 tps | 400k | 🚫 Dominated |
| 81 | **OpenAI: GPT Audio Mini**<br>`openai/gpt-audio-mini` | nous_portal | $0.4920 | **50.4** | 36.0 | 130 tps | 128k | 🚫 Dominated |
| 82 | **OpenAI: GPT-4o-mini (2024-07-18)**<br>`openai/gpt-4o-mini-2024-07-18` | nous_portal | $0.1500 | **50.4** | 44.2 | 130 tps | 128k | 🚫 Dominated |
| 83 | **OpenAI: GPT-5.6 Luna Pro**<br>`openai/gpt-5.6-luna-pro` | nous_portal_batch | $0.1920 | **50.4** | 42.9 | 150 tps | 1050k | 🚫 Dominated |
| 84 | **DeepSeek: DeepSeek V4 Flash**<br>`deepseek/deepseek-v4-flash-0731:US` | nous_portal | $0.2247 | **50.4** | 41.9 | 130 tps | 1048k | 🚫 Dominated |
| 85 | **Qwen: Qwen3.5-Flash**<br>`qwen/qwen3.5-flash-02-23` | nous_portal | $0.0533 | **50.4** | 47.9 | 130 tps | 1000k | 🚫 Dominated |
| 86 | **DeepSeek: DeepSeek Flash Latest**<br>`deepseek/deepseek-flash-latest` | openrouter_market | $0.0676 | **50.4** | 47.3 | 130 tps | 1048k | 🚫 Dominated |
| 87 | **Qwen: Qwen3 Coder Flash**<br>`qwen/qwen3-coder-flash` | nous_portal | $0.1794 | **50.4** | 43.3 | 130 tps | 1000k | 🚫 Dominated |
| 88 | **DeepSeek: DeepSeek V4.1 Flash**<br>`deepseek/deepseek-v4.1-flash:US` | nous_portal | $0.2247 | **50.4** | 41.9 | 130 tps | 1048k | 🚫 Dominated |
| 89 | **Google: Gemini Flash Latest**<br>`google/gemini-flash-latest` | nous_portal | $0.6450 | **50.4** | 33.7 | 130 tps | 1048k | 🚫 Dominated |
| 90 | **OpenAI: GPT Luna Latest**<br>`openai/gpt-luna-latest` | nous_portal | $0.0860 | **50.4** | 46.6 | 130 tps | 1050k | 🚫 Dominated |
| 91 | **Z.ai: GLM Flash Latest**<br>`z-ai/glm-flash-latest` | nous_portal | $0.1073 | **50.4** | 45.8 | 130 tps | 1048k | 🚫 Dominated |
| 92 | **ByteDance Seed: Seed-2.0-Mini**<br>`bytedance-seed/seed-2.0-mini` | nous_portal | $0.0820 | **50.4** | 46.7 | 130 tps | 262k | 🚫 Dominated |
| 93 | **Qwen: Qwen3.6 Flash**<br>`qwen/qwen3.6-flash` | nous_portal | $0.1913 | **50.4** | 42.9 | 130 tps | 1000k | 🚫 Dominated |
| 94 | **Google: Gemini Pro Latest**<br>`google/gemini-pro-latest` | nous_portal | $1.9200 | **50.4** | 24.3 | 130 tps | 1048k | 🚫 Dominated |
| 95 | **Z.ai: GLM 5.3 Flash**<br>`z-ai/glm-5.3-flash:US` | nous_portal | $0.1865 | **50.4** | 43.1 | 130 tps | 1048k | 🚫 Dominated |
| 96 | **Qwen: Qwen3.8 Omni Flash**<br>`qwen/qwen3.8-omni-flash` | nous_portal | $0.1016 | **50.4** | 46.0 | 130 tps | 1000k | 🚫 Dominated |
| 97 | **Qwen: Qwen3.7 Flash**<br>`qwen/qwen3.7-flash` | nous_portal | $0.0256 | **50.4** | 49.2 | 130 tps | 1000k | 💎 **Best Value** |
| 98 | **OpenAI: GPT-6 Luna Pro**<br>`openai/gpt-6-luna-pro` | nous_portal_batch | $0.0860 | **50.4** | 46.6 | 150 tps | 1050k | 🚫 Dominated |
| 99 | **Google: Gemini 2.5 Pro Preview 06-05**<br>`google/gemini-2.5-pro-preview` | nous_portal | $1.4500 | **50.4** | 26.6 | 130 tps | 1048k | 🚫 Dominated |
| 100 | **OpenAI: o4 Mini High**<br>`openai/o4-mini-high` | nous_portal | $0.9350 | **50.4** | 30.4 | 130 tps | 200k | 🚫 Dominated |
| 101 | **DeepSeek: DeepSeek V4 Pro 0813**<br>`deepseek/deepseek-v4-pro-0813` | nous_portal | $0.9620 | **50.1** | 29.9 | 75 tps | 1048k | 🚫 Dominated |
| 102 | **Claude Opus 4.8**<br>`anthropic/claude-opus-4.8` | nous_portal_batch | $4.3000 | **49.6** | 18.6 | 150 tps | 1000k | 🚫 Dominated |
| 103 | **OpenAI: GPT-5.5**<br>`openai/gpt-5.5` | nous_portal_batch | $4.8000 | **49.0** | 17.8 | 150 tps | 1050k | 🚫 Dominated |
| 104 | **OpenAI: GPT-6 Sol**<br>`openai/gpt-6-sol` | nous_portal_batch | $1.7200 | **48.6** | 24.3 | 150 tps | 1050k | 🚫 Dominated |
| 105 | **Z.ai: GLM 5.2**<br>`z-ai/glm-5.2` | openrouter_market | $0.6490 | **48.3** | 32.2 | 70 tps | 1048k | 🚫 Dominated |
| 106 | **Anthropic: Claude Haiku 5.5**<br>`anthropic/claude-haiku-5.5` | nous_portal_batch | $0.0860 | **47.6** | 44.0 | 150 tps | 1000k | 🚫 Dominated |
| 107 | **StepFun: Step 5 Preview**<br>`stepfun/step-5-preview` | nous_portal | $0.6000 | **47.4** | 32.2 | 75 tps | 1000k | 🚫 Dominated |
| 108 | **Sakana: Fugu Max**<br>`sakana/fugu-max` | nous_portal | $1.3500 | **47.4** | 25.6 | 75 tps | 1000k | 🚫 Dominated |
| 109 | **DeepSeek: DeepSeek Pro Latest**<br>`deepseek/deepseek-pro-latest` | nous_portal | $0.9620 | **47.4** | 28.3 | 75 tps | 1048k | 🚫 Dominated |
| 110 | **OpenAI: GPT Terra Latest**<br>`openai/gpt-terra-latest` | nous_portal | $1.9200 | **47.4** | 22.9 | 75 tps | 1050k | 🚫 Dominated |
| 111 | **Z.ai: GLM Latest**<br>`z-ai/glm-latest` | nous_portal | $0.4941 | **47.4** | 33.8 | 75 tps | 1048k | 🚫 Dominated |
| 112 | **Qwen: Qwen3.5 Plus 2026-02-15**<br>`qwen/qwen3.5-plus-02-15` | nous_portal | $0.2652 | **47.4** | 38.4 | 75 tps | 1000k | 🚫 Dominated |
| 113 | **MoonshotAI: Kimi K3**<br>`moonshotai/kimi-k3:US` | nous_portal | $3.1218 | **47.4** | 19.6 | 75 tps | 1048k | 🚫 Dominated |
| 114 | **Poolside: Laguna S 2.1**<br>`poolside/laguna-s-2.1` | openrouter_market | $0.0504 | **47.4** | 45.2 | 70 tps | 1048k | 🚫 Dominated |
| 115 | **Meta: Muse Wonder**<br>`meta/muse-wonder` | nous_portal | $3.3200 | **47.4** | 19.2 | 75 tps | 1048k | 🚫 Dominated |
| 116 | **Anthropic: Claude Fable Latest**<br>`anthropic/claude-fable-latest` | nous_portal | $8.1500 | **47.4** | 14.7 | 75 tps | 1000k | 🚫 Dominated |
| 117 | **Fireworks: Ember-1**<br>`fireworks/ember-1` | nous_portal | $2.5800 | **47.4** | 20.8 | 75 tps | 1048k | 🚫 Dominated |
| 118 | **Qwen: Qwen-Plus**<br>`qwen/qwen-plus` | nous_portal | $0.1872 | **47.4** | 40.5 | 75 tps | 1000k | 🚫 Dominated |
| 119 | **Pareto 26.10 Preview**<br>`unbiased/pareto-26.10-preview` | nous_portal | $0.5780 | **47.4** | 32.5 | 75 tps | 1048k | 🚫 Dominated |
| 120 | **Tencent: Hy4 preview**<br>`tencent/hy4-preview` | nous_portal | $0.2627 | **47.4** | 38.4 | 75 tps | 1048k | 🚫 Dominated |
| 121 | **Sakana: Fugu Ultra v2**<br>`sakana/fugu-ultra-v2` | nous_portal | $4.8000 | **47.4** | 17.2 | 75 tps | 1000k | 🚫 Dominated |
| 122 | **OpenAI: GPT-5.4 Pro**<br>`openai/gpt-5.4-pro` | nous_portal_batch | $30.6000 | **47.4** | 10.6 | 150 tps | 1050k | 🚫 Dominated |
| 123 | **Mistral: Mistral Large 4**<br>`mistralai/mistral-large-4-0` | nous_portal | $0.4550 | **47.4** | 34.5 | 75 tps | 1048k | 🚫 Dominated |
| 124 | **Qwen: Qwen3.5 Plus 2026-04-20**<br>`qwen/qwen3.5-plus-20260420` | nous_portal | $0.3060 | **47.4** | 37.4 | 75 tps | 1000k | 🚫 Dominated |
| 125 | **MoonshotAI: Kimi Latest**<br>`moonshotai/kimi-latest` | nous_portal | $1.9020 | **47.4** | 22.9 | 75 tps | 1048k | 🚫 Dominated |
| 126 | **xAI: Grok Latest**<br>`x-ai/grok-latest` | nous_portal | $1.5000 | **47.4** | 24.7 | 75 tps | 500k | 🚫 Dominated |
| 127 | **OpenAI: GPT-5.6 Terra Pro**<br>`openai/gpt-5.6-terra-pro` | nous_portal_batch | $1.9200 | **47.4** | 22.9 | 150 tps | 1050k | 🚫 Dominated |
| 128 | **Sakana: Fugu Ultra**<br>`sakana/fugu-ultra` | nous_portal | $4.8000 | **47.4** | 17.2 | 75 tps | 1000k | 🚫 Dominated |
| 129 | **OpenAI: GPT-5.5 Pro**<br>`openai/gpt-5.5-pro` | nous_portal_batch | $30.6000 | **47.4** | 10.6 | 150 tps | 1050k | 🚫 Dominated |
| 130 | **Xiaomi: MiMo-V2.6-Pro-UltraSpeed**<br>`xiaomi/mimo-v2.6-pro-ultraspeed` | nous_portal | $2.1966 | **47.4** | 21.9 | 75 tps | 1048k | 🚫 Dominated |
| 131 | **Z.ai: GLM 5.3**<br>`z-ai/glm-5.3:US` | nous_portal | $1.6764 | **47.4** | 23.9 | 75 tps | 1048k | 🚫 Dominated |
| 132 | **Claude Opus 4.7**<br>`anthropic/claude-opus-4.7` | nous_portal_batch | $4.3000 | **47.4** | 17.8 | 150 tps | 1000k | 🚫 Dominated |
| 133 | **Google: Gemini 3.6 Flash**<br>`google/gemini-3.6-flash` | nous_portal_batch | $0.6450 | **46.7** | 31.2 | 150 tps | 1048k | 🚫 Dominated |
| 134 | **xAI: Grok 4.7**<br>`x-ai/grok-4.7` | nous_portal | $1.5000 | **46.4** | 24.2 | 75 tps | 500k | 🚫 Dominated |
| 135 | **inclusionAI: Ling 3.1 Flash**<br>`inclusionai/ling-3.1-flash` | nous_portal | $0.0540 | **45.6** | 43.3 | 130 tps | 262k | 🚫 Dominated |
| 136 | **DeepSeek: DeepSeek V4.1 Flash**<br>`deepseek/deepseek-v4.1-flash` | openrouter_market | $0.0855 | **44.1** | 40.8 | 130 tps | 1048k | 🚫 Dominated |
| 137 | **MiniMax: MiniMax M3**<br>`minimax/minimax-m3` | nous_portal | $0.1950 | **43.9** | 37.2 | 130 tps | 1048k | 🚫 Dominated |
| 138 | **Google: Gemini Embedding 2 Preview**<br>`google/gemini-embedding-2-preview` | nous_portal | $0.0840 | **43.7** | 40.4 | 130 tps | 8k | 🚫 Dominated |
| 139 | **Google: Gemini Embedding 001**<br>`google/gemini-embedding-001` | nous_portal | $0.0630 | **43.7** | 41.2 | 130 tps | 20k | 🚫 Dominated |
| 140 | **Google: Gemini Embedding 2**<br>`google/gemini-embedding-2` | nous_portal_batch | $0.0840 | **43.7** | 40.4 | 150 tps | 8k | 🚫 Dominated |
| 141 | **Sentence Transformers: all-MiniLM-L12-v2**<br>`sentence-transformers/all-minilm-l12-v2` | nous_portal | $0.0021 | **43.7** | 43.6 | 130 tps | 0k | 💎 **Best Value** |
| 142 | **Sentence Transformers: paraphrase-MiniLM-L6-v2**<br>`sentence-transformers/paraphrase-minilm-l6-v2` | nous_portal | $0.0021 | **43.7** | 43.6 | 130 tps | 0k | 💎 **Best Value** |
| 143 | **Sentence Transformers: all-MiniLM-L6-v2**<br>`sentence-transformers/all-minilm-l6-v2` | nous_portal | $0.0021 | **43.7** | 43.6 | 130 tps | 0k | 💎 **Best Value** |
| 144 | **DeepSeek V3**<br>`deepseek-v3` | nous_portal | $0.4100 | **42.9** | 31.9 | 92 tps | 64k | 🚫 Dominated |
| 145 | **GPT-4o**<br>`gpt-4o` | nous_portal | $0.4100 | **42.9** | 31.9 | 75 tps | 128k | 🚫 Dominated |
| 146 | **GPT-4o**<br>`gpt-4o` | nous_portal | $0.4100 | **42.9** | 31.9 | 75 tps | 128k | 🚫 Dominated |
| 147 | **DeepSeek V3**<br>`deepseek-v3` | nous_portal | $0.4100 | **42.9** | 31.9 | 92 tps | 64k | 🚫 Dominated |
| 148 | **ByteDance Seed: Seed-2.0-Lite**<br>`bytedance-seed/seed-2.0-lite` | nous_portal | $0.3050 | **42.9** | 33.9 | 75 tps | 262k | 🚫 Dominated |
| 149 | **OpenAI: gpt-oss-safeguard-20b**<br>`openai/gpt-oss-safeguard-20b` | nous_portal | $0.0750 | **42.9** | 40.0 | 75 tps | 131k | 🚫 Dominated |
| 150 | **Pareto Latest**<br>`unbiased/pareto-latest` | nous_portal | $1.6500 | **42.9** | 21.7 | 75 tps | 262k | 🚫 Dominated |
| 151 | **Qwen: Qwen2.5 7B Instruct**<br>`qwen/qwen-2.5-7b-instruct` | nous_portal | $0.0620 | **42.9** | 40.5 | 75 tps | 32k | 🚫 Dominated |
| 152 | **Nex AGI: Nex-N2.5-Pro**<br>`nex-agi/nex-n2.5-pro` | nous_portal | $0.0565 | **42.9** | 40.7 | 75 tps | 262k | 🚫 Dominated |
| 153 | **Poolside: Laguna XS 2.1**<br>`poolside/laguna-xs-2.1` | openrouter_market | $0.0480 | **42.9** | 41.0 | 70 tps | 262k | 🚫 Dominated |
| 154 | **ByteDance Seed: Seed 1.6**<br>`bytedance-seed/seed-1.6` | nous_portal | $0.3050 | **42.9** | 33.9 | 75 tps | 262k | 🚫 Dominated |
| 155 | **Mistral: Mistral Small 3.2 24B**<br>`mistralai/mistral-small-3.2-24b-instruct` | nous_portal | $0.0515 | **42.9** | 40.8 | 75 tps | 256k | 🚫 Dominated |
| 156 | **OpenAI: GPT-3.5 Turbo 16k**<br>`openai/gpt-3.5-turbo-16k` | nous_portal | $1.6600 | **42.9** | 21.7 | 75 tps | 16k | 🚫 Dominated |
| 157 | **Cohere: Command A+**<br>`cohere/command-a-plus` | nous_portal | $0.3300 | **42.9** | 33.4 | 75 tps | 192k | 🚫 Dominated |
| 158 | **OpenAI: GPT-5 Pro**<br>`openai/gpt-5-pro` | nous_portal_batch | $18.3000 | **42.9** | 10.8 | 150 tps | 400k | 🚫 Dominated |
| 159 | **Pareto 26.9**<br>`unbiased/pareto-26.9` | nous_portal | $1.6500 | **42.9** | 21.7 | 75 tps | 262k | 🚫 Dominated |
| 160 | **AionLabs: Aion-2.0**<br>`aion-labs/aion-2.0` | nous_portal | $0.5200 | **42.9** | 30.2 | 75 tps | 131k | 🚫 Dominated |
| 161 | **ByteDance Seed: Seed 2.1 Turbo**<br>`bytedance-seed/seed-2-1-turbo` | nous_portal | $0.4600 | **42.9** | 31.1 | 75 tps | 262k | 🚫 Dominated |
| 162 | **OpenAI: GPT-5.2 Chat**<br>`openai/gpt-5.2-chat` | nous_portal | $2.0300 | **42.9** | 20.3 | 75 tps | 128k | 🚫 Dominated |
| 163 | **Mistral: Voxtral Small 24B 2507**<br>`mistralai/voxtral-small-24b-2507` | nous_portal | $0.0660 | **42.9** | 40.3 | 75 tps | 32k | 🚫 Dominated |
| 164 | **MissingNo**<br>`stealth/missingno` | nous_portal | $0.0010 | **42.9** | 42.9 | 75 tps | 262k | 💎 **Best Value** |
| 165 | **Cohere: Command R+ (08-2024)**<br>`cohere/command-r-plus-08-2024` | nous_portal | $2.0500 | **42.9** | 20.3 | 75 tps | 128k | 🚫 Dominated |
| 166 | **OpenAI: GPT-5.2 Pro**<br>`openai/gpt-5.2-pro` | nous_portal_batch | $25.6200 | **42.9** | 10.0 | 150 tps | 400k | 🚫 Dominated |
| 167 | **PrismML: Ternary Bonsai 2 27B**<br>`prism-ml/ternary-bonsai-2-27b` | nous_portal | $0.0950 | **42.9** | 39.3 | 75 tps | 262k | 🚫 Dominated |
| 168 | **ByteDance Seed: Seed-2.0-Code**<br>`bytedance-seed/seed-2.0-code` | nous_portal | $0.5100 | **42.9** | 30.4 | 75 tps | 262k | 🚫 Dominated |
| 169 | **DeepSeek: DeepSeek V3**<br>`deepseek/deepseek-chat` | nous_portal | $0.2110 | **42.9** | 36.0 | 75 tps | 163k | 🚫 Dominated |
| 170 | **OpenAI: GPT-5.1-Codex-Max**<br>`openai/gpt-5.1-codex-max` | nous_portal | $1.4500 | **42.9** | 22.6 | 75 tps | 400k | 🚫 Dominated |
| 171 | **OpenAI: GPT Chat Latest**<br>`openai/gpt-chat-latest` | nous_portal | $4.8000 | **42.9** | 15.6 | 75 tps | 400k | 🚫 Dominated |
| 172 | **Reka Edge**<br>`rekaai/reka-edge` | nous_portal | $0.0520 | **42.9** | 40.8 | 75 tps | 16k | 🚫 Dominated |
| 173 | **Mistral: Mistral Nemo**<br>`mistralai/mistral-nemo` | nous_portal | $0.0136 | **42.9** | 42.3 | 75 tps | 131k | 🚫 Dominated |
| 174 | **AionLabs: Aion 3.5**<br>`aion-labs/aion-3.5` | nous_portal | $1.9500 | **42.9** | 20.6 | 75 tps | 262k | 🚫 Dominated |
| 175 | **Mistral: Codestral 2508**<br>`mistralai/codestral-2508` | nous_portal_batch | $0.1980 | **42.9** | 36.3 | 150 tps | 256k | 🚫 Dominated |
| 176 | **Perceptron: Perceptron Mk1.5**<br>`perceptron/perceptron-mk1.5` | nous_portal | $0.2130 | **42.9** | 36.0 | 75 tps | 36k | 🚫 Dominated |
| 177 | **OpenAI: GPT-3.5 Turbo (older v0613)**<br>`openai/gpt-3.5-turbo-0613` | nous_portal | $0.6200 | **42.9** | 28.9 | 75 tps | 4k | 🚫 Dominated |
| 178 | **LiquidAI: LFM2.5-24B-A2B**<br>`liquid/lfm-2.5-24b-a2b` | nous_portal | $0.1000 | **42.9** | 39.2 | 75 tps | 128k | 🚫 Dominated |
| 179 | **Mistral: Mistral Small 3.1 24B**<br>`mistralai/mistral-small-3.1-24b-instruct` | nous_portal | $0.2029 | **42.9** | 36.2 | 75 tps | 128k | 🚫 Dominated |
| 180 | **Relace: Relace Search**<br>`relace/relace-search` | nous_portal | $0.7200 | **42.9** | 27.8 | 75 tps | 256k | 🚫 Dominated |
| 181 | **AionLabs: Aion-3.0**<br>`aion-labs/aion-3.0` | nous_portal | $1.9500 | **42.9** | 20.6 | 75 tps | 131k | 🚫 Dominated |
| 182 | **OpenAI: GPT-4o**<br>`openai/gpt-4o` | nous_portal_batch | $2.0500 | **42.9** | 20.3 | 150 tps | 128k | 🚫 Dominated |
| 183 | **OpenAI: GPT Audio**<br>`openai/gpt-audio` | nous_portal | $2.0500 | **42.9** | 20.3 | 75 tps | 128k | 🚫 Dominated |
| 184 | **Cohere: Command R (08-2024)**<br>`cohere/command-r-08-2024` | nous_portal | $0.1230 | **42.9** | 38.4 | 75 tps | 128k | 🚫 Dominated |
| 185 | **inclusionAI: Ling 3.0 Flash VL**<br>`inclusionai/ling-3.0-flash-vl` | nous_portal | $0.0432 | **42.9** | 41.2 | 130 tps | 131k | 🚫 Dominated |
| 186 | **OpenAI: GPT-6 Luna**<br>`openai/gpt-6-luna` | nous_portal_batch | $0.0860 | **42.9** | 39.6 | 150 tps | 1050k | 🚫 Dominated |
| 187 | **Claude Sonnet 4.6**<br>`anthropic/claude-sonnet-4.6` | nous_portal_batch | $2.5800 | **42.6** | 18.7 | 150 tps | 1000k | 🚫 Dominated |
| 188 | **Meta: Muse Spark 1.1**<br>`meta/muse-spark-1.1` | nous_portal | $0.8900 | **41.5** | 25.4 | 75 tps | 1048k | 🚫 Dominated |
| 189 | **DeepSeek: DeepSeek V4 Flash 0423**<br>`deepseek/deepseek-v4-flash` | nous_portal | $0.1347 | **40.0** | 35.5 | 130 tps | 1048k | 🚫 Dominated |
| 190 | **Google: Gemini 3.5 Flash**<br>`google/gemini-3.5-flash` | nous_portal_batch | $1.4400 | **38.8** | 20.5 | 150 tps | 1048k | 🚫 Dominated |
| 191 | **Upstage: Solar Pro 4**<br>`upstage/solar-pro4` | nous_portal | $0.0738 | **38.4** | 35.9 | 90 tps | 524k | 🚫 Dominated |
| 192 | **DeepSeek: DeepSeek V4 Pro 0423**<br>`deepseek/deepseek-v4-pro` | nous_portal | $0.5190 | **38.2** | 27.0 | 75 tps | 1048k | 🚫 Dominated |
| 193 | **Qwen: Qwen3.7 Max**<br>`qwen/qwen3.7-max` | nous_portal | $1.0620 | **37.9** | 22.0 | 75 tps | 1000k | 🚫 Dominated |
| 194 | **Tencent: Hy3 preview**<br>`tencent/hy3-preview` | nous_portal | $0.1500 | **36.7** | 32.2 | 75 tps | 262k | 🚫 Dominated |
| 195 | **SpaceXAI: Grok Build 0.1**<br>`x-ai/grok-build-0.1` | nous_portal | $0.6200 | **36.4** | 24.6 | 75 tps | 256k | 🚫 Dominated |
| 196 | **Qwen: Qwen3 Embedding 8B**<br>`qwen/qwen3-embedding-8b` | nous_portal | $0.0042 | **36.1** | 36.0 | 75 tps | 32k | 🚫 Dominated |
| 197 | **Intfloat: E5-Base-v2**<br>`intfloat/e5-base-v2` | nous_portal | $0.0021 | **36.1** | 36.1 | 75 tps | 0k | 🚫 Dominated |
| 198 | **OpenAI: Text Embedding Ada 002**<br>`openai/text-embedding-ada-002` | nous_portal_batch | $0.0420 | **36.1** | 34.7 | 150 tps | 8k | 🚫 Dominated |
| 199 | **Sentence Transformers: multi-qa-mpnet-base-dot-v1**<br>`sentence-transformers/multi-qa-mpnet-base-dot-v1` | nous_portal | $0.0021 | **36.1** | 36.1 | 75 tps | 0k | 🚫 Dominated |
| 200 | **BAAI: bge-base-en-v1.5**<br>`baai/bge-base-en-v1.5` | nous_portal | $0.0021 | **36.1** | 36.1 | 75 tps | 0k | 🚫 Dominated |
| 201 | **Intfloat: E5-Large-v2**<br>`intfloat/e5-large-v2` | nous_portal | $0.0042 | **36.1** | 36.0 | 75 tps | 0k | 🚫 Dominated |
| 202 | **VoyageAI by MongoDB: voyage-multimodal-3.5**<br>`voyageai/voyage-multimodal-3.5` | nous_portal | $0.0504 | **36.1** | 34.4 | 75 tps | 32k | 🚫 Dominated |
| 203 | **Sentence Transformers: all-mpnet-base-v2**<br>`sentence-transformers/all-mpnet-base-v2` | nous_portal | $0.0021 | **36.1** | 36.1 | 75 tps | 0k | 🚫 Dominated |
| 204 | **Perplexity: Embed V1 4B**<br>`perplexity/pplx-embed-v1-4b` | nous_portal | $0.0126 | **36.1** | 35.7 | 75 tps | 32k | 🚫 Dominated |
| 205 | **OpenAI: Text Embedding 3 Large**<br>`openai/text-embedding-3-large` | nous_portal_batch | $0.0546 | **36.1** | 34.3 | 150 tps | 8k | 🚫 Dominated |
| 206 | **Perplexity: Embed V1 0.6B**<br>`perplexity/pplx-embed-v1-0.6b` | nous_portal | $0.0017 | **36.1** | 36.1 | 75 tps | 32k | 🚫 Dominated |
| 207 | **Intfloat: Multilingual-E5-Large**<br>`intfloat/multilingual-e5-large` | nous_portal | $0.0042 | **36.1** | 36.0 | 75 tps | 0k | 🚫 Dominated |
| 208 | **BAAI: bge-large-en-v1.5**<br>`baai/bge-large-en-v1.5` | nous_portal | $0.0042 | **36.1** | 36.0 | 75 tps | 0k | 🚫 Dominated |
| 209 | **VoyageAI by MongoDB: voyage-4-lite**<br>`voyageai/voyage-4-lite` | nous_portal | $0.0084 | **36.1** | 35.8 | 75 tps | 32k | 🚫 Dominated |
| 210 | **VoyageAI by MongoDB: voyage-code-4**<br>`voyageai/voyage-code-4` | nous_portal | $0.0504 | **36.1** | 34.4 | 75 tps | 32k | 🚫 Dominated |
| 211 | **BAAI: bge-m3**<br>`baai/bge-m3` | nous_portal | $0.0042 | **36.1** | 36.0 | 75 tps | 8k | 🚫 Dominated |
| 212 | **OpenAI: Text Embedding 3 Small**<br>`openai/text-embedding-3-small` | nous_portal | $0.0084 | **36.1** | 35.8 | 75 tps | 8k | 🚫 Dominated |
| 213 | **Thenlper: GTE-Large**<br>`thenlper/gte-large` | nous_portal | $0.0042 | **36.1** | 36.0 | 75 tps | 0k | 🚫 Dominated |
| 214 | **VoyageAI by MongoDB: voyage-4-large**<br>`voyageai/voyage-4-large` | nous_portal | $0.0504 | **36.1** | 34.4 | 75 tps | 32k | 🚫 Dominated |
| 215 | **Mistral: Codestral Embed 2505**<br>`mistralai/codestral-embed-2505` | nous_portal | $0.0630 | **36.1** | 34.1 | 75 tps | 8k | 🚫 Dominated |
| 216 | **VoyageAI by MongoDB: voyage-4**<br>`voyageai/voyage-4` | nous_portal | $0.0252 | **36.1** | 35.3 | 75 tps | 32k | 🚫 Dominated |
| 217 | **Qwen: Qwen3 Embedding 4B**<br>`qwen/qwen3-embedding-4b` | nous_portal | $0.0084 | **36.1** | 35.8 | 75 tps | 32k | 🚫 Dominated |
| 218 | **Thenlper: GTE-Base**<br>`thenlper/gte-base` | nous_portal | $0.0021 | **36.1** | 36.1 | 75 tps | 0k | 🚫 Dominated |
| 219 | **Mistral: Mistral Embed 2312**<br>`mistralai/mistral-embed-2312` | nous_portal | $0.0420 | **36.1** | 34.7 | 75 tps | 8k | 🚫 Dominated |
| 220 | **OpenAI: GPT-5.4 Mini**<br>`openai/gpt-5.4-mini` | nous_portal_batch | $0.7200 | **36.1** | 23.4 | 150 tps | 400k | 🚫 Dominated |
| 221 | **Z.ai: GLM 5.1**<br>`z-ai/glm-5.1` | nous_portal | $0.7010 | **35.7** | 23.3 | 75 tps | 204k | 🚫 Dominated |
| 222 | **MoonshotAI: Kimi K2.7 Code**<br>`moonshotai/kimi-k2.7-code` | nous_portal | $0.6444 | **35.5** | 23.7 | 75 tps | 262k | 🚫 Dominated |
| 223 | **MoonshotAI: Kimi K2.6**<br>`moonshotai/kimi-k2.6` | nous_portal | $0.4367 | **35.5** | 26.0 | 75 tps | 262k | 🚫 Dominated |
| 224 | **Xiaomi: MiMo-V2.5-Pro**<br>`xiaomi/mimo-v2.5-pro` | nous_portal | $0.1539 | **35.5** | 31.0 | 75 tps | 1050k | 🚫 Dominated |
| 225 | **inclusionAI: Ling 3.0 Flash**<br>`inclusionai/ling-3.0-flash` | nous_portal | $0.0151 | **35.3** | 34.8 | 130 tps | 262k | 🚫 Dominated |
| 226 | **Thinking Machines: Inkling Small**<br>`thinkingmachines/inkling-small` | openrouter_market | $0.3150 | **34.6** | 27.2 | 70 tps | 524k | 🚫 Dominated |
| 227 | **OpenAI: GPT-5.1**<br>`openai/gpt-5.1` | nous_portal_batch | $1.4530 | **34.3** | 18.1 | 150 tps | 400k | 🚫 Dominated |
| 228 | **Google: Gemini 3.1 Pro Preview**<br>`google/gemini-3.1-pro-preview` | nous_portal_batch | $1.9200 | **34.1** | 16.5 | 150 tps | 1048k | 🚫 Dominated |
| 229 | **OpenAI: GPT-5.3-Codex**<br>`openai/gpt-5.3-codex` | nous_portal | $2.0300 | **33.9** | 16.1 | 75 tps | 400k | 🚫 Dominated |
| 230 | **Thinking Machines: Inkling**<br>`thinkingmachines/inkling` | openrouter_market | $0.7860 | **33.8** | 21.4 | 70 tps | 524k | 🚫 Dominated |
| 231 | **MiniMax: MiniMax M2.7**<br>`minimax/minimax-m2.7` | nous_portal | $0.1722 | **33.5** | 28.9 | 130 tps | 204k | 🚫 Dominated |
| 232 | **MoonshotAI: Kimi K2.5**<br>`moonshotai/kimi-k2.5` | nous_portal | $0.5170 | **32.8** | 23.1 | 75 tps | 262k | 🚫 Dominated |
| 233 | **StepFun: Step 3.7 Flash**<br>`stepfun/step-3.7-flash` | nous_portal | $0.1990 | **32.2** | 27.2 | 130 tps | 262k | 🚫 Dominated |
| 234 | **OpenAI: GPT-5.2**<br>`openai/gpt-5.2` | nous_portal_batch | $2.0300 | **32.0** | 15.2 | 150 tps | 400k | 🚫 Dominated |
| 235 | **Google: Gemini 3.5 Flash Lite**<br>`google/gemini-3.5-flash-lite` | nous_portal_batch | $0.3580 | **31.9** | 24.5 | 150 tps | 1048k | 🚫 Dominated |
| 236 | **Qwen: Qwen3.7 Plus**<br>`qwen/qwen3.7-plus` | nous_portal | $0.2624 | **31.9** | 25.9 | 75 tps | 1000k | 🚫 Dominated |
| 237 | **Qwen: Qwen3.6 27B**<br>`qwen/qwen3.6-27b` | nous_portal | $0.2900 | **31.9** | 25.4 | 75 tps | 262k | 🚫 Dominated |
| 238 | **NVIDIA: Nemotron 3 Ultra**<br>`nvidia/nemotron-3-ultra-550b-a55b` | openrouter_market | $0.4300 | **31.5** | 23.2 | 70 tps | 262k | 🚫 Dominated |
| 239 | **Xiaomi: MiMo-V2.5**<br>`xiaomi/mimo-v2.5` | nous_portal | $0.0611 | **31.2** | 29.4 | 75 tps | 1050k | 🚫 Dominated |
| 240 | **OpenAI: GPT-5.4 Nano**<br>`openai/gpt-5.4-nano` | nous_portal_batch | $0.1970 | **31.1** | 26.3 | 150 tps | 400k | 🚫 Dominated |
| 241 | **Claude Opus 4.6**<br>`anthropic/claude-opus-4.6` | nous_portal_batch | $4.3000 | **30.9** | 11.6 | 150 tps | 1000k | 🚫 Dominated |
| 242 | **DeepSeek: DeepSeek V3.2**<br>`deepseek/deepseek-v3.2` | nous_portal | $0.1066 | **30.8** | 28.0 | 75 tps | 163k | 🚫 Dominated |
| 243 | **OpenAI: GPT-5.2-Codex**<br>`openai/gpt-5.2-codex` | nous_portal | $2.0300 | **30.3** | 14.4 | 75 tps | 400k | 🚫 Dominated |
| 244 | **Upstage: Solar Mini 4**<br>`upstage/solar-mini4` | nous_portal | $0.0380 | **30.3** | 29.2 | 130 tps | 524k | 🚫 Dominated |
| 245 | **OpenAI: GPT-5**<br>`openai/gpt-5` | nous_portal_batch | $1.4500 | **29.8** | 15.7 | 150 tps | 400k | 🚫 Dominated |
| 246 | **Anthropic: Claude Sonnet 4.5**<br>`anthropic/claude-sonnet-4.5` | nous_portal_batch | $2.5800 | **29.8** | 13.1 | 150 tps | 1000k | 🚫 Dominated |
| 247 | **MiniMax: MiniMax M2.5**<br>`minimax/minimax-m2.5` | nous_portal | $0.1940 | **29.1** | 24.7 | 130 tps | 204k | 🚫 Dominated |
| 248 | **Z.ai: GLM 5 Turbo**<br>`z-ai/glm-5-turbo` | nous_portal | $0.9040 | **28.6** | 17.4 | 75 tps | 202k | 🚫 Dominated |
| 249 | **Anthropic: Claude Opus 4.5**<br>`anthropic/claude-opus-4.5` | nous_portal_batch | $4.3000 | **28.3** | 10.6 | 150 tps | 200k | 🚫 Dominated |
| 250 | **Qwen: Qwen3.5-122B-A10B**<br>`qwen/qwen3.5-122b-a10b` | nous_portal | $0.3172 | **28.3** | 22.2 | 75 tps | 262k | 🚫 Dominated |
| 251 | **SpaceXAI: Grok 4.20**<br>`x-ai/grok-4.20` | nous_portal | $0.7450 | **27.8** | 17.8 | 75 tps | 2000k | 🚫 Dominated |
| 252 | **MiniMax: MiniMax M2.1**<br>`minimax/minimax-m2.1` | nous_portal | $0.2280 | **27.4** | 22.7 | 130 tps | 204k | 🚫 Dominated |
| 253 | **Anthropic: Claude Sonnet 4**<br>`anthropic/claude-sonnet-4` | nous_portal | $2.5800 | **27.3** | 12.0 | 75 tps | 200k | 🚫 Dominated |
| 254 | **OpenAI: GPT-5 Mini**<br>`openai/gpt-5-mini` | nous_portal_batch | $0.2930 | **27.1** | 21.6 | 150 tps | 400k | 🚫 Dominated |
| 255 | **OpenAI: GPT-5.1-Codex-Mini**<br>`openai/gpt-5.1-codex-mini` | nous_portal | $0.2930 | **26.9** | 21.4 | 130 tps | 400k | 🚫 Dominated |
| 256 | **Meituan: LongCat 2.0**<br>`meituan/longcat-2.0` | nous_portal | $0.2136 | **26.6** | 22.3 | 75 tps | 1048k | 🚫 Dominated |
| 257 | **SpaceXAI: Grok 4.3**<br>`x-ai/grok-4.3` | nous_portal_batch | $0.7450 | **26.6** | 17.1 | 150 tps | 1000k | 🚫 Dominated |
| 258 | **Claude Haiku 4.5**<br>`anthropic/claude-haiku-4.5` | nous_portal_batch | $0.8600 | **26.5** | 16.4 | 150 tps | 409k | 🚫 Dominated |
| 259 | **OpenAI: GPT-5.1-Codex**<br>`openai/gpt-5.1-codex` | nous_portal | $1.4530 | **26.0** | 13.7 | 75 tps | 400k | 🚫 Dominated |
| 260 | **OpenAI: o1**<br>`openai/o1` | nous_portal | $15.0000 | **25.7** | 6.8 | 75 tps | 200k | 🚫 Dominated |
| 261 | **MiniMax: MiniMax M2**<br>`minimax/minimax-m2` | nous_portal | $0.2091 | **25.3** | 21.3 | 130 tps | 204k | 🚫 Dominated |
| 262 | **Qwen: Qwen3.5-27B**<br>`qwen/qwen3.5-27b` | nous_portal | $0.2379 | **25.3** | 20.8 | 75 tps | 262k | 🚫 Dominated |
| 263 | **Qwen: Qwen3.6 35B A3B**<br>`qwen/qwen3.6-35b-a3b` | nous_portal | $0.1000 | **25.1** | 22.9 | 75 tps | 262k | 🚫 Dominated |
| 264 | **Meta: Muse Glimmer 30B**<br>`meta/muse-glimmer-30b` | nous_portal | $0.2340 | **24.4** | 20.2 | 75 tps | 131k | 🚫 Dominated |
| 265 | **MoonshotAI: Kimi K2 Thinking**<br>`moonshotai/kimi-k2-thinking` | nous_portal | $0.5020 | **24.4** | 17.4 | 75 tps | 262k | 🚫 Dominated |
| 266 | **OpenAI: o3 Pro**<br>`openai/o3-pro` | nous_portal | $16.4000 | **24.4** | 6.3 | 75 tps | 200k | 🚫 Dominated |
| 267 | **StepFun: Step 3.5 Flash**<br>`stepfun/step-3.5-flash` | nous_portal | $0.0720 | **23.9** | 22.3 | 130 tps | 262k | 🚫 Dominated |
| 268 | **OpenAI: o4 Mini**<br>`openai/o4-mini` | nous_portal_batch | $0.9350 | **23.6** | 14.2 | 150 tps | 200k | 🚫 Dominated |
| 269 | **Mistral: Mistral Medium 3.5**<br>`mistralai/mistral-medium-3-5` | nous_portal_batch | $1.3800 | **22.9** | 12.3 | 150 tps | 262k | 🚫 Dominated |
| 270 | **OpenAI: o3**<br>`openai/o3` | nous_portal_batch | $1.7000 | **22.8** | 11.5 | 150 tps | 200k | 🚫 Dominated |
| 271 | **Anthropic: Claude Opus 4.1**<br>`anthropic/claude-opus-4.1` | nous_portal_batch | $12.9000 | **22.7** | 6.2 | 150 tps | 200k | 🚫 Dominated |
| 272 | **Google: Gemini 2.5 Flash**<br>`google/gemini-2.5-flash` | nous_portal_batch | $0.3580 | **22.5** | 17.2 | 150 tps | 1048k | 🚫 Dominated |
| 273 | **Qwen: Qwen3.5-35B-A3B**<br>`qwen/qwen3.5-35b-a3b` | nous_portal | $0.1230 | **22.0** | 19.7 | 75 tps | 262k | 🚫 Dominated |
| 274 | **Z.ai: GLM 4.7 Flash**<br>`z-ai/glm-4.7-flash` | nous_portal | $0.0640 | **22.0** | 20.7 | 130 tps | 200k | 🚫 Dominated |
| 275 | **DeepSeek: DeepSeek V3.1 Terminus**<br>`deepseek/deepseek-v3.1-terminus` | nous_portal | $0.2134 | **21.8** | 18.2 | 75 tps | 163k | 🚫 Dominated |
| 276 | **OpenAI: GPT-4.1 Mini**<br>`openai/gpt-4.1-mini` | nous_portal_batch | $0.3400 | **20.8** | 16.1 | 150 tps | 1047k | 🚫 Dominated |
| 277 | **Google: Gemma 4 31B**<br>`google/gemma-4-31b-it` | openrouter_market | $0.0910 | **20.2** | 18.6 | 70 tps | 262k | 🚫 Dominated |
| 278 | **Google: Gemini 3.1 Flash Lite Preview**<br>`google/gemini-3.1-flash-lite-preview` | nous_portal | $0.2400 | **19.9** | 16.4 | 130 tps | 1048k | 🚫 Dominated |
| 279 | **OpenAI o3-mini**<br>`o3-mini` | nous_portal | $0.4100 | **19.8** | 14.8 | 130 tps | 200k | 🚫 Dominated |
| 280 | **OpenAI o3-mini**<br>`o3-mini` | nous_portal | $0.4100 | **19.8** | 14.8 | 130 tps | 200k | 🚫 Dominated |
| 281 | **OpenAI: o3 Mini**<br>`openai/o3-mini` | nous_portal_batch | $1.1000 | **19.8** | 11.4 | 150 tps | 200k | 🚫 Dominated |
| 282 | **Qwen: Qwen3.5-9B**<br>`qwen/qwen3.5-9b` | nous_portal | $0.0610 | **19.7** | 18.6 | 75 tps | 262k | 🚫 Dominated |
| 283 | **DeepSeek: DeepSeek V3.2 Exp**<br>`deepseek/deepseek-v3.2-exp` | nous_portal | $0.1544 | **19.6** | 17.1 | 75 tps | 163k | 🚫 Dominated |
| 284 | **Amazon: Nova 2 Lite**<br>`amazon/nova-2-lite-v1` | nous_portal | $0.3760 | **19.6** | 14.8 | 75 tps | 1000k | 🚫 Dominated |
| 285 | **Google: Gemini 2.5 Pro**<br>`google/gemini-2.5-pro` | nous_portal_batch | $1.4500 | **19.5** | 10.3 | 150 tps | 1048k | 🚫 Dominated |
| 286 | **MiniMax: MiniMax M1**<br>`minimax/minimax-m1` | nous_portal | $0.3880 | **19.1** | 14.4 | 130 tps | 1000k | 🚫 Dominated |
| 287 | **MoonshotAI: Kimi K2 0905**<br>`moonshotai/kimi-k2-0905` | nous_portal | $0.5020 | **18.4** | 13.1 | 75 tps | 262k | 🚫 Dominated |
| 288 | **Google: Gemma 4 26B A4B **<br>`google/gemma-4-26b-a4b-it` | openrouter_market | $0.0472 | **18.2** | 17.4 | 70 tps | 262k | 🚫 Dominated |
| 289 | **Google: Gemini 2.5 Flash Lite**<br>`google/gemini-2.5-flash-lite` | nous_portal_batch | $0.0760 | **17.9** | 16.7 | 150 tps | 1048k | 🚫 Dominated |
| 290 | **Qwen: Qwen3 Coder Next**<br>`qwen/qwen3-coder-next` | nous_portal | $0.1580 | **17.1** | 14.9 | 90 tps | 262k | 🚫 Dominated |
| 291 | **NVIDIA: Nemotron 3 Super**<br>`nvidia/nemotron-3-super-120b-a12b` | openrouter_market | $0.0786 | **17.0** | 15.8 | 70 tps | 262k | 🚫 Dominated |
| 292 | **DeepSeek: DeepSeek V3.1**<br>`deepseek/deepseek-chat-v3.1` | nous_portal | $0.2480 | **17.0** | 13.9 | 75 tps | 163k | 🚫 Dominated |
| 293 | **Qwen: Qwen3 Next 80B A3B Thinking**<br>`qwen/qwen3-next-80b-a3b-thinking` | nous_portal | $0.1830 | **16.6** | 14.2 | 75 tps | 262k | 🚫 Dominated |
| 294 | **DeepSeek: R1 0528**<br>`deepseek/deepseek-r1-0528` | nous_portal | $0.5750 | **16.4** | 11.3 | 75 tps | 163k | 🚫 Dominated |
| 295 | **Qwen: Qwen3 Coder 480B A35B**<br>`qwen/qwen3-coder` | nous_portal | $0.2724 | **16.4** | 13.2 | 90 tps | 262k | 🚫 Dominated |
| 296 | **OpenAI: GPT-5 Nano**<br>`openai/gpt-5-nano` | nous_portal_batch | $0.0610 | **16.3** | 15.4 | 150 tps | 400k | 🚫 Dominated |
| 297 | **OpenAI: GPT-4o (2024-05-13)**<br>`openai/gpt-4o-2024-05-13` | nous_portal | $3.6000 | **16.3** | 6.4 | 75 tps | 128k | 🚫 Dominated |
| 298 | **MoonshotAI: Kimi K2 0711**<br>`moonshotai/kimi-k2` | nous_portal | $0.4694 | **16.1** | 11.6 | 75 tps | 131k | 🚫 Dominated |
| 299 | **OpenAI: GPT-4.1**<br>`openai/gpt-4.1` | nous_portal_batch | $1.7000 | **16.1** | 8.1 | 150 tps | 1047k | 🚫 Dominated |
| 300 | **GPT-4o Mini**<br>`gpt-4o-mini` | openrouter | $0.4100 | **16.0** | 11.9 | 120 tps | 128k | 🚫 Dominated |
| 301 | **GPT-4o Mini**<br>`gpt-4o-mini` | openrouter | $0.4100 | **16.0** | 11.9 | 120 tps | 128k | 🚫 Dominated |
| 302 | **OpenAI: GPT-4o-mini**<br>`openai/gpt-4o-mini` | nous_portal_batch | $0.1500 | **16.0** | 14.0 | 150 tps | 128k | 🚫 Dominated |
| 303 | **OpenAI: gpt-oss-120b**<br>`openai/gpt-oss-120b` | nous_portal_batch | $0.0420 | **16.0** | 15.4 | 150 tps | 131k | 🚫 Dominated |
| 304 | **Inception: Mercury 2.5**<br>`inception/mercury-2.5` | nous_portal | $0.0294 | **15.7** | 15.3 | 75 tps | 260k | 🚫 Dominated |
| 305 | **Qwen: Qwen3 235B A22B Instruct 2507**<br>`qwen/qwen3-235b-a22b-2507` | nous_portal | $0.0717 | **15.4** | 14.4 | 75 tps | 262k | 🚫 Dominated |
| 306 | **Mistral: Devstral 2 2512**<br>`mistralai/devstral-2512` | nous_portal | $0.3440 | **15.4** | 11.9 | 75 tps | 262k | 🚫 Dominated |
| 307 | **OpenAI: GPT-4 Turbo**<br>`openai/gpt-4-turbo` | nous_portal_batch | $7.2000 | **15.3** | 4.9 | 150 tps | 128k | 🚫 Dominated |
| 308 | **Inception: Mercury 2**<br>`inception/mercury-2` | nous_portal | $0.1650 | **14.9** | 13.0 | 75 tps | 128k | 🚫 Dominated |
| 309 | **NVIDIA: Nemotron 3.5 Lightning**<br>`nvidia/nemotron-3.5-lightning` | openrouter_market | $0.0270 | **14.8** | 14.4 | 70 tps | 262k | 🚫 Dominated |
| 310 | **Z.ai: GLM 4.6V**<br>`z-ai/glm-4.6v` | nous_portal | $0.2130 | **14.7** | 12.3 | 75 tps | 131k | 🚫 Dominated |
| 311 | **Z.ai: GLM 4.5 Air**<br>`z-ai/glm-4.5-air` | nous_portal | $0.1390 | **14.6** | 12.9 | 75 tps | 131k | 🚫 Dominated |
| 312 | **Qwen: Qwen3 Coder 30B A3B Instruct**<br>`qwen/qwen3-coder-30b-a3b-instruct` | nous_portal | $0.0574 | **14.4** | 13.6 | 90 tps | 262k | 🚫 Dominated |
| 313 | **OpenAI: o3 Mini High**<br>`openai/o3-mini-high` | nous_portal | $1.1000 | **14.0** | 8.0 | 130 tps | 200k | 🚫 Dominated |
| 314 | **OpenAI: gpt-oss-20b**<br>`openai/gpt-oss-20b` | nous_portal | $0.0198 | **13.6** | 13.4 | 75 tps | 131k | 🚫 Dominated |
| 315 | **Qwen: Qwen3 VL 235B A22B Instruct**<br>`qwen/qwen3-vl-235b-a22b-instruct` | nous_portal | $0.2140 | **13.6** | 11.4 | 75 tps | 262k | 🚫 Dominated |
| 316 | **Mistral: Ministral 3 14B 2512**<br>`mistralai/ministral-14b-2512` | nous_portal | $0.0920 | **13.6** | 12.5 | 130 tps | 262k | 🚫 Dominated |
| 317 | **Qwen: Qwen3 Next 80B A3B Instruct**<br>`qwen/qwen3-next-80b-a3b-instruct` | nous_portal | $0.1478 | **13.3** | 11.7 | 75 tps | 262k | 🚫 Dominated |
| 318 | **Qwen: Qwen3 VL 30B A3B Thinking**<br>`qwen/qwen3-vl-30b-a3b-thinking` | nous_portal | $0.2218 | **13.2** | 11.0 | 75 tps | 262k | 🚫 Dominated |
| 319 | **Mistral: Mistral Small 4**<br>`mistralai/mistral-small-2603` | nous_portal_batch | $0.1140 | **13.1** | 11.8 | 150 tps | 262k | 🚫 Dominated |
| 320 | **Arcee AI: Trinity Large Thinking**<br>`arcee-ai/trinity-large-thinking` | nous_portal | $0.1910 | **13.0** | 11.1 | 75 tps | 262k | 🚫 Dominated |
| 321 | **Amazon: Nova Premier 1.0**<br>`amazon/nova-premier-v1` | nous_portal | $2.3750 | **12.9** | 5.8 | 75 tps | 1000k | 🚫 Dominated |
| 322 | **Llama 3.3 70B Instruct**<br>`llama-3.3-70b` | nous_portal | $0.4100 | **12.8** | 9.6 | 75 tps | 128k | 🚫 Dominated |
| 323 | **Llama 3.3 70B Instruct**<br>`llama-3.3-70b` | nous_portal | $0.4100 | **12.8** | 9.6 | 75 tps | 128k | 🚫 Dominated |
| 324 | **Meta: Llama 3.3 70B Instruct**<br>`meta-llama/llama-3.3-70b-instruct` | nous_portal | $0.0740 | **12.8** | 12.0 | 75 tps | 131k | 🚫 Dominated |
| 325 | **Mistral: Mistral Medium 3**<br>`mistralai/mistral-medium-3` | nous_portal | $0.3440 | **12.7** | 9.8 | 75 tps | 131k | 🚫 Dominated |
| 326 | **DeepSeek R1**<br>`deepseek-r1` | nous_portal | $0.4100 | **12.7** | 9.4 | 75 tps | 64k | 🚫 Dominated |
| 327 | **DeepSeek R1**<br>`deepseek-r1` | nous_portal | $0.4100 | **12.7** | 9.4 | 75 tps | 64k | 🚫 Dominated |
| 328 | **DeepSeek: R1**<br>`deepseek/deepseek-r1` | nous_portal | $0.5440 | **12.7** | 8.8 | 75 tps | 64k | 🚫 Dominated |
| 329 | **OpenAI: GPT-4.1 Nano**<br>`openai/gpt-4.1-nano` | nous_portal_batch | $0.0880 | **12.7** | 11.7 | 150 tps | 1047k | 🚫 Dominated |
| 330 | **OpenAI: GPT-4**<br>`openai/gpt-4` | nous_portal | $18.6000 | **12.6** | 3.2 | 75 tps | 8k | 🚫 Dominated |
| 331 | **OpenAI: GPT-4o (2024-11-20)**<br>`openai/gpt-4o-2024-11-20` | nous_portal | $2.5000 | **12.2** | 5.4 | 75 tps | 128k | 🚫 Dominated |
| 332 | **IBM: Granite 4.2 8B**<br>`ibm-granite/granite-4.2-8b` | nous_portal | $0.0520 | **12.1** | 11.6 | 75 tps | 131k | 🚫 Dominated |
| 333 | **Qwen: Qwen3 235B A22B Thinking 2507**<br>`qwen/qwen3-235b-a22b-thinking-2507` | nous_portal | $0.5390 | **12.1** | 8.4 | 75 tps | 128k | 🚫 Dominated |
| 334 | **Mistral: Mistral Medium 3.1**<br>`mistralai/mistral-medium-3.1` | nous_portal_batch | $0.3440 | **11.9** | 9.2 | 150 tps | 131k | 🚫 Dominated |
| 335 | **Mistral: Ministral 3 8B 2512**<br>`mistralai/ministral-8b-2512` | nous_portal_batch | $0.0690 | **11.8** | 11.1 | 150 tps | 262k | 🚫 Dominated |
| 336 | **Qwen: Qwen3 VL 30B A3B Instruct**<br>`qwen/qwen3-vl-30b-a3b-instruct` | nous_portal | $0.1230 | **11.8** | 10.5 | 75 tps | 262k | 🚫 Dominated |
| 337 | **Qwen 2.5 Coder 32B**<br>`qwen-2.5-coder-32b` | nous_portal | $0.4100 | **11.7** | 8.7 | 105 tps | 131k | 🚫 Dominated |
| 338 | **Qwen 2.5 Coder 32B**<br>`qwen-2.5-coder-32b` | nous_portal | $0.4100 | **11.7** | 8.7 | 105 tps | 131k | 🚫 Dominated |
| 339 | **OpenAI: GPT-4o (2024-08-06)**<br>`openai/gpt-4o-2024-08-06` | nous_portal | $2.5000 | **11.6** | 5.1 | 75 tps | 128k | 🚫 Dominated |
| 340 | **Qwen2.5 72B Instruct**<br>`qwen/qwen-2.5-72b-instruct` | nous_portal | $0.1912 | **11.6** | 9.8 | 75 tps | 32k | 🚫 Dominated |
| 341 | **Qwen: Qwen3 30B A3B**<br>`qwen/qwen3-30b-a3b` | nous_portal | $0.1004 | **11.5** | 10.5 | 75 tps | 40k | 🚫 Dominated |
| 342 | **Z.ai: GLM 4.5V**<br>`z-ai/glm-4.5v` | nous_portal | $0.4260 | **11.5** | 8.5 | 75 tps | 65k | 🚫 Dominated |
| 343 | **Mistral Large 2407**<br>`mistralai/mistral-large-2407` | nous_portal | $1.3200 | **11.5** | 6.2 | 75 tps | 131k | 🚫 Dominated |
| 344 | **DeepSeek: DeepSeek V3 0324**<br>`deepseek/deepseek-chat-v3-0324` | nous_portal | $0.2050 | **11.5** | 9.7 | 75 tps | 163k | 🚫 Dominated |
| 345 | **Upstage: Solar Pro 3**<br>`upstage/solar-pro-3` | nous_portal | $0.1140 | **11.4** | 10.3 | 90 tps | 131k | 🚫 Dominated |
| 346 | **Qwen: Qwen3 30B A3B Instruct 2507**<br>`qwen/qwen3-30b-a3b-instruct-2507` | nous_portal | $0.0678 | **11.4** | 10.7 | 75 tps | 262k | 🚫 Dominated |
| 347 | **Mistral: Mistral Large 3 2512**<br>`mistralai/mistral-large-2512` | nous_portal_batch | $0.3300 | **11.3** | 8.8 | 150 tps | 262k | 🚫 Dominated |
| 348 | **Qwen: Qwen3 VL 8B Instruct**<br>`qwen/qwen3-vl-8b-instruct` | nous_portal | $0.2220 | **11.2** | 9.3 | 75 tps | 262k | 🚫 Dominated |
| 349 | **OpenAI: GPT-3.5 Turbo**<br>`openai/gpt-3.5-turbo` | nous_portal_batch | $0.3600 | **11.2** | 8.5 | 150 tps | 16k | 🚫 Dominated |
| 350 | **Amazon: Nova Pro 1.0**<br>`amazon/nova-pro-v1` | nous_portal | $0.6560 | **10.9** | 7.3 | 75 tps | 300k | 🚫 Dominated |
| 351 | **Hermes 3 (Llama 3.1 70B)**<br>`hermes-3-llama-3.1-70b` | nous_portal | $0.4100 | **10.7** | 7.9 | 78 tps | 128k | 🚫 Dominated |
| 352 | **Hermes 3 (Llama 3.1 70B)**<br>`hermes-3-llama-3.1-70b` | nous_portal | $0.4100 | **10.7** | 7.9 | 78 tps | 128k | 🚫 Dominated |
| 353 | **Amazon: Nova Lite 1.0**<br>`amazon/nova-lite-v1` | nous_portal | $0.0492 | **10.7** | 10.2 | 75 tps | 300k | 🚫 Dominated |
| 354 | **Meta: Llama 3.1 70B Instruct**<br>`meta-llama/llama-3.1-70b-instruct` | nous_portal | $0.2080 | **10.6** | 8.9 | 75 tps | 131k | 🚫 Dominated |
| 355 | **Mistral: Saba**<br>`mistralai/mistral-saba` | nous_portal | $0.1320 | **10.5** | 9.3 | 75 tps | 32k | 🚫 Dominated |
| 356 | **Mistral: Ministral 3 3B 2512**<br>`mistralai/ministral-3b-2512` | nous_portal | $0.0460 | **10.5** | 10.0 | 130 tps | 131k | 🚫 Dominated |
| 357 | **Meta: Llama 3.1 8B Instruct**<br>`meta-llama/llama-3.1-8b-instruct` | nous_portal | $0.0124 | **10.4** | 10.3 | 75 tps | 131k | 🚫 Dominated |
| 358 | **Amazon: Nova Micro 1.0**<br>`amazon/nova-micro-v1` | nous_portal | $0.0287 | **10.0** | 9.7 | 75 tps | 128k | 🚫 Dominated |
| 359 | **Meta: Llama 4 Maverick**<br>`meta-llama/llama-4-maverick` | nous_portal | $0.1515 | **9.9** | 8.7 | 75 tps | 1048k | 🚫 Dominated |
| 360 | **Mistral Large**<br>`mistralai/mistral-large` | nous_portal | $1.3200 | **9.9** | 5.4 | 75 tps | 128k | 🚫 Dominated |
| 361 | **Qwen: Qwen3 32B**<br>`qwen/qwen3-32b` | nous_portal | $0.0616 | **9.8** | 9.2 | 75 tps | 131k | 🚫 Dominated |
| 362 | **Mistral: Mixtral 8x22B Instruct**<br>`mistralai/mixtral-8x22b-instruct` | nous_portal | $1.3200 | **9.8** | 5.3 | 75 tps | 65k | 🚫 Dominated |
| 363 | **NVIDIA: Nemotron 3 Nano 30B A3B**<br>`nvidia/nemotron-3-nano-30b-a3b` | nous_portal | $0.0410 | **9.6** | 9.2 | 75 tps | 262k | 🚫 Dominated |
| 364 | **Qwen: Qwen3 14B**<br>`qwen/qwen3-14b` | nous_portal | $0.0640 | **9.3** | 8.8 | 75 tps | 40k | 🚫 Dominated |
| 365 | **Hermes 3 (Llama 3.1 405B)**<br>`hermes-3-llama-3.1-405b` | nous_portal | $0.4100 | **8.8** | 6.6 | 40 tps | 128k | 🚫 Dominated |
| 366 | **Hermes 3 (Llama 3.1 405B)**<br>`hermes-3-llama-3.1-405b` | nous_portal | $0.4100 | **8.8** | 6.6 | 40 tps | 128k | 🚫 Dominated |
| 367 | **Google: Gemma 3 27B**<br>`google/gemma-3-27b-it` | nous_portal | $0.0725 | **7.7** | 7.2 | 75 tps | 131k | 🚫 Dominated |
| 368 | **Meta: Llama 4 Scout**<br>`meta-llama/llama-4-scout` | nous_portal | $0.0720 | **7.4** | 6.9 | 75 tps | 1310k | 🚫 Dominated |
| 369 | **Google: Gemma 3 12B**<br>`google/gemma-3-12b-it` | nous_portal | $0.0360 | **6.4** | 6.2 | 75 tps | 131k | 🚫 Dominated |

#### 🔗 Cadeia de Roteamento Preditivo para Subagentes (`agentic`)
| Nível de Escalação | Modelo Designado | Provedor | Custo Blended | Condição de Acionamento |
| :--- | :--- | :--- | :---: | :--- |
| **1. Primário (Custo-Benefício)** | `OpenAI o3-mini` | nous_portal | $0.4675 | Modelo padrão para subtarefas com SLA normal e orçamento otimizado. |
| **2. Escalação Máxima (Líder)** | `Claude 3.7 Sonnet` | nous_portal | $2.7300 | Acionado se complexidade/entropia for crítica ou falha de teste anterior. |
| **3. Fallback de Baixo Custo** | `DeepSeek V3` | nous_portal | $0.2360 | Standby caso o provedor principal sofra throttling ou indisponibilidade. |

---

### 🧠 2.3 Perfil: Reasoning / Math (Raciocínio Lógico & Provas Formais)
> **Descrição:** Cadeias densas de pensamento (CoT), análise matemática, provas de invariantes de concorrência e dedução de alta entropia.  
> **Métricas Primárias:** Hard Prompts ELO & GPQA &bull; **Fórmula:** `S_reasoning = 0.50·HardELO + 0.35·GPQA + 0.15·GeneralELO`

| Rank | Modelo Canônico | Provedor | Custo Blended ($/1M) | Score Composto | ROI Index | Throughput | Janela Ctx | Status Pareto |
| :---: | :--- | :---: | :---: | :---: | :---: | :---: | :---: | :--- |
| 1 | **Xiaomi: MiMo-V2.6-Pro**<br>`xiaomi/mimo-v2.6-pro` | nous_portal | $0.5207 | **89.1** | 62.8 | 75 tps | 1050k | 🏆 **Quality Leader** |
| 2 | **DeepSeek R1**<br>`deepseek-r1` | nous_portal | $1.1240 | **85.1** | 48.5 | 75 tps | 64k | 🏆 **Quality Leader** |
| 3 | **Google: Gemini 3 Flash Preview**<br>`google/gemini-3-flash-preview` | nous_portal_batch | $1.4100 | **84.7** | 45.1 | 150 tps | 1048k | 🚫 Dominated |
| 4 | **Claude 3.7 Sonnet**<br>`claude-3-7-sonnet` | nous_portal | $7.4300 | **82.9** | 26.5 | 75 tps | 200k | 🚫 Dominated |
| 5 | **Xiaomi: MiMo-V2.6-Flash**<br>`xiaomi/mimo-v2.6-flash` | nous_portal | $0.1620 | **82.8** | 72.0 | 130 tps | 1050k | 💎 **Best Value** |
| 6 | **OpenAI o3-mini**<br>`o3-mini` | nous_portal | $1.1275 | **81.9** | 46.7 | 130 tps | 200k | 🚫 Dominated |
| 7 | **Z.ai: GLM 5**<br>`z-ai/glm-5` | nous_portal | $1.0320 | **81.1** | 47.5 | 75 tps | 204k | 🚫 Dominated |
| 8 | **OpenAI: GPT-5.4**<br>`openai/gpt-5.4` | nous_portal_batch | $7.0500 | **80.0** | 25.9 | 150 tps | 1050k | 🚫 Dominated |
| 9 | **Qwen: Qwen3.5 397B A17B**<br>`qwen/qwen3.5-397b-a17b` | nous_portal | $1.1076 | **79.6** | 45.6 | 75 tps | 262k | 🚫 Dominated |
| 10 | **Z.ai: GLM 5V Turbo**<br>`z-ai/glm-5v-turbo` | nous_portal | $2.1280 | **79.6** | 37.2 | 75 tps | 202k | 🚫 Dominated |
| 11 | **Tencent: Hy3**<br>`tencent/hy3` | nous_portal | $0.2706 | **79.5** | 64.2 | 75 tps | 262k | 🚫 Dominated |
| 12 | **Z.ai: GLM 4.5**<br>`z-ai/glm-4.5` | nous_portal | $1.1420 | **78.2** | 44.4 | 75 tps | 131k | 🚫 Dominated |
| 13 | **Qwen: Qwen3.6 Plus**<br>`qwen/qwen3.6-plus` | nous_portal | $0.9230 | **74.4** | 45.0 | 75 tps | 1000k | 🚫 Dominated |
| 14 | **Z.ai: GLM 4.6**<br>`z-ai/glm-4.6` | nous_portal | $0.8880 | **73.1** | 44.7 | 75 tps | 204k | 🚫 Dominated |
| 15 | **GPT-4o**<br>`gpt-4o` | nous_portal | $2.5625 | **72.9** | 32.1 | 75 tps | 128k | 🚫 Dominated |
| 16 | **DeepSeek V3**<br>`deepseek-v3` | nous_portal | $0.5660 | **72.4** | 50.0 | 92 tps | 64k | ⚡ **Ultra Low Cost** |
| 17 | **Z.ai: GLM 4.7**<br>`z-ai/glm-4.7` | nous_portal | $0.8760 | **72.4** | 44.4 | 75 tps | 204k | 🚫 Dominated |
| 18 | **Hermes 3 (Llama 3.1 405B)**<br>`hermes-3-llama-3.1-405b` | nous_portal | $0.8500 | **69.7** | 43.2 | 40 tps | 128k | 🚫 Dominated |
| 19 | **Llama 3.3 70B Instruct**<br>`llama-3.3-70b` | nous_portal | $0.5815 | **66.1** | 45.3 | 75 tps | 128k | 🚫 Dominated |
| 20 | **Qwen 2.5 Coder 32B**<br>`qwen-2.5-coder-32b` | nous_portal | $0.3300 | **65.3** | 50.8 | 105 tps | 131k | 💎 **Best Value** |
| 21 | **Anthropic: Claude Fable 5.1**<br>`anthropic/claude-fable-5.1` | nous_portal_batch | $24.0500 | **62.6** | 14.8 | 150 tps | 1000k | 🚫 Dominated |
| 22 | **Hermes 3 (Llama 3.1 70B)**<br>`hermes-3-llama-3.1-70b` | nous_portal | $0.5180 | **62.0** | 43.7 | 78 tps | 128k | 🚫 Dominated |
| 23 | **Anthropic: Claude Opus 5**<br>`anthropic/claude-opus-5` | nous_portal_batch | $12.1000 | **60.8** | 17.0 | 150 tps | 1000k | 🚫 Dominated |
| 24 | **Qwen: Qwen3.8 Max (0902)**<br>`qwen/qwen3.8-max-0902` | nous_portal | $3.2500 | **60.0** | 24.5 | 75 tps | 1000k | 🚫 Dominated |
| 25 | **Meta: Muse Spark 1.3**<br>`meta/muse-spark-1.3` | nous_portal | $2.2300 | **59.6** | 27.4 | 75 tps | 1048k | 🚫 Dominated |
| 26 | **GPT-4o Mini**<br>`gpt-4o-mini` | openrouter | $0.1537 | **58.0** | 50.8 | 120 tps | 128k | 💎 **Best Value** |
| 27 | **xAI: Grok 4.6**<br>`x-ai/grok-4.6` | nous_portal | $3.3000 | **57.8** | 23.5 | 75 tps | 500k | 🚫 Dominated |
| 28 | **Anthropic: Claude Opus 5.5**<br>`anthropic/claude-opus-5.5` | nous_portal_batch | $9.6400 | **57.6** | 17.1 | 150 tps | 1000k | 🚫 Dominated |
| 29 | **Z.ai: GLM 5.3**<br>`z-ai/glm-5.3` | openrouter_market | $1.5418 | **57.4** | 29.7 | 70 tps | 1048k | 🚫 Dominated |
| 30 | **OpenAI: GPT-6 Astra**<br>`openai/gpt-6-astra` | nous_portal_batch | $24.2000 | **56.2** | 13.3 | 150 tps | 1050k | 🚫 Dominated |
| 31 | **Anthropic: Claude Sonnet 5.5**<br>`anthropic/claude-sonnet-5.5` | nous_portal_batch | $4.8200 | **56.0** | 20.3 | 150 tps | 1000k | 🚫 Dominated |
| 32 | **Claude Fable 5**<br>`anthropic/claude-fable-5` | nous_portal_batch | $24.2000 | **55.9** | 13.2 | 150 tps | 1048k | 🚫 Dominated |
| 33 | **OpenAI: GPT-5.6 Sol**<br>`openai/gpt-5.6-sol` | nous_portal_batch | $4.8400 | **55.6** | 20.1 | 150 tps | 1050k | 🚫 Dominated |
| 34 | **MoonshotAI: Kimi K3**<br>`moonshotai/kimi-k3` | nous_portal | $6.3440 | **55.2** | 18.5 | 75 tps | 1048k | 🚫 Dominated |
| 35 | **Z.ai: GLM 5.3 Flash**<br>`z-ai/glm-5.3-flash` | nous_portal_batch | $0.2128 | **55.0** | 46.1 | 150 tps | 1048k | 🚫 Dominated |
| 36 | **Qwen: Qwen3.8 2.4T A95B**<br>`qwen/qwen3.8-2.4t-a95b` | nous_portal | $3.2500 | **54.5** | 22.3 | 75 tps | 1048k | 🚫 Dominated |
| 37 | **Google: Gemini 4 Argon**<br>`google/gemini-4-argon` | nous_portal_batch | $4.8200 | **52.6** | 19.0 | 150 tps | 1048k | 🚫 Dominated |
| 38 | **OpenAI: GPT-6.1 Sol**<br>`openai/gpt-6.1-sol` | nous_portal_batch | $4.8200 | **51.8** | 18.8 | 150 tps | 1050k | 🚫 Dominated |
| 39 | **DeepSeek: DeepSeek V4 Flash Vision Exp**<br>`deepseek/deepseek-v4-flash-vision-exp` | nous_portal | $0.3463 | **51.0** | 39.3 | 130 tps | 1048k | 🚫 Dominated |
| 40 | **OpenAI: GPT Astra Latest**<br>`openai/gpt-astra-latest` | nous_portal | $24.2000 | **50.8** | 12.0 | 40 tps | 1050k | 🚫 Dominated |
| 41 | **OpenAI: GPT-6 Astra Pro Flex**<br>`openai/gpt-6-astra-pro-flex` | nous_portal | $12.1000 | **50.8** | 14.2 | 40 tps | 1050k | 🚫 Dominated |
| 42 | **OpenAI: GPT-6 Astra Pro Fast**<br>`openai/gpt-6-astra-pro-fast` | nous_portal | $48.4000 | **50.8** | 10.4 | 40 tps | 1050k | 🚫 Dominated |
| 43 | **OpenAI: GPT-6 Astra Fast**<br>`openai/gpt-6-astra-fast` | nous_portal | $48.4000 | **50.8** | 10.4 | 40 tps | 1050k | 🚫 Dominated |
| 44 | **OpenAI: GPT-6 Astra Flex**<br>`openai/gpt-6-astra-flex` | nous_portal | $12.1000 | **50.8** | 14.2 | 40 tps | 1050k | 🚫 Dominated |
| 45 | **Anthropic: Claude Opus Latest**<br>`anthropic/claude-opus-latest` | nous_portal | $9.6400 | **50.8** | 15.1 | 40 tps | 1000k | 🚫 Dominated |
| 46 | **OpenAI: GPT-6 Astra Pro**<br>`openai/gpt-6-astra-pro` | nous_portal_batch | $24.2000 | **50.8** | 12.0 | 150 tps | 1050k | 🚫 Dominated |
| 47 | **Qwen: Qwen3.8 27B**<br>`qwen/qwen3.8-27b` | nous_portal | $0.5164 | **50.3** | 35.5 | 75 tps | 1000k | 🚫 Dominated |
| 48 | **OpenAI: GPT-5.6 Terra**<br>`openai/gpt-5.6-terra` | nous_portal_batch | $5.6400 | **49.9** | 17.2 | 150 tps | 1050k | 🚫 Dominated |
| 49 | **Claude Sonnet 5**<br>`anthropic/claude-sonnet-5` | nous_portal_batch | $4.8400 | **49.2** | 17.8 | 150 tps | 1000k | 🚫 Dominated |
| 50 | **Meta: Muse Spark 1.2**<br>`meta/muse-spark-1.2` | nous_portal | $2.2300 | **49.0** | 22.6 | 75 tps | 1048k | 🚫 Dominated |
| 51 | **Claude Opus 4.8**<br>`anthropic/claude-opus-4.8` | nous_portal_batch | $12.1000 | **48.4** | 13.5 | 150 tps | 1000k | 🚫 Dominated |
| 52 | **OpenAI: GPT-5.6 Luna**<br>`openai/gpt-5.6-luna` | nous_portal_batch | $0.5640 | **48.0** | 33.1 | 150 tps | 1050k | 🚫 Dominated |
| 53 | **Claude 3.5 Sonnet**<br>`claude-3-5-sonnet` | nous_portal | $6.1200 | **47.8** | 16.1 | 75 tps | 200k | 🏆 **Quality Leader** |
| 54 | **Claude 3.7 Sonnet**<br>`claude-3-7-sonnet` | nous_portal | $6.1200 | **47.8** | 16.1 | 75 tps | 200k | 💎 **Best Value** |
| 55 | **Claude 3.7 Sonnet**<br>`claude-3-7-sonnet` | nous_portal | $6.1200 | **47.8** | 16.1 | 75 tps | 200k | 🏆 **Quality Leader** |
| 56 | **Claude 3.5 Sonnet**<br>`claude-3-5-sonnet` | nous_portal | $6.1200 | **47.8** | 16.1 | 75 tps | 200k | 💎 **Best Value** |
| 57 | **OpenAI: GPT Sol Latest**<br>`openai/gpt-sol-latest` | nous_portal | $4.8200 | **47.8** | 17.3 | 90 tps | 1050k | 🚫 Dominated |
| 58 | **Anthropic: Claude Sonnet Latest**<br>`anthropic/claude-sonnet-latest` | nous_portal | $4.8200 | **47.8** | 17.3 | 75 tps | 1000k | 🚫 Dominated |
| 59 | **OpenAI: GPT-5.6 Sol Pro**<br>`openai/gpt-5.6-sol-pro` | nous_portal_batch | $4.8400 | **47.8** | 17.3 | 150 tps | 1050k | 🚫 Dominated |
| 60 | **OpenAI: GPT-6 Sol Pro**<br>`openai/gpt-6-sol-pro` | nous_portal_batch | $4.8400 | **47.8** | 17.3 | 150 tps | 1050k | 🚫 Dominated |
| 61 | **Z.ai: GLM 5.3 Prime**<br>`z-ai/glm-5.3-prime` | nous_portal | $4.7520 | **47.8** | 17.4 | 75 tps | 1000k | 🚫 Dominated |
| 62 | **OpenAI: GPT-6.1 Sol Pro**<br>`openai/gpt-6.1-sol-pro` | nous_portal_batch | $4.8200 | **47.8** | 17.3 | 150 tps | 1050k | 🚫 Dominated |
| 63 | **Qwen: Qwen3.8 Max Prime**<br>`qwen/qwen3.8-max-prime` | nous_portal | $6.5000 | **47.8** | 15.8 | 75 tps | 1000k | 🚫 Dominated |
| 64 | **OpenAI: GPT-6 Sol**<br>`openai/gpt-6-sol` | nous_portal_batch | $4.8400 | **47.6** | 17.2 | 150 tps | 1050k | 🚫 Dominated |
| 65 | **xAI: Grok 4.5**<br>`x-ai/grok-4.5` | nous_portal | $3.2600 | **47.4** | 19.4 | 75 tps | 500k | 🚫 Dominated |
| 66 | **Google: Gemini 3.8 Flash**<br>`google/gemini-3.8-flash` | nous_portal_batch | $1.8150 | **47.4** | 23.3 | 150 tps | 1048k | 🚫 Dominated |
| 67 | **DeepSeek: DeepSeek V4 Pro 0813**<br>`deepseek/deepseek-v4-pro-0813` | nous_portal | $3.3080 | **46.8** | 19.0 | 75 tps | 1048k | 🚫 Dominated |
| 68 | **DeepSeek: DeepSeek V4 Flash**<br>`deepseek/deepseek-v4-flash-0731` | openrouter_market | $0.0868 | **46.6** | 43.0 | 130 tps | 1048k | 💎 **Best Value** |
| 69 | **xAI: Grok 4.7**<br>`x-ai/grok-4.7` | nous_portal | $3.3000 | **46.4** | 18.9 | 75 tps | 500k | 🚫 Dominated |
| 70 | **Google: Gemini 3.7 Flash**<br>`google/gemini-3.7-flash` | nous_portal_batch | $1.8150 | **46.0** | 22.6 | 150 tps | 1048k | 🚫 Dominated |
| 71 | **Claude Opus 4.7**<br>`anthropic/claude-opus-4.7` | nous_portal_batch | $12.1000 | **45.6** | 12.8 | 150 tps | 1000k | 🚫 Dominated |
| 72 | **Z.ai: GLM 5.2**<br>`z-ai/glm-5.2` | openrouter_market | $1.5240 | **44.5** | 23.1 | 70 tps | 1048k | 🚫 Dominated |
| 73 | **OpenAI: GPT-5.5**<br>`openai/gpt-5.5` | nous_portal_batch | $14.1000 | **44.1** | 11.9 | 150 tps | 1050k | 🚫 Dominated |
| 74 | **Anthropic: Claude Haiku 5.5**<br>`anthropic/claude-haiku-5.5` | nous_portal_batch | $0.2420 | **43.4** | 35.7 | 150 tps | 1000k | 🚫 Dominated |
| 75 | **StepFun: Step 5 Preview**<br>`stepfun/step-5-preview` | nous_portal | $1.4900 | **42.8** | 22.4 | 75 tps | 1000k | 🚫 Dominated |
| 76 | **Sakana: Fugu Max**<br>`sakana/fugu-max` | nous_portal | $3.2500 | **42.8** | 17.5 | 75 tps | 1000k | 🚫 Dominated |
| 77 | **DeepSeek: DeepSeek Pro Latest**<br>`deepseek/deepseek-pro-latest` | nous_portal | $3.3080 | **42.8** | 17.4 | 75 tps | 1048k | 🚫 Dominated |
| 78 | **OpenAI: GPT Terra Latest**<br>`openai/gpt-terra-latest` | nous_portal | $5.6400 | **42.8** | 14.8 | 75 tps | 1050k | 🚫 Dominated |
| 79 | **Z.ai: GLM Latest**<br>`z-ai/glm-latest` | nous_portal | $1.7969 | **42.8** | 21.1 | 75 tps | 1048k | 🚫 Dominated |
| 80 | **Qwen: Qwen3.5 Plus 2026-02-15**<br>`qwen/qwen3.5-plus-02-15` | nous_portal | $0.7384 | **42.8** | 27.5 | 75 tps | 1000k | 🚫 Dominated |
| 81 | **MoonshotAI: Kimi K3**<br>`moonshotai/kimi-k3:US` | nous_portal | $8.7846 | **42.8** | 13.0 | 75 tps | 1048k | 🚫 Dominated |
| 82 | **Poolside: Laguna S 2.1**<br>`poolside/laguna-s-2.1` | openrouter_market | $0.1098 | **42.8** | 38.7 | 70 tps | 1048k | 🚫 Dominated |
| 83 | **Meta: Muse Wonder**<br>`meta/muse-wonder` | nous_portal | $9.6400 | **42.8** | 12.7 | 75 tps | 1048k | 🚫 Dominated |
| 84 | **Anthropic: Claude Fable Latest**<br>`anthropic/claude-fable-latest` | nous_portal | $24.0500 | **42.8** | 10.1 | 75 tps | 1000k | 🚫 Dominated |
| 85 | **Fireworks: Ember-1**<br>`fireworks/ember-1` | nous_portal | $7.2600 | **42.8** | 13.7 | 75 tps | 1048k | 🚫 Dominated |
| 86 | **Qwen: Qwen-Plus**<br>`qwen/qwen-plus` | nous_portal | $0.4264 | **42.8** | 31.5 | 75 tps | 1000k | 🚫 Dominated |
| 87 | **Pareto 26.10 Preview**<br>`unbiased/pareto-26.10-preview` | nous_portal | $1.6060 | **42.8** | 21.8 | 75 tps | 1048k | 🚫 Dominated |
| 88 | **Tencent: Hy4 preview**<br>`tencent/hy4-preview` | nous_portal | $0.6710 | **42.8** | 28.2 | 75 tps | 1048k | 🚫 Dominated |
| 89 | **Sakana: Fugu Ultra v2**<br>`sakana/fugu-ultra-v2` | nous_portal | $14.1000 | **42.8** | 11.5 | 75 tps | 1000k | 🚫 Dominated |
| 90 | **OpenAI: GPT-5.4 Pro**<br>`openai/gpt-5.4-pro` | nous_portal_batch | $85.2000 | **42.8** | 7.8 | 150 tps | 1050k | 🚫 Dominated |
| 91 | **Mistral: Mistral Large 4**<br>`mistralai/mistral-large-4-0` | nous_portal | $1.1220 | **42.8** | 24.4 | 75 tps | 1048k | 🚫 Dominated |
| 92 | **Qwen: Qwen3.5 Plus 2026-04-20**<br>`qwen/qwen3.5-plus-20260420` | nous_portal | $0.8520 | **42.8** | 26.4 | 75 tps | 1000k | 🚫 Dominated |
| 93 | **MoonshotAI: Kimi Latest**<br>`moonshotai/kimi-latest` | nous_portal | $6.3440 | **42.8** | 14.3 | 75 tps | 1048k | 🚫 Dominated |
| 94 | **xAI: Grok Latest**<br>`x-ai/grok-latest` | nous_portal | $3.3000 | **42.8** | 17.4 | 75 tps | 500k | 🚫 Dominated |
| 95 | **OpenAI: GPT-5.6 Terra Pro**<br>`openai/gpt-5.6-terra-pro` | nous_portal_batch | $5.6400 | **42.8** | 14.8 | 150 tps | 1050k | 🚫 Dominated |
| 96 | **Sakana: Fugu Ultra**<br>`sakana/fugu-ultra` | nous_portal | $14.1000 | **42.8** | 11.5 | 75 tps | 1000k | 🚫 Dominated |
| 97 | **OpenAI: GPT-5.5 Pro**<br>`openai/gpt-5.5-pro` | nous_portal_batch | $85.2000 | **42.8** | 7.8 | 150 tps | 1050k | 🚫 Dominated |
| 98 | **Xiaomi: MiMo-V2.6-Pro-UltraSpeed**<br>`xiaomi/mimo-v2.6-pro-ultraspeed` | nous_portal | $5.2272 | **42.8** | 15.1 | 75 tps | 1048k | 🚫 Dominated |
| 99 | **Z.ai: GLM 5.3**<br>`z-ai/glm-5.3:US` | nous_portal | $3.9138 | **42.8** | 16.5 | 75 tps | 1048k | 🚫 Dominated |
| 100 | **Gemini 2.0 Flash**<br>`gemini-2.0-flash` | nous_portal | $1.0200 | **41.8** | 24.5 | 130 tps | 1000k | 💎 **Best Value** |
| 101 | **Gemini 2.0 Flash**<br>`gemini-2.0-flash` | nous_portal | $1.0200 | **41.8** | 24.5 | 130 tps | 1000k | 💎 **Best Value** |
| 102 | **AionLabs: Aion 3.5 Mini**<br>`aion-labs/aion-3.5-mini` | nous_portal | $0.8760 | **41.8** | 25.6 | 130 tps | 262k | 🚫 Dominated |
| 103 | **Google: Nano Banana 2.1**<br>`google/gemini-nano-banana-2.1` | nous_portal | $3.6600 | **41.8** | 16.4 | 130 tps | 65k | 🚫 Dominated |
| 104 | **ByteDance Seed: Seed 1.6 Flash**<br>`bytedance-seed/seed-1.6-flash` | nous_portal | $0.1530 | **41.8** | 36.5 | 130 tps | 262k | 🚫 Dominated |
| 105 | **Anthropic: Claude Haiku Latest**<br>`anthropic/claude-haiku-latest` | nous_portal | $0.2420 | **41.8** | 34.3 | 130 tps | 1000k | 🚫 Dominated |
| 106 | **Google: Gemini Embedding 2 Preview**<br>`google/gemini-embedding-2-preview` | nous_portal | $0.0880 | **41.8** | 38.5 | 130 tps | 8k | 🚫 Dominated |
| 107 | **Google: Gemini 3.1 Pro Preview Custom Tools**<br>`google/gemini-3.1-pro-preview-customtools` | nous_portal | $5.6400 | **41.8** | 14.4 | 130 tps | 1048k | 🚫 Dominated |
| 108 | **AionLabs: Aion-3.0-Mini**<br>`aion-labs/aion-3.0-mini` | nous_portal | $0.8760 | **41.8** | 25.6 | 130 tps | 131k | 🚫 Dominated |
| 109 | **Google: Nano Banana Pro (Gemini 3 Pro Image)**<br>`google/gemini-3-pro-image` | nous_portal | $5.6400 | **41.8** | 14.4 | 130 tps | 131k | 🚫 Dominated |
| 110 | **Qwen: Qwen3.8 Flash**<br>`qwen/qwen3.8-flash` | nous_portal | $0.2512 | **41.8** | 34.1 | 130 tps | 1000k | 🚫 Dominated |
| 111 | **DeepSeek: DeepSeek V4 Flash Latest**<br>`deepseek/deepseek-v4-flash-latest` | nous_portal | $0.2383 | **41.8** | 34.4 | 130 tps | 1048k | 🚫 Dominated |
| 112 | **Z.ai: GLM 5.3 FlashX**<br>`z-ai/glm-5.3-flashx` | nous_portal | $0.6660 | **41.8** | 27.6 | 130 tps | 1048k | 🚫 Dominated |
| 113 | **Google: Gemini Embedding 001**<br>`google/gemini-embedding-001` | nous_portal | $0.0660 | **41.8** | 39.2 | 130 tps | 20k | 🚫 Dominated |
| 114 | **Google: Gemini 3.1 Flash Lite**<br>`google/gemini-3.1-flash-lite` | nous_portal_batch | $0.7050 | **41.8** | 27.2 | 150 tps | 1048k | 🚫 Dominated |
| 115 | **OpenAI: GPT Mini Latest**<br>`openai/gpt-mini-latest` | nous_portal | $2.1150 | **41.8** | 19.5 | 130 tps | 400k | 🚫 Dominated |
| 116 | **OpenAI: GPT Audio Mini**<br>`openai/gpt-audio-mini` | nous_portal | $1.2240 | **41.8** | 23.2 | 130 tps | 128k | 🚫 Dominated |
| 117 | **Google: Gemini Embedding 2**<br>`google/gemini-embedding-2` | nous_portal_batch | $0.0880 | **41.8** | 38.5 | 150 tps | 8k | 🚫 Dominated |
| 118 | **OpenAI: GPT-4o-mini (2024-07-18)**<br>`openai/gpt-4o-mini-2024-07-18` | nous_portal | $0.3150 | **41.8** | 32.8 | 130 tps | 128k | 🚫 Dominated |
| 119 | **OpenAI: GPT-5.6 Luna Pro**<br>`openai/gpt-5.6-luna-pro` | nous_portal_batch | $0.5640 | **41.8** | 28.8 | 150 tps | 1050k | 🚫 Dominated |
| 120 | **Sentence Transformers: all-MiniLM-L12-v2**<br>`sentence-transformers/all-minilm-l12-v2` | nous_portal | $0.0022 | **41.8** | 41.7 | 130 tps | 0k | 💎 **Best Value** |
| 121 | **DeepSeek: DeepSeek V4 Flash**<br>`deepseek/deepseek-v4-flash-0731:US` | nous_portal | $0.5831 | **41.8** | 28.6 | 130 tps | 1048k | 🚫 Dominated |
| 122 | **Qwen: Qwen3.5-Flash**<br>`qwen/qwen3.5-flash-02-23` | nous_portal | $0.1326 | **41.8** | 37.1 | 130 tps | 1000k | 🚫 Dominated |
| 123 | **DeepSeek: DeepSeek Flash Latest**<br>`deepseek/deepseek-flash-latest` | openrouter_market | $0.2453 | **41.8** | 34.2 | 130 tps | 1048k | 🚫 Dominated |
| 124 | **Qwen: Qwen3 Coder Flash**<br>`qwen/qwen3-coder-flash` | nous_portal | $0.4758 | **41.8** | 30.1 | 130 tps | 1000k | 🚫 Dominated |
| 125 | **DeepSeek: DeepSeek V4.1 Flash**<br>`deepseek/deepseek-v4.1-flash:US` | nous_portal | $0.5831 | **41.8** | 28.6 | 130 tps | 1048k | 🚫 Dominated |
| 126 | **Google: Gemini Flash Latest**<br>`google/gemini-flash-latest` | nous_portal | $1.8150 | **41.8** | 20.5 | 130 tps | 1048k | 🚫 Dominated |
| 127 | **OpenAI: GPT Luna Latest**<br>`openai/gpt-luna-latest` | nous_portal | $0.2420 | **41.8** | 34.3 | 130 tps | 1050k | 🚫 Dominated |
| 128 | **Z.ai: GLM Flash Latest**<br>`z-ai/glm-flash-latest` | nous_portal | $0.3766 | **41.8** | 31.6 | 130 tps | 1048k | 🚫 Dominated |
| 129 | **Sentence Transformers: paraphrase-MiniLM-L6-v2**<br>`sentence-transformers/paraphrase-minilm-l6-v2` | nous_portal | $0.0022 | **41.8** | 41.7 | 130 tps | 0k | 💎 **Best Value** |
| 130 | **ByteDance Seed: Seed-2.0-Mini**<br>`bytedance-seed/seed-2.0-mini` | nous_portal | $0.2040 | **41.8** | 35.2 | 130 tps | 262k | 🚫 Dominated |
| 131 | **Qwen: Qwen3.6 Flash**<br>`qwen/qwen3.6-flash` | nous_portal | $0.5325 | **41.8** | 29.3 | 130 tps | 1000k | 🚫 Dominated |
| 132 | **Google: Gemini Pro Latest**<br>`google/gemini-pro-latest` | nous_portal | $5.6400 | **41.8** | 14.4 | 130 tps | 1048k | 🚫 Dominated |
| 133 | **Z.ai: GLM 5.3 Flash**<br>`z-ai/glm-5.3-flash:US` | nous_portal | $0.4389 | **41.8** | 30.6 | 130 tps | 1048k | 🚫 Dominated |
| 134 | **Sentence Transformers: all-MiniLM-L6-v2**<br>`sentence-transformers/all-minilm-l6-v2` | nous_portal | $0.0022 | **41.8** | 41.7 | 130 tps | 0k | 💎 **Best Value** |
| 135 | **Qwen: Qwen3.8 Omni Flash**<br>`qwen/qwen3.8-omni-flash` | nous_portal | $0.2512 | **41.8** | 34.1 | 130 tps | 1000k | 🚫 Dominated |
| 136 | **Qwen: Qwen3.7 Flash**<br>`qwen/qwen3.7-flash` | nous_portal | $0.0652 | **41.8** | 39.3 | 130 tps | 1000k | 🚫 Dominated |
| 137 | **OpenAI: GPT-6 Luna Pro**<br>`openai/gpt-6-luna-pro` | nous_portal_batch | $0.2420 | **41.8** | 34.3 | 150 tps | 1050k | 🚫 Dominated |
| 138 | **Google: Gemini 2.5 Pro Preview 06-05**<br>`google/gemini-2.5-pro-preview` | nous_portal | $4.5250 | **41.8** | 15.4 | 130 tps | 1048k | 🚫 Dominated |
| 139 | **OpenAI: o4 Mini High**<br>`openai/o4-mini-high` | nous_portal | $2.2550 | **41.8** | 19.1 | 130 tps | 200k | 🚫 Dominated |
| 140 | **inclusionAI: Ling 3.1 Flash**<br>`inclusionai/ling-3.1-flash` | nous_portal | $0.1230 | **41.1** | 36.8 | 130 tps | 262k | 🚫 Dominated |
| 141 | **DeepSeek: DeepSeek V4.1 Flash**<br>`deepseek/deepseek-v4.1-flash` | openrouter_market | $0.2484 | **39.5** | 32.3 | 130 tps | 1048k | 🚫 Dominated |
| 142 | **OpenAI: GPT-6 Luna**<br>`openai/gpt-6-luna` | nous_portal_batch | $0.2420 | **38.1** | 31.3 | 150 tps | 1050k | 🚫 Dominated |
| 143 | **Claude Sonnet 4.6**<br>`anthropic/claude-sonnet-4.6` | nous_portal_batch | $7.2600 | **38.0** | 12.2 | 150 tps | 1000k | 🚫 Dominated |
| 144 | **DeepSeek V3**<br>`deepseek-v3` | nous_portal | $1.0200 | **37.8** | 22.2 | 92 tps | 64k | 🚫 Dominated |
| 145 | **GPT-4o**<br>`gpt-4o` | nous_portal | $1.0200 | **37.8** | 22.2 | 75 tps | 128k | 🚫 Dominated |
| 146 | **GPT-4o**<br>`gpt-4o` | nous_portal | $1.0200 | **37.8** | 22.2 | 75 tps | 128k | 🚫 Dominated |
| 147 | **DeepSeek V3**<br>`deepseek-v3` | nous_portal | $1.0200 | **37.8** | 22.2 | 92 tps | 64k | 🚫 Dominated |
| 148 | **ByteDance Seed: Seed-2.0-Lite**<br>`bytedance-seed/seed-2.0-lite` | nous_portal | $0.9100 | **37.8** | 22.9 | 75 tps | 262k | 🚫 Dominated |
| 149 | **Qwen: Qwen3 Embedding 8B**<br>`qwen/qwen3-embedding-8b` | nous_portal | $0.0044 | **37.8** | 37.6 | 75 tps | 32k | 🚫 Dominated |
| 150 | **Intfloat: E5-Base-v2**<br>`intfloat/e5-base-v2` | nous_portal | $0.0022 | **37.8** | 37.7 | 75 tps | 0k | 🚫 Dominated |
| 151 | **OpenAI: Text Embedding Ada 002**<br>`openai/text-embedding-ada-002` | nous_portal_batch | $0.0440 | **37.8** | 36.2 | 150 tps | 8k | 🚫 Dominated |
| 152 | **Sentence Transformers: multi-qa-mpnet-base-dot-v1**<br>`sentence-transformers/multi-qa-mpnet-base-dot-v1` | nous_portal | $0.0022 | **37.8** | 37.7 | 75 tps | 0k | 🚫 Dominated |
| 153 | **BAAI: bge-base-en-v1.5**<br>`baai/bge-base-en-v1.5` | nous_portal | $0.0022 | **37.8** | 37.7 | 75 tps | 0k | 🚫 Dominated |
| 154 | **OpenAI: gpt-oss-safeguard-20b**<br>`openai/gpt-oss-safeguard-20b` | nous_portal | $0.1575 | **37.8** | 32.9 | 75 tps | 131k | 🚫 Dominated |
| 155 | **Pareto Latest**<br>`unbiased/pareto-latest` | nous_portal | $4.0500 | **37.8** | 14.4 | 75 tps | 262k | 🚫 Dominated |
| 156 | **Intfloat: E5-Large-v2**<br>`intfloat/e5-large-v2` | nous_portal | $0.0044 | **37.8** | 37.6 | 75 tps | 0k | 🚫 Dominated |
| 157 | **VoyageAI by MongoDB: voyage-multimodal-3.5**<br>`voyageai/voyage-multimodal-3.5` | nous_portal | $0.0528 | **37.8** | 35.9 | 75 tps | 32k | 🚫 Dominated |
| 158 | **Qwen: Qwen2.5 7B Instruct**<br>`qwen/qwen-2.5-7b-instruct` | nous_portal | $0.1240 | **37.8** | 33.8 | 75 tps | 32k | 🚫 Dominated |
| 159 | **Nex AGI: Nex-N2.5-Pro**<br>`nex-agi/nex-n2.5-pro` | nous_portal | $0.1330 | **37.8** | 33.6 | 75 tps | 262k | 🚫 Dominated |
| 160 | **Sentence Transformers: all-mpnet-base-v2**<br>`sentence-transformers/all-mpnet-base-v2` | nous_portal | $0.0022 | **37.8** | 37.7 | 75 tps | 0k | 🚫 Dominated |
| 161 | **Poolside: Laguna XS 2.1**<br>`poolside/laguna-xs-2.1` | openrouter_market | $0.0780 | **37.8** | 35.1 | 70 tps | 262k | 🚫 Dominated |
| 162 | **Perplexity: Embed V1 4B**<br>`perplexity/pplx-embed-v1-4b` | nous_portal | $0.0132 | **37.8** | 37.3 | 75 tps | 32k | 🚫 Dominated |
| 163 | **ByteDance Seed: Seed 1.6**<br>`bytedance-seed/seed-1.6` | nous_portal | $0.9100 | **37.8** | 22.9 | 75 tps | 262k | 🚫 Dominated |
| 164 | **OpenAI: Text Embedding 3 Large**<br>`openai/text-embedding-3-large` | nous_portal_batch | $0.0572 | **37.8** | 35.8 | 150 tps | 8k | 🚫 Dominated |
| 165 | **Perplexity: Embed V1 0.6B**<br>`perplexity/pplx-embed-v1-0.6b` | nous_portal | $0.0018 | **37.8** | 37.7 | 75 tps | 32k | 🚫 Dominated |
| 166 | **Intfloat: Multilingual-E5-Large**<br>`intfloat/multilingual-e5-large` | nous_portal | $0.0044 | **37.8** | 37.6 | 75 tps | 0k | 🚫 Dominated |
| 167 | **Mistral: Mistral Small 3.2 24B**<br>`mistralai/mistral-small-3.2-24b-instruct` | nous_portal | $0.1130 | **37.8** | 34.1 | 75 tps | 256k | 🚫 Dominated |
| 168 | **OpenAI: GPT-3.5 Turbo 16k**<br>`openai/gpt-3.5-turbo-16k` | nous_portal | $2.9200 | **37.8** | 16.0 | 75 tps | 16k | 🚫 Dominated |
| 169 | **Cohere: Command A+**<br>`cohere/command-a-plus` | nous_portal | $0.7500 | **37.8** | 24.2 | 75 tps | 192k | 🚫 Dominated |
| 170 | **OpenAI: GPT-5 Pro**<br>`openai/gpt-5-pro` | nous_portal_batch | $54.6000 | **37.8** | 7.5 | 150 tps | 400k | 🚫 Dominated |
| 171 | **Pareto 26.9**<br>`unbiased/pareto-26.9` | nous_portal | $4.0500 | **37.8** | 14.4 | 75 tps | 262k | 🚫 Dominated |
| 172 | **AionLabs: Aion-2.0**<br>`aion-labs/aion-2.0` | nous_portal | $1.0000 | **37.8** | 22.3 | 75 tps | 131k | 🚫 Dominated |
| 173 | **ByteDance Seed: Seed 2.1 Turbo**<br>`bytedance-seed/seed-2-1-turbo` | nous_portal | $1.2200 | **37.8** | 21.0 | 75 tps | 262k | 🚫 Dominated |
| 174 | **BAAI: bge-large-en-v1.5**<br>`baai/bge-large-en-v1.5` | nous_portal | $0.0044 | **37.8** | 37.6 | 75 tps | 0k | 🚫 Dominated |
| 175 | **OpenAI: GPT-5.2 Chat**<br>`openai/gpt-5.2-chat` | nous_portal | $6.3350 | **37.8** | 12.6 | 75 tps | 128k | 🚫 Dominated |
| 176 | **Mistral: Voxtral Small 24B 2507**<br>`mistralai/voxtral-small-24b-2507` | nous_portal | $0.1620 | **37.8** | 32.8 | 75 tps | 32k | 🚫 Dominated |
| 177 | **VoyageAI by MongoDB: voyage-4-lite**<br>`voyageai/voyage-4-lite` | nous_portal | $0.0088 | **37.8** | 37.4 | 75 tps | 32k | 🚫 Dominated |
| 178 | **MissingNo**<br>`stealth/missingno` | nous_portal | $0.0010 | **37.8** | 37.7 | 75 tps | 262k | 💎 **Best Value** |
| 179 | **VoyageAI by MongoDB: voyage-code-4**<br>`voyageai/voyage-code-4` | nous_portal | $0.0528 | **37.8** | 35.9 | 75 tps | 32k | 🚫 Dominated |
| 180 | **Cohere: Command R+ (08-2024)**<br>`cohere/command-r-plus-08-2024` | nous_portal | $5.1000 | **37.8** | 13.4 | 75 tps | 128k | 🚫 Dominated |
| 181 | **OpenAI: GPT-5.2 Pro**<br>`openai/gpt-5.2-pro` | nous_portal_batch | $76.4400 | **37.8** | 7.1 | 150 tps | 400k | 🚫 Dominated |
| 182 | **PrismML: Ternary Bonsai 2 27B**<br>`prism-ml/ternary-bonsai-2-27b` | nous_portal | $0.2375 | **37.8** | 31.1 | 75 tps | 262k | 🚫 Dominated |
| 183 | **ByteDance Seed: Seed-2.0-Code**<br>`bytedance-seed/seed-2.0-code` | nous_portal | $1.4200 | **37.8** | 20.0 | 75 tps | 262k | 🚫 Dominated |
| 184 | **BAAI: bge-m3**<br>`baai/bge-m3` | nous_portal | $0.0044 | **37.8** | 37.6 | 75 tps | 8k | 🚫 Dominated |
| 185 | **OpenAI: Text Embedding 3 Small**<br>`openai/text-embedding-3-small` | nous_portal | $0.0088 | **37.8** | 37.4 | 75 tps | 8k | 🚫 Dominated |
| 186 | **Thenlper: GTE-Large**<br>`thenlper/gte-large` | nous_portal | $0.0044 | **37.8** | 37.6 | 75 tps | 0k | 🚫 Dominated |
| 187 | **DeepSeek: DeepSeek V3**<br>`deepseek/deepseek-chat` | nous_portal | $0.5247 | **37.8** | 26.6 | 75 tps | 163k | 🚫 Dominated |
| 188 | **OpenAI: GPT-5.1-Codex-Max**<br>`openai/gpt-5.1-codex-max` | nous_portal | $4.5250 | **37.8** | 13.9 | 75 tps | 400k | 🚫 Dominated |
| 189 | **VoyageAI by MongoDB: voyage-4-large**<br>`voyageai/voyage-4-large` | nous_portal | $0.0528 | **37.8** | 35.9 | 75 tps | 32k | 🚫 Dominated |
| 190 | **OpenAI: GPT Chat Latest**<br>`openai/gpt-chat-latest` | nous_portal | $14.1000 | **37.8** | 10.2 | 75 tps | 400k | 🚫 Dominated |
| 191 | **Reka Edge**<br>`rekaai/reka-edge` | nous_portal | $0.0840 | **37.8** | 34.9 | 75 tps | 16k | 🚫 Dominated |
| 192 | **Mistral: Codestral Embed 2505**<br>`mistralai/codestral-embed-2505` | nous_portal | $0.0660 | **37.8** | 35.5 | 75 tps | 8k | 🚫 Dominated |
| 193 | **Mistral: Mistral Nemo**<br>`mistralai/mistral-nemo` | nous_portal | $0.0174 | **37.8** | 37.1 | 75 tps | 131k | 🚫 Dominated |
| 194 | **AionLabs: Aion 3.5**<br>`aion-labs/aion-3.5` | nous_portal | $3.7500 | **37.8** | 14.8 | 75 tps | 262k | 🚫 Dominated |
| 195 | **Mistral: Codestral 2508**<br>`mistralai/codestral-2508` | nous_portal_batch | $0.4860 | **37.8** | 27.0 | 150 tps | 256k | 🚫 Dominated |
| 196 | **Perceptron: Perceptron Mk1.5**<br>`perceptron/perceptron-mk1.5` | nous_portal | $0.6660 | **37.8** | 25.0 | 75 tps | 36k | 🚫 Dominated |
| 197 | **OpenAI: GPT-3.5 Turbo (older v0613)**<br>`openai/gpt-3.5-turbo-0613` | nous_portal | $1.2400 | **37.8** | 20.9 | 75 tps | 4k | 🚫 Dominated |
| 198 | **VoyageAI by MongoDB: voyage-4**<br>`voyageai/voyage-4` | nous_portal | $0.0264 | **37.8** | 36.8 | 75 tps | 32k | 🚫 Dominated |
| 199 | **LiquidAI: LFM2.5-24B-A2B**<br>`liquid/lfm-2.5-24b-a2b` | nous_portal | $0.3050 | **37.8** | 29.8 | 75 tps | 128k | 🚫 Dominated |
| 200 | **Mistral: Mistral Small 3.1 24B**<br>`mistralai/mistral-small-3.1-24b-instruct` | nous_portal | $0.3764 | **37.8** | 28.6 | 75 tps | 128k | 🚫 Dominated |
| 201 | **Relace: Relace Search**<br>`relace/relace-search` | nous_portal | $1.6400 | **37.8** | 19.2 | 75 tps | 256k | 🚫 Dominated |
| 202 | **AionLabs: Aion-3.0**<br>`aion-labs/aion-3.0` | nous_portal | $3.7500 | **37.8** | 14.8 | 75 tps | 131k | 🚫 Dominated |
| 203 | **Qwen: Qwen3 Embedding 4B**<br>`qwen/qwen3-embedding-4b` | nous_portal | $0.0088 | **37.8** | 37.4 | 75 tps | 32k | 🚫 Dominated |
| 204 | **OpenAI: GPT-4o**<br>`openai/gpt-4o` | nous_portal_batch | $5.1000 | **37.8** | 13.4 | 150 tps | 128k | 🚫 Dominated |
| 205 | **Thenlper: GTE-Base**<br>`thenlper/gte-base` | nous_portal | $0.0022 | **37.8** | 37.7 | 75 tps | 0k | 🚫 Dominated |
| 206 | **Mistral: Mistral Embed 2312**<br>`mistralai/mistral-embed-2312` | nous_portal | $0.0440 | **37.8** | 36.2 | 75 tps | 8k | 🚫 Dominated |
| 207 | **OpenAI: GPT Audio**<br>`openai/gpt-audio` | nous_portal | $5.1000 | **37.8** | 13.4 | 75 tps | 128k | 🚫 Dominated |
| 208 | **Cohere: Command R (08-2024)**<br>`cohere/command-r-08-2024` | nous_portal | $0.3060 | **37.8** | 29.8 | 75 tps | 128k | 🚫 Dominated |
| 209 | **Google: Gemini 3.6 Flash**<br>`google/gemini-3.6-flash` | nous_portal_batch | $1.8150 | **37.0** | 18.2 | 150 tps | 1048k | 🚫 Dominated |
| 210 | **MiniMax: MiniMax M3**<br>`minimax/minimax-m3` | nous_portal | $0.4860 | **35.3** | 25.3 | 130 tps | 1048k | 🚫 Dominated |
| 211 | **Meta: Muse Spark 1.1**<br>`meta/muse-spark-1.1` | nous_portal | $2.2300 | **34.9** | 16.1 | 75 tps | 1048k | 🚫 Dominated |
| 212 | **inclusionAI: Ling 3.0 Flash VL**<br>`inclusionai/ling-3.0-flash-vl` | nous_portal | $0.0984 | **34.4** | 31.4 | 130 tps | 131k | 🚫 Dominated |
| 213 | **Google: Gemini 3.5 Flash**<br>`google/gemini-3.5-flash` | nous_portal_batch | $4.2300 | **33.6** | 12.7 | 150 tps | 1048k | 🚫 Dominated |
| 214 | **Upstage: Solar Pro 4**<br>`upstage/solar-pro4` | nous_portal | $0.1836 | **33.1** | 28.3 | 90 tps | 524k | 🚫 Dominated |
| 215 | **DeepSeek: DeepSeek V4 Pro 0423**<br>`deepseek/deepseek-v4-pro` | nous_portal | $1.1480 | **32.9** | 18.7 | 75 tps | 1048k | 🚫 Dominated |
| 216 | **OpenAI: GPT-5.3-Codex**<br>`openai/gpt-5.3-codex` | nous_portal | $6.3350 | **32.5** | 10.9 | 75 tps | 400k | 🚫 Dominated |
| 217 | **SpaceXAI: Grok Build 0.1**<br>`x-ai/grok-build-0.1` | nous_portal | $1.2400 | **32.1** | 17.7 | 75 tps | 256k | 🚫 Dominated |
| 218 | **Claude Opus 4.6**<br>`anthropic/claude-opus-4.6` | nous_portal_batch | $12.1000 | **31.9** | 8.9 | 150 tps | 1000k | 🚫 Dominated |
| 219 | **DeepSeek: DeepSeek V4 Flash 0423**<br>`deepseek/deepseek-v4-flash` | nous_portal | $0.4797 | **31.4** | 22.6 | 130 tps | 1048k | 🚫 Dominated |
| 220 | **Qwen: Qwen3.7 Max**<br>`qwen/qwen3.7-max` | nous_portal | $2.4190 | **31.2** | 14.0 | 75 tps | 1000k | 🚫 Dominated |
| 221 | **Tencent: Hy3 preview**<br>`tencent/hy3-preview` | nous_portal | $0.3240 | **31.0** | 24.2 | 75 tps | 262k | 🚫 Dominated |
| 222 | **OpenAI: GPT-5.2**<br>`openai/gpt-5.2` | nous_portal_batch | $6.3350 | **30.4** | 10.2 | 150 tps | 400k | 🚫 Dominated |
| 223 | **Z.ai: GLM 5.1**<br>`z-ai/glm-5.1` | nous_portal | $1.6367 | **30.3** | 15.4 | 75 tps | 204k | 🚫 Dominated |
| 224 | **OpenAI: GPT-5.1**<br>`openai/gpt-5.1` | nous_portal_batch | $4.5260 | **29.6** | 10.9 | 150 tps | 400k | 🚫 Dominated |
| 225 | **Thinking Machines: Inkling Small**<br>`thinkingmachines/inkling-small` | openrouter_market | $0.6800 | **29.4** | 19.3 | 70 tps | 524k | 🚫 Dominated |
| 226 | **Anthropic: Claude Opus 4.5**<br>`anthropic/claude-opus-4.5` | nous_portal_batch | $12.1000 | **29.1** | 8.1 | 150 tps | 200k | 🚫 Dominated |
| 227 | **Xiaomi: MiMo-V2.5-Pro**<br>`xiaomi/mimo-v2.5-pro` | nous_portal | $0.3660 | **29.1** | 22.2 | 75 tps | 1050k | 🚫 Dominated |
| 228 | **MoonshotAI: Kimi K2.7 Code**<br>`moonshotai/kimi-k2.7-code` | nous_portal | $1.6445 | **29.0** | 14.7 | 75 tps | 262k | 🚫 Dominated |
| 229 | **MoonshotAI: Kimi K2.6**<br>`moonshotai/kimi-k2.6` | nous_portal | $1.1762 | **28.8** | 16.2 | 75 tps | 262k | 🚫 Dominated |
| 230 | **OpenAI: GPT-5.2-Codex**<br>`openai/gpt-5.2-codex` | nous_portal | $6.3350 | **28.5** | 9.5 | 75 tps | 400k | 🚫 Dominated |
| 231 | **Thinking Machines: Inkling**<br>`thinkingmachines/inkling` | openrouter_market | $2.0320 | **28.4** | 13.5 | 70 tps | 524k | 🚫 Dominated |
| 232 | **MoonshotAI: Kimi K2.5**<br>`moonshotai/kimi-k2.5` | nous_portal | $1.2360 | **28.2** | 15.6 | 75 tps | 262k | 🚫 Dominated |
| 233 | **Z.ai: GLM 5 Turbo**<br>`z-ai/glm-5-turbo` | nous_portal | $2.1280 | **26.6** | 12.4 | 75 tps | 202k | 🚫 Dominated |
| 234 | **DeepSeek: DeepSeek V3.2**<br>`deepseek/deepseek-v3.2` | nous_portal | $0.2117 | **26.0** | 21.8 | 75 tps | 163k | 🚫 Dominated |
| 235 | **OpenAI: GPT-5**<br>`openai/gpt-5` | nous_portal_batch | $4.5250 | **26.0** | 9.6 | 150 tps | 400k | 🚫 Dominated |
| 236 | **NVIDIA: Nemotron 3 Ultra**<br>`nvidia/nemotron-3-ultra-550b-a55b` | openrouter_market | $1.1000 | **25.9** | 14.9 | 70 tps | 262k | 🚫 Dominated |
| 237 | **SpaceXAI: Grok 4.20**<br>`x-ai/grok-4.20` | nous_portal | $1.5400 | **25.7** | 13.3 | 75 tps | 2000k | 🚫 Dominated |
| 238 | **inclusionAI: Ling 3.0 Flash**<br>`inclusionai/ling-3.0-flash` | nous_portal | $0.0344 | **25.6** | 24.7 | 130 tps | 262k | 🚫 Dominated |
| 239 | **Qwen: Qwen3.6 27B**<br>`qwen/qwen3.6-27b` | nous_portal | $0.9050 | **25.5** | 15.5 | 75 tps | 262k | 🚫 Dominated |
| 240 | **OpenAI: GPT-5.4 Mini**<br>`openai/gpt-5.4-mini` | nous_portal_batch | $2.1150 | **25.5** | 12.0 | 150 tps | 400k | 🚫 Dominated |
| 241 | **Qwen: Qwen3.7 Plus**<br>`qwen/qwen3.7-plus` | nous_portal | $0.6528 | **25.2** | 16.8 | 75 tps | 1000k | 🚫 Dominated |
| 242 | **Upstage: Solar Mini 4**<br>`upstage/solar-mini4` | nous_portal | $0.1010 | **24.1** | 22.0 | 130 tps | 524k | 🚫 Dominated |
| 243 | **OpenAI: GPT-5.4 Nano**<br>`openai/gpt-5.4-nano` | nous_portal_batch | $0.5840 | **24.0** | 16.5 | 150 tps | 400k | 🚫 Dominated |
| 244 | **Xiaomi: MiMo-V2.5**<br>`xiaomi/mimo-v2.5` | nous_portal | $0.1433 | **24.0** | 21.2 | 75 tps | 1050k | 🚫 Dominated |
| 245 | **OpenAI: GPT-5.1-Codex**<br>`openai/gpt-5.1-codex` | nous_portal | $4.5260 | **23.7** | 8.7 | 75 tps | 400k | 🚫 Dominated |
| 246 | **StepFun: Step 3.7 Flash**<br>`stepfun/step-3.7-flash` | nous_portal | $0.5480 | **23.5** | 16.4 | 130 tps | 262k | 🚫 Dominated |
| 247 | **Anthropic: Claude Sonnet 4.5**<br>`anthropic/claude-sonnet-4.5` | nous_portal_batch | $7.2600 | **23.1** | 7.4 | 150 tps | 1000k | 🚫 Dominated |
| 248 | **Qwen: Qwen3.5-27B**<br>`qwen/qwen3.5-27b` | nous_portal | $0.7098 | **22.9** | 14.9 | 75 tps | 262k | 🚫 Dominated |
| 249 | **Qwen: Qwen3.5-122B-A10B**<br>`qwen/qwen3.5-122b-a10b` | nous_portal | $0.9464 | **22.8** | 13.7 | 75 tps | 262k | 🚫 Dominated |
| 250 | **MiniMax: MiniMax M2.5**<br>`minimax/minimax-m2.5` | nous_portal | $0.4940 | **22.8** | 16.3 | 130 tps | 204k | 🚫 Dominated |
| 251 | **Anthropic: Claude Opus 4.1**<br>`anthropic/claude-opus-4.1` | nous_portal_batch | $36.3000 | **22.8** | 4.9 | 150 tps | 200k | 🚫 Dominated |
| 252 | **MiniMax: MiniMax M2.7**<br>`minimax/minimax-m2.7` | nous_portal | $0.4284 | **22.8** | 16.8 | 130 tps | 204k | 🚫 Dominated |
| 253 | **Anthropic: Claude Sonnet 4**<br>`anthropic/claude-sonnet-4` | nous_portal | $7.2600 | **22.6** | 7.3 | 75 tps | 200k | 🚫 Dominated |
| 254 | **MoonshotAI: Kimi K2 Thinking**<br>`moonshotai/kimi-k2-thinking` | nous_portal | $1.2640 | **22.0** | 12.1 | 75 tps | 262k | 🚫 Dominated |
| 255 | **OpenAI: o3 Pro**<br>`openai/o3-pro` | nous_portal | $40.8000 | **21.9** | 4.6 | 75 tps | 200k | 🚫 Dominated |
| 256 | **Google: Gemini 3.5 Flash Lite**<br>`google/gemini-3.5-flash-lite` | nous_portal_batch | $1.1260 | **21.3** | 12.1 | 150 tps | 1048k | 🚫 Dominated |
| 257 | **MiniMax: MiniMax M2.1**<br>`minimax/minimax-m2.1` | nous_portal | $0.6060 | **20.9** | 14.2 | 130 tps | 204k | 🚫 Dominated |
| 258 | **SpaceXAI: Grok 4.3**<br>`x-ai/grok-4.3` | nous_portal_batch | $1.5400 | **20.8** | 10.8 | 150 tps | 1000k | 🚫 Dominated |
| 259 | **OpenAI: GPT-5 Mini**<br>`openai/gpt-5-mini` | nous_portal_batch | $0.9060 | **20.6** | 12.5 | 150 tps | 400k | 🚫 Dominated |
| 260 | **OpenAI: GPT-5.1-Codex-Mini**<br>`openai/gpt-5.1-codex-mini` | nous_portal | $0.9060 | **20.4** | 12.4 | 130 tps | 400k | 🚫 Dominated |
| 261 | **Google: Gemini 3.1 Pro Preview**<br>`google/gemini-3.1-pro-preview` | nous_portal_batch | $5.6400 | **20.3** | 7.0 | 150 tps | 1048k | 🚫 Dominated |
| 262 | **Meituan: LongCat 2.0**<br>`meituan/longcat-2.0` | nous_portal | $0.6012 | **20.3** | 13.8 | 75 tps | 1048k | 🚫 Dominated |
| 263 | **OpenAI: o3**<br>`openai/o3` | nous_portal_batch | $4.1000 | **20.2** | 7.7 | 150 tps | 200k | 🚫 Dominated |
| 264 | **OpenAI: o1**<br>`openai/o1` | nous_portal | $31.5000 | **20.1** | 4.5 | 75 tps | 200k | 🚫 Dominated |
| 265 | **Qwen: Qwen3.5-35B-A3B**<br>`qwen/qwen3.5-35b-a3b` | nous_portal | $0.3400 | **19.3** | 14.9 | 75 tps | 262k | 🚫 Dominated |
| 266 | **Qwen: Qwen3.6 35B A3B**<br>`qwen/qwen3.6-35b-a3b` | nous_portal | $0.3050 | **18.9** | 14.9 | 75 tps | 262k | 🚫 Dominated |
| 267 | **MiniMax: MiniMax M2**<br>`minimax/minimax-m2` | nous_portal | $0.5202 | **18.6** | 13.1 | 130 tps | 204k | 🚫 Dominated |
| 268 | **StepFun: Step 3.5 Flash**<br>`stepfun/step-3.5-flash` | nous_portal | $0.1640 | **17.0** | 14.8 | 130 tps | 262k | 🚫 Dominated |
| 269 | **OpenAI: o4 Mini**<br>`openai/o4-mini` | nous_portal_batch | $2.2550 | **16.7** | 7.7 | 150 tps | 200k | 🚫 Dominated |
| 270 | **Meta: Muse Glimmer 30B**<br>`meta/muse-glimmer-30b` | nous_portal | $0.6080 | **16.6** | 11.3 | 75 tps | 131k | 🚫 Dominated |
| 271 | **DeepSeek: DeepSeek V3.2 Exp**<br>`deepseek/deepseek-v3.2-exp` | nous_portal | $0.2828 | **16.6** | 13.3 | 75 tps | 163k | 🚫 Dominated |
| 272 | **Google: Gemini 2.5 Flash**<br>`google/gemini-2.5-flash` | nous_portal_batch | $1.1260 | **15.5** | 8.8 | 150 tps | 1048k | 🚫 Dominated |
| 273 | **Qwen: Qwen3.5-9B**<br>`qwen/qwen3.5-9b` | nous_portal | $0.0920 | **15.3** | 14.1 | 75 tps | 262k | 🚫 Dominated |
| 274 | **Amazon: Nova 2 Lite**<br>`amazon/nova-2-lite-v1` | nous_portal | $1.1320 | **15.3** | 8.7 | 75 tps | 1000k | 🚫 Dominated |
| 275 | **MoonshotAI: Kimi K2 0905**<br>`moonshotai/kimi-k2-0905` | nous_portal | $1.2640 | **15.3** | 8.4 | 75 tps | 262k | 🚫 Dominated |
| 276 | **Claude Haiku 4.5**<br>`anthropic/claude-haiku-4.5` | nous_portal_batch | $2.4200 | **15.2** | 6.8 | 150 tps | 409k | 🚫 Dominated |
| 277 | **Mistral: Mistral Medium 3.5**<br>`mistralai/mistral-medium-3-5` | nous_portal_batch | $3.6600 | **15.0** | 5.9 | 150 tps | 262k | 🚫 Dominated |
| 278 | **Z.ai: GLM 4.7 Flash**<br>`z-ai/glm-4.7-flash` | nous_portal | $0.1860 | **14.9** | 12.7 | 130 tps | 200k | 🚫 Dominated |
| 279 | **DeepSeek: DeepSeek V3.1 Terminus**<br>`deepseek/deepseek-v3.1-terminus` | nous_portal | $0.5188 | **14.1** | 10.0 | 75 tps | 163k | 🚫 Dominated |
| 280 | **DeepSeek: DeepSeek V3.1**<br>`deepseek/deepseek-chat-v3.1` | nous_portal | $0.5060 | **13.7** | 9.7 | 75 tps | 163k | 🚫 Dominated |
| 281 | **DeepSeek: R1 0528**<br>`deepseek/deepseek-r1-0528` | nous_portal | $1.1300 | **13.1** | 7.5 | 75 tps | 163k | 🚫 Dominated |
| 282 | **OpenAI: GPT-5 Nano**<br>`openai/gpt-5-nano` | nous_portal_batch | $0.1820 | **13.0** | 11.1 | 150 tps | 400k | 🚫 Dominated |
| 283 | **MoonshotAI: Kimi K2 0711**<br>`moonshotai/kimi-k2` | nous_portal | $1.1708 | **12.7** | 7.2 | 75 tps | 131k | 🚫 Dominated |
| 284 | **OpenAI: GPT-4.1**<br>`openai/gpt-4.1` | nous_portal_batch | $4.1000 | **12.7** | 4.8 | 150 tps | 1047k | 🚫 Dominated |
| 285 | **OpenAI o3-mini**<br>`o3-mini` | nous_portal | $1.0200 | **12.5** | 7.3 | 130 tps | 200k | 🚫 Dominated |
| 286 | **OpenAI o3-mini**<br>`o3-mini` | nous_portal | $1.0200 | **12.5** | 7.3 | 130 tps | 200k | 🚫 Dominated |
| 287 | **OpenAI: o3 Mini**<br>`openai/o3-mini` | nous_portal_batch | $2.3100 | **12.5** | 5.7 | 150 tps | 200k | 🚫 Dominated |
| 288 | **Qwen: Qwen3 Next 80B A3B Thinking**<br>`qwen/qwen3-next-80b-a3b-thinking` | nous_portal | $0.5460 | **12.4** | 8.7 | 75 tps | 262k | 🚫 Dominated |
| 289 | **Inception: Mercury 2.5**<br>`inception/mercury-2.5` | nous_portal | $0.0768 | **12.3** | 11.5 | 75 tps | 260k | 🚫 Dominated |
| 290 | **OpenAI: GPT-4.1 Mini**<br>`openai/gpt-4.1-mini` | nous_portal_batch | $0.8200 | **12.2** | 7.6 | 150 tps | 1047k | 🚫 Dominated |
| 291 | **Google: Gemma 4 31B**<br>`google/gemma-4-31b-it` | openrouter_market | $0.1820 | **12.0** | 10.3 | 70 tps | 262k | 🚫 Dominated |
| 292 | **Qwen: Qwen3 235B A22B Instruct 2507**<br>`qwen/qwen3-235b-a22b-2507` | nous_portal | $0.1785 | **12.0** | 10.3 | 75 tps | 262k | 🚫 Dominated |
| 293 | **Qwen: Qwen3 Coder 480B A35B**<br>`qwen/qwen3-coder` | nous_portal | $0.8168 | **11.9** | 7.5 | 90 tps | 262k | 🚫 Dominated |
| 294 | **MiniMax: MiniMax M1**<br>`minimax/minimax-m1` | nous_portal | $1.0560 | **11.7** | 6.8 | 130 tps | 1000k | 🚫 Dominated |
| 295 | **Z.ai: GLM 4.6V**<br>`z-ai/glm-4.6v` | nous_portal | $0.4910 | **11.2** | 8.0 | 75 tps | 131k | 🚫 Dominated |
| 296 | **Z.ai: GLM 4.5 Air**<br>`z-ai/glm-4.5-air` | nous_portal | $0.3970 | **11.1** | 8.3 | 75 tps | 131k | 🚫 Dominated |
| 297 | **OpenAI: GPT-4o (2024-05-13)**<br>`openai/gpt-4o-2024-05-13` | nous_portal | $8.2000 | **10.7** | 3.3 | 75 tps | 128k | 🚫 Dominated |
| 298 | **Google: Gemini 2.5 Flash Lite**<br>`google/gemini-2.5-flash-lite` | nous_portal_batch | $0.2020 | **10.4** | 8.8 | 150 tps | 1048k | 🚫 Dominated |
| 299 | **Google: Gemma 4 26B A4B **<br>`google/gemma-4-26b-a4b-it` | openrouter_market | $0.1090 | **10.2** | 9.2 | 70 tps | 262k | 🚫 Dominated |
| 300 | **OpenAI: gpt-oss-20b**<br>`openai/gpt-oss-20b` | nous_portal | $0.0450 | **10.0** | 9.6 | 75 tps | 131k | 🚫 Dominated |
| 301 | **Qwen: Qwen3 VL 235B A22B Instruct**<br>`qwen/qwen3-vl-235b-a22b-instruct` | nous_portal | $0.4540 | **9.9** | 7.2 | 75 tps | 262k | 🚫 Dominated |
| 302 | **OpenAI: GPT-4 Turbo**<br>`openai/gpt-4-turbo` | nous_portal_batch | $16.4000 | **9.9** | 2.6 | 150 tps | 128k | 🚫 Dominated |
| 303 | **Qwen: Qwen3 Coder 30B A3B Instruct**<br>`qwen/qwen3-coder-30b-a3b-instruct` | nous_portal | $0.1428 | **9.6** | 8.5 | 90 tps | 262k | 🚫 Dominated |
| 304 | **Qwen: Qwen3 Next 80B A3B Instruct**<br>`qwen/qwen3-next-80b-a3b-instruct` | nous_portal | $0.4796 | **9.6** | 6.9 | 75 tps | 262k | 🚫 Dominated |
| 305 | **Qwen: Qwen3 VL 30B A3B Thinking**<br>`qwen/qwen3-vl-30b-a3b-thinking` | nous_portal | $0.5276 | **9.5** | 6.7 | 75 tps | 262k | 🚫 Dominated |
| 306 | **Amazon: Nova Premier 1.0**<br>`amazon/nova-premier-v1` | nous_portal | $6.1250 | **9.2** | 3.1 | 75 tps | 1000k | 🚫 Dominated |
| 307 | **OpenAI: gpt-oss-120b**<br>`openai/gpt-oss-120b` | nous_portal_batch | $0.0780 | **9.0** | 8.4 | 150 tps | 131k | 🚫 Dominated |
| 308 | **Mistral: Mistral Medium 3**<br>`mistralai/mistral-medium-3` | nous_portal | $0.9680 | **9.0** | 5.4 | 75 tps | 131k | 🚫 Dominated |
| 309 | **NVIDIA: Nemotron 3 Super**<br>`nvidia/nemotron-3-super-120b-a12b` | openrouter_market | $0.2152 | **8.9** | 7.4 | 70 tps | 262k | 🚫 Dominated |
| 310 | **Llama 3.3 70B Instruct**<br>`llama-3.3-70b` | nous_portal | $1.0200 | **8.5** | 5.0 | 75 tps | 128k | 🚫 Dominated |
| 311 | **Llama 3.3 70B Instruct**<br>`llama-3.3-70b` | nous_portal | $1.0200 | **8.5** | 5.0 | 75 tps | 128k | 🚫 Dominated |
| 312 | **Meta: Llama 3.3 70B Instruct**<br>`meta-llama/llama-3.3-70b-instruct` | nous_portal | $0.1720 | **8.5** | 7.4 | 75 tps | 131k | 🚫 Dominated |
| 313 | **OpenAI: GPT-4.1 Nano**<br>`openai/gpt-4.1-nano` | nous_portal_batch | $0.2060 | **8.5** | 7.1 | 150 tps | 1047k | 🚫 Dominated |
| 314 | **OpenAI: GPT-4o (2024-11-20)**<br>`openai/gpt-4o-2024-11-20` | nous_portal | $5.2500 | **8.4** | 3.0 | 75 tps | 128k | 🚫 Dominated |
| 315 | **Google: Gemini 3.1 Flash Lite Preview**<br>`google/gemini-3.1-flash-lite-preview` | nous_portal | $0.7050 | **8.2** | 5.4 | 130 tps | 1048k | 🚫 Dominated |
| 316 | **NVIDIA: Nemotron 3.5 Lightning**<br>`nvidia/nemotron-3.5-lightning` | openrouter_market | $0.0605 | **8.2** | 7.7 | 70 tps | 262k | 🚫 Dominated |
| 317 | **Mistral: Devstral 2 2512**<br>`mistralai/devstral-2512` | nous_portal | $0.9680 | **8.1** | 4.8 | 75 tps | 262k | 🚫 Dominated |
| 318 | **OpenAI: GPT-4**<br>`openai/gpt-4` | nous_portal | $37.2000 | **8.0** | 1.7 | 75 tps | 8k | 🚫 Dominated |
| 319 | **Qwen: Qwen3 Coder Next**<br>`qwen/qwen3-coder-next` | nous_portal | $0.3820 | **8.0** | 6.0 | 90 tps | 262k | 🚫 Dominated |
| 320 | **Google: Gemini 2.5 Pro**<br>`google/gemini-2.5-pro` | nous_portal_batch | $4.5250 | **7.9** | 2.9 | 150 tps | 1048k | 🚫 Dominated |
| 321 | **Qwen: Qwen3 VL 30B A3B Instruct**<br>`qwen/qwen3-vl-30b-a3b-instruct` | nous_portal | $0.3060 | **7.9** | 6.2 | 75 tps | 262k | 🚫 Dominated |
| 322 | **OpenAI: GPT-4o (2024-08-06)**<br>`openai/gpt-4o-2024-08-06` | nous_portal | $5.2500 | **7.7** | 2.7 | 75 tps | 128k | 🚫 Dominated |
| 323 | **Qwen2.5 72B Instruct**<br>`qwen/qwen-2.5-72b-instruct` | nous_portal | $0.3184 | **7.7** | 6.0 | 75 tps | 32k | 🚫 Dominated |
| 324 | **GPT-4o Mini**<br>`gpt-4o-mini` | openrouter | $1.0200 | **7.6** | 4.5 | 120 tps | 128k | 🚫 Dominated |
| 325 | **GPT-4o Mini**<br>`gpt-4o-mini` | openrouter | $1.0200 | **7.6** | 4.5 | 120 tps | 128k | 🚫 Dominated |
| 326 | **OpenAI: GPT-4o-mini**<br>`openai/gpt-4o-mini` | nous_portal_batch | $0.3150 | **7.6** | 6.0 | 150 tps | 128k | 🚫 Dominated |
| 327 | **Qwen: Qwen3 30B A3B**<br>`qwen/qwen3-30b-a3b` | nous_portal | $0.2528 | **7.6** | 6.2 | 75 tps | 40k | 🚫 Dominated |
| 328 | **Z.ai: GLM 4.5V**<br>`z-ai/glm-4.5v` | nous_portal | $0.9820 | **7.6** | 4.5 | 75 tps | 65k | 🚫 Dominated |
| 329 | **Mistral Large 2407**<br>`mistralai/mistral-large-2407` | nous_portal | $3.2400 | **7.6** | 3.1 | 75 tps | 131k | 🚫 Dominated |
| 330 | **Inception: Mercury 2**<br>`inception/mercury-2` | nous_portal | $0.4050 | **7.5** | 5.6 | 75 tps | 128k | 🚫 Dominated |
| 331 | **Qwen: Qwen3 30B A3B Instruct 2507**<br>`qwen/qwen3-30b-a3b-instruct-2507` | nous_portal | $0.1596 | **7.5** | 6.5 | 75 tps | 262k | 🚫 Dominated |
| 332 | **Hermes 3 (Llama 3.1 405B)**<br>`hermes-3-llama-3.1-405b` | nous_portal | $1.0200 | **7.4** | 4.3 | 40 tps | 128k | 🚫 Dominated |
| 333 | **Hermes 3 (Llama 3.1 405B)**<br>`hermes-3-llama-3.1-405b` | nous_portal | $1.0200 | **7.4** | 4.3 | 40 tps | 128k | 🚫 Dominated |
| 334 | **Qwen: Qwen3 VL 8B Instruct**<br>`qwen/qwen3-vl-8b-instruct` | nous_portal | $0.4240 | **7.3** | 5.4 | 75 tps | 262k | 🚫 Dominated |
| 335 | **Amazon: Nova Pro 1.0**<br>`amazon/nova-pro-v1` | nous_portal | $1.6320 | **7.0** | 3.6 | 75 tps | 300k | 🚫 Dominated |
| 336 | **Qwen 2.5 Coder 32B**<br>`qwen-2.5-coder-32b` | nous_portal | $1.0200 | **6.7** | 3.9 | 105 tps | 131k | 🚫 Dominated |
| 337 | **Hermes 3 (Llama 3.1 70B)**<br>`hermes-3-llama-3.1-70b` | nous_portal | $1.0200 | **6.7** | 3.9 | 78 tps | 128k | 🚫 Dominated |
| 338 | **Hermes 3 (Llama 3.1 70B)**<br>`hermes-3-llama-3.1-70b` | nous_portal | $1.0200 | **6.7** | 3.9 | 78 tps | 128k | 🚫 Dominated |
| 339 | **Qwen 2.5 Coder 32B**<br>`qwen-2.5-coder-32b` | nous_portal | $1.0200 | **6.7** | 3.9 | 105 tps | 131k | 🚫 Dominated |
| 340 | **Amazon: Nova Lite 1.0**<br>`amazon/nova-lite-v1` | nous_portal | $0.1224 | **6.7** | 6.0 | 75 tps | 300k | 🚫 Dominated |
| 341 | **Meta: Llama 3.1 8B Instruct**<br>`meta-llama/llama-3.1-8b-instruct` | nous_portal | $0.0248 | **6.6** | 6.4 | 75 tps | 131k | 🚫 Dominated |
| 342 | **Meta: Llama 3.1 70B Instruct**<br>`meta-llama/llama-3.1-70b-instruct` | nous_portal | $0.3360 | **6.6** | 5.1 | 75 tps | 131k | 🚫 Dominated |
| 343 | **OpenAI: GPT-3.5 Turbo**<br>`openai/gpt-3.5-turbo` | nous_portal_batch | $0.8200 | **6.5** | 4.1 | 150 tps | 16k | 🚫 Dominated |
| 344 | **Mistral: Saba**<br>`mistralai/mistral-saba` | nous_portal | $0.3240 | **6.5** | 5.1 | 75 tps | 32k | 🚫 Dominated |
| 345 | **Mistral: Mistral Small 4**<br>`mistralai/mistral-small-2603` | nous_portal_batch | $0.3030 | **6.0** | 4.7 | 150 tps | 262k | 🚫 Dominated |
| 346 | **Arcee AI: Trinity Large Thinking**<br>`arcee-ai/trinity-large-thinking` | nous_portal | $0.4320 | **6.0** | 4.4 | 75 tps | 262k | 🚫 Dominated |
| 347 | **Amazon: Nova Micro 1.0**<br>`amazon/nova-micro-v1` | nous_portal | $0.0714 | **5.9** | 5.5 | 75 tps | 128k | 🚫 Dominated |
| 348 | **DeepSeek R1**<br>`deepseek-r1` | nous_portal | $1.0200 | **5.8** | 3.4 | 75 tps | 64k | 🚫 Dominated |
| 349 | **DeepSeek R1**<br>`deepseek-r1` | nous_portal | $1.0200 | **5.8** | 3.4 | 75 tps | 64k | 🚫 Dominated |
| 350 | **DeepSeek: R1**<br>`deepseek/deepseek-r1` | nous_portal | $1.3080 | **5.8** | 3.2 | 75 tps | 64k | 🚫 Dominated |
| 351 | **Mistral Large**<br>`mistralai/mistral-large` | nous_portal | $3.2400 | **5.8** | 2.4 | 75 tps | 128k | 🚫 Dominated |
| 352 | **Mistral: Mixtral 8x22B Instruct**<br>`mistralai/mixtral-8x22b-instruct` | nous_portal | $3.2400 | **5.7** | 2.3 | 75 tps | 65k | 🚫 Dominated |
| 353 | **Mistral: Mistral Medium 3.1**<br>`mistralai/mistral-medium-3.1` | nous_portal_batch | $0.9680 | **5.6** | 3.4 | 150 tps | 131k | 🚫 Dominated |
| 354 | **IBM: Granite 4.2 8B**<br>`ibm-granite/granite-4.2-8b` | nous_portal | $0.1270 | **5.5** | 4.9 | 75 tps | 131k | 🚫 Dominated |
| 355 | **Qwen: Qwen3 235B A22B Thinking 2507**<br>`qwen/qwen3-235b-a22b-thinking-2507` | nous_portal | $1.5980 | **5.5** | 2.8 | 75 tps | 128k | 🚫 Dominated |
| 356 | **DeepSeek: DeepSeek V3 0324**<br>`deepseek/deepseek-chat-v3-0324` | nous_portal | $0.5100 | **4.9** | 3.5 | 75 tps | 163k | 🚫 Dominated |
| 357 | **Mistral: Mistral Large 3 2512**<br>`mistralai/mistral-large-2512` | nous_portal_batch | $0.8100 | **4.8** | 3.0 | 150 tps | 262k | 🚫 Dominated |
| 358 | **Upstage: Solar Pro 3**<br>`upstage/solar-pro-3` | nous_portal | $0.3030 | **4.4** | 3.4 | 90 tps | 131k | 🚫 Dominated |
| 359 | **OpenAI: o3 Mini High**<br>`openai/o3-mini-high` | nous_portal | $2.3100 | **4.0** | 1.8 | 130 tps | 200k | 🚫 Dominated |
| 360 | **Qwen: Qwen3 32B**<br>`qwen/qwen3-32b` | nous_portal | $0.1472 | **3.8** | 3.3 | 75 tps | 131k | 🚫 Dominated |
| 361 | **Mistral: Ministral 3 14B 2512**<br>`mistralai/ministral-14b-2512` | nous_portal | $0.1640 | **3.8** | 3.3 | 130 tps | 262k | 🚫 Dominated |
| 362 | **Meta: Llama 4 Maverick**<br>`meta-llama/llama-4-maverick` | nous_portal | $0.3460 | **3.7** | 2.9 | 75 tps | 1048k | 🚫 Dominated |
| 363 | **NVIDIA: Nemotron 3 Nano 30B A3B**<br>`nvidia/nemotron-3-nano-30b-a3b` | nous_portal | $0.1020 | **3.7** | 3.4 | 75 tps | 262k | 🚫 Dominated |
| 364 | **Qwen: Qwen3 14B**<br>`qwen/qwen3-14b` | nous_portal | $0.1320 | **3.5** | 3.1 | 75 tps | 40k | 🚫 Dominated |
| 365 | **Mistral: Ministral 3 8B 2512**<br>`mistralai/ministral-8b-2512` | nous_portal_batch | $0.1230 | **2.4** | 2.2 | 150 tps | 262k | 🚫 Dominated |
| 366 | **Google: Gemma 3 27B**<br>`google/gemma-3-27b-it` | nous_portal | $0.1460 | **2.1** | 1.8 | 75 tps | 131k | 🚫 Dominated |
| 367 | **Meta: Llama 4 Scout**<br>`meta-llama/llama-4-scout` | nous_portal | $0.1640 | **2.0** | 1.8 | 75 tps | 1310k | 🚫 Dominated |
| 368 | **Mistral: Ministral 3 3B 2512**<br>`mistralai/ministral-3b-2512` | nous_portal | $0.0820 | **1.6** | 1.5 | 130 tps | 131k | 🚫 Dominated |
| 369 | **Google: Gemma 3 12B**<br>`google/gemma-3-12b-it` | nous_portal | $0.0820 | **1.2** | 1.1 | 75 tps | 131k | 🚫 Dominated |

#### 🔗 Cadeia de Roteamento Preditivo para Subagentes (`reasoning`)
| Nível de Escalação | Modelo Designado | Provedor | Custo Blended | Condição de Acionamento |
| :--- | :--- | :--- | :---: | :--- |
| **1. Primário (Custo-Benefício)** | `Xiaomi: MiMo-V2.6-Flash` | nous_portal | $0.1620 | Modelo padrão para subtarefas com SLA normal e orçamento otimizado. |
| **2. Escalação Máxima (Líder)** | `Xiaomi: MiMo-V2.6-Pro` | nous_portal | $0.5207 | Acionado se complexidade/entropia for crítica ou falha de teste anterior. |
| **3. Fallback de Baixo Custo** | `Qwen 2.5 Coder 32B` | nous_portal | $0.3300 | Standby caso o provedor principal sofra throttling ou indisponibilidade. |

---

### ⚡ 2.4 Perfil: Fast / Triage (Triagem de Alta Cadência & Streaming)
> **Descrição:** Sanitização de contexto, triagem de intenção preliminar, ETL de logs e streaming instantâneo de turnos rápidos.  
> **Métricas Primárias:** Throughput (TPS) & TTFT (ms) &bull; **Fórmula:** `S_fast = 0.50·TPS_norm + 0.30·TTFT_norm + 0.20·Price_inv`

| Rank | Modelo Canônico | Provedor | Custo Blended ($/1M) | Score Composto | ROI Index | Throughput | Janela Ctx | Status Pareto |
| :---: | :--- | :---: | :---: | :---: | :---: | :---: | :---: | :--- |
| 1 | **Google: Gemini 3 Flash Preview**<br>`google/gemini-3-flash-preview` | nous_portal_batch | $0.7050 | **88.6** | 57.7 | 150 tps | 1048k | 🏆 **Quality Leader** |
| 2 | **Xiaomi: MiMo-V2.6-Flash**<br>`xiaomi/mimo-v2.6-flash` | nous_portal | $0.1130 | **88.0** | 79.5 | 130 tps | 1050k | 💎 **Best Value** |
| 3 | **Google: Gemini 3.8 Flash**<br>`google/gemini-3.8-flash` | nous_portal_batch | $0.9825 | **86.0** | 51.1 | 150 tps | 1048k | 🚫 Dominated |
| 4 | **Z.ai: GLM 5.3 Flash**<br>`z-ai/glm-5.3-flash` | nous_portal_batch | $0.1384 | **84.6** | 74.9 | 150 tps | 1048k | 🚫 Dominated |
| 5 | **Google: Gemini 3.7 Flash**<br>`google/gemini-3.7-flash` | nous_portal_batch | $0.9825 | **84.6** | 50.2 | 150 tps | 1048k | 🚫 Dominated |
| 6 | **OpenAI: GPT-5.6 Luna**<br>`openai/gpt-5.6-luna` | nous_portal_batch | $0.2820 | **84.6** | 67.7 | 150 tps | 1050k | 🚫 Dominated |
| 7 | **Google: Gemini 3.6 Flash**<br>`google/gemini-3.6-flash` | nous_portal_batch | $0.9825 | **83.9** | 49.8 | 150 tps | 1048k | 🚫 Dominated |
| 8 | **DeepSeek: DeepSeek V4 Flash**<br>`deepseek/deepseek-v4-flash-0731` | openrouter_market | $0.0714 | **83.9** | 78.5 | 130 tps | 1048k | 💎 **Best Value** |
| 9 | **Google: Gemini 3.1 Pro Preview**<br>`google/gemini-3.1-pro-preview` | nous_portal_batch | $2.8200 | **83.8** | 35.8 | 150 tps | 1048k | 🚫 Dominated |
| 10 | **OpenAI o3-mini**<br>`o3-mini` | nous_portal | $0.6738 | **82.7** | 54.6 | 130 tps | 200k | 🏆 **Quality Leader** |
| 11 | **DeepSeek: DeepSeek V4 Flash Vision Exp**<br>`deepseek/deepseek-v4-flash-vision-exp` | nous_portal | $0.2379 | **82.7** | 68.1 | 130 tps | 1048k | 🚫 Dominated |
| 12 | **MiniMax: MiniMax M3**<br>`minimax/minimax-m3` | nous_portal | $0.2850 | **80.7** | 64.5 | 130 tps | 1048k | 🚫 Dominated |
| 13 | **inclusionAI: Ling 3.0 Flash VL**<br>`inclusionai/ling-3.0-flash-vl` | nous_portal | $0.0672 | **80.3** | 75.3 | 130 tps | 131k | 💎 **Best Value** |
| 14 | **OpenAI: GPT-5.4 Mini**<br>`openai/gpt-5.4-mini` | nous_portal_batch | $1.0575 | **80.0** | 46.5 | 150 tps | 400k | 🚫 Dominated |
| 15 | **Qwen 2.5 Coder 32B**<br>`qwen-2.5-coder-32b` | nous_portal | $0.2250 | **79.3** | 65.9 | 105 tps | 131k | 💎 **Best Value** |
| 16 | **Google: Gemini 4 Argon**<br>`google/gemini-4-argon` | nous_portal_batch | $2.6100 | **78.9** | 34.6 | 150 tps | 1048k | 🚫 Dominated |
| 17 | **MiniMax: MiniMax M2.7**<br>`minimax/minimax-m2.7` | nous_portal | $0.2562 | **78.9** | 64.3 | 130 tps | 204k | 🚫 Dominated |
| 18 | **DeepSeek: DeepSeek V4 Flash 0423**<br>`deepseek/deepseek-v4-flash` | nous_portal | $0.1496 | **78.8** | 69.1 | 130 tps | 1048k | 🚫 Dominated |
| 19 | **GPT-4o Mini**<br>`gpt-4o-mini` | openrouter | $0.0919 | **78.4** | 72.0 | 120 tps | 128k | 💎 **Best Value** |
| 20 | **inclusionAI: Ling 3.0 Flash**<br>`inclusionai/ling-3.0-flash` | nous_portal | $0.0235 | **78.3** | 76.6 | 130 tps | 262k | 💎 **Best Value** |
| 21 | **Google: Gemini 3.5 Flash Lite**<br>`google/gemini-3.5-flash-lite` | nous_portal_batch | $0.4930 | **77.9** | 55.6 | 150 tps | 1048k | 🚫 Dominated |
| 22 | **Gemini 2.0 Flash**<br>`gemini-2.0-flash` | nous_portal | $0.6100 | **76.4** | 51.7 | 130 tps | 1000k | 🏆 **Quality Leader** |
| 23 | **Gemini 2.0 Flash**<br>`gemini-2.0-flash` | nous_portal | $0.6100 | **76.4** | 51.7 | 130 tps | 1000k | 🏆 **Quality Leader** |
| 24 | **AionLabs: Aion 3.5 Mini**<br>`aion-labs/aion-3.5-mini` | nous_portal | $0.7180 | **76.4** | 49.5 | 130 tps | 262k | 🚫 Dominated |
| 25 | **Google: Nano Banana 2.1**<br>`google/gemini-nano-banana-2.1` | nous_portal | $1.9800 | **76.4** | 36.5 | 130 tps | 65k | 🚫 Dominated |
| 26 | **ByteDance Seed: Seed 1.6 Flash**<br>`bytedance-seed/seed-1.6-flash` | nous_portal | $0.0915 | **76.4** | 70.2 | 130 tps | 262k | 🚫 Dominated |
| 27 | **Anthropic: Claude Haiku Latest**<br>`anthropic/claude-haiku-latest` | nous_portal | $0.1310 | **76.4** | 68.0 | 130 tps | 1000k | 🚫 Dominated |
| 28 | **Google: Gemini Embedding 2 Preview**<br>`google/gemini-embedding-2-preview` | nous_portal | $0.1640 | **76.4** | 66.3 | 130 tps | 8k | 🚫 Dominated |
| 29 | **Google: Gemini 3.1 Pro Preview Custom Tools**<br>`google/gemini-3.1-pro-preview-customtools` | nous_portal | $2.8200 | **76.4** | 32.6 | 130 tps | 1048k | 🚫 Dominated |
| 30 | **AionLabs: Aion-3.0-Mini**<br>`aion-labs/aion-3.0-mini` | nous_portal | $0.7180 | **76.4** | 49.5 | 130 tps | 131k | 🚫 Dominated |
| 31 | **Google: Nano Banana Pro (Gemini 3 Pro Image)**<br>`google/gemini-3-pro-image` | nous_portal | $2.8200 | **76.4** | 32.6 | 130 tps | 131k | 🚫 Dominated |
| 32 | **Qwen: Qwen3.8 Flash**<br>`qwen/qwen3.8-flash` | nous_portal | $0.1686 | **76.4** | 66.1 | 130 tps | 1000k | 🚫 Dominated |
| 33 | **DeepSeek: DeepSeek V4 Flash Latest**<br>`deepseek/deepseek-v4-flash-latest` | nous_portal | $0.0702 | **76.4** | 71.5 | 130 tps | 1048k | 🚫 Dominated |
| 34 | **Z.ai: GLM 5.3 FlashX**<br>`z-ai/glm-5.3-flashx` | nous_portal | $0.4300 | **76.4** | 56.2 | 130 tps | 1048k | 🚫 Dominated |
| 35 | **Google: Gemini Embedding 001**<br>`google/gemini-embedding-001` | nous_portal | $0.1230 | **76.4** | 68.4 | 130 tps | 20k | 🚫 Dominated |
| 36 | **Google: Gemini 3.1 Flash Lite**<br>`google/gemini-3.1-flash-lite` | nous_portal_batch | $0.3525 | **76.4** | 58.6 | 150 tps | 1048k | 🚫 Dominated |
| 37 | **OpenAI: GPT Mini Latest**<br>`openai/gpt-mini-latest` | nous_portal | $1.0575 | **76.4** | 44.4 | 130 tps | 400k | 🚫 Dominated |
| 38 | **OpenAI: GPT Audio Mini**<br>`openai/gpt-audio-mini` | nous_portal | $0.7320 | **76.4** | 49.3 | 130 tps | 128k | 🚫 Dominated |
| 39 | **Google: Gemini Embedding 2**<br>`google/gemini-embedding-2` | nous_portal_batch | $0.1640 | **76.4** | 66.3 | 150 tps | 8k | 🚫 Dominated |
| 40 | **OpenAI: GPT-4o-mini (2024-07-18)**<br>`openai/gpt-4o-mini-2024-07-18` | nous_portal | $0.1875 | **76.4** | 65.2 | 130 tps | 128k | 🚫 Dominated |
| 41 | **OpenAI: GPT-5.6 Luna Pro**<br>`openai/gpt-5.6-luna-pro` | nous_portal_batch | $0.2820 | **76.4** | 61.2 | 150 tps | 1050k | 🚫 Dominated |
| 42 | **Sentence Transformers: all-MiniLM-L12-v2**<br>`sentence-transformers/all-minilm-l12-v2` | nous_portal | $0.0041 | **76.4** | 76.0 | 130 tps | 0k | ⚡ **Ultra Low Cost** |
| 43 | **DeepSeek: DeepSeek V4 Flash**<br>`deepseek/deepseek-v4-flash-0731:US` | nous_portal | $0.4004 | **76.4** | 57.1 | 130 tps | 1048k | 🚫 Dominated |
| 44 | **Qwen: Qwen3.5-Flash**<br>`qwen/qwen3.5-flash-02-23` | nous_portal | $0.0793 | **76.4** | 70.9 | 130 tps | 1000k | 🚫 Dominated |
| 45 | **DeepSeek: DeepSeek Flash Latest**<br>`deepseek/deepseek-flash-latest` | openrouter_market | $0.0682 | **76.4** | 71.6 | 130 tps | 1048k | 🚫 Dominated |
| 46 | **Qwen: Qwen3 Coder Flash**<br>`qwen/qwen3-coder-flash` | nous_portal | $0.2574 | **76.4** | 62.1 | 130 tps | 1000k | 🚫 Dominated |
| 47 | **DeepSeek: DeepSeek V4.1 Flash**<br>`deepseek/deepseek-v4.1-flash:US` | nous_portal | $0.4004 | **76.4** | 57.1 | 130 tps | 1048k | 🚫 Dominated |
| 48 | **Google: Gemini Flash Latest**<br>`google/gemini-flash-latest` | nous_portal | $0.9825 | **76.4** | 45.3 | 130 tps | 1048k | 🚫 Dominated |
| 49 | **OpenAI: GPT Luna Latest**<br>`openai/gpt-luna-latest` | nous_portal | $0.1310 | **76.4** | 68.0 | 130 tps | 1050k | 🚫 Dominated |
| 50 | **Z.ai: GLM Flash Latest**<br>`z-ai/glm-flash-latest` | nous_portal | $0.1199 | **76.4** | 68.6 | 130 tps | 1048k | 🚫 Dominated |
| 51 | **Sentence Transformers: paraphrase-MiniLM-L6-v2**<br>`sentence-transformers/paraphrase-minilm-l6-v2` | nous_portal | $0.0041 | **76.4** | 76.0 | 130 tps | 0k | 💎 **Best Value** |
| 52 | **ByteDance Seed: Seed-2.0-Mini**<br>`bytedance-seed/seed-2.0-mini` | nous_portal | $0.1220 | **76.4** | 68.5 | 130 tps | 262k | 🚫 Dominated |
| 53 | **Qwen: Qwen3.6 Flash**<br>`qwen/qwen3.6-flash` | nous_portal | $0.2663 | **76.4** | 61.8 | 130 tps | 1000k | 🚫 Dominated |
| 54 | **Google: Gemini Pro Latest**<br>`google/gemini-pro-latest` | nous_portal | $2.8200 | **76.4** | 32.6 | 130 tps | 1048k | 🚫 Dominated |
| 55 | **Z.ai: GLM 5.3 Flash**<br>`z-ai/glm-5.3-flash:US` | nous_portal | $0.2855 | **76.4** | 61.0 | 130 tps | 1048k | 🚫 Dominated |
| 56 | **Sentence Transformers: all-MiniLM-L6-v2**<br>`sentence-transformers/all-minilm-l6-v2` | nous_portal | $0.0041 | **76.4** | 76.0 | 130 tps | 0k | 💎 **Best Value** |
| 57 | **Qwen: Qwen3.8 Omni Flash**<br>`qwen/qwen3.8-omni-flash` | nous_portal | $0.1686 | **76.4** | 66.1 | 130 tps | 1000k | 🚫 Dominated |
| 58 | **Qwen: Qwen3.7 Flash**<br>`qwen/qwen3.7-flash` | nous_portal | $0.0376 | **76.4** | 73.6 | 130 tps | 1000k | 🚫 Dominated |
| 59 | **OpenAI: GPT-6 Luna Pro**<br>`openai/gpt-6-luna-pro` | nous_portal_batch | $0.1310 | **76.4** | 68.0 | 150 tps | 1050k | 🚫 Dominated |
| 60 | **Google: Gemini 2.5 Pro Preview 06-05**<br>`google/gemini-2.5-pro-preview` | nous_portal | $2.0125 | **76.4** | 36.3 | 130 tps | 1048k | 🚫 Dominated |
| 61 | **OpenAI: o4 Mini High**<br>`openai/o4-mini-high` | nous_portal | $1.3475 | **76.4** | 41.2 | 130 tps | 200k | 🚫 Dominated |
| 62 | **Claude Haiku 4.5**<br>`anthropic/claude-haiku-4.5` | nous_portal_batch | $1.3100 | **76.3** | 41.5 | 150 tps | 409k | 🚫 Dominated |
| 63 | **Anthropic: Claude Haiku 5.5**<br>`anthropic/claude-haiku-5.5` | nous_portal_batch | $0.1310 | **76.2** | 67.8 | 150 tps | 1000k | 🚫 Dominated |
| 64 | **DeepSeek V3**<br>`deepseek-v3` | nous_portal | $0.3410 | **76.0** | 58.7 | 92 tps | 64k | 🚫 Dominated |
| 65 | **inclusionAI: Ling 3.1 Flash**<br>`inclusionai/ling-3.1-flash` | nous_portal | $0.0840 | **75.5** | 69.8 | 130 tps | 262k | 🚫 Dominated |
| 66 | **StepFun: Step 3.7 Flash**<br>`stepfun/step-3.7-flash` | nous_portal | $0.2790 | **75.0** | 60.2 | 130 tps | 262k | 🚫 Dominated |
| 67 | **DeepSeek: DeepSeek V4.1 Flash**<br>`deepseek/deepseek-v4.1-flash` | openrouter_market | $0.1365 | **75.0** | 66.5 | 130 tps | 1048k | 🚫 Dominated |
| 68 | **OpenAI: GPT-6 Luna**<br>`openai/gpt-6-luna` | nous_portal_batch | $0.1310 | **74.6** | 66.4 | 150 tps | 1050k | 🚫 Dominated |
| 69 | **Google: Gemini 3.1 Flash Lite Preview**<br>`google/gemini-3.1-flash-lite-preview` | nous_portal | $0.3525 | **73.6** | 56.5 | 130 tps | 1048k | 🚫 Dominated |
| 70 | **Google: Gemini 3.5 Flash**<br>`google/gemini-3.5-flash` | nous_portal_batch | $2.1150 | **73.2** | 34.3 | 150 tps | 1048k | 🚫 Dominated |
| 71 | **Google: Gemini 2.5 Pro**<br>`google/gemini-2.5-pro` | nous_portal_batch | $2.0125 | **73.1** | 34.8 | 150 tps | 1048k | 🚫 Dominated |
| 72 | **OpenAI: GPT-5.6 Sol**<br>`openai/gpt-5.6-sol` | nous_portal_batch | $2.6200 | **72.3** | 31.6 | 150 tps | 1050k | 🚫 Dominated |
| 73 | **Xiaomi: MiMo-V2.6-Pro**<br>`xiaomi/mimo-v2.6-pro` | nous_portal | $0.4314 | **71.6** | 52.7 | 75 tps | 1050k | 🚫 Dominated |
| 74 | **Claude 3.7 Sonnet**<br>`claude-3-7-sonnet` | nous_portal | $4.2550 | **71.4** | 26.9 | 75 tps | 200k | 🚫 Dominated |
| 75 | **Upstage: Solar Mini 4**<br>`upstage/solar-mini4` | nous_portal | $0.0605 | **70.4** | 66.5 | 130 tps | 524k | 🚫 Dominated |
| 76 | **MiniMax: MiniMax M2.5**<br>`minimax/minimax-m2.5` | nous_portal | $0.3140 | **70.0** | 55.0 | 130 tps | 204k | 🚫 Dominated |
| 77 | **MiniMax: MiniMax M2.1**<br>`minimax/minimax-m2.1` | nous_portal | $0.3630 | **69.4** | 53.0 | 130 tps | 204k | 🚫 Dominated |
| 78 | **OpenAI: GPT-5 Mini**<br>`openai/gpt-5-mini` | nous_portal_batch | $0.4030 | **69.3** | 51.8 | 150 tps | 400k | 🚫 Dominated |
| 79 | **Anthropic: Claude Fable 5.1**<br>`anthropic/claude-fable-5.1` | nous_portal_batch | $13.0250 | **69.3** | 19.0 | 150 tps | 1000k | 🚫 Dominated |
| 80 | **OpenAI: GPT-5.1-Codex-Mini**<br>`openai/gpt-5.1-codex-mini` | nous_portal | $0.4030 | **69.3** | 51.7 | 130 tps | 400k | 🚫 Dominated |
| 81 | **Hermes 3 (Llama 3.1 70B)**<br>`hermes-3-llama-3.1-70b` | nous_portal | $0.3790 | **69.3** | 52.4 | 78 tps | 128k | 🚫 Dominated |
| 82 | **OpenAI: GPT-4.1 Mini**<br>`openai/gpt-4.1-mini` | nous_portal_batch | $0.4900 | **69.2** | 49.5 | 150 tps | 1047k | 🚫 Dominated |
| 83 | **Z.ai: GLM 5**<br>`z-ai/glm-5` | nous_portal | $0.6840 | **69.2** | 45.5 | 75 tps | 204k | 🚫 Dominated |
| 84 | **MiniMax: MiniMax M2**<br>`minimax/minimax-m2` | nous_portal | $0.3111 | **68.7** | 54.1 | 130 tps | 204k | 🚫 Dominated |
| 85 | **Qwen: Qwen3.5 397B A17B**<br>`qwen/qwen3.5-397b-a17b` | nous_portal | $0.5538 | **68.7** | 47.7 | 75 tps | 262k | 🚫 Dominated |
| 86 | **Z.ai: GLM 5V Turbo**<br>`z-ai/glm-5v-turbo` | nous_portal | $1.3840 | **68.7** | 36.8 | 75 tps | 202k | 🚫 Dominated |
| 87 | **Tencent: Hy3**<br>`tencent/hy3` | nous_portal | $0.1617 | **68.7** | 59.7 | 75 tps | 262k | 🚫 Dominated |
| 88 | **Z.ai: GLM 4.5**<br>`z-ai/glm-4.5` | nous_portal | $0.7110 | **68.3** | 44.4 | 75 tps | 131k | 🚫 Dominated |
| 89 | **StepFun: Step 3.5 Flash**<br>`stepfun/step-3.5-flash` | nous_portal | $0.1120 | **68.3** | 61.7 | 130 tps | 262k | 🚫 Dominated |
| 90 | **OpenAI: o4 Mini**<br>`openai/o4-mini` | nous_portal_batch | $1.3475 | **68.2** | 36.8 | 150 tps | 200k | 🚫 Dominated |
| 91 | **OpenAI: o3 Mini High**<br>`openai/o3-mini-high` | nous_portal | $1.3750 | **68.0** | 36.5 | 130 tps | 200k | 🚫 Dominated |
| 92 | **xAI: Grok 4.6**<br>`x-ai/grok-4.6` | nous_portal | $2.2500 | **67.9** | 31.2 | 75 tps | 500k | 🚫 Dominated |
| 93 | **OpenAI: GPT-5.6 Terra**<br>`openai/gpt-5.6-terra` | nous_portal_batch | $2.8200 | **67.9** | 29.0 | 150 tps | 1050k | 🚫 Dominated |
| 94 | **Google: Gemini 2.5 Flash**<br>`google/gemini-2.5-flash` | nous_portal_batch | $0.4930 | **67.8** | 48.4 | 150 tps | 1048k | 🚫 Dominated |
| 95 | **Claude Fable 5**<br>`anthropic/claude-fable-5` | nous_portal_batch | $13.1000 | **67.8** | 18.6 | 150 tps | 1048k | 🚫 Dominated |
| 96 | **MoonshotAI: Kimi K3**<br>`moonshotai/kimi-k3` | nous_portal | $2.0920 | **67.7** | 31.8 | 75 tps | 1048k | 🚫 Dominated |
| 97 | **Qwen: Qwen3.8 Max (0902)**<br>`qwen/qwen3.8-max-0902` | nous_portal | $2.2250 | **67.7** | 31.2 | 75 tps | 1000k | 🚫 Dominated |
| 98 | **Z.ai: GLM 4.7 Flash**<br>`z-ai/glm-4.7-flash` | nous_portal | $0.0890 | **67.6** | 62.3 | 130 tps | 200k | 🚫 Dominated |
| 99 | **Meta: Muse Spark 1.3**<br>`meta/muse-spark-1.3` | nous_portal | $1.4400 | **67.6** | 35.7 | 75 tps | 1048k | 🚫 Dominated |
| 100 | **GPT-4o**<br>`gpt-4o` | nous_portal | $1.5312 | **67.6** | 35.0 | 75 tps | 128k | 🚫 Dominated |
| 101 | **Mistral: Ministral 3 14B 2512**<br>`mistralai/ministral-14b-2512` | nous_portal | $0.1820 | **67.5** | 57.8 | 130 tps | 262k | 🚫 Dominated |
| 102 | **OpenAI: GPT-5.5**<br>`openai/gpt-5.5` | nous_portal_batch | $7.0500 | **67.3** | 21.8 | 150 tps | 1050k | 🚫 Dominated |
| 103 | **Z.ai: GLM 5.3**<br>`z-ai/glm-5.3` | openrouter_market | $1.0309 | **67.3** | 39.4 | 70 tps | 1048k | 🚫 Dominated |
| 104 | **OpenAI o3-mini**<br>`o3-mini` | nous_portal | $0.6100 | **66.9** | 45.3 | 130 tps | 200k | 🚫 Dominated |
| 105 | **OpenAI o3-mini**<br>`o3-mini` | nous_portal | $0.6100 | **66.9** | 45.3 | 130 tps | 200k | 🚫 Dominated |
| 106 | **OpenAI: o3 Mini**<br>`openai/o3-mini` | nous_portal_batch | $1.3750 | **66.9** | 35.9 | 150 tps | 200k | 🚫 Dominated |
| 107 | **MiniMax: MiniMax M1**<br>`minimax/minimax-m1` | nous_portal | $0.5480 | **66.7** | 46.4 | 130 tps | 1000k | 🚫 Dominated |
| 108 | **GPT-4o Mini**<br>`gpt-4o-mini` | openrouter | $0.6100 | **66.6** | 45.1 | 120 tps | 128k | 🚫 Dominated |
| 109 | **GPT-4o Mini**<br>`gpt-4o-mini` | openrouter | $0.6100 | **66.6** | 45.1 | 120 tps | 128k | 🚫 Dominated |
| 110 | **OpenAI: GPT-4o-mini**<br>`openai/gpt-4o-mini` | nous_portal_batch | $0.1875 | **66.6** | 56.8 | 150 tps | 128k | 🚫 Dominated |
| 111 | **xAI: Grok 4.5**<br>`x-ai/grok-4.5` | nous_portal | $2.2300 | **66.6** | 30.6 | 75 tps | 500k | 🚫 Dominated |
| 112 | **Meta: Muse Spark 1.2**<br>`meta/muse-spark-1.2` | nous_portal | $1.4400 | **66.5** | 35.1 | 75 tps | 1048k | 🚫 Dominated |
| 113 | **Qwen: Qwen3.8 2.4T A95B**<br>`qwen/qwen3.8-2.4t-a95b` | nous_portal | $2.2250 | **66.4** | 30.6 | 75 tps | 1048k | 🚫 Dominated |
| 114 | **Claude Sonnet 5**<br>`anthropic/claude-sonnet-5` | nous_portal_batch | $2.6200 | **66.3** | 29.0 | 150 tps | 1000k | 🚫 Dominated |
| 115 | **Google: Gemini 2.5 Flash Lite**<br>`google/gemini-2.5-flash-lite` | nous_portal_batch | $0.1210 | **66.3** | 59.5 | 150 tps | 1048k | 🚫 Dominated |
| 116 | **Meta: Muse Spark 1.1**<br>`meta/muse-spark-1.1` | nous_portal | $1.4400 | **66.2** | 35.0 | 75 tps | 1048k | 🚫 Dominated |
| 117 | **OpenAI: GPT-5.4**<br>`openai/gpt-5.4` | nous_portal_batch | $3.5250 | **66.2** | 26.4 | 150 tps | 1050k | 🚫 Dominated |
| 118 | **Mistral: Ministral 3 8B 2512**<br>`mistralai/ministral-8b-2512` | nous_portal_batch | $0.1365 | **66.1** | 58.6 | 150 tps | 262k | 🚫 Dominated |
| 119 | **Llama 3.3 70B Instruct**<br>`llama-3.3-70b` | nous_portal | $0.5657 | **65.7** | 45.3 | 75 tps | 128k | 🚫 Dominated |
| 120 | **Z.ai: GLM 5.2**<br>`z-ai/glm-5.2` | openrouter_market | $1.0190 | **65.5** | 38.5 | 70 tps | 1048k | 🚫 Dominated |
| 121 | **DeepSeek: DeepSeek V4 Pro 0813**<br>`deepseek/deepseek-v4-pro-0813` | nous_portal | $0.9620 | **65.5** | 39.1 | 75 tps | 1048k | 🚫 Dominated |
| 122 | **Qwen: Qwen3.8 27B**<br>`qwen/qwen3.8-27b` | nous_portal | $0.1566 | **65.3** | 57.0 | 75 tps | 1000k | 🚫 Dominated |
| 123 | **Upstage: Solar Pro 4**<br>`upstage/solar-pro4` | nous_portal | $0.1098 | **64.9** | 58.8 | 90 tps | 524k | 🚫 Dominated |
| 124 | **OpenAI: GPT-6.1 Sol**<br>`openai/gpt-6.1-sol` | nous_portal_batch | $2.6100 | **64.7** | 28.3 | 150 tps | 1050k | 🚫 Dominated |
| 125 | **Qwen: Qwen3.7 Max**<br>`qwen/qwen3.7-max` | nous_portal | $1.6520 | **64.6** | 32.7 | 75 tps | 1000k | 🚫 Dominated |
| 126 | **Mistral: Ministral 3 3B 2512**<br>`mistralai/ministral-3b-2512` | nous_portal | $0.0910 | **64.6** | 59.4 | 130 tps | 131k | 🚫 Dominated |
| 127 | **OpenAI: GPT Sol Latest**<br>`openai/gpt-sol-latest` | nous_portal | $2.6100 | **64.1** | 28.1 | 90 tps | 1050k | 🚫 Dominated |
| 128 | **OpenAI: GPT-5.6 Sol Pro**<br>`openai/gpt-5.6-sol-pro` | nous_portal_batch | $2.6200 | **64.1** | 28.0 | 150 tps | 1050k | 🚫 Dominated |
| 129 | **OpenAI: GPT-6 Sol Pro**<br>`openai/gpt-6-sol-pro` | nous_portal_batch | $2.6200 | **64.1** | 28.0 | 150 tps | 1050k | 🚫 Dominated |
| 130 | **OpenAI: GPT-6.1 Sol Pro**<br>`openai/gpt-6.1-sol-pro` | nous_portal_batch | $2.6100 | **64.1** | 28.1 | 150 tps | 1050k | 🚫 Dominated |
| 131 | **Claude Sonnet 4.6**<br>`anthropic/claude-sonnet-4.6` | nous_portal_batch | $3.9300 | **63.7** | 24.6 | 150 tps | 1000k | 🚫 Dominated |
| 132 | **OpenAI: GPT-6 Sol**<br>`openai/gpt-6-sol` | nous_portal_batch | $2.6200 | **63.4** | 27.7 | 150 tps | 1050k | 🚫 Dominated |
| 133 | **MoonshotAI: Kimi K2.6**<br>`moonshotai/kimi-k2.6` | nous_portal | $0.6077 | **63.4** | 43.0 | 75 tps | 262k | 🚫 Dominated |
| 134 | **MoonshotAI: Kimi K2.7 Code**<br>`moonshotai/kimi-k2.7-code` | nous_portal | $0.8900 | **63.1** | 38.5 | 75 tps | 262k | 🚫 Dominated |
| 135 | **Xiaomi: MiMo-V2.5-Pro**<br>`xiaomi/mimo-v2.5-pro` | nous_portal | $0.3048 | **62.9** | 49.7 | 75 tps | 1050k | 🚫 Dominated |
| 136 | **DeepSeek: DeepSeek V4 Pro 0423**<br>`deepseek/deepseek-v4-pro` | nous_portal | $0.9490 | **62.7** | 37.6 | 75 tps | 1048k | 🚫 Dominated |
| 137 | **Tencent: Hy3 preview**<br>`tencent/hy3-preview` | nous_portal | $0.2100 | **62.5** | 52.5 | 75 tps | 262k | 🚫 Dominated |
| 138 | **Xiaomi: MiMo-V2.5**<br>`xiaomi/mimo-v2.5` | nous_portal | $0.1193 | **61.9** | 55.6 | 75 tps | 1050k | 🚫 Dominated |
| 139 | **OpenAI: GPT-5.4 Nano**<br>`openai/gpt-5.4-nano` | nous_portal_batch | $0.2870 | **61.7** | 49.2 | 150 tps | 400k | 🚫 Dominated |
| 140 | **Anthropic: Claude Sonnet 5.5**<br>`anthropic/claude-sonnet-5.5` | nous_portal_batch | $2.6100 | **61.6** | 27.0 | 150 tps | 1000k | 🚫 Dominated |
| 141 | **Qwen: Qwen3.7 Plus**<br>`qwen/qwen3.7-plus` | nous_portal | $0.3904 | **61.6** | 46.3 | 75 tps | 1000k | 🚫 Dominated |
| 142 | **Z.ai: GLM 5.1**<br>`z-ai/glm-5.1` | nous_portal | $1.0943 | **61.6** | 35.4 | 75 tps | 204k | 🚫 Dominated |
| 143 | **DeepSeek R1**<br>`deepseek-r1` | nous_portal | $0.6730 | **61.2** | 40.4 | 75 tps | 64k | 🚫 Dominated |
| 144 | **Qwen: Qwen3.6 Plus**<br>`qwen/qwen3.6-plus` | nous_portal | $0.4615 | **61.2** | 44.4 | 75 tps | 1000k | 🚫 Dominated |
| 145 | **Qwen: Qwen3.6 27B**<br>`qwen/qwen3.6-27b` | nous_portal | $0.4025 | **61.0** | 45.5 | 75 tps | 262k | 🚫 Dominated |
| 146 | **Thinking Machines: Inkling Small**<br>`thinkingmachines/inkling-small` | openrouter_market | $0.4900 | **60.7** | 43.4 | 70 tps | 524k | 🚫 Dominated |
| 147 | **Thinking Machines: Inkling**<br>`thinkingmachines/inkling` | openrouter_market | $1.1810 | **60.5** | 34.0 | 70 tps | 524k | 🚫 Dominated |
| 148 | **Anthropic: Claude Sonnet 4.5**<br>`anthropic/claude-sonnet-4.5` | nous_portal_batch | $3.9300 | **60.5** | 23.3 | 150 tps | 1000k | 🚫 Dominated |
| 149 | **SpaceXAI: Grok Build 0.1**<br>`x-ai/grok-build-0.1` | nous_portal | $1.0200 | **60.3** | 35.4 | 75 tps | 256k | 🚫 Dominated |
| 150 | **Qwen: Qwen3 Coder Next**<br>`qwen/qwen3-coder-next` | nous_portal | $0.1830 | **60.0** | 51.4 | 90 tps | 262k | 🚫 Dominated |
| 151 | **Claude 3.5 Sonnet**<br>`claude-3-5-sonnet` | nous_portal | $3.6600 | **59.8** | 23.6 | 75 tps | 200k | 🚫 Dominated |
| 152 | **Claude 3.7 Sonnet**<br>`claude-3-7-sonnet` | nous_portal | $3.6600 | **59.8** | 23.6 | 75 tps | 200k | 🚫 Dominated |
| 153 | **Claude 3.7 Sonnet**<br>`claude-3-7-sonnet` | nous_portal | $3.6600 | **59.8** | 23.6 | 75 tps | 200k | 🚫 Dominated |
| 154 | **Claude 3.5 Sonnet**<br>`claude-3-5-sonnet` | nous_portal | $3.6600 | **59.8** | 23.6 | 75 tps | 200k | 🚫 Dominated |
| 155 | **Anthropic: Claude Sonnet Latest**<br>`anthropic/claude-sonnet-latest` | nous_portal | $2.6100 | **59.8** | 26.2 | 75 tps | 1000k | 🚫 Dominated |
| 156 | **Z.ai: GLM 5.3 Prime**<br>`z-ai/glm-5.3-prime` | nous_portal | $3.1760 | **59.8** | 24.6 | 75 tps | 1000k | 🚫 Dominated |
| 157 | **Qwen: Qwen3.8 Max Prime**<br>`qwen/qwen3.8-max-prime` | nous_portal | $4.4500 | **59.8** | 22.2 | 75 tps | 1000k | 🚫 Dominated |
| 158 | **OpenAI: GPT-5.1**<br>`openai/gpt-5.1` | nous_portal_batch | $2.0130 | **59.7** | 28.4 | 150 tps | 400k | 🚫 Dominated |
| 159 | **NVIDIA: Nemotron 3 Ultra**<br>`nvidia/nemotron-3-ultra-550b-a55b` | openrouter_market | $0.6300 | **59.6** | 40.1 | 70 tps | 262k | 🚫 Dominated |
| 160 | **Meta: Muse Glimmer 30B**<br>`meta/muse-glimmer-30b` | nous_portal | $0.3640 | **59.5** | 45.4 | 75 tps | 131k | 🚫 Dominated |
| 161 | **Mistral: Mistral Medium 3.5**<br>`mistralai/mistral-medium-3-5` | nous_portal_batch | $1.9800 | **58.9** | 28.2 | 150 tps | 262k | 🚫 Dominated |
| 162 | **MoonshotAI: Kimi K2.5**<br>`moonshotai/kimi-k2.5` | nous_portal | $0.6620 | **58.9** | 39.0 | 75 tps | 262k | 🚫 Dominated |
| 163 | **xAI: Grok 4.7**<br>`x-ai/grok-4.7` | nous_portal | $2.2500 | **58.8** | 27.0 | 75 tps | 500k | 🚫 Dominated |
| 164 | **Z.ai: GLM 4.6**<br>`z-ai/glm-4.6` | nous_portal | $0.5270 | **58.6** | 41.2 | 75 tps | 204k | 🚫 Dominated |
| 165 | **Z.ai: GLM 4.7**<br>`z-ai/glm-4.7` | nous_portal | $0.5030 | **58.4** | 41.5 | 75 tps | 204k | 🚫 Dominated |
| 166 | **Meituan: LongCat 2.0**<br>`meituan/longcat-2.0` | nous_portal | $0.3606 | **58.4** | 44.7 | 75 tps | 1048k | 🚫 Dominated |
| 167 | **StepFun: Step 5 Preview**<br>`stepfun/step-5-preview` | nous_portal | $1.0750 | **58.3** | 33.7 | 75 tps | 1000k | 🚫 Dominated |
| 168 | **Sakana: Fugu Max**<br>`sakana/fugu-max` | nous_portal | $2.2250 | **58.3** | 26.9 | 75 tps | 1000k | 🚫 Dominated |
| 169 | **DeepSeek: DeepSeek Pro Latest**<br>`deepseek/deepseek-pro-latest` | nous_portal | $0.9620 | **58.3** | 34.9 | 75 tps | 1048k | 🚫 Dominated |
| 170 | **OpenAI: GPT Terra Latest**<br>`openai/gpt-terra-latest` | nous_portal | $2.8200 | **58.3** | 24.9 | 75 tps | 1050k | 🚫 Dominated |
| 171 | **Z.ai: GLM Latest**<br>`z-ai/glm-latest` | nous_portal | $0.4963 | **58.3** | 41.6 | 75 tps | 1048k | 🚫 Dominated |
| 172 | **Qwen: Qwen3.5 Plus 2026-02-15**<br>`qwen/qwen3.5-plus-02-15` | nous_portal | $0.3692 | **58.3** | 44.4 | 75 tps | 1000k | 🚫 Dominated |
| 173 | **MoonshotAI: Kimi K3**<br>`moonshotai/kimi-k3:US` | nous_portal | $4.7553 | **58.3** | 21.2 | 75 tps | 1048k | 🚫 Dominated |
| 174 | **Poolside: Laguna S 2.1**<br>`poolside/laguna-s-2.1` | openrouter_market | $0.0909 | **58.3** | 53.7 | 70 tps | 1048k | 🚫 Dominated |
| 175 | **Meta: Muse Wonder**<br>`meta/muse-wonder` | nous_portal | $5.2200 | **58.3** | 20.6 | 75 tps | 1048k | 🚫 Dominated |
| 176 | **Anthropic: Claude Fable Latest**<br>`anthropic/claude-fable-latest` | nous_portal | $13.0250 | **58.3** | 16.0 | 75 tps | 1000k | 🚫 Dominated |
| 177 | **Fireworks: Ember-1**<br>`fireworks/ember-1` | nous_portal | $3.9300 | **58.3** | 22.5 | 75 tps | 1048k | 🚫 Dominated |
| 178 | **Qwen: Qwen-Plus**<br>`qwen/qwen-plus` | nous_portal | $0.2912 | **58.3** | 46.5 | 75 tps | 1000k | 🚫 Dominated |
| 179 | **Pareto 26.10 Preview**<br>`unbiased/pareto-26.10-preview` | nous_portal | $0.9630 | **58.3** | 34.8 | 75 tps | 1048k | 🚫 Dominated |
| 180 | **Tencent: Hy4 preview**<br>`tencent/hy4-preview` | nous_portal | $0.4607 | **58.3** | 42.3 | 75 tps | 1048k | 🚫 Dominated |
| 181 | **Sakana: Fugu Ultra v2**<br>`sakana/fugu-ultra-v2` | nous_portal | $7.0500 | **58.3** | 18.9 | 75 tps | 1000k | 🚫 Dominated |
| 182 | **OpenAI: GPT-5.4 Pro**<br>`openai/gpt-5.4-pro` | nous_portal_batch | $42.6000 | **58.3** | 12.2 | 150 tps | 1050k | 🚫 Dominated |
| 183 | **Mistral: Mistral Large 4**<br>`mistralai/mistral-large-4-0` | nous_portal | $0.7600 | **58.3** | 37.3 | 75 tps | 1048k | 🚫 Dominated |
| 184 | **Qwen: Qwen3.5 Plus 2026-04-20**<br>`qwen/qwen3.5-plus-20260420` | nous_portal | $0.4260 | **58.3** | 43.1 | 75 tps | 1000k | 🚫 Dominated |
| 185 | **MoonshotAI: Kimi Latest**<br>`moonshotai/kimi-latest` | nous_portal | $2.0920 | **58.3** | 27.4 | 75 tps | 1048k | 🚫 Dominated |
| 186 | **xAI: Grok Latest**<br>`x-ai/grok-latest` | nous_portal | $2.2500 | **58.3** | 26.8 | 75 tps | 500k | 🚫 Dominated |
| 187 | **OpenAI: GPT-5.6 Terra Pro**<br>`openai/gpt-5.6-terra-pro` | nous_portal_batch | $2.8200 | **58.3** | 24.9 | 150 tps | 1050k | 🚫 Dominated |
| 188 | **Sakana: Fugu Ultra**<br>`sakana/fugu-ultra` | nous_portal | $7.0500 | **58.3** | 18.9 | 75 tps | 1000k | 🚫 Dominated |
| 189 | **OpenAI: GPT-5.5 Pro**<br>`openai/gpt-5.5-pro` | nous_portal_batch | $42.6000 | **58.3** | 12.2 | 150 tps | 1050k | 🚫 Dominated |
| 190 | **Xiaomi: MiMo-V2.6-Pro-UltraSpeed**<br>`xiaomi/mimo-v2.6-pro-ultraspeed` | nous_portal | $4.3536 | **58.3** | 21.8 | 75 tps | 1048k | 🚫 Dominated |
| 191 | **Z.ai: GLM 5.3**<br>`z-ai/glm-5.3:US` | nous_portal | $2.6169 | **58.3** | 25.5 | 75 tps | 1048k | 🚫 Dominated |
| 192 | **DeepSeek: DeepSeek V3.2**<br>`deepseek/deepseek-v3.2` | nous_portal | $0.2002 | **58.1** | 49.1 | 75 tps | 163k | 🚫 Dominated |
| 193 | **DeepSeek: DeepSeek V3.1 Terminus**<br>`deepseek/deepseek-v3.1-terminus` | nous_portal | $0.3214 | **57.9** | 45.3 | 75 tps | 163k | 🚫 Dominated |
| 194 | **Google: Gemma 4 31B**<br>`google/gemma-4-31b-it` | openrouter_market | $0.1110 | **57.9** | 52.4 | 70 tps | 262k | 🚫 Dominated |
| 195 | **Qwen: Qwen3.5-122B-A10B**<br>`qwen/qwen3.5-122b-a10b` | nous_portal | $0.4212 | **57.8** | 42.8 | 75 tps | 262k | 🚫 Dominated |
| 196 | **SpaceXAI: Grok 4.3**<br>`x-ai/grok-4.3` | nous_portal_batch | $1.2700 | **57.5** | 31.6 | 150 tps | 1000k | 🚫 Dominated |
| 197 | **Qwen: Qwen3.6 35B A3B**<br>`qwen/qwen3.6-35b-a3b` | nous_portal | $0.1125 | **57.4** | 51.9 | 75 tps | 262k | 🚫 Dominated |
| 198 | **DeepSeek V3**<br>`deepseek-v3` | nous_portal | $0.6100 | **56.8** | 38.5 | 92 tps | 64k | 🚫 Dominated |
| 199 | **GPT-4o**<br>`gpt-4o` | nous_portal | $0.6100 | **56.8** | 38.5 | 75 tps | 128k | 🚫 Dominated |
| 200 | **GPT-4o**<br>`gpt-4o` | nous_portal | $0.6100 | **56.8** | 38.5 | 75 tps | 128k | 🚫 Dominated |
| 201 | **DeepSeek V3**<br>`deepseek-v3` | nous_portal | $0.6100 | **56.8** | 38.5 | 92 tps | 64k | 🚫 Dominated |
| 202 | **ByteDance Seed: Seed-2.0-Lite**<br>`bytedance-seed/seed-2.0-lite` | nous_portal | $0.4050 | **56.8** | 42.4 | 75 tps | 262k | 🚫 Dominated |
| 203 | **Qwen: Qwen3 Embedding 8B**<br>`qwen/qwen3-embedding-8b` | nous_portal | $0.0082 | **56.8** | 56.4 | 75 tps | 32k | 🚫 Dominated |
| 204 | **Intfloat: E5-Base-v2**<br>`intfloat/e5-base-v2` | nous_portal | $0.0041 | **56.8** | 56.6 | 75 tps | 0k | 🚫 Dominated |
| 205 | **OpenAI: Text Embedding Ada 002**<br>`openai/text-embedding-ada-002` | nous_portal_batch | $0.0820 | **56.8** | 52.7 | 150 tps | 8k | 🚫 Dominated |
| 206 | **Sentence Transformers: multi-qa-mpnet-base-dot-v1**<br>`sentence-transformers/multi-qa-mpnet-base-dot-v1` | nous_portal | $0.0041 | **56.8** | 56.6 | 75 tps | 0k | 🚫 Dominated |
| 207 | **BAAI: bge-base-en-v1.5**<br>`baai/bge-base-en-v1.5` | nous_portal | $0.0041 | **56.8** | 56.6 | 75 tps | 0k | 🚫 Dominated |
| 208 | **OpenAI: gpt-oss-safeguard-20b**<br>`openai/gpt-oss-safeguard-20b` | nous_portal | $0.0938 | **56.8** | 52.2 | 75 tps | 131k | 🚫 Dominated |
| 209 | **Pareto Latest**<br>`unbiased/pareto-latest` | nous_portal | $2.7750 | **56.8** | 24.4 | 75 tps | 262k | 🚫 Dominated |
| 210 | **Intfloat: E5-Large-v2**<br>`intfloat/e5-large-v2` | nous_portal | $0.0082 | **56.8** | 56.4 | 75 tps | 0k | 🚫 Dominated |
| 211 | **VoyageAI by MongoDB: voyage-multimodal-3.5**<br>`voyageai/voyage-multimodal-3.5` | nous_portal | $0.0984 | **56.8** | 52.0 | 75 tps | 32k | 🚫 Dominated |
| 212 | **Qwen: Qwen2.5 7B Instruct**<br>`qwen/qwen-2.5-7b-instruct` | nous_portal | $0.1020 | **56.8** | 51.8 | 75 tps | 32k | 🚫 Dominated |
| 213 | **Nex AGI: Nex-N2.5-Pro**<br>`nex-agi/nex-n2.5-pro` | nous_portal | $0.0865 | **56.8** | 52.5 | 75 tps | 262k | 🚫 Dominated |
| 214 | **Sentence Transformers: all-mpnet-base-v2**<br>`sentence-transformers/all-mpnet-base-v2` | nous_portal | $0.0041 | **56.8** | 56.6 | 75 tps | 0k | 🚫 Dominated |
| 215 | **Poolside: Laguna XS 2.1**<br>`poolside/laguna-xs-2.1` | openrouter_market | $0.0630 | **56.8** | 53.6 | 70 tps | 262k | 🚫 Dominated |
| 216 | **Perplexity: Embed V1 4B**<br>`perplexity/pplx-embed-v1-4b` | nous_portal | $0.0246 | **56.8** | 55.5 | 75 tps | 32k | 🚫 Dominated |
| 217 | **ByteDance Seed: Seed 1.6**<br>`bytedance-seed/seed-1.6` | nous_portal | $0.4050 | **56.8** | 42.4 | 75 tps | 262k | 🚫 Dominated |
| 218 | **OpenAI: Text Embedding 3 Large**<br>`openai/text-embedding-3-large` | nous_portal_batch | $0.1066 | **56.8** | 51.6 | 150 tps | 8k | 🚫 Dominated |
| 219 | **Perplexity: Embed V1 0.6B**<br>`perplexity/pplx-embed-v1-0.6b` | nous_portal | $0.0033 | **56.8** | 56.7 | 75 tps | 32k | 🚫 Dominated |
| 220 | **Intfloat: Multilingual-E5-Large**<br>`intfloat/multilingual-e5-large` | nous_portal | $0.0082 | **56.8** | 56.4 | 75 tps | 0k | 🚫 Dominated |
| 221 | **Mistral: Mistral Small 3.2 24B**<br>`mistralai/mistral-small-3.2-24b-instruct` | nous_portal | $0.0815 | **56.8** | 52.7 | 75 tps | 256k | 🚫 Dominated |
| 222 | **OpenAI: GPT-3.5 Turbo 16k**<br>`openai/gpt-3.5-turbo-16k` | nous_portal | $2.8600 | **56.8** | 24.2 | 75 tps | 16k | 🚫 Dominated |
| 223 | **Cohere: Command A+**<br>`cohere/command-a-plus` | nous_portal | $0.4050 | **56.8** | 42.4 | 75 tps | 192k | 🚫 Dominated |
| 224 | **OpenAI: GPT-5 Pro**<br>`openai/gpt-5-pro` | nous_portal_batch | $24.3000 | **56.8** | 13.4 | 150 tps | 400k | 🚫 Dominated |
| 225 | **Pareto 26.9**<br>`unbiased/pareto-26.9` | nous_portal | $2.7750 | **56.8** | 24.4 | 75 tps | 262k | 🚫 Dominated |
| 226 | **AionLabs: Aion-2.0**<br>`aion-labs/aion-2.0` | nous_portal | $0.8200 | **56.8** | 35.6 | 75 tps | 131k | 🚫 Dominated |
| 227 | **ByteDance Seed: Seed 2.1 Turbo**<br>`bytedance-seed/seed-2-1-turbo` | nous_portal | $0.6600 | **56.8** | 37.7 | 75 tps | 262k | 🚫 Dominated |
| 228 | **BAAI: bge-large-en-v1.5**<br>`baai/bge-large-en-v1.5` | nous_portal | $0.0082 | **56.8** | 56.4 | 75 tps | 0k | 🚫 Dominated |
| 229 | **OpenAI: GPT-5.2 Chat**<br>`openai/gpt-5.2-chat` | nous_portal | $2.8175 | **56.8** | 24.3 | 75 tps | 128k | 🚫 Dominated |
| 230 | **Mistral: Voxtral Small 24B 2507**<br>`mistralai/voxtral-small-24b-2507` | nous_portal | $0.1110 | **56.8** | 51.4 | 75 tps | 32k | 🚫 Dominated |
| 231 | **VoyageAI by MongoDB: voyage-4-lite**<br>`voyageai/voyage-4-lite` | nous_portal | $0.0164 | **56.8** | 55.9 | 75 tps | 32k | 🚫 Dominated |
| 232 | **MissingNo**<br>`stealth/missingno` | nous_portal | $0.0010 | **56.8** | 56.8 | 75 tps | 262k | 💎 **Best Value** |
| 233 | **VoyageAI by MongoDB: voyage-code-4**<br>`voyageai/voyage-code-4` | nous_portal | $0.0984 | **56.8** | 52.0 | 75 tps | 32k | 🚫 Dominated |
| 234 | **Cohere: Command R+ (08-2024)**<br>`cohere/command-r-plus-08-2024` | nous_portal | $3.0500 | **56.8** | 23.7 | 75 tps | 128k | 🚫 Dominated |
| 235 | **OpenAI: GPT-5.2 Pro**<br>`openai/gpt-5.2-pro` | nous_portal_batch | $34.0200 | **56.8** | 12.5 | 150 tps | 400k | 🚫 Dominated |
| 236 | **PrismML: Ternary Bonsai 2 27B**<br>`prism-ml/ternary-bonsai-2-27b` | nous_portal | $0.1138 | **56.8** | 51.3 | 75 tps | 262k | 🚫 Dominated |
| 237 | **ByteDance Seed: Seed-2.0-Code**<br>`bytedance-seed/seed-2.0-code` | nous_portal | $0.7100 | **56.8** | 37.0 | 75 tps | 262k | 🚫 Dominated |
| 238 | **BAAI: bge-m3**<br>`baai/bge-m3` | nous_portal | $0.0082 | **56.8** | 56.4 | 75 tps | 8k | 🚫 Dominated |
| 239 | **OpenAI: Text Embedding 3 Small**<br>`openai/text-embedding-3-small` | nous_portal | $0.0164 | **56.8** | 55.9 | 75 tps | 8k | 🚫 Dominated |
| 240 | **Thenlper: GTE-Large**<br>`thenlper/gte-large` | nous_portal | $0.0082 | **56.8** | 56.4 | 75 tps | 0k | 🚫 Dominated |
| 241 | **DeepSeek: DeepSeek V3**<br>`deepseek/deepseek-chat` | nous_portal | $0.3139 | **56.8** | 44.7 | 75 tps | 163k | 🚫 Dominated |
| 242 | **OpenAI: GPT-5.1-Codex-Max**<br>`openai/gpt-5.1-codex-max` | nous_portal | $2.0125 | **56.8** | 27.0 | 75 tps | 400k | 🚫 Dominated |
| 243 | **VoyageAI by MongoDB: voyage-4-large**<br>`voyageai/voyage-4-large` | nous_portal | $0.0984 | **56.8** | 52.0 | 75 tps | 32k | 🚫 Dominated |
| 244 | **OpenAI: GPT Chat Latest**<br>`openai/gpt-chat-latest` | nous_portal | $7.0500 | **56.8** | 18.4 | 75 tps | 400k | 🚫 Dominated |
| 245 | **Reka Edge**<br>`rekaai/reka-edge` | nous_portal | $0.0920 | **56.8** | 52.2 | 75 tps | 16k | 🚫 Dominated |
| 246 | **Mistral: Codestral Embed 2505**<br>`mistralai/codestral-embed-2505` | nous_portal | $0.1230 | **56.8** | 50.9 | 75 tps | 8k | 🚫 Dominated |
| 247 | **Mistral: Mistral Nemo**<br>`mistralai/mistral-nemo` | nous_portal | $0.0165 | **56.8** | 55.9 | 75 tps | 131k | 🚫 Dominated |
| 248 | **AionLabs: Aion 3.5**<br>`aion-labs/aion-3.5` | nous_portal | $3.0750 | **56.8** | 23.6 | 75 tps | 262k | 🚫 Dominated |
| 249 | **Mistral: Codestral 2508**<br>`mistralai/codestral-2508` | nous_portal_batch | $0.3330 | **56.8** | 44.2 | 150 tps | 256k | 🚫 Dominated |
| 250 | **Perceptron: Perceptron Mk1.5**<br>`perceptron/perceptron-mk1.5` | nous_portal | $0.2730 | **56.8** | 45.8 | 75 tps | 36k | 🚫 Dominated |
| 251 | **OpenAI: GPT-3.5 Turbo (older v0613)**<br>`openai/gpt-3.5-turbo-0613` | nous_portal | $1.0200 | **56.8** | 33.4 | 75 tps | 4k | 🚫 Dominated |
| 252 | **VoyageAI by MongoDB: voyage-4**<br>`voyageai/voyage-4` | nous_portal | $0.0492 | **56.8** | 54.2 | 75 tps | 32k | 🚫 Dominated |
| 253 | **LiquidAI: LFM2.5-24B-A2B**<br>`liquid/lfm-2.5-24b-a2b` | nous_portal | $0.1125 | **56.8** | 51.4 | 75 tps | 128k | 🚫 Dominated |
| 254 | **Mistral: Mistral Small 3.1 24B**<br>`mistralai/mistral-small-3.1-24b-instruct` | nous_portal | $0.3433 | **56.8** | 43.9 | 75 tps | 128k | 🚫 Dominated |
| 255 | **Relace: Relace Search**<br>`relace/relace-search` | nous_portal | $1.1200 | **56.8** | 32.5 | 75 tps | 256k | 🚫 Dominated |
| 256 | **AionLabs: Aion-3.0**<br>`aion-labs/aion-3.0` | nous_portal | $3.0750 | **56.8** | 23.6 | 75 tps | 131k | 🚫 Dominated |
| 257 | **Qwen: Qwen3 Embedding 4B**<br>`qwen/qwen3-embedding-4b` | nous_portal | $0.0164 | **56.8** | 55.9 | 75 tps | 32k | 🚫 Dominated |
| 258 | **OpenAI: GPT-4o**<br>`openai/gpt-4o` | nous_portal_batch | $3.0500 | **56.8** | 23.7 | 150 tps | 128k | 🚫 Dominated |
| 259 | **Thenlper: GTE-Base**<br>`thenlper/gte-base` | nous_portal | $0.0041 | **56.8** | 56.6 | 75 tps | 0k | 🚫 Dominated |
| 260 | **Mistral: Mistral Embed 2312**<br>`mistralai/mistral-embed-2312` | nous_portal | $0.0820 | **56.8** | 52.7 | 75 tps | 8k | 🚫 Dominated |
| 261 | **OpenAI: GPT Audio**<br>`openai/gpt-audio` | nous_portal | $3.0500 | **56.8** | 23.7 | 75 tps | 128k | 🚫 Dominated |
| 262 | **Cohere: Command R (08-2024)**<br>`cohere/command-r-08-2024` | nous_portal | $0.1830 | **56.8** | 48.7 | 75 tps | 128k | 🚫 Dominated |
| 263 | **OpenAI: o1**<br>`openai/o1` | nous_portal | $18.7500 | **56.8** | 14.2 | 75 tps | 200k | 🚫 Dominated |
| 264 | **Google: Gemma 4 26B A4B **<br>`google/gemma-4-26b-a4b-it` | openrouter_market | $0.0577 | **56.6** | 53.6 | 70 tps | 262k | 🚫 Dominated |
| 265 | **Hermes 3 (Llama 3.1 405B)**<br>`hermes-3-llama-3.1-405b` | nous_portal | $0.9250 | **56.3** | 34.0 | 40 tps | 128k | 🚫 Dominated |
| 266 | **OpenAI: GPT-5**<br>`openai/gpt-5` | nous_portal_batch | $2.0125 | **56.2** | 26.7 | 150 tps | 400k | 🚫 Dominated |
| 267 | **NVIDIA: Nemotron 3 Super**<br>`nvidia/nemotron-3-super-120b-a12b` | openrouter_market | $0.1106 | **56.2** | 50.8 | 70 tps | 262k | 🚫 Dominated |
| 268 | **Anthropic: Claude Sonnet 4**<br>`anthropic/claude-sonnet-4` | nous_portal | $3.9300 | **56.1** | 21.6 | 75 tps | 200k | 🚫 Dominated |
| 269 | **Anthropic: Claude Opus 5**<br>`anthropic/claude-opus-5` | nous_portal_batch | $6.5500 | **55.5** | 18.4 | 150 tps | 1000k | 🚫 Dominated |
| 270 | **OpenAI: GPT-6 Astra**<br>`openai/gpt-6-astra` | nous_portal_batch | $13.1000 | **55.2** | 15.1 | 150 tps | 1050k | 🚫 Dominated |
| 271 | **OpenAI: GPT-5.3-Codex**<br>`openai/gpt-5.3-codex` | nous_portal | $2.8175 | **54.6** | 23.3 | 75 tps | 400k | 🚫 Dominated |
| 272 | **Claude Opus 4.8**<br>`anthropic/claude-opus-4.8` | nous_portal_batch | $6.5500 | **54.4** | 18.0 | 150 tps | 1000k | 🚫 Dominated |
| 273 | **Mistral: Devstral 2 2512**<br>`mistralai/devstral-2512` | nous_portal | $0.5240 | **54.2** | 38.2 | 75 tps | 262k | 🚫 Dominated |
| 274 | **Inception: Mercury 2**<br>`inception/mercury-2` | nous_portal | $0.2775 | **54.2** | 43.5 | 75 tps | 128k | 🚫 Dominated |
| 275 | **Claude Opus 4.7**<br>`anthropic/claude-opus-4.7` | nous_portal_batch | $6.5500 | **54.2** | 17.9 | 150 tps | 1000k | 🚫 Dominated |
| 276 | **Upstage: Solar Pro 3**<br>`upstage/solar-pro-3` | nous_portal | $0.1815 | **54.0** | 46.3 | 90 tps | 131k | 🚫 Dominated |
| 277 | **OpenAI: gpt-oss-120b**<br>`openai/gpt-oss-120b` | nous_portal_batch | $0.0420 | **54.0** | 51.8 | 150 tps | 131k | 🚫 Dominated |
| 278 | **OpenAI: GPT-5.2**<br>`openai/gpt-5.2` | nous_portal_batch | $2.8175 | **54.0** | 23.1 | 150 tps | 400k | 🚫 Dominated |
| 279 | **OpenAI: GPT-5.2-Codex**<br>`openai/gpt-5.2-codex` | nous_portal | $2.8175 | **53.4** | 22.8 | 75 tps | 400k | 🚫 Dominated |
| 280 | **NVIDIA: Nemotron 3.5 Lightning**<br>`nvidia/nemotron-3.5-lightning` | openrouter_market | $0.0333 | **52.9** | 51.2 | 70 tps | 262k | 🚫 Dominated |
| 281 | **Z.ai: GLM 5 Turbo**<br>`z-ai/glm-5-turbo` | nous_portal | $1.3840 | **52.8** | 28.3 | 75 tps | 202k | 🚫 Dominated |
| 282 | **Mistral: Mistral Small 4**<br>`mistralai/mistral-small-2603` | nous_portal_batch | $0.1815 | **52.8** | 45.3 | 150 tps | 262k | 🚫 Dominated |
| 283 | **Qwen: Qwen3 Coder 480B A35B**<br>`qwen/qwen3-coder` | nous_portal | $0.3604 | **52.7** | 40.3 | 90 tps | 262k | 🚫 Dominated |
| 284 | **Arcee AI: Trinity Large Thinking**<br>`arcee-ai/trinity-large-thinking` | nous_portal | $0.2860 | **52.6** | 42.0 | 75 tps | 262k | 🚫 Dominated |
| 285 | **SpaceXAI: Grok 4.20**<br>`x-ai/grok-4.20` | nous_portal | $1.2700 | **52.6** | 28.9 | 75 tps | 2000k | 🚫 Dominated |
| 286 | **OpenAI: GPT-4o (2024-05-13)**<br>`openai/gpt-4o-2024-05-13` | nous_portal | $5.6000 | **52.1** | 18.0 | 75 tps | 128k | 🚫 Dominated |
| 287 | **Qwen: Qwen3 Coder 30B A3B Instruct**<br>`qwen/qwen3-coder-30b-a3b-instruct` | nous_portal | $0.0854 | **52.0** | 48.1 | 90 tps | 262k | 🚫 Dominated |
| 288 | **OpenAI: GPT-5.1-Codex**<br>`openai/gpt-5.1-codex` | nous_portal | $2.0130 | **52.0** | 24.7 | 75 tps | 400k | 🚫 Dominated |
| 289 | **Qwen: Qwen3.5-9B**<br>`qwen/qwen3.5-9b` | nous_portal | $0.0810 | **51.9** | 48.1 | 75 tps | 262k | 🚫 Dominated |
| 290 | **Amazon: Nova 2 Lite**<br>`amazon/nova-2-lite-v1` | nous_portal | $0.4960 | **51.7** | 36.9 | 75 tps | 1000k | 🚫 Dominated |
| 291 | **Qwen: Qwen3.5-27B**<br>`qwen/qwen3.5-27b` | nous_portal | $0.3159 | **51.7** | 40.6 | 75 tps | 262k | 🚫 Dominated |
| 292 | **IBM: Granite 4.2 8B**<br>`ibm-granite/granite-4.2-8b` | nous_portal | $0.0745 | **51.6** | 48.1 | 75 tps | 131k | 🚫 Dominated |
| 293 | **Qwen: Qwen3 235B A22B Thinking 2507**<br>`qwen/qwen3-235b-a22b-thinking-2507` | nous_portal | $0.7190 | **51.5** | 33.4 | 75 tps | 128k | 🚫 Dominated |
| 294 | **MoonshotAI: Kimi K2 Thinking**<br>`moonshotai/kimi-k2-thinking` | nous_portal | $0.7420 | **51.4** | 33.1 | 75 tps | 262k | 🚫 Dominated |
| 295 | **OpenAI: o3 Pro**<br>`openai/o3-pro` | nous_portal | $24.4000 | **51.4** | 12.1 | 75 tps | 200k | 🚫 Dominated |
| 296 | **OpenAI: GPT-4 Turbo**<br>`openai/gpt-4-turbo` | nous_portal_batch | $11.2000 | **51.3** | 14.6 | 150 tps | 128k | 🚫 Dominated |
| 297 | **DeepSeek: DeepSeek V3 0324**<br>`deepseek/deepseek-chat-v3-0324` | nous_portal | $0.3050 | **51.2** | 40.4 | 75 tps | 163k | 🚫 Dominated |
| 298 | **Qwen 2.5 Coder 32B**<br>`qwen-2.5-coder-32b` | nous_portal | $0.6100 | **51.1** | 34.6 | 105 tps | 131k | 🚫 Dominated |
| 299 | **Qwen 2.5 Coder 32B**<br>`qwen-2.5-coder-32b` | nous_portal | $0.6100 | **51.1** | 34.6 | 105 tps | 131k | 🚫 Dominated |
| 300 | **Mistral: Mistral Medium 3.1**<br>`mistralai/mistral-medium-3.1` | nous_portal_batch | $0.5240 | **51.0** | 35.9 | 150 tps | 131k | 🚫 Dominated |
| 301 | **OpenAI: o3**<br>`openai/o3` | nous_portal_batch | $2.4500 | **50.9** | 22.7 | 150 tps | 200k | 🚫 Dominated |
| 302 | **Mistral: Mistral Large 3 2512**<br>`mistralai/mistral-large-2512` | nous_portal_batch | $0.5550 | **50.9** | 35.3 | 150 tps | 262k | 🚫 Dominated |
| 303 | **Qwen: Qwen3.5-35B-A3B**<br>`qwen/qwen3.5-35b-a3b` | nous_portal | $0.1430 | **50.6** | 44.7 | 75 tps | 262k | 🚫 Dominated |
| 304 | **Qwen: Qwen3 Next 80B A3B Thinking**<br>`qwen/qwen3-next-80b-a3b-thinking` | nous_portal | $0.2430 | **50.1** | 41.1 | 75 tps | 262k | 🚫 Dominated |
| 305 | **DeepSeek: DeepSeek V3.2 Exp**<br>`deepseek/deepseek-v3.2-exp` | nous_portal | $0.2624 | **49.8** | 40.4 | 75 tps | 163k | 🚫 Dominated |
| 306 | **Meta: Llama 4 Maverick**<br>`meta-llama/llama-4-maverick` | nous_portal | $0.2203 | **49.7** | 41.5 | 75 tps | 1048k | 🚫 Dominated |
| 307 | **DeepSeek R1**<br>`deepseek-r1` | nous_portal | $0.6100 | **49.5** | 33.5 | 75 tps | 64k | 🚫 Dominated |
| 308 | **DeepSeek R1**<br>`deepseek-r1` | nous_portal | $0.6100 | **49.5** | 33.5 | 75 tps | 64k | 🚫 Dominated |
| 309 | **DeepSeek: R1**<br>`deepseek/deepseek-r1` | nous_portal | $0.8240 | **49.5** | 30.9 | 75 tps | 64k | 🚫 Dominated |
| 310 | **Qwen: Qwen3 32B**<br>`qwen/qwen3-32b` | nous_portal | $0.0936 | **49.4** | 45.4 | 75 tps | 131k | 🚫 Dominated |
| 311 | **MoonshotAI: Kimi K2 0905**<br>`moonshotai/kimi-k2-0905` | nous_portal | $0.7420 | **49.4** | 31.8 | 75 tps | 262k | 🚫 Dominated |
| 312 | **Anthropic: Claude Opus 5.5**<br>`anthropic/claude-opus-5.5` | nous_portal_batch | $5.2200 | **49.4** | 17.5 | 150 tps | 1000k | 🚫 Dominated |
| 313 | **NVIDIA: Nemotron 3 Nano 30B A3B**<br>`nvidia/nemotron-3-nano-30b-a3b` | nous_portal | $0.0610 | **49.2** | 46.4 | 75 tps | 262k | 🚫 Dominated |
| 314 | **Qwen: Qwen3 14B**<br>`qwen/qwen3-14b` | nous_portal | $0.1040 | **49.0** | 44.6 | 75 tps | 40k | 🚫 Dominated |
| 315 | **DeepSeek: DeepSeek V3.1**<br>`deepseek/deepseek-chat-v3.1` | nous_portal | $0.3080 | **49.0** | 38.6 | 75 tps | 163k | 🚫 Dominated |
| 316 | **OpenAI: GPT-4**<br>`openai/gpt-4` | nous_portal | $30.6000 | **48.8** | 11.0 | 75 tps | 8k | 🚫 Dominated |
| 317 | **OpenAI: GPT-5 Nano**<br>`openai/gpt-5-nano` | nous_portal_batch | $0.0810 | **48.7** | 45.2 | 150 tps | 400k | 🚫 Dominated |
| 318 | **MoonshotAI: Kimi K2 0711**<br>`moonshotai/kimi-k2` | nous_portal | $0.6974 | **48.7** | 31.8 | 75 tps | 131k | 🚫 Dominated |
| 319 | **OpenAI: GPT-4.1**<br>`openai/gpt-4.1` | nous_portal_batch | $2.4500 | **48.7** | 21.7 | 150 tps | 1047k | 🚫 Dominated |
| 320 | **Inception: Mercury 2.5**<br>`inception/mercury-2.5` | nous_portal | $0.0474 | **48.5** | 46.4 | 75 tps | 260k | 🚫 Dominated |
| 321 | **Qwen: Qwen3 235B A22B Instruct 2507**<br>`qwen/qwen3-235b-a22b-2507` | nous_portal | $0.1067 | **48.4** | 44.0 | 75 tps | 262k | 🚫 Dominated |
| 322 | **Llama 3.3 70B Instruct**<br>`llama-3.3-70b` | nous_portal | $0.6100 | **48.4** | 32.8 | 75 tps | 128k | 🚫 Dominated |
| 323 | **Llama 3.3 70B Instruct**<br>`llama-3.3-70b` | nous_portal | $0.6100 | **48.4** | 32.8 | 75 tps | 128k | 🚫 Dominated |
| 324 | **Meta: Llama 3.3 70B Instruct**<br>`meta-llama/llama-3.3-70b-instruct` | nous_portal | $0.1140 | **48.4** | 43.7 | 75 tps | 131k | 🚫 Dominated |
| 325 | **Z.ai: GLM 4.6V**<br>`z-ai/glm-4.6v` | nous_portal | $0.3355 | **48.2** | 37.4 | 75 tps | 131k | 🚫 Dominated |
| 326 | **OpenAI: GPT-4.1 Nano**<br>`openai/gpt-4.1-nano` | nous_portal_batch | $0.1230 | **48.2** | 43.2 | 150 tps | 1047k | 🚫 Dominated |
| 327 | **Z.ai: GLM 4.5 Air**<br>`z-ai/glm-4.5-air` | nous_portal | $0.1915 | **48.2** | 41.0 | 75 tps | 131k | 🚫 Dominated |
| 328 | **OpenAI: GPT-3.5 Turbo**<br>`openai/gpt-3.5-turbo` | nous_portal_batch | $0.5600 | **48.1** | 33.3 | 150 tps | 16k | 🚫 Dominated |
| 329 | **OpenAI: GPT Astra Latest**<br>`openai/gpt-astra-latest` | nous_portal | $13.1000 | **48.0** | 13.2 | 40 tps | 1050k | 🚫 Dominated |
| 330 | **OpenAI: GPT-6 Astra Pro Flex**<br>`openai/gpt-6-astra-pro-flex` | nous_portal | $6.5500 | **48.0** | 15.9 | 40 tps | 1050k | 🚫 Dominated |
| 331 | **OpenAI: GPT-6 Astra Pro Fast**<br>`openai/gpt-6-astra-pro-fast` | nous_portal | $26.2000 | **48.0** | 11.2 | 40 tps | 1050k | 🚫 Dominated |
| 332 | **OpenAI: GPT-6 Astra Fast**<br>`openai/gpt-6-astra-fast` | nous_portal | $26.2000 | **48.0** | 11.2 | 40 tps | 1050k | 🚫 Dominated |
| 333 | **OpenAI: GPT-6 Astra Flex**<br>`openai/gpt-6-astra-flex` | nous_portal | $6.5500 | **48.0** | 15.9 | 40 tps | 1050k | 🚫 Dominated |
| 334 | **Anthropic: Claude Opus Latest**<br>`anthropic/claude-opus-latest` | nous_portal | $5.2200 | **48.0** | 17.0 | 40 tps | 1000k | 🚫 Dominated |
| 335 | **OpenAI: GPT-6 Astra Pro**<br>`openai/gpt-6-astra-pro` | nous_portal_batch | $13.1000 | **48.0** | 13.2 | 150 tps | 1050k | 🚫 Dominated |
| 336 | **Google: Gemma 3 27B**<br>`google/gemma-3-27b-it` | nous_portal | $0.0875 | **47.9** | 44.2 | 75 tps | 131k | 🚫 Dominated |
| 337 | **OpenAI: gpt-oss-20b**<br>`openai/gpt-oss-20b` | nous_portal | $0.0243 | **47.8** | 46.7 | 75 tps | 131k | 🚫 Dominated |
| 338 | **Qwen: Qwen3 VL 235B A22B Instruct**<br>`qwen/qwen3-vl-235b-a22b-instruct` | nous_portal | $0.2590 | **47.8** | 38.9 | 75 tps | 262k | 🚫 Dominated |
| 339 | **Qwen: Qwen3 Next 80B A3B Instruct**<br>`qwen/qwen3-next-80b-a3b-instruct` | nous_portal | $0.1838 | **47.7** | 40.8 | 75 tps | 262k | 🚫 Dominated |
| 340 | **Qwen: Qwen3 VL 30B A3B Thinking**<br>`qwen/qwen3-vl-30b-a3b-thinking` | nous_portal | $0.3378 | **47.7** | 36.9 | 75 tps | 262k | 🚫 Dominated |
| 341 | **Amazon: Nova Premier 1.0**<br>`amazon/nova-premier-v1` | nous_portal | $3.3125 | **47.6** | 19.3 | 75 tps | 1000k | 🚫 Dominated |
| 342 | **Mistral: Mistral Medium 3**<br>`mistralai/mistral-medium-3` | nous_portal | $0.5240 | **47.5** | 33.4 | 75 tps | 131k | 🚫 Dominated |
| 343 | **OpenAI: GPT-4o (2024-11-20)**<br>`openai/gpt-4o-2024-11-20` | nous_portal | $3.1250 | **47.4** | 19.6 | 75 tps | 128k | 🚫 Dominated |
| 344 | **Meta: Llama 4 Scout**<br>`meta-llama/llama-4-scout` | nous_portal | $0.1120 | **47.3** | 42.8 | 75 tps | 1310k | 🚫 Dominated |
| 345 | **Qwen: Qwen3 VL 30B A3B Instruct**<br>`qwen/qwen3-vl-30b-a3b-instruct` | nous_portal | $0.1830 | **47.2** | 40.4 | 75 tps | 262k | 🚫 Dominated |
| 346 | **OpenAI: GPT-4o (2024-08-06)**<br>`openai/gpt-4o-2024-08-06` | nous_portal | $3.1250 | **47.2** | 19.5 | 75 tps | 128k | 🚫 Dominated |
| 347 | **Qwen2.5 72B Instruct**<br>`qwen/qwen-2.5-72b-instruct` | nous_portal | $0.3352 | **47.2** | 36.6 | 75 tps | 32k | 🚫 Dominated |
| 348 | **Qwen: Qwen3 30B A3B**<br>`qwen/qwen3-30b-a3b` | nous_portal | $0.1484 | **47.1** | 41.4 | 75 tps | 40k | 🚫 Dominated |
| 349 | **Z.ai: GLM 4.5V**<br>`z-ai/glm-4.5v` | nous_portal | $0.6710 | **47.1** | 31.1 | 75 tps | 65k | 🚫 Dominated |
| 350 | **Mistral Large 2407**<br>`mistralai/mistral-large-2407` | nous_portal | $2.2200 | **47.1** | 21.7 | 75 tps | 131k | 🚫 Dominated |
| 351 | **Qwen: Qwen3 30B A3B Instruct 2507**<br>`qwen/qwen3-30b-a3b-instruct-2507` | nous_portal | $0.1038 | **47.1** | 42.9 | 75 tps | 262k | 🚫 Dominated |
| 352 | **Qwen: Qwen3 VL 8B Instruct**<br>`qwen/qwen3-vl-8b-instruct` | nous_portal | $0.2870 | **47.0** | 37.6 | 75 tps | 262k | 🚫 Dominated |
| 353 | **Amazon: Nova Pro 1.0**<br>`amazon/nova-pro-v1` | nous_portal | $0.9760 | **46.9** | 27.9 | 75 tps | 300k | 🚫 Dominated |
| 354 | **Hermes 3 (Llama 3.1 70B)**<br>`hermes-3-llama-3.1-70b` | nous_portal | $0.6100 | **46.9** | 31.7 | 78 tps | 128k | 🚫 Dominated |
| 355 | **Hermes 3 (Llama 3.1 70B)**<br>`hermes-3-llama-3.1-70b` | nous_portal | $0.6100 | **46.9** | 31.7 | 78 tps | 128k | 🚫 Dominated |
| 356 | **Amazon: Nova Lite 1.0**<br>`amazon/nova-lite-v1` | nous_portal | $0.0732 | **46.9** | 43.8 | 75 tps | 300k | 🚫 Dominated |
| 357 | **Meta: Llama 3.1 70B Instruct**<br>`meta-llama/llama-3.1-70b-instruct` | nous_portal | $0.3680 | **46.8** | 35.7 | 75 tps | 131k | 🚫 Dominated |
| 358 | **Mistral: Saba**<br>`mistralai/mistral-saba` | nous_portal | $0.2220 | **46.8** | 39.0 | 75 tps | 32k | 🚫 Dominated |
| 359 | **Amazon: Nova Micro 1.0**<br>`amazon/nova-micro-v1` | nous_portal | $0.0427 | **46.6** | 44.7 | 75 tps | 128k | 🚫 Dominated |
| 360 | **Mistral Large**<br>`mistralai/mistral-large` | nous_portal | $2.2200 | **46.6** | 21.5 | 75 tps | 128k | 🚫 Dominated |
| 361 | **Google: Gemma 3 12B**<br>`google/gemma-3-12b-it` | nous_portal | $0.0560 | **46.6** | 44.2 | 75 tps | 131k | 🚫 Dominated |
| 362 | **Mistral: Mixtral 8x22B Instruct**<br>`mistralai/mixtral-8x22b-instruct` | nous_portal | $2.2200 | **46.6** | 21.5 | 75 tps | 65k | 🚫 Dominated |
| 363 | **Meta: Llama 3.1 8B Instruct**<br>`meta-llama/llama-3.1-8b-instruct` | nous_portal | $0.0204 | **46.5** | 45.5 | 75 tps | 131k | 🚫 Dominated |
| 364 | **DeepSeek: R1 0528**<br>`deepseek/deepseek-r1-0528` | nous_portal | $0.6500 | **46.0** | 30.7 | 75 tps | 163k | 🚫 Dominated |
| 365 | **Claude Opus 4.6**<br>`anthropic/claude-opus-4.6` | nous_portal_batch | $6.5500 | **41.7** | 13.8 | 150 tps | 1000k | 🚫 Dominated |
| 366 | **Anthropic: Claude Opus 4.5**<br>`anthropic/claude-opus-4.5` | nous_portal_batch | $6.5500 | **40.8** | 13.5 | 150 tps | 200k | 🚫 Dominated |
| 367 | **Anthropic: Claude Opus 4.1**<br>`anthropic/claude-opus-4.1` | nous_portal_batch | $19.6500 | **38.9** | 9.7 | 150 tps | 200k | 🚫 Dominated |
| 368 | **Hermes 3 (Llama 3.1 405B)**<br>`hermes-3-llama-3.1-405b` | nous_portal | $0.6100 | **34.3** | 23.2 | 40 tps | 128k | 🚫 Dominated |
| 369 | **Hermes 3 (Llama 3.1 405B)**<br>`hermes-3-llama-3.1-405b` | nous_portal | $0.6100 | **34.3** | 23.2 | 40 tps | 128k | 🚫 Dominated |

#### 🔗 Cadeia de Roteamento Preditivo para Subagentes (`fast`)
| Nível de Escalação | Modelo Designado | Provedor | Custo Blended | Condição de Acionamento |
| :--- | :--- | :--- | :---: | :--- |
| **1. Primário (Custo-Benefício)** | `Xiaomi: MiMo-V2.6-Flash` | nous_portal | $0.1130 | Modelo padrão para subtarefas com SLA normal e orçamento otimizado. |
| **2. Escalação Máxima (Líder)** | `Google: Gemini 3 Flash Preview` | nous_portal_batch | $0.7050 | Acionado se complexidade/entropia for crítica ou falha de teste anterior. |
| **3. Fallback de Baixo Custo** | `DeepSeek: DeepSeek V4 Flash` | openrouter_market | $0.0714 | Standby caso o provedor principal sofra throttling ou indisponibilidade. |

---

### 🎨 2.5 Perfil: Design / Frontend (Multimodal, UI & Coordenadas)
> **Descrição:** Compreensão de capturas de tela, layouting CSS Vanilla, coordenadas visuais e inspeção multimodal de interfaces.  
> **Métricas Primárias:** VQA Multimodal & General ELO &bull; **Fórmula:** `S_design = 0.60·VQAScore + 0.25·GeneralELO + 0.15·IFEval`

| Rank | Modelo Canônico | Provedor | Custo Blended ($/1M) | Score Composto | ROI Index | Throughput | Janela Ctx | Status Pareto |
| :---: | :--- | :---: | :---: | :---: | :---: | :---: | :---: | :--- |
| 1 | **Claude 3.7 Sonnet**<br>`claude-3-7-sonnet` | nous_portal | $5.9650 | **86.2** | 29.3 | 75 tps | 200k | 🏆 **Quality Leader** |
| 2 | **GPT-4o**<br>`gpt-4o` | nous_portal | $2.0938 | **78.7** | 37.0 | 75 tps | 128k | 💎 **Best Value** |
| 3 | **Google: Gemini 3 Flash Preview**<br>`google/gemini-3-flash-preview` | nous_portal_batch | $1.1150 | **77.4** | 44.3 | 150 tps | 1048k | 🏆 **Quality Leader** |
| 4 | **OpenAI o3-mini**<br>`o3-mini` | nous_portal | $0.9213 | **77.4** | 46.8 | 130 tps | 200k | 💎 **Best Value** |
| 5 | **DeepSeek V3**<br>`deepseek-v3` | nous_portal | $0.4630 | **76.9** | 55.7 | 92 tps | 64k | 💎 **Best Value** |
| 6 | **Xiaomi: MiMo-V2.6-Flash**<br>`xiaomi/mimo-v2.6-flash` | nous_portal | $0.1390 | **76.4** | 67.6 | 130 tps | 1050k | 💎 **Best Value** |
| 7 | **GPT-4o Mini**<br>`gpt-4o-mini` | openrouter | $0.1256 | **74.9** | 67.0 | 120 tps | 128k | 💎 **Best Value** |
| 8 | **Qwen 2.5 Coder 32B**<br>`qwen-2.5-coder-32b` | nous_portal | $0.2750 | **74.1** | 59.6 | 105 tps | 131k | 🚫 Dominated |
| 9 | **Xiaomi: MiMo-V2.6-Pro**<br>`xiaomi/mimo-v2.6-pro` | nous_portal | $0.4341 | **74.0** | 54.4 | 75 tps | 1050k | 🚫 Dominated |
| 10 | **DeepSeek R1**<br>`deepseek-r1` | nous_portal | $0.9190 | **73.4** | 44.4 | 75 tps | 64k | 🚫 Dominated |
| 11 | **Google: Gemini 3.8 Flash**<br>`google/gemini-3.8-flash` | nous_portal_batch | $1.4475 | **72.8** | 38.4 | 150 tps | 1048k | 🚫 Dominated |
| 12 | **Z.ai: GLM 5.3 Flash**<br>`z-ai/glm-5.3-flash` | nous_portal_batch | $0.1752 | **70.2** | 60.4 | 150 tps | 1048k | 🚫 Dominated |
| 13 | **Google: Gemini 3.7 Flash**<br>`google/gemini-3.7-flash` | nous_portal_batch | $1.4475 | **70.2** | 37.0 | 150 tps | 1048k | 🚫 Dominated |
| 14 | **OpenAI: GPT-5.6 Luna**<br>`openai/gpt-5.6-luna` | nous_portal_batch | $0.4460 | **70.1** | 51.2 | 150 tps | 1050k | 🚫 Dominated |
| 15 | **Anthropic: Claude Fable 5.1**<br>`anthropic/claude-fable-5.1` | nous_portal_batch | $19.0750 | **69.8** | 17.5 | 150 tps | 1000k | 🚫 Dominated |
| 16 | **Z.ai: GLM 5**<br>`z-ai/glm-5` | nous_portal | $0.8520 | **69.6** | 43.0 | 75 tps | 204k | 🚫 Dominated |
| 17 | **OpenAI: GPT-5.6 Sol**<br>`openai/gpt-5.6-sol` | nous_portal_batch | $3.8600 | **69.1** | 26.8 | 150 tps | 1050k | 🚫 Dominated |
| 18 | **Google: Gemini 3.6 Flash**<br>`google/gemini-3.6-flash` | nous_portal_batch | $1.4475 | **68.9** | 36.4 | 150 tps | 1048k | 🚫 Dominated |
| 19 | **DeepSeek: DeepSeek V4 Flash**<br>`deepseek/deepseek-v4-flash-0731` | openrouter_market | $0.0742 | **68.9** | 64.3 | 130 tps | 1048k | 💎 **Best Value** |
| 20 | **Qwen: Qwen3.5 397B A17B**<br>`qwen/qwen3.5-397b-a17b` | nous_portal | $0.8814 | **68.8** | 42.1 | 75 tps | 262k | 🚫 Dominated |
| 21 | **Z.ai: GLM 5V Turbo**<br>`z-ai/glm-5v-turbo` | nous_portal | $1.7520 | **68.7** | 34.1 | 75 tps | 202k | 🚫 Dominated |
| 22 | **Tencent: Hy3**<br>`tencent/hy3` | nous_portal | $0.2211 | **68.7** | 57.3 | 75 tps | 262k | 🚫 Dominated |
| 23 | **Google: Gemini 3.1 Pro Preview**<br>`google/gemini-3.1-pro-preview` | nous_portal_batch | $4.4600 | **68.7** | 25.5 | 150 tps | 1048k | 🚫 Dominated |
| 24 | **Z.ai: GLM 4.5**<br>`z-ai/glm-4.5` | nous_portal | $0.9330 | **68.0** | 41.0 | 75 tps | 131k | 🚫 Dominated |
| 25 | **xAI: Grok 4.6**<br>`x-ai/grok-4.6` | nous_portal | $2.7500 | **67.2** | 28.9 | 75 tps | 500k | 🚫 Dominated |
| 26 | **OpenAI: GPT-5.6 Terra**<br>`openai/gpt-5.6-terra` | nous_portal_batch | $4.4600 | **67.1** | 24.9 | 150 tps | 1050k | 🚫 Dominated |
| 27 | **Claude Fable 5**<br>`anthropic/claude-fable-5` | nous_portal_batch | $19.3000 | **67.0** | 16.7 | 150 tps | 1048k | 🚫 Dominated |
| 28 | **MoonshotAI: Kimi K3**<br>`moonshotai/kimi-k3` | nous_portal | $4.8760 | **66.9** | 24.1 | 75 tps | 1048k | 🚫 Dominated |
| 29 | **Qwen: Qwen3.8 Max (0902)**<br>`qwen/qwen3.8-max-0902` | nous_portal | $2.6750 | **66.9** | 29.1 | 75 tps | 1000k | 🚫 Dominated |
| 30 | **Meta: Muse Spark 1.3**<br>`meta/muse-spark-1.3` | nous_portal | $1.8200 | **66.7** | 32.7 | 75 tps | 1048k | 🚫 Dominated |
| 31 | **DeepSeek: DeepSeek V4 Flash Vision Exp**<br>`deepseek/deepseek-v4-flash-vision-exp` | nous_portal | $0.2823 | **66.6** | 53.3 | 130 tps | 1048k | 🚫 Dominated |
| 32 | **OpenAI: GPT-5.5**<br>`openai/gpt-5.5` | nous_portal_batch | $11.1500 | **66.2** | 18.9 | 150 tps | 1050k | 🚫 Dominated |
| 33 | **Z.ai: GLM 5.3**<br>`z-ai/glm-5.3` | openrouter_market | $1.2727 | **66.1** | 36.3 | 70 tps | 1048k | 🚫 Dominated |
| 34 | **Llama 3.3 70B Instruct**<br>`llama-3.3-70b` | nous_portal | $0.5172 | **65.3** | 46.1 | 75 tps | 128k | 🚫 Dominated |
| 35 | **xAI: Grok 4.5**<br>`x-ai/grok-4.5` | nous_portal | $2.6900 | **64.8** | 28.1 | 75 tps | 500k | 🚫 Dominated |
| 36 | **Meta: Muse Spark 1.2**<br>`meta/muse-spark-1.2` | nous_portal | $1.8200 | **64.7** | 31.8 | 75 tps | 1048k | 🚫 Dominated |
| 37 | **Qwen: Qwen3.8 2.4T A95B**<br>`qwen/qwen3.8-2.4t-a95b` | nous_portal | $2.6750 | **64.5** | 28.0 | 75 tps | 1048k | 🚫 Dominated |
| 38 | **Claude Sonnet 5**<br>`anthropic/claude-sonnet-5` | nous_portal_batch | $3.8600 | **64.3** | 24.9 | 150 tps | 1000k | 🚫 Dominated |
| 39 | **Meta: Muse Spark 1.1**<br>`meta/muse-spark-1.1` | nous_portal | $1.8200 | **64.2** | 31.5 | 75 tps | 1048k | 🚫 Dominated |
| 40 | **Anthropic: Claude Opus 5**<br>`anthropic/claude-opus-5` | nous_portal_batch | $9.6500 | **64.1** | 19.1 | 150 tps | 1000k | 🚫 Dominated |
| 41 | **Hermes 3 (Llama 3.1 70B)**<br>`hermes-3-llama-3.1-70b` | nous_portal | $0.4370 | **64.1** | 47.0 | 78 tps | 128k | 🚫 Dominated |
| 42 | **OpenAI: GPT-5.4**<br>`openai/gpt-5.4` | nous_portal_batch | $5.5750 | **64.1** | 22.2 | 150 tps | 1050k | 🚫 Dominated |
| 43 | **OpenAI: GPT-6 Astra**<br>`openai/gpt-6-astra` | nous_portal_batch | $19.3000 | **63.5** | 15.8 | 150 tps | 1050k | 🚫 Dominated |
| 44 | **MiniMax: MiniMax M3**<br>`minimax/minimax-m3` | nous_portal | $0.3950 | **63.1** | 47.3 | 130 tps | 1048k | 🚫 Dominated |
| 45 | **Hermes 3 (Llama 3.1 405B)**<br>`hermes-3-llama-3.1-405b` | nous_portal | $0.7750 | **63.1** | 40.1 | 40 tps | 128k | 🚫 Dominated |
| 46 | **Z.ai: GLM 5.2**<br>`z-ai/glm-5.2` | openrouter_market | $1.2570 | **62.8** | 34.6 | 70 tps | 1048k | 🚫 Dominated |
| 47 | **DeepSeek: DeepSeek V4 Pro 0813**<br>`deepseek/deepseek-v4-pro-0813` | nous_portal | $2.5260 | **62.8** | 27.8 | 75 tps | 1048k | 🚫 Dominated |
| 48 | **Qwen: Qwen3.8 27B**<br>`qwen/qwen3.8-27b` | nous_portal | $0.3918 | **62.4** | 46.9 | 75 tps | 1000k | 🚫 Dominated |
| 49 | **inclusionAI: Ling 3.0 Flash VL**<br>`inclusionai/ling-3.0-flash-vl` | nous_portal | $0.0816 | **62.2** | 57.7 | 130 tps | 131k | 🚫 Dominated |
| 50 | **Claude Opus 4.8**<br>`anthropic/claude-opus-4.8` | nous_portal_batch | $9.6500 | **62.1** | 18.4 | 150 tps | 1000k | 🚫 Dominated |
| 51 | **OpenAI: GPT-5.4 Mini**<br>`openai/gpt-5.4-mini` | nous_portal_batch | $1.6725 | **61.7** | 31.1 | 150 tps | 400k | 🚫 Dominated |
| 52 | **Claude Opus 4.7**<br>`anthropic/claude-opus-4.7` | nous_portal_batch | $9.6500 | **61.7** | 18.3 | 150 tps | 1000k | 🚫 Dominated |
| 53 | **Qwen: Qwen3.7 Max**<br>`qwen/qwen3.7-max` | nous_portal | $2.0060 | **61.3** | 29.2 | 75 tps | 1000k | 🚫 Dominated |
| 54 | **Google: Gemini 4 Argon**<br>`google/gemini-4-argon` | nous_portal_batch | $3.8300 | **59.8** | 23.2 | 150 tps | 1048k | 🚫 Dominated |
| 55 | **MiniMax: MiniMax M2.7**<br>`minimax/minimax-m2.7` | nous_portal | $0.3486 | **59.8** | 46.0 | 130 tps | 204k | 🚫 Dominated |
| 56 | **Claude Sonnet 4.6**<br>`anthropic/claude-sonnet-4.6` | nous_portal_batch | $5.7900 | **59.6** | 20.4 | 150 tps | 1000k | 🚫 Dominated |
| 57 | **DeepSeek: DeepSeek V4 Flash 0423**<br>`deepseek/deepseek-v4-flash` | nous_portal | $0.3657 | **59.5** | 45.3 | 130 tps | 1048k | 🚫 Dominated |
| 58 | **MoonshotAI: Kimi K2.6**<br>`moonshotai/kimi-k2.6` | nous_portal | $0.9411 | **59.0** | 35.4 | 75 tps | 262k | 🚫 Dominated |
| 59 | **inclusionAI: Ling 3.0 Flash**<br>`inclusionai/ling-3.0-flash` | nous_portal | $0.0286 | **58.7** | 57.1 | 130 tps | 262k | 💎 **Best Value** |
| 60 | **MoonshotAI: Kimi K2.7 Code**<br>`moonshotai/kimi-k2.7-code` | nous_portal | $1.3275 | **58.4** | 31.7 | 75 tps | 262k | 🚫 Dominated |
| 61 | **Xiaomi: MiMo-V2.5-Pro**<br>`xiaomi/mimo-v2.5-pro` | nous_portal | $0.3053 | **58.1** | 45.9 | 75 tps | 1050k | 🚫 Dominated |
| 62 | **Google: Gemini 3.5 Flash Lite**<br>`google/gemini-3.5-flash-lite` | nous_portal_batch | $0.8790 | **58.0** | 35.5 | 150 tps | 1048k | 🚫 Dominated |
| 63 | **DeepSeek: DeepSeek V4 Pro 0423**<br>`deepseek/deepseek-v4-pro` | nous_portal | $0.9670 | **57.6** | 34.4 | 75 tps | 1048k | 🚫 Dominated |
| 64 | **Tencent: Hy3 preview**<br>`tencent/hy3-preview` | nous_portal | $0.2700 | **57.3** | 46.2 | 75 tps | 262k | 🚫 Dominated |
| 65 | **Xiaomi: MiMo-V2.5**<br>`xiaomi/mimo-v2.5` | nous_portal | $0.1198 | **56.2** | 50.5 | 75 tps | 1050k | 🚫 Dominated |
| 66 | **OpenAI: GPT-5.4 Nano**<br>`openai/gpt-5.4-nano` | nous_portal_batch | $0.4610 | **55.8** | 40.5 | 150 tps | 400k | 🚫 Dominated |
| 67 | **Anthropic: Claude Sonnet 5.5**<br>`anthropic/claude-sonnet-5.5` | nous_portal_batch | $3.8300 | **55.8** | 21.7 | 150 tps | 1000k | 🚫 Dominated |
| 68 | **Qwen: Qwen3.7 Plus**<br>`qwen/qwen3.7-plus` | nous_portal | $0.5312 | **55.7** | 39.1 | 75 tps | 1000k | 🚫 Dominated |
| 69 | **Z.ai: GLM 5.1**<br>`z-ai/glm-5.1` | nous_portal | $1.3510 | **55.7** | 30.0 | 75 tps | 204k | 🚫 Dominated |
| 70 | **Upstage: Solar Pro 4**<br>`upstage/solar-pro4` | nous_portal | $0.1494 | **55.6** | 48.8 | 90 tps | 524k | 🚫 Dominated |
| 71 | **OpenAI: GPT-6.1 Sol**<br>`openai/gpt-6.1-sol` | nous_portal_batch | $3.8300 | **55.1** | 21.4 | 150 tps | 1050k | 🚫 Dominated |
| 72 | **AionLabs: Aion 3.5 Mini**<br>`aion-labs/aion-3.5-mini` | nous_portal | $0.7540 | **55.1** | 35.3 | 130 tps | 262k | 🚫 Dominated |
| 73 | **AionLabs: Aion-3.0-Mini**<br>`aion-labs/aion-3.0-mini` | nous_portal | $0.7540 | **55.1** | 35.3 | 130 tps | 131k | 🚫 Dominated |
| 74 | **DeepSeek: DeepSeek V4 Flash Latest**<br>`deepseek/deepseek-v4-flash-latest` | nous_portal | $0.1818 | **55.1** | 47.2 | 130 tps | 1048k | 🚫 Dominated |
| 75 | **Google: Gemini Embedding 001**<br>`google/gemini-embedding-001` | nous_portal | $0.0690 | **55.1** | 51.6 | 130 tps | 20k | 🚫 Dominated |
| 76 | **Sentence Transformers: all-MiniLM-L12-v2**<br>`sentence-transformers/all-minilm-l12-v2` | nous_portal | $0.0023 | **55.1** | 54.9 | 130 tps | 0k | 💎 **Best Value** |
| 77 | **DeepSeek: DeepSeek V4 Flash**<br>`deepseek/deepseek-v4-flash-0731:US` | nous_portal | $0.4753 | **55.1** | 39.6 | 130 tps | 1048k | 🚫 Dominated |
| 78 | **Qwen: Qwen3 Coder Flash**<br>`qwen/qwen3-coder-flash` | nous_portal | $0.3822 | **55.1** | 41.6 | 130 tps | 1000k | 🚫 Dominated |
| 79 | **Sentence Transformers: paraphrase-MiniLM-L6-v2**<br>`sentence-transformers/paraphrase-minilm-l6-v2` | nous_portal | $0.0023 | **55.1** | 54.9 | 130 tps | 0k | 💎 **Best Value** |
| 80 | **Sentence Transformers: all-MiniLM-L6-v2**<br>`sentence-transformers/all-minilm-l6-v2` | nous_portal | $0.0023 | **55.1** | 54.9 | 130 tps | 0k | 💎 **Best Value** |
| 81 | **Claude Haiku 4.5**<br>`anthropic/claude-haiku-4.5` | nous_portal_batch | $1.9300 | **55.0** | 26.5 | 150 tps | 409k | 🚫 Dominated |
| 82 | **Qwen: Qwen3.6 Plus**<br>`qwen/qwen3.6-plus` | nous_portal | $0.7345 | **54.9** | 35.4 | 75 tps | 1000k | 🚫 Dominated |
| 83 | **Anthropic: Claude Haiku 5.5**<br>`anthropic/claude-haiku-5.5` | nous_portal_batch | $0.1930 | **54.7** | 46.5 | 150 tps | 1000k | 🚫 Dominated |
| 84 | **Qwen: Qwen3.6 27B**<br>`qwen/qwen3.6-27b` | nous_portal | $0.7075 | **54.5** | 35.5 | 75 tps | 262k | 🚫 Dominated |
| 85 | **Thinking Machines: Inkling Small**<br>`thinkingmachines/inkling-small` | openrouter_market | $0.5700 | **54.1** | 37.3 | 70 tps | 524k | 🚫 Dominated |
| 86 | **Thinking Machines: Inkling**<br>`thinkingmachines/inkling` | openrouter_market | $1.6430 | **53.6** | 27.2 | 70 tps | 524k | 🚫 Dominated |
| 87 | **Anthropic: Claude Sonnet 4.5**<br>`anthropic/claude-sonnet-4.5` | nous_portal_batch | $5.7900 | **53.6** | 18.4 | 150 tps | 1000k | 🚫 Dominated |
| 88 | **inclusionAI: Ling 3.1 Flash**<br>`inclusionai/ling-3.1-flash` | nous_portal | $0.1020 | **53.5** | 48.7 | 130 tps | 262k | 🚫 Dominated |
| 89 | **SpaceXAI: Grok Build 0.1**<br>`x-ai/grok-build-0.1` | nous_portal | $1.0600 | **53.3** | 30.9 | 75 tps | 256k | 🚫 Dominated |
| 90 | **Anthropic: Claude Opus 5.5**<br>`anthropic/claude-opus-5.5` | nous_portal_batch | $7.6600 | **52.9** | 16.7 | 150 tps | 1000k | 🚫 Dominated |
| 91 | **OpenAI: GPT-6 Sol**<br>`openai/gpt-6-sol` | nous_portal_batch | $3.8600 | **52.8** | 20.4 | 150 tps | 1050k | 🚫 Dominated |
| 92 | **StepFun: Step 3.7 Flash**<br>`stepfun/step-3.7-flash` | nous_portal | $0.4370 | **52.6** | 38.6 | 130 tps | 262k | 🚫 Dominated |
| 93 | **DeepSeek: DeepSeek V4.1 Flash**<br>`deepseek/deepseek-v4.1-flash` | openrouter_market | $0.1975 | **52.6** | 44.6 | 130 tps | 1048k | 🚫 Dominated |
| 94 | **OpenAI: GPT Sol Latest**<br>`openai/gpt-sol-latest` | nous_portal | $3.8300 | **52.6** | 20.4 | 90 tps | 1050k | 🚫 Dominated |
| 95 | **OpenAI: GPT-5.6 Sol Pro**<br>`openai/gpt-5.6-sol-pro` | nous_portal_batch | $3.8600 | **52.6** | 20.4 | 150 tps | 1050k | 🚫 Dominated |
| 96 | **OpenAI: GPT-6 Sol Pro**<br>`openai/gpt-6-sol-pro` | nous_portal_batch | $3.8600 | **52.6** | 20.4 | 150 tps | 1050k | 🚫 Dominated |
| 97 | **OpenAI: GPT-6.1 Sol Pro**<br>`openai/gpt-6.1-sol-pro` | nous_portal_batch | $3.8300 | **52.6** | 20.4 | 150 tps | 1050k | 🚫 Dominated |
| 98 | **Z.ai: GLM 5.3 Prime**<br>`z-ai/glm-5.3-prime` | nous_portal | $3.9280 | **52.5** | 20.2 | 75 tps | 1000k | 🚫 Dominated |
| 99 | **OpenAI: GPT-5.1**<br>`openai/gpt-5.1` | nous_portal_batch | $3.5390 | **52.1** | 20.7 | 150 tps | 400k | 🚫 Dominated |
| 100 | **NVIDIA: Nemotron 3 Ultra**<br>`nvidia/nemotron-3-ultra-550b-a55b` | openrouter_market | $0.8900 | **52.1** | 31.8 | 70 tps | 262k | 🚫 Dominated |
| 101 | **Meta: Muse Glimmer 30B**<br>`meta/muse-glimmer-30b` | nous_portal | $0.4920 | **51.9** | 37.1 | 75 tps | 131k | 🚫 Dominated |
| 102 | **OpenAI: GPT-6 Luna**<br>`openai/gpt-6-luna` | nous_portal_batch | $0.1930 | **51.8** | 44.0 | 150 tps | 1050k | 🚫 Dominated |
| 103 | **Gemini 2.0 Flash**<br>`gemini-2.0-flash` | nous_portal | $0.8300 | **51.8** | 32.3 | 130 tps | 1000k | 🏆 **Quality Leader** |
| 104 | **Gemini 2.0 Flash**<br>`gemini-2.0-flash` | nous_portal | $0.8300 | **51.8** | 32.3 | 130 tps | 1000k | 🏆 **Quality Leader** |
| 105 | **Google: Nano Banana 2.1**<br>`google/gemini-nano-banana-2.1` | nous_portal | $2.9400 | **51.8** | 21.8 | 130 tps | 65k | 🚫 Dominated |
| 106 | **ByteDance Seed: Seed 1.6 Flash**<br>`bytedance-seed/seed-1.6-flash` | nous_portal | $0.1245 | **51.8** | 46.3 | 130 tps | 262k | 🚫 Dominated |
| 107 | **Anthropic: Claude Haiku Latest**<br>`anthropic/claude-haiku-latest` | nous_portal | $0.1930 | **51.8** | 44.0 | 130 tps | 1000k | 🚫 Dominated |
| 108 | **Google: Gemini Embedding 2 Preview**<br>`google/gemini-embedding-2-preview` | nous_portal | $0.0920 | **51.8** | 47.6 | 130 tps | 8k | 🚫 Dominated |
| 109 | **Google: Gemini 3.1 Pro Preview Custom Tools**<br>`google/gemini-3.1-pro-preview-customtools` | nous_portal | $4.4600 | **51.8** | 19.2 | 130 tps | 1048k | 🚫 Dominated |
| 110 | **Google: Nano Banana Pro (Gemini 3 Pro Image)**<br>`google/gemini-3-pro-image` | nous_portal | $4.4600 | **51.8** | 19.2 | 130 tps | 131k | 🚫 Dominated |
| 111 | **Qwen: Qwen3.8 Flash**<br>`qwen/qwen3.8-flash` | nous_portal | $0.2058 | **51.8** | 43.6 | 130 tps | 1000k | 🚫 Dominated |
| 112 | **Z.ai: GLM 5.3 FlashX**<br>`z-ai/glm-5.3-flashx` | nous_portal | $0.5500 | **51.8** | 36.0 | 130 tps | 1048k | 🚫 Dominated |
| 113 | **Google: Gemini 3.1 Flash Lite**<br>`google/gemini-3.1-flash-lite` | nous_portal_batch | $0.5575 | **51.8** | 35.9 | 150 tps | 1048k | 🚫 Dominated |
| 114 | **OpenAI: GPT Mini Latest**<br>`openai/gpt-mini-latest` | nous_portal | $1.6725 | **51.8** | 26.1 | 130 tps | 400k | 🚫 Dominated |
| 115 | **OpenAI: GPT Audio Mini**<br>`openai/gpt-audio-mini` | nous_portal | $0.9960 | **51.8** | 30.6 | 130 tps | 128k | 🚫 Dominated |
| 116 | **Google: Gemini Embedding 2**<br>`google/gemini-embedding-2` | nous_portal_batch | $0.0920 | **51.8** | 47.6 | 150 tps | 8k | 🚫 Dominated |
| 117 | **OpenAI: GPT-4o-mini (2024-07-18)**<br>`openai/gpt-4o-mini-2024-07-18` | nous_portal | $0.2625 | **51.8** | 42.0 | 130 tps | 128k | 🚫 Dominated |
| 118 | **OpenAI: GPT-5.6 Luna Pro**<br>`openai/gpt-5.6-luna-pro` | nous_portal_batch | $0.4460 | **51.8** | 37.8 | 150 tps | 1050k | 🚫 Dominated |
| 119 | **Qwen: Qwen3.5-Flash**<br>`qwen/qwen3.5-flash-02-23` | nous_portal | $0.1079 | **51.8** | 46.9 | 130 tps | 1000k | 🚫 Dominated |
| 120 | **DeepSeek: DeepSeek Flash Latest**<br>`deepseek/deepseek-flash-latest` | openrouter_market | $0.1861 | **51.8** | 44.2 | 130 tps | 1048k | 🚫 Dominated |
| 121 | **DeepSeek: DeepSeek V4.1 Flash**<br>`deepseek/deepseek-v4.1-flash:US` | nous_portal | $0.4753 | **51.8** | 37.3 | 130 tps | 1048k | 🚫 Dominated |
| 122 | **Google: Gemini Flash Latest**<br>`google/gemini-flash-latest` | nous_portal | $1.4475 | **51.8** | 27.3 | 130 tps | 1048k | 🚫 Dominated |
| 123 | **OpenAI: GPT Luna Latest**<br>`openai/gpt-luna-latest` | nous_portal | $0.1930 | **51.8** | 44.0 | 130 tps | 1050k | 🚫 Dominated |
| 124 | **Z.ai: GLM Flash Latest**<br>`z-ai/glm-flash-latest` | nous_portal | $0.2876 | **51.8** | 41.3 | 130 tps | 1048k | 🚫 Dominated |
| 125 | **ByteDance Seed: Seed-2.0-Mini**<br>`bytedance-seed/seed-2.0-mini` | nous_portal | $0.1660 | **51.8** | 44.9 | 130 tps | 262k | 🚫 Dominated |
| 126 | **Qwen: Qwen3.6 Flash**<br>`qwen/qwen3.6-flash` | nous_portal | $0.4237 | **51.8** | 38.2 | 130 tps | 1000k | 🚫 Dominated |
| 127 | **Google: Gemini Pro Latest**<br>`google/gemini-pro-latest` | nous_portal | $4.4600 | **51.8** | 19.2 | 130 tps | 1048k | 🚫 Dominated |
| 128 | **Z.ai: GLM 5.3 Flash**<br>`z-ai/glm-5.3-flash:US` | nous_portal | $0.3614 | **51.8** | 39.6 | 130 tps | 1048k | 🚫 Dominated |
| 129 | **Qwen: Qwen3.8 Omni Flash**<br>`qwen/qwen3.8-omni-flash` | nous_portal | $0.2058 | **51.8** | 43.6 | 130 tps | 1000k | 🚫 Dominated |
| 130 | **Qwen: Qwen3.7 Flash**<br>`qwen/qwen3.7-flash` | nous_portal | $0.0528 | **51.8** | 49.2 | 130 tps | 1000k | 🚫 Dominated |
| 131 | **OpenAI: GPT-6 Luna Pro**<br>`openai/gpt-6-luna-pro` | nous_portal_batch | $0.1930 | **51.8** | 44.0 | 150 tps | 1050k | 🚫 Dominated |
| 132 | **Google: Gemini 2.5 Pro Preview 06-05**<br>`google/gemini-2.5-pro-preview` | nous_portal | $3.5375 | **51.8** | 20.6 | 130 tps | 1048k | 🚫 Dominated |
| 133 | **OpenAI: o4 Mini High**<br>`openai/o4-mini-high` | nous_portal | $1.8425 | **51.8** | 25.3 | 130 tps | 200k | 🚫 Dominated |
| 134 | **Claude 3.5 Sonnet**<br>`claude-3-5-sonnet` | nous_portal | $4.9800 | **51.0** | 18.3 | 75 tps | 200k | 🚫 Dominated |
| 135 | **Claude 3.7 Sonnet**<br>`claude-3-7-sonnet` | nous_portal | $4.9800 | **51.0** | 18.3 | 75 tps | 200k | 🚫 Dominated |
| 136 | **Claude 3.7 Sonnet**<br>`claude-3-7-sonnet` | nous_portal | $4.9800 | **51.0** | 18.3 | 75 tps | 200k | 🚫 Dominated |
| 137 | **Claude 3.5 Sonnet**<br>`claude-3-5-sonnet` | nous_portal | $4.9800 | **51.0** | 18.3 | 75 tps | 200k | 🚫 Dominated |
| 138 | **Anthropic: Claude Sonnet Latest**<br>`anthropic/claude-sonnet-latest` | nous_portal | $3.8300 | **51.0** | 19.8 | 75 tps | 1000k | 🚫 Dominated |
| 139 | **Qwen: Qwen3.8 Max Prime**<br>`qwen/qwen3.8-max-prime` | nous_portal | $5.3500 | **51.0** | 17.9 | 75 tps | 1000k | 🚫 Dominated |
| 140 | **Mistral: Mistral Medium 3.5**<br>`mistralai/mistral-medium-3-5` | nous_portal_batch | $2.9400 | **50.8** | 21.4 | 150 tps | 262k | 🚫 Dominated |
| 141 | **MoonshotAI: Kimi K2.5**<br>`moonshotai/kimi-k2.5` | nous_portal | $1.0060 | **50.7** | 29.9 | 75 tps | 262k | 🚫 Dominated |
| 142 | **xAI: Grok 4.7**<br>`x-ai/grok-4.7` | nous_portal | $2.7500 | **50.5** | 21.7 | 75 tps | 500k | 🚫 Dominated |
| 143 | **Z.ai: GLM 4.6**<br>`z-ai/glm-4.6` | nous_portal | $0.7210 | **50.2** | 32.5 | 75 tps | 204k | 🚫 Dominated |
| 144 | **Google: Gemini 3.1 Flash Lite Preview**<br>`google/gemini-3.1-flash-lite-preview` | nous_portal | $0.5575 | **49.9** | 34.6 | 130 tps | 1048k | 🚫 Dominated |
| 145 | **Z.ai: GLM 4.7**<br>`z-ai/glm-4.7` | nous_portal | $0.7090 | **49.9** | 32.5 | 75 tps | 204k | 🚫 Dominated |
| 146 | **Meituan: LongCat 2.0**<br>`meituan/longcat-2.0` | nous_portal | $0.4818 | **49.9** | 35.8 | 75 tps | 1048k | 🚫 Dominated |
| 147 | **OpenAI: GPT Astra Latest**<br>`openai/gpt-astra-latest` | nous_portal | $19.3000 | **49.8** | 12.4 | 40 tps | 1050k | 🚫 Dominated |
| 148 | **OpenAI: GPT-6 Astra Pro Flex**<br>`openai/gpt-6-astra-pro-flex` | nous_portal | $9.6500 | **49.8** | 14.8 | 40 tps | 1050k | 🚫 Dominated |
| 149 | **OpenAI: GPT-6 Astra Pro Fast**<br>`openai/gpt-6-astra-pro-fast` | nous_portal | $38.6000 | **49.8** | 10.6 | 40 tps | 1050k | 🚫 Dominated |
| 150 | **OpenAI: GPT-6 Astra Fast**<br>`openai/gpt-6-astra-fast` | nous_portal | $38.6000 | **49.8** | 10.6 | 40 tps | 1050k | 🚫 Dominated |
| 151 | **OpenAI: GPT-6 Astra Flex**<br>`openai/gpt-6-astra-flex` | nous_portal | $9.6500 | **49.8** | 14.8 | 40 tps | 1050k | 🚫 Dominated |
| 152 | **Anthropic: Claude Opus Latest**<br>`anthropic/claude-opus-latest` | nous_portal | $7.6600 | **49.8** | 15.8 | 40 tps | 1000k | 🚫 Dominated |
| 153 | **OpenAI: GPT-6 Astra Pro**<br>`openai/gpt-6-astra-pro` | nous_portal_batch | $19.3000 | **49.8** | 12.4 | 150 tps | 1050k | 🚫 Dominated |
| 154 | **DeepSeek: DeepSeek Pro Latest**<br>`deepseek/deepseek-pro-latest` | nous_portal | $2.5260 | **49.7** | 22.0 | 75 tps | 1048k | 🚫 Dominated |
| 155 | **Z.ai: GLM Latest**<br>`z-ai/glm-latest` | nous_portal | $1.3628 | **49.7** | 26.7 | 75 tps | 1048k | 🚫 Dominated |
| 156 | **Poolside: Laguna S 2.1**<br>`poolside/laguna-s-2.1` | openrouter_market | $0.0927 | **49.7** | 45.7 | 70 tps | 1048k | 🚫 Dominated |
| 157 | **Qwen: Qwen-Plus**<br>`qwen/qwen-plus` | nous_portal | $0.3536 | **49.7** | 38.2 | 75 tps | 1000k | 🚫 Dominated |
| 158 | **Tencent: Hy4 preview**<br>`tencent/hy4-preview` | nous_portal | $0.5481 | **49.7** | 34.6 | 75 tps | 1048k | 🚫 Dominated |
| 159 | **Z.ai: GLM 5.3**<br>`z-ai/glm-5.3:US` | nous_portal | $3.2307 | **49.7** | 20.4 | 75 tps | 1048k | 🚫 Dominated |
| 160 | **Google: Gemini 3.5 Flash**<br>`google/gemini-3.5-flash` | nous_portal_batch | $3.3450 | **49.3** | 20.0 | 150 tps | 1048k | 🚫 Dominated |
| 161 | **DeepSeek: DeepSeek V3.2**<br>`deepseek/deepseek-v3.2` | nous_portal | $0.1829 | **49.3** | 42.2 | 75 tps | 163k | 🚫 Dominated |
| 162 | **Google: Gemini 2.5 Pro**<br>`google/gemini-2.5-pro` | nous_portal_batch | $3.5375 | **49.2** | 19.6 | 150 tps | 1048k | 🚫 Dominated |
| 163 | **DeepSeek: DeepSeek V3.1 Terminus**<br>`deepseek/deepseek-v3.1-terminus` | nous_portal | $0.4242 | **48.9** | 36.1 | 75 tps | 163k | 🚫 Dominated |
| 164 | **Google: Gemma 4 31B**<br>`google/gemma-4-31b-it` | openrouter_market | $0.1530 | **48.8** | 42.7 | 70 tps | 262k | 🚫 Dominated |
| 165 | **Qwen: Qwen3.5-122B-A10B**<br>`qwen/qwen3.5-122b-a10b` | nous_portal | $0.7436 | **48.8** | 31.4 | 75 tps | 262k | 🚫 Dominated |
| 166 | **SpaceXAI: Grok 4.3**<br>`x-ai/grok-4.3` | nous_portal_batch | $1.3100 | **48.2** | 26.2 | 150 tps | 1000k | 🚫 Dominated |
| 167 | **Qwen: Qwen3.6 35B A3B**<br>`qwen/qwen3.6-35b-a3b` | nous_portal | $0.2375 | **48.0** | 39.6 | 75 tps | 262k | 🚫 Dominated |
| 168 | **DeepSeek V3**<br>`deepseek-v3` | nous_portal | $0.8300 | **47.0** | 29.3 | 92 tps | 64k | 🚫 Dominated |
| 169 | **DeepSeek V3**<br>`deepseek-v3` | nous_portal | $0.8300 | **47.0** | 29.3 | 92 tps | 64k | 🚫 Dominated |
| 170 | **Qwen: Qwen3 Embedding 8B**<br>`qwen/qwen3-embedding-8b` | nous_portal | $0.0046 | **47.0** | 46.7 | 75 tps | 32k | 🚫 Dominated |
| 171 | **Intfloat: E5-Base-v2**<br>`intfloat/e5-base-v2` | nous_portal | $0.0023 | **47.0** | 46.9 | 75 tps | 0k | 🚫 Dominated |
| 172 | **OpenAI: Text Embedding Ada 002**<br>`openai/text-embedding-ada-002` | nous_portal_batch | $0.0460 | **47.0** | 44.9 | 150 tps | 8k | 🚫 Dominated |
| 173 | **Sentence Transformers: multi-qa-mpnet-base-dot-v1**<br>`sentence-transformers/multi-qa-mpnet-base-dot-v1` | nous_portal | $0.0023 | **47.0** | 46.9 | 75 tps | 0k | 🚫 Dominated |
| 174 | **BAAI: bge-base-en-v1.5**<br>`baai/bge-base-en-v1.5` | nous_portal | $0.0023 | **47.0** | 46.9 | 75 tps | 0k | 🚫 Dominated |
| 175 | **OpenAI: gpt-oss-safeguard-20b**<br>`openai/gpt-oss-safeguard-20b` | nous_portal | $0.1312 | **47.0** | 41.8 | 75 tps | 131k | 🚫 Dominated |
| 176 | **Intfloat: E5-Large-v2**<br>`intfloat/e5-large-v2` | nous_portal | $0.0046 | **47.0** | 46.7 | 75 tps | 0k | 🚫 Dominated |
| 177 | **Qwen: Qwen2.5 7B Instruct**<br>`qwen/qwen-2.5-7b-instruct` | nous_portal | $0.1060 | **47.0** | 42.7 | 75 tps | 32k | 🚫 Dominated |
| 178 | **Sentence Transformers: all-mpnet-base-v2**<br>`sentence-transformers/all-mpnet-base-v2` | nous_portal | $0.0023 | **47.0** | 46.9 | 75 tps | 0k | 🚫 Dominated |
| 179 | **Poolside: Laguna XS 2.1**<br>`poolside/laguna-xs-2.1` | openrouter_market | $0.0690 | **47.0** | 44.0 | 70 tps | 262k | 🚫 Dominated |
| 180 | **Perplexity: Embed V1 4B**<br>`perplexity/pplx-embed-v1-4b` | nous_portal | $0.0138 | **47.0** | 46.3 | 75 tps | 32k | 🚫 Dominated |
| 181 | **OpenAI: Text Embedding 3 Large**<br>`openai/text-embedding-3-large` | nous_portal_batch | $0.0598 | **47.0** | 44.4 | 150 tps | 8k | 🚫 Dominated |
| 182 | **Perplexity: Embed V1 0.6B**<br>`perplexity/pplx-embed-v1-0.6b` | nous_portal | $0.0018 | **47.0** | 46.9 | 75 tps | 32k | 🚫 Dominated |
| 183 | **Intfloat: Multilingual-E5-Large**<br>`intfloat/multilingual-e5-large` | nous_portal | $0.0046 | **47.0** | 46.7 | 75 tps | 0k | 🚫 Dominated |
| 184 | **OpenAI: GPT-3.5 Turbo 16k**<br>`openai/gpt-3.5-turbo-16k` | nous_portal | $2.5800 | **47.0** | 20.6 | 75 tps | 16k | 🚫 Dominated |
| 185 | **AionLabs: Aion-2.0**<br>`aion-labs/aion-2.0` | nous_portal | $0.8600 | **47.0** | 29.0 | 75 tps | 131k | 🚫 Dominated |
| 186 | **BAAI: bge-large-en-v1.5**<br>`baai/bge-large-en-v1.5` | nous_portal | $0.0046 | **47.0** | 46.7 | 75 tps | 0k | 🚫 Dominated |
| 187 | **VoyageAI by MongoDB: voyage-4-lite**<br>`voyageai/voyage-4-lite` | nous_portal | $0.0092 | **47.0** | 46.5 | 75 tps | 32k | 🚫 Dominated |
| 188 | **MissingNo**<br>`stealth/missingno` | nous_portal | $0.0010 | **47.0** | 46.9 | 75 tps | 262k | 💎 **Best Value** |
| 189 | **VoyageAI by MongoDB: voyage-code-4**<br>`voyageai/voyage-code-4` | nous_portal | $0.0552 | **47.0** | 44.6 | 75 tps | 32k | 🚫 Dominated |
| 190 | **Cohere: Command R+ (08-2024)**<br>`cohere/command-r-plus-08-2024` | nous_portal | $4.1500 | **47.0** | 17.8 | 75 tps | 128k | 🚫 Dominated |
| 191 | **BAAI: bge-m3**<br>`baai/bge-m3` | nous_portal | $0.0046 | **47.0** | 46.7 | 75 tps | 8k | 🚫 Dominated |
| 192 | **OpenAI: Text Embedding 3 Small**<br>`openai/text-embedding-3-small` | nous_portal | $0.0092 | **47.0** | 46.5 | 75 tps | 8k | 🚫 Dominated |
| 193 | **Thenlper: GTE-Large**<br>`thenlper/gte-large` | nous_portal | $0.0046 | **47.0** | 46.7 | 75 tps | 0k | 🚫 Dominated |
| 194 | **DeepSeek: DeepSeek V3**<br>`deepseek/deepseek-chat` | nous_portal | $0.4270 | **47.0** | 34.6 | 75 tps | 163k | 🚫 Dominated |
| 195 | **VoyageAI by MongoDB: voyage-4-large**<br>`voyageai/voyage-4-large` | nous_portal | $0.0552 | **47.0** | 44.6 | 75 tps | 32k | 🚫 Dominated |
| 196 | **Mistral: Codestral Embed 2505**<br>`mistralai/codestral-embed-2505` | nous_portal | $0.0690 | **47.0** | 44.0 | 75 tps | 8k | 🚫 Dominated |
| 197 | **Mistral: Mistral Nemo**<br>`mistralai/mistral-nemo` | nous_portal | $0.0164 | **47.0** | 46.2 | 75 tps | 131k | 🚫 Dominated |
| 198 | **AionLabs: Aion 3.5**<br>`aion-labs/aion-3.5` | nous_portal | $3.2250 | **47.0** | 19.2 | 75 tps | 262k | 🚫 Dominated |
| 199 | **Mistral: Codestral 2508**<br>`mistralai/codestral-2508` | nous_portal_batch | $0.3990 | **47.0** | 35.2 | 150 tps | 256k | 🚫 Dominated |
| 200 | **OpenAI: GPT-3.5 Turbo (older v0613)**<br>`openai/gpt-3.5-turbo-0613` | nous_portal | $1.0600 | **47.0** | 27.3 | 75 tps | 4k | 🚫 Dominated |
| 201 | **VoyageAI by MongoDB: voyage-4**<br>`voyageai/voyage-4` | nous_portal | $0.0276 | **47.0** | 45.7 | 75 tps | 32k | 🚫 Dominated |
| 202 | **LiquidAI: LFM2.5-24B-A2B**<br>`liquid/lfm-2.5-24b-a2b` | nous_portal | $0.2375 | **47.0** | 38.7 | 75 tps | 128k | 🚫 Dominated |
| 203 | **Relace: Relace Search**<br>`relace/relace-search` | nous_portal | $1.3600 | **47.0** | 25.3 | 75 tps | 256k | 🚫 Dominated |
| 204 | **AionLabs: Aion-3.0**<br>`aion-labs/aion-3.0` | nous_portal | $3.2250 | **47.0** | 19.2 | 75 tps | 131k | 🚫 Dominated |
| 205 | **Qwen: Qwen3 Embedding 4B**<br>`qwen/qwen3-embedding-4b` | nous_portal | $0.0092 | **47.0** | 46.5 | 75 tps | 32k | 🚫 Dominated |
| 206 | **Thenlper: GTE-Base**<br>`thenlper/gte-base` | nous_portal | $0.0023 | **47.0** | 46.9 | 75 tps | 0k | 🚫 Dominated |
| 207 | **Mistral: Mistral Embed 2312**<br>`mistralai/mistral-embed-2312` | nous_portal | $0.0460 | **47.0** | 44.9 | 75 tps | 8k | 🚫 Dominated |
| 208 | **Cohere: Command R (08-2024)**<br>`cohere/command-r-08-2024` | nous_portal | $0.2490 | **47.0** | 38.4 | 75 tps | 128k | 🚫 Dominated |
| 209 | **OpenAI: o1**<br>`openai/o1` | nous_portal | $26.2500 | **46.8** | 10.9 | 75 tps | 200k | 🚫 Dominated |
| 210 | **StepFun: Step 5 Preview**<br>`stepfun/step-5-preview` | nous_portal | $1.2250 | **46.7** | 26.0 | 75 tps | 1000k | 🚫 Dominated |
| 211 | **Sakana: Fugu Max**<br>`sakana/fugu-max` | nous_portal | $2.6750 | **46.7** | 20.3 | 75 tps | 1000k | 🚫 Dominated |
| 212 | **OpenAI: GPT Terra Latest**<br>`openai/gpt-terra-latest` | nous_portal | $4.4600 | **46.7** | 17.3 | 75 tps | 1050k | 🚫 Dominated |
| 213 | **Qwen: Qwen3.5 Plus 2026-02-15**<br>`qwen/qwen3.5-plus-02-15` | nous_portal | $0.5876 | **46.7** | 31.9 | 75 tps | 1000k | 🚫 Dominated |
| 214 | **MoonshotAI: Kimi K3**<br>`moonshotai/kimi-k3:US` | nous_portal | $7.0059 | **46.7** | 15.2 | 75 tps | 1048k | 🚫 Dominated |
| 215 | **Meta: Muse Wonder**<br>`meta/muse-wonder` | nous_portal | $7.6600 | **46.7** | 14.8 | 75 tps | 1048k | 🚫 Dominated |
| 216 | **Anthropic: Claude Fable Latest**<br>`anthropic/claude-fable-latest` | nous_portal | $19.0750 | **46.7** | 11.7 | 75 tps | 1000k | 🚫 Dominated |
| 217 | **Fireworks: Ember-1**<br>`fireworks/ember-1` | nous_portal | $5.7900 | **46.7** | 16.0 | 75 tps | 1048k | 🚫 Dominated |
| 218 | **Pareto 26.10 Preview**<br>`unbiased/pareto-26.10-preview` | nous_portal | $1.2890 | **46.7** | 25.6 | 75 tps | 1048k | 🚫 Dominated |
| 219 | **Sakana: Fugu Ultra v2**<br>`sakana/fugu-ultra-v2` | nous_portal | $11.1500 | **46.7** | 13.4 | 75 tps | 1000k | 🚫 Dominated |
| 220 | **OpenAI: GPT-5.4 Pro**<br>`openai/gpt-5.4-pro` | nous_portal_batch | $67.8000 | **46.7** | 8.9 | 150 tps | 1050k | 🚫 Dominated |
| 221 | **Mistral: Mistral Large 4**<br>`mistralai/mistral-large-4-0` | nous_portal | $0.9200 | **46.7** | 28.3 | 75 tps | 1048k | 🚫 Dominated |
| 222 | **Qwen: Qwen3.5 Plus 2026-04-20**<br>`qwen/qwen3.5-plus-20260420` | nous_portal | $0.6780 | **46.7** | 30.8 | 75 tps | 1000k | 🚫 Dominated |
| 223 | **MoonshotAI: Kimi Latest**<br>`moonshotai/kimi-latest` | nous_portal | $4.8760 | **46.7** | 16.9 | 75 tps | 1048k | 🚫 Dominated |
| 224 | **xAI: Grok Latest**<br>`x-ai/grok-latest` | nous_portal | $2.7500 | **46.7** | 20.1 | 75 tps | 500k | 🚫 Dominated |
| 225 | **OpenAI: GPT-5.6 Terra Pro**<br>`openai/gpt-5.6-terra-pro` | nous_portal_batch | $4.4600 | **46.7** | 17.3 | 150 tps | 1050k | 🚫 Dominated |
| 226 | **Sakana: Fugu Ultra**<br>`sakana/fugu-ultra` | nous_portal | $11.1500 | **46.7** | 13.4 | 75 tps | 1000k | 🚫 Dominated |
| 227 | **OpenAI: GPT-5.5 Pro**<br>`openai/gpt-5.5-pro` | nous_portal_batch | $67.8000 | **46.7** | 8.9 | 150 tps | 1050k | 🚫 Dominated |
| 228 | **Xiaomi: MiMo-V2.6-Pro-UltraSpeed**<br>`xiaomi/mimo-v2.6-pro-ultraspeed` | nous_portal | $4.3608 | **46.7** | 17.4 | 75 tps | 1048k | 🚫 Dominated |
| 229 | **Google: Gemma 4 26B A4B **<br>`google/gemma-4-26b-a4b-it` | openrouter_market | $0.0891 | **46.6** | 42.9 | 70 tps | 262k | 🚫 Dominated |
| 230 | **Qwen: Qwen3 Coder Next**<br>`qwen/qwen3-coder-next` | nous_portal | $0.3090 | **46.5** | 36.6 | 90 tps | 262k | 🚫 Dominated |
| 231 | **OpenAI: GPT-5**<br>`openai/gpt-5` | nous_portal_batch | $3.5375 | **45.8** | 18.2 | 150 tps | 400k | 🚫 Dominated |
| 232 | **NVIDIA: Nemotron 3 Super**<br>`nvidia/nemotron-3-super-120b-a12b` | openrouter_market | $0.1718 | **45.7** | 39.4 | 70 tps | 262k | 🚫 Dominated |
| 233 | **Anthropic: Claude Sonnet 4**<br>`anthropic/claude-sonnet-4` | nous_portal | $5.7900 | **45.6** | 15.7 | 75 tps | 200k | 🚫 Dominated |
| 234 | **Upstage: Solar Mini 4**<br>`upstage/solar-mini4` | nous_portal | $0.0815 | **44.1** | 40.9 | 130 tps | 524k | 🚫 Dominated |
| 235 | **MiniMax: MiniMax M2.5**<br>`minimax/minimax-m2.5` | nous_portal | $0.4020 | **43.4** | 32.4 | 130 tps | 204k | 🚫 Dominated |
| 236 | **OpenAI: GPT-5.3-Codex**<br>`openai/gpt-5.3-codex` | nous_portal | $4.9525 | **42.8** | 15.4 | 75 tps | 400k | 🚫 Dominated |
| 237 | **GPT-4o**<br>`gpt-4o` | nous_portal | $0.8300 | **42.5** | 26.5 | 75 tps | 128k | 🚫 Dominated |
| 238 | **GPT-4o**<br>`gpt-4o` | nous_portal | $0.8300 | **42.5** | 26.5 | 75 tps | 128k | 🚫 Dominated |
| 239 | **ByteDance Seed: Seed-2.0-Lite**<br>`bytedance-seed/seed-2.0-lite` | nous_portal | $0.7150 | **42.5** | 27.6 | 75 tps | 262k | 🚫 Dominated |
| 240 | **Pareto Latest**<br>`unbiased/pareto-latest` | nous_portal | $3.3250 | **42.5** | 17.2 | 75 tps | 262k | 🚫 Dominated |
| 241 | **VoyageAI by MongoDB: voyage-multimodal-3.5**<br>`voyageai/voyage-multimodal-3.5` | nous_portal | $0.0552 | **42.5** | 40.3 | 75 tps | 32k | 🚫 Dominated |
| 242 | **Nex AGI: Nex-N2.5-Pro**<br>`nex-agi/nex-n2.5-pro` | nous_portal | $0.1095 | **42.5** | 38.5 | 75 tps | 262k | 🚫 Dominated |
| 243 | **ByteDance Seed: Seed 1.6**<br>`bytedance-seed/seed-1.6` | nous_portal | $0.7150 | **42.5** | 27.6 | 75 tps | 262k | 🚫 Dominated |
| 244 | **Mistral: Mistral Small 3.2 24B**<br>`mistralai/mistral-small-3.2-24b-instruct` | nous_portal | $0.0945 | **42.5** | 38.9 | 75 tps | 256k | 🚫 Dominated |
| 245 | **Cohere: Command A+**<br>`cohere/command-a-plus` | nous_portal | $0.6150 | **42.5** | 28.7 | 75 tps | 192k | 🚫 Dominated |
| 246 | **OpenAI: GPT-5 Pro**<br>`openai/gpt-5-pro` | nous_portal_batch | $42.9000 | **42.5** | 8.9 | 150 tps | 400k | 🚫 Dominated |
| 247 | **Pareto 26.9**<br>`unbiased/pareto-26.9` | nous_portal | $3.3250 | **42.5** | 17.2 | 75 tps | 262k | 🚫 Dominated |
| 248 | **ByteDance Seed: Seed 2.1 Turbo**<br>`bytedance-seed/seed-2-1-turbo` | nous_portal | $0.9800 | **42.5** | 25.2 | 75 tps | 262k | 🚫 Dominated |
| 249 | **OpenAI: GPT-5.2 Chat**<br>`openai/gpt-5.2-chat` | nous_portal | $4.9525 | **42.5** | 15.3 | 75 tps | 128k | 🚫 Dominated |
| 250 | **Mistral: Voxtral Small 24B 2507**<br>`mistralai/voxtral-small-24b-2507` | nous_portal | $0.1330 | **42.5** | 37.8 | 75 tps | 32k | 🚫 Dominated |
| 251 | **OpenAI: GPT-5.2 Pro**<br>`openai/gpt-5.2-pro` | nous_portal_batch | $60.0600 | **42.5** | 8.3 | 150 tps | 400k | 🚫 Dominated |
| 252 | **PrismML: Ternary Bonsai 2 27B**<br>`prism-ml/ternary-bonsai-2-27b` | nous_portal | $0.1912 | **42.5** | 36.1 | 75 tps | 262k | 🚫 Dominated |
| 253 | **ByteDance Seed: Seed-2.0-Code**<br>`bytedance-seed/seed-2.0-code` | nous_portal | $1.1300 | **42.5** | 24.2 | 75 tps | 262k | 🚫 Dominated |
| 254 | **OpenAI: GPT-5.1-Codex-Max**<br>`openai/gpt-5.1-codex-max` | nous_portal | $3.5375 | **42.5** | 16.9 | 75 tps | 400k | 🚫 Dominated |
| 255 | **OpenAI: GPT Chat Latest**<br>`openai/gpt-chat-latest` | nous_portal | $11.1500 | **42.5** | 12.1 | 75 tps | 400k | 🚫 Dominated |
| 256 | **Reka Edge**<br>`rekaai/reka-edge` | nous_portal | $0.0760 | **42.5** | 39.6 | 75 tps | 16k | 🚫 Dominated |
| 257 | **Perceptron: Perceptron Mk1.5**<br>`perceptron/perceptron-mk1.5` | nous_portal | $0.5190 | **42.5** | 29.9 | 75 tps | 36k | 🚫 Dominated |
| 258 | **Mistral: Mistral Small 3.1 24B**<br>`mistralai/mistral-small-3.1-24b-instruct` | nous_portal | $0.3280 | **42.5** | 33.1 | 75 tps | 128k | 🚫 Dominated |
| 259 | **OpenAI: GPT-4o**<br>`openai/gpt-4o` | nous_portal_batch | $4.1500 | **42.5** | 16.1 | 150 tps | 128k | 🚫 Dominated |
| 260 | **OpenAI: GPT Audio**<br>`openai/gpt-audio` | nous_portal | $4.1500 | **42.5** | 16.1 | 75 tps | 128k | 🚫 Dominated |
| 261 | **MiniMax: MiniMax M2.1**<br>`minimax/minimax-m2.1` | nous_portal | $0.4890 | **42.4** | 30.3 | 130 tps | 204k | 🚫 Dominated |
| 262 | **OpenAI: GPT-5 Mini**<br>`openai/gpt-5-mini` | nous_portal_batch | $0.7090 | **42.2** | 27.5 | 150 tps | 400k | 🚫 Dominated |
| 263 | **Mistral: Devstral 2 2512**<br>`mistralai/devstral-2512` | nous_portal | $0.7720 | **42.2** | 26.8 | 75 tps | 262k | 🚫 Dominated |
| 264 | **OpenAI: GPT-5.1-Codex-Mini**<br>`openai/gpt-5.1-codex-mini` | nous_portal | $0.7090 | **42.1** | 27.4 | 130 tps | 400k | 🚫 Dominated |
| 265 | **Inception: Mercury 2**<br>`inception/mercury-2` | nous_portal | $0.3325 | **42.1** | 32.7 | 75 tps | 128k | 🚫 Dominated |
| 266 | **OpenAI: GPT-4.1 Mini**<br>`openai/gpt-4.1-mini` | nous_portal_batch | $0.6700 | **42.0** | 27.7 | 150 tps | 1047k | 🚫 Dominated |
| 267 | **OpenAI: gpt-oss-120b**<br>`openai/gpt-oss-120b` | nous_portal_batch | $0.0660 | **41.7** | 39.2 | 150 tps | 131k | 🚫 Dominated |
| 268 | **OpenAI: GPT-5.2**<br>`openai/gpt-5.2` | nous_portal_batch | $4.9525 | **41.7** | 15.0 | 150 tps | 400k | 🚫 Dominated |
| 269 | **MiniMax: MiniMax M2**<br>`minimax/minimax-m2` | nous_portal | $0.4233 | **41.1** | 30.4 | 130 tps | 204k | 🚫 Dominated |
| 270 | **OpenAI: GPT-5.2-Codex**<br>`openai/gpt-5.2-codex` | nous_portal | $4.9525 | **40.6** | 14.6 | 75 tps | 400k | 🚫 Dominated |
| 271 | **StepFun: Step 3.5 Flash**<br>`stepfun/step-3.5-flash` | nous_portal | $0.1360 | **40.2** | 35.7 | 130 tps | 262k | 🚫 Dominated |
| 272 | **OpenAI: o4 Mini**<br>`openai/o4-mini` | nous_portal_batch | $1.8425 | **40.0** | 19.6 | 150 tps | 200k | 🚫 Dominated |
| 273 | **OpenAI: o3 Mini High**<br>`openai/o3-mini-high` | nous_portal | $1.9250 | **39.8** | 19.2 | 130 tps | 200k | 🚫 Dominated |
| 274 | **NVIDIA: Nemotron 3.5 Lightning**<br>`nvidia/nemotron-3.5-lightning` | openrouter_market | $0.0497 | **39.7** | 37.9 | 70 tps | 262k | 🚫 Dominated |
| 275 | **Z.ai: GLM 5 Turbo**<br>`z-ai/glm-5-turbo` | nous_portal | $1.7520 | **39.6** | 19.7 | 75 tps | 202k | 🚫 Dominated |
| 276 | **Mistral: Mistral Small 4**<br>`mistralai/mistral-small-2603` | nous_portal_batch | $0.2445 | **39.6** | 32.5 | 150 tps | 262k | 🚫 Dominated |
| 277 | **Google: Gemini 2.5 Flash**<br>`google/gemini-2.5-flash` | nous_portal_batch | $0.8790 | **39.4** | 24.1 | 150 tps | 1048k | 🚫 Dominated |
| 278 | **Arcee AI: Trinity Large Thinking**<br>`arcee-ai/trinity-large-thinking` | nous_portal | $0.3580 | **39.2** | 30.0 | 75 tps | 262k | 🚫 Dominated |
| 279 | **SpaceXAI: Grok 4.20**<br>`x-ai/grok-4.20` | nous_portal | $1.3100 | **39.1** | 21.3 | 75 tps | 2000k | 🚫 Dominated |
| 280 | **Z.ai: GLM 4.7 Flash**<br>`z-ai/glm-4.7-flash` | nous_portal | $0.1470 | **39.1** | 34.3 | 130 tps | 200k | 🚫 Dominated |
| 281 | **Mistral: Ministral 3 14B 2512**<br>`mistralai/ministral-14b-2512` | nous_portal | $0.1460 | **38.8** | 34.1 | 130 tps | 262k | 🚫 Dominated |
| 282 | **Claude Opus 4.6**<br>`anthropic/claude-opus-4.6` | nous_portal_batch | $9.6500 | **38.8** | 11.5 | 150 tps | 1000k | 🚫 Dominated |
| 283 | **DeepSeek R1**<br>`deepseek-r1` | nous_portal | $0.8300 | **38.5** | 24.0 | 75 tps | 64k | 🚫 Dominated |
| 284 | **DeepSeek R1**<br>`deepseek-r1` | nous_portal | $0.8300 | **38.5** | 24.0 | 75 tps | 64k | 🚫 Dominated |
| 285 | **DeepSeek: R1**<br>`deepseek/deepseek-r1` | nous_portal | $1.0720 | **38.5** | 22.3 | 75 tps | 64k | 🚫 Dominated |
| 286 | **OpenAI: GPT-4o (2024-05-13)**<br>`openai/gpt-4o-2024-05-13` | nous_portal | $6.8000 | **38.3** | 12.5 | 75 tps | 128k | 🚫 Dominated |
| 287 | **OpenAI: GPT-5.1-Codex**<br>`openai/gpt-5.1-codex` | nous_portal | $3.5390 | **38.0** | 15.1 | 75 tps | 400k | 🚫 Dominated |
| 288 | **Qwen: Qwen3.5-9B**<br>`qwen/qwen3.5-9b` | nous_portal | $0.0830 | **37.9** | 35.1 | 75 tps | 262k | 🚫 Dominated |
| 289 | **OpenAI o3-mini**<br>`o3-mini` | nous_portal | $0.8300 | **37.7** | 23.5 | 130 tps | 200k | 🚫 Dominated |
| 290 | **OpenAI o3-mini**<br>`o3-mini` | nous_portal | $0.8300 | **37.7** | 23.5 | 130 tps | 200k | 🚫 Dominated |
| 291 | **OpenAI: o3 Mini**<br>`openai/o3-mini` | nous_portal_batch | $1.9250 | **37.7** | 18.2 | 150 tps | 200k | 🚫 Dominated |
| 292 | **Amazon: Nova 2 Lite**<br>`amazon/nova-2-lite-v1` | nous_portal | $0.8880 | **37.6** | 23.0 | 75 tps | 1000k | 🚫 Dominated |
| 293 | **Qwen: Qwen3.5-27B**<br>`qwen/qwen3.5-27b` | nous_portal | $0.5577 | **37.6** | 26.0 | 75 tps | 262k | 🚫 Dominated |
| 294 | **MiniMax: MiniMax M1**<br>`minimax/minimax-m1` | nous_portal | $0.8440 | **37.3** | 23.1 | 130 tps | 1000k | 🚫 Dominated |
| 295 | **IBM: Granite 4.2 8B**<br>`ibm-granite/granite-4.2-8b` | nous_portal | $0.1035 | **37.3** | 33.9 | 75 tps | 131k | 🚫 Dominated |
| 296 | **Anthropic: Claude Opus 4.5**<br>`anthropic/claude-opus-4.5` | nous_portal_batch | $9.6500 | **37.2** | 11.1 | 150 tps | 200k | 🚫 Dominated |
| 297 | **GPT-4o Mini**<br>`gpt-4o-mini` | openrouter | $0.8300 | **37.1** | 23.1 | 120 tps | 128k | 🚫 Dominated |
| 298 | **GPT-4o Mini**<br>`gpt-4o-mini` | openrouter | $0.8300 | **37.1** | 23.1 | 120 tps | 128k | 🚫 Dominated |
| 299 | **OpenAI: GPT-4o-mini**<br>`openai/gpt-4o-mini` | nous_portal_batch | $0.2625 | **37.1** | 30.1 | 150 tps | 128k | 🚫 Dominated |
| 300 | **Qwen: Qwen3 235B A22B Thinking 2507**<br>`qwen/qwen3-235b-a22b-thinking-2507` | nous_portal | $1.2570 | **37.1** | 20.5 | 75 tps | 128k | 🚫 Dominated |
| 301 | **MoonshotAI: Kimi K2 Thinking**<br>`moonshotai/kimi-k2-thinking` | nous_portal | $1.0260 | **37.1** | 21.7 | 75 tps | 262k | 🚫 Dominated |
| 302 | **OpenAI: o3 Pro**<br>`openai/o3-pro` | nous_portal | $33.2000 | **37.0** | 8.2 | 75 tps | 200k | 🚫 Dominated |
| 303 | **OpenAI: GPT-4 Turbo**<br>`openai/gpt-4-turbo` | nous_portal_batch | $13.6000 | **36.8** | 10.0 | 150 tps | 128k | 🚫 Dominated |
| 304 | **DeepSeek: DeepSeek V3 0324**<br>`deepseek/deepseek-chat-v3-0324` | nous_portal | $0.4150 | **36.6** | 27.2 | 75 tps | 163k | 🚫 Dominated |
| 305 | **Google: Gemini 2.5 Flash Lite**<br>`google/gemini-2.5-flash-lite` | nous_portal_batch | $0.1630 | **36.6** | 31.8 | 150 tps | 1048k | 🚫 Dominated |
| 306 | **Mistral: Mistral Medium 3.1**<br>`mistralai/mistral-medium-3.1` | nous_portal_batch | $0.7720 | **36.2** | 23.1 | 150 tps | 131k | 🚫 Dominated |
| 307 | **Mistral: Ministral 3 8B 2512**<br>`mistralai/ministral-8b-2512` | nous_portal_batch | $0.1095 | **36.2** | 32.8 | 150 tps | 262k | 🚫 Dominated |
| 308 | **OpenAI: o3**<br>`openai/o3` | nous_portal_batch | $3.3500 | **36.1** | 14.6 | 150 tps | 200k | 🚫 Dominated |
| 309 | **Mistral: Mistral Large 3 2512**<br>`mistralai/mistral-large-2512` | nous_portal_batch | $0.6650 | **36.0** | 23.9 | 150 tps | 262k | 🚫 Dominated |
| 310 | **Qwen: Qwen3.5-35B-A3B**<br>`qwen/qwen3.5-35b-a3b` | nous_portal | $0.2690 | **35.6** | 28.7 | 75 tps | 262k | 🚫 Dominated |
| 311 | **Upstage: Solar Pro 3**<br>`upstage/solar-pro-3` | nous_portal | $0.2445 | **35.5** | 29.1 | 90 tps | 131k | 🚫 Dominated |
| 312 | **Qwen: Qwen3 Next 80B A3B Thinking**<br>`qwen/qwen3-next-80b-a3b-thinking` | nous_portal | $0.4290 | **34.5** | 25.4 | 75 tps | 262k | 🚫 Dominated |
| 313 | **DeepSeek: DeepSeek V3.2 Exp**<br>`deepseek/deepseek-v3.2-exp` | nous_portal | $0.2472 | **34.1** | 27.9 | 75 tps | 163k | 🚫 Dominated |
| 314 | **Meta: Llama 4 Maverick**<br>`meta-llama/llama-4-maverick` | nous_portal | $0.2858 | **33.9** | 27.1 | 75 tps | 1048k | 🚫 Dominated |
| 315 | **Anthropic: Claude Opus 4.1**<br>`anthropic/claude-opus-4.1` | nous_portal_batch | $28.9500 | **33.8** | 7.7 | 150 tps | 200k | 🚫 Dominated |
| 316 | **Mistral: Ministral 3 3B 2512**<br>`mistralai/ministral-3b-2512` | nous_portal | $0.0730 | **33.5** | 31.3 | 130 tps | 131k | 🚫 Dominated |
| 317 | **Qwen: Qwen3 32B**<br>`qwen/qwen3-32b` | nous_portal | $0.1208 | **33.4** | 30.0 | 75 tps | 131k | 🚫 Dominated |
| 318 | **MoonshotAI: Kimi K2 0905**<br>`moonshotai/kimi-k2-0905` | nous_portal | $1.0260 | **33.4** | 19.6 | 75 tps | 262k | 🚫 Dominated |
| 319 | **Qwen: Qwen3 Coder 480B A35B**<br>`qwen/qwen3-coder` | nous_portal | $0.6412 | **33.1** | 22.1 | 90 tps | 262k | 🚫 Dominated |
| 320 | **NVIDIA: Nemotron 3 Nano 30B A3B**<br>`nvidia/nemotron-3-nano-30b-a3b` | nous_portal | $0.0830 | **32.9** | 30.5 | 75 tps | 262k | 🚫 Dominated |
| 321 | **Qwen: Qwen3 14B**<br>`qwen/qwen3-14b` | nous_portal | $0.1120 | **32.6** | 29.4 | 75 tps | 40k | 🚫 Dominated |
| 322 | **DeepSeek: DeepSeek V3.1**<br>`deepseek/deepseek-chat-v3.1` | nous_portal | $0.4240 | **32.5** | 24.0 | 75 tps | 163k | 🚫 Dominated |
| 323 | **OpenAI: GPT-4**<br>`openai/gpt-4` | nous_portal | $31.8000 | **32.2** | 7.2 | 75 tps | 8k | 🚫 Dominated |
| 324 | **DeepSeek: R1 0528**<br>`deepseek/deepseek-r1-0528` | nous_portal | $0.9500 | **32.2** | 19.3 | 75 tps | 163k | 🚫 Dominated |
| 325 | **OpenAI: GPT-5 Nano**<br>`openai/gpt-5-nano` | nous_portal_batch | $0.1430 | **32.1** | 28.3 | 150 tps | 400k | 🚫 Dominated |
| 326 | **MoonshotAI: Kimi K2 0711**<br>`moonshotai/kimi-k2` | nous_portal | $0.9522 | **31.9** | 19.1 | 75 tps | 131k | 🚫 Dominated |
| 327 | **OpenAI: GPT-4.1**<br>`openai/gpt-4.1` | nous_portal_batch | $3.3500 | **31.9** | 12.9 | 150 tps | 1047k | 🚫 Dominated |
| 328 | **Qwen: Qwen3 Coder 30B A3B Instruct**<br>`qwen/qwen3-coder-30b-a3b-instruct` | nous_portal | $0.1162 | **31.9** | 28.7 | 90 tps | 262k | 🚫 Dominated |
| 329 | **Inception: Mercury 2.5**<br>`inception/mercury-2.5` | nous_portal | $0.0622 | **31.7** | 29.9 | 75 tps | 260k | 🚫 Dominated |
| 330 | **Qwen: Qwen3 235B A22B Instruct 2507**<br>`qwen/qwen3-235b-a22b-2507` | nous_portal | $0.1452 | **31.6** | 27.8 | 75 tps | 262k | 🚫 Dominated |
| 331 | **Llama 3.3 70B Instruct**<br>`llama-3.3-70b` | nous_portal | $0.8300 | **31.5** | 19.6 | 75 tps | 128k | 🚫 Dominated |
| 332 | **Llama 3.3 70B Instruct**<br>`llama-3.3-70b` | nous_portal | $0.8300 | **31.5** | 19.6 | 75 tps | 128k | 🚫 Dominated |
| 333 | **Meta: Llama 3.3 70B Instruct**<br>`meta-llama/llama-3.3-70b-instruct` | nous_portal | $0.1420 | **31.5** | 27.8 | 75 tps | 131k | 🚫 Dominated |
| 334 | **Z.ai: GLM 4.6V**<br>`z-ai/glm-4.6v` | nous_portal | $0.4065 | **31.1** | 23.2 | 75 tps | 131k | 🚫 Dominated |
| 335 | **OpenAI: GPT-4.1 Nano**<br>`openai/gpt-4.1-nano` | nous_portal_batch | $0.1690 | **31.1** | 26.9 | 150 tps | 1047k | 🚫 Dominated |
| 336 | **Z.ai: GLM 4.5 Air**<br>`z-ai/glm-4.5-air` | nous_portal | $0.3145 | **31.1** | 24.4 | 75 tps | 131k | 🚫 Dominated |
| 337 | **OpenAI: GPT-3.5 Turbo**<br>`openai/gpt-3.5-turbo` | nous_portal_batch | $0.6800 | **30.8** | 20.3 | 150 tps | 16k | 🚫 Dominated |
| 338 | **Google: Gemma 3 27B**<br>`google/gemma-3-27b-it` | nous_portal | $0.1225 | **30.5** | 27.4 | 75 tps | 131k | 🚫 Dominated |
| 339 | **OpenAI: gpt-oss-20b**<br>`openai/gpt-oss-20b` | nous_portal | $0.0369 | **30.5** | 29.4 | 75 tps | 131k | 🚫 Dominated |
| 340 | **Qwen: Qwen3 VL 235B A22B Instruct**<br>`qwen/qwen3-vl-235b-a22b-instruct` | nous_portal | $0.3770 | **30.4** | 23.0 | 75 tps | 262k | 🚫 Dominated |
| 341 | **Qwen 2.5 Coder 32B**<br>`qwen-2.5-coder-32b` | nous_portal | $0.8300 | **30.3** | 18.9 | 105 tps | 131k | 🚫 Dominated |
| 342 | **Qwen 2.5 Coder 32B**<br>`qwen-2.5-coder-32b` | nous_portal | $0.8300 | **30.3** | 18.9 | 105 tps | 131k | 🚫 Dominated |
| 343 | **Qwen: Qwen3 Next 80B A3B Instruct**<br>`qwen/qwen3-next-80b-a3b-instruct` | nous_portal | $0.3714 | **30.2** | 23.0 | 75 tps | 262k | 🚫 Dominated |
| 344 | **Qwen: Qwen3 VL 30B A3B Thinking**<br>`qwen/qwen3-vl-30b-a3b-thinking` | nous_portal | $0.4334 | **30.2** | 22.2 | 75 tps | 262k | 🚫 Dominated |
| 345 | **Amazon: Nova Premier 1.0**<br>`amazon/nova-premier-v1` | nous_portal | $4.9375 | **30.0** | 10.8 | 75 tps | 1000k | 🚫 Dominated |
| 346 | **Mistral: Mistral Medium 3**<br>`mistralai/mistral-medium-3` | nous_portal | $0.7720 | **29.9** | 19.0 | 75 tps | 131k | 🚫 Dominated |
| 347 | **OpenAI: GPT-4o (2024-11-20)**<br>`openai/gpt-4o-2024-11-20` | nous_portal | $4.3750 | **29.6** | 11.0 | 75 tps | 128k | 🚫 Dominated |
| 348 | **Meta: Llama 4 Scout**<br>`meta-llama/llama-4-scout` | nous_portal | $0.1360 | **29.5** | 26.1 | 75 tps | 1310k | 🚫 Dominated |
| 349 | **Qwen: Qwen3 VL 30B A3B Instruct**<br>`qwen/qwen3-vl-30b-a3b-instruct` | nous_portal | $0.2490 | **29.3** | 24.0 | 75 tps | 262k | 🚫 Dominated |
| 350 | **OpenAI: GPT-4o (2024-08-06)**<br>`openai/gpt-4o-2024-08-06` | nous_portal | $4.3750 | **29.2** | 10.9 | 75 tps | 128k | 🚫 Dominated |
| 351 | **Qwen2.5 72B Instruct**<br>`qwen/qwen-2.5-72b-instruct` | nous_portal | $0.2856 | **29.2** | 23.3 | 75 tps | 32k | 🚫 Dominated |
| 352 | **Qwen: Qwen3 30B A3B**<br>`qwen/qwen3-30b-a3b` | nous_portal | $0.2052 | **29.1** | 24.6 | 75 tps | 40k | 🚫 Dominated |
| 353 | **Z.ai: GLM 4.5V**<br>`z-ai/glm-4.5v` | nous_portal | $0.8130 | **29.1** | 18.3 | 75 tps | 65k | 🚫 Dominated |
| 354 | **Mistral Large 2407**<br>`mistralai/mistral-large-2407` | nous_portal | $2.6600 | **29.1** | 12.7 | 75 tps | 131k | 🚫 Dominated |
| 355 | **Qwen: Qwen3 30B A3B Instruct 2507**<br>`qwen/qwen3-30b-a3b-instruct-2507` | nous_portal | $0.1314 | **29.1** | 25.9 | 75 tps | 262k | 🚫 Dominated |
| 356 | **Qwen: Qwen3 VL 8B Instruct**<br>`qwen/qwen3-vl-8b-instruct` | nous_portal | $0.3610 | **29.0** | 22.2 | 75 tps | 262k | 🚫 Dominated |
| 357 | **Amazon: Nova Pro 1.0**<br>`amazon/nova-pro-v1` | nous_portal | $1.3280 | **28.8** | 15.6 | 75 tps | 300k | 🚫 Dominated |
| 358 | **Hermes 3 (Llama 3.1 70B)**<br>`hermes-3-llama-3.1-70b` | nous_portal | $0.8300 | **28.6** | 17.9 | 78 tps | 128k | 🚫 Dominated |
| 359 | **Hermes 3 (Llama 3.1 70B)**<br>`hermes-3-llama-3.1-70b` | nous_portal | $0.8300 | **28.6** | 17.9 | 78 tps | 128k | 🚫 Dominated |
| 360 | **Amazon: Nova Lite 1.0**<br>`amazon/nova-lite-v1` | nous_portal | $0.0996 | **28.6** | 26.2 | 75 tps | 300k | 🚫 Dominated |
| 361 | **Meta: Llama 3.1 70B Instruct**<br>`meta-llama/llama-3.1-70b-instruct` | nous_portal | $0.3040 | **28.6** | 22.6 | 75 tps | 131k | 🚫 Dominated |
| 362 | **Mistral: Saba**<br>`mistralai/mistral-saba` | nous_portal | $0.2660 | **28.5** | 23.1 | 75 tps | 32k | 🚫 Dominated |
| 363 | **Amazon: Nova Micro 1.0**<br>`amazon/nova-micro-v1` | nous_portal | $0.0581 | **28.2** | 26.7 | 75 tps | 128k | 🚫 Dominated |
| 364 | **Mistral Large**<br>`mistralai/mistral-large` | nous_portal | $2.6600 | **28.2** | 12.3 | 75 tps | 128k | 🚫 Dominated |
| 365 | **Google: Gemma 3 12B**<br>`google/gemma-3-12b-it` | nous_portal | $0.0680 | **28.2** | 26.4 | 75 tps | 131k | 🚫 Dominated |
| 366 | **Mistral: Mixtral 8x22B Instruct**<br>`mistralai/mixtral-8x22b-instruct` | nous_portal | $2.6600 | **28.1** | 12.2 | 75 tps | 65k | 🚫 Dominated |
| 367 | **Meta: Llama 3.1 8B Instruct**<br>`meta-llama/llama-3.1-8b-instruct` | nous_portal | $0.0212 | **27.9** | 27.4 | 75 tps | 131k | 🚫 Dominated |
| 368 | **Hermes 3 (Llama 3.1 405B)**<br>`hermes-3-llama-3.1-405b` | nous_portal | $0.8300 | **25.3** | 15.8 | 40 tps | 128k | 🚫 Dominated |
| 369 | **Hermes 3 (Llama 3.1 405B)**<br>`hermes-3-llama-3.1-405b` | nous_portal | $0.8300 | **25.3** | 15.8 | 40 tps | 128k | 🚫 Dominated |

#### 🔗 Cadeia de Roteamento Preditivo para Subagentes (`design`)
| Nível de Escalação | Modelo Designado | Provedor | Custo Blended | Condição de Acionamento |
| :--- | :--- | :--- | :---: | :--- |
| **1. Primário (Custo-Benefício)** | `GPT-4o` | nous_portal | $2.0938 | Modelo padrão para subtarefas com SLA normal e orçamento otimizado. |
| **2. Escalação Máxima (Líder)** | `Claude 3.7 Sonnet` | nous_portal | $5.9650 | Acionado se complexidade/entropia for crítica ou falha de teste anterior. |
| **3. Fallback de Baixo Custo** | `OpenAI o3-mini` | nous_portal | $0.9213 | Standby caso o provedor principal sofra throttling ou indisponibilidade. |

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
| `aion-labs/aion-3.5` | **AionLabs: Aion 3.5** | aion-labs | 262,144 tokens | ✅ Sim | ✅ Sim | ❌ Não | ✅ Sim |
| `aion-labs/aion-3.5-mini` | **AionLabs: Aion 3.5 Mini** | aion-labs | 262,144 tokens | ✅ Sim | ✅ Sim | ❌ Não | ✅ Sim |
| `aion-labs/aion-2.0` | **AionLabs: Aion-2.0** | aion-labs | 131,072 tokens | ✅ Sim | ✅ Sim | ❌ Não | ✅ Sim |
| `aion-labs/aion-3.0` | **AionLabs: Aion-3.0** | aion-labs | 131,072 tokens | ✅ Sim | ✅ Sim | ❌ Não | ✅ Sim |
| `aion-labs/aion-3.0-mini` | **AionLabs: Aion-3.0-Mini** | aion-labs | 131,072 tokens | ✅ Sim | ✅ Sim | ❌ Não | ✅ Sim |
| `amazon/nova-2-lite-v1` | **Amazon: Nova 2 Lite** | amazon | 1,000,000 tokens | ✅ Sim | ✅ Sim | 👁️ Sim | ❌ Não |
| `amazon/nova-lite-v1` | **Amazon: Nova Lite 1.0** | amazon | 300,000 tokens | ✅ Sim | ✅ Sim | 👁️ Sim | ❌ Não |
| `amazon/nova-micro-v1` | **Amazon: Nova Micro 1.0** | amazon | 128,000 tokens | ✅ Sim | ✅ Sim | ❌ Não | ❌ Não |
| `amazon/nova-premier-v1` | **Amazon: Nova Premier 1.0** | amazon | 1,000,000 tokens | ✅ Sim | ✅ Sim | 👁️ Sim | ❌ Não |
| `amazon/nova-pro-v1` | **Amazon: Nova Pro 1.0** | amazon | 300,000 tokens | ✅ Sim | ✅ Sim | 👁️ Sim | ❌ Não |
| `anthropic/claude-fable-5.1` | **Anthropic: Claude Fable 5.1** | anthropic | 1,000,000 tokens | ✅ Sim | ✅ Sim | 👁️ Sim | ✅ Sim |
| `anthropic/claude-fable-latest` | **Anthropic: Claude Fable Latest** | anthropic | 1,000,000 tokens | ✅ Sim | ✅ Sim | 👁️ Sim | ✅ Sim |
| `anthropic/claude-haiku-5.5` | **Anthropic: Claude Haiku 5.5** | anthropic | 1,000,000 tokens | ✅ Sim | ✅ Sim | 👁️ Sim | ✅ Sim |
| `anthropic/claude-haiku-latest` | **Anthropic: Claude Haiku Latest** | anthropic | 1,000,000 tokens | ✅ Sim | ✅ Sim | 👁️ Sim | ✅ Sim |
| `anthropic/claude-opus-4.1` | **Anthropic: Claude Opus 4.1** | anthropic | 200,000 tokens | ✅ Sim | ✅ Sim | 👁️ Sim | ❌ Não |
| `anthropic/claude-opus-4.5` | **Anthropic: Claude Opus 4.5** | anthropic | 200,000 tokens | ✅ Sim | ✅ Sim | 👁️ Sim | ✅ Sim |
| `anthropic/claude-opus-5` | **Anthropic: Claude Opus 5** | anthropic | 1,000,000 tokens | ✅ Sim | ✅ Sim | 👁️ Sim | ✅ Sim |
| `anthropic/claude-opus-5.5` | **Anthropic: Claude Opus 5.5** | anthropic | 1,000,000 tokens | ✅ Sim | ✅ Sim | 👁️ Sim | ✅ Sim |
| `anthropic/claude-opus-latest` | **Anthropic: Claude Opus Latest** | anthropic | 1,000,000 tokens | ✅ Sim | ✅ Sim | 👁️ Sim | ✅ Sim |
| `anthropic/claude-sonnet-4` | **Anthropic: Claude Sonnet 4** | anthropic | 200,000 tokens | ✅ Sim | ✅ Sim | 👁️ Sim | ❌ Não |
| `anthropic/claude-sonnet-4.5` | **Anthropic: Claude Sonnet 4.5** | anthropic | 1,000,000 tokens | ✅ Sim | ✅ Sim | 👁️ Sim | ✅ Sim |
| `anthropic/claude-sonnet-5.5` | **Anthropic: Claude Sonnet 5.5** | anthropic | 1,000,000 tokens | ✅ Sim | ✅ Sim | 👁️ Sim | ✅ Sim |
| `anthropic/claude-sonnet-latest` | **Anthropic: Claude Sonnet Latest** | anthropic | 1,000,000 tokens | ✅ Sim | ✅ Sim | 👁️ Sim | ✅ Sim |
| `anthropic/claude-fable-5` | **Claude Fable 5** | anthropic | 1,048,576 tokens | ✅ Sim | ✅ Sim | ❌ Não | ❌ Não |
| `anthropic/claude-haiku-4.5` | **Claude Haiku 4.5** | anthropic | 409,600 tokens | ✅ Sim | ✅ Sim | ❌ Não | ❌ Não |
| `anthropic/claude-opus-4.6` | **Claude Opus 4.6** | anthropic | 1,000,000 tokens | ✅ Sim | ✅ Sim | ❌ Não | ❌ Não |
| `anthropic/claude-opus-4.7` | **Claude Opus 4.7** | anthropic | 1,000,000 tokens | ✅ Sim | ✅ Sim | ❌ Não | ❌ Não |
| `anthropic/claude-opus-4.8` | **Claude Opus 4.8** | anthropic | 1,000,000 tokens | ✅ Sim | ✅ Sim | ❌ Não | ❌ Não |
| `anthropic/claude-sonnet-4.6` | **Claude Sonnet 4.6** | anthropic | 1,000,000 tokens | ✅ Sim | ✅ Sim | ❌ Não | ❌ Não |
| `anthropic/claude-sonnet-5` | **Claude Sonnet 5** | anthropic | 1,000,000 tokens | ✅ Sim | ✅ Sim | ❌ Não | ❌ Não |
| `arcee-ai/trinity-large-thinking` | **Arcee AI: Trinity Large Thinking** | arcee-ai | 262,144 tokens | ✅ Sim | ✅ Sim | ❌ Não | ❌ Não |
| `baai/bge-base-en-v1.5` | **BAAI: bge-base-en-v1.5** | baai | 512 tokens | ❌ Não | ✅ Sim | ❌ Não | ❌ Não |
| `baai/bge-large-en-v1.5` | **BAAI: bge-large-en-v1.5** | baai | 512 tokens | ❌ Não | ✅ Sim | ❌ Não | ❌ Não |
| `baai/bge-m3` | **BAAI: bge-m3** | baai | 8,194 tokens | ❌ Não | ✅ Sim | ❌ Não | ❌ Não |
| `bytedance-seed/seed-1.6` | **ByteDance Seed: Seed 1.6** | bytedance-seed | 262,144 tokens | ✅ Sim | ✅ Sim | 👁️ Sim | ✅ Sim |
| `bytedance-seed/seed-1.6-flash` | **ByteDance Seed: Seed 1.6 Flash** | bytedance-seed | 262,144 tokens | ✅ Sim | ✅ Sim | 👁️ Sim | ✅ Sim |
| `bytedance-seed/seed-2-1-turbo` | **ByteDance Seed: Seed 2.1 Turbo** | bytedance-seed | 262,144 tokens | ✅ Sim | ✅ Sim | 👁️ Sim | ✅ Sim |
| `bytedance-seed/seed-2.0-code` | **ByteDance Seed: Seed-2.0-Code** | bytedance-seed | 262,144 tokens | ✅ Sim | ✅ Sim | 👁️ Sim | ✅ Sim |
| `bytedance-seed/seed-2.0-lite` | **ByteDance Seed: Seed-2.0-Lite** | bytedance-seed | 262,144 tokens | ✅ Sim | ✅ Sim | 👁️ Sim | ✅ Sim |
| `bytedance-seed/seed-2.0-mini` | **ByteDance Seed: Seed-2.0-Mini** | bytedance-seed | 262,144 tokens | ✅ Sim | ✅ Sim | 👁️ Sim | ✅ Sim |
| `cohere/command-a-plus` | **Cohere: Command A+** | cohere | 192,000 tokens | ✅ Sim | ✅ Sim | 👁️ Sim | ✅ Sim |
| `cohere/command-r-08-2024` | **Cohere: Command R (08-2024)** | cohere | 128,000 tokens | ✅ Sim | ✅ Sim | ❌ Não | ✅ Sim |
| `cohere/command-r-plus-08-2024` | **Cohere: Command R+ (08-2024)** | cohere | 128,000 tokens | ✅ Sim | ✅ Sim | ❌ Não | ✅ Sim |
| `deepseek/deepseek-flash-latest` | **DeepSeek: DeepSeek Flash Latest** | deepseek | 1,048,576 tokens | ✅ Sim | ✅ Sim | 👁️ Sim | ✅ Sim |
| `deepseek/deepseek-pro-latest` | **DeepSeek: DeepSeek Pro Latest** | deepseek | 1,048,576 tokens | ✅ Sim | ✅ Sim | ❌ Não | ✅ Sim |
| `deepseek/deepseek-chat` | **DeepSeek: DeepSeek V3** | deepseek | 163,840 tokens | ✅ Sim | ✅ Sim | ❌ Não | ✅ Sim |
| `deepseek/deepseek-chat-v3-0324` | **DeepSeek: DeepSeek V3 0324** | deepseek | 163,840 tokens | ✅ Sim | ✅ Sim | ❌ Não | ✅ Sim |
| `deepseek/deepseek-chat-v3.1` | **DeepSeek: DeepSeek V3.1** | deepseek | 163,840 tokens | ✅ Sim | ✅ Sim | ❌ Não | ✅ Sim |
| `deepseek/deepseek-v3.1-terminus` | **DeepSeek: DeepSeek V3.1 Terminus** | deepseek | 163,840 tokens | ✅ Sim | ✅ Sim | ❌ Não | ✅ Sim |
| `deepseek/deepseek-v3.2` | **DeepSeek: DeepSeek V3.2** | deepseek | 163,840 tokens | ✅ Sim | ✅ Sim | ❌ Não | ✅ Sim |
| `deepseek/deepseek-v3.2-exp` | **DeepSeek: DeepSeek V3.2 Exp** | deepseek | 163,840 tokens | ✅ Sim | ✅ Sim | ❌ Não | ✅ Sim |
| `deepseek/deepseek-v4-flash-0731:US` | **DeepSeek: DeepSeek V4 Flash** | deepseek | 1,048,576 tokens | ✅ Sim | ✅ Sim | ❌ Não | ✅ Sim |
| `deepseek/deepseek-v4-flash-0731` | **DeepSeek: DeepSeek V4 Flash** | deepseek | 1,048,576 tokens | ✅ Sim | ✅ Sim | ❌ Não | ✅ Sim |
| `deepseek/deepseek-v4-flash` | **DeepSeek: DeepSeek V4 Flash 0423** | deepseek | 1,048,576 tokens | ✅ Sim | ✅ Sim | ❌ Não | ✅ Sim |
| `deepseek/deepseek-v4-flash-latest` | **DeepSeek: DeepSeek V4 Flash Latest** | deepseek | 1,048,576 tokens | ✅ Sim | ✅ Sim | ❌ Não | ✅ Sim |
| `deepseek/deepseek-v4-flash-vision-exp` | **DeepSeek: DeepSeek V4 Flash Vision Exp** | deepseek | 1,048,576 tokens | ✅ Sim | ✅ Sim | 👁️ Sim | ✅ Sim |
| `deepseek/deepseek-v4-pro` | **DeepSeek: DeepSeek V4 Pro 0423** | deepseek | 1,048,576 tokens | ✅ Sim | ✅ Sim | ❌ Não | ✅ Sim |
| `deepseek/deepseek-v4-pro-0813` | **DeepSeek: DeepSeek V4 Pro 0813** | deepseek | 1,048,576 tokens | ✅ Sim | ✅ Sim | ❌ Não | ✅ Sim |
| `deepseek/deepseek-v4.1-flash:US` | **DeepSeek: DeepSeek V4.1 Flash** | deepseek | 1,048,576 tokens | ✅ Sim | ✅ Sim | 👁️ Sim | ✅ Sim |
| `deepseek/deepseek-v4.1-flash` | **DeepSeek: DeepSeek V4.1 Flash** | deepseek | 1,048,576 tokens | ✅ Sim | ✅ Sim | 👁️ Sim | ✅ Sim |
| `deepseek/deepseek-r1` | **DeepSeek: R1** | deepseek | 64,000 tokens | ✅ Sim | ✅ Sim | ❌ Não | ✅ Sim |
| `deepseek/deepseek-r1-0528` | **DeepSeek: R1 0528** | deepseek | 163,840 tokens | ✅ Sim | ✅ Sim | ❌ Não | ✅ Sim |
| `fireworks/ember-1` | **Fireworks: Ember-1** | fireworks | 1,048,576 tokens | ✅ Sim | ✅ Sim | 👁️ Sim | ✅ Sim |
| `google/gemini-2.5-flash` | **Google: Gemini 2.5 Flash** | google | 1,048,576 tokens | ✅ Sim | ✅ Sim | 👁️ Sim | ✅ Sim |
| `google/gemini-2.5-flash-lite` | **Google: Gemini 2.5 Flash Lite** | google | 1,048,576 tokens | ✅ Sim | ✅ Sim | 👁️ Sim | ✅ Sim |
| `google/gemini-2.5-pro` | **Google: Gemini 2.5 Pro** | google | 1,048,576 tokens | ✅ Sim | ✅ Sim | 👁️ Sim | ✅ Sim |
| `google/gemini-2.5-pro-preview` | **Google: Gemini 2.5 Pro Preview 06-05** | google | 1,048,576 tokens | ✅ Sim | ✅ Sim | 👁️ Sim | ✅ Sim |
| `google/gemini-3-flash-preview` | **Google: Gemini 3 Flash Preview** | google | 1,048,576 tokens | ✅ Sim | ✅ Sim | 👁️ Sim | ✅ Sim |
| `google/gemini-3.1-flash-lite` | **Google: Gemini 3.1 Flash Lite** | google | 1,048,576 tokens | ✅ Sim | ✅ Sim | 👁️ Sim | ✅ Sim |
| `google/gemini-3.1-flash-lite-preview` | **Google: Gemini 3.1 Flash Lite Preview** | google | 1,048,576 tokens | ✅ Sim | ✅ Sim | 👁️ Sim | ✅ Sim |
| `google/gemini-3.1-pro-preview` | **Google: Gemini 3.1 Pro Preview** | google | 1,048,576 tokens | ✅ Sim | ✅ Sim | 👁️ Sim | ✅ Sim |
| `google/gemini-3.1-pro-preview-customtools` | **Google: Gemini 3.1 Pro Preview Custom Tools** | google | 1,048,576 tokens | ✅ Sim | ✅ Sim | 👁️ Sim | ✅ Sim |
| `google/gemini-3.5-flash` | **Google: Gemini 3.5 Flash** | google | 1,048,576 tokens | ✅ Sim | ✅ Sim | 👁️ Sim | ✅ Sim |
| `google/gemini-3.5-flash-lite` | **Google: Gemini 3.5 Flash Lite** | google | 1,048,576 tokens | ✅ Sim | ✅ Sim | 👁️ Sim | ✅ Sim |
| `google/gemini-3.6-flash` | **Google: Gemini 3.6 Flash** | google | 1,048,576 tokens | ✅ Sim | ✅ Sim | 👁️ Sim | ✅ Sim |
| `google/gemini-3.7-flash` | **Google: Gemini 3.7 Flash** | google | 1,048,576 tokens | ✅ Sim | ✅ Sim | 👁️ Sim | ✅ Sim |
| `google/gemini-3.8-flash` | **Google: Gemini 3.8 Flash** | google | 1,048,576 tokens | ✅ Sim | ✅ Sim | 👁️ Sim | ✅ Sim |
| `google/gemini-4-argon` | **Google: Gemini 4 Argon** | google | 1,048,576 tokens | ✅ Sim | ✅ Sim | 👁️ Sim | ✅ Sim |
| `google/gemini-embedding-001` | **Google: Gemini Embedding 001** | google | 20,000 tokens | ❌ Não | ✅ Sim | ❌ Não | ❌ Não |
| `google/gemini-embedding-2` | **Google: Gemini Embedding 2** | google | 8,192 tokens | ❌ Não | ✅ Sim | 👁️ Sim | ❌ Não |
| `google/gemini-embedding-2-preview` | **Google: Gemini Embedding 2 Preview** | google | 8,192 tokens | ❌ Não | ✅ Sim | 👁️ Sim | ❌ Não |
| `google/gemini-flash-latest` | **Google: Gemini Flash Latest** | google | 1,048,576 tokens | ✅ Sim | ✅ Sim | 👁️ Sim | ✅ Sim |
| `google/gemini-pro-latest` | **Google: Gemini Pro Latest** | google | 1,048,576 tokens | ✅ Sim | ✅ Sim | 👁️ Sim | ✅ Sim |
| `google/gemma-3-12b-it` | **Google: Gemma 3 12B** | google | 131,072 tokens | ✅ Sim | ✅ Sim | 👁️ Sim | ✅ Sim |
| `google/gemma-3-27b-it` | **Google: Gemma 3 27B** | google | 131,072 tokens | ✅ Sim | ✅ Sim | 👁️ Sim | ✅ Sim |
| `google/gemma-4-26b-a4b-it` | **Google: Gemma 4 26B A4B ** | google | 262,144 tokens | ✅ Sim | ✅ Sim | 👁️ Sim | ✅ Sim |
| `google/gemma-4-31b-it` | **Google: Gemma 4 31B** | google | 262,144 tokens | ✅ Sim | ✅ Sim | 👁️ Sim | ✅ Sim |
| `google/gemini-nano-banana-2.1` | **Google: Nano Banana 2.1** | google | 65,536 tokens | ✅ Sim | ✅ Sim | 👁️ Sim | ✅ Sim |
| `google/gemini-3-pro-image` | **Google: Nano Banana Pro (Gemini 3 Pro Image)** | google | 131,072 tokens | ✅ Sim | ✅ Sim | 👁️ Sim | ✅ Sim |
| `ibm-granite/granite-4.2-8b` | **IBM: Granite 4.2 8B** | ibm-granite | 131,072 tokens | ✅ Sim | ✅ Sim | ❌ Não | ✅ Sim |
| `inception/mercury-2` | **Inception: Mercury 2** | inception | 128,000 tokens | ✅ Sim | ✅ Sim | ❌ Não | ✅ Sim |
| `inception/mercury-2.5` | **Inception: Mercury 2.5** | inception | 260,000 tokens | ✅ Sim | ✅ Sim | ❌ Não | ✅ Sim |
| `inclusionai/ling-3.0-flash` | **inclusionAI: Ling 3.0 Flash** | inclusionai | 262,144 tokens | ✅ Sim | ✅ Sim | ❌ Não | ✅ Sim |
| `inclusionai/ling-3.0-flash-vl` | **inclusionAI: Ling 3.0 Flash VL** | inclusionai | 131,072 tokens | ✅ Sim | ✅ Sim | 👁️ Sim | ✅ Sim |
| `inclusionai/ling-3.1-flash` | **inclusionAI: Ling 3.1 Flash** | inclusionai | 262,144 tokens | ✅ Sim | ✅ Sim | ❌ Não | ❌ Não |
| `intfloat/e5-base-v2` | **Intfloat: E5-Base-v2** | intfloat | 512 tokens | ❌ Não | ✅ Sim | ❌ Não | ❌ Não |
| `intfloat/e5-large-v2` | **Intfloat: E5-Large-v2** | intfloat | 512 tokens | ❌ Não | ✅ Sim | ❌ Não | ❌ Não |
| `intfloat/multilingual-e5-large` | **Intfloat: Multilingual-E5-Large** | intfloat | 512 tokens | ❌ Não | ✅ Sim | ❌ Não | ❌ Não |
| `liquid/lfm-2.5-24b-a2b` | **LiquidAI: LFM2.5-24B-A2B** | liquid | 128,000 tokens | ✅ Sim | ✅ Sim | ❌ Não | ✅ Sim |
| `meituan/longcat-2.0` | **Meituan: LongCat 2.0** | meituan | 1,048,756 tokens | ✅ Sim | ✅ Sim | ❌ Não | ❌ Não |
| `meta/muse-glimmer-30b` | **Meta: Muse Glimmer 30B** | meta | 131,072 tokens | ✅ Sim | ✅ Sim | 👁️ Sim | ✅ Sim |
| `meta/muse-spark-1.1` | **Meta: Muse Spark 1.1** | meta | 1,048,576 tokens | ✅ Sim | ✅ Sim | 👁️ Sim | ✅ Sim |
| `meta/muse-spark-1.2` | **Meta: Muse Spark 1.2** | meta | 1,048,576 tokens | ✅ Sim | ✅ Sim | 👁️ Sim | ✅ Sim |
| `meta/muse-spark-1.3` | **Meta: Muse Spark 1.3** | meta | 1,048,576 tokens | ✅ Sim | ✅ Sim | 👁️ Sim | ✅ Sim |
| `meta/muse-wonder` | **Meta: Muse Wonder** | meta | 1,048,576 tokens | ✅ Sim | ✅ Sim | 👁️ Sim | ✅ Sim |
| `meta-llama/llama-3.1-70b-instruct` | **Meta: Llama 3.1 70B Instruct** | meta-llama | 131,072 tokens | ✅ Sim | ✅ Sim | ❌ Não | ✅ Sim |
| `meta-llama/llama-3.1-8b-instruct` | **Meta: Llama 3.1 8B Instruct** | meta-llama | 131,072 tokens | ✅ Sim | ✅ Sim | ❌ Não | ✅ Sim |
| `meta-llama/llama-3.3-70b-instruct` | **Meta: Llama 3.3 70B Instruct** | meta-llama | 131,072 tokens | ✅ Sim | ✅ Sim | ❌ Não | ✅ Sim |
| `meta-llama/llama-4-maverick` | **Meta: Llama 4 Maverick** | meta-llama | 1,048,576 tokens | ✅ Sim | ✅ Sim | 👁️ Sim | ✅ Sim |
| `meta-llama/llama-4-scout` | **Meta: Llama 4 Scout** | meta-llama | 1,310,720 tokens | ✅ Sim | ✅ Sim | 👁️ Sim | ✅ Sim |
| `minimax/minimax-m1` | **MiniMax: MiniMax M1** | minimax | 1,000,000 tokens | ✅ Sim | ✅ Sim | ❌ Não | ❌ Não |
| `minimax/minimax-m2` | **MiniMax: MiniMax M2** | minimax | 204,800 tokens | ✅ Sim | ✅ Sim | ❌ Não | ✅ Sim |
| `minimax/minimax-m2.1` | **MiniMax: MiniMax M2.1** | minimax | 204,800 tokens | ✅ Sim | ✅ Sim | ❌ Não | ✅ Sim |
| `minimax/minimax-m2.5` | **MiniMax: MiniMax M2.5** | minimax | 204,800 tokens | ✅ Sim | ✅ Sim | ❌ Não | ✅ Sim |
| `minimax/minimax-m2.7` | **MiniMax: MiniMax M2.7** | minimax | 204,800 tokens | ✅ Sim | ✅ Sim | ❌ Não | ✅ Sim |
| `minimax/minimax-m3` | **MiniMax: MiniMax M3** | minimax | 1,048,576 tokens | ✅ Sim | ✅ Sim | 👁️ Sim | ✅ Sim |
| `mistralai/mistral-large` | **Mistral Large** | mistralai | 128,000 tokens | ✅ Sim | ✅ Sim | ❌ Não | ✅ Sim |
| `mistralai/mistral-large-2407` | **Mistral Large 2407** | mistralai | 131,072 tokens | ✅ Sim | ✅ Sim | ❌ Não | ✅ Sim |
| `mistralai/codestral-2508` | **Mistral: Codestral 2508** | mistralai | 256,000 tokens | ✅ Sim | ✅ Sim | ❌ Não | ✅ Sim |
| `mistralai/codestral-embed-2505` | **Mistral: Codestral Embed 2505** | mistralai | 8,192 tokens | ❌ Não | ✅ Sim | ❌ Não | ❌ Não |
| `mistralai/devstral-2512` | **Mistral: Devstral 2 2512** | mistralai | 262,144 tokens | ✅ Sim | ✅ Sim | ❌ Não | ✅ Sim |
| `mistralai/ministral-14b-2512` | **Mistral: Ministral 3 14B 2512** | mistralai | 262,144 tokens | ✅ Sim | ✅ Sim | 👁️ Sim | ✅ Sim |
| `mistralai/ministral-3b-2512` | **Mistral: Ministral 3 3B 2512** | mistralai | 131,072 tokens | ✅ Sim | ✅ Sim | 👁️ Sim | ✅ Sim |
| `mistralai/ministral-8b-2512` | **Mistral: Ministral 3 8B 2512** | mistralai | 262,144 tokens | ✅ Sim | ✅ Sim | 👁️ Sim | ✅ Sim |
| `mistralai/mistral-embed-2312` | **Mistral: Mistral Embed 2312** | mistralai | 8,192 tokens | ❌ Não | ✅ Sim | ❌ Não | ❌ Não |
| `mistralai/mistral-large-2512` | **Mistral: Mistral Large 3 2512** | mistralai | 262,144 tokens | ✅ Sim | ✅ Sim | 👁️ Sim | ✅ Sim |
| `mistralai/mistral-large-4-0` | **Mistral: Mistral Large 4** | mistralai | 1,048,576 tokens | ✅ Sim | ✅ Sim | 👁️ Sim | ✅ Sim |
| `mistralai/mistral-medium-3` | **Mistral: Mistral Medium 3** | mistralai | 131,072 tokens | ✅ Sim | ✅ Sim | 👁️ Sim | ✅ Sim |
| `mistralai/mistral-medium-3.1` | **Mistral: Mistral Medium 3.1** | mistralai | 131,072 tokens | ✅ Sim | ✅ Sim | 👁️ Sim | ✅ Sim |
| `mistralai/mistral-medium-3-5` | **Mistral: Mistral Medium 3.5** | mistralai | 262,144 tokens | ✅ Sim | ✅ Sim | 👁️ Sim | ✅ Sim |
| `mistralai/mistral-nemo` | **Mistral: Mistral Nemo** | mistralai | 131,072 tokens | ✅ Sim | ✅ Sim | ❌ Não | ✅ Sim |
| `mistralai/mistral-small-3.1-24b-instruct` | **Mistral: Mistral Small 3.1 24B** | mistralai | 128,000 tokens | ✅ Sim | ✅ Sim | 👁️ Sim | ❌ Não |
| `mistralai/mistral-small-3.2-24b-instruct` | **Mistral: Mistral Small 3.2 24B** | mistralai | 256,000 tokens | ✅ Sim | ✅ Sim | 👁️ Sim | ✅ Sim |
| `mistralai/mistral-small-2603` | **Mistral: Mistral Small 4** | mistralai | 262,144 tokens | ✅ Sim | ✅ Sim | 👁️ Sim | ✅ Sim |
| `mistralai/mixtral-8x22b-instruct` | **Mistral: Mixtral 8x22B Instruct** | mistralai | 65,536 tokens | ✅ Sim | ✅ Sim | ❌ Não | ✅ Sim |
| `mistralai/mistral-saba` | **Mistral: Saba** | mistralai | 32,768 tokens | ✅ Sim | ✅ Sim | ❌ Não | ✅ Sim |
| `mistralai/voxtral-small-24b-2507` | **Mistral: Voxtral Small 24B 2507** | mistralai | 32,768 tokens | ✅ Sim | ✅ Sim | 👁️ Sim | ✅ Sim |
| `moonshotai/kimi-k2` | **MoonshotAI: Kimi K2 0711** | moonshotai | 131,072 tokens | ✅ Sim | ✅ Sim | ❌ Não | ❌ Não |
| `moonshotai/kimi-k2-0905` | **MoonshotAI: Kimi K2 0905** | moonshotai | 262,144 tokens | ✅ Sim | ✅ Sim | ❌ Não | ✅ Sim |
| `moonshotai/kimi-k2-thinking` | **MoonshotAI: Kimi K2 Thinking** | moonshotai | 262,144 tokens | ✅ Sim | ✅ Sim | ❌ Não | ✅ Sim |
| `moonshotai/kimi-k2.5` | **MoonshotAI: Kimi K2.5** | moonshotai | 262,144 tokens | ✅ Sim | ✅ Sim | 👁️ Sim | ✅ Sim |
| `moonshotai/kimi-k2.6` | **MoonshotAI: Kimi K2.6** | moonshotai | 262,144 tokens | ✅ Sim | ✅ Sim | 👁️ Sim | ✅ Sim |
| `moonshotai/kimi-k2.7-code` | **MoonshotAI: Kimi K2.7 Code** | moonshotai | 262,144 tokens | ✅ Sim | ✅ Sim | 👁️ Sim | ✅ Sim |
| `moonshotai/kimi-k3` | **MoonshotAI: Kimi K3** | moonshotai | 1,048,576 tokens | ✅ Sim | ✅ Sim | 👁️ Sim | ✅ Sim |
| `moonshotai/kimi-k3:US` | **MoonshotAI: Kimi K3** | moonshotai | 1,048,576 tokens | ✅ Sim | ✅ Sim | 👁️ Sim | ✅ Sim |
| `moonshotai/kimi-latest` | **MoonshotAI: Kimi Latest** | moonshotai | 1,048,576 tokens | ✅ Sim | ✅ Sim | 👁️ Sim | ✅ Sim |
| `nex-agi/nex-n2.5-pro` | **Nex AGI: Nex-N2.5-Pro** | nex-agi | 262,144 tokens | ✅ Sim | ✅ Sim | 👁️ Sim | ❌ Não |
| `nvidia/nemotron-3-nano-30b-a3b` | **NVIDIA: Nemotron 3 Nano 30B A3B** | nvidia | 262,144 tokens | ✅ Sim | ✅ Sim | ❌ Não | ✅ Sim |
| `nvidia/nemotron-3-super-120b-a12b` | **NVIDIA: Nemotron 3 Super** | nvidia | 262,144 tokens | ✅ Sim | ✅ Sim | ❌ Não | ✅ Sim |
| `nvidia/nemotron-3-ultra-550b-a55b` | **NVIDIA: Nemotron 3 Ultra** | nvidia | 262,144 tokens | ✅ Sim | ✅ Sim | ❌ Não | ✅ Sim |
| `nvidia/nemotron-3.5-lightning` | **NVIDIA: Nemotron 3.5 Lightning** | nvidia | 262,144 tokens | ✅ Sim | ✅ Sim | ❌ Não | ✅ Sim |
| `openai/gpt-astra-latest` | **OpenAI: GPT Astra Latest** | openai | 1,050,000 tokens | ✅ Sim | ✅ Sim | 👁️ Sim | ✅ Sim |
| `openai/gpt-audio` | **OpenAI: GPT Audio** | openai | 128,000 tokens | ✅ Sim | ✅ Sim | 👁️ Sim | ✅ Sim |
| `openai/gpt-audio-mini` | **OpenAI: GPT Audio Mini** | openai | 128,000 tokens | ✅ Sim | ✅ Sim | 👁️ Sim | ✅ Sim |
| `openai/gpt-chat-latest` | **OpenAI: GPT Chat Latest** | openai | 400,000 tokens | ✅ Sim | ✅ Sim | 👁️ Sim | ✅ Sim |
| `openai/gpt-luna-latest` | **OpenAI: GPT Luna Latest** | openai | 1,050,000 tokens | ✅ Sim | ✅ Sim | 👁️ Sim | ✅ Sim |
| `openai/gpt-mini-latest` | **OpenAI: GPT Mini Latest** | openai | 400,000 tokens | ✅ Sim | ✅ Sim | 👁️ Sim | ✅ Sim |
| `openai/gpt-sol-latest` | **OpenAI: GPT Sol Latest** | openai | 1,050,000 tokens | ✅ Sim | ✅ Sim | 👁️ Sim | ✅ Sim |
| `openai/gpt-terra-latest` | **OpenAI: GPT Terra Latest** | openai | 1,050,000 tokens | ✅ Sim | ✅ Sim | 👁️ Sim | ✅ Sim |
| `openai/gpt-3.5-turbo` | **OpenAI: GPT-3.5 Turbo** | openai | 16,385 tokens | ✅ Sim | ✅ Sim | ❌ Não | ✅ Sim |
| `openai/gpt-3.5-turbo-0613` | **OpenAI: GPT-3.5 Turbo (older v0613)** | openai | 4,095 tokens | ✅ Sim | ✅ Sim | ❌ Não | ✅ Sim |
| `openai/gpt-3.5-turbo-16k` | **OpenAI: GPT-3.5 Turbo 16k** | openai | 16,385 tokens | ✅ Sim | ✅ Sim | ❌ Não | ✅ Sim |
| `openai/gpt-4` | **OpenAI: GPT-4** | openai | 8,191 tokens | ✅ Sim | ✅ Sim | ❌ Não | ✅ Sim |
| `openai/gpt-4-turbo` | **OpenAI: GPT-4 Turbo** | openai | 128,000 tokens | ✅ Sim | ✅ Sim | 👁️ Sim | ✅ Sim |
| `openai/gpt-4.1` | **OpenAI: GPT-4.1** | openai | 1,047,576 tokens | ✅ Sim | ✅ Sim | 👁️ Sim | ✅ Sim |
| `openai/gpt-4.1-mini` | **OpenAI: GPT-4.1 Mini** | openai | 1,047,576 tokens | ✅ Sim | ✅ Sim | 👁️ Sim | ✅ Sim |
| `openai/gpt-4.1-nano` | **OpenAI: GPT-4.1 Nano** | openai | 1,047,576 tokens | ✅ Sim | ✅ Sim | 👁️ Sim | ✅ Sim |
| `openai/gpt-4o` | **OpenAI: GPT-4o** | openai | 128,000 tokens | ✅ Sim | ✅ Sim | 👁️ Sim | ✅ Sim |
| `openai/gpt-4o-2024-05-13` | **OpenAI: GPT-4o (2024-05-13)** | openai | 128,000 tokens | ✅ Sim | ✅ Sim | 👁️ Sim | ✅ Sim |
| `openai/gpt-4o-2024-08-06` | **OpenAI: GPT-4o (2024-08-06)** | openai | 128,000 tokens | ✅ Sim | ✅ Sim | 👁️ Sim | ✅ Sim |
| `openai/gpt-4o-2024-11-20` | **OpenAI: GPT-4o (2024-11-20)** | openai | 128,000 tokens | ✅ Sim | ✅ Sim | 👁️ Sim | ✅ Sim |
| `openai/gpt-4o-mini` | **OpenAI: GPT-4o-mini** | openai | 128,000 tokens | ✅ Sim | ✅ Sim | 👁️ Sim | ✅ Sim |
| `openai/gpt-4o-mini-2024-07-18` | **OpenAI: GPT-4o-mini (2024-07-18)** | openai | 128,000 tokens | ✅ Sim | ✅ Sim | 👁️ Sim | ✅ Sim |
| `openai/gpt-5` | **OpenAI: GPT-5** | openai | 400,000 tokens | ✅ Sim | ✅ Sim | 👁️ Sim | ✅ Sim |
| `openai/gpt-5-mini` | **OpenAI: GPT-5 Mini** | openai | 400,000 tokens | ✅ Sim | ✅ Sim | 👁️ Sim | ✅ Sim |
| `openai/gpt-5-nano` | **OpenAI: GPT-5 Nano** | openai | 400,000 tokens | ✅ Sim | ✅ Sim | 👁️ Sim | ✅ Sim |
| `openai/gpt-5-pro` | **OpenAI: GPT-5 Pro** | openai | 400,000 tokens | ✅ Sim | ✅ Sim | 👁️ Sim | ✅ Sim |
| `openai/gpt-5.1` | **OpenAI: GPT-5.1** | openai | 400,000 tokens | ✅ Sim | ✅ Sim | 👁️ Sim | ✅ Sim |
| `openai/gpt-5.1-codex` | **OpenAI: GPT-5.1-Codex** | openai | 400,000 tokens | ✅ Sim | ✅ Sim | 👁️ Sim | ✅ Sim |
| `openai/gpt-5.1-codex-max` | **OpenAI: GPT-5.1-Codex-Max** | openai | 400,000 tokens | ✅ Sim | ✅ Sim | 👁️ Sim | ✅ Sim |
| `openai/gpt-5.1-codex-mini` | **OpenAI: GPT-5.1-Codex-Mini** | openai | 400,000 tokens | ✅ Sim | ✅ Sim | 👁️ Sim | ✅ Sim |
| `openai/gpt-5.2` | **OpenAI: GPT-5.2** | openai | 400,000 tokens | ✅ Sim | ✅ Sim | 👁️ Sim | ✅ Sim |
| `openai/gpt-5.2-chat` | **OpenAI: GPT-5.2 Chat** | openai | 128,000 tokens | ✅ Sim | ✅ Sim | 👁️ Sim | ✅ Sim |
| `openai/gpt-5.2-pro` | **OpenAI: GPT-5.2 Pro** | openai | 400,000 tokens | ✅ Sim | ✅ Sim | 👁️ Sim | ✅ Sim |
| `openai/gpt-5.2-codex` | **OpenAI: GPT-5.2-Codex** | openai | 400,000 tokens | ✅ Sim | ✅ Sim | 👁️ Sim | ✅ Sim |
| `openai/gpt-5.3-codex` | **OpenAI: GPT-5.3-Codex** | openai | 400,000 tokens | ✅ Sim | ✅ Sim | 👁️ Sim | ✅ Sim |
| `openai/gpt-5.4` | **OpenAI: GPT-5.4** | openai | 1,050,000 tokens | ✅ Sim | ✅ Sim | 👁️ Sim | ✅ Sim |
| `openai/gpt-5.4-mini` | **OpenAI: GPT-5.4 Mini** | openai | 400,000 tokens | ✅ Sim | ✅ Sim | 👁️ Sim | ✅ Sim |
| `openai/gpt-5.4-nano` | **OpenAI: GPT-5.4 Nano** | openai | 400,000 tokens | ✅ Sim | ✅ Sim | 👁️ Sim | ✅ Sim |
| `openai/gpt-5.4-pro` | **OpenAI: GPT-5.4 Pro** | openai | 1,050,000 tokens | ✅ Sim | ✅ Sim | 👁️ Sim | ✅ Sim |
| `openai/gpt-5.5` | **OpenAI: GPT-5.5** | openai | 1,050,000 tokens | ✅ Sim | ✅ Sim | 👁️ Sim | ✅ Sim |
| `openai/gpt-5.5-pro` | **OpenAI: GPT-5.5 Pro** | openai | 1,050,000 tokens | ✅ Sim | ✅ Sim | 👁️ Sim | ✅ Sim |
| `openai/gpt-5.6-luna` | **OpenAI: GPT-5.6 Luna** | openai | 1,050,000 tokens | ✅ Sim | ✅ Sim | 👁️ Sim | ✅ Sim |
| `openai/gpt-5.6-luna-pro` | **OpenAI: GPT-5.6 Luna Pro** | openai | 1,050,000 tokens | ✅ Sim | ✅ Sim | 👁️ Sim | ✅ Sim |
| `openai/gpt-5.6-sol` | **OpenAI: GPT-5.6 Sol** | openai | 1,050,000 tokens | ✅ Sim | ✅ Sim | 👁️ Sim | ✅ Sim |
| `openai/gpt-5.6-sol-pro` | **OpenAI: GPT-5.6 Sol Pro** | openai | 1,050,000 tokens | ✅ Sim | ✅ Sim | 👁️ Sim | ✅ Sim |
| `openai/gpt-5.6-terra` | **OpenAI: GPT-5.6 Terra** | openai | 1,050,000 tokens | ✅ Sim | ✅ Sim | 👁️ Sim | ✅ Sim |
| `openai/gpt-5.6-terra-pro` | **OpenAI: GPT-5.6 Terra Pro** | openai | 1,050,000 tokens | ✅ Sim | ✅ Sim | 👁️ Sim | ✅ Sim |
| `openai/gpt-6-astra` | **OpenAI: GPT-6 Astra** | openai | 1,050,000 tokens | ✅ Sim | ✅ Sim | 👁️ Sim | ✅ Sim |
| `openai/gpt-6-astra-fast` | **OpenAI: GPT-6 Astra Fast** | openai | 1,050,000 tokens | ✅ Sim | ✅ Sim | 👁️ Sim | ✅ Sim |
| `openai/gpt-6-astra-flex` | **OpenAI: GPT-6 Astra Flex** | openai | 1,050,000 tokens | ✅ Sim | ✅ Sim | 👁️ Sim | ✅ Sim |
| `openai/gpt-6-astra-pro` | **OpenAI: GPT-6 Astra Pro** | openai | 1,050,000 tokens | ✅ Sim | ✅ Sim | 👁️ Sim | ✅ Sim |
| `openai/gpt-6-astra-pro-fast` | **OpenAI: GPT-6 Astra Pro Fast** | openai | 1,050,000 tokens | ✅ Sim | ✅ Sim | 👁️ Sim | ✅ Sim |
| `openai/gpt-6-astra-pro-flex` | **OpenAI: GPT-6 Astra Pro Flex** | openai | 1,050,000 tokens | ✅ Sim | ✅ Sim | 👁️ Sim | ✅ Sim |
| `openai/gpt-6-luna` | **OpenAI: GPT-6 Luna** | openai | 1,050,000 tokens | ✅ Sim | ✅ Sim | 👁️ Sim | ✅ Sim |
| `openai/gpt-6-luna-pro` | **OpenAI: GPT-6 Luna Pro** | openai | 1,050,000 tokens | ✅ Sim | ✅ Sim | 👁️ Sim | ✅ Sim |
| `openai/gpt-6-sol` | **OpenAI: GPT-6 Sol** | openai | 1,050,000 tokens | ✅ Sim | ✅ Sim | 👁️ Sim | ✅ Sim |
| `openai/gpt-6-sol-pro` | **OpenAI: GPT-6 Sol Pro** | openai | 1,050,000 tokens | ✅ Sim | ✅ Sim | 👁️ Sim | ✅ Sim |
| `openai/gpt-6.1-sol` | **OpenAI: GPT-6.1 Sol** | openai | 1,050,000 tokens | ✅ Sim | ✅ Sim | 👁️ Sim | ✅ Sim |
| `openai/gpt-6.1-sol-pro` | **OpenAI: GPT-6.1 Sol Pro** | openai | 1,050,000 tokens | ✅ Sim | ✅ Sim | 👁️ Sim | ✅ Sim |
| `openai/text-embedding-3-large` | **OpenAI: Text Embedding 3 Large** | openai | 8,192 tokens | ❌ Não | ✅ Sim | ❌ Não | ❌ Não |
| `openai/text-embedding-3-small` | **OpenAI: Text Embedding 3 Small** | openai | 8,192 tokens | ❌ Não | ✅ Sim | ❌ Não | ❌ Não |
| `openai/text-embedding-ada-002` | **OpenAI: Text Embedding Ada 002** | openai | 8,192 tokens | ❌ Não | ✅ Sim | ❌ Não | ❌ Não |
| `openai/gpt-oss-120b` | **OpenAI: gpt-oss-120b** | openai | 131,072 tokens | ✅ Sim | ✅ Sim | ❌ Não | ✅ Sim |
| `openai/gpt-oss-20b` | **OpenAI: gpt-oss-20b** | openai | 131,072 tokens | ✅ Sim | ✅ Sim | ❌ Não | ✅ Sim |
| `openai/gpt-oss-safeguard-20b` | **OpenAI: gpt-oss-safeguard-20b** | openai | 131,072 tokens | ✅ Sim | ✅ Sim | ❌ Não | ✅ Sim |
| `openai/o1` | **OpenAI: o1** | openai | 200,000 tokens | ✅ Sim | ✅ Sim | 👁️ Sim | ✅ Sim |
| `openai/o3` | **OpenAI: o3** | openai | 200,000 tokens | ✅ Sim | ✅ Sim | 👁️ Sim | ✅ Sim |
| `openai/o3-mini` | **OpenAI: o3 Mini** | openai | 200,000 tokens | ✅ Sim | ✅ Sim | ❌ Não | ✅ Sim |
| `openai/o3-mini-high` | **OpenAI: o3 Mini High** | openai | 200,000 tokens | ✅ Sim | ✅ Sim | ❌ Não | ✅ Sim |
| `openai/o3-pro` | **OpenAI: o3 Pro** | openai | 200,000 tokens | ✅ Sim | ✅ Sim | 👁️ Sim | ✅ Sim |
| `openai/o4-mini` | **OpenAI: o4 Mini** | openai | 200,000 tokens | ✅ Sim | ✅ Sim | 👁️ Sim | ✅ Sim |
| `openai/o4-mini-high` | **OpenAI: o4 Mini High** | openai | 200,000 tokens | ✅ Sim | ✅ Sim | 👁️ Sim | ✅ Sim |
| `perceptron/perceptron-mk1.5` | **Perceptron: Perceptron Mk1.5** | perceptron | 36,864 tokens | ✅ Sim | ✅ Sim | 👁️ Sim | ❌ Não |
| `perplexity/pplx-embed-v1-0.6b` | **Perplexity: Embed V1 0.6B** | perplexity | 32,000 tokens | ❌ Não | ✅ Sim | ❌ Não | ❌ Não |
| `perplexity/pplx-embed-v1-4b` | **Perplexity: Embed V1 4B** | perplexity | 32,000 tokens | ❌ Não | ✅ Sim | ❌ Não | ❌ Não |
| `poolside/laguna-s-2.1` | **Poolside: Laguna S 2.1** | poolside | 1,048,576 tokens | ✅ Sim | ✅ Sim | ❌ Não | ❌ Não |
| `poolside/laguna-xs-2.1` | **Poolside: Laguna XS 2.1** | poolside | 262,144 tokens | ✅ Sim | ✅ Sim | ❌ Não | ❌ Não |
| `prism-ml/ternary-bonsai-2-27b` | **PrismML: Ternary Bonsai 2 27B** | prism-ml | 262,144 tokens | ✅ Sim | ✅ Sim | 👁️ Sim | ✅ Sim |
| `qwen/qwen-2.5-72b-instruct` | **Qwen2.5 72B Instruct** | qwen | 32,768 tokens | ✅ Sim | ✅ Sim | ❌ Não | ✅ Sim |
| `qwen/qwen-plus` | **Qwen: Qwen-Plus** | qwen | 1,000,000 tokens | ✅ Sim | ✅ Sim | ❌ Não | ✅ Sim |
| `qwen/qwen-2.5-7b-instruct` | **Qwen: Qwen2.5 7B Instruct** | qwen | 32,768 tokens | ✅ Sim | ✅ Sim | ❌ Não | ✅ Sim |
| `qwen/qwen3-14b` | **Qwen: Qwen3 14B** | qwen | 40,960 tokens | ✅ Sim | ✅ Sim | ❌ Não | ✅ Sim |
| `qwen/qwen3-235b-a22b-2507` | **Qwen: Qwen3 235B A22B Instruct 2507** | qwen | 262,144 tokens | ✅ Sim | ✅ Sim | ❌ Não | ✅ Sim |
| `qwen/qwen3-235b-a22b-thinking-2507` | **Qwen: Qwen3 235B A22B Thinking 2507** | qwen | 128,000 tokens | ✅ Sim | ✅ Sim | ❌ Não | ❌ Não |
| `qwen/qwen3-30b-a3b` | **Qwen: Qwen3 30B A3B** | qwen | 40,960 tokens | ✅ Sim | ✅ Sim | ❌ Não | ✅ Sim |
| `qwen/qwen3-30b-a3b-instruct-2507` | **Qwen: Qwen3 30B A3B Instruct 2507** | qwen | 262,144 tokens | ✅ Sim | ✅ Sim | ❌ Não | ✅ Sim |
| `qwen/qwen3-32b` | **Qwen: Qwen3 32B** | qwen | 131,072 tokens | ✅ Sim | ✅ Sim | ❌ Não | ✅ Sim |
| `qwen/qwen3-coder-30b-a3b-instruct` | **Qwen: Qwen3 Coder 30B A3B Instruct** | qwen | 262,144 tokens | ✅ Sim | ✅ Sim | ❌ Não | ✅ Sim |
| `qwen/qwen3-coder` | **Qwen: Qwen3 Coder 480B A35B** | qwen | 262,144 tokens | ✅ Sim | ✅ Sim | ❌ Não | ✅ Sim |
| `qwen/qwen3-coder-flash` | **Qwen: Qwen3 Coder Flash** | qwen | 1,000,000 tokens | ✅ Sim | ✅ Sim | ❌ Não | ✅ Sim |
| `qwen/qwen3-coder-next` | **Qwen: Qwen3 Coder Next** | qwen | 262,144 tokens | ✅ Sim | ✅ Sim | ❌ Não | ✅ Sim |
| `qwen/qwen3-embedding-4b` | **Qwen: Qwen3 Embedding 4B** | qwen | 32,768 tokens | ❌ Não | ✅ Sim | ❌ Não | ❌ Não |
| `qwen/qwen3-embedding-8b` | **Qwen: Qwen3 Embedding 8B** | qwen | 32,768 tokens | ❌ Não | ✅ Sim | ❌ Não | ❌ Não |
| `qwen/qwen3-next-80b-a3b-instruct` | **Qwen: Qwen3 Next 80B A3B Instruct** | qwen | 262,144 tokens | ✅ Sim | ✅ Sim | ❌ Não | ✅ Sim |
| `qwen/qwen3-next-80b-a3b-thinking` | **Qwen: Qwen3 Next 80B A3B Thinking** | qwen | 262,144 tokens | ✅ Sim | ✅ Sim | ❌ Não | ✅ Sim |
| `qwen/qwen3-vl-235b-a22b-instruct` | **Qwen: Qwen3 VL 235B A22B Instruct** | qwen | 262,144 tokens | ✅ Sim | ✅ Sim | 👁️ Sim | ✅ Sim |
| `qwen/qwen3-vl-30b-a3b-instruct` | **Qwen: Qwen3 VL 30B A3B Instruct** | qwen | 262,144 tokens | ✅ Sim | ✅ Sim | 👁️ Sim | ✅ Sim |
| `qwen/qwen3-vl-30b-a3b-thinking` | **Qwen: Qwen3 VL 30B A3B Thinking** | qwen | 262,144 tokens | ✅ Sim | ✅ Sim | 👁️ Sim | ✅ Sim |
| `qwen/qwen3-vl-8b-instruct` | **Qwen: Qwen3 VL 8B Instruct** | qwen | 262,144 tokens | ✅ Sim | ✅ Sim | 👁️ Sim | ✅ Sim |
| `qwen/qwen3.5-397b-a17b` | **Qwen: Qwen3.5 397B A17B** | qwen | 262,144 tokens | ✅ Sim | ✅ Sim | 👁️ Sim | ✅ Sim |
| `qwen/qwen3.5-plus-02-15` | **Qwen: Qwen3.5 Plus 2026-02-15** | qwen | 1,000,000 tokens | ✅ Sim | ✅ Sim | 👁️ Sim | ✅ Sim |
| `qwen/qwen3.5-plus-20260420` | **Qwen: Qwen3.5 Plus 2026-04-20** | qwen | 1,000,000 tokens | ✅ Sim | ✅ Sim | 👁️ Sim | ✅ Sim |
| `qwen/qwen3.5-122b-a10b` | **Qwen: Qwen3.5-122B-A10B** | qwen | 262,144 tokens | ✅ Sim | ✅ Sim | 👁️ Sim | ✅ Sim |
| `qwen/qwen3.5-27b` | **Qwen: Qwen3.5-27B** | qwen | 262,144 tokens | ✅ Sim | ✅ Sim | 👁️ Sim | ✅ Sim |
| `qwen/qwen3.5-35b-a3b` | **Qwen: Qwen3.5-35B-A3B** | qwen | 262,144 tokens | ✅ Sim | ✅ Sim | 👁️ Sim | ✅ Sim |
| `qwen/qwen3.5-9b` | **Qwen: Qwen3.5-9B** | qwen | 262,144 tokens | ✅ Sim | ✅ Sim | 👁️ Sim | ✅ Sim |
| `qwen/qwen3.5-flash-02-23` | **Qwen: Qwen3.5-Flash** | qwen | 1,000,000 tokens | ✅ Sim | ✅ Sim | 👁️ Sim | ✅ Sim |
| `qwen/qwen3.6-27b` | **Qwen: Qwen3.6 27B** | qwen | 262,144 tokens | ✅ Sim | ✅ Sim | 👁️ Sim | ✅ Sim |
| `qwen/qwen3.6-35b-a3b` | **Qwen: Qwen3.6 35B A3B** | qwen | 262,144 tokens | ✅ Sim | ✅ Sim | 👁️ Sim | ✅ Sim |
| `qwen/qwen3.6-flash` | **Qwen: Qwen3.6 Flash** | qwen | 1,000,000 tokens | ✅ Sim | ✅ Sim | 👁️ Sim | ✅ Sim |
| `qwen/qwen3.6-plus` | **Qwen: Qwen3.6 Plus** | qwen | 1,000,000 tokens | ✅ Sim | ✅ Sim | 👁️ Sim | ✅ Sim |
| `qwen/qwen3.7-flash` | **Qwen: Qwen3.7 Flash** | qwen | 1,000,000 tokens | ✅ Sim | ✅ Sim | 👁️ Sim | ✅ Sim |
| `qwen/qwen3.7-max` | **Qwen: Qwen3.7 Max** | qwen | 1,000,000 tokens | ✅ Sim | ✅ Sim | ❌ Não | ✅ Sim |
| `qwen/qwen3.7-plus` | **Qwen: Qwen3.7 Plus** | qwen | 1,000,000 tokens | ✅ Sim | ✅ Sim | 👁️ Sim | ✅ Sim |
| `qwen/qwen3.8-2.4t-a95b` | **Qwen: Qwen3.8 2.4T A95B** | qwen | 1,048,576 tokens | ✅ Sim | ✅ Sim | ❌ Não | ✅ Sim |
| `qwen/qwen3.8-27b` | **Qwen: Qwen3.8 27B** | qwen | 1,000,000 tokens | ✅ Sim | ✅ Sim | 👁️ Sim | ✅ Sim |
| `qwen/qwen3.8-flash` | **Qwen: Qwen3.8 Flash** | qwen | 1,000,000 tokens | ✅ Sim | ✅ Sim | 👁️ Sim | ✅ Sim |
| `qwen/qwen3.8-max-0902` | **Qwen: Qwen3.8 Max (0902)** | qwen | 1,000,000 tokens | ✅ Sim | ✅ Sim | 👁️ Sim | ✅ Sim |
| `qwen/qwen3.8-max-prime` | **Qwen: Qwen3.8 Max Prime** | qwen | 1,000,000 tokens | ✅ Sim | ✅ Sim | 👁️ Sim | ✅ Sim |
| `qwen/qwen3.8-omni-flash` | **Qwen: Qwen3.8 Omni Flash** | qwen | 1,000,000 tokens | ✅ Sim | ✅ Sim | 👁️ Sim | ✅ Sim |
| `rekaai/reka-edge` | **Reka Edge** | rekaai | 16,384 tokens | ✅ Sim | ✅ Sim | 👁️ Sim | ❌ Não |
| `relace/relace-search` | **Relace: Relace Search** | relace | 256,000 tokens | ✅ Sim | ✅ Sim | ❌ Não | ✅ Sim |
| `sakana/fugu-max` | **Sakana: Fugu Max** | sakana | 1,000,000 tokens | ✅ Sim | ✅ Sim | 👁️ Sim | ❌ Não |
| `sakana/fugu-ultra` | **Sakana: Fugu Ultra** | sakana | 1,000,000 tokens | ✅ Sim | ✅ Sim | 👁️ Sim | ❌ Não |
| `sakana/fugu-ultra-v2` | **Sakana: Fugu Ultra v2** | sakana | 1,000,000 tokens | ✅ Sim | ✅ Sim | 👁️ Sim | ❌ Não |
| `sentence-transformers/all-minilm-l12-v2` | **Sentence Transformers: all-MiniLM-L12-v2** | sentence-transformers | 512 tokens | ❌ Não | ✅ Sim | ❌ Não | ❌ Não |
| `sentence-transformers/all-minilm-l6-v2` | **Sentence Transformers: all-MiniLM-L6-v2** | sentence-transformers | 512 tokens | ❌ Não | ✅ Sim | ❌ Não | ❌ Não |
| `sentence-transformers/all-mpnet-base-v2` | **Sentence Transformers: all-mpnet-base-v2** | sentence-transformers | 512 tokens | ❌ Não | ✅ Sim | ❌ Não | ❌ Não |
| `sentence-transformers/multi-qa-mpnet-base-dot-v1` | **Sentence Transformers: multi-qa-mpnet-base-dot-v1** | sentence-transformers | 512 tokens | ❌ Não | ✅ Sim | ❌ Não | ❌ Não |
| `sentence-transformers/paraphrase-minilm-l6-v2` | **Sentence Transformers: paraphrase-MiniLM-L6-v2** | sentence-transformers | 512 tokens | ❌ Não | ✅ Sim | ❌ Não | ❌ Não |
| `stealth/missingno` | **MissingNo** | stealth | 262,144 tokens | ✅ Sim | ❌ Não | ❌ Não | ✅ Sim |
| `stepfun/step-3.5-flash` | **StepFun: Step 3.5 Flash** | stepfun | 262,144 tokens | ✅ Sim | ✅ Sim | ❌ Não | ❌ Não |
| `stepfun/step-3.7-flash` | **StepFun: Step 3.7 Flash** | stepfun | 262,144 tokens | ✅ Sim | ✅ Sim | 👁️ Sim | ✅ Sim |
| `stepfun/step-5-preview` | **StepFun: Step 5 Preview** | stepfun | 1,000,000 tokens | ✅ Sim | ✅ Sim | 👁️ Sim | ✅ Sim |
| `tencent/hy3` | **Tencent: Hy3** | tencent | 262,144 tokens | ✅ Sim | ✅ Sim | ❌ Não | ✅ Sim |
| `tencent/hy3-preview` | **Tencent: Hy3 preview** | tencent | 262,144 tokens | ✅ Sim | ✅ Sim | ❌ Não | ❌ Não |
| `tencent/hy4-preview` | **Tencent: Hy4 preview** | tencent | 1,048,576 tokens | ✅ Sim | ✅ Sim | ❌ Não | ✅ Sim |
| `thenlper/gte-base` | **Thenlper: GTE-Base** | thenlper | 512 tokens | ❌ Não | ✅ Sim | ❌ Não | ❌ Não |
| `thenlper/gte-large` | **Thenlper: GTE-Large** | thenlper | 512 tokens | ❌ Não | ✅ Sim | ❌ Não | ❌ Não |
| `thinkingmachines/inkling` | **Thinking Machines: Inkling** | thinkingmachines | 524,288 tokens | ✅ Sim | ✅ Sim | 👁️ Sim | ❌ Não |
| `thinkingmachines/inkling-small` | **Thinking Machines: Inkling Small** | thinkingmachines | 524,288 tokens | ✅ Sim | ✅ Sim | 👁️ Sim | ❌ Não |
| `unbiased/pareto-26.10-preview` | **Pareto 26.10 Preview** | unbiased | 1,048,576 tokens | ✅ Sim | ✅ Sim | 👁️ Sim | ❌ Não |
| `unbiased/pareto-26.9` | **Pareto 26.9** | unbiased | 262,144 tokens | ✅ Sim | ✅ Sim | 👁️ Sim | ✅ Sim |
| `unbiased/pareto-latest` | **Pareto Latest** | unbiased | 262,144 tokens | ✅ Sim | ✅ Sim | 👁️ Sim | ✅ Sim |
| `upstage/solar-mini4` | **Upstage: Solar Mini 4** | upstage | 524,288 tokens | ✅ Sim | ✅ Sim | ❌ Não | ✅ Sim |
| `upstage/solar-pro-3` | **Upstage: Solar Pro 3** | upstage | 131,072 tokens | ✅ Sim | ✅ Sim | ❌ Não | ✅ Sim |
| `upstage/solar-pro4` | **Upstage: Solar Pro 4** | upstage | 524,288 tokens | ✅ Sim | ✅ Sim | ❌ Não | ✅ Sim |
| `voyageai/voyage-4` | **VoyageAI by MongoDB: voyage-4** | voyageai | 32,000 tokens | ❌ Não | ✅ Sim | ❌ Não | ❌ Não |
| `voyageai/voyage-4-large` | **VoyageAI by MongoDB: voyage-4-large** | voyageai | 32,000 tokens | ❌ Não | ✅ Sim | ❌ Não | ❌ Não |
| `voyageai/voyage-4-lite` | **VoyageAI by MongoDB: voyage-4-lite** | voyageai | 32,000 tokens | ❌ Não | ✅ Sim | ❌ Não | ❌ Não |
| `voyageai/voyage-code-4` | **VoyageAI by MongoDB: voyage-code-4** | voyageai | 32,000 tokens | ❌ Não | ✅ Sim | ❌ Não | ❌ Não |
| `voyageai/voyage-multimodal-3.5` | **VoyageAI by MongoDB: voyage-multimodal-3.5** | voyageai | 32,000 tokens | ❌ Não | ✅ Sim | 👁️ Sim | ❌ Não |
| `x-ai/grok-4.20` | **SpaceXAI: Grok 4.20** | x-ai | 2,000,000 tokens | ✅ Sim | ✅ Sim | 👁️ Sim | ✅ Sim |
| `x-ai/grok-4.3` | **SpaceXAI: Grok 4.3** | x-ai | 1,000,000 tokens | ✅ Sim | ✅ Sim | 👁️ Sim | ✅ Sim |
| `x-ai/grok-build-0.1` | **SpaceXAI: Grok Build 0.1** | x-ai | 256,000 tokens | ✅ Sim | ✅ Sim | 👁️ Sim | ✅ Sim |
| `x-ai/grok-4.5` | **xAI: Grok 4.5** | x-ai | 500,000 tokens | ✅ Sim | ✅ Sim | 👁️ Sim | ✅ Sim |
| `x-ai/grok-4.6` | **xAI: Grok 4.6** | x-ai | 500,000 tokens | ✅ Sim | ✅ Sim | 👁️ Sim | ✅ Sim |
| `x-ai/grok-4.7` | **xAI: Grok 4.7** | x-ai | 500,000 tokens | ✅ Sim | ✅ Sim | 👁️ Sim | ✅ Sim |
| `x-ai/grok-latest` | **xAI: Grok Latest** | x-ai | 500,000 tokens | ✅ Sim | ✅ Sim | 👁️ Sim | ✅ Sim |
| `xiaomi/mimo-v2.5` | **Xiaomi: MiMo-V2.5** | xiaomi | 1,050,000 tokens | ✅ Sim | ✅ Sim | 👁️ Sim | ✅ Sim |
| `xiaomi/mimo-v2.5-pro` | **Xiaomi: MiMo-V2.5-Pro** | xiaomi | 1,050,000 tokens | ✅ Sim | ✅ Sim | ❌ Não | ✅ Sim |
| `xiaomi/mimo-v2.6-flash` | **Xiaomi: MiMo-V2.6-Flash** | xiaomi | 1,050,000 tokens | ✅ Sim | ✅ Sim | 👁️ Sim | ✅ Sim |
| `xiaomi/mimo-v2.6-pro` | **Xiaomi: MiMo-V2.6-Pro** | xiaomi | 1,050,000 tokens | ✅ Sim | ✅ Sim | 👁️ Sim | ✅ Sim |
| `xiaomi/mimo-v2.6-pro-ultraspeed` | **Xiaomi: MiMo-V2.6-Pro-UltraSpeed** | xiaomi | 1,048,576 tokens | ✅ Sim | ✅ Sim | 👁️ Sim | ✅ Sim |
| `z-ai/glm-4.5` | **Z.ai: GLM 4.5** | z-ai | 131,072 tokens | ✅ Sim | ✅ Sim | ❌ Não | ✅ Sim |
| `z-ai/glm-4.5-air` | **Z.ai: GLM 4.5 Air** | z-ai | 131,072 tokens | ✅ Sim | ✅ Sim | ❌ Não | ❌ Não |
| `z-ai/glm-4.5v` | **Z.ai: GLM 4.5V** | z-ai | 65,536 tokens | ✅ Sim | ✅ Sim | 👁️ Sim | ✅ Sim |
| `z-ai/glm-4.6` | **Z.ai: GLM 4.6** | z-ai | 204,800 tokens | ✅ Sim | ✅ Sim | ❌ Não | ✅ Sim |
| `z-ai/glm-4.6v` | **Z.ai: GLM 4.6V** | z-ai | 131,072 tokens | ✅ Sim | ✅ Sim | 👁️ Sim | ✅ Sim |
| `z-ai/glm-4.7` | **Z.ai: GLM 4.7** | z-ai | 204,800 tokens | ✅ Sim | ✅ Sim | ❌ Não | ✅ Sim |
| `z-ai/glm-4.7-flash` | **Z.ai: GLM 4.7 Flash** | z-ai | 200,000 tokens | ✅ Sim | ✅ Sim | ❌ Não | ✅ Sim |
| `z-ai/glm-5` | **Z.ai: GLM 5** | z-ai | 204,800 tokens | ✅ Sim | ✅ Sim | ❌ Não | ✅ Sim |
| `z-ai/glm-5-turbo` | **Z.ai: GLM 5 Turbo** | z-ai | 202,752 tokens | ✅ Sim | ✅ Sim | ❌ Não | ✅ Sim |
| `z-ai/glm-5.1` | **Z.ai: GLM 5.1** | z-ai | 204,800 tokens | ✅ Sim | ✅ Sim | ❌ Não | ✅ Sim |
| `z-ai/glm-5.2` | **Z.ai: GLM 5.2** | z-ai | 1,048,576 tokens | ✅ Sim | ✅ Sim | ❌ Não | ✅ Sim |
| `z-ai/glm-5.3:US` | **Z.ai: GLM 5.3** | z-ai | 1,048,576 tokens | ✅ Sim | ✅ Sim | ❌ Não | ✅ Sim |
| `z-ai/glm-5.3` | **Z.ai: GLM 5.3** | z-ai | 1,048,576 tokens | ✅ Sim | ✅ Sim | ❌ Não | ✅ Sim |
| `z-ai/glm-5.3-flash` | **Z.ai: GLM 5.3 Flash** | z-ai | 1,048,576 tokens | ✅ Sim | ✅ Sim | 👁️ Sim | ✅ Sim |
| `z-ai/glm-5.3-flash:US` | **Z.ai: GLM 5.3 Flash** | z-ai | 1,048,576 tokens | ✅ Sim | ✅ Sim | 👁️ Sim | ✅ Sim |
| `z-ai/glm-5.3-flashx` | **Z.ai: GLM 5.3 FlashX** | z-ai | 1,048,576 tokens | ✅ Sim | ✅ Sim | 👁️ Sim | ✅ Sim |
| `z-ai/glm-5.3-prime` | **Z.ai: GLM 5.3 Prime** | z-ai | 1,000,000 tokens | ✅ Sim | ✅ Sim | ❌ Não | ✅ Sim |
| `z-ai/glm-5v-turbo` | **Z.ai: GLM 5V Turbo** | z-ai | 202,752 tokens | ✅ Sim | ✅ Sim | 👁️ Sim | ✅ Sim |
| `z-ai/glm-flash-latest` | **Z.ai: GLM Flash Latest** | z-ai | 1,048,576 tokens | ✅ Sim | ✅ Sim | 👁️ Sim | ✅ Sim |
| `z-ai/glm-latest` | **Z.ai: GLM Latest** | z-ai | 1,048,576 tokens | ✅ Sim | ✅ Sim | ❌ Não | ✅ Sim |

---

## 💸 4. MATRIZ COMERCIAL DE PREÇOS & TELEMETRIA POR PROVEDOR
Preços de mercado reais levantados via APIs (Input, Output, Cache Read e latências TTFT):

| Modelo Canônico | Provedor / Rota | Input $/1M | Output $/1M | Cache Read $/1M | TTFT Médio | Throughput Médio |
| :--- | :--- | :---: | :---: | :---: | :---: | :---: |
| **AionLabs: Aion-2.0** (`aion-labs/aion-2.0`) | `nous_portal` | $0.80 | $1.60 | $0.2000 | 380 ms | 75.0 tok/s |
| **AionLabs: Aion-2.0** (`aion-labs/aion-2.0`) | `openrouter_market` | $0.80 | $1.60 | $0.2000 | 450 ms | 70.0 tok/s |
| **AionLabs: Aion-3.0** (`aion-labs/aion-3.0`) | `nous_portal` | $3.00 | $6.00 | $0.7500 | 380 ms | 75.0 tok/s |
| **AionLabs: Aion-3.0** (`aion-labs/aion-3.0`) | `openrouter_market` | $3.00 | $6.00 | $0.7500 | 450 ms | 70.0 tok/s |
| **AionLabs: Aion-3.0-Mini** (`aion-labs/aion-3.0-mini`) | `nous_portal` | $0.70 | $1.40 | $0.1800 | 220 ms | 130.0 tok/s |
| **AionLabs: Aion-3.0-Mini** (`aion-labs/aion-3.0-mini`) | `openrouter_market` | $0.70 | $1.40 | $0.1800 | 450 ms | 130.0 tok/s |
| **AionLabs: Aion 3.5** (`aion-labs/aion-3.5`) | `nous_portal` | $3.00 | $6.00 | $0.7500 | 380 ms | 75.0 tok/s |
| **AionLabs: Aion 3.5** (`aion-labs/aion-3.5`) | `openrouter_market` | $3.00 | $6.00 | $0.7500 | 450 ms | 70.0 tok/s |
| **AionLabs: Aion 3.5 Mini** (`aion-labs/aion-3.5-mini`) | `nous_portal` | $0.70 | $1.40 | $0.1800 | 220 ms | 130.0 tok/s |
| **AionLabs: Aion 3.5 Mini** (`aion-labs/aion-3.5-mini`) | `openrouter_market` | $0.70 | $1.40 | $0.1800 | 450 ms | 130.0 tok/s |
| **Amazon: Nova 2 Lite** (`amazon/nova-2-lite-v1`) | `nous_portal` | $0.30 | $2.50 | $0.0600 | 380 ms | 75.0 tok/s |
| **Amazon: Nova 2 Lite** (`amazon/nova-2-lite-v1`) | `openrouter_market` | $0.30 | $2.50 | $0.0750 | 450 ms | 70.0 tok/s |
| **Amazon: Nova Lite 1.0** (`amazon/nova-lite-v1`) | `nous_portal` | $0.06 | $0.24 | $0.0120 | 380 ms | 75.0 tok/s |
| **Amazon: Nova Lite 1.0** (`amazon/nova-lite-v1`) | `openrouter_market` | $0.06 | $0.24 | $0.0150 | 450 ms | 70.0 tok/s |
| **Amazon: Nova Micro 1.0** (`amazon/nova-micro-v1`) | `nous_portal` | $0.04 | $0.14 | $0.0070 | 380 ms | 75.0 tok/s |
| **Amazon: Nova Micro 1.0** (`amazon/nova-micro-v1`) | `openrouter_market` | $0.04 | $0.14 | $0.0088 | 450 ms | 70.0 tok/s |
| **Amazon: Nova Premier 1.0** (`amazon/nova-premier-v1`) | `nous_portal` | $2.50 | $12.50 | $0.6250 | 380 ms | 75.0 tok/s |
| **Amazon: Nova Premier 1.0** (`amazon/nova-premier-v1`) | `openrouter_market` | $2.50 | $12.50 | $0.6250 | 450 ms | 70.0 tok/s |
| **Amazon: Nova Pro 1.0** (`amazon/nova-pro-v1`) | `nous_portal` | $0.80 | $3.20 | $0.1600 | 380 ms | 75.0 tok/s |
| **Amazon: Nova Pro 1.0** (`amazon/nova-pro-v1`) | `openrouter_market` | $0.80 | $3.20 | $0.2000 | 450 ms | 70.0 tok/s |
| **Claude Fable 5** (`anthropic/claude-fable-5`) | `nous_portal_batch` | $5.00 | $25.00 | $0.5000 | 5000 ms | 150.0 tok/s |
| **Claude Fable 5** (`anthropic/claude-fable-5`) | `openrouter_market` | $5.00 | $25.00 | $0.5000 | 450 ms | 70.0 tok/s |
| **Claude Fable 5** (`anthropic/claude-fable-5`) | `nous_portal` | $10.00 | $50.00 | $1.0000 | 380 ms | 75.0 tok/s |
| **Claude Fable 5** (`anthropic/claude-fable-5`) | `openrouter_market` | $10.00 | $50.00 | $1.0000 | 450 ms | 70.0 tok/s |
| **Anthropic: Claude Fable 5.1** (`anthropic/claude-fable-5.1`) | `nous_portal_batch` | $5.00 | $25.00 | $0.1250 | 5000 ms | 150.0 tok/s |
| **Anthropic: Claude Fable 5.1** (`anthropic/claude-fable-5.1`) | `openrouter_market` | $5.00 | $25.00 | $0.1250 | 450 ms | 70.0 tok/s |
| **Anthropic: Claude Fable 5.1** (`anthropic/claude-fable-5.1`) | `nous_portal` | $10.00 | $50.00 | $0.2500 | 380 ms | 75.0 tok/s |
| **Anthropic: Claude Fable 5.1** (`anthropic/claude-fable-5.1`) | `openrouter_market` | $10.00 | $50.00 | $0.2500 | 450 ms | 70.0 tok/s |
| **Anthropic: Claude Fable Latest** (`anthropic/claude-fable-latest`) | `nous_portal` | $10.00 | $50.00 | $0.2500 | 380 ms | 75.0 tok/s |
| **Anthropic: Claude Fable Latest** (`anthropic/claude-fable-latest`) | `openrouter_market` | $10.00 | $50.00 | $0.2500 | 450 ms | 70.0 tok/s |
| **Claude Haiku 4.5** (`anthropic/claude-haiku-4.5`) | `nous_portal_batch` | $0.50 | $2.50 | $0.0500 | 5000 ms | 150.0 tok/s |
| **Claude Haiku 4.5** (`anthropic/claude-haiku-4.5`) | `openrouter_market` | $0.50 | $2.50 | $0.0500 | 450 ms | 130.0 tok/s |
| **Claude Haiku 4.5** (`anthropic/claude-haiku-4.5`) | `nous_portal` | $1.00 | $5.00 | $0.1000 | 220 ms | 130.0 tok/s |
| **Claude Haiku 4.5** (`anthropic/claude-haiku-4.5`) | `openrouter_market` | $1.00 | $5.00 | $0.1000 | 450 ms | 130.0 tok/s |
| **Anthropic: Claude Haiku 5.5** (`anthropic/claude-haiku-5.5`) | `nous_portal_batch` | $0.05 | $0.25 | $0.0050 | 5000 ms | 150.0 tok/s |
| **Anthropic: Claude Haiku 5.5** (`anthropic/claude-haiku-5.5`) | `openrouter_market` | $0.05 | $0.25 | $0.0050 | 450 ms | 130.0 tok/s |
| **Anthropic: Claude Haiku 5.5** (`anthropic/claude-haiku-5.5`) | `nous_portal` | $0.10 | $0.50 | $0.0100 | 220 ms | 130.0 tok/s |
| **Anthropic: Claude Haiku 5.5** (`anthropic/claude-haiku-5.5`) | `openrouter_market` | $0.10 | $0.50 | $0.0100 | 450 ms | 130.0 tok/s |
| **Anthropic: Claude Haiku Latest** (`anthropic/claude-haiku-latest`) | `nous_portal` | $0.10 | $0.50 | $0.0100 | 220 ms | 130.0 tok/s |
| **Anthropic: Claude Haiku Latest** (`anthropic/claude-haiku-latest`) | `openrouter_market` | $0.10 | $0.50 | $0.0100 | 450 ms | 130.0 tok/s |
| **Anthropic: Claude Opus 4.1** (`anthropic/claude-opus-4.1`) | `nous_portal_batch` | $7.50 | $37.50 | $0.7500 | 5000 ms | 150.0 tok/s |
| **Anthropic: Claude Opus 4.1** (`anthropic/claude-opus-4.1`) | `openrouter_market` | $7.50 | $37.50 | $0.7500 | 450 ms | 45.0 tok/s |
| **Anthropic: Claude Opus 4.1** (`anthropic/claude-opus-4.1`) | `nous_portal` | $15.00 | $75.00 | $1.5000 | 550 ms | 40.0 tok/s |
| **Anthropic: Claude Opus 4.1** (`anthropic/claude-opus-4.1`) | `openrouter_market` | $15.00 | $75.00 | $1.5000 | 450 ms | 45.0 tok/s |
| **Anthropic: Claude Opus 4.5** (`anthropic/claude-opus-4.5`) | `nous_portal_batch` | $2.50 | $12.50 | $0.2500 | 5000 ms | 150.0 tok/s |
| **Anthropic: Claude Opus 4.5** (`anthropic/claude-opus-4.5`) | `openrouter_market` | $2.50 | $12.50 | $0.2500 | 450 ms | 45.0 tok/s |
| **Anthropic: Claude Opus 4.5** (`anthropic/claude-opus-4.5`) | `nous_portal` | $5.00 | $25.00 | $0.5000 | 550 ms | 40.0 tok/s |
| **Anthropic: Claude Opus 4.5** (`anthropic/claude-opus-4.5`) | `openrouter_market` | $5.00 | $25.00 | $0.5000 | 450 ms | 45.0 tok/s |
| **Claude Opus 4.6** (`anthropic/claude-opus-4.6`) | `nous_portal_batch` | $2.50 | $12.50 | $0.2500 | 5000 ms | 150.0 tok/s |
| **Claude Opus 4.6** (`anthropic/claude-opus-4.6`) | `openrouter_market` | $2.50 | $12.50 | $0.2500 | 450 ms | 45.0 tok/s |
| **Claude Opus 4.6** (`anthropic/claude-opus-4.6`) | `nous_portal` | $5.00 | $25.00 | $0.5000 | 550 ms | 40.0 tok/s |
| **Claude Opus 4.6** (`anthropic/claude-opus-4.6`) | `openrouter_market` | $5.00 | $25.00 | $0.5000 | 450 ms | 45.0 tok/s |
| **Claude Opus 4.7** (`anthropic/claude-opus-4.7`) | `nous_portal_batch` | $2.50 | $12.50 | $0.2500 | 5000 ms | 150.0 tok/s |
| **Claude Opus 4.7** (`anthropic/claude-opus-4.7`) | `openrouter_market` | $2.50 | $12.50 | $0.2500 | 450 ms | 45.0 tok/s |
| **Claude Opus 4.7** (`anthropic/claude-opus-4.7`) | `nous_portal` | $5.00 | $25.00 | $0.5000 | 550 ms | 40.0 tok/s |
| **Claude Opus 4.7** (`anthropic/claude-opus-4.7`) | `openrouter_market` | $5.00 | $25.00 | $0.5000 | 450 ms | 45.0 tok/s |
| **Claude Opus 4.8** (`anthropic/claude-opus-4.8`) | `nous_portal_batch` | $2.50 | $12.50 | $0.2500 | 5000 ms | 150.0 tok/s |
| **Claude Opus 4.8** (`anthropic/claude-opus-4.8`) | `openrouter_market` | $2.50 | $12.50 | $0.2500 | 450 ms | 45.0 tok/s |
| **Claude Opus 4.8** (`anthropic/claude-opus-4.8`) | `nous_portal` | $5.00 | $25.00 | $0.5000 | 550 ms | 40.0 tok/s |
| **Claude Opus 4.8** (`anthropic/claude-opus-4.8`) | `openrouter_market` | $5.00 | $25.00 | $0.5000 | 450 ms | 45.0 tok/s |
| **Anthropic: Claude Opus 5** (`anthropic/claude-opus-5`) | `nous_portal_batch` | $2.50 | $12.50 | $0.2500 | 5000 ms | 150.0 tok/s |
| **Anthropic: Claude Opus 5** (`anthropic/claude-opus-5`) | `openrouter_market` | $2.50 | $12.50 | $0.2500 | 450 ms | 45.0 tok/s |
| **Anthropic: Claude Opus 5** (`anthropic/claude-opus-5`) | `nous_portal` | $5.00 | $25.00 | $0.5000 | 550 ms | 40.0 tok/s |
| **Anthropic: Claude Opus 5** (`anthropic/claude-opus-5`) | `openrouter_market` | $5.00 | $25.00 | $0.5000 | 450 ms | 45.0 tok/s |
| **Anthropic: Claude Opus 5.5** (`anthropic/claude-opus-5.5`) | `nous_portal_batch` | $2.00 | $10.00 | $0.1000 | 5000 ms | 150.0 tok/s |
| **Anthropic: Claude Opus 5.5** (`anthropic/claude-opus-5.5`) | `openrouter_market` | $2.00 | $10.00 | $0.1000 | 450 ms | 45.0 tok/s |
| **Anthropic: Claude Opus 5.5** (`anthropic/claude-opus-5.5`) | `nous_portal` | $4.00 | $20.00 | $0.2000 | 550 ms | 40.0 tok/s |
| **Anthropic: Claude Opus 5.5** (`anthropic/claude-opus-5.5`) | `openrouter_market` | $4.00 | $20.00 | $0.2000 | 450 ms | 45.0 tok/s |
| **Anthropic: Claude Opus Latest** (`anthropic/claude-opus-latest`) | `nous_portal` | $4.00 | $20.00 | $0.2000 | 550 ms | 40.0 tok/s |
| **Anthropic: Claude Opus Latest** (`anthropic/claude-opus-latest`) | `openrouter_market` | $4.00 | $20.00 | $0.2000 | 450 ms | 45.0 tok/s |
| **Anthropic: Claude Sonnet 4** (`anthropic/claude-sonnet-4`) | `nous_portal` | $3.00 | $15.00 | $0.3000 | 380 ms | 75.0 tok/s |
| **Anthropic: Claude Sonnet 4** (`anthropic/claude-sonnet-4`) | `openrouter_market` | $3.00 | $15.00 | $0.3000 | 450 ms | 70.0 tok/s |
| **Anthropic: Claude Sonnet 4.5** (`anthropic/claude-sonnet-4.5`) | `nous_portal_batch` | $1.50 | $7.50 | $0.1500 | 5000 ms | 150.0 tok/s |
| **Anthropic: Claude Sonnet 4.5** (`anthropic/claude-sonnet-4.5`) | `openrouter_market` | $1.50 | $7.50 | $0.1500 | 450 ms | 70.0 tok/s |
| **Anthropic: Claude Sonnet 4.5** (`anthropic/claude-sonnet-4.5`) | `nous_portal` | $3.00 | $15.00 | $0.3000 | 380 ms | 75.0 tok/s |
| **Anthropic: Claude Sonnet 4.5** (`anthropic/claude-sonnet-4.5`) | `openrouter_market` | $3.00 | $15.00 | $0.3000 | 450 ms | 70.0 tok/s |
| **Claude Sonnet 4.6** (`anthropic/claude-sonnet-4.6`) | `nous_portal_batch` | $1.50 | $7.50 | $0.1500 | 5000 ms | 150.0 tok/s |
| **Claude Sonnet 4.6** (`anthropic/claude-sonnet-4.6`) | `openrouter_market` | $1.50 | $7.50 | $0.1500 | 450 ms | 70.0 tok/s |
| **Claude Sonnet 4.6** (`anthropic/claude-sonnet-4.6`) | `nous_portal` | $3.00 | $15.00 | $0.3000 | 380 ms | 75.0 tok/s |
| **Claude Sonnet 4.6** (`anthropic/claude-sonnet-4.6`) | `openrouter_market` | $3.00 | $15.00 | $0.3000 | 450 ms | 70.0 tok/s |
| **Claude Sonnet 5** (`anthropic/claude-sonnet-5`) | `nous_portal_batch` | $1.00 | $5.00 | $0.1000 | 5000 ms | 150.0 tok/s |
| **Claude Sonnet 5** (`anthropic/claude-sonnet-5`) | `openrouter_market` | $1.00 | $5.00 | $0.1000 | 450 ms | 70.0 tok/s |
| **Claude Sonnet 5** (`anthropic/claude-sonnet-5`) | `nous_portal` | $2.00 | $10.00 | $0.2000 | 380 ms | 75.0 tok/s |
| **Claude Sonnet 5** (`anthropic/claude-sonnet-5`) | `openrouter_market` | $2.00 | $10.00 | $0.2000 | 450 ms | 70.0 tok/s |
| **Anthropic: Claude Sonnet 5.5** (`anthropic/claude-sonnet-5.5`) | `nous_portal_batch` | $1.00 | $5.00 | $0.0500 | 5000 ms | 150.0 tok/s |
| **Anthropic: Claude Sonnet 5.5** (`anthropic/claude-sonnet-5.5`) | `openrouter_market` | $1.00 | $5.00 | $0.0500 | 450 ms | 70.0 tok/s |
| **Anthropic: Claude Sonnet 5.5** (`anthropic/claude-sonnet-5.5`) | `nous_portal` | $2.00 | $10.00 | $0.1000 | 380 ms | 75.0 tok/s |
| **Anthropic: Claude Sonnet 5.5** (`anthropic/claude-sonnet-5.5`) | `openrouter_market` | $2.00 | $10.00 | $0.1000 | 450 ms | 70.0 tok/s |
| **Anthropic: Claude Sonnet Latest** (`anthropic/claude-sonnet-latest`) | `nous_portal` | $2.00 | $10.00 | $0.1000 | 380 ms | 75.0 tok/s |
| **Anthropic: Claude Sonnet Latest** (`anthropic/claude-sonnet-latest`) | `openrouter_market` | $2.00 | $10.00 | $0.1000 | 450 ms | 70.0 tok/s |
| **Arcee AI: Trinity Large Thinking** (`arcee-ai/trinity-large-thinking`) | `nous_portal` | $0.25 | $0.80 | $0.0600 | 380 ms | 75.0 tok/s |
| **Arcee AI: Trinity Large Thinking** (`arcee-ai/trinity-large-thinking`) | `openrouter_market` | $0.25 | $0.80 | $0.0600 | 450 ms | 70.0 tok/s |
| **BAAI: bge-base-en-v1.5** (`baai/bge-base-en-v1.5`) | `nous_portal` | $0.01 | $0.00 | $0.0010 | 380 ms | 75.0 tok/s |
| **BAAI: bge-large-en-v1.5** (`baai/bge-large-en-v1.5`) | `nous_portal` | $0.01 | $0.00 | $0.0020 | 380 ms | 75.0 tok/s |
| **BAAI: bge-m3** (`baai/bge-m3`) | `nous_portal` | $0.01 | $0.00 | $0.0020 | 380 ms | 75.0 tok/s |
| **ByteDance Seed: Seed 1.6** (`bytedance-seed/seed-1.6`) | `nous_portal` | $0.25 | $2.00 | $0.0500 | 380 ms | 75.0 tok/s |
| **ByteDance Seed: Seed 1.6** (`bytedance-seed/seed-1.6`) | `openrouter_market` | $0.25 | $2.00 | $0.0625 | 450 ms | 70.0 tok/s |
| **ByteDance Seed: Seed 1.6 Flash** (`bytedance-seed/seed-1.6-flash`) | `nous_portal` | $0.07 | $0.30 | $0.0150 | 220 ms | 130.0 tok/s |
| **ByteDance Seed: Seed 1.6 Flash** (`bytedance-seed/seed-1.6-flash`) | `openrouter_market` | $0.07 | $0.30 | $0.0187 | 450 ms | 130.0 tok/s |
| **ByteDance Seed: Seed 2.1 Turbo** (`bytedance-seed/seed-2-1-turbo`) | `nous_portal` | $0.50 | $2.50 | $0.1000 | 380 ms | 75.0 tok/s |
| **ByteDance Seed: Seed 2.1 Turbo** (`bytedance-seed/seed-2-1-turbo`) | `openrouter_market` | $0.50 | $2.50 | $0.1250 | 450 ms | 70.0 tok/s |
| **ByteDance Seed: Seed-2.0-Code** (`bytedance-seed/seed-2.0-code`) | `nous_portal` | $0.50 | $3.00 | $0.1000 | 380 ms | 75.0 tok/s |
| **ByteDance Seed: Seed-2.0-Code** (`bytedance-seed/seed-2.0-code`) | `openrouter_market` | $0.50 | $3.00 | $0.1250 | 450 ms | 70.0 tok/s |
| **ByteDance Seed: Seed-2.0-Lite** (`bytedance-seed/seed-2.0-lite`) | `nous_portal` | $0.25 | $2.00 | $0.0500 | 380 ms | 75.0 tok/s |
| **ByteDance Seed: Seed-2.0-Lite** (`bytedance-seed/seed-2.0-lite`) | `openrouter_market` | $0.25 | $2.00 | $0.0625 | 450 ms | 70.0 tok/s |
| **ByteDance Seed: Seed-2.0-Mini** (`bytedance-seed/seed-2.0-mini`) | `nous_portal` | $0.10 | $0.40 | $0.0200 | 220 ms | 130.0 tok/s |
| **ByteDance Seed: Seed-2.0-Mini** (`bytedance-seed/seed-2.0-mini`) | `openrouter_market` | $0.10 | $0.40 | $0.0250 | 450 ms | 130.0 tok/s |
| **Claude 3.5 Sonnet** (`claude-3-5-sonnet`) | `nous_portal` | $3.00 | $12.00 | $0.6000 | 380 ms | 75.0 tok/s |
| **Claude 3.5 Sonnet** (`claude-3-5-sonnet`) | `nous_portal` | $3.00 | $12.00 | $0.6000 | 380 ms | 75.0 tok/s |
| **Claude 3.7 Sonnet** (`claude-3-7-sonnet`) | `nous_portal` | $3.00 | $12.00 | $0.6000 | 380 ms | 75.0 tok/s |
| **Claude 3.7 Sonnet** (`claude-3-7-sonnet`) | `nous_portal` | $3.00 | $12.00 | $0.6000 | 380 ms | 75.0 tok/s |
| **Claude 3.7 Sonnet** (`claude-3-7-sonnet`) | `nous_portal` | $3.40 | $15.00 | $0.3500 | 420 ms | 72.0 tok/s |
| **Cohere: Command A+** (`cohere/command-a-plus`) | `nous_portal` | $0.30 | $1.50 | $0.1500 | 380 ms | 75.0 tok/s |
| **Cohere: Command A+** (`cohere/command-a-plus`) | `openrouter_market` | $0.30 | $1.50 | $0.1500 | 450 ms | 70.0 tok/s |
| **Cohere: Command R (08-2024)** (`cohere/command-r-08-2024`) | `nous_portal` | $0.15 | $0.60 | $0.0300 | 380 ms | 75.0 tok/s |
| **Cohere: Command R (08-2024)** (`cohere/command-r-08-2024`) | `openrouter_market` | $0.15 | $0.60 | $0.0375 | 450 ms | 70.0 tok/s |
| **Cohere: Command R+ (08-2024)** (`cohere/command-r-plus-08-2024`) | `nous_portal` | $2.50 | $10.00 | $0.5000 | 380 ms | 75.0 tok/s |
| **Cohere: Command R+ (08-2024)** (`cohere/command-r-plus-08-2024`) | `openrouter_market` | $2.50 | $10.00 | $0.6250 | 450 ms | 70.0 tok/s |
| **DeepSeek R1** (`deepseek-r1`) | `nous_portal` | $0.50 | $2.00 | $0.1000 | 550 ms | 75.0 tok/s |
| **DeepSeek R1** (`deepseek-r1`) | `nous_portal` | $0.50 | $2.00 | $0.1000 | 550 ms | 75.0 tok/s |
| **DeepSeek R1** (`deepseek-r1`) | `nous_portal` | $0.55 | $2.19 | $0.1400 | 580 ms | 52.0 tok/s |
| **DeepSeek R1** (`deepseek-r1`) | `openrouter` | $0.70 | $2.50 | $0.1750 | 450 ms | 45.0 tok/s |
| **DeepSeek R1** (`deepseek-r1`) | `openrouter_market` | $0.70 | $2.50 | $0.1750 | 450 ms | 45.0 tok/s |
| **DeepSeek R1** (`deepseek-r1`) | `openrouter_market` | $0.70 | $2.50 | $0.1750 | 450 ms | 45.0 tok/s |
| **DeepSeek V3** (`deepseek-v3`) | `nous_portal` | $0.28 | $1.10 | $0.0700 | 320 ms | 92.0 tok/s |
| **DeepSeek V3** (`deepseek-v3`) | `openrouter` | $0.32 | $0.89 | $0.0800 | 450 ms | 85.0 tok/s |
| **DeepSeek V3** (`deepseek-v3`) | `openrouter_market` | $0.32 | $0.89 | $0.0800 | 450 ms | 85.0 tok/s |
| **DeepSeek V3** (`deepseek-v3`) | `openrouter_market` | $0.32 | $0.89 | $0.0800 | 450 ms | 85.0 tok/s |
| **DeepSeek V3** (`deepseek-v3`) | `nous_portal` | $0.50 | $2.00 | $0.1000 | 380 ms | 75.0 tok/s |
| **DeepSeek V3** (`deepseek-v3`) | `nous_portal` | $0.50 | $2.00 | $0.1000 | 380 ms | 75.0 tok/s |
| **DeepSeek: DeepSeek V3** (`deepseek/deepseek-chat`) | `nous_portal` | $0.26 | $1.03 | $0.0515 | 380 ms | 75.0 tok/s |
| **DeepSeek: DeepSeek V3** (`deepseek/deepseek-chat`) | `openrouter_market` | $0.32 | $0.89 | $0.0800 | 450 ms | 70.0 tok/s |
| **DeepSeek: DeepSeek V3 0324** (`deepseek/deepseek-chat-v3-0324`) | `nous_portal` | $0.25 | $1.00 | $0.0500 | 380 ms | 75.0 tok/s |
| **DeepSeek: DeepSeek V3 0324** (`deepseek/deepseek-chat-v3-0324`) | `openrouter_market` | $0.29 | $1.14 | $0.1100 | 450 ms | 70.0 tok/s |
| **DeepSeek: DeepSeek V3.1** (`deepseek/deepseek-chat-v3.1`) | `nous_portal` | $0.25 | $0.95 | $0.1300 | 380 ms | 75.0 tok/s |
| **DeepSeek: DeepSeek V3.1** (`deepseek/deepseek-chat-v3.1`) | `openrouter_market` | $0.25 | $0.95 | $0.1300 | 450 ms | 70.0 tok/s |
| **DeepSeek: DeepSeek Flash Latest** (`deepseek/deepseek-flash-latest`) | `openrouter_market` | $0.01 | $0.89 | $0.0054 | 450 ms | 130.0 tok/s |
| **DeepSeek: DeepSeek Flash Latest** (`deepseek/deepseek-flash-latest`) | `nous_portal` | $0.01 | $0.60 | $0.0080 | 220 ms | 130.0 tok/s |
| **DeepSeek: DeepSeek Pro Latest** (`deepseek/deepseek-pro-latest`) | `nous_portal` | $0.18 | $8.00 | $0.1800 | 380 ms | 75.0 tok/s |
| **DeepSeek: DeepSeek Pro Latest** (`deepseek/deepseek-pro-latest`) | `openrouter_market` | $0.18 | $8.00 | $0.1800 | 450 ms | 70.0 tok/s |
| **DeepSeek: R1** (`deepseek/deepseek-r1`) | `nous_portal` | $0.70 | $2.50 | $0.1400 | 550 ms | 75.0 tok/s |
| **DeepSeek: R1** (`deepseek/deepseek-r1`) | `openrouter_market` | $0.70 | $2.50 | $0.1750 | 450 ms | 45.0 tok/s |
| **DeepSeek: R1 0528** (`deepseek/deepseek-r1-0528`) | `nous_portal` | $0.50 | $2.15 | $0.3500 | 550 ms | 75.0 tok/s |
| **DeepSeek: R1 0528** (`deepseek/deepseek-r1-0528`) | `openrouter_market` | $0.50 | $2.15 | $0.3500 | 450 ms | 45.0 tok/s |
| **DeepSeek: DeepSeek V3.1 Terminus** (`deepseek/deepseek-v3.1-terminus`) | `nous_portal` | $0.27 | $1.00 | $0.0540 | 380 ms | 75.0 tok/s |
| **DeepSeek: DeepSeek V3.1 Terminus** (`deepseek/deepseek-v3.1-terminus`) | `openrouter_market` | $0.27 | $1.00 | $0.0675 | 450 ms | 85.0 tok/s |
| **DeepSeek: DeepSeek V3.2** (`deepseek/deepseek-v3.2`) | `nous_portal` | $0.21 | $0.31 | $0.0216 | 380 ms | 75.0 tok/s |
| **DeepSeek: DeepSeek V3.2** (`deepseek/deepseek-v3.2`) | `openrouter_market` | $0.26 | $0.80 | $0.1400 | 450 ms | 85.0 tok/s |
| **DeepSeek: DeepSeek V3.2 Exp** (`deepseek/deepseek-v3.2-exp`) | `nous_portal` | $0.27 | $0.41 | $0.0540 | 380 ms | 75.0 tok/s |
| **DeepSeek: DeepSeek V3.2 Exp** (`deepseek/deepseek-v3.2-exp`) | `openrouter_market` | $0.27 | $0.41 | $0.0675 | 450 ms | 85.0 tok/s |
| **DeepSeek: DeepSeek V4 Flash 0423** (`deepseek/deepseek-v4-flash`) | `nous_portal` | $0.04 | $1.15 | $0.0117 | 220 ms | 130.0 tok/s |
| **DeepSeek: DeepSeek V4 Flash 0423** (`deepseek/deepseek-v4-flash`) | `openrouter_market` | $0.09 | $0.18 | $0.0182 | 450 ms | 130.0 tok/s |
| **DeepSeek: DeepSeek V4 Flash** (`deepseek/deepseek-v4-flash-0731`) | `openrouter_market` | $0.02 | $0.64 | $0.0160 | 450 ms | 130.0 tok/s |
| **DeepSeek: DeepSeek V4 Flash** (`deepseek/deepseek-v4-flash-0731`) | `nous_portal` | $0.07 | $0.14 | $0.0140 | 220 ms | 130.0 tok/s |
| **DeepSeek: DeepSeek V4 Flash** (`deepseek/deepseek-v4-flash-0731:US`) | `nous_portal` | $0.36 | $1.09 | $0.0115 | 220 ms | 130.0 tok/s |
| **DeepSeek: DeepSeek V4 Flash Latest** (`deepseek/deepseek-v4-flash-latest`) | `nous_portal` | $0.01 | $0.58 | $0.0108 | 220 ms | 130.0 tok/s |
| **DeepSeek: DeepSeek V4 Flash Latest** (`deepseek/deepseek-v4-flash-latest`) | `openrouter_market` | $0.01 | $0.58 | $0.0108 | 450 ms | 130.0 tok/s |
| **DeepSeek: DeepSeek V4 Flash Vision Exp** (`deepseek/deepseek-v4-flash-vision-exp`) | `nous_portal` | $0.22 | $0.65 | $0.0069 | 220 ms | 130.0 tok/s |
| **DeepSeek: DeepSeek V4 Flash Vision Exp** (`deepseek/deepseek-v4-flash-vision-exp`) | `openrouter_market` | $0.22 | $0.65 | $0.0069 | 450 ms | 130.0 tok/s |
| **DeepSeek: DeepSeek V4 Pro 0423** (`deepseek/deepseek-v4-pro`) | `nous_portal` | $0.94 | $1.89 | $0.0800 | 380 ms | 75.0 tok/s |
| **DeepSeek: DeepSeek V4 Pro 0423** (`deepseek/deepseek-v4-pro`) | `openrouter_market` | $0.96 | $1.91 | $0.0796 | 450 ms | 70.0 tok/s |
| **DeepSeek: DeepSeek V4 Pro 0813** (`deepseek/deepseek-v4-pro-0813`) | `nous_portal` | $0.18 | $8.00 | $0.1800 | 380 ms | 75.0 tok/s |
| **DeepSeek: DeepSeek V4 Pro 0813** (`deepseek/deepseek-v4-pro-0813`) | `openrouter_market` | $0.66 | $1.98 | $0.0220 | 450 ms | 70.0 tok/s |
| **DeepSeek: DeepSeek V4.1 Flash** (`deepseek/deepseek-v4.1-flash`) | `openrouter_market` | $0.01 | $0.89 | $0.0054 | 450 ms | 130.0 tok/s |
| **DeepSeek: DeepSeek V4.1 Flash** (`deepseek/deepseek-v4.1-flash`) | `nous_portal` | $0.11 | $0.51 | $0.0040 | 220 ms | 130.0 tok/s |
| **DeepSeek: DeepSeek V4.1 Flash** (`deepseek/deepseek-v4.1-flash`) | `nous_portal_batch` | $0.11 | $0.34 | $0.0034 | 5000 ms | 150.0 tok/s |
| **DeepSeek: DeepSeek V4.1 Flash** (`deepseek/deepseek-v4.1-flash`) | `openrouter_market` | $0.11 | $0.34 | $0.0034 | 450 ms | 130.0 tok/s |
| **DeepSeek: DeepSeek V4.1 Flash** (`deepseek/deepseek-v4.1-flash:US`) | `nous_portal` | $0.36 | $1.09 | $0.0115 | 220 ms | 130.0 tok/s |
| **Fireworks: Ember-1** (`fireworks/ember-1`) | `nous_portal` | $3.00 | $15.00 | $0.3000 | 380 ms | 75.0 tok/s |
| **Fireworks: Ember-1** (`fireworks/ember-1`) | `openrouter_market` | $3.00 | $15.00 | $0.3000 | 450 ms | 70.0 tok/s |
| **Gemini 2.0 Flash** (`gemini-2.0-flash`) | `nous_portal` | $0.50 | $2.00 | $0.1000 | 220 ms | 130.0 tok/s |
| **Gemini 2.0 Flash** (`gemini-2.0-flash`) | `nous_portal` | $0.50 | $2.00 | $0.1000 | 220 ms | 130.0 tok/s |
| **Google: Gemini 2.5 Flash** (`google/gemini-2.5-flash`) | `nous_portal_batch` | $0.15 | $1.25 | $0.0300 | 5000 ms | 150.0 tok/s |
| **Google: Gemini 2.5 Flash** (`google/gemini-2.5-flash`) | `openrouter_market` | $0.15 | $1.25 | $0.0300 | 450 ms | 130.0 tok/s |
| **Google: Gemini 2.5 Flash** (`google/gemini-2.5-flash`) | `nous_portal` | $0.30 | $2.50 | $0.0300 | 220 ms | 130.0 tok/s |
| **Google: Gemini 2.5 Flash** (`google/gemini-2.5-flash`) | `openrouter_market` | $0.30 | $2.50 | $0.0300 | 450 ms | 130.0 tok/s |
| **Google: Gemini 2.5 Flash Lite** (`google/gemini-2.5-flash-lite`) | `nous_portal_batch` | $0.05 | $0.20 | $0.0100 | 5000 ms | 150.0 tok/s |
| **Google: Gemini 2.5 Flash Lite** (`google/gemini-2.5-flash-lite`) | `openrouter_market` | $0.05 | $0.20 | $0.0100 | 450 ms | 130.0 tok/s |
| **Google: Gemini 2.5 Flash Lite** (`google/gemini-2.5-flash-lite`) | `nous_portal` | $0.10 | $0.40 | $0.0100 | 220 ms | 130.0 tok/s |
| **Google: Gemini 2.5 Flash Lite** (`google/gemini-2.5-flash-lite`) | `openrouter_market` | $0.10 | $0.40 | $0.0100 | 450 ms | 130.0 tok/s |
| **Google: Gemini 2.5 Pro** (`google/gemini-2.5-pro`) | `nous_portal_batch` | $0.62 | $5.00 | $0.1250 | 5000 ms | 150.0 tok/s |
| **Google: Gemini 2.5 Pro** (`google/gemini-2.5-pro`) | `openrouter_market` | $0.62 | $5.00 | $0.1250 | 450 ms | 130.0 tok/s |
| **Google: Gemini 2.5 Pro** (`google/gemini-2.5-pro`) | `nous_portal` | $1.25 | $10.00 | $0.1250 | 220 ms | 130.0 tok/s |
| **Google: Gemini 2.5 Pro** (`google/gemini-2.5-pro`) | `openrouter_market` | $1.25 | $10.00 | $0.1250 | 450 ms | 130.0 tok/s |
| **Google: Gemini 2.5 Pro Preview 06-05** (`google/gemini-2.5-pro-preview`) | `nous_portal` | $1.25 | $10.00 | $0.1250 | 220 ms | 130.0 tok/s |
| **Google: Gemini 2.5 Pro Preview 06-05** (`google/gemini-2.5-pro-preview`) | `openrouter_market` | $1.25 | $10.00 | $0.1250 | 450 ms | 130.0 tok/s |
| **Google: Gemini 3 Flash Preview** (`google/gemini-3-flash-preview`) | `nous_portal_batch` | $0.25 | $1.50 | $0.0500 | 5000 ms | 150.0 tok/s |
| **Google: Gemini 3 Flash Preview** (`google/gemini-3-flash-preview`) | `openrouter_market` | $0.25 | $1.50 | $0.0625 | 450 ms | 130.0 tok/s |
| **Google: Gemini 3 Flash Preview** (`google/gemini-3-flash-preview`) | `nous_portal` | $0.50 | $3.00 | $0.0500 | 220 ms | 130.0 tok/s |
| **Google: Gemini 3 Flash Preview** (`google/gemini-3-flash-preview`) | `openrouter_market` | $0.50 | $3.00 | $0.0500 | 450 ms | 130.0 tok/s |
| **Google: Nano Banana Pro (Gemini 3 Pro Image)** (`google/gemini-3-pro-image`) | `nous_portal` | $2.00 | $12.00 | $0.2000 | 220 ms | 130.0 tok/s |
| **Google: Nano Banana Pro (Gemini 3 Pro Image)** (`google/gemini-3-pro-image`) | `openrouter_market` | $2.00 | $12.00 | $0.2000 | 450 ms | 130.0 tok/s |
| **Google: Gemini 3.1 Flash Lite** (`google/gemini-3.1-flash-lite`) | `nous_portal_batch` | $0.12 | $0.75 | $0.0125 | 5000 ms | 150.0 tok/s |
| **Google: Gemini 3.1 Flash Lite** (`google/gemini-3.1-flash-lite`) | `openrouter_market` | $0.12 | $0.75 | $0.0125 | 450 ms | 130.0 tok/s |
| **Google: Gemini 3.1 Flash Lite** (`google/gemini-3.1-flash-lite`) | `nous_portal` | $0.25 | $1.50 | $0.0250 | 220 ms | 130.0 tok/s |
| **Google: Gemini 3.1 Flash Lite** (`google/gemini-3.1-flash-lite`) | `openrouter_market` | $0.25 | $1.50 | $0.0250 | 450 ms | 130.0 tok/s |
| **Google: Gemini 3.1 Flash Lite Preview** (`google/gemini-3.1-flash-lite-preview`) | `nous_portal` | $0.25 | $1.50 | $0.0250 | 220 ms | 130.0 tok/s |
| **Google: Gemini 3.1 Flash Lite Preview** (`google/gemini-3.1-flash-lite-preview`) | `openrouter_market` | $0.25 | $1.50 | $0.0250 | 450 ms | 130.0 tok/s |
| **Google: Gemini 3.1 Pro Preview** (`google/gemini-3.1-pro-preview`) | `nous_portal_batch` | $1.00 | $6.00 | $0.2000 | 5000 ms | 150.0 tok/s |
| **Google: Gemini 3.1 Pro Preview** (`google/gemini-3.1-pro-preview`) | `openrouter_market` | $1.00 | $6.00 | $0.2500 | 450 ms | 130.0 tok/s |
| **Google: Gemini 3.1 Pro Preview** (`google/gemini-3.1-pro-preview`) | `nous_portal` | $2.00 | $12.00 | $0.2000 | 220 ms | 130.0 tok/s |
| **Google: Gemini 3.1 Pro Preview** (`google/gemini-3.1-pro-preview`) | `openrouter_market` | $2.00 | $12.00 | $0.2000 | 450 ms | 130.0 tok/s |
| **Google: Gemini 3.1 Pro Preview Custom Tools** (`google/gemini-3.1-pro-preview-customtools`) | `nous_portal` | $2.00 | $12.00 | $0.2000 | 220 ms | 130.0 tok/s |
| **Google: Gemini 3.1 Pro Preview Custom Tools** (`google/gemini-3.1-pro-preview-customtools`) | `openrouter_market` | $2.00 | $12.00 | $0.2000 | 450 ms | 130.0 tok/s |
| **Google: Gemini 3.5 Flash** (`google/gemini-3.5-flash`) | `nous_portal_batch` | $0.75 | $4.50 | $0.0750 | 5000 ms | 150.0 tok/s |
| **Google: Gemini 3.5 Flash** (`google/gemini-3.5-flash`) | `openrouter_market` | $0.75 | $4.50 | $0.0750 | 450 ms | 130.0 tok/s |
| **Google: Gemini 3.5 Flash** (`google/gemini-3.5-flash`) | `nous_portal` | $1.50 | $9.00 | $0.1500 | 220 ms | 130.0 tok/s |
| **Google: Gemini 3.5 Flash** (`google/gemini-3.5-flash`) | `openrouter_market` | $1.50 | $9.00 | $0.1500 | 450 ms | 130.0 tok/s |
| **Google: Gemini 3.5 Flash Lite** (`google/gemini-3.5-flash-lite`) | `nous_portal_batch` | $0.15 | $1.25 | $0.0150 | 5000 ms | 150.0 tok/s |
| **Google: Gemini 3.5 Flash Lite** (`google/gemini-3.5-flash-lite`) | `openrouter_market` | $0.15 | $1.25 | $0.0150 | 450 ms | 130.0 tok/s |
| **Google: Gemini 3.5 Flash Lite** (`google/gemini-3.5-flash-lite`) | `nous_portal` | $0.30 | $2.50 | $0.0300 | 220 ms | 130.0 tok/s |
| **Google: Gemini 3.5 Flash Lite** (`google/gemini-3.5-flash-lite`) | `openrouter_market` | $0.30 | $2.50 | $0.0300 | 450 ms | 130.0 tok/s |
| **Google: Gemini 3.6 Flash** (`google/gemini-3.6-flash`) | `nous_portal_batch` | $0.38 | $1.88 | $0.0375 | 5000 ms | 150.0 tok/s |
| **Google: Gemini 3.6 Flash** (`google/gemini-3.6-flash`) | `openrouter_market` | $0.38 | $1.88 | $0.0375 | 450 ms | 130.0 tok/s |
| **Google: Gemini 3.6 Flash** (`google/gemini-3.6-flash`) | `nous_portal` | $0.75 | $3.75 | $0.0750 | 220 ms | 130.0 tok/s |
| **Google: Gemini 3.6 Flash** (`google/gemini-3.6-flash`) | `openrouter_market` | $0.75 | $3.75 | $0.0750 | 450 ms | 130.0 tok/s |
| **Google: Gemini 3.7 Flash** (`google/gemini-3.7-flash`) | `nous_portal_batch` | $0.38 | $1.88 | $0.0375 | 5000 ms | 150.0 tok/s |
| **Google: Gemini 3.7 Flash** (`google/gemini-3.7-flash`) | `openrouter_market` | $0.38 | $1.88 | $0.0375 | 450 ms | 130.0 tok/s |
| **Google: Gemini 3.7 Flash** (`google/gemini-3.7-flash`) | `nous_portal` | $0.75 | $3.75 | $0.0750 | 220 ms | 130.0 tok/s |
| **Google: Gemini 3.7 Flash** (`google/gemini-3.7-flash`) | `openrouter_market` | $0.75 | $3.75 | $0.0750 | 450 ms | 130.0 tok/s |
| **Google: Gemini 3.8 Flash** (`google/gemini-3.8-flash`) | `nous_portal_batch` | $0.38 | $1.88 | $0.0375 | 5000 ms | 150.0 tok/s |
| **Google: Gemini 3.8 Flash** (`google/gemini-3.8-flash`) | `openrouter_market` | $0.38 | $1.88 | $0.0375 | 450 ms | 130.0 tok/s |
| **Google: Gemini 3.8 Flash** (`google/gemini-3.8-flash`) | `nous_portal` | $0.75 | $3.75 | $0.0750 | 220 ms | 130.0 tok/s |
| **Google: Gemini 3.8 Flash** (`google/gemini-3.8-flash`) | `openrouter_market` | $0.75 | $3.75 | $0.0750 | 450 ms | 130.0 tok/s |
| **Google: Gemini 4 Argon** (`google/gemini-4-argon`) | `nous_portal_batch` | $1.00 | $5.00 | $0.0500 | 5000 ms | 150.0 tok/s |
| **Google: Gemini 4 Argon** (`google/gemini-4-argon`) | `openrouter_market` | $1.00 | $5.00 | $0.0500 | 450 ms | 130.0 tok/s |
| **Google: Gemini 4 Argon** (`google/gemini-4-argon`) | `nous_portal` | $2.00 | $10.00 | $0.1000 | 220 ms | 130.0 tok/s |
| **Google: Gemini 4 Argon** (`google/gemini-4-argon`) | `openrouter_market` | $2.00 | $10.00 | $0.1000 | 450 ms | 130.0 tok/s |
| **Google: Gemini Embedding 001** (`google/gemini-embedding-001`) | `nous_portal` | $0.15 | $0.00 | $0.0300 | 220 ms | 130.0 tok/s |
| **Google: Gemini Embedding 2** (`google/gemini-embedding-2`) | `nous_portal_batch` | $0.10 | $0.00 | $0.0200 | 5000 ms | 150.0 tok/s |
| **Google: Gemini Embedding 2** (`google/gemini-embedding-2`) | `nous_portal` | $0.20 | $0.00 | $0.0400 | 220 ms | 130.0 tok/s |
| **Google: Gemini Embedding 2 Preview** (`google/gemini-embedding-2-preview`) | `nous_portal` | $0.20 | $0.00 | $0.0400 | 220 ms | 130.0 tok/s |
| **Google: Gemini Flash Latest** (`google/gemini-flash-latest`) | `nous_portal` | $0.75 | $3.75 | $0.0750 | 220 ms | 130.0 tok/s |
| **Google: Gemini Flash Latest** (`google/gemini-flash-latest`) | `openrouter_market` | $0.75 | $3.75 | $0.0750 | 450 ms | 130.0 tok/s |
| **Google: Nano Banana 2.1** (`google/gemini-nano-banana-2.1`) | `nous_portal` | $1.50 | $7.50 | $0.3000 | 220 ms | 130.0 tok/s |
| **Google: Nano Banana 2.1** (`google/gemini-nano-banana-2.1`) | `openrouter_market` | $1.50 | $7.50 | $0.3750 | 450 ms | 130.0 tok/s |
| **Google: Gemini Pro Latest** (`google/gemini-pro-latest`) | `nous_portal` | $2.00 | $12.00 | $0.2000 | 220 ms | 130.0 tok/s |
| **Google: Gemini Pro Latest** (`google/gemini-pro-latest`) | `openrouter_market` | $2.00 | $12.00 | $0.2000 | 450 ms | 130.0 tok/s |
| **Google: Gemma 3 12B** (`google/gemma-3-12b-it`) | `nous_portal` | $0.05 | $0.15 | $0.0100 | 380 ms | 75.0 tok/s |
| **Google: Gemma 3 12B** (`google/gemma-3-12b-it`) | `openrouter_market` | $0.05 | $0.15 | $0.0125 | 450 ms | 70.0 tok/s |
| **Google: Gemma 3 27B** (`google/gemma-3-27b-it`) | `nous_portal` | $0.07 | $0.28 | $0.0400 | 380 ms | 75.0 tok/s |
| **Google: Gemma 3 27B** (`google/gemma-3-27b-it`) | `openrouter_market` | $0.08 | $0.45 | $0.0400 | 450 ms | 70.0 tok/s |
| **Google: Gemma 4 26B A4B ** (`google/gemma-4-26b-a4b-it`) | `openrouter_market` | $0.00 | $0.00 | N/A | 450 ms | 70.0 tok/s |
| **Google: Gemma 4 26B A4B ** (`google/gemma-4-26b-a4b-it`) | `nous_portal` | $0.04 | $0.22 | $0.0210 | 380 ms | 75.0 tok/s |
| **Google: Gemma 4 26B A4B ** (`google/gemma-4-26b-a4b-it`) | `openrouter_market` | $0.08 | $0.26 | $0.0425 | 450 ms | 70.0 tok/s |
| **Google: Gemma 4 31B** (`google/gemma-4-31b-it`) | `openrouter_market` | $0.00 | $0.00 | N/A | 450 ms | 70.0 tok/s |
| **Google: Gemma 4 31B** (`google/gemma-4-31b-it`) | `nous_portal` | $0.09 | $0.34 | $0.0500 | 380 ms | 75.0 tok/s |
| **Google: Gemma 4 31B** (`google/gemma-4-31b-it`) | `openrouter_market` | $0.09 | $0.34 | $0.0500 | 450 ms | 70.0 tok/s |
| **GPT-4o** (`gpt-4o`) | `nous_portal` | $0.50 | $2.00 | $0.1000 | 380 ms | 75.0 tok/s |
| **GPT-4o** (`gpt-4o`) | `nous_portal` | $0.50 | $2.00 | $0.1000 | 380 ms | 75.0 tok/s |
| **GPT-4o** (`gpt-4o`) | `openrouter` | $1.25 | $5.00 | $0.3125 | 450 ms | 70.0 tok/s |
| **GPT-4o** (`gpt-4o`) | `openrouter_market` | $1.25 | $5.00 | $0.6250 | 450 ms | 70.0 tok/s |
| **GPT-4o** (`gpt-4o`) | `openrouter_market` | $1.25 | $5.00 | $0.6250 | 450 ms | 70.0 tok/s |
| **GPT-4o** (`gpt-4o`) | `openrouter` | $2.50 | $10.00 | $0.6250 | 450 ms | 70.0 tok/s |
| **GPT-4o** (`gpt-4o`) | `openrouter_market` | $2.50 | $10.00 | $1.2500 | 450 ms | 70.0 tok/s |
| **GPT-4o** (`gpt-4o`) | `openrouter_market` | $2.50 | $10.00 | $1.2500 | 450 ms | 70.0 tok/s |
| **GPT-4o Mini** (`gpt-4o-mini`) | `openrouter` | $0.07 | $0.30 | $0.0187 | 450 ms | 120.0 tok/s |
| **GPT-4o Mini** (`gpt-4o-mini`) | `openrouter_market` | $0.07 | $0.30 | $0.0375 | 450 ms | 130.0 tok/s |
| **GPT-4o Mini** (`gpt-4o-mini`) | `openrouter_market` | $0.07 | $0.30 | $0.0375 | 450 ms | 130.0 tok/s |
| **GPT-4o Mini** (`gpt-4o-mini`) | `openrouter` | $0.15 | $0.60 | $0.0375 | 450 ms | 120.0 tok/s |
| **GPT-4o Mini** (`gpt-4o-mini`) | `openrouter_market` | $0.15 | $0.60 | $0.0750 | 450 ms | 130.0 tok/s |
| **GPT-4o Mini** (`gpt-4o-mini`) | `openrouter_market` | $0.15 | $0.60 | $0.0750 | 450 ms | 130.0 tok/s |
| **GPT-4o Mini** (`gpt-4o-mini`) | `nous_portal` | $0.50 | $2.00 | $0.1000 | 220 ms | 130.0 tok/s |
| **GPT-4o Mini** (`gpt-4o-mini`) | `nous_portal` | $0.50 | $2.00 | $0.1000 | 220 ms | 130.0 tok/s |
| **Hermes 3 (Llama 3.1 405B)** (`hermes-3-llama-3.1-405b`) | `nous_portal` | $0.50 | $2.00 | $0.1000 | 550 ms | 40.0 tok/s |
| **Hermes 3 (Llama 3.1 405B)** (`hermes-3-llama-3.1-405b`) | `nous_portal` | $0.50 | $2.00 | $0.1000 | 550 ms | 40.0 tok/s |
| **Hermes 3 (Llama 3.1 405B)** (`hermes-3-llama-3.1-405b`) | `openrouter` | $1.00 | $1.00 | $0.2500 | 450 ms | 35.0 tok/s |
| **Hermes 3 (Llama 3.1 405B)** (`hermes-3-llama-3.1-405b`) | `openrouter_market` | $1.00 | $1.00 | $0.2500 | 450 ms | 35.0 tok/s |
| **Hermes 3 (Llama 3.1 405B)** (`hermes-3-llama-3.1-405b`) | `openrouter_market` | $1.00 | $1.00 | $0.2500 | 450 ms | 35.0 tok/s |
| **Hermes 3 (Llama 3.1 405B)** (`hermes-3-llama-3.1-405b`) | `openrouter_market` | $1.00 | $1.00 | $0.2500 | 450 ms | 35.0 tok/s |
| **Hermes 3 (Llama 3.1 405B)** (`hermes-3-llama-3.1-405b`) | `nous_portal` | $1.20 | $3.50 | $0.3000 | 650 ms | 38.0 tok/s |
| **Hermes 3 (Llama 3.1 70B)** (`hermes-3-llama-3.1-70b`) | `nous_portal` | $0.35 | $0.90 | $0.0900 | 310 ms | 78.0 tok/s |
| **Hermes 3 (Llama 3.1 70B)** (`hermes-3-llama-3.1-70b`) | `nous_portal` | $0.50 | $2.00 | $0.1000 | 380 ms | 75.0 tok/s |
| **Hermes 3 (Llama 3.1 70B)** (`hermes-3-llama-3.1-70b`) | `nous_portal` | $0.50 | $2.00 | $0.1000 | 380 ms | 75.0 tok/s |
| **Hermes 3 (Llama 3.1 70B)** (`hermes-3-llama-3.1-70b`) | `openrouter` | $0.70 | $0.70 | $0.1750 | 450 ms | 70.0 tok/s |
| **Hermes 3 (Llama 3.1 70B)** (`hermes-3-llama-3.1-70b`) | `openrouter_market` | $0.70 | $0.70 | $0.1750 | 450 ms | 70.0 tok/s |
| **Hermes 3 (Llama 3.1 70B)** (`hermes-3-llama-3.1-70b`) | `openrouter_market` | $0.70 | $0.70 | $0.1750 | 450 ms | 70.0 tok/s |
| **Hermes 3 (Llama 3.1 70B)** (`hermes-3-llama-3.1-70b`) | `openrouter_market` | $0.70 | $0.70 | $0.1750 | 450 ms | 70.0 tok/s |
| **IBM: Granite 4.2 8B** (`ibm-granite/granite-4.2-8b`) | `nous_portal` | $0.06 | $0.25 | $0.0150 | 380 ms | 75.0 tok/s |
| **IBM: Granite 4.2 8B** (`ibm-granite/granite-4.2-8b`) | `openrouter_market` | $0.06 | $0.25 | $0.0150 | 450 ms | 70.0 tok/s |
| **Inception: Mercury 2** (`inception/mercury-2`) | `nous_portal` | $0.25 | $0.75 | $0.0250 | 380 ms | 75.0 tok/s |
| **Inception: Mercury 2** (`inception/mercury-2`) | `openrouter_market` | $0.25 | $0.75 | $0.0250 | 450 ms | 70.0 tok/s |
| **Inception: Mercury 2.5** (`inception/mercury-2.5`) | `nous_portal` | $0.04 | $0.15 | $0.0040 | 380 ms | 75.0 tok/s |
| **Inception: Mercury 2.5** (`inception/mercury-2.5`) | `openrouter_market` | $0.04 | $0.15 | $0.0040 | 450 ms | 70.0 tok/s |
| **inclusionAI: Ling 3.0 Flash** (`inclusionai/ling-3.0-flash`) | `nous_portal` | $0.02 | $0.06 | $0.0042 | 220 ms | 130.0 tok/s |
| **inclusionAI: Ling 3.0 Flash** (`inclusionai/ling-3.0-flash`) | `openrouter_market` | $0.02 | $0.06 | $0.0042 | 450 ms | 130.0 tok/s |
| **inclusionAI: Ling 3.0 Flash VL** (`inclusionai/ling-3.0-flash-vl`) | `nous_portal` | $0.06 | $0.18 | $0.0120 | 220 ms | 130.0 tok/s |
| **inclusionAI: Ling 3.0 Flash VL** (`inclusionai/ling-3.0-flash-vl`) | `openrouter_market` | $0.06 | $0.18 | $0.0120 | 450 ms | 130.0 tok/s |
| **inclusionAI: Ling 3.1 Flash** (`inclusionai/ling-3.1-flash`) | `nous_portal` | $0.07 | $0.22 | $0.0150 | 220 ms | 130.0 tok/s |
| **inclusionAI: Ling 3.1 Flash** (`inclusionai/ling-3.1-flash`) | `openrouter_market` | $0.07 | $0.22 | $0.0150 | 450 ms | 130.0 tok/s |
| **Intfloat: E5-Base-v2** (`intfloat/e5-base-v2`) | `nous_portal` | $0.01 | $0.00 | $0.0010 | 380 ms | 75.0 tok/s |
| **Intfloat: E5-Large-v2** (`intfloat/e5-large-v2`) | `nous_portal` | $0.01 | $0.00 | $0.0020 | 380 ms | 75.0 tok/s |
| **Intfloat: Multilingual-E5-Large** (`intfloat/multilingual-e5-large`) | `nous_portal` | $0.01 | $0.00 | $0.0020 | 380 ms | 75.0 tok/s |
| **LiquidAI: LFM2.5-24B-A2B** (`liquid/lfm-2.5-24b-a2b`) | `nous_portal` | $0.05 | $0.70 | $0.0250 | 380 ms | 75.0 tok/s |
| **LiquidAI: LFM2.5-24B-A2B** (`liquid/lfm-2.5-24b-a2b`) | `openrouter_market` | $0.05 | $0.70 | $0.0250 | 450 ms | 70.0 tok/s |
| **Llama 3.3 70B Instruct** (`llama-3.3-70b`) | `nous_portal` | $0.50 | $2.00 | $0.1000 | 380 ms | 75.0 tok/s |
| **Llama 3.3 70B Instruct** (`llama-3.3-70b`) | `nous_portal` | $0.50 | $2.00 | $0.1000 | 380 ms | 75.0 tok/s |
| **Llama 3.3 70B Instruct** (`llama-3.3-70b`) | `openrouter` | $0.59 | $0.79 | $0.1475 | 450 ms | 70.0 tok/s |
| **Llama 3.3 70B Instruct** (`llama-3.3-70b`) | `openrouter_market` | $0.59 | $0.79 | $0.2950 | 450 ms | 70.0 tok/s |
| **Llama 3.3 70B Instruct** (`llama-3.3-70b`) | `openrouter_market` | $0.59 | $0.79 | $0.2950 | 450 ms | 70.0 tok/s |
| **Meituan: LongCat 2.0** (`meituan/longcat-2.0`) | `nous_portal` | $0.30 | $1.20 | $0.0060 | 380 ms | 75.0 tok/s |
| **Meituan: LongCat 2.0** (`meituan/longcat-2.0`) | `openrouter_market` | $0.30 | $1.20 | $0.0060 | 450 ms | 70.0 tok/s |
| **Meta: Llama 3.1 70B Instruct** (`meta-llama/llama-3.1-70b-instruct`) | `nous_portal` | $0.40 | $0.40 | $0.0800 | 380 ms | 75.0 tok/s |
| **Meta: Llama 3.1 70B Instruct** (`meta-llama/llama-3.1-70b-instruct`) | `openrouter_market` | $0.40 | $0.40 | $0.1000 | 450 ms | 70.0 tok/s |
| **Meta: Llama 3.1 8B Instruct** (`meta-llama/llama-3.1-8b-instruct`) | `nous_portal` | $0.02 | $0.04 | $0.0040 | 380 ms | 75.0 tok/s |
| **Meta: Llama 3.1 8B Instruct** (`meta-llama/llama-3.1-8b-instruct`) | `openrouter_market` | $0.05 | $0.08 | $0.0250 | 450 ms | 70.0 tok/s |
| **Meta: Llama 3.3 70B Instruct** (`meta-llama/llama-3.3-70b-instruct`) | `nous_portal` | $0.10 | $0.32 | $0.0200 | 380 ms | 75.0 tok/s |
| **Meta: Llama 3.3 70B Instruct** (`meta-llama/llama-3.3-70b-instruct`) | `openrouter_market` | $0.59 | $0.79 | $0.2950 | 450 ms | 70.0 tok/s |
| **Meta: Llama 4 Maverick** (`meta-llama/llama-4-maverick`) | `nous_portal` | $0.19 | $0.65 | $0.0500 | 380 ms | 75.0 tok/s |
| **Meta: Llama 4 Maverick** (`meta-llama/llama-4-maverick`) | `openrouter_market` | $0.19 | $0.65 | $0.0500 | 450 ms | 70.0 tok/s |
| **Meta: Llama 4 Scout** (`meta-llama/llama-4-scout`) | `nous_portal` | $0.10 | $0.30 | $0.0200 | 380 ms | 75.0 tok/s |
| **Meta: Llama 4 Scout** (`meta-llama/llama-4-scout`) | `openrouter_market` | $0.18 | $0.59 | $0.0450 | 450 ms | 70.0 tok/s |
| **Meta: Muse Glimmer 30B** (`meta/muse-glimmer-30b`) | `nous_portal` | $0.30 | $1.20 | $0.0400 | 380 ms | 75.0 tok/s |
| **Meta: Muse Glimmer 30B** (`meta/muse-glimmer-30b`) | `openrouter_market` | $0.30 | $1.20 | $0.0400 | 450 ms | 70.0 tok/s |
| **Meta: Muse Spark 1.1** (`meta/muse-spark-1.1`) | `nous_portal` | $1.25 | $4.25 | $0.1500 | 380 ms | 75.0 tok/s |
| **Meta: Muse Spark 1.1** (`meta/muse-spark-1.1`) | `openrouter_market` | $1.25 | $4.25 | $0.1500 | 450 ms | 70.0 tok/s |
| **Meta: Muse Spark 1.2** (`meta/muse-spark-1.2`) | `nous_portal` | $1.25 | $4.25 | $0.1500 | 380 ms | 75.0 tok/s |
| **Meta: Muse Spark 1.2** (`meta/muse-spark-1.2`) | `openrouter_market` | $1.25 | $4.25 | $0.1500 | 450 ms | 70.0 tok/s |
| **Meta: Muse Spark 1.3** (`meta/muse-spark-1.3`) | `nous_portal` | $1.25 | $4.25 | $0.1500 | 380 ms | 75.0 tok/s |
| **Meta: Muse Spark 1.3** (`meta/muse-spark-1.3`) | `openrouter_market` | $1.25 | $4.25 | $0.1500 | 450 ms | 70.0 tok/s |
| **Meta: Muse Wonder** (`meta/muse-wonder`) | `nous_portal` | $4.00 | $20.00 | $0.2000 | 380 ms | 75.0 tok/s |
| **Meta: Muse Wonder** (`meta/muse-wonder`) | `openrouter_market` | $4.00 | $20.00 | $0.2000 | 450 ms | 70.0 tok/s |
| **MiniMax: MiniMax M1** (`minimax/minimax-m1`) | `nous_portal` | $0.40 | $2.20 | $0.0800 | 220 ms | 130.0 tok/s |
| **MiniMax: MiniMax M1** (`minimax/minimax-m1`) | `openrouter_market` | $0.55 | $2.20 | $0.1375 | 450 ms | 130.0 tok/s |
| **MiniMax: MiniMax M2** (`minimax/minimax-m2`) | `nous_portal` | $0.26 | $1.02 | $0.0510 | 220 ms | 130.0 tok/s |
| **MiniMax: MiniMax M2** (`minimax/minimax-m2`) | `openrouter_market` | $0.26 | $1.02 | $0.0638 | 450 ms | 130.0 tok/s |
| **MiniMax: MiniMax M2.1** (`minimax/minimax-m2.1`) | `nous_portal` | $0.30 | $1.20 | $0.0300 | 220 ms | 130.0 tok/s |
| **MiniMax: MiniMax M2.1** (`minimax/minimax-m2.1`) | `openrouter_market` | $0.30 | $1.20 | $0.0300 | 450 ms | 130.0 tok/s |
| **MiniMax: MiniMax M2.5** (`minimax/minimax-m2.5`) | `nous_portal` | $0.27 | $0.95 | $0.0300 | 220 ms | 130.0 tok/s |
| **MiniMax: MiniMax M2.5** (`minimax/minimax-m2.5`) | `openrouter_market` | $0.27 | $1.08 | $0.0270 | 450 ms | 130.0 tok/s |
| **MiniMax: MiniMax M2.7** (`minimax/minimax-m2.7`) | `nous_portal` | $0.21 | $0.84 | $0.0420 | 220 ms | 130.0 tok/s |
| **MiniMax: MiniMax M2.7** (`minimax/minimax-m2.7`) | `openrouter_market` | $0.30 | $1.20 | $0.0600 | 450 ms | 130.0 tok/s |
| **MiniMax: MiniMax M3** (`minimax/minimax-m3`) | `nous_portal` | $0.23 | $0.96 | $0.0500 | 220 ms | 130.0 tok/s |
| **MiniMax: MiniMax M3** (`minimax/minimax-m3`) | `openrouter_market` | $0.30 | $1.20 | $0.0600 | 450 ms | 130.0 tok/s |
| **Mistral: Codestral 2508** (`mistralai/codestral-2508`) | `nous_portal_batch` | $0.15 | $0.45 | $0.0150 | 5000 ms | 150.0 tok/s |
| **Mistral: Codestral 2508** (`mistralai/codestral-2508`) | `openrouter_market` | $0.15 | $0.45 | $0.0150 | 450 ms | 70.0 tok/s |
| **Mistral: Codestral 2508** (`mistralai/codestral-2508`) | `nous_portal` | $0.30 | $0.90 | $0.0300 | 380 ms | 75.0 tok/s |
| **Mistral: Codestral 2508** (`mistralai/codestral-2508`) | `openrouter_market` | $0.30 | $0.90 | $0.0300 | 450 ms | 70.0 tok/s |
| **Mistral: Codestral Embed 2505** (`mistralai/codestral-embed-2505`) | `nous_portal` | $0.15 | $0.00 | $0.0300 | 380 ms | 75.0 tok/s |
| **Mistral: Devstral 2 2512** (`mistralai/devstral-2512`) | `nous_portal` | $0.40 | $2.00 | $0.0400 | 380 ms | 75.0 tok/s |
| **Mistral: Devstral 2 2512** (`mistralai/devstral-2512`) | `openrouter_market` | $0.40 | $2.00 | $0.0400 | 450 ms | 70.0 tok/s |
| **Mistral: Ministral 3 14B 2512** (`mistralai/ministral-14b-2512`) | `nous_portal` | $0.20 | $0.20 | $0.0200 | 220 ms | 130.0 tok/s |
| **Mistral: Ministral 3 14B 2512** (`mistralai/ministral-14b-2512`) | `openrouter_market` | $0.20 | $0.20 | $0.0200 | 450 ms | 130.0 tok/s |
| **Mistral: Ministral 3 3B 2512** (`mistralai/ministral-3b-2512`) | `nous_portal` | $0.10 | $0.10 | $0.0100 | 220 ms | 130.0 tok/s |
| **Mistral: Ministral 3 3B 2512** (`mistralai/ministral-3b-2512`) | `openrouter_market` | $0.10 | $0.10 | $0.0100 | 450 ms | 130.0 tok/s |
| **Mistral: Ministral 3 8B 2512** (`mistralai/ministral-8b-2512`) | `nous_portal_batch` | $0.07 | $0.07 | $0.0075 | 5000 ms | 150.0 tok/s |
| **Mistral: Ministral 3 8B 2512** (`mistralai/ministral-8b-2512`) | `openrouter_market` | $0.07 | $0.07 | $0.0075 | 450 ms | 130.0 tok/s |
| **Mistral: Ministral 3 8B 2512** (`mistralai/ministral-8b-2512`) | `nous_portal` | $0.15 | $0.15 | $0.0150 | 220 ms | 130.0 tok/s |
| **Mistral: Ministral 3 8B 2512** (`mistralai/ministral-8b-2512`) | `openrouter_market` | $0.15 | $0.15 | $0.0150 | 450 ms | 130.0 tok/s |
| **Mistral: Mistral Embed 2312** (`mistralai/mistral-embed-2312`) | `nous_portal` | $0.10 | $0.00 | $0.0200 | 380 ms | 75.0 tok/s |
| **Mistral Large** (`mistralai/mistral-large`) | `nous_portal` | $2.00 | $6.00 | $0.2000 | 380 ms | 75.0 tok/s |
| **Mistral Large** (`mistralai/mistral-large`) | `openrouter_market` | $2.00 | $6.00 | $0.2000 | 450 ms | 70.0 tok/s |
| **Mistral Large 2407** (`mistralai/mistral-large-2407`) | `nous_portal` | $2.00 | $6.00 | $0.2000 | 380 ms | 75.0 tok/s |
| **Mistral Large 2407** (`mistralai/mistral-large-2407`) | `openrouter_market` | $2.00 | $6.00 | $0.2000 | 450 ms | 70.0 tok/s |
| **Mistral: Mistral Large 3 2512** (`mistralai/mistral-large-2512`) | `nous_portal_batch` | $0.25 | $0.75 | $0.0250 | 5000 ms | 150.0 tok/s |
| **Mistral: Mistral Large 3 2512** (`mistralai/mistral-large-2512`) | `openrouter_market` | $0.25 | $0.75 | $0.0250 | 450 ms | 70.0 tok/s |
| **Mistral: Mistral Large 3 2512** (`mistralai/mistral-large-2512`) | `nous_portal` | $0.50 | $1.50 | $0.0500 | 380 ms | 75.0 tok/s |
| **Mistral: Mistral Large 3 2512** (`mistralai/mistral-large-2512`) | `openrouter_market` | $0.50 | $1.50 | $0.0500 | 450 ms | 70.0 tok/s |
| **Mistral: Mistral Large 4** (`mistralai/mistral-large-4-0`) | `nous_portal` | $0.68 | $2.09 | $0.0700 | 380 ms | 75.0 tok/s |
| **Mistral: Mistral Large 4** (`mistralai/mistral-large-4-0`) | `openrouter_market` | $0.68 | $2.09 | $0.0700 | 450 ms | 70.0 tok/s |
| **Mistral: Mistral Medium 3** (`mistralai/mistral-medium-3`) | `nous_portal` | $0.40 | $2.00 | $0.0400 | 380 ms | 75.0 tok/s |
| **Mistral: Mistral Medium 3** (`mistralai/mistral-medium-3`) | `openrouter_market` | $0.40 | $2.00 | $0.0400 | 450 ms | 70.0 tok/s |
| **Mistral: Mistral Medium 3.5** (`mistralai/mistral-medium-3-5`) | `nous_portal_batch` | $0.75 | $3.75 | $0.1500 | 5000 ms | 150.0 tok/s |
| **Mistral: Mistral Medium 3.5** (`mistralai/mistral-medium-3-5`) | `openrouter_market` | $0.75 | $3.75 | $0.1875 | 450 ms | 70.0 tok/s |
| **Mistral: Mistral Medium 3.5** (`mistralai/mistral-medium-3-5`) | `nous_portal` | $1.50 | $7.50 | $0.3000 | 380 ms | 75.0 tok/s |
| **Mistral: Mistral Medium 3.5** (`mistralai/mistral-medium-3-5`) | `openrouter_market` | $1.50 | $7.50 | $0.3750 | 450 ms | 70.0 tok/s |
| **Mistral: Mistral Medium 3.1** (`mistralai/mistral-medium-3.1`) | `nous_portal_batch` | $0.20 | $1.00 | $0.0200 | 5000 ms | 150.0 tok/s |
| **Mistral: Mistral Medium 3.1** (`mistralai/mistral-medium-3.1`) | `openrouter_market` | $0.20 | $1.00 | $0.0200 | 450 ms | 70.0 tok/s |
| **Mistral: Mistral Medium 3.1** (`mistralai/mistral-medium-3.1`) | `nous_portal` | $0.40 | $2.00 | $0.0400 | 380 ms | 75.0 tok/s |
| **Mistral: Mistral Medium 3.1** (`mistralai/mistral-medium-3.1`) | `openrouter_market` | $0.40 | $2.00 | $0.0400 | 450 ms | 70.0 tok/s |
| **Mistral: Mistral Nemo** (`mistralai/mistral-nemo`) | `nous_portal` | $0.02 | $0.02 | $0.0108 | 380 ms | 75.0 tok/s |
| **Mistral: Mistral Nemo** (`mistralai/mistral-nemo`) | `openrouter_market` | $0.03 | $0.03 | $0.0073 | 450 ms | 70.0 tok/s |
| **Mistral: Saba** (`mistralai/mistral-saba`) | `nous_portal` | $0.20 | $0.60 | $0.0200 | 380 ms | 75.0 tok/s |
| **Mistral: Saba** (`mistralai/mistral-saba`) | `openrouter_market` | $0.20 | $0.60 | $0.0200 | 450 ms | 70.0 tok/s |
| **Mistral: Mistral Small 4** (`mistralai/mistral-small-2603`) | `nous_portal_batch` | $0.07 | $0.30 | $0.0075 | 5000 ms | 150.0 tok/s |
| **Mistral: Mistral Small 4** (`mistralai/mistral-small-2603`) | `openrouter_market` | $0.07 | $0.30 | $0.0075 | 450 ms | 70.0 tok/s |
| **Mistral: Mistral Small 4** (`mistralai/mistral-small-2603`) | `nous_portal` | $0.15 | $0.60 | $0.0150 | 380 ms | 75.0 tok/s |
| **Mistral: Mistral Small 4** (`mistralai/mistral-small-2603`) | `openrouter_market` | $0.15 | $0.60 | $0.0150 | 450 ms | 70.0 tok/s |
| **Mistral: Mistral Small 3.1 24B** (`mistralai/mistral-small-3.1-24b-instruct`) | `nous_portal` | $0.35 | $0.55 | $0.0702 | 380 ms | 75.0 tok/s |
| **Mistral: Mistral Small 3.1 24B** (`mistralai/mistral-small-3.1-24b-instruct`) | `openrouter_market` | $0.35 | $0.55 | $0.0877 | 450 ms | 70.0 tok/s |
| **Mistral: Mistral Small 3.2 24B** (`mistralai/mistral-small-3.2-24b-instruct`) | `nous_portal` | $0.07 | $0.20 | $0.0150 | 380 ms | 75.0 tok/s |
| **Mistral: Mistral Small 3.2 24B** (`mistralai/mistral-small-3.2-24b-instruct`) | `openrouter_market` | $0.09 | $0.25 | $0.0234 | 450 ms | 70.0 tok/s |
| **Mistral: Mixtral 8x22B Instruct** (`mistralai/mixtral-8x22b-instruct`) | `nous_portal` | $2.00 | $6.00 | $0.2000 | 380 ms | 75.0 tok/s |
| **Mistral: Mixtral 8x22B Instruct** (`mistralai/mixtral-8x22b-instruct`) | `openrouter_market` | $2.00 | $6.00 | $0.2000 | 450 ms | 70.0 tok/s |
| **Mistral: Voxtral Small 24B 2507** (`mistralai/voxtral-small-24b-2507`) | `nous_portal` | $0.10 | $0.30 | $0.0100 | 380 ms | 75.0 tok/s |
| **Mistral: Voxtral Small 24B 2507** (`mistralai/voxtral-small-24b-2507`) | `openrouter_market` | $0.10 | $0.30 | $0.0100 | 450 ms | 70.0 tok/s |
| **MoonshotAI: Kimi K2 0711** (`moonshotai/kimi-k2`) | `nous_portal` | $0.57 | $2.30 | $0.1140 | 380 ms | 75.0 tok/s |
| **MoonshotAI: Kimi K2 0711** (`moonshotai/kimi-k2`) | `openrouter_market` | $0.57 | $2.30 | $0.1425 | 450 ms | 70.0 tok/s |
| **MoonshotAI: Kimi K2 0905** (`moonshotai/kimi-k2-0905`) | `nous_portal` | $0.60 | $2.50 | $0.1200 | 380 ms | 75.0 tok/s |
| **MoonshotAI: Kimi K2 0905** (`moonshotai/kimi-k2-0905`) | `openrouter_market` | $0.60 | $2.50 | $0.1500 | 450 ms | 70.0 tok/s |
| **MoonshotAI: Kimi K2 Thinking** (`moonshotai/kimi-k2-thinking`) | `nous_portal` | $0.60 | $2.50 | $0.1200 | 380 ms | 75.0 tok/s |
| **MoonshotAI: Kimi K2 Thinking** (`moonshotai/kimi-k2-thinking`) | `openrouter_market` | $0.60 | $2.50 | $0.1500 | 450 ms | 70.0 tok/s |
| **MoonshotAI: Kimi K2.5** (`moonshotai/kimi-k2.5`) | `nous_portal` | $0.49 | $2.50 | $0.2000 | 380 ms | 75.0 tok/s |
| **MoonshotAI: Kimi K2.5** (`moonshotai/kimi-k2.5`) | `openrouter_market` | $0.50 | $2.50 | $0.1500 | 450 ms | 70.0 tok/s |
| **MoonshotAI: Kimi K2.6** (`moonshotai/kimi-k2.6`) | `nous_portal` | $0.44 | $2.45 | $0.0990 | 380 ms | 75.0 tok/s |
| **MoonshotAI: Kimi K2.6** (`moonshotai/kimi-k2.6`) | `openrouter_market` | $0.65 | $3.41 | $0.1500 | 450 ms | 70.0 tok/s |
| **MoonshotAI: Kimi K2.7 Code** (`moonshotai/kimi-k2.7-code`) | `nous_portal` | $0.67 | $3.35 | $0.1800 | 380 ms | 75.0 tok/s |
| **MoonshotAI: Kimi K2.7 Code** (`moonshotai/kimi-k2.7-code`) | `openrouter_market` | $0.67 | $3.35 | $0.1800 | 450 ms | 70.0 tok/s |
| **MoonshotAI: Kimi K3** (`moonshotai/kimi-k3`) | `nous_portal` | $0.70 | $15.00 | $0.3200 | 380 ms | 75.0 tok/s |
| **MoonshotAI: Kimi K3** (`moonshotai/kimi-k3`) | `openrouter_market` | $0.70 | $15.00 | $0.3200 | 450 ms | 70.0 tok/s |
| **MoonshotAI: Kimi K3** (`moonshotai/kimi-k3`) | `nous_portal_batch` | $2.28 | $11.40 | $0.2280 | 5000 ms | 150.0 tok/s |
| **MoonshotAI: Kimi K3** (`moonshotai/kimi-k3`) | `openrouter_market` | $2.28 | $11.40 | $0.2280 | 450 ms | 70.0 tok/s |
| **MoonshotAI: Kimi K3** (`moonshotai/kimi-k3:US`) | `nous_portal` | $3.63 | $18.15 | $0.3630 | 380 ms | 75.0 tok/s |
| **MoonshotAI: Kimi Latest** (`moonshotai/kimi-latest`) | `nous_portal` | $0.70 | $15.00 | $0.3200 | 380 ms | 75.0 tok/s |
| **MoonshotAI: Kimi Latest** (`moonshotai/kimi-latest`) | `openrouter_market` | $0.70 | $15.00 | $0.3200 | 450 ms | 70.0 tok/s |
| **Nex AGI: Nex-N2.5-Pro** (`nex-agi/nex-n2.5-pro`) | `nous_portal` | $0.07 | $0.25 | $0.0150 | 380 ms | 75.0 tok/s |
| **Nex AGI: Nex-N2.5-Pro** (`nex-agi/nex-n2.5-pro`) | `openrouter_market` | $0.07 | $0.25 | $0.0150 | 450 ms | 70.0 tok/s |
| **NVIDIA: Nemotron 3 Nano 30B A3B** (`nvidia/nemotron-3-nano-30b-a3b`) | `nous_portal` | $0.05 | $0.20 | $0.0100 | 380 ms | 75.0 tok/s |
| **NVIDIA: Nemotron 3 Nano 30B A3B** (`nvidia/nemotron-3-nano-30b-a3b`) | `openrouter_market` | $0.06 | $0.24 | $0.0150 | 450 ms | 70.0 tok/s |
| **NVIDIA: Nemotron 3 Super** (`nvidia/nemotron-3-super-120b-a12b`) | `openrouter_market` | $0.00 | $0.00 | N/A | 450 ms | 70.0 tok/s |
| **NVIDIA: Nemotron 3 Super** (`nvidia/nemotron-3-super-120b-a12b`) | `nous_portal` | $0.08 | $0.45 | $0.0160 | 380 ms | 75.0 tok/s |
| **NVIDIA: Nemotron 3 Super** (`nvidia/nemotron-3-super-120b-a12b`) | `openrouter_market` | $0.08 | $0.45 | $0.0200 | 450 ms | 70.0 tok/s |
| **NVIDIA: Nemotron 3 Ultra** (`nvidia/nemotron-3-ultra-550b-a55b`) | `openrouter_market` | $0.00 | $0.00 | N/A | 450 ms | 70.0 tok/s |
| **NVIDIA: Nemotron 3 Ultra** (`nvidia/nemotron-3-ultra-550b-a55b`) | `nous_portal` | $0.50 | $2.20 | $0.1000 | 380 ms | 75.0 tok/s |
| **NVIDIA: Nemotron 3 Ultra** (`nvidia/nemotron-3-ultra-550b-a55b`) | `openrouter_market` | $0.60 | $2.40 | $0.1200 | 450 ms | 70.0 tok/s |
| **NVIDIA: Nemotron 3.5 Lightning** (`nvidia/nemotron-3.5-lightning`) | `openrouter_market` | $0.00 | $0.00 | N/A | 450 ms | 70.0 tok/s |
| **NVIDIA: Nemotron 3.5 Lightning** (`nvidia/nemotron-3.5-lightning`) | `nous_portal` | $0.02 | $0.12 | $0.0125 | 380 ms | 75.0 tok/s |
| **NVIDIA: Nemotron 3.5 Lightning** (`nvidia/nemotron-3.5-lightning`) | `openrouter_market` | $0.07 | $0.20 | $0.0400 | 450 ms | 70.0 tok/s |
| **OpenAI o3-mini** (`o3-mini`) | `nous_portal` | $0.50 | $2.00 | $0.1000 | 220 ms | 130.0 tok/s |
| **OpenAI o3-mini** (`o3-mini`) | `nous_portal` | $0.50 | $2.00 | $0.1000 | 220 ms | 130.0 tok/s |
| **OpenAI o3-mini** (`o3-mini`) | `openrouter` | $0.55 | $2.20 | $0.1375 | 450 ms | 120.0 tok/s |
| **OpenAI o3-mini** (`o3-mini`) | `openrouter_market` | $0.55 | $2.20 | $0.2750 | 450 ms | 130.0 tok/s |
| **OpenAI o3-mini** (`o3-mini`) | `openrouter_market` | $0.55 | $2.20 | $0.2750 | 450 ms | 130.0 tok/s |
| **OpenAI o3-mini** (`o3-mini`) | `openrouter` | $1.10 | $4.40 | $0.2750 | 450 ms | 120.0 tok/s |
| **OpenAI o3-mini** (`o3-mini`) | `openrouter_market` | $1.10 | $4.40 | $0.5500 | 450 ms | 130.0 tok/s |
| **OpenAI o3-mini** (`o3-mini`) | `openrouter_market` | $1.10 | $4.40 | $0.5500 | 450 ms | 130.0 tok/s |
| **OpenAI: GPT-3.5 Turbo** (`openai/gpt-3.5-turbo`) | `nous_portal_batch` | $0.25 | $0.75 | $0.0500 | 5000 ms | 150.0 tok/s |
| **OpenAI: GPT-3.5 Turbo** (`openai/gpt-3.5-turbo`) | `openrouter_market` | $0.25 | $0.75 | $0.0625 | 450 ms | 70.0 tok/s |
| **OpenAI: GPT-3.5 Turbo** (`openai/gpt-3.5-turbo`) | `nous_portal` | $0.50 | $1.50 | $0.1000 | 380 ms | 75.0 tok/s |
| **OpenAI: GPT-3.5 Turbo** (`openai/gpt-3.5-turbo`) | `openrouter_market` | $0.50 | $1.50 | $0.1250 | 450 ms | 70.0 tok/s |
| **OpenAI: GPT-3.5 Turbo (older v0613)** (`openai/gpt-3.5-turbo-0613`) | `nous_portal` | $1.00 | $2.00 | $0.2000 | 380 ms | 75.0 tok/s |
| **OpenAI: GPT-3.5 Turbo (older v0613)** (`openai/gpt-3.5-turbo-0613`) | `openrouter_market` | $1.00 | $2.00 | $0.2500 | 450 ms | 70.0 tok/s |
| **OpenAI: GPT-3.5 Turbo 16k** (`openai/gpt-3.5-turbo-16k`) | `nous_portal` | $3.00 | $4.00 | $0.6000 | 380 ms | 75.0 tok/s |
| **OpenAI: GPT-3.5 Turbo 16k** (`openai/gpt-3.5-turbo-16k`) | `openrouter_market` | $3.00 | $4.00 | $0.7500 | 450 ms | 70.0 tok/s |
| **OpenAI: GPT-4** (`openai/gpt-4`) | `nous_portal` | $30.00 | $60.00 | $6.0000 | 380 ms | 75.0 tok/s |
| **OpenAI: GPT-4** (`openai/gpt-4`) | `openrouter_market` | $30.00 | $60.00 | $7.5000 | 450 ms | 70.0 tok/s |
| **OpenAI: GPT-4 Turbo** (`openai/gpt-4-turbo`) | `nous_portal_batch` | $5.00 | $15.00 | $1.0000 | 5000 ms | 150.0 tok/s |
| **OpenAI: GPT-4 Turbo** (`openai/gpt-4-turbo`) | `openrouter_market` | $5.00 | $15.00 | $1.2500 | 450 ms | 70.0 tok/s |
| **OpenAI: GPT-4 Turbo** (`openai/gpt-4-turbo`) | `nous_portal` | $10.00 | $30.00 | $2.0000 | 380 ms | 75.0 tok/s |
| **OpenAI: GPT-4 Turbo** (`openai/gpt-4-turbo`) | `openrouter_market` | $10.00 | $30.00 | $2.5000 | 450 ms | 70.0 tok/s |
| **OpenAI: GPT-4.1** (`openai/gpt-4.1`) | `nous_portal_batch` | $1.00 | $4.00 | $0.2500 | 5000 ms | 150.0 tok/s |
| **OpenAI: GPT-4.1** (`openai/gpt-4.1`) | `openrouter_market` | $1.00 | $4.00 | $0.2500 | 450 ms | 70.0 tok/s |
| **OpenAI: GPT-4.1** (`openai/gpt-4.1`) | `nous_portal` | $2.00 | $8.00 | $0.5000 | 380 ms | 75.0 tok/s |
| **OpenAI: GPT-4.1** (`openai/gpt-4.1`) | `openrouter_market` | $2.00 | $8.00 | $0.5000 | 450 ms | 70.0 tok/s |
| **OpenAI: GPT-4.1 Mini** (`openai/gpt-4.1-mini`) | `nous_portal_batch` | $0.20 | $0.80 | $0.0500 | 5000 ms | 150.0 tok/s |
| **OpenAI: GPT-4.1 Mini** (`openai/gpt-4.1-mini`) | `openrouter_market` | $0.20 | $0.80 | $0.0500 | 450 ms | 130.0 tok/s |
| **OpenAI: GPT-4.1 Mini** (`openai/gpt-4.1-mini`) | `nous_portal` | $0.40 | $1.60 | $0.1000 | 220 ms | 130.0 tok/s |
| **OpenAI: GPT-4.1 Mini** (`openai/gpt-4.1-mini`) | `openrouter_market` | $0.40 | $1.60 | $0.1000 | 450 ms | 130.0 tok/s |
| **OpenAI: GPT-4.1 Nano** (`openai/gpt-4.1-nano`) | `nous_portal_batch` | $0.05 | $0.20 | $0.0125 | 5000 ms | 150.0 tok/s |
| **OpenAI: GPT-4.1 Nano** (`openai/gpt-4.1-nano`) | `openrouter_market` | $0.05 | $0.20 | $0.0125 | 450 ms | 70.0 tok/s |
| **OpenAI: GPT-4.1 Nano** (`openai/gpt-4.1-nano`) | `nous_portal` | $0.10 | $0.40 | $0.0300 | 380 ms | 75.0 tok/s |
| **OpenAI: GPT-4.1 Nano** (`openai/gpt-4.1-nano`) | `openrouter_market` | $0.10 | $0.40 | $0.0250 | 450 ms | 70.0 tok/s |
| **OpenAI: GPT-4o** (`openai/gpt-4o`) | `nous_portal_batch` | $1.25 | $5.00 | $0.6250 | 5000 ms | 150.0 tok/s |
| **OpenAI: GPT-4o** (`openai/gpt-4o`) | `openrouter_market` | $1.25 | $5.00 | $0.6250 | 450 ms | 70.0 tok/s |
| **OpenAI: GPT-4o** (`openai/gpt-4o`) | `nous_portal` | $2.50 | $10.00 | $0.5000 | 380 ms | 75.0 tok/s |
| **OpenAI: GPT-4o** (`openai/gpt-4o`) | `openrouter_market` | $2.50 | $10.00 | $1.2500 | 450 ms | 70.0 tok/s |
| **OpenAI: GPT-4o (2024-05-13)** (`openai/gpt-4o-2024-05-13`) | `nous_portal` | $5.00 | $15.00 | $1.0000 | 380 ms | 75.0 tok/s |
| **OpenAI: GPT-4o (2024-05-13)** (`openai/gpt-4o-2024-05-13`) | `openrouter_market` | $5.00 | $15.00 | $1.2500 | 450 ms | 70.0 tok/s |
| **OpenAI: GPT-4o (2024-08-06)** (`openai/gpt-4o-2024-08-06`) | `nous_portal` | $2.50 | $10.00 | $1.2500 | 380 ms | 75.0 tok/s |
| **OpenAI: GPT-4o (2024-08-06)** (`openai/gpt-4o-2024-08-06`) | `openrouter_market` | $2.50 | $10.00 | $1.2500 | 450 ms | 70.0 tok/s |
| **OpenAI: GPT-4o (2024-11-20)** (`openai/gpt-4o-2024-11-20`) | `nous_portal` | $2.50 | $10.00 | $1.2500 | 380 ms | 75.0 tok/s |
| **OpenAI: GPT-4o (2024-11-20)** (`openai/gpt-4o-2024-11-20`) | `openrouter_market` | $2.50 | $10.00 | $1.2500 | 450 ms | 70.0 tok/s |
| **OpenAI: GPT-4o-mini** (`openai/gpt-4o-mini`) | `nous_portal_batch` | $0.07 | $0.30 | $0.0375 | 5000 ms | 150.0 tok/s |
| **OpenAI: GPT-4o-mini** (`openai/gpt-4o-mini`) | `openrouter_market` | $0.07 | $0.30 | $0.0375 | 450 ms | 130.0 tok/s |
| **OpenAI: GPT-4o-mini** (`openai/gpt-4o-mini`) | `nous_portal` | $0.15 | $0.60 | $0.0750 | 220 ms | 130.0 tok/s |
| **OpenAI: GPT-4o-mini** (`openai/gpt-4o-mini`) | `openrouter_market` | $0.15 | $0.60 | $0.0750 | 450 ms | 130.0 tok/s |
| **OpenAI: GPT-4o-mini (2024-07-18)** (`openai/gpt-4o-mini-2024-07-18`) | `nous_portal` | $0.15 | $0.60 | $0.0750 | 220 ms | 130.0 tok/s |
| **OpenAI: GPT-4o-mini (2024-07-18)** (`openai/gpt-4o-mini-2024-07-18`) | `openrouter_market` | $0.15 | $0.60 | $0.0750 | 450 ms | 130.0 tok/s |
| **OpenAI: GPT-5** (`openai/gpt-5`) | `nous_portal_batch` | $0.62 | $5.00 | $0.0625 | 5000 ms | 150.0 tok/s |
| **OpenAI: GPT-5** (`openai/gpt-5`) | `openrouter_market` | $0.62 | $5.00 | $0.0625 | 450 ms | 70.0 tok/s |
| **OpenAI: GPT-5** (`openai/gpt-5`) | `nous_portal` | $1.25 | $10.00 | $0.1250 | 380 ms | 75.0 tok/s |
| **OpenAI: GPT-5** (`openai/gpt-5`) | `openrouter_market` | $1.25 | $10.00 | $0.1250 | 450 ms | 70.0 tok/s |
| **OpenAI: GPT-5 Mini** (`openai/gpt-5-mini`) | `nous_portal_batch` | $0.12 | $1.00 | $0.0125 | 5000 ms | 150.0 tok/s |
| **OpenAI: GPT-5 Mini** (`openai/gpt-5-mini`) | `openrouter_market` | $0.12 | $1.00 | $0.0125 | 450 ms | 130.0 tok/s |
| **OpenAI: GPT-5 Mini** (`openai/gpt-5-mini`) | `nous_portal` | $0.25 | $2.00 | $0.0300 | 220 ms | 130.0 tok/s |
| **OpenAI: GPT-5 Mini** (`openai/gpt-5-mini`) | `openrouter_market` | $0.25 | $2.00 | $0.0250 | 450 ms | 130.0 tok/s |
| **OpenAI: GPT-5 Nano** (`openai/gpt-5-nano`) | `nous_portal_batch` | $0.02 | $0.20 | $0.0025 | 5000 ms | 150.0 tok/s |
| **OpenAI: GPT-5 Nano** (`openai/gpt-5-nano`) | `openrouter_market` | $0.02 | $0.20 | $0.0025 | 450 ms | 70.0 tok/s |
| **OpenAI: GPT-5 Nano** (`openai/gpt-5-nano`) | `nous_portal` | $0.05 | $0.40 | $0.0100 | 380 ms | 75.0 tok/s |
| **OpenAI: GPT-5 Nano** (`openai/gpt-5-nano`) | `openrouter_market` | $0.05 | $0.40 | $0.0050 | 450 ms | 70.0 tok/s |
| **OpenAI: GPT-5 Pro** (`openai/gpt-5-pro`) | `nous_portal_batch` | $7.50 | $60.00 | $1.5000 | 5000 ms | 150.0 tok/s |
| **OpenAI: GPT-5 Pro** (`openai/gpt-5-pro`) | `openrouter_market` | $7.50 | $60.00 | $1.8750 | 450 ms | 70.0 tok/s |
| **OpenAI: GPT-5 Pro** (`openai/gpt-5-pro`) | `nous_portal` | $15.00 | $120.00 | $3.0000 | 380 ms | 75.0 tok/s |
| **OpenAI: GPT-5 Pro** (`openai/gpt-5-pro`) | `openrouter_market` | $15.00 | $120.00 | $3.7500 | 450 ms | 70.0 tok/s |
| **OpenAI: GPT-5.1** (`openai/gpt-5.1`) | `nous_portal_batch` | $0.62 | $5.00 | $0.0625 | 5000 ms | 150.0 tok/s |
| **OpenAI: GPT-5.1** (`openai/gpt-5.1`) | `openrouter_market` | $0.62 | $5.00 | $0.0625 | 450 ms | 70.0 tok/s |
| **OpenAI: GPT-5.1** (`openai/gpt-5.1`) | `nous_portal` | $1.25 | $10.00 | $0.1300 | 380 ms | 75.0 tok/s |
| **OpenAI: GPT-5.1** (`openai/gpt-5.1`) | `openrouter_market` | $1.25 | $10.00 | $0.1250 | 450 ms | 70.0 tok/s |
| **OpenAI: GPT-5.1-Codex** (`openai/gpt-5.1-codex`) | `nous_portal` | $1.25 | $10.00 | $0.1300 | 380 ms | 75.0 tok/s |
| **OpenAI: GPT-5.1-Codex** (`openai/gpt-5.1-codex`) | `openrouter_market` | $1.25 | $10.00 | $0.1300 | 450 ms | 70.0 tok/s |
| **OpenAI: GPT-5.1-Codex-Max** (`openai/gpt-5.1-codex-max`) | `nous_portal` | $1.25 | $10.00 | $0.1250 | 380 ms | 75.0 tok/s |
| **OpenAI: GPT-5.1-Codex-Max** (`openai/gpt-5.1-codex-max`) | `openrouter_market` | $1.25 | $10.00 | $0.1250 | 450 ms | 70.0 tok/s |
| **OpenAI: GPT-5.1-Codex-Mini** (`openai/gpt-5.1-codex-mini`) | `nous_portal` | $0.25 | $2.00 | $0.0300 | 220 ms | 130.0 tok/s |
| **OpenAI: GPT-5.1-Codex-Mini** (`openai/gpt-5.1-codex-mini`) | `openrouter_market` | $0.25 | $2.00 | $0.0300 | 450 ms | 130.0 tok/s |
| **OpenAI: GPT-5.2** (`openai/gpt-5.2`) | `nous_portal_batch` | $0.88 | $7.00 | $0.0875 | 5000 ms | 150.0 tok/s |
| **OpenAI: GPT-5.2** (`openai/gpt-5.2`) | `openrouter_market` | $0.88 | $7.00 | $0.0875 | 450 ms | 70.0 tok/s |
| **OpenAI: GPT-5.2** (`openai/gpt-5.2`) | `nous_portal` | $1.75 | $14.00 | $0.1750 | 380 ms | 75.0 tok/s |
| **OpenAI: GPT-5.2** (`openai/gpt-5.2`) | `openrouter_market` | $1.75 | $14.00 | $0.1750 | 450 ms | 70.0 tok/s |
| **OpenAI: GPT-5.2 Chat** (`openai/gpt-5.2-chat`) | `nous_portal` | $1.75 | $14.00 | $0.1750 | 380 ms | 75.0 tok/s |
| **OpenAI: GPT-5.2 Chat** (`openai/gpt-5.2-chat`) | `openrouter_market` | $1.75 | $14.00 | $0.1750 | 450 ms | 70.0 tok/s |
| **OpenAI: GPT-5.2-Codex** (`openai/gpt-5.2-codex`) | `nous_portal` | $1.75 | $14.00 | $0.1750 | 380 ms | 75.0 tok/s |
| **OpenAI: GPT-5.2-Codex** (`openai/gpt-5.2-codex`) | `openrouter_market` | $1.75 | $14.00 | $0.1750 | 450 ms | 70.0 tok/s |
| **OpenAI: GPT-5.2 Pro** (`openai/gpt-5.2-pro`) | `nous_portal_batch` | $10.50 | $84.00 | $2.1000 | 5000 ms | 150.0 tok/s |
| **OpenAI: GPT-5.2 Pro** (`openai/gpt-5.2-pro`) | `openrouter_market` | $10.50 | $84.00 | $2.6250 | 450 ms | 70.0 tok/s |
| **OpenAI: GPT-5.2 Pro** (`openai/gpt-5.2-pro`) | `nous_portal` | $21.00 | $168.00 | $4.2000 | 380 ms | 75.0 tok/s |
| **OpenAI: GPT-5.2 Pro** (`openai/gpt-5.2-pro`) | `openrouter_market` | $21.00 | $168.00 | $5.2500 | 450 ms | 70.0 tok/s |
| **OpenAI: GPT-5.3-Codex** (`openai/gpt-5.3-codex`) | `nous_portal` | $1.75 | $14.00 | $0.1750 | 380 ms | 75.0 tok/s |
| **OpenAI: GPT-5.3-Codex** (`openai/gpt-5.3-codex`) | `openrouter_market` | $1.75 | $14.00 | $0.1750 | 450 ms | 70.0 tok/s |
| **OpenAI: GPT-5.4** (`openai/gpt-5.4`) | `nous_portal_batch` | $1.25 | $7.50 | $0.1250 | 5000 ms | 150.0 tok/s |
| **OpenAI: GPT-5.4** (`openai/gpt-5.4`) | `openrouter_market` | $1.25 | $7.50 | $0.1250 | 450 ms | 70.0 tok/s |
| **OpenAI: GPT-5.4** (`openai/gpt-5.4`) | `nous_portal` | $2.50 | $15.00 | $0.2500 | 380 ms | 75.0 tok/s |
| **OpenAI: GPT-5.4** (`openai/gpt-5.4`) | `openrouter_market` | $2.50 | $15.00 | $0.2500 | 450 ms | 70.0 tok/s |
| **OpenAI: GPT-5.4 Mini** (`openai/gpt-5.4-mini`) | `nous_portal_batch` | $0.38 | $2.25 | $0.0375 | 5000 ms | 150.0 tok/s |
| **OpenAI: GPT-5.4 Mini** (`openai/gpt-5.4-mini`) | `openrouter_market` | $0.38 | $2.25 | $0.0375 | 450 ms | 130.0 tok/s |
| **OpenAI: GPT-5.4 Mini** (`openai/gpt-5.4-mini`) | `nous_portal` | $0.75 | $4.50 | $0.0750 | 220 ms | 130.0 tok/s |
| **OpenAI: GPT-5.4 Mini** (`openai/gpt-5.4-mini`) | `openrouter_market` | $0.75 | $4.50 | $0.0750 | 450 ms | 130.0 tok/s |
| **OpenAI: GPT-5.4 Nano** (`openai/gpt-5.4-nano`) | `nous_portal_batch` | $0.10 | $0.62 | $0.0100 | 5000 ms | 150.0 tok/s |
| **OpenAI: GPT-5.4 Nano** (`openai/gpt-5.4-nano`) | `openrouter_market` | $0.10 | $0.62 | $0.0100 | 450 ms | 70.0 tok/s |
| **OpenAI: GPT-5.4 Nano** (`openai/gpt-5.4-nano`) | `nous_portal` | $0.20 | $1.25 | $0.0200 | 380 ms | 75.0 tok/s |
| **OpenAI: GPT-5.4 Nano** (`openai/gpt-5.4-nano`) | `openrouter_market` | $0.20 | $1.25 | $0.0200 | 450 ms | 70.0 tok/s |
| **OpenAI: GPT-5.4 Pro** (`openai/gpt-5.4-pro`) | `nous_portal_batch` | $15.00 | $90.00 | $3.0000 | 5000 ms | 150.0 tok/s |
| **OpenAI: GPT-5.4 Pro** (`openai/gpt-5.4-pro`) | `openrouter_market` | $15.00 | $90.00 | $3.7500 | 450 ms | 70.0 tok/s |
| **OpenAI: GPT-5.4 Pro** (`openai/gpt-5.4-pro`) | `nous_portal` | $30.00 | $180.00 | $6.0000 | 380 ms | 75.0 tok/s |
| **OpenAI: GPT-5.4 Pro** (`openai/gpt-5.4-pro`) | `openrouter_market` | $30.00 | $180.00 | $7.5000 | 450 ms | 70.0 tok/s |
| **OpenAI: GPT-5.5** (`openai/gpt-5.5`) | `nous_portal_batch` | $2.50 | $15.00 | $0.2500 | 5000 ms | 150.0 tok/s |
| **OpenAI: GPT-5.5** (`openai/gpt-5.5`) | `openrouter_market` | $2.50 | $15.00 | $0.2500 | 450 ms | 70.0 tok/s |
| **OpenAI: GPT-5.5** (`openai/gpt-5.5`) | `nous_portal` | $5.00 | $30.00 | $0.5000 | 380 ms | 75.0 tok/s |
| **OpenAI: GPT-5.5** (`openai/gpt-5.5`) | `openrouter_market` | $5.00 | $30.00 | $0.5000 | 450 ms | 70.0 tok/s |
| **OpenAI: GPT-5.5 Pro** (`openai/gpt-5.5-pro`) | `nous_portal_batch` | $15.00 | $90.00 | $3.0000 | 5000 ms | 150.0 tok/s |
| **OpenAI: GPT-5.5 Pro** (`openai/gpt-5.5-pro`) | `openrouter_market` | $15.00 | $90.00 | $3.7500 | 450 ms | 70.0 tok/s |
| **OpenAI: GPT-5.5 Pro** (`openai/gpt-5.5-pro`) | `nous_portal` | $30.00 | $180.00 | $6.0000 | 380 ms | 75.0 tok/s |
| **OpenAI: GPT-5.5 Pro** (`openai/gpt-5.5-pro`) | `openrouter_market` | $30.00 | $180.00 | $7.5000 | 450 ms | 70.0 tok/s |
| **OpenAI: GPT-5.6 Luna** (`openai/gpt-5.6-luna`) | `nous_portal_batch` | $0.10 | $0.60 | $0.0100 | 5000 ms | 150.0 tok/s |
| **OpenAI: GPT-5.6 Luna** (`openai/gpt-5.6-luna`) | `openrouter_market` | $0.10 | $0.60 | $0.0100 | 450 ms | 130.0 tok/s |
| **OpenAI: GPT-5.6 Luna** (`openai/gpt-5.6-luna`) | `nous_portal` | $0.20 | $1.20 | $0.0200 | 220 ms | 130.0 tok/s |
| **OpenAI: GPT-5.6 Luna** (`openai/gpt-5.6-luna`) | `openrouter_market` | $0.20 | $1.20 | $0.0200 | 450 ms | 130.0 tok/s |
| **OpenAI: GPT-5.6 Luna Pro** (`openai/gpt-5.6-luna-pro`) | `nous_portal_batch` | $0.10 | $0.60 | $0.0100 | 5000 ms | 150.0 tok/s |
| **OpenAI: GPT-5.6 Luna Pro** (`openai/gpt-5.6-luna-pro`) | `openrouter_market` | $0.10 | $0.60 | $0.0100 | 450 ms | 130.0 tok/s |
| **OpenAI: GPT-5.6 Luna Pro** (`openai/gpt-5.6-luna-pro`) | `nous_portal` | $0.20 | $1.20 | $0.0200 | 220 ms | 130.0 tok/s |
| **OpenAI: GPT-5.6 Luna Pro** (`openai/gpt-5.6-luna-pro`) | `openrouter_market` | $0.20 | $1.20 | $0.0200 | 450 ms | 130.0 tok/s |
| **OpenAI: GPT-5.6 Sol** (`openai/gpt-5.6-sol`) | `nous_portal_batch` | $1.00 | $5.00 | $0.1000 | 5000 ms | 150.0 tok/s |
| **OpenAI: GPT-5.6 Sol** (`openai/gpt-5.6-sol`) | `openrouter_market` | $1.00 | $5.00 | $0.1000 | 450 ms | 95.0 tok/s |
| **OpenAI: GPT-5.6 Sol** (`openai/gpt-5.6-sol`) | `nous_portal` | $2.00 | $10.00 | $0.2000 | 380 ms | 90.0 tok/s |
| **OpenAI: GPT-5.6 Sol** (`openai/gpt-5.6-sol`) | `openrouter_market` | $2.00 | $10.00 | $0.2000 | 450 ms | 95.0 tok/s |
| **OpenAI: GPT-5.6 Sol Pro** (`openai/gpt-5.6-sol-pro`) | `nous_portal_batch` | $1.00 | $5.00 | $0.1000 | 5000 ms | 150.0 tok/s |
| **OpenAI: GPT-5.6 Sol Pro** (`openai/gpt-5.6-sol-pro`) | `openrouter_market` | $1.00 | $5.00 | $0.1000 | 450 ms | 95.0 tok/s |
| **OpenAI: GPT-5.6 Sol Pro** (`openai/gpt-5.6-sol-pro`) | `nous_portal` | $2.00 | $10.00 | $0.2000 | 380 ms | 90.0 tok/s |
| **OpenAI: GPT-5.6 Sol Pro** (`openai/gpt-5.6-sol-pro`) | `openrouter_market` | $2.00 | $10.00 | $0.2000 | 450 ms | 95.0 tok/s |
| **OpenAI: GPT-5.6 Terra** (`openai/gpt-5.6-terra`) | `nous_portal_batch` | $1.00 | $6.00 | $0.1000 | 5000 ms | 150.0 tok/s |
| **OpenAI: GPT-5.6 Terra** (`openai/gpt-5.6-terra`) | `openrouter_market` | $1.00 | $6.00 | $0.1000 | 450 ms | 70.0 tok/s |
| **OpenAI: GPT-5.6 Terra** (`openai/gpt-5.6-terra`) | `nous_portal` | $2.00 | $12.00 | $0.2000 | 380 ms | 75.0 tok/s |
| **OpenAI: GPT-5.6 Terra** (`openai/gpt-5.6-terra`) | `openrouter_market` | $2.00 | $12.00 | $0.2000 | 450 ms | 70.0 tok/s |
| **OpenAI: GPT-5.6 Terra Pro** (`openai/gpt-5.6-terra-pro`) | `nous_portal_batch` | $1.00 | $6.00 | $0.1000 | 5000 ms | 150.0 tok/s |
| **OpenAI: GPT-5.6 Terra Pro** (`openai/gpt-5.6-terra-pro`) | `openrouter_market` | $1.00 | $6.00 | $0.1000 | 450 ms | 70.0 tok/s |
| **OpenAI: GPT-5.6 Terra Pro** (`openai/gpt-5.6-terra-pro`) | `nous_portal` | $2.00 | $12.00 | $0.2000 | 380 ms | 75.0 tok/s |
| **OpenAI: GPT-5.6 Terra Pro** (`openai/gpt-5.6-terra-pro`) | `openrouter_market` | $2.00 | $12.00 | $0.2000 | 450 ms | 70.0 tok/s |
| **OpenAI: GPT-6 Astra** (`openai/gpt-6-astra`) | `nous_portal_batch` | $5.00 | $25.00 | $0.5000 | 5000 ms | 150.0 tok/s |
| **OpenAI: GPT-6 Astra** (`openai/gpt-6-astra`) | `openrouter_market` | $5.00 | $25.00 | $0.5000 | 450 ms | 45.0 tok/s |
| **OpenAI: GPT-6 Astra** (`openai/gpt-6-astra`) | `nous_portal` | $10.00 | $50.00 | $1.0000 | 550 ms | 40.0 tok/s |
| **OpenAI: GPT-6 Astra** (`openai/gpt-6-astra`) | `openrouter_market` | $10.00 | $50.00 | $1.0000 | 450 ms | 45.0 tok/s |
| **OpenAI: GPT-6 Astra Fast** (`openai/gpt-6-astra-fast`) | `nous_portal` | $20.00 | $100.00 | $2.0000 | 550 ms | 40.0 tok/s |
| **OpenAI: GPT-6 Astra Flex** (`openai/gpt-6-astra-flex`) | `nous_portal` | $5.00 | $25.00 | $0.5000 | 550 ms | 40.0 tok/s |
| **OpenAI: GPT-6 Astra Pro** (`openai/gpt-6-astra-pro`) | `nous_portal_batch` | $5.00 | $25.00 | $0.5000 | 5000 ms | 150.0 tok/s |
| **OpenAI: GPT-6 Astra Pro** (`openai/gpt-6-astra-pro`) | `openrouter_market` | $5.00 | $25.00 | $0.5000 | 450 ms | 45.0 tok/s |
| **OpenAI: GPT-6 Astra Pro** (`openai/gpt-6-astra-pro`) | `nous_portal` | $10.00 | $50.00 | $1.0000 | 550 ms | 40.0 tok/s |
| **OpenAI: GPT-6 Astra Pro** (`openai/gpt-6-astra-pro`) | `openrouter_market` | $10.00 | $50.00 | $1.0000 | 450 ms | 45.0 tok/s |
| **OpenAI: GPT-6 Astra Pro Fast** (`openai/gpt-6-astra-pro-fast`) | `nous_portal` | $20.00 | $100.00 | $2.0000 | 550 ms | 40.0 tok/s |
| **OpenAI: GPT-6 Astra Pro Flex** (`openai/gpt-6-astra-pro-flex`) | `nous_portal` | $5.00 | $25.00 | $0.5000 | 550 ms | 40.0 tok/s |
| **OpenAI: GPT-6 Luna** (`openai/gpt-6-luna`) | `nous_portal_batch` | $0.05 | $0.25 | $0.0050 | 5000 ms | 150.0 tok/s |
| **OpenAI: GPT-6 Luna** (`openai/gpt-6-luna`) | `openrouter_market` | $0.05 | $0.25 | $0.0050 | 450 ms | 130.0 tok/s |
| **OpenAI: GPT-6 Luna** (`openai/gpt-6-luna`) | `nous_portal` | $0.10 | $0.50 | $0.0100 | 220 ms | 130.0 tok/s |
| **OpenAI: GPT-6 Luna** (`openai/gpt-6-luna`) | `openrouter_market` | $0.10 | $0.50 | $0.0100 | 450 ms | 130.0 tok/s |
| **OpenAI: GPT-6 Luna Pro** (`openai/gpt-6-luna-pro`) | `nous_portal_batch` | $0.05 | $0.25 | $0.0050 | 5000 ms | 150.0 tok/s |
| **OpenAI: GPT-6 Luna Pro** (`openai/gpt-6-luna-pro`) | `openrouter_market` | $0.05 | $0.25 | $0.0050 | 450 ms | 130.0 tok/s |
| **OpenAI: GPT-6 Luna Pro** (`openai/gpt-6-luna-pro`) | `nous_portal` | $0.10 | $0.50 | $0.0100 | 220 ms | 130.0 tok/s |
| **OpenAI: GPT-6 Luna Pro** (`openai/gpt-6-luna-pro`) | `openrouter_market` | $0.10 | $0.50 | $0.0100 | 450 ms | 130.0 tok/s |
| **OpenAI: GPT-6 Sol** (`openai/gpt-6-sol`) | `nous_portal_batch` | $1.00 | $5.00 | $0.1000 | 5000 ms | 150.0 tok/s |
| **OpenAI: GPT-6 Sol** (`openai/gpt-6-sol`) | `openrouter_market` | $1.00 | $5.00 | $0.1000 | 450 ms | 95.0 tok/s |
| **OpenAI: GPT-6 Sol** (`openai/gpt-6-sol`) | `nous_portal` | $2.00 | $10.00 | $0.2000 | 380 ms | 90.0 tok/s |
| **OpenAI: GPT-6 Sol** (`openai/gpt-6-sol`) | `openrouter_market` | $2.00 | $10.00 | $0.2000 | 450 ms | 95.0 tok/s |
| **OpenAI: GPT-6 Sol Pro** (`openai/gpt-6-sol-pro`) | `nous_portal_batch` | $1.00 | $5.00 | $0.1000 | 5000 ms | 150.0 tok/s |
| **OpenAI: GPT-6 Sol Pro** (`openai/gpt-6-sol-pro`) | `openrouter_market` | $1.00 | $5.00 | $0.1000 | 450 ms | 95.0 tok/s |
| **OpenAI: GPT-6 Sol Pro** (`openai/gpt-6-sol-pro`) | `nous_portal` | $2.00 | $10.00 | $0.2000 | 380 ms | 90.0 tok/s |
| **OpenAI: GPT-6 Sol Pro** (`openai/gpt-6-sol-pro`) | `openrouter_market` | $2.00 | $10.00 | $0.2000 | 450 ms | 95.0 tok/s |
| **OpenAI: GPT-6.1 Sol** (`openai/gpt-6.1-sol`) | `nous_portal_batch` | $1.00 | $5.00 | $0.0500 | 5000 ms | 150.0 tok/s |
| **OpenAI: GPT-6.1 Sol** (`openai/gpt-6.1-sol`) | `openrouter_market` | $1.00 | $5.00 | $0.0500 | 450 ms | 95.0 tok/s |
| **OpenAI: GPT-6.1 Sol** (`openai/gpt-6.1-sol`) | `nous_portal` | $2.00 | $10.00 | $0.1000 | 380 ms | 90.0 tok/s |
| **OpenAI: GPT-6.1 Sol** (`openai/gpt-6.1-sol`) | `openrouter_market` | $2.00 | $10.00 | $0.1000 | 450 ms | 95.0 tok/s |
| **OpenAI: GPT-6.1 Sol Pro** (`openai/gpt-6.1-sol-pro`) | `nous_portal_batch` | $1.00 | $5.00 | $0.0500 | 5000 ms | 150.0 tok/s |
| **OpenAI: GPT-6.1 Sol Pro** (`openai/gpt-6.1-sol-pro`) | `openrouter_market` | $1.00 | $5.00 | $0.0500 | 450 ms | 95.0 tok/s |
| **OpenAI: GPT-6.1 Sol Pro** (`openai/gpt-6.1-sol-pro`) | `nous_portal` | $2.00 | $10.00 | $0.1000 | 380 ms | 90.0 tok/s |
| **OpenAI: GPT-6.1 Sol Pro** (`openai/gpt-6.1-sol-pro`) | `openrouter_market` | $2.00 | $10.00 | $0.1000 | 450 ms | 95.0 tok/s |
| **OpenAI: GPT Astra Latest** (`openai/gpt-astra-latest`) | `nous_portal` | $10.00 | $50.00 | $1.0000 | 550 ms | 40.0 tok/s |
| **OpenAI: GPT Astra Latest** (`openai/gpt-astra-latest`) | `openrouter_market` | $10.00 | $50.00 | $1.0000 | 450 ms | 45.0 tok/s |
| **OpenAI: GPT Audio** (`openai/gpt-audio`) | `nous_portal` | $2.50 | $10.00 | $0.5000 | 380 ms | 75.0 tok/s |
| **OpenAI: GPT Audio** (`openai/gpt-audio`) | `openrouter_market` | $2.50 | $10.00 | $0.6250 | 450 ms | 70.0 tok/s |
| **OpenAI: GPT Audio Mini** (`openai/gpt-audio-mini`) | `nous_portal` | $0.60 | $2.40 | $0.1200 | 220 ms | 130.0 tok/s |
| **OpenAI: GPT Audio Mini** (`openai/gpt-audio-mini`) | `openrouter_market` | $0.60 | $2.40 | $0.1500 | 450 ms | 130.0 tok/s |
| **OpenAI: GPT Chat Latest** (`openai/gpt-chat-latest`) | `nous_portal` | $5.00 | $30.00 | $0.5000 | 380 ms | 75.0 tok/s |
| **OpenAI: GPT Chat Latest** (`openai/gpt-chat-latest`) | `openrouter_market` | $5.00 | $30.00 | $0.5000 | 450 ms | 70.0 tok/s |
| **OpenAI: GPT Luna Latest** (`openai/gpt-luna-latest`) | `nous_portal` | $0.10 | $0.50 | $0.0100 | 220 ms | 130.0 tok/s |
| **OpenAI: GPT Luna Latest** (`openai/gpt-luna-latest`) | `openrouter_market` | $0.10 | $0.50 | $0.0100 | 450 ms | 130.0 tok/s |
| **OpenAI: GPT Mini Latest** (`openai/gpt-mini-latest`) | `nous_portal` | $0.75 | $4.50 | $0.0750 | 220 ms | 130.0 tok/s |
| **OpenAI: GPT Mini Latest** (`openai/gpt-mini-latest`) | `openrouter_market` | $0.75 | $4.50 | $0.0750 | 450 ms | 130.0 tok/s |
| **OpenAI: gpt-oss-120b** (`openai/gpt-oss-120b`) | `nous_portal_batch` | $0.03 | $0.14 | $0.0059 | 5000 ms | 150.0 tok/s |
| **OpenAI: gpt-oss-120b** (`openai/gpt-oss-120b`) | `openrouter_market` | $0.03 | $0.14 | $0.0074 | 450 ms | 70.0 tok/s |
| **OpenAI: gpt-oss-120b** (`openai/gpt-oss-120b`) | `nous_portal` | $0.03 | $0.15 | $0.0300 | 380 ms | 75.0 tok/s |
| **OpenAI: gpt-oss-120b** (`openai/gpt-oss-120b`) | `openrouter_market` | $0.04 | $0.17 | $0.0092 | 450 ms | 70.0 tok/s |
| **OpenAI: gpt-oss-20b** (`openai/gpt-oss-20b`) | `nous_portal` | $0.02 | $0.09 | $0.0090 | 380 ms | 75.0 tok/s |
| **OpenAI: gpt-oss-20b** (`openai/gpt-oss-20b`) | `openrouter_market` | $0.02 | $0.09 | $0.0090 | 450 ms | 70.0 tok/s |
| **OpenAI: gpt-oss-20b** (`openai/gpt-oss-20b`) | `nous_portal_batch` | $0.02 | $0.11 | $0.0048 | 5000 ms | 150.0 tok/s |
| **OpenAI: gpt-oss-20b** (`openai/gpt-oss-20b`) | `openrouter_market` | $0.02 | $0.11 | $0.0060 | 450 ms | 70.0 tok/s |
| **OpenAI: gpt-oss-safeguard-20b** (`openai/gpt-oss-safeguard-20b`) | `nous_portal` | $0.07 | $0.30 | $0.0375 | 380 ms | 75.0 tok/s |
| **OpenAI: gpt-oss-safeguard-20b** (`openai/gpt-oss-safeguard-20b`) | `openrouter_market` | $0.07 | $0.30 | $0.0375 | 450 ms | 70.0 tok/s |
| **OpenAI: GPT Sol Latest** (`openai/gpt-sol-latest`) | `nous_portal` | $2.00 | $10.00 | $0.1000 | 380 ms | 90.0 tok/s |
| **OpenAI: GPT Sol Latest** (`openai/gpt-sol-latest`) | `openrouter_market` | $2.00 | $10.00 | $0.1000 | 450 ms | 95.0 tok/s |
| **OpenAI: GPT Terra Latest** (`openai/gpt-terra-latest`) | `nous_portal` | $2.00 | $12.00 | $0.2000 | 380 ms | 75.0 tok/s |
| **OpenAI: GPT Terra Latest** (`openai/gpt-terra-latest`) | `openrouter_market` | $2.00 | $12.00 | $0.2000 | 450 ms | 70.0 tok/s |
| **OpenAI: o1** (`openai/o1`) | `nous_portal` | $15.00 | $60.00 | $7.5000 | 380 ms | 75.0 tok/s |
| **OpenAI: o1** (`openai/o1`) | `openrouter_market` | $15.00 | $60.00 | $7.5000 | 450 ms | 70.0 tok/s |
| **OpenAI: o3** (`openai/o3`) | `nous_portal_batch` | $1.00 | $4.00 | $0.2500 | 5000 ms | 150.0 tok/s |
| **OpenAI: o3** (`openai/o3`) | `openrouter_market` | $1.00 | $4.00 | $0.2500 | 450 ms | 70.0 tok/s |
| **OpenAI: o3** (`openai/o3`) | `nous_portal` | $2.00 | $8.00 | $0.5000 | 380 ms | 75.0 tok/s |
| **OpenAI: o3** (`openai/o3`) | `openrouter_market` | $2.00 | $8.00 | $0.5000 | 450 ms | 70.0 tok/s |
| **OpenAI: o3 Mini** (`openai/o3-mini`) | `nous_portal_batch` | $0.55 | $2.20 | $0.2750 | 5000 ms | 150.0 tok/s |
| **OpenAI: o3 Mini** (`openai/o3-mini`) | `openrouter_market` | $0.55 | $2.20 | $0.2750 | 450 ms | 130.0 tok/s |
| **OpenAI: o3 Mini** (`openai/o3-mini`) | `nous_portal` | $1.10 | $4.40 | $0.5500 | 220 ms | 130.0 tok/s |
| **OpenAI: o3 Mini** (`openai/o3-mini`) | `openrouter_market` | $1.10 | $4.40 | $0.5500 | 450 ms | 130.0 tok/s |
| **OpenAI: o3 Mini High** (`openai/o3-mini-high`) | `nous_portal` | $1.10 | $4.40 | $0.5500 | 220 ms | 130.0 tok/s |
| **OpenAI: o3 Mini High** (`openai/o3-mini-high`) | `openrouter_market` | $1.10 | $4.40 | $0.5500 | 450 ms | 130.0 tok/s |
| **OpenAI: o3 Pro** (`openai/o3-pro`) | `nous_portal` | $20.00 | $80.00 | $4.0000 | 380 ms | 75.0 tok/s |
| **OpenAI: o3 Pro** (`openai/o3-pro`) | `openrouter_market` | $20.00 | $80.00 | $5.0000 | 450 ms | 70.0 tok/s |
| **OpenAI: o4 Mini** (`openai/o4-mini`) | `nous_portal_batch` | $0.55 | $2.20 | $0.1375 | 5000 ms | 150.0 tok/s |
| **OpenAI: o4 Mini** (`openai/o4-mini`) | `openrouter_market` | $0.55 | $2.20 | $0.1375 | 450 ms | 130.0 tok/s |
| **OpenAI: o4 Mini** (`openai/o4-mini`) | `nous_portal` | $1.10 | $4.40 | $0.2750 | 220 ms | 130.0 tok/s |
| **OpenAI: o4 Mini** (`openai/o4-mini`) | `openrouter_market` | $1.10 | $4.40 | $0.2750 | 450 ms | 130.0 tok/s |
| **OpenAI: o4 Mini High** (`openai/o4-mini-high`) | `nous_portal` | $1.10 | $4.40 | $0.2750 | 220 ms | 130.0 tok/s |
| **OpenAI: o4 Mini High** (`openai/o4-mini-high`) | `openrouter_market` | $1.10 | $4.40 | $0.2750 | 450 ms | 130.0 tok/s |
| **OpenAI: Text Embedding 3 Large** (`openai/text-embedding-3-large`) | `nous_portal_batch` | $0.07 | $0.00 | $0.0130 | 5000 ms | 150.0 tok/s |
| **OpenAI: Text Embedding 3 Large** (`openai/text-embedding-3-large`) | `nous_portal` | $0.13 | $0.00 | $0.0260 | 380 ms | 75.0 tok/s |
| **OpenAI: Text Embedding 3 Small** (`openai/text-embedding-3-small`) | `nous_portal` | $0.02 | $0.00 | $0.0040 | 380 ms | 75.0 tok/s |
| **OpenAI: Text Embedding 3 Small** (`openai/text-embedding-3-small`) | `nous_portal_batch` | $0.02 | $0.00 | $0.0040 | 5000 ms | 150.0 tok/s |
| **OpenAI: Text Embedding Ada 002** (`openai/text-embedding-ada-002`) | `nous_portal_batch` | $0.05 | $0.00 | $0.0100 | 5000 ms | 150.0 tok/s |
| **OpenAI: Text Embedding Ada 002** (`openai/text-embedding-ada-002`) | `nous_portal` | $0.10 | $0.00 | $0.0200 | 380 ms | 75.0 tok/s |
| **Perceptron: Perceptron Mk1.5** (`perceptron/perceptron-mk1.5`) | `nous_portal` | $0.15 | $1.50 | $0.0300 | 380 ms | 75.0 tok/s |
| **Perceptron: Perceptron Mk1.5** (`perceptron/perceptron-mk1.5`) | `openrouter_market` | $0.15 | $1.50 | $0.0375 | 450 ms | 70.0 tok/s |
| **Perplexity: Embed V1 0.6B** (`perplexity/pplx-embed-v1-0.6b`) | `nous_portal` | $0.00 | $0.00 | $0.0008 | 380 ms | 75.0 tok/s |
| **Perplexity: Embed V1 4B** (`perplexity/pplx-embed-v1-4b`) | `nous_portal` | $0.03 | $0.00 | $0.0060 | 380 ms | 75.0 tok/s |
| **Poolside: Laguna S 2.1** (`poolside/laguna-s-2.1`) | `openrouter_market` | $0.00 | $0.00 | N/A | 450 ms | 70.0 tok/s |
| **Poolside: Laguna S 2.1** (`poolside/laguna-s-2.1`) | `nous_portal` | $0.09 | $0.18 | $0.0090 | 380 ms | 75.0 tok/s |
| **Poolside: Laguna S 2.1** (`poolside/laguna-s-2.1`) | `openrouter_market` | $0.09 | $0.18 | $0.0090 | 450 ms | 70.0 tok/s |
| **Poolside: Laguna XS 2.1** (`poolside/laguna-xs-2.1`) | `openrouter_market` | $0.00 | $0.00 | N/A | 450 ms | 70.0 tok/s |
| **Poolside: Laguna XS 2.1** (`poolside/laguna-xs-2.1`) | `nous_portal` | $0.06 | $0.12 | $0.0300 | 380 ms | 75.0 tok/s |
| **Poolside: Laguna XS 2.1** (`poolside/laguna-xs-2.1`) | `openrouter_market` | $0.06 | $0.12 | $0.0300 | 450 ms | 70.0 tok/s |
| **PrismML: Ternary Bonsai 2 27B** (`prism-ml/ternary-bonsai-2-27b`) | `nous_portal` | $0.07 | $0.50 | $0.0375 | 380 ms | 75.0 tok/s |
| **PrismML: Ternary Bonsai 2 27B** (`prism-ml/ternary-bonsai-2-27b`) | `openrouter_market` | $0.07 | $0.50 | $0.0375 | 450 ms | 70.0 tok/s |
| **Qwen 2.5 Coder 32B** (`qwen-2.5-coder-32b`) | `nous_portal` | $0.20 | $0.60 | $0.0500 | 290 ms | 105.0 tok/s |
| **Qwen 2.5 Coder 32B** (`qwen-2.5-coder-32b`) | `nous_portal` | $0.50 | $2.00 | $0.1000 | 380 ms | 90.0 tok/s |
| **Qwen 2.5 Coder 32B** (`qwen-2.5-coder-32b`) | `nous_portal` | $0.50 | $2.00 | $0.1000 | 380 ms | 90.0 tok/s |
| **Qwen 2.5 Coder 32B** (`qwen-2.5-coder-32b`) | `openrouter` | $0.66 | $1.00 | $0.1650 | 450 ms | 95.0 tok/s |
| **Qwen 2.5 Coder 32B** (`qwen-2.5-coder-32b`) | `openrouter_market` | $0.66 | $1.00 | $0.1650 | 450 ms | 95.0 tok/s |
| **Qwen 2.5 Coder 32B** (`qwen-2.5-coder-32b`) | `openrouter_market` | $0.66 | $1.00 | $0.1650 | 450 ms | 95.0 tok/s |
| **Qwen 2.5 Coder 32B** (`qwen-2.5-coder-32b`) | `openrouter_market` | $0.66 | $1.00 | $0.1650 | 450 ms | 95.0 tok/s |
| **Qwen2.5 72B Instruct** (`qwen/qwen-2.5-72b-instruct`) | `nous_portal` | $0.36 | $0.40 | $0.0720 | 380 ms | 75.0 tok/s |
| **Qwen2.5 72B Instruct** (`qwen/qwen-2.5-72b-instruct`) | `openrouter_market` | $0.36 | $0.40 | $0.0900 | 450 ms | 70.0 tok/s |
| **Qwen: Qwen2.5 7B Instruct** (`qwen/qwen-2.5-7b-instruct`) | `nous_portal` | $0.10 | $0.20 | $0.0200 | 380 ms | 75.0 tok/s |
| **Qwen: Qwen2.5 7B Instruct** (`qwen/qwen-2.5-7b-instruct`) | `openrouter_market` | $0.10 | $0.20 | $0.0250 | 450 ms | 70.0 tok/s |
| **Qwen: Qwen-Plus** (`qwen/qwen-plus`) | `nous_portal` | $0.26 | $0.78 | $0.0520 | 380 ms | 75.0 tok/s |
| **Qwen: Qwen-Plus** (`qwen/qwen-plus`) | `openrouter_market` | $0.26 | $0.78 | $0.0520 | 450 ms | 70.0 tok/s |
| **Qwen: Qwen3 14B** (`qwen/qwen3-14b`) | `nous_portal` | $0.10 | $0.22 | $0.0200 | 380 ms | 75.0 tok/s |
| **Qwen: Qwen3 14B** (`qwen/qwen3-14b`) | `openrouter_market` | $0.12 | $0.24 | $0.0300 | 450 ms | 70.0 tok/s |
| **Qwen: Qwen3 235B A22B Instruct 2507** (`qwen/qwen3-235b-a22b-2507`) | `nous_portal` | $0.09 | $0.35 | $0.0175 | 380 ms | 75.0 tok/s |
| **Qwen: Qwen3 235B A22B Instruct 2507** (`qwen/qwen3-235b-a22b-2507`) | `openrouter_market` | $0.09 | $0.55 | $0.0225 | 450 ms | 70.0 tok/s |
| **Qwen: Qwen3 235B A22B Thinking 2507** (`qwen/qwen3-235b-a22b-thinking-2507`) | `nous_portal` | $0.45 | $3.50 | $0.0900 | 380 ms | 75.0 tok/s |
| **Qwen: Qwen3 235B A22B Thinking 2507** (`qwen/qwen3-235b-a22b-thinking-2507`) | `openrouter_market` | $0.45 | $3.50 | $0.1125 | 450 ms | 70.0 tok/s |
| **Qwen: Qwen3 30B A3B** (`qwen/qwen3-30b-a3b`) | `nous_portal` | $0.12 | $0.50 | $0.0240 | 380 ms | 75.0 tok/s |
| **Qwen: Qwen3 30B A3B** (`qwen/qwen3-30b-a3b`) | `openrouter_market` | $0.12 | $0.50 | $0.0300 | 450 ms | 70.0 tok/s |
| **Qwen: Qwen3 30B A3B Instruct 2507** (`qwen/qwen3-30b-a3b-instruct-2507`) | `nous_portal` | $0.09 | $0.30 | $0.0180 | 380 ms | 75.0 tok/s |
| **Qwen: Qwen3 30B A3B Instruct 2507** (`qwen/qwen3-30b-a3b-instruct-2507`) | `openrouter_market` | $0.10 | $0.30 | $0.0250 | 450 ms | 70.0 tok/s |
| **Qwen: Qwen3 32B** (`qwen/qwen3-32b`) | `nous_portal` | $0.08 | $0.28 | $0.0160 | 380 ms | 75.0 tok/s |
| **Qwen: Qwen3 32B** (`qwen/qwen3-32b`) | `openrouter_market` | $0.08 | $0.28 | $0.0200 | 450 ms | 70.0 tok/s |
| **Qwen: Qwen3 Coder 480B A35B** (`qwen/qwen3-coder`) | `nous_portal` | $0.22 | $1.80 | $0.0440 | 380 ms | 90.0 tok/s |
| **Qwen: Qwen3 Coder 480B A35B** (`qwen/qwen3-coder`) | `openrouter_market` | $0.30 | $1.00 | $0.1000 | 450 ms | 70.0 tok/s |
| **Qwen: Qwen3 Coder 30B A3B Instruct** (`qwen/qwen3-coder-30b-a3b-instruct`) | `nous_portal` | $0.07 | $0.28 | $0.0140 | 380 ms | 90.0 tok/s |
| **Qwen: Qwen3 Coder 30B A3B Instruct** (`qwen/qwen3-coder-30b-a3b-instruct`) | `openrouter_market` | $0.07 | $0.28 | $0.0175 | 450 ms | 70.0 tok/s |
| **Qwen: Qwen3 Coder Flash** (`qwen/qwen3-coder-flash`) | `nous_portal` | $0.20 | $0.97 | $0.0390 | 220 ms | 130.0 tok/s |
| **Qwen: Qwen3 Coder Flash** (`qwen/qwen3-coder-flash`) | `openrouter_market` | $0.20 | $0.97 | $0.0390 | 450 ms | 130.0 tok/s |
| **Qwen: Qwen3 Coder Next** (`qwen/qwen3-coder-next`) | `nous_portal` | $0.12 | $0.80 | $0.0700 | 380 ms | 90.0 tok/s |
| **Qwen: Qwen3 Coder Next** (`qwen/qwen3-coder-next`) | `openrouter_market` | $0.12 | $0.80 | $0.0700 | 450 ms | 70.0 tok/s |
| **Qwen: Qwen3 Embedding 4B** (`qwen/qwen3-embedding-4b`) | `nous_portal` | $0.02 | $0.00 | $0.0040 | 380 ms | 75.0 tok/s |
| **Qwen: Qwen3 Embedding 8B** (`qwen/qwen3-embedding-8b`) | `nous_portal` | $0.01 | $0.00 | $0.0020 | 380 ms | 75.0 tok/s |
| **Qwen: Qwen3 Next 80B A3B Instruct** (`qwen/qwen3-next-80b-a3b-instruct`) | `nous_portal` | $0.09 | $1.10 | $0.0180 | 380 ms | 75.0 tok/s |
| **Qwen: Qwen3 Next 80B A3B Instruct** (`qwen/qwen3-next-80b-a3b-instruct`) | `openrouter_market` | $0.10 | $1.10 | $0.0700 | 450 ms | 70.0 tok/s |
| **Qwen: Qwen3 Next 80B A3B Thinking** (`qwen/qwen3-next-80b-a3b-thinking`) | `nous_portal` | $0.15 | $1.20 | $0.0300 | 380 ms | 75.0 tok/s |
| **Qwen: Qwen3 Next 80B A3B Thinking** (`qwen/qwen3-next-80b-a3b-thinking`) | `openrouter_market` | $0.15 | $1.20 | $0.0375 | 450 ms | 70.0 tok/s |
| **Qwen: Qwen3 VL 235B A22B Instruct** (`qwen/qwen3-vl-235b-a22b-instruct`) | `nous_portal` | $0.20 | $0.88 | $0.1100 | 380 ms | 75.0 tok/s |
| **Qwen: Qwen3 VL 235B A22B Instruct** (`qwen/qwen3-vl-235b-a22b-instruct`) | `openrouter_market` | $0.21 | $1.90 | $0.1000 | 450 ms | 70.0 tok/s |
| **Qwen: Qwen3 VL 30B A3B Instruct** (`qwen/qwen3-vl-30b-a3b-instruct`) | `nous_portal` | $0.15 | $0.60 | $0.0300 | 380 ms | 75.0 tok/s |
| **Qwen: Qwen3 VL 30B A3B Instruct** (`qwen/qwen3-vl-30b-a3b-instruct`) | `openrouter_market` | $0.15 | $0.60 | $0.0375 | 450 ms | 70.0 tok/s |
| **Qwen: Qwen3 VL 30B A3B Thinking** (`qwen/qwen3-vl-30b-a3b-thinking`) | `nous_portal` | $0.29 | $1.00 | $0.0580 | 380 ms | 75.0 tok/s |
| **Qwen: Qwen3 VL 30B A3B Thinking** (`qwen/qwen3-vl-30b-a3b-thinking`) | `openrouter_market` | $0.29 | $1.00 | $0.0725 | 450 ms | 70.0 tok/s |
| **Qwen: Qwen3 VL 8B Instruct** (`qwen/qwen3-vl-8b-instruct`) | `nous_portal` | $0.25 | $0.75 | $0.1200 | 380 ms | 75.0 tok/s |
| **Qwen: Qwen3 VL 8B Instruct** (`qwen/qwen3-vl-8b-instruct`) | `openrouter_market` | $0.25 | $0.75 | $0.1200 | 450 ms | 70.0 tok/s |
| **Qwen: Qwen3.5-122B-A10B** (`qwen/qwen3.5-122b-a10b`) | `nous_portal` | $0.26 | $2.08 | $0.0520 | 380 ms | 75.0 tok/s |
| **Qwen: Qwen3.5-122B-A10B** (`qwen/qwen3.5-122b-a10b`) | `openrouter_market` | $0.26 | $2.08 | $0.0650 | 450 ms | 70.0 tok/s |
| **Qwen: Qwen3.5-27B** (`qwen/qwen3.5-27b`) | `nous_portal` | $0.20 | $1.56 | $0.0390 | 380 ms | 75.0 tok/s |
| **Qwen: Qwen3.5-27B** (`qwen/qwen3.5-27b`) | `openrouter_market` | $0.26 | $2.60 | $0.0650 | 450 ms | 70.0 tok/s |
| **Qwen: Qwen3.5-35B-A3B** (`qwen/qwen3.5-35b-a3b`) | `nous_portal` | $0.08 | $0.75 | $0.0400 | 380 ms | 75.0 tok/s |
| **Qwen: Qwen3.5-35B-A3B** (`qwen/qwen3.5-35b-a3b`) | `openrouter_market` | $0.15 | $1.00 | $0.0500 | 450 ms | 70.0 tok/s |
| **Qwen: Qwen3.5 397B A17B** (`qwen/qwen3.5-397b-a17b`) | `nous_portal` | $0.39 | $2.34 | $0.0780 | 380 ms | 75.0 tok/s |
| **Qwen: Qwen3.5 397B A17B** (`qwen/qwen3.5-397b-a17b`) | `openrouter_market` | $0.50 | $3.60 | $0.3000 | 450 ms | 70.0 tok/s |
| **Qwen: Qwen3.5-9B** (`qwen/qwen3.5-9b`) | `nous_portal` | $0.08 | $0.13 | $0.0400 | 380 ms | 75.0 tok/s |
| **Qwen: Qwen3.5-9B** (`qwen/qwen3.5-9b`) | `openrouter_market` | $0.10 | $0.15 | $0.0250 | 450 ms | 70.0 tok/s |
| **Qwen: Qwen3.5-Flash** (`qwen/qwen3.5-flash-02-23`) | `nous_portal` | $0.07 | $0.26 | $0.0130 | 220 ms | 130.0 tok/s |
| **Qwen: Qwen3.5-Flash** (`qwen/qwen3.5-flash-02-23`) | `openrouter_market` | $0.07 | $0.26 | $0.0163 | 450 ms | 130.0 tok/s |
| **Qwen: Qwen3.5 Plus 2026-02-15** (`qwen/qwen3.5-plus-02-15`) | `nous_portal` | $0.26 | $1.56 | $0.0520 | 380 ms | 75.0 tok/s |
| **Qwen: Qwen3.5 Plus 2026-02-15** (`qwen/qwen3.5-plus-02-15`) | `openrouter_market` | $0.26 | $1.56 | $0.0650 | 450 ms | 70.0 tok/s |
| **Qwen: Qwen3.5 Plus 2026-04-20** (`qwen/qwen3.5-plus-20260420`) | `nous_portal` | $0.30 | $1.80 | $0.0600 | 380 ms | 75.0 tok/s |
| **Qwen: Qwen3.5 Plus 2026-04-20** (`qwen/qwen3.5-plus-20260420`) | `openrouter_market` | $0.30 | $1.80 | $0.0750 | 450 ms | 70.0 tok/s |
| **Qwen: Qwen3.6 27B** (`qwen/qwen3.6-27b`) | `nous_portal` | $0.25 | $2.00 | $0.0250 | 380 ms | 75.0 tok/s |
| **Qwen: Qwen3.6 27B** (`qwen/qwen3.6-27b`) | `openrouter_market` | $0.32 | $3.20 | $0.0800 | 450 ms | 70.0 tok/s |
| **Qwen: Qwen3.6 35B A3B** (`qwen/qwen3.6-35b-a3b`) | `nous_portal` | $0.05 | $0.70 | $0.0250 | 380 ms | 75.0 tok/s |
| **Qwen: Qwen3.6 35B A3B** (`qwen/qwen3.6-35b-a3b`) | `openrouter_market` | $0.10 | $0.95 | $0.1000 | 450 ms | 70.0 tok/s |
| **Qwen: Qwen3.6 Flash** (`qwen/qwen3.6-flash`) | `nous_portal` | $0.19 | $1.12 | $0.0375 | 220 ms | 130.0 tok/s |
| **Qwen: Qwen3.6 Flash** (`qwen/qwen3.6-flash`) | `openrouter_market` | $0.19 | $1.12 | $0.0469 | 450 ms | 130.0 tok/s |
| **Qwen: Qwen3.6 Plus** (`qwen/qwen3.6-plus`) | `nous_portal` | $0.33 | $1.95 | $0.0650 | 380 ms | 75.0 tok/s |
| **Qwen: Qwen3.6 Plus** (`qwen/qwen3.6-plus`) | `openrouter_market` | $0.33 | $1.95 | $0.0813 | 450 ms | 70.0 tok/s |
| **Qwen: Qwen3.7 Flash** (`qwen/qwen3.7-flash`) | `nous_portal` | $0.03 | $0.13 | $0.0060 | 220 ms | 130.0 tok/s |
| **Qwen: Qwen3.7 Flash** (`qwen/qwen3.7-flash`) | `openrouter_market` | $0.03 | $0.13 | $0.0060 | 450 ms | 130.0 tok/s |
| **Qwen: Qwen3.7 Max** (`qwen/qwen3.7-max`) | `nous_portal` | $1.48 | $4.42 | $0.2950 | 380 ms | 75.0 tok/s |
| **Qwen: Qwen3.7 Max** (`qwen/qwen3.7-max`) | `openrouter_market` | $1.48 | $4.42 | $0.2950 | 450 ms | 70.0 tok/s |
| **Qwen: Qwen3.7 Plus** (`qwen/qwen3.7-plus`) | `nous_portal` | $0.32 | $1.28 | $0.0640 | 380 ms | 75.0 tok/s |
| **Qwen: Qwen3.7 Plus** (`qwen/qwen3.7-plus`) | `openrouter_market` | $0.32 | $1.28 | $0.0640 | 450 ms | 70.0 tok/s |
| **Qwen: Qwen3.8 2.4T A95B** (`qwen/qwen3.8-2.4t-a95b`) | `nous_portal` | $2.00 | $6.00 | $0.2500 | 380 ms | 75.0 tok/s |
| **Qwen: Qwen3.8 2.4T A95B** (`qwen/qwen3.8-2.4t-a95b`) | `openrouter_market` | $2.00 | $6.00 | $0.2500 | 450 ms | 70.0 tok/s |
| **Qwen: Qwen3.8 27B** (`qwen/qwen3.8-27b`) | `nous_portal` | $0.04 | $1.25 | $0.0039 | 380 ms | 75.0 tok/s |
| **Qwen: Qwen3.8 27B** (`qwen/qwen3.8-27b`) | `openrouter_market` | $0.04 | $2.30 | $0.0198 | 450 ms | 70.0 tok/s |
| **Qwen: Qwen3.8 Flash** (`qwen/qwen3.8-flash`) | `nous_portal` | $0.15 | $0.47 | $0.0160 | 220 ms | 130.0 tok/s |
| **Qwen: Qwen3.8 Flash** (`qwen/qwen3.8-flash`) | `openrouter_market` | $0.15 | $0.47 | $0.0160 | 450 ms | 130.0 tok/s |
| **Qwen: Qwen3.8 Max (0902)** (`qwen/qwen3.8-max-0902`) | `nous_portal` | $2.00 | $6.00 | $0.2500 | 380 ms | 75.0 tok/s |
| **Qwen: Qwen3.8 Max (0902)** (`qwen/qwen3.8-max-0902`) | `openrouter_market` | $2.00 | $6.00 | $0.2500 | 450 ms | 70.0 tok/s |
| **Qwen: Qwen3.8 Max Prime** (`qwen/qwen3.8-max-prime`) | `nous_portal` | $4.00 | $12.00 | $0.5000 | 380 ms | 75.0 tok/s |
| **Qwen: Qwen3.8 Max Prime** (`qwen/qwen3.8-max-prime`) | `openrouter_market` | $4.00 | $12.00 | $0.5000 | 450 ms | 70.0 tok/s |
| **Qwen: Qwen3.8 Omni Flash** (`qwen/qwen3.8-omni-flash`) | `nous_portal` | $0.15 | $0.47 | $0.0160 | 220 ms | 130.0 tok/s |
| **Qwen: Qwen3.8 Omni Flash** (`qwen/qwen3.8-omni-flash`) | `openrouter_market` | $0.15 | $0.47 | $0.0160 | 450 ms | 130.0 tok/s |
| **Reka Edge** (`rekaai/reka-edge`) | `nous_portal` | $0.10 | $0.10 | $0.0200 | 380 ms | 75.0 tok/s |
| **Reka Edge** (`rekaai/reka-edge`) | `openrouter_market` | $0.10 | $0.10 | $0.0250 | 450 ms | 70.0 tok/s |
| **Relace: Relace Search** (`relace/relace-search`) | `nous_portal` | $1.00 | $3.00 | $0.2000 | 380 ms | 75.0 tok/s |
| **Relace: Relace Search** (`relace/relace-search`) | `openrouter_market` | $1.00 | $3.00 | $0.2500 | 450 ms | 70.0 tok/s |
| **Sakana: Fugu Max** (`sakana/fugu-max`) | `nous_portal` | $2.00 | $6.00 | $0.2500 | 380 ms | 75.0 tok/s |
| **Sakana: Fugu Max** (`sakana/fugu-max`) | `openrouter_market` | $2.00 | $6.00 | $0.2500 | 450 ms | 70.0 tok/s |
| **Sakana: Fugu Ultra** (`sakana/fugu-ultra`) | `nous_portal` | $5.00 | $30.00 | $0.5000 | 380 ms | 75.0 tok/s |
| **Sakana: Fugu Ultra** (`sakana/fugu-ultra`) | `openrouter_market` | $5.00 | $30.00 | $0.5000 | 450 ms | 70.0 tok/s |
| **Sakana: Fugu Ultra v2** (`sakana/fugu-ultra-v2`) | `nous_portal` | $5.00 | $30.00 | $0.5000 | 380 ms | 75.0 tok/s |
| **Sakana: Fugu Ultra v2** (`sakana/fugu-ultra-v2`) | `openrouter_market` | $5.00 | $30.00 | $0.5000 | 450 ms | 70.0 tok/s |
| **Sentence Transformers: all-MiniLM-L12-v2** (`sentence-transformers/all-minilm-l12-v2`) | `nous_portal` | $0.01 | $0.00 | $0.0010 | 220 ms | 130.0 tok/s |
| **Sentence Transformers: all-MiniLM-L6-v2** (`sentence-transformers/all-minilm-l6-v2`) | `nous_portal` | $0.01 | $0.00 | $0.0010 | 220 ms | 130.0 tok/s |
| **Sentence Transformers: all-mpnet-base-v2** (`sentence-transformers/all-mpnet-base-v2`) | `nous_portal` | $0.01 | $0.00 | $0.0010 | 380 ms | 75.0 tok/s |
| **Sentence Transformers: multi-qa-mpnet-base-dot-v1** (`sentence-transformers/multi-qa-mpnet-base-dot-v1`) | `nous_portal` | $0.01 | $0.00 | $0.0010 | 380 ms | 75.0 tok/s |
| **Sentence Transformers: paraphrase-MiniLM-L6-v2** (`sentence-transformers/paraphrase-minilm-l6-v2`) | `nous_portal` | $0.01 | $0.00 | $0.0010 | 220 ms | 130.0 tok/s |
| **MissingNo** (`stealth/missingno`) | `nous_portal` | $0.00 | $0.00 | N/A | 380 ms | 75.0 tok/s |
| **StepFun: Step 3.5 Flash** (`stepfun/step-3.5-flash`) | `nous_portal` | $0.10 | $0.30 | $0.0200 | 220 ms | 130.0 tok/s |
| **StepFun: Step 3.5 Flash** (`stepfun/step-3.5-flash`) | `openrouter_market` | $0.10 | $0.30 | $0.0250 | 450 ms | 130.0 tok/s |
| **StepFun: Step 3.7 Flash** (`stepfun/step-3.7-flash`) | `nous_portal` | $0.20 | $1.15 | $0.0400 | 220 ms | 130.0 tok/s |
| **StepFun: Step 3.7 Flash** (`stepfun/step-3.7-flash`) | `openrouter_market` | $0.20 | $1.15 | $0.0400 | 450 ms | 130.0 tok/s |
| **StepFun: Step 5 Preview** (`stepfun/step-5-preview`) | `nous_portal` | $1.00 | $2.70 | $0.0500 | 380 ms | 75.0 tok/s |
| **StepFun: Step 5 Preview** (`stepfun/step-5-preview`) | `openrouter_market` | $1.00 | $2.70 | $0.0500 | 450 ms | 70.0 tok/s |
| **Tencent: Hy3** (`tencent/hy3`) | `nous_portal` | $0.13 | $0.53 | $0.0330 | 380 ms | 75.0 tok/s |
| **Tencent: Hy3** (`tencent/hy3`) | `openrouter_market` | $0.13 | $0.53 | $0.0330 | 450 ms | 70.0 tok/s |
| **Tencent: Hy3 preview** (`tencent/hy3-preview`) | `nous_portal` | $0.18 | $0.60 | $0.0600 | 380 ms | 75.0 tok/s |
| **Tencent: Hy3 preview** (`tencent/hy3-preview`) | `openrouter_market` | $0.18 | $0.60 | $0.0600 | 450 ms | 70.0 tok/s |
| **Tencent: Hy4 preview** (`tencent/hy4-preview`) | `nous_portal` | $0.42 | $1.25 | $0.0210 | 380 ms | 75.0 tok/s |
| **Tencent: Hy4 preview** (`tencent/hy4-preview`) | `openrouter_market` | $0.83 | $2.50 | $0.0420 | 450 ms | 70.0 tok/s |
| **Thenlper: GTE-Base** (`thenlper/gte-base`) | `nous_portal` | $0.01 | $0.00 | $0.0010 | 380 ms | 75.0 tok/s |
| **Thenlper: GTE-Large** (`thenlper/gte-large`) | `nous_portal` | $0.01 | $0.00 | $0.0020 | 380 ms | 75.0 tok/s |
| **Thinking Machines: Inkling** (`thinkingmachines/inkling`) | `openrouter_market` | $0.00 | $0.00 | N/A | 450 ms | 70.0 tok/s |
| **Thinking Machines: Inkling** (`thinkingmachines/inkling`) | `nous_portal` | $0.95 | $4.05 | $0.1600 | 380 ms | 75.0 tok/s |
| **Thinking Machines: Inkling** (`thinkingmachines/inkling`) | `openrouter_market` | $1.00 | $4.05 | $0.1700 | 450 ms | 70.0 tok/s |
| **Thinking Machines: Inkling Small** (`thinkingmachines/inkling-small`) | `openrouter_market` | $0.00 | $0.00 | N/A | 450 ms | 70.0 tok/s |
| **Thinking Machines: Inkling Small** (`thinkingmachines/inkling-small`) | `nous_portal` | $0.45 | $1.20 | $0.1000 | 380 ms | 75.0 tok/s |
| **Thinking Machines: Inkling Small** (`thinkingmachines/inkling-small`) | `openrouter_market` | $0.45 | $1.20 | $0.1000 | 450 ms | 70.0 tok/s |
| **Pareto 26.10 Preview** (`unbiased/pareto-26.10-preview`) | `nous_portal` | $0.80 | $3.20 | $0.0300 | 380 ms | 75.0 tok/s |
| **Pareto 26.10 Preview** (`unbiased/pareto-26.10-preview`) | `openrouter_market` | $0.80 | $3.20 | $0.0300 | 450 ms | 70.0 tok/s |
| **Pareto 26.9** (`unbiased/pareto-26.9`) | `nous_portal` | $2.50 | $7.50 | $0.2500 | 380 ms | 75.0 tok/s |
| **Pareto 26.9** (`unbiased/pareto-26.9`) | `openrouter_market` | $2.50 | $7.50 | $0.2500 | 450 ms | 70.0 tok/s |
| **Pareto Latest** (`unbiased/pareto-latest`) | `nous_portal` | $2.50 | $7.50 | $0.2500 | 380 ms | 75.0 tok/s |
| **Pareto Latest** (`unbiased/pareto-latest`) | `openrouter_market` | $2.50 | $7.50 | $0.2500 | 450 ms | 70.0 tok/s |
| **Upstage: Solar Mini 4** (`upstage/solar-mini4`) | `nous_portal` | $0.05 | $0.20 | $0.0050 | 220 ms | 130.0 tok/s |
| **Upstage: Solar Mini 4** (`upstage/solar-mini4`) | `openrouter_market` | $0.05 | $0.20 | $0.0050 | 450 ms | 130.0 tok/s |
| **Upstage: Solar Pro 3** (`upstage/solar-pro-3`) | `nous_portal` | $0.15 | $0.60 | $0.0150 | 380 ms | 90.0 tok/s |
| **Upstage: Solar Pro 3** (`upstage/solar-pro-3`) | `openrouter_market` | $0.15 | $0.60 | $0.0150 | 450 ms | 95.0 tok/s |
| **Upstage: Solar Pro 4** (`upstage/solar-pro4`) | `nous_portal` | $0.09 | $0.36 | $0.0180 | 380 ms | 90.0 tok/s |
| **Upstage: Solar Pro 4** (`upstage/solar-pro4`) | `openrouter_market` | $0.09 | $0.36 | $0.0180 | 450 ms | 95.0 tok/s |
| **VoyageAI by MongoDB: voyage-4** (`voyageai/voyage-4`) | `nous_portal` | $0.06 | $0.00 | $0.0120 | 380 ms | 75.0 tok/s |
| **VoyageAI by MongoDB: voyage-4-large** (`voyageai/voyage-4-large`) | `nous_portal` | $0.12 | $0.00 | $0.0240 | 380 ms | 75.0 tok/s |
| **VoyageAI by MongoDB: voyage-4-lite** (`voyageai/voyage-4-lite`) | `nous_portal` | $0.02 | $0.00 | $0.0040 | 380 ms | 75.0 tok/s |
| **VoyageAI by MongoDB: voyage-code-4** (`voyageai/voyage-code-4`) | `nous_portal` | $0.12 | $0.00 | $0.0240 | 380 ms | 75.0 tok/s |
| **VoyageAI by MongoDB: voyage-multimodal-3.5** (`voyageai/voyage-multimodal-3.5`) | `nous_portal` | $0.12 | $0.00 | $0.0240 | 380 ms | 75.0 tok/s |
| **SpaceXAI: Grok 4.20** (`x-ai/grok-4.20`) | `nous_portal` | $1.25 | $2.50 | $0.2000 | 380 ms | 75.0 tok/s |
| **SpaceXAI: Grok 4.20** (`x-ai/grok-4.20`) | `openrouter_market` | $1.25 | $2.50 | $0.2000 | 450 ms | 70.0 tok/s |
| **SpaceXAI: Grok 4.3** (`x-ai/grok-4.3`) | `nous_portal_batch` | $1.00 | $2.00 | $0.1600 | 5000 ms | 150.0 tok/s |
| **SpaceXAI: Grok 4.3** (`x-ai/grok-4.3`) | `openrouter_market` | $1.00 | $2.00 | $0.1600 | 450 ms | 70.0 tok/s |
| **SpaceXAI: Grok 4.3** (`x-ai/grok-4.3`) | `nous_portal` | $1.25 | $2.50 | $0.2000 | 380 ms | 75.0 tok/s |
| **SpaceXAI: Grok 4.3** (`x-ai/grok-4.3`) | `openrouter_market` | $1.25 | $2.50 | $0.2000 | 450 ms | 70.0 tok/s |
| **xAI: Grok 4.5** (`x-ai/grok-4.5`) | `nous_portal` | $2.00 | $6.00 | $0.3000 | 380 ms | 75.0 tok/s |
| **xAI: Grok 4.5** (`x-ai/grok-4.5`) | `openrouter_market` | $2.00 | $6.00 | $0.3000 | 450 ms | 70.0 tok/s |
| **xAI: Grok 4.6** (`x-ai/grok-4.6`) | `nous_portal` | $2.00 | $6.00 | $0.5000 | 380 ms | 75.0 tok/s |
| **xAI: Grok 4.6** (`x-ai/grok-4.6`) | `openrouter_market` | $2.00 | $6.00 | $0.5000 | 450 ms | 70.0 tok/s |
| **xAI: Grok 4.7** (`x-ai/grok-4.7`) | `nous_portal` | $2.00 | $6.00 | $0.5000 | 380 ms | 75.0 tok/s |
| **xAI: Grok 4.7** (`x-ai/grok-4.7`) | `openrouter_market` | $2.00 | $6.00 | $0.5000 | 450 ms | 70.0 tok/s |
| **SpaceXAI: Grok Build 0.1** (`x-ai/grok-build-0.1`) | `nous_portal` | $1.00 | $2.00 | $0.2000 | 380 ms | 75.0 tok/s |
| **SpaceXAI: Grok Build 0.1** (`x-ai/grok-build-0.1`) | `openrouter_market` | $1.00 | $2.00 | $0.2000 | 450 ms | 70.0 tok/s |
| **xAI: Grok Latest** (`x-ai/grok-latest`) | `nous_portal` | $2.00 | $6.00 | $0.5000 | 380 ms | 75.0 tok/s |
| **xAI: Grok Latest** (`x-ai/grok-latest`) | `openrouter_market` | $2.00 | $6.00 | $0.5000 | 450 ms | 70.0 tok/s |
| **Xiaomi: MiMo-V2.5** (`xiaomi/mimo-v2.5`) | `nous_portal` | $0.12 | $0.24 | $0.0026 | 380 ms | 75.0 tok/s |
| **Xiaomi: MiMo-V2.5** (`xiaomi/mimo-v2.5`) | `openrouter_market` | $0.14 | $0.28 | $0.0028 | 450 ms | 70.0 tok/s |
| **Xiaomi: MiMo-V2.5-Pro** (`xiaomi/mimo-v2.5-pro`) | `nous_portal` | $0.30 | $0.61 | $0.0028 | 380 ms | 75.0 tok/s |
| **Xiaomi: MiMo-V2.5-Pro** (`xiaomi/mimo-v2.5-pro`) | `openrouter_market` | $0.43 | $0.87 | $0.0036 | 450 ms | 70.0 tok/s |
| **Xiaomi: MiMo-V2.6-Flash** (`xiaomi/mimo-v2.6-flash`) | `nous_portal` | $0.10 | $0.28 | $0.0500 | 220 ms | 130.0 tok/s |
| **Xiaomi: MiMo-V2.6-Flash** (`xiaomi/mimo-v2.6-flash`) | `openrouter_market` | $0.14 | $0.28 | $0.0028 | 450 ms | 130.0 tok/s |
| **Xiaomi: MiMo-V2.6-Pro** (`xiaomi/mimo-v2.6-pro`) | `nous_portal` | $0.43 | $0.87 | $0.0036 | 380 ms | 75.0 tok/s |
| **Xiaomi: MiMo-V2.6-Pro** (`xiaomi/mimo-v2.6-pro`) | `openrouter_market` | $0.43 | $0.87 | $0.0036 | 450 ms | 70.0 tok/s |
| **Xiaomi: MiMo-V2.6-Pro-UltraSpeed** (`xiaomi/mimo-v2.6-pro-ultraspeed`) | `nous_portal` | $4.35 | $8.70 | $0.0360 | 380 ms | 75.0 tok/s |
| **Xiaomi: MiMo-V2.6-Pro-UltraSpeed** (`xiaomi/mimo-v2.6-pro-ultraspeed`) | `openrouter_market` | $4.35 | $8.70 | $0.0360 | 450 ms | 70.0 tok/s |
| **Z.ai: GLM 4.5** (`z-ai/glm-4.5`) | `nous_portal` | $0.60 | $2.20 | $0.1100 | 380 ms | 75.0 tok/s |
| **Z.ai: GLM 4.5** (`z-ai/glm-4.5`) | `openrouter_market` | $0.60 | $2.20 | $0.1100 | 450 ms | 70.0 tok/s |
| **Z.ai: GLM 4.5 Air** (`z-ai/glm-4.5-air`) | `nous_portal` | $0.13 | $0.85 | $0.0250 | 380 ms | 75.0 tok/s |
| **Z.ai: GLM 4.5 Air** (`z-ai/glm-4.5-air`) | `openrouter_market` | $0.13 | $0.85 | $0.0250 | 450 ms | 70.0 tok/s |
| **Z.ai: GLM 4.5V** (`z-ai/glm-4.5v`) | `nous_portal` | $0.60 | $1.80 | $0.1100 | 380 ms | 75.0 tok/s |
| **Z.ai: GLM 4.5V** (`z-ai/glm-4.5v`) | `openrouter_market` | $0.60 | $1.80 | $0.1100 | 450 ms | 70.0 tok/s |
| **Z.ai: GLM 4.6** (`z-ai/glm-4.6`) | `nous_portal` | $0.43 | $1.75 | $0.0800 | 380 ms | 75.0 tok/s |
| **Z.ai: GLM 4.6** (`z-ai/glm-4.6`) | `openrouter_market` | $0.50 | $2.00 | $0.1000 | 450 ms | 70.0 tok/s |
| **Z.ai: GLM 4.6V** (`z-ai/glm-4.6v`) | `nous_portal` | $0.30 | $0.90 | $0.0550 | 380 ms | 75.0 tok/s |
| **Z.ai: GLM 4.6V** (`z-ai/glm-4.6v`) | `openrouter_market` | $0.30 | $0.90 | $0.0550 | 450 ms | 70.0 tok/s |
| **Z.ai: GLM 4.7** (`z-ai/glm-4.7`) | `nous_portal` | $0.40 | $1.75 | $0.0800 | 380 ms | 75.0 tok/s |
| **Z.ai: GLM 4.7** (`z-ai/glm-4.7`) | `openrouter_market` | $0.60 | $2.20 | $0.1100 | 450 ms | 70.0 tok/s |
| **Z.ai: GLM 4.7 Flash** (`z-ai/glm-4.7-flash`) | `nous_portal` | $0.06 | $0.40 | $0.0100 | 220 ms | 130.0 tok/s |
| **Z.ai: GLM 4.7 Flash** (`z-ai/glm-4.7-flash`) | `openrouter_market` | $0.06 | $0.40 | $0.0151 | 450 ms | 130.0 tok/s |
| **Z.ai: GLM 5** (`z-ai/glm-5`) | `nous_portal` | $0.60 | $1.92 | $0.1200 | 380 ms | 75.0 tok/s |
| **Z.ai: GLM 5** (`z-ai/glm-5`) | `openrouter_market` | $0.60 | $1.92 | $0.1200 | 450 ms | 70.0 tok/s |
| **Z.ai: GLM 5 Turbo** (`z-ai/glm-5-turbo`) | `nous_portal` | $1.20 | $4.00 | $0.2400 | 380 ms | 75.0 tok/s |
| **Z.ai: GLM 5 Turbo** (`z-ai/glm-5-turbo`) | `openrouter_market` | $1.20 | $4.00 | $0.2400 | 450 ms | 70.0 tok/s |
| **Z.ai: GLM 5.1** (`z-ai/glm-5.1`) | `nous_portal` | $0.97 | $3.04 | $0.1794 | 380 ms | 75.0 tok/s |
| **Z.ai: GLM 5.1** (`z-ai/glm-5.1`) | `openrouter_market` | $1.33 | $4.18 | $0.2470 | 450 ms | 70.0 tok/s |
| **Z.ai: GLM 5.2** (`z-ai/glm-5.2`) | `openrouter_market` | $0.05 | $7.00 | $0.0490 | 450 ms | 70.0 tok/s |
| **Z.ai: GLM 5.2** (`z-ai/glm-5.2`) | `nous_portal` | $0.90 | $2.83 | $0.1600 | 380 ms | 75.0 tok/s |
| **Z.ai: GLM 5.3** (`z-ai/glm-5.3`) | `openrouter_market` | $0.08 | $4.80 | $0.0590 | 450 ms | 70.0 tok/s |
| **Z.ai: GLM 5.3** (`z-ai/glm-5.3`) | `nous_portal_batch` | $0.45 | $2.00 | $0.1000 | 5000 ms | 150.0 tok/s |
| **Z.ai: GLM 5.3** (`z-ai/glm-5.3`) | `openrouter_market` | $0.45 | $2.00 | $0.1000 | 450 ms | 70.0 tok/s |
| **Z.ai: GLM 5.3** (`z-ai/glm-5.3`) | `nous_portal` | $0.91 | $2.86 | $0.1690 | 380 ms | 75.0 tok/s |
| **Z.ai: GLM 5.3 Flash** (`z-ai/glm-5.3-flash`) | `nous_portal_batch` | $0.06 | $0.20 | $0.0120 | 5000 ms | 150.0 tok/s |
| **Z.ai: GLM 5.3 Flash** (`z-ai/glm-5.3-flash`) | `openrouter_market` | $0.06 | $0.20 | $0.0120 | 450 ms | 130.0 tok/s |
| **Z.ai: GLM 5.3 Flash** (`z-ai/glm-5.3-flash`) | `nous_portal` | $0.12 | $0.40 | $0.0240 | 220 ms | 130.0 tok/s |
| **Z.ai: GLM 5.3 Flash** (`z-ai/glm-5.3-flash`) | `openrouter_market` | $0.15 | $0.50 | $0.0300 | 450 ms | 130.0 tok/s |
| **Z.ai: GLM 5.3 Flash** (`z-ai/glm-5.3-flash:US`) | `nous_portal` | $0.25 | $0.83 | $0.0495 | 220 ms | 130.0 tok/s |
| **Z.ai: GLM 5.3 FlashX** (`z-ai/glm-5.3-flashx`) | `nous_portal` | $0.37 | $1.25 | $0.0900 | 220 ms | 130.0 tok/s |
| **Z.ai: GLM 5.3 FlashX** (`z-ai/glm-5.3-flashx`) | `openrouter_market` | $0.37 | $1.25 | $0.0900 | 450 ms | 130.0 tok/s |
| **Z.ai: GLM 5.3 Prime** (`z-ai/glm-5.3-prime`) | `nous_portal` | $2.80 | $8.80 | $0.5600 | 380 ms | 75.0 tok/s |
| **Z.ai: GLM 5.3 Prime** (`z-ai/glm-5.3-prime`) | `openrouter_market` | $2.80 | $8.80 | $0.5600 | 450 ms | 70.0 tok/s |
| **Z.ai: GLM 5.3** (`z-ai/glm-5.3:US`) | `nous_portal` | $2.31 | $7.26 | $0.4290 | 380 ms | 75.0 tok/s |
| **Z.ai: GLM 5V Turbo** (`z-ai/glm-5v-turbo`) | `nous_portal` | $1.20 | $4.00 | $0.2400 | 380 ms | 75.0 tok/s |
| **Z.ai: GLM 5V Turbo** (`z-ai/glm-5v-turbo`) | `openrouter_market` | $1.20 | $4.00 | $0.2400 | 450 ms | 70.0 tok/s |
| **Z.ai: GLM Flash Latest** (`z-ai/glm-flash-latest`) | `nous_portal` | $0.04 | $0.90 | $0.0108 | 220 ms | 130.0 tok/s |
| **Z.ai: GLM Flash Latest** (`z-ai/glm-flash-latest`) | `openrouter_market` | $0.04 | $0.90 | $0.0108 | 450 ms | 130.0 tok/s |
| **Z.ai: GLM Latest** (`z-ai/glm-latest`) | `nous_portal` | $0.06 | $4.40 | $0.0586 | 380 ms | 75.0 tok/s |
| **Z.ai: GLM Latest** (`z-ai/glm-latest`) | `openrouter_market` | $0.06 | $4.40 | $0.0586 | 450 ms | 70.0 tok/s |

---

## 🎯 5. BENCHMARKS EMPÍRICOS E MÉTRICAS DE ENGENHARIA
Resultados empíricos normalizados (Artificial Analysis Intelligence / Coding / Agentic e LMSYS Arena Elo) utilizados no cômputo da Fronteira de Pareto:

| Modelo Canônico | Artificial Analysis / Elo | Coding Score | Hard/Agentic | BFCL (Tools) | IFEval | GPQA | VQA Score |
| :--- | :---: | :---: | :---: | :---: | :---: | :---: | :---: |
| **Claude 3.7 Sonnet** | 50.0 | **50.0** | 50.0 | 55.0% | 50.0% | 45.0% | 55.0% |
| **Claude 3.5 Sonnet** | 50.0 | **50.0** | 50.0 | 55.0% | 50.0% | 45.0% | 55.0% |
| **DeepSeek V3** | 40.0 | **40.0** | 40.0 | 45.0% | 40.0% | 35.0% | — |
| **DeepSeek R1** | 11.4 | **24.6** | 1.1 | 1.1% | — | — | — |
| **Qwen 2.5 Coder 32B** | 6.7 | — | — | — | — | — | — |
| **GPT-4o** | 40.0 | **40.0** | 40.0 | 45.0% | 40.0% | 35.0% | 45.0% |
| **GPT-4o Mini** | 6.7 | **11.4** | — | — | — | — | — |
| **OpenAI o3-mini** | 12.5 | — | — | — | — | — | — |
| **Llama 3.3 70B Instruct** | 7.7 | **11.9** | — | — | — | — | — |
| **Hermes 3 (Llama 3.1 405B)** | 7.4 | — | — | — | — | — | — |
| **Hermes 3 (Llama 3.1 70B)** | 6.7 | — | — | — | — | — | — |
| **Gemini 2.0 Flash** | 44.0 | **44.0** | 44.0 | 49.0% | 44.0% | 39.0% | 49.0% |
| **ByteDance Seed: Seed-2.0-Lite** | 40.0 | **40.0** | 40.0 | 45.0% | 40.0% | 35.0% | 45.0% |
| **MoonshotAI: Kimi K3** | 43.6 | **76.2** | 50.0 | 50.0% | — | — | — |
| **StepFun: Step 5 Preview** | 45.0 | **45.0** | 45.0 | 50.0% | 45.0% | 40.0% | 50.0% |
| **AionLabs: Aion 3.5 Mini** | 44.0 | **44.0** | 44.0 | 49.0% | 44.0% | 39.0% | — |
| **Google: Gemini 3.1 Pro Preview** | 29.7 | **68.8** | 8.2 | 8.2% | — | — | — |
| **OpenAI: GPT Sol Latest** | 50.0 | **50.0** | 50.0 | 55.0% | 50.0% | 45.0% | 55.0% |
| **Qwen: Qwen3 Embedding 8B** | 40.0 | **40.0** | 40.0 | 30.0% | 40.0% | 35.0% | — |
| **MiniMax: MiniMax M3** | 29.2 | **58.6** | 29.5 | 29.5% | — | — | — |
| **Anthropic: Claude Sonnet 4** | 18.9 | **37.6** | — | — | — | — | — |
| **Intfloat: E5-Base-v2** | 40.0 | **40.0** | 40.0 | 30.0% | 40.0% | 35.0% | — |
| **OpenAI: o4 Mini** | 16.7 | — | — | — | — | — | — |
| **Sakana: Fugu Max** | 45.0 | **45.0** | 45.0 | 50.0% | 45.0% | 40.0% | 50.0% |
| **Qwen: Qwen3 32B** | 8.6 | **15.3** | 0.9 | 0.9% | — | — | — |
| **OpenAI: GPT Astra Latest** | 53.0 | **53.0** | 53.0 | 58.0% | 53.0% | 48.0% | 58.0% |
| **OpenAI: GPT-6 Astra Pro Flex** | 53.0 | **53.0** | 53.0 | 58.0% | 53.0% | 48.0% | 58.0% |
| **MiniMax: MiniMax M2.5** | 22.8 | — | — | — | — | — | — |
| **Mistral: Saba** | 6.5 | — | — | — | — | — | — |
| **Anthropic: Claude Sonnet Latest** | 50.0 | **50.0** | 50.0 | 55.0% | 50.0% | 45.0% | 55.0% |
| **OpenAI: Text Embedding Ada 002** | 40.0 | **40.0** | 40.0 | 30.0% | 40.0% | 35.0% | — |
| **Google: Nano Banana 2.1** | 44.0 | **44.0** | 44.0 | 49.0% | 44.0% | 39.0% | 49.0% |
| **DeepSeek: DeepSeek V4.1 Flash** | 39.5 | — | — | — | — | — | — |
| **Z.ai: GLM 5.1** | 26.1 | **55.8** | 23.9 | 23.9% | — | — | — |
| **Google: Gemini 4 Argon** | 52.6 | — | — | — | — | — | — |
| **Qwen: Qwen3 Coder Next** | 9.2 | **36.2** | 0.9 | 0.9% | — | — | — |
| **Qwen: Qwen3 235B A22B Instruct 2507** | 12.0 | — | — | — | — | — | — |
| **Mistral Large** | 5.8 | — | — | — | — | — | — |
| **Sentence Transformers: multi-qa-mpnet-base-dot-v1** | 40.0 | **40.0** | 40.0 | 30.0% | 40.0% | 35.0% | — |
| **OpenAI: GPT-5.6 Sol Pro** | 50.0 | **50.0** | 50.0 | 55.0% | 50.0% | 45.0% | 55.0% |
| **Thinking Machines: Inkling Small** | 25.7 | **52.9** | 23.5 | 23.5% | — | — | — |
| **Inception: Mercury 2.5** | 12.3 | — | — | — | — | — | — |
| **BAAI: bge-base-en-v1.5** | 40.0 | **40.0** | 40.0 | 30.0% | 40.0% | 35.0% | — |
| **Anthropic: Claude Sonnet 5.5** | 56.0 | — | — | — | — | — | — |
| **Xiaomi: MiMo-V2.6-Flash** | 37.9 | **1455.2** | 1455.2 | — | — | — | — |
| **OpenAI: gpt-oss-safeguard-20b** | 40.0 | **40.0** | 40.0 | 45.0% | 40.0% | 35.0% | — |
| **OpenAI: GPT-6 Astra Pro Fast** | 53.0 | **53.0** | 53.0 | 58.0% | 53.0% | 48.0% | 58.0% |
| **Tencent: Hy3 preview** | 25.3 | **58.8** | 24.1 | 24.1% | — | — | — |
| **Qwen: Qwen3 Next 80B A3B Thinking** | 11.2 | **17.4** | — | — | — | — | — |
| **OpenAI: GPT-5.1-Codex** | 23.7 | — | — | — | — | — | — |
| **OpenAI: GPT-6 Astra Fast** | 53.0 | **53.0** | 53.0 | 58.0% | 53.0% | 48.0% | 58.0% |
| **Google: Gemma 3 12B** | 3.8 | **5.8** | 0.1 | 0.1% | — | — | — |
| **Qwen: Qwen3.8 Max (0902)** | 45.4 | **76.2** | 56.0 | 56.0% | — | — | — |
| **Claude Sonnet 4.6** | 30.1 | **63.0** | 31.8 | 31.8% | — | — | — |
| **Claude Opus 4.6** | 31.9 | — | — | — | — | — | — |
| **OpenAI: o1** | 15.2 | **39.7** | — | — | — | — | — |
| **DeepSeek: DeepSeek Pro Latest** | 45.0 | **45.0** | 45.0 | 50.0% | 45.0% | 40.0% | — |
| **Pareto Latest** | 40.0 | **40.0** | 40.0 | 45.0% | 40.0% | 35.0% | 45.0% |
| **Mistral: Mistral Medium 3.1** | 9.2 | **20.5** | 1.9 | 1.9% | — | — | — |
| **ByteDance Seed: Seed 1.6 Flash** | 44.0 | **44.0** | 44.0 | 49.0% | 44.0% | 39.0% | 49.0% |
| **OpenAI: GPT-6 Astra** | 52.7 | **76.9** | 51.0 | 51.0% | — | — | — |
| **Qwen: Qwen3.6 35B A3B** | 18.2 | **41.9** | 13.1 | 13.1% | — | — | — |
| **OpenAI: GPT-4o (2024-05-13)** | 7.3 | **24.2** | — | — | — | — | — |
| **Intfloat: E5-Large-v2** | 40.0 | **40.0** | 40.0 | 30.0% | 40.0% | 35.0% | — |
| **VoyageAI by MongoDB: voyage-multimodal-3.5** | 40.0 | **40.0** | 40.0 | 30.0% | 40.0% | 35.0% | 45.0% |
| **Qwen: Qwen2.5 7B Instruct** | 40.0 | **40.0** | 40.0 | 45.0% | 40.0% | 35.0% | — |
| **Nex AGI: Nex-N2.5-Pro** | 40.0 | **40.0** | 40.0 | 45.0% | 40.0% | 35.0% | 45.0% |
| **Z.ai: GLM 5.3** | 44.8 | **74.8** | 53.1 | 53.1% | — | — | — |
| **OpenAI: gpt-oss-120b** | 11.6 | **30.4** | 3.7 | 3.7% | — | — | — |
| **Meta: Llama 4 Maverick** | 10.0 | **16.3** | 0.6 | 0.6% | — | — | — |
| **MoonshotAI: Kimi K2 0711** | 12.7 | — | — | — | — | — | — |
| **Google: Gemini 3.1 Flash Lite Preview** | 15.6 | **34.7** | 1.6 | 1.6% | — | — | — |
| **Sentence Transformers: all-mpnet-base-v2** | 40.0 | **40.0** | 40.0 | 30.0% | 40.0% | 35.0% | — |
| **Google: Gemini 3.6 Flash** | 34.0 | **69.2** | 29.0 | 29.0% | — | — | — |
| **IBM: Granite 4.2 8B** | 11.1 | **22.4** | 1.3 | 1.3% | — | — | — |
| **MoonshotAI: Kimi K2.7 Code** | 25.8 | **60.8** | 21.0 | 21.0% | — | — | — |
| **Claude Opus 4.7** | 40.7 | **73.6** | 38.6 | 38.6% | — | — | — |
| **OpenAI: GPT-5.4** | 39.0 | **71.1** | 1452.2 | — | — | — | — |
| **Qwen: Qwen3.5 397B A17B** | 21.4 | **1437.9** | 1437.9 | — | — | — | — |
| **Google: Gemini 3.5 Flash Lite** | 22.2 | **49.3** | 14.3 | 14.3% | — | — | — |
| **OpenAI: GPT-5.2** | 30.4 | — | — | — | — | — | — |
| **Inception: Mercury 2** | 13.8 | **31.1** | 1.6 | 1.6% | — | — | — |
| **Z.ai: GLM 5 Turbo** | 26.6 | — | — | — | — | — | — |
| **Mistral: Devstral 2 2512** | 8.6 | **31.3** | 2.3 | 2.3% | — | — | — |
| **Meta: Llama 4 Scout** | 8.1 | **8.2** | 0.5 | 0.5% | — | — | — |
| **Google: Gemma 3 27B** | 4.9 | **10.1** | 0.1 | 0.1% | — | — | — |
| **OpenAI: GPT Terra Latest** | 45.0 | **45.0** | 45.0 | 50.0% | 45.0% | 40.0% | 50.0% |
| **Poolside: Laguna XS 2.1** | 40.0 | **40.0** | 40.0 | 45.0% | 40.0% | 35.0% | — |
| **Anthropic: Claude Haiku Latest** | 44.0 | **44.0** | 44.0 | 49.0% | 44.0% | 39.0% | 49.0% |
| **Perplexity: Embed V1 4B** | 40.0 | **40.0** | 40.0 | 30.0% | 40.0% | 35.0% | — |
| **ByteDance Seed: Seed 1.6** | 40.0 | **40.0** | 40.0 | 45.0% | 40.0% | 35.0% | 45.0% |
| **OpenAI: GPT-6 Sol Pro** | 50.0 | **50.0** | 50.0 | 55.0% | 50.0% | 45.0% | 55.0% |
| **Z.ai: GLM Latest** | 45.0 | **45.0** | 45.0 | 50.0% | 45.0% | 40.0% | — |
| **OpenAI: Text Embedding 3 Large** | 40.0 | **40.0** | 40.0 | 30.0% | 40.0% | 35.0% | — |
| **Perplexity: Embed V1 0.6B** | 40.0 | **40.0** | 40.0 | 30.0% | 40.0% | 35.0% | — |
| **Z.ai: GLM 5.3 Flash** | 41.8 | **71.5** | 50.9 | 50.9% | — | — | — |
| **Qwen: Qwen3 VL 30B A3B Instruct** | 7.9 | — | — | — | — | — | — |
| **Google: Gemini Embedding 2 Preview** | 44.0 | **44.0** | 44.0 | 34.0% | 44.0% | 39.0% | 49.0% |
| **Google: Gemini 3.1 Pro Preview Custom Tools** | 44.0 | **44.0** | 44.0 | 49.0% | 44.0% | 39.0% | 49.0% |
| **Tencent: Hy3** | 1437.4 | **1437.4** | 1437.4 | — | — | — | — |
| **Intfloat: Multilingual-E5-Large** | 40.0 | **40.0** | 40.0 | 30.0% | 40.0% | 35.0% | — |
| **Mistral: Mistral Small 3.2 24B** | 40.0 | **40.0** | 40.0 | 45.0% | 40.0% | 35.0% | 45.0% |
| **OpenAI: GPT-3.5 Turbo 16k** | 40.0 | **40.0** | 40.0 | 45.0% | 40.0% | 35.0% | — |
| **Cohere: Command A+** | 40.0 | **40.0** | 40.0 | 45.0% | 40.0% | 35.0% | 45.0% |
| **AionLabs: Aion-3.0-Mini** | 44.0 | **44.0** | 44.0 | 49.0% | 44.0% | 39.0% | — |
| **Google: Nano Banana Pro (Gemini 3 Pro Image)** | 44.0 | **44.0** | 44.0 | 49.0% | 44.0% | 39.0% | 49.0% |
| **Qwen: Qwen3 Coder 30B A3B Instruct** | 9.6 | — | — | — | — | — | — |
| **Google: Gemini 3.8 Flash** | 40.9 | **76.3** | 40.2 | 40.2% | — | — | — |
| **Qwen: Qwen3.5-9B** | 13.3 | **23.5** | — | — | — | — | — |
| **Z.ai: GLM 4.7** | 22.2 | **45.3** | 1435.5 | — | — | — | — |
| **Qwen: Qwen3.5 Plus 2026-02-15** | 45.0 | **45.0** | 45.0 | 50.0% | 45.0% | 40.0% | 50.0% |
| **MiniMax: MiniMax M2** | 18.6 | — | — | — | — | — | — |
| **OpenAI: GPT-5.5** | 38.4 | **74.9** | 36.4 | 36.4% | — | — | — |
| **OpenAI: GPT-5 Pro** | 40.0 | **40.0** | 40.0 | 45.0% | 40.0% | 35.0% | 45.0% |
| **MoonshotAI: Kimi K3** | 45.0 | **45.0** | 45.0 | 50.0% | 45.0% | 40.0% | 50.0% |
| **OpenAI: GPT-5.3-Codex** | 32.5 | — | — | — | — | — | — |
| **OpenAI: GPT-5.6 Luna** | 37.3 | **71.4** | 42.1 | 42.1% | — | — | — |
| **Qwen: Qwen3.8 Flash** | 44.0 | **44.0** | 44.0 | 49.0% | 44.0% | 39.0% | 49.0% |
| **Thinking Machines: Inkling** | 25.0 | **52.1** | 22.5 | 22.5% | — | — | — |
| **Pareto 26.9** | 40.0 | **40.0** | 40.0 | 45.0% | 40.0% | 35.0% | 45.0% |
| **Anthropic: Claude Haiku 5.5** | 43.4 | — | — | — | — | — | — |
| **DeepSeek: DeepSeek V4 Flash Latest** | 44.0 | **44.0** | 44.0 | 49.0% | 44.0% | 39.0% | — |
| **Qwen: Qwen3 30B A3B** | 7.6 | — | — | — | — | — | — |
| **OpenAI: GPT-4.1 Mini** | 10.2 | **20.2** | — | — | — | — | — |
| **Z.ai: GLM 5.3 FlashX** | 44.0 | **44.0** | 44.0 | 49.0% | 44.0% | 39.0% | 49.0% |
| **Meituan: LongCat 2.0** | 19.1 | **45.3** | 14.0 | 14.0% | — | — | — |
| **Qwen: Qwen3.7 Max** | 29.5 | **66.0** | 22.5 | 22.5% | — | — | — |
| **Poolside: Laguna S 2.1** | 45.0 | **45.0** | 45.0 | 50.0% | 45.0% | 40.0% | — |
| **AionLabs: Aion-2.0** | 40.0 | **40.0** | 40.0 | 45.0% | 40.0% | 35.0% | — |
| **Google: Gemini Embedding 001** | 44.0 | **44.0** | 44.0 | 34.0% | 44.0% | 39.0% | — |
| **Qwen: Qwen3 30B A3B Instruct 2507** | 7.5 | — | — | — | — | — | — |
| **Google: Gemini 3.1 Flash Lite** | 44.0 | **44.0** | 44.0 | 49.0% | 44.0% | 39.0% | 49.0% |
| **OpenAI: GPT-4.1** | 12.7 | — | — | — | — | — | — |
| **Google: Gemma 4 26B A4B ** | 16.7 | **39.3** | 2.9 | 2.9% | — | — | — |
| **OpenAI: GPT-5.4 Mini** | 24.1 | **56.1** | 17.9 | 17.9% | — | — | — |
| **ByteDance Seed: Seed 2.1 Turbo** | 40.0 | **40.0** | 40.0 | 45.0% | 40.0% | 35.0% | 45.0% |
| **BAAI: bge-large-en-v1.5** | 40.0 | **40.0** | 40.0 | 30.0% | 40.0% | 35.0% | — |
| **OpenAI: GPT Mini Latest** | 44.0 | **44.0** | 44.0 | 49.0% | 44.0% | 39.0% | 49.0% |
| **OpenAI: GPT-5 Mini** | 20.6 | — | — | — | — | — | — |
| **Meta: Muse Glimmer 30B** | 17.5 | **49.0** | 8.5 | 8.5% | — | — | — |
| **Z.ai: GLM 5.2** | 33.7 | **68.8** | 38.4 | 38.4% | — | — | — |
| **MoonshotAI: Kimi K2.6** | 27.0 | **61.8** | 20.5 | 20.5% | — | — | — |
| **OpenAI: GPT Audio Mini** | 44.0 | **44.0** | 44.0 | 49.0% | 44.0% | 39.0% | 49.0% |
| **Amazon: Nova Pro 1.0** | 7.0 | — | — | — | — | — | — |
| **SpaceXAI: Grok 4.20** | 25.7 | — | — | — | — | — | — |
| **DeepSeek: R1** | 11.4 | **24.6** | 1.1 | 1.1% | — | — | — |
| **Z.ai: GLM 5.3 Prime** | 50.0 | **50.0** | 50.0 | 55.0% | 50.0% | 45.0% | — |
| **OpenAI: o3 Mini** | 12.5 | — | — | — | — | — | — |
| **xAI: Grok 4.7** | 46.4 | — | — | — | — | — | — |
| **Upstage: Solar Pro 3** | 7.8 | **16.2** | 1.4 | 1.4% | — | — | — |
| **Google: Gemini 2.5 Flash Lite** | 10.4 | — | — | — | — | — | — |
| **OpenAI: GPT-5.2-Codex** | 28.5 | — | — | — | — | — | — |
| **Google: Gemini Embedding 2** | 44.0 | **44.0** | 44.0 | 34.0% | 44.0% | 39.0% | 49.0% |
| **Qwen: Qwen3 VL 235B A22B Instruct** | 9.9 | — | — | — | — | — | — |
| **OpenAI: GPT-5.2 Chat** | 40.0 | **40.0** | 40.0 | 45.0% | 40.0% | 35.0% | 45.0% |
| **OpenAI: GPT-6 Astra Flex** | 53.0 | **53.0** | 53.0 | 58.0% | 53.0% | 48.0% | 58.0% |
| **OpenAI: GPT-4o-mini (2024-07-18)** | 44.0 | **44.0** | 44.0 | 49.0% | 44.0% | 39.0% | 49.0% |
| **DeepSeek: DeepSeek V4 Flash Vision Exp** | 34.8 | **65.0** | 47.5 | 47.5% | — | — | — |
| **Z.ai: GLM 4.5** | 12.8 | **1430.0** | 1430.0 | — | — | — | — |
| **Z.ai: GLM 4.5V** | 7.6 | — | — | — | — | — | — |
| **OpenAI: o3 Mini High** | 11.0 | **16.3** | 0.9 | 0.9% | — | — | — |
| **OpenAI: GPT-5.6 Luna Pro** | 44.0 | **44.0** | 44.0 | 49.0% | 44.0% | 39.0% | 49.0% |
| **Qwen: Qwen3.5-27B** | 22.9 | — | — | — | — | — | — |
| **Anthropic: Claude Opus 5** | 50.8 | **78.0** | 56.5 | 56.5% | — | — | — |
| **Anthropic: Claude Opus Latest** | 53.0 | **53.0** | 53.0 | 58.0% | 53.0% | 48.0% | 58.0% |
| **OpenAI: GPT-6 Astra Pro** | 53.0 | **53.0** | 53.0 | 58.0% | 53.0% | 48.0% | 58.0% |
| **Anthropic: Claude Opus 4.1** | 22.8 | — | — | — | — | — | — |
| **Sentence Transformers: all-MiniLM-L12-v2** | 44.0 | **44.0** | 44.0 | 34.0% | 44.0% | 39.0% | — |
| **Meta: Muse Wonder** | 45.0 | **45.0** | 45.0 | 50.0% | 45.0% | 40.0% | 50.0% |
| **xAI: Grok 4.6** | 44.3 | **76.8** | 53.0 | 53.0% | — | — | — |
| **Qwen: Qwen3 VL 30B A3B Thinking** | 9.5 | — | — | — | — | — | — |
| **Mistral: Voxtral Small 24B 2507** | 40.0 | **40.0** | 40.0 | 45.0% | 40.0% | 35.0% | 45.0% |
| **DeepSeek: DeepSeek V3.1 Terminus** | 14.8 | **43.5** | 6.8 | 6.8% | — | — | — |
| **OpenAI: GPT-4** | 6.7 | **13.1** | — | — | — | — | — |
| **Qwen: Qwen3.6 Plus** | 27.0 | **54.5** | 1436.4 | — | — | — | — |
| **DeepSeek: DeepSeek V3.1** | 13.7 | — | — | — | — | — | — |
| **OpenAI: GPT-3.5 Turbo** | 5.5 | **10.7** | — | — | — | — | — |
| **Mistral: Ministral 3 3B 2512** | 4.8 | **4.8** | 0.8 | 0.8% | — | — | — |
| **DeepSeek: DeepSeek V4 Flash** | 44.0 | **44.0** | 44.0 | 49.0% | 44.0% | 39.0% | — |
| **Anthropic: Claude Sonnet 4.5** | 20.7 | **52.1** | 15.8 | 15.8% | — | — | — |
| **Qwen: Qwen3.5-Flash** | 44.0 | **44.0** | 44.0 | 49.0% | 44.0% | 39.0% | 49.0% |
| **Anthropic: Claude Fable Latest** | 45.0 | **45.0** | 45.0 | 50.0% | 45.0% | 40.0% | 50.0% |
| **Mistral: Mistral Large 3 2512** | 9.3 | **20.1** | 1.0 | 1.0% | — | — | — |
| **VoyageAI by MongoDB: voyage-4-lite** | 40.0 | **40.0** | 40.0 | 30.0% | 40.0% | 35.0% | — |
| **Qwen: Qwen3 14B** | 8.2 | **13.8** | 0.9 | 0.9% | — | — | — |
| **OpenAI: gpt-oss-20b** | 10.0 | — | — | — | — | — | — |
| **OpenAI: GPT-4o-mini** | 6.7 | **11.4** | — | — | — | — | — |
| **MissingNo** | 40.0 | **40.0** | 40.0 | 45.0% | 40.0% | 35.0% | — |
| **Fireworks: Ember-1** | 45.0 | **45.0** | 45.0 | 50.0% | 45.0% | 40.0% | 50.0% |
| **OpenAI: o3 Pro** | 21.9 | — | — | — | — | — | — |
| **VoyageAI by MongoDB: voyage-code-4** | 40.0 | **40.0** | 40.0 | 30.0% | 40.0% | 35.0% | — |
| **Cohere: Command R+ (08-2024)** | 40.0 | **40.0** | 40.0 | 45.0% | 40.0% | 35.0% | — |
| **Xiaomi: MiMo-V2.6-Pro** | 46.3 | **1490.3** | 1490.3 | — | — | — | — |
| **OpenAI: GPT-5.2 Pro** | 40.0 | **40.0** | 40.0 | 45.0% | 40.0% | 35.0% | 45.0% |
| **Anthropic: Claude Opus 5.5** | 57.6 | — | — | — | — | — | — |
| **PrismML: Ternary Bonsai 2 27B** | 40.0 | **40.0** | 40.0 | 45.0% | 40.0% | 35.0% | 45.0% |
| **Arcee AI: Trinity Large Thinking** | 10.8 | **25.8** | 1.0 | 1.0% | — | — | — |
| **DeepSeek: DeepSeek Flash Latest** | 44.0 | **44.0** | 44.0 | 49.0% | 44.0% | 39.0% | 49.0% |
| **ByteDance Seed: Seed-2.0-Code** | 40.0 | **40.0** | 40.0 | 45.0% | 40.0% | 35.0% | 45.0% |
| **Meta: Llama 3.1 8B Instruct** | 6.9 | **5.4** | — | — | — | — | — |
| **BAAI: bge-m3** | 40.0 | **40.0** | 40.0 | 30.0% | 40.0% | 35.0% | — |
| **Qwen: Qwen3 Coder Flash** | 44.0 | **44.0** | 44.0 | 49.0% | 44.0% | 39.0% | — |
| **OpenAI: Text Embedding 3 Small** | 40.0 | **40.0** | 40.0 | 30.0% | 40.0% | 35.0% | — |
| **OpenAI: GPT-6 Luna** | 38.1 | — | — | — | — | — | — |
| **Qwen: Qwen-Plus** | 45.0 | **45.0** | 45.0 | 50.0% | 45.0% | 40.0% | — |
| **Google: Gemma 4 31B** | 14.7 | **43.4** | 4.2 | 4.2% | — | — | — |
| **Thenlper: GTE-Large** | 40.0 | **40.0** | 40.0 | 30.0% | 40.0% | 35.0% | — |
| **DeepSeek: DeepSeek V4.1 Flash** | 44.0 | **44.0** | 44.0 | 49.0% | 44.0% | 39.0% | 49.0% |
| **Z.ai: GLM 5V Turbo** | 23.5 | **1437.5** | 1437.5 | — | — | — | — |
| **Qwen: Qwen3 235B A22B Thinking 2507** | 12.7 | **22.1** | 1.3 | 1.3% | — | — | — |
| **DeepSeek: R1 0528** | 13.1 | — | — | — | — | — | — |
| **Pareto 26.10 Preview** | 45.0 | **45.0** | 45.0 | 50.0% | 45.0% | 40.0% | 50.0% |
| **DeepSeek: DeepSeek V3** | 40.0 | **40.0** | 40.0 | 45.0% | 40.0% | 35.0% | — |
| **OpenAI: GPT-4o (2024-08-06)** | 7.7 | — | — | — | — | — | — |
| **Google: Gemini Flash Latest** | 44.0 | **44.0** | 44.0 | 49.0% | 44.0% | 39.0% | 49.0% |
| **Anthropic: Claude Opus 4.5** | 29.1 | — | — | — | — | — | — |
| **Claude Haiku 4.5** | 16.9 | **43.9** | 8.0 | 8.0% | — | — | — |
| **Google: Gemini 3.5 Flash** | 33.6 | — | — | — | — | — | — |
| **NVIDIA: Nemotron 3.5 Lightning** | 12.9 | **26.8** | 3.5 | 3.5% | — | — | — |
| **Qwen: Qwen3.6 27B** | 21.4 | **53.7** | 18.5 | 18.5% | — | — | — |
| **DeepSeek: DeepSeek V4 Flash 0423** | 24.4 | **52.0** | 26.3 | 26.3% | — | — | — |
| **SpaceXAI: Grok 4.3** | 24.9 | **42.2** | 15.5 | 15.5% | — | — | — |
| **OpenAI: GPT Luna Latest** | 44.0 | **44.0** | 44.0 | 49.0% | 44.0% | 39.0% | 49.0% |
| **Tencent: Hy4 preview** | 45.0 | **45.0** | 45.0 | 50.0% | 45.0% | 40.0% | — |
| **Z.ai: GLM Flash Latest** | 44.0 | **44.0** | 44.0 | 49.0% | 44.0% | 39.0% | 49.0% |
| **Qwen: Qwen3 VL 8B Instruct** | 7.3 | — | — | — | — | — | — |
| **DeepSeek: DeepSeek V3.2** | 21.5 | **44.2** | — | — | — | — | — |
| **OpenAI: GPT-5 Nano** | 13.0 | — | — | — | — | — | — |
| **OpenAI: GPT-5.1-Codex-Max** | 40.0 | **40.0** | 40.0 | 45.0% | 40.0% | 35.0% | 45.0% |
| **Sentence Transformers: paraphrase-MiniLM-L6-v2** | 44.0 | **44.0** | 44.0 | 34.0% | 44.0% | 39.0% | — |
| **SpaceXAI: Grok Build 0.1** | 27.2 | **51.5** | — | — | — | — | — |
| **OpenAI: GPT-6.1 Sol Pro** | 50.0 | **50.0** | 50.0 | 55.0% | 50.0% | 45.0% | 55.0% |
| **Xiaomi: MiMo-V2.5-Pro** | 26.0 | **60.2** | 21.3 | 21.3% | — | — | — |
| **DeepSeek: DeepSeek V4 Pro 0423** | 30.4 | **59.4** | 26.3 | 26.3% | — | — | — |
| **VoyageAI by MongoDB: voyage-4-large** | 40.0 | **40.0** | 40.0 | 30.0% | 40.0% | 35.0% | — |
| **Claude Fable 5** | 49.6 | **76.5** | 50.7 | 50.7% | — | — | — |
| **ByteDance Seed: Seed-2.0-Mini** | 44.0 | **44.0** | 44.0 | 49.0% | 44.0% | 39.0% | 49.0% |
| **Qwen: Qwen3.8 2.4T A95B** | 39.9 | **71.9** | 50.1 | 50.1% | — | — | — |
| **MiniMax: MiniMax M2.1** | 20.9 | — | — | — | — | — | — |
| **Mistral: Mistral Medium 3** | 9.0 | — | — | — | — | — | — |
| **Anthropic: Claude Fable 5.1** | 53.4 | **81.6** | 57.9 | 57.9% | — | — | — |
| **OpenAI: GPT-6 Sol** | 47.6 | — | — | — | — | — | — |
| **Qwen: Qwen3.7 Plus** | 25.2 | **55.9** | 17.5 | 17.5% | — | — | — |
| **Sakana: Fugu Ultra v2** | 45.0 | **45.0** | 45.0 | 50.0% | 45.0% | 40.0% | 50.0% |
| **Claude Sonnet 5** | 38.2 | **71.5** | 43.6 | 43.6% | — | — | — |
| **Amazon: Nova 2 Lite** | 13.4 | **23.0** | — | — | — | — | — |
| **OpenAI: GPT Chat Latest** | 40.0 | **40.0** | 40.0 | 45.0% | 40.0% | 35.0% | 45.0% |
| **Reka Edge** | 40.0 | **40.0** | 40.0 | 45.0% | 40.0% | 35.0% | 45.0% |
| **Mistral Large 2407** | 7.6 | — | — | — | — | — | — |
| **OpenAI: GPT-4.1 Nano** | 7.8 | **11.1** | — | — | — | — | — |
| **Mistral: Codestral Embed 2505** | 40.0 | **40.0** | 40.0 | 30.0% | 40.0% | 35.0% | — |
| **Qwen: Qwen3.5-35B-A3B** | 19.3 | — | — | — | — | — | — |
| **Qwen: Qwen3.6 Flash** | 44.0 | **44.0** | 44.0 | 49.0% | 44.0% | 39.0% | 49.0% |
| **Google: Gemini 2.5 Pro** | 16.1 | **33.3** | 1.6 | 1.6% | — | — | — |
| **Qwen: Qwen3.5-122B-A10B** | 17.7 | **43.3** | — | — | — | — | — |
| **Claude Opus 4.8** | 41.8 | **74.3** | 41.9 | 41.9% | — | — | — |
| **Google: Gemini Pro Latest** | 44.0 | **44.0** | 44.0 | 49.0% | 44.0% | 39.0% | 49.0% |
| **Amazon: Nova Premier 1.0** | 9.2 | — | — | — | — | — | — |
| **OpenAI: GPT-5.4 Pro** | 45.0 | **45.0** | 45.0 | 50.0% | 45.0% | 40.0% | 50.0% |
| **StepFun: Step 3.7 Flash** | 19.5 | **39.6** | — | — | — | — | — |
| **OpenAI: GPT-5.1-Codex-Mini** | 20.4 | — | — | — | — | — | — |
| **Mistral: Mistral Medium 3.5** | 14.2 | **46.9** | 7.0 | 7.0% | — | — | — |
| **Z.ai: GLM 4.6** | 18.5 | **45.8** | 1439.9 | — | — | — | — |
| **MiniMax: MiniMax M1** | 11.7 | — | — | — | — | — | — |
| **Mistral: Mistral Nemo** | 40.0 | **40.0** | 40.0 | 45.0% | 40.0% | 35.0% | — |
| **OpenAI: GPT-5.6 Sol** | 47.0 | **77.4** | 50.2 | 50.2% | — | — | — |
| **Mistral: Ministral 3 8B 2512** | 5.5 | **9.7** | 0.6 | 0.6% | — | — | — |
| **Meta: Muse Spark 1.1** | 33.7 | **71.3** | 25.8 | 25.8% | — | — | — |
| **OpenAI: GPT-5** | 23.0 | **37.8** | — | — | — | — | — |
| **Meta: Llama 3.3 70B Instruct** | 7.7 | **11.9** | — | — | — | — | — |
| **Mistral: Mistral Large 4** | 45.0 | **45.0** | 45.0 | 50.0% | 45.0% | 40.0% | 50.0% |
| **AionLabs: Aion 3.5** | 40.0 | **40.0** | 40.0 | 45.0% | 40.0% | 35.0% | — |
| **MoonshotAI: Kimi K2.5** | 23.5 | **46.8** | — | — | — | — | — |
| **Qwen: Qwen3.5 Plus 2026-04-20** | 45.0 | **45.0** | 45.0 | 50.0% | 45.0% | 40.0% | 50.0% |
| **OpenAI: GPT-5.4 Nano** | 20.7 | **56.1** | 16.0 | 16.0% | — | — | — |
| **Mistral: Codestral 2508** | 40.0 | **40.0** | 40.0 | 45.0% | 40.0% | 35.0% | — |
| **Z.ai: GLM 5.3 Flash** | 44.0 | **44.0** | 44.0 | 49.0% | 44.0% | 39.0% | 49.0% |
| **OpenAI: GPT-4 Turbo** | 7.0 | **21.5** | — | — | — | — | — |
| **Sentence Transformers: all-MiniLM-L6-v2** | 44.0 | **44.0** | 44.0 | 34.0% | 44.0% | 39.0% | — |
| **Perceptron: Perceptron Mk1.5** | 40.0 | **40.0** | 40.0 | 45.0% | 40.0% | 35.0% | 45.0% |
| **Qwen: Qwen3.8 Omni Flash** | 44.0 | **44.0** | 44.0 | 49.0% | 44.0% | 39.0% | 49.0% |
| **Qwen: Qwen3.8 27B** | 33.7 | **68.1** | 45.8 | 45.8% | — | — | — |
| **NVIDIA: Nemotron 3 Nano 30B A3B** | 8.9 | **14.4** | 1.0 | 1.0% | — | — | — |
| **Google: Gemini 2.5 Flash** | 15.5 | — | — | — | — | — | — |
| **Meta: Llama 3.1 70B Instruct** | 6.6 | — | — | — | — | — | — |
| **MoonshotAI: Kimi Latest** | 45.0 | **45.0** | 45.0 | 50.0% | 45.0% | 40.0% | 50.0% |
| **Qwen: Qwen3.7 Flash** | 44.0 | **44.0** | 44.0 | 49.0% | 44.0% | 39.0% | 49.0% |
| **OpenAI: GPT-6 Luna Pro** | 44.0 | **44.0** | 44.0 | 49.0% | 44.0% | 39.0% | 49.0% |
| **OpenAI: GPT-3.5 Turbo (older v0613)** | 40.0 | **40.0** | 40.0 | 45.0% | 40.0% | 35.0% | — |
| **VoyageAI by MongoDB: voyage-4** | 40.0 | **40.0** | 40.0 | 30.0% | 40.0% | 35.0% | — |
| **NVIDIA: Nemotron 3 Ultra** | 22.9 | **49.3** | 20.1 | 20.1% | — | — | — |
| **MiniMax: MiniMax M2.7** | 22.8 | **52.6** | 15.3 | 15.3% | — | — | — |
| **Amazon: Nova Lite 1.0** | 6.7 | — | — | — | — | — | — |
| **LiquidAI: LFM2.5-24B-A2B** | 40.0 | **40.0** | 40.0 | 45.0% | 40.0% | 35.0% | — |
| **DeepSeek: DeepSeek V4 Flash** | 34.3 | **69.1** | 41.0 | 41.0% | — | — | — |
| **Google: Gemini 3 Flash Preview** | 26.3 | **1465.8** | 1465.8 | — | — | — | — |
| **Mistral: Ministral 3 14B 2512** | 6.0 | **14.4** | 1.1 | 1.1% | — | — | — |
| **Xiaomi: MiMo-V2.5** | 25.2 | **56.8** | 15.8 | 15.8% | — | — | — |
| **DeepSeek: DeepSeek V3 0324** | 9.7 | **21.2** | 0.8 | 0.8% | — | — | — |
| **Amazon: Nova Micro 1.0** | 5.9 | — | — | — | — | — | — |
| **Mistral: Mixtral 8x22B Instruct** | 5.7 | — | — | — | — | — | — |
| **Mistral: Mistral Small 3.1 24B** | 40.0 | **40.0** | 40.0 | 45.0% | 40.0% | 35.0% | 45.0% |
| **Meta: Muse Spark 1.3** | 48.1 | **75.8** | 55.5 | 55.5% | — | — | — |
| **Upstage: Solar Mini 4** | 24.1 | — | — | — | — | — | — |
| **Z.ai: GLM 5** | 27.9 | **1446.1** | 1446.1 | — | — | — | — |
| **Google: Gemini 2.5 Pro Preview 06-05** | 44.0 | **44.0** | 44.0 | 49.0% | 44.0% | 39.0% | 49.0% |
| **Z.ai: GLM 4.5 Air** | 11.1 | — | — | — | — | — | — |
| **OpenAI: o3** | 20.2 | — | — | — | — | — | — |
| **MoonshotAI: Kimi K2 Thinking** | 22.0 | — | — | — | — | — | — |
| **xAI: Grok Latest** | 45.0 | **45.0** | 45.0 | 50.0% | 45.0% | 40.0% | 50.0% |
| **Relace: Relace Search** | 40.0 | **40.0** | 40.0 | 45.0% | 40.0% | 35.0% | — |
| **OpenAI: GPT-5.6 Terra Pro** | 45.0 | **45.0** | 45.0 | 50.0% | 45.0% | 40.0% | 50.0% |
| **AionLabs: Aion-3.0** | 40.0 | **40.0** | 40.0 | 45.0% | 40.0% | 35.0% | — |
| **Qwen: Qwen3 Next 80B A3B Instruct** | 9.6 | — | — | — | — | — | — |
| **OpenAI: o4 Mini High** | 44.0 | **44.0** | 44.0 | 49.0% | 44.0% | 39.0% | 49.0% |
| **Sakana: Fugu Ultra** | 45.0 | **45.0** | 45.0 | 50.0% | 45.0% | 40.0% | 50.0% |
| **Qwen: Qwen3 Embedding 4B** | 40.0 | **40.0** | 40.0 | 30.0% | 40.0% | 35.0% | — |
| **OpenAI: GPT-5.5 Pro** | 45.0 | **45.0** | 45.0 | 50.0% | 45.0% | 40.0% | 50.0% |
| **Z.ai: GLM 4.6V** | 11.2 | — | — | — | — | — | — |
| **DeepSeek: DeepSeek V3.2 Exp** | 16.6 | — | — | — | — | — | — |
| **StepFun: Step 3.5 Flash** | 17.0 | — | — | — | — | — | — |
| **MoonshotAI: Kimi K2 0905** | 15.3 | — | — | — | — | — | — |
| **inclusionAI: Ling 3.0 Flash** | 20.1 | **50.6** | 19.3 | 19.3% | — | — | — |
| **OpenAI: GPT-4o** | 40.0 | **40.0** | 40.0 | 45.0% | 40.0% | 35.0% | 45.0% |
| **OpenAI: GPT-6.1 Sol** | 51.8 | — | — | — | — | — | — |
| **inclusionAI: Ling 3.0 Flash VL** | 24.6 | **57.0** | 28.7 | 28.7% | — | — | — |
| **Qwen2.5 72B Instruct** | 7.7 | — | — | — | — | — | — |
| **Thenlper: GTE-Base** | 40.0 | **40.0** | 40.0 | 30.0% | 40.0% | 35.0% | — |
| **Google: Gemini 3.7 Flash** | 39.6 | **71.5** | — | — | — | — | — |
| **Meta: Muse Spark 1.2** | 39.6 | **72.2** | 43.2 | 43.2% | — | — | — |
| **Xiaomi: MiMo-V2.6-Pro-UltraSpeed** | 45.0 | **45.0** | 45.0 | 50.0% | 45.0% | 40.0% | 50.0% |
| **OpenAI: GPT-5.1** | 24.7 | **49.4** | — | — | — | — | — |
| **Qwen: Qwen3 Coder 480B A35B** | 11.9 | — | — | — | — | — | — |
| **DeepSeek: DeepSeek V4 Pro 0813** | 36.0 | **68.8** | 41.3 | 41.3% | — | — | — |
| **OpenAI: GPT-5.6 Terra** | 42.1 | **76.7** | 43.2 | 43.2% | — | — | — |
| **Mistral: Mistral Small 4** | 11.3 | **26.6** | 0.8 | 0.8% | — | — | — |
| **inclusionAI: Ling 3.1 Flash** | 41.1 | — | — | — | — | — | — |
| **NVIDIA: Nemotron 3 Super** | 12.8 | **37.7** | 1.7 | 1.7% | — | — | — |
| **xAI: Grok 4.5** | 38.8 | **72.4** | 41.2 | 41.2% | — | — | — |
| **Z.ai: GLM 4.7 Flash** | 14.9 | — | — | — | — | — | — |
| **Upstage: Solar Pro 4** | 28.2 | **52.7** | — | — | — | — | — |
| **OpenAI: GPT-4o (2024-11-20)** | 8.4 | — | — | — | — | — | — |
| **Qwen: Qwen3.8 Max Prime** | 50.0 | **50.0** | 50.0 | 55.0% | 50.0% | 45.0% | 55.0% |
| **Z.ai: GLM 5.3** | 45.0 | **45.0** | 45.0 | 50.0% | 45.0% | 40.0% | — |
| **Mistral: Mistral Embed 2312** | 40.0 | **40.0** | 40.0 | 30.0% | 40.0% | 35.0% | — |
| **OpenAI: GPT Audio** | 40.0 | **40.0** | 40.0 | 45.0% | 40.0% | 35.0% | 45.0% |
| **Cohere: Command R (08-2024)** | 40.0 | **40.0** | 40.0 | 45.0% | 40.0% | 35.0% | — |

---

## 🛡️ 6. DIRETRIZES DE FINOPS & MATRIZ DE DECISÃO LOCAL VS CLOUD

1. **Regra de Ouro do Silício Local:** Tarefas de desidratação AST, triagem de entropia, inspeção de tipos e refatorações cirúrgicas de arquivo único pertencem **100% à dGPU RTX 2060m e CPU Host i9** (custo zero de nuvem).
2. **Critérios de Escalação para Nuvem:**
   - **Alta Entropia Cognitiva ($H > 0.85$):** Questões abertas de arquitetura de software demandam `Claude 3.7 Sonnet`, `DeepSeek R1` ou `Gemini 4 Argon`.
   - **Estouro de Raio de Impacto (Blast Radius > 5 arquivos):** Refatorações cross-crate devem delegar a síntese para a Nuvem.
   - **Janela Gigante (> 32k tokens):** Contextos muito extensos devem utilizar o prefix caching do `DeepSeek V3` ou `Gemini 2.5 Flash`.
3. **Prefix Caching Obrigatório:** Sempre priorize provedores com suporte a cache read com desconto $\ge 75\%$ em sessões multi-turn longas.
