# MCP_TOOL_CONTRACTS.md — Catálogo Canônico Exaustivo e Contratos de Ferramentas MCP (v7.2)

> **ESTATUS DO DOCUMENTO:** CANÔNICO E OBRIGATÓRIO (DOCS-AS-GUARDRAILS)
> **APLICAÇÃO:** Agentes de IA (Cursor, Windsurf, Claude Code, Trae, Hermes Agent) e Engenheiros de Sistemas.
> **CRATES DIRETAMENTE REGIDAS:** `souls_protocol`, `souls_ast`, `souls_memory`, `souls_inference_runtime`, `souls_model_router`, `souls_core`, `souls_server`.
> **PROPÓSITO:** Congelar as assinaturas, tipos, parâmetros, esquemas JSON-RPC 2.0, políticas de revelação progressiva e códigos de erro de domínio expostos pelo barramento soberano `souls_mcp`. Nenhuma ferramenta pode ter sua assinatura alterada sem revisão formal de contrato.

## 1. PREÂMBULO, TRANSPORTE E REVELAÇÃO PROGRESSIVA

### 1.1 Vetores de Transporte e Nomenclatura Soberana (ADR-041)

O subsistema MCP do Souls Engine é hospedado pelo binário bare-metal `souls_server.exe` através de dois transportes rigorosamente isolados:

1. **Localhost TCP Loopback com Streamable HTTP/SSE:** Vinculado em `http://127.0.0.1:9123/mcp` (Canal primário de integração com o Hermes Agent e microsserviços).
2. **Standard I/O (`stdio`) Descontaminado:** Pipe bare-metal de comunicação direta com IDEs (Cursor, Windsurf, Trae) via JSON-RPC 2.0 puro, com isolamento estrito contra poluição de escape ANSI ou logs de stderr.
	- **Server Identifier (Servername):** `souls_mcp`
	- **Disciplina Zero-Brand (ADR-026):** Ferramentas são expostas limpas por padrão (ex: `outline`, `read`, `semantic_search`), com resolução e normalização transparente de aliases no despachante RPC (`souls_<tool>`, `souls_mcp.<tool>`, `ctx_<tool>`).
	- **Tetos de Caracteres:** Nome da ferramenta $\le 32\text{ caracteres}$; Descrição utilitária $\le 160\text{ caracteres}$.

### 1.2 O Paradoxo do Inchaço de Contexto e o Mecanismo de Revelação Progressiva

A exposição estática simultânea de mais de 50 esquemas JSON consome entre 12.000 e 20.000 tokens no prompt de sistema do agente, provocando _Tool Blindness_ (cegueira de ferramentas), alucinação de parâmetros e aumento desnecessário de custos FinOps.

Para resolver o inchaço de contexto **sem amputar o silício**, o Souls Engine v7 adota a **Revelação Progressiva de Ferramentas (Progressive Tool Disclosure)**:

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                 NÍVEL 0: BOOT COMPACTO (Sempre Visível)                     │
│  [outline] [symbol] [read] [edit] [thinking] [route_subtask] [tool_search]  │
└──────────────────────────────────────┬──────────────────────────────────────┘
                                       │
                    Invoca 'tool_search(namespace)'
                                       ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│                 NÍVEL 1: CATÁLOGOS DINÂMICOS SOB DEMANDA                    │
│   code.*     ──► AST Tree-sitter, Callers/Callees, Myers Diff               │
│   context.*  ──► Compressão LEAN, CCR De/Rehydration, Multi-Read            │
│   memory.*   ──► Grafo LadybugDB, Observações, SQLite STRICT, LanceDB       │
│   think.*    ──► Raciocínio Socrático DAG, Sessões, Intent Probing          │
│   ops.*      ──► Heatmap gitoxide, Impact/Blast Radius, Hardware & FinOps   │
└─────────────────────────────────────────────────────────────────────────────┘
```

1. **Conjunto Base de Boot:** O agente recebe inicialmente apenas as ferramentas essenciais de cada namespace mais a meta-ferramenta canônica `tool_search`.
2. **Ativação Just-in-Time:** Quando a tarefa exige manipulação de memória ontológica profunda, varredura de impacto ou compressão em lote, o agente invoca `tool_search(namespace)` para receber a descrição e o JSON Schema exato das ferramentas especializadas daquele domínio.
3. **Modo Monolítico para IDEs:** Clientes de desenvolvimento local que implementam janelas amplas ou filtragem nativa de ferramentas recebem o catálogo completo sem restrições.

### 1.3 As 5 Skills Canônicas de Orquestração Procedural

Ferramentas MCP são as _garras mecânicas_; as **Skills** são os _playbooks procedurais_ que ensinam a IA a encadear as garras sem sangrar tokens. O Souls Engine amarra o catálogo a 5 Skills canônicas (armazenadas em `.agents/skills/` ou `$HERMES_HOME/skills/`):

1. **`souls-code-surgeon`:** Governa o namespace `code.*`. Proíbe leitura de arquivos integrais; impõe o fluxo: `outline` $\rightarrow$ `symbol` / `callers` $\rightarrow$ `delta_diff` / `edit`.
2. **`souls-context-lean`:** Governa o namespace `context.*`. Impõe medição de tokens com `smart_read`; se o arquivo for volumoso, comuta para `compress` / `fill` com desidratação CCR na RAM.
3. **`souls-memory-triad`:** Governa o namespace `memory.*`. Executa busca híbrida RRF (`semantic_search`), sanitiza alucinações no LadybugDB (`mem_open_nodes`) e registra fatos atômicos (`knowledge`, `mem_create_*`).
4. **`souls-socratic-thinking`:** Governa o namespace `think.*`. Ativa o freio cognitivo `thinking` em tarefas de alta entropia, desdobrando hipóteses em grafos DAG antes da escrita de código.
5. **`souls-ops-finops`:** Governa o namespace `ops.*`. Antes de qualquer refatoração estrutural, calcula o _Blast Radius_ com `repo_impact` e inspeciona a relevância com `repo_heatmap`.

## 2. QUADRO GERAL DAS 52 GARRAS MCP POR NAMESPACE

Abaixo consta a relação exaustiva de todas as ferramentas operacionais do Souls Engine v7.2, sua maturidade clínica, latência média no silício e crate Rust de ancoragem:

| **#** | **Garra (tool_name)**         | **Namespace** | **O Que Faz no Sistema**                                                                                | **Crate Executora**       | **Maturidade Clínica** | **Latência Típica** |
| ----- | ----------------------------- | ------------- | ------------------------------------------------------------------------------------------------------- | ------------------------- | ---------------------- | ------------------- |
| —     | **`tool_search`**             | `core`        | **Meta-Ferramenta:** Busca e retorna esquemas detalhados de ferramentas por namespace ou palavra-chave. | `souls_server`            | `LIVE_PRODUCTION`      | **< 1.0 ms**        |
| 1     | **`outline`**                 | `code`        | Gera esqueleto AST desidratando corpos de funções (redução de 70-85% de tokens).                        | `souls_ast`               | `LIVE_PRODUCTION`      | **9.64 ms**         |
| 2     | **`symbol`**                  | `code`        | Resolve coordenadas (`file:line:col`) e escopo de símbolos via AST Tree-sitter.                         | `souls_ast`               | `LIVE_PRODUCTION`      | **258.32 ms**       |
| 3     | **`callers`**                 | `code`        | Mapeia quem invoca um determinado símbolo em todo o workspace.                                          | `souls_ast`               | `LIVE_PRODUCTION`      | **0.32 ms**         |
| 4     | **`callees`**                 | `code`        | Mapeia quais funções e métodos são consumidos internamente pelo símbolo.                                | `souls_ast`               | `LIVE_PRODUCTION`      | **0.41 ms**         |
| 5     | **`get_ast`**                 | `code`        | Extrai a árvore sintática concreta (CST/AST) em formato JSON estruturado.                               | `souls_ast`               | `LIVE_PRODUCTION`      | **1.20 ms**         |
| 6     | **`edit`**                    | `code`        | Aplica edições cirúrgicas Search/Replace com travamento assíncrono e validação.                         | `souls_ast`               | `LIVE_PRODUCTION`      | **17.68 ms**        |
| 7     | **`replace`**                 | `code`        | Substitui blocos estruturais de código com rollback atômico em falha sintática.                         | `souls_ast`               | `LIVE_PRODUCTION`      | **15.42 ms**        |
| 8     | **`delta_diff`**              | `code`        | Computa Myers Diff estrutural entre versões usando o algoritmo da crate `similar`.                      | `souls_ast`               | `LIVE_PRODUCTION`      | **0.52 ms**         |
| 9     | **`read`**                    | `context`     | Lê arquivo individual aplicando formatação TOON e SymbolMap de alta densidade.                          | `souls_core`              | `LIVE_PRODUCTION`      | **21.87 ms**        |
| 10    | **`smart_read`**              | `context`     | Mede tokens via `tiktoken` na CPU antes da leitura com auto-shrink adaptativo.                          | `souls_core`              | `LIVE_PRODUCTION`      | **4.10 ms**         |
| 11    | **`multi_read`**              | `context`     | Leitura concorrente de múltiplos arquivos na RAM com agrupamento CCR.                                   | `souls_core`              | `LIVE_PRODUCTION`      | **32.45 ms**        |
| 12    | **`compress`**                | `context`     | Aplica compressão LEAN, removendo ruído sintático e comentários redundantes.                            | `souls_core`              | `LIVE_PRODUCTION`      | **142.67 ms**       |
| 13    | **`dedup`**                   | `context`     | Substitui trechos duplicados de 5+ linhas por marcadores estruturais compactos.                         | `souls_core`              | `LIVE_PRODUCTION`      | **0.63 ms**         |
| 14    | **`fill`**                    | `context`     | Reidrata marcadores de compressão CCR de volta para o texto lossless via hash.                          | `souls_core`              | `LIVE_PRODUCTION`      | **0.29 ms**         |
| 15    | **`stub_fill`**               | `context`     | Preenche stubs demarcados em arquivos locais validando integridade de fechamento.                       | `souls_core`              | `LIVE_PRODUCTION`      | **0.85 ms**         |
| 16    | **`headroom_retrieve`**       | `context`     | Recupera blocos de cache comprimido diretamente da RAM (Loopback CCR).                                  | `souls_core`              | `UPGRADE_CANDIDATE`    | **0.27 ms**         |
| 17    | **`semantic_search`**         | `memory`      | Busca híbrida RRF combinando BM25 (FTS5) e cosseno (LanceDB, 0MB VRAM).                                 | `souls_memory`            | `LIVE_PRODUCTION`      | **87.60 ms**        |
| 18    | **`sqlite_query`**            | `memory`      | Executa consultas SQL read-only ultra-seguras no FrankenSQLite STRICT WAL.                              | `souls_memory`            | `LIVE_PRODUCTION`      | **17.83 ms**        |
| 19    | **`knowledge`**               | `memory`      | Armazena, versiona e recupera diretrizes epistêmicas no banco de dados L2.                              | `souls_memory`            | `LIVE_PRODUCTION`      | **1.83 ms**         |
| 20    | **`mem_search`**              | `memory`      | Busca lexical FTS5 ultrarrápida sobre o repositório de observações do grafo.                            | `souls_memory`            | `LIVE_PRODUCTION`      | **0.34 ms**         |
| 21    | **`mem_open_nodes`**          | `memory`      | Carrega entidades por nome, hidratando observações e conexões ativas.                                   | `souls_memory`            | `LIVE_PRODUCTION`      | **0.24 ms**         |
| 22    | **`mem_read_graph`**          | `memory`      | Varre a topologia ativa do grafo ontológico (com limite defensivo contra OOM).                          | `souls_memory`            | `LIVE_PRODUCTION`      | **0.24 ms**         |
| 23    | **`mem_create_entities`**     | `memory`      | Cria entidades idempotentes no grafo LadybugDB via MPSC assíncrono.                                     | `souls_memory`            | `LIVE_PRODUCTION`      | **0.24 ms**         |
| 24    | **`mem_create_relations`**    | `memory`      | Cria arestas direcionadas e tipadas (`depends_on`, `conflicts_with`) no grafo.                          | `souls_memory`            | `LIVE_PRODUCTION`      | **0.31 ms**         |
| 25    | **`mem_add_observations`**    | `memory`      | Anexa observações clínicas a nós existentes com sincronização atômica FTS5.                             | `souls_memory`            | `LIVE_PRODUCTION`      | **0.27 ms**         |
| 26    | **`mem_delete_entities`**     | `memory`      | Expurga entidades e suas arestas associadas sob confirmação HITL.                                       | `souls_memory`            | `LIVE_PRODUCTION`      | **0.30 ms**         |
| 27    | **`mem_delete_relations`**    | `memory`      | Remove arestas lógicas específicas no grafo ontológico.                                                 | `souls_memory`            | `LIVE_PRODUCTION`      | **0.34 ms**         |
| 28    | **`mem_delete_observations`** | `memory`      | Exclui observações atômicas com expurgo do índice invertido FTS5.                                       | `souls_memory`            | `LIVE_PRODUCTION`      | **0.45 ms**         |
| 29    | **`handoff`**                 | `memory`      | Persiste payloads de troca de turno e contexto entre subagentes delegados.                              | `souls_memory`            | `LIVE_PRODUCTION`      | **2.36 ms**         |
| 30    | **`sub_agent`**               | `memory`      | Registra e gerencia o ciclo de vida e estado de subagentes autônomos.                                   | `souls_memory`            | `LIVE_PRODUCTION`      | **2.02 ms**         |
| 31    | **`thinking`**                | `think`       | Freio cognitivo socrático (`core_think`): hipóteses, branches e revisões em DAG.                        | `souls_core`              | `LIVE_PRODUCTION`      | **1.38 ms**         |
| 32    | **`session`**                 | `think`       | Governa a sessão ativa, executando descarte ou reset atômico de caches voláteis.                        | `souls_core`              | `LIVE_PRODUCTION`      | **0.65 ms**         |
| 33    | **`analyze_session`**         | `think`       | Analisa a profundidade lógica e consistência socrática da sessão em RAM.                                | `souls_core`              | `LIVE_PRODUCTION`      | **3.37 ms**         |
| 34    | **`export_session`**          | `think`       | Exporta a árvore de deliberação socrática em Markdown/JSON estruturado.                                 | `souls_core`              | `LIVE_PRODUCTION`      | **3.02 ms**         |
| 35    | **`merge_sessions`**          | `think`       | Funde ramos concorrentes de deliberação cognitiva sob consistência eventual.                            | `souls_core`              | `LIVE_PRODUCTION`      | **2.42 ms**         |
| 36    | **`intent`**                  | `think`       | Avalia ambiguidade informacional via entropia e logit probing (Tier 0/0.5).                             | `souls_inference_runtime` | `UPGRADE_CANDIDATE`    | **0.78 ms**         |
| 37    | **`route_subtask`**           | `ops`         | Roteador FinOps ParetoBandit multiobjetivo com barreira termodinâmica $\Phi$.                           | `souls_model_router`      | `LIVE_PRODUCTION`      | **< 1.0 ms**        |
| 38    | **`repo_heatmap`**            | `ops`         | Mede o calor (_Frecency_) dos arquivos do workspace via commit graph `gitoxide`.                        | `souls_ast`               | `LIVE_PRODUCTION`      | **2.65 s**          |
| 39    | **`repo_impact`**             | `ops`         | Analisa o raio de impacto (_Blast Radius_) de alterações via BFS reversa no grafo.                      | `souls_ast`               | `LIVE_PRODUCTION`      | **428.01 ms**       |
| 40    | **`feedback`**                | `ops`         | Extrai métricas FinOps históricas ($E^3$, custos por token, latências percentilares).                   | `souls_core`              | `LIVE_PRODUCTION`      | **3.54 ms**         |
| 41    | **`metrics`**                 | `ops`         | Telemetria em tempo real de acertos de _prefix cache_ e alocação de hardware.                           | `souls_inference_runtime` | `UPGRADE_CANDIDATE`    | **0.26 ms**         |
| 42    | **`sys_time`**                | `ops`         | Timestamps em ISO-8601 e Unix Epoch com precisão de microssegundos via `chrono`.                        | `souls_core`              | `LIVE_PRODUCTION`      | **0.28 ms**         |
| 43    | **`tree`**                    | `sys`         | Lente de diretórios não-bloqueante com dot-flattening e poda de caminhos tóxicos.                       | `souls_core`              | `LIVE_PRODUCTION`      | **2.04 ms**         |
| 44    | **`search`**                  | `sys`         | Busca textual via regex acelerada em memória com agrupamento sintático LEAN.                            | `souls_core`              | `LIVE_PRODUCTION`      | **6.30 ms**         |
| 45    | **`shell`**                   | `sys`         | Terminal assíncrono restrito com filtro anti-ANSI e poda de logs redundantes.                           | `souls_core`              | `LIVE_PRODUCTION`      | **136.92 ms**       |
| 46    | **`execute`**                 | `sys`         | Executor isolado em sandbox de sistema operacional (AppContainer/JobObject).                            | `souls_core`              | `LIVE_PRODUCTION`      | **0.32 ms**         |
| 47    | **`fetch_web`**               | `sys`         | Scraper assíncrono via `reqwest` com conversor de HTML para Markdown limpo.                             | `souls_core`              | `LIVE_PRODUCTION`      | **~ 1.5 s**         |
| 48    | **`web_search`**              | `sys`         | Motor de busca DuckDuckGo HTML com extração direta de snippets sem tracking.                            | `souls_core`              | `LIVE_PRODUCTION`      | **909.64 ms**       |
| 49    | **`repo_meta`**               | `sys`         | Extrai metadados estruturais de repositórios Git via `gix` / `octocrab`.                                | `souls_core`              | `LIVE_PRODUCTION`      | **1.73 s**          |
| 50    | **`routes`**                  | `sys`         | Inspeciona rotas ativas do daemon Axum e status de ganchos IPC/SSE.                                     | `souls_server`            | `UPGRADE_CANDIDATE`    | **< 2.0 ms**        |

## 3. CONTRATOS DETALHADOS POR NAMESPACE FUNCIONAL

### 3.0 Meta-Ferramenta: `tool_search` (Revelação Progressiva)

#### Descrição Operacional

Permite ao agente descobrir, pesquisar e inspecionar os esquemas JSON exatos das ferramentas MCP sob demanda, eliminando o inchaço de contexto no prompt de sistema.

#### JSON Schema de Entrada (`inputSchema`)

```
{
  "type": "object",
  "properties": {
    "namespace": {
      "type": "string",
      "enum": ["all", "code", "context", "memory", "think", "ops", "sys"],
      "description": "Namespace funcional para listar ferramentas especializadas."
    },
    "query": {
      "type": "string",
      "description": "Termo de busca conceitual ou funcional (ex: 'blast radius', 'fts5', 'ast')."
    }
  },
  "additionalProperties": false
}
```

### 3.1 Namespace `code.*` (AST Tree-Sitter & Code Surgery)

#### 3.1.1 `outline` (alias: `souls_ast_outline`)

- **Descrição:** Extrai assinaturas públicas, structs, traits, enums e docstrings de um arquivo de código, suprimindo implementações com marcadores métricos de desidratação (ex: `/* ... omitted [28 lines] ... */`).
- **Linguagens:** Rust (`.rs`), Python (`.py`), TypeScript/JavaScript (`.ts`, `.tsx`, `.js`, `.jsx`), Go (`.go`).
- **JSON Schema:**

```
{
  "type": "object",
  "properties": {
    "path": {
      "type": "string",
      "description": "Caminho relativo do arquivo no workspace (ex: 'crates/souls_core/src/fs.rs'). Finais de linha devem ser estritamente LF."
    },
    "include_private": {
      "type": "boolean",
      "description": "Se verdadeiro, preserva declarações internas e privadas. Padrão: false.",
      "default": false
    }
  },
  "required": ["path"],
  "additionalProperties": false
}
```

#### 3.1.2 `symbol` (alias: `souls_ast_slice`)

- **Descrição:** Localiza cirurgicamente um nó sintático específico na árvore concreta do Tree-sitter com base em uma consulta de identificador ou caminho qualificado, retornando seu corpo integral.
- **JSON Schema:**

```
{
  "type": "object",
  "properties": {
    "path": {
      "type": "string",
      "description": "Caminho relativo do arquivo de código."
    },
    "symbol_query": {
      "type": "string",
      "description": "Identificador ou caminho qualificado a extrair (ex: 'SoulsMemoryStore::persist_turn' ou 'canonical_lance_schema')."
    }
  },
  "required": ["path", "symbol_query"],
  "additionalProperties": false
}
```

#### 3.1.3 `callers` & `callees`

- **Descrição:** Traça o grafo de dependências sintáticas no workspace, revelando quem invoca uma função/struct (`callers`) ou quais chamadas internas são efetuadas por ela (`callees`).
- **JSON Schema Comum:**

```
{
  "type": "object",
  "properties": {
    "symbol_name": {
      "type": "string",
      "description": "Nome da função ou método a ser inspecionado no grafo."
    },
    "file_scope": {
      "type": "string",
      "description": "Caminho opcional do arquivo para desambiguação de símbolos homônimos."
    }
  },
  "required": ["symbol_name"],
  "additionalProperties": false
}
```

#### 3.1.4 `delta_diff` (alias: `souls_myers_diff`)

- **Descrição:** Computa a diferença mínima estruturada (Myers Diff) entre o arquivo em disco e um novo conteúdo proposto, validando integridade de fechamento sintático antes da aplicação.
- **JSON Schema:**

```
{
  "type": "object",
  "properties": {
    "path": {
      "type": "string",
      "description": "Caminho relativo do arquivo de destino."
    },
    "new_content": {
      "type": "string",
      "description": "Novo conteúdo integral proposto (estritamente com quebras de linha LF)."
    }
  },
  "required": ["path", "new_content"],
  "additionalProperties": false
}
```

#### 3.1.5 `edit` & `replace`

- **Descrição:** `edit` aplica substituições atômicas de blocos exatos com travamento assíncrono. `replace` substitui intervalos semânticos inteiros com rollback automático se a validação sintática falhar.
- **JSON Schema (`edit`):**

```
{
  "type": "object",
  "properties": {
    "path": { "type": "string", "description": "Arquivo de destino." },
    "old_string": { "type": "string", "description": "Trecho exato a ser localizado e substituído." },
    "new_string": { "type": "string", "description": "Novo trecho a ser inserido." }
  },
  "required": ["path", "old_string", "new_string"],
  "additionalProperties": false
}
```

### 3.2 Namespace `context.*` (Compressão LEAN & Token Shields)

#### 3.2.1 `smart_read` & `read`

- **Descrição:** `smart_read` inspeciona a contagem de tokens na CPU via `tiktoken` antes de emitir a leitura. Se o arquivo ultrapassar o limite seguro, aplica compressão LEAN adaptativa. `read` devolve o arquivo formatado sob a convenção TOON/SymbolMap.
- **JSON Schema (`smart_read`):**

```
{
  "type": "object",
  "properties": {
    "path": { "type": "string", "description": "Caminho do arquivo." },
    "max_tokens": {
      "type": "integer",
      "description": "Limite máximo de tokens antes de acionar compressão. Padrão: 4000.",
      "default": 4000
    }
  },
  "required": ["path"],
  "additionalProperties": false
}
```

#### 3.2.2 `compress`, `dedup` & `fill`

- **Descrição:** Pipeline de desidratação e reidratação em memória RAM:
    - `compress`: Achata código e documentação removendo ruído de sintaxe.
    - `dedup`: Converte repetições de código idêntico em marcadores estruturais.
    - `fill`: Reidrata instantaneamente (294 $\mu$s) marcadores CCR para o texto integral usando tabelas de dispersão na RAM.
- **JSON Schema (`fill`):**

```
{
  "type": "object",
  "properties": {
    "marker_hash": {
      "type": "string",
      "description": "Hash hexadecimal do bloco compactado retornado previamente pelo Souls Engine."
    }
  },
  "required": ["marker_hash"],
  "additionalProperties": false
}
```

#### 3.2.3 `headroom_retrieve` _(UPGRADE CANDIDATE)_

- **Racional de Investigação:** No legado, atuava como stub de recuperação CCR. Na v7, sua lógica deve ser reativada para conectar o buffer volátil de compressão da crate `souls_core` com os slots de contexto do Hermes Agent, evitando recomposição de histórico repetido.

### 3.3 Namespace `memory.*` (Tríade Hipocampal L3 & Grafo LadybugDB)

#### 3.3.1 `semantic_search` (alias: `souls_memory_recall`)

- **Descrição:** Recuperação profunda através de busca híbrida com Fusão Recíproca de Ranqueamento (RRF, $k=60$), combinando busca léxica BM25 (FrankenSQLite FTS5) e cosseno no LanceDB. Os nós resultantes passam pela barreira ontológica do grafo LadybugDB em RAM.
- **JSON Schema:**

```
{
  "type": "object",
  "properties": {
    "query": { "type": "string", "description": "Consulta conceitual, técnica ou léxica." },
    "limit": { "type": "integer", "description": "Quantidade máxima de registros a retornar. Padrão: 5.", "default": 5 },
    "partition_filter": {
      "type": "string",
      "enum": ["ALL", "STABLE", "EVOLVING"],
      "description": "Filtro de partição ontológica. Padrão: 'ALL'.",
      "default": "ALL"
    }
  },
  "required": ["query"],
  "additionalProperties": false
}
```

#### 3.3.2 `sqlite_query`

- **Descrição:** Executa instruções SQL `SELECT` read-only ultra-seguras contra o banco `souls_state.db`. O interpretador valida a AST SQL para barrar qualquer operação de mutação (`INSERT`, `UPDATE`, `DELETE`, `DROP`).
- **JSON Schema:**

```
{
  "type": "object",
  "properties": {
    "query": { "type": "string", "description": "Instrução SQL SELECT estruturada (tabelas em modo STRICT)." }
  },
  "required": ["query"],
  "additionalProperties": false
}
```

#### 3.3.3 Operações do Grafo Cognitivo LadybugDB (`mem_*`)

- **`mem_create_entities`:** Registra novos nós ontológicos (`name`, `entity_type`, `metadata_json`).
- **`mem_create_relations`:** Registra arestas com semântica estrita: `depends_on`, `conflicts_with`, `implements`, `relates_to`.
- **`mem_add_observations`:** Anexa fatos empíricos a nós com sincronização automática e atômica com o índice FTS5.
- **`mem_open_nodes`:** Realiza leitura direcionada de uma entidade, trazendo seu histórico de observações e arestas conectadas.
- **`mem_read_graph`:** Retorna a topologia do grafo ativo (sujeito a `LIMIT 500` para proteger a memória da IDE).
- **JSON Schema (`mem_create_relations`):**

```
{
  "type": "object",
  "properties": {
    "source": { "type": "string", "description": "Nome da entidade de origem." },
    "target": { "type": "string", "description": "Nome da entidade de destino." },
    "relationship": {
      "type": "string",
      "enum": ["depends_on", "conflicts_with", "implements", "relates_to"],
      "description": "Semântica estrita da relação lógica."
    }
  },
  "required": ["source", "target", "relationship"],
  "additionalProperties": false
}
```

#### 3.3.4 `knowledge` & `handoff`

- **Descrição:** `knowledge` persiste diretrizes epistêmicas e regras de governança permanentes. `handoff` transfere o estado e variáveis de execução para subagentes criados pelo Hermes via `delegate_task`.

### 3.4 Namespace `think.*` (Cognição Socrática BMAD/DAG)

#### 3.4.1 `thinking` (alias: `core_think`)

- **Descrição:** Espaço socrático de raciocínio multi-passos estruturado como Grafo Acíclico Dirigido (DAG). Permite desdobrar hipóteses, criar ramificações (`branchFromThought`) e registrar revisões formais de pensamento (`isRevision`), aceitando parâmetros em `snake_case` e `camelCase`.
- **JSON Schema:**

```
{
  "type": "object",
  "properties": {
    "thought": { "type": "string", "description": "Conteúdo analítico do passo de raciocínio atual." },
    "thought_number": { "type": "integer", "description": "Índice sequencial do pensamento atual (1, 2, ...)." },
    "total_thoughts": { "type": "integer", "description": "Estimativa projetada do total de passos necessários." },
    "next_thought_needed": { "type": "boolean", "description": "Indica se o raciocínio requer desdobramentos adicionais." },
    "is_revision": { "type": "boolean", "description": "Indica se este passo retifica uma hipótese anterior." },
    "revises_thought": { "type": "integer", "description": "Índice do pensamento retificado (obrigatório se is_revision for true)." },
    "branch_from_thought": { "type": "integer", "description": "Ponto de bifurcação caso esteja explorando uma hipótese alternativa." }
  },
  "required": ["thought", "thought_number", "total_thoughts", "next_thought_needed"],
  "additionalProperties": false
}
```

#### 3.4.2 `session`, `analyze_session`, `export_session` & `merge_sessions`

- **Descrição:** Conjunto para gerenciar o ciclo de vida do encadeamento de raciocínio. `analyze_session` avalia se há incoerências lógicas entre passos; `export_session` consolida a deliberação em formato estruturado; `merge_sessions` funde ramos concorrentes sob consistência eventual.

#### 3.4.3 `intent` _(UPGRADE CANDIDATE)_

- **Racional de Investigação:** No legado, atuava como stub de ambiguidade. No Souls Engine v7, deve ser conectado à crate `souls_inference_runtime` para utilizar o **Tier 0 (ModernBERT/GLiClass)** e o **Tier 0.5 (Gemma-2-2B CPU Logit Probing)** para calcular a entropia de uma requisição e detectar injeções de prompt antes de a inferência principal ser disparada.

### 3.5 Namespace `ops.*` & `sys.*` (Observabilidade, FinOps & Sistema)

#### 3.5.1 `route_subtask` (alias: `souls_route_subtask`)

- **Descrição:** Consulta o motor bayesiano multiobjetivo ParetoBandit para recomendar o Tier ótimo de execução, endpoint e hiperparâmetros para subtarefas delegadas (`delegate_task`) e slots auxiliares (`auxiliary.*`). Integra a barreira termodinâmica $\Phi(T_{\text{GPU}}, V_{\text{livre}}, k)$ da RTX 2060m.
- **JSON Schema:**

```
{
  "type": "object",
  "properties": {
    "task_description": { "type": "string", "description": "Descrição sucinta do objetivo da subtarefa." },
    "estimated_input_tokens": { "type": "integer", "description": "Contagem projetada de tokens de entrada." },
    "estimated_output_tokens": { "type": "integer", "description": "Contagem projetada de tokens de saída." },
    "requires_code_generation": { "type": "boolean", "description": "Indica se a subtarefa demanda síntese de código." },
    "complexity_tier_hint": {
      "type": "string",
      "enum": ["TRIVIAL", "STANDARD", "COMPLEX", "REFLECTIVE"],
      "default": "STANDARD"
    }
  },
  "required": ["task_description", "estimated_input_tokens", "estimated_output_tokens", "requires_code_generation"],
  "additionalProperties": false
}
```

#### 3.5.2 `repo_heatmap` & `repo_impact`

- **Descrição:**
    - `repo_heatmap`: Calcula o ranking de calor e Frecency $F(a) = \sum \text{Changes} \cdot e^{-\lambda \Delta t}$ inspecionando commits locais via biblioteca bare-metal `gitoxide` (`gix`).
    - `repo_impact`: Traça o raio de impacto (_Blast Radius_) de uma modificação de arquivo via travessia reversa em largura (BFS) sobre o grafo de dependências AST.
- **JSON Schema (`repo_impact`):**

```
{
  "type": "object",
  "properties": {
    "path": { "type": "string", "description": "Caminho do arquivo a ter seu impacto analisado." },
    "depth": { "type": "integer", "description": "Profundidade máxima da busca reversa. Padrão: 3.", "default": 3 }
  },
  "required": ["path"],
  "additionalProperties": false
}
```

#### 3.5.3 `metrics` & `feedback` _(UPGRADE CANDIDATE para `metrics`)_

- **Descrição:**
    - `feedback`: Extrai registros de execução, auditoria de latência e pontuação $E^3$ acumulada no SQLite.
    - `metrics`: Fornece telemetria viva de taxa de acerto de _prefix cache_ dos provedores e status da GPU via NVML. Deve ser conectado ao despachante de telemetria da v7.

#### 3.5.4 Token Shields de Sistema: `tree`, `search`, `shell`, `web_search`, `fetch_web`

- **Descrição:**
    - `tree`: Lente de diretórios com dot-flattening e exclusão forçada de pastas de compilação (`target/`, `node_modules/`, `.git/`).
    - `search`: Regex de alta performance com exclusão automática de binários e formatação LEAN agrupada.
    - `shell`: Executor de comandos com filtro sanitizador de sequências ANSI e truncamento defensivo de saídas longas.
    - `web_search` & `fetch_web`: Busca na web via DuckDuckGo e conversão direta de HTML para Markdown na memória RAM, poupando tokens de DOM.

#### 3.5.5 `routes` _(UPGRADE CANDIDATE)_

- **Racional de Investigação:** No legado, mapeava canais IPC entre Tauri e Svelte. No Souls Engine v7, sua função é inspecionar e expor a tabela viva de rotas do servidor Axum (`/mcp`, `/api/v1/*`), portas ativas e estado dos ganchos SSE, permitindo diagnóstico imediato pelo operador ou pelo Hermes.

## 4. ENVELOPES CANÔNICOS DE TRANSPORTE JSON-RPC 2.0

Para erradicar discrepâncias de serialização entre o runtime Python do Hermes Agent e a camada Axum/Stdio em Rust, os envelopes canônicos de comunicação MCP seguem estritamente as definições abaixo.

### 4.1 Envelope de Requisição de Execução (`tools/call`)

```
{
  "jsonrpc": "2.0",
  "id": "req_001",
  "method": "tools/call",
  "params": {
    "name": "outline",
    "arguments": {
      "path": "crates/souls_core/src/mpsc_flusher.rs",
      "include_private": false
    }
  }
}
```

### 4.2 Envelope de Resposta com Sucesso

O campo `result.content` deve conter compulsoriamente um array de blocos estruturados, sendo o tipo primário `"text"` contendo o payload serializado ou string pura:

```
{
  "jsonrpc": "2.0",
  "id": "req_001",
  "result": {
    "content": [
      {
        "type": "text",
        "text": "// Language: Rust | Lines: 24 | Dehydrated: 81.2%\n\npub struct MpscBatchBuffer<T> { ... }"
      }
    ],
    "isError": false
  }
}
```

### 4.3 Envelope de Erro Padronizado

Caso ocorra uma falha tratada ou exceção de domínio:

```
{
  "jsonrpc": "2.0",
  "id": "req_001",
  "error": {
    "code": -32001,
    "message": "FileTooLargeForAST: O arquivo excede o limite físico de análise de 2 MB.",
    "data": {
      "path": "crates/souls_memory/src/huge_dump.rs",
      "size_bytes": 3145728,
      "max_allowed_bytes": 2097152
    }
  }
}
```

## 5. TABELA CANÔNICA DE CÓDIGOS DE ERRO DE DOMÍNIO

Qualquer exceção capturada pelas barreiras `catch_unwind` da crate `souls_core` ou validadores de esquema deve ser mapeada para os seguintes códigos de domínio fixos (faixa `-32000` a `-32099` reservada para extensões de aplicação do JSON-RPC):

| **Código**   | **Identificador de Erro**     | **Causa Técnica Primária**                                                                                   | **Ação Recomendada para o Agente / IDE**                                |
| ------------ | ----------------------------- | ------------------------------------------------------------------------------------------------------------ | ----------------------------------------------------------------------- |
| **`-32000`** | `InternalEnginePanic`         | Pânico recuperado na thread Tokio via `catch_unwind`.                                                        | Registrar falha crítica; não retentar sem alterar parâmetros.           |
| **`-32001`** | `FileTooLargeForAST`          | Arquivo físico $> 2\text{ MB}$, excedendo teto de parsing na RAM.                                            | Recuar de modo transparente para leitura de blocos (`smart_read`).      |
| **`-32002`** | `VramThermalThrottled`        | Watchdog NVML disparou barreira: $T \ge 82^\circ\text{C}$ ou $\text{VRAM}_{\text{livre}} \le 800\text{ MB}$. | Desviar subtarefa para Tier 3 (Nuvem Fast) ou processamento em CPU.     |
| **`-32003`** | `SymbolNotFound`              | O identificador fornecido em `symbol_query` não foi achado na AST.                                           | Invocar `outline` para verificar a grafia exata dos símbolos.           |
| **`-32004`** | `AstGrammarParseError`        | Erro fatal na gramática Tree-sitter decorrente de código quebrado.                                           | Inspecionar arquivo manualmente ou tentar `delta_diff` parcial.         |
| **`-32005`** | `OntologicalBarrierViolation` | LadybugDB detectou conflito com regra `STABLE` via aresta `conflicts_with`.                                  | Descartar memória candidata; ela viola diretrizes do projeto.           |
| **`-32006`** | `InvalidInputParameters`      | Violação de tipagem, parâmetro obrigatório ausente ou string vazia.                                          | Ajustar os parâmetros conforme o JSON Schema formal da ferramenta.      |
| **`-32007`** | `RefsPathLeakViolation`       | Caminho fornecido tenta escapar da partição `Z:\` via `..` ou aponta para `C:\`.                             | Corrigir caminho relativo para residir estritamente no workspace `Z:`.  |
| **`-32008`** | `GitoxideRepositoryError`     | Falha ao inspecionar o repositório Git bare-metal via biblioteca `gix`.                                      | Verificar se o diretório `.git` está acessível e não corrompido.        |
| **`-32009`** | `SqlStrictViolation`          | Violação de tipo de dado ou instrução de escrita proibida em `sqlite_query`.                                 | Executar estritamente consultas `SELECT` aderentes ao DDL STRICT.       |
| **`-32010`** | `ToolNotFoundInNamespace`     | Ferramenta solicitada não existe ou não pertence ao namespace ativado.                                       | Invocar `tool_search` para verificar o catálogo e nomes de ferramentas. |

## 6. LINHAS VERMELHAS E ANTI-PATTERNS NAS FERRAMENTAS MCP

1. **PROIBIÇÃO DE ALTERAÇÃO DE NOMES DE CHAVES:** É estritamente proibido renomear qualquer campo de entrada nos JSON Schemas (ex: renomear `path` para `file_path`, ou `symbol_query` para `query`). Isso quebra a integração com o Hermes Agent e clientes de IDE sem aviso prévio.
2. **PROIBIÇÃO DE PAYLOADS CRLF:** Toda ferramenta que devolve código-fonte ou texto estruturado (`outline`, `symbol`, `delta_diff`, `read`) deve higienizar o payload, garantindo que todas as quebras de linha sejam exclusivamente `\n` (LF).
3. **PROIBIÇÃO DE LOGS NO STDOUT EM MODO STDIO:** Nenhuma crate pode imprimir mensagens via `println!` ou logs de depuração para o descritor `stdout` quando operando via stdio. Toda a observabilidade deve ser direcionada para o buffer MPSC ou descritor `stderr` formatado.
4. **PROIBIÇÃO DE I/O BLOQUEANTE NO DISCO FORA DO BUFFER:** Ferramentas MCP não devem ler ou gravar arquivos sem passar pelas salvaguardas da crate `souls_core` e validações de caminho em `Z:\`.
5. **PROIBIÇÃO DE TELEMETRIA SÍNCRONA:** O registro de métricas de invocação de ferramentas deve ser emitido via canal assíncrono Tokio MPSC para esvaziamento agregado a cada 5 segundos no SQLite, sem travar o ciclo de resposta da ferramenta.
6. **DISCIPLINA DE RESPOSTA DAS FERRAMENTAS RESTRINGIDAS:** Ferramentas marcadas como `UPGRADE_CANDIDATE` não devem ser removidas da árvore de código; devem ser investigadas e transplantadas com as adaptações da arquitetura v7.