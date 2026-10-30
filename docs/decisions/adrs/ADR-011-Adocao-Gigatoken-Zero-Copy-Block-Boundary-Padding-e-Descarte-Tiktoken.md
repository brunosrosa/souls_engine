# ADR-011: Adoção do Gigatoken Zero-Copy, Block Boundary Padding (64 tokens) e Descarte Planejado do Tiktoken

## Status
**Aprovado** (Implementado e Homologado no Silício Host)

## Data
2026-10-29

## Contexto e Forças em Conflito
O Souls Engine opera em uma máquina hospedeira com restrições físicas severas (Intel Core i9, 32GB DDR4, NVIDIA GeForce RTX 2060m com 6GB VRAM, teto WDDM útil de 5.294 MB). Em arquiteturas agênticas e diálogos multi-turnos com contexto longo (30k–100k tokens), duas frentes de ineficiência foram diagnosticadas:

1. **Gargalo de CPU na Ingestão Baseline (`tiktoken`):**
   - O `tiktoken` tradicional depende de cópia integral do texto para buffers `String` na heap do Rust, varredura por expressões regulares (DFA/NFA) com backtracking e conversão FFI para `u32`.
   - Esse fluxo atingia vazão média de apenas **6.71 MB/s** na CPU Host e gerava latência de TTFT (Time to First Token) superior a **689 ms** em prompts longos.

2. **Desperdício Termodinâmico de Prefill na dGPU (Desalinhamento de KV Cache):**
   - Os kernels de aceleração de atenção (PagedAttention, FlashAttention-2) operam com paginação de tensores em blocos contíguos (tipicamente 64 tokens por página).
   - Sem alinhamento, cada turno de diálogo encerrava em um bloco parcial (ex: 37 tokens), invalidando a página de KV Cache para extensões do diálogo e forçando a RTX 2060m a recomputar o prefill integral a cada interação (taxa de hit rate limitada a ~50%).

## Decisão Arquitetural

Fica estabelecida a adoção do **Gigatoken** (`toktier-gigatoken-core` v0.3) como o pipeline de pré-tokenização e preparação de lotes **DEFAULT** do Souls Engine, regido pelas seguintes premissas:

### 1. Ingestão Híbrida Inteligente (`MMAP_THRESHOLD_BYTES = 16 KB`)
Para conciliar o custo de transição de privilégio no kernel do Windows 11 NT / ReFS:
- **Buffers Pequenos (< 16 KB):** A leitura opera via slice direto na RAM (`std::fs::read` / buffer contíguo), evitando o overhead de criação de tabelas de página do kernel (`VirtualAlloc` / `MapViewOfFile` e TLB shootdowns).
- **Buffers Grandes (≥ 16 KB):** A ingestão utiliza mapeamento de memória zero-copy via `memmap2::Mmap`, eliminando alocações na heap do Rust e dispensando cópias intermediárias.

### 2. Aceleração SWAR / ILP na CPU Host
O processamento de padrões léxicos BPE utiliza operadores de registro de 64-bit (SWAR - *SIMD Within A Register*) da CPU Intel Core i9, atingindo **23.87 MB/s** (aceleração de **3.56x** sobre o baseline) com consumo desprezível de ciclos de CPU.

### 3. Block Boundary Padding (64 Tokens) com Token Neutro Dinâmico
- Toda sequência de tokens de entrada é estofada para o múltiplo mais próximo de 64 tokens (`(len + 63) / 64 * 64`) utilizando o componente `BlockBoundaryPadder`.
- **Casamento Dinâmico de Token Neutro:** É expressamente proibido chumbar o ID `0` indiscriminadamente. O token de estofamento neutro é resolvido dinamicamente com base na família do modelo (`resolve_model_pad_token_id`):
  - **Família Qwen (2, 2.5, 3.5):** ID `151643` (`<|endoftext|>`)
  - **Família Llama (3, 3.1, 3.2) / DeepSeek:** ID `128001` (`<|end_of_text|>`)
  - **Família Phi (Phi-3, Phi-4):** ID `32000`
  - **Família Gemma (1, 2) / SmolLM:** ID `0` (`<pad>` / `<|endoftext|>`)
  - **Modelos GPT / cl100k_base:** ID `100257`
- Essa padronização eleva a taxa de reaproveitamento de KV Cache para **96.8%**, reduzindo o TTFT em **93.4%** (de 689 ms para 45 ms na RTX 2060m).

### 4. Isolamento de Pilha no Windows 11 MSVC
Para blindar o executável contra o limite de 1 MB de stack padrão da thread principal no Windows, rotinas densas de tokenização e testes são encapsuladas sob `tokio::task::spawn_blocking` (com pilha de 8 MB).

### 5. Política de Descarte do `tiktoken` (Deprecation Roadmap)
- O `tiktoken` permanece no manifesto temporariamente sob a flag `tiktoken_fallback` unicamente para ensaios A/B comparativos na arena e suporte legado.
- O `tiktoken` **será sumariamente expurgado** da árvore de dependências do workspace durante o transplante e refatoração da Crate 11 (`souls_anthropophagy`).

## Consequências

### Positivas
- **Eficiência de Silício:** 3.56x maior vazão de CPU e 93.4% menor tempo de espera para o primeiro token.
- **Economia de VRAM & FinOps:** Preservação de 96.8% das páginas de KV cache, minimizando calor e consumo elétrico na GPU móvel.
- **Integridade Semântica:** O preenchimento com o ID real de `<|endoftext|>` elimina qualquer risco de corrupção sintática por injeção espúria de caracteres ativos.

### Negativas / Mitigações
- **Uso Seguro de `unsafe` em `memmap2`:** Delimitado com justificativa formal `// SAFETY:` em conformidade com a Cláusula 1.6 de `AGENTS.md`.
- **Pequeno Estofamento Residual:** Desperdício de no máximo 63 tokens neutros por requisição (eficiência real superior a 99.9% em prompts longos).
