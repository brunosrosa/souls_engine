# DESIGN.md — Especificação de Design da Sessão 3: Conexão Viva com Hermes Agent (v7)

> **ESTATUS DO DOCUMENTO:** CANÔNICO E APROVADO (SDD)
> **SUB-SISTEMA:** Simbiose Hermes Agent Desktop & Ativação de Handlers MCP de Memória
> **CRATES DIRETAMENTE AFETADAS:** `crates/souls_server` (Single Crate Rule)

---

## 1. CONEXÃO MCP DO HERMES AGENT (`config.yaml`)

No arquivo de configuração do Hermes Agent (`%LOCALAPPDATA%\hermes\config.yaml`), injeta-se o bloco do Souls Engine sob o transporte oficial moderno Streamable HTTP (2025-03-26+):

```yaml
mcp_servers:
  souls_engine:
    url: "http://127.0.0.1:9123/mcp"
    transport: "http"
```

E para o provider local `llama.cpp` apontando para o daemon `:9123`:

```yaml
providers:
  souls_local:
    type: "openai"
    base_url: "http://127.0.0.1:9123/v1"
    api_key: "souls-local-token"
```

---

## 2. INJEÇÃO DE GOVERNANÇA AGÊNTICA (`SOUL.md`)

No arquivo de persona e regras globais do Hermes (`%LOCALAPPDATA%\hermes\SOUL.md`), anexa-se a cláusula de subagentes e FinOps:

```markdown
<!-- SOULS ENGINE FINOPS & SUBAGENT GOVERNANCE -->
### GOVERNANÇA DE SUBAGENTES E FINOPS (SOULS ENGINE)
1. **Consulta Mandatória de Despacho:** Antes de invocar a ferramenta `delegate_task` para qualquer tarefa, você DEVE compulsoriamente consultar a ferramenta `route_subtask` informando `task_description` e `estimated_input_tokens`.
2. **Respeito aos Tiers:** Se o `route_subtask` indicar silício local (Tier 1 / RTX 2060m), configure a tarefa para o provider `souls_local`. Se prescrever Tier 2, delegue para nuvem econômica.
3. **Persistência de Memória:** Utilize a ferramenta `handoff` para transferir contexto e `sub_agent` para acompanhar os diários de execução, evitando poluir o contexto mestre.
```

---

## 3. UNIFICAÇÃO DE DIRETÓRIOS VIA JUNCTION POINT

Execução de Junction Point transparente no Windows NT:
- Origem: `%LOCALAPPDATA%\hermes\models`
- Alvo: `C:\Users\rosas\llm_local_models`
- Vantagem: Sem duplicação de arquivos GGUF (economia de 32+ GB no NVMe do usuário).

---

## 4. CONTRATOS DE HANDLERS MCP REAIS EM `souls_server`

Substituição dos fallbacks de staging em `crates/souls_server/src/mcp/handlers.rs` por implementações reais acopladas ao `state.memory` ([`SoulsMemoryStore`](file:///Z:/souls_engine/crates/souls_memory/src/lib.rs)):

| Garra MCP | Função Handler em `handlers.rs` | Chamada Interna em `SoulsMemoryStore` |
| :--- | :--- | :--- |
| `handoff` | `handle_handoff` | `state.memory.sub_agent_manager().create_handoff(...)` e `get_handoff(...)` |
| `sub_agent` | `handle_sub_agent` | `state.memory.sub_agent_manager().spawn_subagent(...)`, `update_status(...)` |
| `thinking` | `handle_thinking` | `state.memory.get_or_create_thinking_engine(...).add_thought(...)` |
| `session` | `handle_session` | Inspeciona ou limpa sessão ativa no `ThinkingEngine` |
| `analyze_session` | `handle_analyze_session` | `engine.analyze_coherence()` |
| `export_session` | `handle_export_session` | `engine.export_markdown()` ou `export_json()` |
| `merge_sessions` | `handle_merge_sessions` | `engine.merge_branch(...)` |
| `knowledge` | `handle_knowledge` | `state.memory.upsert_epistemic_memory(...)` e `get_epistemic_memory(...)` |
| `sqlite_query` | `handle_sqlite_query` | `state.memory.execute_read_only_query(...)` |
| `semantic_search` | `handle_semantic_search` | `state.memory.search_epistemic_fts5(...)` |
| `mem_create_entities` | `handle_mem_create_entities` | `state.memory.ladybug().upsert_node(...)` |
| `mem_create_relations` | `handle_mem_create_relations` | `state.memory.ladybug().upsert_edge(...)` |
| `mem_read_graph` | `handle_mem_read_graph` | `state.memory.ladybug().read_graph(...)` |
