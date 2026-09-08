# DESIGN.md — Arquitetura de Transplante e Sinergia de Crates

## 1. Mapeamento Topológico de Transplante

Os componentes viáveis de `_donor/` serão integrados dentro da estrita topologia de 9 crates canônicas do Souls Engine v7. **A criação de uma 10ª crate foi categoricamente descartada**, pois a matriz das 9 crates atuais é suficiente para abraçar todos os domínios mapeados sem introduzir acoplamentos cíclicos ou "Context Rot".

### Mapeamento Direto por Crate de Destino

1. **`souls_anthropophagy`:**
   - Absorverá integralmente `souls_mc_anthropophagy/src/harvester/*`.
   - Inclui execução isolada do SAST (`sast/*.rs`) e as verificações estáticas de SPDX.

2. **`souls_inference_runtime`:**
   - Recebe a lógica refinada de `vram_hardware/*` (Watchdogs de memória via NVML).
   - Incorpora `inference/ort_scorer.rs` e `gliclass_engine.rs` como motores Tier 0.
   - Incorpora `inference/llama_logit_probing.rs` como Tier 0.5 para sondagem de tokens locais.

3. **`souls_ast`:**
   - Incorpora `ast/repo_heatmap.rs` e `ast/repo_impact.rs` ancorados firmemente sobre a crate auxiliar `gix`.
   - Incorpora o algoritmo matemático limpo contido em `context/myers_diff.rs`.
   - Preserva o isolamento de segurança WASM (WASI 0.2/0.3 via `Wasmtime`) para os parsers Tree-sitter, descartando o uso de chamadas FFI C nativas para evitar crashes no processo principal, mantendo `ts_set_allocator` (bump-arena) e `memmap2` para performance sub-milissegundo.
   - Incorpora `context/souls_symbol.rs`.

4. **`souls_core`:**
   - Acolhe o motor de leitura inteligente: `context/souls_smart_read.rs` e `multi_read.rs`.
   - Integra as primitivas de compressão LEAN: `context/ansi_filter.rs`, `dedup.rs` e `ccr_dedup.rs`.
   - Concentra a barreira de falha e `mpsc_flusher`.

5. **`souls_memory`:**
   - Funde os arquivos fragmentados de `memory/` (Chyros Daemon, Langevin Decay, RRF Fusion).
   - Engloba a manipulação do LadybugDB (atualmente em `state_thinking/memory_graph/`) e acesso vetorial `vector_store.rs`.

6. **`souls_protocol`:**
   - Concentrará todos os DTOs universais legados transferidos durante a depuração de tipagem.

## 2. Padrões de Assinaturas, Concorrência e Tipagem

### 2.1 Isolamento de Inanição de Threads (`spawn_blocking`)
A execução de I/O bloqueante (fora do escopo natural assíncrono), rotinas computacionais densas (Myers Diff, inferência de CPU pura) ou parsing (Tree-sitter) deverão utilizar envoltórios para proteção do pool do Tokio:

```rust
// Exemplo canônico de assinatura transplantada:
pub async fn compute_safe_myers_diff(old_content: &str, new_content: &str) -> Result<MyersPatch, AstError> {
    let old_copy = old_content.to_owned();
    let new_copy = new_content.to_owned();
    tokio::task::spawn_blocking(move || {
        // Computação cpu-bound isolada
        similar_engine::diff(&old_copy, &new_copy)
    }).await.map_err(|_| AstError::InternalEnginePanic)?
}
```

### 2.2 ReFS Win32 e Block Cloning (ADR-005)

Sempre que um transplante (notadamente na crate `souls_memory` e manipulações do LanceDB ou cópia massiva do `Chyros Daemon`) tocar no disco `Z:\souls_engine\.souls_data\`, chamadas puras da biblioteca padrão (`std::fs::copy`) deverão ser evitadas em favor de Block Cloning nativo via Kernel NT para evitar SSD Write Amplification.

O código deve utilizar a API Win32 assíncrona (com `windows-sys` v0.59+ `Win32_Storage_FileSystem` e `Win32_System_Threading`) para executar o control code `FSCTL_DUPLICATE_EXTENTS_TO_FILE` via `DeviceIoControl`:

```rust
use std::os::windows::io::AsRawHandle;
use windows_sys::Win32::Storage::FileSystem::{FILE_SHARE_READ, FILE_SHARE_WRITE, FILE_SHARE_DELETE};
use windows_sys::Win32::System::Ioctl::FSCTL_DUPLICATE_EXTENTS_TO_FILE;
use windows_sys::Win32::System::IO::DeviceIoControl;

pub async fn clone_file_refs(src: &Path, dest: &Path) -> Result<(), CoreError> {
    // ... abertura de handles (CreateFileW) ...
    // Exemplo de ioctl para Block Cloning Copy-on-Write (tempo zero)
    // DeviceIoControl(
    //     dest_handle,
    //     FSCTL_DUPLICATE_EXTENTS_TO_FILE,
    //     &duplicate_request as *const _ as *const c_void,
    //     ...
    // );
}
```

### 2.3 Matriz de Domínio de Erros MCP (`thiserror`)

Qualquer transplante envolverá a purga absoluta de `anyhow` do código de produção. As respostas das funções migrarão de `anyhow::Result<T>` para uma tipagem estrita via `thiserror` (v2) em `souls_protocol`, de modo que os erros sejam mapeáveis diretamente para os códigos JSON-RPC padronizados (faixa `-32000` a `-32099`):

```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum McpDomainError {
    #[error("InternalEnginePanic: Pânico recuperado na thread Tokio")]
    InternalEnginePanic, // Código -32000

    #[error("FileTooLargeForAST: O arquivo excedeu o limite físico")]
    FileTooLargeForAST, // Código -32001

    #[error("VramThermalThrottled: Watchdog NVML disparou barreira")]
    VramThermalThrottled, // Código -32002
    
    #[error("SymbolNotFound: O identificador não foi achado na AST")]
    SymbolNotFound, // Código -32003
}
```
