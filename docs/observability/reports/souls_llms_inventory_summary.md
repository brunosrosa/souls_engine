# 📊 SOULS SILICON OBSERVABILITY & HERMES LLM DOSSIER (v7 Canônica)
> **Gerado em:** 2026-10-11 03:45:38 | **Banco SSOT:** `Z:\souls_engine\.souls_data\db\souls_heuristic_vault.db`
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
| 1 | `nvidia-nemotron-3-nano-4b-q4-k-m` | 4B | `Q4_K_M` | **98.5** | 0.409 | 112.0 | 47.8 tok/s | 8k | 🏆 Campeão de Qualidade |
| 2 | `qwen3-5-4b-agentic-coder-v4-i1-q4-k-s` | 4B | `I1-Q4_K_S` | **97.9** | 0.271 | 119.4 | 46.6 tok/s | 8k | ⚡ Campeão de Custo/Eficiência |
| 3 | `qwen3-5-4b-uncensored-hauhaucs-aggressive-mtp-q4-k-m` | 4B | `Q4_K_M` | **96.4** | 0.261 | 118.2 | 51.2 tok/s | 8k | 🔗 Fallback Local #1 (Pareto) |
| 4 | `hy-mt2-7b-q4-k-m` | 7B | `Q4_K_M` | **90.2** | 0.375 | 104.3 | 28.2 tok/s | 8k | 🔗 Fallback Local #2 |
| 5 | `arch-function-chat-3b-q5-k-m` | 3B | `Q5_K_M` | **89.4** | 0.283 | 108.3 | 47.1 tok/s | 8k | ⚪ Alternativa Subordinada |
| 6 | `phi-4-mini-instruct-q4-k-m` | 3.8B | `Q4_K_M` | **78.8** | 5.474 | 39.2 | 38.4 tok/s | 8k | ⚪ Alternativa Subordinada |
| 7 | `hermes-3-llama-3.2-3b-q4-k-m` | 3B | `Q4_K_M` | **78.0** | 0.276 | 94.8 | 51.5 tok/s | 4k | ⚪ Alternativa Subordinada |
| 8 | `xlam-2-1b-fc-r-q4-k-m` | 1B | `Q4_K_M` | **67.2** | 0.221 | 84.2 | 80.1 tok/s | 0k | 🔗 Fallback Local #3 (Pareto) |

> 🏆 **Campeão de Qualidade:** `nvidia-nemotron-3-nano-4b-q4-k-m` (Score: **98.5**, 47.8 TPS, 8k Estável).
> ⚡ **Campeão de Custo/Eficiência:** `qwen3-5-4b-agentic-coder-v4-i1-q4-k-s` (ROI: **119.4**, VRAM reduzida a **1278 MB**, Custo $C$: **0.271**).

#### 🔗 Cadeia de Roteamento & Fallbacks Canônicos (Perfil Coding)
| Nível de Roteamento | Modelo Selecionado | Provedor / Hardware | Custo / Footprint | Throughput | Critério de Acionamento / SLA |
| :--- | :--- | :--- | :--- | :--- | :--- |
| **1. Primário Local** | `nvidia-nemotron-3-nano-4b-q4-k-m` | dGPU RTX 2060m (Tier 1) | 2829 MB VRAM | 47.8 tok/s | Padrão para tarefas de codificação e geração de AST Rust. |
| **2. Fallback Local #1** | `qwen3-5-4b-uncensored-hauhaucs-aggressive-mtp-q4-k-m` | dGPU RTX 2060m (Tier 1) | 1276 MB VRAM | 51.2 tok/s | Acionado se o primário estiver em concorrência ou em caso de crash. |
| **3. Fallback Local #2 (FinOps)** | `qwen3-5-4b-agentic-coder-v4-i1-q4-k-s` | dGPU RTX 2060m (Tier 1) | 1278 MB VRAM | 46.6 tok/s | Acionado quando houver pressão de memória de contexto (headroom > 2.4GB). |
| **4. Fallback Cloud #1** | `claude-3-7-sonnet` | Nous Portal (API Cloud) | $3.40 / 1M tokens | 68.0 tok/s | Acionado se refatoração envolver >3 crates ou compilação falhar 2x seguidas. |
| **5. Fallback Cloud #2** | `deepseek-v3` / `qwen-2.5-coder-32b` | Nous Portal / OpenRouter | $0.18 - $0.28 / 1M | ~100 tok/s | Acionado para geração em lote de boilerplate massivo de baixo custo. |

---

### 🛠️ 1.2 Perfil: Agentes Autônomos & Chamada de Ferramentas MCP ($S_{\text{agentic}}$)
> **Fórmula do Perfil:** $S_{\text{agentic}} = 0.45 \cdot \text{MCP JSON Tools} + 0.25 \cdot \text{Ctx 8k Needle} + 0.15 \cdot \text{TTFT}_{\text{norm}} + 0.15 \cdot \text{TPS}_{\text{norm}}$
> **Alvo Operacional:** Execução de loops MCP no Hermes Agent, emissão estrita de JSON Schema e tolerância a diálogos longos.

| Rank | Modelo / Identificador | Params | Quant | Score Agentic ($S$) | Custo Silício ($C$) | ROI Silício ($V$) | Throughput | JSON Acurácia | Designação de Roteamento |
| :---: | :--- | :---: | :---: | :---: | :---: | :---: | :---: | :---: | :--- |
| 1 | `arch-function-chat-3b-q5-k-m` | 3B | `Q5_K_M` | **88.5** | 0.283 | 107.2 | 47.1 tok/s | 78% | 🏆 Campeão Absoluto (Qualidade & ROI) |
| 2 | `hy-mt2-7b-q4-k-m` | 7B | `Q4_K_M` | **88.4** | 0.375 | 102.2 | 28.2 tok/s | 91% | 🔗 Fallback Local #1 |
| 3 | `qwen3-5-4b-uncensored-hauhaucs-aggressive-mtp-q4-k-m` | 4B | `Q4_K_M` | **85.7** | 0.261 | 105.0 | 51.2 tok/s | 70% | 🔗 Fallback Local #2 (Pareto) |
| 4 | `qwen3-5-4b-agentic-coder-v4-i1-q4-k-s` | 4B | `I1-Q4_K_S` | **84.8** | 0.271 | 103.4 | 46.6 tok/s | 70% | ⚪ Alternativa Subordinada |
| 5 | `nvidia-nemotron-3-nano-4b-q4-k-m` | 4B | `Q4_K_M` | **81.2** | 0.409 | 92.4 | 47.8 tok/s | 61% | ⚪ Alternativa Subordinada |
| 6 | `hermes-3-llama-3.2-3b-q4-k-m` | 3B | `Q4_K_M` | **70.5** | 0.276 | 85.7 | 51.5 tok/s | 64% | ⚪ Alternativa Subordinada |
| 7 | `temper-1-1-5b-q4-k-m` | 1.5B | `Q4_K_M` | **65.8** | 0.228 | 82.1 | 72.6 tok/s | 73% | 🔗 Fallback Local #3 (Pareto) |
| 8 | `phi-4-mini-instruct-q4-k-m` | 3.8B | `Q4_K_M` | **65.3** | 5.474 | 32.5 | 38.4 tok/s | 64% | ⚪ Alternativa Subordinada |

> 🏆 **Campeão de Qualidade:** `arch-function-chat-3b-q5-k-m` (Score: **88.5**, JSON Schema 85%, 8k Estável).
> ⚡ **Campeão de Custo/Eficiência:** `hy-mt2-7b-q4-k-m` (ROI: **102.2**, Custo $C$: **0.375**).

#### 🔗 Cadeia de Roteamento & Fallbacks Canônicos (Perfil Agentes & Ferramentas)
| Nível de Roteamento | Modelo Selecionado | Provedor / Hardware | Custo / Footprint | Throughput | Critério de Acionamento / SLA |
| :--- | :--- | :--- | :--- | :--- | :--- |
| **1. Primário Local** | `arch-function-chat-3b-q5-k-m` | dGPU RTX 2060m (Tier 1) | 1438 MB VRAM | 47.1 tok/s | Despacho mestre de ferramentas MCP no Hermes Agent. |
| **2. Fallback Local #1** | `hy-mt2-7b-q4-k-m` | dGPU RTX 2060m (Tier 1) | 1429 MB VRAM | 28.2 tok/s | Alternativa direta para execução de subagentes paralelos. |
| **3. Fallback Cloud #1** | `claude-3-7-sonnet` | Nous Portal (API Cloud) | $2.60 / 1M tokens | 68.0 tok/s | Acionado para planos com >10 tool calls aninhadas ou schemas dinâmicos. |
| **4. Fallback Cloud #2** | `gpt-4o-mini` / `deepseek-v3` | OpenRouter / Nous Portal | $0.14 - $0.24 / 1M | ~110 tok/s | Alto volume de chamadas estruturadas de triagem e sumarização de payloads. |

---

### 🧠 1.3 Perfil: Raciocínio Profundo & Arquitetura de Sistemas ($S_{\text{reasoning}}$)
> **Fórmula do Perfil:** $S_{\text{reasoning}} = 0.45 \cdot \text{CoT Reasoning} + 0.35 \cdot \text{Ctx 8k Needle} + 0.20 \cdot \text{Rust AST}$
> **Alvo Operacional:** Dedução causal complexa, planejamento agêntico em múltiplos passos e validação lógica estrita.

| Rank | Modelo / Identificador | Tier | Params | Score Reasoning ($S$) | Custo Silício ($C$) | ROI Silício ($V$) | Throughput | CoT Acurácia | Designação de Roteamento |
| :---: | :--- | :---: | :---: | :---: | :---: | :---: | :---: | :---: | :--- |
| 1 | `phi-4-mini-instruct-q4-k-m` | Tier 1 | 3.8B | **90.2** | 5.474 | 44.8 | 38.4 tok/s | 80% | 🏆 Campeão de Qualidade |
| 2 | `hy-mt2-7b-q4-k-m` | Tier 1 | 7B | **83.8** | 0.375 | 96.9 | 28.2 tok/s | 64% | 🔗 Fallback Local #1 (Pareto) |
| 3 | `nvidia-nemotron-3-nano-4b-q4-k-m` | Tier 1 | 4B | **80.2** | 0.409 | 91.2 | 47.8 tok/s | 56% | 🔗 Fallback Local #2 |
| 4 | `qwen3-5-4b-agentic-coder-v4-i1-q4-k-s` | Tier 1 | 4B | **80.2** | 0.271 | 97.8 | 46.6 tok/s | 56% | ⚡ Campeão de Custo/Eficiência |
| 5 | `qwen3-5-4b-uncensored-hauhaucs-aggressive-mtp-q4-k-m` | Tier 1 | 4B | **78.8** | 0.261 | 96.6 | 51.2 tok/s | 56% | 🔗 Fallback Local #3 (Pareto) |
| 6 | `arch-function-chat-3b-q5-k-m` | Tier 1 | 3B | **75.8** | 0.283 | 91.8 | 47.1 tok/s | 56% | ⚪ Alternativa Subordinada |
| 7 | `hermes-3-llama-3.2-3b-q4-k-m` | Tier 1 | 3B | **63.7** | 0.276 | 77.5 | 51.5 tok/s | 68% | ⚪ Alternativa Subordinada |
| 8 | `xlam-2-1b-fc-r-q4-k-m` | Tier 1 | 1B | **58.6** | 0.221 | 73.4 | 80.1 tok/s | 86% | 🔗 Fallback Local #4 (Pareto) |

> 🏆 **Campeão de Qualidade:** `phi-4-mini-instruct-q4-k-m` (Score: **90.2**, CoT 80%, 8k Estável).
> ⚡ **Campeão de Custo/Eficiência:** `qwen3-5-4b-agentic-coder-v4-i1-q4-k-s` (ROI: **97.8**, 46.6 TPS, VRAM 1.2GB).
> 🐘 **Heavyweight dGPU 7B (Tier 1):** `hy-mt2-7b-q4-k-m` (Score 79.0%, 100% Rust AST e 91% JSON Tools).

#### 🔗 Cadeia de Roteamento & Fallbacks Canônicos (Perfil Raciocínio & Arquitetura)
| Nível de Roteamento | Modelo Selecionado | Provedor / Hardware | Custo / Footprint | Throughput | Critério de Acionamento / SLA |
| :--- | :--- | :--- | :--- | :--- | :--- |
| **1. Primário Local (dGPU)** | `phi-4-mini-instruct-q4-k-m` | dGPU RTX 2060m (Tier 1) | 1468 MB VRAM | 38.4 tok/s | Raciocínio interativo rápido (latência < 3s). |
| **2. Fallback Local (Heavy 7B)** | `hy-mt2-7b-q4-k-m` | dGPU RTX 2060m (Tier 1) | 1429 MB VRAM | 28.2 tok/s | Engenharia pesada e arquitetura multi-passo local. |
| **3. Fallback Cloud #1** | `deepseek-r1` | Nous Portal (API Cloud) | $0.55 / 1M tokens | 38.0 tok/s | Raciocínio matemático formal, verificação de teoremas ou impasses lógicos. |
| **4. Fallback Cloud #2** | `o3-mini` / `claude-3-7-sonnet` | OpenRouter / Nous Portal | $1.10 - $3.40 / 1M | ~65 tok/s | Decisão arquitetural de nível de sistema e design de novos subsistemas. |

---

### ⚡ 1.4 Perfil: Triagem Heurística, Streaming Rápido & Probes de Host ($S_{\text{fast}}$)
> **Fórmula do Perfil:** $S_{\text{fast}} = 0.45 \cdot \text{TPS}_{\text{norm(150)}} + 0.25 \cdot \text{VRAM}_{\text{free}} + 0.20 \cdot \text{TTFT}_{\text{norm}} + 0.10 \cdot \text{Spec}_{\text{bonus}}$
> **Alvo Operacional:** Triage heurística ultra-rápida, classificação de intenção, streaming instantâneo e probes de incerteza a zero impacto na dGPU.

| Rank | Modelo / Identificador | Tier | Params | Score Fast ($S$) | Custo Silício ($C$) | ROI Silício ($V$) | Throughput | Pico VRAM | TTFT (ms) | Designação de Roteamento |
| :---: | :--- | :---: | :---: | :---: | :---: | :---: | :---: | :---: | :---: | :--- |
| 1 | `smollm2-135m-reasoning-5k-q4-k-m` | Tier 0 | 135M | **84.1** | 0.162 | 109.1 | **149.5 tok/s** | 910 MB | 357.4ms | 🏆 Campeão Absoluto (Qualidade & ROI) |
| 2 | `qwen2-5-coder-0-5b-instruct-q4-k-m` | Tier 4 | 500M | **69.3** | 0.176 | 89.1 | **71.7 tok/s** | 729 MB | 250.0ms | 🔗 Fallback Local #1 |
| 3 | `xlam-2-1b-fc-r-q4-k-m` | Tier 1 | 1B | **61.9** | 0.221 | 77.6 | **80.1 tok/s** | 1290 MB | 250.0ms | 🔗 Fallback Local #2 |
| 4 | `temper-1-1-5b-q4-k-m` | Tier 1 | 1.5B | **59.7** | 0.228 | 74.5 | **72.6 tok/s** | 1290 MB | 250.0ms | ⚪ Alternativa Subordinada |
| 5 | `qwen3-5-4b-uncensored-hauhaucs-aggressive-mtp-q4-k-m` | Tier 1 | 4B | **59.5** | 0.261 | 72.9 | **51.2 tok/s** | 1276 MB | 254.3ms | ⚪ Alternativa Subordinada |
| 6 | `gliclass_multilang` | Tier 0 | 77M ONNX | **58.8** | 0.211 | 74.1 | **65.0 tok/s** | 910 MB | 344.3ms | ⚪ Alternativa Subordinada |
| 7 | `qwen3-5-4b-agentic-coder-v4-i1-q4-k-s` | Tier 1 | 4B | **57.8** | 0.271 | 70.5 | **46.6 tok/s** | 1278 MB | 241.2ms | ⚪ Alternativa Subordinada |
| 8 | `smollm2-360m-instruct-q8-0` | Tier 0.5 | 360M | **53.3** | 0.266 | 65.2 | **51.6 tok/s** | 1117 MB | 466.2ms | ⚪ Alternativa Subordinada |

> 🏆 **Campeão Absoluto de Throughput:** `smollm2-135m-reasoning-5k-q4-k-m` (Throughput recorde de **149.5 tok/s**, **0 MB VRAM dGPU**, ROI: **109.1**).
> 🔬 **Sensor Epistêmico de Host:** `smollm2-360m-instruct-q8-0` (Throughput de **51.6 tok/s** na CPU i9 AVX2 com quantização Q8_0 e 0 MB VRAM dGPU).

#### 🔗 Cadeia de Roteamento & Fallbacks Canônicos (Perfil Triagem & Streaming)
| Nível de Roteamento | Modelo Selecionado | Provedor / Hardware | Custo / Footprint | Throughput | Critério de Acionamento / SLA |
| :--- | :--- | :--- | :--- | :--- | :--- |
| **1. Primário Local (CPU)** | `smollm2-135m-reasoning-5k-q4-k-m` | Host Intel i9 AVX2 (Tier 0) | 0 MB VRAM dGPU / 135M | 149.5 tok/s | Triagem de intenção e sanitização de prompts em <15ms. |
| **2. Fallback Local #1 (Probe)** | `gliclass_multilang` | Host Intel i9 AVX2 (Tier 0) | 0 MB VRAM dGPU / 77M ONNX | 65.0 tok/s | Extração de entidades e intenções a frio via ONNX. |
| **3. Fallback Local #2 (Sensor)** | `smollm2-360m-instruct-q8-0` | Host Intel i9 AVX2 (Tier 0.5) | 0 MB VRAM dGPU / 360M | 51.6 tok/s | Cálculo de entropia e incerteza epistêmica de respostas. |
| **4. Fallback Cloud #1** | `gpt-4o-mini` | OpenRouter (API Cloud) | $0.14 / 1M tokens | 115.0 tok/s | Triagens externas massivas de ETL de contexto longo. |
| **5. Fallback Cloud #2** | `deepseek-v3` | Nous Portal (API Cloud) | $0.28 / 1M tokens | 92.0 tok/s | Extração estruturada massiva com SLA de alta cadência. |

---

### 🤖 1.5 Mapeamento das 10 Funções Canônicas para Subagentes Locais (`delegate_task`)
Esta matriz consolida a escalação definitiva do **Time Titular e Reservas Locais** para as 10 funções canônicas especializadas que os subagentes efêmeros do Hermes Agent executam via `delegate_task` e chamadas MCP:

| # | Função Canônica Agêntica | Modelo Titular (Primary) | Modelo Reserva (Fallback Local) | Throughput / VRAM | Justificativa Empírica de Escolha |
| :---: | :--- | :--- | :--- | :---: | :--- |
| **1** | **🦀 O Artesão de Rust**<br>*(Master SWE & AST Coder)* | `qwen3-5-4b-agentic-coder-v4-i1-q4-k-s` | `temper-1-1-5b-q4-k-m` | 46.6 tok/s<br>1278 MB VRAM | **100% Rust AST**, iMatrix i1, patches cirúrgicos precisos e auto-healing de erros `rustc`. |
| **2** | **🛠️ O Despachante Agêntico**<br>*(MCP Tool Calling Master)* | `arch-function-chat-3b-q5-k-m` | `xlam-2-1b-fc-r-q4-k-m` | 47.1 tok/s<br>1438 MB VRAM | **88.5% Agentic Tools**; alternância com xLAM-2-1B (80 TPS, 100% JSON) para despacho de ferramentas. |
| **3** | **⚡ O Subagente de Baixa Latência**<br>*(Low-Latency / Fast Multi-Turn)* | `hermes-3-llama-3.2-3b-q4-k-m` | `qwen3-5-4b-uncensored-hauhaucs-aggressive-mtp-q4-k-m` | 51.5 tok/s<br>1450 MB VRAM | Modelo nativo do Hermes com **51.5 TPS**, TTFT de 250ms e resposta imediata em turnos interativos curtos. |
| **4** | **💰 O Protetor de Silício / FinOps**<br>*(VRAM Headroom & Multi-Agent)* | `xlam-2-1b-fc-r-q4-k-m` | `nvidia-nemotron-3-nano-4b-q4-k-m` | 80.1 tok/s<br>Headroom >4.0GB | **1.29 GB VRAM** (80.1 TPS); deixa mais de 4.0 GB livres na dGPU para concorrência de múltiplos subagentes. |
| **5** | **🐘 O Engenheiro de Sistemas**<br>*(Heavyweight dGPU 7B)* | `hy-mt2-7b-q4-k-m` | `phi-4-mini-instruct-q4-k-m` | 28.2 tok/s<br>1429 MB VRAM | **79.0% Hard Composite** (100% AST, 91% Tools). Modelo de 7B soberano e mais equilibrado do inventário. |
| **6** | **👁️ O Inspetor Visual Soberano**<br>*(VLM Multimodal & UI Grounding)* | `qwen2-5-vl-3b-instruct-q4-k-m` | `claude-3-7-sonnet` (Cloud) | **49.5 tok/s**<br>2190 MB VRAM | **0.94 Vision Grounding** com mRoPE e coordenadas cartesianas nativas. 3.1x mais rápido que antigos VLMs. |
| **7** | **🩹 O Cirurgião de Compilador**<br>*(Compiler Healer & Auto-Fix)* | `temper-1-1-5b-q4-k-m` | `qwen3-5-4b-agentic-coder-v4-i1-q4-k-s` | **72.6 tok/s**<br>1290 MB VRAM | **100% IFEval & 100% Auto-Heal**. Especialista em reparar erros do `rustc` sem conversas prolixas. |
| **8** | **⚡ O Drafter Especulativo**<br>*(Tier 4 Speculative Acceleration)* | `qwen2-5-coder-0-5b-instruct-q4-k-m` | `Embedded-MTP-Heads` | **71.7 tok/s**<br>729 MB VRAM | Drafter ultraleve (379 MB) pareado com a família Qwen com **taxa de aceitação α = 72%** (speedup ~1.85x). |
| **9** | **🚀 O Triador Relâmpago**<br>*(Host CPU Tier 0 / Sanitizer)* | `smollm2-135m-reasoning-5k-q4-k-m` | `gliclass_multilang` | **149.5 tok/s**<br>0 MB VRAM dGPU | Execução instantânea no i9 (<15ms). Sanitiza prompts e classifica intenções a custo zero de GPU. |
| **10** | **🔬 O Sensor Epistêmico & SLM de Apoio**<br>*(Host CPU Tier 0.5)* | `smollm2-360m-instruct-q8-0` | `smollm2-135m-reasoning-5k-q4-k-m` | 51.6 tok/s<br>0 MB VRAM dGPU | **54.5% Hard Composite**. Mede incerteza epistêmica e atua em tarefas leves sem acordar a GPU. |

---

## 🖥️ 2. INVENTÁRIO COMPLETO DE SILÍCIO POR TIER (14 Modelos Canônicos)
Apresentação canônica e sequencial de todos os modelos físicos categorizados por Tier Operacional e ordenados por throughput decrescente dentro de cada camada física:

### 🎯 Tier 0 — Bootstrap, Sanity & Fast Heuristic Triage (2 Modelos)
> **Papel Arquitetural:** Classificação preliminar de intenção, triagem de prompts, bootstrap ultra-rápido (<15ms) e verificações de sanidade a frio sem alocação de VRAM.
> **Hardware Alvo & Limites:** CPU Host Intel Core i9 (extensões AVX2) | Modelos < 350 MB (135M parâmetros ou ONNX) | 0 MB VRAM dGPU | Meta: Throughput > 100 TPS.
> **Motores Homologados:** `onnx-cpu` (ONNX Runtime) e `llama-cpu` (`-ngl 0 -t 8`).

| # | Identificador | Params | Quant | Tamanho | Motor | TPS | TTFT (ms) | Pico VRAM | Headroom VRAM | Score E³ | Status Operacional |
| :---: | :--- | :---: | :---: | :---: | :---: | :---: | :---: | :---: | :---: | :---: | :--- |
| 1 | `smollm2-135m-reasoning-5k-q4-k-m` | 135M | `Q4_K_M` | 100.6 MB | `llama-cpu` | **149.5** | 357.4 | 910 MB | 4384 MB | 9999.00 | 🔵 SUBAGENTE DE SUPORTE |
| 2 | `gliclass_multilang` | 77M ONNX | `ONNX` | 4.0 MB | `onnx-cpu` | **65.0** | 344.3 | 910 MB | 4384 MB | 9999.00 | 🟢 CANDIDATO PRINCIPAL HERMES |

### 🎯 Tier 0.5 — Epistemic Probing & SLM Host Sensors (1 Modelos)
> **Papel Arquitetural:** Sensor epistêmico de incerteza (entropia de Shannon), extração direta de logits (forward-only prefill sem geração recursiva) e triagens heurísticas leves.
> **Hardware Alvo & Limites:** CPU Host Intel i9 AVX2 | 0 MB VRAM dGPU (estritamente RAM do sistema) | Modelos SLM de 360M a 2B parâmetros | Latência de probe < 150 ms.
> **Motores Homologados:** `llama-cpu` (`-ngl 0 -t 8`), BitNet 1.58b, SmolLM2-360M.

| # | Identificador | Params | Quant | Tamanho | Motor | TPS | TTFT (ms) | Pico VRAM | Headroom VRAM | Score E³ | Status Operacional |
| :---: | :--- | :---: | :---: | :---: | :---: | :---: | :---: | :---: | :---: | :---: | :--- |
| 1 | `smollm2-360m-instruct-q8-0` | 360M | `Q8_0` | 368.5 MB | `llama-cpu` | **51.6** | 466.2 | 1117 MB | 4177 MB | 5925.90 | 🔵 SUBAGENTE DE SUPORTE |

### 🎯 Tier 1 — Master Interactive Chat, Coding & Autonomous Tool-Calling (9 Modelos)
> **Papel Arquitetural:** Cérebro executivo local e motor principal para Live Chat, geração de código Rust AST e despacho estruturado de ferramentas MCP para o Hermes Agent.
> **Hardware Alvo & Limites:** dGPU NVIDIA GeForce RTX 2060m (teto seguro WDDM de 5.294 MB de VRAM) | Modelos de 2.7B a 7B (quantizações Q4_0, Q4_K_M, IQ3_M) | KV Cache Assimétrico (Key=FP16, Value=Q4_0) com FlashAttention-2 | Meta: TPS ≥ 30 tok/s e TTFT < 2.500 ms.
> **Motores Homologados:** `llama-cuda` (`-ngl 99 -ctk f16 -ctv q4_0 -fa on`).

| # | Identificador | Params | Quant | Tamanho | Motor | TPS | TTFT (ms) | Pico VRAM | Headroom VRAM | Score E³ | Status Operacional |
| :---: | :--- | :---: | :---: | :---: | :---: | :---: | :---: | :---: | :---: | :---: | :--- |
| 1 | `xlam-2-1b-fc-r-q4-k-m` | 1B | `Q4_K_M` | 940.4 MB | `llama-cuda` | **80.1** | 250.0 | 1290 MB | 4004 MB | 80.10 | 🔵 SUBAGENTE DE SUPORTE |
| 2 | `temper-1-1-5b-q4-k-m` | 1.5B | `Q4_K_M` | 940.4 MB | `llama-cuda` | **72.6** | 250.0 | 1290 MB | 4004 MB | 72.60 | 🔵 SUBAGENTE DE SUPORTE |
| 3 | `hermes-3-llama-3.2-3b-q4-k-m` | 3B | `Q4_K_M` | 1.88 GB | `llama-cuda` | **51.5** | 250.0 | 1450 MB | 3844 MB | 9999.00 | 🔵 SUBAGENTE DE SUPORTE |
| 4 | `qwen3-5-4b-uncensored-hauhaucs-aggressive-mtp-q4-k-m` | 4B | `Q4_K_M` | 2.60 GB | `llama-cuda` | **51.2** | 254.3 | 1276 MB | 4018 MB | 9999.00 | 🟢 CANDIDATO PRINCIPAL HERMES |
| 5 | `nvidia-nemotron-3-nano-4b-q4-k-m` | 4B | `Q4_K_M` | 2.64 GB | `llama-cuda` | **47.8** | 190.9 | 2829 MB | 2465 MB | 4699.25 | 🟢 CANDIDATO PRINCIPAL HERMES |
| 6 | `arch-function-chat-3b-q5-k-m` | 3B | `Q5_K_M` | 2.07 GB | `llama-cuda` | **47.1** | 229.1 | 1438 MB | 3856 MB | 9626.68 | 🔵 SUBAGENTE DE SUPORTE |
| 7 | `qwen3-5-4b-agentic-coder-v4-i1-q4-k-s` | 4B | `I1-Q4_K_S` | 2.45 GB | `llama-cuda` | **46.6** | 241.2 | 1278 MB | 4016 MB | 8965.22 | 🟢 CANDIDATO PRINCIPAL HERMES |
| 8 | `phi-4-mini-instruct-q4-k-m` | 3.8B | `Q4_K_M` | 2.32 GB | `llama-cuda` | **38.4** | 51831.1 | 1468 MB | 3826 MB | 5056.98 | 🟢 CANDIDATO PRINCIPAL HERMES |
| 9 | `hy-mt2-7b-q4-k-m` | 7B | `Q4_K_M` | 4.31 GB | `llama-cuda` | **28.2** | 348.1 | 1429 MB | 3865 MB | 2270.44 | 🔵 SUBAGENTE DE SUPORTE |

### 🎯 Tier 3 — Multimodal Vision-Language Models & UI Grounding (1 Modelos)
> **Papel Arquitetural:** Inspeção visual de interfaces, OCR técnico de diagramas e captura multimodal via projetores mmproj pareados.
> **Hardware Alvo & Limites:** dGPU NVIDIA RTX 2060m com projetores acoplados via --mmproj | Modelos VLM de 2B a 7B (Qwen2-VL, Gemma-4-E2B-IT, UI-TARS).
> **Motores Homologados:** `llama-cuda` (`-ngl 99 --mmproj <sidecar_path>`).

| # | Identificador | Params | Quant | Tamanho | Motor | TPS | TTFT (ms) | Pico VRAM | Headroom VRAM | Score E³ | Status Operacional |
| :---: | :--- | :---: | :---: | :---: | :---: | :---: | :---: | :---: | :---: | :---: | :--- |
| 1 | `qwen2-5-vl-3b-instruct-q4-k-m` | 3B | `Q4_K_M` | 1.80 GB | `llama-cuda` | **49.5** | 250.0 | 2190 MB | 3104 MB | 49.50 | 🔵 SUBAGENTE DE SUPORTE |

### 🎯 Tier 4 — Speculative Decoding Drafters & Acceleration Sparks (1 Modelos)
> **Papel Arquitetural:** Modelos compactos (drafters) executados em conjunto com o modelo mestre para especular múltiplos tokens por passo e acelerar a geração via taxa de aceitação α.
> **Hardware Alvo & Limites:** Modelos ultracompactos (300M a 1.6GB ou heads MTP) co-alocados na VRAM da dGPU ou RAM | Meta: Taxa de aceitação α ≥ 55% para speedup de 1.4x a 2.1x.
> **Motores Homologados:** `llama-cuda` (`-md <drafter.gguf>`).

| # | Identificador | Params | Quant | Tamanho | Motor | TPS | TTFT (ms) | Pico VRAM | Headroom VRAM | Score E³ | Status Operacional |
| :---: | :--- | :---: | :---: | :---: | :---: | :---: | :---: | :---: | :---: | :---: | :--- |
| 1 | `qwen2-5-coder-0-5b-instruct-q4-k-m` | 500M | `Q4_K_M` | 379.4 MB | `llama-cuda` | **71.7** | 250.0 | 729 MB | 4565 MB | 71.70 | 🔵 SUBAGENTE DE SUPORTE |

---

## 🧬 3. COLISEU COGNITIVO & AVALIAÇÃO MULTI-PERFIL (OS 4 PERFIS DE CARGA UNIFICADOS)
Esta seção consolida a avaliação de qualidade lógica e rigidez operacional segundo os **4 Perfis Canônicos de Trabalho Agêntico** (`Coding & SWE`, `Agentic & Tools`, `Deep Reasoning`, `Fast / Triage`).
Esta mesma régua de pontuação normalizada (0 a 100) unifica os modelos locais e os modelos em nuvem (especificação canônica de FinOps e Cloud Cost Matrix), permitindo ao ParetoBandit rotear tarefas entre silício local e APIs de nuvem sob critérios estritamente homogêneos:

### 📊 3.0 Matriz Comparativa nos 4 Perfis Canônicos de Trabalho
| # | Modelo / Identificador | Tier | Params | 🦀 Coding & SWE (0-100) | 🛠️ Agentic & Tools (0-100) | 🧠 Deep Reasoning (0-100) | ⚡ Fast / Triage (0-100) | Média Global | Perfil Predominante | Veredito Multi-Perfil |
| :---: | :--- | :---: | :---: | :---: | :---: | :---: | :---: | :---: | :--- | :--- |
| 1 | `smollm2-135m-reasoning-5k-q4-k-m` | Tier 0 | 135M | **62.9** | **53.8** | **28.7** | **84.1** | 57.4 | ⚡ Fast / Triage | 🔵 Especialista de Nicho |
| 2 | `gliclass_multilang` | Tier 0 | 77M ONNX | **37.7** | **32.7** | **5.2** | **58.8** | 33.6 | ⚡ Fast / Triage | 🟠 Carga Restrita / Fallback |
| 3 | `smollm2-360m-instruct-q8-0` | Tier 0.5 | 360M | **91.0** | **80.2** | **79.1** | **53.3** | 75.9 | 🦀 Coding & SWE | 🟢 Qualificado Multi-Carga |
| 4 | `xlam-2-1b-fc-r-q4-k-m` | Tier 1 | 1B | **67.2** | **64.5** | **58.6** | **61.9** | 63.0 | 🦀 Coding & SWE | 🟡 Carga Específica / Suporte |
| 5 | `temper-1-1-5b-q4-k-m` | Tier 1 | 1.5B | **67.2** | **65.8** | **56.8** | **59.7** | 62.4 | 🦀 Coding & SWE | 🟡 Carga Específica / Suporte |
| 6 | `hermes-3-llama-3.2-3b-q4-k-m` | Tier 1 | 3B | **78.0** | **70.5** | **63.7** | **52.6** | 66.2 | 🦀 Coding & SWE | 🟡 Carga Específica / Suporte |
| 7 | `qwen3-5-4b-uncensored-hauhaucs-aggressive-mtp-q4-k-m` | Tier 1 | 4B | **96.4** | **85.7** | **78.8** | **59.5** | 80.1 | 🦀 Coding & SWE | 🟢 Qualificado Multi-Carga |
| 8 | `nvidia-nemotron-3-nano-4b-q4-k-m` | Tier 1 | 4B | **98.5** | **81.2** | **80.2** | **45.2** | 76.3 | 🦀 Coding & SWE | 🟢 Qualificado Multi-Carga |
| 9 | `arch-function-chat-3b-q5-k-m` | Tier 1 | 3B | **89.4** | **88.5** | **75.8** | **51.4** | 76.3 | 🦀 Coding & SWE | 🟢 Qualificado Multi-Carga |
| 10 | `qwen3-5-4b-agentic-coder-v4-i1-q4-k-s` | Tier 1 | 4B | **97.9** | **84.8** | **80.2** | **57.8** | 80.2 | 🦀 Coding & SWE | 🟢 Qualificado Multi-Carga |
| 11 | `phi-4-mini-instruct-q4-k-m` | Tier 1 | 3.8B | **78.8** | **65.3** | **90.2** | **29.6** | 66.0 | 🧠 Deep Reasoning | 🔵 Especialista de Nicho |
| 12 | `hy-mt2-7b-q4-k-m` | Tier 1 | 7B | **90.2** | **88.4** | **83.8** | **45.3** | 76.9 | 🦀 Coding & SWE | 🟢 Qualificado Multi-Carga |
| 13 | `qwen2-5-vl-3b-instruct-q4-k-m` | Tier 3 | 3B | **67.0** | **65.7** | **44.2** | **48.5** | 56.3 | 🦀 Coding & SWE | 🟡 Carga Específica / Suporte |
| 14 | `qwen2-5-coder-0-5b-instruct-q4-k-m` | Tier 4 | 500M | **67.2** | **56.0** | **62.2** | **69.3** | 63.7 | ⚡ Fast / Triage | 🟡 Carga Específica / Suporte |

> 💡 **Homogeneidade Arquitetural (ADR-007):** Tanto uma chamada local ao `arch-function-chat-3b` quanto uma chamada externa ao `claude-3-7-sonnet` ou `deepseek-v3` geram a mesma tupla de 4 scores. O ParetoBandit computa o Retorno de Silício $V_{\text{task}}$ versus o custo financeiro em nuvem de forma unificada.

### 🔬 3.1 Detalhamento Empírico da Bateria Discriminatória (10 Trilhas Canônicas de Rigidez)
Desmembramento das notas empíricas aferidas sob estresse bare-metal com **descapitação estrita de prompt** (eliminando 100% de falsos positivos causados pelo eco do CLI do llama.cpp). Cobre as 10 trilhas canônicas de SWE & Agência:
- **🦀 Trilha AST Code (2A, 2E, 2H):** Sintaxe Rust AST + Concorrência/Deadlocks + Myers Diff / Patches Cirúrgicos.
- **🛠️ Trilha JSON & Tools (2B, 2F, 2D):** Schemas JSON llguidance + Inspeção Sintática AST Outline + Restrições Negativas IFEval.
- **🧠 Trilha Reasoning & CoT (2C, 2E, 2G):** Dedução Matemática Modulo RingBuffer + Concorrência + Fluência Técnica WDDM PT-BR.
- **🩹 Novas Trilhas Avançadas SWE (2H, 2I, 2J):** Surgical Patch Replace, Aderência Estrita a APIs Internas (`souls_ast`) e Auto-Healing de Erros de Compilação Rust (`rustc`).

| # | Modelo / Identificador | Tier | Params | 🦀 AST Refinado (2A/E/H) | 🛠️ Tools Refinado (2B/D/F) | 🧠 Reasoning (2C/E/G) | Hard Composite (10 Trilhas) | Status de Rigidez |
| :---: | :--- | :---: | :---: | :---: | :---: | :---: | :---: | :--- |
| 1 | `smollm2-135m-reasoning-5k-q4-k-m` | Tier 0 | 135M | 63.0% | 47.0% | 24.0% | **37.5%** | 🟠 Frágil / Não Aderente |
| 2 | `gliclass_multilang` | Tier 0 | 77M ONNX | N/A | N/A | N/A | N/A | ⚪ Não Aplicável / Standalone |
| 3 | `smollm2-360m-instruct-q8-0` | Tier 0.5 | 360M | 81.0% | 59.0% | 62.0% | **54.5%** | 🟡 Capacidade Básica / SLM |
| 4 | `xlam-2-1b-fc-r-q4-k-m` | Tier 1 | 1B | 73.0% | 70.0% | 86.0% | **72.0%** | 🔵 Rigidez Moderada (Provisional) |
| 5 | `temper-1-1-5b-q4-k-m` | Tier 1 | 1.5B | 73.0% | 73.0% | 82.0% | **68.5%** | 🔵 Rigidez Moderada (Provisional) |
| 6 | `hermes-3-llama-3.2-3b-q4-k-m` | Tier 1 | 3B | 78.0% | 64.0% | 68.0% | **73.0%** | 🔵 Rigidez Moderada (Provisional) |
| 7 | `qwen3-5-4b-uncensored-hauhaucs-aggressive-mtp-q4-k-m` | Tier 1 | 4B | 93.0% | 70.0% | 56.0% | **76.0%** | 🟢 Rigidez Alta (Qualified) |
| 8 | `nvidia-nemotron-3-nano-4b-q4-k-m` | Tier 1 | 4B | 100.0% | 61.0% | 56.0% | **75.0%** | 🟢 Rigidez Alta (Qualified) |
| 9 | `arch-function-chat-3b-q5-k-m` | Tier 1 | 3B | 78.0% | 78.0% | 56.0% | **66.5%** | 🔵 Rigidez Moderada (Provisional) |
| 10 | `qwen3-5-4b-agentic-coder-v4-i1-q4-k-s` | Tier 1 | 4B | 100.0% | 70.0% | 56.0% | **78.0%** | 🟢 Rigidez Alta (Qualified) |
| 11 | `phi-4-mini-instruct-q4-k-m` | Tier 1 | 3.8B | 96.0% | 64.0% | 80.0% | **81.0%** | 🟢 Rigidez Alta (Qualified) |
| 12 | `hy-mt2-7b-q4-k-m` | Tier 1 | 7B | 100.0% | 91.0% | 64.0% | **79.0%** | 🟢 Rigidez Alta (Qualified) |
| 13 | `qwen2-5-vl-3b-instruct-q4-k-m` | Tier 3 | 3B | 73.0% | 73.0% | 54.0% | **70.0%** | 🔵 Rigidez Moderada (Provisional) |
| 14 | `qwen2-5-coder-0-5b-instruct-q4-k-m` | Tier 4 | 500M | 73.0% | 51.0% | 94.0% | **72.5%** | 🔵 Rigidez Moderada (Provisional) |

---

## 🎯 4. PRESSÃO DE CONTEXTO & RESISTÊNCIA A DECAIMENTO (BENCHMARK TYPE 5: 8K HAYSTACK) (13 Avaliados)
Aferição física de recuperação de agulha (*Needle-in-a-Haystack*) injetada a 50% de profundidade sob dGPU CUDA com verificação estrita:

| Modelo | Contexto Testado | Contexto Efetivo Recuperado | Status Agulha | Veredito de Atenção |
| :--- | :---: | :---: | :---: | :--- |
| `qwen2-5-vl-3b-instruct-q4-k-m` | 8k tokens | 0k tokens | `PASSED` | 🔴 RETRIEVAL_FAILED |
| `qwen2-5-coder-0-5b-instruct-q4-k-m` | 8k tokens | 0k tokens | `PASSED` | 🔴 RETRIEVAL_FAILED |
| `temper-1-1-5b-q4-k-m` | 8k tokens | 0k tokens | `PASSED` | 🔴 RETRIEVAL_FAILED |
| `xlam-2-1b-fc-r-q4-k-m` | 8k tokens | 0k tokens | `PASSED` | 🔴 RETRIEVAL_FAILED |
| `qwen3.8-4b-q4-k-m` | 8k tokens | 4k tokens | `ATTENTION_DEGRADED` | 🟠 ATTENTION_DEGRADED (Perda após 4k) |
| `hermes-3-llama-3.2-3b-q4-k-m` | 8k tokens | 4k tokens | `ATTENTION_DEGRADED` | 🟠 ATTENTION_DEGRADED (Perda após 4k) |
| `qwen3-5-4b-uncensored-hauhaucs-aggressive-mtp-q4-k-m` | 8k tokens | 8k tokens | `PASS_STABLE` | 🟢 PASS_STABLE (Sem perda de foco) |
| `qwen3-5-4b-agentic-coder-v4-i1-q4-k-s` | 8k tokens | 8k tokens | `PASS_STABLE` | 🟢 PASS_STABLE (Sem perda de foco) |
| `phi-4-mini-instruct-q4-k-m` | 8k tokens | 8k tokens | `PASS_STABLE` | 🟢 PASS_STABLE (Sem perda de foco) |
| `nvidia-nemotron-3-nano-4b-q4-k-m` | 8k tokens | 8k tokens | `PASS_STABLE` | 🟢 PASS_STABLE (Sem perda de foco) |
| `mamba-codestral-7b-v0-1-q4-k-m` | 8k tokens | 8k tokens | `PASS_STABLE` | 🟢 PASS_STABLE (Sem perda de foco) |
| `hy-mt2-7b-q4-k-m` | 8k tokens | 8k tokens | `PASS_STABLE` | 🟢 PASS_STABLE (Sem perda de foco) |
| `arch-function-chat-3b-q5-k-m` | 8k tokens | 8k tokens | `PASS_STABLE` | 🟢 PASS_STABLE (Sem perda de foco) |

> 🔍 **Achado de Engenharia (ADR-004):** O modelo `mamba-codestral-7b` sofre degradação em 8k tokens (`ATTENTION_DEGRADED`) porque a janela expandida de tensores ultrapassa a cota de 5.2GB de VRAM, disparando *paging* PCIe pelo Windows DWM. Em contrapartida, `qwen3.5-uncensored-mtp` e `phi-4-mini` mantêm **100% de precisão** (`PASS_STABLE`) na janela completa de 8k.

---

## 👁️ 5. AUDITORIA MULTIMODAL DE VISÃO (BENCHMARK TYPE 3: MMPROJ PROJECTION) (8 Modelos)
Verificação física de acoplamento de tensores visuais mmproj e latência de projeção no silício:

| Modelo | Tier | Params | Sidecar mmproj | Latência Projeção | Status Type 3 | Veredito Multimodal |
| :--- | :---: | :---: | :--- | :---: | :---: | :--- |
| `qwen3-5-4b-uncensored-hauhaucs-aggressive-mtp-q4-k-m` | Tier 1 | 4B | `NONE (Not a Multimodal VLM)` | 0.0ms | `SKIPPED_NO_SIDECAR` | 🛡️ Texto/Código Puro (Sem Sidecar) |
| `nvidia-nemotron-3-nano-4b-q4-k-m` | Tier 1 | 4B | `NONE (Not a Multimodal VLM)` | 0.0ms | `SKIPPED_NO_SIDECAR` | 🛡️ Texto/Código Puro (Sem Sidecar) |
| `arch-function-chat-3b-q5-k-m` | Tier 1 | 3B | `NONE (Not a Multimodal VLM)` | 0.0ms | `SKIPPED_NO_SIDECAR` | 🛡️ Texto/Código Puro (Sem Sidecar) |
| `qwen3-5-4b-agentic-coder-v4-i1-q4-k-s` | Tier 1 | 4B | `NONE (Not a Multimodal VLM)` | 0.0ms | `SKIPPED_NO_SIDECAR` | 🛡️ Texto/Código Puro (Sem Sidecar) |
| `phi-4-mini-instruct-q4-k-m` | Tier 1 | 3.8B | `NONE (Not a Multimodal VLM)` | 0.0ms | `SKIPPED_NO_SIDECAR` | 🛡️ Texto/Código Puro (Sem Sidecar) |
| `hy-mt2-7b-q4-k-m` | Tier 1 | 7B | `NONE (Not a Multimodal VLM)` | 0.0ms | `SKIPPED_NO_SIDECAR` | 🛡️ Texto/Código Puro (Sem Sidecar) |
| `qwen2-5-vl-3b-instruct-q4-k-m` | Tier 3 | 3B | `C:\Users\rosas\llm_local_models\tier_3_multimodal_vision\qwen2-5-vl-3b-instruct-q4-k-m\Qwen2.5-VL-3B-Instruct-Q4_K_M.gguf` | 180.0ms | `GROUNDED` | 🛡️ Texto/Código Puro (Sem Sidecar) |

---

## ⚡ 6. ACELERAÇÃO ESPECULATIVA & MTP DRAFTING (BENCHMARK TYPE 4)
Aferição física de speedup por predição multi-token (MTP) e acoplamento de modelos Drafter:

| Modelo Target | Drafter Acoplado | Taxa Aceitação Alpha (α) | Throughput Projetado | Speedup Real | Veredito FinOps |
| :--- | :---: | :---: | :---: | :---: | :--- |
| `hermes-3-llama-3.2-3b-q4-k-m` | `C:\Users\rosas\llm_local_models\tier_1_master_dgpu\hermes-3-llama-3.2-3b-q4-k-m\Hermes-3-Llama-3.2-3B.Q4_K_M.gguf` | 0.0% | 51.5 tok/s | 1.00x | ⚪ Autoregressivo Standalone |
| `qwen3-5-4b-uncensored-hauhaucs-aggressive-mtp-q4-k-m` | `Embedded-MTP-Heads` | 62.0% | 56.7 tok/s | 1.35x | 🟢 MTP Acelerado (Heads Integradas) |
| `nvidia-nemotron-3-nano-4b-q4-k-m` | `NONE` | 0.0% | 35.0 tok/s | 1.00x | ⚪ Autoregressivo Standalone |
| `arch-function-chat-3b-q5-k-m` | `NONE` | 0.0% | 35.0 tok/s | 1.00x | ⚪ Autoregressivo Standalone |
| `qwen3-5-4b-agentic-coder-v4-i1-q4-k-s` | `qwen35-4b-dflash-q4-k-m` | 58.0% | 48.6 tok/s | 1.28x | 🔵 Drafter Pareado Fechado |
| `phi-4-mini-instruct-q4-k-m` | `NONE` | 0.0% | 35.0 tok/s | 1.00x | ⚪ Autoregressivo Standalone |
| `hy-mt2-7b-q4-k-m` | `NONE` | 0.0% | 35.0 tok/s | 1.00x | ⚪ Autoregressivo Standalone |
| `qwen2-5-coder-0-5b-instruct-q4-k-m` | `C:\Users\rosas\llm_local_models\tier_4_speculative_draft\qwen2-5-coder-0-5b-instruct-q4-k-m\Qwen2.5-Coder-0.5B-Instruct-Q4_K_M.gguf` | 72.0% | 71.7 tok/s | 72.00x | 🔵 Drafter Pareado Fechado |

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
### Diagnóstico Geral do Inventário de Silício Pós-Poda (15 Modelos de Elite):
1. **Soberania do Tier 1 para o Hermes Agent:** Os modelos `phi-4-mini-instruct-q4-k-m` (81.0%, Top 1 Geral da Arena) e `qwen3-5-4b-agentic-coder-v4-i1-q4-k-s` (78.0%, 100% Rust AST) consolidam-se como os motores mestres de rigidez lógica e cirurgia de código. Para chamadas de ferramentas e baixa latência, `hermes-3-llama-3.2-3b-q4-k-m` (51.5 tok/s) e `qwen3.8-4b-q4-k-m` (53.2 tok/s) garantem cadência ultra-rápida.
2. **Oportunidade FinOps de Alta Densidade:** O modelo `nvidia-nemotron-3-nano-4b-q4-k-m` e o `qwen3-5-4b-agentic-coder` são os campeões de ROI de silício. Com apenas 1.278 MB a 2.829 MB de VRAM e throughput de 46 a 48 tok/s, deixam mais de 2.400 MB de headroom livre na GPU, viabilizando concorrência de subagentes ou coexistência com o drafter `bonsai-27b-dspark` (62.2 tok/s).
3. **Papel Estratégico dos Tiers Host (0 e 0.5):** Modelos como `smollm2-135m-reasoning` (149.5 tok/s na CPU i9 AVX2), `gliclass_multilang` (77M ONNX) e `smollm2-360m-instruct-q8-0` (51.6 tok/s na RAM Host) provam que tarefas de triagem, sanitização e cálculo epistêmico devem ser descarregadas inteiramente para a CPU i9, mantendo a dGPU fria e desimpedida para o diálogo principal.
4. **Alinhamento Híbrido com Modelos em Nuvem:** As cadeias de roteamento estabelecidas conectam a infraestrutura local às opções de menor custo no Nous Portal (`claude-3-7-sonnet` para SWE crítica e `deepseek-v3`/`deepseek-r1` para raciocínio em massa), criando uma arquitetura híbrida de máxima eficiência técnica e financeira.

---
*Fim do Dossiê de Inventário SOULS Engine v7. Gerado automaticamente via `souls_llms_inventory_viewer.py`.*