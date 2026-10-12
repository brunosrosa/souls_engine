---
name: souls-socratic-thinking
description: Playbook do freio cognitivo socrático. Força deliberação estruturada em grafo DAG (hipóteses, bifurcações e revisões) antes de tocar em código de alta entropia ou contratos de dados.
metadata:
  version: "1.0.0"
  author: "Souls Engine Architecture"
  namespace: "think"
---

# Souls Socratic Thinking — Playbook do Freio Cognitivo DAG (v7)

Este playbook rege o processo deliberativo em tarefas de alta entropia, prevenindo o "vibe coding" através do motor de raciocínio socrático (`core_think`).

## 🛑 QUANDO ACIONAR O FREIO COGNITIVO
O freio socrático deve ser ativado compulsoriamente antes de:
1. Criar ou alterar contratos de banco de dados (`SCHEMAS_DDL_MANIFEST.md`).
2. Implementar novas crates ou mudar topologia do workspace.
3. Decidir entre alternativas de concorrência Tokio, FFI unsafe ou alocação de silício.
4. Resolver bugs sutis onde a causa-raiz é incerta.

---

## 🛠️ FLUXO OPERACIONAL EM GRAFO DAG

```
1. thinking(branch) ─────► 2. thinking(revise) ─────► 3. analyze_session ─────► 4. Execução
   (Hipótese & Trade-offs)    (Resolução de Conflitos)   (Validação de Consistência)   (Código Validado)
```

### 1. Início de Sessão & Levantamento de Hipóteses
Inicie a deliberação registrando a premissa inicial:
```json
thinking({
  "action": "branch",
  "hypothesis": "Adotar mpsc::channel com buffer limitado para absorver picos de I/O",
  "trade_offs": "Consome ~4MB de heap vs impede travamento da thread Tokio principal"
})
```

### 2. Ramificação de Alternativas Concorrentes
Se existirem múltiplas abordagens viáveis:
* Crie bifurcações no grafo explorando cada hipótese com seus prós e contras.
* Avalie o custo FinOps e a pressão térmica sobre a dGPU para cada alternativa.

### 3. Revisão Socrática e Poda de Ramos Inválidos
Se uma premissa violar um guardrail ([AGENTS.md](file:///Z:/souls_engine/AGENTS.md)) ou colidir com uma verdade ontológica:
```json
thinking({
  "action": "revise",
  "node_id": "hipotese_1",
  "reason": "Violou a regra de exclusão mútua de VRAM sob WDDM (ADR-004)"
})
```

### 4. Análise de Coerência com `analyze_session`
Antes de começar a escrever o código:
* Invoque `analyze_session()`.
* O motor analisa a profundidade lógica, valida se todas as contradições foram resolvidas e se a cadeia de raciocínio está matematicamente e filosoficamente alinhada.
