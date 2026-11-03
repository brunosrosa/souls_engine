import re

path = r"Z:\souls_engine\crates\souls_server\src\dashboard_ui.html"
with open(path, "r", encoding="utf-8") as f:
    content = f.read()

# 1. Patch CSS
css_needle = ".badge-active-hermes {"
css_replacement = """    /* Sticky columns & Column Visibility for Arena Table */
    .arena-table-sticky-wrapper {
      position: relative;
      overflow-x: auto;
      max-width: 100%;
    }
    .arena-table th.sticky-col-1,
    .arena-table td.sticky-col-1 {
      position: sticky;
      left: 0;
      z-index: 10;
      background: #0f172a;
    }
    .arena-table th.sticky-col-2,
    .arena-table td.sticky-col-2 {
      position: sticky;
      left: 42px;
      z-index: 10;
      background: #0f172a;
      box-shadow: 3px 0 8px rgba(0, 0, 0, 0.45);
    }
    .arena-table thead th.sticky-col-1,
    .arena-table thead th.sticky-col-2 {
      z-index: 25;
      background: #0b1120;
    }
    .arena-table tbody tr:hover td.sticky-col-1,
    .arena-table tbody tr:hover td.sticky-col-2 {
      background: #1e293b !important;
    }
    .arena-table tbody tr.active-hermes-row td.sticky-col-1,
    .arena-table tbody tr.active-hermes-row td.sticky-col-2 {
      background: #0d211f !important;
    }
    .col-hidden {
      display: none !important;
    }
    .th-guide {
      cursor: help;
      text-decoration: underline dotted rgba(255, 255, 255, 0.25);
    }
    .badge-cheaper {
      background: rgba(16, 185, 129, 0.18);
      color: #34d399;
      border: 1px solid rgba(16, 185, 129, 0.4);
      font-size: 0.6rem;
      padding: 1px 4px;
      border-radius: 3px;
      margin-left: 4px;
    }
    .col-toggle-item {
      display: flex;
      align-items: center;
      gap: 6px;
      font-size: 0.72rem;
      color: #cbd5e1;
      cursor: pointer;
      padding: 3px 4px;
      border-radius: 3px;
      user-select: none;
    }
    .col-toggle-item:hover {
      background: rgba(255, 255, 255, 0.05);
      color: #fff;
    }

    .badge-active-hermes {"""

if css_needle in content and "arena-table-sticky-wrapper" not in content:
    content = content.replace(css_needle, css_replacement, 1)
    print("CSS patched.")

# 2. Patch Provider Filter button label: 🔀 OpenRouter (Referência) -> 🔀 OpenRouter (API)
content = content.replace("🔀 OpenRouter (Referência)", "🔀 OpenRouter (API)")
print("Provider filter button label updated.")

# 3. Patch Table Controls & Headers
old_table_block = """        <!-- SEÇÃO 2: RANKING COMPARATIVO DE MERCADO -->
        <div style="margin-top: 18px; margin-bottom: 10px; display: flex; align-items: center; justify-content: space-between; flex-wrap: wrap; gap: 10px;">
          <div>
            <h4 style="font-size: 0.88rem; color: #fff; font-weight: 700; display: flex; align-items: center; gap: 6px; margin: 0;">
              <span style="color: var(--cyan);">📊</span> Ranking Comparativo de Modelos (Benchmarks & Custos)
            </h4>
            <span style="font-size: 0.72rem; color: var(--text-muted);" id="cloud-ranked-count">Carregando catálogo...</span>
          </div>

          <!-- BARRA DE PESQUISA E ORDENAÇÃO RÁPIDA -->
          <div style="display: flex; align-items: center; gap: 8px; flex-wrap: wrap;">
            <div style="position: relative;">
              <input type="text" id="cloud-search-input" placeholder="🔍 Filtrar modelo, fabricante ou id..." oninput="handleCloudSearch(this.value)" style="background: rgba(15, 23, 42, 0.8); border: 1px solid var(--border-subtle); color: #fff; border-radius: var(--radius-sm); padding: 5px 10px; font-size: 0.76rem; width: 230px;" />
            </div>
            <select id="cloud-sort-select" onchange="handleCloudSort(this.value)" style="background: rgba(15, 23, 42, 0.8); border: 1px solid var(--border-subtle); color: #cbd5e1; border-radius: var(--radius-sm); padding: 5px 8px; font-size: 0.76rem;">
              <option value="score_desc">🏆 Maior Score Geral</option>
              <option value="coding_desc">💻 Maior Score Coding (SWE)</option>
              <option value="bfcl_desc">🤖 Maior Score Tool Calling (BFCL)</option>
              <option value="reasoning_desc">🧠 Maior Score Reasoning (Hard/Math)</option>
              <option value="roi_desc">💎 Maior Custo-Benefício (ROI)</option>
              <option value="cost_asc">💵 Menor Tarifa ($/1M)</option>
              <option value="tps_desc">⚡ Maior Throughput (TPS)</option>
              <option value="ctx_desc">📚 Maior Janela de Contexto</option>
            </select>
          </div>
        </div>

        <div class="table-container">
          <table class="arena-table">
            <thead>
              <tr>
                <th class="text-center sortable-th" style="width: 45px;" onclick="setTableSort('rank')"># <span class="sort-icon" id="sort-icon-rank">↕</span></th>
                <th class="text-center" style="width: 125px;">Status</th>
                <th class="sortable-th" style="min-width: 230px;" onclick="setTableSort('name')">Modelo / Identificador <span class="sort-icon" id="sort-icon-name">↕</span></th>
                <th class="text-center" style="width: 130px;">Disponibilidade</th>
                <th class="text-center sortable-th" style="width: 110px;" onclick="setTableSort('cost')" title="Tarifa por 1M de tokens combinados">Tarifa ($/1M) <span class="sort-icon" id="sort-icon-cost">↕</span></th>
                <th class="text-center sortable-th active-sort" style="width: 90px;" onclick="setTableSort('score')" title="Score ponderado de inteligência multiobjetivo">Score Geral <span class="sort-icon" id="sort-icon-score">▼</span></th>
                <th class="text-center sortable-th" style="width: 85px;" onclick="setTableSort('coding')" title="Capacidade em Engenharia de Software e Programação">Coding <span class="sort-icon" id="sort-icon-coding">↕</span></th>
                <th class="text-center sortable-th" style="width: 85px;" onclick="setTableSort('bfcl')" title="Berkeley Function Calling: precisão em chamar ferramentas MCP">Tools (BFCL) <span class="sort-icon" id="sort-icon-bfcl">↕</span></th>
                <th class="text-center sortable-th" style="width: 90px;" onclick="setTableSort('reasoning')" title="Raciocínio complexo, problemas difíceis e matemática">Reasoning <span class="sort-icon" id="sort-icon-reasoning">↕</span></th>
                <th class="text-center sortable-th" style="width: 85px;" onclick="setTableSort('roi')" title="Relação entre capacidade cognitiva e custo financeiro">ROI Index <span class="sort-icon" id="sort-icon-roi">↕</span></th>
                <th class="text-center sortable-th" style="width: 80px;" onclick="setTableSort('tps')" title="Velocidade média em tokens por segundo">TPS <span class="sort-icon" id="sort-icon-tps">↕</span></th>
                <th class="text-center sortable-th" style="width: 95px;" onclick="setTableSort('ctx')" title="Janela de contexto máximo suportada">Janela Ctx <span class="sort-icon" id="sort-icon-ctx">↕</span></th>
                <th class="text-center" style="width: 80px;">Ação</th>
              </tr>
            </thead>
            <tbody id="cloud-ranked-tbody">
              <!-- Renderizado dinamicamente via fetchCloudRankedModels() -->
              <tr>
                <td colspan="13" style="text-align: center; color: var(--text-muted); padding: 24px;">Buscando modelos da Cloud Arena com sub-scores de benchmarks...</td>
              </tr>
            </tbody>"""

new_table_block = """        <!-- SEÇÃO 2: RANKING COMPARATIVO DE MERCADO -->
        <div style="margin-top: 18px; margin-bottom: 10px; display: flex; align-items: center; justify-content: space-between; flex-wrap: wrap; gap: 10px;">
          <div>
            <h4 style="font-size: 0.88rem; color: #fff; font-weight: 700; display: flex; align-items: center; gap: 6px; margin: 0;">
              <span style="color: var(--cyan);">📊</span> Ranking Comparativo de Modelos (Benchmarks & Custos)
            </h4>
            <span style="font-size: 0.72rem; color: var(--text-muted);" id="cloud-ranked-count">Carregando catálogo...</span>
          </div>

          <!-- BARRA DE PESQUISA, SELETOR DE COLUNAS E ORDENAÇÃO -->
          <div style="display: flex; align-items: center; gap: 8px; flex-wrap: wrap;">
            <div style="position: relative;">
              <input type="text" id="cloud-search-input" placeholder="🔍 Filtrar modelo, fabricante ou id..." oninput="handleCloudSearch(this.value)" style="background: rgba(15, 23, 42, 0.8); border: 1px solid var(--border-subtle); color: #fff; border-radius: var(--radius-sm); padding: 5px 10px; font-size: 0.76rem; width: 210px;" />
            </div>

            <!-- CONTROLE DE VISIBILIDADE DE COLUNAS (DROPDOWN) -->
            <div style="position: relative;">
              <button class="btn" id="btn-col-toggle" onclick="toggleColumnMenu(event)" style="background: rgba(15, 23, 42, 0.85); border: 1px solid var(--border-subtle); color: #cbd5e1; font-size: 0.76rem; padding: 5px 10px; display: flex; align-items: center; gap: 5px; cursor: pointer;">
                <span>👁️</span> Colunas <span id="col-count-badge" style="background: rgba(56, 189, 248, 0.2); color: #38bdf8; font-size: 0.65rem; padding: 1px 4px; border-radius: 3px;">12/12</span> ▾
              </button>
              <div id="col-dropdown-menu" style="display: none; position: absolute; right: 0; top: 100%; margin-top: 5px; background: #0f172a; border: 1px solid var(--border-subtle); border-radius: var(--radius-md); padding: 10px; width: 250px; z-index: 100; box-shadow: 0 12px 30px rgba(0,0,0,0.65);">
                <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 8px; border-bottom: 1px solid rgba(255,255,255,0.08); padding-bottom: 5px;">
                  <span style="font-size: 0.72rem; font-weight: 700; color: #fff;">Exibir Colunas</span>
                  <div style="display: flex; gap: 6px;">
                    <button onclick="toggleAllColumns(true)" style="background: none; border: none; color: var(--cyan); font-size: 0.68rem; cursor: pointer;">Todas</button>
                    <span style="color: var(--text-muted);">|</span>
                    <button onclick="resetDefaultColumns()" style="background: none; border: none; color: #a78bfa; font-size: 0.68rem; cursor: pointer;">Padrão</button>
                  </div>
                </div>
                <div style="display: flex; flex-direction: column; gap: 4px; max-height: 280px; overflow-y: auto;">
                  <label class="col-toggle-item"><input type="checkbox" id="chk-col-status" checked onchange="toggleColumn('status', this.checked)"> Status</label>
                  <label class="col-toggle-item"><input type="checkbox" id="chk-col-score" checked onchange="toggleColumn('score', this.checked)"> Score Geral</label>
                  <label class="col-toggle-item"><input type="checkbox" id="chk-col-roi" checked onchange="toggleColumn('roi', this.checked)"> ROI Index</label>
                  <label class="col-toggle-item"><input type="checkbox" id="chk-col-nous" checked onchange="toggleColumn('nous', this.checked)"> Tarifa Nous ($/1M)</label>
                  <label class="col-toggle-item"><input type="checkbox" id="chk-col-or" checked onchange="toggleColumn('or', this.checked)"> Tarifa OpenRouter ($/1M)</label>
                  <label class="col-toggle-item"><input type="checkbox" id="chk-col-coding" checked onchange="toggleColumn('coding', this.checked)"> Coding (SWE)</label>
                  <label class="col-toggle-item"><input type="checkbox" id="chk-col-bfcl" checked onchange="toggleColumn('bfcl', this.checked)"> Tools (BFCL)</label>
                  <label class="col-toggle-item"><input type="checkbox" id="chk-col-reasoning" checked onchange="toggleColumn('reasoning', this.checked)"> Reasoning (GPQA)</label>
                  <label class="col-toggle-item"><input type="checkbox" id="chk-col-ifeval" checked onchange="toggleColumn('ifeval', this.checked)"> Instrução (IFEval)</label>
                  <label class="col-toggle-item"><input type="checkbox" id="chk-col-tps" checked onchange="toggleColumn('tps', this.checked)"> Throughput (TPS)</label>
                  <label class="col-toggle-item"><input type="checkbox" id="chk-col-ctx" checked onchange="toggleColumn('ctx', this.checked)"> Janela Ctx</label>
                  <label class="col-toggle-item"><input type="checkbox" id="chk-col-prov" checked onchange="toggleColumn('prov', this.checked)"> Disponibilidade</label>
                </div>
              </div>
            </div>

            <select id="cloud-sort-select" onchange="handleCloudSort(this.value)" style="background: rgba(15, 23, 42, 0.8); border: 1px solid var(--border-subtle); color: #cbd5e1; border-radius: var(--radius-sm); padding: 5px 8px; font-size: 0.76rem;">
              <option value="score_desc">🏆 Maior Score Geral</option>
              <option value="roi_desc">💎 Maior Custo-Benefício (ROI)</option>
              <option value="nous_asc">🧠 Menor Tarifa Nous ($/1M)</option>
              <option value="or_asc">🔀 Menor Tarifa OpenRouter ($/1M)</option>
              <option value="coding_desc">💻 Maior Score Coding (SWE)</option>
              <option value="bfcl_desc">🤖 Maior Score Tool Calling (BFCL)</option>
              <option value="reasoning_desc">🧠 Maior Score Reasoning (Hard/Math)</option>
              <option value="ifeval_desc">📋 Maior Score Instrução (IFEval)</option>
              <option value="tps_desc">⚡ Maior Throughput (TPS)</option>
              <option value="ctx_desc">📚 Maior Janela de Contexto</option>
            </select>
          </div>
        </div>

        <div class="table-container arena-table-sticky-wrapper">
          <table class="arena-table">
            <thead>
              <tr>
                <!-- COLUNA 1: POSIÇÃO (CONGELADA / STICKY) -->
                <th class="text-center sortable-th sticky-col-1 th-guide" style="width: 42px;" onclick="setTableSort('rank')" title="Posição no ranking canônico pelo perfil ativo.&#10;📉 Como Interpretar: Quanto MENOR o número, melhor (#1 é o topo).">
                  # <span class="sort-icon" id="sort-icon-rank">↕</span>
                </th>

                <!-- COLUNA 2: MODELO / IDENTIFICADOR (CONGELADA / STICKY ANTES DO STATUS) -->
                <th class="sortable-th sticky-col-2 th-guide" style="min-width: 250px;" onclick="setTableSort('name')" title="Identificador do modelo e fabricante.&#10;Clique no botão de prancheta para copiar o ID canônico para a área de transferência.">
                  Modelo / Identificador <span class="sort-icon" id="sort-icon-name">↕</span>
                </th>

                <!-- COLUNA 3: STATUS PARETO -->
                <th class="text-center th-col-status th-guide" style="width: 120px;" title="Status de Otimização na Fronteira de Pareto.&#10;👑 Líder Pareto: Ponto ótimo não-dominado.&#10;💎 Melhor ROI: Maior razão inteligência por dólar.&#10;🔹 Alternativa: Modelo homologado complementar.">
                  Status
                </th>

                <!-- COLUNA 4: SCORE GERAL -->
                <th class="text-center sortable-th active-sort th-col-score th-guide" style="width: 90px;" onclick="setTableSort('score')" title="Score Multiobjetivo Geral Ponderado (0 a 100).&#10;📈 Como Interpretar: Quanto MAIOR, melhor.&#10;Cálculo: Combinação de SWE-bench, BFCL e GPQA com pesos do perfil ativo (Maestro: 35% Tools + 35% Reasoning + 30% Coding).">
                  Score Geral <span class="sort-icon" id="sort-icon-score">▼</span>
                </th>

                <!-- COLUNA 5: ROI INDEX (IMEDIATAMENTE AO LADO DO SCORE GERAL) -->
                <th class="text-center sortable-th th-col-roi th-guide" style="width: 90px;" onclick="setTableSort('roi')" title="Índice de Retorno sobre Investimento (Score ÷ Blended Cost).&#10;📈 Como Interpretar: Quanto MAIOR, melhor.&#10;Cálculo: Mede o volume de inteligência entregue por cada dólar investido.">
                  ROI Index <span class="sort-icon" id="sort-icon-roi">↕</span>
                </th>

                <!-- COLUNA 6: TARIFA NOUS PORTAL -->
                <th class="text-center sortable-th th-col-nous th-guide" style="width: 115px;" onclick="setTableSort('nous_cost')" title="Tarifa da API do Nous Portal por 1M de tokens (Blended: 25% Entrada + 75% Saída).&#10;📉 Como Interpretar: Quanto MENOR, melhor.&#10;Utiliza o saldo de créditos cadastrado na sua conta do Nous Portal para o Hermes.">
                  Tarifa Nous ($/1M) <span class="sort-icon" id="sort-icon-nous_cost">↕</span>
                </th>

                <!-- COLUNA 7: TARIFA OPENROUTER -->
                <th class="text-center sortable-th th-col-or th-guide" style="width: 115px;" onclick="setTableSort('or_cost')" title="Tarifa da API do OpenRouter por 1M de tokens (Blended: 25% Entrada + 75% Saída).&#10;📉 Como Interpretar: Quanto MENOR, melhor.&#10;Cotação de mercado direta para comparar se vale a pena usar créditos no OpenRouter.">
                  Tarifa OR ($/1M) <span class="sort-icon" id="sort-icon-or_cost">↕</span>
                </th>

                <!-- COLUNA 8: CODING (SWE-BENCH) -->
                <th class="text-center sortable-th th-col-coding th-guide" style="width: 85px;" onclick="setTableSort('coding')" title="Capacidade em Engenharia de Software e Programação (SWE-bench / Code ELO).&#10;📈 Como Interpretar: Quanto MAIOR, melhor.&#10;Mede resolução real de bugs e refatoração de código.">
                  Coding <span class="sort-icon" id="sort-icon-coding">↕</span>
                </th>

                <!-- COLUNA 9: TOOLS (BFCL) -->
                <th class="text-center sortable-th th-col-bfcl th-guide" style="width: 85px;" onclick="setTableSort('bfcl')" title="Berkeley Function Calling Leaderboard (Chamada de Ferramentas e MCP).&#10;📈 Como Interpretar: Quanto MAIOR, melhor.&#10;Mede precisão sintática e semântica no disparo de tools.">
                  Tools (BFCL) <span class="sort-icon" id="sort-icon-bfcl">↕</span>
                </th>

                <!-- COLUNA 10: REASONING (GPQA) -->
                <th class="text-center sortable-th th-col-reasoning th-guide" style="width: 90px;" onclick="setTableSort('reasoning')" title="Raciocínio Lógico e Científico (GPQA Diamond / Hard Prompts ELO).&#10;📈 Como Interpretar: Quanto MAIOR, melhor.&#10;Avalia deduções complexas em problemas de pós-graduação.">
                  Reasoning <span class="sort-icon" id="sort-icon-reasoning">↕</span>
                </th>

                <!-- COLUNA 11: INSTRUÇÃO (IFEVAL) -->
                <th class="text-center sortable-th th-col-ifeval th-guide" style="width: 85px;" onclick="setTableSort('ifeval')" title="Fidelidade a Instruções e Restrições de Formato (IFEval).&#10;📈 Como Interpretar: Quanto MAIOR, melhor.&#10;Mede cumprimento estrito de JSON schemas e formatação.">
                  Instrução <span class="sort-icon" id="sort-icon-ifeval">↕</span>
                </th>

                <!-- COLUNA 12: TPS -->
                <th class="text-center sortable-th th-col-tps th-guide" style="width: 80px;" onclick="setTableSort('tps')" title="Velocidade média em tokens por segundo (TPS).&#10;📈 Como Interpretar: Quanto MAIOR, melhor.&#10;Indica velocidade de resposta para subagentes em lote.">
                  TPS <span class="sort-icon" id="sort-icon-tps">↕</span>
                </th>

                <!-- COLUNA 13: JANELA CTX -->
                <th class="text-center sortable-th th-col-ctx th-guide" style="width: 95px;" onclick="setTableSort('ctx')" title="Capacidade máxima de janela de contexto em tokens (input + output).&#10;📈 Como Interpretar: Quanto MAIOR, melhor.&#10;Permite ingerir repositórios inteiros sem truncamento.">
                  Janela Ctx <span class="sort-icon" id="sort-icon-ctx">↕</span>
                </th>

                <!-- COLUNA 14: DISPONIBILIDADE -->
                <th class="text-center th-col-prov th-guide" style="width: 120px;" title="Provedores com modelo indexado e pronto para consumo via API.&#10;🧠 Nous: Disponível com créditos ativos do Hermes.&#10;🔀 OR: Disponível via API OpenRouter.">
                  Provedores
                </th>

                <!-- COLUNA 15: AÇÃO -->
                <th class="text-center" style="width: 75px;">Ação</th>
              </tr>
            </thead>
            <tbody id="cloud-ranked-tbody">
              <!-- Renderizado dinamicamente via fetchCloudRankedModels() -->
              <tr>
                <td colspan="15" style="text-align: center; color: var(--text-muted); padding: 24px;">Buscando modelos da Cloud Arena com sub-scores de benchmarks e tarifas comparadas...</td>
              </tr>
            </tbody>"""

if old_table_block in content:
    content = content.replace(old_table_block, new_table_block, 1)
    print("Table block updated.")
else:
    print("WARNING: old_table_block not matched directly!")

with open(path, "w", encoding="utf-8") as f:
    f.write(content)
print("Saved part 1.")

