# ARENA_BENCHMARK_SPECIFICATION.md — Especificação Canônica da Arena de Avaliação e Benchmarking de Silício Local (v7)

> **ESTATUS DO DOCUMENTO:** CANÔNICO E OBRIGATÓRIO (DOCS-AS-GUARDRAILS)  
> **APLICAÇÃO:** Engenheiros de Sistemas, Agentes de IA Analistas e Implementadores.  
> **CRATES DIRETAMENTE REGIDAS:** `souls_llm_local_arena`, `souls_inference_runtime`, `souls_protocol`, `souls_model_router`.  
> **PROPÓSITO:** Fixar a taxonomia conceitual definitiva, o catálogo de baterias de testes, as fórmulas de julgamento e a arquitetura modular da Arena de Modelos Locais, erradicando qualquer colisão terminológica entre porte físico de modelos e tipos de benchmark.

---

## 1. PREÂMBULO E A ONTOLOGIA DAS TRÊS DIMENSÕES ORTOGONAIS (ADR-007)

Na base legada (v5/v6), o termo *"Tier"* era sobrecarregado simultaneamente para designar o porte de memória de um modelo e o tipo de teste executado, induzindo agentes e desenvolvedores a confusões crônicas de escopo.

A partir da v7 Canônica, o domínio de benchmarking e avaliação de silício local é governado por **três dimensões estritamente ortogonais e independentes**:

```
┌──────────────────────────────────┐     ┌──────────────────────────────────┐     ┌──────────────────────────────────┐
│           DIMENSÃO 1             │     │           DIMENSÃO 2             │     │           DIMENSÃO 3             │
│           ModelTier              │  X  │        InferenceEngine           │  X  │         BenchmarkType            │
│       (Papel no Silício)         │     │      (Motor de Execução)         │     │      (Bateria de Prova)          │
└──────────────────────────────────┘     └──────────────────────────────────┘     └──────────────────────────────────┘
```

Qualquer ensaio na Arena avalia compulsoriamente a tupla:
$$\text{Ensaio} = \langle \text{ModelDescriptor}, \text{InferenceEngine}, \text{BenchmarkType}, \text{ExecutionParameters} \rangle$$

---

## 2. DIMENSÃO 1: `ModelTier` (O Porte Físico e Papel no Ecossistema)

A palavra **Tier** é de monopólio exclusivo da classificação física do modelo em relação ao hardware bare-metal hospedeiro (Intel Core i9-9880H AVX2, 32GB RAM DDR4 e dGPU NVIDIA RTX 2060m com teto útil de 5.294 MB sob WDDM 3.x):

| `ModelTier` | Faixa Típica de Parâmetros | Hardware Alvo | VRAM Consumida | Papel no Souls Engine | Modelos de Referência Típicos |
|---|:---:|---|:---:|---|---|
| **`Tier 0` (Bootstrap & Sanidade)** | $10\text{M} \dots 500\text{M}$ | CPU Intel i9 (AVX2) | **0 MB** | Embeddings rápidos, classificação de intenção, triagem de segurança. | `GLiClass Multilang Ultra`, `ModernBERT` |
| **`Tier 0.5` (Sensor Epistêmico)** | $500\text{M} \dots 2.5\text{B}$ | CPU Intel i9 (AVX2) | **0 MB** | *Logit probing*, medição de entropia de Shannon, freio de ambiguidade socrático. | `SmolLM 135M/360M`, `Gemma 2B`, `Qwen2.5 0.5B/1.5B` |
| **`Tier 1` (Master Bare-Metal)** | $2.5\text{B} \dots 8\text{B}$ | dGPU RTX 2060m (CUDA) | $\le 5.294\text{ MB}$ | Modelo de raciocínio principal, chat interativo, geração de código com AST. | `Qwen 3.5 4B Coder`, `Mamba-Codestral 7B`, `Phi-4-mini` |
| **`Tier 2` (Background & MoE)** | $14\text{B} \dots 33\text{B}+$ | Offload Híbrido (RAM Host) | Offload parcial | Subagentes autônomos em segundo plano, refatorações maciças, MoE. | `Bonsai 27B`, `Mixtral-8x7B`, `Qwen 14B` |
| **`Tier 3` (Vision & VLM)** | $2\text{B} \dots 8\text{B} + \text{mmproj}$ | dGPU / CPU Híbrido | $\le 5.2\text{ GB}$ | Leitura de capturas de tela, inspeção de UI e raciocínio visual (VQA). | `Qwen2-VL 2B/7B`, `MiniCPM-V` |
| **`Tier 4` (Speculative Drafters)** | $100\text{M} \dots 1.5\text{B}$ | dGPU / CPU | Residual | Modelos ultra-rápidos de rascunho especulativo emparelhados com Tiers 1 ou 2. | `DFlash`, `DSpark`, `Qwen-Draft` |

---

## 3. DIMENSÃO 2: `InferenceEngine` (A Matriz Multimotor)

O Souls Engine não amarra um modelo a um único motor. O mesmo arquivo de pesos (ex: `.gguf`) pode ser testado em diferentes motores para finalidades de desempenho ou isolamento distintas:

```
                  ┌──► LlamaCppEngine (CUDA sm_75) ──► Geração Interativa na dGPU (KV assimétrico)
                  │
[Modelo GGUF] ────┼──► LlamaCppEngine (CPU AVX2)  ──► Probing de Logits e Embeddings (0 MB VRAM)
                  │
                  └──► MistralRsEngine            ──► Testes de Quantização Especializada e MoE
```

### Catálogo de Motores Homologados

1. **`LlamaCppEngine(Backend::Cuda)`:**
   - Compilação nativa sobre drivers CUDA da RTX 2060m (`sm_75`).
   - Recursos: Asymmetric KV Cache (Key: FP16/Q8_0, Value: Q4_0), FlashAttention-2 (`-fa`), governança de VRAM via `HardwareWatchdog`.
2. **`LlamaCppEngine(Backend::Cpu)`:**
   - Compilação nativa para CPU x86_64 utilizando instruções vetoriais AVX2 da CPU Intel i9.
   - Recursos: 0 MB de VRAM consumida, ideal para sondagem de probabilidades em lote.
3. **`OnnxRuntimeEngine(CpuAvx2)`:**
   - Driver oficial ONNX Runtime (`ort`) em CPU AVX2 para modelos de classificação estruturada e encoders.
4. **`MistralRsEngine`:**
   - Suporte para backends Rust puros, quantizações especiais e arquiteturas específicas.
5. **`MockEngine`:**
   - Motor sintético em memória para testes unitários automatizados da esteira de benchmark.

---

## 4. DIMENSÃO 3: `BenchmarkType` (As 5 Baterias de Avaliação)

Substitui-se a denominação *"Tier de Benchmark"* pela nomenclatura padronizada **`BenchmarkType`** (ou *Tracks* de Avaliação):

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                      MATRIZ DE BENCHMARK TYPES (ARENA)                      │
├─────────────────────────────────────────────────────────────────────────────┤
│ Type 1: HardwareProfile   ──► Quantitativo Puro (VRAM, RAM, TTFT, TPOT, TPS)│
│ Type 2: CognitiveEval     ──► Qualitativo & Rigidez (Rust AST, CoT, JSON)   │
│ Type 3: MultimodalVision  ──► Pareamento Visual (Projetor mmproj + VQA)     │
│ Type 4: SpeculativeDraft  ──► Eficiência Especulativa (Taxa Alpha & Speedup)│
│ Type 5: ContextPressure   ──► Resistência a Contexto Longo (Needle 4k-32k)  │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

### 4.1 Type 1 — `HardwareProfile` (Aferição Físico-Quantitativa)

- **Objetivo Primário:** Medir o custo físico de execução no silício real da máquina sem viés cognitivo.
- **Métricas Registradas:**
  - **Cold Load Time ($T_{\text{cold}}$, ms):** Tempo de mapeamento de memória do arquivo GGUF via `memmap2` e alocação de tensores.
  - **Peak VRAM ($V_{\text{peak}}$, MB):** Medição contínua via NVML a cada 50 ms. Se ultrapassar $5.294\text{ MB}$, o teste registra alerta de risco de spillover.
  - **Host RAM ($R_{\text{host}}$, MB):** Pegada de memória física consumida pelo processo na RAM do Windows 11.
  - **Time-to-First-Token ($TTFT$, ms):** Latência de prefill da janela de prompt inicial.
  - **Time-per-Output-Token ($TPOT$, ms):** Latência média por token subsequente no loop autorregressivo.
  - **Sustained Throughput ($TPS$, tokens/s):** Velocidade real sustentada calculada como:
    $$TPS = \frac{N_{\text{completion}}}{T_{\text{generation\_seconds}}}$$
  - **KV Cache Footprint ($M_{\text{kv}}$, MB):** Memória alocada para o contexto configurado.

---

### 4.2 Type 2 — `CognitiveEval` (Validação Qualitativa e Rigidez Estrutural)

- **Objetivo Primário:** Determinar a confiabilidade epistêmica do modelo para atuar como engenheiro de código sem alucinações sintáticas.
- **Composto por Três Sub-Trilhas Canônicas:**

#### Trilha 2A: `RustAstSyntax` (Rigidez Sintática de Código)
- O modelo recebe desafios de geração de structs, enums, lifetimes e traits em Rust.
- A resposta é parseada cirurgicamente via Tree-sitter em memória (`souls_ast`).
- **Critério de Julgamento:**
  - Código compila sintaticamente e fecha chaves/parênteses com $\text{TreeError} = 0$: **QUALIFIED**.
  - Código truncado ou com chaves desconexas: **REJECTED**.

#### Trilha 2B: `StructuredJson` (Conformidade com Esquemas MCP)
- O modelo é provocado a invocar ferramentas MCP emitindo payloads JSON estritos sob um JSON Schema determinado.
- **Critério de Julgamento:**
  - JSON válido nativamente sem reparos: **Nota 5.0 (Strict Pass)**.
  - JSON truncado mas curado pelo `heal_json_response`: **Nota 3.5 (Healed Pass)**.
  - JSON estruturalmente irrecuperável: **Nota 0.0 (Fail)**.

#### Trilha 2C: `CotEfficiency` (Eficiência do Raciocínio Socrático)
- Avaliação de modelos que operam com tags de raciocínio `<think> ... </think>`.
- **Métricas:**
  - Contagem de Tokens de Pensamento ($N_{\text{think}}$).
  - Contagem de Tokens Úteis de Resposta ($N_{\text{answer}}$).
  - **Razão CoT ($R_{\text{cot}}$):**
    $$R_{\text{cot}} = \frac{N_{\text{think}}}{N_{\text{answer}}}$$
  - **Métrica $E^3$ (Eficácia, Economia, Eficiência):**
    $$E^3 = \frac{(\text{Acurácia})^2}{\text{Latência Total (segundos)} + 0.001} \times \frac{1}{1 + \ln(1 + R_{\text{cot}})}$$
    Modelos que queimam 2.000 tokens de raciocínio para responder uma query trivial sofrem penalidade FinOps severa no ranking.

---

### 4.3 Type 3 — `MultimodalVision` (Avaliação Multimodal VLM)

- **Objetivo Primário:** Validar modelos de visão emparelhando o modelo base com seu adaptador `mmproj` correspondente.
- **Métricas:**
  - Sucesso no carregamento de tensores do projetor CLIP/SigLIP.
  - Latência de projeção multimodal ($TTFT_{\text{vision}}$, ms).
  - Acurácia de extração de texto de capturas de tela e diagrama de arquitetura.

---

### 4.4 Type 4 — `SpeculativeDraft` (Aceleração Especulativa e MTP)

- **Objetivo Primário:** Descobrir se um modelo menor (Tier 4, ex: 0.5B) acelera a geração do modelo mestre (Tier 1, ex: 7B).
- **Métricas:**
  - **Taxa de Aceitação $\alpha$:** Proporção de tokens rascunhados que o modelo mestre validou sem descarte.
  - **Speedup Real ($S$):**
    $$S = \frac{TPS_{\text{speculative}}}{TPS_{\text{base}}}$$
  - **Regra FinOps:** Se $\alpha < 0.45$ ou $S < 1.15\times$, a Arena emite o veredito **`DESACELEROU (ALERTA)`**, desativando o modo especulativo para aquele par no `ParetoBanditRouter`.

---

### 4.5 Type 5 — `ContextPressure` (Sonda Anti-Colapso de Contexto)

- **Objetivo Primário:** Testar a robustez do mecanismo de atenção em janelas de contexto dilatadas.
- **Metodologia:** Injeção de fatos sintéticos críticos (*Needles*) em matrizes densas de código nos comprimentos de **4k, 8k, 16k e 32k tokens**.
- **Veredito:**
  - Recupera o fato sem alteração métrica: **`PASS (No Degradation)`**.
  - Falha na recuperação ou colapso de perplexidade: **`FAIL (Degradation Detected)`**. Registra o limite de contexto seguro no banco de heurísticas.

---

## 5. ARQUITETURA MODULAR DA CRATE `souls_llm_local_arena`

A implementação em Rust na crate canônica é decomposta em módulos especializados e isolados:

```
crates/souls_llm_local_arena/
├── Cargo.toml                          # Dependências do workspace v7
└── src/
    ├── lib.rs                          # Facade pública (ArenaRunner, BenchmarkSuite)
    ├── error.rs                        # ArenaError tipado com thiserror
    ├── domain/                         # Entidades puras do domínio de benchmark
    │   ├── mod.rs
    │   ├── model.rs                    # ModelDescriptor (path, tier, quant, context)
    │   ├── engine.rs                   # Trait ArenaInferEngine + Enum EngineType
    │   └── result.rs                   # DTOs de relatórios (ProfileReport, EvalReport, etc.)
    ├── discovery/                      # Scanner de modelos locais zero-copy
    │   ├── mod.rs
    │   ├── scanner.rs                  # Varredura em C:\Users\rosas\.lmstudio e .souls_data
    │   ├── metadata.rs                 # Inspeção GGUF O(1) via souls_inference_runtime::gguf_mmap
    │   └── vault.rs                    # Leitura e sincronização com souls_heuristic_vault.db
    ├── batteries/                      # Os 5 Tipos de Baterias Canônicas
    │   ├── mod.rs
    │   ├── type1_hardware.rs           # Type 1: HardwareProfile
    │   ├── type2_cognitive.rs          # Type 2: CognitiveEval (AST, CoT, JSON)
    │   ├── type3_vision.rs             # Type 3: MultimodalVision
    │   ├── type4_speculative.rs        # Type 4: SpeculativeDraft
    │   └── type5_pressure.rs           # Type 5: ContextPressure
    ├── harness/                        # Isolamento de execução e segurança
    │   ├── mod.rs
    │   ├── worker.rs                   # Dedicated OS Worker Thread com catch_unwind
    │   └── nvml_guard.rs               # Barreira térmica (>82°C) e alocação máxima de 5.2GB
    ├── reporters/                      # Apresentação visual de resultados
    │   ├── mod.rs
    │   ├── terminal.rs                 # Tabelas limpas para terminal Windows pwsh
    │   └── markdown.rs                 # Gerador de relatórios em docs/telemetry/reports/
    └── bin/
        └── souls_arena.rs              # Ponto de entrada CLI Standalone
```

---

## 6. INTERFACE DA CLI STANDALONE (`souls_arena`)

A CLI standalone expõe comandos claros, sem qualquer ambiguidade entre `--tier` (do modelo) e `--type` (do benchmark):

```powershell
# 1. Descoberta: Listar todos os modelos locais catalogados
cargo run -p souls_llm_local_arena --bin souls_arena -- list

# 2. Type 1: Profile rápido de hardware de um modelo específico
cargo run -p souls_llm_local_arena --bin souls_arena -- run --type 1 --model "Qwen3.5-4B"

# 3. Comparativo Multimotor: Comparar o mesmo modelo em CPU AVX2 vs GPU CUDA
cargo run -p souls_llm_local_arena --bin souls_arena -- run --type 1 --model "SmolLM3-3B" --engine llama-cuda
cargo run -p souls_llm_local_arena --bin souls_arena -- run --type 1 --model "SmolLM3-3B" --engine llama-cpu

# 4. Filtrar por ModelTier: Executar o Type 1 em todos os modelos de Tier 1 (dGPU)
cargo run -p souls_llm_local_arena --bin souls_arena -- run --type 1 --tier 1

# 5. Type 2: Avaliação Cognitiva E3 (Rust AST, CoT e JSON Tool Calling)
cargo run -p souls_llm_local_arena --bin souls_arena -- run --type 2 --model "Mamba-Codestral-7B"

# 6. Bateria Completa: Executar todos os Types aplicáveis e exportar dossiê Markdown
cargo run -p souls_llm_local_arena --bin souls_arena -- run --type all --model "Qwen3.5-4B" --output docs/telemetry/reports/qwen_report.md
```

---

## 7. ESQUEMA DE PERSISTÊNCIA NO FRANKENSQLITE (`souls_heuristic_vault.db`)

Todos os resultados gerados pela Arena são persistidos no banco de dados SQLite local, alimentando diretamente:
- A tabela `model_registry`: armazena metadados estruturais, caminhos de arquivo, $TTFT$, $TPOT$, $TPS$ e pontuações especializadas.
- A tabela `arena_telemetry`: histórico cronológico das rodadas de avaliação empírica para cálculo de tendências de aquecimento e consistência.

---

## 8. INTEGRAÇÃO SIMBIÓTICA COM O SOULS ENGINE

A crate `souls_llm_local_arena` opera em duas fases de ciclo de vida:
1. **Fase Standalone (Presente):** Execução direta via terminal para qualificar os modelos baixados e montar a matriz empírica de silício.
2. **Fase Integrada (Futuro Imediato):** 
   - O `souls_model_router` consome os resultados de $E^3$, $TTFT$ e $TPS$ gravados pela Arena para calibrar as distribuições Beta ($\alpha$ e $\beta$) do **ParetoBandit**.
   - O `souls_server` expõe a Arena através da rota REST `GET /v1/models/benchmark` e das garras MCP `ops.arena_benchmark` e `ops.model_leaderboard`.

---

*Especificação aprovada sob os guardrails constitucionais do Souls Engine v7.*
