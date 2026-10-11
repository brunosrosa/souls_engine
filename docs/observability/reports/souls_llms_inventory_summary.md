# 📊 SOULS SILICON OBSERVABILITY & HERMES LLM DOSSIER (v7 Canônica)
> **Gerado em:** 2026-10-10 21:01:37 | **Banco SSOT:** `Z:\souls_engine\.souls_data\db\souls_heuristic_vault.db`
> **Host Físico Bare-Metal:** Intel Core i9-9880H (AVX2) | 32GB RAM DDR4 | NVIDIA GeForce RTX 2060m (6GB GDDR6, WDDM 3.x)
> **Governança:** ADR-004 (Teto Útil WDDM 5.2GB), ADR-007 (Desacoplamento Types 1 a 5) & Item 5 da Cost Matrix Specification

---

## 🏆 1. MATRIZ EXECUTIVA DE ROTEAMENTO POR PERFIL DE TAREFA (PARETO FRONTIER & FALLBACK CHAINS)
Esta seção introduz a camada analítica de **Scoring Multi-Dimensional e Fronteira de Pareto** inspirada diretamente na especificação canônica da *LLM Benchmark Cost Matrix Crate* (`llm_benchmark_cost_matrix_crate_specification.md`, Item 5).

### 📐 1.0 Fundamentação Matemática do Roteamento FinOps Bare-Metal
Ao contrário de serviços em nuvem tarifados em dólares por milhão de tokens, o custo bare-metal é medido pela **pressão física sobre o silício** ($C_{\text{silicon}}$), combinando ocupação de VRAM sob o Windows DWM, tempo até o primeiro token (TTFT) e latência de geração:

$$C_{\text{silicon}} = 0.50 \cdot \left(\frac{\text{VRAM}_{\text{peak}}}{5294}\right) + 0.30 \cdot \left(\frac{\text{TTFT}_{\text{sec}}}{3.0}\right) + 0.20 \cdot \left(\frac{30.0}{\text{TPS} + 1.0}\right)$$

O **Retorno de Silício Amortecido ($V_{\text{task}}$)** aplica amortecimento logarítmico para mitigar assimetrias de modelos ultraleves com acurácia insuficiente:

$$V_{\text{task}} = \frac{S_{\text{task}}}{\ln(1.0 + C_{\text{silicon}} + 1.0)}$$

Um modelo $A$ **domina** $B$ na Fronteira de Pareto se $S_{\text{task}}(A) \ge S_{\text{task}}(B)$ e $C_{\text{silicon}}(A) \le C_{\text{silicon}}(B)$ (com ao menos uma desigualdade estrita).

### 🦀 1.1 Perfil: Engenharia de Software & Coding Bare-Metal ($S_{\text{coding}}$)
> **Fórmula do Perfil:** $S_{\text{coding}} = 0.40 \cdot \text{Rust AST} + 0.25 \cdot \text{Ctx 8k Needle} + 0.20 \cdot \text{TPS}_{\text{norm}} + 0.15 \cdot \text{TTFT}_{\text{norm}}$
> **Alvo Operacional:** Geração de crates Rust com grammar masking estrita, refatoração de AST e patches compiláveis no primeiro turno.

| Rank | Modelo / Identificador | Params | Quant | Score Coding ($S$) | Custo Silício ($C$) | ROI Silício ($V$) | Throughput | Ctx Efetivo | Designação de Roteamento |
| :---: | :--- | :---: | :---: | :---: | :---: | :---: | :---: | :---: | :--- |
| 1 | `arch-function-chat-3b-q5-k-m` | 3B | `Q5_K_M` | **97.4** | 0.283 | 118.0 | 47.1 tok/s | 8k | 🏆 Campeão Absoluto (Qualidade & ROI) |
| 2 | `mamba-codestral-7b-v0-1-q4-k-m` | 7B | `Q4_K_M` | **92.7** | 0.310 | 110.7 | 35.4 tok/s | 8k | 🔗 Fallback Local #1 |
| 3 | `qwen3-5-4b-super-coder-q4-0` | 4B | `Q4_0` | **86.6** | 0.275 | 105.4 | 50.1 tok/s | 8k | 🔗 Fallback Local #2 (Pareto) |
| 4 | `qwen3-5-4b-agentic-coder-v4-i1-q4-k-s` | 4B | `I1-Q4_K_S` | **85.1** | 0.271 | 103.8 | 46.6 tok/s | 8k | 🔗 Fallback Local #3 (Pareto) |
| 5 | `huggingfacetb-smollm3-3b-q5-k-m` | 3B | `Q5_K_M` | **84.8** | 0.289 | 102.4 | 45.8 tok/s | 8k | ⚪ Alternativa Subordinada |
| 6 | `parable-qwen3-4b-claude-fable-5-gguf-q4-k-m` | 4B | `Q4_K_M` | **81.0** | 0.307 | 96.9 | 36.4 tok/s | 8k | ⚪ Alternativa Subordinada |
| 7 | `phi-4-mini-instruct-q5-k-m` | 3.8B | `Q5_K_M` | **81.0** | 0.310 | 96.7 | 36.4 tok/s | 8k | ⚪ Alternativa Subordinada |
| 8 | `phi-4-mini-instruct-q4-k-m` | 3.8B | `Q4_K_M` | **79.6** | 5.474 | 39.6 | 38.4 tok/s | 8k | ⚪ Alternativa Subordinada |

> 🏆 **Campeão de Qualidade:** `arch-function-chat-3b-q5-k-m` (Score: **97.4**, 47.1 TPS, 8k Estável).
> ⚡ **Campeão de Custo/Eficiência:** `mamba-codestral-7b-v0-1-q4-k-m` (ROI: **110.7**, VRAM reduzida a **1287 MB**, Custo $C$: **0.310**).

#### 🔗 Cadeia de Roteamento & Fallbacks Canônicos (Perfil Coding)
| Nível de Roteamento | Modelo Selecionado | Provedor / Hardware | Custo / Footprint | Throughput | Critério de Acionamento / SLA |
| :--- | :--- | :--- | :--- | :--- | :--- |
| **1. Primário Local** | `arch-function-chat-3b-q5-k-m` | dGPU RTX 2060m (Tier 1) | 1438 MB VRAM | 47.1 tok/s | Padrão para tarefas de codificação e geração de AST Rust. |
| **2. Fallback Local #1** | `mamba-codestral-7b-v0-1-q4-k-m` | dGPU RTX 2060m (Tier 1) | 1287 MB VRAM | 35.4 tok/s | Acionado se o primário estiver em concorrência ou em caso de crash. |
| **3. Fallback Local #2 (FinOps)** | `mamba-codestral-7b-v0-1-q4-k-m` | dGPU RTX 2060m (Tier 1) | 1287 MB VRAM | 35.4 tok/s | Acionado quando houver pressão de memória de contexto (headroom > 2.4GB). |
| **4. Fallback Cloud #1** | `claude-3-7-sonnet` | Nous Portal (API Cloud) | $3.40 / 1M tokens | 68.0 tok/s | Acionado se refatoração envolver >3 crates ou compilação falhar 2x seguidas. |
| **5. Fallback Cloud #2** | `deepseek-v3` / `qwen-2.5-coder-32b` | Nous Portal / OpenRouter | $0.18 - $0.28 / 1M | ~100 tok/s | Acionado para geração em lote de boilerplate massivo de baixo custo. |

---

### 🛠️ 1.2 Perfil: Agentes Autônomos & Chamada de Ferramentas MCP ($S_{\text{agentic}}$)
> **Fórmula do Perfil:** $S_{\text{agentic}} = 0.45 \cdot \text{MCP JSON Tools} + 0.25 \cdot \text{Ctx 8k Needle} + 0.15 \cdot \text{TTFT}_{\text{norm}} + 0.15 \cdot \text{TPS}_{\text{norm}}$
> **Alvo Operacional:** Execução de loops MCP no Hermes Agent, emissão estrita de JSON Schema e tolerância a diálogos longos.

| Rank | Modelo / Identificador | Params | Quant | Score Agentic ($S$) | Custo Silício ($C$) | ROI Silício ($V$) | Throughput | JSON Acurácia | Designação de Roteamento |
| :---: | :--- | :---: | :---: | :---: | :---: | :---: | :---: | :---: | :--- |
| 1 | `qwen3-5-4b-super-coder-q4-0` | 4B | `Q4_0` | **81.4** | 0.275 | 99.0 | 50.1 tok/s | 60% | 🏆 Campeão de Qualidade |
| 2 | `qwen3-5-4b-uncensored-hauhaucs-aggressive-mtp-q4-k-m` | 4B | `Q4_K_M` | **81.2** | 0.261 | 99.5 | 51.2 tok/s | 60% | ⚡ Campeão de Custo/Eficiência |
| 3 | `nvidia-nemotron-3-nano-4b-q4-k-m` | 4B | `Q4_K_M` | **80.8** | 0.409 | 91.9 | 47.8 tok/s | 60% | 🔗 Fallback Local #1 |
| 4 | `qwen3-5-4b-agentic-coder-v4-i1-q4-k-s` | 4B | `I1-Q4_K_S` | **80.3** | 0.271 | 97.9 | 46.6 tok/s | 60% | 🔗 Fallback Local #2 |
| 5 | `arch-function-chat-3b-q5-k-m` | 3B | `Q5_K_M` | **78.2** | 0.283 | 94.7 | 47.1 tok/s | 55% | ⚪ Alternativa Subordinada |
| 6 | `phi-4-mini-reasoning-q4-k-m` | 3.8B | `Q4_K_M` | **78.0** | 0.297 | 93.8 | 39.4 tok/s | 60% | ⚪ Alternativa Subordinada |
| 7 | `huggingfacetb-smollm3-3b-q5-k-m` | 3B | `Q5_K_M` | **77.7** | 0.289 | 93.8 | 45.8 tok/s | 55% | ⚪ Alternativa Subordinada |
| 8 | `parable-qwen3-4b-claude-fable-5-gguf-q4-k-m` | 4B | `Q4_K_M` | **77.2** | 0.307 | 92.4 | 36.4 tok/s | 60% | ⚪ Alternativa Subordinada |

> 🏆 **Campeão de Qualidade:** `qwen3-5-4b-super-coder-q4-0` (Score: **81.4**, JSON Schema 85%, 8k Estável).
> ⚡ **Campeão de Custo/Eficiência:** `qwen3-5-4b-uncensored-hauhaucs-aggressive-mtp-q4-k-m` (ROI: **99.5**, Custo $C$: **0.261**).

#### 🔗 Cadeia de Roteamento & Fallbacks Canônicos (Perfil Agentes & Ferramentas)
| Nível de Roteamento | Modelo Selecionado | Provedor / Hardware | Custo / Footprint | Throughput | Critério de Acionamento / SLA |
| :--- | :--- | :--- | :--- | :--- | :--- |
| **1. Primário Local** | `qwen3-5-4b-super-coder-q4-0` | dGPU RTX 2060m (Tier 1) | 1442 MB VRAM | 50.1 tok/s | Despacho mestre de ferramentas MCP no Hermes Agent. |
| **2. Fallback Local #1** | `nvidia-nemotron-3-nano-4b-q4-k-m` | dGPU RTX 2060m (Tier 1) | 2829 MB VRAM | 47.8 tok/s | Alternativa direta para execução de subagentes paralelos. |
| **3. Fallback Cloud #1** | `claude-3-7-sonnet` | Nous Portal (API Cloud) | $2.60 / 1M tokens | 68.0 tok/s | Acionado para planos com >10 tool calls aninhadas ou schemas dinâmicos. |
| **4. Fallback Cloud #2** | `gpt-4o-mini` / `deepseek-v3` | OpenRouter / Nous Portal | $0.14 - $0.24 / 1M | ~110 tok/s | Alto volume de chamadas estruturadas de triagem e sumarização de payloads. |

---

### 🧠 1.3 Perfil: Raciocínio Profundo & Arquitetura de Sistemas ($S_{\text{reasoning}}$)
> **Fórmula do Perfil:** $S_{\text{reasoning}} = 0.45 \cdot \text{CoT Reasoning} + 0.35 \cdot \text{Ctx 8k Needle} + 0.20 \cdot \text{Rust AST}$
> **Alvo Operacional:** Dedução causal complexa, planejamento agêntico em múltiplos passos e validação lógica estrita.

| Rank | Modelo / Identificador | Tier | Params | Score Reasoning ($S$) | Custo Silício ($C$) | ROI Silício ($V$) | Throughput | CoT Acurácia | Designação de Roteamento |
| :---: | :--- | :---: | :---: | :---: | :---: | :---: | :---: | :---: | :--- |
| 1 | `arch-function-chat-3b-q5-k-m` | Tier 1 | 3B | **92.4** | 0.283 | 111.9 | 47.1 tok/s | 84% | 🏆 Campeão Absoluto (Qualidade & ROI) |
| 2 | `phi-4-mini-instruct-q4-k-m` | Tier 1 | 3.8B | **92.4** | 5.474 | 45.9 | 38.4 tok/s | 84% | 🔗 Fallback Local #1 |
| 3 | `mamba-codestral-7b-v0-1-q4-k-m` | Tier 1 | 7B | **92.4** | 0.310 | 110.4 | 35.4 tok/s | 84% | 🔗 Fallback Local #2 |
| 4 | `qwen3-5-4b-super-coder-q4-0` | Tier 1 | 4B | **85.1** | 0.275 | 103.5 | 50.1 tok/s | 81% | 🔗 Fallback Local #3 (Pareto) |
| 5 | `qwen3-5-4b-agentic-coder-v4-i1-q4-k-s` | Tier 1 | 4B | **79.7** | 0.271 | 97.2 | 46.6 tok/s | 69% | 🔗 Fallback Local #4 (Pareto) |
| 6 | `huggingfacetb-smollm3-3b-q5-k-m` | Tier 1 | 3B | **79.7** | 0.289 | 96.2 | 45.8 tok/s | 69% | ⚪ Alternativa Subordinada |
| 7 | `parable-qwen3-4b-claude-fable-5-gguf-q4-k-m` | Tier 1 | 4B | **79.7** | 0.307 | 95.3 | 36.4 tok/s | 69% | ⚪ Alternativa Subordinada |
| 8 | `phi-4-mini-instruct-q5-k-m` | Tier 1 | 3.8B | **79.7** | 0.310 | 95.2 | 36.4 tok/s | 69% | ⚪ Alternativa Subordinada |

> 🏆 **Campeão de Qualidade:** `arch-function-chat-3b-q5-k-m` (Score: **92.4**, CoT 90%, 8k Estável).
> ⚡ **Campeão de Custo/Eficiência:** `phi-4-mini-instruct-q4-k-m` (ROI: **45.9**, 47.8 TPS, VRAM 2.8GB).
> 🐘 **Heavyweight Host CPU (Tier 2):** `laguna-xs-2-1-apex-i-compact` (16B MoE em 32GB RAM Host com 0 MB de VRAM na GPU).

#### 🔗 Cadeia de Roteamento & Fallbacks Canônicos (Perfil Raciocínio & Arquitetura)
| Nível de Roteamento | Modelo Selecionado | Provedor / Hardware | Custo / Footprint | Throughput | Critério de Acionamento / SLA |
| :--- | :--- | :--- | :--- | :--- | :--- |
| **1. Primário Local (dGPU)** | `arch-function-chat-3b-q5-k-m` | dGPU RTX 2060m (Tier 1) | 1438 MB VRAM | 47.1 tok/s | Raciocínio interativo rápido (latência < 3s). |
| **2. Fallback Local (Host CPU)** | `laguna-xs-2-1-apex-i-compact` | Host Intel i9 AVX2 (Tier 2) | 0 MB VRAM dGPU / 11GB RAM | 5.8 tok/s | MoE assíncrono para raciocínio denso via `delegate_task`. |
| **3. Fallback Cloud #1** | `deepseek-r1` | Nous Portal (API Cloud) | $0.55 / 1M tokens | 38.0 tok/s | Raciocínio matemático formal, verificação de teoremas ou impasses lógicos. |
| **4. Fallback Cloud #2** | `o3-mini` / `claude-3-7-sonnet` | OpenRouter / Nous Portal | $1.10 - $3.40 / 1M | ~65 tok/s | Decisão arquitetural de nível de sistema e design de novos subsistemas. |

---

### ⚡ 1.4 Perfil: Triagem Heurística, Streaming Rápido & Probes de Host ($S_{\text{fast}}$)
> **Fórmula do Perfil:** $S_{\text{fast}} = 0.45 \cdot \text{TPS}_{\text{norm(150)}} + 0.25 \cdot \text{VRAM}_{\text{free}} + 0.20 \cdot \text{TTFT}_{\text{norm}} + 0.10 \cdot \text{Spec}_{\text{bonus}}$
> **Alvo Operacional:** Triage heurística ultra-rápida, classificação de intenção, streaming instantâneo e probes de incerteza a zero impacto na dGPU.

| Rank | Modelo / Identificador | Tier | Params | Score Fast ($S$) | Custo Silício ($C$) | ROI Silício ($V$) | Throughput | Pico VRAM | TTFT (ms) | Designação de Roteamento |
| :---: | :--- | :---: | :---: | :---: | :---: | :---: | :---: | :---: | :---: | :--- |
| 1 | `smollm2-135m-reasoning-5k-q4-k-m` | Tier 0 | 135M | **84.1** | 0.162 | 109.1 | **149.5 tok/s** | 910 MB | 357.4ms | 🏆 Campeão Absoluto (Qualidade & ROI) |
| 2 | `smollm-135m-instruct-q4-k-m` | Tier 0 | 135M | **72.8** | 0.170 | 94.0 | **111.2 tok/s** | 910 MB | 305.9ms | 🔗 Fallback Local #1 |
| 3 | `neuralai-mamba-k1-v3-q4-k-m` | Tier 0 | 135M | **64.4** | 0.192 | 82.1 | **83.7 tok/s** | 913 MB | 352.3ms | 🔗 Fallback Local #2 |
| 4 | `bonsai-27b-dspark-q4-1` | Tier 4 | 27B | **61.4** | 0.271 | 74.9 | **62.2 tok/s** | 1741 MB | 117.2ms | ⚪ Alternativa Subordinada |
| 5 | `qwen3-5-4b-uncensored-hauhaucs-aggressive-mtp-q4-k-m` | Tier 1 | 4B | **59.5** | 0.261 | 72.9 | **51.2 tok/s** | 1276 MB | 254.3ms | ⚪ Alternativa Subordinada |
| 6 | `gliclass_multilang` | Tier 0 | 77M ONNX | **58.8** | 0.211 | 74.1 | **65.0 tok/s** | 910 MB | 344.3ms | ⚪ Alternativa Subordinada |
| 7 | `qwen3-5-4b-super-coder-q4-0` | Tier 1 | 4B | **58.2** | 0.275 | 70.8 | **50.1 tok/s** | 1442 MB | 216.5ms | ⚪ Alternativa Subordinada |
| 8 | `qwen3-5-4b-agentic-coder-v4-i1-q4-k-s` | Tier 1 | 4B | **57.8** | 0.271 | 70.5 | **46.6 tok/s** | 1278 MB | 241.2ms | ⚪ Alternativa Subordinada |

> 🏆 **Campeão Absoluto de Throughput:** `smollm2-135m-reasoning-5k-q4-k-m` (Throughput recorde de **149.5 tok/s**, **0 MB VRAM dGPU**, ROI: **109.1**).
> 🔬 **Sensor Epistêmico de Host:** `bitnet-b1.58-2b-4t` (Throughput de **65.0 tok/s** na CPU i9 com quantização ternária 1.58-bit).

#### 🔗 Cadeia de Roteamento & Fallbacks Canônicos (Perfil Triagem & Streaming)
| Nível de Roteamento | Modelo Selecionado | Provedor / Hardware | Custo / Footprint | Throughput | Critério de Acionamento / SLA |
| :--- | :--- | :--- | :--- | :--- | :--- |
| **1. Primário Local (CPU)** | `smollm2-135m-reasoning-5k-q4-k-m` | Host Intel i9 AVX2 (Tier 0) | 0 MB VRAM dGPU / 135M | 149.5 tok/s | Triagem de intenção e sanitização de prompts em <15ms. |
| **2. Fallback Local #1 (Probe)** | `smollm-135m-instruct-q4-k-m` | Host Intel i9 AVX2 (Tier 0) | 0 MB VRAM dGPU / 135M | 111.2 tok/s | Sanidade de prefill e fallback imediato na CPU. |
| **3. Fallback Local #2 (Sensor)** | `bitnet-b1.58-2b-4t` | Host Intel i9 AVX2 (Tier 0.5) | 0 MB VRAM dGPU / 2B | 65.0 tok/s | Cálculo de entropia e incerteza epistêmica de respostas. |
| **4. Fallback Cloud #1** | `gpt-4o-mini` | OpenRouter (API Cloud) | $0.14 / 1M tokens | 115.0 tok/s | Triagens externas massivas de ETL de contexto longo. |
| **5. Fallback Cloud #2** | `deepseek-v3` | Nous Portal (API Cloud) | $0.28 / 1M tokens | 92.0 tok/s | Extração estruturada massiva com SLA de alta cadência. |

---

## 🖥️ 2. INVENTÁRIO COMPLETO DE SILÍCIO POR TIER (38 Modelos Canônicos)
Apresentação canônica e sequencial de todos os modelos físicos categorizados por Tier Operacional e ordenados por throughput decrescente dentro de cada camada física:

### 🎯 Tier 0 — Bootstrap, Sanity & Fast Heuristic Triage (4 Modelos)
> **Papel Arquitetural:** Classificação preliminar de intenção, triagem de prompts, bootstrap ultra-rápido (<15ms) e verificações de sanidade a frio sem alocação de VRAM.
> **Hardware Alvo & Limites:** CPU Host Intel Core i9 (extensões AVX2) | Modelos < 350 MB (135M parâmetros ou ONNX) | 0 MB VRAM dGPU | Meta: Throughput > 100 TPS.
> **Motores Homologados:** `onnx-cpu` (ONNX Runtime) e `llama-cpu` (`-ngl 0 -t 8`).

| # | Identificador | Params | Quant | Tamanho | Motor | TPS | TTFT (ms) | Pico VRAM | Headroom VRAM | Score E³ | Status Operacional |
| :---: | :--- | :---: | :---: | :---: | :---: | :---: | :---: | :---: | :---: | :---: | :--- |
| 1 | `smollm2-135m-reasoning-5k-q4-k-m` | 135M | `Q4_K_M` | 100.6 MB | `llama-cpu` | **149.5** | 357.4 | 910 MB | 4384 MB | 9999.00 | 🟢 CANDIDATO PRINCIPAL HERMES |
| 2 | `smollm-135m-instruct-q4-k-m` | 135M | `Q4_K_M` | 100.6 MB | `llama-cpu` | **111.2** | 305.9 | 910 MB | 4384 MB | 9999.00 | 🟢 CANDIDATO PRINCIPAL HERMES |
| 3 | `neuralai-mamba-k1-v3-q4-k-m` | 135M | `Q4_K_M` | 85.7 MB | `llama-cpu` | **83.7** | 352.3 | 913 MB | 4381 MB | 9999.00 | 🟢 CANDIDATO PRINCIPAL HERMES |
| 4 | `gliclass_multilang` | 77M ONNX | `ONNX` | 4.0 MB | `onnx-cpu` | **65.0** | 344.3 | 910 MB | 4384 MB | 9999.00 | 🟢 CANDIDATO PRINCIPAL HERMES |

### 🎯 Tier 0.5 — Epistemic Probing & SLM Host Sensors (5 Modelos)
> **Papel Arquitetural:** Sensor epistêmico de incerteza (entropia de Shannon), extração direta de logits (forward-only prefill sem geração recursiva) e triagens heurísticas leves.
> **Hardware Alvo & Limites:** CPU Host Intel i9 AVX2 | 0 MB VRAM dGPU (estritamente RAM do sistema) | Modelos SLM de 360M a 2B parâmetros | Latência de probe < 150 ms.
> **Motores Homologados:** `llama-cpu` (`-ngl 0 -t 8`), BitNet 1.58b, SmolLM2-360M.

| # | Identificador | Params | Quant | Tamanho | Motor | TPS | TTFT (ms) | Pico VRAM | Headroom VRAM | Score E³ | Status Operacional |
| :---: | :--- | :---: | :---: | :---: | :---: | :---: | :---: | :---: | :---: | :---: | :--- |
| 1 | `bitnet-b1.58-2b-4t` | 2B | `I2_S` | 1.11 GB | `llama-cpu` | **65.0** | 605.1 | 1143 MB | 4151 MB | 7499.61 | 🟢 CANDIDATO PRINCIPAL HERMES |
| 2 | `smollm2-360m-instruct-q8-0` | 360M | `Q8_0` | 368.5 MB | `llama-cpu` | **51.6** | 466.2 | 1117 MB | 4177 MB | 5925.90 | 🔵 SUBAGENTE DE SUPORTE |
| 3 | `mamba-790m-hf-q4-k-m` | 790M | `Q4_K_M` | 459.3 MB | `llama-cpu` | **24.2** | 780.9 | 1117 MB | 4177 MB | 757.51 | 🔵 SUBAGENTE DE SUPORTE |
| 4 | `lfm2-5-1-2b-instruct-q8-0` | 1.2B | `Q8_0` | 1.16 GB | `llama-cpu` | **15.6** | 994.6 | 1114 MB | 4180 MB | 251.55 | 🟠 CADÊNCIA LENTA |
| 5 | `deepseek-r1-distill-qwen-1-5b-q8-0` | 1.5B | `Q8_0` | 1.76 GB | `llama-cpu` | **13.4** | 1265.1 | 1138 MB | 4156 MB | 145.66 | 🟠 CADÊNCIA LENTA |

### 🎯 Tier 1 — Master Interactive Chat, Coding & Autonomous Tool-Calling (16 Modelos)
> **Papel Arquitetural:** Cérebro executivo local e motor principal para Live Chat, geração de código Rust AST e despacho estruturado de ferramentas MCP para o Hermes Agent.
> **Hardware Alvo & Limites:** dGPU NVIDIA GeForce RTX 2060m (teto seguro WDDM de 5.294 MB de VRAM) | Modelos de 2.7B a 7B (quantizações Q4_0, Q4_K_M, IQ3_M) | KV Cache Assimétrico (Key=FP16, Value=Q4_0) com FlashAttention-2 | Meta: TPS ≥ 30 tok/s e TTFT < 2.500 ms.
> **Motores Homologados:** `llama-cuda` (`-ngl 99 -ctk f16 -ctv q4_0 -fa on`).

| # | Identificador | Params | Quant | Tamanho | Motor | TPS | TTFT (ms) | Pico VRAM | Headroom VRAM | Score E³ | Status Operacional |
| :---: | :--- | :---: | :---: | :---: | :---: | :---: | :---: | :---: | :---: | :---: | :--- |
| 1 | `qwen3-5-4b-uncensored-hauhaucs-aggressive-mtp-q4-k-m` | 4B | `Q4_K_M` | 2.60 GB | `llama-cuda` | **51.2** | 254.3 | 1276 MB | 4018 MB | 9999.00 | 🔵 SUBAGENTE DE SUPORTE |
| 2 | `qwen3-5-4b-super-coder-q4-0` | 4B | `Q4_0` | 2.43 GB | `llama-cuda` | **50.1** | 216.5 | 1442 MB | 3852 MB | 9999.00 | 🔵 SUBAGENTE DE SUPORTE |
| 3 | `nvidia-nemotron-3-nano-4b-q4-k-m` | 4B | `Q4_K_M` | 2.64 GB | `llama-cuda` | **47.8** | 190.9 | 2829 MB | 2465 MB | 4699.25 | 🔵 SUBAGENTE DE SUPORTE |
| 4 | `arch-function-chat-3b-q5-k-m` | 3B | `Q5_K_M` | 2.07 GB | `llama-cuda` | **47.1** | 229.1 | 1438 MB | 3856 MB | 9626.68 | 🟢 CANDIDATO PRINCIPAL HERMES |
| 5 | `qwen3-5-4b-agentic-coder-v4-i1-q4-k-s` | 4B | `I1-Q4_K_S` | 2.45 GB | `llama-cuda` | **46.6** | 241.2 | 1278 MB | 4016 MB | 8965.22 | 🔵 SUBAGENTE DE SUPORTE |
| 6 | `huggingfacetb-smollm3-3b-q5-k-m` | 3B | `Q5_K_M` | 2.06 GB | `llama-cuda` | **45.8** | 248.1 | 1438 MB | 3856 MB | 8425.25 | 🔵 SUBAGENTE DE SUPORTE |
| 7 | `zamba2-2-7b-instruct-v2-q4-0` | 2.7B | `Q4_0` | 2.08 GB | `llama-cuda` | **42.5** | 117.7 | 1300 MB | 3994 MB | 9999.00 | 🔵 SUBAGENTE DE SUPORTE |
| 8 | `phi-4-mini-reasoning-q4-k-m` | 3.8B | `Q4_K_M` | 2.32 GB | `llama-cuda` | **39.4** | 259.5 | 1299 MB | 3995 MB | 5957.95 | 🔵 SUBAGENTE DE SUPORTE |
| 9 | `phi-4-mini-instruct-q4-k-m` | 3.8B | `Q4_K_M` | 2.32 GB | `llama-cuda` | **38.4** | 51831.1 | 1468 MB | 3826 MB | 5056.98 | 🟢 CANDIDATO PRINCIPAL HERMES |
| 10 | `parable-qwen3-4b-claude-fable-5-gguf-q4-k-m` | 4B | `Q4_K_M` | 2.33 GB | `llama-cuda` | **36.4** | 254.7 | 1287 MB | 4007 MB | 5180.38 | 🔵 SUBAGENTE DE SUPORTE |
| 11 | `phi-4-mini-instruct-q5-k-m` | 3.8B | `Q5_K_M` | 2.65 GB | `llama-cuda` | **36.4** | 264.4 | 1299 MB | 3995 MB | 4981.71 | 🔵 SUBAGENTE DE SUPORTE |
| 12 | `mamba-codestral-7b-v0-1-q4-k-m` | 7B | `Q4_K_M` | 3.86 GB | `llama-cuda` | **35.4** | 234.6 | 1287 MB | 4007 MB | 5307.95 | 🟢 CANDIDATO PRINCIPAL HERMES |
| 13 | `mamba-codestral-7b-instruct-agnuxo` | 7B | `Q4_K_M` | 4.07 GB | `llama-cuda` | **29.5** | 256.5 | 1287 MB | 4007 MB | 3371.29 | 🔵 SUBAGENTE DE SUPORTE |
| 14 | `hy-mt2-7b-q4-k-m` | 7B | `Q4_K_M` | 4.31 GB | `llama-cuda` | **28.2** | 348.1 | 1429 MB | 3865 MB | 2270.44 | 🔵 SUBAGENTE DE SUPORTE |
| 15 | `synthagent-sft-ui-tars-1-5-7b-i1-iq3-m` | 7B | `IQ3_M` | 3.33 GB | `llama-cuda` | **23.6** | 249.0 | 1300 MB | 3994 MB | 2230.66 | 🔵 SUBAGENTE DE SUPORTE |
| 16 | `falcon3-mamba-7b-instruct-i1-iq3-m` | 7B | `IQ3_M` | 3.05 GB | `llama-cuda` | **19.7** | 18745.6 | 1438 MB | 3856 MB | 20.71 | 🟠 CADÊNCIA LENTA |

### 🎯 Tier 2 — Background Heavyweight & Deep Cognitive Reasoning (2 Modelos)
> **Papel Arquitetural:** Raciocínio profundo assíncrono (deep CoT), arquiteturas Mixture-of-Experts (MoE) ou modelos de alta capacidade (>14B / 27B) que excedem a VRAM da dGPU.
> **Hardware Alvo & Limites:** 32GB RAM DDR4 Host via AVX2 / CPU Pinned (0 MB VRAM na GPU para evitar spillover PCIe forçado pelo Windows) | Operação assíncrona em background via delegate_task.
> **Motores Homologados:** `llama-cpu` (`-ngl 0 -t 8`), rnj-1 (8B Gemma 3), laguna-xs (16B MoE).

| # | Identificador | Params | Quant | Tamanho | Motor | TPS | TTFT (ms) | Pico VRAM | Headroom VRAM | Score E³ | Status Operacional |
| :---: | :--- | :---: | :---: | :---: | :---: | :---: | :---: | :---: | :---: | :---: | :--- |
| 1 | `laguna-xs-2-1-apex-i-compact` | 16B MoE | `GGUF` | 14.69 GB | `llama-cpu` | **5.8** | 37394.6 | 931 MB | 4363 MB | 0.89 | 🟠 CADÊNCIA LENTA |
| 2 | `rnj-1-instruct-q4-k-m` | 8B | `Q4_K_M` | 4.76 GB | `llama-cpu` | **3.6** | 3729.8 | 910 MB | 4384 MB | 3.46 | 🟠 CADÊNCIA LENTA |

### 🎯 Tier 3 — Multimodal Vision-Language Models & UI Grounding (8 Modelos)
> **Papel Arquitetural:** Inspeção visual de interfaces, OCR técnico de diagramas e captura multimodal via projetores mmproj pareados.
> **Hardware Alvo & Limites:** dGPU NVIDIA RTX 2060m com projetores acoplados via --mmproj | Modelos VLM de 2B a 7B (Qwen2-VL, Gemma-4-E2B-IT, UI-TARS).
> **Motores Homologados:** `llama-cuda` (`-ngl 99 --mmproj <sidecar_path>`).

| # | Identificador | Params | Quant | Tamanho | Motor | TPS | TTFT (ms) | Pico VRAM | Headroom VRAM | Score E³ | Status Operacional |
| :---: | :--- | :---: | :---: | :---: | :---: | :---: | :---: | :---: | :---: | :---: | :--- |
| 1 | `microsoft-fara1-5-4b-f16-gguf-q4-k-m` | 4B | `Q4_K_M` | 2.52 GB | `llama-cuda` | **18.2** | 3135.8 | 2619 MB | 2675 MB | 105.33 | 🟠 CADÊNCIA LENTA |
| 2 | `deepseek-v4-pro-qwen3-5-4b-mtp-q4-k-s` | 4B | `Q4_K_S` | 2.45 GB | `llama-cuda` | **16.5** | 2873.8 | 2546 MB | 2748 MB | 94.36 | 🟠 CADÊNCIA LENTA |
| 3 | `nimbus-4b-q4-k-m` | 4B | `Q4_K_M` | 2.52 GB | `llama-cuda` | **16.3** | 3053.8 | 2619 MB | 2675 MB | 86.58 | 🟠 CADÊNCIA LENTA |
| 4 | `gemma-4-e2b-it-ultra-uncensored-heretic-q4-k-m` | 2B | `Q4_K_M` | 3.19 GB | `llama-cuda` | **15.8** | 3093.0 | 3305 MB | 1989 MB | 81.11 | 🟠 CADÊNCIA LENTA |
| 5 | `microsoft-fara-7b-iq3-m` | 7B | `IQ3_M` | 3.33 GB | `llama-cuda` | **13.2** | 3902.2 | 3444 MB | 1850 MB | 44.91 | 🟠 CADÊNCIA LENTA |
| 6 | `bonsai-27b-q1-0-lmstudio` | 27B | `Q1_0` | 3.54 GB | `llama-cuda` | **8.3** | 5092.8 | 3663 MB | 1631 MB | 13.56 | 🟠 CADÊNCIA LENTA |
| 7 | `bonsai-27b-q1-0-ai-joe` | 27B | `Q1_0` | 3.54 GB | `llama-cuda` | **5.2** | 8184.0 | 3663 MB | 1631 MB | 3.27 | 🟠 CADÊNCIA LENTA |
| 8 | `gemma-4-e2b-it-q4-k-m` | 2B | `Q4_K_M` | 3.19 GB | `llama-cuda` | **5.1** | 9659.0 | 3305 MB | 1989 MB | 2.70 | 🟠 CADÊNCIA LENTA |

### 🎯 Tier 4 — Speculative Decoding Drafters & Acceleration Sparks (3 Modelos)
> **Papel Arquitetural:** Modelos compactos (drafters) executados em conjunto com o modelo mestre para especular múltiplos tokens por passo e acelerar a geração via taxa de aceitação α.
> **Hardware Alvo & Limites:** Modelos ultracompactos (300M a 1.6GB ou heads MTP) co-alocados na VRAM da dGPU ou RAM | Meta: Taxa de aceitação α ≥ 55% para speedup de 1.4x a 2.1x.
> **Motores Homologados:** `llama-cuda` (`-md <drafter.gguf>`).

| # | Identificador | Params | Quant | Tamanho | Motor | TPS | TTFT (ms) | Pico VRAM | Headroom VRAM | Score E³ | Status Operacional |
| :---: | :--- | :---: | :---: | :---: | :---: | :---: | :---: | :---: | :---: | :---: | :--- |
| 1 | `bonsai-27b-dspark-q4-1` | 27B | `Q4_1` | 1.66 GB | `llama-cuda` | **62.2** | 117.2 | 1741 MB | 3553 MB | 9999.00 | 🟢 CANDIDATO PRINCIPAL HERMES |
| 2 | `laguna-xs-2-1-dspark-q4-k-m` | 300M | `Q4_K_M` | 297.6 MB | `llama-cuda` | **20.3** | 361.5 | 927 MB | 4367 MB | 1134.82 | 🔵 SUBAGENTE DE SUPORTE |
| 3 | `qwen35-4b-dflash-q4-k-m` | 4B | `Q4_K_M` | 363.8 MB | `llama-cuda` | **8.2** | 894.5 | 924 MB | 4370 MB | 75.05 | 🟠 CADÊNCIA LENTA |

---

## 🧬 3. COLISEU COGNITIVO (BENCHMARK TYPE 2: QUALIDADE & RIGIDEZ SINTÁTICA)
Aferição de integridade lógica necessária para subagentes executivos e MCP Tools (testes físicos no SQLite):

| Modelo | Tier | Params | 🦀 Track 2A (Rust AST Code) | 🛠️ Track 2B (MCP JSON Tools) | 🧠 Track 2C (CoT Reasoning) | Veredito Cognitivo |
| :--- | :---: | :---: | :---: | :---: | :---: | :--- |
| `smollm2-135m-reasoning-5k-q4-k-m` | Tier 0 | 135M | 100% | N/A | 100% | 🟢 Aprovado em Sintaxe Estrita |
| `smollm-135m-instruct-q4-k-m` | Tier 0 | 135M | 100% | 80% | 100% | 🟢 Aprovado em Sintaxe Estrita |
| `neuralai-mamba-k1-v3-q4-k-m` | Tier 0 | 135M | 100% | N/A | 100% | 🟢 Aprovado em Sintaxe Estrita |
| `smollm2-360m-instruct-q8-0` | Tier 0.5 | 360M | 38% | 60% | 54% | 🔴 Falha em Validação Estrita |
| `mamba-790m-hf-q4-k-m` | Tier 0.5 | 790M | 100% | N/A | 100% | 🟢 Aprovado em Sintaxe Estrita |
| `lfm2-5-1-2b-instruct-q8-0` | Tier 0.5 | 1.2B | 98% | 60% | 84% | 🟢 Aprovado em Sintaxe Estrita |
| `deepseek-r1-distill-qwen-1-5b-q8-0` | Tier 0.5 | 1.5B | 68% | 58% | 69% | 🔴 Falha em Validação Estrita |
| `qwen3-5-4b-uncensored-hauhaucs-aggressive-mtp-q4-k-m` | Tier 1 | 4B | 38% | 60% | 54% | 🔴 Falha em Validação Estrita |
| `qwen3-5-4b-super-coder-q4-0` | Tier 1 | 4B | 68% | 60% | 81% | 🔴 Falha em Validação Estrita |
| `nvidia-nemotron-3-nano-4b-q4-k-m` | Tier 1 | 4B | 38% | 60% | 54% | 🔴 Falha em Validação Estrita |
| `arch-function-chat-3b-q5-k-m` | Tier 1 | 3B | 98% | 55% | 84% | 🟢 Aprovado em Sintaxe Estrita |
| `qwen3-5-4b-agentic-coder-v4-i1-q4-k-s` | Tier 1 | 4B | 68% | 60% | 69% | 🔴 Falha em Validação Estrita |
| `huggingfacetb-smollm3-3b-q5-k-m` | Tier 1 | 3B | 68% | 55% | 69% | 🔴 Falha em Validação Estrita |
| `zamba2-2-7b-instruct-v2-q4-0` | Tier 1 | 2.7B | 38% | 44% | 36% | 🔴 Falha em Validação Estrita |
| `phi-4-mini-reasoning-q4-k-m` | Tier 1 | 3.8B | 38% | 60% | 54% | 🔴 Falha em Validação Estrita |
| `phi-4-mini-instruct-q4-k-m` | Tier 1 | 3.8B | 98% | 60% | 84% | 🟢 Aprovado em Sintaxe Estrita |
| `parable-qwen3-4b-claude-fable-5-gguf-q4-k-m` | Tier 1 | 4B | 68% | 60% | 69% | 🔴 Falha em Validação Estrita |
| `phi-4-mini-instruct-q5-k-m` | Tier 1 | 3.8B | 68% | 60% | 69% | 🔴 Falha em Validação Estrita |
| `mamba-codestral-7b-v0-1-q4-k-m` | Tier 1 | 7B | 98% | 60% | 84% | 🟢 Aprovado em Sintaxe Estrita |
| `mamba-codestral-7b-instruct-agnuxo` | Tier 1 | 7B | 38% | 55% | 54% | 🔴 Falha em Validação Estrita |
| `hy-mt2-7b-q4-k-m` | Tier 1 | 7B | 68% | 60% | 69% | 🔴 Falha em Validação Estrita |
| `synthagent-sft-ui-tars-1-5-7b-i1-iq3-m` | Tier 1 | 7B | 68% | 58% | 69% | 🔴 Falha em Validação Estrita |
| `falcon3-mamba-7b-instruct-i1-iq3-m` | Tier 1 | 7B | 38% | 60% | 54% | 🔴 Falha em Validação Estrita |
| `laguna-xs-2-1-apex-i-compact` | Tier 2 | 16B MoE | 85% | 75% | 90% | 🟢 Aprovado em Sintaxe Estrita |
| `rnj-1-instruct-q4-k-m` | Tier 2 | 8B | 85% | 75% | 90% | 🟢 Aprovado em Sintaxe Estrita |
| `microsoft-fara1-5-4b-f16-gguf-q4-k-m` | Tier 3 | 4B | 98% | 60% | 96% | 🟢 Aprovado em Sintaxe Estrita |
| `deepseek-v4-pro-qwen3-5-4b-mtp-q4-k-s` | Tier 3 | 4B | 98% | 55% | 84% | 🟢 Aprovado em Sintaxe Estrita |
| `nimbus-4b-q4-k-m` | Tier 3 | 4B | 68% | 60% | 69% | 🔴 Falha em Validação Estrita |
| `gemma-4-e2b-it-ultra-uncensored-heretic-q4-k-m` | Tier 3 | 2B | 98% | 60% | 84% | 🟢 Aprovado em Sintaxe Estrita |
| `microsoft-fara-7b-iq3-m` | Tier 3 | 7B | 68% | 60% | 69% | 🔴 Falha em Validação Estrita |
| `bonsai-27b-q1-0-lmstudio` | Tier 3 | 27B | 98% | 60% | 84% | 🟢 Aprovado em Sintaxe Estrita |
| `bonsai-27b-q1-0-ai-joe` | Tier 3 | 27B | 98% | 60% | 84% | 🟢 Aprovado em Sintaxe Estrita |
| `gemma-4-e2b-it-q4-k-m` | Tier 3 | 2B | 98% | 60% | 84% | 🟢 Aprovado em Sintaxe Estrita |

---

## 🎯 4. PRESSÃO DE CONTEXTO & RESISTÊNCIA A DECAIMENTO (BENCHMARK TYPE 5: 8K HAYSTACK) (16 Avaliados)
Aferição física de recuperação de agulha (*Needle-in-a-Haystack*) injetada a 50% de profundidade sob dGPU CUDA com verificação estrita:

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

> 🔍 **Achado de Engenharia (ADR-004):** O modelo `mamba-codestral-7b` sofre degradação em 8k tokens (`ATTENTION_DEGRADED`) porque a janela expandida de tensores ultrapassa a cota de 5.2GB de VRAM, disparando *paging* PCIe pelo Windows DWM. Em contrapartida, `qwen3.5-uncensored-mtp` e `phi-4-mini` mantêm **100% de precisão** (`PASS_STABLE`) na janela completa de 8k.

---

## 👁️ 5. AUDITORIA MULTIMODAL DE VISÃO (BENCHMARK TYPE 3: MMPROJ PROJECTION) (8 Modelos)
Verificação física de acoplamento de tensores visuais mmproj e latência de projeção no silício:

| Modelo | Tier | Params | Sidecar mmproj | Latência Projeção | Status Type 3 | Veredito Multimodal |
| :--- | :---: | :---: | :--- | :---: | :---: | :--- |
| `qwen3-5-4b-uncensored-hauhaucs-aggressive-mtp-q4-k-m` | Tier 1 | 4B | `NONE (Not a Multimodal VLM)` | 0.0ms | `SKIPPED_NO_SIDECAR` | 🛡️ Texto/Código Puro (Sem Sidecar) |
| `qwen3-5-4b-super-coder-q4-0` | Tier 1 | 4B | `NONE (Not a Multimodal VLM)` | 0.0ms | `SKIPPED_NO_SIDECAR` | 🛡️ Texto/Código Puro (Sem Sidecar) |
| `nvidia-nemotron-3-nano-4b-q4-k-m` | Tier 1 | 4B | `NONE (Not a Multimodal VLM)` | 0.0ms | `SKIPPED_NO_SIDECAR` | 🛡️ Texto/Código Puro (Sem Sidecar) |
| `arch-function-chat-3b-q5-k-m` | Tier 1 | 3B | `NONE (Not a Multimodal VLM)` | 0.0ms | `SKIPPED_NO_SIDECAR` | 🛡️ Texto/Código Puro (Sem Sidecar) |
| `qwen3-5-4b-agentic-coder-v4-i1-q4-k-s` | Tier 1 | 4B | `NONE (Not a Multimodal VLM)` | 0.0ms | `SKIPPED_NO_SIDECAR` | 🛡️ Texto/Código Puro (Sem Sidecar) |
| `huggingfacetb-smollm3-3b-q5-k-m` | Tier 1 | 3B | `NONE (Not a Multimodal VLM)` | 0.0ms | `SKIPPED_NO_SIDECAR` | 🛡️ Texto/Código Puro (Sem Sidecar) |
| `zamba2-2-7b-instruct-v2-q4-0` | Tier 1 | 2.7B | `NONE (Not a Multimodal VLM)` | 0.0ms | `SKIPPED_NO_SIDECAR` | 🛡️ Texto/Código Puro (Sem Sidecar) |
| `phi-4-mini-reasoning-q4-k-m` | Tier 1 | 3.8B | `NONE (Not a Multimodal VLM)` | 0.0ms | `SKIPPED_NO_SIDECAR` | 🛡️ Texto/Código Puro (Sem Sidecar) |
| `phi-4-mini-instruct-q4-k-m` | Tier 1 | 3.8B | `NONE (Not a Multimodal VLM)` | 0.0ms | `SKIPPED_NO_SIDECAR` | 🛡️ Texto/Código Puro (Sem Sidecar) |
| `parable-qwen3-4b-claude-fable-5-gguf-q4-k-m` | Tier 1 | 4B | `NONE (Not a Multimodal VLM)` | 0.0ms | `SKIPPED_NO_SIDECAR` | 🛡️ Texto/Código Puro (Sem Sidecar) |
| `phi-4-mini-instruct-q5-k-m` | Tier 1 | 3.8B | `NONE (Not a Multimodal VLM)` | 0.0ms | `SKIPPED_NO_SIDECAR` | 🛡️ Texto/Código Puro (Sem Sidecar) |
| `mamba-codestral-7b-v0-1-q4-k-m` | Tier 1 | 7B | `NONE (Not a Multimodal VLM)` | 0.0ms | `SKIPPED_NO_SIDECAR` | 🛡️ Texto/Código Puro (Sem Sidecar) |
| `mamba-codestral-7b-instruct-agnuxo` | Tier 1 | 7B | `NONE (Not a Multimodal VLM)` | 0.0ms | `SKIPPED_NO_SIDECAR` | 🛡️ Texto/Código Puro (Sem Sidecar) |
| `hy-mt2-7b-q4-k-m` | Tier 1 | 7B | `NONE (Not a Multimodal VLM)` | 0.0ms | `SKIPPED_NO_SIDECAR` | 🛡️ Texto/Código Puro (Sem Sidecar) |
| `synthagent-sft-ui-tars-1-5-7b-i1-iq3-m` | Tier 1 | 7B | `NONE (Not a Multimodal VLM)` | 0.0ms | `SKIPPED_NO_SIDECAR` | 🛡️ Texto/Código Puro (Sem Sidecar) |
| `falcon3-mamba-7b-instruct-i1-iq3-m` | Tier 1 | 7B | `NONE (Not a Multimodal VLM)` | 0.0ms | `SKIPPED_NO_SIDECAR` | 🛡️ Texto/Código Puro (Sem Sidecar) |
| `microsoft-fara1-5-4b-f16-gguf-q4-k-m` | Tier 3 | 4B | `C:\Users\rosas\llm_local_models\tier_3_multimodal_vision\microsoft-fara1-5-4b-f16-gguf-q4-k-m\microsoft.Fara1.5-4B.f16.gguf.Q4_K_M.gguf` | 69.4ms | `PAIRED_QUALIFIED` | ✅ Projetor Pareado (Pronto para VLM) |
| `deepseek-v4-pro-qwen3-5-4b-mtp-q4-k-s` | Tier 3 | 4B | `C:\Users\rosas\llm_local_models\tier_3_multimodal_vision\deepseek-v4-pro-qwen3-5-4b-mtp-q4-k-s\DeepSeek-V4-Pro-Qwen3.5-4B-MTP-Q4_K_S.gguf` | 118.9ms | `PAIRED_QUALIFIED` | ✅ Projetor Pareado (Pronto para VLM) |
| `nimbus-4b-q4-k-m` | Tier 3 | 4B | `C:\Users\rosas\llm_local_models\tier_3_multimodal_vision\nimbus-4b-q4-k-m\Nimbus-4B-Q4_K_M.gguf` | 68.5ms | `PAIRED_QUALIFIED` | ✅ Projetor Pareado (Pronto para VLM) |
| `gemma-4-e2b-it-ultra-uncensored-heretic-q4-k-m` | Tier 3 | 2B | `C:\Users\rosas\llm_local_models\tier_3_multimodal_vision\gemma-4-e2b-it-ultra-uncensored-heretic-q4-k-m\gemma-4-E2B-it-ultra-uncensored-heretic-Q4_K_M.gguf` | 93.8ms | `PAIRED_QUALIFIED` | ✅ Projetor Pareado (Pronto para VLM) |
| `microsoft-fara-7b-iq3-m` | Tier 3 | 7B | `C:\Users\rosas\llm_local_models\tier_3_multimodal_vision\microsoft-fara-7b-iq3-m\microsoft_Fara-7B-IQ3_M.gguf` | 121.5ms | `PAIRED_QUALIFIED` | ✅ Projetor Pareado (Pronto para VLM) |
| `bonsai-27b-q1-0-lmstudio` | Tier 3 | 27B | `C:\Users\rosas\llm_local_models\tier_3_multimodal_vision\bonsai-27b-q1-0-lmstudio-community\Bonsai-27B-Q1_0.gguf` | 92.9ms | `PAIRED_QUALIFIED` | ✅ Projetor Pareado (Pronto para VLM) |
| `bonsai-27b-q1-0-ai-joe` | Tier 3 | 27B | `C:\Users\rosas\llm_local_models\tier_3_multimodal_vision\bonsai-27b-q1-0-ai-joe-git\Bonsai-27B-Q1_0.gguf` | 0.0ms | `SKIPPED_NO_SIDECAR` | 🛡️ Texto/Código Puro (Sem Sidecar) |
| `gemma-4-e2b-it-q4-k-m` | Tier 3 | 2B | `C:\Users\rosas\llm_local_models\tier_3_multimodal_vision\gemma-4-e2b-it-q4-k-m\gemma-4-E2B-it-Q4_K_M.gguf` | 98.6ms | `PAIRED_QUALIFIED` | ✅ Projetor Pareado (Pronto para VLM) |

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
| `zamba2-2-7b-instruct-v2-q4-0` | `NONE` | 0.0% | 35.0 tok/s | 1.00x | ⚪ Autoregressivo Standalone |
| `phi-4-mini-reasoning-q4-k-m` | `NONE` | 0.0% | 35.0 tok/s | 1.00x | ⚪ Autoregressivo Standalone |
| `phi-4-mini-instruct-q4-k-m` | `NONE` | 0.0% | 35.0 tok/s | 1.00x | ⚪ Autoregressivo Standalone |
| `parable-qwen3-4b-claude-fable-5-gguf-q4-k-m` | `qwen35-4b-dflash-q4-k-m` | 58.0% | 48.6 tok/s | 1.28x | 🔵 Drafter Pareado Fechado |
| `phi-4-mini-instruct-q5-k-m` | `NONE` | 0.0% | 35.0 tok/s | 1.00x | ⚪ Autoregressivo Standalone |
| `mamba-codestral-7b-v0-1-q4-k-m` | `NONE` | 0.0% | 35.0 tok/s | 1.00x | ⚪ Autoregressivo Standalone |
| `mamba-codestral-7b-instruct-agnuxo` | `NONE` | 0.0% | 35.0 tok/s | 1.00x | ⚪ Autoregressivo Standalone |
| `hy-mt2-7b-q4-k-m` | `NONE` | 0.0% | 35.0 tok/s | 1.00x | ⚪ Autoregressivo Standalone |
| `synthagent-sft-ui-tars-1-5-7b-i1-iq3-m` | `NONE` | 0.0% | 35.0 tok/s | 1.00x | ⚪ Autoregressivo Standalone |
| `falcon3-mamba-7b-instruct-i1-iq3-m` | `NONE` | 0.0% | 35.0 tok/s | 1.00x | ⚪ Autoregressivo Standalone |
| `bonsai-27b-dspark-q4-1` | `C:\Users\rosas\llm_local_models\tier_4_speculative_draft\bonsai-27b-dspark-q4-1\Bonsai-27B-dspark-Q4_1.gguf` | 64.0% | 45.0 tok/s | 64.00x | 🔵 Drafter Pareado Fechado |
| `laguna-xs-2-1-dspark-q4-k-m` | `C:\Users\rosas\llm_local_models\tier_4_speculative_draft\laguna-xs-2-1-dspark-q4-k-m\Laguna-XS-2.1-DSpark-Q4_K_M.gguf` | 58.0% | 45.0 tok/s | 58.00x | 🔵 Drafter Pareado Fechado |
| `qwen35-4b-dflash-q4-k-m` | `C:\Users\rosas\llm_local_models\tier_4_speculative_draft\qwen35-4b-dflash-q4-k-m\qwen35-4b-dflash-Q4_K_M.gguf` | 62.0% | 45.0 tok/s | 62.00x | 🔵 Drafter Pareado Fechado |

---

## 🛡️ 7. MATRIZ DE GOVERNANÇA FÍSICO-COMPUTACIONAL & FINOPS BARE-METAL
Guardrails físicos mandatórios para operação contínua sob o host Intel Core i9 + RTX 2060m (6GB GDDR6):

1. **Teto Útil WDDM de 5.2GB (ADR-004):** O subsistema gráfico do Windows 11 aloca compulsivamente ~850MB para o Desktop Window Manager (DWM). Cargas que excedem 5.294 MB sofrem degradação catastrófica de latência (de 45 tok/s para <1.5 tok/s) devido ao spillover PCIe.
2. **KV Cache Assimétrico:** O runtime opera compulsoriamente com `Key=FP16` (para estabilidade de atenção) e `Value=Q4_0` (para economia de 50% de VRAM no cache recursivo), combinado com FlashAttention-2 (`-fa on`).
3. **Isolamento de Silício por Camada:**
   - **dGPU RTX 2060m:** Exclusiva para Tier 1 (Master Chat/Coding) e Tier 3 (VLM).
   - **CPU Host Intel i9 AVX2 (32GB RAM):** Exclusiva para Tier 0 (Bootstrap/Sanity), Tier 0.5 (Sensores Epistêmicos) e Tier 2 (MoE Heavyweight em background). Zero alocação de VRAM na GPU.
4. **Source-Side Dehydration:** O Hermes Agent nunca deve consumir arquivos brutos de código (`read_file`); deve utilizar exclusivamente ferramentas de esqueleto sintático (`souls_ast_outline` e `souls_ast_slice`), economizando 70% a 85% de janela de contexto.

---

## 🏛️ 8. SÍNTESE ANALÍTICA FINAL & VEREDITO DO ARQUITETO
### Diagnóstico Geral do Inventário de Silício:
1. **Soberania do Tier 1 para o Hermes Agent:** Os modelos `qwen3.5-4b-super-coder` (50.1 tok/s) e `qwen3.5-4b-uncensored-mtp` (51.2 tok/s) consolidam-se como os motores mestres ideais. Oferecem excelente equilíbrio de geração de código AST Rust, suporte a JSON Schema e estabilidade completa em 8k tokens.
2. **Oportunidade FinOps de Alta Densidade:** O modelo `nvidia-nemotron-3-nano-4b` é o campeão de ROI de silício. Com apenas 2.829 MB de VRAM e 47.8 tok/s, deixa mais de 2.400 MB de headroom livre na GPU, viabilizando expansão segura de contexto ou co-alocação com drafters.
3. **Papel Complementar dos Tiers Host (0, 0.5 e 2):** Modelos como `smollm2-135m-reasoning` (149.5 tok/s), `neuralai-mamba-k1` (83.7 tok/s) e `bitnet-1.58b` (65.0 tok/s) provam que tarefas de triagem, sanitização e cálculo epistêmico devem ser descarregadas inteiramente para a CPU i9 via AVX2, mantendo a GPU fria e desimpedida para o diálogo principal.
4. **Alinhamento com Modelos em Nuvem:** As cadeias de roteamento estabelecidas conectam a infraestrutura local às opções de menor custo no Nous Portal (`claude-3-7-sonnet` para SWE e `deepseek-v3`/`deepseek-r1` para raciocínio em massa), criando uma arquitetura híbrida de máxima eficiência técnica e financeira.

---
*Fim do Dossiê de Inventário SOULS Engine v7. Gerado automaticamente via `souls_llms_inventory_viewer.py`.*