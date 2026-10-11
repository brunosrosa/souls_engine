# 🏟️ `souls_llm_local_arena` — Coliseu Bare-Metal de Silício & Benchmark Local (v7)

> **Crate Oficial do Souls Engine v7**  
> **Localização:** `crates/souls_llm_local_arena`  
> **Governança:** ADR-004 (Teto WDDM 5.2GB), ADR-007 (Desacoplamento Ontológico) e ADR-008 (Scoring Multi-Perfil e Fronteira de Pareto)  
> **Segurança:** `#![forbid(unsafe_code)]` padrão MSVC nativo  

---

## 📌 1. Visão Geral & Propósito

A crate `souls_llm_local_arena` é o motor bare-metal de **descoberta física, profiling de hardware, auditoria cognitiva rigorosa e governança de modelos de linguagem locais (LLMs/SLMs)** do Souls Engine.

Operando como o alicerce empírico para o orquestrador **Hermes Agent** (Python 3.11), esta crate elimina completamente o *"vibe coding"* e avaliações baseadas em suposições sintéticas:
- **Zero Mocks em Produção:** Métricas de latência e throughput são colhidas diretamente no silício (NVIDIA GeForce RTX 2060m e CPU Intel Core i9 AVX2).
- **Decapitação Estrita de Prompt:** Elimina 100% de falsos positivos gerados por eco de CLI no motor llama.cpp.
- **Persistência SSOT:** Gravação estrita no banco relacional SQLite STRICT WAL ([`souls_heuristic_vault.db`](file:///z:/souls_engine/.souls_data/db/souls_heuristic_vault.db)).
- **Zero Duplicação com o Hermes:** Sincronização bidirecional de modelos via **Hard Links NTFS/ReFS de 0 bytes** para `%LOCALAPPDATA%\hermes\models`.

---

## 🏛️ 2. Taxonomia Canônica dos Tiers Físicos de Silício

A governança do Souls Engine divide os modelos estritamente pela **camada física de execução no host** (conforme estabelecido em ADR-007):

| Tier Canônico | Dispositivo / Alvo | Restrições Físicas & VRAM | Papel no Souls Engine & Hermes Agent | Motores Suportados |
| :--- | :--- | :--- | :--- | :--- |
| **Tier 0**<br>*(Bootstrap & Sanity)* | **CPU Intel Core i9** (extensões AVX2) | **0 MB VRAM dGPU**<br>Modelos < 350 MB | Triagem de intenção instantânea (<15ms), sanitização de inputs a custo zero de GPU. | `onnx-cpu`<br>`llama-cpu` (`-ngl 0 -t 8`) |
| **Tier 0.5**<br>*(Epistemic Probing)* | **CPU Intel Core i9** (extensões AVX2) | **0 MB VRAM dGPU**<br>Modelos SLM 360M - 2B | Sensor epistêmico de incerteza (entropia de Shannon), extração direta de logits (forward prefill). | `llama-cpu`<br>BitNet 1.58b |
| **Tier 1**<br>*(Master & Subagentes)* | **dGPU RTX 2060m** (GDDR6 dedicado) | **Teto útil 5.2GB WDDM**<br>Modelos de 1B a 7B | Cérebro executivo local para Live Chat, geração de código Rust AST e despacho estruturado MCP. | `llama-cuda`<br>(`-ctk f16 -ctv q4_0 -fa on`) |
| **Tier 2**<br>*(Heavy MoE Offload)* | **Host DDR4 RAM** (Offload Pinned) | **0 MB VRAM dGPU**<br>Modelos >14B / MoE 16B+ | Raciocínio pesado assíncrono em background (`delegate_task`). Evita spillover PCIe na GPU. | `llama-cpu` (`-ngl 0 -t 8`) |
| **Tier 3**<br>*(Multimodal Vision)* | **dGPU RTX 2060m** com mmproj sidecar | **VLM 2B a 4B**<br>Modelos com projetor pareado | Inspeção visual de interfaces (UI Grounding), OCR de diagramas técnicos e captura multimodal. | `llama-cuda`<br>(`--mmproj <sidecar>`) |
| **Tier 4**<br>*(Speculative Drafters)* | **dGPU RTX 2060m** ou Host RAM | **Modelos compactos**<br>300M a 1.6GB ou MTP | Aceleração especulativa pareada com modelos mestres via taxa de aceitação de tokens ($\alpha \ge 55\%$). | `llama-cuda`<br>(`-md <drafter.gguf>`) |

---

## ⚡ 3. As 5 Baterias de Benchmark Canônicas (Types 1 a 5)

Cada modelo descoberto pode ser submetido a baterias de testes mutuamente exclusivas e ortogonais:

```
                                  ┌─────────────────────────────┐
                                  │      MODELO NO DISCO        │
                                  └──────────────┬──────────────┘
                                                 │
            ┌────────────────────┬───────────────┼───────────────┬───────────────────┐
            ▼                    ▼               ▼               ▼                   ▼
    ┌───────────────┐    ┌───────────────┐┌──────────────┐┌───────────────┐   ┌───────────────┐
    │    TYPE 1     │    │    TYPE 2     ││    TYPE 3    ││    TYPE 4     │   │    TYPE 5     │
    │HardwareProfile│    │ CognitiveEval ││ MultimodalVLM││SpeculativeDrft│   │ContextPressure│
    └───────┬───────┘    └───────┬───────┘└──────┬───────┘└───────┬───────┘   └───────┬───────┘
            │                    │               │                │                   │
            ▼                    ▼               ▼                ▼                   ▼
      TTFT, TPOT, TPS     10 Trilhas SWE     mmproj VQA     Taxa Aceitação α    Needle Haystack 8k
      e VRAM Peak MB     (Decapitação)     (UI Grounding)   (Speedup Real)      (Resistência WDDM)
```

1. **Type 1 — `HardwareProfile`:**
   - Medição real de latência de cold load, tempo até o primeiro token (TTFT em ms), tempo por token de saída (TPOT em ms), taxa de geração contínua (TPS) e pico de VRAM consumida com monitoramento térmico pelo NVML.
2. **Type 2 — `CognitiveEval` (10 Trilhas Canônicas de Rigidez SWE):**
   - **2A (AST Rust):** Emissão estrita de `enum`, `struct` com lifetime e bloco `impl`.
   - **2B (JSON Schema):** Emissão sintática válida segundo schemas `llguidance`.
   - **2C (CoT Reasoning):** Dedução lógica multi-passo (cálculo de ring buffer módulo 16).
   - **2D (IFEval Negative Constraints):** Proibição de `unwrap`, proibição de comentários, <25 linhas e token sentinela.
   - **2E (Bug Hunting):** Detecção de deadlock por inversão de ordem em mutexes Rust.
   - **2F (MCP Outline Ambiguity):** Discriminação cirúrgica entre `read_file` e `souls_ast_outline`.
   - **2G (PT-BR Técnico NT/WDDM):** Fluência técnica vernácula explicando limites de silício.
   - **2H (Surgical Patch Replace):** Emissão estrita de patches no formato `SEARCH/REPLACE`.
   - **2I (Strict API Anti-Hallucination):** Aderência estrita a contratos expostos por crates internas.
   - **2J (Compiler Auto-Heal):** Correção direta de diagnósticos de erro do compilador `rustc`.
3. **Type 3 — `MultimodalVision`:**
   - Avaliação de acoplamento do sidecar `mmproj` e precisão de bounding boxes cartesianas.
4. **Type 4 — `SpeculativeDraft`:**
   - Teste de aceleração especulativa com modelo de rascunho acoplado (`-md`), medindo a taxa de aceitação de tokens ($\alpha$).
5. **Type 5 — `ContextPressure` (Needle-in-a-Haystack 8k):**
   - Recuperação de agulha semântica injetada no meio de um haystack de 8.192 tokens para aferir estabilidade de atenção sob a margem do Windows DWM.

---

## 💾 4. Simbiose Hermes Agent & Hard Links de 0 Bytes

Para evitar a duplicação de dezenas de gigabytes de arquivos `.gguf` entre a pasta do Souls Engine (`C:\Users\rosas\llm_local_models`) e o Hermes Agent (`%LOCALAPPDATA%\hermes\models`), a crate implementa sincronização nativa por **Hard Links NTFS/ReFS**:

```
C:\Users\rosas\llm_local_models\
  ├── tier_1_master_dgpu\xlam-2-1b-fc-r-q4-k-m\xLAM-2-1B-fc-r-Q4_K_M.gguf  ──┐ (Mesmo Inode / Cluster)
                                                                             │
C:\Users\rosas\AppData\Local\hermes\models\                                   │
  └── xLAM-2-1B-fc-r-Q4_K_M.gguf  ◄──────────────────────────────────────────┘ (0 Bytes adicionais!)
```

- **Criação Automática:** Todo modelo ativo sincronizado ganha uma entrada no diretório do Hermes.
- **Poda Automática:** Se um modelo for expurgado da base do Souls Engine, o link correspondente em `hermes/models` é removido imediatamente para evitar resíduos.

---

## 🛠️ 5. Guia de Uso do CLI `souls_arena`

A crate expõe o binário executável `souls_arena.exe`.

### 5.1 Compilação Nativa
```powershell
# Compilação padrão MSVC em Z:\souls_engine
cargo build -p souls_llm_local_arena --bin souls_arena
```

### 5.2 Comandos Principais

#### 📋 Listar Modelos Detectados
```powershell
# Lista todos os modelos disponíveis no disco
cargo run -p souls_llm_local_arena --bin souls_arena -- list

# Filtrar por Tier específico (0, 0.5, 1, 2, 3, 4)
cargo run -p souls_llm_local_arena --bin souls_arena -- list --tier 1
```

#### 🔄 Sincronização SSOT & Hermes Links
Varre os modelos no disco, cadastra no banco SQLite e atualiza os Hard Links na pasta do Hermes Agent:
```powershell
cargo run -p souls_llm_local_arena --bin souls_arena -- sync
```

#### ⚡ Executar Baterias de Benchmark (`run`)
```powershell
# Executar medição física de Hardware (Type 1) em todos os modelos de Tier 1
cargo run -p souls_llm_local_arena --bin souls_arena -- run --type 1 --tier 1

# Executar a bateria de Rigidez Cognitiva (Type 2) em um modelo específico
cargo run -p souls_llm_local_arena --bin souls_arena -- run --type 2 --model xlam-2-1b

# Executar todas as baterias e exportar dossiê Markdown oficial
cargo run -p souls_llm_local_arena --bin souls_arena -- run --type all --output docs/observability/reports/arena_report.md
```

---

## 💻 6. Uso como Biblioteca Rust (API Pública)

```rust
use souls_llm_local_arena::{ArenaRunner, BenchmarkType, EngineType, ModelTier};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let runner = ArenaRunner::new();

    // 1. Descoberta física de modelos
    let models = runner.discover_models();
    println!("Modelos descobertos: {}", models.len());

    // 2. Sincronização com o SQLite Heuristic Vault
    let count = runner.sync_models_to_vault().await?;
    println!("Modelos registrados no banco: {}", count);

    // 3. Sincronização de Hard Links de 0 bytes com o Hermes Agent
    let report = runner.sync_hermes_links(None)?;
    println!(
        "Hermes Links: {} criados, {} verificados, {} removidos",
        report.links_created, report.links_verified, report.links_removed
    );

    // 4. Execução de HardwareProfile em um modelo específico
    if let Some(target) = models.iter().find(|m| m.tier == ModelTier::Tier1Master) {
        let mut runner = runner;
        let hw_res = runner.run_hardware_profile(target, EngineType::LlamaCuda).await?;
        println!("Modelo: {} | TPS: {:.1} | TTFT: {:.1} ms", target.name, hw_res.tps, hw_res.ttft_ms);
    }

    Ok(())
}
```

---

## 🧪 7. Testes e Higiene de Compilação

Em estrita conformidade com a **Cláusula 2.2 do `AGENTS.md`**, nenhuma alteração nesta crate é entregue sem validação limpa:

```powershell
cargo check -p souls_llm_local_arena
cargo test -p souls_llm_local_arena
```

---
*Souls Engine v7 Bare-Metal Infrastructure — Mantido sob governança inegociável pelo Arquiteto Chefe.*
