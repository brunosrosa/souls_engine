with open("crates/souls_server/src/dashboard_ui.html", "r", encoding="utf-8") as f:
    content = f.read()

# 1. Target the HTML of panel-local-arena
old_html_arena = """      <!-- PAINEL 3: LOCAL SILICON ARENA (BENCHMARKS BARE-METAL) -->
      <div id="panel-local-arena" style="display: none;">
        <!-- CARD 1: RUNNER DE BENCHMARK INTERATIVO -->
        <div class="card" style="margin-bottom: 20px;">
          <div class="card-header">
            <div>
              <div class="card-title">
                <div class="card-icon icon-emerald">🏟️</div>
                Local Silicon Arena — Aferição & Provas de Silício Bare-Metal (ADR-007)
              </div>
              <p style="font-size: 0.76rem; color: var(--text-muted); margin-top: 4px;">
                Execução de baterias de prova diretamente no hardware real (NVIDIA RTX 2060m CUDA & Intel i9 AVX2). Telemetria registrada no FrankenSQLite STRICT.
              </p>
            </div>
            <div style="display: flex; gap: 8px;">
              <button class="btn btn-emerald" onclick="syncLocalArenaHermes()">
                <span>🔗</span> Sincronizar HardLinks Hermes
              </button>
              <button class="btn btn-cyan" onclick="fetchLocalArenaData()">
                <span>🔄</span> Atualizar Benchmarks
              </button>
            </div>
          </div>

          <!-- CONTROLE DE DISPARO DE PROVA -->
          <div style="background: rgba(15, 23, 42, 0.6); border: 1px solid rgba(255, 255, 255, 0.08); border-radius: 12px; padding: 18px; margin-top: 14px;">
            <div style="display: grid; grid-template-columns: 2fr 2fr 1.5fr auto; gap: 14px; align-items: end;">
              <div>
                <label style="font-size: 0.74rem; color: var(--text-muted); font-weight: 600; display: block; margin-bottom: 6px;">SELECIONAR MODELO LOCAL</label>
                <select id="arena-runner-model-select" class="form-select" style="width: 100%; background: #0f172a; border: 1px solid rgba(255,255,255,0.15); border-radius: 8px; color: #fff; padding: 8px 12px; font-size: 0.8rem;">
                  <option value="">Carregando modelos instalados...</option>
                </select>
              </div>

              <div>
                <label style="font-size: 0.74rem; color: var(--text-muted); font-weight: 600; display: block; margin-bottom: 6px;">BATERIA DE PROVA CANÔNICA</label>
                <select id="arena-runner-type-select" class="form-select" style="width: 100%; background: #0f172a; border: 1px solid rgba(255,255,255,0.15); border-radius: 8px; color: #fff; padding: 8px 12px; font-size: 0.8rem;">
                  <option value="1">Type 1 — HardwareProfile (Velocidade TPS, VRAM Real, TTFT/TPOT)</option>
                  <option value="2">Type 2 — CognitiveEval (Rust AST Parsing, CoT Socrático)</option>
                  <option value="3">Type 3 — MultimodalVision (VLM + mmproj-F16.gguf Pareamento)</option>
                  <option value="4">Type 4 — SpeculativeDraft (Taxa Alpha & Speedup de Rascunho)</option>
                  <option value="5">Type 5 — ContextPressure (Needle-in-a-Haystack 4k a 32k)</option>
                </select>
              </div>

              <div>
                <label style="font-size: 0.74rem; color: var(--text-muted); font-weight: 600; display: block; margin-bottom: 6px;">MOTOR DE SILÍCIO</label>
                <select id="arena-runner-engine-select" class="form-select" style="width: 100%; background: #0f172a; border: 1px solid rgba(255,255,255,0.15); border-radius: 8px; color: #fff; padding: 8px 12px; font-size: 0.8rem;">
                  <option value="">Auto (CUDA para dGPU / AVX2 para CPU)</option>
                  <option value="llama-cuda">LlamaCpp CUDA (NVIDIA RTX 2060m)</option>
                  <option value="llama-cpu">LlamaCpp CPU (Intel Core i9 AVX2)</option>
                  <option value="onnx-cpu">ONNX Runtime CPU (Zero VRAM)</option>
                </select>
              </div>

              <div>
                <button class="btn btn-emerald" id="btn-run-arena-benchmark" onclick="executeSelectedArenaBenchmark()" style="height: 38px; padding: 0 20px; white-space: nowrap; font-weight: 600;">
                  <span>⚡</span> Rodar Prova no Silício
                </button>
              </div>
            </div>

            <!-- RESULTADO EM TEMPO REAL DO TESTE -->
            <div id="arena-live-feedback" style="display: none; margin-top: 16px; padding: 14px; border-radius: 10px; background: rgba(16, 185, 129, 0.08); border: 1px solid rgba(16, 185, 129, 0.25);">
              <div style="display: flex; justify-content: space-between; align-items: center;">
                <div style="font-size: 0.82rem; font-weight: 600; color: #fff;" id="arena-live-title">
                  Resultado do Benchmark
                </div>
                <div id="arena-live-verdict-badge"></div>
              </div>
              <div id="arena-live-metrics-grid" style="display: grid; grid-template-columns: repeat(auto-fit, minmax(130px, 1fr)); gap: 10px; margin-top: 10px;"></div>
            </div>
          </div>
        </div>

        <!-- CARD 2: TABELA DE MODELOS & TELEMETRIA REAL NO SILÍCIO -->
        <div class="card" style="margin-bottom: 20px;">
          <div class="card-header">
            <div>
              <div class="card-title">
                <div class="card-icon icon-cyan">📊</div>
                Inventário Homologado no Silício & Métricas Empíricas
              </div>
              <p style="font-size: 0.76rem; color: var(--text-muted); margin-top: 4px;">
                Valores reais obtidos por telemetria NVML e cronometragem bare-metal.
              </p>
            </div>
            <div id="arena-models-summary-badge" class="badge badge-champion">14 Modelos Mapeados</div>
          </div>

          <div class="table-container">
            <table class="arena-table">
              <thead>
                <tr>
                  <th style="width: 80px;" class="text-center">Tier</th>
                  <th>Modelo & Identificador</th>
                  <th style="width: 100px;">Quantização</th>
                  <th style="width: 120px;" class="text-center">Throughput (TPS)</th>
                  <th style="width: 100px;" class="text-center">TTFT Prefill</th>
                  <th style="width: 120px;" class="text-center">VRAM Pico (NVML)</th>
                  <th style="width: 90px;" class="text-center">E³ Score</th>
                  <th style="width: 140px;" class="text-center">Status de Prova</th>
                  <th style="width: 130px;" class="text-center">Ação Rápida</th>
                </tr>
              </thead>
              <tbody id="arena-models-tbody">
                <tr>
                  <td colspan="9" style="text-align: center; color: var(--text-muted);">Carregando modelos da Arena...</td>
                </tr>
              </tbody>
            </table>
          </div>
        </div>

        <!-- CARD 3: HISTÓRICO DE AUDITORIA DE TELEMETRIA (arena_telemetry) -->
        <div class="card">
          <div class="card-header">
            <div>
              <div class="card-title">
                <div class="card-icon icon-purple">📜</div>
                Histórico Recente de Ensaios no Silício (Heuristic Vault)
              </div>
              <p style="font-size: 0.76rem; color: var(--text-muted); margin-top: 4px;">
                Últimas 50 execuções registradas na tabela STRICT <code class="mono">arena_telemetry</code>.
              </p>
            </div>
          </div>

          <div class="table-container">
            <table class="arena-table">
              <thead>
                <tr>
                  <th style="width: 50px;" class="text-center">ID</th>
                  <th>Modelo</th>
                  <th style="width: 130px;">Bateria</th>
                  <th style="width: 110px;" class="text-center">Motor</th>
                  <th style="width: 90px;" class="text-center">TPS</th>
                  <th style="width: 90px;" class="text-center">TTFT</th>
                  <th style="width: 110px;" class="text-center">VRAM Pico</th>
                  <th style="width: 100px;" class="text-center">Status</th>
                  <th style="width: 140px;">Data / Hora</th>
                </tr>
              </thead>
              <tbody id="arena-telemetry-tbody">
                <tr>
                  <td colspan="9" style="text-align: center; color: var(--text-muted);">Nenhum histórico registrado no vault ainda.</td>
                </tr>
              </tbody>
            </table>
          </div>
        </div>
      </div>"""

new_html_arena = """      <!-- PAINEL 3: LOCAL SILICON ARENA (BENCHMARKS BARE-METAL) -->
      <div id="panel-local-arena" style="display: none;">
        <!-- CARD 1: RUNNER DE BENCHMARK INTERATIVO -->
        <div class="card" style="margin-bottom: 20px;">
          <div class="card-header" style="flex-wrap: wrap; gap: 12px; align-items: center; justify-content: space-between;">
            <div>
              <div class="card-title">
                <div class="card-icon icon-emerald">🏟️</div>
                Arena de Testes de Silício (Benchmarks Locais)
              </div>
              <p style="font-size: 0.78rem; color: var(--text-muted); margin-top: 4px;">
                Meça a velocidade real (tokens/s), tempo de resposta (TTFT) e consumo de VRAM diretamente na sua placa NVIDIA RTX 2060m e CPU Intel i9.
              </p>
            </div>
            <div style="display: flex; gap: 8px; flex-wrap: wrap;">
              <button class="btn btn-emerald" onclick="syncLocalArenaHermes()" title="Cria links diretos e atualiza o catálogo do orquestrador Hermes">
                <span>🔗</span> Sincronizar com Hermes Agent
              </button>
              <button class="btn btn-cyan" onclick="fetchLocalArenaData()" title="Recarregar telemetria e dados medidos">
                <span>🔄</span> Atualizar Dados
              </button>
            </div>
          </div>

          <!-- CONTROLE DE DISPARO DE PROVA -->
          <div style="background: rgba(15, 23, 42, 0.6); border: 1px solid rgba(255, 255, 255, 0.08); border-radius: 12px; padding: 18px; margin-top: 14px;">
            <div style="display: grid; grid-template-columns: 2fr 2fr 1.5fr auto; gap: 14px; align-items: end;">
              <div>
                <label style="font-size: 0.74rem; color: var(--text-muted); font-weight: 600; display: block; margin-bottom: 6px;">MODELO LOCAL PARA TESTE</label>
                <select id="arena-runner-model-select" class="form-select" style="width: 100%; background: #0f172a; border: 1px solid rgba(255,255,255,0.15); border-radius: 8px; color: #fff; padding: 8px 12px; font-size: 0.8rem;">
                  <option value="">Carregando modelos instalados...</option>
                </select>
              </div>

              <div>
                <label style="font-size: 0.74rem; color: var(--text-muted); font-weight: 600; display: block; margin-bottom: 6px;">TIPO DE TESTE / PROVA</label>
                <select id="arena-runner-type-select" class="form-select" style="width: 100%; background: #0f172a; border: 1px solid rgba(255,255,255,0.15); border-radius: 8px; color: #fff; padding: 8px 12px; font-size: 0.8rem;">
                  <option value="1">⚡ Velocidade & Memória (Throughput tok/s, VRAM e TTFT)</option>
                  <option value="2">🧠 Avaliação Cognitiva (Raciocínio CoT & Parsing Rust AST)</option>
                  <option value="3">👁️ Visão Multimodal (Pareamento de Projetor VLM mmproj)</option>
                  <option value="4">🚀 Aceleração Especulativa (Drafting & Ganho de Velocidade)</option>
                  <option value="5">📦 Pressão de Contexto (Janela Longa de 4k até 32k tokens)</option>
                </select>
              </div>

              <div>
                <label style="font-size: 0.74rem; color: var(--text-muted); font-weight: 600; display: block; margin-bottom: 6px;">ACELERADOR DE SILÍCIO</label>
                <select id="arena-runner-engine-select" class="form-select" style="width: 100%; background: #0f172a; border: 1px solid rgba(255,255,255,0.15); border-radius: 8px; color: #fff; padding: 8px 12px; font-size: 0.8rem;">
                  <option value="">Auto (CUDA na GPU / AVX2 na CPU)</option>
                  <option value="llama-cuda">NVIDIA CUDA (Placa de Vídeo RTX 2060m)</option>
                  <option value="llama-cpu">CPU Intel AVX2 (Memória RAM do Computador)</option>
                  <option value="onnx-cpu">ONNX Runtime CPU (Zero Consumo de VRAM)</option>
                </select>
              </div>

              <div>
                <button class="btn btn-emerald" id="btn-run-arena-benchmark" onclick="executeSelectedArenaBenchmark()" style="height: 38px; padding: 0 20px; white-space: nowrap; font-weight: 600;">
                  <span>⚡</span> Iniciar Teste no Silício
                </button>
              </div>
            </div>

            <!-- RESULTADO EM TEMPO REAL DO TESTE -->
            <div id="arena-live-feedback" style="display: none; margin-top: 16px; padding: 14px; border-radius: 10px; background: rgba(16, 185, 129, 0.08); border: 1px solid rgba(16, 185, 129, 0.25);">
              <div style="display: flex; justify-content: space-between; align-items: center; flex-wrap: wrap; gap: 8px;">
                <div style="font-size: 0.84rem; font-weight: 600; color: #fff;" id="arena-live-title">
                  Resultado do Benchmark
                </div>
                <div id="arena-live-verdict-badge"></div>
              </div>
              <div id="arena-live-metrics-grid" style="display: grid; grid-template-columns: repeat(auto-fit, minmax(130px, 1fr)); gap: 10px; margin-top: 12px;"></div>
            </div>
          </div>
        </div>

        <!-- CARD 2: TABELA DE MODELOS & TELEMETRIA REAL NO SILÍCIO -->
        <div class="card" style="margin-bottom: 20px;">
          <div class="card-header" style="flex-wrap: wrap; gap: 12px; align-items: center; justify-content: space-between;">
            <div>
              <div class="card-title">
                <div class="card-icon icon-cyan">📊</div>
                Modelos Homologados & Resultados Medidos
              </div>
              <p style="font-size: 0.78rem; color: var(--text-muted); margin-top: 4px;">
                Velocidade e consumo medidos no seu computador real. Modelos aprovados aqui alimentam automaticamente o Hub e o Roteador de Subagentes.
              </p>
            </div>
            <div id="arena-models-summary-badge" class="badge badge-champion">14 Modelos Mapeados</div>
          </div>

          <!-- BARRA DE FILTROS & BUSCA RÁPIDA NA ARENA -->
          <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 14px; flex-wrap: wrap; gap: 10px; border-bottom: 1px solid var(--border-subtle); padding-bottom: 12px;">
            <div class="profile-selector" id="arena-filter-pills" style="margin-bottom: 0;">
              <button class="profile-pill active" id="btn-arena-filter-all" onclick="filterArenaModels('all')">Todos os Modelos</button>
              <button class="profile-pill" id="btn-arena-filter-benchmarked" onclick="filterArenaModels('benchmarked')">✅ Homologados</button>
              <button class="profile-pill" id="btn-arena-filter-pending" onclick="filterArenaModels('pending')">⏳ Pendentes</button>
              <button class="profile-pill" id="btn-arena-filter-tier0" onclick="filterArenaModels('tier0')">⚡ Tier 0 / 0.5 (Triagem)</button>
              <button class="profile-pill" id="btn-arena-filter-tier1" onclick="filterArenaModels('tier1')">💻 Tier 1 (Programação)</button>
            </div>

            <div style="display: flex; align-items: center; gap: 8px;">
              <input type="text" id="arena-models-search" placeholder="🔍 Filtrar modelo..." class="setting-input" style="padding: 5px 12px; font-size: 0.75rem; width: 200px; border-radius: 6px;" oninput="onArenaModelsSearchInput()">
            </div>
          </div>

          <div class="table-container">
            <table class="arena-table">
              <thead>
                <tr>
                  <th style="width: 80px;" class="text-center">Tier</th>
                  <th>Modelo & Identificador</th>
                  <th style="width: 100px;">Quantização</th>
                  <th style="width: 120px;" class="text-center">Throughput (TPS)</th>
                  <th style="width: 100px;" class="text-center">TTFT Prefill</th>
                  <th style="width: 120px;" class="text-center">VRAM Pico (NVML)</th>
                  <th style="width: 90px;" class="text-center" title="Eficiência E³: Throughput / (VRAM × TTFT)">E³ Score</th>
                  <th style="width: 130px;" class="text-center">Status</th>
                  <th style="width: 160px;" class="text-center">Ações</th>
                </tr>
              </thead>
              <tbody id="arena-models-tbody">
                <tr>
                  <td colspan="9" style="text-align: center; color: var(--text-muted);">Carregando modelos da Arena...</td>
                </tr>
              </tbody>
            </table>
          </div>
        </div>

        <!-- CARD 3: HISTÓRICO DE AUDITORIA DE TELEMETRIA (arena_telemetry) -->
        <div class="card">
          <div class="card-header" style="flex-wrap: wrap; gap: 12px; align-items: center; justify-content: space-between;">
            <div>
              <div class="card-title">
                <div class="card-icon icon-purple">📜</div>
                Histórico de Testes Realizados no Silício
              </div>
              <p style="font-size: 0.78rem; color: var(--text-muted); margin-top: 4px;">
                Registro detalhado dos testes de desempenho executados na sua máquina com telemetria oficial de hardware.
              </p>
            </div>
            <div id="arena-telemetry-summary-badge" class="badge badge-lowcost">Histórico de Provas</div>
          </div>

          <div class="table-container">
            <table class="arena-table">
              <thead>
                <tr>
                  <th style="width: 60px;" class="text-center">ID</th>
                  <th>Modelo</th>
                  <th style="width: 150px;">Tipo de Prova</th>
                  <th style="width: 110px;" class="text-center">Acelerador</th>
                  <th style="width: 100px;" class="text-center">Velocidade</th>
                  <th style="width: 90px;" class="text-center">TTFT</th>
                  <th style="width: 110px;" class="text-center">VRAM Pico</th>
                  <th style="width: 100px;" class="text-center">Status</th>
                  <th style="width: 140px;">Data / Hora</th>
                </tr>
              </thead>
              <tbody id="arena-telemetry-tbody">
                <tr>
                  <td colspan="9" style="text-align: center; color: var(--text-muted);">Nenhum histórico registrado no vault ainda.</td>
                </tr>
              </tbody>
            </table>
          </div>
        </div>
      </div>"""

assert old_html_arena in content, "old_html_arena not found in dashboard_ui.html"
content = content.replace(old_html_arena, new_html_arena, 1)
print("Arena HTML successfully updated!")

with open("crates/souls_server/src/dashboard_ui.html", "w", encoding="utf-8") as f:
    f.write(content)

