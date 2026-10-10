# ADR-008 — Roteamento Multi-Perfil por Tarefa, Modelagem de Custo de Silício e Fronteira de Pareto Bare-Metal

> **ESTATUS DO DOCUMENTO:** APROVADO E CANÔNICO (DOCS-AS-GUARDRAILS)  
> **DATA DE RATIFICAÇÃO:** Outubro de 2026 (Linha de Base v7 Canônica)  
> **CRATES DIRETAMENTE REGIDAS:** `souls_model_router`, `souls_llm_local_arena`, `souls_protocol`, `souls_server`.  
> **ESPECIFICAÇÕES CORRELATAS:** `llm_benchmark_cost_matrix_crate_specification.md` (Item 5), `SCHEMAS_DDL_MANIFEST.md`.  
> **APLICAÇÃO:** Todos os Agentes de IA (Cursor, Windsurf, Claude Code, Trae) e Engenheiros de Sistemas.

---

## 1. CONTEXTO E FORÇAS EM CONFLITO

Nas iterações iniciais da Arena de Modelos Locais (v7), a classificação e prontidão de modelos eram avaliadas por meio de uma métrica escalar unidimensional: throughput de geração ($TPS$) combinado com o teto de VRAM segura ($5.2\text{GB}$ sob o WDDM do Windows 11).

Contudo, a evolução operacional do **Hermes Agent** revelou uma limitação arquitetural severa decorrente do uso de um leaderboard escalar único:

1. **Assimetria Funcional de Cargas de Trabalho:**  
   Um modelo com alto throughput bruto (ex.: `smollm2-135m` a $149.5\text{ tok/s}$) é excelente para triagem preliminar de prompts e classificação de intenção, mas é incapaz de gerar código Rust sintaticamente estrito com Grammar Masking. Em contrapartida, um modelo de ponta em engenharia de software (ex.: `qwen3.5-4b-super-coder` a $50.1\text{ tok/s}$) despende recursos desnecessários se for acionado para higienizar um texto simples.
2. **Desacoplamento Custo-Nuvem vs. Custo-Silício:**  
   Enquanto runtimes de nuvem precificam modelos em dólares por milhão de tokens (\$ / 1M), a operação bare-metal local incorre em **custos físicos de hardware**: ocupação de memória de vídeo na dGPU (concorrendo com o Desktop Window Manager do Windows 11), tempo até o primeiro token ($TTFT$) e latência intrínseca de decodificação.
3. **Ausência de Cadeias Determinísticas de Fallback:**  
   Quando um subagente de suporte falha ou quando a janela de contexto de um modelo local se esgota (conforme observado no decaimento de atenção do `mamba-codestral-7b` em $8\text{k}$ tokens), o sistema necessita de uma cadeia determinística e previsível de recuo: primeiro para outro modelo local não-dominado na Fronteira de Pareto e, em último caso, para o modelo em nuvem de menor custo homologado (via Nous Portal ou OpenRouter).

---

## 2. DECISÃO ARQUITETURAL

Fica formalmente estabelecida a **Modelagem de Roteamento Multi-Perfil por Tarefa e Fronteira de Pareto de Silício**, sincronizando o motor local bare-metal com os princípios da *LLM Benchmark Cost Matrix Crate*:

```
┌────────────────────────────────────────────────────────────────────────┐
│             SISTEMA DE ROTEAMENTO MULTI-PERFIL (SOULS ENGINE v7)       │
└──────────────────────────────────┬─────────────────────────────────────┘
                                   │
         ┌─────────────────────────┼─────────────────────────┐
         ▼                         ▼                         ▼
   🦀 Coding (SWE)           🛠️ Agentic Tools         🧠 Deep Reasoning
   S_coding: Rust AST        S_agentic: MCP JSON      S_reasoning: CoT & Logic
   + Ctx 8k Needle           + Ctx 8k Needle          + Ctx 8k Needle
         │                         │                         │
         └─────────────────────────┼─────────────────────────┘
                                   ▼
                           ⚡ Fast Streaming
                           S_fast: TPS + VRAM Livre + Low TTFT
```

### 2.1 Os 4 Perfis Canônicos de Carga de Trabalho

Cada perfil agrega dimensões cognitivas e físicas normalizadas ($S_{\text{norm}} \in [0, 100]$):

1. **Perfil A — Engenharia de Software & Coding Bare-Metal ($S_{\text{coding}}$):**
   $$S_{\text{coding}} = 0.40 \cdot S(\text{Rust AST}) + 0.25 \cdot S(\text{Ctx 8k Needle}) + 0.20 \cdot S_{\text{norm}}(\text{TPS}_{50}) + 0.15 \cdot S_{\text{norm}}(\text{TTFT})$$
2. **Perfil B — Agentes Autônomos & Ferramentas MCP ($S_{\text{agentic}}$):**
   $$S_{\text{agentic}} = 0.45 \cdot S(\text{MCP JSON Tools}) + 0.25 \cdot S(\text{Ctx 8k Needle}) + 0.15 \cdot S_{\text{norm}}(\text{TTFT}) + 0.15 \cdot S_{\text{norm}}(\text{TPS}_{50})$$
3. **Perfil C — Raciocínio Profundo & Arquitetura de Sistemas ($S_{\text{reasoning}}$):**
   $$S_{\text{reasoning}} = 0.45 \cdot S(\text{CoT Reasoning}) + 0.35 \cdot S(\text{Ctx 8k Needle}) + 0.20 \cdot S(\text{Rust AST})$$
4. **Perfil D — Triagem Heurística, Streaming Rápido & Probes de Host ($S_{\text{fast}}$):**
   $$S_{\text{fast}} = 0.45 \cdot S_{\text{norm}}(\text{TPS}_{150}) + 0.25 \cdot S_{\text{norm}}(\text{VRAM}_{\text{free}}) + 0.20 \cdot S_{\text{norm}}(\text{TTFT}) + 0.10 \cdot S(\text{SpeculativeBonus})$$

### 2.2 Função de Custo de Silício ($C_{\text{silicon}}$)

O custo físico do silício bare-metal sob a arquitetura RTX 2060m + Intel i9 é ponderado por:
$$C_{\text{silicon}} = 0.50 \cdot \left(\frac{\text{VRAM}_{\text{peak}}}{5294.0}\right) + 0.30 \cdot \left(\frac{\text{TTFT}_{\text{sec}}}{3.0}\right) + 0.20 \cdot \left(\frac{30.0}{\text{TPS} + 1.0}\right)$$
- Modelos executados puramente em CPU Host (Tiers 0 e 0.5) possuem $\text{VRAM}_{\text{peak}} \approx 0$, minimizando o custo para a dGPU.
- Modelos que se aproximam do teto útil WDDM ($5.294\text{ MB}$) sofrem penalidade de custo por risco de spillover PCIe.

### 2.3 Retorno de Silício Amortecido ($V_{\text{task}}$ — Índice de ROI)

Para mitigar a distorção onde SLMs ultraleves com acurácia insuficiente dominam as classificações devido à divisão por grandezas próximas de zero, aplica-se amortecimento logarítmico:
$$V_{\text{task}} = \frac{S_{\text{task}}}{\ln(1.0 + C_{\text{silicon}} + 1.0)}$$

### 2.4 Algoritmo da Fronteira de Pareto Bare-Metal

Um modelo $A$ domina um modelo $B$ em determinado perfil se:
1. $S_{\text{task}}(A) \ge S_{\text{task}}(B)$ e $C_{\text{silicon}}(A) \le C_{\text{silicon}}(B)$
2. Ao menos uma das desigualdades é estrita ($>$ ou $<$).

Modelos não dominados formam a **Fronteira de Pareto** e recebem as designações canônicas:
- **🏆 Campeão de Qualidade:** O modelo da fronteira com a maior nota $S_{\text{task}}$.
- **⚡ Campeão de Custo/Eficiência:** O modelo da fronteira com o maior ROI $V_{\text{task}}$.
- **🔗 Fallback Local #1 / #2:** Os modelos alternativos não dominados sequenciais.

### 2.5 Persistência Relacional SSOT (`model_task_profiles`)

Fica estipulado que todos os scores calculados, custos e designações de Pareto devem ser compulsoriamente persistidos no banco `souls_heuristic_vault.db` na tabela `model_task_profiles` e espelhados nas colunas analíticas de `model_registry`:
- Qualquer execução de benchmark via CLI (`souls_arena.exe`) recalcula os perfis de forma atômica no final da suíte.
- O visualizador de dossiê (`souls_llms_inventory_viewer.py`) consome e sincroniza os mesmos dados.

---

## 3. CADEIAS DE ROTEAMENTO DETERMINÍSTICAS (LOCAL $\to$ CLOUD)

Para integração simbiótica com o **Hermes Agent**, cada perfil define formalmente sua cadeia de recuo:

| Perfil | Primário Local (dGPU) | Fallback Local #1 | Fallback Local #2 / FinOps | Fallback Cloud #1 (Nous Portal) | Fallback Cloud #2 (OpenRouter) |
| :--- | :--- | :--- | :--- | :--- | :--- |
| **Coding** | `qwen3.5-4b-super-coder` | `qwen3.5-4b-uncensored-mtp` | `nvidia-nemotron-3-nano` | `claude-3-7-sonnet` ($3.40/1M) | `deepseek-v3` / `qwen-2.5-coder-32b` |
| **Agentic Tools** | `qwen3.5-4b-super-coder` | `qwen3.5-4b-uncensored-mtp` | `qwen3.5-agentic-coder-v4` | `claude-3-7-sonnet` ($2.60/1M) | `gpt-4o-mini` / `deepseek-v3` |
| **Reasoning** | `nvidia-nemotron-3-nano` | `qwen3.5-4b-super-coder` | `laguna-xs-16b MoE` (Host RAM) | `deepseek-r1` ($0.55/1M) | `o3-mini` / `claude-3-7-sonnet` |
| **Fast Streaming** | `smollm2-135m-reasoning` (CPU) | `smollm-135m-instruct` (CPU) | `bitnet-b1.58-2b-4t` (Sensor CPU) | `gpt-4o-mini` ($0.14/1M) | `deepseek-v3` ($0.28/1M) |

---

## 4. CONSEQUÊNCIAS E GUARDRAILS

### 4.1 Efeitos Positivos
- **Erradicação do Vibe Routing:** Agentes de IA e subagentes não escolhem modelos com base em suposições; o despacho é regido pela Fronteira de Pareto.
- **Headroom Térmico e de VRAM Preservado:** Tarefas de triagem e sanitização operam em CPU Host (Tier 0/0.5), mantendo a dGPU RTX 2060m fria e com 4GB+ de VRAM livre para o diálogo principal do Hermes Agent.
- **Conformidade FinOps Total:** Se a carga local exceder limites físicos de janela de contexto ou capacidade de raciocínio, o desvio em nuvem recorre aos menores custos blended catalogados.

### 4.2 Linhas Vermelhas (Guardrails Inegociáveis)
1. **PROIBIDO Chaveamento no Chat Mestre:** O ParetoBandit e a tabela de perfis operam **exclusivamente** para despacho de subagentes (`delegate_task`) e tarefas auxiliares (`auxiliary.*`). O modelo interativo do chat principal do usuário é fixo.
2. **PROIBIDO Uso de Dados Sintéticos/Mock:** Toda métrica presente em `model_task_profiles` deve originar-se de execuções físicas registradas em `arena_telemetry`.
3. **PROIBIDO Modificar Pesos sem Atualização do ADR:** Os coeficientes de cada perfil ($w_1, w_2, w_3, w_4$) são canônicos e só podem ser alterados mediante aditivo formal neste documento.
