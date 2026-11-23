---
name: souls-evolutive-harness
description: Playbook operacional do Souls Evolutive Harness (v0.9.4 / ADR-034). Rege auto-evolução segura com pinagem constitucional SHA-256 de c1 (SOUL.md/AGENTS.md), fibras táticas reversíveis Cordis (LIFO <1ms + detecção de deriva ELO/EMA), Protocolo Cognitive Catnap (preempção <1ms, teto financeiro e Quiet Hours), Tombstones lógicos reversíveis para contradições de memória e cristalização Ralph Loop para Quarentena do Skills Hub.
version: "0.9.4"
author: "Souls Engine Team"
license: "MIT"
tags: ["evolutive-harness", "cordis", "catnap", "tombstones", "ralph-loop", "weevolve", "v0.9.4"]
---

# Souls Evolutive Harness — Auto-Evolução Segura & Contínua (`v0.9.4` / `ADR-034`)

## When to Use

Acione este playbook e suas ferramentas canônicas via `tool_dispatch` sempre que o usuário ou o próprio fluxo agêntico precisar:
- Auditar a integridade das regras centrais imutáveis (`c1`: `AGENTS.md`, `SOUL.md`) e o estado geral de evolução do sistema (`harness_status`).
- Reverter instantaneamente (`< 1 ms` LIFO) uma otimização tática recente de roteamento ou contexto que tenha causado deriva (`harness_cordis_rollback`).
- Consolidar melhorias leves ou investigar falhas passadas aproveitando tempo ocioso sem interromper o usuário (`harness_catnap_pulse`).
- Resolver ou desfazer contradições entre memórias antigas e decisões novas sem deletar registros permanentes (`harness_tombstone_resolve`).
- Transformar uma lição resolvida após erro difícil em uma habilidade modular revisável (`SKILL.md` na Quarentena) (`harness_ralph_crystallize`).

## Procedure

### 1. Inspeção de Estado & Proteção `c1`
1. Consulte o estado consolidado com `tool_dispatch(tool="harness_status", arguments={"include_index": false})`.
2. Verifique `summary.pins_verified == true`. Se qualquer arquivo `c1` apresentar divergência de SHA-256 LF (*fail-closed*), **aborte imediatamente** qualquer automação e alerte o operador humano.

### 2. Reversão Tática instantânea (Cordis LIFO)
- Quando uma adaptação em `c2` (guardrails táticos), `c5` (dicas de ferramentas), `c7` (perfis de subagentes) ou `c8` (hooks) piorar o desempenho ou gerar reclamação do usuário, execute:
  `tool_dispatch(tool="harness_cordis_rollback", arguments={"fiber_id": "<id_da_fibra>"})`
- A pilha inversa (`ctx.effect`) restaura os valores anteriores em `< 1 ms` e registra penalidade WeEvolve (`cordis_rollback`).

### 3. Higiene de Memória sem Apagamento (`TencentDB Tombstones`)
- Ao detectar mudança de preferência, troca de tecnologia ou instrução contraditória:
  - Para **inspeção somente leitura** (padrão seguro, zero mutações): `tool_dispatch(tool="harness_tombstone_resolve", arguments={"action": "preview"})` (ou omitindo `action` / `action="dry_run"`).
  - Para **varredura automática mutante** (atua **exclusivamente sobre memórias `EVOLVING`**, nunca altera `STABLE`, retornando `"mutated": true` e `reason_pt`): `tool_dispatch(tool="harness_tombstone_resolve", arguments={"action": "scan"})`.
  - Para marcar explicitamente um par específico: `tool_dispatch(tool="harness_tombstone_resolve", arguments={"action": "apply", "superseded_memory_id": "...", "superseded_by_memory_id": "...", "reason_pt": "..."})`.
  - Para restaurar em 1 clique uma memória antiga: `tool_dispatch(tool="harness_tombstone_resolve", arguments={"action": "restore", "tombstone_id": "..."})`.
- Registros na partição `STABLE` **nunca** são apagados fisicamente (`ADR-006`) nem superados por varreduras automáticas (`scan`).

### 4. Cristalização de Habilidades (`The Ralph Loop -> Quarentena`)
- Quando uma falha recorrente for diagnosticada e resolvida com sucesso, cristalize a lição em habilidade modular revisável:
  `tool_dispatch(tool="harness_ralph_crystallize", arguments={"title_pt": "...", "domain": "ops", "problem_pattern_pt": "...", "resolution_playbook": "..."})`
- A nova `SKILL.md` passa por scanner Zero-Trust, avaliação FinOps ($E^3$), normalização LF e cálculo Myers Diff, ficando **exclusivamente em Quarentena** para aprovação humana no Painel `Auto-Evolução & Aprendizado` ou `Skills & MCP Tools`.

### 5. Consolidação Cooperativa em Tempo Ocioso (`Catnap Pulse`)
- Dispare manualmente ou inspecione micro-fatias seguras com:
  `tool_dispatch(tool="harness_catnap_pulse", arguments={"slice_kind": "shadow_probe_ppc", "force_manual": true})`
- Os valores aceitos para `slice_kind` são: `shadow_probe_ppc`, `ladybug_reconcile`, `langevin_micro_batch`, `diary_distillation`, `ralph_crystallize`.

## Guardrails & Pitfalls

1. **Imutabilidade de `c1` (`AGENTS.md` + `SOUL.md` em `%LOCALAPPDATA%\hermes`, + auditoria de `MEMORY.md` / `USER.md`):** `AGENTS.md` e `SOUL.md` operam em modo `fail_closed` (1 byte alterado aborta automações Catnap/Ralph); `memories/MEMORY.md` e `memories/USER.md` são monitorados em modo `watch` (registram drift sem travar escritas legítimas do Hermes). Re-assinatura explícita via `POST /api/v1/harness/pins/resign`. Adaptações entram exclusivamente via injeção efêmera em `prefetch()` / `route_subtask` preservando `> 95%` de hit no prefix caching.
2. **Proibição de `timeout_retry = true` em ações não-idempotentes:** Nunca repita automaticamente chamadas mutantes (envio de e-mail, deleção ou mutação externa) após timeout.
3. **Preservação de Sigilo (`SCRUB_ENV`):** Variáveis sensíveis (`GITHUB_TOKEN`, `OPENAI_API_KEY`, etc.) são removidas antes de qualquer sonda ou script `c6`.
4. **Custo Zero por Padrão & Termodinâmica Realista:** `max_daily_budget_usd = 0.00` por padrão; sondas respeitam o teto térmico padrão `gpu_temp_c <= 62.0°C` com banda de histerese `+5.0°C` em voo (até `68.0°C` para fatias leves CPU/SQLite `0 MB VRAM`) e Horário de Silêncio configurável (`quiet_hours_enabled`).

## Verification

- Confirme `summary.pins_verified == true` em `harness_status`.
- Confirme criação limpa (apenas LF `\n`) em `quarantine/<slug>/SKILL.md` após `harness_ralph_crystallize`.
- Confirme restauração imediata nas buscas após `harness_tombstone_resolve(action="restore")`.
