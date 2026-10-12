# DESIGN.md — Especificação Técnica da Sessão 4: AST Tree-Sitter & Hardware Watchdog (v7)

> **ESTATUS DO DOCUMENTO:** CANÔNICO (SDD)  
> **ESPECIFICAÇÕES DE REFERÊNCIA:** `CRATES_SPECIFICATION.md`, `MCP_TOOL_CONTRACTS.md`, `ARENA_BENCHMARK_SPECIFICATION.md`

---

## 1. DESIGN DO SUBSISTEMA DE AST (`crates/souls_ast`)

### 1.1 Desidratação Sintática (`outline`)
* **Propósito:** Substituir leituras cegas de arquivos inteiros por esqueletos sintáticos desidratados.
* **Mecânica:**
  1. O arquivo em disco é lido e sanitizado para quebras de linha LF (`\n`).
  2. O parser isolado (ou extrator de gramática baseado em Tree-sitter) identifica itens de nível superior:
     - Rust: `struct`, `enum`, `fn`, `trait`, `impl`, `type`, `mod`, `pub use`.
     - Python: `def`, `class`, decorators.
     - TS/JS: `interface`, `type`, `class`, `function`, `export const`.
  3. O corpo das funções e blocos de implementação é omitido e substituído por:
     `/* ... omitted [N lines] ... */`
  4. Métricas calculadas:
     - `original_lines`, `dehydrated_lines`, `saved_tokens_pct = (1.0 - (dehydrated_chars / original_chars)) * 100`.

### 1.2 Extração Cirúrgica de Símbolos (`symbol`)
* **Propósito:** Localizar com exatidão um bloco sintático específico (ex: `SoulsMemoryStore::persist_turn` ou `struct AppState`).
* **Mecânica:**
  1. Varredura do arquivo buscando assinaturas que contenham o identificador alvo.
  2. Extração do intervalo de linhas completo `[start_line, end_line]` do bloco, preservando indentação e comentários anexos.

### 1.3 Cirurgia de Código com Garantia de Fechamento (`delta_diff`, `edit`, `replace`)
* **Propósito:** Edição atômica e computação de patch mínimo com algoritmo de Myers.
* **Mecânica:**
  1. Normalização compulsória de CRLF para LF antes de qualquer operação.
  2. `delta_diff`: gera o objeto `MyersPatch` estruturado com `additions`, `deletions` e `changes` linha a linha.
  3. `edit`: substitui string exata com verificação de unicidade.
  4. `replace`: substitui bloco com verificação de sintaxe de fechamento (parênteses, chaves e colchetes balanceados).

### 1.4 Gitoxide Frecency Heatmap & Blast Radius (`repo_heatmap`, `repo_impact`)
* **Propósito:**
  - `repo_heatmap`: calcula o calor de modificações de arquivos usando decaimento Langevin ($e^{-\lambda \Delta t}$).
  - `repo_impact`: analisa o grafo de impacto (arquivos que importam o arquivo alterado) via busca em largura reversa (BFS).

---

## 2. DESIGN DO SUBSISTEMA DE HARDWARE WATCHDOG & LOGIT PROBING (`crates/souls_inference_runtime`)

### 2.1 Sensor Epistêmico de Host (`intent`)
* **Propósito:** Avaliação preliminar de ambiguidade informacional, cálculo de entropia de Shannon e classificação de risco/intenção em <15ms na CPU (0 MB VRAM).
* **Mecânica:**
  1. O prompt/task recebido é submetido ao `LlamaLogitProber` ou `OnnxScorer`.
  2. Extração do vetor de logits unnormalized no último token do prefill (`batch.logits[last_token] = true`).
  3. Cálculo da Softmax numericamente estável em RAM e derivação da entropia de Shannon ($H$).
  4. Se $H \ge 0.75$, a flag `circuit_breaker_triggered` é ativada.
  5. Retorno estruturado contendo classe de intenção (`Coding`, `Architecture`, `FastHeuristic`, `SecurityPromptInjection`), entropia calculada e tempo de execução em milissegundos.

---

## 3. CONTRATO DE TRANSPORTE NO DESPACHANTE MCP (`crates/souls_server`)

No despachante JSON-RPC `tools/call`, as seguintes rotas deixam de usar `handle_staging_fallback` e passam a invocar os novos manipuladores em `mcp/handlers.rs`:
* `"outline"` $\to$ `handlers::handle_outline(state, args)`
* `"symbol"` $\to$ `handlers::handle_symbol(state, args)`
* `"callers"` $\to$ `handlers::handle_callers(state, args)`
* `"callees"` $\to$ `handlers::handle_callees(state, args)`
* `"get_ast"` $\to$ `handlers::handle_get_ast(state, args)`
* `"delta_diff"` $\to$ `handlers::handle_delta_diff(state, args)`
* `"edit"` $\to$ `handlers::handle_edit(state, args)`
* `"replace"` $\to$ `handlers::handle_replace(state, args)`
* `"repo_heatmap"` $\to$ `handlers::handle_repo_heatmap(state, args)`
* `"repo_impact"` $\to$ `handlers::handle_repo_impact(state, args)`
* `"intent"` $\to$ `handlers::handle_intent(state, args)`
