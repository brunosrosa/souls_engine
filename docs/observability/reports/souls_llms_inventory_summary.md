# 📊 SOULS SILICON OBSERVABILITY & HERMES LLM DOSSIER (v7 Canônica)
> **Gerado em:** 2026-10-10 00:05:15 | **Banco SSOT:** `Z:\souls_engine\.souls_data\db\souls_heuristic_vault.db`
> **Host Físico:** Intel Core i9-9880H (AVX2) | 32GB RAM | NVIDIA GeForce RTX 2060m (6GB GDDR6, WDDM 3.x)
> **Governança:** ADR-004 (Teto Útil WDDM 5.2GB) & ADR-007 (Desacoplamento dos Benchmark Types 1 a 5)

---

## 🏆 1. LEADERBOARD EXECUTIVO TIER 1: PRONTIDÃO PARA O HERMES AGENT
Esta matriz classifica exclusivamente os modelos de **Tier 1** (Live Chat, Coding e Tool-Calling) capazes de acelerar localmente o Hermes Agent sem colapso de VRAM.

| Rank | Modelo / Identificador | Params | Quant | Throughput (TPS) | TTFT (ms) | Pico VRAM | Headroom VRAM | Score E³ | Qualificação Hermes Agent |
| :---: | :--- | :---: | :---: | :---: | :---: | :---: | :---: | :---: | :--- |
| 1 | `qwen3-5-4b-uncensored-hauhaucs-aggressive-mtp-q4-k-m` | 4B | `Q4_K_M` | **51.2** | 254.3 | 4727 MB | 567 MB | 9999.00 | 🟢 CANDIDATO PRINCIPAL HERMES |
| 2 | `qwen3-5-4b-super-coder-q4-0` | 4B | `Q4_0` | **50.1** | 216.5 | 4727 MB | 567 MB | 9999.00 | 🟢 CANDIDATO PRINCIPAL HERMES |
| 3 | `nvidia-nemotron-3-nano-4b-q4-k-m` | 4B | `Q4_K_M` | **47.8** | 190.9 | 2829 MB | 2465 MB | 4699.25 | 🟢 CANDIDATO PRINCIPAL HERMES |
| 4 | `arch-function-chat-3b-q5-k-m` | 3B | `Q5_K_M` | **47.1** | 229.1 | 4734 MB | 560 MB | 9999.00 | 🟢 CANDIDATO PRINCIPAL HERMES |
| 5 | `qwen3-5-4b-agentic-coder-v4-i1-q4-k-s` | 4B | `I1-Q4_K_S` | **46.6** | 241.2 | 4724 MB | 570 MB | 9999.00 | 🟢 CANDIDATO PRINCIPAL HERMES |
| 6 | `huggingfacetb-smollm3-3b-q5-k-m` | 3B | `Q5_K_M` | **45.8** | 248.1 | 4726 MB | 568 MB | 9999.00 | 🟢 CANDIDATO PRINCIPAL HERMES |
| 7 | `falcon3-mamba-7b-instruct-i1-iq3-m` | 7B | `IQ3_M` | **42.5** | 18745.6 | 4726 MB | 568 MB | 9999.00 | 🟢 CANDIDATO PRINCIPAL HERMES |
| 8 | `hy-mt2-7b-q4-k-m` | 7B | `Q4_K_M` | **42.5** | 348.1 | 4726 MB | 568 MB | 9999.00 | 🟢 CANDIDATO PRINCIPAL HERMES |
| 9 | `mamba-codestral-7b-instruct-agnuxo` | 7B | `Q4_K_M` | **42.5** | 256.5 | 4726 MB | 568 MB | 9999.00 | 🔵 SUBAGENTE DE SUPORTE |
| 10 | `mamba-codestral-7b-v0-1-q4-k-m` | 7B | `Q4_K_M` | **42.5** | 234.6 | 4730 MB | 564 MB | 9999.00 | 🟢 CANDIDATO PRINCIPAL HERMES |
| 11 | `parable-qwen3-4b-claude-fable-5-gguf-q4-k-m` | 4B | `Q4_K_M` | **42.5** | 254.7 | 4726 MB | 568 MB | 9999.00 | 🟢 CANDIDATO PRINCIPAL HERMES |
| 12 | `phi-4-mini-instruct-q4-k-m` | 3.8B | `Q4_K_M` | **42.5** | 51831.1 | 4726 MB | 568 MB | 9999.00 | 🟢 CANDIDATO PRINCIPAL HERMES |
| 13 | `phi-4-mini-instruct-q5-k-m` | 3.8B | `Q5_K_M` | **42.5** | 264.4 | 4726 MB | 568 MB | 9999.00 | 🟢 CANDIDATO PRINCIPAL HERMES |
| 14 | `phi-4-mini-reasoning-q4-k-m` | 3.8B | `Q4_K_M` | **42.5** | 259.5 | 4726 MB | 568 MB | 9999.00 | 🟢 CANDIDATO PRINCIPAL HERMES |
| 15 | `synthagent-sft-ui-tars-1-5-7b-i1-iq3-m` | 7B | `IQ3_M` | **42.5** | 249.0 | 4724 MB | 570 MB | 9999.00 | 🟢 CANDIDATO PRINCIPAL HERMES |
| 16 | `zamba2-2-7b-instruct-v2-q4-0` | 2.7B | `Q4_0` | **42.5** | 117.7 | 4724 MB | 570 MB | 9999.00 | 🟢 CANDIDATO PRINCIPAL HERMES |

> **Critério de Ouro FinOps:** Modelos com mais de **35 tokens/s** e VRAM inferior a **4.200 MB** deixam margem folgada para até **8k tokens de contexto**, sendo os candidatos ideais para a experiência mestre do Hermes Agent.

---

## 🖥️ 2. INVENTÁRIO COMPLETO DE SILÍCIO POR TIER (38 Modelos)
Apresentação canônica e sequencial de todos os modelos locais categorizados por Tier Físico e ordenados por throughput decrescente dentro de cada camada:

### 🎯 Tier 0 — Bootstrap, Sanity & Fast Heuristic Triage
> **Papel Arquitetural:** Classificação preliminar de intenção, triagem de prompts, bootstrap ultra-rápido (<15ms) e verificações de sanidade a frio sem alocação de VRAM.
> **Hardware Alvo & Limites:** CPU Host Intel Core i9 (extensões AVX2) | Modelos < 350 MB (135M parâmetros ou ONNX) | 0 MB VRAM dGPU | Meta: Throughput > 100 TPS.
> **Motores Homologados:** `onnx-cpu` (ONNX Runtime) e `llama-cpu` (`-ngl 0`).

| # | Identificador | Params | Quant | Tamanho | Motor | TPS | TTFT (ms) | Pico VRAM | Score E³ | Status Operacional |
| :---: | :--- | :---: | :---: | :---: | :---: | :---: | :---: | :---: | :---: | :--- |
| 1 | `smollm2-135m-reasoning-5k-q4-k-m` | 135M | `Q4_K_M` | 100.6 MB | `llama-cpu` | 149.5 | 357.4 | 910 MB | 9999.00 | 🟢 CANDIDATO PRINCIPAL HERMES |
| 2 | `smollm-135m-instruct-q4-k-m` | 135M | `Q4_K_M` | 100.6 MB | `llama-cpu` | 111.2 | 305.9 | 910 MB | 9999.00 | 🟢 CANDIDATO PRINCIPAL HERMES |
| 3 | `neuralai-mamba-k1-v3-q4-k-m` | 135M | `Q4_K_M` | 85.7 MB | `llama-cpu` | 83.7 | 352.3 | 913 MB | 9999.00 | 🟢 CANDIDATO PRINCIPAL HERMES |
| 4 | `qwen35-4b-dflash-q4-k-m` | 4B | `Q4_K_M` | 4.0 MB | `llama-cuda` | 8.2 | 894.5 | 924 MB | 75.05 | 🟠 CADÊNCIA LENTA |

### 🎯 Tier 0.5 — Epistemic Probing & SLM Host Sensors
> **Papel Arquitetural:** Sensor epistêmico de incerteza (entropia de Shannon), extração direta de logits (forward-only prefill sem geração recursiva) e triagens heurísticas leves.
> **Hardware Alvo & Limites:** CPU Host Intel i9 AVX2 | 0 MB VRAM dGPU (estritamente RAM do sistema) | Modelos SLM de 360M a 2B parâmetros | Latência de probe < 150 ms.
> **Motores Homologados:** `llama-cpu` (`-ngl 0 -t 8`), BitNet 1.58b, SmolLM2-360M.

| # | Identificador | Params | Quant | Tamanho | Motor | TPS | TTFT (ms) | Pico VRAM | Score E³ | Status Operacional |
| :---: | :--- | :---: | :---: | :---: | :---: | :---: | :---: | :---: | :---: | :--- |
| 1 | `bitnet-b1.58-2b-4t` | 2B | `I2_S` | 1.11 GB | `llama-cpu` | 65.0 | 605.1 | 1143 MB | 7499.61 | 🟢 CANDIDATO PRINCIPAL HERMES |
| 2 | `smollm2-360m-instruct-q8-0` | 360M | `Q8_0` | 368.5 MB | `llama-cpu` | 51.6 | 466.2 | 1117 MB | 5925.90 | 🟢 CANDIDATO PRINCIPAL HERMES |
| 3 | `mamba-790m-hf-q4-k-m` | 790M | `Q4_K_M` | 459.3 MB | `llama-cpu` | 24.2 | 780.9 | 1117 MB | 757.51 | 🔵 SUBAGENTE DE SUPORTE |
| 4 | `lfm2-5-1-2b-instruct-q8-0` | 1.2B | `Q8_0` | 1.16 GB | `llama-cpu` | 15.6 | 994.6 | 1114 MB | 251.55 | 🟠 CADÊNCIA LENTA |
| 5 | `deepseek-r1-distill-qwen-1-5b-q8-0` | 1.5B | `Q8_0` | 1.76 GB | `llama-cpu` | 13.4 | 1265.1 | 1138 MB | 145.66 | 🟠 CADÊNCIA LENTA |

### 🎯 Tier 1 — Master Interactive Chat, Coding & Autonomous Tool-Calling
> **Papel Arquitetural:** Cérebro executivo local e motor principal para Live Chat, geração de código Rust AST e despacho estruturado de ferramentas MCP para o Hermes Agent.
> **Hardware Alvo & Limites:** dGPU NVIDIA GeForce RTX 2060m (teto seguro WDDM de 5.294 MB de VRAM) | Modelos de 2.7B a 7B (quantizações Q4_0, Q4_K_M, IQ3_M) | KV Cache Assimétrico (Key=FP16, Value=Q4_0) com FlashAttention-2 | Meta: TPS ≥ 30 tok/s e TTFT < 2.500 ms.
> **Motores Homologados:** `llama-cuda` (`-ngl 99 -ctk f16 -ctv q4_0 -fa on`).

| # | Identificador | Params | Quant | Tamanho | Motor | TPS | TTFT (ms) | Pico VRAM | Score E³ | Status Operacional |
| :---: | :--- | :---: | :---: | :---: | :---: | :---: | :---: | :---: | :---: | :--- |
| 1 | `qwen3-5-4b-uncensored-hauhaucs-aggressive-mtp-q4-k-m` | 4B | `Q4_K_M` | 2.60 GB | `llama-cuda` | 51.2 | 254.3 | 4727 MB | 9999.00 | 🟢 CANDIDATO PRINCIPAL HERMES |
| 2 | `qwen3-5-4b-super-coder-q4-0` | 4B | `Q4_0` | 2.43 GB | `llama-cuda` | 50.1 | 216.5 | 4727 MB | 9999.00 | 🟢 CANDIDATO PRINCIPAL HERMES |
| 3 | `nvidia-nemotron-3-nano-4b-q4-k-m` | 4B | `Q4_K_M` | 2.64 GB | `llama-cuda` | 47.8 | 190.9 | 2829 MB | 4699.25 | 🟢 CANDIDATO PRINCIPAL HERMES |
| 4 | `arch-function-chat-3b-q5-k-m` | 3B | `Q5_K_M` | 2.07 GB | `llama-cuda` | 47.1 | 229.1 | 4734 MB | 9999.00 | 🟢 CANDIDATO PRINCIPAL HERMES |
| 5 | `qwen3-5-4b-agentic-coder-v4-i1-q4-k-s` | 4B | `I1-Q4_K_S` | 2.45 GB | `llama-cuda` | 46.6 | 241.2 | 4724 MB | 9999.00 | 🟢 CANDIDATO PRINCIPAL HERMES |
| 6 | `huggingfacetb-smollm3-3b-q5-k-m` | 3B | `Q5_K_M` | 2.06 GB | `llama-cuda` | 45.8 | 248.1 | 4726 MB | 9999.00 | 🟢 CANDIDATO PRINCIPAL HERMES |
| 7 | `falcon3-mamba-7b-instruct-i1-iq3-m` | 7B | `IQ3_M` | 3.05 GB | `llama-cuda` | 42.5 | 18745.6 | 4726 MB | 9999.00 | 🟢 CANDIDATO PRINCIPAL HERMES |
| 8 | `hy-mt2-7b-q4-k-m` | 7B | `Q4_K_M` | 4.31 GB | `llama-cuda` | 42.5 | 348.1 | 4726 MB | 9999.00 | 🟢 CANDIDATO PRINCIPAL HERMES |
| 9 | `mamba-codestral-7b-instruct-agnuxo` | 7B | `Q4_K_M` | 4.07 GB | `llama-cuda` | 42.5 | 256.5 | 4726 MB | 9999.00 | 🔵 SUBAGENTE DE SUPORTE |
| 10 | `mamba-codestral-7b-v0-1-q4-k-m` | 7B | `Q4_K_M` | 3.86 GB | `llama-cuda` | 42.5 | 234.6 | 4730 MB | 9999.00 | 🟢 CANDIDATO PRINCIPAL HERMES |
| 11 | `parable-qwen3-4b-claude-fable-5-gguf-q4-k-m` | 4B | `Q4_K_M` | 2.33 GB | `llama-cuda` | 42.5 | 254.7 | 4726 MB | 9999.00 | 🟢 CANDIDATO PRINCIPAL HERMES |
| 12 | `phi-4-mini-instruct-q4-k-m` | 3.8B | `Q4_K_M` | 2.32 GB | `llama-cuda` | 42.5 | 51831.1 | 4726 MB | 9999.00 | 🟢 CANDIDATO PRINCIPAL HERMES |
| 13 | `phi-4-mini-instruct-q5-k-m` | 3.8B | `Q5_K_M` | 2.65 GB | `llama-cuda` | 42.5 | 264.4 | 4726 MB | 9999.00 | 🟢 CANDIDATO PRINCIPAL HERMES |
| 14 | `phi-4-mini-reasoning-q4-k-m` | 3.8B | `Q4_K_M` | 2.32 GB | `llama-cuda` | 42.5 | 259.5 | 4726 MB | 9999.00 | 🟢 CANDIDATO PRINCIPAL HERMES |
| 15 | `synthagent-sft-ui-tars-1-5-7b-i1-iq3-m` | 7B | `IQ3_M` | 3.33 GB | `llama-cuda` | 42.5 | 249.0 | 4724 MB | 9999.00 | 🟢 CANDIDATO PRINCIPAL HERMES |
| 16 | `zamba2-2-7b-instruct-v2-q4-0` | 2.7B | `Q4_0` | 2.08 GB | `llama-cuda` | 42.5 | 117.7 | 4724 MB | 9999.00 | 🟢 CANDIDATO PRINCIPAL HERMES |

### 🎯 Tier 2 — Background Heavyweight & Deep Cognitive Reasoning
> **Papel Arquitetural:** Raciocínio profundo assíncrono (deep CoT), arquiteturas Mixture-of-Experts (MoE) ou modelos de alta capacidade (>14B / 27B) que excedem a VRAM da dGPU.
> **Hardware Alvo & Limites:** 32GB RAM DDR4 Host via AVX2 / CPU Pinned (0 MB VRAM na GPU para evitar spillover PCIe forçado pelo Windows) | Operação assíncrona em background via delegate_task.
> **Motores Homologados:** `llama-cpu` (`-ngl 0 -t 8`), rnj-1 (8B Gemma 3), laguna-xs (16B MoE).

| # | Identificador | Params | Quant | Tamanho | Motor | TPS | TTFT (ms) | Pico VRAM | Score E³ | Status Operacional |
| :---: | :--- | :---: | :---: | :---: | :---: | :---: | :---: | :---: | :---: | :--- |
| 1 | `laguna-xs-2-1-apex-i-compact` | 16B MoE | `GGUF` | 14.69 GB | `llama-cpu` | 5.8 | 37394.6 | 931 MB | 0.89 | 🟠 CADÊNCIA LENTA |
| 2 | `rnj-1-instruct-q4-k-m` | 8B | `Q4_K_M` | 4.76 GB | `llama-cpu` | 3.6 | 3729.8 | 910 MB | 3.46 | 🟠 CADÊNCIA LENTA |

### 🎯 Tier 3 — Multimodal Vision-Language Models & UI Grounding
> **Papel Arquitetural:** Inspeção visual de interfaces, OCR técnico de diagramas e captura multimodal via projetores mmproj pareados.
> **Hardware Alvo & Limites:** dGPU NVIDIA RTX 2060m com projetores acoplados via --mmproj | Modelos VLM de 2B a 7B (Qwen2-VL, Gemma-4-E2B-IT, UI-TARS).
> **Motores Homologados:** `llama-cuda` (`-ngl 99 --mmproj <sidecar_path>`).

| # | Identificador | Params | Quant | Tamanho | Motor | TPS | TTFT (ms) | Pico VRAM | Score E³ | Status Operacional |
| :---: | :--- | :---: | :---: | :---: | :---: | :---: | :---: | :---: | :---: | :--- |
| 1 | `microsoft-fara1-5-4b-f16-gguf-q4-k-m` | 4B | `Q4_K_M` | 2.52 GB | `llama-cuda` | 18.2 | 3135.8 | 2619 MB | 105.33 | 🟠 CADÊNCIA LENTA |
| 2 | `deepseek-v4-pro-qwen3-5-4b-mtp-q4-k-s` | 4B | `Q4_K_S` | 2.45 GB | `llama-cuda` | 16.5 | 2873.8 | 2546 MB | 94.36 | 🟠 CADÊNCIA LENTA |
| 3 | `nimbus-4b-q4-k-m` | 4B | `Q4_K_M` | 2.52 GB | `llama-cuda` | 16.3 | 3053.8 | 2619 MB | 86.58 | 🟠 CADÊNCIA LENTA |
| 4 | `gemma-4-e2b-it-ultra-uncensored-heretic-q4-k-m` | 2B | `Q4_K_M` | 3.19 GB | `llama-cuda` | 15.8 | 3093.0 | 3305 MB | 81.11 | 🟠 CADÊNCIA LENTA |
| 5 | `microsoft-fara-7b-iq3-m` | 7B | `IQ3_M` | 3.33 GB | `llama-cuda` | 13.2 | 3902.2 | 3444 MB | 44.91 | 🟠 CADÊNCIA LENTA |
| 6 | `bonsai-27b-q1-0-lmstudio` | 27B | `Q1_0` | 3.54 GB | `llama-cuda` | 8.3 | 5092.8 | 3663 MB | 13.56 | 🟠 CADÊNCIA LENTA |
| 7 | `bonsai-27b-q1-0-ai-joe` | 27B | `Q1_0` | 3.54 GB | `llama-cuda` | 5.2 | 8184.0 | 3663 MB | 3.27 | 🟠 CADÊNCIA LENTA |
| 8 | `gemma-4-e2b-it-q4-k-m` | 2B | `Q4_K_M` | 3.19 GB | `llama-cuda` | 5.1 | 9659.0 | 3305 MB | 2.70 | 🟠 CADÊNCIA LENTA |

### 🎯 Tier 4 — Speculative Decoding Drafters & Acceleration Sparks
> **Papel Arquitetural:** Modelos compactos (drafters) executados em conjunto com o modelo mestre para especular múltiplos tokens por passo e acelerar a geração via taxa de aceitação α.
> **Hardware Alvo & Limites:** Modelos ultracompactos (300M a 1.6GB ou heads MTP) co-alocados na VRAM da dGPU ou RAM | Meta: Taxa de aceitação α ≥ 55% para speedup de 1.4x a 2.1x.
> **Motores Homologados:** `llama-cuda` (`-md <drafter.gguf>`).

| # | Identificador | Params | Quant | Tamanho | Motor | TPS | TTFT (ms) | Pico VRAM | Score E³ | Status Operacional |
| :---: | :--- | :---: | :---: | :---: | :---: | :---: | :---: | :---: | :---: | :--- |
| 1 | `bonsai-27b-dspark-q4-1` | 27B | `Q4_1` | 1.66 GB | `llama-cuda` | 62.2 | 117.2 | 1741 MB | 9999.00 | 🟢 CANDIDATO PRINCIPAL HERMES |
| 2 | `laguna-xs-2-1-dspark-q4-k-m` | 300M | `Q4_K_M` | 297.6 MB | `llama-cuda` | 20.3 | 361.5 | 927 MB | 1134.82 | 🔵 SUBAGENTE DE SUPORTE |
| 3 | `qwen35-4b-dflash-q4-k-m` | 4B | `Q4_K_M` | 363.8 MB | `llama-cuda` | 8.2 | 894.5 | 924 MB | 75.05 | 🟠 CADÊNCIA LENTA |

---

## 🧬 3. COLISEU COGNITIVO (BENCHMARK TYPE 2: QUALIDADE & RIGIDEZ)
Aferição de integridade lógica necessária para subagentes executivos e MCP Tools:

| Modelo | Tier | 🦀 Track 2A (Rust AST Code) | 🛠️ Track 2B (MCP JSON Tools) | 🧠 Track 2C (CoT Reasoning) | Veredito Cognitivo |
| :--- | :---: | :---: | :---: | :---: | :--- |
| `qwen3-5-4b-uncensored-hauhaucs-aggressive-mtp-q4-k-m` | Tier 1 | 95% | 85% | 90% | 🟢 Aprovado em Sintaxe Estrita |
| `qwen3-5-4b-super-coder-q4-0` | Tier 1 | 95% | 85% | 90% | 🟢 Aprovado em Sintaxe Estrita |
| `nvidia-nemotron-3-nano-4b-q4-k-m` | Tier 1 | 95% | 10% | 90% | 🟢 Aprovado em Sintaxe Estrita |
| `arch-function-chat-3b-q5-k-m` | Tier 1 | 95% | 10% | 90% | 🟢 Aprovado em Sintaxe Estrita |
| `qwen3-5-4b-agentic-coder-v4-i1-q4-k-s` | Tier 1 | 95% | 85% | 90% | 🟢 Aprovado em Sintaxe Estrita |
| `huggingfacetb-smollm3-3b-q5-k-m` | Tier 1 | 95% | 85% | 90% | 🟢 Aprovado em Sintaxe Estrita |
| `falcon3-mamba-7b-instruct-i1-iq3-m` | Tier 1 | 95% | 10% | 90% | 🟢 Aprovado em Sintaxe Estrita |
| `hy-mt2-7b-q4-k-m` | Tier 1 | 20% | 85% | 90% | 🟢 Aprovado em Sintaxe Estrita |
| `mamba-codestral-7b-instruct-agnuxo` | Tier 1 | 20% | 10% | 90% | 🔴 Falha em Validação Estrita |
| `mamba-codestral-7b-v0-1-q4-k-m` | Tier 1 | 95% | 85% | 90% | 🟢 Aprovado em Sintaxe Estrita |
| `parable-qwen3-4b-claude-fable-5-gguf-q4-k-m` | Tier 1 | 95% | 85% | 90% | 🟢 Aprovado em Sintaxe Estrita |
| `phi-4-mini-instruct-q4-k-m` | Tier 1 | 95% | 85% | 90% | 🟢 Aprovado em Sintaxe Estrita |
| `phi-4-mini-instruct-q5-k-m` | Tier 1 | 20% | 85% | 90% | 🟢 Aprovado em Sintaxe Estrita |
| `phi-4-mini-reasoning-q4-k-m` | Tier 1 | 95% | 10% | 90% | 🟢 Aprovado em Sintaxe Estrita |
| `synthagent-sft-ui-tars-1-5-7b-i1-iq3-m` | Tier 1 | 95% | 10% | 90% | 🟢 Aprovado em Sintaxe Estrita |
| `zamba2-2-7b-instruct-v2-q4-0` | Tier 1 | 95% | 100% | 90% | 🟢 Aprovado em Sintaxe Estrita |

---

## 🎯 4. PRESSÃO DE CONTEXTO & RESISTÊNCIA A DECAIMENTO (BENCHMARK TYPE 5) (16 Avaliados)
Aferição física de recuperação de agulha (*Needle-in-a-Haystack*) injetada a 50% de profundidade sob dGPU CUDA:

| Modelo | Contexto Testado | Contexto Efetivo Recuperado | Status Agulha | Veredito de Atenção |
| :--- | :---: | :---: | :---: | :--- |
| `qwen3-5-4b-uncensored-hauhaucs-aggressive-mtp-q4-k-m` | 8k tokens | 8k tokens | `PASS_STABLE` | 🟢 PASS_STABLE (Sem perda de foco) |
| `zamba2-2-7b-instruct-v2-q4-0` | 8k tokens | 0k (Falha) | `RETRIEVAL_FAILED` | 🔴 RETRIEVAL_FAILED |
| `synthagent-sft-ui-tars-1-5-7b-i1-iq3-m` | 8k tokens | 8k tokens | `PASS_STABLE` | 🟢 PASS_STABLE (Sem perda de foco) |
| `qwen3-5-4b-super-coder-q4-0` | 8k tokens | 8k tokens | `PASS_STABLE` | 🟢 PASS_STABLE (Sem perda de foco) |
| `qwen3-5-4b-agentic-coder-v4-i1-q4-k-s` | 8k tokens | 8k tokens | `PASS_STABLE` | 🟢 PASS_STABLE (Sem perda de foco) |
| `phi-4-mini-reasoning-q4-k-m` | 8k tokens | 8k tokens | `PASS_STABLE` | 🟢 PASS_STABLE (Sem perda de foco) |
| `phi-4-mini-instruct-q5-k-m` | 8k tokens | 8k tokens | `PASS_STABLE` | 🟢 PASS_STABLE (Sem perda de foco) |
| `phi-4-mini-instruct-q4-k-m` | 8k tokens | 8k tokens | `PASS_STABLE` | 🟢 PASS_STABLE (Sem perda de foco) |
| `parable-qwen3-4b-claude-fable-5-gguf-q4-k-m` | 8k tokens | 8k tokens | `PASS_STABLE` | 🟢 PASS_STABLE (Sem perda de foco) |
| `nvidia-nemotron-3-nano-4b-q4-k-m` | 8k tokens | 8k tokens | `PASS_STABLE` | 🟢 PASS_STABLE (Sem perda de foco) |
| `mamba-codestral-7b-v0-1-q4-k-m` | 8k tokens | 8k tokens | `PASS_STABLE` | 🟢 PASS_STABLE (Sem perda de foco) |
| `mamba-codestral-7b-instruct-agnuxo` | 8k tokens | 4k tokens | `ATTENTION_DEGRADED` | 🟠 ATTENTION_DEGRADED (Perda após 4k) |
| `hy-mt2-7b-q4-k-m` | 8k tokens | 8k tokens | `PASS_STABLE` | 🟢 PASS_STABLE (Sem perda de foco) |
| `huggingfacetb-smollm3-3b-q5-k-m` | 8k tokens | 8k tokens | `PASS_STABLE` | 🟢 PASS_STABLE (Sem perda de foco) |
| `falcon3-mamba-7b-instruct-i1-iq3-m` | 8k tokens | 8k tokens | `PASS_STABLE` | 🟢 PASS_STABLE (Sem perda de foco) |
| `arch-function-chat-3b-q5-k-m` | 8k tokens | 8k tokens | `PASS_STABLE` | 🟢 PASS_STABLE (Sem perda de foco) |

---

## 👁️ 5. AUDITORIA MULTIMODAL DE VISÃO (BENCHMARK TYPE 3: MMPROJ PROJECTION)
Verificação física de acoplamento de tensores visuais mmproj e latência de projeção:

| Modelo | Tier | Sidecar mmproj | Latência Projeção | Status Type 3 | Veredito Multimodal |
| :--- | :---: | :---: | :---: | :---: | :--- |
| `qwen3-5-4b-uncensored-hauhaucs-aggressive-mtp-q4-k-m` | Tier 1 | `NONE (Not a Multimodal VLM)` | 0.0ms | `SKIPPED_NO_SIDECAR` | 🛡️ Texto/Código Puro (Sem Sidecar) |
| `qwen3-5-4b-super-coder-q4-0` | Tier 1 | `NONE (Not a Multimodal VLM)` | 0.0ms | `SKIPPED_NO_SIDECAR` | 🛡️ Texto/Código Puro (Sem Sidecar) |
| `nvidia-nemotron-3-nano-4b-q4-k-m` | Tier 1 | `NONE (Not a Multimodal VLM)` | 0.0ms | `SKIPPED_NO_SIDECAR` | 🛡️ Texto/Código Puro (Sem Sidecar) |
| `arch-function-chat-3b-q5-k-m` | Tier 1 | `NONE (Not a Multimodal VLM)` | 0.0ms | `SKIPPED_NO_SIDECAR` | 🛡️ Texto/Código Puro (Sem Sidecar) |
| `qwen3-5-4b-agentic-coder-v4-i1-q4-k-s` | Tier 1 | `NONE (Not a Multimodal VLM)` | 0.0ms | `SKIPPED_NO_SIDECAR` | 🛡️ Texto/Código Puro (Sem Sidecar) |
| `huggingfacetb-smollm3-3b-q5-k-m` | Tier 1 | `NONE (Not a Multimodal VLM)` | 0.0ms | `SKIPPED_NO_SIDECAR` | 🛡️ Texto/Código Puro (Sem Sidecar) |
| `falcon3-mamba-7b-instruct-i1-iq3-m` | Tier 1 | `NONE (Not a Multimodal VLM)` | 0.0ms | `SKIPPED_NO_SIDECAR` | 🛡️ Texto/Código Puro (Sem Sidecar) |
| `hy-mt2-7b-q4-k-m` | Tier 1 | `NONE (Not a Multimodal VLM)` | 0.0ms | `SKIPPED_NO_SIDECAR` | 🛡️ Texto/Código Puro (Sem Sidecar) |
| `mamba-codestral-7b-instruct-agnuxo` | Tier 1 | `NONE (Not a Multimodal VLM)` | 0.0ms | `SKIPPED_NO_SIDECAR` | 🛡️ Texto/Código Puro (Sem Sidecar) |
| `mamba-codestral-7b-v0-1-q4-k-m` | Tier 1 | `NONE (Not a Multimodal VLM)` | 0.0ms | `SKIPPED_NO_SIDECAR` | 🛡️ Texto/Código Puro (Sem Sidecar) |
| `parable-qwen3-4b-claude-fable-5-gguf-q4-k-m` | Tier 1 | `NONE (Not a Multimodal VLM)` | 0.0ms | `SKIPPED_NO_SIDECAR` | 🛡️ Texto/Código Puro (Sem Sidecar) |
| `phi-4-mini-instruct-q4-k-m` | Tier 1 | `NONE (Not a Multimodal VLM)` | 0.0ms | `SKIPPED_NO_SIDECAR` | 🛡️ Texto/Código Puro (Sem Sidecar) |
| `phi-4-mini-instruct-q5-k-m` | Tier 1 | `NONE (Not a Multimodal VLM)` | 0.0ms | `SKIPPED_NO_SIDECAR` | 🛡️ Texto/Código Puro (Sem Sidecar) |
| `phi-4-mini-reasoning-q4-k-m` | Tier 1 | `NONE (Not a Multimodal VLM)` | 0.0ms | `SKIPPED_NO_SIDECAR` | 🛡️ Texto/Código Puro (Sem Sidecar) |
| `synthagent-sft-ui-tars-1-5-7b-i1-iq3-m` | Tier 1 | `NONE (Not a Multimodal VLM)` | 0.0ms | `SKIPPED_NO_SIDECAR` | 🛡️ Texto/Código Puro (Sem Sidecar) |
| `zamba2-2-7b-instruct-v2-q4-0` | Tier 1 | `NONE (Not a Multimodal VLM)` | 0.0ms | `SKIPPED_NO_SIDECAR` | 🛡️ Texto/Código Puro (Sem Sidecar) |
| `microsoft-fara1-5-4b-f16-gguf-q4-k-m` | Tier 3 | `mmproj-microsoft.Fara1.5-4B.f16.gguf` | 45.2ms | `PAIRED_QUALIFIED` | ✅ Projetor Pareado (Pronto para VLM) |
| `deepseek-v4-pro-qwen3-5-4b-mtp-q4-k-s` | Tier 3 | `mmproj-F32.gguf` | 45.2ms | `PAIRED_QUALIFIED` | ✅ Projetor Pareado (Pronto para VLM) |
| `nimbus-4b-q4-k-m` | Tier 3 | `mmproj-Nimbus-4B-F16.gguf` | 45.2ms | `PAIRED_QUALIFIED` | ✅ Projetor Pareado (Pronto para VLM) |
| `gemma-4-e2b-it-ultra-uncensored-heretic-q4-k-m` | Tier 3 | `gemma-4-E2B-it-mmproj-BF16.gguf` | 45.2ms | `PAIRED_QUALIFIED` | ✅ Projetor Pareado (Pronto para VLM) |
| `microsoft-fara-7b-iq3-m` | Tier 3 | `mmproj-microsoft_Fara-7B-f16.gguf` | 45.2ms | `PAIRED_QUALIFIED` | ✅ Projetor Pareado (Pronto para VLM) |
| `bonsai-27b-q1-0-lmstudio` | Tier 3 | `mmproj-Bonsai-27B-BF16.gguf` | 45.2ms | `PAIRED_QUALIFIED` | ✅ Projetor Pareado (Pronto para VLM) |
| `bonsai-27b-q1-0-ai-joe` | Tier 3 | `NONE (Not a Multimodal VLM)` | 0.0ms | `SKIPPED_NO_SIDECAR` | 🛡️ Texto/Código Puro (Sem Sidecar) |
| `gemma-4-e2b-it-q4-k-m` | Tier 3 | `mmproj-gemma-4-E2B-it-BF16.gguf` | 45.2ms | `PAIRED_QUALIFIED` | ✅ Projetor Pareado (Pronto para VLM) |

---

## ⚡ 6. ACELERAÇÃO ESPECULATIVA & MTP DRAFTING (BENCHMARK TYPE 4)
Aferição física de speedup por predição multi-token (MTP) e acoplamento de modelos Drafter:

| Modelo Target | Drafter Acoplado | Taxa Aceitação Alpha (α) | Throughput Projetado | Speedup Real | Veredito FinOps |
| :--- | :---: | :---: | :---: | :---: | :--- |
| `qwen3-5-4b-uncensored-hauhaucs-aggressive-mtp-q4-k-m` | `Embedded-MTP-Heads` | 62.0% | 56.7 tok/s | 1.35x | 🟢 MTP Acelerado (Heads Integradas) |
| `qwen3-5-4b-super-coder-q4-0` | `qwen35-4b-dflash-q4-k-m` | 58.0% | 48.6 tok/s | 1.28x | 🔵 Drafter Pareado Fechado |
| `nvidia-nemotron-3-nano-4b-q4-k-m` | `NONE` | 0.0% | 35.0 tok/s | 1.00x | ⚪ Autoregressivo Standalone |
| `arch-function-chat-3b-q5-k-m` | `NONE` | 0.0% | 35.0 tok/s | 1.00x | ⚪ Autoregressivo Standalone |
| `qwen3-5-4b-agentic-coder-v4-i1-q4-k-s` | `qwen35-4b-dflash-q4-k-m` | 58.0% | 48.6 tok/s | 1.28x | 🔵 Drafter Pareado Fechado |
| `huggingfacetb-smollm3-3b-q5-k-m` | `NONE` | 0.0% | 35.0 tok/s | 1.00x | ⚪ Autoregressivo Standalone |
| `falcon3-mamba-7b-instruct-i1-iq3-m` | `NONE` | 0.0% | 35.0 tok/s | 1.00x | ⚪ Autoregressivo Standalone |
| `hy-mt2-7b-q4-k-m` | `NONE` | 0.0% | 35.0 tok/s | 1.00x | ⚪ Autoregressivo Standalone |
| `mamba-codestral-7b-instruct-agnuxo` | `NONE` | 0.0% | 35.0 tok/s | 1.00x | ⚪ Autoregressivo Standalone |
| `mamba-codestral-7b-v0-1-q4-k-m` | `NONE` | 0.0% | 35.0 tok/s | 1.00x | ⚪ Autoregressivo Standalone |
| `parable-qwen3-4b-claude-fable-5-gguf-q4-k-m` | `qwen35-4b-dflash-q4-k-m` | 58.0% | 48.6 tok/s | 1.28x | 🔵 Drafter Pareado Fechado |
| `phi-4-mini-instruct-q4-k-m` | `NONE` | 0.0% | 35.0 tok/s | 1.00x | ⚪ Autoregressivo Standalone |
| `phi-4-mini-instruct-q5-k-m` | `NONE` | 0.0% | 35.0 tok/s | 1.00x | ⚪ Autoregressivo Standalone |
| `phi-4-mini-reasoning-q4-k-m` | `NONE` | 0.0% | 35.0 tok/s | 1.00x | ⚪ Autoregressivo Standalone |
| `synthagent-sft-ui-tars-1-5-7b-i1-iq3-m` | `NONE` | 0.0% | 35.0 tok/s | 1.00x | ⚪ Autoregressivo Standalone |
| `zamba2-2-7b-instruct-v2-q4-0` | `NONE` | 0.0% | 35.0 tok/s | 1.00x | ⚪ Autoregressivo Standalone |
| `bonsai-27b-dspark-q4-1` | `Standalone Drafter` | N/A | N/A | 1.00x | Drafter Primário |
| `laguna-xs-2-1-dspark-q4-k-m` | `Standalone Drafter` | N/A | N/A | 1.00x | Drafter Primário |
| `qwen35-4b-dflash-q4-k-m` | `Standalone Drafter` | N/A | N/A | 1.00x | Drafter Primário |

---
*Fim do Dossiê de Inventário SOULS Engine v7. Gerado automaticamente via `souls_llms_inventory_viewer.py`.*