# RFC-007: Especificação Arquitetural da Crate `souls_llm_inventory` (v7)

> **Status:** Proposto / Aprovado para Roadmap Futuro  
> **Data:** 13 de Outubro de 2026  
> **Autor:** Arquiteto do Sistema & Antigravity IDE  
> **Domínio:** Infraestrutura de Silício, Catálogo de Modelos Locais & Download Manager  

---

## 1. Contexto e Motivação

Atualmente, o **Souls Engine v7** possui subsistemas dedicados para inferência local bare-metal (`souls_inference_runtime`), roteamento multiobjetivo (`souls_model_router`) e benchmarks em arena (`souls_llm_local_arena`). No entanto, o **gerenciamento físico do inventário de arquivos de pesos em disco** (`.gguf`, `.onnx`), a descoberta automática de novos modelos e o download seguro a partir de repositórios (como HuggingFace Hub) encontram-se dispersos ou dependentes de scripts manuais.

A futura crate **`souls_llm_inventory`** surge para sanar essa lacuna de forma especializada, desacoplando o *gerenciamento de ativos em disco* do *mecanismo de execução de tensores*.

### Objetivos Primários:
1. **Descoberta e Catalogação Zero-Copy:** Varrer o diretório universal de modelos locais (`%USERPROFILE%\llm_local_models`) e extrair metadados estruturados de cabeçalhos GGUF (arquitetura, quantização, contagem de parâmetros, context length) sem alocar buffers de tensores na RAM ou VRAM.
2. **Profiling de Silício Adaptativo:** Cruzar os requisitos de memória de cada modelo com as métricas do NVML/WDDM da GPU hospedeira (ex: teto útil de 5.294 MB na NVIDIA RTX 2060m) para classificar se o modelo roda 100% na dGPU, em modo híbrido CPU/GPU ou se provocará spillover no barramento PCIe.
3. **Download Manager Resiliente com SSE:** Gerenciar downloads de arquivos `.gguf` pesados com suporte a *HTTP Range requests* (capacidade de pausar e retomar downloads interrompidos), verificação de integridade criptográfica (SHA-256) e broadcast de progresso em tempo real para o Web Dashboard.
4. **Mapeamento de Perfis Cognitivos:** Classificar modelos instalados nos tiers funcionais do Souls Engine (Tier 0: Velocista, Tier 0.5: Socrático/Cohomologia, Tier 1: Trator de Código).

---

## 2. Padrão de Armazenamento e Portabilidade Universal

Para garantir que o Souls Engine funcione de forma transparente em qualquer máquina Windows sem dependência de nomes de usuário fixos:
- **Caminho Universal Canônico:**  
  `%USERPROFILE%\llm_local_models` (resolvido em Rust via `dirs::home_dir().join("llm_local_models")` e em PowerShell via `$env:USERPROFILE\llm_local_models`).
- **Override Opcional:**  
  Configurável no arquivo `souls_config.toml` via chave:
  ```toml
  [storage]
  models_directory = "C:\\caminho\\customizado\\llm_local_models"
  ```
- **Tolerância a Catálogo Vazio:** Se o diretório não existir ou estiver vazio, o motor cria o diretório de forma idempotente e o dashboard apresenta uma interface convidativa para download do primeiro modelo recomendado para a GPU detectada.

---

## 3. Topologia e Módulos Internos da Crate

A crate `souls_llm_inventory` será estruturada com a seguinte árvore de módulos:

```text
crates/souls_llm_inventory/
├── Cargo.toml
├── src/
│   ├── lib.rs                  # Interface pública da crate
│   ├── catalog.rs              # Varredura assíncrona e leitura de metadados GGUF/ONNX
│   ├── profiler.rs             # Hardware Profiler (WDDM/VRAM/AVX2 vs Requisitos de Modelo)
│   ├── downloader.rs           # Download Manager assíncrono com HTTP Range e Resume
│   ├── checksum.rs             # Verificação incremental de integridade SHA-256
│   ├── profiles.rs             # Classificação automática por perfil cognitivo (Tiers 0, 0.5, 1)
│   ├── registry.rs             # Catálogo de modelos recomendados e quantizações ideais
│   └── error.rs                # Enumeração de erros tipados (InventoryError)
```

---

## 4. Contratos de Dados (DTOs Canônicos)

### 4.1 Metadados do Modelo em Disco (`LocalModelMeta`)

```rust
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Formato e runtime suportado para o modelo local.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ModelFormat {
    Gguf,
    Onnx,
    SafeTensors,
}

/// Classificação de viabilidade física no hardware detectado.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HardwareFit {
    /// Cabe 100% na VRAM dedicada sob o teto WDDM (máximo desempenho: 40-50 tok/s).
    OptimalVram,
    /// Requer offload parcial para CPU RAM via AVX2 (desempenho intermediário: 15-25 tok/s).
    PartialCpuOffload,
    /// Excede o silício disponível; risco de spillover PCIe e throttling severo (< 2 tok/s).
    IncompatibleSpillover,
}

/// Metadados consolidados de um modelo inspecionado em disco.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocalModelMeta {
    pub file_name: String,
    pub absolute_path: PathBuf,
    pub format: ModelFormat,
    pub size_bytes: u64,
    pub size_gb: f64,
    pub architecture: String,         // ex: "qwen2", "llama", "phi3"
    pub parameter_count: Option<u64>,  // ex: 3_000_000_000 (3B)
    pub quantization: String,          // ex: "Q4_K_M", "Q5_K_M", "FP16"
    pub context_length_max: u32,       // ex: 32768
    pub hardware_fit: HardwareFit,     // Viabilidade no hardware local
    pub assigned_tier: Option<String>, // "tier0_velocista", "tier1_coder", etc.
    pub last_modified_unix: u64,
}
```

### 4.2 Estado do Download em Tempo Real (`DownloadProgress`)

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DownloadState {
    Queued,
    Connecting,
    Downloading,
    Paused,
    VerifyingChecksum,
    Completed,
    Failed(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadProgress {
    pub download_id: String,
    pub model_id: String,
    pub file_name: String,
    pub total_bytes: u64,
    pub downloaded_bytes: u64,
    pub progress_pct: f32,
    pub speed_mb_s: f32,
    pub eta_seconds: u32,
    pub state: DownloadState,
}
```

---

## 5. Regras do Hardware Profiler (RTX 2060m & WDDM)

O módulo `profiler.rs` utilizará a seguinte fórmula matemática para classificar a adequação do modelo:

$$\text{VRAM}_{\text{necessária}} = \text{Tamanho}_{\text{pesos}} + (\text{Contexto} \times \text{KV-Cache-Per-Token}) + \text{Margem}_{\text{CUDA (300 MB)}}$$

$$\text{Margem Útil WDDM} = \text{VRAM}_{\text{total}} - \text{Consumo}_{\text{DWM/Windows}} \approx 6.144 - 850 = 5.294 \text{ MB}$$

1. **Se $\text{VRAM}_{\text{necessária}} \le 4.900 \text{ MB}$:**
   - Classificação: `OptimalVram` (Aprovado com folga de segurança para o Tier 1).
2. **Se $4.900 \text{ MB} < \text{VRAM}_{\text{necessária}} \le 12.000 \text{ MB}$:**
   - Classificação: `PartialCpuOffload` (Execução direcionada para CPU Intel Core i9 com extensões AVX2, poupando a dGPU para evitar spillover PCIe).
3. **Se $\text{VRAM}_{\text{necessária}} > 12.000 \text{ MB}$:**
   - Classificação: `IncompatibleSpillover` (Alerta visual no Dashboard contra download por inviabilidade de hardware).

---

## 6. Integração com o Web Dashboard do Souls Engine

A crate exporá rotas REST no servidor Axum para consumo pelo Web Dashboard:

| Método | Endpoint | Descrição |
| :--- | :--- | :--- |
| `GET` | `/api/v1/inventory/models` | Lista todos os modelos locais com profiling de silício. |
| `GET` | `/api/v1/inventory/recommendations` | Lista modelos recomendados no HuggingFace para a GPU detectada. |
| `POST` | `/api/v1/inventory/download` | Inicia o download de um modelo (`repo_id`, `filename`). |
| `POST` | `/api/v1/inventory/download/:id/pause` | Pausa um download ativo. |
| `POST` | `/api/v1/inventory/download/:id/resume` | Retoma download via HTTP Range header. |
| `DELETE` | `/api/v1/inventory/models/:filename` | Remove modelo do disco de forma segura. |
| `GET` | `/api/v1/inventory/stream` | Stream SSE de progresso de downloads e telemetria de disco. |

---

## 7. Conformidade com as Linhas Vermelhas (AGENTS.md)

- **Linha Vermelha 1 (Zero Desktop UI):** Toda a interação do usuário é feita exclusivamente via Web Dashboard oficial servido via HTTP/SSE.
- **Linha Vermelha 4 (Sem poluição):** O inventário respeita o diretório unificado de modelos e metadados persistentes em `.souls_data`.
- **Linha Vermelha 5 (Topologia de Crates):** Esta crate é catalogada como proposta formal para ser incorporada no momento oportuno da arquitetura de LLMs locais.
- **Linha Vermelha 6 (Zero Unsafe):** Código 100% seguro em Rust puro (`tokio`, `reqwest`, `sha2`, `serde`).
