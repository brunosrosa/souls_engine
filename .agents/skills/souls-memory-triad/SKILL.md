---
name: souls-memory-triad
description: Playbook de consulta e persistência no hipocampo bare-metal L2 do Souls Engine. Governa busca híbrida RRF (LanceDB + FTS5), versionamento de verdades epistêmicas e consultas seguras no FrankenSQLite STRICT WAL.
metadata:
  version: "1.0.0"
  author: "Souls Engine Architecture"
  namespace: "memory"
---

# Souls Memory Triad — Playbook do Hipocampo Bare-Metal L2 (v7)

Este playbook rege a consulta e persistência de memórias de arquitetura e fatos consolidados no Souls Engine (`127.0.0.1:9123`).

## 🧠 A TRÍADE DE MEMÓRIA L2
O Souls Engine não confia em memória conversacional efêmera. O hipocampo L2 é estruturado em três pilares:
1. **Busca Híbrida RRF (`semantic_search`):** Fusão de BM25 (SQLite FTS5) com busca vetorial cosseno (LanceDB sobre Apache Arrow) operando a **estritamente 0 MB de VRAM** da dGPU.
2. **Cofre Epistêmico Versionado (`knowledge`):** Armazena regras arquiteturais, contratos imutáveis (`STABLE`) e hipóteses em evolução (`EVOLVING`).
3. **Consultas Estruturadas (`sqlite_query`):** Consultas SQL `SELECT` puras em tabelas `STRICT` no FrankenSQLite WAL.

---

## 🛠️ FLUXO OPERACIONAL

### 1. Consulta Prévia Obrigatória (Evitar Alucinações)
Antes de responder a dúvidas de design, dependências entre crates ou regras de negócio:
* Execute `semantic_search(query="sua dúvida ou conceito", limit=5)`.
* O algoritmo RRF (`Reciprocal Rank Fusion`) computa:
  $$RRF(d) = \sum_{m \in \{BM25, LanceDB\}} \frac{1}{60 + \text{rank}_m(d)} + \text{ExactBonus}$$
* Inspecione as memórias retornadas. Se houver contradição com o prompt do usuário, a verdade de **L2** prevalece.

### 2. Versionamento de Fatos com `knowledge`
Quando uma decisão arquitetural for ratificada ou uma nova regra for estabelecida:
* Invoque `knowledge(action="store", category="architecture", content="Descrição formal da regra", partition="STABLE")`.
* Partições aceitas:
  * `STABLE`: Imunidade ontológica contra esquecimento e expurgo do Langevin Decay.
  * `EVOLVING`: Memórias dinâmicas que decaem com o tempo se não forem reutilizadas.

### 3. Inspeção Relacional via `sqlite_query`
Para auditar tabelas do `souls_heuristic_vault.db` ou `souls_state.db`:
* Invoque `sqlite_query(query="SELECT * FROM canonical_models WHERE developer = 'Anthropic' LIMIT 10")`.
* **Guardrail Inviolável:** Apenas queries `SELECT` de leitura são permitidas. Qualquer tentativa de `INSERT`, `UPDATE` ou `DROP` dispara erro imediato de segurança (`SqlStrictViolation`).
