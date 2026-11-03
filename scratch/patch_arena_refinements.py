import re

with open("crates/souls_server/src/dashboard_ui.html", "r", encoding="utf-8") as f:
    content = f.read()

# 1. Update the Runner HTML (Ponto #1) and Filter Pills + Headers (Pontos #2, #6, #7)
old_runner_and_table_header = """          <!-- CONTROLE DE DISPARO DE PROVA -->
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
              </thead>"""

new_runner_and_table_header = """          <!-- CONTROLE DE DISPARO DE PROVA -->
          <div style="background: rgba(15, 23, 42, 0.6); border: 1px solid rgba(255, 255, 255, 0.08); border-radius: 12px; padding: 18px; margin-top: 14px; display: flex; flex-direction: column; gap: 14px;">
            <!-- LINHA 1: MODELO LOCAL (AMPLO) + BOTÃO INICIAR TESTE -->
            <div style="display: grid; grid-template-columns: 1fr auto; gap: 14px; align-items: end;">
              <div>
                <label style="font-size: 0.74rem; color: var(--text-muted); font-weight: 600; display: block; margin-bottom: 6px;">MODELO LOCAL PARA TESTE</label>
                <select id="arena-runner-model-select" class="form-select" style="width: 100%; background: #0f172a; border: 1px solid rgba(255,255,255,0.15); border-radius: 8px; color: #fff; padding: 9px 14px; font-size: 0.82rem;">
                  <option value="">Carregando modelos instalados...</option>
                </select>
              </div>

              <div>
                <button class="btn btn-emerald" id="btn-run-arena-benchmark" onclick="executeSelectedArenaBenchmark()" style="height: 40px; padding: 0 24px; white-space: nowrap; font-weight: 600; font-size: 0.82rem;">
                  <span>⚡</span> Iniciar Teste
                </button>
              </div>
            </div>

            <!-- LINHA 2: TIPO DE TESTE / PROVA (ESPAÇOSO) + ACELERADOR DE SILÍCIO -->
            <div style="display: grid; grid-template-columns: 1.6fr 1fr; gap: 14px; align-items: end;">
              <div>
                <label style="font-size: 0.74rem; color: var(--text-muted); font-weight: 600; display: block; margin-bottom: 6px;">TIPO DE TESTE / PROVA</label>
                <select id="arena-runner-type-select" class="form-select" style="width: 100%; background: #0f172a; border: 1px solid rgba(255,255,255,0.15); border-radius: 8px; color: #fff; padding: 9px 14px; font-size: 0.82rem;">
                  <option value="1">⚡ Velocidade & Memória (Throughput tok/s, VRAM e TTFT)</option>
                  <option value="2">🧠 Avaliação Cognitiva (Raciocínio CoT & Parsing Rust AST)</option>
                  <option value="3">👁️ Visão Multimodal (Pareamento de Projetor VLM mmproj)</option>
                  <option value="4">🚀 Aceleração Especulativa (Drafting & Ganho de Velocidade)</option>
                  <option value="5">📦 Pressão de Contexto (Janela Longa de 4k até 32k tokens)</option>
                </select>
              </div>

              <div>
                <label style="font-size: 0.74rem; color: var(--text-muted); font-weight: 600; display: block; margin-bottom: 6px;">ACELERADOR DE SILÍCIO</label>
                <select id="arena-runner-engine-select" class="form-select" style="width: 100%; background: #0f172a; border: 1px solid rgba(255,255,255,0.15); border-radius: 8px; color: #fff; padding: 9px 14px; font-size: 0.82rem;">
                  <option value="">Auto (CUDA na GPU / AVX2 na CPU)</option>
                  <option value="llama-cuda">NVIDIA CUDA (Placa de Vídeo RTX 2060m)</option>
                  <option value="llama-cpu">CPU Intel AVX2 (Memória RAM do Computador)</option>
                  <option value="onnx-cpu">ONNX Runtime CPU (Zero Consumo de VRAM)</option>
                </select>
              </div>
            </div>

            <!-- RESULTADO EM TEMPO REAL DO TESTE -->
            <div id="arena-live-feedback" style="display: none; margin-top: 10px; padding: 14px; border-radius: 10px; background: rgba(16, 185, 129, 0.08); border: 1px solid rgba(16, 185, 129, 0.25);">
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
              <button class="profile-pill" id="btn-arena-filter-tier0" onclick="filterArenaModels('tier0')">Tier 0</button>
              <button class="profile-pill" id="btn-arena-filter-tier0_5" onclick="filterArenaModels('tier0_5')">Tier 0.5</button>
              <button class="profile-pill" id="btn-arena-filter-tier1" onclick="filterArenaModels('tier1')">Tier 1</button>
              <button class="profile-pill" id="btn-arena-filter-tier2" onclick="filterArenaModels('tier2')">Tier 2</button>
            </div>

            <div style="display: flex; align-items: center; gap: 8px;">
              <input type="text" id="arena-models-search" placeholder="🔍 Filtrar modelo..." class="setting-input" style="padding: 5px 12px; font-size: 0.75rem; width: 200px; border-radius: 6px;" oninput="onArenaModelsSearchInput()">
            </div>
          </div>

          <div class="table-container">
            <table class="arena-table">
              <thead>
                <tr>
                  <th style="width: 80px; cursor: pointer; user-select: none;" class="text-center" onclick="sortArenaTable('tier')" title="Clique para ordenar por Tier">
                    Tier <span id="sort-ind-tier" style="font-size: 0.65rem; color: var(--cyan);">▲</span>
                  </th>
                  <th style="cursor: pointer; user-select: none;" onclick="sortArenaTable('name')" title="Clique para ordenar por Modelo">
                    Modelo & Identificador <span id="sort-ind-name" style="font-size: 0.65rem; color: var(--text-muted);"></span>
                  </th>
                  <th style="width: 100px; cursor: pointer; user-select: none;" class="text-center" onclick="sortArenaTable('quant')" title="Clique para ordenar por Quantização">
                    Quantização <span id="sort-ind-quant" style="font-size: 0.65rem; color: var(--text-muted);"></span>
                  </th>
                  <th style="width: 130px; cursor: pointer; user-select: none;" class="text-center" onclick="sortArenaTable('tps')" title="Throughput: Velocidade de geração em tokens por segundo após o primeiro token. Clique para ordenar.">
                    Throughput (TPS) <span style="cursor: help; color: var(--cyan); font-size: 0.72rem; margin-left: 2px;" title="Throughput: Velocidade de geração em tokens por segundo após o primeiro token.">ⓘ</span> <span id="sort-ind-tps" style="font-size: 0.65rem; color: var(--text-muted);"></span>
                  </th>
                  <th style="width: 110px; cursor: pointer; user-select: none;" class="text-center" onclick="sortArenaTable('ttft')" title="Time To First Token: Tempo até o primeiro token em milissegundos. Mede a agilidade de resposta inicial. Clique para ordenar.">
                    TTFT Prefill <span style="cursor: help; color: var(--cyan); font-size: 0.72rem; margin-left: 2px;" title="Time To First Token: Tempo até o primeiro token em milissegundos. Mede a agilidade de resposta inicial.">ⓘ</span> <span id="sort-ind-ttft" style="font-size: 0.65rem; color: var(--text-muted);"></span>
                  </th>
                  <th style="width: 120px; cursor: pointer; user-select: none;" class="text-center" onclick="sortArenaTable('vram')" title="VRAM Pico: Consumo máximo real de memória da GPU medido pelo sensor NVML durante a execução. Clique para ordenar.">
                    VRAM Pico <span style="cursor: help; color: var(--cyan); font-size: 0.72rem; margin-left: 2px;" title="Consumo máximo real de memória da GPU medido pelo sensor NVML durante a execução.">ⓘ</span> <span id="sort-ind-vram" style="font-size: 0.65rem; color: var(--text-muted);"></span>
                  </th>
                  <th style="width: 100px; cursor: pointer; user-select: none;" class="text-center" onclick="sortArenaTable('e3')" title="Eficiência Empírica E³: Throughput / (VRAM_GB × TTFT_s). Mede quanto desempenho o modelo entrega por cada GB de VRAM e milissegundo de latência. Quanto maior, mais eficiente! Clique para ordenar.">
                    E³ Score <span style="cursor: help; color: #c084fc; font-size: 0.75rem; margin-left: 2px; font-weight: bold;" title="Eficiência Empírica E³: Throughput / (VRAM_GB × TTFT_s). Mede quanto desempenho o modelo entrega por cada GB de VRAM e milissegundo de latência. Quanto maior, mais eficiente!">ⓘ</span> <span id="sort-ind-e3" style="font-size: 0.65rem; color: var(--text-muted);"></span>
                  </th>
                  <th style="width: 130px; cursor: pointer; user-select: none;" class="text-center" onclick="sortArenaTable('status')" title="Clique para ordenar por Status de Homologação">
                    Status <span id="sort-ind-status" style="font-size: 0.65rem; color: var(--text-muted);"></span>
                  </th>
                  <th style="width: 160px;" class="text-center">Ações</th>
                </tr>
              </thead>"""

assert old_runner_and_table_header in content, "old_runner_and_table_header not found in dashboard_ui.html"
content = content.replace(old_runner_and_table_header, new_runner_and_table_header, 1)
print("HTML for runner and table header successfully replaced!")

with open("crates/souls_server/src/dashboard_ui.html", "w", encoding="utf-8") as f:
    f.write(content)

