//! # Canonical MCP Tool Registry (`souls_server::mcp::registry`)
//!
//! Exposes all 55 canonical MCP tools, input schemas, namespaces, and Progressive
//! Tool Disclosure (Level 0 boot compact vs Level 1 dynamic discovery).

use serde_json::{json, Value};
use std::sync::OnceLock;

/// Canonical metadata for a registered MCP tool.
#[derive(Debug, Clone)]
pub struct ToolMeta {
    pub name: &'static str,
    pub namespace: &'static str,
    pub description: &'static str,
    pub is_level_0: bool,
    pub input_schema: fn() -> Value,
}

static ALL_TOOLS: OnceLock<Vec<ToolMeta>> = OnceLock::new();

/// Returns the static repository of all 55 tools.
pub fn get_canonical_tools() -> &'static [ToolMeta] {
    ALL_TOOLS.get_or_init(build_canonical_tools)
}

fn build_canonical_tools() -> Vec<ToolMeta> {
    vec![
        // Meta-Tool (Core / Level 0)
        ToolMeta {
            name: "tool_search",
            namespace: "core",
            description: "Descoberta dinâmica e progressiva de ferramentas por namespace ou query.",
            is_level_0: true,
            input_schema: || json!({
                "type": "object",
                "properties": {
                    "namespace": {
                        "type": "string",
                        "enum": ["all", "code", "context", "memory", "think", "ops", "sys", "research", "ui", "google", "inbox"],
                        "description": "Namespace funcional para listar ferramentas."
                    },
                    "query": {
                        "type": "string",
                        "description": "Termo de busca conceitual ou funcional."
                    }
                },
                "additionalProperties": false
            }),
        },
        ToolMeta {
            name: "tool_dispatch",
            namespace: "core",
            description: "Despacha e executa qualquer uma das ferramentas canônicas do Souls Engine por nome e argumentos. Permite que clientes MCP estritos invoquem ferramentas de Level 1 sem restrição de schema estático no boot.",
            is_level_0: true,
            input_schema: || json!({
                "type": "object",
                "properties": {
                    "tool": {
                        "type": "string",
                        "description": "Nome canônico da ferramenta a executar (ex: google_briefing, semantic_search, knowledge, google_gmail, etc)."
                    },
                    "arguments": {
                        "type": "object",
                        "description": "Argumentos JSON específicos da ferramenta alvo (consulte via tool_search)."
                    }
                },
                "required": ["tool"],
                "additionalProperties": false
            }),
        },
        // code.* namespace
        ToolMeta {
            name: "outline",
            namespace: "code",
            description: "Extrai assinaturas públicas e tipos desidratando corpos de funções via AST.",
            is_level_0: true,
            input_schema: || json!({
                "type": "object",
                "properties": {
                    "path": { "type": "string", "description": "Caminho relativo do arquivo no workspace." },
                    "include_private": { "type": "boolean", "default": false, "description": "Se verdadeiro, preserva declarações internas." }
                },
                "required": ["path"],
                "additionalProperties": false
            }),
        },
        ToolMeta {
            name: "symbol",
            namespace: "code",
            description: "Localiza cirurgicamente um nó sintático específico na AST concreta.",
            is_level_0: true,
            input_schema: || json!({
                "type": "object",
                "properties": {
                    "path": { "type": "string", "description": "Caminho relativo do arquivo de código." },
                    "symbol_query": { "type": "string", "description": "Identificador ou caminho qualificado a extrair." }
                },
                "required": ["path", "symbol_query"],
                "additionalProperties": false
            }),
        },
        ToolMeta {
            name: "callers",
            namespace: "code",
            description: "Mapeia quem invoca um determinado símbolo em todo o workspace.",
            is_level_0: false,
            input_schema: || json!({
                "type": "object",
                "properties": {
                    "symbol_name": { "type": "string", "description": "Nome da função ou método a inspecionar." },
                    "file_scope": { "type": "string", "description": "Caminho opcional do arquivo para desambiguação." }
                },
                "required": ["symbol_name"],
                "additionalProperties": false
            }),
        },
        ToolMeta {
            name: "callees",
            namespace: "code",
            description: "Mapeia quais funções e métodos são consumidos internamente pelo símbolo.",
            is_level_0: false,
            input_schema: || json!({
                "type": "object",
                "properties": {
                    "symbol_name": { "type": "string", "description": "Nome da função ou método a inspecionar." },
                    "file_scope": { "type": "string", "description": "Caminho opcional do arquivo para desambiguação." }
                },
                "required": ["symbol_name"],
                "additionalProperties": false
            }),
        },
        ToolMeta {
            name: "get_ast",
            namespace: "code",
            description: "Extrai a árvore sintática concreta (CST/AST) em formato JSON estruturado.",
            is_level_0: false,
            input_schema: || json!({
                "type": "object",
                "properties": {
                    "path": { "type": "string", "description": "Caminho relativo do arquivo de código." }
                },
                "required": ["path"],
                "additionalProperties": false
            }),
        },
        ToolMeta {
            name: "edit",
            namespace: "code",
            description: "Aplica edições cirúrgicas Search/Replace com travamento e validação.",
            is_level_0: true,
            input_schema: || json!({
                "type": "object",
                "properties": {
                    "path": { "type": "string", "description": "Arquivo de destino." },
                    "old_string": { "type": "string", "description": "Trecho exato a ser localizado e substituído." },
                    "new_string": { "type": "string", "description": "Novo bloco de substituição." }
                },
                "required": ["path", "old_string", "new_string"],
                "additionalProperties": false
            }),
        },
        ToolMeta {
            name: "replace",
            namespace: "code",
            description: "Substitui blocos estruturais de código com rollback atômico em falha sintática.",
            is_level_0: false,
            input_schema: || json!({
                "type": "object",
                "properties": {
                    "path": { "type": "string", "description": "Arquivo de destino." },
                    "target_block": { "type": "string", "description": "Bloco estrutural a substituir." },
                    "replacement": { "type": "string", "description": "Novo conteúdo a injetar." }
                },
                "required": ["path", "target_block", "replacement"],
                "additionalProperties": false
            }),
        },
        ToolMeta {
            name: "delta_diff",
            namespace: "code",
            description: "Computa Myers Diff estrutural entre versões usando o algoritmo da crate similar.",
            is_level_0: false,
            input_schema: || json!({
                "type": "object",
                "properties": {
                    "path": { "type": "string", "description": "Caminho relativo do arquivo de destino." },
                    "new_content": { "type": "string", "description": "Novo conteúdo integral proposto (LF puro)." }
                },
                "required": ["path", "new_content"],
                "additionalProperties": false
            }),
        },
        // context.* namespace
        ToolMeta {
            name: "read",
            namespace: "context",
            description: "Lê arquivo individual aplicando formatação TOON e SymbolMap de alta densidade.",
            is_level_0: true,
            input_schema: || json!({
                "type": "object",
                "properties": {
                    "path": { "type": "string", "description": "Caminho relativo do arquivo a ser lido." }
                },
                "required": ["path"],
                "additionalProperties": false
            }),
        },
        ToolMeta {
            name: "smart_read",
            namespace: "context",
            description: "Mede tokens via CPU antes da leitura com auto-shrink adaptativo.",
            is_level_0: false,
            input_schema: || json!({
                "type": "object",
                "properties": {
                    "path": { "type": "string", "description": "Arquivo a inspecionar." },
                    "token_budget": { "type": "integer", "description": "Limite máximo de tokens permitidos." }
                },
                "required": ["path"],
                "additionalProperties": false
            }),
        },
        ToolMeta {
            name: "multi_read",
            namespace: "context",
            description: "Leitura concorrente de múltiplos arquivos na RAM com agrupamento CCR.",
            is_level_0: false,
            input_schema: || json!({
                "type": "object",
                "properties": {
                    "paths": { "type": "array", "items": { "type": "string" }, "description": "Lista de caminhos de arquivos." }
                },
                "required": ["paths"],
                "additionalProperties": false
            }),
        },
        ToolMeta {
            name: "compress",
            namespace: "context",
            description: "Aplica compressão LEAN removendo ruído sintático e comentários redundantes.",
            is_level_0: false,
            input_schema: || json!({
                "type": "object",
                "properties": {
                    "text": { "type": "string", "description": "Texto bruto a ser compactado." }
                },
                "required": ["text"],
                "additionalProperties": false
            }),
        },
        ToolMeta {
            name: "dedup",
            namespace: "context",
            description: "Substitui trechos duplicados de 5+ linhas por marcadores estruturais compactos.",
            is_level_0: false,
            input_schema: || json!({
                "type": "object",
                "properties": {
                    "text": { "type": "string", "description": "Texto a deduplicar." }
                },
                "required": ["text"],
                "additionalProperties": false
            }),
        },
        ToolMeta {
            name: "fill",
            namespace: "context",
            description: "Reidrata marcadores de compressão CCR de volta para o texto lossless via hash.",
            is_level_0: false,
            input_schema: || json!({
                "type": "object",
                "properties": {
                    "compressed_text": { "type": "string", "description": "Texto comprimido com marcadores CCR." }
                },
                "required": ["compressed_text"],
                "additionalProperties": false
            }),
        },
        ToolMeta {
            name: "stub_fill",
            namespace: "context",
            description: "Preenche stubs demarcados em arquivos locais validando integridade de fechamento.",
            is_level_0: false,
            input_schema: || json!({
                "type": "object",
                "properties": {
                    "path": { "type": "string", "description": "Caminho do arquivo com stubs." }
                },
                "required": ["path"],
                "additionalProperties": false
            }),
        },
        ToolMeta {
            name: "headroom_retrieve",
            namespace: "context",
            description: "Recupera blocos de cache comprimido diretamente da RAM (Loopback CCR).",
            is_level_0: false,
            input_schema: || json!({
                "type": "object",
                "properties": {
                    "cache_key": { "type": "string", "description": "Identificador do bloco em cache." }
                },
                "required": ["cache_key"],
                "additionalProperties": false
            }),
        },
        // memory.* namespace
        ToolMeta {
            name: "semantic_search",
            namespace: "memory",
            description: "Busca híbrida RRF combinando BM25 (FTS5) e cosseno (LanceDB, 0MB VRAM).",
            is_level_0: false,
            input_schema: || json!({
                "type": "object",
                "properties": {
                    "query": { "type": "string", "description": "Consulta de busca semântica." },
                    "limit": { "type": "integer", "default": 10, "description": "Quantidade máxima de registros." }
                },
                "required": ["query"],
                "additionalProperties": false
            }),
        },
        ToolMeta {
            name: "sqlite_query",
            namespace: "memory",
            description: "Executa consultas SQL read-only ultra-seguras no FrankenSQLite STRICT WAL.",
            is_level_0: false,
            input_schema: || json!({
                "type": "object",
                "properties": {
                    "sql": { "type": "string", "description": "Query SQL estritamente SELECT." }
                },
                "required": ["sql"],
                "additionalProperties": false
            }),
        },
        ToolMeta {
            name: "knowledge",
            namespace: "memory",
            description: "Armazena, versiona e recupera diretrizes epistêmicas no banco L2.",
            is_level_0: false,
            input_schema: || json!({
                "type": "object",
                "properties": {
                    "key": { "type": "string", "description": "Chave da diretriz." },
                    "action": { "type": "string", "enum": ["get", "set", "list"], "description": "Ação desejada." },
                    "content": { "type": "string", "description": "Conteúdo a persistir quando set." }
                },
                "required": ["key", "action"],
                "additionalProperties": false
            }),
        },
        ToolMeta {
            name: "mem_search",
            namespace: "memory",
            description: "Busca lexical FTS5 ultrarrápida sobre o repositório de observações do grafo.",
            is_level_0: false,
            input_schema: || json!({
                "type": "object",
                "properties": {
                    "query": { "type": "string", "description": "Termo a pesquisar nas observações." }
                },
                "required": ["query"],
                "additionalProperties": false
            }),
        },
        ToolMeta {
            name: "mem_open_nodes",
            namespace: "memory",
            description: "Carrega entidades por nome, hidratando observações e conexões ativas.",
            is_level_0: false,
            input_schema: || json!({
                "type": "object",
                "properties": {
                    "names": { "type": "array", "items": { "type": "string" }, "description": "Lista de entidades." }
                },
                "required": ["names"],
                "additionalProperties": false
            }),
        },
        ToolMeta {
            name: "mem_read_graph",
            namespace: "memory",
            description: "Varre a topologia ativa do grafo ontológico LadybugDB.",
            is_level_0: false,
            input_schema: || json!({
                "type": "object",
                "properties": {
                    "limit": { "type": "integer", "default": 100, "description": "Limite máximo de nós." }
                },
                "additionalProperties": false
            }),
        },
        ToolMeta {
            name: "mem_create_entities",
            namespace: "memory",
            description: "Cria entidades idempotentes no grafo LadybugDB via MPSC assíncrono.",
            is_level_0: false,
            input_schema: || json!({
                "type": "object",
                "properties": {
                    "entities": { "type": "array", "items": { "type": "object" }, "description": "Entidades a registrar." }
                },
                "required": ["entities"],
                "additionalProperties": false
            }),
        },
        ToolMeta {
            name: "mem_create_relations",
            namespace: "memory",
            description: "Cria arestas direcionadas e tipadas no grafo ontológico.",
            is_level_0: false,
            input_schema: || json!({
                "type": "object",
                "properties": {
                    "relations": { "type": "array", "items": { "type": "object" }, "description": "Relações entre entidades." }
                },
                "required": ["relations"],
                "additionalProperties": false
            }),
        },
        ToolMeta {
            name: "mem_add_observations",
            namespace: "memory",
            description: "Anexa observações clínicas a nós existentes com sincronização atômica FTS5.",
            is_level_0: false,
            input_schema: || json!({
                "type": "object",
                "properties": {
                    "observations": { "type": "array", "items": { "type": "object" }, "description": "Observações a registrar." }
                },
                "required": ["observations"],
                "additionalProperties": false
            }),
        },
        ToolMeta {
            name: "mem_delete_entities",
            namespace: "memory",
            description: "Expurga entidades e suas arestas associadas sob confirmação HITL.",
            is_level_0: false,
            input_schema: || json!({
                "type": "object",
                "properties": {
                    "entity_names": { "type": "array", "items": { "type": "string" }, "description": "Nomes das entidades." }
                },
                "required": ["entity_names"],
                "additionalProperties": false
            }),
        },
        ToolMeta {
            name: "mem_delete_relations",
            namespace: "memory",
            description: "Remove arestas lógicas específicas no grafo ontológico.",
            is_level_0: false,
            input_schema: || json!({
                "type": "object",
                "properties": {
                    "relations": { "type": "array", "items": { "type": "object" }, "description": "Arestas a remover." }
                },
                "required": ["relations"],
                "additionalProperties": false
            }),
        },
        ToolMeta {
            name: "mem_delete_observations",
            namespace: "memory",
            description: "Exclui observações atômicas com expurgo do índice invertido FTS5.",
            is_level_0: false,
            input_schema: || json!({
                "type": "object",
                "properties": {
                    "deletions": { "type": "array", "items": { "type": "object" }, "description": "Observações a expurgar." }
                },
                "required": ["deletions"],
                "additionalProperties": false
            }),
        },
        ToolMeta {
            name: "handoff",
            namespace: "memory",
            description: "Persiste payloads de troca de turno e contexto entre subagentes delegados.",
            is_level_0: false,
            input_schema: || json!({
                "type": "object",
                "properties": {
                    "handoff_id": { "type": "string", "description": "ID único do handoff." },
                    "from_agent": { "type": "string", "description": "Agente emissor." },
                    "to_agent": { "type": "string", "description": "Agente receptor." },
                    "payload": { "type": "string", "description": "Dados de contexto serializados." }
                },
                "required": ["handoff_id", "from_agent", "to_agent", "payload"],
                "additionalProperties": false
            }),
        },
        ToolMeta {
            name: "sub_agent",
            namespace: "memory",
            description: "Registra e gerencia o ciclo de vida e estado de subagentes autônomos.",
            is_level_0: false,
            input_schema: || json!({
                "type": "object",
                "properties": {
                    "agent_id": { "type": "string", "description": "ID do subagente." },
                    "task_name": { "type": "string", "description": "Nome da tarefa atribuída." },
                    "status": { "type": "string", "description": "Estado atual da execução." }
                },
                "required": ["agent_id", "task_name", "status"],
                "additionalProperties": false
            }),
        },
        // think.* namespace
        ToolMeta {
            name: "thinking",
            namespace: "think",
            description: "Freio cognitivo socrático: hipóteses, branches e revisões em DAG.",
            is_level_0: true,
            input_schema: || json!({
                "type": "object",
                "properties": {
                    "thought": { "type": "string", "description": "Conteúdo do pensamento ou hipótese." },
                    "thought_number": { "type": "integer", "description": "Índice do pensamento." },
                    "total_thoughts": { "type": "integer", "description": "Total estimado de pensamentos." },
                    "next_thought_needed": { "type": "boolean", "description": "Se mais iterações são requeridas." },
                    "sessionId": { "type": "string", "description": "ID opcional da sessão de pensamento." }
                },
                "required": ["thought", "thought_number", "total_thoughts", "next_thought_needed"],
                "additionalProperties": false
            }),
        },
        ToolMeta {
            name: "session",
            namespace: "think",
            description: "Governa a sessão ativa, executando descarte ou reset atômico de caches.",
            is_level_0: false,
            input_schema: || json!({
                "type": "object",
                "properties": {
                    "action": { "type": "string", "enum": ["reset", "inspect", "flush"], "description": "Ação sobre a sessão." }
                },
                "required": ["action"],
                "additionalProperties": false
            }),
        },
        ToolMeta {
            name: "analyze_session",
            namespace: "think",
            description: "Analisa a profundidade lógica e consistência socrática da sessão na RAM.",
            is_level_0: false,
            input_schema: || json!({
                "type": "object",
                "properties": {
                    "session_id": { "type": "string", "description": "ID da sessão socrática." }
                },
                "required": ["session_id"],
                "additionalProperties": false
            }),
        },
        ToolMeta {
            name: "export_session",
            namespace: "think",
            description: "Exporta a árvore de deliberação socrática em Markdown/JSON estruturado.",
            is_level_0: false,
            input_schema: || json!({
                "type": "object",
                "properties": {
                    "session_id": { "type": "string", "description": "ID da sessão a exportar." }
                },
                "required": ["session_id"],
                "additionalProperties": false
            }),
        },
        ToolMeta {
            name: "merge_sessions",
            namespace: "think",
            description: "Funde ramos concorrentes de deliberação cognitiva sob consistência eventual.",
            is_level_0: false,
            input_schema: || json!({
                "type": "object",
                "properties": {
                    "source_session": { "type": "string", "description": "Sessão de origem." },
                    "target_session": { "type": "string", "description": "Sessão de destino." }
                },
                "required": ["source_session", "target_session"],
                "additionalProperties": false
            }),
        },
        ToolMeta {
            name: "intent",
            namespace: "think",
            description: "Avalia ambiguidade informacional via entropia e logit probing (Tier 0/0.5).",
            is_level_0: false,
            input_schema: || json!({
                "type": "object",
                "properties": {
                    "prompt": { "type": "string", "description": "Prompt ou instrução a analisar." }
                },
                "required": ["prompt"],
                "additionalProperties": false
            }),
        },
        // ops.* namespace
        ToolMeta {
            name: "route_subtask",
            namespace: "ops",
            description: "Roteador FinOps ParetoBandit multiobjetivo com barreira termodinâmica Phi.",
            is_level_0: true,
            input_schema: || json!({
                "type": "object",
                "properties": {
                    "task_type": { "type": "string", "description": "Categoria da subtarefa (coding, fast, reasoning, agentic)." },
                    "estimated_tokens": { "type": "integer", "default": 500, "description": "Expectativa de tokens a processar." },
                    "latency_sensitivity": { "type": "number", "default": 0.5, "description": "Sensibilidade à latência (0.0 a 1.0)." }
                },
                "required": ["task_type"],
                "additionalProperties": false
            }),
        },
        ToolMeta {
            name: "repo_heatmap",
            namespace: "ops",
            description: "Mede o calor (Frecency) dos arquivos do workspace via commit graph gitoxide.",
            is_level_0: false,
            input_schema: || json!({
                "type": "object",
                "properties": {
                    "limit": { "type": "integer", "default": 20, "description": "Quantidade máxima de arquivos retornados." }
                },
                "additionalProperties": false
            }),
        },
        ToolMeta {
            name: "repo_impact",
            namespace: "ops",
            description: "Analisa o raio de impacto (Blast Radius) de alterações via BFS reversa.",
            is_level_0: false,
            input_schema: || json!({
                "type": "object",
                "properties": {
                    "target_file": { "type": "string", "description": "Arquivo alterado para cálculo de impacto." }
                },
                "required": ["target_file"],
                "additionalProperties": false
            }),
        },
        ToolMeta {
            name: "feedback",
            namespace: "ops",
            description: "Extrai métricas FinOps históricas (E3, custos por token, latências).",
            is_level_0: false,
            input_schema: || json!({
                "type": "object",
                "properties": {
                    "model_id": { "type": "string", "description": "Identificador do modelo." }
                },
                "additionalProperties": false
            }),
        },
        ToolMeta {
            name: "metrics",
            namespace: "ops",
            description: "Telemetria em tempo real de hardware e conexões ativas do daemon.",
            is_level_0: false,
            input_schema: || json!({
                "type": "object",
                "properties": {},
                "additionalProperties": false
            }),
        },
        ToolMeta {
            name: "sys_time",
            namespace: "ops",
            description: "Timestamps em ISO-8601 e Unix Epoch com precisão de microssegundos.",
            is_level_0: false,
            input_schema: || json!({
                "type": "object",
                "properties": {},
                "additionalProperties": false
            }),
        },
        // sys.* namespace
        ToolMeta {
            name: "tree",
            namespace: "sys",
            description: "Lente de diretórios não-bloqueante com poda de caminhos tóxicos.",
            is_level_0: false,
            input_schema: || json!({
                "type": "object",
                "properties": {
                    "path": { "type": "string", "default": ".", "description": "Diretório base." },
                    "depth": { "type": "integer", "default": 2, "description": "Profundidade máxima de varredura." }
                },
                "additionalProperties": false
            }),
        },
        ToolMeta {
            name: "search",
            namespace: "sys",
            description: "Busca textual acelerada em memória com agrupamento sintático.",
            is_level_0: false,
            input_schema: || json!({
                "type": "object",
                "properties": {
                    "pattern": { "type": "string", "description": "Expressão ou termo de busca." },
                    "dir": { "type": "string", "default": ".", "description": "Diretório a buscar." }
                },
                "required": ["pattern"],
                "additionalProperties": false
            }),
        },
        ToolMeta {
            name: "shell",
            namespace: "sys",
            description: "Terminal assíncrono restrito com filtro anti-ANSI e poda de logs.",
            is_level_0: false,
            input_schema: || json!({
                "type": "object",
                "properties": {
                    "command": { "type": "string", "description": "Comando de sistema permitido." }
                },
                "required": ["command"],
                "additionalProperties": false
            }),
        },
        ToolMeta {
            name: "execute",
            namespace: "sys",
            description: "Executor isolado em sandbox de sistema operacional (JobObject).",
            is_level_0: false,
            input_schema: || json!({
                "type": "object",
                "properties": {
                    "binary": { "type": "string", "description": "Binário a executar." },
                    "args": { "type": "array", "items": { "type": "string" }, "description": "Argumentos do processo." }
                },
                "required": ["binary"],
                "additionalProperties": false
            }),
        },
        ToolMeta {
            name: "fetch_web",
            namespace: "sys",
            description: "Scraper assíncrono com conversão de HTML para Markdown limpo.",
            is_level_0: false,
            input_schema: || json!({
                "type": "object",
                "properties": {
                    "url": { "type": "string", "description": "URL para requisição." }
                },
                "required": ["url"],
                "additionalProperties": false
            }),
        },
        ToolMeta {
            name: "web_search",
            namespace: "sys",
            description: "Motor de busca DuckDuckGo HTML com extração direta de snippets.",
            is_level_0: false,
            input_schema: || json!({
                "type": "object",
                "properties": {
                    "query": { "type": "string", "description": "Termo de busca na internet." }
                },
                "required": ["query"],
                "additionalProperties": false
            }),
        },
        ToolMeta {
            name: "repo_meta",
            namespace: "sys",
            description: "Extrai metadados estruturais de repositórios Git via gix.",
            is_level_0: false,
            input_schema: || json!({
                "type": "object",
                "properties": {
                    "path": { "type": "string", "default": ".", "description": "Caminho do repositório." }
                },
                "additionalProperties": false
            }),
        },
        ToolMeta {
            name: "routes",
            namespace: "sys",
            description: "Inspeciona rotas ativas do daemon Axum e status de ganchos MCP/SSE.",
            is_level_0: false,
            input_schema: || json!({
                "type": "object",
                "properties": {},
                "additionalProperties": false
            }),
        },
        ToolMeta {
            name: "profile_list",
            namespace: "sys",
            description: "Inspeciona os 5 perfis cognitivos do Master Brain do Hermes Agent e os modelos campeões da Fronteira de Pareto.",
            is_level_0: false,
            input_schema: || json!({
                "type": "object",
                "properties": {},
                "additionalProperties": false
            }),
        },
        ToolMeta {
            name: "profile_switch",
            namespace: "sys",
            description: "Alterna atomicamente o modelo do Master Brain no config.yaml do Hermes Agent com backup preventivo (.bak).",
            is_level_0: false,
            input_schema: || json!({
                "type": "object",
                "properties": {
                    "profile": {
                        "type": "string",
                        "enum": ["frontier_balanced", "deep_swe", "deep_reasoning", "ultra_fast", "local_sovereign"],
                        "description": "Identificador do perfil canônico a ser ativado no Master Brain."
                    },
                    "model_override": {
                        "type": "string",
                        "description": "Sobrescreve explicitamente o modelo alvo (ex: 'deepseek/deepseek-chat')."
                    },
                    "provider": {
                        "type": "string",
                        "description": "Sobrescreve o provedor de inferência (ex: 'nous', 'custom', 'openrouter')."
                    }
                },
                "additionalProperties": false
            }),
        },
        // ops.* Cloud Arena tools
        ToolMeta {
            name: "cloud_champions",
            namespace: "ops",
            description: "Consulta os modelos campeões da Fronteira de Pareto nos 9 perfis cognitivos ADR-009 (Executive, Coding, Agentic, Reasoning, Fast, Design, Research, Security, Curation).",
            is_level_0: false,
            input_schema: || json!({
                "type": "object",
                "properties": {
                    "profile": {
                        "type": "string",
                        "enum": ["all", "executive", "coding", "agentic", "reasoning", "fast", "design", "research", "security", "curation"],
                        "default": "all",
                        "description": "Perfil de carga alvo para filtrar os campeões de nuvem."
                    },
                    "date": {
                        "type": "string",
                        "description": "Data específica do snapshot no formato YYYY-MM-DD (padrão: hoje)."
                    }
                },
                "additionalProperties": false
            }),
        },
        ToolMeta {
            name: "cloud_arena_sync",
            namespace: "ops",
            description: "Dispara sincronização sob demanda dos preços e benchmarks de modelos em nuvem (Nous Portal e OpenRouter).",
            is_level_0: false,
            input_schema: || json!({
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
            }),
        },
        // research.* namespace (v0.7.0 Deep Research Hub & Mission Control Plane)
        ToolMeta {
            name: "research_claim",
            namespace: "research",
            description: "Reivindica a próxima missão de pesquisa enfileirada no Souls Engine para execução pelo Hermes Agent.",
            is_level_0: false,
            input_schema: || json!({
                "type": "object",
                "properties": {
                    "agent_id": {
                        "type": "string",
                        "description": "Identificador do agente reivindicador (ex: 'hermes_master' ou nome da persona)."
                    },
                    "run_id": {
                        "type": "string",
                        "description": "ID opcional de uma missão específica para reivindicar diretamente."
                    }
                },
                "required": ["agent_id"],
                "additionalProperties": false
            }),
        },
        ToolMeta {
            name: "research_event",
            namespace: "research",
            description: "Reporta progresso, consumo de tokens por tier e atualiza o batimento cardíaco da missão ativa.",
            is_level_0: false,
            input_schema: || json!({
                "type": "object",
                "properties": {
                    "run_id": {
                        "type": "string",
                        "description": "ID da missão ativa no formato YYYYMMDD-HHMMSS-slug."
                    },
                    "phase": {
                        "type": "string",
                        "description": "Fase atual (ex: 'scoping', 'scouts_round_1', 'synthesis', 'verifying')."
                    },
                    "round": {
                        "type": "integer",
                        "default": 1,
                        "description": "Ronda IterResearch corrente."
                    },
                    "summary": {
                        "type": "string",
                        "description": "Resumo sintético do progresso ou descobertas desta ronda."
                    },
                    "tokens_tier0": { "type": "integer", "description": "Tokens poupados no Tier 0 determinístico." },
                    "tokens_tier1": { "type": "integer", "description": "Tokens consumidos em SLMs locais MinionS." },
                    "tokens_tier2": { "type": "integer", "description": "Tokens consumidos nos exploradores Fast-Scout." },
                    "tokens_tier3": { "type": "integer", "description": "Tokens consumidos no cérebro estratégico." },
                    "usd_spent": { "type": "number", "description": "Custo financeiro incorrido nesta fase em USD." },
                    "turns_used": { "type": "integer", "description": "Turnos totais consumidos do IterationBudget." }
                },
                "required": ["run_id", "phase", "summary"],
                "additionalProperties": false
            }),
        },
        ToolMeta {
            name: "research_steering",
            namespace: "research",
            description: "Consulta diretrizes em voo ou comandos de controle (pausa, forçar síntese, cancelar) injetados pelo operador.",
            is_level_0: false,
            input_schema: || json!({
                "type": "object",
                "properties": {
                    "run_id": {
                        "type": "string",
                        "description": "ID da missão para consultar orientações de steering."
                    }
                },
                "required": ["run_id"],
                "additionalProperties": false
            }),
        },
        ToolMeta {
            name: "research_complete",
            namespace: "research",
            description: "Conclui uma missão de pesquisa com verificação factual e registra o status final.",
            is_level_0: false,
            input_schema: || json!({
                "type": "object",
                "properties": {
                    "run_id": {
                        "type": "string",
                        "description": "ID da missão a finalizar."
                    },
                    "status": {
                        "type": "string",
                        "enum": ["confirmed", "blocked", "failed"],
                        "description": "Status de conclusão (ex: 'confirmed' após validação pelo subagente verificador)."
                    },
                    "report_path": {
                        "type": "string",
                        "description": "Caminho do relatório final no sistema de arquivos."
                    },
                    "receipt_path": {
                        "type": "string",
                        "description": "Caminho do recibo 03_verification_receipt.json gerado pelo verificador."
                    }
                },
                "required": ["run_id", "status"],
                "additionalProperties": false
            }),
        },
        // ui.* namespace (v0.8.3 Cognitive Vision O(1) & AXTree LEAN)
        ToolMeta {
            name: "ui_inspect",
            namespace: "ui",
            description: "Extrai e serializa a árvore de acessibilidade no formato LEAN dot-notation com 71% de redução de tokens e 0 MB VRAM.",
            is_level_0: false,
            input_schema: || json!({
                "type": "object",
                "properties": {
                    "include_click_coords": { "type": "boolean", "default": true, "description": "Incluir coordenadas geométricas de clique" },
                    "max_depth": { "type": "integer", "description": "Profundidade máxima de serialização da árvore" }
                },
                "additionalProperties": true
            }),
        },
        ToolMeta {
            name: "ui_interact",
            namespace: "ui",
            description: "Calcula coordenadas de clique centrais determinísticas e resolve o nó tátil sob o cursor.",
            is_level_0: false,
            input_schema: || json!({
                "type": "object",
                "properties": {
                    "x": { "type": "number", "description": "Coordenada X na tela" },
                    "y": { "type": "number", "description": "Coordenada Y na tela" }
                },
                "required": ["x", "y"],
                "additionalProperties": true
            }),
        },
        // google.* namespace (v0.8.2 Google Universe Bridge & NotebookLM)
        ToolMeta {
            name: "notebooklm_rpc",
            namespace: "google",
            description: "Executa uma chamada RPC batchexecute no Google NotebookLM (notas, deep research, RAG, studio audio).",
            is_level_0: false,
            input_schema: || json!({
                "type": "object",
                "properties": {
                    "rpc_id": { "type": "string", "description": "Identificador do RPC (ex: 'wXbhsf', 'rLM1Ne', 'QA9ei', 'gArtLc', 'orqRsd')" },
                    "arguments": { "type": "array", "description": "Lista de argumentos em formato JSON para o payload" }
                },
                "required": ["rpc_id", "arguments"],
                "additionalProperties": false
            }),
        },
        // google.* Life Hub & Multi-Account Governance (v0.9.2 - 14 tools)
        ToolMeta {
            name: "google_accounts",
            namespace: "google",
            description: "Gerencia contas Google no cofre encriptado GV2, executa o Google Doctor, importa credenciais gws e ajusta políticas BMAD.",
            is_level_0: false,
            input_schema: || json!({
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
            }),
        },
        ToolMeta {
            name: "google_briefing",
            namespace: "google",
            description: "Gera o Daily Briefing unificado (Agenda Calendar, Blocos Deep Focus, Trânsito Routes v2, Gmail Prioritário, Tasks e Inbox BMAD) com degradação graciosa offline.",
            is_level_0: false,
            input_schema: || json!({
                "type": "object",
                "properties": {
                    "account_id": { "type": "string", "description": "ID ou email da conta Google opcional." }
                },
                "additionalProperties": false
            }),
        },
        ToolMeta {
            name: "google_gmail",
            namespace: "google",
            description: "Triagem determinística de e-mails Gmail, leitura com delimitação anti-injection, criação de rascunhos, rótulos em lote e envio via BMAD Inbox.",
            is_level_0: false,
            input_schema: || json!({
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
            }),
        },
        ToolMeta {
            name: "google_calendar",
            namespace: "google",
            description: "Consulta agenda Google Calendar, encontra blocos de foco profundo (Deep Focus Slots) e cria/remove eventos com Taint Guard BMAD.",
            is_level_0: false,
            input_schema: || json!({
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
            }),
        },
        ToolMeta {
            name: "google_drive",
            namespace: "google",
            description: "Pesquisa arquivos no Google Drive, inspeciona metadados, baixa para exchange sandbox ou propõe compartilhamento/exclusão via BMAD Inbox.",
            is_level_0: false,
            input_schema: || json!({
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
            }),
        },
        ToolMeta {
            name: "google_docs_scratchpad",
            namespace: "google",
            description: "Rascunho incremental por linhas para Google Docs (load, insert, replace, delete, render) e push via Myers Diff no BMAD Agent Inbox.",
            is_level_0: false,
            input_schema: || json!({
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
            }),
        },
        ToolMeta {
            name: "google_sheets",
            namespace: "google",
            description: "Leitura e análise financeira determinística (totais, categorias, anomalias >2x média, projeção 30d) ou escrita via BMAD Inbox em Google Sheets.",
            is_level_0: false,
            input_schema: || json!({
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
            }),
        },
        ToolMeta {
            name: "google_slides",
            namespace: "google",
            description: "Planejamento estruturado (5 a 15 slides) e criação assistida de apresentações Google Slides.",
            is_level_0: false,
            input_schema: || json!({
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
            }),
        },
        ToolMeta {
            name: "google_tasks",
            namespace: "google",
            description: "Gerencia Google Tasks, executa Obsidian Quick Capture (fallback soberano para Google Keep em contas pessoais) e checa elegibilidade Keep Enterprise.",
            is_level_0: false,
            input_schema: || json!({
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
            }),
        },
        ToolMeta {
            name: "google_contacts",
            namespace: "google",
            description: "Busca contatos via Google People API e detecta aniversários próximos.",
            is_level_0: false,
            input_schema: || json!({
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
            }),
        },
        ToolMeta {
            name: "google_commute",
            namespace: "google",
            description: "Calcula rota com tráfego ao vivo (Google Maps Routes API v2) e gera alerta proativo de horário de saída.",
            is_level_0: false,
            input_schema: || json!({
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
            }),
        },
        ToolMeta {
            name: "google_youtube",
            namespace: "google",
            description: "Extrai e cacheia transcrições de vídeos YouTube (Innertube + parser XML/SRV3 puro em Rust) com delimitação anti-injection.",
            is_level_0: false,
            input_schema: || json!({
                "type": "object",
                "properties": {
                    "url_or_id": { "type": "string" },
                    "preferred_lang": { "type": "string", "default": "en" },
                    "session_id": { "type": "string" },
                    "api_base_override": { "type": "string" }
                },
                "required": ["url_or_id"],
                "additionalProperties": false
            }),
        },
        ToolMeta {
            name: "google_notebooklm",
            namespace: "google",
            description: "Gerencia cadernos Google NotebookLM, fontes Drive (check_freshness/sync_drive), RAG query, Studio Audio Overview e Mapas Mentais.",
            is_level_0: false,
            input_schema: || json!({
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
            }),
        },
        ToolMeta {
            name: "google_gems",
            namespace: "google",
            description: "Gerencia catálogo local de Custom Gems e despacha prompts via Gemini API oficial (bloqueando StreamGenerate em contas primárias).",
            is_level_0: false,
            input_schema: || json!({
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
            }),
        },
        ToolMeta {
            name: "google_pulse",
            namespace: "google",
            description: "Pulso Proativo Google 24/7: status multi-cadência, disparo sob demanda, eventos pendentes, reconhecimento, configuração de Quiet Hours e vínculos Drive <-> NotebookLM.",
            is_level_0: false,
            input_schema: || json!({
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
            }),
        },
        ToolMeta {
            name: "google_apps_script",
            namespace: "google",
            description: "Automações 24/7 no Google Apps Script: templates prontos, criação de projeto, inspeção estática de segredos/permissões em português e propostas push/deploy/run com Myers Diff no BMAD Agent Inbox.",
            is_level_0: false,
            input_schema: || json!({
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
            }),
        },
        ToolMeta {
            name: "google_forms",
            namespace: "google",
            description: "Google Forms API v1: leitura de esquema, criação governada via BMAD Agent Inbox, listagem de respostas e estatísticas determinísticas com resumo Markdown.",
            is_level_0: false,
            input_schema: || json!({
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
            }),
        },
        ToolMeta {
            name: "google_chat",
            namespace: "google",
            description: "Google Chat API v1 (exclusivo Workspace corporativo para Spaces/Messages) + propostas de envio via Webhook ou Espaço governadas pelo BMAD Agent Inbox.",
            is_level_0: false,
            input_schema: || json!({
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
            }),
        },
        ToolMeta {
            name: "google_deep_analyze",
            namespace: "google",
            description: "Leitura econômica MinionS em documentos Google, PDFs (até 25MB), threads Gmail, Formulários e textos extensos usando Tier 0 BM25 + cache local e escalonamento opcional para SLM local.",
            is_level_0: false,
            input_schema: || json!({
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
            }),
        },
        // inbox.* namespace (v0.9.2 BMAD Agent Inbox - 3 tools)
        ToolMeta {
            name: "inbox_list",
            namespace: "inbox",
            description: "Lista propostas pendentes ou históricas no BMAD Agent Inbox (com contagem de pendências e redação de tokens Gravity Lock).",
            is_level_0: false,
            input_schema: || json!({
                "type": "object",
                "properties": {
                    "status": { "type": "string", "enum": ["all", "pending", "approved", "executed", "rejected", "failed", "expired"], "default": "all" },
                    "limit": { "type": "integer", "default": 50 }
                },
                "additionalProperties": false
            }),
        },
        ToolMeta {
            name: "inbox_inspect_diff",
            namespace: "inbox",
            description: "Inspeciona o Myers Diff completo, snapshots before/after, nível de risco e status de taint de uma proposta no BMAD Agent Inbox.",
            is_level_0: false,
            input_schema: || json!({
                "type": "object",
                "properties": {
                    "proposal_id": { "type": "string" }
                },
                "required": ["proposal_id"],
                "additionalProperties": false
            }),
        },
        ToolMeta {
            name: "inbox_propose",
            namespace: "inbox",
            description: "Enfileira uma proposta de mutação externa no BMAD Agent Inbox computando Myers Diff real via souls_ast e herdando estado de Taint Guard da sessão.",
            is_level_0: false,
            input_schema: || json!({
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
            }),
        },
        // ops.* namespace (v0.8.1 Headless Agentic CLI Harness)
        ToolMeta {
            name: "agentic_dispatch",
            namespace: "ops",
            description: "Despacha tarefas em segundo plano para CLIs agênticas no host (agy, claude, codex) com rollback automático git restore.",
            is_level_0: false,
            input_schema: || json!({
                "type": "object",
                "properties": {
                    "cli": { "type": "string", "enum": ["agy", "claude", "codex"], "description": "CLI agêntica de destino" },
                    "prompt": { "type": "string", "description": "Instrução textual da tarefa de código" },
                    "working_dir": { "type": "string", "description": "Diretório de trabalho (padrão: Z:\\souls_engine)" },
                    "timeout_secs": { "type": "integer", "default": 300, "description": "Tempo limite em segundos" },
                    "auto_restore": { "type": "boolean", "default": true, "description": "Reverter arquivos se o processo falhar" }
                },
                "required": ["cli", "prompt"],
                "additionalProperties": false
            }),
        },
    ]
}

/// Lists tools according to Progressive Disclosure policy.
pub fn list_tools(full_mode: bool) -> Value {
    let tools = get_canonical_tools();
    let filtered: Vec<Value> = tools
        .iter()
        .filter(|t| full_mode || t.is_level_0)
        .map(|t| {
            json!({
                "name": t.name,
                "description": t.description,
                "inputSchema": (t.input_schema)()
            })
        })
        .collect();

    json!({ "tools": filtered })
}

/// Dynamic discovery meta-tool search handler.
#[allow(dead_code)]
pub fn search_tools(namespace: Option<&str>, query: Option<&str>) -> Value {
    let tools = get_canonical_tools();
    let query_lower = query.map(|q| q.to_lowercase());

    let results: Vec<Value> = tools
        .iter()
        .filter(|t| {
            if let Some(ns) = namespace {
                if ns != "all" && !ns.eq_ignore_ascii_case(t.namespace) {
                    return false;
                }
            }
            if let Some(ref q) = query_lower {
                let name_match = t.name.to_lowercase().contains(q);
                let desc_match = t.description.to_lowercase().contains(q);
                let ns_match = t.namespace.to_lowercase().contains(q);
                name_match || desc_match || ns_match
            } else {
                true
            }
        })
        .map(|t| {
            json!({
                "name": t.name,
                "namespace": t.namespace,
                "description": t.description,
                "inputSchema": (t.input_schema)()
            })
        })
        .collect();

    json!({
        "matched_count": results.len(),
        "tools": results
    })
}
