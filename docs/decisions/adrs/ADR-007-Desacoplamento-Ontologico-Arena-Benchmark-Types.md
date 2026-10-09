# ADR-007 — Desacoplamento Ontológico da Arena de Modelos Locais: Segregação entre ModelTier, InferenceEngine e BenchmarkType

> **ESTATUS DO DOCUMENTO:** APROVADO E CANÔNICO (DOCS-AS-GUARDRAILS)  
> **DATA DE RATIFICAÇÃO:** Outubro de 2026 (Linha de Base v7 Canônica)  
> **CRATES DIRETAMENTE REGIDAS:** `souls_llm_local_arena`, `souls_inference_runtime`, `souls_protocol`, `souls_model_router`.  
> **APLICAÇÃO:** Todos os Agentes de IA (Cursor, Windsurf, Claude Code, Trae) e Engenheiros de Sistemas.

---

## 1. CONTEXTO E FORÇAS EM CONFLITO

Nas iterações experimentais do Souls Engine (v5 e v6) presentes no acervo legado em `_donor/souls_mc_core/src/bin/souls_arena_cli.rs`, o motor de profiling e benchmark bare-metal foi concebido para avaliar modelos locais em múltiplas frentes de silício e cognição.

Contudo, a evolução orgânica do código produziu uma **colisão terminológica e conceitual crítica**: o termo **"Tier"** passou a ser utilizado simultaneamente para dois domínios ontológicos completamente distintos:

1. **Classificação Física do Modelo no Hardware:**  
   Definia o porte de memória e o papel do modelo no sistema operacional:
   - `Tier 0` (Bootstrap & Sanidade em CPU AVX2, ex: GLiClass, ModernBERT).
   - `Tier 0.5` (Sensor Epistêmico em CPU AVX2, ex: Gemma 2B, SmolLM).
   - `Tier 1` (Live Chat & Master na dGPU RTX 2060m, ex: Qwen 3.5 4B).
   - `Tier 2` (Background Agent & MoE em RAM Host, ex: Bonsai 27B).
   - `Tier 3` (Vision Multimodal com projetor `mmproj`).
   - `Tier 4` (Speculative Drafters, ex: DFlash, DSpark).

2. **Modalidade ou Prova de Benchmark:**  
   O runner e a CLI (`souls_arena_cli`) chamavam as rotinas de teste de:
   - *"Tier 1: Profile"* (medição de VRAM, TTFT, TPOT e TPS).
   - *"Tier 2: Eval"* (avaliação de código Rust e CoT reasoning).
   - *"Tier 3: Vision"* (combate multimodal).
   - *"Tier 4: Speculative"* (combate de rascunho e taxa $\alpha$).
   - *"Tier 5: Pressure"* (sonda Needle-in-a-Haystack de 4k a 32k).

### Forças de Atrito Resultantes:
- **Sobrecarga Semântica e Confusão Cognitiva:** Diálogos entre desenvolvedores e comandos passados a agentes de IA geravam ambiguidade imediata (ex: *"executar o Tier 1"* significava testar o modelo principal ou rodar o teste de hardware?).
- **Acoplamento Rígido entre Modelo e Motor:** A arquitetura legada presumia que cada modelo pertencia a um motor de inferência estático, ignorando que o mesmo arquivo de pesos GGUF pode ser executado em motores distintos (ex: `LlamaCpp(CPU)` para sondagem rápida de logits com 0 MB de VRAM vs. `LlamaCpp(CUDA)` para geração interativa de código na dGPU).
- **Inviabilidade de Reutilização Direta:** O monólito de 2.265 linhas em `_donor/` misturava chamadas de subprocessos, parsing de logs, manipulação direta de SQLite e formatação de texto em um único arquivo, impedindo seu uso limpo como biblioteca dentro da crate canônica `souls_llm_local_arena`.

---

## 2. DECISÃO ARQUITETURAL

Fica formalmente decretado o **Desacoplamento Ontológico das Três Dimensões** no subsistema de avaliação de modelos do Souls Engine v7:

```
┌──────────────────────────────────┐     ┌──────────────────────────────────┐     ┌──────────────────────────────────┐
│           DIMENSÃO 1             │     │           DIMENSÃO 2             │     │           DIMENSÃO 3             │
│           ModelTier              │  X  │        InferenceEngine           │  X  │         BenchmarkType            │
│       (Papel no Silício)         │     │      (Motor de Execução)         │     │      (Bateria de Prova)          │
└──────────────────────────────────┘     └──────────────────────────────────┘     └──────────────────────────────────┘
```

### 2.1 Monopólio do Termo `ModelTier`
O termo **Tier** passa a ser monopólio exclusivo da **Dimensão 1**, classificando o papel arquitetural e a alocação de hardware do modelo (Tiers 0, 0.5, 1, 2, 3 e 4), conforme estipulado na Seção 2 do manifesto canônico.

### 2.2 Estabelecimento dos `BenchmarkTypes`
As modalidades de ensaio na Arena passam a ser denominadas compulsoriamente como **`BenchmarkType`** (ou Baterias Canônicas de Prova):
- **`Type 1` — `HardwareProfile`:** Aferição físico-quantitativa pura ($TTFT$, $TPOT$, $TPS$, VRAM de pico monitorada via NVML a cada 50 ms, Host RAM e KV Cache).
- **`Type 2` — `CognitiveEval`:** Validação qualitativa e rigidez estrutural em três trilhas:
  - *Trilha 2A (`RustAstSyntax`):* Geração sintática estrita com validação Tree-sitter.
  - *Trilha 2B (`StructuredJson`):* Conformidade de chamadas MCP e tolerância de Response Healing.
  - *Trilha 2C (`CotEfficiency`):* Razão de tokens de pensamento `<think>` e cálculo da Métrica $E^3$.
- **`Type 3` — `MultimodalVision`:** Pareamento com adaptadores `mmproj` e latência de projeção VQA.
- **`Type 4` — `SpeculativeDraft`:** Avaliação de speedup e taxa de aceitação $\alpha$ de drafters emparelhados.
- **`Type 5` — `ContextPressure`:** Sonda *Needle-in-a-Haystack* estressando janelas de 4k a 32k tokens contra colapso de atenção.

### 2.3 Suporte Nativo a Comparações Multimotor
A Arena adota a tupla formal:
$$\text{Ensaio} = \langle \text{ModelDescriptor}, \text{InferenceEngine}, \text{BenchmarkType}, \text{ExecutionParameters} \rangle$$
permitindo que o mesmo modelo seja testado em múltiplos motores (ex: `LlamaCpp(CUDA)` vs `LlamaCpp(CPU)` vs `OnnxRuntime`) para comparar trade-offs de silício e consumo de memória.

### 2.4 Ciclo de Vida em Duas Fases: Standalone ➔ Integrado
A crate `crates/souls_llm_local_arena` é desenhada com arquitetura dual:
1. **Fase Standalone:** Expõe o binário executável `src/bin/souls_arena.rs` na CLI do terminal (`cargo run -p souls_llm_local_arena --bin souls_arena -- <comando>`), permitindo qualificar imediatamente os modelos locais baixados no disco (`C:\Users\rosas\.lmstudio\models\` e `.souls_data\models\`).
2. **Fase Integrada:** Expõe sua API em `src/lib.rs` para o `souls_server` (rotas REST e garras MCP `ops.arena_benchmark` / `ops.model_leaderboard`) e alimenta diretamente os priors bayesianos ($\alpha$ e $\beta$) do `ParetoBanditRouter`.

---

## 3. CONSEQUÊNCIAS TÉCNICAS E ANÁLISE DE IMPACTO

### 3.1 Consequências Positivas
- **Erradicação Completa da Ambiguidade Semântica:** A distinção clara entre `--tier` (do modelo) e `--type` (do teste) impede erros de comando e alucinações de agentes de IA.
- **Modularização Cirúrgica de Código:** O monólito de 2.200 linhas é decomposto em subpastas com responsabilidade única (`domain/`, `discovery/`, `batteries/`, `harness/`, `reporters/`), facilitando a manutenção e testes unitários atômicos.
- **Consistência de Dados no SQLite:** Os resultados alimentam a tabela `model_registry` no `souls_heuristic_vault.db` com esquemas rígidos e compatíveis com a esteira FinOps da v7.
- **Conformidade Termodinâmica (ADR-004):** A execução do `Type 1` ancora-se no `HardwareWatchdog` do `souls_inference_runtime`, garantindo aborto imediato antes de atingir $5.294\text{ MB}$ de VRAM ou $82^\circ\text{C}$ de temperatura.

### 3.2 Desvantagens e Mitigações
- **Necessidade de Refatorar os Callers da CLI Legada:** Scripts antigos que passavam argumentos como `--tier 1` referindo-se a profiling deverão utilizar a sintaxe canônica `--type 1`.
  - *Mitigação:* O analisador de argumentos da nova CLI `souls_arena` implementa compatibilidade transparente com aliases legados (`--mode profile` / `--mode eval`).

---

## 4. LINHAS VERMELHAS PARA AGENTES DE IA

1. **PROIBIDO** utilizar a palavra "Tier" para descrever tipos ou modos de teste em código, documentação ou logs. O termo é reservado exclusivamente para `ModelTier`.
2. **PROIBIDO** acoplar os algoritmos de benchmark diretamente a um único backend de inferência. A camada de avaliação deve consumir a trait genérica `ArenaInferEngine`.
3. **PROIBIDO** ignorar as barreiras de VRAM ($\le 5.294\text{ MB}$) ou temperatura ($\le 82^\circ\text{C}$) durante as baterias de estresse.

---

*Registrado e arquivado na documentação canônica de governança do Souls Engine v7.*
