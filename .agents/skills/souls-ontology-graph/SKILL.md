---
name: souls-ontology-graph
description: Playbook para navegação e manipulação do Grafo Ontológico LadybugDB (L2) no Souls Engine. Governa entidades, observações clínicas e arestas tipadas (depends_on, conflicts_with, implements, relates_to).
metadata:
  version: "1.0.0"
  author: "Souls Engine Architecture"
  namespace: "memory"
---

# Souls Ontology Graph — Playbook do Grafo LadybugDB (v7)

Este playbook rege a modelagem de entidades e relações conceituais no grafo ontológico LadybugDB (`petgraph` em RAM serializado em SQLite STRICT).

## 🕸️ ESTRUTURA DO GRAFO
O grafo mapeia entidades do projeto e suas interconexões com tipagem semântica rigorosa:
* **Nós (`entities`):** Crates, ferramentas MCP, modelos de linguagem, arquivos de arquitetura ou conceitos de domínio.
* **Arestas Tipadas (`relationships`):**
  * `depends_on`: Relação de dependência estrita (ex.: `souls_server` depende de `souls_memory`).
  * `conflicts_with`: Incompatibilidade mútua ou regra de exclusão (ex.: `llama-cuda` conflita com `tier2-moe` na VRAM).
  * `implements`: Conexão entre implementação e contrato (ex.: `souls_ast` implementa `outline`).
  * `relates_to`: Associação semântica livre entre entidades.

---

## 🛠️ FLUXO OPERACIONAL

### 1. Descoberta de Topologia com `mem_open_nodes` e `mem_search`
* Para buscar entidades por nome ou termo:
  * `mem_search(query="model_router")`: busca rápida via FTS5.
* Para inspecionar uma entidade e suas arestas ativas:
  * `mem_open_nodes(names=["souls_llm_cloud_arena"])`: carrega nós, observações clínicas e conexões vizinhas.

### 2. Criação Idempotente de Entidades
Ao registrar um novo subsistema ou componente:
```json
mem_create_entities({
  "entities": [
    {
      "name": "souls_llm_cloud_arena",
      "entity_type": "crate",
      "observations": [
        "10ª crate canônica do workspace",
        "Responsável por ingestão de mercado e Pareto nos 9 perfis"
      ]
    }
  ]
})
```

### 3. Conexão Semântica com `mem_create_relations`
Para conectar entidades recém-criadas ou existentes:
```json
mem_create_relations({
  "relations": [
    {
      "from": "souls_server",
      "to": "souls_llm_cloud_arena",
      "relation_type": "depends_on"
    }
  ]
})
```

### 4. Barreira Ontológica contra Conflitos
Se o LadybugDB detectar que uma memória proposta possui aresta `conflicts_with` contra uma entidade `STABLE`, a inserção é bloqueada com código de domínio `-32005` (`OntologicalBarrierViolation`), impedindo a poluição do sistema com diretrizes incompatíveis.
