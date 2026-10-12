# PLAN.md — Plano de Transplante e Consolidação do Cérebro L3 (`souls_memory`) (v7 Canônica)

> **ESTATUS DO DOCUMENTO:** CANÔNICO E APROVADO (SDD)
> **SUB-SISTEMA:** Hipocampo L3 Soberano & Memória de Sessões de Subagentes
> **CRATE ALVO:** `crates/souls_memory`
> **SESSÃO DO ROADMAP:** Sessão 2 (Bloco D: Cérebro L3)

---

## 1. RACIONAL ARQUITETURAL E OBJETIVO

O **Cérebro L3** é o alicerce hipocampal do Souls Engine. Ele fornece soberania epistêmica, persistência atômica e arbitragem cognitiva sem depender de proxies de rede, janelas nativas ou memória de curto prazo do interpretador Python do Hermes Agent.

Na Sessão 1, estabelecemos o daemon bare-metal `souls_server.exe` (:9123) com o barramento MCP de 51 ferramentas, catalogadas com *Progressive Tool Disclosure*.
Na **Sessão 2**, nosso objetivo é transformar os stubs de memória e cognição em implementações de produção de alta fidelidade dentro de `crates/souls_memory`, ancoradas no **FrankenSQLite STRICT WAL** e em estruturas em RAM (`petgraph`, `DashMap`).

### Responsabilidades Canônicas da Sessão 2:
1. **Governança de Subagentes e Handoff:**
   - Persistir o ciclo de vida e estado de transição de subagentes delegados (`sub_agent_sessions`).
   - Armazenar envelopes de variáveis de ambiente e resumo de contexto entre o modelo mestre e os subagentes (`handoff_payloads`).
   - Registrar notas operacionais e reflexões atômicas dos subagentes em tempo de execução (`sub_agent_diaries`).
2. **Cognição Socrática e Grafo DAG de Hipóteses:**
   - Transplantar o `ThinkingEngine` para raciocínio estruturado multi-passos (`thinking`/`core_think`).
   - Suportar os 4 paradigmas cognitivos: `LinearCoT`, `TreeToT`, `GraphGoT` e `CouncilMAD`.
   - Permitir ramificação de hipóteses (`branch_from_thought`), retificação de passos anteriores (`is_revision`, `revises_thought`) e disjuntor de sobre-raciocínio (*overthinking limit* = 5, HITL = 7).
   - Oferecer fusão de deliberações concorrentes (`merge_sessions`) e exportação em Markdown/JSON (`export_session`, `analyze_session`).
3. **Ponte Assíncrona MPSC Desacoplada:**
   - Evitar que chamadas RPC no despachante MCP sofram latência de disco I/O através de um canal MPSC bufferizado com worker assíncrono Tokio.
4. **Respeito Estrito à Regra de Crate Única:**
   - Modificar exclusivamente os arquivos pertencentes a `crates/souls_memory` e documentação de apoio em `docs/`.

---

## 2. INVENTÁRIO FORENSE DO MATERIAL EM `_donor/`

| Arquivo de Origem no `_donor/` | Papel Funcional Original | Classificação de Triagem | Destino Canônico em `crates/souls_memory/` |
| :--- | :--- | :--- | :--- |
| `souls_mc_core/src/cognition/state_thinking/thinking/engine.rs` | `ThinkingEngine`, paradigmas de raciocínio, branches e limites de overthinking. | **HEALTHY_ORGAN** | `crates/souls_memory/src/thinking.rs` |
| `souls_mc_core/src/cognition/state_thinking/thinking/ops.rs` | DDL V5 e persistência socrática em SQLite (`socratic_sessions`, `socratic_thoughts`). | **ORGAN_REPAIR** (Converter de `rusqlite` síncrono para `sqlx` STRICT WAL assíncrono). | `crates/souls_memory/src/sqlite.rs` |
| `souls_mc_core/src/cognition/state_thinking/thinking/persistence.rs` | Tipos de pensamento (`ThoughtType`, `SocraticThought`, `SessionId`). | **HEALTHY_ORGAN** | `crates/souls_memory/src/thinking.rs` |
| `souls_mc_core/src/cognition/state_thinking/memory_graph/mpsc_bridge.rs` | Canal MPSC com worker em thread dedicada consumindo operações de escrita. | **ORGAN_REPAIR** (Modernizar para `tokio::sync::mpsc` com worker nativo Tokio assíncrono). | `crates/souls_memory/src/mpsc_bridge.rs` |
| `souls_mc_core/src/cognition/state_thinking/thinking/worker.rs` | Worker de gravação de diários (`sub_agent_diaries`) e sessões socráticas. | **ORGAN_REPAIR** (Consolidar no `session_manager.rs` e `mpsc_bridge.rs`). | `crates/souls_memory/src/session_manager.rs` |

---

## 3. GUARDRAILS OPERACIONAIS

1. **FrankenSQLite STRICT Obrigatório:** Nenhuma tabela é criada sem `STRICT`. Tipos não conformes são rejeitados no ato da inserção.
2. **Zero Singletons Globais:** Todo o estado reside em instâncias coordenadas por `SoulsMemoryStore` e injetadas via `Arc`.
3. **Prevenção de Inanição Tokio:** Consultas e deserializações volumosas rodam de forma não-bloqueante no pool assíncrono `sqlx`.
4. **Finais de Linha LF e UTF-8 Puro:** Todo arquivo é salvo em LF sem BOM.
5. **Aderência aos ADRs:** ADR-001 (Dev Drive ReFS), ADR-003 (Imutabilidade do Chat Mestre), ADR-040 (FrankenSQLite STRICT WAL).
