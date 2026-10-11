# INSTRUÇÃO TÁTICA DE ENGENHARIA BARE-METAL: PLANO DE RE-ARQUITETURA `souls_anthropophagy`

Este documento consolida a arquitetura técnica para o transplante do ecossistema de antropofagia legado (`_donor/souls_mc_anthropophagy/`) para a nova crate nativa e canônica do Souls Engine (`crates/souls_anthropophagy/`), sob a lei do **Single Crate Rule** e mantendo o doador estritamente **Read-Only**.

## 1. Decisões Arquiteturais Consolidadas (HITL / /grill-me)

Após deliberação via `/grill-me`, o Arquiteto definiu as seguintes bases para a crate:
- **Modelo de IA Exclusivo por Etapa:** A invocação das LLMs na fase cognitiva (`n4_synthesizer`) ocorrerá **diretamente via nuvem** (reqwest), contornando o Souls Model Router interno. As chaves, modelos específicos por fase (ex: qwen, deepseek, kimi) e base URLs serão definidos no `.env` (ex: `ANTHROPOPHAGY_N4_MODEL=deepseek/deepseek-chat`).
- **Orquestração On-Demand:** A crate será uma `lib`. O DAG (`pipeline.rs`) será invocado estritamente on-demand (acionado pelo Hermes Agent via requisição) e **não** ficará rodando como um daemon em loop infinito no `souls_server.exe`.
- **Sidecars em Caixa de Areia (Bin):** Os analisadores estáticos (`opengrep.exe`, `biome.exe`, `ruff.exe`, etc.) serão mantidos na pasta raiz `Z:\souls_engine\crates\souls_anthropophagy\bin` e controlados via `std::process::Command` para maximizar a confiabilidade legada.

## 2. Esquemas e Reutilização Canônica (SSOT)

Nenhum DTO, Tabela ou ENUM será inventado. Todo o mapeamento do SQLite foi extraído rigorosamente do legado (`ssot_injector.rs`):
- O banco será o `souls_heuristic_vault.db` local (FrankenSQLite).
- Estruturas como `TerminalClassification`, `CannibalizationAction`, `ArchitecturalCategory`, `RiskLevel4`, `TimeHorizon`, e a exata `MasterSolutionsRow` de 84 colunas serão espelhadas 1:1, sem adição de metadados alienígenas.
- O DAG se orienta pela coluna `status_processamento` (`PENDENTE`, `FASE_1_OK`, etc) da tabela `repositorios`.

## 3. Estrutura Proposta da Crate

```text
crates/souls_anthropophagy/src/
├── lib.rs                   # Contratos públicos e Facades para invocações On-Demand
├── pipeline.rs              # DAG Orquestrador assíncrono (executa N0..N6)
├── config.rs                # Mapeamento de LLMs por etapa e chaves via .env
├── vault.rs                 # Operações atômicas no FrankenSQLite STRICT WAL (repo_heuristics e artefatos_brutos)
├── models/
│   ├── mod.rs
│   └── taxonomy.rs          # Espelho 1:1 das ENUMs canônicas e das 84 colunas DTOs (MasterSolutionsRow)
├── sidecars.rs              # std::process::Command para binários SAST locais (opengrep, biome, etc.)
└── phases/
    ├── mod.rs
    ├── n0_inbound.rs        # GSheets -> SQLite (Identifica 'PENDENTE' ou 'RODAR_N*')
    ├── n1_guardian.rs       # Validador ETag/Release via GitHub API
    ├── n2_scout.rs          # Batedor FinOps - Triagem de README truncado
    ├── n3_harvester.rs      # Clone nativo via gix, extração AST (Tree-sitter desidratado) e logs SAST
    ├── n4_synthesizer.rs    # Motor Cognitivo: Chamadas REST (reqwest) exclusivas por etapa de destilação
    ├── n5_deep_formatter.rs # Formatação Pydantic/JSON estrito
    └── outbox_sync.rs       # SQLite -> batchUpdate no GSheets
```

## 4. Questionamentos de Aprofundamento (/grill-me Rodada 2)

> [!WARNING]
> Arquiteto, apliquei as suas definições com sucesso. Ao dissecar `outbox_sync.rs`, `synthesizer.rs` e o diretório `sast/`, me deparei com os últimos 3 pontos de interrogação que definem o detalhe microscópico da extração. Por favor responda:

1. **Prompts e Heurísticas Cognitivas (N4):** O módulo `synthesizer` legado contém blocos e lógicas complexas de prompting (destilação, classificação de componentes, avaliação de risco, etc). Quer que eu codifique esses system prompts gigante como `const` hardcoded no próprio `n4_synthesizer.rs` ou seria melhor criar um diretório de arquivos estáticos markdown (ex: `crates/souls_anthropophagy/prompts/`) carregados dinamicamente via `include_str!` para facilitar a sua manutenção pelo VSCode?
2. **Compressão/Desidratação no Harvester (N3):** O harvester local (usando Tree-sitter e sidecars) deve continuar empurrando 100% de seus logs e códigos (JSONs brutos) para a tabela `artefatos_brutos` no SQLite (FrankenSQLite pode inchar rápido se não for otimizado), ou devemos usar compressão ZSTD nos payloads que entram nessa tabela para manter o banco leve?
3. **Autenticação com o Google Workspace (Inbound/Outbox):** O `daemon_watcher.rs` e `outbox_sync.rs` do doador consumiam a função `souls_mc_lib::persist::google_workspace_mcp`. Dado que a crate deve ser enxuta e autossuficiente (sem reinventar a roda), você quer que a `souls_anthropophagy` continue usando as chaves Service Account via MCP do Hermes, ou prefere injetar um Service Account JWT bruto no `.env` da própria engine (ex: `GOOGLE_SA_JSON_PATH`) para a crate fazer o `reqwest` puro e autônomo nas planilhas do Sheets?

Aguardo suas respostas no `/grill-me` para finalizar o Plano e iniciarmos a codificação.
