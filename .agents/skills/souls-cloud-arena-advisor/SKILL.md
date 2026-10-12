---
name: souls-cloud-arena-advisor
description: Playbook para consulta da matriz analítica de mercado da Cloud Arena. Governa inspeção de modelos líderes da Fronteira de Pareto nos 9 perfis cognitivos e sincronização de tarifas.
metadata:
  version: "1.0.0"
  author: "Souls Engine Architecture"
  namespace: "ops"
---

# Souls Cloud Arena Advisor — Playbook de Inteligência de Nuvem (v7)

Este playbook rege a inspeção de modelos de nuvem (Nous Research Portal e OpenRouter) auditados diariamente pelo motor analítico da `souls_llm_cloud_arena` (ADR-009).

## 🏆 OS 9 PERFIS COGNITIVOS DA CLOUD ARENA
A Cloud Arena classifica os modelos em 9 perfis especializados com base em benchmarks e custos ponderados:
1. `executive`: O **Master Brain** (a mente mestre do chat interativo do Hermes).
2. `coding`: Engenharia de software e geração de código Rust/Python com AST.
3. `agentic`: Loops autônomos de MCP JSON tool-calling.
4. `reasoning`: Raciocínio matemático, deduções formais e GPQA Diamond.
5. `fast`: Triagem ultrarrápida e streaming instantâneo.
6. `design`: UI web moderna, tokens visuais e multimodal VQA.
7. `research`: Ingestão massiva de documentos em 128k-1M tokens sem degradação.
8. `security`: Análise de vulnerabilidades, detecção de race conditions e auditoria.
9. `curation`: Curadoria de wikis, Obsidian DB e escrita técnica bilingue.

---

## 🛠️ FLUXO OPERACIONAL

### 1. Consulta dos Campeões de Pareto com `cloud_champions`
Para inspecionar quais modelos dominam a Fronteira de Pareto:
```json
cloud_champions({
  "profile": "executive" // ou "coding", "agentic", "curation", "all"
})
```
* **Retorno:** Lista estruturada de modelos contendo:
  * `canonical_id` e `display_name`
  * `composite_score` ($S_{\text{task}}$ de 0 a 100)
  * `task_blended_cost` ($C_{\text{task}}$ em USD/1M tokens)
  * `roi_index` ($V_{\text{task}}$ amortecido logarítmico)
  * `pareto_tag`: `Leader` (maior qualidade absoluta), `BestValue` (maior ROI) ou `UltraLowCost`.

### 2. Sincronização Sob Demanda com `cloud_arena_sync`
O Souls Engine executa sincronização diária em segundo plano no boot do servidor. Caso deseje forçar uma atualização imediata de tarifas ou novos lançamentos:
```json
cloud_arena_sync({
  "force": true
})
```

### 3. Recomendação de Presets para o Hermes
Quando o operador perguntar sobre configurações ou modelos ideais:
* Consulte `cloud_champions(profile="executive")` para orientar a configuração do chat mestre no `config.yaml`.
* Consulte os perfis operacionais (`coding`, `agentic`, `fast`) para calibrar as opções de despacho de subagentes.
