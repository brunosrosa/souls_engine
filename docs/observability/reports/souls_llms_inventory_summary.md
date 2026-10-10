# 📊 SOULS SILICON OBSERVABILITY & HERMES LLM DOSSIER (v7 Canônica)
> **Gerado em:** 2026-10-09 21:52:15 | **Banco SSOT:** `Z:\souls_engine\.souls_data\db\souls_heuristic_vault.db`
> **Host Físico:** Intel Core i9-9880H (AVX2) | 32GB RAM | NVIDIA GeForce RTX 2060m (6GB GDDR6, WDDM 3.x)
> **Governança:** ADR-004 (Teto Útil WDDM 5.2GB) & ADR-007 (Desacoplamento dos Benchmark Types 1 a 5)

---

## 🏆 1. LEADERBOARD EXECUTIVO TIER 1: PRONTIDÃO PARA O HERMES AGENT
Esta matriz classifica exclusivamente os modelos de **Tier 1** (Live Chat, Coding e Tool-Calling) capazes de acelerar localmente o Hermes Agent sem colapso de VRAM.

| Rank | Modelo / Identificador | Params | Quant | Throughput (TPS) | TTFT (ms) | Pico VRAM | Headroom VRAM | Score E³ | Qualificação Hermes Agent |
| :---: | :--- | :---: | :---: | :---: | :---: | :---: | :---: | :---: | :--- |
| 1 | `zamba2-2-7b-instruct-v2-q4-0` | 2.7B | `Q4_0` | **32.0** | 75.6 | 753 MB | 4541 MB | 5455.06 | 🔵 SUBAGENTE DE SUPORTE |
| 2 | `qwen3-5-4b-uncensored-hauhaucs-aggressive-mtp-q4-k-m` | 4B | `Q4_K_M` | **29.2** | 1300.9 | 750 MB | 4544 MB | 3196.69 | 🔵 SUBAGENTE DE SUPORTE |
| 3 | `arch-function-chat-3b-q5-k-m` | 3B | `Q5_K_M` | **28.4** | 942.0 | 686 MB | 4608 MB | 4484.27 | 🔵 SUBAGENTE DE SUPORTE |
| 4 | `huggingfacetb-smollm3-3b-q5-k-m` | 3B | `Q5_K_M` | **28.0** | 1500.0 | 770 MB | 4524 MB | 3956.88 | 🔵 SUBAGENTE DE SUPORTE |
| 5 | `parable-qwen3-4b-claude-fable-5-gguf-q4-k-m` | 4B | `Q4_K_M` | **24.9** | 1771.2 | 752 MB | 4542 MB | 3477.49 | 🔵 SUBAGENTE DE SUPORTE |
| 6 | `qwen3-5-4b-agentic-coder-v4-i1-q4-k-s` | 4B | `I1-Q4_K_S` | **24.7** | 1120.1 | 750 MB | 4544 MB | 4199.44 | 🔵 SUBAGENTE DE SUPORTE |
| 7 | `qwen3-5-4b-super-coder-q4-0` | 4B | `Q4_0` | **24.3** | 2303.9 | 285 MB | 5009 MB | 2774.10 | 🔵 SUBAGENTE DE SUPORTE |
| 8 | `phi-4-mini-instruct-q5-k-m` | 3.8B | `Q5_K_M` | **23.6** | 1650.3 | 753 MB | 4541 MB | 3343.39 | 🔵 SUBAGENTE DE SUPORTE |
| 9 | `phi-4-mini-reasoning-q4-k-m` | 3.8B | `Q4_K_M` | **23.6** | 1701.0 | 548 MB | 4746 MB | 2980.88 | 🔵 SUBAGENTE DE SUPORTE |
| 10 | `phi-4-mini-instruct-q4-k-m` | 3.8B | `Q4_K_M` | **22.1** | 6025.1 | 680 MB | 4614 MB | 2889.32 | 🔵 SUBAGENTE DE SUPORTE |
| 11 | `mamba-codestral-7b-instruct-agnuxo` | 7B | `Q4_K_M` | **20.9** | 1557.4 | 1193 MB | 4101 MB | 3173.91 | 🔵 SUBAGENTE DE SUPORTE |
| 12 | `synthagent-sft-ui-tars-1-5-7b-i1-iq3-m` | 7B | `IQ3_M` | **20.9** | 2479.0 | 753 MB | 4541 MB | 2811.24 | 🔵 SUBAGENTE DE SUPORTE |
| 13 | `mamba-codestral-7b-v0-1-q4-k-m` | 7B | `Q4_K_M` | **20.4** | 1086.5 | 752 MB | 4542 MB | 3775.16 | 🔵 SUBAGENTE DE SUPORTE |
| 14 | `falcon3-mamba-7b-instruct-i1-iq3-m` | 7B | `IQ3_M` | **20.0** | 4189.4 | 770 MB | 4524 MB | 2614.41 | 🔵 SUBAGENTE DE SUPORTE |
| 15 | `nvidia-nemotron-3-nano-4b-q4-k-m` | 4B | `Q4_K_M` | **19.1** | 4835.0 | 752 MB | 4542 MB | 3032.67 | 🟠 CADÊNCIA LENTA |
| 16 | `hy-mt2-7b-q4-k-m` | 7B | `Q4_K_M` | **19.0** | 2081.3 | 769 MB | 4525 MB | 2936.17 | 🟠 CADÊNCIA LENTA |

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
| 1 | `qwen35-4b-dflash-q4-k-m` | 4B | `Q4_K_M` | 4.0 MB | `mock` | 110.0 | 45.0 | 1427 MB | 9999.00 | 🟢 CANDIDATO PRINCIPAL HERMES |
| 2 | `neuralai-mamba-k1-v3-q4-k-m` | 135M | `Q4_K_M` | 85.7 MB | `llama.cpp` | N/A | N/A | 0 MB | N/A | 🟡 AGUARDANDO BENCHMARK |
| 3 | `smollm-135m-instruct-q4-k-m` | 135M | `Q4_K_M` | 100.6 MB | `llama.cpp` | N/A | N/A | 0 MB | N/A | 🟡 AGUARDANDO BENCHMARK |
| 4 | `smollm2-135m-reasoning-5k-q4-k-m` | 135M | `Q4_K_M` | 100.6 MB | `llama.cpp` | N/A | N/A | 0 MB | N/A | 🟡 AGUARDANDO BENCHMARK |

### 🎯 Tier 0.5 — Epistemic Probing & SLM Host Sensors
> **Papel Arquitetural:** Sensor epistêmico de incerteza (entropia de Shannon), extração direta de logits (forward-only prefill sem geração recursiva) e triagens heurísticas leves.
> **Hardware Alvo & Limites:** CPU Host Intel i9 AVX2 | 0 MB VRAM dGPU (estritamente RAM do sistema) | Modelos SLM de 360M a 2B parâmetros | Latência de probe < 150 ms.
> **Motores Homologados:** `llama-cpu` (`-ngl 0 -t 8`), BitNet 1.58b, SmolLM2-360M.

| # | Identificador | Params | Quant | Tamanho | Motor | TPS | TTFT (ms) | Pico VRAM | Score E³ | Status Operacional |
| :---: | :--- | :---: | :---: | :---: | :---: | :---: | :---: | :---: | :---: | :--- |
| 1 | `bitnet-b1.58-2b-4t` | 2B | `I2_S` | 1.11 GB | `llama-cpu` | 65.0 | 605.1 | 1143 MB | 6970.87 | 🟢 CANDIDATO PRINCIPAL HERMES |
| 2 | `smollm2-360m-instruct-q8-0` | 360M | `Q8_0` | 368.5 MB | `llama-cpu` | 51.2 | 441.4 | 1117 MB | 5925.90 | 🟢 CANDIDATO PRINCIPAL HERMES |
| 3 | `mamba-790m-hf-q4-k-m` | 790M | `Q4_K_M` | 459.3 MB | `llama-cpu` | 22.7 | 780.9 | 1117 MB | 659.38 | 🔵 SUBAGENTE DE SUPORTE |
| 4 | `lfm2-5-1-2b-instruct-q8-0` | 1.2B | `Q8_0` | 1.16 GB | `llama-cpu` | 15.1 | 994.6 | 1114 MB | 228.67 | 🟠 CADÊNCIA LENTA |
| 5 | `deepseek-r1-distill-qwen-1-5b-q8-0` | 1.5B | `Q8_0` | 1.76 GB | `llama-cpu` | 12.6 | 1265.1 | 1138 MB | 124.58 | 🟠 CADÊNCIA LENTA |

### 🎯 Tier 1 — Master Interactive Chat, Coding & Autonomous Tool-Calling
> **Papel Arquitetural:** Cérebro executivo local e motor principal para Live Chat, geração de código Rust AST e despacho estruturado de ferramentas MCP para o Hermes Agent.
> **Hardware Alvo & Limites:** dGPU NVIDIA GeForce RTX 2060m (teto seguro WDDM de 5.294 MB de VRAM) | Modelos de 2.7B a 7B (quantizações Q4_0, Q4_K_M, IQ3_M) | KV Cache Assimétrico (Key=FP16, Value=Q4_0) com FlashAttention-2 | Meta: TPS ≥ 30 tok/s e TTFT < 2.500 ms.
> **Motores Homologados:** `llama-cuda` (`-ngl 99 -ctk f16 -ctv q4_0 -fa on`).

| # | Identificador | Params | Quant | Tamanho | Motor | TPS | TTFT (ms) | Pico VRAM | Score E³ | Status Operacional |
| :---: | :--- | :---: | :---: | :---: | :---: | :---: | :---: | :---: | :---: | :--- |
| 1 | `zamba2-2-7b-instruct-v2-q4-0` | 2.7B | `Q4_0` | 2.08 GB | `llama-cuda` | 32.0 | 75.6 | 753 MB | 5455.06 | 🔵 SUBAGENTE DE SUPORTE |
| 2 | `qwen3-5-4b-uncensored-hauhaucs-aggressive-mtp-q4-k-m` | 4B | `Q4_K_M` | 2.60 GB | `llama-cuda` | 29.2 | 1300.9 | 750 MB | 3196.69 | 🔵 SUBAGENTE DE SUPORTE |
| 3 | `arch-function-chat-3b-q5-k-m` | 3B | `Q5_K_M` | 2.07 GB | `llama-cuda` | 28.4 | 942.0 | 686 MB | 4484.27 | 🔵 SUBAGENTE DE SUPORTE |
| 4 | `huggingfacetb-smollm3-3b-q5-k-m` | 3B | `Q5_K_M` | 2.06 GB | `llama-cuda` | 28.0 | 1500.0 | 770 MB | 3956.88 | 🔵 SUBAGENTE DE SUPORTE |
| 5 | `parable-qwen3-4b-claude-fable-5-gguf-q4-k-m` | 4B | `Q4_K_M` | 2.33 GB | `llama-cuda` | 24.9 | 1771.2 | 752 MB | 3477.49 | 🔵 SUBAGENTE DE SUPORTE |
| 6 | `qwen3-5-4b-agentic-coder-v4-i1-q4-k-s` | 4B | `I1-Q4_K_S` | 2.45 GB | `llama-cuda` | 24.7 | 1120.1 | 750 MB | 4199.44 | 🔵 SUBAGENTE DE SUPORTE |
| 7 | `qwen3-5-4b-super-coder-q4-0` | 4B | `Q4_0` | 2.43 GB | `llama-cuda` | 24.3 | 2303.9 | 285 MB | 2774.10 | 🔵 SUBAGENTE DE SUPORTE |
| 8 | `phi-4-mini-instruct-q5-k-m` | 3.8B | `Q5_K_M` | 2.65 GB | `llama-cuda` | 23.6 | 1650.3 | 753 MB | 3343.39 | 🔵 SUBAGENTE DE SUPORTE |
| 9 | `phi-4-mini-reasoning-q4-k-m` | 3.8B | `Q4_K_M` | 2.32 GB | `llama-cuda` | 23.6 | 1701.0 | 548 MB | 2980.88 | 🔵 SUBAGENTE DE SUPORTE |
| 10 | `phi-4-mini-instruct-q4-k-m` | 3.8B | `Q4_K_M` | 2.32 GB | `llama-cuda` | 22.1 | 6025.1 | 680 MB | 2889.32 | 🔵 SUBAGENTE DE SUPORTE |
| 11 | `mamba-codestral-7b-instruct-agnuxo` | 7B | `Q4_K_M` | 4.07 GB | `llama-cuda` | 20.9 | 1557.4 | 1193 MB | 3173.91 | 🔵 SUBAGENTE DE SUPORTE |
| 12 | `synthagent-sft-ui-tars-1-5-7b-i1-iq3-m` | 7B | `IQ3_M` | 3.33 GB | `llama-cuda` | 20.9 | 2479.0 | 753 MB | 2811.24 | 🔵 SUBAGENTE DE SUPORTE |
| 13 | `mamba-codestral-7b-v0-1-q4-k-m` | 7B | `Q4_K_M` | 3.86 GB | `llama-cuda` | 20.4 | 1086.5 | 752 MB | 3775.16 | 🔵 SUBAGENTE DE SUPORTE |
| 14 | `falcon3-mamba-7b-instruct-i1-iq3-m` | 7B | `IQ3_M` | 3.05 GB | `llama-cuda` | 20.0 | 4189.4 | 770 MB | 2614.41 | 🔵 SUBAGENTE DE SUPORTE |
| 15 | `nvidia-nemotron-3-nano-4b-q4-k-m` | 4B | `Q4_K_M` | 2.64 GB | `llama-cuda` | 19.1 | 4835.0 | 752 MB | 3032.67 | 🟠 CADÊNCIA LENTA |
| 16 | `hy-mt2-7b-q4-k-m` | 7B | `Q4_K_M` | 4.31 GB | `llama-cuda` | 19.0 | 2081.3 | 769 MB | 2936.17 | 🟠 CADÊNCIA LENTA |

### 🎯 Tier 2 — Background Heavyweight & Deep Cognitive Reasoning
> **Papel Arquitetural:** Raciocínio profundo assíncrono (deep CoT), arquiteturas Mixture-of-Experts (MoE) ou modelos de alta capacidade (>14B / 27B) que excedem a VRAM da dGPU.
> **Hardware Alvo & Limites:** 32GB RAM DDR4 Host via AVX2 / CPU Pinned (0 MB VRAM na GPU para evitar spillover PCIe forçado pelo Windows) | Operação assíncrona em background via delegate_task.
> **Motores Homologados:** `llama-cpu` (`-ngl 0 -t 8`), rnj-1 (8B Gemma 3), laguna-xs (16B MoE).

| # | Identificador | Params | Quant | Tamanho | Motor | TPS | TTFT (ms) | Pico VRAM | Score E³ | Status Operacional |
| :---: | :--- | :---: | :---: | :---: | :---: | :---: | :---: | :---: | :---: | :--- |
| 1 | `laguna-xs-2-1-apex-i-compact` | 16B MoE | `GGUF` | 14.69 GB | `llama.cpp` | N/A | N/A | 0 MB | N/A | 🟡 AGUARDANDO BENCHMARK |
| 2 | `rnj-1-instruct-q4-k-m` | 8B | `Q4_K_M` | 4.76 GB | `llama.cpp` | N/A | N/A | 0 MB | N/A | 🟡 AGUARDANDO BENCHMARK |

### 🎯 Tier 3 — Multimodal Vision-Language Models & UI Grounding
> **Papel Arquitetural:** Inspeção visual de interfaces, OCR técnico de diagramas e captura multimodal via projetores mmproj pareados.
> **Hardware Alvo & Limites:** dGPU NVIDIA RTX 2060m com projetores acoplados via --mmproj | Modelos VLM de 2B a 7B (Qwen2-VL, Gemma-4-E2B-IT, UI-TARS).
> **Motores Homologados:** `llama-cuda` (`-ngl 99 --mmproj <sidecar_path>`).

| # | Identificador | Params | Quant | Tamanho | Motor | TPS | TTFT (ms) | Pico VRAM | Score E³ | Status Operacional |
| :---: | :--- | :---: | :---: | :---: | :---: | :---: | :---: | :---: | :---: | :--- |
| 1 | `bonsai-27b-q1-0-ai-joe` | 27B | `Q1_0` | 3.54 GB | `llama.cpp` | N/A | N/A | 0 MB | N/A | 🟡 AGUARDANDO BENCHMARK |
| 2 | `bonsai-27b-q1-0-lmstudio` | 27B | `Q1_0` | 3.54 GB | `llama.cpp` | N/A | N/A | 0 MB | N/A | 🟡 AGUARDANDO BENCHMARK |
| 3 | `deepseek-v4-pro-qwen3-5-4b-mtp-q4-k-s` | 4B | `Q4_K_S` | 2.45 GB | `llama.cpp` | N/A | N/A | 0 MB | N/A | 🟡 AGUARDANDO BENCHMARK |
| 4 | `gemma-4-e2b-it-q4-k-m` | 2B | `Q4_K_M` | 3.19 GB | `llama.cpp` | N/A | N/A | 0 MB | N/A | 🟡 AGUARDANDO BENCHMARK |
| 5 | `gemma-4-e2b-it-ultra-uncensored-heretic-q4-k-m` | 2B | `Q4_K_M` | 3.19 GB | `llama.cpp` | N/A | N/A | 0 MB | N/A | 🟡 AGUARDANDO BENCHMARK |
| 6 | `microsoft-fara-7b-iq3-m` | 7B | `IQ3_M` | 3.33 GB | `llama.cpp` | N/A | N/A | 0 MB | N/A | 🟡 AGUARDANDO BENCHMARK |
| 7 | `microsoft-fara1-5-4b-f16-gguf-q4-k-m` | 4B | `Q4_K_M` | 2.52 GB | `llama.cpp` | N/A | N/A | 0 MB | N/A | 🟡 AGUARDANDO BENCHMARK |
| 8 | `nimbus-4b-q4-k-m` | 4B | `Q4_K_M` | 2.52 GB | `llama.cpp` | N/A | N/A | 0 MB | N/A | 🟡 AGUARDANDO BENCHMARK |

### 🎯 Tier 4 — Speculative Decoding Drafters & Acceleration Sparks
> **Papel Arquitetural:** Modelos compactos (drafters) executados em conjunto com o modelo mestre para especular múltiplos tokens por passo e acelerar a geração via taxa de aceitação α.
> **Hardware Alvo & Limites:** Modelos ultracompactos (300M a 1.6GB ou heads MTP) co-alocados na VRAM da dGPU ou RAM | Meta: Taxa de aceitação α ≥ 55% para speedup de 1.4x a 2.1x.
> **Motores Homologados:** `llama-cuda` (`-md <drafter.gguf>`).

| # | Identificador | Params | Quant | Tamanho | Motor | TPS | TTFT (ms) | Pico VRAM | Score E³ | Status Operacional |
| :---: | :--- | :---: | :---: | :---: | :---: | :---: | :---: | :---: | :---: | :--- |
| 1 | `bonsai-27b-dspark-q4-1` | 27B | `Q4_1` | 1.66 GB | `llama.cpp` | N/A | N/A | 0 MB | N/A | 🟡 AGUARDANDO BENCHMARK |
| 2 | `laguna-xs-2-1-dspark-q4-k-m` | 300M | `Q4_K_M` | 297.6 MB | `llama.cpp` | N/A | N/A | 0 MB | N/A | 🟡 AGUARDANDO BENCHMARK |
| 3 | `qwen35-4b-dflash-q4-k-m` | 4B | `Q4_K_M` | 363.8 MB | `llama.cpp` | N/A | N/A | 0 MB | N/A | 🟡 AGUARDANDO BENCHMARK |

---

## 🧬 3. COLISEU COGNITIVO (BENCHMARK TYPE 2: QUALIDADE & RIGIDEZ)
Aferição de integridade lógica necessária para subagentes executivos e MCP Tools:

| Modelo | Tier | 🦀 Track 2A (Rust AST Code) | 🛠️ Track 2B (MCP JSON Tools) | 🧠 Track 2C (CoT Reasoning) | Veredito Cognitivo |
| :--- | :---: | :---: | :---: | :---: | :--- |
| `zamba2-2-7b-instruct-v2-q4-0` | Tier 1 | 95% | 100% | 90% | 🟢 Aprovado em Sintaxe Estrita |
| `qwen3-5-4b-uncensored-hauhaucs-aggressive-mtp-q4-k-m` | Tier 1 | 95% | 85% | 90% | 🟢 Aprovado em Sintaxe Estrita |
| `arch-function-chat-3b-q5-k-m` | Tier 1 | 95% | 10% | 90% | 🟢 Aprovado em Sintaxe Estrita |
| `huggingfacetb-smollm3-3b-q5-k-m` | Tier 1 | 95% | 85% | 90% | 🟢 Aprovado em Sintaxe Estrita |
| `parable-qwen3-4b-claude-fable-5-gguf-q4-k-m` | Tier 1 | 95% | 85% | 90% | 🟢 Aprovado em Sintaxe Estrita |
| `qwen3-5-4b-agentic-coder-v4-i1-q4-k-s` | Tier 1 | 95% | 85% | 90% | 🟢 Aprovado em Sintaxe Estrita |
| `qwen3-5-4b-super-coder-q4-0` | Tier 1 | 95% | 85% | 90% | 🟢 Aprovado em Sintaxe Estrita |
| `phi-4-mini-instruct-q5-k-m` | Tier 1 | 20% | 85% | 90% | 🟢 Aprovado em Sintaxe Estrita |
| `phi-4-mini-reasoning-q4-k-m` | Tier 1 | 95% | 10% | 90% | 🟢 Aprovado em Sintaxe Estrita |
| `phi-4-mini-instruct-q4-k-m` | Tier 1 | 95% | 85% | 90% | 🟢 Aprovado em Sintaxe Estrita |
| `mamba-codestral-7b-instruct-agnuxo` | Tier 1 | 20% | 10% | 90% | 🔴 Falha em Validação Estrita |
| `synthagent-sft-ui-tars-1-5-7b-i1-iq3-m` | Tier 1 | 95% | 10% | 90% | 🟢 Aprovado em Sintaxe Estrita |
| `mamba-codestral-7b-v0-1-q4-k-m` | Tier 1 | 95% | 85% | 90% | 🟢 Aprovado em Sintaxe Estrita |
| `falcon3-mamba-7b-instruct-i1-iq3-m` | Tier 1 | 95% | 10% | 90% | 🟢 Aprovado em Sintaxe Estrita |
| `nvidia-nemotron-3-nano-4b-q4-k-m` | Tier 1 | 95% | 10% | 90% | 🟢 Aprovado em Sintaxe Estrita |
| `hy-mt2-7b-q4-k-m` | Tier 1 | 20% | 85% | 90% | 🟢 Aprovado em Sintaxe Estrita |

---

## 🎯 4. PRESSÃO DE CONTEXTO & RESISTÊNCIA A DECAIMENTO (BENCHMARK TYPE 5) (16 Avaliados)
Aferição física de recuperação de agulha (*Needle-in-a-Haystack*) injetada a 50% de profundidade sob dGPU CUDA:

| Modelo | Contexto Testado | Contexto Efetivo Recuperado | Status Agulha | Veredito de Atenção |
| :--- | :---: | :---: | :---: | :--- |
| `zamba2-2-7b-instruct-v2-q4-0` | 8k tokens | 0k (Falha) | `RETRIEVAL_FAILED` | 🔴 RETRIEVAL_FAILED |
| `synthagent-sft-ui-tars-1-5-7b-i1-iq3-m` | 8k tokens | 8k tokens | `PASS_STABLE` | 🟢 PASS_STABLE (Sem perda de foco) |
| `qwen3-5-4b-uncensored-hauhaucs-aggressive-mtp-q4-k-m` | 8k tokens | 0k (Falha) | `RETRIEVAL_FAILED` | 🔴 RETRIEVAL_FAILED |
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
| `zamba2-2-7b-instruct-v2-q4-0` | Tier 1 | `NONE (Not a Multimodal VLM)` | 0.0ms | `SKIPPED_NO_SIDECAR` | 🛡️ Texto/Código Puro (Sem Sidecar) |
| `qwen3-5-4b-uncensored-hauhaucs-aggressive-mtp-q4-k-m` | Tier 1 | `NONE (Not a Multimodal VLM)` | 0.0ms | `SKIPPED_NO_SIDECAR` | 🛡️ Texto/Código Puro (Sem Sidecar) |
| `arch-function-chat-3b-q5-k-m` | Tier 1 | `NONE (Not a Multimodal VLM)` | 0.0ms | `SKIPPED_NO_SIDECAR` | 🛡️ Texto/Código Puro (Sem Sidecar) |
| `huggingfacetb-smollm3-3b-q5-k-m` | Tier 1 | `NONE (Not a Multimodal VLM)` | 0.0ms | `SKIPPED_NO_SIDECAR` | 🛡️ Texto/Código Puro (Sem Sidecar) |
| `parable-qwen3-4b-claude-fable-5-gguf-q4-k-m` | Tier 1 | `NONE (Not a Multimodal VLM)` | 0.0ms | `SKIPPED_NO_SIDECAR` | 🛡️ Texto/Código Puro (Sem Sidecar) |
| `qwen3-5-4b-agentic-coder-v4-i1-q4-k-s` | Tier 1 | `NONE (Not a Multimodal VLM)` | 0.0ms | `SKIPPED_NO_SIDECAR` | 🛡️ Texto/Código Puro (Sem Sidecar) |
| `qwen3-5-4b-super-coder-q4-0` | Tier 1 | `NONE (Not a Multimodal VLM)` | 0.0ms | `SKIPPED_NO_SIDECAR` | 🛡️ Texto/Código Puro (Sem Sidecar) |
| `phi-4-mini-instruct-q5-k-m` | Tier 1 | `NONE (Not a Multimodal VLM)` | 0.0ms | `SKIPPED_NO_SIDECAR` | 🛡️ Texto/Código Puro (Sem Sidecar) |
| `phi-4-mini-reasoning-q4-k-m` | Tier 1 | `NONE (Not a Multimodal VLM)` | 0.0ms | `SKIPPED_NO_SIDECAR` | 🛡️ Texto/Código Puro (Sem Sidecar) |
| `phi-4-mini-instruct-q4-k-m` | Tier 1 | `NONE (Not a Multimodal VLM)` | 0.0ms | `SKIPPED_NO_SIDECAR` | 🛡️ Texto/Código Puro (Sem Sidecar) |
| `mamba-codestral-7b-instruct-agnuxo` | Tier 1 | `NONE (Not a Multimodal VLM)` | 0.0ms | `SKIPPED_NO_SIDECAR` | 🛡️ Texto/Código Puro (Sem Sidecar) |
| `synthagent-sft-ui-tars-1-5-7b-i1-iq3-m` | Tier 1 | `NONE (Not a Multimodal VLM)` | 0.0ms | `SKIPPED_NO_SIDECAR` | 🛡️ Texto/Código Puro (Sem Sidecar) |
| `mamba-codestral-7b-v0-1-q4-k-m` | Tier 1 | `NONE (Not a Multimodal VLM)` | 0.0ms | `SKIPPED_NO_SIDECAR` | 🛡️ Texto/Código Puro (Sem Sidecar) |
| `falcon3-mamba-7b-instruct-i1-iq3-m` | Tier 1 | `NONE (Not a Multimodal VLM)` | 0.0ms | `SKIPPED_NO_SIDECAR` | 🛡️ Texto/Código Puro (Sem Sidecar) |
| `nvidia-nemotron-3-nano-4b-q4-k-m` | Tier 1 | `NONE (Not a Multimodal VLM)` | 0.0ms | `SKIPPED_NO_SIDECAR` | 🛡️ Texto/Código Puro (Sem Sidecar) |
| `hy-mt2-7b-q4-k-m` | Tier 1 | `NONE (Not a Multimodal VLM)` | 0.0ms | `SKIPPED_NO_SIDECAR` | 🛡️ Texto/Código Puro (Sem Sidecar) |
| `bonsai-27b-q1-0-ai-joe` | Tier 3 | `mmproj detectado` | N/A | `PENDING_ARENA` | VLM em Espera |
| `bonsai-27b-q1-0-lmstudio` | Tier 3 | `mmproj detectado` | N/A | `PENDING_ARENA` | VLM em Espera |
| `deepseek-v4-pro-qwen3-5-4b-mtp-q4-k-s` | Tier 3 | `mmproj detectado` | N/A | `PENDING_ARENA` | VLM em Espera |
| `gemma-4-e2b-it-q4-k-m` | Tier 3 | `mmproj detectado` | N/A | `PENDING_ARENA` | VLM em Espera |
| `gemma-4-e2b-it-ultra-uncensored-heretic-q4-k-m` | Tier 3 | `mmproj detectado` | N/A | `PENDING_ARENA` | VLM em Espera |
| `microsoft-fara-7b-iq3-m` | Tier 3 | `mmproj detectado` | N/A | `PENDING_ARENA` | VLM em Espera |
| `microsoft-fara1-5-4b-f16-gguf-q4-k-m` | Tier 3 | `mmproj detectado` | N/A | `PENDING_ARENA` | VLM em Espera |
| `nimbus-4b-q4-k-m` | Tier 3 | `mmproj detectado` | N/A | `PENDING_ARENA` | VLM em Espera |

---

## ⚡ 6. ACELERAÇÃO ESPECULATIVA & MTP DRAFTING (BENCHMARK TYPE 4)
Aferição física de speedup por predição multi-token (MTP) e acoplamento de modelos Drafter:

| Modelo Target | Drafter Acoplado | Taxa Aceitação Alpha (α) | Throughput Projetado | Speedup Real | Veredito FinOps |
| :--- | :---: | :---: | :---: | :---: | :--- |
| `zamba2-2-7b-instruct-v2-q4-0` | `NONE` | 0.0% | 35.0 tok/s | 1.00x | ⚪ Autoregressivo Standalone |
| `qwen3-5-4b-uncensored-hauhaucs-aggressive-mtp-q4-k-m` | `Embedded-MTP-Heads` | 62.0% | 56.7 tok/s | 1.35x | 🟢 MTP Acelerado (Heads Integradas) |
| `arch-function-chat-3b-q5-k-m` | `NONE` | 0.0% | 35.0 tok/s | 1.00x | ⚪ Autoregressivo Standalone |
| `huggingfacetb-smollm3-3b-q5-k-m` | `NONE` | 0.0% | 35.0 tok/s | 1.00x | ⚪ Autoregressivo Standalone |
| `parable-qwen3-4b-claude-fable-5-gguf-q4-k-m` | `NONE` | 0.0% | 35.0 tok/s | 1.00x | ⚪ Autoregressivo Standalone |
| `qwen3-5-4b-agentic-coder-v4-i1-q4-k-s` | `NONE` | 0.0% | 35.0 tok/s | 1.00x | ⚪ Autoregressivo Standalone |
| `qwen3-5-4b-super-coder-q4-0` | `NONE` | 0.0% | 35.0 tok/s | 1.00x | ⚪ Autoregressivo Standalone |
| `phi-4-mini-instruct-q5-k-m` | `NONE` | 0.0% | 35.0 tok/s | 1.00x | ⚪ Autoregressivo Standalone |
| `phi-4-mini-reasoning-q4-k-m` | `NONE` | 0.0% | 35.0 tok/s | 1.00x | ⚪ Autoregressivo Standalone |
| `phi-4-mini-instruct-q4-k-m` | `NONE` | 0.0% | 35.0 tok/s | 1.00x | ⚪ Autoregressivo Standalone |
| `mamba-codestral-7b-instruct-agnuxo` | `NONE` | 0.0% | 35.0 tok/s | 1.00x | ⚪ Autoregressivo Standalone |
| `synthagent-sft-ui-tars-1-5-7b-i1-iq3-m` | `NONE` | 0.0% | 35.0 tok/s | 1.00x | ⚪ Autoregressivo Standalone |
| `mamba-codestral-7b-v0-1-q4-k-m` | `NONE` | 0.0% | 35.0 tok/s | 1.00x | ⚪ Autoregressivo Standalone |
| `falcon3-mamba-7b-instruct-i1-iq3-m` | `NONE` | 0.0% | 35.0 tok/s | 1.00x | ⚪ Autoregressivo Standalone |
| `nvidia-nemotron-3-nano-4b-q4-k-m` | `NONE` | 0.0% | 35.0 tok/s | 1.00x | ⚪ Autoregressivo Standalone |
| `hy-mt2-7b-q4-k-m` | `NONE` | 0.0% | 35.0 tok/s | 1.00x | ⚪ Autoregressivo Standalone |
| `bonsai-27b-dspark-q4-1` | `Standalone Drafter` | N/A | N/A | 1.00x | Drafter Primário |
| `laguna-xs-2-1-dspark-q4-k-m` | `Standalone Drafter` | N/A | N/A | 1.00x | Drafter Primário |
| `qwen35-4b-dflash-q4-k-m` | `Standalone Drafter` | N/A | N/A | 1.00x | Drafter Primário |

---
*Fim do Dossiê de Inventário SOULS Engine v7. Gerado automaticamente via `souls_llms_inventory_viewer.py`.*