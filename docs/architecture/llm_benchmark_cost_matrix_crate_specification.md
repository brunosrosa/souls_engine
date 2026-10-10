# Especificação Técnica: Crate `llm-cost-matrix`

Documento de especificação técnica e plano de execução para implementação de uma ferramenta CLI / crate em Rust (`llm-cost-matrix`) responsável por ingerir, normalizar, persistir e analisar diariamente o custo-benefício de modelos de linguagem (LLMs) em nuvem, cruzando provedores de inferência (Nous Research Portal, OpenRouter) com benchmarks consolidados (LMSYS, BFCL, IFEval, SWE-bench).

---

## 1. Visão Geral e Objetivo

O objetivo da crate é eliminar escolhas arbitrárias de modelos de IA em ambientes de produção através de um pipeline diário de dados com as seguintes responsabilidades:

1. **Ingestão Automática:** Consumir catálogos de modelos, precificação detalhada (input, output, cache read/write) e telemetria operacional (latência, throughput) de APIs públicas e autenticadas.
2. **Resolução Canônica de Entidades:** Normalizar slugs heterogêneos de diferentes plataformas para identificadores canônicos padronizados.
3. **Persistência Relacional:** Salvar snapshots temporais em banco SQLite local (`sqlite3` com WAL habilitado).
4. **Scoring Multidimensional:** Calcular índices compostos ponderados por caso de uso (ex.: Coding, Tool Calling / Agentes, Deep Reasoning, RAG / ETL de alto volume).
5. **Fronteira de Pareto e Relatório:** Identificar modelos dominantes (não superados em custo e qualidade simultaneamente) e gerar relatórios diários em Markdown, JSON e visualização de terminal.

---

## 2. Arquitetura da Solução

```
 ┌───────────────────────────────────────────────────────────────┐
 │                   Fontes de Dados (Sources)                   │
 │  ┌─────────────────┐  ┌─────────────────┐  ┌───────────────┐  │
 │  │ Nous Portal API │  │ OpenRouter API  │  │ Benchmark Data│  │
 │  │ (Preço + Mod.)  │  │ (Preço + SLA)   │  │ (Elo/BFCL/SWE)│  │
 │  └────────┬────────┘  └────────┬────────┘  └───────┬───────┘  │
 └───────────┼────────────────────┼───────────────────┼──────────┘
             ▼                    ▼                   ▼
 ┌───────────────────────────────────────────────────────────────┐
 │             Camada de Resolução Canônica (Resolver)           │
 │               Map: "deepseek/v3" -> "deepseek-v3"             │
 └───────────────────────────────┬───────────────────────────────┘
                                 ▼
 ┌───────────────────────────────────────────────────────────────┐
 │             Armazenamento SQLite (Migrations + WAL)           │
 │      - technical_specs        - pricing_snapshots             │
 │      - benchmark_snapshots    - composite_evaluations         │
 └───────────────────────────────┬───────────────────────────────┘
                                 ▼
 ┌───────────────────────────────────────────────────────────────┐
 │             Motor de Analytics e Otimização                   │
 │      - Task-Specific ROI      - Pareto Frontier Filter        │
 └───────────────────────────────┬───────────────────────────────┘
                                 ▼
 ┌───────────────────────────────────────────────────────────────┐
 │             Relatório Final & CLI Dashboard                   │
 │            CLI Terminal / Markdown Diário / JSON              │
 └───────────────────────────────────────────────────────────────┘
```

---

## 3. Fontes de Dados e Contratos de Integração

### 3.1. Nous Portal Inference API
* **Endpoint:** `GET https://inference-api.nousresearch.com/v1/models`
* **Autenticação:** Header `Authorization: Bearer <NOUS_API_KEY>`
* **Dados Extraídos:**
  * Model ID (ex.: `deepseek/deepseek-v3`, `nousresearch/hermes-3-llama-3.1-405b`)
  * Preço de Prompt (USD por milhão de tokens)
  * Preço de Completion (USD por milhão de tokens)
  * Preço de Leitura de Cache de Contexto (USD por milhão de tokens)
  * Tamanho máximo da janela de contexto (`context_length`)

### 3.2. OpenRouter API
* **Endpoint:** `GET https://openrouter.ai/api/v1/models`
* **Autenticação:** Pública ou `Authorization: Bearer <OPENROUTER_API_KEY>`
* **Dados Extraídos:**
  * Model ID e slugs alternativos
  * Parâmetros suportados (`supports_tools`, `supports_response_format_json_schema`)
  * Telemetria operacional média: TTFT (ms), Throughput (tokens/s)
  * Preço spot de mercado

### 3.3. Base de Benchmarks Cognitivos (LMSYS / Hugging Face / Static Dataset)
* **Ingestão:** Requisição a endpoints públicos do Space LMSYS ou snapshot versionado JSON embutido na crate com recarga remota.
* **Métricas Extraídas:**
  * **LMSYS Elo Geral:** Avaliação empírica humana cega.
  * **Coding Elo / SWE-bench:** Avaliação de geração de código e resolução de issues reais.
  * **Hard Prompts Elo / GPQA Diamond:** Avaliação de raciocínio lógico sob ambiguidade e complexidade formal.
  * **BFCL (Berkeley Function Calling Leaderboard):** Acurácia de chamadas de funções e parsing AST.
  * **IFEval Score:** Conformidade com restrições sintáticas e instruções negativas.

---

## 4. Esquema de Banco de Dados (SQLite)

O banco de dados deve utilizar o modo WAL (`PRAGMA journal_mode = WAL;`) para concorrência de leitura e escrita limpa.

```sql
-- 1. Modelos Canônicos (Entidades únicas normalizadas)
CREATE TABLE IF NOT EXISTS canonical_models (
    id TEXT PRIMARY KEY,                 -- ex: "anthropic-claude-3-7-sonnet"
    display_name TEXT NOT NULL,          -- ex: "Claude 3.7 Sonnet"
    family TEXT NOT NULL,                -- ex: "Claude", "DeepSeek", "Llama"
    developer TEXT NOT NULL,             -- ex: "Anthropic", "DeepSeek", "Meta"
    context_window INTEGER NOT NULL,
    supports_tools INTEGER NOT NULL DEFAULT 0,
    supports_structured_outputs INTEGER NOT NULL DEFAULT 0,
    supports_caching INTEGER NOT NULL DEFAULT 0,
    is_multimodal INTEGER NOT NULL DEFAULT 0,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- 2. Tabela de Aliases para Resolução de Entidades
CREATE TABLE IF NOT EXISTS model_aliases (
    alias TEXT PRIMARY KEY,              -- ex: "deepseek/deepseek-chat"
    canonical_id TEXT NOT NULL,          -- ex: "deepseek-v3"
    source TEXT NOT NULL,                -- ex: "openrouter", "nous_portal"
    FOREIGN KEY (canonical_id) REFERENCES canonical_models(id)
);

-- 3. Snapshots de Preço e Telemetria Diária
CREATE TABLE IF NOT EXISTS pricing_telemetry_snapshots (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    canonical_id TEXT NOT NULL,
    source TEXT NOT NULL,                -- "nous_portal", "openrouter"
    prompt_price_per_m REAL NOT NULL,    -- USD por 1M tokens
    completion_price_per_m REAL NOT NULL,-- USD por 1M tokens
    cache_read_price_per_m REAL DEFAULT 0.0,
    cache_write_price_per_m REAL DEFAULT 0.0,
    ttft_ms REAL,                        -- Time to first token (milissegundos)
    throughput_tps REAL,                 -- Tokens por segundo
    recorded_at DATE NOT NULL,
    FOREIGN KEY (canonical_id) REFERENCES canonical_models(id)
);

-- 4. Snapshots de Benchmarks Cognitivos
CREATE TABLE IF NOT EXISTS benchmark_snapshots (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    canonical_id TEXT NOT NULL,
    general_elo REAL,
    coding_elo REAL,
    hard_prompts_elo REAL,
    bfcl_score REAL,                     -- 0.0 a 100.0
    ifeval_score REAL,                   -- 0.0 a 100.0
    gpqa_score REAL,                     -- 0.0 a 100.0
    recorded_at DATE NOT NULL,
    FOREIGN KEY (canonical_id) REFERENCES canonical_models(id)
);

-- 5. Avaliações Consolidadas e Índices Calculados
CREATE TABLE IF NOT EXISTS daily_evaluations (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    canonical_id TEXT NOT NULL,
    evaluation_date DATE NOT NULL,
    profile_task TEXT NOT NULL,          -- "coding", "agentic", "reasoning", "high_volume_etl"
    composite_score REAL NOT NULL,       -- 0.0 a 100.0
    task_blended_cost REAL NOT NULL,     -- USD por 1M tokens do perfil
    roi_index REAL NOT NULL,
    is_pareto_optimal INTEGER NOT NULL,  -- 1 se está na fronteira, 0 caso contrário
    FOREIGN KEY (canonical_id) REFERENCES canonical_models(id)
);

CREATE INDEX IF NOT EXISTS idx_pricing_lookup ON pricing_telemetry_snapshots(canonical_id, recorded_at);
CREATE INDEX IF NOT EXISTS idx_bench_lookup ON benchmark_snapshots(canonical_id, recorded_at);
```

---

## 5. Modelagem Matemática e Índices Analíticos

### 5.1. Normalização de Métricas
Para agregar pontuações com diferentes escalas (Elos de 1000 a 1400 e percentuais de 0 a 100), utiliza-se normalização min-max:

$$S_{\text{norm}}(x) = \frac{x - x_{\min}}{x_{\max} - x_{\min}} \times 100$$

### 5.2. Nota Composta Específica por Perfil ($S_{\text{task}}$)

Cada caso de uso demanda pesos distintos para cada dimensão cognitiva:

#### A. Perfil: Engenharia de Software / Coding
$$S_{\text{coding}} = 0.40 \cdot S_{\text{norm}}(\text{Coding Elo}) + 0.25 \cdot S(\text{BFCL}) + 0.20 \cdot S_{\text{norm}}(\text{Hard Elo}) + 0.15 \cdot S(\text{IFEval})$$

#### B. Perfil: Agentes Autônomos e Tool Calling
$$S_{\text{agentic}} = 0.45 \cdot S(\text{BFCL}) + 0.30 \cdot S(\text{IFEval}) + 0.15 \cdot S_{\text{norm}}(\text{Hard Elo}) + 0.10 \cdot S_{\text{norm}}(\text{Throughput})$$

#### C. Perfil: Raciocínio Profundo e Arquitetura
$$S_{\text{reasoning}} = 0.45 \cdot S(\text{GPQA}) + 0.35 \cdot S_{\text{norm}}(\text{Hard Elo}) + 0.20 \cdot S(\text{IFEval})$$

#### D. Perfil: ETL de Alto Volume e Extração Estruturada
$$S_{\text{etl}} = 0.50 \cdot S(\text{IFEval}) + 0.30 \cdot S_{\text{norm}}(\text{Throughput}) + 0.20 \cdot S_{\text{norm}}(\text{General Elo})$$

### 5.3. Custo Ponderado da Carga de Trabalho ($C_{\text{task}}$)

O custo ponderado por 1M tokens varia conforme a proporção típica de tokens de cada perfil:

$$C_{\text{task}} = (w_{\text{cache}} \cdot P_{\text{cache}}) + (w_{\text{in}} \cdot P_{\text{in}}) + (w_{\text{out}} \cdot P_{\text{out}})$$

| Perfil | $w_{\text{cache}}$ | $w_{\text{in}}$ | $w_{\text{out}}$ | Justificativa |
|---|---|---|---|---|
| **Coding** | 0.40 | 0.40 | 0.20 | Arquivos de código longos com saídas médias |
| **Agentic** | 0.60 | 0.30 | 0.10 | Histórico acumulado e prompts de ferramentas; saídas JSON curtas |
| **Reasoning** | 0.20 | 0.40 | 0.40 | Contextos medianos, cadeias longas de raciocínio (Chain-of-Thought) |
| **High-Volume ETL** | 0.10 | 0.80 | 0.10 | Documentos gigantescos para extrações curtas |

### 5.4. Cálculo de Retorno sobre Custo (Índice de ROI Amortecido)

Para evitar que modelos ultra-baratos com nota baixa dominem a classificação devido à divisão por números próximos de zero, usa-se amortecimento logarítmico:

$$V_{\text{task}} = \frac{S_{\text{task}}}{\ln(1 + C_{\text{task}})}$$

### 5.5. Algoritmo da Fronteira de Pareto
Um modelo $A$ domina um modelo $B$ se:
1. $S_{\text{task}}(A) \ge S_{\text{task}}(B)$ e $C_{\text{task}}(A) \le C_{\text{task}}(B)$
2. Ao menos uma das desigualdades é estrita ($>$ ou $<$).

Modelos que não são dominados por nenhum outro pertencem à **Fronteira de Pareto** e são marcados com `is_pareto_optimal = 1`.

---

## 6. Estrutura da Crate em Rust

### 6.1. Árvore de Arquivos do Projeto

```text
llm-cost-matrix/
├── Cargo.toml
├── README.md
├── src/
│   ├── main.rs
│   ├── lib.rs
│   ├── config.rs
│   ├── error.rs
│   ├── canonical/
│   │   ├── mod.rs
│   │   ├── resolver.rs
│   │   └── seed_aliases.rs
│   ├── sources/
│   │   ├── mod.rs
│   │   ├── nous.rs
│   │   ├── openrouter.rs
│   │   └── benchmarks.rs
│   ├── storage/
│   │   ├── mod.rs
│   │   ├── migrations.rs
│   │   └── repository.rs
│   ├── analytics/
│   │   ├── mod.rs
│   │   ├── scoring.rs
│   │   ├── pareto.rs
│   │   └── profiles.rs
│   └── report/
│       ├── mod.rs
│       ├── markdown.rs
│       └── terminal.rs
└── seeds/
    └── benchmark_seed.json
```

### 6.2. Dependências Críticas (`Cargo.toml`)

```toml
[package]
name = "llm-cost-matrix"
version = "0.1.0"
edition = "2021"
authors = ["Engenharia de IA"]
description = "Ingestão diária, análise de fronteira de Pareto e scoring de custo-benefício para LLMs"

[dependencies]
# CLI & Runtime
clap = { version = "4.5", features = ["derive", "env"] }
tokio = { version = "1.40", features = ["full"] }
chrono = { version = "0.4", features = ["serde"] }

# HTTP & Serde
reqwest = { version = "0.12", features = ["json", "rustls-tls"], default-features = false }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"

# Banco de Dados
rusqlite = { version = "0.32", features = ["bundled", "chrono"] }

# Apresentação e Visualização
comfy-table = "7.1"
colored = "2.1"
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }
thiserror = "1.0"
```

---

## 7. Módulos e Responsabilidades Detalhadas

### 7.1. Módulo `canonical::resolver`
Responsável por pegar strings heterogêneas dos provedores e mapear para a chave canônica:
* `"nousresearch/hermes-3-llama-3.1-405b"` $\to$ `"hermes-3-llama-3.1-405b"`
* `"deepseek/deepseek-chat"` $\to$ `"deepseek-v3"`
* `"anthropic/claude-3-7-sonnet:beta"` $\to$ `"claude-3-7-sonnet"`
* Inclui um fallback de normalização por regex caso o alias exato não exista no banco.

### 7.2. Módulo `sources::nous`
* Consome a API do Nous Portal.
* Converte unidades: caso a API forneça custo por token individual, multiplica por $10^6$ para obter USD por 1M tokens.
* Trata timeouts com backoff exponencial simples.

### 7.3. Módulo `analytics::pareto`
* Implementa o algoritmo de ordenação para identificar a casca convexa/fronteira não dominada para cada perfil.
* Retorna uma lista com a flag booleana de optimalidade para persistência.

### 7.4. Módulo `report::markdown`
* Gera o artefato final diário com:
  1. **Tabela Resumo da Fronteira de Pareto** para cada perfil (`Coding`, `Agentic`, `Reasoning`, `ETL`).
  2. **Comparativo Nous Portal vs. OpenRouter** (onde o mesmo modelo está mais barato ou tem menor latência).
  3. **Recomendações Diretas de Roteamento:** Modelo Campeão de Qualidade, Modelo Melhor Custo x Benefício e Modelo de Menor Custo Funcional.

---

## 8. Roteiro de Execução para o Agente na IDE

Instruções sequenciais para orientar o agente de IA durante a criação do projeto:

```text
ETAPA 1: Setup do Projeto e Banco
1.1 Inicializar Cargo.toml com as dependências listadas na seção 6.2.
1.2 Criar src/error.rs com definições de erro usando thiserror.
1.3 Implementar src/storage/migrations.rs executando o DDL SQL completo da seção 4.
1.4 Implementar src/storage/repository.rs com transações e upserts para as tabelas.

ETAPA 2: Camada de Resolução Canônica
2.1 Implementar seed inicial com mapeamento dos 30 principais modelos de mercado 
    (Anthropic Claude 3.5/3.7, DeepSeek V3/R1, Qwen 2.5/Coder, OpenAI GPT-4o/o1/o3, Llama 3.1/3.3).
2.2 Criar função de resolução em src/canonical/resolver.rs que consulta a tabela model_aliases.

ETAPA 3: Clientes HTTP e Ingestores (Sources)
3.1 Implementar src/sources/nous.rs tipando o retorno do endpoint /v1/models.
3.2 Implementar src/sources/openrouter.rs extraindo limites, precificação e métricas de SLA.
3.3 Implementar src/sources/benchmarks.rs carregando o dataset de métricas cognitivas.

ETAPA 4: Motor de Analytics
4.1 Implementar src/analytics/profiles.rs definindo os pesos de cada perfil.
4.2 Implementar src/analytics/scoring.rs com normalização e cálculo de S_task e C_task.
4.3 Implementar src/analytics/pareto.rs com o algoritmo de dominância de Pareto.

ETAPA 5: CLI e Gerador de Relatório
5.1 Criar comandos no clap em src/main.rs:
    - ingest: Faz o scrape e salva no banco.
    - score: Recalcula notas e avaliações de Pareto do dia.
    - report: Exporta relatório em Markdown e exibe tabela no terminal.
    - run-daily: Executa ingest + score + report em sequência.
5.2 Implementar saída visual em tabela via comfy-table e exportação para arquivo markdown.
```

---

## 9. Formato do Relatório Gerado (`daily_llm_matrix.md`)

Exemplo da saída gerada pelo comando `llm-cost-matrix report`:

```markdown
# Relatório de Custo-Benefício de LLMs — 2026-10-09

## 1. Campeões por Perfil de Carga (Fronteira de Pareto)

### Perfil: Autonomous Coding & SWE
| Modelo Canônico | Provedor Mais Barato | Score Coding (0-100) | Custo Blended (/1M) | Throughput | Status Pareto |
|---|---|---|---|---|---|
| **claude-3-7-sonnet** | Nous Portal | 94.6 | $3.40 | 68 t/s | **Ótimo (Qualidade)** |
| **deepseek-v3** | Nous Portal | 88.2 | $0.28 | 92 t/s | **Ótimo (Custo x Benefício)** |
| **qwen-2.5-coder-32b**| OpenRouter | 82.5 | $0.18 | 115 t/s | **Ótimo (Ultra Low-Cost)** |

### Perfil: Agentes Autônomos & Tool Calling (BFCL + IFEval)
| Modelo Canônico | Provedor Mais Barato | Score Agentic (0-100) | Custo Blended (/1M) | Suporta Schema? | Status Pareto |
|---|---|---|---|---|---|
| **claude-3-7-sonnet** | Nous Portal | 95.1 | $2.60 | Sim | **Ótimo (Qualidade)** |
| **gpt-4o-mini** | OpenRouter | 78.4 | $0.14 | Sim | **Ótimo (Alto Volume)** |
| **deepseek-v3** | Nous Portal | 89.0 | $0.24 | Sim | **Ótimo (Custo x Benefício)** |

---

## 2. Recomendações de Roteamento para Produção

* **Tarefas de Programação / Refatoração Crítica:** Roteie primariamente para `claude-3-7-sonnet` (via Nous Portal para cache read de menor custo).
* **Agentes de Raciocínio & Triagem Intermediária:** Roteie para `deepseek-v3`, que oferece 92% da performance do líder custando menos de 10% do valor.
* **Tarefas de Extração Estruturada Massiva:** Use `gpt-4o-mini` ou `qwen-2.5-coder-32b`.
```

---

## 10. Implementação Canônica e Simbiose Local-Nuvem no Souls Engine v7 (ADR-008)

Em outubro de 2026, a especificação técnica deste documento foi formalmente implementada e adaptada para a arquitetura bare-metal do **Souls Engine v7**, estabelecendo uma ponte simbiótica entre os **38 modelos locais** (Tiers 0 a 4) e o catálogo de nuvem do **Nous Research Portal**.

### 10.1. Da Nuvem ao Silício Bare-Metal: O Índice $C_{\text{silicon}}$
Enquanto em nuvem o custo é mensurado em dólares por milhão de tokens ($C_{\text{task}}$), no ambiente bare-metal local (host Intel Core i9 + NVIDIA GeForce RTX 2060m) o custo é definido pelo consumo físico de recursos:

$$C_{\text{silicon}} = 0.50 \cdot \left(\frac{\text{VRAM}_{\text{MB}}}{5294}\right) + 0.35 \cdot \left(\frac{\text{TTFT}_{\text{ms}}}{3000}\right) + 0.15 \cdot \left(\frac{35.0}{\max(\text{TPS}, 1.0)}\right)$$

### 10.2. Os 4 Perfis Canônicos Unificados
O Souls Engine consolidou as categorias ideadas neste documento na tabela SSOT `model_task_profiles` e nas colunas analíticas da tabela `model_registry` no banco SQLite STRICT WAL (`souls_heuristic_vault.db`):

1. **Coding & SWE Profile ($S_{\text{coding}}$):** $0.50 \cdot \text{AST} + 0.25 \cdot \text{JSON} + 0.15 \cdot \text{Reasoning} + 0.10 \cdot \text{Needle}$.
2. **Autonomous Agentic Profile ($S_{\text{agentic}}$):** $0.50 \cdot \text{JSON} + 0.25 \cdot \text{AST} + 0.15 \cdot \text{Reasoning} + 0.10 \cdot \text{Needle}$.
3. **Deep Cognitive Reasoning Profile ($S_{\text{reasoning}}$):** $0.65 \cdot \text{Reasoning} + 0.15 \cdot \text{AST} + 0.10 \cdot \text{JSON} + 0.10 \cdot \text{Needle}$.
4. **Fast Heuristic Triage Profile ($S_{\text{fast}}$):** $0.40 \cdot \text{Throughput Norm} + 0.30 \cdot \text{TTFT Norm} + 0.15 \cdot \text{JSON} + 0.15 \cdot \text{AST}$.

### 10.3. ROI Amortecido com Penalidade de Silício ($V_{\text{task}}$)
Para evitar que modelos ultra-leves de baixa cognição dominassem artificialmente o ranking pelo denominador baixo, adotou-se o fator de amortecimento logarítmico:

$$V_{\text{task}} = \frac{S_{\text{task}}}{1.0 + \ln(1.0 + C_{\text{silicon}})}$$

### 10.4. Cadeias Determinísticas de Fallback (Local $\to$ Nuvem Nous Portal)
As rotas de despacho do `ParetoBandit` e do Hermes Agent seguem a progressão contínua:

| Perfil | Campeão Local de Silício | Fallback Local #1 | Fallback Nuvem #1 (Nous Portal) | Fallback Nuvem #2 (Nous Portal) |
|---|---|---|---|---|
| **Autonomous Coding** | `qwen3.5-4b-super-coder` (50.1 t/s) | `qwen3.5-4b-uncensored-mtp` | `claude-3-7-sonnet` ($3.40/M) | `deepseek-v3` ($0.28/M) |
| **Agentic Tool Calling**| `qwen3.5-4b-uncensored-mtp` (51.2 t/s)| `qwen3.5-4b-agentic-coder-v4` | `claude-3-7-sonnet` ($2.60/M) | `deepseek-v3` ($0.24/M) |
| **Deep Reasoning** | `qwen3.5-4b-uncensored-mtp` (51.2 t/s)| `phi-4-mini-reasoning` / `laguna-xs` | `deepseek-r1` ($0.55/M) | `claude-3-7-sonnet` |
| **Fast / Triage** | `smollm2-135m-reasoning` (149.5 t/s) | `neuralai-mamba-k1` (83.7 t/s) | `qwen-2.5-coder-32b` | `gpt-4o-mini` ($0.14/M) |

Esta governança une previsibilidade física bare-metal (zero custo de API na dGPU/Host) com escalabilidade resiliente em nuvem nos cenários de esgotamento de contexto ou complexidade extrema.