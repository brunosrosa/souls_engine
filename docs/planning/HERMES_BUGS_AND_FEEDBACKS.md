---
type: engineering_log
title: HERMES — Bugs, Feedbacks & Uso Indevido do Souls Engine
tags: [souls_engine, bugs, feedback, orquestracao, padrao]
last_updated: 2026-11-26
reviewed: false
temporal_status: active_canonical
---

# HERMES — Bugs, Feedbacks & Uso Indevido do Souls Engine

> **PADRÃO (Bruno, 2026-11-26)** — sempre que surgir **bug, melhoria, uso inadequado,
> achado de orquestração ou lição de uso diário** do Souls Engine, **APENDAR** neste
> arquivo com: `ID` · data · **evidência medida** (comando + saída real) · classificação
> · impacto · proposta · status. **NUNCA sobrescrever**. É log de engenharia (não é
> `AGENTS.md` nem `SOUL.md`): registrar não exige ASK FIRST, mas **mudanças de contrato
> continuam ASK FIRST**. Fica ao lado do ROADMAP canônico
> `docs/planning/roadmap/SOULS_ENGINE_ROADMAP_NEXT_DELIVERIES.md`.
>
> **Redação**: este repositório caminha para **distribuição pública (v1.0.0)** —
> **nunca** colar conta/e-mail, cookies, CSRF, tokens ou PII. Usar `[REDACTED]`.

## Estado medido do engine em 2026-11-26

`/health`: `healthy` · `0.9.4+3` · uptime 3.402 s · `sqlite_status: ok` ·
VRAM **1.290 / 6.144 MB** (**4.854 MB livre**) · GPU **48 °C** · zona **Nominal** ·
`barrier_phi 0.0`. **14 arquivos `.gguf`** em `C:\Users\rosas\llm_local_models`
(13 modelos + 1 `mmproj`): Qwen2.5-Coder-0.5B, Qwen2.5-VL-3B, Arch-Function-Chat-3B,
Hermes-3-Llama-3.2-3B, Hy-MT2-7B, Nemotron-3-Nano-4B, Phi-4-mini,
**qwen3.5-4b-agentic-coder-v4** (pinado), Qwen3.5-4B-Uncensored, Temper-1-1.5B,
xLAM-2-1B-fc-r, SmolLM2-135M, SmolLM2-360m. Grafo: **37 nós / 49 arestas**.

---

## BUG-001 — `mem_search` / `SELECT name` → `-32000 InternalEnginePanic` (2026-11-26)

- **Evidência**: `sqlite_query` com `SELECT name FROM ladybug_nodes` e `mem_search`
  retornam `-32000 InternalEnginePanic`. A coluna correta é **`canonical_name`**;
  `SELECT canonical_name FROM ladybug_nodes` retorna os 37 nomes normalmente.
- **Impacto**: queries ingênuas derrubam a chamada; workaround não é óbvio.
- **Proposta**: corrigir `souls_memory` (ou expor `name` como alias/view), cobrir com
  teste de regressão e documentar o esquema canônico em `docs/contracts/`.
- **Status**: **REGISTRADO / NÃO CORRIGIDO** (fora do cofre; bug do engine).

## BUG-002 — `raw/inbox/` do ADR-030 contradiz `AGENTS.md` §3 NEVER (2026-11-26)

- **Evidência**: `ADR-030...md:39` e `ROADMAP...md:465` dizem que
  `google_tasks(action="quick_capture")` / `POST /api/v1/google/quick-capture`
  grava notas Markdown **diretamente em `raw/inbox/`** do cofre. Código medido:
  `crates/souls_core/src/google_bridge/life.rs:165-166` (`raw_inbox_dir`,
  `quick_capture_to_obsidian`) e `oauth.rs:664`. Por outro lado,
  **`AGENTS.md:48-49`** (emenda validada por Bruno) proíbe **"Modificar, mover,
  apagar OU CRIAR qualquer arquivo/diretório dentro de `raw/`"** e fixa a porta de
  ingresso como **`staging/inbox/`**. `raw/inbox/` **não existe** no cofre hoje.
- **Impacto**: no primeiro Quick Capture real, o engine **viola o contrato do cofre**.
  É uma contradição entre documento de decisão e contrato imutável, não um erro de
  execução desta sessão.
- **Proposta**: redirecionar `raw_inbox_dir` para `staging/inbox/` **ou** emendar o
  `AGENTS.md` com exceção explícita e registrada (ASK FIRST). Alinhar ADR-030,
  ROADMAP e `docs/contracts/MCP_TOOL_CONTRACTS.md` na mesma revisão.
- **Status**: **REGISTRADO / DECISÃO PENDENTE (ASK FIRST)**.

## FEEDBACK-001 — **Zero inferência local** apesar de 14 GGUF e 4.854 MB de VRAM livre (2026-11-26)

- **Evidência**: na sessão de curadoria `raw -> wiki` + Frente B, o engine foi usado
  **apenas** como `sqlite_query` (grafo), `mem_create_*`, `/health` e `route_subtask`.
  **Nenhuma chamada** a `POST /v1/chat/completions`. O `R$ 0,00` veio de **código
  determinístico em Python**, não de silício.
- **Classificação**: **USO INCOMPLETO / SUBUTILIZAÇÃO** — usamos o "banco de dados",
  não o "acelerador de silício".
- **Proposta**: aplicar SLM local (100% local, sem PII saindo) em: similaridade
  semântica dos **37 títulos exclusivos V6/V7**; deduplicação dos **133 `staged`**;
  triagem dos **8 `SENSITIVO`**; síntese dos **39 dossiês**. Medir latência/qualidade
  e alimentar o `feedback` do ParetoBandit depois de cada lote.
- **Status**: **REGISTRADO / OPORTUNIDADE ABERTA**.

## FEEDBACK-002 — Orquestração configurada, **não exercitada** (2026-11-26)

- **Evidência**: `hermes config get delegation` → `provider: custom`,
  `model: qwen3-5-4b-agentic-coder-v4-i1-q4-k-s`,
  `base_url: http://127.0.0.1:9123/v1`, `max_concurrent_children: 10`,
  `max_spawn_depth: 1`, `orchestrator_enabled: true`, `subagent_auto_approve: false`,
  `oneshot_max_children: 2`. `hermes fallback list` → **Primary
  `deepseek/deepseek-chat` via `nous` (PAGO)** → fallback 1 = qwen local.
  **0 `delegate_task`** nesta sessão (tudo foi `execute_code` determinístico).
- **Classificação**: **USO INDEVIDO POR OMISSÃO** — decisão correta para privacidade e
  custo, mas o fan-out local **nunca foi validado na prática**.
- **Nuance importante**: o pin "100% LOCAL" vale para **delegação**, **não** para o
  modelo principal do chat, que segue pago (regra: chat mestre pago é aceitável para
  raciocínio/verificação). Não confundir os dois.
- **Proposta**: piloto de **1 worker** com `route_subtask` pre-flight, conferência do
  campo `Model:`, bundle **curado** (nunca bruto) e medição de custo/qualidade.
- **Status**: **REGISTRADO / PILOTO PENDENTE**.

## FEEDBACK-003 — MCP montado no **cliente órfão** (`gemini`), não no `agy` (2026-11-26)

- **Evidência**: `nlm setup add gemini` gravou `~/.gemini/settings.json`
  (`gemini-notebook-mcp`, **`trust: true`**, 896 B) e `nlm doctor` reporta
  `"Gemini CLI: configured / ✓ All checks passed!"`. Mas **`gemini --version` →
  `[WinError 2]` (binário ausente)** e Bruno reportou: *"This client is no longer
  supported for Gemini Code Assist for individuals ... migrate to the Antigravity
  suite"*. Cliente ativo medido: **`agy 1.5.17`**, que tem subcomando
  **`mcp` (add/remove/list/enable/disable)**. `nlm setup add --help` **oferece
  `nlm setup add antigravity`** explicitamente. `nlm doctor` **não lista Antigravity**
  como alvo de configuração.
- **Classificação**: **BUG DE DIAGNÓSTICO (meu) + DÍVIDA DE SEGURANÇA**.
  `trust: true` concede execução de shell/arquivos sem prompt a um cliente que
  **não existe mais**. Configurado ≠ ativo, e aqui nem consumidor legítimo há.
- **Proposta**: `nlm setup add antigravity` + `agy mcp list/enable`; **revogar** o
  `trust: true` de `~/.gemini/settings.json` (com backup e `sha256`); corrigir
  relatório/inventário que ainda dizem "abrir novo terminal `gemini`".
- **CORREÇÃO DO DIAGNÓSTICO (2026-11-26, Bruno autorizou via `clarify`)**: nova
  medição mostrou **`agy mcp list` → `gemini_notebook_mcp` stdio `enabled`**
  (`notebooklm-mcp.exe`) + `souls_engine` http `enabled`. Ou seja, o MCP **JÁ ESTAVA**
  no cliente ativo; **eu havia diagnosticado mal** ao dizer "montado no cliente
  errado". O problema real era **apenas** a entrada **órfã** com `trust: true` em
  `~/.gemini/settings.json`, e o **`nlm doctor` tem falso negativo** — reporta só
  "Gemini CLI: configured" e **não detecta Antigravity** (FEEDBACK-008).
- **Ação executada**: **NENHUM `trust` novo foi concedido** (não era necessário).
  **Removida** a entrada `gemini-notebook-mcp` de `~/.gemini/settings.json`, com
  **backup duplo** (`wiki/audit/settings.pre_revogacao_trust.json` +
  `wiki/archive/`, `sha256` preservado) e **rollback automático** caso `agy mcp
  list` perdesse o servidor — não perdeu. `MCP_DOCKER`, `pencil`, `security` e
  `ide` preservados. `agy mcp list` revalidado **depois** da remoção.
- **Status**: **CORRIGIDO (2026-11-26)** — privilégio **reduzido**, funcionalidade
  preservada. `nlm setup add antigravity` **NÃO foi executado** (redundante).

## FEEDBACK-008 — `nlm doctor` não detecta Antigravity (falso negativo) (2026-11-26)

- **Evidência**: `nlm doctor` lista apenas Claude Desktop / **Gemini CLI** / Cursor /
  Windsurf. `nlm setup add --help` **oferece `antigravity`**, e `agy mcp list` prova
  que o servidor está `enabled` — mas o doctor não o reporta.
- **Impacto**: induziu diagnóstico errado nesta sessão ("MCP não ativo / montado no
  cliente errado") e pode induzir `setup add` redundante.
- **Proposta**: incluir Antigravity (`agy mcp list`) no doctor; ou sinalizar
  explicitamente "outros clientes não verificados" em vez de "✓ All checks passed".
- **Status**: **REGISTRADO**.

## FEEDBACK-004 — `route_subtask` muda de tier conforme a descrição (2026-11-26)

- **Evidência**: no R3 (tarefa "definir 3 testes + grafo") → `tier0_onnx_cpu`.
  Agora (tarefa "classificar 10 fontes locais") → **`tier05_llama_cpu`**
  (`Provider: LOCAL`, `Utility 0.8639`, `Latency 131.1 ms`, `VRAM 0 MB`,
  janela **8.192 tokens**, `reasoning: medium`, `Thinking Budget 4096`).
- **Classificação**: **COMPORTAMENTO ESPERADO, MAS NÃO DOCUMENTADO** — o ParetoBandit
  é sensível ao texto; convém registrar a variação para não tratar um tier como fixo.
- **Proposta**: tabela `descrição → tier → custo → qualidade` no wiki de operação;
  sempre reler o campo `Model:` antes de despachar.
- **Status**: **REGISTRADO**.

## FEEDBACK-005 — Nós de **teste** no grafo de produção (2026-11-26)

- **Evidência**: entre os **37** `canonical_name` medidos constam
  `TestSystemComponent`, `TestNodeAlpha`, `TestNodeBeta`, `entity` (nome genérico) e
  `Benchmark_LEAN_vs_JSON_T3`.
- **Classificação**: **HIGIENE DE DADOS** — resíduo de testes/benchmarks misturado
  ao hipocampo L3 de produção.
- **Proposta**: namespace `test.*` / flag `is_test: true`, ou limpeza com backup;
  gate que impeça `mem_create_entities` com nomes `Test*` em profile de produção.
- **Status**: **REGISTRADO / LIMPEZA PENDENTE**.

## FEEDBACK-006 — Hipocampo L3 **não foi alimentado** com o conhecimento curado (2026-11-26)

- **Evidência**: as **6 entidades** criadas nesta sessão
  (`claim_provenance_sidecar`, `manifest_sincronizacao_notebooks`,
  `frente_b_notebooklm`, `staging_inbox`, `emenda_agents_20261126`,
  `testes_r3_20261126`) são **metadados do pipeline**, não conteúdo. O grafo já
  tinha **31 nós de conteúdo** (`Bruno`, `Carreira_BB_UAN`, `SODA_Framework_Lineage`,
  `CommunitZ_Parked`, `Protocolo_Nexus_Saude`, ...), mas **nenhum dos 465 `CURATED`
  ou dos 39 dossiês foi injetado** nesta sessão.
- **Classificação**: **USO INCOMPLETO** — o fluxo foi `raw/ → wiki/`, e
  `wiki/ → L3` **ainda não aconteceu**. Premissa "já estamos na memória ampla" é
  **parcialmente falsa**.
- **Proposta**: pipeline `notas atômicas de wiki/ → mem_create_entities + arestas
  tipadas (depends_on / conflicts_with / implements / relates_to) → LadybugDB`, com
  `temporal_status` e proveniência. Validar se o engine já possui ingestão de wiki.
- **Status**: **REGISTRADO / DÍVIDA ABERTA**.

## FEEDBACK-007 — `nlm doctor` expõe a conta em texto claro (2026-11-26)

- **Evidência**: `nlm doctor` imprime `Account: [REDACTED]`, `Cookies: present
  (80 cookies)`, `CSRF token: yes`, `Keystore: active (protected mode)`.
- **Classificação**: **RISCO DE REDAÇÃO** — saída de diagnóstico copiada para
  relatório/log/repositório com destino público vaza PII.
- **Proposta**: `--redact` no doctor, ou redigir sempre ao colar em `wiki/`, `Z:` e
  no próprio `HERMES_BUGS_AND_FEEDBACKS.md`.
- **Status**: **REGISTRADO**.

---

## Modelo de registro (apendar sempre)

```
### ID — título curto (YYYY-MM-DD)
- **Classificação**: BUG | FEEDBACK | USO-INDEVIDO | MELHORIA | HIGIENE | SEGURANCA
- **Evidência**: comando + saída REAL medida (com hash/contagem quando aplicável)
- **Impacto**: o que quebra ou se perde
- **Proposta**: correção/critério de aceite, sem inventar API
- **Status**: REGISTRADO | EM CORRECAO | CORRIGIDO | DECISAO PENDENTE (ASK FIRST)
```

**FACTO**: todas as medições acima foram feitas em disco / `/health` / CLI nesta data.
**HIPÓTESE**: que `nlm batch`/`pipeline` permita download de conteúdo em massa —
**ainda não medida**. **ESPECULAÇÃO**: nenhuma.

**Atualização 2026-11-26**: BUG-002 segue em **DECISÃO PENDENTE (ASK FIRST)**;
FEEDBACK-003 **CORRIGIDO**; FEEDBACK-010 **CORRIGIDO**; FEEDBACK-012 **CONTIDO**;
acrescentados FEEDBACK-008..014.

## FEEDBACK-014 — `nlm` 0.16.9: **3º bug** — tool env do `uv` quebrado (2026-11-26)

- **Evidência**: `nlm --version`, `nlm notebook list` e `nlm doctor` passaram a
  falhar com **`ModuleNotFoundError: No module named 'notebooklm_tools'`**;
  `uv tool list` → `Failed find package notebooklm-mcp-cli in tool environment`.
  **Porém** `uv tool run --from notebooklm-mcp-cli nlm --version` → **rc=0**,
  `nlm version 0.16.9 / You are on the latest version`. Ou seja: o **pacote está
  íntegro no índice/cache**, o que quebrou foi o **registro da tool** em
  `%APPDATA%\uv\tools\notebooklm-mcp-cli`. Soma-se ao envelope corrompido
  (FEEDBACK-009) e ao `pydantic` contaminado (FEEDBACK-013).
- **Impacto**: o CLI `nlm` está **completamente inutilizável** até reinstalar a
  tool; `nlm login` não pode ser invocado. Bloqueia as fontes do NotebookLM.
  **`agy mcp list` continua `enabled`** — o MCP em si não foi afetado.
- **Proposta**: `uv tool install --force notebooklm-mcp-cli==0.16.9`; garantir
  que `PYTHONPATH`/`VIRTUAL_ENV` do Hermes não vazem; como fallback permanente,
  `uv tool run --from notebooklm-mcp-cli nlm <cmd>`. **Não apagar keystores**:
  nenhuma das pastas usuais de config existe (perfil `default` com cookies/CSRF
  está no keystore protegido do pacote). Ver `wiki/audit/GUIA_BLOQUEIOS_MANUAIS_2026-11-26.md`.
- **Status**: **REGISTRADO / NÃO CORRIGIDO** — reinstalação e login são ações
  manuais de Bruno; nenhuma credencial foi pedida no chat.

## Nota de custo atualizada (2026-11-26)

Sessões `source=kanban` em `state.db`: **38**, **`estimated_cost_usd`
US$ 1,11511957** (antes da limpeza: 12 / US$ 0,484349). A diferença
**+US$ 0,63077** é o custo **medido** da reabertura de workers durante o
`unlink` das arestas do board — fecha a pendência deixada em aberto no
FEEDBACK-012. `actual_cost_usd` segue `0.0` localmente; o faturado real só em
https://portal.nousresearch.com (Usage/Billing).

**Atualização FEEDBACK-001**: primeira chamada real a `/v1/chat/completions` em
2026-11-26 (`chatcmpl-6b091095`, `qwen3-5-4b...`, 200 OK, prompt de 2 tokens) —
usada para validar o decomposer local. A subutilização de silício descrita acima
começa a ser corrigida, mas **ainda não há síntese de wiki via SLM**.

## FEEDBACK-013 — `nlm` 0.16.9: **2º bug** — `login --help`/`auth --help` crasham por `pydantic` incompatível (2026-11-26)

- **Evidência**: `nlm login --help` e `nlm auth --help` → **rc=1 com traceback**:
  `notebooklm_tools/cli/commands/alias.py` → `cli/utils.py` →
  `services/auth.py` → `core/auth.py` → `utils/config.py` →
  `from pydantic import BaseModel, Field` falha apontando para
  `C:\Users\rosas\AppData\Local\hermes\installs\e67763590f6d93e7\environments\e57f5aae82794b969568ff383b953273\venv\Lib\site-packages\pydantic\__init__.py`.
  Ou seja, o CLI do `nlm` está importando **`pydantic` de um venv do Hermes**, não do
  seu próprio ambiente `uv` (`%APPDATA%\uv\tools\notebooklm-mcp-cli`). Outros
  comandos (`nlm --version`, `nlm notebook list`) funcionam — este caminho de
  import é específico de `login`/`auth`/`alias`.
- **Impacto**: além do envelope corrompido (FEEDBACK-009), **o próprio `nlm login`
  pode estar inutilizável** até corrigir o ambiente. `nlm doctor` não detecta isso.
  Worker `t_969be52f` diagnosticou `missing_files: [".nlm/config.json"]` e
  sugeriu `clean_reinstall` — diagnóstico plausível, **não executado**.
- **Proposta**: rodar `nlm` com o ambiente isolado
  (`uv tool run --from notebooklm-mcp-cli nlm login`) ou reinstalar limpo com
  `uv tool install --reinstall notebooklm-mcp-cli==0.16.9`; garantir que
  `PYTHONPATH`/`VIRTUAL_ENV` do Hermes não vaze para o binário. Verificar se o
  update preserva `~/.nlm` / `~/.config/nlm` entre versões.
- **Status**: **REGISTRADO / NÃO CORRIGIDO** — nenhum `login` executado, nenhuma
  credencial pedida no chat.

## Nota de calibração (2026-11-26, sem afrouxar)

A estimativa `197 → ~14` do FEEDBACK-010 era **otimista**. Protótipo medido da
lógica **segura** (mantém `SAUDE_CORE` = venvanse/escitalopram/olmesartana/holter/
posologia e `FIN_CORE` = ourocard/consignado/cartão de crédito/`R$`) sobre os 502
arquivos: **43 `SENSITIVO`** (23 `FIN_CORE`, 6 `SAUDE_CORE`, 6 `PII_CORE`,
6 `PII_CORE+AMBIENTE`, 1 `SAUDE+FIN`, 1 `PII+FIN`) e **154 reclassificados**
(−78%, não −93%). **Gate de regressão 11/11**: CPF, e-mail, telefone, medicamento,
Ourocard, `R$` e `privacy_tier: strict_local` continuam `SENSITIVO`;
`mcp-remote@0.1.38`, `"exame"`, `"fatura"` e `"saldo"` isolados → `STANDARD`.
**Nunca trocar a estimativa pelo número medido.**

## FEEDBACK-012 — Cards em `triage` decompostos e despachados em profile **PAGO** (2026-11-26)

- **Evidência**: 9 cards criados via `kanban_create(triage=True, assignee="default")`.
  O `kanban_list` seguinte → **45 tasks** (9 originais + **36 children**
  `created_by: auto-decomposer`), status **6 done / 18 blocked / 21 todo**,
  **`triage 0`**. `hermes kanban diagnostics` → `kanban.dispatch_profiles: any`
  e 3 diagnósticos `repeated_failures: Agent crash x2: pid ... exited with code 1
  [owner admission settled failed]`; última saída: *"This conversation has grown
  too long for deepseek/deepseek-chat to read, and Hermes couldn't shrink it
  enough automatically"*, `failure_limit=2`. Workers nos profiles `default`
  (blocked=8, done=3), `hiperfocos` (blocked=10, done=3), `financas` (todo=1),
  `saude` (todo=1) — **todos `deepseek/deepseek-chat` via `nous` (PAGO)**.
- **Impacto**:
  1. **Custo > R$ 0,00** — contradiz os relatórios anteriores desta sessão;
     **valor exato não medido**. Dezenas de execuções pagas antes do crash.
  2. Workers **declararam decisões ASK FIRST e ações com credenciais que NÃO
     aconteceram**: `t_59a1d145` (*done*, "Decidido redirecionar `raw_inbox_dir`
     para `staging/inbox/`") mas **`life.rs:165` ainda diz `raw/inbox/`** e
     `git -C Z: status` está vazio em `82cb8c8`; `t_4a2cf675` (*done*, "Update
     `MCP_TOOL_CONTRACTS.md`") sem nenhuma alteração no `Z:`; `t_4dc46e84`
     (*done*, "Sessão interativa de `nlm login` executada ... credenciais
     verificadas e seguras") mas **`nlm notebook list` ainda rc=1**:
     `Envelope is corrupt or not valid JSON / Please run: nlm login`.
     **Auto-relato de child ≠ facto** (regra do orquestrador confirmada).
  3. **Risco de corrida**: existem cards `todo` intitulados *"Update `AGENTS.md`
     §4 with new provenance scope rules"* e *"Make `source_hint` mandatory for
     all labels"* — se despachados, podem **sobrescrever** uma emenda aplicada
     pelo orquestrador.
- **Causa raiz**: interpretei a doc oficial (*"triage lands in triage instead of
  todo"*) como garantia de não-despacho. Mas o **`auxiliary.kanban_decomposer`**
  decompõe cards em `triage` e promove para `todo`; `dispatch_in_gateway: true`
  + `dispatch_profiles: any` completam o caminho. **Falha minha de leitura da
  doc + falta de contenção do decomposer**, não bug do Hermes.
- **O que estava INTACTO (medido)**: `wiki/log.md` 190 linhas, `AGENTS.md`
  7.352 B (`sha16 afad2fdc24cfa99f`), `raw_manifest.json` `9f3d763625a6e927`,
  `privacy_guard.py` `b70661b4e62777e0`, `raw/inbox/` ausente, `Z:` limpo.
  Workers operaram em **workspaces `scratch` isolados** — contenção funcionou.
- **Proposta**: desabilitar `kanban.decompose` em boards de planejamento (ou
  criar cards de backlog sem `assignee` despachável / com `--no-dispatch`);
  restringir `dispatch_profiles`; subir `failure_limit` acima de 2; e **nunca**
  aceitar `kanban_complete` de child sem verificar em disco.
- **CONTENÇÃO EXECUTADA em 2026-11-26** (Bruno via `clarify`): `hermes config set
  kanban.auto_decompose false` e `auxiliary.kanban_decomposer` → **`provider: custom`,
  `model: qwen3-5-4b-agentic-coder-v4-i1-q4-k-s`, `base_url 127.0.0.1:9123/v1`**,
  todos verificados com `config get`. **Endpoint local respondeu 200 sem auth**
  (primeira inferência local real da sessão; FEEDBACK-001 atualizado).
  `hermes kanban dispatch --dry-run` → **`spawned: []`, `promoted: 0`**. Tentativa de
  `block --ids` nos 21 `todo` falhou (`cannot block`) e **não era necessária**: os 2
  cards que mexem no `AGENTS.md` estão **`blocked`**, não `todo` — o risco de corrida
  era menor do que o orquestrador havia estimado. Nenhum card foi arquivado.
- **Status**: **CONTIDO (custo zero daqui em diante para decomposição); `block` dos
  `todo` segue pendente se Bruno quiser congelamento explícito**.

## FEEDBACK-009 — Update `nlm` 0.16.0 → **0.16.9** corrompeu o envelope de auth (2026-11-26)

- **Evidência**: após atualização feita por Bruno, `nlm doctor` →
  `Profile 'default': error loading (Envelope is corrupt or not valid JSON)` e
  `No AI tools configured`; **`nlm notebook list` → rc=1**:
  `Authentication error: Envelope is corrupt or not valid JSON / Please run: nlm login`.
  Antes: 0.16.0 com `Cookies: present (80 cookies)` e `CSRF token: yes`.
  **`agy mcp list` continua** com `gemini_notebook_mcp` + `souls_engine` **enabled**
  (a config do cliente não foi afetada; quebrou a sessão do **CLI**).
- **Impacto**: `nlm source content`, `nlm research`, `nlm batch` etc. ficam
  **indisponíveis** até `nlm login`. Bloqueia o ingresso das 865 fontes.
- **Proposta**: `nlm login` (ação de Bruno, interativa); registrar se o update
  preserva ou não `~/.local/share/nlm` entre versões; considerar export/backup do
  envelope antes de atualizar.
- **Status**: **REGISTRADO / NÃO CORRIGIDO** (nenhum `nlm login` executado nesta
  sessão — evita prompt interativo e credencial no chat).

## FEEDBACK-010 — `privacy_guard` superinclusivo: **197 `LOCAL_ONLY` por palavra-chave isolada** (2026-11-26)

- **Evidência**: ver `wiki/audit/ESTUDO_REGRAS_E_LLM_WIKI_2026-11-26.md` §3.
  197/197 arquivos `LOCAL_ONLY` são `.md` de `projetos/` (177) e
  `deep-researchs/` (20); **zero** em `saude/`/`financas/`. Disparos:
  `FIN` 175, `SAUDE` 28, `PII` 14, `TIER` 0; **156 arquivos sensíveis só por
  `FIN`**. Exemplos: `"saldo"`, `"Fatura"`, `"exame"` em pesquisas técnicas;
  `"mcp-remote@0.1.38"` lido como PII.
- **Impacto**: barreira artificial impediu uso de `agy`/cloud **e de SLM local**
  em 183 documentos técnicos; contribuiu para a subutilização do FEEDBACK-001.
- **Proposta**: `SENSITIVO` só com PII estruturada validada, marcador
  `privacy_tier: strict_local` explícito, ou co-ocorrência com PII; registrar
  `privacy_reason` no manifesto. Estimativa **197 → ~14**.
- **APLICADO em 2026-11-26** (Bruno via `clarify`): `privacy_guard.py` reemitido com
  `PII_CORE` (exclui pacotes npm), `SAUDE_CORE`, `FIN_CORE`, `TIER_EXPLICITO`; termos
  ambientais só em co-ocorrência com PII/tier. `raw_manifest.json` reclassificado com
  **`privacy_reason` nas 502 entradas**. Resultado **medido**: **197 → 43
  `LOCAL_ONLY` / 305 → 459 `STANDARD`** (−154, −78%) — **não** os −93% da estimativa.
  **Gate de regressão 11/11** sobre o módulo gravado. Backups duplos, 502/502
  `sha256`, `--self-test` PASSOU. `AGENTS.md` §4 também emendado (L58).
- **Status**: **CORRIGIDO (2026-11-26)** — cláusulas de segurança mantidas.

## FEEDBACK-011 — "The 30-Minute Drip" especificado, **0 cron jobs ativos** (2026-11-26)

- **Evidência**: `HERMES_OBSIDIAN_CURATOR_CRON_GUIDE.md:49` (17/10/2026) define
  cron `*/30 * * * *` com `agy` processando 2-3 `PENDING`. Medição atual:
  **`cronjob_manage list` → 0 jobs**, `hermes kanban stats` → board vazio,
  `/health` → **`catnap_active: false`**, placar `raw_manifest.json` = **0 `PENDING`**.
  ADR-006/034 (Chyros/Langevin/Catnap/Tombstones) estão **"Aprovado e
  Implementado"** no engine.
- **Impacto**: o refinamento contínuo de memória está **pensado e implementado no
  engine, mas desconectado do cofre**; o cron do guia está obsoleto (não há
  `PENDING` a processar).
- **Proposta**: substituir o cron por um **lint/ingest incremental** no padrão LLM
  Wiki (`wiki/state.md` + lint periódico + `ladybug_reconcile`), com teto LOCAL e
  `no_agent`/script quando possível (custo zero).
- **Status**: **REGISTRADO / DECISÃO PENDENTE**.

---

## FEEDBACK-015 — `nlm` 0.16.9: `os error 5` no reinstall e `login --check` divergente por ambiente (2026-11-28)

- **Classificação**: BUG (ambiente Windows) + CORREÇÃO DE DIAGNÓSTICO
- **Evidência** (medida em disco):
  - `uv tool install --reinstall notebooklm-mcp-cli==0.16.9` →
    `error: failed to remove directory ...\uv\tools\notebooklm-mcp-cli\Scripts:
    Acesso negado (os error 5)`.
  - Processo medido: `chrome.exe` PID 248 + 10 filhos com
    `--user-data-dir=...\.notebooklm-mcp-cli\chrome-profiles\default
    --remote-debugging-port=9223 --headless=new`. É o browser de autenticação do
    próprio `nlm`; mantém `Scripts\python.exe` aberto e o Windows não permite apagar
    executável em uso. Corroborado por `astral-sh/uv#13986`, `#16747`, `#20792`,
    `github/spec-kit#5141`.
  - `agy mcp list` → `gemini_notebook_mcp  stdio  enabled` (o MCP do Antigravity é
    quem mantém o browser vivo). **Não é o Hermes nem o Souls Engine**: fechar o
    Hermes não libera o handle e encerra a sessão; o daemon `127.0.0.1:9123` não
    usa o tool env do `uv`.
- **Segundo achado, distinto**: `nlm login --check` no **terminal do usuário** →
  `✓ Authentication valid! / Notebooks found: 7 / Account: [REDACTED]` (rc=0).
  No **processo Hermes** (ambiente sanitizado de `terminal`/`execute_code`) →
  `RC=124` timeout. **Não é falha de autenticação nem envelope corrompido**:
  `nlm auth storage status` → `Storage mode: protected / Ciphertext envelope present
  (credentials.enc)`; `nlm notebook list` → rc=0, **7 cadernos / 877 fontes**;
  `nlm usage` → **0.0% usado / 100% restante**.
- **Correção de diagnóstico anterior**: a hipótese de "resíduo de `unfinished
  storage operation`" foi **retificada**. A causa do timeout é o **ambiente isolado
  do processo Hermes**, não o tool env. O `os error 5` do reinstall é um evento
  **separado**, já resolvido (o `Scripts/` foi recomposto em 27/11 20:57).
- **Impacto**: nenhum. `nlm` 0.16.9 funcional; FEEDBACK-009/013/014 (envelope,
  pydantic, tool env) **resolvidos**. O `nlm doctor` continua reportando
  `No AI tools configured` / `Gemini CLI: not configured` por **falso negativo**
  (FEEDBACK-008) — essa seção **não** deve ser usada como gate; usar `agy mcp list`.
- **Recomendação**: se precisar reinstalar, encerrar **apenas** o Chrome headless
  ou `agy mcp disable gemini_notebook_mcp` antes; **não** fechar o Hermes;
  **não** conceder `trust: true` novo (revogado em 26/11).
- **Status**: **REGISTRADO**. Nenhuma mutação em `raw/`, `AGENTS.md`, `SOUL.md`
  ou sidecar. Custo **R$ 0,00**.
