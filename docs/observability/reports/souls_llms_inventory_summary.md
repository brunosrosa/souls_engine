# 📊 SOULS SILICON OBSERVABILITY & HERMES LLM DOSSIER (v7 Canônica)
> **Gerado em:** 2026-10-09 01:08:07 | **Banco SSOT:** `Z:\souls_engine\.souls_data\db\souls_heuristic_vault.db`
> **Host Físico:** Intel Core i9-9880H (AVX2) | 32GB RAM | NVIDIA GeForce RTX 2060m (6GB GDDR6, WDDM 3.x)
> **Governança:** ADR-004 (Teto Útil WDDM 5.2GB) & ADR-007 (Desacoplamento dos Benchmark Types 1 a 5)

---

## 🏆 1. LEADERBOARD EXECUTIVO TIER 1: PRONTIDÃO PARA O HERMES AGENT
Esta matriz classifica exclusivamente os modelos de **Tier 1** (Live Chat, Coding e Tool-Calling) capazes de acelerar localmente o Hermes Agent sem colapso de VRAM.

| Rank | Modelo / Identificador | Params | Quant | Throughput (TPS) | TTFT (ms) | Pico VRAM | Headroom VRAM | Score E³ | Qualificação Hermes Agent |
| :---: | :--- | :---: | :---: | :---: | :---: | :---: | :---: | :---: | :--- |
| 1 | `arch-function-chat-3b-q5-k-m` | 3B | `Q5_K_M` | **42.5** | 45.0 | 4734 MB | 560 MB | 9999.00 | 🟢 CANDIDATO PRINCIPAL HERMES |
| 2 | `falcon3-mamba-7b-instruct-i1-iq3-m` | 7B | `IQ3_M` | **42.5** | 45.0 | 4726 MB | 568 MB | 9999.00 | 🟢 CANDIDATO PRINCIPAL HERMES |
| 3 | `huggingfacetb-smollm3-3b-q5-k-m` | 3B | `Q5_K_M` | **42.5** | 45.0 | 4726 MB | 568 MB | 9999.00 | 🟢 CANDIDATO PRINCIPAL HERMES |
| 4 | `hy-mt2-7b-q4-k-m` | 7B | `Q4_K_M` | **42.5** | 45.0 | 4726 MB | 568 MB | 9999.00 | 🟢 CANDIDATO PRINCIPAL HERMES |
| 5 | `mamba-codestral-7b-instruct-agnuxo` | 7B | `Q4_K_M` | **42.5** | 45.0 | 4723 MB | 571 MB | 9999.00 | 🟢 CANDIDATO PRINCIPAL HERMES |
| 6 | `mamba-codestral-7b-v0-1-q4-k-m` | 7B | `Q4_K_M` | **42.5** | 45.0 | 4730 MB | 564 MB | 9999.00 | 🟢 CANDIDATO PRINCIPAL HERMES |
| 7 | `nvidia-nemotron-3-nano-4b-q4-k-m` | 4B | `Q4_K_M` | **42.5** | 45.0 | 4730 MB | 564 MB | 9999.00 | 🟢 CANDIDATO PRINCIPAL HERMES |
| 8 | `parable-qwen3-4b-claude-fable-5-gguf-q4-k-m` | 4B | `Q4_K_M` | **42.5** | 45.0 | 4726 MB | 568 MB | 9999.00 | 🟢 CANDIDATO PRINCIPAL HERMES |
| 9 | `phi-4-mini-instruct-q4-k-m` | 3.8B | `Q4_K_M` | **42.5** | 45.0 | 4726 MB | 568 MB | 9999.00 | 🟢 CANDIDATO PRINCIPAL HERMES |
| 10 | `phi-4-mini-instruct-q5-k-m` | 3.8B | `Q5_K_M` | **42.5** | 45.0 | 4726 MB | 568 MB | 9999.00 | 🟢 CANDIDATO PRINCIPAL HERMES |
| 11 | `phi-4-mini-reasoning-q4-k-m` | 3.8B | `Q4_K_M` | **42.5** | 45.0 | 4726 MB | 568 MB | 9999.00 | 🟢 CANDIDATO PRINCIPAL HERMES |
| 12 | `qwen3-5-4b-agentic-coder-v4-i1-q4-k-s` | 4B | `I1-Q4_K_S` | **42.5** | 45.0 | 4724 MB | 570 MB | 9999.00 | 🟢 CANDIDATO PRINCIPAL HERMES |
| 13 | `qwen3-5-4b-super-coder-q4-0` | 4B | `Q4_0` | **42.5** | 45.0 | 4727 MB | 567 MB | 9999.00 | 🟢 CANDIDATO PRINCIPAL HERMES |
| 14 | `qwen3-5-4b-uncensored-hauhaucs-aggressive-mtp-q4-k-m` | 4B | `Q4_K_M` | **42.5** | 45.0 | 4727 MB | 567 MB | 9999.00 | 🟢 CANDIDATO PRINCIPAL HERMES |
| 15 | `synthagent-sft-ui-tars-1-5-7b-i1-iq3-m` | 7B | `IQ3_M` | **42.5** | 45.0 | 4724 MB | 570 MB | 9999.00 | 🟢 CANDIDATO PRINCIPAL HERMES |
| 16 | `zamba2-2-7b-instruct-v2-q4-0` | 2.7B | `Q4_0` | **42.5** | 45.0 | 4724 MB | 570 MB | 9999.00 | 🟢 CANDIDATO PRINCIPAL HERMES |

> **Critério de Ouro FinOps:** Modelos com mais de **35 tokens/s** e VRAM inferior a **4.200 MB** deixam margem folgada para até **8k tokens de contexto**, sendo os candidatos ideais para a experiência mestre do Hermes Agent.

---

## 🖥️ 2. INVENTÁRIO COMPLETO DE SILÍCIO POR TIER (38 Modelos)

### 🎯 Tier 1

| # | Identificador | Quant | Tamanho | Motor | TPS | TTFT (ms) | Pico VRAM | Score E³ | Status Operacional |
| :---: | :--- | :---: | :---: | :---: | :---: | :---: | :---: | :---: | :--- |
| 1 | `arch-function-chat-3b-q5-k-m` | `Q5_K_M` | 2.07 GB | `mock` | 42.5 | 45.0 | 4734 MB | 9999.00 | 🟢 CANDIDATO PRINCIPAL HERMES |
| 2 | `falcon3-mamba-7b-instruct-i1-iq3-m` | `IQ3_M` | 3.05 GB | `mock` | 42.5 | 45.0 | 4726 MB | 9999.00 | 🟢 CANDIDATO PRINCIPAL HERMES |
| 3 | `huggingfacetb-smollm3-3b-q5-k-m` | `Q5_K_M` | 2.06 GB | `mock` | 42.5 | 45.0 | 4726 MB | 9999.00 | 🟢 CANDIDATO PRINCIPAL HERMES |
| 4 | `hy-mt2-7b-q4-k-m` | `Q4_K_M` | 4.31 GB | `mock` | 42.5 | 45.0 | 4726 MB | 9999.00 | 🟢 CANDIDATO PRINCIPAL HERMES |
| 5 | `mamba-codestral-7b-instruct-agnuxo` | `Q4_K_M` | 4.07 GB | `mock` | 42.5 | 45.0 | 4723 MB | 9999.00 | 🟢 CANDIDATO PRINCIPAL HERMES |
| 6 | `mamba-codestral-7b-v0-1-q4-k-m` | `Q4_K_M` | 3.86 GB | `mock` | 42.5 | 45.0 | 4730 MB | 9999.00 | 🟢 CANDIDATO PRINCIPAL HERMES |
| 7 | `nvidia-nemotron-3-nano-4b-q4-k-m` | `Q4_K_M` | 2.64 GB | `mock` | 42.5 | 45.0 | 4730 MB | 9999.00 | 🟢 CANDIDATO PRINCIPAL HERMES |
| 8 | `parable-qwen3-4b-claude-fable-5-gguf-q4-k-m` | `Q4_K_M` | 2.33 GB | `mock` | 42.5 | 45.0 | 4726 MB | 9999.00 | 🟢 CANDIDATO PRINCIPAL HERMES |
| 9 | `phi-4-mini-instruct-q4-k-m` | `Q4_K_M` | 2.32 GB | `mock` | 42.5 | 45.0 | 4726 MB | 9999.00 | 🟢 CANDIDATO PRINCIPAL HERMES |
| 10 | `phi-4-mini-instruct-q5-k-m` | `Q5_K_M` | 2.65 GB | `mock` | 42.5 | 45.0 | 4726 MB | 9999.00 | 🟢 CANDIDATO PRINCIPAL HERMES |
| 11 | `phi-4-mini-reasoning-q4-k-m` | `Q4_K_M` | 2.32 GB | `mock` | 42.5 | 45.0 | 4726 MB | 9999.00 | 🟢 CANDIDATO PRINCIPAL HERMES |
| 12 | `qwen3-5-4b-agentic-coder-v4-i1-q4-k-s` | `I1-Q4_K_S` | 2.45 GB | `mock` | 42.5 | 45.0 | 4724 MB | 9999.00 | 🟢 CANDIDATO PRINCIPAL HERMES |
| 13 | `qwen3-5-4b-super-coder-q4-0` | `Q4_0` | 2.43 GB | `mock` | 42.5 | 45.0 | 4727 MB | 9999.00 | 🟢 CANDIDATO PRINCIPAL HERMES |
| 14 | `qwen3-5-4b-uncensored-hauhaucs-aggressive-mtp-q4-k-m` | `Q4_K_M` | 2.60 GB | `mock` | 42.5 | 45.0 | 4727 MB | 9999.00 | 🟢 CANDIDATO PRINCIPAL HERMES |
| 15 | `synthagent-sft-ui-tars-1-5-7b-i1-iq3-m` | `IQ3_M` | 3.33 GB | `mock` | 42.5 | 45.0 | 4724 MB | 9999.00 | 🟢 CANDIDATO PRINCIPAL HERMES |
| 16 | `zamba2-2-7b-instruct-v2-q4-0` | `Q4_0` | 2.08 GB | `mock` | 42.5 | 45.0 | 4724 MB | 9999.00 | 🟢 CANDIDATO PRINCIPAL HERMES |
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
| `arch-function-chat-3b-q5-k-m` | Tier 1 | N/A | N/A | N/A | 🟡 Aguardando Bateria Type 2 |
| `falcon3-mamba-7b-instruct-i1-iq3-m` | Tier 1 | N/A | N/A | N/A | 🟡 Aguardando Bateria Type 2 |
| `huggingfacetb-smollm3-3b-q5-k-m` | Tier 1 | N/A | N/A | N/A | 🟡 Aguardando Bateria Type 2 |
| `hy-mt2-7b-q4-k-m` | Tier 1 | N/A | N/A | N/A | 🟡 Aguardando Bateria Type 2 |
| `mamba-codestral-7b-instruct-agnuxo` | Tier 1 | N/A | N/A | N/A | 🟡 Aguardando Bateria Type 2 |
| `mamba-codestral-7b-v0-1-q4-k-m` | Tier 1 | N/A | N/A | N/A | 🟡 Aguardando Bateria Type 2 |
| `nvidia-nemotron-3-nano-4b-q4-k-m` | Tier 1 | N/A | N/A | N/A | 🟡 Aguardando Bateria Type 2 |
| `parable-qwen3-4b-claude-fable-5-gguf-q4-k-m` | Tier 1 | N/A | N/A | N/A | 🟡 Aguardando Bateria Type 2 |
| `phi-4-mini-instruct-q4-k-m` | Tier 1 | N/A | N/A | N/A | 🟡 Aguardando Bateria Type 2 |
| `phi-4-mini-instruct-q5-k-m` | Tier 1 | N/A | N/A | N/A | 🟡 Aguardando Bateria Type 2 |
| `phi-4-mini-reasoning-q4-k-m` | Tier 1 | N/A | N/A | N/A | 🟡 Aguardando Bateria Type 2 |
| `qwen3-5-4b-agentic-coder-v4-i1-q4-k-s` | Tier 1 | N/A | N/A | N/A | 🟡 Aguardando Bateria Type 2 |
| `qwen3-5-4b-super-coder-q4-0` | Tier 1 | N/A | N/A | N/A | 🟡 Aguardando Bateria Type 2 |
| `qwen3-5-4b-uncensored-hauhaucs-aggressive-mtp-q4-k-m` | Tier 1 | N/A | N/A | N/A | 🟡 Aguardando Bateria Type 2 |
| `synthagent-sft-ui-tars-1-5-7b-i1-iq3-m` | Tier 1 | N/A | N/A | N/A | 🟡 Aguardando Bateria Type 2 |
| `zamba2-2-7b-instruct-v2-q4-0` | Tier 1 | N/A | N/A | N/A | 🟡 Aguardando Bateria Type 2 |
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