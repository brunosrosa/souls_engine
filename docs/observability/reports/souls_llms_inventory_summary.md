# 📊 SOULS SILICON OBSERVABILITY & HERMES LLM DOSSIER (v7 Canônica)
> **Gerado em:** 2026-10-09 17:47:51 | **Banco SSOT:** `Z:\souls_engine\.souls_data\db\souls_heuristic_vault.db`
> **Host Físico:** Intel Core i9-9880H (AVX2) | 32GB RAM | NVIDIA GeForce RTX 2060m (6GB GDDR6, WDDM 3.x)
> **Governança:** ADR-004 (Teto Útil WDDM 5.2GB) & ADR-007 (Desacoplamento dos Benchmark Types 1 a 5)

---

## 🏆 1. LEADERBOARD EXECUTIVO TIER 1: PRONTIDÃO PARA O HERMES AGENT
Esta matriz classifica exclusivamente os modelos de **Tier 1** (Live Chat, Coding e Tool-Calling) capazes de acelerar localmente o Hermes Agent sem colapso de VRAM.

| Rank | Modelo / Identificador | Params | Quant | Throughput (TPS) | TTFT (ms) | Pico VRAM | Headroom VRAM | Score E³ | Qualificação Hermes Agent |
| :---: | :--- | :---: | :---: | :---: | :---: | :---: | :---: | :---: | :--- |
| 1 | `zamba2-2-7b-instruct-v2-q4-0` | 2.7B | `Q4_0` | **44.1** | 121.0 | 1205 MB | 4089 MB | 8727.90 | 🟢 CANDIDATO PRINCIPAL HERMES |
| 2 | `arch-function-chat-3b-q5-k-m` | 3B | `Q5_K_M` | **44.1** | 1695.7 | 1234 MB | 4060 MB | 4794.68 | 🟢 CANDIDATO PRINCIPAL HERMES |
| 3 | `huggingfacetb-smollm3-3b-q5-k-m` | 3B | `Q5_K_M` | **37.8** | 2400.1 | 1233 MB | 4061 MB | 4692.41 | 🟢 CANDIDATO PRINCIPAL HERMES |
| 4 | `qwen3-5-4b-uncensored-hauhaucs-aggressive-mtp-q4-k-m` | 4B | `Q4_K_M` | **35.4** | 2081.5 | 1201 MB | 4093 MB | 5114.44 | 🟢 CANDIDATO PRINCIPAL HERMES |
| 5 | `phi-4-mini-instruct-q4-k-m` | 3.8B | `Q4_K_M` | **34.7** | 11046.0 | 1247 MB | 4047 MB | 3248.92 | 🔵 SUBAGENTE DE SUPORTE |
| 6 | `parable-qwen3-4b-claude-fable-5-gguf-q4-k-m` | 4B | `Q4_K_M` | **32.8** | 2833.9 | 1203 MB | 4091 MB | 3925.39 | 🔵 SUBAGENTE DE SUPORTE |
| 7 | `qwen3-5-4b-agentic-coder-v4-i1-q4-k-s` | 4B | `I1-Q4_K_S` | **32.6** | 1792.1 | 1200 MB | 4094 MB | 5080.50 | 🔵 SUBAGENTE DE SUPORTE |
| 8 | `phi-4-mini-instruct-q5-k-m` | 3.8B | `Q5_K_M` | **30.8** | 2640.5 | 1205 MB | 4089 MB | 3710.82 | 🔵 SUBAGENTE DE SUPORTE |
| 9 | `qwen3-5-4b-super-coder-q4-0` | 4B | `Q4_0` | **28.4** | 2852.5 | 353 MB | 4941 MB | 2654.37 | 🔵 SUBAGENTE DE SUPORTE |
| 10 | `phi-4-mini-reasoning-q4-k-m` | 3.8B | `Q4_K_M` | **28.1** | 2338.9 | 753 MB | 4541 MB | 3074.59 | 🔵 SUBAGENTE DE SUPORTE |
| 11 | `synthagent-sft-ui-tars-1-5-7b-i1-iq3-m` | 7B | `IQ3_M` | **26.4** | 3966.4 | 1205 MB | 4089 MB | 2859.39 | 🔵 SUBAGENTE DE SUPORTE |
| 12 | `mamba-codestral-7b-v0-1-q4-k-m` | 7B | `Q4_K_M` | **25.7** | 1738.5 | 1203 MB | 4091 MB | 4401.66 | 🔵 SUBAGENTE DE SUPORTE |
| 13 | `mamba-codestral-7b-instruct-agnuxo` | 7B | `Q4_K_M` | **25.5** | 2336.0 | 1789 MB | 3505 MB | 4078.03 | 🔵 SUBAGENTE DE SUPORTE |
| 14 | `falcon3-mamba-7b-instruct-i1-iq3-m` | 7B | `IQ3_M` | **25.0** | 6703.1 | 1233 MB | 4061 MB | 2544.45 | 🔵 SUBAGENTE DE SUPORTE |
| 15 | `nvidia-nemotron-3-nano-4b-q4-k-m` | 4B | `Q4_K_M` | **23.6** | 7736.0 | 1203 MB | 4091 MB | 3213.68 | 🔵 SUBAGENTE DE SUPORTE |
| 16 | `hy-mt2-7b-q4-k-m` | 7B | `Q4_K_M` | **23.3** | 3330.1 | 1231 MB | 4063 MB | 3059.28 | 🔵 SUBAGENTE DE SUPORTE |

> **Critério de Ouro FinOps:** Modelos com mais de **35 tokens/s** e VRAM inferior a **4.200 MB** deixam margem folgada para até **8k tokens de contexto**, sendo os candidatos ideais para a experiência mestre do Hermes Agent.

---

## 🖥️ 2. INVENTÁRIO COMPLETO DE SILÍCIO POR TIER (38 Modelos)

### 🎯 Tier 1

| # | Identificador | Quant | Tamanho | Motor | TPS | TTFT (ms) | Pico VRAM | Score E³ | Status Operacional |
| :---: | :--- | :---: | :---: | :---: | :---: | :---: | :---: | :---: | :--- |
| 1 | `zamba2-2-7b-instruct-v2-q4-0` | `Q4_0` | 2.08 GB | `llama-cuda` | 44.1 | 121.0 | 1205 MB | 8727.90 | 🟢 CANDIDATO PRINCIPAL HERMES |
| 2 | `arch-function-chat-3b-q5-k-m` | `Q5_K_M` | 2.07 GB | `llama-cuda` | 44.1 | 1695.7 | 1234 MB | 4794.68 | 🟢 CANDIDATO PRINCIPAL HERMES |
| 3 | `huggingfacetb-smollm3-3b-q5-k-m` | `Q5_K_M` | 2.06 GB | `llama-cuda` | 37.8 | 2400.1 | 1233 MB | 4692.41 | 🟢 CANDIDATO PRINCIPAL HERMES |
| 4 | `qwen3-5-4b-uncensored-hauhaucs-aggressive-mtp-q4-k-m` | `Q4_K_M` | 2.60 GB | `llama-cuda` | 35.4 | 2081.5 | 1201 MB | 5114.44 | 🟢 CANDIDATO PRINCIPAL HERMES |
| 5 | `phi-4-mini-instruct-q4-k-m` | `Q4_K_M` | 2.32 GB | `llama-cuda` | 34.7 | 11046.0 | 1247 MB | 3248.92 | 🔵 SUBAGENTE DE SUPORTE |
| 6 | `parable-qwen3-4b-claude-fable-5-gguf-q4-k-m` | `Q4_K_M` | 2.33 GB | `llama-cuda` | 32.8 | 2833.9 | 1203 MB | 3925.39 | 🔵 SUBAGENTE DE SUPORTE |
| 7 | `qwen3-5-4b-agentic-coder-v4-i1-q4-k-s` | `I1-Q4_K_S` | 2.45 GB | `llama-cuda` | 32.6 | 1792.1 | 1200 MB | 5080.50 | 🔵 SUBAGENTE DE SUPORTE |
| 8 | `phi-4-mini-instruct-q5-k-m` | `Q5_K_M` | 2.65 GB | `llama-cuda` | 30.8 | 2640.5 | 1205 MB | 3710.82 | 🔵 SUBAGENTE DE SUPORTE |
| 9 | `qwen3-5-4b-super-coder-q4-0` | `Q4_0` | 2.43 GB | `llama-cuda` | 28.4 | 2852.5 | 353 MB | 2654.37 | 🔵 SUBAGENTE DE SUPORTE |
| 10 | `phi-4-mini-reasoning-q4-k-m` | `Q4_K_M` | 2.32 GB | `llama-cuda` | 28.1 | 2338.9 | 753 MB | 3074.59 | 🔵 SUBAGENTE DE SUPORTE |
| 11 | `synthagent-sft-ui-tars-1-5-7b-i1-iq3-m` | `IQ3_M` | 3.33 GB | `llama-cuda` | 26.4 | 3966.4 | 1205 MB | 2859.39 | 🔵 SUBAGENTE DE SUPORTE |
| 12 | `mamba-codestral-7b-v0-1-q4-k-m` | `Q4_K_M` | 3.86 GB | `llama-cuda` | 25.7 | 1738.5 | 1203 MB | 4401.66 | 🔵 SUBAGENTE DE SUPORTE |
| 13 | `mamba-codestral-7b-instruct-agnuxo` | `Q4_K_M` | 4.07 GB | `llama-cuda` | 25.5 | 2336.0 | 1789 MB | 4078.03 | 🔵 SUBAGENTE DE SUPORTE |
| 14 | `falcon3-mamba-7b-instruct-i1-iq3-m` | `IQ3_M` | 3.05 GB | `llama-cuda` | 25.0 | 6703.1 | 1233 MB | 2544.45 | 🔵 SUBAGENTE DE SUPORTE |
| 15 | `nvidia-nemotron-3-nano-4b-q4-k-m` | `Q4_K_M` | 2.64 GB | `llama-cuda` | 23.6 | 7736.0 | 1203 MB | 3213.68 | 🔵 SUBAGENTE DE SUPORTE |
| 16 | `hy-mt2-7b-q4-k-m` | `Q4_K_M` | 4.31 GB | `llama-cuda` | 23.3 | 3330.1 | 1231 MB | 3059.28 | 🔵 SUBAGENTE DE SUPORTE |
### 🎯 Tier 0.5

| # | Identificador | Quant | Tamanho | Motor | TPS | TTFT (ms) | Pico VRAM | Score E³ | Status Operacional |
| :---: | :--- | :---: | :---: | :---: | :---: | :---: | :---: | :---: | :--- |
| 1 | `bitnet-b1.58-2b-4t` | `I2_S` | 1.11 GB | `llama.cpp` | N/A | N/A | 0 MB | N/A | 🟡 AGUARDANDO BENCHMARK |
| 2 | `deepseek-r1-distill-qwen-1-5b-q8-0` | `Q8_0` | 1.76 GB | `llama.cpp` | N/A | N/A | 0 MB | N/A | 🟡 AGUARDANDO BENCHMARK |
| 3 | `lfm2-5-1-2b-instruct-q8-0` | `Q8_0` | 1.16 GB | `llama.cpp` | N/A | N/A | 0 MB | N/A | 🟡 AGUARDANDO BENCHMARK |
| 4 | `mamba-790m-hf-q4-k-m` | `Q4_K_M` | 459.3 MB | `llama.cpp` | N/A | N/A | 0 MB | N/A | 🟡 AGUARDANDO BENCHMARK |
| 5 | `smollm2-360m-instruct-q8-0` | `Q8_0` | 368.5 MB | `llama.cpp` | N/A | N/A | 0 MB | N/A | 🟡 AGUARDANDO BENCHMARK |
### 🎯 Tier 0

| # | Identificador | Quant | Tamanho | Motor | TPS | TTFT (ms) | Pico VRAM | Score E³ | Status Operacional |
| :---: | :--- | :---: | :---: | :---: | :---: | :---: | :---: | :---: | :--- |
| 1 | `qwen35-4b-dflash-q4-k-m` | `Q4_K_M` | 4.0 MB | `mock` | 110.0 | 45.0 | 1427 MB | 9999.00 | 🟢 CANDIDATO PRINCIPAL HERMES |
| 2 | `neuralai-mamba-k1-v3-q4-k-m` | `Q4_K_M` | 85.7 MB | `llama.cpp` | N/A | N/A | 0 MB | N/A | 🟡 AGUARDANDO BENCHMARK |
| 3 | `smollm-135m-instruct-q4-k-m` | `Q4_K_M` | 100.6 MB | `llama.cpp` | N/A | N/A | 0 MB | N/A | 🟡 AGUARDANDO BENCHMARK |
| 4 | `smollm2-135m-reasoning-5k-q4-k-m` | `Q4_K_M` | 100.6 MB | `llama.cpp` | N/A | N/A | 0 MB | N/A | 🟡 AGUARDANDO BENCHMARK |
### 🎯 Tier 3

| # | Identificador | Quant | Tamanho | Motor | TPS | TTFT (ms) | Pico VRAM | Score E³ | Status Operacional |
| :---: | :--- | :---: | :---: | :---: | :---: | :---: | :---: | :---: | :--- |
| 1 | `bonsai-27b-q1-0-ai-joe` | `Q1_0` | 3.54 GB | `llama.cpp` | N/A | N/A | 0 MB | N/A | 🟡 AGUARDANDO BENCHMARK |
| 2 | `bonsai-27b-q1-0-lmstudio` | `Q1_0` | 3.54 GB | `llama.cpp` | N/A | N/A | 0 MB | N/A | 🟡 AGUARDANDO BENCHMARK |
| 3 | `deepseek-v4-pro-qwen3-5-4b-mtp-q4-k-s` | `Q4_K_S` | 2.45 GB | `llama.cpp` | N/A | N/A | 0 MB | N/A | 🟡 AGUARDANDO BENCHMARK |
| 4 | `gemma-4-e2b-it-q4-k-m` | `Q4_K_M` | 3.19 GB | `llama.cpp` | N/A | N/A | 0 MB | N/A | 🟡 AGUARDANDO BENCHMARK |
| 5 | `gemma-4-e2b-it-ultra-uncensored-heretic-q4-k-m` | `Q4_K_M` | 3.19 GB | `llama.cpp` | N/A | N/A | 0 MB | N/A | 🟡 AGUARDANDO BENCHMARK |
| 6 | `microsoft-fara-7b-iq3-m` | `IQ3_M` | 3.33 GB | `llama.cpp` | N/A | N/A | 0 MB | N/A | 🟡 AGUARDANDO BENCHMARK |
| 7 | `microsoft-fara1-5-4b-f16-gguf-q4-k-m` | `Q4_K_M` | 2.52 GB | `llama.cpp` | N/A | N/A | 0 MB | N/A | 🟡 AGUARDANDO BENCHMARK |
| 8 | `nimbus-4b-q4-k-m` | `Q4_K_M` | 2.52 GB | `llama.cpp` | N/A | N/A | 0 MB | N/A | 🟡 AGUARDANDO BENCHMARK |
### 🎯 Tier 4

| # | Identificador | Quant | Tamanho | Motor | TPS | TTFT (ms) | Pico VRAM | Score E³ | Status Operacional |
| :---: | :--- | :---: | :---: | :---: | :---: | :---: | :---: | :---: | :--- |
| 1 | `bonsai-27b-dspark-q4-1` | `Q4_1` | 1.66 GB | `llama.cpp` | N/A | N/A | 0 MB | N/A | 🟡 AGUARDANDO BENCHMARK |
| 2 | `laguna-xs-2-1-dspark-q4-k-m` | `Q4_K_M` | 297.6 MB | `llama.cpp` | N/A | N/A | 0 MB | N/A | 🟡 AGUARDANDO BENCHMARK |
| 3 | `qwen35-4b-dflash-q4-k-m` | `Q4_K_M` | 363.8 MB | `llama.cpp` | N/A | N/A | 0 MB | N/A | 🟡 AGUARDANDO BENCHMARK |
### 🎯 Tier 2

| # | Identificador | Quant | Tamanho | Motor | TPS | TTFT (ms) | Pico VRAM | Score E³ | Status Operacional |
| :---: | :--- | :---: | :---: | :---: | :---: | :---: | :---: | :---: | :--- |
| 1 | `laguna-xs-2-1-apex-i-compact` | `GGUF` | 14.69 GB | `llama.cpp` | N/A | N/A | 0 MB | N/A | 🟡 AGUARDANDO BENCHMARK |
| 2 | `rnj-1-instruct-q4-k-m` | `Q4_K_M` | 4.76 GB | `llama.cpp` | N/A | N/A | 0 MB | N/A | 🟡 AGUARDANDO BENCHMARK |

---

## 🧬 3. COLISEU COGNITIVO (BENCHMARK TYPE 2: QUALIDADE & RIGIDEZ)
Aferição de integridade lógica necessária para subagentes executivos e MCP Tools:

| Modelo | Tier | 🦀 Track 2A (Rust AST Code) | 🛠️ Track 2B (MCP JSON Tools) | 🧠 Track 2C (CoT Reasoning) | Veredito Cognitivo |
| :--- | :---: | :---: | :---: | :---: | :--- |
| `zamba2-2-7b-instruct-v2-q4-0` | Tier 1 | 95% | 100% | 90% | 🟢 Aprovado em Sintaxe Estrita |
| `arch-function-chat-3b-q5-k-m` | Tier 1 | 95% | 10% | 90% | 🟢 Aprovado em Sintaxe Estrita |
| `huggingfacetb-smollm3-3b-q5-k-m` | Tier 1 | 95% | 85% | 90% | 🟢 Aprovado em Sintaxe Estrita |
| `qwen3-5-4b-uncensored-hauhaucs-aggressive-mtp-q4-k-m` | Tier 1 | 95% | 85% | 90% | 🟢 Aprovado em Sintaxe Estrita |
| `phi-4-mini-instruct-q4-k-m` | Tier 1 | 95% | 85% | 90% | 🟢 Aprovado em Sintaxe Estrita |
| `parable-qwen3-4b-claude-fable-5-gguf-q4-k-m` | Tier 1 | 95% | 85% | 90% | 🟢 Aprovado em Sintaxe Estrita |
| `qwen3-5-4b-agentic-coder-v4-i1-q4-k-s` | Tier 1 | 95% | 85% | 90% | 🟢 Aprovado em Sintaxe Estrita |
| `phi-4-mini-instruct-q5-k-m` | Tier 1 | 20% | 85% | 90% | 🟢 Aprovado em Sintaxe Estrita |
| `qwen3-5-4b-super-coder-q4-0` | Tier 1 | 95% | 85% | 90% | 🟢 Aprovado em Sintaxe Estrita |
| `phi-4-mini-reasoning-q4-k-m` | Tier 1 | 95% | 10% | 90% | 🟢 Aprovado em Sintaxe Estrita |
| `synthagent-sft-ui-tars-1-5-7b-i1-iq3-m` | Tier 1 | 95% | 10% | 90% | 🟢 Aprovado em Sintaxe Estrita |
| `mamba-codestral-7b-v0-1-q4-k-m` | Tier 1 | 95% | 85% | 90% | 🟢 Aprovado em Sintaxe Estrita |
| `mamba-codestral-7b-instruct-agnuxo` | Tier 1 | 20% | 10% | 90% | 🔴 Falha em Validação Estrita |
| `falcon3-mamba-7b-instruct-i1-iq3-m` | Tier 1 | 95% | 10% | 90% | 🟢 Aprovado em Sintaxe Estrita |
| `nvidia-nemotron-3-nano-4b-q4-k-m` | Tier 1 | 95% | 10% | 90% | 🟢 Aprovado em Sintaxe Estrita |
| `hy-mt2-7b-q4-k-m` | Tier 1 | 20% | 85% | 90% | 🟢 Aprovado em Sintaxe Estrita |
| `bitnet-b1.58-2b-4t` | Tier 0.5 | N/A | N/A | N/A | 🟡 Aguardando Bateria Type 2 |
| `deepseek-r1-distill-qwen-1-5b-q8-0` | Tier 0.5 | N/A | N/A | N/A | 🟡 Aguardando Bateria Type 2 |
| `lfm2-5-1-2b-instruct-q8-0` | Tier 0.5 | N/A | N/A | N/A | 🟡 Aguardando Bateria Type 2 |
| `mamba-790m-hf-q4-k-m` | Tier 0.5 | N/A | N/A | N/A | 🟡 Aguardando Bateria Type 2 |
| `smollm2-360m-instruct-q8-0` | Tier 0.5 | N/A | N/A | N/A | 🟡 Aguardando Bateria Type 2 |

---

## 👁️ 4. MODELOS MULTIMODAIS & PROJETORES DE VISÃO MMPROJ (TIER 3) (8)

| Modelo VLM | Quant | Tamanho | Projetor Sidecar Pareado | Status Multimodal |
| :--- | :---: | :---: | :---: | :--- |
| `bonsai-27b-q1-0-ai-joe` | `Q1_0` | 3.54 GB | ❌ Não detectado | Pronto para OCR/VQA |
| `bonsai-27b-q1-0-lmstudio` | `Q1_0` | 3.54 GB | ✅ Sim (Pareado) | Pronto para OCR/VQA |
| `deepseek-v4-pro-qwen3-5-4b-mtp-q4-k-s` | `Q4_K_S` | 2.45 GB | ✅ Sim (Pareado) | Pronto para OCR/VQA |
| `gemma-4-e2b-it-q4-k-m` | `Q4_K_M` | 3.19 GB | ✅ Sim (Pareado) | Pronto para OCR/VQA |
| `gemma-4-e2b-it-ultra-uncensored-heretic-q4-k-m` | `Q4_K_M` | 3.19 GB | ✅ Sim (Pareado) | Pronto para OCR/VQA |
| `microsoft-fara-7b-iq3-m` | `IQ3_M` | 3.33 GB | ✅ Sim (Pareado) | Pronto para OCR/VQA |
| `microsoft-fara1-5-4b-f16-gguf-q4-k-m` | `Q4_K_M` | 2.52 GB | ✅ Sim (Pareado) | Pronto para OCR/VQA |
| `nimbus-4b-q4-k-m` | `Q4_K_M` | 2.52 GB | ✅ Sim (Pareado) | Pronto para OCR/VQA |

---

## ⚡ 5. ACELERAÇÃO ESPECULATIVA & MODELOS DRAFTER (TIER 4) (3)

| Modelo Drafter | Tamanho | Taxa de Aceitação Alpha (α) | Speedup Projetado | Veredito FinOps |
| :--- | :---: | :---: | :---: | :--- |
| `bonsai-27b-dspark-q4-1` | 1.66 GB | N/A (Aguardando Type 4) | 1.00x | ⏳ Pendente de Combate |
| `laguna-xs-2-1-dspark-q4-k-m` | 297.6 MB | N/A (Aguardando Type 4) | 1.00x | ⏳ Pendente de Combate |
| `qwen35-4b-dflash-q4-k-m` | 363.8 MB | N/A (Aguardando Type 4) | 1.00x | ⏳ Pendente de Combate |

---
*Fim do Dossiê de Inventário SOULS Engine v7. Gerado automaticamente via `souls_llms_inventory_viewer.py`.*