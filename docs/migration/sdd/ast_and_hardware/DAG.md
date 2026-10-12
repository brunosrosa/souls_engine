# DAG.md — Grafo de Dependências Dirigido da Sessão 4 (v7)

> **ESTATUS DO DOCUMENTO:** CANÔNICO & VINCULANTE (SDD)  
> **REPRESENTAÇÃO:** Grafo Acíclico Dirigido das Tarefas Atômicas da Sessão 4

---

## 1. SEQUÊNCIA DE EXECUÇÃO E RESTRIÇÃO DE CRATE ÚNICA

```mermaid
graph TD
    T1["TASK-AST-001: APIs em souls_ast"] --> T3["TASK-AST-003: Handlers AST em souls_server"]
    T2["TASK-AST-002: Probing em souls_inference_runtime"] --> T4["TASK-AST-004: Handler Intent em souls_server"]
    T3 --> T5["TASK-AST-005: Roteamento no Dispatcher"]
    T4 --> T5
    T5 --> T6["TASK-AST-006: Testes de Integração MCP"]
    T6 --> T7["TASK-AST-007: Verificação Compulsória"]
```

## 2. RAZÃO DE DEPENDÊNCIAS
1. **TASK-AST-001** desbloqueia os handlers de desidratação e diff em **TASK-AST-003**.
2. **TASK-AST-002** desbloqueia o handler de avaliação epistêmica em **TASK-AST-004**.
3. Ambos os blocos convergem no despachante de rotas em **TASK-AST-005**.
4. Testes e validação final encerram a sessão em **TASK-AST-006** e **TASK-AST-007**.
