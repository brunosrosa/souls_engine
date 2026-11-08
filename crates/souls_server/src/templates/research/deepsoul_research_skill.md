---
name: deepsoul-research
description: Playbook canônico de orquestração de Pesquisa Profunda (Deep Research) multiagente para o Hermes Agent integrado ao Souls Engine. Governa decomposição em árvore, despacho para perfis fast-scout, extração estruturada MinionS, reconstrução de espaço de trabalho IterResearch e auditoria com 4 etiquetas epistêmicas.
metadata:
  version: "1.0.0"
  author: "Souls Engine Architecture"
  namespace: "research"
---

# DeepSoul Research Playbook (Hermes Agent & Souls Engine)

Este playbook rege a execução de missões de **Pesquisa Profunda** delegadas pelo Souls Engine ao **Hermes Agent**.

---

## 🏛️ 1. OS 5 PRINCÍPIOS FUNDAMENTAIS

1. **Orquestrador Não Faz Raspagem Bruta:** O modelo líder (`LeadResearcher`) jamais consome páginas HTML integrais no seu histórico principal. Toda a coleta é delegada a subagentes via `delegate_task(profile="fast-scout")`.
2. **Filesystem Pointer Handoff:** Cada subagente explorador grava suas extrações densas diretamente no diretório do workspace da missão (`<workspace_path>/scouts/scout_{n}.md`) e retorna apenas um resumo compacto (`output_schema`).
3. **Reconstrução do Espaço de Trabalho (IterResearch):** A cada ronda de pesquisa, o histórico intermediário é descartado e o orquestrador atualiza o **Relatório Central em Evolução** (`01_evolving_report.md`).
4. **4 Etiquetas Epistêmicas Obrigatórias:** Todo fato ou número deve carregar uma tag:
   - `[VERIFIED]`: Confirmado por 2+ fontes primárias independentes.
   - `[SOURCED]`: Atribuído a uma fonte secundária confiável identificada por URL.
   - `[REASONED]`: Inferência lógica ou dedução matemática fundamentada.
   - `[ESTIMATED]`: Estimativa de mercado ou projeção aproximada.
5. **Auditoria Read-Only Sentinela:** Antes da entrega final, o subagente `verifier-readonly` (`toolsets=["file"]`) audita 100% das afirmações contra os relatórios brutos dos scouts, gerando `03_verification_receipt.json`.

---

## 🔄 2. CICLO OPERACIONAL DE EXECUÇÃO

```
1. Claim (MCP research_claim)
       │
       ▼
2. Scoping & Steering (MCP research_steering)
       │
       ▼
3. Rondas de Scouts (delegate_task profile="fast-scout")
       │
       ▼
4. Relatório Parcial (01_evolving_report.md + MCP research_event)
       │
       ▼
5. Síntese Final & Verificação (profile="verifier-readonly")
       │
       ▼
6. Conclusão (MCP research_complete)
```

### 2.1 Reivindicação da Missão
O agente invoca a ferramenta MCP:
```json
souls_research_claim({"agent_id": "hermes_master"})
```
A resposta contém o `workspace_path`, o `brief` com objetivos e exclusões, e o `budget` financeiro/turnos.

### 2.2 Rondas IterResearch & Escuta de Steering
No início de cada ronda, consulte o canal de comando do operador:
```json
souls_research_steering({"run_id": "<run_id>"})
```
Se `force_synthesis_requested` for verdadeiro ou se o orçamento atingir 72 turnos (*Beast Mode*), encerre novas buscas imediatamente e avance para a síntese.

### 2.3 Despacho de Subagentes Fast-Scout
Dispare subagentes paralelos com escopo atômico:
```json
delegate_task({
  "tasks": [
    {
      "goal": "Mapear benchmarks empíricos de IOPS e latência",
      "context": "Foque estritamente em Windows Dev Drive ReFS e IOCP. Grave em <workspace_path>/scouts/scout_1.md",
      "profile": "fast-scout"
    }
  ]
})
```

### 2.4 Notificação de Progresso
Após cada ronda, reporte a telemetria ao Souls Engine:
```json
souls_research_event({
  "run_id": "<run_id>",
  "phase": "scouts_round_1",
  "round": 1,
  "summary": "Coletadas 8 fontes primárias; 2 lacunas identificadas",
  "turns_used": 14,
  "usd_spent": 0.18
})
```

### 2.5 Verificação Factual e Encerramento
Gere o relatório em `02_draft_report.md`, invoque o perfil `verifier-readonly` para validar o recibo `03_verification_receipt.json` e encerre:
```json
souls_research_complete({
  "run_id": "<run_id>",
  "status": "confirmed",
  "report_path": "<workspace_path>/02_draft_report.md",
  "receipt_path": "<workspace_path>/03_verification_receipt.json"
})
```

