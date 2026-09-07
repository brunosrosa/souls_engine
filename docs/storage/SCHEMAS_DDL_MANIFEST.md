# SCHEMAS_DDL_MANIFEST.md — A Verdade Canônica de Persistência: FrankenSQLite STRICT & LanceDB Arrow (v7)

> **ESTATUS DO DOCUMENTO:** CANÔNICO E IMUTÁVEL (DOCS-AS-GUARDRAILS)
> **APLICAÇÃO:** Todos os agentes de IA (Cursor, Windsurf, Claude Code) e engenheiros humanos.
> **CRATES DIRETAMENTE REGIDAS:** `souls_memory`, `souls_protocol`, `souls_server`, `souls_anthropophagy`.
> **PROPÓSITO:** Congelar as definições DDL relacionais, contratos colunares Arrow e esquemas vetoriais, erradicando alucinações de colunas, incompatibilidade de tipos e quebras de migração no Dev Drive ReFS (`Z:\souls_engine\.souls_data\`).

## 1. PREÂMBULO E TOPOLOGIA DE PERSISTÊNCIA WIN32 REFS

Toda a persistência de dados do Souls Engine reside exclusivamente no volume montado em **Dev Drive ReFS** na partição canônica:

```
Z:\souls_engine\.souls_data\
├── db\
│   ├── souls_state.db           # FrankenSQLite STRICT WAL (Base de dados relacional principal)
│   ├── souls_state.db-wal       # Write-Ahead Log (Buffer volátil de escrita)
│   └── souls_state.db-shm       # Shared Memory Index (Ponteiros de coordenação de leitores)
├── vectors\
│   └── souls_vectors.lance\     # Repositório vetorial colunar Apache Arrow / LanceDB
├── models\                      # Pesos quantizados GGUF e modelos ONNX
└── spool\                       # Buffer de telemetria transitória e logs de recuperação
```

### 1.1 Primitivos de Conexão e PRAGMAs Win32 Obrigatórios

Para anular contenções de concorrência (`SQLITE_BUSY`) e violações de bloqueio obrigatório no kernel Windows NT (`ERROR_SHARING_VIOLATION` código 32 e `ERROR_USER_MAPPED_FILE` código 1224), qualquer inicialização de pool de conexões SQLite através do `sqlx` na crate `souls_memory` deve aplicar rigorosamente os seguintes parâmetros:

```
-- Executado no handshake de cada conexão do pool SQLite
PRAGMA journal_mode = WAL;
PRAGMA synchronous = NORMAL;
PRAGMA busy_timeout = 5000;
PRAGMA temp_store = MEMORY;
PRAGMA foreign_keys = ON;
PRAGMA cache_size = -64000; -- Alocação fixa de 64 MB de cache em RAM por conexão
PRAGMA auto_vacuum = NONE;  -- Desfragmentação manual delegada exclusivamente ao Chyros Daemon
```

### 1.2 Regras de Compartilhamento no Kernel NT

O driver de abertura de descritores de arquivo deve especificar as flags de compartilhamento Win32:

```
// flags aplicadas via windows-sys = "0.59" na abertura de descritores NT
FILE_SHARE_READ | FILE_SHARE_WRITE | FILE_SHARE_DELETE
```

O interpretador Python do Hermes Agent e subagentes externos estão **terminantemente proibidos** de instanciar conexões diretas de escrita no arquivo `souls_state.db` ou abrir descritores em `Z:\souls_engine\.souls_data\vectors\`. Todo o I/O é serializado exclusivamente pela crate `souls_memory` no processo nativo `souls_server.exe`.

## 2. DDL CANÔNICO DO FRANKENSQLITE (MODO STRICT)

A tipagem dinâmica padrão do SQLite é categoricamente banida. Todas as tabelas relacionais devem conter a cláusula `STRICT` no fechamento de sua declaração, garantindo que qualquer inserção com desvio de tipo gere um erro fatal no momento da inserção (`DataTypeMismatch`).

### 2.1 Tabela `epistemic_memories` (Hipocampo Relacional L3)

Armazena fatos permanentes, diretrizes operacionais, decisões conceituais e fragmentos transitórios da interação homem-máquina.

```
CREATE TABLE IF NOT EXISTS epistemic_memories (
    id TEXT PRIMARY KEY NOT NULL,
    session_id TEXT NOT NULL,
    category TEXT NOT NULL,
    content TEXT NOT NULL,
    partition TEXT NOT NULL CHECK(partition IN ('STABLE', 'EVOLVING')),
    salience REAL NOT NULL CHECK(salience >= 0.0 AND salience <= 1.0),
    access_count INTEGER NOT NULL DEFAULT 0,
    source_ref TEXT,
    created_at INTEGER NOT NULL,
    last_accessed_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL
) STRICT;

CREATE INDEX IF NOT EXISTS idx_epistemic_partition_salience 
    ON epistemic_memories (partition, salience DESC);

CREATE INDEX IF NOT EXISTS idx_epistemic_session 
    ON epistemic_memories (session_id);

CREATE INDEX IF NOT EXISTS idx_epistemic_category 
    ON epistemic_memories (category);
```

### 2.2 Tabela Virtual `epistemic_memories_fts` (Motor FTS5 BM25)

Prover busca léxica de alta velocidade e cálculo de pontuação BM25 integrada ao algoritmo Reciprocal Rank Fusion (RRF).

```
CREATE VIRTUAL TABLE IF NOT EXISTS epistemic_memories_fts USING fts5 (
    id UNINDEXED,
    content,
    category,
    tokenize = 'unicode61 remove_diacritics 2'
);
```

#### Triggers de Sincronização Atômica FTS5

A sincronização entre a tabela `STRICT` e o índice invertido FTS5 é atômica e automática:

```
CREATE TRIGGER IF NOT EXISTS trg_epistemic_memories_ai 
AFTER INSERT ON epistemic_memories BEGIN
    INSERT INTO epistemic_memories_fts (id, content, category)
    VALUES (new.id, new.content, new.category);
END;

CREATE TRIGGER IF NOT EXISTS trg_epistemic_memories_ad 
AFTER DELETE ON epistemic_memories BEGIN
    DELETE FROM epistemic_memories_fts WHERE id = old.id;
END;

CREATE TRIGGER IF NOT EXISTS trg_epistemic_memories_au 
AFTER UPDATE ON epistemic_memories BEGIN
    DELETE FROM epistemic_memories_fts WHERE id = old.id;
    INSERT INTO epistemic_memories_fts (id, content, category)
    VALUES (new.id, new.content, new.category);
END;
```

### 2.3 Tabela `telemetry_events` (Buffer de Desidratação MPSC)

Recebe os eventos esvaziados a cada 5 segundos pelo `MpscBatchBuffer` da crate `souls_core`, prevenindo SSD Write Amplification no NVMe.

```
CREATE TABLE IF NOT EXISTS telemetry_events (
    id TEXT PRIMARY KEY NOT NULL,
    session_id TEXT NOT NULL,
    event_type TEXT NOT NULL,
    execution_tier TEXT NOT NULL,
    payload_json TEXT NOT NULL,
    latency_ms REAL NOT NULL,
    tokens_input INTEGER NOT NULL DEFAULT 0,
    tokens_output INTEGER NOT NULL DEFAULT 0,
    direct_cost_usd REAL NOT NULL DEFAULT 0.0,
    e3_score REAL NOT NULL DEFAULT 0.0,
    created_at INTEGER NOT NULL
) STRICT;

CREATE INDEX IF NOT EXISTS idx_telemetry_created_tier 
    ON telemetry_events (created_at DESC, execution_tier);

CREATE INDEX IF NOT EXISTS idx_telemetry_session 
    ON telemetry_events (session_id);
```

### 2.4 Tabela `pareto_bandit_priors` (Persistência Bayesiana FinOps)

Armazena as distribuições Beta a posteriori $\text{Beta}(\alpha_{k,c}, \beta_{k,c})$ para cada braço de modelo $k$ e categoria de tarefa $c$.

```
CREATE TABLE IF NOT EXISTS pareto_bandit_priors (
    tier_name TEXT NOT NULL,
    task_category TEXT NOT NULL,
    alpha REAL NOT NULL CHECK(alpha >= 1.0),
    beta REAL NOT NULL CHECK(beta >= 1.0),
    pull_count INTEGER NOT NULL DEFAULT 0,
    cumulative_reward REAL NOT NULL DEFAULT 0.0,
    last_updated_at INTEGER NOT NULL,
    PRIMARY KEY (tier_name, task_category)
) STRICT;
```

### 2.5 Tabela `organ_vault` (Catálogo da Crate `souls_anthropophagy`)

Armazena os algoritmos purificados, corpos de funções isoladas e estruturas de dados extraídas de repositórios abertos auditados por conformidade SPDX.

```
CREATE TABLE IF NOT EXISTS organ_vault (
    id TEXT PRIMARY KEY NOT NULL,
    symbol_name TEXT NOT NULL,
    language TEXT NOT NULL,
    spdx_license TEXT NOT NULL,
    source_repository TEXT NOT NULL,
    ast_kind TEXT NOT NULL,
    dependencies_json TEXT NOT NULL,
    signature_decl TEXT NOT NULL,
    pure_implementation TEXT NOT NULL,
    cyclomatic_complexity INTEGER NOT NULL,
    loc_count INTEGER NOT NULL,
    created_at INTEGER NOT NULL
) STRICT;

CREATE INDEX IF NOT EXISTS idx_organ_symbol 
    ON organ_vault (symbol_name, language);

CREATE INDEX IF NOT EXISTS idx_organ_license 
    ON organ_vault (spdx_license);
```

### 2.6 Tabelas de Espelhamento do Grafo Ontológico LadybugDB

Embora o grafo LadybugDB resida em memória RAM em uma estrutura de dados `petgraph` de alta velocidade, sua topologia é serializada relacionalmente para garantir reidratação em cold boot em menos de 20 ms.

```
CREATE TABLE IF NOT EXISTS ladybug_nodes (
    id TEXT PRIMARY KEY NOT NULL,
    node_type TEXT NOT NULL,
    canonical_name TEXT NOT NULL,
    metadata_json TEXT NOT NULL,
    created_at INTEGER NOT NULL
) STRICT;

CREATE INDEX IF NOT EXISTS idx_ladybug_node_type 
    ON ladybug_nodes (node_type, canonical_name);

CREATE TABLE IF NOT EXISTS ladybug_edges (
    source_id TEXT NOT NULL,
    target_id TEXT NOT NULL,
    relationship TEXT NOT NULL CHECK(relationship IN ('depends_on', 'conflicts_with', 'implements', 'relates_to')),
    weight REAL NOT NULL DEFAULT 1.0,
    created_at INTEGER NOT NULL,
    PRIMARY KEY (source_id, target_id, relationship),
    FOREIGN KEY (source_id) REFERENCES ladybug_nodes(id) ON DELETE CASCADE,
    FOREIGN KEY (target_id) REFERENCES ladybug_nodes(id) ON DELETE CASCADE
) STRICT;
```

### 2.7 Tabela `session_checkpoints` (Pre-Compress Hook API v2)

Registra os snapshots do histórico conversacional validados antes de qualquer truncamento executado pelo `ContextCompressor` do Hermes Agent.

```
CREATE TABLE IF NOT EXISTS session_checkpoints (
    id TEXT PRIMARY KEY NOT NULL,
    session_id TEXT NOT NULL,
    checkpoint_version INTEGER NOT NULL DEFAULT 2,
    raw_messages_blob TEXT NOT NULL,
    message_count INTEGER NOT NULL,
    pre_compress_tokens INTEGER NOT NULL,
    hash_sha256 TEXT NOT NULL,
    created_at INTEGER NOT NULL
) STRICT;

CREATE INDEX IF NOT EXISTS idx_checkpoint_session 
    ON session_checkpoints (session_id, created_at DESC);
```

## 3. ESQUEMAS COLUNARES APACHE ARROW (LANCEDB L3)

O repositório vetorial é governado com exclusividade pela crate `souls_memory` através do LanceDB, estruturado diretamente sobre o formato de dados colunar **Apache Arrow**.

### 3.1 Garantia de Silício (0 MB de VRAM)

O LanceDB opera exclusivamente acoplado ao disco Dev Drive ReFS através de mapeamento de memória virtual (`mmap`). As tabelas colunares Arrow e os índices de produto vetorial operam na RAM física do sistema (consumindo ~180 MB de heap sob carga), mantendo **estritamente 0 MB de VRAM** da dGPU RTX 2060m alocados para o banco de vetores.

### 3.2 Tabela Vetorial Canônica: `epistemic_vectors`

- **Caminho Físico:** `Z:\souls_engine\.souls_data\vectors\souls_vectors.lance`
- **Dimensão do Vetor:** $384$ floats de 32 bits (`Float32`).
- **Modelo Canônico de Embedding:** `bge-small-en-v1.5` / `all-MiniLM-L6-v2` executado via ONNX Runtime em CPU AVX2 (Tier 0).
- **Métrica de Distância:** Cosseno (`MetricType::Cosine`).

#### Definição Formal do Arrow Schema

| **Nome da Coluna Arrow** | **Tipo de Dado Arrow (arrow::datatypes::DataType)**      | **Nullable** | **Descrição Técnica**                                                          |
| ------------------------ | -------------------------------------------------------- | ------------ | ------------------------------------------------------------------------------ |
| `id`                     | `Utf8` (String)                                          | **Não**      | Identificador único idêntico à chave primária em `epistemic_memories.id`.      |
| `vector`                 | `FixedSizeList(Field::new("item", Float32, false), 384)` | **Não**      | Vetor denso de embedding de dimensão 384 em precisão simples IEEE 754.         |
| `session_id`             | `Utf8` (String)                                          | **Não**      | Identificador da sessão conversacional para filtragem por escopo.              |
| `category`               | `Utf8` (String)                                          | **Não**      | Tag categórica para particionamento pré-busca (ex: `architecture`, `finops`).  |
| `partition`              | `Utf8` (String)                                          | **Não**      | Partição ontológica: restrita aos valores literais `"STABLE"` ou `"EVOLVING"`. |
| `salience`               | `Float32`                                                | **Não**      | Grau de relevância da memória ($S_m \in [0.0, 1.0]$).                          |
| `created_at`             | `Int64`                                                  | **Não**      | Carimbo de data/hora Epoch Unix em milissegundos.                              |

#### Declaração do Schema em Rust (`souls_memory/src/lancedb.rs`)

```
use arrow::datatypes::{DataType, Field, Schema};
use std::sync::Arc;

pub fn canonical_lance_schema() -> Arc<Schema> {
    Arc::new(Schema::new(vec![
        Field::new("id", DataType::Utf8, false),
        Field::new(
            "vector",
            DataType::FixedSizeList(
                Arc::new(Field::new("item", DataType::Float32, false)),
                384,
            ),
            false,
        ),
        Field::new("session_id", DataType::Utf8, false),
        Field::new("category", DataType::Utf8, false),
        Field::new("partition", DataType::Utf8, false),
        Field::new("salience", DataType::Float32, false),
        Field::new("created_at", DataType::Int64, false),
    ]))
}
```

## 4. POLÍTICAS DE RETENÇÃO E METABOLISMO DO CHYROS DAEMON

O Chyros Daemon é a rotina noturna assíncrona da crate `souls_memory` que atua durante períodos de ociosidade prolongada da CPU Intel i9.

### 4.1 Imunidade Ontológica da Partição `STABLE`

Qualquer registro onde `partition = 'STABLE'` possui imunidade física incondicional.

- O valor de `salience` é mantido fixo em $1.0$.
- É **expressamente proibido** aplicar decaimento de Langevin ou comandos de `DELETE` automatizados sobre registros estáveis.

### 4.2 Langevin Decay sobre a Partição `EVOLVING`

Para registros onde `partition = 'EVOLVING'`, o daemon calcula o decaimento discreto a cada ciclo noturno:

$$S_m(t + \Delta t) = S_m(t) \cdot \exp\left(-\frac{\Delta t}{\tau \cdot (\text{access\_count} + 1)}\right) + \mathcal{N}(0, \sigma^2)$$

Onde:

- $\Delta t$: Tempo decorrido desde `last_accessed_at` em dias.
- $\tau$: Constante de tempo de meia-vida básica ($\tau = 7.0\text{ dias}$).
- $\mathcal{N}(0, \sigma^2)$: Ruído estocástico gaussiano com $\sigma = 0.02$.

### 4.3 Limiar Crítico de Expurgo

Se, após a atualização matemática, a saliência cair abaixo do limiar crítico:

$$S_m(t + \Delta t) < 0.15$$

O Chyros Daemon executa a purga atômica coordenada em duas etapas:
1. `DELETE FROM epistemic_memories WHERE id = ?;` (Acionando o trigger de exclusão no FTS5).
2. Remoção física do registro vetorial correspondente na tabela LanceDB:
    `lance_table.delete(format!("id = '{}'", id_expurgado)).await?;`

### 4.4 Protocolo de Desfragmentação sem Fragmentar o Dev Drive

Ao final da varredura, o daemon **não** executa `VACUUM` síncrono no arquivo em uso (o que causaria locks e fragmentação de páginas no ReFS). Em vez disso, invoca a replicação desfragmentada atômica:

```
VACUUM INTO 'Z:\souls_engine\.souls_data\db\souls_state_compact.db';
```

Após a finalização do comando, o arquivo antigo é rotacionado de forma atômica utilizando primitivos do kernel NT com `MoveFileExW` substituindo o descritor mestre.

## 5. MAPEAMENTO BIDIRECIONAL DE TIPOS: SQLITE, ARROW E RUST

Para eliminar incongruências em tempo de compilação, a tabela abaixo formaliza a correspondência exata de tipos entre o FrankenSQLite STRICT, o Arrow Schema do LanceDB e as structs canônicas da crate `souls_protocol`:

| **Domínio de Dados**      | **FrankenSQLite STRICT**                   | **Apache Arrow LanceDB**      | **Tipo em Rust (souls_protocol)**              |
| ------------------------- | ------------------------------------------ | ----------------------------- | ---------------------------------------------- |
| Identificador Universal   | `TEXT`                                     | `Utf8`                        | `String` (UUID v4 / Prefixo `mem_`)            |
| Vetor de Embedding        | _(Não armazenado no SQLite)_               | `FixedSizeList(Float32, 384)` | `[f32; 384]` ou `Vec<f32>`                     |
| Partição Epistêmica       | `TEXT` (`CHECK in ('STABLE', 'EVOLVING')`) | `Utf8`                        | `enum EpistemicPartition { Stable, Evolving }` |
| Saliência / Recompensa    | `REAL`                                     | `Float32`                     | `f32` (Normalizado em $[0.0, 1.0]$)            |
| Contadores / Offsets      | `INTEGER`                                  | `Int64`                       | `i64` / `u64`                                  |
| Timestamps Unix Epoch     | `INTEGER`                                  | `Int64`                       | `i64` (Milissegundos desde Epoch)              |
| Latência em Milissegundos | `REAL`                                     | `Float64`                     | `f64`                                          |
| Payloads Estruturados     | `TEXT`                                     | `Utf8`                        | `serde_json::Value`                            |
| Licença SPDX              | `TEXT`                                     | `Utf8`                        | `String` (Validado via `spdx-rs`)              |

### 5.1 Structs Canônicas em Rust (`crates/souls_protocol/src/dto.rs`)

```
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum EpistemicPartition {
    Stable,
    Evolving,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EpistemicMemoryRecord {
    pub id: String,
    pub session_id: String,
    pub category: String,
    pub content: String,
    pub partition: EpistemicPartition,
    pub salience: f32,
    pub access_count: i64,
    pub source_ref: Option<String>,
    pub created_at: i64,
    pub last_accessed_at: i64,
    pub updated_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TelemetryBatchRecord {
    pub id: String,
    pub session_id: String,
    pub event_type: String,
    pub execution_tier: String,
    pub payload_json: String,
    pub latency_ms: f64,
    pub tokens_input: i64,
    pub tokens_output: i64,
    pub direct_cost_usd: f64,
    pub e3_score: f64,
    pub created_at: i64,
}
```

## 6. PROTOCOLO DE MIGRAÇÃO E LINHAS VERMELHAS DE PERSISTÊNCIA

1. **PROIBIÇÃO ABSOLUTA DE `ALTER TABLE` ARBITRÁRIO:** Modificações de esquema exigem a emissão de uma migration numerada sequencial em `crates/souls_memory/migrations/V00X__descricao.sql`. É expressamente proibido alterar nomes de colunas ou remover constraints em produção sem recriação atômica via tabela de transição temporária.
2. **PROIBIÇÃO DE MODIFICAÇÃO DE PRAGMAS:** Sob nenhuma hipótese o agente de IA deve sugerir alterar `journal_mode` de `WAL` para `DELETE` ou desativar `foreign_keys`.
3. **PROIBIÇÃO DE INSERÇÃO DIRETA FORA DO TOKIO RUNTIME:** Qualquer I/O no SQLite ou LanceDB deve ser despachado de forma assíncrona nas threads especializadas do Tokio em `souls_memory`.
4. **INTEGRIDADE DE CHECKSUM:** A tabela `session_checkpoints` deve conter obrigatoriamente o hash SHA-256 do payload cru gerado no gancho `on_pre_compress` do Hermes Agent para fins de auditoria forense de integridade conversacional.