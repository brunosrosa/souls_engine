# ADR-034: Souls Evolutive Harness — Microkernel Cordis, Catnap Protocol, Tombstones Lógicos e The Ralph Loop (`v0.9.4`)

## Status
**Aprovado e Implementado** (Milestone `v0.9.4`)

## Data
2026-11-23

## Contexto e Problema
Sistemas agênticos capazes de se auto-aprimorar enfrentam quatro armadilhas estruturais quando operados no mundo real sobre Windows 11 e APIs com *Prefix Caching*:
1. **Quebra de Prefix Caching & Deriva Constitucional:** Mutar `SOUL.md`, `AGENTS.md` ou `system_prompt_block` invalida o prefix cache do modelo mestre (reduzindo acertos de `>95%`) e introduz risco de deriva silenciosa ou alteração indesejada de identidade (`c1`).
2. **Mutações Irreversíveis e "Split-Brain" Hemisférico:** Melhorias de roteamento ou dicas táticas sem mecanismo de reversão acumulam lixo de prompt (*prompt bloat*) ou degradam a qualidade silenciosamente quando sinais empíricos caem.
3. **Consolidadores Noturnos Frágeis:** Jobs agendados para 4h da manhã falham quando o usuário desliga o notebook ou competem por CPU/VRAM durante sessões ativas, além de criar custos inesperados se chamarem APIs pagas por padrão.
4. **Apagamento Destrutivo vs Contradições em Memória:** Excluir registros `STABLE` viola o ADR-006 e remove trilha de auditoria, enquanto manter fatos contraditórios ativos polui o *Hybrid Recall (RRF)*. Além disso, separar barras no Windows (`\` vs `/`) quebra chaves em árvores estruturais de configuração.

---

## Decisão de Arquitetura

### 1. Superfície Anatômica Modular `c1`–`c8` & Blindagem Windows (`souls_protocol::harness_evolve` + `souls_core::harness::surface`)
- **8 Slots Canônicos (`HarnessComponentSlot`):**
  - `c1` (`C1SystemPrompt` — `AGENTS.md`, `SOUL.md`): **Imutável (*fail-closed*)** e verificado por hash SHA-256 normalizado em LF (`\n`). Qualquer tentativa automatizada de mutação ou divergência em disco aborta o ciclo de evolução com `CoreError::HarnessGovernanceViolation`.
  - `c2`, `c5`, `c7`, `c8`: Mutáveis de forma efêmera e 100% reversível via **Fibras Cordis**.
  - `c4` (`C4Skills`) e `c6` (`C6Scripts`): Alterados exclusivamente pelo fluxo **The Ralph Loop -> Skills Hub Quarentena** com Scanner Zero-Trust + **Myers Diff** para aprovação explícita do operador.
  - `c3` (`C3Memory`): Higiene via **Tombstones Lógicos JIT**.
- **Normalização Estrita & Segurança:**
  - `normalize_posix_rel_path()` força `/` e bloqueia `..` ou prefixos de unidade (`C:\`).
  - `scrub_candidate_environment()` (`SCRUB_ENV`) remove tokens/chaves de provedores antes de sondas e scripts candidatos `c6`.
  - `fold_aci_observation()` compacta observações verbosas preservando início, fim e erros finais; `is_non_idempotent_tool()` garante `timeout_retry = false` em ferramentas mutantes.

### 2. Microkernel Reversível DeepSeek Cordis (`souls_core::harness::cordis`)
- **Composabilidade Temporal (LIFO `dispose_fiber`):** Cada fibra tática (`c2`/`c5`/`c7`/`c8`) registra operações inversas `EffectUndoOp` desfeitas em ordem estritamente inversa em `< 1 ms`.
- **Composabilidade Espacial (Coefeitos Reativos):** Fibras declaram dependências (`lance_vectors`, `nvml_dgpu`, `google_oauth`, etc.) e entram em pausa automática se o subsistema ficar indisponível.
- **Guarda de Deriva Split-Brain:** Ao atingir `drift_streak >= 3` sinais negativos consecutivos ou `ema_score < 0.40`, `record_hemispheric_signal()` aciona `dispose_fiber()` automaticamente.
- **Preservação de Prefix Caching:** As diretrizes ativas nunca alteram o `system_prompt_snapshot`; entram efemeramente de forma **contextual à intenção da consulta** via `GET /api/v1/harness/cues?query=...` em `prefetch()` (`adapters/hermes_memory_souls/__init__.py`) e `evolutive_harness_cues` em `route_subtask` (consultas pessoais triviais recebem zero ruído). Fibras desfeitas podem ser reativadas em 1 clique via `POST /api/v1/harness/cordis/reactivate` e seu estado/ELO sobrevive a reinícios (`cordis_fibers.json` + hidratação `weevolve_ratings`).

### 3. Protocolo Cognitive Catnap (`souls_core::harness::catnap` + `souls_server::routes::harness`)
- **Gates Físicos, Histerese & FinOps:** Executa em loop cooperativo automático de segundo plano (`45s`, round-robin entre `shadow_probe_ppc`, `ladybug_reconcile`, `langevin_micro_batch`, `diary_distillation`, `ralph_crystallize`) somente quando `idle_elapsed_secs >= 180s`, `gpu_temp_c <= 62.0°C` (com banda de histerese **`+5.0°C`** em voo até `67.0°C` e permissão até `68.0°C` para fatias puramente CPU/SQLite `0 MB VRAM`), `cpu_load < 15%` e `active_connections == 0`.
- **Preempção Cooperativa `< 1 ms` Isolada de Polling Passivo:** `AppState::acquire_connection()` em rotas de trabalho ativo dispara `evolutive_harness.notify_activity()`, sinalizando `AtomicBool` (`SeqCst`) para aborto instantâneo; já rotas passivas de observabilidade (`/health`, `/api/v1/telemetry`, `GET /api/v1/harness/overview`, `GET /api/v1/harness/cues`) usam `acquire_passive_connection()` para nunca zerar a ociosidade enquanto o Dashboard estiver aberto.
- **Custo Zero por Padrão & Backoff Isolado:** `max_daily_budget_usd = 0.00` por padrão (`allow_free_cloud_improver = true`), *backoff* exponencial isolado por `slice_kind`, **Quiet Hours** configurável e desligável (`AGENTS.md` §9.5), e invalidação determinística por SHA-256 `runmark.fingerprint`.
- **Configuração Persistente Cirúrgica:** `[evolutive_harness]` em `souls_config.toml` com parser/gravador específico de seção preservando blocos irmãos e finais LF.

### 4. Tombstones Lógicos Reversíveis, Ranking WeEvolve & Experience Library (`souls_memory::evolve`)
- **Tombstones JIT sem Deleção Física (Tabela `#17 epistemic_memory_tombstones`):** Ao detectar substituição de preferência (`k: v`), diretriz de troca (`>= 2` tokens substantivos específicos fora da lista de stopwords genéricas ou citação direta do termo substituído) ou inversão de polaridade, marca o fato antigo como `status = 'superseded'`, adiciona aresta `conflicts_with` no `LadybugDB` (garantindo criação prévia dos nós `mem_node_<id>`) e filtra o ID em `0 ms` no FTS5, listagens e `RrfFusionEngine`. A restauração em 1 clique (`restore_tombstoned_memory`) remove a aresta `conflicts_with` correspondente e reabilita imediatamente a memória antiga sem violar o ADR-006.
- **Bradley-Terry ELO & EMA ($\alpha = 0.15$) (Tabela `#18 weevolve_ratings`):** Pontua fibras Cordis, skills e perfis de subagentes a partir de sinais reais de conclusão, falhas e reversões manuais.
- **Catnap Ledger & Domain Experience Library (Tabela `#19 catnap_consolidation_ledger`):** Grava episódios em `EXPERIENCE.md` + `metadata.json` deduplicados por `runmark_fingerprint` (com teto de retenção em `50` episódios) e mantém inventário $\mathcal{O}(M)$ `INDEX.md` com censo `always_on_chars`.

### 5. The Ralph Loop, MCP (`96 Tools / 18 Skills`) & Superfície Humana Zero-Jargão (`souls_server`)
- **Ralph Loop -> Quarentena & Aprovação em 1 Clique:** `crystallize_ralph_skill()` gera `SKILL.md` normalizada em LF, executa Scanner Zero-Trust + cálculo FinOps ($E^3$) + Myers Diff e deposita na Quarentena do Skills Hub para revisão visual ou aprovação direta via `POST /api/v1/harness/ralph/approve`.
- **12 Rotas REST (`/api/v1/harness/*`), 5 Ferramentas MCP & 18ª Skill Switchboard:** Totalizando **96** ferramentas canônicas (**8** Boot Level 0 + **88** progressivas) e **18** skills (`souls-evolutive-harness`).
- **UI Centrada no Humano (`AGENTS.md` §9):** Sub-aba **Auto-Evolução & Aprendizado** sob o pilar **Agentes & Personas** (`evolutive_harness.html` + `evolutive_harness.js`), 100% livre de estilos inline (`style="..."`) e emojis crus, com botões de 1 clique em português claro e telemetria técnica encapsulada em `<details class="lab-drawer" id="drawer-lab-evolutive-harness">`.

---

## Consequências

### Positivas
- **Aprendizado Sem Desperdício e Zero Regressões Silenciosas:** Toda melhoria tática tem pilha de reversão `< 1 ms` e cai automaticamente diante de deriva empírica, sem contaminar a identidade central (`c1`) nem o prefix cache.
- **Memória Sempre Coerente Sem Perda de Auditoria:** Contradições param de poluir buscas imediatamente com opção de restauração humana em 1 clique.
- **Controle Total de Custos & Hardware:** Consolidação cooperativa em tempo ocioso respeita temperatura da GPU, orçamento `$0.00` padrão e interação imediata do usuário.

### Riscos e Mitigações
- **Poluição de contexto por excesso de fibras táticas:** Mitigado pelo censo `always_on_chars` no `INDEX.md`, filtro por `target_task_type` no `route_subtask` e reversão LIFO/WeEvolve automática em `drift_streak >= 3`.
- **Contenção em disco no Dev Drive ReFS:** Mitigada por rotação limpa no ledger SQLite STRICT WAL e preservação rigorosa de finais de linha LF em todas as artes cristalizadas.
