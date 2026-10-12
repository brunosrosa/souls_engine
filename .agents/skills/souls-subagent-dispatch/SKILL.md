---
name: souls-subagent-dispatch
description: Playbook para orquestração de subagentes e despacho no silício. Rege a consulta mandatória ao ParetoBandit antes de delegate_task, gerenciamento de handoff e ciclo de vida isolado.
metadata:
  version: "1.0.0"
  author: "Souls Engine Architecture"
  namespace: "ops"
---

# Souls Subagent Dispatch — Playbook de Despacho e Silício (v7)

Este playbook rege a delegação de tarefas do chat mestre do Hermes Agent para subagentes autônomos (`delegate_task`), garantindo conformidade com a barreira termodinâmica da RTX 2060m e o roteamento ótimo de Pareto.

## 🚨 REGRA CONSTITUCIONAL DO PARETOBANDIT (ADR-003)
O modelo do chat interativo mestre é **estritamente fixo e imutável intradiálogo** para proteger o *prefix caching*. O ParetoBandit atua **exclusivamente na fronteira de despacho de subagentes** (`delegate_task`).

---

## 🛠️ FLUXO DE DESPACHO EM 4 ETAPAS

```
1. route_subtask ────────► 2. handoff ────────► 3. delegate_task ────────► 4. sub_agent(complete)
   (Prescrição de Modelo)   (Isolamento Contexto)  (Execução no Hermes)      (Fechamento e Métricas)
```

### Etapa 1: Consulta Prévia ao `route_subtask`
Antes de disparar qualquer subagente:
```json
route_subtask({
  "task_profile": "coding", // ou "agentic", "fast", "reasoning", "curation", etc.
  "prompt_sample": "Refatorar módulo heatmap.rs com novos testes unitários",
  "max_cost_usd": 0.00
})
```
* **O Roteador Avalia:**
  1. VRAM livre na RTX 2060m (teto útil de 5.294 MB sob WDDM).
  2. Temperatura da GPU via NVML (se $T \ge 78^\circ\text{C}$, desvia para CPU ou Nuvem).
  3. Thompson Sampling bayesiano com notas empíricas $E^3$ históricas.
* **Retorno:** Retorna o `prescribed_model`, o `tier` e se deve rodar localmente ou na nuvem.

### Etapa 2: Registro de Handoff
Para não poluir a janela de contexto do chat mestre:
* Invoque `handoff(session_id="...", payload="instruções detalhadas para o worker")`.
* O payload é gravado no SQLite STRICT e o subagente inicia com uma memória de trabalho limpa.

### Etapa 3: Execução da Tarefa no Hermes
* Inicie o subagente delegando a tarefa com o modelo prescrito pelo `route_subtask`.

### Etapa 4: Fechamento de Sessão e Telemetria
Quando o subagente terminar:
* Chame `sub_agent(action="complete", session_id="...", outcome="SUCCESS", tokens_used=1250)`.
* O ParetoBandit atualiza as distribuições Beta ($\alpha$ e $\beta$) do modelo correspondente para calibrar futuras escolhas.
