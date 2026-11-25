---
name: souls-subagent-dispatch
description: Playbook para orquestração de subagentes e despacho no silício. Rege a consulta mandatória ao ParetoBandit antes de delegate_task, gerenciamento de handoff e ciclo de vida isolado.
metadata:
  version: "1.0.0"
  author: "Souls Engine Architecture"
  namespace: "ops"
---

# Souls Subagent Dispatch — Playbook de Despacho e Silício

Este playbook rege a delegação de tarefas do chat mestre do Hermes Agent para subagentes autônomos (`delegate_task`), garantindo conformidade com a barreira termodinâmica da RTX 2060m e o roteamento ótimo de Pareto.

## 🚨 REGRA CONSTITUCIONAL DO PARETOBANDIT (ADR-003)
O modelo do chat interativo mestre é **estritamente fixo e imutável intradiálogo** para proteger o *prefix caching*. O ParetoBandit atua **exclusivamente na fronteira de despacho de subagentes** (`delegate_task`).

### Escopo do veto de custo (não confundir com o chat mestre)
O **chat mestre pago é autorizado** para raciocínio, verificação, consolidação e leitura de estado: é o modelo fixo da sessão e seu custo é pressuposto, não um despacho. O veto incide **exclusivamente no fan-out** — `delegate_task` com bundle bruto em cloud paga. Não transforme custo zero em obstáculo para raciocínio interativo; lotes vão para silício local ou CLI por assinatura, e o orquestrador pensa onde já está.

---

## 🛠️ FLUXO DE DESPACHO

```
-1. privacy_guard ──► 1. route_subtask ──► 2. handoff ──► 3. delegate_task ──► 4. sub_agent(complete)
   (Sensibilidade)       (Custo/VRAM)         (Contexto)     (Execução)           (Fechamento/Beta)
```

### Etapa -1 (anterior a tudo): classificar a privacidade do payload
Antes de `route_subtask`, rode `obsidian-privacy-shield/scripts/privacy_guard.py` sobre **frontmatter + corpo do payload real**, nunca apenas sobre metadados declarados em manifestos. Se `SENSITIVO` (`cloud_permitido = false`): encerre aqui e despache no SLM local (`/v1/chat/completions`). `route_subtask` otimiza custo/VRAM e **não conhece sensibilidade**; consultá-lo antes de classificar é erro de ordem. A etapa vale para qualquer payload, inclusive corpora isolados de teste (`wiki/`). Checklist em `references/cost-gate-preflight.md`.

### Etapa 1: Consulta Prévia ao `route_subtask`
Antes de disparar qualquer subagente:
```json
route_subtask({
  "task_type": "coding", // ou "agentic", "fast", "reasoning", "curation", etc.
  "estimated_tokens": 4000,
  "max_budget_usd": 0.00
})
```
* **O Roteador Avalia:**
  1. VRAM útil na RTX 2060m derivada em runtime (`/health.hardware.vram_usable_mb = vram_total_mb - config.wddm_reserve_mb`).
  2. Temperatura da GPU via NVML (com histerese térmica e desvio para CPU/Nuvem em sobrecarga).
  3. Thompson Sampling bayesiano com notas empíricas $E^3$ históricas.
* **Retorno:** Retorna `model_id`, `tier_name`, `active_context_window_tokens` (Janela de Contexto ativa) e `thinking_budget_tokens` (Orçamento de Raciocínio).

### Etapa 2: Registro de Handoff
Para não poluir a janela de contexto do chat mestre:
* Invoque `handoff(session_id="...", payload="instruções detalhadas para o worker")`.
* O payload é gravado no SQLite STRICT e o subagente inicia com uma memória de trabalho limpa.

### Etapa 3: Execução da Tarefa no Hermes
* Inicie o subagente delegando a tarefa com o modelo prescrito pelo `route_subtask`.

### Etapa 4: Fechamento de Sessão e Telemetria
Quando o subagente terminar:
* Chame `tool_dispatch(tool="sub_agent", arguments={"action": "complete", "session_id": "<agent_id_do_spawn>", "outcome": "SUCCESS", "tokens_used": 1250, "model_id": "<modelo_prescrito>", "task_type": "code"})`.
* O handler aceita `agent_id` ou alias `session_id`, fecha `sub_agent_sessions`, atualiza `weevolve_ratings`, registra `telemetry_events` e atualiza as distribuições Beta ($\alpha$ e $\beta$) no ParetoBandit (`pareto_bandit_priors`). Se a sessão não existir, retorna erro tipado `-32013 ResourceNotFound`.
* Se `route_subtask` retornar `-32012 TokenBudgetExceeded`, realize chunking / map-reduce abaixo do `max_active_context_tokens` informado antes de tentar novamente (não é erro térmico).

## Referências de profundidade
* Playbook `obsidian-privacy-shield`, seção *Roteamento por Tier* — classificar sensibilidade **antes** do pre-flight de custo; fronteiras que parecem locais e não são.
* `references/cost-gate-preflight.md` — checklist de custo + gates de entrega (medir conteúdo, não strings; etapa -1 de privacidade).
* `references/artifact-quality-audit.md` — auditoria em camadas, receita RED→GREEN de calibração e pitfalls do próprio verificador.
* `scripts/preflight_probe.py` — sonda determinística de pre-flight.
