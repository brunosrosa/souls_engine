---
name: souls-blast-radius
description: Playbook para medição do raio de impacto (Blast Radius) e Frecency antes de refatorações estruturais. Governa repo_impact e repo_heatmap para prevenção de quebras colaterais.
metadata:
  version: "1.0.0"
  author: "Souls Engine Architecture"
  namespace: "ops"
---

# Souls Blast Radius & Frecency — Playbook de Análise de Impacto (v7)

Este playbook rege a inspeção de risco estrutural e relevância histórica de código antes de qualquer refatoração que altere assinaturas públicas ou traits compartilhadas.

## 🎯 FLUXO OPERACIONAL

### 1. Cálculo do Raio de Impacto com `repo_impact`
Antes de modificar um símbolo ou struct em crates de fundação (`souls_core`, `souls_protocol`, `souls_memory`):
```json
repo_impact({
  "path": "crates/souls_protocol/src/dto.rs",
  "symbol": "EpistemicMemoryRecord"
})
```
* **O Motor Avalia:** Executa BFS reversa no grafo de AST para mapear todas as structs, funções e arquivos que colapsarão se a assinatura mudar.
* **Critério de Decisão:** Se o blast radius for $\ge 5$ arquivos, o agente deve compulsoriamente planejar a refatoração via `souls-socratic-thinking` antes de editar.

### 2. Medição de Calor de Desenvolvimento com `repo_heatmap`
Para inspecionar quais arquivos sofrem mutações mais frequentes:
* `repo_heatmap()`: analisa o histórico de commits do repositório bare-metal via biblioteca `gitoxide` (`gix`), calculando a Frecency combinada de recência e frequência de commits.
