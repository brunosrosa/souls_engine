with open("crates/souls_server/src/dashboard_ui.html", "r", encoding="utf-8") as f:
    content = f.read()

start_marker = "    // 3.2 LOCAL SILICON ARENA (ADR-007)"
end_marker = "    // ==========================================\n    // 3.3 TWO-STAGE SEMANTIC SWITCHBOARD (<1.7ms)"

assert start_marker in content, "start_marker not found"
assert end_marker in content, "end_marker not found"

start_idx = content.find(start_marker)
end_idx = content.find(end_marker)

new_arena_js = """    // 3.2 LOCAL SILICON ARENA (ADR-007)
    // ==========================================
    let arenaModelsData = [];
    let arenaTelemetryData = [];
    let currentArenaModelsFilter = 'all';
    let currentArenaSearchQuery = '';
    let currentArenaSortKey = 'tier';
    let currentArenaSortAsc = true;

    async function fetchLocalArenaData() {
      try {
        const [resModels, resBench] = await Promise.all([
          fetch('/api/v1/arena/models'),
          fetch('/api/v1/arena/benchmarks')
        ]);

        if (resModels.ok) {
          const dataModels = await resModels.json();
          arenaModelsData = dataModels.models || [];
          renderArenaModelsTable(arenaModelsData);
          populateArenaRunnerModelSelect(arenaModelsData);

          const benchmarkedCount = arenaModelsData.filter(m => m.status === 'BENCHMARKED').length;
          const badge = document.getElementById('badge-subtab-arena-benchmarks');
          if (badge) {
            badge.innerText = benchmarkedCount;
            badge.style.display = benchmarkedCount > 0 ? 'inline-block' : 'none';
          }
          const sumBadge = document.getElementById('arena-models-summary-badge');
          if (sumBadge) {
            sumBadge.innerText = `${arenaModelsData.length} Modelos Mapeados • ${benchmarkedCount} Homologados`;
          }
        }

        if (resBench.ok) {
          const dataBench = await resBench.json();
          arenaTelemetryData = dataBench.history || [];
          renderArenaTelemetryTable(arenaTelemetryData);
        }
      } catch (err) {
        console.error('Falha ao carregar dados da Arena Local:', err);
      }
    }

    function populateArenaRunnerModelSelect(models) {
      const select = document.getElementById('arena-runner-model-select');
      if (!select) return;
      if (!models || models.length === 0) {
        select.innerHTML = '<option value="">Nenhum modelo disponível</option>';
        return;
      }
      select.innerHTML = models.map(m => {
        const isBenchmarked = m.status === 'BENCHMARKED';
        const prefix = isBenchmarked ? '✅' : '⏳';
        const tpsStr = m.latest_tps ? ` • ${m.latest_tps.toFixed(0)} tok/s` : '';
        const paramsStr = m.parameters ? ` • ${m.parameters}` : '';
        return `<option value="${m.model_id}">${prefix} ${m.name} (${m.tier}${paramsStr} • ${m.quantization || 'GGUF'}${tpsStr})</option>`;
      }).join('');
    }

    function filterArenaModels(filter) {
      currentArenaModelsFilter = filter;
      document.querySelectorAll('#arena-filter-pills .profile-pill').forEach(b => b.classList.remove('active'));
      const activeBtn = document.getElementById('btn-arena-filter-' + filter.replace('.', '_'));
      if (activeBtn) activeBtn.classList.add('active');
      renderArenaModelsTable(arenaModelsData);
    }

    function onArenaModelsSearchInput() {
      const input = document.getElementById('arena-models-search');
      currentArenaSearchQuery = input ? input.value.trim().toLowerCase() : '';
      renderArenaModelsTable(arenaModelsData);
    }

    function sortArenaTable(key) {
      if (currentArenaSortKey === key) {
        currentArenaSortAsc = !currentArenaSortAsc;
      } else {
        currentArenaSortKey = key;
        currentArenaSortAsc = true;
      }
      updateArenaSortIndicators();
      renderArenaModelsTable(arenaModelsData);
    }

    function updateArenaSortIndicators() {
      const keys = ['tier', 'name', 'quant', 'tps', 'ttft', 'vram', 'e3', 'status'];
      keys.forEach(k => {
        const el = document.getElementById('sort-ind-' + k);
        if (el) {
          if (k === currentArenaSortKey) {
            el.innerText = currentArenaSortAsc ? '▲' : '▼';
            el.style.color = 'var(--cyan)';
          } else {
            el.innerText = '';
          }
        }
      });
    }

    function renderArenaModelsTable(models) {
      const tbody = document.getElementById('arena-models-tbody');
      if (!tbody) return;

      const filtered = (models || []).filter(m => {
        if (currentArenaModelsFilter === 'benchmarked' && m.status !== 'BENCHMARKED') return false;
        if (currentArenaModelsFilter === 'pending' && m.status === 'BENCHMARKED') return false;
        
        const tierLower = (m.tier || '').toLowerCase();
        if (currentArenaModelsFilter === 'tier0' && !(tierLower === 'tier 0' || (tierLower.includes('tier 0') && !tierLower.includes('0.5')))) return false;
        if (currentArenaModelsFilter === 'tier0_5' && !(tierLower.includes('tier 0.5') || tierLower.includes('tier 05'))) return false;
        if (currentArenaModelsFilter === 'tier1' && !(tierLower.includes('tier 1'))) return false;
        if (currentArenaModelsFilter === 'tier2' && !(tierLower.includes('tier 2'))) return false;

        if (currentArenaSearchQuery) {
          const q = currentArenaSearchQuery;
          const nameMatch = (m.name || '').toLowerCase().includes(q);
          const idMatch = (m.model_id || '').toLowerCase().includes(q);
          const quantMatch = (m.quantization || '').toLowerCase().includes(q);
          if (!nameMatch && !idMatch && !quantMatch) return false;
        }
        return true;
      });

      // Ordenação dinâmica
      filtered.sort((a, b) => {
        let cmp = 0;
        if (currentArenaSortKey === 'tier') {
          cmp = getTierRank(a.tier) - getTierRank(b.tier);
          if (cmp === 0) cmp = (a.name || a.model_id || '').localeCompare(b.name || b.model_id || '');
        } else if (currentArenaSortKey === 'name') {
          cmp = (a.name || a.model_id || '').localeCompare(b.name || b.model_id || '');
        } else if (currentArenaSortKey === 'quant') {
          cmp = (a.quantization || '').localeCompare(b.quantization || '');
        } else if (currentArenaSortKey === 'tps') {
          cmp = (a.latest_tps || 0) - (b.latest_tps || 0);
        } else if (currentArenaSortKey === 'ttft') {
          cmp = (a.latest_ttft_ms || 99999) - (b.latest_ttft_ms || 99999);
        } else if (currentArenaSortKey === 'vram') {
          cmp = (a.latest_vram_peak_mb || 0) - (b.latest_vram_peak_mb || 0);
        } else if (currentArenaSortKey === 'e3') {
          cmp = (a.e3_score || 0) - (b.e3_score || 0);
        } else if (currentArenaSortKey === 'status') {
          cmp = (a.status || '').localeCompare(b.status || '');
        }
        return currentArenaSortAsc ? cmp : -cmp;
      });

      if (filtered.length === 0) {
        tbody.innerHTML = '<tr><td colspan="9" style="text-align: center; color: var(--text-muted); padding: 20px;">Nenhum modelo corresponde aos filtros aplicados.</td></tr>';
        return;
      }

      tbody.innerHTML = filtered.map(m => {
        const isBenchmarked = m.status === 'BENCHMARKED';
        const statusBadge = isBenchmarked
          ? '<span class="badge" style="background: rgba(16, 185, 129, 0.15); color: var(--emerald); border: 1px solid rgba(16, 185, 129, 0.3);">✅ Homologado</span>'
          : '<span class="badge" style="background: rgba(234, 179, 8, 0.15); color: #facc15; border: 1px solid rgba(234, 179, 8, 0.3);">⏳ Pendente</span>';

        const tpsHtml = m.latest_tps
          ? `<div style="line-height: 1.2;"><strong style="color: var(--emerald); font-size: 0.88rem;">${m.latest_tps.toFixed(1)}</strong><div style="font-size: 0.65rem; color: var(--text-muted);">tok/s</div></div>`
          : '<span style="color: var(--text-muted);">--</span>';

        const ttftHtml = m.latest_ttft_ms
          ? `<div style="line-height: 1.2;"><span class="mono" style="color: var(--cyan); font-size: 0.82rem; font-weight: 600;">${m.latest_ttft_ms.toFixed(0)}</span><div style="font-size: 0.65rem; color: var(--text-muted);">ms</div></div>`
          : '<span style="color: var(--text-muted);">--</span>';

        const vramHtml = m.latest_vram_peak_mb
          ? `<div style="line-height: 1.2;"><span class="mono" style="color: #f43f5e; font-size: 0.82rem; font-weight: 600;">${m.latest_vram_peak_mb.toFixed(0)}</span><div style="font-size: 0.65rem; color: var(--text-muted);">MB</div></div>`
          : `<div style="line-height: 1.2;"><span class="mono" style="color: var(--text-muted); font-size: 0.75rem;">${m.size_mb ? (m.size_mb + 36).toFixed(0) : '--'}</span><div style="font-size: 0.65rem; color: var(--text-muted);">MB (est)</div></div>`;

        const e3Html = m.e3_score
          ? `<span class="badge" style="background: rgba(168, 85, 247, 0.15); color: #c084fc; border: 1px solid rgba(168, 85, 247, 0.3); font-size: 0.72rem; font-weight: 700;" title="Eficiência Empírica E³: ${m.e3_score.toFixed(1)} (Throughput / [VRAM × Latência])">${m.e3_score > 9000 ? '9999' : m.e3_score.toFixed(1)}</span>`
          : '<span style="color: var(--text-muted); font-size: 0.75rem;" title="Score não calculado ainda. Execute o teste Type 1 para aferir Throughput, TTFT e VRAM.">--</span>';

        const activeFile = m.file_path || m.model_id;

        return `
          <tr>
            <td class="text-center" style="vertical-align: middle;">
              ${renderTierBadge(m.tier)}
            </td>
            <td>
              <div style="font-weight: 600; color: #fff; font-size: 0.82rem;">${m.name}</div>
              <div class="mono" style="font-size: 0.68rem; color: var(--text-muted); margin-top: 2px;">${m.model_id}</div>
            </td>
            <td class="text-center mono" style="font-size: 0.78rem;">${m.quantization || 'GGUF'}</td>
            <td class="text-center">${tpsHtml}</td>
            <td class="text-center">${ttftHtml}</td>
            <td class="text-center">${vramHtml}</td>
            <td class="text-center">${e3Html}</td>
            <td class="text-center">${statusBadge}</td>
            <td class="text-center">
              <div style="display: flex; gap: 5px; justify-content: center; align-items: center;">
                <button class="btn btn-emerald" style="padding: 4px 10px; font-size: 0.72rem; height: 28px;" onclick="runSpecificArenaBenchmark('${m.model_id}', 1)" title="Executar Teste de Silício (Hardware Profile)">
                  <span>⚡</span> Testar
                </button>
                ${isBenchmarked ? `
                  <button class="btn btn-green" style="padding: 4px 10px; font-size: 0.72rem; height: 28px;" onclick="openQuickApplyModal('${activeFile}', 'local_silicon', true)" title="Definir no Hermes Agent">
                    <span>⚡</span> Usar
                  </button>
                ` : ''}
              </div>
            </td>
          </tr>
        `;
      }).join('');
    }

    function renderArenaTelemetryTable(history) {
      const tbody = document.getElementById('arena-telemetry-tbody');
      const badge = document.getElementById('arena-telemetry-summary-badge');
      if (badge && history) {
        badge.innerText = `${history.length} Provas Registradas`;
      }
      if (!tbody) return;
      if (!history || history.length === 0) {
        tbody.innerHTML = '<tr><td colspan="9" style="text-align: center; color: var(--text-muted); padding: 18px 0;">Nenhum ensaio registrado no SQLite Vault.</td></tr>';
        return;
      }

      tbody.innerHTML = history.slice(0, 30).map(h => {
        let statusBadge = '<span class="badge badge-champion" style="font-size: 0.65rem;">NOMINAL</span>';
        const st = (h.status || '').toUpperCase();
        if (st.includes('NOMINAL') || st.includes('SUCCESS') || st.includes('VERIFIED')) {
          statusBadge = '<span class="badge" style="background: rgba(16, 185, 129, 0.15); color: var(--emerald); border: 1px solid rgba(16, 185, 129, 0.3); font-size: 0.65rem;">NOMINAL</span>';
        } else if (st.includes('WARN') || st.includes('THROTTLE')) {
          statusBadge = '<span class="badge" style="background: rgba(234, 179, 8, 0.15); color: #facc15; border: 1px solid rgba(234, 179, 8, 0.3); font-size: 0.65rem;">THROTTLED</span>';
        } else if (st.includes('FAIL') || st.includes('ERR')) {
          statusBadge = '<span class="badge" style="background: rgba(244, 63, 94, 0.15); color: #f43f5e; border: 1px solid rgba(244, 63, 94, 0.3); font-size: 0.65rem;">FALHA</span>';
        }

        const bt = (h.benchmark_type || '').toLowerCase();
        let typeBadge = '';
        if (bt.includes('hardeval') || bt.includes('hardware') || bt.includes('profile')) {
          typeBadge = '<span class="badge" style="background: rgba(16, 185, 129, 0.15); color: #34d399; font-size: 0.68rem;">⚡ Velocidade & VRAM</span>';
        } else if (bt.includes('cognitive') || bt.includes('eval') || bt.includes('cot')) {
          typeBadge = '<span class="badge" style="background: rgba(99, 102, 241, 0.15); color: #818cf8; font-size: 0.68rem;">🧠 Avaliação Cognitiva</span>';
        } else if (bt.includes('vision') || bt.includes('multimodal') || bt.includes('vlm')) {
          typeBadge = '<span class="badge" style="background: rgba(6, 182, 212, 0.15); color: var(--cyan); font-size: 0.68rem;">👁️ Visão Multimodal</span>';
        } else if (bt.includes('speculative') || bt.includes('draft')) {
          typeBadge = '<span class="badge" style="background: rgba(245, 158, 11, 0.15); color: #fbbf24; font-size: 0.68rem;">🚀 Especulativa</span>';
        } else if (bt.includes('context') || bt.includes('pressure') || bt.includes('needle')) {
          typeBadge = '<span class="badge" style="background: rgba(244, 63, 94, 0.15); color: #fb7185; font-size: 0.68rem;">📦 Pressão Contexto</span>';
        } else {
          typeBadge = `<span class="badge" style="background: rgba(168, 85, 247, 0.15); color: #c084fc; font-size: 0.68rem;">🔬 ${h.benchmark_type || 'Geral'}</span>`;
        }

        return `
          <tr>
            <td class="text-center mono" style="color: var(--text-muted); font-size: 0.72rem;">#${h.id}</td>
            <td>
              <div style="font-weight: 600; color: #fff; font-size: 0.8rem;">${h.model_id}</div>
            </td>
            <td>${typeBadge}</td>
            <td class="text-center mono" style="font-size: 0.72rem; color: var(--cyan);">${h.engine}</td>
            <td class="text-center mono" style="color: var(--emerald); font-weight: 600;">${h.tps > 0 ? h.tps.toFixed(1) + ' <span style="font-size: 0.65rem; color: var(--text-muted);">tps</span>' : '--'}</td>
            <td class="text-center mono">${h.ttft_ms > 0 ? h.ttft_ms.toFixed(0) + ' <span style="font-size: 0.65rem; color: var(--text-muted);">ms</span>' : '--'}</td>
            <td class="text-center mono" style="color: #f43f5e;">${h.vram_peak_mb > 0 ? h.vram_peak_mb.toFixed(0) + ' <span style="font-size: 0.65rem; color: var(--text-muted);">MB</span>' : '--'}</td>
            <td class="text-center">${statusBadge}</td>
            <td class="mono" style="font-size: 0.7rem; color: var(--text-muted);">${h.created_at || '--'}</td>
          </tr>
        `;
      }).join('');
    }

    async function executeSelectedArenaBenchmark() {
      const modelId = document.getElementById('arena-runner-model-select')?.value;
      if (!modelId) {
        showToast('Selecione um modelo para executar a prova.', 'warning');
        return;
      }

      const benchType = parseInt(document.getElementById('arena-runner-type-select')?.value || '1', 10);
      const engine = document.getElementById('arena-runner-engine-select')?.value || null;

      await runSpecificArenaBenchmark(modelId, benchType, engine);
    }

    async function runSpecificArenaBenchmark(modelId, benchType = 1, engine = null) {
      const btn = document.getElementById('btn-run-arena-benchmark');
      const feedback = document.getElementById('arena-live-feedback');
      const titleEl = document.getElementById('arena-live-title');
      const verdictEl = document.getElementById('arena-live-verdict-badge');
      const gridEl = document.getElementById('arena-live-metrics-grid');

      if (btn) {
        btn.disabled = true;
        btn.innerHTML = '<span style="display:inline-block; animation:spin 1s linear infinite; margin-right:6px;">⏳</span> Executando Prova...';
      }

      if (feedback) {
        feedback.style.display = 'block';
        if (titleEl) titleEl.innerText = `Executando Prova no Silício para ${modelId}...`;
        if (verdictEl) verdictEl.innerHTML = '<span class="badge" style="background: rgba(6, 182, 212, 0.2); color: var(--cyan);">NVML Watchdog Ativo</span>';
        if (gridEl) gridEl.innerHTML = '<div style="color: var(--text-muted); font-size: 0.75rem; grid-column: 1 / -1;"><span style="display:inline-block; animation:spin 1s linear infinite; margin-right:6px;">⚙️</span> Carregando tensores via memmap2 e medindo telemetria bare-metal...</div>';
      }

      try {
        const payload = {
          model_id: modelId,
          benchmark_type: benchType,
          engine: engine
        };

        const res = await fetch('/api/v1/arena/benchmark/run', {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify(payload)
        });

        const data = await res.json();

        if (res.ok && data.status === 'success') {
          showToast(`Prova ${data.benchmark_type} concluída no silício com sucesso!`, 'success');
          if (titleEl) titleEl.innerText = `Resultado da Prova: ${data.model_id} (${data.benchmark_type})`;
          if (verdictEl) verdictEl.innerHTML = `<span class="badge badge-champion">${data.metrics?.verdict || 'NOMINAL'}</span>`;

          if (gridEl && data.metrics) {
            let cardsHtml = '';
            // Type 1: HardwareProfile
            if (data.metrics.tps !== undefined) {
              cardsHtml += `
                <div class="hw-spec-card">
                  <div class="hw-spec-label">Throughput Real</div>
                  <div class="hw-spec-val" style="color: var(--emerald);">${data.metrics.tps.toFixed(1)} <span style="font-size: 0.7rem;">tok/s</span></div>
                </div>
              `;
            }
            if (data.metrics.ttft_ms !== undefined) {
              cardsHtml += `
                <div class="hw-spec-card">
                  <div class="hw-spec-label">Tempo Resposta (TTFT)</div>
                  <div class="hw-spec-val" style="color: var(--cyan);">${data.metrics.ttft_ms.toFixed(0)} <span style="font-size: 0.7rem;">ms</span></div>
                </div>
              `;
            }
            if (data.metrics.peak_vram_mb !== undefined) {
              cardsHtml += `
                <div class="hw-spec-card">
                  <div class="hw-spec-label">VRAM Pico NVML</div>
                  <div class="hw-spec-val" style="color: #f43f5e;">${data.metrics.peak_vram_mb.toFixed(0)} <span style="font-size: 0.7rem;">MB</span></div>
                </div>
              `;
            }
            if (data.metrics.ram_mb !== undefined) {
              cardsHtml += `
                <div class="hw-spec-card">
                  <div class="hw-spec-label">RAM Host</div>
                  <div class="hw-spec-val">${data.metrics.ram_mb.toFixed(0)} <span style="font-size: 0.7rem;">MB</span></div>
                </div>
              `;
            }
            if (data.metrics.e3_score !== undefined) {
              cardsHtml += `
                <div class="hw-spec-card">
                  <div class="hw-spec-label">Eficiência E³</div>
                  <div class="hw-spec-val" style="color: #a855f7;">${data.metrics.e3_score > 9000 ? '9999' : data.metrics.e3_score.toFixed(1)}</div>
                </div>
              `;
            }

            // Type 2: CognitiveEval
            if (data.metrics.accuracy_pct !== undefined) {
              cardsHtml += `
                <div class="hw-spec-card">
                  <div class="hw-spec-label">Acurácia Sintática</div>
                  <div class="hw-spec-val" style="color: var(--emerald);">${data.metrics.accuracy_pct.toFixed(1)}%</div>
                </div>
              `;
            }
            if (data.metrics.avg_tps !== undefined) {
              cardsHtml += `
                <div class="hw-spec-card">
                  <div class="hw-spec-label">Throughput Médio</div>
                  <div class="hw-spec-val" style="color: var(--emerald);">${data.metrics.avg_tps.toFixed(1)} <span style="font-size: 0.7rem;">tok/s</span></div>
                </div>
              `;
            }

            // Type 3: MultimodalVision
            if (data.metrics.multimodal_ttft_ms !== undefined) {
              cardsHtml += `
                <div class="hw-spec-card">
                  <div class="hw-spec-label">Latência Visão VLM</div>
                  <div class="hw-spec-val" style="color: var(--cyan);">${data.metrics.multimodal_ttft_ms.toFixed(0)} <span style="font-size: 0.7rem;">ms</span></div>
                </div>
              `;
            }
            if (data.metrics.projection_latency_ms !== undefined) {
              cardsHtml += `
                <div class="hw-spec-card">
                  <div class="hw-spec-label">Projeção mmproj</div>
                  <div class="hw-spec-val" style="color: #a855f7;">${data.metrics.projection_latency_ms.toFixed(0)} <span style="font-size: 0.7rem;">ms</span></div>
                </div>
              `;
            }

            // Type 4: SpeculativeDraft
            if (data.metrics.speedup_ratio !== undefined) {
              cardsHtml += `
                <div class="hw-spec-card">
                  <div class="hw-spec-label">Ganho de Velocidade</div>
                  <div class="hw-spec-val" style="color: var(--emerald);">${data.metrics.speedup_ratio.toFixed(2)}x</div>
                </div>
              `;
            }
            if (data.metrics.acceptance_rate_alpha !== undefined) {
              cardsHtml += `
                <div class="hw-spec-card">
                  <div class="hw-spec-label">Taxa Alpha (Aceite)</div>
                  <div class="hw-spec-val" style="color: var(--cyan);">${(data.metrics.acceptance_rate_alpha * 100).toFixed(1)}%</div>
                </div>
              `;
            }

            // Type 5: ContextPressure
            if (data.metrics.max_effective_context !== undefined) {
              cardsHtml += `
                <div class="hw-spec-card">
                  <div class="hw-spec-label">Janela Máxima Efetiva</div>
                  <div class="hw-spec-val" style="color: var(--cyan);">${Number(data.metrics.max_effective_context).toLocaleString()} <span style="font-size: 0.7rem;">tokens</span></div>
                </div>
              `;
            }
            if (data.metrics.degradation_detected !== undefined) {
              cardsHtml += `
                <div class="hw-spec-card">
                  <div class="hw-spec-label">Degradação</div>
                  <div class="hw-spec-val" style="color: ${data.metrics.degradation_detected ? '#f43f5e' : 'var(--emerald)'};">${data.metrics.degradation_detected ? 'Detectada ⚠️' : 'Zero Queda ✅'}</div>
                </div>
              `;
            }

            // Ação Direta no Feedback: Usar no Hermes
            cardsHtml += `
              <div style="grid-column: 1 / -1; margin-top: 10px; display: flex; justify-content: flex-end; gap: 8px;">
                <button class="btn btn-green" onclick="openQuickApplyModal('${data.model_id}', 'local_silicon', true)" style="font-size: 0.76rem; font-weight: 600; padding: 6px 14px;">
                  <span>⚡</span> Usar este Modelo no Hermes Agent
                </button>
              </div>
            `;

            gridEl.innerHTML = cardsHtml;
          }

          // Atualiza tabelas
          await fetchLocalArenaData();
        } else {
          showToast(data.message || 'Falha ao executar prova no silício.', 'error');
          if (titleEl) titleEl.innerText = 'Falha na Prova de Silício';
          if (verdictEl) verdictEl.innerHTML = '<span class="badge" style="background: rgba(244, 63, 94, 0.2); color: #f43f5e;">ERRO</span>';
          if (gridEl) gridEl.innerHTML = `<div style="color: #f43f5e; font-size: 0.8rem; grid-column: 1 / -1;">${data.message || 'Erro desconhecido'}</div>`;
        }
      } catch (err) {
        console.error('Erro ao executar benchmark:', err);
        showToast('Erro de comunicação ao executar benchmark.', 'error');
      } finally {
        if (btn) {
          btn.disabled = false;
          btn.innerHTML = '<span>⚡</span> Iniciar Teste';
        }
      }
    }

    async function syncLocalArenaHermes() {
      const btn = event?.currentTarget;
      if (btn) {
        btn.disabled = true;
        btn.innerHTML = '<span style="display:inline-block; animation:spin 1s linear infinite; margin-right:4px;">⏳</span> Sincronizando...';
      }
      try {
        const res = await fetch('/api/v1/arena/sync', { method: 'POST' });
        if (res.ok) {
          const data = await res.json();
          const count = data.synced_vault_models || 0;
          showToast(`✨ Sincronizado com sucesso! ${count} modelos no Heuristic Vault e links do Hermes ativos.`, 'success');
          await fetchLocalArenaData();
        } else {
          showToast('Falha ao sincronizar links com o Hermes.', 'error');
        }
      } catch (err) {
        console.error('Falha na sincronização:', err);
        showToast('Erro de conexão ao sincronizar com o Hermes.', 'error');
      } finally {
        if (btn) {
          btn.disabled = false;
          btn.innerHTML = '<span>🔗</span> Sincronizar com Hermes Agent';
        }
      }
    }
"""

content = content[:start_idx] + new_arena_js + "\n\n" + content[end_idx:]

with open("crates/souls_server/src/dashboard_ui.html", "w", encoding="utf-8") as f:
    f.write(content)

print("Arena JavaScript updated with sorting, tier badges, and clean filters!")

