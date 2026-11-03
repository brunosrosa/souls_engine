import re

with open('crates/souls_server/src/dashboard_ui.html', 'r', encoding='utf-8') as f:
    html = f.read()

# 1. Expand the column dropdown menu
old_col_menu = """                <div style="display: flex; flex-direction: column; gap: 4px; max-height: 280px; overflow-y: auto;">
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
                </div>"""

new_col_menu = """                <div style="display: flex; flex-direction: column; gap: 4px; max-height: 280px; overflow-y: auto;">
                  <label class="col-toggle-item"><input type="checkbox" id="chk-col-status" checked onchange="toggleColumn('status', this.checked)"> Status Pareto</label>
                  <label class="col-toggle-item"><input type="checkbox" id="chk-col-score" checked onchange="toggleColumn('score', this.checked)"> Score Geral</label>
                  <label class="col-toggle-item"><input type="checkbox" id="chk-col-roi" checked onchange="toggleColumn('roi', this.checked)"> ROI Index</label>
                  <label class="col-toggle-item"><input type="checkbox" id="chk-col-nous" checked onchange="toggleColumn('nous', this.checked)"> Tarifa Nous ($/1M)</label>
                  <label class="col-toggle-item"><input type="checkbox" id="chk-col-or" checked onchange="toggleColumn('or', this.checked)"> Tarifa OpenRouter ($/1M)</label>
                  <label class="col-toggle-item"><input type="checkbox" id="chk-col-intel" checked onchange="toggleColumn('intel', this.checked)"> 🧠 Inteligência (Arena ELO)</label>
                  <label class="col-toggle-item"><input type="checkbox" id="chk-col-coding" checked onchange="toggleColumn('coding', this.checked)"> 💻 Coding (SWE)</label>
                  <label class="col-toggle-item"><input type="checkbox" id="chk-col-bfcl" checked onchange="toggleColumn('bfcl', this.checked)"> 🤖 Tools (BFCL)</label>
                  <label class="col-toggle-item"><input type="checkbox" id="chk-col-reasoning" checked onchange="toggleColumn('reasoning', this.checked)"> 🧠 Reasoning (GPQA)</label>
                  <label class="col-toggle-item"><input type="checkbox" id="chk-col-design" checked onchange="toggleColumn('design', this.checked)"> 🎨 Design UI/UX</label>
                  <label class="col-toggle-item"><input type="checkbox" id="chk-col-vision" checked onchange="toggleColumn('vision', this.checked)"> 👁️ Visão (VQA)</label>
                  <label class="col-toggle-item"><input type="checkbox" id="chk-col-ifeval" checked onchange="toggleColumn('ifeval', this.checked)"> 📋 Instrução (IFEval)</label>
                  <label class="col-toggle-item"><input type="checkbox" id="chk-col-ttft" checked onchange="toggleColumn('ttft', this.checked)"> ⏱️ Latência (TTFT)</label>
                  <label class="col-toggle-item"><input type="checkbox" id="chk-col-tps" checked onchange="toggleColumn('tps', this.checked)"> ⚡ Throughput (TPS)</label>
                  <label class="col-toggle-item"><input type="checkbox" id="chk-col-ctx" checked onchange="toggleColumn('ctx', this.checked)"> 📚 Janela & RULER</label>
                  <label class="col-toggle-item"><input type="checkbox" id="chk-col-prov" checked onchange="toggleColumn('prov', this.checked)"> 🌐 Provedores</label>
                </div>"""

if old_col_menu in html:
    html = html.replace(old_col_menu, new_col_menu)
    print("Updated column dropdown checkboxes")
else:
    print("WARNING: old_col_menu not found")

# 2. Update sort select dropdown
old_sort_select = """            <select id="cloud-sort-select" onchange="handleCloudSort(this.value)" style="background: rgba(15, 23, 42, 0.8); border: 1px solid var(--border-subtle); color: #cbd5e1; border-radius: var(--radius-sm); padding: 5px 8px; font-size: 0.76rem;">
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
            </select>"""

new_sort_select = """            <select id="cloud-sort-select" onchange="handleCloudSort(this.value)" style="background: rgba(15, 23, 42, 0.8); border: 1px solid var(--border-subtle); color: #cbd5e1; border-radius: var(--radius-sm); padding: 5px 8px; font-size: 0.76rem;">
              <option value="score_desc">🏆 Maior Score Geral</option>
              <option value="roi_desc">💎 Maior Custo-Benefício (ROI)</option>
              <option value="nous_asc">🧠 Menor Tarifa Nous ($/1M)</option>
              <option value="or_asc">🔀 Menor Tarifa OpenRouter ($/1M)</option>
              <option value="intel_desc">🧠 Maior Inteligência (Arena ELO)</option>
              <option value="coding_desc">💻 Maior Score Coding (SWE)</option>
              <option value="bfcl_desc">🤖 Maior Score Tool Calling (BFCL)</option>
              <option value="reasoning_desc">🧠 Maior Score Reasoning (Hard/Math)</option>
              <option value="design_desc">🎨 Maior Score Design / UI/UX</option>
              <option value="vision_desc">👁️ Maior Score Visão / VQA</option>
              <option value="ifeval_desc">📋 Maior Score Instrução (IFEval)</option>
              <option value="ttft_asc">⏱️ Menor Latência (TTFT)</option>
              <option value="tps_desc">⚡ Maior Throughput (TPS)</option>
              <option value="ctx_desc">📚 Maior Janela de Contexto</option>
            </select>"""

if old_sort_select in html:
    html = html.replace(old_sort_select, new_sort_select)
    print("Updated sort select options")
else:
    print("WARNING: old_sort_select not found")

# 3. Update table headers
old_table_header = """          <table class="arena-table">
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
            </tbody>
          </table>"""

new_table_header = """          <table class="arena-table">
            <thead>
              <tr>
                <!-- COLUNA 1: POSIÇÃO (CONGELADA / STICKY) -->
                <th class="text-center sortable-th sticky-col-1 th-guide" style="width: 42px;" onclick="setTableSort('rank')" title="Posição no ranking canônico pelo perfil ativo.&#10;📉 Como Interpretar: Quanto MENOR o número, melhor (#1 é o topo).">
                  # <span class="sort-icon" id="sort-icon-rank">↕</span>
                </th>

                <!-- COLUNA 2: MODELO / IDENTIFICADOR (CONGELADA / STICKY ANTES DO STATUS) -->
                <th class="sortable-th sticky-col-2 th-guide" style="min-width: 250px;" onclick="setTableSort('name')" title="Identificador do modelo, fabricante e capacidades multimodais.&#10;Clique no botão de prancheta para copiar o ID canônico.">
                  Modelo / Identificador <span class="sort-icon" id="sort-icon-name">↕</span>
                </th>

                <!-- COLUNA 3: STATUS PARETO -->
                <th class="text-center th-col-status th-guide" style="width: 125px;" title="Status de Otimização na Fronteira de Pareto.&#10;👑 Líder Qualidade: Ponto ótimo não-dominado.&#10;💎 Melhor ROI: Maior razão inteligência por dólar.&#10;⚡ Ultra Econômico: Menor tarifa com alta velocidade.&#10;🔹 Homologado: Modelo homologado complementar.">
                  Status
                </th>

                <!-- COLUNA 4: SCORE GERAL -->
                <th class="text-center sortable-th active-sort th-col-score th-guide" style="width: 90px;" onclick="setTableSort('score')" title="Score Multiobjetivo Geral Ponderado (0 a 100).&#10;📈 Como Interpretar: Quanto MAIOR, melhor.&#10;Cálculo: Combinação de benchmarks ponderados pelas prioridades do perfil cognitivo ativo.">
                  Score Geral <span class="sort-icon" id="sort-icon-score">▼</span>
                </th>

                <!-- COLUNA 5: ROI INDEX -->
                <th class="text-center sortable-th th-col-roi th-guide" style="width: 90px;" onclick="setTableSort('roi')" title="Índice de Retorno sobre Investimento (Score ÷ Blended Cost).&#10;📈 Como Interpretar: Quanto MAIOR, melhor.&#10;Cálculo: Mede o volume de capacidade cognitiva entregue por cada dólar investido.">
                  ROI Index <span class="sort-icon" id="sort-icon-roi">↕</span>
                </th>

                <!-- COLUNA 6: TARIFA NOUS PORTAL -->
                <th class="text-center sortable-th th-col-nous th-guide" style="width: 115px;" onclick="setTableSort('nous_cost')" title="Tarifa da API do Nous Portal por 1M de tokens (Blended: 25% Entrada + 75% Saída).&#10;📉 Como Interpretar: Quanto MENOR, melhor.&#10;★ Indica menor tarifa entre provedores homologados. Consome créditos do Nous.">
                  Tarifa Nous ($/1M) <span class="sort-icon" id="sort-icon-nous_cost">↕</span>
                </th>

                <!-- COLUNA 7: TARIFA OPENROUTER -->
                <th class="text-center sortable-th th-col-or th-guide" style="width: 115px;" onclick="setTableSort('or_cost')" title="Tarifa da API do OpenRouter por 1M de tokens (Blended: 25% Entrada + 75% Saída).&#10;📉 Como Interpretar: Quanto MENOR, melhor.&#10;★ Indica menor tarifa entre provedores homologados.">
                  Tarifa OR ($/1M) <span class="sort-icon" id="sort-icon-or_cost">↕</span>
                </th>

                <!-- COLUNA 8: INTELIGÊNCIA (ARENA ELO) -->
                <th class="text-center sortable-th th-col-intel th-guide" style="width: 95px;" onclick="setTableSort('intel')" title="Inteligência Geral e Discernimento Cognitivo (LMSYS Arena ELO / Hard Prompts).&#10;📈 Como Interpretar: Quanto MAIOR, melhor.&#10;Avalia capacidade geral de síntese, julgamento e resolução de problemas abertos.">
                  Inteligência <span class="sort-icon" id="sort-icon-intel">↕</span>
                </th>

                <!-- COLUNA 9: CODING (SWE-BENCH) -->
                <th class="text-center sortable-th th-col-coding th-guide" style="width: 85px;" onclick="setTableSort('coding')" title="Capacidade em Engenharia de Software e Programação (SWE-bench / Code ELO).&#10;📈 Como Interpretar: Quanto MAIOR, melhor.&#10;Mede resolução real de bugs em repositórios e refatoração de código.">
                  Coding <span class="sort-icon" id="sort-icon-coding">↕</span>
                </th>

                <!-- COLUNA 10: TOOLS (BFCL) -->
                <th class="text-center sortable-th th-col-bfcl th-guide" style="width: 85px;" onclick="setTableSort('bfcl')" title="Berkeley Function Calling Leaderboard (Chamada de Ferramentas e MCP).&#10;📈 Como Interpretar: Quanto MAIOR, melhor.&#10;Mede precisão sintática e semântica no disparo de tools.">
                  Tools (BFCL) <span class="sort-icon" id="sort-icon-bfcl">↕</span>
                </th>

                <!-- COLUNA 11: REASONING (GPQA) -->
                <th class="text-center sortable-th th-col-reasoning th-guide" style="width: 90px;" onclick="setTableSort('reasoning')" title="Raciocínio Lógico e Científico (GPQA Diamond / Hard Math).&#10;📈 Como Interpretar: Quanto MAIOR, melhor.&#10;Avalia deduções complexas e cadeias lógicas profundas.">
                  Reasoning <span class="sort-icon" id="sort-icon-reasoning">↕</span>
                </th>

                <!-- COLUNA 12: DESIGN / UI/UX -->
                <th class="text-center sortable-th th-col-design th-guide" style="width: 90px;" onclick="setTableSort('design')" title="Capacidade de Frontend, Design UI/UX e Estética de Código.&#10;📈 Como Interpretar: Quanto MAIOR, melhor.&#10;Mede refinamento visual, geração CSS/Tailwind, acessibilidade e componentes interativos.">
                  Design UI/UX <span class="sort-icon" id="sort-icon-design">↕</span>
                </th>

                <!-- COLUNA 13: VISÃO / VQA -->
                <th class="text-center sortable-th th-col-vision th-guide" style="width: 85px;" onclick="setTableSort('vision')" title="Visão Computacional e Entendimento Multimodal (MMMU / ChartQA / MathVista).&#10;📈 Como Interpretar: Quanto MAIOR, melhor.&#10;Avalia compreensão de prints de tela, diagramas técnicos e dados visuais.">
                  Visão (VQA) <span class="sort-icon" id="sort-icon-vision">↕</span>
                </th>

                <!-- COLUNA 14: INSTRUÇÃO (IFEVAL) -->
                <th class="text-center sortable-th th-col-ifeval th-guide" style="width: 85px;" onclick="setTableSort('ifeval')" title="Fidelidade a Instruções e Restrições de Formato (IFEval).&#10;📈 Como Interpretar: Quanto MAIOR, melhor.&#10;Mede cumprimento estrito de JSON schemas, delimitadores e contagens.">
                  Instrução <span class="sort-icon" id="sort-icon-ifeval">↕</span>
                </th>

                <!-- COLUNA 15: TTFT (LATÊNCIA) -->
                <th class="text-center sortable-th th-col-ttft th-guide" style="width: 85px;" onclick="setTableSort('ttft')" title="Time to First Token (Latência de Resposta Inicial em milissegundos).&#10;📉 Como Interpretar: Quanto MENOR, melhor.&#10;Impacta diretamente a reatividade do diálogo e início do streaming.">
                  TTFT <span class="sort-icon" id="sort-icon-ttft">↕</span>
                </th>

                <!-- COLUNA 16: TPS -->
                <th class="text-center sortable-th th-col-tps th-guide" style="width: 80px;" onclick="setTableSort('tps')" title="Velocidade média em tokens por segundo (Throughput).&#10;📈 Como Interpretar: Quanto MAIOR, melhor.&#10;Indica velocidade de streaming e geração em lote para subagentes.">
                  TPS <span class="sort-icon" id="sort-icon-tps">↕</span>
                </th>

                <!-- COLUNA 17: JANELA & RETENÇÃO -->
                <th class="text-center sortable-th th-col-ctx th-guide" style="width: 110px;" onclick="setTableSort('ctx')" title="Capacidade máxima de janela de contexto e retenção Needle-in-a-Haystack / RULER.&#10;📈 Como Interpretar: Quanto MAIOR, melhor.&#10;Exibe o tamanho máximo e a taxa de retenção de fatos no contexto.">
                  Janela & RULER <span class="sort-icon" id="sort-icon-ctx">↕</span>
                </th>

                <!-- COLUNA 18: DISPONIBILIDADE -->
                <th class="text-center th-col-prov th-guide" style="width: 110px;" title="Provedores com modelo indexado e pronto para consumo via API.&#10;🧠 Nous: Disponível com créditos do Nous Portal.&#10;🔀 OR: Disponível via API OpenRouter.">
                  Provedores
                </th>

                <!-- COLUNA 19: AÇÃO -->
                <th class="text-center" style="width: 75px;">Ação</th>
              </tr>
            </thead>
            <tbody id="cloud-ranked-tbody">
              <!-- Renderizado dinamicamente via fetchCloudRankedModels() -->
              <tr>
                <td colspan="19" style="text-align: center; color: var(--text-muted); padding: 24px;">Buscando modelos da Cloud Arena com sub-scores de benchmarks e tarifas comparadas...</td>
              </tr>
            </tbody>
          </table>"""

if old_table_header in html:
    html = html.replace(old_table_header, new_table_header)
    print("Updated table headers to include Intel, Design, Vision, TTFT")
else:
    print("WARNING: old_table_header not found")

with open('crates/souls_server/src/dashboard_ui.html', 'w', encoding='utf-8', newline='\n') as f:
    f.write(html)
print("Step 2 done")

