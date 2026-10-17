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
                        "enum": ["all", "code", "context", "memory", "think", "ops", "sys"],
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
            description: "Consulta os modelos campeões da Fronteira de Pareto nos 5 perfis cognitivos (Coding, Agentic, Reasoning, Fast, Design).",
            is_level_0: false,
            input_schema: || json!({
                "type": "object",
                "properties": {
                    "profile": {
                        "type": "string",
                        "enum": ["all", "coding", "agentic", "reasoning", "fast", "design"],
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
