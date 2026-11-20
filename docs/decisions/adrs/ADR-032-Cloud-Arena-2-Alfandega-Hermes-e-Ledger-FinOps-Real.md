# ADR-032: Cloud Arena 2.0, Alfândega Hermes ↔ Souls e Ledger FinOps Real do Hermes (`v0.9.2.2`)

## Status
**Aprovado e Implementado** (Milestone v0.9.2.2 / `version = "0.9.2+2"` SemVer 2.0)

## Data
2026-11-20

## Contexto e Causas-Raiz Diagnosticadas no Ambiente Real
Durante a auditoria operacional entre o Souls Engine (`Z:\souls_engine`) e o **Hermes Agent Desktop** (`%LOCALAPPDATA%\hermes`), foram confirmadas quatro causas-raiz e sete pontos de quebra na integração entre a Cloud Arena e o chat mestre do Hermes:

1. **Slugs de Provedor Inválidos e Mutação Incompleta no `config.yaml`:**
   - O antigo `apply_profile_to_hermes()` gravava identificadores internos (`"nous_portal"`, `"openrouter_market"`, `"local_silicon"`) no `config.yaml` do Hermes e alterava apenas `model.default` e `model.provider`, deixando `base_url`, `api_mode` e `context_length` órfãos. O Hermes Desktop (`hermes_cli/providers.py`) rejeitava trocas posteriores com `JsonRpcGatewayError: model_resolution_failed`.
2. **Sessões Presas em Fallback Local (`state.db` + `moa.enabled` + Falta de Pré-Voo OAuth):**
   - Com o OAuth do Nous Portal expirado (`relogin_required: true`, `code: invalid_grant` em `auth.json`), novas mensagens caíam silenciosamente para o fallback local (`llamacpp`), gravando `"gateway_runtime": {"provider": "llamacpp"}` sem `"follow_profile_config": true` dentro do `state.db`. Sessões já abertas permaneciam presas no modelo local mesmo após ajustes no `config.yaml`, enquanto `moa.enabled: true` continuava disparando múltiplas chamadas em paralelo ao Nous.
3. **Colisão de Variantes, Perda de `:free` e Modelos Exclusivos:**
   - `nous.rs` descartava todos os modelos gratuitos (`:free`); `openrouter.rs` e `sync.rs` não registravam modelos exclusivos do OpenRouter; `seed_aliases.rs` continha alias conflitante (`~openai/gpt-sol-latest`) e `cloud.rs` consultava `has_nous`/`has_openrouter` sem filtrar pela última data (`MAX(recorded_at)`) nem excluir `nous_portal_batch`. No frontend, `humanizeModelName()` usava `.includes()` genéricos que colidiam variantes (`gpt-6.1-sol-pro` vs `gpt-6.1-sol`, `deepseek-v4-pro` vs `deepseek-v4`), a ordenação de `Inteligência` comparava campos diferentes dos renderizados na célula e a tabela não rebalanceava `Status`/Pódio ao mudar filtros.
4. **Falta de Visibilidade Financeira Real do Hermes:**
   - Os consumos reais registrados pelo Hermes em `session_model_usage` (`state.db` e `profiles/*/state.db`) frequentemente ficavam com `cost_status = 'unknown'`, sem consolidação histórica nem projeção mensal por gateway, modelo e perfil.

---

## Decisões Arquiteturais

### 1. Canonicalização de Rotas e Catálogo Unificado (`souls_llm_cloud_arena` + `profiles.rs`)
- **Preservação Estrita de Variantes & Modelos Gratuitos (`:free`):** `nous.rs`, `openrouter.rs`, `resolver.rs`, `seed_aliases.rs` e `sync.rs` preservam `:free` com tarifa `$0.00 / 1M` e variantes (`-pro`, `-flash`, `-flex`, `-high`, `-max`), ingerindo simultaneamente o catálogo Nous Portal e modelos generativos exclusivos do OpenRouter.
- **Índices Únicos Idempotentes & Provedor Canônico:** `migrations.rs` garante unicidade em `daily_evaluations(canonical_id, evaluation_date, profile_task)` e `repository.rs::get_champions()` emite slugs canônicos (`"nous"`, `"openrouter"`).
- **Normalização Canônica (`normalize_hermes_provider_and_route`):** Traduz aliases internos (`nous_portal`, `openrouter_market`, `local_silicon`, `llamacpp`, `google`) para slugs oficiais do Hermes (`nous`, `openrouter`, `custom`, `gemini`, `deepseek`, `anthropic`, `openai-api`, `zai`, `xai`, `alibaba`, `copilot`, etc.), gravando atomicamente `default`, `provider`, `base_url`, `api_mode` e removendo `context_length`/`key_env` obsoletos ao trocar de rota.
- **Destravamento Multi-Perfil, Sessões e Favoritos:** `apply_profile_to_hermes_extended()` atualiza opcionalmente todos os sub-perfis (`profiles/*/config.yaml`), destrava sessões presas em `llamacpp` no `state.db` (`follow_profile_config: true`) e adiciona `<provider>::<model_id>` não-destrutivamente em `ui-preferences.json` (`hermes.desktop.favorite-models`).

### 2. Alfândega Hermes ↔ Souls (`/api/v1/hermes/customs` & `/repair`)
- **`inspect_hermes_customs_with_options()`:** Audita `auth.json`, `.env`, `credentials.env`, `config.yaml` (raiz e sub-perfis, `moa.enabled`, `fallback_providers`) e `state.db` (contagem de sessões presas) sem expor segredos brutos.
- **`POST /api/v1/settings/hermes-preset`:** Com `strict_auth: true` na UI, bloqueia trocas para gateways não autenticados (retornando `status: "auth_required"`) e oferece facilitadores 1-clique (`hermes login --provider nous` ou gravação inline de API keys em `%LOCALAPPDATA%\hermes\.env` e `profiles/*/.env`).
- **`POST /api/v1/hermes/customs/repair`:** Corrige slugs/rotas corrompidos, alterna `moa.enabled` de forma segura (apenas indentação raiz do bloco `moa:`) e destrava sessões presas em fallback.

### 3. Cloud Arena 2.0: Rebalanceamento Dinâmico & Ordenação Unificada
- **Ordem Canônica de Gateways:** **`Nous Portal` (padrão)** → **`OpenRouter`** → **Gateways Ativas dinâmicas** detectadas pela Alfândega → **`Gratuitos ($0)`** → **`Todos (Mercado Global)`**, com contadores dinâmicos e botão **"Limpar Filtros"**.
- **Usabilidade Imediata:** Coluna **`Ação` (`Usar`)** movida imediatamente após **`Status`**; modal `#modal-quick-apply` enriquecido com seleção de Gateway, checagem real de credencial e **Simulador de Impacto Financeiro Mensal**.
- **Fidelidade Matemática:** `getEffectiveCloudModelMetrics()` e `computeDynamicParetoTags()` unificam Pódio, badges da tabela e `.sort()` em todas as colunas (sincronizado bidirecionalmente com `#cloud-sort-select`).

### 4. Ledger FinOps Real do Hermes (`hermes_usage_ledger` & `/api/v1/cloud/finops`)
- Tabela `hermes_usage_ledger` (`STRICT`) em `souls_heuristic_vault.db`.
- `sync_and_summarize_hermes_finops()` abre `%LOCALAPPDATA%\hermes\state.db` e `profiles/*/state.db` em modo somente leitura (`?mode=ro`, com fallback em cópia temporária caso haja lock), cruza `input_tokens`, `output_tokens`, `cache_read_tokens` e `cache_write_tokens` com as tarifas da Cloud Arena e expõe períodos (**Hoje, 7d, 30d, Acumulado, Projeção Mensal Run-Rate**), economia de Prompt Cache e breakdowns na **Cloud Arena** e na **Visão Geral** (`/api/v1/telemetry`).
