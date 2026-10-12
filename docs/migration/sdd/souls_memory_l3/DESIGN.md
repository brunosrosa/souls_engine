# DESIGN.md — Especificação Técnica de Interfaces e DDL do Cérebro L3 (`souls_memory`) (v7 Canônica)

> **ESTATUS DO DOCUMENTO:** CANÔNICO E APROVADO (SDD)
> **SUB-SISTEMA:** Hipocampo L3 Soberano & Memória de Sessões de Subagentes
> **CRATE ALVO:** `crates/souls_memory`

---

## 1. DDL CANÔNICO COMPLEMENTAR FRANKENSQLITE STRICT

Adicionado ao `CANONICAL_DDL_MANIFEST` em `crates/souls_memory/src/sqlite.rs`:

```sql
-- 8. Sub-Agent Lifecycle Sessions
CREATE TABLE IF NOT EXISTS sub_agent_sessions (
    agent_id TEXT PRIMARY KEY NOT NULL,
    parent_session_id TEXT NOT NULL,
    task_description TEXT NOT NULL,
    assigned_tier TEXT NOT NULL,
    status TEXT NOT NULL CHECK(status IN ('SPAWNED', 'RUNNING', 'COMPLETED', 'FAILED', 'KILLED')),
    created_at INTEGER NOT NULL,
    completed_at INTEGER
) STRICT;

CREATE INDEX IF NOT EXISTS idx_subagent_parent 
    ON sub_agent_sessions (parent_session_id, status);

-- 9. Handoff Payloads (Transição de Contexto entre Agentes)
CREATE TABLE IF NOT EXISTS handoff_payloads (
    id TEXT PRIMARY KEY NOT NULL,
    source_session_id TEXT NOT NULL,
    target_agent_id TEXT NOT NULL,
    context_summary TEXT NOT NULL,
    variables_json TEXT NOT NULL,
    created_at INTEGER NOT NULL
) STRICT;

CREATE INDEX IF NOT EXISTS idx_handoff_target 
    ON handoff_payloads (target_agent_id);

-- 10. Sub-Agent Operational Diaries
CREATE TABLE IF NOT EXISTS sub_agent_diaries (
    id TEXT PRIMARY KEY NOT NULL,
    agent_id TEXT NOT NULL,
    session_id TEXT NOT NULL,
    content TEXT NOT NULL,
    created_at INTEGER NOT NULL,
    FOREIGN KEY (agent_id) REFERENCES sub_agent_sessions(agent_id) ON DELETE CASCADE
) STRICT;

CREATE INDEX IF NOT EXISTS idx_diaries_agent 
    ON sub_agent_diaries (agent_id, created_at ASC);

-- 11. Socratic Thinking Sessions (DAG Metacognitivo)
CREATE TABLE IF NOT EXISTS socratic_sessions (
    session_id TEXT PRIMARY KEY NOT NULL,
    paradigm TEXT NOT NULL,
    metadata_json TEXT NOT NULL DEFAULT '{}',
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL
) STRICT;

-- 12. Socratic Thoughts (Nós da Árvore de Raciocínio)
CREATE TABLE IF NOT EXISTS socratic_thoughts (
    thought_id TEXT PRIMARY KEY NOT NULL,
    session_id TEXT NOT NULL,
    thought_number INTEGER NOT NULL,
    total_thoughts INTEGER NOT NULL,
    thought_content TEXT NOT NULL,
    is_revision INTEGER NOT NULL DEFAULT 0,
    revises_thought INTEGER,
    branch_from_thought INTEGER,
    branch_id TEXT NOT NULL DEFAULT 'main',
    duration_ms INTEGER NOT NULL DEFAULT 0,
    created_at INTEGER NOT NULL,
    FOREIGN KEY (session_id) REFERENCES socratic_sessions(session_id) ON DELETE CASCADE
) STRICT;

CREATE INDEX IF NOT EXISTS idx_thoughts_session_order 
    ON socratic_thoughts (session_id, thought_number ASC);
CREATE INDEX IF NOT EXISTS idx_thoughts_branch 
    ON socratic_thoughts (session_id, branch_id);
```

---

## 2. ESTRUTURAS DE DADOS E CONTRATOS EM RUST

### 2.1 Módulo `session_manager.rs`
Governa o ciclo de vida dos subagentes delegados e seus pacotes de transição.

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum SubAgentStatus {
    Spawned,
    Running,
    Completed,
    Failed,
    Killed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubAgentSessionRecord {
    pub agent_id: String,
    pub parent_session_id: String,
    pub task_description: String,
    pub assigned_tier: String,
    pub status: SubAgentStatus,
    pub created_at: i64,
    pub completed_at: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HandoffPayloadRecord {
    pub id: String,
    pub source_session_id: String,
    pub target_agent_id: String,
    pub context_summary: String,
    pub variables_json: String,
    pub created_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubAgentDiaryRecord {
    pub id: String,
    pub agent_id: String,
    pub session_id: String,
    pub content: String,
    pub created_at: i64,
}

pub struct SubAgentManager {
    pool: Arc<Pool<Sqlite>>,
    active_sessions: DashMap<String, SubAgentSessionRecord>,
}
```

### 2.2 Módulo `thinking.rs`
Governa o raciocínio socrático em grafo acíclico dirigido (DAG), detecção de revisões e proteção contra *overthinking*.

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum ThinkingParadigm {
    #[default]
    LinearCoT,
    TreeToT,
    GraphGoT,
    CouncilMAD,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ThinkingContext {
    pub file_path: Option<String>,
    pub impact_score: f64,
    pub consecutive_failures: u32,
    pub hitl_authorized: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SocraticThoughtInput {
    pub thought: String,
    pub thought_number: u32,
    pub total_thoughts: u32,
    pub next_thought_needed: bool,
    pub is_revision: Option<bool>,
    pub revises_thought: Option<u32>,
    pub branch_from_thought: Option<u32>,
    pub branch_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SocraticThoughtRecord {
    pub thought_id: String,
    pub session_id: String,
    pub thought_number: u32,
    pub total_thoughts: u32,
    pub thought_content: String,
    pub is_revision: bool,
    pub revises_thought: Option<u32>,
    pub branch_from_thought: Option<u32>,
    pub branch_id: String,
    pub duration_ms: i64,
    pub created_at: i64,
}

pub struct ThinkingEngine {
    session_id: String,
    paradigm: ThinkingParadigm,
    thoughts: Vec<SocraticThoughtRecord>,
    branches: HashMap<String, Vec<String>>,
    hard_limit: u32,
    hitl_authorized: bool,
}
```

### 2.3 Módulo `mpsc_bridge.rs`
Canal assíncrono não-bloqueante para amortização de I/O em background.

```rust
pub enum MemoryBridgeOp {
    RecordDiary(SubAgentDiaryRecord),
    PersistThought(SocraticThoughtRecord),
    SaveCheckpoint {
        id: String,
        session_id: String,
        blob: String,
        count: i64,
        tokens: i64,
        hash: String,
    },
    FireAndForgetSql(String),
}

pub struct MemoryMpscBridge {
    tx: tokio::sync::mpsc::Sender<MemoryBridgeOp>,
}
```

---

## 3. CONEXÃO COM FERRAMENTAS MCP

| Ferramenta MCP | Método em `souls_memory` |
| :--- | :--- |
| `handoff` | `SubAgentManager::create_handoff` / `get_handoff` |
| `sub_agent` | `SubAgentManager::spawn_subagent`, `update_status`, `list_subagents` |
| `thinking` / `core_think` | `ThinkingEngine::add_thought` |
| `session` | `ThinkingEngine::reset` / `clear` |
| `analyze_session` | `ThinkingEngine::analyze_coherence` |
| `export_session` | `ThinkingEngine::export_markdown` / `export_json` |
| `merge_sessions` | `ThinkingEngine::merge_branch` |
| `knowledge` | `sqlite::insert_epistemic_memory` / `get_epistemic_memory` |
| `sqlite_query` | `sqlite::execute_read_only_query` |
