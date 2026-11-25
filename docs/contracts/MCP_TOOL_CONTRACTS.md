# MCP_TOOL_CONTRACTS.md — Catálogo Canônico Exaustivo e Contratos de Ferramentas MCP (`mcp@7.2.0`)

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

Para resolver o inchaço de contexto **sem amputar o silício**, o Souls Engine adota a **Revelação Progressiva de Ferramentas (Progressive Tool Disclosure)**:

```
┌───────────────────────────────────────────────────────────────────────────────────────────┐
│                    NÍVEL 0: BOOT COMPACTO (Sempre Visível — 8 Tools)                      │
│  [outline] [symbol] [read] [edit] [thinking] [route_subtask] [tool_search] [tool_dispatch]│
└─────────────────────────────────────────────┬─────────────────────────────────────────────┘
                                              │
                    Invoca 'tool_search(namespace)' ou 'tool_dispatch(tool, args)'
                                              ▼
┌───────────────────────────────────────────────────────────────────────────────────────────┐
│                    NÍVEL 1: CATÁLOGOS DINÂMICOS SOB DEMANDA                               │
│   code.*     ──► AST Tree-sitter, Callers/Callees, Myers Diff                             │
│   context.*  ──► Compressão LEAN, CCR De/Rehydration, Multi-Read                          │
│   memory.*   ──► Grafo LadybugDB, Observações, SQLite STRICT, LanceDB                     │
│   think.*    ──► Raciocínio Socrático DAG, Sessões, Intent Probing                        │
│   ops.*      ──► Heatmap gitoxide, Impact/Blast Radius, Hardware & FinOps                 │
│   google.*   ──► Google Personal & Life Hub (Gmail, Calendar, Drive, Docs, Sheets, etc.)  │
│   inbox.*    ──► BMAD Agent Inbox com Myers Diff e Gravity Lock                           │
└───────────────────────────────────────────────────────────────────────────────────────────┘
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

Abaixo consta a relação exaustiva de todas as ferramentas operacionais do Souls Engine (`mcp@7.2.0`), sua maturidade clínica, latência média no silício e crate Rust de ancoragem:

| **#** | **Garra (tool_name)**         | **Namespace** | **O Que Faz no Sistema**                                                                                | **Crate Executora**       | **Maturidade Clínica** | **Latência Típica** |
| ----- | ----------------------------- | ------------- | ------------------------------------------------------------------------------------------------------- | ------------------------- | ---------------------- | ------------------- |
| —     | **`tool_search`**             | `core`        | **Meta-Ferramenta:** Busca e retorna esquemas detalhados de ferramentas por namespace ou palavra-chave. | `souls_server`            | `LIVE_PRODUCTION`      | **< 1.0 ms**        |
| —     | **`tool_dispatch`**           | `core`        | **Meta-Ferramenta:** Despacho universal para clientes MCP estritos (alias `dispatch`), permitindo invocar qualquer ferramenta Nível 1 sem injeção estática de schemas. | `souls_server` | `LIVE_PRODUCTION` | **< 1.0 ms** |
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
| 16    | **`headroom_retrieve`**       | `context`     | Recupera blocos de cache comprimido diretamente da RAM (Loopback CCR).                                  | `souls_core`              | `LIVE_PRODUCTION`      | **0.27 ms**         |
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
| 50    | **`routes`**                  | `sys`         | Inspeciona rotas ativas do daemon Axum e status de ganchos IPC/SSE.                                     | `souls_server`            | `UPGRADE_CANDIDATE`    | **< 2.0 ms**        |
| 51    | **`cloud_champions`**         | `ops`         | Consulta os modelos campeões da Fronteira de Pareto por perfil (Executive, Coding, etc.) da Cloud Arena.| `souls_server`            | `LIVE_PRODUCTION`      | **< 2.0 ms**        |
| 52    | **`cloud_arena_sync`**        | `ops`         | Dispara sincronização sob demanda de tarifas e benchmarks do Nous Portal e OpenRouter.                  | `souls_server`            | `LIVE_PRODUCTION`      | **~ 1.2 s**         |

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

### 3.0.1 Meta-Ferramenta: `tool_dispatch` / `dispatch` (Despacho Universal em Clientes MCP Estritos)

#### Descrição Operacional

Criada na versão `v0.9.2+3` em resposta à necessidade de clientes MCP estritos (como o runtime Python do Hermes Agent, cujo validador de chamadas checa o nome da tool contra a lista estática recebida no handshake de boot).

A meta-ferramenta `tool_dispatch` (com o alias canônico `dispatch`) reside permanentemente no **Nível 0 (Boot)**. Ela permite ao agente invocar qualquer uma das 78 ferramentas especializadas sob revelação progressiva (Nível 1) diretamente através do cliente MCP padrão, contornando a validação estática no host sem a necessidade de expor todos os esquemas JSON de uma só vez (economizando ~20k tokens de contexto e preservando o *prefix caching* de >95%).

#### JSON Schema de Entrada (`inputSchema`)

```json
{
  "type": "object",
  "properties": {
    "tool": {
      "type": "string",
      "description": "Nome canônico da ferramenta MCP ou seu alias registrado a ser despachada (ex: 'google_gmail', 'semantic_search', 'repo_impact', 'outline')."
    },
    "arguments": {
      "type": "object",
      "description": "Objeto JSON contendo os argumentos requeridos pela ferramenta alvo conforme seu inputSchema descoberto via tool_search."
    }
  },
  "required": ["tool"],
  "additionalProperties": false
}
```

#### Guardrails & Propriedades de Execução

1. **Trava Anti-Recursão Mandatória:** Se a ferramenta alvo for `tool_dispatch` ou `dispatch`, a chamada é abortada imediatamente com erro JSON-RPC código `-32602` (*Invalid params*): `"Recursion detected: cannot dispatch tool_dispatch recursively"`.
2. **Normalização de Alias:** A string informada em `tool` é normalizada através do mapa de aliases do Souls Engine (ex: `dispatch` $\rightarrow$ `tool_dispatch`, `google_gmail` $\rightarrow$ `google_gmail`, `outline` $\rightarrow$ `outline`), garantindo resolução determinística.
3. **Contenção no Dev Drive ReFS:** Qualquer argumento com caminhos de arquivo passa pela validação de fronteira, impedindo travessia para fora de `Z:\souls_engine\`.
4. **Despacho Direto ao Registro Interno:** A chamada é encaminhada em memória para o dispatcher axum/mcp nativo do Rust, executando com a mesma latência ultrabaixa (<1ms de overhead de roteamento) de uma invocação direta.

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

#### 3.5.6 Cloud Arena & Inteligência de Mercado: `cloud_champions` & `cloud_arena_sync` (ADR-009)

- **`cloud_champions`:**
  - **Descrição:** Retorna os modelos líderes da Fronteira de Pareto nos perfis cognitivos canônicos (Executive/Master Brain, Coding, Agentic, Reasoning, Fast, Design, etc.). Consumido pelo Hermes Agent para calibrar a configuração do modelo default e por subagentes para seleção de especialistas.
  - **JSON Schema (`inputSchema`):**
    ```json
    {
      "type": "object",
      "properties": {
        "profile": {
          "type": "string",
          "enum": ["all", "executive", "coding", "agentic", "reasoning", "fast", "design", "research", "security", "curation"],
          "default": "all",
          "description": "Perfil cognitivo alvo para filtrar os campeões de nuvem."
        },
        "date": {
          "type": "string",
          "description": "Data específica do snapshot no formato YYYY-MM-DD (padrão: hoje)."
        }
      },
      "additionalProperties": false
    }
    ```

- **`cloud_arena_sync`:**
  - **Descrição:** Dispara sincronização em segundo plano das tabelas de tarifas e benchmarks do Nous Research Portal e OpenRouter.
  - **JSON Schema (`inputSchema`):**
    ```json
    {
      "type": "object",
      "properties": {
        "force": {
          "type": "boolean",
          "default": false,
          "description": "Se verdadeiro, força a sincronização mesmo se já executada hoje."
        },
        "date": {
          "type": "string",
          "description": "Data específica para registrar a avaliação no formato YYYY-MM-DD."
        }
      },
      "additionalProperties": false
    }
    ```

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
6. **DISCIPLINA DE RESPOSTA DAS FERRAMENTAS RESTRINGIDAS:** Ferramentas marcadas como `UPGRADE_CANDIDATE` não devem ser removidas da árvore de código; devem ser investigadas e transplantadas com as adaptações da arquitetura canônica.

---

## 7. EXPANSÃO CANÔNICA DA MILESTONE v0.9.2 — GOOGLE PERSONAL & LIFE HUB E BMAD AGENT INBOX

> **CATÁLOGO TOTAL CONSOLIDADO (v0.9.2):** **80 Ferramentas Canônicas** (**7** Boot Level 0 + **73** Progressivas sob demanda) + **16 Skills Canônicas** no Two-Stage Semantic Switchboard (`souls_server::mcp::switchboard`).  
> **REFERÊNCIA ARQUITETURAL:** [ADR-030](file:///Z:/souls_engine/docs/decisions/adrs/ADR-030-Google-Personal-Life-Hub-BMAD-Inbox-e-Hibrido-Seguro.md) • Implementação: `crates/souls_server/src/mcp/tools_google.rs`, `crates/souls_server/src/mcp/registry.rs`.

### 7.1 Distribuição Atualizada dos 11 Namespaces (80 Tools)

| **Namespace** | **Qtd** | **Observação de Evolução** |
| :--- | :---: | :--- |
| `core` | **1** | `tool_search` (Boot Level 0; `namespace` enum expandido para incluir `research`, `ui`, `google`, `inbox`) |
| `code` | **8** | `outline` e `symbol` em Boot Level 0 |
| `context` | **8** | `read` em Boot Level 0 |
| `memory` | **14** | Memória L2/L3 + grafo LadybugDB + subagentes/handoff |
| `think` | **6** | `thinking` em Boot Level 0 |
| `ops` | **9** | `route_subtask` em Boot Level 0; inclui `cloud_champions`, `cloud_arena_sync`, `agentic_dispatch` |
| `sys` | **10** | Inclui `tree`, `search`, `shell`, `execute`, `fetch_web`, `web_search`, `routes`, etc. |
| `research` | **4** | `research_claim`, `research_event`, `research_steering`, `research_complete` (v0.7.0) |
| `ui` | **2** | `ui_inspect`, `ui_interact` (v0.8.3 AXTree LEAN) |
| `google` | **15** | `notebooklm_rpc` (v0.8.2) + **14 novas ferramentas Life Hub (v0.9.2)** |
| `inbox` | **3** | **3 novas ferramentas BMAD Agent Inbox (v0.9.2)** |
| **TOTAL** | **80** | **7 Boot Level 0 + 73 Progressivas** |

Atualização do enum `namespace` em `tool_search` (`registry.rs`):
```json
"enum": ["all", "code", "context", "memory", "think", "ops", "sys", "research", "ui", "google", "inbox"]
```

### 7.2 As 14 Novas Ferramentas do Namespace `google.*` (v0.9.2)

Todas as ferramentas abaixo possuem `is_level_0: false` (descobertas via `tool_search(namespace="google")` ou Switchboard) e executam governança BMAD (`BlastRadiusTier`, `SessionTaintTracker`, `wrap_untrusted_content`).

#### 7.2.1 `google_accounts`
- **Descrição:** Gerencia contas Google no cofre encriptado GV2, executa o Google Doctor, importa credenciais `gws` e ajusta políticas BMAD.
- **JSON Schema (`inputSchema`):**
```json
{
  "type": "object",
  "properties": {
    "action": { "type": "string", "enum": ["list", "doctor", "import_gws", "set_policy", "upsert", "delete"], "default": "list" },
    "account_id": { "type": "string" },
    "email": { "type": "string" },
    "label": { "type": "string" },
    "kind": { "type": "string", "enum": ["personal_gmail", "workspace_enterprise", "secondary_sandbox"] },
    "policy": { "type": "string", "enum": ["read_only", "drafts_and_tasks_only", "balanced_hotl_hitl", "whitelist_write"] },
    "is_primary": { "type": "boolean" },
    "write_whitelist_ids": { "type": "array", "items": { "type": "string" } }
  },
  "additionalProperties": false
}
```

#### 7.2.2 `google_briefing`
- **Descrição:** Gera o Daily Briefing unificado (Agenda Calendar, Blocos Deep Focus, Trânsito Routes v2, Gmail Prioritário, Tasks e Inbox BMAD) com degradação graciosa offline.
- **JSON Schema (`inputSchema`):**
```json
{
  "type": "object",
  "properties": {
    "account_id": { "type": "string", "description": "ID ou email da conta Google opcional." }
  },
  "additionalProperties": false
}
```

#### 7.2.3 `google_gmail`
- **Descrição:** Triagem determinística de e-mails Gmail, leitura com delimitação anti-injection, criação de rascunhos, rótulos em lote e envio via BMAD Inbox.
- **JSON Schema (`inputSchema`):**
```json
{
  "type": "object",
  "properties": {
    "action": { "type": "string", "enum": ["triage", "read", "draft", "bulk", "send"], "default": "triage" },
    "account_id": { "type": "string" },
    "query": { "type": "string" },
    "max_results": { "type": "integer" },
    "message_id": { "type": "string" },
    "to": { "type": "string" },
    "subject": { "type": "string" },
    "body": { "type": "string" },
    "ids": { "type": "array", "items": { "type": "string" } },
    "add_labels": { "type": "array", "items": { "type": "string" } },
    "remove_labels": { "type": "array", "items": { "type": "string" } },
    "session_id": { "type": "string" },
    "api_base_override": { "type": "string" }
  },
  "additionalProperties": false
}
```

#### 7.2.4 `google_calendar`
- **Descrição:** Consulta agenda Google Calendar, encontra blocos de foco profundo (Deep Focus Slots) e cria/remove eventos com Taint Guard BMAD.
- **JSON Schema (`inputSchema`):**
```json
{
  "type": "object",
  "properties": {
    "action": { "type": "string", "enum": ["list", "focus_slots", "create", "delete"], "default": "list" },
    "account_id": { "type": "string" },
    "time_min": { "type": "string" },
    "time_max": { "type": "string" },
    "work_start": { "type": "string" },
    "work_end": { "type": "string" },
    "min_duration_mins": { "type": "integer" },
    "summary": { "type": "string" },
    "start_rfc3339": { "type": "string" },
    "end_rfc3339": { "type": "string" },
    "location": { "type": "string" },
    "attendees": { "type": "array", "items": { "type": "string" } },
    "send_updates": { "type": "string" },
    "event_id": { "type": "string" },
    "session_id": { "type": "string" },
    "api_base_override": { "type": "string" }
  },
  "additionalProperties": false
}
```

#### 7.2.5 `google_drive`
- **Descrição:** Pesquisa arquivos no Google Drive, inspeciona metadados, baixa para exchange sandbox ou propõe compartilhamento/exclusão via BMAD Inbox.
- **JSON Schema (`inputSchema`):**
```json
{
  "type": "object",
  "properties": {
    "action": { "type": "string", "enum": ["search", "metadata", "download_sandbox", "share", "delete"], "default": "search" },
    "account_id": { "type": "string" },
    "query": { "type": "string" },
    "page_size": { "type": "integer" },
    "file_id": { "type": "string" },
    "filename": { "type": "string" },
    "role": { "type": "string" },
    "perm_type": { "type": "string" },
    "email_address": { "type": "string" },
    "session_id": { "type": "string" },
    "api_base_override": { "type": "string" }
  },
  "additionalProperties": false
}
```

#### 7.2.6 `google_docs_scratchpad`
- **Descrição:** Rascunho incremental por linhas para Google Docs (`load`, `insert`, `replace`, `delete`, `render`) e `push` via Myers Diff no BMAD Agent Inbox.
- **JSON Schema (`inputSchema`):**
```json
{
  "type": "object",
  "properties": {
    "action": { "type": "string", "enum": ["load", "insert", "replace", "delete", "render", "push"], "default": "load" },
    "doc_id": { "type": "string" },
    "title": { "type": "string" },
    "at_line": { "type": "integer" },
    "start_line": { "type": "integer" },
    "end_line": { "type": "integer" },
    "lines": { "type": "array", "items": { "type": "string" } },
    "remote_baseline": { "type": "string" },
    "account_id": { "type": "string" },
    "session_id": { "type": "string" },
    "api_base_override": { "type": "string" }
  },
  "required": ["doc_id"],
  "additionalProperties": false
}
```

#### 7.2.7 `google_sheets`
- **Descrição:** Leitura e análise financeira determinística (totais, categorias, anomalias >2x média, projeção 30d) ou escrita via BMAD Inbox em Google Sheets.
- **JSON Schema (`inputSchema`):**
```json
{
  "type": "object",
  "properties": {
    "action": { "type": "string", "enum": ["analyze", "read", "write"], "default": "analyze" },
    "spreadsheet_id": { "type": "string" },
    "range": { "type": "string" },
    "rows": { "type": "array", "items": { "type": "array", "items": { "type": "string" } } },
    "values": { "type": "array", "items": { "type": "array" } },
    "account_id": { "type": "string" },
    "session_id": { "type": "string" },
    "api_base_override": { "type": "string" }
  },
  "required": ["spreadsheet_id", "range"],
  "additionalProperties": false
}
```

#### 7.2.8 `google_slides`
- **Descrição:** Planejamento estruturado (5 a 15 slides) e criação assistida de apresentações Google Slides.
- **JSON Schema (`inputSchema`):**
```json
{
  "type": "object",
  "properties": {
    "action": { "type": "string", "enum": ["plan", "create"], "default": "plan" },
    "topic": { "type": "string" },
    "briefing": { "type": "string" },
    "target_slides": { "type": "integer" },
    "account_id": { "type": "string" },
    "session_id": { "type": "string" },
    "api_base_override": { "type": "string" }
  },
  "required": ["topic"],
  "additionalProperties": false
}
```

#### 7.2.9 `google_tasks`
- **Descrição:** Gerencia Google Tasks, executa Obsidian Quick Capture (fallback soberano para Google Keep em contas pessoais) e checa elegibilidade Keep Enterprise.
- **JSON Schema (`inputSchema`):**
```json
{
  "type": "object",
  "properties": {
    "action": { "type": "string", "enum": ["list", "create", "complete", "quick_capture", "check_keep_eligibility"], "default": "list" },
    "account_id": { "type": "string" },
    "tasklist_id": { "type": "string" },
    "show_completed": { "type": "boolean" },
    "title": { "type": "string" },
    "notes": { "type": "string" },
    "due_rfc3339": { "type": "string" },
    "task_id": { "type": "string" },
    "body": { "type": "string" },
    "tags": { "type": "array", "items": { "type": "string" } },
    "session_id": { "type": "string" },
    "api_base_override": { "type": "string" }
  },
  "additionalProperties": false
}
```

#### 7.2.10 `google_contacts`
- **Descrição:** Busca contatos via Google People API e detecta aniversários próximos.
- **JSON Schema (`inputSchema`):**
```json
{
  "type": "object",
  "properties": {
    "action": { "type": "string", "enum": ["search", "upcoming_birthdays"], "default": "search" },
    "query": { "type": "string" },
    "page_size": { "type": "integer" },
    "within_days": { "type": "integer" },
    "contacts": { "type": "array", "items": { "type": "object" } },
    "account_id": { "type": "string" },
    "session_id": { "type": "string" },
    "api_base_override": { "type": "string" }
  },
  "additionalProperties": false
}
```

#### 7.2.11 `google_commute`
- **Descrição:** Calcula rota com tráfego ao vivo (Google Maps Routes API v2) e gera alerta proativo de horário de saída.
- **JSON Schema (`inputSchema`):**
```json
{
  "type": "object",
  "properties": {
    "origin": { "type": "string" },
    "destination": { "type": "string" },
    "event_id": { "type": "string" },
    "event_title": { "type": "string" },
    "event_start_rfc3339": { "type": "string" },
    "buffer_mins": { "type": "integer" },
    "api_key": { "type": "string" },
    "api_base_override": { "type": "string" }
  },
  "required": ["origin", "destination"],
  "additionalProperties": false
}
```

#### 7.2.12 `google_youtube`
- **Descrição:** Extrai e cacheia transcrições de vídeos YouTube (Innertube + parser XML/SRV3 puro em Rust) com delimitação anti-injection.
- **JSON Schema (`inputSchema`):**
```json
{
  "type": "object",
  "properties": {
    "url_or_id": { "type": "string" },
    "preferred_lang": { "type": "string", "default": "en" },
    "session_id": { "type": "string" },
    "api_base_override": { "type": "string" }
  },
  "required": ["url_or_id"],
  "additionalProperties": false
}
```

#### 7.2.13 `google_notebooklm`
- **Descrição:** Gerencia cadernos Google NotebookLM, fontes Drive (`check_freshness`/`sync_drive`), RAG query, Studio Audio Overview e Mapas Mentais.
- **JSON Schema (`inputSchema`):**
```json
{
  "type": "object",
  "properties": {
    "action": { "type": "string", "enum": ["list", "get", "add_source", "check_freshness", "sync_drive", "query", "studio_create", "studio_status", "mindmap"], "default": "list" },
    "notebook_id": { "type": "string" },
    "source_id": { "type": "string" },
    "source_url": { "type": "string" },
    "query": { "type": "string" },
    "prompt": { "type": "string" },
    "arguments": { "type": "array" },
    "session_id": { "type": "string" },
    "endpoint_override": { "type": "string" }
  },
  "additionalProperties": false
}
```

#### 7.2.14 `google_gems`
- **Descrição:** Gerencia catálogo local de Custom Gems e despacha prompts via Gemini API oficial (bloqueando `StreamGenerate` em contas primárias).
- **JSON Schema (`inputSchema`):**
```json
{
  "type": "object",
  "properties": {
    "action": { "type": "string", "enum": ["list", "upsert", "delete", "dispatch", "secondary_stream_generate"], "default": "list" },
    "id": { "type": "string" },
    "name": { "type": "string" },
    "description": { "type": "string" },
    "system_instruction": { "type": "string" },
    "model": { "type": "string" },
    "backend": { "type": "string" },
    "remote_gem_id": { "type": "string" },
    "prompt": { "type": "string" },
    "account_id": { "type": "string" },
    "api_key": { "type": "string" },
    "base_url": { "type": "string" },
    "model_hash": { "type": "string" }
  },
  "additionalProperties": false
}
```

### 7.3 As 3 Novas Ferramentas do Namespace `inbox.*` (BMAD Agent Inbox)

#### 7.3.1 `inbox_list`
- **Descrição:** Lista propostas pendentes ou históricas no BMAD Agent Inbox (com contagem de pendências e redação de tokens Gravity Lock).
- **JSON Schema (`inputSchema`):**
```json
{
  "type": "object",
  "properties": {
    "status": { "type": "string", "enum": ["all", "pending", "approved", "executed", "rejected", "failed", "expired"], "default": "all" },
    "limit": { "type": "integer", "default": 50 }
  },
  "additionalProperties": false
}
```

#### 7.3.2 `inbox_inspect_diff`
- **Descrição:** Inspeciona o Myers Diff completo, snapshots `before`/`after`, nível de risco e status de taint de uma proposta no BMAD Agent Inbox.
- **JSON Schema (`inputSchema`):**
```json
{
  "type": "object",
  "properties": {
    "proposal_id": { "type": "string" }
  },
  "required": ["proposal_id"],
  "additionalProperties": false
}
```

#### 7.3.3 `inbox_propose`
- **Descrição:** Enfileira uma proposta de mutação externa no BMAD Agent Inbox computando Myers Diff real via `souls_ast` e herdando estado de Taint Guard da sessão.
- **JSON Schema (`inputSchema`):**
```json
{
  "type": "object",
  "properties": {
    "account_id": { "type": "string" },
    "service": { "type": "string" },
    "action": { "type": "string" },
    "title": { "type": "string" },
    "summary": { "type": "string" },
    "tier": { "type": "string", "enum": ["tier0_read_only", "tier1_safe_reversible", "tier2_external_destructive"] },
    "target_resource": { "type": "string" },
    "before_snapshot": { "type": "string" },
    "after_snapshot": { "type": "string" },
    "myers_diff": { "type": "string" },
    "payload_json": { "type": "object" },
    "ttl_secs": { "type": "integer" },
    "session_id": { "type": "string" }
  },
  "required": ["service", "action", "title", "target_resource"],
  "additionalProperties": false
}
```

### 7.4 Registro da 16ª Skill Canônica no Switchboard (`google-life-hub`)

Em `crates/souls_server/src/mcp/switchboard.rs` (`CANONICAL_SKILLS`), a 16ª skill canônica é registrada com roteamento direto para o domínio Google/Inbox:
- **Identificador:** `google-life-hub`
- **Arquivo de Playbook:** `.agents/skills/google-life-hub/SKILL.md`
- **Primary Tools:** `google_briefing`, `google_gmail`, `google_calendar`, `google_docs_scratchpad`, `google_notebooklm`, `inbox_propose`, `inbox_list`
- **Keywords de Afinidade Semântica:** `gmail`, `email`, `calendar`, `agenda`, `drive`, `docs`, `sheets`, `slides`, `tasks`, `keep`, `contacts`, `maps`, `commute`, `youtube`, `transcript`, `notebooklm`, `podcast`, `audio`, `gems`, `inbox`, `bmad`, `diff`, `google`

---

## 8. EXPANSÃO CANÔNICA DA MILESTONE v0.9.2.1 — PULSO PROATIVO, APPS SCRIPT 24/7, FORMS, CHAT WORKSPACE E MINIONS DEEP ANALYZE

> **CATÁLOGO TOTAL CONSOLIDADO (v0.9.2.1):** **85 Ferramentas Canônicas** (**7** Boot Level 0 + **78** Progressivas sob demanda) + **16 Skills Canônicas** no Two-Stage Semantic Switchboard (`souls_server::mcp::switchboard`).  
> **REFERÊNCIA ARQUITETURAL:** [ADR-031](file:///Z:/souls_engine/docs/decisions/adrs/ADR-031-Google-Life-Hub-v0921-Proatividade-Apps-Script-Forms-MinionS.md) • Implementação: `crates/souls_server/src/mcp/tools_google.rs`, `crates/souls_server/src/mcp/registry.rs`.

### 8.1 Distribuição Consolidada dos 11 Namespaces (85 Tools = `62 + 20 + 3`)

| **Grupo / Namespace** | **Qtd** | **Observação de Evolução** |
| :--- | :---: | :--- |
| Demais namespaces (`core` 1, `code` 8, `context` 8, `memory` 14, `think` 6, `ops` 9, `sys` 10, `research` 4, `ui` 2) | **62** | **7** ferramentas em Boot Level 0 (`tool_search`, `outline`, `symbol`, `read`, `thinking`, `route_subtask`, e 1 adicional conforme `registry.rs`) |
| `google` | **20** | `notebooklm_rpc` (v0.8.2) + 14 tools Life Hub (v0.9.2) + **5 novas tools v0.9.2.1** (`google_pulse`, `google_apps_script`, `google_forms`, `google_chat`, `google_deep_analyze`) |
| `inbox` | **3** | `inbox_list`, `inbox_inspect_diff`, `inbox_propose` (v0.9.2) |
| **TOTAL** | **85** | **7 Boot Level 0 + 78 Progressivas** |

### 8.2 As 5 Novas Ferramentas do Namespace `google.*` (v0.9.2.1)

Todas as 5 ferramentas abaixo possuem `is_level_0: false` e respeitam a governança BMAD (`BlastRadiusTier`, `SessionTaintTracker`, `wrap_untrusted_content`, `AgentInboxStore` com Myers Diff + Gravity Lock).

#### 8.2.1 `google_pulse`
- **Descrição:** Pulso Proativo Google 24/7: status multi-cadência, disparo sob demanda, eventos pendentes, reconhecimento, configuração de Quiet Hours e vínculos Drive <-> NotebookLM.
- **JSON Schema (`inputSchema`):**
```json
{
  "type": "object",
  "properties": {
    "action": { "type": "string", "enum": ["status", "run_now", "pending", "ack", "configure", "sync_links_list", "sync_links_add", "sync_links_toggle"], "default": "status" },
    "limit": { "type": "integer", "default": 3 },
    "drain": { "type": "boolean", "default": true },
    "event_ids": { "type": "array", "items": { "type": "string" } },
    "config": { "type": "object" },
    "webhook_url": { "type": "string" },
    "telegram_bot_token": { "type": "string" },
    "telegram_chat_id": { "type": "string" },
    "id": { "type": "string" },
    "notebook_id": { "type": "string" },
    "notebook_title": { "type": "string" },
    "source_id": { "type": "string" },
    "drive_file_id": { "type": "string" },
    "drive_file_name": { "type": "string" },
    "auto_sync_enabled": { "type": "boolean" },
    "enabled": { "type": "boolean" }
  },
  "additionalProperties": false
}
```

#### 8.2.2 `google_apps_script`
- **Descrição:** Automações 24/7 no Google Apps Script: templates prontos, criação de projeto, inspeção estática de segredos/permissões em português e propostas push/deploy/run com Myers Diff no BMAD Agent Inbox.
- **JSON Schema (`inputSchema`):**
```json
{
  "type": "object",
  "properties": {
    "action": { "type": "string", "enum": ["templates", "create", "get_content", "scan_security", "propose_push", "propose_deploy", "propose_run", "processes"], "default": "templates" },
    "title": { "type": "string" },
    "parent_id": { "type": "string" },
    "template_id": { "type": "string" },
    "script_id": { "type": "string" },
    "files": { "type": "array" },
    "previous_files": { "type": "array" },
    "description": { "type": "string" },
    "function_name": { "type": "string" },
    "parameters": { "type": "array" },
    "dev_mode": { "type": "boolean" },
    "account_id": { "type": "string" },
    "session_id": { "type": "string" },
    "api_base_override": { "type": "string" }
  },
  "additionalProperties": false
}
```

#### 8.2.3 `google_forms`
- **Descrição:** Google Forms API v1: leitura de esquema, criação governada via BMAD Agent Inbox, listagem de respostas e estatísticas determinísticas com resumo Markdown.
- **JSON Schema (`inputSchema`):**
```json
{
  "type": "object",
  "properties": {
    "action": { "type": "string", "enum": ["get", "create", "responses", "stats"], "default": "get" },
    "form_id": { "type": "string" },
    "title": { "type": "string" },
    "document_title": { "type": "string" },
    "requests": {},
    "account_id": { "type": "string" },
    "session_id": { "type": "string" },
    "api_base_override": { "type": "string" }
  },
  "additionalProperties": false
}
```

#### 8.2.4 `google_chat`
- **Descrição:** Google Chat API v1 (exclusivo Workspace corporativo para Spaces/Messages) + propostas de envio via Webhook ou Espaço governadas pelo BMAD Agent Inbox.
- **JSON Schema (`inputSchema`):**
```json
{
  "type": "object",
  "properties": {
    "action": { "type": "string", "enum": ["check_eligibility", "list_spaces", "list_messages", "propose_send", "propose_webhook"], "default": "check_eligibility" },
    "space_name": { "type": "string" },
    "webhook_url": { "type": "string" },
    "text": { "type": "string" },
    "page_size": { "type": "integer", "default": 20 },
    "account_id": { "type": "string" },
    "session_id": { "type": "string" },
    "api_base_override": { "type": "string" },
    "allow_local_test_webhook": { "type": "boolean" }
  },
  "additionalProperties": false
}
```

#### 8.2.5 `google_deep_analyze`
- **Descrição:** Leitura econômica MinionS em documentos Google, PDFs (até 25MB), threads Gmail, Formulários e textos extensos usando Tier 0 BM25 + cache local e escalonamento opcional para SLM local.
- **JSON Schema (`inputSchema`):**
```json
{
  "type": "object",
  "properties": {
    "source_kind": { "type": "string", "enum": ["drive_doc", "drive_pdf", "gmail_thread", "google_form", "raw_text"], "default": "raw_text" },
    "source_ref": { "type": "string" },
    "query": { "type": "string" },
    "text": { "type": "string" },
    "mode": { "type": "string", "enum": ["auto", "fast_deterministic", "force_local_slm"], "default": "auto" },
    "top_k": { "type": "integer", "default": 8 },
    "account_id": { "type": "string" },
    "session_id": { "type": "string" },
    "api_base_override": { "type": "string" }
  },
  "required": ["query"],
  "additionalProperties": false
}
```

### 8.3 Atualização da Skill `google-life-hub` no Switchboard (`switchboard.rs`)

Em `crates/souls_server/src/mcp/switchboard.rs` (`CANONICAL_SKILLS`), a 16ª skill `google-life-hub` foi expandida na v0.9.2.1 para rotear também as 5 novas capacidades:
- **Identificador:** `google-life-hub`
- **Descrição:** `"Life Hub pessoal e profissional Google (Gmail, Calendar, Drive, Docs, Sheets, Slides, Tasks, Contacts, Maps, YouTube, NotebookLM, Gems, Pulso 24/7, Apps Script, Forms, Chat Workspace e Leitura Econômica MinionS) com governança BMAD Agent Inbox e Myers Diff."`
- **Primary Tools (12):** `google_briefing`, `google_pulse`, `google_gmail`, `google_calendar`, `google_docs_scratchpad`, `google_apps_script`, `google_forms`, `google_chat`, `google_deep_analyze`, `google_notebooklm`, `inbox_propose`, `inbox_list`
- **Keywords de Afinidade Semântica:** `gmail`, `email`, `calendar`, `agenda`, `drive`, `docs`, `sheets`, `slides`, `tasks`, `keep`, `contacts`, `maps`, `commute`, `youtube`, `transcript`, `notebooklm`, `podcast`, `audio`, `gems`, `inbox`, `bmad`, `diff`, `google`, `pulse`, `proativo`, `heartbeat`, `apps script`, `automacao`, `clasp`, `forms`, `formulario`, `chat`, `minions`, `analise profunda`, `auto sync`, `drive notebook`

---

## 9. MILESTONE v0.9.3 — PERCEPÇÃO MULTIMODAL HÍBRIDA, ÁUDIO SOBERANO PT-BR & RAPPORT POLIVAGAL (`91 Tools & 17 Skills`)

> **Versão:** `v0.9.3` (ADR-033)
> **Total de Ferramentas Canônicas:** **91 ferramentas** (**8** visíveis no Boot Nível 0 com `tool_dispatch` + **83** sob revelação progressiva via `tool_search`)
> **Total de Skills no Switchboard:** **17 skills canônicas** (incluindo `souls-multimodal-perception`)

### 9.1 Resumo do Namespace `ui.*` Expandido (`7 Tools`)

| Ferramenta | Modo / Novidade v0.9.3 | Descrição Canônica |
| :--- | :--- | :--- |
| `ui_inspect` | **Atualizada** (`window_filter`, `intent_filter`, captura live) | Extrai e serializa a árvore de acessibilidade (CDP ou Windows ao vivo) em LEAN dot-notation com diagnóstico de blind-spot e `0 MB VRAM`. |
| `ui_interact` | **Atualizada** (resolução ao vivo) | Calcula coordenadas centrais determinísticas e resolve o nó tátil sob o cursor `(x, y)`. |
| `ui_snapshot` | **Nova** | Captura snapshot visual sob demanda com marcas Set-of-Mark (SoM), detecção de blind-spot, deduplicação SHA-256 e escalonamento opcional. |
| `ui_diff` | **Nova** | Computa diff estrutural LEAN entre `previous_tree` e `after_tree` (ou captura ao vivo se omitida) poupando tokens. |
| `voice_transcribe` | **Nova** | Transcreve áudio Base64 ou avalia `sample_text` com detecção de emoção SenseVoice, filtro de alucinação e veredito PCOMP (`0 MB VRAM`). |
| `voice_synthesize` | **Nova** | Sintetiza fala em português brasileiro na CPU (`0 MB VRAM`) com modelagem prosódica Camada 3 e adaptação ao modo cognitivo. |
| `voice_rapport` | **Nova** | Consulta ou atualiza o perfil de rapport linguístico, formalidade, cadência e modo cognitivo de voz (`UserRapportProfileDto`). |

### 9.2 Contratos JSON Schema das Ferramentas `ui.*` (`crates/souls_server/src/mcp/registry.rs`)

#### `ui_inspect`
```json
{
  "type": "object",
  "properties": {
    "window_filter": { "type": "string", "description": "Filtro opcional de título da janela" },
    "intent_filter": { "type": "string", "description": "Filtro de intenção: all, actionable, inputs, buttons" },
    "include_click_coords": { "type": "boolean", "default": true, "description": "Incluir coordenadas geométricas de clique" },
    "max_depth": { "type": "integer", "description": "Profundidade máxima de serialização da árvore" },
    "cursor_x": { "type": "number", "description": "Coordenada X opcional do cursor" },
    "cursor_y": { "type": "number", "description": "Coordenada Y opcional do cursor" }
  },
  "additionalProperties": true
}
```

#### `ui_interact`
```json
{
  "type": "object",
  "properties": {
    "x": { "type": "number", "description": "Coordenada X na tela" },
    "y": { "type": "number", "description": "Coordenada Y na tela" }
  },
  "required": ["x", "y"],
  "additionalProperties": true
}
```

#### `ui_snapshot`
```json
{
  "type": "object",
  "properties": {
    "window_filter": { "type": "string", "description": "Filtro opcional da janela alvo" },
    "region": { "type": "array", "items": { "type": "integer" }, "description": "Região [x, y, width, height]" },
    "mode": { "type": "string", "enum": ["auto", "deterministic_som_only", "local_ephemeral_vlm", "cloud_frontier"], "default": "auto" },
    "session_id": { "type": "string", "description": "ID de sessão para deduplicação SHA-256" },
    "include_base64": { "type": "boolean", "default": false, "description": "Incluir PNG em Base64 quando a tela mudar" },
    "prompt": { "type": "string", "description": "Pergunta visual opcional sobre o snapshot" }
  },
  "additionalProperties": false
}
```

#### `ui_diff`
```json
{
  "type": "object",
  "properties": {
    "previous_tree": { "type": "object", "description": "Árvore AXNode de referência anterior" },
    "after_tree": { "type": "object", "description": "Árvore AXNode posterior opcional (captura ao vivo se omitida)" },
    "window_filter": { "type": "string", "description": "Filtro opcional de janela para captura ao vivo" },
    "intent_filter": { "type": "string", "description": "Filtro de intenção opcional" }
  },
  "required": ["previous_tree"],
  "additionalProperties": false
}
```

#### `voice_transcribe`
```json
{
  "type": "object",
  "properties": {
    "audio_base64": { "type": "string", "description": "Áudio codificado em Base64" },
    "filename": { "type": "string", "description": "Nome ou extensão do arquivo (ex: audio.wav)" },
    "language": { "type": "string", "default": "pt", "description": "Idioma alvo (padrão: pt)" },
    "sample_text": { "type": "string", "description": "Texto de amostra opcional para avaliação PCOMP/SenseVoice sem áudio" },
    "silence_duration_ms": { "type": "integer", "default": 450, "description": "Duração do silêncio final em ms" },
    "user_id": { "type": "string", "default": "default", "description": "Identificador do usuário para EMA de rapport" }
  },
  "additionalProperties": false
}
```

#### `voice_synthesize`
```json
{
  "type": "object",
  "properties": {
    "input": { "type": "string", "description": "Texto em português para sintetizar" },
    "cognitive_mode": { "type": "string", "enum": ["auto", "hands_on", "sparring", "decompression", "crisis_paralysis", "philosophical"] },
    "apply_prosody_shaping": { "type": "boolean", "default": true },
    "voice": { "type": "string" },
    "user_id": { "type": "string", "default": "default" }
  },
  "required": ["input"],
  "additionalProperties": false
}
```

#### `voice_rapport`
```json
{
  "type": "object",
  "properties": {
    "action": { "type": "string", "enum": ["get", "set"], "default": "get" },
    "user_id": { "type": "string", "default": "default" },
    "active_cognitive_mode": { "type": "string", "enum": ["auto", "hands_on", "sparring", "decompression", "crisis_paralysis", "philosophical"] },
    "formality_score": { "type": "number" },
    "preferred_pacing": { "type": "string" },
    "contemplative_base_ms": { "type": "integer" },
    "mirroring_ratio": { "type": "number" },
    "backchannel_enabled": { "type": "boolean" },
    "slang_lexicon": { "type": "array", "items": { "type": "string" } }
  },
  "additionalProperties": false
}
```

### 9.3 17ª Skill Canônica no Switchboard (`switchboard.rs`)

- **Identificador:** `souls-multimodal-perception`
- **Descrição:** `"Percepção multimodal 0 MB VRAM: Visão Cognitiva AXTree LEAN, SoM & Snapshot sob demanda e Voz Soberana PT-BR (PCOMP, SenseVoice, prosódia e rapport)."`
- **Primary Tools (7):** `ui_inspect`, `ui_interact`, `ui_snapshot`, `ui_diff`, `voice_transcribe`, `voice_synthesize`, `voice_rapport`
- **Keywords de Afinidade Semântica:** `vision`, `screen`, `axtree`, `snapshot`, `canvas`, `voice`, `audio`, `tts`, `stt`, `speech`, `voz`, `fala`, `tela`, `pt-br`, `rapport`, `tdah`, `2e`

---

## 10. MILESTONE v0.9.4 — SOULS EVOLUTIVE HARNESS (`97 Tools & 18 Skills`, incl. Hardening `0.9.4+3`)

> **Versão:** `v0.9.4` / `engine@0.9.4+3` (ADR-034)
> **Total de Ferramentas Canônicas:** **97 ferramentas** (**8** visíveis no Boot Nível 0 com `tool_dispatch` + **89** sob revelação progressiva via `tool_search`)
> **Total de Skills no Switchboard:** **18 skills canônicas** (incluindo `souls-evolutive-harness`)

### 10.1 Resumo das Ferramentas `harness_*` (`6 Tools`)

| Ferramenta | Namespace | Descrição Canônica |
| :--- | :---: | :--- |
| `harness_status` | `ops` | Retorna integridade SHA-256 de `c1` (incluindo `drift_detected`, `resolved_path` e `checked_at`), fibras Cordis ativas/revertidas, estado e gates Catnap, Tombstones, ranking WeEvolve e habilidades em Quarentena. |
| `harness_catnap_pulse` | `ops` | Avalia os gates de ociosidade/GPU/orçamento e executa uma micro-fatia cooperativa (`< 1 ms` preempção) auditando `actor`, `trigger_mode` e `authorization.provenance` no `catnap_consolidation_ledger` e `harness_audit_log.jsonl`. |
| `harness_cordis_create` | `ops` | Cria e monta uma fibra tática Cordis (suportando `ephemeral: true` e `ttl_secs` em `0.9.4+3`) para exercitar rollback LIFO `< 1 ms` sem afetar proteções persistentes. |
| `harness_cordis_rollback` | `ops` | Desmonta uma fibra tática Cordis (`c2`/`c5`/`c7`/`c8`) em ordem estritamente LIFO em `< 1 ms` (sem penalidade ELO quando `ephemeral: true`). |
| `harness_tombstone_resolve` | `memory` | Varre (`scan`), aplica (`apply`), lista (`list`) ou restaura em 1 clique (`restore`) Tombstones lógicos sobre memórias contraditórias sem deletar `STABLE`. |
| `harness_ralph_crystallize` | `ops` | Cristaliza uma solução aprendida em `SKILL.md` normalizada em LF na Quarentena do Skills Hub após Scanner Zero-Trust e cálculo Myers Diff. |

### 10.2 Contratos JSON Schema (`crates/souls_server/src/mcp/registry.rs`)

#### `harness_status`
```json
{
  "type": "object",
  "properties": {
    "include_index": { "type": "boolean", "default": false, "description": "Incluir a tabela completa INDEX.md da biblioteca de experiências." }
  },
  "additionalProperties": false
}
```

#### `harness_catnap_pulse`
```json
{
  "type": "object",
  "properties": {
    "slice_kind": { "type": "string", "enum": ["shadow_probe_ppc", "ladybug_reconcile", "langevin_micro_batch", "diary_distillation", "ralph_crystallize"] },
    "force_manual": { "type": "boolean", "default": true },
    "actor": { "type": "string", "description": "Identificador do acionador humano/IDE para auditoria." },
    "authorization": { "type": "object", "description": "Envelope opcional com provenance { channel, timestamp, scope, actor }." }
  },
  "additionalProperties": false
}
```

#### `harness_cordis_create`
```json
{
  "type": "object",
  "properties": {
    "title_pt": { "type": "string" },
    "cue_pattern": { "type": "string" },
    "rule_instruction_pt": { "type": "string" },
    "slot_code": { "type": "string", "enum": ["c2", "c5", "c7", "c8"], "default": "c7" },
    "target_task_type": { "type": "string" },
    "ttl_secs": { "type": "integer", "description": "TTL em segundos para expiração automática." },
    "ephemeral": { "type": "boolean", "default": true, "description": "Se true, permanece só em memória e permite rollback LIFO sem penalidade ELO." }
  },
  "required": ["title_pt", "cue_pattern", "rule_instruction_pt"],
  "additionalProperties": false
}
```

#### `harness_cordis_rollback`
```json
{
  "type": "object",
  "properties": {
    "fiber_id": { "type": "string" },
    "authorization": { "type": "object", "description": "Envelope opcional com provenance para auditoria." }
  },
  "required": ["fiber_id"],
  "additionalProperties": false
}
```

#### `harness_tombstone_resolve`
```json
{
  "type": "object",
  "properties": {
    "action": { "type": "string", "enum": ["scan", "apply", "restore", "list"], "default": "scan" },
    "memory_id": { "type": "string" },
    "superseded_memory_id": { "type": "string" },
    "superseded_by_memory_id": { "type": "string" },
    "tombstone_id": { "type": "string" },
    "reason_pt": { "type": "string" },
    "limit": { "type": "integer", "default": 30 }
  },
  "additionalProperties": false
}
```

#### `harness_ralph_crystallize`
```json
{
  "type": "object",
  "properties": {
    "title_pt": { "type": "string" },
    "domain": { "type": "string", "default": "ops" },
    "problem_pattern_pt": { "type": "string" },
    "resolution_playbook": { "type": "string" },
    "source_session_id": { "type": "string" }
  },
  "required": ["title_pt", "problem_pattern_pt", "resolution_playbook"],
  "additionalProperties": false
}
```

### 10.3 18ª Skill Canônica no Switchboard (`switchboard.rs`)

- **Identificador:** `souls-evolutive-harness`
- **Descrição:** `"Auto-evolução segura (ADR-034): pinagem SHA-256 de c1, microkernel Cordis reversível, cochilos Catnap (<1ms preempção), Tombstones lógicos e Ralph Loop."`
- **Primary Tools (6):** `harness_status`, `harness_catnap_pulse`, `harness_cordis_create`, `harness_cordis_rollback`, `harness_tombstone_resolve`, `harness_ralph_crystallize`

### 10.4 Refinos de Contrato na Release `0.9.4+3`
- **`route_subtask`:** `structuredContent` e o texto distinguem explicitamente `active_context_window_tokens` (`Context Window: <N> tokens`) de `thinking_budget_tokens` (`Thinking Budget: <M> tokens`).
- **`mem_create_entities`:** Rejeita nomes em prosa longa (`> 8` palavras), URLs (`http://`/`https://`), quebras de linha ou caracteres de controle com `-32007 InvalidInputParameters` e aceita `cleanup_residue: true` para expurgo idempotente no LadybugDB (também disponível em `POST /api/v1/memory/ladybug/cleanup-residue`).
- **Proveniência Estrita em Mutações HTTP (`authorization.provenance`):** `POST /api/v1/harness/pins/resign` (com suporte seletivo a `paths`) e `POST /api/v1/harness/ralph/approve` retornam `400 Bad Request` caso `authorization.provenance` (`channel`, `timestamp > 0`, `scope`) esteja ausente.
- **Keywords de Afinidade Semântica:** `harness`, `evolve`, `evolucao`, `auto-evolucao`, `catnap`, `cordis`, `rollback`, `tombstone`, `ralph`, `skill`, `quarantine`, `weevolve`, `elo`, `c1`, `pin`, `idle`, `ocio`



